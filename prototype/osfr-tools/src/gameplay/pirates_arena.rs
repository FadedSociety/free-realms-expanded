//! L963: Pirate's Plunder's match shell for `Gameplay` (slice 1). The arena visit follows the
//! fishing hole (`gameplay/fishing_scene.rs`): keep the world pose, zone into the arena, hold the
//! start packets (ZoneConfig and state 3) for the load, and zone back once. The match states move on
//! the client's reports and the clock; the rules are `crate::ship_combat`.
use super::*;
use crate::minigame::{Activity, ActivityFinish, MinigameRuns, RunMode, RunSpec};
use crate::ship_combat::{self, Match};
use std::sync::atomic::{AtomicI32, Ordering};

/// The run ids a match's offer takes, out of the other games' ranges.
static NEXT_RUN: AtomicI32 = AtomicI32::new(63_000_001);

/// What a match reports to its run: first place is a win.
struct Round {
    gold: u32,
    won: bool,
}

impl Activity for Round {
    fn score(&self) -> u32 {
        self.gold
    }

    fn finish(&self) -> Option<ActivityFinish> {
        Some(ActivityFinish {
            won: self.won,
            stars: 0,
            elapsed_seconds: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct PirateArena {
    /// The world pose kept at the start.
    pub(super) return_to: CharacterState,
    /// The offer the arena's load sends, once.
    offer: Option<Vec<Vec<u8>>>,
    /// The start packets, held until the player presses Go (`None` after).
    pending: Option<Vec<Vec<u8>>>,
    /// The ship sailing now; the cave swaps it.
    ship: ship_combat::Ship,
    /// A report was inside the base's cave: its window opens once per visit.
    in_cave: bool,
    /// The window is open: only then does a SelectShip count, or any client could call its ship
    /// home from the far side of the map.
    choosing: bool,
    /// The family-39 run: the title bar, the hidden world HUD and the ways out hang on it.
    runs: MinigameRuns,
    run_id: i32,
    game_id: i32,
    /// The arena scene; `current_zone` reports it, as for a fishing hole.
    pub(super) scene: crate::zone_transition::ZoneTransition,
    game: Match,
    /// L966: which of the content's arenas, and its treasure round.
    arena: usize,
    plunder: ship_combat::Plunder,
}

impl Gameplay {
    /// The name the standings show.
    pub fn set_character_name(&mut self, name: String) {
        self.character_name = name;
    }

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
        let mut plunder = ship_combat::Plunder::new(arena.pickups.len());
        let crew = content.crew(arena);
        let ship = content.opening_ship();
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
            // One human ship, and the captains in the other seats.
            ship_combat::player_count(1, u32::try_from(crew.len()).unwrap_or(0)),
            ship_combat::game_state(ship_combat::WAITING_FOR_CLIENTS),
            // The local player's ship: the client makes its local player from it (00a6c3b0).
            // Dock slot 0's colour (`load` keeps the list non-empty).
            ship_combat::create_ship(character.guid, 0, home, &ship, &content.colours[0]),
            // (R) The first flag and the first colour.
            ship_combat::player_card(
                character.guid,
                &self.character_name,
                &ship,
                content.decals.first(),
                0,
            ),
            ship_combat::ship_lists(character.guid, &content.fleet, &content.decals),
            // Its camera's config, which the ship camera reads from its first frame.
            ship_combat::camera_config(&content.camera),
        ];
        pending.extend(crew.iter().flat_map(|bot| bot.introduce(content)));
        plunder.man(crew);
        pending.push(ship_combat::mark_treasure(&arena.pickups));
        pending.extend(ship_combat::show_pickups(&arena.pickups, &content.treasure));
        // The standings show from the first frame, not the first bank.
        pending.push(ship_combat::refresh_ui());
        let game_id = i32::try_from(arena.minigame_id).map_err(invalid)?;
        let mut spec = RunSpec::empty(
            game_id,
            ship_combat::MINIGAME_TYPE,
            RunMode::Encounter {
                encounter_id: game_id,
                player_guid: character.guid,
            },
        );
        spec.name_id = arena.name_id;
        spec.icon_id = ship_combat::OFFER_ICON;
        spec.description_id = ship_combat::OFFER_DESCRIPTION;
        // The game's own board ends it; 39/19 then only gives the world HUD back.
        spec.flags[4] = 0;
        let run_id = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
        let mut runs = MinigameRuns::new(run_id)?;
        let offer = runs.offer(spec)?.replies;
        out.changed = true;
        out.scene_zoning = Some(scene.clone());
        let text = format!("Pirate's Plunder: into {}.", arena.scene);
        self.pirates = Some(Box::new(PirateArena {
            return_to: character.clone(),
            offer: Some(offer),
            pending: Some(pending),
            ship,
            in_cave: false,
            choosing: false,
            runs,
            run_id,
            game_id,
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

    /// After the arena loads: the offer, once. Go sends the start packets.
    pub(super) fn pirates_arena_loaded(&mut self) -> Option<Vec<Vec<u8>>> {
        self.pirates.as_mut()?.offer.take()
    }

    /// Go starts the match; the X or a cancel leaves it, a quit in play unpaid as a loss.
    fn pirates_run_packet(&mut self, inner: &[u8], now_ms: u64) -> io::Result<Outcome> {
        let mut out = Outcome::default();
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(out);
        };
        if inner.starts_with(&[39, 0, 5]) {
            let go = arena.runs.handle_client(inner)?;
            out.replies = go.replies;
            if go.accepted {
                arena.runs.started(arena.run_id);
                out.replies.extend(arena.pending.take().into_iter().flatten());
            }
        } else if arena.game.state() != ship_combat::EXITING {
            // On the results board the match is over and paid; only before that is it a loss.
            let quit = if arena.game.state() == ship_combat::GAME_OVER {
                let round = Round {
                    gold: arena.plunder.gold(),
                    won: arena.plunder.placement() == 1,
                };
                (arena.runs).finish_win_without_reward(arena.run_id, &round)?
            } else {
                arena.runs.forfeit(arena.run_id)?
            };
            // Still on the offer, 39/19 alone closes it, as for a derby.
            out.replies = if quit.accepted {
                quit.replies
            } else {
                vec![crate::minigame::state_remove_packet(arena.run_id, arena.game_id)]
            };
            arena.pending = None;
            arena.game.quit(now_ms);
            out.replies.push(ship_combat::game_state(ship_combat::EXITING));
        }
        Ok(out)
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
        if let Some(out) = self.handle_garage(inner)? {
            return Ok(Some(out));
        }
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(None);
        };
        // The offer's own close is 26/42, as for a derby; a closed run's 39/7 is only an
        // acknowledgement, which Exiting ignores.
        if inner == [26, 0, 42, 0]
            || (arena.runs.names_run(inner)
                && (inner.starts_with(&[39, 0, 5])
                    || inner.starts_with(&[39, 0, 6])
                    || inner.starts_with(&[39, 0, 7])))
        {
            return self.pirates_run_packet(inner, now_ms).map(Some);
        }
        if !inner.starts_with(&ship_combat::FAMILY.to_le_bytes()) {
            return Ok(None);
        }
        let mut out = Outcome::default();
        if let Some(bits) = ship_combat::client_states(inner) {
            let moved = arena.game.report(bits, now_ms);
            self.send_states(moved, now_ms, &mut out)?;
        }
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(Some(out));
        };
        if let Some((key, decal)) = ship_combat::selected_ship(inner) {
            self.swap_ship(&key, &decal, now_ms, &mut out);
            return Ok(Some(out));
        }
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(Some(out));
        };
        if let (Some(side), Some(content)) =
            (ship_combat::fired_side(inner), self.ship_combat.as_deref())
        {
            if arena.game.playing_since().is_some() {
                let volley = arena.plunder.fire(side, now_ms, &arena.ship, content);
                out.replies.extend(volley);
            }
            return Ok(Some(out));
        }
        // L966: the client's own ship report, while the clock runs.
        let report = ship_combat::ship_position(inner);
        let content = self.ship_combat.as_deref();
        if let (Some((guid, at)), Some(content)) = (report, content)
            && guid == arena.return_to.guid
            && let Some(start) = arena.game.playing_since()
            && let Some(place) = content.arenas.get(arena.arena)
        {
            let treasure = &content.treasure;
            let ship = arena.ship.clone();
            // The round starts with the ship on its pad when Playing began, and that lets the ship
            // go: until its first UpdatePlayerState it has no health, so the keys do nothing
            // (`ControllerShip` 00af8120), and it sits held by the flag the client set on building
            // the physics (0x100, 00ca2470). A report proves the physics exists (00ca7480), so the
            // flag cannot come back after this.
            if arena.plunder.begin(place, start) {
                let hold = arena.plunder.hold();
                out.replies
                    .push(ship_combat::player_state(guid, ship.health, hold));
                out.replies.extend(arena.plunder.launch_crew());
            }
            let earned = (arena.plunder).sail(guid, at, now_ms, place, treasure, &ship);
            out.replies.extend(earned);
            if let Some(motion) = ship_combat::ship_motion(inner) {
                arena.plunder.steer(at, motion, treasure.max_speed);
            }
            // Steering into the base's cave opens Ship Selection, once a visit; play goes on. Only
            // a position the speed bound accepted counts, so a forged report opens nothing, and a
            // ship that leaves for good (three radii) loses the window.
            let near = |radius: f32| {
                let dock = place.docks.first();
                let spot = arena.plunder.position();
                dock.zip(spot)
                    .is_some_and(|(dock, at)| ship_combat::near(dock.enter, at, radius))
            };
            let in_cave = near(treasure.enter_radius);
            let open = near(treasure.enter_radius * 3.0);
            if in_cave && !arena.in_cave {
                out.replies.push(ship_combat::open_ship_selection(guid));
                arena.choosing = true;
            }
            arena.in_cave = in_cave;
            arena.choosing &= open;
        }
        Ok(Some(out))
    }

    /// The cave's choice, in play: the old ship goes and the new one starts on the pad. An unknown
    /// key changes nothing; a flag off the list keeps the ship's own.
    fn swap_ship(&mut self, key: &str, decal: &str, now_ms: u64, out: &mut Outcome) {
        let (Some(arena), Some(content)) = (self.pirates.as_mut(), self.ship_combat.as_deref())
        else {
            return;
        };
        // An empty key keeps the ship: only the flag changed.
        let chosen = if key.is_empty() {
            Some(arena.ship.clone())
        } else {
            content.fleet_ship(key)
        };
        let (Some(mut ship), Some(place)) = (chosen, content.arenas.get(arena.arena)) else {
            return;
        };
        // A ship going down has no say: the window may have opened before it was hit.
        if arena.game.playing_since().is_none()
            || !std::mem::take(&mut arena.choosing)
            || arena.plunder.sunk()
        {
            return;
        }
        let flag = content.decals.iter().find(|d| d.alias == decal);
        if let Some(flag) = flag {
            ship.flag.clone_from(&flag.alias);
        }
        let shown = flag.or_else(|| content.decals.iter().find(|d| d.alias == ship.flag));
        let guid = arena.return_to.guid;
        let colour = &content.colours[0];
        let home = place.docks[0].bank;
        out.replies.extend([
            ship_combat::destroy_ship(guid),
            ship_combat::create_ship(guid, 0, home, &ship, colour),
            ship_combat::player_card(guid, &self.character_name, &ship, shown, 0),
            ship_combat::ship_lists(guid, &content.fleet, &content.decals),
            ship_combat::refresh_ui(),
        ]);
        arena.plunder.restart(&ship, home, now_ms);
        arena.ship = ship;
    }

    /// The states' packets. Game Over also opens the results board and settles the match; the
    /// state machine reaches it once.
    fn send_states(&mut self, moved: Vec<u32>, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        for state in moved {
            out.replies.push(ship_combat::game_state(state));
            if state == ship_combat::GAME_OVER {
                out.replies.push(ship_combat::player_awards());
                self.settle_pirates(now_ms, out)?;
            }
            if state == ship_combat::EXITING
                && let Some(arena) = self.pirates.as_mut()
            {
                let round = Round {
                    gold: arena.plunder.gold(),
                    won: arena.plunder.placement() == 1,
                };
                let ended = (arena.runs).finish_win_without_reward(arena.run_id, &round)?;
                out.replies.extend(ended.replies);
            }
        }
        Ok(())
    }

    /// The match's credit and pay, for the arena's own row. First place is the win, and finishing
    /// pays whatever the place (R). The score is the gold banked; nothing else is measured.
    fn settle_pirates(&mut self, now_ms: u64, out: &mut Outcome) -> io::Result<()> {
        let Some(arena) = self.pirates.as_ref() else {
            return Ok(());
        };
        let Some(content) = self.ship_combat.clone() else {
            return Ok(());
        };
        let Some(place) = content.arenas.get(arena.arena) else {
            return Ok(());
        };
        let (gold, guid) = (arena.plunder.gold(), arena.return_to.guid);
        let placement = arena.plunder.placement();
        let result = crate::quest_engine::MinigameResult {
            won: placement == 1,
            elapsed_seconds: None,
            score: Some(gold),
            placement: Some(placement),
            knocked_out: false,
        };
        self.credit_minigame(place.minigame_id, result, out)?;
        let grant = crate::rewards::RewardGrant {
            coins: content.win_coins,
            items: Vec::new(),
            job_xp: BTreeMap::new(),
            sc: 0,
            display: crate::rewards::RewardDisplay {
                source_guid: guid,
                player_guid: guid,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: Vec::new(),
            },
        };
        let rewards = self.reward_snapshot();
        let key = format!("pirates-match-{now_ms}");
        match crate::rewards::apply_grant(&rewards, &grant, &key) {
            Ok((next, replies)) => {
                out.replies.extend(replies);
                self.adopt_reward_state(next, out)
            }
            Err(error) => {
                eprintln!("pirates payout refused: {error}");
                Ok(())
            }
        }
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
            let moved = arena.game.advance(now_ms);
            self.send_states(moved, now_ms, &mut out)?;
        }
        let Some(arena) = self.pirates.as_mut() else {
            return Ok(out);
        };
        let content = self.ship_combat.as_deref();
        if arena.game.state() == ship_combat::PLAYING
            && let Some(content) = content
            && let Some(place) = content.arenas.get(arena.arena)
        {
            let back = (arena.plunder).respawn(now_ms, &place.pickups, &content.treasure);
            out.replies.extend(back);
            let guid = arena.return_to.guid;
            out.replies.extend(arena.plunder.refloat(guid, now_ms, place, content));
            out.replies.extend(arena.plunder.release(guid, now_ms, &arena.ship));
            let crew = arena.plunder.sail_crew(now_ms, place, content);
            out.replies.extend(crew);
        }
        // Balls still in the air at the whistle come down too: only DespawnProjectile ends one.
        if matches!(arena.game.state(), ship_combat::PLAYING | ship_combat::GAME_OVER)
            && let Some(content) = content
        {
            let guid = arena.return_to.guid;
            let guns = arena.plunder.gunnery(guid, now_ms, &arena.ship, content);
            out.replies.extend(guns);
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

