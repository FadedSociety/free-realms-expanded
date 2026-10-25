//! T658: chance item drops on a field-battle win (`drops` rows in
//! `field_battles.json`).
//!
//! The prize wheel grants exactly one previewed prize. A client item that the
//! evidence says an enemy carries (not a wheel prize) rides here instead: each
//! row is rolled once per win and a hit lands as `EncounterReward::bonus_items`.
//!
//! (R) Chance per win, quantity and the "granted only if it fits" rule are
//! reconstructed: no source recovers a drop rate. The rate is data, so it can be
//! retuned (or set to 0 to retire a drop) without code.

use super::*;

/// Rows one battle may carry (the prize wheel is capped at four).
const MAX_DROPS: usize = 4;

/// One chance item on a battle win.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BattleDrop {
    item_id: i32,
    #[serde(default = "one")]
    quantity: u32,
    /// Probability per win, `0.0..=1.0`.
    chance: f64,
    source: Vec<crate::content::SourceTag>,
}

/// Load-time bounds for a battle's `drops`: bounded rows, catalogue-shaped
/// quantities, a finite chance, and provenance on every row.
pub(super) fn validate(drops: &[BattleDrop]) -> io::Result<()> {
    if drops.len() > MAX_DROPS
        || drops.iter().any(|d| {
            d.item_id <= 0
                || !(1..=MAX_PRIZE_QUANTITY).contains(&d.quantity)
                || !(0.0..=1.0).contains(&d.chance)
        })
    {
        return Err(invalid("invalid field battle drops"));
    }
    drops
        .iter()
        .try_for_each(|d| field_rewards::validate_source(&d.source))
}

/// The `(item, quantity)` rows whose independent roll hit. Pure: the same rows
/// and generator state give the same drops.
fn roll(drops: &[BattleDrop], rng: &mut Rng) -> Vec<(i32, u32)> {
    drops
        .iter()
        .filter(|d| rng.next_f64() < d.chance)
        .map(|d| (d.item_id, d.quantity))
        .collect()
}

impl EncounterEngine {
    /// Roll the won battle's drops. No battle row, no drops, or no OS
    /// randomness means no drop: a win is never refused over a bonus.
    pub(super) fn roll_battle_drops(&mut self, battle: u32) -> Vec<(i32, u32)> {
        let Some(row) = self.battles.iter().find(|b| b.id == battle) else {
            return Vec::new();
        };
        if row.drops.is_empty() {
            return Vec::new();
        }
        if self.power_up_rng.is_none() {
            self.power_up_rng = Rng::from_entropy().ok();
        }
        let Some(rng) = self.power_up_rng.as_mut() else {
            return Vec::new();
        };
        roll(&row.drops, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop_row(item_id: i32, quantity: u32, chance: f64) -> BattleDrop {
        BattleDrop {
            item_id,
            quantity,
            chance,
            source: vec![crate::content::SourceTag::Reconstructed(
                "(R) test row".into(),
            )],
        }
    }

    /// The first draw of a seeded generator, read through the same public API.
    fn first_draw(seed: u64) -> f64 {
        Rng::seeded(seed).next_f64()
    }

    #[test]
    fn the_chance_decides_against_the_generators_draw() {
        let draw = first_draw(7);
        assert!(draw > 0.0 && draw < 1.0);
        let above = [drop_row(893, 2, (draw + 1.0) / 2.0)];
        let below = [drop_row(893, 2, draw / 2.0)];
        assert_eq!(roll(&above, &mut Rng::seeded(7)), vec![(893, 2)]);
        assert!(roll(&below, &mut Rng::seeded(7)).is_empty());
        // The extremes never need the generator.
        assert!(roll(&[drop_row(893, 1, 0.0)], &mut Rng::seeded(7)).is_empty());
        assert_eq!(
            roll(&[drop_row(893, 1, 1.0)], &mut Rng::seeded(7)),
            vec![(893, 1)]
        );
    }

    #[test]
    fn bad_rows_are_refused_at_load_time() {
        assert!(validate(&[drop_row(893, 1, 0.25)]).is_ok());
        assert!(validate(&[]).is_ok());
        for bad in [
            drop_row(0, 1, 0.25),
            drop_row(893, 0, 0.25),
            drop_row(893, MAX_PRIZE_QUANTITY + 1, 0.25),
            drop_row(893, 1, 1.5),
            drop_row(893, 1, -0.1),
            drop_row(893, 1, f64::NAN),
        ] {
            assert!(validate(&[bad]).is_err());
        }
        assert!(validate(&vec![drop_row(893, 1, 0.5); MAX_DROPS + 1]).is_err());
        let mut no_source = drop_row(893, 1, 0.5);
        no_source.source.clear();
        assert!(validate(&[no_source]).is_err(), "provenance is required");
        let mut loose = drop_row(893, 1, 0.5);
        loose.source = vec![crate::content::SourceTag::Reconstructed("guess".into())];
        assert!(validate(&[loose]).is_err(), "a reconstructed row says (R)");
    }
}
