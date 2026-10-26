mod common;

use acan_core::{
    Clash, ClashKey, ClashState, CodeRev, Declaration, DeclarationInput, Event, Generation,
    HumanCmd, Locus, Overlap, OverlapKind, Rejection, SettleRight, Settlement, Stage, TaskId,
    TaskState, detect,
};
use common::{Fx, events, index, machine, path, sym};

fn task(n: u32) -> TaskId {
    format!("t-{n}").parse().expect("valid task id")
}

fn decl(task_no: u32, footprint: DeclarationInput) -> Declaration {
    let rev: CodeRev = "rev-1".parse().expect("valid rev");
    Declaration {
        task: task(task_no),
        generation: Generation::new(1),
        index: rev,
        footprint,
    }
}

fn symbols(refs: &[(&str, &str)]) -> DeclarationInput {
    DeclarationInput {
        symbols: refs.iter().map(|&(f, n)| sym(f, n)).collect(),
        ..DeclarationInput::default()
    }
}

fn one(kind: OverlapKind, at: Locus) -> Vec<Overlap> {
    vec![Overlap {
        key: ClashKey::new(task(1), task(2), at),
        kind,
    }]
}

fn clash_of(kind: OverlapKind, at: Locus) -> Clash {
    Clash {
        id: "c-0".parse().expect("valid clash id"),
        key: ClashKey::new(task(1), task(2), at),
        kind,
        found_at: "rev-1".parse().expect("valid rev"),
        explanation: None,
        state: ClashState::Open,
    }
}

#[test]
#[ignore = "unimplemented"]
fn a_shared_implementation_menu_offers_narrow_either_or_serialize() {
    let at = Locus::Symbol {
        file: path("src/api/routes.rs"),
        name: "rate_limit".to_owned(),
    };
    let menu = clash_of(OverlapKind::SharedImplementation, at).menu();
    let want = vec![
        Settlement::Narrow { task: task(1) },
        Settlement::Narrow { task: task(2) },
        Settlement::Serialize {
            first: task(1),
            then: task(2),
        },
    ];
    assert_eq!(menu, want);
}

#[test]
#[ignore = "unimplemented"]
fn yielder_names_the_task_that_gives_way() {
    let key = ClashKey::new(
        task(1),
        task(2),
        Locus::File {
            file: path("src/api/mod.rs"),
        },
    );
    assert_eq!(
        Settlement::Narrow { task: task(2) }.yielder(&key),
        Some(task(2))
    );
    assert_eq!(
        Settlement::Serialize {
            first: task(1),
            then: task(2)
        }
        .yielder(&key),
        Some(task(2))
    );
    assert_eq!(Settlement::Narrow { task: task(9) }.yielder(&key), None);
}
#[test]
fn clash_key_puts_the_lower_task_first() {
    let at = Locus::File {
        file: path("src/api/routes.rs"),
    };
    let key = ClashKey::new(task(7), task(3), at.clone());
    assert_eq!((key.a(), key.b()), (task(3), task(7)));
    assert_eq!(key, ClashKey::new(task(3), task(7), at));
}

#[test]
#[ignore = "unimplemented"]
fn same_symbol_is_shared_implementation() {
    let d1 = decl(1, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let d2 = decl(2, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let at = Locus::Symbol {
        file: path("src/api/routes.rs"),
        name: "rate_limit".to_owned(),
    };
    assert_eq!(
        detect(&[&d1, &d2], &index()),
        one(OverlapKind::SharedImplementation, at)
    );
}

#[test]
#[ignore = "unimplemented"]
fn disjoint_symbols_in_one_file_are_advisory() {
    let d1 = decl(1, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let d2 = decl(2, symbols(&[("src/api/routes.rs", "health")]));
    let got = detect(&[&d1, &d2], &index());
    assert_eq!(
        got,
        one(
            OverlapKind::DisjointRegions,
            Locus::File {
                file: path("src/api/routes.rs")
            }
        )
    );
    assert!(!got[0].kind.blocking());
}

#[test]
#[ignore = "unimplemented"]
fn two_appends_to_one_registry_are_shared_insertion() {
    let inserts = || DeclarationInput {
        inserts: vec![path("src/api/mod.rs")],
        ..DeclarationInput::default()
    };
    let (d1, d2) = (decl(1, inserts()), decl(2, inserts()));
    let want = one(
        OverlapKind::SharedInsertion,
        Locus::File {
            file: path("src/api/mod.rs"),
        },
    );
    assert_eq!(detect(&[&d1, &d2], &index()), want);
}

#[test]
#[ignore = "unimplemented"]
fn whole_file_write_meets_any_symbol_in_it() {
    let d1 = decl(
        1,
        DeclarationInput {
            files: vec![path("src/api/routes.rs")],
            ..DeclarationInput::default()
        },
    );
    let d2 = decl(2, symbols(&[("src/api/routes.rs", "health")]));
    let want = one(
        OverlapKind::WholeFile,
        Locus::File {
            file: path("src/api/routes.rs"),
        },
    );
    assert_eq!(detect(&[&d1, &d2], &index()), want);
}

#[test]
#[ignore = "unimplemented"]
fn unresolved_symbol_counts_as_a_whole_file_write() {
    let d1 = decl(1, symbols(&[("src/api/routes.rs", "does_not_exist")]));
    let d2 = decl(2, symbols(&[("src/api/routes.rs", "health")]));
    let want = one(
        OverlapKind::WholeFile,
        Locus::File {
            file: path("src/api/routes.rs"),
        },
    );
    assert_eq!(detect(&[&d1, &d2], &index()), want);
}

#[test]
#[ignore = "unimplemented"]
fn two_lockfile_writers_are_generated_output() {
    let lock = || DeclarationInput {
        files: vec![path("Cargo.lock")],
        ..DeclarationInput::default()
    };
    let (d1, d2) = (decl(1, lock()), decl(2, lock()));
    let want = one(
        OverlapKind::GeneratedOutput,
        Locus::File {
            file: path("Cargo.lock"),
        },
    );
    assert_eq!(detect(&[&d1, &d2], &index()), want);
}

#[test]
#[ignore = "unimplemented"]
fn interface_change_meets_an_importing_machine_as_contract() {
    let d1 = decl(
        1,
        DeclarationInput {
            interfaces: vec![sym("src/auth/session.rs", "verify")],
            ..DeclarationInput::default()
        },
    );
    let d2 = decl(2, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let want = one(
        OverlapKind::Contract,
        Locus::Belt {
            from: machine("api"),
            to: machine("auth"),
        },
    );
    assert_eq!(detect(&[&d1, &d2], &index()), want);
}

#[test]
#[ignore = "unimplemented"]
fn interface_change_without_an_importing_belt_is_no_overlap() {
    let d1 = decl(
        1,
        DeclarationInput {
            interfaces: vec![sym("src/api/routes.rs", "router")],
            ..DeclarationInput::default()
        },
    );
    let d2 = decl(2, symbols(&[("src/auth/login.rs", "login")]));
    assert_eq!(detect(&[&d1, &d2], &index()), vec![]);
}

#[test]
#[ignore = "unimplemented"]
fn a_task_never_clashes_with_itself() {
    let d1 = decl(1, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let d1_again = decl(1, symbols(&[("src/api/routes.rs", "rate_limit")]));
    assert_eq!(detect(&[&d1, &d1_again], &index()), vec![]);
}

#[test]
#[ignore = "unimplemented"]
fn detection_does_not_depend_on_input_order() {
    let d1 = decl(1, symbols(&[("src/api/routes.rs", "rate_limit")]));
    let d2 = decl(
        2,
        symbols(&[
            ("src/api/routes.rs", "rate_limit"),
            ("src/api/routes.rs", "health"),
        ]),
    );
    let d3 = decl(
        3,
        DeclarationInput {
            files: vec![path("src/api/routes.rs")],
            ..DeclarationInput::default()
        },
    );
    let idx = index();
    let forward = detect(&[&d1, &d2, &d3], &idx);
    assert!(!forward.is_empty());
    assert_eq!(forward, detect(&[&d3, &d1, &d2], &idx));
    assert!(
        forward.windows(2).all(|w| w[0] <= w[1]),
        "output is sorted: {forward:?}"
    );
}

/// Claims both tasks (task1 by owner's agent, task2 by bob's agent) and declares the same symbol.
fn clashing() -> Fx {
    let mut fx = Fx::new();
    let (a, b, t1, t2) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1, fx.task2);
    let f1 = fx.claim(&a, t1);
    let f2 = fx.claim(&b, t2);
    fx.declare(&a, f1, symbols(&[("src/api/routes.rs", "rate_limit")]))
        .expect("declare accepted");
    fx.declare(&b, f2, symbols(&[("src/api/routes.rs", "rate_limit")]))
        .expect("declare accepted");
    fx
}

#[test]
#[ignore = "unimplemented"]
fn overlapping_declaration_records_a_blocking_clash_and_holds_both_tasks() {
    let fx = clashing();
    let clashes: Vec<_> = fx.origin.world().clashes().collect();
    assert_eq!(clashes.len(), 1);
    assert_eq!(clashes[0].kind, OverlapKind::SharedImplementation);
    assert_eq!(clashes[0].state, ClashState::Open);
    for t in [fx.task1, fx.task2] {
        assert!(matches!(
            fx.origin.world().task(t).expect("task").state,
            TaskState::Claimed {
                stage: Stage::Declared { .. },
                ..
            }
        ));
    }
}

#[test]
#[ignore = "unimplemented"]
fn advisory_overlap_is_not_recorded_and_both_tasks_build() {
    let mut fx = Fx::new();
    let (a, b, t1, t2) = (fx.agent_a.clone(), fx.agent_b.clone(), fx.task1, fx.task2);
    let f1 = fx.claim(&a, t1);
    let f2 = fx.claim(&b, t2);
    fx.declare(&a, f1, symbols(&[("src/api/routes.rs", "rate_limit")]))
        .expect("declare accepted");
    let batch = fx
        .declare(&b, f2, symbols(&[("src/api/routes.rs", "health")]))
        .expect("declare accepted");
    assert!(
        events(&batch)
            .iter()
            .any(|e| matches!(e, Event::BuildCleared { task } if *task == t2))
    );
    assert_eq!(fx.origin.world().clashes().count(), 0);
}

#[test]
#[ignore = "unimplemented"]
fn narrowing_redeclaration_dissolves_the_clash() {
    let mut fx = clashing();
    let (b, t2) = (fx.agent_b.clone(), fx.task2);
    let f2 = fx.fence(t2).expect("claimed");
    let batch = fx
        .declare(&b, f2, symbols(&[("src/api/routes.rs", "health")]))
        .expect("redeclare accepted");
    assert!(
        events(&batch)
            .iter()
            .any(|e| matches!(e, Event::ClashDissolved { .. }))
    );
    assert!(
        fx.origin
            .world()
            .clashes()
            .all(|c| c.state == ClashState::Dissolved)
    );
}

#[test]
#[ignore = "unimplemented"]
fn member_settles_their_own_agents_yield() {
    let mut fx = clashing();
    let (bob, t2) = (fx.bob.clone(), fx.task2);
    let clash = fx.origin.world().clashes().next().expect("clash").clone();
    let choice = Settlement::Narrow { task: t2 };
    assert_eq!(
        fx.origin.world().settle_right(&bob, &clash, &choice),
        Some(SettleRight::Settle)
    );
    fx.human(
        &bob,
        HumanCmd::SettleClash {
            clash: clash.id,
            choice: choice.clone(),
        },
    )
    .expect("settle accepted");
    assert_eq!(
        fx.origin.world().clash(clash.id).expect("clash").state,
        ClashState::Settled { choice, by: bob }
    );
    assert!(matches!(
        fx.origin.world().task(t2).expect("task").state,
        TaskState::Claimed {
            stage: Stage::Declaring,
            ..
        }
    ));
}

#[test]
#[ignore = "unimplemented"]
fn member_may_only_propose_another_members_yield_and_owner_confirms() {
    let mut fx = clashing();
    let (bob, owner, t1) = (fx.bob.clone(), fx.owner.clone(), fx.task1);
    let clash = fx.origin.world().clashes().next().expect("clash").clone();
    let choice = Settlement::Narrow { task: t1 };
    assert_eq!(
        fx.origin.world().settle_right(&bob, &clash, &choice),
        Some(SettleRight::Propose)
    );
    fx.human(
        &bob,
        HumanCmd::SettleClash {
            clash: clash.id,
            choice: choice.clone(),
        },
    )
    .expect("proposal accepted");
    let state = fx
        .origin
        .world()
        .clash(clash.id)
        .expect("clash")
        .state
        .clone();
    assert_eq!(
        state,
        ClashState::Proposed {
            choice: choice.clone(),
            by: bob
        }
    );
    fx.human(
        &owner,
        HumanCmd::SettleClash {
            clash: clash.id,
            choice: choice.clone(),
        },
    )
    .expect("owner settles");
    assert_eq!(
        fx.origin.world().clash(clash.id).expect("clash").state,
        ClashState::Settled { choice, by: owner }
    );
}

#[test]
#[ignore = "unimplemented"]
fn a_choice_off_the_menu_is_rejected() {
    let mut fx = clashing();
    let owner = fx.owner.clone();
    let clash = fx.origin.world().clashes().next().expect("clash").id;
    let outsider: TaskId = "t-99".parse().expect("valid task id");
    let got = fx.human(
        &owner,
        HumanCmd::SettleClash {
            clash,
            choice: Settlement::Narrow { task: outsider },
        },
    );
    assert!(matches!(got, Err(Rejection::Invalid { .. })), "got {got:?}");
}

#[test]
#[ignore = "unimplemented"]
fn serialize_clears_first_and_holds_then_until_first_lands() {
    let mut fx = clashing();
    let (owner, t1, t2) = (fx.owner.clone(), fx.task1, fx.task2);
    let clash = fx.origin.world().clashes().next().expect("clash").id;
    let batch = fx
        .human(
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
    let cleared: Vec<TaskId> = events(&batch)
        .into_iter()
        .filter_map(|e| match e {
            Event::BuildCleared { task } => Some(*task),
            _ => None,
        })
        .collect();
    assert_eq!(cleared, vec![t1]);
    assert!(matches!(
        fx.origin.world().task(t2).expect("task").state,
        TaskState::Claimed {
            stage: Stage::Declared { .. },
            ..
        }
    ));
}
