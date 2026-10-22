//! Piece 077 - warpstones and travel: recovered warpstone request/destination
//! wire layouts, the curated destination set and the reconstructed travel rules.
//!
//! Provenance and the full boundary live in
//! `notes/Research/Pieces/077 Warpstones and travel.md` and in the header of
//! `tests/piece_077.rs`. In short:
//!
//! * ORIGINAL (first-party native client, `pseudocode.c`): the base ids below
//!   are written next to each packet's vftable; `PacketZoneTeleportRequest`
//!   carries exactly one 32-bit payload and `ClientUpdatePacketUpdateLocation`
//!   writes sub-opcode `0xc`. The client's point-of-interest data source column
//!   order is recovered by `FUN_00b97c40`.
//! * ORIGINAL (native map, W142): the client's `57` handler `00b98b70` reads
//!   `u16` opcode, then an `i32` blob length (`ReadBoundedLengthBlob`
//!   `008bfc00`), clears its table (`00b98160`) and reads the flag-framed stream
//!   (`00b98990`: flag byte, record, flag byte ...) with `00b98290` (13 dwords,
//!   a length-prefixed string, one dword). So the length prefix is real, a `57`
//!   replaces the whole table, and the client builds an op-90 request only for
//!   an id already in that table (`00b991d0`, reached from Lua through
//!   `00c17bb0`). The client sends `122` itself when its character falls below
//!   a world floor (`00bf9430`).
//! * ORIGINAL (preserved live captures): [`CAPTURED_TELEPORT_SAMPLES`] are the
//!   eight real request frames and their `38/12` replies.
//! * ORIGINAL (supplied emulator data, equal to the captured gateway `57`):
//!   the zone-hub rows (notification type 7, sub name "Press to Teleport...")
//!   in [`CURATED`].
//! * RECONSTRUCTED (R): everything in [`WarpRules`] and [`WarpRegistry`] that
//!   decides whether a request moves the character. No captured frame proves
//!   the original server ever refused a request.
//!
//! The registry never moves anyone itself: it returns a [`WarpReply`] whose
//! relocation the gateway applies (the same `Outcome.relocation` path as
//! `realm goto`), so server position, saved position and movement validation
//! stay in agreement. Nothing grants items, currency or experience.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// `PacketPointOfInterestDefinitionRequest` (client asks for the destination list).
pub const POINT_OF_INTEREST_DEFINITION_REQUEST_OPCODE: u16 = 56;
/// `PacketPointOfInterestDefinitionReply` (server sends the destination list).
pub const POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE: u16 = 57;
/// `PacketWorldTeleportRequest` (teleport to a player guid, e.g. a friend).
pub const WORLD_TELEPORT_REQUEST_OPCODE: u16 = 58;
/// `PacketZoneTeleportRequest` (the warpstone / point-of-interest request).
pub const ZONE_TELEPORT_REQUEST_OPCODE: u16 = 90;
/// `PacketZoneSafeTeleportRequest` (return to the nearest safe point).
pub const ZONE_SAFE_TELEPORT_REQUEST_OPCODE: u16 = 122;
/// `BaseClientUpdatePacket` family opcode.
pub const CLIENT_UPDATE_PACKET_OPCODE: u16 = 38;
/// `ClientUpdatePacketUpdateLocation` sub-opcode inside the family above.
pub const UPDATE_LOCATION_SUB_OPCODE: u16 = 12;
/// The seamless open world. Warpstones only work here (R); cave, arena and kart
/// scenes are other zones. Mirrors `world::OPEN_WORLD_ZONE`.
pub const OPEN_WORLD_ZONE: u32 = 0;

/// Length-prefixed ASCII string, as written by both recovered serializers
/// (`SOEWriter.AddASCIIString`, `PacketWriter.Write(string)`).
fn encode_ascii_string(value: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + value.len());
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
    out
}

fn read_ascii_string(body: &[u8], offset: &mut usize) -> Result<String, WarpCodecError> {
    let start = *offset;
    let length = read_i32(body, offset)?;
    if length < 0 {
        return Err(WarpCodecError::NegativeLength {
            offset: start,
            length,
        });
    }
    let length = length as usize;
    let end = offset
        .checked_add(length)
        .ok_or(WarpCodecError::Truncated {
            expected: usize::MAX,
            got: body.len(),
        })?;
    let slice = body.get(*offset..end).ok_or(WarpCodecError::Truncated {
        expected: end,
        got: body.len(),
    })?;
    *offset = end;
    String::from_utf8(slice.to_vec())
        .map_err(|_| WarpCodecError::InvalidAsciiString { offset: start })
}

fn read_i32(body: &[u8], offset: &mut usize) -> Result<i32, WarpCodecError> {
    let end = offset.checked_add(4).ok_or(WarpCodecError::Truncated {
        expected: usize::MAX,
        got: body.len(),
    })?;
    let slice = body.get(*offset..end).ok_or(WarpCodecError::Truncated {
        expected: end,
        got: body.len(),
    })?;
    *offset = end;
    Ok(i32::from_le_bytes(slice.try_into().unwrap()))
}

fn read_f32(body: &[u8], offset: &mut usize) -> Result<f32, WarpCodecError> {
    let end = offset.checked_add(4).ok_or(WarpCodecError::Truncated {
        expected: usize::MAX,
        got: body.len(),
    })?;
    let slice = body.get(*offset..end).ok_or(WarpCodecError::Truncated {
        expected: end,
        got: body.len(),
    })?;
    *offset = end;
    Ok(f32::from_le_bytes(slice.try_into().unwrap()))
}

/// Strict body-length failures. Every request below states its exact length,
/// so a short body and a body with trailing bytes are both malformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpCodecError {
    Truncated {
        expected: usize,
        got: usize,
    },
    TrailingBytes {
        expected: usize,
        got: usize,
    },
    MissingTerminator,
    NegativeLength {
        offset: usize,
        length: i32,
    },
    /// The `57` length prefix does not match the bytes that follow it.
    LengthMismatch {
        declared: i32,
        actual: usize,
    },
    InvalidAsciiString {
        offset: usize,
    },
    UnexpectedSubOpcode {
        expected: u16,
        got: u16,
    },
}

fn expect_exact(body: &[u8], expected: usize) -> Result<(), WarpCodecError> {
    if body.len() < expected {
        return Err(WarpCodecError::Truncated {
            expected,
            got: body.len(),
        });
    }
    if body.len() > expected {
        return Err(WarpCodecError::TrailingBytes {
            expected,
            got: body.len(),
        });
    }
    Ok(())
}

/// `PacketZoneTeleportRequest`: one little-endian point-of-interest id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneTeleportRequest {
    pub destination_id: i32,
}

impl ZoneTeleportRequest {
    pub const BODY_LEN: usize = 4;

    pub fn decode(body: &[u8]) -> Result<Self, WarpCodecError> {
        expect_exact(body, Self::BODY_LEN)?;
        Ok(Self {
            destination_id: i32::from_le_bytes(body.try_into().unwrap()),
        })
    }

    pub fn encode(self) -> [u8; Self::BODY_LEN] {
        self.destination_id.to_le_bytes()
    }
}

/// `PacketZoneSafeTeleportRequest`: header only. The destination is resolved by
/// the world side (nearest safe point), so the request carries no id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneSafeTeleportRequest;

impl ZoneSafeTeleportRequest {
    pub const BODY_LEN: usize = 0;

    pub fn decode(body: &[u8]) -> Result<Self, WarpCodecError> {
        expect_exact(body, Self::BODY_LEN)?;
        Ok(Self)
    }
}

/// `PacketWorldTeleportRequest`: one little-endian 64-bit player guid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldTeleportRequest {
    pub player_guid: u64,
}

impl WorldTeleportRequest {
    pub const BODY_LEN: usize = 8;

    pub fn decode(body: &[u8]) -> Result<Self, WarpCodecError> {
        expect_exact(body, Self::BODY_LEN)?;
        Ok(Self {
            player_guid: u64::from_le_bytes(body.try_into().unwrap()),
        })
    }

    pub fn encode(self) -> [u8; Self::BODY_LEN] {
        self.player_guid.to_le_bytes()
    }
}

/// `ClientUpdatePacketUpdateLocation`: the position/rotation block the server
/// answers a warpstone request with, then the teleport flag and the one trailer
/// byte the captures carry. The trailer byte is opaque; the captured samples
/// hold every observed value and no meaning is invented for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UpdateLocation {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub teleport: bool,
    pub trailer: u8,
}

impl UpdateLocation {
    pub const BODY_LEN: usize = 34;
    pub const MESSAGE_LEN: usize = 2 + Self::BODY_LEN;

    pub fn decode_body(body: &[u8]) -> Result<Self, WarpCodecError> {
        expect_exact(body, Self::BODY_LEN)?;
        let mut offset = 0;
        let mut position = [0.0f32; 4];
        for value in &mut position {
            *value = read_f32(body, &mut offset)?;
        }
        let mut rotation = [0.0f32; 4];
        for value in &mut rotation {
            *value = read_f32(body, &mut offset)?;
        }
        let teleport = body[offset] != 0;
        offset += 1;
        let trailer = body[offset];
        Ok(Self {
            position,
            rotation,
            teleport,
            trailer,
        })
    }

    pub fn encode_body(self) -> [u8; Self::BODY_LEN] {
        let mut body = [0u8; Self::BODY_LEN];
        let mut offset = 0;
        for value in self.position.iter().chain(self.rotation.iter()) {
            body[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        body[offset] = u8::from(self.teleport);
        body[offset + 1] = self.trailer;
        body
    }

    /// Sub-opcode plus body, i.e. everything after the `38` family opcode.
    pub fn encode_message(self) -> Vec<u8> {
        let mut message = Vec::with_capacity(Self::MESSAGE_LEN);
        message.extend_from_slice(&UPDATE_LOCATION_SUB_OPCODE.to_le_bytes());
        message.extend_from_slice(&self.encode_body());
        message
    }

    pub fn decode_message(message: &[u8]) -> Result<Self, WarpCodecError> {
        expect_exact(message, Self::MESSAGE_LEN)?;
        let sub = u16::from_le_bytes(message[..2].try_into().unwrap());
        if sub != UPDATE_LOCATION_SUB_OPCODE {
            return Err(WarpCodecError::UnexpectedSubOpcode {
                expected: UPDATE_LOCATION_SUB_OPCODE,
                got: sub,
            });
        }
        Self::decode_body(&message[2..])
    }
}

/// One point-of-interest row from the destination list. Field order follows the
/// two recovered community serializers, which agree with the client's own data
/// source column order for every shared column.
#[derive(Clone, Debug, PartialEq)]
pub struct PointOfInterestDefinition {
    pub id: i32,
    pub name_id: i32,
    pub location_id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
    pub icon_id: i32,
    pub notification_type: i32,
    pub sub_name_id: i32,
    pub unknown_11: i32,
    pub breadcrumb_quest_id: i32,
    pub teleport_location_id: i32,
    pub atlas_name: String,
    pub unknown_15: i32,
}

/// Encodes the `57` destination list as the recovered bool-framed record stream
/// (`true` + record per row, then `false`).
///
/// Open question, kept out of the encoder: the legacy server additionally
/// prefixes this stream with its byte length, the newer community server does
/// not, and no preserved capture contains a `57` frame. [`decode_point_of_interest_reply`]
/// therefore tolerates the length prefix; the encoder writes the bare stream.
pub fn encode_point_of_interest_reply(records: &[PointOfInterestDefinition]) -> Vec<u8> {
    let mut body = Vec::new();
    for record in records {
        body.push(1);
        body.extend_from_slice(&record.id.to_le_bytes());
        body.extend_from_slice(&record.name_id.to_le_bytes());
        body.extend_from_slice(&record.location_id.to_le_bytes());
        for value in [record.x, record.y, record.z, record.heading] {
            body.extend_from_slice(&value.to_le_bytes());
        }
        body.extend_from_slice(&record.icon_id.to_le_bytes());
        body.extend_from_slice(&record.notification_type.to_le_bytes());
        body.extend_from_slice(&record.sub_name_id.to_le_bytes());
        body.extend_from_slice(&record.unknown_11.to_le_bytes());
        body.extend_from_slice(&record.breadcrumb_quest_id.to_le_bytes());
        body.extend_from_slice(&record.teleport_location_id.to_le_bytes());
        body.extend_from_slice(&encode_ascii_string(&record.atlas_name));
        body.extend_from_slice(&record.unknown_15.to_le_bytes());
    }
    body.push(0);
    body
}

/// The whole `57` inner message: `u16` opcode, `i32` length of the flag-framed
/// stream, then the stream (terminator included). Native reader `00b98b70`.
pub fn point_of_interest_reply_message(records: &[PointOfInterestDefinition]) -> Vec<u8> {
    let stream = encode_point_of_interest_reply(records);
    let mut message = Vec::with_capacity(6 + stream.len());
    message.extend_from_slice(&POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE.to_le_bytes());
    message.extend_from_slice(&(stream.len() as i32).to_le_bytes());
    message.extend_from_slice(&stream);
    message
}

/// Strict inverse of [`point_of_interest_reply_message`]: the length must be
/// exactly what follows, as the native bounded reader requires.
pub fn decode_point_of_interest_message(
    message: &[u8],
) -> Result<Vec<PointOfInterestDefinition>, WarpCodecError> {
    if message.len() < 6 {
        return Err(WarpCodecError::Truncated {
            expected: 6,
            got: message.len(),
        });
    }
    let sub = u16::from_le_bytes([message[0], message[1]]);
    if sub != POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE {
        return Err(WarpCodecError::UnexpectedSubOpcode {
            expected: POINT_OF_INTEREST_DEFINITION_REPLY_OPCODE,
            got: sub,
        });
    }
    let declared = i32::from_le_bytes(message[2..6].try_into().unwrap());
    if declared < 0 || declared as usize != message.len() - 6 {
        return Err(WarpCodecError::LengthMismatch {
            declared,
            actual: message.len() - 6,
        });
    }
    let stream = &message[6..];
    let mut offset = 0;
    let mut records = Vec::new();
    loop {
        let more = *stream
            .get(offset)
            .ok_or(WarpCodecError::MissingTerminator)?;
        offset += 1;
        if more == 0 {
            return if offset == stream.len() {
                Ok(records)
            } else {
                Err(WarpCodecError::TrailingBytes {
                    expected: offset,
                    got: stream.len(),
                })
            };
        }
        records.push(decode_point_of_interest_record(stream, &mut offset)?);
    }
}

pub fn decode_point_of_interest_reply(
    body: &[u8],
) -> Result<Vec<PointOfInterestDefinition>, WarpCodecError> {
    let body = strip_optional_length_prefix(body);
    let mut offset = 0;
    let mut records = Vec::new();
    loop {
        let more = *body.get(offset).ok_or(WarpCodecError::MissingTerminator)?;
        offset += 1;
        if more == 0 {
            return Ok(records);
        }
        records.push(decode_point_of_interest_record(body, &mut offset)?);
    }
}

fn strip_optional_length_prefix(body: &[u8]) -> &[u8] {
    if body.len() >= 4 {
        let declared = u32::from_le_bytes(body[..4].try_into().unwrap()) as usize;
        if declared == body.len() - 4 {
            let inner = &body[4..];
            if inner
                .first()
                .is_some_and(|first| *first == 0 || *first == 1)
            {
                return inner;
            }
        }
    }
    body
}

fn decode_point_of_interest_record(
    body: &[u8],
    offset: &mut usize,
) -> Result<PointOfInterestDefinition, WarpCodecError> {
    Ok(PointOfInterestDefinition {
        id: read_i32(body, offset)?,
        name_id: read_i32(body, offset)?,
        location_id: read_i32(body, offset)?,
        x: read_f32(body, offset)?,
        y: read_f32(body, offset)?,
        z: read_f32(body, offset)?,
        heading: read_f32(body, offset)?,
        icon_id: read_i32(body, offset)?,
        notification_type: read_i32(body, offset)?,
        sub_name_id: read_i32(body, offset)?,
        unknown_11: read_i32(body, offset)?,
        breadcrumb_quest_id: read_i32(body, offset)?,
        teleport_location_id: read_i32(body, offset)?,
        atlas_name: read_ascii_string(body, offset)?,
        unknown_15: read_i32(body, offset)?,
    })
}

/// One warpstone destination the server can teleport a character to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpDestination {
    pub id: i32,
    /// Shown in the attunement notice (client locale text of the row's name id).
    pub name: &'static str,
    /// Recovered rows for non-warpstone points of interest are legitimately in
    /// the client's destination list; only warpstones travel.
    pub is_warpstone: bool,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    /// `rotation` is a captured original spawn facing. When false the arrival
    /// keeps the character's current facing: no facing convention is invented.
    pub facing_known: bool,
}

/// One captured request and the reply the original server sent for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapturedTeleportSample {
    pub capture: &'static str,
    pub capture_sha256: &'static str,
    pub request_frame: u32,
    pub reply_frame: u32,
    pub destination_id: i32,
    /// Body after the `90` opcode.
    pub request_body_hex: &'static str,
    /// Sub-opcode plus body after the `38` family opcode.
    pub reply_message_hex: &'static str,
}

/// The eight `90` request frames and their `38/12` replies found in the
/// preserved community captures (`lab/.../ofrserver/Packets`). `p2.pcap` and
/// `packets_2.pcapng` are the same recording in two container formats; both are
/// pinned so a re-run of the checker can prove it.
pub const CAPTURED_TELEPORT_SAMPLES: [CapturedTeleportSample; 8] = [
    CapturedTeleportSample {
        capture: "p2.pcap",
        capture_sha256: "65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108",
        request_frame: 420,
        reply_frame: 457,
        destination_id: 9,
        request_body_hex: "09000000",
        reply_message_hex: "0c00c77b37c442601141bc7c6c440000803fdfd77c3f000000007c5020be000000000101",
    },
    CapturedTeleportSample {
        capture: "p2.pcap",
        capture_sha256: "65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108",
        request_frame: 1282,
        reply_frame: 1302,
        destination_id: 155,
        request_body_hex: "9b000000",
        reply_message_hex: "0c00aea7014377bee441336394430000803f10d18b3e00000000134576bf000000000102",
    },
    CapturedTeleportSample {
        capture: "p12.pcap",
        capture_sha256: "99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97",
        request_frame: 14089,
        reply_frame: 14126,
        destination_id: 9,
        request_body_hex: "09000000",
        reply_message_hex: "0c00c77b37c442601141bc7c6c440000803fdfd77c3f000000007c5020be000000000101",
    },
    CapturedTeleportSample {
        capture: "p12.pcap",
        capture_sha256: "99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97",
        request_frame: 14951,
        reply_frame: 14971,
        destination_id: 155,
        request_body_hex: "9b000000",
        reply_message_hex: "0c00aea7014377bee441336394430000803f10d18b3e00000000134576bf000000000102",
    },
    CapturedTeleportSample {
        capture: "packets_3.pcapng",
        capture_sha256: "c56341ca756984602b369bbad3a8a410e627e94a1346c8852aa46b17e7fff41a",
        request_frame: 432,
        reply_frame: 456,
        destination_id: 152,
        request_body_hex: "98000000",
        reply_message_hex: "0c00f6306344d122db3f0ca20b450000803f1d1e78bf00000000af257c3e000000000103",
    },
    CapturedTeleportSample {
        capture: "packets_3.pcapng",
        capture_sha256: "c56341ca756984602b369bbad3a8a410e627e94a1346c8852aa46b17e7fff41a",
        request_frame: 5393,
        reply_frame: 5395,
        destination_id: 131,
        request_body_hex: "83000000",
        reply_message_hex: "0c0068d100433333b441c15ac3430000803ffc6c48be000000006a0c7b3f000000000100",
    },
    CapturedTeleportSample {
        capture: "packets_4.pcapng",
        capture_sha256: "5734593a76f08a79013378d2f5c71c40356e6b54d41d2ad245d7f7a8af8013d5",
        request_frame: 9609,
        reply_frame: 9626,
        destination_id: 15,
        request_body_hex: "0f000000",
        reply_message_hex: "0c00e5d604c5508dbdc1df0f92c30000803f04b86d3f000000001302bebe000000000103",
    },
    CapturedTeleportSample {
        capture: "packets_4.pcapng",
        capture_sha256: "5734593a76f08a79013378d2f5c71c40356e6b54d41d2ad245d7f7a8af8013d5",
        request_frame: 10512,
        reply_frame: 10623,
        destination_id: 31,
        request_body_hex: "1f000000",
        reply_message_hex: "0c00db590bc5e7fbb2c1588111440000803f436e7dbf000000009db6103e000000000100",
    },
];

/// The six distinct destinations the captures teleport to, with the spawn
/// position and rotation the original server sent back for them. These are
/// captured wire values, not a complete catalogue: the full `57` list was never
/// captured and stays a documented gap.
pub fn captured_destinations() -> Vec<WarpDestination> {
    [
        // Literals are the exact f32 values from the captured replies, so
        // re-encoding reproduces the frames byte-for-byte.
        (
            9,
            [-733.9340209960938, 9.086000442504883, 945.948974609375, 1.0],
            [0.9876689314842224, 0.0, -0.15655702352523804, 0.0],
        ),
        (
            155,
            [
                129.65499877929688,
                28.593000411987305,
                296.7749938964844,
                1.0,
            ],
            [0.27307939529418945, 0.0, -0.9619914889335632, 0.0],
        ),
        (
            152,
            [908.7650146484375, 1.7120000123977661, 2234.1279296875, 1.0],
            [-0.9692094922065735, 0.0, 0.24623750150203705, 0.0],
        ),
        (
            131,
            [
                128.8179931640625,
                22.524999618530273,
                390.7090148925781,
                1.0,
            ],
            [-0.19572824239730835, 0.0, 0.9806581735610962, 0.0],
        ),
        (
            15,
            [
                -2125.430908203125,
                -23.694000244140625,
                -292.1239929199219,
                1.0,
            ],
            [0.9285891056060791, 0.0, -0.37110957503318787, 0.0],
        ),
        (
            31,
            [-2229.615966796875, -22.37299919128418, 582.02099609375, 1.0],
            [-0.989963710308075, 0.0, 0.14132161438465118, 0.0],
        ),
    ]
    .into_iter()
    .map(|(id, position, rotation)| WarpDestination {
        id,
        name: "captured",
        is_warpstone: true,
        position,
        rotation,
        facing_known: true,
    })
    .collect()
}

/// One curated zone hub. ORIGINAL data: the supplied emulator's
/// `PointOfInterestDefinitions.json`, which `piece_073` proved equal, field for
/// field, to the `57` message the original client accepted. Every hub shares
/// the fields in the `HUB_*` constants below.
struct CuratedRow {
    id: i32,
    /// Client locale text of `name_id`.
    name: &'static str,
    name_id: i32,
    location_id: i32,
    position: [f32; 3],
    heading: f32,
    unknown_11: i32,
    teleport_location_id: i32,
    unknown_15: i32,
}

/// Notification type 7 ("ZoneHub"), the rows whose sub name is the client text
/// "Press to Teleport..." (locale 382845): the warpstone towns.
const HUB_NOTIFICATION_TYPE: i32 = 7;
const HUB_ICON_ID: i32 = 1345;
const HUB_SUB_NAME_ID: i32 = 382845;
const HUB_ATLAS_NAME: &str = "sacredgrove";

const fn hub(
    id: i32,
    name: &'static str,
    name_id: i32,
    location_id: i32,
    position: [f32; 3],
    heading: f32,
    unknown_11: i32,
    teleport_location_id: i32,
    unknown_15: i32,
) -> CuratedRow {
    CuratedRow {
        id,
        name,
        name_id,
        location_id,
        position,
        heading,
        unknown_11,
        teleport_location_id,
        unknown_15,
    }
}

/// (R) The curated set: the eight original zone hubs (ids 3-10, the range the
/// client treats as zone changes in `ScriptsBase.lua`) plus Lavender Coast
/// Pass. All lie on the zone-0 terrain grid, within 0.2 m of its height.
const CURATED: [CuratedRow; 9] = [
    hub(
        3,
        "Snowhill",
        3961,
        281,
        [51.391, 30.987, 363.156],
        1.487,
        651,
        210020,
        60,
    ),
    hub(
        4,
        "Briarwood",
        18140,
        282,
        [-808.386, -21.029, 1631.789],
        0.14,
        651,
        210021,
        90,
    ),
    hub(
        5,
        "Sanctuary",
        3499,
        283,
        [-1414.636, -27.631, 351.567],
        2.247,
        651,
        210022,
        20,
    ),
    hub(
        6,
        "Greenwood Forest",
        428995,
        221686,
        [-1583.1395, -45.37643, -547.8314],
        -0.21685,
        651,
        581,
        30,
    ),
    hub(
        7,
        "Cobblestone Village",
        18142,
        285,
        [-1904.46, -39.79, 413.316],
        0.596,
        0,
        210023,
        10,
    ),
    hub(
        8,
        "Shrouded Glade",
        3502,
        11292,
        [1.167, 41.038, 1114.959],
        2.477,
        651,
        210024,
        85,
    ),
    hub(
        9,
        "Lakeshore",
        18899,
        11293,
        [-733.952, 9.09, 945.973],
        -2.673,
        651,
        210025,
        60,
    ),
    hub(
        10,
        "Briarheart",
        4029,
        11294,
        [44.487, -30.922, 2804.612],
        -0.876,
        651,
        210026,
        95,
    ),
    hub(
        15,
        "Lavender Coast Pass",
        41393,
        11299,
        [-2124.768, -23.903, -292.377],
        1.694,
        651,
        210030,
        30,
    ),
];

fn curated_definition(row: &CuratedRow) -> PointOfInterestDefinition {
    PointOfInterestDefinition {
        id: row.id,
        name_id: row.name_id,
        location_id: row.location_id,
        x: row.position[0],
        y: row.position[1],
        z: row.position[2],
        heading: row.heading,
        icon_id: HUB_ICON_ID,
        notification_type: HUB_NOTIFICATION_TYPE,
        sub_name_id: HUB_SUB_NAME_ID,
        unknown_11: row.unknown_11,
        breadcrumb_quest_id: 0,
        teleport_location_id: row.teleport_location_id,
        atlas_name: HUB_ATLAS_NAME.to_string(),
        unknown_15: row.unknown_15,
    }
}

/// The curated hubs as wire rows, every one (the registry lists only attuned
/// ones). Exposed so a private gate can compare them with the captured `57`.
pub fn curated_definitions() -> Vec<PointOfInterestDefinition> {
    CURATED.iter().map(curated_definition).collect()
}

/// Spawn for a curated hub. ORIGINAL where the captures hold the original
/// server's spawn for that id (Lakeshore, Lavender Coast Pass); otherwise the
/// hub's own row position with the character's facing kept (R). The arrival
/// height is finished by the server's terrain in [`WarpRegistry`].
fn curated_destination(row: &CuratedRow) -> WarpDestination {
    if let Some(captured) = captured_destinations()
        .into_iter()
        .find(|captured| captured.id == row.id)
    {
        return WarpDestination {
            name: row.name,
            ..captured
        };
    }
    WarpDestination {
        id: row.id,
        name: row.name,
        is_warpstone: true,
        position: [row.position[0], row.position[1], row.position[2], 1.0],
        rotation: [0.0; 4],
        facing_known: false,
    }
}

/// (R) Travel tunables. Defaults are guesses nobody recovered; the sanity
/// bounds in [`WarpRules::checked`] only keep a bad data file from disabling
/// the gates or locking everyone out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpRules {
    /// A warpstone attunes when the character is this close (planar metres).
    /// The server's existing interaction radius is 12 m.
    pub attune_radius_m: f32,
    /// Minimum gap between two successful teleports of one character.
    pub cooldown_ms: u64,
    /// Minimum gap between two "I am stuck" safe teleports (`122`) that the
    /// server cannot verify as a fall below the world.
    pub stuck_cooldown_ms: u64,
    /// A character this far under the terrain counts as fallen out of the world.
    pub below_world_m: f32,
    /// L039 (R): a saved pose this far under the terrain is moved at world
    /// entry. The grid keeps the highest surface, and zone 0's mines and crypts
    /// lie up to 188 m under it (103 spawns in `npc_spawns_extra.json` are over
    /// 50 m under; deepest: 100000002757 at -904, -91, 2478), so the default
    /// stays clear of every known interior.
    pub login_below_world_m: f32,
}

impl Default for WarpRules {
    fn default() -> Self {
        Self {
            attune_radius_m: 12.0,
            cooldown_ms: 5_000,
            stuck_cooldown_ms: 300_000,
            below_world_m: 3.0,
            login_below_world_m: 250.0,
        }
    }
}

impl WarpRules {
    pub fn checked(self) -> Result<Self, &'static str> {
        if !(1.0..=100.0).contains(&self.attune_radius_m) {
            return Err("attune radius must be 1-100 m");
        }
        if self.cooldown_ms > 3_600_000 || self.stuck_cooldown_ms > 3_600_000 {
            return Err("cooldowns must be at most one hour");
        }
        if !(0.5..=100.0).contains(&self.below_world_m) {
            return Err("below-world margin must be 0.5-100 m");
        }
        if !(3.0..=500.0).contains(&self.login_below_world_m) {
            return Err("login below-world margin must be 3-500 m");
        }
        Ok(self)
    }
}

/// (R) Arrival is lifted this far above the terrain; the client grounds a
/// character from above, never from below. Same value as `realm goto`.
const ARRIVAL_LIFT_M: f32 = 0.5;

/// (R) L039: no terrain under the pose but terrain this far away on both sides
/// along x or along z means a hole in the grid (a runtime object stands in it),
/// not the edge of the map. In `lab/terrain/zone_0.hgt` the 18 small holes are
/// at most 32 m across and every edge void or missing chunk is 252 m or more.
const HOLE_PROBE_M: f32 = 40.0;

/// Why the character may not teleport right now. The caller derives it from
/// live state it owns (encounters, crafting/minigame movies, knockout).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocker {
    KnockedOut,
    Encounter,
    Busy,
}

/// What the registry may know about the requester: all of it server state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TravelContext {
    pub guid: u64,
    pub zone_id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub blocked: Option<Blocker>,
}

/// A teleport target the caller already verified is an online friend of the
/// requester. The registry still checks zone and the target's own state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FriendView {
    pub zone_id: u32,
    pub position: [f32; 3],
    pub blocked: bool,
}

/// Server-owned lookups the registry needs: terrain height at (x, z) in the
/// open world, and the friend view for a guid (`None` when the guid is not an
/// online friend of the requester).
pub struct WarpWorld<'a> {
    pub ground: &'a dyn Fn(f32, f32) -> Option<f32>,
    pub friend: &'a dyn Fn(u64) -> Option<FriendView>,
}

/// Where to put the character. The gateway turns this into the same
/// `38/12` update and saved position as a developer teleport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarpRelocation {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

/// Why a request produced no relocation. Every variant is a refusal the caller
/// must surface; none of them moves the character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpRejection {
    MalformedRequest(WarpCodecError),
    UnknownDestination(i32),
    LockedDestination(i32),
    NotAWarpstone(i32),
    WrongZone(u32),
    Blocked(Blocker),
    Cooldown,
    SelfTarget,
    TargetUnavailable,
}

impl WarpRejection {
    /// Player-facing text. Unknown, locked and non-warpstone ids share one
    /// message, and every friend refusal shares another, so a probing client
    /// learns nothing about ids or other players. Malformed bodies get none.
    pub fn notice(&self) -> Option<&'static str> {
        Some(match self {
            WarpRejection::MalformedRequest(_) => return None,
            WarpRejection::UnknownDestination(_)
            | WarpRejection::LockedDestination(_)
            | WarpRejection::NotAWarpstone(_) => "You have not attuned to that warpstone yet.",
            WarpRejection::WrongZone(_) => "Warpstones only work in the open world.",
            WarpRejection::Blocked(Blocker::KnockedOut) => {
                "You cannot use a warpstone while knocked out."
            }
            WarpRejection::Blocked(_) => "Finish what you are doing before using a warpstone.",
            WarpRejection::Cooldown => "The warpstone needs a moment to recharge.",
            WarpRejection::SelfTarget | WarpRejection::TargetUnavailable => {
                "You cannot teleport to that player."
            }
        })
    }
}

/// Everything a handled request produces. `replies` are inner packets (the
/// gateway wraps them), `notices` are system chat lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WarpReply {
    pub replies: Vec<Vec<u8>>,
    pub notices: Vec<String>,
    pub relocation: Option<WarpRelocation>,
    /// Persisted state changed (a new attunement): the caller must save.
    pub changed: bool,
    pub rejection: Option<WarpRejection>,
}

impl WarpReply {
    fn refused(rejection: WarpRejection) -> Self {
        Self {
            notices: rejection.notice().map(str::to_owned).into_iter().collect(),
            rejection: Some(rejection),
            ..Self::default()
        }
    }
}

/// The part of the registry that survives relog (inside the character's
/// gameplay save). Cooldowns are session-only (R).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarpState {
    #[serde(default)]
    pub attuned: BTreeSet<i32>,
}

impl WarpState {
    pub fn is_empty(&self) -> bool {
        self.attuned.is_empty()
    }
}

/// Destination table plus one character's attunement and cooldowns.
///
/// The gates are RECONSTRUCTED policy (see [`WarpRules`]): a warpstone must be
/// attuned (visited) before travel (shipped locale 41300; community wiki
/// "Warpstones"). The request layouts and the destination rows are recovered.
#[derive(Clone, Debug, Default)]
pub struct WarpRegistry {
    destinations: Vec<WarpDestination>,
    definitions: Vec<PointOfInterestDefinition>,
    attuned: BTreeSet<i32>,
    rules: WarpRules,
    last_travel_ms: Option<u64>,
    last_stuck_ms: Option<u64>,
}

impl WarpRegistry {
    pub fn new(destinations: Vec<WarpDestination>) -> Self {
        Self {
            destinations,
            ..Self::default()
        }
    }

    /// The curated warpstone set with its client-visible rows.
    pub fn curated() -> Self {
        Self {
            destinations: CURATED.iter().map(curated_destination).collect(),
            definitions: curated_definitions(),
            ..Self::default()
        }
    }

    pub fn set_rules(&mut self, rules: WarpRules) -> Result<(), &'static str> {
        self.rules = rules.checked()?;
        Ok(())
    }

    /// Marks a destination attuned. Returns whether the set changed, mirroring
    /// the once-only attunement notice.
    pub fn attune(&mut self, destination_id: i32) -> bool {
        self.attuned.insert(destination_id)
    }

    pub fn is_attuned(&self, destination_id: i32) -> bool {
        self.attuned.contains(&destination_id)
    }

    pub fn destination(&self, destination_id: i32) -> Option<&WarpDestination> {
        self.destinations
            .iter()
            .find(|entry| entry.id == destination_id)
    }

    pub fn state(&self) -> WarpState {
        WarpState {
            attuned: self.attuned.clone(),
        }
    }

    /// Loads a saved set, keeping only ids that are warpstones here (a save
    /// from an older table or a hand-edited file cannot unlock anything else).
    pub fn restore(&mut self, state: &WarpState) {
        self.attuned = state
            .attuned
            .iter()
            .copied()
            .filter(|id| self.destination(*id).is_some_and(|d| d.is_warpstone))
            .collect();
    }

    /// The full `57` message listing the attuned warpstones. The client replaces
    /// its whole table with it (`00b98b70` clears, then reads), so this is also
    /// the update after a new attunement.
    pub fn definitions_message(&self) -> Vec<u8> {
        let rows: Vec<_> = self
            .definitions
            .iter()
            .filter(|row| self.attuned.contains(&row.id))
            .cloned()
            .collect();
        point_of_interest_reply_message(&rows)
    }

    /// Position-driven attunement: standing within the radius of a warpstone in
    /// the open world attunes it. The server's own position, never a client
    /// claim; call it after a movement update was accepted.
    pub fn visit(&mut self, ctx: &TravelContext) -> WarpReply {
        if ctx.zone_id != OPEN_WORLD_ZONE || !ctx.position.iter().all(|v| v.is_finite()) {
            return WarpReply::default();
        }
        let radius_squared = self.rules.attune_radius_m * self.rules.attune_radius_m;
        let fresh: Vec<(i32, &'static str)> = self
            .destinations
            .iter()
            .filter(|d| d.is_warpstone && !self.attuned.contains(&d.id))
            .filter(|d| {
                let dx = d.position[0] - ctx.position[0];
                let dz = d.position[2] - ctx.position[2];
                dx * dx + dz * dz <= radius_squared
            })
            .map(|d| (d.id, d.name))
            .collect();
        if fresh.is_empty() {
            return WarpReply::default();
        }
        self.attuned.extend(fresh.iter().map(|(id, _)| *id));
        WarpReply {
            replies: vec![self.definitions_message()],
            notices: fresh
                .iter()
                .map(|(_, name)| format!("You are now attuned to the {name} Warpstone."))
                .collect(),
            changed: true,
            ..WarpReply::default()
        }
    }

    /// Routes one client request. `None` means the opcode is not a warpstone
    /// opcode; every handled request returns `Some`, refused or not.
    pub fn handle(
        &mut self,
        inner: &[u8],
        ctx: &TravelContext,
        now_ms: u64,
        world: &WarpWorld,
    ) -> Option<WarpReply> {
        let opcode = u16::from_le_bytes(inner.get(..2)?.try_into().ok()?);
        let body = &inner[2..];
        let result = match opcode {
            POINT_OF_INTEREST_DEFINITION_REQUEST_OPCODE => self.definition_request(body),
            ZONE_TELEPORT_REQUEST_OPCODE => self.zone_teleport(body, ctx, now_ms, world),
            ZONE_SAFE_TELEPORT_REQUEST_OPCODE => self.safe_teleport(body, ctx, now_ms, world),
            WORLD_TELEPORT_REQUEST_OPCODE => self.friend_teleport(body, ctx, now_ms, world),
            _ => return None,
        };
        Some(result.unwrap_or_else(WarpReply::refused))
    }

    fn definition_request(&self, body: &[u8]) -> Result<WarpReply, WarpRejection> {
        expect_exact(body, 0).map_err(WarpRejection::MalformedRequest)?;
        Ok(WarpReply {
            replies: vec![self.definitions_message()],
            ..WarpReply::default()
        })
    }

    /// `90`: travel to an attuned warpstone from anywhere in the open world.
    fn zone_teleport(
        &mut self,
        body: &[u8],
        ctx: &TravelContext,
        now_ms: u64,
        world: &WarpWorld,
    ) -> Result<WarpReply, WarpRejection> {
        let request = ZoneTeleportRequest::decode(body).map_err(WarpRejection::MalformedRequest)?;
        let id = request.destination_id;
        let destination = *self
            .destination(id)
            .ok_or(WarpRejection::UnknownDestination(id))?;
        if !destination.is_warpstone {
            return Err(WarpRejection::NotAWarpstone(id));
        }
        if !self.is_attuned(id) {
            return Err(WarpRejection::LockedDestination(id));
        }
        self.check_context(ctx)?;
        self.check_cooldown(self.last_travel_ms, self.rules.cooldown_ms, now_ms)?;
        self.last_travel_ms = Some(now_ms);
        Ok(relocate(arrive_at_destination(&destination, ctx, world)))
    }

    /// `122`: the client sends it when it falls below the world floor, and a
    /// stuck button may send it too. The destination is always the nearest
    /// warpstone to the server's own position (R); the request carries no
    /// position. A fall the server can verify against the terrain only shares
    /// the ordinary cooldown; otherwise the long stuck cooldown applies.
    fn safe_teleport(
        &mut self,
        body: &[u8],
        ctx: &TravelContext,
        now_ms: u64,
        world: &WarpWorld,
    ) -> Result<WarpReply, WarpRejection> {
        ZoneSafeTeleportRequest::decode(body).map_err(WarpRejection::MalformedRequest)?;
        self.check_context(ctx)?;
        let fallen = (world.ground)(ctx.position[0], ctx.position[2])
            .is_some_and(|ground| ctx.position[1] < ground - self.rules.below_world_m);
        self.check_cooldown(self.last_travel_ms, self.rules.cooldown_ms, now_ms)?;
        if !fallen {
            self.check_cooldown(self.last_stuck_ms, self.rules.stuck_cooldown_ms, now_ms)?;
        }
        let nearest = self
            .nearest_warpstone(ctx.position)
            .ok_or(WarpRejection::TargetUnavailable)?;
        if !fallen {
            self.last_stuck_ms = Some(now_ms);
        }
        self.last_travel_ms = Some(now_ms);
        Ok(relocate(arrive_at_destination(&nearest, ctx, world)))
    }

    /// The warpstone nearest to `position` in the plane (R): the `122` target.
    fn nearest_warpstone(&self, position: [f32; 3]) -> Option<WarpDestination> {
        let distance = |d: &WarpDestination| {
            (d.position[0] - position[0]).powi(2) + (d.position[2] - position[2]).powi(2)
        };
        self.destinations
            .iter()
            .filter(|d| d.is_warpstone)
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .copied()
    }

    /// L039 (R): at world entry, an open-world pose more than
    /// `login_below_world_m` under the terrain, or with no terrain under it and
    /// not inside a hole (off the map), goes to the nearest warpstone exactly as
    /// a `122` does. `ground` is the zone's loaded terrain: the caller skips the
    /// check when none is loaded. Blockers and cooldowns do not apply, since
    /// the character is not in the world yet, and nothing here changes state.
    /// A recovered pose passes, so recovery is idempotent.
    pub fn recover_login_pose(
        &self,
        ctx: &TravelContext,
        ground: &dyn Fn(f32, f32) -> Option<f32>,
    ) -> Option<(WarpRelocation, &'static str)> {
        if ctx.zone_id != OPEN_WORLD_ZONE {
            return None;
        }
        let [x, y, z] = ctx.position;
        let hit = |dx: f32, dz: f32| ground(x + dx, z + dz).is_some();
        let d = HOLE_PROBE_M;
        let in_hole = (hit(d, 0.0) && hit(-d, 0.0)) || (hit(0.0, d) && hit(0.0, -d));
        let reason = match ground(x, z) {
            Some(height) if y < height - self.rules.login_below_world_m => "below_world",
            None if !in_hole => "off_map",
            _ => return None,
        };
        let world = WarpWorld {
            ground,
            friend: &|_| None,
        };
        let nearest = self.nearest_warpstone(ctx.position)?;
        Some((arrive_at_destination(&nearest, ctx, &world), reason))
    }

    /// `58`: go to an online friend in the same open world (R). The request
    /// names a guid and nothing else; the arrival is the friend's server-side
    /// position.
    fn friend_teleport(
        &mut self,
        body: &[u8],
        ctx: &TravelContext,
        now_ms: u64,
        world: &WarpWorld,
    ) -> Result<WarpReply, WarpRejection> {
        let request =
            WorldTeleportRequest::decode(body).map_err(WarpRejection::MalformedRequest)?;
        if request.player_guid == ctx.guid {
            return Err(WarpRejection::SelfTarget);
        }
        self.check_context(ctx)?;
        let friend = (world.friend)(request.player_guid).ok_or(WarpRejection::TargetUnavailable)?;
        if friend.zone_id != OPEN_WORLD_ZONE || friend.blocked {
            return Err(WarpRejection::TargetUnavailable);
        }
        self.check_cooldown(self.last_travel_ms, self.rules.cooldown_ms, now_ms)?;
        self.last_travel_ms = Some(now_ms);
        Ok(relocate(arrival(friend.position, None, ctx, world)))
    }

    fn check_context(&self, ctx: &TravelContext) -> Result<(), WarpRejection> {
        if ctx.zone_id != OPEN_WORLD_ZONE {
            return Err(WarpRejection::WrongZone(ctx.zone_id));
        }
        match ctx.blocked {
            Some(blocker) => Err(WarpRejection::Blocked(blocker)),
            None => Ok(()),
        }
    }

    fn check_cooldown(
        &self,
        last: Option<u64>,
        window_ms: u64,
        now_ms: u64,
    ) -> Result<(), WarpRejection> {
        match last {
            Some(last) if now_ms.saturating_sub(last) < window_ms => Err(WarpRejection::Cooldown),
            _ => Ok(()),
        }
    }
}

fn relocate(relocation: WarpRelocation) -> WarpReply {
    WarpReply {
        relocation: Some(relocation),
        ..WarpReply::default()
    }
}

fn arrive_at_destination(
    destination: &WarpDestination,
    ctx: &TravelContext,
    world: &WarpWorld,
) -> WarpRelocation {
    let [x, y, z, _] = destination.position;
    let [fx, fy, fz, _] = destination.rotation;
    arrival(
        [x, y, z],
        destination.facing_known.then_some([fx, fy, fz]),
        ctx,
        world,
    )
}

/// Arrival point: the stored height, raised to the server's terrain plus a
/// small lift when the terrain is higher; facing kept unless one is known.
fn arrival(
    position: [f32; 3],
    facing: Option<[f32; 3]>,
    ctx: &TravelContext,
    world: &WarpWorld,
) -> WarpRelocation {
    WarpRelocation {
        position: lifted(position, world),
        rotation: facing.unwrap_or(ctx.rotation),
    }
}

fn lifted(mut position: [f32; 3], world: &WarpWorld) -> [f32; 3] {
    if let Some(ground) = (world.ground)(position[0], position[2]).filter(|g| g.is_finite()) {
        position[1] = position[1].max(ground + ARRIVAL_LIFT_M);
    }
    position
}

impl WarpRegistry {
    /// L043 (`realm move`, admin only): the warpstone named `name` (ASCII case-insensitive),
    /// attuned or not, and its arrival by the 90/122 rule above. The facing is `None` where
    /// that rule keeps the moved character's own.
    pub fn named_arrival(
        &self,
        name: &str,
        world: &WarpWorld,
    ) -> Option<([f32; 3], Option<[f32; 3]>)> {
        let destination = self
            .destinations
            .iter()
            .find(|d| d.is_warpstone && d.name.eq_ignore_ascii_case(name))?;
        let [x, y, z, _] = destination.position;
        let [fx, fy, fz, _] = destination.rotation;
        Some((
            lifted([x, y, z], world),
            destination.facing_known.then_some([fx, fy, fz]),
        ))
    }
}
