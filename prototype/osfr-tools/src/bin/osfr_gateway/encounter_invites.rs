//! T440 (Shared World slice 5): the group encounter invitation route. After a committed Go
//! starts a grouped member's field-battle run, every other connected, eligible member gets the
//! original 41/102 prompt; their 41/103 answer comes back here. Accept enters the inviter's
//! World instance through `stage_encounter_join` and the member's own `commit_outcome`. Decline,
//! expiry and every mismatch leave zone, instance, pose, save and party untouched. The codecs and
//! the pending table are `osfr_tools::encounter_invite`.
use super::*;
use osfr_tools::encounter_invite::{self as invite, InvitePacket, Pending, Reply};

/// What one dispatched message left for the route, read before its commit replaces the gateway.
pub(super) struct Staged {
    replies: Vec<Reply>,
    /// The message zoned the player while they had no run: a start, not a reconnect or a move.
    may_start: bool,
    zone: u32,
}

impl Staged {
    pub(super) fn new(gateway: &Gateway, outcome: &GatewayOutcome) -> Self {
        Self {
            replies: outcome
                .applications
                .iter()
                .filter_map(|inner| invite::decode_reply(inner))
                .collect(),
            may_start: outcome.began_zoning && gateway.joinable_run().is_none(),
            zone: gateway.state().zone_id,
        }
    }
}

/// Run after a committed transaction: answer first, then offer a run this message started.
pub(super) fn run(
    me: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    staged: Staged,
) -> Result<(), Box<dyn std::error::Error>> {
    for reply in &staged.replies {
        answer(me, sessions, world, membership, now_ms, reply)?;
    }
    if staged.may_start {
        offer(me, sessions, world, membership, now_ms, staged.zone)?;
    }
    Ok(())
}

pub(super) fn session_of<'a>(
    world: &SharedWorld,
    sessions: &'a mut SessionMap,
    guid: u64,
) -> Option<&'a mut Session> {
    let key = world.session_keys.get(world.active_sessions.get(&guid)?)?;
    sessions.get_mut(key)
}

/// One 41/102 to each other group member standing in `zone` who could start the battle.
fn offer(
    me: &Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
    membership: &mut World,
    now_ms: u64,
    zone: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some((encounter, run, title_id)) = me.gateway.joinable_run() else {
        return Ok(());
    };
    let Some(group) = world.social.group_of(me.character_guid) else {
        return Ok(());
    };
    let members = world.social.group_members(group).map(<[u64]>::to_vec);
    for guid in members.unwrap_or_default() {
        let in_run = membership.player_location(guid).map(|at| at.instance_id) == Some(run);
        let Some(member) = session_of(world, sessions, guid) else {
            continue;
        };
        // A busy or ineligible member gets nothing (R).
        if guid == me.character_guid
            || in_run
            || member.gateway.state().zone_id != zone
            || !member.gateway.can_join_group_battle(encounter)
        {
            continue;
        }
        let row = Pending {
            inviter: me.character_guid,
            run,
            encounter,
            deadline_ms: now_ms.saturating_add(invite::TTL_MS),
        };
        if !membership
            .invites_mut()
            .offer(guid, row, now_ms, MAX_SESSIONS)
        {
            continue;
        }
        let packet = InvitePacket {
            encounter,
            run,
            inviter: me.character_guid,
            title_id,
            countdown: invite::COUNTDOWN,
        };
        // The member's own full queue must not end the starter's session; the row just expires.
        let _ = member.queue_shared_message(&wrap_application(&invite::encode_invite(&packet)));
    }
    Ok(())
}

/// The responder is this session, never the packet, and no reply field is a key (a real client's
/// word B is its last battle instance). A reply with no live invite changes nothing. Decline
/// consumes the invite. Accept enters the server-held run only after the group, the inviter's
/// live run and this player's Go gates are rechecked. The invite is not consumed here: the join's
/// own membership install removes it (`World::commit_membership`), so it goes with the join or
/// not at all, and a refused Accept leaves it answerable until it expires.
fn answer(
    me: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
    membership: &mut World,
    now_ms: u64,
    reply: &Reply,
) -> Result<(), Box<dyn std::error::Error>> {
    let guid = me.character_guid;
    let Some(row) = membership.invites_mut().live(guid, now_ms).cloned() else {
        return Ok(());
    };
    if !reply.accept {
        membership.invites_mut().answer(guid, now_ms); // The party is not touched.
        return Ok(());
    }
    let group = world
        .social
        .group_of(guid)
        .filter(|group| world.social.group_of(row.inviter) == Some(*group));
    let run_live = membership.instance(row.run).is_some()
        && membership
            .player_location(row.inviter)
            .map(|at| at.instance_id)
            == Some(row.run)
        && session_of(world, sessions, row.inviter).is_some_and(|inviter| {
            inviter.gateway.joinable_run().map(|(e, r, _)| (e, r)) == Some((row.encounter, row.run))
        });
    let in_run = membership.player_location(guid).map(|at| at.instance_id) == Some(row.run);
    // T441: the arena this joiner enters, when it admits; a full, won, lost or faulted arena
    // takes nobody, so the joiner plays their own run (the brief's solo fallback).
    let (Some(_), true, false) = (group, run_live, in_run) else {
        return Ok(());
    };
    let arena = group_arena::prepare(world, sessions, membership, row.inviter, row.run);
    let mut staged = me.gateway.clone();
    let Ok(outcome) = staged.stage_encounter_join(row.encounter, row.run, now_ms) else {
        return Ok(());
    };
    let previous = me.gateway.state().clone();
    if me.commit_outcome(
        staged, previous, outcome, world, membership, sessions, now_ms,
    )? {
        if let Some(arena) = arena {
            group_arena::attach(membership, arena, row.run, guid);
        }
    }
    Ok(())
}

#[cfg(test)]
impl Server {
    /// The party the character is in, from the live social state (tests only).
    pub fn test_group(&self, guid: u64) -> Option<u32> {
        self.world.social.group_of(guid)
    }

    /// The live field-battle run the character's session holds (tests only).
    pub fn test_joinable_run(&self, guid: u64) -> Option<(u32, i32, i32)> {
        let key = self
            .world
            .session_keys
            .get(self.world.active_sessions.get(&guid)?)?;
        self.sessions.get(key)?.gateway.joinable_run()
    }

    /// Whether the character holds a live encounter invite (tests only).
    pub fn test_has_invite(&mut self, guid: u64) -> bool {
        let now = self.start.elapsed().as_millis() as u64;
        self.membership.invites_mut().live(guid, now).is_some()
    }

    /// Move the server clock forward: every deadline is measured on `start` (tests only).
    pub fn test_advance_clock(&mut self, ms: u64) {
        self.start = self.start.checked_sub(Duration::from_millis(ms)).unwrap();
    }
}
