//! Versioned account/character JSON storage. Writes share character_store's
//! synced same-directory temporary-file replacement and Windows retry path.

use crate::{
    account::{Account, AccountFlags, CharacterRecord, SlotPolicy},
    character_store::{self, CharacterState},
};
use osfr_tools::identity::{self, CharacterIdentity};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
};

// (R) Local envelope version; no original account save format was recovered.
const SCHEMA_VERSION: u32 = 1;
// (R) Local defensive file bound.
pub const MAX_FILE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreData {
    /// (R) Local monotonic allocator; no original account-ID allocator is known.
    next_account_id: u64,
    /// (R) Local monotonic allocator; no original character-GUID encoding is known.
    next_character_guid: u64,
    accounts: BTreeMap<u64, Account>,
    characters: BTreeMap<u64, CharacterRecord>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    store: StoreData,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCharacterDocument {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    version: Option<u32>,
    character: CharacterState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountStore {
    data: StoreData,
}

impl AccountStore {
    pub fn new() -> Self {
        Self {
            data: StoreData {
                next_account_id: 1,
                next_character_guid: 1,
                accounts: BTreeMap::new(),
                characters: BTreeMap::new(),
            },
        }
    }

    /// Account IDs advance in the persisted store and are not recycled.
    pub fn create_account(
        &mut self,
        login_name_hash: [u8; 32],
        flags: AccountFlags,
        slots: SlotPolicy,
    ) -> io::Result<u64> {
        // (R) Require at least one configured slot and one account per auth hash.
        if slots.max_characters == 0
            || self
                .data
                .accounts
                .values()
                .any(|account| account.login_name_hash == login_name_hash)
        {
            return Err(invalid_data("invalid or duplicate account record"));
        }
        let id = self.data.next_account_id;
        self.data.next_account_id = id
            .checked_add(1)
            .ok_or_else(|| invalid_data("account id space exhausted"))?;
        self.data.accounts.insert(
            id,
            Account {
                id,
                login_name_hash,
                flags,
                slots,
                character_guids: Vec::new(),
            },
        );
        Ok(id)
    }

    pub fn account(&self, id: u64) -> Option<&Account> {
        self.data.accounts.get(&id)
    }

    pub fn account_id_by_login_hash(&self, login_name_hash: &[u8; 32]) -> Option<u64> {
        self.data
            .accounts
            .values()
            .find(|account| &account.login_name_hash == login_name_hash)
            .map(|account| account.id)
    }

    pub fn character(&self, guid: u64) -> Option<&CharacterRecord> {
        self.data.characters.get(&guid)
    }

    pub(crate) fn character_name_taken(&self, first: &str, last: &str) -> bool {
        // (R) Use case-insensitive global uniqueness across displayed and pending names.
        let first = first.to_lowercase();
        let last = last.to_lowercase();
        self.data.characters.values().any(|character| {
            name_keys(&character.identity)
                .iter()
                .any(|(known_first, known_last)| known_first == &first && known_last == &last)
        })
    }

    pub(crate) fn insert_character(
        &mut self,
        account_id: u64,
        identity: CharacterIdentity,
        world_state: Option<CharacterState>,
    ) -> io::Result<u64> {
        identity
            .validate()
            .map_err(|_| invalid_data("invalid character identity"))?;
        let account = self
            .data
            .accounts
            .get(&account_id)
            .ok_or_else(|| invalid_data("unknown account"))?;
        if account.character_guids.len() >= account.slots.max_characters as usize {
            return Err(invalid_data("account character slots are full"));
        }
        if name_keys(&identity)
            .iter()
            .any(|(first, last)| self.character_name_taken(first, last))
        {
            return Err(invalid_data("character name is already assigned"));
        }
        let guid = self.data.next_character_guid;
        let next_guid = guid
            .checked_add(1)
            .ok_or_else(|| invalid_data("character guid space exhausted"))?;
        if world_state.as_ref().is_some_and(|state| state.guid != guid) {
            return Err(invalid_data("character world state guid mismatch"));
        }
        if let Some(state) = &world_state {
            state.validate()?;
        }
        self.data.next_character_guid = next_guid;
        self.data.characters.insert(
            guid,
            CharacterRecord {
                guid,
                owner_account_id: account_id,
                identity,
                world_state,
            },
        );
        self.data
            .accounts
            .get_mut(&account_id)
            .expect("account checked above")
            .character_guids
            .push(guid);
        Ok(guid)
    }

    fn adopt_legacy_character(
        &mut self,
        account_id: u64,
        state: CharacterState,
        identity: CharacterIdentity,
    ) -> io::Result<()> {
        state.validate()?;
        identity
            .validate()
            .map_err(|_| invalid_data("invalid legacy character identity"))?;
        let account = self
            .data
            .accounts
            .get(&account_id)
            .ok_or_else(|| invalid_data("unknown account"))?;
        if account.slots.max_characters == 0
            || name_keys(&identity)
                .iter()
                .any(|(first, last)| self.character_name_taken(first, last))
        {
            return Err(invalid_data("legacy character cannot be adopted"));
        }
        let guid = state.guid;
        if self.data.characters.contains_key(&guid) {
            return Err(invalid_data("legacy character guid is already assigned"));
        }
        let next_guid = guid
            .checked_add(1)
            .ok_or_else(|| invalid_data("character guid space exhausted"))?;
        self.data.next_character_guid = self.data.next_character_guid.max(next_guid);
        self.data.characters.insert(
            guid,
            CharacterRecord {
                guid,
                owner_account_id: account_id,
                identity,
                world_state: Some(state),
            },
        );
        self.data
            .accounts
            .get_mut(&account_id)
            .expect("account checked above")
            .character_guids
            .push(guid);
        Ok(())
    }

    pub(crate) fn remove_character(&mut self, account_id: u64, guid: u64) -> bool {
        let Some(character) = self.data.characters.get(&guid) else {
            return false;
        };
        if character.owner_account_id != account_id {
            return false;
        }
        self.data.characters.remove(&guid);
        if let Some(account) = self.data.accounts.get_mut(&account_id) {
            account.character_guids.retain(|owned| *owned != guid);
        }
        true
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.validate()?;
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&Document {
            schema_version: SCHEMA_VERSION,
            store: self.data.clone(),
        })
        .map_err(invalid_data)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(invalid_data("account store exceeds 1 MiB"));
        }
        character_store::replace_bytes(path, &bytes)
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take((MAX_FILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(invalid_data("account store exceeds 1 MiB"));
        }
        let document: Document = serde_json::from_slice(&bytes).map_err(invalid_data)?;
        if document.schema_version != SCHEMA_VERSION {
            return Err(invalid_data("unsupported account store schema version"));
        }
        let store = Self {
            data: document.store,
        };
        store.validate()?;
        Ok(store)
    }

    /// Adopt the existing per-character save and identity without changing
    /// either legacy file. An existing account store is authoritative.
    pub fn load_or_migrate(
        path: &Path,
        legacy_save_path: &Path,
        legacy_identity_path: &Path,
        login_name_hash: [u8; 32],
        flags: AccountFlags,
        slots: SlotPolicy,
    ) -> io::Result<Self> {
        match File::open(path) {
            Ok(_) => return Self::load(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let state = read_legacy_state(legacy_save_path)?;
        let identity = identity::load(legacy_identity_path)?
            .ok_or_else(|| invalid_data("legacy character identity is missing"))?;
        let mut store = Self::new();
        let account_id = store.create_account(login_name_hash, flags, slots)?;
        store.adopt_legacy_character(account_id, state, identity)?;
        store.save(path)?;
        Ok(store)
    }

    fn validate(&self) -> io::Result<()> {
        if self.data.next_account_id == 0 || self.data.next_character_guid == 0 {
            return Err(invalid_data("account store allocator is invalid"));
        }
        let mut hashes = BTreeSet::new();
        let mut owned = BTreeSet::new();
        for (id, account) in &self.data.accounts {
            if *id == 0
                || account.id != *id
                || account.slots.max_characters == 0
                || account.character_guids.len() > account.slots.max_characters as usize
                || !hashes.insert(account.login_name_hash)
            {
                return Err(invalid_data("invalid account record"));
            }
            let mut account_guids = BTreeSet::new();
            for guid in &account.character_guids {
                if !account_guids.insert(*guid) || !owned.insert(*guid) {
                    return Err(invalid_data("duplicate account character reference"));
                }
                let Some(character) = self.data.characters.get(guid) else {
                    return Err(invalid_data("account references a missing character"));
                };
                if character.owner_account_id != *id {
                    return Err(invalid_data("character account ownership mismatch"));
                }
            }
        }
        let mut names = BTreeSet::new();
        let mut max_guid = 0;
        for (guid, character) in &self.data.characters {
            if *guid == 0
                || character.guid != *guid
                || !owned.contains(guid)
                || character.identity.validate().is_err()
            {
                return Err(invalid_data("invalid character record"));
            }
            for name in name_keys(&character.identity) {
                if !names.insert(name) {
                    return Err(invalid_data("duplicate character name"));
                }
            }
            if let Some(state) = &character.world_state {
                state.validate()?;
                if state.guid != *guid {
                    return Err(invalid_data("character world state guid mismatch"));
                }
            }
            max_guid = max_guid.max(*guid);
        }
        let max_account = self.data.accounts.keys().copied().max().unwrap_or(0);
        if self.data.next_account_id <= max_account || self.data.next_character_guid <= max_guid {
            return Err(invalid_data("account store allocator would reuse an id"));
        }
        Ok(())
    }
}

fn name_keys(identity: &CharacterIdentity) -> BTreeSet<(String, String)> {
    [
        (&identity.first_name, &identity.last_name),
        (
            &identity.creation.temporary_first_name,
            &identity.creation.temporary_last_name,
        ),
    ]
    .into_iter()
    .filter(|(first, last)| !first.is_empty() && !last.is_empty())
    .map(|(first, last)| (first.to_lowercase(), last.to_lowercase()))
    .collect()
}

fn read_legacy_state(path: &Path) -> io::Result<CharacterState> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take((character_store::MAX_FILE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > character_store::MAX_FILE_BYTES {
        return Err(invalid_data("legacy character save exceeds 16 KiB"));
    }
    let document: LegacyCharacterDocument = serde_json::from_slice(&bytes).map_err(invalid_data)?;
    match (document.schema_version, document.version) {
        (Some(2), None) | (None, Some(1)) => {}
        _ => return Err(invalid_data("unsupported legacy character save version")),
    }
    document.character.validate()?;
    Ok(document.character)
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
