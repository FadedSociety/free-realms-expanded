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

pub(super) fn merge_skill_outcome(out: &mut EncounterOutcome, other: EncounterOutcome) {
    out.changed |= other.changed;
    out.replies.extend(other.replies);
    out.rewards.extend(other.rewards);
    out.defeated.extend(other.defeated);
    out.completed = other.completed.or(out.completed);
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
        self.expire_skill_buffs(now_ms);
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
            let states: Vec<&EnemyState> = match &self.state.active {
                Some(a) => a.enemies.iter().collect(),
                None => self
                    .state
                    .world_enemies
                    .iter()
                    .chain(self.field.iter().filter(|f| f.present).map(|f| &f.state))
                    .collect(),
            };
            let mut nearby: Vec<_> = states
                .into_iter()
                .filter(|s| self.enemy(s.guid).is_some() && s.health > 0 && !s.dormant)
                .collect();
            let caster_area = area.is_some_and(|(_, _, caster)| caster);
            let select_range = if caster_area { area.unwrap().0 } else { range };
            // (R) A non-hostile/self target selects the nearest enemy; an explicit
            // hostile target out of reach fails instead of silently hitting another.
            let selected = nearby
                .iter()
                .find(|s| s.guid == target)
                .copied()
                .or_else(|| {
                    nearby
                        .iter()
                        .copied()
                        .filter(|s| distance(position, s.position) <= select_range)
                        .min_by(|a, b| {
                            (self.shielded(a.guid), distance(position, a.position))
                                .partial_cmp(&(
                                    self.shielded(b.guid),
                                    distance(position, b.position),
                                ))
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                });
            let Some(selected) = selected else {
                return Ok(refuse(288));
            };
            if distance(position, selected.position) > select_range {
                return Ok(refuse(288));
            }
            if self
                .last_attack
                .is_some_and(|last| now_ms.saturating_sub(last) < 600)
            {
                return Ok(refuse(287));
            }
            let center = if caster_area {
                position
            } else {
                selected.position
            };
            let (radius, limit, _) = area.unwrap_or((0.0, 1, false));
            nearby.retain(|s| {
                if area.is_some() {
                    distance(center, s.position) <= radius
                } else {
                    s.guid == selected.guid
                }
            });
            nearby.sort_by(|a, b| {
                (
                    a.guid != selected.guid,
                    distance(center, a.position),
                    a.guid,
                )
                    .partial_cmp(&(
                        b.guid != selected.guid,
                        distance(center, b.position),
                        b.guid,
                    ))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            targets.extend(nearby.into_iter().take(limit).map(|s| s.guid));
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
                } => {
                    for &guid in &targets {
                        if self.enemy(guid).is_some_and(|(_, s)| s.health > 0)
                            && !self.shielded(guid)
                        {
                            self.skill_dots.insert(
                                (guid, ability.definition_id),
                                SkillDot {
                                    damage: (base as f32 * multiplier).round() as u32,
                                    next_ms: now_ms.saturating_add(interval_ms),
                                    until_ms: now_ms.saturating_add(duration_ms),
                                    interval_ms,
                                },
                            );
                        }
                    }
                }
                JobEffect::Stun { duration_ms } => {
                    for &guid in &targets {
                        if self.enemy(guid).is_some_and(|(_, s)| s.health > 0)
                            && !self.shielded(guid)
                        {
                            self.skill_stuns
                                .entry(guid)
                                .and_modify(|until| {
                                    *until = (*until).max(now_ms.saturating_add(duration_ms))
                                })
                                .or_insert(now_ms.saturating_add(duration_ms));
                            self.moving.remove(&guid);
                            out.replies.push(animation(guid, 1, true));
                            out.replies.push(animation(guid, 1401, false)); // client com_stunned
                        }
                    }
                }
                JobEffect::Buff {
                    recipient,
                    duration_ms,
                    damage_multiplier,
                    damage_taken_multiplier,
                    crit_bonus,
                    crit_damage_bonus,
                } => {
                    let recipients = if recipient == EffectRecipient::Caster {
                        vec![0]
                    } else {
                        targets.clone()
                    };
                    for guid in recipients {
                        if guid != 0
                            && (!self.enemy(guid).is_some_and(|(_, s)| s.health > 0)
                                || self.shielded(guid))
                        {
                            continue;
                        }
                        self.skill_buffs.insert(
                            (guid, ability.definition_id),
                            SkillBuff {
                                until_ms: now_ms.saturating_add(duration_ms),
                                damage: damage_multiplier,
                                taken: damage_taken_multiplier,
                                crit: crit_bonus,
                                crit_damage: crit_damage_bonus,
                            },
                        );
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

    /// Energy comes back over time; reported when it changes.
    pub(super) fn expire_skill_buffs(&mut self, now_ms: u64) {
        self.skill_buffs.retain(|_, b| now_ms < b.until_ms);
        self.skill_stuns.retain(|_, until| now_ms < *until);
    }

    /// (R) Strongest modifier wins; repeated casts refresh instead of stacking.
    pub(super) fn skill_damage(&self, target: u64, damage: u32) -> u32 {
        let outgoing = self
            .skill_buffs
            .iter()
            .filter(|((guid, _), _)| *guid == 0)
            .map(|(_, b)| b.damage)
            .fold(1.0_f32, f32::max);
        let taken = self
            .skill_buffs
            .iter()
            .filter(|((guid, _), _)| *guid == target)
            .map(|(_, b)| b.taken)
            .fold(1.0_f32, f32::max);
        (damage as f32 * outgoing * taken).round() as u32
    }

    pub(super) fn skill_mitigation(&self) -> f32 {
        self.skill_buffs
            .iter()
            .filter(|((guid, _), _)| *guid == 0)
            .map(|(_, b)| b.taken)
            .fold(1.0_f32, f32::min)
    }

    pub(super) fn tick_job_effects(
        &mut self,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> io::Result<()> {
        self.expire_skill_buffs(now_ms);
        // At most 60 ticks per definition/target (30 s / 500 ms); delayed
        // calls catch up only the authored window, never extend its lifetime.
        let pending = std::mem::take(&mut self.skill_dots);
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
                self.skill_dots.insert(key, dot);
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
        let bonus = self
            .skill_buffs
            .iter()
            .filter(|((guid, _), _)| *guid == 0)
            .map(|(_, b)| b.crit)
            .max()
            .unwrap_or(0);
        ((self.crit_seed >> 33) % 100) < u64::from((self.crit_percent + bonus).min(100))
    }
}
