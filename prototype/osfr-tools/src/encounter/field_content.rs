//! Field content loading, validation and staged reloads (`field_enemies.json`,
//! `field_battles.json`), and the quest pin's lookups over the loaded rows.
use super::*;

impl EncounterEngine {
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
            let level = f.actor.level;
            self.scale.scale_enemy(
                &mut f.actor,
                self.scale.health_factor(level),
                self.scale.tier_factor(level),
            );
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
        Arc::make_mut(&mut self.content).field_sites = field
            .iter()
            .map(|f| (f.actor.definition_id, f.actor.guid, f.actor.position))
            .collect();
        self.field_sim.enemies = field;
        self.paid_revive_cost = paid_revive_cost;
        self.rebuild_behaviours()
    }

    /// Guid and home position of the lowest-guid field enemy of this kind: where a kill
    /// objective's "Take Me There" pin points (R). Present or not, so it works for a quest
    /// whose enemies appear only once it is active, and read from the loaded rows, so it
    /// still answers once the live enemies moved into the shared field (T427).
    pub fn field_enemy_site(&self, definition_id: u32) -> Option<(u64, [f32; 3])> {
        let definition_id = i32::try_from(definition_id).ok()?;
        self.content
            .field_sites
            .iter()
            .filter(|site| site.0 == definition_id)
            .min_by_key(|site| site.1)
            .map(|&(_, guid, home)| (guid, home))
    }

    /// Guid and position of field battle `id`'s spawner, its zone-0 marker: where a
    /// dungeon objective's pin points. Cave dungeons have no marker.
    pub fn battle_marker(&self, id: u32) -> Option<(u64, [f32; 3])> {
        let battle = self.battles.iter().find(|b| b.id == id)?;
        Some((battle.spawner.guid, battle.spawner.position))
    }

    /// The marker of the lowest-id battle whose spawner or roster is of this kind (R):
    /// where a kill objective points when no field enemy of the kind exists.
    pub fn battle_marker_for_enemy(&self, definition_id: u32) -> Option<(u64, [f32; 3])> {
        let definition_id = i32::try_from(definition_id).ok()?;
        let battle = self
            .battles
            .iter()
            .filter(|b| {
                std::iter::once(&b.spawner)
                    .chain(&b.enemies)
                    .any(|a| a.definition_id == definition_id)
            })
            .min_by_key(|b| b.id)?;
        Some((battle.spawner.guid, battle.spawner.position))
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
            battle_drops::validate(&b.drops)?;
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
            let factor = self.scale.health_factor(difficulty_level(b.difficulty));
            for a in &mut b.enemies {
                self.scale.scale_enemy(a, factor, factor);
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
}
