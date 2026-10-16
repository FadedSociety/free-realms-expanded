//! Kart weapon packet codecs and a bounded local visual lifecycle.
//!
//! Layout source: `lab/reports/kart-weapon-layouts-20261009/REPORT.md` (reader chains and byte
//! widths) plus its `scripts/verify-weapon-layouts.py` checker. Every field keeps the recovered
//! packet-object offset in its name; unlabelled words stay unlabelled and are never guessed into
//! semantics. Subtypes:
//!
//! * `24` `CreateProxiedVehicleRaceMissile` (88 B), `25` destroy missile (4 B),
//!   `26` `UpdateVehicleRaceMissileData` (116 B);
//! * `27` `CreateProxiedVehicleRaceExplosion` (48 B), `28` destroy explosion (4 B);
//! * `29` `CreateProxiedVehicleRaceMine` (80 B), `30` destroy mine (4 B),
//!   `31` `UpdateVehicleRaceMineData` (68 B).
//!
//! Recovery status (do not overclaim): the payloads are reader-exact, but **no captured frame
//! exists for any of these subtypes**, the lifetimes are not recovered, and the transform axis
//! order is resolved: the reader proved a row-major 4x4 (basis rows 0..11, translation in slots
//! 12/13/14, `1.0` in slot 15) against the known pickup matrix. The `24`/`29` create matrix is
//! **owner-local**; the missile `26` update is a direct **world** matrix (`FUN_00fe2850` consumes the
//! full matrix); the mine `31` update is the **inverse-owner local** matrix so a moving owner never
//! moves the dropped mine. `use_item_with_matrix` stays available for content that wants to supply
//! an exact transform. The lifecycle labels every reconstructed number ([`MISSILE_TTL_MS`],
//! [`MINE_TTL_MS`], [`MISSILE_SPEED_MPS`], [`EXPLOSION_FUSE_MS`]). Hits are explicitly out of
//! scope: there is no raw client hit relay and no victim targeting here, because the native hit path
//! is unresolved.
//!
//! Open items from the lead review: the heading row is `basis[2]` (third row, byte offset 24), not
//! `basis[1]`; the create frames' owner words carry the session guid (the native create path looks
//! up the owner's real kart placement); the mine ships a world-stationary transform until the
//! reader confirms the owner-relative variant; and **no explosion `27` is emitted by the lifecycle
//! yet** - its unlabelled `+0x10/+0x18/scalars` are unresolved and a zero-filled placeholder would
//! drive the actor consumer with a fake asset (documented on [`WeaponState::tick`]).
//!
//! This module is self-contained (std + serde only) so `tests/kart_weapons.rs` can compile it with
//! `#[path]` before the lead registers `pub mod kart_weapons;` in `lib.rs`. `inner_frame` matches
//! `kart_entry::encode_inner` (`u16` opcode 33 then `u16` subtype, little-endian).

use serde::{Deserialize, Serialize};
use std::fmt;

/// ORIGINAL: the vehicle-race inner opcode every frame in this family is wrapped in.
pub const INNER_OPCODE: u16 = 33;

/// ORIGINAL: client `VehicleRacePacketCreateProxiedVehicleRaceMissile`.
pub const MISSILE_CREATE_SUBTYPE: u16 = 24;
/// ORIGINAL: client `VehicleRacePacketDestroyProxiedVehicleRaceMissile`.
pub const MISSILE_DESTROY_SUBTYPE: u16 = 25;
/// ORIGINAL: client `VehicleRacePacketUpdateVehicleRaceMissileData`.
pub const MISSILE_UPDATE_SUBTYPE: u16 = 26;
/// ORIGINAL: client `VehicleRacePacketCreateProxiedVehicleRaceExplosion`.
pub const EXPLOSION_CREATE_SUBTYPE: u16 = 27;
/// ORIGINAL: client `VehicleRacePacketDestroyProxiedVehicleRaceExplosion`.
pub const EXPLOSION_DESTROY_SUBTYPE: u16 = 28;
/// ORIGINAL: client `VehicleRacePacketCreateProxiedVehicleRaceMine`.
pub const MINE_CREATE_SUBTYPE: u16 = 29;
/// ORIGINAL: client `VehicleRacePacketDestroyProxiedVehicleRaceMine`.
pub const MINE_DESTROY_SUBTYPE: u16 = 30;
/// ORIGINAL: client `VehicleRacePacketUpdateVehicleRaceMineData`.
pub const MINE_UPDATE_SUBTYPE: u16 = 31;

/// ORIGINAL: reader-exact payload widths (see the report's layout table).
pub const MISSILE_CREATE_PAYLOAD_LEN: usize = 88;
pub const MISSILE_DESTROY_PAYLOAD_LEN: usize = 4;
pub const MISSILE_UPDATE_PAYLOAD_LEN: usize = 116;
pub const EXPLOSION_CREATE_PAYLOAD_LEN: usize = 48;
pub const EXPLOSION_DESTROY_PAYLOAD_LEN: usize = 4;
pub const MINE_CREATE_PAYLOAD_LEN: usize = 80;
pub const MINE_DESTROY_PAYLOAD_LEN: usize = 4;
pub const MINE_UPDATE_PAYLOAD_LEN: usize = 68;

/// ORIGINAL asset ids (community `Models.txt`, first column): missile actor and race mine actor.
pub const MISSILE_MODEL_ID: u32 = 466;
pub const MINE_MODEL_ID: u32 = 481;

/// RECONSTRUCTED local lifecycle bounds. None of these is a retail measurement; the native report
/// found no TTL constant and no captured frame for these subtypes.
pub const MISSILE_TTL_MS: u64 = 4_000;
pub const MINE_TTL_MS: u64 = 12_000;
/// RECONSTRUCTED missile speed used to integrate the update frames.
pub const MISSILE_SPEED_MPS: f32 = 35.0;
/// RECONSTRUCTED local create offsets: the corrected reader confirms the create matrix is **local**
/// to the owner, so the rocket spawns three metres ahead and the mine three metres behind in the
/// owner's frame. The local "forward" axis is our row-2 convention until a native probe proves it.
pub const MISSILE_CREATE_FORWARD_M: f32 = 3.0;
pub const MINE_CREATE_FORWARD_M: f32 = -3.0;
/// RECONSTRUCTED update cadence: the report allows an update stream at most 10 Hz.
pub const MISSILE_UPDATE_INTERVAL_MS: u64 = 100;
/// RECONSTRUCTED local fuse: how long an explosion actor lives before `28` removes it.
pub const EXPLOSION_FUSE_MS: u64 = 800;
/// Bounds for a restored weapon's spawn..expiry span.
pub const MIN_WEAPON_TTL_MS: u64 = 250;
pub const MAX_WEAPON_TTL_MS: u64 = 60_000;
/// Bounds: live weapons, live effects and monotonically allocated ids per race.
pub const MAX_LIVE_WEAPONS: usize = 16;
pub const MAX_LIVE_EFFECTS: usize = 16;
pub const MAX_TOTAL_ALLOCATIONS: u32 = 128;
/// Tolerance for the orthonormality checks (float32 basis rows from the client).
pub const BASIS_TOLERANCE: f32 = 1e-3;

/// Typed failure: malformed wire input, bad content, or a refused lifecycle action.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponError {
    PayloadLength {
        subtype: u16,
        expected: usize,
        actual: usize,
    },
    NonFinite {
        subtype: u16,
        field: &'static str,
    },
    UnsupportedSubtype {
        subtype: u16,
    },
    /// The state already holds the live-weapon or live-effect bound.
    TooManyLive {
        count: usize,
        max: usize,
    },
    /// The race exhausted its monotonic id budget.
    IdBudgetExhausted {
        allocated: u32,
        max: u32,
    },
    /// A state entry names id 0 or an id beyond the allocation budget.
    IdOutOfRange {
        id: u32,
    },
    /// The same id appears twice: an internal replay or a tampered save.
    DuplicateId {
        id: u32,
    },
    /// The stored counter cannot produce the stored entries (a tampered save).
    BadCounter {
        next_id: u32,
    },
    /// A state entry belongs to another player than the session's guid.
    ForeignOwner {
        owner: u64,
        expected: u64,
    },
    /// The caller passed a zero owner guid (the session guid must be non-zero).
    ZeroOwner,
    /// A stored expiry sits outside the race window.
    OutsideWindow {
        id: u32,
        expires_ms: u64,
    },
    /// The caller asked to use an item at or after the race window end.
    RaceEnded {
        now_ms: u64,
        race_end_ms: u64,
    },
    /// The caller's clock predates the last lifecycle tick (a regression, never a free spawn).
    ClockRegression {
        now_ms: u64,
        last_tick_ms: u64,
    },
    /// A stored weapon's spawn/expiry span is outside the reconstructed TTL bounds.
    BadTtlSpan {
        id: u32,
        spawned_ms: u64,
        expires_ms: u64,
    },
    /// The stored tick clock is outside the race window.
    BadClock {
        last_tick_ms: u64,
    },
    /// A pose value is not finite.
    PoseNotFinite {
        field: &'static str,
    },
    /// A pose basis is not orthonormal.
    PoseNotOrthonormal,
    /// A pose basis is left-handed or degenerate.
    PoseNotRightHanded,
    /// The cached observation is stamped in the future.
    FuturePose {
        at_ms: u64,
        now_ms: u64,
    },
}

impl fmt::Display for WeaponError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeaponError::PayloadLength {
                subtype,
                expected,
                actual,
            } => write!(
                f,
                "weapon subtype {subtype} payload must be {expected} bytes, got {actual}"
            ),
            WeaponError::NonFinite { subtype, field } => {
                write!(f, "weapon subtype {subtype} field {field} is not finite")
            }
            WeaponError::UnsupportedSubtype { subtype } => {
                write!(f, "subtype {subtype} is not a kart weapon frame")
            }
            WeaponError::TooManyLive { count, max } => {
                write!(f, "weapon state holds {count} entries, the bound is {max}")
            }
            WeaponError::IdBudgetExhausted { allocated, max } => {
                write!(f, "weapon id budget exhausted: {allocated} of {max}")
            }
            WeaponError::IdOutOfRange { id } => write!(f, "weapon id {id} is out of range"),
            WeaponError::DuplicateId { id } => write!(f, "weapon id {id} is already live"),
            WeaponError::BadCounter { next_id } => {
                write!(f, "weapon id counter {next_id} is incoherent")
            }
            WeaponError::ForeignOwner { owner, expected } => {
                write!(
                    f,
                    "weapon owner {owner:016x} is not the session {expected:016x}"
                )
            }
            WeaponError::ZeroOwner => write!(f, "weapon owner guid must be non-zero"),
            WeaponError::OutsideWindow { id, expires_ms } => write!(
                f,
                "weapon {id} expiry {expires_ms} is outside the race window"
            ),
            WeaponError::RaceEnded {
                now_ms,
                race_end_ms,
            } => write!(
                f,
                "weapon use at {now_ms} is at or past the race end {race_end_ms}"
            ),
            WeaponError::ClockRegression {
                now_ms,
                last_tick_ms,
            } => write!(
                f,
                "weapon use at {now_ms} predates the last tick {last_tick_ms}"
            ),
            WeaponError::BadTtlSpan {
                id,
                spawned_ms,
                expires_ms,
            } => write!(
                f,
                "weapon {id} span {spawned_ms}..{expires_ms} is outside the TTL bounds"
            ),
            WeaponError::BadClock { last_tick_ms } => {
                write!(
                    f,
                    "weapon tick clock {last_tick_ms} is outside the race window"
                )
            }
            WeaponError::PoseNotFinite { field } => {
                write!(f, "weapon pose {field} is not finite")
            }
            WeaponError::PoseNotOrthonormal => write!(f, "weapon pose basis is not orthonormal"),
            WeaponError::PoseNotRightHanded => write!(f, "weapon pose basis is not right-handed"),
            WeaponError::FuturePose { at_ms, now_ms } => {
                write!(f, "weapon pose stamped {at_ms} is ahead of {now_ms}")
            }
        }
    }
}

impl std::error::Error for WeaponError {}

/// ORIGINAL layout `24`: `u32 +0x0c | u32 +0x10 | vec4 +0x20 | vec4 +0x30 | vec4 +0x40 |
/// vec4 +0x50 | f32 +0x60 (NaN-checked) | u64 +0x68 | u32 +0x70`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissileCreate {
    pub id: u32,
    /// Second u32; the report labels the asset-id role a hypothesis (missile actor 466).
    pub model: u32,
    /// The four vec4s (`+0x20..+0x5f`) passed through raw; the consumer uses this as its transform
    /// source and the axis order is unproven.
    pub matrix: [f32; 16],
    /// `+0x60` (NaN-checked by the reader); unit unrecovered.
    pub scalar_0x60: f32,
    /// `+0x68`; unlabelled.
    pub field_0x68: u64,
    /// `+0x70`; unlabelled.
    pub field_0x70: u32,
}

/// ORIGINAL layout `26`: `u32 +0x0c | u64 +0x10 | vec4 +0x20 | vec4 +0x30 | vec4 +0x40 |
/// vec4 +0x50 | vec4 +0x60 | vec4 +0x70 | u64 +0x80`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissileUpdate {
    pub id: u32,
    /// `+0x10`; unlabelled.
    pub field_0x10: u64,
    /// `+0x20..+0x5f`: the full world 4x4. `FUN_00fe2850` reads indices 0..=14 and ignores 15.
    pub matrix: [f32; 16],
    /// `+0x60..+0x6f`: the first half of the two PTR VEC3 spans (`matrix[15] + tail_0x60[0..2]` and
    /// `tail_0x60[2..4] + tail_0x70[0]`), not independent scalars.
    pub tail_0x60: [f32; 4],
    /// `+0x70..+0x7f`: the tail of the second PTR VEC3 span.
    pub tail_0x70: [f32; 4],
    /// `+0x80`; unlabelled.
    pub field_0x80: u64,
}

/// ORIGINAL layout `27`: `u32 +0x0c | u32 +0x10 | u64 +0x18 | vec4 +0x20 |
/// f32 +0x30 (NaN) | f32 +0x34 (NaN) | f32 +0x38 | f32 +0x3c`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplosionCreate {
    pub id: u32,
    /// `+0x10`; corrected reader: a flag (1 vs 0), not a model id.
    pub field_0x10: u32,
    /// `+0x18`; corrected reader: the owner racer guid.
    pub field_0x18: u64,
    /// `+0x20` raw; the consumer copies this vec4 as the effect's position source.
    pub block_0x20: [f32; 4],
    pub scalar_0x30: f32,
    pub scalar_0x34: f32,
    pub scalar_0x38: f32,
    pub scalar_0x3c: f32,
}

/// ORIGINAL layout `29`: `u32 +0x0c | u32 +0x10 | vec4 +0x20 | vec4 +0x30 | vec4 +0x40 |
/// vec4 +0x50 | u64 +0x60`. The consumer copies all sixteen floats verbatim into the actor.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MineCreate {
    pub id: u32,
    /// Second u32; asset-id role is a hypothesis (race mine actor 481).
    pub model: u32,
    pub matrix: [f32; 16],
    /// `+0x60`; unlabelled (the consumer reads its two words into actor fields).
    pub field_0x60: u64,
}

/// ORIGINAL layout `31`: `u32 +0x0c | vec4 +0x10 | vec4 +0x20 | vec4 +0x30 | vec4 +0x40`.
/// Note the vec4 run starts one word earlier than the missile update does.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MineUpdate {
    pub id: u32,
    pub matrix: [f32; 16],
}

/// The shared `u32` destroy shape used by `25`, `28` and `30`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponDestroy {
    pub id: u32,
}

struct Reader<'a> {
    subtype: u16,
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(subtype: u16, bytes: &'a [u8], expected: usize) -> Result<Self, WeaponError> {
        if bytes.len() != expected {
            return Err(WeaponError::PayloadLength {
                subtype,
                expected,
                actual: bytes.len(),
            });
        }
        Ok(Self {
            subtype,
            bytes,
            at: 0,
        })
    }

    fn u32(&mut self) -> u32 {
        let value = u32::from_le_bytes(self.bytes[self.at..self.at + 4].try_into().unwrap());
        self.at += 4;
        value
    }

    fn u64(&mut self) -> u64 {
        let value = u64::from_le_bytes(self.bytes[self.at..self.at + 8].try_into().unwrap());
        self.at += 8;
        value
    }

    fn f32(&mut self, field: &'static str) -> Result<f32, WeaponError> {
        let value = f32::from_le_bytes(self.bytes[self.at..self.at + 4].try_into().unwrap());
        self.at += 4;
        if !value.is_finite() {
            return Err(WeaponError::NonFinite {
                subtype: self.subtype,
                field,
            });
        }
        Ok(value)
    }

    fn f32s<const N: usize>(&mut self, field: &'static str) -> Result<[f32; N], WeaponError> {
        let mut values = [0.0f32; N];
        for index in 0..N {
            values[index] = self.f32(field)?;
        }
        Ok(values)
    }

    fn finish(self) -> Result<(), WeaponError> {
        debug_assert_eq!(self.at, self.bytes.len());
        Ok(())
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn new(len: usize) -> Self {
        Writer(Vec::with_capacity(len))
    }

    fn u32(&mut self, value: u32) {
        self.0.extend(value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.0.extend(value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.0.extend(value.to_le_bytes());
    }

    fn f32s<const N: usize>(&mut self, values: [f32; N]) {
        for value in values {
            self.f32(value);
        }
    }

    fn finish(self, expected: usize) -> Vec<u8> {
        debug_assert_eq!(self.0.len(), expected);
        self.0
    }
}

/// Inner frame: `u16` opcode 33, `u16` subtype, then the payload.
pub fn inner_frame(subtype: u16, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(4 + payload.len());
    frame.extend(INNER_OPCODE.to_le_bytes());
    frame.extend(subtype.to_le_bytes());
    frame.extend_from_slice(payload);
    frame
}

/// Identity basis with the translation in slots 12/13/14 - the corrected reader confirmed those
/// slots against the known pickup matrix.
pub fn local_offset_matrix(offset: [f32; 3]) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, offset[0], offset[1],
        offset[2], 1.0,
    ]
}

/// A full row-major 4x4: three basis rows then the translation in slots 12/13/14 and `1.0` in 15.
pub fn world_matrix(basis: [[f32; 3]; 3], position: [f32; 3]) -> [f32; 16] {
    [
        basis[0][0],
        basis[0][1],
        basis[0][2],
        0.0,
        basis[1][0],
        basis[1][1],
        basis[1][2],
        0.0,
        basis[2][0],
        basis[2][1],
        basis[2][2],
        0.0,
        position[0],
        position[1],
        position[2],
        1.0,
    ]
}

/// The missile `26` update matrix. `FUN_00fe2850` (2369967..238) reads indices 0..=14 and ignores
/// 15, while `FUN_00fe08a0` (+0x5c, 2368719..25) and `FUN_00fe0990` (+0x68, 2368763..69) are PTR
/// VEC3 setters spanning `matrix[15] + tail_0x60[0..2]` and `tail_0x60[2..4] + tail_0x70[0]`.
/// This lifecycle sends pure position updates at 10 Hz, so slot 15 and every tail word stay `0.0`:
/// a stray `1.0` in slot 15 would be read as an accidental 1 m/s speed by the first triplet. The
/// **create** matrix keeps `w = 1.0` (see [`local_offset_matrix`]).
pub fn missile_update_matrix(basis: [[f32; 3]; 3], position: [f32; 3]) -> [f32; 16] {
    let mut matrix = world_matrix(basis, position);
    matrix[15] = 0.0;
    matrix
}

pub fn encode_missile_create(value: &MissileCreate) -> Vec<u8> {
    let mut writer = Writer::new(MISSILE_CREATE_PAYLOAD_LEN);
    writer.u32(value.id);
    writer.u32(value.model);
    writer.f32s(value.matrix);
    writer.f32(value.scalar_0x60);
    writer.u64(value.field_0x68);
    writer.u32(value.field_0x70);
    writer.finish(MISSILE_CREATE_PAYLOAD_LEN)
}

pub fn decode_missile_create(payload: &[u8]) -> Result<MissileCreate, WeaponError> {
    let mut reader = Reader::new(MISSILE_CREATE_SUBTYPE, payload, MISSILE_CREATE_PAYLOAD_LEN)?;
    let value = MissileCreate {
        id: reader.u32(),
        model: reader.u32(),
        matrix: reader.f32s("matrix")?,
        scalar_0x60: reader.f32("scalar_0x60")?,
        field_0x68: reader.u64(),
        field_0x70: reader.u32(),
    };
    reader.finish()?;
    Ok(value)
}

pub fn encode_missile_update(value: &MissileUpdate) -> Vec<u8> {
    let mut writer = Writer::new(MISSILE_UPDATE_PAYLOAD_LEN);
    writer.u32(value.id);
    writer.u64(value.field_0x10);
    writer.f32s(value.matrix);
    writer.f32s(value.tail_0x60);
    writer.f32s(value.tail_0x70);
    writer.u64(value.field_0x80);
    writer.finish(MISSILE_UPDATE_PAYLOAD_LEN)
}

pub fn decode_missile_update(payload: &[u8]) -> Result<MissileUpdate, WeaponError> {
    let mut reader = Reader::new(MISSILE_UPDATE_SUBTYPE, payload, MISSILE_UPDATE_PAYLOAD_LEN)?;
    let value = MissileUpdate {
        id: reader.u32(),
        field_0x10: reader.u64(),
        matrix: reader.f32s("matrix")?,
        tail_0x60: reader.f32s("tail_0x60")?,
        tail_0x70: reader.f32s("tail_0x70")?,
        field_0x80: reader.u64(),
    };
    reader.finish()?;
    Ok(value)
}

pub fn encode_explosion_create(value: &ExplosionCreate) -> Vec<u8> {
    let mut writer = Writer::new(EXPLOSION_CREATE_PAYLOAD_LEN);
    writer.u32(value.id);
    writer.u32(value.field_0x10);
    writer.u64(value.field_0x18);
    writer.f32s(value.block_0x20);
    writer.f32(value.scalar_0x30);
    writer.f32(value.scalar_0x34);
    writer.f32(value.scalar_0x38);
    writer.f32(value.scalar_0x3c);
    writer.finish(EXPLOSION_CREATE_PAYLOAD_LEN)
}

pub fn decode_explosion_create(payload: &[u8]) -> Result<ExplosionCreate, WeaponError> {
    let mut reader = Reader::new(
        EXPLOSION_CREATE_SUBTYPE,
        payload,
        EXPLOSION_CREATE_PAYLOAD_LEN,
    )?;
    let value = ExplosionCreate {
        id: reader.u32(),
        field_0x10: reader.u32(),
        field_0x18: reader.u64(),
        block_0x20: reader.f32s("block_0x20")?,
        scalar_0x30: reader.f32("scalar_0x30")?,
        scalar_0x34: reader.f32("scalar_0x34")?,
        scalar_0x38: reader.f32("scalar_0x38")?,
        scalar_0x3c: reader.f32("scalar_0x3c")?,
    };
    reader.finish()?;
    Ok(value)
}

pub fn encode_mine_create(value: &MineCreate) -> Vec<u8> {
    let mut writer = Writer::new(MINE_CREATE_PAYLOAD_LEN);
    writer.u32(value.id);
    writer.u32(value.model);
    writer.f32s(value.matrix);
    writer.u64(value.field_0x60);
    writer.finish(MINE_CREATE_PAYLOAD_LEN)
}

pub fn decode_mine_create(payload: &[u8]) -> Result<MineCreate, WeaponError> {
    let mut reader = Reader::new(MINE_CREATE_SUBTYPE, payload, MINE_CREATE_PAYLOAD_LEN)?;
    let value = MineCreate {
        id: reader.u32(),
        model: reader.u32(),
        matrix: reader.f32s("matrix")?,
        field_0x60: reader.u64(),
    };
    reader.finish()?;
    Ok(value)
}

pub fn encode_mine_update(value: &MineUpdate) -> Vec<u8> {
    let mut writer = Writer::new(MINE_UPDATE_PAYLOAD_LEN);
    writer.u32(value.id);
    writer.f32s(value.matrix);
    writer.finish(MINE_UPDATE_PAYLOAD_LEN)
}

pub fn decode_mine_update(payload: &[u8]) -> Result<MineUpdate, WeaponError> {
    let mut reader = Reader::new(MINE_UPDATE_SUBTYPE, payload, MINE_UPDATE_PAYLOAD_LEN)?;
    let value = MineUpdate {
        id: reader.u32(),
        matrix: reader.f32s("matrix")?,
    };
    reader.finish()?;
    Ok(value)
}

pub fn encode_destroy(value: &WeaponDestroy) -> Vec<u8> {
    value.id.to_le_bytes().to_vec()
}

pub fn decode_destroy(subtype: u16, payload: &[u8]) -> Result<WeaponDestroy, WeaponError> {
    let mut reader = Reader::new(subtype, payload, MISSILE_DESTROY_PAYLOAD_LEN)?;
    let value = WeaponDestroy { id: reader.u32() };
    reader.finish()?;
    Ok(value)
}

/// ORIGINAL: the client's C33/6 transmitter sends nine basis floats and then the position at byte
/// offset 36, so the pose keeps the basis rows and the translation exactly as received.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponPose {
    /// Three rows of three floats, in the transmitter's order.
    pub basis: [[f32; 3]; 3],
    pub position: [f32; 3],
}

impl WeaponPose {
    /// Identity basis at the origin.
    pub fn identity() -> Self {
        Self {
            basis: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            position: [0.0; 3],
        }
    }

    /// The movement direction: `basis[2]`, the third row (byte offset 24) the lead confirmed.
    pub fn heading(&self) -> [f32; 3] {
        self.basis[2]
    }

    /// RECONSTRUCTED world-stationary transform for a dropped mine: identity basis plus the drop
    /// position, so the mine does not inherit the kart's orientation. The owner-relative inverse
    /// alternative is unproven and stays open until the reader confirms the create-slot fallthrough.
    pub fn world_stationary_matrix(&self) -> [f32; 16] {
        let p = self.position;
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, p[0], p[1], p[2], 1.0,
        ]
    }

    /// Every float finite, each row unit length, rows mutually orthogonal and the basis
    /// right-handed (determinant `+1` within [`BASIS_TOLERANCE`]).
    pub fn validate(&self) -> Result<(), WeaponError> {
        for (index, row) in self.basis.iter().enumerate() {
            if !row.iter().all(|value| value.is_finite()) {
                return Err(WeaponError::PoseNotFinite {
                    field: match index {
                        0 => "basis[0]",
                        1 => "basis[1]",
                        _ => "basis[2]",
                    },
                });
            }
        }
        if !self.position.iter().all(|value| value.is_finite()) {
            return Err(WeaponError::PoseNotFinite { field: "position" });
        }
        for row in self.basis {
            let length = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
            if (length - 1.0).abs() > BASIS_TOLERANCE {
                return Err(WeaponError::PoseNotOrthonormal);
            }
        }
        for (first, second) in [(0, 1), (0, 2), (1, 2)] {
            let a = self.basis[first];
            let b = self.basis[second];
            let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
            if dot.abs() > BASIS_TOLERANCE {
                return Err(WeaponError::PoseNotOrthonormal);
            }
        }
        let a = self.basis[0];
        let b = self.basis[1];
        let c = self.basis[2];
        let cross = [
            b[1] * c[2] - b[2] * c[1],
            b[2] * c[0] - b[0] * c[2],
            b[0] * c[1] - b[1] * c[0],
        ];
        let determinant = a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2];
        if determinant <= 0.0 || (determinant - 1.0).abs() > BASIS_TOLERANCE {
            return Err(WeaponError::PoseNotRightHanded);
        }
        Ok(())
    }

    /// PROVEN row-major 4x4: the three basis rows as sent, the position in slots 12/13/14 and `1.0`
    /// in slot 15 (the reader checked it against the known pickup matrix). This is the world
    /// matrix; the create path uses the owner-local [`local_offset_matrix`].
    pub fn matrix(&self) -> [f32; 16] {
        let b = self.basis;
        let p = self.position;
        [
            b[0][0], b[0][1], b[0][2], 0.0, b[1][0], b[1][1], b[1][2], 0.0, b[2][0], b[2][1],
            b[2][2], 0.0, p[0], p[1], p[2], 1.0,
        ]
    }
}

/// The lead's cached latest pose: validated once and refused when stale or future-stamped.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponObservation {
    pub pose: WeaponPose,
    pub at_ms: u64,
}

impl WeaponObservation {
    /// Validate the last accepted session placement without inventing a send cadence.
    /// Native idle C33/6 updates can be more than 40 s apart. The service owns
    /// the race window, ownership, reachability and reset on exit/reconnect.
    /// This cached placement drives visual actors only; it cannot establish hits.
    pub fn validate_for(&self, now_ms: u64) -> Result<(), WeaponError> {
        if self.at_ms > now_ms {
            return Err(WeaponError::FuturePose {
                at_ms: self.at_ms,
                now_ms,
            });
        }
        self.pose.validate()
    }
}

/// `world - owner` in world coordinates.
fn world_delta(world: [f32; 3], owner: [f32; 3]) -> [f32; 3] {
    [
        world[0] - owner[0],
        world[1] - owner[1],
        world[2] - owner[2],
    ]
}

/// Inverse-rotate a world delta into the owner's frame. An orthonormal basis is its own inverse
/// transpose, so each local component is the dot product with the matching basis row.
fn inverse_rotate(pose: &WeaponPose, delta: [f32; 3]) -> [f32; 3] {
    let basis = pose.basis;
    [
        basis[0][0] * delta[0] + basis[0][1] * delta[1] + basis[0][2] * delta[2],
        basis[1][0] * delta[0] + basis[1][1] * delta[1] + basis[1][2] * delta[2],
        basis[2][0] * delta[0] + basis[2][1] * delta[1] + basis[2][2] * delta[2],
    ]
}

/// Which local weapon an item use spawns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponKind {
    Missile,
    Mine,
}

impl WeaponKind {
    /// RECONSTRUCTED lifetime (no TTL constant was recovered).
    pub fn ttl_ms(self) -> u64 {
        match self {
            WeaponKind::Missile => MISSILE_TTL_MS,
            WeaponKind::Mine => MINE_TTL_MS,
        }
    }

    /// The create-frame model word (second u32; asset-id role is a labelled hypothesis).
    pub fn model(self) -> u32 {
        match self {
            WeaponKind::Missile => MISSILE_MODEL_ID,
            WeaponKind::Mine => MINE_MODEL_ID,
        }
    }

    pub fn destroy_subtype(self) -> u16 {
        match self {
            WeaponKind::Missile => MISSILE_DESTROY_SUBTYPE,
            WeaponKind::Mine => MINE_DESTROY_SUBTYPE,
        }
    }

    /// The reconstructed owner-local forward offset the create frame carries.
    pub fn create_forward_m(self) -> f32 {
        match self {
            WeaponKind::Missile => MISSILE_CREATE_FORWARD_M,
            WeaponKind::Mine => MINE_CREATE_FORWARD_M,
        }
    }
}

/// One live local weapon.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponRegistration {
    pub id: u32,
    pub kind: WeaponKind,
    /// The session player guid that fired it.
    pub owner: u64,
    pub spawned_ms: u64,
    pub expires_ms: u64,
    /// Next missile update is due here; mines are static and use `expires_ms`.
    pub next_update_ms: u64,
    pub position: [f32; 3],
    /// The pose basis the missile keeps while flying (row 2 is the heading; C33/6 order).
    pub basis: [[f32; 3]; 3],
    /// Stamp of the owner observation the last emitted update was computed from.
    #[serde(default)]
    pub last_owner_ms: u64,
}

/// One live local explosion effect awaiting its `28`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRegistration {
    pub id: u32,
    pub owner: u64,
    pub expires_ms: u64,
    pub position: [f32; 4],
}

/// Persisted bounded weapon state. Every field defaults, so content/saves written before weapons
/// restore as "nothing live". One entry per allocated id; ids are never reused within a race.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponState {
    #[serde(default)]
    pub weapons: Vec<WeaponRegistration>,
    #[serde(default)]
    pub effects: Vec<EffectRegistration>,
    /// Monotonic allocation counter (ids 1..=[`MAX_TOTAL_ALLOCATIONS`]).
    #[serde(default)]
    pub next_id: u32,
    /// Last tick clock, so a regressed tick is harmless.
    #[serde(default)]
    pub last_tick_ms: u64,
}

impl WeaponState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn live(&self) -> usize {
        self.weapons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.weapons.is_empty() && self.effects.is_empty()
    }

    /// True while this id belongs to the local race (foreign ids must be refused).
    pub fn is_owned(&self, id: u32) -> bool {
        self.weapons.iter().any(|weapon| weapon.id == id)
            || self.effects.iter().any(|effect| effect.id == id)
    }

    pub fn owner_of(&self, id: u32) -> Option<u64> {
        self.weapons
            .iter()
            .find(|weapon| weapon.id == id)
            .map(|weapon| weapon.owner)
            .or_else(|| {
                self.effects
                    .iter()
                    .find(|effect| effect.id == id)
                    .map(|effect| effect.owner)
            })
    }

    fn allocate(&mut self) -> Result<u32, WeaponError> {
        if self.next_id >= MAX_TOTAL_ALLOCATIONS {
            return Err(WeaponError::IdBudgetExhausted {
                allocated: self.next_id,
                max: MAX_TOTAL_ALLOCATIONS,
            });
        }
        self.next_id += 1;
        Ok(self.next_id)
    }

    /// Pure pre-check the caller runs before popping its item queue: every refusal a `use_item`
    /// could produce, without mutating or allocating anything.
    pub fn status(
        &self,
        owner: u64,
        observation: &WeaponObservation,
        now_ms: u64,
        race_end_ms: u64,
    ) -> Result<(), WeaponError> {
        if race_end_ms.saturating_sub(now_ms) < MIN_WEAPON_TTL_MS {
            return Err(WeaponError::RaceEnded {
                now_ms,
                race_end_ms,
            });
        }
        if now_ms < self.last_tick_ms {
            // A use stamped before the last lifecycle tick is a regression, never a free spawn.
            return Err(WeaponError::ClockRegression {
                now_ms,
                last_tick_ms: self.last_tick_ms,
            });
        }
        if self.weapons.len() >= MAX_LIVE_WEAPONS {
            return Err(WeaponError::TooManyLive {
                count: self.weapons.len(),
                max: MAX_LIVE_WEAPONS,
            });
        }
        if self.next_id >= MAX_TOTAL_ALLOCATIONS {
            return Err(WeaponError::IdBudgetExhausted {
                allocated: self.next_id,
                max: MAX_TOTAL_ALLOCATIONS,
            });
        }
        if owner == 0 {
            return Err(WeaponError::ZeroOwner);
        }
        // A duplicate id can only come from a tampered save; callers with a live id are replayed.
        if let Some(id) = self
            .weapons
            .iter()
            .map(|weapon| weapon.id)
            .find(|id| self.effects.iter().any(|effect| effect.id == *id))
        {
            return Err(WeaponError::DuplicateId { id });
        }
        observation.validate_for(now_ms)?;
        Ok(())
    }

    /// Spawns one local missile/mine and returns its single create frame (`24`/`29`).
    ///
    /// The create matrix is **local to the owner** (corrected reader evidence), so this uses an
    /// identity basis with a reconstructed forward offset - rocket
    /// [`MISSILE_CREATE_FORWARD_M`] ahead, mine [`MINE_CREATE_FORWARD_M`] behind - never the world
    /// matrix, which would double the owner's translation. Hits are not modelled; only spawn,
    /// movement and expiry are claimed.
    pub fn use_item(
        &mut self,
        kind: WeaponKind,
        owner: u64,
        observation: &WeaponObservation,
        now_ms: u64,
        race_end_ms: u64,
    ) -> Result<Vec<Vec<u8>>, WeaponError> {
        let matrix = match kind {
            WeaponKind::Missile => local_offset_matrix([0.0, 0.0, MISSILE_CREATE_FORWARD_M]),
            WeaponKind::Mine => local_offset_matrix([0.0, 0.0, MINE_CREATE_FORWARD_M]),
        };
        self.use_item_with_matrix(kind, owner, observation, matrix, now_ms, race_end_ms)
    }

    /// Spawns one local missile/mine with a caller-supplied `[f32; 16]` transform. The pose is
    /// still recorded for ownership/movement, but the wire matrix is exactly what the caller
    /// passes - this is the entry point for the corrected reader evidence.
    pub fn use_item_with_matrix(
        &mut self,
        kind: WeaponKind,
        owner: u64,
        observation: &WeaponObservation,
        matrix: [f32; 16],
        now_ms: u64,
        race_end_ms: u64,
    ) -> Result<Vec<Vec<u8>>, WeaponError> {
        self.status(owner, observation, now_ms, race_end_ms)?;
        if !matrix.iter().all(|value| value.is_finite()) {
            return Err(WeaponError::NonFinite {
                subtype: match kind {
                    WeaponKind::Missile => MISSILE_CREATE_SUBTYPE,
                    WeaponKind::Mine => MINE_CREATE_SUBTYPE,
                },
                field: "matrix",
            });
        }
        let pose = observation.pose;
        let id = self.allocate()?;
        let expires_ms = now_ms.saturating_add(kind.ttl_ms()).min(race_end_ms);
        let heading = pose.basis[2];
        let forward = kind.create_forward_m();
        // The world spawn point the owner-local create offset resolves to, so the first world-space
        // update continues from where the actor actually appeared.
        let spawn_position = [
            pose.position[0] + heading[0] * forward,
            pose.position[1] + heading[1] * forward,
            pose.position[2] + heading[2] * forward,
        ];
        let registration = WeaponRegistration {
            id,
            kind,
            owner,
            spawned_ms: now_ms,
            expires_ms,
            next_update_ms: now_ms
                .saturating_add(MISSILE_UPDATE_INTERVAL_MS)
                .min(expires_ms),
            position: spawn_position,
            basis: pose.basis,
            last_owner_ms: observation.at_ms,
        };
        let frame = match kind {
            WeaponKind::Missile => inner_frame(
                MISSILE_CREATE_SUBTYPE,
                &encode_missile_create(&MissileCreate {
                    id,
                    model: MISSILE_MODEL_ID,
                    matrix,
                    // RECONSTRUCTED/HYPOTHESIS defaults from the report's probe template.
                    scalar_0x60: 1.0,
                    // Owner guid: the native create path looks up the owner's real kart placement.
                    field_0x68: owner,
                    field_0x70: 0,
                }),
            ),
            WeaponKind::Mine => inner_frame(
                MINE_CREATE_SUBTYPE,
                &encode_mine_create(&MineCreate {
                    id,
                    model: MINE_MODEL_ID,
                    matrix,
                    // Owner guid (the mine's only u64 word).
                    field_0x60: owner,
                }),
            ),
        };
        self.weapons.push(registration);
        Ok(vec![frame])
    }

    fn destroy_frame(kind: WeaponKind, id: u32) -> Vec<u8> {
        inner_frame(
            kind.destroy_subtype(),
            &encode_destroy(&WeaponDestroy { id }),
        )
    }

    /// Updates then terminations: at most 10 Hz `26`/`31` frames, then the weapon's `25`/`30`
    /// destroy exactly once at expiry. A regressed clock is ignored.
    ///
    /// The corrected reader proves the transforms are owner-local (row-major, translation in slots
    /// 12/13/14) and that the update consumer routes only the first vec4 plus `+0x5c` and `+0x68`
    /// into setters, so the last accepted owner observation is inverted into each weapon's local frame:
    /// missiles integrate their world position along `basis[2]` first, mines stay world-stationary
    /// and emit `31` so the owner-relative transform tracks the moving owner. Without a valid
    /// observation a missile falls back to its world translation and a mine skips the update.
    ///
    /// OPEN (lead review): expiry does **not** emit the `27` explosion yet. The `+0x10` flag (0 =
    /// world path) and `+0x18` owner racer word are now known, but the four tail setter scalars are
    /// not, and a placeholder would drive the effect consumer with fake data.
    pub fn tick(&mut self, now_ms: u64, observation: Option<&WeaponObservation>) -> Vec<Vec<u8>> {
        if now_ms < self.last_tick_ms {
            return Vec::new();
        }
        self.last_tick_ms = now_ms;
        let accepted = observation.filter(|observation| observation.validate_for(now_ms).is_ok());
        let mut updates = Vec::new();
        let mut terminations = Vec::new();
        let mut survivors = Vec::with_capacity(self.weapons.len());
        let drained = std::mem::take(&mut self.weapons);
        for mut weapon in drained {
            if now_ms >= weapon.expires_ms {
                if weapon.kind == WeaponKind::Missile {
                    // A late tick integrates only the leg up to the expiry, so the terminal point
                    // is where the missile actually died; publish it before the destroy.
                    let last_integrated = weapon
                        .next_update_ms
                        .saturating_sub(MISSILE_UPDATE_INTERVAL_MS)
                        .max(weapon.spawned_ms);
                    let remaining_ms = weapon.expires_ms.saturating_sub(last_integrated);
                    if remaining_ms > 0 {
                        let step = MISSILE_SPEED_MPS * (remaining_ms as f32 / 1_000.0);
                        let heading = weapon.basis[2];
                        for axis in 0..3 {
                            weapon.position[axis] += heading[axis] * step;
                        }
                        updates.push(inner_frame(
                            MISSILE_UPDATE_SUBTYPE,
                            &encode_missile_update(&MissileUpdate {
                                id: weapon.id,
                                field_0x10: 0,
                                matrix: missile_update_matrix(weapon.basis, weapon.position),
                                tail_0x60: [0.0; 4],
                                tail_0x70: [0.0; 4],
                                field_0x80: 0,
                            }),
                        ));
                    }
                }
                terminations.push(Self::destroy_frame(weapon.kind, weapon.id));
                continue;
            }
            if now_ms >= weapon.next_update_ms {
                if weapon.kind == WeaponKind::Missile {
                    // The 10 Hz gate caps every step at one interval.
                    let delta_ms =
                        now_ms.saturating_sub(weapon.next_update_ms) + MISSILE_UPDATE_INTERVAL_MS;
                    let step = MISSILE_SPEED_MPS * (delta_ms as f32 / 1_000.0);
                    let heading = weapon.basis[2];
                    for axis in 0..3 {
                        weapon.position[axis] += heading[axis] * step;
                    }
                }
                if weapon.kind == WeaponKind::Mine && accepted.is_none() {
                    // A static mine has nothing new to say without a valid owner pose.
                    weapon.next_update_ms = now_ms
                        .saturating_add(MISSILE_UPDATE_INTERVAL_MS)
                        .min(weapon.expires_ms);
                    survivors.push(weapon);
                    continue;
                }
                let matrix = match weapon.kind {
                    // Verified: the missile update carries the absolute world matrix.
                    // The lead's deeper `FUN_00fe2850` read (2369967..2370003) consumes the full
                    // matrix (basis indices 1/5/9/2/6/10/4/12/8/13/14) as a direct render
                    // transform, so the missile update carries its WORLD matrix - no owner
                    // multiply and no first-vec4 position shortcut.
                    WeaponKind::Missile => missile_update_matrix(weapon.basis, weapon.position),
                    // The mine update carries the inverse-owner local position, so a moving owner
                    // does not move the dropped mine in the world.
                    WeaponKind::Mine => {
                        let observation = accepted.expect("a mine update requires a valid pose");
                        let delta = world_delta(weapon.position, observation.pose.position);
                        // The mine update is owner-local: the inverse-owner position with an
                        // identity basis, so a moving owner never moves the dropped mine.
                        local_offset_matrix(inverse_rotate(&observation.pose, delta))
                    }
                };
                updates.push(match weapon.kind {
                    WeaponKind::Missile => inner_frame(
                        MISSILE_UPDATE_SUBTYPE,
                        &encode_missile_update(&MissileUpdate {
                            id: weapon.id,
                            field_0x10: 0,
                            matrix,
                            // The consumer routes matrix[15] (payload +0x5c) and this block's third
                            // word (payload +0x68); both stay at their reconstructed values.
                            tail_0x60: [0.0; 4],
                            tail_0x70: [0.0; 4],
                            field_0x80: 0,
                        }),
                    ),
                    WeaponKind::Mine => inner_frame(
                        MINE_UPDATE_SUBTYPE,
                        &encode_mine_update(&MineUpdate {
                            id: weapon.id,
                            matrix,
                        }),
                    ),
                });
                if let Some(observation) = accepted {
                    weapon.last_owner_ms = observation.at_ms;
                }
                weapon.next_update_ms = now_ms
                    .saturating_add(MISSILE_UPDATE_INTERVAL_MS)
                    .min(weapon.expires_ms);
            }
            survivors.push(weapon);
        }
        self.weapons = survivors;
        let mut effects_due = Vec::new();
        self.effects.retain(|effect| {
            if now_ms >= effect.expires_ms {
                effects_due.push(inner_frame(
                    EXPLOSION_DESTROY_SUBTYPE,
                    &encode_destroy(&WeaponDestroy { id: effect.id }),
                ));
                false
            } else {
                true
            }
        });
        let mut replies = updates;
        replies.extend(terminations);
        replies.extend(effects_due);
        replies
    }

    /// Race teardown: destroys every live weapon and effect without explosions. The id counter and
    /// tick clock stay, so a resumed same-race save can never reuse an id.
    pub fn clear(&mut self) -> Vec<Vec<u8>> {
        let mut replies = Vec::new();
        for weapon in self.weapons.drain(..) {
            replies.push(Self::destroy_frame(weapon.kind, weapon.id));
        }
        for effect in self.effects.drain(..) {
            replies.push(inner_frame(
                EXPLOSION_DESTROY_SUBTYPE,
                &encode_destroy(&WeaponDestroy { id: effect.id }),
            ));
        }
        replies
    }

    /// New race only: [`WeaponState::clear`] plus the counter/clock reset.
    pub fn begin(&mut self) -> Vec<Vec<u8>> {
        let replies = self.clear();
        self.next_id = 0;
        self.last_tick_ms = 0;
        replies
    }

    /// Structural, ownership and race-window validation for a restored save.
    pub fn validate_for(
        &self,
        owner: u64,
        race_start_ms: u64,
        race_end_ms: u64,
    ) -> Result<(), WeaponError> {
        if self.weapons.len() > MAX_LIVE_WEAPONS {
            return Err(WeaponError::TooManyLive {
                count: self.weapons.len(),
                max: MAX_LIVE_WEAPONS,
            });
        }
        if self.effects.len() > MAX_LIVE_EFFECTS {
            return Err(WeaponError::TooManyLive {
                count: self.effects.len(),
                max: MAX_LIVE_EFFECTS,
            });
        }
        if self.next_id > MAX_TOTAL_ALLOCATIONS {
            return Err(WeaponError::BadCounter {
                next_id: self.next_id,
            });
        }
        let mut ids: Vec<u32> = Vec::new();
        for weapon in &self.weapons {
            Self::check_entry(
                weapon.id,
                weapon.owner,
                weapon.expires_ms,
                owner,
                race_start_ms,
                race_end_ms,
                &mut ids,
            )?;
            // The forged-pose guard: the stored basis must be a real orthonormal, right-handed
            // pose and every float finite - a tampered save cannot park a huge position.
            let pose = WeaponPose {
                basis: weapon.basis,
                position: weapon.position,
            };
            pose.validate()?;
            if weapon.spawned_ms < race_start_ms || weapon.spawned_ms >= weapon.expires_ms {
                return Err(WeaponError::BadTtlSpan {
                    id: weapon.id,
                    spawned_ms: weapon.spawned_ms,
                    expires_ms: weapon.expires_ms,
                });
            }
            let span = weapon.expires_ms - weapon.spawned_ms;
            if span < MIN_WEAPON_TTL_MS || span > weapon.kind.ttl_ms() {
                return Err(WeaponError::BadTtlSpan {
                    id: weapon.id,
                    spawned_ms: weapon.spawned_ms,
                    expires_ms: weapon.expires_ms,
                });
            }
            if weapon.next_update_ms < weapon.spawned_ms
                || weapon.next_update_ms > weapon.expires_ms
            {
                return Err(WeaponError::OutsideWindow {
                    id: weapon.id,
                    expires_ms: weapon.next_update_ms,
                });
            }
            if weapon.last_owner_ms != 0
                && (weapon.last_owner_ms < race_start_ms || weapon.last_owner_ms > race_end_ms)
            {
                return Err(WeaponError::OutsideWindow {
                    id: weapon.id,
                    expires_ms: weapon.last_owner_ms,
                });
            }
        }
        for effect in &self.effects {
            Self::check_entry(
                effect.id,
                effect.owner,
                effect.expires_ms,
                owner,
                race_start_ms,
                race_end_ms,
                &mut ids,
            )?;
            if !effect.position.iter().all(|value| value.is_finite()) {
                return Err(WeaponError::NonFinite {
                    subtype: EXPLOSION_CREATE_SUBTYPE,
                    field: "effect position",
                });
            }
        }
        if let Some(max_id) = ids.iter().copied().max() {
            if self.next_id < max_id {
                return Err(WeaponError::BadCounter {
                    next_id: self.next_id,
                });
            }
        }
        if self.last_tick_ms != 0
            && (self.last_tick_ms < race_start_ms || self.last_tick_ms > race_end_ms)
        {
            return Err(WeaponError::BadClock {
                last_tick_ms: self.last_tick_ms,
            });
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn check_entry(
        id: u32,
        entry_owner: u64,
        expires_ms: u64,
        owner: u64,
        race_start_ms: u64,
        race_end_ms: u64,
        ids: &mut Vec<u32>,
    ) -> Result<(), WeaponError> {
        if id == 0 || id > MAX_TOTAL_ALLOCATIONS {
            return Err(WeaponError::IdOutOfRange { id });
        }
        if ids.contains(&id) {
            return Err(WeaponError::DuplicateId { id });
        }
        ids.push(id);
        if entry_owner != owner {
            return Err(WeaponError::ForeignOwner {
                owner: entry_owner,
                expected: owner,
            });
        }
        if expires_ms <= race_start_ms || expires_ms > race_end_ms {
            return Err(WeaponError::OutsideWindow { id, expires_ms });
        }
        Ok(())
    }
}
