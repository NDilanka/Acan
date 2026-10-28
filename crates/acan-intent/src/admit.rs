//! Admission: the only way stored records become evidence the gate may weigh.
//!
//! Storage (S01) reads observations by hash and decodes them as plain records. Before any of
//! them reaches normalization, masking, comparison, a predicate or a receipt, it passes through
//! a function here, which checks it against the stored hashes and the run record and returns an
//! admitted type with private fields. Nothing else constructs one.
//!
//! What a reached step leaves behind, by kind. This is the one completeness rule for runs,
//! retained examples ([`crate::RunManifest`]) and twin runs:
//! - `http` and `observe_side_effect` steps: exactly one stored observation each, which may be
//!   [`crate::Completeness::Incomplete`] but is never left out.
//! - `capture` steps: exactly one [`crate::CaptureOutcome`] in [`Run::captures`] (captured or
//!   missing) naming the step and its binding, and no observation. The outcome proves the step
//!   was reached.
//! - `advance_time` steps: no record. A clock step is reached when the run completed or stopped
//!   after it.
//!
//! Records follow the sequence (the fixture recipe's steps, then the probe's) in order. Which
//! steps were reached follows from [`RunOutcome`]:
//! - `completed`: every step. A completed run missing any observation or capture outcome is
//!   refused.
//! - `timed_out` and `setup_failed` with a step: every step before the named one, which must be
//!   in the sequence (in the recipe for `setup_failed`). The named step may hold its record; no
//!   later step holds one.
//! - `setup_failed` without a step, and `isolation_unavailable`: no step. The run has no record.
//! - `crashed`, `cancelled` and `error`, which name no step: a prefix of the sequence. Every
//!   step before the last record holds its own.
//!
//! [`retained_example`] returns [`Unsupported`] until P01 implements it.

use crate::compare::Contrast;
use crate::error::Unsupported;
use crate::hash::{CanonicalError, ObservationHash};
use crate::ledger::ContrastExample;
use crate::observation::{Observation, ObservedRun};
use crate::predicate::RetainedExample;
use crate::probe::{CorpusProbe, Fixture, Run, RunOutcome, Step};
use crate::provenance::CoreDerived;
use crate::types::StepId;
use crate::verdict::Comparison;

/// Why stored records were not admitted. Never evidence, never a pass.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum AdmitError {
    /// Not implemented in this build.
    #[error(transparent)]
    Unsupported(#[from] Unsupported),
    /// A record has no canonical JSON form, so it has no hash.
    #[error("a record has no canonical JSON form: {detail}")]
    Unhashable {
        /// The serializer's message.
        detail: String,
    },
    /// The run is not of this probe, or the probe not of this fixture.
    #[error("the run is not of this probe and fixture")]
    WrongSpec,
    /// A stored hash is not its observation's.
    #[error("the observation of step {step_id} is not stored under its own hash")]
    HashMismatch {
        /// The observation's step.
        step_id: StepId,
    },
    /// An observation names another run, commit or probe.
    #[error("the observation of step {step_id} names another run, commit or probe")]
    ForeignObservation {
        /// The observation's step.
        step_id: StepId,
    },
    /// A record is out of sequence order, repeated, or names a step that leaves no such record.
    #[error("the record of step {step_id} is out of sequence")]
    OutOfSequence {
        /// The record's step.
        step_id: StepId,
    },
    /// A step the run reached has no record.
    #[error("step {step_id} was reached but has no record")]
    Unrecorded {
        /// The step.
        step_id: StepId,
    },
    /// A step after the one where the run stopped has a record.
    #[error("step {step_id} has a record after the run stopped")]
    AfterStop {
        /// The step.
        step_id: StepId,
    },
    /// The outcome names a step outside the part of the sequence it can stop in.
    #[error("the run stopped at {step_id}, which is not a step it could stop at")]
    UnknownStop {
        /// The step.
        step_id: StepId,
    },
}

impl From<CanonicalError> for AdmitError {
    fn from(err: CanonicalError) -> Self {
        Self::Unhashable {
            detail: err.to_string(),
        }
    }
}

/// Admits one run and its stored observations as evidence.
///
/// Checks, all of which must hold: `probe` starts from `fixture`; `run.probe` is `probe`'s
/// [`crate::ProbeSpec::probe_hash`] and `run.env.fixture_hash` is `fixture`'s
/// [`Fixture::fixture_hash`]; every `stored` hash equals its observation's
/// [`Observation::observation_hash`]; every observation names `run`'s id, commit and probe; and
/// the observations and [`Run::captures`] are exactly the records the module rule asks of the
/// steps the outcome reached, in sequence order.
///
/// # Errors
///
/// Returns [`AdmitError`] when any check fails.
pub fn observed_run(
    probe: &CorpusProbe,
    fixture: &Fixture,
    run: Run,
    stored: Vec<(ObservationHash, Observation)>,
) -> Result<ObservedRun, AdmitError> {
    if probe.spec.fixture != fixture.id
        || run.probe != probe.spec.probe_hash()?
        || run.env.fixture_hash != fixture.fixture_hash()?
    {
        return Err(AdmitError::WrongSpec);
    }
    for (hash, observation) in &stored {
        let step_id = || observation.step_id.clone();
        if *hash != observation.observation_hash()? {
            return Err(AdmitError::HashMismatch { step_id: step_id() });
        }
        if observation.run_id != run.id
            || observation.commit != run.commit
            || observation.probe != run.probe
        {
            return Err(AdmitError::ForeignObservation { step_id: step_id() });
        }
    }
    let sequence: Vec<&Step> = fixture
        .recipe
        .iter()
        .chain(probe.spec.steps.iter())
        .collect();
    let observed = stored.iter().map(|(_, observation)| &observation.step_id);
    check_records(&sequence, fixture.recipe.len(), &run, observed)?;
    Ok(ObservedRun::admitted(run, stored))
}

/// Which steps of the sequence a run's outcome says were reached.
enum Reached {
    /// Every step.
    All,
    /// Every step before this index; the step at it may hold its record.
    Through(usize),
    /// None.
    Nothing,
    /// A prefix the outcome does not name.
    Prefix,
}

fn reached(
    sequence: &[&Step],
    recipe_len: usize,
    outcome: &RunOutcome,
) -> Result<Reached, AdmitError> {
    let stop_in = |steps: &[&Step], step_id: &StepId| {
        steps
            .iter()
            .position(|step| step.id() == step_id)
            .map(Reached::Through)
            .ok_or_else(|| AdmitError::UnknownStop {
                step_id: step_id.clone(),
            })
    };
    match outcome {
        RunOutcome::Completed {} => Ok(Reached::All),
        RunOutcome::TimedOut { step_id } => stop_in(sequence, step_id),
        RunOutcome::SetupFailed {
            step_id: Some(step_id),
            ..
        } => stop_in(&sequence[..recipe_len], step_id),
        RunOutcome::SetupFailed { step_id: None, .. } | RunOutcome::IsolationUnavailable { .. } => {
            Ok(Reached::Nothing)
        }
        RunOutcome::Crashed { .. } | RunOutcome::Cancelled {} | RunOutcome::Error { .. } => {
            Ok(Reached::Prefix)
        }
    }
}

/// Checks the module's completeness rule over the observed steps and the run's captures.
fn check_records<'a>(
    sequence: &[&Step],
    recipe_len: usize,
    run: &Run,
    observed: impl Iterator<Item = &'a StepId>,
) -> Result<(), AdmitError> {
    let mut observed = observed.peekable();
    let mut captures = run.captures.iter().peekable();
    let mut first_gap = None;
    let mut last_record = None;
    for (index, step) in sequence.iter().enumerate() {
        let recorded = match step {
            Step::Http(_) | Step::ObserveSideEffect(_) => {
                observed.next_if(|id| *id == step.id()).is_some()
            }
            Step::Capture(capture) => captures
                .next_if(|outcome| {
                    outcome.step_id() == &capture.id && outcome.binding() == &capture.binding
                })
                .is_some(),
            Step::AdvanceTime(_) => continue,
        };
        if !recorded {
            first_gap.get_or_insert(index);
        } else if let Some(gap) = first_gap {
            return Err(AdmitError::Unrecorded {
                step_id: sequence[gap].id().clone(),
            });
        } else {
            last_record = Some(index);
        }
    }
    let stray = observed
        .next()
        .or_else(|| captures.next().map(|outcome| outcome.step_id()));
    if let Some(step_id) = stray {
        return Err(AdmitError::OutOfSequence {
            step_id: step_id.clone(),
        });
    }
    let unrecorded = |gap: usize| AdmitError::Unrecorded {
        step_id: sequence[gap].id().clone(),
    };
    let after_stop = |last: usize| AdmitError::AfterStop {
        step_id: sequence[last].id().clone(),
    };
    match reached(sequence, recipe_len, &run.outcome)? {
        Reached::All => first_gap.map_or(Ok(()), |gap| Err(unrecorded(gap))),
        Reached::Through(stop) => match (first_gap, last_record) {
            (Some(gap), _) if gap < stop => Err(unrecorded(gap)),
            (_, Some(last)) if last > stop => Err(after_stop(last)),
            _ => Ok(()),
        },
        Reached::Nothing => last_record.map_or(Ok(()), |last| Err(after_stop(last))),
        Reached::Prefix => Ok(()),
    }
}

/// Admits a retained ruling example: the contrast rebuilt from its runs, checked against the
/// manifest it was retained under.
///
/// Checks that `contrast`'s probe and fixture are the manifest's, that each admitted run's
/// record and observation hashes are the manifest's for its role, and that `comparison` is the
/// comparison of exactly that contrast. The example's identity is
/// [`ContrastExample::contrast_example_hash`].
///
/// # Errors
///
/// Returns [`AdmitError`] when the contrast does not match the manifest.
pub fn retained_example<'a>(
    _manifest: &ContrastExample,
    _contrast: Contrast<'a>,
    _comparison: &'a CoreDerived<Comparison>,
) -> Result<RetainedExample<'a>, AdmitError> {
    Err(Unsupported("P01: admit::retained_example").into())
}
