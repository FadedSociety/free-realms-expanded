//! Transport-free validation for client-reported player positions.
//!
//! The native opcode 125 has no velocity, sequence, or timestamp field. The
//! caller supplies server elapsed time and server-owned speed policy instead.
//! All tolerances and world bounds here are reconstructed server policy (R),
//! not recovered client constants.

use std::time::Duration;

/// Local defensive coordinate bound (R); native world bounds were not found.
pub const MAX_POSITION_COMPONENT: f32 = 1_000_000.0;
/// Prevent tiny/quantized receive intervals from producing a zero allowance (R).
pub const MIN_INTERVAL: Duration = Duration::from_millis(50);
/// Normal movement gets a wide multiplier for prediction, network jitter, and
/// differences between the client and server movement model (R). The capture
/// corpus includes 19.1 units in a 50 ms receive interval (382 units/second).
pub const SOFT_SPEED_MULTIPLIER: f64 = 40.0;
/// Gross movement beyond this envelope is rejected and snapped to the prior
/// accepted pose. Intermediate overages are clamped (R).
pub const HARD_SPEED_MULTIPLIER: f64 = 80.0;
/// Vertical envelope intentionally allows large jump/fall/zone-load deltas (R).
const VERTICAL_BASE_SPEED: f64 = 100.0;
const VERTICAL_GRACE: f64 = 20.0;
const VERTICAL_SOFT_MULTIPLIER: f64 = 3.0;
const VERTICAL_HARD_MULTIPLIER: f64 = 12.0;
/// Reconstructed positional match tolerance for an authorized discontinuity (R).
const DISCONTINUITY_RADIUS: f64 = 16.0;
/// Let client zone/teleport state settle before normal checks resume (R).
const DISCONTINUITY_TIMEOUT: Duration = Duration::from_secs(5);
/// Native opcode-125 bit 4 selects the 0.25 entity-speed scale at 0090e0f4;
/// sender 00934a20 sources it from the entity flag at +0x650.
const NATIVE_SLOW_SPEED_BIT: u8 = 1 << 4;
const NATIVE_SLOW_SPEED_SCALE: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementState {
    /// Server-authoritative zone identifier.
    pub zone_id: u32,
    pub position: [f32; 3],
    /// Sanitized horizontal direction. Opcode 125 carries a direction vector,
    /// not a quaternion.
    pub direction: [f32; 3],
    /// Client state byte. Bit 4 has a recovered quarter-speed selector; other
    /// bits remain unnamed and do not identify locomotion modes.
    pub state: u8,
    /// Preserved opaque client byte; native evidence does not establish flags.
    pub unknown: u8,
}

impl MovementState {
    pub fn new(zone_id: u32, position: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            zone_id,
            position,
            direction,
            state: 0,
            unknown: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementUpdate {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    /// Sender bit 4 selects a reduced entity-speed scale. Other bits lack
    /// verified walk/run/jump/fall/swim/mount meanings.
    pub state: u8,
    pub unknown: u8,
}

/// Server-owned limits and modifiers (R). No universal player locomotion
/// speeds or numeric mount modifier were recovered from the client.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpeedModifiers {
    /// Configured base speed in world units per second (R).
    pub base_speed_per_second: f32,
    /// Mount speed factor (R; the 1.5 maximum is wiki-derived, not client code).
    pub mount_multiplier: f32,
    /// Active buff factor (R).
    pub buff_multiplier: f32,
}

impl SpeedModifiers {
    /// (R) Base speed and buff modifiers are not server-owned yet; 10 units/s with the
    /// validator's generous capture-tested envelope preserves W53b reports while rejecting gross
    /// jumps. A boarded ride (`ride_speed`) scales the base by its speed over the base, held to
    /// the validator's 1.0..=1.5 mount range, so a ride that stands still (the Lawn Chair, 0)
    /// keeps the on-foot allowance.
    pub fn riding(ride_speed: Option<f32>) -> Self {
        const BASE: f32 = 10.0;
        Self {
            base_speed_per_second: BASE,
            mount_multiplier: ride_speed.map_or(1.0, |speed| (speed / BASE).clamp(1.0, 1.5)),
            buff_multiplier: 1.0,
        }
    }

    fn effective_speed(self, state: u8) -> Option<f64> {
        let base = f64::from(self.base_speed_per_second);
        let mount = f64::from(self.mount_multiplier);
        let buff = f64::from(self.buff_multiplier);
        if !base.is_finite()
            || !mount.is_finite()
            || !buff.is_finite()
            || base <= 0.0
            || mount < 1.0
            || buff < 1.0
            || base > 10_000.0
            // The 1.5 mount cap is wiki-derived; the client exposes no constant.
            || mount > 1.5
            || buff > 10.0
        {
            return None;
        }
        // (R) The reviewed mount guide says boosts do not stack. Use the larger
        // server-owned factor instead of multiplying mount and buff together.
        // (R) Treat the recovered client slow-speed bit as a cap reduction.
        // This untrusted bit can lower allowance but cannot raise it.
        let state_scale = if state & NATIVE_SLOW_SPEED_BIT != 0 {
            NATIVE_SLOW_SPEED_SCALE
        } else {
            1.0
        };
        Some(base * mount.max(buff) * state_scale)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscontinuityKind {
    ZoneChange,
    DeveloperTeleport,
}

/// Opaque, server-issued one-use authorization for a known relocation.
/// Construct this only from a server-controlled zone transition or relocation
/// response; there is no client field that can create or supply this token.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedDiscontinuity {
    kind: DiscontinuityKind,
    zone_id: u32,
    position: [f32; 3],
    elapsed: Duration,
}

impl ExpectedDiscontinuity {
    pub fn zone_change(zone_id: u32, position: [f32; 3]) -> Self {
        Self {
            kind: DiscontinuityKind::ZoneChange,
            zone_id,
            position,
            elapsed: Duration::ZERO,
        }
    }

    pub fn developer_teleport(zone_id: u32, position: [f32; 3]) -> Self {
        Self {
            kind: DiscontinuityKind::DeveloperTeleport,
            zone_id,
            position,
            elapsed: Duration::ZERO,
        }
    }

    pub fn kind(&self) -> DiscontinuityKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Accept,
    Clamp,
    RejectAndSnapBack,
    /// Ignore the client report without committing it or sending a correction.
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    WithinAllowance,
    ExpectedZoneChange,
    ExpectedDeveloperTeleport,
    SoftHorizontalOverspeed,
    SoftVerticalOverspeed,
    SoftHorizontalAndVerticalOverspeed,
    ZoneChangeWithoutMatchingToken,
    NonFinitePosition,
    OutOfBoundsPosition,
    NonFiniteDirection,
    InvalidServerSpeedPolicy,
    GrossHorizontalOverspeed,
    GrossVerticalOverspeed,
    GrossHorizontalAndVerticalOverspeed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationResult {
    pub decision: Decision,
    pub state: MovementState,
    pub reason: Reason,
    /// An unused token is returned so the integration layer can keep it in
    /// session state. Accepted ordinary movement consumes stale expectations.
    pub expected_discontinuity: Option<ExpectedDiscontinuity>,
}

/// Validate one client position against the previous accepted pose.
///
/// `elapsed` is server time since the previous client position report. A
/// matching discontinuity pose is accepted. Far reports are ignored until the
/// first in-radius report or the reconstructed five-second timeout. The server
/// must commit its own transition target when issuing the token; `Ignore` means
/// neither commit the client pose nor send a correction. Clear canceled tokens.
pub fn validate(
    previous: &MovementState,
    update: MovementUpdate,
    server_zone_id: u32,
    elapsed: Duration,
    modifiers: SpeedModifiers,
    mut expected_discontinuity: Option<ExpectedDiscontinuity>,
) -> ValidationResult {
    expected_discontinuity = expected_discontinuity.map(|mut token| {
        token.elapsed = token.elapsed.saturating_add(elapsed);
        token
    });

    if update.position.iter().any(|v| !v.is_finite()) {
        return rejected(previous, Reason::NonFinitePosition, expected_discontinuity);
    }
    if update
        .position
        .iter()
        .any(|v| v.abs() > MAX_POSITION_COMPONENT)
    {
        return rejected(
            previous,
            Reason::OutOfBoundsPosition,
            expected_discontinuity,
        );
    }
    if update.direction.iter().any(|v| !v.is_finite()) {
        return rejected(previous, Reason::NonFiniteDirection, expected_discontinuity);
    }

    if let Some(token) = expected_discontinuity
        .clone()
        .filter(|token| token.zone_id == server_zone_id)
    {
        let reason = match token.kind() {
            DiscontinuityKind::ZoneChange => Reason::ExpectedZoneChange,
            DiscontinuityKind::DeveloperTeleport => Reason::ExpectedDeveloperTeleport,
        };
        // (R) The location/zone packet may overtake a queued stale pose. Keep
        // the one-use token and silently drop far reports until target arrival
        // or timeout; never echo a snap-back to the old pre-transition pose.
        if token.elapsed < DISCONTINUITY_TIMEOUT {
            if within_radius(token.position, update.position, DISCONTINUITY_RADIUS) {
                return ValidationResult {
                    decision: Decision::Accept,
                    state: accepted_state(server_zone_id, update, previous.direction),
                    reason,
                    expected_discontinuity: None,
                };
            }
            return ValidationResult {
                decision: Decision::Ignore,
                state: *previous,
                reason,
                expected_discontinuity: Some(token),
            };
        }
        // (R) Expiry ends suppression. Continue against the server-committed
        // target held in `previous`, with ordinary movement and hack limits.
        expected_discontinuity = None;
    }

    if server_zone_id != previous.zone_id {
        return rejected(
            previous,
            Reason::ZoneChangeWithoutMatchingToken,
            expected_discontinuity,
        );
    }
    let Some(max_speed) = modifiers.effective_speed(update.state) else {
        return rejected(
            previous,
            Reason::InvalidServerSpeedPolicy,
            expected_discontinuity,
        );
    };

    // (R) Keep the full server elapsed interval so packet/zone-load stalls add
    // movement allowance instead of producing false positives.
    let dt = elapsed.as_secs_f64().max(MIN_INTERVAL.as_secs_f64());
    let dx = f64::from(update.position[0]) - f64::from(previous.position[0]);
    let dz = f64::from(update.position[2]) - f64::from(previous.position[2]);
    let dy = f64::from(update.position[1]) - f64::from(previous.position[1]);
    let horizontal = dx.hypot(dz);
    let vertical = dy.abs();

    let soft_horizontal = max_speed * dt * SOFT_SPEED_MULTIPLIER;
    let hard_horizontal = max_speed * dt * HARD_SPEED_MULTIPLIER;
    let vertical_speed = max_speed.max(VERTICAL_BASE_SPEED);
    let soft_vertical = vertical_speed * dt * VERTICAL_SOFT_MULTIPLIER + VERTICAL_GRACE;
    let hard_vertical = vertical_speed * dt * VERTICAL_HARD_MULTIPLIER + VERTICAL_GRACE;

    let hard_h = horizontal > hard_horizontal;
    let hard_v = vertical > hard_vertical;
    if hard_h || hard_v {
        let reason = match (hard_h, hard_v) {
            (true, true) => Reason::GrossHorizontalAndVerticalOverspeed,
            (true, false) => Reason::GrossHorizontalOverspeed,
            (false, true) => Reason::GrossVerticalOverspeed,
            (false, false) => unreachable!(),
        };
        return rejected(previous, reason, expected_discontinuity);
    }

    let soft_h = horizontal > soft_horizontal;
    let soft_v = vertical > soft_vertical;
    if !soft_h && !soft_v {
        return ValidationResult {
            decision: Decision::Accept,
            state: accepted_state(server_zone_id, update, previous.direction),
            reason: Reason::WithinAllowance,
            expected_discontinuity: None,
        };
    }

    let mut clamped = update;
    if soft_h && horizontal > 0.0 {
        let factor = soft_horizontal / horizontal;
        clamped.position[0] = (f64::from(previous.position[0]) + dx * factor) as f32;
        clamped.position[2] = (f64::from(previous.position[2]) + dz * factor) as f32;
    }
    if soft_v {
        clamped.position[1] =
            (f64::from(previous.position[1]) + dy.signum() * soft_vertical) as f32;
    }
    ValidationResult {
        decision: Decision::Clamp,
        state: accepted_state(server_zone_id, clamped, previous.direction),
        reason: match (soft_h, soft_v) {
            (true, true) => Reason::SoftHorizontalAndVerticalOverspeed,
            (true, false) => Reason::SoftHorizontalOverspeed,
            (false, true) => Reason::SoftVerticalOverspeed,
            (false, false) => unreachable!(),
        },
        // (R) A clamped ordinary pose supersedes a stale one-use relocation.
        expected_discontinuity: None,
    }
}

fn accepted_state(
    zone_id: u32,
    update: MovementUpdate,
    previous_direction: [f32; 3],
) -> MovementState {
    let x = f64::from(update.direction[0]);
    let z = f64::from(update.direction[2]);
    let length = x.hypot(z);
    let direction = if length > f64::EPSILON {
        [(x / length) as f32, 0.0, (z / length) as f32]
    } else {
        // Native SetHorizontalDirection skips the all-zero direction vector.
        previous_direction
    };
    MovementState {
        zone_id,
        position: update.position,
        direction,
        state: update.state,
        unknown: update.unknown,
    }
}

fn within_radius(a: [f32; 3], b: [f32; 3], radius: f64) -> bool {
    if a.iter().chain(&b).any(|v| !v.is_finite()) {
        return false;
    }
    let dx = f64::from(a[0]) - f64::from(b[0]);
    let dy = f64::from(a[1]) - f64::from(b[1]);
    let dz = f64::from(a[2]) - f64::from(b[2]);
    dx * dx + dy * dy + dz * dz <= radius * radius
}

fn rejected(
    previous: &MovementState,
    reason: Reason,
    expected_discontinuity: Option<ExpectedDiscontinuity>,
) -> ValidationResult {
    ValidationResult {
        decision: Decision::RejectAndSnapBack,
        state: *previous,
        reason,
        expected_discontinuity,
    }
}
