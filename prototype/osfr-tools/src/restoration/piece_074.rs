//! 074: collections and collectibles for one restored server.
//!
//! ORIGINAL facts implemented here, with the recovered source of each (all outside Git):
//! * `OSFR Client/Resources/CollectionCategories.txt` is a first-party client table with the
//!   header `#ID^NAME_ID^ICON_ID^` and ten rows, ids 1..10 in file order. Its rows are carried
//!   verbatim in [`COLLECTION_CATEGORIES`].
//! * `lab/decompiled/client-ui/ScriptsBase.lua` defines the client data source
//!   `BaseClient.CollectionCategory` as three columns — `CollectionCategoryID` 0,
//!   `CollectionCategoryNAME` 1, `CollectionCategoryICON_ID` 2 — and `BaseClient.CollectionData`
//!   as seventeen columns, `CollectionID` 0, `Name` 1, `Description` 2, `ImageID` 3, two unused
//!   columns, `EntryCount` 6, `ImageTintValue` 7, `CollectionCategoryID` 8 and exactly eight entry
//!   flags `EntryCollected1..8` at 9..16.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` `FUN_009fce20` returns 3 for the category row
//!   width, `FUN_009fce30` names those three columns, and `FUN_009fd050` buckets a category with
//!   `param_2 % 10`.
//! * The same client constructs `ClientUpdatePacketCollectionStart`,
//!   `ClientUpdatePacketCollectionAddEntry`, `ClientUpdatePacketCollectionRemoveEntry` and
//!   `ClientUpdatePacketCollectionRemove`, resolves `BaseClient.CollectionData.Reward`, and drives
//!   the `CollectionStarted` / `UpdateCollection` / `CollectionCompleted` UI notifications.
//!
//! COMMUNITY evidence, never a first-party claim: the pinned wiki corpus gives the resolved names
//! of the ten categories and the set/entry/area/spawn/reward shape of individual collections; the
//! pinned community server decodes a collection row into an entry list whose rows carry `Index`,
//! `Collected` and a reward bundle.
//!
//! RECONSTRUCTED here: the catalogue/ledger split, the durable document, every error and every
//! bound. The captured collection and entry ids, entry item links and reward rows live in
//! `data/content/collection_catalogue.json` (loader `collection_catalogue.rs`, T447), which builds
//! this catalogue. No node placement or radius was recovered, so none is invented —
//! [`CollectionDefinition::entries`] carries caller-supplied entry ids and nothing in this module
//! grants an item or currency.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// ORIGINAL: `CollectionCategories.txt` carries ten rows.
pub const COLLECTION_CATEGORY_COUNT: usize = 10;
/// RECONSTRUCTED schema version of the durable ledger document.
pub const LEDGER_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED document bound, in the spirit of the other restoration stores.
pub const MAX_LEDGER_BYTES: usize = 256 * 1024;
/// RECONSTRUCTED entry bound.
pub const MAX_LEDGER_ENTRIES: usize = 32_768;

/// ORIGINAL: one `CollectionCategories.txt` row. `name_id` resolves in the shipped locale, which is
/// the only place the ten category labels are written down; `icon_id` indexes the client icon set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionCategory {
    pub id: i32,
    pub name_id: i32,
    pub icon_id: i32,
}

/// ORIGINAL: the ten rows of `CollectionCategories.txt`, in file order.
pub const COLLECTION_CATEGORIES: [CollectionCategory; COLLECTION_CATEGORY_COUNT] = [
    CollectionCategory {
        id: 1,
        name_id: 93_636,
        icon_id: 9_535,
    },
    CollectionCategory {
        id: 2,
        name_id: 392_292,
        icon_id: 1_865,
    },
    CollectionCategory {
        id: 3,
        name_id: 401_015,
        icon_id: 7_220,
    },
    CollectionCategory {
        id: 4,
        name_id: 392_293,
        icon_id: 5_088,
    },
    CollectionCategory {
        id: 5,
        name_id: 392_294,
        icon_id: 1_433,
    },
    CollectionCategory {
        id: 6,
        name_id: 392_295,
        icon_id: 5_166,
    },
    CollectionCategory {
        id: 7,
        name_id: 392_296,
        icon_id: 7_536,
    },
    CollectionCategory {
        id: 8,
        name_id: 392_297,
        icon_id: 9_293,
    },
    CollectionCategory {
        id: 9,
        name_id: 390_416,
        icon_id: 4_566,
    },
    CollectionCategory {
        id: 10,
        name_id: 392_298,
        icon_id: 1_319,
    },
];

/// ORIGINAL: the category row for an id, if the id is one of the ten recovered rows.
pub fn category(category_id: i32) -> Option<CollectionCategory> {
    COLLECTION_CATEGORIES
        .iter()
        .copied()
        .find(|row| row.id == category_id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CollectionError {
    /// The definition names a category outside the ten recovered rows.
    UnknownCategory(i32),
    /// A collection definition with no entries can never be started or completed.
    EmptyCollection(i32),
    /// One entry id appears twice inside a single definition.
    DuplicateEntry(i32),
    /// A second definition reused a collection id.
    DuplicateCollection(i32),
    /// The entry is not part of any catalogued collection.
    UnknownEntry(i32),
    /// The durable document is unreadable, unsupported or out of bounds.
    Corrupt(&'static str),
}

impl std::fmt::Display for CollectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownCategory(id) => write!(formatter, "unknown collection category {id}"),
            Self::EmptyCollection(id) => write!(formatter, "collection {id} has no entries"),
            Self::DuplicateEntry(id) => write!(formatter, "entry {id} is listed twice"),
            Self::DuplicateCollection(id) => write!(formatter, "collection {id} already exists"),
            Self::UnknownEntry(id) => write!(formatter, "entry {id} is not catalogued"),
            Self::Corrupt(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for CollectionError {}

/// RECONSTRUCTED: one collection set and its entries. The entry identifiers are caller-supplied:
/// the recovered client exposes eight per-entry flags and an `EntryCount`, but the values behind
/// them were not recovered. The view limit is evidence, not a rule here, so this module neither
/// truncates a definition nor invents a display policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionDefinition {
    pub id: i32,
    pub category_id: i32,
    pub entries: Vec<i32>,
}

/// RECONSTRUCTED: the catalogue the ledger validates against.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CollectionCatalogue {
    definitions: Vec<CollectionDefinition>,
}

impl CollectionCatalogue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, definition: CollectionDefinition) -> Result<(), CollectionError> {
        if category(definition.category_id).is_none() {
            return Err(CollectionError::UnknownCategory(definition.category_id));
        }
        if definition.entries.is_empty() {
            return Err(CollectionError::EmptyCollection(definition.id));
        }
        let mut seen = BTreeSet::new();
        for entry in &definition.entries {
            if !seen.insert(*entry) {
                return Err(CollectionError::DuplicateEntry(*entry));
            }
        }
        if self.definition(definition.id).is_some() {
            return Err(CollectionError::DuplicateCollection(definition.id));
        }
        self.definitions.push(definition);
        self.definitions.sort_by_key(|row| row.id);
        Ok(())
    }

    pub fn definition(&self, collection_id: i32) -> Option<&CollectionDefinition> {
        self.definitions.iter().find(|row| row.id == collection_id)
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// The collection that owns an entry id.
    pub fn entry_owner(&self, entry_id: i32) -> Option<i32> {
        self.definitions
            .iter()
            .find(|row| row.entries.contains(&entry_id))
            .map(|row| row.id)
    }
}

/// RECONSTRUCTED: what a successful collect did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectOutcome {
    pub entry_id: i32,
    pub collection_id: i32,
    pub newly_collected: bool,
    pub completion: Option<CompletionRecord>,
}

/// RECONSTRUCTED: a completion, emitted to the caller exactly once per collection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionRecord {
    pub collection_id: i32,
    pub entry_count: usize,
}

/// RECONSTRUCTED: how far one collection is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionProgress {
    pub collected: usize,
    pub total: usize,
    pub complete: bool,
}

/// RECONSTRUCTED: the per-character collection ledger. Saved inside the character's
/// `ProgressionExtrasState`; both sets are persisted, so a missing one loads empty.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CollectionLedger {
    collected: BTreeSet<i32>,
    completed: BTreeSet<i32>,
}

impl CollectionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect(
        &mut self,
        catalogue: &CollectionCatalogue,
        entry_id: i32,
    ) -> Result<CollectOutcome, CollectionError> {
        let definition = catalogue
            .entry_owner(entry_id)
            .and_then(|collection_id| catalogue.definition(collection_id))
            .ok_or(CollectionError::UnknownEntry(entry_id))?;
        let newly_collected = self.collected.insert(entry_id);
        let complete = definition
            .entries
            .iter()
            .all(|entry| self.collected.contains(entry));
        // Exactly once: `insert` reports the transition, so a replay of any entry of an already
        // completed collection cannot re-emit the completion and grant its reward a second time.
        let newly_completed = complete && self.completed.insert(definition.id);
        let completion = newly_completed.then(|| CompletionRecord {
            collection_id: definition.id,
            entry_count: definition.entries.len(),
        });
        Ok(CollectOutcome {
            entry_id,
            collection_id: definition.id,
            newly_collected,
            completion,
        })
    }

    pub fn collected_count(&self) -> usize {
        self.collected.len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn is_collected(&self, entry_id: i32) -> bool {
        self.collected.contains(&entry_id)
    }

    pub fn is_completed(&self, collection_id: i32) -> bool {
        self.completed.contains(&collection_id)
    }

    pub fn progress(
        &self,
        catalogue: &CollectionCatalogue,
        collection_id: i32,
    ) -> Option<CollectionProgress> {
        let definition = catalogue.definition(collection_id)?;
        let collected = definition
            .entries
            .iter()
            .filter(|entry| self.collected.contains(entry))
            .count();
        Some(CollectionProgress {
            collected,
            total: definition.entries.len(),
            complete: collected == definition.entries.len(),
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CollectionError> {
        let document = LedgerDocument {
            schema_version: LEDGER_SCHEMA_VERSION,
            collected: self.collected.iter().copied().collect(),
            completed: self.completed.iter().copied().collect(),
        };
        let bytes = serde_json::to_vec_pretty(&document)
            .map_err(|_| CollectionError::Corrupt("ledger does not serialize"))?;
        if bytes.len() > MAX_LEDGER_BYTES {
            return Err(CollectionError::Corrupt("ledger exceeds its byte bound"));
        }
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CollectionError> {
        if bytes.len() > MAX_LEDGER_BYTES {
            return Err(CollectionError::Corrupt("ledger exceeds its byte bound"));
        }
        let document: LedgerDocument = serde_json::from_slice(bytes)
            .map_err(|_| CollectionError::Corrupt("ledger is not a valid document"))?;
        if document.schema_version != LEDGER_SCHEMA_VERSION {
            return Err(CollectionError::Corrupt("unsupported ledger version"));
        }
        if document.collected.len() > MAX_LEDGER_ENTRIES {
            return Err(CollectionError::Corrupt("ledger exceeds its entry bound"));
        }
        Ok(Self {
            collected: document.collected.into_iter().collect(),
            completed: document.completed.into_iter().collect(),
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerDocument {
    schema_version: u32,
    collected: Vec<i32>,
    completed: Vec<i32>,
}
