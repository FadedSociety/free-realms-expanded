//! Kart race service: server-side lifecycle, control, gate validation and finish.
//!
//! ORIGINAL facts implemented here (nothing else is named or invented):
//!
//! * subtype 7 `VehicleRacePacketGateHit` and subtype 17 `VehicleRacePacketPickupHitRequest` are
//!   `u64 guid | 3 x f32 position | f32 1.0` (24 bytes), send-only client claims
//!   (`notes/Research/Kart Vehicle Configuration and Input.md`); subtype 6
//!   `VehicleRacePacketUpdateVehicleRaceServerData` is an 85-byte send-only bulk block whose interior
//!   is recovered in `Kart Motion Layout` and carries no guid; ownership is session-bound.
//! * subtype 10 `VehicleRacePacketVehicleRaceCountdownStatus`: dispatcher case `0xa` parses it
//!   through `FUN_00ba1850` -> `FUN_00ba04f0`, which reads `u64 | u32`; the dispatcher stores the
//!   4-byte value at `processor+0x1b4` and raises a pending flag at `+0x1c2` (the guid is parsed and
//!   unused). `FUN_00ba6300` consumes the flag, calls `Countdown:SetCount(value)`, clears it and
//!   sends the same value back with the local racer guid - the captured client echo.
//! * subtype 1 game state (`processor+0x194`), subtype 3 player count, subtype 4 npc count and
//!   subtype 14 progress (`laps`, `lap limit`, `best lap`, `total time`) are recovered; the results
//!   screen is client-derived from `lap limit <= laps`, so the server finish is the final progress
//!   block and there is no finish packet (`notes/Research/Kart Race Finish Protocol.md`).
//! * the capture cadence: states `3,4,5,6,7,8,9` and countdown `4,3,2,1,0`; 31 gate hits from one
//!   local guid over 51.6 s with a peak step speed of 40.5 m/s
//!   (`lab/reports/kart-local-gate-binding-20261009T162829Z/proof.json`). The entry handler owns
//!   `33/9`+`33/2` and the state-3/players-1 bundle, so this service's schedule starts at state 4.
//!
//! AUTHORITY. A client gate hit is a claim, never evidence: it must name the owned player guid, the
//! position must fall inside the expected gate volume, the server clock must have advanced and the
//! implied movement must stay inside the bounded budget (see `claim_credit_m`). Attribution prefers
//! the expected gate, so physical volumes may overlap without letting a skipped or stale claim
//! advance. Refused claims change nothing, so an arbitrary stream of gate hits - including a
//! whole-lap replay at teleport speed - cannot finish a race or produce a reward. Laps and lap times
//! are delegated to the registered `crate::kart_racing::RaceSession`; this module only decides what
//! counts.
//!
//! REWARD. The service prepares, it never pays: after a validated finish it exposes the finished
//! `RaceSession` **once** through [`KartService::take_finished_session`] so the lead can run the
//! reward registry (job/profile gate, priced track, real inventory). There is no grant, price or
//! inventory code here.
//!
//! ITEM USE. Subtype 19 is the captured `VehicleRacePacketUseItemRequest` (ctor `FUN_00b9c620` ->
//! `FUN_00b992d0(0x13)`, `pseudocode.c:1576140`). The sender in `FUN_00ba48a0` fires it only while
//! the local vehicle carries at least one queued item (`vehicle+0x270 > 0`), the use key is latched
//! (`vehicle+0x67`) and no use is already in flight (`vehicle+0x68 == 0`), then sets `+0x68`; its
//! 8-byte payload is the same two u32 id halves as `33/22` (`racer+0xe60`, `racer+0xe64`), which is
//! exactly the owned racer guid read little-endian. Subtype 20 is `VehicleRacePacketUseItemResponse`
//! (ctor `FUN_00b99a90` -> `FUN_00b992d0(0x14)`, `pseudocode.c:1574386`); its reader
//! (`FUN_00ba1540` -> `FUN_00ba0a70`) consumes exactly one u32 item type, and dispatcher
//! `FUN_00ba74d0` case `0x14` applies it: types 1..4 are item callbacks, **type 5 is the boost**
//! (`vehicle+0x118 = DAT_017ec67c`, `vehicle+0x64 = 1`). The same case then pops one queued item
//! (`FUN_00fe94c0`), so the server mirror of the client's queue (`FUN_00fe9410`, capped at three)
//! pops exactly one entry per accepted use. That bounds accepted uses by the awarded entries: the
//! frame carries no distinct request id (only the owned racer id pair), so duplicate frames are
//! refused only once the queue runs out.
//!
//! REFILL PACING. The `33/22` refill is paced by the client's own accumulator, never by a fixed
//! server interval: `FUN_00feaa30(vehicle, dt)` adds `config+0x740 * dt` to `vehicle+0x114` while it
//! is below `DAT_017ec67c` (clamped there) and drains it with `config+0x744 * dt` while the boost is
//! active, `FUN_00c5f7c0` seeds a freshly built kart with a full accumulator, and the sender in
//! `FUN_00ba48a0` fires the request when the accumulator is full and the queue holds fewer than
//! three entries, then resets it to zero. Captured legitimate gaps are therefore config-shaped -
//! 10.603 s (v13), ~33 s (v11), minutes (v12) after Go - so the service accepts every owned,
//! in-phase request inside the race window while the mirrored three-slot queue has room and refuses
//! `ItemQueueFull` once it is full. The frame carries no distinct request or transaction id (only
//! the owned racer id pair), so a replayed frame is
//! indistinguishable from a further refill and is admitted only while an entry is free: distinct
//! repeat application requests may fill the remaining slots, while transport retransmissions are
//! deduplicated below this service (reliable sequence plus CRC). The config energy terms that pace
//! the client (`config+0x740/0x744/0x748/0x74c`) are not server-authoritative yet, so this mirror
//! never re-derives them. A frame whose clock predates Go is refused as a regression on every arm.
//!
//! PICKUPS. Subtype `15` create, `16` destroy, `17` client claim and `18` grant are recovered and
//! implemented through [`crate::kart_pickups`]: content supplies `pickups` (default empty, so legacy
//! content emits exactly the frames it did before), the service spawns every live definition once at
//! Go, and a valid `33/17` claim stages the nearest live box inside its radius. The claim keeps its
//! own observation budget and shares the gate movement credit (`check_movement`); an accepted pickup
//! pays that credit, pushes one recovered-typed item (1 missile, 3 mine, 4 shield, 5 boost) into the
//! mirror queue and answers `33/18` then `33/16`. It never touches the racing session, so gates,
//! laps and times cannot move from a pickup; the client owns each type's own effect and no weapon
//! effect is implemented here. Respawn is opt-in content (`pickup_respawn`, default `None` = the
//! legacy permanent consumption): a collection schedules the box's return after the bounded delay,
//! `tick` announces it once the racing clock passes the due time - never before Go, after the race
//! window, after finish/abort or across a clock regression - and a refused claim schedules nothing.
//! A restored snapshot that never spawned refuses collection, and the policy plus the fired/pending
//! respawn state are content-bound, so a tampered save cannot move a box.
//!
//! SHIELD LIFETIME. The `33/20` type-4 use turns the client shield on, and the recovered `33/21`
//! `VehicleRacePacketItemExpired` with value 4 turns the same effect off (raw PE comparison at
//! 0xba88c1/0xba88c6; value 1 is ignored). This service paces that expiry: a type-4 use starts a
//! reconstructed [`SHIELD_DURATION_MS`] deadline clamped to the race window, a repeated use
//! refreshes it, `tick` emits exactly one `33/21(4)` once the deadline passes inside a live racing
//! window, and finish/abort clear the deadline **without** emitting `33/21` (the native UI teardown
//! removes the effect; the lead's probe checks in-race expiry only). The deadline is persisted
//! and restore refuses a value outside `race_started_ms..=race_end` or a non-racing phase. No other
//! item type gets this shield lifetime. Opt-in weapon simulation lives in `kart_weapons` and
//! uses validated session-cached C33/6 poses; hit/knockback acceptance remains separate.
//!
//! GATE GEOMETRY. The authored gate leaves are recovered from the shipped CDTA
//! (`sg_kt_gates_bbec_004.cdt`: 144 vertices, 72 triangles, 17 vertical planes), and the lead's
//! content supplies them as `gate_curtains` with a reconstructed thickness. The waypoint positions,
//! the oriented band and the legacy radius remain reconstructed content. Still unrecovered: the
//! `33/6` interior, the `33/14` unmapped fields, the exact finish frame order, the `33/15` pickup
//! spawn layout and any other-racer simulation. Progress is emitted per accepted gate instead of at
//! the captured ~0.3 s cadence.
//!
//! GATE VOLUME. The recovered trigger records (`creationAppPhysics::Trigger FUN_00fe3310`) place
//! type-3 physics volumes in the track scene; `FUN_00feaa30` raises the entry latch on entering one
//! (`0x1dc`) and the sender reports the *vehicle* position at that boundary, not the volume centre.
//! The authored extents are still unrecovered, so [`GateBand`] is an opt-in reconstructed oriented
//! volume whose travel axis comes from the ordered waypoints or one supplied horizontal normal.
//! Without it the legacy spherical [`GateWaypoint::radius`] volume is used unchanged.
//!
//! The captured samples are a player's trajectory, not the track: the real gate volumes and the
//! finish line are the lead's content reconstruction, and the `33/6`/`33/17` observation streams
//! have their own high-rate budget so they can never consume the gate claim budget.
//!
//! ponytail: one flat service, one schedule cursor, one gate cursor derived from the racing session.

use crate::kart_entry::{self, EntryGate};
use crate::kart_pickups::{
    PendingRespawn, PickupError, PickupRefusal, PickupRespawn, PickupSet, PickupSnapshot,
    PickupSpec, SHIELD_EXPIRE_VALUE, SHIELD_ITEM_TYPE, encode_expire, is_recovered_item_type,
};
use crate::kart_progress_native::{self, ProgressPayload};
use crate::kart_racing::{
    CheckpointOutcome, KartRacingError, Phase, RaceSession, RewardSchedule, SessionSnapshot,
    TrackLayout,
};
use crate::kart_weapons::{WeaponKind, WeaponObservation, WeaponPose, WeaponState};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::kart_progress_native::PROGRESS_SUBTYPE;

/// ORIGINAL: client bulk server-data subtype (send-only, 85-byte payload).
pub const SERVER_DATA_SUBTYPE: u16 = 6;
/// ORIGINAL: client gate-hit subtype.
pub const GATE_HIT_SUBTYPE: u16 = 7;
/// ORIGINAL: countdown status subtype (server broadcast and client echo).
pub const COUNTDOWN_SUBTYPE: u16 = 10;
/// ORIGINAL: owned-guid BoostAwardRequest and header-only BoostAwardResponse.
pub const BOOST_REQUEST_SUBTYPE: u16 = 22;
pub const BOOST_RESPONSE_SUBTYPE: u16 = 23;
/// ORIGINAL: owned-guid `VehicleRacePacketUseItemRequest` (client use) and the one-u32
/// `VehicleRacePacketUseItemResponse` that applies the queued item on the client.
pub const USE_ITEM_SUBTYPE: u16 = 19;
pub const USE_ITEM_RESPONSE_SUBTYPE: u16 = 20;
/// ORIGINAL: `33/19` and `33/22` both carry the owned racer id pair (`racer+0xe60`, `racer+0xe64`)
/// as one 8-byte little-endian u64.
pub const OWNED_ID_PAYLOAD_LEN: usize = 8;
/// ORIGINAL: the client's item queue (`FUN_00fe9410`) refuses a fourth entry, so the server mirror
/// is capped at the same three slots.
pub const ITEM_QUEUE_CAP: usize = 3;
/// ORIGINAL: item type 5 is the only type whose recovered `33/20` effect is the boost
/// (`vehicle+0x118 = DAT_017ec67c`, `vehicle+0x64 = 1`, dispatcher case `0x14`).
pub const BOOST_ITEM_TYPE: u32 = 5;
/// RECONSTRUCTED shield lifetime: a `33/20` type-4 use turns the client shield on, and the
/// recovered `33/21` (`VehicleRacePacketItemExpired`) with value 4 turns that same effect off (raw
/// PE comparison at 0xba88c1/0xba88c6). This service paces the expiry;
/// the 5 s duration is reconstructed content, not a retail measurement.
pub const SHIELD_DURATION_MS: u64 = 5_000;
/// LEGACY epoch width, retained only for the saved-epoch shape bound. It is **not** a pacing gate:
/// the native refill accumulator is config-driven (`FUN_00feaa30`) and seeded full by
/// `FUN_00c5f7c0`, so captured legitimate requests arrive ~10.6 s (v13), ~33 s (v11) and minutes
/// (v12) after Go. The recovered three-slot queue is the real bound.
pub const BOOST_INTERVAL_MS: u64 = 20_000;
/// ORIGINAL: client pickup-hit subtype (same layout as the gate hit). Still an observation: the
/// `33/15` spawn layout and pickup geometry are unrecovered (`33/18` response is pinned).
pub const PICKUP_HIT_SUBTYPE: u16 = 17;
/// ORIGINAL: server game-state subtype.
pub const GAME_STATE_SUBTYPE: u16 = 1;
/// ORIGINAL: server player-count subtype.
pub const PLAYER_COUNT_SUBTYPE: u16 = 3;
/// ORIGINAL: server npc-count subtype.
pub const NPC_COUNT_SUBTYPE: u16 = 4;
/// ORIGINAL: 85-byte bulk block.
pub const SERVER_DATA_PAYLOAD_LEN: usize = 85;
/// ORIGINAL: `u64 guid | 3 x f32 | f32` claim block (client writer `sub_BA48A0` at 0x00ba557d).
pub const GATE_HIT_PAYLOAD_LEN: usize = 24;
/// ORIGINAL: `u64 guid | u32 value` countdown block.
pub const COUNTDOWN_PAYLOAD_LEN: usize = 12;
/// ORIGINAL: the recovered `33/14` payload length.
pub const PROGRESS_PAYLOAD_LEN: usize = kart_progress_native::PROGRESS_PAYLOAD_LEN;
/// RECONSTRUCTED bound: gates per track.
pub const MAX_GATES: usize = 128;
/// RECONSTRUCTED bound: schedule steps per race.
pub const MAX_SCHEDULE_STEPS: usize = 64;
/// ORIGINAL: the captured countdown starts at 4 and ends at 0.
pub const MAX_COUNTDOWN_VALUE: u32 = 4;
/// RECONSTRUCTED bound: racer slots and counters stay inside the recovered 8-slot range bound.
pub const MAX_RACERS: usize = 16;
/// RECONSTRUCTED policy: two claims closer than this are a rate violation, not a speed question -
/// except when the saved movement credit pays for the leg, which is how the captured client's
/// queued pair (two datagrams 16 ms apart) is admitted.
pub const MIN_CLAIM_DELTA_MS: u64 = 50;
/// RECONSTRUCTED network tolerance: the elapsed time one claim may pay for before saved slack.
/// A claim inside [`MIN_CLAIM_DELTA_MS`] is paid for from `claim_credit_m` alone; a longer gap
/// adds at most this window of travel, so a teleport cannot buy unlimited distance by idling.
pub const CLAIM_CREDIT_WINDOW_MS: u64 = 3_000;
/// RECONSTRUCTED network tolerance: saved travel credit is capped at this many seconds of travel
/// (`CLAIM_CREDIT_CAP_SECONDS * bounds.max_speed_mps`, 180 m at the captured 60 m/s). One claim
/// may therefore pay for the [`CLAIM_CREDIT_WINDOW_MS`] window plus the bank: at most 360 m at
/// 60 m/s, and only after a legitimate leg banked the slack first. A pristine save has no bank, so
/// an idle claim still cannot jump more than the 180 m window.
pub const CLAIM_CREDIT_CAP_SECONDS: f32 = 3.0;
/// RECONSTRUCTED floor: a gate volume smaller than this cannot cover the captured sampling gap
/// (neighbouring captured hits are 9.8 m apart), so content must use at least this radius.
pub const MIN_GATE_RADIUS_M: f32 = 3.0;
/// RECONSTRUCTED bound: a band half extent beyond this would swallow a whole lap, so it is refused
/// instead of clamped.
pub const MAX_GATE_BAND_HALF_M: f32 = 100.0;
/// RECONSTRUCTED floor: ordered neighbours closer than this cannot define a travel axis.
pub const MIN_BAND_AXIS_M: f32 = 0.05;
/// RECONSTRUCTED tolerance: default half-thickness around an authored curtain leaf. The shipped
/// CDTA gives the leaf surface only (vertices/normals); the 3.0 m is chosen from the capture - the
/// 97 `33/7` claims sit 1.37..=2.57 m off a leaf - so it admits the capture without widening the
/// gate into a band. Content may override it up to [`MAX_GATE_CURTAIN_HALF_DEPTH_M`].
pub const GATE_CURTAIN_HALF_DEPTH_M: f32 = 3.0;
/// RECONSTRUCTED bound: content may not thicken a curtain leaf beyond this.
pub const MAX_GATE_CURTAIN_HALF_DEPTH_M: f32 = 6.0;
/// RECONSTRUCTED bound: leaves per gate. The recovered groups carry one or two (the aliases).
pub const MAX_GATE_CURTAIN_LEAVES: usize = 2;
/// RECONSTRUCTED floor: a triangle below this area (half cross product) is degenerate.
pub const MIN_GATE_CURTAIN_AREA_M2: f32 = 0.5;
/// ORIGINAL shape: every recovered leaf is vertical, so the plane normal has no vertical part.
pub const MAX_GATE_CURTAIN_NORMAL_Y: f32 = 0.05;
/// RECONSTRUCTED tolerance: how far the fourth vertex may sit off the first triangle's plane. The
/// shipped leaves measure 6.8e-05 m, so 0.05 m is a content-fidelity guard, not a widened volume.
pub const MAX_GATE_CURTAIN_PLANARITY_M: f32 = 0.05;
/// RECONSTRUCTED bound: one leaf may not span more than this corner-to-corner.
pub const MAX_GATE_CURTAIN_SPAN_M: f32 = 400.0;
/// ORIGINAL: the client's game-state guard is `state < 0xb`; the captured sequence is 3..9.
pub const MAX_GAME_STATE: i32 = 10;

/// RECONSTRUCTED content: one ordered gate volume. Original gate geometry is unrecovered; the lead's
/// content supplies a labelled reconstruction.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateWaypoint {
    pub position: [f32; 3],
    pub radius: f32,
}

/// RECONSTRUCTED content: one oriented gate band shared by every gate of a speedway.
///
/// The recovered trigger latches on *entering* a type-3 volume and reports the vehicle position at
/// that boundary, so a single sphere around one captured lane rejects a legitimate hit in another
/// lane. `half_width` is horizontal and perpendicular to the travel direction, `half_depth` runs
/// along it, and `half_height` is vertical (`y` up). `normal` is the travel direction (the gate
/// plane normal); when absent it is derived per gate from the ordered waypoints
/// (`prev -> next`, `prev -> current` for the last gate, `next -> current` for the first).
///
/// The authored trigger extents remain unrecovered: these dimensions are explicit reconstructed
/// content, never a retail measurement. `None` on [`KartServiceContent`] keeps the legacy sphere.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateBand {
    pub half_width: f32,
    pub half_depth: f32,
    pub half_height: f32,
    #[serde(default)]
    pub normal: Option<[f32; 3]>,
}

/// ORIGINAL geometry: one authored gate leaf, recovered from the shipped CDTA curtains
/// (`sg_kt_gates_bbec_004.cdt`: 144 vertices, 72 triangles of vertical leaves). Four corners in
/// ring order; the leaf is the two triangles `0-1-2` and `0-2-3`.
///
/// `half_depth` is the leaf's own half-thickness in metres. It is a reconstructed bound around the
/// authored surface, not a retail measurement: the captured claims sit up to 2.57 m off a leaf.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateCurtain {
    pub vertices: [[f32; 3]; 4],
    #[serde(default = "default_gate_curtain_half_depth")]
    pub half_depth: f32,
}

/// The default leaf half-thickness for content that omits it.
pub fn default_gate_curtain_half_depth() -> f32 {
    GATE_CURTAIN_HALF_DEPTH_M
}

/// One recovered server frame the schedule may emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduledFrame {
    GameState(i32),
    PlayerCount(u32),
    NpcCount(u32),
    Countdown(u32),
}

/// One schedule entry, `at_ms` measured from [`KartService::begin`]. The recovered cadence is states
/// `4,5,6,7,8,9` and countdown `4,3,2,1,0` (the entry handler owns state 3 and the player count).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceStep {
    pub at_ms: u64,
    pub frame: ScheduledFrame,
}

/// RECONSTRUCTED bounds. `max_speed_mps` accepts the captured route (peak 40.5 m/s) and refuses
/// teleports; `max_claims` counts every claim attempt and must leave room for a legitimate race.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceBounds {
    pub max_speed_mps: f32,
    pub max_step_ms: u64,
    pub max_race_ms: u64,
    pub max_claims: u32,
    /// Separate budget for the high-rate send-only observations (`33/6`, `33/17`): the capture
    /// carries thousands of bulk frames, so they must never consume the gate claim budget.
    ///
    /// Older content that predates this field loads with the default: 600 s at 200 Hz.
    #[serde(default = "default_max_observations")]
    pub max_observations: u32,
}

/// Default observation budget for content written before the field existed: 600 s at 200 Hz.
pub fn default_max_observations() -> u32 {
    120_000
}

/// Lead/private content for one speedway race. The unmapped `33/14` fields come from
/// `progress_template`; the authority fields (guid, laps, times) are written by the service.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KartServiceContent {
    pub speedway: u16,
    pub laps: u32,
    #[serde(default)]
    pub reward: Option<RewardSchedule>,
    pub gates: Vec<GateWaypoint>,
    pub schedule: Vec<RaceStep>,
    pub progress_template: ProgressPayload,
    pub start_position: [f32; 3],
    pub bounds: RaceBounds,
    /// RECONSTRUCTED opt-in oriented gate volume; see [`GateBand`]. Absent keeps the sphere.
    #[serde(default)]
    pub gate_band: Option<GateBand>,
    /// ORIGINAL authored geometry: one inner list per gate of bounded vertical curtain leaves
    /// (1..=2 each). Omitted or empty keeps the reconstructed band or the legacy sphere; a
    /// non-empty list must carry exactly one group per gate.
    #[serde(default)]
    pub gate_curtains: Vec<Vec<GateCurtain>>,
    /// Authored `33/15` pickups. Absent or empty content emits exactly the pre-pickup frames, so
    /// older content keeps working unchanged.
    #[serde(default)]
    pub pickups: Vec<PickupSpec>,
    /// RECONSTRUCTED opt-in pickup respawn. Absent keeps every collected box permanent, so legacy
    /// content and saves behave exactly as before.
    #[serde(default)]
    pub pickup_respawn: Option<PickupRespawn>,
    /// Opt-in reconstructed weapon simulation using recovered native packet layouts.
    #[serde(default)]
    pub weapon_effects: bool,
}

/// The identity of one race. `session` binds the service to the entry the lead started; the wire
/// checks here are guid based because every client claim carries the racer guid, so `player_guid` is
/// the real character guid and is never replaced by a captured value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceContext {
    pub session: u64,
    pub player_guid: u64,
    pub instance_id: u32,
    pub encounter_id: u32,
}

/// Typed refusal: nothing was sent and no state moved.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRefusal {
    GateDisabled,
    NoOpenRace,
    ForeignGuid {
        offered: u64,
        owned: u64,
    },
    InvalidPhase {
        phase: RacePhase,
    },
    UnsupportedSubtype {
        subtype: u16,
    },
    /// The claim sits in a gate the racer already passed (back-tracking or replay).
    StaleGate {
        expected: u32,
        got: u32,
    },
    /// The claim sits in a later gate while the next one is still expected.
    SkippedGate {
        expected: u32,
        got: u32,
    },
    /// The claim is inside no gate volume.
    OutsideGates,
    NonFinitePosition,
    /// The claim's distance is not covered by the bounded movement budget (the elapsed window plus
    /// banked credit). `speed_mps` is the implied speed for the log; it can sit below `limit_mps`
    /// when a long idle claim runs past the bounded budget.
    ImplausibleSpeed {
        speed_mps: f32,
        limit_mps: f32,
    },
    /// Two claims arrived closer than the rate floor (or the clock regressed).
    ClaimRate {
        delta_ms: u64,
    },
    /// The whole-race time window elapsed; the race aborts without a reward.
    RaceTimeout {
        elapsed_ms: u64,
        limit_ms: u64,
    },
    /// The per-race claim budget is exhausted.
    ClaimBudget {
        claims: u32,
    },
    /// The per-race observation budget is exhausted (bulk/pickup frames, no gate effect).
    ObservationBudget {
        observations: u32,
    },
    /// A countdown echo names a value ahead of what the server sent.
    CountdownSpoof {
        value: u32,
        sent: u32,
    },
    /// LEGACY: the old fixed refill interval refused early requests. It is no longer emitted -
    /// the client's config-driven accumulator is the pacing and the queue cap is the bound - but
    /// the variant is kept so existing callers and logs keep compiling.
    BoostNotReady,
    /// `33/19` arrived while the mirrored item queue held nothing to consume.
    NoItemQueued,
    /// A boost award would push a fourth entry past the recovered three-slot cap.
    ItemQueueFull,
    /// No pickup is inside the claim radius (already collected, too far, or no content pickups).
    NoPickupInRange,
    /// A restored race never spawned its pickups, so nothing can be collected in it.
    PickupsNotSpawned,
    /// A weapon cannot be launched without a fresh validated pose or a free actor slot.
    WeaponUnavailable,
}

impl fmt::Display for ServiceRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceRefusal::WeaponUnavailable => write!(
                f,
                "weapon requires a fresh valid pose and a free actor slot"
            ),
            ServiceRefusal::GateDisabled => write!(f, "kart service gate is disabled"),
            ServiceRefusal::NoOpenRace => write!(f, "no kart race is open"),
            ServiceRefusal::ForeignGuid { offered, owned } => {
                write!(f, "guid {offered:016x} is not the owned {owned:016x}")
            }
            ServiceRefusal::InvalidPhase { phase } => {
                write!(f, "request cannot be accepted in phase {phase:?}")
            }
            ServiceRefusal::UnsupportedSubtype { subtype } => {
                write!(f, "subtype {subtype} is not part of the race service")
            }
            ServiceRefusal::StaleGate { expected, got } => {
                write!(f, "gate {got} was already passed, expected {expected}")
            }
            ServiceRefusal::SkippedGate { expected, got } => {
                write!(f, "gate {got} claimed while {expected} is still expected")
            }
            ServiceRefusal::OutsideGates => write!(f, "claim is inside no gate volume"),
            ServiceRefusal::NonFinitePosition => write!(f, "claim position is not finite"),
            ServiceRefusal::ImplausibleSpeed {
                speed_mps,
                limit_mps,
            } => write!(
                f,
                "claim movement is not covered by the bounded time-and-credit budget \
                 (implied {speed_mps:.1} m/s, content bound {limit_mps:.1} m/s)"
            ),
            ServiceRefusal::ClaimRate { delta_ms } => {
                write!(f, "claim {delta_ms} ms after the previous accepted sample")
            }
            ServiceRefusal::RaceTimeout {
                elapsed_ms,
                limit_ms,
            } => write!(f, "race ran {elapsed_ms} ms over the {limit_ms} ms window"),
            ServiceRefusal::ClaimBudget { claims } => {
                write!(f, "claim budget of {claims} exhausted")
            }
            ServiceRefusal::ObservationBudget { observations } => {
                write!(f, "observation budget of {observations} exhausted")
            }
            ServiceRefusal::CountdownSpoof { value, sent } => {
                write!(f, "countdown echo {value} is ahead of the sent {sent}")
            }
            ServiceRefusal::BoostNotReady => write!(f, "boost refill is not due"),
            ServiceRefusal::NoItemQueued => write!(f, "no item is queued to use"),
            ServiceRefusal::ItemQueueFull => {
                write!(f, "item queue is full at {ITEM_QUEUE_CAP} entries")
            }
            ServiceRefusal::NoPickupInRange => {
                write!(f, "no live pickup is inside the claim radius")
            }
            ServiceRefusal::PickupsNotSpawned => {
                write!(f, "this race never spawned its pickups")
            }
        }
    }
}

/// Typed failure: malformed wire input, or content/context outside its bounds.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceError {
    FrameTooShort {
        len: usize,
        needed: usize,
    },
    UnsupportedOpcode {
        opcode: u16,
    },
    PayloadLength {
        subtype: u16,
        expected: usize,
        actual: usize,
    },
    NoContext,
    AlreadyOpen {
        phase: RacePhase,
    },
    ZeroSession,
    ZeroPlayerGuid,
    ZeroInstance,
    ZeroEncounter,
    GateCount {
        count: usize,
        max: usize,
    },
    GateRadius {
        index: usize,
        radius: f32,
    },
    NonFiniteGate {
        index: usize,
    },
    /// A reconstructed [`GateBand`] field is non-finite, non-positive or beyond its bound.
    GateBand {
        field: &'static str,
    },
    /// A gate has no derivable horizontal travel axis (single gate without `normal`, coincident
    /// ordered neighbours, or a purely vertical axis).
    GateBandAxis {
        index: usize,
    },
    /// `gate_curtains` is non-empty but does not carry exactly one group per gate.
    GateCurtainCount {
        groups: usize,
        gates: usize,
    },
    /// A curtain group has no leaf, or more than [`MAX_GATE_CURTAIN_LEAVES`] leaves.
    GateCurtainLeafCount {
        gate: usize,
        leaves: usize,
        max: usize,
    },
    /// A curtain leaf is non-finite, degenerate, not vertical, or unbounded.
    GateCurtainGeometry {
        gate: usize,
        leaf: usize,
    },
    NonFiniteStart,
    ScheduleCount {
        steps: usize,
        max: usize,
    },
    ScheduleOrder {
        index: usize,
    },
    ScheduleCountdown {
        index: usize,
        value: u32,
    },
    ScheduleState {
        index: usize,
        state: i32,
    },
    ScheduleCounter {
        index: usize,
        value: u32,
    },
    Bounds {
        field: &'static str,
    },
    ProgressTemplate {
        field: &'static str,
    },
    /// The registered racing session refused the layout.
    Track(KartRacingError),
    /// A frame the entry codec refused for a reason outside the frame-width cases below.
    MalformedInner {
        detail: String,
    },
    CorruptSnapshot {
        field: &'static str,
    },
    /// The content's pickup definitions are not a valid set.
    Pickup(PickupError),
}

impl From<kart_entry::EntryError> for ServiceError {
    fn from(error: kart_entry::EntryError) -> Self {
        match error {
            kart_entry::EntryError::FrameTooShort { len, needed } => {
                ServiceError::FrameTooShort { len, needed }
            }
            kart_entry::EntryError::UnsupportedOpcode { opcode } => {
                ServiceError::UnsupportedOpcode { opcode }
            }
            other => ServiceError::MalformedInner {
                detail: other.to_string(),
            },
        }
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::FrameTooShort { len, needed } => {
                write!(f, "inner frame needs {needed} bytes, {len} present")
            }
            ServiceError::UnsupportedOpcode { opcode } => {
                write!(f, "inner opcode {opcode} is not the vehicle-race family")
            }
            ServiceError::PayloadLength {
                subtype,
                expected,
                actual,
            } => write!(
                f,
                "subtype {subtype} payload must be {expected} bytes, got {actual}"
            ),
            ServiceError::NoContext => write!(f, "no kart race is open"),
            ServiceError::AlreadyOpen { phase } => {
                write!(f, "a race is already open in phase {phase:?}")
            }
            ServiceError::ZeroSession => write!(f, "session must be non-zero"),
            ServiceError::ZeroPlayerGuid => write!(f, "player guid must be non-zero"),
            ServiceError::ZeroInstance => write!(f, "instance id must be non-zero"),
            ServiceError::ZeroEncounter => write!(f, "encounter id must be non-zero"),
            ServiceError::GateCount { count, max } => {
                write!(f, "content needs 1..={max} gates, got {count}")
            }
            ServiceError::GateRadius { index, radius } => {
                write!(f, "gate {index} radius {radius} is not positive and finite")
            }
            ServiceError::NonFiniteGate { index } => {
                write!(f, "gate {index} position is not finite")
            }
            ServiceError::GateBand { field } => {
                write!(f, "gate band {field} is not positive, finite and bounded")
            }
            ServiceError::GateBandAxis { index } => write!(
                f,
                "gate {index} has no horizontal travel axis for its band: supply `normal` or \
                 non-coincident ordered neighbours"
            ),
            ServiceError::GateCurtainCount { groups, gates } => write!(
                f,
                "gate curtains need exactly one group per gate ({gates}), got {groups}"
            ),
            ServiceError::GateCurtainLeafCount { gate, leaves, max } => write!(
                f,
                "gate {gate} needs 1..={max} curtain leaves, got {leaves}"
            ),
            ServiceError::GateCurtainGeometry { gate, leaf } => write!(
                f,
                "gate {gate} curtain leaf {leaf} is non-finite, degenerate, not vertical or \
                 unbounded"
            ),
            ServiceError::NonFiniteStart => write!(f, "start position is not finite"),
            ServiceError::ScheduleCount { steps, max } => {
                write!(f, "schedule needs 0..={max} steps, got {steps}")
            }
            ServiceError::ScheduleOrder { index } => {
                write!(f, "schedule step {index} moves time backwards")
            }
            ServiceError::ScheduleCountdown { index, value } => {
                write!(
                    f,
                    "schedule step {index} countdown {value} exceeds {MAX_COUNTDOWN_VALUE}"
                )
            }
            ServiceError::ScheduleState { index, state } => {
                write!(
                    f,
                    "schedule step {index} state {state} is outside 1..={MAX_GAME_STATE}"
                )
            }
            ServiceError::ScheduleCounter { index, value } => {
                write!(
                    f,
                    "schedule step {index} counter {value} exceeds {MAX_RACERS}"
                )
            }
            ServiceError::Bounds { field } => write!(f, "race bound {field} is unusable"),
            ServiceError::ProgressTemplate { field } => {
                write!(f, "progress template field {field} is unusable")
            }
            ServiceError::Track(error) => write!(f, "track layout refused: {error:?}"),
            ServiceError::MalformedInner { detail } => {
                write!(f, "inner frame refused: {detail}")
            }
            ServiceError::CorruptSnapshot { field } => {
                write!(f, "snapshot field {field} is incoherent")
            }
            ServiceError::Pickup(error) => write!(f, "pickup content refused: {error}"),
        }
    }
}

impl std::error::Error for ServiceError {}

/// The service phase. `Finished` means the validated route completed; `Aborted` means the race
/// stopped without a result (timeout or an explicit abort) and can never pay.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RacePhase {
    #[default]
    Idle,
    Preparing,
    Countdown,
    Racing,
    Finished,
    Aborted,
}

/// One service answer: ordered inner op-33 frames for the gateway wrapper, or a typed refusal.
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceOutcome {
    Handled {
        replies: Vec<Vec<u8>>,
        reward_ready: bool,
        abort: bool,
    },
    Refused(ServiceRefusal),
}

impl ServiceOutcome {
    pub fn replies(&self) -> &[Vec<u8>] {
        match self {
            ServiceOutcome::Handled { replies, .. } => replies,
            ServiceOutcome::Refused(_) => &[],
        }
    }

    pub fn reward_ready(&self) -> bool {
        matches!(
            self,
            ServiceOutcome::Handled {
                reward_ready: true,
                ..
            }
        )
    }
}

/// Persisted service state: everything the validator needs to resume without re-granting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceSnapshot {
    pub phase: RacePhase,
    pub context: RaceContext,
    pub session: SessionSnapshot,
    /// The bounds the race was created with. A different limit set must not resume the race.
    pub bounds: RaceBounds,
    /// The reconstructed gate volume the race was created with. A different band must not resume
    /// the race; older snapshots without the field load as `None`.
    #[serde(default)]
    pub gate_band: Option<GateBand>,
    /// The authored curtain geometry the race was created with. A different curtain set must not
    /// resume the race; older snapshots without the field load as empty (legacy content).
    #[serde(default)]
    pub gate_curtains: Vec<Vec<GateCurtain>>,
    pub schedule_cursor: usize,
    /// Absolute server time of begin or Go; schedule offsets are relative to it.
    pub countdown_started_ms: u64,
    /// Captured schedule time already spent preparing before Go. Old saves use zero.
    #[serde(default)]
    pub countdown_offset_ms: u64,
    pub last_position: [f32; 3],
    pub last_claim_ms: u64,
    pub race_started_ms: u64,
    /// Every in-window `33/7` claim attempt, accepted or refused: the bounded flood guard. The
    /// phase, budget-ceiling and race-window refusals are pure and spend nothing.
    pub attempts: u32,
    /// An accepted `33/6` bulk frame, or an in-window `33/17` pickup attempt, accepted or refused:
    /// the bounded observation guard. The phase, budget-ceiling and race-window refusals spend
    /// nothing.
    pub observations: u32,
    pub countdown_sent: Option<u32>,
    /// LEGACY server-clock refill epoch. Retained so pre-repair saves keep loading and so the
    /// shape is still bounded; it no longer gates an award (the native accumulator paces the
    /// client and the three-slot queue bounds the server).
    #[serde(default)]
    pub last_boost_epoch: u64,
    /// Saved travel credit in metres. Older snapshots load as zero; restore bounds it against
    /// this content's speed bound so a hand-edited save cannot mint a teleport allowance.
    #[serde(default)]
    pub claim_credit_m: f32,
    /// Mirrored client item queue (recovered types only). One entry is popped per accepted
    /// `33/19`; saves written before this field existed restore as an empty queue, so a legacy
    /// snapshot can never conjure a grant.
    #[serde(default)]
    pub queued_items: Vec<u32>,
    /// The pickup definitions this race was created with. A different set must not resume the
    /// race; saves written before pickups existed load as empty (legacy content).
    #[serde(default)]
    pub pickups: Vec<PickupSpec>,
    /// Consumed pickup handles. Older saves load as empty, so nothing is pre-collected.
    #[serde(default)]
    pub pickups_consumed: Vec<u32>,
    /// True once the live `33/15` spawns were emitted. Older saves load as `false`, so a restored
    /// legacy race never collects.
    #[serde(default)]
    pub pickups_spawned: bool,
    /// The respawn policy the race was created with. A different policy must not resume the race;
    /// saves written before respawns existed load as `None` (permanent consumption).
    #[serde(default)]
    pub pickup_respawn: Option<PickupRespawn>,
    /// Respawns already fired this race, one entry per return. Older saves load as empty.
    #[serde(default)]
    pub pickups_respawned: Vec<u32>,
    /// Returns scheduled but not yet due. Older saves load as empty.
    #[serde(default)]
    pub pickups_pending: Vec<PendingRespawn>,
    /// Absolute server time the active shield expires, clamped to the race window. Older saves load
    /// as `None` (no shield running), and restore rejects values outside the window or a non-racing
    /// phase.
    #[serde(default)]
    pub shield_expiry: Option<u64>,
    #[serde(default)]
    pub weapon_effects: bool,
    #[serde(default)]
    pub weapons: WeaponState,
    #[serde(default)]
    pub weapon_pose: Option<WeaponObservation>,
    pub reward_taken: bool,
}

/// The bounded race service. One race at a time; the gate defaults off.
pub struct KartService {
    content: KartServiceContent,
    layout: TrackLayout,
    band: Option<BandVolume>,
    curtains: Option<CurtainVolume>,
    gate: EntryGate,
    phase: RacePhase,
    context: Option<RaceContext>,
    session: Option<RaceSession>,
    schedule_cursor: usize,
    countdown_started_ms: u64,
    countdown_offset_ms: u64,
    race_started_ms: u64,
    last_position: [f32; 3],
    last_claim_ms: u64,
    attempts: u32,
    observations: u32,
    countdown_sent: Option<u32>,
    last_boost_epoch: u64,
    /// Saved travel distance (metres) a legitimate flush may spend; see
    /// [`CLAIM_CREDIT_WINDOW_MS`] and [`CLAIM_CREDIT_CAP_SECONDS`].
    claim_credit_m: f32,
    queued_items: Vec<u32>,
    pickups: PickupSet,
    pickups_spawned: bool,
    /// Server time the reconstructed type-4 shield effect is turned off, if one is running.
    shield_expiry: Option<u64>,
    weapons: WeaponState,
    weapon_pose: Option<WeaponObservation>,
    reward_taken: bool,
}

fn finite(values: [f32; 3]) -> bool {
    values.iter().all(|value| value.is_finite())
}

/// Precomputed reconstructed band geometry: the shared extents plus one horizontal travel axis per
/// gate, resolved once at construction so a claim never re-derives it.
#[derive(Clone, Debug, PartialEq)]
struct BandVolume {
    band: GateBand,
    axes: Vec<[f32; 2]>,
}

/// Precomputed authored curtain geometry: per gate the bounded leaves a claim is measured
/// against, with the validated half-thickness of each leaf. Resolved once at construction so a
/// claim never re-derives it.
#[derive(Clone, Debug, PartialEq)]
struct CurtainVolume {
    leaves: Vec<Vec<[[f32; 3]; 4]>>,
    half_depths: Vec<Vec<f32>>,
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Squared distance from `point` to the triangle `a-b-c` (Ericson, Real-Time Collision Detection).
fn point_triangle_distance2(point: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = subtract(b, a);
    let ac = subtract(c, a);
    let ap = subtract(point, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return dot(ap, ap);
    }
    let bp = subtract(point, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return dot(bp, bp);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        let q = [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v];
        let diff = subtract(point, q);
        return dot(diff, diff);
    }
    let cp = subtract(point, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return dot(cp, cp);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        let q = [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w];
        let diff = subtract(point, q);
        return dot(diff, diff);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let q = [
            b[0] + (c[0] - b[0]) * w,
            b[1] + (c[1] - b[1]) * w,
            b[2] + (c[2] - b[2]) * w,
        ];
        let diff = subtract(point, q);
        return dot(diff, diff);
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let q = [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ];
    let diff = subtract(point, q);
    dot(diff, diff)
}

/// Distance from `point` to one curtain leaf: the nearer of its two triangles. A point on the
/// plane inside the ring measures the plane distance; outside the ring the distance grows to the
/// nearest edge or corner, so the leaf stays bounded.
fn curtain_distance(point: [f32; 3], quad: &[[f32; 3]; 4]) -> f32 {
    let first = point_triangle_distance2(point, quad[0], quad[1], quad[2]);
    let second = point_triangle_distance2(point, quad[0], quad[2], quad[3]);
    first.min(second).sqrt()
}

/// Validates one leaf ring and returns its unit plane normal. `None` when a vertex is non-finite,
/// a triangle is degenerate (true area below [`MIN_GATE_CURTAIN_AREA_M2`]), either triangle is not
/// vertical, the ring is not planar within [`MAX_GATE_CURTAIN_PLANARITY_M`], the ring is not convex
/// (a concave ring or bow tie turns the wrong way at some vertex), or the leaf is larger than
/// [`MAX_GATE_CURTAIN_SPAN_M`].
fn curtain_normal(quad: &[[f32; 3]; 4]) -> Option<[f32; 3]> {
    if !quad.iter().all(|vertex| finite(*vertex)) {
        return None;
    }
    for first in 0..4 {
        for second in first + 1..4 {
            let span = subtract(quad[first], quad[second]);
            if dot(span, span).sqrt() > MAX_GATE_CURTAIN_SPAN_M {
                return None;
            }
        }
    }
    let first = cross(subtract(quad[1], quad[0]), subtract(quad[2], quad[0]));
    let second = cross(subtract(quad[2], quad[0]), subtract(quad[3], quad[0]));
    let first_cross = dot(first, first).sqrt();
    let second_cross = dot(second, second).sqrt();
    // The documented floor is a true triangle area, i.e. half the cross magnitude.
    if first_cross / 2.0 < MIN_GATE_CURTAIN_AREA_M2 || second_cross / 2.0 < MIN_GATE_CURTAIN_AREA_M2
    {
        return None;
    }
    let normal = [
        first[0] / first_cross,
        first[1] / first_cross,
        first[2] / first_cross,
    ];
    let second_normal = [
        second[0] / second_cross,
        second[1] / second_cross,
        second[2] / second_cross,
    ];
    if !normal.iter().all(|value| value.is_finite())
        || !second_normal.iter().all(|value| value.is_finite())
    {
        return None;
    }
    // Both triangles of the leaf must be vertical, not just the first.
    if normal[1].abs() > MAX_GATE_CURTAIN_NORMAL_Y
        || second_normal[1].abs() > MAX_GATE_CURTAIN_NORMAL_Y
    {
        return None;
    }
    // The fourth vertex must lie in the first triangle's plane; a folded leaf would otherwise
    // become a bent slab whose far corner sits far outside the authored gate.
    if dot(subtract(quad[3], quad[0]), normal).abs() > MAX_GATE_CURTAIN_PLANARITY_M {
        return None;
    }
    // Convexity: every consecutive turn must rotate the same way as the plane normal. A concave
    // ring or a bow tie turns the other way at some vertex.
    for index in 0..4 {
        let edge = subtract(quad[(index + 1) % 4], quad[index]);
        let next = subtract(quad[(index + 2) % 4], quad[(index + 1) % 4]);
        if dot(cross(edge, next), normal) <= 0.0 {
            return None;
        }
    }
    Some(normal)
}

/// Validates the optional authored curtain geometry and precomputes it. `None` when the content
/// carries none (legacy content uses the band or the sphere).
fn curtain_volume(content: &KartServiceContent) -> Result<Option<CurtainVolume>, ServiceError> {
    if content.gate_curtains.is_empty() {
        return Ok(None);
    }
    if content.gate_curtains.len() != content.gates.len() {
        return Err(ServiceError::GateCurtainCount {
            groups: content.gate_curtains.len(),
            gates: content.gates.len(),
        });
    }
    let mut leaves = Vec::with_capacity(content.gate_curtains.len());
    let mut half_depths = Vec::with_capacity(content.gate_curtains.len());
    for (gate, group) in content.gate_curtains.iter().enumerate() {
        if group.is_empty() || group.len() > MAX_GATE_CURTAIN_LEAVES {
            return Err(ServiceError::GateCurtainLeafCount {
                gate,
                leaves: group.len(),
                max: MAX_GATE_CURTAIN_LEAVES,
            });
        }
        let mut gate_leaves = Vec::with_capacity(group.len());
        let mut gate_depths = Vec::with_capacity(group.len());
        for (leaf, curtain) in group.iter().enumerate() {
            if curtain_normal(&curtain.vertices).is_none()
                || !curtain.half_depth.is_finite()
                || curtain.half_depth <= 0.0
                || curtain.half_depth > MAX_GATE_CURTAIN_HALF_DEPTH_M
            {
                return Err(ServiceError::GateCurtainGeometry { gate, leaf });
            }
            gate_leaves.push(curtain.vertices);
            gate_depths.push(curtain.half_depth);
        }
        leaves.push(gate_leaves);
        half_depths.push(gate_depths);
    }
    Ok(Some(CurtainVolume {
        leaves,
        half_depths,
    }))
}

/// Horizontal (`x`, `z`) part of a direction, or `None` when it is non-finite or too short to
/// orient. `y` is the vertical axis.
fn horizontal_axis(direction: [f32; 3]) -> Option<[f32; 2]> {
    let length = (direction[0] * direction[0] + direction[2] * direction[2]).sqrt();
    (length.is_finite() && length >= MIN_BAND_AXIS_M)
        .then(|| [direction[0] / length, direction[2] / length])
}

/// Resolves one horizontal travel axis per gate: the supplied normal when present, otherwise the
/// ordered waypoints (`prev -> next`, `prev -> current` at the last gate, `next -> current` at the
/// first). A gate that cannot define an axis is refused, never defaulted.
fn band_axes(content: &KartServiceContent, band: &GateBand) -> Result<Vec<[f32; 2]>, ServiceError> {
    let supplied = match band.normal {
        Some(normal) if !finite(normal) => return Err(ServiceError::GateBand { field: "normal" }),
        Some(normal) => {
            Some(horizontal_axis(normal).ok_or(ServiceError::GateBand { field: "normal" })?)
        }
        None => None,
    };
    content
        .gates
        .iter()
        .enumerate()
        .map(|(index, gate)| {
            if let Some(axis) = supplied {
                return Ok(axis);
            }
            let current = gate.position;
            let previous = index
                .checked_sub(1)
                .map(|previous| content.gates[previous].position);
            let next = content.gates.get(index + 1).map(|next| next.position);
            let candidate = match (previous, next) {
                (Some(previous), Some(next)) => [
                    next[0] - previous[0],
                    next[1] - previous[1],
                    next[2] - previous[2],
                ],
                (Some(previous), None) => [
                    current[0] - previous[0],
                    current[1] - previous[1],
                    current[2] - previous[2],
                ],
                (None, Some(next)) => [
                    next[0] - current[0],
                    next[1] - current[1],
                    next[2] - current[2],
                ],
                (None, None) => return Err(ServiceError::GateBandAxis { index }),
            };
            horizontal_axis(candidate).ok_or(ServiceError::GateBandAxis { index })
        })
        .collect()
}

fn validate_content(
    content: &KartServiceContent,
) -> Result<(TrackLayout, Option<BandVolume>, Option<CurtainVolume>), ServiceError> {
    if !finite(content.start_position) {
        return Err(ServiceError::NonFiniteStart);
    }
    if content.gates.is_empty() || content.gates.len() > MAX_GATES {
        return Err(ServiceError::GateCount {
            count: content.gates.len(),
            max: MAX_GATES,
        });
    }
    for (index, gate) in content.gates.iter().enumerate() {
        if !finite(gate.position) {
            return Err(ServiceError::NonFiniteGate { index });
        }
        if !gate.radius.is_finite() || gate.radius < MIN_GATE_RADIUS_M {
            return Err(ServiceError::GateRadius {
                index,
                radius: gate.radius,
            });
        }
    }
    if content.schedule.len() > MAX_SCHEDULE_STEPS {
        return Err(ServiceError::ScheduleCount {
            steps: content.schedule.len(),
            max: MAX_SCHEDULE_STEPS,
        });
    }
    let mut previous = 0u64;
    for (index, step) in content.schedule.iter().enumerate() {
        if step.at_ms < previous {
            return Err(ServiceError::ScheduleOrder { index });
        }
        previous = step.at_ms;
        match step.frame {
            ScheduledFrame::GameState(state) => {
                if state < 1 || state > MAX_GAME_STATE {
                    return Err(ServiceError::ScheduleState { index, state });
                }
            }
            ScheduledFrame::Countdown(value) => {
                if value > MAX_COUNTDOWN_VALUE {
                    return Err(ServiceError::ScheduleCountdown { index, value });
                }
            }
            ScheduledFrame::PlayerCount(value) | ScheduledFrame::NpcCount(value) => {
                if value as usize > MAX_RACERS {
                    return Err(ServiceError::ScheduleCounter { index, value });
                }
            }
        }
    }
    if !content.bounds.max_speed_mps.is_finite() || content.bounds.max_speed_mps <= 0.0 {
        return Err(ServiceError::Bounds {
            field: "max_speed_mps",
        });
    }
    if content.bounds.max_step_ms == 0 {
        return Err(ServiceError::Bounds {
            field: "max_step_ms",
        });
    }
    if content.bounds.max_race_ms == 0 {
        return Err(ServiceError::Bounds {
            field: "max_race_ms",
        });
    }
    if content.bounds.max_observations == 0 {
        return Err(ServiceError::Bounds {
            field: "max_observations",
        });
    }
    let minimum = content.gates.len() as u32 * content.laps.max(1);
    if content.bounds.max_claims < minimum {
        return Err(ServiceError::Bounds {
            field: "max_claims",
        });
    }
    if content.progress_template.lap_limit != content.laps as i32 {
        return Err(ServiceError::ProgressTemplate { field: "lap_limit" });
    }
    if content.progress_template.racer_guid != 0 {
        return Err(ServiceError::ProgressTemplate {
            field: "racer_guid",
        });
    }
    if content.progress_template.slot < 0 || content.progress_template.slot as usize >= MAX_RACERS {
        return Err(ServiceError::ProgressTemplate { field: "slot" });
    }
    if content.progress_template.local_racer > 1 {
        return Err(ServiceError::ProgressTemplate {
            field: "local_racer",
        });
    }
    let band = match content.gate_band {
        Some(band) => {
            for (field, value) in [
                ("half_width", band.half_width),
                ("half_depth", band.half_depth),
                ("half_height", band.half_height),
            ] {
                if !value.is_finite() || value <= 0.0 || value > MAX_GATE_BAND_HALF_M {
                    return Err(ServiceError::GateBand { field });
                }
            }
            let axes = band_axes(content, &band)?;
            Some(BandVolume { band, axes })
        }
        None => None,
    };
    let checkpoints: Vec<u32> = (1..=content.gates.len() as u32).collect();
    let curtains = curtain_volume(content)?;
    let layout = TrackLayout::new(content.speedway, content.laps, checkpoints, content.reward)
        .map_err(ServiceError::Track)?;
    Ok((layout, band, curtains))
}

struct Claim {
    guid: u64,
    position: [f32; 3],
}

/// One claim's movement charge: what the bounded credit window allowed and how far the anchor
/// actually moved. Refusals never produce one, so a refused claim banks nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MovementCharge {
    allowance: f32,
    distance: f32,
}

/// Where a claim position fell. `Expected` is the only advancing outcome.
enum GateAttribution {
    Expected,
    Stale(u32),
    Skipped(u32),
    Outside,
}

/// The racing session is not `Debug`, so the service reports its own summary instead.
impl fmt::Debug for KartService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KartService")
            .field("phase", &self.phase)
            .field("context", &self.context)
            .field("schedule_cursor", &self.schedule_cursor)
            .field("race_started_ms", &self.race_started_ms)
            .field("countdown_started_ms", &self.countdown_started_ms)
            .field("attempts", &self.attempts)
            .field("observations", &self.observations)
            .field("countdown_sent", &self.countdown_sent)
            .field("queued_items", &self.queued_items)
            .field("pickups_spawned", &self.pickups_spawned)
            .field("pickups_consumed", &self.pickups.snapshot().consumed)
            .field("shield_expiry", &self.shield_expiry)
            .field("reward_taken", &self.reward_taken)
            .field("gate_band", &self.content.gate_band)
            .field(
                "session_laps",
                &self
                    .session
                    .as_ref()
                    .map(|session| session.laps_completed()),
            )
            .finish()
    }
}

/// The racing session is not `Clone`, so a clone rehydrates it from its own snapshot. The gate is
/// preserved (a clone of an enabled service stays enabled) and nothing is validated against content
/// here because the snapshot came from this very service.
impl Clone for KartService {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            layout: self.layout.clone(),
            band: self.band.clone(),
            curtains: self.curtains.clone(),
            gate: self.gate,
            phase: self.phase,
            context: self.context,
            session: self.session.as_ref().map(|session| {
                RaceSession::restore(session.snapshot()).expect("own session snapshot restores")
            }),
            schedule_cursor: self.schedule_cursor,
            countdown_started_ms: self.countdown_started_ms,
            countdown_offset_ms: self.countdown_offset_ms,
            race_started_ms: self.race_started_ms,
            last_position: self.last_position,
            last_claim_ms: self.last_claim_ms,
            attempts: self.attempts,
            observations: self.observations,
            countdown_sent: self.countdown_sent,
            last_boost_epoch: self.last_boost_epoch,
            claim_credit_m: self.claim_credit_m,
            queued_items: self.queued_items.clone(),
            pickups: self.pickups.clone(),
            pickups_spawned: self.pickups_spawned,
            shield_expiry: self.shield_expiry,
            weapons: self.weapons.clone(),
            weapon_pose: self.weapon_pose,
            reward_taken: self.reward_taken,
        }
    }
}

impl KartService {
    pub fn new(content: KartServiceContent) -> Result<Self, ServiceError> {
        let (layout, band, curtains) = validate_content(&content)?;
        let pickups = PickupSet::with_respawn(content.pickups.clone(), content.pickup_respawn)
            .map_err(ServiceError::Pickup)?;
        Ok(Self {
            content,
            layout,
            band,
            curtains,
            gate: EntryGate::Disabled,
            phase: RacePhase::Idle,
            context: None,
            session: None,
            schedule_cursor: 0,
            countdown_started_ms: 0,
            countdown_offset_ms: 0,
            race_started_ms: 0,
            last_position: [0.0; 3],
            last_claim_ms: 0,
            attempts: 0,
            observations: 0,
            countdown_sent: None,
            last_boost_epoch: 0,
            claim_credit_m: 0.0,
            queued_items: Vec::new(),
            pickups,
            pickups_spawned: false,
            shield_expiry: None,
            weapons: WeaponState::default(),
            weapon_pose: None,
            reward_taken: false,
        })
    }

    pub fn set_gate(&mut self, gate: EntryGate) {
        self.gate = gate;
    }

    pub fn gate(&self) -> EntryGate {
        self.gate
    }

    pub fn phase(&self) -> RacePhase {
        self.phase
    }

    pub fn session(&self) -> Option<&RaceSession> {
        self.session.as_ref()
    }

    /// The entry session this race is bound to; lets the lead assert the entry handoff identity.
    pub fn entry_session(&self) -> Option<u64> {
        self.context.map(|context| context.session)
    }

    pub fn player_guid(&self) -> Option<u64> {
        self.context.map(|context| context.player_guid)
    }

    /// Bounded observation counter for the send-only bulk/pickup blocks; never authority.
    pub fn observations(&self) -> u32 {
        self.observations
    }

    /// True when the content carries a price schedule; the reward registry still decides payment.
    pub fn reward_priced(&self) -> bool {
        self.content.reward.is_some()
    }

    /// True while a validated finish is pending and the one-shot offer was not consumed.
    pub fn finish_flag(&self) -> bool {
        self.phase == RacePhase::Finished && !self.reward_taken && self.session.is_some()
    }

    /// The one-shot hand-off to the lead's reward path. Returns the finished session once.
    pub fn take_finished_session(&mut self) -> Option<&mut RaceSession> {
        if !self.finish_flag() {
            return None;
        }
        self.reward_taken = true;
        self.session.as_mut()
    }

    /// Binds a race to the entry the lead started. One race at a time.
    pub fn begin(
        &mut self,
        context: RaceContext,
        now_ms: u64,
    ) -> Result<ServiceOutcome, ServiceError> {
        if !matches!(
            self.phase,
            RacePhase::Idle | RacePhase::Finished | RacePhase::Aborted
        ) {
            return Err(ServiceError::AlreadyOpen { phase: self.phase });
        }
        if context.session == 0 {
            return Err(ServiceError::ZeroSession);
        }
        if context.player_guid == 0 {
            return Err(ServiceError::ZeroPlayerGuid);
        }
        if context.instance_id == 0 {
            return Err(ServiceError::ZeroInstance);
        }
        if context.encounter_id == 0 {
            return Err(ServiceError::ZeroEncounter);
        }
        let mut session = RaceSession::open(context.instance_id, self.layout.clone(), now_ms)
            .map_err(ServiceError::Track)?;
        session.begin(now_ms).map_err(ServiceError::Track)?;
        self.context = Some(context);
        self.session = Some(session);
        self.phase = RacePhase::Countdown;
        self.schedule_cursor = 0;
        self.countdown_started_ms = now_ms;
        self.countdown_offset_ms = 0;
        self.last_boost_epoch = 0;
        self.claim_credit_m = 0.0;
        self.queued_items.clear();
        self.pickups.reset();
        self.pickups_spawned = false;
        self.shield_expiry = None;
        self.weapons = WeaponState::default();
        self.weapon_pose = None;
        self.race_started_ms = now_ms;
        self.last_position = self.content.start_position;
        self.last_claim_ms = now_ms;
        self.attempts = 0;
        self.observations = 0;
        self.countdown_sent = None;
        self.reward_taken = false;
        Ok(ServiceOutcome::Handled {
            replies: Vec::new(),
            reward_ready: false,
            abort: false,
        })
    }

    /// Load the original vehicle/track setup states without running the Go countdown.
    pub fn prepare(&mut self, context: RaceContext, now_ms: u64) -> Result<(), ServiceError> {
        self.begin(context, now_ms)?;
        self.phase = RacePhase::Preparing;
        Ok(())
    }

    /// Go resumes at the first countdown setup state, preserving loaded vehicles.
    pub fn start_countdown(&mut self, now_ms: u64) -> Result<(), ServiceError> {
        if self.phase != RacePhase::Preparing {
            return Err(ServiceError::AlreadyOpen { phase: self.phase });
        }
        let first = self
            .content
            .schedule
            .iter()
            .position(|s| {
                matches!(
                    s.frame,
                    ScheduledFrame::GameState(7..=10) | ScheduledFrame::Countdown(_)
                )
            })
            .unwrap_or(0);
        self.schedule_cursor = first;
        self.countdown_started_ms = now_ms;
        self.countdown_offset_ms = self
            .content
            .schedule
            .get(first)
            .map_or(0, |step| step.at_ms);
        self.race_started_ms = now_ms;
        self.last_claim_ms = now_ms;
        self.claim_credit_m = 0.0;
        self.phase = RacePhase::Countdown;
        Ok(())
    }

    /// Starts the racing phase. Called automatically when the schedule emits countdown 0.
    pub fn start_race(&mut self, now_ms: u64) -> Result<ServiceOutcome, ServiceError> {
        let Some(session) = self.session.as_mut() else {
            return Err(ServiceError::NoContext);
        };
        if self.phase != RacePhase::Countdown {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                phase: self.phase,
            }));
        }
        session.start(now_ms).map_err(ServiceError::Track)?;
        self.phase = RacePhase::Racing;
        self.race_started_ms = now_ms;
        self.last_claim_ms = now_ms;
        self.last_position = self.content.start_position;
        // The native countdown reaches zero before its state9 racing/toolbar transition.
        // Native035338 lost the first grid-box toolbar grant in that interval. Preserve
        // the race clock, but announce actors after the scheduled transition so a collision
        // cannot grant an item into the prior toolbar. Legacy schedules without state9
        // retain their countdown-zero announcement; no extra save flag is needed.
        let toolbar_transition_pending = self.content.schedule[self.schedule_cursor..]
            .iter()
            .any(|step| step.frame == ScheduledFrame::GameState(9));
        let replies = if toolbar_transition_pending {
            Vec::new()
        } else {
            self.spawn_pickups()
        };
        Ok(ServiceOutcome::Handled {
            replies,
            reward_ready: false,
            abort: false,
        })
    }

    /// Announces the live `33/15` boxes exactly once per race. Empty content returns no frames, so
    /// a legacy race's wire output is byte-identical to the pre-pickup service.
    fn spawn_pickups(&mut self) -> Vec<Vec<u8>> {
        if self.pickups_spawned {
            return Vec::new();
        }
        self.pickups_spawned = true;
        self.pickups.create_frames()
    }

    /// Emits every due schedule step in content order and enforces the race time window.
    pub fn tick(&mut self, now_ms: u64) -> Result<ServiceOutcome, ServiceError> {
        if self.gate == EntryGate::Disabled {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::GateDisabled));
        }
        if self.context.is_none() {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::NoOpenRace));
        }
        if self.phase == RacePhase::Racing {
            let elapsed = now_ms.saturating_sub(self.race_started_ms);
            if elapsed > self.content.bounds.max_race_ms {
                let replies = self.weapons.clear();
                self.abort();
                return Ok(ServiceOutcome::Handled {
                    replies,
                    reward_ready: false,
                    abort: true,
                });
            }
        }
        let mut replies = Vec::new();
        while self.schedule_cursor < self.content.schedule.len() {
            let step = self.content.schedule[self.schedule_cursor];
            if self.phase == RacePhase::Preparing
                && matches!(
                    step.frame,
                    ScheduledFrame::GameState(7..=10) | ScheduledFrame::Countdown(_)
                )
            {
                break;
            }
            // Go rebases only the remaining schedule, without subtracting from an early server clock.
            let due_at = self
                .countdown_started_ms
                .saturating_add(step.at_ms.saturating_sub(self.countdown_offset_ms));
            if due_at > now_ms {
                break;
            }
            self.schedule_cursor += 1;
            match step.frame {
                ScheduledFrame::GameState(state) => {
                    replies.push(kart_entry::encode_inner(
                        GAME_STATE_SUBTYPE,
                        &state.to_le_bytes(),
                    ));
                    if state == 9 && self.phase == RacePhase::Racing {
                        // State first, actors second: a grid collision then reaches the live
                        // racing toolbar. The existing one-shot guard also covers restore/ticks.
                        replies.extend(self.spawn_pickups());
                    }
                }
                ScheduledFrame::PlayerCount(value) => {
                    replies.push(kart_entry::encode_inner(
                        PLAYER_COUNT_SUBTYPE,
                        &value.to_le_bytes(),
                    ));
                }
                ScheduledFrame::NpcCount(value) => {
                    replies.push(kart_entry::encode_inner(
                        NPC_COUNT_SUBTYPE,
                        &value.to_le_bytes(),
                    ));
                }
                ScheduledFrame::Countdown(value) => {
                    let guid = self.context.map_or(0, |context| context.player_guid);
                    let mut payload = guid.to_le_bytes().to_vec();
                    payload.extend(value.to_le_bytes());
                    replies.push(kart_entry::encode_inner(COUNTDOWN_SUBTYPE, &payload));
                    self.countdown_sent = Some(value);
                    if value == 0 && self.phase == RacePhase::Countdown {
                        // Go both starts the race and emits its one-shot `33/15` spawns.
                        if let ServiceOutcome::Handled {
                            replies: spawns, ..
                        } = self.start_race(due_at)?
                        {
                            replies.extend(spawns);
                        }
                    }
                }
            }
        }
        // Timed pickup returns fire only inside a live racing window: after Go, before the race
        // window closes, never on a clock that predates the last accepted claim (a stale tick must
        // not resurrect a return the accepted stream has already moved past) and never after
        // finish/abort (those phases skip this block). A refused or full-queue claim never
        // schedules one, so nothing else moves.
        if self.phase == RacePhase::Racing
            && self.race_elapsed(now_ms).is_ok()
            && now_ms >= self.last_claim_ms
        {
            // The shield is turned off exactly once, at or after its deadline.
            if self.shield_expiry.is_some_and(|expiry| now_ms >= expiry) {
                self.shield_expiry = None;
                replies.push(encode_expire(SHIELD_EXPIRE_VALUE));
            }
            replies.extend(self.pickups.due_respawns(now_ms));
            if self.content.weapon_effects {
                replies.extend(self.weapons.tick(now_ms, self.weapon_pose.as_ref()));
            }
        }
        Ok(ServiceOutcome::Handled {
            replies,
            reward_ready: self.finish_flag(),
            abort: false,
        })
    }

    /// Inner 33 bytes in. Client claims are validated; server-only subtypes are refused.
    pub fn handle_inner(
        &mut self,
        inner: &[u8],
        now_ms: u64,
    ) -> Result<ServiceOutcome, ServiceError> {
        if self.gate == EntryGate::Disabled {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::GateDisabled));
        }
        let (subtype, payload) = kart_entry::decode_inner(inner)?;
        match subtype {
            BOOST_REQUEST_SUBTYPE => {
                let guid = self.owned_id(subtype, payload)?;
                if let Err(refusal) = self.own_guid(guid) {
                    return Ok(ServiceOutcome::Refused(refusal));
                }
                if self.phase != RacePhase::Racing {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                        phase: self.phase,
                    }));
                }
                // Lower edge first: a frame whose clock predates Go is a regression, never a free
                // refill. `saturating_sub` alone would read such a frame as elapsed zero.
                let elapsed = match self.race_elapsed(now_ms) {
                    Ok(elapsed) => elapsed,
                    Err(refusal) => return Ok(ServiceOutcome::Refused(refusal)),
                };
                // No fixed interval gates this request. The native client's refill accumulator is
                // config-driven (`FUN_00feaa30`: `vehicle+0x114 += config+0x740 * dt`, clamped at
                // `DAT_017ec67c`, reset to 0 by the sender in `FUN_00ba48a0`, and seeded full by
                // `FUN_00c5f7c0`), so legitimate requests land anywhere from ~10.6 s to minutes
                // after Go depending on the race config. The frame's only identifier is the owned
                // racer id pair parsed above; it carries no distinct request or transaction id, so
                // the recovered three-slot queue is the pacing bound: the client's own `33/23` case
                // pushes one type-5 entry, and a full queue refuses before anything moves.
                if self.queued_items.len() >= ITEM_QUEUE_CAP {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::ItemQueueFull));
                }
                // Kept for save compatibility and shape validation only; it no longer gates.
                self.last_boost_epoch = elapsed / BOOST_INTERVAL_MS;
                self.queued_items.push(BOOST_ITEM_TYPE);
                // This does not advance gates, finish the race or register an economy award.
                Ok(ServiceOutcome::Handled {
                    replies: vec![kart_entry::encode_inner(BOOST_RESPONSE_SUBTYPE, &[])],
                    reward_ready: false,
                    abort: false,
                })
            }
            USE_ITEM_SUBTYPE => {
                let guid = self.owned_id(subtype, payload)?;
                if let Err(refusal) = self.own_guid(guid) {
                    return Ok(ServiceOutcome::Refused(refusal));
                }
                if self.phase != RacePhase::Racing {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                        phase: self.phase,
                    }));
                }
                // The same lower-edge guard as the refill: a use before Go is a clock regression.
                let _elapsed = match self.race_elapsed(now_ms) {
                    Ok(elapsed) => elapsed,
                    Err(refusal) => return Ok(ServiceOutcome::Refused(refusal)),
                };
                // The captured order is request-then-response: the client asks to use, the server
                // names the item type, the client pops one entry and applies it. `33/19` carries no
                // request id, so two genuinely distinct presses are byte-identical; the sound bound
                // is once-per-queued-entry - consume exactly the head entry. The reliable ordered
                // transport (sequence plus CRC) already drops retransmissions below this service, so
                // a frame arriving here is treated as a further press and is admitted only while an
                // entry is left to pay for it.
                let Some(item) = self.queued_items.first().copied() else {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::NoItemQueued));
                };
                let mut weapon_replies = Vec::new();
                if self.content.weapon_effects && matches!(item, 1 | 3) {
                    let Some(observation) = self.weapon_pose else {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                    };
                    let kind = if item == 1 {
                        WeaponKind::Missile
                    } else {
                        WeaponKind::Mine
                    };
                    let race_end = self
                        .race_started_ms
                        .saturating_add(self.content.bounds.max_race_ms);
                    match self
                        .weapons
                        .use_item(kind, guid, &observation, now_ms, race_end)
                    {
                        Ok(frames) => weapon_replies = frames,
                        Err(_) => {
                            return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                        }
                    }
                }
                self.queued_items.remove(0);
                if item == SHIELD_ITEM_TYPE {
                    // Using a shield (re)starts the reconstructed lifetime; a repeated use refreshes
                    // the same deadline, and the deadline never outlives the race window.
                    let race_end_ms = self
                        .race_started_ms
                        .saturating_add(self.content.bounds.max_race_ms);
                    self.shield_expiry =
                        Some(now_ms.saturating_add(SHIELD_DURATION_MS).min(race_end_ms));
                }
                let mut replies = vec![kart_entry::encode_inner(
                    USE_ITEM_RESPONSE_SUBTYPE,
                    &item.to_le_bytes(),
                )];
                replies.extend(weapon_replies);
                Ok(ServiceOutcome::Handled {
                    replies,
                    reward_ready: false,
                    abort: false,
                })
            }
            GATE_HIT_SUBTYPE => {
                let claim = self.decode_claim(subtype, payload)?;
                if let Err(refusal) = self.own_guid(claim.guid) {
                    return Ok(ServiceOutcome::Refused(refusal));
                }
                self.claim_gate(claim.position, now_ms)
            }
            PICKUP_HIT_SUBTYPE => {
                let claim = self.decode_claim(subtype, payload)?;
                if let Err(refusal) = self.own_guid(claim.guid) {
                    return Ok(ServiceOutcome::Refused(refusal));
                }
                if self.phase != RacePhase::Racing {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                        phase: self.phase,
                    }));
                }
                // The claim keeps its own observation budget; the gate claim budget is untouched.
                if self.observations >= self.content.bounds.max_observations {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::ObservationBudget {
                        observations: self.observations,
                    }));
                }
                // Pickups share the race-window edges; the movement guard then bounds the leg. The
                // window is a precondition like the gate budget: a timeout or clock regression
                // must not spend it, or the refusal would dirty the saved race (F148).
                let _elapsed = match self.race_elapsed(now_ms) {
                    Ok(elapsed) => elapsed,
                    Err(refusal) => return Ok(ServiceOutcome::Refused(refusal)),
                };
                self.observations = self.observations.saturating_add(1);
                if !self.pickups_spawned {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::PickupsNotSpawned));
                }
                // Room for the grant before any box is consumed: a full mirror queue refuses and
                // leaves the box, the credit and the anchor untouched.
                if self.queued_items.len() >= ITEM_QUEUE_CAP {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::ItemQueueFull));
                }
                // Movement authority is shared with the gate claim: the same bounded credit window
                // decides whether the vehicle could have reached this position.
                let charge = match self.check_movement(claim.position, now_ms) {
                    Ok(charge) => charge,
                    Err(refusal) => return Ok(ServiceOutcome::Refused(refusal)),
                };
                let owned = self.context.map_or(0, |context| context.player_guid);
                let staged = match self.pickups.stage_claim(claim.guid, owned, claim.position) {
                    Ok(staged) => staged,
                    Err(PickupRefusal::ForeignGuid { offered, owned }) => {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::ForeignGuid {
                            offered,
                            owned,
                        }));
                    }
                    Err(PickupRefusal::NonFinitePosition) => {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::NonFinitePosition));
                    }
                    Err(_) => {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::NoPickupInRange));
                    }
                };
                let race_end_ms = self
                    .race_started_ms
                    .saturating_add(self.content.bounds.max_race_ms);
                let replies = match self.pickups.commit(staged, now_ms, race_end_ms) {
                    Ok(replies) => replies,
                    Err(_) => {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::NoPickupInRange));
                    }
                };
                // The grant is exactly one recovered item; the racing session is never touched.
                self.queued_items.push(staged.item_type());
                self.accept_movement(claim.position, now_ms, &charge);
                Ok(ServiceOutcome::Handled {
                    replies,
                    reward_ready: self.finish_flag(),
                    abort: false,
                })
            }
            SERVER_DATA_SUBTYPE => {
                self.require_payload(SERVER_DATA_SUBTYPE, payload, SERVER_DATA_PAYLOAD_LEN)?;
                if self.phase != RacePhase::Racing {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                        phase: self.phase,
                    }));
                }
                // This carries no guid: ownership is the open session, never the two tail words.
                // Observations keep their own budget so the high-rate bulk stream can never eat the
                // gate claim budget.
                if self.observations >= self.content.bounds.max_observations {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::ObservationBudget {
                        observations: self.observations,
                    }));
                }
                if self.content.weapon_effects {
                    if self.race_elapsed(now_ms).is_err() {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                    }
                    let read = |offset: usize| {
                        f32::from_le_bytes(payload[offset..offset + 4].try_into().unwrap())
                    };
                    let pose = WeaponPose {
                        basis: [
                            [read(0), read(4), read(8)],
                            [read(12), read(16), read(20)],
                            [read(24), read(28), read(32)],
                        ],
                        position: [read(36), read(40), read(44)],
                    };
                    if (0..19).any(|i| !read(i * 4).is_finite())
                        || pose.validate().is_err()
                        || payload[76] & !(1 | 2 | 4 | 64 | 128) != 0
                        || u32::from_le_bytes(payload[81..85].try_into().unwrap()) != 1
                    {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                    }
                    let (previous, stamp) = self
                        .weapon_pose
                        .map(|p| (p.pose.position, p.at_ms))
                        .unwrap_or((self.last_position, self.last_claim_ms));
                    if now_ms < stamp {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                    }
                    let allowance = self.content.bounds.max_speed_mps
                        * ((now_ms - stamp).min(3_000) as f32 / 1_000.0)
                        + 3.0;
                    let distance = pose
                        .position
                        .iter()
                        .zip(previous)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    if distance > allowance {
                        return Ok(ServiceOutcome::Refused(ServiceRefusal::WeaponUnavailable));
                    }
                    self.weapon_pose = Some(WeaponObservation {
                        pose,
                        at_ms: now_ms,
                    });
                }
                self.observations = self.observations.saturating_add(1);
                Ok(ServiceOutcome::Handled {
                    replies: Vec::new(),
                    reward_ready: self.finish_flag(),
                    abort: false,
                })
            }
            COUNTDOWN_SUBTYPE => {
                self.require_payload(COUNTDOWN_SUBTYPE, payload, COUNTDOWN_PAYLOAD_LEN)?;
                let guid = u64::from_le_bytes(payload[0..8].try_into().unwrap());
                let value = u32::from_le_bytes(payload[8..12].try_into().unwrap());
                if let Err(refusal) = self.own_guid(guid) {
                    return Ok(ServiceOutcome::Refused(refusal));
                }
                if !matches!(self.phase, RacePhase::Countdown | RacePhase::Racing) {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                        phase: self.phase,
                    }));
                }
                let sent = self.countdown_sent.unwrap_or(0);
                if value > sent {
                    return Ok(ServiceOutcome::Refused(ServiceRefusal::CountdownSpoof {
                        value,
                        sent,
                    }));
                }
                Ok(ServiceOutcome::Handled {
                    replies: Vec::new(),
                    reward_ready: self.finish_flag(),
                    abort: false,
                })
            }
            other => Ok(ServiceOutcome::Refused(
                ServiceRefusal::UnsupportedSubtype { subtype: other },
            )),
        }
    }

    /// Stops the race without a result. A finished race keeps its pending offer.
    pub fn abort(&mut self) {
        if self.phase != RacePhase::Finished {
            self.phase = RacePhase::Aborted;
            // No effect survives the race: the restore guard would refuse a live shield here.
            self.shield_expiry = None;
            self.weapons.clear();
            self.weapon_pose = None;
            if let Some(session) = self.session.as_mut() {
                session.abandon();
            }
        }
    }

    /// `None` while no race is open; the snapshot never panics.
    pub fn snapshot(&self) -> Option<ServiceSnapshot> {
        let (context, session) = (self.context?, self.session.as_ref()?);
        let pickup_state = self.pickups.snapshot();
        Some(ServiceSnapshot {
            phase: self.phase,
            context,
            session: session.snapshot(),
            bounds: self.content.bounds,
            gate_band: self.content.gate_band,
            gate_curtains: self.content.gate_curtains.clone(),
            schedule_cursor: self.schedule_cursor,
            countdown_started_ms: self.countdown_started_ms,
            countdown_offset_ms: self.countdown_offset_ms,
            last_position: self.last_position,
            last_claim_ms: self.last_claim_ms,
            race_started_ms: self.race_started_ms,
            attempts: self.attempts,
            observations: self.observations,
            countdown_sent: self.countdown_sent,
            last_boost_epoch: self.last_boost_epoch,
            claim_credit_m: self.claim_credit_m,
            queued_items: self.queued_items.clone(),
            pickups: self.content.pickups.clone(),
            pickups_consumed: pickup_state.consumed,
            pickups_spawned: self.pickups_spawned,
            pickup_respawn: self.content.pickup_respawn,
            pickups_respawned: pickup_state.respawned,
            pickups_pending: pickup_state.pending,
            shield_expiry: self.shield_expiry,
            weapon_effects: self.content.weapon_effects,
            weapons: self.weapons.clone(),
            weapon_pose: self.weapon_pose,
            reward_taken: self.reward_taken,
        })
    }

    /// Restores a saved race against the same content, refusing incoherent snapshots.
    pub fn restore(
        content: KartServiceContent,
        snapshot: ServiceSnapshot,
    ) -> Result<Self, ServiceError> {
        let (layout, band, curtains) = validate_content(&content)?;
        if snapshot.weapon_effects != content.weapon_effects
            || (!content.weapon_effects
                && (snapshot.weapons != WeaponState::default() || snapshot.weapon_pose.is_some()))
            || (!matches!(snapshot.phase, RacePhase::Racing)
                && (!snapshot.weapons.is_empty() || snapshot.weapon_pose.is_some()))
            || snapshot
                .weapons
                .validate_for(
                    snapshot.context.player_guid,
                    snapshot.race_started_ms,
                    snapshot
                        .race_started_ms
                        .saturating_add(content.bounds.max_race_ms),
                )
                .is_err()
            || snapshot.weapon_pose.is_some_and(|p| {
                p.pose.validate().is_err()
                    || p.at_ms < snapshot.race_started_ms
                    || p.at_ms
                        > snapshot
                            .race_started_ms
                            .saturating_add(content.bounds.max_race_ms)
            })
        {
            return Err(ServiceError::CorruptSnapshot { field: "weapons" });
        }
        // The context is validated exactly like `begin`, so a corrupt snapshot cannot smuggle in a
        // zero session, guid or instance id.
        if snapshot.context.session == 0 {
            return Err(ServiceError::ZeroSession);
        }
        if snapshot.context.player_guid == 0 {
            return Err(ServiceError::ZeroPlayerGuid);
        }
        if snapshot.context.instance_id == 0 {
            return Err(ServiceError::ZeroInstance);
        }
        if snapshot.context.encounter_id == 0 {
            return Err(ServiceError::ZeroEncounter);
        }
        let session = RaceSession::restore(snapshot.session.clone())
            .map_err(|_| ServiceError::CorruptSnapshot { field: "session" })?;
        if snapshot.schedule_cursor > content.schedule.len() {
            return Err(ServiceError::CorruptSnapshot {
                field: "schedule_cursor",
            });
        }
        let go_offset = content
            .schedule
            .iter()
            .find(|step| {
                matches!(
                    step.frame,
                    ScheduledFrame::GameState(7..=10) | ScheduledFrame::Countdown(_)
                )
            })
            .map_or(0, |step| step.at_ms);
        if (snapshot.countdown_offset_ms != 0 && snapshot.countdown_offset_ms != go_offset)
            || (snapshot.phase == RacePhase::Preparing && snapshot.countdown_offset_ms != 0)
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "countdown_offset_ms",
            });
        }
        if snapshot.attempts > content.bounds.max_claims {
            return Err(ServiceError::CorruptSnapshot { field: "attempts" });
        }
        if snapshot.observations > content.bounds.max_observations {
            return Err(ServiceError::CorruptSnapshot {
                field: "observations",
            });
        }
        if snapshot.last_boost_epoch > content.bounds.max_race_ms / BOOST_INTERVAL_MS
            || (snapshot.last_boost_epoch != 0
                && matches!(
                    snapshot.phase,
                    RacePhase::Idle | RacePhase::Preparing | RacePhase::Countdown
                ))
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "last_boost_epoch",
            });
        }
        // Structural bounds only: the mirrored queue is capped at the recovered three slots and may
        // hold only the recovered item types (1 missile, 3 mine, 4 shield, 5 boost). Restore cannot
        // prove the provenance of a hand-edited save, so it bounds the shape rather than claiming
        // to detect invented entries. A save that predates the field restores empty.
        if snapshot.queued_items.len() > ITEM_QUEUE_CAP
            || snapshot
                .queued_items
                .iter()
                .any(|item| !is_recovered_item_type(*item))
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "queued_items",
            });
        }
        // The snapshot must belong to this pickup content: a different definition set resuming a
        // consumed mark would move a box that never existed on that track.
        if snapshot.pickups != content.pickups {
            return Err(ServiceError::CorruptSnapshot { field: "pickups" });
        }
        // The respawn policy is content-bound too: a different policy must not resume a race.
        if snapshot.pickup_respawn != content.pickup_respawn {
            return Err(ServiceError::CorruptSnapshot {
                field: "pickup_respawn",
            });
        }
        if !snapshot.pickups_spawned
            && (!snapshot.pickups_consumed.is_empty()
                || !snapshot.pickups_respawned.is_empty()
                || !snapshot.pickups_pending.is_empty())
        {
            // A race that never announced a box cannot have collected one.
            return Err(ServiceError::CorruptSnapshot {
                field: "pickups_spawned",
            });
        }
        let pickups = PickupSet::restore(
            content.pickups.clone(),
            content.pickup_respawn,
            PickupSnapshot {
                consumed: snapshot.pickups_consumed.clone(),
                respawned: snapshot.pickups_respawned.clone(),
                pending: snapshot.pickups_pending.clone(),
            },
            snapshot.race_started_ms,
            snapshot
                .race_started_ms
                .saturating_add(content.bounds.max_race_ms),
        )
        .map_err(|_| ServiceError::CorruptSnapshot {
            field: "pickups_consumed",
        })?;
        // An active shield is runtime state: it may only exist in a racing phase, after Go and no
        // later than the race window closes, so a tampered save cannot park a live effect.
        if let Some(expiry) = snapshot.shield_expiry {
            let race_end = snapshot
                .race_started_ms
                .saturating_add(content.bounds.max_race_ms);
            if snapshot.phase != RacePhase::Racing
                || expiry < snapshot.race_started_ms
                || expiry > race_end
            {
                return Err(ServiceError::CorruptSnapshot {
                    field: "shield_expiry",
                });
            }
        }
        if snapshot.bounds != content.bounds {
            return Err(ServiceError::CorruptSnapshot { field: "bounds" });
        }
        if snapshot.gate_band != content.gate_band {
            return Err(ServiceError::CorruptSnapshot { field: "gate_band" });
        }
        if snapshot.gate_curtains != content.gate_curtains {
            return Err(ServiceError::CorruptSnapshot {
                field: "gate_curtains",
            });
        }
        let credit_cap = CLAIM_CREDIT_CAP_SECONDS * content.bounds.max_speed_mps;
        if !snapshot.claim_credit_m.is_finite()
            || snapshot.claim_credit_m < 0.0
            || snapshot.claim_credit_m > credit_cap
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "claim_credit_m",
            });
        }
        if snapshot.reward_taken && snapshot.phase != RacePhase::Finished {
            return Err(ServiceError::CorruptSnapshot {
                field: "reward_taken",
            });
        }
        if !finite(snapshot.last_position) {
            return Err(ServiceError::CorruptSnapshot {
                field: "last_position",
            });
        }
        // The snapshot must belong to this content: an altered track, price or session binding must
        // never resume into a different (possibly priced) race.
        let bound = &snapshot.session;
        if bound.speedway != layout.speedway
            || bound.laps != layout.laps
            || bound.checkpoints != layout.checkpoints
            || bound.reward != layout.reward
        {
            return Err(ServiceError::CorruptSnapshot { field: "track" });
        }
        if bound.session_id != snapshot.context.instance_id {
            return Err(ServiceError::CorruptSnapshot {
                field: "session_id",
            });
        }
        if snapshot.countdown_started_ms > snapshot.last_claim_ms
            && snapshot.phase != RacePhase::Countdown
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "countdown_started_ms",
            });
        }
        if snapshot.race_started_ms > snapshot.last_claim_ms
            && snapshot.phase != RacePhase::Countdown
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "last_claim_ms",
            });
        }
        if snapshot.race_started_ms < snapshot.countdown_started_ms {
            return Err(ServiceError::CorruptSnapshot {
                field: "race_started_ms",
            });
        }
        if snapshot.race_started_ms > snapshot.last_claim_ms
            && snapshot.phase != RacePhase::Countdown
        {
            return Err(ServiceError::CorruptSnapshot {
                field: "race_started_ms",
            });
        }
        let coherent = match snapshot.phase {
            RacePhase::Preparing => {
                session.phase() == Phase::Countdown
                    && snapshot.countdown_sent.is_none()
                    && !content.schedule[..snapshot.schedule_cursor]
                        .iter()
                        .any(|s| {
                            matches!(
                                s.frame,
                                ScheduledFrame::GameState(7..=10) | ScheduledFrame::Countdown(_)
                            )
                        })
            }
            RacePhase::Countdown => session.phase() == Phase::Countdown,
            RacePhase::Racing => session.phase() == Phase::Racing,
            RacePhase::Finished => session.phase() == Phase::Finished,
            RacePhase::Aborted => session.phase() == Phase::Abandoned,
            RacePhase::Idle => false,
        };
        if !coherent {
            return Err(ServiceError::CorruptSnapshot { field: "phase" });
        }
        Ok(Self {
            content,
            layout,
            band,
            curtains,
            gate: EntryGate::Disabled,
            phase: snapshot.phase,
            context: Some(snapshot.context),
            session: Some(session),
            schedule_cursor: snapshot.schedule_cursor,
            countdown_started_ms: snapshot.countdown_started_ms,
            countdown_offset_ms: snapshot.countdown_offset_ms,
            race_started_ms: snapshot.race_started_ms,
            last_position: snapshot.last_position,
            last_claim_ms: snapshot.last_claim_ms,
            attempts: snapshot.attempts,
            observations: snapshot.observations,
            countdown_sent: snapshot.countdown_sent,
            last_boost_epoch: snapshot.last_boost_epoch,
            claim_credit_m: snapshot.claim_credit_m,
            queued_items: snapshot.queued_items,
            pickups,
            pickups_spawned: snapshot.pickups_spawned,
            shield_expiry: snapshot.shield_expiry,
            weapons: snapshot.weapons,
            weapon_pose: snapshot.weapon_pose,
            reward_taken: snapshot.reward_taken,
        })
    }

    /// The race-window edges shared by the refill, use and pickup arms. A frame whose clock predates
    /// Go is a [`ServiceRefusal::ClaimRate`] regression (the same refusal the movement guard uses),
    /// a frame past the window is a timeout, and an in-window frame yields its elapsed time. Pure:
    /// a refusal leaves the queue, the legacy epoch and every other field untouched.
    fn race_elapsed(&self, now_ms: u64) -> Result<u64, ServiceRefusal> {
        if now_ms < self.race_started_ms {
            return Err(ServiceRefusal::ClaimRate { delta_ms: 0 });
        }
        let elapsed = now_ms - self.race_started_ms;
        if elapsed > self.content.bounds.max_race_ms {
            return Err(ServiceRefusal::RaceTimeout {
                elapsed_ms: elapsed,
                limit_ms: self.content.bounds.max_race_ms,
            });
        }
        Ok(elapsed)
    }

    /// Movement authority shared by gate hits and pickup claims: finite position, a monotonic
    /// clock, a bounded step and the same bounded travel credit. Pure - a refusal moves nothing.
    ///
    /// A claim pays with the elapsed time it is allowed (at most [`CLAIM_CREDIT_WINDOW_MS`], so
    /// idling cannot buy an unbounded jump) plus the saved credit; a claim inside the rate floor
    /// pays only from saved credit, so a client-side flush can spend slack but never mint new time.
    fn check_movement(
        &self,
        position: [f32; 3],
        now_ms: u64,
    ) -> Result<MovementCharge, ServiceRefusal> {
        if !finite(position) {
            return Err(ServiceRefusal::NonFinitePosition);
        }
        if now_ms < self.last_claim_ms {
            // A claim whose clock predates the accepted one is a regression, never free travel.
            return Err(ServiceRefusal::ClaimRate { delta_ms: 0 });
        }
        let delta_ms = now_ms.saturating_sub(self.last_claim_ms);
        if delta_ms > self.content.bounds.max_step_ms {
            return Err(ServiceRefusal::ClaimRate { delta_ms });
        }
        let distance = {
            let dx = position[0] - self.last_position[0];
            let dy = position[1] - self.last_position[1];
            let dz = position[2] - self.last_position[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        };
        let elapsed_ms = delta_ms.min(CLAIM_CREDIT_WINDOW_MS);
        let mut allowance =
            self.content.bounds.max_speed_mps * (elapsed_ms as f32 / 1_000.0) + self.claim_credit_m;
        if delta_ms < MIN_CLAIM_DELTA_MS {
            if distance > self.claim_credit_m {
                return Err(ServiceRefusal::ClaimRate { delta_ms });
            }
            allowance = self.claim_credit_m;
        }
        if !allowance.is_finite() || distance > allowance {
            let speed = if delta_ms == 0 {
                f32::INFINITY
            } else {
                distance / (delta_ms as f32 / 1_000.0)
            };
            return Err(ServiceRefusal::ImplausibleSpeed {
                speed_mps: speed,
                limit_mps: self.content.bounds.max_speed_mps,
            });
        }
        Ok(MovementCharge {
            allowance,
            distance,
        })
    }

    /// Applies an accepted charge: the anchor moves and the unused allowance banks (bounded, and
    /// only ever from travelled time).
    fn accept_movement(&mut self, position: [f32; 3], now_ms: u64, charge: &MovementCharge) {
        self.last_position = position;
        self.last_claim_ms = now_ms;
        let credit_cap = CLAIM_CREDIT_CAP_SECONDS * self.content.bounds.max_speed_mps;
        self.claim_credit_m = (charge.allowance - charge.distance).min(credit_cap);
    }

    fn own_guid(&self, offered: u64) -> Result<(), ServiceRefusal> {
        let Some(context) = self.context else {
            return Err(ServiceRefusal::NoOpenRace);
        };
        if offered != context.player_guid {
            return Err(ServiceRefusal::ForeignGuid {
                offered,
                owned: context.player_guid,
            });
        }
        Ok(())
    }

    fn require_payload(
        &self,
        subtype: u16,
        payload: &[u8],
        expected: usize,
    ) -> Result<(), ServiceError> {
        if payload.len() != expected {
            return Err(ServiceError::PayloadLength {
                subtype,
                expected,
                actual: payload.len(),
            });
        }
        Ok(())
    }

    /// The owned racer id pair `racer+0xe60 | racer+0xe64` that `33/19` and `33/22` both send as
    /// one little-endian u64. Width is checked first so a malformed frame is a typed error.
    fn owned_id(&self, subtype: u16, payload: &[u8]) -> Result<u64, ServiceError> {
        self.require_payload(subtype, payload, OWNED_ID_PAYLOAD_LEN)?;
        Ok(u64::from_le_bytes(
            payload.try_into().expect("checked width"),
        ))
    }

    /// The `33/7` gate claim and the `33/17` pickup claim share one layout, so the caller's subtype
    /// is carried into the width error instead of always naming the gate hit.
    fn decode_claim(&self, subtype: u16, payload: &[u8]) -> Result<Claim, ServiceError> {
        if payload.len() != GATE_HIT_PAYLOAD_LEN {
            return Err(ServiceError::PayloadLength {
                subtype,
                expected: GATE_HIT_PAYLOAD_LEN,
                actual: payload.len(),
            });
        }
        Ok(Claim {
            guid: u64::from_le_bytes(payload[0..8].try_into().unwrap()),
            position: [
                f32::from_le_bytes(payload[8..12].try_into().unwrap()),
                f32::from_le_bytes(payload[12..16].try_into().unwrap()),
                f32::from_le_bytes(payload[16..20].try_into().unwrap()),
            ],
        })
    }

    fn claim_gate(
        &mut self,
        position: [f32; 3],
        now_ms: u64,
    ) -> Result<ServiceOutcome, ServiceError> {
        if self.phase != RacePhase::Racing {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                phase: self.phase,
            }));
        }
        // Every in-window claim attempt counts against the bounded flood guard.
        if self.attempts >= self.content.bounds.max_claims {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::ClaimBudget {
                claims: self.attempts,
            }));
        }
        // The packet path must enforce the same whole-race window as `tick`: a claim arriving just
        // after the limit but before the next 20 Hz tick must not reach the finish line.
        if let Err(refusal) = self.race_elapsed(now_ms) {
            return Ok(ServiceOutcome::Refused(refusal));
        }
        self.attempts += 1;
        let charge = match self.check_movement(position, now_ms) {
            Ok(charge) => charge,
            Err(refusal) => return Ok(ServiceOutcome::Refused(refusal)),
        };
        let expected = self
            .session
            .as_ref()
            .and_then(|session| session.next_checkpoint())
            .unwrap_or(0);
        if expected == 0 {
            return Ok(ServiceOutcome::Refused(ServiceRefusal::InvalidPhase {
                phase: self.phase,
            }));
        }
        // Attribution: the expected gate wins even where physical volumes overlap, so an overlap can
        // never let a skipped or stale claim advance.
        match self.attribute_gate(position, expected) {
            GateAttribution::Expected => {}
            GateAttribution::Stale(got) => {
                return Ok(ServiceOutcome::Refused(ServiceRefusal::StaleGate {
                    expected,
                    got,
                }));
            }
            GateAttribution::Skipped(got) => {
                return Ok(ServiceOutcome::Refused(ServiceRefusal::SkippedGate {
                    expected,
                    got,
                }));
            }
            GateAttribution::Outside => {
                return Ok(ServiceOutcome::Refused(ServiceRefusal::OutsideGates));
            }
        }
        let got = expected;
        let outcome = self
            .session
            .as_mut()
            .expect("racing phase has a session")
            .on_checkpoint(now_ms, got);
        let refusal = match outcome {
            CheckpointOutcome::CheckpointAccepted { .. }
            | CheckpointOutcome::LapCompleted { .. } => None,
            CheckpointOutcome::ClockRegression { .. } | CheckpointOutcome::NotRacing => {
                Some(ServiceRefusal::ClaimRate { delta_ms: 0 })
            }
            CheckpointOutcome::Duplicate
            | CheckpointOutcome::OutOfOrder { .. }
            | CheckpointOutcome::UnknownCheckpoint(_) => {
                Some(ServiceRefusal::StaleGate { expected, got })
            }
        };
        if let Some(refusal) = refusal {
            return Ok(ServiceOutcome::Refused(refusal));
        }
        self.accept_movement(position, now_ms, &charge);
        let finished = self
            .session
            .as_ref()
            .is_some_and(|session| session.phase() == Phase::Finished);
        if finished {
            self.phase = RacePhase::Finished;
            // A finished race keeps its last progress frame; no effect expires after it.
            self.shield_expiry = None;
        }
        let reply = self.progress_frame(now_ms);
        let mut replies = vec![reply];
        if finished {
            replies.extend(self.weapons.clear());
            self.weapon_pose = None;
        }
        Ok(ServiceOutcome::Handled {
            replies,
            reward_ready: self.finish_flag(),
            abort: false,
        })
    }

    /// Attributes a claim position to a gate. The expected gate wins over any overlapping volume;
    /// otherwise the nearest later containing gate is a skip and the nearest earlier one is stale.
    fn attribute_gate(&self, position: [f32; 3], expected: u32) -> GateAttribution {
        if let Some(gate) = self.content.gates.get(expected as usize - 1) {
            if self.contains(expected as usize - 1, gate, position) {
                return GateAttribution::Expected;
            }
        }
        let mut skipped = None;
        let mut stale = None;
        for (index, gate) in self.content.gates.iter().enumerate() {
            if !self.contains(index, gate, position) {
                continue;
            }
            let id = index as u32 + 1;
            if id > expected {
                skipped = Some(skipped.map_or(id, |current: u32| current.min(id)));
            } else if id < expected {
                stale = Some(stale.map_or(id, |current: u32| current.max(id)));
            }
        }
        match (skipped, stale) {
            (Some(id), _) => GateAttribution::Skipped(id),
            (None, Some(id)) => GateAttribution::Stale(id),
            (None, None) => GateAttribution::Outside,
        }
    }

    /// True when a claim position lies inside this gate's volume: the authored CDTA curtains when
    /// the content supplies them, otherwise the reconstructed oriented band, otherwise the legacy
    /// sphere. Curtains win over a band so content can never widen an authored leaf back into a
    /// lane.
    ///
    /// A curtain claim is on the gate when its distance to one of the gate's bounded leaves is
    /// within that leaf's `half_depth`: the plane distance when the projection falls inside the
    /// ring, and the distance to the nearest edge or corner otherwise, so two gates' leaves never
    /// merge into one volume. The band frame is centred on the waypoint: `along` runs on the travel
    /// axis, `across` is the horizontal perpendicular and `y` is vertical; all three faces are
    /// inclusive.
    fn contains(&self, index: usize, gate: &GateWaypoint, position: [f32; 3]) -> bool {
        if let Some(volume) = &self.curtains {
            return volume.leaves[index]
                .iter()
                .zip(volume.half_depths[index].iter())
                .any(|(quad, half_depth)| curtain_distance(position, quad) <= *half_depth);
        }
        let dx = position[0] - gate.position[0];
        let dy = position[1] - gate.position[1];
        let dz = position[2] - gate.position[2];
        match &self.band {
            Some(volume) => {
                let axis = volume.axes[index];
                let along = dx * axis[0] + dz * axis[1];
                let across = dz * axis[0] - dx * axis[1];
                along.abs() <= volume.band.half_depth
                    && across.abs() <= volume.band.half_width
                    && dy.abs() <= volume.band.half_height
            }
            None => (dx * dx + dy * dy + dz * dz).sqrt() <= gate.radius,
        }
    }

    /// Builds the local racer's `33/14` block: the unmapped template fields plus this session's
    /// authoritative guid, laps, best lap and total time.
    fn progress_frame(&self, now_ms: u64) -> Vec<u8> {
        let context = self.context.expect("progress without context");
        let session = self.session.as_ref().expect("progress without session");
        let mut payload = self.content.progress_template;
        payload.racer_guid = context.player_guid;
        payload.laps_completed = session.laps_completed() as i32;
        payload.lap_limit = self.content.laps as i32;
        payload.local_racer = 1;
        payload.best_lap_bits = (session.best_lap_ms().unwrap_or(0) as f32 / 1_000.0).to_bits();
        let elapsed = now_ms.saturating_sub(self.race_started_ms) as f32 / 1_000.0;
        payload.total_time_bits = elapsed.to_bits();
        kart_entry::encode_inner(
            PROGRESS_SUBTYPE,
            &kart_progress_native::encode_progress(&payload),
        )
    }
}
