//! Account ownership and pre-game character request handlers.
//!
//! The client proves account membership and maximum character count. Slot
//! defaults, underage/open-chat meaning, name moderation, and delete outcomes
//! are reconstructed policy (R), not original server rules.

use crate::{account_store::AccountStore, character_store::CharacterState};
use osfr_tools::{
    identity::{self, CharacterIdentity, NativeCreation, Resources},
    login,
};
use serde::{Deserialize, Serialize};

pub type LoginNameHash = [u8; 32];

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountFlags {
    /// Recovered account-info field (`ClientAccountManager::OnAccountInfo`).
    pub membership: bool,
    /// (R) The assigned character guide does not expose this account flag.
    pub underage: bool,
    /// (R) The assigned character guide does not expose this account flag.
    pub open_chat: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotPolicy {
    /// (R) Client reads `MaxCharacters`; its original value is not recovered.
    pub max_characters: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub id: u64,
    pub login_name_hash: LoginNameHash,
    pub flags: AccountFlags,
    pub slots: SlotPolicy,
    pub character_guids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterRecord {
    pub guid: u64,
    pub owner_account_id: u64,
    /// Stored with the legacy identity module's local guid sentinel (1).
    pub identity: CharacterIdentity,
    pub world_state: Option<CharacterState>,
}

impl CharacterRecord {
    pub(crate) fn wire_identity(&self) -> CharacterIdentity {
        let mut identity = self.identity.clone();
        identity.guid = self.guid;
        identity
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterCreateStatus {
    Created,
    Failed,
    ServerFailure,
    NameTaken,
    TemporaryNameTaken,
    TooManyCharacters,
    InvalidName,
}

impl CharacterCreateStatus {
    pub const fn client_code(self) -> i32 {
        match self {
            Self::Created => 1,
            Self::Failed => 2,
            // (R) Missing local account context maps to the client's generic failure 3.
            Self::ServerFailure => 3,
            Self::NameTaken => 4,
            Self::TemporaryNameTaken => 5,
            Self::TooManyCharacters => 6,
            Self::InvalidName => 7,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterCreateReply {
    pub status: CharacterCreateStatus,
    pub guid: u64,
    pub message_bytes: Vec<u8>,
}

pub fn create_character(
    store: &mut AccountStore,
    account_id: u64,
    creation: NativeCreation,
    resources: &Resources,
) -> CharacterCreateReply {
    let fail = |status| CharacterCreateReply {
        status,
        guid: 0,
        message_bytes: login::create_reply(status.client_code(), 0),
    };
    let Some(account) = store.account(account_id) else {
        return fail(CharacterCreateStatus::ServerFailure);
    };
    // (R) The maximum is account configuration; original server values are unknown.
    if account.character_guids.len() >= account.slots.max_characters as usize {
        return fail(CharacterCreateStatus::TooManyCharacters);
    }

    let effective_first = if creation.first_name.is_empty() {
        &creation.temporary_first_name
    } else {
        &creation.first_name
    };
    let effective_last = if creation.last_name.is_empty() {
        &creation.temporary_last_name
    } else {
        &creation.last_name
    };
    if !valid_client_name(effective_first, effective_last) {
        return fail(CharacterCreateStatus::InvalidName);
    }

    if let Some((first, last)) = nonempty_pair(&creation.first_name, &creation.last_name) {
        if !valid_client_name(first, last) {
            return fail(CharacterCreateStatus::InvalidName);
        }
        if store.character_name_taken(first, last) {
            return fail(CharacterCreateStatus::NameTaken);
        }
    }
    if let Some((first, last)) = nonempty_pair(
        &creation.temporary_first_name,
        &creation.temporary_last_name,
    ) {
        if !valid_client_name(first, last) {
            return fail(CharacterCreateStatus::InvalidName);
        }
        if store.character_name_taken(first, last) {
            return fail(CharacterCreateStatus::TemporaryNameTaken);
        }
    }

    // Reuse the same resource-backed creator validation called by login.rs.
    let identity = match resources.resolve(creation) {
        Ok(identity) => identity,
        Err(_) => return fail(CharacterCreateStatus::Failed),
    };
    if !valid_client_name(&identity.first_name, &identity.last_name) {
        return fail(CharacterCreateStatus::InvalidName);
    }
    match store.insert_character(account_id, identity, None) {
        Ok(guid) => CharacterCreateReply {
            status: CharacterCreateStatus::Created,
            guid,
            message_bytes: login::create_reply(1, guid),
        },
        Err(_) => fail(CharacterCreateStatus::Failed),
    }
}

fn nonempty_pair<'a>(first: &'a str, last: &'a str) -> Option<(&'a str, &'a str)> {
    (!first.is_empty() && !last.is_empty()).then_some((first, last))
}

fn valid_client_name(first: &str, last: &str) -> bool {
    // (R) Client UI thresholds are enforced locally; original server checks are unknown.
    let first_len = first.chars().count();
    let last_len = last.chars().count();
    (2..=14).contains(&first_len)
        && (2..=14).contains(&last_len)
        && first_len + last_len <= 15
        && identity::valid_name(first, last)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameCheckStatus {
    Available,
    Taken,
    Profane,
    TotalLengthInvalid,
    ServerFailure,
    FirstTooShort,
    LastTooShort,
    FirstTooLong,
    LastTooLong,
    Invalid,
}

impl NameCheckStatus {
    pub const fn client_code(self) -> i32 {
        match self {
            Self::Available => 1,
            Self::Taken => 2,
            Self::Profane => 3,
            Self::TotalLengthInvalid => 5,
            Self::ServerFailure => 6,
            Self::FirstTooShort => 7,
            Self::LastTooShort => 8,
            Self::FirstTooLong => 9,
            Self::LastTooLong => 10,
            Self::Invalid => -1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameSuggestion {
    pub name: String,
    pub last_prefix: String,
    pub last_suffix: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameCheckReply {
    pub status: NameCheckStatus,
    pub suggestions: Vec<NameSuggestion>,
    pub message_bytes: Vec<u8>,
}

pub fn check_name(store: &AccountStore, server_id: i32, first: &str, last: &str) -> NameCheckReply {
    let status = check_name_status(store, first, last);
    let suggestions = Vec::new();
    let mut packet = 211i16.to_le_bytes().to_vec();
    put_string(&mut packet, first);
    put_string(&mut packet, last);
    packet.extend(status.client_code().to_le_bytes());
    let mut message_bytes = vec![17];
    message_bytes.extend(server_id.to_le_bytes());
    put_payload(&mut message_bytes, &packet);
    NameCheckReply {
        status,
        suggestions,
        message_bytes,
    }
}

fn check_name_status(store: &AccountStore, first: &str, last: &str) -> NameCheckStatus {
    let first_len = first.chars().count();
    let last_len = last.chars().count();
    if first_len < 2 {
        NameCheckStatus::FirstTooShort
    } else if last_len < 2 {
        NameCheckStatus::LastTooShort
    } else if first_len > 14 {
        NameCheckStatus::FirstTooLong
    } else if last_len > 14 {
        NameCheckStatus::LastTooLong
    } else if first_len + last_len > 15 {
        NameCheckStatus::TotalLengthInvalid
    } else if !identity::valid_name(first, last) {
        NameCheckStatus::Invalid
    } else if store.character_name_taken(first, last) {
        NameCheckStatus::Taken
    } else {
        NameCheckStatus::Available
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterListEntry {
    pub guid: u64,
    pub server_id: i32,
    pub status: i32,
    pub payload_base64: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterListReply {
    pub status: i32,
    pub can_bypass_server_lock: bool,
    pub characters: Vec<CharacterListEntry>,
    pub message_bytes: Vec<u8>,
}

pub fn character_list(store: &AccountStore, account_id: u64, server_id: i32) -> CharacterListReply {
    let records = store
        .account(account_id)
        .map(|account| {
            account
                .character_guids
                .iter()
                .filter_map(|guid| store.character(*guid))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // (R) Preserve the existing local selection helper's empty/nonempty result values.
    let status: i32 = if records.is_empty() { 2 } else { 1 };
    let mut message_bytes = vec![12];
    message_bytes.extend(status.to_le_bytes());
    message_bytes.push(0);
    message_bytes.extend((records.len() as u32).to_le_bytes());
    let mut characters = Vec::with_capacity(records.len());
    for record in records {
        let identity = record.wire_identity();
        let single = login::selection_reply(Some(&identity));
        let data_len = u32::from_le_bytes(single[26..30].try_into().unwrap()) as usize;
        let data = &single[30..30 + data_len];
        let payload_base64 = base64(data);
        message_bytes.extend(record.guid.to_le_bytes());
        message_bytes.extend(server_id.to_le_bytes());
        // (R) The native reader exposes this field; its server success value is unverified.
        message_bytes.extend(1i32.to_le_bytes());
        put_string(&mut message_bytes, &payload_base64);
        characters.push(CharacterListEntry {
            guid: record.guid,
            server_id,
            status: 1,
            payload_base64,
        });
    }
    CharacterListReply {
        status,
        can_bypass_server_lock: false,
        characters,
        message_bytes,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterDeleteStatus {
    Deleted,
    NotOwnedOrMissing,
}

impl CharacterDeleteStatus {
    pub const fn client_code(self) -> i32 {
        match self {
            // (R) The native reader exposes an integer status; these values
            // mirror the local login helper's failure code and success path.
            Self::Deleted => 1,
            Self::NotOwnedOrMissing => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterDeleteReply {
    pub status: CharacterDeleteStatus,
    pub message_bytes: Vec<u8>,
}

pub fn delete_character(
    store: &mut AccountStore,
    account_id: u64,
    guid: u64,
) -> CharacterDeleteReply {
    // (R) Enforce account ownership and use local success/failure status values.
    let status = if store.remove_character(account_id, guid) {
        CharacterDeleteStatus::Deleted
    } else {
        CharacterDeleteStatus::NotOwnedOrMissing
    };
    let mut message_bytes = vec![10];
    message_bytes.extend(guid.to_le_bytes());
    message_bytes.extend(status.client_code().to_le_bytes());
    put_payload(&mut message_bytes, &[]);
    CharacterDeleteReply {
        status,
        message_bytes,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterLoginStatus {
    Success,
    AccountInUse,
    UnableToLoadCharacter,
    UnableToAssignGateway,
}

impl CharacterLoginStatus {
    pub const fn client_code(self) -> i32 {
        match self {
            Self::Success => 1,
            Self::AccountInUse => 2,
            Self::UnableToLoadCharacter => 6,
            Self::UnableToAssignGateway => 7,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthenticatedSession {
    account_id: u64,
    selected_character: Option<u64>,
}

impl AuthenticatedSession {
    pub fn new(account_id: u64) -> Self {
        Self {
            account_id,
            selected_character: None,
        }
    }

    pub fn account_id(&self) -> u64 {
        self.account_id
    }

    pub fn selected_character(&self) -> Option<u64> {
        self.selected_character
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterLoginReply {
    pub status: CharacterLoginStatus,
    pub message_bytes: Vec<u8>,
}

pub fn select_and_bind_session(
    store: &AccountStore,
    session: &mut AuthenticatedSession,
    guid: u64,
    server_id: i32,
    gateway_address: &str,
    ticket: &str,
) -> CharacterLoginReply {
    // (R) Bind only after a server-side ownership check; client GUIDs are untrusted.
    let owns_character = store
        .character(guid)
        .is_some_and(|character| character.owner_account_id == session.account_id);
    let status = if owns_character {
        session.selected_character = Some(guid);
        CharacterLoginStatus::Success
    } else {
        CharacterLoginStatus::UnableToLoadCharacter
    };
    CharacterLoginReply {
        status,
        message_bytes: login::character_login_reply(
            guid,
            server_id,
            status.client_code(),
            gateway_address,
            ticket,
        ),
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value.as_bytes());
}

fn put_payload(out: &mut Vec<u8>, payload: &[u8]) {
    out.extend((payload.len() as u32).to_le_bytes());
    out.extend(payload);
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        out.push(TABLE[usize::from(a >> 2)] as char);
        out.push(TABLE[usize::from(((a & 3) << 4) | (b >> 4))] as char);
        out.push(if chunk.len() > 1 {
            TABLE[usize::from(((b & 15) << 2) | (c >> 6))] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[usize::from(c & 63)] as char
        } else {
            '='
        });
    }
    out
}
