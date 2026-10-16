//! Work item 060: dungeon instance lifecycle (membership, spawn/reset, completion, exit,
//! restart/disconnect).
//!
//! Evidence labels follow the other restoration modules: ORIGINAL is read from the supplied retail
//! captures, the supplied client binary, the pinned community serializers or the pinned wiki corpus;
//! RECONSTRUCTED is a bounded local policy the lead must replace with original server evidence;
//! MISSING is an explicit gap.
//!
//! ORIGINAL facts:
//! * The dungeon entry family is `BaseEncounterPacket` = **41**, with a `u16` sub-opcode and a
//!   12-byte header `[i32 encounter id][i32 instance id]` after it. The supplied client and the
//!   working local serializer `src/encounter.rs` (`encounter_header`) both write that layout, and
//!   the community serializer for the unmerged Sanctuary PR #76
//!   (`9236220c44086ba9a31393cc30b5ebd44d05787f`, `BaseEncounterPacket.cs` sha256
//!   `d1499ed14953925e9b612d05111da8e8ffd2694bccf959c4c33c95154496dabe`) documents the same size.
//! * Three retail capture frames in `lab/reports/original-minigame-packets/` re-encode byte for byte
//!   through the encoders below: `demoderby-23087-flow0-frame5078-op41-sub2.bin` (28 B,
//!   sha256 `ecf5eda4a3dcd73ea1ab492ed676ad235880d4f32d9acdb5b80e31f68603e7bf`) is
//!   `41/2` = header + `u64` player guid + one trailing byte; `demoderby-22789-flow0-frame119-op41-sub106.bin`
//!   (23 B, sha256 `625e25eaf19c75cf2d157bcff9c2d6ca5d0b9ff2995155375f326f9fde354b91`) is
//!   `41/106` = header + `i32` state; `demoderby-22822-flow0-frame153-op41-sub107.bin` (19 B,
//!   sha256 `95c4e6234252b93af4c520607e2cb77802434e800d356632191f4b9b9661c3c3`) is `41/107` =
//!   the bare header. These frames were captured from a minigame activity, so they prove the family
//!   header and the three bodies, not dungeon-specific field values.
//! * The entry walk uses encounter states `4` ready -> `5` after GO -> `6` running, after `41/107`
//!   flips the offer popup into the GO button (`EncounterStatePacket.cs` sha256
//!   `ccc56cc24a10d3b6f5af1c9357b7abfe998077d8e4d2b9e09dcd965e309f1a6a`,
//!   `EncounterZoneIsReadyPacket.cs` sha256
//!   `4e1c6aff1bc36b07bc1ac3ba3d7641d4007467019a2a5754c8e211d4f2b3ca5a`). The client handlers are
//!   present in the shipped binary as `MiniGame:SetMiniGameReady`, `MiniGame:RequestStartGame`,
//!   `MiniGame:LeaveMiniGame`, `MiniGame:RequestRestartGame` and `MiniGame:OnGameOver`
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`, sha256
//!   `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`, lines 1195728,
//!   1206499, 1205447, 1205032 and 1192885).
//! * Settlement reuses the minigame family 39 layout recovered by work item 016 from nine preserved
//!   retail frames and the native constructor `FUN_008a4420`: `u16 39 | u8 sub | i32 state |
//!   i32 group | i32 game`, `39/18` = header + `u8 Won` + three `i32` (`0, 0, 1`), `39/19` = bare
//!   header. There is no captured `39/18` or `39/19` frame - the same gap piece 016 records.
//! * Membership is instance scoped in the corpus: the pinned wiki page "Dungeons" (revision
//!   `49285`, `wiki.sqlite`) says dungeons are combat minigames with static entrances and gates
//!   entry by difficulty (1 and 2 "can be entered by anyone"; 3/4/5 require a combat job of at
//!   least level 5/9/13). Page "Bixie Hive" (revision `49119`) describes bonus rewards granting "all
//!   players in the instance", and page "Groups" (revision `45235`) defines groups as two or more
//!   players who fight in dungeons together.
//!
//! RECONSTRUCTED here (no original server instance manager was recovered):
//! * every membership transition and its ordering, the roster freeze at the GO packet, one live
//!   instance per player, the spawn-generation guard, reset semantics, once-only completion, exit
//!   and disconnect/reconnect/reap policy, and the instance-id namespace.
//! * `capacity` and `grace_ms` are caller supplied because neither the original party size nor any
//!   disconnect grace period was recovered. Nothing here invents one.
//!
//! MISSING / explicit gaps:
//! * no captured dungeon (op41) instance traffic exists; the three op41 vectors above are activity
//!   frames. The unmerged PR #76 bodies are community reconstruction, not retail bytes.
//! * no wipe/defeat condition, enemy health, wave/gate rule (work item 061) or reward amount (work
//!   item 063) is modelled. [`Lifecycle::reset`] is called by the owning engine when it decides a
//!   run must restart; this module only validates ordering and generation monotonicity.
//! * no roster-removal, invite, kick or leader packet is written: only the four bodies with recovered
//!   byte layouts are emitted. Membership changes outside the terminal packet set are silent.
//! * no reward is granted. [`Completion`] carries ids and the roster, never amounts.
//!
//! ponytail: one map of members, one optional spawn record and one membership index. No trait
//! objects, no packet writer for unrecovered bodies, no tick loop. Integration note for the lead:
//! the terminal encoders duplicate the proven layouts in `src/encounter.rs` and piece 016 so this
//! module can be exercised standalone; register one canonical codec when the pieces merge.

use std::collections::{BTreeMap, BTreeSet};

/// ORIGINAL: `BaseEncounterPacket` opcode.
pub const ENCOUNTER_FAMILY: u16 = 41;
/// ORIGINAL: `EncounterPacketPlayerEnter` (`41/2`).
pub const SUB_PLAYER_ENTER: u16 = 2;
/// ORIGINAL: `EncounterStatePacket` (`41/106`).
pub const SUB_STATE: u16 = 106;
/// ORIGINAL: `EncounterZoneIsReadyPacket` (`41/107`).
pub const SUB_ZONE_READY: u16 = 107;
/// ORIGINAL: the header is the two `u16` fields plus two `i32` identity fields.
pub const ENCOUNTER_HEADER_LEN: usize = 12;
/// ORIGINAL: the recovered encounter state walk.
pub const STATE_OFFER: i32 = 2;
pub const STATE_READY: i32 = 4;
pub const STATE_GO: i32 = 5;
pub const STATE_RUNNING: i32 = 6;
/// ORIGINAL: minigame settlement family (`BaseMiniGamePacket`, recovered by work item 016).
pub const MINIGAME_FAMILY: u16 = 39;
pub const SUB_LEAVE_REQUEST: u8 = 7;
pub const SUB_RESTART_REQUEST: u8 = 11;
pub const SUB_GAME_OVER: u8 = 18;
pub const SUB_STATE_REMOVE: u8 = 19;
pub const MINIGAME_HEADER_LEN: usize = 15;
/// ORIGINAL: the launch details body defines the minigame state under the *encounter* id, and the
/// recovered `39/x` frames carry the current-state sentinel in the remaining two identity fields.
pub const CURRENT_STATE: i32 = -1;

/// ORIGINAL: the recovered dungeon difficulty gate from the pinned wiki corpus. `None` means the
/// page text does not describe an entry requirement for that value.
pub fn required_combat_level(difficulty: u8) -> Option<u32> {
    match difficulty {
        1 | 2 => Some(0),
        3 => Some(5),
        4 => Some(9),
        5 => Some(13),
        _ => None,
    }
}

/// ORIGINAL layout: `u16 41 | u16 sub | i32 encounter | i32 instance`.
pub fn encode_encounter_header(sub: u16, encounter_id: i32, instance_id: i32) -> Vec<u8> {
    let mut bytes = ENCOUNTER_FAMILY.to_le_bytes().to_vec();
    bytes.extend(sub.to_le_bytes());
    bytes.extend(encounter_id.to_le_bytes());
    bytes.extend(instance_id.to_le_bytes());
    bytes
}

/// ORIGINAL body (`41/2`): header + `u64` player guid + one trailing byte. Every preserved frame
/// carries `0` there.
pub fn encode_player_enter(encounter_id: i32, instance_id: i32, guid: u64) -> Vec<u8> {
    let mut bytes = encode_encounter_header(SUB_PLAYER_ENTER, encounter_id, instance_id);
    bytes.extend(guid.to_le_bytes());
    bytes.push(0);
    bytes
}

/// ORIGINAL body (`41/106`): header + `i32` state.
pub fn encode_encounter_state(encounter_id: i32, instance_id: i32, state: i32) -> Vec<u8> {
    let mut bytes = encode_encounter_header(SUB_STATE, encounter_id, instance_id);
    bytes.extend(state.to_le_bytes());
    bytes
}

/// ORIGINAL body (`41/107`): the bare header, nothing else.
pub fn encode_zone_ready(encounter_id: i32, instance_id: i32) -> Vec<u8> {
    encode_encounter_header(SUB_ZONE_READY, encounter_id, instance_id)
}

/// ORIGINAL layout recovered by piece 016: `u16 39 | u8 sub | i32 state | i32 group | i32 game`.
pub fn encode_minigame_header(sub: u8, state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    let mut bytes = MINIGAME_FAMILY.to_le_bytes().to_vec();
    bytes.push(sub);
    for value in [state_id, group_id, game_id] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// ORIGINAL body (`39/18`): header + `u8 Won` + the recovered `0, 0, 1` triple.
pub fn encode_game_over(won: bool, state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    let mut bytes = encode_minigame_header(SUB_GAME_OVER, state_id, group_id, game_id);
    bytes.push(u8::from(won));
    for value in [0_i32, 0, 1] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// ORIGINAL body (`39/19`): the bare minigame header.
pub fn encode_state_remove(state_id: i32, group_id: i32, game_id: i32) -> Vec<u8> {
    encode_minigame_header(SUB_STATE_REMOVE, state_id, group_id, game_id)
}

/// RECONSTRUCTED: caller supplied limits. The original party size and disconnect grace are not
/// recovered; nothing here invents a default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub capacity: usize,
    pub grace_ms: u64,
}

/// RECONSTRUCTED server phases. `Ready` means every connected member finished loading and the GO
/// packet was emitted; `Active` means the run is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Loading,
    Ready,
    Active,
    Completed,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Member {
    Joined,
    Ready,
    Disconnected { deadline_ms: u64 },
}

impl Member {
    pub fn is_connected(self) -> bool {
        !matches!(self, Member::Disconnected { .. })
    }

    fn deadline(self) -> Option<u64> {
        match self {
            Member::Disconnected { deadline_ms } => Some(deadline_ms),
            _ => None,
        }
    }
}

/// RECONSTRUCTED: the current spawn set of one generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub generation: u32,
    pub actors: usize,
    pub defeated: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub instance_id: i32,
    pub encounter_id: i32,
    /// Everyone who was still in the instance when it completed, including graced members.
    pub roster: Vec<u64>,
    pub generation: u32,
    /// `39/18` won followed by `39/19` state removal. No reward bytes are present.
    pub packets: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    UnknownInstance,
    UnknownDifficulty(u8),
    InstanceClosed,
    RosterClosed,
    AlreadyMember,
    InstanceFull,
    NotEligible { required: u32, actual: u32 },
    AlreadyStarted,
    NotReady,
    NotActive,
    NotAMember,
    NotConnected,
    NotDisconnected,
    MembershipExpired,
    NothingExpired,
    AlreadySpawned,
    NothingToReset,
    EmptySpawn,
    DuplicateActor(u64),
    ActorIdReused(u64),
    UnknownActor(u64),
    AlreadyDefeated(u64),
    EnemiesAlive(usize),
    AlreadyCompleted,
    NoConnectedMember,
    OtherInstance(i32),
    Exhausted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SpawnState {
    generation: u32,
    actors: BTreeSet<u64>,
    defeated: BTreeSet<u64>,
}

impl SpawnState {
    fn summary(&self) -> Spawn {
        Spawn {
            generation: self.generation,
            actors: self.actors.len(),
            defeated: self.defeated.len(),
        }
    }
}

/// RECONSTRUCTED lifecycle for one dungeon instance. The owner is the encounter engine: this type
/// validates ordering and emits only the recovered bodies it can prove.
#[derive(Clone, Debug)]
pub struct Lifecycle {
    instance_id: i32,
    encounter_id: i32,
    difficulty: u8,
    config: Config,
    phase: Phase,
    members: BTreeMap<u64, Member>,
    spawn: Option<SpawnState>,
    /// Actor ids already used by an earlier generation of this instance.
    spent: BTreeSet<u64>,
    next_generation: u32,
}

impl Lifecycle {
    pub fn new(instance_id: i32, encounter_id: i32, difficulty: u8, config: Config) -> Self {
        Lifecycle {
            instance_id,
            encounter_id,
            difficulty,
            config,
            phase: Phase::Loading,
            members: BTreeMap::new(),
            spawn: None,
            spent: BTreeSet::new(),
            next_generation: 0,
        }
    }

    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }

    pub fn encounter_id(&self) -> i32 {
        self.encounter_id
    }

    pub fn difficulty(&self) -> u8 {
        self.difficulty
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn member(&self, guid: u64) -> Option<Member> {
        self.members.get(&guid).copied()
    }

    pub fn is_member(&self, guid: u64) -> bool {
        self.members.contains_key(&guid)
    }

    pub fn roster(&self) -> Vec<u64> {
        self.members.keys().copied().collect()
    }

    pub fn connected(&self) -> usize {
        self.members
            .values()
            .filter(|member| member.is_connected())
            .count()
    }

    pub fn generation(&self) -> u32 {
        self.next_generation
    }

    /// Joins a member before the GO packet. Eligible members are the recovered difficulty gate,
    /// then the caller-supplied capacity.
    pub fn enter(&mut self, guid: u64, combat_level: u32) -> Result<Vec<Vec<u8>>, Reject> {
        match self.phase {
            Phase::Closed => return Err(Reject::InstanceClosed),
            Phase::Loading => {}
            _ => return Err(Reject::RosterClosed),
        }
        if self.members.contains_key(&guid) {
            return Err(Reject::AlreadyMember);
        }
        let required = required_combat_level(self.difficulty)
            .ok_or(Reject::UnknownDifficulty(self.difficulty))?;
        if combat_level < required {
            return Err(Reject::NotEligible {
                required,
                actual: combat_level,
            });
        }
        if self.members.len() >= self.config.capacity {
            return Err(Reject::InstanceFull);
        }
        self.members.insert(guid, Member::Joined);
        Ok(vec![encode_player_enter(
            self.encounter_id,
            self.instance_id,
            guid,
        )])
    }

    /// Marks a member's zone load finished. The GO packet (`41/107`) is emitted once, when every
    /// connected member is ready. Graced members do not hold the gate.
    pub fn zone_ready(&mut self, guid: u64) -> Result<Vec<Vec<u8>>, Reject> {
        match self.phase {
            Phase::Closed => return Err(Reject::InstanceClosed),
            Phase::Completed => return Err(Reject::AlreadyCompleted),
            _ => {}
        }
        match self.members.get(&guid) {
            None => return Err(Reject::NotAMember),
            Some(Member::Disconnected { .. }) => return Err(Reject::NotConnected),
            Some(_) => {}
        }
        self.members.insert(guid, Member::Ready);
        if self.phase != Phase::Loading {
            return Ok(Vec::new());
        }
        let ready = self
            .members
            .values()
            .all(|member| !member.is_connected() || matches!(member, Member::Ready));
        if ready && self.connected() > 0 {
            self.phase = Phase::Ready;
            return Ok(vec![encode_zone_ready(self.encounter_id, self.instance_id)]);
        }
        Ok(Vec::new())
    }

    /// Starts the run: the recovered `41/106` walk from GO (`5`) to running (`6`).
    pub fn start(&mut self, guid: u64) -> Result<Vec<Vec<u8>>, Reject> {
        match self.members.get(&guid) {
            None => return Err(Reject::NotAMember),
            Some(Member::Disconnected { .. }) => return Err(Reject::NotConnected),
            Some(_) => {}
        }
        match self.phase {
            Phase::Ready => {}
            Phase::Closed => return Err(Reject::InstanceClosed),
            Phase::Completed => return Err(Reject::AlreadyCompleted),
            Phase::Active => return Err(Reject::AlreadyStarted),
            Phase::Loading => return Err(Reject::NotReady),
        }
        self.phase = Phase::Active;
        Ok(vec![
            encode_encounter_state(self.encounter_id, self.instance_id, STATE_GO),
            encode_encounter_state(self.encounter_id, self.instance_id, STATE_RUNNING),
        ])
    }

    /// Registers the actors spawned for the current generation. The owning engine writes the spawn
    /// packets; this guard prevents a double spawn and a reused actor id.
    pub fn spawn(&mut self, actors: &[u64]) -> Result<Spawn, Reject> {
        self.require_active()?;
        if self.spawn.is_some() {
            return Err(Reject::AlreadySpawned);
        }
        if self.connected() == 0 {
            return Err(Reject::NoConnectedMember);
        }
        if actors.is_empty() {
            return Err(Reject::EmptySpawn);
        }
        let mut set = BTreeSet::new();
        for actor in actors {
            if self.spent.contains(actor) {
                return Err(Reject::ActorIdReused(*actor));
            }
            if !set.insert(*actor) {
                return Err(Reject::DuplicateActor(*actor));
            }
        }
        let generation = self
            .next_generation
            .checked_add(1)
            .ok_or(Reject::Exhausted)?;
        self.next_generation = generation;
        let state = SpawnState {
            generation,
            actors: set,
            defeated: BTreeSet::new(),
        };
        let summary = state.summary();
        self.spawn = Some(state);
        Ok(summary)
    }

    pub fn defeat(&mut self, actor: u64) -> Result<Spawn, Reject> {
        self.require_active()?;
        let state = self.spawn.as_mut().ok_or(Reject::NotActive)?;
        if state.defeated.contains(&actor) {
            return Err(Reject::AlreadyDefeated(actor));
        }
        if !state.actors.contains(&actor) {
            return Err(Reject::UnknownActor(actor));
        }
        state.defeated.insert(actor);
        Ok(state.summary())
    }

    /// Restarts the run inside the same instance. The owning engine decides when (wipe or a repeat
    /// run); this clears the current spawn, retires its actor ids and resumes at running.
    pub fn reset(&mut self, guid: u64) -> Result<Vec<Vec<u8>>, Reject> {
        self.require_active()?;
        self.require_connected(guid)?;
        let state = self.spawn.take().ok_or(Reject::NothingToReset)?;
        self.spent.extend(state.actors);
        Ok(vec![encode_encounter_state(
            self.encounter_id,
            self.instance_id,
            STATE_RUNNING,
        )])
    }

    /// Settles the instance once every spawned actor of the current generation is defeated. Emits
    /// the recovered `39/18` + `39/19` pair and returns ids only - never an amount.
    pub fn complete(&mut self, guid: u64) -> Result<Completion, Reject> {
        match self.phase {
            Phase::Completed => return Err(Reject::AlreadyCompleted),
            Phase::Closed => return Err(Reject::InstanceClosed),
            Phase::Active => {}
            _ => return Err(Reject::NotActive),
        }
        if !self.members.contains_key(&guid) {
            return Err(Reject::NotAMember);
        }
        if self.connected() == 0 {
            return Err(Reject::NoConnectedMember);
        }
        if !self
            .members
            .get(&guid)
            .is_some_and(|member| member.is_connected())
        {
            return Err(Reject::NotConnected);
        }
        let state = self.spawn.as_ref().ok_or(Reject::NotActive)?;
        let alive = state.actors.len() - state.defeated.len();
        if alive > 0 {
            return Err(Reject::EnemiesAlive(alive));
        }
        self.phase = Phase::Completed;
        Ok(Completion {
            instance_id: self.instance_id,
            encounter_id: self.encounter_id,
            roster: self.roster(),
            generation: self.next_generation,
            packets: vec![
                encode_game_over(true, self.encounter_id, CURRENT_STATE, self.encounter_id),
                encode_state_remove(self.encounter_id, CURRENT_STATE, self.encounter_id),
            ],
        })
    }

    /// Leaves the instance. The last member closes it and receives the recovered `39/19` removal
    /// (a completed instance already sent that packet).
    pub fn exit(&mut self, guid: u64) -> Result<Vec<Vec<u8>>, Reject> {
        if self.members.remove(&guid).is_none() {
            return Err(Reject::NotAMember);
        }
        if !self.members.is_empty() {
            return Ok(Vec::new());
        }
        let completed = self.phase == Phase::Completed;
        self.phase = Phase::Closed;
        if completed {
            Ok(Vec::new())
        } else {
            Ok(vec![encode_state_remove(
                self.encounter_id,
                CURRENT_STATE,
                self.encounter_id,
            )])
        }
    }

    /// A dropped connection keeps the slot until the caller-supplied grace window expires.
    pub fn disconnect(&mut self, guid: u64, now_ms: u64) -> Result<(), Reject> {
        if self.phase == Phase::Closed {
            return Err(Reject::InstanceClosed);
        }
        match self.members.get(&guid) {
            None => Err(Reject::NotAMember),
            Some(Member::Disconnected { .. }) => Err(Reject::NotConnected),
            Some(_) => {
                self.members.insert(
                    guid,
                    Member::Disconnected {
                        deadline_ms: now_ms.saturating_add(self.config.grace_ms),
                    },
                );
                Ok(())
            }
        }
    }

    /// Reattaches a graced member. The spawn generation and the defeat set survive, so a reconnect
    /// never respawns the run. A reconnecting client still has to finish loading again.
    pub fn reconnect(&mut self, guid: u64, now_ms: u64) -> Result<(), Reject> {
        match self.members.get(&guid).and_then(|member| member.deadline()) {
            None => Err(match self.members.get(&guid) {
                None => Reject::NotAMember,
                Some(_) => Reject::NotDisconnected,
            }),
            Some(deadline) => {
                if now_ms >= deadline {
                    return Err(Reject::MembershipExpired);
                }
                self.members.insert(guid, Member::Ready);
                Ok(())
            }
        }
    }

    /// Removes members whose grace window closed. Expiring the last member closes the instance.
    pub fn reap(&mut self, now_ms: u64) -> Result<Vec<Vec<u8>>, Reject> {
        let expired: Vec<u64> = self
            .members
            .iter()
            .filter(|(_, member)| member.deadline().is_some_and(|deadline| now_ms >= deadline))
            .map(|(guid, _)| *guid)
            .collect();
        if expired.is_empty() {
            return Err(Reject::NothingExpired);
        }
        for guid in expired {
            self.members.remove(&guid);
        }
        if !self.members.is_empty() {
            return Ok(Vec::new());
        }
        let completed = self.phase == Phase::Completed;
        self.phase = Phase::Closed;
        if completed {
            Ok(Vec::new())
        } else {
            Ok(vec![encode_state_remove(
                self.encounter_id,
                CURRENT_STATE,
                self.encounter_id,
            )])
        }
    }

    fn require_active(&self) -> Result<(), Reject> {
        match self.phase {
            Phase::Active => Ok(()),
            Phase::Closed => Err(Reject::InstanceClosed),
            Phase::Completed => Err(Reject::AlreadyCompleted),
            _ => Err(Reject::NotActive),
        }
    }

    fn require_connected(&self, guid: u64) -> Result<(), Reject> {
        match self.members.get(&guid) {
            None => Err(Reject::NotAMember),
            Some(member) if !member.is_connected() => Err(Reject::NotConnected),
            Some(_) => Ok(()),
        }
    }
}

/// RECONSTRUCTED instance manager: monotonically assigned ids and at most one live instance per
/// player. Closed instances stay addressable for inspection; the lead's manager may prune them.
#[derive(Clone, Debug)]
pub struct Registry {
    next_instance_id: i32,
    instances: BTreeMap<i32, Lifecycle>,
    membership: BTreeMap<u64, i32>,
}

impl Registry {
    pub fn new(first_instance_id: i32) -> Self {
        Registry {
            next_instance_id: first_instance_id,
            instances: BTreeMap::new(),
            membership: BTreeMap::new(),
        }
    }

    pub fn open(
        &mut self,
        encounter_id: i32,
        difficulty: u8,
        config: Config,
    ) -> Result<i32, Reject> {
        if self.next_instance_id == i32::MAX {
            return Err(Reject::Exhausted);
        }
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;
        self.instances.insert(
            instance_id,
            Lifecycle::new(instance_id, encounter_id, difficulty, config),
        );
        Ok(instance_id)
    }

    pub fn get(&self, instance_id: i32) -> Option<&Lifecycle> {
        self.instances.get(&instance_id)
    }

    pub fn get_mut(&mut self, instance_id: i32) -> Result<&mut Lifecycle, Reject> {
        self.instances
            .get_mut(&instance_id)
            .ok_or(Reject::UnknownInstance)
    }

    pub fn instance_of(&self, guid: u64) -> Option<i32> {
        self.membership.get(&guid).copied()
    }

    pub fn live_instances(&self) -> usize {
        self.instances
            .values()
            .filter(|life| life.phase() != Phase::Closed)
            .count()
    }

    /// Joins a player. A player already inside a live instance is rejected with that instance id.
    pub fn enter(
        &mut self,
        instance_id: i32,
        guid: u64,
        combat_level: u32,
    ) -> Result<Vec<Vec<u8>>, Reject> {
        if let Some(other) = self.membership.get(&guid).copied() {
            if other != instance_id {
                return Err(Reject::OtherInstance(other));
            }
        }
        let packets = self.get_mut(instance_id)?.enter(guid, combat_level)?;
        self.membership.insert(guid, instance_id);
        Ok(packets)
    }

    pub fn exit(&mut self, instance_id: i32, guid: u64) -> Result<Vec<Vec<u8>>, Reject> {
        let packets = self.get_mut(instance_id)?.exit(guid)?;
        self.membership.remove(&guid);
        Ok(packets)
    }

    pub fn reap(&mut self, instance_id: i32, now_ms: u64) -> Result<Vec<Vec<u8>>, Reject> {
        let packets = self.get_mut(instance_id)?.reap(now_ms)?;
        let stale: Vec<u64> = self
            .membership
            .iter()
            .filter(|(_, owner)| **owner == instance_id)
            .map(|(guid, _)| *guid)
            .filter(|guid| !self.instances[&instance_id].is_member(*guid))
            .collect();
        for guid in stale {
            self.membership.remove(&guid);
        }
        Ok(packets)
    }
}
