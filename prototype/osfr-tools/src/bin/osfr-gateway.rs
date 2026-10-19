//! One local character and zone, using private reference initialization data.
//! Rust owns the session, reliable transport, application dispatch and snapshot.
use osfr_tools::trace;

use osfr_tools::account_store::AccountStore;
use osfr_tools::character_store::CharacterState;
use osfr_tools::fishing_service::FishingHourLedger;
use osfr_tools::game_rules::{GameRuleSet, MembershipState, RulePolicy};
use osfr_tools::game_store::{load_or_create, save};
use osfr_tools::gameplay::Gameplay;
use osfr_tools::gateway::{Gateway, GatewayOutcome};
use osfr_tools::login;
use osfr_tools::mounts::{MountTail, RidePose, RideView};
use osfr_tools::net_policy::{Bucket, IpLimiter, KeyedBuckets};
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
    panic::{AssertUnwindSafe, catch_unwind},
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
    /// (R) Rolling-hour fishing payout timestamps keyed by character. Server-owned and shared
    /// across reconnects; durable persistence is a save-schema decision for the lead.
    fishing_hours: FishingHourLedger,
    social: SocialService,
    identities: BTreeMap<u64, SocialIdentity>,
    social_paths: BTreeMap<u64, PathBuf>,
    session_keys: BTreeMap<SessionId, SessionKey>,
    active_sessions: BTreeMap<u64, SessionId>,
    chat_sessions: BTreeMap<SessionId, ChatMember>,
    chat_history: ChatHistory,
    /// W173: each joined character's channel switches and bubble colour (26/32, 26/33, 26/18).
    chat_settings: social_chat::ChatSettingsBook,
    security: Arc<Mutex<Security>>,
}

/// Live chat identity for one session. Zone, position and group are derived from the player
/// registry and social at routing time and must never be cached here.
#[derive(Clone, Debug)]
struct ChatMember {
    name: NameData,
    is_member: bool,
}

/// Security Model L6/L9 state. Every staged clone of the world shares it, so a
/// rolled-back message neither duplicates nor refunds an account's budget.
struct Security {
    budgets: social_chat::ActionBudgets,
    combat: KeyedBuckets<u64>,
    log: Option<SecurityLog>,
}

impl Default for Security {
    fn default() -> Self {
        Self {
            budgets: social_chat::ActionBudgets::default(),
            // (R) Shared, nonrefundable account budget, 10/s burst 20; bounded keys.
            combat: KeyedBuckets::new(10., 20., MAX_SESSIONS),
            log: None,
        }
    }
}

impl std::fmt::Debug for Security {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Security")
    }
}

impl Security {
    fn allow_combat(&mut self, account: u64, now: Instant) -> bool {
        self.combat.allow(account, now)
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

/// Live appearance updates name a player and are safe only for viewers who
/// already received that player's AddPc.
fn live_outfit_packet(packet: &[u8]) -> bool {
    // 35/7 reader 0x0091af50; 35/6 is T027's existing equipment-change prefix.
    matches!(packet, [35, 0, 6, 0, ..] | [35, 0, 7, 0, ..])
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
            fishing_hours: FishingHourLedger::default(),
            social: SocialService::new(17).unwrap(),
            identities: BTreeMap::new(),
            social_paths: BTreeMap::new(),
            session_keys: BTreeMap::new(),
            active_sessions: BTreeMap::new(),
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
        self.chat_sessions.insert(
            session.id,
            ChatMember {
                name: chat_name,
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

    /// W162: `packets` for other players in interest range of `session`. Live appearance updates
    /// also require the recipient's directed view to contain the actor. The registry position is
    /// read before a zone transfer removes the player.
    fn ride_fanout(&self, session: SessionId, packets: &[Vec<u8>]) -> Vec<Delivery> {
        let Some(player) = self.players.get(session) else {
            return Vec::new();
        };
        let mut deliveries = Vec::new();
        for other in self.players.nearby_sessions(player.zone, player.position) {
            if other == session {
                continue;
            }
            let knows_character = self.players.knows_character(other, player.character_guid);
            for packet in packets {
                if knows_character || !live_outfit_packet(packet) {
                    deliveries.push(Delivery {
                        recipient: other,
                        bytes: wrap_application(packet),
                    });
                }
            }
        }
        deliveries
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

    /// Drop a disconnected character's volatile group state. Chat scope is derived from the
    /// current social service when a message is routed, so survivors need no refresh.
    fn social_disconnect(&mut self, guid: u64) -> io::Result<Vec<Delivery>> {
        let messages = self.social.disconnect(guid).map_err(invalid_io)?;
        Ok(messages
            .into_iter()
            .filter_map(|message| self.social_delivery(message))
            .collect())
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

    /// Client PacketChat reader 0x00b140c0 -> 0x00b11600 consumes channel, sender/target
    /// identity, message and position; the live player record owns the spatial fields.
    fn chat_session(&self, session_id: SessionId) -> Option<ChatSession> {
        let player = self.players.get(session_id)?;
        let member = self.chat_sessions.get(&session_id)?;
        Some(ChatSession {
            session_id: session_id.0,
            character_guid: player.character_guid,
            name: member.name.clone(),
            zone_id: player.zone.zone_id,
            position: player.position,
            chat_area_id: None,
            group_id: self.social.group_of(player.character_guid).map(u64::from),
            guild_id: None,
            is_member: member.is_member,
        })
    }

    fn chat_candidates(
        &self,
        sender: SessionId,
        channel: social_chat::ChatChannel,
    ) -> Vec<ChatSession> {
        // Say and Area stay inside the sender's instance; Yell is zone-wide (R).
        let local_instance = matches!(
            channel,
            social_chat::ChatChannel::Say | social_chat::ChatChannel::Area
        );
        let zone = self.players.get(sender).map(|player| player.zone);
        self.chat_sessions
            .keys()
            .filter(|id| {
                !local_instance
                    || zone.is_some_and(|zone| {
                        self.players
                            .get(**id)
                            .is_some_and(|player| player.zone == zone)
                    })
            })
            .filter_map(|id| self.chat_session(*id))
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
        // The one redaction rule; a bundle can hide a chat child, so bundles stay hidden too.
        let hex = trace::hex_prefix_len(summary.body, summary.sensitive || summary.combined, 96);
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
        staged_gateway.restore_fishing_hours(world.fishing_hours.history(self.character_guid))?;
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
        let transfer = outcome.began_zoning || previous.zone_id != current.zone_id;
        let prepared = (|| -> Result<_, Box<dyn std::error::Error>> {
            let pending = staged_gateway.take_pending_transfer();
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
            // Lead (security): only the session that owns this character publishes its
            // fishing history. A handshake session still carries the template guid, which
            // equals the first account's, so an unauthenticated tick must not write the cap.
            let owns_character = self.selected_character
                && staged_gateway.state().guid == self.character_guid
                && staged_world.active_sessions.get(&self.character_guid) == Some(&self.id);
            if owns_character {
                staged_world.fishing_hours.publish(
                    self.character_guid,
                    staged_gateway.fishing_hour_history(),
                    now_ms,
                );
            }
            let mut deliveries = staged_world.ride_fanout(self.id, &outcome.nearby);
            if transfer && staged_world.players.contains(self.id) {
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
            if outcome.gameplay_changed && staged_world.players.contains(self.id) {
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
                && !staged_world.players.contains(self.id)
            {
                let look = staged_gateway.player_presentation(self.is_member)?;
                deliveries.extend(staged_world.join_player(self, &current, look, instance)?);
                refresh_public_look(&mut staged_world, current.guid, &staged_gateway);
                deliveries.extend(refresh_trade_player(
                    &mut staged_world,
                    self.id,
                    &staged_gateway,
                ));
            } else if moved && staged_world.players.contains(self.id) {
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
            }

            if staged_world.players.contains(self.id) {
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
                // Lead (persistence): a pet change saves character and roster in ONE atomic
                // write. Two writes could crash between them and keep a consumed adoption item
                // without its pet, or the pet without spending the item.
                if outcome.pets_changed {
                    staged_gateway.save_with_pet_roster(&self.state_path)?;
                } else {
                    save(&self.state_path, &staged_gateway.snapshot())?;
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
        let Some(sender) = world.chat_session(self.id) else {
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
        let Some(sender) = world.chat_session(self.id) else {
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
        let zone = self.gateway.state().zone_id;
        // A seeded field has the same kill id in every instance, so the claim is
        // per instance; without a committed place there is nobody to heal.
        let Some(instance) = membership
            .player_location(self.character_guid)
            .map(|l| l.instance_id)
        else {
            return Ok(());
        };
        if !membership.claim_group_heal(instance, heal.kill) {
            return Ok(());
        }
        let members = world
            .social
            .group_members(group)
            .map(<[u64]>::to_vec)
            .unwrap_or_default();
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
                && membership.player_location(guid).map(|l| l.instance_id) == Some(instance);
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
        #[cfg(test)]
        if isolation::PANIC_IN_TICK.get() == Some(self.id) {
            panic!("isolation probe");
        }
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
    /// Shared fields whose tick faulted: frozen until the instance empties and is collected.
    faulted_fields: BTreeSet<i32>,
    /// Shared fields with nobody in them that already settled: not ticked until someone enters.
    dormant_fields: BTreeSet<i32>,
    start: Instant,
    generation: u32,
    trace: Option<trace::Trace>,
    studio: Option<TcpListener>,
    // Studio asset browser: index files loaded on first search.
    studio_assets: BTreeMap<String, Vec<serde_json::Value>>,
}

impl Server {
    /// A field whose tick panics or errors is frozen (logged once) until it empties and is
    /// collected; the server and every other field keep running.
    ///
    /// Dormant by default: a field nobody is in (at any readiness) ticks once more, which drops
    /// departed casters' DoTs and kill credits, then sleeps until someone enters. Its clocks are
    /// absolute and motion clamps the first elapsed step, so waking needs no catch-up.
    fn tick_shared_fields(&mut self, now_ms: u64) {
        let instances = self.membership.field_instances();
        self.faulted_fields
            .retain(|instance| instances.contains(instance));
        self.dormant_fields
            .retain(|instance| instances.contains(instance));
        for instance in instances {
            if self.faulted_fields.contains(&instance) {
                continue;
            }
            let occupied = self.sessions.values().any(|session| {
                self.membership
                    .player_location(session.character_guid)
                    .is_some_and(|l| l.instance_id == instance)
            });
            if occupied {
                self.dormant_fields.remove(&instance);
            } else if !self.dormant_fields.insert(instance) {
                // Already settled; drop stale deltas so a later arrival never replays them.
                self.membership.take_field_packets(instance);
                continue;
            }
            if let Err(error) = contain(|| self.tick_shared_instance(instance, now_ms)) {
                self.faulted_fields.insert(instance);
                event(
                    json!({"event":"field_faulted","instance":instance,"reason":error.to_string()}),
                );
            }
        }
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
            let Some((location_instance, ready)) = self
                .membership
                .player_location(session.character_guid)
                .map(|l| (l.instance_id, l.ready))
            else {
                continue;
            };
            if location_instance != instance {
                continue;
            }
            let valid = ready && session.gateway.is_world_loaded();
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
            if valid
                && session.npc_visibility_revision == Some(revision)
                && session.npc_view_position == Some(session.gateway.state().position)
                && session.npc_view_fields == fields
            {
                continue;
            }
            // A session that loses readiness or its loaded world remains in this
            // instance until re-entry. Remove actors it still knows through the
            // original 35/3 path (client dispatcher 0x00930714, reader 0x008d69f0).
            // Sessions in another instance keep their own view and are reconciled
            // by that instance.
            let (wanted, packets) = if valid {
                session
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
                                refreshed
                                    .push(osfr_tools::encounter::SharedFieldState::removal(*guid));
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
                    .unwrap_or_default()
            } else {
                (
                    BTreeSet::new(),
                    session
                        .known_npcs
                        .iter()
                        .map(|guid| osfr_tools::encounter::SharedFieldState::removal(*guid))
                        .collect(),
                )
            };
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
            session.npc_view_position = valid.then_some(session.gateway.state().position);
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
            faulted_fields: BTreeSet::new(),
            dormant_fields: BTreeSet::new(),
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
    pub fn test_mark_not_ready(&mut self, guid: u64) {
        let id = self.world.active_sessions[&guid];
        let key = self.world.session_keys[&id];
        self.sessions.get_mut(&key).unwrap().gateway.reset_session();
        let location = self.membership.player_location(guid).unwrap().clone();
        let zone_id = self
            .membership
            .instance(location.instance_id)
            .map(|row| row.zone_id)
            .unwrap();
        let change = self
            .membership
            .prepare_membership(
                guid,
                zone_id,
                Some(location.instance_id),
                location.pose,
                false,
                self.start.elapsed().as_millis() as u64,
            )
            .unwrap();
        self.membership.commit_membership(change);
    }
    #[cfg(test)]
    pub fn test_known_npcs(&self, guid: u64) -> BTreeSet<u64> {
        let id = self.world.active_sessions[&guid];
        self.sessions[&self.world.session_keys[&id]]
            .known_npcs
            .clone()
    }
    #[cfg(test)]
    pub fn test_pickup_count(&self, guid: u64) -> Option<usize> {
        let id = self.world.active_sessions.get(&guid)?;
        Some(
            self.sessions
                .get(self.world.session_keys.get(id)?)?
                .gateway
                .gameplay()?
                .pickup_count(),
        )
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
                self.end_session(key, session, "server_shutdown");
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
                                if let Err(error) = contain(|| self.accept_handshake(peer, request))
                                {
                                    event(
                                        json!({"event":"handshake_failed","peer":peer.to_string(),"reason":error.to_string()}),
                                    );
                                }
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
                                let result = contain(|| {
                                    current.handle(
                                        &self.socket,
                                        wire,
                                        &self.options,
                                        now_ms,
                                        &mut self.world,
                                        &mut self.membership,
                                        &mut self.sessions,
                                    )
                                });
                                match result {
                                    Ok(false) => {
                                        self.sessions.insert(key, current);
                                    }
                                    Ok(true) => self.end_session(key, current, "client_disconnect"),
                                    Err(error) => {
                                        self.end_session(key, current, &error.to_string())
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
            self.tick_shared_fields(now_ms);
        }
        let keys = self.sessions.keys().copied().collect::<Vec<_>>();
        for key in keys {
            let Some(mut current) = self.sessions.remove(&key) else {
                continue;
            };
            let result = contain(|| {
                if tick_due {
                    current.tick(
                        &self.options,
                        now_ms,
                        &mut self.world,
                        &mut self.membership,
                        &mut self.sessions,
                    )?;
                }
                current.flush(&self.socket, now_ms)
            });
            if let Err(error) = result {
                self.end_session(key, current, &error.to_string());
            } else if current.receiver.expire_reassembly(Instant::now()) {
                // A fragmented message left incomplete past its fixed deadline
                // (KeepAlive cannot extend it; Security Model L7).
                self.end_session(key, current, "reassembly_timeout");
            } else if !current.selected_character
                && current.session_start.elapsed() > LOGIN_DEADLINE
            {
                // Not logged in yet: KeepAlive does not extend this (Security Model L6).
                self.end_session(key, current, "login_deadline");
            } else if current.last_activity.elapsed() > Duration::from_secs(60) {
                self.end_session(key, current, "idle_timeout");
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
                let Some(inner) = trace::parse_hex(&hex).filter(|b| b.len() <= 64 * 1024) else {
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
                    self.end_session(key, session, "studio_savestate_load");
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
                if let Err(error) = osfr_tools::character_store::replace_bytes(&path, &request.body)
                {
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
            self.end_session(old_key, old, "connection_replaced");
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

    /// Never fails: a stage that errors is logged and the rest still run, so a failed save
    /// cannot leave a ghost player in the world or stop the server.
    fn end_session(&mut self, key: SessionKey, mut session: Session, reason: &str) {
        let fault = |stage: &str, error: String| {
            event(
                json!({"event":"session_end_fault","peer":key.peer.to_string(),"stage":stage,"reason":error}),
            );
        };
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
            let saved = contain(|| Ok(save(&session.state_path, &session.gateway.snapshot())?));
            if let Err(error) = saved {
                fault("save", error.to_string());
            }
        }
        let mut deliveries = self.world.leave_player(session.id);
        let owns_active =
            self.world.active_sessions.get(&session.character_guid) == Some(&session.id);
        if owns_active && self.world.social.profile(session.character_guid).is_some() {
            match self.world.social_disconnect(session.character_guid) {
                Ok(more) => deliveries.extend(more),
                Err(error) => fault("social", error.to_string()),
            }
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
            if delivery.recipient != session.id
                && let Err(error) =
                    queue_delivery(&mut session, &mut self.sessions, &self.world, delivery)
            {
                fault("delivery", error.to_string());
            }
        }
        event(json!({"event":"session_ended","peer":key.peer.to_string(),"reason":reason}));
        if let Some(trace) = &session.trace
            && let Err(error) = trace.record(json!({
                "event":"session_ended","session":session.id.0,"peer":key.peer.to_string(),
                "direction":"local","outcome":"closed","reason":reason
            }))
        {
            fault("trace", error.to_string());
        }
    }
}

/// The isolation boundary: a panic in `work` becomes an "internal_error" for its caller, which
/// ends that session (saved as it stands) or freezes that field. The panic text still reaches
/// stderr through the default hook.
// ponytail: a panic part-way through a cross-session handler (trade, group) can drop a partner
// session it had taken out of the map; reconcile `world.session_keys` if that is ever seen.
fn contain<T>(
    work: impl FnOnce() -> Result<T, Box<dyn std::error::Error>>,
) -> Result<T, Box<dyn std::error::Error>> {
    catch_unwind(AssertUnwindSafe(work)).unwrap_or_else(|_| Err("internal_error".into()))
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
#[path = "osfr_gateway/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "osfr_gateway/f120_fishing_hour_reconnect.rs"]
mod f120_fishing_hour_reconnect;

#[cfg(test)]
#[path = "osfr_gateway/isolation.rs"]
mod isolation;

#[cfg(test)]
#[path = "osfr_gateway/dormant_fields.rs"]
mod dormant_fields;

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

/// Pins the shared combat budget's rate (10/s), burst (20) and key cap
/// (`MAX_SESSIONS`): sixteen busy accounts refuse a seventeenth, 100 ms of idle
/// time returns exactly one token, and two seconds of idle time lets the
/// seventeenth in by evicting the refilled buckets.
#[cfg(test)]
#[test]
fn shared_combat_budget_pins_rate_burst_and_session_cap() {
    let world = SharedWorld::new();
    let allow =
        |account: u64, at: Instant| world.security.lock().unwrap().allow_combat(account, at);
    let now = Instant::now();
    for account in 1..=MAX_SESSIONS as u64 {
        assert!(allow(account, now), "account {account}");
    }
    let seventeenth = MAX_SESSIONS as u64 + 1;
    assert!(!allow(seventeenth, now));
    assert_eq!(world.security.lock().unwrap().combat.len(), MAX_SESSIONS);

    // Drain account 1, then 10/s returns exactly one token after 100 ms.
    for _ in 1..20 {
        assert!(allow(1, now));
    }
    let refilled = now + Duration::from_millis(100);
    assert!(allow(1, refilled));
    assert!(!allow(1, refilled));

    // Every bucket that can pay a whole burst is idle and evicted.
    let idle = now + Duration::from_secs(2);
    assert!(allow(seventeenth, idle));
    assert!(world.security.lock().unwrap().combat.len() <= MAX_SESSIONS);
}

/// W173: the chat channel switches and bubble colour (26/32, 26/33, 26/18) through
/// `Session::route_application` with three sessions and no sockets.
#[cfg(test)]
#[path = "osfr_gateway/w173_chat_settings_routing.rs"]
mod w173_chat_settings_routing;

/// T020: family-40 grouping and group chat through `Session::route_application` with three
/// sessions and no sockets. The two clients that form the party are the gate; the third session
/// proves that group chat and roster events stay inside the authenticated group.
#[cfg(test)]
#[path = "osfr_gateway/t020_group_routing.rs"]
mod t020_group_routing;

/// W163: the binary's trade glue, driven through `Session::route_application` exactly as
/// `commit_outcome` does, with two sessions and no sockets. The commit is the lead's: with the
/// fail-closed `commit_trade` a finished trade must cancel and move nothing.
#[cfg(test)]
#[path = "osfr_gateway/w163_trade_routing.rs"]
mod w163_trade_routing;

/// Lead (security): the journaled two-character trade commit on real gateways.
#[cfg(test)]
#[path = "osfr_gateway/lead_trade_commit.rs"]
mod lead_trade_commit;
