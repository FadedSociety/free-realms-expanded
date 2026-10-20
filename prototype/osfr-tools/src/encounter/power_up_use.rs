//! T222: the held combat power-up on its own action-bar key.
//!
//! The 2014 client has no separate combat power-up input. `ActionBarSlot3`
//! (default `3`; binding table 0x1b4198c row 21, keysym 0x33) is dispatched by
//! the regular-controller input dispatcher `00b02880` case 21 into
//! `DispatchActionBarSlot` `008a7eb0` with slot 2, and the AbilityProcessor
//! sends 36/10 bar 1 slot 2 (writer `00a342e0`). The `UsePowerup` action (row
//! 15, default Control_L) belongs to the demolition-derby/minigame controllers
//! and is not handled by `00b02880`. The locale dev note "power ups - add super
//! shield to bar" makes the slot server-owned, so `power_ups::SLOT` is the
//! held kind's own key (see `notes/Research/Power-up Use Key.md`).
//!
//! (R) Every magnitude, duration and radius comes from `power_ups.json`. The
//! kind is consumed before any effect runs, so a refused or replayed request
//! cannot duplicate it; there is no energy cost, cooldown or client-supplied
//! target. Super Shield's `speed_pct` has no server-side movement model and is
//! display-only for now.

use super::skills::{SkillBuff, SkillDot, merge_skill_outcome};
use super::*;

impl EncounterEngine {
    /// The held kind as the client's key-slot record, if it can be shown.
    pub fn held_power_up_slot(&self) -> Option<crate::power_ups::Slot> {
        self.power_ups
            .as_ref()?
            .slot(self.held_power_up.as_deref()?)
    }

    /// Use the held power-up at the server's position. `None` means nothing is
    /// held, so the caller keeps its normal handling of the request. A held
    /// kind is consumed before the effect runs; a knocked-out player or a
    /// finished battle gets the 36/1 refusal and keeps the kind.
    pub fn use_power_up(
        &mut self,
        position: [f32; 3],
        profile: i32,
        now_ms: u64,
    ) -> io::Result<Option<EncounterOutcome>> {
        let Some(kind_id) = self.held_power_up.clone() else {
            return Ok(None);
        };
        if !valid_position(position) {
            return Err(invalid("invalid power-up position"));
        }
        let Some(power_ups) = self.power_ups.clone() else {
            // A reload removed the tables: drop the stale hold.
            self.held_power_up = None;
            return Ok(Some(EncounterOutcome::default()));
        };
        let Some(kind) = power_ups.kind(&kind_id) else {
            self.held_power_up = None;
            return Ok(Some(EncounterOutcome::default()));
        };
        let Some(slot) = power_ups.slot(&kind_id) else {
            self.held_power_up = None;
            return Ok(Some(EncounterOutcome::default()));
        };
        if self.state.player_health == 0
            || self
                .state
                .active
                .as_ref()
                .is_some_and(|a| a.phase != EncounterPhase::Fighting)
        {
            return Ok(Some(EncounterOutcome {
                replies: vec![crate::abilities::unsupported_cast_failure()],
                ..Default::default()
            }));
        }
        // Consume first: nothing below can run a second time for this hold.
        self.held_power_up = None;
        self.skills.expire(now_ms);
        // (R) The overworld has no battle tier; the player's job-level weapon
        // tier picks the authored damage curve (the same x1.75 steps).
        let tier = (weapon_tier(self.job_level).clamp(1, 5) - 1) as usize;
        let (_, base) = profile_stats(profile, 0);
        let base = (base as f32 * self.damage_scale).max(0.0).round() as u32;
        let mut out = EncounterOutcome::default();
        if let Some(area) = &kind.area {
            let states: Vec<(u64, [f32; 3])> = match &self.state.active {
                Some(active) => active
                    .enemies
                    .iter()
                    .map(|s| (s.guid, s.position))
                    .collect(),
                None => self
                    .state
                    .world_enemies
                    .iter()
                    .chain(self.field.iter().filter(|f| f.present).map(|f| &f.state))
                    .map(|s| (s.guid, s.position))
                    .collect(),
            };
            let mut targets: Vec<(u64, [f32; 3])> = states
                .into_iter()
                .filter(|(guid, at)| {
                    self.enemy(*guid)
                        .is_some_and(|(_, s)| s.health > 0 && !s.dormant)
                        && distance(position, *at) <= area.radius as f32
                })
                .collect();
            targets.sort_by(|a, b| {
                distance(position, a.1)
                    .total_cmp(&distance(position, b.1))
                    .then_with(|| a.0.cmp(&b.0))
            });
            targets.truncate(area.max_targets as usize);
            if let Some(multiplier) = kind.damage_by_tier.as_ref().and_then(|t| t.get(tier)) {
                let damage = (base as f32 * *multiplier as f32).round() as u32;
                for (guid, at) in &targets {
                    merge_skill_outcome(
                        &mut out,
                        self.strike_damage(*guid, *at, f32::MAX, damage, now_ms, false)?,
                    );
                }
            }
            if let Some(burn) = &kind.burn {
                let damage = (base as f32 * burn.multiplier as f32).round() as u32;
                for (guid, _) in &targets {
                    if self.enemy(*guid).is_some_and(|(_, s)| s.health > 0) {
                        self.skills.dots.insert(
                            (*guid, slot.definition_id),
                            SkillDot {
                                damage,
                                next_ms: now_ms.saturating_add(u64::from(burn.interval_ms)),
                                until_ms: now_ms.saturating_add(u64::from(burn.duration_ms)),
                                interval_ms: u64::from(burn.interval_ms),
                            },
                        );
                    }
                }
            }
            if let Some(stun_ms) = kind.stun_ms {
                for (guid, _) in &targets {
                    if self.enemy(*guid).is_some_and(|(_, s)| s.health > 0) {
                        let until = now_ms.saturating_add(u64::from(stun_ms));
                        self.skills
                            .stuns
                            .entry(*guid)
                            .and_modify(|current| *current = (*current).max(until))
                            .or_insert(until);
                        self.moving.remove(guid);
                        out.replies.push(animation(*guid, 1, true));
                        out.replies.push(animation(*guid, 1401, false)); // com_stunned
                    }
                }
            }
        }
        // Super Shield: a 0.0 damage-taken buff is the server's invulnerability.
        // The client gets one effect tag for the HUD; `speed_pct` is
        // display-only because the server does not drive client motion (R).
        if kind.invulnerable || kind.speed_pct.is_some() {
            let duration = kind.duration_ms.unwrap_or(0);
            if duration > 0 {
                self.skills.caster_buffs.insert(
                    slot.definition_id,
                    SkillBuff {
                        until_ms: now_ms.saturating_add(u64::from(duration)),
                        damage: 1.0,
                        taken: if kind.invulnerable { 0.0 } else { 1.0 },
                        crit: 0,
                        crit_damage: 0.0,
                    },
                );
                out.replies.push(crate::abilities::build_effect_tag(
                    slot.definition_id,
                    u64::from(duration),
                    slot.icon_id,
                    slot.name_id,
                ));
            }
        }
        Ok(Some(out))
    }
}
