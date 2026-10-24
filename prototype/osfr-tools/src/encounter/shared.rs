//! Shared field state: one overworld enemy simulation per instance, shared by every participant, and its engine entry points.
use super::field::Wander;
use super::field_rewards::KillCredit;
use super::*;

/// (R) The one field-simulation value. The engine and the world each own one
/// `FieldSim`, and `with_shared_fields` swaps the whole value in and out for a
/// single action. A future per-field collection belongs in here, never as a
/// second engine field that has to be swapped separately.
#[derive(Clone, Default)]
pub(super) struct FieldSim {
    /// Field enemies: health, presence, motion, roam/leash and respawn clocks.
    pub(super) enemies: Vec<FieldEnemy>,
    /// Battle markers walking their four-point loop, by battle id.
    pub(super) wander: BTreeMap<u32, Wander>,
    /// Enemy GUID to its last attack time. `tick_encounter` and `tick_field`
    /// read and write it; `reset_session` clears it selectively on exit.
    pub(super) attacks: BTreeMap<u64, u64>,
    /// Enemy GUIDs whose walk animation is active. `tick_encounter`,
    /// `tick_field` and `tick_wanderers_for` write it, late-join initialisation
    /// reads it, and only `reset_session` clears it: never reset the whole
    /// struct, or a cleared walk would desync observers.
    pub(super) moving: BTreeSet<u64>,
    /// Hit times and unclaimed credits for shared-field kills.
    pub(super) kills: KillLedger,
}

/// (R) Shared-field kill bookkeeping for power-up drops (W158 slice 2b).
#[derive(Clone, Default)]
pub(super) struct KillLedger {
    /// enemy guid -> player guid -> time of that player's last damage.
    pub(super) hits: BTreeMap<u64, BTreeMap<u64, u64>>,
    /// player guid -> credits (reward and kill event) earned by another's kill.
    pub(super) credits: BTreeMap<u64, Vec<KillCredit>>,
}

/// (R) Parsed intents only. Session supplies identity, position and instance.
#[derive(Clone, Debug)]
pub enum SharedFieldAction {
    Ability { slot: i32, target: u64 },
    Interact { target: u64, flag: u8 },
    Select { target: u64, choice: i32 },
}

impl SharedFieldAction {
    pub fn target(&self) -> u64 {
        match *self {
            Self::Ability { target, .. }
            | Self::Interact { target, .. }
            | Self::Select { target, .. } => target,
        }
    }
    pub(crate) fn request(&self) -> Vec<u8> {
        match *self {
            Self::Ability { slot, target } => [
                header(36, 10),
                1_i32.to_le_bytes().to_vec(),
                slot.to_le_bytes().to_vec(),
                0_i32.to_le_bytes().to_vec(),
                target.to_le_bytes().to_vec(),
            ]
            .concat(),
            Self::Interact { target, flag } => {
                [header(26, 8), target.to_le_bytes().to_vec(), vec![flag]].concat()
            }
            Self::Select { target, choice } => [
                header(26, 10),
                target.to_le_bytes().to_vec(),
                choice.to_le_bytes().to_vec(),
            ]
            .concat(),
        }
    }
}

/// (R) One bounded instance population. No player health, energy or cooldowns.
/// World owns this outside cloneable session/social staging.
#[derive(Clone)]
pub struct SharedFieldState {
    /// The one simulation value: enemies, marker walks, attack clocks, walk
    /// state and kill credits. A future field collection must live inside
    /// `FieldSim`, not here or on the engine.
    pub(super) field_sim: FieldSim,
    pub(super) dots: BTreeMap<u64, BTreeMap<(u64, i32), SkillDot>>,
    pub(super) stuns: BTreeMap<u64, u64>,
    pub(super) target_buffs: BTreeMap<(u64, i32), SkillBuff>,
    pub(super) last_tick: Option<u64>,
    pub(super) template: Arc<Content>,
    pub(super) battles: Arc<Vec<FieldBattle>>,
}

impl SharedFieldState {
    pub fn enemies_targeting(&self, guid: u64) -> bool {
        self.field_sim.enemies.iter().any(|f| {
            f.state.health > 0 && f.motion == FieldMotion::Chasing && f.target == Some(guid)
        })
    }
    pub(crate) fn restrict_to(&mut self, known: &BTreeSet<u64>) -> Vec<bool> {
        self.field_sim
            .enemies
            .iter_mut()
            .map(|f| {
                let present = f.present;
                f.present &= known.contains(&f.actor.guid);
                present
            })
            .collect()
    }
    pub(crate) fn restore_presence(&mut self, previous: Vec<bool>) {
        for (f, present) in self.field_sim.enemies.iter_mut().zip(previous) {
            f.present = present;
        }
    }
    pub fn health(&self, guid: u64) -> Option<u32> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .map(|f| f.state.health)
    }
    pub fn respawn_at(&self, guid: u64) -> Option<u64> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == guid)?
            .respawn_at
    }
    pub fn position(&self, guid: u64) -> Option<[f32; 3]> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .map(|f| f.state.position)
            .or_else(|| {
                self.battles
                    .iter()
                    .find(|b| b.spawner.guid == guid)
                    .map(|b| {
                        self.field_sim
                            .wander
                            .get(&b.id)
                            .map_or(b.spawner.position, |w| w.position)
                    })
            })
    }
    pub fn attack_clock(&self, guid: u64) -> Option<u64> {
        self.field_sim.attacks.get(&guid).copied()
    }
    pub fn target(&self, guid: u64) -> Option<u64> {
        self.field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .and_then(|f| f.target)
    }
    /// (R) Leaving the ready instance drops its caster-owned DoTs. No disconnected
    /// account can retain an unbounded effect entry or a second ticking path.
    ///
    /// (R) A kill credit already earned survives the player's absence until its
    /// TTL (F276: a reconnect must still collect the earned reward). The
    /// field-owned drop and the quest event do not: a player who left the field
    /// keeps only the coin/job-XP pair (`KillCredit::retain_after_leave`). A
    /// quest-only credit has no such pair, so it is dropped here rather than
    /// kept for a player who left.
    pub fn retain_casters(&mut self, ready: &BTreeSet<u64>, now_ms: u64) {
        self.dots
            .retain(|caster, dots| ready.contains(caster) && !dots.is_empty());
        // Players who left the field neither share future kills nor keep drops.
        for (player, credits) in &mut self.field_sim.kills.credits {
            let present = ready.contains(player);
            credits.retain_mut(|credit| {
                if now_ms >= credit.expires_ms {
                    return false;
                }
                if present {
                    return true;
                }
                credit.retain_after_leave()
            });
        }
        self.field_sim
            .kills
            .credits
            .retain(|_, credits| !credits.is_empty());
        for hitters in self.field_sim.kills.hits.values_mut() {
            hitters.retain(|player, _| ready.contains(player));
        }
        self.field_sim
            .kills
            .hits
            .retain(|_, hitters| !hitters.is_empty());
    }

    /// Pending kill credits for one player (tests and diagnostics).
    pub fn pending_kill_credits(&self, player: u64) -> usize {
        self.field_sim
            .kills
            .credits
            .get(&player)
            .map_or(0, Vec::len)
    }
    pub fn due(&mut self, now_ms: u64) -> Option<u64> {
        let elapsed = self
            .last_tick
            .map_or(100, |last| now_ms.saturating_sub(last));
        if elapsed < 100 {
            return None;
        }
        self.last_tick = Some(now_ms);
        Some(elapsed)
    }
    /// (R) Eligibility changes never reset health or corpse deadlines.
    pub fn set_active_quests(&mut self, active: &BTreeSet<u32>) {
        for f in &mut self.field_sim.enemies {
            let eligible =
                f.quest_ids.is_empty() || f.quest_ids.iter().any(|id| active.contains(id));
            f.present = eligible || f.motion != FieldMotion::Idle && f.state.health > 0;
            if !eligible && f.motion == FieldMotion::Chasing {
                f.motion = FieldMotion::Returning;
                f.target = None;
            }
        }
    }
    /// (R) Candidates for the common NPC budget. Dead rows are retained only
    /// for existing observers until their committed lethal HP delta is flushed.
    pub fn interest_candidates(&self, hold_deaths: bool) -> Vec<crate::world::NpcInterest> {
        self.field_sim
            .enemies
            .iter()
            .filter(|f| f.present && (f.state.health > 0 || hold_deaths))
            .map(|f| crate::world::NpcInterest {
                guid: f.actor.guid,
                position: f.state.position,
                engaged: f.motion == FieldMotion::Chasing,
            })
            .chain(self.battles.iter().map(|b| crate::world::NpcInterest {
                guid: b.spawner.guid,
                position: self.position(b.spawner.guid).unwrap_or(b.spawner.position),
                engaged: false,
            }))
            .collect()
    }
    pub fn initialization(&self, guid: u64) -> io::Result<Vec<Vec<u8>>> {
        if let Some(f) = self
            .field_sim
            .enemies
            .iter()
            .find(|f| f.actor.guid == guid && f.present && f.state.health > 0)
        {
            return Ok(vec![
                spawn_actor(&self.template.template, &f.actor, f.state.position, 0),
                enemy_hp(guid, f.state.health, f.actor.max_health),
                // Late observers start with current status and motion (R).
                animation(
                    guid,
                    if self.stuns.contains_key(&guid) {
                        1401
                    } else if self.field_sim.moving.contains(&guid) {
                        3
                    } else {
                        1
                    },
                    true,
                ),
                relevance(&[guid], false),
            ]);
        }
        if let Some(b) = self.battles.iter().find(|b| b.spawner.guid == guid) {
            let mut spawn = crate::world::content_spawn_template();
            spawn.actor_id = guid;
            spawn.name_string_id = b.spawner.name_id;
            spawn.model_id = b.spawner.model_id;
            spawn.nameplate_image_id = b.nameplate_image_id;
            if b.nameplate_image_id > 0 {
                spawn.active_profile = 0;
            }
            let at = self.position(guid).unwrap_or(b.spawner.position);
            spawn.position = [at[0], at[1], at[2], 1.];
            spawn.rotation = [0., 0., 1., 0.];
            // Same literal-name writer used by passive NPC startup.
            spawn.npc_name = b.spawner.name.clone();
            return Ok(vec![
                crate::world::unframe_tunnel(&crate::startup_world::build_add_npc(&spawn))?,
                relevance(&[guid], false),
            ]);
        }
        Ok(Vec::new())
    }
    pub fn removal(guid: u64) -> Vec<u8> {
        remove(guid)
    }
}

impl EncounterEngine {
    /// (R) Transfer mutable field state once; retain immutable target metadata.
    pub fn take_shared_fields(&mut self) -> Option<SharedFieldState> {
        if self.shared_fields.is_some() {
            return None;
        }
        self.shared_fields = Some(
            self.field_sim
                .enemies
                .iter()
                .map(|f| f.actor.guid)
                .chain(self.battles.iter().map(|b| b.spawner.guid))
                .collect(),
        );
        Some(SharedFieldState {
            field_sim: FieldSim {
                enemies: std::mem::take(&mut self.field_sim.enemies),
                wander: std::mem::take(&mut self.field_sim.wander),
                ..FieldSim::default()
            },
            dots: BTreeMap::new(),
            stuns: BTreeMap::new(),
            target_buffs: BTreeMap::new(),
            last_tick: None,
            template: self.content.clone(),
            battles: Arc::new(self.battles.clone()),
        })
    }

    pub fn shared_field_guids(&self) -> impl Iterator<Item = u64> + '_ {
        self.shared_fields
            .iter()
            .flat_map(|guids| guids.iter().copied())
    }

    /// Parse once at the existing Gameplay seam. Malformed targets never become intents.
    pub fn shared_action(
        &self,
        inner: &[u8],
        character: &CharacterState,
    ) -> io::Result<Option<SharedFieldAction>> {
        if self.shared_fields.is_none()
            || self.shared_sender.is_some()
            || self.active()
            || character.zone_id != self.content.world.zone_id
        {
            return Ok(None);
        }
        let known = |guid| {
            self.shared_fields
                .as_ref()
                .is_some_and(|s| s.contains(&guid))
        };
        if inner.starts_with(&[36, 0, 10, 0]) {
            if inner.len() < 16 {
                return Err(invalid("truncated shared ability"));
            }
            let bar = i32::from_le_bytes(inner[4..8].try_into().unwrap());
            let slot = i32::from_le_bytes(inner[8..12].try_into().unwrap());
            let kind = i32::from_le_bytes(inner[12..16].try_into().unwrap());
            if kind == 1 {
                if inner.len() != 32 {
                    return Err(invalid("malformed shared ground ability"));
                }
                return Ok(None);
            }
            if kind != 0 || inner.len() != 24 {
                return Err(invalid("malformed shared target ability"));
            }
            if bar != 1 || !(0..=9).contains(&slot) {
                return Ok(None);
            }
            let target = u64::from_le_bytes(inner[16..24].try_into().unwrap());
            // Personal story actors keep their own route, including self selection.
            if self.enemy(target).is_some()
                || (target == character.guid
                    && self.state.world_enemies.iter().any(|enemy| {
                        self.enemy(enemy.guid).is_some()
                            && enemy.health > 0
                            && distance(character.position, enemy.position)
                                <= self.scale.profile_stats(self.player_profile, 0).0
                    }))
            {
                return Ok(None);
            }
            return Ok(Some(SharedFieldAction::Ability { slot, target }));
        }
        if inner.starts_with(&[26, 0, 8, 0]) || inner.starts_with(&[26, 0, 10, 0]) {
            let interaction = inner[2] == 8;
            if inner.len() != if interaction { 13 } else { 16 } || interaction && inner[12] > 1 {
                return Err(invalid("malformed shared interaction"));
            }
            let target = u64::from_le_bytes(inner[4..12].try_into().unwrap());
            if known(target) {
                return Ok(Some(if interaction {
                    SharedFieldAction::Interact {
                        target,
                        flag: inner[12],
                    }
                } else {
                    SharedFieldAction::Select {
                        target,
                        choice: i32::from_le_bytes(inner[12..16].try_into().unwrap()),
                    }
                }));
            }
        }
        Ok(None)
    }

    /// (R) Borrow canonical rows only while calculating one staged action. The
    /// player engine is never stored in World and never retains these rows.
    pub(crate) fn with_shared_fields<T>(
        &mut self,
        runtime: &mut SharedFieldState,
        sender: u64,
        action: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let resets = runtime
            .field_sim
            .enemies
            .iter()
            .map(|f| (f.actor.guid, f.reset_serial))
            .collect::<BTreeMap<_, _>>();
        let old_sender = self.shared_sender.replace(sender);
        std::mem::swap(&mut self.field_sim, &mut runtime.field_sim);
        let personal_dots = std::mem::replace(
            &mut self.skills.dots,
            runtime.dots.remove(&sender).unwrap_or_default(),
        );
        let personal_stuns =
            std::mem::replace(&mut self.skills.stuns, std::mem::take(&mut runtime.stuns));
        let personal_target_buffs = std::mem::replace(
            &mut self.skills.target_buffs,
            std::mem::take(&mut runtime.target_buffs),
        );
        let result = action(self);
        runtime.dots.insert(
            sender,
            std::mem::replace(&mut self.skills.dots, personal_dots),
        );
        runtime.dots.retain(|_, dots| !dots.is_empty());
        runtime.stuns = std::mem::replace(&mut self.skills.stuns, personal_stuns);
        runtime.target_buffs =
            std::mem::replace(&mut self.skills.target_buffs, personal_target_buffs);
        std::mem::swap(&mut self.field_sim, &mut runtime.field_sim);
        self.shared_sender = old_sender;
        // Evade/defeat invalidates every caster's effects on that incarnation.
        for f in &runtime.field_sim.enemies {
            if f.state.health == 0
                || f.motion == FieldMotion::Returning
                || resets.get(&f.actor.guid) != Some(&f.reset_serial)
            {
                for dots in runtime.dots.values_mut() {
                    dots.retain(|(guid, _), _| *guid != f.actor.guid);
                }
            }
        }
        result
    }

    pub fn shared_attack(
        &mut self,
        runtime: &mut SharedFieldState,
        sender: u64,
        target: u64,
        position: [f32; 3],
        profile: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        self.with_shared_fields(runtime, sender, |engine| {
            engine.attack(target, position, profile, 0, now_ms)
        })
    }

    pub fn shared_job_ability(
        &mut self,
        runtime: &mut SharedFieldState,
        sender: u64,
        ability: &crate::opening_abilities::JobAbility,
        target: u64,
        position: [f32; 3],
        profile: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        self.with_shared_fields(runtime, sender, |engine| {
            if target != sender
                && target != 0
                && !engine.enemy(target).is_some_and(|(_, s)| s.health > 0)
            {
                return Ok(EncounterOutcome::default());
            }
            engine.job_ability(ability, target, position, profile, now_ms)
        })
    }

    pub fn shared_effects(
        &mut self,
        runtime: &mut SharedFieldState,
        sender: u64,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        self.with_shared_fields(runtime, sender, |engine| {
            let mut out = EncounterOutcome::default();
            engine.tick_job_effects(now_ms, &mut out)?;
            engine.claim_kill_credits(sender, now_ms, &mut out);
            Ok(out)
        })
    }

    pub fn shared_motion(
        &mut self,
        runtime: &mut SharedFieldState,
        players: &[FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> io::Result<(EncounterOutcome, Vec<(u64, u32)>)> {
        self.with_shared_fields(runtime, 0, |engine| {
            let mut out = EncounterOutcome::default();
            let mut damage = 0;
            let mut hits = Vec::new();
            engine.tick_field(
                [0.; 3],
                now_ms,
                elapsed,
                &mut out,
                &mut damage,
                Some(players),
                &mut hits,
            );
            // Canonical marker clock/transform; each marker considers the nearest observer.
            engine.tick_wanderers_for(players, now_ms, elapsed, &mut out);
            Ok((out, hits))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const TEST_ENEMY: u64 = 131000108001;
    const TEST_BATTLE: u32 = 96;

    /// A minimal content directory: `load` requires one dungeon, and the field
    /// file supplies one enemy for `take_shared_fields` to move.
    fn engine() -> (std::path::PathBuf, EncounterEngine) {
        static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "osfr-field-sim-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0_u8; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = serde_json::json!({"zone_id":0,"name":"0","sky":"","geometry_id":214,"position":[0,0,0],"rotation":[0,0,1],"tutorial":false});
        let mut cave = zone.clone();
        cave["zone_id"] = serde_json::json!(411);
        let actor = serde_json::json!({"guid":300,"definition_id":930110,"model_id":3,"name_id":100,"name":"930110","texture_alias":"","position":[0,0,0],"max_health":300,"damage":0,"boss":false});
        fs::write(
            dir.join("encounters.json"),
            serde_json::json!({
                "schema_version": 1,
                "template_file": "npc.bin",
                "template_sha256": format!("{:x}", Sha256::digest(&template)),
                "world": zone,
                "world_enemies": [],
                "dungeons": [{
                    "id": 411,
                    "title_id": 12,
                    "description_id": 13,
                    "objective_id": 14,
                    "zone": cave,
                    "guide": null,
                    "enemies": [actor],
                    "reward_coins": 0,
                    "reward_items": []
                }]
            })
            .to_string(),
        )
        .unwrap();
        let enemy = serde_json::json!({"guid":TEST_ENEMY,"definition_id":930108,"model_id":3,"name_id":100,"name":"930108","texture_alias":"","position":[0,0,0],"max_health":300,"damage":7,"boss":false,"respawn_seconds":2,"aggro_range":8,"leash_radius":12,"behaviour":{"mode":"stationary","reach":2.5,"attack_ms":1000}});
        fs::write(
            dir.join("field_enemies.json"),
            serde_json::json!({"schema_version": 1, "enemies": [enemy]}).to_string(),
        )
        .unwrap();
        let mut engine = EncounterEngine::load(&dir, None).unwrap();
        engine
            .load_field_enemies(&dir.join("field_enemies.json"))
            .unwrap();
        (dir, engine)
    }

    #[test]
    fn take_shared_fields_moves_only_enemies_and_walks() {
        let (dir, mut engine) = engine();
        engine.field_sim.attacks.insert(TEST_ENEMY, 11);
        engine.field_sim.moving.insert(TEST_ENEMY);
        engine
            .field_sim
            .kills
            .hits
            .entry(TEST_ENEMY)
            .or_default()
            .insert(101, 22);
        engine.field_sim.wander.insert(
            TEST_BATTLE,
            Wander {
                position: [1., 2., 3.],
                waypoint: 2,
                pause_until: 4,
            },
        );

        let runtime = engine.take_shared_fields().unwrap();

        assert_eq!(runtime.field_sim.enemies.len(), 1, "enemies transfer");
        assert_eq!(runtime.field_sim.enemies[0].actor.guid, TEST_ENEMY);
        assert_eq!(
            runtime.field_sim.wander.get(&TEST_BATTLE).unwrap().position,
            [1., 2., 3.],
            "walks transfer"
        );
        assert!(runtime.field_sim.attacks.is_empty(), "attacks start empty");
        assert!(runtime.field_sim.moving.is_empty(), "moving starts empty");
        assert!(runtime.field_sim.kills.hits.is_empty(), "hits start empty");
        assert!(
            runtime.field_sim.kills.credits.is_empty(),
            "credits start empty"
        );
        assert_eq!(runtime.pending_kill_credits(101), 0);
        // Today's selective transfer: the engine keeps its own clocks and ledger.
        assert_eq!(engine.field_sim.attacks.get(&TEST_ENEMY), Some(&11));
        assert!(engine.field_sim.moving.contains(&TEST_ENEMY));
        assert!(engine.field_sim.kills.hits.contains_key(&TEST_ENEMY));
        assert!(
            engine.field_sim.enemies.is_empty(),
            "the live enemies moved out"
        );
        assert!(
            engine.field_sim.wander.is_empty(),
            "the live walks moved out"
        );
        let _ = fs::remove_dir_all(dir);
    }
}
