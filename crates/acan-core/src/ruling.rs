//! The ruling log as the world holds it: every human ruling in log order, and the checks that
//! keep it append-only.
//!
//! ```text
//! RecordRuling(event) by its own actor --> next in the chain              [RulingRecorded]
//!   event_seq = head + 1, prev_hash = head's chain hash
//!   a decision or note opens a lineage, or supersedes that lineage's head by event_seq
//! RecordRuling(the current head again) --> no-op
//! anything else at a used event_seq, a second supersession of one event, or a lineage
//!   continued without naming its head --> refused
//! ```
//!
//! Only a member records a ruling, and only as themselves: no agent command reaches this
//! module, and replay refuses a `RulingRecorded` whose actor is not the member named in it.
//! Retiring a probe narrows the corpus, so only an owner may do it. A note is shown but never
//! counted; only a decision at the head of its lineage is checked.

use std::collections::BTreeMap;

use acan_intent::{LedgerEvent, LedgerHash, LedgerHead, RulingId, RulingVersion, Supersedes};

use crate::event::{Event, Rejection};
use crate::ids::{MemberId, OrderId};
use crate::lane;
use crate::member::Role;
use crate::world::World;

/// Why an event cannot be appended to the ruling log.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LedgerRefusal {
    /// The event does not follow the head.
    #[error("event {got} does not follow the ruling log head {head}")]
    OutOfSequence {
        /// The head's sequence number.
        head: u64,
        /// The event's.
        got: u64,
    },
    /// The event names another predecessor than the head.
    #[error("event {event_seq} does not chain to the ruling log head")]
    BrokenChain {
        /// The event.
        event_seq: u64,
    },
    /// The event continues a lineage without naming its head.
    #[error("ruling {ruling_id} continues without superseding its head {head}")]
    Unsuperseded {
        /// The lineage.
        ruling_id: RulingId,
        /// Its head.
        head: u64,
    },
    /// The event supersedes something other than its lineage's head.
    #[error("ruling {ruling_id} supersedes event {named}, but its head is {head:?}")]
    NotHead {
        /// The lineage.
        ruling_id: RulingId,
        /// What the event names.
        named: u64,
        /// The lineage head, if the lineage exists.
        head: Option<u64>,
    },
    /// The event has no canonical form to hash.
    #[error("event {event_seq} has no canonical JSON form")]
    Unhashable {
        /// The event.
        event_seq: u64,
    },
}

/// Whether an event extends the log or repeats its head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Append {
    /// Byte-identical to the head: nothing to record.
    Duplicate,
    /// The next event.
    Next,
}

/// The ruling log, folded in event order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    events: Vec<LedgerEvent>,
    head: LedgerHead,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            head: LedgerHead {
                event_seq: 0,
                hash: None,
            },
        }
    }
}

fn lineage(event: &LedgerEvent) -> Option<(&RulingId, Option<Supersedes>)> {
    match event {
        LedgerEvent::DecisionConfirmed(decision) => {
            Some((&decision.ruling_id, decision.supersedes))
        }
        LedgerEvent::NoteRecorded(note) => Some((&note.ruling_id, note.supersedes)),
        LedgerEvent::ProbeRetired(_) => None,
    }
}

/// The order a decision or note was ruled in. A retired probe belongs to no order.
pub(crate) fn ruled_in(event: &LedgerEvent) -> Option<&acan_intent::OrderRef> {
    match event {
        LedgerEvent::DecisionConfirmed(decision) => Some(&decision.order),
        LedgerEvent::NoteRecorded(note) => Some(&note.order),
        LedgerEvent::ProbeRetired(_) => None,
    }
}

impl Ledger {
    /// The log revision: the last event and the chain hash through it.
    #[must_use]
    pub fn head(&self) -> LedgerHead {
        self.head
    }

    /// Every event, oldest first.
    #[must_use]
    pub fn events(&self) -> &[LedgerEvent] {
        &self.events
    }

    /// The event in force for every ruling lineage: its latest decision or note.
    #[must_use]
    pub fn lineage_heads(&self) -> BTreeMap<&RulingId, &LedgerEvent> {
        self.events
            .iter()
            .filter_map(|event| lineage(event).map(|(id, _)| (id, event)))
            .collect()
    }

    /// The version of every ruling lineage, in ruling order, as a confirmation binds them.
    #[must_use]
    pub fn ruling_versions(&self) -> Vec<RulingVersion> {
        self.lineage_heads()
            .into_iter()
            .map(|(ruling_id, event)| RulingVersion {
                ruling_id: ruling_id.clone(),
                event_seq: event.event_seq(),
            })
            .collect()
    }

    /// Whether `event` may be appended, by the log rules in [`acan_intent::LedgerEvent`]'s module.
    pub(crate) fn check_append(&self, event: &LedgerEvent) -> Result<Append, LedgerRefusal> {
        let seq = event.event_seq();
        if seq == self.head.event_seq && self.events.last() == Some(event) {
            return Ok(Append::Duplicate);
        }
        if Some(seq) != self.head.event_seq.checked_add(1) {
            return Err(LedgerRefusal::OutOfSequence {
                head: self.head.event_seq,
                got: seq,
            });
        }
        if event.prev_hash() != self.head.hash {
            return Err(LedgerRefusal::BrokenChain { event_seq: seq });
        }
        event
            .chain_hash()
            .map_err(|_| LedgerRefusal::Unhashable { event_seq: seq })?;
        let Some((ruling_id, supersedes)) = lineage(event) else {
            return Ok(Append::Next);
        };
        let head = self.lineage_heads().get(ruling_id).map(|e| e.event_seq());
        match (head, supersedes) {
            (None, None) => Ok(Append::Next),
            (Some(head), Some(named)) if named.event_seq == head => Ok(Append::Next),
            (Some(head), None) => Err(LedgerRefusal::Unsuperseded {
                ruling_id: ruling_id.clone(),
                head,
            }),
            (head, Some(named)) => Err(LedgerRefusal::NotHead {
                ruling_id: ruling_id.clone(),
                named: named.event_seq,
                head,
            }),
        }
    }

    /// Appends an event [`Ledger::check_append`] accepted.
    pub(crate) fn append(&mut self, event: LedgerEvent) {
        let hash: Option<LedgerHash> = event.chain_hash().ok();
        self.head = LedgerHead {
            event_seq: event.event_seq(),
            hash,
        };
        self.events.push(event);
    }
}

/// `RecordRuling`: a member appends one human ruling, signed as themselves. A decision or note
/// belongs to an order, and only that order's poster or an owner rules in it.
pub(crate) fn decide_record(
    world: &World,
    member: &MemberId,
    event: LedgerEvent,
) -> Result<Vec<Event>, Rejection> {
    if event.actor().as_str() != member.as_str() {
        return Err(Rejection::forbidden(
            "a ruling is signed by the member who records it",
        ));
    }
    if matches!(event, LedgerEvent::ProbeRetired(_))
        && world.member(member).is_none_or(|m| m.role != Role::Owner)
    {
        return Err(Rejection::forbidden("only an owner retires a probe"));
    }
    if let Some(order) = ruled_in(&event) {
        let id: OrderId = order
            .as_str()
            .parse()
            .map_err(|_| Rejection::invalid("the ruling names an order this world cannot have"))?;
        let order = world
            .order(id)
            .ok_or_else(|| Rejection::unknown("order", id))?;
        if !lane::may_decide(world, member, order) {
            return Err(Rejection::forbidden(
                "only the poster or an owner rules in an order",
            ));
        }
    }
    match world.ledger().check_append(&event) {
        Ok(Append::Duplicate) => Ok(Vec::new()),
        Ok(Append::Next) => Ok(vec![Event::RulingRecorded {
            event: Box::new(event),
        }]),
        Err(refusal) => Err(Rejection::Invalid {
            why: refusal.to_string(),
        }),
    }
}
