//! 095: multi-character persistence with a reversible single-character migration.
//!
//! The previous local storage assumed exactly one character per save file
//! (`src/character_store.rs`, version 1 `{"version":1,"character":{...}}`, and
//! `identity.rs` rejecting any guid but 1). This module is the version boundary in
//! front of that save: it reads only the real version-1 reader/writer, imports one
//! legacy character into a version-2 multi-character document, and can hand the
//! legacy document back byte-for-byte for rollback.
//!
//! ORIGINAL facts used here (provenance in
//! `notes/Research/Pieces/095 Multi-character persistence.md`):
//! * `decompiled/client-ui/ScriptsBase.lua`: `AccountInfo_DS.MaxCharacters = 1`,
//!   `CharacterGuid = 4`, `CharacterName = 5`; `CharacterSelect:Populate` invokes
//!   `addCharacter` once per account-info row and always appends the localized row
//!   `addCharacter", 300, 0, Ui.GetStringById(436106), "add"`;
//!   `CharacterSelect:GetMaxCharacters` returns `ds:GetData(0, AccountInfo_DS.MaxCharacters)`;
//!   `OnUserEvent("addCharacter")` continues only when `GetMaxCharacters() > 1` and
//!   otherwise shows locale 435820.
//! * Locale rows 436106 "Add a Character" and 435820 "Get more Characters!
//!   Become A Member!".
//!
//! COMMUNITY cross-check (pinned `lab/references/FreeRealms-Legacy`, not first-party):
//! `AccountInfo` writes IsMember/MaxCharacters/IsAdminAccount in that order,
//! `CharacterSelectInfoReply` (opcode 12) carries an entity list, the create handler
//! refuses beyond `DbUser.MaxCharacters` with result 6, `DbUser.MaxCharacters`
//! defaults to 10, and `GuidHelper` encodes a player guid as `id << 4 | 1`.
//!
//! RECONSTRUCTED here, labelled at each item:
//! * the version-2 document shape, the ordered character list and the `active`
//!   pointer. No original file of this shape is claimed to exist.
//! * the local slot bound. `MAX_CHARACTERS = 10` mirrors the community default of
//!   the recovered `MaxCharacters` account field; the original per-account value is
//!   unrecovered, and the login layer still owns how many slots an account may use.
//! * the migration provenance digest and the rollback rule: the legacy file is
//!   never rewritten by migration, and a rollback refuses when the store holds a
//!   character the version-1 document cannot represent.
//!
//! Deliberate boundaries:
//! * character creation, deletion, name checks and the login wire are not changed
//!   here; this module only owns the local save format and its migration.
//! * the store is a local defensive bound, not a recovered world or account limit.
//! * the legacy save must have one writer; migration reads it twice (digest then
//!   parse) and refuses rather than guesses when the recorded state diverges.
//!
//! ponytail: one version dispatch, one legacy shape, and the version-1 bytes are
//! produced by the real `character_store` writer instead of a local re-implementation,
//! so a rollback cannot drift from what the single-character build wrote.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

// INTEGRATION: when the lead registers `pub mod restoration;` / `piece_095` in
// `src/lib.rs`, replace this `#[path]` include with
// `use crate::character_store;` and keep the re-export below. Until then the owned
// test target includes the real version-1 module by path so this file compiles
// against the actual save format rather than a copy of it.
#[path = "../character_store.rs"]
#[allow(dead_code)]
mod character_store;

pub use character_store::CharacterState;

/// Version this module writes. Mirrors the `Document.version` field it serializes.
pub const STORE_VERSION: u32 = 2;
/// Version of the single-character save this module migrates from.
pub const LEGACY_VERSION: u32 = 1;
/// Local slot bound. RECONSTRUCTED: the recovered `MaxCharacters` account field is
/// community-defaulted to 10; the original per-account slot count is unrecovered.
pub const MAX_CHARACTERS: usize = 10;
/// Local defensive bound for the multi-character document, not a recovered limit.
pub const MAX_STORE_BYTES: usize = 64 * 1024;

/// Read the version-1 single-character save (creating `default` when absent), using
/// the real single-character reader so both entry points accept the same bytes.
pub fn load_legacy(path: &Path, default: CharacterState) -> io::Result<CharacterState> {
    character_store::load_or_create(path, default)
}

/// Write the canonical version-1 single-character save. Reused for rollback so the
/// restored document is byte-identical to what the single-character build wrote.
pub fn save_legacy(path: &Path, character: &CharacterState) -> io::Result<()> {
    character_store::save(path, character)
}

/// How `migrate_legacy` resolved the legacy save against the multi-character store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Migration {
    /// No store existed, so the legacy character became its first entry.
    Imported { guid: u64 },
    /// The store already held the legacy character and the legacy save is unchanged.
    Existing,
}

/// Provenance of the version-1 save a store was imported from. The digest is of the
/// exact source bytes; a later change to the legacy file is refused, never merged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MigrationSource {
    version: u32,
    sha256: String,
}

impl MigrationSource {
    fn validate(&self) -> io::Result<()> {
        if self.version != LEGACY_VERSION {
            return Err(invalid("unsupported migration source version"));
        }
        if self.sha256.len() != 64 || !self.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(invalid("invalid migration source digest"));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u32,
    #[serde(default)]
    active: Option<u64>,
    characters: Vec<CharacterState>,
    #[serde(default)]
    migrated_from: Option<MigrationSource>,
}

/// Ordered, bounded set of local characters plus the selected (`active`) guid.
///
/// Entries are keyed by [`CharacterState::guid`]; guids are unique and nonzero, and
/// each character carries its own zone, position and rotation. Nothing here is
/// shared between entries, so updating one character cannot change another.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiCharacterStore {
    characters: Vec<CharacterState>,
    active: Option<u64>,
    migrated_from: Option<MigrationSource>,
}

impl MultiCharacterStore {
    pub fn new() -> Self {
        Self {
            characters: Vec::new(),
            active: None,
            migrated_from: None,
        }
    }

    /// Start a store from one migrated legacy character.
    pub fn from_legacy(character: CharacterState) -> io::Result<Self> {
        let mut store = Self::new();
        store.insert(character)?;
        Ok(store)
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.characters.is_empty()
    }

    /// The entries in insertion order, the order the client renders rows in.
    pub fn characters(&self) -> &[CharacterState] {
        &self.characters
    }

    pub fn get(&self, guid: u64) -> Option<&CharacterState> {
        self.characters.iter().find(|c| c.guid == guid)
    }

    /// Guid of the selected character, when a selection has been persisted.
    pub fn active(&self) -> Option<u64> {
        self.active
    }

    /// Add a character. Refuses a duplicate guid, a guid of zero, invalid
    /// coordinates and anything beyond the local slot bound. The first entry
    /// becomes active so a one-character save still has a selection.
    pub fn insert(&mut self, character: CharacterState) -> io::Result<()> {
        character.validate()?;
        if self.get(character.guid).is_some() {
            return Err(invalid("duplicate character guid"));
        }
        if self.characters.len() >= MAX_CHARACTERS {
            return Err(invalid(format!(
                "multi-character store already holds {MAX_CHARACTERS} characters"
            )));
        }
        if self.active.is_none() {
            self.active = Some(character.guid);
        }
        self.characters.push(character);
        Ok(())
    }

    /// Replace the stored entry with the same guid. Unknown guids are refused rather
    /// than silently creating a character.
    pub fn update(&mut self, character: CharacterState) -> io::Result<()> {
        character.validate()?;
        let entry = self
            .characters
            .iter_mut()
            .find(|c| c.guid == character.guid)
            .ok_or_else(|| invalid("unknown character guid"))?;
        *entry = character;
        Ok(())
    }

    /// Persist which character the account selected last.
    pub fn select(&mut self, guid: u64) -> io::Result<()> {
        if self.get(guid).is_none() {
            return Err(invalid("unknown character guid"));
        }
        self.active = Some(guid);
        Ok(())
    }

    fn validate(&self) -> io::Result<()> {
        if self.characters.len() > MAX_CHARACTERS {
            return Err(invalid(format!(
                "multi-character store holds more than {MAX_CHARACTERS} characters"
            )));
        }
        for character in &self.characters {
            character.validate()?;
        }
        for (index, character) in self.characters.iter().enumerate() {
            if self.characters[..index]
                .iter()
                .any(|earlier| earlier.guid == character.guid)
            {
                return Err(invalid("duplicate character guid"));
            }
        }
        if let Some(active) = self.active {
            if self.get(active).is_none() {
                return Err(invalid("active character is not in the store"));
            }
        }
        if let Some(source) = &self.migrated_from {
            source.validate()?;
        }
        Ok(())
    }
}

impl Default for MultiCharacterStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Load an existing multi-character store. Corrupt, unsupported, oversized and
/// unreadable stores are errors; they are never reset or rewritten here.
pub fn load(path: &Path) -> io::Result<MultiCharacterStore> {
    let bytes = read_bounded(path, MAX_STORE_BYTES)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no multi-character store"))?;
    let document: Document = serde_json::from_slice(&bytes).map_err(invalid)?;
    if document.version != STORE_VERSION {
        return Err(invalid("unsupported multi-character store version"));
    }
    let store = MultiCharacterStore {
        characters: document.characters,
        active: document.active,
        migrated_from: document.migrated_from,
    };
    store.validate()?;
    Ok(store)
}

/// Atomically replace the multi-character store, same-directory, including Windows.
/// There must be only one writer for this file.
pub fn save(path: &Path, store: &MultiCharacterStore) -> io::Result<()> {
    store.validate()?;
    let document = Document {
        version: STORE_VERSION,
        active: store.active,
        characters: store.characters.clone(),
        migrated_from: store.migrated_from.clone(),
    };
    let bytes = serde_json::to_vec_pretty(&document).map_err(invalid)?;
    if bytes.len() > MAX_STORE_BYTES {
        return Err(invalid(format!(
            "multi-character store exceeds {} KiB",
            MAX_STORE_BYTES / 1024
        )));
    }
    character_store::replace_bytes(path, &bytes)
}

/// Import the version-1 single-character save into a version-2 multi-character
/// store, or confirm that an existing store already represents it.
///
/// The legacy file is never rewritten: it stays the rollback source, and its digest
/// is recorded in the store so a later change is refused instead of silently
/// dropped. A missing legacy file is created only when neither save exists, exactly
/// like the single-character `load_or_create` did.
pub fn migrate_legacy(
    legacy_path: &Path,
    store_path: &Path,
    default: CharacterState,
) -> io::Result<Migration> {
    let store_exists = store_path.exists();
    let legacy_bytes = read_bounded(legacy_path, character_store::MAX_FILE_BYTES)?;
    // Absence is only tolerated when a store already exists to answer with.
    let legacy_character = if legacy_bytes.is_none() && store_exists {
        None
    } else {
        Some(load_legacy(legacy_path, default)?)
    };
    // Reading may have just created the version-1 file, so digest it then.
    let digest = match (legacy_bytes.as_deref(), &legacy_character) {
        (Some(bytes), _) => Some(digest(bytes)),
        (None, Some(_)) => Some(digest(
            &read_bounded(legacy_path, character_store::MAX_FILE_BYTES)?
                .ok_or_else(|| invalid("legacy save disappeared during migration"))?,
        )),
        (None, None) => None,
    };

    if !store_exists {
        let character = legacy_character.expect("a new store imports the created legacy save");
        let mut store = MultiCharacterStore::from_legacy(character)?;
        let guid = store.active.expect("the first entry is active");
        store.migrated_from = digest.map(source);
        save(store_path, &store)?;
        return Ok(Migration::Imported { guid });
    }

    let mut store = load(store_path)?;
    if let Some(recorded) = store.migrated_from.clone() {
        let current = digest.ok_or_else(|| invalid("legacy save is missing after migration"))?;
        if current != recorded.sha256 {
            return Err(invalid(
                "legacy save changed after migration; resolve the divergence first",
            ));
        }
        return Ok(Migration::Existing);
    }

    let Some(character) = legacy_character else {
        return Ok(Migration::Existing);
    };
    let guid = character.guid;
    match store.get(guid) {
        Some(existing) if existing == &character => {}
        Some(_) => {
            return Err(invalid(
                "legacy save conflicts with the multi-character store",
            ));
        }
        None => store.insert(character)?,
    }
    store.migrated_from = digest.map(source);
    save(store_path, &store)?;
    Ok(Migration::Imported { guid })
}

/// Reverse the migration for one character: write the canonical version-1 document
/// back to `legacy_path`.
///
/// Refuses unless the store holds exactly that one character, because a
/// single-character document cannot represent the rest and rollback must never
/// silently drop a character.
pub fn rollback_to_legacy(
    store: &MultiCharacterStore,
    legacy_path: &Path,
    guid: u64,
) -> io::Result<()> {
    let character = store
        .get(guid)
        .ok_or_else(|| invalid("store has no such character"))?;
    if store.len() != 1 {
        return Err(invalid(
            "a version-1 save cannot represent more than one character",
        ));
    }
    save_legacy(legacy_path, character)
}

fn source(sha256: String) -> MigrationSource {
    MigrationSource {
        version: LEGACY_VERSION,
        sha256,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_bounded(path: &Path, maximum: usize) -> io::Result<Option<Vec<u8>>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take((maximum + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid(format!(
            "save exceeds the {} KiB local bound",
            maximum / 1024
        )));
    }
    Ok(Some(bytes))
}

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
