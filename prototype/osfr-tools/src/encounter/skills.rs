//! Job skills: abilities, damage over time, buffs, stuns, mitigation, energy and critical hits.
use super::*;

#[derive(Clone)]
pub(super) struct SkillDot {
    pub(super) damage: u32,
    pub(super) next_ms: u64,
    pub(super) until_ms: u64,
    pub(super) interval_ms: u64,
}

#[derive(Clone)]
pub(super) struct SkillBuff {
    pub(super) until_ms: u64,
    pub(super) damage: f32,
    pub(super) taken: f32,
    pub(super) crit: u32,
    pub(super) crit_damage: f32,
}

/// (R) Session-only skill state: DoTs, stuns and buffs. Reapplication refreshes
/// one entry per target/definition; caster buffs are private to the player,
/// while stuns and target buffs are shared with the field.
#[derive(Clone, Default)]
pub(super) struct SkillEffects {
    pub(super) dots: BTreeMap<(u64, i32), SkillDot>,
    pub(super) stuns: BTreeMap<u64, u64>,
    pub(super) caster_buffs: BTreeMap<i32, SkillBuff>,
    pub(super) target_buffs: BTreeMap<(u64, i32), SkillBuff>,
}

impl SkillEffects {
    pub(super) fn clear(&mut self) {
        self.dots.clear();
        self.stuns.clear();
        self.caster_buffs.clear();
        self.target_buffs.clear();
    }

    /// A defeated or evaded incarnation ends: drop every effect on it.
    pub(super) fn forget_target(&mut self, guid: u64) {
        self.dots.retain(|(target, _), _| *target != guid);
        self.stuns.remove(&guid);
        self.target_buffs.retain(|(target, _), _| *target != guid);
    }

    pub(super) fn expire(&mut self, now_ms: u64) {
        self.stuns.retain(|_, until| now_ms < *until);
        self.caster_buffs.retain(|_, buff| now_ms < buff.until_ms);
        self.target_buffs.retain(|_, buff| now_ms < buff.until_ms);
    }

    pub(super) fn stunned(&self, guid: u64, now_ms: u64) -> bool {
        self.stuns.get(&guid).is_some_and(|until| now_ms < *until)
    }

    /// (R) Strongest modifier wins; repeated casts refresh instead of stacking.
    pub(super) fn damage(&self, target: u64, damage: u32) -> u32 {
        let outgoing = self
            .caster_buffs
            .values()
            .map(|buff| buff.damage)
            .fold(1.0_f32, f32::max);
        let taken = self
            .target_buffs
            .iter()
            .filter(|((guid, _), _)| *guid == target)
            .map(|(_, buff)| buff.taken)
            .fold(1.0_f32, f32::max);
        (damage as f32 * outgoing * taken).round() as u32
    }

    pub(super) fn mitigation(&self) -> f32 {
        self.caster_buffs
            .values()
            .map(|buff| buff.taken)
            .fold(1.0_f32, f32::min)
    }

    pub(super) fn crit_bonus(&self) -> u32 {
        self.caster_buffs
            .values()
            .map(|buff| buff.crit)
            .max()
            .unwrap_or(0)
    }

    pub(super) fn crit_damage_bonus(&self) -> f32 {
        self.caster_buffs
            .values()
            .map(|buff| buff.crit_damage)
            .fold(0.0_f32, f32::max)
    }
}

pub(super) fn merge_skill_outcome(out: &mut EncounterOutcome, other: EncounterOutcome) {
    out.changed |= other.changed;
    out.replies.extend(other.replies);
    out.rewards.extend(other.rewards);
    out.defeated.extend(other.defeated);
    out.completed = other.completed.or(out.completed);
    out.win_completed = other.win_completed.or(out.win_completed);
    if other.feedback.is_some() {
        out.feedback = other.feedback;
    }
    if other.zoning.is_some() {
        out.zoning = other.zoning;
    }
}

impl EncounterEngine {
    /// A job ability (toolbar slot 1..): strike the targeted enemy (or the
    /// nearest one in reach, unshielded first) for a multiple of the basic
    /// attack, and/or heal the player. Refusals use the client's ability
    /// failure strings: 285 not enough energy, 287 not ready, 288 no target.
    pub fn job_ability(
        &mut self,
        ability: &crate::opening_abilities::JobAbility,
        target: u64,
        position: [f32; 3],
        profile_id: i32,
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        use crate::abilities::{build_energy, build_failure, build_hitpoint_delta};
        if !valid_position(position) {
            return Err(invalid("invalid job ability position"));
        }
        let refuse = |id| EncounterOutcome {
            replies: vec![build_failure(id)],
            ..Default::default()
        };
        if !ability.valid() || ability.unavailable {
            return Ok(refuse(286));
        }
        if self.state.player_health == 0
            || self
                .state
                .active
                .as_ref()
                .is_some_and(|a| a.phase != EncounterPhase::Fighting)
        {
            return Ok(refuse(286));
        }
        if self
            .ability_ready
            .get(&ability.definition_id)
            .is_some_and(|ready| now_ms < *ready)
        {
            return Ok(refuse(287));
        }
        if self.energy < ability.energy_cost {
            return Ok(refuse(285));
        }
        self.skills.expire(now_ms);
        let mut out = EncounterOutcome::default();
        let offensive =
            ability.damage_multiplier > 0.0 || ability.effects.iter().any(JobEffect::needs_target);
        let (basic_range, base) = profile_stats(profile_id, 0);
        let range = if ability.range > 0.0 {
            ability.range
        } else {
            basic_range
        };
        let base = (base as f32 * self.damage_scale).round() as u32;
        let mut targets = Vec::new();
        if offensive {
            let area = ability.effects.iter().find_map(|e| match *e {
                JobEffect::AreaDamage {
                    radius,
                    max_targets,
                    centered_on_caster,
                } => Some((radius, max_targets, centered_on_caster)),
                _ => None,
            });
            let nearby = self.living_enemies();
            let caster_area = area.is_some_and(|(_, _, caster)| caster);
            let select_range = if caster_area { area.unwrap().0 } else { range };
            // (R) A non-hostile/self target selects the nearest enemy; an explicit
            // hostile target out of reach fails instead of silently hitting another.
            let selected = nearby
                .iter()
                .copied()
                .find(|(guid, _)| *guid == target)
                .or_else(|| {
                    nearby
                        .iter()
                        .copied()
                        .filter(|(_, at)| distance(position, *at) <= select_range)
                        .min_by(|(a_guid, a_at), (b_guid, b_at)| {
                            (self.shielded(*a_guid), distance(position, *a_at))
                                .partial_cmp(&(self.shielded(*b_guid), distance(position, *b_at)))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                });
            let Some((selected_guid, selected_at)) = selected else {
                return Ok(refuse(288));
            };
            if distance(position, selected_at) > select_range {
                return Ok(refuse(288));
            }
            if self
                .last_attack
                .is_some_and(|last| now_ms.saturating_sub(last) < 600)
            {
                return Ok(refuse(287));
            }
            let center = if caster_area { position } else { selected_at };
            let (radius, limit, _) = area.unwrap_or((0.0, 1, false));
            targets = if area.is_some() {
                self.area_targets(center, radius, limit as usize, Some(selected_guid))
            } else {
                vec![selected_guid]
            };
            if targets.is_empty() {
                return Ok(refuse(288));
            }
            if self.active() && !self.armed {
                self.armed = true;
                self.last_tick = Some(now_ms);
            }
            self.last_attack = Some(now_ms);
            // All hits reuse the existing authoritative shield/summon/defeat/reward
            // path; secondary hits and later DoT ticks do not consume swing time.
            if ability.damage_multiplier > 0.0 {
                let damage = (base as f32 * ability.damage_multiplier).round() as u32;
                for &guid in &targets {
                    if let Some(at) = self.enemy(guid).map(|(_, s)| s.position) {
                        merge_skill_outcome(
                            &mut out,
                            self.strike_damage(guid, at, f32::MAX, damage, now_ms, false)?,
                        );
                    }
                }
            }
        }
        if ability.heal > 0 {
            let before = self.state.player_health;
            // (R) Keep the native-tested 250 at level one, scale by the existing
            // HP curve at later levels. Healing never revives a knocked-out actor.
            let heal = (u64::from(ability.heal) * u64::from(level_health(self.job_level))
                / u64::from(PLAYER_MAX)) as u32;
            self.state.player_health = before.saturating_add(heal).min(self.max_health);
            let healed = (self.state.player_health - before) as i32;
            self.log_fight(now_ms, "heal", "you", "you", i64::from(healed));
            out.replies.push(build_hitpoint_delta(1, 1, healed));
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
            out.changed = true;
        }
        for effect in &ability.effects {
            match *effect {
                JobEffect::AreaDamage { .. } => {} // target list already resolved above
                JobEffect::DamageOverTime {
                    multiplier,
                    duration_ms,
                    interval_ms,
                } => self.apply_dot(
                    &targets,
                    ability.definition_id,
                    (base as f32 * multiplier).round() as u32,
                    duration_ms,
                    interval_ms,
                    now_ms,
                ),
                JobEffect::Stun { duration_ms } => {
                    self.apply_stun(&targets, duration_ms, now_ms, &mut out)
                }
                JobEffect::Buff {
                    recipient,
                    duration_ms,
                    damage_multiplier,
                    damage_taken_multiplier,
                    crit_bonus,
                    crit_damage_bonus,
                } => {
                    let buff = SkillBuff {
                        until_ms: now_ms.saturating_add(duration_ms),
                        damage: damage_multiplier,
                        taken: damage_taken_multiplier,
                        crit: crit_bonus,
                        crit_damage: crit_damage_bonus,
                    };
                    if recipient == EffectRecipient::Caster {
                        self.apply_caster_buff(ability.definition_id, buff);
                    } else {
                        for &guid in &targets {
                            if self.enemy(guid).is_some_and(|(_, s)| s.health > 0)
                                && !self.shielded(guid)
                            {
                                self.skills
                                    .target_buffs
                                    .insert((guid, ability.definition_id), buff.clone());
                            }
                        }
                    }
                }
            }
        }
        // (R) Reuse verified same-job swing where exact clips are unrecovered.
        out.replies.push(animation(
            self.shared_sender.unwrap_or(1),
            if ability.animation_id > 0 {
                ability.animation_id
            } else {
                1099
            },
            false,
        ));
        if ability.effect_id > 0 {
            out.replies.push(composite_effect(
                self.shared_sender.unwrap_or(1),
                ability.effect_id,
                position,
            ));
        }
        self.energy -= ability.energy_cost;
        if ability.energy_cost > 0 {
            out.replies.push(build_energy(self.energy, ENERGY_MAX));
            self.energy_regen_at = now_ms.saturating_add(ENERGY_STEP_MS);
        }
        self.ability_ready.insert(
            ability.definition_id,
            now_ms.saturating_add(ability.cooldown_ms),
        );
        Ok(out)
    }

    /// Every live, present, non-dormant enemy, as (guid, position). Job
    /// abilities and the held power-up select their targets from here.
    pub(super) fn living_enemies(&self) -> Vec<(u64, [f32; 3])> {
        let states: Vec<&EnemyState> = match &self.state.active {
            Some(a) => a.enemies.iter().collect(),
            None => self
                .state
                .world_enemies
                .iter()
                .chain(self.field.iter().filter(|f| f.present).map(|f| &f.state))
                .collect(),
        };
        states
            .into_iter()
            .filter(|s| self.enemy(s.guid).is_some() && s.health > 0 && !s.dormant)
            .map(|s| (s.guid, s.position))
            .collect()
    }

    /// The nearest `max_targets` living enemies within `radius` of `center`,
    /// `primary` first (ties by distance, then guid). Shared by a job
    /// ability's area and the held power-up's caster-centred area.
    pub(super) fn area_targets(
        &self,
        center: [f32; 3],
        radius: f32,
        max_targets: usize,
        primary: Option<u64>,
    ) -> Vec<u64> {
        let mut targets: Vec<(u64, [f32; 3])> = self
            .living_enemies()
            .into_iter()
            .filter(|(_, at)| distance(center, *at) <= radius)
            .collect();
        targets.sort_by(|(a_guid, a_at), (b_guid, b_at)| {
            (Some(*a_guid) != primary, distance(center, *a_at), *a_guid)
                .partial_cmp(&(Some(*b_guid) != primary, distance(center, *b_at), *b_guid))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        targets.truncate(max_targets);
        targets.into_iter().map(|(guid, _)| guid).collect()
    }

    /// The authored burn: one refreshing entry per target/definition. A
    /// shielded summoner, like a dead target, never takes it.
    pub(super) fn apply_dot(
        &mut self,
        targets: &[u64],
        definition_id: i32,
        damage: u32,
        duration_ms: u64,
        interval_ms: u64,
        now_ms: u64,
    ) {
        for &guid in targets {
            if self.enemy(guid).is_some_and(|(_, s)| s.health > 0) && !self.shielded(guid) {
                self.skills.dots.insert(
                    (guid, definition_id),
                    SkillDot {
                        damage,
                        next_ms: now_ms.saturating_add(interval_ms),
                        until_ms: now_ms.saturating_add(duration_ms),
                        interval_ms,
                    },
                );
            }
        }
    }

    /// Freeze each living, unshielded target until `now_ms + duration_ms` and
    /// send the client's `com_stunned` clip (1401).
    pub(super) fn apply_stun(
        &mut self,
        targets: &[u64],
        duration_ms: u64,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        for &guid in targets {
            if self.enemy(guid).is_some_and(|(_, s)| s.health > 0) && !self.shielded(guid) {
                let until = now_ms.saturating_add(duration_ms);
                self.skills
                    .stuns
                    .entry(guid)
                    .and_modify(|current| *current = (*current).max(until))
                    .or_insert(until);
                self.moving.remove(&guid);
                out.replies.push(animation(guid, 1, true));
                out.replies.push(animation(guid, 1401, false)); // client com_stunned
            }
        }
    }

    /// A caster-private buff (the player's own damage/mitigation fold).
    pub(super) fn apply_caster_buff(&mut self, definition_id: i32, buff: SkillBuff) {
        self.skills.caster_buffs.insert(definition_id, buff);
    }

    pub(super) fn tick_job_effects(
        &mut self,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> io::Result<()> {
        self.skills.expire(now_ms);
        // At most 60 ticks per definition/target (30 s / 500 ms); delayed
        // calls catch up only the authored window, never extend its lifetime.
        let pending = std::mem::take(&mut self.skills.dots);
        for (key, mut dot) in pending {
            let (guid, _) = key;
            while dot.next_ms <= now_ms && dot.next_ms <= dot.until_ms {
                let Some(at) = self
                    .enemy(guid)
                    .filter(|(_, s)| s.health > 0)
                    .map(|(_, s)| s.position)
                else {
                    break;
                };
                merge_skill_outcome(
                    out,
                    self.strike_damage(guid, at, f32::MAX, dot.damage, now_ms, false)?,
                );
                let Some(next) = dot.next_ms.checked_add(dot.interval_ms) else {
                    dot.until_ms = 0; // no repeat forever at a saturated deadline
                    break;
                };
                dot.next_ms = next;
                if self
                    .state
                    .active
                    .as_ref()
                    .is_some_and(|a| a.phase != EncounterPhase::Fighting)
                {
                    break;
                }
            }
            if dot.next_ms <= dot.until_ms
                && self.enemy(guid).is_some_and(|(_, s)| s.health > 0)
                && self
                    .state
                    .active
                    .as_ref()
                    .is_none_or(|a| a.phase == EncounterPhase::Fighting)
            {
                self.skills.dots.insert(key, dot);
            }
        }
        Ok(())
    }

    pub(super) fn regenerate_energy(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        if self.energy < ENERGY_MAX && now_ms >= self.energy_regen_at {
            self.energy = (self.energy + ENERGY_STEP).min(ENERGY_MAX);
            self.energy_regen_at = now_ms.saturating_add(ENERGY_STEP_MS);
            out.replies
                .push(crate::abilities::build_energy(self.energy, ENERGY_MAX));
        }
    }

    /// The encounter's summoner while any of its summoned minions stands
    /// (the owner: "you kill those, then keep attacking the summoner") (R).
    pub(super) fn shielded(&self, guid: u64) -> bool {
        let (Some(d), Some(active)) = (self.dungeon(), self.state.active.as_ref()) else {
            return false;
        };
        d.enemies.iter().any(|a| a.guid == guid && a.skittish)
            && d.enemies
                .iter()
                .zip(&active.enemies)
                .any(|(a, s)| a.summon_below_percent.is_some() && s.health > 0 && !s.dormant)
    }

    /// One critical-hit roll for the player at the active job's chance.
    pub(crate) fn roll_crit(&mut self) -> bool {
        self.crit_seed = self
            .crit_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let bonus = self.skills.crit_bonus();
        ((self.crit_seed >> 33) % 100) < u64::from((self.crit_percent + bonus).min(100))
    }
}
