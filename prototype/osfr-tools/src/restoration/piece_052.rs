//! Piece 052 - enemy patrol and idle: the recovered NPC movement/animation wire
//! contract plus a bounded, reconstructed patrol controller.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Native NPC Motion and Animation.md`, re-derived against the
//! preserved inputs by the private recovery checker `piece_052.py`):
//!
//! * Movement is the standalone opcode 125 application message,
//!   `u16 opcode, u64 guid, 3f position, 3f rotation, u8 state, u8 unknown`
//!   = 36 bytes. Native reader `008e5940` (original file offset 1030083) consumes
//!   exactly those fields, initializes rotation W = 0 and has no timestamp field.
//!   The three rotation floats are a direction vector, not a truncated
//!   quaternion: native `0095dae0` masks with `[1,0,1,0]` (nonzero horizontal X/Z
//!   required), normalizes XYZ and derives yaw with `atan2(X, Z)`. Community NPC
//!   movement uses State 1 / Unknown 0.
//! * Animation is `u16 opcode(35), u16 subtype(8), u64 guid, i32 animationId,
//!   i32 unknown, u8 flags` = 21 bytes (native reader `00908a50`, subtype-8
//!   dispatch at 1093435). Flags bit 0 set writes the entity base/idle animation;
//!   clear immediately resolves and plays the animation.
//! * `Resources/CodeAnimationMappings.txt` maps data ids 1=`loc_stand`,
//!   2=`loc_walk`, 3=`loc_run`, 1000=`com_h2h_attack`, 1001=`com_h2h_attack_01`.
//!   The AddNpc spawn template carries Walk/Run/Stand animation ids in that order
//!   at offsets 137/141/145, and MovementType 2 at 233.
//!
//! RECONSTRUCTED, labelled at each call site: waypoint order, dwell time, patrol
//! speed, arrival radius and route extent bounds. The original per-actor patrol
//! routes and dwell timings are not recovered, so this module never ships one:
//! it validates a caller-supplied route and steps it deterministically. The
//! speed ceiling is the pinned community `Npc.Speed` default (6.25), which is
//! community evidence rather than first-party data.
//!
//! NOT CLAIMED: rendered native playback, per-model animation support, or that a
//! recovered enemy historically patrolled this way. The native gate belongs to
//! the lead; the replay checker in `the private recovery checker piece_052.py --run <capture>`
//! is the intended evidence tool for it.

/// Standalone movement application message opcode (native reader `008e5940`).
pub const MOVEMENT_OPCODE: u16 = 125;
/// Wire size of the recovered movement message.
pub const MOVEMENT_BYTES: usize = 36;
/// State byte used by community NPC movement; bit 0 set passes 0.0 as the speed
/// parameter, clear calls the entity speed getter (`008b3180`).
pub const MOVEMENT_STATE_MOVING: u8 = 1;
/// Base-player-update animation opcode.
pub const ANIMATION_OPCODE: u16 = 35;
/// Animation subtype carrying `SetAnimation`.
pub const ANIMATION_SUBTYPE: u16 = 8;
/// Wire size of the recovered animation message.
pub const ANIMATION_BYTES: usize = 21;
/// Flags bit 0: set assigns the entity base/idle animation.
pub const ANIMATION_FLAG_BASE: u8 = 1;

/// `loc_stand` data animation id (`CodeAnimationMappings.txt`).
pub const ANIM_ID_STAND: i32 = 1;
/// `loc_walk` data animation id.
pub const ANIM_ID_WALK: i32 = 2;
/// `loc_run` data animation id.
pub const ANIM_ID_RUN: i32 = 3;
/// `com_h2h_attack_01` data animation id, kept here because a patrolling enemy
/// that reaches its target still attacks with the same 35/8 family.
pub const ANIM_ID_ATTACK: i32 = 1001;

/// Maximum waypoints in one caller-supplied patrol route.
pub const MAX_WAYPOINTS: usize = 16;
/// Inferred patrol speed ceiling: the pinned community `Npc.Speed` default
/// (Sanctuary `Npc.cs`), not a first-party constant.
pub const MAX_PATROL_SPEED: f32 = 6.25;
/// Maximum accepted arrival radius, in metres.
pub const MAX_ARRIVAL_RADIUS: f32 = 5.0;
/// Maximum dwell time at one waypoint, in milliseconds.
pub const MAX_DWELL_MS: u32 = 10_000;
/// Maximum diagonal extent of a patrol route, in metres.
pub const MAX_ROUTE_EXTENT_M: f32 = 200.0;
/// Minimum interval between patrol updates, in milliseconds.
pub const TICK_MS: u64 = 100;
/// Largest wall-clock delta one update may simulate, in milliseconds. Bounds a
/// long stall or a scheduler jump to a short step instead of a teleport.
pub const MAX_STEP_MS: u64 = 250;

const MAX_ABS_POSITION: f32 = 1_000_000.0;

/// The Walk/Run/Stand animation ids in the exact order the AddNpc spawn template
/// stores them at offsets 137/141/145.
pub fn locomotion_animation_ids() -> (i32, i32, i32) {
    (ANIM_ID_WALK, ANIM_ID_RUN, ANIM_ID_STAND)
}

/// Encode one opcode 125 movement message.
///
/// `position` and `heading` must already be finite; the controller validates
/// them. `heading` is the true direction vector (`atan2(X, Z)` yaw), so an
/// all-zero or vertical-only value would be ignored by the native setter.
pub fn movement_packet(guid: u64, position: [f32; 3], heading: [f32; 3]) -> Vec<u8> {
    let mut b = MOVEMENT_OPCODE.to_le_bytes().to_vec();
    b.extend(guid.to_le_bytes());
    for value in position.into_iter().chain(heading) {
        b.extend(value.to_le_bytes());
    }
    b.push(MOVEMENT_STATE_MOVING);
    b.push(0);
    b
}

/// Encode one opcode 35/subtype 8 animation message.
///
/// `base` sets the entity's base animation (Flags bit 0); clearing it plays the
/// animation immediately, which is what an attack uses.
pub fn animation_packet(guid: u64, animation_id: i32, base: bool) -> Vec<u8> {
    let mut b = ANIMATION_OPCODE.to_le_bytes().to_vec();
    b.extend(ANIMATION_SUBTYPE.to_le_bytes());
    b.extend(guid.to_le_bytes());
    b.extend(animation_id.to_le_bytes());
    b.extend(0_i32.to_le_bytes());
    b.push(u8::from(base));
    b
}

/// One patrol waypoint. `dwell_ms` is reconstructed policy, not recovered data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Waypoint {
    pub position: [f32; 3],
    pub dwell_ms: u32,
}

/// Why a caller-supplied patrol route or controller was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatrolError {
    /// A patrol needs at least two waypoints to have a leg.
    TooFewWaypoints,
    /// More waypoints than [`MAX_WAYPOINTS`].
    TooManyWaypoints,
    /// A waypoint coordinate is NaN or infinite.
    NonFinitePosition,
    /// A waypoint lies beyond the accepted coordinate range.
    PositionOutOfRange,
    /// A leg has no horizontal length, so no direction vector exists for it.
    VerticalOnlySegment,
    /// A leg is already inside the arrival radius, so it could never be walked.
    SegmentInsideArrivalRadius,
    /// Speed is NaN or infinite.
    NonFiniteSpeed,
    /// Speed is zero, negative or above [`MAX_PATROL_SPEED`].
    SpeedOutOfRange,
    /// Arrival radius is not a finite value in `0 < r <= MAX_ARRIVAL_RADIUS`.
    InvalidArrivalRadius,
    /// A dwell time is above [`MAX_DWELL_MS`].
    DwellOutOfRange,
    /// The route's bounding box exceeds [`MAX_ROUTE_EXTENT_M`].
    RouteTooLarge,
    /// Guid 0 is not an entity and guid 1 is the local player, never an enemy.
    InvalidGuid,
}

impl std::fmt::Display for PatrolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            PatrolError::TooFewWaypoints => "patrol route needs at least two waypoints",
            PatrolError::TooManyWaypoints => "patrol route has too many waypoints",
            PatrolError::NonFinitePosition => "patrol waypoint is not finite",
            PatrolError::PositionOutOfRange => "patrol waypoint is out of range",
            PatrolError::VerticalOnlySegment => "patrol leg has no horizontal direction",
            PatrolError::SegmentInsideArrivalRadius => "patrol leg is inside the arrival radius",
            PatrolError::NonFiniteSpeed => "patrol speed is not finite",
            PatrolError::SpeedOutOfRange => "patrol speed is out of range",
            PatrolError::InvalidArrivalRadius => "patrol arrival radius is out of range",
            PatrolError::DwellOutOfRange => "patrol dwell time is out of range",
            PatrolError::RouteTooLarge => "patrol route is too large",
            PatrolError::InvalidGuid => "patrol guid is not an enemy guid",
        };
        f.write_str(text)
    }
}

impl std::error::Error for PatrolError {}

/// A validated, bounded patrol route.
#[derive(Clone, Debug, PartialEq)]
pub struct PatrolRoute {
    waypoints: Vec<Waypoint>,
    speed: f32,
    arrival_radius: f32,
}

impl PatrolRoute {
    /// Validate and accept a patrol route.
    ///
    /// The route walks the waypoints in order and loops from the last back to
    /// the first. Every leg (including the closing leg) must have a nonzero
    /// horizontal direction and a planar length greater than the arrival radius,
    /// otherwise no heading exists for it and the actor could never arrive.
    pub fn new(
        waypoints: Vec<Waypoint>,
        speed: f32,
        arrival_radius: f32,
    ) -> Result<Self, PatrolError> {
        if waypoints.len() < 2 {
            return Err(PatrolError::TooFewWaypoints);
        }
        if waypoints.len() > MAX_WAYPOINTS {
            return Err(PatrolError::TooManyWaypoints);
        }
        if !speed.is_finite() {
            return Err(PatrolError::NonFiniteSpeed);
        }
        if speed <= 0.0 || speed > MAX_PATROL_SPEED {
            return Err(PatrolError::SpeedOutOfRange);
        }
        if !arrival_radius.is_finite()
            || arrival_radius <= 0.0
            || arrival_radius > MAX_ARRIVAL_RADIUS
        {
            return Err(PatrolError::InvalidArrivalRadius);
        }
        for waypoint in &waypoints {
            if waypoint.position.iter().any(|value| !value.is_finite()) {
                return Err(PatrolError::NonFinitePosition);
            }
            if waypoint
                .position
                .iter()
                .any(|value| value.abs() > MAX_ABS_POSITION)
            {
                return Err(PatrolError::PositionOutOfRange);
            }
            if waypoint.dwell_ms > MAX_DWELL_MS {
                return Err(PatrolError::DwellOutOfRange);
            }
        }
        let mut extent = [0.0_f32; 3];
        for axis in 0..3 {
            let min = waypoints
                .iter()
                .map(|w| w.position[axis])
                .fold(f32::INFINITY, f32::min);
            let max = waypoints
                .iter()
                .map(|w| w.position[axis])
                .fold(f32::NEG_INFINITY, f32::max);
            extent[axis] = max - min;
        }
        if (extent[0] * extent[0] + extent[1] * extent[1] + extent[2] * extent[2]).sqrt()
            > MAX_ROUTE_EXTENT_M
        {
            return Err(PatrolError::RouteTooLarge);
        }
        for index in 0..waypoints.len() {
            let from = waypoints[index].position;
            let to = waypoints[(index + 1) % waypoints.len()].position;
            let full =
                ((to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2) + (to[2] - from[2]).powi(2))
                    .sqrt();
            let horizontal = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
            if full <= f32::EPSILON {
                return Err(PatrolError::SegmentInsideArrivalRadius);
            }
            if horizontal <= f32::EPSILON {
                return Err(PatrolError::VerticalOnlySegment);
            }
            if horizontal <= arrival_radius {
                return Err(PatrolError::SegmentInsideArrivalRadius);
            }
        }
        Ok(Self {
            waypoints,
            speed,
            arrival_radius,
        })
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn arrival_radius(&self) -> f32 {
        self.arrival_radius
    }
}

/// Current locomotion state of one patrol controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatrolPhase {
    /// Standing still at the current waypoint for its dwell window.
    Idle,
    /// Running toward the next waypoint.
    Running,
}

/// What one controller update emitted.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PatrolUpdate {
    /// Wire messages in the order they must be sent.
    pub replies: Vec<Vec<u8>>,
    /// True when the update changed the actor position.
    pub moved: bool,
    /// Locomotion animation id this update switched to, when it switched.
    pub animation: Option<i32>,
}

/// Bounded patrol state machine for one enemy actor.
///
/// The controller owns only position, heading, the current waypoint and the
/// locomotion phase. It emits the recovered movement/animation messages and
/// never invents a route, speed or dwell value; those come from the caller.
#[derive(Clone, Debug)]
pub struct PatrolController {
    guid: u64,
    route: PatrolRoute,
    position: [f32; 3],
    heading: [f32; 3],
    waypoint_index: usize,
    phase: PatrolPhase,
    idle_until_ms: u64,
    last_tick_ms: u64,
}

impl PatrolController {
    /// Start an actor at the first waypoint, standing until its dwell expires.
    pub fn new(guid: u64, route: PatrolRoute, start_ms: u64) -> Result<Self, PatrolError> {
        if guid <= 1 {
            return Err(PatrolError::InvalidGuid);
        }
        let position = route.waypoints[0].position;
        let next = route.waypoints[1].position;
        let heading = horizontal_heading(next, position, [1.0, 0.0, 0.0]);
        Ok(Self {
            guid,
            idle_until_ms: start_ms.saturating_add(route.waypoints[0].dwell_ms as u64),
            last_tick_ms: start_ms,
            route,
            position,
            heading,
            waypoint_index: 0,
            phase: PatrolPhase::Idle,
        })
    }

    pub fn guid(&self) -> u64 {
        self.guid
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn heading(&self) -> [f32; 3] {
        self.heading
    }

    pub fn phase(&self) -> PatrolPhase {
        self.phase
    }

    /// Index of the waypoint the actor is standing at, or heading away from.
    pub fn waypoint_index(&self) -> usize {
        self.waypoint_index
    }

    /// Advance the controller to `now_ms`.
    ///
    /// Updates are throttled to [`TICK_MS`] and simulate at most [`MAX_STEP_MS`]
    /// per call, so a stalled or jumping clock cannot teleport the actor. The
    /// run leg emits the run base animation once, movement while it walks, and
    /// the stand base animation on arrival, which is the idle/run transition
    /// shape the original client plays.
    pub fn tick(&mut self, now_ms: u64) -> PatrolUpdate {
        let mut update = PatrolUpdate::default();
        if now_ms < self.last_tick_ms.saturating_add(TICK_MS) {
            return update;
        }
        let mut started_run = false;
        if self.phase == PatrolPhase::Idle {
            if now_ms < self.idle_until_ms {
                return update;
            }
            self.phase = PatrolPhase::Running;
            // Charge one tick quantum so the first run update already moves;
            // the step is still bounded by MAX_STEP_MS.
            self.last_tick_ms = now_ms.saturating_sub(TICK_MS);
            started_run = true;
        }
        let target_index = (self.waypoint_index + 1) % self.route.waypoints.len();
        let target = self.route.waypoints[target_index].position;
        let delta = [
            target[0] - self.position[0],
            target[1] - self.position[1],
            target[2] - self.position[2],
        ];
        let horizontal = (delta[0] * delta[0] + delta[2] * delta[2]).sqrt();
        let elapsed_ms = now_ms.saturating_sub(self.last_tick_ms).min(MAX_STEP_MS);
        self.last_tick_ms = now_ms;
        if horizontal <= self.route.arrival_radius {
            self.position = target;
            self.phase = PatrolPhase::Idle;
            self.waypoint_index = target_index;
            self.idle_until_ms =
                now_ms.saturating_add(self.route.waypoints[target_index].dwell_ms as u64);
            update
                .replies
                .push(movement_packet(self.guid, self.position, self.heading));
            update
                .replies
                .push(animation_packet(self.guid, ANIM_ID_STAND, true));
            update.animation = Some(ANIM_ID_STAND);
            update.moved = true;
            return update;
        }
        let full = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        let max_step = self.route.speed * elapsed_ms as f32 / 1000.0;
        // Never overshoot past the arrival radius: the next update then arrives
        // instead of oscillating around the waypoint.
        let scale = (max_step / full).min((horizontal - self.route.arrival_radius) / horizontal);
        self.heading = [delta[0] / horizontal, 0.0, delta[2] / horizontal];
        for axis in 0..3 {
            self.position[axis] += delta[axis] * scale;
        }
        update
            .replies
            .push(movement_packet(self.guid, self.position, self.heading));
        if started_run {
            update
                .replies
                .push(animation_packet(self.guid, ANIM_ID_RUN, true));
            update.animation = Some(ANIM_ID_RUN);
        }
        update.moved = true;
        update
    }
}

/// Planar unit heading from `from` toward `to`, falling back to `fallback` when
/// the leg has no horizontal direction (route validation prevents that case).
fn horizontal_heading(to: [f32; 3], from: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let dx = to[0] - from[0];
    let dz = to[2] - from[2];
    let length = (dx * dx + dz * dz).sqrt();
    if length <= f32::EPSILON {
        fallback
    } else {
        [dx / length, 0.0, dz / length]
    }
}
