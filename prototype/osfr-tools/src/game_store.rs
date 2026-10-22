//! Version-seven character, gameplay, housing, pet and recipe snapshots, committed together.
//! Older envelopes migrate without changing character, gameplay or instance-counter state.
//! The pet roster rides in `Snapshot` (S17), so one `replace_bytes` carries it with the rest.
use crate::character_store::{CharacterState, replace_bytes};
use crate::gameplay::GameplayState;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

pub const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;
// (R) Local JSON format version; no original client save schema was recovered.
pub const BASE_SCHEMA_VERSION: u32 = 5;
pub const PET_SCHEMA_VERSION: u32 = 6;
pub const SCHEMA_VERSION: u32 = 7;
pub const GAMEPLAY_SCHEMA_VERSION: u32 = SCHEMA_VERSION;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub character: CharacterState,
    #[serde(default)]
    pub gameplay: Option<GameplayState>,
    /// The saved pet roster (`pets::PetRoster`), written in the same atomic replace as the
    /// character and gameplay, so an adoption's spent item and its new pet land together or not
    /// at all. `None` keeps the roster already on disk (an empty live roster sends `None`); a
    /// future feature that can shrink a roster (release, trade away) must send `Some`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pets: Option<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schema_version: Option<u32>,
    #[serde(default, skip_serializing)]
    version: Option<u32>,
    character: CharacterState,
    #[serde(default)]
    gameplay: Option<GameplayState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    housing: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pets: Option<Value>,
}

fn invalid(message: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub fn load_or_create(path: &Path, initial: CharacterState) -> io::Result<Snapshot> {
    let document = match read_document(path) {
        Ok(document) => document,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let initial = Snapshot {
                character: initial,
                gameplay: None,
                pets: None,
            };
            save(path, &initial)?;
            return Ok(initial);
        }
        Err(error) => return Err(error),
    };
    Ok(Snapshot {
        character: document.character,
        gameplay: document.gameplay,
        pets: document.pets,
    })
}

/// Write the whole character document in one atomic replace. Housing is read back from disk
/// (it still saves read-modify-write), and so is the roster when `snapshot.pets` is `None`.
pub fn save(path: &Path, snapshot: &Snapshot) -> io::Result<()> {
    snapshot.character.validate()?;
    let (housing, pets) = match read_document(path) {
        Ok(document) => (document.housing, snapshot.pets.clone().or(document.pets)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => (None, snapshot.pets.clone()),
        Err(error) => return Err(error),
    };
    let document = Document {
        // Keep legacy/housing-only saves on schema 5. Pet state requires schema 6.
        schema_version: Some(if snapshot.gameplay.is_some() {
            GAMEPLAY_SCHEMA_VERSION
        } else if pets.is_some() {
            PET_SCHEMA_VERSION
        } else {
            BASE_SCHEMA_VERSION
        }),
        version: None,
        character: snapshot.character.clone(),
        gameplay: snapshot.gameplay.clone(),
        housing,
        pets,
    };
    write_document(path, &document)
}

/// Load housing state from the same canonical save envelope. The caller supplies its own state
/// type so this module stays independent of the not-yet-routed housing service.
pub fn load_housing_state<T: DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    read_document(path)?
        .housing
        .map(serde_json::from_value)
        .transpose()
        .map_err(invalid)
}

/// Atomically replace only the housing payload while preserving character, gameplay and counters.
pub fn save_housing_state<T: Serialize>(path: &Path, housing: &T) -> io::Result<()> {
    let mut document = read_document(path)?;
    document.housing = Some(serde_json::to_value(housing).map_err(invalid)?);
    document.schema_version = Some(if document.gameplay.is_some() {
        GAMEPLAY_SCHEMA_VERSION
    } else if document.pets.is_some() {
        PET_SCHEMA_VERSION
    } else {
        BASE_SCHEMA_VERSION
    });
    document.version = None;
    write_document(path, &document)
}

fn read_document(path: &Path) -> io::Result<Document> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take((MAX_SNAPSHOT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(invalid("gameplay snapshot exceeds 1 MiB"));
    }
    let document: Document = serde_json::from_slice(&bytes).map_err(invalid)?;
    // Version 4 adds InventoryItem.stored_count, 5 housing, 6 pets, and 7
    // persisted NPC quest/learned-recipe state. Serde defaults migrate older
    // gameplay snapshots with empty new state (R); legacy counters stay intact.
    let supported = match (document.schema_version, document.version) {
        (Some(7 | 6 | 5 | 4 | 3), None) => true,
        (None, Some(2)) => true,
        (None, Some(1)) => document.gameplay.is_none(),
        _ => false,
    };
    if !supported {
        return Err(invalid("unsupported gameplay snapshot version"));
    }
    document.character.validate()?;
    Ok(document)
}

fn write_document(path: &Path, document: &Document) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(&document).map_err(invalid)?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(invalid("gameplay snapshot exceeds 1 MiB"));
    }
    replace_bytes(path, &bytes)
}
