//! One local character and zone, using private reference initialization data.
//! Rust owns the session, reliable transport, application dispatch and snapshot.
#[path = "../trace.rs"]
mod trace;

use osfr_tools::account_store::AccountStore;
use osfr_tools::character_store::CharacterState;
use osfr_tools::game_rules::{GameRuleSet, MembershipState, RulePolicy};
use osfr_tools::game_store::{load_or_create, save};
use osfr_tools::gameplay::Gameplay;
use osfr_tools::gateway::{Gateway, GatewayOutcome};
use osfr_tools::login;
use osfr_tools::players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance};
use osfr_tools::session::{CRC_SEED, SessionRequest, clock_reflect, decode_controls};
use osfr_tools::social_chat::{
    self, AccountChatPermissions, ChatHistory, ChatLine, ChatSession, NameData, QuickChatCatalog,
    QuickChatRequest, SpamGate,
};
use osfr_tools::social_lists::{
    CharacterIdentity as SocialIdentity, CharacterLookup, SocialLists, SocialMessage, SocialName,
    SocialService,
};
use osfr_tools::soe::{
    CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
};
use osfr_tools::soe_send::{ReliableSender, SendError, SenderConfig};
use osfr_tools::studio;

/// Studio savestates sit next to the character save.
fn savestate_dir(state_path: &std::path::Path) -> PathBuf {
    state_path
        .parent()
        .map(|p| p.join("savestates"))
        .unwrap_or_else(|| PathBuf::from("savestates"))
}

/// Percent-decoding for Studio query strings and paths.
fn studio_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                match u8::from_str_radix(
                    std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("zz"),
                    16,
                ) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                        continue;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b'+' => out.push(b' '),
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Every .json file under `dir`, as paths relative to `root` with '/'.
fn collect_json_files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// The Studio page (notes/Design/Studio Toolkit.md).
const STUDIO_PAGE: &str = include_str!("../../studio/index.html");
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs, io,
    net::{SocketAddr, TcpListener, UdpSocket},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_MESSAGE: usize = 16 * 1024 * 1024;

fn event(value: serde_json::Value) {
    println!("{value}");
}

fn trace_transport(
    trace: &trace::Trace,
    session: Option<u64>,
    peer: SocketAddr,
    direction: &str,
    opcode: u16,
    payload: &[u8],
    completed_messages: Option<usize>,
    outcome: &str,
    reason: Option<&str>,
) -> io::Result<()> {
    let reliable = matches!(opcode, RELIABLE_DATA | RELIABLE_FRAGMENT);
    let sequence = reliable
        .then(|| payload.get(..2))
        .flatten()
        .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
    let message = if reliable && payload.len() >= 2 {
        &payload[2..]
    } else {
        payload
    };
    let decoded = if opcode == RELIABLE_DATA {
        Some(trace::protocol_summary(message))
    } else {
        None
    };
    let combined = decoded.as_ref().is_some_and(|summary| summary.combined)
        || completed_messages.is_some_and(|count| count > 1);
    let mode = if opcode == RELIABLE_FRAGMENT {
        "fragment"
    } else if combined {
        "combined"
    } else if reliable {
        "complete"
    } else {
        "control"
    };
    let summary = decoded;
    let mut value = json!({
        "event":"packet",
        "session":session,
        "peer":peer.to_string(),
        "direction":direction,
        "transport":{
            "soe_opcode":opcode,
            "soe_name":trace::transport_name(opcode),
            "reliable_seq":sequence,
            "mode":mode
        },
        "outcome":outcome,
        "reason":reason
    });
    if let Some(object) = value.as_object_mut() {
        if let Some(summary) = summary {
            object.insert(
                "tunnel_outer_opcode".into(),
                json!(summary.tunnel_outer_opcode),
            );
            object.insert("family".into(), json!(summary.family));
            object.insert("subtype".into(), json!(summary.subtype));
            object.insert("family_name".into(), json!(summary.family_name));
            object.insert("body_len".into(), json!(summary.body.len()));
            object.insert(
                "body_head_hex".into(),
                json!(trace::hex_prefix(
                    summary.body,
                    summary.sensitive || summary.combined || opcode == RELIABLE_FRAGMENT
                )),
            );
        } else {
            object.insert(
                "body_len".into(),
                json!(if reliable {
                    message.len()
                } else {
                    payload.len()
                }),
            );
            object.insert("body_head_hex".into(), json!(None::<String>));
        }
    }
    trace.record(value)
}

fn trace_gateway_message(
    trace: &trace::Trace,
    session: u64,
    peer: SocketAddr,
    transport: Option<(u16, Option<u16>, &'static str, bool)>,
    detail: &osfr_tools::gateway::GatewayTraceEvent,
) -> io::Result<()> {
    trace.record(json!({
        "event":detail.event,
        "session":session,
        "peer":peer.to_string(),
        "direction":"in",
        "transport":transport.map(|(opcode, sequence, mode, _)| json!({
            "soe_opcode":opcode,
            "soe_name":trace::transport_name(opcode),
            "reliable_seq":sequence,
            "mode":if detail.combined { "combined" } else { mode }
        })),
        "tunnel_outer_opcode":detail.tunnel_outer_opcode,
        "family":detail.family,
        "subtype":detail.subtype,
        "family_name":detail.family.zip(detail.subtype).and_then(|(f,s)| trace::family_subtype_name(f,s)),
        "body_len":detail.body_len,
        "body_head_hex":detail.body_head_hex,
        "handler":detail.handler,
        "outcome":detail.outcome,
        "reason":detail.reason,
        "movement":detail.movement
    }))
}

fn trace_application_out(
    trace: &trace::Trace,
    session: u64,
    peer: SocketAddr,
    message: &[u8],
    cause: &[serde_json::Value],
    recipient: Option<u64>,
) -> io::Result<()> {
    let summary = trace::protocol_summary(message);
    trace.record(json!({
        "event":"app_out",
        "session":session,
        "peer":peer.to_string(),
        "direction":"out",
        "recipient":recipient,
        "tunnel_outer_opcode":summary.tunnel_outer_opcode,
        "family":summary.family,
        "subtype":summary.subtype,
        "family_name":summary.family.zip(summary.subtype).and_then(|(f,s)| trace::family_subtype_name(f,s)),
        "body_len":summary.body.len(),
        "body_head_hex":trace::hex_prefix(summary.body, summary.sensitive),
        "cause":cause,
        "outcome":"queued"
    }))
}

fn trace_gateway_rejected(
    trace: &trace::Trace,
    session: u64,
    peer: SocketAddr,
    transport: Option<(u16, Option<u16>, &'static str, bool)>,
    message: &[u8],
    handler: &str,
    reason: &str,
) -> io::Result<()> {
    let summary = trace::protocol_summary(message);
    trace.record(json!({
        "event":"packet","session":session,"peer":peer.to_string(),"direction":"in",
        "transport":transport.map(|(opcode, sequence, mode, _)| json!({
            "soe_opcode":opcode,"soe_name":trace::transport_name(opcode),"reliable_seq":sequence,"mode":mode
        })),
        "tunnel_outer_opcode":summary.tunnel_outer_opcode,
        "family":summary.family,"subtype":summary.subtype,
        "family_name":summary.family.zip(summary.subtype).and_then(|(f,s)| trace::family_subtype_name(f,s)),
        "body_len":summary.body.len(),
        "body_head_hex":trace::hex_prefix(summary.body, summary.sensitive || summary.combined),
        "handler":handler,"outcome":"rejected","reason":reason
    }))
}

pub struct Options {
    pub bind: SocketAddr,
    pub reference: PathBuf,
    pub state: PathBuf,
    pub content: Option<PathBuf>,
    pub identity: Option<PathBuf>,
    pub account_store: Option<PathBuf>,
    pub gateway_ticket: String,
    pub handoff_key: Vec<u8>,
    pub seconds: u64,
    pub dev_tools: bool,
    /// Studio tooling port on 127.0.0.1 (dev tools only; 0 disables).
    pub studio_port: u16,
    /// The game client's Logs folder, shown in Studio's client log panel.
    pub client_logs: Option<PathBuf>,
    /// Ground-height grids (`zone_<id>.hgt` from osfr-terrain).
    pub terrain: Option<Arc<osfr_tools::terrain::Terrain>>,
    /// World Builder live layer: placed NPCs, marks, authored content.
    pub world: Option<PathBuf>,
    /// Row keys of the stock client locale, so authored text ids avoid them.
    pub stock_text: Option<Arc<std::collections::HashSet<u32>>>,
    /// Accept a private-network bind and peers (Security Model L1).
    pub allow_lan: bool,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut args = env::args().skip(1);
        let bind: SocketAddr = args
            .next()
            .ok_or("usage: osfr-gateway 127.0.0.1:PORT --reference DIR --state FILE [--content DIR] [--identity FILE] [--account-store FILE] [--seconds N] [--trace FILE] [--dev-tools]")?
            .parse()?;
        let (mut reference, mut state, mut seconds, mut content, mut identity, mut account_store) =
            (None, None, 0, None, None, None);
        let (mut gateway_ticket, mut handoff_key_file) =
            (login::DEFAULT_GATEWAY_TICKET.to_owned(), None);
        let mut dev_tools = false;
        let mut allow_lan = false;
        let mut studio_port = 47_800;
        let mut client_logs = None;
        let mut terrain = None;
        let mut world = None;
        let mut stock_text = None;
        while let Some(key) = args.next() {
            if key == "--dev-tools" {
                dev_tools = true;
                continue;
            }
            if key == "--allow-lan" {
                allow_lan = true;
                continue;
            }
            let value = args.next().ok_or("missing option value")?;
            match key.as_str() {
                "--reference" => reference = Some(PathBuf::from(value)),
                "--state" => state = Some(PathBuf::from(value)),
                "--content" => content = Some(PathBuf::from(value)),
                "--identity" => identity = Some(PathBuf::from(value)),
                "--account-store" => account_store = Some(PathBuf::from(value)),
                "--ticket" => gateway_ticket = value,
                "--handoff-key-file" => handoff_key_file = Some(PathBuf::from(value)),
                "--seconds" => seconds = value.parse()?,
                "--studio-port" => studio_port = value.parse()?,
                "--client-logs" => client_logs = Some(PathBuf::from(value)),
                "--world" => world = Some(PathBuf::from(value)),
                "--client-locale" => {
                    let dat = Path::new(&value).join("en_us_data.dat");
                    stock_text = Some(Arc::new(osfr_tools::world_text::stock_hashes(&dat)?));
                }
                "--terrain" => {
                    terrain = Some(Arc::new(osfr_tools::terrain::Terrain::load(Path::new(
                        &value,
                    ))?))
                }
                "--trace" => {}
                _ => return Err(format!("unknown option {key}").into()),
            }
        }
        // Security Model L1/L5: loopback by default; a private-network bind only
        // with --allow-lan, in account-store mode, and never with dev tools.
        if !osfr_tools::net_policy::allowed(bind.ip(), allow_lan) {
            return Err("gateway binds loopback, or a private-network IP with --allow-lan".into());
        }
        if !bind.ip().is_loopback() && (dev_tools || account_store.is_none()) {
            return Err("a non-loopback gateway needs --account-store and refuses --dev-tools".into());
        }
        let handoff_key = handoff_key_file
            .map(fs::read)
            .transpose()?
            .unwrap_or_default();
        if account_store.is_some() && handoff_key.len() < 16 {
            return Err("--account-store requires a 16-byte --handoff-key-file".into());
        }
        Ok(Self {
            bind,
            reference: reference.ok_or("--reference is required")?,
            state: state.ok_or("--state is required")?,
            content,
            identity,
            account_store,
            gateway_ticket,
            handoff_key,
            seconds,
            dev_tools,
            studio_port,
            client_logs,
            terrain,
            world,
            stock_text,
            allow_lan,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SessionKey {
    peer: SocketAddr,
    connection_id: u32,
}

type SessionMap = BTreeMap<SessionKey, Session>;

struct Session {
    id: SessionId,
    peer: SocketAddr,
    request: SessionRequest,
    response: Vec<u8>,
    codec: Codec,
    receiver: ReliableReceiver,
    sender: ReliableSender,
    gateway: Gateway,
    state_path: PathBuf,
    // Studio packet inspector: the most recent application messages.
    recent: std::collections::VecDeque<serde_json::Value>,
    recent_on: bool,
    social_path: Option<PathBuf>,
    account_guid: u64,
    character_guid: u64,
    social_identity: Option<SocialIdentity>,
    chat_permissions: AccountChatPermissions,
    is_member: bool,
    quick_chat: QuickChatCatalog,
    selected_character: bool,
    spam_gate: SpamGate,
    session_start: Instant,
    last_activity: Instant,
    sent: u64,
    received: u64,
    trace: Option<trace::Trace>,
}

#[derive(Clone, Debug)]
struct SharedWorld {
    players: PlayerRegistry,
    social: SocialService,
    identities: BTreeMap<u64, SocialIdentity>,
    social_paths: BTreeMap<u64, PathBuf>,
    session_keys: BTreeMap<SessionId, SessionKey>,
    active_sessions: BTreeMap<u64, SessionId>,
    joined: BTreeSet<SessionId>,
    chat_sessions: BTreeMap<SessionId, ChatSession>,
    chat_history: ChatHistory,
}

impl SharedWorld {
    fn new() -> Self {
        // (R) Local runner defaults; native visibility radius and group cap are unknown.
        Self {
            players: PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap()),
            social: SocialService::new(17).unwrap(),
            identities: BTreeMap::new(),
            social_paths: BTreeMap::new(),
            session_keys: BTreeMap::new(),
            active_sessions: BTreeMap::new(),
            joined: BTreeSet::new(),
            chat_sessions: BTreeMap::new(),
            chat_history: ChatHistory::default(),
        }
    }

    fn social_profile(&mut self, identity: &SocialIdentity, path: &Path) -> io::Result<bool> {
        let guid = identity.character_guid;
        self.identities.insert(guid, identity.clone());
        self.social_paths.insert(guid, path.to_owned());
        if self.social.profile(guid).is_some() {
            return Ok(false);
        }
        let profile = match SocialLists::load(path, identity.account_id, guid) {
            Ok(profile) => profile,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                SocialLists::new(identity.clone()).map_err(invalid_io)?
            }
            Err(error) => return Err(error),
        };
        self.social.register(profile).map_err(invalid_io)?;
        Ok(true)
    }

    fn join_player(
        &mut self,
        session: &Session,
        state: &CharacterState,
    ) -> io::Result<Vec<Delivery>> {
        let zone = ZoneInstance {
            zone_id: state.zone_id,
            instance_id: 0, // (R) The current CharacterState has no instance field.
        };
        let messages = self
            .players
            .join(
                session.id,
                session.account_guid,
                session.character_guid,
                zone,
                state.position,
                heading(state.rotation),
            )
            .map_err(invalid_io)?;
        self.joined.insert(session.id);
        self.active_sessions
            .insert(session.character_guid, session.id);
        let mut deliveries = messages
            .into_iter()
            .map(|message| Delivery {
                recipient: message.recipient,
                bytes: message.bytes,
            })
            .collect::<Vec<_>>();
        let chat_name = session
            .social_identity
            .as_ref()
            .map(|identity| NameData {
                fields: [0; 3], // (R) NameData metadata is not supplied by character identity.
                first: identity.name.first.clone(),
                last: identity.name.last.clone(),
            })
            .unwrap_or_default();
        let group_id = self.social.group_of(session.character_guid).map(u64::from);
        self.chat_sessions.insert(
            session.id,
            ChatSession {
                session_id: session.id.0,
                character_guid: session.character_guid,
                name: chat_name,
                zone_id: state.zone_id,
                position: state.position,
                chat_area_id: None,
                group_id,
                guild_id: None,
                is_member: session.is_member,
            },
        );
        if let (Some(identity), Some(path)) = (&session.social_identity, &session.social_path) {
            self.social_profile(identity, path)?;
            for message in self
                .social
                .login(session.character_guid)
                .map_err(invalid_io)?
            {
                if let Some(delivery) = self.social_delivery(message) {
                    deliveries.push(delivery);
                }
            }
        }
        Ok(deliveries)
    }

    fn leave_player(&mut self, session_id: SessionId) -> Vec<Delivery> {
        self.joined.remove(&session_id);
        self.chat_sessions.remove(&session_id);
        self.players
            .leave(session_id)
            .into_iter()
            .map(|message| Delivery {
                recipient: message.recipient,
                bytes: message.bytes,
            })
            .collect()
    }

    fn social_delivery(&self, message: SocialMessage) -> Option<Delivery> {
        let recipient = self.active_sessions.get(&message.recipient_guid)?;
        Some(Delivery {
            recipient: *recipient,
            bytes: wrap_application(&message.bytes),
        })
    }

    fn refresh_group_ids(&mut self) {
        for chat in self.chat_sessions.values_mut() {
            chat.group_id = self.social.group_of(chat.character_guid).map(u64::from);
        }
    }
}

#[derive(Clone, Debug)]
struct Delivery {
    recipient: SessionId,
    bytes: Vec<u8>,
}

struct IdentityLookup<'a>(&'a BTreeMap<u64, SocialIdentity>);

impl CharacterLookup for IdentityLookup<'_> {
    fn find_by_name(&self, name: &SocialName) -> Option<SocialIdentity> {
        self.0
            .values()
            .find(|identity| {
                identity.name.first.eq_ignore_ascii_case(&name.first)
                    && identity.name.last.eq_ignore_ascii_case(&name.last)
            })
            .cloned()
    }
}

fn invalid_io(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn heading(rotation: [f32; 3]) -> f32 {
    rotation[0].atan2(rotation[2])
}

fn social_path(state_path: &Path) -> PathBuf {
    // (R) Keep host-managed social data beside the character save.
    state_path.with_extension("social.json")
}

fn character_state_path(configured: &Path, guid: u64) -> PathBuf {
    // (R) Store account-selected character state under a local per-character filename.
    configured
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("characters")
        .join(format!("{guid}.json"))
}

fn load_gateway_from(
    options: &Options,
    state_path: &Path,
    initial: CharacterState,
) -> Result<Gateway, Box<dyn std::error::Error>> {
    if let Some(parent) = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut initial = initial;
    if let Some(content) = &options.content {
        initial = Gameplay::initial_character(content, initial)?;
    }
    let mut snapshot = load_or_create(state_path, initial.clone())?;
    if snapshot.character.guid != initial.guid {
        return Err("character save belongs to another character".into());
    }
    let gateway = if let Some(content) = &options.content {
        if content.join("opening.json").is_file()
            && snapshot
                .gameplay
                .as_ref()
                .is_none_or(|game| game.opening.is_none())
        {
            snapshot.character = initial;
        }
        Gateway::load_gameplay(
            &options.reference,
            snapshot.character,
            content,
            snapshot.gameplay,
        )?
    } else if snapshot.gameplay.is_some() {
        return Err("a gameplay snapshot requires --content; refusing to discard progress".into());
    } else {
        Gateway::load(&options.reference, snapshot.character)?
    };
    let mut gateway = gateway;
    gateway.set_dev_tools(options.dev_tools);
    gateway.set_terrain(options.terrain.clone());
    gateway.set_world_dir(options.world.clone())?;
    gateway.set_stock_text(options.stock_text.clone());
    Ok(gateway)
}

fn load_gateway(options: &Options) -> Result<Gateway, Box<dyn std::error::Error>> {
    let initial = Gateway::reference_state(&options.reference)?;
    load_gateway_from(options, &options.state, initial)
}

fn load_gateway_template(options: &Options) -> Result<Gateway, Box<dyn std::error::Error>> {
    let initial = Gateway::reference_state(&options.reference)?;
    if let Some(content) = &options.content {
        let initial = Gameplay::initial_character(content, initial)?;
        Ok(Gateway::load_gameplay(
            &options.reference,
            initial,
            content,
            None,
        )?)
    } else {
        Ok(Gateway::load(&options.reference, initial)?)
    }
}

impl Session {
    fn new(
        id: SessionId,
        peer: SocketAddr,
        request: SessionRequest,
        options: &Options,
        seed: u32,
        trace: Option<trace::Trace>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if request.crc_length == 0 {
            return Err("gateway requires a nonzero CRC trailer".into());
        }
        let mut gateway = if options.account_store.is_some() {
            load_gateway_template(options)?
        } else {
            load_gateway(options)?
        };
        gateway.set_trace_enabled(trace.is_some());
        let mut social_identity = None;
        let mut account_guid = gateway.state().guid;
        let mut is_member = false;
        let selected_character = options.account_store.is_none();
        if options.account_store.is_none() {
            if let Some(path) = &options.identity {
                let identity = osfr_tools::identity::load(path)?
                    .ok_or("create a character before entering the gateway")?;
                if gateway.set_identity(identity.clone())? {
                    save(&options.state, &gateway.snapshot())?;
                }
                account_guid = gateway.state().guid; // (R) No account ID reaches this local path.
                social_identity = Some(SocialIdentity {
                    account_id: account_guid,
                    character_guid: gateway.state().guid,
                    name: SocialName::new(identity.first_name.clone(), identity.last_name.clone())
                        .map_err(invalid_io)?,
                });
                is_member = false;
            }
        }
        let quick_chat = gateway.quick_chat_catalog();
        let response = request.response_with_seed(seed);
        let character_guid = gateway.state().guid;
        let codec = Codec::new(seed, request.crc_length, true, MAX_MESSAGE)?;
        let sender = ReliableSender::new(
            codec.clone(),
            SenderConfig {
                datagram_limit: request.buffer_size,
                window: 32,
                max_message: MAX_MESSAGE,
                max_queued_bytes: 32 * 1024 * 1024,
                retry_ms: 250,
                max_retries: 40,
            },
        )?;
        Ok(Self {
            id,
            peer,
            request,
            response,
            codec,
            receiver: ReliableReceiver::new(0, 1024, MAX_MESSAGE, 2 * 1024 * 1024)?,
            sender,
            gateway,
            state_path: options.state.clone(),
            recent: std::collections::VecDeque::new(),
            recent_on: options.dev_tools,
            social_path: social_identity
                .as_ref()
                .map(|_| social_path(&options.state)),
            account_guid,
            character_guid,
            social_identity,
            // (R) Identity-file sessions have no account flags; use permissive local chat.
            chat_permissions: AccountChatPermissions::new(false, true),
            is_member,
            quick_chat,
            selected_character,
            spam_gate: SpamGate::default(),
            session_start: Instant::now(),
            last_activity: Instant::now(),
            sent: 0,
            received: 0,
            trace,
        })
    }

    fn select_character_from_login(
        &mut self,
        options: &Options,
        message: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.selected_character || options.account_store.is_none() {
            return Ok(());
        }
        let Some((guid, ticket)) = login_character_binding(message)? else {
            return Ok(());
        };
        let store = AccountStore::load(options.account_store.as_ref().unwrap())?;
        let record = store
            .character(guid)
            .ok_or("login selected a character absent from the account store")?;
        let account = store
            .account(record.owner_account_id)
            .ok_or("character account is missing")?;
        let legacy_ticket = account.id == 1 && guid == 1 && ticket == options.gateway_ticket;
        if !legacy_ticket
            && !login::verify_gateway_binding_ticket(&options.handoff_key, account, guid, &ticket)
        {
            return Err("gateway character binding ticket is invalid".into());
        }
        let mut identity = record.identity.clone();
        identity.guid = guid;
        let mut initial = record
            .world_state
            .clone()
            .unwrap_or(Gateway::reference_state(&options.reference)?);
        initial.guid = guid;
        let path = character_state_path(&options.state, guid);
        if !path.exists() && options.state.is_file() {
            // (R) Preserve the full legacy gameplay snapshot when adopting the
            // first character; AccountStore's migration stores CharacterState only.
            let legacy = load_or_create(&options.state, initial.clone())?;
            if legacy.character.guid == guid {
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&options.state, &path)?;
                initial = legacy.character;
            }
        }
        self.gateway = load_gateway_from(options, &path, initial)?;
        self.gateway.set_trace_enabled(self.trace.is_some());
        self.gateway.set_game_rules(GameRuleSet::new(
            MembershipState::new(account.flags.membership, None),
            RulePolicy::default(),
        ));
        if self.gateway.set_identity(identity.clone())? {
            save(&path, &self.gateway.snapshot())?;
        }
        self.state_path = path.clone();
        self.social_path = Some(social_path(&path));
        self.account_guid = record.owner_account_id;
        self.character_guid = guid;
        self.social_identity = Some(SocialIdentity {
            account_id: record.owner_account_id,
            character_guid: guid,
            name: SocialName::new(identity.first_name.clone(), identity.last_name.clone())
                .map_err(invalid_io)?,
        });
        self.chat_permissions =
            AccountChatPermissions::new(account.flags.underage, account.flags.open_chat);
        self.is_member = account.flags.membership;
        self.quick_chat = self.gateway.quick_chat_catalog();
        self.selected_character = true;
        Ok(())
    }

    fn send(&mut self, socket: &UdpSocket, wire: &[u8]) -> io::Result<()> {
        socket.send_to(wire, self.peer)?;
        self.sent += 1;
        if let Some(trace) = &self.trace {
            match self.codec.decode(wire) {
                Ok(packet) => trace_transport(
                    trace,
                    Some(self.id.0),
                    self.peer,
                    "out",
                    packet.opcode,
                    &packet.payload,
                    None,
                    "sent",
                    None,
                )?,
                Err(_) => trace.record(json!({
                    "event":"packet",
                    "session":self.id.0,
                    "peer":self.peer.to_string(),
                    "direction":"out",
                    "transport":{"soe_opcode":wire.get(..2).map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))},
                    "body_len":wire.len(),
                    "body_head_hex":null,
                    "outcome":"sent"
                }))?,
            }
        }
        Ok(())
    }

    fn handle(
        &mut self,
        socket: &UdpSocket,
        wire: &[u8],
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        sessions: &mut SessionMap,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if wire.first() == Some(&5) {
            let message = match self.codec.decode_kart_pose_datagram(wire) {
                Ok(message) => message,
                Err(error) => {
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.id.0,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":5,"soe_name":"DirectApplication","mode":"direct"},
                            "body_len":wire.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                        }))?;
                    }
                    return Ok(false);
                }
            };
            if let Some(trace) = &self.trace {
                let summary = trace::protocol_summary(&message[6..]);
                trace.record(json!({
                    "event":"packet","session":self.id.0,"peer":self.peer.to_string(),
                    "direction":"in","transport":{"soe_opcode":5,"soe_name":"DirectApplication","mode":"direct"},
                    "family":summary.family,"subtype":summary.subtype,
                    "family_name":summary.family.zip(summary.subtype).and_then(|(f,s)| trace::family_subtype_name(f,s)),
                    "body_len":summary.body.len(),"body_head_hex":trace::hex_prefix(summary.body, summary.sensitive),
                    "outcome":"received"
                }))?;
            }
            self.last_activity = Instant::now();
            self.received += 1;
            self.dispatch_message(
                &message,
                options,
                now_ms,
                world,
                sessions,
                Some((5, None, "direct", false)),
            )?;
            return Ok(false);
        }
        let packets = match decode_controls(&self.codec, wire) {
            Ok(packets) => packets,
            Err(error) => {
                if let Some(trace) = &self.trace {
                    trace.record(json!({
                        "event":"packet","session":self.id.0,"peer":self.peer.to_string(),
                        "direction":"in","transport":{"soe_opcode":wire.get(..2).map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))},
                        "body_len":wire.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                    }))?;
                }
                return Ok(false);
            }
        };
        self.last_activity = Instant::now();
        self.received += 1;
        for packet in packets {
            match packet.opcode {
                RELIABLE_DATA | RELIABLE_FRAGMENT => {
                    let outcome = match self.receiver.receive(&packet) {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            if let Some(trace) = &self.trace {
                                trace_transport(
                                    trace,
                                    Some(self.id.0),
                                    self.peer,
                                    "in",
                                    packet.opcode,
                                    &packet.payload,
                                    None,
                                    "rejected",
                                    Some(&error.to_string()),
                                )?;
                            }
                            return Err(error.into());
                        }
                    };
                    let rejected_reason = outcome
                        .rejected_buffered
                        .as_ref()
                        .map(|(sequence, error)| format!("seq={sequence}: {error}"));
                    if let Some(trace) = &self.trace {
                        trace_transport(
                            trace,
                            Some(self.id.0),
                            self.peer,
                            "in",
                            packet.opcode,
                            &packet.payload,
                            Some(outcome.messages.len()),
                            if outcome.rejected_buffered.is_some() {
                                "rejected"
                            } else {
                                "received"
                            },
                            rejected_reason.as_deref(),
                        )?;
                    }
                    if outcome.rejected_buffered.is_some() {
                        return Err("invalid queued reliable packet".into());
                    }
                    if let Some(ack) = outcome.acknowledgement {
                        let ack = ack.packet();
                        self.send(socket, &self.codec.encode(ack.opcode, &ack.payload)?)?;
                    }
                    let count = outcome.messages.len();
                    let sequence = (count == 1)
                        .then(|| packet.payload.get(..2))
                        .flatten()
                        .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]));
                    for message in outcome.messages {
                        self.dispatch_message(
                            &message,
                            options,
                            now_ms,
                            world,
                            sessions,
                            Some((
                                packet.opcode,
                                sequence,
                                if packet.opcode == RELIABLE_FRAGMENT {
                                    "fragment"
                                } else if count > 1 {
                                    "combined"
                                } else {
                                    "complete"
                                },
                                packet.opcode == RELIABLE_FRAGMENT,
                            )),
                        )?;
                    }
                }
                CUMULATIVE_ACK | SELECTIVE_ACK => match self.sender.acknowledge(&packet) {
                    Ok(()) => {
                        if let Some(trace) = &self.trace {
                            trace_transport(
                                trace,
                                Some(self.id.0),
                                self.peer,
                                "in",
                                packet.opcode,
                                &packet.payload,
                                None,
                                "handled",
                                None,
                            )?;
                        }
                    }
                    Err(SendError::InvalidAcknowledgement) => {
                        if let Some(trace) = &self.trace {
                            trace_transport(
                                trace,
                                Some(self.id.0),
                                self.peer,
                                "in",
                                packet.opcode,
                                &packet.payload,
                                None,
                                "ignored",
                                Some("invalid acknowledgement"),
                            )?;
                        }
                    }
                    Err(error) => {
                        if let Some(trace) = &self.trace {
                            trace_transport(
                                trace,
                                Some(self.id.0),
                                self.peer,
                                "in",
                                packet.opcode,
                                &packet.payload,
                                None,
                                "rejected",
                                Some(&error.to_string()),
                            )?;
                        }
                        return Err(error.into());
                    }
                },
                6 => {
                    if let Some(trace) = &self.trace {
                        trace_transport(
                            trace,
                            Some(self.id.0),
                            self.peer,
                            "in",
                            packet.opcode,
                            &packet.payload,
                            None,
                            "handled",
                            None,
                        )?;
                    }
                    self.send(socket, &[0, 6])?
                }
                7 => {
                    if let Some(trace) = &self.trace {
                        trace_transport(
                            trace,
                            Some(self.id.0),
                            self.peer,
                            "in",
                            packet.opcode,
                            &packet.payload,
                            None,
                            "handled",
                            None,
                        )?;
                    }
                    if let Some(reply) = clock_reflect(&packet, now_ms, self.sent, self.received) {
                        self.send(socket, &self.codec.encode(reply.opcode, &reply.payload)?)?;
                    }
                }
                5 => {
                    if let Some(trace) = &self.trace {
                        trace_transport(
                            trace,
                            Some(self.id.0),
                            self.peer,
                            "in",
                            packet.opcode,
                            &packet.payload,
                            None,
                            "handled",
                            None,
                        )?;
                    }
                    if packet.payload.len() == 6
                        && packet.payload[..4] == self.request.id.to_be_bytes()
                    {
                        return Ok(true);
                    }
                }
                _ => {
                    if let Some(trace) = &self.trace {
                        trace_transport(
                            trace,
                            Some(self.id.0),
                            self.peer,
                            "in",
                            packet.opcode,
                            &packet.payload,
                            None,
                            "ignored",
                            Some("unsupported SOE opcode"),
                        )?;
                    }
                }
            }
        }
        Ok(false)
    }

    /// Remember an application message for the Studio packet inspector.
    fn note_packet(&mut self, direction: &str, message: &[u8]) {
        let summary = trace::protocol_summary(message);
        let name = summary
            .family
            .zip(summary.subtype)
            .and_then(|(f, s)| trace::family_subtype_name(f, s));
        let hex = if summary.sensitive {
            None
        } else {
            Some(
                summary
                    .body
                    .iter()
                    .take(96)
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
            )
        };
        if self.recent.len() >= 400 {
            self.recent.pop_front();
        }
        self.recent.push_back(json!({
            "t": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
            "dir": direction,
            "family": summary.family,
            "sub": summary.subtype,
            "name": name,
            "len": summary.body.len(),
            "hex": hex
        }));
    }

    fn dispatch_message(
        &mut self,
        message: &[u8],
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        sessions: &mut SessionMap,
        transport: Option<(u16, Option<u16>, &'static str, bool)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.recent_on {
            self.note_packet("in", message);
        }
        if options.account_store.is_some()
            && !self.selected_character
            && login_character_binding(message)?.is_none()
        {
            if let Some(trace) = &self.trace {
                trace_gateway_rejected(
                    trace,
                    self.id.0,
                    self.peer,
                    transport,
                    message,
                    "gateway.character_selection",
                    "account character selection is required before gateway use",
                )?;
            }
            return Err("account character selection is required before gateway use".into());
        }
        if let Err(error) = self.select_character_from_login(options, message) {
            if let Some(trace) = &self.trace {
                trace_gateway_rejected(
                    trace,
                    self.id.0,
                    self.peer,
                    transport,
                    message,
                    "gateway.character_selection",
                    &error.to_string(),
                )?;
            }
            return Err(error);
        }
        let previous = self.gateway.state().clone();
        let mut staged_gateway = self.gateway.clone();
        let outcome = match staged_gateway.handle_at(message, now_ms) {
            Ok(outcome) => outcome,
            Err(error) => {
                if let Some(trace) = &self.trace {
                    trace_gateway_rejected(
                        trace,
                        self.id.0,
                        self.peer,
                        transport,
                        message,
                        "gateway.handle_at",
                        &error.to_string(),
                    )?;
                }
                return Err(error.into());
            }
        };
        if let Some(trace) = &self.trace {
            for detail in &outcome.trace {
                trace_gateway_message(trace, self.id.0, self.peer, transport, detail)?;
            }
        }
        self.commit_outcome(staged_gateway, previous, outcome, world, sessions)?;
        Ok(())
    }

    fn commit_outcome(
        &mut self,
        staged_gateway: Gateway,
        previous: CharacterState,
        outcome: GatewayOutcome,
        world: &mut SharedWorld,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current = staged_gateway.state().clone();
        let mut staged_world = world.clone();
        let mut deliveries = Vec::new();
        if previous.zone_id != current.zone_id && staged_world.joined.contains(&self.id) {
            deliveries.extend(staged_world.leave_player(self.id));
        }
        let moved = previous.position != current.position || previous.rotation != current.rotation;
        if outcome.entered_world && !staged_world.joined.contains(&self.id) {
            deliveries.extend(staged_world.join_player(self, &current)?);
        } else if moved && staged_world.joined.contains(&self.id) {
            let messages = staged_world
                .players
                .update_pose(
                    self.id,
                    ZoneInstance {
                        zone_id: current.zone_id,
                        instance_id: 0,
                    },
                    current.position,
                    heading(current.rotation),
                )
                .map_err(invalid_io)?;
            deliveries.extend(messages.into_iter().map(|message| Delivery {
                recipient: message.recipient,
                bytes: message.bytes,
            }));
            if let Some(chat) = staged_world.chat_sessions.get_mut(&self.id) {
                chat.zone_id = current.zone_id;
                chat.position = current.position;
            }
        }

        if staged_world.joined.contains(&self.id) {
            for application in &outcome.applications {
                if osfr_tools::chat::request(application, self.character_guid)
                    .ok()
                    .flatten()
                    .is_some_and(|text| text.trim().to_ascii_lowercase().starts_with("realm "))
                {
                    continue; // Gameplay's developer command path ran first.
                }
                deliveries.extend(self.route_application(application, &mut staged_world));
                staged_world.refresh_group_ids();
            }
        }

        if outcome.state_changed {
            save(&self.state_path, &staged_gateway.snapshot())?;
        }
        persist_social_changes(world, &staged_world)?;
        self.gateway = staged_gateway;
        *world = staged_world;
        self.character_guid = self.gateway.state().guid;

        // One `app_out` line per application message, before SOE packing, so
        // combined/fragmented replies (e.g. the 120-byte 49/1) stay visible, with
        // the inbound message(s) that caused them.
        let cause: Vec<serde_json::Value> = outcome
            .trace
            .iter()
            .map(|detail| json!({"family":detail.family,"subtype":detail.subtype,"handler":detail.handler}))
            .collect();
        for reply in outcome.replies {
            if let Some(trace) = &self.trace {
                trace_application_out(trace, self.id.0, self.peer, &reply, &cause, None)?;
            }
            if self.recent_on {
                self.note_packet("out", &reply);
            }
            self.sender.queue_message(&reply)?;
        }
        for delivery in deliveries {
            if let Some(trace) = &self.trace {
                trace_application_out(
                    trace,
                    self.id.0,
                    self.peer,
                    &delivery.bytes,
                    &cause,
                    Some(delivery.recipient.0),
                )?;
            }
            queue_delivery(self, sessions, world, delivery)?;
        }
        if outcome.state_changed {
            if let Some(trace) = &self.trace {
                trace.record(json!({
                    "event":"character_saved","session":self.id.0,"peer":self.peer.to_string(),
                    "direction":"local","zone_id":self.gateway.state().zone_id
                }))?;
            }
            event(json!({"event":"character_saved","state":self.gateway.state()}));
        }
        if outcome.gameplay_changed {
            if let Some(trace) = &self.trace {
                trace.record(json!({
                    "event":"gameplay_saved","session":self.id.0,"peer":self.peer.to_string(),
                    "direction":"local"
                }))?;
            }
            event(json!({"event":"gameplay_saved","progress":self.gateway.gameplay_summary()}));
        }
        if outcome.entered_world {
            if let Some(trace) = &self.trace {
                trace.record(json!({
                    "event":"world_ready","session":self.id.0,"peer":self.peer.to_string(),
                    "direction":"local","zone_id":self.gateway.state().zone_id
                }))?;
            }
            event(json!({"event":"world_ready","state":self.gateway.state()}));
        }
        Ok(())
    }

    fn route_application(&mut self, inner: &[u8], world: &mut SharedWorld) -> Vec<Delivery> {
        if inner.starts_with(&[15, 0, 1, 0]) {
            return self.route_typed_chat(inner, world);
        }
        if inner.starts_with(&[67, 0, 2, 0]) || inner.starts_with(&[67, 0, 3, 0]) {
            return self.route_quick_chat(inner, world);
        }
        let Ok(Some(request)) = parse_social_request(inner) else {
            return Vec::new();
        };
        let lookup = IdentityLookup(&world.identities);
        let messages = match request {
            SocialRequest::AddFriend(name) => {
                world
                    .social
                    .request_friend(self.character_guid, &name, &lookup)
            }
            SocialRequest::RemoveFriend(name) => {
                world
                    .social
                    .remove_friend(self.character_guid, &name, &lookup)
            }
            SocialRequest::FriendResponse {
                first_guid,
                second_guid,
                accepted,
            } => {
                let mut accepted_service = None;
                for requester in [first_guid, second_guid] {
                    if requester == self.character_guid {
                        continue;
                    }
                    let mut candidate = world.social.clone();
                    if let Ok(result) =
                        candidate.respond_friend(self.character_guid, requester, accepted)
                    {
                        accepted_service = Some((candidate, result));
                        break;
                    }
                }
                if let Some((candidate, result)) = accepted_service {
                    world.social = candidate;
                    Ok(result)
                } else {
                    return Vec::new();
                }
            }
            SocialRequest::AddIgnore(name) => {
                world.social.add_ignore(self.character_guid, &name, &lookup)
            }
            SocialRequest::RemoveIgnore(name) => {
                world
                    .social
                    .remove_ignore(self.character_guid, &name, &lookup)
            }
            SocialRequest::GroupInvite(name) => {
                world
                    .social
                    .invite_to_group(self.character_guid, &name, &lookup)
            }
            SocialRequest::GroupLeave => world.social.leave_group(self.character_guid),
            SocialRequest::GroupResponse(accepted) => world
                .social
                .respond_group_invite(self.character_guid, accepted),
            SocialRequest::GroupKick(guid) => {
                world.social.kick_from_group(self.character_guid, guid)
            }
        };
        messages
            .unwrap_or_default()
            .into_iter()
            .filter_map(|message| world.social_delivery(message))
            .collect()
    }

    fn route_typed_chat(&mut self, inner: &[u8], world: &mut SharedWorld) -> Vec<Delivery> {
        let request = match social_chat::parse_client_chat(inner) {
            Ok(request) => request,
            Err(error) => return chat_error_delivery(error, self.id),
        };
        if self
            .chat_permissions
            .authorize_free_text(&request.message)
            .is_err()
            || !self.spam_gate.allow(
                self.session_start.elapsed().as_millis() as u64,
                &request.message,
            )
        {
            return Vec::new();
        }
        let sessions = world.chat_sessions.values().cloned().collect::<Vec<_>>();
        let ignores = world.social.chat_ignore_lists();
        let recipients =
            match social_chat::resolve_recipients(&request, self.id.0, &sessions, &ignores) {
                Ok(recipients) => recipients,
                Err(error) => return chat_error_delivery(error, self.id),
            };
        let Some(sender) = world.chat_sessions.get(&self.id).cloned() else {
            return Vec::new();
        };
        let target = recipients
            .first()
            .and_then(|id| sessions.iter().find(|session| session.session_id == *id));
        let Ok(packet) = social_chat::server_chat_message(&request, &sender, target) else {
            return Vec::new();
        };
        let Ok(bytes) = social_chat::encode_packet_chat(&packet) else {
            return Vec::new();
        };
        world
            .chat_history
            .record(&sender, ChatLine::new([0; 2], request.message));
        recipients
            .into_iter()
            .map(|id| Delivery {
                recipient: SessionId(id),
                bytes: wrap_application(&bytes),
            })
            .collect()
    }

    fn route_quick_chat(&mut self, inner: &[u8], world: &mut SharedWorld) -> Vec<Delivery> {
        let request = match social_chat::decode_quick_chat_request(inner, &self.quick_chat) {
            Ok(request) => request,
            Err(error) => return chat_error_delivery(error, self.id),
        };
        let sessions = world.chat_sessions.values().cloned().collect::<Vec<_>>();
        let ignores = world.social.chat_ignore_lists();
        let recipients = match social_chat::resolve_quick_chat_recipients(
            &request, self.id.0, &sessions, &ignores,
        ) {
            Ok(recipients) => recipients,
            Err(error) => return chat_error_delivery(error, self.id),
        };
        let Some(sender) = world.chat_sessions.get(&self.id) else {
            return Vec::new();
        };
        let encoded = match &request {
            QuickChatRequest::Tell {
                phrase_id,
                target_name,
            } => social_chat::encode_quick_chat_tell(
                &self.quick_chat,
                *phrase_id,
                sender.character_guid,
                &sender.name,
                target_name,
            ),
            QuickChatRequest::Channel {
                phrase_id,
                channel,
                guild_guid,
            } => social_chat::encode_quick_chat_channel(
                &self.quick_chat,
                *phrase_id,
                *channel,
                sender.character_guid,
                &sender.name,
                *guild_guid,
            ),
        };
        let Ok(bytes) = encoded else {
            return Vec::new();
        };
        recipients
            .into_iter()
            .map(|id| Delivery {
                recipient: SessionId(id),
                bytes: wrap_application(&bytes),
            })
            .collect()
    }

    fn flush(&mut self, socket: &UdpSocket, now_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        for wire in self.sender.poll(now_ms)? {
            self.send(socket, &wire)?;
        }
        Ok(())
    }

    fn tick(
        &mut self,
        _options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let previous = self.gateway.state().clone();
        let mut staged_gateway = self.gateway.clone();
        let outcome = staged_gateway.tick_at(now_ms)?;
        self.commit_outcome(staged_gateway, previous, outcome, world, sessions)
    }
}

fn persist_social_changes(before: &SharedWorld, after: &SharedWorld) -> io::Result<()> {
    for (guid, path) in &after.social_paths {
        let profile = after.social.profile(*guid);
        if profile != before.social.profile(*guid) {
            if let Some(profile) = profile {
                profile.save(path)?;
            }
        }
    }
    Ok(())
}

fn queue_delivery(
    current: &mut Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
    delivery: Delivery,
) -> Result<(), Box<dyn std::error::Error>> {
    if delivery.recipient == current.id {
        current.sender.queue_message(&delivery.bytes)?;
    } else if let Some(key) = world.session_keys.get(&delivery.recipient) {
        if let Some(session) = sessions.get_mut(key) {
            session.sender.queue_message(&delivery.bytes)?;
        }
    }
    Ok(())
}

fn chat_error_delivery(error: social_chat::ChatError, recipient: SessionId) -> Vec<Delivery> {
    error
        .client_reply()
        .map(|bytes| {
            vec![Delivery {
                recipient,
                bytes: wrap_application(&bytes),
            }]
        })
        .unwrap_or_default()
}

fn wrap_application(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn login_character_binding(message: &[u8]) -> io::Result<Option<(u64, String)>> {
    if message.get(..2) != Some(&[1, 0]) {
        return Ok(None);
    }
    let mut reader = WireReader::new(message);
    reader.u16()?;
    let ticket = reader.string(256)?;
    let guid = reader.u64()?;
    Ok(Some((guid, ticket)))
}

struct WireReader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> WireReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(size)
            .ok_or_else(|| invalid_io("wire overflow"))?;
        let part = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid_io("short wire packet"))?;
        self.at = end;
        Ok(part)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn string(&mut self, max: usize) -> io::Result<String> {
        let size = usize::try_from(self.u32()?).map_err(invalid_io)?;
        if size > max {
            return Err(invalid_io("wire string exceeds bound"));
        }
        String::from_utf8(self.take(size)?.to_vec()).map_err(invalid_io)
    }
    fn social_name(&mut self) -> io::Result<SocialName> {
        for _ in 0..3 {
            self.i32()?;
        }
        SocialName::new(self.string(128)?, self.string(128)?).map_err(invalid_io)
    }
    fn end(&self) -> io::Result<()> {
        if self.at == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_io("trailing social packet data"))
        }
    }
}

enum SocialRequest {
    AddFriend(SocialName),
    RemoveFriend(SocialName),
    FriendResponse {
        first_guid: u64,
        second_guid: u64,
        accepted: bool,
    },
    AddIgnore(SocialName),
    RemoveIgnore(SocialName),
    GroupInvite(SocialName),
    GroupLeave,
    GroupResponse(bool),
    GroupKick(u64),
}

fn parse_social_request(bytes: &[u8]) -> io::Result<Option<SocialRequest>> {
    if bytes.len() < 4 {
        return Ok(None);
    }
    let family = u16::from_le_bytes(bytes[..2].try_into().unwrap());
    let sub = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
    let mut r = WireReader::new(bytes);
    r.u16()?;
    r.u16()?;
    let request = match (family, sub) {
        (26, 14) => SocialRequest::AddFriend(
            SocialName::new(r.string(128)?, r.string(128)?).map_err(invalid_io)?,
        ),
        (26, 15) => SocialRequest::RemoveFriend(
            SocialName::new(r.string(128)?, r.string(128)?).map_err(invalid_io)?,
        ),
        (26, 17) => {
            let first_guid = r.u64()?;
            let second_guid = r.u64()?;
            let accepted = r.u8()? != 0; // (R) Nonzero client result is interpreted as acceptance.
            r.social_name()?;
            SocialRequest::FriendResponse {
                first_guid,
                second_guid,
                accepted,
            }
        }
        (26, 30) => {
            // Recovered client flag: 1 adds and 0 removes.
            let adding = match r.u8()? {
                0 => false,
                1 => true,
                _ => return Err(invalid_io("invalid ignore request flag")),
            };
            let name = SocialName::new(r.string(128)?, r.string(128)?).map_err(invalid_io)?;
            if adding {
                SocialRequest::AddIgnore(name)
            } else {
                SocialRequest::RemoveIgnore(name)
            }
        }
        (40, 1) => {
            r.u16()?;
            r.u16()?;
            r.u64()?;
            r.social_name()?;
            r.u64()?;
            let target = r.social_name()?;
            r.u32()?;
            for _ in 0..5 {
                r.u32()?;
            }
            SocialRequest::GroupInvite(target)
        }
        (40, 3) => {
            r.u64()?;
            r.social_name()?;
            SocialRequest::GroupLeave
        }
        (40, 4) => {
            r.u16()?;
            r.u16()?;
            r.u64()?;
            r.social_name()?;
            r.u64()?;
            r.social_name()?;
            r.u32()?;
            let accepted = r.u8()? != 0;
            for _ in 0..5 {
                r.u32()?;
            }
            SocialRequest::GroupResponse(accepted)
        }
        (40, 6) => {
            let guid = r.u64()?;
            r.social_name()?;
            SocialRequest::GroupKick(guid)
        }
        _ => return Ok(None),
    };
    r.end()?;
    Ok(Some(request))
}

pub struct Server {
    options: Options,
    socket: UdpSocket,
    sessions: SessionMap,
    world: SharedWorld,
    start: Instant,
    generation: u32,
    trace: Option<trace::Trace>,
    studio: Option<TcpListener>,
    // Studio asset browser: index files loaded on first search.
    studio_assets: BTreeMap<String, Vec<serde_json::Value>>,
}

impl Server {
    pub fn new(options: Options) -> Result<Self, Box<dyn std::error::Error>> {
        if !osfr_tools::net_policy::allowed(options.bind.ip(), options.allow_lan) {
            return Err("gateway binds loopback, or a private-network IP with --allow-lan".into());
        }
        if let Some(parent) = options
            .state
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        if let Some(account_store) = &options.account_store {
            // On first use the login server creates the store by migrating the
            // legacy save; sessions load it after login, so it may not exist yet.
            if account_store.exists() {
                AccountStore::load(account_store)?;
            }
            load_gateway_template(&options)?;
        } else {
            let checked = load_gateway(&options)?;
            save(&options.state, &checked.snapshot())?;
        }
        let socket = UdpSocket::bind(options.bind)?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;
        // Studio developer tooling: loopback only, and only with --dev-tools.
        let studio = if options.dev_tools && options.studio_port != 0 {
            match TcpListener::bind(("127.0.0.1", options.studio_port)) {
                Ok(listener) => {
                    listener.set_nonblocking(true)?;
                    println!(
                        "{}",
                        json!({"event":"studio_listening","address":listener.local_addr()?.to_string()})
                    );
                    Some(listener)
                }
                Err(error) => {
                    println!(
                        "{}",
                        json!({"event":"studio_unavailable","reason":error.to_string()})
                    );
                    None
                }
            }
        } else {
            None
        };
        Ok(Self {
            options,
            socket,
            sessions: BTreeMap::new(),
            world: SharedWorld::new(),
            start: Instant::now(),
            generation: SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos(),
            trace: None,
            studio,
            studio_assets: BTreeMap::new(),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // One event loop serializes all shared world mutations; session state stays per entry.
        let now_ms = self.start.elapsed().as_millis() as u64;
        self.poll_studio(now_ms);
        let mut buffer = [0; 65_535];
        match self.socket.recv_from(&mut buffer) {
            Ok((length, peer))
                if osfr_tools::net_policy::allowed(peer.ip(), self.options.allow_lan) =>
            {
                let wire = &buffer[..length];
                if wire.starts_with(&[0, 1]) {
                    match SessionRequest::parse(wire) {
                        Ok(request) => {
                            if let Some(trace) = &self.trace {
                                trace.record(json!({
                                    "event":"packet","peer":peer.to_string(),"direction":"in",
                                    "transport":{"soe_opcode":1,"soe_name":"SessionRequest","mode":"handshake"},
                                    "body_len":wire.len(),"body_head_hex":null,"outcome":"accepted"
                                }))?;
                            }
                            self.accept_handshake(peer, request)?;
                        }
                        Err(error) => {
                            if let Some(trace) = &self.trace {
                                trace.record(json!({
                                    "event":"packet","peer":peer.to_string(),"direction":"in",
                                    "transport":{"soe_opcode":1,"soe_name":"SessionRequest","mode":"handshake"},
                                    "body_len":wire.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                                }))?;
                            }
                        }
                    }
                } else if let Some(key) = self.session_key_for_peer(peer) {
                    if let Some(mut current) = self.sessions.remove(&key) {
                        let result = current.handle(
                            &self.socket,
                            wire,
                            &self.options,
                            now_ms,
                            &mut self.world,
                            &mut self.sessions,
                        );
                        match result {
                            Ok(false) => {
                                self.sessions.insert(key, current);
                            }
                            Ok(true) => self.end_session(key, current, "client_disconnect")?,
                            Err(error) => self.end_session(key, current, &error.to_string())?,
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error.into()),
        }
        let keys = self.sessions.keys().copied().collect::<Vec<_>>();
        for key in keys {
            let Some(mut current) = self.sessions.remove(&key) else {
                continue;
            };
            let result = current
                .tick(&self.options, now_ms, &mut self.world, &mut self.sessions)
                .and_then(|_| current.flush(&self.socket, now_ms));
            if let Err(error) = result {
                self.end_session(key, current, &error.to_string())?;
            } else if current.last_activity.elapsed() > Duration::from_secs(60) {
                self.end_session(key, current, "idle_timeout")?;
            } else {
                self.sessions.insert(key, current);
            }
        }
        Ok(())
    }

    /// Serve pending Studio requests (a few per step; loopback peers only).
    fn poll_studio(&mut self, now_ms: u64) {
        let Some(listener) = &self.studio else {
            return;
        };
        let mut streams = Vec::new();
        for _ in 0..4 {
            match listener.accept() {
                Ok((stream, peer)) if peer.ip().is_loopback() => streams.push(stream),
                Ok(_) => {}
                Err(_) => break,
            }
        }
        for mut stream in streams {
            let (status, kind, body) = match studio::read_request(&mut stream) {
                Ok(request) => self.studio_route(&request, now_ms),
                Err(error) => (
                    400,
                    "application/json",
                    json!({"error": error.to_string()}).to_string().into_bytes(),
                ),
            };
            let _ = studio::write_response(&mut stream, status, kind, &body);
        }
    }

    /// Where authored content lives: the live gameplay's directory, else the
    /// bundled data/content.
    fn studio_content_dir(&self) -> PathBuf {
        self.studio_session()
            .and_then(|key| {
                self.sessions[&key]
                    .gateway
                    .gameplay()
                    .map(|g| g.content_dir().to_path_buf())
            })
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"))
    }

    /// The session a developer is playing (the first character in the world).
    fn studio_session(&self) -> Option<SessionKey> {
        self.sessions
            .iter()
            .find(|(_, s)| s.gateway.gameplay().is_some() && s.selected_character)
            .map(|(key, _)| *key)
    }

    fn studio_route(
        &mut self,
        request: &studio::Request,
        now_ms: u64,
    ) -> (u16, &'static str, Vec<u8>) {
        let json_reply = |status: u16, value: serde_json::Value| {
            (status, "application/json", value.to_string().into_bytes())
        };
        let (path, query) = request
            .path
            .split_once('?')
            .unwrap_or((request.path.as_str(), ""));
        let param = |name: &str| -> String {
            query
                .split('&')
                .filter_map(|pair| pair.split_once('='))
                .find(|(key, _)| *key == name)
                .map(|(_, value)| studio_decode(value))
                .unwrap_or_default()
        };
        let body_json =
            || serde_json::from_slice::<serde_json::Value>(&request.body).unwrap_or_default();
        let studio_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("studio");
        match (request.method.as_str(), path) {
            ("GET", "/api/packets") => {
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let recent: Vec<_> = self.sessions[&key].recent.iter().cloned().collect();
                json_reply(200, json!({"packets": recent}))
            }
            ("POST", "/api/send") => {
                // Poke: send a hand-written game message (hex of the inner
                // message, e.g. "23000500...") to the client as-is.
                let hex: String = body_json()
                    .get("hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                let bytes: Option<Vec<u8>> = (hex.len() % 2 == 0 && !hex.is_empty())
                    .then(|| {
                        (0..hex.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
                            .collect()
                    })
                    .flatten();
                let Some(inner) = bytes.filter(|b| b.len() <= 64 * 1024) else {
                    return json_reply(
                        400,
                        json!({"error": "expected {\"hex\": \"<inner message hex>\"}"}),
                    );
                };
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let mut message = vec![5, 0, 1];
                message.extend((inner.len() as u32).to_le_bytes());
                message.extend(&inner);
                let session = self.sessions.get_mut(&key).unwrap();
                session.note_packet("poke", &message);
                match session.sender.queue_message(&message) {
                    Ok(()) => json_reply(200, json!({"ok": true, "bytes": inner.len()})),
                    Err(error) => json_reply(400, json!({"error": format!("{error:?}")})),
                }
            }
            ("GET", "/api/fightlog") => {
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let log = self.sessions[&key].gateway.gameplay().unwrap().fight_log();
                json_reply(200, json!({"events": log}))
            }
            ("GET", "/api/savestates") => {
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let dir = savestate_dir(&self.sessions[&key].state_path);
                let mut names: Vec<String> = fs::read_dir(&dir)
                    .map(|entries| {
                        entries
                            .filter_map(Result::ok)
                            .filter_map(|e| {
                                e.file_name()
                                    .to_str()
                                    .and_then(|n| n.strip_suffix(".json"))
                                    .map(str::to_owned)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                json_reply(200, json!({"savestates": names}))
            }
            ("POST", "/api/savestate") => {
                let body = body_json();
                let name = |field: &str| {
                    body.get(field)
                        .and_then(|v| v.as_str())
                        .filter(|n| {
                            !n.is_empty()
                                && n.len() <= 48
                                && n.chars()
                                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                        })
                        .map(str::to_owned)
                };
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let state_path = self.sessions[&key].state_path.clone();
                let dir = savestate_dir(&state_path);
                if let Some(name) = name("save") {
                    let session = &self.sessions[&key];
                    let result = save(&state_path, &session.gateway.snapshot())
                        .and_then(|_| fs::create_dir_all(&dir))
                        .and_then(|_| {
                            fs::copy(&state_path, dir.join(format!("{name}.json"))).map(|_| ())
                        });
                    return match result {
                        Ok(()) => json_reply(200, json!({"ok": true, "saved": name})),
                        Err(error) => json_reply(400, json!({"error": error.to_string()})),
                    };
                }
                if let Some(name) = name("load") {
                    let source = dir.join(format!("{name}.json"));
                    if !source.is_file() {
                        return json_reply(404, json!({"error": format!("no savestate {name}")}));
                    }
                    // End the session (it saves first), then restore the file:
                    // the client reconnects into the saved state.
                    let session = self.sessions.remove(&key).unwrap();
                    if let Err(error) = self.end_session(key, session, "studio_savestate_load") {
                        return json_reply(400, json!({"error": error.to_string()}));
                    }
                    return match fs::copy(&source, &state_path) {
                        Ok(_) => json_reply(
                            200,
                            json!({"ok": true, "loaded": name, "note": "log in again to enter the saved state"}),
                        ),
                        Err(error) => json_reply(400, json!({"error": error.to_string()})),
                    };
                }
                json_reply(
                    400,
                    json!({"error": "expected {\"save\": name} or {\"load\": name} (letters, digits, - and _)"}),
                )
            }
            ("GET", "/api/clientlog") => {
                let Some(dir) = &self.options.client_logs else {
                    return json_reply(
                        200,
                        json!({"lines": [], "note": "start the gateway with --client-logs <client>/Logs"}),
                    );
                };
                let mut lines = Vec::new();
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.filter_map(Result::ok) {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) != Some("log") {
                            continue;
                        }
                        let file = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or_default()
                            .to_owned();
                        let text = fs::read(&path)
                            .map(|b| String::from_utf8_lossy(&b).into_owned())
                            .unwrap_or_default();
                        for line in text.lines().rev().take(400) {
                            let lower = line.to_ascii_lowercase();
                            if lower.contains("error")
                                || lower.contains("fail")
                                || lower.contains("missing")
                                || lower.contains("warn")
                            {
                                lines.push(json!({"file": file, "line": line}));
                            }
                        }
                    }
                }
                lines.truncate(300);
                json_reply(200, json!({"lines": lines}))
            }
            ("GET", "/api/content") => {
                let dir = self.studio_content_dir();
                let mut files = Vec::new();
                collect_json_files(&dir, &dir, &mut files);
                files.sort();
                json_reply(200, json!({"files": files}))
            }
            ("GET" | "POST", p) if p.starts_with("/api/content/") => {
                let relative = studio_decode(&p["/api/content/".len()..]);
                let safe = relative.ends_with(".json")
                    && !relative.contains("..")
                    && !relative.starts_with(['/', '\\'])
                    && relative
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || "_-./".contains(c));
                if !safe {
                    return json_reply(400, json!({"error": "bad content path"}));
                }
                let path = self.studio_content_dir().join(&relative);
                if request.method == "GET" {
                    return match fs::read(&path) {
                        Ok(bytes) => (200, "application/json", bytes),
                        Err(error) => json_reply(404, json!({"error": error.to_string()})),
                    };
                }
                if serde_json::from_slice::<serde_json::Value>(&request.body).is_err() {
                    return json_reply(400, json!({"error": "not valid JSON; nothing written"}));
                }
                if let Err(error) = fs::write(&path, &request.body) {
                    return json_reply(400, json!({"error": error.to_string()}));
                }
                json_reply(
                    200,
                    json!({"ok": true, "written": relative, "hint": "run 'reload' to apply battles, enemies or abilities"}),
                )
            }
            ("GET", p) if p.starts_with("/api/asset-file/") => {
                // Studio inspector: the browser loads index files (e.g. the
                // 40+ MB graph.json) once and walks them itself, so the game
                // loop never holds or searches them.
                let name = &p["/api/asset-file/".len()..];
                let safe = name.ends_with(".json")
                    && name.chars().all(|c| {
                        c.is_ascii_lowercase()
                            || c.is_ascii_digit()
                            || c == '_'
                            || c == '.'
                            || c == '-'
                    })
                    && !name.contains("..");
                if !safe {
                    return json_reply(400, json!({"error": "bad asset file name"}));
                }
                match fs::read(studio_dir.join("assets").join(name)) {
                    Ok(bytes) => (200, "application/json", bytes),
                    Err(_) => json_reply(
                        404,
                        json!({"error": format!("{name} missing: run python -I project/scripts/build_asset_graph.py")}),
                    ),
                }
            }
            ("GET", "/api/report") => match fs::read(studio_dir.join("report.json")) {
                Ok(bytes) => (200, "application/json", bytes),
                Err(_) => json_reply(
                    404,
                    json!({"error": "no report yet: run python -I project/scripts/content_check.py"}),
                ),
            },
            ("GET", "/api/assets") => {
                let kind = param("kind");
                let q = param("q").to_ascii_lowercase();
                if !["images", "models", "text", "animations", "effects"].contains(&kind.as_str()) {
                    return json_reply(
                        400,
                        json!({"error": "kind is one of images, models, text, animations, effects"}),
                    );
                }
                if !self.studio_assets.contains_key(&kind) {
                    let rows = fs::read(studio_dir.join("assets").join(format!("{kind}.json")))
                        .ok()
                        .and_then(|b| serde_json::from_slice::<Vec<serde_json::Value>>(&b).ok());
                    let Some(rows) = rows else {
                        return json_reply(
                            404,
                            json!({"error": "no asset index: run python -I project/scripts/build_asset_index.py"}),
                        );
                    };
                    self.studio_assets.insert(kind.clone(), rows);
                }
                let rows = &self.studio_assets[&kind];
                let words: Vec<&str> = q.split_whitespace().collect();
                let matches: Vec<_> = rows
                    .iter()
                    .filter(|row| {
                        let text = row.to_string().to_ascii_lowercase();
                        words.iter().all(|w| text.contains(w))
                    })
                    .take(200)
                    .cloned()
                    .collect();
                json_reply(
                    200,
                    json!({"kind": kind, "count": matches.len(), "rows": matches}),
                )
            }
            ("GET", p) if p.starts_with("/api/world/") => {
                // Live-world files (placed NPCs, marks) for the Studio map.
                let name = &p["/api/world/".len()..];
                if !name.ends_with(".json") || !name.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '.') {
                    return json_reply(400, json!({"error": "bad world file name"}));
                }
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let dir = self.sessions[&key].gateway.gameplay().unwrap().live_dir();
                match fs::read(dir.join(name)) {
                    Ok(bytes) => (200, "application/json", bytes),
                    Err(_) => json_reply(404, json!({"error": format!("no {name} yet")})),
                }
            }
            ("GET", "/api/terrain") => {
                // Studio map: which zones have ground heights, and their grids.
                let zones: Vec<_> = self
                    .options
                    .terrain
                    .iter()
                    .flat_map(|t| t.zone_ids().into_iter().filter_map(|id| Some((id, t.grid(id)?))))
                    .map(|(id, g)| {
                        json!({"zone": id, "min_x": g.min_x, "min_z": g.min_z, "cell": g.cell,
                               "cols": g.cols, "rows": g.rows})
                    })
                    .collect();
                json_reply(200, json!({"zones": zones}))
            }
            ("GET", p) if p.starts_with("/api/terrain/") => {
                // The raw grid (OSFRHGT1); the browser draws the map itself.
                let file = p["/api/terrain/".len()..]
                    .parse::<u32>()
                    .ok()
                    .and_then(|zone| self.options.terrain.as_ref()?.file(zone));
                match file.map(fs::read) {
                    Some(Ok(bytes)) => (200, "application/octet-stream", bytes),
                    _ => json_reply(
                        404,
                        json!({"error": "no terrain for that zone: run osfr-terrain and start with --terrain"}),
                    ),
                }
            }
            ("GET", "/api/height") => {
                let number = |name: &str| param(name).parse::<f32>().ok();
                let zone = param("zone").parse::<u32>().unwrap_or(0);
                let height = match (number("x"), number("z"), &self.options.terrain) {
                    (Some(x), Some(z), Some(terrain)) => terrain.height_at(zone, x, z),
                    _ => None,
                };
                json_reply(200, json!({"zone": zone, "height": height}))
            }
            ("GET", "/") => (
                200,
                "text/html; charset=utf-8",
                STUDIO_PAGE.as_bytes().to_vec(),
            ),
            ("GET", "/api/state") => {
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                let session = &self.sessions[&key];
                let character = session.gateway.state();
                let gameplay = session.gateway.gameplay().unwrap();
                let p = character.position;
                let mut actors: Vec<_> = gameplay
                    .known_actors(character)
                    .into_iter()
                    .map(|t| {
                        let d = ((t.position[0] - p[0]).powi(2) + (t.position[2] - p[2]).powi(2))
                            .sqrt();
                        (d, t)
                    })
                    .collect();
                actors.sort_by(|a, b| a.0.total_cmp(&b.0));
                let actors: Vec<_> = actors
                    .into_iter()
                    .take(60)
                    .map(|(d, t)| {
                        json!({"guid": t.guid.to_string(), "name": t.name, "kind": t.kind,
                               "position": t.position, "distance": d})
                    })
                    .collect();
                json_reply(
                    200,
                    json!({"character": character, "summary": gameplay.summary(), "actors": actors}),
                )
            }
            ("POST", "/api/command") => {
                let text = serde_json::from_slice::<serde_json::Value>(&request.body)
                    .ok()
                    .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(str::to_owned))
                    .unwrap_or_default();
                let text = text.trim().to_owned();
                if text.is_empty() || text.len() > 400 || text.contains(['\r', '\n']) {
                    return json_reply(
                        400,
                        json!({"error": "expected {\"text\": \"<realm command>\"}"}),
                    );
                }
                let Some(key) = self.studio_session() else {
                    return json_reply(409, json!({"error": "no character in the world"}));
                };
                // Run it exactly as if typed in chat: the reply shows in the client.
                let inner = osfr_tools::chat::local_request(&format!("realm {text}"));
                let mut message = vec![5, 0, 1];
                message.extend((inner.len() as u32).to_le_bytes());
                message.extend(inner);
                let mut current = self.sessions.remove(&key).unwrap();
                let result = current.dispatch_message(
                    &message,
                    &self.options,
                    now_ms,
                    &mut self.world,
                    &mut self.sessions,
                    None,
                );
                self.sessions.insert(key, current);
                match result {
                    Ok(()) => {
                        json_reply(200, json!({"ok": true, "command": format!("realm {text}")}))
                    }
                    Err(error) => json_reply(400, json!({"error": error.to_string()})),
                }
            }
            _ => json_reply(404, json!({"error": "unknown studio route"})),
        }
    }

    fn session_key_for_peer(&self, peer: SocketAddr) -> Option<SessionKey> {
        self.sessions.keys().find(|key| key.peer == peer).copied()
    }

    fn accept_handshake(
        &mut self,
        peer: SocketAddr,
        request: SessionRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = SessionKey {
            peer,
            connection_id: request.id,
        };
        if let Some(existing) = self.sessions.get_mut(&key) {
            let response = existing.response.clone();
            existing.send(&self.socket, &response)?;
            return Ok(());
        }
        if let Some(old_key) = self.session_key_for_peer(peer) {
            let old = self.sessions.remove(&old_key).unwrap();
            self.end_session(old_key, old, "connection_replaced")?;
        }
        // (R) Without account-store character selection, the configured save is exclusive.
        if self.options.account_store.is_none() && !self.sessions.is_empty() {
            if let Some(trace) = &self.trace {
                trace.record(json!({
                    "event":"session_rejected","peer":peer.to_string(),"direction":"local",
                    "outcome":"rejected","reason":"single configured character state is already in use"
                }))?;
            }
            event(
                json!({"event":"session_rejected","reason":"single configured character state is already in use"}),
            );
            return Ok(());
        }
        self.generation = self.generation.wrapping_add(1);
        let seed = CRC_SEED ^ self.generation;
        let id = SessionId(u64::from(self.generation));
        match Session::new(
            id,
            peer,
            request.clone(),
            &self.options,
            seed,
            self.trace.clone(),
        ) {
            Ok(mut session) => {
                self.world.session_keys.insert(id, key);
                if let Some(trace) = &session.trace {
                    trace.record(json!({
                        "event":"session_started","session":session.id.0,"peer":peer.to_string(),
                        "direction":"local","crc_length":request.crc_length,
                        "buffer_size":request.buffer_size,"outcome":"accepted"
                    }))?;
                }
                event(
                    json!({"event":"session_started","peer":peer.to_string(),"crc_length":request.crc_length,"buffer_size":request.buffer_size}),
                );
                let response = session.response.clone();
                session.send(&self.socket, &response)?;
                self.sessions.insert(key, session);
            }
            Err(error) => {
                if let Some(trace) = &self.trace {
                    trace.record(json!({
                        "event":"session_rejected","peer":peer.to_string(),"direction":"local",
                        "outcome":"rejected","reason":error.to_string()
                    }))?;
                }
                event(json!({"event":"session_rejected","reason":error.to_string()}));
            }
        }
        Ok(())
    }

    fn end_session(
        &mut self,
        key: SessionKey,
        mut session: Session,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // A duplicate login of a character that another session already owns
        // must not save its stale copy over the live one (10-15 audit).
        let another_owns = self
            .world
            .active_sessions
            .get(&session.character_guid)
            .is_some_and(|owner| *owner != session.id);
        if session.selected_character && !another_owns {
            save(&session.state_path, &session.gateway.snapshot())?;
        }
        let mut deliveries = self.world.leave_player(session.id);
        let owns_active =
            self.world.active_sessions.get(&session.character_guid) == Some(&session.id);
        if owns_active && self.world.social.profile(session.character_guid).is_some() {
            for message in self
                .world
                .social
                .disconnect(session.character_guid)
                .map_err(invalid_io)?
            {
                if let Some(delivery) = self.world.social_delivery(message) {
                    deliveries.push(delivery);
                }
            }
        }
        if owns_active {
            self.world.active_sessions.remove(&session.character_guid);
        }
        self.world.session_keys.remove(&session.id);
        for delivery in deliveries {
            if delivery.recipient != session.id {
                queue_delivery(&mut session, &mut self.sessions, &self.world, delivery)?;
            }
        }
        event(json!({"event":"session_ended","peer":key.peer.to_string(),"reason":reason}));
        if let Some(trace) = &session.trace {
            trace.record(json!({
                "event":"session_ended","session":session.id.0,"peer":key.peer.to_string(),
                "direction":"local","outcome":"closed","reason":reason
            }))?;
        }
        Ok(())
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let trace_path = trace_path_from_args();
    let options = Options::parse()?;
    let mut server = Server::new(options)?;
    server.trace = trace_path.as_deref().map(trace::Trace::open).transpose()?;
    event(
        json!({"event":"listening","address":server.local_addr()?.to_string(),"reference":server.options.reference,"dev_tools":server.options.dev_tools}),
    );
    while server.options.seconds == 0
        || server.start.elapsed() < Duration::from_secs(server.options.seconds)
    {
        server.step()?;
    }
    let keys = server.sessions.keys().copied().collect::<Vec<_>>();
    for key in keys {
        if let Some(session) = server.sessions.remove(&key) {
            server.end_session(key, session, "server_shutdown")?;
        }
    }
    event(json!({"event":"stopped"}));
    Ok(())
}

fn trace_path_from_args() -> Option<PathBuf> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--trace" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

fn main() {
    if let Err(error) = run() {
        eprintln!("osfr-gateway: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miniz_oxide::deflate::compress_to_vec_zlib;

    fn wrap(inner: &[u8]) -> Vec<u8> {
        let mut app = vec![5, 0, 1];
        app.extend((inner.len() as u32).to_le_bytes());
        app.extend(inner);
        app
    }
    fn direct(app: &[u8], compressed: bool) -> Vec<u8> {
        let mut wire = vec![app[0], u8::from(compressed)];
        if compressed {
            wire.extend(compress_to_vec_zlib(&app[1..], 6));
        } else {
            wire.extend(&app[1..]);
        }
        let crc = osfr_tools::soe::seeded_crc32(CRC_SEED, &wire).to_be_bytes();
        wire.extend(&crc[1..]);
        wire
    }

    #[test]
    #[ignore = "requires OSFR_KART_WEAPONS private content and initialization"]
    fn private_direct_pose_reaches_session_and_unlocks_reliable_rocket_use() {
        let root = PathBuf::from(env::var_os("OSFR_KART_WEAPONS").unwrap());
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let saved = env::temp_dir().join(format!(
            "osfr-kart-direct-{}-{nonce}.json",
            std::process::id()
        ));
        std::fs::copy(root.join("initial-character.json"), &saved).unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let options = Options {
            bind: socket.local_addr().unwrap(),
            reference: root.join("../../references/rust-gateway-reference"),
            state: saved.clone(),
            content: Some(root.join("content")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.to_owned(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: true,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: None,
            stock_text: None,
            allow_lan: false,
        };
        let mut session = Session::new(
            SessionId(17),
            client.local_addr().unwrap(),
            SessionRequest {
                id: 17,
                crc_length: 3,
                buffer_size: 512,
            },
            &options,
            CRC_SEED,
            None,
        )
        .unwrap();
        let mut world = SharedWorld::new();
        let mut sessions = SessionMap::new();
        world.session_keys.insert(
            session.id,
            SessionKey {
                peer: session.peer,
                connection_id: session.request.id,
            },
        );
        let mut login = 1u16.to_le_bytes().to_vec();
        for (i, text) in ["private-direct-regression", "1.910.1.530630", ""]
            .iter()
            .enumerate()
        {
            login.extend((text.len() as u32).to_le_bytes());
            login.extend(text.bytes());
            if i == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        session.gateway.handle_at(&login, 0).unwrap();
        session.gateway.handle_at(&wrap(&[13, 0]), 100).unwrap();
        session.gateway.handle_at(&wrap(&[10, 0]), 200).unwrap();
        let mut chat = vec![15, 0, 1, 0, 0, 0];
        chat.extend(1u64.to_le_bytes());
        chat.extend(0u64.to_le_bytes());
        chat.extend([0; 40]);
        chat.extend(10u32.to_le_bytes());
        chat.extend(b"realm race");
        chat.extend([0; 28]);
        session.gateway.handle_at(&wrap(&chat), 1100).unwrap();
        session.gateway.handle_at(&wrap(&[13, 0]), 1200).unwrap();
        let mut register = vec![33, 0, 9, 0];
        register.extend(1u64.to_le_bytes());
        register.extend([0; 8]);
        session.gateway.handle_at(&wrap(&register), 1500).unwrap();
        session.gateway.handle_at(&wrap(&[10, 0]), 1600).unwrap();
        let doc: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("initial-character.json")).unwrap())
                .unwrap();
        let instance = doc["gameplay"]["kart_entry"]["next_instance"]
            .as_u64()
            .unwrap() as u32;
        let mut start = vec![41, 0, 108, 0];
        start.extend(917566u32.to_le_bytes());
        start.extend(instance.to_le_bytes());
        start.extend(1u64.to_le_bytes());
        session.gateway.handle_at(&wrap(&start), 2000).unwrap();
        session.gateway.tick_at(20_000).unwrap();
        let content: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("content/kart_entry.json")).unwrap())
                .unwrap();
        let position: [f32; 3] =
            serde_json::from_value(content["race"]["pickups"][0]["position"].clone()).unwrap();
        let mut claim = vec![33, 0, 17, 0];
        claim.extend(1u64.to_le_bytes());
        for v in position {
            claim.extend(v.to_le_bytes());
        }
        claim.extend(1f32.to_le_bytes());
        session.gateway.handle_at(&wrap(&claim), 20_100).unwrap();
        let mut use_item = vec![33, 0, 19, 0];
        use_item.extend(1u64.to_le_bytes());
        assert!(
            session
                .gateway
                .handle_at(&wrap(&use_item), 20_105)
                .unwrap()
                .replies
                .is_empty()
        );
        let before = serde_json::to_value(session.gateway.snapshot()).unwrap();
        assert_eq!(
            before["gameplay"]["kart_entry"]["race"]["queued_items"],
            json!([1])
        );
        // Corrupt pose and a valid-CRC direct use must not change live gameplay.
        let mut pose = vec![33, 0, 6, 0];
        let mut body = vec![0; 85];
        for (i, v) in [1f32, 0., 0., 0., 1., 0., 0., 0., 1.]
            .iter()
            .chain(position.iter())
            .enumerate()
        {
            body[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        body[81..85].copy_from_slice(&1u32.to_le_bytes());
        pose.extend(body);
        let mut corrupt = direct(&wrap(&pose), false);
        corrupt[20] ^= 1;
        session
            .handle(
                &socket,
                &corrupt,
                &options,
                20_110,
                &mut world,
                &mut sessions,
            )
            .unwrap();
        session
            .handle(
                &socket,
                &direct(&wrap(&use_item), true),
                &options,
                20_111,
                &mut world,
                &mut sessions,
            )
            .unwrap();
        assert_eq!(
            serde_json::to_value(session.gateway.snapshot()).unwrap(),
            before
        );
        for compressed in [false, true] {
            session
                .handle(
                    &socket,
                    &direct(&wrap(&pose), compressed),
                    &options,
                    20_120,
                    &mut world,
                    &mut sessions,
                )
                .unwrap();
        }
        let after = serde_json::to_value(session.gateway.snapshot()).unwrap();
        assert!(after["gameplay"]["kart_entry"]["race"]["weapon_pose"].is_object());
        assert_eq!(
            after["gameplay"]["kart_entry"]["race"]["queued_items"],
            json!([1])
        );
        let mut reliable = 0u16.to_be_bytes().to_vec();
        reliable.extend(wrap(&use_item));
        session
            .handle(
                &socket,
                &session.codec.encode(RELIABLE_DATA, &reliable).unwrap(),
                &options,
                20_130,
                &mut world,
                &mut sessions,
            )
            .unwrap();
        let paid = serde_json::to_value(session.gateway.snapshot()).unwrap();
        assert_eq!(
            paid["gameplay"]["kart_entry"]["race"]["queued_items"],
            json!([])
        );
        assert_eq!(
            paid["gameplay"]["kart_entry"]["race"]["weapons"]["weapons"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(paid["character"], before["character"]);
        assert_eq!(
            load_or_create(&saved, session.gateway.state().clone()).unwrap(),
            session.gateway.snapshot()
        );
        std::fs::remove_file(saved).unwrap();
    }
}
