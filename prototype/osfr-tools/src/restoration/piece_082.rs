//! Piece 082 - inventory search crash: a deterministic preflight for the
//! client-side search row model.
//!
//! ORIGINAL native facts this module is built on (all under `../lab`,
//! outside this repository):
//!
//! * `decompiled/client/FreeRealms/pseudocode.c:1170126` and `:1170136` create
//!   `BaseClient.InventoryItemsData` and a second `InventoryDataSource` named
//!   `BaseClient.SearchInventoryItemsData`; `:1170140` writes the search flag at
//!   object offset `0x2e8` on that second source only. The source constructor is
//!   `FUN_00a6deb0` (`:1340855`), which also owns the `IString<unsigned short>`
//!   search text at offset `0x2f0`.
//! * `FUN_00a6e910` (`:1341483`) is the refresh path. With the flag clear it
//!   takes the plain rebuild branch; with the flag set it resolves the stored
//!   search string (`FUN_0083d250`), rebuilds the visible rows through the data
//!   source's per-row predicate virtual at vtable slot `+0x3c`
//!   (`FUN_00a6e7b0`, `:1341392`), copies the applied text to offset `0x2ec`
//!   and clears the pending flag at `0x2d0`.
//! * The crashed run (`lab/runs/client-baseline-20261008T033402Z`) recorded exit
//!   `0xC0000005` and its last client-to-server frame 21.6 s before the exit, so
//!   the failing path is local; no server request was captured for the search.
//!
//! RECONSTRUCTED (deliberately labelled, never presented as recovered):
//! the row model below. Each owned instance projects to one search row through
//! the catalogue the client actually received, and the preflight reports rows
//! whose definition, category or localized name cannot resolve. It exists so a
//! native reproduction can state exactly which data state was under test.
//!
//! GAPS: the predicate's match rule (case folding, substring vs prefix, which
//! field is compared) is not recovered, so this module never reproduces a
//! filtered result set and never claims the crash cause. It grants nothing and
//! changes no content. the private recovery checker `piece_082.py` runs the same model over the
//! preserved crash-run inputs and cross-checks the fingerprint produced here.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// Data source the client creates for the Stuff search field.
pub const SEARCH_DATA_SOURCE_NAME: &str = "BaseClient.SearchInventoryItemsData";
/// Data source the client creates for the plain item list.
pub const PLAIN_ROWS_DATA_SOURCE_NAME: &str = "BaseClient.InventoryItemsData";
/// Byte flag that marks a data source as the search source (`pseudocode.c:1170140`).
pub const SEARCH_FLAG_OFFSET: usize = 0x2e8;
/// Offset of the pending-search `IString` inside the data source.
pub const SEARCH_TEXT_OFFSET: usize = 0x2f0;
/// Offset the refresh path copies the applied search text to.
pub const APPLIED_SEARCH_TEXT_OFFSET: usize = 0x2ec;
/// Offset cleared once a search refresh completed.
pub const SEARCH_STALE_FLAG_OFFSET: usize = 0x2d0;
/// Virtual slot of the per-row predicate used while rebuilding rows.
pub const SEARCH_ROW_PREDICATE_VTABLE_SLOT: usize = 0x3c;
/// Text entered in the crashed native reproduction.
pub const SEARCH_QUERY: &str = "Wheeler";

/// One owned inventory instance as it reaches the row list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedInstance {
    pub instance_id: u32,
    pub definition_id: u32,
    pub rental: bool,
}

/// Catalogue row the client received for one definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionRow {
    pub category_id: i32,
    pub name_id: u32,
}

/// Catalogue state a row can resolve against.
#[derive(Clone, Debug, Default)]
pub struct ContentIndex {
    definitions: BTreeMap<u32, DefinitionRow>,
    categories: BTreeSet<i32>,
    localized: BTreeSet<u32>,
}

impl ContentIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a received definition. Repeats of an identical record are
    /// allowed (the captured stream contains 54 such repeats); a conflicting
    /// record for the same id is refused rather than silently overwritten.
    pub fn insert_definition(
        &mut self,
        definition_id: u32,
        category_id: i32,
        name_id: u32,
    ) -> Result<(), PreflightError> {
        if definition_id == 0 {
            return Err(PreflightError::InvalidDefinitionId(definition_id));
        }
        if category_id < 0 {
            return Err(PreflightError::InvalidCategoryId(category_id));
        }
        let row = DefinitionRow {
            category_id,
            name_id,
        };
        match self.definitions.get(&definition_id) {
            Some(existing) if *existing != row => {
                Err(PreflightError::ConflictingDefinition(definition_id))
            }
            Some(_) => Ok(()),
            None => {
                self.definitions.insert(definition_id, row);
                Ok(())
            }
        }
    }

    /// Register one category id from the 44/2 category table.
    pub fn insert_category(&mut self, category_id: i32) -> Result<(), PreflightError> {
        if category_id < 0 {
            return Err(PreflightError::InvalidCategoryId(category_id));
        }
        self.categories.insert(category_id);
        Ok(())
    }

    /// Register a name id the locale export resolves.
    pub fn insert_localized_name(&mut self, name_id: u32) {
        self.localized.insert(name_id);
    }

    pub fn definition(&self, definition_id: u32) -> Option<&DefinitionRow> {
        self.definitions.get(&definition_id)
    }

    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn category_count(&self) -> usize {
        self.categories.len()
    }
}

/// Resolution state of one row after the preflight projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowState {
    /// The definition resolved; carried values come from the received record.
    Resolved { category_id: i32, name_id: u32 },
    /// The client's received catalogue has no record for this definition id.
    MissingDefinition,
}

/// One projected search row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchRow {
    pub instance_id: u32,
    pub definition_id: u32,
    pub rental: bool,
    pub state: RowState,
}

/// A row whose definition id never arrived in the client's catalogue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingDefinition {
    pub instance_id: u32,
    pub definition_id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreflightError {
    InvalidInstanceId(u32),
    InvalidDefinitionId(u32),
    InvalidCategoryId(i32),
    DuplicateInstanceId(u32),
    ConflictingDefinition(u32),
}

impl std::fmt::Display for PreflightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreflightError::InvalidInstanceId(id) => write!(f, "instance id {id} is not valid"),
            PreflightError::InvalidDefinitionId(id) => {
                write!(f, "definition id {id} is not valid")
            }
            PreflightError::InvalidCategoryId(id) => write!(f, "category id {id} is negative"),
            PreflightError::DuplicateInstanceId(id) => {
                write!(f, "instance id {id} appears more than once")
            }
            PreflightError::ConflictingDefinition(id) => {
                write!(f, "definition {id} repeats with different values")
            }
        }
    }
}

impl std::error::Error for PreflightError {}

/// What one preflight pass found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreflightReport {
    pub instance_count: usize,
    pub definition_count: usize,
    pub category_count: usize,
    pub localized_name_count: usize,
    /// Rows in instance-id order (the order the client indexes them).
    pub rows: Vec<SearchRow>,
    pub missing_definitions: Vec<MissingDefinition>,
    /// definition count per category id that the 44/2 table does not contain.
    pub definitions_missing_category: BTreeMap<i32, u32>,
    /// Sorted unique name ids referenced by definitions the locale lacks.
    pub definitions_missing_name: Vec<u32>,
    /// SHA-256 over the canonical row/catalogue text; identifies the exact data
    /// state a native reproduction ran against.
    pub fingerprint: String,
}

impl PreflightReport {
    pub fn resolved_rows(&self) -> usize {
        self.rows.len() - self.missing_definitions.len()
    }

    pub fn unresolved_rows(&self) -> usize {
        self.missing_definitions.len()
    }
}

/// One owned-item row list plus the catalogue it resolves against.
#[derive(Clone, Debug, Default)]
pub struct InventorySearch {
    instances: BTreeMap<u32, OwnedInstance>,
    content: ContentIndex,
}

impl InventorySearch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_instance(&mut self, instance: OwnedInstance) -> Result<(), PreflightError> {
        if instance.instance_id == 0 {
            return Err(PreflightError::InvalidInstanceId(instance.instance_id));
        }
        if instance.definition_id == 0 {
            return Err(PreflightError::InvalidDefinitionId(instance.definition_id));
        }
        if self.instances.contains_key(&instance.instance_id) {
            return Err(PreflightError::DuplicateInstanceId(instance.instance_id));
        }
        self.instances.insert(instance.instance_id, instance);
        Ok(())
    }

    pub fn content_mut(&mut self) -> &mut ContentIndex {
        &mut self.content
    }

    pub fn content(&self) -> &ContentIndex {
        &self.content
    }

    pub fn report(&self) -> PreflightReport {
        let mut rows = Vec::with_capacity(self.instances.len());
        let mut missing_definitions = Vec::new();
        for instance in self.instances.values() {
            let state = match self.content.definition(instance.definition_id) {
                Some(row) => RowState::Resolved {
                    category_id: row.category_id,
                    name_id: row.name_id,
                },
                None => {
                    missing_definitions.push(MissingDefinition {
                        instance_id: instance.instance_id,
                        definition_id: instance.definition_id,
                    });
                    RowState::MissingDefinition
                }
            };
            rows.push(SearchRow {
                instance_id: instance.instance_id,
                definition_id: instance.definition_id,
                rental: instance.rental,
                state,
            });
        }

        let mut definitions_missing_category = BTreeMap::new();
        let mut definitions_missing_name = BTreeSet::new();
        for row in self.content.definitions.values() {
            if !self.content.categories.contains(&row.category_id) {
                *definitions_missing_category
                    .entry(row.category_id)
                    .or_insert(0) += 1;
            }
            if row.name_id != 0 && !self.content.localized.contains(&row.name_id) {
                definitions_missing_name.insert(row.name_id);
            }
        }

        let fingerprint = canonical_fingerprint(
            &rows,
            &self.content.definitions,
            &self.content.categories,
            &self.content.localized,
        );

        PreflightReport {
            instance_count: self.instances.len(),
            definition_count: self.content.definitions.len(),
            category_count: self.content.categories.len(),
            localized_name_count: self.content.localized.len(),
            rows,
            missing_definitions,
            definitions_missing_category,
            definitions_missing_name: definitions_missing_name.into_iter().collect(),
            fingerprint,
        }
    }
}

/// Canonical text hashed into [`PreflightReport::fingerprint`]. The Python
/// checker mirrors this byte for byte so both implementations agree on the
/// data state under test.
fn canonical_fingerprint(
    rows: &[SearchRow],
    definitions: &BTreeMap<u32, DefinitionRow>,
    categories: &BTreeSet<i32>,
    localized: &BTreeSet<u32>,
) -> String {
    let mut text = String::new();
    for row in rows {
        let state = match row.state {
            RowState::Resolved { .. } => "resolved",
            RowState::MissingDefinition => "missing",
        };
        text.push_str(&format!(
            "row {} {} {} {}\n",
            row.instance_id,
            row.definition_id,
            u8::from(row.rental),
            state
        ));
    }
    for (definition_id, row) in definitions {
        text.push_str(&format!(
            "def {} {} {}\n",
            definition_id, row.category_id, row.name_id
        ));
    }
    for category_id in categories {
        text.push_str(&format!("cat {category_id}\n"));
    }
    for name_id in localized {
        text.push_str(&format!("name {name_id}\n"));
    }
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
