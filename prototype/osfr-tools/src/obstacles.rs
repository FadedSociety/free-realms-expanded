//! Zone obstacle meshes from the client's placed `CDTA` collision assets (T534).
//!
//! `osfr-terrain --obstacles` transforms the `CDTA` prefix mesh named by every placed
//! `.adr` runtime object into world space and writes `zone_<id>.obs` beside the
//! height grid. `Terrain::load` loads the file; a zone without one keeps the
//! heightfield-only behaviour, so the feature is dormant until the bake is run.
//!
//! Format `OSFROBS1` (little endian): an 8-byte magic, `u32` version, `f32` cell
//! size, `f32` min x, `f32` min z, `u32` cols, `u32` rows, `u32` triangles,
//! `u32` bucket items; then `triangles` world-space triangles as nine `f32`; then
//! `cols * rows + 1` `u32` bucket starts; then the `u32` triangle ids. A triangle is
//! listed in every cell its x/z bounds touch, so both queries inspect only the
//! cells near the point or segment they are asked about.
//!
//! Query cost is statically bounded: a file may list at most `MAX_BUCKET_ITEMS`
//! triangles in one bucket, `surface_below` scans one bucket, and
//! `segment_blocked` inspects at most `MAX_QUERY_TRIANGLES` triangles, refusing
//! (blocked) when a pathological path would exceed that. A hostile client can
//! therefore not turn one click into a whole-mesh walk (T534 review).
//!
//! Evidence: the placement reader is `read_terrain_object` in `restoration/piece_098.rs`
//! (mirroring the pinned MIT ForgeLightToolkit reader). The client parser
//! `sub_D43670` (0x00d43670) reads the same 16-byte position, 16-byte rotation and
//! `f32` scale, and `RuntimeObject::CreateActor` (`sub_D43910` 0x00d43910) copies
//! the three fields into the actor spawn record. In the FabledRealms bake the stored
//! rotation is a radian Euler triple, not a quaternion: only 1,072 of 41,491 rows
//! have unit 4-vector length, component 0 spans -6.15..3.14, component 1 stays
//! inside -1.57..1.57 (pitch) and component 3 is always 0.
//!
//! (R) Not recovered from the client: the Euler-to-matrix order (roll, then pitch,
//! then yaw, matching the pinned reader's Unity `Quaternion.Euler(y, x, z)`), the
//! body height and the step clearance. The walk caller supplies the body height and
//! step clearance from the `pets.json` `follow` block, and the two synthetic gates
//! pin the behaviour the server depends on.

use std::{collections::HashMap, fs, path::Path};

const MAGIC: &[u8; 8] = b"OSFROBS1";
const VERSION: u32 = 1;
/// A bucket is this wide on both axes (m). Triangles are small relative to it.
pub const CELL_M: f32 = 8.0;
/// (R) A surface steeper than this is a wall, never walking ground. One definition for
/// the bake, the collision analysis (`restoration/piece_064.rs`) and the walk query.
pub const MAX_SLOPE_DEGREES: f32 = 50.0;
const HEADER: usize = 8 + 4 + 4 + 4 + 4 + 4 + 4 + 4 + 4;
const MAX_FILE_BYTES: usize = 256 << 20;
/// With `MAX_ITEMS` and `MAX_BUCKETS`, the largest `to_bytes` is under
/// `MAX_FILE_BYTES`; `tests::the_writer_caps_fit_the_loader_limit` pins the arithmetic.
const MAX_TRIANGLES: usize = 2_000_000;
const MAX_BUCKET_SIDE: u64 = 4096;
const MAX_BUCKETS: usize = 4_000_000;
const MAX_ITEMS: usize = 32_000_000;
/// The most triangles one 8 m bucket may hold. The real FabledRealms bake peaks
/// at 2,275, so this keeps every legitimate query bounded while refusing a file
/// that would make one point query scan an unbounded list.
const MAX_BUCKET_ITEMS: usize = 4_096;
/// The most triangle tests one swept segment query may spend before it refuses.
/// A path through several dense buckets is refused as blocked rather than
/// scanned without bound; the real bake's mean bucket holds 37.6 triangles.
const MAX_QUERY_TRIANGLES: usize = 8_192;
/// Total `.obs` bytes kept resident. The FabledRealms zone is 114.6 MB, so this
/// bounds a multi-zone server without silently loading every file; further zones are
/// skipped with one log line and keep the heightfield-only walk.
const MAX_LOADED_BYTES: usize = 512 << 20;
const EPS: f32 = 1e-6;

/// World position of one placement-local point.
///
/// `rotation` is the stored radian Euler triple `[yaw, pitch, roll]`. The local point
/// is scaled first, then rotated roll about z, pitch about x and yaw about y, then
/// translated. The pinned reader passes `(stored.y, stored.x, stored.z)` to Unity
/// `Quaternion.Euler`, whose order is z, x, y; this is that order in the asset's own
/// right-handed coordinates (R).
pub fn placed(position: [f32; 3], rotation: [f32; 3], scale: f32, local: [f32; 3]) -> [f32; 3] {
    let (sin_yaw, cos_yaw) = rotation[0].sin_cos();
    let (sin_pitch, cos_pitch) = rotation[1].sin_cos();
    let (sin_roll, cos_roll) = rotation[2].sin_cos();
    let (x, y, z) = (local[0] * scale, local[1] * scale, local[2] * scale);
    // Roll about z, then pitch about x, then yaw about y.
    let (x, y) = (x * cos_roll - y * sin_roll, x * sin_roll + y * cos_roll);
    let (y, z) = (y * cos_pitch - z * sin_pitch, y * sin_pitch + z * cos_pitch);
    let (x, z) = (x * cos_yaw + z * sin_yaw, -x * sin_yaw + z * cos_yaw);
    [position[0] + x, position[1] + y, position[2] + z]
}

/// The `y` at which `triangle` crosses the vertical line through `(x, z)`, when the
/// triangle is walking ground there: its normal is within `max_slope_degrees` of
/// vertical (either winding) and `(x, z)` lies inside its x/z projection. `None` for
/// a wall, a degenerate or non-finite triangle, or a point outside it.
///
/// This is the one triangle policy shared by the pet walk's surface query and the
/// `piece_064` collision analysis, so a slope or containment fix never lives twice.
pub fn triangle_surface(
    triangle: [[f32; 3]; 3],
    x: f32,
    z: f32,
    max_slope_degrees: f32,
) -> Option<f32> {
    if !x.is_finite()
        || !z.is_finite()
        || !max_slope_degrees.is_finite()
        || triangle.iter().flatten().any(|value| !value.is_finite())
    {
        return None;
    }
    let [a, b, c] = triangle;
    let (ux, uy, uz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
    let (vx, vy, vz) = (c[0] - a[0], c[1] - a[1], c[2] - a[2]);
    let (nx, ny, nz) = (uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx);
    let length_sq = nx * nx + ny * ny + nz * nz;
    let limit = max_slope_degrees.to_radians().cos();
    if length_sq <= 1e-12 || ny * ny < length_sq * limit * limit {
        return None;
    }
    // Barycentric containment in the x/z projection.
    let denominator = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
    if denominator.abs() <= 1e-12 {
        return None;
    }
    let l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denominator;
    let l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denominator;
    let l3 = 1.0 - l1 - l2;
    if l1 < -EPS || l2 < -EPS || l3 < -EPS {
        return None;
    }
    Some(l1 * a[1] + l2 * b[1] + l3 * c[1])
}

/// One zone's baked world triangles and their uniform-grid index.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneObstacles {
    cell: f32,
    min_x: f32,
    min_z: f32,
    cols: u32,
    rows: u32,
    triangles: Vec<[[f32; 3]; 3]>,
    bucket_start: Vec<u32>,
    bucket_items: Vec<u32>,
}

impl ZoneObstacles {
    /// Bucket world triangles into a fresh grid. Fails on a non-finite vertex, an
    /// empty mesh, or a grid over the format's bounds; the baker writes nothing then.
    pub fn from_world_triangles(triangles: Vec<[[f32; 3]; 3]>, cell: f32) -> Result<Self, String> {
        if triangles.is_empty() {
            return Err("obstacle mesh has no triangles".into());
        }
        if !(cell.is_finite() && (0.5..=256.0).contains(&cell)) {
            return Err(format!("obstacle cell {cell} is outside 0.5..=256 m"));
        }
        let (mut min_x, mut min_z) = (f32::MAX, f32::MAX);
        let (mut max_x, mut max_z) = (f32::MIN, f32::MIN);
        for triangle in &triangles {
            for vertex in triangle {
                if !vertex.iter().all(|value| value.is_finite()) {
                    return Err("obstacle vertex is not finite".into());
                }
                min_x = min_x.min(vertex[0]);
                min_z = min_z.min(vertex[2]);
                max_x = max_x.max(vertex[0]);
                max_z = max_z.max(vertex[2]);
            }
        }
        // A finite vertex span can still be `f32::MAX`, whose quotient saturates the
        // integer cast; `saturating_add` keeps that a clean bounds refusal, not a panic.
        let cols = (((max_x - min_x) / cell).floor() as u64).saturating_add(1);
        let rows = (((max_z - min_z) / cell).floor() as u64).saturating_add(1);
        if cols == 0 || rows == 0 || cols > MAX_BUCKET_SIDE || rows > MAX_BUCKET_SIDE {
            return Err(format!(
                "obstacle grid {cols}x{rows} is outside the format bounds"
            ));
        }
        let buckets = (cols * rows) as usize;
        if buckets > MAX_BUCKETS || triangles.len() > MAX_TRIANGLES {
            return Err("obstacle grid or triangle count is over the format limit".into());
        }
        let mut counts = vec![0u32; buckets];
        let mut items = 0usize;
        for triangle in &triangles {
            let (c0, c1) = self_span(triangle.iter().map(|v| v[0]), min_x, cell, cols);
            let (r0, r1) = self_span(triangle.iter().map(|v| v[2]), min_z, cell, rows);
            for row in r0..=r1 {
                for col in c0..=c1 {
                    let bucket = row as usize * cols as usize + col as usize;
                    counts[bucket] += 1;
                    if counts[bucket] as usize > MAX_BUCKET_ITEMS {
                        return Err("one obstacle bucket holds too many triangles".into());
                    }
                    items += 1;
                    if items > MAX_ITEMS {
                        return Err("obstacle bucket references are over the format limit".into());
                    }
                }
            }
        }
        let mut bucket_start = vec![0u32; buckets + 1];
        for index in 0..buckets {
            bucket_start[index + 1] = bucket_start[index] + counts[index];
        }
        let mut cursor = bucket_start.clone();
        let mut bucket_items = vec![0u32; items];
        for (id, triangle) in triangles.iter().enumerate() {
            let (c0, c1) = self_span(triangle.iter().map(|v| v[0]), min_x, cell, cols);
            let (r0, r1) = self_span(triangle.iter().map(|v| v[2]), min_z, cell, rows);
            for row in r0..=r1 {
                for col in c0..=c1 {
                    let bucket = row as usize * cols as usize + col as usize;
                    bucket_items[cursor[bucket] as usize] = id as u32;
                    cursor[bucket] += 1;
                }
            }
        }
        Ok(Self {
            cell,
            min_x,
            min_z,
            cols: cols as u32,
            rows: rows as u32,
            triangles,
            bucket_start,
            bucket_items,
        })
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// True when any triangle crosses the walk segment between
    /// `ground + clearance` and `ground + body_height`. `clearance` is the follow
    /// block's `step_up_m` and `body_height` its `body_height_m`; both are content
    /// values, not compiled gameplay limits. The test is the exact swept band: for
    /// every vertical offset in that range, the lifted segment's barycentric
    /// coordinates are affine in the offset, so one interval intersection covers a
    /// wall, a thin tabletop between probe heights and a low curb. Invalid input is
    /// refused as blocked, never allowed.
    pub fn segment_blocked(
        &self,
        a: [f32; 3],
        b: [f32; 3],
        clearance: f32,
        body_height: f32,
    ) -> bool {
        if !a.iter().chain(&b).all(|value| value.is_finite())
            || !clearance.is_finite()
            || clearance < 0.0
            || !body_height.is_finite()
            || body_height <= clearance + 0.1
        {
            return true;
        }
        if (a[0] - b[0]).abs() <= 1e-4 && (a[2] - b[2]).abs() <= 1e-4 {
            return false;
        }
        self.segment_hits(a, b, clearance, body_height)
    }

    /// The highest horizontal surface at `(x, z)` at or below `from_y` and at most
    /// `step_down` below it, so a bridge deck answers as ground. `step_down` is the
    /// follow block's `step_down_m`.
    pub fn surface_below(&self, x: f32, z: f32, from_y: f32, step_down: f32) -> Option<f32> {
        if !x.is_finite()
            || !z.is_finite()
            || !from_y.is_finite()
            || !(step_down.is_finite() && step_down > 0.0)
        {
            return None;
        }
        let (col, row) = self.cell_of(x, z)?;
        let bucket = row as usize * self.cols as usize + col as usize;
        let start = self.bucket_start[bucket] as usize;
        let end = self.bucket_start[bucket + 1] as usize;
        let mut best: Option<f32> = None;
        for &id in &self.bucket_items[start..end] {
            let Some(y) = triangle_surface(self.triangles[id as usize], x, z, MAX_SLOPE_DEGREES)
            else {
                continue;
            };
            if y <= from_y && y >= from_y - step_down && best.is_none_or(|top| y > top) {
                best = Some(y);
            }
        }
        best
    }

    fn cell_of(&self, x: f32, z: f32) -> Option<(u32, u32)> {
        let col = ((x - self.min_x) / self.cell).floor();
        let row = ((z - self.min_z) / self.cell).floor();
        if !col.is_finite() || !row.is_finite() {
            return None;
        }
        if col < 0.0 || row < 0.0 || col >= self.cols as f32 || row >= self.rows as f32 {
            return None;
        }
        Some((col as u32, row as u32))
    }

    fn segment_hits(&self, a: [f32; 3], b: [f32; 3], clearance: f32, body_height: f32) -> bool {
        // Only the cells the segment's x/z box overlaps can hold a hit. A segment
        // entirely outside the grid has none (every triangle is inside it).
        let low_x = a[0].min(b[0]);
        let high_x = a[0].max(b[0]);
        let low_z = a[2].min(b[2]);
        let high_z = a[2].max(b[2]);
        let grid_high_x = self.min_x + self.cols as f32 * self.cell;
        let grid_high_z = self.min_z + self.rows as f32 * self.cell;
        if low_x > grid_high_x || high_x < self.min_x || low_z > grid_high_z || high_z < self.min_z
        {
            return false;
        }
        let col = |x: f32| {
            (((x - self.min_x) / self.cell).floor()).clamp(0.0, (self.cols - 1) as f32) as u32
        };
        let row = |z: f32| {
            (((z - self.min_z) / self.cell).floor()).clamp(0.0, (self.rows - 1) as f32) as u32
        };
        let mut budget = MAX_QUERY_TRIANGLES;
        for row in row(low_z)..=row(high_z) {
            for col in col(low_x)..=col(high_x) {
                if self.bucket_hits(col, row, a, b, clearance, body_height, &mut budget) {
                    return true;
                }
            }
        }
        false
    }

    /// Inspect one bucket's triangles, stopping after `budget` tests. Exhausting the
    /// budget is a refusal (blocked): a path that would need more work than the
    /// format's query bound is not allowed through, never scanned without limit.
    fn bucket_hits(
        &self,
        col: u32,
        row: u32,
        a: [f32; 3],
        b: [f32; 3],
        clearance: f32,
        body_height: f32,
        budget: &mut usize,
    ) -> bool {
        let bucket = row as usize * self.cols as usize + col as usize;
        let start = self.bucket_start[bucket] as usize;
        let end = self.bucket_start[bucket + 1] as usize;
        for &id in &self.bucket_items[start..end] {
            if *budget == 0 {
                return true;
            }
            *budget -= 1;
            if triangle_crosses_band(a, b, self.triangles[id as usize], clearance, body_height) {
                return true;
            }
        }
        false
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            HEADER
                + self.triangles.len() * 36
                + (self.bucket_start.len() + self.bucket_items.len()) * 4,
        );
        out.extend(MAGIC);
        out.extend(VERSION.to_le_bytes());
        out.extend(self.cell.to_le_bytes());
        out.extend(self.min_x.to_le_bytes());
        out.extend(self.min_z.to_le_bytes());
        out.extend(self.cols.to_le_bytes());
        out.extend(self.rows.to_le_bytes());
        out.extend((self.triangles.len() as u32).to_le_bytes());
        out.extend((self.bucket_items.len() as u32).to_le_bytes());
        for triangle in &self.triangles {
            for vertex in triangle {
                for value in vertex {
                    out.extend(value.to_le_bytes());
                }
            }
        }
        for start in &self.bucket_start {
            out.extend(start.to_le_bytes());
        }
        for id in &self.bucket_items {
            out.extend(id.to_le_bytes());
        }
        out
    }

    /// Parse one bounded `OSFROBS1` zone. Any inconsistency is an error; the caller
    /// logs it and leaves the zone without obstacles.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_FILE_BYTES {
            return Err("obstacle file is over the size limit".into());
        }
        if bytes.len() < HEADER || &bytes[..8] != MAGIC {
            return Err("not an OSFROBS1 obstacle file".into());
        }
        let f = |at: usize| -> Result<f32, String> {
            let raw = bytes
                .get(at..at + 4)
                .ok_or_else(|| "truncated obstacle header".to_string())?
                .try_into()
                .map_err(|_| "truncated obstacle header".to_string())?;
            Ok(f32::from_le_bytes(raw))
        };
        let u = |at: usize| -> Result<u32, String> {
            let raw = bytes
                .get(at..at + 4)
                .ok_or_else(|| "truncated obstacle header".to_string())?
                .try_into()
                .map_err(|_| "truncated obstacle header".to_string())?;
            Ok(u32::from_le_bytes(raw))
        };
        let version = u(8)?;
        if version != VERSION {
            return Err(format!("unsupported obstacle version {version}"));
        }
        let cell = f(12)?;
        let (min_x, min_z) = (f(16)?, f(20)?);
        let (cols, rows) = (u(24)?, u(28)?);
        let (triangles, items) = (u(32)? as usize, u(36)? as usize);
        if !(cell.is_finite() && (0.5..=256.0).contains(&cell))
            || !min_x.is_finite()
            || !min_z.is_finite()
        {
            return Err("bad obstacle grid header".into());
        }
        if cols as u64 > MAX_BUCKET_SIDE || rows as u64 > MAX_BUCKET_SIDE {
            return Err("obstacle grid is over the format limit".into());
        }
        let buckets = cols as usize * rows as usize;
        if buckets == 0 || buckets > MAX_BUCKETS || triangles > MAX_TRIANGLES || items > MAX_ITEMS {
            return Err("obstacle counts are over the format limit".into());
        }
        let expected = HEADER
            .checked_add(triangles * 36)
            .and_then(|size| size.checked_add((buckets + 1) * 4))
            .and_then(|size| size.checked_add(items * 4))
            .ok_or_else(|| "obstacle file size overflows".to_string())?;
        if bytes.len() != expected {
            return Err(format!(
                "obstacle file is {} bytes, header says {expected}",
                bytes.len()
            ));
        }
        let mut offset = HEADER;
        let mut mesh = Vec::with_capacity(triangles);
        for _ in 0..triangles {
            let mut triangle = [[0.0f32; 3]; 3];
            for vertex in &mut triangle {
                for value in vertex {
                    let raw: [u8; 4] = bytes[offset..offset + 4]
                        .try_into()
                        .map_err(|_| "truncated obstacle triangle".to_string())?;
                    *value = f32::from_le_bytes(raw);
                    if !value.is_finite() {
                        return Err("obstacle triangle is not finite".into());
                    }
                    offset += 4;
                }
            }
            mesh.push(triangle);
        }
        let mut bucket_start = Vec::with_capacity(buckets + 1);
        for _ in 0..=buckets {
            bucket_start.push(u(offset)?);
            offset += 4;
        }
        if bucket_start[0] != 0
            || bucket_start[buckets] as usize != items
            || bucket_start.windows(2).any(|pair| pair[0] > pair[1])
        {
            return Err("obstacle bucket starts are inconsistent".into());
        }
        if bucket_start
            .windows(2)
            .any(|pair| (pair[1] - pair[0]) as usize > MAX_BUCKET_ITEMS)
        {
            return Err("one obstacle bucket holds too many triangles".into());
        }
        let mut bucket_items = Vec::with_capacity(items);
        for _ in 0..items {
            let id = u(offset)?;
            offset += 4;
            if id as usize >= triangles {
                return Err("obstacle bucket names a missing triangle".into());
            }
            bucket_items.push(id);
        }
        Ok(Self {
            cell,
            min_x,
            min_z,
            cols,
            rows,
            triangles: mesh,
            bucket_start,
            bucket_items,
        })
    }
}

/// Indices of the buckets that cover one axis' `[min, max]`.
fn self_span(values: impl Iterator<Item = f32>, origin: f32, cell: f32, count: u64) -> (u32, u32) {
    let mut low = f32::MAX;
    let mut high = f32::MIN;
    for value in values {
        low = low.min(value);
        high = high.max(value);
    }
    let first = ((low - origin) / cell).floor().max(0.0) as u64;
    let last = ((high - origin) / cell).floor().max(0.0) as u64;
    (first.min(count - 1) as u32, last.min(count - 1) as u32)
}

/// Whether the body band swept along one walk segment hits `triangle`.
///
/// A triangle with a non-degenerate x/z projection is a height field: the walk
/// point's barycentric weights are affine in the path parameter, so the interval
/// where the triangle's surface lies `clearance..=body_height` above the walked
/// ground is one linear-programming clip. This catches a thin tabletop that a
/// horizontal probe between sample heights would miss. A vertical triangle (a wall,
/// whose x/z projection is a line) has no height field and uses the swept Möller test
/// in [`segment_band_hits`].
fn triangle_crosses_band(
    a: [f32; 3],
    b: [f32; 3],
    triangle: [[f32; 3]; 3],
    clearance: f32,
    body_height: f32,
) -> bool {
    let [va, vb, vc] = triangle;
    let denominator = (vb[2] - vc[2]) * (va[0] - vc[0]) + (vc[0] - vb[0]) * (va[2] - vc[2]);
    if denominator.abs() <= 1e-12 {
        return segment_band_hits(a, b, triangle, clearance, body_height);
    }
    let direction = sub3(b, a);
    let l1_at = ((vb[2] - vc[2]) * (a[0] - vc[0]) + (vc[0] - vb[0]) * (a[2] - vc[2])) / denominator;
    let l1_per = ((vb[2] - vc[2]) * direction[0] + (vc[0] - vb[0]) * direction[2]) / denominator;
    let l2_at = ((vc[2] - va[2]) * (a[0] - vc[0]) + (va[0] - vc[0]) * (a[2] - vc[2])) / denominator;
    let l2_per = ((vc[2] - va[2]) * direction[0] + (va[0] - vc[0]) * direction[2]) / denominator;
    let l3_at = 1.0 - l1_at - l2_at;
    let l3_per = -(l1_per + l2_per);
    let y_at = l1_at * va[1] + l2_at * vb[1] + l3_at * vc[1];
    let y_per = l1_per * va[1] + l2_per * vb[1] + l3_per * vc[1];
    // Height of the triangle above the walked ground at parameter `t`.
    let h_at = y_at - a[1];
    let h_per = y_per - direction[1];
    let mut low = 0.0f32;
    let mut high = 1.0f32;
    let constraints = [
        (l1_per, l1_at + EPS),
        (-l1_per, 1.0 + EPS - l1_at),
        (l2_per, l2_at + EPS),
        (-l2_per, 1.0 + EPS - l2_at),
        (l3_per, l3_at + EPS),
        (-l3_per, 1.0 + EPS - l3_at),
        (h_per, h_at - clearance),
        (-h_per, body_height - h_at),
    ];
    for (slope, offset) in constraints {
        if !clip_band(&mut low, &mut high, slope, offset) {
            return false;
        }
    }
    low <= high + EPS
}

/// Whether some vertical offset `h` in `clearance..=body_height` makes the segment
/// `a..b`, lifted by `h`, cross `triangle`.
///
/// Möller–Trumbore's barycentric `u`, `v` and ray parameter `t` are affine in the
/// lift: the triangle and the segment direction do not change, only the segment
/// origin. Each inside/parameter condition is therefore one linear inequality in
/// `h`; intersecting all of them with the band gives the exact set of offsets that
/// hit. Nothing is sampled.
fn segment_band_hits(
    a: [f32; 3],
    b: [f32; 3],
    triangle: [[f32; 3]; 3],
    clearance: f32,
    body_height: f32,
) -> bool {
    let [v0, v1, v2] = triangle;
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let direction = sub3(b, a);
    let hvec = cross(direction, edge2);
    let determinant = dot(edge1, hvec);
    if determinant.abs() < 1e-8 {
        return false;
    }
    let inverse = 1.0 / determinant;
    let s = sub3(a, v0);
    let q = cross(s, edge1);
    // `lift` is `cross((0, 1, 0), edge1)`, the change in `q` per metre of lift.
    let lift = [edge1[2], 0.0, -edge1[0]];
    let u_at = dot(s, hvec) * inverse;
    let u_per = hvec[1] * inverse;
    let v_at = dot(direction, q) * inverse;
    let v_per = dot(direction, lift) * inverse;
    let t_at = dot(edge2, q) * inverse;
    let t_per = dot(edge2, lift) * inverse;
    let mut low = clearance;
    let mut high = body_height;
    // Barycentric containment (with the walk check's small seam tolerance) and the
    // segment parameter in 0..=1.
    let constraints = [
        (u_per, u_at + EPS),
        (-u_per, 1.0 + EPS - u_at),
        (v_per, v_at + EPS),
        (-v_per, 1.0 + EPS - v_at),
        (-(u_per + v_per), 1.0 + EPS - u_at - v_at),
        (t_per, t_at + EPS),
        (-t_per, 1.0 + EPS - t_at),
    ];
    for (slope, offset) in constraints {
        if !clip_band(&mut low, &mut high, slope, offset) {
            return false;
        }
    }
    low <= high + EPS
}

/// Keep the part of `low..=high` where `slope * h + offset >= 0`; false when empty.
fn clip_band(low: &mut f32, high: &mut f32, slope: f32, offset: f32) -> bool {
    if slope.abs() <= 1e-9 {
        return offset >= -EPS;
    }
    let bound = -offset / slope;
    if slope > 0.0 {
        *low = low.max(bound);
    } else {
        *high = high.min(bound);
    }
    *low <= *high + EPS
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

/// Every zone's baked obstacle mesh, loaded from `zone_<id>.obs` files.
#[derive(Debug, Default)]
pub struct Obstacles {
    zones: HashMap<u32, ZoneObstacles>,
}

impl Obstacles {
    /// Load every parseable `zone_<id>.obs` in `dir`, up to `MAX_LOADED_BYTES` total.
    /// A bad or over-budget file disables that zone's obstacles (one log line); it
    /// never fails the terrain load.
    pub fn load(dir: &Path) -> Self {
        let mut zones = HashMap::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return Self { zones };
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|entry| entry.file_name());
        let mut loaded_bytes = 0usize;
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(id) = name
                .strip_prefix("zone_")
                .and_then(|rest| rest.strip_suffix(".obs"))
                .and_then(|id| id.parse::<u32>().ok())
            else {
                continue;
            };
            let size = entry
                .metadata()
                .map(|meta| meta.len() as usize)
                .unwrap_or(0);
            if size > MAX_FILE_BYTES {
                eprintln!("obstacles: {name}: file is over the size limit");
                continue;
            }
            if loaded_bytes.saturating_add(size) > MAX_LOADED_BYTES {
                eprintln!("obstacles: {name}: over the loaded-obstacle budget; zone skipped");
                continue;
            }
            let bytes = match fs::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(error) => {
                    eprintln!("obstacles: {name}: {error}");
                    continue;
                }
            };
            match ZoneObstacles::from_bytes(&bytes) {
                Ok(zone) => {
                    loaded_bytes += bytes.len();
                    zones.insert(id, zone);
                }
                Err(error) => eprintln!("obstacles: {name}: {error}"),
            }
        }
        Self { zones }
    }

    pub fn has(&self, zone: u32) -> bool {
        self.zones.contains_key(&zone)
    }

    pub fn segment_blocked(
        &self,
        zone: u32,
        a: [f32; 3],
        b: [f32; 3],
        clearance: f32,
        body_height: f32,
    ) -> bool {
        self.zones
            .get(&zone)
            .is_some_and(|mesh| mesh.segment_blocked(a, b, clearance, body_height))
    }

    pub fn surface_below(
        &self,
        zone: u32,
        x: f32,
        z: f32,
        from_y: f32,
        step_down: f32,
    ) -> Option<f32> {
        self.zones
            .get(&zone)
            .and_then(|mesh| mesh.surface_below(x, z, from_y, step_down))
    }

    #[cfg(test)]
    fn zone_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.zones.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (R) The shipped `pets.json` follow limits, repeated here so the query tests stay
    /// independent of the content loader.
    const CLEARANCE: f32 = 1.0;
    const BODY: f32 = 1.8;
    const STEP_DOWN: f32 = 3.0;

    fn zone(triangles: Vec<[[f32; 3]; 3]>) -> ZoneObstacles {
        ZoneObstacles::from_world_triangles(triangles, CELL_M).unwrap()
    }

    /// A vertical 2 m wall at x = 0, from z = -5 to 5, rising from y = 0 to 2.
    fn wall_triangles() -> Vec<[[f32; 3]; 3]> {
        vec![
            [[0.0, 0.0, -5.0], [0.0, 2.0, -5.0], [0.0, 2.0, 5.0]],
            [[0.0, 0.0, -5.0], [0.0, 2.0, 5.0], [0.0, 0.0, 5.0]],
        ]
    }

    fn wall() -> ZoneObstacles {
        zone(wall_triangles())
    }

    /// A horizontal 4 x 4 m deck at y = 5.
    fn bridge() -> ZoneObstacles {
        zone(vec![
            [[0.0, 5.0, 0.0], [4.0, 5.0, 0.0], [4.0, 5.0, 4.0]],
            [[0.0, 5.0, 0.0], [4.0, 5.0, 4.0], [0.0, 5.0, 4.0]],
        ])
    }

    #[test]
    fn bytes_round_trip_and_two_bakes_are_identical() {
        use sha2::{Digest, Sha256};
        let first = ZoneObstacles::from_world_triangles(wall_triangles(), CELL_M).unwrap();
        let second = ZoneObstacles::from_world_triangles(wall_triangles(), CELL_M).unwrap();
        let first = first.to_bytes();
        let second = second.to_bytes();
        assert_eq!(first, second, "the same triangles bake to the same bytes");
        assert_eq!(
            format!("{:x}", Sha256::digest(&first)),
            format!("{:x}", Sha256::digest(&second)),
            "baking twice gives one sha256"
        );
        let parsed = ZoneObstacles::from_bytes(&first).unwrap();
        assert_eq!(parsed, wall());
        assert_eq!(parsed.triangle_count(), 2);
        assert_eq!(&first[..8], MAGIC);
    }

    #[test]
    fn the_loader_refuses_damage() {
        let good = wall().to_bytes();
        assert!(ZoneObstacles::from_bytes(b"OSFROBS0").is_err());
        assert!(ZoneObstacles::from_bytes(&good[..good.len() - 1]).is_err());
        let mut version = good.clone();
        version[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert!(ZoneObstacles::from_bytes(&version).is_err());
        let mut count = good.clone();
        count[32..36].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ZoneObstacles::from_bytes(&count).is_err());
        let mut finite = good.clone();
        finite[HEADER..HEADER + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(ZoneObstacles::from_bytes(&finite).is_err());
        let mut order = good.clone();
        let last_start = HEADER + 2 * 36;
        order[last_start..last_start + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(ZoneObstacles::from_bytes(&order).is_err());
    }

    /// The writer accepts `MAX_TRIANGLES`/`MAX_ITEMS`/`MAX_BUCKETS`, so the file it may
    /// produce must stay under the loader's `MAX_FILE_BYTES`; otherwise a dense zone
    /// bakes a file the server refuses.
    #[test]
    fn the_writer_caps_fit_the_loader_limit() {
        let largest = HEADER + MAX_TRIANGLES * 36 + (MAX_BUCKETS + 1) * 4 + MAX_ITEMS * 4;
        assert!(
            largest <= MAX_FILE_BYTES,
            "the writer can exceed the loader cap: {largest} > {MAX_FILE_BYTES}"
        );
        // A header at the maxima is refused before any triangle is allocated.
        let mut header = Vec::from(&MAGIC[..]);
        header.extend(VERSION.to_le_bytes());
        header.extend(CELL_M.to_le_bytes());
        header.extend(0.0f32.to_le_bytes());
        header.extend(0.0f32.to_le_bytes());
        header.extend((MAX_BUCKET_SIDE as u32).to_le_bytes());
        header.extend((MAX_BUCKET_SIDE as u32).to_le_bytes());
        header.extend((MAX_TRIANGLES as u32).to_le_bytes());
        header.extend((MAX_ITEMS as u32).to_le_bytes());
        assert!(ZoneObstacles::from_bytes(&header).is_err());
    }

    #[test]
    fn a_wall_blocks_the_walk_segment_but_a_low_curb_does_not() {
        let wall = wall();
        assert!(wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY));
        assert!(!wall.segment_blocked([2.0, 0.0, 0.0], [4.0, 0.0, 0.0], CLEARANCE, BODY));
        let high_wall = zone(vec![
            [[0.0, 1.0, -5.0], [0.0, 3.0, -5.0], [0.0, 3.0, 5.0]],
            [[0.0, 1.0, -5.0], [0.0, 3.0, 5.0], [0.0, 1.0, 5.0]],
        ]);
        assert!(
            !high_wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.0, 0.7),
            "a wall above a low body is not crossed"
        );
        assert!(
            high_wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY),
            "the same wall blocks a standing body"
        );
        assert!(
            wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, 0.0),
            "an invalid body height is refused, not allowed"
        );
        let curb = zone(vec![
            [[0.0, 0.0, -1.0], [0.0, 0.95, -1.0], [0.0, 0.95, 1.0]],
            [[0.0, 0.0, -1.0], [0.0, 0.95, 1.0], [0.0, 0.0, 1.0]],
        ]);
        assert!(
            !curb.segment_blocked([-1.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY),
            "an obstacle below the step clearance is stepped over"
        );
        assert!(
            curb.segment_blocked([-1.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.0, BODY),
            "the same obstacle blocks when the clearance is zero"
        );
    }

    /// T534 review: the band is exact, not three horizontal probes, so a thin obstacle
    /// between the old probe heights (1.0, 1.375 and 1.75 m) still stops a walk.
    #[test]
    fn a_thin_obstacle_between_probe_heights_blocks_the_walk() {
        let table = zone(vec![
            [[-1.0, 1.5, -1.0], [1.0, 1.5, -1.0], [1.0, 1.5, 1.0]],
            [[-1.0, 1.5, -1.0], [1.0, 1.5, 1.0], [-1.0, 1.5, 1.0]],
        ]);
        assert!(
            table.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY),
            "a tabletop at 1.5 m crosses the body band"
        );
        let higher = zone(vec![
            [[-1.0, 2.5, -1.0], [1.0, 2.5, -1.0], [1.0, 2.5, 1.0]],
            [[-1.0, 2.5, -1.0], [1.0, 2.5, 1.0], [-1.0, 2.5, 1.0]],
        ]);
        assert!(
            !higher.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY),
            "a surface above the body is not an obstacle in the band"
        );
    }

    #[test]
    fn a_bridge_deck_answers_as_ground_within_the_step_drop() {
        let bridge = bridge();
        assert_eq!(bridge.surface_below(2.0, 2.0, 5.0, STEP_DOWN), Some(5.0));
        assert_eq!(
            bridge.surface_below(2.0, 2.0, 5.5, STEP_DOWN),
            Some(5.0),
            "a probe slightly above the deck still stands on it"
        );
        assert_eq!(
            bridge.surface_below(2.0, 2.0, 8.0, STEP_DOWN),
            Some(5.0),
            "a deck exactly step_down below is still ground"
        );
        assert_eq!(
            bridge.surface_below(2.0, 2.0, 9.0, STEP_DOWN),
            None,
            "a deck below the step drop is not ground"
        );
        assert_eq!(
            bridge.surface_below(2.0, 2.0, 4.5, STEP_DOWN),
            None,
            "a deck above the probe is not ground"
        );
        assert_eq!(bridge.surface_below(20.0, 20.0, 5.0, STEP_DOWN), None);
        let wall = wall();
        assert_eq!(
            wall.surface_below(0.0, 0.0, 5.0, STEP_DOWN),
            None,
            "a vertical wall is never a walking surface"
        );
    }

    #[test]
    fn non_finite_queries_are_refused_not_allowed() {
        let wall = wall();
        let bridge = bridge();
        assert!(
            wall.segment_blocked([f32::NAN, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, BODY),
            "a non-finite segment is blocked"
        );
        assert!(
            wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], f32::NAN, BODY),
            "a non-finite clearance is blocked"
        );
        assert!(
            wall.segment_blocked([-2.0, 0.0, 0.0], [2.0, 0.0, 0.0], CLEARANCE, f32::NAN),
            "a non-finite body height is blocked"
        );
        assert_eq!(bridge.surface_below(f32::NAN, 2.0, 5.0, STEP_DOWN), None);
        assert_eq!(bridge.surface_below(2.0, f32::NAN, 5.0, STEP_DOWN), None);
        assert_eq!(bridge.surface_below(2.0, 2.0, f32::NAN, STEP_DOWN), None);
        assert_eq!(bridge.surface_below(2.0, 2.0, 5.0, f32::NAN), None);
        assert_eq!(bridge.surface_below(2.0, 2.0, 5.0, 0.0), None);
        assert_eq!(
            triangle_surface(
                [[f32::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                0.5,
                0.5,
                MAX_SLOPE_DEGREES
            ),
            None,
            "a non-finite triangle is never a surface"
        );
    }

    #[test]
    fn placed_points_use_scale_then_roll_pitch_yaw() {
        let origin = [10.0, 1.0, -3.0];
        assert_eq!(
            placed(
                origin,
                [std::f32::consts::FRAC_PI_2, 0.0, 0.0],
                1.0,
                [1.0, 0.0, 0.0]
            ),
            [10.0, 1.0, -4.0],
            "a quarter yaw turns +x to -z"
        );
        assert_eq!(
            placed(
                origin,
                [0.0, std::f32::consts::FRAC_PI_2, 0.0],
                1.0,
                [0.0, 0.0, 1.0]
            ),
            [10.0, 0.0, -3.0],
            "a quarter pitch turns +z to -y"
        );
        assert_eq!(
            placed(
                origin,
                [0.0, 0.0, std::f32::consts::FRAC_PI_2],
                2.0,
                [1.0, 0.0, 0.0]
            ),
            [10.0, 3.0, -3.0],
            "scale applies before the quarter roll"
        );
    }

    /// `count` copies of a small horizontal triangle at y = 100 (above every body band)
    /// in the cell that starts at `x`.
    fn crowd_at(x: f32, count: usize) -> Vec<[[f32; 3]; 3]> {
        vec![[[x, 100.0, 0.0], [x + 1.0, 100.0, 0.0], [x, 100.0, 1.0],]; count]
    }

    /// T534 review: one segment query may inspect at most `MAX_QUERY_TRIANGLES` triangles.
    /// A path through a crowd over that budget is refused as blocked, not scanned without
    /// bound; under the budget the same non-blocking crowd is allowed.
    #[test]
    fn a_query_over_the_triangle_budget_refuses_instead_of_scanning() {
        let under = zone(
            crowd_at(0.0, MAX_QUERY_TRIANGLES / 2)
                .into_iter()
                .chain(crowd_at(8.0, MAX_QUERY_TRIANGLES / 2 - 2))
                .collect(),
        );
        assert!(
            !under.segment_blocked([-1.0, 0.0, 0.0], [17.0, 0.0, 0.0], CLEARANCE, BODY),
            "a non-blocking crowd under the budget is not refused"
        );
        let over = zone(
            crowd_at(0.0, MAX_BUCKET_ITEMS)
                .into_iter()
                .chain(crowd_at(8.0, MAX_BUCKET_ITEMS))
                .chain(crowd_at(16.0, MAX_BUCKET_ITEMS))
                .collect(),
        );
        assert!(
            over.segment_blocked([-1.0, 0.0, 0.0], [17.0, 0.0, 0.0], CLEARANCE, BODY),
            "a path over the triangle budget is refused (blocked), not scanned"
        );
    }

    /// T534 review: the writer and the loader both refuse a bucket over `MAX_BUCKET_ITEMS`, so
    /// `surface_below` can never scan an unbounded bucket list.
    #[test]
    fn the_writer_and_loader_refuse_an_oversized_bucket() {
        let over = crowd_at(0.0, MAX_BUCKET_ITEMS + 1);
        assert!(ZoneObstacles::from_world_triangles(over, CELL_M).is_err());

        // A hand-built file bypasses the writer, so the loader must enforce the same cap.
        let triangles = vec![[[0.0, 100.0, 0.0], [1.0, 100.0, 0.0], [0.0, 100.0, 1.0]]; 4097];
        let forged = ZoneObstacles {
            cell: CELL_M,
            min_x: 0.0,
            min_z: 0.0,
            cols: 1,
            rows: 1,
            triangles,
            bucket_start: vec![0, 4097],
            bucket_items: (0..4097u32).collect(),
        };
        assert!(ZoneObstacles::from_bytes(&forged.to_bytes()).is_err());
    }

    /// T534 review: a finite pair of vertices can still span `f32::MAX`; the bounds check must
    /// refuse it instead of overflowing `u64 + 1` in a debug build.
    #[test]
    fn an_oversized_span_is_refused_without_panicking() {
        let triangle = [[f32::MIN, 0.0, 0.0], [f32::MAX, 0.0, 0.0], [0.0, 0.0, 1.0]];
        assert!(ZoneObstacles::from_world_triangles(vec![triangle], CELL_M).is_err());
    }

    #[test]
    fn load_keeps_good_zones_and_skips_bad_ones() {
        let dir = std::env::temp_dir().join(format!("osfr-obstacles-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("zone_7.obs"), bridge().to_bytes()).unwrap();
        fs::write(dir.join("zone_8.obs"), b"OSFROBS1 broken").unwrap();
        fs::write(dir.join("notes.txt"), b"ignored").unwrap();
        let obstacles = Obstacles::load(&dir);
        assert_eq!(obstacles.zone_ids(), vec![7]);
        assert!(obstacles.has(7));
        assert!(!obstacles.has(8));
        assert_eq!(
            obstacles.surface_below(7, 2.0, 2.0, 5.0, STEP_DOWN),
            Some(5.0)
        );
        assert_eq!(obstacles.surface_below(8, 2.0, 2.0, 5.0, STEP_DOWN), None);
        assert!(!obstacles.segment_blocked(8, [0.0; 3], [1.0; 3], CLEARANCE, BODY));
        fs::remove_dir_all(&dir).unwrap();
    }
}
