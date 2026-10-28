//! Ground heights from the client's terrain chunks, for developer teleports,
//! NPC placement and enemy ground-following.
//!
//! `osfr-terrain` rasterises every `GCNK` render triangle of a zone into a
//! regular grid (`<terrain dir>/zone_<id>.hgt`); the gateway loads it and
//! answers [`HeightGrid::height_at`] with bilinear interpolation.
//!
//! Evidence (run 2026-10-15): GCNK vertex positions are world coordinates
//! (x, y up, z). Against 2,134 Legacy FabledRealms NPC spawns the identity
//! mapping has a median height error of 0.21 m on a 4 m raster; swapped or
//! negated axes are tens of metres out.
//!
//! (R) The grid alone is terrain only: bridges, buildings and rocks are runtime
//! objects with their own collision, so `height_at` reports the ground under them.
//! T534: `obstacles()` loads the `zone_<id>.obs` files the bake writes beside the
//! grids, so a mover can refuse a wall and stand on a bridge deck.
use crate::obstacles::Obstacles;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const MAGIC: &[u8; 8] = b"OSFRHGT1";
const HEADER: usize = 8 + 4 * 5;
/// Heights are stored in decimetres; this marks a node no triangle covers.
const EMPTY: i16 = i16::MIN;
const SCALE: f32 = 10.0;

#[derive(Clone, Debug, PartialEq)]
pub struct HeightGrid {
    pub min_x: f32,
    pub min_z: f32,
    pub cell: f32,
    pub cols: u32,
    pub rows: u32,
    heights: Vec<i16>,
}

impl HeightGrid {
    /// An empty grid whose nodes cover `[min, max]` every `cell` metres.
    pub fn new(min: [f32; 2], max: [f32; 2], cell: f32) -> Result<Self, String> {
        if !(cell > 0.0) || !(max[0] > min[0]) || !(max[1] > min[1]) {
            return Err("terrain grid needs positive cell and extent".into());
        }
        let cols = ((max[0] - min[0]) / cell).ceil() as u32 + 1;
        let rows = ((max[1] - min[1]) / cell).ceil() as u32 + 1;
        if cols as u64 * rows as u64 > 64 << 20 {
            return Err(format!("terrain grid {cols}x{rows} is too large"));
        }
        Ok(Self {
            min_x: min[0],
            min_z: min[1],
            cell,
            cols,
            rows,
            heights: vec![EMPTY; cols as usize * rows as usize],
        })
    }

    /// Rasterise one world-space triangle; overlapping surfaces keep the highest.
    pub fn add_triangle(&mut self, t: [[f32; 3]; 3]) {
        let [a, b, c] = t;
        let area = (b[0] - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (b[2] - a[2]);
        if area.abs() < 1e-9 || t.iter().flatten().any(|v| !v.is_finite()) {
            return;
        }
        let col = |x: f32| (x - self.min_x) / self.cell;
        let row = |z: f32| (z - self.min_z) / self.cell;
        let c0 = col(a[0].min(b[0]).min(c[0])).ceil().max(0.0) as u32;
        let c1 = (col(a[0].max(b[0]).max(c[0])).floor() as i64).min(self.cols as i64 - 1);
        let r0 = row(a[2].min(b[2]).min(c[2])).ceil().max(0.0) as u32;
        let r1 = (row(a[2].max(b[2]).max(c[2])).floor() as i64).min(self.rows as i64 - 1);
        for r in r0 as i64..=r1 {
            let z = self.min_z + r as f32 * self.cell;
            for cc in c0 as i64..=c1 {
                let x = self.min_x + cc as f32 * self.cell;
                // Barycentric weights; a small epsilon closes seams between triangles.
                let w1 = ((x - a[0]) * (c[2] - a[2]) - (c[0] - a[0]) * (z - a[2])) / area;
                let w2 = ((b[0] - a[0]) * (z - a[2]) - (x - a[0]) * (b[2] - a[2])) / area;
                let w0 = 1.0 - w1 - w2;
                if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                    continue;
                }
                let y = w0 * a[1] + w1 * b[1] + w2 * c[1];
                let stored = (y * SCALE).round().clamp(-32767.0, 32767.0) as i16;
                let slot = &mut self.heights[r as usize * self.cols as usize + cc as usize];
                if *slot == EMPTY || stored > *slot {
                    *slot = stored;
                }
            }
        }
    }

    fn node(&self, col: i64, row: i64) -> Option<f32> {
        if col < 0 || row < 0 || col >= self.cols as i64 || row >= self.rows as i64 {
            return None;
        }
        let value = self.heights[row as usize * self.cols as usize + col as usize];
        (value != EMPTY).then(|| value as f32 / SCALE)
    }

    /// Ground height at (x, z), bilinear between the four surrounding nodes.
    /// Missing nodes (a chunk edge) fall back to the mean of those present.
    pub fn height_at(&self, x: f32, z: f32) -> Option<f32> {
        if !x.is_finite() || !z.is_finite() {
            return None;
        }
        let fx = (x - self.min_x) / self.cell;
        let fz = (z - self.min_z) / self.cell;
        let (c, r) = (fx.floor() as i64, fz.floor() as i64);
        let (tx, tz) = (fx - c as f32, fz - r as f32);
        let corners = [
            (self.node(c, r), (1.0 - tx) * (1.0 - tz)),
            (self.node(c + 1, r), tx * (1.0 - tz)),
            (self.node(c, r + 1), (1.0 - tx) * tz),
            (self.node(c + 1, r + 1), tx * tz),
        ];
        if corners.iter().all(|(h, _)| h.is_some()) {
            return Some(corners.iter().map(|(h, w)| h.unwrap() * w).sum());
        }
        let present: Vec<f32> = corners.iter().filter_map(|(h, _)| *h).collect();
        (!present.is_empty()).then(|| present.iter().sum::<f32>() / present.len() as f32)
    }

    /// Lowest and highest stored height, if any node is covered.
    pub fn height_range(&self) -> Option<(f32, f32)> {
        let mut covered = self.heights.iter().filter(|h| **h != EMPTY);
        let first = *covered.next()?;
        let (lo, hi) = covered.fold((first, first), |(lo, hi), h| (lo.min(*h), hi.max(*h)));
        Some((lo as f32 / SCALE, hi as f32 / SCALE))
    }

    /// Share of nodes with a height.
    pub fn coverage(&self) -> f32 {
        let filled = self.heights.iter().filter(|h| **h != EMPTY).count();
        filled as f32 / self.heights.len().max(1) as f32
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEADER + self.heights.len() * 2);
        out.extend(MAGIC);
        out.extend(self.min_x.to_le_bytes());
        out.extend(self.min_z.to_le_bytes());
        out.extend(self.cell.to_le_bytes());
        out.extend(self.cols.to_le_bytes());
        out.extend(self.rows.to_le_bytes());
        for h in &self.heights {
            out.extend(h.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err("not an OSFRHGT1 height grid".into());
        }
        let f = |at: usize| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let u = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let (cols, rows) = (u(20), u(24));
        let count = cols as usize * rows as usize;
        if bytes.len() != HEADER + count * 2 || !(f(16) > 0.0) {
            return Err("height grid size does not match its header".into());
        }
        Ok(Self {
            min_x: f(8),
            min_z: f(12),
            cell: f(16),
            cols,
            rows,
            heights: bytes[HEADER..]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]))
                .collect(),
        })
    }
}

/// Height grids per zone id, loaded from `zone_<id>.hgt` files.
#[derive(Debug, Default)]
pub struct Terrain {
    dir: Option<PathBuf>,
    zones: HashMap<u32, HeightGrid>,
    /// T534: placed-object collision from the same directory; empty until the bake
    /// writes a zone file, which keeps the heightfield-only behaviour.
    obstacles: Obstacles,
}

impl Terrain {
    /// Load the `.hgt` grids and the `.obs` obstacle files beside them.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut terrain = Self::load_heights(dir)?;
        terrain.obstacles = Obstacles::load(dir);
        Ok(terrain)
    }

    /// Load only the `.hgt` grids. Callers that never query placed-object collision (the
    /// placement suites' shared lab terrain, for example) skip parsing every `.obs` file;
    /// `obstacles()` is then empty, which is the heightfield-only behaviour.
    pub fn load_heights(dir: &Path) -> Result<Self, String> {
        let mut zones = HashMap::new();
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(id) = name
                .strip_prefix("zone_")
                .and_then(|rest| rest.strip_suffix(".hgt"))
                .and_then(|id| id.parse::<u32>().ok())
            else {
                continue;
            };
            let bytes = fs::read(entry.path()).map_err(|e| format!("{name}: {e}"))?;
            zones.insert(
                id,
                HeightGrid::from_bytes(&bytes).map_err(|e| format!("{name}: {e}"))?,
            );
        }
        Ok(Self {
            dir: Some(dir.to_owned()),
            zones,
            obstacles: Obstacles::default(),
        })
    }

    pub fn height_at(&self, zone: u32, x: f32, z: f32) -> Option<f32> {
        self.zones.get(&zone)?.height_at(x, z)
    }

    pub fn grid(&self, zone: u32) -> Option<&HeightGrid> {
        self.zones.get(&zone)
    }

    /// T534: the placed-object meshes beside the grids; a zone without a file is
    /// absent, so callers keep the T490 heightfield-only walk.
    pub fn obstacles(&self) -> &Obstacles {
        &self.obstacles
    }

    /// The raw `.hgt` file for the Studio map.
    pub fn file(&self, zone: u32) -> Option<PathBuf> {
        let path = self.dir.as_ref()?.join(format!("zone_{zone}.hgt"));
        (self.zones.contains_key(&zone) && path.is_file()).then_some(path)
    }

    pub fn zone_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.zones.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slope() -> HeightGrid {
        // A plane y = x / 2 over [0, 10] x [0, 10], as two triangles.
        let mut grid = HeightGrid::new([0.0, 0.0], [10.0, 10.0], 2.0).unwrap();
        let p = |x: f32, z: f32| [x, x / 2.0, z];
        grid.add_triangle([p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)]);
        grid.add_triangle([p(0.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)]);
        grid
    }

    #[test]
    fn rasterised_plane_interpolates_between_nodes() {
        let grid = slope();
        assert_eq!(grid.coverage(), 1.0);
        for (x, z) in [(0.0, 0.0), (3.3, 7.1), (9.9, 0.5), (5.0, 5.0)] {
            let h = grid.height_at(x, z).unwrap();
            assert!((h - x / 2.0).abs() < 0.06, "{x},{z}: {h}");
        }
        assert_eq!(grid.height_at(-50.0, 3.0), None);
        assert_eq!(grid.height_range(), Some((0.0, 5.0)));
    }

    #[test]
    fn overlapping_surfaces_keep_the_highest() {
        let mut grid = slope();
        grid.add_triangle([[0.0, 20.0, 0.0], [4.0, 20.0, 0.0], [0.0, 20.0, 4.0]]);
        assert!((grid.height_at(0.5, 0.5).unwrap() - 20.0).abs() < 0.01);
        assert!((grid.height_at(8.0, 8.0).unwrap() - 4.0).abs() < 0.06);
    }

    #[test]
    fn bytes_round_trip_and_reject_damage() {
        let grid = slope();
        let bytes = grid.to_bytes();
        assert_eq!(HeightGrid::from_bytes(&bytes).unwrap(), grid);
        assert!(HeightGrid::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(HeightGrid::from_bytes(b"OSFRHGT0").is_err());
    }

    #[test]
    fn terrain_loads_obstacles_beside_the_grid() {
        let dir =
            std::env::temp_dir().join(format!("osfr-terrain-obstacles-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("zone_7.hgt"), slope().to_bytes()).unwrap();
        let deck = crate::obstacles::ZoneObstacles::from_world_triangles(
            vec![[[0.0, 5.0, 0.0], [4.0, 5.0, 0.0], [0.0, 5.0, 4.0]]],
            8.0,
        )
        .unwrap();
        fs::write(dir.join("zone_7.obs"), deck.to_bytes()).unwrap();
        let terrain = Terrain::load(&dir).unwrap();
        assert!(terrain.obstacles().has(7));
        assert_eq!(
            terrain.obstacles().surface_below(7, 0.5, 0.5, 5.0, 3.0),
            Some(5.0)
        );
        assert_eq!(
            terrain.obstacles().surface_below(8, 0.5, 0.5, 5.0, 3.0),
            None
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    /// T534 review: a caller that only needs heights (the shared placement suites) skips every
    /// `.obs` parse, so a multi-zone obstacle bake does not tax each test process. `load` still
    /// reads them.
    #[test]
    fn a_heights_only_load_skips_the_obstacle_files() {
        let dir = std::env::temp_dir().join(format!("osfr-terrain-heights-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("zone_7.hgt"), slope().to_bytes()).unwrap();
        let deck = crate::obstacles::ZoneObstacles::from_world_triangles(
            vec![[[0.0, 5.0, 0.0], [4.0, 5.0, 0.0], [0.0, 5.0, 4.0]]],
            8.0,
        )
        .unwrap();
        fs::write(dir.join("zone_7.obs"), deck.to_bytes()).unwrap();
        let heights = Terrain::load_heights(&dir).unwrap();
        assert!(heights.height_at(7, 4.0, 4.0).is_some());
        assert!(
            !heights.obstacles().has(7),
            "the height-only load skipped it"
        );
        assert_eq!(
            heights.obstacles().surface_below(7, 0.5, 0.5, 5.0, 3.0),
            None
        );
        let full = Terrain::load(&dir).unwrap();
        assert!(full.obstacles().has(7), "the ordinary load reads it");
        fs::remove_dir_all(&dir).unwrap();
    }

    /// T534 review: an obstacle file without a height grid keeps the zone ungridded, so the
    /// pet walk's owner-height fallback still applies. The baker writes the grid first and no
    /// `.obs` for a refused zone, but a hand-placed or stale file must not look like ground.
    #[test]
    fn an_obstacle_file_without_a_height_grid_is_not_a_grid() {
        let dir =
            std::env::temp_dir().join(format!("osfr-terrain-obs-only-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let deck = crate::obstacles::ZoneObstacles::from_world_triangles(
            vec![[[0.0, 5.0, 0.0], [4.0, 5.0, 0.0], [0.0, 5.0, 4.0]]],
            8.0,
        )
        .unwrap();
        fs::write(dir.join("zone_7.obs"), deck.to_bytes()).unwrap();
        let terrain = Terrain::load(&dir).unwrap();
        assert!(terrain.grid(7).is_none(), "there is no height grid");
        assert!(terrain.obstacles().has(7), "the obstacle file loaded");
        assert_eq!(terrain.height_at(7, 0.5, 0.5), None);
        assert_eq!(
            terrain.obstacles().surface_below(7, 0.5, 0.5, 5.0, 3.0),
            Some(5.0)
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn terrain_loads_zone_files_by_id() {
        let dir = std::env::temp_dir().join(format!("osfr-terrain-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("zone_7.hgt"), slope().to_bytes()).unwrap();
        fs::write(dir.join("notes.txt"), b"ignored").unwrap();
        let terrain = Terrain::load(&dir).unwrap();
        assert_eq!(terrain.zone_ids(), vec![7]);
        assert!((terrain.height_at(7, 4.0, 4.0).unwrap() - 2.0).abs() < 0.06);
        assert_eq!(terrain.height_at(0, 4.0, 4.0), None);
        assert!(terrain.file(7).is_some());
        fs::remove_dir_all(&dir).unwrap();
    }
}
