//! L084 (Sanctuary port P3): keyed loot tables of weighted entries. Adapted from Sanctuary's reward tables
//! (BubbaJoeX fork of Open-Source-Free-Realms/Sanctuary @1ff876a, `src/Sanctuary.Game/Resources/Definitions/
//! Rewards/*.cs` and the drop rolls in `src/Sanctuary.Game/RewardManager.cs`, AGPL-3.0-or-later).
//!
//! A table rolls one entry by weight. An entry is an item with a quantity, coins, active-job XP, another
//! table (rolled in turn), or nothing. Content lives in `data/content/loot_tables.json`, so new tables, bags
//! and mob drops are rows, not code. The whole file is checked at load: bounded sizes, weights and amounts,
//! provenance on every table, and table references that exist and do not loop.
//!
//! Where loot belongs (owner field note 2026-10-31, `notes/Research/Owner Field Notes.md`): regular dungeon
//! and battle enemies dropped only power-ups; permanent items came from end-of-dungeon reward pools and from
//! bosses (green weapons until December 2009); open-world enemies dropped items in Sunstone Valley. So a
//! roll is opt-in per reward row, never a blanket chance on every enemy (Sanctuary's `mob-loot` is a guess).
use crate::content::SourceTag;
use serde::Deserialize;
use std::{collections::BTreeMap, io, path::Path};

pub const FILE: &str = "loot_tables.json";
const MAX_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TABLES: usize = 1024;
const MAX_ENTRIES: usize = 512;
const MAX_WEIGHT: u32 = 1_000_000;
const MAX_QUANTITY: u32 = 1_000;
const MAX_AMOUNT: u32 = 100_000;
/// Nested `table` entries may go this deep; a deeper or looping chain refuses to load.
const MAX_DEPTH: usize = 4;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct LootFile {
    schema_version: u32,
    #[serde(default)]
    tables: Vec<LootTable>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct LootTable {
    key: String,
    entries: Vec<Entry>,
    source: Vec<SourceTag>,
}

/// One weighted outcome of a roll.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Entry {
    Item {
        weight: u32,
        definition_id: i32,
        #[serde(default = "one")]
        quantity: u32,
    },
    Coins {
        weight: u32,
        amount: u32,
    },
    JobXp {
        weight: u32,
        amount: u32,
    },
    Table {
        weight: u32,
        key: String,
    },
    Nothing {
        weight: u32,
    },
}

fn one() -> u32 {
    1
}

impl Entry {
    fn weight(&self) -> u32 {
        match self {
            Self::Item { weight, .. }
            | Self::Coins { weight, .. }
            | Self::JobXp { weight, .. }
            | Self::Table { weight, .. }
            | Self::Nothing { weight } => *weight,
        }
    }
}

/// What a roll gave: coins, active-job XP and `(item definition, quantity)` pairs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Loot {
    pub coins: u32,
    pub job_xp: u32,
    pub items: Vec<(i32, u32)>,
}

impl Loot {
    pub fn is_empty(&self) -> bool {
        self.coins == 0 && self.job_xp == 0 && self.items.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LootTables {
    tables: BTreeMap<String, Vec<Entry>>,
}

fn invalid(what: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.into())
}

impl LootTables {
    /// `directory/loot_tables.json`; a missing file is no tables (old content packages).
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(FILE);
        let bytes = match std::fs::metadata(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
            Ok(meta) if meta.len() > MAX_BYTES => return Err(invalid("loot tables exceed 2 MiB")),
            Ok(_) => std::fs::read(&path)?,
        };
        Self::parse(&bytes)
    }

    pub fn parse(bytes: &[u8]) -> io::Result<Self> {
        let file: LootFile =
            serde_json::from_slice(bytes).map_err(|e| invalid(format!("loot tables: {e}")))?;
        if file.schema_version != 1 || file.tables.len() > MAX_TABLES {
            return Err(invalid(
                "loot tables: unsupported schema or too many tables",
            ));
        }
        let mut tables = BTreeMap::new();
        for table in file.tables {
            let key = table.key.trim();
            if key.is_empty() || key != table.key || key.len() > 64 {
                return Err(invalid(
                    "loot table: a key is empty, padded or over 64 bytes",
                ));
            }
            if table.entries.is_empty() || table.entries.len() > MAX_ENTRIES {
                return Err(invalid(format!(
                    "loot table {key}: 1 to {MAX_ENTRIES} entries"
                )));
            }
            crate::encounter::field_rewards::validate_source(&table.source)
                .map_err(|_| invalid(format!("loot table {key}: needs provenance")))?;
            for entry in &table.entries {
                let ok = (1..=MAX_WEIGHT).contains(&entry.weight())
                    && match entry {
                        Entry::Item {
                            definition_id,
                            quantity,
                            ..
                        } => *definition_id > 0 && (1..=MAX_QUANTITY).contains(quantity),
                        Entry::Coins { amount, .. } | Entry::JobXp { amount, .. } => {
                            (1..=MAX_AMOUNT).contains(amount)
                        }
                        Entry::Table { .. } | Entry::Nothing { .. } => true,
                    };
                if !ok {
                    return Err(invalid(format!(
                        "loot table {key}: an entry is out of bounds"
                    )));
                }
            }
            if tables.insert(table.key.clone(), table.entries).is_some() {
                return Err(invalid(format!("loot table {key} is listed twice")));
            }
        }
        let loaded = Self { tables };
        for key in loaded.tables.keys() {
            loaded.check_chain(key, 0)?;
        }
        Ok(loaded)
    }

    /// Every `table` reference exists and no chain is deeper than `MAX_DEPTH` (which also refuses loops).
    fn check_chain(&self, key: &str, depth: usize) -> io::Result<()> {
        if depth > MAX_DEPTH {
            return Err(invalid(format!(
                "loot table {key}: nested too deep or looping"
            )));
        }
        let entries = self
            .tables
            .get(key)
            .ok_or_else(|| invalid(format!("loot table {key} does not exist")))?;
        for entry in entries {
            if let Entry::Table { key, .. } = entry {
                self.check_chain(key, depth + 1)?;
            }
        }
        Ok(())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.tables.contains_key(key)
    }

    /// Whether `key` exists and can only ever pay items (no coin or job-XP entry, nested tables included).
    pub fn pays_only_items(&self, key: &str) -> bool {
        self.tables.get(key).is_some_and(|entries| {
            entries.iter().all(|entry| match entry {
                Entry::Coins { .. } | Entry::JobXp { .. } => false,
                Entry::Table { key, .. } => self.pays_only_items(key),
                Entry::Item { .. } | Entry::Nothing { .. } => true,
            })
        })
    }

    /// Roll `key` once. `below(n)` returns a uniform value in `0..n` (the caller's own random source); an
    /// unknown key rolls nothing. Load checks bound the nesting.
    pub fn roll(&self, key: &str, below: &mut impl FnMut(u32) -> u32) -> Loot {
        let mut loot = Loot::default();
        self.roll_into(key, below, &mut loot, 0);
        loot
    }

    fn roll_into(
        &self,
        key: &str,
        below: &mut impl FnMut(u32) -> u32,
        loot: &mut Loot,
        depth: usize,
    ) {
        let Some(entries) = self.tables.get(key).filter(|_| depth <= MAX_DEPTH) else {
            return;
        };
        let total: u32 = entries
            .iter()
            .map(Entry::weight)
            .fold(0, u32::saturating_add);
        let mut pick = below(total.max(1)).min(total.saturating_sub(1));
        let Some(entry) = entries.iter().find(|entry| {
            let hit = pick < entry.weight();
            pick = pick.saturating_sub(entry.weight());
            hit
        }) else {
            return;
        };
        match entry {
            Entry::Item {
                definition_id,
                quantity,
                ..
            } => match loot.items.iter_mut().find(|(id, _)| id == definition_id) {
                Some((_, count)) => *count = count.saturating_add(*quantity),
                None => loot.items.push((*definition_id, *quantity)),
            },
            Entry::Coins { amount, .. } => loot.coins = loot.coins.saturating_add(*amount),
            Entry::JobXp { amount, .. } => loot.job_xp = loot.job_xp.saturating_add(*amount),
            Entry::Table { key, .. } => self.roll_into(key, below, loot, depth + 1),
            Entry::Nothing { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn source() -> serde_json::Value {
        json!([{ "kind": "reconstructed", "evidence": "(R) test" }])
    }

    fn tables(tables: serde_json::Value) -> io::Result<LootTables> {
        LootTables::parse(
            json!({ "schema_version": 1, "tables": tables })
                .to_string()
                .as_bytes(),
        )
    }

    #[test]
    fn a_roll_picks_by_weight_follows_nested_tables_and_merges_items() {
        let loot = tables(json!([
            { "key": "bag", "source": source(), "entries": [
                { "kind": "coins", "weight": 1, "amount": 50 },
                { "kind": "table", "weight": 3, "key": "gems" },
            ]},
            { "key": "gems", "source": source(), "entries": [
                { "kind": "item", "weight": 2, "definition_id": 77, "quantity": 2 },
                { "kind": "nothing", "weight": 2 },
            ]},
        ]))
        .unwrap();
        // `below` returns its scripted values in order: 0 picks coins (weight 1 of 4).
        let roll = |script: &[u32]| {
            let mut values = script.iter().copied();
            loot.roll("bag", &mut |n| values.next().unwrap().min(n - 1))
        };
        assert_eq!(
            roll(&[0]),
            Loot {
                coins: 50,
                ..Loot::default()
            }
        );
        assert_eq!(
            roll(&[1, 0]),
            Loot {
                items: vec![(77, 2)],
                ..Loot::default()
            }
        );
        assert_eq!(roll(&[3, 3]), Loot::default(), "the nothing entry");
        assert!(loot.roll("missing", &mut |_| 0).is_empty());
    }

    #[test]
    fn bad_content_refuses_to_load() {
        let entry = json!({ "kind": "coins", "weight": 1, "amount": 5 });
        for (why, rows) in [
            (
                "no provenance",
                json!([{ "key": "a", "source": [], "entries": [entry] }]),
            ),
            (
                "empty",
                json!([{ "key": "a", "source": source(), "entries": [] }]),
            ),
            (
                "zero weight",
                json!([{ "key": "a", "source": source(), "entries": [{ "kind": "nothing", "weight": 0 }] }]),
            ),
            (
                "zero quantity",
                json!([{ "key": "a", "source": source(), "entries": [{ "kind": "item", "weight": 1, "definition_id": 5, "quantity": 0 }] }]),
            ),
            (
                "unknown table",
                json!([{ "key": "a", "source": source(), "entries": [{ "kind": "table", "weight": 1, "key": "b" }] }]),
            ),
            (
                "loop",
                json!([
                    { "key": "a", "source": source(), "entries": [{ "kind": "table", "weight": 1, "key": "b" }] },
                    { "key": "b", "source": source(), "entries": [{ "kind": "table", "weight": 1, "key": "a" }] },
                ]),
            ),
            (
                "twice",
                json!([{ "key": "a", "source": source(), "entries": [entry] }, { "key": "a", "source": source(), "entries": [entry] }]),
            ),
            (
                "unknown field",
                json!([{ "key": "a", "source": source(), "entries": [{ "kind": "coins", "weight": 1, "amount": 5, "bonus": 1 }] }]),
            ),
        ] {
            assert!(tables(rows).is_err(), "{why}");
        }
        assert!(tables(json!([])).unwrap().tables.is_empty());
    }
}
