//! Per-character application gateway backed by a private, verified reference.
//!
//! Field layouts were checked against the supplied serializer and pinned
//! Sanctuary packet definitions. Startup messages are built from typed data,
//! not replayed captured bytes. This module never runs C# code.

use crate::bundle_budget::BundleBudget;
use crate::character_store::CharacterState;
use crate::dev_teleport;
use crate::encounter::{ZoneTransition, zone_begin_packet};
use crate::game_store::Snapshot;
use crate::gameplay::{Gameplay, GameplayState};
use crate::identity::CharacterIdentity;
use crate::movement::{
    self, Decision as MovementDecision, ExpectedDiscontinuity, MovementState, MovementUpdate,
};
use crate::{
    players::PlayerPresentation, startup_selfdata::SelfData, startup_snapshot::StartupSnapshot,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod admin_move;
mod application_preload;
mod companions;
mod encounter_join;
mod group_arena;
mod staff;
mod startup_source;
mod startup_stream;
mod trade_bridge;

const MAX_MESSAGE: usize = 1_048_576;
const MAX_REFERENCE: usize = 16_777_216;
const MAX_BUNDLE_CHILDREN: usize = 128;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Login,
    Ready,
}

#[derive(Deserialize)]
struct TemplateFile {
    kind: Kind,
    file: String,
    length: usize,
    sha256: String,
    opcode: u16,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
struct SelfLayout {
    guid_offset: usize,
    position_offset: usize,
    rotation_offset: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
struct ZoneLayout {
    zone_id_offset: usize,
}

#[derive(Deserialize)]
struct Manifest {
    schema_version: u32,
    client_version: String,
    initial_state: CharacterState,
    self_layout: SelfLayout,
    zone_layout: ZoneLayout,
    templates: Vec<TemplateFile>,
}

#[derive(Deserialize)]
struct ProfileLayoutFile {
    schema_version: u32,
    profiles: Vec<ProfileLayout>,
}

#[derive(Deserialize)]
struct ProfileLayout {
    id: i32,
    rank_offset: usize,
}

#[derive(Clone)]
pub struct Gateway {
    state: CharacterState,
    version: String,
    startup_snapshot: StartupSnapshot,
    self_data_template: SelfData,
    profile_rank_offsets: BTreeMap<i32, usize>,
    logged_in: bool,
    ready: bool,
    loaded: bool,
    /// The welcome splash went out this login; reloads skip it (see `startup_for`).
    welcomed: bool,
    gameplay: Option<Gameplay>,
    identity: Option<CharacterIdentity>,
    /// Developer-only chat commands; off unless the launcher opts in.
    dev_tools: bool,
    role_commands: bool,
    terrain: Option<std::sync::Arc<crate::terrain::Terrain>>,
    style_tables: Option<std::sync::Arc<crate::style_cards::ClientTables>>,
    world_dir: Option<std::path::PathBuf>,
    stock_text: Option<std::sync::Arc<std::collections::HashSet<u32>>>,
    role: crate::account_store::AccountRole,
    /// L048: the AddPc referee flag, fixed at login (`bind_account_role`); display only. L052:
    /// the same decision grants the session's staff jobs (`Gameplay::set_staff_jobs`).
    referee: bool,
    // (R) Bound only by the host after verifying the character's owner and ticket.
    role_account: Option<(std::path::PathBuf, u64)>,
    /// L042: this account's mute and ban, read with the role by `refresh_role`.
    sanctions: crate::moderation::Sanctions,
    movement_state: MovementState,
    expected_discontinuity: Option<ExpectedDiscontinuity>,
    last_movement_ms: Option<u64>,
    /// Staged opcode-164 relay body; the host takes it once after commit.
    jump_relay: Option<Vec<u8>>,
    trace_enabled: bool,
    // (R) Server-only authorization seam; no client parser sets an instance id.
    pending_transfer: Option<crate::players::ZoneInstance>,
}

#[derive(Debug, Default)]
pub struct GatewayOutcome {
    pub dev_command_denials: usize,
    pub shared_field: Option<crate::encounter::SharedFieldAction>,
    pub replies: Vec<Vec<u8>>,
    /// Inner packets (also in `replies`, wrapped) that players near this one must receive too:
    /// W162's ride actor, its rider's attachment and their removal, and T027's live 35/6 equip
    /// change / 35/7 job switch.
    pub nearby: Vec<Vec<u8>>,
    /// T199: `realm broadcast` text gameplay staged for one post-commit fanout to every
    /// online chat session. Session-only: dropped when the transaction is refused or fails.
    pub broadcast: Option<String>,
    /// L043: an admin `realm move` the host runs once after a successful commit, like `broadcast`.
    pub admin_move: Option<crate::gameplay::AdminMove>,
    /// T458: an admin `realm event start|stop`, run by the host after a commit, like `admin_move`.
    pub world_event: Option<crate::world_events::WorldEventCommand>,
    /// T461: a boombox press the host places once after the commit, like `admin_move`.
    pub boombox: Option<i32>,
    /// L042: role-gated moderation commands; the host runs them after the commit.
    pub moderation: Vec<crate::moderation::Command>,
    /// Inner application payloads that passed the gateway parser, in request order.
    pub applications: Vec<Vec<u8>>,
    pub state_changed: bool,
    pub entered_world: bool,
    pub gameplay_changed: bool,
    /// (R) Also true for same-scene transfers, which must reset interest membership.
    pub began_zoning: bool,
    /// W158: a big heart was picked up; the server heals the rest of the group.
    pub group_heal: Option<crate::encounter::GroupHeal>,
    /// Populated only when the owning local server explicitly enables tracing.
    pub trace: Vec<GatewayTraceEvent>,
}

#[derive(Clone, Debug, Default)]
pub struct GatewayTraceEvent {
    pub event: &'static str,
    pub tunnel_outer_opcode: Option<u16>,
    pub family: Option<u16>,
    pub subtype: Option<u16>,
    pub body_len: usize,
    pub body_head_hex: Option<String>,
    pub combined: bool,
    pub handler: &'static str,
    pub outcome: &'static str,
    pub reason: Option<String>,
    pub movement: Option<&'static str>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }
    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(size)
            .ok_or_else(|| invalid("field length overflow"))?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| invalid("truncated application field"))?;
        self.offset = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> io::Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid("invalid application boolean")),
        }
    }
    fn u16(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn vector(&mut self) -> io::Result<[f32; 3]> {
        let mut vector = [0.0; 3];
        for component in &mut vector {
            *component = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
            if !component.is_finite() || component.abs() > 1_000_000.0 {
                return Err(invalid("invalid or excessive coordinate"));
            }
        }
        Ok(vector)
    }
    fn string(&mut self, maximum: usize) -> io::Result<&'a str> {
        let size = self.u32()? as usize;
        if size > maximum {
            return Err(invalid("application string exceeds its limit"));
        }
        std::str::from_utf8(self.take(size)?)
            .map_err(|_| invalid("invalid UTF-8 application string"))
    }
    fn end(&self) -> io::Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid("trailing application fields"))
        }
    }
}

fn read_bounded(path: &Path, maximum: usize) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.len() > maximum as u64 {
        return Err(invalid("reference input is not a bounded regular file"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(invalid("reference file grew beyond its limit"));
    }
    Ok(bytes)
}

fn tunnel(message: &[u8]) -> io::Result<(u16, &[u8])> {
    let mut reader = Reader::new(message);
    let outer = reader.u16()?;
    if !matches!(outer, 5 | 6) {
        return Err(invalid("expected gateway tunnel"));
    }
    reader.bool()?;
    let size = reader.u32()? as usize;
    if !(2..=MAX_MESSAGE).contains(&size) {
        return Err(invalid("invalid tunnel payload length"));
    }
    let payload = reader.take(size)?;
    reader.end()?;
    Ok((outer, payload))
}

fn derive_self_layout(message: &[u8]) -> io::Result<SelfLayout> {
    let (outer, payload) = tunnel(message)?;
    let mut reader = Reader::new(payload);
    if outer != 5 || reader.u16()? != 12 {
        return Err(invalid("missing self template"));
    }
    if reader.u32()? as usize != payload.len() - 6 {
        return Err(invalid("self payload length mismatch"));
    }
    let pc = reader.take(payload.len() - 6)?;
    let mut prefix = Reader::new(pc);
    prefix.take(20)?; // unknown i64, GUID i64, model i32
    prefix.string(4096)?;
    prefix.string(4096)?;
    prefix.take(8)?; // hair and eye colors
    for _ in 0..3 {
        prefix.string(4096)?;
    }
    prefix.take(20)?; // five unknown i32 fields
    let position_offset = 13 + prefix.offset;
    prefix.take(32)?; // full four-component position and rotation
    Ok(SelfLayout {
        guid_offset: 21,
        position_offset,
        rotation_offset: position_offset + 16,
    })
}

fn derive_zone_layout(message: &[u8]) -> io::Result<ZoneLayout> {
    let (outer, payload) = tunnel(message)?;
    let mut reader = Reader::new(payload);
    if outer != 5 || reader.u16()? != 43 {
        return Err(invalid("missing zone template"));
    }
    reader.string(4096)?;
    reader.u32()?; // zone type, distinct from zone ID
    reader.bool()?;
    reader.bool()?;
    reader.string(4096)?; // sky
    reader.bool()?;
    let zone_id_offset = 7 + reader.offset;
    reader.take(8)?; // ID and geometry ID
    reader.bool()?;
    reader.bool()?;
    reader.end()?;
    Ok(ZoneLayout { zone_id_offset })
}

/// Validates all 46 captured templates, then returns template 7's bytes.
fn load_reference(directory: &Path) -> io::Result<(Manifest, Vec<u8>)> {
    let bytes = read_bounded(&directory.join("manifest.json"), 65_536)?;
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid gateway manifest JSON"))?;
    manifest.initial_state.validate()?;
    if manifest.schema_version != 1
        || manifest.client_version.is_empty()
        || manifest.client_version.len() > 64
        || manifest.templates.len() != 46
    {
        return Err(invalid("unsupported gateway reference manifest"));
    }
    let mut templates = Vec::new();
    let mut total = 0usize;
    for (index, file) in manifest.templates.iter().enumerate() {
        let path = Path::new(&file.file);
        if path.components().count() != 1
            || !matches!(path.components().next(), Some(Component::Normal(_)))
        {
            return Err(invalid("template path must be a direct relative filename"));
        }
        if file.kind != if index < 8 { Kind::Login } else { Kind::Ready } {
            return Err(invalid("template phase/order mismatch"));
        }
        let bytes = read_bounded(&directory.join(path), MAX_MESSAGE)?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("reference size overflow"))?;
        if total > MAX_REFERENCE
            || bytes.len() != file.length
            || format!("{:x}", Sha256::digest(&bytes)) != file.sha256
        {
            return Err(invalid("gateway template length/hash mismatch"));
        }
        let (outer, inner) = tunnel(&bytes)?;
        if outer != 5 || u16::from_le_bytes(inner[..2].try_into().unwrap()) != file.opcode {
            return Err(invalid("template opcode mismatch"));
        }
        templates.push(bytes);
    }
    let expected = [41, 2, 166, 43, 144, 35, 193, 12];
    if manifest.templates[..8]
        .iter()
        .map(|t| t.opcode)
        .ne(expected)
        || manifest.templates[8].opcode != 67
        || !manifest.templates[8..].iter().any(|t| t.opcode == 14)
    {
        return Err(invalid(
            "reference is not the verified initial world sequence",
        ));
    }
    if derive_self_layout(&templates[7])? != manifest.self_layout
        || derive_zone_layout(&templates[3])? != manifest.zone_layout
    {
        return Err(invalid("reference field offsets differ from proven layout"));
    }
    let sl = manifest.self_layout;
    let self_bytes = &templates[7];
    let mut guid = Reader::new(&self_bytes[sl.guid_offset..]);
    let mut position = Reader::new(&self_bytes[sl.position_offset..]);
    let mut rotation = Reader::new(&self_bytes[sl.rotation_offset..]);
    let mut zone = Reader::new(&templates[3][manifest.zone_layout.zone_id_offset..]);
    if guid.u64()? != manifest.initial_state.guid
        || position.vector()? != manifest.initial_state.position
        || rotation.vector()? != manifest.initial_state.rotation
        || zone.u32()? != manifest.initial_state.zone_id
    {
        return Err(invalid(
            "reference snapshot differs from serialized character",
        ));
    }
    Ok((manifest, templates.swap_remove(7)))
}

enum Event {
    Login,
    Ready,
    Loaded,
    Position(MovementUpdate, Option<crate::jump_relay::JumpImpulse>),
    Camera,
    Time,
    Application(Vec<u8>),
}
// (R) Account writes cannot participate in a staged character transaction.
fn reject_bundled_role_changes(events: &[Event], guid: u64) -> io::Result<()> {
    if events.iter().any(|event| match event {
        Event::Application(inner) => crate::chat::request(inner, guid)
            .ok()
            .flatten()
            .is_some_and(|text| {
                let lower = text.trim().to_ascii_lowercase();
                lower
                    .strip_prefix("realm ")
                    .is_some_and(|c| c.split_whitespace().next() == Some("role"))
            }),
        _ => false,
    }) {
        return Err(invalid("role changes require an unbundled request"));
    }
    Ok(())
}

fn trace_message(message: &[u8], event: &Event, combined: bool) -> GatewayTraceEvent {
    let top = message
        .get(..2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
    let (outer, body) = if matches!(top, Some(5 | 6)) {
        tunnel(message).map_or((None, message), |(outer, body)| (Some(outer), body))
    } else {
        (None, message)
    };
    let family = body
        .get(..2)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
    let subtype = body
        .get(2..4)
        .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));
    let sensitive = top == Some(1)
        || family
            .zip(subtype)
            .is_some_and(|(family, subtype)| crate::trace::sensitive_payload(family, subtype));
    let handler = match event {
        Event::Login => "gateway.login",
        Event::Ready => "gateway.ready",
        Event::Loaded => "gateway.loaded",
        Event::Position(..) => "movement",
        Event::Camera => "gateway.camera",
        Event::Time => "gateway.time_sync",
        Event::Application(_) => application_handler(body),
    };
    GatewayTraceEvent {
        event: if matches!(event, Event::Position(..)) {
            "movement"
        } else {
            "packet"
        },
        tunnel_outer_opcode: outer,
        family,
        subtype,
        body_len: body.len(),
        body_head_hex: crate::trace::hex_prefix(body, sensitive),
        combined,
        handler,
        outcome: "handled",
        reason: None,
        movement: None,
    }
}

/// The handler that owns an application message body, by family/subtype.
/// Routing uses it as well as tracing, so behaviour never depends on --trace.
fn application_handler(body: &[u8]) -> &'static str {
    // StopInspect is u16 149 + u8 2 (three bytes), so it cannot use the u16 sub below.
    if body == [149, 0, 2] {
        return "player_radial";
    }
    // W174 (R): a bare family 16 (two bytes) arrives once, right before the client disconnects
    // (31 of 31 native runs that ended in a disconnect, none without). Meaning: exit notice.
    if body == [16, 0] {
        return "client_exit_notice";
    }
    let word = |at: usize| {
        body.get(at..at + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    // W174: warpstone requests (56, 58, 90, 122) are handled in gameplay.rs. They sit outside
    // the match on purpose: 56 and 122 are valid with an empty body, so a wildcard seed would
    // make the hostile corpus replay a real teleport as a "cut".
    if matches!(word(0), Some(56 | 58 | 90 | 122)) {
        return "warp";
    }
    match (word(0), word(2)) {
        // T006: family 129 (guilds) is answered by the binary's guild router; naming it here
        // keeps an understood refusal out of the "unknown application message" log.
        (Some(129), _) => "guilds",
        // T519: the Fotomat portrait request and the quiet portrait upload CRC.
        (Some(115), _) => "fotomat",
        (Some(156), Some(2)) => "fotomat",
        // T007: the matchmaking list/stats/join, the empty in-progress list, and the
        // lobby close. Family 101 has a one-byte subtype, so 101/9 is matched on body[2].
        (Some(101), _) if body.get(2) == Some(&9) => "matchmaking",
        (Some(141), Some(1 | 3 | 13)) | (Some(41), Some(129)) => "matchmaking",
        (Some(15), Some(1)) => "social_chat",
        (Some(67), Some(2 | 3)) => "quick_chat",
        (Some(26), Some(4 | 6 | 8 | 10 | 23)) => "npc_interaction",
        (Some(26), Some(19)) => "player_radial",
        (Some(26), Some(14 | 15 | 17 | 21 | 30)) => "social_router",
        // W173: chat channel switches (32 on, 33 off) and bubble colour (18).
        (Some(26), Some(18 | 32 | 33)) => "chat_settings",
        (Some(40), Some(1 | 3 | 4 | 6 | 14 | 16)) => "social_router",
        (Some(49), Some(2 | 14 | 15)) => "quest_engine",
        // Family 47 carries a one-byte subtype; word(2) would swallow the first body byte.
        (Some(47), _) if matches!(body.get(2), Some(12 | 13)) => "quest_engine",
        // Family 45 also has a one-byte subtype: 45/7 is the client's goal-complete claim (W175).
        (Some(45), _) if body.get(2) == Some(&7) => "quest_engine",
        // Family 98 also has a one-byte subtype: 98/1 is the client's path request (W164).
        (Some(98), _) if body.get(2) == Some(&1) => "pathing",
        // Family 59 also has a one-byte subtype: the six client trade requests (W163).
        (Some(59), _) if matches!(body.get(2), Some(3 | 6..=10)) => "trade",
        (Some(26), Some(42)) | (Some(33), _) | (Some(41), Some(108)) | (Some(102), Some(1)) => {
            "kart"
        }
        // Family 34 is the demolition derby processor. Its client records are routed through the
        // active derby service; unknown subtypes stay inert there.
        (Some(34), _) => "derby",
        // Family 76 is the native Soccer family. Gameplay consumes it only while
        // a soccer session is active; naming it keeps understood refusals out of
        // the unknown-application log.
        (Some(76), _) => "soccer",
        // Family 168 carries a one-byte sub-opcode, so only the family is matched.
        (Some(168), _) => "mounts",
        // Family 53 likewise has a one-byte subtype. Only the implemented pet requests
        // carry the pet label; unknown subtypes remain visible as unknown traffic.
        (Some(53), _)
            if body
                .get(2)
                .is_some_and(|s| crate::pets::DECODED_SUBTYPES.contains(s)) =>
        {
            "pets"
        }
        // T765/T782: the card manager's ticket request (39/37) and the cancel's ack (39/7).
        (Some(39), _) if matches!(body.get(2), Some(7 | 37)) => "card_game",
        // W174: handled in gameplay.rs, where a refusal can send nothing. Naming them keeps
        // the gateway from logging an understood request as "unknown application message".
        (Some(26), Some(13)) => "class_selection",
        (Some(26), Some(24)) => "recipe_book",
        (Some(41), Some(109 | 122)) => "encounter",
        // T440: the answer to a group encounter invite; the host routes it after the commit.
        (Some(41), Some(103)) => "encounter_invite",
        (Some(42), Some(6 | 9)) => "item_bar",
        (Some(42), Some(15 | 16)) => "item_use",
        (Some(35), Some(33)) => "consumable_effects",
        // Family 167 is a service byte (1 = activity) and a one-byte sub (2 Play, 4 Refresh).
        (Some(167), _) if body.get(2) == Some(&1) && matches!(body.get(3), Some(2 | 4)) => {
            "activity_portal"
        }
        // W174 quiet list (R): accepted and discarded, never stored (109 text can hold local
        // paths), never relayed. 105/109/194 are Legacy PacketClientMetrics/PacketClientLog and
        // Ui.SetWallOfData; 193/1 is the client's announcements request (native 00a3e050).
        (Some(26), Some(20)) => "free_interaction",
        (Some(26), Some(26 | 37)) => "profile_notice",
        (Some(105), _) => "client_metrics",
        (Some(109), _) => "client_log",
        (Some(193), _) if body.get(2) == Some(&1) => "announcements_request",
        (Some(194), _) => "ui_telemetry",
        (Some(36), Some(10 | 12)) => "combat",
        (Some(42), Some(2 | 3 | 8)) | (Some(35), Some(34)) => "inventory",
        // W178: title select/refresh and the claim window are answered by Gameplay.
        (Some(152), Some(4 | 5)) => "titles",
        (Some(108), Some(1 | 3 | 5)) => "claims",
        // W172: Coin Store siblings and the Marketplace shell are understood merchant
        // traffic even when a particular row is intentionally answered with nothing.
        (Some(66), Some(1 | 3 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 25 | 27 | 38 | 43))
        | (Some(165), Some(2 | 4 | 5 | 8 | 12 | 14)) => "merchant",
        _ => "gameplay.handle_in_world",
    }
}

fn set_trace(
    trace: &mut Option<Vec<GatewayTraceEvent>>,
    index: usize,
    outcome: &'static str,
    handler: Option<&'static str>,
    reason: Option<String>,
    movement: Option<&'static str>,
) {
    let Some(record) = trace.as_mut().and_then(|records| records.get_mut(index)) else {
        return;
    };
    record.outcome = outcome;
    if let Some(handler) = handler {
        record.handler = handler;
    }
    record.reason = reason;
    record.movement = movement;
}

fn parse_message(message: &[u8], state: &CharacterState, version: &str) -> io::Result<Event> {
    let mut reader = Reader::new(message);
    match reader.u16()? {
        1 => {
            if reader.string(256)?.is_empty() {
                return Err(invalid("empty local login ticket"));
            }
            if reader.u64()? != state.guid {
                return Err(invalid("login selects another character"));
            }
            if reader.string(64)? != version {
                return Err(invalid("unsupported client version"));
            }
            reader.string(4096)?;
            reader.end()?;
            Ok(Event::Login)
        }
        5 | 6 => {
            let (outer, inner) = tunnel(message)?;
            let mut reader = Reader::new(inner);
            let opcode = reader.u16()?;
            if outer == 6 {
                return Ok(Event::Application(inner.to_vec()));
            }
            let event = match opcode {
                13 => Event::Ready,
                10 => Event::Loaded,
                125 => {
                    if reader.u64()? != state.guid {
                        return Err(invalid("position selects another character"));
                    }
                    let position = reader.vector()?;
                    let direction = reader.vector()?;
                    let update = MovementUpdate {
                        position,
                        direction,
                        state: reader.u8()?,
                        unknown: reader.u8()?,
                    };
                    Event::Position(update, None)
                }
                164 => {
                    // Native jump reader 008e5ba0 / writer 009127f0: 40-byte body.
                    let jump = crate::jump_relay::decode(inner, state.guid)?;
                    reader.take(38)?;
                    Event::Position(jump.movement(), Some(jump.impulse()))
                }
                126 => {
                    reader.vector()?;
                    reader.vector()?;
                    Event::Camera
                }
                52 => {
                    reader.take(12)?;
                    reader.bool()?;
                    Event::Time
                }
                _ => return Ok(Event::Application(inner.to_vec())),
            };
            reader.end()?;
            Ok(event)
        }
        _ => Err(invalid("unsupported top-level gateway message")),
    }
}

fn application_messages(bytes: &[u8]) -> io::Result<Vec<&[u8]>> {
    if bytes.len() < 2 || bytes.len() > MAX_MESSAGE {
        return Err(invalid("invalid application length"));
    }
    if bytes[..2] != [0, 25] {
        return Ok(vec![bytes]);
    }
    let mut reader = Reader::new(&bytes[2..]);
    let mut messages = Vec::new();
    while reader.offset < reader.bytes.len() {
        if messages.len() >= MAX_BUNDLE_CHILDREN {
            return Err(invalid("too many bundled application messages"));
        }
        let (size, prefix) = crate::soe::variable_length(&reader.bytes[reader.offset..])
            .ok_or_else(|| invalid("truncated bundled message length"))?;
        reader.take(prefix)?;
        if !(2..=MAX_MESSAGE).contains(&size) {
            return Err(invalid("invalid bundled message length"));
        }
        let child = reader.take(size)?;
        if child[..2] == [0, 25] {
            return Err(invalid("nested message bundles are unsupported"));
        }
        messages.push(child);
    }
    if messages.is_empty() {
        return Err(invalid("empty application bundle"));
    }
    Ok(messages)
}

fn wrapped(inner: &[u8]) -> Vec<u8> {
    let mut message = vec![5, 0, 1];
    message.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    message.extend_from_slice(inner);
    message
}

impl Gateway {
    /// (R) The server splits a fully parsed bundle into ordered commits before
    /// resolving shared actions; later movement can never move an earlier hit.
    pub fn is_combat_request(&self, message: &[u8]) -> io::Result<bool> {
        Ok(
            matches!(parse_message(message,&self.state,&self.version)?, Event::Application(inner) if inner.starts_with(&[36,0,10,0]) || inner.starts_with(&[26,0,10,0])),
        )
    }
    pub fn ordered_messages(&self, message: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        let messages = application_messages(message)?;
        let events = messages
            .iter()
            .map(|message| parse_message(message, &self.state, &self.version))
            .collect::<io::Result<Vec<_>>>()?;
        if message.starts_with(&[0, 25]) {
            reject_bundled_role_changes(&events, self.state.guid)?;
        }
        Ok(messages.into_iter().map(<[u8]>::to_vec).collect())
    }

    /// Application payloads only, in request order. Used by the server's
    /// account action budget before it clones gameplay.
    pub fn application_requests(&self, message: &[u8]) -> io::Result<Vec<Vec<u8>>> {
        application_messages(message)?
            .into_iter()
            .map(|message| parse_message(message, &self.state, &self.version))
            .filter_map(|event| match event {
                Ok(Event::Application(inner)) => Some(Ok(inner)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }
    pub fn take_shared_fields(&mut self) -> Option<crate::encounter::SharedFieldState> {
        self.gameplay.as_mut()?.take_shared_fields()
    }
    pub fn bind_run_instance(&mut self, instance: i32, scene_reload: bool) -> io::Result<bool> {
        self.gameplay
            .as_mut()
            .map_or(Ok(false), |g| g.bind_run_instance(instance, scene_reload))
    }
    pub fn field_participant(&self) -> Option<crate::encounter::FieldParticipant> {
        self.gameplay.as_ref()?.field_participant(&self.state)
    }
    pub fn handle_shared_field(
        &mut self,
        action: &crate::encounter::SharedFieldAction,
        field: &mut crate::encounter::SharedFieldState,
        known: &std::collections::BTreeSet<u64>,
        now_ms: u64,
    ) -> io::Result<crate::gameplay::Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("shared combat requires gameplay"))?
            .handle_shared_field(action, field, known, &self.state, now_ms)
    }
    pub fn tick_shared_effects(
        &mut self,
        field: &mut crate::encounter::SharedFieldState,
        now_ms: u64,
    ) -> io::Result<crate::gameplay::Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("shared effects require gameplay"))?
            .tick_shared_effects(field, &self.state, now_ms)
    }
    pub fn tick_shared_motion(
        &mut self,
        field: &mut crate::encounter::SharedFieldState,
        players: &[crate::encounter::FieldParticipant],
        now_ms: u64,
        elapsed: u64,
    ) -> io::Result<(crate::encounter::EncounterOutcome, Vec<(u64, u32)>)> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("shared motion requires gameplay"))?
            .tick_shared_motion(field, players, now_ms, elapsed)
    }
    pub fn receive_field_damage(
        &mut self,
        damage: u32,
        now_ms: u64,
        engaged: bool,
    ) -> io::Result<crate::gameplay::Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("shared damage requires gameplay"))?
            .receive_field_damage(damage, now_ms, engaged)
    }
    /// W158: heal this player for another member's big heart.
    pub fn receive_group_heal(&mut self, percent: u32) -> io::Result<crate::gameplay::Outcome> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("group heal requires gameplay"))?
            .receive_group_heal(percent)
    }

    pub fn reference_state(directory: &Path) -> io::Result<CharacterState> {
        Ok(load_reference(directory)?.0.initial_state)
    }

    pub fn load(directory: &Path, state: CharacterState) -> io::Result<Self> {
        Self::load_internal(directory, state, None, None, BTreeMap::new())
    }

    /// Load with an explicit typed startup snapshot, primarily for synthetic tests.
    pub fn load_with_startup_snapshot(
        directory: &Path,
        state: CharacterState,
        snapshot: StartupSnapshot,
    ) -> io::Result<Self> {
        Self::load_internal(directory, state, None, Some(snapshot), BTreeMap::new())
    }

    pub fn load_gameplay(
        directory: &Path,
        state: CharacterState,
        content: &Path,
        saved: Option<GameplayState>,
    ) -> io::Result<Self> {
        Self::load_with_gameplay(directory, state, content, saved)
    }

    fn load_internal<'a>(
        source: impl Into<startup_source::Source<'a>>,
        mut state: CharacterState,
        mut gameplay: Option<Gameplay>,
        snapshot_override: Option<StartupSnapshot>,
        profile_rank_offsets: BTreeMap<i32, usize>,
    ) -> io::Result<Self> {
        state.validate()?;
        let source = source.into();
        let base = source.base()?;
        // (R) The local account-store path assigns a distinct GUID to each character;
        // the shared startup template is patched with that GUID at send time.
        let startup_snapshot = if let Some(snapshot) = snapshot_override {
            snapshot.validate()?;
            snapshot
        } else {
            let snapshot = StartupSnapshot::load(&source.snapshot_path()?)?;
            if snapshot.login.zone.zone_id != base.initial_state.zone_id {
                return Err(invalid("startup snapshot belongs to another initial zone"));
            }
            snapshot
        };
        if let Some(game) = &mut gameplay {
            game.exclude_startup_npcs(
                startup_snapshot
                    .world
                    .npc_spawns
                    .iter()
                    .map(|npc| npc.actor_id),
            );
        }
        if let Some(game) = &mut gameplay {
            if let Some(return_location) = game.abandon_battle_for_reconnect() {
                if state.guid != return_location.guid {
                    return Err(invalid("battle reconnect character ownership differs"));
                }
                state = return_location;
                state.validate()?;
            }
            if !game.supports_location(&state) {
                return Err(invalid("saved zone differs from gameplay encounter"));
            }
            if let Some(return_location) = game.abandon_kart_for_reconnect() {
                // Check the saved ownership/location before replacing it.
                if state.guid != return_location.guid {
                    return Err(invalid("kart reconnect character ownership differs"));
                }
                state = return_location;
                state.validate()?;
            }
        }
        if gameplay
            .as_ref()
            .is_some_and(|g| !g.supports_location(&state))
            || (gameplay.is_none() && state.zone_id != base.initial_state.zone_id)
        {
            return Err(invalid(
                "unsupported saved zone: reference supports only its initial zone",
            ));
        }
        let movement_state = MovementState::new(state.zone_id, state.position, state.rotation);
        Ok(Self {
            state,
            version: base.version,
            startup_snapshot,
            self_data_template: base.self_data,
            profile_rank_offsets,
            logged_in: false,
            ready: false,
            loaded: false,
            welcomed: false,
            gameplay,
            identity: None,
            dev_tools: false,
            role_commands: false,
            terrain: None,
            style_tables: None,
            world_dir: None,
            stock_text: None,
            role: crate::account_store::AccountRole::Player,
            referee: false,
            role_account: None,
            sanctions: Default::default(),
            movement_state,
            expected_discontinuity: None,
            last_movement_ms: None,
            jump_relay: None,
            trace_enabled: false,
            pending_transfer: None,
        })
    }

    /// The live gameplay engine, for developer tooling (Studio).
    pub fn gameplay(&self) -> Option<&Gameplay> {
        self.gameplay.as_ref()
    }

    /// The live gameplay engine, for host services beside the UDP session (T770 card service).
    pub fn gameplay_mut(&mut self) -> Option<&mut Gameplay> {
        self.gameplay.as_mut()
    }

    /// Restore the host-owned rolling-hour fishing ledger for this character. A fresh
    /// `Gameplay` starts empty, so the gateway calls this from its character-scoped state after
    /// loading a save; the timestamps never enter the save envelope.
    pub fn restore_fishing_hours(&mut self, hours: Vec<u64>) -> io::Result<()> {
        let Some(gameplay) = self.gameplay.as_mut() else {
            return if hours.is_empty() {
                Ok(())
            } else {
                Err(invalid("fishing cap history requires gameplay"))
            };
        };
        gameplay.restore_fishing_hours(self.state.guid, hours)
    }

    pub fn fishing_hour_history(&self) -> Option<Vec<u64>> {
        self.gameplay
            .as_ref()
            .map(|gameplay| gameplay.fishing_hour_history(self.state.guid))
    }

    pub fn state(&self) -> &CharacterState {
        &self.state
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in
    }
    pub fn is_world_loaded(&self) -> bool {
        self.loaded
    }

    pub fn take_pending_transfer(&mut self) -> Option<crate::players::ZoneInstance> {
        self.pending_transfer.take()
    }

    /// (R) Host-authorized transfer only. Do not expose this through raw packets
    /// or developer chat. Ordinary encounter consent is `stage_encounter_join` (T440).
    pub fn stage_instance_transfer(
        &mut self,
        target: crate::players::ZoneInstance,
        scene: ZoneTransition,
        now_ms: u64,
    ) -> io::Result<GatewayOutcome> {
        target
            .world_instance_id()
            .map_err(|_| invalid("instance id outside world range"))?;
        if target.zone_id != scene.zone_id || !self.loaded {
            return Err(invalid("instance transfer scene/readiness mismatch"));
        }
        i32::try_from(scene.zone_id).map_err(|_| invalid("scene id outside wire range"))?;
        let state = CharacterState {
            guid: self.state.guid,
            zone_id: scene.zone_id,
            position: scene.position,
            rotation: scene.rotation,
        };
        state.validate()?;
        let packet = wrapped(&zone_begin_packet(&scene));
        // Reuse opcode-31 layout, native reader 008e4ae0 (zone_transition.rs).
        crate::zone_transition::decode(&packet)?;
        self.state = state;
        self.movement_state = MovementState::new(scene.zone_id, scene.position, scene.rotation);
        self.expected_discontinuity = Some(ExpectedDiscontinuity::zone_change(
            scene.zone_id,
            scene.position,
        ));
        self.last_movement_ms = Some(now_ms);
        self.ready = false;
        self.loaded = false;
        self.pending_transfer = Some(target);
        Ok(GatewayOutcome {
            replies: vec![packet],
            state_changed: true,
            began_zoning: true,
            ..Default::default()
        })
    }

    /// (R) Runtime membership adapter: regional content labels are not scene ids.
    /// Scene geometry/tutorial values reuse the existing startup/transition route.
    pub fn runtime_zone(&self) -> io::Result<crate::content::Zone> {
        if let Some(scene) = self.gameplay.as_ref().and_then(Gameplay::current_zone) {
            return Ok(crate::content::Zone {
                id: scene.zone_id,
                name: scene.name,
                geometry_id: Some(
                    u32::try_from(scene.geometry_id)
                        .map_err(|_| invalid("negative scene geometry"))?,
                ),
                zone_type: Some(2),
                tutorial: Some(scene.tutorial),
                sky: Some(scene.sky),
                source: Vec::new(),
            });
        }
        let scene = &self.startup_snapshot.login.zone;
        Ok(crate::content::Zone {
            id: self.state.zone_id,
            name: scene.name.clone(),
            geometry_id: Some(scene.geometry_id),
            zone_type: Some(scene.zone_type),
            tutorial: Some(scene.tutorial),
            sky: Some(scene.sky.clone()),
            source: Vec::new(),
        })
    }

    /// The public look other players get in AddPc (Build Plan 1.3), from the
    /// same identity, profile and inventory inputs as `build_self_message`.
    /// (R) Name ids stay zero (literal-name mode); speed keeps the players.rs default.
    /// See `Gameplay::public_look` (W143 Inspect).
    pub fn public_look(&self) -> Option<crate::player_radial::PublicLook> {
        self.gameplay.as_ref()?.public_look()
    }

    pub fn player_presentation(&self, member: bool) -> io::Result<PlayerPresentation> {
        let mut look = PlayerPresentation {
            character_guid: self.state.guid,
            member,
            referee: self.referee,
            ..PlayerPresentation::default()
        };
        if let Some(identity) = &self.identity {
            look.first_name.clone_from(&identity.first_name);
            look.last_name.clone_from(&identity.last_name);
        }
        if let Some(appearance) = self.current_look(self.gameplay.as_ref()) {
            look.customization_ids = appearance.portrait_ids();
            (look.model, look.hair_color, look.eye_color) = (
                appearance.model,
                appearance.hair_color,
                appearance.eye_color,
            );
            (look.head, look.hair, look.skin_tone) =
                (appearance.head, appearance.hair, appearance.skin_tone);
            (look.face_paint, look.model_customization) =
                (appearance.face_paint, appearance.model_customization);
        }
        // Without loaded profiles, self-data keeps the template's job; match it.
        let profile = self
            .gameplay
            .as_ref()
            .and_then(Gameplay::active_profile)
            .unwrap_or(self.self_data_template.jobs.active_profile_id as i32);
        look.active_profile = profile;
        if let Some(game) = &self.gameplay {
            look.attachments_wire = game.profile_attachments(profile)?;
            look.wield_type = game.public_wield_type(profile);
            look.set_visual(game.consumable_effects().visual());
        }
        Ok(look)
    }

    /// The last accepted movement state and unknown byte, relayed to others.
    pub fn movement_bytes(&self) -> (u8, u8) {
        (self.movement_state.state, self.movement_state.unknown)
    }

    pub fn take_jump_relay(&mut self) -> Option<Vec<u8>> {
        self.jump_relay.take()
    }

    pub fn enable_gameplay(
        &mut self,
        directory: &Path,
        state: Option<GameplayState>,
    ) -> io::Result<()> {
        let mut gameplay = Gameplay::load(directory, state)?;
        if let Some(return_location) = gameplay.abandon_battle_for_reconnect() {
            if self.state.guid != return_location.guid {
                return Err(invalid("battle reconnect character ownership differs"));
            }
            return_location.validate()?;
            self.state = return_location;
        }
        if !gameplay.supports_location(&self.state) {
            return Err(invalid("saved zone differs from gameplay encounter"));
        }
        gameplay.set_dev_tools(self.dev_tools);
        gameplay.set_role_commands(self.role_commands);
        gameplay.set_terrain(self.terrain.clone());
        gameplay.set_style_tables(self.style_tables.clone());
        gameplay.set_world_dir(self.world_dir.clone())?;
        gameplay.set_stock_text(self.stock_text.clone());
        gameplay.set_role(self.role);
        self.gameplay = Some(gameplay);
        Ok(())
    }

    /// The session's authority for developer commands (Build Plan 1.5).
    pub fn set_role(&mut self, role: crate::account_store::AccountRole) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_role(role);
        }
        self.role = role;
    }

    /// Host-only binding: `account_id` is the verified CharacterRecord owner,
    /// never an identifier supplied by a gameplay packet.
    pub fn bind_account_role(&mut self, path: &Path, account_id: u64) {
        self.role_account = Some((path.to_owned(), account_id));
        self.refresh_role();
        // (R) OSFR policy: moderators and admins show as referees; a change shows at next login.
        use crate::account_store::AccountRole::{Admin, Moderator};
        self.referee = matches!(self.role, Moderator | Admin);
        // L052 (R): the staff jobs (Referee, Enforcer) follow the same login decision.
        if let Some(game) = self.gameplay.as_mut() {
            game.set_staff_jobs(self.referee);
        }
    }

    /// (R) Re-read on use so online demotion takes effect in every session.
    /// An unreadable store, missing account or missing role fails closed.
    pub fn refresh_role(&mut self) {
        if let Some((path, id)) = &self.role_account {
            let store = crate::account_store::AccountStore::load(path).ok();
            // L042: a mute applies from the next message; an unreadable store mutes (R).
            let sanctions = store.as_ref().map(|store| store.sanctions(*id));
            self.sanctions = sanctions.unwrap_or(crate::moderation::Sanctions::FAIL_CLOSED);
            let role = store
                .and_then(|store| store.account_role(*id))
                .unwrap_or(crate::account_store::AccountRole::Player);
            self.set_role(role);
        }
    }

    pub fn role(&self) -> crate::account_store::AccountRole {
        self.role
    }

    /// Authorization audit before staging: a later rejected bundle child must
    /// not erase a developer refusal from the shared SecurityLog (R).
    pub fn denied_dev_commands(&mut self, message: &[u8]) -> io::Result<usize> {
        self.refresh_role();
        let Some(game) = self.gameplay.as_ref().filter(|_| self.loaded) else {
            return Ok(0);
        };
        let mut count = 0;
        for child in application_messages(message)? {
            if let Event::Application(inner) = parse_message(child, &self.state, &self.version)? {
                count += usize::from(
                    game.developer_command_denied(&inner, self.state.guid)
                        .unwrap_or(false),
                );
            }
        }
        Ok(count)
    }

    fn change_account_role(&mut self, command: &str) -> &'static str {
        use crate::account_store::{AccountRole, AccountStore, AccountStoreLock, STAFF_LOCK_WAIT};
        let words: Vec<_> = command.split_whitespace().collect();
        let ["role", id, role] = words.as_slice() else {
            return "Usage: realm role <account-id> <player|builder|moderator|admin>";
        };
        let (Some(target), Some(role)) = (
            id.parse::<u64>().ok().filter(|id| *id > 0),
            AccountRole::parse(role),
        ) else {
            return "Usage: realm role <account-id> <player|builder|moderator|admin>";
        };
        let Some((path, actor)) = &self.role_account else {
            return "Cannot do that: a verified account binding is required.";
        };
        // (R) Cooperate with login/CLI writers; never overwrite a newer key or role.
        let Ok(_lock) = AccountStoreLock::acquire_within(path, STAFF_LOCK_WAIT) else {
            return "Cannot do that: account store is busy or unavailable.";
        };
        let result = AccountStore::load(path).and_then(|mut store| {
            store.set_account_role(*actor, target, role)?;
            store.save(path)
        });
        self.refresh_role();
        if result.is_ok() {
            "Account role saved."
        } else {
            "Cannot do that: account role was not saved."
        }
    }

    /// Stock locale row keys for World Builder text ids.
    pub fn set_stock_text(
        &mut self,
        hashes: Option<std::sync::Arc<std::collections::HashSet<u32>>>,
    ) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_stock_text(hashes.clone());
        }
        self.stock_text = hashes;
    }

    /// The World Builder live layer folder (`--world`).
    pub fn set_world_dir(&mut self, dir: Option<std::path::PathBuf>) -> io::Result<()> {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_world_dir(dir.clone())?;
        }
        self.world_dir = dir;
        Ok(())
    }

    /// The client tables style cards resolve through; without them every use is refused.
    pub fn set_style_tables(
        &mut self,
        tables: Option<std::sync::Arc<crate::style_cards::ClientTables>>,
    ) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_style_tables(tables.clone());
        }
        self.style_tables = tables;
    }

    /// The look self-data and AddPc show: the style-card look, else the identity's.
    fn current_look(&self, gameplay: Option<&Gameplay>) -> Option<crate::style_cards::Look> {
        let saved = gameplay.and_then(Gameplay::look);
        saved.or_else(|| {
            self.identity
                .as_ref()
                .map(crate::style_cards::Look::from_identity)
        })
    }

    /// Ground heights for enemy movement and developer teleports.
    pub fn set_terrain(&mut self, terrain: Option<std::sync::Arc<crate::terrain::Terrain>>) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_terrain(terrain.clone());
        }
        self.terrain = terrain;
    }

    /// L039: before login only (no pose sent yet), an open-world pose off the map or deep under
    /// the terrain moves to the nearest warpstone. Returns the old position and why, to save and log.
    pub fn recover_login_pose(&mut self) -> Option<([f32; 3], &'static str)> {
        if self.logged_in {
            return None;
        }
        let (to, reason) = self.gameplay.as_ref()?.recover_login_pose(&self.state)?;
        self.movement_state = MovementState::new(to.zone_id, to.position, to.rotation);
        Some((std::mem::replace(&mut self.state, to).position, reason))
    }

    /// Enable the developer-only chat commands such as `realm goto`. Off by
    /// default so an ordinary local session cannot relocate by accident.
    pub fn set_dev_tools(&mut self, enabled: bool) {
        self.dev_tools = enabled;
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_dev_tools(enabled);
        }
    }

    /// See `Gameplay::set_role_commands` (Security Model decision 1).
    pub fn set_role_commands(&mut self, enabled: bool) {
        self.role_commands = enabled;
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_role_commands(enabled);
        }
    }

    /// Enables per-message diagnostic metadata without changing message handling.
    pub fn set_trace_enabled(&mut self, enabled: bool) {
        self.trace_enabled = enabled;
    }

    pub fn set_game_rules(&mut self, rules: crate::game_rules::GameRuleSet) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_game_rules(rules);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            // L012, T514: a save taken in a match, a house or a fishing hole stores the world pose.
            character: self
                .gameplay
                .as_ref()
                .and_then(|game| {
                    game.soccer_return(&self.state)
                        .or_else(|| game.scene_return())
                })
                .unwrap_or_else(|| self.state.clone()),
            gameplay: self.gameplay.as_ref().map(Gameplay::state),
            // S17: the roster rides in the same document write as the character and inventory.
            pets: self.gameplay.as_ref().and_then(Gameplay::pets_document),
        }
    }

    pub fn set_identity(&mut self, identity: CharacterIdentity) -> io::Result<bool> {
        if identity.guid != self.state.guid {
            return Err(invalid("identity GUID differs from character save"));
        }
        // (R) The creation identity type reserves GUID 1; self-data carries the
        // selected character GUID in its separate identity field.
        let mut canonical_identity = identity.clone();
        canonical_identity.guid = 1;
        canonical_identity.validate()?;
        let changed = match self.gameplay.as_mut() {
            Some(game) => game.apply_identity(&canonical_identity)?,
            None => false,
        };
        self.identity = Some(identity);
        Ok(changed)
    }

    pub fn gameplay_summary(&self) -> Option<serde_json::Value> {
        self.gameplay.as_ref().map(Gameplay::summary)
    }

    /// W178: trusted server/store hook for one queued claim; not a network route.
    pub fn issue_claim(
        &mut self,
        claim_id: u64,
        grant: crate::rewards::RewardGrant,
    ) -> io::Result<()> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("claims require gameplay"))?
            .issue_claim(claim_id, grant)
    }

    /// W178: trusted achievement-completion hook; clients cannot grant a title.
    pub fn award_achievement_titles(
        &mut self,
        newly_completed: &[String],
        bindings: &[crate::progression_extras::AchievementTitleBinding],
    ) -> io::Result<Vec<Vec<u8>>> {
        self.gameplay
            .as_mut()
            .ok_or_else(|| invalid("titles require gameplay"))?
            .award_achievement_titles(newly_completed, bindings)
    }

    /// Phrase IDs are sourced from the same typed startup data sent to this client.
    pub fn quick_chat_catalog(&self) -> crate::social_chat::QuickChatCatalog {
        crate::social_chat::QuickChatCatalog::from_rows(
            self.startup_snapshot
                .world
                .quick_chat
                .iter()
                .map(|row| (row.id, row.animation_id)),
        )
    }

    /// (R) Actor state for emote validation. `mounted` comes from the mount service in
    /// gameplay (W162); `moving` from the last accepted position update.
    pub fn emote_context(&self, now_ms: u64) -> crate::social_chat::piece_093::EmoteContext {
        let mounted = self.gameplay.as_ref().is_some_and(Gameplay::is_mounted);
        crate::social_chat::emote_context(mounted, self.last_movement_ms, now_ms)
    }

    /// Begin a new local session while retaining the latest character snapshot.
    pub fn reset_session(&mut self) {
        self.logged_in = false;
        self.ready = false;
        self.loaded = false;
        self.welcomed = false;
        if let Some(game) = self.gameplay.as_mut() {
            if let Some(return_location) = game.abandon_kart_for_reconnect() {
                self.state = return_location;
            }
            if let Some(return_location) = game.abandon_soccer_for_reconnect(&self.state) {
                self.state = return_location;
            }
            if let Some(return_location) = game.abandon_scene_for_reconnect() {
                self.state = return_location;
            }
            game.reset_session();
        }
        self.movement_state =
            MovementState::new(self.state.zone_id, self.state.position, self.state.rotation);
        self.expected_discontinuity = None;
        self.last_movement_ms = None;
        self.jump_relay = None;
    }

    pub fn handle(&mut self, message: &[u8]) -> io::Result<GatewayOutcome> {
        self.handle_at(message, 0)
    }

    pub fn handle_at(&mut self, message: &[u8], now_ms: u64) -> io::Result<GatewayOutcome> {
        self.refresh_role();
        // Decode every child first. Malformed later children cannot partially
        // update a saved character or release half an initialization stream.
        let messages = application_messages(message)?;
        let combined = message.starts_with(&[0, 25]);
        let events = messages
            .iter()
            .map(|m| parse_message(m, &self.state, &self.version))
            .collect::<io::Result<Vec<_>>>()?;
        if combined {
            reject_bundled_role_changes(&events, self.state.guid)?;
        }
        let mut trace = self.trace_enabled.then(|| {
            messages
                .iter()
                .zip(&events)
                .map(|(message, event)| trace_message(message, event, combined))
                .collect::<Vec<_>>()
        });
        let mut state = self.state.clone();
        let mut gameplay = self.gameplay.clone();
        let mut movement_state = self.movement_state;
        let mut expected_discontinuity = self.expected_discontinuity.clone();
        let mut last_movement_ms = self.last_movement_ms;
        let mut impulse = None;
        let (mut logged_in, mut ready, mut loaded) = (self.logged_in, self.ready, self.loaded);
        let mut welcomed = self.welcomed;
        let mut outcome = GatewayOutcome::default();
        // Count only the replies appended since the previous event; a bundle must not rescan
        // every earlier batch on each child (F179 item 19).
        let mut budget = BundleBudget::bundle(combined);
        let mut counted_replies = 0usize;
        for (event_index, event) in events.into_iter().enumerate() {
            budget.stage(&outcome.replies[counted_replies..])?;
            counted_replies = outcome.replies.len();
            match event {
                Event::Login => {
                    if !logged_in {
                        outcome.replies.extend(self.startup_for(
                            Kind::Login,
                            &state,
                            gameplay.as_ref(),
                            welcomed,
                        )?);
                        // T798: the card window's session ticket, with a card launch block only.
                        let card = gameplay.as_ref().map(Gameplay::card_session_data);
                        outcome.replies.extend(card.transpose()?.flatten());
                        logged_in = true;
                    }
                }
                Event::Ready => {
                    if !logged_in {
                        return Err(invalid("ready before login"));
                    }
                    if !ready {
                        if let Some(replies) = gameplay
                            .as_ref()
                            .map(Gameplay::kart_ready_stream)
                            .transpose()?
                            .flatten()
                        {
                            outcome.replies.extend(replies.iter().map(|r| wrapped(r)));
                        } else {
                            outcome.replies.extend(self.startup_for(
                                Kind::Ready,
                                &state,
                                gameplay.as_ref(),
                                welcomed,
                            )?);
                            welcomed = true;
                        }
                        ready = true;
                    }
                }
                Event::Loaded => {
                    if !ready {
                        return Err(invalid("loading complete before initialization"));
                    }
                    outcome.entered_world |= !loaded;
                    if !loaded {
                        if let Some(game) = gameplay.as_mut() {
                            let result = game.initialize_world(&state, now_ms)?;
                            outcome.state_changed |= result.changed;
                            outcome.gameplay_changed |= result.changed;
                            outcome
                                .replies
                                .extend(result.replies.iter().map(|reply| wrapped(reply)));
                            outcome.replies.extend(
                                game.update_npc_visibility(&state)?
                                    .iter()
                                    .map(|reply| wrapped(reply)),
                            );
                        }
                    }
                    loaded = true;
                }
                Event::Position(update, jump) => {
                    if !ready {
                        // Movement from the prior scene can already be queued
                        // when a begin-zoning reply is emitted.
                        if gameplay.as_ref().is_some_and(Gameplay::is_opening) {
                            set_trace(
                                &mut trace,
                                event_index,
                                "ignored",
                                None,
                                Some("prior_scene_movement_during_opening".into()),
                                Some("ignore"),
                            );
                            continue;
                        }
                        return Err(invalid("position before world initialization"));
                    }
                    let elapsed = last_movement_ms
                        .map(|last| Duration::from_millis(now_ms.saturating_sub(last)))
                        .unwrap_or(Duration::from_secs(1));
                    let validated = movement::validate(
                        &movement_state,
                        update,
                        state.zone_id,
                        elapsed,
                        Gameplay::speed_modifiers(gameplay.as_ref()),
                        expected_discontinuity,
                    );
                    expected_discontinuity = validated.expected_discontinuity;
                    match validated.decision {
                        MovementDecision::Ignore => {
                            set_trace(
                                &mut trace,
                                event_index,
                                "ignored",
                                None,
                                Some(format!("{:?}", validated.reason)),
                                Some("ignore"),
                            );
                            continue;
                        }
                        MovementDecision::RejectAndSnapBack => {
                            set_trace(
                                &mut trace,
                                event_index,
                                "rejected",
                                None,
                                Some(format!("{:?}", validated.reason)),
                                Some("reject"),
                            );
                            state.position = validated.state.position;
                            state.rotation = validated.state.direction;
                            movement_state = validated.state;
                            outcome.state_changed = true;
                            outcome.replies.push(wrapped(&dev_teleport::location_packet(
                                state.position,
                                state.rotation,
                            )));
                            continue;
                        }
                        MovementDecision::Accept | MovementDecision::Clamp => {
                            let clamped = validated.decision == MovementDecision::Clamp;
                            set_trace(
                                &mut trace,
                                event_index,
                                if clamped { "clamped" } else { "accepted" },
                                None,
                                Some(format!("{:?}", validated.reason)),
                                Some(if clamped { "clamp" } else { "accept" }),
                            );
                            outcome.state_changed |= state.position != validated.state.position
                                || state.rotation != validated.state.direction;
                            state.position = validated.state.position;
                            state.rotation = validated.state.direction;
                            movement_state = validated.state;
                            last_movement_ms = Some(now_ms);
                            impulse = jump.or(impulse);
                            if clamped {
                                outcome.replies.push(wrapped(&dev_teleport::location_packet(
                                    state.position,
                                    state.rotation,
                                )));
                            }
                        }
                    }
                    if let Some(game) = gameplay.as_mut() {
                        let result = game.on_location(&state, now_ms)?;
                        outcome.state_changed |= result.changed;
                        outcome.gameplay_changed |= result.changed;
                        outcome
                            .replies
                            .extend(result.replies.iter().map(|reply| wrapped(reply)));
                        outcome.nearby.extend(result.nearby);
                        outcome.replies.extend(
                            game.update_npc_visibility(&state)?
                                .iter()
                                .map(|reply| wrapped(reply)),
                        );
                    }
                }
                Event::Camera => {
                    if !ready {
                        if gameplay.as_ref().is_some_and(Gameplay::is_opening) {
                            set_trace(
                                &mut trace,
                                event_index,
                                "ignored",
                                None,
                                Some("prior_scene_camera_during_opening".into()),
                                None,
                            );
                            continue;
                        }
                        return Err(invalid("camera before world initialization"));
                    }
                }
                Event::Time => {
                    if !logged_in {
                        return Err(invalid("time sync before login"));
                    }
                    let seconds = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|_| invalid("system clock precedes Unix epoch"))?
                        .as_secs();
                    let seconds =
                        i64::try_from(seconds).map_err(|_| invalid("system time overflow"))?;
                    let clock = gameplay.as_ref().map(Gameplay::game_clock);
                    let inner = clock.unwrap_or_default().sync_reply(seconds);
                    outcome.replies.push(wrapped(&inner));
                }
                Event::Application(inner) => {
                    impulse = None;
                    if !loaded && crate::fotomat::handles(&inner) {
                        if let Some(reply) = self.fotomat_loading_reply(&inner, state.guid) {
                            outcome.replies.push(wrapped(&reply));
                        }
                        continue;
                    }
                    if let Some(game) = gameplay.as_mut() {
                        if !loaded && !(ready && game.accepts_kart_loading_application(&inner)) {
                            set_trace(
                                &mut trace,
                                event_index,
                                "ignored",
                                None,
                                Some("gameplay_not_loaded".into()),
                                None,
                            );
                            continue;
                        }
                        // W174: these requests are accepted and discarded before gameplay.
                        if application_preload::quiet_application(&inner) {
                            outcome.applications.push(inner.clone());
                            continue;
                        }
                        let mut staged_game = game.clone();
                        // `realm role` and the L042 moderation words (gateway/staff.rs).
                        if crate::chat::request(&inner, state.guid)
                            .ok()
                            .flatten()
                            .is_some_and(|text| self.staff_command(game, &text, &mut outcome))
                        {
                            continue;
                        }
                        let result = match staged_game.handle_in_world(&inner, &state, now_ms) {
                            Ok(result) => result,
                            Err(error) => {
                                if combined {
                                    return Err(error);
                                }
                                set_trace(
                                    &mut trace,
                                    event_index,
                                    "rejected",
                                    Some("gateway.application"),
                                    Some(error.to_string()),
                                    None,
                                );
                                continue;
                            }
                        };
                        if !result.changed
                            && result.replies.is_empty()
                            && result.relocation.is_none()
                            && result.zoning.is_none()
                            && result.scene_zoning.is_none()
                            && result.shared_field.is_none()
                        {
                            // Known handlers may legitimately reply later (the
                            // social router runs after this); decided from the
                            // bytes, not from trace records, which exist only
                            // with --trace (chat was dropped without it).
                            let known_no_reply =
                                application_handler(&inner) != "gameplay.handle_in_world";
                            if !known_no_reply {
                                set_trace(
                                    &mut trace,
                                    event_index,
                                    "rejected",
                                    Some("gateway.application"),
                                    Some("unknown application message".into()),
                                    None,
                                );
                                continue;
                            }
                        }
                        outcome.shared_field = result.shared_field;
                        // T199: a later non-broadcast child in the same bundle must not
                        // clear an earlier notice; two broadcasts keep the last.
                        outcome.broadcast = result.broadcast.or(outcome.broadcast.take());
                        outcome.admin_move = result.admin_move.or(outcome.admin_move.take());
                        outcome.world_event = result.world_event.or(outcome.world_event.take());
                        outcome.boombox = result.boombox.or(outcome.boombox.take());
                        outcome.nearby.extend(result.nearby);
                        outcome.dev_command_denials += usize::from(result.dev_command_denied);
                        outcome.applications.push(inner.clone());
                        gameplay = Some(staged_game);
                        outcome.state_changed |= result.changed;
                        outcome.gameplay_changed |= result.changed;
                        if let Some(relocation) = result.relocation {
                            // A developer teleport stays in the current zone.
                            // Stage the position before the packet is built so
                            // the replied bytes and the saved snapshot agree, and
                            // send it before the chat reply like the pinned
                            // community handler does.
                            state.position = relocation.position;
                            state.rotation = relocation.rotation;
                            movement_state =
                                MovementState::new(state.zone_id, state.position, state.rotation);
                            expected_discontinuity =
                                Some(ExpectedDiscontinuity::developer_teleport(
                                    state.zone_id,
                                    state.position,
                                ));
                            last_movement_ms = Some(now_ms);
                            outcome.state_changed = true;
                            outcome.replies.push(wrapped(&dev_teleport::location_packet(
                                state.position,
                                state.rotation,
                            )));
                        }
                        outcome
                            .replies
                            .extend(result.replies.iter().map(|reply| wrapped(reply)));
                        if let Some(zone) = result.zoning {
                            outcome.began_zoning = true;
                            state.zone_id = zone.zone_id;
                            state.position = zone.position;
                            state.rotation = zone.rotation;
                            movement_state =
                                MovementState::new(state.zone_id, state.position, state.rotation);
                            expected_discontinuity = Some(ExpectedDiscontinuity::zone_change(
                                state.zone_id,
                                state.position,
                            ));
                            last_movement_ms = Some(now_ms);
                            outcome.state_changed = true;
                            ready = false;
                            loaded = false;
                            outcome.replies.push(wrapped(&zone_begin_packet(&zone)));
                        }
                        if let Some(scene) = result.scene_zoning {
                            outcome.began_zoning = true;
                            state.zone_id = u32::try_from(scene.id)
                                .map_err(|_| invalid("negative native scene instance"))?;
                            state.position = scene.position[..3].try_into().unwrap();
                            state.rotation = scene.rotation[..3].try_into().unwrap();
                            movement_state =
                                MovementState::new(state.zone_id, state.position, state.rotation);
                            expected_discontinuity = Some(ExpectedDiscontinuity::zone_change(
                                state.zone_id,
                                state.position,
                            ));
                            last_movement_ms = Some(now_ms);
                            outcome.state_changed = true;
                            ready = false;
                            loaded = false;
                            outcome
                                .replies
                                .push(crate::zone_transition::encode(&scene)?);
                        }
                    }
                    if gameplay.is_none() {
                        outcome.applications.push(inner.clone());
                        set_trace(
                            &mut trace,
                            event_index,
                            "forwarded",
                            Some("gateway.application_router"),
                            None,
                            None,
                        );
                    }
                }
            }
        }
        budget.stage(&outcome.replies[counted_replies..])?;
        self.jump_relay =
            impulse.map(|impulse| crate::jump_relay::relay(state.guid, &movement_state, impulse));
        self.state = state;
        self.gameplay = gameplay;
        self.movement_state = movement_state;
        self.expected_discontinuity = expected_discontinuity;
        self.last_movement_ms = last_movement_ms;
        self.logged_in = logged_in;
        self.ready = ready;
        self.loaded = loaded;
        self.welcomed = welcomed;
        outcome.trace = trace.unwrap_or_default();
        Ok(outcome)
    }

    pub fn tick_at(&mut self, now_ms: u64) -> io::Result<GatewayOutcome> {
        if !self.loaded
            && !(self.ready
                && self
                    .gameplay
                    .as_ref()
                    .is_some_and(|g| g.accepts_kart_loading_application(&[33, 0])))
        {
            return Ok(GatewayOutcome::default());
        }
        let Some(game) = &self.gameplay else {
            return Ok(GatewayOutcome::default());
        };
        if !game.is_opening()
            && !game.accepts_kart_loading_application(&[33, 0])
            && !game.derby_active()
            && !game.is_soccer_active()
        {
            return Ok(self.pet_tick(now_ms));
        }
        let mut gameplay = game.clone();
        let result = gameplay.tick_with_pets(&self.state, now_ms)?;
        let mut outcome = GatewayOutcome {
            group_heal: result.group_heal,
            state_changed: result.changed,
            gameplay_changed: result.changed,
            replies: result.replies.iter().map(|r| wrapped(r)).collect(),
            nearby: result.nearby.clone(),
            ..Default::default()
        };
        if let Some(zone) = result.zoning {
            outcome.began_zoning = true;
            self.state.zone_id = zone.zone_id;
            self.state.position = zone.position;
            self.state.rotation = zone.rotation;
            self.movement_state =
                MovementState::new(self.state.zone_id, self.state.position, self.state.rotation);
            self.expected_discontinuity = Some(ExpectedDiscontinuity::zone_change(
                self.state.zone_id,
                self.state.position,
            ));
            self.last_movement_ms = Some(now_ms);
            self.ready = false;
            self.loaded = false;
            outcome.state_changed = true;
            outcome.replies.push(wrapped(&zone_begin_packet(&zone)));
        }
        if let Some(scene) = result.scene_zoning {
            outcome.began_zoning = true;
            self.state.zone_id =
                u32::try_from(scene.id).map_err(|_| invalid("negative native scene instance"))?;
            self.state.position = scene.position[..3].try_into().unwrap();
            self.state.rotation = scene.rotation[..3].try_into().unwrap();
            self.movement_state =
                MovementState::new(self.state.zone_id, self.state.position, self.state.rotation);
            self.expected_discontinuity = Some(ExpectedDiscontinuity::zone_change(
                self.state.zone_id,
                self.state.position,
            ));
            self.last_movement_ms = Some(now_ms);
            self.ready = false;
            self.loaded = false;
            outcome.state_changed = true;
            outcome
                .replies
                .push(crate::zone_transition::encode(&scene)?);
        }
        self.gameplay = Some(gameplay);
        Ok(outcome)
    }
}

fn load_profile_rank_offsets(content: &Path) -> io::Result<BTreeMap<i32, usize>> {
    let path = content.join("profiles.json");
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(1_048_577).read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err(invalid("profile content exceeds bounds"));
    }
    let layout: ProfileLayoutFile = serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if layout.schema_version != 1 || layout.profiles.len() > 32 {
        return Err(invalid("unsupported profile layout"));
    }
    let mut offsets = BTreeMap::new();
    for profile in layout.profiles {
        if profile.id <= 0
            || profile.rank_offset < 4
            || offsets.insert(profile.id, profile.rank_offset).is_some()
        {
            return Err(invalid("invalid profile rank layout"));
        }
    }
    Ok(offsets)
}

#[cfg(test)]
mod radial_routing {
    use super::application_handler;

    #[test]
    fn player_radial_requests_have_a_known_handler() {
        let mut select = vec![26, 0, 19, 0];
        select.extend([0u8; 16]);
        assert_eq!(application_handler(&select), "player_radial");
        // StopInspect is u16 149 + u8 2: three bytes, no u16 sub-opcode.
        assert_eq!(application_handler(&[149, 0, 2]), "player_radial");
        // Neighbouring traffic keeps its handler.
        assert_eq!(application_handler(&[26, 0, 10, 0]), "npc_interaction");
        assert_eq!(
            application_handler(&[149, 0, 3]),
            "gameplay.handle_in_world"
        );
    }

    #[test]
    fn trade_requests_are_forwarded_to_the_binary() {
        // W163: the six client trade senders (sub 3 reply, 6 accept, 7 cancel, 8 lock, 9 item,
        // 10 coins). Server-only subs and a bare header stay unknown.
        for sub in [3u8, 6, 7, 8, 9, 10] {
            assert_eq!(application_handler(&[0x3b, 0, sub, 1, 0, 0, 0]), "trade");
        }
        assert_eq!(
            application_handler(&[0x3b, 0, 5, 1, 0, 0, 0]),
            "gameplay.handle_in_world"
        );
        assert_eq!(application_handler(&[0x3b, 0]), "gameplay.handle_in_world");
    }

    #[test]
    fn chat_settings_requests_are_forwarded_to_the_binary() {
        // W173: 26/18 bubble colour, 26/32 channel on, 26/33 channel off.
        for sub in [18u8, 32, 33] {
            assert_eq!(
                application_handler(&[26, 0, sub, 0, 0, 0, 0, 0]),
                "chat_settings"
            );
        }
        assert_eq!(
            application_handler(&[26, 0, 34, 0]),
            "gameplay.handle_in_world"
        );
    }

    fn head(family: u16, sub: u16) -> Vec<u8> {
        [family.to_le_bytes(), sub.to_le_bytes()].concat()
    }

    #[test]
    fn handled_requests_carry_their_handler_names() {
        // W174: these reach their handlers inside gameplay.rs; a refusal that sends nothing must
        // not be logged as an unknown message.
        for (family, sub, label) in [
            (42, 6, "item_bar"),
            (42, 9, "item_bar"),
            (42, 15, "item_use"),
            (42, 16, "item_use"),
            (26, 24, "recipe_book"),
            (26, 13, "class_selection"),
            (36, 12, "combat"),
            (41, 122, "encounter"),
            (41, 109, "encounter"),
            (34, 6, "derby"),
            (56, 0, "warp"),
            (58, 0, "warp"),
            (90, 0, "warp"),
            (122, 0, "warp"),
            (141, 1, "matchmaking"),
            (141, 3, "matchmaking"),
            (141, 13, "matchmaking"),
            (41, 129, "matchmaking"),
        ] {
            let mut body = head(family, sub);
            body.extend([0u8; 8]);
            assert_eq!(application_handler(&body), label, "{family}/{sub}");
        }
        // T007: family 101 has a one-byte subtype; 101/9 carries a u32 lobby type.
        assert_eq!(application_handler(&[101, 0, 9, 3, 0, 0, 0]), "matchmaking");
        assert_eq!(
            application_handler(&[101, 0, 1, 0]),
            "gameplay.handle_in_world"
        );
        // 167 carries a service byte and a one-byte sub: 1/2 Play and 1/4 Refresh only.
        assert_eq!(application_handler(&[167, 0, 1, 4]), "activity_portal");
        assert_eq!(application_handler(&[53, 0, 4, 0, 0, 0, 0, 1]), "pets");
        assert_eq!(application_handler(&[53, 0, 6, 1, 0, 0, 0, b'A']), "pets");
        assert_eq!(application_handler(&[53, 0, 24, 2, 0, 0, 0]), "pets");
        assert_eq!(application_handler(&[39, 0, 37, 1, 0, 0, 0]), "card_game");
        assert_eq!(
            application_handler(&[53, 0, 19, 1, 0, 0, 0, 0, 0, 0, 0]),
            "pets"
        );
        assert_eq!(application_handler(&[53, 0, 28, 1]), "pets");
        assert_eq!(
            application_handler(&[53, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0]),
            "pets"
        );
        assert_eq!(
            application_handler(&[53, 0, 0xff, 0xff]),
            "gameplay.handle_in_world"
        );
        assert_eq!(
            application_handler(&[167, 0, 1, 2, 0, 0, 0, 0]),
            "activity_portal"
        );
        assert_eq!(
            application_handler(&[167, 0, 1, 1]),
            "gameplay.handle_in_world"
        );
        assert_eq!(
            application_handler(&[167, 0, 2, 4]),
            "gameplay.handle_in_world"
        );
    }

    #[test]
    fn telemetry_and_benign_requests_are_a_named_quiet_list() {
        // W174 (R): accepted, answered with nothing, never stored or relayed.
        let mut metrics = head(105, 1);
        metrics.extend([0u8; 146]);
        assert_eq!(application_handler(&metrics), "client_metrics");
        // 109 is two strings after the opcode, no sub-opcode: any first byte is the same message.
        for first in [17u8, 18, 19, 20, 25] {
            assert_eq!(application_handler(&[109, 0, first, 0, 0, 0]), "client_log");
        }
        // 194 has a one-byte sub (2 and 3 in the traces).
        assert_eq!(
            application_handler(&[194, 0, 2, 1, 0, 0, 0]),
            "ui_telemetry"
        );
        assert_eq!(application_handler(&[194, 0, 3, 5, 0]), "ui_telemetry");
        // 193/1 is the client's AnnouncementDataRequestPacket; 193/2 and 193/4 are server sends.
        assert_eq!(application_handler(&[193, 0, 1]), "announcements_request");
        assert_eq!(
            application_handler(&[193, 0, 2]),
            "gameplay.handle_in_world"
        );
        assert_eq!(
            application_handler(&[193, 0, 4]),
            "gameplay.handle_in_world"
        );
        // Bare family 16 (two bytes) is the exit notice (R); anything longer stays unknown.
        assert_eq!(application_handler(&[16, 0]), "client_exit_notice");
        assert_eq!(application_handler(&[16, 0, 0]), "gameplay.handle_in_world");
        assert_eq!(
            application_handler(&[16, 0, 0, 0]),
            "gameplay.handle_in_world"
        );
        assert_eq!(application_handler(&head(26, 20)), "free_interaction");
        assert_eq!(application_handler(&head(26, 26)), "profile_notice");
        assert_eq!(application_handler(&head(26, 37)), "profile_notice");
    }

    #[test]
    fn coin_store_and_marketplace_siblings_are_merchant_traffic() {
        // W172: the five login-time Marketplace requests and the Coin Store siblings are
        // understood requests, so a silent refusal must not log as an unknown message.
        for sub in [1u8, 3, 10, 12, 14, 16, 18, 20, 22, 25, 27, 38, 43] {
            assert_eq!(application_handler(&[66, 0, sub, 0]), "merchant");
        }
        for sub in [2u8, 4, 5, 8, 12, 14] {
            assert_eq!(application_handler(&[165, 0, sub, 0]), "merchant");
        }
        // Unrelated neighbours stay unknown (6 is the server-to-client completion).
        assert_eq!(
            application_handler(&[165, 0, 6, 0]),
            "gameplay.handle_in_world"
        );
        assert_eq!(
            application_handler(&[66, 0, 11, 0]),
            "gameplay.handle_in_world"
        );
    }

    #[test]
    fn real_unknowns_still_stand_out() {
        // Neighbours of the quiet list and families nothing routes.
        for (family, sub) in [
            (104, 1),
            (106, 1),
            (192, 1),
            (195, 1),
            (777, 1),
            (0xffff, 0xffff),
        ] {
            let mut body = head(family, sub);
            body.extend([0u8; 8]);
            assert_eq!(
                application_handler(&body),
                "gameplay.handle_in_world",
                "{family}/{sub}"
            );
        }
    }
}
