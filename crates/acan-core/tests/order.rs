mod common;

use std::collections::BTreeSet;

use acan_core::{
    AgentCmd, Check, Event, HumanCmd, Job, Judgement, LeaseEnd, OrderPhase, OrderRev, OrderState,
    Rejection, ReplayError, Replica, SystemCmd, TaskSpec, TaskState, WitnessKind, WitnessSpec,
};
use common::{Fx, events, machine};

#[test]
#[ignore = "unimplemented"]
fn a_posted_order_mints_locked_witnesses_and_derives_a_split_job() {
    let mut fx = Fx::new();
    let owner = fx.owner.clone();
    let witnesses = vec![WitnessSpec {
        check: Check::Test {
            command: "cargo test".to_owned(),
        },
        reason: "suite stays green".to_owned(),
    }];
    fx.human(
        &owner,
        HumanCmd::PostOrder {
            machine: machine("auth"),
            sentence: "Rotate session keys".to_owned(),
            witnesses,
        },
    )
    .expect("order posted");
    let world = fx.origin.world();
    let order = world.orders().last().expect("second order");
    assert_eq!(
        (order.rev, order.state),
        (OrderRev::new(0), OrderState::Splitting)
    );
    assert_eq!(world.phase(order.id), Some(OrderPhase::Splitting));
    let witness = world.witness(order.witnesses[0]).expect("witness minted");
    assert!(matches!(witness.kind, WitnessKind::Check { .. }));
    assert_eq!(witness.pinned_by, fx.owner);
    assert!(world.jobs().contains(&Job::Split {
        order: order.id,
        rev: OrderRev::new(0)
    }));
}

#[test]
#[ignore = "unimplemented"]
fn a_split_order_reads_back_and_derives_a_review_job_per_new_readback() {
    let mut fx = Fx::new();
    assert_eq!(
        fx.origin.world().phase(fx.order),
        Some(OrderPhase::ReadingBack)
    );
    let a = fx.agent_a.clone();
    let readback = fx.read_back(&a, fx.task1);
    let want = Job::Review {
        order: fx.order,
        rev: OrderRev::new(0),
        readbacks: BTreeSet::from([readback]),
    };
    assert!(
        fx.origin.world().jobs().contains(&want),
        "jobs: {:?}",
        fx.origin.world().jobs()
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_second_split_result_is_rejected() {
    let mut fx = Fx::new();
    let spec = TaskSpec {
        title: "again".to_owned(),
        brief: String::new(),
        scope: vec![machine("api")],
    };
    let got = fx.system(SystemCmd::SplitReady {
        order: fx.order,
        rev: OrderRev::new(0),
        tasks: vec![spec],
    });
    assert!(
        matches!(got, Err(Rejection::WrongState { .. })),
        "got {got:?}"
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_review_for_an_old_revision_is_rejected() {
    let mut fx = Fx::new();
    let a = fx.agent_a.clone();
    let readback = fx.read_back(&a, fx.task1);
    fx.ask(readback).expect("question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    let owner = fx.owner.clone();
    fx.human(
        &owner,
        HumanCmd::AnswerQuestion {
            question,
            option: 0,
        },
    )
    .expect("answer accepted");
    let got = fx.system(SystemCmd::ReadbacksReviewed {
        order: fx.order,
        rev: OrderRev::new(0),
        verdicts: vec![],
        question: None,
    });
    assert_eq!(
        got,
        Err(Rejection::StaleRevision {
            current: OrderRev::new(1)
        })
    );
}

#[test]
#[ignore = "unimplemented"]
fn disagreeing_readbacks_pin_exactly_one_question() {
    let mut fx = Fx::new();
    let (a, b) = (fx.agent_a.clone(), fx.agent_b.clone());
    let first = fx.read_back(&a, fx.task1);
    let second = fx.read_back(&b, fx.task2);
    fx.ask(first).expect("first question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    let batch = fx.ask(second).expect("second review accepted");
    assert!(
        !events(&batch)
            .iter()
            .any(|e| matches!(e, Event::QuestionPinned { .. })),
        "a second pin: {batch:?}"
    );
    assert_eq!(
        fx.origin.world().phase(fx.order),
        Some(OrderPhase::Asking { question })
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_claim_while_a_question_is_open_is_rejected() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    let matched = fx.read_back(&a, t1);
    fx.review(matched, Judgement::Match)
        .expect("review accepted");
    let unclear = fx.read_back(&b, fx.task2);
    fx.ask(unclear).expect("question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    assert_eq!(
        fx.agent(&a, AgentCmd::Claim { task: t1 }),
        Err(Rejection::OpenQuestion { question })
    );
}

#[test]
#[ignore = "unimplemented"]
fn answering_adds_a_ruling_bumps_the_revision_and_supersedes_live_leases() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    fx.claim(&a, t1);
    let unclear = fx.read_back(&b, fx.task2);
    fx.ask(unclear).expect("question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    let owner = fx.owner.clone();
    let batch = fx
        .human(
            &owner,
            HumanCmd::AnswerQuestion {
                question,
                option: 1,
            },
        )
        .expect("answer accepted");
    let got = events(&batch);
    assert!(
        got.iter()
            .any(|e| matches!(e, Event::QuestionAnswered { witness, .. }
        if matches!(&witness.kind, WitnessKind::Ruling { text, .. } if text == "Retry once")))
    );
    assert!(
        got.iter()
            .any(|e| matches!(e, Event::OrderRevised { rev, .. } if *rev == OrderRev::new(1)))
    );
    assert!(got.iter().any(
        |e| matches!(e, Event::LeaseEnded { why: LeaseEnd::Superseded, task, .. } if *task == t1)
    ));
    let order = fx.origin.world().order(fx.order).expect("order");
    assert_eq!((order.rev, order.witnesses.len()), (OrderRev::new(1), 2));
    assert_eq!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Open
    );
    assert!(fx.origin.world().open_question(fx.order).is_none());
}

#[test]
#[ignore = "unimplemented"]
fn a_verdict_from_an_old_revision_no_longer_lets_its_agent_claim() {
    let mut fx = Fx::new();
    let (a, b, t1) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1);
    let matched = fx.read_back(&a, t1);
    fx.review(matched, Judgement::Match)
        .expect("review accepted");
    let unclear = fx.read_back(&b, fx.task2);
    fx.ask(unclear).expect("question pinned");
    let question = fx
        .origin
        .world()
        .open_question(fx.order)
        .expect("open question")
        .id;
    let owner = fx.owner.clone();
    fx.human(
        &owner,
        HumanCmd::AnswerQuestion {
            question,
            option: 0,
        },
    )
    .expect("answer accepted");
    assert_eq!(
        fx.agent(&a, AgentCmd::Claim { task: t1 }),
        Err(Rejection::NotMatched {
            failing: fx.witnesses()
        })
    );
}

#[test]
#[ignore = "unimplemented"]
fn cancelling_drops_every_task_and_ends_live_leases() {
    let mut fx = Fx::new();
    let (a, t1, t2) = (fx.agent_a.clone(), fx.task1, fx.task2);
    fx.claim(&a, t1);
    let owner = fx.owner.clone();
    let batch = fx
        .human(&owner, HumanCmd::CancelOrder { order: fx.order })
        .expect("cancel accepted");
    assert!(events(&batch).iter().any(|e| matches!(
        e,
        Event::LeaseEnded {
            why: LeaseEnd::Cancelled,
            ..
        }
    )));
    for t in [t1, t2] {
        assert_eq!(
            fx.origin.world().task(t).expect("task").state,
            TaskState::Dropped
        );
    }
    assert_eq!(
        fx.origin.world().phase(fx.order),
        Some(OrderPhase::Cancelled)
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_replica_folding_the_log_agrees_with_the_origin() {
    let mut fx = Fx::new();
    let a = fx.agent_a.clone();
    fx.claim(&a, fx.task1);
    let replica = Replica::from_log(fx.log.clone()).expect("log folds");
    assert_eq!(replica.world(), fx.origin.world());
    assert_eq!(replica.next_seq(), fx.origin.world().next_seq());
}

#[test]
#[ignore = "unimplemented"]
fn a_replica_ignores_duplicates_and_reports_gaps() {
    let fx = Fx::new();
    let mut replica = Replica::from_log(fx.log[..2].to_vec()).expect("prefix folds");
    replica.ingest(&fx.log[1]).expect("a duplicate is ignored");
    assert_eq!(
        replica.ingest(&fx.log[3]),
        Err(ReplayError::SeqGap {
            expected: fx.log[2].seq,
            got: fx.log[3].seq
        })
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_log_that_does_not_open_the_world_is_refused() {
    let fx = Fx::new();
    assert_eq!(
        Replica::from_log(fx.log[1..].to_vec()).err(),
        Some(ReplayError::NotOpened)
    );
}
