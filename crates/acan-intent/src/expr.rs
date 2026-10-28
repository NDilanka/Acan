//! The predicate AST the core evaluates at the gate.
//!
//! Predicates are data, never code. Relations such as token identity, audit joins, shared-user
//! quotas and recovery are expressed with `compare`, `count` and `window`, not by masking
//! literals. A behavior that no predicate can express is not enforced, and an unexpressible
//! required behavior blocks automatic land.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize};

use crate::hash::{CanonicalError, ContentHash, PredicateHash};
use crate::scope::Principal;
use crate::types::{
    CaptureName, ClientName, InvalidValue, MethodSelector, NonEmpty, ObservationPath, PathPattern,
    RouteTemplate, Scalar, StepId, TrustedSink,
};

/// Defines an integer newtype bounded at decode time.
macro_rules! bounded {
    ($(#[$meta:meta])* $name:ident($inner:ty), $min:literal ..= $max:literal, $what:literal) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
        #[serde(transparent)]
        pub struct $name($inner);

        impl $name {
            /// The smallest accepted value.
            pub const MIN: $inner = $min;
            /// The largest accepted value.
            pub const MAX: $inner = $max;

            /// The value.
            #[must_use]
            pub fn get(self) -> $inner {
                self.0
            }
        }

        impl TryFrom<$inner> for $name {
            type Error = InvalidValue;

            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                if (Self::MIN..=Self::MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(InvalidValue::Malformed {
                        what: $what,
                        raw: value.to_string(),
                    })
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::try_from(<$inner>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({ "type": "integer", "minimum": $min, "maximum": $max })
            }
        }
    };
}

bounded!(
    /// A window length in milliseconds of the controlled clock: 1 ms to one day.
    WindowMs(u64), 1 ..= 86_400_000, "window length in ms"
);
bounded!(
    /// How many matching events a window allows: 0 to 10,000.
    AllowedCount(u32), 0 ..= 10_000, "allowed count"
);

/// Which run of a contrast a value comes from.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RunSelector {
    /// The first base run.
    BaseA,
    /// The second base run.
    BaseB,
    /// The candidate run.
    Candidate,
}

/// An address of one value the core reads while evaluating a predicate.
#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(tag = "ref", rename_all = "snake_case", deny_unknown_fields)]
pub enum ValueRef {
    /// An observed field of one step.
    Field {
        /// The run.
        run: RunSelector,
        /// The step.
        step_id: StepId,
        /// The field.
        path: ObservationPath,
    },
    /// A run-local capture.
    Capture {
        /// The run.
        run: RunSelector,
        /// The binding.
        binding: CaptureName,
    },
    /// A field of a trusted side-effect event.
    Event {
        /// The run.
        run: RunSelector,
        /// The sink.
        sink: TrustedSink,
        /// The field, with `[*]` for every event.
        path: PathPattern,
    },
}

/// The right side of a comparison.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operand", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operand {
    /// Another value.
    Ref {
        /// The value.
        value_ref: ValueRef,
    },
    /// A literal from a human spec. Never from a model.
    Literal {
        /// The literal.
        value: Scalar,
    },
}

/// A comparison operator.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Less than.
    Lt,
    /// Less than or equal.
    Le,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Ge,
}

/// A JSON type a field must have.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SchemaType {
    /// `null`.
    Null,
    /// A boolean.
    Boolean,
    /// An integer.
    Integer,
    /// Any number.
    Number,
    /// A string.
    String,
    /// An array.
    Array,
    /// An object.
    Object,
}

/// A string format a field must have.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum StringFormat {
    /// An RFC 4122 UUID.
    Uuid,
    /// An RFC 3339 timestamp.
    DateTime,
    /// An absolute URI.
    Uri,
    /// Lowercase hex digits.
    Hex,
    /// A URL-safe short code of letters and digits.
    ShortCode,
}

/// The shape a field must have. A closed schema, not JSON Schema.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValueSchema {
    /// The type.
    #[serde(rename = "type")]
    pub kind: SchemaType,
    /// The string format, for `string` only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<StringFormat>,
}

/// Which request events a `count` or a `window` counts.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventFilter {
    /// The run whose events are counted.
    pub run: RunSelector,
    /// The method.
    pub method: MethodSelector,
    /// The route, or every route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_template: Option<RouteTemplate>,
    /// The response statuses, or every status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_in: Option<NonEmpty<u16>>,
    /// Whose requests.
    pub principal: Principal,
    /// Which client, or every client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<ClientName>,
}

/// What a window groups events by.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    /// One window per client identity.
    Client,
    /// One window per session principal.
    Principal,
    /// One window per fixture actor.
    Actor,
    /// One window for all events.
    None,
}

/// Where a window starts.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WindowAnchor {
    /// At the first matching event of each group.
    FirstMatch,
    /// At one step.
    Step {
        /// The step.
        step_id: StepId,
    },
}

/// Whether an event exactly at the window's end is inside it.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    /// Inside.
    Inclusive,
    /// Outside.
    Exclusive,
}

/// Whether rejected attempts still use up the quota.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum QuotaCounting {
    /// Rejected attempts count toward the quota.
    Counted,
    /// They do not.
    NotCounted,
}

/// How attempts beyond the allowed count must be answered.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RejectedAttempts {
    /// The statuses a rejected attempt must get, such as 429.
    pub status_in: NonEmpty<u16>,
    /// Whether rejected attempts count toward the quota.
    pub counted: QuotaCounting,
}

/// A bounded rate window: at most `allowed_count` matching events per group within
/// `duration_ms`, and every attempt beyond it rejected as `rejected_attempts` says.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WindowPolicy {
    /// Which events count.
    pub event_filter: EventFilter,
    /// What they are grouped by.
    pub group_by: GroupBy,
    /// Where each window starts.
    pub anchor: WindowAnchor,
    /// How long a window lasts.
    pub duration_ms: WindowMs,
    /// How many events a window allows.
    pub allowed_count: AllowedCount,
    /// Whether the end instant is inside.
    pub boundary: Boundary,
    /// How excess attempts must be answered.
    pub rejected_attempts: RejectedAttempts,
}

/// A predicate over the observations of one contrast.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "predicate", rename_all = "snake_case", deny_unknown_fields)]
pub enum Predicate {
    /// The candidate equals base at these fields, after normalization and masks.
    EqualsBase {
        /// The step.
        step_id: StepId,
        /// The fields.
        path: PathPattern,
    },
    /// A value equals a literal.
    Equals {
        /// The value.
        at: ValueRef,
        /// The literal.
        value: Scalar,
    },
    /// A step's HTTP status is one of these.
    StatusIn {
        /// The step.
        step_id: StepId,
        /// The allowed statuses.
        statuses: NonEmpty<u16>,
    },
    /// A value is present.
    Present {
        /// The value.
        at: ValueRef,
    },
    /// A value is absent.
    Absent {
        /// The value.
        at: ValueRef,
    },
    /// A value has a shape.
    MatchesSchema {
        /// The value.
        at: ValueRef,
        /// The shape.
        schema: ValueSchema,
    },
    /// After the first `n` matching requests within `window_ms`, `then` holds.
    AfterNWithin {
        /// How many requests come first.
        n: AllowedCount,
        /// Within how long.
        window_ms: WindowMs,
        /// What holds afterwards.
        then: Box<Predicate>,
    },
    /// Every predicate holds.
    All {
        /// The predicates.
        of: NonEmpty<Predicate>,
    },
    /// A relation between a value and another value or a literal.
    Compare {
        /// The left value.
        left: ValueRef,
        /// The operator.
        op: CompareOp,
        /// The right value.
        right: Operand,
    },
    /// The number of matching events compares to a bound.
    Count {
        /// Which events.
        filter: EventFilter,
        /// The operator.
        op: CompareOp,
        /// The bound.
        value: u32,
    },
    /// A bounded rate window holds.
    Window {
        /// The window.
        policy: WindowPolicy,
    },
}

impl Predicate {
    /// The predicate's identity.
    ///
    /// # Errors
    ///
    /// Returns [`CanonicalError`] only if serialization fails.
    pub fn predicate_hash(&self) -> Result<PredicateHash, CanonicalError> {
        ContentHash::of(self).map(PredicateHash::from)
    }
}

/// A predicate a model proposed for a residue, with its reason. The core saves it only after a
/// human accepts it and it accepts every accepted and rejects every rejected observation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PredicateProposal {
    /// The proposed predicate.
    pub predicate: Predicate,
    /// Why, in one or two sentences.
    pub rationale: String,
}
