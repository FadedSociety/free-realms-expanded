//! Piece 053 - aggro and leash: acquisition, pursuit, disengage/reset and invalid or
//! out-of-zone target rejection.
//!
//! ORIGINAL facts used here (provenance in `notes/Research/Pieces/053 Aggro and leash.md`):
//!
//! * The `AddNpc` wire record carries an `int` `Disposition` with the recovered
//!   `0` hostile / `1` neutral / `2` ally values
//!   (`lab/references/Sanctuary/src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketAddNpc.cs`,
//!   pinned revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`).
//! * The native selector `00a2ac20` requires `Disposition == 0` at NPC `+0x178`, rejects the dead
//!   flag at `+0x6f8` bit 7, then applies a facing dot product and a line-of-sight ray
//!   (`lab/reports/native-combat-controls.json`). That is native target *selection*: the same
//!   eligibility facts bound an NPC's aggro here, and the facing/LOS test stays with the client.
//!
//! RECONSTRUCTED policy, labelled at each constant and call site: the pursuit radius, melee
//! reach, movement speed, step clamp, minimum tick interval, stop margin and the
//! return-to-anchor leash. The numeric values mirror the currently registered `encounter.rs`
//! behaviour (`delta > 40.0` drops pursuit, 3 units per second clamped to 250 ms, stop margin
//! 2.4 so f32 rounding cannot strand a pursuer outside the 2.5 melee reach) so the lead can
//! register this module without a behaviour change. The original server's aggro and leash
//! numbers were not recovered.
//!
//! No wire serialization lives here: movement, animation and attack messages stay with the
//! owning encounter subsystem. This module reports only what the state machine decided.

/// Recovered hostile `Disposition` on the `AddNpc` wire record; the native selector accepts it.
pub const DISPOSITION_HOSTILE: i32 = 0;
/// Recovered neutral `Disposition`; such an actor never acquires an aggro target.
pub const DISPOSITION_NEUTRAL: i32 = 1;
/// Recovered ally `Disposition`; such an actor never acquires an aggro target.
pub const DISPOSITION_ALLY: i32 = 2;

/// Reconstructed pursuit radius in world units. `40.0` is inside the band; `40.001` is not.
pub const AGGRO_PURSUIT_RANGE: f32 = 40.0;
/// Reconstructed melee reach in world units. `2.5` is inside the band; `2.501` is not.
pub const AGGRO_MELEE_REACH: f32 = 2.5;
/// Reconstructed pursuit speed in world units per second.
pub const AGGRO_MOVE_SPEED: f32 = 3.0;
/// Reconstructed cap on the interval one tick may advance the actor for.
pub const AGGRO_STEP_CLAMP_MS: u64 = 250;
/// Reconstructed minimum interval between two movement ticks.
pub const AGGRO_TICK_INTERVAL_MS: u64 = 100;
/// Reconstructed stopping margin; keeps the rounded f32 position inside [`AGGRO_MELEE_REACH`].
pub const AGGRO_STOP_MARGIN: f32 = 2.4;
/// Distance at or below which a returning actor is snapped onto its anchor.
pub const AGGRO_HOME_EPSILON: f32 = 0.01;

/// Where an actor is in the reconstruction's aggro cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggroPhase {
    /// Parked at the anchor, scanning for an eligible target.
    Idle,
    /// Has a target: closing to melee reach, or already inside it.
    Pursuing,
    /// Lost the target and walking back to the anchor; it cannot re-acquire until it arrives.
    Returning,
}

/// Why a candidate cannot be aggroed. Hostile/alive are original eligibility; the non-finite,
/// zone and range tests are this reconstruction's fail-closed policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRejection {
    NotHostile,
    Dead,
    OutOfZone,
    OutOfRange,
    NonFinitePosition,
}

/// Why a tracked target was dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisengageReason {
    OutOfRange,
    Dead,
    OutOfZone,
    TargetGone,
}

/// One candidate a caller offers to an actor. `zone` is the caller's zone/instance tag: the world
/// stage and a dungeon instance use different tags, so a target from another one is out of zone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TargetCandidate {
    pub guid: u64,
    pub zone: i32,
    pub position: [f32; 3],
    pub alive: bool,
}

/// What one tick decided. Movement and attack messages are the owning subsystem's job.
#[derive(Clone, Debug, PartialEq)]
pub enum AggroEvent {
    /// The actor picked up a target this tick.
    Acquired { target: u64 },
    /// The actor moved; `heading` is the horizontal direction it moved toward.
    Pursuing {
        target: u64,
        position: [f32; 3],
        heading: [f32; 3],
    },
    /// The target is inside [`AGGRO_MELEE_REACH`]; the owner applies its own damage cooldown.
    InReach { target: u64 },
    /// The actor dropped its target and started returning.
    Disengaged {
        target: u64,
        reason: DisengageReason,
    },
    /// The actor reached its anchor and is idle again.
    ReturnedHome { position: [f32; 3] },
}

/// An NPC's aggro state machine. Pure state and geometry: no clock, no packets, no damage.
#[derive(Clone, Debug)]
pub struct AggroActor {
    guid: u64,
    disposition: i32,
    zone: i32,
    anchor: [f32; 3],
    position: [f32; 3],
    phase: AggroPhase,
    target: Option<u64>,
    last_tick_ms: Option<u64>,
}

impl AggroActor {
    /// Park a new actor on its anchor. The anchor doubles as the leash point. `disposition` is the
    /// actor's own recovered `Disposition`; anything but `0` (hostile) never acquires a target.
    pub fn new(guid: u64, disposition: i32, zone: i32, anchor: [f32; 3]) -> Self {
        Self {
            guid,
            disposition,
            zone,
            anchor,
            position: anchor,
            phase: AggroPhase::Idle,
            target: None,
            last_tick_ms: None,
        }
    }

    pub fn guid(&self) -> u64 {
        self.guid
    }

    pub fn disposition(&self) -> i32 {
        self.disposition
    }

    pub fn phase(&self) -> AggroPhase {
        self.phase
    }

    pub fn position(&self) -> [f32; 3] {
        self.position
    }

    pub fn anchor(&self) -> [f32; 3] {
        self.anchor
    }

    pub fn target(&self) -> Option<u64> {
        self.target
    }

    /// Advance the actor. `candidates` is the caller's current view of nearby players; a candidate
    /// that disappears from the slice counts as gone.
    ///
    /// Ticks closer together than [`AGGRO_TICK_INTERVAL_MS`] change nothing, mirroring the
    /// registered encounter loop; the first tick behaves like one full interval.
    pub fn tick(&mut self, candidates: &[TargetCandidate], now_ms: u64) -> Vec<AggroEvent> {
        if !valid_position(self.anchor) || !valid_position(self.position) {
            return Vec::new();
        }
        let elapsed = self
            .last_tick_ms
            .map_or(AGGRO_TICK_INTERVAL_MS, |last| now_ms.saturating_sub(last));
        if elapsed < AGGRO_TICK_INTERVAL_MS {
            return Vec::new();
        }
        self.last_tick_ms = Some(now_ms);
        match self.phase {
            AggroPhase::Idle => self.acquire(candidates),
            AggroPhase::Pursuing => self.pursue(candidates, elapsed),
            AggroPhase::Returning => self.return_home(elapsed),
        }
    }

    fn acquire(&mut self, candidates: &[TargetCandidate]) -> Vec<AggroEvent> {
        let mut eligible = candidates
            .iter()
            .filter(|candidate| {
                rejection_for(candidate, self.disposition, self.zone, self.position).is_none()
            })
            .map(|candidate| (candidate.guid, distance(self.position, candidate.position)))
            .collect::<Vec<_>>();
        // Deterministic pick: nearest first, lowest guid on a tie.
        eligible.sort_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(left.0.cmp(&right.0))
        });
        let Some((guid, _)) = eligible.first().copied() else {
            return Vec::new();
        };
        self.target = Some(guid);
        self.phase = AggroPhase::Pursuing;
        vec![AggroEvent::Acquired { target: guid }]
    }

    fn pursue(&mut self, candidates: &[TargetCandidate], elapsed_ms: u64) -> Vec<AggroEvent> {
        let Some(target) = self.target else {
            self.phase = AggroPhase::Returning;
            return Vec::new();
        };
        let tracked = candidates.iter().find(|candidate| candidate.guid == target);
        match tracked {
            None => {
                self.target = None;
                self.phase = AggroPhase::Returning;
                vec![AggroEvent::Disengaged {
                    target,
                    reason: DisengageReason::TargetGone,
                }]
            }
            Some(candidate) => match disengage_reason(candidate, self.zone, self.position) {
                Some(reason) => {
                    self.target = None;
                    self.phase = AggroPhase::Returning;
                    vec![AggroEvent::Disengaged { target, reason }]
                }
                None => self.close_in(candidate, elapsed_ms),
            },
        }
    }

    fn close_in(&mut self, target: &TargetCandidate, elapsed_ms: u64) -> Vec<AggroEvent> {
        let guid = target.guid;
        let delta = distance(self.position, target.position);
        if delta <= AGGRO_MELEE_REACH {
            return vec![AggroEvent::InReach { target: guid }];
        }
        let heading = horizontal_heading(self.position, target.position, delta);
        // Stop inside reach so f32 rounding at world coordinates cannot strand a pursuer just
        // outside the attack threshold.
        let step = (AGGRO_MOVE_SPEED * elapsed_ms.min(AGGRO_STEP_CLAMP_MS) as f32 / 1000.0)
            .min(delta - AGGRO_STOP_MARGIN);
        self.step_toward(target.position, delta, step);
        vec![AggroEvent::Pursuing {
            target: guid,
            position: self.position,
            heading,
        }]
    }

    fn return_home(&mut self, elapsed_ms: u64) -> Vec<AggroEvent> {
        let delta = distance(self.position, self.anchor);
        let step = AGGRO_MOVE_SPEED * elapsed_ms.min(AGGRO_STEP_CLAMP_MS) as f32 / 1000.0;
        if delta > AGGRO_HOME_EPSILON {
            self.step_toward(self.anchor, delta, step);
        }
        if distance(self.position, self.anchor) <= AGGRO_HOME_EPSILON {
            self.position = self.anchor;
            self.phase = AggroPhase::Idle;
            return vec![AggroEvent::ReturnedHome {
                position: self.position,
            }];
        }
        Vec::new()
    }

    fn step_toward(&mut self, goal: [f32; 3], delta: f32, step: f32) {
        let step = step.min(delta);
        for (coord, to) in self.position.iter_mut().zip(goal) {
            *coord += (to - *coord) / delta * step;
        }
    }
}

/// First typed reason a candidate cannot be aggroed by this actor, or `None` when it is eligible.
///
/// The hostile/alive tests are original eligibility; the non-finite, zone and range tests are this
/// reconstruction's fail-closed policy.
pub fn rejection_for(
    candidate: &TargetCandidate,
    actor_disposition: i32,
    actor_zone: i32,
    actor_position: [f32; 3],
) -> Option<TargetRejection> {
    if !valid_position(candidate.position) || !valid_position(actor_position) {
        return Some(TargetRejection::NonFinitePosition);
    }
    if actor_disposition != DISPOSITION_HOSTILE {
        return Some(TargetRejection::NotHostile);
    }
    if !candidate.alive {
        return Some(TargetRejection::Dead);
    }
    if candidate.zone != actor_zone {
        return Some(TargetRejection::OutOfZone);
    }
    if distance(actor_position, candidate.position) > AGGRO_PURSUIT_RANGE {
        return Some(TargetRejection::OutOfRange);
    }
    None
}

/// Why an already-tracked target must be dropped, or `None` while it stays valid.
///
/// The actor's own `Disposition` is a spawn-time property, so only the candidate's reasons are
/// reported here; a non-finite target position is treated as a gone target until the caller has a
/// valid one.
pub fn disengage_reason(
    candidate: &TargetCandidate,
    actor_zone: i32,
    actor_position: [f32; 3],
) -> Option<DisengageReason> {
    if !valid_position(candidate.position) || !valid_position(actor_position) {
        return Some(DisengageReason::TargetGone);
    }
    if !candidate.alive {
        return Some(DisengageReason::Dead);
    }
    if candidate.zone != actor_zone {
        return Some(DisengageReason::OutOfZone);
    }
    if distance(actor_position, candidate.position) > AGGRO_PURSUIT_RANGE {
        return Some(DisengageReason::OutOfRange);
    }
    None
}

fn valid_position(position: [f32; 3]) -> bool {
    position.iter().all(|coord| coord.is_finite())
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Horizontal direction from the actor toward its target, matching the recovered opcode 125
/// rotation (X/Z mask; `atan2(X, Z)`) that the registered encounter loop already emits.
fn horizontal_heading(from: [f32; 3], to: [f32; 3], delta: f32) -> [f32; 3] {
    [(to[0] - from[0]) / delta, 0.0, (to[2] - from[2]) / delta]
}
