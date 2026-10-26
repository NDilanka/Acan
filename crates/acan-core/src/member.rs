//! Members, agent principals, and who may settle a clash.

use serde::{Deserialize, Serialize};

use crate::event::{Event, Rejection};
use crate::ids::{AgentId, MemberId};
use crate::world::World;

/// A member's role in a world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// The plant owner. May add members and settle any clash.
    Owner,
    /// May post orders, register agents, answer questions, and settle clashes their own agents yield.
    Member,
}

/// A human in a world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// The member's id from the origin's token store.
    pub id: MemberId,
    /// The member's role.
    pub role: Role,
}

/// An agent acting for a member. Agents read back, claim, declare and report; they never decide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentPrincipal {
    /// The agent's id from the origin's token store.
    pub id: AgentId,
    /// The member who registered the agent and answers for it.
    pub owner: MemberId,
}

/// What a member's `SettleClash` does, from [`crate::World::settle_right`].
///
/// A member may yield their own agent's task. They may only propose that another member's
/// agent yields. The plant owner settles either way. A proposal becomes a settlement when
/// someone with [`SettleRight::Settle`] sends the same choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SettleRight {
    /// The choice takes effect.
    Settle,
    /// The choice is recorded as a proposal and the clash stays blocking.
    Propose,
}

/// `AddMember`: owner only. Re-adding a member with the same role is a no-op. An owner may
/// not demote themselves, so a world always keeps someone who can add members.
pub(crate) fn decide_add_member(
    world: &World,
    by: &MemberId,
    member: MemberId,
    role: Role,
) -> Result<Vec<Event>, Rejection> {
    if world.member(by).map(|m| m.role) != Some(Role::Owner) {
        return Err(Rejection::forbidden("only an owner adds members"));
    }
    if &member == by && role != Role::Owner {
        return Err(Rejection::invalid("an owner cannot demote themselves"));
    }
    if world.member(&member).is_some_and(|m| m.role == role) {
        return Ok(Vec::new());
    }
    Ok(vec![Event::MemberAdded {
        member: Member { id: member, role },
    }])
}

/// `RegisterAgent`: the caller becomes the agent's owner. An agent has exactly one owner.
pub(crate) fn decide_register_agent(
    world: &World,
    by: &MemberId,
    agent: AgentId,
) -> Result<Vec<Event>, Rejection> {
    match world.agent(&agent) {
        Some(principal) if &principal.owner == by => Ok(Vec::new()),
        Some(_) => Err(Rejection::forbidden(
            "agent is registered to another member",
        )),
        None => Ok(vec![Event::AgentRegistered {
            agent: AgentPrincipal {
                id: agent,
                owner: by.clone(),
            },
        }]),
    }
}
