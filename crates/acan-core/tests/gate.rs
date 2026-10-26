mod common;

use std::collections::BTreeMap;

use acan_core::{
    AgentCmd, CommitId, Event, GateRun, GateRunId, GateVerdict, Generation, Job, LeaseEnd,
    OrderRev, Rejection, Stage, SystemCmd, TaskId, TaskState, WitnessId, WitnessOutcome,
};
use common::{Fx, events};

fn commit(raw: &str) -> CommitId {
    raw.parse().expect("valid commit")
}

fn run_of(fx: &Fx, task: TaskId) -> GateRunId {
    match &fx.origin.world().task(task).expect("task").state {
        TaskState::Claimed {
            stage: Stage::Gating { run, .. },
            ..
        } => *run,
        other => panic!("task is not gating: {other:?}"),
    }
}

fn gate_run(witnesses: &[&str], results: &[(&str, WitnessOutcome)]) -> GateRun {
    let w = |raw: &str| -> WitnessId { raw.parse().expect("valid witness") };
    GateRun {
        id: "g-0".parse().expect("valid run"),
        task: "t-0".parse().expect("valid task"),
        generation: Generation::new(1),
        rev: OrderRev::new(0),
        head: commit("abc123"),
        witnesses: witnesses.iter().map(|&raw| w(raw)).collect(),
        results: results
            .iter()
            .map(|(raw, o)| (w(raw), o.clone()))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn verdict_waits_for_every_frozen_witness() {
    let run = gate_run(&["w-0", "w-1"], &[("w-0", WitnessOutcome::Pass)]);
    assert_eq!(run.verdict(), GateVerdict::Running);
}

#[test]
fn an_error_never_counts_as_a_pass() {
    let error = WitnessOutcome::Error {
        detail: "server did not start".to_owned(),
    };
    let run = gate_run(
        &["w-0", "w-1"],
        &[("w-0", WitnessOutcome::Pass), ("w-1", error)],
    );
    assert_eq!(
        run.verdict(),
        GateVerdict::Failed {
            failing: vec!["w-1".parse().expect("valid witness")]
        }
    );
}

#[test]
fn a_run_with_no_gated_witnesses_passes() {
    assert_eq!(gate_run(&[], &[]).verdict(), GateVerdict::Passed);
}

#[test]
fn reporting_before_the_build_is_cleared_is_rejected() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.claim(&a, t1);
    let got = fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("abc123"),
        },
    );
    assert!(
        matches!(got, Err(Rejection::WrongState { .. })),
        "got {got:?}"
    );
}

#[test]
fn reporting_a_build_freezes_the_gated_witnesses_and_derives_one_job_each() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.build(&a, t1);
    fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("abc123"),
        },
    )
    .expect("report accepted");
    let run = run_of(&fx, t1);
    let gate = fx.origin.world().gate(run).expect("gate run");
    assert_eq!(gate.witnesses, fx.witnesses());
    assert_eq!(gate.generation, fence.generation);
    let jobs = fx.origin.world().jobs();
    for &witness in &gate.witnesses {
        assert!(
            jobs.contains(&Job::RunWitness { run, witness }),
            "jobs: {jobs:?}"
        );
    }
}

#[test]
fn a_failing_witness_sends_the_task_back_to_building() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.build(&a, t1);
    fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("abc123"),
        },
    )
    .expect("report accepted");
    let (run, witness) = (run_of(&fx, t1), fx.witnesses()[0]);
    let outcome = WitnessOutcome::Fail {
        detail: "login test failed".to_owned(),
    };
    let batch = fx
        .system(SystemCmd::WitnessResult {
            run,
            witness,
            outcome,
        })
        .expect("result accepted");
    assert!(events(&batch).iter().any(|e| matches!(
        e,
        Event::GateClosed {
            verdict: GateVerdict::Failed { .. },
            ..
        }
    )));
    assert!(matches!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Claimed {
            stage: Stage::Building { .. },
            ..
        }
    ));
}

#[test]
fn a_passing_gate_queues_the_task_and_delivers_the_lease() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.build(&a, t1);
    fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("abc123"),
        },
    )
    .expect("report accepted");
    let (run, witness) = (run_of(&fx, t1), fx.witnesses()[0]);
    let batch = fx
        .system(SystemCmd::WitnessResult {
            run,
            witness,
            outcome: WitnessOutcome::Pass,
        })
        .expect("result accepted");
    assert!(events(&batch).iter().any(|e| matches!(
        e,
        Event::LeaseEnded {
            why: LeaseEnd::Delivered,
            ..
        }
    )));
    assert!(
        matches!(fx.origin.world().task(t1).expect("task").state, TaskState::Queued { run: r, .. } if r == run)
    );
    assert_eq!(fx.origin.world().land_queue(), vec![t1]);
    assert!(fx.origin.world().jobs().contains(&Job::Land {
        task: t1,
        head: commit("abc123")
    }));
}

#[test]
fn a_result_for_a_superseded_run_is_rejected() {
    let mut fx = Fx::new();
    let (a, t1) = (fx.agent_a.clone(), fx.task1);
    let fence = fx.build(&a, t1);
    fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("abc123"),
        },
    )
    .expect("report accepted");
    let (old, witness) = (run_of(&fx, t1), fx.witnesses()[0]);
    let fail = WitnessOutcome::Fail {
        detail: "first try".to_owned(),
    };
    fx.system(SystemCmd::WitnessResult {
        run: old,
        witness,
        outcome: fail,
    })
    .expect("result accepted");
    fx.agent(
        &a,
        AgentCmd::ReportBuilt {
            fence,
            head: commit("def456"),
        },
    )
    .expect("second report accepted");
    let before = fx.origin.world().clone();
    let got = fx.system(SystemCmd::WitnessResult {
        run: old,
        witness,
        outcome: WitnessOutcome::Pass,
    });
    assert!(got.is_err(), "a stale run result was accepted: {got:?}");
    assert_eq!(fx.origin.world(), &before);
}

/// Builds both fixture tasks on disjoint footprints and passes both gates, task1 first.
fn both_queued() -> Fx {
    let mut fx = Fx::new();
    let (a, b, t1, t2) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1, fx.task2);
    let witness = fx.witnesses()[0];
    let f1 = fx.build(&a, t1);
    let f2 = fx.claim(&b, t2);
    let health = acan_core::DeclarationInput {
        symbols: vec![common::sym("src/api/routes.rs", "health")],
        ..Default::default()
    };
    fx.declare(&b, f2, health).expect("declare accepted");
    for (agent, task, fence, head) in [(&a, t1, f1, "aaa111"), (&b, t2, f2, "bbb222")] {
        fx.agent(
            agent,
            AgentCmd::ReportBuilt {
                fence,
                head: commit(head),
            },
        )
        .expect("report accepted");
        let run = run_of(&fx, task);
        fx.system(SystemCmd::WitnessResult {
            run,
            witness,
            outcome: WitnessOutcome::Pass,
        })
        .expect("pass accepted");
    }
    fx
}

#[test]
fn only_the_front_of_the_land_queue_gets_a_land_job() {
    let fx = both_queued();
    let lands: Vec<Job> = fx
        .origin
        .world()
        .jobs()
        .into_iter()
        .filter(|j| matches!(j, Job::Land { .. }))
        .collect();
    assert_eq!(
        lands,
        vec![Job::Land {
            task: fx.task1,
            head: commit("aaa111")
        }]
    );
}

#[test]
fn a_land_result_for_a_task_behind_the_front_is_rejected() {
    let mut fx = both_queued();
    let got = fx.system(SystemCmd::LandResult {
        task: fx.task2,
        head: commit("bbb222"),
        result: Ok(commit("m2")),
    });
    assert!(got.is_err(), "landed out of queue order: {got:?}");
}

#[test]
fn a_merged_task_lands_and_a_failed_merge_reopens_the_next() {
    let mut fx = both_queued();
    let (t1, t2) = (fx.task1, fx.task2);
    fx.system(SystemCmd::LandResult {
        task: t1,
        head: commit("aaa111"),
        result: Ok(commit("m1")),
    })
    .expect("land accepted");
    assert_eq!(
        fx.origin.world().task(t1).expect("task").state,
        TaskState::Landed {
            commit: commit("m1")
        }
    );
    let conflict = Err("merge conflict in src/api/routes.rs".to_owned());
    fx.system(SystemCmd::LandResult {
        task: t2,
        head: commit("bbb222"),
        result: conflict,
    })
    .expect("failure accepted");
    assert_eq!(
        fx.origin.world().task(t2).expect("task").state,
        TaskState::Open
    );
}
