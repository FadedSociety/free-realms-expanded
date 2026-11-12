//! L085 (Sanctuary port P4a): a potion's heal over time. Sanctuary's item abilities carry a restore spread
//! over a duration (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary @1ff876a,
//! `src/Sanctuary.Game/Resources/Definitions/ItemAbilityDefinition.cs` `Health`, AGPL-3.0-or-later); the
//! amounts are ours. Our client's Small Health Potion text reads "restores some of your health over 5
//! seconds", and the wiki tooltip of another era reads "Instantly heals you for 2% health" plus "11%
//! health/second for 3 seconds" (35% in all). The engine heals the rest once a second; a new potion
//! replaces a running heal, and a knock-out ends it.
//!
//! C3: the same for energy (the fork's `Energy` restore, its `EffectiveMaxEnergy`), on its own track,
//! so an energy potion never cuts a running heal short; and a food or potion's combat buff, held as
//! one caster buff of the skill fold (`skills.rs`) under its own key.
use super::*;

/// The caster-buff key of a consumable's combat buff: below every job ability definition id, so it
/// never replaces a skill's buff.
const CONSUMABLE_BUFF: i32 = -1;

/// What is left of a restore, in hundredths of a percent of the maximum: the health track (L085)
/// and the energy track (C3), each stepping once a second.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HealOverTime {
    health: Track,
    energy: Track,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Track {
    total: u32,
    left: u32,
    per_second: u32,
    next_ms: u64,
}

impl Track {
    /// `percent` over `over_ms`, one step a second (at least one step).
    fn new(percent: u32, over_ms: u64, now_ms: u64) -> Self {
        let seconds = u32::try_from((over_ms / 1000).max(1)).unwrap_or(u32::MAX);
        let total = percent.min(100) * 100;
        Self {
            total,
            left: total,
            per_second: total.div_ceil(seconds),
            next_ms: now_ms.saturating_add(1000),
        }
    }

    /// The next step due by `now_ms`, if any.
    fn step(&mut self, now_ms: u64) -> Option<u32> {
        if now_ms < self.next_ms || self.left == 0 {
            return None;
        }
        let step = self.per_second.min(self.left);
        self.left -= step;
        self.next_ms = self.next_ms.saturating_add(1000);
        Some(step)
    }
}

impl HealOverTime {
    fn running(self) -> Option<Self> {
        (self.health.left > 0 || self.energy.left > 0).then_some(self)
    }
}

impl EncounterEngine {
    /// Heal `percent` of maximum health over `over_ms`, one step a second (at least one step).
    pub fn heal_player_over_time(&mut self, percent: u32, over_ms: u64, now_ms: u64) {
        let mut restore = self.heal_over_time.unwrap_or_default();
        restore.health = Track::new(percent, over_ms, now_ms);
        self.heal_over_time = restore.running();
    }

    /// C3: restore `percent` of maximum energy over `over_ms`, as [`Self::heal_player_over_time`].
    pub fn restore_player_energy_over_time(&mut self, percent: u32, over_ms: u64, now_ms: u64) {
        let mut restore = self.heal_over_time.unwrap_or_default();
        restore.energy = Track::new(percent, over_ms, now_ms);
        self.heal_over_time = restore.running();
    }

    /// C3: `percent` of maximum energy at once, clamped to the maximum. A knocked-out player gets
    /// nothing. Returns the 38/13 energy update, or nothing when the energy did not change.
    pub fn restore_player_energy_percent(&mut self, percent: u32) -> Vec<Vec<u8>> {
        if self.state.player_health == 0 {
            return Vec::new();
        }
        self.add_energy(percent.min(100) * 100).into_iter().collect()
    }

    /// C3: the player's energy and its maximum.
    pub fn player_energy(&self) -> (i32, i32) {
        (self.energy, self.energy_max())
    }

    /// C3: a food or potion's combat buff until `until_ms`: `damage_percent` more damage dealt and
    /// `taken_percent` less taken. It replaces the last one; 0 and 0 ends it.
    pub fn set_consumable_buff(&mut self, damage_percent: u32, taken_percent: u32, until_ms: u64) {
        if damage_percent == 0 && taken_percent == 0 {
            self.skills.caster_buffs.remove(&CONSUMABLE_BUFF);
            return;
        }
        let buff = skills::SkillBuff {
            until_ms,
            damage: 1.0 + damage_percent.min(1_000) as f32 / 100.0,
            taken: 1.0 - taken_percent.min(100) as f32 / 100.0,
            crit: 0,
            crit_damage: 0.0,
        };
        self.apply_caster_buff(CONSUMABLE_BUFF, buff);
    }

    /// The maximum the energy regenerates to: the weapon kits' while they run the active job,
    /// else the rules' (as `regenerate_energy` picks it).
    fn energy_max(&self) -> i32 {
        match &self.skills.kit.table {
            Some(table) if self.kit_applies(self.player_profile) => table.energy.max,
            _ => self.rules.energy.max,
        }
    }

    /// Adds `hundredths` of a percent of maximum energy; the 38/13 when it changed.
    fn add_energy(&mut self, hundredths: u32) -> Option<Vec<u8>> {
        let gain = i64::from(self.energy_max()) * i64::from(hundredths) / 10_000;
        self.add_energy_points(gain)
    }

    /// Adds `gain` energy, clamped to the maximum; the 38/13 when it changed.
    fn add_energy_points(&mut self, gain: i64) -> Option<Vec<u8>> {
        let max = self.energy_max();
        let energy = (i64::from(self.energy) + gain).clamp(0, i64::from(max)) as i32;
        if energy == self.energy {
            return None;
        }
        self.energy = energy;
        Some(crate::abilities::build_energy(energy, max))
    }

    /// The steps due by `now_ms`, sent as one health update and one energy update.
    pub(super) fn tick_heal_over_time(&mut self, now_ms: u64, out: &mut EncounterOutcome) {
        let Some(mut restore) = self.heal_over_time else {
            return;
        };
        if self.state.player_health == 0 {
            self.heal_over_time = None;
            return;
        }
        let before = self.state.player_health;
        while let Some(step) = restore.health.step(now_ms) {
            let gain = u64::from(self.max_health) * u64::from(step) / 10_000;
            let health = u64::from(self.state.player_health) + gain;
            self.state.player_health = health.min(u64::from(self.max_health)) as u32;
        }
        if self.state.player_health != before {
            out.replies
                .push(player_hp(self.state.player_health, self.max_health));
        }
        // Energy is whole points of a 100-point bar: give what the done part rounds down to, so
        // the steps add up to the potion's whole amount.
        let given = |track: Track| track.total - track.left;
        let before = given(restore.energy);
        while restore.energy.step(now_ms).is_some() {}
        let max = i64::from(self.energy_max());
        let points = |hundredths: u32| max * i64::from(hundredths) / 10_000;
        let gain = points(given(restore.energy)) - points(before);
        if gain > 0 {
            out.replies.extend(self.add_energy_points(gain));
        }
        self.heal_over_time = restore.running();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C3: energy comes back at once (clamped, nothing sent when it does not change) or a step a
    /// second on its own track, so a heal over time keeps running beside it.
    #[test]
    fn c3_energy_restores_at_once_or_over_time_beside_a_heal() {
        let (_dir, mut engine) = super::super::shared::tests::engine();
        let (full, max) = engine.player_energy();
        assert_eq!(full, max);
        assert!(engine.restore_player_energy_percent(50).is_empty(), "full stays full");

        engine.energy = 0;
        assert_eq!(engine.restore_player_energy_percent(25).len(), 1, "one 38/13");
        assert_eq!(engine.player_energy().0, max / 4);
        engine.restore_player_energy_percent(100);
        assert_eq!(engine.player_energy().0, max, "clamped to the maximum");

        engine.energy = 0;
        engine.state.player_health = engine.max_health / 2;
        engine.heal_player_over_time(40, 4_000, 0);
        engine.restore_player_energy_over_time(40, 4_000, 0);
        let mut out = EncounterOutcome::default();
        engine.tick_heal_over_time(999, &mut out);
        assert!(out.replies.is_empty(), "nothing before the first second");
        engine.tick_heal_over_time(1_000, &mut out);
        assert_eq!(out.replies.len(), 2, "one health and one energy update");
        assert_eq!(engine.player_energy().0, max / 10);
        engine.tick_heal_over_time(4_000, &mut out);
        assert_eq!(engine.player_energy().0, max * 4 / 10);
        let health = engine.state.player_health;
        assert!(health > engine.max_health / 2, "{health}");
        assert!(engine.heal_over_time.is_none(), "both tracks are done");

        // A knock-out stops both.
        engine.restore_player_energy_over_time(40, 4_000, 5_000);
        engine.state.player_health = 0;
        engine.tick_heal_over_time(6_000, &mut out);
        assert!(engine.heal_over_time.is_none());
        assert!(engine.restore_player_energy_percent(50).is_empty());
    }

    /// C3: a consumable's combat buff is one caster buff under its own key, and 0/0 ends it.
    #[test]
    fn c3_a_combat_buff_folds_into_damage_and_mitigation_and_ends() {
        let (_dir, mut engine) = super::super::shared::tests::engine();
        engine.set_consumable_buff(0, 25, 30_000);
        assert!((engine.skills.mitigation() - 0.75).abs() < 1e-6);
        engine.set_consumable_buff(8, 0, 30_000);
        assert!((engine.skills.mitigation() - 1.0).abs() < 1e-6, "the attack buff replaced it");
        assert_eq!(engine.skills.damage(1, 100), 108);
        engine.set_consumable_buff(0, 0, 0);
        assert!(engine.skills.caster_buffs.is_empty());
    }
}
