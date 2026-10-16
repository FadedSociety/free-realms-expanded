//! Piece 077 - warpstones and travel: recovered warpstone request/destination
//! wire layouts plus the reconstructed locked-destination gate.
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
//! * ORIGINAL (preserved live captures): [`CAPTURED_TELEPORT_SAMPLES`] are the
//!   eight real request frames and their `38/12` replies.
//! * RECONSTRUCTED: [`WarpRegistry`]'s attunement gate. The shipped locale
//!   (41300, "You are now attuned to the Solace Warpstone.") and the community
//!   wiki describe activation; no captured frame proves a server rejection, so
//!   the gate is policy this restoration owns, not recovered behavior.
//!
//! Nothing here grants items, currency or zone changes on its own: the caller
//! decides what to do with a resolved destination.

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
    Truncated { expected: usize, got: usize },
    TrailingBytes { expected: usize, got: usize },
    MissingTerminator,
    NegativeLength { offset: usize, length: i32 },
    InvalidAsciiString { offset: usize },
    UnexpectedSubOpcode { expected: u16, got: u16 },
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
    /// Recovered rows for non-warpstone points of interest are legitimately in
    /// the client's destination list; only warpstones travel.
    pub is_warpstone: bool,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
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
        is_warpstone: true,
        position,
        rotation,
    })
    .collect()
}

/// Why a teleport request produced no reply. Every variant is a rejection the
/// caller must surface; none of them moves the character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarpRejection {
    MalformedRequest(WarpCodecError),
    UnknownDestination(i32),
    LockedDestination(i32),
    NotAWarpstone(i32),
}

/// Destination table plus the character's attunement set.
///
/// The gate is RECONSTRUCTED policy: a warpstone must be attuned before travel
/// (shipped locale 41300; community wiki "Warpstones"). The request layout and
/// the reply bytes it emits are the recovered ones.
#[derive(Clone, Debug, Default)]
pub struct WarpRegistry {
    destinations: Vec<WarpDestination>,
    attuned: BTreeSet<i32>,
}

impl WarpRegistry {
    pub fn new(destinations: Vec<WarpDestination>) -> Self {
        Self {
            destinations,
            attuned: BTreeSet::new(),
        }
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

    /// Validates a `90` request body and answers with the `38/12` message body.
    /// `trailer` is the caller-supplied byte the captured replies carry; this
    /// module does not invent its meaning.
    pub fn handle_zone_teleport(&self, body: &[u8], trailer: u8) -> Result<Vec<u8>, WarpRejection> {
        let request = ZoneTeleportRequest::decode(body).map_err(WarpRejection::MalformedRequest)?;
        let destination = self
            .destination(request.destination_id)
            .ok_or(WarpRejection::UnknownDestination(request.destination_id))?;
        if !destination.is_warpstone {
            return Err(WarpRejection::NotAWarpstone(request.destination_id));
        }
        if !self.is_attuned(request.destination_id) {
            return Err(WarpRejection::LockedDestination(request.destination_id));
        }
        Ok(UpdateLocation {
            position: destination.position,
            rotation: destination.rotation,
            teleport: true,
            trailer,
        }
        .encode_message())
    }
}
