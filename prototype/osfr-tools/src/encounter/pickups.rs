//! Power-ups on the ground. A kill's drop table and (rc-0136) a paced fight's heal drop both
//! place their pickups through [`EncounterEngine::place_pickup`]; moved out of `encounter.rs`.
//!
//! The client decides none of this: it holds no combat power-up drop chances (its one pickup
//! chance table, `PickupSpawnChanceLoad` `01066cf0`, belongs to the pirate ship battles'
//! `ShipZoneConfigLoad` `010680f0`). It draws the pickup's model (736
//! `powerup_health_buff.adr` for a heart) and plays the pickup effect the server sends.
use super::*;

/// (R) rc-0137 (owner: "no fight dropped a single power-up or heal"): the drop table of an
/// enemy that names none, by its rank, as field enemies drop by theirs (`power_ups.json`: `boss`
/// a guaranteed big heart, `elite` more often, `standard` the rest). An object and a minion (a
/// summoned helper, which must not be farmed) drop nothing.
pub(in crate::encounter) fn rank_table(rank: Rank) -> Option<String> {
    let table = match rank {
        Rank::Object | Rank::Minion => return None,
        Rank::Normal => "standard",
        Rank::Tough | Rank::Elite => "elite",
        Rank::Boss | Rank::DungeonBoss | Rank::NamedBoss | Rank::WorldBoss => "boss",
    };
    Some(table.to_owned())
}

impl EncounterEngine {
    /// Roll `table` for one participant of the kill and spawn each drop as a
    /// pickup actor. An unknown table, no loaded file or a full ground drops
    /// nothing. With several participants every chance is scaled by the file's
    /// group multiplier.
    pub(super) fn drop_power_ups(
        &mut self,
        table: &str,
        kill: (u64, u64),
        at: [f32; 3],
        now_ms: u64,
        participants: usize,
        out: &mut EncounterOutcome,
    ) {
        let Some(power_ups) = self.power_ups.clone() else {
            return;
        };
        let (Some(table), Some(rng)) = (power_ups.table(table), self.power_up_rng.as_mut()) else {
            return;
        };
        let scale = if participants > 1 {
            power_ups.group_chance_multiplier()
        } else {
            1.0
        };
        let kinds = power_ups::roll_scaled(table, rng, scale);
        for (i, kind_id) in kinds.into_iter().enumerate() {
            if self.pickups.len() >= power_ups.pickup.max_alive_per_instance as usize {
                break;
            }
            // A paced fight's hearts all come out of its heal budget (`pacing.rs`).
            let heal = (power_ups.kind(kind_id)).and_then(|k| k.heal_pct_max).unwrap_or(0);
            if !self.paced_heart_fits(heal) {
                continue;
            }
            if self.place_pickup(&power_ups, kind_id, i, kill, at, now_ms, out) {
                self.charge_paced_heart(heal);
            }
        }
    }

    /// Spawn one pickup of `kind_id` beside `at` (the `i`th of its kill, side by side). False
    /// for an unknown kind or a full ground.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn place_pickup(
        &mut self,
        power_ups: &PowerUps,
        kind_id: &str,
        i: usize,
        kill: (u64, u64),
        at: [f32; 3],
        now_ms: u64,
        out: &mut EncounterOutcome,
    ) -> bool {
        let Some(kind) = power_ups.kind(kind_id) else {
            return false;
        };
        if self.pickups.len() >= power_ups.pickup.max_alive_per_instance as usize {
            return false;
        }
        self.next_pickup += 1;
        let guid = PICKUP_GUID_BASE | self.next_pickup;
        // (R) Several drops from one kill sit side by side, not stacked.
        let position = self
            .ground()
            .snapped([at[0] + i as f32 * 0.8, at[1], at[2]]);
        let actor = Actor {
            guid,
            definition_id: 0,
            model_id: kind.model_id as i32,
            name_id: 0,
            name: String::new(),
            texture_alias: String::new(),
            position,
            max_health: 1,
            damage: 0,
            level: 1,
            boss: false,
            stages: Vec::new(),
            summon_below_percent: None,
            skittish: false,
            scale: kind.scale.map(|s| s as f32),
            behaviour: None,
            power_up_table: None,
            ..Default::default()
        };
        // Disposition 1 (neutral): 0 is hostile and would be Tab-targeted.
        out.replies.push(self.spawn(&actor, position, 1));
        self.pickups.push(PowerUpDrop {
            guid,
            kind: kind_id.to_owned(),
            kill,
            position,
            expires_ms: now_ms.saturating_add(u64::from(power_ups.pickup.ttl_ms)),
            told: false,
        });
        true
    }

    /// Where the health pickups lying in this run stand (the simulator steps onto them).
    pub(super) fn health_pickups(&self) -> Vec<[f32; 3]> {
        let Some(power_ups) = self.power_ups.as_ref() else {
            return Vec::new();
        };
        (self.pickups.iter())
            .filter(|d| (power_ups.kind(&d.kind)).is_some_and(|k| k.heal_pct_max.is_some()))
            .map(|d| d.position)
            .collect()
    }
}
