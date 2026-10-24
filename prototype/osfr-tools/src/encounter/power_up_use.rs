//! T222: the held combat power-up on its own action-bar key.
//!
//! The 2014 client has no separate combat power-up input. `ActionBarSlot3`
//! (default `3`; binding table 0x1b4198c row 21, keysym 0x33) is dispatched by
//! the regular-controller input dispatcher `00b02880` case 21 into
//! `DispatchActionBarSlot` `008a7eb0` with slot 2; the alternate-bar flag it
//! passes selects mode 1 (job bar) normally and mode 2 (item bar) while the
//! alternate bar is held. The job path is `009cf430` and the AbilityProcessor
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

use super::skills::{SkillBuff, merge_skill_outcome};
use super::*;

impl EncounterEngine {
    /// The held kind as the client's key-slot record, if it can be shown.
    pub fn held_power_up_slot(&self) -> Option<crate::power_ups::Slot> {
        self.power_ups
            .as_ref()?
            .slot(self.held_power_up.as_deref()?)
    }

    /// The active fight's battle tier, 1..5: a field battle's authored
    /// `difficulty`. The overworld and the opening cave carry no battle tier,
    /// so they count as tier 1.
    fn battle_tier(&self) -> usize {
        self.dungeon()
            .and_then(|d| self.battles.iter().find(|b| b.id == d.id))
            .map_or(1, |b| b.difficulty.clamp(1, 5) as usize)
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
        // (R) The battle tier indexes the authored curve. The current basic
        // already carries the job level's weapon scale, so the curve is
        // applied exactly once (a tier-1 Fight uses the plain basic).
        let tier = self.battle_tier() - 1;
        let (_, base) = self.scale.profile_stats(profile, 0);
        let base = (base as f32 * self.damage_scale).round() as u32;
        let mut out = EncounterOutcome::default();
        if let Some(area) = &kind.area {
            let targets = self.area_targets(
                position,
                area.radius as f32,
                area.max_targets as usize,
                None,
            );
            if let Some(multiplier) = kind.damage_by_tier.as_ref().and_then(|t| t.get(tier)) {
                let damage = (base as f32 * *multiplier as f32).round() as u32;
                for &guid in &targets {
                    if let Some(at) = self.enemy(guid).map(|(_, s)| s.position) {
                        merge_skill_outcome(
                            &mut out,
                            self.strike_damage(guid, at, f32::MAX, damage, now_ms, false)?,
                        );
                    }
                }
            }
            if let Some(burn) = &kind.burn {
                self.apply_dot(
                    &targets,
                    slot.definition_id,
                    (base as f32 * burn.multiplier as f32).round() as u32,
                    u64::from(burn.duration_ms),
                    u64::from(burn.interval_ms),
                    now_ms,
                );
            }
            if let Some(stun_ms) = kind.stun_ms {
                self.apply_stun(&targets, u64::from(stun_ms), now_ms, &mut out);
            }
        }
        // Super Shield: a 0.0 damage-taken buff is the server's invulnerability.
        // The client gets one effect tag for the HUD; `speed_pct` is
        // display-only because the server does not drive client motion (R).
        if kind.invulnerable || kind.speed_pct.is_some() {
            let duration = kind.duration_ms.unwrap_or(0);
            if duration > 0 {
                self.apply_caster_buff(
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
