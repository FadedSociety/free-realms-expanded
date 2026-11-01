//! L088 (Sanctuary port P4c): battle items, the orbs and spheres thrown from the item bar. Adapted
//! from Sanctuary's item-ability attack (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary,
//! `src/Sanctuary.Gateway/Helpers/Abilities/ItemAbility.cs` `FindMobs`/`AttackMobs`,
//! AGPL-3.0-or-later); the numbers are the wiki's, in `consumables.json`. An orb lands on the clicked
//! enemy, or on the nearest one in reach when none is clicked; a clicked enemy out of reach is no
//! target, like a job ability's. A sphere's blast also takes every enemy within its radius of that one.
use super::skills::merge_skill_outcome;
use super::*;

/// (R) How far a battle item is thrown: the Wizard's reach.
pub const BATTLE_ITEM_REACH: f32 = 20.0;
/// (R) The most enemies one blast takes, as the Flame Wave power-up's.
const MAX_BLAST_TARGETS: usize = 16;

impl EncounterEngine {
    /// The enemy a battle item thrown from `position` lands on, if any.
    pub fn battle_item_target(&self, position: [f32; 3], aimed: Option<u64>) -> Option<u64> {
        let in_reach = self.area_targets(position, BATTLE_ITEM_REACH, usize::MAX, aimed);
        match aimed {
            Some(guid) if self.enemy(guid).is_some() => in_reach.contains(&guid).then_some(guid),
            _ => in_reach.first().copied(),
        }
    }

    /// Throw a battle item: `damage` (its tier-1 value, scaled by the job level's tier like every
    /// hit) and a `stun_ms` stun on the enemy it lands on and, with a `radius`, on every enemy
    /// within it of that one.
    pub fn use_battle_item(
        &mut self,
        damage: u32,
        stun_ms: u64,
        radius: f32,
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
        let targets = if radius > 0.0 {
            self.area_targets(at, radius, MAX_BLAST_TARGETS, Some(primary))
        } else {
            vec![primary]
        };
        let damage = (damage as f32 * self.damage_scale).round() as u32;
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
        if stun_ms > 0 {
            self.apply_stun(&targets, stun_ms, now_ms, &mut out);
        }
        Ok(out)
    }
}
