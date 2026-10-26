//! Which collection entry each placed exploration coin ticks (T449) and the
//! live authoring overlay (T525).
//!
//! `data/content/collection_coins.json` is written by `scripts/build_collection_coins.py` from our
//! coin spawn rows (`npc_spawns_extra.json`), the collection catalogue and pinned community data,
//! with per-row provenance. Correcting a pair is a one-row edit; in game it is never rewritten.
//! `collection_coin_edits.json` sits beside the live-world placements and holds one row per
//! authored coin, actor and binding together, so one atomic write covers both. An absent overlay
//! keeps the shipped map; an absent shipped map keeps the feature dormant.

use crate::collection_catalogue::CollectionCatalogueContent;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    io::Read,
    path::Path,
};

pub const FILE_NAME: &str = "collection_coins.json";
pub const SCHEMA_VERSION: u32 = 1;
/// The live overlay beside `npc_placements.json` (T525).
pub const EDITS_FILE_NAME: &str = "collection_coin_edits.json";
pub const EDITS_SCHEMA_VERSION: u32 = 1;
/// Live authored coin GUIDs start here, above `PLACEMENT_GUID_BASE` and below 132e9.
pub const EDIT_GUID_BASE: u64 = 131_000_000_000;
pub const EDIT_GUID_END: u64 = 132_000_000_000;
/// The only overlay source kind; the character GUID is the author, never a name.
pub const LIVE_SOURCE: &str = "reconstructed";
const MAX_FILE_BYTES: u64 = 1 << 20;
/// (R) The overlay is a per-coin correction layer, not a second content set. The
/// cap also keeps a full file well under `MAX_FILE_BYTES`, so an admin cannot
/// write an overlay no later login can read.
const MAX_EDITS: usize = 512;
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

/// The live-world overlay (`collection_coin_edits.json`): one actor and its binding.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CoinEditsFile {
    pub schema_version: u32,
    /// The next live GUID to hand out; it only grows, so an unbound row's GUID is never
    /// reused (a session that missed the unbind would keep the old actor). 0 = not set.
    #[serde(default)]
    pub next_guid: u64,
    pub records: Vec<CoinEdit>,
}

/// One authored coin. The row carries the actor and the entry together, so a
/// coin can never be drawn at one spot while ticking a different binding.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CoinEdit {
    pub spawn_guid: u64,
    pub model_id: u32,
    pub name_string_id: u32,
    pub position: [f32; 3],
    #[serde(default)]
    pub heading: f32,
    pub collection_id: i32,
    pub entry_id: i32,
    pub tier: Tier,
    pub pairing: Pairing,
    pub source: EditSource,
    /// A content coin GUID this row hides for everyone, when the entry is repointed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EditSource {
    pub kind: String,
    pub recorded_ms: u64,
    pub character_guid: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
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

    /// 386170 "Rare Exploration Coin", 4412 "Elite Exploration Coin": the `name_string_id` of
    /// every shipped coin row (`npc_spawns_extra.json`, from Legacy `Npcs.json` Id=4436 and
    /// Id=2115). The client locale (`en_us_data.dat`) holds both names but is keyed by hash,
    /// so the id-to-hash tie is not checked here (R).
    pub fn name_string_id(self) -> u32 {
        match self {
            Self::Rare => 386_170,
            Self::Elite => 4_412,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Rare => "rare",
            Self::Elite => "elite",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Pairing {
    /// A community playtester checked it against the basis the row's `playtest` source names
    /// (wiki, videos or guides; not 2014 evidence).
    Playtest,
    /// (R) Paired by id order, never checked.
    IdOrder,
    /// (R) Authored in game by `realm coin`; the binding is our chosen correction.
    Live,
}

impl Pairing {
    pub fn name(self) -> &'static str {
        match self {
            Self::Playtest => "playtest",
            Self::IdOrder => "id-order",
            Self::Live => "live",
        }
    }
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
    /// Shipped bindings plus overlay rows, with every replaced content coin removed.
    coins: BTreeMap<u64, Coin>,
    /// The validated overlay rows, in file order, for the actor view and rewrites.
    edits: Vec<CoinEdit>,
    /// Content coin GUIDs hidden for everyone by an overlay row.
    replaced: BTreeSet<u64>,
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

impl CollectionCoins {
    /// Reads `collection_coins.json` from `dir`, checked against `catalogue`; `None` when the
    /// shipped map is absent. Shipped map only: the overlay is read by `read_overlay` /
    /// `Gameplay::set_world_dir`.
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
            if coin.pairing == Pairing::Live {
                return Err(refuse("pairing live belongs to the live overlay".into()));
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
            edits: Vec::new(),
            replaced: BTreeSet::new(),
        }))
    }

    /// Merges validated overlay rows over the shipped bindings: a row with `replaces`
    /// repoints its entry and hides the named content coin; every other row needs an
    /// entry no content coin binds.
    pub fn merge(
        base: Self,
        catalogue: &CollectionCatalogueContent,
        edits: Vec<CoinEdit>,
    ) -> io::Result<Self> {
        if edits.len() > MAX_EDITS {
            return Err(invalid(format!(
                "collection coin edits must hold at most {MAX_EDITS} rows"
            )));
        }
        let mut live_guids = BTreeSet::new();
        let mut live_entries = BTreeSet::new();
        let mut replaced = BTreeSet::new();
        for edit in &edits {
            let guid = edit.spawn_guid;
            let refuse = |reason: String| invalid(format!("coin edit {guid}: {reason}"));
            if !(EDIT_GUID_BASE..EDIT_GUID_END).contains(&guid) {
                return Err(refuse(format!(
                    "spawn_guid must be within {EDIT_GUID_BASE}..{EDIT_GUID_END}"
                )));
            }
            if !live_guids.insert(guid) {
                return Err(refuse("spawn_guid is listed twice".into()));
            }
            if !live_entries.insert(edit.entry_id) {
                return Err(refuse(format!(
                    "entry {} has two live coins",
                    edit.entry_id
                )));
            }
            let Some((_, owner)) = catalogue.entry(edit.entry_id) else {
                return Err(refuse(format!(
                    "entry {} is not in the catalogue",
                    edit.entry_id
                )));
            };
            if edit.collection_id != owner.id {
                return Err(refuse(format!(
                    "entry {} belongs to {}",
                    edit.entry_id, owner.id
                )));
            }
            if Tier::of_collection(owner.id) != Some(edit.tier) {
                return Err(refuse(format!(
                    "{:?} coin in collection {}",
                    edit.tier, owner.id
                )));
            }
            if edit.pairing != Pairing::Live {
                return Err(refuse("pairing must be live".into()));
            }
            if edit.source.kind != LIVE_SOURCE {
                return Err(refuse(format!("source kind must be {LIVE_SOURCE}")));
            }
            if !edit.position.iter().all(|value| value.is_finite()) || !edit.heading.is_finite() {
                return Err(refuse("position and heading must be finite".into()));
            }
            if edit.model_id == 0 || edit.model_id > i32::MAX as u32 {
                return Err(refuse("model_id must fit a positive i32".into()));
            }
            if edit.name_string_id != edit.tier.name_string_id() {
                return Err(refuse(format!(
                    "name_string_id must be {} for {:?}",
                    edit.tier.name_string_id(),
                    edit.tier
                )));
            }
            match edit.replaces {
                Some(old) => {
                    let Some(bound) = base.coins.get(&old) else {
                        return Err(refuse(format!("replaces {old} is not a content coin")));
                    };
                    if bound.entry_id != edit.entry_id || bound.collection_id != edit.collection_id
                    {
                        return Err(refuse(format!(
                            "replaces {old} is bound to entry {}",
                            bound.entry_id
                        )));
                    }
                    if !replaced.insert(old) {
                        return Err(refuse(format!("replaces {old} twice")));
                    }
                }
                None => {
                    if let Some((other, _)) = base
                        .coins
                        .iter()
                        .find(|(_, coin)| coin.entry_id == edit.entry_id)
                    {
                        return Err(refuse(format!(
                            "entry {} is bound to {other}; name it in replaces",
                            edit.entry_id
                        )));
                    }
                }
            }
        }
        let mut coins = base.coins;
        for old in &replaced {
            coins.remove(old);
        }
        for edit in &edits {
            coins.insert(
                edit.spawn_guid,
                Coin {
                    spawn_guid: edit.spawn_guid,
                    legacy_npc_id: None,
                    tier: edit.tier,
                    collection_id: edit.collection_id,
                    entry_id: edit.entry_id,
                    pairing: edit.pairing,
                    source: vec![Source {
                        kind: LIVE_SOURCE.into(),
                        evidence: "(R) authored in game; the overlay row is the source".into(),
                    }],
                },
            );
        }
        Ok(Self {
            pickup_range: base.pickup_range,
            coins,
            edits,
            replaced,
        })
    }

    pub fn by_guid(&self, guid: u64) -> Option<&Coin> {
        self.coins.get(&guid)
    }

    /// The coin currently bound to `entry_id`, content or live.
    pub fn coin_for_entry(&self, entry_id: i32) -> Option<(u64, &Coin)> {
        self.coins
            .iter()
            .find(|(_, coin)| coin.entry_id == entry_id)
            .map(|(guid, coin)| (*guid, coin))
    }

    pub fn pickup_range(&self) -> f32 {
        self.pickup_range
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &Coin)> {
        self.coins.iter().map(|(&guid, coin)| (guid, coin))
    }

    /// The overlay rows, in file order.
    pub fn edits(&self) -> &[CoinEdit] {
        &self.edits
    }

    pub fn edit(&self, guid: u64) -> Option<&CoinEdit> {
        self.edits.iter().find(|edit| edit.spawn_guid == guid)
    }

    pub fn overlay_guids(&self) -> impl Iterator<Item = u64> + '_ {
        self.edits.iter().map(|edit| edit.spawn_guid)
    }

    pub fn replaced(&self) -> impl Iterator<Item = u64> + '_ {
        self.replaced.iter().copied()
    }
}

/// The rows of [`read_overlay`], for the session watch.
pub fn read_edits(path: &Path) -> io::Result<Vec<CoinEdit>> {
    Ok(read_overlay(path)?.records)
}

/// Reads `collection_coin_edits.json`; a missing file is an empty overlay.
pub fn read_overlay(path: &Path) -> io::Result<CoinEditsFile> {
    let mut bytes = Vec::new();
    match fs::File::open(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(CoinEditsFile {
                schema_version: EDITS_SCHEMA_VERSION,
                next_guid: 0,
                records: Vec::new(),
            });
        }
        file => file?.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?,
    };
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid("collection coin edits over 1 MiB".into()));
    }
    let mut file: CoinEditsFile = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if file.schema_version != EDITS_SCHEMA_VERSION {
        return Err(invalid("unsupported collection coin edits schema".into()));
    }
    if file.next_guid != 0 && !(EDIT_GUID_BASE..=EDIT_GUID_END).contains(&file.next_guid) {
        return Err(invalid("next_guid is outside the live GUID band".into()));
    }
    // An overlay written before the counter existed has none: lift it above every row, so an
    // unbound top row's GUID is never handed out again.
    let above_rows = file
        .records
        .iter()
        .map(|edit| edit.spawn_guid.saturating_add(1));
    file.next_guid = file.next_guid.max(above_rows.max().unwrap_or(0));
    Ok(file)
}

/// The exact bytes written to `collection_coin_edits.json`, one atomic write per command.
pub fn overlay_bytes(file: &CoinEditsFile) -> io::Result<Vec<u8>> {
    let text = serde_json::to_string_pretty(file).map_err(|error| invalid(error.to_string()))?;
    Ok((text + "\n").into_bytes())
}
