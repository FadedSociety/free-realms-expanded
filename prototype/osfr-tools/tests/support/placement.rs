//! Position helpers shared by the NPC placement suites (T696 and later).
//!
//! Test-only; production reads positions through `world.rs`.
#![allow(dead_code)]

use osfr_tools::world::NpcVisibility;
use serde_json::Value;

/// A content row's `position` (or any `[x, y, z]` array) as `f32`s.
pub fn xyz(value: &Value) -> [f32; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

/// The authored `f64` position, for comparing a row with its cited source line
/// before the loader narrows it to `f32`. Panics when the value is not three
/// finite numbers.
pub fn xyz_f64(value: &Value) -> [f64; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

/// One NPC body the loader draws in the open world: its actor id, the regional row it is
/// bound to (`None` for scene props and enemies) and where it stands.
pub struct Drawn {
    pub actor: u64,
    pub content_key: Option<u64>,
    pub position: [f32; 3],
}

/// Every body `NpcVisibility::load` draws, ordered by actor id. `teleport_targets` backs the
/// same rows as the 35/2 AddNpc stream (placed, extras and Studio placements, minus the
/// loader's skips), so a placement scan reads this and never re-filters the content files.
pub fn drawn_npcs(visibility: &NpcVisibility) -> Vec<Drawn> {
    let mut drawn: Vec<Drawn> = visibility
        .teleport_targets(0) // the open-world label `world.rs` files every spawn under (R)
        .into_iter()
        .map(|target| Drawn {
            actor: target.guid,
            content_key: visibility.content_key(target.guid),
            position: target.position,
        })
        .collect();
    drawn.sort_by_key(|npc| npc.actor);
    drawn
}

/// Straight-line distance in metres, computed in `f64` so a 1 m rule has no `f32` slack.
pub fn distance(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
        .sum::<f64>()
        .sqrt()
}
