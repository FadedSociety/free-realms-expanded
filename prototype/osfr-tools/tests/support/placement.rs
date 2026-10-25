//! Position helpers shared by the NPC placement suites (T696 and later).
//!
//! Test-only; production reads positions through `world.rs`.
#![allow(dead_code)]

use serde_json::Value;

/// A content row's `position` (or any `[x, y, z]` array) as `f32`s.
pub fn xyz(value: &Value) -> [f32; 3] {
    serde_json::from_value(value.clone()).expect("a finite xyz position")
}

/// Straight-line distance in metres, computed in `f64` so a 1 m rule has no `f32` slack.
pub fn distance(left: [f32; 3], right: [f32; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2))
        .sum::<f64>()
        .sqrt()
}
