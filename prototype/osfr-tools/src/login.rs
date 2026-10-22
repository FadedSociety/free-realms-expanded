//! Minimal loopback external-login protocol (LoginUdp_6). Native packet layouts
//! are checked against pinned community serializers. The account identity is a
//! SHA-256 hash of the accepted local login name; this is not external auth.
use crate::{
    account::{
        self, AccountFlags, AuthenticatedSession, CharacterCreateStatus, CharacterLoginStatus,
        SlotPolicy,
    },
    account_store::AccountStore,
    character_store::CharacterState,
    game_rules::{GameRuleSet, MembershipState, RulePolicy},
    identity::{
        self, CharacterIdentity, NativeCreation, Reader, Resources, invalid, payload, string,
    },
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

pub const MAX_LOGIN_MESSAGE: usize = 64 * 1024;
/// Synthetic local token, not authentication. Native CCM initialization uses
/// ASCII ticket bytes, so this default has a valid AES key length.
pub const DEFAULT_GATEWAY_TICKET: &str = "00000000000000000000000000000001";
static MIGRATION_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Continuous RC4 state, independent in each direction. Ciphertext is queued
/// once: retransmission must replay stored ciphertext without advancing RC4.
#[derive(Clone)]
pub struct Rc4 {
    permutation: [u8; 256],
    i: u8,
    j: u8,
}
impl Rc4 {
    pub fn new(key: &[u8]) -> io::Result<Self> {
        if key.is_empty() || key.len() > 256 {
            return Err(invalid("invalid private login key size"));
        }
        let mut permutation = [0; 256];
        for (i, x) in permutation.iter_mut().enumerate() {
            *x = i as u8;
        }
        let mut j = 0u8;
        for i in 0..256 {
            j = j
                .wrapping_add(permutation[i])
                .wrapping_add(key[i % key.len()]);
            permutation.swap(i, usize::from(j));
        }
        Ok(Self {
            permutation,
            i: 0,
            j: 0,
        })
    }
    pub fn apply(&mut self, bytes: &mut [u8]) {
        for b in bytes {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.permutation[usize::from(self.i)]);
            self.permutation
                .swap(usize::from(self.i), usize::from(self.j));
            let k = self.permutation[usize::from(self.i)]
                .wrapping_add(self.permutation[usize::from(self.j)]);
            *b ^= self.permutation[usize::from(k)];
        }
    }
}

/// SOE zero-escape/group processing happens BEFORE application RC4 decryption.
/// An encrypted first byte of zero receives one extra zero on transmission.
pub fn frame_ciphertext(bytes: &[u8]) -> io::Result<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > MAX_LOGIN_MESSAGE {
        return Err(invalid("invalid encrypted login length"));
    }
    let mut out = Vec::with_capacity(bytes.len() + 1);
    if bytes[0] == 0 {
        out.push(0);
    }
    out.extend(bytes);
    Ok(out)
}

pub fn ciphertext_messages(bytes: &[u8]) -> io::Result<Vec<&[u8]>> {
    if bytes.is_empty() || bytes.len() > MAX_LOGIN_MESSAGE + 1 {
        return Err(invalid("invalid login frame length"));
    }
    if bytes[0] != 0 {
        return Ok(vec![bytes]);
    }
    if bytes.get(1) == Some(&0) {
        return Ok(vec![&bytes[1..]]);
    }
    if !bytes.starts_with(&[0, 25]) {
        return Err(invalid("unsupported login application control"));
    }
    let mut r = Reader::new(&bytes[2..]);
    let mut out = Vec::new();
    while r.offset < bytes.len() - 2 {
        if out.len() >= 64 {
            return Err(invalid("too many login group children"));
        }
        let (n, prefix) = crate::soe::variable_length(&bytes[2 + r.offset..])
            .ok_or_else(|| invalid("truncated login group child length"))?;
        r.take(prefix)?;
        if n == 0 || n > MAX_LOGIN_MESSAGE + 1 {
            return Err(invalid("invalid login group child length"));
        }
        let child = r.take(n)?;
        if child[0] == 0 {
            if child.get(1) != Some(&0) {
                return Err(invalid("nested login groups unsupported"));
            }
            out.push(&child[1..]);
        } else {
            out.push(child);
        }
    }
    if out.is_empty() {
        return Err(invalid("empty login group"));
    }
    Ok(out)
}

pub struct LoginConfig {
    pub identity_file: PathBuf,
    pub legacy_save_file: PathBuf,
    pub account_store_file: PathBuf,
    pub gateway_address: String,
    /// Local login name accepted by this service; it is not a web account.
    pub local_session: String,
    pub gateway_ticket: String,
    /// Private shared key for login-to-gateway character-binding tickets.
    pub handoff_key: Vec<u8>,
    /// (R) Configured local slot maximum. Default is six.
    pub max_characters: u32,
    /// (R) Account chat policy; original semantics are unknown.
    pub underage: bool,
    /// (R) Account chat policy; original semantics are unknown.
    pub open_chat: bool,
}

pub struct LoginService {
    resources: Resources,
    config: LoginConfig,
    logged_in: bool,
    account_store: Option<AccountStore>,
    account_id: Option<u64>,
    authenticated_session: Option<AuthenticatedSession>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterBinding {
    pub account_id: u64,
    pub character_guid: u64,
}
#[derive(Debug)]
pub struct LoginOutcome {
    pub replies: Vec<Vec<u8>>,
    pub created: bool,
    pub selected: bool,
    /// Trusted server-side binding issued with the gateway ticket.
    pub selected_binding: Option<CharacterBinding>,
    /// Validation reason only; never include names, payloads, tickets or keys.
    pub creation_rejection: Option<String>,
}

impl LoginService {
    pub fn new(resources: Resources, config: LoginConfig) -> io::Result<Self> {
        identity::load(&config.identity_file)?;
        let address: std::net::SocketAddr = config.gateway_address.parse().map_err(invalid)?;
        // L053: loopback, or a private-network/VPN gateway (Security Model decision 2); never public.
        // The osfr-login binary admits a private one only with --allow-lan.
        if !crate::net_policy::allowed(address.ip(), true)
            || config.local_session.is_empty()
            || config.local_session.len() > 1024
            || !config.gateway_ticket.is_ascii()
            || !matches!(config.gateway_ticket.len(), 16 | 24 | 32)
            || config.handoff_key.len() < 16
            || config.max_characters == 0
            || config.max_characters > i32::MAX as u32
        {
            return Err(invalid("invalid local login configuration"));
        }
        Ok(Self {
            resources,
            config,
            logged_in: false,
            account_store: None,
            account_id: None,
            authenticated_session: None,
        })
    }
    pub fn reset_connection(&mut self) {
        self.logged_in = false;
        self.account_store = None;
        self.account_id = None;
        self.authenticated_session = None;
    }

    fn authenticate_account(&mut self, login_name: &str) -> io::Result<u64> {
        let login_name_hash: [u8; 32] = Sha256::digest(login_name.as_bytes()).into();
        let flags = AccountFlags {
            membership: false,
            underage: self.config.underage,
            open_chat: self.config.open_chat,
        };
        let slots = SlotPolicy {
            max_characters: self.config.max_characters,
        };
        let store_path = &self.config.account_store_file;
        let mut store = match fs::File::open(store_path) {
            Ok(_) => AccountStore::load(store_path)?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let has_save = self.config.legacy_save_file.is_file();
                let has_identity = self.config.identity_file.is_file();
                if has_save && has_identity {
                    migrate_legacy_account_store(
                        store_path,
                        &self.config.legacy_save_file,
                        &self.config.identity_file,
                        login_name_hash,
                        flags.clone(),
                        slots,
                    )?
                } else {
                    if has_save {
                        return Err(invalid("legacy character save has no identity"));
                    }
                    let mut store = AccountStore::new();
                    let account_id = store.create_account(login_name_hash, flags.clone(), slots)?;
                    if has_identity {
                        let identity = identity::load(&self.config.identity_file)?
                            .ok_or_else(|| invalid("legacy character identity is missing"))?;
                        store.insert_character(account_id, identity, None)?;
                    }
                    store.save(store_path)?;
                    store
                }
            }
            Err(error) => return Err(error),
        };
        let account_id = match store.account_id_by_login_hash(&login_name_hash) {
            Some(id) => id,
            None => {
                let id = store.create_account(login_name_hash, flags, slots)?;
                store.save(store_path)?;
                id
            }
        };
        self.account_store = Some(store);
        self.account_id = Some(account_id);
        self.authenticated_session = Some(AuthenticatedSession::new(account_id));
        Ok(account_id)
    }

    fn store_and_account(&self) -> io::Result<(&AccountStore, u64)> {
        let store = self
            .account_store
            .as_ref()
            .ok_or_else(|| invalid("authenticated account is unavailable"))?;
        let account_id = self
            .account_id
            .ok_or_else(|| invalid("authenticated account is unavailable"))?;
        Ok((store, account_id))
    }

    /// L055: create and delete save the whole store, so they start from the file, never the copy
    /// cached at login: another session on the same file (every unauthenticated loopback peer
    /// shares one) may have saved since. Re-runs the account lookup here; the caller's
    /// create/delete re-run slot, name and ownership checks on this copy. osfr-login handles one
    /// request at a time, so nothing in the process saves between this load and the caller's save.
    fn reload_account_store(&mut self) -> io::Result<()> {
        let (_, account_id) = self.store_and_account()?;
        let store = AccountStore::load(&self.config.account_store_file)?;
        let login_name_hash: [u8; 32] = Sha256::digest(self.config.local_session.as_bytes()).into();
        if store.account_id_by_login_hash(&login_name_hash) != Some(account_id) {
            return Err(invalid("authenticated account is unavailable"));
        }
        self.account_store = Some(store);
        Ok(())
    }

    fn character_list_reply(&self, server_id: i32) -> io::Result<Vec<u8>> {
        let (store, account_id) = self.store_and_account()?;
        let reply = account::character_list(store, account_id, server_id);
        let mut bytes = vec![12];
        bytes.extend(reply.status.to_le_bytes());
        bytes.push(u8::from(reply.can_bypass_server_lock));
        bytes.extend((reply.characters.len() as u32).to_le_bytes());
        for entry in reply.characters {
            let record = store
                .character(entry.guid)
                .ok_or_else(|| invalid("listed character is missing"))?;
            bytes.extend(entry.guid.to_le_bytes());
            bytes.extend(entry.server_id.to_le_bytes());
            bytes.extend(entry.status.to_le_bytes());
            // The row payload is the appearance record the native character
            // select renders from (accepted natively before W59); see
            // `selection_reply`. A base64 name here drops the model.
            payload(&mut bytes, &appearance_payload(&record.identity));
        }
        Ok(bytes)
    }
    pub fn handle(&mut self, bytes: &[u8]) -> io::Result<LoginOutcome> {
        if bytes.len() > MAX_LOGIN_MESSAGE {
            return Err(invalid("login application exceeds bound"));
        }
        let mut r = Reader::new(bytes);
        let opcode = r.u8()?;
        let mut outcome = LoginOutcome {
            replies: Vec::new(),
            created: false,
            selected: false,
            selected_binding: None,
            creation_rejection: None,
        };
        if opcode != 1 && !self.logged_in {
            return Err(invalid("login required before character requests"));
        }
        match opcode {
            1 => {
                let login_name = r.string(1024)?;
                r.string(4096)?;
                r.end()?;
                self.logged_in = login_name == self.config.local_session;
                if self.logged_in {
                    self.authenticate_account(&login_name)?;
                    let (store, account_id) = self.store_and_account()?;
                    let account = store
                        .account(account_id)
                        .ok_or_else(|| invalid("authenticated account is unavailable"))?;
                    outcome.replies.push(login_reply_for_account(
                        true,
                        account.flags.membership,
                        account.slots.max_characters,
                    ));
                } else {
                    self.reset_connection();
                    outcome.replies.push(login_reply(false));
                }
            }
            11 => {
                r.end()?;
                outcome.replies.push(self.character_list_reply(1)?);
            }
            13 => {
                r.end()?;
                outcome.replies.push(server_list_reply());
            }
            5 => {
                let server = r.i32()?;
                let data = r.payload(4096)?;
                r.end()?;
                let creation = NativeCreation::decode(data)?;
                self.reload_account_store()?;
                let (store, account_id) = self.store_and_account()?;
                if !matches!(server, 0 | 1) {
                    outcome.creation_rejection = Some("unsupported local server".into());
                    outcome.replies.push(create_reply(2, 0));
                } else {
                    let mut staged = store.clone();
                    let reply = account::create_character(
                        &mut staged,
                        account_id,
                        creation,
                        &self.resources,
                    );
                    if reply.status == CharacterCreateStatus::Created {
                        staged.save(&self.config.account_store_file)?;
                        if account_id == 1 && reply.guid == 1 {
                            let record = staged.character(1).unwrap();
                            identity::save(&self.config.identity_file, &record.identity)?;
                        }
                        self.account_store = Some(staged);
                        outcome.created = true;
                    } else if reply.status == CharacterCreateStatus::Failed {
                        outcome.creation_rejection = Some("character creation failed".into());
                    }
                    outcome.replies.push(reply.message_bytes);
                }
            }
            7 => {
                let guid = r.u64()?;
                let server = r.i32()?;
                let p = r.payload(4096)?;
                r.end()?;
                let mut client = Reader::new(p);
                client.string(128)?;
                client.i32()?;
                if client.u8()? > 1 {
                    return Err(invalid("invalid client login boolean"));
                }
                client.u8()?;
                client.take(8)?;
                client.end()?;
                let (store, account_id) = self.store_and_account()?;
                let mut authenticated = self
                    .authenticated_session
                    .clone()
                    .ok_or_else(|| invalid("authenticated session is unavailable"))?;
                let reply = if matches!(server, 0 | 1) {
                    account::select_and_bind_session(
                        store,
                        &mut authenticated,
                        guid,
                        server,
                        &self.config.gateway_address,
                        &self.config.gateway_ticket,
                    )
                } else {
                    account::CharacterLoginReply {
                        status: CharacterLoginStatus::UnableToAssignGateway,
                        message_bytes: character_login_reply(
                            guid,
                            server,
                            CharacterLoginStatus::UnableToAssignGateway.client_code(),
                            &self.config.gateway_address,
                            &self.config.gateway_ticket,
                        ),
                    }
                };
                if reply.status == CharacterLoginStatus::Success {
                    let account = store
                        .account(account_id)
                        .ok_or_else(|| invalid("authenticated account is unavailable"))?;
                    // L023: every account, the host's 1/1 included, gets an HMAC binding ticket;
                    // the public fixed ticket would let anyone who reaches the gateway in as 1/1.
                    let ticket = gateway_binding_ticket(&self.config.handoff_key, account, guid);
                    self.authenticated_session = Some(authenticated);
                    outcome.selected = true;
                    outcome.selected_binding = Some(CharacterBinding {
                        account_id,
                        character_guid: guid,
                    });
                    outcome.replies.push(character_login_reply(
                        guid,
                        server,
                        reply.status.client_code(),
                        &self.config.gateway_address,
                        &ticket,
                    ));
                } else {
                    outcome.replies.push(reply.message_bytes);
                }
            }
            16 => {
                let server = r.i32()?;
                let p = r.payload(4096)?;
                r.end()?;
                let mut inner = Reader::new(p);
                if inner.take(2)? != 210i16.to_le_bytes() {
                    return Err(invalid("unsupported tunneled login request"));
                }
                let first = inner.string(128)?;
                let last = inner.string(128)?;
                for _ in 0..3 {
                    inner.string(512)?;
                }
                inner.end()?;
                let (store, _) = self.store_and_account()?;
                let mut reply = account::check_name(store, server, &first, &last);
                if reply.status == account::NameCheckStatus::Invalid {
                    // (R) Preserve the legacy login service's invalid-name code;
                    // the client handles 4 and -1 with the same error text.
                    let end = reply.message_bytes.len();
                    reply.message_bytes[end - 4..].copy_from_slice(&4i32.to_le_bytes());
                }
                outcome.replies.push(reply.message_bytes);
            }
            9 => {
                let guid = r.u64()?;
                r.end()?;
                self.reload_account_store()?;
                let (store, account_id) = self.store_and_account()?;
                let mut staged = store.clone();
                let reply = account::delete_character(&mut staged, account_id, guid);
                if reply.status == account::CharacterDeleteStatus::Deleted {
                    staged.save(&self.config.account_store_file)?;
                    self.account_store = Some(staged);
                    if self
                        .authenticated_session
                        .as_ref()
                        .is_some_and(|session| session.selected_character() == Some(guid))
                    {
                        self.authenticated_session = Some(AuthenticatedSession::new(account_id));
                        outcome.selected = false;
                    }
                }
                outcome.replies.push(reply.message_bytes);
            }
            _ => return Err(invalid("unsupported external login application opcode")),
        }
        Ok(outcome)
    }
}

fn migrate_legacy_account_store(
    store_path: &std::path::Path,
    legacy_save_path: &std::path::Path,
    identity_path: &std::path::Path,
    login_name_hash: [u8; 32],
    flags: AccountFlags,
    slots: SlotPolicy,
) -> io::Result<AccountStore> {
    // The account migrator accepts CharacterState-only v1/v2 documents. Current
    // gateway saves also carry gameplay fields, so project only CharacterState
    // into a private temporary v1 document; both source files remain unchanged.
    let bytes = fs::read(legacy_save_path)?;
    if bytes.len() > 1024 * 1024 {
        return Err(invalid("legacy character save exceeds migration bound"));
    }
    let document: serde_json::Value = serde_json::from_slice(&bytes).map_err(invalid)?;
    let version = document
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| document.get("version").and_then(serde_json::Value::as_u64));
    if !matches!(version, Some(1 | 2 | 5 | 6)) {
        return Err(invalid("unsupported legacy character save version"));
    }
    let state: CharacterState = serde_json::from_value(
        document
            .get("character")
            .cloned()
            .ok_or_else(|| invalid("legacy save has no character state"))?,
    )
    .map_err(invalid)?;
    state.validate()?;
    let sequence = MIGRATION_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let projected_path = std::env::temp_dir().join(format!(
        "osfr-login-migration-{}-{sequence}.json",
        std::process::id()
    ));
    let projected = serde_json::to_vec(&serde_json::json!({
        "version": 1,
        "character": state,
    }))
    .map_err(invalid)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&projected_path)?;
    file.write_all(&projected)?;
    file.sync_all()?;
    drop(file);
    let migrated = AccountStore::load_or_migrate(
        store_path,
        &projected_path,
        identity_path,
        login_name_hash,
        flags,
        slots,
    );
    let cleanup = fs::remove_file(&projected_path);
    match (migrated, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(store), Ok(())) => Ok(store),
    }
}

/// Ticket binds the login-authenticated account and selected character without
/// sending an account id as authority in the client login packet.
pub fn gateway_binding_ticket(
    key: &[u8],
    account: &crate::account::Account,
    character_guid: u64,
) -> String {
    let mut message = b"osfr-login-binding-v1\0".to_vec();
    message.extend(account.id.to_le_bytes());
    message.extend(character_guid.to_le_bytes());
    message.extend(account.login_name_hash);
    let mac = hmac_sha256(key, &message);
    mac[..16].iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn verify_gateway_binding_ticket(
    key: &[u8],
    account: &crate::account::Account,
    character_guid: u64,
    ticket: &str,
) -> bool {
    if key.len() < 16 || ticket.len() != 32 {
        return false;
    }
    let expected = gateway_binding_ticket(key, account, character_guid);
    expected
        .as_bytes()
        .iter()
        .zip(ticket.as_bytes())
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut normalized = [0u8; BLOCK];
    if key.len() > BLOCK {
        normalized[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        normalized[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36; BLOCK];
    let mut outer_pad = [0x5c; BLOCK];
    for index in 0..BLOCK {
        inner_pad[index] ^= normalized[index];
        outer_pad[index] ^= normalized[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(message);
    let digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(digest);
    outer.finalize().into()
}

pub fn login_reply(success: bool) -> Vec<u8> {
    login_reply_fields(success, false, 1)
}

fn login_reply_for_account(success: bool, membership: bool, max_characters: u32) -> Vec<u8> {
    login_reply_fields(success, membership, max_characters as i32)
}

fn login_reply_fields(success: bool, membership: bool, max_characters: i32) -> Vec<u8> {
    let mut out = vec![2, u8::from(success)];
    out.extend((if success { 1i32 } else { 0i32 }).to_le_bytes());
    out.push(0);
    // The callback values use account state. (R) No admin account flag is stored locally.
    let rules = GameRuleSet::new(
        MembershipState::new(membership, None),
        RulePolicy::default(),
    );
    let callback = rules.account_info_callback_bytes(max_characters.max(0) as u32, false, 0);
    let mut account = vec![u8::from(
        u32::from_le_bytes(callback[..4].try_into().unwrap()) != 0,
    )];
    account.extend_from_slice(&callback[4..8]);
    account.push(u8::from(
        u32::from_le_bytes(callback[8..12].try_into().unwrap()) != 0,
    ));
    payload(&mut out, &account);
    out
}

pub fn create_reply(result: i32, guid: u64) -> Vec<u8> {
    let mut out = vec![6];
    out.extend(result.to_le_bytes());
    out.extend(guid.to_le_bytes());
    out
}

pub fn selection_reply(identity: Option<&CharacterIdentity>) -> Vec<u8> {
    let mut out = vec![12];
    out.extend((if identity.is_some() { 1i32 } else { 2i32 }).to_le_bytes());
    out.push(0);
    out.extend((if identity.is_some() { 1i32 } else { 0i32 }).to_le_bytes());
    if let Some(i) = identity {
        let data = appearance_payload(i);
        out.extend(i.guid.to_le_bytes());
        out.extend(1i32.to_le_bytes());
        out.extend(1i32.to_le_bytes());
        payload(&mut out, &data);
    }
    out
}

fn appearance_payload(i: &CharacterIdentity) -> Vec<u8> {
    let mut data = Vec::new();
    string(&mut data, "");
    data.extend(i.guid.to_le_bytes());
    string(&mut data, &i.display_name());
    data.extend(i.model.to_le_bytes());
    for s in [
        &i.head,
        &i.hair,
        &i.model_customization,
        &i.face_paint,
        &i.skin_tone,
    ] {
        string(&mut data, s);
    }
    data.extend(i.eye_color.to_le_bytes());
    data.extend(i.hair_color.to_le_bytes());
    data.extend((i.attachments.len() as u32).to_le_bytes());
    for attachment in &i.attachments {
        attachment.encode(&mut data);
    }
    data
}

pub fn character_login_reply(
    guid: u64,
    server: i32,
    status: i32,
    address: &str,
    ticket: &str,
) -> Vec<u8> {
    let mut out = vec![8];
    out.extend(guid.to_le_bytes());
    out.extend(server.to_le_bytes());
    out.extend(status.to_le_bytes());
    let mut data = Vec::new();
    if status == 1 {
        string(&mut data, address);
        string(&mut data, ticket);
        string(&mut data, ticket);
        data.extend(guid.to_le_bytes());
        data.extend(0i64.to_le_bytes());
        for _ in 0..3 {
            string(&mut data, "");
        }
    }
    payload(&mut out, &data);
    out
}

pub fn server_list_reply() -> Vec<u8> {
    let mut out = vec![14];
    out.extend(1i32.to_le_bytes());
    out.extend(1i32.to_le_bytes());
    out.push(0);
    string(&mut out, "Local Rust Realm");
    out.extend(0i32.to_le_bytes());
    string(&mut out, "");
    out.extend(0i32.to_le_bytes());
    out.extend(0i32.to_le_bytes());
    string(&mut out, "");
    out.extend(0i32.to_le_bytes());
    string(&mut out, "");
    out.push(1);
    out
}

#[cfg(test)]
mod binding_ticket_tests {
    use super::hmac_sha256;

    #[test]
    fn handoff_mac_matches_rfc4231_sha256_case_one() {
        let mac = hmac_sha256(&[0x0b; 20], b"Hi There");
        let encoded: String = mac.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(
            encoded,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }
}
