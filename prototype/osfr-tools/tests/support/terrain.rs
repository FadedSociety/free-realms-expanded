//! Shared T103/T282 placement sample: the 5 m authoring disc and its 1 m slope footprint.
//!
//! Test files include this module with `#[path = "support/terrain.rs"] mod terrain;`.
//! Production never reads it. The caller keeps its own rule assertions (the 25 degree cap,
//! the distance to the nearest actor); this module owns only the sampling so a change to the
//! authoring rule cannot go stale in one placement suite and pass there.
#![allow(dead_code)]

use osfr_tools::terrain::{HeightGrid, Terrain};
use std::path::Path;

/// The shipped zone-0/9600 terrain grids every placement suite samples.
pub fn load() -> Terrain {
    Terrain::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/terrain"))
        .expect("the shipped zone-0/9600 terrain grids load")
}

/// 81 one-metre disc points plus a 24-point 5 m ring (105 samples): the T103 authoring sample.
pub fn disc_points(x: f32, z: f32) -> Vec<(f32, f32)> {
    let mut points = Vec::new();
    for dx in -5..=5 {
        for dz in -5..=5 {
            let (dx, dz) = (dx as f32, dz as f32);
            if dx * dx + dz * dz <= 25.0 {
                points.push((x + dx, z + dz));
            }
        }
    }
    for angle in (0..360).step_by(15) {
        let radians = (angle as f32).to_radians();
        points.push((x + 5.0 * radians.cos(), z + 5.0 * radians.sin()));
    }
    points
}

/// The bilinear 1 m slope footprint around one sample. A missing grid point fails the
/// caller's placement assertion, so this panics rather than returning an option.
pub fn slope_degrees(grid: &HeightGrid, x: f32, z: f32) -> f32 {
    let [west, east, south, north] = [
        grid.height_at(x - 1.0, z),
        grid.height_at(x + 1.0, z),
        grid.height_at(x, z - 1.0),
        grid.height_at(x, z + 1.0),
    ]
    .map(|height| height.expect("terrain around a sampled point is covered"));
    let dx = (east - west) / 2.0;
    let dz = (north - south) / 2.0;
    dx.hypot(dz).atan().to_degrees()
}
