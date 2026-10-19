//! Shared field state: one overworld enemy simulation per instance, shared by every participant, and its engine entry points.
use super::*;

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
    pub(super) enemies: Vec<FieldEnemy>,
    pub(super) wander: BTreeMap<u32, Wander>,
    pub(super) attacks: BTreeMap<u64, u64>,
    pub(super) moving: BTreeSet<u64>,
    pub(super) dots: BTreeMap<u64, BTreeMap<(u64, i32), SkillDot>>,
    pub(super) stuns: BTreeMap<u64, u64>,
    pub(super) target_buffs: BTreeMap<(u64, i32), SkillBuff>,
    pub(super) last_tick: Option<u64>,
    pub(super) template: Arc<Content>,
    pub(super) battles: Arc<Vec<FieldBattle>>,
    pub(super) kills: KillLedger,
}

impl SharedFieldState {
    pub fn enemies_targeting(&self, guid: u64) -> bool {
        self.enemies.iter().any(|f| {
            f.state.health > 0 && f.motion == FieldMotion::Chasing && f.target == Some(guid)
        })
    }
    pub(crate) fn restrict_to(&mut self, known: &BTreeSet<u64>) -> Vec<bool> {
        self.enemies
            .iter_mut()
            .map(|f| {
                let present = f.present;
                f.present &= known.contains(&f.actor.guid);
                present
            })
            .collect()
    }
    pub(crate) fn restore_presence(&mut self, previous: Vec<bool>) {
        for (f, present) in self.enemies.iter_mut().zip(previous) {
            f.present = present;
        }
    }
    pub fn health(&self, guid: u64) -> Option<u32> {
        self.enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .map(|f| f.state.health)
    }
    pub fn respawn_at(&self, guid: u64) -> Option<u64> {
        self.enemies
            .iter()
            .find(|f| f.actor.guid == guid)?
            .respawn_at
    }
    pub fn position(&self, guid: u64) -> Option<[f32; 3]> {
        self.enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .map(|f| f.state.position)
            .or_else(|| {
                self.battles
                    .iter()
                    .find(|b| b.spawner.guid == guid)
                    .map(|b| {
                        self.wander
                            .get(&b.id)
                            .map_or(b.spawner.position, |w| w.position)
                    })
            })
    }
    pub fn attack_clock(&self, guid: u64) -> Option<u64> {
        self.attacks.get(&guid).copied()
    }
    pub fn target(&self, guid: u64) -> Option<u64> {
        self.enemies
            .iter()
            .find(|f| f.actor.guid == guid)
            .and_then(|f| f.target)
    }
    /// (R) Leaving the ready instance drops its caster-owned DoTs. No disconnected
    /// account can retain an unbounded effect entry or a second ticking path.
    pub fn retain_casters(&mut self, ready: &BTreeSet<u64>) {
        self.dots
            .retain(|caster, dots| ready.contains(caster) && !dots.is_empty());
        // Players who left the field neither share future kills nor keep drops.
        self.kills
            .credits
            .retain(|player, _| ready.contains(player));
        for hitters in self.kills.hits.values_mut() {
            hitters.retain(|player, _| ready.contains(player));
        }
        self.kills.hits.retain(|_, hitters| !hitters.is_empty());
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
        for f in &mut self.enemies {
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
        self.enemies
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
                    } else if self.moving.contains(&guid) {
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
            self.field
                .iter()
                .map(|f| f.actor.guid)
                .chain(self.battles.iter().map(|b| b.spawner.guid))
                .collect(),
        );
        Some(SharedFieldState {
            enemies: std::mem::take(&mut self.field),
            wander: std::mem::take(&mut self.wander),
            attacks: BTreeMap::new(),
            moving: BTreeSet::new(),
            dots: BTreeMap::new(),
            stuns: BTreeMap::new(),
            target_buffs: BTreeMap::new(),
            last_tick: None,
            template: self.content.clone(),
            battles: Arc::new(self.battles.clone()),
            kills: KillLedger::default(),
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
                                <= profile_stats(self.player_profile, 0).0
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
            .enemies
            .iter()
            .map(|f| (f.actor.guid, f.reset_serial))
            .collect::<BTreeMap<_, _>>();
        let old_sender = self.shared_sender.replace(sender);
        std::mem::swap(&mut self.field, &mut runtime.enemies);
        std::mem::swap(&mut self.wander, &mut runtime.wander);
        std::mem::swap(&mut self.enemy_attacks, &mut runtime.attacks);
        std::mem::swap(&mut self.moving, &mut runtime.moving);
        std::mem::swap(&mut self.ledger, &mut runtime.kills);
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
        std::mem::swap(&mut self.field, &mut runtime.enemies);
        std::mem::swap(&mut self.wander, &mut runtime.wander);
        std::mem::swap(&mut self.enemy_attacks, &mut runtime.attacks);
        std::mem::swap(&mut self.moving, &mut runtime.moving);
        std::mem::swap(&mut self.ledger, &mut runtime.kills);
        self.shared_sender = old_sender;
        // Evade/defeat invalidates every caster's effects on that incarnation.
        for f in &runtime.enemies {
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
