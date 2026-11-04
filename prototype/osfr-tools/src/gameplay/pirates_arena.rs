//! L963: Pirate's Plunder's match shell for `Gameplay` (slice 1). The arena visit follows the
//! fishing hole (`gameplay/fishing_scene.rs`): keep the world pose, zone into the arena, hold the
//! start packets (ZoneConfig and state 3) for the load, and zone back once. The match states move on
//! the client's reports and the clock; the rules are `crate::ship_combat`.
use super::*;
use crate::ship_combat::{self, Match};

#[derive(Clone, Debug)]
pub(super) struct PirateArena {
    /// The world pose kept at the start.
    pub(super) return_to: CharacterState,
    /// The start packets, held until the arena has loaded (`None` after).
    pending: Option<Vec<Vec<u8>>>,
    /// The arena scene; `current_zone` reports it, as for a fishing hole.
    pub(super) scene: crate::zone_transition::ZoneTransition,
    game: Match,
    /// L966: which of the content's arenas, and its treasure round.
    arena: usize,
    plunder: ship_combat::Plunder,
}

impl Gameplay {
    /// Developer `realm pirates [arena]`: a solo match in arena 1-3 (the entry until the Games row
    /// is wired; R). One line answers.
    pub(super) fn begin_pirates(
        &mut self,
        arena: usize,
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<String> {
        if self.pirates.is_some() {
            return Ok("A Pirate's Plunder match is already open.".into());
        }
        // The Games row's own gate; the developer command passes here too, so a match never
        // starts from a fishing hole or a house (whose pose would become the way back).
        if self.activity_busy() {
            return Ok(crate::harvest_nodes::Denied::Busy.message());
        }
        let Some(content) = self.ship_combat.as_deref() else {
            return Ok("Pirate's Plunder is not available.".into());
        };
        let Some((index, arena)) =
            (arena.checked_sub(1)).and_then(|i| Some((i, content.arenas.get(i)?)))
        else {
            return Ok(format!("Usage: realm pirates <1-{}>", content.arenas.len()));
        };
        let plunder = ship_combat::Plunder::new(arena.pickups.len());
        // Every ship starts on its own pad; the solo player's is dock slot 0 (`load` keeps one).
        let home = arena.docks[0].bank;
        let [x, y, z] = home;
        let scene = crate::zone_transition::ZoneTransition {
            envelope_flag: 1,
            scene: arena.scene.clone(),
            type_id: 2,
            position: [x, y, z, 1.0],
            rotation: arena.rotation,
            unknown: 1,
            id: i32::try_from(arena.zone_id).map_err(invalid)?,
            // (R) As for the fishing holes.
            geometry_id: 214,
            override_update_radius: true,
            ..crate::zone_transition::ZoneTransition::default()
        };
        // Zoning turns every processor off (`OnPlayerBeginZoning` 0093d180), so the status that
        // turns ship combat on waits for the load with the rest.
        let mut pending = vec![
            crate::kart_gateway::player_status(character.guid, ship_combat::STATUS_SHIP_COMBAT),
            ship_combat::zone_config(
                arena,
                &content.zone_config,
                &content.colours,
                &content.treasure,
            ),
            // A solo match: the one human ship, no AI (R until the AI slice).
            ship_combat::player_count(1, 0),
            ship_combat::game_state(ship_combat::WAITING_FOR_CLIENTS),
            // The local player's ship: the client makes its local player from it (00a6c3b0).
            // Dock slot 0's colour (`load` keeps the list non-empty).
            ship_combat::create_ship(character.guid, 0, home, &content.ship, &content.colours[0]),
            // Its camera's config, which the ship camera reads from its first frame.
            ship_combat::camera_config(&content.camera),
        ];
        pending.extend(ship_combat::show_pickups(&arena.pickups, &content.treasure));
        out.changed = true;
        out.scene_zoning = Some(scene.clone());
        let text = format!("Pirate's Plunder: into {}.", arena.scene);
        self.pirates = Some(Box::new(PirateArena {
            return_to: character.clone(),
            pending: Some(pending),
            game: Match::new(content),
            scene,
            arena: index,
            plunder,
        }));
        Ok(text)
    }

    /// Developer `realm pirates [arena]` (arena 1 without one).
    pub(super) fn developer_pirates(
        &mut self,
        rest: &[&str],
        character: &CharacterState,
        out: &mut Outcome,
    ) -> io::Result<()> {
        let arena = rest.first().and_then(|n| n.parse().ok()).unwrap_or(1);
        let text = self.begin_pirates(arena, character, out)?;
        out.replies.push(chat::system(&text));
        Ok(())
    }

    /// After the arena loads: the held packets, once.
    pub(super) fn pirates_arena_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        self.pirates.as_mut()?.pending.take()
    }

    /// A match is open, so the gateway's tick runs (`supervise_pirates`) whatever else is loaded.
    pub fn pirates_active(&self) -> bool {
        self.pirates.is_some()
    }

    /// A family-94 packet while a match is open (`None` otherwise): the client's report word moves
    /// the states on; the rest is taken quietly until its slice routes it.
    pub(super) fn handle_pirates(
        &mut self,
        inner: &[u8],
        now_ms: u64,
    ) -> io::Result<Option<Outcome>> {
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(None);
        };
        if !inner.starts_with(&ship_combat::FAMILY.to_le_bytes()) {
            return Ok(None);
        }
        let mut out = Outcome::default();
        if let Some(bits) = ship_combat::client_states(inner) {
            out.replies = states(arena.game.report(bits, now_ms));
        }
        // L966: the client's own ship report, while the clock runs.
        let report = ship_combat::ship_position(inner);
        let content = self.ship_combat.as_deref();
        if let (Some((guid, at)), Some(content)) = (report, content)
            && guid == arena.return_to.guid
            && let Some(start) = arena.game.playing_since()
            && let Some(place) = content.arenas.get(arena.arena)
        {
            // The round starts with the ship on its pad when Playing began.
            arena.plunder.begin(place, start);
            let (treasure, ship) = (&content.treasure, &content.ship);
            let earned = (arena.plunder).sail(guid, at, now_ms, place, treasure, ship);
            out.replies.extend(earned);
        }
        Ok(Some(out))
    }

    /// Per tick, onto the fishing supervision's `out`: the states the clock brings, then the way
    /// back once Exiting went out.
    pub(super) fn supervise_pirates(
        &mut self,
        mut out: Outcome,
        now_ms: u64,
    ) -> io::Result<Outcome> {
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(out);
        };
        if arena.pending.is_none() {
            arena.game.loaded(now_ms);
            out.replies.extend(states(arena.game.advance(now_ms)));
        }
        let content = self.ship_combat.as_deref();
        if arena.game.state() == ship_combat::PLAYING
            && let Some(content) = content
            && let Some(place) = content.arenas.get(arena.arena)
        {
            let back = (arena.plunder).respawn(now_ms, &place.pickups, &content.treasure);
            out.replies.extend(back);
        }
        if arena.game.finished(now_ms) {
            let back = arena.return_to.clone();
            let normal = ship_combat::STATUS_NORMAL;
            out.replies
                .push(crate::kart_gateway::player_status(back.guid, normal));
            out.scene_zoning = Some(self.world_zoning(&back)?);
            out.changed = true;
            self.pirates = None;
        }
        Ok(out)
    }
}

fn states(states: Vec<u32>) -> Vec<Vec<u8>> {
    states.into_iter().map(ship_combat::game_state).collect()
}
