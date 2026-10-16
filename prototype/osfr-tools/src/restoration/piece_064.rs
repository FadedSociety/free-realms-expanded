//! Piece 064 - ForgeLight `CDTA` collision container prefix and the
//! render-versus-collision boundary for the starting dungeon.
//!
//! ORIGINAL facts implemented here (provenance and hashes in
//! `notes/Research/Pieces/064 Dungeon collision/reachability.md`):
//!
//! * Every cave room/door actor ships a `*.cdt` asset that begins with the
//!   ASCII magic `CDTA`. The room actor `sg_npx_cavern_01_room_01.adr`
//!   (pack_19/001776.adr) contains the literal file name
//!   `sg_npx_cavern_01_room_01.cdt`, so the binding is first-party, not
//!   inferred from the name alone.
//! * The container prefix is
//!   `CDTA | u32 version | u32 a | u32 b | u32 c | u32 vertex_count |
//!    f32[vertex_count * 3] | u32 index_count | u16[index_count]`.
//!   `version = 1` on every observed asset; `(a, b, c) = (1, 1, 0)` on the
//!   rooms and doors and `(8, 1, 0)` on the water collider, so the consumer
//!   must not assume fixed values there. Every observed index is
//!   `< vertex_count`; index counts are not always a multiple of three
//!   (`sg_npx_cavern_01_door_01.cdt` has 464), so trailing values are kept
//!   but not interpreted.
//! * On `pack_01/001632.cdt` (room_01) 994 of the 1007 unique prefix
//!   triangles are byte-equal render triangles of the pinned community
//!   reader's `sg_npx_cavern_01_room_01_lod0.dme` export.
//!
//! RECONSTRUCTED, labelled at the call site: `MAX_SLOPE_DEGREES` and the
//! "highest walkable surface below the probe" query. They are an analysis
//! policy for the checker, not recovered game physics, and the absolute
//! surface orientation is accepted because the real payloads contain both
//! windings.
//!
//! NOT CLAIMED: that the parsed prefix is the complete collision mesh. The
//! bytes after the prefix (91% of `001632.cdt`) and the zlib-compressed
//! collision payload inside every `*.gcnk` chunk stay unparsed, so no
//! reachability or native-traversal claim follows from this module alone.

/// ASCII magic of the recovered ForgeLight collision container.
pub const CDTA_MAGIC: [u8; 4] = *b"CDTA";
/// Container version observed on every preserved cave collision asset.
pub const CDTA_VERSION: u32 = 1;
/// Fixed prefix size up to and including the vertex count.
const PREFIX_BYTES: usize = 24;

/// Reconstructed analysis policy: a triangle is treated as walkable surface
/// when its normal is within this many degrees of vertical, in either
/// winding. The value is a checker choice; the real client slope limit is
/// not recovered.
pub const MAX_SLOPE_DEGREES: f32 = 50.0;

/// Axis-aligned bounds of a vertex set, in the asset's own (actor-local) space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// How a collision bound relates to the render bound of the same actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsRelation {
    /// Both boxes agree within the tolerance: the collision bound is not
    /// evidence of a difference.
    Identical,
    /// The collision box is inside the render box: the collision asset covers
    /// a smaller volume than what is rendered.
    CollisionSubsetOfRender,
    /// The collision box leaves the render box: the collision asset covers
    /// volume that is not rendered.
    CollisionExceedsRender,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdtaError {
    TooShort,
    BadMagic,
    UnsupportedVersion(u32),
    TruncatedVertices,
    TruncatedIndices,
    IndexOutOfRange { index: u16, vertices: usize },
}

/// Parsed `CDTA` container prefix. `tail_len` records the bytes this reader
/// deliberately does not decode.
#[derive(Debug, Clone, PartialEq)]
pub struct CdtaMesh {
    pub version: u32,
    pub header: [u32; 3],
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
    pub tail_len: usize,
}

impl CdtaMesh {
    pub fn parse(bytes: &[u8]) -> Result<Self, CdtaError> {
        if bytes.len() < PREFIX_BYTES {
            return Err(CdtaError::TooShort);
        }
        if bytes[0..4] != CDTA_MAGIC {
            return Err(CdtaError::BadMagic);
        }
        let read_u32 = |offset: usize| {
            u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ])
        };
        let version = read_u32(4);
        if version != CDTA_VERSION {
            return Err(CdtaError::UnsupportedVersion(version));
        }
        let header = [read_u32(8), read_u32(12), read_u32(16)];
        let vertex_count = read_u32(20) as usize;
        let vertex_bytes = vertex_count
            .checked_mul(12)
            .ok_or(CdtaError::TruncatedVertices)?;
        let index_count_offset = PREFIX_BYTES + vertex_bytes;
        if bytes.len() < index_count_offset + 4 {
            return Err(CdtaError::TruncatedVertices);
        }
        let mut vertices = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let base = PREFIX_BYTES + i * 12;
            vertices.push([
                f32::from_le_bytes([
                    bytes[base],
                    bytes[base + 1],
                    bytes[base + 2],
                    bytes[base + 3],
                ]),
                f32::from_le_bytes([
                    bytes[base + 4],
                    bytes[base + 5],
                    bytes[base + 6],
                    bytes[base + 7],
                ]),
                f32::from_le_bytes([
                    bytes[base + 8],
                    bytes[base + 9],
                    bytes[base + 10],
                    bytes[base + 11],
                ]),
            ]);
        }
        let index_count = u32::from_le_bytes([
            bytes[index_count_offset],
            bytes[index_count_offset + 1],
            bytes[index_count_offset + 2],
            bytes[index_count_offset + 3],
        ]) as usize;
        let index_bytes = index_count
            .checked_mul(2)
            .ok_or(CdtaError::TruncatedIndices)?;
        let indices_offset = index_count_offset + 4;
        if bytes.len() < indices_offset + index_bytes {
            return Err(CdtaError::TruncatedIndices);
        }
        let mut indices = Vec::with_capacity(index_count);
        for i in 0..index_count {
            let base = indices_offset + i * 2;
            let index = u16::from_le_bytes([bytes[base], bytes[base + 1]]);
            if index as usize >= vertex_count {
                return Err(CdtaError::IndexOutOfRange {
                    index,
                    vertices: vertex_count,
                });
            }
            indices.push(index);
        }
        Ok(Self {
            version,
            header,
            vertices,
            indices,
            tail_len: bytes.len() - (indices_offset + index_bytes),
        })
    }

    /// Whole triangles in the prefix. A trailing partial triangle (the door
    /// assets carry one) is not interpreted.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn triangles(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        (0..self.triangle_count()).map(move |i| {
            [
                self.vertices[self.indices[i * 3] as usize],
                self.vertices[self.indices[i * 3 + 1] as usize],
                self.vertices[self.indices[i * 3 + 2] as usize],
            ]
        })
    }

    pub fn bounds(&self) -> Option<Aabb> {
        let first = *self.vertices.first()?;
        let mut bounds = Aabb {
            min: first,
            max: first,
        };
        for vertex in &self.vertices {
            for axis in 0..3 {
                bounds.min[axis] = bounds.min[axis].min(vertex[axis]);
                bounds.max[axis] = bounds.max[axis].max(vertex[axis]);
            }
        }
        Some(bounds)
    }
}

/// Compare a collision bound with the render bound of the same actor. The
/// distinction matters: an identical bound is not collision proof, and a
/// subset bound means the collision asset deliberately covers less than the
/// render mesh.
pub fn classify_bounds(collision: &Aabb, render: &Aabb, epsilon: f32) -> BoundsRelation {
    let inside = (0..3).all(|axis| {
        collision.min[axis] >= render.min[axis] - epsilon
            && collision.max[axis] <= render.max[axis] + epsilon
    });
    let coincides = inside
        && (0..3).all(|axis| {
            (collision.min[axis] - render.min[axis]).abs() <= epsilon
                && (collision.max[axis] - render.max[axis]).abs() <= epsilon
        });
    match (coincides, inside) {
        (true, _) => BoundsRelation::Identical,
        (false, true) => BoundsRelation::CollisionSubsetOfRender,
        (false, false) => BoundsRelation::CollisionExceedsRender,
    }
}

/// Highest walkable prefix surface at `(x, z)` that is at or below `probe_y`.
/// Triangles outside the slope policy are ignored; the query is an analysis
/// helper, not a character controller.
pub fn horizontal_floor(
    mesh: &CdtaMesh,
    x: f32,
    z: f32,
    probe_y: f32,
    max_slope_degrees: f32,
) -> Option<f32> {
    let limit = max_slope_degrees.to_radians().cos();
    let mut best: Option<f32> = None;
    for [a, b, c] in mesh.triangles() {
        let (ux, uy, uz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
        let (vx, vy, vz) = (c[0] - a[0], c[1] - a[1], c[2] - a[2]);
        let (nx, ny, nz) = (uy * vz - uz * vy, uz * vx - ux * vz, ux * vy - uy * vx);
        let length = (nx * nx + ny * ny + nz * nz).sqrt();
        if length <= 1e-12 || (ny / length).abs() < limit {
            continue;
        }
        // Barycentric containment in the XZ projection.
        let denom = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
        if denom.abs() <= 1e-12 {
            continue;
        }
        let l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denom;
        let l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denom;
        let l3 = 1.0 - l1 - l2;
        if l1 < -1e-6 || l2 < -1e-6 || l3 < -1e-6 {
            continue;
        }
        let y = l1 * a[1] + l2 * b[1] + l3 * c[1];
        if y <= probe_y && best.map_or(true, |current| y > current) {
            best = Some(y);
        }
    }
    best
}
