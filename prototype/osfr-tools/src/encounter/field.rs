//! Overworld field enemies and battle markers: content loading and reloads, wander and chase AI, and the battle start screen.
use super::*;

/// A battle marker's walk (R): four points WANDER_RADIUS around its home at
/// walking pace, pausing at each (the owner: the Troll Summoner "just walks
/// around"). Height stays at home: the server has no terrain heights.
#[derive(Clone, Copy, Debug)]
pub(super) struct Wander {
    pub(super) position: [f32; 3],
    pub(super) waypoint: usize,
    pub(super) pause_until: u64,
}

pub(super) const WANDER_RADIUS: f32 = 5.0;

pub(super) const WANDER_SPEED: f32 = 1.5;

pub(super) const WANDER_PAUSE_MS: u64 = 3000;

/// Only walk while a player could see it (NPC visibility radius is 100 m).
pub(super) const WANDER_SEEN: f32 = 90.0;

/// Closer than this it stops and faces the player (an easy click target).
pub(super) const WANDER_NOTICE: f32 = 8.0;

/// Battle-marker interaction radius: the start screen opens, and its Go
/// (41/108) may start the battle, only within this distance of the marker.
pub(super) const BATTLE_SCREEN_RADIUS: f32 = 8.0;

/// The one level rule for a published battle: the active job level must reach
/// the row's configured floor. Portal admission, the battle marker and
/// `EncounterEngine::begin` all call this; `Some(required)` means refused.
pub(crate) fn level_refusal(level: u32, required: u32) -> Option<u32> {
    (level < required).then_some(required)
}

impl FieldEnemy {
    fn reset_motion(&mut self) {
        self.motion = FieldMotion::Idle;
        self.roam_goal = None;
        self.pause_until = 0;
        self.seed = self.actor.guid;
        self.target = None;
        self.reset_serial = self.reset_serial.wrapping_add(1);
    }

    /// (R) Per-actor deterministic random stream, same LCG as combat crits.
    fn random(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.seed >> 40) as f32 / 16_777_216.0
    }
}

impl EncounterEngine {
    /// Authored field level, for client presentation by the owning gameplay
    /// route once its wire carrier is verified. Never infer it from HP.
    pub fn field_level(&self, guid: u64) -> Option<u32> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.present && f.actor.guid == guid)
            .map(|f| f.actor.level)
    }

    /// Load the overworld enemies (`field_enemies.json`). Their GUIDs must not
    /// collide with opening, dungeon or battle-spawner actors.
    pub fn load_field_enemies(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        if bytes.len() > 128 * 1024 {
            return Err(invalid("field enemy content too large"));
        }
        let content: FieldContent =
            serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
        if content.schema_version != 1
            || content.enemies.len() > 64
            || content.spawn_groups.len() > 64
        {
            return Err(invalid("unsupported field enemy content"));
        }
        let paid_revive_cost = content
            .paid_revive
            .as_ref()
            .map_or(Ok(revive::RETAIL_REVIVE_COST), revive::PaidRevive::cost)?;
        let FieldContent {
            mut enemies,
            spawn_groups,
            ..
        } = content;
        let mut actor_index = BTreeMap::new();
        for (index, actor) in enemies.iter().enumerate() {
            if actor_index.insert(actor.actor.guid, index).is_some() {
                return Err(invalid("duplicate field enemy GUID"));
            }
        }
        let mut group_ids = BTreeSet::new();
        let mut grouped = BTreeSet::new();
        for group in spawn_groups {
            if group.id.is_empty()
                || group.id.len() > 120
                || group.id.chars().any(char::is_control)
                || !group_ids.insert(group.id.clone())
                || group.region.is_empty()
                || group.region.len() > 120
                || group.region.chars().any(char::is_control)
                || !(1..=3600).contains(&group.respawn_seconds)
                || !(group.aggro_range > 0.0 && group.aggro_range <= 40.0)
                || !group.leash_radius.is_finite()
                || group.leash_radius <= 0.0
                || group.leash_radius > 1000.0
                || group.members.is_empty()
                || group.members.len() > 64
                || group.source.is_empty()
            {
                return Err(invalid("invalid field spawn group"));
            }
            for source in &group.source {
                let Some(evidence) = source.get("evidence").and_then(|v| v.as_str()) else {
                    return Err(invalid("field spawn group source needs evidence"));
                };
                if evidence.trim().is_empty() {
                    return Err(invalid("invalid field spawn group source"));
                }
                if source.get("kind").and_then(|v| v.as_str()) == Some("reconstructed")
                    && !evidence.starts_with("(R)")
                {
                    return Err(invalid("reconstructed field spawn group source needs (R)"));
                }
            }
            if !group.source.iter().any(|source| {
                source.get("kind").and_then(|v| v.as_str()) == Some("reconstructed")
                    && source
                        .get("evidence")
                        .and_then(|v| v.as_str())
                        .is_some_and(|evidence| evidence.starts_with("(R)"))
            }) {
                return Err(invalid("field spawn group needs reconstructed provenance"));
            }
            for guid in group.members {
                if !grouped.insert(guid) {
                    return Err(invalid("duplicate field spawn group member"));
                }
                let index = *actor_index
                    .get(&guid)
                    .ok_or_else(|| invalid("unknown field spawn group member"))?;
                let actor = &mut enemies[index];
                actor.respawn_seconds = Some(group.respawn_seconds);
                actor.aggro_range = Some(group.aggro_range);
                actor.level_band = group.level_band;
                actor.zone_id = group.zone_id;
                actor.region = Some(group.region.clone());
                actor.leash_radius = Some(group.leash_radius);
                actor.actor.behaviour = Some(group.behaviour.clone());
            }
        }
        let mut guids: BTreeSet<u64> = self
            .content
            .world_enemies
            .iter()
            .map(|a| a.guid)
            .chain(self.content.dungeons.iter().flat_map(|d| {
                d.enemies
                    .iter()
                    .map(|a| a.guid)
                    .chain(d.guide.iter().map(|g| g.guid))
            }))
            .chain(self.battles.iter().map(|b| b.spawner.guid))
            .collect();
        let mut field = Vec::with_capacity(enemies.len());
        let mut areas = BTreeSet::new();
        for mut f in enemies {
            validate_actor(&f.actor)?;
            let behaviour =
                resolve_behaviour(&f.actor, &self.behaviour_presets).map_err(|e| invalid(&e))?;
            let leash = f.leash_radius.unwrap_or(behaviour.leash);
            let respawn_seconds = f
                .respawn_seconds
                .ok_or_else(|| invalid("field enemy has no respawn timer"))?;
            let aggro_range = f
                .aggro_range
                .ok_or_else(|| invalid("field enemy has no aggro range"))?;
            if !f.actor.stages.is_empty()
                || f.actor.boss
                || !(1..=3600).contains(&respawn_seconds)
                || !(aggro_range > 0.0 && aggro_range <= 40.0)
                || !(1..=20).contains(&f.actor.level)
                || f.level_band.is_some_and(|[lo, hi]| {
                    lo == 0 || lo > f.actor.level || hi < f.actor.level || hi > 20
                })
                || f.zone_id != self.content.world.zone_id
                || !leash.is_finite()
                || leash <= 0.0
                || leash > 1000.0
                || f.region
                    .as_ref()
                    .is_some_and(|s| s.len() > 120 || s.chars().any(char::is_control))
            {
                return Err(invalid("invalid field enemy"));
            }
            if let Some(area) = &f.spawn_area {
                if area.id.is_empty()
                    || area.id.len() > 120
                    || area.id.chars().any(char::is_control)
                    || !areas.insert(area.id.clone())
                    || area.count != 1
                    || !valid_position(area.centre)
                    || area.centre != f.actor.position
                    || !area.radius.is_finite()
                    || area.radius < 0.0
                    || area.radius > leash
                {
                    return Err(invalid("invalid field spawn area"));
                }
            }
            if !guids.insert(f.actor.guid) {
                return Err(invalid("duplicate field enemy GUID"));
            }
            // (R) Authored stats are level-one baselines. Reuse final-era HP
            // curve and weapon tiers, rather than inventing per-kind curves.
            let health_scale = level_health(f.actor.level) as f32 / PLAYER_MAX as f32;
            f.actor.max_health = (f.actor.max_health as f32 * health_scale).round() as u32;
            f.actor.damage = (f.actor.damage as f32 * 1.75_f32.powi(weapon_tier(f.actor.level) - 1))
                .round() as u32;
            field.push(FieldEnemy {
                state: enemy_state(&f.actor),
                respawn_ms: u64::from(respawn_seconds) * 1000,
                aggro_range,
                present: f.quest_ids.is_empty(),
                quest_ids: f.quest_ids,
                leash_radius: f.leash_radius,
                roam_radius: f.spawn_area.map_or(0.0, |a| a.radius),
                motion: FieldMotion::Idle,
                roam_goal: None,
                pause_until: 0,
                target: None,
                reset_serial: 0,
                seed: f.actor.guid,
                actor: f.actor,
                respawn_at: None,
            });
        }
        self.field_sim.enemies = field;
        self.paid_revive_cost = paid_revive_cost;
        self.rebuild_behaviours()
    }

    /// Guid and home position of the lowest-guid field enemy of this kind: where a kill
    /// objective's "Take Me There" pin points (R). Present or not, so it works for a quest
    /// whose enemies appear only once it is active.
    pub fn field_enemy_site(&self, definition_id: u32) -> Option<(u64, [f32; 3])> {
        let definition_id = i32::try_from(definition_id).ok()?;
        self.field_sim
            .enemies
            .iter()
            .filter(|f| f.actor.definition_id == definition_id)
            .min_by_key(|f| f.actor.guid)
            .map(|f| (f.actor.guid, f.actor.position))
    }

    /// Developer teleport targets for living field enemies (zone 0).
    pub fn field_teleport_targets(&self) -> Vec<crate::dev_teleport::Target> {
        self.field_sim
            .enemies
            .iter()
            .filter(|f| f.present && f.state.health > 0)
            .map(|f| crate::dev_teleport::Target {
                guid: f.actor.guid,
                name: f.actor.name.clone(),
                zone_id: self.content.world.zone_id,
                position: f.state.position,
                heading: None,
                kind: "enemy".into(),
            })
            .chain(self.battles.iter().map(|b| crate::dev_teleport::Target {
                guid: b.spawner.guid,
                name: b.spawner.name.clone(),
                zone_id: 0,
                position: b.spawner.position,
                heading: None,
                kind: "battle".into(),
            }))
            .chain(self.dungeon().into_iter().flat_map(|d| {
                // Inside an arena: the enemies currently standing.
                let states = &self.state.active.as_ref().unwrap().enemies;
                d.enemies
                    .iter()
                    .zip(states)
                    .filter(|(_, s)| s.health > 0 && !s.dormant)
                    .map(|(a, s)| crate::dev_teleport::Target {
                        guid: a.guid,
                        name: a.name.clone(),
                        zone_id: d.zone.zone_id,
                        position: s.position,
                        heading: None,
                        kind: "enemy".into(),
                    })
            }))
            .collect()
    }

    /// Load overworld battles (`field_battles.json`); spawner and enemy GUIDs
    /// must not collide with any other encounter actor.
    pub fn load_field_battles(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        if bytes.len() > 128 * 1024 {
            return Err(invalid("field battle content too large"));
        }
        let file: FieldBattleFile =
            serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
        if file.schema_version != 1 || file.battles.len() > 64 {
            return Err(invalid("unsupported field battle content"));
        }
        let mut guids: BTreeSet<u64> = self
            .content
            .world_enemies
            .iter()
            .map(|a| a.guid)
            .chain(self.field_sim.enemies.iter().map(|f| f.actor.guid))
            .chain(self.content.dungeons.iter().flat_map(|d| {
                d.enemies
                    .iter()
                    .map(|a| a.guid)
                    .chain(d.guide.iter().map(|g| g.guid))
            }))
            .collect();
        let mut ids = BTreeSet::new();
        for b in &file.battles {
            if !ids.insert(b.id)
                || self.content.dungeons.iter().any(|d| d.id == b.id)
                || !(1..=5).contains(&b.difficulty)
                || b.enemies.is_empty()
                || b.enemies.len() > 16
            {
                return Err(invalid("invalid field battle"));
            }
            validate_zone(&b.zone)?;
            if b.prizes.len() > 4
                || b.prizes.iter().any(|p| {
                    p.item_id <= 0
                        || p.name_id <= 0
                        || p.icon_id <= 0
                        || !(1..=MAX_PRIZE_QUANTITY).contains(&p.quantity)
                })
            {
                return Err(invalid("invalid field battle prizes"));
            }
            for a in std::iter::once(&b.spawner).chain(&b.enemies) {
                validate_actor(a)?;
                if !guids.insert(a.guid) {
                    return Err(invalid("duplicate field battle GUID"));
                }
            }
        }
        // Battles run through the cave's instanced-dungeon flow: begin() zones
        // into the arena, on_zone_ready spawns the enemies, results and exit
        // return the player to where they started.
        let content = Arc::make_mut(&mut self.content);
        let mut battles = file.battles;
        for b in &mut battles {
            // Enemies scale like a player at the difficulty's level gate, so
            // each fight keeps the same share of danger (R).
            let factor = level_health(difficulty_level(b.difficulty)) as f32 / PLAYER_MAX as f32;
            for a in &mut b.enemies {
                a.max_health = (a.max_health as f32 * factor).round() as u32;
                a.damage = (a.damage as f32 * factor).round() as u32;
            }
        }
        for b in &battles {
            content.dungeons.push(Dungeon {
                id: b.id,
                title_id: b.title_id,
                description_id: b.description_id,
                objective_id: b.objective_id,
                icon_id: b.icon_id,
                zone: b.zone.clone(),
                guide: None,
                enemies: b.enemies.clone(),
                reward_coins: b.reward_coins,
                reward_items: Vec::new(),
                prizes: b.prizes.clone(),
            });
        }
        self.battles = battles;
        self.rebuild_behaviours()
    }

    /// Battle spawners for the world's proximity visibility:
    /// (guid, name, name id, model id, position).
    /// (guid, name, name id, model, position, overhead icon image id).
    pub fn battle_spawners(&self) -> Vec<(u64, String, i32, i32, [f32; 3], i32)> {
        self.battles
            .iter()
            .map(|b| {
                (
                    b.spawner.guid,
                    b.spawner.name.clone(),
                    b.spawner.name_id,
                    b.spawner.model_id,
                    b.spawner.position,
                    b.nameplate_image_id,
                )
            })
            .collect()
    }

    /// A field battle is playable only when every previewed prize can be
    /// granted: the definition resolves and its quantity fits one stack.
    /// Cave encounters have no `battles` row and are not subject to this check.
    pub fn battle_prizes_available(&self, id: u32, accepts: impl Fn(i32, u32) -> bool) -> bool {
        self.battles
            .iter()
            .find(|battle| battle.id == id)
            .is_none_or(|battle| {
                battle
                    .prizes
                    .iter()
                    .all(|prize| accepts(prize.item_id, prize.quantity))
            })
    }

    /// Run a developer load against a clone and swap it in only when every
    /// step succeeds. A reload is a rare developer action and the engine
    /// already derives `Clone`; a refusal drops the clone, so content,
    /// battles, resolved behaviours, the open start screen and the marker
    /// walks all keep their old values.
    fn staged(&mut self, f: impl FnOnce(&mut Self) -> io::Result<()>) -> io::Result<()> {
        let mut staged = self.clone();
        f(&mut staged)?;
        *self = staged;
        Ok(())
    }

    /// Developer hot reload of `field_battles.json` (outside encounters).
    pub fn reload_field_battles(&mut self, path: &Path) -> io::Result<()> {
        if self.shared_fields.is_some() {
            return Err(invalid("shared field reload requires an idle world reload"));
        }
        if self.active() {
            return Err(invalid(
                "finish or exit the encounter before reloading battles",
            ));
        }
        self.staged(|engine| {
            let old: BTreeSet<u32> = engine.battles.iter().map(|b| b.id).collect();
            // This file owns these dungeon rows: drop them before the read, or
            // a battle that reuses its own spawner GUIDs collides with itself.
            Arc::make_mut(&mut engine.content)
                .dungeons
                .retain(|d| !old.contains(&d.id));
            engine.battles.clear();
            engine.load_field_battles(path)?;
            engine.pending_battle = None;
            engine.field_sim.wander.clear();
            Ok(())
        })
    }

    /// Developer hot reload of `field_enemies.json`: despawn, reload, respawn.
    pub fn reload_field_enemies(&mut self, path: &Path) -> io::Result<Vec<Vec<u8>>> {
        if self.shared_fields.is_some() {
            return Err(invalid("shared field reload requires an idle world reload"));
        }
        if self.active() {
            return Err(invalid(
                "finish or exit the encounter before reloading enemies",
            ));
        }
        // The old actors are despawned and their per-guid state forgotten only
        // after the new file loads, so a refusal returns neither the packets
        // nor the cleanup.
        let previous: Vec<u64> = self
            .field_sim
            .enemies
            .iter()
            .map(|f| f.actor.guid)
            .collect();
        let mut packets: Vec<Vec<u8>> = self
            .field_sim
            .enemies
            .iter()
            .filter(|f| f.present && f.state.health > 0)
            .map(|f| remove(f.actor.guid))
            .collect();
        self.staged(|engine| {
            engine.load_field_enemies(path)?;
            for guid in &previous {
                engine.field_sim.moving.remove(guid);
                engine.field_sim.attacks.remove(guid);
                engine.skills.forget_target(*guid);
            }
            Ok(())
        })?;
        packets.extend(self.field_initialization());
        Ok(packets)
    }

    /// The return location of a saved battle dropped at load, once.
    pub fn take_dropped_return(&mut self) -> Option<CharacterState> {
        self.dropped_return.take()
    }

    /// An overworld battle's arena is active (a dungeon without a guide).
    pub fn in_battle_arena(&self) -> bool {
        self.dungeon().is_some_and(|d| d.guide.is_none())
    }

    /// Show a battle's start screen again (e.g. after Go was refused: the
    /// screen hid the HUD, so the player needs it back to use Switch Jobs).
    pub fn reopen_battle_screen(&mut self, battle: u32, profile: i32) -> Vec<Vec<u8>> {
        let Some(b) = self.battles.iter().find(|b| b.id == battle) else {
            return Vec::new();
        };
        let instance = self.state.next_instance_id;
        let packets = vec![
            details_packet(
                b.id,
                instance,
                [
                    b.title_id,
                    b.icon_id,
                    b.description_id,
                    b.objective_id,
                    b.difficulty,
                ],
                b.enemies.len(),
                profile,
                false,
                &b.prizes,
            ),
            encounter_header(107, b.id, instance),
        ];
        self.pending_battle = Some((b.id, instance));
        packets
    }

    /// `Some(required)` when `level` is below the battle's configured floor.
    pub(crate) fn battle_level_refusal(&self, battle: u32, level: u32) -> Option<u32> {
        let required = self
            .battles
            .iter()
            .find(|row| row.id == battle)?
            .required_combat_level;
        level_refusal(level, required)
    }

    /// Level refusal for the battle whose start screen is open.
    pub(crate) fn pending_battle_level_refusal(&self, level: u32) -> Option<u32> {
        let (id, _) = self.pending_battle?;
        self.battle_level_refusal(id, level)
    }

    /// Every production `begin` route asks this before the encounter changes.
    pub(super) fn admit_battle(&self, battle: u32) -> io::Result<()> {
        match self.battle_level_refusal(battle, self.job_level) {
            Some(required) => Err(invalid(&format!("combat level {required} required"))),
            None => Ok(()),
        }
    }

    /// The battle whose start screen is open, consumed by Go (41/108). The
    /// player must still stand within `BATTLE_SCREEN_RADIUS` of the marker:
    /// an out-of-reach screen is dropped so Go cannot start a battle after
    /// walking away. `None` covers no open screen and a dropped one.
    pub fn take_pending_battle(&mut self, position: [f32; 3]) -> Option<u32> {
        let (id, _) = self.pending_battle.take()?;
        let near =
            self.battles.iter().find(|b| b.id == id).is_some_and(|b| {
                distance(position, self.marker_position(b)) <= BATTLE_SCREEN_RADIUS
            });
        near.then_some(id)
    }

    /// The marker's current spot: where its walk has taken it, else home.
    fn marker_position(&self, battle: &FieldBattle) -> [f32; 3] {
        self.field_sim
            .wander
            .get(&battle.id)
            .map_or(battle.spawner.position, |w| w.position)
    }

    pub fn is_battle_spawner(&self, guid: u64) -> bool {
        self.battles.iter().any(|b| b.spawner.guid == guid)
    }

    /// Actors whose interactions the encounter owns (overworld enemies and
    /// battle spawners); Legacy also lists some of their GUIDs as plain NPCs.
    pub fn is_field_actor(&self, guid: u64) -> bool {
        self.is_field_enemy(guid) || self.is_battle_spawner(guid)
    }

    /// Spawner click: show the battle's start screen (difficulty, description,
    /// goal) without launching it. Go comes back as 41/108.
    pub(super) fn battle_screen(
        &mut self,
        guid: u64,
        position: [f32; 3],
        profile: i32,
    ) -> EncounterOutcome {
        if self.active() {
            return EncounterOutcome::default();
        }
        let Some(b) = self.battles.iter().find(|b| b.spawner.guid == guid) else {
            return EncounterOutcome::default();
        };
        let at = self.marker_position(b);
        if distance(position, at) > BATTLE_SCREEN_RADIUS {
            return EncounterOutcome::default();
        }
        let instance = self.state.next_instance_id;
        let prizes = b.prizes.clone();
        let (id, icon, title, description, objective, difficulty, count) = (
            b.id,
            b.icon_id,
            b.title_id,
            b.description_id,
            b.objective_id,
            b.difficulty,
            b.enemies.len(),
        );
        self.pending_battle = Some((id, instance));
        EncounterOutcome {
            replies: vec![
                details_packet(
                    id,
                    instance,
                    [title, icon, description, objective, difficulty],
                    count,
                    profile,
                    false,
                    &prizes,
                ),
                // 41/107 ZoneIsReady: the battle is in place, nothing to load.
                // DispatchEncounterPacket case 107 calls SetMiniGameReady
                // (00aa3fdf), turning the start screen's "Loading..." into Go!
                // (native run 023211 stayed on Loading without it).
                encounter_header(107, id, instance),
            ],
            ..Default::default()
        }
    }

    pub fn is_field_enemy(&self, guid: u64) -> bool {
        self.field_sim
            .enemies
            .iter()
            .any(|f| f.present && f.actor.guid == guid)
    }

    /// Show or hide quest-gated field enemies for the player's active quests.
    /// A newly present enemy spawns at full health at home.
    pub fn set_active_quests(&mut self, active: &BTreeSet<u32>) -> EncounterOutcome {
        let mut out = EncounterOutcome::default();
        if self.active() || self.shared_fields.is_some() {
            return out;
        }
        let mut shown = Vec::new();
        for i in 0..self.field_sim.enemies.len() {
            let f = &mut self.field_sim.enemies[i];
            if f.quest_ids.is_empty() {
                continue;
            }
            let present = f.quest_ids.iter().any(|id| active.contains(id));
            if present == f.present {
                continue;
            }
            f.present = present;
            out.changed = true;
            if present {
                f.state = enemy_state(&f.actor);
                f.respawn_at = None;
                f.reset_motion();
                self.field_sim.attacks.remove(&f.actor.guid);
                shown.push(i);
            } else {
                if f.state.health > 0 {
                    out.replies.push(remove(f.actor.guid));
                }
                self.field_sim.moving.remove(&f.actor.guid);
            }
        }
        for i in &shown {
            let f = &self.field_sim.enemies[*i];
            out.replies.push(self.spawn(&f.actor, f.state.position, 0));
            out.replies
                .push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
        }
        if !shown.is_empty() {
            let guids: Vec<u64> = shown
                .iter()
                .map(|i| self.field_sim.enemies[*i].actor.guid)
                .collect();
            out.replies.push(relevance(&guids, false));
        }
        out
    }

    pub fn field_guids(&self) -> Vec<u64> {
        self.field_sim
            .enemies
            .iter()
            .map(|f| f.actor.guid)
            .collect()
    }

    pub(super) fn field_initialization(&mut self) -> Vec<Vec<u8>> {
        self.field_combat = false;
        let mut packets = Vec::new();
        let mut alive = Vec::new();
        for f in &self.field_sim.enemies {
            if f.present && f.state.health > 0 {
                packets.push(self.spawn(&f.actor, f.state.position, 0));
                packets.push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
                alive.push(f.actor.guid);
            }
        }
        if !alive.is_empty() {
            packets.push(relevance(&alive, false));
        }
        packets
    }

    pub(super) fn tick_wanderers_for(
        &mut self,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
        out: &mut EncounterOutcome,
    ) {
        let ground = self.ground();
        for b in &self.battles {
            let home = b.spawner.position;
            let w = self.field_sim.wander.entry(b.id).or_insert(Wander {
                position: home,
                waypoint: 0,
                pause_until: 0,
            });
            let Some(player) = players
                .iter()
                .min_by(|a, b| {
                    distance(a.position, w.position)
                        .total_cmp(&distance(b.position, w.position))
                        .then(a.guid.cmp(&b.guid))
                })
                .map(|p| p.position)
            else {
                continue;
            };
            let near = distance(player, w.position);
            if near > WANDER_SEEN {
                continue;
            }
            let guid = b.spawner.guid;
            if near < WANDER_NOTICE {
                // It notices the player: stop and face them, so it can be clicked.
                if self.field_sim.moving.contains(&guid) {
                    // Notice exception: this facing divides by the 3-D `near`
                    // clamped at 1 cm, not by `heading`'s true distance. Below
                    // 1 cm the direction is shorter than a unit vector; the
                    // client normalizes it and ignores it under its 0.01
                    // squared-length threshold (`0x0095dae0`), so these bytes
                    // must not change.
                    let d = near.max(0.01);
                    let face = [
                        (player[0] - w.position[0]) / d,
                        0.0,
                        (player[2] - w.position[2]) / d,
                    ];
                    out.replies.push(movement(guid, w.position, face));
                }
                stand(guid, &mut self.field_sim.moving, out);
                continue;
            }
            if now_ms < w.pause_until {
                continue;
            }
            let angle =
                std::f32::consts::FRAC_PI_4 + w.waypoint as f32 * std::f32::consts::FRAC_PI_2;
            // Waypoints sit on the ground, so the walker can reach them exactly.
            let goal = ground.snapped([
                home[0] + WANDER_RADIUS * angle.cos(),
                home[1],
                home[2] + WANDER_RADIUS * angle.sin(),
            ]);
            let left = distance(w.position, goal);
            let step = WANDER_SPEED * elapsed.min(250) as f32 / 1000.0;
            if left <= step {
                w.position = goal;
                w.waypoint = (w.waypoint + 1) % 4;
                w.pause_until = now_ms.saturating_add(WANDER_PAUSE_MS);
                // Waypoint exception: the arrival facing divides the goal
                // offset by WANDER_RADIUS, not by the 3-D distance to it.
                let face = [
                    (goal[0] - home[0]) / WANDER_RADIUS,
                    0.0,
                    (goal[2] - home[2]) / WANDER_RADIUS,
                ];
                out.replies.push(movement(guid, w.position, face));
                stand(guid, &mut self.field_sim.moving, out);
                continue;
            }
            let Some(face) = heading(w.position, goal, left) else {
                continue;
            };
            for (coord, to) in w.position.iter_mut().zip(goal) {
                *coord += (to - *coord) / left * step;
            }
            ground.snap(&mut w.position);
            out.replies.push(movement(guid, w.position, face));
            walk(guid, 2, &mut self.field_sim.moving, out); // loc_walk
        }
    }

    /// (R) Field actors roam, chase within a home leash, then evade home and
    /// heal. Returning actors ignore aggro until they arrive.
    pub(super) fn tick_field(
        &mut self,
        position: [f32; 3],
        now_ms: u64,
        elapsed: u64,
        out: &mut EncounterOutcome,
        total_damage: &mut u32,
        participants: Option<&[FieldParticipant]>,
        hits: &mut Vec<(u64, u32)>,
    ) {
        let ground = self.ground();
        let solo = [self.field_participant(1, position, true)];
        let players = participants.unwrap_or(&solo);
        let shared = participants.is_some();
        let mut engaged = false;
        let mut respawned = Vec::new();
        for (i, f) in self.field_sim.enemies.iter_mut().enumerate() {
            let guid = f.actor.guid;
            if f.state.health == 0 {
                if f.respawn_at.is_some_and(|at| now_ms >= at) {
                    f.respawn_at = None;
                    f.state = enemy_state(&f.actor);
                    f.reset_motion();
                    if f.present {
                        respawned.push(i);
                    }
                }
                continue;
            }
            let b = self.behaviour_of.get(&guid).cloned().unwrap_or_default();
            let home = ground.snapped(f.actor.position);
            let leash = f.leash_radius.unwrap_or(b.leash);
            if !f.present {
                continue;
            }
            let valid = |p: &&FieldParticipant| {
                p.combat
                    && p.known_npcs
                        .as_ref()
                        .is_none_or(|known| known.contains(&guid))
                    && p.health > 0
                    && now_ms >= p.grace_until
                    && valid_position(p.position)
                    && (!shared || horizontal_distance(p.position, home) <= leash + b.reach)
                    && distance(p.position, f.state.position)
                        <= if f.motion == FieldMotion::Chasing {
                            f.aggro_range + 4.0
                        } else {
                            f.aggro_range
                        }
            };
            let player = players
                .iter()
                .filter(valid)
                .find(|p| Some(p.guid) == f.target)
                .or_else(|| {
                    players.iter().filter(valid).min_by(|a, b| {
                        distance(a.position, f.state.position)
                            .total_cmp(&distance(b.position, f.state.position))
                            .then(a.guid.cmp(&b.guid))
                    })
                });
            let position = player.map_or(home, |p| p.position);
            let delta = player.map_or(f32::MAX, |_| distance(position, f.state.position));
            f.target = player.map(|p| p.guid);
            let mitigation = player.map_or(1., |p| p.mitigation);
            // (R) A 4 m hysteresis margin avoids target flicker at aggro range.
            if !shared && now_ms < self.field_grace_until
                || horizontal_distance(f.state.position, home) > leash
                || (f.motion == FieldMotion::Chasing && delta > f.aggro_range + 4.0)
            {
                f.motion = FieldMotion::Returning;
                f.roam_goal = None;
                f.target = None;
            }
            if f.motion == FieldMotion::Idle && delta <= f.aggro_range {
                f.motion = FieldMotion::Chasing;
                f.roam_goal = None;
            }
            if self.skills.stunned(guid, now_ms) && f.motion != FieldMotion::Returning {
                engaged |= f.motion == FieldMotion::Chasing;
                continue;
            }
            if f.motion == FieldMotion::Idle
                && f.roam_goal.is_none()
                && now_ms >= f.pause_until
                && f.roam_radius > 0.0
                && b.mode != Mode::Stationary
            {
                // (R) Uniform disc samples, stable per GUID; no band/level roll.
                let angle = f.random() * std::f32::consts::TAU;
                let radius = f.random().sqrt() * f.roam_radius.min(leash);
                f.roam_goal = ground.field_position([
                    home[0] + radius * angle.cos(),
                    home[1],
                    home[2] + radius * angle.sin(),
                ]);
                if f.roam_goal.is_none() {
                    f.pause_until = now_ms.saturating_add(3000);
                }
            }
            let goal = match f.motion {
                FieldMotion::Returning => Some(home),
                FieldMotion::Chasing if delta > b.reach => Some(position),
                FieldMotion::Idle if now_ms >= f.pause_until => f.roam_goal,
                _ => None,
            };
            if let Some(goal) = goal {
                let left = distance(f.state.position, goal);
                let speed = if f.motion == FieldMotion::Returning {
                    b.run_speed
                } else {
                    b.move_speed
                };
                let stop = if f.motion == FieldMotion::Chasing {
                    (b.reach - 0.1).max(0.0)
                } else {
                    0.0
                };
                let step = (speed * elapsed.min(250) as f32 / 1000.0).min((left - stop).max(0.0));
                if left > 0.001 && step > 0.0 && b.mode != Mode::Stationary {
                    let (next, grounded) =
                        match advance(f.state.position, goal, left, step, &ground) {
                            Ok(next) => (next, true),
                            Err(next) => (next, false),
                        };
                    // Check BEFORE moving. Never take even one step outside home leash.
                    if f.motion == FieldMotion::Chasing && horizontal_distance(next, home) > leash {
                        f.motion = FieldMotion::Returning;
                        f.roam_goal = None;
                    } else if grounded {
                        let face = heading(f.state.position, goal, left).unwrap_or_default();
                        f.state.position = next;
                        out.replies.push(movement(guid, next, face));
                        walk(guid, 3, &mut self.field_sim.moving, out);
                        out.changed = true;
                    } else {
                        f.roam_goal = None;
                        f.pause_until = now_ms.saturating_add(3000);
                        if f.motion == FieldMotion::Chasing {
                            f.motion = FieldMotion::Returning;
                        }
                        stand(guid, &mut self.field_sim.moving, out);
                    }
                }
                if f.motion != FieldMotion::Chasing && distance(f.state.position, goal) <= 0.001 {
                    stand(guid, &mut self.field_sim.moving, out);
                    if f.motion == FieldMotion::Returning {
                        if f.state.health != f.actor.max_health {
                            f.state.health = f.actor.max_health;
                            out.replies
                                .push(enemy_hp(guid, f.state.health, f.actor.max_health));
                            out.changed = true;
                        }
                        self.field_sim.attacks.remove(&guid);
                        self.skills.forget_target(guid);
                        f.motion = FieldMotion::Idle;
                        f.target = None;
                        f.reset_serial = f.reset_serial.wrapping_add(1);
                    }
                    f.roam_goal = None;
                    f.pause_until = now_ms.saturating_add(1500 + (f.random() * 1500.0) as u64);
                    continue;
                }
            }
            if f.motion != FieldMotion::Chasing {
                continue;
            }
            engaged = true;
            if distance(position, f.state.position) <= b.reach {
                stand(guid, &mut self.field_sim.moving, out);
                if self
                    .field_sim
                    .attacks
                    .get(&guid)
                    .is_some_and(|last| now_ms.saturating_sub(*last) < b.attack_ms)
                {
                    continue;
                }
                self.field_sim.attacks.insert(guid, now_ms);
                out.replies.push(animation(guid, 1001, false));
                let victim = player.expect("a chasing actor has a valid target in reach");
                let prior = if shared {
                    hits.iter()
                        .filter(|(guid, _)| *guid == victim.guid)
                        .map(|(_, d)| *d)
                        .fold(0_u32, u32::saturating_add)
                } else {
                    *total_damage
                };
                let left = victim.health.saturating_sub(prior);
                let attack_damage = (f.actor.damage as f32 * mitigation).round() as u32;
                let damage = attack_damage.min(left);
                out.replies.push(attack_processed(
                    guid,
                    victim.guid,
                    damage,
                    left.saturating_sub(damage),
                    victim.max_health,
                    false,
                ));
                if shared {
                    hits.push((victim.guid, damage));
                } else {
                    *total_damage = total_damage.saturating_add(attack_damage);
                }
            }
        }
        for i in respawned {
            let f = &self.field_sim.enemies[i];
            if shared {
                // (R) One observer diff emits AddNpc/full HP, never a second
                // simulation spawn or HP correction for the same respawn.
                out.changed = true;
                continue;
            }
            out.replies.push(self.spawn(&f.actor, f.state.position, 0));
            out.replies
                .push(enemy_hp(f.actor.guid, f.state.health, f.actor.max_health));
            out.replies.push(relevance(&[f.actor.guid], false));
            out.changed = true;
        }
        if !shared && engaged != self.field_combat && !self.world_stage_fighting() {
            self.field_combat = engaged;
            out.replies.extend([
                rules(engaged),
                combat_flag(132, 0, 0, engaged),
                combat_flag(133, 0, 0, engaged),
            ]);
        }
    }
}

/// Facing from `from` toward `to` for a walk or chase step: the horizontal
/// delta divided by the full 3-D distance. `left` is the caller's
/// `distance(from, to)`, so the hot path computes the square root once.
/// Movement is opcode 125 (`0x008e5940`); its direction is a horizontal
/// vector that the client normalizes before `atan2(X, Z)` (`0x0095dae0`), so
/// the y component is zero. `None` when the spots coincide, in which case the
/// caller sends a zero facing (there is no direction to communicate).
/// Documented exceptions kept inline: the notice facing in
/// `tick_wanderers_for` (divides by 3-D distance clamped at 1 cm) and the
/// waypoint-arrival heading (divides the goal offset by `WANDER_RADIUS`);
/// `kite_turn`'s negated facing in `encounter.rs` is separate.
fn heading(from: [f32; 3], to: [f32; 3], left: f32) -> Option<[f32; 3]> {
    (left > 0.001).then(|| [(to[0] - from[0]) / left, 0.0, (to[2] - from[2]) / left])
}

/// The integrated next spot toward `goal`. `left` is the caller's
/// `distance(position, goal)` and the caller guards it above 1 mm; `Ok` is
/// snapped by `ground`, and `Err` carries the same integrated spot when ground
/// refuses it (`Ground::field_position`), so the caller can still apply its
/// "home leash before moving" rule first, exactly as the old two-step check
/// did. The bare `Ground::snap` walk in `tick_wanderers_for` stays inline
/// because it never refuses a step.
fn advance(
    position: [f32; 3],
    goal: [f32; 3],
    left: f32,
    step: f32,
    ground: &Ground,
) -> Result<[f32; 3], [f32; 3]> {
    let mut next = position;
    for (coord, to) in next.iter_mut().zip(goal) {
        *coord += (to - *coord) / left * step;
    }
    ground.field_position(next).ok_or(next)
}

/// Stop the walk animation if this actor was walking: the `moving` set is the
/// only record of the start/stop pairs, so one helper keeps them in step. The
/// movement frame is pushed by the caller before this one.
fn stand(guid: u64, moving: &mut BTreeSet<u64>, out: &mut EncounterOutcome) {
    if moving.remove(&guid) {
        out.replies.push(animation(guid, 1, true)); // loc_stand
    }
}

/// Start the walk animation if this actor was standing. `id` is 2 (`loc_walk`)
/// for a battle marker and 3 (`loc_run`) for a field actor.
fn walk(guid: u64, id: i32, moving: &mut BTreeSet<u64>, out: &mut EncounterOutcome) {
    if moving.insert(guid) {
        out.replies.push(animation(guid, id, true));
    }
}
