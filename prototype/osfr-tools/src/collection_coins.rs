//! Which collection entry each placed exploration coin ticks (T449).
//!
//! `data/content/collection_coins.json` is written by `scripts/build_collection_coins.py` from our
//! coin spawn rows (`npc_spawns_extra.json`), the collection catalogue and pinned community data,
//! with per-row provenance. Correcting a pair is a one-row edit. Nothing at runtime reads it yet,
//! and an absent file keeps the feature dormant.

use crate::collection_catalogue::CollectionCatalogueContent;
use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, io::Read, path::Path};

pub const FILE_NAME: &str = "collection_coins.json";
pub const SCHEMA_VERSION: u32 = 1;
const MAX_FILE_BYTES: u64 = 1 << 20;
/// Sanity bounds on the tunable pickup range (R), not recovered limits.
const PICKUP_RANGE: std::ops::RangeInclusive<f32> = 1.0..=50.0;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoinsFile {
    pub schema_version: u32,
    /// (R) Sanctuary's coin interact range; not recovered from the client.
    pub pickup_range: f32,
    pub provenance: BTreeMap<String, String>,
    pub records: Vec<Coin>,
    pub gaps: Gaps,
}

/// One placed coin and the entry it ticks.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Coin {
    pub spawn_guid: u64,
    /// The spawn row's `legacy_npc_id`; null for a coin no legacy NPC row places.
    pub legacy_npc_id: Option<u32>,
    /// From the spawn row's name: 386170 "Rare Exploration Coin", 4412 "Elite Exploration Coin".
    pub tier: Tier,
    /// Always the catalogue owner of `entry_id`.
    pub collection_id: i32,
    pub entry_id: i32,
    pub pairing: Pairing,
    pub source: Vec<Source>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Rare,
    Elite,
}

impl Tier {
    /// The exploration collections, by client locale name: 265-276 "<place> Rare Exploration",
    /// 284-291 "<place> Elite Exploration", 311 = 3795 "Sunstone Valley Rare Exploration",
    /// 312 = 3797 "Sunstone Valley Elite Exploration".
    pub fn of_collection(collection_id: i32) -> Option<Self> {
        match collection_id {
            265..=276 | 311 => Some(Self::Rare),
            284..=291 | 312 => Some(Self::Elite),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Pairing {
    /// Checked in game, per the row's `playtest` source.
    Playtest,
    /// (R) Paired by id order, never checked.
    IdOrder,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: String,
    pub evidence: String,
}

/// Generated from the catalogue; informational, the loader does not act on it.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gaps {
    pub entries_without_coin: Vec<Gap>,
    pub notes: Vec<Note>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub collection_id: i32,
    pub entry_id: i32,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub entry_id: i32,
    pub spawn_guid: u64,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct CollectionCoins {
    pickup_range: f32,
    coins: BTreeMap<u64, Coin>,
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl CollectionCoins {
    /// Reads `collection_coins.json` from `dir`, checked against `catalogue`; `None` when absent.
    pub fn load(dir: &Path, catalogue: &CollectionCatalogueContent) -> io::Result<Option<Self>> {
        let mut bytes = Vec::new();
        match fs::File::open(dir.join(FILE_NAME)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            file => file?.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?,
        };
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(invalid("collection coins over 1 MiB".into()));
        }
        let file: CoinsFile = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(invalid("unsupported collection coins schema".into()));
        }
        if !PICKUP_RANGE.contains(&file.pickup_range) {
            return Err(invalid("pickup_range must be within 1..=50".into()));
        }
        let mut coins = BTreeMap::new();
        let mut entries = BTreeMap::new();
        for coin in file.records {
            let (guid, entry_id) = (coin.spawn_guid, coin.entry_id);
            let refuse = |reason: String| invalid(format!("coin {guid}: {reason}"));
            let Some((_, owner)) = catalogue.entry(entry_id) else {
                return Err(refuse(format!("entry {entry_id} is not in the catalogue")));
            };
            if coin.collection_id != owner.id {
                return Err(refuse(format!("entry {entry_id} belongs to {}", owner.id)));
            }
            if Tier::of_collection(owner.id) != Some(coin.tier) {
                return Err(refuse(format!(
                    "{:?} coin in collection {}",
                    coin.tier, owner.id
                )));
            }
            if let Some(other) = entries.insert(entry_id, guid) {
                return Err(refuse(format!("entry {entry_id} is bound to {other} too")));
            }
            if coins.insert(guid, coin).is_some() {
                return Err(refuse("spawn_guid is listed twice".into()));
            }
        }
        Ok(Some(Self {
            pickup_range: file.pickup_range,
            coins,
        }))
    }

    pub fn by_guid(&self, guid: u64) -> Option<&Coin> {
        self.coins.get(&guid)
    }

    pub fn pickup_range(&self) -> f32 {
        self.pickup_range
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &Coin)> {
        self.coins.iter().map(|(&guid, coin)| (guid, coin))
    }
}
