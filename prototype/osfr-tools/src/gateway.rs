//! Per-character application gateway backed by private, verified world templates.
//!
//! Field layouts were checked against the supplied serializer and pinned
//! Sanctuary packet definitions. Templates are local reference data, not a
//! reconstructed content/gameplay service. This module never runs C# code.

use crate::character_store::CharacterState;
use crate::dev_teleport;
use crate::encounter::{ZoneTransition, zone_begin_packet};
use crate::game_store::Snapshot;
use crate::gameplay::{Gameplay, GameplayState};
use crate::identity::{self, CharacterIdentity};
use crate::movement::{
    self, Decision as MovementDecision, ExpectedDiscontinuity, MovementState, MovementUpdate,
    SpeedModifiers,
};
use crate::{
    players::PlayerPresentation,
    profiles::ProfileState,
    startup_selfdata::{self, SelfData},
    startup_snapshot::StartupSnapshot,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
struct Template {
    kind: Kind,
    bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct Gateway {
    state: CharacterState,
    version: String,
    templates: Vec<Template>,
    startup_snapshot: Option<StartupSnapshot>,
    template_fallback: bool,
    self_data_template: Option<SelfData>,
    profile_rank_offsets: BTreeMap<i32, usize>,
    self_layout: SelfLayout,
    zone_layout: ZoneLayout,
    logged_in: bool,
    ready: bool,
    loaded: bool,
    gameplay: Option<Gameplay>,
    identity: Option<CharacterIdentity>,
    /// Developer-only chat commands; off unless the launcher opts in.
    dev_tools: bool,
    role_commands: bool,
    terrain: Option<std::sync::Arc<crate::terrain::Terrain>>,
    world_dir: Option<std::path::PathBuf>,
    stock_text: Option<std::sync::Arc<std::collections::HashSet<u32>>>,
    role: crate::account_store::AccountRole,
    // (R) Bound only by the host after verifying the character's owner and ticket.
    role_account: Option<(std::path::PathBuf, u64)>,
    movement_state: MovementState,
    expected_discontinuity: Option<ExpectedDiscontinuity>,
    last_movement_ms: Option<u64>,
    trace_enabled: bool,
    // (R) Server-only authorization seam; no client parser sets an instance id.
    pending_transfer: Option<crate::players::ZoneInstance>,
}

#[derive(Debug, Default)]
pub struct GatewayOutcome {
    pub dev_command_denials: usize,
    pub shared_field: Option<crate::encounter::SharedFieldAction>,
    pub replies: Vec<Vec<u8>>,
    /// Inner application payloads that passed the gateway parser, in request order.
    pub applications: Vec<Vec<u8>>,
    pub state_changed: bool,
    pub entered_world: bool,
    pub gameplay_changed: bool,
    /// (R) Also true for same-scene transfers, which must reset interest membership.
    pub began_zoning: bool,
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

fn load_reference(directory: &Path) -> io::Result<(Manifest, Vec<Template>)> {
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
        templates.push(Template {
            kind: file.kind,
            bytes,
        });
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
    if derive_self_layout(&templates[7].bytes)? != manifest.self_layout
        || derive_zone_layout(&templates[3].bytes)? != manifest.zone_layout
    {
        return Err(invalid("reference field offsets differ from proven layout"));
    }
    let sl = manifest.self_layout;
    let self_bytes = &templates[7].bytes;
    let mut guid = Reader::new(&self_bytes[sl.guid_offset..]);
    let mut position = Reader::new(&self_bytes[sl.position_offset..]);
    let mut rotation = Reader::new(&self_bytes[sl.rotation_offset..]);
    let mut zone = Reader::new(&templates[3].bytes[manifest.zone_layout.zone_id_offset..]);
    if guid.u64()? != manifest.initial_state.guid
        || position.vector()? != manifest.initial_state.position
        || rotation.vector()? != manifest.initial_state.rotation
        || zone.u32()? != manifest.initial_state.zone_id
    {
        return Err(invalid(
            "reference snapshot differs from serialized character",
        ));
    }
    Ok((manifest, templates))
}

enum Event {
    Login,
    Ready,
    Loaded,
    Position(MovementUpdate),
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

fn trace_hex_prefix(bytes: &[u8], sensitive: bool) -> Option<String> {
    let head = bytes.iter().take(32).copied().collect::<Vec<_>>();
    let mut letters = 0;
    let looks_like_text = head.iter().any(|byte| {
        if byte.is_ascii_alphabetic() {
            letters += 1;
        } else if !byte.is_ascii_whitespace() {
            letters = 0;
        }
        letters >= 3
    });
    (!sensitive && !looks_like_text)
        .then(|| head.iter().map(|byte| format!("{byte:02x}")).collect())
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
        || family.zip(subtype).is_some_and(|(family, subtype)| {
            family == 1
                || matches!(family, 15 | 22 | 40 | 68 | 74 | 112 | 149 | 175)
                || (family == 26 && matches!(subtype, 14 | 15 | 16 | 17 | 30))
        });
    let handler = match event {
        Event::Login => "gateway.login",
        Event::Ready => "gateway.ready",
        Event::Loaded => "gateway.loaded",
        Event::Position(_) => "movement",
        Event::Camera => "gateway.camera",
        Event::Time => "gateway.time_sync",
        Event::Application(_) => application_handler(body),
    };
    GatewayTraceEvent {
        event: if matches!(event, Event::Position(_)) {
            "movement"
        } else {
            "packet"
        },
        tunnel_outer_opcode: outer,
        family,
        subtype,
        body_len: body.len(),
        body_head_hex: trace_hex_prefix(body, sensitive),
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
    let word = |at: usize| {
        body.get(at..at + 2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    };
    match (word(0), word(2)) {
        (Some(15), Some(1)) => "social_chat",
        (Some(67), Some(2 | 3)) => "quick_chat",
        (Some(26), Some(4 | 6 | 8 | 10 | 23)) => "npc_interaction",
        (Some(26), Some(19)) => "player_radial",
        (Some(26), Some(14 | 15 | 17 | 21 | 30)) => "social_router",
        (Some(40), Some(1 | 3 | 4 | 6 | 14 | 16)) => "social_router",
        (Some(49), Some(2 | 14 | 15)) => "quest_engine",
        // Family 47 carries a one-byte subtype; word(2) would swallow the first body byte.
        (Some(47), _) if matches!(body.get(2), Some(12 | 13)) => "quest_engine",
        // Family 59 also has a one-byte subtype: trade requests, answered as unavailable (W156, R).
        (Some(59), _) if matches!(body.get(2), Some(3 | 6..=10)) => "trade_unavailable",
        (Some(26), Some(42)) | (Some(33), _) | (Some(41), Some(108)) | (Some(102), Some(1)) => {
            "kart"
        }
        (Some(36), Some(10 | 12)) => "combat",
        (Some(42), Some(2 | 3 | 8)) | (Some(35), Some(34)) => "inventory",
        (Some(66), Some(43)) | (Some(165), Some(4 | 5)) => "merchant",
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
                    Event::Position(MovementUpdate {
                        position,
                        direction,
                        state: reader.u8()?,
                        unknown: reader.u8()?,
                    })
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
        let mut size = reader.u8()? as usize;
        if size == 255 {
            size = u16::from_be_bytes(reader.take(2)?.try_into().unwrap()) as usize;
            if size == 255 {
                size = u32::from_be_bytes(reader.take(4)?.try_into().unwrap()) as usize;
            }
        }
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

/// PacketSendZoneDetails from the pinned community serializer (opcode 43).
fn initial_zone(zone: &ZoneTransition) -> Vec<u8> {
    let mut bytes = 43u16.to_le_bytes().to_vec();
    bytes.extend((zone.name.len() as u32).to_le_bytes());
    bytes.extend(zone.name.bytes());
    bytes.extend(2i32.to_le_bytes());
    bytes.extend([u8::from(zone.tutorial), 0]);
    bytes.extend((zone.sky.len() as u32).to_le_bytes());
    bytes.extend(zone.sky.bytes());
    bytes.push(0);
    bytes.extend(zone.zone_id.to_le_bytes());
    bytes.extend(zone.geometry_id.to_le_bytes());
    bytes.extend([0, 0]);
    bytes
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
    pub fn take_shared_fields(&mut self) -> Option<crate::encounter::SharedFieldState> {
        self.gameplay.as_mut()?.take_shared_fields()
    }
    pub fn bind_run_instance(&mut self, instance: i32) -> io::Result<bool> {
        self.gameplay
            .as_mut()
            .map_or(Ok(false), |g| g.bind_run_instance(instance))
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
        let mut gameplay = Gameplay::load(content, saved)?;
        gameplay.bind_npc_quest_catalogue(content, state.guid)?;
        let profile_rank_offsets = load_profile_rank_offsets(content)?;
        Self::load_internal(directory, state, Some(gameplay), None, profile_rank_offsets)
    }

    fn load_internal(
        directory: &Path,
        mut state: CharacterState,
        mut gameplay: Option<Gameplay>,
        snapshot_override: Option<StartupSnapshot>,
        profile_rank_offsets: BTreeMap<i32, usize>,
    ) -> io::Result<Self> {
        state.validate()?;
        let (manifest, templates) = load_reference(directory)?;
        let template_fallback = if snapshot_override.is_some() {
            false
        } else {
            template_fallback_enabled()?
        };
        // (R) The local account-store path assigns a distinct GUID to each character;
        // the shared startup template is patched with that GUID at send time.
        let startup_snapshot = if template_fallback {
            None
        } else if let Some(snapshot) = snapshot_override {
            snapshot.validate()?;
            Some(snapshot)
        } else {
            let snapshot = StartupSnapshot::load(&configured_snapshot_path(directory)?)?;
            if snapshot.login.zone.zone_id != manifest.initial_state.zone_id {
                return Err(invalid("startup snapshot belongs to another initial zone"));
            }
            Some(snapshot)
        };
        if let Some(game) = &mut gameplay {
            if let Some(snapshot) = &startup_snapshot {
                game.exclude_startup_npcs(snapshot.world.npc_spawns.iter().map(|npc| npc.actor_id));
            } else if template_fallback {
                let actor_guids = templates
                    .iter()
                    .skip(10)
                    .take(23)
                    .map(|template| {
                        let (_, inner) = tunnel(&template.bytes)?;
                        crate::startup_world::parse_add_npc(inner)
                            .map(|npc| npc.actor_id)
                            .map_err(|_| invalid("startup NPC template could not be parsed"))
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                game.exclude_startup_npcs(actor_guids);
            }
        }
        let self_data_template = if template_fallback {
            None
        } else {
            let bytes = &templates
                .get(7)
                .ok_or_else(|| invalid("self-data reference template is missing"))?
                .bytes;
            Some(
                std::panic::catch_unwind(|| startup_selfdata::parse_self(bytes))
                    .map_err(|_| invalid("invalid typed self-data reference template"))?,
            )
        };
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
            || (gameplay.is_none() && state.zone_id != manifest.initial_state.zone_id)
        {
            return Err(invalid(
                "unsupported saved zone: reference supports only its initial zone",
            ));
        }
        let movement_state = MovementState::new(state.zone_id, state.position, state.rotation);
        Ok(Self {
            state,
            version: manifest.client_version,
            templates,
            startup_snapshot,
            template_fallback,
            self_data_template,
            profile_rank_offsets,
            self_layout: manifest.self_layout,
            zone_layout: manifest.zone_layout,
            logged_in: false,
            ready: false,
            loaded: false,
            gameplay,
            identity: None,
            dev_tools: false,
            role_commands: false,
            terrain: None,
            world_dir: None,
            stock_text: None,
            role: crate::account_store::AccountRole::Player,
            role_account: None,
            movement_state,
            expected_discontinuity: None,
            last_movement_ms: None,
            trace_enabled: false,
            pending_transfer: None,
        })
    }

    /// The live gameplay engine, for developer tooling (Studio).
    pub fn gameplay(&self) -> Option<&Gameplay> {
        self.gameplay.as_ref()
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
    /// or developer chat. Ordinary encounter consent is a later build slice.
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
        if let Some(snapshot) = &self.startup_snapshot {
            let scene = &snapshot.login.zone;
            return Ok(crate::content::Zone {
                id: self.state.zone_id,
                name: scene.name.clone(),
                geometry_id: Some(scene.geometry_id),
                zone_type: Some(scene.zone_type),
                tutorial: Some(scene.tutorial),
                sky: Some(scene.sky.clone()),
                source: Vec::new(),
            });
        }
        // Preserve the opt-in captured-template fallback using its already validated 43 row.
        let (_, payload) = tunnel(&self.templates[3].bytes)?;
        let mut reader = Reader::new(payload);
        reader.u16()?;
        let name = reader.string(4096)?.to_owned();
        let zone_type = i32::try_from(reader.u32()?).map_err(|_| invalid("zone type overflow"))?;
        let tutorial = reader.bool()?;
        reader.bool()?;
        let sky = reader.string(4096)?.to_owned();
        reader.bool()?;
        reader.u32()?;
        let geometry_id = reader.u32()?;
        Ok(crate::content::Zone {
            id: self.state.zone_id,
            name,
            geometry_id: Some(geometry_id),
            zone_type: Some(zone_type),
            tutorial: Some(tutorial),
            sky: Some(sky),
            source: Vec::new(),
        })
    }

    /// The public look other players get in AddPc (Build Plan 1.3), from the
    /// same identity, profile and inventory inputs as `build_self_message`.
    /// (R) Name ids stay zero (literal-name mode); speed and wield type keep
    /// the players.rs defaults until those are server-owned.
    /// See `Gameplay::public_look` (W143 Inspect).
    pub fn public_look(&self) -> Option<crate::player_radial::PublicLook> {
        self.gameplay.as_ref()?.public_look()
    }

    pub fn player_presentation(&self, member: bool) -> io::Result<PlayerPresentation> {
        let mut look = PlayerPresentation {
            character_guid: self.state.guid,
            member,
            ..PlayerPresentation::default()
        };
        if let Some(identity) = &self.identity {
            look.first_name.clone_from(&identity.first_name);
            look.last_name.clone_from(&identity.last_name);
            look.model = identity.model;
            look.head.clone_from(&identity.head);
            look.hair.clone_from(&identity.hair);
            look.hair_color = identity.hair_color;
            look.eye_color = identity.eye_color;
            look.skin_tone.clone_from(&identity.skin_tone);
            look.face_paint.clone_from(&identity.face_paint);
            look.model_customization
                .clone_from(&identity.model_customization);
        }
        // Without loaded profiles, self-data keeps the template's job; match it.
        let profile = self
            .gameplay
            .as_ref()
            .and_then(Gameplay::active_profile)
            .or_else(|| {
                self.self_data_template
                    .as_ref()
                    .map(|data| data.jobs.active_profile_id as i32)
            });
        if let Some(profile) = profile {
            look.active_profile = profile;
            if let Some(game) = &self.gameplay {
                look.attachments_wire = game.profile_attachments(profile)?;
            }
        }
        Ok(look)
    }

    /// The last accepted movement state and unknown byte, relayed to others.
    pub fn movement_bytes(&self) -> (u8, u8) {
        (self.movement_state.state, self.movement_state.unknown)
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
    }

    /// (R) Re-read on use so online demotion takes effect in every session.
    /// An unreadable store, missing account or missing role fails closed.
    pub fn refresh_role(&mut self) {
        if let Some((path, id)) = &self.role_account {
            let role = crate::account_store::AccountStore::load(path)
                .ok()
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
        use crate::account_store::{AccountRole, AccountStore, AccountStoreLock};
        let words: Vec<_> = command.split_whitespace().collect();
        let ["role", id, role] = words.as_slice() else {
            return "Usage: realm role <account-id> <player|builder|admin>";
        };
        let (Some(target), Some(role)) = (
            id.parse::<u64>().ok().filter(|id| *id > 0),
            AccountRole::parse(role),
        ) else {
            return "Usage: realm role <account-id> <player|builder|admin>";
        };
        let Some((path, actor)) = &self.role_account else {
            return "Cannot do that: a verified account binding is required.";
        };
        // (R) Cooperate with login/CLI writers; never overwrite a newer key or role.
        let Ok(_lock) = AccountStoreLock::acquire(path) else {
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

    /// Ground heights for enemy movement and developer teleports.
    pub fn set_terrain(&mut self, terrain: Option<std::sync::Arc<crate::terrain::Terrain>>) {
        if let Some(gameplay) = self.gameplay.as_mut() {
            gameplay.set_terrain(terrain.clone());
        }
        self.terrain = terrain;
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
            character: self.state.clone(),
            gameplay: self.gameplay.as_ref().map(Gameplay::state),
        }
    }

    pub fn set_identity(&mut self, identity: CharacterIdentity) -> io::Result<bool> {
        if identity.guid != self.state.guid {
            return Err(invalid("identity GUID differs from character save"));
        }
        let canonical_identity = creation_identity(&identity);
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

    /// Phrase IDs are sourced from the same typed startup data sent to this client.
    pub fn quick_chat_catalog(&self) -> crate::social_chat::QuickChatCatalog {
        crate::social_chat::QuickChatCatalog::from_rows(
            self.startup_snapshot
                .as_ref()
                .into_iter()
                .flat_map(|snapshot| {
                    snapshot
                        .world
                        .quick_chat
                        .iter()
                        .map(|row| (row.id, row.animation_id))
                }),
        )
    }

    /// (R) Actor state for emote validation. `mounted` stays false until the
    /// mount service is routed into gameplay (mounts.rs is not wired yet).
    pub fn emote_context(&self, now_ms: u64) -> crate::social_chat::piece_093::EmoteContext {
        crate::social_chat::emote_context(false, self.last_movement_ms, now_ms)
    }

    /// Begin a new local session while retaining the latest character snapshot.
    pub fn reset_session(&mut self) {
        self.logged_in = false;
        self.ready = false;
        self.loaded = false;
        if let Some(game) = self.gameplay.as_mut() {
            if let Some(return_location) = game.abandon_kart_for_reconnect() {
                self.state = return_location;
            }
            game.reset_session();
        }
        self.movement_state =
            MovementState::new(self.state.zone_id, self.state.position, self.state.rotation);
        self.expected_discontinuity = None;
        self.last_movement_ms = None;
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
        let (mut logged_in, mut ready, mut loaded) = (self.logged_in, self.ready, self.loaded);
        let mut outcome = GatewayOutcome::default();
        for (event_index, event) in events.into_iter().enumerate() {
            match event {
                Event::Login => {
                    if !logged_in {
                        outcome.replies.extend(self.startup_for(
                            Kind::Login,
                            &state,
                            gameplay.as_ref(),
                        )?);
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
                            )?);
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
                Event::Position(update) => {
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
                    // (R) Base speed and mount/buff modifiers are not server-owned
                    // yet; 10 units/s with the validator's generous capture-tested
                    // envelope preserves W53b reports while rejecting gross jumps.
                    let validated = movement::validate(
                        &movement_state,
                        update,
                        state.zone_id,
                        elapsed,
                        SpeedModifiers {
                            base_speed_per_second: 10.0,
                            mount_multiplier: 1.0,
                            buff_multiplier: 1.0,
                        },
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
                    let mut inner = 52u16.to_le_bytes().to_vec();
                    inner.extend_from_slice(&seconds.to_le_bytes());
                    inner.extend_from_slice(&0i32.to_le_bytes());
                    inner.push(1);
                    outcome.replies.push(wrapped(&inner));
                }
                Event::Application(inner) => {
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
                        let mut staged_game = game.clone();
                        if let Some(text) = crate::chat::request(&inner, state.guid).ok().flatten()
                        {
                            let lower = text.trim().to_ascii_lowercase();
                            if let Some(command) = lower
                                .strip_prefix("realm ")
                                .filter(|c| c.split_whitespace().next() == Some("role"))
                            {
                                let allowed = game.may_run("role");
                                let reply = if allowed {
                                    self.change_account_role(command)
                                } else {
                                    outcome.dev_command_denials += 1;
                                    "Cannot do that: developer command requires --dev-tools and an authorized account role."
                                };
                                game.set_role(self.role);
                                outcome.replies.push(wrapped(&crate::chat::system(reply)));
                                continue;
                            }
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
        self.state = state;
        self.gameplay = gameplay;
        self.movement_state = movement_state;
        self.expected_discontinuity = expected_discontinuity;
        self.last_movement_ms = last_movement_ms;
        self.logged_in = logged_in;
        self.ready = ready;
        self.loaded = loaded;
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
        if !game.is_opening() && !game.accepts_kart_loading_application(&[33, 0]) {
            return Ok(GatewayOutcome::default());
        }
        let mut gameplay = game.clone();
        let result = gameplay.tick(&self.state, now_ms)?;
        let mut outcome = GatewayOutcome {
            state_changed: result.changed,
            gameplay_changed: result.changed,
            replies: result.replies.iter().map(|r| wrapped(r)).collect(),
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

    fn templates_for(
        &self,
        kind: Kind,
        state: &CharacterState,
        gameplay: Option<&Gameplay>,
    ) -> io::Result<Vec<Vec<u8>>> {
        self.templates
            .iter()
            .enumerate()
            .filter(|(index, t)| {
                t.kind == kind
                    && !(gameplay.is_some_and(Gameplay::is_opening) && (10..=32).contains(index))
            })
            .map(|(index, t)| {
                let mut bytes = t.bytes.clone();
                if index == 7 {
                    bytes[self.self_layout.guid_offset..self.self_layout.guid_offset + 8]
                        .copy_from_slice(&state.guid.to_le_bytes());
                    for (offset, vector) in [
                        (self.self_layout.position_offset, state.position),
                        (self.self_layout.rotation_offset, state.rotation),
                    ] {
                        for (component, value) in vector.iter().enumerate() {
                            bytes[offset + component * 4..offset + component * 4 + 4]
                                .copy_from_slice(&value.to_le_bytes());
                        }
                    }
                } else if index == 3 {
                    bytes[self.zone_layout.zone_id_offset..self.zone_layout.zone_id_offset + 4]
                        .copy_from_slice(&state.zone_id.to_le_bytes());
                }
                if index == 7 {
                    if let Some(game) = gameplay {
                        if let Some(identity) = &self.identity {
                            bytes = game.patch_gender(&bytes, identity.gender)?;
                        }
                        bytes = game.patch_self(&bytes)?;
                    }
                    if let Some(identity) = &self.identity {
                        bytes = identity::patch_self(&bytes, &creation_identity(identity))?;
                    }
                } else if index == 5 {
                    if let Some(packet) = gameplay.and_then(Gameplay::definitions_packet) {
                        bytes = wrapped(&packet);
                    }
                } else if index == 3 {
                    if let Some(zone) = gameplay.and_then(Gameplay::current_zone) {
                        bytes = wrapped(&initial_zone(&zone));
                    }
                }
                Ok(bytes)
            })
            .collect()
    }

    fn startup_for(
        &self,
        kind: Kind,
        state: &CharacterState,
        gameplay: Option<&Gameplay>,
    ) -> io::Result<Vec<Vec<u8>>> {
        if self.template_fallback {
            return self.templates_for(kind, state, gameplay);
        }
        let snapshot = self
            .startup_snapshot
            .as_ref()
            .ok_or_else(|| invalid("typed startup snapshot is not loaded"))?;
        let range = match kind {
            Kind::Login => 0..8,
            Kind::Ready => 8..46,
        };
        range
            .filter(|index| {
                !(gameplay.is_some_and(Gameplay::is_opening) && (10..=32).contains(index))
            })
            .map(|index| {
                let message = match index {
                    3 => {
                        let zone = gameplay.and_then(Gameplay::current_zone);
                        snapshot.build_zone(state.zone_id, zone.as_ref())
                    }
                    5 => match gameplay.and_then(Gameplay::definitions_packet) {
                        Some(inner) => wrapped(&inner),
                        None => snapshot.build_packet(index, state.guid)?,
                    },
                    7 => self.build_self_message(state, gameplay)?,
                    _ => snapshot.build_packet(index, state.guid)?,
                };
                Ok(message)
            })
            .collect()
    }

    fn build_self_message(
        &self,
        state: &CharacterState,
        gameplay: Option<&Gameplay>,
    ) -> io::Result<Vec<u8>> {
        let mut data = self
            .self_data_template
            .as_ref()
            .ok_or_else(|| invalid("typed self-data template is not loaded"))?
            .clone();
        data.identity.guid = state.guid;
        for index in 0..3 {
            data.transform.position[index] = state.position[index].to_bits();
            data.transform.rotation[index] = state.rotation[index].to_bits();
        }
        if let Some(identity) = &self.identity {
            data.identity.model = identity.model as u32;
            data.appearance.head.clone_from(&identity.head);
            data.appearance.hair.clone_from(&identity.hair);
            data.appearance.hair_color = identity.hair_color as u32;
            data.appearance.eye_color = identity.eye_color as u32;
            data.appearance.skin_tone.clone_from(&identity.skin_tone);
            data.appearance.face_paint.clone_from(&identity.face_paint);
            data.appearance
                .model_customization
                .clone_from(&identity.model_customization);
            data.appearance.customization_ids =
                [1, 2, 5, 4, 3].map(|index| identity.creation.choices[index] as u32);
            data.name.first.clone_from(&identity.first_name);
            data.name.last.clone_from(&identity.last_name);
            data.gender = identity.gender as u32;
        }
        if let Some(game) = gameplay {
            let current = game.state();
            data.inventory = current.inventory;
            data.quests = current.quests;
            if let Some(profiles) = current.profiles {
                self.apply_profile_state(&mut data, &profiles)?;
            }
        }
        // (R) Action-bar assignments have no field in the current saved
        // GameplayState, so preserve the captured typed rows until that state exists.
        // The module-owned opaque post-effect tail remains untouched.
        Ok(startup_selfdata::build_self(&data))
    }

    fn apply_profile_state(&self, data: &mut SelfData, profiles: &ProfileState) -> io::Result<()> {
        data.jobs.active_profile_id = profiles.active as u32;
        for (id, rank_offset) in &self.profile_rank_offsets {
            let row = data
                .jobs
                .profiles
                .iter_mut()
                .find(|row| row.map_key == *id as u32)
                .ok_or_else(|| invalid("self-data is missing a configured job profile"))?;
            // The content offset includes the four-byte profile-map key; the
            // typed row stores its opaque prefix after that key.
            let offset = rank_offset
                .checked_sub(4)
                .ok_or_else(|| invalid("invalid job progression offset"))?;
            let end = offset
                .checked_add(12)
                .ok_or_else(|| invalid("job progression offset overflow"))?;
            let progression = row
                .opaque_before_equipment_without_key
                .get_mut(offset..end)
                .ok_or_else(|| invalid("self-data job progression field is truncated"))?;
            let xp = profiles.experience.get(id).copied().unwrap_or(0);
            let rank = (1 + xp / 100).min(20) as i32;
            let progress = if rank == 20 { 0 } else { (xp % 100) as i32 };
            // (R) The local rank curve and zero-star policy match Profiles.
            progression[..4].copy_from_slice(&rank.to_le_bytes());
            progression[4..8].copy_from_slice(&progress.to_le_bytes());
            progression[8..12].copy_from_slice(&0i32.to_le_bytes());
        }
        Ok(())
    }
}

fn creation_identity(identity: &CharacterIdentity) -> CharacterIdentity {
    let mut canonical = identity.clone();
    // (R) The creation identity type reserves GUID 1; self-data carries the
    // selected character GUID in its separate identity field.
    canonical.guid = 1;
    canonical
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

fn configured_snapshot_path(reference: &Path) -> io::Result<std::path::PathBuf> {
    if let Some(path) = std::env::var_os("OSFR_GATEWAY_STARTUP_SNAPSHOT") {
        return Ok(path.into());
    }
    let lab_root = reference
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("set OSFR_GATEWAY_STARTUP_SNAPSHOT to the typed startup JSON"))?;
    Ok(lab_root.join("content/startup-snapshot/startup.json"))
}

fn template_fallback_enabled() -> io::Result<bool> {
    match std::env::var("OSFR_GATEWAY_STARTUP_TEMPLATE_FALLBACK") {
        Ok(value) if value == "1" || value.eq_ignore_ascii_case("true") => Ok(true),
        Ok(value) if value == "0" || value.eq_ignore_ascii_case("false") => Ok(false),
        Ok(_) => Err(invalid(
            "OSFR_GATEWAY_STARTUP_TEMPLATE_FALLBACK must be 0, 1, true, or false",
        )),
        Err(std::env::VarError::NotPresent) => Ok(false),
        Err(std::env::VarError::NotUnicode(_)) => Err(invalid(
            "OSFR_GATEWAY_STARTUP_TEMPLATE_FALLBACK is not Unicode",
        )),
    }
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
    fn trade_requests_are_forwarded_to_the_unavailable_answer() {
        // Client trade senders: sub 3 reply, 6 accept, 7 cancel, 8 lock, 9 item, 10 coins.
        for sub in [3u8, 6, 7, 8, 9, 10] {
            assert_eq!(
                application_handler(&[0x3b, 0, sub, 1, 0, 0, 0]),
                "trade_unavailable"
            );
        }
        // Server-only subs stay unknown, and so does a bare header.
        assert_eq!(
            application_handler(&[0x3b, 0, 5, 1, 0, 0, 0]),
            "gameplay.handle_in_world"
        );
        assert_eq!(application_handler(&[0x3b, 0]), "gameplay.handle_in_world");
    }
}
