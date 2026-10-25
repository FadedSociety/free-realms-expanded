//! Collection catalogue taken from our own login capture (T447).
//!
//! `data/content/collection_catalogue.json` is written by the ignored generator
//! `tests/collection_catalogue.rs::regenerate_or_check_collection_catalogue` from the SendSelf
//! template `lab/references/rust-gateway-reference/0007-login.bin`: one row per captured collection
//! record (client reader `009099d0`), its keyed entry rows (`008dacc0`) and its completion reward
//! bundle (`008e7930`, entry kinds from `0103c070`). The generator fails on any drift from the
//! template, so definitions and entries are never edited by hand; reward values may be tuned in the
//! file within the load bounds below. An absent file keeps the feature dormant.

use crate::{
    progression_extras::{CollectionCatalogue, CollectionDefinition, empty_grant},
    rewards::{ItemGrant, RewardGrant},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    io::Read,
    path::Path,
};

pub const FILE_NAME: &str = "collection_catalogue.json";
pub const SCHEMA_VERSION: u32 = 1;
const MAX_FILE_BYTES: u64 = 1 << 20;
/// `ScriptsBase.lua` `CollectionData_DS.EntryCollected1..8`: a collection shows at most eight.
const MAX_ENTRIES: usize = 8;
// Sanity bounds on tunable reward values (R), not recovered limits.
const MAX_COUNT: u32 = 100;
const MAX_XP: u32 = 100_000;
const MAX_COINS: u32 = 100_000;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CatalogueFile {
    pub schema_version: u32,
    pub provenance: BTreeMap<String, String>,
    /// One per captured record, in capture order.
    pub collections: Vec<Collection>,
    pub gaps: Gaps,
}

/// One collection record (`009099d0`: dwords to +0x9c, +0xa0, +0xa4, +0xa8, the icon pair, +0xb8).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    pub id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub category_id: i32,
    pub icon_id: i32,
    pub icon_tint_id: i32,
    /// (R) Record dword +0xb8: 1 on 251 rows, 0 on 277 and 278; meaning unknown.
    pub flag6: u32,
    pub entries: Vec<Entry>,
    pub reward: Reward,
}

/// One keyed entry row (`008dacc0`). The row's collected byte is the captured player's state and
/// is deliberately not kept.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: i32,
    pub index: u32,
    pub name_id: i32,
    pub icon_id: i32,
    pub icon_tint_id: i32,
    pub item_definition_id: Option<i32>,
    /// Where `item_definition_id` came from; the per-row provenance.
    pub item_link: Option<ItemLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_link_note: Option<ItemLinkNote>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum ItemLink {
    /// Sanctuary `Collections.json` `Entries[].ItemDefinitionId`.
    Sanctuary,
    /// The single Legacy item whose `NameId` is the entry's `name_id`.
    LegacyUniqueName,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemLinkNote {
    #[serde(rename = "no item with this name")]
    NoItem,
    #[serde(rename = "ambiguous name")]
    AmbiguousName,
}

/// Completion reward (`008e7930`). Display bytes stay in the template.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Reward {
    /// Bundle dword 0, stored at +0x50 and shown as "coins" by `00b89ca0` (vtable +0x50,
    /// `0091c2e0`). That completion paid it is (R).
    pub coins: u32,
    pub items: Vec<RewardItem>,
    pub xp: Vec<RewardXp>,
    /// (R) Kind 12 `Token` rows; what a token is for is unknown.
    pub tokens: Vec<RewardToken>,
    /// Kind 10 `CollectionAdd` tails (`CollectionID=%d`, `0103b800`).
    pub collection_add: Vec<i32>,
    /// Kind 6 `QuestAdd` tails (`QuestID=%d`, `0103bb50`).
    pub quest_add: Vec<i32>,
}

/// A kind-1 row: `+0x24` is the item (`Item=%d`, `0103b4a0`) and `+0x10` its display name
/// (`RewardEntryDataSource` column 1, `00c33430`).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RewardItem {
    /// The item granted, which must have a definition: the captured `+0x24` id, else (R) the
    /// single Legacy item named by `+0x10` (T630), else `None` (no item, the rest of the reward
    /// stands).
    pub item_definition_id: Option<i32>,
    pub count: u32,
    /// Lxzxrus `RewardItemIdsGirls`, only where it lists one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub female_item_definition_id: Option<i32>,
    /// The captured `+0x24` id, kept only when it has no definition and so is not granted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_item_definition_id: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RewardXp {
    pub profile_id: i32,
    pub amount: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RewardToken {
    pub token_id: i32,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Gaps {
    pub reward_items_missing_from_legacy: Vec<i32>,
    pub null_item_links: BTreeMap<ItemLinkNote, usize>,
    /// `collection_add` targets with no captured record.
    pub collection_add_not_captured: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct CollectionCatalogueContent {
    collections: Vec<Collection>,
    catalogue: CollectionCatalogue,
    /// Entry id to (collection position, entry position).
    entries: BTreeMap<i32, (usize, usize)>,
    /// Item id to the one entry linking it; `None` when several do.
    items: BTreeMap<i32, Option<i32>>,
    /// T522: source collection id to the meta collection entry its completion records.
    meta: BTreeMap<i32, i32>,
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

/// T522: source collection to meta entry, from the captured kind-10 `collection_add` targets.
///
/// A target T is accepted only when as many sets add it as T has entries, each source names one
/// T entry (equal `name_id` first, else equal `icon_id`; the icon rule is (R), no 309 source
/// matches a 309 name) and no two sources name the same entry, so the map is one-to-one. Pairs
/// whose source is itself a target are then dropped until stable, so a completion cascade stays
/// within two levels and cannot loop.
fn meta_map(collections: &[Collection]) -> BTreeMap<i32, i32> {
    let mut sources: BTreeMap<i32, Vec<&Collection>> = BTreeMap::new();
    for collection in collections {
        for target in &collection.reward.collection_add {
            sources.entry(*target).or_default().push(collection);
        }
    }
    let by_id: BTreeMap<i32, &Collection> = collections
        .iter()
        .map(|collection| (collection.id, collection))
        .collect();
    let mut pairs: Vec<(i32, i32, i32)> = Vec::new();
    for (target_id, set_sources) in sources {
        let Some(target) = by_id.get(&target_id) else {
            continue;
        };
        if set_sources.len() != target.entries.len() {
            continue;
        }
        let mut matched = Vec::with_capacity(set_sources.len());
        let mut used = BTreeSet::new();
        let mut one_to_one = true;
        for source in set_sources {
            let named: Vec<&Entry> = target
                .entries
                .iter()
                .filter(|entry| entry.name_id == source.name_id)
                .collect();
            let candidates: Vec<&Entry> = if named.is_empty() {
                target
                    .entries
                    .iter()
                    .filter(|entry| entry.icon_id == source.icon_id)
                    .collect()
            } else {
                named
            };
            let [entry] = candidates.as_slice() else {
                one_to_one = false;
                break;
            };
            if !used.insert(entry.id) {
                one_to_one = false;
                break;
            }
            matched.push((source.id, target_id, entry.id));
        }
        if one_to_one {
            pairs.extend(matched);
        }
    }
    loop {
        let targets: BTreeSet<i32> = pairs.iter().map(|(_, target, _)| *target).collect();
        let before = pairs.len();
        pairs.retain(|(source, _, _)| !targets.contains(source));
        if pairs.len() == before {
            break;
        }
    }
    pairs
        .into_iter()
        .map(|(source, _, entry)| (source, entry))
        .collect()
}

impl CollectionCatalogueContent {
    /// Reads `collection_catalogue.json` from `dir`; `None` when it is absent.
    pub fn load(dir: &Path) -> io::Result<Option<Self>> {
        let mut bytes = Vec::new();
        match fs::File::open(dir.join(FILE_NAME)) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            file => file?.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?,
        };
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(invalid("collection catalogue over 1 MiB".into()));
        }
        let file: CatalogueFile = serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(invalid("unsupported collection catalogue schema".into()));
        }
        let mut catalogue = CollectionCatalogue::new();
        let mut entries = BTreeMap::new();
        let mut items = BTreeMap::new();
        for (position, collection) in file.collections.iter().enumerate() {
            let id = collection.id;
            let refuse = |reason: String| invalid(format!("collection {id}: {reason}"));
            let mut indexes: Vec<u32> = collection.entries.iter().map(|e| e.index).collect();
            indexes.sort_unstable();
            if indexes.len() > MAX_ENTRIES
                || !indexes.into_iter().eq(1..=collection.entries.len() as u32)
            {
                return Err(refuse("entry indexes must run 1..=n, n at most 8".into()));
            }
            let reward = &collection.reward;
            let counts = reward.items.iter().map(|item| item.count);
            if reward.coins > MAX_COINS
                || !counts
                    .chain(reward.tokens.iter().map(|token| token.count))
                    .all(|count| (1..=MAX_COUNT).contains(&count))
                || !reward.xp.iter().all(|xp| (1..=MAX_XP).contains(&xp.amount))
            {
                return Err(refuse("reward value out of bounds".into()));
            }
            for (slot, entry) in collection.entries.iter().enumerate() {
                if entries.insert(entry.id, (position, slot)).is_some() {
                    return Err(refuse(format!("entry {} is listed twice", entry.id)));
                }
                if let Some(item) = entry.item_definition_id {
                    items
                        .entry(item)
                        .and_modify(|owner| *owner = None)
                        .or_insert(Some(entry.id));
                }
            }
            catalogue
                .insert(CollectionDefinition {
                    id,
                    category_id: collection.category_id,
                    entries: collection.entries.iter().map(|entry| entry.id).collect(),
                })
                .map_err(|error| refuse(error.to_string()))?;
        }
        let meta = meta_map(&file.collections);
        Ok(Some(Self {
            collections: file.collections,
            catalogue,
            entries,
            items,
            meta,
        }))
    }

    pub fn catalogue(&self) -> &CollectionCatalogue {
        &self.catalogue
    }

    pub fn collections(&self) -> &[Collection] {
        &self.collections
    }

    pub fn collection(&self, id: i32) -> Option<&Collection> {
        self.collections
            .iter()
            .find(|collection| collection.id == id)
    }

    /// An entry and the collection that owns it.
    pub fn entry(&self, entry_id: i32) -> Option<(&Entry, &Collection)> {
        let &(position, slot) = self.entries.get(&entry_id)?;
        let collection = &self.collections[position];
        Some((&collection.entries[slot], collection))
    }

    /// What finishing `collection_id` pays (T480): the captured coins, each reward item fitted to
    /// `gender`, and job XP for the profiles in `unlocked`. Tokens, `collection_add` and
    /// `quest_add` tails are never paid, since what they do is unknown (R). A girl (2) gets the
    /// listed girls' id where there is one, else the captured item (R); gender 0 gets the captured
    /// item (R). An item `item_known` does not accept is skipped so it cannot refuse the rest, and
    /// so is XP for a locked profile, which the reward applier would refuse whole (R). Unknown
    /// collection: the empty grant.
    pub fn completion_grant(
        &self,
        collection_id: i32,
        player_guid: u64,
        gender: i32,
        unlocked: &BTreeSet<i32>,
        item_known: impl Fn(i32) -> bool,
    ) -> RewardGrant {
        let mut grant = empty_grant();
        let Some(collection) = self.collection(collection_id) else {
            return grant;
        };
        let reward = &collection.reward;
        grant.coins = reward.coins;
        let fitted = |item: &RewardItem| match item.female_item_definition_id {
            Some(girls) if gender == 2 && item_known(girls) => Some(girls),
            _ => item.item_definition_id.filter(|&id| item_known(id)),
        };
        grant.items = reward
            .items
            .iter()
            .filter_map(|item| {
                Some(ItemGrant {
                    definition_id: fitted(item)?,
                    count: item.count,
                })
            })
            .collect();
        for xp in reward
            .xp
            .iter()
            .filter(|xp| unlocked.contains(&xp.profile_id))
        {
            let paid = grant.job_xp.entry(xp.profile_id).or_default();
            *paid = paid.saturating_add(xp.amount);
        }
        let display = &mut grant.display;
        display.source_guid = player_guid;
        display.player_guid = player_guid;
        display.icon_id = collection.icon_id;
        display.name_id = collection.name_id;
        grant
    }

    /// The entry an item fills, only when exactly one entry links that item.
    pub fn entry_for_item(&self, item_definition_id: i32) -> Option<i32> {
        self.items.get(&item_definition_id).copied().flatten()
    }

    /// T522: the meta collection entry that completing `collection_id` also records.
    pub fn meta_entry(&self, collection_id: i32) -> Option<i32> {
        self.meta.get(&collection_id).copied()
    }

    /// The accepted source-collection to meta-entry pairs (T522).
    pub fn meta_pairs(&self) -> &BTreeMap<i32, i32> {
        &self.meta
    }
}
