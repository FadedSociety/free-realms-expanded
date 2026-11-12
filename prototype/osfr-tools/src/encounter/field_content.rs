//! Field content loading, validation and staged reloads (`field_enemies.json`,
//! `field_battles.json`), and the quest pin's lookups over the loaded rows.
use super::behaviour::resolve_behaviour;
use super::*;

impl EncounterEngine {
    /// Load the overworld enemies (`field_enemies.json`). Their GUIDs must not
    /// collide with opening, dungeon or battle-spawner actors.
    pub fn load_field_enemies(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        // L1191: 256 KiB as for field_camps.json; the three instance fields on all 52 rows passed 128.
        if bytes.len() > 256 * 1024 {
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
            combat_marker,
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
            check_sources("field spawn group", &group.source)?;
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
        let mut guids = self.claimed_guids();
        let mut field = Vec::with_capacity(enemies.len());
        let mut areas = BTreeSet::new();
        for f in enemies {
            field.push(self.build_field_enemy(f, &mut guids, &mut areas, combat_marker)?);
        }
        Arc::make_mut(&mut self.content).field_sites = field
            .iter()
            .map(|f| (f.actor.definition_id, f.actor.guid, f.actor.position))
            .collect();
        self.field_sim.enemies = field;
        self.paid_revive_cost = paid_revive_cost;
        self.rebuild_behaviours()
    }

    /// GUIDs the opening, dungeons and battles already claim: a field enemy may not reuse one.
    pub(in crate::encounter) fn claimed_guids(&self) -> BTreeSet<u64> {
        self.content
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
            .collect()
    }

    /// Validate one authored field actor and give it its live state. `guids` and `areas` are
    /// the ids already taken; a row that passes claims its own.
    pub(super) fn build_field_enemy(
        &self,
        mut f: FieldActor,
        guids: &mut BTreeSet<u64>,
        areas: &mut BTreeSet<String>,
        combat_marker: Option<Notification>,
    ) -> io::Result<FieldEnemy> {
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
        // (R) Authored stats are level-one baselines. Reuse the final-era HP
        // curve and weapon tiers, rather than inventing per-kind curves.
        self.scale.scale_field_enemy(&mut f.actor);
        let marker = f.combat_marker.resolve(combat_marker, f.actor.name_id)?;
        if f.instance_enemy && f.instance.is_none() {
            return Err(invalid("an instance_enemy field enemy needs its instance guid"));
        }
        Ok(FieldEnemy {
            instance: f.instance.filter(|_| f.instance_enemy),
            attacks: f.attacks_in_overworld,
            marker,
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
            threat: Default::default(),
        })
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

    /// Owner 2026-11-12: world objects (`npc_spawns_extra.json` records and regional `npcs/*.json`
    /// rows) with `interactable` and an `instance` starter guid open that starter's start card
    /// when clicked, as its own entrance does. Call after the battles load; an `instance` that is
    /// no battle starter or entrance is skipped and logged (`tests/entrances.rs` refuses it).
    pub fn load_object_links(&mut self, directory: &Path) -> io::Result<()> {
        let mut paths = vec![directory.join("npc_spawns_extra.json")];
        if let Ok(entries) = fs::read_dir(directory.join("npcs")) {
            paths.extend(
                entries
                    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                    .filter(|path| path.extension().is_some_and(|ext| ext == "json")),
            );
        }
        let mut links = std::collections::BTreeMap::new();
        for path in paths {
            let Ok(bytes) = fs::read(&path) else {
                continue;
            };
            let file: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
            for row in file["records"].as_array().into_iter().flatten() {
                if row["interactable"] != true {
                    continue;
                }
                let guid = row["spawn_guid"].as_u64().or_else(|| row["npc_id"].as_u64());
                let at = serde_json::from_value::<[f32; 3]>(row["position"].clone()).ok();
                let (Some(guid), Some(at), Some(starter)) = (guid, at, row["instance"].as_u64())
                else {
                    continue;
                };
                if !self.is_battle_spawner(starter) {
                    eprintln!("object {guid}: instance {starter} is no battle starter; skipped");
                    continue;
                }
                links.insert(guid, (at, starter));
            }
        }
        Arc::make_mut(&mut self.content).object_links = links;
        Ok(())
    }

    /// Load overworld battles (`field_battles.json`); spawner and enemy GUIDs
    /// must not collide with any other encounter actor.
    pub fn load_field_battles(&mut self, path: &Path) -> io::Result<()> {
        let bytes = fs::read(path)?;
        // 1 MiB, as `field_kill_rewards.json`: per-class wheels (L1194: up to 8 rows for each of
        // seven jobs) put the 64-battle ceiling well past the old 128 KiB.
        if bytes.len() > 1024 * 1024 {
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
            if !card::wheel_ok(&b.prizes) {
                return Err(invalid("invalid field battle prizes"));
            }
            battle_drops::validate(&b.drops, self.field_rewards.loot_tables())?;
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
            let difficulty = b.difficulty;
            for a in &mut b.enemies {
                self.scale.scale_battle_enemy(a, difficulty);
                // A row's enemy drops by its rank too; a summoned minion does not.
                if a.power_up_table.is_none() && a.summon_below_percent.is_none() {
                    let rank = a.rank.unwrap_or(if a.boss { Rank::Boss } else { Rank::Normal });
                    a.power_up_table = super::pickups::rank_table(rank);
                }
            }
            b.marker = b
                .combat_marker
                .resolve(file.combat_marker, b.spawner.name_id)?;
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
                wheel_coins: b.wheel_coins,
            });
        }
        self.battles = battles;
        // The doors and starters of `entrances.json` beside this file join the list.
        if let Some(directory) = path.parent() {
            self.place_entrances(directory)?;
        }
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
            // A fight file's arena row is not this file's, though an entrance starts it.
            let fights = engine.fight_state.fights.clone();
            let old: BTreeSet<u32> = (engine.battles.iter().map(|b| b.id))
                .filter(|id| !fights.contains_key(id))
                .collect();
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
            if let Some(directory) = path.parent() {
                engine.load_field_camps(directory)?;
            }
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

/// A content row's provenance: every source names its evidence, and at least one reconstructed
/// source starts with `(R)`, so a guess is never presented as a recovered fact.
pub(super) fn check_sources(what: &str, sources: &[serde_json::Value]) -> io::Result<()> {
    for source in sources {
        let Some(evidence) = source.get("evidence").and_then(|v| v.as_str()) else {
            return Err(invalid(&format!("{what} source needs evidence")));
        };
        if evidence.trim().is_empty() {
            return Err(invalid(&format!("invalid {what} source")));
        }
        if source.get("kind").and_then(|v| v.as_str()) == Some("reconstructed")
            && !evidence.starts_with("(R)")
        {
            return Err(invalid(&format!("reconstructed {what} source needs (R)")));
        }
    }
    if !sources.iter().any(|source| {
        source.get("kind").and_then(|v| v.as_str()) == Some("reconstructed")
            && source
                .get("evidence")
                .and_then(|v| v.as_str())
                .is_some_and(|evidence| evidence.starts_with("(R)"))
    }) {
        return Err(invalid(&format!("{what} needs reconstructed provenance")));
    }
    Ok(())
}
