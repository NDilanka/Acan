//! Predicate evaluation and the save check for proposed rulings.
//!
//! [`evaluate`] answers one predicate for one contrast. Every answer is a [`CheckResult`], and
//! none passes on missing evidence: a predicate the evidence cannot answer is `Unknown`, one the
//! evidence contradicts is `Fail`, and only a holding predicate passes. A comparison that is
//! `Unknown` makes every predicate of its contrast `Unknown`. A comparison that was not made from
//! this contrast's runs is refused with an error. Values come from the admitted runs, and a
//! `Fail` detail names a path or a count, never a value.
//!
//! [`check_examples`] saves a proposed ruling only when the predicate actually passes on every
//! example it accepts and actually fails on every example it rejects. An unknown on either side
//! is a misjudgement.
//!
//! The rate window ([`Predicate::Window`]) always proves that no attempt beyond the allowance is
//! allowed. The first attempts being allowed, and recovery once a window ends, are proven only
//! when the window lists [`WindowObligation`]s for them. A run that allows nothing is unknown.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::str::FromStr;

use serde_json::Value;

use crate::compare::{
    self, CompareError, Contrast, RunContext, flatten, matches_pattern, overlaps, sink_name,
    text_has_format,
};
use crate::expr::{
    BaseAnchor, Boundary, CompareOp, EventFilter, GroupBy, Operand, Predicate, QuotaCounting,
    RejectedAttempts, RunSelector, SchemaType, ValueRef, ValueSchema, WindowAnchor,
    WindowObligation, WindowPolicy,
};
use crate::hash::{CanonicalError, ContrastExampleHash};
use crate::observation::{ObservedRun, RequestRecord};
use crate::probe::CaptureOutcome;
use crate::provenance::CoreDerived;
use crate::scope::Principal;
use crate::types::{
    Evidence, HttpMethod, MethodSelector, NonEmpty, ObservationPath, PathPattern, Scalar, StepId,
};
use crate::verdict::{CheckResult, Comparison, Outcome, UnknownReason};

/// Why a predicate could not be evaluated, or a decision does not hold up. Never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PredicateError {
    /// The predicate's inputs do not belong together.
    #[error("the predicate's inputs do not belong together: {detail}")]
    Mismatch {
        /// What does not match.
        detail: String,
    },
    /// A run could not be normalized.
    #[error(transparent)]
    Compare(#[from] CompareError),
    /// A predicate or decision names an example that is not listed.
    #[error("the example {example} is not among the listed examples")]
    NotListed {
        /// The example.
        example: ContrastExampleHash,
    },
    /// An example is listed as both accepted and rejected.
    #[error("the example {example} is both accepted and rejected")]
    Ambiguous {
        /// The example.
        example: ContrastExampleHash,
    },
    /// The predicate does not give the answer the decision lists for an example.
    #[error("the predicate does not give the listed answer on the example {example}")]
    Misjudged {
        /// The example.
        example: ContrastExampleHash,
        /// Whether the example is accepted (`true`) or rejected (`false`).
        accepted: bool,
        /// What the predicate gave.
        actual: CheckResult,
    },
}

/// A retained ruling example, rebuilt from storage and checked against its manifest by
/// [`crate::admit::retained_example`]. Nothing else constructs one.
#[derive(Debug, Clone, Copy)]
pub struct RetainedExample<'a> {
    hash: ContrastExampleHash,
    contrast: Contrast<'a>,
    comparison: &'a CoreDerived<Comparison>,
}

impl<'a> RetainedExample<'a> {
    pub(crate) fn admitted(
        hash: ContrastExampleHash,
        contrast: Contrast<'a>,
        comparison: &'a CoreDerived<Comparison>,
    ) -> Self {
        Self {
            hash,
            contrast,
            comparison,
        }
    }

    /// The manifest's hash, which a decision lists.
    #[must_use]
    pub fn hash(&self) -> ContrastExampleHash {
        self.hash
    }

    /// The rebuilt contrast.
    #[must_use]
    pub fn contrast(&self) -> &Contrast<'a> {
        &self.contrast
    }

    /// Its comparison.
    #[must_use]
    pub fn comparison(&self) -> &'a CoreDerived<Comparison> {
        self.comparison
    }
}

/// Evaluates `predicate` over one contrast and its comparison.
///
/// An `equals_base` with a [`crate::BaseAnchor::Fixed`] anchor compares the candidate with the
/// base runs of the example in `anchors` whose hash it names; a [`crate::BaseAnchor::Fresh`]
/// anchor uses `contrast`'s own base runs. A fixed anchor missing from `anchors` is
/// [`CheckResult::Unknown`], never a pass and never a silent switch to the fresh base.
///
/// # Errors
///
/// Returns [`PredicateError`] when the comparison is not the one these runs make, or the runs
/// cannot be compared or normalized. A predicate the evidence cannot answer, such as a run that
/// did not complete, is an `Unknown` result, not an error.
pub fn evaluate(
    predicate: &Predicate,
    contrast: &Contrast<'_>,
    comparison: &CoreDerived<Comparison>,
    anchors: &[RetainedExample<'_>],
) -> Result<CheckResult, PredicateError> {
    let comparison_value = comparison.value();
    if comparison_value.probe_hash != contrast.probe.spec.probe_hash().map_err(hashing)? {
        return Err(PredicateError::Mismatch {
            detail: "the comparison is of another probe".to_owned(),
        });
    }
    compare::verify_comparison(contrast, comparison)?;
    if let Outcome::Unknown { reason } = &comparison_value.outcome {
        return Ok(CheckResult::Unknown {
            reason: reason.clone(),
        });
    }
    judge(predicate, contrast, comparison_value, anchors)
}

/// Succeeds only if `predicate` passes on every accepted example and fails on every rejected
/// one, and `ruled_on` is among them and judged the same way. A proposed ruling is saved only
/// after this succeeds; fixed anchors resolve among the accepted and rejected examples.
///
/// # Errors
///
/// Returns [`PredicateError`] when `ruled_on` or a fixed anchor is not listed, an example is
/// listed twice, an example cannot be evaluated, or the predicate misjudges an example. The
/// misjudged variant names the example.
pub fn check_examples(
    predicate: &Predicate,
    ruled_on: &RetainedExample<'_>,
    accepts: &[RetainedExample<'_>],
    rejects: &[RetainedExample<'_>],
) -> Result<(), PredicateError> {
    let listed: Vec<RetainedExample<'_>> = accepts.iter().chain(rejects).copied().collect();
    for (index, example) in listed.iter().enumerate() {
        if listed[..index]
            .iter()
            .any(|other| other.hash == example.hash)
        {
            return Err(PredicateError::Ambiguous {
                example: example.hash,
            });
        }
        if accepts.iter().any(|a| a.hash == example.hash)
            && rejects.iter().any(|r| r.hash == example.hash)
        {
            return Err(PredicateError::Ambiguous {
                example: example.hash,
            });
        }
    }
    for anchor in predicate.fixed_anchors() {
        if !listed.iter().any(|example| example.hash == anchor) {
            return Err(PredicateError::NotListed { example: anchor });
        }
    }
    if !listed.iter().any(|example| example.hash == ruled_on.hash) {
        return Err(PredicateError::NotListed {
            example: ruled_on.hash,
        });
    }
    for (examples, accepted) in [(accepts, true), (rejects, false)] {
        for example in examples {
            let actual = evaluate(predicate, &example.contrast, example.comparison, &listed)?;
            let correct = matches!(
                (&actual, accepted),
                (CheckResult::Pass {}, true) | (CheckResult::Fail { .. }, false)
            );
            if !correct {
                return Err(PredicateError::Misjudged {
                    example: example.hash,
                    accepted,
                    actual,
                });
            }
        }
    }
    Ok(())
}

fn hashing(err: CanonicalError) -> PredicateError {
    PredicateError::Compare(CompareError::from(err))
}

fn judge(
    predicate: &Predicate,
    contrast: &Contrast<'_>,
    comparison: &Comparison,
    anchors: &[RetainedExample<'_>],
) -> Result<CheckResult, PredicateError> {
    match predicate {
        Predicate::EqualsBase {
            step_id,
            path,
            anchor,
        } => equals_base(step_id, path, anchor, contrast, comparison, anchors),
        Predicate::Equals { at, value } => {
            let expected = scalar_value(value);
            let values = match resolve(contrast, at)? {
                Ok(values) => values,
                Err(unknown) => return Ok(unknown),
            };
            Ok(match values.as_slice() {
                [] => fail("the value is absent"),
                [value] if *value == expected => pass(),
                [_] => fail("the value differs from the literal"),
                _ => several(),
            })
        }
        Predicate::StatusIn { step_id, statuses } => Ok(status_in(contrast, step_id, statuses)),
        Predicate::Present { at } => Ok(match resolve(contrast, at)? {
            Ok(values) if values.is_empty() => fail("the value is absent"),
            Ok(_) => pass(),
            Err(unknown) => unknown,
        }),
        Predicate::Absent { at } => Ok(match resolve(contrast, at)? {
            Ok(values) if values.is_empty() => pass(),
            Ok(_) => fail("the value is present"),
            Err(unknown) => unknown,
        }),
        Predicate::MatchesSchema { at, schema } => Ok(match resolve(contrast, at)? {
            Ok(values) if values.is_empty() => fail("the value is absent"),
            Ok(values) if values.iter().all(|value| conforms(value, schema)) => pass(),
            Ok(_) => fail("the value does not match its schema"),
            Err(unknown) => unknown,
        }),
        Predicate::AfterNWithin { .. } => Ok(CheckResult::Unknown {
            reason: UnknownReason::Error {
                detail: "after_n_within is unsupported: the contract names no event filter for it"
                    .to_owned(),
            },
        }),
        Predicate::All { of } => {
            let mut first_fail = None;
            let mut first_unknown = None;
            for part in of.iter() {
                match judge(part, contrast, comparison, anchors)? {
                    CheckResult::Pass {} => {}
                    fail @ CheckResult::Fail { .. } => {
                        first_fail.get_or_insert(fail);
                    }
                    unknown @ CheckResult::Unknown { .. } => {
                        first_unknown.get_or_insert(unknown);
                    }
                }
            }
            Ok(first_fail.or(first_unknown).unwrap_or(pass()))
        }
        Predicate::Compare { left, op, right } => compare_values(contrast, left, *op, right),
        Predicate::Count { filter, op, value } => {
            let matched = matching(run_for(contrast, filter.run), filter).len();
            let Ok(count) = u32::try_from(matched) else {
                return Ok(unknown_incomparable("the count does not fit in a u32"));
            };
            Ok(if holds(*op, count.cmp(value)) {
                pass()
            } else {
                fail(&format!("the count is {count}, outside the bound"))
            })
        }
        Predicate::Window { policy } => Ok(window(contrast, policy)),
    }
}

fn pass() -> CheckResult {
    CheckResult::Pass {}
}

fn fail(detail: &str) -> CheckResult {
    CheckResult::Fail {
        detail: detail.to_owned(),
    }
}

fn unknown_incomparable(detail: impl Into<String>) -> CheckResult {
    CheckResult::Unknown {
        reason: UnknownReason::Incomparable {
            detail: detail.into(),
        },
    }
}

fn several() -> CheckResult {
    unknown_incomparable("the reference names several values, so it cannot be compared")
}

fn run_for<'a>(contrast: &Contrast<'a>, which: RunSelector) -> &'a ObservedRun {
    match which {
        RunSelector::BaseA => contrast.base_a,
        RunSelector::BaseB => contrast.base_b,
        RunSelector::Candidate => contrast.candidate,
    }
}

fn scalar_value(scalar: &Scalar) -> Value {
    match scalar {
        Scalar::Null => Value::Null,
        Scalar::Bool(flag) => Value::Bool(*flag),
        Scalar::Number(number) => Value::Number(number.clone()),
        Scalar::Text(text) => Value::String(text.clone()),
    }
}

/// The values a reference holds: empty when an observed field is absent. An unknown when its
/// step or sink has no observation, or when a capture did not bind: a missing capture is never
/// read as an absent value, so `absent` cannot pass on it.
fn resolve(
    contrast: &Contrast<'_>,
    at: &ValueRef,
) -> Result<Result<Vec<Value>, CheckResult>, PredicateError> {
    match at {
        ValueRef::Field { run, step_id, path } => {
            Ok(match run_for(contrast, *run).observed(step_id, path) {
                None => Err(unknown_incomparable(format!(
                    "the run has no observation at {step_id}"
                ))),
                Some(observed) => Ok(match observed.value() {
                    Evidence::Present { value } => vec![value.clone()],
                    Evidence::Missing {} => Vec::new(),
                }),
            })
        }
        ValueRef::Capture { run, binding } => {
            let run = run_for(contrast, *run);
            let record = run
                .run()
                .captures
                .iter()
                .find(|outcome| outcome.binding() == binding);
            match record {
                None => Ok(Err(unknown_incomparable(format!(
                    "the run has no record for the capture {binding}"
                )))),
                Some(CaptureOutcome::Missing { .. }) => Ok(Err(CheckResult::Unknown {
                    reason: UnknownReason::Incomplete {},
                })),
                Some(CaptureOutcome::Captured { .. }) => {
                    let context = RunContext {
                        probe: contrast.probe,
                        fixture: contrast.fixture,
                        run,
                    };
                    let captured = compare::captured_values(&context)?;
                    Ok(match captured.get(binding) {
                        Some(value) => Ok(vec![value.clone()]),
                        None => Err(unknown_incomparable(format!(
                            "no capture step binds {binding}"
                        ))),
                    })
                }
            }
        }
        ValueRef::Event { run, sink, path } => {
            let run = run_for(contrast, *run);
            let pattern = event_pattern(*sink, path);
            let mut seen = false;
            let mut values = Vec::new();
            for (_, observation) in run.observations() {
                let Some(tree) = observation.channels.side_effects.get(sink) else {
                    continue;
                };
                seen = true;
                let mut nodes = Vec::new();
                flatten(
                    &format!("side_effect.{}", sink_name(*sink)),
                    tree,
                    &mut nodes,
                );
                values.extend(
                    nodes
                        .into_iter()
                        .filter(|(node, _)| matches_pattern(&pattern, node))
                        .map(|(_, value)| value),
                );
            }
            Ok(if seen {
                Ok(values)
            } else {
                Err(unknown_incomparable(format!(
                    "no observation carries the {} sink",
                    sink_name(*sink)
                )))
            })
        }
    }
}

fn event_pattern(sink: crate::types::TrustedSink, path: &PathPattern) -> String {
    let separator = if path.as_str().starts_with('[') {
        ""
    } else {
        "."
    };
    format!(
        "side_effect.{}{separator}{}",
        sink_name(sink),
        path.as_str()
    )
}

fn compare_values(
    contrast: &Contrast<'_>,
    left: &ValueRef,
    op: CompareOp,
    right: &Operand,
) -> Result<CheckResult, PredicateError> {
    let left = match resolve(contrast, left)? {
        Ok(values) => values,
        Err(unknown) => return Ok(unknown),
    };
    let right = match right {
        Operand::Literal { value } => vec![scalar_value(value)],
        Operand::Ref { value_ref } => match resolve(contrast, value_ref)? {
            Ok(values) => values,
            Err(unknown) => return Ok(unknown),
        },
    };
    let (left, right) = match (left.as_slice(), right.as_slice()) {
        ([], _) | (_, []) => return Ok(fail("a compared value is absent")),
        ([left], [right]) => (left, right),
        _ => return Ok(several()),
    };
    let holds_now = match op {
        CompareOp::Eq => left == right,
        CompareOp::Ne => left != right,
        _ => match order(left, right) {
            Some(ordering) => holds(op, ordering),
            None => {
                return Ok(unknown_incomparable(
                    "the two values have no order between them",
                ));
            }
        },
    };
    Ok(if holds_now {
        pass()
    } else {
        fail("the comparison does not hold")
    })
}

fn order(left: &Value, right: &Value) -> Option<Ordering> {
    match (left, right) {
        (Value::Number(x), Value::Number(y)) => x.as_f64()?.partial_cmp(&y.as_f64()?),
        (Value::String(x), Value::String(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

fn holds(op: CompareOp, ordering: Ordering) -> bool {
    match op {
        CompareOp::Eq => ordering == Ordering::Equal,
        CompareOp::Ne => ordering != Ordering::Equal,
        CompareOp::Lt => ordering == Ordering::Less,
        CompareOp::Le => ordering != Ordering::Greater,
        CompareOp::Gt => ordering == Ordering::Greater,
        CompareOp::Ge => ordering != Ordering::Less,
    }
}

fn conforms(value: &Value, schema: &ValueSchema) -> bool {
    if schema.format.is_some() && schema.kind != SchemaType::String {
        return false;
    }
    match schema.kind {
        SchemaType::Null => value.is_null(),
        SchemaType::Boolean => value.is_boolean(),
        SchemaType::Integer => value
            .as_number()
            .is_some_and(|number| number.is_i64() || number.is_u64()),
        SchemaType::Number => value.is_number(),
        SchemaType::String => match value {
            Value::String(text) => schema
                .format
                .is_none_or(|format| text_has_format(text, format)),
            _ => false,
        },
        SchemaType::Array => value.is_array(),
        SchemaType::Object => value.is_object(),
    }
}

fn status_in(contrast: &Contrast<'_>, step: &StepId, statuses: &NonEmpty<u16>) -> CheckResult {
    let Ok(status_path) = ObservationPath::from_str("status") else {
        return unknown_incomparable("the status path is not an observation path");
    };
    let Some(observed) = contrast.candidate.observed(step, &status_path) else {
        return unknown_incomparable(format!("the candidate has no observation at {step}"));
    };
    match observed.value() {
        Evidence::Present {
            value: Value::Number(number),
        } => match number.as_u64().and_then(|code| u16::try_from(code).ok()) {
            Some(code) if statuses.iter().any(|allowed| *allowed == code) => pass(),
            Some(code) => fail(&format!("status {code} is not allowed at {step}")),
            None => fail(&format!("status at {step} is not a status code")),
        },
        Evidence::Present { .. } | Evidence::Missing {} => {
            fail(&format!("there is no status at {step}"))
        }
    }
}

/// `equals_base`: the candidate differs from base nowhere under `step` and `pattern`. Masked
/// fields never appear as differences, so a masked field holds.
fn equals_base(
    step: &StepId,
    pattern: &PathPattern,
    anchor: &BaseAnchor,
    contrast: &Contrast<'_>,
    comparison: &Comparison,
    anchors: &[RetainedExample<'_>],
) -> Result<CheckResult, PredicateError> {
    if !contrast
        .candidate
        .observations()
        .any(|(_, observation)| observation.step_id == *step)
    {
        return Ok(unknown_incomparable(format!(
            "the candidate has no observation at {step}"
        )));
    }
    let owned = match anchor {
        BaseAnchor::Fresh {} => None,
        BaseAnchor::Fixed { example } => {
            let Some(retained) = anchors.iter().find(|retained| retained.hash == *example) else {
                return Ok(unknown_incomparable(
                    "the fixed anchor is not among the retained examples",
                ));
            };
            match anchor_comparison(contrast, retained)? {
                Some(comparison) => Some(comparison),
                None => {
                    return Ok(unknown_incomparable(
                        "the fixed anchor is an example of another probe",
                    ));
                }
            }
        }
    };
    let outcome = owned
        .as_ref()
        .map_or(&comparison.outcome, |owned| &owned.value().outcome);
    Ok(match outcome {
        Outcome::Unchanged {} => pass(),
        Outcome::Changed { differences } => {
            let differs = differences.iter().any(|difference| {
                difference.step_id == *step && overlaps(pattern.as_str(), difference.path.as_str())
            });
            if differs {
                fail(&format!(
                    "the candidate differs from base at {step} {}",
                    pattern.as_str()
                ))
            } else {
                pass()
            }
        }
        Outcome::Unknown { reason } => CheckResult::Unknown {
            reason: reason.clone(),
        },
    })
}

/// The candidate compared with the base runs of a retained example. `None` when the example
/// belongs to another probe.
fn anchor_comparison(
    contrast: &Contrast<'_>,
    retained: &RetainedExample<'_>,
) -> Result<Option<CoreDerived<Comparison>>, PredicateError> {
    let example = retained.contrast();
    if example.probe.spec.probe_hash().map_err(hashing)?
        != contrast.probe.spec.probe_hash().map_err(hashing)?
    {
        return Ok(None);
    }
    let masks = compare::derive_masks(example)?;
    let anchored = Contrast {
        probe: contrast.probe,
        fixture: contrast.fixture,
        base_a: example.base_a,
        base_b: example.base_b,
        candidate: contrast.candidate,
    };
    Ok(Some(compare::compare(&anchored, &masks)?))
}

/// A request the run sent, with its position in the run.
pub(crate) struct Sent<'r> {
    pub(crate) step_id: &'r StepId,
    pub(crate) record: &'r RequestRecord,
    pub(crate) status: Option<u16>,
}

/// Every request the run sent, in sequence order.
pub(crate) fn sent(run: &ObservedRun) -> Vec<Sent<'_>> {
    run.observations()
        .filter_map(|(_, observation)| {
            observation.request.as_ref().map(|record| Sent {
                step_id: &observation.step_id,
                record,
                status: observation.channels.status,
            })
        })
        .collect()
}

/// The requests a filter admits, ordered by when they were sent, then by sequence.
fn matching<'r>(run: &'r ObservedRun, filter: &EventFilter) -> Vec<Sent<'r>> {
    let mut events: Vec<(usize, Sent<'r>)> = run
        .observations()
        .enumerate()
        .filter_map(|(index, (_, observation))| {
            observation.request.as_ref().map(|record| {
                (
                    index,
                    Sent {
                        step_id: &observation.step_id,
                        record,
                        status: observation.channels.status,
                    },
                )
            })
        })
        .filter(|(_, event)| admits(filter, event))
        .collect();
    events.sort_by_key(|(index, event)| (event.record.sent_at_ms, *index));
    events.into_iter().map(|(_, event)| event).collect()
}

fn admits(filter: &EventFilter, event: &Sent<'_>) -> bool {
    method_admits(filter.method, event.record.method)
        && filter
            .route_template
            .as_ref()
            .is_none_or(|route| *route == event.record.route_template)
        && filter.status_in.as_ref().is_none_or(|statuses| {
            event
                .status
                .is_some_and(|status| statuses.iter().any(|code| *code == status))
        })
        && principal_admits(&filter.principal, event.record.actor.as_ref())
        && filter
            .client
            .as_ref()
            .is_none_or(|client| *client == event.record.client)
}

/// Whether a method selector admits a request method.
pub(crate) fn method_admits(selector: MethodSelector, method: HttpMethod) -> bool {
    match selector {
        MethodSelector::Any => true,
        MethodSelector::Get => method == HttpMethod::Get,
        MethodSelector::Head => method == HttpMethod::Head,
        MethodSelector::Post => method == HttpMethod::Post,
        MethodSelector::Put => method == HttpMethod::Put,
        MethodSelector::Patch => method == HttpMethod::Patch,
        MethodSelector::Delete => method == HttpMethod::Delete,
        MethodSelector::Options => method == HttpMethod::Options,
    }
}

/// Whether a principal admits a request, by the actor that sent it.
pub(crate) fn principal_admits(
    principal: &Principal,
    actor: Option<&crate::types::ActorName>,
) -> bool {
    match principal {
        Principal::Any {} => true,
        Principal::Anonymous {} => actor.is_none(),
        Principal::Authenticated {} => actor.is_some(),
        Principal::Actor { name } => actor == Some(name),
    }
}

/// One group's state while a window is evaluated.
struct Open {
    /// Where the group's current window starts; `None` before its first request without an
    /// anchor.
    start: Option<u64>,
    /// Counted requests in the current window.
    count: u32,
    /// Matching requests in the current window, counted or not.
    seen: u32,
    /// A request in an earlier window was rejected and the request that opens the next window
    /// has not come yet.
    awaiting_recovery: bool,
}

impl Open {
    fn new(anchor_at: Option<u64>) -> Self {
        Self {
            start: anchor_at,
            count: 0,
            seen: 0,
            awaiting_recovery: false,
        }
    }
}

/// A bounded rate window. Each group's windows are tumbling: a window opens at its first event
/// (or at the anchor step), lasts `duration_ms`, and the next event after it opens the next
/// window. Within a window the first `allowed_count` counted events are allowed; every event
/// after them must receive a rejected status.
///
/// Without obligations this proves an upper bound only, as [`WindowPolicy`] says: a run that
/// rejects every attempt passes it, so a run with no allowed attempt is unknown rather than a
/// pass. [`WindowObligation::FirstAllowed`] makes the first `allowed_count` requests of every
/// window a fail when rejected. [`WindowObligation::Recovers`] requires that a group which saw a
/// rejection is allowed again by the request that opens its next window: no such request is
/// unknown, and a rejected one fails.
fn window(contrast: &Contrast<'_>, policy: &WindowPolicy) -> CheckResult {
    let run = run_for(contrast, policy.event_filter.run);
    let mut events = matching(run, &policy.event_filter);
    if events.is_empty() {
        return unknown_incomparable("no request in the run matches the window's filter");
    }
    let anchor_at = match &policy.anchor {
        WindowAnchor::FirstMatch {} => None,
        WindowAnchor::Step { step_id } => {
            match sent(run).iter().find(|event| event.step_id == step_id) {
                Some(event) => Some(event.record.sent_at_ms),
                None => {
                    return unknown_incomparable(format!(
                        "the window's anchor step {step_id} sent no request"
                    ));
                }
            }
        }
    };
    if let Some(at) = anchor_at {
        events.retain(|event| event.record.sent_at_ms >= at);
        if events.is_empty() {
            return unknown_incomparable(
                "no request in the run matches the window's filter at or after its anchor",
            );
        }
    }
    let duration = policy.duration_ms.get();
    let allowed = policy.allowed_count.get();
    let first_allowed = policy
        .obligations
        .contains(&WindowObligation::FirstAllowed {});
    let recovers = policy.obligations.contains(&WindowObligation::Recovers {});
    let mut groups: BTreeMap<Option<String>, Open> = BTreeMap::new();
    let mut any_allowed = false;
    let mut any_rejected = false;
    for event in &events {
        let key = group_key(policy.group_by, event.record);
        let open = groups.entry(key).or_insert_with(|| Open::new(anchor_at));
        let at = event.record.sent_at_ms;
        let rejected = is_rejected(&policy.rejected_attempts, event.status);
        if !open
            .start
            .is_some_and(|start| inside(start, at, duration, policy.boundary))
        {
            if recovers && open.awaiting_recovery && rejected {
                return fail(&format!(
                    "the request at {} that opens the next window was rejected",
                    event.step_id
                ));
            }
            open.awaiting_recovery = false;
            open.start = Some(at);
            open.count = 0;
            open.seen = 0;
        }
        if first_allowed && open.seen < allowed && rejected {
            return fail(&format!(
                "a request at {} within the first {allowed} of its window was rejected",
                event.step_id
            ));
        }
        open.seen = open.seen.saturating_add(1);
        any_allowed |= !rejected;
        if rejected {
            any_rejected = true;
            open.awaiting_recovery = true;
        }
        if open.count < allowed {
            if !rejected || policy.rejected_attempts.counted == QuotaCounting::Counted {
                open.count += 1;
            }
        } else if !rejected {
            return fail(&format!(
                "an attempt at {} beyond the {allowed} allowed was not rejected",
                event.step_id
            ));
        }
    }
    if recovers {
        if !any_rejected {
            return unknown_incomparable(
                "no request was rejected, so recovery after the window is not shown",
            );
        }
        if groups.values().any(|open| open.awaiting_recovery) {
            return unknown_incomparable(
                "no request follows a rejected window's end, so recovery is not shown",
            );
        }
    }
    if !any_allowed {
        return unknown_incomparable(
            "every matching attempt was rejected, so the run shows no allowed attempt",
        );
    }
    pass()
}

fn group_key(group: GroupBy, record: &RequestRecord) -> Option<String> {
    match group {
        GroupBy::Client => Some(record.client.as_str().to_owned()),
        GroupBy::Principal | GroupBy::Actor => {
            record.actor.as_ref().map(|actor| actor.as_str().to_owned())
        }
        GroupBy::None => None,
    }
}

fn inside(start: u64, at: u64, duration: u64, boundary: Boundary) -> bool {
    let end = start.saturating_add(duration);
    at >= start
        && match boundary {
            Boundary::Inclusive => at <= end,
            Boundary::Exclusive => at < end,
        }
}

fn is_rejected(rejected: &RejectedAttempts, status: Option<u16>) -> bool {
    status.is_some_and(|code| rejected.status_in.iter().any(|allowed| *allowed == code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::tests::corpus;
    use crate::hash::ContentHash;
    use crate::observation::{Completeness, IncompleteReason, Observation};
    use crate::probe::{ProbeCorpus, Run, RunOutcome, RunRole};

    /// The first probe of the corpus without its relations, so one-observation runs compare.
    fn single_step_corpus() -> ProbeCorpus {
        let mut corpus = corpus();
        corpus.probes[0].normalization.relations.clear();
        corpus
    }

    /// One run of the first probe in `role`, with one observation at s6 that answered `status`
    /// with the body `error`. `edit` may change the run record or the observation.
    fn run_with(
        corpus: &ProbeCorpus,
        role: RunRole,
        status: u16,
        error: &str,
        edit: impl FnOnce(&mut Run, &mut Observation),
    ) -> ObservedRun {
        let probe = &corpus.probes[0];
        let mut run: Run = serde_json::from_str(include_str!("../fixtures/run.json")).expect("run");
        run.id = format!("run-{role:?}").parse().expect("a run id");
        run.role = role;
        run.probe = probe.spec.probe_hash().expect("the probe hashes");
        run.env.fixture_hash = corpus.fixtures[0]
            .fixture_hash()
            .expect("the fixture hashes");
        run.captures = Vec::new();
        run.outcome = RunOutcome::Completed {};
        let mut observation: Observation =
            serde_json::from_str(include_str!("../fixtures/observation.json"))
                .expect("observation");
        observation.run_id = run.id.clone();
        observation.probe = run.probe;
        observation.commit = run.commit.clone();
        observation.step_id = "s6".parse().expect("a step id");
        observation.channels.status = Some(status);
        observation.channels.body = Some(serde_json::json!({ "error": error }));
        edit(&mut run, &mut observation);
        let hash = observation.observation_hash().expect("hash");
        ObservedRun::admitted(run, vec![(hash, observation)])
    }

    /// Base A, base B and the candidate, all answering `status` with `error`.
    fn trio_of(corpus: &ProbeCorpus, status: u16, error: &str) -> [ObservedRun; 3] {
        [
            run_with(corpus, RunRole::BaseA, status, error, |_, _| {}),
            run_with(corpus, RunRole::BaseB, status, error, |_, _| {}),
            run_with(corpus, RunRole::Candidate, status, error, |_, _| {}),
        ]
    }

    fn contrast_of<'a>(corpus: &'a ProbeCorpus, runs: &'a [ObservedRun; 3]) -> Contrast<'a> {
        Contrast {
            probe: &corpus.probes[0],
            fixture: &corpus.fixtures[0],
            base_a: &runs[0],
            base_b: &runs[1],
            candidate: &runs[2],
        }
    }

    /// The comparison the contrast's runs make, under the masks they derive.
    fn comparison_of(contrast: &Contrast<'_>) -> CoreDerived<Comparison> {
        let masks = compare::derive_masks(contrast).expect("the masks derive");
        compare::compare(contrast, &masks).expect("the comparison runs")
    }

    fn example_hash(name: &[u8]) -> ContrastExampleHash {
        ContentHash::of_bytes(name).into()
    }

    fn status_in(statuses: &[u16]) -> Predicate {
        serde_json::from_value(serde_json::json!({
            "predicate": "status_in",
            "step_id": "s6",
            "statuses": statuses,
        }))
        .expect("the predicate decodes")
    }

    #[test]
    fn a_predicate_passes_and_fails_on_the_runs_its_examples_hold() {
        let corpus = single_step_corpus();
        let limited = trio_of(&corpus, 429, "too many attempts");
        let failed = trio_of(&corpus, 401, "invalid credentials");
        let (limited_contrast, failed_contrast) = (
            contrast_of(&corpus, &limited),
            contrast_of(&corpus, &failed),
        );
        let (limited_comparison, failed_comparison) = (
            comparison_of(&limited_contrast),
            comparison_of(&failed_contrast),
        );
        let predicate = status_in(&[429]);
        assert_eq!(
            evaluate(&predicate, &limited_contrast, &limited_comparison, &[]),
            Ok(CheckResult::Pass {})
        );
        assert!(matches!(
            evaluate(&predicate, &failed_contrast, &failed_comparison, &[]),
            Ok(CheckResult::Fail { .. })
        ));
    }

    #[test]
    fn check_examples_saves_a_predicate_that_accepts_and_rejects_as_listed() {
        let corpus = single_step_corpus();
        let limited = trio_of(&corpus, 429, "too many attempts");
        let failed = trio_of(&corpus, 401, "invalid credentials");
        let (limited_contrast, failed_contrast) = (
            contrast_of(&corpus, &limited),
            contrast_of(&corpus, &failed),
        );
        let (limited_comparison, failed_comparison) = (
            comparison_of(&limited_contrast),
            comparison_of(&failed_contrast),
        );
        let accept = RetainedExample::admitted(
            example_hash(b"limited"),
            limited_contrast,
            &limited_comparison,
        );
        let reject =
            RetainedExample::admitted(example_hash(b"failed"), failed_contrast, &failed_comparison);
        assert_eq!(
            check_examples(&status_in(&[429]), &accept, &[accept], &[reject]),
            Ok(())
        );
    }

    #[test]
    fn check_examples_names_the_example_a_predicate_misjudges() {
        let corpus = single_step_corpus();
        let limited = trio_of(&corpus, 429, "too many attempts");
        let also_limited = trio_of(&corpus, 429, "slow down");
        let (limited_contrast, also_contrast) = (
            contrast_of(&corpus, &limited),
            contrast_of(&corpus, &also_limited),
        );
        let (limited_comparison, also_comparison) = (
            comparison_of(&limited_contrast),
            comparison_of(&also_contrast),
        );
        let accept = RetainedExample::admitted(
            example_hash(b"limited"),
            limited_contrast,
            &limited_comparison,
        );
        let wrongly_rejected =
            RetainedExample::admitted(example_hash(b"also"), also_contrast, &also_comparison);
        assert_eq!(
            check_examples(&status_in(&[429]), &accept, &[accept], &[wrongly_rejected]),
            Err(PredicateError::Misjudged {
                example: example_hash(b"also"),
                accepted: false,
                actual: CheckResult::Pass {},
            })
        );
    }

    #[test]
    fn a_fail_on_an_accepted_example_is_misjudged() {
        let corpus = single_step_corpus();
        let failed = trio_of(&corpus, 401, "invalid credentials");
        let failed_contrast = contrast_of(&corpus, &failed);
        let failed_comparison = comparison_of(&failed_contrast);
        let accept =
            RetainedExample::admitted(example_hash(b"failed"), failed_contrast, &failed_comparison);
        assert!(matches!(
            check_examples(&status_in(&[429]), &accept, &[accept], &[]),
            Err(PredicateError::Misjudged {
                example,
                accepted: true,
                actual: CheckResult::Fail { .. },
            }) if example == example_hash(b"failed")
        ));
    }

    #[test]
    fn check_examples_refuses_an_example_listed_twice_or_ruled_on_outside_the_lists() {
        let corpus = single_step_corpus();
        let limited = trio_of(&corpus, 429, "too many attempts");
        let failed = trio_of(&corpus, 401, "invalid credentials");
        let (limited_contrast, failed_contrast) = (
            contrast_of(&corpus, &limited),
            contrast_of(&corpus, &failed),
        );
        let (limited_comparison, failed_comparison) = (
            comparison_of(&limited_contrast),
            comparison_of(&failed_contrast),
        );
        let accept = RetainedExample::admitted(
            example_hash(b"limited"),
            limited_contrast,
            &limited_comparison,
        );
        let reject =
            RetainedExample::admitted(example_hash(b"failed"), failed_contrast, &failed_comparison);
        let elsewhere = RetainedExample::admitted(
            example_hash(b"elsewhere"),
            limited_contrast,
            &limited_comparison,
        );
        let predicate = status_in(&[429]);
        assert_eq!(
            check_examples(&predicate, &accept, &[accept], &[accept]),
            Err(PredicateError::Ambiguous {
                example: example_hash(b"limited")
            })
        );
        assert_eq!(
            check_examples(&predicate, &elsewhere, &[accept], &[reject]),
            Err(PredicateError::NotListed {
                example: example_hash(b"elsewhere")
            })
        );
    }

    #[test]
    fn a_fixed_anchor_that_is_not_listed_is_refused_and_unknown_when_evaluated() {
        let corpus = single_step_corpus();
        let limited = trio_of(&corpus, 429, "too many attempts");
        let contrast = contrast_of(&corpus, &limited);
        let comparison = comparison_of(&contrast);
        let missing = example_hash(b"missing");
        let anchored: Predicate = serde_json::from_value(serde_json::json!({
            "predicate": "equals_base",
            "step_id": "s6",
            "path": "body.error",
            "anchor": { "kind": "fixed", "example": missing },
        }))
        .expect("the predicate decodes");
        assert!(matches!(
            evaluate(&anchored, &contrast, &comparison, &[]),
            Ok(CheckResult::Unknown { .. })
        ));
        let accept = RetainedExample::admitted(example_hash(b"limited"), contrast, &comparison);
        assert_eq!(
            check_examples(&anchored, &accept, &[accept], &[]),
            Err(PredicateError::NotListed { example: missing })
        );
    }

    /// Runs that each leave the predicate unknown: a candidate that timed out, bases that
    /// disagree, a skipped candidate step, and a candidate observation the bases do not have.
    fn unknown_cases(corpus: &ProbeCorpus) -> Vec<(&'static str, [ObservedRun; 3])> {
        let limited = "too many attempts";
        vec![
            (
                "timed out",
                [
                    run_with(corpus, RunRole::BaseA, 429, limited, |_, _| {}),
                    run_with(corpus, RunRole::BaseB, 429, limited, |_, _| {}),
                    run_with(corpus, RunRole::Candidate, 429, limited, |run, _| {
                        run.outcome = RunOutcome::TimedOut {
                            step_id: "s6".parse().expect("a step id"),
                        };
                    }),
                ],
            ),
            (
                "unstable",
                [
                    run_with(corpus, RunRole::BaseA, 429, limited, |_, _| {}),
                    run_with(
                        corpus,
                        RunRole::BaseB,
                        401,
                        "invalid credentials",
                        |_, _| {},
                    ),
                    run_with(corpus, RunRole::Candidate, 429, limited, |_, _| {}),
                ],
            ),
            (
                "skipped",
                [
                    run_with(corpus, RunRole::BaseA, 429, limited, |_, _| {}),
                    run_with(corpus, RunRole::BaseB, 429, limited, |_, _| {}),
                    run_with(
                        corpus,
                        RunRole::Candidate,
                        429,
                        limited,
                        |_, observation| {
                            observation.completeness = Completeness::Incomplete {
                                reasons: NonEmpty::new(
                                    IncompleteReason::StepSkipped {},
                                    std::iter::empty(),
                                ),
                            };
                        },
                    ),
                ],
            ),
            (
                "incomparable",
                [
                    run_with(corpus, RunRole::BaseA, 429, limited, |_, _| {}),
                    run_with(corpus, RunRole::BaseB, 429, limited, |_, _| {}),
                    run_with(
                        corpus,
                        RunRole::Candidate,
                        429,
                        limited,
                        |_, observation| {
                            observation.step_id = "s9".parse().expect("a step id");
                        },
                    ),
                ],
            ),
        ]
    }

    #[test]
    fn an_unknown_example_is_misjudged_on_either_side() {
        let corpus = single_step_corpus();
        let predicate = status_in(&[429]);
        let limited = trio_of(&corpus, 429, "too many attempts");
        let limited_contrast = contrast_of(&corpus, &limited);
        let limited_comparison = comparison_of(&limited_contrast);
        let accept = RetainedExample::admitted(
            example_hash(b"limited"),
            limited_contrast,
            &limited_comparison,
        );
        let failed = trio_of(&corpus, 401, "invalid credentials");
        let failed_contrast = contrast_of(&corpus, &failed);
        let failed_comparison = comparison_of(&failed_contrast);
        let reject =
            RetainedExample::admitted(example_hash(b"failed"), failed_contrast, &failed_comparison);
        for (name, runs) in unknown_cases(&corpus) {
            let contrast = contrast_of(&corpus, &runs);
            let comparison = comparison_of(&contrast);
            let example =
                RetainedExample::admitted(example_hash(name.as_bytes()), contrast, &comparison);
            assert!(
                matches!(
                    check_examples(&predicate, &accept, &[accept], &[example]),
                    Err(PredicateError::Misjudged {
                        accepted: false,
                        actual: CheckResult::Unknown { .. },
                        ..
                    })
                ),
                "an unknown rejected example ({name}) was not misjudged"
            );
            let accepted =
                RetainedExample::admitted(example_hash(name.as_bytes()), contrast, &comparison);
            assert!(
                matches!(
                    check_examples(&predicate, &accepted, &[accepted], &[reject]),
                    Err(PredicateError::Misjudged {
                        accepted: true,
                        actual: CheckResult::Unknown { .. },
                        ..
                    })
                ),
                "an unknown accepted example ({name}) was not misjudged"
            );
        }
    }

    #[test]
    fn a_comparison_these_runs_did_not_make_is_refused() {
        let corpus = single_step_corpus();
        let runs = trio_of(&corpus, 429, "too many attempts");
        let contrast = contrast_of(&corpus, &runs);
        let hand_made = CoreDerived::derived(crate::verdict::tests::unchanged(&corpus));
        assert!(matches!(
            evaluate(&status_in(&[429]), &contrast, &hand_made, &[]),
            Err(PredicateError::Compare(
                compare::CompareError::ForeignComparison { .. }
            ))
        ));
    }
}
