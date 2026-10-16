//! Native kart lobby definition codec: opcode 102 sub 2.
//!
//! ORIGINAL facts implemented here (nothing about field *meaning* is invented):
//!
//! * framing — tunnel `u16 outer (5 or 6) | u8 flag | u32 inner_length`, inner
//!   `u16 opcode 102 | u16 sub 2 | i32 body_length | body`, where `inner_length = 8 + body_length`.
//!   `FUN_008d67a0` reads the two header words and rejects anything whose sub is not 2;
//!   `FUN_008bfc00` reads the `i32` body length and requires `cursor + length <= end`, so the native
//!   reader bounds the body but does not require the buffer to end there.
//! * body — five tables in a fixed order, each introduced by an `i32` row count:
//!   `games  u32 key + i32 id + i32 game_type + i32 unknown + i32 name_id`,
//!   `options u32 key + [i32; 3]`, `values u32 key + [i32; 3]`,
//!   `display u32 key + i32 unknown + i32 locale_id + i32 byte_length + byte_length bytes`,
//!   `settings u32 key + [i32; 5]`.
//! * the private capture used by the ignored gate: 2393 raw bytes, body 2378 bytes, counts
//!   `[6, 30, 43, 37, 17]`, body sha256 `51c8f011…`. Captured catalogue rows are test content.
//!
//! Unrecovered and therefore not named here: what the options/values triples mean, what the display
//! row's second word is, what the games row's fourth word is, and the settings field semantics beyond
//! their wire order. Those positions stay `unknown`/positional.
//!
//! Trailing-byte policy is explicit because the client's tolerance is **not** proven: the preserved
//! capture is an exact-consumption frame, and the independent review lists "tolerance for trailing
//! bytes after the five tables" as unverified. `TrailingBytes::Strict` refuses any byte after the
//! declared body and any byte after the five tables; `TrailingBytes::Tolerant` accepts both and
//! reports how many were ignored. Neither mode is a claim about the native client.
//!
//! This module does **not** replace `kart_racing::LobbyGameDefinition`, which mirrors the pinned
//! community writer's 140-byte subset (six rows plus four zero counts). Routing the gateway response
//! through this codec is the lead's integration step.
//!
//! ponytail: one cursor, five straight-line table readers, no trait objects and no schema framework.
//! Add a table or a field only when a recovered source names one.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ORIGINAL: outer tunnel opcodes observed for the lobby response.
pub const TUNNEL_OUTER_OPCODES: [u16; 2] = [5, 6];
/// ORIGINAL: tunnel header size (`u16 outer`, `u8 flag`, `u32 inner_length`).
pub const TUNNEL_HEADER_LEN: usize = 7;
/// ORIGINAL: inner header size (`u16 opcode`, `u16 sub`, `i32 body_length`).
pub const LOBBY_HEADER_LEN: usize = 8;
/// ORIGINAL: the lobby definition opcode.
pub const LOBBY_INNER_OPCODE: u16 = 102;
/// ORIGINAL: the lobby definition sub-opcode that `FUN_008d67a0` requires.
pub const LOBBY_INNER_SUB: u16 = 2;
/// ORIGINAL: fixed bytes per games row.
pub const GAMES_ROW_BYTES: usize = 20;
/// ORIGINAL: fixed bytes per options row.
pub const OPTIONS_ROW_BYTES: usize = 16;
/// ORIGINAL: fixed bytes per values row.
pub const VALUES_ROW_BYTES: usize = 16;
/// ORIGINAL: fixed bytes per display row, before its length-prefixed bytes.
pub const DISPLAY_ROW_FIXED_BYTES: usize = 16;
/// ORIGINAL: fixed bytes per settings row.
pub const SETTINGS_ROW_BYTES: usize = 24;
/// ORIGINAL: five `i32` counts make up an empty body.
pub const EMPTY_BODY_BYTES: usize = 5 * 4;

/// The five tables, in wire order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyTable {
    Games,
    Options,
    Values,
    Display,
    Settings,
}

impl LobbyTable {
    pub fn name(self) -> &'static str {
        match self {
            LobbyTable::Games => "games",
            LobbyTable::Options => "options",
            LobbyTable::Values => "values",
            LobbyTable::Display => "display",
            LobbyTable::Settings => "settings",
        }
    }
}

/// Where the ignored bytes sat when a tolerant decode accepted them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrailingPosition {
    /// After the five tables but inside the declared body.
    Body,
    /// After the declared body but inside the supplied frame.
    Frame,
}

/// RECONSTRUCTED policy: the client's tolerance is unproven, so the caller chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrailingBytes {
    Strict,
    Tolerant,
}

/// ORIGINAL games row: `u32 key` then four `i32`. Row 1 of the captured test catalogue is
/// `(1, 1, 1, 37, 3030)`; the fourth word has no recovered meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyGamesRow {
    pub key: u32,
    pub id: i32,
    pub game_type: i32,
    pub unknown: i32,
    pub name_id: i32,
}

/// ORIGINAL options row: `u32 key` then three `i32` with unrecovered meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyOptionsRow {
    pub key: u32,
    pub unknown: [i32; 3],
}

/// ORIGINAL values row: `u32 key` then three `i32` with unrecovered meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyValuesRow {
    pub key: u32,
    pub unknown: [i32; 3],
}

/// ORIGINAL display row: `u32 key`, one `i32` with unrecovered meaning, the locale id and a
/// length-prefixed byte string (the capture carries single ASCII digits).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyDisplayRow {
    pub key: u32,
    pub unknown: i32,
    pub locale_id: i32,
    pub text: Vec<u8>,
}

/// ORIGINAL settings row: `u32 key` then five `i32` in wire order. The captured rows carry a name
/// id in the fourth word, but only the order is recovered, so the fields stay positional.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbySettingsRow {
    pub key: u32,
    pub fields: [i32; 5],
}

/// ORIGINAL body: the five tables in wire order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LobbyNativeBody {
    pub games: Vec<LobbyGamesRow>,
    pub options: Vec<LobbyOptionsRow>,
    pub values: Vec<LobbyValuesRow>,
    pub display: Vec<LobbyDisplayRow>,
    pub settings: Vec<LobbySettingsRow>,
}

/// One decoded frame plus how many bytes a tolerant policy ignored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedLobby {
    pub body: LobbyNativeBody,
    pub trailing_bytes: usize,
}

/// Typed refusal. Every variant is reachable from a committed vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LobbyCodecError {
    TooShort {
        needed: usize,
        available: usize,
    },
    UnsupportedOuterOpcode(u16),
    InnerLengthMismatch {
        declared: usize,
        actual: usize,
    },
    UnsupportedInnerOpcode(u16),
    UnsupportedInnerSub(u16),
    NegativeBodyLength(i32),
    BodyLengthMismatch {
        declared: usize,
        available: usize,
    },
    NegativeCount {
        table: LobbyTable,
        count: i32,
    },
    CountExceedsRemaining {
        table: LobbyTable,
        count: usize,
        available: usize,
    },
    NegativeTextLength {
        row: usize,
        length: i32,
    },
    TextExceedsRemaining {
        row: usize,
        length: usize,
        available: usize,
    },
    TrailingBytes {
        position: TrailingPosition,
        count: usize,
    },
    FieldTooLarge {
        table: LobbyTable,
        length: usize,
    },
    BodyTooLarge {
        length: usize,
    },
}

impl fmt::Display for LobbyCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LobbyCodecError::TooShort { needed, available } => {
                write!(f, "frame needs {needed} bytes, {available} available")
            }
            LobbyCodecError::UnsupportedOuterOpcode(opcode) => {
                write!(f, "unsupported tunnel outer opcode {opcode}")
            }
            LobbyCodecError::InnerLengthMismatch { declared, actual } => {
                write!(
                    f,
                    "tunnel declares {declared} inner bytes, {actual} present"
                )
            }
            LobbyCodecError::UnsupportedInnerOpcode(opcode) => {
                write!(f, "unsupported inner opcode {opcode}")
            }
            LobbyCodecError::UnsupportedInnerSub(sub) => write!(f, "unsupported inner sub {sub}"),
            LobbyCodecError::NegativeBodyLength(length) => {
                write!(f, "negative body length {length}")
            }
            LobbyCodecError::BodyLengthMismatch {
                declared,
                available,
            } => {
                write!(f, "body declares {declared} bytes, {available} available")
            }
            LobbyCodecError::NegativeCount { table, count } => {
                write!(f, "negative {} row count {count}", table.name())
            }
            LobbyCodecError::CountExceedsRemaining {
                table,
                count,
                available,
            } => write!(
                f,
                "{} row count {count} cannot fit in {available} bytes",
                table.name()
            ),
            LobbyCodecError::NegativeTextLength { row, length } => {
                write!(f, "display row {row} has negative text length {length}")
            }
            LobbyCodecError::TextExceedsRemaining {
                row,
                length,
                available,
            } => write!(
                f,
                "display row {row} declares {length} text bytes, {available} available"
            ),
            LobbyCodecError::TrailingBytes { position, count } => match position {
                TrailingPosition::Body => write!(f, "{count} bytes after the five tables"),
                TrailingPosition::Frame => write!(f, "{count} bytes after the declared body"),
            },
            LobbyCodecError::FieldTooLarge { table, length } => {
                write!(f, "{} field length {length} exceeds i32", table.name())
            }
            LobbyCodecError::BodyTooLarge { length } => {
                write!(f, "body length {length} exceeds the framing field")
            }
        }
    }
}

impl std::error::Error for LobbyCodecError {}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Cursor { bytes, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], LobbyCodecError> {
        let needed = self.pos + length;
        if needed > self.bytes.len() {
            return Err(LobbyCodecError::TooShort {
                needed,
                available: self.bytes.len(),
            });
        }
        let slice = &self.bytes[self.pos..needed];
        self.pos = needed;
        Ok(slice)
    }

    fn i32(&mut self) -> Result<i32, LobbyCodecError> {
        let bytes = self.take(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u32(&mut self) -> Result<u32, LobbyCodecError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

fn read_count(
    cursor: &mut Cursor<'_>,
    table: LobbyTable,
    row_bytes: usize,
) -> Result<usize, LobbyCodecError> {
    let count = cursor.i32()?;
    if count < 0 {
        return Err(LobbyCodecError::NegativeCount { table, count });
    }
    let count = count as usize;
    let available = cursor.remaining();
    if count > available / row_bytes {
        return Err(LobbyCodecError::CountExceedsRemaining {
            table,
            count,
            available,
        });
    }
    Ok(count)
}

/// Decodes a body. Returns the body and the bytes ignored after the five tables.
pub fn decode_body(
    bytes: &[u8],
    policy: TrailingBytes,
) -> Result<(LobbyNativeBody, usize), LobbyCodecError> {
    let mut cursor = Cursor::new(bytes);

    let games_count = read_count(&mut cursor, LobbyTable::Games, GAMES_ROW_BYTES)?;
    let mut games = Vec::with_capacity(games_count);
    for _ in 0..games_count {
        games.push(LobbyGamesRow {
            key: cursor.u32()?,
            id: cursor.i32()?,
            game_type: cursor.i32()?,
            unknown: cursor.i32()?,
            name_id: cursor.i32()?,
        });
    }

    let options_count = read_count(&mut cursor, LobbyTable::Options, OPTIONS_ROW_BYTES)?;
    let mut options = Vec::with_capacity(options_count);
    for _ in 0..options_count {
        options.push(LobbyOptionsRow {
            key: cursor.u32()?,
            unknown: [cursor.i32()?, cursor.i32()?, cursor.i32()?],
        });
    }

    let values_count = read_count(&mut cursor, LobbyTable::Values, VALUES_ROW_BYTES)?;
    let mut values = Vec::with_capacity(values_count);
    for _ in 0..values_count {
        values.push(LobbyValuesRow {
            key: cursor.u32()?,
            unknown: [cursor.i32()?, cursor.i32()?, cursor.i32()?],
        });
    }

    let display_count = read_count(&mut cursor, LobbyTable::Display, DISPLAY_ROW_FIXED_BYTES)?;
    let mut display = Vec::with_capacity(display_count);
    for row in 0..display_count {
        let key = cursor.u32()?;
        let unknown = cursor.i32()?;
        let locale_id = cursor.i32()?;
        let length = cursor.i32()?;
        if length < 0 {
            return Err(LobbyCodecError::NegativeTextLength { row, length });
        }
        let length = length as usize;
        let available = cursor.remaining();
        if length > available {
            return Err(LobbyCodecError::TextExceedsRemaining {
                row,
                length,
                available,
            });
        }
        display.push(LobbyDisplayRow {
            key,
            unknown,
            locale_id,
            text: cursor.take(length)?.to_vec(),
        });
    }

    let settings_count = read_count(&mut cursor, LobbyTable::Settings, SETTINGS_ROW_BYTES)?;
    let mut settings = Vec::with_capacity(settings_count);
    for _ in 0..settings_count {
        settings.push(LobbySettingsRow {
            key: cursor.u32()?,
            fields: [
                cursor.i32()?,
                cursor.i32()?,
                cursor.i32()?,
                cursor.i32()?,
                cursor.i32()?,
            ],
        });
    }

    let trailing = cursor.remaining();
    if trailing > 0 && policy == TrailingBytes::Strict {
        return Err(LobbyCodecError::TrailingBytes {
            position: TrailingPosition::Body,
            count: trailing,
        });
    }

    Ok((
        LobbyNativeBody {
            games,
            options,
            values,
            display,
            settings,
        },
        trailing,
    ))
}

/// Decodes an inner 102/2 packet (`opcode | sub | i32 body_length | body`).
pub fn decode_inner(frame: &[u8], policy: TrailingBytes) -> Result<DecodedLobby, LobbyCodecError> {
    if frame.len() < LOBBY_HEADER_LEN {
        return Err(LobbyCodecError::TooShort {
            needed: LOBBY_HEADER_LEN,
            available: frame.len(),
        });
    }
    let opcode = u16::from_le_bytes([frame[0], frame[1]]);
    if opcode != LOBBY_INNER_OPCODE {
        return Err(LobbyCodecError::UnsupportedInnerOpcode(opcode));
    }
    let sub = u16::from_le_bytes([frame[2], frame[3]]);
    if sub != LOBBY_INNER_SUB {
        return Err(LobbyCodecError::UnsupportedInnerSub(sub));
    }
    let declared = i32::from_le_bytes([frame[4], frame[5], frame[6], frame[7]]);
    if declared < 0 {
        return Err(LobbyCodecError::NegativeBodyLength(declared));
    }
    let declared = declared as usize;
    let available = frame.len() - LOBBY_HEADER_LEN;
    if declared > available {
        return Err(LobbyCodecError::BodyLengthMismatch {
            declared,
            available,
        });
    }

    // The declared length bounds the body exactly; bytes after it are frame-level trailing data.
    let frame_trailing = available - declared;
    if frame_trailing > 0 && policy == TrailingBytes::Strict {
        return Err(LobbyCodecError::TrailingBytes {
            position: TrailingPosition::Frame,
            count: frame_trailing,
        });
    }
    let body_bytes = &frame[LOBBY_HEADER_LEN..LOBBY_HEADER_LEN + declared];
    let (body, body_trailing) = decode_body(body_bytes, policy)?;

    Ok(DecodedLobby {
        body,
        trailing_bytes: frame_trailing + body_trailing,
    })
}

/// Decodes a tunnel frame (`u16 outer | u8 flag | u32 inner_length | inner`).
pub fn decode_tunnel(frame: &[u8], policy: TrailingBytes) -> Result<DecodedLobby, LobbyCodecError> {
    if frame.len() < TUNNEL_HEADER_LEN {
        return Err(LobbyCodecError::TooShort {
            needed: TUNNEL_HEADER_LEN,
            available: frame.len(),
        });
    }
    let outer = u16::from_le_bytes([frame[0], frame[1]]);
    if !TUNNEL_OUTER_OPCODES.contains(&outer) {
        return Err(LobbyCodecError::UnsupportedOuterOpcode(outer));
    }
    let declared = u32::from_le_bytes([frame[3], frame[4], frame[5], frame[6]]) as usize;
    let actual = frame.len() - TUNNEL_HEADER_LEN;
    if declared != actual {
        return Err(LobbyCodecError::InnerLengthMismatch { declared, actual });
    }
    decode_inner(&frame[TUNNEL_HEADER_LEN..], policy)
}

fn write_count(out: &mut Vec<u8>, table: LobbyTable, count: usize) -> Result<(), LobbyCodecError> {
    let count = i32::try_from(count).map_err(|_| LobbyCodecError::FieldTooLarge {
        table,
        length: count,
    })?;
    out.extend(count.to_le_bytes());
    Ok(())
}

/// Encodes a body: five count-prefixed tables in wire order.
pub fn encode_body(body: &LobbyNativeBody) -> Result<Vec<u8>, LobbyCodecError> {
    let mut out = Vec::with_capacity(EMPTY_BODY_BYTES + body.games.len() * GAMES_ROW_BYTES);

    write_count(&mut out, LobbyTable::Games, body.games.len())?;
    for row in &body.games {
        out.extend(row.key.to_le_bytes());
        out.extend(row.id.to_le_bytes());
        out.extend(row.game_type.to_le_bytes());
        out.extend(row.unknown.to_le_bytes());
        out.extend(row.name_id.to_le_bytes());
    }

    write_count(&mut out, LobbyTable::Options, body.options.len())?;
    for row in &body.options {
        out.extend(row.key.to_le_bytes());
        for field in row.unknown {
            out.extend(field.to_le_bytes());
        }
    }

    write_count(&mut out, LobbyTable::Values, body.values.len())?;
    for row in &body.values {
        out.extend(row.key.to_le_bytes());
        for field in row.unknown {
            out.extend(field.to_le_bytes());
        }
    }

    write_count(&mut out, LobbyTable::Display, body.display.len())?;
    for row in &body.display {
        out.extend(row.key.to_le_bytes());
        out.extend(row.unknown.to_le_bytes());
        out.extend(row.locale_id.to_le_bytes());
        let length = i32::try_from(row.text.len()).map_err(|_| LobbyCodecError::FieldTooLarge {
            table: LobbyTable::Display,
            length: row.text.len(),
        })?;
        out.extend(length.to_le_bytes());
        out.extend(&row.text);
    }

    write_count(&mut out, LobbyTable::Settings, body.settings.len())?;
    for row in &body.settings {
        out.extend(row.key.to_le_bytes());
        for field in row.fields {
            out.extend(field.to_le_bytes());
        }
    }

    Ok(out)
}

/// Encodes the inner 102/2 packet, including the `i32` body length.
pub fn encode_inner(body: &LobbyNativeBody) -> Result<Vec<u8>, LobbyCodecError> {
    let payload = encode_body(body)?;
    let length = i32::try_from(payload.len()).map_err(|_| LobbyCodecError::BodyTooLarge {
        length: payload.len(),
    })?;
    let mut out = Vec::with_capacity(LOBBY_HEADER_LEN + payload.len());
    out.extend(LOBBY_INNER_OPCODE.to_le_bytes());
    out.extend(LOBBY_INNER_SUB.to_le_bytes());
    out.extend(length.to_le_bytes());
    out.extend(payload);
    Ok(out)
}

/// Encodes the tunnel frame the gateway sends. `outer` is 5 or 6 in the preserved traffic.
pub fn encode_tunnel(
    outer: u16,
    flag: u8,
    body: &LobbyNativeBody,
) -> Result<Vec<u8>, LobbyCodecError> {
    if !TUNNEL_OUTER_OPCODES.contains(&outer) {
        return Err(LobbyCodecError::UnsupportedOuterOpcode(outer));
    }
    let inner = encode_inner(body)?;
    let length = u32::try_from(inner.len()).map_err(|_| LobbyCodecError::BodyTooLarge {
        length: inner.len(),
    })?;
    let mut out = Vec::with_capacity(TUNNEL_HEADER_LEN + inner.len());
    out.extend(outer.to_le_bytes());
    out.push(flag);
    out.extend(length.to_le_bytes());
    out.extend(inner);
    Ok(out)
}
