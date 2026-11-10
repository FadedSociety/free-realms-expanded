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

/// One chance item on a battle win, or (L084) one roll of a loot table: an end-of-dungeon reward pool.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BattleDrop {
    #[serde(default)]
    item_id: i32,
    /// L084: a `loot_tables.json` key rolled instead of `item_id` (items only; see `validate`).
    #[serde(default)]
    table: Option<String>,
    #[serde(default = "one")]
    quantity: u32,
    /// Probability per win, `0.0..=1.0`.
    chance: f64,
    source: Vec<crate::content::SourceTag>,
}

/// Load-time bounds for a battle's `drops`: bounded rows, catalogue-shaped
/// quantities, a finite chance, and provenance on every row. A row names an item or (L084) a listed table
/// that pays only items: the win already pays coins and XP, and `realm win` clears bonus items alone.
pub(super) fn validate(
    drops: &[BattleDrop],
    tables: &crate::loot_tables::LootTables,
) -> io::Result<()> {
    if drops.len() > MAX_DROPS
        || drops.iter().any(|d| {
            let names_one = match &d.table {
                None => d.item_id > 0,
                Some(table) => d.item_id == 0 && tables.pays_only_items(table),
            };
            !names_one
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

/// The `(item, quantity)` rows whose independent roll hit, a table row adding what its table rolled. Pure:
/// the same rows, tables and generator state give the same drops.
fn roll(
    drops: &[BattleDrop],
    tables: &crate::loot_tables::LootTables,
    rng: &mut Rng,
) -> Vec<(i32, u32)> {
    let mut items = Vec::new();
    for drop in drops {
        if rng.next_f64() >= drop.chance {
            continue;
        }
        match &drop.table {
            None => items.push((drop.item_id, drop.quantity)),
            Some(table) => {
                let mut below = |n: u32| (rng.next_u64() % u64::from(n.max(1))) as u32;
                items.extend(tables.roll(table, &mut below).items);
            }
        }
    }
    items
}

impl EncounterEngine {
    /// Roll the won battle's drops. No battle row, no drops, or no OS
    /// randomness means no drop: a win is never refused over a bonus.
    pub(super) fn roll_battle_drops(&mut self, battle: u32) -> Vec<(i32, u32)> {
        // A fight file that is not a `field_battles.json` row carries its own drops.
        let fights = self.fight_state.drops.clone();
        let Some(drops) = (self.battles.iter().find(|b| b.id == battle))
            .map(|row| &row.drops)
            .or_else(|| fights.get(&battle))
            .filter(|drops| !drops.is_empty())
            .cloned()
        else {
            return Vec::new();
        };
        if self.power_up_rng.is_none() {
            self.power_up_rng = Rng::from_entropy().ok();
        }
        let Some(rng) = self.power_up_rng.as_mut() else {
            return Vec::new();
        };
        roll(&drops, self.field_rewards.loot_tables(), rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drop_row(item_id: i32, quantity: u32, chance: f64) -> BattleDrop {
        BattleDrop {
            item_id,
            table: None,
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
        let none = crate::loot_tables::LootTables::default();
        let roll = |drops: &[BattleDrop], seed| roll(drops, &none, &mut Rng::seeded(seed));
        assert_eq!(roll(&above, 7), vec![(893, 2)]);
        assert!(roll(&below, 7).is_empty());
        // The extremes never need the generator.
        assert!(roll(&[drop_row(893, 1, 0.0)], 7).is_empty());
        assert_eq!(roll(&[drop_row(893, 1, 1.0)], 7), vec![(893, 1)]);
    }

    /// L084: a table row is an end-of-dungeon pool: it rolls the table on a hit, and only a listed table that
    /// pays nothing but items may be named.
    #[test]
    fn a_table_row_rolls_its_pool_and_must_pay_items_only() {
        let tables = crate::loot_tables::LootTables::parse(
            serde_json::json!({"schema_version": 1, "tables": [
                {"key": "pool", "source": [{"kind": "reconstructed", "evidence": "(R) test"}],
                 "entries": [{"kind": "item", "weight": 1, "definition_id": 501, "quantity": 3}]},
                {"key": "coins", "source": [{"kind": "reconstructed", "evidence": "(R) test"}],
                 "entries": [{"kind": "coins", "weight": 1, "amount": 9}]},
            ]})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
        let table_row = |key: &str| BattleDrop {
            item_id: 0,
            table: Some(key.into()),
            ..drop_row(1, 1, 1.0)
        };
        assert_eq!(
            roll(&[table_row("pool")], &tables, &mut Rng::seeded(3)),
            vec![(501, 3)]
        );
        assert!(validate(&[table_row("pool")], &tables).is_ok());
        for bad in ["coins", "missing"] {
            assert!(validate(&[table_row(bad)], &tables).is_err(), "{bad}");
        }
        let both = BattleDrop {
            item_id: 893,
            ..table_row("pool")
        };
        assert!(
            validate(&[both], &tables).is_err(),
            "an item or a table, not both"
        );
    }

    #[test]
    fn bad_rows_are_refused_at_load_time() {
        let none = crate::loot_tables::LootTables::default();
        let validate = |drops: &[BattleDrop]| validate(drops, &none);
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
