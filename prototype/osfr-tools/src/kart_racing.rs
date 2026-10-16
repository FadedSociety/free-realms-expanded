//! Kart Racing (original lobby game type 1) bounded session contract.
//!
//! Evidence labels used below (`ORIGINAL` / `RECONSTRUCTED`) follow `notes/Research/Kart Racing
//! Restoration.md`. Everything marked ORIGINAL is read from the supplied client, the supplied
//! community server data, or the archived original capture; RECONSTRUCTED means this module supplies
//! a bounded placeholder that the lead must replace with native evidence before any original-content
//! claim. Open native gates are listed in the research note.
//!
//! ORIGINAL wire facts (archived capture `packets_racing_minigame.pcapng`, decoded bodies in
//! `lab/reports/original-minigame-packets/racing-*.bin`):
//! * racing is a subtype stream of tunneled world opcodes 5 and 6 carrying inner opcode 33 and then a
//!   u16 subtype; in this capture every opcode-33 body carries outer opcode 5 in both directions, so
//!   direction comes from the connection and is never read off the outer opcode;
//! * the tunnel flag is 0 or 1 in the capture (12,855 flag-0 and 9,534 flag-1 of 22,389 opcode-33
//!   bodies). The flag is preserved verbatim and its meaning is not recovered here, so it is never
//!   interpreted as direction, reliability, or validity beyond the 0/1 range;
//! * observed subtypes and payload sizes are recorded in
//!   `lab/reports/kart-racing-20261009T0158Z/kart-racing-wire-schema.json`;
//! * subtype 11 opens with `u32 name byte count` plus the kart body asset name. The captured names are
//!   length-prefixed and not NUL-terminated: the bytes that follow are the attribute block, which
//!   starts with the `f32` 400.0 (`00 00 c8 43`) in all six frames. An empty, truncated, embedded-NUL
//!   or non-UTF-8 name window is rejected instead of yielding a partial name.
//!
//! ORIGINAL native facts (Ghidra export `decompiled/client/FreeRealms/pseudocode.c`): the client owns
//! `ControllerVehicleRace`, `BaseVehicleRacePacket`, `VehicleRacePacketGateHit`,
//! `VehicleRacePacketSetRaceConfig`, `VehicleRacePacketUpdateVehicleRaceProgressData` and the
//! proxied mine/missile/pickup classes, while `ControllerFlashMiniGame` is the separate path used by
//! the cooking/forging movies. Kart Racing is therefore a native vehicle-race controller, not a Flash
//! minigame.
//!
//! RECONSTRUCTED here: lap counts, concrete checkpoint ids and their order, timer semantics, and the
//! reward schedule. No original numeric reward or lap count is known yet, so `RewardProposal` stays
//! `Unpriced` unless the caller supplies an explicit schedule.
//!
//! Reward authority: a grant is derived only from the session's own validated state. `claim_reward`
//! takes no proposal, so a `RaceResultProposal` returned by `finish` is an audit record and never a
//! bearer token: only a `Finished` session whose ordered, in-order checkpoint run counts every
//! configured lap can grant, and only when the lead explicitly priced the track (`TrackLayout.reward`
//! is `Some`). Tracks whose checkpoint ids/order/geometry or physics are still unknown stay unpriced
//! and therefore NOT reward-eligible; this module never invents gate geometry, positions, or payouts.
//!
//! Deliberate boundaries:
//! * The two bulk racer streams (payloads of 85 and 73 bytes) are treated as opaque observations and
//!   both captured tunnel flags are accepted losslessly. Their layout is not decoded, so no position
//!   from a client is ever authoritative and no progress is derived from them. Progress comes only
//!   from ordered checkpoint ids; see `observe_racer_state`.
//! * `placement` is always `None`: race-wide placement needs server state this module does not own.
//! * Session ownership (which member may send checkpoints for a session) is gateway state; this module
//!   exposes `session_id` for that binding but cannot verify it alone.
//!
//! ponytail: one flat session struct and one ordered checkpoint list; no trait objects, no per-racer
//! map, no timers. Add per-racer state only when the native `UpdateVehicleRaceClientData` layout is
//! decoded and a second racer must be simulated.
use serde::{Deserialize, Serialize};

/// ORIGINAL: lobby game definition type for kart racing (community `LobbyGameDefinition` comment).
pub const KART_RACE_GAME_TYPE: i32 = 1;
/// ORIGINAL: lobby game definition type for demolition derby.
pub const DEMO_DERBY_GAME_TYPE: i32 = 2;
/// ORIGINAL: client locale id 3052 = "Kart Race Lobby".
pub const KART_RACE_LOBBY_NAME_ID: i32 = 3052;
/// ORIGINAL: client locale id 3053 = "Demo Derby Lobby".
pub const DEMO_DERBY_LOBBY_NAME_ID: i32 = 3053;
/// ORIGINAL: supplied server `PacketSendSelfToClient.json` "Start of Kart Driver" job GUID.
pub const KART_DRIVER_JOB_GUID: i32 = 48;
/// ORIGINAL: that job's `JobName` locale id (`Kart Driver`).
pub const KART_DRIVER_JOB_NAME_ID: i32 = 2917;
/// ORIGINAL: inner opcode of the tunneled racing subtype stream.
pub const VEHICLE_RACE_INNER_OPCODE: u16 = 33;
/// ORIGINAL: payload bytes of the bulk racer state subtype (archived file length 96).
pub const RACER_STATE_BODY_LEN: usize = 85;
/// ORIGINAL: payload bytes of the bulk race-progress subtype (archived file length 84).
pub const RACE_PROGRESS_BODY_LEN: usize = 73;
/// ORIGINAL: tunnel outer opcodes observed for application messages. Both are accepted; the archived
/// racing capture carries every opcode-33 body on outer 5 in both directions, so this value does not
/// encode direction.
pub const TUNNEL_OUTER_OPCODES: [u16; 2] = [5, 6];
/// RECONSTRUCTED guard: a track never has more checkpoints than this.
pub const MAX_CHECKPOINTS: usize = 512;
/// RECONSTRUCTED guard: a race never has more laps than this.
pub const MAX_LAPS: u32 = 99;

/// ORIGINAL speedway identity (id, name locale id, POI id) from the supplied community
/// `PointOfInterests` resource, corroborated by the public wiki's five-speedway list.
/// RECONSTRUCTED: zone placement is not verified here and is deliberately omitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speedway {
    pub id: u16,
    pub name_id: i32,
    pub poi_id: u32,
}

/// ORIGINAL: Wildwood, Thunder Falls, Frost Ridge, Tangletrack and Sandstorm speedways.
pub const SPEEDWAYS: [Speedway; 5] = [
    Speedway {
        id: 31,
        name_id: 18206,
        poi_id: 31,
    },
    Speedway {
        id: 32,
        name_id: 3606,
        poi_id: 32,
    },
    Speedway {
        id: 33,
        name_id: 382855,
        poi_id: 33,
    },
    Speedway {
        id: 34,
        name_id: 382856,
        poi_id: 34,
    },
    Speedway {
        id: 39,
        name_id: 401623,
        poi_id: 39,
    },
];

/// One lobby game catalogue row. The unchanged community writer emits key + `id` + `game_type` +
/// `unknown` + `name_id`; the original capture's first row carries the same 37/3030 pair, so this
/// encoder mirrors the community layout exactly and is verified against its compiled output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LobbyGameEntry {
    pub id: i32,
    pub game_type: i32,
    pub unknown: i32,
    pub name_id: i32,
}

/// Community serializer subset of the lobby game definitions. This mirrors the pinned Sanctuary
/// writer byte-for-byte, but it is **not** the native opcode 102/2 wire framing: the captured 2,393
/// byte payload carries a leading `u32` remaining-length field (2378), a row count of 6 and further
/// blocks after the rows. Do not use this encoder as a live 102/2 catalogue response until the native
/// framing is recovered; that remainder is an open native gate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LobbyGameDefinition {
    pub entries: Vec<LobbyGameEntry>,
}

impl LobbyGameDefinition {
    pub fn new(entries: Vec<LobbyGameEntry>) -> Self {
        Self { entries }
    }

    /// Byte-identical to the unchanged community serializer for the same entries in insertion order:
    /// `i32 count`, then per entry `i32 key` plus four `i32` fields, then four zero `i32`s.
    /// ORIGINAL evidence shows the original catalogue goes beyond this bounded subset; that remainder
    /// is a native gate and is not encoded here.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.entries.len() * 20 + 16);
        out.extend((self.entries.len() as i32).to_le_bytes());
        for entry in &self.entries {
            out.extend(entry.id.to_le_bytes());
            out.extend(entry.id.to_le_bytes());
            out.extend(entry.game_type.to_le_bytes());
            out.extend(entry.unknown.to_le_bytes());
            out.extend(entry.name_id.to_le_bytes());
        }
        out.extend([0u8; 16]);
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    TooShort,
    UnsupportedOuterOpcode(u16),
    UnsupportedTunnelFlag(u8),
    LengthMismatch {
        declared: usize,
        available: usize,
    },
    UnsupportedInnerOpcode(u16),
    /// The subtype-11 name window is empty, NUL-less, or otherwise not a usable asset name.
    InvalidVehicleName,
    InvalidUtf8,
}

/// A decoded tunneled racing frame. `payload` excludes the tunnel header, the inner opcode and the
/// subtype, so its length is the archived file length minus 11. `flag` is the captured tunnel flag
/// (0 or 1), preserved verbatim and not interpreted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RacingFrame<'a> {
    pub outer_opcode: u16,
    pub flag: u8,
    pub subtype: u16,
    pub payload: &'a [u8],
}

/// Decode `u16 outer | u8 flag | u32 inner length | u16 inner opcode | u16 subtype | payload`,
/// checking every recovered field. Outer opcodes 5 and 6 are accepted and both captured tunnel flags
/// (0 and 1) decode losslessly; anything else is rejected.
pub fn decode_racing_frame(packet: &[u8]) -> Result<RacingFrame<'_>, FrameError> {
    const HEADER: usize = 7;
    if packet.len() < HEADER {
        return Err(FrameError::TooShort);
    }
    let outer_opcode = u16::from_le_bytes([packet[0], packet[1]]);
    if !TUNNEL_OUTER_OPCODES.contains(&outer_opcode) {
        return Err(FrameError::UnsupportedOuterOpcode(outer_opcode));
    }
    let flag = packet[2];
    if flag > 1 {
        return Err(FrameError::UnsupportedTunnelFlag(flag));
    }
    let declared = u32::from_le_bytes([packet[3], packet[4], packet[5], packet[6]]) as usize;
    let available = packet.len() - HEADER;
    if declared != available {
        return Err(FrameError::LengthMismatch {
            declared,
            available,
        });
    }
    let inner = &packet[HEADER..];
    if inner.len() < 4 {
        return Err(FrameError::TooShort);
    }
    let inner_opcode = u16::from_le_bytes([inner[0], inner[1]]);
    if inner_opcode != VEHICLE_RACE_INNER_OPCODE {
        return Err(FrameError::UnsupportedInnerOpcode(inner_opcode));
    }
    Ok(RacingFrame {
        outer_opcode,
        flag,
        subtype: u16::from_le_bytes([inner[2], inner[3]]),
        payload: &inner[4..],
    })
}

/// ORIGINAL subtype 11 layout: `u32 name byte count` then exactly that many name bytes
/// (`kt_kart08_body_bbe.cdt`, `sg_kt_speedster_body_bbec_002.cdt`, ...). The captured layout is
/// length-prefixed, not NUL-terminated: every decoded frame continues straight into the attribute
/// block, which begins with `00 00 c8 43` (400.0). A declared window that is empty, truncated, or
/// contains a NUL byte is rejected instead of producing a partial name.
pub fn vehicle_body_asset(payload: &[u8]) -> Result<&str, FrameError> {
    if payload.len() < 8 {
        return Err(FrameError::TooShort);
    }
    let name_len = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    if name_len == 0 {
        return Err(FrameError::InvalidVehicleName);
    }
    let end = 4usize.checked_add(name_len).ok_or(FrameError::TooShort)?;
    if end > payload.len() {
        return Err(FrameError::TooShort);
    }
    let name = &payload[4..end];
    if name.contains(&0) {
        return Err(FrameError::InvalidVehicleName);
    }
    std::str::from_utf8(name).map_err(|_| FrameError::InvalidUtf8)
}

/// RECONSTRUCTED: the lead decides these numbers; nothing recovered so far fixes them. A track whose
/// checkpoint ids, order or geometry are still unknown must stay unpriced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RewardSchedule {
    pub coins: u32,
    pub xp: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewardProposal {
    /// No original reward evidence exists for this track, so nothing may be granted.
    Unpriced,
    Granted {
        coins: u32,
        xp: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KartRacingError {
    NoCheckpoints,
    ZeroCheckpoint,
    DuplicateCheckpoint(u32),
    TooManyCheckpoints,
    ZeroLaps,
    TooManyLaps,
    /// A priced schedule must price something; `coins` and `xp` cannot both be zero.
    EmptyRewardSchedule,
    NotInLobby,
    NotInCountdown,
    NotRacing,
    NotFinished,
    /// The caller-supplied timestamp moved backwards; nothing was recorded.
    ClockRegression,
    RewardAlreadyClaimed,
    CorruptSnapshot,
}

/// RECONSTRUCTED checkpoint order. ORIGINAL evidence proves gates exist
/// (`VehicleRacePacketGateHit`) and that the wire carries per-track state; the concrete ids, their
/// order and the lap count still need native confirmation. A track in that state stays unpriced
/// (`reward: None`) and is therefore not reward-eligible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackLayout {
    pub speedway: u16,
    pub laps: u32,
    pub checkpoints: Vec<u32>,
    pub reward: Option<RewardSchedule>,
}

impl TrackLayout {
    pub fn new(
        speedway: u16,
        laps: u32,
        checkpoints: Vec<u32>,
        reward: Option<RewardSchedule>,
    ) -> Result<Self, KartRacingError> {
        let layout = Self {
            speedway,
            laps,
            checkpoints,
            reward,
        };
        layout.validate()?;
        Ok(layout)
    }

    pub fn validate(&self) -> Result<(), KartRacingError> {
        if self.checkpoints.is_empty() {
            return Err(KartRacingError::NoCheckpoints);
        }
        if self.checkpoints.len() > MAX_CHECKPOINTS {
            return Err(KartRacingError::TooManyCheckpoints);
        }
        if self.checkpoints.iter().any(|id| *id == 0) {
            return Err(KartRacingError::ZeroCheckpoint);
        }
        for (index, id) in self.checkpoints.iter().enumerate() {
            if self.checkpoints[..index].contains(id) {
                return Err(KartRacingError::DuplicateCheckpoint(*id));
            }
        }
        if self.laps == 0 {
            return Err(KartRacingError::ZeroLaps);
        }
        if self.laps > MAX_LAPS {
            return Err(KartRacingError::TooManyLaps);
        }
        if let Some(schedule) = self.reward
            && schedule.coins == 0
            && schedule.xp == 0
        {
            return Err(KartRacingError::EmptyRewardSchedule);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Lobby,
    Countdown,
    Racing,
    Finished,
    Abandoned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckpointOutcome {
    CheckpointAccepted {
        lap: u32,
        index: usize,
    },
    LapCompleted {
        lap: u32,
        lap_ms: u64,
    },
    /// Re-delivery of the checkpoint already counted. Idempotent by design.
    Duplicate,
    OutOfOrder {
        expected: u32,
        got: u32,
    },
    UnknownCheckpoint(u32),
    /// The observation's timestamp is older than the current lap's start. Nothing was recorded.
    ClockRegression {
        now_ms: u64,
        since_ms: u64,
    },
    NotRacing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationOutcome {
    Recorded { bytes: usize },
    UnexpectedLength { bytes: usize },
}

/// Persisted session state. `reward` is the lead-supplied price schedule (defaults to `None` for
/// older snapshots, which therefore restore unpriced and cannot grant).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSnapshot {
    pub session_id: u32,
    pub speedway: u16,
    pub laps: u32,
    pub checkpoints: Vec<u32>,
    #[serde(default)]
    pub reward: Option<RewardSchedule>,
    pub phase: Phase,
    pub started_ms: u64,
    pub lap_started_ms: u64,
    pub next_index: usize,
    pub laps_completed: u32,
    pub best_lap_ms: Option<u64>,
    pub last_checkpoint: Option<u32>,
    pub reward_claimed: bool,
    pub observed_racer_states: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RaceResultProposal {
    pub session_id: u32,
    pub speedway: u16,
    pub laps_completed: u32,
    pub total_ms: u64,
    pub best_lap_ms: Option<u64>,
    /// Always `None`: placement needs server-wide race state, which this module does not own.
    pub placement: Option<u32>,
    pub reward: RewardProposal,
}

/// One kart-racing session for one player. Progress is accepted only from ordered checkpoint ids.
pub struct RaceSession {
    session_id: u32,
    layout: TrackLayout,
    phase: Phase,
    started_ms: u64,
    lap_started_ms: u64,
    next_index: usize,
    laps_completed: u32,
    best_lap_ms: Option<u64>,
    last_checkpoint: Option<u32>,
    reward_claimed: bool,
    observed_racer_states: u32,
}

impl RaceSession {
    pub fn open(
        session_id: u32,
        layout: TrackLayout,
        now_ms: u64,
    ) -> Result<Self, KartRacingError> {
        layout.validate()?;
        Ok(Self {
            session_id,
            layout,
            phase: Phase::Lobby,
            started_ms: now_ms,
            lap_started_ms: now_ms,
            next_index: 0,
            laps_completed: 0,
            best_lap_ms: None,
            last_checkpoint: None,
            reward_claimed: false,
            observed_racer_states: 0,
        })
    }

    /// Session identity for gateway-side ownership binding. Claims themselves remain session-local.
    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn laps_completed(&self) -> u32 {
        self.laps_completed
    }

    pub fn best_lap_ms(&self) -> Option<u64> {
        self.best_lap_ms
    }

    pub fn next_checkpoint(&self) -> Option<u32> {
        self.layout.checkpoints.get(self.next_index).copied()
    }

    pub fn observed_racer_states(&self) -> u32 {
        self.observed_racer_states
    }

    pub fn begin(&mut self, _now_ms: u64) -> Result<(), KartRacingError> {
        if self.phase != Phase::Lobby {
            return Err(KartRacingError::NotInLobby);
        }
        self.phase = Phase::Countdown;
        Ok(())
    }

    pub fn start(&mut self, now_ms: u64) -> Result<(), KartRacingError> {
        if self.phase != Phase::Countdown {
            return Err(KartRacingError::NotInCountdown);
        }
        if now_ms < self.started_ms {
            return Err(KartRacingError::ClockRegression);
        }
        self.phase = Phase::Racing;
        self.started_ms = now_ms;
        self.lap_started_ms = now_ms;
        Ok(())
    }

    /// Validate one checkpoint observation. Only the configured next gate is accepted; the gate that
    /// was already counted is an idempotent duplicate, and an observation whose timestamp predates the
    /// current lap is rejected before anything changes.
    pub fn on_checkpoint(&mut self, now_ms: u64, checkpoint_id: u32) -> CheckpointOutcome {
        if self.phase != Phase::Racing {
            return CheckpointOutcome::NotRacing;
        }
        let Some(expected) = self.next_checkpoint() else {
            return CheckpointOutcome::NotRacing;
        };
        if now_ms < self.lap_started_ms {
            return CheckpointOutcome::ClockRegression {
                now_ms,
                since_ms: self.lap_started_ms,
            };
        }
        if checkpoint_id == expected {
            let index = self.next_index;
            self.last_checkpoint = Some(checkpoint_id);
            self.next_index += 1;
            if self.next_index < self.layout.checkpoints.len() {
                return CheckpointOutcome::CheckpointAccepted {
                    lap: self.laps_completed + 1,
                    index,
                };
            }
            let lap_ms = now_ms.saturating_sub(self.lap_started_ms);
            self.laps_completed += 1;
            self.next_index = 0;
            self.lap_started_ms = now_ms;
            if self.best_lap_ms.is_none_or(|best| lap_ms < best) {
                self.best_lap_ms = Some(lap_ms);
            }
            if self.laps_completed >= self.layout.laps {
                self.phase = Phase::Finished;
            }
            return CheckpointOutcome::LapCompleted {
                lap: self.laps_completed,
                lap_ms,
            };
        }
        if self.last_checkpoint == Some(checkpoint_id) {
            return CheckpointOutcome::Duplicate;
        }
        if self.layout.checkpoints.contains(&checkpoint_id) {
            return CheckpointOutcome::OutOfOrder {
                expected,
                got: checkpoint_id,
            };
        }
        CheckpointOutcome::UnknownCheckpoint(checkpoint_id)
    }

    /// Record an opaque bulk racer observation for diagnostics. This never advances lap or checkpoint
    /// progress: a client position must not be authoritative, and the payload layout is not decoded
    /// here. Both captured payload sizes are accepted regardless of the tunnel flag they arrived on.
    pub fn observe_racer_state(&mut self, body: &[u8]) -> ObservationOutcome {
        if body.len() == RACER_STATE_BODY_LEN || body.len() == RACE_PROGRESS_BODY_LEN {
            self.observed_racer_states = self.observed_racer_states.saturating_add(1);
            ObservationOutcome::Recorded { bytes: body.len() }
        } else {
            ObservationOutcome::UnexpectedLength { bytes: body.len() }
        }
    }

    /// Read the final result of a `Finished` race. This is an audit record, not a grant: it may be
    /// called repeatedly and carries no claim authority. A clock that predates the race start is
    /// rejected instead of reporting a zero-length total.
    pub fn finish(&self, now_ms: u64) -> Result<RaceResultProposal, KartRacingError> {
        if self.phase != Phase::Finished {
            return Err(KartRacingError::NotFinished);
        }
        if now_ms < self.started_ms {
            return Err(KartRacingError::ClockRegression);
        }
        let reward = match self.layout.reward {
            Some(schedule) => RewardProposal::Granted {
                coins: schedule.coins,
                xp: schedule.xp,
            },
            None => RewardProposal::Unpriced,
        };
        Ok(RaceResultProposal {
            session_id: self.session_id,
            speedway: self.layout.speedway,
            laps_completed: self.laps_completed,
            total_ms: now_ms.saturating_sub(self.started_ms),
            best_lap_ms: self.best_lap_ms,
            placement: None,
            reward,
        })
    }

    pub fn abandon(&mut self) {
        self.phase = Phase::Abandoned;
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            session_id: self.session_id,
            speedway: self.layout.speedway,
            laps: self.layout.laps,
            checkpoints: self.layout.checkpoints.clone(),
            reward: self.layout.reward,
            phase: self.phase,
            started_ms: self.started_ms,
            lap_started_ms: self.lap_started_ms,
            next_index: self.next_index,
            laps_completed: self.laps_completed,
            best_lap_ms: self.best_lap_ms,
            last_checkpoint: self.last_checkpoint,
            reward_claimed: self.reward_claimed,
            observed_racer_states: self.observed_racer_states,
        }
    }

    /// Reconnect support: rebuild exactly the stored progress and validate it first. Timestamps are
    /// kept as recorded so a resumed race keeps its lap timing, and the price schedule travels with
    /// the snapshot so a legitimately finished race is still payable after a restart. Incoherent
    /// snapshots (progress fields that contradict each other, a claimed marker outside a finished
    /// priced run, or an empty price schedule) are rejected as `CorruptSnapshot`.
    pub fn restore(snapshot: SessionSnapshot) -> Result<Self, KartRacingError> {
        let layout = TrackLayout {
            speedway: snapshot.speedway,
            laps: snapshot.laps,
            checkpoints: snapshot.checkpoints.clone(),
            reward: snapshot.reward,
        };
        layout
            .validate()
            .map_err(|_| KartRacingError::CorruptSnapshot)?;

        if snapshot.next_index >= layout.checkpoints.len() || snapshot.laps_completed > layout.laps
        {
            return Err(KartRacingError::CorruptSnapshot);
        }
        let checkpoints = &layout.checkpoints;
        let expected_last = if snapshot.next_index == 0 {
            if snapshot.laps_completed == 0 {
                None
            } else {
                Some(checkpoints[checkpoints.len() - 1])
            }
        } else {
            Some(checkpoints[snapshot.next_index - 1])
        };
        let phase_coherent = match snapshot.phase {
            Phase::Lobby | Phase::Countdown => {
                snapshot.laps_completed == 0
                    && snapshot.next_index == 0
                    && snapshot.last_checkpoint.is_none()
            }
            Phase::Racing => snapshot.laps_completed < layout.laps,
            Phase::Finished => snapshot.laps_completed == layout.laps && snapshot.next_index == 0,
            Phase::Abandoned => true,
        };
        let coherent = snapshot.lap_started_ms >= snapshot.started_ms
            && (snapshot.laps_completed == 0) == snapshot.best_lap_ms.is_none()
            && snapshot.last_checkpoint == expected_last
            && phase_coherent
            && (!snapshot.reward_claimed
                || (snapshot.laps_completed == layout.laps
                    && layout.reward.is_some()
                    && matches!(snapshot.phase, Phase::Finished | Phase::Abandoned)));
        if !coherent {
            return Err(KartRacingError::CorruptSnapshot);
        }
        Ok(Self {
            session_id: snapshot.session_id,
            layout,
            phase: snapshot.phase,
            started_ms: snapshot.started_ms,
            lap_started_ms: snapshot.lap_started_ms,
            next_index: snapshot.next_index,
            laps_completed: snapshot.laps_completed,
            best_lap_ms: snapshot.best_lap_ms,
            last_checkpoint: snapshot.last_checkpoint,
            reward_claimed: snapshot.reward_claimed,
            observed_racer_states: snapshot.observed_racer_states,
        })
    }

    /// The only grant authority in this module. The award is derived from this session's own
    /// validated state, so no caller can supply or forge an amount: the session must be `Finished`
    /// with every configured lap counted, and the layout must carry an explicit price schedule.
    /// Unknown checkpoint geometry or physics is therefore never reward-eligible. An unpriced (or
    /// unpriced-by-default reconstruction) run returns `Unpriced` and never consumes the one-shot
    /// grant. A priced grant is paid at most once per session.
    pub fn claim_reward(&mut self) -> Result<RewardProposal, KartRacingError> {
        if self.phase != Phase::Finished || self.laps_completed != self.layout.laps {
            return Err(KartRacingError::NotFinished);
        }
        match self.layout.reward {
            None => Ok(RewardProposal::Unpriced),
            Some(schedule) => {
                if self.reward_claimed {
                    return Err(KartRacingError::RewardAlreadyClaimed);
                }
                self.reward_claimed = true;
                Ok(RewardProposal::Granted {
                    coins: schedule.coins,
                    xp: schedule.xp,
                })
            }
        }
    }
}
