//! T455: server-owned pet following. Pure: no I/O, no clock, no terrain access of its own.
//!
//! The client has no follow logic; it only reports clicks (SendPetFollowLocation `00b5fcf0`,
//! notes/Build Guides/Gameplay/Pets.md), so the server moves the pet and sends opcode 125 the
//! way it moves enemies. Every number is a reconstructed (R) default from the `pets.json`
//! `follow` block; the behaviour (follow behind, settle beside a still owner, warp on a
//! teleport) is the one OSFR PR #152 shows, read for behaviour only.

use serde::{Deserialize, Serialize};

/// Below this the owner did not move between two updates (m); position jitter, not walking.
const STILL_EPS_M: f32 = 0.05;
/// The pet has reached its spot (m).
const ARRIVE_M: f32 = 0.1;

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
            && (50..=10_000).contains(&self.send_ms)
            && self.catch_up_start_m > self.behind_m
            && self.warp_far_m > self.catch_up_start_m
    }
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
    }
}

/// Advance the pet one owner update or tick. `paused` is the 53/24 pet-window mode (not 0):
/// the pet stops and faces its owner. A non-finite owner pose or ground value skips the step
/// and keeps the last pose.
pub fn step(
    config: &FollowConfig,
    owner: OwnerPose,
    pet: PetPose,
    ground: impl Fn(f32, f32) -> Option<f32>,
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
        p.heading = face;
        return (p, fresh.then(|| frame(MoveKind::Stop, &p)));
    }
    p.paused = false;

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

    let to_target = sub(target, p.position);
    let remaining = (to_target[0] * to_target[0] + to_target[1] * to_target[1]).sqrt();
    let reach = speed * dt;
    let (x, z, arrived) = if remaining <= ARRIVE_M.max(reach) {
        (target[0], target[2], true)
    } else {
        let d = [to_target[0] / remaining, to_target[1] / remaining];
        p.heading = d;
        (
            p.position[0] + d[0] * reach,
            p.position[2] + d[1] * reach,
            false,
        )
    };
    let y = match ground(x, z) {
        Some(y) if !y.is_finite() => return (pet, None),
        Some(y) if (y - p.position[1]).abs() > config.warp_climb_m && p.last_sent_ms.is_some() => {
            return warp(config, p, owner, &ground);
        }
        Some(y) => y,
        // A zone without a terrain grid keeps the owner's height (R).
        None => owner.position[1],
    };
    p.position = [x, y, z];

    if arrived && settling {
        p.settled = true;
        p.moving = false;
        p.heading = unit(sub(owner.position, p.position)).unwrap_or(p.heading);
        let stop = sent(&mut p, now, MoveKind::Stop);
        return (p, Some(stop));
    }
    p.moving = remaining > ARRIVE_M;
    let due = p
        .last_sent_ms
        .is_none_or(|sent| now.saturating_sub(sent) >= config.send_ms);
    if p.moving && due {
        let walk = sent(&mut p, now, MoveKind::Walk);
        return (p, Some(walk));
    }
    (p, None)
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
