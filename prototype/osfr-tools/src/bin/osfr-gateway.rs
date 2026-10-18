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
use osfr_tools::mounts::{MountTail, RidePose, RideView};
use osfr_tools::net_policy::{Bucket, IpLimiter};
use osfr_tools::pets::{PetPresence, pet_removal_packet};
use osfr_tools::player_radial::{Chosen, MenuOffer, PlayerRadial, PublicLook};
use osfr_tools::players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance};
use osfr_tools::security_log::{EventKind, SecurityEvent, SecurityLog};
use osfr_tools::session::{CRC_SEED, SessionRequest, clock_reflect, decode_controls};
use osfr_tools::social_chat::{
    self, AccountChatPermissions, ChatHistory, ChatLine, ChatSession, NameData, QuickChatCatalog,
    QuickChatRequest, SpamGate,
};
use osfr_tools::social_lists::{
    CharacterIdentity as SocialIdentity, CharacterLookup, LocationRequest, MapSpot, SocialLists,
    SocialMessage, SocialName, SocialService, parse_location_request,
};
use osfr_tools::soe::{
    CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
};
use osfr_tools::soe_send::{ReliableSender, SendError, SenderConfig};
use osfr_tools::studio;
use osfr_tools::trade::{TradeMessage, TradeService, TradeSettlement};
use osfr_tools::world::{World, WorldPose};

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
    sync::{Arc, Mutex},
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
    /// Bounded refusal log (Security Model L9) in a host-protected folder.
    pub security_log: Option<PathBuf>,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut args = env::args().skip(1);
        let bind: SocketAddr = args
            .next()
            .ok_or("usage: osfr-gateway 127.0.0.1:PORT --reference DIR --state FILE [--content DIR] [--identity FILE] [--account-store FILE] [--seconds N] [--trace FILE] [--security-log FILE] [--dev-tools]")?
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
        let mut security_log = None;
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
                "--security-log" => security_log = Some(PathBuf::from(value)),
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
            return Err(
                "a non-loopback gateway needs --account-store and refuses --dev-tools".into(),
            );
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
            security_log,
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
    known_npcs: BTreeSet<u64>,
    npc_visibility_revision: Option<u64>,
    // (R) W106's directed-view approach: unchanged inputs retain the known set.
    npc_view_position: Option<[f32; 3]>,
    npc_view_fields: Vec<osfr_tools::world::NpcInterest>,
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
    /// (R) Emote actor state, refreshed from the committed gateway before routing.
    emote: social_chat::piece_093::EmoteContext,
    /// Open player radial menu / inspect for this session (W143).
    radial: PlayerRadial,
    selected_character: bool,
    spam_gate: SpamGate,
    session_start: Instant,
    /// (R) Movement-only changes are saved at most every `POSITION_SAVE_MS`:
    /// time of this session's last save (None: none yet, so the next is written
    /// at once), and whether a newer pose is unsaved.
    position_saved_ms: Option<u64>,
    position_dirty: bool,
    last_activity: Instant,
    packets: Bucket,
    bytes: Bucket,
    sent: u64,
    received: u64,
    trace: Option<trace::Trace>,
}

#[derive(Clone, Debug)]
struct SharedWorld {
    players: PlayerRegistry,
    /// What a stranger may see of each joined character, keyed by character guid (W143).
    /// Filled at join and on profile/equipment change; empty means no Inspect entry (R).
    public_looks: BTreeMap<u64, PublicLook>,
    /// Player trade sessions (W163). Staged with the world, so a message that is rolled back
    /// leaves no trade behind.
    trades: TradeService,
    /// Settlements produced while staging. They are committed only after the staged world is
    /// installed (`settle_trades`), never inside the staging step (R).
    trade_settlements: Vec<TradeSettlement>,
    /// W162: each mounted or ride-summoning character's ride, keyed by character guid.
    rides: BTreeMap<u64, RideView>,
    /// W177: each summoned pet, keyed by its owner's character guid.
    pets: BTreeMap<u64, PetPresence>,
    /// (R) Rolling-hour fishing payout timestamps keyed by character. This survives a
    /// reconnect and save reload for the process lifetime; durable persistence is a save-schema
    /// decision for the lead.
    fishing_hours: BTreeMap<u64, Vec<u64>>,
    social: SocialService,
    identities: BTreeMap<u64, SocialIdentity>,
    social_paths: BTreeMap<u64, PathBuf>,
    session_keys: BTreeMap<SessionId, SessionKey>,
    active_sessions: BTreeMap<u64, SessionId>,
    joined: BTreeSet<SessionId>,
    chat_sessions: BTreeMap<SessionId, ChatSession>,
    chat_history: ChatHistory,
    /// W173: each joined character's channel switches and bubble colour (26/32, 26/33, 26/18).
    chat_settings: social_chat::ChatSettingsBook,
    security: Arc<Mutex<Security>>,
}

/// Security Model L6/L9 state. Every staged clone of the world shares it, so a
/// rolled-back message neither duplicates nor refunds an account's budget.
#[derive(Default)]
struct Security {
    budgets: social_chat::ActionBudgets,
    combat: BTreeMap<u64, Bucket>,
    log: Option<SecurityLog>,
}

impl std::fmt::Debug for Security {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Security")
    }
}

impl Security {
    // (R) Shared, nonrefundable account budget, 10/s burst 20; bounded keys.
    fn allow_combat(&mut self, account: u64, now: Instant) -> bool {
        if !self.combat.contains_key(&account) && self.combat.len() >= MAX_SESSIONS {
            self.combat.retain(|_, bucket| {
                let mut probe = *bucket;
                !probe.take(20., 10., 20., now)
            });
            if self.combat.len() >= MAX_SESSIONS {
                return false;
            }
        }
        self.combat
            .entry(account)
            .or_insert_with(|| Bucket::full(20., now))
            .take(1., 10., 20., now)
    }

    /// The log batches and throttles its own I/O; a log error never drops a peer.
    fn record(&mut self, kind: EventKind, account_id: Option<u64>, session_id: Option<u64>) {
        let Some(log) = &mut self.log else { return };
        if let Err(error) = log.record(SecurityEvent {
            kind,
            account_id,
            session_id,
        }) {
            eprintln!("security log: {error}");
        }
    }
}

impl SharedWorld {
    fn new() -> Self {
        // (R) Local runner defaults; native visibility radius and group cap are unknown.
        Self {
            trades: TradeService::new(),
            trade_settlements: Vec::new(),
            players: PlayerRegistry::new(InterestPolicy::new(100.0, 64).unwrap()),
            public_looks: BTreeMap::new(),
            rides: BTreeMap::new(),
            pets: BTreeMap::new(),
            fishing_hours: BTreeMap::new(),
            social: SocialService::new(17).unwrap(),
            identities: BTreeMap::new(),
            social_paths: BTreeMap::new(),
            session_keys: BTreeMap::new(),
            active_sessions: BTreeMap::new(),
            joined: BTreeSet::new(),
            chat_sessions: BTreeMap::new(),
            chat_history: ChatHistory::default(),
            chat_settings: social_chat::ChatSettingsBook::default(),
            security: Arc::default(),
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
        look: osfr_tools::players::PlayerPresentation,
        zone: ZoneInstance,
    ) -> io::Result<Vec<Delivery>> {
        let messages = self
            .players
            .join_with_presentation(
                session.id,
                session.account_guid,
                session.character_guid,
                zone,
                state.position,
                heading(state.rotation),
                look,
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
        // W173: reload this character's chat switches and bubble from the file beside the
        // social list; no identity means in-memory only.
        self.chat_settings.join(
            session.character_guid,
            session
                .social_path
                .as_deref()
                .map(chat_settings_path)
                .as_deref(),
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
        let mut deliveries = Vec::new();
        if let Some(player) = self.players.get(session_id) {
            self.public_looks.remove(&player.character_guid);
            if let Some(view) = self.rides.remove(&player.character_guid) {
                deliveries = self.ride_fanout(session_id, &[view.removal_packet()]);
            }
            if let Some(pet) = self.pets.remove(&player.character_guid) {
                deliveries
                    .extend(self.ride_fanout(session_id, &[pet_removal_packet(pet.actor_guid)]));
            }
            // Leaving (disconnect or zoning) ends any trade the character is in (W163).
            deliveries.extend(trade_deliveries(
                self.trades.unregister_player(player.character_guid),
            ));
        }
        self.joined.remove(&session_id);
        self.chat_sessions.remove(&session_id);
        deliveries.extend(
            self.players
                .leave(session_id)
                .into_iter()
                .map(|message| Delivery {
                    recipient: message.recipient,
                    bytes: message.bytes,
                }),
        );
        deliveries
    }

    /// W162: `packets` for every other player within interest range of `session`, found from the
    /// registry's current position (so call it before a zone transfer removes the player).
    fn ride_fanout(&self, session: SessionId, packets: &[Vec<u8>]) -> Vec<Delivery> {
        let Some(player) = self
            .players
            .get(session)
            .filter(|_| self.joined.contains(&session))
        else {
            return Vec::new();
        };
        self.players
            .nearby_sessions(player.zone, player.position)
            .into_iter()
            .filter(|other| *other != session)
            .flat_map(|other| {
                packets.iter().map(move |packet| Delivery {
                    recipient: other,
                    bytes: wrap_application(packet),
                })
            })
            .collect()
    }

    /// W162: whoever is told about a mounted player (AddPc, 35/1) also gets the ride actor and
    /// the rider's attachment, and whoever loses the player (RemovePlayer, 35/3) loses the ride.
    fn with_rides(&self, deliveries: Vec<Delivery>) -> Vec<Delivery> {
        if self.rides.is_empty() {
            return deliveries;
        }
        let mut out = Vec::with_capacity(deliveries.len());
        for delivery in deliveries {
            let recipient = delivery.recipient;
            let extra = rider_event(&delivery.bytes).and_then(|(guid, added)| {
                let view = self.rides.get(&guid)?;
                if !added {
                    return Some(vec![view.removal_packet()]);
                }
                let player = self.players.get(*self.active_sessions.get(&guid)?)?;
                Some(view.packets(RidePose {
                    position: player.position,
                    heading: player.heading,
                }))
            });
            // W167 (R): a seated rider's 125 is followed by the ride's copy (see `RideView::follow`).
            let extra = extra.or_else(|| self.ride_follow(&delivery.bytes));
            out.push(delivery);
            out.extend(extra.into_iter().flatten().map(|packet| Delivery {
                recipient,
                bytes: wrap_application(&packet),
            }));
        }
        out
    }

    /// W167: the ride's copy of a seated rider's 125 position update (tunnel frame in, inner
    /// packets out); `None` for every other frame.
    fn ride_follow(&self, frame: &[u8]) -> Option<Vec<Vec<u8>>> {
        let inner = frame.get(7..)?;
        let rider = u64::from_le_bytes(inner.get(2..10)?.try_into().ok()?);
        Some(vec![self.rides.get(&rider)?.follow(inner)?])
    }

    /// W177: replay a summoned pet's actor to whoever is told that its owner entered, and
    /// remove it from whoever loses the owner. This mirrors the W162 ride late-viewer path.
    fn with_pets(&self, deliveries: Vec<Delivery>) -> Vec<Delivery> {
        if self.pets.is_empty() {
            return deliveries;
        }
        let mut out = Vec::with_capacity(deliveries.len());
        for delivery in deliveries {
            let recipient = delivery.recipient;
            let extra = rider_event(&delivery.bytes).and_then(|(guid, added)| {
                let pet = self.pets.get(&guid)?;
                if !added {
                    return Some(vec![pet_removal_packet(pet.actor_guid)]);
                }
                let mut view = pet.clone();
                let player = self.players.get(*self.active_sessions.get(&guid)?)?;
                view.position = player.position;
                view.spawn_packet().map(|packet| vec![packet])
            });
            out.push(delivery);
            out.extend(extra.into_iter().flatten().map(|packet| Delivery {
                recipient,
                bytes: wrap_application(&packet),
            }));
        }
        out
    }

    fn social_delivery(&self, message: SocialMessage) -> Option<Delivery> {
        let recipient = self.active_sessions.get(&message.recipient_guid)?;
        Some(Delivery {
            recipient: *recipient,
            bytes: wrap_application(&message.bytes),
        })
    }

    /// Drop a disconnected character's volatile group state and refresh every survivor's chat
    /// scope. Without the refresh, the last member of a dissolved group would keep its old party
    /// channel and could still address itself on channel 4.
    fn social_disconnect(&mut self, guid: u64) -> io::Result<Vec<Delivery>> {
        let messages = self.social.disconnect(guid).map_err(invalid_io)?;
        let deliveries = messages
            .into_iter()
            .filter_map(|message| self.social_delivery(message))
            .collect();
        self.refresh_group_ids();
        Ok(deliveries)
    }

    /// (R) Where each joined character stands, read from the authenticated player registry.
    /// `in_battle` stays false until the world tracks it; battles run in their own instance.
    fn map_spots(&self) -> BTreeMap<u64, MapSpot> {
        self.active_sessions
            .iter()
            .filter_map(|(guid, session)| {
                let player = self.players.get(*session)?;
                Some((
                    *guid,
                    MapSpot {
                        zone_id: player.zone.zone_id,
                        instance_id: player.zone.instance_id,
                        in_battle: false,
                        x: player.position[0],
                        z: player.position[2],
                    },
                ))
            })
            .collect()
    }

    fn refresh_group_ids(&mut self) {
        for chat in self.chat_sessions.values_mut() {
            chat.group_id = self.social.group_of(chat.character_guid).map(u64::from);
        }
    }

    fn chat_candidates(
        &self,
        sender: SessionId,
        channel: social_chat::ChatChannel,
    ) -> Vec<ChatSession> {
        // (R) Spatial channels share canonical membership; tells and party chat cross instances.
        let local = matches!(
            channel,
            social_chat::ChatChannel::Say
                | social_chat::ChatChannel::Yell
                | social_chat::ChatChannel::Area
        );
        let zone = self.players.get(sender).map(|player| player.zone);
        self.chat_sessions
            .iter()
            .filter(|(id, _)| {
                !local
                    || zone.is_some_and(|zone| {
                        self.players
                            .get(**id)
                            .is_some_and(|player| player.zone == zone)
                    })
            })
            .map(|(_, chat)| chat.clone())
            .collect()
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

/// W173 (R): the chat settings file sits beside the social list, `<guid>.chat.json`.
fn chat_settings_path(social: &Path) -> PathBuf {
    social.with_extension("").with_extension("chat.json")
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
    gateway.load_pet_roster(state_path)?;
    gateway.take_shared_fields();
    gateway.set_dev_tools(options.dev_tools);
    // Decision 1: account-store roles are server-owned, so builder/admin
    // `realm` commands run by role without --dev-tools (Studio stays off).
    gateway.set_role_commands(options.account_store.is_some());
    gateway.set_terrain(options.terrain.clone());
    gateway.set_world_dir(options.world.clone())?;
    gateway.set_stock_text(options.stock_text.clone());
    // (R) Without an account store the gateway can only bind loopback (checked
    // at startup), so this is the host's own solo dev session: admin only when
    // --dev-tools is on. Account-store sessions get their stored role at
    // character binding and default to player.
    gateway.set_role(if options.account_store.is_none() && options.dev_tools {
        osfr_tools::account_store::AccountRole::Admin
    } else {
        osfr_tools::account_store::AccountRole::Player
    });
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
        template: Option<&Gateway>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if request.crc_length == 0 {
            return Err("gateway requires a nonzero CRC trailer".into());
        }
        // An account-store session is a placeholder until login binds the
        // character (which loads current content then). Cloning the startup
        // template keeps an unauthenticated handshake from re-parsing all game
        // content on the event loop (Security Model S05).
        let mut gateway = if options.account_store.is_some() {
            match template {
                Some(template) => template.clone(),
                None => load_gateway_template(options)?,
            }
        } else {
            load_gateway(options)?
        };
        gateway.take_shared_fields();
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
            known_npcs: BTreeSet::new(),
            npc_visibility_revision: None,
            npc_view_position: None,
            npc_view_fields: Vec::new(),
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
            emote: social_chat::piece_093::EmoteContext::standing(),
            radial: PlayerRadial::default(),
            selected_character,
            spam_gate: SpamGate::default(),
            session_start: Instant::now(),
            position_saved_ms: None,
            position_dirty: false,
            last_activity: Instant::now(),
            packets: Bucket::full(PACKET_BUDGET.1, Instant::now()),
            bytes: Bucket::full(BYTE_BUDGET.1, Instant::now()),
            sent: 0,
            received: 0,
            trace,
        })
    }

    fn select_character_from_login(
        &mut self,
        options: &Options,
        message: &[u8],
        sessions: &SessionMap,
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
        // (R) First selected session owns the account and character, including
        // before Ready/Loaded. Reject duplicates before reading or writing a
        // save; a snapshot loaded here could otherwise outlive its active owner
        // and overwrite later progress on disconnect or shutdown.
        if sessions.values().any(|session| {
            session.selected_character
                && (session.account_guid == account.id || session.character_guid == guid)
        }) {
            return Err("account or character already has a selected session".into());
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
        self.gateway.bind_account_role(
            options.account_store.as_ref().unwrap(),
            record.owner_account_id,
        );
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
        membership: &mut World,
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
                membership,
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
                            membership,
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
        membership: &mut World,
        sessions: &mut SessionMap,
        transport: Option<(u16, Option<u16>, &'static str, bool)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.dispatch_message_inner(
            message, options, now_ms, world, membership, sessions, transport, false,
        )
    }

    fn dispatch_message_inner(
        &mut self,
        message: &[u8],
        options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
        transport: Option<(u16, Option<u16>, &'static str, bool)>,
        budget_checked: bool,
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
        if let Err(error) = self.select_character_from_login(options, message, sessions) {
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
        let children = self.gateway.ordered_messages(message)?;
        if !budget_checked {
            // Account-wide action budget runs before the gateway clones gameplay
            // or stages a bundle. Every application child is charged once; one
            // refusal drops the whole bundle without mutations or replies (R).
            let budget_requests = self.gateway.application_requests(message)?;
            let now = Instant::now();
            let mut security = world.security.lock().unwrap_or_else(|e| e.into_inner());
            let mut refused = false;
            for request in &budget_requests {
                if !security.budgets.allow(self.account_guid, request, now) {
                    refused = true;
                    break;
                }
            }
            if refused {
                security.record(
                    EventKind::BudgetRefusal,
                    Some(self.account_guid),
                    Some(self.id.0),
                );
                return Ok(());
            }
            drop(security);
        }
        self.gateway.restore_fishing_hours(
            world
                .fishing_hours
                .get(&self.character_guid)
                .cloned()
                .unwrap_or_default(),
        )?;
        let combat = children.iter().try_fold(false, |seen, child| {
            self.gateway
                .is_combat_request(child)
                .map(|is_combat| seen || is_combat)
        })?;
        // (R) Shared combat uses ordered World commits (W116). Other bundles
        // keep Gateway's staged character/inventory transaction intact.
        if children.len() > 1 && combat {
            for child in children {
                self.dispatch_message_inner(
                    &child, options, now_ms, world, membership, sessions, transport, true,
                )?;
            }
            return Ok(());
        }
        if combat {
            let mut security = world.security.lock().unwrap_or_else(|e| e.into_inner());
            if !security.allow_combat(self.account_guid, Instant::now()) {
                security.record(
                    EventKind::BudgetRefusal,
                    Some(self.account_guid),
                    Some(self.id.0),
                );
                return Ok(());
            }
        }
        for _ in 0..self.gateway.denied_dev_commands(message)? {
            world
                .security
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .record(
                    EventKind::DevCommandDenial,
                    options
                        .account_store
                        .as_ref()
                        .and(self.selected_character.then_some(self.account_guid)),
                    Some(self.id.0),
                );
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
        self.commit_outcome(
            staged_gateway,
            previous,
            outcome,
            world,
            membership,
            sessions,
            now_ms,
        )?;
        Ok(())
    }

    fn commit_outcome(
        &mut self,
        mut staged_gateway: Gateway,
        previous: CharacterState,
        mut outcome: GatewayOutcome,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
        now_ms: u64,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut staged_field = None;
        let mut field_packets = Vec::new();
        if let Some(action) = outcome.shared_field.take() {
            if let Some((instance, field)) = membership
                .field_for_player(self.character_guid, staged_gateway.state().zone_id)
                .filter(|_| staged_gateway.is_world_loaded())
            {
                let target = action.target();
                if target == self.character_guid || target == 0 || self.known_npcs.contains(&target)
                {
                    // ponytail: at most 64 rows in one instance; no World clone.
                    let mut field = field.clone();
                    let result = staged_gateway.handle_shared_field(
                        &action,
                        &mut field,
                        &self.known_npcs,
                        now_ms,
                    )?;
                    outcome.state_changed |= result.changed;
                    outcome.gameplay_changed |= result.changed;
                    for reply in result.replies {
                        if field_packet(&reply) {
                            field_packets.push(reply);
                        } else {
                            outcome.replies.push(wrap_application(&reply));
                        }
                    }
                    staged_field = Some((instance, field));
                }
            }
        }
        // W162: rides end with zoning, a fight or minigame, or a knockout (R).
        let zoned = outcome.began_zoning || previous.zone_id != staged_gateway.state().zone_id;
        let ended = staged_gateway.settle_rides(zoned);
        outcome.replies.extend(ended.replies);
        outcome.nearby.extend(ended.nearby);
        let ended = staged_gateway.settle_pets(zoned, now_ms)?;
        outcome.state_changed |= ended.state_changed;
        outcome.gameplay_changed |= ended.gameplay_changed;
        outcome.pets_changed |= ended.pets_changed;
        outcome.replies.extend(ended.replies);
        outcome.nearby.extend(ended.nearby);
        let current = staged_gateway.state().clone();
        let pending = staged_gateway.take_pending_transfer();
        let transfer =
            outcome.began_zoning || previous.zone_id != current.zone_id || pending.is_some();
        let prepared = (|| -> Result<_, Box<dyn std::error::Error>> {
            let old_instance = membership
                .player_location(self.character_guid)
                .map(|row| row.instance_id);
            let change = if staged_gateway.is_logged_in() {
                if pending.is_none() {
                    membership.register_runtime_zone(staged_gateway.runtime_zone()?)?;
                }
                let target = pending
                    .map(ZoneInstance::world_instance_id)
                    .transpose()?
                    .or_else(|| (!transfer).then_some(old_instance).flatten());
                Some(membership.prepare_membership(
                    self.character_guid,
                    current.zone_id,
                    target,
                    WorldPose {
                        position: [
                            current.position[0],
                            current.position[1],
                            current.position[2],
                            1.0,
                        ],
                        rotation: [
                            current.rotation[0],
                            current.rotation[1],
                            current.rotation[2],
                            1.0,
                        ],
                    },
                    staged_gateway.is_world_loaded(),
                    now_ms,
                )?)
            } else {
                None
            };
            let instance = ZoneInstance::from_world(
                current.zone_id,
                change
                    .as_ref()
                    .map(|change| change.instance_id())
                    .or(old_instance)
                    .unwrap_or(0),
            )?;
            // W111 gap: bind the existing solo run before Ready can emit 41/*.
            let bound = staged_gateway.bind_run_instance(instance.world_instance_id()?)?;
            outcome.state_changed |= bound;
            let mut staged_world = world.clone();
            let cutoff = now_ms.saturating_sub(osfr_tools::fishing_service::HOUR_WINDOW_MS);
            staged_world.fishing_hours.retain(|_, hours| {
                hours.retain(|at| *at > cutoff);
                !hours.is_empty()
            });
            if let Some(hours) = staged_gateway.fishing_hour_history() {
                let guid = staged_gateway.state().guid;
                if hours.is_empty() {
                    staged_world.fishing_hours.remove(&guid);
                } else {
                    staged_world.fishing_hours.insert(guid, hours);
                }
            }
            let mut deliveries = staged_world.ride_fanout(self.id, &outcome.nearby);
            if transfer && staged_world.joined.contains(&self.id) {
                deliveries.extend(staged_world.leave_player(self.id));
            }
            // A state-only change (e.g. jump or crouch flags) is relayed like a move.
            let (state, unknown) = staged_gateway.movement_bytes();
            let moved = previous.position != current.position
                || previous.rotation != current.rotation
                || (state, unknown) != self.gateway.movement_bytes();
            // W143: what strangers may see on Inspect. Refreshed on any gameplay
            // change (equipment, job) and published at join; removed on leave.
            // T027: the same snapshot feeds AddPc, so a viewer who arrives after
            // the change sees the new outfit and job. Current viewers were told
            // by the gameplay 35/6 or 35/7 fanout.
            if outcome.gameplay_changed && staged_world.joined.contains(&self.id) {
                refresh_public_look(&mut staged_world, current.guid, &staged_gateway);
                let presentation = staged_gateway.player_presentation(self.is_member)?;
                staged_world
                    .players
                    .set_presentation(self.id, presentation)?;
                deliveries.extend(refresh_trade_player(
                    &mut staged_world,
                    self.id,
                    &staged_gateway,
                ));
            }
            if outcome.entered_world
                && staged_gateway.is_world_loaded()
                && !staged_world.joined.contains(&self.id)
            {
                let look = staged_gateway.player_presentation(self.is_member)?;
                deliveries.extend(staged_world.join_player(self, &current, look, instance)?);
                refresh_public_look(&mut staged_world, current.guid, &staged_gateway);
                deliveries.extend(refresh_trade_player(
                    &mut staged_world,
                    self.id,
                    &staged_gateway,
                ));
            } else if moved && staged_world.joined.contains(&self.id) {
                let messages = staged_world
                    .players
                    .update_pose_with_state(
                        self.id,
                        instance,
                        current.position,
                        heading(current.rotation),
                        state,
                        unknown,
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
                self.emote = staged_gateway.emote_context(now_ms);
                match staged_gateway.ride_view() {
                    Some(view) => {
                        staged_world.players.set_mount(self.id, view.tail());
                        staged_world.rides.insert(current.guid, view);
                    }
                    None => {
                        staged_world.players.set_mount(self.id, MountTail::NONE);
                        staged_world.rides.remove(&current.guid);
                    }
                }
                match staged_gateway.pet_view() {
                    Some(view) => {
                        staged_world.pets.insert(current.guid, view);
                    }
                    None => {
                        staged_world.pets.remove(&current.guid);
                    }
                }
                for application in &outcome.applications {
                    if osfr_tools::chat::request(application, self.character_guid)
                        .ok()
                        .flatten()
                        .is_some_and(|text| text.trim().to_ascii_lowercase().starts_with("realm "))
                    {
                        continue; // Gameplay's developer command path ran first.
                    }
                    deliveries.extend(self.route_application(
                        application,
                        &mut staged_world,
                        now_ms,
                    ));
                    staged_world.refresh_group_ids();
                }
            }
            staged_world.social.expire_group_invites(now_ms);
            // W163: a trade ends when a player walks out of reach, changes zone instance (a
            // battle runs in its own), leaves, or lets it run out.
            deliveries.extend(trade_deliveries(staged_world.trades.maintain(
                &staged_world.players,
                &staged_world.social,
                now_ms,
            )));

            // (R) A pose-only change (no gameplay change, zone transfer or run
            // binding) is written at most every POSITION_SAVE_MS; `tick` and
            // logout flush it. One save per accepted pose would be thousands of
            // file writes a second at 1,000 players.
            let deferred = outcome.state_changed
                && !outcome.gameplay_changed
                && !transfer
                && !bound
                && self
                    .position_saved_ms
                    .is_some_and(|saved| now_ms < saved.saturating_add(POSITION_SAVE_MS));
            if outcome.state_changed && !deferred {
                save(&self.state_path, &staged_gateway.snapshot())?;
                if outcome.pets_changed {
                    staged_gateway.save_pet_roster(&self.state_path)?;
                }
            }
            persist_social_changes(world, &staged_world)?;
            staged_world
                .chat_settings
                .save_changed(&world.chat_settings)?;
            let deliveries = staged_world.with_rides(deliveries);
            let deliveries = staged_world.with_pets(deliveries);
            Ok((staged_world, deliveries, change, deferred))
        })();
        let (staged_world, deliveries, change, deferred) = match prepared {
            Ok(staged) => staged,
            // (R) A precommit transfer refusal keeps the live session/location/members.
            // Postcommit queue errors below still disconnect; they cannot roll back saves.
            Err(_) if transfer => return Ok(false),
            Err(error) => return Err(error),
        };
        if let Some(change) = change {
            membership.commit_membership(change);
        }
        if let Some((instance, field)) = staged_field {
            membership.install_field(instance, field);
            membership.queue_field_packets(instance, field_packets);
        }
        if transfer {
            self.known_npcs.clear();
            self.npc_visibility_revision = None;
            self.npc_view_position = None;
            self.npc_view_fields.clear();
            self.radial.reset();
        }
        self.gateway = staged_gateway;
        *world = staged_world;
        self.character_guid = self.gateway.state().guid;
        if outcome.state_changed {
            self.position_dirty = deferred;
            if !deferred {
                self.position_saved_ms = Some(now_ms);
            }
        }

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
        for delivery in settle_trades(self, sessions, world)? {
            queue_delivery(self, sessions, world, delivery)?;
        }
        if outcome.state_changed && !deferred {
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
        if let Some(heal) = outcome.group_heal {
            self.heal_group(heal, world, membership, sessions)?;
        }
        Ok(true)
    }

    /// 26/19 select, 26/10 choice on an open player menu, 149/2 close (W143).
    /// A refusal is a silent drop (R): nothing is sent, so ids cannot be probed.
    fn route_player_radial(
        &mut self,
        inner: &[u8],
        world: &mut SharedWorld,
        now_ms: u64,
    ) -> Vec<Delivery> {
        let looks = &world.public_looks;
        let me = self.character_guid;
        let replies = if inner == [149, 0, 2] {
            self.radial.stop_inspect(inner).map(|()| Vec::new())
        } else if inner.starts_with(&[26, 0, 19, 0]) {
            // W163: Trade is offered only where an invite would be admitted right now.
            let trades = &world.trades;
            self.radial
                .select_menu(&world.players, self.id, inner, |guid| MenuOffer {
                    inspect: looks.contains_key(&guid),
                    trade: trades.can_invite(&world.players, &world.social, me, guid),
                })
        } else {
            match self
                .radial
                .choose_entry(&world.players, self.id, inner, |guid| {
                    looks.get(&guid).cloned()
                }) {
                Ok(Some(Chosen::Trade(target))) => {
                    // The service re-checks everything; a refusal is a silent drop (R).
                    let invited =
                        world
                            .trades
                            .invite(&world.players, &world.social, me, target, now_ms);
                    return trade_deliveries(invited.unwrap_or_default());
                }
                Ok(Some(Chosen::Inspect(packet))) => Ok(vec![packet]),
                Ok(None) => Ok(Vec::new()),
                Err(error) => Err(error),
            }
        };
        replies
            .unwrap_or_default()
            .iter()
            .map(|reply| Delivery {
                recipient: self.id,
                bytes: wrap_application(reply),
            })
            .collect()
    }

    /// The six client trade requests (59/3, 6, 7, 8, 9, 10). The actor is `self.character_guid`;
    /// the body never names anyone. A settlement is queued for `settle_trades`, not committed here.
    fn route_trade(&mut self, inner: &[u8], world: &mut SharedWorld, now_ms: u64) -> Vec<Delivery> {
        let Ok(outcome) = world.trades.handle(
            &world.players,
            &world.social,
            self.character_guid,
            inner,
            now_ms,
        ) else {
            return Vec::new(); // dropped without a reply (R)
        };
        world.trade_settlements.extend(outcome.settlement);
        trade_deliveries(outcome.messages)
    }

    fn route_application(
        &mut self,
        inner: &[u8],
        world: &mut SharedWorld,
        now_ms: u64,
    ) -> Vec<Delivery> {
        if inner.starts_with(&[26, 0, 19, 0])
            || inner == [149, 0, 2]
            || (inner.starts_with(&[26, 0, 10, 0]) && self.radial.menu_target().is_some())
        {
            return self.route_player_radial(inner, world, now_ms);
        }
        if inner.starts_with(&[59, 0]) {
            return self.route_trade(inner, world, now_ms);
        }
        if matches!(inner, [26, 0, 18 | 32 | 33, 0, ..]) {
            // W173: the client's channel switches and bubble colour. Keyed by the
            // server-bound character; a refusal is a silent drop with no reply (R).
            let _ = world.chat_settings.handle(self.character_guid, inner);
            return Vec::new();
        }
        // W178: the per-character Gameplay stage owns titles and the claim window;
        // no shared-world delivery is composed here.
        if matches!(inner, [108, 0, 1 | 3 | 5, 0, ..]) || matches!(inner, [152, 0, 4 | 5, 0, ..]) {
            return Vec::new();
        }
        if inner.starts_with(&[15, 0, 1, 0]) {
            return self.route_typed_chat(inner, world);
        }
        if inner.starts_with(&[67, 0, 2, 0]) || inner.starts_with(&[67, 0, 3, 0]) {
            return self.route_quick_chat(inner, world);
        }
        if let Some(request) = parse_location_request(inner) {
            return self.route_location(request, world);
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
                    .invite_to_group(self.character_guid, &name, &lookup, now_ms)
            }
            SocialRequest::GroupLeave => world.social.leave_group(self.character_guid),
            SocialRequest::GroupResponse(accepted) => {
                world
                    .social
                    .respond_group_invite(self.character_guid, accepted, now_ms)
            }
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

    /// 26/21, 40/14 and 40/16. The actor is `self.character_guid`; the body never names anyone.
    fn route_location(
        &mut self,
        request: LocationRequest,
        world: &mut SharedWorld,
    ) -> Vec<Delivery> {
        let spots = world.map_spots();
        let spot_of = |guid| spots.get(&guid).copied();
        let guid = self.character_guid;
        let messages = match request {
            LocationRequest::FriendPositions => world
                .social
                .friend_positions(guid, spot_of)
                .map(|reply| reply.into_iter().collect()),
            LocationRequest::ChangingJob(changing) => {
                world.social.group_changing_job(guid, changing)
            }
            LocationRequest::MapPing { x, z } => world.social.group_map_ping(guid, x, z, spot_of),
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
        let sessions = world.chat_candidates(self.id, request.channel);
        let ignores = world.social.chat_ignore_lists();
        let recipients =
            match social_chat::resolve_recipients(&request, self.id.0, &sessions, &ignores) {
                Ok(recipients) => recipients,
                Err(error) => return chat_error_delivery(error, self.id),
            };
        let Some(sender) = world.chat_sessions.get(&self.id).cloned() else {
            return Vec::new();
        };
        // W173: drop listeners that switched this channel off (never the sender's echo).
        let recipients =
            world
                .chat_settings
                .listeners(request.channel, self.id.0, &sessions, recipients);
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
        let phrase_id = match &request {
            QuickChatRequest::Channel { phrase_id, .. }
            | QuickChatRequest::Tell { phrase_id, .. } => *phrase_id,
        };
        // (R) Animated phrases follow the recovered emote restrictions; a refusal
        // is a silent drop, like the client's disabled emotes button.
        if self
            .quick_chat
            .authorize_emote(phrase_id, self.emote)
            .is_err()
        {
            return Vec::new();
        }
        let channel = match &request {
            QuickChatRequest::Channel { channel, .. } => *channel,
            QuickChatRequest::Tell { .. } => social_chat::ChatChannel::Tell,
        };
        let sessions = world.chat_candidates(self.id, channel);
        let ignores = world.social.chat_ignore_lists();
        let recipients = match social_chat::resolve_quick_chat_recipients(
            &request, self.id.0, &sessions, &ignores,
        ) {
            Ok(recipients) => recipients,
            Err(error) => return chat_error_delivery(error, self.id),
        };
        // W173: drop listeners that switched this channel off (never the sender's echo).
        let recipients = world
            .chat_settings
            .listeners(channel, self.id.0, &sessions, recipients);
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

    fn queue_shared_message(&mut self, message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(trace) = &self.trace {
            trace_application_out(trace, self.id.0, self.peer, message, &[], Some(self.id.0))?;
        }
        if self.recent_on {
            self.note_packet("out", message);
        }
        self.sender.queue_message(message)?;
        Ok(())
    }

    /// W158: a big heart heals the rest of the picker's group, once per kill (R).
    /// Members must be in the same zone and instance and not knocked out (the
    /// engine never revives). Each member's state is saved before the heal shows.
    fn heal_group(
        &self,
        heal: osfr_tools::encounter::GroupHeal,
        world: &SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(group) = world.social.group_of(self.character_guid) else {
            return Ok(());
        };
        if !membership.claim_group_heal(heal.kill) {
            return Ok(());
        }
        let members = world
            .social
            .group_members(group)
            .map(<[u64]>::to_vec)
            .unwrap_or_default();
        let zone = self.gateway.state().zone_id;
        let instance = membership
            .player_location(self.character_guid)
            .map(|l| l.instance_id);
        for guid in members.into_iter().filter(|g| *g != self.character_guid) {
            let Some(session) = world
                .active_sessions
                .get(&guid)
                .and_then(|id| world.session_keys.get(id))
                .and_then(|key| sessions.get_mut(key))
            else {
                continue;
            };
            let same_place = session.gateway.state().zone_id == zone
                && membership.player_location(guid).map(|l| l.instance_id) == instance;
            if !same_place {
                continue;
            }
            let mut staged = session.gateway.clone();
            let healed = staged.receive_group_heal(heal.percent)?;
            if healed.changed {
                save(&session.state_path, &staged.snapshot())?;
                session.gateway = staged;
            }
            for reply in healed.replies {
                session.queue_shared_message(&wrap_application(&reply))?;
            }
        }
        Ok(())
    }

    fn tick(
        &mut self,
        _options: &Options,
        now_ms: u64,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let previous = self.gateway.state().clone();
        let mut staged_gateway = self.gateway.clone();
        let outcome = staged_gateway.tick_at(now_ms)?;
        self.commit_outcome(
            staged_gateway,
            previous,
            outcome,
            world,
            membership,
            sessions,
            now_ms,
        )?;
        self.flush_position(now_ms)
    }

    /// Write a deferred pose once `POSITION_SAVE_MS` has passed since the last save.
    fn flush_position(&mut self, now_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        let due = self
            .position_saved_ms
            .is_none_or(|saved| now_ms >= saved.saturating_add(POSITION_SAVE_MS));
        if self.position_dirty && due {
            save(&self.state_path, &self.gateway.snapshot())?;
            self.position_dirty = false;
            self.position_saved_ms = Some(now_ms);
        }
        Ok(())
    }
}

/// (R) Longest a pose-only change waits before it is written (crash loss bound).
const POSITION_SAVE_MS: u64 = 5_000;

fn trade_deliveries(messages: Vec<TradeMessage>) -> Vec<Delivery> {
    messages
        .into_iter()
        .map(|message| Delivery {
            recipient: message.recipient,
            bytes: wrap_application(&message.bytes),
        })
        .collect()
}

/// (W163) Keep the trade service's copy of this character's inventory current. A character
/// whose inventory does not validate is not tradable: it is unregistered, which also ends a
/// trade it is in.
fn refresh_trade_player(
    world: &mut SharedWorld,
    session: SessionId,
    gateway: &Gateway,
) -> Vec<Delivery> {
    let Some(player) = world.players.get(session).cloned() else {
        return Vec::new();
    };
    let guid = player.character_guid;
    match gateway.trade_inventory() {
        Some(inventory) => {
            if world.trades.inventory(guid).is_some() {
                // A snapshot that fails the service's checks keeps the previous one (R).
                let _ = world.trades.sync_inventory(guid, inventory);
            } else {
                let _ = world.trades.register_player(&player, inventory);
            }
            // A battle or dungeon run ends a trade and blocks a new one.
            trade_deliveries(world.trades.set_away(guid, gateway.in_encounter()))
        }
        None => trade_deliveries(world.trades.unregister_player(guid)),
    }
}

/// One side of a trade journal: a character save path and the complete snapshot to write.
#[derive(serde::Serialize, serde::Deserialize)]
struct TradeJournalEntry {
    path: PathBuf,
    snapshot: osfr_tools::game_store::Snapshot,
}

const TRADE_JOURNAL_EXTENSION: &str = "trade-journal";

/// The live session playing `guid`: the session being dispatched, or one taken out of the map.
fn trade_party<'a>(
    guid: u64,
    me: &'a mut Session,
    taken: &'a mut [(SessionKey, Session)],
) -> Option<&'a mut Session> {
    if me.character_guid == guid {
        return Some(me);
    }
    taken
        .iter_mut()
        .map(|(_, session)| session)
        .find(|session| session.character_guid == guid)
}

/// Lead (security): the all-or-nothing commit of a settled trade (W163 settlement).
///
/// 1. Both players must be live in this gateway and their inventories must still equal the
///    settlement's `before` exactly; otherwise nothing moves (the trade cancels).
/// 2. Each side's `after` is adopted on a staged copy of its gateway, which validates it.
/// 3. A journal holding both complete snapshots is written to a temporary file, synced and
///    renamed into place, so it is either whole or absent.
/// 4. Both character saves are written, then the journal is removed. A crash or write error in
///    between leaves the journal, and the next start writes both saves from it
///    (`recover_trade_journals`), so nobody keeps both sides or loses both.
/// 5. Both live sessions adopt their staged gateways and are sent their inventory updates.
fn commit_trade(
    settlement: &TradeSettlement,
    me: &mut Session,
    sessions: &mut SessionMap,
    world: &SharedWorld,
) -> Result<Option<Vec<Delivery>>, Box<dyn std::error::Error>> {
    let mut taken: Vec<(SessionKey, Session)> = Vec::new();
    for side in &settlement.sides {
        if side.guid == me.character_guid {
            continue;
        }
        let key = world
            .active_sessions
            .get(&side.guid)
            .and_then(|id| world.session_keys.get(id))
            .copied();
        match key.and_then(|key| sessions.remove(&key).map(|session| (key, session))) {
            Some(pair) => taken.push(pair),
            None => {
                sessions.extend(taken);
                return Ok(None);
            }
        }
    }
    let result = commit_trade_parties(settlement, me, &mut taken);
    sessions.extend(taken);
    result
}

fn commit_trade_parties(
    settlement: &TradeSettlement,
    me: &mut Session,
    taken: &mut [(SessionKey, Session)],
) -> Result<Option<Vec<Delivery>>, Box<dyn std::error::Error>> {
    let [first, second] = &settlement.sides;
    if first.guid == second.guid {
        return Ok(None);
    }
    // 1-2. Still exactly what was agreed, and the result validates.
    let mut staged = Vec::with_capacity(2);
    for side in &settlement.sides {
        let Some(party) = trade_party(side.guid, me, taken) else {
            return Ok(None);
        };
        if !party.selected_character || party.gateway.inventory_state() != Some(&side.before) {
            return Ok(None);
        }
        let mut gateway = party.gateway.clone();
        let Ok(replies) = gateway.adopt_trade_inventory(side.after.clone()) else {
            return Ok(None);
        };
        staged.push((
            side.guid,
            party.id,
            party.state_path.clone(),
            gateway,
            replies,
        ));
    }
    // 3. The journal: whole or absent.
    let entries: Vec<TradeJournalEntry> = staged
        .iter()
        .map(|(_, _, path, gateway, _)| TradeJournalEntry {
            path: path.clone(),
            snapshot: gateway.snapshot(),
        })
        .collect();
    let directory = staged[0]
        .2
        .parent()
        .ok_or("character save has no directory")?
        .to_path_buf();
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let journal = directory.join(format!(
        "trade-{}-{stamp}.{TRADE_JOURNAL_EXTENSION}",
        settlement.session_id
    ));
    let pending = journal.with_extension("tmp");
    {
        let mut file = fs::File::create(&pending)?;
        std::io::Write::write_all(&mut file, &serde_json::to_vec(&entries)?)?;
        file.sync_all()?;
    }
    fs::rename(&pending, &journal)?;
    // 4. Both saves, then the journal goes.
    for entry in &entries {
        save(&entry.path, &entry.snapshot)?;
    }
    fs::remove_file(&journal)?;
    // 5. Install and tell each player.
    let mut deliveries = Vec::new();
    for (guid, id, _, gateway, replies) in staged {
        if let Some(party) = trade_party(guid, me, taken) {
            party.gateway = gateway;
        }
        deliveries.extend(replies.iter().map(|reply| Delivery {
            recipient: id,
            bytes: wrap_application(reply),
        }));
    }
    Ok(Some(deliveries))
}

/// Finish any trade a crash interrupted: write every save a complete journal names, then
/// remove the journal. Rewriting a save that already holds the same snapshot is harmless.
/// A `.tmp` journal was never completed, so that trade never started writing: delete it.
fn recover_trade_journals(directory: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    let mut recovered = 0;
    for entry in entries {
        let path = entry?.path();
        let is_trade = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("trade-"));
        if !is_trade {
            continue;
        }
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("tmp") => fs::remove_file(&path)?,
            Some(TRADE_JOURNAL_EXTENSION) => {
                let journal: Vec<TradeJournalEntry> = serde_json::from_slice(&fs::read(&path)?)?;
                for side in &journal {
                    save(&side.path, &side.snapshot)?;
                }
                fs::remove_file(&path)?;
                event(json!({"event":"trade_journal_recovered","sides":journal.len()}));
                recovered += 1;
            }
            _ => {}
        }
    }
    Ok(recovered)
}

/// Commit the settlements queued while staging, now that the staged world is installed, and
/// tell the trade service and both players how each went.
fn settle_trades(
    me: &mut Session,
    sessions: &mut SessionMap,
    world: &mut SharedWorld,
) -> Result<Vec<Delivery>, Box<dyn std::error::Error>> {
    let mut deliveries = Vec::new();
    for settlement in std::mem::take(&mut world.trade_settlements) {
        let committed = commit_trade(&settlement, me, sessions, world)?;
        let ok = committed.is_some();
        deliveries.extend(committed.unwrap_or_default());
        deliveries.extend(trade_deliveries(
            world
                .trades
                .settled(settlement.session_id, ok)
                .unwrap_or_default(),
        ));
    }
    Ok(deliveries)
}

/// (R) Inspect shows only unlocked jobs with ranks and the active job's worn
/// items; no look (no profiles loaded) means no Inspect entry.
fn refresh_public_look(world: &mut SharedWorld, guid: u64, gateway: &Gateway) {
    match gateway.public_look() {
        Some(look) => {
            world.public_looks.insert(guid, look);
        }
        None => {
            world.public_looks.remove(&guid);
        }
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

/// The character and whether it was added (true) or removed (false), for the registry's AddPc
/// (35/1) and RemovePlayer (35/3) tunnel frames; `None` for every other delivery.
fn rider_event(frame: &[u8]) -> Option<(u64, bool)> {
    let inner = frame.get(7..)?;
    match inner.get(..4)? {
        [35, 0, 1, 0] => Some((u64::from_le_bytes(inner.get(4..12)?.try_into().ok()?), true)),
        // 35/3 carries a u16 submode before the GUID.
        [35, 0, 3, 0] => Some((
            u64::from_le_bytes(inner.get(6..14)?.try_into().ok()?),
            false,
        )),
        _ => None,
    }
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

// Admission and budgets (Security Model L6). (R) starting values: the native
// client peaked at 41 datagrams/s and 4.4 KB/s in run 20261016T045046Z.
// Loopback (the host's own bots and client) is exempt from the per-IP limits.
const MAX_SESSIONS: usize = 16;
const MAX_SESSIONS_PER_IP: usize = 4;
const HANDSHAKES_PER_IP: (f64, f64) = (5.0, 10.0); // per second, burst
const LOGIN_DEADLINE: Duration = Duration::from_secs(5);
const PACKET_BUDGET: (f64, f64) = (200.0, 400.0);
const BYTE_BUDGET: (f64, f64) = (256.0 * 1024.0, 1024.0 * 1024.0);

pub struct Server {
    options: Options,
    /// Account-store mode: the pre-login gateway each handshake clones (S05).
    template: Option<Gateway>,
    handshakes: IpLimiter,
    refused: u64,
    refusal_logged: Option<Instant>,
    security_flushed: Instant,
    next_tick_ms: u64,
    retired_handshakes: BTreeMap<SessionKey, Instant>,
    socket: UdpSocket,
    sessions: SessionMap,
    world: SharedWorld,
    // (R) Single writer, deliberately outside SharedWorld staging.
    membership: World,
    start: Instant,
    generation: u32,
    trace: Option<trace::Trace>,
    studio: Option<TcpListener>,
    // Studio asset browser: index files loaded on first search.
    studio_assets: BTreeMap<String, Vec<serde_json::Value>>,
}

impl Server {
    fn tick_shared_fields(&mut self, now_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
        for instance in self.membership.field_instances() {
            self.tick_shared_instance(instance, now_ms)?;
        }
        Ok(())
    }

    fn tick_shared_instance(
        &mut self,
        instance: i32,
        now_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(mut field) = self.membership.field(instance).cloned() else {
            return Ok(());
        };
        let mut active = Vec::new();
        let mut quests = BTreeSet::new();
        for (key, session) in &self.sessions {
            if self
                .membership
                .player_location(session.character_guid)
                .is_some_and(|l| l.instance_id == instance && l.ready)
                && session.gateway.is_world_loaded()
            {
                if let Some(player) = session.gateway.field_participant() {
                    if let Some(game) = session.gateway.gameplay() {
                        quests.extend(game.active_field_quests());
                    }
                    active.push((*key, player));
                }
            }
        }
        field.retain_casters(&active.iter().map(|(_, player)| player.guid).collect());
        field.set_active_quests(&quests);
        // (R) Current poses/caps first. Hold only already-known corpses through
        // lethal HP delivery, then remove once. New observers get current HP,
        // never a replay of action deltas committed before their AddNpc.
        let previous: BTreeMap<_, _> = self
            .sessions
            .values()
            .map(|s| (s.id, s.known_npcs.clone()))
            .collect();
        self.sync_field_views(instance, &field, true)?;
        for (key, player) in &mut active {
            player.known_npcs = Some(self.sessions[key].known_npcs.clone());
        }
        let committed = self.membership.take_field_packets(instance);
        // (R) Only a lethal committed HP update needs the held corpse removed.
        // Idle ticks already reconciled poses/readiness above; do not rank the
        // entire passive population twice when simulation changes nothing.
        let mut resync = committed.iter().any(|packet| {
            packet.starts_with(&[35, 0, 5, 0])
                && packet.get(12..16) == Some(0_u32.to_le_bytes().as_slice())
        });
        let deliveries = self
            .sessions
            .values()
            .flat_map(|recipient| {
                let retained = recipient
                    .known_npcs
                    .intersection(&previous[&recipient.id])
                    .copied()
                    .collect();
                field_packets_for(
                    recipient,
                    &self.membership,
                    &self.world.players,
                    instance,
                    &committed,
                    &retained,
                )
            })
            .collect::<Vec<_>>();
        self.queue_field_deliveries(deliveries)?;
        let Some(elapsed) = field.due(now_ms) else {
            self.membership.install_field(instance, field.clone());
            if resync {
                self.sync_field_views(instance, &field, false)?;
            }
            return Ok(());
        };
        let mut staged = BTreeMap::new();
        let mut changed = BTreeSet::new();
        let mut private = BTreeMap::<SessionKey, Vec<Vec<u8>>>::new();
        let mut packets = Vec::new();
        for (key, _) in &active {
            let mut gateway = self.sessions[key].gateway.clone();
            let out = gateway.tick_shared_effects(&mut field, now_ms)?;
            resync |= out.changed;
            if out.changed {
                changed.insert(*key);
            }
            for reply in out.replies {
                if field_packet(&reply) {
                    packets.push(reply);
                } else {
                    private.entry(*key).or_default().push(reply);
                }
            }
            staged.insert(*key, gateway);
        }
        if let Some((driver, _)) = active.first() {
            let players = active.iter().map(|(_, p)| p.clone()).collect::<Vec<_>>();
            let (out, hits) = staged
                .get_mut(driver)
                .unwrap()
                .tick_shared_motion(&mut field, &players, now_ms, elapsed)?;
            resync |= out.changed;
            packets.extend(out.replies.into_iter().filter(|p| field_packet(p)));
            for (key, player) in &active {
                let damage = hits
                    .iter()
                    .filter(|(guid, _)| *guid == player.guid)
                    .map(|(_, damage)| *damage)
                    .fold(0_u32, u32::saturating_add);
                let engaged = field.enemies_targeting(player.guid);
                let out = staged
                    .get_mut(key)
                    .unwrap()
                    .receive_field_damage(damage, now_ms, engaged)?;
                if out.changed {
                    changed.insert(*key);
                }
                private.entry(*key).or_default().extend(out.replies);
            }
        }
        // NPC/resource changes and packets publish only after affected saves.
        for key in &changed {
            save(&self.sessions[key].state_path, &staged[key].snapshot())?;
        }
        for (key, gateway) in staged {
            self.sessions.get_mut(&key).unwrap().gateway = gateway;
        }
        self.membership.install_field(instance, field.clone());
        for (key, replies) in private {
            for reply in replies {
                self.sessions
                    .get_mut(&key)
                    .unwrap()
                    .queue_shared_message(&wrap_application(&reply))?;
            }
        }
        let deliveries = self
            .sessions
            .values()
            .flat_map(|recipient| {
                field_packets_for(
                    recipient,
                    &self.membership,
                    &self.world.players,
                    instance,
                    &packets,
                    &recipient.known_npcs,
                )
            })
            .collect::<Vec<_>>();
        self.queue_field_deliveries(deliveries)?;
        if resync {
            self.sync_field_views(instance, &field, false)?;
        }
        Ok(())
    }

    fn queue_field_deliveries(
        &mut self,
        deliveries: Vec<Delivery>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for delivery in deliveries {
            if let Some(session) = self
                .world
                .session_keys
                .get(&delivery.recipient)
                .and_then(|key| self.sessions.get_mut(key))
            {
                session.queue_shared_message(&delivery.bytes)?;
            }
        }
        Ok(())
    }

    fn sync_field_views(
        &mut self,
        instance: i32,
        field: &osfr_tools::encounter::SharedFieldState,
        hold_deaths: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let candidates = field.interest_candidates(hold_deaths);
        let Some(row) = self.membership.instance(instance) else {
            return Ok(());
        };
        let zone = ZoneInstance::from_world(row.zone_id, instance)?;
        // Reuse W96's grid for candidates, never the capped PC observer sets.
        let nearby: BTreeSet<_> = candidates
            .iter()
            .flat_map(|npc| self.world.players.nearby_sessions(zone, npc.position))
            .collect();
        for session in self.sessions.values_mut() {
            let valid = self
                .membership
                .player_location(session.character_guid)
                .is_some_and(|l| l.instance_id == instance && l.ready)
                && session.gateway.is_world_loaded();
            if !valid {
                continue;
            }
            let fields: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|npc| {
                    nearby.contains(&session.id)
                        && (field.health(npc.guid) != Some(0)
                            || session.known_npcs.contains(&npc.guid))
                })
                .collect();
            let revision = session
                .gateway
                .gameplay()
                .map_or(0, |game| game.npc_visibility_revision());
            if session.npc_visibility_revision == Some(revision)
                && session.npc_view_position == Some(session.gateway.state().position)
                && session.npc_view_fields == fields
            {
                continue;
            }
            let (wanted, packets) = session
                .gateway
                .gameplay()
                .map(|game| {
                    let revision = game.npc_visibility_revision();
                    let mut known = session.known_npcs.clone();
                    let mut refreshed = Vec::new();
                    if session
                        .npc_visibility_revision
                        .is_some_and(|old| old != revision)
                    {
                        // ponytail: admin placement edits resend at most 64
                        // passive actors; no second per-actor revision cache.
                        known.retain(|guid| {
                            if field.position(*guid).is_some() {
                                return true;
                            }
                            refreshed.push(osfr_tools::encounter::SharedFieldState::removal(*guid));
                            false
                        });
                    }
                    let (wanted, packets) =
                        game.shared_npc_view(session.gateway.state(), &fields, &known)?;
                    refreshed.extend(packets);
                    session.npc_visibility_revision = Some(revision);
                    Ok::<_, io::Error>((wanted, refreshed))
                })
                .transpose()?
                .unwrap_or_default();
            for reply in packets {
                session.queue_shared_message(&wrap_application(&reply))?;
            }
            for guid in wanted
                .difference(&session.known_npcs)
                .copied()
                .collect::<Vec<_>>()
            {
                for reply in field.initialization(guid)? {
                    session.queue_shared_message(&wrap_application(&reply))?;
                }
            }
            session.known_npcs = wanted;
            session.npc_view_position = Some(session.gateway.state().position);
            session.npc_view_fields = fields;
        }
        Ok(())
    }

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
        if options.account_store.is_some() {
            // Finish any trade a crash interrupted before a player can load either save.
            let saves = character_state_path(&options.state, 0);
            if let Some(directory) = saves.parent() {
                recover_trade_journals(directory)?;
            }
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
        let world = SharedWorld::new();
        if let Some(path) = &options.security_log {
            // Logging is best effort: an unusable file disables it, never the server.
            match SecurityLog::open(path) {
                Ok(log) => world.security.lock().unwrap().log = Some(log),
                Err(error) => {
                    event(json!({"event":"security_log_unavailable","reason":error.to_string()}))
                }
            }
        }
        let mut membership = World::runtime();
        let mut seed = load_gateway_template(&options)?;
        if let Some(fields) = seed.take_shared_fields() {
            membership.install_field_seed(0, fields);
        }
        let template = options.account_store.is_some().then_some(seed);
        Ok(Self {
            template,
            options,
            socket,
            sessions: BTreeMap::new(),
            next_tick_ms: 0,
            retired_handshakes: BTreeMap::new(),
            world,
            membership,
            start: Instant::now(),
            generation: SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos(),
            trace: None,
            studio,
            studio_assets: BTreeMap::new(),
            handshakes: IpLimiter::new(HANDSHAKES_PER_IP.0, HANDSHAKES_PER_IP.1, 1024),
            refused: 0,
            refusal_logged: None,
            security_flushed: Instant::now(),
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Count refused traffic and log a summary at most every 10 s, so a flood
    /// cannot grow the log.
    fn refuse(&mut self, reason: &'static str) {
        self.refused += 1;
        self.world
            .security
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .record(EventKind::AdmissionRefusal, None, None);
        if self
            .refusal_logged
            .is_none_or(|at| at.elapsed() >= Duration::from_secs(10))
        {
            event(json!({"event":"admission_refused","reason":reason,"count":self.refused}));
            self.refused = 0;
            self.refusal_logged = Some(Instant::now());
        }
    }

    /// Write any batched security counts (timer and shutdown).
    pub fn flush_security(&mut self) {
        let mut security = self
            .world
            .security
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(Err(error)) = security.log.as_mut().map(SecurityLog::flush) {
            eprintln!("security log: {error}");
        }
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }

    #[cfg(test)]
    pub fn test_queues_idle(&self) -> bool {
        self.sessions.values().all(|s| s.sender.is_idle())
    }
    #[cfg(test)]
    pub fn test_field_health(&self, guid: u64) -> Option<u32> {
        self.test_field_health_in(0, guid)
    }
    #[cfg(test)]
    pub fn test_field_health_in(&self, instance: i32, guid: u64) -> Option<u32> {
        self.membership.field(instance)?.health(guid)
    }
    #[cfg(test)]
    pub fn test_field_instances(&self) -> Vec<i32> {
        self.membership.field_instances()
    }
    #[cfg(test)]
    pub fn test_known_npcs(&self, guid: u64) -> BTreeSet<u64> {
        let id = self.world.active_sessions[&guid];
        self.sessions[&self.world.session_keys[&id]]
            .known_npcs
            .clone()
    }
    #[cfg(test)]
    pub fn test_field_clock(&self, guid: u64) -> (u64, Option<u64>) {
        self.test_field_clock_in(0, guid)
    }
    #[cfg(test)]
    pub fn test_field_clock_in(&self, instance: i32, guid: u64) -> (u64, Option<u64>) {
        (
            self.start.elapsed().as_millis() as u64,
            self.membership
                .field(instance)
                .and_then(|f| f.respawn_at(guid)),
        )
    }
    #[cfg(test)]
    pub fn test_pc_interest_limit(&mut self, limit: usize) {
        assert!(self.sessions.is_empty());
        self.world.players = PlayerRegistry::new(InterestPolicy::new(100.0, limit).unwrap());
    }
    /// Test setup only: use the existing authorized teleport/commit path.
    #[cfg(test)]
    pub fn test_observer_position(&mut self, guid: u64, position: [f32; 3]) {
        assert!(position.iter().all(|v| v.is_finite()));
        let id = self.world.active_sessions[&guid];
        let key = self.world.session_keys[&id];
        let mut session = self.sessions.remove(&key).unwrap();
        let mut staged = session.gateway.clone();
        let role = staged.role();
        let path = self.options.account_store.as_ref().unwrap();
        let mut accounts = AccountStore::load(path).unwrap();
        accounts
            .set_account_role_offline(
                session.account_guid,
                osfr_tools::account_store::AccountRole::Builder,
            )
            .unwrap();
        accounts.save(path).unwrap();
        staged.set_dev_tools(true);
        staged.set_role(osfr_tools::account_store::AccountRole::Builder);
        let request = osfr_tools::chat::local_request(&format!(
            "realm goto {} {} {}",
            position[0], position[1], position[2]
        ));
        let now = self.start.elapsed().as_millis() as u64;
        let result = staged.handle_at(&wrap_application(&request), now);
        accounts
            .set_account_role_offline(session.account_guid, role)
            .unwrap();
        accounts.save(path).unwrap();
        let outcome = result.unwrap();
        staged.set_role(role);
        staged.set_dev_tools(self.options.dev_tools);
        let previous = session.gateway.state().clone();
        assert!(
            session
                .commit_outcome(
                    staged,
                    previous,
                    outcome,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                    now
                )
                .unwrap()
        );
        assert_eq!(session.gateway.state().position, position);
        self.sessions.insert(key, session);
    }
    // (R) Socket-test setup is host-only and absent from the shipped binary.
    #[cfg(test)]
    pub fn test_child(&mut self) -> i32 {
        self.membership
            .register_runtime_zone(osfr_tools::content::Zone {
                id: 9600,
                name: 9600.to_string(),
                geometry_id: Some(9600),
                zone_type: Some(2),
                tutorial: Some(false),
                sky: Some(String::new()),
                source: Vec::new(),
            })
            .unwrap();
        self.membership
            .create_child_instance(
                0,
                9600,
                osfr_tools::world::InstanceKind::Encounter { encounter_id: 0 },
                self.start.elapsed().as_millis() as u64,
            )
            .unwrap()
    }

    #[cfg(test)]
    pub fn test_field_child(&mut self) -> i32 {
        self.membership
            .create_child_instance(
                0,
                0,
                osfr_tools::world::InstanceKind::Encounter { encounter_id: 0 },
                self.start.elapsed().as_millis() as u64,
            )
            .unwrap()
    }

    #[cfg(test)]
    pub fn test_location(&self, guid: u64) -> Option<(CharacterState, i32, bool, BTreeSet<u64>)> {
        let session_id = self.world.active_sessions.get(&guid)?;
        let key = self.world.session_keys.get(session_id)?;
        let session = self.sessions.get(key)?;
        let location = self.membership.player_location(guid)?;
        Some((
            session.gateway.state().clone(),
            location.instance_id,
            location.ready,
            self.membership
                .instance(location.instance_id)?
                .members
                .clone(),
        ))
    }

    #[cfg(test)]
    pub fn test_transfer(&mut self, guid: u64, target: i32, fail_save: bool) -> Result<(), String> {
        let zone = self
            .membership
            .instance(target)
            .ok_or("unknown instance")?
            .zone_id;
        let id = self.world.active_sessions.get(&guid).ok_or("not joined")?;
        let key = *self.world.session_keys.get(id).ok_or("missing session")?;
        let mut session = self.sessions.remove(&key).ok_or("missing session")?;
        let original_path = session.state_path.clone();
        if fail_save {
            session.state_path = original_path.parent().unwrap().to_owned();
        }
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            let mut staged = session.gateway.clone();
            let outcome = staged.stage_instance_transfer(
                ZoneInstance::from_world(zone, target)?,
                osfr_tools::encounter::ZoneTransition {
                    zone_id: zone,
                    name: zone.to_string(),
                    sky: String::new(),
                    geometry_id: zone as i32,
                    position: [1.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 1.0],
                    tutorial: false,
                },
                self.start.elapsed().as_millis() as u64,
            )?;
            let previous = session.gateway.state().clone();
            session
                .commit_outcome(
                    staged,
                    previous,
                    outcome,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                    self.start.elapsed().as_millis() as u64,
                )
                .and_then(|committed| {
                    if committed {
                        Ok(())
                    } else {
                        Err("transfer refused before commit".into())
                    }
                })
        })();
        session.state_path = original_path;
        self.sessions.insert(key, session);
        result.map_err(|error| error.to_string())
    }

    /// Close sessions through the same persistence path as disconnects.
    pub fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let keys = self.sessions.keys().copied().collect::<Vec<_>>();
        for key in keys {
            if let Some(session) = self.sessions.remove(&key) {
                self.end_session(key, session, "server_shutdown")?;
            }
        }
        self.flush_security();
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // One event loop serializes all shared world mutations; session state stays per entry.
        let now_ms = self.start.elapsed().as_millis() as u64;
        self.membership.collect_empty_instances(now_ms);
        self.poll_studio(now_ms);
        if self.security_flushed.elapsed() >= Duration::from_secs(5) {
            self.flush_security();
            self.security_flushed = Instant::now();
        }
        let mut buffer = [0; 65_535];
        // (R) Bound each receive batch; tick the world after at most 64 packets.
        for _ in 0..64 {
            match self.socket.recv_from(&mut buffer) {
                Ok((length, peer))
                    if osfr_tools::net_policy::allowed(peer.ip(), self.options.allow_lan) =>
                {
                    let wire = &buffer[..length];
                    if wire.starts_with(&[0, 1])
                        && !peer.ip().is_loopback()
                        && !self.handshakes.allow(peer.ip(), Instant::now())
                    {
                        self.refuse("handshake_rate");
                    } else if wire.starts_with(&[0, 1]) {
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
                            let now = Instant::now();
                            let over_budget =
                                !current
                                    .packets
                                    .take(1.0, PACKET_BUDGET.0, PACKET_BUDGET.1, now)
                                    || !current.bytes.take(
                                        wire.len() as f64,
                                        BYTE_BUDGET.0,
                                        BYTE_BUDGET.1,
                                        now,
                                    );
                            if over_budget {
                                // Dropped unread; the session and everyone else keep running.
                                self.sessions.insert(key, current);
                                self.refuse("packet_budget");
                            } else {
                                let result = current.handle(
                                    &self.socket,
                                    wire,
                                    &self.options,
                                    now_ms,
                                    &mut self.world,
                                    &mut self.membership,
                                    &mut self.sessions,
                                );
                                match result {
                                    Ok(false) => {
                                        self.sessions.insert(key, current);
                                    }
                                    Ok(true) => {
                                        self.end_session(key, current, "client_disconnect")?
                                    }
                                    Err(error) => {
                                        self.end_session(key, current, &error.to_string())?
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                // ConnectionReset is Windows reporting that an earlier datagram hit
                // a closed port (a player whose game exited); it is not fatal.
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::ConnectionReset
                    ) =>
                {
                    break;
                }
                Err(error) => return Err(error.into()),
            }
        }
        let now_ms = self.start.elapsed().as_millis() as u64;
        // (R) 20 Hz simulation; ACK/retry flushing still runs every step.
        let tick_due = now_ms >= self.next_tick_ms;
        if tick_due {
            self.next_tick_ms = now_ms.saturating_add(50);
            self.tick_shared_fields(now_ms)?;
        }
        let keys = self.sessions.keys().copied().collect::<Vec<_>>();
        for key in keys {
            let Some(mut current) = self.sessions.remove(&key) else {
                continue;
            };
            let result = if tick_due {
                current.tick(
                    &self.options,
                    now_ms,
                    &mut self.world,
                    &mut self.membership,
                    &mut self.sessions,
                )
            } else {
                Ok(())
            }
            .and_then(|_| current.flush(&self.socket, now_ms));
            if let Err(error) = result {
                self.end_session(key, current, &error.to_string())?;
            } else if current.receiver.expire_reassembly(Instant::now()) {
                // A fragmented message left incomplete past its fixed deadline
                // (KeepAlive cannot extend it; Security Model L7).
                self.end_session(key, current, "reassembly_timeout")?;
            } else if !current.selected_character
                && current.session_start.elapsed() > LOGIN_DEADLINE
            {
                // Not logged in yet: KeepAlive does not extend this (Security Model L6).
                self.end_session(key, current, "login_deadline")?;
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

    /// (R) Studio acts only as a verified admin, never the first arbitrary player.
    fn studio_session(&self) -> Option<SessionKey> {
        self.sessions
            .iter()
            .find(|(_, s)| {
                s.gateway.gameplay().is_some()
                    && s.selected_character
                    && s.gateway.role() == osfr_tools::account_store::AccountRole::Admin
            })
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
        for session in self.sessions.values_mut() {
            session.gateway.refresh_role();
        }
        if !self.options.dev_tools || self.studio_session().is_none() {
            self.world
                .security
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .record(EventKind::DevCommandDenial, None, None);
            return json_reply(
                403,
                json!({"error": "admin account with --dev-tools required"}),
            );
        }
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
                if !name.ends_with(".json")
                    || !name
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.')
                {
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
                    .flat_map(|t| {
                        t.zone_ids()
                            .into_iter()
                            .filter_map(|id| Some((id, t.grid(id)?)))
                    })
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
                    &mut self.membership,
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
        // (R) Retain a bounded 60-second retired-id window. A queued request
        // must not revive an expired connection with a different CRC seed.
        let now = Instant::now();
        self.retired_handshakes.retain(|_, until| *until > now);
        if self.retired_handshakes.contains_key(&key) {
            return Ok(());
        }
        if let Some(existing) = self.sessions.get_mut(&key) {
            let response = existing.response.clone();
            existing.send(&self.socket, &response)?;
            return Ok(());
        }
        if let Some(old_key) = self.session_key_for_peer(peer) {
            let old = self.sessions.remove(&old_key).unwrap();
            self.end_session(old_key, old, "connection_replaced")?;
        }
        let same_ip = self
            .sessions
            .keys()
            .filter(|k| k.peer.ip() == peer.ip())
            .count();
        if self.sessions.len() >= MAX_SESSIONS
            || (!peer.ip().is_loopback() && same_ip >= MAX_SESSIONS_PER_IP)
        {
            self.refuse("session_cap");
            return Ok(());
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
            self.template.as_ref(),
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
        if reason == "login_deadline" {
            if self.retired_handshakes.len() >= MAX_SESSIONS * 4 {
                if let Some(oldest) = self
                    .retired_handshakes
                    .iter()
                    .min_by_key(|(_, until)| **until)
                    .map(|(key, _)| *key)
                {
                    self.retired_handshakes.remove(&oldest);
                }
            }
            self.retired_handshakes
                .insert(key, Instant::now() + Duration::from_secs(60));
        }
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
            deliveries.extend(self.world.social_disconnect(session.character_guid)?);
        }
        if owns_active {
            self.membership.leave(
                session.character_guid,
                self.start.elapsed().as_millis() as u64,
            );
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
    server.shutdown()?;
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

    #[test]
    fn w120_player_denial_is_logged_and_studio_requires_admin() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let scratch =
            std::env::temp_dir().join(format!("osfr-w120-session-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        let log_path = scratch.join("security.jsonl");
        let _ = fs::remove_file(&log_path);
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join("character.json"),
            content: Some(root.join("references/rust-opening-cooking-original-assets")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: true,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: Some(scratch.join("world")),
            stock_text: None,
            allow_lan: false,
            security_log: Some(log_path.clone()),
        };
        let mut server = Server::new(options).unwrap();
        let mut session = Session::new(
            SessionId(120),
            "127.0.0.1:12345".parse().unwrap(),
            SessionRequest {
                id: 120,
                crc_length: 3,
                buffer_size: 512,
            },
            &server.options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        // A no-account-store session can only be the host's own loopback dev
        // session: admin with --dev-tools. Demote it to test a player's denial.
        assert_eq!(
            session.gateway.role(),
            osfr_tools::account_store::AccountRole::Admin
        );
        session
            .gateway
            .set_role(osfr_tools::account_store::AccountRole::Player);
        let mut login = 1u16.to_le_bytes().to_vec();
        for (i, value) in ["w120", "1.910.1.530630", ""].into_iter().enumerate() {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if i == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        session.gateway.handle(&login).unwrap();
        session.gateway.handle(&wrap(&[13, 0])).unwrap();
        session.gateway.handle(&wrap(&[10, 0])).unwrap();
        let before = session.gateway.snapshot();
        session
            .dispatch_message(
                &wrap(&osfr_tools::chat::local_request("realm god")),
                &server.options,
                1000,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .unwrap();
        assert_eq!(session.gateway.snapshot(), before);
        let mut invalid_equip = vec![42, 0, 3, 0];
        invalid_equip.extend((-1_i32).to_le_bytes());
        invalid_equip.extend([0; 8]);
        let mut bundle = vec![0, 25];
        for child in [
            wrap(&osfr_tools::chat::local_request("realm god")),
            wrap(&invalid_equip),
        ] {
            bundle.push(child.len() as u8);
            bundle.extend(child);
        }
        assert!(
            session
                .dispatch_message(
                    &bundle,
                    &server.options,
                    1000,
                    &mut server.world,
                    &mut server.membership,
                    &mut server.sessions,
                    None
                )
                .is_err()
        );
        assert_eq!(session.gateway.snapshot(), before);
        server.flush_security();
        let rows: Vec<serde_json::Value> = fs::read_to_string(&log_path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert!(
            rows.iter()
                .any(|row| row["kind"] == "dev_command_denial" && row["reason"] == "authorization"),
            "{rows:?}"
        );
        let denied: u64 = rows
            .iter()
            .filter(|row| row["kind"] == "dev_command_denial")
            .map(|row| row["count"].as_u64().unwrap())
            .sum();
        assert_eq!(
            denied, 2,
            "rollback must not erase an authorization refusal"
        );
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        server.sessions.insert(key, session);
        let request = studio::Request {
            method: "GET".into(),
            path: "/api/state".into(),
            body: Vec::new(),
        };
        assert_eq!(server.studio_route(&request, 1001).0, 403);
        server
            .sessions
            .get_mut(&key)
            .unwrap()
            .gateway
            .set_role(osfr_tools::account_store::AccountRole::Builder);
        assert_eq!(server.studio_route(&request, 1002).0, 403);
        server
            .sessions
            .get_mut(&key)
            .unwrap()
            .gateway
            .set_role(osfr_tools::account_store::AccountRole::Admin);
        assert_eq!(server.studio_route(&request, 1003).0, 200);
        drop(server);
        fs::remove_dir_all(scratch).unwrap();
    }
    #[test]
    fn pose_only_saves_are_coalesced_and_flushed_by_tick() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let scratch = std::env::temp_dir().join(format!("osfr-pose-save-{}", std::process::id()));
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).unwrap();
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join("character.json"),
            content: None,
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: None,
            stock_text: None,
            allow_lan: false,
            security_log: None,
        };
        let mut server = Server::new(options).unwrap();
        let mut session = Session::new(
            SessionId(7),
            "127.0.0.1:12346".parse().unwrap(),
            SessionRequest {
                id: 7,
                crc_length: 3,
                buffer_size: 512,
            },
            &server.options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        let mut login = 1u16.to_le_bytes().to_vec();
        for (i, value) in ["pose", "1.910.1.530630", ""].into_iter().enumerate() {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if i == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        session.gateway.handle(&login).unwrap();
        session.gateway.handle(&wrap(&[13, 0])).unwrap();
        session.gateway.handle(&wrap(&[10, 0])).unwrap();
        let start = session.gateway.state().clone();
        let saved_x = || -> f32 {
            let text = fs::read_to_string(&session_path(&scratch)).unwrap();
            let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
            doc["character"]["position"][0].as_f64().unwrap() as f32
        };
        let step = |session: &mut Session, server: &mut Server, dx: f32, now_ms: u64| {
            let mut pose = vec![125, 0];
            pose.extend(start.guid.to_le_bytes());
            let p = [start.position[0] + dx, start.position[1], start.position[2]];
            for value in p.into_iter().chain(start.rotation) {
                pose.extend(value.to_le_bytes());
            }
            pose.extend([2, 255]);
            session
                .dispatch_message(
                    &wrap(&pose),
                    &server.options,
                    now_ms,
                    &mut server.world,
                    &mut server.membership,
                    &mut server.sessions,
                    None,
                )
                .unwrap();
        };
        // The first pose after the interval is written at once.
        step(&mut session, &mut server, 0.5, 10_000);
        assert_eq!(saved_x(), start.position[0] + 0.5);
        // A second pose inside the interval waits: the file keeps the old one.
        step(&mut session, &mut server, 1.0, 10_500);
        assert_eq!(session.gateway.state().position[0], start.position[0] + 1.0);
        assert_eq!(saved_x(), start.position[0] + 0.5);
        // Canary: the tick before the interval ends writes nothing yet...
        session
            .tick(
                &server.options,
                14_000,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
            )
            .unwrap();
        assert_eq!(saved_x(), start.position[0] + 0.5);
        // ...and the first tick after it writes the newest pose.
        session
            .tick(
                &server.options,
                15_000,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
            )
            .unwrap();
        assert_eq!(saved_x(), start.position[0] + 1.0);
        drop(server);
        fs::remove_dir_all(scratch).unwrap();
    }

    fn session_path(scratch: &std::path::Path) -> PathBuf {
        scratch.join("character.json")
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
            security_log: None,
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
            None,
        )
        .unwrap();
        let mut world = SharedWorld::new();
        let mut membership = World::runtime();
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
                &mut membership,
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
                &mut membership,
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
                    &mut membership,
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
                &mut membership,
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

#[cfg(test)]
mod f120_fishing_hour_reconnect {
    use super::*;
    use osfr_tools::game_store::Snapshot;
    use osfr_tools::gateway::Gateway;
    use serde_json::{Value, json};

    const NOW: u64 = 10_000;
    const REED: u64 = 9_000_000_181;
    const GATE_POSITION: [f32; 3] = [-1319.765, -65.62, 898.9357];
    const ACTOR: osfr_tools::fishing::Handle = osfr_tools::fishing::Handle {
        id: 0x1122_3344,
        seq: 0x5566_7788,
    };

    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn wrap(inner: &[u8]) -> Vec<u8> {
        let mut bytes = vec![5, 0, 1];
        bytes.extend((inner.len() as u32).to_le_bytes());
        bytes.extend(inner);
        bytes
    }

    fn copy_tree(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    fn login() -> Vec<u8> {
        let mut bytes = 1u16.to_le_bytes().to_vec();
        for (index, value) in ["f120-fixture", "1.910.1.530630", ""]
            .into_iter()
            .enumerate()
        {
            bytes.extend((value.len() as u32).to_le_bytes());
            bytes.extend(value.bytes());
            if index == 0 {
                bytes.extend(1u64.to_le_bytes());
            }
        }
        bytes
    }

    fn npc(sub: u16, target: u64, choice: Option<i32>) -> Vec<u8> {
        let mut bytes = [26u16.to_le_bytes(), sub.to_le_bytes()].concat();
        bytes.extend(target.to_le_bytes());
        match choice {
            Some(choice) => bytes.extend(choice.to_le_bytes()),
            None => bytes.push(0),
        }
        bytes
    }

    fn lifecycle(instance: i32) -> Vec<u8> {
        let mut bytes = vec![39, 0, 5];
        bytes.extend(instance.to_le_bytes());
        bytes.extend((-1i32).to_le_bytes());
        bytes.extend((-1i32).to_le_bytes());
        bytes
    }

    fn request(sub: u16, body: &[u8]) -> Vec<u8> {
        let mut bytes = [138u16.to_le_bytes(), sub.to_le_bytes()].concat();
        bytes.extend(body);
        bytes
    }

    fn register() -> Vec<u8> {
        let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
        body.push(0);
        request(2, &body)
    }

    fn cast() -> Vec<u8> {
        let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
        for value in [
            GATE_POSITION[0] + 10.0,
            GATE_POSITION[1],
            GATE_POSITION[2],
            1.0,
        ] {
            body.extend(value.to_le_bytes());
        }
        body.push(0);
        request(6, &body)
    }

    fn reel() -> Vec<u8> {
        let mut body = [ACTOR.id.to_le_bytes(), ACTOR.seq.to_le_bytes()].concat();
        body.push(1);
        request(7, &body)
    }

    fn menu(gateway: &mut Gateway, target: u64, now: u64) -> Vec<Vec<u8>> {
        gateway
            .handle_at(&wrap(&npc(8, target, None)), now)
            .unwrap()
            .replies
            .into_iter()
            .map(|packet| packet[7..].to_vec())
            .collect()
    }

    fn fish_option(menu: &[Vec<u8>], string_id: u32) -> Option<i32> {
        let packet = menu
            .iter()
            .find(|packet| packet.starts_with(&[26, 0, 9, 0]))?;
        let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
        (0..count).find_map(|index| {
            let at = 17 + index * 28;
            let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
            let label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
            (label == string_id).then_some(id)
        })
    }

    fn send(
        session: &mut Session,
        options: &Options,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
        message: &[u8],
        now: u64,
    ) {
        session
            .dispatch_message(
                &wrap(message),
                options,
                now,
                world,
                membership,
                sessions,
                None,
            )
            .unwrap();
    }

    fn send_raw(
        session: &mut Session,
        options: &Options,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
        message: &[u8],
        now: u64,
    ) {
        session
            .dispatch_message(message, options, now, world, membership, sessions, None)
            .unwrap();
    }

    fn coins(snapshot: &Snapshot) -> u32 {
        snapshot.gameplay.as_ref().unwrap().inventory.coins
    }

    fn fisherman_xp(snapshot: &Snapshot) -> u32 {
        snapshot
            .gameplay
            .as_ref()
            .unwrap()
            .profiles
            .as_ref()
            .unwrap()
            .experience
            .get(&137)
            .copied()
            .unwrap_or(0)
    }

    fn try_catch(
        session: &mut Session,
        options: &Options,
        world: &mut SharedWorld,
        membership: &mut World,
        sessions: &mut SessionMap,
        at: u64,
    ) -> (GatewayOutcome, u64) {
        send(
            session,
            options,
            world,
            membership,
            sessions,
            &cast(),
            at + 10,
        );
        session.gateway.tick_at(at + 1_600).unwrap();
        session.gateway.tick_at(at + 2_600).unwrap();
        let mut probe = session.gateway.clone();
        let outcome = probe.handle_at(&wrap(&reel()), at + 2_700).unwrap();
        send(
            session,
            options,
            world,
            membership,
            sessions,
            &reel(),
            at + 2_700,
        );
        (outcome, at + 2_700)
    }

    #[test]
    fn an_hour_cap_survives_reconnect_save_reload_and_a_blocked_catch_writes_nothing() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let content_source = std::env::var_os("OSFR_OPENING_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("references/rust-opening-cooking-original-assets"));
        let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("references/rust-gateway-reference"));
        assert!(
            content_source.join("inventory.json").is_file(),
            "private opening fixture is missing"
        );
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let scratch = Scratch(
            std::env::temp_dir().join(format!("osfr-f120-fishing-{}-{nonce}", std::process::id())),
        );
        fs::create_dir_all(&scratch.0).unwrap();
        let content = scratch.0.join("content");
        copy_tree(&content_source, &content);
        let mut spots: Value =
            serde_json::from_str(include_str!("../../data/content/fishing_spots.json")).unwrap();
        spots["catch_cap"]["enabled"] = json!(true);
        spots["catch_cap"]["window"] = json!("hour");
        spots["catch_cap"]["max_paid_catches"] = json!(1);
        for row in spots["payout"]["by_grade"].as_array_mut().unwrap() {
            row["coins"] = json!(25);
        }
        // Use one existing stackable item for every synthetic fixture size so the private
        // opening inventory can contain the catch without importing the full item catalogue.
        for sizes in spots["items"].as_object_mut().unwrap().values_mut() {
            *sizes = json!([68_001, 0, 0, 0]);
        }
        fs::write(
            content.join("fishing_spots.json"),
            serde_json::to_vec(&spots).unwrap(),
        )
        .unwrap();

        let state_path = scratch.0.join("character.json");
        let mut character = Gateway::reference_state(&reference).unwrap();
        character.position = GATE_POSITION;
        let initial = Gateway::load_gameplay(&reference, character, &content, None).unwrap();
        save(&state_path, &initial.snapshot()).unwrap();
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: reference.clone(),
            state: state_path.clone(),
            content: Some(content.clone()),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: Some(scratch.0.join("world")),
            stock_text: None,
            allow_lan: false,
            security_log: None,
        };
        let new_session = |id: u64| {
            Session::new(
                SessionId(id),
                format!("127.0.0.1:{}", 30_000 + id).parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    buffer_size: 512,
                },
                &options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap()
        };
        let mut world = SharedWorld::new();
        let mut membership = World::runtime();
        let mut sessions = SessionMap::new();
        let mut first = new_session(41);
        send_raw(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &login(),
            0,
        );
        send(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &[13, 0],
            100,
        );
        send(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &[10, 0],
            200,
        );
        let option =
            fish_option(&menu(&mut first.gateway, REED, 300), 73_552).expect("fishing option");
        send(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &npc(10, REED, Some(option)),
            400,
        );
        send(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &lifecycle(1),
            500,
        );
        send(
            &mut first,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &register(),
            600,
        );
        assert_eq!(coins(&first.gateway.snapshot()), 0);
        assert_eq!(fisherman_xp(&first.gateway.snapshot()), 0);

        let mut at = NOW;
        let caught_at = loop {
            let (outcome, when) = try_catch(
                &mut first,
                &options,
                &mut world,
                &mut membership,
                &mut sessions,
                at,
            );
            if outcome.state_changed {
                break when;
            }
            at = when + 10_000;
            assert!(at < NOW + 3_000_000, "no deterministic catch before expiry");
        };
        let paid = first.gateway.snapshot();
        assert_eq!(coins(&paid), 25);
        assert_eq!(fisherman_xp(&paid), 10);
        let history = first.gateway.fishing_hour_history().unwrap();
        assert_eq!(history, vec![caught_at]);
        assert_eq!(
            world.fishing_hours.get(&first.character_guid),
            Some(&history)
        );
        let saved_after_payout = fs::read(&state_path).unwrap();
        let guid = first.character_guid;
        world.leave_player(first.id);
        world.active_sessions.remove(&guid);
        membership.leave(guid, caught_at + 1_000);
        drop(first);

        let mut second = new_session(42);
        send_raw(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &login(),
            caught_at + 1_000,
        );
        send(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &[13, 0],
            caught_at + 1_100,
        );
        send(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &[10, 0],
            caught_at + 1_200,
        );
        assert_eq!(second.gateway.fishing_hour_history().unwrap(), history);
        assert_eq!(second.gateway.snapshot(), paid);
        assert_eq!(fs::read(&state_path).unwrap(), saved_after_payout);

        let option = fish_option(&menu(&mut second.gateway, REED, caught_at + 1_300), 73_552)
            .expect("fishing option after reload");
        send(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &npc(10, REED, Some(option)),
            caught_at + 1_400,
        );
        send(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &lifecycle(1),
            caught_at + 1_500,
        );
        send(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            &register(),
            caught_at + 1_600,
        );
        let before_refusal = second.gateway.snapshot();
        let before_refusal_file = fs::read(&state_path).unwrap();
        let (blocked, blocked_at) = try_catch(
            &mut second,
            &options,
            &mut world,
            &mut membership,
            &mut sessions,
            caught_at + 60_000,
        );
        assert!(!blocked.state_changed, "a capped catch requests no save");
        assert!(!blocked.gameplay_changed);
        let replies = blocked.replies.concat();
        assert!(
            String::from_utf8_lossy(&replies).contains("reward limit"),
            "the capped catch takes the refusal path"
        );
        assert_eq!(second.gateway.snapshot(), before_refusal);
        assert_eq!(coins(&second.gateway.snapshot()), 25);
        assert_eq!(fisherman_xp(&second.gateway.snapshot()), 10);
        assert_eq!(fs::read(&state_path).unwrap(), before_refusal_file);
        assert_eq!(second.gateway.fishing_hour_history().unwrap(), history);
        assert_eq!(world.fishing_hours.get(&guid), Some(&history));

        let mut after_expiry = caught_at + 3_602_000;
        let paid_again = loop {
            let (outcome, when) = try_catch(
                &mut second,
                &options,
                &mut world,
                &mut membership,
                &mut sessions,
                after_expiry,
            );
            if outcome.state_changed {
                break when;
            }
            after_expiry = when + 10_000;
            assert!(
                after_expiry < blocked_at + 3_700_000,
                "no catch after the hour window expired"
            );
        };
        assert!(paid_again > caught_at + 3_600_000);
        assert_eq!(coins(&second.gateway.snapshot()), 50);
        assert_eq!(fisherman_xp(&second.gateway.snapshot()), 20);
        assert_eq!(
            second.gateway.fishing_hour_history().unwrap(),
            vec![paid_again]
        );
        assert_eq!(world.fishing_hours.get(&guid), Some(&vec![paid_again]));
        assert_ne!(fs::read(&state_path).unwrap(), before_refusal_file);
    }
}

fn field_packet(packet: &[u8]) -> bool {
    packet.starts_with(&[125, 0])
        || packet.get(..4).is_some_and(|h| {
            matches!(
                h,
                [35, 0, 5, 0] | [35, 0, 8, 0] | [35, 0, 16, 0] | [32, 0, 7, 0]
            )
        })
}
fn field_packets_for(
    recipient: &Session,
    membership: &World,
    players: &PlayerRegistry,
    instance: i32,
    packets: &[Vec<u8>],
    known_npcs: &BTreeSet<u64>,
) -> Vec<Delivery> {
    if !field_recipient_matches(
        recipient.gateway.is_world_loaded(),
        membership
            .player_location(recipient.character_guid)
            .map(|l| (l.instance_id, l.ready)),
        instance,
    ) {
        return Vec::new();
    }
    let known = |guid| {
        guid == recipient.character_guid
            || known_npcs.contains(&guid)
            || players.knows_character(recipient.id, guid)
    };
    packets
        .iter()
        .filter(|packet| {
            let offset = if packet.starts_with(&[125, 0]) { 2 } else { 4 };
            let Some(guid) = packet
                .get(offset..offset + 8)
                .and_then(|b| b.try_into().ok())
                .map(u64::from_le_bytes)
            else {
                return false;
            };
            if !known(guid) {
                return false;
            }
            if packet.starts_with(&[32, 0, 7, 0]) {
                let Some(target) = packet
                    .get(20..28)
                    .and_then(|b| b.try_into().ok())
                    .map(u64::from_le_bytes)
                else {
                    return false;
                };
                return known(target);
            }
            true
        })
        .map(|packet| Delivery {
            recipient: recipient.id,
            bytes: wrap_application(packet),
        })
        .collect()
}
// (R) A server-owned membership, never the attacker's directed PC view,
// determines which ready instance may observe a field event.
fn field_recipient_matches(loaded: bool, location: Option<(i32, bool)>, instance: i32) -> bool {
    #[cfg(test)]
    if std::env::var_os("OSFR_W127_CROSS_INSTANCE_CANARY").is_some() {
        return loaded; // Deliberate test-only broadcast mutation.
    }
    loaded && location.is_some_and(|(id, ready)| id == instance && ready)
}

#[cfg(test)]
#[test]
fn w127_other_instance_recipient_canary() {
    assert!(field_recipient_matches(true, Some((0, true)), 0));
    assert!(
        !field_recipient_matches(true, Some((7, true)), 0),
        "recipient assertion: another instance received field deltas"
    );
    assert!(!field_recipient_matches(true, Some((0, false)), 0));
    assert!(!field_recipient_matches(false, Some((0, true)), 0));
    assert!(!field_recipient_matches(true, None, 0));
}

#[cfg(test)]
#[test]
fn shared_combat_budget_cannot_be_refunded_by_staging() {
    let world = SharedWorld::new();
    let clone = world.clone();
    let now = Instant::now();
    for _ in 0..20 {
        assert!(clone.security.lock().unwrap().allow_combat(101, now));
    }
    assert!(!world.security.lock().unwrap().allow_combat(101, now));
    drop(clone);
    assert!(!world.security.lock().unwrap().allow_combat(101, now));
    assert!(world.security.lock().unwrap().allow_combat(202, now));
}

/// W173: the chat channel switches and bubble colour (26/32, 26/33, 26/18) through
/// `Session::route_application` with three sessions and no sockets.
#[cfg(test)]
mod w173_chat_settings_routing {
    use super::*;
    use osfr_tools::players::PlayerPresentation;
    use osfr_tools::progression_extras::{ClaimWindow, claim_request};
    use osfr_tools::rewards::{RewardDisplay, RewardGrant};
    use osfr_tools::social_chat::{BubbleStyle, ChatChannel, PacketChat};

    const GUIDS: [u64; 3] = [301, 302, 303];

    struct Rig {
        sessions: [Session; 3],
        world: SharedWorld,
        scratch: PathBuf,
        options: Options,
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.scratch);
        }
    }

    fn rig(name: &str) -> Rig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let scratch = std::env::temp_dir().join(format!("osfr-w173-{name}-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join("character.json"),
            content: Some(root.join("references/rust-opening-cooking-original-assets")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: Some(scratch.join("world")),
            stock_text: None,
            allow_lan: false,
            security_log: None,
        };
        let session = |index: usize| {
            let id = 17 + index as u64;
            let mut session = Session::new(
                SessionId(id),
                "127.0.0.1:12345".parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    buffer_size: 512,
                },
                &options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap();
            session.character_guid = GUIDS[index];
            session.account_guid = GUIDS[index] + 10_000;
            session
        };
        let sessions = [session(0), session(1), session(2)];
        let mut world = SharedWorld::new();
        for (index, session) in sessions.iter().enumerate() {
            let guid = GUIDS[index];
            let x = index as f32 * 5.0;
            world
                .players
                .join_with_presentation(
                    session.id,
                    guid + 10_000,
                    guid,
                    ZoneInstance {
                        zone_id: 1,
                        instance_id: 1,
                    },
                    [x, 0.0, 0.0],
                    0.0,
                    PlayerPresentation {
                        character_guid: guid,
                        first_name: format!("P{guid}"),
                        ..PlayerPresentation::default()
                    },
                )
                .unwrap();
            world.chat_sessions.insert(
                session.id,
                ChatSession {
                    session_id: session.id.0,
                    character_guid: guid,
                    // Synthetic fixture names, not recovered player names.
                    name: NameData {
                        fields: [0; 3],
                        first: format!("P{guid}"),
                        last: String::new(),
                    },
                    zone_id: 1,
                    position: [x, 0.0, 0.0],
                    chat_area_id: None,
                    group_id: None,
                    guild_id: None,
                    is_member: true,
                },
            );
            world.chat_settings.join(guid, None);
            world.joined.insert(session.id);
        }
        Rig {
            sessions,
            world,
            scratch,
            options,
        }
    }

    fn channel_packet(sub: u8, name: &str) -> Vec<u8> {
        let mut bytes = vec![26, 0, sub, 0];
        bytes.extend((name.len() as i32).to_le_bytes());
        bytes.extend(name.as_bytes());
        bytes
    }

    fn wrap(inner: &[u8]) -> Vec<u8> {
        let mut bytes = vec![5, 0, 1];
        bytes.extend((inner.len() as u32).to_le_bytes());
        bytes.extend(inner);
        bytes
    }

    fn bubble_packet(fg: i32, bg: i32, size: i32, guid: u64) -> Vec<u8> {
        let mut bytes = vec![26, 0, 18, 0];
        for value in [fg, bg, size] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend(guid.to_le_bytes());
        bytes
    }

    fn chat(channel: ChatChannel, to: Option<&str>) -> Vec<u8> {
        social_chat::encode_packet_chat(&PacketChat {
            channel,
            sender_guid: 0,
            target_guid: 0,
            sender_name: NameData::default(),
            target_name: NameData {
                fields: [0; 3],
                first: to.unwrap_or_default().to_owned(),
                last: String::new(),
            },
            message: "synthetic line".to_owned(),
            position: [0.0; 4],
            guild_guid: 0,
            language: 0,
            area_id: None,
        })
        .unwrap()
    }

    fn recipients(deliveries: Vec<Delivery>) -> Vec<u64> {
        let mut ids: Vec<u64> = deliveries.iter().map(|d| d.recipient.0).collect();
        ids.sort();
        ids
    }

    fn send(session: &mut Session, world: &mut SharedWorld, packet: &[u8]) -> Vec<Delivery> {
        session.route_application(packet, world, 1_000)
    }

    fn muted(world: &SharedWorld, index: usize, channel: ChatChannel) -> bool {
        world
            .chat_settings
            .get(GUIDS[index])
            .unwrap()
            .is_muted(channel)
    }

    #[test]
    fn a_muted_channel_is_dropped_for_that_player_only() {
        let mut rig = rig("mute");
        let [a, b, c] = &mut rig.sessions;
        let world = &mut rig.world;
        // B switches Trade off and Area on; the client gets no reply to either.
        assert!(send(b, world, &channel_packet(33, "WorldTrade")).is_empty());
        assert!(send(b, world, &channel_packet(32, "WorldArea")).is_empty());
        assert!(muted(world, 1, ChatChannel::Trade));
        assert!(!muted(world, 0, ChatChannel::Trade));
        // C speaks on Trade: A and C hear it, B does not.
        let trade = send(c, world, &chat(ChatChannel::Trade, None));
        assert_eq!(recipients(trade), vec![17, 19]);
        // A tells B, and B's Say reaches everyone: neither channel can be switched off.
        let tell = send(a, world, &chat(ChatChannel::Tell, Some("P302")));
        assert_eq!(recipients(tell), vec![18]);
        let say = send(b, world, &chat(ChatChannel::Say, None));
        assert_eq!(recipients(say), vec![17, 18, 19]);
        // Switching Trade back on clears the mute.
        assert!(send(b, world, &channel_packet(32, "WorldTrade")).is_empty());
        assert!(!muted(world, 1, ChatChannel::Trade));
    }

    #[test]
    fn forged_names_and_wild_values_change_nothing() {
        let mut rig = rig("forged");
        let [a, ..] = &mut rig.sessions;
        let world = &mut rig.world;
        let before = world.chat_settings.clone();
        for bad in [
            channel_packet(33, "WorldSay"),
            channel_packet(33, "GroupSay"),
            channel_packet(32, &"x".repeat(5_000)),
            bubble_packet(-1, 0, 1, 0),
            bubble_packet(0, 0x0100_0000, 1, 0),
            bubble_packet(0, 0, 99, 0),
            vec![26, 0, 18, 0, 1],
            vec![26, 0, 32, 0],
        ] {
            assert!(send(a, world, &bad).is_empty());
            assert_eq!(world.chat_settings, before);
        }
    }

    #[test]
    fn a_bubble_colour_is_stored_for_the_sender_alone() {
        let mut rig = rig("bubble");
        let [_, b, _] = &mut rig.sessions;
        let world = &mut rig.world;
        // The wire guid slot is forged to name player A; it must be ignored.
        let forged = bubble_packet(0x112233, 0x445566, 3, GUIDS[0]);
        assert!(send(b, world, &forged).is_empty());
        let style = |guid| world.chat_settings.get(guid).unwrap().bubble();
        let chosen = BubbleStyle {
            foreground: 0x112233,
            background: 0x445566,
            size: 3,
        };
        assert_eq!(style(GUIDS[1]), chosen);
        assert_eq!(style(GUIDS[0]), BubbleStyle::default());
    }

    #[test]
    fn claim_requests_and_bundled_children_are_budgeted_before_staging() {
        const CLAIM: u64 = 0xA11C_E5;
        let mut rig = rig("claim-budget");
        let session = &mut rig.sessions[0];
        let mut login = 1u16.to_le_bytes().to_vec();
        for (index, text) in ["claim-budget", "1.910.1.530630", ""]
            .into_iter()
            .enumerate()
        {
            login.extend((text.len() as u32).to_le_bytes());
            login.extend(text.as_bytes());
            if index == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        session.gateway.handle(&login).unwrap();
        session.gateway.handle(&wrap(&[13, 0])).unwrap();
        session.gateway.handle(&wrap(&[10, 0])).unwrap();
        session
            .gateway
            .issue_claim(
                CLAIM,
                RewardGrant {
                    coins: 1,
                    items: Vec::new(),
                    job_xp: Default::default(),
                    display: RewardDisplay {
                        source_guid: 0,
                        player_guid: 1,
                        reward_kind: 0,
                        icon_id: 0,
                        name_id: 0,
                        rows: Vec::new(),
                    },
                },
            )
            .unwrap();
        session.recent_on = true;
        let mut membership = World::runtime();
        let mut sessions = SessionMap::new();
        let recover = |session: &Session| {
            (
                session.gateway.snapshot(),
                fs::read(&session.state_path).ok(),
            )
        };
        let baseline = recover(session);
        for _ in 0..9 {
            session
                .dispatch_message(
                    &wrap(&ClaimWindow::list_request()),
                    &rig.options,
                    1_000,
                    &mut rig.world,
                    &mut membership,
                    &mut sessions,
                    None,
                )
                .unwrap();
        }
        session.recent.clear();

        let mut bundle = vec![0, 25];
        for request in [
            wrap(&claim_request(0x5151, &[CLAIM]).unwrap()),
            wrap(&ClaimWindow::list_request()),
        ] {
            bundle.push(request.len() as u8);
            bundle.extend(request);
        }
        session
            .dispatch_message(
                &bundle,
                &rig.options,
                1_000,
                &mut rig.world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();

        assert_eq!(
            recover(session),
            baseline,
            "refusal changed gameplay or saved bytes"
        );
        assert!(
            session.recent.iter().all(|row| row["dir"] != "out"),
            "a refused bundle produced replies: {:?}",
            session.recent
        );
    }

    #[test]
    fn combat_bundle_children_are_budgeted_once() {
        let mut rig = rig("combat-budget");
        let session = &mut rig.sessions[0];
        let mut login = 1u16.to_le_bytes().to_vec();
        for (index, text) in ["combat-budget", "1.910.1.530630", ""]
            .into_iter()
            .enumerate()
        {
            login.extend((text.len() as u32).to_le_bytes());
            login.extend(text.as_bytes());
            if index == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        session.gateway.handle(&login).unwrap();
        session.gateway.handle(&wrap(&[13, 0])).unwrap();
        session.gateway.handle(&wrap(&[10, 0])).unwrap();

        let mut bundle = vec![0, 25];
        for _ in 0..2 {
            let mut request = vec![26, 0, 10, 0];
            request.extend(0u64.to_le_bytes());
            request.extend(0i32.to_le_bytes());
            let request = wrap(&request);
            bundle.push(request.len() as u8);
            bundle.extend(request);
        }
        let mut membership = World::runtime();
        let mut sessions = SessionMap::new();
        session
            .dispatch_message(
                &bundle,
                &rig.options,
                1_000,
                &mut rig.world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();

        let now = Instant::now();
        let mut security = rig.world.security.lock().unwrap_or_else(|e| e.into_inner());
        let allowed = (0..10)
            .filter(|_| {
                security
                    .budgets
                    .allow(session.account_guid, &ClaimWindow::list_request(), now)
            })
            .count();
        assert_eq!(
            allowed, 8,
            "a combat bundle must charge each child once, not once per recursive dispatch"
        );
    }
}

/// T020: family-40 grouping and group chat through `Session::route_application` with three
/// sessions and no sockets. The two clients that form the party are the gate; the third session
/// proves that group chat and roster events stay inside the authenticated group.
#[cfg(test)]
mod t020_group_routing {
    use super::*;
    use osfr_tools::players::PlayerPresentation;
    use osfr_tools::social_chat::{ChatChannel, PacketChat};

    const GUIDS: [u64; 3] = [301, 302, 303];
    const NAMES: [&str; 3] = ["P301", "P302", "P303"];

    struct Rig {
        sessions: [Session; 3],
        world: SharedWorld,
        scratch: PathBuf,
        options: Options,
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.scratch);
        }
    }

    fn options(scratch: &Path) -> Options {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join("character.json"),
            content: Some(root.join("references/rust-opening-cooking-original-assets")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: Some(scratch.join("world")),
            stock_text: None,
            allow_lan: false,
            security_log: None,
        }
    }

    fn sessions(options: &Options) -> [Session; 3] {
        std::array::from_fn(|index| {
            let id = 17 + index as u64;
            let mut session = Session::new(
                SessionId(id),
                "127.0.0.1:12345".parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    buffer_size: 512,
                },
                &options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap();
            session.character_guid = GUIDS[index];
            session.account_guid = GUIDS[index] + 10_000;
            session
        })
    }

    fn session_key(session: &Session) -> SessionKey {
        SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        }
    }

    fn seed_world(world: &mut SharedWorld, sessions: &[Session; 3]) {
        for (index, session) in sessions.iter().enumerate() {
            let guid = GUIDS[index];
            let first = NAMES[index];
            world
                .players
                .join_with_presentation(
                    session.id,
                    guid + 10_000,
                    guid,
                    ZoneInstance {
                        zone_id: 1,
                        instance_id: 1,
                    },
                    [index as f32 * 5.0, 0.0, 0.0],
                    0.0,
                    PlayerPresentation {
                        character_guid: guid,
                        first_name: first.to_owned(),
                        ..PlayerPresentation::default()
                    },
                )
                .unwrap();
            let identity = SocialIdentity {
                account_id: guid + 10_000,
                character_guid: guid,
                name: SocialName::new(first, "").unwrap(),
            };
            world.identities.insert(guid, identity.clone());
            world
                .social
                .register(SocialLists::new(identity).unwrap())
                .unwrap();
            world.social.login(guid).unwrap();
            world.active_sessions.insert(guid, session.id);
            world.chat_sessions.insert(
                session.id,
                ChatSession {
                    session_id: session.id.0,
                    character_guid: guid,
                    name: NameData {
                        fields: [0; 3],
                        first: first.to_owned(),
                        last: String::new(),
                    },
                    zone_id: 1,
                    position: [index as f32 * 5.0, 0.0, 0.0],
                    chat_area_id: None,
                    group_id: None,
                    guild_id: None,
                    is_member: true,
                },
            );
            world.chat_settings.join(guid, None);
            world.joined.insert(session.id);
            world.session_keys.insert(session.id, session_key(session));
        }
    }

    fn rig(name: &str) -> Rig {
        let scratch = std::env::temp_dir().join(format!("osfr-t020-{name}-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        let options = options(&scratch);
        let sessions = sessions(&options);
        let mut world = SharedWorld::new();
        seed_world(&mut world, &sessions);
        Rig {
            sessions,
            world,
            scratch,
            options,
        }
    }

    fn server_rig(name: &str) -> (Server, [SessionKey; 3], PathBuf) {
        let scratch = std::env::temp_dir().join(format!("osfr-t020-{name}-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        let mut server = Server::new(options(&scratch)).unwrap();
        let sessions = sessions(&server.options);
        let keys = std::array::from_fn(|index| session_key(&sessions[index]));
        seed_world(&mut server.world, &sessions);
        for (key, session) in keys.into_iter().zip(sessions) {
            server.sessions.insert(key, session);
        }
        (server, keys, scratch)
    }

    fn send(rig: &mut Rig, index: usize, packet: &[u8]) -> Vec<Delivery> {
        let deliveries = rig.sessions[index].route_application(packet, &mut rig.world, 1_000);
        // Production calls this after every routed application message; group chat reads the
        // same canonical membership from the chat session.
        rig.world.refresh_group_ids();
        deliveries
    }

    fn route_server(server: &mut Server, key: SessionKey, packet: &[u8]) -> Vec<Delivery> {
        let mut session = server.sessions.remove(&key).unwrap();
        let deliveries = session.route_application(packet, &mut server.world, 1_000);
        server.sessions.insert(key, session);
        server.world.refresh_group_ids();
        deliveries
    }

    fn inner(delivery: &Delivery) -> &[u8] {
        assert_eq!(&delivery.bytes[..3], &[5, 0, 1]);
        &delivery.bytes[7..]
    }

    fn put_string(out: &mut Vec<u8>, value: &str) {
        out.extend_from_slice(&(value.len() as i32).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    }

    fn put_name(out: &mut Vec<u8>, index: usize) {
        for _ in 0..3 {
            out.extend_from_slice(&0i32.to_le_bytes());
        }
        put_string(out, NAMES[index]);
        put_string(out, "");
    }

    fn group_header(sub: u16) -> Vec<u8> {
        let mut out = 40u16.to_le_bytes().to_vec();
        out.extend_from_slice(&sub.to_le_bytes());
        out
    }

    fn invite(from: usize, to: usize, group_id: u32) -> Vec<u8> {
        let mut out = group_header(1);
        out.extend_from_slice(&0i16.to_le_bytes());
        out.extend_from_slice(&0i16.to_le_bytes());
        out.extend_from_slice(&GUIDS[from].to_le_bytes());
        put_name(&mut out, from);
        out.extend_from_slice(&GUIDS[to].to_le_bytes());
        put_name(&mut out, to);
        out.extend_from_slice(&group_id.to_le_bytes());
        out.extend_from_slice(
            &[0u32; 5]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        out
    }

    fn response(from: usize, to: usize, group_id: u32, accepted: bool) -> Vec<u8> {
        let mut out = group_header(4);
        out.extend_from_slice(&0i16.to_le_bytes());
        out.extend_from_slice(&0i16.to_le_bytes());
        out.extend_from_slice(&GUIDS[from].to_le_bytes());
        put_name(&mut out, from);
        out.extend_from_slice(&GUIDS[to].to_le_bytes());
        put_name(&mut out, to);
        out.extend_from_slice(&group_id.to_le_bytes());
        out.push(u8::from(accepted));
        out.extend_from_slice(
            &[0u32; 5]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
        );
        out
    }

    fn leave(index: usize) -> Vec<u8> {
        let mut out = group_header(3);
        out.extend_from_slice(&GUIDS[index].to_le_bytes());
        put_name(&mut out, index);
        out
    }

    fn kick(index: usize) -> Vec<u8> {
        let mut out = group_header(6);
        out.extend_from_slice(&GUIDS[index].to_le_bytes());
        put_name(&mut out, index);
        out
    }

    fn chat(channel: ChatChannel) -> Vec<u8> {
        social_chat::encode_packet_chat(&PacketChat {
            channel,
            sender_guid: 0,
            target_guid: 0,
            sender_name: NameData::default(),
            target_name: NameData::default(),
            message: "synthetic line".to_owned(),
            position: [0.0; 4],
            guild_guid: 0,
            language: 0,
            area_id: None,
        })
        .unwrap()
    }

    fn family_sub(delivery: &Delivery) -> (u16, u16) {
        let bytes = inner(delivery);
        (
            u16::from_le_bytes([bytes[0], bytes[1]]),
            u16::from_le_bytes([bytes[2], bytes[3]]),
        )
    }

    fn has(deliveries: &[Delivery], index: usize, family: u16, sub: u16) -> bool {
        deliveries.iter().any(|delivery| {
            delivery.recipient == SessionId(17 + index as u64)
                && family_sub(delivery) == (family, sub)
        })
    }

    fn result_for(deliveries: &[Delivery], index: usize, sub: u16) -> Option<u32> {
        deliveries
            .iter()
            .find(|delivery| {
                delivery.recipient == SessionId(17 + index as u64)
                    && family_sub(delivery) == (40, sub)
            })
            .map(|delivery| u32::from_le_bytes(inner(delivery)[4..8].try_into().unwrap()))
    }

    fn recipients(deliveries: &[Delivery]) -> Vec<u64> {
        let mut ids: Vec<u64> = deliveries.iter().map(|d| d.recipient.0).collect();
        ids.sort();
        ids
    }

    #[test]
    fn two_sessions_form_chat_and_leave_a_party_and_refusals_change_nothing() {
        let mut rig = rig("party");
        let a = 0;
        let b = 1;
        let c = 2;

        // A singleton cannot invite itself: no solo group and no pending invite may appear.
        let refused = send(&mut rig, c, &invite(c, c, 0));
        assert_eq!(result_for(&refused, c, 2), Some(3));
        assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
        assert_eq!(rig.world.social.group_invitee(GUIDS[c]), None);

        // 40/1 creates the transient leader row (40/8), invites the target, and answers 40/2=0.
        let invited = send(&mut rig, a, &invite(a, b, 0));
        assert!(has(&invited, a, 40, 8));
        assert_eq!(result_for(&invited, a, 2), Some(0));
        assert!(has(&invited, b, 40, 1));
        let group_id = rig.world.social.group_of(GUIDS[a]).unwrap();
        let pending = rig.world.social.group_invitee(GUIDS[b]);
        assert_eq!(pending, Some((group_id, GUIDS[a])));

        // Repeats and cross-groups refuse against unchanged membership.
        let repeated = send(&mut rig, a, &invite(a, b, group_id));
        assert_eq!(result_for(&repeated, a, 2), Some(6));
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a]].as_slice())
        );
        assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

        let cross = send(&mut rig, c, &invite(c, b, 0));
        assert_eq!(result_for(&cross, c, 2), Some(4));
        assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a]].as_slice())
        );
        assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

        let self_while_grouped = send(&mut rig, a, &invite(a, a, group_id));
        assert_eq!(result_for(&self_while_grouped, a, 2), Some(3));
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a]].as_slice())
        );
        assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

        // 40/4 accept fans out the invitation notice, member row and full roster.
        let accepted = send(&mut rig, b, &response(a, b, group_id, true));
        assert!(has(&accepted, a, 40, 4));
        assert!(has(&accepted, a, 40, 9));
        assert!(has(&accepted, a, 40, 8));
        assert!(has(&accepted, b, 40, 4));
        assert!(has(&accepted, b, 40, 9));
        assert!(has(&accepted, b, 40, 8));
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a], GUIDS[b]].as_slice())
        );

        // B's channel-4 chat reaches A and echoes to B; C is not a member and hears nothing.
        let party_chat = send(&mut rig, b, &chat(ChatChannel::Group));
        assert_eq!(
            recipients(&party_chat),
            vec![rig.sessions[a].id.0, rig.sessions[b].id.0]
        );
        let line = social_chat::decode_packet_chat(inner(
            party_chat
                .iter()
                .find(|delivery| delivery.recipient == rig.sessions[a].id)
                .unwrap(),
        ))
        .unwrap();
        assert_eq!(line.sender_guid, GUIDS[b]);
        assert_eq!(line.sender_name.first, NAMES[b]);
        assert_eq!(line.channel, ChatChannel::Group);
        assert!(send(&mut rig, c, &chat(ChatChannel::Group)).is_empty());

        // Add C, then prove a non-leader cannot kick and the leader can.
        send(&mut rig, a, &invite(a, c, group_id));
        let accepted_c = send(&mut rig, c, &response(a, c, group_id, true));
        assert!(has(&accepted_c, a, 40, 8));
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a], GUIDS[b], GUIDS[c]].as_slice())
        );

        let refused_kick = send(&mut rig, b, &kick(a));
        assert_eq!(result_for(&refused_kick, b, 7), Some(2));
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a], GUIDS[b], GUIDS[c]].as_slice())
        );

        let kicked = send(&mut rig, a, &kick(c));
        assert!(has(&kicked, a, 40, 6));
        assert!(has(&kicked, b, 40, 6));
        assert!(has(&kicked, c, 40, 6));
        assert!(has(&kicked, a, 40, 8));
        assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[a], GUIDS[b]].as_slice())
        );
        assert_eq!(rig.world.chat_sessions[&rig.sessions[c].id].group_id, None);
        rig.sessions[c].spam_gate = SpamGate::default();
        assert!(send(&mut rig, c, &chat(ChatChannel::Group)).is_empty());

        // 40/3 leaves; with one member left the volatile party dissolves for both players.
        let left = send(&mut rig, b, &leave(b));
        assert!(has(&left, a, 40, 3));
        assert!(has(&left, b, 40, 3));
        assert_eq!(rig.world.social.group_of(GUIDS[a]), None);
        assert_eq!(rig.world.social.group_of(GUIDS[b]), None);
        assert!(send(&mut rig, a, &chat(ChatChannel::Group)).is_empty());
    }

    #[test]
    fn interleaved_invites_cannot_overlap_membership_or_group_chat() {
        let mut rig = rig("interleaved");
        let (a, b, c) = (0, 1, 2);

        // A invited B, but B created a different party and C joined it first.
        send(&mut rig, a, &invite(a, b, 0));
        let a_group = rig.world.social.group_of(GUIDS[a]).unwrap();
        send(&mut rig, b, &invite(b, c, 0));
        let b_group = rig.world.social.group_of(GUIDS[b]).unwrap();
        assert_ne!(a_group, b_group);
        send(&mut rig, c, &response(b, c, b_group, true));

        // B's stale acceptance of A's invite must leave both parties untouched.
        let refused = send(&mut rig, b, &response(a, b, a_group, true));
        assert_eq!(result_for(&refused, b, 5), Some(1));
        assert_eq!(rig.world.social.group_of(GUIDS[a]), Some(a_group));
        assert_eq!(rig.world.social.group_of(GUIDS[b]), Some(b_group));
        assert_eq!(rig.world.social.group_of(GUIDS[c]), Some(b_group));
        assert_eq!(
            rig.world.social.group_members(a_group),
            Some([GUIDS[a]].as_slice())
        );
        assert_eq!(
            rig.world.social.group_members(b_group),
            Some([GUIDS[b], GUIDS[c]].as_slice())
        );
        assert_eq!(
            rig.world.social.group_invitee(GUIDS[b]),
            Some((a_group, GUIDS[a]))
        );

        // Chat follows the same canonical mappings as both rosters.
        let b_chat = send(&mut rig, b, &chat(ChatChannel::Group));
        assert_eq!(
            recipients(&b_chat),
            vec![rig.sessions[b].id.0, rig.sessions[c].id.0]
        );
        let a_chat = send(&mut rig, a, &chat(ChatChannel::Group));
        assert_eq!(recipients(&a_chat), vec![rig.sessions[a].id.0]);

        // C leaving dissolves B's two-member party, not A's unrelated party.
        send(&mut rig, c, &leave(c));
        assert_eq!(rig.world.social.group_of(GUIDS[a]), Some(a_group));
        assert_eq!(rig.world.social.group_of(GUIDS[b]), None);
        assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
        assert_eq!(
            rig.world.social.group_members(a_group),
            Some([GUIDS[a]].as_slice())
        );
        assert_eq!(
            rig.world.social.group_invitee(GUIDS[b]),
            Some((a_group, GUIDS[a]))
        );
        assert_eq!(
            rig.world.chat_sessions[&rig.sessions[a].id].group_id,
            Some(u64::from(a_group))
        );
        assert_eq!(rig.world.chat_sessions[&rig.sessions[b].id].group_id, None);
        assert_eq!(rig.world.chat_sessions[&rig.sessions[c].id].group_id, None);
        rig.sessions[b].spam_gate = SpamGate::default();
        assert!(send(&mut rig, b, &chat(ChatChannel::Group)).is_empty());
    }

    #[test]
    fn gateway_tick_expires_an_idle_invite() {
        let mut rig = rig("tick-expiry");
        send(&mut rig, 0, &invite(0, 1, 0));
        let group_id = rig.world.social.group_of(GUIDS[0]).unwrap();
        assert_eq!(
            rig.world.social.group_invitee(GUIDS[1]),
            Some((group_id, GUIDS[0]))
        );

        let mut membership = World::runtime();
        membership
            .register_runtime_zone(osfr_tools::content::Zone {
                id: 1,
                name: String::new(),
                geometry_id: Some(1),
                zone_type: Some(2),
                tutorial: Some(false),
                sky: Some(String::new()),
                source: Vec::new(),
            })
            .unwrap();
        let mut sessions = SessionMap::new();
        rig.sessions[0]
            .tick(
                &rig.options,
                61_000,
                &mut rig.world,
                &mut membership,
                &mut sessions,
            )
            .unwrap();

        assert_eq!(rig.world.social.group_invitee(GUIDS[1]), None);
        assert_eq!(
            rig.world.social.group_members(group_id),
            Some([GUIDS[0]].as_slice())
        );
    }

    #[test]
    fn disconnect_dissolves_the_party_and_clears_party_chat_for_the_survivor() {
        let mut rig = rig("disconnect");
        let (a, b) = (0, 1);
        send(&mut rig, a, &invite(a, b, 0));
        let group_id = rig.world.social.group_of(GUIDS[a]).unwrap();
        send(&mut rig, b, &response(a, b, group_id, true));
        assert_eq!(
            rig.world.chat_sessions[&rig.sessions[a].id].group_id,
            Some(u64::from(group_id))
        );
        assert_eq!(
            rig.world.chat_sessions[&rig.sessions[b].id].group_id,
            Some(u64::from(group_id))
        );

        // This is the same order as `Server::end_session`: remove the player/chat row, then
        // disconnect the social state. The survivor must lose the dissolved channel too.
        rig.world.leave_player(rig.sessions[b].id);
        let delivered = rig.world.social_disconnect(GUIDS[b]).unwrap();
        assert!(
            delivered
                .iter()
                .any(|delivery| delivery.recipient == rig.sessions[a].id
                    && family_sub(delivery) == (40, 3))
        );
        assert_eq!(rig.world.social.group_of(GUIDS[a]), None);
        assert_eq!(rig.world.chat_sessions[&rig.sessions[a].id].group_id, None);
        assert!(send(&mut rig, a, &chat(ChatChannel::Group)).is_empty());
    }

    #[test]
    fn end_session_of_the_owner_dissolves_the_party_and_clears_survivor_chat() {
        let (mut server, keys, scratch) = server_rig("end-session-owner");
        let (a, b) = (0, 1);
        route_server(&mut server, keys[a], &invite(a, b, 0));
        let group_id = server.world.social.group_of(GUIDS[a]).unwrap();
        route_server(&mut server, keys[b], &response(a, b, group_id, true));
        assert_eq!(
            server.world.chat_sessions[&server.sessions[&keys[a]].id].group_id,
            Some(u64::from(group_id))
        );

        let session_b = server.sessions.remove(&keys[b]).unwrap();
        server
            .end_session(keys[b], session_b, "test_owner")
            .unwrap();

        assert_eq!(server.world.social.group_of(GUIDS[a]), None);
        assert_eq!(server.world.social.group_of(GUIDS[b]), None);
        assert_eq!(server.world.social.group_members(group_id), None);
        assert_eq!(
            server.world.chat_sessions[&server.sessions[&keys[a]].id].group_id,
            None
        );
        assert!(!server.world.active_sessions.contains_key(&GUIDS[b]));
        assert!(route_server(&mut server, keys[a], &chat(ChatChannel::Group)).is_empty());

        drop(server);
        fs::remove_dir_all(scratch).unwrap();
    }

    #[test]
    fn stale_non_owning_end_session_preserves_the_live_party_and_chat() {
        let (mut server, keys, scratch) = server_rig("end-session-stale");
        let (a, b) = (0, 1);
        route_server(&mut server, keys[a], &invite(a, b, 0));
        let group_id = server.world.social.group_of(GUIDS[a]).unwrap();
        route_server(&mut server, keys[b], &response(a, b, group_id, true));

        let live_b_id = server.sessions[&keys[b]].id;
        let mut stale = Session::new(
            SessionId(40),
            "127.0.0.1:12348".parse().unwrap(),
            SessionRequest {
                id: 40,
                crc_length: 3,
                buffer_size: 512,
            },
            &server.options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        stale.character_guid = GUIDS[b];
        stale.account_guid = GUIDS[b] + 10_000;
        let stale_key = session_key(&stale);
        server.world.session_keys.insert(stale.id, stale_key);
        server.sessions.insert(stale_key, stale);

        let memberships = GUIDS.map(|guid| server.world.social.group_of(guid));
        let roster = server
            .world
            .social
            .group_members(group_id)
            .unwrap()
            .to_vec();
        let chat_groups = [a, b]
            .map(|index| server.world.chat_sessions[&server.sessions[&keys[index]].id].group_id);

        let stale = server.sessions.remove(&stale_key).unwrap();
        server
            .end_session(stale_key, stale, "duplicate_session")
            .unwrap();

        assert!(server.sessions.contains_key(&keys[b]));
        assert_eq!(server.world.active_sessions[&GUIDS[b]], live_b_id);
        assert_eq!(
            GUIDS.map(|guid| server.world.social.group_of(guid)),
            memberships
        );
        assert_eq!(
            server.world.social.group_members(group_id),
            Some(roster.as_slice())
        );
        assert_eq!(
            [a, b].map(|index| {
                server.world.chat_sessions[&server.sessions[&keys[index]].id].group_id
            }),
            chat_groups
        );
        let party_chat = route_server(&mut server, keys[a], &chat(ChatChannel::Group));
        assert_eq!(
            recipients(&party_chat),
            vec![server.sessions[&keys[a]].id.0, live_b_id.0]
        );

        drop(server);
        fs::remove_dir_all(scratch).unwrap();
    }
}

/// W163: the binary's trade glue, driven through `Session::route_application` exactly as
/// `commit_outcome` does, with two sessions and no sockets. The commit is the lead's: with the
/// fail-closed `commit_trade` a finished trade must cancel and move nothing.
#[cfg(test)]
mod w163_trade_routing {
    use super::*;
    use osfr_tools::inventory::{InventoryItem, InventoryState};
    use osfr_tools::players::PlayerPresentation;
    use osfr_tools::trade::{TradeInventory, TradeItemDefinition};
    use std::collections::{BTreeMap, BTreeSet};

    const A: u64 = 101;
    const B: u64 = 202;

    struct Rig {
        a: Session,
        b: Session,
        world: SharedWorld,
        scratch: PathBuf,
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.scratch);
        }
    }

    fn row(id: i32, definition_id: i32, count: u32) -> InventoryItem {
        InventoryItem {
            definition_id,
            tint: 0,
            id,
            count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        }
    }

    fn inventory(coins: u32, items: Vec<InventoryItem>) -> TradeInventory {
        let next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
        TradeInventory::new(
            InventoryState {
                coins,
                items,
                equipment: Vec::new(),
                next_instance_id: Some(i64::from(next)),
            },
            BTreeMap::from([(
                100,
                TradeItemDefinition {
                    no_trade: false,
                    max_stack: Some(99),
                },
            )]),
            BTreeSet::new(),
            100,
        )
        .unwrap()
    }

    fn rig(name: &str) -> Rig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let scratch = std::env::temp_dir().join(format!("osfr-w163-{name}-{}", std::process::id()));
        fs::create_dir_all(&scratch).unwrap();
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join("character.json"),
            content: Some(root.join("references/rust-opening-cooking-original-assets")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: Some(scratch.join("world")),
            stock_text: None,
            allow_lan: false,
            security_log: None,
        };
        let session = |id: u64, guid: u64| {
            let mut session = Session::new(
                SessionId(id),
                "127.0.0.1:12345".parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    buffer_size: 512,
                },
                &options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap();
            session.character_guid = guid;
            session
        };
        let (a, b) = (session(17, A), session(18, B));
        let mut world = SharedWorld::new();
        for (session, guid, x, coins, items) in [
            (&a, A, 0.0, 50, vec![row(10, 100, 3)]),
            (&b, B, 5.0, 70, vec![]),
        ] {
            let presentation = PlayerPresentation {
                character_guid: guid,
                first_name: format!("P{guid}"),
                ..PlayerPresentation::default()
            };
            world
                .players
                .join_with_presentation(
                    session.id,
                    guid + 10_000,
                    guid,
                    ZoneInstance {
                        zone_id: 1,
                        instance_id: 1,
                    },
                    [x, 0.0, 0.0],
                    0.0,
                    presentation,
                )
                .unwrap();
            let identity = SocialIdentity {
                account_id: guid + 10_000,
                character_guid: guid,
                // Synthetic fixture identifiers, not recovered player names.
                name: SocialName::new(format!("P{guid}"), "").unwrap(),
            };
            world
                .social
                .register(SocialLists::new(identity).unwrap())
                .unwrap();
            world.social.login(guid).unwrap();
            world.joined.insert(session.id);
            let player = world.players.get(session.id).unwrap().clone();
            world
                .trades
                .register_player(&player, inventory(coins, items))
                .unwrap();
        }
        Rig {
            a,
            b,
            world,
            scratch,
        }
    }

    fn inner(delivery: &Delivery) -> &[u8] {
        assert_eq!(&delivery.bytes[..3], &[5, 0, 1]);
        &delivery.bytes[7..]
    }

    fn select(own: u64, target: u64) -> Vec<u8> {
        let mut bytes = vec![26, 0, 19, 0];
        bytes.extend(own.to_le_bytes());
        bytes.extend(target.to_le_bytes());
        bytes
    }

    fn choice(guid: u64, id: i32) -> Vec<u8> {
        let mut bytes = vec![26, 0, 10, 0];
        bytes.extend(guid.to_le_bytes());
        bytes.extend(id.to_le_bytes());
        bytes
    }

    fn pkt(sub: u8, session: u32, body: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0x3b, 0, sub];
        bytes.extend(session.to_le_bytes());
        bytes.extend(body);
        bytes
    }

    /// Every delivery as (recipient session, family-59 sub).
    fn subs(deliveries: &[Delivery]) -> Vec<(u64, u8)> {
        deliveries
            .iter()
            .map(|delivery| (delivery.recipient.0, inner(delivery)[2]))
            .collect()
    }

    #[test]
    fn radial_trade_entry_starts_an_invite_for_the_other_session() {
        let mut rig = rig("invite");
        let menu = rig
            .a
            .route_application(&select(A, B), &mut rig.world, 1_000);
        assert_eq!(menu.len(), 1);
        assert_eq!(menu[0].recipient, rig.a.id);
        let packet = inner(&menu[0]);
        assert_eq!(&packet[..4], &[26, 0, 9, 0]);
        // One entry, the Trade id: Inspect is absent because B has no public look here.
        assert_eq!(i32::from_le_bytes(packet[13..17].try_into().unwrap()), 1);
        assert_eq!(
            i32::from_le_bytes(packet[17..21].try_into().unwrap()),
            osfr_tools::player_radial::TRADE_ENTRY_ID
        );
        let invite = rig.a.route_application(
            &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
            &mut rig.world,
            1_001,
        );
        assert_eq!(subs(&invite), [(18, 2)], "the invite goes to B only");
        assert!(rig.world.trades.session_id_for(A).is_some());
        // Replaying the choice, or a second menu for a busy player, offers nothing.
        assert!(
            rig.a
                .route_application(
                    &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
                    &mut rig.world,
                    1_002
                )
                .is_empty()
        );
        assert!(
            rig.a
                .route_application(&select(A, B), &mut rig.world, 1_003)
                .is_empty()
        );
    }

    #[test]
    fn a_full_trade_ends_in_a_settlement_and_cancels_when_the_commit_refuses() {
        let mut rig = rig("full");
        rig.a
            .route_application(&select(A, B), &mut rig.world, 1_000);
        rig.a.route_application(
            &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
            &mut rig.world,
            1_000,
        );
        let id = rig.world.trades.session_id_for(A).unwrap();
        let started = rig
            .b
            .route_application(&pkt(3, id, &[1]), &mut rig.world, 2_000);
        let mut got = subs(&started);
        got.sort();
        assert_eq!(got, [(17, 4), (18, 4)]);

        let mut item = 10i32.to_le_bytes().to_vec();
        item.extend(2u32.to_le_bytes());
        let echoed = rig
            .a
            .route_application(&pkt(9, id, &item), &mut rig.world, 2_100);
        assert_eq!(subs(&echoed), [(17, 12), (18, 12)]);
        let coins =
            rig.b
                .route_application(&pkt(10, id, &30u32.to_le_bytes()), &mut rig.world, 2_200);
        assert_eq!(subs(&coins), [(17, 11), (18, 11)]);

        let counter = rig.world.trades.session_counter(id).unwrap();
        let mut lock = counter.to_le_bytes().to_vec();
        lock.push(1);
        rig.a
            .route_application(&pkt(8, id, &lock), &mut rig.world, 2_300);
        let confirm = rig
            .b
            .route_application(&pkt(8, id, &lock), &mut rig.world, 2_300);
        assert_eq!(
            subs(&confirm).iter().filter(|(_, sub)| *sub == 18).count(),
            2
        );
        let accept = pkt(6, id, &counter.to_le_bytes());
        assert!(
            rig.a
                .route_application(&accept, &mut rig.world, 12_300)
                .is_empty()
        );
        let before = [
            rig.world.trades.inventory(A).unwrap().clone(),
            rig.world.trades.inventory(B).unwrap().clone(),
        ];
        // The second accept queues a settlement for `settle_trades`; nothing is sent or moved
        // while staging, so a staged world that is dropped leaves no half trade behind.
        assert!(
            rig.b
                .route_application(&accept, &mut rig.world, 12_300)
                .is_empty()
        );
        assert_eq!(rig.world.trade_settlements.len(), 1);
        let settlement = rig.world.trade_settlements[0].clone();
        assert_eq!(settlement.session_id, id);
        assert_eq!(settlement.sides[0].after.coins, 80);
        assert_eq!(settlement.sides[1].after.coins, 40);
        assert_eq!(rig.world.trades.inventory(A).unwrap(), &before[0]);

        // This rig's sessions hold no live inventory equal to the settlement's `before`, so the
        // lead's commit refuses: both are told "canceled" (status 1), the window closes (59/5)
        // and nothing in either inventory changed (the commit's own tests cover success).
        let end = settle_trades(&mut rig.a, &mut SessionMap::new(), &mut rig.world).unwrap();
        assert_eq!(subs(&end).len(), 4);
        for pair in end.chunks(2) {
            assert_eq!(inner(&pair[0])[2], 15);
            assert_eq!(inner(&pair[0])[3], 1);
            assert_eq!(inner(&pair[1])[2], 5);
        }
        assert!(rig.world.trade_settlements.is_empty());
        assert_eq!(rig.world.trades.inventory(A).unwrap(), &before[0]);
        assert_eq!(rig.world.trades.inventory(B).unwrap(), &before[1]);
        assert!(rig.world.trades.session_id_for(A).is_none());
        // Replays after the end only get the stale-window close, to the sender.
        let replay = rig.a.route_application(&accept, &mut rig.world, 13_000);
        assert_eq!(subs(&replay), [(17, 5)]);
        assert!(
            settle_trades(&mut rig.a, &mut SessionMap::new(), &mut rig.world)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn leaving_the_world_or_walking_away_ends_the_trade_for_the_partner() {
        let mut rig = rig("leave");
        rig.a
            .route_application(&select(A, B), &mut rig.world, 1_000);
        rig.a.route_application(
            &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
            &mut rig.world,
            1_000,
        );
        let id = rig.world.trades.session_id_for(A).unwrap();
        rig.b
            .route_application(&pkt(3, id, &[1]), &mut rig.world, 1_100);
        // B walks 60 m away: the housekeeping `commit_outcome` runs after each step ends it.
        rig.world
            .players
            .update_pose(
                rig.b.id,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [60.0, 0.0, 0.0],
                0.0,
            )
            .unwrap();
        let end = trade_deliveries(rig.world.trades.maintain(
            &rig.world.players,
            &rig.world.social,
            1_200,
        ));
        assert_eq!(subs(&end).len(), 4);
        assert!(rig.world.trades.session_id_for(A).is_none());

        // A second trade, ended by A leaving (disconnect or zoning both call leave_player).
        rig.world
            .players
            .update_pose(
                rig.b.id,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [5.0, 0.0, 0.0],
                0.0,
            )
            .unwrap();
        rig.a
            .route_application(&select(A, B), &mut rig.world, 2_000);
        rig.a.route_application(
            &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
            &mut rig.world,
            2_000,
        );
        let id = rig.world.trades.session_id_for(A).unwrap();
        rig.b
            .route_application(&pkt(3, id, &[1]), &mut rig.world, 2_100);
        let gone = rig.world.leave_player(rig.a.id);
        let to_b: Vec<_> = gone
            .iter()
            .filter(|delivery| {
                delivery.recipient == rig.b.id
                    && delivery.bytes.starts_with(&[5, 0, 1])
                    && inner(delivery)[..2] == [0x3b, 0]
            })
            .map(|delivery| inner(delivery)[2])
            .collect();
        assert_eq!(to_b, [15, 5], "status then end for the partner");
        assert!(rig.world.trades.session_id_for(B).is_none());
        assert!(rig.world.trades.inventory(A).is_none());
    }

    #[test]
    fn hostile_trade_packets_from_one_session_cannot_reach_the_other() {
        let mut rig = rig("hostile");
        rig.a
            .route_application(&select(A, B), &mut rig.world, 1_000);
        rig.a.route_application(
            &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
            &mut rig.world,
            1_000,
        );
        let id = rig.world.trades.session_id_for(A).unwrap();
        rig.b
            .route_application(&pkt(3, id, &[1]), &mut rig.world, 1_100);
        let before = rig.world.trades.clone();
        // A nobody session with the right ids: answered to itself only, state untouched.
        let mut stranger = rig
            .b
            .route_application(&pkt(7, id + 9, &[]), &mut rig.world, 1_200);
        stranger.extend(
            rig.b
                .route_application(&pkt(6, 0, &[0; 4]), &mut rig.world, 1_200),
        );
        assert_eq!(subs(&stranger), [(18, 5)]);
        // Truncated, oversized and server-only subs are dropped.
        for bad in [
            vec![0x3b, 0, 9, 1, 0],
            pkt(9, id, &[0; 9]),
            pkt(5, id, &[]),
            pkt(2, id, &[0; 8]),
            vec![0x3b, 0],
        ] {
            assert!(
                rig.a
                    .route_application(&bad, &mut rig.world, 1_300)
                    .is_empty()
            );
        }
        assert_eq!(rig.world.trades, before);
    }

    #[test]
    fn a_loaded_gameplay_inventory_is_registered_and_kept_current_for_trading() {
        let mut rig = rig("refresh");
        let mut login = 1u16.to_le_bytes().to_vec();
        for (i, value) in ["w163", "1.910.1.530630", ""].into_iter().enumerate() {
            login.extend((value.len() as u32).to_le_bytes());
            login.extend(value.as_bytes());
            if i == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        rig.a.gateway.handle(&login).unwrap();
        rig.a.gateway.handle(&wrap_application(&[13, 0])).unwrap();
        rig.a.gateway.handle(&wrap_application(&[10, 0])).unwrap();
        let loaded = rig
            .a
            .gateway
            .trade_inventory()
            .expect("a loaded gameplay validates as a trade inventory");
        // Register, then sync: the service ends up with exactly the gameplay's inventory.
        rig.world.trades.unregister_player(A);
        assert!(refresh_trade_player(&mut rig.world, rig.a.id, &rig.a.gateway).is_empty());
        assert_eq!(rig.world.trades.inventory(A), Some(&loaded.state));
        assert!(refresh_trade_player(&mut rig.world, rig.a.id, &rig.a.gateway).is_empty());
        assert_eq!(rig.world.trades.inventory(A), Some(&loaded.state));
        // A session without loaded gameplay is not tradable: it is unregistered.
        rig.world.trades.unregister_player(B);
        assert!(rig.world.trades.inventory(B).is_none());
        refresh_trade_player(&mut rig.world, rig.b.id, &rig.b.gateway);
        assert_eq!(
            rig.world.trades.inventory(B).is_some(),
            rig.b.gateway.trade_inventory().is_some()
        );
    }
}

/// Lead (security): the journaled two-character trade commit on real gateways.
#[cfg(test)]
mod lead_trade_commit {
    use super::*;
    use osfr_tools::inventory::InventoryState;
    use osfr_tools::trade::{SettlementSide, TradeSettlement};

    fn scratch(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("osfr-trade-commit-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn party(scratch: &Path, id: u64, guid: u64) -> Session {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: root.join("references/rust-gateway-reference"),
            state: scratch.join(format!("{guid}.json")),
            content: Some(root.join("references/rust-opening-cooking-original-assets")),
            identity: None,
            account_store: None,
            gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
            handoff_key: Vec::new(),
            seconds: 1,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: None,
            stock_text: None,
            allow_lan: false,
            security_log: None,
        };
        let mut session = Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 20_000 + id).parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 512,
            },
            &options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        session.character_guid = guid;
        session.selected_character = true;
        session.state_path = options.state;
        session
    }

    fn with_coins(session: &mut Session, coins: u32) -> InventoryState {
        let mut state = session.gateway.inventory_state().unwrap().clone();
        state.coins = coins;
        session
            .gateway
            .adopt_trade_inventory(state.clone())
            .unwrap();
        state
    }

    /// A gives B 10 coins.
    fn settlement(a: &InventoryState, b: &InventoryState) -> TradeSettlement {
        let side = |guid, before: &InventoryState, delta: i64| {
            let mut after = before.clone();
            after.coins = (i64::from(before.coins) + delta) as u32;
            SettlementSide {
                guid,
                before: before.clone(),
                after,
                gave: Vec::new(),
                received: Vec::new(),
            }
        };
        TradeSettlement {
            session_id: 7,
            sides: [side(101, a, -10), side(202, b, 10)],
        }
    }

    fn saved_coins(path: &Path) -> u64 {
        let document: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        document["gameplay"]["inventory"]["coins"].as_u64().unwrap()
    }

    fn leftovers(directory: &Path) -> Vec<String> {
        fs::read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .filter(|name| name.starts_with("trade-"))
            .collect()
    }

    #[test]
    fn a_settled_trade_moves_coins_in_both_saves_and_both_live_sessions() {
        let dir = scratch("ok");
        let mut a = party(&dir, 1, 101);
        let mut b = party(&dir, 2, 202);
        let before_a = with_coins(&mut a, 25);
        let before_b = with_coins(&mut b, 5);
        let key = SessionKey {
            peer: b.peer,
            connection_id: b.request.id,
        };
        let mut taken = vec![(key, b)];
        let deliveries =
            commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken)
                .unwrap()
                .expect("a matching trade commits");
        assert_eq!(a.gateway.inventory_state().unwrap().coins, 15);
        assert_eq!(taken[0].1.gateway.inventory_state().unwrap().coins, 15);
        assert_eq!(saved_coins(&a.state_path), 15);
        assert_eq!(saved_coins(&taken[0].1.state_path), 15);
        // Each player is told their new balance (38/19); no journal is left behind.
        for id in [a.id, taken[0].1.id] {
            assert!(deliveries.iter().any(|delivery| delivery.recipient == id
                && delivery.bytes.windows(4).any(|w| w == [38, 0, 19, 0])));
        }
        assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_stale_trade_moves_nothing_and_writes_nothing() {
        let dir = scratch("stale");
        let mut a = party(&dir, 3, 101);
        let mut b = party(&dir, 4, 202);
        let before_a = with_coins(&mut a, 25);
        let before_b = with_coins(&mut b, 5);
        // A spends coins after the offers were validated: the settlement is stale.
        with_coins(&mut a, 24);
        let saves = |a: &Session, b: &Session| {
            [&a.state_path, &b.state_path].map(|path| fs::read(path).ok())
        };
        let before_saves = saves(&a, &b);
        let key = SessionKey {
            peer: b.peer,
            connection_id: b.request.id,
        };
        let mut taken = vec![(key, b)];
        let result =
            commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken).unwrap();
        assert!(result.is_none(), "a stale trade must be refused");
        assert_eq!(a.gateway.inventory_state().unwrap().coins, 24);
        assert_eq!(taken[0].1.gateway.inventory_state().unwrap().coins, 5);
        assert_eq!(saves(&a, &taken[0].1), before_saves);
        assert!(leftovers(&dir).is_empty());
        // Canary: the same settlement against the state it was made for does commit.
        with_coins(&mut a, 25);
        assert!(
            commit_trade_parties(&settlement(&before_a, &before_b), &mut a, &mut taken)
                .unwrap()
                .is_some()
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn an_interrupted_trade_is_finished_from_its_journal_at_start() {
        let dir = scratch("recover");
        let mut a = party(&dir, 5, 101);
        let mut b = party(&dir, 6, 202);
        with_coins(&mut a, 15);
        with_coins(&mut b, 15);
        // The crash happened after the journal and before either save.
        let journal: Vec<TradeJournalEntry> = [&a, &b]
            .iter()
            .map(|party| TradeJournalEntry {
                path: party.state_path.clone(),
                snapshot: party.gateway.snapshot(),
            })
            .collect();
        fs::write(
            dir.join(format!("trade-7-1.{TRADE_JOURNAL_EXTENSION}")),
            serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();
        // A half-written journal means the trade never started writing saves.
        fs::write(dir.join("trade-8-2.tmp"), b"{ half").unwrap();
        assert_eq!(recover_trade_journals(&dir).unwrap(), 1);
        assert_eq!(saved_coins(&a.state_path), 15);
        assert_eq!(saved_coins(&b.state_path), 15);
        assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));
        // Running recovery again is harmless.
        assert_eq!(recover_trade_journals(&dir).unwrap(), 0);
        fs::remove_dir_all(dir).unwrap();
    }
}
