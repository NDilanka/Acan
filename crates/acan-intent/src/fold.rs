//! Replays the ruling log into derived state, with no model call.
//!
//! The log rules in [`crate::ledger`] are enforced here: sequence numbers run 1, 2, 3 with no
//! gaps, each event chains to the one before it, a byte-identical repeat of the head is a
//! no-op, and every other reused sequence number is a conflict. A supersession names an earlier
//! event of the same lineage that is the lineage's current head, and is refused if it
//! supersedes again, names another lineage, or names itself or a later event.
//!
//! Derived state never reads the corpus, so an expectation is only ever `active` or
//! `superseded` here. `stale` is decided where the corpus is known.

use std::collections::BTreeMap;

use crate::gate::LedgerHead;
use crate::hash::{CanonicalError, LedgerHash};
use crate::ledger::{Expectation, ExpectationStatus, FoldedLedger, LedgerEvent, RulingNote};
use crate::provenance::CoreDerived;
use crate::types::RulingId;

/// Why the log could not be folded. Replay stops; nothing is guessed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum FoldError {
    /// The event's sequence number is not the next one.
    #[error("event {event_seq} is out of sequence; the next event is {expected}")]
    Gap {
        /// The event's sequence number.
        event_seq: u64,
        /// The sequence number the log expected.
        expected: u64,
    },
    /// A different event reuses a sequence number that is already in the log.
    #[error("event {event_seq} reuses a sequence number that is already in the log")]
    Conflict {
        /// The reused sequence number.
        event_seq: u64,
    },
    /// The event's `prev_hash` is not the chain hash of the event before it.
    #[error("event {event_seq} does not chain to the event before it")]
    BrokenChain {
        /// The event.
        event_seq: u64,
    },
    /// The event has no canonical JSON form, so it has no chain hash.
    #[error("event {event_seq} has no canonical JSON form: {detail}")]
    Unhashable {
        /// The event.
        event_seq: u64,
        /// The serializer's message.
        detail: String,
    },
    /// The event supersedes an event that is not in the log.
    #[error("event {event_seq} supersedes event {target}, which is not in the log")]
    UnknownTarget {
        /// The event.
        event_seq: u64,
        /// The replaced event.
        target: u64,
    },
    /// The event supersedes an event that is not strictly earlier than itself, which includes a
    /// cycle.
    #[error("event {event_seq} supersedes event {target}, which is not an earlier event")]
    NotEarlier {
        /// The event.
        event_seq: u64,
        /// The replaced event.
        target: u64,
    },
    /// The event supersedes an event of another ruling lineage.
    #[error("event {event_seq} supersedes event {target} from another ruling lineage")]
    LineageMismatch {
        /// The event.
        event_seq: u64,
        /// The replaced event.
        target: u64,
    },
    /// The replaced event was already superseded.
    #[error("event {event_seq} supersedes event {target}, which is already superseded")]
    AlreadySuperseded {
        /// The event.
        event_seq: u64,
        /// The replaced event.
        target: u64,
    },
    /// The replaced event is not the lineage's current head.
    #[error("event {event_seq} supersedes event {target}, which is not the head of its lineage")]
    NotLineageHead {
        /// The event.
        event_seq: u64,
        /// The replaced event.
        target: u64,
    },
    /// A new event in a lineage with a head does not supersede that head.
    #[error("event {event_seq} starts a lineage whose head {head} it does not supersede")]
    Unsuperseded {
        /// The event.
        event_seq: u64,
        /// The lineage's current head.
        head: u64,
    },
}

/// Folds every event of the log, in order.
///
/// A byte-identical repeat of the head event is skipped. Otherwise each event must be the next
/// one in sequence, chain to the event before it, and satisfy its supersession rule. The result
/// is the derived state the gate reads, with the head it was folded to.
///
/// # Errors
///
/// Returns [`FoldError`] on a gap, a broken chain, a conflicting or invalid supersession, or an
/// event with no canonical form.
pub fn fold(events: &[LedgerEvent]) -> Result<CoreDerived<FoldedLedger>, FoldError> {
    let mut state = State::default();
    for event in events {
        state.apply(event)?;
    }
    Ok(CoreDerived::derived(state.finish()))
}

/// Where an applied event sits in the derived state.
struct Applied {
    ruling_id: Option<RulingId>,
    superseded: bool,
    slot: Slot,
}

/// The output list an applied event feeds.
enum Slot {
    Expectation(usize),
    Note(usize),
    Probe,
}

#[derive(Default)]
struct State {
    applied: BTreeMap<u64, Applied>,
    heads: BTreeMap<RulingId, u64>,
    expectations: Vec<Expectation>,
    notes: Vec<RulingNote>,
    retired_probes: Vec<crate::hash::ProbeHash>,
    last: Option<(LedgerEvent, LedgerHash)>,
}

impl State {
    fn apply(&mut self, event: &LedgerEvent) -> Result<(), FoldError> {
        let seq = event.event_seq();
        let expected = self
            .last
            .as_ref()
            .map_or(1, |(last, _)| last.event_seq() + 1);
        if seq < expected {
            return match &self.last {
                Some((last, _)) if last == event => Ok(()),
                _ => Err(FoldError::Conflict { event_seq: seq }),
            };
        }
        if seq > expected {
            return Err(FoldError::Gap {
                event_seq: seq,
                expected,
            });
        }
        let chain = event.chain_hash().map_err(|err| unhashable(seq, &err))?;
        match (&self.last, event.prev_hash()) {
            (None, None) => {}
            (Some((_, previous)), Some(prev)) if prev == *previous => {}
            _ => return Err(FoldError::BrokenChain { event_seq: seq }),
        }
        let (ruling_id, supersedes) = match event {
            LedgerEvent::DecisionConfirmed(decision) => (
                Some(decision.ruling_id.clone()),
                decision.supersedes.map(|s| s.event_seq),
            ),
            LedgerEvent::NoteRecorded(note) => (
                Some(note.ruling_id.clone()),
                note.supersedes.map(|s| s.event_seq),
            ),
            LedgerEvent::ProbeRetired(_) => (None, None),
        };
        if let Some(target) = supersedes {
            self.supersede(seq, target, ruling_id.as_ref())?;
        } else if let Some(ruling) = &ruling_id
            && let Some(head) = self.heads.get(ruling)
        {
            return Err(FoldError::Unsuperseded {
                event_seq: seq,
                head: *head,
            });
        }
        let slot = match event {
            LedgerEvent::DecisionConfirmed(decision) => {
                self.expectations.push(Expectation {
                    ruling_id: decision.ruling_id.clone(),
                    event_seq: seq,
                    selector: decision.selector.clone(),
                    predicate: decision.predicate_ast.clone(),
                    accepts: decision.accepts.clone(),
                    rejects: decision.rejects.clone(),
                    ruled_on: decision.ruled_on,
                    applicability: decision.applicability,
                    normalization_version: decision.normalization_version.clone(),
                    normalization_hash: decision.normalization_hash,
                    status: ExpectationStatus::Active,
                    label: decision.label.clone(),
                    machine_ids: decision.machine_ids.clone(),
                });
                Slot::Expectation(self.expectations.len() - 1)
            }
            LedgerEvent::NoteRecorded(note) => {
                self.notes.push(RulingNote {
                    ruling_id: note.ruling_id.clone(),
                    event_seq: seq,
                    text: note.text.clone(),
                    reason: note.reason.clone(),
                    superseded: false,
                    machine_ids: note.machine_ids.clone(),
                });
                Slot::Note(self.notes.len() - 1)
            }
            LedgerEvent::ProbeRetired(retired) => {
                if !self.retired_probes.contains(&retired.probe_hash) {
                    self.retired_probes.push(retired.probe_hash);
                }
                Slot::Probe
            }
        };
        if let Some(ruling) = &ruling_id {
            self.heads.insert(ruling.clone(), seq);
        }
        self.applied.insert(
            seq,
            Applied {
                ruling_id,
                superseded: false,
                slot,
            },
        );
        self.last = Some((event.clone(), chain));
        Ok(())
    }

    /// Checks a supersession and marks the replaced event superseded.
    fn supersede(
        &mut self,
        seq: u64,
        target: u64,
        ruling_id: Option<&RulingId>,
    ) -> Result<(), FoldError> {
        if target >= seq {
            return Err(FoldError::NotEarlier {
                event_seq: seq,
                target,
            });
        }
        let Some(replaced) = self.applied.get_mut(&target) else {
            return Err(FoldError::UnknownTarget {
                event_seq: seq,
                target,
            });
        };
        if replaced.ruling_id.as_ref() != ruling_id {
            return Err(FoldError::LineageMismatch {
                event_seq: seq,
                target,
            });
        }
        if replaced.superseded {
            return Err(FoldError::AlreadySuperseded {
                event_seq: seq,
                target,
            });
        }
        let is_head = ruling_id.is_some_and(|ruling| self.heads.get(ruling) == Some(&target));
        if !is_head {
            return Err(FoldError::NotLineageHead {
                event_seq: seq,
                target,
            });
        }
        replaced.superseded = true;
        match replaced.slot {
            Slot::Expectation(index) => {
                self.expectations[index].status = ExpectationStatus::Superseded;
            }
            Slot::Note(index) => self.notes[index].superseded = true,
            Slot::Probe => {}
        }
        Ok(())
    }

    fn finish(self) -> FoldedLedger {
        let head = match self.last {
            Some((last, hash)) => LedgerHead {
                event_seq: last.event_seq(),
                hash: Some(hash),
            },
            None => LedgerHead {
                event_seq: 0,
                hash: None,
            },
        };
        FoldedLedger {
            head,
            expectations: self.expectations,
            notes: self.notes,
            retired_probes: self.retired_probes,
        }
    }
}

fn unhashable(event_seq: u64, err: &CanonicalError) -> FoldError {
    FoldError::Unhashable {
        event_seq,
        detail: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::ContentHash;
    use crate::ledger::{DecisionConfirmed, NoteRecorded};

    const FIXTURES: [&str; 5] = [
        include_str!("../fixtures/ledger-decision.json"),
        include_str!("../fixtures/ledger-note.json"),
        include_str!("../fixtures/ledger-decision-dashboard.json"),
        include_str!("../fixtures/ledger-decision-supersede.json"),
        include_str!("../fixtures/ledger-probe-retired.json"),
    ];

    fn chain() -> Vec<LedgerEvent> {
        FIXTURES
            .iter()
            .map(|text| serde_json::from_str(text).expect("a ledger event"))
            .collect()
    }

    fn decision(events: &[LedgerEvent]) -> DecisionConfirmed {
        match &events[0] {
            LedgerEvent::DecisionConfirmed(decision) => (**decision).clone(),
            _ => panic!("the first fixture is a decision"),
        }
    }

    /// Re-sequences `event` as event `seq` chained to `previous`.
    fn chained(event: LedgerEvent, seq: u64, previous: Option<&LedgerEvent>) -> LedgerEvent {
        let prev = previous.map(|p| p.chain_hash().expect("hashing"));
        match event {
            LedgerEvent::DecisionConfirmed(mut d) => {
                d.event_seq = seq;
                d.prev_hash = prev;
                LedgerEvent::DecisionConfirmed(d)
            }
            LedgerEvent::NoteRecorded(mut n) => {
                n.event_seq = seq;
                n.prev_hash = prev;
                LedgerEvent::NoteRecorded(n)
            }
            LedgerEvent::ProbeRetired(mut p) => {
                p.event_seq = seq;
                p.prev_hash = prev;
                LedgerEvent::ProbeRetired(p)
            }
        }
    }

    #[test]
    fn folding_the_fixture_log_is_deterministic_and_keeps_its_head() {
        let events = chain();
        let first = fold(&events).expect("the fixture log folds");
        let second = fold(&events).expect("the same log folds again");
        assert_eq!(first, second);
        let folded = first.value();
        assert_eq!(folded.head.event_seq, 5);
        assert_eq!(
            folded.head.hash,
            Some(events[4].chain_hash().expect("hashing"))
        );
        let statuses: Vec<_> = folded
            .expectations
            .iter()
            .map(|e| (e.event_seq, e.status))
            .collect();
        assert_eq!(
            statuses,
            vec![
                (1, ExpectationStatus::Superseded),
                (3, ExpectationStatus::Active),
                (4, ExpectationStatus::Active),
            ]
        );
        assert_eq!(folded.notes.len(), 1);
        assert!(!folded.notes[0].superseded);
        assert_eq!(folded.retired_probes.len(), 1);
    }

    #[test]
    fn an_identical_repeat_of_the_head_changes_nothing() {
        let mut events = chain();
        let head = events.last().cloned().expect("a head");
        events.push(head);
        let repeated = fold(&events).expect("a repeat of the head is a duplicate");
        assert_eq!(repeated, fold(&chain()).expect("the log folds"));
    }

    #[test]
    fn a_different_event_at_a_used_sequence_number_is_a_conflict() {
        let mut events = chain();
        let retired = match events.last().expect("a head") {
            LedgerEvent::ProbeRetired(p) => p.clone(),
            _ => panic!("the fixture head is a retirement"),
        };
        let mut other = retired;
        other.reason = "a different reason".to_owned();
        events.push(LedgerEvent::ProbeRetired(other));
        assert_eq!(fold(&events), Err(FoldError::Conflict { event_seq: 5 }));
    }

    #[test]
    fn a_gap_or_a_broken_chain_is_refused() {
        let mut gap = chain();
        gap.remove(1);
        assert_eq!(
            fold(&gap),
            Err(FoldError::Gap {
                event_seq: 3,
                expected: 2
            })
        );
        let mut broken = chain();
        if let LedgerEvent::NoteRecorded(note) = &mut broken[1] {
            note.prev_hash = Some(ContentHash::of_bytes(b"elsewhere").into());
        }
        assert_eq!(fold(&broken), Err(FoldError::BrokenChain { event_seq: 2 }));
    }

    #[test]
    fn a_supersession_of_a_replaced_event_is_refused() {
        let events = chain();
        let mut log = events[..4].to_vec();
        let mut fork = decision(&events[3..4]);
        fork.supersedes = Some(crate::ledger::Supersedes { event_seq: 1 });
        let again = chained(
            LedgerEvent::DecisionConfirmed(Box::new(fork)),
            5,
            log.last(),
        );
        log.push(again);
        assert_eq!(
            fold(&log),
            Err(FoldError::AlreadySuperseded {
                event_seq: 5,
                target: 1
            })
        );
    }

    #[test]
    fn a_supersession_cycle_is_refused() {
        let events = chain();
        let mut self_loop = decision(&events[0..1]);
        self_loop.supersedes = Some(crate::ledger::Supersedes { event_seq: 1 });
        let log = vec![LedgerEvent::DecisionConfirmed(Box::new(self_loop))];
        assert_eq!(
            fold(&log),
            Err(FoldError::NotEarlier {
                event_seq: 1,
                target: 1
            })
        );
    }

    #[test]
    fn a_new_decision_in_a_headed_lineage_must_supersede_it() {
        let events = chain();
        let mut log = events[..1].to_vec();
        let plain = decision(&events[0..1]);
        let second = chained(
            LedgerEvent::DecisionConfirmed(Box::new(plain)),
            2,
            log.last(),
        );
        log.push(second);
        assert_eq!(
            fold(&log),
            Err(FoldError::Unsuperseded {
                event_seq: 2,
                head: 1
            })
        );
    }

    #[test]
    fn a_note_can_be_superseded_by_a_decision_of_its_lineage() {
        let events = chain();
        let note = match &events[1] {
            LedgerEvent::NoteRecorded(note) => note.clone(),
            _ => panic!("the second fixture is a note"),
        };
        let mut lineage = decision(&events[0..1]);
        lineage.ruling_id = note.ruling_id.clone();
        let first = chained(LedgerEvent::NoteRecorded(note.clone()), 1, None);
        let mut successor = lineage.clone();
        successor.supersedes = Some(crate::ledger::Supersedes { event_seq: 1 });
        let log = vec![
            first.clone(),
            chained(
                LedgerEvent::DecisionConfirmed(Box::new(successor)),
                2,
                Some(&first),
            ),
        ];
        let folded = fold(&log).expect("a decision supersedes its lineage's note");
        assert!(folded.value().notes[0].superseded);
        assert_eq!(
            folded.value().expectations[0].status,
            ExpectationStatus::Active
        );
    }

    #[test]
    fn the_empty_log_folds_to_the_empty_head() {
        let folded = fold(&[]).expect("the empty log folds");
        assert_eq!(folded.value().head.event_seq, 0);
        assert_eq!(folded.value().head.hash, None);
        assert!(folded.value().expectations.is_empty());
        assert_eq!(folded.value(), &FoldedLedger::empty());
    }

    #[test]
    fn a_note_event_round_trips_through_the_fold() {
        let note: NoteRecorded = match &chain()[1] {
            LedgerEvent::NoteRecorded(note) => note.clone(),
            _ => panic!("the second fixture is a note"),
        };
        let first = chained(LedgerEvent::NoteRecorded(note.clone()), 1, None);
        let folded = fold(&[first]).expect("one note folds");
        assert_eq!(folded.value().notes[0].text, note.text);
        assert_ne!(ContentHash::of_bytes(b"a"), ContentHash::of_bytes(b"b"));
    }
}
