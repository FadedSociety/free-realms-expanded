//! Soccer packet layouts recovered from the native client readers and writers.
//! This codec does not use OSFR-held packet captures or server JSON as facts.
//! Only sub-13, sub-14 and sub-16 layouts and the cited fields in those layouts
//! are treated as native-map client facts. Other packet variants remain
//! legacy codec candidates and do not support build-guide facts. Match
//! authority, physics, roster sizing, clock duration, output values, result
//! routing and rewards are reconstructed server policy (R).

use std::collections::BTreeSet;
use std::fmt;
use std::io;

pub const TUNNEL_OPCODE: u16 = 76;
/// Native `MiniGameStatusHandler` type id for Soccer.
pub const SOCCER_TYPE_ID: i32 = 12;
pub const ZONE: &str = "sg_soccer";
/// R: fallback half length until a client-authored duration or native match
/// run establishes the historical value. The client consumes server time.
pub const HALF_SECONDS: f32 = 180.0;
const FLAG: bool = true;

/// Where a fact comes from. Reconstructed values are never presented as original.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    Original,
    Reconstructed,
    Missing,
}

/// Which way a gateway message is labelled by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Server -> client. The only direction allowed to change match state.
    ServerToClient,
    /// Client -> server. Requests are decoded and returned to the caller, but
    /// they never change authoritative state.
    ClientToServer,
}

/// Client input classes that the Rust authority decodes. The packet reader
/// establishes their layouts; sender identity and direction remain gateway
/// policy, not payload facts.
pub const CLIENT_PACKET_IDS: [u16; 4] = [1, 3, 14, 16];

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn take<'a>(payload: &mut &'a [u8], count: usize) -> io::Result<&'a [u8]> {
    if payload.len() < count {
        return Err(invalid("truncated soccer payload"));
    }
    let (head, rest) = payload.split_at(count);
    *payload = rest;
    Ok(head)
}

fn read_u8(payload: &mut &[u8]) -> io::Result<u8> {
    Ok(take(payload, 1)?[0])
}

fn read_u16(payload: &mut &[u8]) -> io::Result<u16> {
    Ok(u16::from_le_bytes(take(payload, 2)?.try_into().unwrap()))
}

fn read_u32(payload: &mut &[u8]) -> io::Result<u32> {
    Ok(u32::from_le_bytes(take(payload, 4)?.try_into().unwrap()))
}

fn read_u64(payload: &mut &[u8]) -> io::Result<u64> {
    Ok(u64::from_le_bytes(take(payload, 8)?.try_into().unwrap()))
}

fn read_f32(payload: &mut &[u8]) -> io::Result<f32> {
    Ok(f32::from_le_bytes(take(payload, 4)?.try_into().unwrap()))
}

fn read_f32s<const N: usize>(payload: &mut &[u8]) -> io::Result<[f32; N]> {
    let mut values = [0.0f32; N];
    for value in values.iter_mut() {
        *value = read_f32(payload)?;
    }
    Ok(values)
}

fn read_u32s<const N: usize>(payload: &mut &[u8]) -> io::Result<[u32; N]> {
    let mut values = [0u32; N];
    for value in values.iter_mut() {
        *value = read_u32(payload)?;
    }
    Ok(values)
}

fn read_bytes<const N: usize>(payload: &mut &[u8]) -> io::Result<[u8; N]> {
    Ok(take(payload, N)?.try_into().unwrap())
}

fn end(payload: &[u8]) -> io::Result<()> {
    if payload.is_empty() {
        Ok(())
    } else {
        Err(invalid("trailing soccer bytes"))
    }
}

fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        out.extend(value.to_le_bytes());
    }
}

/// Soccer codec model. Only the cited sub-13/14/16 layouts are native-map
/// facts; other variants are legacy candidates pending client-reader checks.
#[derive(Clone, Debug, PartialEq)]
pub enum SoccerPacket {
    /// id 1, 65 bytes: u64 player, position, 45-byte orientation/velocity block.
    UpdatePlayerPos {
        player: u64,
        position: [f32; 3],
        tail: [u8; 45],
    },
    /// R candidate goal relay: three u32 fields. The third field's sum meaning
    /// and acceptance rule are not established by the reader evidence used here.
    GoalMade { fields: [u32; 3] },
    /// Candidate registration relay layout: u64 player then two marker bytes.
    RegisterPlayer { player: u64, marker: [u8; 2] },
    /// id 4, 91 bytes: row-major 4x4 transform (translation in row 3), velocity,
    /// then 15 opaque bytes (observed constant tail).
    UpdateSoccerBall {
        transform: [f32; 16],
        velocity: [f32; 3],
        tail: [u8; 15],
    },
    /// Candidate game-state relay: 18 u32 words; values remain opaque.
    UpdateGameState { words: [u32; 18] },
    /// id 6, 16 bytes: u64 player, u32 team, u32 slot.
    SetPlayerTeam { player: u64, team: u32, slot: u32 },
    /// Candidate config relay. Its detailed offsets are not used as client facts.
    SetClientConfig {
        raw: Vec<u8>,
        ball_asset: String,
        ball_spawn: [f32; 4],
    },
    /// id 8, 38 bytes: (u32 length, name) x2, RGBA x2, definition id x2, 1 byte.
    TeamColours {
        team_a_name: String,
        team_b_name: String,
        team_a_rgba: [u8; 4],
        team_b_rgba: [u8; 4],
        team_a_id: u32,
        team_b_id: u32,
        tail: [u8; 1],
    },
    /// id 9, 26 bytes: u8 state, u32 definition, u8 marker, position, unit scale, u32 instance.
    SpawnPickUp {
        state: u8,
        definition: u32,
        marker: u8,
        position: [f32; 3],
        scale: f32,
        instance: u32,
    },
    /// id 10, 13 bytes: u8 flag, f32 value, u64 player.
    AcquirePickUp { flag: u8, value: f32, player: u64 },
    /// id 11, 17 bytes: u8 flag, u64 new holder, u64 previous holder.
    AcquireBall {
        flag: u8,
        holder: u64,
        previous: u64,
    },
    /// id 12, 25 bytes: u8 flag, ball position, unit scale, u64 previous holder.
    FreeBall {
        flag: u8,
        position: [f32; 3],
        scale: f32,
        previous: u64,
    },
    /// id 13, 36 bytes (`00b79580` client reader): `u32`, `f32[4]`, `f32`,
    /// `f32`, `u32`, `f32`. Field semantics and server emission are R/open.
    ImpulseBall {
        word: u32,
        vector: [f32; 4],
        value_a: f32,
        value_b: f32,
        action: u32,
        value_c: f32,
    },
    /// id 14, 62 bytes (`00b794b0` client reader): `u64` player guid, `u8` soccer state,
    /// `u8` quantized charge, `u32` state-change counter, then action direction,
    /// steering vector and position vector as `f32[4]` each. The state counter
    /// is not the transport sequence. Only the first vector authorizes a
    /// bounded kick direction; the server still owns ball motion.
    UpdatePlayerState {
        player: u64,
        state: u8,
        aux: u8,
        state_counter: u32,
        vectors: [[f32; 4]; 3],
    },
    /// id 15, 28 bytes: u64 player, five u32 fields.
    UpdatePlayerStats { player: u64, words: [u32; 5] },
    /// id 16, 20 bytes (`00b793c0` client reader): `u64` player guid,
    /// two `f32` direction components, then `u32` control bits. The bits are
    /// integer input state; no bit is named kick/pass by the client.
    UpdatePlayerControls {
        player: u64,
        direction: [f32; 2],
        controls: u32,
    },
    /// id 18, 21 bytes: four floats then five opaque bytes.
    HighArcBall { head: [f32; 4], tail: [u8; 5] },
    /// id 19, 44 bytes: u64 player, nine u32 fields.
    PlayerVariableStats { player: u64, words: [u32; 9] },
    /// id 21, 176 bytes: u64 player, 42 per-player tuning floats
    /// (Soccer Star Kick Power/Tackle Power/Toughness family; field names not proven).
    SetSoccerPlayerOneTimeStats { player: u64, stats: [f32; 42] },
    /// id 23, 8 bytes: countdown remaining (0.0 before kickoff) and elapsed seconds.
    UpdateGameTime { remaining: f32, elapsed: f32 },
    /// Candidate bounce relay: two u32 fields.
    BallBounce { words: [u32; 2] },
    /// Any soccer sub-opcode without a checked layout or any future id.
    Unknown { sub: u16, payload: Vec<u8> },
}

/// One decoded gateway message: outer envelope plus the soccer packet.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub outer: u16,
    pub flag: bool,
    pub packet: SoccerPacket,
}

fn soccer_sub(packet: &SoccerPacket) -> u16 {
    match packet {
        SoccerPacket::UpdatePlayerPos { .. } => 1,
        SoccerPacket::GoalMade { .. } => 2,
        SoccerPacket::RegisterPlayer { .. } => 3,
        SoccerPacket::UpdateSoccerBall { .. } => 4,
        SoccerPacket::UpdateGameState { .. } => 5,
        SoccerPacket::SetPlayerTeam { .. } => 6,
        SoccerPacket::SetClientConfig { .. } => 7,
        SoccerPacket::TeamColours { .. } => 8,
        SoccerPacket::SpawnPickUp { .. } => 9,
        SoccerPacket::AcquirePickUp { .. } => 10,
        SoccerPacket::AcquireBall { .. } => 11,
        SoccerPacket::FreeBall { .. } => 12,
        SoccerPacket::ImpulseBall { .. } => 13,
        SoccerPacket::UpdatePlayerState { .. } => 14,
        SoccerPacket::UpdatePlayerStats { .. } => 15,
        SoccerPacket::UpdatePlayerControls { .. } => 16,
        SoccerPacket::HighArcBall { .. } => 18,
        SoccerPacket::PlayerVariableStats { .. } => 19,
        SoccerPacket::SetSoccerPlayerOneTimeStats { .. } => 21,
        SoccerPacket::UpdateGameTime { .. } => 23,
        SoccerPacket::BallBounce { .. } => 25,
        SoccerPacket::Unknown { sub, .. } => *sub,
    }
}

fn payload_bytes(packet: &SoccerPacket) -> Vec<u8> {
    let mut out = Vec::new();
    match packet {
        SoccerPacket::UpdatePlayerPos {
            player,
            position,
            tail,
        } => {
            out.extend(player.to_le_bytes());
            put_f32s(&mut out, position);
            out.extend(tail);
        }
        SoccerPacket::GoalMade { fields } => {
            for field in fields {
                out.extend(field.to_le_bytes());
            }
        }
        SoccerPacket::RegisterPlayer { player, marker } => {
            out.extend(player.to_le_bytes());
            out.extend(marker);
        }
        SoccerPacket::UpdateSoccerBall {
            transform,
            velocity,
            tail,
        } => {
            put_f32s(&mut out, transform);
            put_f32s(&mut out, velocity);
            out.extend(tail);
        }
        SoccerPacket::UpdateGameState { words } => {
            for word in words {
                out.extend(word.to_le_bytes());
            }
        }
        SoccerPacket::SetPlayerTeam { player, team, slot } => {
            out.extend(player.to_le_bytes());
            out.extend(team.to_le_bytes());
            out.extend(slot.to_le_bytes());
        }
        SoccerPacket::SetClientConfig { raw, .. } => out.extend(raw),
        SoccerPacket::TeamColours {
            team_a_name,
            team_b_name,
            team_a_rgba,
            team_b_rgba,
            team_a_id,
            team_b_id,
            tail,
        } => {
            out.extend((team_a_name.len() as u32).to_le_bytes());
            out.extend(team_a_name.as_bytes());
            out.extend((team_b_name.len() as u32).to_le_bytes());
            out.extend(team_b_name.as_bytes());
            out.extend(team_a_rgba);
            out.extend(team_b_rgba);
            out.extend(team_a_id.to_le_bytes());
            out.extend(team_b_id.to_le_bytes());
            out.extend(tail);
        }
        SoccerPacket::SpawnPickUp {
            state,
            definition,
            marker,
            position,
            scale,
            instance,
        } => {
            out.push(*state);
            out.extend(definition.to_le_bytes());
            out.push(*marker);
            put_f32s(&mut out, position);
            out.extend(scale.to_le_bytes());
            out.extend(instance.to_le_bytes());
        }
        SoccerPacket::AcquirePickUp {
            flag,
            value,
            player,
        } => {
            out.push(*flag);
            out.extend(value.to_le_bytes());
            out.extend(player.to_le_bytes());
        }
        SoccerPacket::AcquireBall {
            flag,
            holder,
            previous,
        } => {
            out.push(*flag);
            out.extend(holder.to_le_bytes());
            out.extend(previous.to_le_bytes());
        }
        SoccerPacket::FreeBall {
            flag,
            position,
            scale,
            previous,
        } => {
            out.push(*flag);
            put_f32s(&mut out, position);
            out.extend(scale.to_le_bytes());
            out.extend(previous.to_le_bytes());
        }
        SoccerPacket::ImpulseBall {
            word,
            vector,
            value_a,
            value_b,
            action,
            value_c,
        } => {
            out.extend(word.to_le_bytes());
            put_f32s(&mut out, vector);
            out.extend(value_a.to_le_bytes());
            out.extend(value_b.to_le_bytes());
            out.extend(action.to_le_bytes());
            out.extend(value_c.to_le_bytes());
        }
        SoccerPacket::UpdatePlayerState {
            player,
            state,
            aux,
            state_counter,
            vectors,
        } => {
            out.extend(player.to_le_bytes());
            out.push(*state);
            out.push(*aux);
            out.extend(state_counter.to_le_bytes());
            for vector in vectors {
                put_f32s(&mut out, vector);
            }
        }
        SoccerPacket::UpdatePlayerStats { player, words } => {
            out.extend(player.to_le_bytes());
            for word in words {
                out.extend(word.to_le_bytes());
            }
        }
        SoccerPacket::UpdatePlayerControls {
            player,
            direction,
            controls,
        } => {
            out.extend(player.to_le_bytes());
            put_f32s(&mut out, direction);
            out.extend(controls.to_le_bytes());
        }
        SoccerPacket::HighArcBall { head, tail } => {
            put_f32s(&mut out, head);
            out.extend(tail);
        }
        SoccerPacket::PlayerVariableStats { player, words } => {
            out.extend(player.to_le_bytes());
            for word in words {
                out.extend(word.to_le_bytes());
            }
        }
        SoccerPacket::SetSoccerPlayerOneTimeStats { player, stats } => {
            out.extend(player.to_le_bytes());
            put_f32s(&mut out, stats);
        }
        SoccerPacket::UpdateGameTime { remaining, elapsed } => {
            out.extend(remaining.to_le_bytes());
            out.extend(elapsed.to_le_bytes());
        }
        SoccerPacket::BallBounce { words } => {
            for word in words {
                out.extend(word.to_le_bytes());
            }
        }
        SoccerPacket::Unknown { payload, .. } => out.extend(payload),
    }
    out
}

/// Encode a message in the Soccer family-76 gateway envelope.
pub fn encode(message: &Message) -> Vec<u8> {
    let inner = encode_inner(&message.packet);
    let mut out = Vec::with_capacity(inner.len() + 7);
    out.extend(message.outer.to_le_bytes());
    out.push(u8::from(message.flag));
    out.extend((inner.len() as u32).to_le_bytes());
    out.extend(inner);
    out
}

/// Encode the application body the gateway carries inside a tunnel:
/// `u16 76 | u16 sub | payload`. This is what `Outcome::replies` expects.
pub fn encode_inner(packet: &SoccerPacket) -> Vec<u8> {
    let payload = payload_bytes(packet);
    let mut out = Vec::with_capacity(payload.len() + 4);
    out.extend(TUNNEL_OPCODE.to_le_bytes());
    out.extend(soccer_sub(packet).to_le_bytes());
    out.extend(payload);
    out
}

/// Decode one gateway message carrying Soccer opcode 76.
pub fn decode(bytes: &[u8]) -> io::Result<Message> {
    if bytes.len() < 11 {
        return Err(invalid("truncated soccer envelope"));
    }
    let outer = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    if !matches!(outer, 5 | 6) {
        return Err(invalid("expected gateway tunnel"));
    }
    let flag = match bytes[2] {
        0 => false,
        1 => true,
        _ => return Err(invalid("invalid tunnel flag")),
    };
    let size = u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize;
    if size != bytes.len() - 7 {
        return Err(invalid("soccer envelope length mismatch"));
    }
    let mut inner = &bytes[7..];
    let op = read_u16(&mut inner)?;
    if op != TUNNEL_OPCODE {
        return Err(invalid("not a soccer tunnel"));
    }
    let sub = read_u16(&mut inner)?;
    let packet = decode_packet(sub, inner)?;
    Ok(Message {
        outer,
        flag,
        packet,
    })
}

fn decode_packet(sub: u16, payload: &[u8]) -> io::Result<SoccerPacket> {
    let mut reader = payload;
    let packet = match sub {
        1 => SoccerPacket::UpdatePlayerPos {
            player: read_u64(&mut reader)?,
            position: read_f32s(&mut reader)?,
            tail: read_bytes(&mut reader)?,
        },
        2 => SoccerPacket::GoalMade {
            fields: read_u32s(&mut reader)?,
        },
        3 => SoccerPacket::RegisterPlayer {
            player: read_u64(&mut reader)?,
            marker: read_bytes(&mut reader)?,
        },
        4 => SoccerPacket::UpdateSoccerBall {
            transform: read_f32s(&mut reader)?,
            velocity: read_f32s(&mut reader)?,
            tail: read_bytes(&mut reader)?,
        },
        5 => SoccerPacket::UpdateGameState {
            words: read_u32s(&mut reader)?,
        },
        6 => SoccerPacket::SetPlayerTeam {
            player: read_u64(&mut reader)?,
            team: read_u32(&mut reader)?,
            slot: read_u32(&mut reader)?,
        },
        7 => return decode_client_config(payload),
        8 => decode_team_colours(&mut reader)?,
        9 => SoccerPacket::SpawnPickUp {
            state: read_u8(&mut reader)?,
            definition: read_u32(&mut reader)?,
            marker: read_u8(&mut reader)?,
            position: read_f32s(&mut reader)?,
            scale: read_f32(&mut reader)?,
            instance: read_u32(&mut reader)?,
        },
        10 => SoccerPacket::AcquirePickUp {
            flag: read_u8(&mut reader)?,
            value: read_f32(&mut reader)?,
            player: read_u64(&mut reader)?,
        },
        11 => SoccerPacket::AcquireBall {
            flag: read_u8(&mut reader)?,
            holder: read_u64(&mut reader)?,
            previous: read_u64(&mut reader)?,
        },
        12 => SoccerPacket::FreeBall {
            flag: read_u8(&mut reader)?,
            position: read_f32s(&mut reader)?,
            scale: read_f32(&mut reader)?,
            previous: read_u64(&mut reader)?,
        },
        13 => SoccerPacket::ImpulseBall {
            word: read_u32(&mut reader)?,
            vector: read_f32s::<4>(&mut reader)?,
            value_a: read_f32(&mut reader)?,
            value_b: read_f32(&mut reader)?,
            action: read_u32(&mut reader)?,
            value_c: read_f32(&mut reader)?,
        },
        14 => SoccerPacket::UpdatePlayerState {
            player: read_u64(&mut reader)?,
            state: read_u8(&mut reader)?,
            aux: read_u8(&mut reader)?,
            state_counter: read_u32(&mut reader)?,
            vectors: [
                read_f32s::<4>(&mut reader)?,
                read_f32s::<4>(&mut reader)?,
                read_f32s::<4>(&mut reader)?,
            ],
        },
        15 => SoccerPacket::UpdatePlayerStats {
            player: read_u64(&mut reader)?,
            words: read_u32s(&mut reader)?,
        },
        16 => SoccerPacket::UpdatePlayerControls {
            player: read_u64(&mut reader)?,
            direction: [read_f32(&mut reader)?, read_f32(&mut reader)?],
            controls: read_u32(&mut reader)?,
        },
        18 => SoccerPacket::HighArcBall {
            head: read_f32s(&mut reader)?,
            tail: read_bytes(&mut reader)?,
        },
        19 => SoccerPacket::PlayerVariableStats {
            player: read_u64(&mut reader)?,
            words: read_u32s(&mut reader)?,
        },
        21 => SoccerPacket::SetSoccerPlayerOneTimeStats {
            player: read_u64(&mut reader)?,
            stats: read_f32s(&mut reader)?,
        },
        23 => SoccerPacket::UpdateGameTime {
            remaining: read_f32(&mut reader)?,
            elapsed: read_f32(&mut reader)?,
        },
        25 => SoccerPacket::BallBounce {
            words: read_u32s(&mut reader)?,
        },
        _ => SoccerPacket::Unknown {
            sub,
            payload: payload.to_vec(),
        },
    };
    if !matches!(packet, SoccerPacket::Unknown { .. }) {
        end(reader)?;
    }
    Ok(packet)
}

/// Legacy candidate SetClientConfig offsets; not used as native-map facts.
const CONFIG_ASSET_LENGTH_OFFSET: usize = 124;
const CONFIG_ASSET_OFFSET: usize = 128;
const CONFIG_BALL_SPAWN_OFFSET: usize = 157;

fn decode_client_config(payload: &[u8]) -> io::Result<SoccerPacket> {
    if payload.len() < CONFIG_BALL_SPAWN_OFFSET + 16 {
        return Err(invalid("truncated client config"));
    }
    let length = u32::from_le_bytes(
        payload[CONFIG_ASSET_LENGTH_OFFSET..CONFIG_ASSET_LENGTH_OFFSET + 4]
            .try_into()
            .unwrap(),
    ) as usize;
    let asset_end = CONFIG_ASSET_OFFSET + length;
    if asset_end > payload.len() {
        return Err(invalid("client config ball asset length"));
    }
    let ball_asset = std::str::from_utf8(&payload[CONFIG_ASSET_OFFSET..asset_end])
        .map_err(|_| invalid("client config ball asset not utf-8"))?
        .to_string();
    let mut spawn = [0.0f32; 4];
    for (index, value) in spawn.iter_mut().enumerate() {
        let offset = CONFIG_BALL_SPAWN_OFFSET + index * 4;
        *value = f32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap());
    }
    Ok(SoccerPacket::SetClientConfig {
        raw: payload.to_vec(),
        ball_asset,
        ball_spawn: spawn,
    })
}

fn decode_team_colours(reader: &mut &[u8]) -> io::Result<SoccerPacket> {
    let length_a = read_u32(reader)? as usize;
    if length_a > 64 {
        return Err(invalid("team colour name too long"));
    }
    let team_a_name = std::str::from_utf8(take(reader, length_a)?)
        .map_err(|_| invalid("team a name"))?
        .to_string();
    let length_b = read_u32(reader)? as usize;
    if length_b > 64 {
        return Err(invalid("team colour name too long"));
    }
    let team_b_name = std::str::from_utf8(take(reader, length_b)?)
        .map_err(|_| invalid("team b name"))?
        .to_string();
    Ok(SoccerPacket::TeamColours {
        team_a_name,
        team_b_name,
        team_a_rgba: read_bytes(reader)?,
        team_b_rgba: read_bytes(reader)?,
        team_a_id: read_u32(reader)?,
        team_b_id: read_u32(reader)?,
        tail: read_bytes(reader)?,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct TeamState {
    pub name: String,
    pub colour_rgba: [u8; 4],
    pub definition_id: u32,
}

impl Default for TeamState {
    fn default() -> Self {
        Self {
            name: String::new(),
            colour_rgba: [0; 4],
            definition_id: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    pub teams: [TeamState; 2],
    /// (player, team, slot) for every SetPlayerTeam assignment, ordered by player id.
    pub roster: Vec<(u64, u32, u32)>,
}

impl Session {
    pub fn roster_of_team(&self, team: u32) -> impl Iterator<Item = &(u64, u32, u32)> {
        self.roster.iter().filter(move |entry| entry.1 == team)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    Running,
    Restart,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clock {
    pub remaining: f32,
    pub elapsed: f32,
    /// R fallback duration; the native client consumes clock values supplied
    /// by the server and does not establish a fixed half length.
    pub duration: Evidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Goal {
    pub scoring_team: usize,
    pub score_after: [u32; 2],
    pub source: Evidence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RewardTeam {
    pub name: String,
    pub colour_rgba: [u8; 4],
    pub score: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RewardProposal {
    pub minigame_type_id: i32,
    pub zone: &'static str,
    pub score: [u32; 2],
    pub winner: Option<usize>,
    pub teams: [RewardTeam; 2],
    /// Reconstructed proposal shape; reward rules are missing native evidence.
    pub record: Evidence,
    pub reward_table: Evidence,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Phase::Lobby => "lobby",
            Phase::Running => "running",
            Phase::Restart => "restart",
            Phase::Finished => "finished",
        })
    }
}

/// Legacy message-driven reducer for candidate server outputs. It has no
/// simulated physics and is not the authoritative service path.
#[derive(Clone, Debug)]
pub struct Match {
    session: Session,
    phase: Phase,
    clock: Clock,
    score: [u32; 2],
    possession: Option<u64>,
    last_holder: u64,
    ball_position: Option<[f32; 3]>,
    goals: Vec<Goal>,
    registered: BTreeSet<u64>,
    restart_source: Evidence,
}

impl Default for Match {
    fn default() -> Self {
        Self::new()
    }
}

impl Match {
    pub fn new() -> Self {
        Self {
            session: Session::default(),
            phase: Phase::Lobby,
            clock: Clock {
                remaining: 0.0,
                elapsed: 0.0,
                duration: Evidence::Reconstructed,
            },
            score: [0, 0],
            possession: None,
            last_holder: 0,
            ball_position: None,
            goals: Vec::new(),
            registered: BTreeSet::new(),
            restart_source: Evidence::Missing,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }

    pub fn score(&self) -> [u32; 2] {
        self.score
    }

    pub fn possession(&self) -> Option<u64> {
        self.possession
    }

    pub fn ball_position(&self) -> Option<[f32; 3]> {
        self.ball_position
    }

    pub fn goals(&self) -> &[Goal] {
        &self.goals
    }

    pub fn restart_source(&self) -> Evidence {
        self.restart_source
    }

    pub fn player_team(&self, player: u64) -> Option<(u32, u32)> {
        self.session
            .roster
            .iter()
            .find(|entry| entry.0 == player)
            .map(|entry| (entry.1, entry.2))
    }

    /// Apply one gateway message with its caller-supplied direction label.
    ///
    /// `ServerToClient` messages drive the authoritative state machine.
    /// `ClientToServer` messages are decoded and returned, but they never
    /// change match state; a client message carrying a server-only packet id
    /// (for example a spoofed `GoalMade`) is rejected outright, so untrusted
    /// bytes cannot score.
    pub fn handle(&mut self, direction: Direction, bytes: &[u8]) -> io::Result<Message> {
        let message = decode(bytes)?;
        if direction == Direction::ClientToServer {
            if !CLIENT_PACKET_IDS.contains(&soccer_sub(&message.packet)) {
                return Err(invalid(
                    "client may not send server-authoritative soccer packets",
                ));
            }
            return Ok(message);
        }
        match &message.packet {
            SoccerPacket::TeamColours {
                team_a_name,
                team_b_name,
                team_a_rgba,
                team_b_rgba,
                team_a_id,
                team_b_id,
                ..
            } => {
                self.session.teams[0] = TeamState {
                    name: team_a_name.clone(),
                    colour_rgba: *team_a_rgba,
                    definition_id: *team_a_id,
                };
                self.session.teams[1] = TeamState {
                    name: team_b_name.clone(),
                    colour_rgba: *team_b_rgba,
                    definition_id: *team_b_id,
                };
            }
            SoccerPacket::RegisterPlayer { player, .. } => {
                self.registered.insert(*player);
            }
            SoccerPacket::SetPlayerTeam { player, team, slot } => {
                if *team > 1 || *slot > 3 {
                    return Err(invalid("soccer team slot out of range"));
                }
                if let Some((current_team, current_slot)) = self.player_team(*player) {
                    if current_team == *team && current_slot == *slot {
                        // R: tolerate a repeated identical assignment.
                    } else {
                        return Err(invalid("soccer player team reassignment"));
                    }
                } else if self
                    .session
                    .roster
                    .iter()
                    .any(|entry| entry.1 == *team && entry.2 == *slot)
                {
                    return Err(invalid("soccer team slot already taken"));
                } else {
                    self.session.roster.push((*player, *team, *slot));
                    self.session.roster.sort();
                }
            }
            SoccerPacket::UpdateGameTime { remaining, elapsed } => {
                self.apply_game_time(*remaining, *elapsed)?;
            }
            SoccerPacket::GoalMade { fields } => {
                self.apply_goal(*fields)?;
            }
            SoccerPacket::AcquireBall {
                holder, previous, ..
            } => {
                if *previous != self.last_holder {
                    return Err(invalid("soccer possession chain mismatch"));
                }
                if !self.registered.contains(holder) {
                    return Err(invalid("soccer possession by an unregistered player"));
                }
                self.possession = Some(*holder);
                self.last_holder = *holder;
            }
            SoccerPacket::FreeBall {
                position,
                scale,
                previous,
                ..
            } => {
                if *previous != self.last_holder {
                    return Err(invalid("soccer free ball chain mismatch"));
                }
                if !position.iter().all(|value| value.is_finite()) || !scale.is_finite() {
                    return Err(invalid("non-finite soccer free ball geometry"));
                }
                self.possession = None;
                self.ball_position = Some(*position);
            }
            SoccerPacket::UpdateSoccerBall {
                transform,
                velocity,
                ..
            } => {
                if !transform
                    .iter()
                    .chain(velocity.iter())
                    .all(|value| value.is_finite())
                {
                    return Err(invalid("non-finite soccer ball transform"));
                }
                self.ball_position = Some([transform[12], transform[13], transform[14]]);
            }
            _ => {}
        }
        Ok(message)
    }

    fn apply_game_time(&mut self, remaining: f32, elapsed: f32) -> io::Result<()> {
        if !remaining.is_finite() || !elapsed.is_finite() || remaining < 0.0 {
            return Err(invalid("invalid soccer clock"));
        }
        match self.phase {
            Phase::Lobby => {
                self.clock.elapsed = elapsed;
                if remaining > 0.0 {
                    // R: this reducer caps its clock at the configured fallback.
                    if remaining > HALF_SECONDS + 0.001 {
                        return Err(invalid("soccer clock exceeds the reconstructed fallback"));
                    }
                    self.clock.remaining = remaining;
                    self.phase = Phase::Running;
                    self.ball_position.get_or_insert([0.0; 3]);
                }
            }
            Phase::Running => {
                if elapsed < self.clock.elapsed {
                    return Err(invalid("soccer clock elapsed went backwards"));
                }
                if remaining > self.clock.remaining + 0.001 {
                    return Err(invalid("soccer clock remaining increased"));
                }
                self.clock.remaining = remaining;
                self.clock.elapsed = elapsed;
                if remaining <= 0.0 {
                    // R: transition to the reducer's terminal phase.
                    self.phase = Phase::Finished;
                }
            }
            Phase::Restart => {
                // R: after a goal, this reducer permits a clock hold but does
                // not claim that the native server uses this transition.
                if remaining > self.clock.remaining + 0.001 {
                    return Err(invalid("soccer clock ran up during restart hold"));
                }
                if elapsed < self.clock.elapsed {
                    return Err(invalid("soccer clock elapsed went backwards"));
                }
                self.clock.remaining = remaining;
                self.clock.elapsed = elapsed;
            }
            Phase::Finished => {
                self.clock.elapsed = elapsed;
            }
        }
        Ok(())
    }

    fn apply_goal(&mut self, fields: [u32; 3]) -> io::Result<()> {
        // R: accept a one-step score increase while running and hold at restart.
        if self.phase != Phase::Running {
            return Err(invalid("soccer goal outside a running half"));
        }
        if fields[2] != fields[0] + fields[1] {
            return Err(invalid("soccer goal scoreboard inconsistent"));
        }
        let next = [fields[0], fields[1]];
        let scoring_team = if next[1] == self.score[1] + 1 && next[0] == self.score[0] {
            1
        } else if next[0] == self.score[0] + 1 && next[1] == self.score[1] {
            0
        } else {
            return Err(invalid("soccer goal did not advance the scoreboard"));
        };
        self.score = next;
        self.goals.push(Goal {
            scoring_team,
            score_after: next,
            source: Evidence::Reconstructed,
        });
        // R: hold the clock until a server-controlled restart.
        self.phase = Phase::Restart;
        self.possession = None;
        Ok(())
    }

    /// Reconstructed restart: resume the held half clock. Returns the
    /// Candidate UpdateGameTime packet; resume behavior is R.
    pub fn restart(&mut self) -> Vec<Vec<u8>> {
        if self.phase != Phase::Restart {
            return Vec::new();
        }
        self.phase = Phase::Running;
        self.restart_source = Evidence::Reconstructed;
        vec![encode(&Message {
            outer: 5,
            flag: FLAG,
            packet: SoccerPacket::UpdateGameTime {
                remaining: self.clock.remaining,
                elapsed: self.clock.elapsed,
            },
        })]
    }

    /// Reconstructed completion proposal; no Soccer reward table is recovered.
    pub fn reward_proposal(&self) -> RewardProposal {
        let winner = if self.score[0] > self.score[1] {
            Some(0)
        } else if self.score[1] > self.score[0] {
            Some(1)
        } else {
            None
        };
        RewardProposal {
            minigame_type_id: SOCCER_TYPE_ID,
            zone: ZONE,
            score: self.score,
            winner,
            teams: [
                RewardTeam {
                    name: self.session.teams[0].name.clone(),
                    colour_rgba: self.session.teams[0].colour_rgba,
                    score: self.score[0],
                },
                RewardTeam {
                    name: self.session.teams[1].name.clone(),
                    colour_rgba: self.session.teams[1].colour_rgba,
                    score: self.score[1],
                },
            ],
            record: Evidence::Reconstructed,
            reward_table: Evidence::Missing,
        }
    }
}
