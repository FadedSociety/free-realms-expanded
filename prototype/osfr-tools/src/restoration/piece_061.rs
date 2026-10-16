//! Piece 061 - dungeon waves and gates: the recovered tutorial-cave room chain,
//! its two static gate actors and the reconstructed ordered-completion policy.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/061 Dungeon waves and gates.md`):
//!
//! * `sg_npx_cavern_01Areas.xml` (8600 bytes, extracted from `Assets_002.pack`
//!   index 3872, sha256 `aae003c3...`): the tutorial room boxes
//!   `AMB_FIRST_ROOM` (6116) `[261,12,268]`-`[316,42,304]`,
//!   `AMB_TUT_SECOND_ROOM` (6114) `[316,8,224]`-`[446,55,340]`,
//!   `AMB_TUT_FINAL_ROOM` (6113) `[292,-6,351]`-`[432,79,486]` and the
//!   `STING_TUTORIAL` sting (6115) `[273,25,287]`-`[274,29,288]`.
//! * Original terrain runtime objects in `sg_npx_cavern_01_4_4.gcnk`
//!   (pack_01/001617.gcnk, sha256 `5f26ced5...`):
//!   `sg_npx_cavern_01_door_01.adr` object 1358893701 at
//!   `[314.99448,28.665304,288.69412]` rotation `[-1.5707964,0,0,0]` and
//!   `sg_npx_cavern_01_door_02.adr` object 1362704680 at
//!   `[360.06262,33.38316,345.52478]` rotation `[-3.1415927,0,0,0]`.
//! * Original asset-manifest rows: the door_01 clip set
//!   {stand, open, open_stand, close}, the rock-barrier clips
//!   {loc_stand_retract, loc_shoot_up, loc_stand_deployed} and the interactive
//!   verb sounds `OBJ_DOOR_LRG_OPEN_VERB.mp3` / `OBJ_DOOR_LRG_CLOSE_VERB.mp3`.
//!
//! RECONSTRUCTED, labelled at each call site: the ordered-completion policy, the
//! caller-supplied wave sizes (the original per-room actor counts are
//! unrecovered) and the server-side gate and position enforcement.
//!
//! NOT RECOVERED, therefore not invented here: any trigger volume or door
//! script. The recovered area table contains only ambient, music and room-type
//! records, so the gate-opening trigger lived outside that table; no NPX server
//! script or dynamic spawn table was recovered. This module never grants items,
//! currency or XP, and it does not claim native behaviour.

use std::collections::BTreeSet;

/// Axis-aligned area box as recorded in the original area table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AreaBox {
    /// `AreaDefinition name` attribute.
    pub area_name: &'static str,
    /// `AreaDefinition id` attribute.
    pub area_id: u32,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl AreaBox {
    pub fn contains(&self, position: [f32; 3]) -> bool {
        (0..3).all(|axis| position[axis] >= self.min[axis] && position[axis] <= self.max[axis])
    }

    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
}

/// First tutorial room; the recovered spawn sample `[287,28,286]` sits inside it.
pub const FIRST_ROOM: AreaBox = AreaBox {
    area_name: "AMB_FIRST_ROOM",
    area_id: 6116,
    min: [261.0, 12.0, 268.0],
    max: [316.0, 42.0, 304.0],
};

/// Second tutorial room. Its near wall plane `x = 316` is shared with the first room.
pub const SECOND_ROOM: AreaBox = AreaBox {
    area_name: "AMB_TUT_SECOND_ROOM",
    area_id: 6114,
    min: [316.0, 8.0, 224.0],
    max: [446.0, 55.0, 340.0],
};

/// Final tutorial room, entered through the passage that holds `door_02`.
pub const FINAL_ROOM: AreaBox = AreaBox {
    area_name: "AMB_TUT_FINAL_ROOM",
    area_id: 6113,
    min: [292.0, -6.0, 351.0],
    max: [432.0, 79.0, 486.0],
};

/// Tutorial sting box; the only remaining tutorial-named area record.
pub const TUTORIAL_STING: AreaBox = AreaBox {
    area_name: "STING_TUTORIAL",
    area_id: 6115,
    min: [273.0, 25.0, 287.0],
    max: [274.0, 29.0, 288.0],
};

/// Rooms of the reconstructed ordered chain, in recovered tutorial order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RoomId {
    First,
    Second,
    Final,
}

impl RoomId {
    /// The three rooms in the order the original area names record.
    pub fn tutorial_order() -> [RoomId; 3] {
        [RoomId::First, RoomId::Second, RoomId::Final]
    }

    pub fn bounds(self) -> &'static AreaBox {
        match self {
            RoomId::First => &FIRST_ROOM,
            RoomId::Second => &SECOND_ROOM,
            RoomId::Final => &FINAL_ROOM,
        }
    }

    pub fn index(self) -> usize {
        match self {
            RoomId::First => 0,
            RoomId::Second => 1,
            RoomId::Final => 2,
        }
    }
}

/// A gate actor instance recorded in the original terrain runtime objects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GateInstance {
    pub actor: &'static str,
    pub object_id: u32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}

/// `door_01`, on the shared first/second room wall plane `x = 316`.
pub const DOOR_01: GateInstance = GateInstance {
    actor: "sg_npx_cavern_01_door_01.adr",
    object_id: 1_358_893_701,
    position: [314.99448, 28.665_304, 288.694_12],
    rotation: [-1.570_796_4, 0.0, 0.0, 0.0],
    scale: 1.0,
};

/// `door_02`, in the passage between the second room's far wall and the final room.
pub const DOOR_02: GateInstance = GateInstance {
    actor: "sg_npx_cavern_01_door_02.adr",
    object_id: 1_362_704_680,
    position: [360.062_62, 33.383_16, 345.524_78],
    rotation: [-3.141_592_7, 0.0, 0.0, 0.0],
    scale: 1.0,
};

/// Original door_01 clip rows; `open`/`open_stand` are the open state and
/// `stand`/`close` the closed state.
pub const DOOR_01_CLIPS: [&str; 4] = [
    "sg_npx_cavern_01_door_01_stand.gr2",
    "sg_npx_cavern_01_door_01_open.gr2",
    "sg_npx_cavern_01_door_01_open_stand.gr2",
    "sg_npx_cavern_01_door_01_close.gr2",
];

/// Original rock-barrier deploy/retract clips. The barrier actor ships without a
/// recovered placement in the preserved tutorial tiles; only its clips are claimed.
pub const ROCK_BARRIER_CLIPS: [&str; 3] = [
    "_npx_cavern_01_rockbarrier_01_loc_stand_retract.gr2",
    "_npx_cavern_01_rockbarrier_01_loc_shoot_up.gr2",
    "_npx_cavern_01_rockbarrier_01_loc_stand_deployed.gr2",
];

/// Interactive door verb sounds; a door is an object the player can activate.
pub const DOOR_VERB_SOUNDS: [&str; 2] =
    ["OBJ_DOOR_LRG_OPEN_VERB.mp3", "OBJ_DOOR_LRG_CLOSE_VERB.mp3"];

/// The two recovered gates of the tutorial chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateId {
    Door01,
    Door02,
}

impl GateId {
    pub fn all() -> [GateId; 2] {
        [GateId::Door01, GateId::Door02]
    }

    pub fn instance(self) -> &'static GateInstance {
        match self {
            GateId::Door01 => &DOOR_01,
            GateId::Door02 => &DOOR_02,
        }
    }

    /// Room the player is in before walking through this gate.
    pub fn room_before(self) -> RoomId {
        match self {
            GateId::Door01 => RoomId::First,
            GateId::Door02 => RoomId::Second,
        }
    }

    /// Room this gate leads into; it stays locked until `room_before` is cleared.
    pub fn room_after(self) -> RoomId {
        match self {
            GateId::Door01 => RoomId::Second,
            GateId::Door02 => RoomId::Final,
        }
    }

    /// The gate that leads into `room`, when one exists.
    pub fn leading_into(room: RoomId) -> Option<GateId> {
        match room {
            RoomId::First => None,
            RoomId::Second => Some(GateId::Door01),
            RoomId::Final => Some(GateId::Door02),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateState {
    Closed,
    Open,
}

/// Caller-supplied wave size for one room.
///
/// The original tutorial wave composition is unrecovered, so this module never
/// ships a count of its own; the owning encounter content passes the total it
/// has evidence for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaveSpec {
    room: RoomId,
    total_actors: u32,
}

impl WaveSpec {
    pub fn new(room: RoomId, total_actors: u32) -> Self {
        Self { room, total_actors }
    }

    pub fn room(&self) -> RoomId {
        self.room
    }

    pub fn total_actors(&self) -> u32 {
        self.total_actors
    }
}

/// One enemy defeat that the owning encounter module already validated.
///
/// A raw client packet or a reported kill count is not a validated clear and
/// must never reach this type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidatedClear {
    room: RoomId,
    actor_instance: u64,
}

impl ValidatedClear {
    pub fn enemy_defeated(room: RoomId, actor_instance: u64) -> Self {
        Self {
            room,
            actor_instance,
        }
    }

    pub fn room(&self) -> RoomId {
        self.room
    }

    pub fn actor_instance(&self) -> u64 {
        self.actor_instance
    }
}

/// What one validated clear changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClearOutcome {
    pub room: RoomId,
    pub cleared_actors: u32,
    pub total_actors: u32,
    pub room_cleared: bool,
    /// Set only on the clear that completed a room and unlocked its gate.
    pub gate_opened: Option<GateId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateError {
    /// The supplied wave specs do not cover the three recovered rooms exactly once.
    InvalidWaveSpec,
    /// Actor instance 0 is not a real spawned instance.
    InvalidActorInstance,
    /// This actor instance was already credited.
    DuplicateClear,
    /// The clear names a room the chain has not reached yet.
    OutOfOrderClear(RoomId),
    /// A gate request arrived before the room behind it was cleared.
    GateLocked(GateId),
    /// A position report would move the player past a closed gate.
    BypassAttempt { room: RoomId, blocked_by: GateId },
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::InvalidWaveSpec => write!(f, "wave specs do not cover the recovered rooms"),
            GateError::InvalidActorInstance => write!(f, "actor instance id 0 is not an instance"),
            GateError::DuplicateClear => write!(f, "actor instance already cleared"),
            GateError::OutOfOrderClear(room) => {
                write!(f, "clear for room {room:?} arrived out of order")
            }
            GateError::GateLocked(gate) => write!(f, "gate {gate:?} is still locked"),
            GateError::BypassAttempt { room, blocked_by } => {
                write!(f, "room {room:?} is behind closed gate {blocked_by:?}")
            }
        }
    }
}

impl std::error::Error for GateError {}

/// Ordered tutorial-cave progression: each room's wave must be cleared before
/// its gate opens and before the next room accepts movement or clears.
///
/// This is reconstructed server policy, not recovered original logic; the
/// recovered inputs are the room boxes, the gate actors and their placements.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DungeonChain {
    waves: Vec<WaveSpec>,
    cleared: [BTreeSet<u64>; 3],
    gates: [GateState; 2],
}

impl DungeonChain {
    /// Build a chain from one wave spec per recovered room.
    pub fn new(waves: Vec<WaveSpec>) -> Result<Self, GateError> {
        if waves.len() != RoomId::tutorial_order().len() {
            return Err(GateError::InvalidWaveSpec);
        }
        for room in RoomId::tutorial_order() {
            if !waves.iter().any(|spec| spec.room == room) {
                return Err(GateError::InvalidWaveSpec);
            }
        }
        if waves.iter().any(|spec| spec.total_actors == 0) {
            return Err(GateError::InvalidWaveSpec);
        }
        if RoomId::tutorial_order()
            .iter()
            .enumerate()
            .any(|(index, room)| waves[index].room != *room)
        {
            // Keep the stored order identical to the recovered room order so the
            // remaining methods can index by room.
            return Err(GateError::InvalidWaveSpec);
        }
        Ok(Self {
            waves,
            cleared: Default::default(),
            gates: [GateState::Closed; 2],
        })
    }

    /// The first room whose wave is not complete; `None` once the chain is done.
    pub fn active_room(&self) -> Option<RoomId> {
        RoomId::tutorial_order()
            .into_iter()
            .find(|room| !self.room_cleared(*room))
    }

    pub fn wave(&self, room: RoomId) -> &WaveSpec {
        &self.waves[room.index()]
    }

    pub fn cleared_actors(&self, room: RoomId) -> u32 {
        self.cleared[room.index()].len() as u32
    }

    pub fn room_cleared(&self, room: RoomId) -> bool {
        self.cleared_actors(room) >= self.waves[room.index()].total_actors
    }

    pub fn gate_state(&self, gate: GateId) -> GateState {
        self.gates[gate.room_after().index() - 1]
    }

    pub fn is_complete(&self) -> bool {
        self.active_room().is_none()
    }

    /// Credit one validated clear. Only the active room accepts clears, and only
    /// the clear that completes a room opens the following gate.
    pub fn record_clear(&mut self, clear: ValidatedClear) -> Result<ClearOutcome, GateError> {
        if clear.actor_instance == 0 {
            return Err(GateError::InvalidActorInstance);
        }
        if self.active_room() != Some(clear.room) {
            return Err(GateError::OutOfOrderClear(clear.room));
        }
        let index = clear.room.index();
        if self.cleared[index].contains(&clear.actor_instance) {
            return Err(GateError::DuplicateClear);
        }
        let total = self.waves[index].total_actors;
        self.cleared[index].insert(clear.actor_instance);
        let cleared_actors = self.cleared_actors(clear.room);
        let room_cleared = cleared_actors >= total;
        let gate_opened = if room_cleared {
            GateId::all()
                .into_iter()
                .find(|gate| gate.room_before() == clear.room)
                .map(|gate| {
                    self.gates[gate.room_after().index() - 1] = GateState::Open;
                    gate
                })
        } else {
            None
        };
        Ok(ClearOutcome {
            room: clear.room,
            cleared_actors,
            total_actors: total,
            room_cleared,
            gate_opened,
        })
    }

    /// Handle a player's door request.
    ///
    /// The request alone can never open a gate: the gate must already be unlocked
    /// by the completed wave of the room behind it, otherwise the request is
    /// refused. Repeated requests for an unlocked gate are idempotent.
    pub fn request_gate(&mut self, gate: GateId) -> Result<GateState, GateError> {
        if !self.room_cleared(gate.room_before()) {
            return Err(GateError::GateLocked(gate));
        }
        self.gates[gate.room_after().index() - 1] = GateState::Open;
        Ok(GateState::Open)
    }

    /// Resolve a player position against the recovered room boxes.
    ///
    /// Returns `Ok(None)` outside every recovered box (the corridors and the
    /// world beyond the map), `Ok(Some(room))` for a legal position, and
    /// [`GateError::BypassAttempt`] when the position is inside a room whose
    /// gate has not been opened yet.
    pub fn report_position(&self, position: [f32; 3]) -> Result<Option<RoomId>, GateError> {
        let room = RoomId::tutorial_order()
            .into_iter()
            .find(|room| room.bounds().contains(position));
        let Some(room) = room else {
            return Ok(None);
        };
        // Report the earliest closed gate on the path into this room.
        let blocking = GateId::all()
            .into_iter()
            .filter(|gate| gate.room_after().index() <= room.index())
            .find(|gate| matches!(self.gate_state(*gate), GateState::Closed));
        if let Some(gate) = blocking {
            return Err(GateError::BypassAttempt {
                room,
                blocked_by: gate,
            });
        }
        Ok(Some(room))
    }
}
