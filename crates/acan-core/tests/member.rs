mod common;

use acan_core::{AgentCmd, HumanCmd, MemberId, Rejection, Role};
use common::Fx;

#[test]
fn only_an_owner_adds_members() {
    let mut fx = Fx::new();
    let bob = fx.bob.clone();
    let carol: MemberId = "carol".parse().expect("valid member");
    let got = fx.human(
        &bob,
        HumanCmd::AddMember {
            member: carol,
            role: Role::Member,
        },
    );
    assert!(
        matches!(got, Err(Rejection::Forbidden { .. })),
        "got {got:?}"
    );
}

#[test]
fn an_owner_cannot_demote_themselves() {
    let mut fx = Fx::new();
    let owner = fx.owner.clone();
    let got = fx.human(
        &owner,
        HumanCmd::AddMember {
            member: owner.clone(),
            role: Role::Member,
        },
    );
    assert!(matches!(got, Err(Rejection::Invalid { .. })), "got {got:?}");
}

#[test]
fn re_adding_a_member_with_the_same_role_appends_nothing() {
    let mut fx = Fx::new();
    let (owner, bob) = (fx.owner.clone(), fx.bob.clone());
    let batch = fx
        .human(
            &owner,
            HumanCmd::AddMember {
                member: bob,
                role: Role::Member,
            },
        )
        .expect("idempotent add");
    assert!(batch.is_empty(), "got {batch:?}");
}

#[test]
fn an_agent_has_one_owner() {
    let mut fx = Fx::new();
    let (bob, a) = (fx.bob.clone(), fx.agent_a.clone());
    let got = fx.human(&bob, HumanCmd::RegisterAgent { agent: a });
    assert!(
        matches!(got, Err(Rejection::Forbidden { .. })),
        "got {got:?}"
    );
}

#[test]
fn strangers_and_unregistered_agents_are_forbidden() {
    let mut fx = Fx::new();
    let stranger: MemberId = "mallory".parse().expect("valid member");
    let got = fx.human(&stranger, HumanCmd::CancelOrder { order: fx.order });
    assert!(
        matches!(got, Err(Rejection::Forbidden { .. })),
        "got {got:?}"
    );
    let rogue = "rogue".parse().expect("valid agent");
    let got = fx.agent(&rogue, AgentCmd::Claim { task: fx.task1 });
    assert!(
        matches!(got, Err(Rejection::Forbidden { .. })),
        "got {got:?}"
    );
}
