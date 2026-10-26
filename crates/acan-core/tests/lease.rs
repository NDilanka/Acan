mod common;

use acan_core::{
    AgentCmd, CommitId, DeclarationInput, Event, Fence, Generation, Judgement, LEASE_TTL_MS,
    LeaseEnd, Rejection, Stage, SystemCmd, TaskState, WitnessOutcome,
};
use common::{Fx, events, sym};

#[test]
fn claim_without_matched_readback_is_rejected() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let readback = fx.read_back(&a, t1);
    fx.review(
        readback,
        Judgement::Disagrees {
            reason: "touches login".to_owned(),
        },
    )
    .expect("review accepted");
    let got = fx.agent(&a, AgentCmd::Claim { task: t1 });
    assert_eq!(
        got,
        Err(Rejection::NotMatched {
            failing: fx.witnesses()
        })
    );
    assert_eq!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Open
    );
}

#[test]
fn first_claim_grants_generation_one_in_declaring() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.claim(&a, t1);
    assert_eq!(
        fence,
        Fence {
            task: t1,
            generation: Generation::new(1)
        }
    );
    let TaskState::Claimed { claim, stage } = &fx.origin.world().task(t1).expect("task").state
    else {
        panic!("task is not claimed");
    };
    assert_eq!(*stage, Stage::Declaring);
    assert_eq!(claim.lease.holder, a);
    assert_eq!(claim.lease.expires_at, fx.now.plus_millis(LEASE_TTL_MS));
}

#[test]
fn second_agent_is_told_who_holds_the_task() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    fx.claim(&a, t1);
    let readback = fx.read_back(&b, t1);
    fx.review(readback, Judgement::Match)
        .expect("review accepted");
    let got = fx.agent(&b, AgentCmd::Claim { task: t1 });
    assert_eq!(
        got,
        Err(Rejection::HeldBy {
            holder: a,
            until: fx.now.plus_millis(LEASE_TTL_MS)
        })
    );
}

#[test]
fn repeated_claim_by_the_holder_appends_nothing() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    fx.claim(&a, t1);
    let batch = fx
        .agent(&a, AgentCmd::Claim { task: t1 })
        .expect("idempotent claim");
    assert!(batch.is_empty(), "got {batch:?}");
}

#[test]
fn renew_keeps_generation_and_extends_expiry() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.claim(&a, t1);
    fx.advance(30_000);
    fx.agent(&a, AgentCmd::Renew { fence })
        .expect("renew accepted");
    assert_eq!(fx.fence(t1), Some(fence));
    let TaskState::Claimed { claim, .. } = &fx.origin.world().task(t1).expect("task").state else {
        panic!("task is not claimed");
    };
    assert_eq!(claim.lease.expires_at, fx.now.plus_millis(LEASE_TTL_MS));
}

#[test]
fn claim_after_expiry_ends_old_lease_and_bumps_generation_in_one_batch() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    fx.claim(&a, t1);
    let readback = fx.read_back(&b, t1);
    fx.review(readback, Judgement::Match)
        .expect("review accepted");
    fx.advance(LEASE_TTL_MS);
    let batch = fx
        .agent(&b, AgentCmd::Claim { task: t1 })
        .expect("claim after expiry");
    let got = events(&batch);
    assert!(
        matches!(got[0], Event::LeaseEnded { why: LeaseEnd::Expired, generation, .. } if generation.get() == 1)
    );
    assert!(
        matches!(got[1], Event::LeaseGranted { lease, .. } if lease.generation.get() == 2 && lease.holder == b)
    );
}

#[test]
fn stale_generation_write_changes_nothing() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    let old = fx.claim(&a, t1);
    fx.advance(LEASE_TTL_MS);
    let new = fx.claim(&b, t1);
    let before = fx.origin.world().clone();
    let got = fx.declare(&a, old, DeclarationInput::default());
    assert_eq!(
        got,
        Err(Rejection::StaleGeneration {
            current: Some(new.generation)
        })
    );
    assert_eq!(
        fx.origin.world(),
        &before,
        "a rejected command must not change state"
    );
}

#[test]
fn tick_expires_due_leases_and_reopens_the_task() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    fx.claim(&a, t1);
    fx.advance(LEASE_TTL_MS);
    let batch = fx.system(SystemCmd::Tick).expect("tick accepted");
    assert!(matches!(
        events(&batch)[..],
        [Event::LeaseEnded {
            why: LeaseEnd::Expired,
            ..
        }]
    ));
    assert_eq!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Open
    );
}

#[test]
fn release_reopens_the_task_and_generation_never_resets() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let first = fx.claim(&a, t1);
    fx.agent(&a, AgentCmd::Release { fence: first })
        .expect("release accepted");
    assert_eq!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Open
    );
    let second = fx.claim(&a, t1);
    assert_eq!(second.generation, Generation::new(2));
}

#[test]
fn a_lease_does_not_expire_while_its_gate_runs_and_a_failure_renews_it() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.build(&a, t1);
    let head: CommitId = "abc123".parse().expect("valid commit");
    fx.agent(&a, AgentCmd::ReportBuilt { fence, head })
        .expect("report accepted");
    fx.advance(3 * LEASE_TTL_MS);
    let batch = fx.system(SystemCmd::Tick).expect("tick accepted");
    assert!(batch.is_empty(), "a gating lease expired: {batch:?}");
    let run = match &fx.origin.world().task(t1).expect("task").state {
        TaskState::Claimed {
            stage: Stage::Gating { run, .. },
            ..
        } => *run,
        other => panic!("task is not gating: {other:?}"),
    };
    let witness = fx.witnesses()[0];
    let fail = WitnessOutcome::Fail {
        detail: "login test failed".to_owned(),
    };
    let batch = fx
        .system(SystemCmd::WitnessResult {
            run,
            witness,
            outcome: fail,
        })
        .expect("result accepted");
    let renewed_until = fx.now.plus_millis(LEASE_TTL_MS);
    assert!(
        events(&batch).iter().any(|e| matches!(
            e,
            Event::LeaseRenewed { expires_at, .. } if *expires_at == renewed_until
        )),
        "no renewal on close: {batch:?}"
    );
    assert_eq!(fx.fence(t1), Some(fence), "the worker keeps its fence");
    fx.advance(LEASE_TTL_MS);
    let batch = fx.system(SystemCmd::Tick).expect("tick accepted");
    assert!(
        matches!(
            events(&batch)[..],
            [Event::LeaseEnded {
                why: LeaseEnd::Expired,
                ..
            }]
        ),
        "expiry resumes after the gate closes: {batch:?}"
    );
}

#[test]
fn a_fence_from_another_agent_is_forbidden() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    let fence = fx.claim(&a, t1);
    let got = fx.agent(&b, AgentCmd::Renew { fence });
    assert!(
        matches!(got, Err(Rejection::Forbidden { .. })),
        "got {got:?}"
    );
}

#[test]
fn generation_exhaustion_in_a_stored_log_rejects_the_next_claim() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    fx.claim(&a, t1);
    let max = Generation::new(u32::MAX);
    for stamped in &mut fx.log {
        if let Event::LeaseGranted { task, lease, .. } = &mut stamped.event
            && *task == t1
        {
            lease.generation = max;
        }
    }
    fx.append_stored(Event::LeaseEnded {
        task: t1,
        generation: max,
        why: LeaseEnd::Released,
    })
    .expect("the stored log replays");
    let before = fx.origin.world().clone();
    assert!(matches!(
        fx.agent(&a, AgentCmd::Claim { task: t1 }),
        Err(Rejection::Invalid { .. })
    ));
    assert_eq!(&before, fx.origin.world());
}

#[test]
fn every_stale_fenced_command_rolls_back_the_expiry_it_triggered() {
    let mut fx = Fx::new();
    let (a, b, t1, t2) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1, fx.task2);
    let footprint = || DeclarationInput {
        symbols: vec![sym("src/api/routes.rs", "rate_limit")],
        ..DeclarationInput::default()
    };
    let old = fx.claim(&a, t1);
    let f2 = fx.claim(&b, t2);
    fx.declare(&a, old, footprint()).expect("declare accepted");
    fx.declare(&b, f2, footprint()).expect("declare accepted");
    fx.agent(&a, AgentCmd::Release { fence: old })
        .expect("release accepted");
    fx.claim(&a, t1);
    fx.advance(LEASE_TTL_MS);
    for cmd in [
        AgentCmd::Renew { fence: old },
        AgentCmd::Release { fence: old },
        AgentCmd::Declare {
            fence: old,
            footprint: footprint(),
        },
        AgentCmd::ReportBuilt {
            fence: old,
            head: "stale".parse::<CommitId>().expect("valid commit"),
        },
    ] {
        let before = fx.origin.world().clone();
        let got = fx.agent(&a, cmd);
        assert!(
            matches!(got, Err(Rejection::StaleGeneration { .. })),
            "{got:?}"
        );
        assert_eq!(
            &before,
            fx.origin.world(),
            "a rejected command committed expiry effects"
        );
    }
}
