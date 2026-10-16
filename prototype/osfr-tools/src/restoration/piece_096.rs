//! Work item 096: typed world initialization writers.
//!
//! The gateway currently replays 46 captured emulator messages exactly as received.
//! This module replaces that dependency one writer at a time: each typed writer decodes
//! the preserved message into named fields and re-encodes the identical bytes, so the
//! lead can build the login/ready stream from values instead of captured files without
//! changing a single wire byte.
//!
//! Recovered facts (recovered emulator source, preserved outside Git):
//! * `lab/decompiled/server/Gateway/Gateway.Login/LoginManager.cs` —
//!   `HandleLoginRequest`/`HandlePacketClientIsReady` fix the message order and
//!   `SendTunneledClientPacket` fixes the tunnel frame (outer opcode 5, boolean `true`,
//!   little-endian `u32` payload length, payload). `SendLoginReply`,
//!   `SendInitializationParameters`, `SendZoneDetails`, `SendClientGameSettings`,
//!   `SendAnnouncementData`, `SendPacketLoadWelcomeScreen`,
//!   `SendPacketZoneDoneSendingInitialData` and `SendPacketMOTD` fix the field order of
//!   their messages; `SendPacketLoadWelcomeScreen`, the two inline literals after
//!   `SendClientUpdatePacketAddEffectTag` and the ability literal are opaque blobs with
//!   no field structure in the source.
//! * `lab/decompiled/server/Gateway/Gateway.Player/PlayerCode.cs` —
//!   `SendPlayerUpdateItemDefinitions` fixes the 41-field item row and its nested stat
//!   and ability lists; `SendSelfToClient` frames a JSON-derived payload;
//!   `SendEncounterOverworldCombat` and `SendClientUpdatePacketDoneSendingPreloadCharacters`
//!   fix their field order.
//! * `lab/decompiled/server/Gateway/Gateway.Player/UpdateStats.cs` —
//!   `SendUpdatePacketUpdateStat` writes sub-opcode 7, the player GUID, an `i32` count and
//!   that many `(i32, i32, f32)` entries.
//!
//! Reconstructed here: the Rust field names for values the recovered writer passes as
//! literals, and the reading of the update-stat count as an entry count. This module is
//! not wired into the gateway and does not claim native acceptance; the proof is the
//! unchanged-writer oracle in `tests/piece_096.rs` plus the work-item checker.
//!
//! Lead integration step: register `pub mod piece_096;` under `src/restoration/mod.rs`,
//! then either verify each captured slot with [`verify_captured`] during gateway load or
//! build the typed messages with runtime values and drop the corresponding file from the
//! replay list. Slots without a writer keep replaying captured bytes. Gateway already has a
//! narrower `wrapped`/`initial_zone` pair; [`TypedWriter::ZoneDetails`] and [`frame`] are the
//! checked forms that should replace it so two zone writers cannot drift apart.

use std::fmt;

/// Tunnel outer opcode for `TunnelAppPacketServerToClient`.
pub const TUNNEL_OUTER: u16 = 5;
/// Upper bound on one decoded application message, matching the gateway limit.
pub const MAX_MESSAGE: usize = 1_048_576;
/// Length of the preserved login/ready initialization sequence.
pub const INIT_SLOT_COUNT: usize = 46;

const MAX_ITEMS: usize = 8_192;
const MAX_ANNOUNCEMENTS: usize = 1_024;
const MAX_ITEM_CHILDREN: usize = 4_096;
const MAX_TEXT: usize = 65_536;

/// A recovered emulator function whose message is fully typed here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedWriter {
    EncounterOverworldCombat,
    LoginReply,
    InitializationParameters,
    ZoneDetails,
    ClientGameSettings,
    ItemDefinitions,
    AnnouncementData,
    SelfToClient,
    ZoneDoneSendingInitialData,
    DoneSendingPreloadCharacters,
    UpdateStat,
    Motd,
}

impl TypedWriter {
    /// Every typed writer, in initialization order.
    pub const ALL: [TypedWriter; 12] = [
        TypedWriter::EncounterOverworldCombat,
        TypedWriter::LoginReply,
        TypedWriter::InitializationParameters,
        TypedWriter::ZoneDetails,
        TypedWriter::ClientGameSettings,
        TypedWriter::ItemDefinitions,
        TypedWriter::AnnouncementData,
        TypedWriter::SelfToClient,
        TypedWriter::ZoneDoneSendingInitialData,
        TypedWriter::DoneSendingPreloadCharacters,
        TypedWriter::UpdateStat,
        TypedWriter::Motd,
    ];

    /// Slot in the preserved initialization sequence.
    pub const fn slot(self) -> usize {
        match self {
            TypedWriter::EncounterOverworldCombat => 0,
            TypedWriter::LoginReply => 1,
            TypedWriter::InitializationParameters => 2,
            TypedWriter::ZoneDetails => 3,
            TypedWriter::ClientGameSettings => 4,
            TypedWriter::ItemDefinitions => 5,
            TypedWriter::AnnouncementData => 6,
            TypedWriter::SelfToClient => 7,
            TypedWriter::ZoneDoneSendingInitialData => 42,
            TypedWriter::DoneSendingPreloadCharacters => 43,
            TypedWriter::UpdateStat => 44,
            TypedWriter::Motd => 45,
        }
    }

    /// Application opcode the recovered writer passes to `SOEWriter`.
    pub const fn opcode(self) -> u16 {
        match self {
            TypedWriter::EncounterOverworldCombat => 41,
            TypedWriter::LoginReply => 2,
            TypedWriter::InitializationParameters => 166,
            TypedWriter::ZoneDetails => 43,
            TypedWriter::ClientGameSettings => 144,
            TypedWriter::ItemDefinitions => 35,
            TypedWriter::AnnouncementData => 193,
            TypedWriter::SelfToClient => 12,
            TypedWriter::ZoneDoneSendingInitialData => 14,
            TypedWriter::DoneSendingPreloadCharacters => 38,
            TypedWriter::UpdateStat => 38,
            TypedWriter::Motd => 87,
        }
    }

    /// Recovered function name, used in reports and panic messages.
    pub fn name(self) -> &'static str {
        match self {
            TypedWriter::EncounterOverworldCombat => "PlayerCode.SendEncounterOverworldCombat",
            TypedWriter::LoginReply => "LoginManager.SendLoginReply",
            TypedWriter::InitializationParameters => "LoginManager.SendInitializationParameters",
            TypedWriter::ZoneDetails => "LoginManager.SendZoneDetails",
            TypedWriter::ClientGameSettings => "LoginManager.SendClientGameSettings",
            TypedWriter::ItemDefinitions => "PlayerCode.SendPlayerUpdateItemDefinitions",
            TypedWriter::AnnouncementData => "LoginManager.SendAnnouncementData",
            TypedWriter::SelfToClient => "PlayerCode.SendSelfToClient",
            TypedWriter::ZoneDoneSendingInitialData => {
                "LoginManager.SendPacketZoneDoneSendingInitialData"
            }
            TypedWriter::DoneSendingPreloadCharacters => {
                "PlayerCode.SendClientUpdatePacketDoneSendingPreloadCharacters"
            }
            TypedWriter::UpdateStat => "UpdateStats.SendUpdatePacketUpdateStat",
            TypedWriter::Motd => "LoginManager.SendPacketMOTD",
        }
    }

    /// Resolve an opcode that identifies exactly one typed writer. Ops 35 and 38 appear
    /// in more than one schema, so they need the slot instead.
    pub fn for_opcode(opcode: u16) -> Option<TypedWriter> {
        if matches!(opcode, 35 | 38) {
            return None;
        }
        TypedWriter::ALL
            .iter()
            .copied()
            .find(|w| w.opcode() == opcode)
    }
}

/// One position of the preserved initialization sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub opcode: u16,
    pub writer: Option<TypedWriter>,
}

const fn typed(slot: TypedWriter) -> Slot {
    Slot {
        opcode: slot.opcode(),
        writer: Some(slot),
    }
}

const fn captured(opcode: u16) -> Slot {
    Slot {
        opcode,
        writer: None,
    }
}

/// The captured sequence, asserted against the preserved manifest by the checker.
pub const INIT_SLOTS: [Slot; INIT_SLOT_COUNT] = [
    typed(TypedWriter::EncounterOverworldCombat),
    typed(TypedWriter::LoginReply),
    typed(TypedWriter::InitializationParameters),
    typed(TypedWriter::ZoneDetails),
    typed(TypedWriter::ClientGameSettings),
    typed(TypedWriter::ItemDefinitions),
    typed(TypedWriter::AnnouncementData),
    typed(TypedWriter::SelfToClient),
    captured(67),
    captured(57),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(35),
    captured(38),
    captured(38),
    captured(36),
    captured(38),
    captured(38),
    captured(209),
    captured(209),
    captured(165),
    captured(93),
    typed(TypedWriter::ZoneDoneSendingInitialData),
    typed(TypedWriter::DoneSendingPreloadCharacters),
    typed(TypedWriter::UpdateStat),
    typed(TypedWriter::Motd),
];

/// Typed writer for a sequence slot, or `None` while that slot still replays captured bytes.
pub fn writer_for(slot: usize) -> Option<TypedWriter> {
    INIT_SLOTS.get(slot).and_then(|slot| slot.writer)
}

/// The recovered emitter of a still-captured slot. Function attribution comes from the
/// recovered call list and the recovered per-function message counts; identifying which
/// NPC each op35 entity-add row describes still needs the NPC catalogue.
pub fn captured_writer(slot: usize) -> Option<&'static str> {
    let name = match slot {
        8 => "ChatSystem.SendQuickChatPacket",
        9 => "Map.SendPacketPointOfInterestDefinitionReply",
        10..=15 => "NPCSystem.SendPlayerUpdatePacketAddNpc",
        16..=29 => "NPCSystem.SendWarpStones",
        30..=32 => "NPCSystem.SendWarpWatchers",
        33 => "UpdateStats.SendClientUpdatePacketHitpoints",
        34 => "UpdateStats.SendClientUpdatePacketMana",
        35 => "UpdateStats.SendAbilityPacketSetDefinition (inline literal)",
        36 => "UpdateStats.SendClientUpdatePacketAddEffectTag",
        37 => "UpdateStats.SendClientUpdatePacketAddEffectTag (second inline literal)",
        38 => "LoginManager.HandlePacketClientIsReady inline literal (opcode 209)",
        39 => "LoginManager.HandlePacketClientIsReady inline literal (opcode 209)",
        40 => "LoginManager.HandlePacketClientIsReady inline literal (opcode 165)",
        41 => "LoginManager.SendPacketLoadWelcomeScreen (inline literal)",
        _ => return None,
    };
    Some(name)
}

/// Failure of decoding, encoding validation or unchanged-writer verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitError {
    /// The slot has no typed writer yet and must keep replaying captured bytes.
    NotTypedSlot(usize),
    TunnelOuter(u16),
    TunnelFlag(u8),
    LengthMismatch {
        declared: usize,
        actual: usize,
    },
    TrailingBytes(usize),
    Truncated(&'static str),
    OpcodeMismatch {
        expected: u16,
        actual: u16,
    },
    InvalidBoolean {
        field: &'static str,
        value: u8,
    },
    TooManyRows {
        field: &'static str,
        count: usize,
    },
    TextTooLong {
        field: &'static str,
        length: usize,
    },
    NonAsciiText(&'static str),
    NonFinite(&'static str),
    MessageTooLarge(usize),
    /// The typed writer did not reproduce the captured bytes at this offset.
    ByteMismatch {
        offset: usize,
    },
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitError::NotTypedSlot(slot) => write!(f, "initialization slot {slot} is not typed"),
            InitError::TunnelOuter(outer) => {
                write!(f, "expected tunnel outer {TUNNEL_OUTER}, found {outer}")
            }
            InitError::TunnelFlag(value) => write!(f, "invalid tunnel boolean {value}"),
            InitError::LengthMismatch { declared, actual } => {
                write!(f, "declared payload length {declared} != {actual}")
            }
            InitError::TrailingBytes(count) => write!(f, "{count} trailing bytes"),
            InitError::Truncated(field) => write!(f, "truncated {field}"),
            InitError::OpcodeMismatch { expected, actual } => {
                write!(f, "expected opcode {expected}, found {actual}")
            }
            InitError::InvalidBoolean { field, value } => {
                write!(f, "invalid boolean {value} in {field}")
            }
            InitError::TooManyRows { field, count } => {
                write!(f, "{field} count {count} exceeds the bound")
            }
            InitError::TextTooLong { field, length } => {
                write!(f, "{field} length {length} exceeds the bound")
            }
            InitError::NonAsciiText(field) => write!(f, "{field} is not ASCII"),
            InitError::NonFinite(field) => write!(f, "{field} is not finite"),
            InitError::MessageTooLarge(size) => write!(f, "message size {size} exceeds the bound"),
            InitError::ByteMismatch { offset } => {
                write!(
                    f,
                    "typed writer differs from the capture at offset {offset}"
                )
            }
        }
    }
}

impl std::error::Error for InitError {}

/// Result of re-encoding one captured message through its typed writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriterVector {
    pub slot: usize,
    pub writer: TypedWriter,
    pub opcode: u16,
    pub length: usize,
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, size: usize, field: &'static str) -> Result<&'a [u8], InitError> {
        let end = self
            .offset
            .checked_add(size)
            .ok_or(InitError::Truncated(field))?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(InitError::Truncated(field))?;
        self.offset = end;
        Ok(bytes)
    }

    fn u8(&mut self, field: &'static str) -> Result<u8, InitError> {
        Ok(self.take(1, field)?[0])
    }

    fn bool(&mut self, field: &'static str) -> Result<bool, InitError> {
        match self.u8(field)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(InitError::InvalidBoolean { field, value }),
        }
    }

    fn u16(&mut self, field: &'static str) -> Result<u16, InitError> {
        Ok(u16::from_le_bytes(self.take(2, field)?.try_into().unwrap()))
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, InitError> {
        Ok(u32::from_le_bytes(self.take(4, field)?.try_into().unwrap()))
    }

    fn i32(&mut self, field: &'static str) -> Result<i32, InitError> {
        Ok(i32::from_le_bytes(self.take(4, field)?.try_into().unwrap()))
    }

    fn i64(&mut self, field: &'static str) -> Result<i64, InitError> {
        Ok(i64::from_le_bytes(self.take(8, field)?.try_into().unwrap()))
    }

    fn f32(&mut self, field: &'static str) -> Result<f32, InitError> {
        let value = f32::from_le_bytes(self.take(4, field)?.try_into().unwrap());
        if value.is_finite() {
            Ok(value)
        } else {
            Err(InitError::NonFinite(field))
        }
    }

    /// ASCII string with a `u32` byte length, matching the recovered `AddASCIIString`.
    fn text(&mut self, field: &'static str) -> Result<String, InitError> {
        let length = self.u32(field)? as usize;
        if length > MAX_TEXT {
            return Err(InitError::TextTooLong { field, length });
        }
        let bytes = self.take(length, field)?;
        if !bytes.is_ascii() {
            return Err(InitError::NonAsciiText(field));
        }
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    /// NUL-terminated ASCII string, matching the recovered `AddNullTerminatedString`.
    fn null_terminated(&mut self, field: &'static str) -> Result<String, InitError> {
        let rest = self.take(self.bytes.len() - self.offset, field)?;
        let end = rest
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(InitError::Truncated(field))?;
        if rest[end + 1..].is_empty() {
            let bytes = &rest[..end];
            if !bytes.is_ascii() {
                return Err(InitError::NonAsciiText(field));
            }
            Ok(String::from_utf8_lossy(bytes).into_owned())
        } else {
            Err(InitError::TrailingBytes(rest.len() - end - 1))
        }
    }

    fn end(&self) -> Result<(), InitError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(InitError::TrailingBytes(self.bytes.len() - self.offset))
        }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }
}

fn encode_text(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn encode_null_terminated(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

/// Wrap a typed payload in the recovered tunnel frame.
pub fn frame(payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 7);
    out.extend_from_slice(&TUNNEL_OUTER.to_le_bytes());
    out.push(1);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

#[derive(Clone, Debug, PartialEq)]
pub struct EncounterOverworldCombat {
    pub encounter_id: u16,
    pub unknown_a: u32,
    pub unknown_b: u32,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoginReply {
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InitializationParameters {
    pub environment: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZoneDetails {
    pub name: String,
    pub zone_type: u32,
    pub tutorial: bool,
    pub unknown2: bool,
    pub sky: String,
    pub is_in_arena: bool,
    pub id: u32,
    pub geometry_id: u32,
    pub is_in_starting_social_zone: bool,
    pub unknown6: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientGameSettings {
    pub values: [u32; 3],
    pub enabled: bool,
    pub scale: f32,
}

/// One `ClientItemStatDefinition` row: key plus four fields written as
/// `Unknown/Unknown2/Unknown3/Unknown4` in the recovered writer.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemStatDefinition {
    pub key: i32,
    pub unknown: i32,
    pub unknown2: i32,
    pub unknown3: i32,
    pub unknown4: bool,
}

/// One `ItemAbilityEntry` row, four unnamed recovered fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAbilityEntry {
    pub unknown: i32,
    pub unknown2: i32,
    pub unknown3: i32,
    pub unknown4: i32,
}

/// One `ClientItemDefinition` row in the exact order of
/// `PlayerCode.SendPlayerUpdateItemDefinitions`.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemDefinition {
    pub id: i32,
    pub name_id: i32,
    pub description_id: i32,
    pub icon_id: i32,
    pub icon_tint_id: i32,
    pub tint: i32,
    pub unknown5: i32,
    pub cost: i32,
    pub class: i32,
    pub profile_override: i32,
    pub slot: i32,
    pub no_trade: bool,
    pub no_sale: bool,
    pub model_name: String,
    pub texture_alias: String,
    pub gender_usage: i32,
    pub item_type: i32,
    pub category_id: i32,
    pub members_only: bool,
    pub non_minigame: bool,
    pub weapon_trail_effect_id: i32,
    pub composite_effect_id: i32,
    pub power_rating: i32,
    pub min_profile_rank: i32,
    pub rarity: i32,
    pub activatable_ability_id: i32,
    pub passive_ability_id: i32,
    pub single_use: bool,
    pub max_stack_size: i32,
    pub is_tintable: bool,
    pub tint_alias: String,
    pub force_disable_preview: bool,
    pub member_discount: i32,
    pub vip_rank_required: i32,
    pub race_set_id: i32,
    pub client_equip_req_set_id: i32,
    pub resell_value: i32,
    pub unknown36: i32,
    pub unknown37: i32,
    pub unknown38: i32,
    pub stats: Vec<ItemStatDefinition>,
    pub abilities: Vec<ItemAbilityEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ItemDefinitions {
    /// Inner opcode the recovered writer passes as `AddHostUInt16` (37 for login).
    pub inner_opcode: u16,
    pub items: Vec<ItemDefinition>,
}

/// One announcement row. The six leading and three tailing numbers are literals in the
/// recovered writer, so no historical names exist for them.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnouncementRow {
    pub numbers: [u32; 6],
    pub action: String,
    pub parameter: String,
    pub tail: [u32; 3],
    pub extra: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnnouncementData {
    pub revision: u8,
    pub rows: Vec<AnnouncementRow>,
}

/// `ClientPcData.ReadFromJSON` output inside its recovered frame. Only the envelope is
/// typed; the JSON-derived body stays opaque evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SelfToClient {
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZoneDoneSendingInitialData;

#[derive(Clone, Debug, PartialEq)]
pub struct DoneSendingPreloadCharacters {
    pub sub_opcode: u16,
    pub flag: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateStatEntry {
    pub stat_id: i32,
    pub unknown: i32,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateStat {
    pub sub_opcode: u16,
    pub guid: i64,
    pub entries: Vec<UpdateStatEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Motd {
    pub title: String,
    pub body: String,
}

/// One typed initialization message.
#[derive(Clone, Debug, PartialEq)]
pub enum InitMessage {
    EncounterOverworldCombat(EncounterOverworldCombat),
    LoginReply(LoginReply),
    InitializationParameters(InitializationParameters),
    ZoneDetails(ZoneDetails),
    ClientGameSettings(ClientGameSettings),
    ItemDefinitions(ItemDefinitions),
    AnnouncementData(AnnouncementData),
    SelfToClient(SelfToClient),
    ZoneDoneSendingInitialData(ZoneDoneSendingInitialData),
    DoneSendingPreloadCharacters(DoneSendingPreloadCharacters),
    UpdateStat(UpdateStat),
    Motd(Motd),
}

impl InitMessage {
    pub fn writer(&self) -> TypedWriter {
        match self {
            InitMessage::EncounterOverworldCombat(_) => TypedWriter::EncounterOverworldCombat,
            InitMessage::LoginReply(_) => TypedWriter::LoginReply,
            InitMessage::InitializationParameters(_) => TypedWriter::InitializationParameters,
            InitMessage::ZoneDetails(_) => TypedWriter::ZoneDetails,
            InitMessage::ClientGameSettings(_) => TypedWriter::ClientGameSettings,
            InitMessage::ItemDefinitions(_) => TypedWriter::ItemDefinitions,
            InitMessage::AnnouncementData(_) => TypedWriter::AnnouncementData,
            InitMessage::SelfToClient(_) => TypedWriter::SelfToClient,
            InitMessage::ZoneDoneSendingInitialData(_) => TypedWriter::ZoneDoneSendingInitialData,
            InitMessage::DoneSendingPreloadCharacters(_) => {
                TypedWriter::DoneSendingPreloadCharacters
            }
            InitMessage::UpdateStat(_) => TypedWriter::UpdateStat,
            InitMessage::Motd(_) => TypedWriter::Motd,
        }
    }

    pub fn opcode(&self) -> u16 {
        self.writer().opcode()
    }

    /// Opcode plus body, the payload the tunnel frame carries.
    pub fn encode_payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.opcode().to_le_bytes());
        match self {
            InitMessage::EncounterOverworldCombat(message) => {
                out.extend_from_slice(&message.encounter_id.to_le_bytes());
                out.extend_from_slice(&message.unknown_a.to_le_bytes());
                out.extend_from_slice(&message.unknown_b.to_le_bytes());
                out.push(u8::from(message.enabled));
            }
            InitMessage::LoginReply(message) => out.push(u8::from(message.accepted)),
            InitMessage::InitializationParameters(message) => {
                encode_null_terminated(&mut out, &message.environment)
            }
            InitMessage::ZoneDetails(message) => {
                encode_text(&mut out, &message.name);
                out.extend_from_slice(&message.zone_type.to_le_bytes());
                out.push(u8::from(message.tutorial));
                out.push(u8::from(message.unknown2));
                encode_text(&mut out, &message.sky);
                out.push(u8::from(message.is_in_arena));
                out.extend_from_slice(&message.id.to_le_bytes());
                out.extend_from_slice(&message.geometry_id.to_le_bytes());
                out.push(u8::from(message.is_in_starting_social_zone));
                out.push(u8::from(message.unknown6));
            }
            InitMessage::ClientGameSettings(message) => {
                for value in message.values {
                    out.extend_from_slice(&value.to_le_bytes());
                }
                out.push(u8::from(message.enabled));
                out.extend_from_slice(&message.scale.to_le_bytes());
            }
            InitMessage::ItemDefinitions(message) => {
                let mut inner = Vec::new();
                inner.extend_from_slice(&(message.items.len() as i32).to_le_bytes());
                for item in &message.items {
                    for value in [
                        item.id,
                        item.name_id,
                        item.description_id,
                        item.icon_id,
                        item.icon_tint_id,
                        item.tint,
                        item.unknown5,
                        item.cost,
                        item.class,
                        item.profile_override,
                        item.slot,
                    ] {
                        inner.extend_from_slice(&value.to_le_bytes());
                    }
                    inner.push(u8::from(item.no_trade));
                    inner.push(u8::from(item.no_sale));
                    encode_text(&mut inner, &item.model_name);
                    encode_text(&mut inner, &item.texture_alias);
                    for value in [item.gender_usage, item.item_type, item.category_id] {
                        inner.extend_from_slice(&value.to_le_bytes());
                    }
                    inner.push(u8::from(item.members_only));
                    inner.push(u8::from(item.non_minigame));
                    for value in [
                        item.weapon_trail_effect_id,
                        item.composite_effect_id,
                        item.power_rating,
                        item.min_profile_rank,
                        item.rarity,
                        item.activatable_ability_id,
                        item.passive_ability_id,
                    ] {
                        inner.extend_from_slice(&value.to_le_bytes());
                    }
                    inner.push(u8::from(item.single_use));
                    inner.extend_from_slice(&item.max_stack_size.to_le_bytes());
                    inner.push(u8::from(item.is_tintable));
                    encode_text(&mut inner, &item.tint_alias);
                    inner.push(u8::from(item.force_disable_preview));
                    for value in [
                        item.member_discount,
                        item.vip_rank_required,
                        item.race_set_id,
                        item.client_equip_req_set_id,
                        item.resell_value,
                        item.unknown36,
                        item.unknown37,
                        item.unknown38,
                    ] {
                        inner.extend_from_slice(&value.to_le_bytes());
                    }
                    inner.extend_from_slice(&(item.stats.len() as i32).to_le_bytes());
                    for stat in &item.stats {
                        inner.extend_from_slice(&stat.key.to_le_bytes());
                        inner.extend_from_slice(&stat.unknown.to_le_bytes());
                        inner.extend_from_slice(&stat.unknown2.to_le_bytes());
                        inner.extend_from_slice(&stat.unknown3.to_le_bytes());
                        inner.push(u8::from(stat.unknown4));
                    }
                    inner.extend_from_slice(&(item.abilities.len() as i32).to_le_bytes());
                    for ability in &item.abilities {
                        for value in [
                            ability.unknown,
                            ability.unknown2,
                            ability.unknown3,
                            ability.unknown4,
                        ] {
                            inner.extend_from_slice(&value.to_le_bytes());
                        }
                    }
                }
                out.extend_from_slice(&message.inner_opcode.to_le_bytes());
                out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
                out.extend_from_slice(&inner);
            }
            InitMessage::AnnouncementData(message) => {
                out.push(message.revision);
                out.extend_from_slice(&(message.rows.len() as u32).to_le_bytes());
                for row in &message.rows {
                    for value in row.numbers {
                        out.extend_from_slice(&value.to_le_bytes());
                    }
                    encode_text(&mut out, &row.action);
                    encode_text(&mut out, &row.parameter);
                    for value in row.tail {
                        out.extend_from_slice(&value.to_le_bytes());
                    }
                    encode_text(&mut out, &row.extra);
                }
            }
            InitMessage::SelfToClient(message) => {
                out.extend_from_slice(&(message.payload.len() as u32).to_le_bytes());
                out.extend_from_slice(&message.payload);
            }
            InitMessage::ZoneDoneSendingInitialData(_) => {}
            InitMessage::DoneSendingPreloadCharacters(message) => {
                out.extend_from_slice(&message.sub_opcode.to_le_bytes());
                out.push(u8::from(message.flag));
            }
            InitMessage::UpdateStat(message) => {
                out.extend_from_slice(&message.sub_opcode.to_le_bytes());
                out.extend_from_slice(&message.guid.to_le_bytes());
                out.extend_from_slice(&(message.entries.len() as i32).to_le_bytes());
                for entry in &message.entries {
                    out.extend_from_slice(&entry.stat_id.to_le_bytes());
                    out.extend_from_slice(&entry.unknown.to_le_bytes());
                    out.extend_from_slice(&entry.value.to_le_bytes());
                }
            }
            InitMessage::Motd(message) => {
                encode_text(&mut out, &message.title);
                encode_text(&mut out, &message.body);
            }
        }
        out
    }

    /// Tunnel-framed bytes, byte-identical to what the recovered writer sent.
    pub fn encode(&self) -> Vec<u8> {
        frame(&self.encode_payload())
    }
}

/// Split a tunnel frame into its payload after checking the recovered header.
pub fn split_frame(frame: &[u8]) -> Result<&[u8], InitError> {
    if frame.len() < 7 {
        return Err(InitError::Truncated("tunnel"));
    }
    let outer = u16::from_le_bytes(frame[..2].try_into().unwrap());
    if outer != TUNNEL_OUTER {
        return Err(InitError::TunnelOuter(outer));
    }
    match frame[2] {
        0 | 1 => {}
        value => return Err(InitError::TunnelFlag(value)),
    }
    let declared = u32::from_le_bytes(frame[3..7].try_into().unwrap()) as usize;
    let actual = frame.len() - 7;
    if declared != actual {
        return Err(InitError::LengthMismatch { declared, actual });
    }
    Ok(&frame[7..])
}

fn decode_items(reader: &mut Reader<'_>) -> Result<ItemDefinitions, InitError> {
    let inner_opcode = reader.u16("item definitions inner opcode")?;
    let length = reader.u32("item definitions length")? as usize;
    if length > MAX_MESSAGE {
        return Err(InitError::MessageTooLarge(length));
    }
    let raw = reader.take(length, "item definitions payload")?;
    let mut inner = Reader::new(raw);
    let count = inner.i32("item count")?;
    if count < 0 || count as usize > MAX_ITEMS {
        return Err(InitError::TooManyRows {
            field: "item count",
            count: count.max(0) as usize,
        });
    }
    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let mut leading = [0i32; 11];
        for value in &mut leading {
            *value = inner.i32("item field")?;
        }
        let no_trade = inner.bool("item no-trade")?;
        let no_sale = inner.bool("item no-sale")?;
        let model_name = inner.text("item model name")?;
        let texture_alias = inner.text("item texture alias")?;
        let mut middle = [0i32; 3];
        for value in &mut middle {
            *value = inner.i32("item field")?;
        }
        let members_only = inner.bool("item members-only")?;
        let non_minigame = inner.bool("item non-minigame")?;
        let mut effects = [0i32; 7];
        for value in &mut effects {
            *value = inner.i32("item field")?;
        }
        let single_use = inner.bool("item single-use")?;
        let max_stack_size = inner.i32("item max stack size")?;
        let is_tintable = inner.bool("item is-tintable")?;
        let tint_alias = inner.text("item tint alias")?;
        let force_disable_preview = inner.bool("item force-disable-preview")?;
        let mut tail = [0i32; 8];
        for value in &mut tail {
            *value = inner.i32("item field")?;
        }
        let stats = decode_stats(&mut inner)?;
        let abilities = decode_abilities(&mut inner)?;
        items.push(ItemDefinition {
            id: leading[0],
            name_id: leading[1],
            description_id: leading[2],
            icon_id: leading[3],
            icon_tint_id: leading[4],
            tint: leading[5],
            unknown5: leading[6],
            cost: leading[7],
            class: leading[8],
            profile_override: leading[9],
            slot: leading[10],
            no_trade,
            no_sale,
            model_name,
            texture_alias,
            gender_usage: middle[0],
            item_type: middle[1],
            category_id: middle[2],
            members_only,
            non_minigame,
            weapon_trail_effect_id: effects[0],
            composite_effect_id: effects[1],
            power_rating: effects[2],
            min_profile_rank: effects[3],
            rarity: effects[4],
            activatable_ability_id: effects[5],
            passive_ability_id: effects[6],
            single_use,
            max_stack_size,
            is_tintable,
            tint_alias,
            force_disable_preview,
            member_discount: tail[0],
            vip_rank_required: tail[1],
            race_set_id: tail[2],
            client_equip_req_set_id: tail[3],
            resell_value: tail[4],
            unknown36: tail[5],
            unknown37: tail[6],
            unknown38: tail[7],
            stats,
            abilities,
        });
    }
    inner.end()?;
    Ok(ItemDefinitions {
        inner_opcode,
        items,
    })
}

fn decode_stats(reader: &mut Reader<'_>) -> Result<Vec<ItemStatDefinition>, InitError> {
    let count = reader.i32("item stat count")?;
    if count < 0 || count as usize > MAX_ITEM_CHILDREN {
        return Err(InitError::TooManyRows {
            field: "item stat count",
            count: count.max(0) as usize,
        });
    }
    let mut stats = Vec::with_capacity(count as usize);
    for _ in 0..count {
        stats.push(ItemStatDefinition {
            key: reader.i32("item stat key")?,
            unknown: reader.i32("item stat field")?,
            unknown2: reader.i32("item stat field")?,
            unknown3: reader.i32("item stat field")?,
            unknown4: reader.bool("item stat flag")?,
        });
    }
    Ok(stats)
}

fn decode_abilities(reader: &mut Reader<'_>) -> Result<Vec<ItemAbilityEntry>, InitError> {
    let count = reader.i32("item ability count")?;
    if count < 0 || count as usize > MAX_ITEM_CHILDREN {
        return Err(InitError::TooManyRows {
            field: "item ability count",
            count: count.max(0) as usize,
        });
    }
    let mut abilities = Vec::with_capacity(count as usize);
    for _ in 0..count {
        abilities.push(ItemAbilityEntry {
            unknown: reader.i32("item ability field")?,
            unknown2: reader.i32("item ability field")?,
            unknown3: reader.i32("item ability field")?,
            unknown4: reader.i32("item ability field")?,
        });
    }
    Ok(abilities)
}

/// Decode one typed payload (opcode plus body) for a sequence slot.
pub fn decode_payload(slot: usize, payload: &[u8]) -> Result<InitMessage, InitError> {
    let writer = writer_for(slot).ok_or(InitError::NotTypedSlot(slot))?;
    let mut reader = Reader::new(payload);
    let opcode = reader.u16("opcode")?;
    if opcode != writer.opcode() {
        return Err(InitError::OpcodeMismatch {
            expected: writer.opcode(),
            actual: opcode,
        });
    }
    let message = match writer {
        TypedWriter::EncounterOverworldCombat => {
            let message = EncounterOverworldCombat {
                encounter_id: reader.u16("encounter id")?,
                unknown_a: reader.u32("encounter field")?,
                unknown_b: reader.u32("encounter field")?,
                enabled: reader.bool("encounter enabled")?,
            };
            InitMessage::EncounterOverworldCombat(message)
        }
        TypedWriter::LoginReply => InitMessage::LoginReply(LoginReply {
            accepted: reader.bool("login reply")?,
        }),
        TypedWriter::InitializationParameters => {
            InitMessage::InitializationParameters(InitializationParameters {
                environment: reader.null_terminated("initialization environment")?,
            })
        }
        TypedWriter::ZoneDetails => {
            let message = ZoneDetails {
                name: reader.text("zone name")?,
                zone_type: reader.u32("zone type")?,
                tutorial: reader.bool("zone tutorial")?,
                unknown2: reader.bool("zone field")?,
                sky: reader.text("zone sky")?,
                is_in_arena: reader.bool("zone arena flag")?,
                id: reader.u32("zone id")?,
                geometry_id: reader.u32("zone geometry id")?,
                is_in_starting_social_zone: reader.bool("zone social flag")?,
                unknown6: reader.bool("zone field")?,
            };
            InitMessage::ZoneDetails(message)
        }
        TypedWriter::ClientGameSettings => {
            let mut values = [0u32; 3];
            for value in &mut values {
                *value = reader.u32("client game setting")?;
            }
            InitMessage::ClientGameSettings(ClientGameSettings {
                values,
                enabled: reader.bool("client game settings enabled")?,
                scale: reader.f32("client game settings scale")?,
            })
        }
        TypedWriter::ItemDefinitions => InitMessage::ItemDefinitions(decode_items(&mut reader)?),
        TypedWriter::AnnouncementData => {
            let revision = reader.u8("announcement revision")?;
            let count = reader.u32("announcement count")? as usize;
            if count > MAX_ANNOUNCEMENTS {
                return Err(InitError::TooManyRows {
                    field: "announcement count",
                    count,
                });
            }
            let mut rows = Vec::with_capacity(count);
            for _ in 0..count {
                let mut numbers = [0u32; 6];
                for value in &mut numbers {
                    *value = reader.u32("announcement field")?;
                }
                let action = reader.text("announcement action")?;
                let parameter = reader.text("announcement parameter")?;
                let mut tail = [0u32; 3];
                for value in &mut tail {
                    *value = reader.u32("announcement field")?;
                }
                let extra = reader.text("announcement text")?;
                rows.push(AnnouncementRow {
                    numbers,
                    action,
                    parameter,
                    tail,
                    extra,
                });
            }
            InitMessage::AnnouncementData(AnnouncementData { revision, rows })
        }
        TypedWriter::SelfToClient => {
            let length = reader.u32("self payload length")? as usize;
            if length > MAX_MESSAGE {
                return Err(InitError::MessageTooLarge(length));
            }
            InitMessage::SelfToClient(SelfToClient {
                payload: reader.take(length, "self payload")?.to_vec(),
            })
        }
        TypedWriter::ZoneDoneSendingInitialData => {
            InitMessage::ZoneDoneSendingInitialData(ZoneDoneSendingInitialData)
        }
        TypedWriter::DoneSendingPreloadCharacters => {
            InitMessage::DoneSendingPreloadCharacters(DoneSendingPreloadCharacters {
                sub_opcode: reader.u16("preload sub-opcode")?,
                flag: reader.bool("preload flag")?,
            })
        }
        TypedWriter::UpdateStat => {
            let sub_opcode = reader.u16("update stat sub-opcode")?;
            let guid = reader.i64("update stat guid")?;
            let count = reader.i32("update stat count")?;
            if count < 0 || count as usize > MAX_ITEM_CHILDREN {
                return Err(InitError::TooManyRows {
                    field: "update stat count",
                    count: count.max(0) as usize,
                });
            }
            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count {
                entries.push(UpdateStatEntry {
                    stat_id: reader.i32("update stat id")?,
                    unknown: reader.i32("update stat field")?,
                    value: reader.f32("update stat value")?,
                });
            }
            InitMessage::UpdateStat(UpdateStat {
                sub_opcode,
                guid,
                entries,
            })
        }
        TypedWriter::Motd => InitMessage::Motd(Motd {
            title: reader.text("motd title")?,
            body: reader.text("motd body")?,
        }),
    };
    if reader.remaining() != 0 {
        return Err(InitError::TrailingBytes(reader.remaining()));
    }
    Ok(message)
}

/// Decode one tunnel-framed message for a sequence slot.
pub fn decode(slot: usize, frame: &[u8]) -> Result<InitMessage, InitError> {
    decode_payload(slot, split_frame(frame)?)
}

/// Prove that the typed writer reproduces one captured message byte for byte.
pub fn verify_captured(slot: usize, captured: &[u8]) -> Result<WriterVector, InitError> {
    let message = decode(slot, captured)?;
    let encoded = message.encode();
    if encoded.len() != captured.len() {
        return Err(InitError::ByteMismatch {
            offset: encoded.len().min(captured.len()),
        });
    }
    if let Some(offset) = encoded
        .iter()
        .zip(captured.iter())
        .position(|(left, right)| left != right)
    {
        return Err(InitError::ByteMismatch { offset });
    }
    Ok(WriterVector {
        slot,
        writer: message.writer(),
        opcode: message.opcode(),
        length: encoded.len(),
    })
}
