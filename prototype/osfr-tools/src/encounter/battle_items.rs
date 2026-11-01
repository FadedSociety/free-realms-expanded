//! L088 (Sanctuary port P4c): battle items, the orbs and spheres thrown from the item bar. Adapted
//! from Sanctuary's item-ability attack (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary,
//! `src/Sanctuary.Gateway/Helpers/Abilities/ItemAbility.cs` `FindMobs`/`AttackMobs`,
//! AGPL-3.0-or-later); the numbers are the wiki's, in `consumables.json`. An orb lands on the clicked
//! enemy, or on the nearest one in reach when none is clicked; a clicked enemy out of reach is no
//! target, like a job ability's. A sphere's blast also takes every enemy within its radius of that one.
use super::skills::{Hold, merge_skill_outcome};
use super::*;

/// (R) How far a battle item is thrown: the Wizard's reach.
pub const BATTLE_ITEM_REACH: f32 = 20.0;
/// (R) The most enemies one blast takes, as the Flame Wave power-up's.
const MAX_BLAST_TARGETS: usize = 16;
/// L091 (R): how long a knocked-back enemy stays down before it walks back (the fork's knockback hold).
const KNOCKBACK_MS: u64 = 2_000;

impl EncounterEngine {
    /// The enemy a battle item thrown from `position` lands on, if any.
    pub fn battle_item_target(&self, position: [f32; 3], aimed: Option<u64>) -> Option<u64> {
        let in_reach = self.area_targets(position, BATTLE_ITEM_REACH, usize::MAX, aimed);
        match aimed {
            Some(guid) if self.enemy(guid).is_some() => in_reach.contains(&guid).then_some(guid),
            _ => in_reach.first().copied(),
        }
    }

    /// Throw a battle item: its damage (the tier-1 value, scaled by the job level's tier like every
    /// hit), stun, sleep and root land on the enemy it hits and, with a radius, on every enemy within
    /// it of that one.
    pub fn use_battle_item(
        &mut self,
        strike: crate::item_bar::Strike,
        aimed: Option<u64>,
        position: [f32; 3],
        now_ms: u64,
    ) -> io::Result<EncounterOutcome> {
        let mut out = EncounterOutcome::default();
        let Some(primary) = self.battle_item_target(position, aimed) else {
            return Ok(out);
        };
        let Some(at) = self.enemy(primary).map(|(_, s)| s.position) else {
            return Ok(out);
        };
        let targets = if strike.radius > 0.0 {
            self.area_targets(at, strike.radius, MAX_BLAST_TARGETS, Some(primary))
        } else {
            vec![primary]
        };
        let damage = (strike.damage as f32 * self.damage_scale).round() as u32;
        if damage > 0 {
            for &guid in &targets {
                if let Some(at) = self.enemy(guid).map(|(_, s)| s.position) {
                    merge_skill_outcome(
                        &mut out,
                        self.strike_damage(guid, at, f32::MAX, damage, now_ms, false)?,
                    );
                }
            }
        }
        if strike.stun_ms > 0 {
            self.apply_stun(&targets, strike.stun_ms, now_ms, &mut out);
        }
        if strike.sleep_ms > 0 {
            self.apply_hold(&targets, strike.sleep_ms, false, now_ms, &mut out);
        }
        if strike.root_ms > 0 {
            self.apply_hold(&targets, strike.root_ms, true, now_ms, &mut out);
        }
        if strike.knockback > 0.0 {
            self.apply_knockback(&targets, position, strike.knockback, now_ms, &mut out);
        }
        Ok(out)
    }

    /// L091: push each living target `distance` straight away from `from` (the thrower), onto the
    /// ground, facing the thrower, then hold it for `KNOCKBACK_MS` with the client's com_knock_down
    /// clip (1402). A push that would leave the terrain (a ledge, a hole) does not move it (R).
    fn apply_knockback(
        &mut self,
        targets: &[u64],
        from: [f32; 3],
        distance: f32,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        let ground = self.ground();
        let mut pushed = Vec::new();
        for &guid in targets {
            let Some(at) = self
                .enemy(guid)
                .filter(|(_, s)| s.health > 0)
                .map(|(_, s)| s.position)
            else {
                continue;
            };
            if self.shielded(guid) {
                continue;
            }
            let (dx, dz) = (at[0] - from[0], at[2] - from[2]);
            let length = dx.hypot(dz);
            let (ux, uz) = if length > 0.001 {
                (dx / length, dz / length)
            } else {
                (1.0, 0.0)
            };
            let Some(to) =
                ground.field_position([at[0] + ux * distance, at[1], at[2] + uz * distance])
            else {
                continue;
            };
            self.set_enemy_position(guid, to);
            out.replies.push(movement(guid, to, [-ux, 0.0, -uz]));
            pushed.push(guid);
        }
        self.apply_stun(&pushed, KNOCKBACK_MS, now_ms, out);
        for &guid in &pushed {
            out.replies.push(animation(guid, 1402, false)); // client com_knock_down
        }
    }

    /// Where a knocked-back enemy now stands: the field's copy, or the fight's or world's.
    fn set_enemy_position(&mut self, guid: u64, to: [f32; 3]) {
        if self.state.active.is_none()
            && let Some(f) = self
                .field_sim
                .enemies
                .iter_mut()
                .find(|f| f.present && f.actor.guid == guid)
        {
            f.state.position = to;
            return;
        }
        let states = if let Some(active) = &mut self.state.active {
            &mut active.enemies
        } else {
            &mut self.state.world_enemies
        };
        if let Some(s) = states.iter_mut().find(|s| s.guid == guid) {
            s.position = to;
        }
    }

    /// L089: put each living target to sleep (a stun its next hit ends) or root it (it cannot walk
    /// until a hit) for `duration_ms`. A sleeper shows the stun clip (R: no sleep clip is wired).
    fn apply_hold(
        &mut self,
        targets: &[u64],
        duration_ms: u64,
        root: bool,
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) {
        if !root {
            self.apply_stun(targets, duration_ms, now_ms, out);
        }
        let until = now_ms.saturating_add(duration_ms);
        for &guid in targets {
            if self.enemy(guid).is_some_and(|(_, s)| s.health > 0) && !self.shielded(guid) {
                self.skills.holds.insert(guid, Hold { until, root });
                if root && (self.arena.moving.remove(&guid) | self.field_sim.moving.remove(&guid)) {
                    out.replies.push(animation(guid, 1, true));
                }
            }
        }
    }
}
