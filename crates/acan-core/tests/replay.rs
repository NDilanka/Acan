mod common;

use acan_core::{
    AgentCmd, CommitId, Event, Explanation, Fence, HumanCmd, Judgement, OrderPhase, Origin,
    Replica, Settlement, Stage, Step, SystemCmd, TaskId, TaskState, WitnessOutcome,
};
use common::{Fx, events, sym};

fn commit(raw: &str) -> CommitId {
    raw.parse().expect("valid commit")
}

fn phase(fx: &Fx) -> OrderPhase {
    fx.origin.world().phase(fx.order).expect("order exists")
}

fn gate_once(
    fx: &mut Fx,
    agent: &acan_core::AgentId,
    fence: Fence,
    head: &str,
    outcome: &WitnessOutcome,
) {
    fx.agent(
        agent,
        AgentCmd::ReportBuilt {
            fence,
            head: commit(head),
        },
    )
    .expect("report accepted");
    assert_eq!(phase(fx), OrderPhase::Gating);
    let run = match &fx.origin.world().task(fence.task).expect("task").state {
        TaskState::Claimed {
            stage: Stage::Gating { run, .. },
            ..
        } => *run,
        other => panic!("not gating: {other:?}"),
    };
    let gate = fx.origin.world().gate(run).expect("gate run").clone();
    for witness in gate.witnesses {
        fx.system(SystemCmd::WitnessResult {
            run,
            witness,
            outcome: outcome.clone(),
        })
        .expect("result accepted");
    }
}

fn land(fx: &mut Fx, task: TaskId, head: &str, merged: &str) {
    fx.system(SystemCmd::LandResult {
        task,
        head: commit(head),
        result: Ok(commit(merged)),
    })
    .expect("land accepted");
}

/// Drives one order through every step of the loop and returns the fixture with its full log.
fn full_loop() -> Fx {
    let mut fx = Fx::new();
    let (owner, a, b, t1, t2) = (
        fx.owner.clone(),
        fx.agent_a.clone(),
        fx.agent_b.clone(),
        fx.task1,
        fx.task2,
    );
    assert_eq!(phase(&fx), OrderPhase::ReadingBack);
    fx.system(SystemCmd::IndexUpdated).expect("first scan");
    fx.system(SystemCmd::ModelUsed {
        model: "muse-spark".to_owned(),
        step: Step::Split,
        input_tokens: 900,
        output_tokens: 120,
    })
    .expect("usage recorded");

    let first = fx.read_back(&a, t1);
    fx.review(first, Judgement::Match).expect("review accepted");
    let unclear = fx.read_back(&b, t2);
    fx.ask(unclear).expect("question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    assert_eq!(phase(&fx), OrderPhase::Asking { question });
    fx.human(
        &owner,
        HumanCmd::AnswerQuestion {
            question,
            option: 0,
        },
    )
    .expect("answer accepted");

    let f1 = fx.claim(&a, t1);
    let f2 = fx.claim(&b, t2);
    let limiter = || acan_core::DeclarationInput {
        symbols: vec![sym("src/api/routes.rs", "rate_limit")],
        ..Default::default()
    };
    fx.declare(&a, f1, limiter()).expect("declare accepted");
    let batch = fx.declare(&b, f2, limiter()).expect("declare accepted");
    assert_eq!(
        events(&batch)
            .iter()
            .filter(|e| matches!(e, Event::ClashFound { .. }))
            .count(),
        1,
        "exactly one clash"
    );
    assert_eq!(phase(&fx), OrderPhase::Clashing);
    let clash = fx.origin.world().clashes().next().expect("clash").id;
    fx.system(SystemCmd::ClashExplained {
        clash,
        explanation: Explanation {
            consequence: "both rewrite the limiter".to_owned(),
            breaks: None,
            suggested: Some(Settlement::Serialize {
                first: t1,
                then: t2,
            }),
        },
    })
    .expect("explanation accepted");
    fx.human(
        &owner,
        HumanCmd::SettleClash {
            clash,
            choice: Settlement::Serialize {
                first: t1,
                then: t2,
            },
        },
    )
    .expect("settle accepted");
    assert_eq!(phase(&fx), OrderPhase::Building);

    let fail = WitnessOutcome::Fail {
        detail: "login test failed".to_owned(),
    };
    gate_once(&mut fx, &a, f1, "aaa000", &fail);
    gate_once(&mut fx, &a, f1, "aaa111", &WitnessOutcome::Pass);
    land(&mut fx, t1, "aaa111", "m1");
    gate_once(&mut fx, &b, f2, "bbb222", &WitnessOutcome::Pass);
    assert_eq!(phase(&fx), OrderPhase::Landing);
    land(&mut fx, t2, "bbb222", "m2");
    assert_eq!(phase(&fx), OrderPhase::Landed);
    fx
}

#[test]
fn replaying_the_full_loop_rebuilds_the_same_world() {
    let fx = full_loop();
    let replica = Replica::from_log(fx.log.clone()).expect("log folds");
    assert_eq!(replica.world(), fx.origin.world());
    let rebuilt = Origin::replay(fx.log.clone()).expect("log folds");
    assert_eq!(rebuilt.world(), fx.origin.world());
}

#[test]
fn streaming_the_full_loop_event_by_event_rebuilds_the_same_world() {
    let fx = full_loop();
    let mut replica = Replica::from_log(fx.log[..1].to_vec()).expect("opening folds");
    for stamped in &fx.log[1..] {
        replica.ingest(stamped).expect("in-order event applies");
    }
    assert_eq!(replica.world(), fx.origin.world());
}

#[test]
fn the_stored_json_replays_to_the_same_world() {
    let fx = full_loop();
    let json = serde_json::to_string(&fx.log).expect("log serializes");
    let stored: Vec<acan_core::Stamped> = serde_json::from_str(&json).expect("log parses");
    let replica = Replica::from_log(stored).expect("log folds");
    assert_eq!(replica.world(), fx.origin.world());
}
