//! Native kart race-entry handler: the tunneled inner-opcode-33 register exchange.
//!
//! ORIGINAL facts implemented here (nothing about field *meaning* is invented):
//!
//! * the client request is inner `33/9` `VehicleRacePacketRegisterVehicleRacePlayer`, a 16-byte
//!   payload `u64 session | u32 | u32`; the capture carries `session 91ca73956acc524b`,
//!   `u32 0x2a6ac44a`, `u32 1`, and the two `u32` word meanings stay unrecovered
//!   (`notes/Research/Kart Native Race Entry Contract.md`, field map `33/9`).
//!   The session is read little-endian like every other field in this family, so the captured wire
//!   bytes `91ca73956acc524b` are the value `0x4b52cc6a9573ca91`; the field map's `session_u64_le`
//!   name is a byte-order rendering, not a value.
//! * the ordered server answer is `33/34` `SetRaceConfig` (84-byte body), one `33/11`
//!   `CreateProxiedVehicleRace` per entry vehicle (`u32 name length | name | f32 | attribute block`),
//!   `33/5` `UpdateVehicleRaceClientData` (85-byte body) and one `33/14`
//!   `UpdateVehicleRaceProgressData` per racer (73 bytes, decoded by the registered
//!   `crate::kart_progress_native`). The capture order is exactly that.
//! * the client join `33/2` `UpdateVehicleGamePlayerState` (`u64 session | u32`) is answered with
//!   `33/3` `UpdatePlayerCount` and `33/1` `UpdateVehicleGameState` (one `i32` each). The captured
//!   values are, in order, `1` and `3`.
//! * the later captured bundle repeats `33/5`/`33/14` with *different* payloads and carries `33/1`
//!   = `4` plus `33/4` `UpdateNpcCount` = `7`. Those two bulk payloads belong to the live racer
//!   update stream, so [`KartEntryHandler::on_race_ready`] emits only the recovered state/npc-count
//!   transitions and never re-authors the bulk blocks.
//! * the scene is named by opcode 31 (registered `crate::zone_transition`, `PacketClientBeginZoning`)
//!   and the captured `sg_kt_track` frame carries rotation `(0, 0, 1, 0)` - `w = 0`. The encounter
//!   writer `encounter::zone_begin_packet` hardcodes `w = 1.0`, so this module encodes the scene
//!   through the registered zone codec and exposes it separately from the inner reply list.
//!
//! WHAT THIS MODULE DELIBERATELY DOES NOT DO:
//!
//! * no finish, results, placement, payout or economy path exists here; no captured result message
//!   exists and nothing in this module grants, prices or mutates inventory, quests, classes or
//!   character state. Refusals and malformed frames return before any state movement.
//! * no physics, gates or positions are derived from client frames. Client control subtypes
//!   (`33/6`, `33/7`, `33/17`, ...) are not handled: [`EntryRefusal::UnsupportedSubtype`] is the
//!   whole answer. There is no derby path.
//! * opaque captured blocks stay opaque: the `33/34` body is carried as its 21 wire words with no
//!   invented names, the `33/5` body as a verbatim byte block, and the `33/11` tail as a verbatim
//!   attribute block. The lead's private content package supplies them; nothing here fabricates
//!   field semantics.
//!
//! Framing: the gateway hands [`KartEntryHandler::handle_inner`] the *inner* bytes
//! (`u16 opcode 33 | u16 subtype | payload`, no length field for this family) and wraps every reply
//! it returns. This module never emits a tunnel envelope, and the op31 scene frame is returned only
//! through [`KartEntryHandler::scene_frame`], never inside the reply list.
//!
//! ponytail: one flat handler struct, one ordered reply vector, no trait objects, no timers, no
//! per-racer simulation. Add a subtype only when a recovered reader names its layout.

use crate::kart_progress_native::{self, ProgressPayload};
use crate::zone_transition::{self, ZoneTransition};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

pub use crate::kart_racing::VEHICLE_RACE_INNER_OPCODE;

/// ORIGINAL: subtype of the client register request.
pub const REGISTER_SUBTYPE: u16 = 9;
/// ORIGINAL: subtype of the client join/player-state request.
pub const PLAYER_STATE_SUBTYPE: u16 = 2;
/// ORIGINAL: subtype of the server game-state counter.
pub const GAME_STATE_SUBTYPE: u16 = crate::kart_progress_native::GAME_STATE_SUBTYPE;
/// ORIGINAL: subtype of the server player-count counter.
pub const PLAYER_COUNT_SUBTYPE: u16 = 3;
/// ORIGINAL: subtype of the server npc-count counter.
pub const NPC_COUNT_SUBTYPE: u16 = 4;
/// ORIGINAL: subtype of the bulk racer client-data block.
pub const CLIENT_DATA_SUBTYPE: u16 = 5;
/// ORIGINAL: subtype of a proxied racer creation.
pub const CREATE_VEHICLE_SUBTYPE: u16 = 11;
/// ORIGINAL: subtype of the 73-byte progress block (registered progress codec).
pub const PROGRESS_SUBTYPE: u16 = crate::kart_progress_native::PROGRESS_SUBTYPE;
/// ORIGINAL: subtype of the race configuration block.
pub const SET_RACE_CONFIG_SUBTYPE: u16 = 34;

/// ORIGINAL: inner header size for the 33 family (`u16 opcode | u16 subtype`).
pub const INNER_HEADER_LEN: usize = 4;
/// ORIGINAL: the `33/9` payload is `u64 session | u32 | u32`.
pub const REGISTER_PAYLOAD_LEN: usize = 16;
/// ORIGINAL: the `33/2` payload is `u64 session | u32`.
pub const PLAYER_STATE_PAYLOAD_LEN: usize = 12;
/// ORIGINAL: every counter payload (`33/1`, `33/3`, `33/4`) is one `i32`.
pub const COUNTER_PAYLOAD_LEN: usize = 4;
/// ORIGINAL: the captured `33/34` body is 84 bytes = 21 little-endian words.
pub const RACE_CONFIG_WORDS: usize = 21;
/// ORIGINAL: the captured `33/5` body is 85 bytes.
pub const CLIENT_DATA_PAYLOAD_LEN: usize = 85;

/// ORIGINAL captured `33/1` value on the register exchange (countdown/start state).
pub const CAPTURED_GAME_STATE_START: i32 = 3;
/// ORIGINAL captured `33/1` value in the later bundle (racer stream state).
pub const CAPTURED_GAME_STATE_STREAMING: i32 = 4;

/// RECONSTRUCTED bound: a speedway entry never lists more than this many vehicles.
pub const MAX_ENTRY_VEHICLES: usize = 8;
/// RECONSTRUCTED bound: a race never has more than this many racers.
pub const MAX_RACERS: usize = 16;
/// RECONSTRUCTED bound: a body asset name never exceeds this many UTF-8 bytes.
pub const MAX_ASSET_NAME_BYTES: usize = 120;
/// RECONSTRUCTED bound: an attribute block never exceeds this many bytes.
pub const MAX_ATTRIBUTE_BLOCK_BYTES: usize = 64 * 1024;

/// GATE: off by default. Only an explicit opt-in enables the entry exchange; while disabled every
/// frame is refused before it is even parsed and no reply is produced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryGate {
    #[default]
    Disabled,
    Enabled,
}

/// The recovered entry exchange step. `Waiting` also means "a context is open and no register has
/// arrived yet"; the initial handler (or one after `close`) reports `Closed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPhase {
    Waiting,
    Started,
    Joined,
    Streaming,
    Closed,
}

/// ORIGINAL `33/9` payload with the two unrecovered words kept positional.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterVehicleRacePlayer {
    pub session: u64,
    /// Captured `0x2a6ac44a`; meaning unrecovered, carried verbatim.
    pub word_8: u32,
    /// Captured `1`; meaning unrecovered, carried verbatim.
    pub word_12: u32,
}

/// ORIGINAL `33/2` payload; the trailing word is a client counter, captured as `2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerState {
    pub session: u64,
    pub counter: u32,
}

/// ORIGINAL `33/34` body: the captured 21 wire words in order. Field *semantics* beyond the wire
/// layout are unrecovered, so the words stay positional instead of being named.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceConfig {
    pub words: Vec<u32>,
}

impl RaceConfig {
    /// `u32` words in wire order, little-endian.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.words.len() * 4);
        for word in &self.words {
            out.extend(word.to_le_bytes());
        }
        out
    }
}

/// ORIGINAL `33/11` body: `u32 name byte count | name | f32 | attribute block`. The `f32`
/// immediately after the name is `400.0` in all six captured frames and its meaning is unrecovered;
/// the remaining attribute bytes are preserved verbatim.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProxiedVehicle {
    pub body_asset: String,
    pub first_attribute: f32,
    pub attribute_block: Vec<u8>,
}

impl ProxiedVehicle {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.body_asset.len() + self.attribute_block.len());
        out.extend((self.body_asset.len() as u32).to_le_bytes());
        out.extend(self.body_asset.as_bytes());
        out.extend(self.first_attribute.to_le_bytes());
        out.extend(&self.attribute_block);
        out
    }
}

/// Lead/private-content input for one speedway entry. Every captured-derived block arrives from
/// content instead of being invented here.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceEntryContent {
    pub race_config: RaceConfig,
    pub entry_vehicles: Vec<ProxiedVehicle>,
    /// Verbatim `33/5` body (85 bytes in the capture).
    pub client_data: Vec<u8>,
    /// One `33/14` body per racer, in emission order.
    pub progress: Vec<ProgressPayload>,
    pub player_count: u32,
    pub npc_count: u32,
}

/// One open speedway entry. The lead builds this when the player enters the speedway; `session` is
/// the owned GUID the client must echo back in `33/9` and `33/2`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceEntryContext {
    pub session: u64,
    pub instance_id: u32,
    pub encounter_id: u32,
    /// Complete op31 scene fields, including the rotation whose `w` must survive as captured.
    pub scene: ZoneTransition,
}

/// Typed refusal: nothing was sent and no handler state moved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryRefusal {
    /// The exchange is off by default.
    GateDisabled,
    /// No speedway entry is open for this session.
    NoOpenContext,
    /// The register/join names a session GUID this entry does not own (spoofed or foreign).
    ForeignSession { offered: u64, owned: u64 },
    /// A replay of a request whose phase has already been consumed, or a retired session.
    StaleSession { phase: EntryPhase },
    /// The request arrived in a phase that cannot accept it yet.
    InvalidPhase { phase: EntryPhase },
    /// Client control or unhandled subtypes (`6`, `7`, `17`, ...) stay inert.
    UnsupportedSubtype { subtype: u16 },
}

/// Typed failure: malformed wire input, or content/context outside its bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryError {
    FrameTooShort {
        len: usize,
        needed: usize,
    },
    UnsupportedOpcode {
        opcode: u16,
    },
    PayloadLength {
        subtype: u16,
        expected: usize,
        actual: usize,
    },
    /// `on_race_ready` without an open entry.
    NoContext,
    /// Opening a second entry before the current one is closed.
    AlreadyOpen {
        phase: EntryPhase,
    },
    ZeroSession,
    ZeroInstance,
    ZeroEncounter,
    /// The registered zone codec refused the context scene.
    SceneRefused {
        detail: String,
    },
    VehicleCount {
        count: usize,
        max: usize,
    },
    VehicleName {
        index: usize,
        bytes: usize,
    },
    VehicleAttribute {
        index: usize,
    },
    AttributeBlock {
        index: usize,
        bytes: usize,
    },
    ConfigWords {
        words: usize,
        expected: usize,
    },
    ClientData {
        bytes: usize,
        expected: usize,
    },
    ProgressCount {
        count: usize,
        max: usize,
    },
    RacerGuid {
        index: usize,
    },
    DuplicateRacerGuid {
        guid: u64,
    },
    RacerSlot {
        index: usize,
        slot: i32,
    },
    Counter {
        field: &'static str,
        value: u32,
    },
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryError::FrameTooShort { len, needed } => {
                write!(f, "inner frame needs {needed} bytes, {len} present")
            }
            EntryError::UnsupportedOpcode { opcode } => {
                write!(f, "inner opcode {opcode} is not the vehicle-race family")
            }
            EntryError::PayloadLength {
                subtype,
                expected,
                actual,
            } => write!(
                f,
                "subtype {subtype} payload must be {expected} bytes, got {actual}"
            ),
            EntryError::NoContext => write!(f, "no speedway entry is open"),
            EntryError::AlreadyOpen { phase } => {
                write!(f, "an entry is already open in phase {phase:?}")
            }
            EntryError::ZeroSession => write!(f, "session GUID must be non-zero"),
            EntryError::ZeroInstance => write!(f, "instance id must be non-zero"),
            EntryError::ZeroEncounter => write!(f, "encounter id must be non-zero"),
            EntryError::SceneRefused { detail } => {
                write!(f, "scene frame refused by the zone codec: {detail}")
            }
            EntryError::VehicleCount { count, max } => {
                write!(f, "entry needs 1..={max} vehicles, got {count}")
            }
            EntryError::VehicleName { index, bytes } => write!(
                f,
                "vehicle {index} asset name is {bytes} bytes and outside 1..={MAX_ASSET_NAME_BYTES}"
            ),
            EntryError::VehicleAttribute { index } => {
                write!(f, "vehicle {index} first attribute is not finite")
            }
            EntryError::AttributeBlock { index, bytes } => write!(
                f,
                "vehicle {index} attribute block is {bytes} bytes and outside 1..={MAX_ATTRIBUTE_BLOCK_BYTES}"
            ),
            EntryError::ConfigWords { words, expected } => {
                write!(f, "race config needs exactly {expected} words, got {words}")
            }
            EntryError::ClientData { bytes, expected } => {
                write!(f, "client data must be {expected} bytes, got {bytes}")
            }
            EntryError::ProgressCount { count, max } => {
                write!(f, "progress needs 1..={max} racers, got {count}")
            }
            EntryError::RacerGuid { index } => write!(f, "racer {index} GUID must be non-zero"),
            EntryError::DuplicateRacerGuid { guid } => {
                write!(f, "racer GUID {guid} appears more than once")
            }
            EntryError::RacerSlot { index, slot } => {
                write!(f, "racer {index} slot {slot} is outside 0..{MAX_RACERS}")
            }
            EntryError::Counter { field, value } => {
                write!(f, "{field} {value} exceeds the {MAX_RACERS}-racer bound")
            }
        }
    }
}

impl std::error::Error for EntryError {}

impl From<EntryError> for io::Error {
    fn from(error: EntryError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }
}

impl fmt::Display for EntryRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryRefusal::GateDisabled => write!(f, "kart entry gate is disabled"),
            EntryRefusal::NoOpenContext => write!(f, "no speedway entry is open"),
            EntryRefusal::ForeignSession { offered, owned } => {
                write!(f, "session {offered:016x} is not the owned {owned:016x}")
            }
            EntryRefusal::StaleSession { phase } => {
                write!(f, "session is stale in phase {phase:?}")
            }
            EntryRefusal::InvalidPhase { phase } => {
                write!(f, "request cannot be accepted in phase {phase:?}")
            }
            EntryRefusal::UnsupportedSubtype { subtype } => {
                write!(f, "subtype {subtype} is not part of the entry exchange")
            }
        }
    }
}

/// One handler answer: ordered inner frames for the gateway's tunnel wrapper, or a typed refusal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryOutcome {
    Replies(Vec<Vec<u8>>),
    Refused(EntryRefusal),
}

/// Decodes the inner 33 frame (`u16 opcode | u16 subtype | payload`).
pub fn decode_inner(inner: &[u8]) -> Result<(u16, &[u8]), EntryError> {
    if inner.len() < INNER_HEADER_LEN {
        return Err(EntryError::FrameTooShort {
            len: inner.len(),
            needed: INNER_HEADER_LEN,
        });
    }
    let opcode = u16::from_le_bytes([inner[0], inner[1]]);
    if opcode != VEHICLE_RACE_INNER_OPCODE {
        return Err(EntryError::UnsupportedOpcode { opcode });
    }
    Ok((
        u16::from_le_bytes([inner[2], inner[3]]),
        &inner[INNER_HEADER_LEN..],
    ))
}

/// Encodes one inner 33 frame. No tunnel envelope: the gateway wraps replies itself.
pub fn encode_inner(subtype: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(INNER_HEADER_LEN + payload.len());
    out.extend(VEHICLE_RACE_INNER_OPCODE.to_le_bytes());
    out.extend(subtype.to_le_bytes());
    out.extend(payload);
    out
}

fn require_payload(subtype: u16, payload: &[u8], expected: usize) -> Result<(), EntryError> {
    if payload.len() != expected {
        return Err(EntryError::PayloadLength {
            subtype,
            expected,
            actual: payload.len(),
        });
    }
    Ok(())
}

/// Decodes the `33/9` payload (`u64 session | u32 | u32`).
pub fn decode_register(payload: &[u8]) -> Result<RegisterVehicleRacePlayer, EntryError> {
    require_payload(REGISTER_SUBTYPE, payload, REGISTER_PAYLOAD_LEN)?;
    Ok(RegisterVehicleRacePlayer {
        session: u64::from_le_bytes(payload[0..8].try_into().unwrap()),
        word_8: u32::from_le_bytes(payload[8..12].try_into().unwrap()),
        word_12: u32::from_le_bytes(payload[12..16].try_into().unwrap()),
    })
}

/// Decodes the `33/2` payload (`u64 session | u32`).
pub fn decode_player_state(payload: &[u8]) -> Result<PlayerState, EntryError> {
    require_payload(PLAYER_STATE_SUBTYPE, payload, PLAYER_STATE_PAYLOAD_LEN)?;
    Ok(PlayerState {
        session: u64::from_le_bytes(payload[0..8].try_into().unwrap()),
        counter: u32::from_le_bytes(payload[8..12].try_into().unwrap()),
    })
}

fn validate_content(content: &RaceEntryContent) -> Result<(), EntryError> {
    if content.race_config.words.len() != RACE_CONFIG_WORDS {
        return Err(EntryError::ConfigWords {
            words: content.race_config.words.len(),
            expected: RACE_CONFIG_WORDS,
        });
    }
    if content.client_data.len() != CLIENT_DATA_PAYLOAD_LEN {
        return Err(EntryError::ClientData {
            bytes: content.client_data.len(),
            expected: CLIENT_DATA_PAYLOAD_LEN,
        });
    }
    if content.entry_vehicles.is_empty() || content.entry_vehicles.len() > MAX_ENTRY_VEHICLES {
        return Err(EntryError::VehicleCount {
            count: content.entry_vehicles.len(),
            max: MAX_ENTRY_VEHICLES,
        });
    }
    for (index, vehicle) in content.entry_vehicles.iter().enumerate() {
        let bytes = vehicle.body_asset.len();
        if bytes == 0 || bytes > MAX_ASSET_NAME_BYTES {
            return Err(EntryError::VehicleName { index, bytes });
        }
        if !vehicle.first_attribute.is_finite() {
            return Err(EntryError::VehicleAttribute { index });
        }
        let bytes = vehicle.attribute_block.len();
        if bytes == 0 || bytes > MAX_ATTRIBUTE_BLOCK_BYTES {
            return Err(EntryError::AttributeBlock { index, bytes });
        }
    }
    if content.progress.is_empty() || content.progress.len() > MAX_RACERS {
        return Err(EntryError::ProgressCount {
            count: content.progress.len(),
            max: MAX_RACERS,
        });
    }
    for (index, racer) in content.progress.iter().enumerate() {
        if racer.racer_guid == 0 {
            return Err(EntryError::RacerGuid { index });
        }
        if racer.slot < 0 || racer.slot as usize >= MAX_RACERS {
            return Err(EntryError::RacerSlot {
                index,
                slot: racer.slot,
            });
        }
        if content.progress[..index]
            .iter()
            .any(|other| other.racer_guid == racer.racer_guid)
        {
            return Err(EntryError::DuplicateRacerGuid {
                guid: racer.racer_guid,
            });
        }
    }
    for (field, value) in [
        ("player_count", content.player_count),
        ("npc_count", content.npc_count),
    ] {
        if value as usize > MAX_RACERS {
            return Err(EntryError::Counter { field, value });
        }
    }
    Ok(())
}

/// The bounded entry handler. One open speedway entry at a time; the gate defaults off.
#[derive(Clone, Debug)]
pub struct KartEntryHandler {
    content: RaceEntryContent,
    gate: EntryGate,
    phase: EntryPhase,
    context: Option<RaceEntryContext>,
    retired_session: Option<u64>,
}

impl KartEntryHandler {
    /// Validates the content once; every later frame check reuses the same bounds.
    pub fn new(content: RaceEntryContent) -> Result<Self, EntryError> {
        validate_content(&content)?;
        Ok(Self {
            content,
            gate: EntryGate::Disabled,
            phase: EntryPhase::Closed,
            context: None,
            retired_session: None,
        })
    }

    pub fn set_gate(&mut self, gate: EntryGate) {
        self.gate = gate;
    }

    pub fn gate(&self) -> EntryGate {
        self.gate
    }

    pub fn phase(&self) -> EntryPhase {
        self.phase
    }

    /// The owned session GUID while an entry is open, or the retired one after `close`.
    pub fn session(&self) -> Option<u64> {
        self.context
            .as_ref()
            .map(|context| context.session)
            .or(self.retired_session)
    }

    /// The owned instance id while an entry is open.
    pub fn instance_id(&self) -> Option<u32> {
        self.context.as_ref().map(|context| context.instance_id)
    }

    /// The owned encounter id while an entry is open.
    pub fn encounter_id(&self) -> Option<u32> {
        self.context.as_ref().map(|context| context.encounter_id)
    }

    /// The typed op31 scene context. Send it through `crate::zone_transition::encode`, never through
    /// the encounter writer (which forces rotation `w = 1.0`).
    pub fn scene(&self) -> Option<&ZoneTransition> {
        self.context.as_ref().map(|context| &context.scene)
    }

    /// The complete op31 frame (envelope included) exactly as the registered zone codec writes it.
    /// This is not an inner reply: the gateway must not wrap it a second time.
    pub fn scene_frame(&self) -> io::Result<Vec<u8>> {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no speedway entry is open"))?;
        zone_transition::encode(&context.scene)
    }

    /// Opens one speedway entry. `AlreadyOpen` until the previous entry is `close`d.
    pub fn open(&mut self, context: RaceEntryContext) -> Result<(), EntryError> {
        if self.phase != EntryPhase::Closed {
            return Err(EntryError::AlreadyOpen { phase: self.phase });
        }
        if context.session == 0 {
            return Err(EntryError::ZeroSession);
        }
        if context.instance_id == 0 {
            return Err(EntryError::ZeroInstance);
        }
        if context.encounter_id == 0 {
            return Err(EntryError::ZeroEncounter);
        }
        // Reuse the registered zone codec as the scene validator (bounds, finite floats, strings).
        zone_transition::encode(&context.scene).map_err(|error| EntryError::SceneRefused {
            detail: error.to_string(),
        })?;
        self.retired_session = None;
        self.context = Some(context);
        self.phase = EntryPhase::Waiting;
        Ok(())
    }

    /// Retires the open entry. A later register for the retired session is `StaleSession`; any other
    /// session without an open entry is `NoOpenContext`.
    pub fn close(&mut self) {
        self.retired_session = self.context.take().map(|context| context.session);
        self.phase = EntryPhase::Closed;
    }

    /// Entry point for the gateway: inner bytes in, ordered inner replies or a typed refusal out.
    pub fn handle_inner(&mut self, inner: &[u8]) -> Result<EntryOutcome, EntryError> {
        if self.gate == EntryGate::Disabled {
            return Ok(EntryOutcome::Refused(EntryRefusal::GateDisabled));
        }
        let (subtype, payload) = decode_inner(inner)?;
        match subtype {
            REGISTER_SUBTYPE => {
                let register = decode_register(payload)?;
                self.on_register(&register)
            }
            PLAYER_STATE_SUBTYPE => {
                let state = decode_player_state(payload)?;
                self.on_player_state(&state)
            }
            other => Ok(EntryOutcome::Refused(EntryRefusal::UnsupportedSubtype {
                subtype: other,
            })),
        }
    }

    /// `33/9`: the owned session may start exactly one entry, answering with the captured order
    /// `33/34`, one `33/11` per entry vehicle, `33/5`, then one `33/14` per racer.
    pub fn on_register(
        &mut self,
        register: &RegisterVehicleRacePlayer,
    ) -> Result<EntryOutcome, EntryError> {
        if let Err(refusal) = self.own_session(register.session) {
            return Ok(EntryOutcome::Refused(refusal));
        }
        match self.phase {
            EntryPhase::Waiting => {}
            phase => {
                return Ok(EntryOutcome::Refused(EntryRefusal::StaleSession { phase }));
            }
        }

        let mut replies = Vec::with_capacity(3 + self.content.entry_vehicles.len());
        replies.push(encode_inner(
            SET_RACE_CONFIG_SUBTYPE,
            &self.content.race_config.encode(),
        ));
        for vehicle in &self.content.entry_vehicles {
            replies.push(encode_inner(CREATE_VEHICLE_SUBTYPE, &vehicle.encode()));
        }
        let mut client_data = self.content.client_data.clone();
        // Original reader FUN_00b9fb10: the final two dwords echo the
        // local player's 33/9 registration words. Keep the fixed track
        // transform and slot; do not replay another client's registration.
        client_data[77..81].copy_from_slice(&register.word_8.to_le_bytes());
        client_data[81..85].copy_from_slice(&register.word_12.to_le_bytes());
        replies.push(encode_inner(CLIENT_DATA_SUBTYPE, &client_data));
        for racer in &self.content.progress {
            replies.push(encode_inner(
                PROGRESS_SUBTYPE,
                &kart_progress_native::encode_progress(racer),
            ));
        }
        self.phase = EntryPhase::Started;
        Ok(EntryOutcome::Replies(replies))
    }

    /// `33/2`: the owned session confirms the join; answer `33/3` (player count) and `33/1` (the
    /// captured start state). A replay is stale and changes nothing.
    pub fn on_player_state(&mut self, state: &PlayerState) -> Result<EntryOutcome, EntryError> {
        if let Err(refusal) = self.own_session(state.session) {
            return Ok(EntryOutcome::Refused(refusal));
        }
        match self.phase {
            EntryPhase::Started => {}
            EntryPhase::Joined | EntryPhase::Streaming => {
                return Ok(EntryOutcome::Refused(EntryRefusal::StaleSession {
                    phase: self.phase,
                }));
            }
            phase => return Ok(EntryOutcome::Refused(EntryRefusal::InvalidPhase { phase })),
        }
        self.phase = EntryPhase::Joined;
        Ok(EntryOutcome::Replies(vec![
            encode_inner(
                PLAYER_COUNT_SUBTYPE,
                &self.content.player_count.to_le_bytes(),
            ),
            encode_inner(GAME_STATE_SUBTYPE, &CAPTURED_GAME_STATE_START.to_le_bytes()),
        ]))
    }

    /// The later captured bundle carries only two recovered transitions for this slice: `33/1`
    /// (streaming state) and `33/4` (npc count). The repeated `33/5`/`33/14` payloads differ from the
    /// entry pair and belong to the live update stream, so they are not re-authored here.
    pub fn on_race_ready(&mut self) -> Result<EntryOutcome, EntryError> {
        if self.context.is_none() {
            return Err(EntryError::NoContext);
        }
        if self.phase != EntryPhase::Joined {
            return Ok(EntryOutcome::Refused(EntryRefusal::InvalidPhase {
                phase: self.phase,
            }));
        }
        self.phase = EntryPhase::Streaming;
        Ok(EntryOutcome::Replies(vec![
            encode_inner(
                GAME_STATE_SUBTYPE,
                &CAPTURED_GAME_STATE_STREAMING.to_le_bytes(),
            ),
            encode_inner(NPC_COUNT_SUBTYPE, &self.content.npc_count.to_le_bytes()),
        ]))
    }

    /// Resolves the owned session or the matching refusal. Spoofed input is reported, never silent.
    fn own_session(&self, offered: u64) -> Result<(), EntryRefusal> {
        match self.context.as_ref() {
            Some(context) if context.session == offered => Ok(()),
            Some(context) => Err(EntryRefusal::ForeignSession {
                offered,
                owned: context.session,
            }),
            None => match self.retired_session {
                Some(retired) if retired == offered => {
                    Err(EntryRefusal::StaleSession { phase: self.phase })
                }
                _ => Err(EntryRefusal::NoOpenContext),
            },
        }
    }
}
