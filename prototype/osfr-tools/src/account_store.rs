//! Versioned account/character JSON storage. Writes share character_store's
//! synced same-directory temporary-file replacement and Windows retry path.

use crate::{
    account::{Account, AccountFlags, CharacterRecord, SlotPolicy},
    character_store::{self, CharacterState},
    login_auth::validate_public_key,
    moderation::Sanctions,
};
use osfr_tools::identity::{self, CharacterIdentity};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

// (R) Local envelope version; no original account save format was recovered.
const SCHEMA_VERSION: u32 = 3;
const ROLE_SCHEMA_VERSION: u32 = 2;
const LEGACY_SCHEMA_VERSION: u32 = 1;
// (R) Local defensive file bound.
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
// (R) A small rotation/recovery allowance per account; no historical key model.
pub const MAX_PUBLIC_KEYS: usize = 4;

/// Local account authority. No original Free Realms role model was recovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    Player,
    Builder,
    /// L042: mute, unmute, kick, ban and unban only.
    Moderator,
    Admin,
}

impl AccountRole {
    /// Parse the exact role names stored in accounts.json and accepted by `realm role`.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "player" => Some(Self::Player),
            "builder" => Some(Self::Builder),
            "moderator" => Some(Self::Moderator),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    /// (R) Moderation hierarchy: Admin > Moderator > Builder = Player. Staff act only on a
    /// strictly lower role, so equals (and admins) cannot moderate each other.
    pub fn outranks(self, target: Self) -> bool {
        let rank = |role| match role {
            Self::Player | Self::Builder => 0,
            Self::Moderator => 1,
            Self::Admin => 2,
        };
        rank(self) > rank(target)
    }
}

/// Reconstructed local authority for developer/tool `realm` commands; unknown
/// commands are admin-only. Ordinary gameplay commands bypass this check.
/// `command` must be the route's canonical lowercase command key. Content reloads
/// include their subtype (for example, `reload battles`).
pub fn can_run_realm_command(role: AccountRole, command: &str) -> bool {
    match role {
        AccountRole::Admin => true,
        AccountRole::Builder => matches!(
            command,
            "place"
                | "unplace"
                | "mark"
                | "unmark"
                | "marks"
                | "placements"
                | "text"
                | "texts"
                | "goto"
                | "here"
                | "reload all"
                | "reload battles"
                | "reload enemies"
                | "reload abilities"
                | "reload behaviours"
                | "reload power_ups"
                | "reload placements"
        ),
        // (R) L042: moderators get the moderation commands and nothing else.
        AccountRole::Moderator => matches!(command, "mute" | "unmute" | "kick" | "ban" | "unban"),
        AccountRole::Player => false,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreData {
    /// (R) Local monotonic allocator; no original account-ID allocator is known.
    next_account_id: u64,
    /// (R) Local monotonic allocator; no original character-GUID encoding is known.
    next_character_guid: u64,
    accounts: BTreeMap<u64, Account>,
    characters: BTreeMap<u64, CharacterRecord>,
    /// (R) Authority is local policy, stored by account ID outside the recovered account record.
    #[serde(default)]
    roles: BTreeMap<u64, AccountRole>,
    /// (R) Launcher public keys only. Empty means disabled, never passwordless.
    /// Kept outside Account to preserve its recovered pre-game record shape.
    #[serde(default)]
    public_keys: BTreeMap<u64, Vec<[u8; 32]>>,
    /// L042 mutes and bans by account ID (unix seconds); absent means none. Older files load
    /// without it; an older binary refuses a file that has it (deny_unknown_fields).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    sanctions: BTreeMap<u64, Sanctions>,
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

/// (R) Cooperative single-writer lock for login, the gateway's staff writes and the offline CLI.
/// Every writer acquires it before loading the store and holds it across one load-modify-save
/// only (L042: login per request, not for its life). A stale lock is an operator recovery
/// decision, never removed on age/PID alone.
pub struct AccountStoreLock(PathBuf);

/// (R) How long a writer waits out another writer's load-modify-save before reporting busy.
const LOCK_WAIT: std::time::Duration = std::time::Duration::from_millis(500);

/// (R) The gateway's staff writes (`realm role` and moderation) run on the world loop that
/// every player shares, so they wait far less: a stale lock costs that loop 50 ms per command.
pub const STAFF_LOCK_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

impl AccountStoreLock {
    pub fn acquire(store_path: &Path) -> io::Result<Self> {
        Self::acquire_within(store_path, LOCK_WAIT)
    }

    /// `acquire`, giving up with `AlreadyExists` once `wait` has passed. Giving up prints one
    /// stderr line (the host's sign of a stale lock left by a killed writer).
    pub fn acquire_within(store_path: &Path, wait: std::time::Duration) -> io::Result<Self> {
        let mut name = store_path.as_os_str().to_os_string();
        name.push(".lock");
        let path = PathBuf::from(name);
        let deadline = std::time::Instant::now() + wait;
        let mut file = loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Err(error)
                    if error.kind() == io::ErrorKind::AlreadyExists
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    eprintln!(
                        "account store lock {} is held; request refused",
                        path.display()
                    );
                    return Err(error);
                }
                opened => break opened?,
            }
        };
        let guard = Self(path);
        writeln!(file, "pid={}", std::process::id())?;
        file.sync_all()?;
        Ok(guard)
    }
}

impl Drop for AccountStoreLock {
    /// A lock left behind refuses every later writer, so a failed removal is reported on the
    /// request that leaves it (stderr; the guard has no security log in reach).
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.0) {
            eprintln!(
                "account store lock {} could not be removed: {error}",
                self.0.display()
            );
        }
    }
}

impl AccountStore {
    pub fn new() -> Self {
        Self {
            data: StoreData {
                next_account_id: 1,
                next_character_guid: 1,
                accounts: BTreeMap::new(),
                characters: BTreeMap::new(),
                roles: BTreeMap::new(),
                public_keys: BTreeMap::new(),
                sanctions: BTreeMap::new(),
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
        // (R) Keep every new account, including ID 1, at player. The login path
        // auto-creates accounts from client-selected names, so first-login admin
        // would let a client race to seize authority. Bootstrap an admin locally.
        self.data.roles.insert(id, AccountRole::Player);
        self.data.public_keys.insert(id, Vec::new());
        Ok(id)
    }

    pub fn account(&self, id: u64) -> Option<&Account> {
        self.data.accounts.get(&id)
    }

    pub fn account_role(&self, id: u64) -> Option<AccountRole> {
        self.data.roles.get(&id).copied()
    }

    pub fn sanctions(&self, id: u64) -> Sanctions {
        self.data.sanctions.get(&id).copied().unwrap_or_default()
    }

    /// Callers pass an existing account (validated on save); no sanction removes the row.
    pub(crate) fn set_sanctions(&mut self, id: u64, sanctions: Sanctions) {
        if sanctions == Sanctions::default() {
            self.data.sanctions.remove(&id);
        } else {
            self.data.sanctions.insert(id, sanctions);
        }
    }

    /// The account that owns every character whose displayed name has these words (compared
    /// by `name_words`); none when no character matches or the matches have different owners.
    /// Sanctions are per account, so one account's look-alike characters stay one target.
    pub(crate) fn account_by_display_name(&self, words: &[String]) -> Option<u64> {
        let wanted = name_words(words.iter().map(String::as_str));
        let mut owners = self
            .data
            .characters
            .values()
            .filter(|character| {
                let identity = &character.identity;
                name_words([identity.first_name.as_str(), identity.last_name.as_str()]) == wanted
            })
            .map(|character| character.owner_account_id);
        let owner = owners.next()?;
        owners.all(|other| other == owner).then_some(owner)
    }

    pub fn account_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.data.accounts.keys().copied()
    }

    pub fn public_keys(&self, id: u64) -> Option<&[[u8; 32]]> {
        self.data.public_keys.get(&id).map(Vec::as_slice)
    }

    /// (R) Operator provisioning, not a client-authorized mutation. The owner
    /// must serialize and persist this change, or use the offline CLI lock.
    pub fn add_public_key(&mut self, id: u64, bytes: &[u8]) -> io::Result<()> {
        let key = validate_public_key(bytes).map_err(invalid_data)?;
        let keys = self
            .data
            .public_keys
            .get(&id)
            .ok_or_else(|| invalid_data("unknown account"))?;
        if keys.len() >= MAX_PUBLIC_KEYS
            || self
                .data
                .public_keys
                .values()
                .any(|known| known.contains(&key))
        {
            return Err(invalid_data("public key is duplicate or key limit reached"));
        }
        self.data
            .public_keys
            .get_mut(&id)
            .ok_or_else(|| invalid_data("unknown account"))?
            .push(key);
        Ok(())
    }

    /// (R) Removing the last key disables login; it does not delete the account.
    pub fn remove_public_key(&mut self, id: u64, bytes: &[u8]) -> io::Result<()> {
        let key = validate_public_key(bytes).map_err(invalid_data)?;
        let keys = self
            .data
            .public_keys
            .get_mut(&id)
            .ok_or_else(|| invalid_data("unknown account"))?;
        let index = keys
            .iter()
            .position(|known| *known == key)
            .ok_or_else(|| invalid_data("public key is not enrolled"))?;
        keys.remove(index);
        Ok(())
    }

    /// Change a role only when `actor_account_id` is already an admin.
    /// The caller must persist the store after a successful change.
    pub fn set_account_role(
        &mut self,
        actor_account_id: u64,
        target_account_id: u64,
        role: AccountRole,
    ) -> io::Result<()> {
        if self.account_role(actor_account_id) != Some(AccountRole::Admin) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "admin role required",
            ));
        }
        self.set_account_role_offline(target_account_id, role)
    }

    /// (R) Local operator bootstrap/recovery only. Never route this method from
    /// a remote packet. Network role changes use the admin-checked method above.
    /// The offline CLI holds AccountStoreLock; host filesystem ACLs are authority.
    pub fn set_account_role_offline(
        &mut self,
        target_account_id: u64,
        role: AccountRole,
    ) -> io::Result<()> {
        if !self.data.accounts.contains_key(&target_account_id) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "target account does not exist",
            ));
        }
        self.data.roles.insert(target_account_id, role);
        Ok(())
    }

    /// (R) L057 local operator command only (`osfr-account member`); never route it from a
    /// packet. The offline CLI holds AccountStoreLock; host filesystem ACLs are authority.
    pub fn set_membership_offline(&mut self, account_id: u64, member: bool) -> io::Result<()> {
        let account = self.data.accounts.get_mut(&account_id).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "target account does not exist")
        })?;
        account.flags.membership = member;
        Ok(())
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

    /// L043 (`realm move`, admin only): the one character whose displayed name equals `name`,
    /// ignoring case. `None` when no character or several match, so a name never picks one of two.
    pub fn character_by_display_name(&self, name: &str) -> Option<u64> {
        let name = name.to_lowercase();
        let mut found = self
            .data
            .characters
            .iter()
            .filter(|(_, record)| record.identity.display_name().to_lowercase() == name);
        match (found.next(), found.next()) {
            (Some((guid, _)), None) => Some(*guid),
            _ => None,
        }
    }

    pub(crate) fn character_name_taken(&self, first: &str, last: &str) -> bool {
        // (R) Global uniqueness across displayed and pending names by their folded words, as
        // `realm` moderation reads a name: 'Al Bo' 'Cy' takes 'Al' 'Bo Cy' (L042) and 'Éva' takes
        // 'Eva' (L049). Every stored pair counts, one with an empty part included, because the
        // resolver reads those too. `validate` stays pair-based, so older stores keep loading.
        let wanted = name_words([first, last]);
        !wanted.is_empty()
            && self.data.characters.values().any(|character| {
                let (identity, creation) = (&character.identity, &character.identity.creation);
                [
                    (&identity.first_name, &identity.last_name),
                    (
                        &creation.temporary_first_name,
                        &creation.temporary_last_name,
                    ),
                ]
                .into_iter()
                .any(|(known_first, known_last)| {
                    name_words([known_first.as_str(), known_last.as_str()]) == wanted
                })
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
        if name_pairs(&identity).any(|(first, last)| self.character_name_taken(first, last)) {
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
            || name_pairs(&identity).any(|(first, last)| self.character_name_taken(first, last))
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
        let mut data = document.store;
        if document.schema_version < SCHEMA_VERSION {
            if !data.public_keys.is_empty() {
                return Err(invalid_data("legacy account store contains public keys"));
            }
            // (R) Migration never enrols keys implicitly or changes existing IDs.
            data.public_keys = data.accounts.keys().map(|id| (*id, Vec::new())).collect();
        }
        match document.schema_version {
            LEGACY_SCHEMA_VERSION => {
                if !data.roles.is_empty() {
                    return Err(invalid_data("schema 1 account store contains roles"));
                }
                data.roles = data
                    .accounts
                    .keys()
                    .map(|id| (*id, AccountRole::Player))
                    .collect();
            }
            ROLE_SCHEMA_VERSION | SCHEMA_VERSION => {}
            _ => return Err(invalid_data("unsupported account store schema version")),
        }
        let store = Self { data };
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
        if self.data.roles.len() != self.data.accounts.len()
            || self
                .data
                .roles
                .keys()
                .any(|account_id| !self.data.accounts.contains_key(account_id))
        {
            return Err(invalid_data("account role records do not match accounts"));
        }
        if self.data.public_keys.len() != self.data.accounts.len()
            || self
                .data
                .public_keys
                .keys()
                .any(|id| !self.data.accounts.contains_key(id))
        {
            return Err(invalid_data("public key records do not match accounts"));
        }
        if self
            .data
            .sanctions
            .keys()
            .any(|id| !self.data.accounts.contains_key(id))
        {
            return Err(invalid_data("sanction records do not match accounts"));
        }
        let mut all_keys = BTreeSet::new();
        for keys in self.data.public_keys.values() {
            if keys.len() > MAX_PUBLIC_KEYS
                || keys
                    .iter()
                    .any(|key| validate_public_key(key).is_err() || !all_keys.insert(*key))
            {
                return Err(invalid_data("invalid, duplicate or excessive public keys"));
            }
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

/// Load and save duplicate check: lowercase only, not `name_key`, so saves whose names already
/// fold together keep loading and saving.
fn name_keys(identity: &CharacterIdentity) -> BTreeSet<(String, String)> {
    name_pairs(identity)
        .map(|(first, last)| (first.to_lowercase(), last.to_lowercase()))
        .collect()
}

/// The displayed and pending (temporary) name pairs a character holds.
fn name_pairs(identity: &CharacterIdentity) -> impl Iterator<Item = (&str, &str)> {
    [
        (&identity.first_name, &identity.last_name),
        (
            &identity.creation.temporary_first_name,
            &identity.creation.temporary_last_name,
        ),
    ]
    .into_iter()
    .filter(|(first, last)| !first.is_empty() && !last.is_empty())
    .map(|(first, last)| (first.as_str(), last.as_str()))
}

/// Uniqueness key of one name part: Latin letters fold to their base, the rest lowercase, so
/// 'Éva', 'EVA' and 'Eva' are one name. Folding comes before lowercasing because 'İ' lowercases
/// to 'i' plus a combining dot. Saves keep the typed spelling; only comparisons use the key.
fn name_key(part: &str) -> String {
    let mut key = String::with_capacity(part.len());
    for c in part.chars() {
        match LATIN_FOLDS
            .iter()
            .find(|(_, letters)| !c.is_ascii() && letters.contains(c))
        {
            Some((base, _)) => key.push_str(base),
            None => key.extend(c.to_lowercase()),
        }
    }
    key
}

/// (R) Base letters for U+00C0..=U+024F, the range `account::new_name_part` allows: every base
/// letter or ligature with a mark, stroke, hook, bar or tail, plus the look-alikes ð, þ, ß, ĸ, ı,
/// ȷ, ŉ, ŋ and African Ɖ. Letters with no base reading (schwa, ezh, yogh, wynn, open, turned and
/// reversed forms, clicks, tone letters, glottal stops) only lowercase.
const LATIN_FOLDS: [(&str, &str); 38] = [
    ("'n", "ŉ"),
    ("a", "ÀÁÂÃÄÅàáâãäåĀāĂăĄąǍǎǞǟǠǡǺǻȀȁȂȃȦȧȺ"),
    ("ae", "ÆæǢǣǼǽ"),
    ("b", "ƀƁƂƃɃ"),
    ("c", "ÇçĆćĈĉĊċČčƇƈȻȼ"),
    ("d", "ÐðĎďĐđƉƊƋƌȡ"),
    ("db", "ȸ"),
    ("dz", "ǄǅǆǱǲǳ"),
    ("e", "ÈÉÊËèéêëĒēĔĕĖėĘęĚěȄȅȆȇȨȩɆɇ"),
    ("f", "Ƒƒ"),
    ("g", "ĜĝĞğĠġĢģƓǤǥǦǧǴǵ"),
    ("h", "ĤĥĦħȞȟ"),
    ("hv", "ƕǶ"),
    ("i", "ÌÍÎÏìíîïĨĩĪīĬĭĮįİıƗǏǐȈȉȊȋ"),
    ("ij", "Ĳĳ"),
    ("j", "ĴĵǰȷɈɉ"),
    ("k", "ĶķĸƘƙǨǩ"),
    ("l", "ĹĺĻļĽľĿŀŁłƚȴȽ"),
    ("lj", "Ǉǈǉ"),
    ("n", "ÑñŃńŅņŇňŊŋƝƞǸǹȠȵ"),
    ("nj", "Ǌǋǌ"),
    ("o", "ÒÓÔÕÖØòóôõöøŌōŎŏŐőƟƠơǑǒǪǫǬǭǾǿȌȍȎȏȪȫȬȭȮȯȰȱ"),
    ("oe", "Œœ"),
    ("oi", "Ƣƣ"),
    ("ou", "Ȣȣ"),
    ("p", "Ƥƥ"),
    ("q", "Ɋɋ"),
    ("qp", "ȹ"),
    ("r", "ŔŕŖŗŘřȐȑȒȓɌɍ"),
    ("s", "ŚśŜŝŞşŠšſȘșȿ"),
    ("ss", "ß"),
    ("t", "ŢţŤťŦŧƫƬƭƮȚțȶȾ"),
    ("th", "Þþ"),
    ("u", "ÙÚÛÜùúûüŨũŪūŬŭŮůŰűŲųƯưǓǔǕǖǗǘǙǚǛǜȔȕȖȗɄ"),
    ("v", "Ʋ"),
    ("w", "Ŵŵ"),
    ("y", "ÝýÿŶŷŸƳƴȲȳɎɏ"),
    ("z", "ŹźŻżŽžƵƶȤȥɀ"),
];

/// A name as its whitespace-separated words, each folded by `name_key`, however the spaces fall
/// between and inside the parts: what `realm` moderation commands match and new names must keep
/// unique. `name_key` folds one character at a time and never yields whitespace, so folding each
/// word equals folding each part.
fn name_words<'a>(parts: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    parts
        .into_iter()
        .flat_map(str::split_whitespace)
        .map(name_key)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_two_migrates_to_three_without_changing_roles_or_identity() {
        let fixture =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/account/accounts-v2.json");
        let before = std::fs::read(&fixture).unwrap();
        let store = AccountStore::load(&fixture).unwrap();
        assert_eq!(store.account_role(1), Some(AccountRole::Admin));
        assert_eq!(store.public_keys(1), Some([].as_slice()));
        let path =
            std::env::temp_dir().join(format!("osfr-w83-migrate-{}.json", std::process::id()));
        store.save(&path).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["schema_version"], 3);
        assert_eq!(saved["store"]["public_keys"]["1"], serde_json::json!([]));
        assert_eq!(AccountStore::load(&path).unwrap(), store);
        let original: serde_json::Value = serde_json::from_slice(&before).unwrap();
        for field in [
            "accounts",
            "characters",
            "roles",
            "next_account_id",
            "next_character_guid",
        ] {
            assert_eq!(saved["store"][field], original["store"][field]);
        }
        assert_eq!(std::fs::read(fixture).unwrap(), before);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn public_keys_are_valid_unique_bounded_and_persisted() {
        use ed25519_dalek::SigningKey;
        let mut store = AccountStore::new();
        let first = store
            .create_account(
                [1; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 2 },
            )
            .unwrap();
        let second = store
            .create_account(
                [2; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 2 },
            )
            .unwrap();
        let key = SigningKey::from_bytes(&[1; 32]).verifying_key().to_bytes();
        store.add_public_key(first, &key).unwrap();
        assert!(store.add_public_key(first, &key).is_err());
        assert!(store.add_public_key(second, &key).is_err());
        for seed in 2..=MAX_PUBLIC_KEYS as u8 {
            store
                .add_public_key(
                    first,
                    &SigningKey::from_bytes(&[seed; 32])
                        .verifying_key()
                        .to_bytes(),
                )
                .unwrap();
        }
        let before = store.clone();
        let extra = SigningKey::from_bytes(&[99; 32]).verifying_key().to_bytes();
        assert!(store.add_public_key(first, &extra).is_err());
        for bytes in [vec![], vec![0; 31], vec![0; 32], vec![0; 33]] {
            assert!(store.add_public_key(second, &bytes).is_err());
        }
        assert!(store.add_public_key(99, &extra).is_err());
        assert_eq!(store, before);
        let path = std::env::temp_dir().join(format!("osfr-w83-keys-{}.json", std::process::id()));
        store.save(&path).unwrap();
        assert_eq!(AccountStore::load(&path).unwrap(), store);
        store.remove_public_key(first, &key).unwrap();
        assert!(store.remove_public_key(first, &key).is_err());
        assert_eq!(store.public_keys(first).unwrap().len(), MAX_PUBLIC_KEYS - 1);
        store.add_public_key(second, &key).unwrap();
        assert!(store.remove_public_key(99, &key).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn schema_three_fails_closed_on_missing_or_invalid_key_records() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/account/accounts-v2.json"))
                .unwrap();
        let path =
            std::env::temp_dir().join(format!("osfr-w83-invalid-{}.json", std::process::id()));
        for keys in [
            serde_json::json!({}),
            serde_json::json!({"2": []}),
            serde_json::json!({"1": [vec![0u8; 32]]}),
        ] {
            let mut document = fixture.clone();
            document["schema_version"] = serde_json::json!(3);
            document["store"]["public_keys"] = keys;
            let bytes = serde_json::to_vec(&document).unwrap();
            std::fs::write(&path, &bytes).unwrap();
            assert_eq!(
                AccountStore::load(&path).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
        // Schema 2 must not smuggle in key enrolment during migration.
        let mut document = fixture;
        document["store"]["public_keys"] = serde_json::json!({"1": []});
        std::fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(AccountStore::load(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn account_store_lock_excludes_other_writers_without_removing_the_owner() {
        let path = std::env::temp_dir().join(format!("osfr-w83-lock-{}.json", std::process::id()));
        let guard = AccountStoreLock::acquire(&path).unwrap();
        assert!(
            matches!(AccountStoreLock::acquire(&path), Err(error) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        assert!(
            matches!(AccountStoreLock::acquire(&path), Err(error) if error.kind() == io::ErrorKind::AlreadyExists)
        );
        drop(guard);
        drop(AccountStoreLock::acquire(&path).unwrap());
        // L042: a writer waits out another writer's short load-modify-save.
        let guard = AccountStoreLock::acquire(&path).unwrap();
        let holder = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(100));
            drop(guard);
        });
        drop(AccountStoreLock::acquire(&path).unwrap());
        holder.join().unwrap();
    }

    /// L042: a writer refused by a held lock, and a lock its guard fails to remove (which refuses
    /// every later writer), are reported on stderr. They run in a child copy of this test binary,
    /// whose stderr is read here.
    #[test]
    fn a_held_or_unremovable_lock_is_reported() {
        const CHILD: &str = "OSFR_L042_LOCK_LEAK_CHILD";
        if let Some(path) = std::env::var_os(CHILD) {
            let guard = AccountStoreLock::acquire(Path::new(&path)).unwrap();
            let busy = AccountStoreLock::acquire_within(Path::new(&path), Default::default());
            assert!(busy.is_err());
            // Another hand removed the lock file, so the guard's own removal fails.
            std::fs::remove_file(&guard.0).unwrap();
            drop(guard);
            return;
        }
        let path = std::env::temp_dir().join(format!("osfr-l042-leak-{}.json", std::process::id()));
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "account_store::tests::a_held_or_unremovable_lock_is_reported",
            ])
            .arg("--nocapture")
            .env(CHILD, &path)
            .output()
            .unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("is held; request refused"), "{stderr}");
        assert!(stderr.contains("could not be removed"), "{stderr}");
    }
}
