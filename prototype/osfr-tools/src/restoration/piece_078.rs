//! Zone transitions (work item 078): unload/load/spawn ordering, the persisted world location
//! and stale prior-zone refusal.
//!
//! Evidence labels used throughout:
//!
//! ORIGINAL (preserved community capture `p2.pcap`, sha256
//! `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`; the same session is also
//! preserved as `p12.pcap` and `packets_2.pcapng`, which repeat all four transitions):
//! * inner opcodes: 31 `ClientBeginZoning` (server), 41 `PreloadCharacters`, 13 `ClientIsReady`,
//!   14 `ZoneDoneSendingInitialData`, 10 `ClientFinishedLoading`, 125 `PlayerUpdatePosition`;
//! * every transition follows 31 -> 13 -> 14 -> 10 and the first client position report comes
//!   after the 10. No client 125 report exists between the 31 and the 10 in any of the twelve
//!   recovered transitions over the three capture files;
//! * the BeginZoning `Id` is not a unique zone key: the recovered encounter clearing
//!   (`sg_random_encounter_clearing`) and the world zone (`FabledRealms`) both carry `Id` 0, so
//!   the caller names the world zone rather than inferring it from this word;
//! * returning to the world zone spawns at the exact f32 world position the client last reported
//!   before leaving it (p2: `[-218.84437561035156, -16.988563537597656, 204.8990936279297]`
//!   reported at frame 15682 and reused at frame 21274; `[-220.20208740234375, -18.28126335144043,
//!   230.61207580566406]` reported at frame 22063 and reused at frame 82556).
//!
//! RECOVERED-COMMUNITY (pinned `lab/references/Sanctuary`, not the supplied binaries):
//! `Player.TeleportToZone` stores `StartingZonePosition`/`StartingZoneRotation` when leaving a
//! `WorldZone`, `ZoneManager.TryMovePlayerToZone` removes from the old zone and adds to the new
//! one under one lock with rollback, `EvictIfEmpty` unloads an emptied non-starting instance,
//! `Player.OnZoneChanged` marks the player invisible, and
//! `GatewayConnection.SavePlayerToDatabase` persists the current position only while in the
//! starting zone and otherwise the stored world origin. `PacketClientFinishedLoadingHandler` is
//! what makes the player visible again (the spawn).
//!
//! FIRST-PARTY native observation: a client can queue a position report for its prior scene after
//! the server has applied a new location; the preserved developer-teleport run persisted the
//! later arrival report, not the stale origin
//! (`notes/Reviews/Native Developer Teleport 005748.md`), and `gateway.rs` already notes that
//! "movement from the prior scene can already be queued when a begin-zoning reply is emitted".
//!
//! RECONSTRUCTED (this module only): the phase names, the refusal decisions and the fact that a
//! stale report is *ignored* rather than treated as a fatal protocol error. This module writes no
//! packet bytes and grants no reward, item or experience; the BeginZoning serializer itself is
//! owned by the gateway/encounter and piece 022.

pub const CLIENT_BEGIN_ZONING: u16 = 31;
pub const CLIENT_PRELOAD_CHARACTERS: u16 = 41;
pub const CLIENT_IS_READY: u16 = 13;
pub const ZONE_DONE_SENDING_INITIAL_DATA: u16 = 14;
pub const CLIENT_FINISHED_LOADING: u16 = 10;
pub const PLAYER_UPDATE_POSITION: u16 = 125;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionEvent {
    BeginZoning,
    ClientReady,
    InitialDataSent,
    FinishedLoading,
}

/// ORIGINAL: the inner opcode each transition stage was observed with.
pub fn opcode(event: TransitionEvent) -> u16 {
    match event {
        TransitionEvent::BeginZoning => CLIENT_BEGIN_ZONING,
        TransitionEvent::ClientReady => CLIENT_IS_READY,
        TransitionEvent::InitialDataSent => ZONE_DONE_SENDING_INITIAL_DATA,
        TransitionEvent::FinishedLoading => CLIENT_FINISHED_LOADING,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub zone_id: i32,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    InWorld,
    Loading,
    Initializing,
    Spawning,
}

/// What the caller (gateway) must do with the zone it is leaving and the target spawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneChange {
    pub unload_zone_id: i32,
    pub spawn: Location,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionError {
    SameZone,
    NotInWorld,
    NotLoading,
    NotInitializing,
    NotSpawning,
    NonFinite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionReport {
    Applied,
    IgnoredWhileLoading,
    Invalid,
}

#[derive(Clone, Copy, Debug)]
pub struct ZoneTransition {
    world_zone_id: i32,
    phase: Phase,
    location: Location,
    world_origin: Location,
    pending: Option<Location>,
}

fn finite(value: &Location) -> bool {
    value
        .position
        .iter()
        .chain(value.rotation.iter())
        .all(|v| v.is_finite())
}

impl ZoneTransition {
    /// Starts in a loaded zone at the given spawn.
    pub fn new(world_zone_id: i32, spawn: Location) -> Result<Self, TransitionError> {
        if !finite(&spawn) {
            return Err(TransitionError::NonFinite);
        }
        Ok(Self {
            world_zone_id,
            phase: Phase::InWorld,
            location: spawn,
            world_origin: spawn,
            pending: None,
        })
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn world_zone_id(&self) -> i32 {
        self.world_zone_id
    }

    /// The location that survives a restart: the current position in the world zone, otherwise
    /// the stored world origin.
    pub fn persisted_location(&self) -> Location {
        if self.location.zone_id == self.world_zone_id {
            self.location
        } else {
            self.world_origin
        }
    }

    /// Begins a transition: unload the current zone, load the destination and stage its spawn.
    ///
    /// Leaving the world zone stores the world origin (community `Player.TeleportToZone`); the
    /// captured return spawns re-use that stored position bit for bit.
    pub fn begin(&mut self, destination: Location) -> Result<ZoneChange, TransitionError> {
        if self.phase != Phase::InWorld {
            return Err(TransitionError::NotInWorld);
        }
        if !finite(&destination) {
            return Err(TransitionError::NonFinite);
        }
        if destination.zone_id == self.location.zone_id {
            return Err(TransitionError::SameZone);
        }
        if self.location.zone_id == self.world_zone_id {
            self.world_origin = self.location;
        }
        let change = ZoneChange {
            unload_zone_id: self.location.zone_id,
            spawn: destination,
        };
        // The player moves into the destination zone immediately (community
        // `Player.OnZoneChanged` -> `UpdatePosition`), but stays invisible until the finished
        // loading notice releases the spawn.
        self.location = destination;
        self.pending = Some(destination);
        self.phase = Phase::Loading;
        Ok(change)
    }

    /// Client 13: the zone initialization may start.
    pub fn on_client_ready(&mut self) -> Result<(), TransitionError> {
        if self.phase != Phase::Loading {
            return Err(TransitionError::NotLoading);
        }
        self.phase = Phase::Initializing;
        Ok(())
    }

    /// The server finished sending the zone initialization (inner opcode 14).
    pub fn on_initial_data_sent(&mut self) -> Result<(), TransitionError> {
        if self.phase != Phase::Initializing {
            return Err(TransitionError::NotInitializing);
        }
        self.phase = Phase::Spawning;
        Ok(())
    }

    /// The client's finished-loading notice: the pending spawn becomes the live location.
    pub fn on_finished_loading(&mut self) -> Result<Location, TransitionError> {
        if self.phase != Phase::Spawning {
            return Err(TransitionError::NotSpawning);
        }
        let spawn = self.pending.take().ok_or(TransitionError::NotSpawning)?;
        self.location = spawn;
        self.phase = Phase::InWorld;
        Ok(spawn)
    }

    /// Applies a client position report to the live location.
    ///
    /// While a transition is in flight every report is a queued prior-scene sample, so it is
    /// ignored and the persisted location is left untouched (the first-party observation above).
    pub fn report_position(&mut self, position: [f32; 3], rotation: [f32; 3]) -> PositionReport {
        if self.phase != Phase::InWorld {
            return PositionReport::IgnoredWhileLoading;
        }
        if !position
            .iter()
            .chain(rotation.iter())
            .all(|v| v.is_finite())
        {
            return PositionReport::Invalid;
        }
        self.location.position = position;
        self.location.rotation = rotation;
        PositionReport::Applied
    }

    /// Whether an action bound to `zone_id` may run against the live zone.
    pub fn accepts_zone_action(&self, zone_id: i32) -> bool {
        self.phase == Phase::InWorld && zone_id == self.location.zone_id
    }
}
