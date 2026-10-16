//! Native kart progress payload codecs for the vehicle-race family (inner opcode 33).
//!
//! ORIGINAL facts implemented here:
//!
//! * subtype 14 `VehicleRacePacketUpdateVehicleRaceProgressData` — `FUN_00b9fc80` reads 73 bytes in
//!   this order, every read bounds-checked:
//!   `u64 racer guid @0x00`, `i32 slot @0x08`, `f32 @0x0c`, `f32 @0x10`, `i32 laps completed @0x14`,
//!   `u8 local-racer flag @0x18`, `i32 lap limit @0x19`, `f32 @0x1d`, `f32 best lap @0x21`,
//!   `f32 total time @0x25`, `u64 x4 @0x29/@0x31/@0x39/@0x41`.
//! * subtype 1 `VehicleRacePacketUpdateVehicleGameState` — one `i32` state (4 bytes), stored at
//!   `processor+0x194`; the capture carries `3, 4, 5, 6, 7, 8, 9`.
//! * subtype 12 `VehicleRacePacketDestroyProxiedVehicleRace` — one `u64` guid (8 bytes). It removes
//!   a racer; it is **not** a finish.
//! * the reader wrapper only reports success when it consumed the buffer exactly and did not
//!   truncate, so a payload of any other length is refused here.
//! * NaN: the parser raises its NaN flag (`packet+0x11`) for the two `f32` fields at `0x0c` and
//!   `0x10` only, and that flag does not change the wrapper's return value — NaN is flagged, not
//!   refused. FLT_MAX (`0x7f7fffff`) is observed valid in the private capture at `0x10` (6 frames)
//!   and `0x21` (5 frames).
//!
//! WHAT THIS MODULE DOES NOT DO. `Finished` is not on the wire: the client derives it as
//! `lap limit < laps completed + 1` in `VehicleRaceResultsDataSource` (row `+0x20`), which is not the
//! same expression as the results-screen trigger `lap limit <= laps completed` once `laps + 1`
//! wraps. `results_row_finished` and `results_screen_trigger` mirror those two derivations *for
//! reporting only*; no function here accepts a lap count, grants a reward, advances a phase, or
//! decides that a race is over. Decoding a frame with
//! `laps_completed >= lap_limit` proves only that the server sent that number.
//!
//! Field meaning that the preserved sources do not recover stays positional: `unknown_0c`,
//! `unknown_10` (stored to racer+0x1614 while the game state is 9 and used by the sort predicate),
//! `unknown_1d` and the four trailing `u64`s. Float fields are stored as raw `u32` bit patterns so
//! that NaN, negative zero, denormals and FLT_MAX all round-trip byte for byte.
//!
//! Framing is not duplicated here: `kart_racing::decode_racing_frame` already decodes the tunnel
//! header, the inner opcode 33 and the subtype, and the owned test round-trips capture frames through
//! both. Payloads are the input to this module.
//!
//! ponytail: three flat structs and a cursor-free fixed-offset reader — no trait objects, no
//! schema framework, no builder.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ORIGINAL subtype: one `i32` game state.
pub const GAME_STATE_SUBTYPE: u16 = 1;
/// ORIGINAL subtype: the 73-byte progress block.
pub const PROGRESS_SUBTYPE: u16 = 14;
/// ORIGINAL subtype: one `u64` racer guid removal.
pub const DESTROY_RACER_SUBTYPE: u16 = 12;

/// ORIGINAL payload length of subtype 1.
pub const GAME_STATE_PAYLOAD_LEN: usize = 4;
/// ORIGINAL payload length of subtype 14 (the reader consumes exactly this many bytes).
pub const PROGRESS_PAYLOAD_LEN: usize = 73;
/// ORIGINAL payload length of subtype 12.
pub const DESTROY_RACER_PAYLOAD_LEN: usize = 8;

/// Payload-relative offsets of subtype 14, kept next to the reader for cross-checking.
pub const PROGRESS_GUID_OFFSET: usize = 0x00;
pub const PROGRESS_SLOT_OFFSET: usize = 0x08;
pub const PROGRESS_UNKNOWN_0C_OFFSET: usize = 0x0c;
pub const PROGRESS_UNKNOWN_10_OFFSET: usize = 0x10;
pub const PROGRESS_LAPS_OFFSET: usize = 0x14;
pub const PROGRESS_LOCAL_FLAG_OFFSET: usize = 0x18;
pub const PROGRESS_LAP_LIMIT_OFFSET: usize = 0x19;
pub const PROGRESS_UNKNOWN_1D_OFFSET: usize = 0x1d;
pub const PROGRESS_BEST_LAP_OFFSET: usize = 0x21;
pub const PROGRESS_TOTAL_TIME_OFFSET: usize = 0x25;
pub const PROGRESS_TAIL_U64_OFFSETS: [usize; 4] = [0x29, 0x31, 0x39, 0x41];

/// The two payload floats the native reader NaN-checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressFloatField {
    Unknown0c,
    Unknown10,
}

impl ProgressFloatField {
    pub fn name(self) -> &'static str {
        match self {
            ProgressFloatField::Unknown0c => "0x0c",
            ProgressFloatField::Unknown10 => "0x10",
        }
    }
}

/// RECONSTRUCTED policy knob, not a recovered rule.
///
/// `Preserve` is the native-faithful reading: every bit pattern round-trips and nothing is judged.
///
/// `RefuseNaN` is a stricter local rule with a deliberately narrow scope: it refuses a NaN in
/// exactly the two fields the native parser flags (`0x0c` and `0x10`). It does **not** reject
/// infinities, does not range-check any value, and does not inspect `0x1d`, `0x21` or `0x25` — a NaN
/// in those three fields is preserved under both policies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NonFinitePolicy {
    Preserve,
    RefuseNaN,
}

/// ORIGINAL subtype-14 layout with unrecovered fields kept positional and floats kept as raw bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressPayload {
    pub racer_guid: u64,
    pub slot: i32,
    /// Bit pattern of the `f32` at `0x0c`; NaN-checked natively, no recovered consumer.
    pub unknown_0c_bits: u32,
    /// Bit pattern of the `f32` at `0x10`; stored to racer+0x1614 while the game state is 9 and
    /// used by the results sort predicate. Meaning unrecovered.
    pub unknown_10_bits: u32,
    pub laps_completed: i32,
    pub local_racer: u8,
    pub lap_limit: i32,
    /// Bit pattern of the `f32` at `0x1d`; meaning unrecovered.
    pub unknown_1d_bits: u32,
    /// Bit pattern of the `f32` at `0x21` (`VehicleRaceResultsDataSource` BestLapTime).
    pub best_lap_bits: u32,
    /// Bit pattern of the `f32` at `0x25` (`VehicleRaceResultsDataSource` TotalTime).
    pub total_time_bits: u32,
    pub tail_0x29: u64,
    pub tail_0x31: u64,
    pub tail_0x39: u64,
    pub tail_0x41: u64,
}

impl ProgressPayload {
    pub fn unknown_0c(&self) -> f32 {
        f32::from_bits(self.unknown_0c_bits)
    }

    pub fn unknown_10(&self) -> f32 {
        f32::from_bits(self.unknown_10_bits)
    }

    pub fn unknown_1d(&self) -> f32 {
        f32::from_bits(self.unknown_1d_bits)
    }

    pub fn best_lap_time(&self) -> f32 {
        f32::from_bits(self.best_lap_bits)
    }

    pub fn total_time(&self) -> f32 {
        f32::from_bits(self.total_time_bits)
    }
}

/// Typed refusal. Length refusals are native behaviour (exact consumption); `NonFinite` is the
/// reconstructed strict policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProgressCodecError {
    WrongLength { expected: usize, actual: usize },
    NonFinite { field: ProgressFloatField },
}

impl fmt::Display for ProgressCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgressCodecError::WrongLength { expected, actual } => {
                write!(f, "payload must be {expected} bytes, got {actual}")
            }
            ProgressCodecError::NonFinite { field } => {
                write!(f, "float at {} is not finite", field.name())
            }
        }
    }
}

impl std::error::Error for ProgressCodecError {}

fn require_len(payload: &[u8], expected: usize) -> Result<(), ProgressCodecError> {
    if payload.len() != expected {
        return Err(ProgressCodecError::WrongLength {
            expected,
            actual: payload.len(),
        });
    }
    Ok(())
}

fn u32_at(payload: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        payload[offset],
        payload[offset + 1],
        payload[offset + 2],
        payload[offset + 3],
    ])
}

fn i32_at(payload: &[u8], offset: usize) -> i32 {
    u32_at(payload, offset) as i32
}

fn u64_at(payload: &[u8], offset: usize) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&payload[offset..offset + 8]);
    u64::from_le_bytes(bytes)
}

/// Decodes one subtype-14 payload. Length must be exactly 73 bytes.
pub fn decode_progress(
    payload: &[u8],
    policy: NonFinitePolicy,
) -> Result<ProgressPayload, ProgressCodecError> {
    require_len(payload, PROGRESS_PAYLOAD_LEN)?;
    let decoded = ProgressPayload {
        racer_guid: u64_at(payload, PROGRESS_GUID_OFFSET),
        slot: i32_at(payload, PROGRESS_SLOT_OFFSET),
        unknown_0c_bits: u32_at(payload, PROGRESS_UNKNOWN_0C_OFFSET),
        unknown_10_bits: u32_at(payload, PROGRESS_UNKNOWN_10_OFFSET),
        laps_completed: i32_at(payload, PROGRESS_LAPS_OFFSET),
        local_racer: payload[PROGRESS_LOCAL_FLAG_OFFSET],
        lap_limit: i32_at(payload, PROGRESS_LAP_LIMIT_OFFSET),
        unknown_1d_bits: u32_at(payload, PROGRESS_UNKNOWN_1D_OFFSET),
        best_lap_bits: u32_at(payload, PROGRESS_BEST_LAP_OFFSET),
        total_time_bits: u32_at(payload, PROGRESS_TOTAL_TIME_OFFSET),
        tail_0x29: u64_at(payload, PROGRESS_TAIL_U64_OFFSETS[0]),
        tail_0x31: u64_at(payload, PROGRESS_TAIL_U64_OFFSETS[1]),
        tail_0x39: u64_at(payload, PROGRESS_TAIL_U64_OFFSETS[2]),
        tail_0x41: u64_at(payload, PROGRESS_TAIL_U64_OFFSETS[3]),
    };

    if policy == NonFinitePolicy::RefuseNaN {
        for (field, bits) in [
            (ProgressFloatField::Unknown0c, decoded.unknown_0c_bits),
            (ProgressFloatField::Unknown10, decoded.unknown_10_bits),
        ] {
            if f32::from_bits(bits).is_nan() {
                return Err(ProgressCodecError::NonFinite { field });
            }
        }
    }

    Ok(decoded)
}

/// Encodes one subtype-14 payload. Always exactly 73 bytes; every field is written verbatim.
pub fn encode_progress(payload: &ProgressPayload) -> Vec<u8> {
    let mut out = Vec::with_capacity(PROGRESS_PAYLOAD_LEN);
    out.extend(payload.racer_guid.to_le_bytes());
    out.extend(payload.slot.to_le_bytes());
    out.extend(payload.unknown_0c_bits.to_le_bytes());
    out.extend(payload.unknown_10_bits.to_le_bytes());
    out.extend(payload.laps_completed.to_le_bytes());
    out.push(payload.local_racer);
    out.extend(payload.lap_limit.to_le_bytes());
    out.extend(payload.unknown_1d_bits.to_le_bytes());
    out.extend(payload.best_lap_bits.to_le_bytes());
    out.extend(payload.total_time_bits.to_le_bytes());
    out.extend(payload.tail_0x29.to_le_bytes());
    out.extend(payload.tail_0x31.to_le_bytes());
    out.extend(payload.tail_0x39.to_le_bytes());
    out.extend(payload.tail_0x41.to_le_bytes());
    debug_assert_eq!(out.len(), PROGRESS_PAYLOAD_LEN);
    out
}

/// Decodes one subtype-1 game-state payload (one `i32`).
pub fn decode_game_state(payload: &[u8]) -> Result<i32, ProgressCodecError> {
    require_len(payload, GAME_STATE_PAYLOAD_LEN)?;
    Ok(i32_at(payload, 0))
}

/// Encodes one subtype-1 game-state payload. No state value is interpreted.
pub fn encode_game_state(state: i32) -> Vec<u8> {
    state.to_le_bytes().to_vec()
}

/// Decodes one subtype-12 destroy-racer payload (one `u64` guid). Removal, not a finish.
pub fn decode_destroy_racer(payload: &[u8]) -> Result<u64, ProgressCodecError> {
    require_len(payload, DESTROY_RACER_PAYLOAD_LEN)?;
    Ok(u64_at(payload, 0))
}

/// Encodes one subtype-12 destroy-racer payload.
pub fn encode_destroy_racer(racer_guid: u64) -> Vec<u8> {
    racer_guid.to_le_bytes().to_vec()
}

/// READ-ONLY REPORTING HELPER for `VehicleRaceResultsDataSource` row `+0x20`. The recovered native
/// expression is `lap limit < laps completed + 1`, evaluated on 32-bit signed ints, so the addition
/// wraps at `i32::MAX`.
///
/// It authorizes nothing. The server must not treat a decoded frame as a finish, a reward or a
/// phase change, and no caller may use this to accept a result.
pub fn results_row_finished(laps_completed: i32, lap_limit: i32) -> bool {
    lap_limit < laps_completed.wrapping_add(1)
}

/// READ-ONLY REPORTING HELPER for the results-screen trigger `FUN_00ba6530`:
/// `processor+0x68 (lap limit) <= racer+0x15ec (laps completed)`.
///
/// This is **not** the data-source row predicate: it differs from `results_row_finished` when
/// `laps_completed + 1` wraps at `i32::MAX`. It authorizes nothing either.
pub fn results_screen_trigger(laps_completed: i32, lap_limit: i32) -> bool {
    lap_limit <= laps_completed
}
