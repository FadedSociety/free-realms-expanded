//! Where a challenge's things go around its giver.
//!
//! Adapted from the Sanctuary community server fork (BubbaJoeX),
//! `Sanctuary.Game/Challenges/ChallengeLayout.cs`, AGPL-3.0 (credited in NOTICE; the fork's
//! scatter, race-loop and nearest-point rules, rewritten for this crate's heightmap instead of a
//! navmesh and for a seeded generator).

use super::Tuning;
use std::collections::{HashMap, VecDeque};

/// How many cells the walk goes out each way, whatever the content asks.
const MAX_WALK_CELLS: i32 = 256;

/// SplitMix64: a layout only needs spread, not secrecy, and a seed keeps a run reproducible.
#[derive(Clone, Debug)]
pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A float in `[0, 1)`.
    pub(super) fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// The ground under a spot: `None` is a hole in a zone that has a height grid, so no point is
/// planted there.
pub(super) type Ground<'a> = &'a dyn Fn(f32, f32) -> Option<f32>;
/// Whether a placed prop stands between two points on the ground.
pub(super) type Blocked<'a> = &'a dyn Fn([f32; 3], [f32; 3]) -> bool;
/// The floor of a placed prop (a deck, a cave's floor) at (x, z) within a step of height `from`.
pub(super) type PropFloor<'a> = &'a dyn Fn(f32, f32, f32) -> Option<f32>;

/// Everything a point sampler needs besides the spot.
pub(super) struct Field<'a> {
    pub(super) center: [f32; 3],
    pub(super) ground: Ground<'a>,
    /// The zone has no height grid: every spot stands at the giver's height.
    pub(super) flat: bool,
    pub(super) tuning: &'a Tuning,
    /// The cells a player can walk to from the giver, with the floor height there. Empty on a
    /// flat zone, or where the walk could not leave the giver.
    walkable: HashMap<(i32, i32), f32>,
    blocked: Blocked<'a>,
    prop_floor: PropFloor<'a>,
}

impl<'a> Field<'a> {
    /// `radius` is the widest ring any spot may be asked of.
    pub(super) fn new(
        center: [f32; 3],
        ground: Ground<'a>,
        blocked: Blocked<'a>,
        prop_floor: PropFloor<'a>,
        flat: bool,
        tuning: &'a Tuning,
        radius: f32,
    ) -> Self {
        // A giver with no floor near it (the grid here is the hillside over its cave, and no
        // mesh has the cave's floor) has nothing to walk on.
        let flat = flat
            || floor_at(ground, prop_floor, center[0], center[2], center[1])
                .is_none_or(|height| (height - center[1]).abs() > tuning.max_rise);
        let mut walkable = if flat {
            HashMap::new()
        } else {
            walk_out(center, ground, blocked, prop_floor, tuning, radius * 1.25)
        };
        // Boxed in by props or a rough patch: a game with odd spots beats one that cannot start.
        let least = tuning.min_center_distance / tuning.walk_cell;
        if !(walkable.keys()).any(|(col, row)| (*col as f32).hypot(*row as f32) >= least) {
            walkable.clear();
        }
        Self {
            center,
            ground,
            flat,
            tuning,
            walkable,
            blocked,
            prop_floor,
        }
    }

    /// A random walkable-looking point within `radius` of `spot`. A spot whose ground is a hole or
    /// differs from the giver's by more than `max_rise` (a cliff, a valley) is refused.
    fn sample(&self, spot: [f32; 2], radius: f32, rng: &mut Rng) -> Option<[f32; 3]> {
        let angle = rng.unit() * std::f32::consts::TAU;
        let distance = radius * rng.unit().sqrt();
        let (x, z) = (
            spot[0] + angle.sin() * distance,
            spot[1] + angle.cos() * distance,
        );
        let y = if self.flat {
            self.center[1]
        } else {
            let walked = self.walkable.get(&cell_of(self.center, self.tuning, x, z));
            if walked.is_none() && !self.walkable.is_empty() {
                return None;
            }
            let from = walked.copied().unwrap_or(self.center[1]);
            let height = floor_at(self.ground, self.prop_floor, x, z, from)?;
            if (height - self.center[1]).abs() > self.tuning.max_rise
                || self.inside_a_prop([x, height, z])
            {
                return None;
            }
            height
        };
        Some([x, y, z])
    }

    #[cfg(test)]
    pub(super) fn walkable_is_filtering(&self) -> bool {
        !self.walkable.is_empty()
    }

    /// Whether a prop (a tree's trunk, a wall) stands right at `spot`.
    fn inside_a_prop(&self, spot: [f32; 3]) -> bool {
        const REACH: f32 = 1.5;
        [(REACH, 0.0), (-REACH, 0.0), (0.0, REACH), (0.0, -REACH)]
            .iter()
            .any(|(dx, dz)| (self.blocked)(spot, [spot[0] + dx, spot[1], spot[2] + dz]))
    }
}

/// What a player walking at height `from` stands on at (x, z): a prop's floor within a step, or
/// the height grid.
fn floor_at(ground: Ground, prop_floor: PropFloor, x: f32, z: f32, from: f32) -> Option<f32> {
    prop_floor(x, z, from).or_else(|| ground(x, z))
}

fn cell_of(center: [f32; 3], tuning: &Tuning, x: f32, z: f32) -> (i32, i32) {
    (
        ((x - center[0]) / tuning.walk_cell).round() as i32,
        ((z - center[2]) / tuning.walk_cell).round() as i32,
    )
}

/// Flood out from the giver over the height grid, one cell at a time, stopping at any step too
/// steep to walk (a rock's side, a wall). Height alone would plant things on top of rocks.
// ponytail: the grid has no navmesh or jump; a climb reachable only by jumping counts as blocked.
fn walk_out(
    center: [f32; 3],
    ground: Ground,
    blocked: Blocked,
    prop_floor: PropFloor,
    tuning: &Tuning,
    radius: f32,
) -> HashMap<(i32, i32), f32> {
    let limit = ((radius / tuning.walk_cell).ceil() as i32).min(MAX_WALK_CELLS);
    let start: (i32, i32) = (0, 0);
    let first = floor_at(ground, prop_floor, center[0], center[2], center[1]).unwrap_or(center[1]);
    let mut seen = HashMap::from([(start, first)]);
    let mut queue = VecDeque::from([(start, first)]);
    while let Some(((col, row), height)) = queue.pop_front() {
        for (dc, dr) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next = (col + dc, row + dr);
            if next.0.abs() > limit || next.1.abs() > limit || seen.contains_key(&next) {
                continue;
            }
            let (x, z) = (
                center[0] + next.0 as f32 * tuning.walk_cell,
                center[2] + next.1 as f32 * tuning.walk_cell,
            );
            let here_at = [
                center[0] + col as f32 * tuning.walk_cell,
                height,
                center[2] + row as f32 * tuning.walk_cell,
            ];
            if let Some(there) = floor_at(ground, prop_floor, x, z, height)
                && (there - height).abs() <= tuning.walk_step
                && !blocked(here_at, [x, there, z])
            {
                seen.insert(next, there);
                queue.push_back((next, there));
            }
        }
    }
    seen
}

pub(super) fn flat_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

/// Up to `count` points scattered within `radius` of the giver, apart from each other and from it.
pub(super) fn scatter(field: &Field, radius: f32, count: u32, rng: &mut Rng) -> Vec<[f32; 3]> {
    let tuning = field.tuning;
    let mut points: Vec<[f32; 3]> = Vec::new();
    let attempts = count.saturating_mul(tuning.attempts_per_point);
    for _ in 0..attempts {
        if points.len() >= count as usize {
            break;
        }
        let Some(point) = field.sample([field.center[0], field.center[2]], radius, rng) else {
            continue;
        };
        if flat_distance(point, field.center) < tuning.min_center_distance
            || points
                .iter()
                .any(|other| flat_distance(*other, point) < tuning.min_spacing)
        {
            continue;
        }
        points.push(point);
    }
    points
}

/// A race's flags: a loop out round the giver and back to it, the last one by the giver.
pub(super) fn race_loop(field: &Field, radius: f32, count: u32, rng: &mut Rng) -> Vec<[f32; 3]> {
    let tuning = field.tuning;
    let start = rng.unit() * std::f32::consts::TAU;
    let legs = count.saturating_sub(1);
    let mut flags = Vec::new();
    for i in 0..legs {
        let angle = start + std::f32::consts::TAU * i as f32 / legs as f32;
        // Out to the edge and back, so the loop is not a plain circle.
        let distance =
            radius * (0.6 + 0.4 * (std::f32::consts::PI * (i + 1) as f32 / count as f32).sin());
        let spot = [
            field.center[0] + angle.sin() * distance,
            field.center[2] + angle.cos() * distance,
        ];
        // Rough ground refuses most tries, so a flag gets as many as a scattered thing.
        let flag = (0..tuning.attempts_per_point).find_map(|_| {
            field
                .sample(spot, radius * 0.3, rng)
                .filter(|flag| flat_distance(*flag, field.center) >= tuning.min_center_distance)
        });
        flags.extend(flag);
    }
    // The finish, beside the giver.
    let finish = [
        field.center[0],
        field.center[2] + tuning.min_center_distance,
    ];
    flags.extend((0..tuning.attempts_per_point).find_map(|_| field.sample(finish, 4.0, rng)));
    flags
}

/// The index of the point nearest to `from` that `open` keeps, or `None`.
pub(super) fn nearest(
    from: [f32; 3],
    points: &[[f32; 3]],
    open: impl Fn(usize) -> bool,
) -> Option<usize> {
    (0..points.len())
        .filter(|i| open(*i))
        .min_by(|a, b| flat_distance(from, points[*a]).total_cmp(&flat_distance(from, points[*b])))
}
