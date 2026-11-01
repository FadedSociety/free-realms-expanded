//! L085 (Sanctuary port P4a): a potion's heal over time. Sanctuary's item abilities carry a restore spread
//! over a duration (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary @1ff876a,
//! `src/Sanctuary.Game/Resources/Definitions/ItemAbilityDefinition.cs` `Health`, AGPL-3.0-or-later); the
//! amounts are ours. Our client's Small Health Potion text reads "restores some of your health over 5
//! seconds", and the wiki tooltip of another era reads "Instantly heals you for 2% health" plus "11%
//! health/second for 3 seconds" (35% in all). The engine heals the rest once a second; a new potion
//! replaces a running heal, and a knock-out ends it.
use super::*;

/// What is left of a heal, in hundredths of a percent of maximum health.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HealOverTime {
    left: u32,
    per_second: u32,
    next_ms: u64,
}

impl EncounterEngine {
    /// Heal `percent` of maximum health over `over_ms`, one step a second (at least one step).
    pub fn heal_player_over_time(&mut self, percent: u32, over_ms: u64, now_ms: u64) {
        let seconds = u32::try_from((over_ms / 1000).max(1)).unwrap_or(u32::MAX);
        let total = percent.min(100) * 100;
        self.heal_over_time = (total > 0).then_some(HealOverTime {
            left: total,
            per_second: total.div_ceil(seconds),
            next_ms: now_ms.saturating_add(1000),
        });
    }

    /// The steps due by `now_ms`, sent as one health update.
    pub(super) fn tick_heal_over_time(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let Some(mut heal) = self.heal_over_time else {
            return;
        };
        if self.state.player_health == 0 {
            self.heal_over_time = None;
            return;
        }
        let before = self.state.player_health;
        while now_ms >= heal.next_ms && heal.left > 0 {
            let step = heal.per_second.min(heal.left);
            heal.left -= step;
            heal.next_ms = heal.next_ms.saturating_add(1000);
            let gain = u64::from(self.max_health) * u64::from(step) / 10_000;
            let health = u64::from(self.state.player_health) + gain;
            self.state.player_health = health.min(u64::from(self.max_health)) as u32;
        }
        if self.state.player_health != before {
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
        }
        self.heal_over_time = (heal.left > 0).then_some(heal);
    }
}
