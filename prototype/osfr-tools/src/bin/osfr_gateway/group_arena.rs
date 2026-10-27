//! T441 (Shared World slice 6): grouped players in one battle run share one arena. The World
//! owns the `SharedArena` (enemies, clocks, stuns); a session's staged action borrows it
//! (`Held`) and the server drives the enemies once per tick for the whole run (`tick`). Each
//! member keeps their own health, knockout, results and payout; the win fans out on the next tick.
//! The party's Accept (T440) opens or joins the arena; nothing a client sends names one.
use super::*;
use osfr_tools::encounter::{EncounterPhase, FieldParticipant, SharedArena, is_pickup_guid};

/// One staged action's loan of the World's arena. `enter` before the gateway runs, `leave` after
/// it, `publish` once the commit succeeded; a refused or failed action drops the copy, so the
/// World's arena never changes for it.
pub(super) struct Held {
    arena: Option<(i32, SharedArena)>,
    /// What the enemies did that the other members' clients must also see.
    public: Vec<Vec<u8>>,
}

impl Held {
    pub(super) fn enter(gateway: &mut Gateway, membership: &World, guid: u64) -> Self {
        let arena = membership.arena_of(guid).and_then(|(instance, arena)| {
            gateway
                .enter_arena(arena)
                .then(|| (instance, arena.clone()))
        });
        Self {
            arena,
            public: Vec::new(),
        }
    }

    /// `combat` is true for a hit: other messages (a scene load) answer only their sender.
    pub(super) fn leave(&mut self, gateway: &mut Gateway, outcome: &GatewayOutcome, combat: bool) {
        let Some((_, arena)) = &mut self.arena else {
            return;
        };
        gateway.leave_arena(arena);
        self.public = outcome
            .replies
            .iter()
            .filter(|reply| combat && reply.get(7..).is_some_and(public_reply))
            .cloned()
            .collect();
    }

    /// The commit went through: keep the arena, drop members who left it, tell the others.
    pub(super) fn publish(
        self,
        me: &Session,
        sessions: &mut SessionMap,
        world: &SharedWorld,
        membership: &mut World,
    ) {
        let Some((instance, mut arena)) = self.arena else {
            return;
        };
        retain_present(&mut arena, membership, instance);
        if arena.members.is_empty() {
            membership.remove_arena(instance);
            return;
        }
        let others = arena
            .members
            .iter()
            .copied()
            .filter(|guid| *guid != me.character_guid)
            .collect::<Vec<_>>();
        membership.install_arena(instance, arena);
        for guid in others {
            if let Some(member) = encounter_invites::session_of(world, sessions, guid) {
                queue_all(member, &self.public);
            }
        }
    }
}

/// An enemy-side packet every member's client needs, and no packet about the acting player
/// (the arena names the player's own actor 1, which is a different character on each client):
/// enemy health, spawn, removal, effects, motion, animation and the boss bar, and the objective
/// count. Window dressing for one member (damage popups, results, wheel) is not.
fn public_reply(inner: &[u8]) -> bool {
    let (offset, listed) = match inner.get(..4) {
        Some([125, 0, ..]) => (2, true),
        Some([35, 0, 3, 0]) => (6, true),
        Some([35, 0, 12, 0]) => (8, true), // after the i32 count
        Some([35, 0, 2 | 5 | 8 | 16, 0] | [32, 0, 9, 0]) => (4, true),
        Some([45, 0, 1 | 2, _]) => return true,
        _ => (0, false),
    };
    listed
        && inner
            .get(offset..offset + 8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u64::from_le_bytes)
            .is_some_and(|actor| actor != 1 && !is_pickup_guid(actor))
}

/// 32/7 attack-processed: its target is the member's own actor 1, so only that member gets it.
fn private_hit(inner: &[u8]) -> bool {
    inner.starts_with(&[32, 0, 7, 0])
}

fn queue_all(session: &mut Session, replies: &[Vec<u8>]) {
    for reply in replies {
        if let Err(error) = session.queue_shared_message(reply) {
            Server::refuse_field_delivery(session, error);
            return;
        }
    }
}

/// Members who no longer stand in the run's instance (closed, disconnected, moved) leave it.
fn retain_present(arena: &mut SharedArena, membership: &World, instance: i32) {
    arena.members.retain(|guid| {
        membership
            .player_location(*guid)
            .is_some_and(|at| at.instance_id == instance)
    });
}

/// The arena `row.run` would put this joiner in: the live one when it still admits, else a new
/// one copied from the inviter's own run. `None`: the run takes no member, so the joiner plays
/// their own (over, frozen or full); the join itself is T440's.
pub(super) fn prepare(
    world: &SharedWorld,
    sessions: &mut SessionMap,
    membership: &World,
    inviter: u64,
    run: i32,
) -> Option<SharedArena> {
    match membership.arena(run) {
        Some(arena) => arena.admits().then(|| arena.clone()),
        None => encounter_invites::session_of(world, sessions, inviter)?
            .gateway
            .open_arena(),
    }
}

/// The joiner's Accept committed: they are a member from their next action on.
pub(super) fn attach(membership: &mut World, mut arena: SharedArena, run: i32, joiner: u64) {
    arena.members.insert(joiner);
    membership.install_arena(run, arena);
}

/// Once per server tick: every shared arena's enemies move and swing once for the whole run,
/// whoever is in it. A fault freezes that arena only; the server and every other run go on.
pub(super) fn tick(server: &mut Server, now_ms: u64) {
    for instance in server.membership.arena_instances() {
        if server
            .membership
            .arena(instance)
            .is_none_or(|arena| arena.faulted)
        {
            continue;
        }
        if let Err(error) = contain(|| tick_arena(server, instance, now_ms)) {
            if let Some(mut arena) = server.membership.arena(instance).cloned() {
                arena.faulted = true;
                server.membership.install_arena(instance, arena);
            }
            event(json!({"event":"arena_faulted","instance":instance,"reason":error.to_string()}));
        }
    }
}

fn key_of(server: &Server, guid: u64) -> Option<SessionKey> {
    let id = server.world.active_sessions.get(&guid)?;
    server.world.session_keys.get(id).copied()
}

/// One member's step inside the arena loan, committed for that member alone: it runs on a copy
/// of their gateway, a changed copy is saved, then installed. A refused step or a failed save
/// changes nothing for the member and steps the arena back (`None`); a pending win retries on the
/// next tick. `Some(changed)` when it went through.
fn step(
    server: &mut Server,
    key: SessionKey,
    arena: &mut SharedArena,
    work: &mut dyn FnMut(&mut Gateway) -> io::Result<bool>,
) -> Option<bool> {
    let session = server.sessions.get_mut(&key)?;
    let (mut gateway, saved) = (session.gateway.clone(), arena.clone());
    if !gateway.enter_arena(arena) {
        return None;
    }
    let result = work(&mut gateway);
    gateway.leave_arena(arena);
    let result = result.and_then(|changed| {
        if changed {
            save(&session.state_path, &gateway.snapshot())?;
        }
        Ok(changed)
    });
    match result {
        Ok(changed) => {
            session.gateway = gateway;
            Some(changed)
        }
        Err(error) => {
            *arena = saved;
            event(json!({"event":"arena_member_refused","reason":error.to_string()}));
            None
        }
    }
}

fn tick_arena(
    server: &mut Server,
    instance: i32,
    now_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(mut arena) = server.membership.arena(instance).cloned() else {
        return Ok(());
    };
    retain_present(&mut arena, &server.membership, instance);
    if arena.members.is_empty() {
        server.membership.remove_arena(instance);
        return Ok(());
    }
    // Members who can be acted on: placed and ready here, scene loaded.
    let active: Vec<(u64, SessionKey)> = arena
        .members
        .iter()
        .filter_map(|guid| {
            let ready = server.membership.player_location(*guid)?.ready;
            let key = key_of(server, *guid)?;
            (ready && server.sessions.get(&key)?.gateway.is_world_loaded()).then_some((*guid, key))
        })
        .collect();
    let standing = |server: &Server, phases: &[EncounterPhase]| {
        arena.members.iter().any(|guid| {
            key_of(server, *guid)
                .and_then(|key| server.sessions.get(&key)?.gateway.run_phase())
                .is_some_and(|phase| phases.contains(&phase))
        })
    };
    let live = [EncounterPhase::Loading, EncounterPhase::Fighting];
    if !arena.won() {
        arena.set_lost(!standing(server, &live)); // every member knocked out or gone
    }
    let mut private: BTreeMap<SessionKey, Vec<Vec<u8>>> = BTreeMap::new();
    let mut public = Vec::new();
    if arena.won() {
        // The goal is down: each member still in the run gets their own results and payout.
        for (_, key) in &active {
            let pending = server.sessions[key]
                .gateway
                .run_phase()
                .is_some_and(|phase| live.contains(&phase));
            if !pending {
                continue;
            }
            let mut replies = Vec::new();
            let done = step(server, *key, &mut arena, &mut |gateway| {
                let out = gateway.finish_arena_win(now_ms)?;
                replies = out.replies;
                Ok(out.changed)
            });
            if done.is_some() {
                private
                    .entry(*key)
                    .or_default()
                    .extend(replies.iter().map(|reply| wrap_application(reply)));
            }
        }
    } else if !arena.lost() {
        if let Some(elapsed) = arena.due(now_ms) {
            let players: Vec<FieldParticipant> = active
                .iter()
                .filter_map(|(_, key)| server.sessions[key].gateway.arena_participant())
                .collect();
            if let (Some(&(_, driver)), false) = (active.first(), players.is_empty()) {
                let mut hits = Vec::new();
                step(server, driver, &mut arena, &mut |gateway| {
                    hits = gateway.tick_arena_enemies(&players, now_ms, elapsed);
                    Ok(false)
                });
                for (guid, damage, replies) in hits {
                    let Some((_, key)) = active.iter().find(|(g, _)| *g == guid) else {
                        continue;
                    };
                    for reply in replies {
                        if private_hit(&reply) {
                            private
                                .entry(*key)
                                .or_default()
                                .push(wrap_application(&reply));
                        } else {
                            public.push(wrap_application(&reply));
                        }
                    }
                    if damage == 0 {
                        continue;
                    }
                    let mut received = Vec::new();
                    let done = step(server, *key, &mut arena, &mut |gateway| {
                        let out = gateway.receive_arena_damage(damage, now_ms)?;
                        received = out.replies;
                        Ok(out.changed)
                    });
                    if done.is_some() {
                        private
                            .entry(*key)
                            .or_default()
                            .extend(received.iter().map(|reply| wrap_application(reply)));
                    }
                }
            }
        }
    }
    server.membership.install_arena(instance, arena);
    for (key, replies) in private {
        if let Some(session) = server.sessions.get_mut(&key) {
            queue_all(session, &replies);
        }
    }
    for (_, key) in &active {
        if let Some(session) = server.sessions.get_mut(key) {
            queue_all(session, &public);
        }
    }
    Ok(())
}

#[cfg(test)]
impl Server {
    /// The shared run this character is a member of: its run instance and a copy (tests only).
    pub fn test_arena(&self, guid: u64) -> Option<(i32, SharedArena)> {
        self.membership
            .arena_of(guid)
            .map(|(instance, arena)| (instance, arena.clone()))
    }

    /// An enemy of the character's run as (position, health): the shared arena's when the
    /// character is a member, else the character's own run (tests only).
    pub fn test_enemy(&self, guid: u64, enemy: u64) -> Option<([f32; 3], u32)> {
        let rows = match self.membership.arena_of(guid) {
            Some((_, arena)) => arena.enemies().to_vec(),
            None => {
                self.test_gateway(guid)?
                    .snapshot()
                    .gameplay?
                    .encounter?
                    .active?
                    .enemies
            }
        };
        rows.iter()
            .find(|row| row.guid == enemy)
            .map(|row| (row.position, row.health))
    }

    /// How many shared runs the World holds (tests only).
    pub fn test_arena_count(&self) -> usize {
        self.membership.arena_instances().len()
    }

    /// The live gateway of the character's session (tests only).
    pub fn test_gateway(&self, guid: u64) -> Option<&Gateway> {
        let key = key_of(self, guid)?;
        self.sessions.get(&key).map(|session| &session.gateway)
    }

    /// Where the character's saves go (tests only: a path under a file makes every save fail).
    pub fn test_state_path(&mut self, guid: u64) -> &mut PathBuf {
        let key = key_of(self, guid).unwrap();
        &mut self.sessions.get_mut(&key).unwrap().state_path
    }

    /// Knock the character out through the arena path, as the driver would (tests only).
    pub fn test_knock_out(&mut self, guid: u64, now_ms: u64) {
        let key = key_of(self, guid).unwrap();
        let (instance, mut arena) = self.test_arena(guid).unwrap();
        let mut gateway = self.sessions[&key].gateway.clone();
        assert!(gateway.enter_arena(&arena));
        let out = gateway.receive_arena_damage(u32::MAX, now_ms).unwrap();
        gateway.leave_arena(&mut arena);
        save(&self.sessions[&key].state_path, &gateway.snapshot()).unwrap();
        let session = self.sessions.get_mut(&key).unwrap();
        session.gateway = gateway;
        let replies = out
            .replies
            .iter()
            .map(|r| wrap_application(r))
            .collect::<Vec<_>>();
        queue_all(session, &replies);
        self.membership.install_arena(instance, arena);
    }
}

#[cfg(test)]
mod public_reply_tests {
    use super::*;

    fn packet(head: [u8; 4], pad: usize, guid: u64) -> Vec<u8> {
        let mut bytes = head.to_vec();
        bytes.extend(vec![0; pad]);
        bytes.extend(guid.to_le_bytes());
        bytes
    }

    #[test]
    fn pickup_spawns_and_removes_stay_with_the_collector() {
        let pickup = 0x5057_5550_0000_0003;
        let enemy = 0x4f50_454e_0000_0007;
        for (head, pad) in [([35, 0, 2, 0], 0), ([35, 0, 3, 0], 2)] {
            assert!(public_reply(&packet(head, pad, enemy)), "enemy {head:?}");
            assert!(!public_reply(&packet(head, pad, pickup)), "pickup {head:?}");
        }
    }

    #[test]
    fn relevance_is_judged_by_its_first_guid_not_its_count() {
        let mut relevance = vec![35, 0, 12, 0, 1, 0, 0, 0];
        relevance.extend(0x4f50_454e_0000_0007_u64.to_le_bytes());
        assert!(public_reply(&relevance));
        relevance[8..16].copy_from_slice(&1_u64.to_le_bytes());
        assert!(!public_reply(&relevance));
    }
}
