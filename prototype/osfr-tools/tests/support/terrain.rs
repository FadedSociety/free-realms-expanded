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

/// A synthetic `cell`-metre heightfield over `min..max`: each cell is two triangles whose
/// corner heights come from `height`. A suite that needs a flat plane, a step or a cliff builds
/// it here instead of copying the raster loop.
pub fn surface(
    min: [f32; 2],
    max: [f32; 2],
    cell: f32,
    height: impl Fn(f32, f32) -> f32,
) -> HeightGrid {
    let mut grid = HeightGrid::new(min, max, cell).expect("a valid synthetic grid");
    let cols = ((max[0] - min[0]) / cell).ceil() as i64;
    let rows = ((max[1] - min[1]) / cell).ceil() as i64;
    for col in 0..cols {
        for row in 0..rows {
            let (x0, z0) = (min[0] + col as f32 * cell, min[1] + row as f32 * cell);
            let (x1, z1) = (x0 + cell, z0 + cell);
            let h00 = height(x0, z0);
            let h10 = height(x1, z0);
            let h11 = height(x1, z1);
            let h01 = height(x0, z1);
            grid.add_triangle([[x0, h00, z0], [x1, h10, z0], [x1, h11, z1]]);
            grid.add_triangle([[x0, h00, z0], [x1, h11, z1], [x0, h01, z1]]);
        }
    }
    grid
}
