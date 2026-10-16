//! Housing placement client wire (MMO work queue item 025).
//!
//! RECOVERED-ORIGINAL framing and request bodies, transcribed from the Ghidra export of the shipped
//! client (`lab/decompiled/client/FreeRealms/pseudocode.c`):
//!
//! * `FUN_00aba910(src, writer)` writes `u16` base opcode then `u16` inner opcode. `FUN_00aa4990`
//!   stamps base opcode `127` at source `+0x04` and the inner opcode at `+0x08`, so every housing
//!   body below is prefixed by `7f 00` and its inner opcode.
//! * `FUN_00abcb90` (sent through `FUN_00ac0400`, ctor `FUN_00aa5340` = opcode 1) writes
//!   `u32, u32, u32, u8`.
//! * `FUN_00abcb30` (sent through `FUN_00ac02f0`, ctor `FUN_00aa4cc0` = opcode 6) writes `u8`.
//! * `FUN_00abca80` (sent through `FUN_00ac01e0`, ctor `FUN_00aa83c0` = opcode 19) writes
//!   `u64, u32, u8`; the pinned community reader for opcode 19 reads the same three fields.
//! * `FUN_00abcd90` (sent through `FUN_00ac0a80`, ctor `FUN_00aa54f0` = opcode 20) writes `u32`.
//! * `FUN_00ac0510` (PickupFixture, ctor `FUN_00aa81a0` = opcode 3) writes one `u64`.
//! * `FUN_00abf620` (sent through `FUN_00ac7ce0`, ctor `FUN_00aad140` = opcode 5) writes
//!   `u64, quad, quad, u32` and then an undecoded nested block (`FUN_00abd1d0`) plus a trailing
//!   `u32`. That block is preserved verbatim as an opaque tail; no field names are invented for it.
//!
//! NOT recovered here, and therefore not modelled: the server responses for furniture update,
//! removal and placement denial (opcodes 40, 41, 53) and the instance payloads (28, 39). The
//! community C# model is a reconstruction, not client evidence, so it is not encoded as if it were
//! recovered. Routing decoded requests into `housing.rs` (item 006) is the integration step.

use std::fmt;

/// Base opcode of the whole housing family (`BaseHousingPacket` vtable construction).
pub const BASE_OPCODE: u16 = 127;

/// Byte sequence the client writes for every housing packet header.
pub const HEADER_LEN: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HousingWireMessage {
    /// Inner opcode 1, writer `FUN_00abcb90`. Semantics of the three words are not recovered.
    PlaceFixtureRequest {
        first: u32,
        second: u32,
        third: u32,
        flag: u8,
    },
    /// Inner opcode 3, writer `FUN_00ac0510`.
    PickupFixture { fixture_guid: u64 },
    /// Inner opcode 5, writer `FUN_00abf620`; the nested block stays opaque.
    SaveFixture(SaveFixture),
    /// Inner opcode 6, writer `FUN_00abcb30`.
    SetEditMode { in_edit_mode: bool },
    /// Inner opcode 19, writer `FUN_00abca80`.
    EnterRequest {
        house_guid: u64,
        unknown: u32,
        unknown_flag: bool,
    },
    /// Inner opcode 20, writer `FUN_00abcd90`.
    PreviewByItemDefinitionIdRequest { item_definition_id: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFixture {
    pub house_guid: u64,
    /// Four `u32` words written by `FUN_0091a640` at source `+0x20`. Float interpretation is not proven.
    pub quad_a: [u32; 4],
    /// Four `u32` words written by `FUN_0091a640` at source `+0x30`.
    pub quad_b: [u32; 4],
    /// `u32` written from source `+0x40`.
    pub value: u32,
    /// The undecoded `FUN_00abd1d0` block, preserved byte for byte so a round trip cannot drop data.
    pub tail: Vec<u8>,
    /// `u32` the writer emits from source `+0x7c` after the nested block.
    pub trailing: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    TooShort,
    WrongBaseOpcode { found: u16 },
    UnknownOpcode { opcode: u16 },
    Truncated,
    TrailingBytes { count: usize },
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "housing wire frame shorter than its header"),
            Self::WrongBaseOpcode { found } => {
                write!(
                    f,
                    "base opcode {found} is not the housing base {BASE_OPCODE}"
                )
            }
            Self::UnknownOpcode { opcode } => write!(f, "unrecovered housing opcode {opcode}"),
            Self::Truncated => write!(f, "housing body ends mid-field"),
            Self::TrailingBytes { count } => {
                write!(f, "{count} bytes remain after the housing body")
            }
        }
    }
}

impl std::error::Error for WireError {}

impl HousingWireMessage {
    pub fn opcode(&self) -> u16 {
        match self {
            Self::PlaceFixtureRequest { .. } => 1,
            Self::PickupFixture { .. } => 3,
            Self::SaveFixture(_) => 5,
            Self::SetEditMode { .. } => 6,
            Self::EnterRequest { .. } => 19,
            Self::PreviewByItemDefinitionIdRequest { .. } => 20,
        }
    }

    pub fn packet_name(&self) -> &'static str {
        match self {
            Self::PlaceFixtureRequest { .. } => "ClientHousingPacketPlaceFixtureRequest",
            Self::PickupFixture { .. } => "ClientHousingPacketPickupFixture",
            Self::SaveFixture(_) => "ClientHousingPacketSaveFixture",
            Self::SetEditMode { .. } => "ClientHousingPacketSetEditMode",
            Self::EnterRequest { .. } => "ClientHousingPacketEnterRequest",
            Self::PreviewByItemDefinitionIdRequest { .. } => {
                "ClientHousingPacketPreviewByItemDefinitionIdRequest"
            }
        }
    }
}

/// Serializes a housing packet exactly as the recovered client writers do.
pub fn encode(message: &HousingWireMessage) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&BASE_OPCODE.to_le_bytes());
    bytes.extend_from_slice(&message.opcode().to_le_bytes());
    match message {
        HousingWireMessage::PlaceFixtureRequest {
            first,
            second,
            third,
            flag,
        } => {
            bytes.extend_from_slice(&first.to_le_bytes());
            bytes.extend_from_slice(&second.to_le_bytes());
            bytes.extend_from_slice(&third.to_le_bytes());
            bytes.push(*flag);
        }
        HousingWireMessage::PickupFixture { fixture_guid } => {
            bytes.extend_from_slice(&fixture_guid.to_le_bytes());
        }
        HousingWireMessage::SaveFixture(save) => {
            bytes.extend_from_slice(&save.house_guid.to_le_bytes());
            for word in save.quad_a.iter().chain(save.quad_b.iter()) {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            bytes.extend_from_slice(&save.value.to_le_bytes());
            bytes.extend_from_slice(&save.tail);
            bytes.extend_from_slice(&save.trailing.to_le_bytes());
        }
        HousingWireMessage::SetEditMode { in_edit_mode } => {
            bytes.push(u8::from(*in_edit_mode));
        }
        HousingWireMessage::EnterRequest {
            house_guid,
            unknown,
            unknown_flag,
        } => {
            bytes.extend_from_slice(&house_guid.to_le_bytes());
            bytes.extend_from_slice(&unknown.to_le_bytes());
            bytes.push(u8::from(*unknown_flag));
        }
        HousingWireMessage::PreviewByItemDefinitionIdRequest { item_definition_id } => {
            bytes.extend_from_slice(&item_definition_id.to_le_bytes());
        }
    }
    bytes
}

/// Parses a full housing datagram body (header included).
pub fn decode(bytes: &[u8]) -> Result<HousingWireMessage, WireError> {
    if bytes.len() < HEADER_LEN {
        return Err(WireError::TooShort);
    }
    let base = u16::from_le_bytes([bytes[0], bytes[1]]);
    if base != BASE_OPCODE {
        return Err(WireError::WrongBaseOpcode { found: base });
    }
    let opcode = u16::from_le_bytes([bytes[2], bytes[3]]);
    let body = &bytes[HEADER_LEN..];
    let (message, used) = match opcode {
        1 => {
            if body.len() < 13 {
                return Err(WireError::Truncated);
            }
            (
                HousingWireMessage::PlaceFixtureRequest {
                    first: read_u32(&body[0..4]),
                    second: read_u32(&body[4..8]),
                    third: read_u32(&body[8..12]),
                    flag: body[12],
                },
                13,
            )
        }
        3 => {
            if body.len() < 8 {
                return Err(WireError::Truncated);
            }
            (
                HousingWireMessage::PickupFixture {
                    fixture_guid: read_u64(&body[0..8]),
                },
                8,
            )
        }
        5 => {
            if body.len() < 8 + 16 + 16 + 4 + 4 {
                return Err(WireError::Truncated);
            }
            let mut quad_a = [0u32; 4];
            let mut quad_b = [0u32; 4];
            for (index, slot) in quad_a.iter_mut().enumerate() {
                *slot = read_u32(&body[8 + index * 4..12 + index * 4]);
            }
            for (index, slot) in quad_b.iter_mut().enumerate() {
                *slot = read_u32(&body[24 + index * 4..28 + index * 4]);
            }
            let trailing_at = body.len() - 4;
            (
                HousingWireMessage::SaveFixture(SaveFixture {
                    house_guid: read_u64(&body[0..8]),
                    quad_a,
                    quad_b,
                    value: read_u32(&body[40..44]),
                    tail: body[44..trailing_at].to_vec(),
                    trailing: read_u32(&body[trailing_at..]),
                }),
                body.len(),
            )
        }
        6 => {
            if body.is_empty() {
                return Err(WireError::Truncated);
            }
            (
                HousingWireMessage::SetEditMode {
                    in_edit_mode: body[0] != 0,
                },
                1,
            )
        }
        19 => {
            if body.len() < 13 {
                return Err(WireError::Truncated);
            }
            (
                HousingWireMessage::EnterRequest {
                    house_guid: read_u64(&body[0..8]),
                    unknown: read_u32(&body[8..12]),
                    unknown_flag: body[12] != 0,
                },
                13,
            )
        }
        20 => {
            if body.len() < 4 {
                return Err(WireError::Truncated);
            }
            (
                HousingWireMessage::PreviewByItemDefinitionIdRequest {
                    item_definition_id: read_u32(&body[0..4]),
                },
                4,
            )
        }
        other => return Err(WireError::UnknownOpcode { opcode: other }),
    };
    if used != body.len() {
        return Err(WireError::TrailingBytes {
            count: body.len() - used,
        });
    }
    Ok(message)
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn read_u64(bytes: &[u8]) -> u64 {
    let mut value = [0u8; 8];
    value.copy_from_slice(bytes);
    u64::from_le_bytes(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    U8,
    U32,
    U64,
    U32Quad,
    /// Undecoded writer block preserved verbatim; contributes no fixed size.
    OpaqueTail,
}

impl FieldKind {
    pub fn size(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U32 => 4,
            Self::U64 => 8,
            Self::U32Quad => 16,
            Self::OpaqueTail => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
}

pub struct SchemaEntry {
    pub opcode: u16,
    pub packet: &'static str,
    pub direction: Direction,
    pub fields: &'static [FieldKind],
    pub evidence: &'static str,
}

impl SchemaEntry {
    pub fn wire_size(&self) -> usize {
        HEADER_LEN + self.fields.iter().map(|field| field.size()).sum::<usize>()
    }

    /// A deterministic sample used by tests to prove the table and the encoder agree.
    pub fn sample(&self) -> HousingWireMessage {
        match self.opcode {
            1 => HousingWireMessage::PlaceFixtureRequest {
                first: 1,
                second: 2,
                third: 3,
                flag: 4,
            },
            3 => HousingWireMessage::PickupFixture { fixture_guid: 5 },
            5 => HousingWireMessage::SaveFixture(SaveFixture {
                house_guid: 6,
                quad_a: [7, 8, 9, 10],
                quad_b: [11, 12, 13, 14],
                value: 15,
                tail: Vec::new(),
                trailing: 16,
            }),
            6 => HousingWireMessage::SetEditMode { in_edit_mode: true },
            19 => HousingWireMessage::EnterRequest {
                house_guid: 16,
                unknown: 17,
                unknown_flag: false,
            },
            20 => HousingWireMessage::PreviewByItemDefinitionIdRequest {
                item_definition_id: 18,
            },
            other => panic!("no sample for housing opcode {other}"),
        }
    }
}

/// Recovered wire table. Every entry names the writer function it was transcribed from.
pub fn schema() -> &'static [SchemaEntry] {
    &[
        SchemaEntry {
            opcode: 1,
            packet: "ClientHousingPacketPlaceFixtureRequest",
            direction: Direction::ClientToServer,
            fields: &[
                FieldKind::U32,
                FieldKind::U32,
                FieldKind::U32,
                FieldKind::U8,
            ],
            evidence: "FUN_00abcb90 / ctor FUN_00aa5340 / send FUN_00ac0400",
        },
        SchemaEntry {
            opcode: 3,
            packet: "ClientHousingPacketPickupFixture",
            direction: Direction::ClientToServer,
            fields: &[FieldKind::U64],
            evidence: "FUN_00ac0510 / ctor FUN_00aa81a0",
        },
        SchemaEntry {
            opcode: 5,
            packet: "ClientHousingPacketSaveFixture",
            direction: Direction::ClientToServer,
            fields: &[
                FieldKind::U64,
                FieldKind::U32Quad,
                FieldKind::U32Quad,
                FieldKind::U32,
                FieldKind::OpaqueTail,
                FieldKind::U32,
            ],
            evidence: "FUN_00abf620 / ctor FUN_00aad140 / send FUN_00ac7ce0",
        },
        SchemaEntry {
            opcode: 6,
            packet: "ClientHousingPacketSetEditMode",
            direction: Direction::ClientToServer,
            fields: &[FieldKind::U8],
            evidence: "FUN_00abcb30 / ctor FUN_00aa4cc0 / community reader agrees",
        },
        SchemaEntry {
            opcode: 19,
            packet: "ClientHousingPacketEnterRequest",
            direction: Direction::ClientToServer,
            fields: &[FieldKind::U64, FieldKind::U32, FieldKind::U8],
            evidence: "FUN_00abca80 / ctor FUN_00aa83c0 / community reader agrees",
        },
        SchemaEntry {
            opcode: 20,
            packet: "ClientHousingPacketPreviewByItemDefinitionIdRequest",
            direction: Direction::ClientToServer,
            fields: &[FieldKind::U32],
            evidence: "FUN_00abcd90 / ctor FUN_00aa54f0 / send FUN_00ac0a80",
        },
    ]
}

/// Opcodes of the family whose bodies are still unrecovered, kept explicit so callers cannot
/// mistake a silent skip for support.
pub const UNRECOVERED_OPCODES: &[(u16, &str)] = &[
    (28, "HousingPacketInstanceData (server to client)"),
    (39, "HousingPacketInstanceList (server to client)"),
    (40, "HousingPacketFixtureUpdate (server to client)"),
    (41, "HousingPacketFixtureRemove (server to client)"),
    (53, "HousingPacketFixturePlacementDenied (server to client)"),
];
