//! Native Demo Derby packets and a bounded server-authoritative bout candidate.
//!
//! Client facts come from Build Guides/Gameplay/Demo Derby and the accepted native map:
//! the processor reader is 009f39b0, client senders include 009f1550 and 009f1680,
//! and field readers include 009ef570, 009ef5e0, 009f3800, 009ee850, 009ee9e0,
//! 009eeba0, 009eec60 and 009eeeb0. No OSFR packet capture is used here.
//!
//! All game rules in DerbyRules, NPC fill/keys, combat, pickup selection, timers, result
//! ordering and generic minigame defaults are reconstructed policy (R). F164 withholds rewards:
//! every collision is a client `34/16` report, so there is no server-verified outcome to pay from.
//! Unknown reader fields remain raw words/bytes. Native launch and gateway routing are external
//! integration responsibilities.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const DEMO_DERBY_TYPE_ID: i32 = 11;
pub const DEMO_DERBY_LOBBY_NAME_ID: i32 = 3053;
pub const DEMO_DERBY_INNER_OPCODE: u16 = 34;
pub const MAX_DERBY_DRIVERS: usize = 8;
pub const DERBY_ACTIVE_STATUS_BIT: u32 = 11;
const MAX_OPAQUE_BYTES: usize = 1024 * 1024;
const CREATE_VEHICLE_TAIL_LEN: usize = 69;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParticipantKey(pub u64);

impl ParticipantKey {
    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PacketDirection {
    ServerToClient,
    ClientToServer,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubtypeSpec {
    pub subtype: u16,
    pub direction: PacketDirection,
    pub packet_class: &'static str,
    pub client_evidence: &'static str,
}

/// Direction and client handler/sender provenance from the Demo Derby guide's Network table.
/// The body reader/writer addresses are recorded beside each message variant below.
pub const SUBTYPE_SPECS: [SubtypeSpec; 20] = [
    spec(
        1,
        PacketDirection::ServerToClient,
        "UpdateVehicleGameState",
        "reader 009ef570; handler 009f39b0",
    ),
    spec(
        2,
        PacketDirection::ClientToServer,
        "UpdateVehicleGamePlayerState",
        "writer 009f0a50; sender 009f1440",
    ),
    spec(
        3,
        PacketDirection::ServerToClient,
        "UpdatePlayerCount",
        "reader 009ef330; handler 009f39b0",
    ),
    spec(
        4,
        PacketDirection::ServerToClient,
        "UpdateNpcCount",
        "reader 009ef370; handler 009f39b0",
    ),
    spec(
        5,
        PacketDirection::Both,
        "UpdateVehicleDemolitionDerbyData",
        "reader 009ef5e0; writer 009f0ce0",
    ),
    spec(
        6,
        PacketDirection::ClientToServer,
        "RegisterVehicleDemolitionDerbyPlayer",
        "sender 009f1790; activation 009f1b80",
    ),
    spec(
        7,
        PacketDirection::Both,
        "VehicleDemolitionDerbyCountdownStatus",
        "reader 009eedb0; sender 009f20f0",
    ),
    spec(
        8,
        PacketDirection::ServerToClient,
        "DestroyProxiedVehicleDemolitionDerby",
        "handler 009f39b0 case 8",
    ),
    spec(
        9,
        PacketDirection::ServerToClient,
        "KickProxiedVehicleDemolitionDerby",
        "handler 009f39b0 case 9",
    ),
    spec(
        10,
        PacketDirection::ServerToClient,
        "CreateProxiedVehicleDemolitionDerby",
        "reader 009f3800; vehicle reader 009d7b50",
    ),
    spec(
        11,
        PacketDirection::ServerToClient,
        "UpdateVehicleDemolitionDerbyStateData",
        "reader 009ee850; handler 009f39b0",
    ),
    spec(
        12,
        PacketDirection::ServerToClient,
        "CreateProxiedVehicleDemolitionDerbyPickup",
        "reader 009ee9e0; handler 009f39b0",
    ),
    spec(
        13,
        PacketDirection::ServerToClient,
        "DestroyProxiedVehicleDemolitionDerbyPickup",
        "handler 009f39b0 case 13",
    ),
    spec(
        14,
        PacketDirection::ClientToServer,
        "PickupHitRequest",
        "sender 009f1550",
    ),
    spec(
        15,
        PacketDirection::ServerToClient,
        "PickupHitResponse",
        "reader 009eec60; handler 009f39b0",
    ),
    spec(
        16,
        PacketDirection::Both,
        "VehicleHit",
        "reader 009eeba0; sender 009f22c0 via 009f1680",
    ),
    spec(
        17,
        PacketDirection::ServerToClient,
        "ItemExpired",
        "reader 009ef4d0; handler 009f39b0",
    ),
    spec(
        18,
        PacketDirection::ServerToClient,
        "ResetProxiedVehicleDemolitionDerby",
        "reader 009ef520; handler 009f39b0",
    ),
    spec(
        19,
        PacketDirection::ServerToClient,
        "SetDerbyConfig",
        "reader 009ee190; handler 009f39b0",
    ),
    spec(
        20,
        PacketDirection::ServerToClient,
        "AnnouncerMessage",
        "reader 009eeeb0; handler 009f39b0",
    ),
];

const fn spec(
    subtype: u16,
    direction: PacketDirection,
    packet_class: &'static str,
    client_evidence: &'static str,
) -> SubtypeSpec {
    SubtypeSpec {
        subtype,
        direction,
        packet_class,
        client_evidence,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DerbyError {
    TooShort,
    WrongOpcode(u16),
    UnknownSubtypeLength { subtype: u16, length: usize },
    TrailingBytes,
    InvalidField(&'static str),
    InvalidPhase,
    InvalidIdentity,
}

impl fmt::Display for DerbyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => f.write_str("Demo Derby packet is truncated"),
            Self::WrongOpcode(opcode) => write!(f, "inner opcode {opcode} is not Demo Derby"),
            Self::UnknownSubtypeLength { subtype, length } => {
                write!(
                    f,
                    "Demo Derby subtype {subtype} has invalid body length {length}"
                )
            }
            Self::TrailingBytes => f.write_str("Demo Derby packet has trailing bytes"),
            Self::InvalidField(field) => write!(f, "invalid Demo Derby field: {field}"),
            Self::InvalidPhase => f.write_str("Demo Derby operation is invalid in this phase"),
            Self::InvalidIdentity => f.write_str("Demo Derby identity does not match the session"),
        }
    }
}

impl std::error::Error for DerbyError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u32)]
pub enum PickupKind {
    Health = 1,
    Boost = 2,
    Shield = 3,
    AttackUp = 4,
    Ultimate = 5,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleCreate {
    pub key: ParticipantKey,
    pub name: Vec<u8>,
    /// Complete variable-length ReadVehicleDefinition bytes; its interior remains opaque.
    pub definition: Vec<u8>,
    /// Four float4 blocks read by 009f3800. Their meanings are unknown.
    pub start_transform_bits: [u32; 16],
    pub unknown_flag: bool,
    pub unknown_value_bits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehiclePose {
    pub key: ParticipantKey,
    /// The reader at 009ef5e0 consumes the vehicle's 4x4 block.
    pub transform_bits: [u32; 16],
    /// Two float4 values; 00c546a0 and 00c546e0 are not semantically named.
    pub vector_a_bits: [u32; 4],
    pub vector_b_bits: [u32; 4],
    pub controls_bits: [u32; 3],
    /// Four bytes; the final flag has no recovered meaning.
    pub flags: [u8; 4],
    /// Client-side 64-bit value; time-versus-sequence meaning remains unknown.
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleState {
    pub key: ParticipantKey,
    /// Client displays this as health; semantic identification is a strong pseudocode inference.
    pub health: u32,
    /// Confirmed consumer: score field in the HUD and results list.
    pub score: u32,
    /// Two dwords with unknown consumers.
    pub unknown_values: [u32; 2],
    /// Client result damage is raw field E minus raw field F. Mapping these to dealt/received is R.
    pub damage_dealt: u32,
    pub damage_received: u32,
    /// Client turns each byte into a boolean and passes three effect slots to 00fe9650.
    pub flags: [u8; 3],
    /// Four checked float fields with unknown meaning.
    pub unknown_floats_bits: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyConfig {
    pub value_a_bits: u32,
    pub value_b: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyPickup {
    pub id: u32,
    pub definition_id: u32,
    pub kind: PickupKind,
    pub position_bits: [u32; 4],
    pub unknown_floats_bits: [u32; 2],
    /// Reader consumes three floats; their client-side meanings are not recovered.
    pub response_parameters_bits: [u32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerbyPickupBody {
    pub id: u32,
    pub definition_id: u32,
    pub position_bits: [u32; 4],
    pub unknown_floats_bits: [u32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VehicleHitReport {
    /// Sender writes its local vehicle key first and the contacted key second.
    pub reporter: ParticipantKey,
    pub other: ParticipantKey,
    pub impact_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DerbyMessage {
    GameState(i32),
    PlayerPhase {
        key: ParticipantKey,
        phase: i32,
    },
    PlayerCount(i32),
    NpcCount(i32),
    VehiclePose(VehiclePose),
    RegisterPlayer {
        key: ParticipantKey,
        unknown_key: u64,
    },
    CountdownStatus {
        key: ParticipantKey,
        count: u32,
    },
    DestroyVehicle(ParticipantKey),
    KickVehicle(ParticipantKey),
    CreateVehicle(VehicleCreate),
    VehicleState(VehicleState),
    CreatePickup(DerbyPickupBody),
    DestroyPickup(u32),
    PickupHitRequest {
        key: ParticipantKey,
        position_bits: [u32; 4],
    },
    PickupHitResponse {
        key: ParticipantKey,
        kind: u32,
        parameters_bits: [u32; 3],
    },
    VehicleHit(VehicleHitReport),
    ItemExpired(u32),
    ResetVehicle(ParticipantKey),
    SetConfig(DerbyConfig),
    Announcer {
        kind: u32,
        sub_kind: u32,
        first: ParticipantKey,
        second: ParticipantKey,
    },
    Unknown {
        subtype: u16,
        body: Vec<u8>,
    },
}

impl DerbyMessage {
    pub fn subtype(&self) -> u16 {
        match self {
            Self::GameState(_) => 1,
            Self::PlayerPhase { .. } => 2,
            Self::PlayerCount(_) => 3,
            Self::NpcCount(_) => 4,
            Self::VehiclePose(_) => 5,
            Self::RegisterPlayer { .. } => 6,
            Self::CountdownStatus { .. } => 7,
            Self::DestroyVehicle(_) => 8,
            Self::KickVehicle(_) => 9,
            Self::CreateVehicle(_) => 10,
            Self::VehicleState(_) => 11,
            Self::CreatePickup(_) => 12,
            Self::DestroyPickup(_) => 13,
            Self::PickupHitRequest { .. } => 14,
            Self::PickupHitResponse { .. } => 15,
            Self::VehicleHit(_) => 16,
            Self::ItemExpired(_) => 17,
            Self::ResetVehicle(_) => 18,
            Self::SetConfig(_) => 19,
            Self::Announcer { .. } => 20,
            Self::Unknown { subtype, .. } => *subtype,
        }
    }

    fn write_body(&self, out: &mut Vec<u8>) -> Result<(), DerbyError> {
        match self {
            Self::GameState(value) | Self::PlayerCount(value) | Self::NpcCount(value) => {
                put_i32(out, *value)
            }
            Self::PlayerPhase { key, phase } => {
                put_key(out, *key);
                put_i32(out, *phase);
            }
            Self::VehiclePose(pose) => {
                put_key(out, pose.key);
                put_words(out, &pose.transform_bits);
                put_words(out, &pose.vector_a_bits);
                put_words(out, &pose.vector_b_bits);
                put_words(out, &pose.controls_bits);
                out.extend(pose.flags);
                put_u64(out, pose.sequence);
            }
            Self::RegisterPlayer { key, unknown_key } => {
                put_key(out, *key);
                put_u64(out, *unknown_key);
            }
            Self::CountdownStatus { key, count } => {
                put_key(out, *key);
                put_u32(out, *count);
            }
            Self::DestroyVehicle(key) | Self::KickVehicle(key) | Self::ResetVehicle(key) => {
                put_key(out, *key)
            }
            Self::CreateVehicle(vehicle) => {
                if vehicle.name.len() > MAX_OPAQUE_BYTES
                    || vehicle.definition.len() > MAX_OPAQUE_BYTES
                {
                    return Err(DerbyError::InvalidField("vehicle create blob"));
                }
                put_key(out, vehicle.key);
                put_i32(
                    out,
                    i32::try_from(vehicle.name.len())
                        .map_err(|_| DerbyError::InvalidField("vehicle name length"))?,
                );
                out.extend(&vehicle.name);
                out.extend(&vehicle.definition);
                put_words(out, &vehicle.start_transform_bits);
                out.push(u8::from(vehicle.unknown_flag));
                put_u32(out, vehicle.unknown_value_bits);
            }
            Self::VehicleState(state) => {
                put_key(out, state.key);
                put_u32(out, state.health);
                put_u32(out, state.score);
                put_words(out, &state.unknown_values);
                put_u32(out, state.damage_dealt);
                put_u32(out, state.damage_received);
                out.extend(state.flags);
                put_words(out, &state.unknown_floats_bits);
            }
            Self::CreatePickup(pickup) => {
                put_u32(out, pickup.id);
                put_u32(out, pickup.definition_id);
                put_words(out, &pickup.position_bits);
                put_words(out, &pickup.unknown_floats_bits);
            }
            Self::DestroyPickup(id) | Self::ItemExpired(id) => put_u32(out, *id),
            Self::PickupHitRequest { key, position_bits } => {
                put_key(out, *key);
                put_words(out, position_bits);
            }
            Self::PickupHitResponse {
                key,
                kind,
                parameters_bits,
            } => {
                put_key(out, *key);
                put_u32(out, *kind);
                put_words(out, parameters_bits);
            }
            Self::VehicleHit(report) => {
                put_key(out, report.reporter);
                put_key(out, report.other);
                put_u32(out, report.impact_bits);
            }
            Self::SetConfig(config) => {
                put_u32(out, config.value_a_bits);
                put_u32(out, config.value_b);
            }
            Self::Announcer {
                kind,
                sub_kind,
                first,
                second,
            } => {
                put_u32(out, *kind);
                put_u32(out, *sub_kind);
                put_key(out, *first);
                put_key(out, *second);
            }
            Self::Unknown { subtype, body } => {
                if SUBTYPE_SPECS.iter().any(|spec| spec.subtype == *subtype)
                    || body.len() > MAX_OPAQUE_BYTES
                {
                    return Err(DerbyError::InvalidField("unknown subtype"));
                }
                out.extend(body);
            }
        }
        Ok(())
    }
}

/// Encode one inner message: u16 opcode 34, u16 subtype, then the exact client-reader body.
/// The outer SOE tunnel is owned by the transport layer.
pub fn encode_inner(message: &DerbyMessage) -> Result<Vec<u8>, DerbyError> {
    let mut body = Vec::new();
    message.write_body(&mut body)?;
    let mut packet = Vec::with_capacity(4 + body.len());
    put_u16(&mut packet, DEMO_DERBY_INNER_OPCODE);
    put_u16(&mut packet, message.subtype());
    packet.extend(body);
    Ok(packet)
}

/// Decode all twenty client packet classes. Unknown subtype numbers remain byte-preserved;
/// known-but-truncated bodies fail rather than borrowing capture lengths.
pub fn decode_inner(packet: &[u8]) -> Result<DerbyMessage, DerbyError> {
    if packet.len() < 4 {
        return Err(DerbyError::TooShort);
    }
    let opcode = u16::from_le_bytes([packet[0], packet[1]]);
    if opcode != DEMO_DERBY_INNER_OPCODE {
        return Err(DerbyError::WrongOpcode(opcode));
    }
    let subtype = u16::from_le_bytes([packet[2], packet[3]]);
    let mut reader = Reader::new(&packet[4..]);
    let message = match subtype {
        1 => DerbyMessage::GameState(reader.i32()?),
        2 => DerbyMessage::PlayerPhase {
            key: reader.key()?,
            phase: reader.i32()?,
        },
        3 => DerbyMessage::PlayerCount(reader.i32()?),
        4 => DerbyMessage::NpcCount(reader.i32()?),
        5 => DerbyMessage::VehiclePose(VehiclePose {
            key: reader.key()?,
            transform_bits: reader.words()?,
            vector_a_bits: reader.words()?,
            vector_b_bits: reader.words()?,
            controls_bits: reader.words()?,
            flags: reader.array()?,
            sequence: reader.u64()?,
        }),
        6 => DerbyMessage::RegisterPlayer {
            key: reader.key()?,
            unknown_key: reader.u64()?,
        },
        7 => DerbyMessage::CountdownStatus {
            key: reader.key()?,
            count: reader.u32()?,
        },
        8 => DerbyMessage::DestroyVehicle(reader.key()?),
        9 => DerbyMessage::KickVehicle(reader.key()?),
        10 => {
            let key = reader.key()?;
            let name_len = reader.i32()?;
            if name_len < 0 || name_len as usize > MAX_OPAQUE_BYTES {
                return Err(DerbyError::InvalidField("vehicle name length"));
            }
            let name = reader.bytes(name_len as usize)?.to_vec();
            if reader.remaining() < CREATE_VEHICLE_TAIL_LEN {
                return Err(DerbyError::UnknownSubtypeLength {
                    subtype,
                    length: packet.len() - 4,
                });
            }
            let definition = reader
                .bytes(reader.remaining() - CREATE_VEHICLE_TAIL_LEN)?
                .to_vec();
            DerbyMessage::CreateVehicle(VehicleCreate {
                key,
                name,
                definition,
                start_transform_bits: reader.words()?,
                unknown_flag: reader.u8()? != 0,
                unknown_value_bits: reader.u32()?,
            })
        }
        11 => DerbyMessage::VehicleState(VehicleState {
            key: reader.key()?,
            health: reader.u32()?,
            score: reader.u32()?,
            unknown_values: reader.words()?,
            damage_dealt: reader.u32()?,
            damage_received: reader.u32()?,
            flags: reader.array()?,
            unknown_floats_bits: reader.words()?,
        }),
        12 => DerbyMessage::CreatePickup(DerbyPickupBody {
            id: reader.u32()?,
            definition_id: reader.u32()?,
            position_bits: reader.words()?,
            unknown_floats_bits: reader.words()?,
        }),
        13 => DerbyMessage::DestroyPickup(reader.u32()?),
        14 => DerbyMessage::PickupHitRequest {
            key: reader.key()?,
            position_bits: reader.words()?,
        },
        15 => DerbyMessage::PickupHitResponse {
            key: reader.key()?,
            kind: reader.u32()?,
            parameters_bits: reader.words()?,
        },
        16 => DerbyMessage::VehicleHit(VehicleHitReport {
            reporter: reader.key()?,
            other: reader.key()?,
            impact_bits: reader.u32()?,
        }),
        17 => DerbyMessage::ItemExpired(reader.u32()?),
        18 => DerbyMessage::ResetVehicle(reader.key()?),
        19 => DerbyMessage::SetConfig(DerbyConfig {
            value_a_bits: reader.u32()?,
            value_b: reader.u32()?,
        }),
        20 => DerbyMessage::Announcer {
            kind: reader.u32()?,
            sub_kind: reader.u32()?,
            first: reader.key()?,
            second: reader.key()?,
        },
        _ => {
            return Ok(DerbyMessage::Unknown {
                subtype,
                body: reader.remaining_bytes().to_vec(),
            });
        }
    };
    reader.finish(subtype, packet.len() - 4)?;
    Ok(message)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyVehicleTemplate {
    pub name: Vec<u8>,
    /// Serialized ReadVehicleDefinition bytes; content owns the still-opaque client schema.
    pub definition: Vec<u8>,
    pub start_transform_bits: [u32; 16],
    pub unknown_flag: bool,
    pub unknown_value_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerbyRules {
    /// All numeric combat and scheduling values here are reconstructed policy (R).
    pub max_drivers: usize,
    pub start_health: u32,
    pub contact_damage_per_unit: f32,
    pub contact_cooldown_ms: u64,
    /// Maximum centre-to-centre distance for a reported vehicle contact (R).
    #[serde(default = "default_contact_radius_m")]
    pub contact_radius_m: f32,
    /// Largest reportable impact; larger values are not plausible contacts (R).
    #[serde(default = "default_max_impact")]
    pub max_impact: f32,
    /// A contact needs both drivers' latest poses to be this recent (R).
    #[serde(default = "default_max_pose_age_ms")]
    pub max_pose_age_ms: u64,
    /// Hard ceiling on one contact's resolved damage (R).
    #[serde(default = "default_max_damage_per_contact")]
    pub max_damage_per_contact: u32,
    /// An offered game that the client never starts is retired after this delay (R).
    #[serde(default = "default_lobby_timeout_ms")]
    pub lobby_timeout_ms: u64,
    pub pickup_radius: f32,
    pub health_pickup: u32,
    pub boost_pickup: f32,
    pub shield_duration_ms: u64,
    pub attack_duration_ms: u64,
    pub ultimate_duration_ms: u64,
    pub attack_multiplier: f32,
    pub ultimate_multiplier: f32,
    pub respawn_delay_ms: u64,
    pub results_duration_ms: u64,
    pub knockout_points: u32,
    pub leader_knockout_points: u32,
    pub knockout_point_loss: u32,
    pub score_target: Option<u32>,
    pub time_limit_ms: Option<u64>,
    /// Client reads one float and one dword at 009ee190; meanings are unknown.
    pub config: DerbyConfig,
}

fn default_contact_radius_m() -> f32 {
    5.0
}

fn default_max_impact() -> f32 {
    100.0
}

fn default_max_pose_age_ms() -> u64 {
    5_000
}

fn default_max_damage_per_contact() -> u32 {
    100
}

fn default_lobby_timeout_ms() -> u64 {
    120_000
}

impl Default for DerbyRules {
    fn default() -> Self {
        Self {
            max_drivers: MAX_DERBY_DRIVERS,
            start_health: 100,
            contact_damage_per_unit: 1.0,
            contact_cooldown_ms: 500,
            contact_radius_m: default_contact_radius_m(),
            max_impact: default_max_impact(),
            max_pose_age_ms: default_max_pose_age_ms(),
            max_damage_per_contact: default_max_damage_per_contact(),
            lobby_timeout_ms: default_lobby_timeout_ms(),
            pickup_radius: 3.0,
            health_pickup: 25,
            boost_pickup: 25.0,
            shield_duration_ms: 5_000,
            attack_duration_ms: 5_000,
            ultimate_duration_ms: 5_000,
            attack_multiplier: 2.0,
            ultimate_multiplier: 2.0,
            respawn_delay_ms: 4_000,
            results_duration_ms: 10_000,
            knockout_points: 2,
            leader_knockout_points: 3,
            knockout_point_loss: 1,
            score_target: None,
            time_limit_ms: None,
            config: DerbyConfig {
                value_a_bits: 0,
                value_b: 0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerbyPhase {
    Lobby,
    Starting,
    WaitingForState(i32),
    Countdown,
    Running,
    Results,
    CloseEnabled,
    Closed,
    Aborted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerbyAction {
    /// Participant keys are authenticated human recipients; bots have no network session.
    pub recipients: Vec<ParticipantKey>,
    pub packet: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerbyDriverState {
    pub key: ParticipantKey,
    pub human: bool,
    pub connected: bool,
    pub health: u32,
    pub score: u32,
    pub damage_dealt: u32,
    pub damage_received: u32,
    pub boost: f32,
    pub shield_until_ms: Option<u64>,
    pub attack_until_ms: Option<u64>,
    pub ultimate_until_ms: Option<u64>,
}

#[derive(Clone)]
struct Driver {
    state: DerbyDriverState,
    vehicle: DerbyVehicleTemplate,
    pose: Option<VehiclePose>,
    pose_at_ms: Option<u64>,
    respawn_at_ms: Option<u64>,
}

#[derive(Clone)]
pub struct DerbyService {
    instance_id: i32,
    game_id: i32,
    rules: DerbyRules,
    phase: DerbyPhase,
    drivers: BTreeMap<ParticipantKey, Driver>,
    humans: BTreeSet<ParticipantKey>,
    registered: BTreeSet<ParticipantKey>,
    phase_acks: BTreeSet<ParticipantKey>,
    countdown_acks: BTreeSet<ParticipantKey>,
    countdown_value: u32,
    pickups: BTreeMap<u32, DerbyPickup>,
    contact_times: BTreeMap<(ParticipantKey, ParticipantKey), u64>,
    started_at_ms: Option<u64>,
    results_until_ms: Option<u64>,
    lobby_deadline_ms: Option<u64>,
    closed: BTreeSet<ParticipantKey>,
}

impl DerbyService {
    pub fn new(
        instance_id: i32,
        game_id: i32,
        owner: ParticipantKey,
        vehicle: DerbyVehicleTemplate,
        rules: DerbyRules,
        pickups: Vec<DerbyPickup>,
    ) -> Result<Self, DerbyError> {
        if instance_id <= 0 || game_id <= 0 || owner.0 == 0 {
            return Err(DerbyError::InvalidIdentity);
        }
        if !(2..=MAX_DERBY_DRIVERS).contains(&rules.max_drivers)
            || rules.start_health == 0
            || !rules.contact_damage_per_unit.is_finite()
            || rules.contact_damage_per_unit <= 0.0
            || !rules.contact_radius_m.is_finite()
            || rules.contact_radius_m <= 0.0
            || rules.contact_radius_m > 1_000.0
            || !rules.max_impact.is_finite()
            || rules.max_impact <= 0.0
            || rules.max_impact > 1_000_000.0
            || !(1..=60_000).contains(&rules.max_pose_age_ms)
            || !(1..=1_000_000).contains(&rules.max_damage_per_contact)
            || !(1..=600_000).contains(&rules.lobby_timeout_ms)
            || !rules.pickup_radius.is_finite()
            || rules.pickup_radius <= 0.0
            || !rules.boost_pickup.is_finite()
            || !rules.attack_multiplier.is_finite()
            || !rules.ultimate_multiplier.is_finite()
            || rules.score_target == Some(0)
            || rules.time_limit_ms == Some(0)
            || rules.results_duration_ms == 0
        {
            return Err(DerbyError::InvalidField("reconstructed rules"));
        }
        if vehicle.definition.len() > MAX_OPAQUE_BYTES
            || vehicle.name.len() > MAX_OPAQUE_BYTES
            || vehicle
                .start_transform_bits
                .iter()
                .any(|bits| !f32::from_bits(*bits).is_finite())
        {
            return Err(DerbyError::InvalidField("vehicle template"));
        }
        let mut pickup_map = BTreeMap::new();
        for pickup in pickups {
            if pickup.id == 0
                || pickup.definition_id == 0
                || pickup
                    .position_bits
                    .iter()
                    .any(|bits| !f32::from_bits(*bits).is_finite())
                || pickup
                    .unknown_floats_bits
                    .iter()
                    .any(|bits| !f32::from_bits(*bits).is_finite())
                || pickup
                    .response_parameters_bits
                    .iter()
                    .any(|bits| !f32::from_bits(*bits).is_finite())
                || pickup_map.insert(pickup.id, pickup).is_some()
            {
                return Err(DerbyError::InvalidField("pickup content"));
            }
        }
        let mut drivers = BTreeMap::new();
        drivers.insert(
            owner,
            Driver {
                state: DerbyDriverState {
                    key: owner,
                    human: true,
                    connected: true,
                    health: rules.start_health,
                    score: 0,
                    damage_dealt: 0,
                    damage_received: 0,
                    boost: 0.0,
                    shield_until_ms: None,
                    attack_until_ms: None,
                    ultimate_until_ms: None,
                },
                vehicle,
                pose: None,
                pose_at_ms: None,
                respawn_at_ms: None,
            },
        );
        Ok(Self {
            instance_id,
            game_id,
            rules,
            phase: DerbyPhase::Lobby,
            drivers,
            humans: BTreeSet::from([owner]),
            registered: BTreeSet::new(),
            phase_acks: BTreeSet::new(),
            countdown_acks: BTreeSet::new(),
            countdown_value: 4,
            pickups: pickup_map,
            contact_times: BTreeMap::new(),
            started_at_ms: None,
            results_until_ms: None,
            lobby_deadline_ms: None,
            closed: BTreeSet::new(),
        })
    }

    pub fn phase(&self) -> DerbyPhase {
        self.phase
    }

    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }

    pub fn game_id(&self) -> i32 {
        self.game_id
    }

    pub fn set_lobby_deadline(&mut self, now_ms: u64) {
        self.lobby_deadline_ms = (self.phase == DerbyPhase::Lobby)
            .then(|| now_ms.saturating_add(self.rules.lobby_timeout_ms));
    }

    pub fn human_count(&self) -> usize {
        self.drivers
            .values()
            .filter(|driver| driver.state.human && driver.state.connected)
            .count()
    }

    pub fn npc_count(&self) -> usize {
        self.drivers
            .values()
            .filter(|driver| !driver.state.human)
            .count()
    }

    pub fn driver(&self, key: ParticipantKey) -> Option<&DerbyDriverState> {
        self.drivers.get(&key).map(|driver| &driver.state)
    }

    pub fn add_human(
        &mut self,
        key: ParticipantKey,
        vehicle: DerbyVehicleTemplate,
    ) -> Result<bool, DerbyError> {
        if self.phase != DerbyPhase::Lobby {
            return Err(DerbyError::InvalidPhase);
        }
        if key.0 == 0
            || vehicle.definition.len() > MAX_OPAQUE_BYTES
            || vehicle.name.len() > MAX_OPAQUE_BYTES
        {
            return Err(DerbyError::InvalidIdentity);
        }
        if self.drivers.contains_key(&key) {
            return Ok(false);
        }
        if self.humans.len() >= self.rules.max_drivers {
            return Ok(false);
        }
        self.drivers.insert(
            key,
            Driver {
                state: DerbyDriverState {
                    key,
                    human: true,
                    connected: true,
                    health: self.rules.start_health,
                    score: 0,
                    damage_dealt: 0,
                    damage_received: 0,
                    boost: 0.0,
                    shield_until_ms: None,
                    attack_until_ms: None,
                    ultimate_until_ms: None,
                },
                vehicle,
                pose: None,
                pose_at_ms: None,
                respawn_at_ms: None,
            },
        );
        self.humans.insert(key);
        Ok(true)
    }

    /// Build the common 39/16 native-game offer. Unnamed trailing defaults are (R), matching the
    /// shared minigame writer; no Flash launch is emitted for native type 11.
    pub fn offer_packet(&self) -> Result<Vec<u8>, DerbyError> {
        if self.phase != DerbyPhase::Lobby {
            return Err(DerbyError::InvalidPhase);
        }
        let mut packet = vec![39, 0, 16];
        put_i32(&mut packet, self.instance_id);
        put_i32(&mut packet, -1);
        put_i32(&mut packet, self.game_id);
        for value in [DEMO_DERBY_LOBBY_NAME_ID, 0, 0, 1, 0, DEMO_DERBY_TYPE_ID] {
            put_i32(&mut packet, value);
        }
        packet.push(0);
        packet.extend([0; 3 * 69]);
        put_i32(&mut packet, 0);
        packet.extend([1; 5]);
        put_i32(&mut packet, 0);
        put_i32(&mut packet, 1);
        packet.push(1);
        put_i32(&mut packet, self.game_id);
        packet.extend([0; 4]);
        put_i32(&mut packet, 0);
        put_i32(&mut packet, 0);
        Ok(packet)
    }

    /// A normal Go request is 39/5. The client type-11 status bit then arms processor activation
    /// (bit 11, SetActive 009f1b80); its player-update carrier is not established by the client.
    pub fn handle_minigame_packet(&mut self, packet: &[u8]) -> Result<Vec<u8>, DerbyError> {
        if packet.len() < 3 || packet[..2] != [39, 0] || packet[2] != 5 {
            return Ok(Vec::new());
        }
        if packet.len() != 15 {
            return Err(DerbyError::TooShort);
        }
        let instance_id = read_i32(packet, 3);
        let group_id = read_i32(packet, 7);
        let game_id = read_i32(packet, 11);
        if instance_id != self.instance_id || group_id != -1 || game_id != -1 {
            return Ok(Vec::new());
        }
        if self.phase != DerbyPhase::Lobby {
            return Ok(Vec::new());
        }
        self.phase = DerbyPhase::Starting;
        self.lobby_deadline_ms = None;
        let mut ack = vec![39, 0, 17];
        put_i32(&mut ack, self.instance_id);
        put_i32(&mut ack, -1);
        put_i32(&mut ack, self.game_id);
        Ok(ack)
    }

    /// Retire an unfinished match without a reward. `26/42` is the offer/results-panel close;
    /// `39/6` is the native mid-game quit and `39/7` cancels a pending game. The 39 records must
    /// name this live instance. Client CancelMiniGame `009bef50` calls `009bdc60`, which writes
    /// subtype 7 through `008a4420` as `[39, u16, i32, i32, i32]`. `None` is inert.
    pub fn cancel(
        &mut self,
        sender: ParticipantKey,
        packet: &[u8],
    ) -> Result<Option<Vec<u8>>, DerbyError> {
        if !self.is_connected_human(sender)
            || matches!(self.phase, DerbyPhase::Closed | DerbyPhase::Aborted)
        {
            return Ok(None);
        }
        let accepted = if packet == [26, 0, 42, 0] {
            true
        } else if packet.len() == 15
            && packet[..2] == [39, 0]
            && matches!(packet[2], 6 | 7)
            && read_i32(packet, 3) == self.instance_id
            && read_i32(packet, 7) == -1
            && read_i32(packet, 11) == -1
        {
            true
        } else {
            false
        };
        if !accepted {
            return Ok(None);
        }
        self.phase = DerbyPhase::Aborted;
        self.pickups.clear();
        Ok(Some(crate::minigame::state_remove_packet(
            self.instance_id,
            self.game_id,
        )))
    }

    /// Client RegisterVehicleDemolitionDerbyPlayer (34/6, sender 009f1790) binds its first key to
    /// the authenticated sender. The second u64 is retained by the codec but its meaning is unknown.
    pub fn handle_client(
        &mut self,
        sender: ParticipantKey,
        packet: &[u8],
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        let message = decode_inner(packet)?;
        match message {
            DerbyMessage::RegisterPlayer { key, .. } => {
                if self.phase != DerbyPhase::Starting
                    || key != sender
                    || !self.humans.contains(&sender)
                {
                    return Ok(Vec::new());
                }
                self.registered.insert(sender);
                if self.registered.len() == self.human_count() {
                    self.setup_match(now_ms)
                } else {
                    Ok(Vec::new())
                }
            }
            DerbyMessage::PlayerPhase { key, phase } if key == sender => {
                self.acknowledge(sender, phase, now_ms)
            }
            DerbyMessage::CountdownStatus { key, count } if key == sender => {
                self.countdown(sender, count, now_ms)
            }
            DerbyMessage::VehiclePose(pose) if pose.key == sender => {
                self.relay_pose(sender, pose, now_ms)
            }
            DerbyMessage::PickupHitRequest { key, position_bits } if key == sender => {
                self.pickup_hit(sender, position_bits, now_ms)
            }
            DerbyMessage::VehicleHit(report) if report.reporter == sender => {
                self.contact(sender, report, now_ms)
            }
            _ => Ok(Vec::new()),
        }
    }

    fn setup_match(&mut self, now_ms: u64) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Starting || self.human_count() == 0 {
            return Err(DerbyError::InvalidPhase);
        }
        self.fill_npcs(now_ms)?;
        let mut actions = Vec::new();
        self.push_broadcast(&mut actions, DerbyMessage::SetConfig(self.rules.config))?;
        self.push_broadcast(
            &mut actions,
            DerbyMessage::PlayerCount(self.human_count() as i32),
        )?;
        self.push_broadcast(
            &mut actions,
            DerbyMessage::NpcCount(self.npc_count() as i32),
        )?;
        for driver in self.drivers.values() {
            self.push_broadcast(
                &mut actions,
                DerbyMessage::CreateVehicle(VehicleCreate {
                    key: driver.state.key,
                    name: driver.vehicle.name.clone(),
                    definition: driver.vehicle.definition.clone(),
                    start_transform_bits: driver.vehicle.start_transform_bits,
                    unknown_flag: driver.vehicle.unknown_flag,
                    unknown_value_bits: driver.vehicle.unknown_value_bits,
                }),
            )?;
        }
        for pickup in self.pickups.values() {
            self.push_broadcast(
                &mut actions,
                DerbyMessage::CreatePickup(DerbyPickupBody {
                    id: pickup.id,
                    definition_id: pickup.definition_id,
                    position_bits: pickup.position_bits,
                    unknown_floats_bits: pickup.unknown_floats_bits,
                }),
            )?;
        }
        self.phase = DerbyPhase::WaitingForState(3);
        self.push_broadcast(&mut actions, DerbyMessage::GameState(3))?;
        Ok(actions)
    }

    fn fill_npcs(&mut self, now_ms: u64) -> Result<(), DerbyError> {
        // (R) Solo and short lobbies fill to eight drivers. The retail NPC AI/key allocator is unknown.
        let template = self.drivers.values().next().unwrap().vehicle.clone();
        let mut slot = 1_u32;
        while self.drivers.len() < self.rules.max_drivers {
            let raw = 0xd000_0000_0000_0000 | ((self.instance_id as u64) << 8) | slot as u64;
            let key = ParticipantKey(raw);
            if self.drivers.contains_key(&key) || key.0 == 0 {
                return Err(DerbyError::InvalidIdentity);
            }
            self.drivers.insert(
                key,
                Driver {
                    state: DerbyDriverState {
                        key,
                        human: false,
                        connected: true,
                        health: self.rules.start_health,
                        score: 0,
                        damage_dealt: 0,
                        damage_received: 0,
                        boost: 0.0,
                        shield_until_ms: None,
                        attack_until_ms: None,
                        ultimate_until_ms: None,
                    },
                    vehicle: template.clone(),
                    pose: Some(VehiclePose {
                        key,
                        transform_bits: template.start_transform_bits,
                        vector_a_bits: [0; 4],
                        vector_b_bits: [0; 4],
                        controls_bits: [0; 3],
                        flags: [0; 4],
                        sequence: 0,
                    }),
                    pose_at_ms: Some(now_ms),
                    respawn_at_ms: None,
                },
            );
            slot += 1;
        }
        Ok(())
    }

    fn acknowledge(
        &mut self,
        sender: ParticipantKey,
        phase: i32,
        _now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        if !self.is_connected_human(sender) {
            return Ok(Vec::new());
        }
        if phase == 2 {
            return Ok(Vec::new());
        }
        let DerbyPhase::WaitingForState(waiting) = self.phase else {
            return Ok(Vec::new());
        };
        if phase != waiting || !(3..=8).contains(&phase) {
            return Ok(Vec::new());
        }
        self.phase_acks.insert(sender);
        if self.phase_acks.len() != self.human_count() {
            return Ok(Vec::new());
        }
        self.phase_acks.clear();
        if waiting == 8 {
            self.phase = DerbyPhase::Countdown;
            self.countdown_value = 4;
            return self.countdown_actions(4);
        }
        let next = waiting + 1;
        self.phase = DerbyPhase::WaitingForState(next);
        self.broadcast_one(DerbyMessage::GameState(next))
    }

    fn countdown(
        &mut self,
        sender: ParticipantKey,
        count: u32,
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Countdown
            || !self.is_connected_human(sender)
            || count != self.countdown_value
        {
            return Ok(Vec::new());
        }
        self.countdown_acks.insert(sender);
        if self.countdown_acks.len() != self.human_count() {
            return Ok(Vec::new());
        }
        self.countdown_acks.clear();
        if count == 0 {
            self.phase = DerbyPhase::Running;
            self.started_at_ms = Some(now_ms);
            return self.broadcast_one(DerbyMessage::GameState(9));
        }
        self.countdown_value -= 1;
        self.countdown_actions(self.countdown_value)
    }

    fn countdown_actions(&self, count: u32) -> Result<Vec<DerbyAction>, DerbyError> {
        let mut actions = Vec::new();
        for key in self.connected_humans() {
            self.push_to(
                &mut actions,
                key,
                DerbyMessage::CountdownStatus { key, count },
            )?;
        }
        Ok(actions)
    }

    fn relay_pose(
        &mut self,
        sender: ParticipantKey,
        pose: VehiclePose,
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Running || !self.is_connected_human(sender) {
            return Ok(Vec::new());
        }
        if pose_position(&pose).is_none() {
            return Ok(Vec::new());
        }
        let driver = self.drivers.get_mut(&sender).unwrap();
        driver.pose = Some(pose);
        driver.pose_at_ms = Some(now_ms);
        let recipients: Vec<_> = self
            .connected_humans()
            .into_iter()
            .filter(|key| *key != sender)
            .collect();
        if recipients.is_empty() {
            return Ok(Vec::new());
        }
        Ok(vec![DerbyAction {
            recipients,
            packet: encode_inner(&DerbyMessage::VehiclePose(pose))?,
        }])
    }

    fn contact(
        &mut self,
        sender: ParticipantKey,
        report: VehicleHitReport,
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Running
            || report.reporter != sender
            || sender == report.other
            || !self.is_connected_human(sender)
            || !self.drivers.contains_key(&report.other)
        {
            return Ok(Vec::new());
        }
        let impact = f32::from_bits(report.impact_bits);
        if !impact.is_finite() || impact <= 0.0 || impact > self.rules.max_impact {
            return Ok(Vec::new());
        }
        let Some(attacker_position) = self.drivers[&sender].pose.as_ref().and_then(pose_position)
        else {
            return Ok(Vec::new());
        };
        let Some(victim_position) = self.drivers[&report.other]
            .pose
            .as_ref()
            .and_then(pose_position)
        else {
            return Ok(Vec::new());
        };
        let pose_is_fresh = |driver: &Driver| {
            driver.pose_at_ms.is_some_and(|at| {
                at <= now_ms && now_ms.saturating_sub(at) <= self.rules.max_pose_age_ms
            })
        };
        if !pose_is_fresh(&self.drivers[&sender]) || !pose_is_fresh(&self.drivers[&report.other]) {
            return Ok(Vec::new());
        }
        let distance_squared = (attacker_position[0] - victim_position[0]).powi(2)
            + (attacker_position[1] - victim_position[1]).powi(2)
            + (attacker_position[2] - victim_position[2]).powi(2);
        if !distance_squared.is_finite()
            || distance_squared > self.rules.contact_radius_m * self.rules.contact_radius_m
        {
            return Ok(Vec::new());
        }
        let attacker = self.drivers.get(&sender).unwrap();
        let multiplier = if attacker
            .state
            .ultimate_until_ms
            .is_some_and(|until| until > now_ms)
        {
            self.rules.ultimate_multiplier
        } else if attacker
            .state
            .attack_until_ms
            .is_some_and(|until| until > now_ms)
        {
            self.rules.attack_multiplier
        } else {
            1.0
        };
        let damage = (impact * self.rules.contact_damage_per_unit * multiplier)
            .ceil()
            .clamp(1.0, u32::MAX as f32) as u32;
        if damage > self.rules.max_damage_per_contact {
            return Ok(Vec::new());
        }
        let victim_was_alive = self.drivers[&report.other].state.health > 0;
        if !victim_was_alive {
            return Ok(Vec::new());
        }
        let pair = ordered_pair(sender, report.other);
        if self.contact_times.get(&pair).is_some_and(|previous| {
            now_ms.saturating_sub(*previous) < self.rules.contact_cooldown_ms
        }) {
            return Ok(Vec::new());
        }
        self.contact_times.insert(pair, now_ms);
        let mut actions = Vec::new();
        let recipients: Vec<_> = self
            .connected_humans()
            .into_iter()
            .filter(|key| *key != sender)
            .collect();
        if !recipients.is_empty() {
            actions.push(DerbyAction {
                recipients,
                packet: encode_inner(&DerbyMessage::VehicleHit(report))?,
            });
        }
        if self.drivers[&report.other]
            .state
            .shield_until_ms
            .is_some_and(|until| until > now_ms)
        {
            return Ok(actions);
        }
        let was_leader = self.unique_leader();
        {
            let attacker = self.drivers.get_mut(&sender).unwrap();
            attacker.state.damage_dealt = attacker.state.damage_dealt.saturating_add(damage);
        }
        {
            let victim = self.drivers.get_mut(&report.other).unwrap();
            victim.state.damage_received = victim.state.damage_received.saturating_add(damage);
            victim.state.health = victim.state.health.saturating_sub(damage);
        }
        let knocked_out = self.drivers[&report.other].state.health == 0;
        if knocked_out {
            let points = if was_leader == Some(report.other) {
                self.rules.leader_knockout_points
            } else {
                self.rules.knockout_points
            };
            self.drivers.get_mut(&sender).unwrap().state.score =
                self.drivers[&sender].state.score.saturating_add(points);
            self.drivers.get_mut(&report.other).unwrap().state.score = self.drivers[&report.other]
                .state
                .score
                .saturating_sub(self.rules.knockout_point_loss);
            self.drivers.get_mut(&report.other).unwrap().respawn_at_ms =
                Some(now_ms.saturating_add(self.rules.respawn_delay_ms));
        }
        self.push_broadcast(&mut actions, self.state_message(sender))?;
        self.push_broadcast(&mut actions, self.state_message(report.other))?;
        if knocked_out {
            self.push_broadcast(
                &mut actions,
                DerbyMessage::Announcer {
                    kind: 2,
                    sub_kind: 0,
                    first: sender,
                    second: report.other,
                },
            )?;
            if self.unique_leader().is_some() && self.unique_leader() != was_leader {
                self.push_broadcast(
                    &mut actions,
                    DerbyMessage::Announcer {
                        kind: 1,
                        sub_kind: 0,
                        first: self.unique_leader().unwrap(),
                        second: ParticipantKey(0),
                    },
                )?;
            }
        }
        if self
            .rules
            .score_target
            .is_some_and(|target| self.drivers[&sender].state.score >= target)
        {
            actions.extend(self.finish_match(now_ms)?);
        }
        Ok(actions)
    }

    fn pickup_hit(
        &mut self,
        sender: ParticipantKey,
        position_bits: [u32; 4],
        now_ms: u64,
    ) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Running
            || !self.is_connected_human(sender)
            || self.drivers[&sender].state.health == 0
        {
            return Ok(Vec::new());
        }
        let position = [
            f32::from_bits(position_bits[0]),
            f32::from_bits(position_bits[1]),
            f32::from_bits(position_bits[2]),
        ];
        if !position.iter().all(|value| value.is_finite()) {
            return Ok(Vec::new());
        }
        let radius2 = self.rules.pickup_radius * self.rules.pickup_radius;
        let nearest = self
            .pickups
            .values()
            .filter_map(|pickup| {
                let p = [
                    f32::from_bits(pickup.position_bits[0]),
                    f32::from_bits(pickup.position_bits[1]),
                    f32::from_bits(pickup.position_bits[2]),
                ];
                let d2 = (p[0] - position[0]).powi(2)
                    + (p[1] - position[1]).powi(2)
                    + (p[2] - position[2]).powi(2);
                (d2.is_finite() && d2 <= radius2).then_some((d2, pickup.id))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0));
        let Some((_, id)) = nearest else {
            return Ok(Vec::new());
        };
        let pickup = self.pickups.remove(&id).unwrap();
        let driver = self.drivers.get_mut(&sender).unwrap();
        match pickup.kind {
            PickupKind::Health => {
                driver.state.health = driver
                    .state
                    .health
                    .saturating_add(self.rules.health_pickup)
                    .min(self.rules.start_health);
            }
            PickupKind::Boost => driver.state.boost += self.rules.boost_pickup,
            PickupKind::Shield => {
                driver.state.shield_until_ms =
                    Some(now_ms.saturating_add(self.rules.shield_duration_ms));
            }
            PickupKind::AttackUp => {
                driver.state.attack_until_ms =
                    Some(now_ms.saturating_add(self.rules.attack_duration_ms));
            }
            PickupKind::Ultimate => {
                driver.state.ultimate_until_ms =
                    Some(now_ms.saturating_add(self.rules.ultimate_duration_ms));
            }
        }
        let mut actions = Vec::new();
        self.push_broadcast(&mut actions, DerbyMessage::DestroyPickup(id))?;
        self.push_broadcast(
            &mut actions,
            DerbyMessage::PickupHitResponse {
                key: sender,
                kind: pickup.kind as u32,
                parameters_bits: pickup.response_parameters_bits,
            },
        )?;
        if matches!(
            pickup.kind,
            PickupKind::Shield | PickupKind::AttackUp | PickupKind::Ultimate
        ) {
            self.push_broadcast(
                &mut actions,
                DerbyMessage::Announcer {
                    kind: 0,
                    sub_kind: pickup.kind as u32,
                    first: sender,
                    second: ParticipantKey(0),
                },
            )?;
        }
        self.push_broadcast(&mut actions, self.state_message(sender))?;
        Ok(actions)
    }

    fn state_message(&self, key: ParticipantKey) -> DerbyMessage {
        let state = &self.drivers[&key].state;
        DerbyMessage::VehicleState(VehicleState {
            key,
            health: state.health,
            score: state.score,
            unknown_values: [0; 2],
            damage_dealt: state.damage_dealt,
            damage_received: state.damage_received,
            flags: [
                u8::from(state.shield_until_ms.is_some()),
                u8::from(state.attack_until_ms.is_some()),
                u8::from(state.ultimate_until_ms.is_some()),
            ],
            unknown_floats_bits: [0; 4],
        })
    }

    fn unique_leader(&self) -> Option<ParticipantKey> {
        let mut scores = self
            .drivers
            .values()
            .filter(|driver| driver.state.connected)
            .map(|driver| (driver.state.key, driver.state.score));
        let (mut key, mut high) = scores.next()?;
        let mut tied = false;
        for (candidate, score) in scores {
            if score > high {
                key = candidate;
                high = score;
                tied = false;
            } else if score == high {
                tied = true;
            }
        }
        (!tied && high > 0).then_some(key)
    }

    pub fn finish_match(&mut self, now_ms: u64) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase != DerbyPhase::Running {
            return Ok(Vec::new());
        }
        let mut drivers: Vec<_> = self
            .drivers
            .values()
            .filter(|driver| driver.state.connected)
            .collect();
        drivers.sort_by(|left, right| {
            let left_damage = left.state.damage_dealt as i64 - left.state.damage_received as i64;
            let right_damage = right.state.damage_dealt as i64 - right.state.damage_received as i64;
            right
                .state
                .score
                .cmp(&left.state.score)
                .then_with(|| right_damage.cmp(&left_damage))
                .then_with(|| left.state.key.cmp(&right.state.key))
        });
        let mut actions = Vec::new();
        for driver in drivers {
            self.push_broadcast(&mut actions, self.state_message(driver.state.key))?;
        }
        self.phase = DerbyPhase::Results;
        self.phase_acks.clear();
        self.results_until_ms = Some(now_ms.saturating_add(self.rules.results_duration_ms));
        self.push_broadcast(&mut actions, DerbyMessage::GameState(10))?;
        Ok(actions)
    }

    /// Process elapsed reconstructed timers. Native duration, respawn point and power-up lifetimes
    /// are not recovered; configured timers are server policy (R).
    pub fn tick(&mut self, now_ms: u64) -> Result<Vec<DerbyAction>, DerbyError> {
        if self.phase == DerbyPhase::Lobby {
            if self
                .lobby_deadline_ms
                .is_some_and(|deadline| deadline <= now_ms)
            {
                self.phase = DerbyPhase::Aborted;
                self.lobby_deadline_ms = None;
                self.pickups.clear();
                let removal = crate::minigame::state_remove_packet(self.instance_id, self.game_id);
                return Ok(self
                    .connected_humans()
                    .into_iter()
                    .map(|key| DerbyAction {
                        recipients: vec![key],
                        packet: removal.clone(),
                    })
                    .collect());
            }
            return Ok(Vec::new());
        }
        if self.phase == DerbyPhase::Results {
            if self
                .results_until_ms
                .is_some_and(|deadline| deadline <= now_ms)
            {
                self.results_until_ms = None;
                self.phase = DerbyPhase::CloseEnabled;
                return self.broadcast_one(DerbyMessage::GameState(11));
            }
            return Ok(Vec::new());
        }
        if self.phase != DerbyPhase::Running {
            return Ok(Vec::new());
        }
        let mut actions = Vec::new();
        let respawned: Vec<_> = self
            .drivers
            .iter()
            .filter_map(|(key, driver)| {
                driver
                    .respawn_at_ms
                    .is_some_and(|deadline| deadline <= now_ms)
                    .then_some(*key)
            })
            .collect();
        for key in respawned {
            let driver = self.drivers.get_mut(&key).unwrap();
            driver.state.health = self.rules.start_health;
            driver.respawn_at_ms = None;
            self.push_broadcast(&mut actions, DerbyMessage::ResetVehicle(key))?;
            self.push_broadcast(&mut actions, self.state_message(key))?;
        }
        // NPC poses are server-owned placeholders until the native AI is recovered. Refresh their
        // observation age so a human contact can still be validated against the last known position.
        for driver in self
            .drivers
            .values_mut()
            .filter(|driver| !driver.state.human)
        {
            if driver.pose.is_some() {
                driver.pose_at_ms = Some(now_ms);
            }
        }
        let expired: Vec<_> = self
            .drivers
            .iter()
            .flat_map(|(key, driver)| {
                [
                    (*key, 3, driver.state.shield_until_ms),
                    (*key, 4, driver.state.attack_until_ms),
                    (*key, 5, driver.state.ultimate_until_ms),
                ]
            })
            .filter_map(|(key, category, expiry)| {
                expiry
                    .filter(|deadline| *deadline <= now_ms)
                    .map(|_| (key, category))
            })
            .collect();
        for (key, category) in expired {
            let state = &mut self.drivers.get_mut(&key).unwrap().state;
            match category {
                3 => state.shield_until_ms = None,
                4 => state.attack_until_ms = None,
                5 => state.ultimate_until_ms = None,
                _ => unreachable!(),
            }
            self.push_broadcast(&mut actions, DerbyMessage::ItemExpired(category))?;
        }
        let timed_out = self.started_at_ms.is_some_and(|started| {
            self.rules
                .time_limit_ms
                .is_some_and(|limit| now_ms.saturating_sub(started) >= limit)
        });
        if timed_out {
            actions.extend(self.finish_match(now_ms)?);
        }
        Ok(actions)
    }

    pub fn disconnect(&mut self, key: ParticipantKey) -> Result<Vec<DerbyAction>, DerbyError> {
        if !self.humans.contains(&key) || !self.drivers.contains_key(&key) {
            return Ok(Vec::new());
        }
        self.drivers.remove(&key);
        self.humans.remove(&key);
        self.registered.remove(&key);
        self.phase_acks.remove(&key);
        self.countdown_acks.remove(&key);
        if self.human_count() == 0 {
            self.phase = DerbyPhase::Aborted;
            self.pickups.clear();
            return Ok(Vec::new());
        }
        let mut actions = Vec::new();
        self.push_broadcast(&mut actions, DerbyMessage::DestroyVehicle(key))?;
        self.push_broadcast(
            &mut actions,
            DerbyMessage::PlayerCount(self.human_count() as i32),
        )?;
        self.push_broadcast(
            &mut actions,
            DerbyMessage::NpcCount(self.npc_count() as i32),
        )?;
        Ok(actions)
    }

    /// Called after the common minigame layer validates its Close packet (26/42 and, if enabled,
    /// the 41/109 encounter leave). The client click is enabled only after derby state 11.
    /// F164 withholds derby payouts: every collision is a client `34/16` report, so no
    /// server-verified outcome exists to pay from. The Close still retires the match once.
    pub fn handle_close(
        &mut self,
        sender: ParticipantKey,
        packet: &[u8],
    ) -> Result<bool, DerbyError> {
        if self.phase != DerbyPhase::CloseEnabled
            || !self.is_connected_human(sender)
            || packet != [26, 0, 42, 0]
            || self.closed.contains(&sender)
        {
            return Ok(false);
        }
        self.closed.insert(sender);
        if self.closed.len() == self.human_count() {
            self.phase = DerbyPhase::Closed;
        }
        Ok(true)
    }

    fn is_connected_human(&self, key: ParticipantKey) -> bool {
        self.humans.contains(&key)
            && self
                .drivers
                .get(&key)
                .is_some_and(|driver| driver.state.human && driver.state.connected)
    }

    fn connected_humans(&self) -> Vec<ParticipantKey> {
        self.humans
            .iter()
            .copied()
            .filter(|key| self.is_connected_human(*key))
            .collect()
    }

    fn push_broadcast(
        &self,
        actions: &mut Vec<DerbyAction>,
        message: DerbyMessage,
    ) -> Result<(), DerbyError> {
        let recipients = self.connected_humans();
        if recipients.is_empty() {
            return Ok(());
        }
        actions.push(DerbyAction {
            recipients,
            packet: encode_inner(&message)?,
        });
        Ok(())
    }

    fn push_to(
        &self,
        actions: &mut Vec<DerbyAction>,
        recipient: ParticipantKey,
        message: DerbyMessage,
    ) -> Result<(), DerbyError> {
        if self.is_connected_human(recipient) {
            actions.push(DerbyAction {
                recipients: vec![recipient],
                packet: encode_inner(&message)?,
            });
        }
        Ok(())
    }

    fn broadcast_one(&self, message: DerbyMessage) -> Result<Vec<DerbyAction>, DerbyError> {
        let mut actions = Vec::new();
        self.push_broadcast(&mut actions, message)?;
        Ok(actions)
    }
}

fn ordered_pair(left: ParticipantKey, right: ParticipantKey) -> (ParticipantKey, ParticipantKey) {
    if left < right {
        (left, right)
    } else {
        (right, left)
    }
}

/// The derby pose carries a row-major 4x4; translation is in slots 12/13/14 (the same convention
/// proven for the kart weapon matrices). Non-finite matrices are not relayable or contactable.
fn pose_position(pose: &VehiclePose) -> Option<[f32; 3]> {
    if pose
        .transform_bits
        .iter()
        .any(|bits| !f32::from_bits(*bits).is_finite())
    {
        return None;
    }
    Some([
        f32::from_bits(pose.transform_bits[12]),
        f32::from_bits(pose.transform_bits[13]),
        f32::from_bits(pose.transform_bits[14]),
    ])
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], DerbyError> {
        let end = self.offset.checked_add(count).ok_or(DerbyError::TooShort)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(DerbyError::TooShort)?;
        self.offset = end;
        Ok(value)
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.offset
    }

    fn remaining_bytes(&self) -> &'a [u8] {
        &self.bytes[self.offset..]
    }

    fn u8(&mut self) -> Result<u8, DerbyError> {
        Ok(self.bytes(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, DerbyError> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, DerbyError> {
        Ok(i32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, DerbyError> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }

    fn key(&mut self) -> Result<ParticipantKey, DerbyError> {
        Ok(ParticipantKey(self.u64()?))
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], DerbyError> {
        Ok(self.bytes(N)?.try_into().unwrap())
    }

    fn words<const N: usize>(&mut self) -> Result<[u32; N], DerbyError> {
        let mut result = [0; N];
        for word in &mut result {
            *word = self.u32()?;
        }
        Ok(result)
    }

    fn finish(&self, subtype: u16, length: usize) -> Result<(), DerbyError> {
        if self.remaining() != 0 {
            return Err(DerbyError::UnknownSubtypeLength { subtype, length });
        }
        Ok(())
    }
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend(value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend(value.to_le_bytes());
}

fn put_key(out: &mut Vec<u8>, value: ParticipantKey) {
    put_u64(out, value.0);
}

fn put_words<const N: usize>(out: &mut Vec<u8>, values: &[u32; N]) {
    for value in values {
        put_u32(out, *value);
    }
}

fn read_i32(packet: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}
