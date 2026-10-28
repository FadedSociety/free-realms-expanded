//! T455: server-owned pet following. Pure: no I/O, no clock, no terrain access of its own.
//!
//! The client has no follow logic; it only reports clicks (SendPetFollowLocation `00b5fcf0`,
//! notes/Build Guides/Gameplay/Pets.md), so the server moves the pet and sends opcode 125 the
//! way it moves enemies. Every number is a reconstructed (R) default from the `pets.json`
//! `follow` block; the behaviour (follow behind, settle beside a still owner, warp on a
//! teleport) is the one OSFR PR #152 shows, read for behaviour only.
//!
//! T490: a click walks the pet to that spot (53/26), and every step of a path is checked
//! against the heightfield first: a rise over `step_up_m`, a drop over `step_down_m` or
//! unknown ground refuses the step, and a pet that makes no progress for `warp_far_ms` warps
//! to its owner.

use serde::{Deserialize, Serialize};

/// Below this the owner did not move between two updates (m); position jitter, not walking.
const STILL_EPS_M: f32 = 0.05;
/// The pet has reached its spot (m).
const ARRIVE_M: f32 = 0.1;
/// (R) A safety bound on one walk check. Callers cap a click path with `click_max_m` (40 m at
/// the shipped values); this only stops a bad caller from sampling a whole world.
const MAX_WALK_SAMPLES: usize = 4_096;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FollowConfig {
    pub side_m: f32,
    pub behind_m: f32,
    pub min_speed: f32,
    pub max_speed: f32,
    pub catch_up_start_m: f32,
    pub catch_up_gain: f32,
    pub max_speed_multiplier: f32,
    pub settle_delay_ms: u64,
    pub settle_radius_m: f32,
    pub settle_min_deg: f32,
    pub settle_max_deg: f32,
    pub warp_owner_jump_m: f32,
    pub warp_far_m: f32,
    pub warp_far_ms: u64,
    pub warp_climb_m: f32,
    /// F833 (R): how far a follow click may land from its owner before it is refused. The
    /// client's own sender `00b5fcf0` reports only a target 8-40 m from the point it tracks, so
    /// 40 m admits every real click; reachability itself is checked by `walkable` (T490).
    pub click_max_m: f32,
    /// T490: the client's own minimum click distance from the owner (`DAT_0176b1b0` = 8.0 m in
    /// `00b5fcf0`); a nearer click answers 0.
    pub min_click_m: f32,
    /// T490: the client's own minimum displacement between two sent clicks (`DAT_01b5d878` =
    /// 10.0 m in `00b5fcf0`, gate A); a click nearer the last accepted one answers 0.
    pub min_click_step_m: f32,
    /// T490: the most the ground may rise between two walk samples (1 m, as OSFR PR #152
    /// steps up; the value is (R)).
    pub step_up_m: f32,
    /// T490: the most the ground may fall between two walk samples (3 m, the same reference;
    /// (R)).
    pub step_down_m: f32,
    /// T490: the distance between walk samples (the same reference samples every 0.5 m; (R)).
    pub sample_m: f32,
    /// T490: the client sends a click at most every 200 ms (`DAT_01b5d874` = 0xC8 against
    /// `timeGetTime` milliseconds, sub_7027F0 / sub_8B0E20); the server answers one sooner 0.
    pub min_click_interval_ms: u64,
    /// T490: how far the owner may move before a waiting pet resumes following (R).
    pub resume_m: f32,
    /// T490: how long the pet waits at a clicked spot after reaching it (R).
    pub go_timeout_ms: u64,
    pub send_ms: u64,
}

impl FollowConfig {
    /// Sanity bounds checked at content load, so a bad file fails there, never mid-session.
    pub fn valid(&self) -> bool {
        let metres = [
            self.side_m,
            self.behind_m,
            self.catch_up_start_m,
            self.settle_radius_m,
            self.warp_owner_jump_m,
            self.warp_far_m,
            self.warp_climb_m,
        ];
        metres
            .iter()
            .all(|m| m.is_finite() && (0.1..=500.0).contains(m))
            && self.min_speed.is_finite()
            && self.max_speed.is_finite()
            && 0.1 <= self.min_speed
            && self.min_speed <= self.max_speed
            && self.max_speed <= 50.0
            && self.catch_up_gain.is_finite()
            && (0.0..=10.0).contains(&self.catch_up_gain)
            && (1.0..=5.0).contains(&self.max_speed_multiplier)
            && 0.0 <= self.settle_min_deg
            && self.settle_min_deg < self.settle_max_deg
            && self.settle_max_deg <= 180.0
            && self.settle_delay_ms <= 60_000
            && (1..=60_000).contains(&self.warp_far_ms)
            && (1.0..=100.0).contains(&self.click_max_m)
            && self.click_max_m <= self.warp_far_m
            && self.min_click_m.is_finite()
            && 0.5 <= self.min_click_m
            && self.min_click_m < self.click_max_m
            && self.min_click_step_m.is_finite()
            && 1.0 <= self.min_click_step_m
            && self.min_click_step_m <= self.click_max_m
            && self.step_up_m.is_finite()
            && (0.1..=10.0).contains(&self.step_up_m)
            && self.step_down_m.is_finite()
            && (0.1..=20.0).contains(&self.step_down_m)
            && self.sample_m.is_finite()
            && (0.1..=5.0).contains(&self.sample_m)
            && self.min_click_interval_ms <= 60_000
            && self.resume_m.is_finite()
            && (0.5..=50.0).contains(&self.resume_m)
            && (1_000..=600_000).contains(&self.go_timeout_ms)
            && (50..=10_000).contains(&self.send_ms)
            && self.catch_up_start_m > self.behind_m
            && self.warp_far_m > self.catch_up_start_m
    }

    /// F833/T490 (R): a follow click is admitted only in the client's own 8-40 m band from the
    /// owner, so a forged coordinate cannot park the pet on the far side of the world. The
    /// client's sender `00b5fcf0` measures the 4D distance between two engine vectors; the
    /// server reads only the owner's three position components, so the owner's fourth is 0 and
    /// the click's fourth is part of the distance (`00b5fcf0` writes all four). The client's own
    /// sender rejects what this refuses.
    pub fn admits_click(&self, owner: [f32; 3], target: [f32; 4]) -> bool {
        let distance = dist4([owner[0], owner[1], owner[2], 0.0], target);
        (self.min_click_m..=self.click_max_m).contains(&distance)
    }

    /// T490: the client's gate A: a click is taken only once it is `min_click_step_m` from the
    /// click the client sent last (`00b5fcf0`). The server's nearest stand-in for the client's
    /// cache is the last click it accepted (R: a click the client computed but did not send
    /// updates the client's cache and not this one, so the server can be the stricter side).
    pub fn advances_click(&self, last: [f32; 4], target: [f32; 4]) -> bool {
        dist4(last, target) >= self.min_click_step_m
    }
}

/// T490: whether the ground along the straight (x, z) path from `from` to `to` is walkable.
///
/// The path is sampled at fixed `sample_m` intervals and again at the endpoint (review F850:
/// dividing a short step into equal parts put its only inner sample wherever the step happened
/// to end, which missed a cliff between the samples); each sample must exist, be finite, and
/// differ from the previous one by at most `step_up` up and `step_down` down. Unknown ground
/// (`None`) is never walkable, so a zone without a height grid refuses every path. Heights come
/// only from `ground`, so a client-supplied y can never decide the check. Cost is the number of
/// samples (`length / sample_m`), bounded by `MAX_WALK_SAMPLES` and by the caller's
/// `click_max_m`.
pub fn walkable(
    ground: impl Fn(f32, f32) -> Option<f32>,
    from: [f32; 3],
    to: [f32; 3],
    step_up: f32,
    step_down: f32,
    sample_m: f32,
) -> bool {
    if !from.iter().all(|value| value.is_finite())
        || !to.iter().all(|value| value.is_finite())
        || !step_up.is_finite()
        || !step_down.is_finite()
        || !(sample_m > 0.0)
    {
        return false;
    }
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let length = (dx * dx + dz * dz).sqrt();
    if !length.is_finite() {
        return false;
    }
    let samples = (length / sample_m).ceil();
    if samples > MAX_WALK_SAMPLES as f32 {
        return false;
    }
    let Some(mut previous) = ground(from[0], from[2]).filter(|y| y.is_finite()) else {
        return false;
    };
    let mut distance = sample_m;
    while distance < length {
        let t = distance / length;
        let Some(y) = ground(from[0] + dx * t, from[2] + dz * t).filter(|y| y.is_finite()) else {
            return false;
        };
        if y - previous > step_up || previous - y > step_down {
            return false;
        }
        previous = y;
        distance += sample_m;
    }
    let Some(y) = ground(to[0], to[2]).filter(|y| y.is_finite()) else {
        return false;
    };
    y - previous <= step_up && previous - y <= step_down
}

/// The owner as the server last accepted it: position and the time of that update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OwnerPose {
    pub position: [f32; 3],
    pub at_ms: u64,
}

/// What one step asks the wire to send for the pet actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MoveKind {
    Walk,
    Stop,
    Warp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Move {
    pub kind: MoveKind,
    pub position: [f32; 3],
    pub heading: [f32; 3],
}

/// T490: a clicked spot the pet walks to and then waits at (session only, like the pose).
#[derive(Clone, Copy, Debug, PartialEq)]
struct GoTarget {
    target: [f32; 3],
    /// The owner's position when the click was accepted; moving `resume_m` from it ends the go.
    owner: [f32; 3],
    /// When the pet reached the spot; the wait timeout counts from here, not from the click,
    /// so a long walk never expires before it arrives.
    arrived_ms: Option<u64>,
}

/// Session-only pet pose: never saved and never part of the roster.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PetPose {
    pub position: [f32; 3],
    /// Unit (x, z) facing.
    pub heading: [f32; 2],
    last_owner: Option<OwnerPose>,
    owner_facing: [f32; 2],
    owner_speed: f32,
    still_since_ms: Option<u64>,
    far_since_ms: Option<u64>,
    last_sent_ms: Option<u64>,
    catching_up: bool,
    moving: bool,
    settled: bool,
    paused: bool,
    /// T490: the clicked destination the pet walks to or waits at.
    go: Option<GoTarget>,
    /// T490: when a walk step was last refused, for the `warp_far_ms` no-progress warp.
    blocked_since_ms: Option<u64>,
}

impl PetPose {
    /// The pet has walked to its spot beside a still owner and stands there (T531: its idle life
    /// runs only while this holds; a walk, a warp or the pet window clears it). T490: a pet
    /// waiting at a clicked spot counts as settled too.
    pub fn settled(&self) -> bool {
        self.settled
    }

    /// The owner's last accepted position, which bounds a follow click; before the first owner
    /// update it is the pet's spawn, already beside the owner.
    pub fn owner_position(&self) -> [f32; 3] {
        self.last_owner
            .map_or(self.position, |owner| owner.position)
    }

    /// T490: the spot a 53/26 click sent the pet to, while it walks there or waits; `None` once
    /// following resumes. Session-only, like the pose.
    pub fn go_target(&self) -> Option<[f32; 3]> {
        self.go.map(|go| go.target)
    }
}

/// A fresh pose `side_m` to the owner's side (+x until a walk shows the owner's facing (R)).
pub fn spawn(config: &FollowConfig, owner: [f32; 3]) -> PetPose {
    PetPose {
        position: [owner[0] + config.side_m, owner[1], owner[2]],
        ..spawn_inner(owner)
    }
}

/// A pose at the owner's own position, for content without a follow block (the pet stands).
pub fn spawn_at(owner: [f32; 3]) -> PetPose {
    PetPose {
        position: owner,
        ..spawn_inner(owner)
    }
}

fn spawn_inner(owner: [f32; 3]) -> PetPose {
    PetPose {
        position: owner,
        heading: [-1.0, 0.0],
        last_owner: None,
        owner_facing: [0.0, 1.0],
        owner_speed: 0.0,
        still_since_ms: None,
        far_since_ms: None,
        last_sent_ms: None,
        catching_up: false,
        moving: false,
        settled: false,
        paused: false,
        go: None,
        blocked_since_ms: None,
    }
}

/// T490: the pet was sent to `target` (a 53/26 the server accepted). The caller has already
/// checked the click's bounds and walkability; this only records the destination, and the next
/// `step` walks there.
pub fn go_to(pet: PetPose, target: [f32; 3], owner: [f32; 3]) -> PetPose {
    PetPose {
        go: Some(GoTarget {
            target,
            owner,
            arrived_ms: None,
        }),
        // A new accepted click starts a new goal: a no-progress clock from the old walk must not
        // warp the pet before it takes its first step toward this one.
        blocked_since_ms: None,
        ..pet
    }
}

/// Advance the pet one owner update or tick. `paused` is the 53/24 pet-window mode (not 0):
/// the pet stops and faces its owner. `gridded` is whether the zone has a height grid at all:
/// with one, unknown ground is a hole and refuses the step; without one the step keeps T455's
/// owner-height rule. A non-finite owner pose or ground value skips the step and keeps the last
/// pose.
pub fn step(
    config: &FollowConfig,
    owner: OwnerPose,
    pet: PetPose,
    ground: impl Fn(f32, f32) -> Option<f32>,
    gridded: bool,
    paused: bool,
) -> (PetPose, Option<Move>) {
    if !owner.position.iter().all(|v| v.is_finite()) {
        return (pet, None);
    }
    let mut p = pet;
    let now = owner.at_ms;
    let dt = p
        .last_owner
        .map_or(0.0, |last| now.saturating_sub(last.at_ms) as f32 / 1000.0);
    if let Some(last) = p.last_owner {
        let moved = dist(last.position, owner.position);
        if moved > config.warp_owner_jump_m {
            p.last_owner = Some(owner);
            return warp(config, p, owner, &ground);
        }
        if moved > STILL_EPS_M {
            p.owner_facing = unit(sub(owner.position, last.position)).unwrap_or(p.owner_facing);
            if dt > 0.0 {
                p.owner_speed = moved / dt;
            }
            p.still_since_ms = None;
        } else if p.still_since_ms.is_none() {
            p.still_since_ms = Some(now);
        }
    } else {
        p.still_since_ms = Some(now);
    }
    p.last_owner = Some(owner);

    let gap = dist(p.position, owner.position);
    if gap > config.warp_far_m {
        let since = *p.far_since_ms.get_or_insert(now);
        if now.saturating_sub(since) >= config.warp_far_ms {
            return warp(config, p, owner, &ground);
        }
    } else {
        p.far_since_ms = None;
    }

    if paused {
        let face = unit(sub(owner.position, p.position)).unwrap_or(p.heading);
        let fresh = !p.paused || p.moving;
        p.paused = true;
        p.moving = false;
        p.settled = false;
        p.blocked_since_ms = None;
        p.heading = face;
        return (p, fresh.then(|| frame(MoveKind::Stop, &p)));
    }
    p.paused = false;

    // T490: a clicked destination owns the pet until it arrives and its wait ends.
    if let Some(go) = p.go {
        let resumed = dist(go.owner, owner.position) > config.resume_m;
        let waited = go
            .arrived_ms
            .is_some_and(|at| now.saturating_sub(at) >= config.go_timeout_ms);
        if resumed || waited {
            p.go = None;
            p.blocked_since_ms = None;
            // The clicked spot's wait is over; the follow rules decide where the pet goes next.
            p.settled = false;
        } else if go.arrived_ms.is_some() {
            return (p, None);
        } else {
            let pace = config.min_speed;
            return match advance(
                config, &mut p, owner, &ground, gridded, go.target, pace, now, dt,
            ) {
                Advance::Bad => (pet, None),
                Advance::Blocked => (p, None),
                Advance::Warp => warp(config, p, owner, &ground),
                Advance::Moved { arrived, moving } => {
                    if arrived {
                        p.go = Some(GoTarget {
                            arrived_ms: Some(now),
                            ..go
                        });
                        p.settled = true;
                        p.moving = false;
                        p.heading = unit(sub(owner.position, p.position)).unwrap_or(p.heading);
                        let stop = sent(&mut p, now, MoveKind::Stop);
                        return (p, Some(stop));
                    }
                    p.settled = false;
                    p.moving = moving;
                    let walk = walk_if_due(config, &mut p, now);
                    (p, walk)
                }
            };
        }
    }

    let side = side_of(p.owner_facing, owner.position, p.position);
    let settling = p
        .still_since_ms
        .is_some_and(|since| now.saturating_sub(since) >= config.settle_delay_ms);
    if settling && p.settled {
        return (p, None);
    }
    let target = if settling {
        settle_spot(config, p.owner_facing, owner.position, p.position, side)
    } else {
        p.settled = false;
        add(
            owner.position,
            scale(p.owner_facing, -config.behind_m),
            scale(right(p.owner_facing), side * config.side_m),
        )
    };
    let mut speed = p.owner_speed.clamp(config.min_speed, config.max_speed);
    if gap > config.catch_up_start_m {
        p.catching_up = true;
    } else if gap <= config.behind_m + config.side_m {
        p.catching_up = false;
    }
    if p.catching_up {
        let boosted = speed + config.catch_up_gain * (gap - config.behind_m).max(0.0);
        speed = boosted.min(speed * config.max_speed_multiplier);
    }

    match advance(
        config, &mut p, owner, &ground, gridded, target, speed, now, dt,
    ) {
        Advance::Bad => (pet, None),
        Advance::Blocked => (p, None),
        Advance::Warp => warp(config, p, owner, &ground),
        Advance::Moved { arrived, moving } => {
            if arrived && settling {
                p.settled = true;
                p.moving = false;
                p.heading = unit(sub(owner.position, p.position)).unwrap_or(p.heading);
                let stop = sent(&mut p, now, MoveKind::Stop);
                return (p, Some(stop));
            }
            p.moving = moving;
            let walk = walk_if_due(config, &mut p, now);
            (p, walk)
        }
    }
}

/// What one `advance` did, so the follow and clicked-destination steps share every rule.
enum Advance {
    /// A bad ground value: the caller keeps the pose it had before the step.
    Bad,
    /// The heightfield refused the walk: the pet stays and the no-progress timer runs.
    Blocked,
    /// The climb limit or the no-progress timeout puts the pet beside its owner.
    Warp,
    Moved {
        arrived: bool,
        /// The distance was over the arrival radius before the step (a snap reports a walk).
        moving: bool,
    },
}

/// One movement toward `target` at `speed` m/s: the single place the ground height, the 6 m
/// climb limit and the T490 walk check are applied.
fn advance(
    config: &FollowConfig,
    p: &mut PetPose,
    owner: OwnerPose,
    ground: &impl Fn(f32, f32) -> Option<f32>,
    gridded: bool,
    target: [f32; 3],
    speed: f32,
    now: u64,
    dt: f32,
) -> Advance {
    let from = p.position;
    let to_target = sub(target, from);
    let remaining = (to_target[0] * to_target[0] + to_target[1] * to_target[1]).sqrt();
    let reach = speed * dt;
    let (x, z, arrived, heading) = if remaining <= ARRIVE_M.max(reach) {
        (target[0], target[2], true, None)
    } else {
        let d = [to_target[0] / remaining, to_target[1] / remaining];
        (
            from[0] + d[0] * reach,
            from[2] + d[1] * reach,
            false,
            Some(d),
        )
    };
    let stepped = ground(x, z);
    let y = match stepped {
        Some(y) if !y.is_finite() => return Advance::Bad,
        Some(y) if (y - p.position[1]).abs() > config.warp_climb_m && p.last_sent_ms.is_some() => {
            return Advance::Warp;
        }
        Some(y) => y,
        // A zone without a terrain grid keeps the owner's height (R).
        None => owner.position[1],
    };
    // T490 (review F850): check the ground at least one `sample_m` ahead along the next step's
    // direction (or up to the target), not only as far as this tick moves. A 20 Hz follow step
    // is 0.1-0.6 m and a heightfield smears a cliff across one cell, so a step-sized check would
    // creep down it one interpolated cell at a time.
    let (check_x, check_z) = heading.map_or((x, z), |d| {
        let look = reach.max(config.sample_m).min(remaining);
        (from[0] + d[0] * look, from[2] + d[1] * look)
    });
    // A gridded zone always checks: both ends unknown there mean a hole (or outside the grid),
    // which `walkable` refuses. Only a zone with no grid at all keeps the owner-height fallback.
    let checked = gridded
        || stepped.is_some_and(|y| y.is_finite())
        || ground(from[0], from[2]).is_some_and(|y| y.is_finite());
    if checked
        && (x != from[0] || z != from[2])
        && !walkable(
            ground,
            from,
            [check_x, y, check_z],
            config.step_up_m,
            config.step_down_m,
            config.sample_m,
        )
    {
        let since = *p.blocked_since_ms.get_or_insert(now);
        if now.saturating_sub(since) >= config.warp_far_ms {
            return Advance::Warp;
        }
        return Advance::Blocked;
    }
    p.blocked_since_ms = None;
    if let Some(heading) = heading {
        p.heading = heading;
    }
    p.position = [x, y, z];
    Advance::Moved {
        arrived,
        moving: remaining > ARRIVE_M,
    }
}

fn walk_if_due(config: &FollowConfig, p: &mut PetPose, now: u64) -> Option<Move> {
    let due = p
        .last_sent_ms
        .is_none_or(|sent| now.saturating_sub(sent) >= config.send_ms);
    (p.moving && due).then(|| sent(p, now, MoveKind::Walk))
}

fn warp(
    config: &FollowConfig,
    mut p: PetPose,
    owner: OwnerPose,
    ground: &impl Fn(f32, f32) -> Option<f32>,
) -> (PetPose, Option<Move>) {
    let spot = add(
        owner.position,
        [0.0, 0.0],
        scale(right(p.owner_facing), config.side_m),
    );
    let y = ground(spot[0], spot[2])
        .filter(|y| y.is_finite())
        .unwrap_or(owner.position[1]);
    p.position = [spot[0], y, spot[2]];
    p.heading = p.owner_facing;
    p.far_since_ms = None;
    p.catching_up = false;
    p.moving = false;
    p.settled = false;
    p.go = None;
    p.blocked_since_ms = None;
    let now = owner.at_ms;
    let warp = sent(&mut p, now, MoveKind::Warp);
    (p, Some(warp))
}

fn sent(p: &mut PetPose, now: u64, kind: MoveKind) -> Move {
    p.last_sent_ms = Some(now);
    frame(kind, p)
}

fn frame(kind: MoveKind, p: &PetPose) -> Move {
    Move {
        kind,
        position: p.position,
        heading: [p.heading[0], 0.0, p.heading[1]],
    }
}

/// The settle spot `settle_radius_m` from the owner, on the pet's side, at the angle off the
/// owner's facing nearest the pet's current bearing within [settle_min_deg, settle_max_deg].
fn settle_spot(
    config: &FollowConfig,
    facing: [f32; 2],
    owner: [f32; 3],
    pet: [f32; 3],
    side: f32,
) -> [f32; 3] {
    let bearing = unit(sub(pet, owner)).map_or(config.settle_min_deg, |b| {
        (facing[0] * b[0] + facing[1] * b[1])
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    });
    let angle = bearing
        .clamp(config.settle_min_deg, config.settle_max_deg)
        .to_radians();
    let r = right(facing);
    let dir = [
        facing[0] * angle.cos() + r[0] * side * angle.sin(),
        facing[1] * angle.cos() + r[1] * side * angle.sin(),
    ];
    add(owner, scale(dir, config.settle_radius_m), [0.0, 0.0])
}

/// +1 when the pet is on the owner's right, -1 on the left (right by default).
fn side_of(facing: [f32; 2], owner: [f32; 3], pet: [f32; 3]) -> f32 {
    let r = right(facing);
    let d = sub(pet, owner);
    if r[0] * d[0] + r[1] * d[1] < 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn right(facing: [f32; 2]) -> [f32; 2] {
    [facing[1], -facing[0]]
}

/// (x, z) of `b - a`.
fn sub(b: [f32; 3], a: [f32; 3]) -> [f32; 2] {
    [b[0] - a[0], b[2] - a[2]]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(b, a);
    (d[0] * d[0] + d[1] * d[1]).sqrt()
}

/// The client's own click distance: four components, like `00b5fcf0` measures them.
fn dist4(a: [f32; 4], b: [f32; 4]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) + (a[3] - b[3]).powi(2))
        .sqrt()
}

fn unit(v: [f32; 2]) -> Option<[f32; 2]> {
    let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (n > 1e-4).then(|| [v[0] / n, v[1] / n])
}

fn scale(v: [f32; 2], s: f32) -> [f32; 2] {
    [v[0] * s, v[1] * s]
}

/// `origin` plus two (x, z) offsets, keeping origin's height.
fn add(origin: [f32; 3], a: [f32; 2], b: [f32; 2]) -> [f32; 3] {
    [origin[0] + a[0] + b[0], origin[1], origin[2] + a[1] + b[1]]
}
