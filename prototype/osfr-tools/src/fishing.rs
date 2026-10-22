//! Free Realms fishing (`BaseFishingPacket`, opcode 138): wire codec and session tracker.
//!
//! Recovered from the decompiled native client (`../lab/decompiled/client/FreeRealms`):
//! `FUN_00b66830`/`FUN_00b69550` move two 16-bit header fields (opcode, sub-opcode);
//! `FUN_008fb810` writes an eight-byte actor handle; `FUN_0091a640`/`FUN_008e2410`
//! move a four-float vector; `FUN_00b6a380|400|460|4d0`, `FUN_00b6b510` are the five
//! client request writers; `FUN_00b66890`, `FUN_00b66990`, `FUN_00b66a00`,
//! `FUN_00b66a80`, `FUN_00b66b80`, `FUN_00b66be0`, `FUN_00b68810`, `FUN_00b69130`
//! are the server-packet readers. The server enum in the preserved OSFR server
//! (`LibSOE/SOE/BaseFishingPackets.cs`, `BasePackets.cs` opcode 138) matches every
//! sub-opcode the client constructors install.
//!
//! Reconstructed, not recovered: phase labels, the ordering rules the tracker
//! enforces, and any timing. This codec keeps candidate server bodies opaque;
//! `fishing_service` writes sub 3/4/11 from recovered widths and order while
//! marking their field assignments and values as reconstructed policy.

// Public API is consumed by the gateway during integration, not by this module.
#![allow(dead_code)]

use std::{error, fmt};

/// SOE application opcode for every `BaseFishingPacket`.
pub const OPCODE: u16 = 138;
/// Local bound for opaque bodies and server packets. It is a decoder-side
/// allocation guard, not a recovered native maximum: no evidence ties the
/// client's trace-buffer size to the size of a fishing packet on the wire.
pub const MAX_BODY: usize = 512;
/// Largest accepted variable-length blob inside a `FishingPacketFishingResult`.
pub const MAX_BLOB: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Truncated,
    WrongOpcode,
    UnknownSub(u16),
    TrailingBytes,
    Oversize,
    OutOfOrder,
    /// Request or result presented by an actor this session is not bound to.
    ForeignActor,
    /// Value the session refuses to act on; the wire codec still carries it.
    InvalidValue,
    /// A service-level session bound is already occupied by other players.
    AtCapacity,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "truncated fishing packet"),
            Self::WrongOpcode => write!(f, "not a fishing packet"),
            Self::UnknownSub(code) => write!(f, "unknown fishing sub-opcode {code}"),
            Self::TrailingBytes => write!(f, "unexpected fishing packet bytes"),
            Self::Oversize => write!(f, "fishing packet body too large"),
            Self::OutOfOrder => write!(f, "fishing request out of order"),
            Self::ForeignActor => write!(f, "fishing packet belongs to another actor"),
            Self::InvalidValue => write!(f, "fishing value is not usable"),
            Self::AtCapacity => write!(f, "fishing session capacity reached"),
        }
    }
}

impl error::Error for Error {}

/// The 16 sub-opcodes both the client classes and the server enum define.
/// Values 9 and 17 are absent from both, so they decode as `UnknownSub`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sub {
    UpdateData,
    RegisterPlayerRequest,
    RegisterPlayerResponse,
    FishInfoUpdate,
    CastAnimRequest,
    CastRequest,
    ReelInRequest,
    SpawnBobber,
    UpdateBobber,
    SpawnSchool,
    DespawnSchool,
    UpdateSchool,
    Result,
    SpecialRequest,
    SpecialResponse,
    SpawnFishRun,
}

impl Sub {
    pub fn from_u16(code: u16) -> Option<Self> {
        Some(match code {
            1 => Self::UpdateData,
            2 => Self::RegisterPlayerRequest,
            3 => Self::RegisterPlayerResponse,
            4 => Self::FishInfoUpdate,
            5 => Self::CastAnimRequest,
            6 => Self::CastRequest,
            7 => Self::ReelInRequest,
            8 => Self::SpawnBobber,
            10 => Self::UpdateBobber,
            11 => Self::SpawnSchool,
            12 => Self::DespawnSchool,
            13 => Self::UpdateSchool,
            14 => Self::Result,
            15 => Self::SpecialRequest,
            16 => Self::SpecialResponse,
            18 => Self::SpawnFishRun,
            _ => return None,
        })
    }

    pub fn as_u16(self) -> u16 {
        match self {
            Self::UpdateData => 1,
            Self::RegisterPlayerRequest => 2,
            Self::RegisterPlayerResponse => 3,
            Self::FishInfoUpdate => 4,
            Self::CastAnimRequest => 5,
            Self::CastRequest => 6,
            Self::ReelInRequest => 7,
            Self::SpawnBobber => 8,
            Self::UpdateBobber => 10,
            Self::SpawnSchool => 11,
            Self::DespawnSchool => 12,
            Self::UpdateSchool => 13,
            Self::Result => 14,
            Self::SpecialRequest => 15,
            Self::SpecialResponse => 16,
            Self::SpawnFishRun => 18,
        }
    }
}

/// Eight-byte actor reference written and read by every fishing packet.
/// The two words are compared as a pair by the native controller; the meaning
/// of the second word is not established, so both stay opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Handle {
    pub id: u32,
    pub seq: u32,
}

impl Handle {
    pub fn encode(self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[..4].copy_from_slice(&self.id.to_le_bytes());
        out[4..].copy_from_slice(&self.seq.to_le_bytes());
        out
    }

    fn read(body: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            id: u32::read(body, 0)?,
            seq: u32::read(body, 4)?,
        })
    }
}

/// Four-float vector as moved by the native four-dword copy helpers.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn encode(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        for (slot, value) in out
            .chunks_exact_mut(4)
            .zip([self.x, self.y, self.z, self.w])
        {
            slot.copy_from_slice(&value.to_le_bytes());
        }
        out
    }

    fn read(body: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            x: f32::read(body, 0)?,
            y: f32::read(body, 4)?,
            z: f32::read(body, 8)?,
            w: f32::read(body, 12)?,
        })
    }
}

trait Read: Sized {
    fn read(body: &[u8], offset: usize) -> Result<Self, Error>;
}

impl Read for u32 {
    fn read(body: &[u8], offset: usize) -> Result<Self, Error> {
        let raw = body.get(offset..offset + 4).ok_or(Error::Truncated)?;
        Ok(Self::from_le_bytes(raw.try_into().unwrap()))
    }
}

impl Read for f32 {
    fn read(body: &[u8], offset: usize) -> Result<Self, Error> {
        Ok(f32::from_bits(u32::read(body, offset)?))
    }
}

/// The five client-to-server fishing requests with recovered field order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientRequest {
    /// Sub 2: native `FUN_00b6a400` writes handle then one byte.
    RegisterPlayer { actor: Handle, flag: u8 },
    /// Sub 5: `FUN_00b6a380` writes handle, one float, then a vector.
    CastAnim {
        actor: Handle,
        distance: f32,
        position: Vec4,
    },
    /// Sub 6: `FUN_00b6a460` writes handle, a vector, then one byte.
    Cast {
        actor: Handle,
        target: Vec4,
        flag: u8,
    },
    /// Sub 7: `FUN_00b6a4d0` writes handle then one byte.
    ReelIn { actor: Handle, flag: u8 },
    /// Sub 15: `FUN_00b6b510` writes only the handle.
    Special { actor: Handle },
}

impl ClientRequest {
    pub fn sub(&self) -> Sub {
        match self {
            Self::RegisterPlayer { .. } => Sub::RegisterPlayerRequest,
            Self::CastAnim { .. } => Sub::CastAnimRequest,
            Self::Cast { .. } => Sub::CastRequest,
            Self::ReelIn { .. } => Sub::ReelInRequest,
            Self::Special { .. } => Sub::SpecialRequest,
        }
    }

    pub fn actor(&self) -> Handle {
        match self {
            Self::RegisterPlayer { actor, .. }
            | Self::CastAnim { actor, .. }
            | Self::Cast { actor, .. }
            | Self::ReelIn { actor, .. }
            | Self::Special { actor } => *actor,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = frame(self.sub(), &[]);
        match *self {
            Self::RegisterPlayer { actor, flag } => {
                out.extend(actor.encode());
                out.push(flag);
            }
            Self::CastAnim {
                actor,
                distance,
                position,
            } => {
                out.extend(actor.encode());
                out.extend(distance.to_le_bytes());
                out.extend(position.encode());
            }
            Self::Cast {
                actor,
                target,
                flag,
            } => {
                out.extend(actor.encode());
                out.extend(target.encode());
                out.push(flag);
            }
            Self::ReelIn { actor, flag } => {
                out.extend(actor.encode());
                out.push(flag);
            }
            Self::Special { actor } => out.extend(actor.encode()),
        }
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let (sub, body) = unframe(bytes)?;
        let actor = Handle::read(body)?;
        match sub {
            Sub::RegisterPlayerRequest => {
                exact(body, 9)?;
                Ok(Self::RegisterPlayer {
                    actor,
                    flag: body[8],
                })
            }
            Sub::CastAnimRequest => {
                exact(body, 28)?;
                Ok(Self::CastAnim {
                    actor,
                    distance: f32::read(body, 8)?,
                    position: Vec4::read(&body[12..])?,
                })
            }
            Sub::CastRequest => {
                exact(body, 25)?;
                Ok(Self::Cast {
                    actor,
                    target: Vec4::read(&body[8..])?,
                    flag: body[24],
                })
            }
            Sub::ReelInRequest => {
                exact(body, 9)?;
                Ok(Self::ReelIn {
                    actor,
                    flag: body[8],
                })
            }
            Sub::SpecialRequest => {
                exact(body, 8)?;
                Ok(Self::Special { actor })
            }
            other => Err(Error::UnknownSub(other.as_u16())),
        }
    }
}

/// Fixed-size bodies must be exactly consumed; variable ones only bounded.
fn exact(body: &[u8], len: usize) -> Result<(), Error> {
    match body.len().cmp(&len) {
        std::cmp::Ordering::Less => Err(Error::Truncated),
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Greater => Err(Error::TrailingBytes),
    }
}

fn frame(sub: Sub, body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend(OPCODE.to_le_bytes());
    out.extend(sub.as_u16().to_le_bytes());
    out.extend(body);
    out
}

fn unframe(bytes: &[u8]) -> Result<(Sub, &[u8]), Error> {
    let code = u16::from_le_bytes(bytes.get(..2).ok_or(Error::Truncated)?.try_into().unwrap());
    if code != OPCODE {
        return Err(Error::WrongOpcode);
    }
    let sub = u16::from_le_bytes(bytes.get(2..4).ok_or(Error::Truncated)?.try_into().unwrap());
    let sub = Sub::from_u16(sub).ok_or(Error::UnknownSub(sub))?;
    Ok((sub, bytes.get(4..).unwrap()))
}

/// Server-to-client `FishingPacketFishingResult` (sub 14) body.
///
/// Field order and types come from the native reader `FUN_00b66be0`; the roles
/// of the individual values are not established, so they keep their native
/// offsets as names. The three count-prefixed byte runs (blob_2c, blob_58,
/// blob_64) are the runs the reader skips through `FUN_00894b10`; their element
/// structure was not recovered, so they stay raw bytes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResultBody {
    pub actor: Handle,
    pub at_18: u32,
    pub at_1c: bool,
    pub at_20: u32,
    pub at_24: u32,
    pub blob_2c: Vec<u8>,
    pub at_38: f32,
    pub at_3c: u32,
    pub at_40: f32,
    pub at_44: u32,
    pub at_48: u32,
    pub at_4c: u32,
    pub at_50: u32,
    pub blob_58: Vec<u8>,
    pub blob_64: Vec<u8>,
    pub at_74: u32,
    pub at_7c: f32,
    pub at_80: bool,
    pub at_78: u32,
}

impl ResultBody {
    /// The native reader marks a NaN float as a decode error; the same rule
    /// rejects a result the client would have discarded.
    pub fn valid(&self) -> bool {
        [self.at_38, self.at_40, self.at_7c]
            .iter()
            .all(|v| v.is_finite())
    }

    fn decode(body: &[u8]) -> Result<Self, Error> {
        let mut out = Self {
            actor: Handle::read(body)?,
            ..Self::default()
        };
        out.at_18 = u32::read(body, 8)?;
        out.at_1c = *body.get(12).ok_or(Error::Truncated)? != 0;
        out.at_20 = u32::read(body, 13)?;
        out.at_24 = u32::read(body, 17)?;
        let (blob, next) = read_blob(body, 21)?;
        out.blob_2c = blob;
        out.at_38 = f32::read(body, next)?;
        out.at_3c = u32::read(body, next + 4)?;
        out.at_40 = f32::read(body, next + 8)?;
        out.at_44 = u32::read(body, next + 12)?;
        out.at_48 = u32::read(body, next + 16)?;
        out.at_4c = u32::read(body, next + 20)?;
        out.at_50 = u32::read(body, next + 24)?;
        let (blob, next) = read_blob(body, next + 28)?;
        out.blob_58 = blob;
        let (blob, next) = read_blob(body, next)?;
        out.blob_64 = blob;
        out.at_74 = u32::read(body, next)?;
        out.at_7c = f32::read(body, next + 4)?;
        out.at_80 = *body.get(next + 8).ok_or(Error::Truncated)? != 0;
        out.at_78 = u32::read(body, next + 9)?;
        if next + 13 != body.len() {
            return Err(Error::TrailingBytes);
        }
        Ok(out)
    }
}

fn read_blob(body: &[u8], offset: usize) -> Result<(Vec<u8>, usize), Error> {
    let len = u32::read(body, offset)? as usize;
    if len > MAX_BLOB || body.len() < offset + 4 + len {
        return Err(if len > MAX_BLOB {
            Error::Oversize
        } else {
            Error::Truncated
        });
    }
    Ok((
        body[offset + 4..offset + 4 + len].to_vec(),
        offset + 4 + len,
    ))
}

fn put_blob(out: &mut Vec<u8>, value: &[u8]) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value);
}

/// Server-to-client `SpawnFishRun` (sub 18). Case 18 (`0x00b6d3ab`) has no handle and acts on
/// the local player's fishing entry, so it goes to the caster only. It parses through
/// `0x00b6aed0` (called at `0x00b6d3cb`), which refuses a body the reader `0x00b6a2b0` does not
/// consume exactly; that reader takes the fields below in order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FishRun {
    /// +0x0c.
    pub flag: bool,
    /// +0x10: the lure drawn at the hook when the head record's `on_hook` is 0 (`loc_B6D9F9`);
    /// looked up unchecked at `0x00b6da36`, so it must be a real model.
    pub model: i32,
    /// +0x14: texture alias string.
    pub texture: Vec<u8>,
    /// +0x24: `0x00b694e0` reads an i32 count, then `0x00b67550` per fish.
    pub fish: Vec<RunFish>,
}

/// One underwater fish (`0x00b67550`). Its model is looked up unchecked (`0x00b6d4a7`); only
/// the head record may be `on_hook`, a later one with the byte set is skipped (`0x00b6d4c4`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunFish {
    /// +0x00: the key sub 10 names.
    pub id: u32,
    /// +0x04.
    pub model: i32,
    /// +0x08 and +0x18: two strings, tint then texture alias (names R).
    pub tint: Vec<u8>,
    pub texture: Vec<u8>,
    /// +0x28: size grade 1..=4.
    pub size_grade: u32,
    /// +0x2c: not read by case 18.
    pub at_2c: u32,
    /// +0x30: this record is the thing on the hook (name R). Case 18 places it at the bobber
    /// (`0x00b6d4fe`; others scatter at `0x00b6d5d4`), ties the line to its `hinge_2` bone and
    /// makes no lure (`0x00b6d81f`..`0x00b6d8e7`). `0x00cd3a70` copies it to the fish's +0x10 and,
    /// when set, starts the fish in state 6 with +0xdb = 1. State 4 then sends 138/7 without its
    /// +0xd9 (hooked) gate (`0x00b6cbd5`), and a 138/10 escape shows 'bbe_fishing_item_lost'
    /// instead of 'bbe_fishing_fish_escaped' (`0x00b6df48`).
    pub on_hook: bool,
    /// +0x34..=+0x58, copied onto the fish at `0x00b6d749`.
    pub motion: [f32; 10],
    /// +0x5c, read by `0x008d53b0`.
    pub motion_tail: f32,
}

pub fn encode_spawn_fish_run(run: &FishRun) -> Vec<u8> {
    let mut out = frame(Sub::SpawnFishRun, &[]);
    out.push(u8::from(run.flag));
    out.extend(run.model.to_le_bytes());
    put_blob(&mut out, &run.texture);
    out.extend((run.fish.len() as u32).to_le_bytes());
    for fish in &run.fish {
        out.extend(fish.id.to_le_bytes());
        out.extend(fish.model.to_le_bytes());
        put_blob(&mut out, &fish.tint);
        put_blob(&mut out, &fish.texture);
        out.extend(fish.size_grade.to_le_bytes());
        out.extend(fish.at_2c.to_le_bytes());
        out.push(u8::from(fish.on_hook));
        for value in fish.motion.iter().chain([&fish.motion_tail]) {
            out.extend(value.to_le_bytes());
        }
    }
    out
}

impl FishRun {
    fn decode(body: &[u8]) -> Result<Self, Error> {
        let flag = *body.first().ok_or(Error::Truncated)? != 0;
        let model = u32::read(body, 1)? as i32;
        let (texture, at) = read_blob(body, 5)?;
        // `0x00b694e0` loops `j < count` on an i32, so a negative count reads no records.
        let count = u32::read(body, at)? as i32;
        let mut at = at + 4;
        let mut fish = Vec::new();
        for _ in 0..count.max(0) {
            let (tint, next) = read_blob(body, at + 8)?;
            let (texture, next) = read_blob(body, next)?;
            let mut motion = [0.0; 10];
            for (index, value) in motion.iter_mut().enumerate() {
                *value = f32::read(body, next + 9 + 4 * index)?;
            }
            fish.push(RunFish {
                id: u32::read(body, at)?,
                model: u32::read(body, at + 4)? as i32,
                tint,
                texture,
                size_grade: u32::read(body, next)?,
                at_2c: u32::read(body, next + 4)?,
                on_hook: *body.get(next + 8).ok_or(Error::Truncated)? != 0,
                motion,
                motion_tail: f32::read(body, next + 49)?,
            });
            at = next + 53;
        }
        exact(body, at)?;
        Ok(Self {
            flag,
            model,
            texture,
            fish,
        })
    }
}

/// Server-to-client packets whose native reader was located, plus an opaque
/// fallback for the sub-opcodes that were not recovered.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerPacket {
    /// Sub 1 (`FUN_00b66890`): handle then a vector.
    UpdateData {
        actor: Handle,
        position: Vec4,
    },
    /// Sub 5 (`FUN_00b668f0`): the cast-animation request echoed to proxies.
    CastAnimEcho {
        actor: Handle,
        distance: f32,
        position: Vec4,
    },
    /// Sub 7 (`FUN_00b66990`): the reel-in request echoed to proxies.
    ReelInEcho {
        actor: Handle,
        flag: bool,
    },
    /// Sub 8 (`FUN_00b66a00`): handle, one u32, then two vectors.
    SpawnBobber {
        actor: Handle,
        code: u32,
        position: Vec4,
        velocity: Vec4,
    },
    /// Sub 10 (`FUN_00b66a80`): handle, one u32, then two flags.
    UpdateBobber {
        actor: Handle,
        code: u32,
        a: bool,
        b: bool,
    },
    /// Sub 12 (`FUN_00b68810`): the school id.
    DespawnSchool {
        school: u32,
    },
    /// Sub 13 (`FUN_00b66b80`): the school id and its vector.
    UpdateSchool {
        school: u32,
        position: Vec4,
    },
    /// Sub 14 (`FUN_00b66be0`).
    Result {
        body: ResultBody,
    },
    /// Sub 18 (`0x00b6a2b0`).
    SpawnFishRun {
        run: FishRun,
    },
    Opaque {
        sub: Sub,
        body: Vec<u8>,
    },
}

impl ServerPacket {
    pub fn decode_known(sub: Sub, body: &[u8]) -> Result<Self, Error> {
        if body.len() > MAX_BODY {
            return Err(Error::Oversize);
        }
        Ok(match sub {
            Sub::UpdateData => {
                exact(body, 24)?;
                Self::UpdateData {
                    actor: Handle::read(body)?,
                    position: Vec4::read(&body[8..])?,
                }
            }
            Sub::CastAnimRequest => {
                exact(body, 28)?;
                Self::CastAnimEcho {
                    actor: Handle::read(body)?,
                    distance: f32::read(body, 8)?,
                    position: Vec4::read(&body[12..])?,
                }
            }
            Sub::ReelInRequest => {
                exact(body, 9)?;
                Self::ReelInEcho {
                    actor: Handle::read(body)?,
                    flag: body[8] != 0,
                }
            }
            Sub::SpawnBobber => {
                exact(body, 44)?;
                Self::SpawnBobber {
                    actor: Handle::read(body)?,
                    code: u32::read(body, 8)?,
                    position: Vec4::read(&body[12..])?,
                    velocity: Vec4::read(&body[28..])?,
                }
            }
            Sub::UpdateBobber => {
                exact(body, 14)?;
                Self::UpdateBobber {
                    actor: Handle::read(body)?,
                    code: u32::read(body, 8)?,
                    a: body[12] != 0,
                    b: body[13] != 0,
                }
            }
            Sub::DespawnSchool => {
                exact(body, 4)?;
                Self::DespawnSchool {
                    school: u32::read(body, 0)?,
                }
            }
            Sub::UpdateSchool => {
                exact(body, 20)?;
                Self::UpdateSchool {
                    school: u32::read(body, 0)?,
                    position: Vec4::read(&body[4..])?,
                }
            }
            Sub::Result => Self::Result {
                body: ResultBody::decode(body)?,
            },
            Sub::SpawnFishRun => Self::SpawnFishRun {
                run: FishRun::decode(body)?,
            },
            _ => Self::Opaque {
                sub,
                body: body.to_vec(),
            },
        })
    }

    pub fn sub(&self) -> Sub {
        match self {
            Self::UpdateData { .. } => Sub::UpdateData,
            Self::CastAnimEcho { .. } => Sub::CastAnimRequest,
            Self::ReelInEcho { .. } => Sub::ReelInRequest,
            Self::SpawnBobber { .. } => Sub::SpawnBobber,
            Self::UpdateBobber { .. } => Sub::UpdateBobber,
            Self::DespawnSchool { .. } => Sub::DespawnSchool,
            Self::UpdateSchool { .. } => Sub::UpdateSchool,
            Self::Result { .. } => Sub::Result,
            Self::SpawnFishRun { .. } => Sub::SpawnFishRun,
            Self::Opaque { sub, .. } => *sub,
        }
    }
}

/// What applying a request did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Applied,
    /// Repeated request the native controller ignores in the same state.
    Duplicate,
}

/// Phase of one player's fishing session.
///
/// Labels are reconstructed; `native_state` exposes the controller values the
/// native code stores at `ControllerFishing + 0x1c` where they were observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    /// Native state 4: the bite/hook window.
    Hook,
    /// Native state 5: fish hooked, reel-in accepted.
    Reeling,
    /// Native state 6: cast sent, awaiting the hook.
    Casting,
    /// A `FishingPacketFishingResult` arrived.
    Landing,
}

impl Phase {
    pub const fn native_state(self) -> Option<u8> {
        match self {
            Self::Idle => None,
            Self::Hook => Some(4),
            Self::Reeling => Some(5),
            Self::Casting => Some(6),
            Self::Landing => None,
        }
    }
}

/// Request ordering tracker bound to one client actor.
///
/// Reconstructed policy, not recovered history: ordering rules, the
/// first-registration-binds rule, same-actor re-registration as a no-op,
/// refusal of foreign actors, value validation and the unbinding performed by
/// `reset` are local decisions derived from the controller state machine and
/// the client's own duplicate guard. The client's duplicate guard for the cast
/// state is the only ordering behavior directly observed. No timing is
/// invented, so callers drive phase changes through `hooked` and decoded
/// server packets.
///
/// Only `FishingPacketFishingResult` is actor-scoped: the other server packets
/// include proxy echoes of other players, so they never move this session's
/// phase. A result is accepted only from `Phase::Reeling`, the phase an owned
/// reel-in produces, so a result that was never requested, or a replayed one,
/// is refused instead of re-entering `Phase::Landing`. That one-shot rule is a
/// reconstructed server-authority policy: the wire result carries no nonce,
/// and the closing review findings R1/R2 require the gate before any reward
/// can exist. `registered` survives `Landing`; `cancel` re-arms the session
/// for the next cast and `reset` unbinds the actor.
#[derive(Debug, Clone)]
pub struct Session {
    actor: Option<Handle>,
    registered: bool,
    phase: Phase,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            actor: None,
            registered: false,
            phase: Phase::Idle,
        }
    }

    pub fn actor(&self) -> Option<Handle> {
        self.actor
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn registered(&self) -> bool {
        self.registered
    }

    /// Reconstructed server event: the nibble has become a hooked fish.
    pub fn hooked(&mut self) {
        self.phase = Phase::Reeling;
    }

    /// Reconstructed server event: a fish is nibbling before the hook window.
    pub fn nibbling(&mut self) {
        self.phase = Phase::Hook;
    }

    /// Reconstructed policy: clears the phase, the registration flag and the
    /// actor binding, so a different player may register next.
    pub fn reset(&mut self) {
        self.actor = None;
        self.registered = false;
        self.phase = Phase::Idle;
    }

    /// Reconstructed policy: abort the current attempt (cast timeout, lost
    /// reel window or a consumed landing) while keeping the actor registered,
    /// so the next cast can start without a fresh registration.
    pub fn cancel(&mut self) {
        self.phase = Phase::Idle;
    }

    pub fn request(&mut self, request: &ClientRequest) -> Result<Outcome, Error> {
        validate(request)?;
        match request {
            ClientRequest::RegisterPlayer { actor, .. } => match self.actor {
                None => {
                    self.actor = Some(*actor);
                    self.registered = true;
                    Ok(Outcome::Applied)
                }
                Some(bound) if bound == *actor => Ok(Outcome::Duplicate),
                Some(_) => Err(Error::ForeignActor),
            },
            other => {
                let bound = self.actor.ok_or(Error::OutOfOrder)?;
                if bound != other.actor() {
                    return Err(Error::ForeignActor);
                }
                self.apply_ordered(other)
            }
        }
    }

    fn apply_ordered(&mut self, request: &ClientRequest) -> Result<Outcome, Error> {
        match request {
            ClientRequest::Cast { .. } => {
                if !self.registered {
                    return Err(Error::OutOfOrder);
                }
                if self.phase == Phase::Casting {
                    return Ok(Outcome::Duplicate);
                }
                self.phase = Phase::Casting;
                Ok(Outcome::Applied)
            }
            ClientRequest::CastAnim { .. } => match self.phase {
                Phase::Casting => Ok(Outcome::Applied),
                _ => Err(Error::OutOfOrder),
            },
            ClientRequest::ReelIn { .. } => match self.phase {
                Phase::Reeling => Ok(Outcome::Applied),
                _ => Err(Error::OutOfOrder),
            },
            ClientRequest::Special { .. } => Ok(Outcome::Applied),
            ClientRequest::RegisterPlayer { .. } => unreachable!("handled by the caller"),
        }
    }

    pub fn server(&mut self, packet: &ServerPacket) -> Result<Option<Phase>, Error> {
        match packet {
            ServerPacket::Result { body } if !body.valid() => Err(Error::InvalidValue),
            ServerPacket::Result { body } => {
                if self.actor != Some(body.actor) {
                    return Err(Error::ForeignActor);
                }
                // One-shot result gate (closes review findings R1/R2): the
                // only phase that can own a catch is the reel window an owned
                // reel-in produced. `Idle` after registration, `Casting`, and
                // a repeat after `Landing` are all refused without mutation.
                if self.phase != Phase::Reeling {
                    return Err(Error::OutOfOrder);
                }
                self.phase = Phase::Landing;
                Ok(Some(self.phase))
            }
            ServerPacket::Opaque { body, .. } if body.len() > MAX_BODY => Err(Error::Oversize),
            _ => Ok(None),
        }
    }
}

/// Behavioral validation in front of any state mutation.
///
/// Reconstructed guard, stricter than the evidence: the only recovered value
/// rejection is the NaN check inside the native result reader (`FUN_00b66be0`
/// sets its decode-error flag on a NaN float). Refusing infinities, negative
/// cast distances and non-finite vectors is a local policy chosen here, and the
/// wire codec stays lossless so the original bytes survive a decode/encode
/// round trip.
fn validate(request: &ClientRequest) -> Result<(), Error> {
    let finite =
        |v: &Vec4| v.x.is_finite() && v.y.is_finite() && v.z.is_finite() && v.w.is_finite();
    match request {
        ClientRequest::CastAnim {
            distance, position, ..
        } => {
            if !distance.is_finite() || *distance < 0.0 || !finite(position) {
                return Err(Error::InvalidValue);
            }
        }
        ClientRequest::Cast { target, .. } => {
            if !finite(target) {
                return Err(Error::InvalidValue);
            }
        }
        ClientRequest::RegisterPlayer { .. }
        | ClientRequest::ReelIn { .. }
        | ClientRequest::Special { .. } => {}
    }
    Ok(())
}
