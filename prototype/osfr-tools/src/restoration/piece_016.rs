//! Work item 016: minigame loss and retry on the original `op39` family.
//!
//! ORIGINAL, recovered:
//!
//! * Every `op39` message is `u16 39 | u8 sub-opcode | i32 StateId | i32 GroupId | i32 GameId`
//!   inside the shared tunnel envelope (`u16 outer | u8 flag | u32 inner length`). This layout is
//!   taken from the community Sanctuary serializer pinned at PR #76 head
//!   `9236220c44086ba9a31393cc30b5ebd44d05787f` (`BaseMiniGamePacket.cs`, sha256
//!   `548de79e7d1200fb655bd432d15e6c03b26e22049d64a01ae67ad7d791442024`), matches the native
//!   constructor `FUN_008a4420` (writes `0x27` then sub, state, id, id), and parses all nine
//!   preserved `lab/reports/original-minigame-packets/` minigame frames byte-exactly.
//! * Loss presentation is `39/18` (`MiniGameGameOverPacket`): `u8 Won` then three `i32`
//!   (`0, 0, 1` in the recovered serializer). The native handler `FUN_009ad480`
//!   ("MiniGame:OnGameOver") stores that byte at state `+99` and the three ints at
//!   `+0x70/+0x74/+0x78`. The community comment and the native loss run agree that this byte
//!   flips the end card between the win presentation and the `TRY AGAIN!` card (locale id
//!   `401577`; `Retry` is locale id `432500`).
//! * `39/23` (`MiniGameKnockOutPacket`) is `i32 CurrentKnockouts | i32 MaxKnockouts`; seven such
//!   server-to-client frames are preserved in the three captures and re-encode byte-exactly.
//! * `39/19` (`MiniGameStateRemovePacket`) is a bare header. `39/17`
//!   (`MiniGameGameStartPacket`) is a bare header that acknowledges the client's `39/5` start
//!   request (`FUN_009be7d0` "MiniGame:RequestStartGame"). `39/7` is the client leave request
//!   (`FUN_009bf190` "MiniGame:LeaveMiniGame", two preserved frames). `39/11` is the client
//!   restart request (`FUN_009bcb60` "MiniGame:RequestRestartGame").
//!
//! RECONSTRUCTED, labelled: [`Ledger`] is server policy, not recovered code. The original server
//! that owned the `StateId` lifecycle was not recovered, so this module defines the smallest
//! contract that satisfies the item: a start or retry mints a fresh `StateId` that is never
//! reused, a loss or quit settles the instance with no reward, a finish for a state that is not
//! the current one (a stale finish from before a retry) is rejected, and only a win on the
//! current, unsettled state returns one [`RewardProposal`]. A proposal carries ids only: the
//! amounts and the atomic grant stay with the owning engine and the lead-owned economy layer.
//!
//! GAPS: no captured or recovered traffic carries `39/16` (info), `39/18` (game over), `39/19`
//! (state removal), `39/47` (end score) or `39/67` (create-game result); family `26` (the
//! `CommandPacketClosedMinigameEndScreen` end-screen close) never appears in the captures either.
//! The `GroupId`/`GameId` field split is unverified by capture (both are `-1` in every preserved
//! frame), `39/11` has no recovered name, and no reward contents or amounts are recovered.

pub const MINIGAME_INNER_OPCODE: u16 = 39;
pub const SERVER_TUNNEL_OUTER_OPCODE: u16 = 5;
pub const CLIENT_TUNNEL_OUTER_OPCODE: u16 = 6;
/// 7-byte tunnel envelope plus the two-byte inner opcode and the one-byte sub-opcode.
pub const MINIGAME_FRAME_HEADER_LEN: usize = 10;
/// `u16 39 | u8 sub | i32 StateId | i32 GroupId | i32 GameId`.
pub const BASE_HEADER_LEN: usize = 15;
/// `StateId`/`GroupId`/`GameId` sentinel meaning "the current state".
pub const CURRENT_STATE: i32 = -1;

pub const SUB_START_REQUEST: u8 = 5;
pub const SUB_LEAVE_REQUEST: u8 = 7;
pub const SUB_RESTART_REQUEST: u8 = 11;
pub const SUB_GAME_START: u8 = 17;
pub const SUB_GAME_OVER: u8 = 18;
pub const SUB_STATE_REMOVE: u8 = 19;
pub const SUB_KNOCK_OUT: u8 = 23;

const GAME_OVER_LEN: usize = BASE_HEADER_LEN + 13;
const KNOCK_OUT_LEN: usize = BASE_HEADER_LEN + 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    TooShort,
    UnsupportedOuterOpcode(u16),
    UnsupportedTunnelFlag(u8),
    UnsupportedInnerOpcode(u16),
    LengthMismatch { declared: usize, available: usize },
    MalformedBody(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseHeader {
    pub state_id: i32,
    pub group_id: i32,
    pub game_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnockOut {
    pub state_id: i32,
    pub group_id: i32,
    pub game_id: i32,
    pub current: i32,
    pub max: i32,
}

impl KnockOut {
    /// Knockouts still available, as the recovered HUD reads them (`Max - Current`).
    pub fn remaining(&self) -> i32 {
        self.max - self.current
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameOver {
    pub state_id: i32,
    pub group_id: i32,
    pub game_id: i32,
    pub won: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MinigameFrame<'a> {
    pub outer_opcode: u16,
    pub tunnel_flag: u8,
    pub subtype: u8,
    /// The inner `op39` message: base header plus any subtype payload.
    pub inner: &'a [u8],
}

impl MinigameFrame<'_> {
    pub fn knock_out(&self) -> Result<KnockOut, FrameError> {
        if self.subtype != SUB_KNOCK_OUT || self.inner.len() != KNOCK_OUT_LEN {
            return Err(FrameError::MalformedBody("not a knockout body"));
        }
        let header = decode_base_header(self.inner)?;
        Ok(KnockOut {
            state_id: header.state_id,
            group_id: header.group_id,
            game_id: header.game_id,
            current: read_i32(self.inner, BASE_HEADER_LEN),
            max: read_i32(self.inner, BASE_HEADER_LEN + 4),
        })
    }

    pub fn game_over(&self) -> Result<GameOver, FrameError> {
        if self.subtype != SUB_GAME_OVER || self.inner.len() != GAME_OVER_LEN {
            return Err(FrameError::MalformedBody("not a game-over body"));
        }
        let header = decode_base_header(self.inner)?;
        Ok(GameOver {
            state_id: header.state_id,
            group_id: header.group_id,
            game_id: header.game_id,
            won: self.inner[BASE_HEADER_LEN] != 0,
        })
    }
}

pub fn decode_frame(packet: &[u8]) -> Result<MinigameFrame<'_>, FrameError> {
    if packet.len() < MINIGAME_FRAME_HEADER_LEN {
        return Err(FrameError::TooShort);
    }
    let outer_opcode = u16::from_le_bytes([packet[0], packet[1]]);
    if outer_opcode != SERVER_TUNNEL_OUTER_OPCODE && outer_opcode != CLIENT_TUNNEL_OUTER_OPCODE {
        return Err(FrameError::UnsupportedOuterOpcode(outer_opcode));
    }
    // Both recovered flag values occur: 1 on the sequenced streams and 0 on the bulk streams.
    if packet[2] > 1 {
        return Err(FrameError::UnsupportedTunnelFlag(packet[2]));
    }
    let inner_opcode = u16::from_le_bytes([packet[7], packet[8]]);
    if inner_opcode != MINIGAME_INNER_OPCODE {
        return Err(FrameError::UnsupportedInnerOpcode(inner_opcode));
    }
    let declared = u32::from_le_bytes([packet[3], packet[4], packet[5], packet[6]]) as usize;
    let inner = &packet[7..];
    if declared != inner.len() {
        return Err(FrameError::LengthMismatch {
            declared,
            available: inner.len(),
        });
    }
    Ok(MinigameFrame {
        outer_opcode,
        tunnel_flag: packet[2],
        subtype: packet[9],
        inner,
    })
}

pub fn decode_base_header(inner: &[u8]) -> Result<BaseHeader, FrameError> {
    if inner.len() < BASE_HEADER_LEN {
        return Err(FrameError::MalformedBody("short base header"));
    }
    if u16::from_le_bytes([inner[0], inner[1]]) != MINIGAME_INNER_OPCODE {
        return Err(FrameError::MalformedBody("not an op39 message"));
    }
    Ok(BaseHeader {
        state_id: read_i32(inner, 3),
        group_id: read_i32(inner, 7),
        game_id: read_i32(inner, 11),
    })
}

pub fn encode_base_header(subtype: u8, state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    let mut out = MINIGAME_INNER_OPCODE.to_le_bytes().to_vec();
    out.push(subtype);
    for value in [state_id, group_id, game_id] {
        out.extend(value.to_le_bytes());
    }
    out
}

pub fn encode_game_start(state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    encode_base_header(SUB_GAME_START, state_id, group_id, game_id)
}

pub fn encode_state_remove(state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    encode_base_header(SUB_STATE_REMOVE, state_id, group_id, game_id)
}

/// `39/18` with the recovered `Won` byte and the three serializer defaults `0, 0, 1`.
pub fn encode_game_over(won: bool, state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    let mut out = encode_base_header(SUB_GAME_OVER, state_id, group_id, game_id);
    out.push(u8::from(won));
    for value in [0_i32, 0, 1] {
        out.extend(value.to_le_bytes());
    }
    out
}

pub fn encode_knock_out(knockout: &KnockOut) -> Vec<u8> {
    let mut out = encode_base_header(
        SUB_KNOCK_OUT,
        knockout.state_id,
        knockout.group_id,
        knockout.game_id,
    );
    out.extend(knockout.current.to_le_bytes());
    out.extend(knockout.max.to_le_bytes());
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Playing,
    Won,
    Lost,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Won,
    Lost,
    Quit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    NoCurrentState,
    AlreadyPlaying,
    StaleState { offered: i32, current: i32 },
    AlreadySettled,
    Exhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceState {
    pub state_id: i32,
    pub group_id: i32,
    pub game_id: i32,
    pub phase: Phase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartAck {
    pub state_id: i32,
    pub replies: Vec<Vec<u8>>,
}

/// A win on the current, unsettled instance. Ids only: no amount is recovered, so no amount is
/// invented here. The owning engine and the lead-owned economy layer commit any real grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardProposal {
    pub state_id: i32,
    pub game_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finish {
    pub won: bool,
    pub replies: Vec<Vec<u8>>,
    pub reward: Option<RewardProposal>,
}

/// Reconstructed server-side instance policy for one player and one minigame state stream.
///
/// `StateId` is the recovered instance identity: the start acknowledgement echoes it and the
/// client's restart request names it. The ledger never reuses an id, so a late finish that names
/// a previous instance is always distinguishable from the current one.
pub struct Ledger {
    next_state_id: i32,
    current: Option<InstanceState>,
}

impl Ledger {
    /// `first_state_id` is the engine's chosen namespace base; ids count up from it.
    pub fn new(first_state_id: i32) -> Ledger {
        Ledger {
            next_state_id: first_state_id,
            current: None,
        }
    }

    pub fn current(&self) -> Option<&InstanceState> {
        self.current.as_ref()
    }

    /// Mints the next never-reused `StateId` and acknowledges it with `39/17`.
    fn mint(&mut self, group_id: i32, game_id: i32) -> Result<StartAck, Reject> {
        if self.next_state_id == i32::MAX {
            return Err(Reject::Exhausted);
        }
        let state_id = self.next_state_id;
        self.next_state_id += 1;
        self.current = Some(InstanceState {
            state_id,
            group_id,
            game_id,
            phase: Phase::Playing,
        });
        Ok(StartAck {
            state_id,
            replies: vec![encode_game_start(state_id, group_id, game_id)],
        })
    }

    /// First instance. A second start before the current one settles is rejected.
    pub fn start(&mut self, group_id: i32, game_id: i32) -> Result<StartAck, Reject> {
        if self.current.is_some() {
            return Err(Reject::AlreadyPlaying);
        }
        self.mint(group_id, game_id)
    }

    /// The original retry path (`39/11`, "MiniGame:RequestRestartGame"). Requires the named
    /// instance to be the current one and to be terminal, and always mints a fresh id.
    pub fn retry(&mut self, state_id: i32) -> Result<StartAck, Reject> {
        let current = match self.current {
            Some(current) => current,
            None => return Err(Reject::NoCurrentState),
        };
        if state_id != current.state_id {
            return Err(Reject::StaleState {
                offered: state_id,
                current: current.state_id,
            });
        }
        if current.phase == Phase::Playing {
            return Err(Reject::AlreadyPlaying);
        }
        self.mint(current.group_id, current.game_id)
    }

    /// Settles the named instance. A win on the current, unsettled instance returns one reward
    /// proposal; a loss, a quit, a stale id and a repeated finish never do.
    pub fn finish(&mut self, state_id: i32, outcome: Outcome) -> Result<Finish, Reject> {
        let current = match self.current {
            Some(current) => current,
            None => return Err(Reject::NoCurrentState),
        };
        if state_id != current.state_id {
            return Err(Reject::StaleState {
                offered: state_id,
                current: current.state_id,
            });
        }
        if current.phase != Phase::Playing {
            return Err(Reject::AlreadySettled);
        }
        let (won, phase) = match outcome {
            Outcome::Won => (true, Phase::Won),
            Outcome::Lost => (false, Phase::Lost),
            Outcome::Quit => (false, Phase::Quit),
        };
        self.current = Some(InstanceState { phase, ..current });
        let mut replies = Vec::new();
        if outcome != Outcome::Quit {
            replies.push(encode_game_over(
                won,
                state_id,
                current.group_id,
                current.game_id,
            ));
        }
        replies.push(encode_state_remove(
            state_id,
            current.group_id,
            current.game_id,
        ));
        Ok(Finish {
            won,
            replies,
            reward: won.then_some(RewardProposal {
                state_id,
                game_id: current.game_id,
            }),
        })
    }
}

fn read_i32(buffer: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap())
}
