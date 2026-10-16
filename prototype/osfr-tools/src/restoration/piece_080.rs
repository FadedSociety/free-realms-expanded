//! Piece 080 - world collision data: reader and column query for the recovered
//! GCNK collision section.
//!
//! ORIGINAL facts implemented here (provenance and command evidence in
//! `notes/Research/Pieces/080 World collision data.md` and the private bundle
//! `../lab/reports/piece-080-20261009T044143Z/`):
//!
//! * A `.gcnk` chunk is `GCNK` + int32 version + two zlib sections, each preceded by
//!   (uncompressed, compressed) lengths. Section 2 is the collision data whose
//!   `LoadCollision` is still a `TODO Bullet Physics` no-op in the pinned community
//!   ForgeLightToolkit reader `9c06714c5d79d9152c322814bc904abad944e06e`.
//! * Section 2 is 16 per-tile payloads on the same 4x4 grid of 66-unit cells the
//!   terrain section uses, each followed by a 16-byte trailer block.
//! * A payload is `u32 payloadSize`, 28 zero bytes, an AABB (two 4-float
//!   `btVector3FloatData`), a per-axis quantisation scale equal to 65535 / span,
//!   a 128-byte prefix, a `btQuantizedBvhNode` array (6 x u16 quantised AABB plus an
//!   i32 escape-or-triangle index, 16 bytes per node), 32-byte
//!   `btBvhSubtreeInfo`-shaped subtree headers and a 16-byte trailer.
//! * Quantised coordinates dequantise with `world = aabbMin + q / scale`, matching
//!   Bullet's `btQuantizedBvh::unQuantize`. The original client's grounding path
//!   casts from `Y+2` down to `Y-2000` through collision query `00fd4290` with filter
//!   `0x143` and replaces the movement target Y with the hit Y, so a leaf column
//!   containing the sample X/Z is the surface that query can hit.
//!
//! INFERRED, labelled at the call site: the first node's escape index is the node
//! count, leaf nodes carry mesh triangle indices, and a leaf's quantised AABB is the
//! conservative box around that triangle. The triangle/vertex payload itself stays
//! unrecovered, so `column` reports bands, not exact triangle hit heights.

use miniz_oxide::inflate::decompress_to_vec_zlib;

/// Per-chunk collision tile count (4 x 4 grid, matching the terrain section).
pub const TILE_COUNT: usize = 16;
/// World size of one collision cell in X and Z (1-unit bleed over the 64-unit tile).
pub const TILE_CELL: f32 = 66.0;
/// Offset of the quantised BVH node array inside a payload.
pub const PAYLOAD_PREFIX: usize = 208;
/// Size of one `btQuantizedBvhNode`.
pub const NODE_SIZE: usize = 16;
/// Size of one `btBvhSubtreeInfo`-shaped header.
pub const SUBTREE_SIZE: usize = 32;
/// Trailing bytes after the subtree headers inside a payload.
pub const PAYLOAD_TRAILER: usize = 16;
/// Quantisation range used by the stored per-axis scale.
pub const QUANT_MAX: f32 = 65535.0;
/// Preserved original chunk used by the piece 080 evidence bundle.
pub const STARTING_CHUNK: &str = "../../../lab/extracted/client-assets/pack_01/001956.gcnk";
/// SHA256 of that preserved chunk (recorded in the lab intake manifests).
pub const STARTING_CHUNK_BYTES: usize = 435_650;

/// Reader failure; the message names the tile and field that did not match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionError(pub String);

impl std::fmt::Display for CollisionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gcnk collision: {}", self.0)
    }
}

impl std::error::Error for CollisionError {}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, CollisionError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| CollisionError(format!("truncated u32 at {offset}")))?;
    Ok(u32::from_le_bytes(raw.try_into().expect("4 bytes")))
}

fn i32_at(bytes: &[u8], offset: usize) -> Result<i32, CollisionError> {
    Ok(u32_at(bytes, offset)? as i32)
}

fn u16x6_at(bytes: &[u8], offset: usize) -> Result<[u16; 6], CollisionError> {
    let mut out = [0u16; 6];
    for (i, slot) in out.iter_mut().enumerate() {
        let raw = bytes
            .get(offset + 2 * i..offset + 2 * i + 2)
            .ok_or_else(|| CollisionError(format!("truncated u16 at {}", offset + 2 * i)))?;
        *slot = u16::from_le_bytes(raw.try_into().expect("2 bytes"));
    }
    Ok(out)
}

fn f32_at(bytes: &[u8], offset: usize) -> Result<f32, CollisionError> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| CollisionError(format!("truncated f32 at {offset}")))?;
    Ok(f32::from_le_bytes(raw.try_into().expect("4 bytes")))
}

/// One `btQuantizedBvhNode`: quantised AABB plus escape index (internal) or
/// triangle index (leaf, non-negative).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantizedNode {
    pub q_min: [u16; 3],
    pub q_max: [u16; 3],
    pub escape_or_triangle: i32,
}

impl QuantizedNode {
    pub fn is_leaf(&self) -> bool {
        self.escape_or_triangle >= 0
    }

    pub fn triangle_index(&self) -> Option<u32> {
        self.is_leaf().then_some(self.escape_or_triangle as u32)
    }

    pub fn escape_index(&self) -> Option<u32> {
        (!self.is_leaf()).then_some((-self.escape_or_triangle) as u32)
    }
}

/// One `btBvhSubtreeInfo`-shaped header (12 bytes of AABB, root index, size, 12
/// zero padding bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubtreeHeader {
    pub q_min: [u16; 3],
    pub q_max: [u16; 3],
    pub root_node_index: u32,
    pub subtree_size: u32,
}

/// One per-tile collision payload.
#[derive(Debug, Clone)]
pub struct Tile {
    pub index: usize,
    pub payload_size: usize,
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
    pub scale: [f32; 3],
    pub nodes: Vec<QuantizedNode>,
    pub subtrees: Vec<SubtreeHeader>,
}

impl Tile {
    pub fn span(&self) -> [f32; 3] {
        [
            self.aabb_max[0] - self.aabb_min[0],
            self.aabb_max[1] - self.aabb_min[1],
            self.aabb_max[2] - self.aabb_min[2],
        ]
    }

    /// `world = aabbMin + q / scale` for one quantised corner.
    pub fn dequantize(&self, q: [u16; 3]) -> [f32; 3] {
        [
            self.aabb_min[0] + q[0] as f32 / self.scale[0],
            self.aabb_min[1] + q[1] as f32 / self.scale[1],
            self.aabb_min[2] + q[2] as f32 / self.scale[2],
        ]
    }

    pub fn contains_xz(&self, x: f32, z: f32) -> bool {
        self.aabb_min[0] <= x
            && x <= self.aabb_max[0]
            && self.aabb_min[2] <= z
            && z <= self.aabb_max[2]
    }

    pub fn leaf_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_leaf()).count()
    }
}

/// One collision leaf whose dequantised XZ box contains a query column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnHit {
    pub tile_index: usize,
    pub node_index: usize,
    pub triangle_index: u32,
    pub y_min: f32,
    pub y_max: f32,
}

/// The recovered collision section of one chunk.
#[derive(Debug, Clone)]
pub struct CollisionSection {
    pub tiles: Vec<Tile>,
}

impl CollisionSection {
    /// Split a `GCNK` container and read its collision section.
    pub fn from_gcnk(bytes: &[u8]) -> Result<(i32, Vec<u8>, Self), CollisionError> {
        if bytes.len() < 16 || &bytes[..4] != b"GCNK" {
            return Err(CollisionError("not a GCNK container".into()));
        }
        let version = i32_at(bytes, 4)?;
        if !(1..=6).contains(&version) {
            return Err(CollisionError(format!(
                "unsupported GCNK version {version}"
            )));
        }
        let mut offset = 8usize;
        let mut sections = Vec::new();
        for label in ["terrain", "collision"] {
            let uncompressed = u32_at(bytes, offset)? as usize;
            let compressed = u32_at(bytes, offset + 4)? as usize;
            offset += 8;
            let data = bytes
                .get(offset..offset + compressed)
                .ok_or_else(|| CollisionError(format!("{label} section out of range")))?;
            let blob = decompress_to_vec_zlib(data)
                .map_err(|err| CollisionError(format!("{label} section is not zlib: {err:?}")))?;
            if blob.len() != uncompressed {
                return Err(CollisionError(format!(
                    "{label} section decompressed to {} bytes, header says {uncompressed}",
                    blob.len()
                )));
            }
            offset += compressed;
            sections.push(blob);
        }
        let terrain = sections.remove(0);
        let collision = sections.remove(0);
        Ok((version, terrain, Self::parse(&collision)?))
    }

    /// Parse a decompressed collision section.
    pub fn parse(section: &[u8]) -> Result<Self, CollisionError> {
        let mut tiles = Vec::with_capacity(TILE_COUNT);
        let mut offset = 0usize;
        for index in 0..TILE_COUNT {
            let size = u32_at(section, offset)? as usize;
            if size <= 0 || offset + size + 16 > section.len() {
                return Err(CollisionError(format!(
                    "entry {index} size {size} out of range"
                )));
            }
            tiles.push(parse_tile(&section[offset..offset + size], index)?);
            let trailer = &section[offset + size..offset + size + 16];
            if trailer[4..].iter().any(|b| *b != 0) {
                return Err(CollisionError(format!(
                    "entry {index} trailer is not zero padded"
                )));
            }
            offset += size + 16;
        }
        if offset != section.len() {
            return Err(CollisionError(format!(
                "{} unparsed bytes remain",
                section.len() - offset
            )));
        }
        let mut xs: Vec<i32> = tiles
            .iter()
            .map(|t| (t.aabb_min[0] * 1000.0) as i32)
            .collect();
        let mut zs: Vec<i32> = tiles
            .iter()
            .map(|t| (t.aabb_min[2] * 1000.0) as i32)
            .collect();
        xs.sort_unstable();
        xs.dedup();
        zs.sort_unstable();
        zs.dedup();
        if xs.len() != 4 || zs.len() != 4 {
            return Err(CollisionError(
                "collision tiles do not form a 4x4 grid".into(),
            ));
        }
        for tile in &tiles {
            let span = tile.span();
            if (span[0] - TILE_CELL).abs() > 0.05 || (span[2] - TILE_CELL).abs() > 0.05 {
                return Err(CollisionError(format!(
                    "tile {} is not a 66-unit cell",
                    tile.index
                )));
            }
        }
        Ok(Self { tiles })
    }

    pub fn tile_for(&self, x: f32, z: f32) -> Option<&Tile> {
        self.tiles.iter().find(|t| t.contains_xz(x, z))
    }

    /// Collision leaves whose dequantised XZ box contains the query column.
    pub fn column(&self, x: f32, z: f32) -> Vec<ColumnHit> {
        let mut hits = Vec::new();
        for tile in self.tiles.iter().filter(|t| t.contains_xz(x, z)) {
            for (node_index, node) in tile.nodes.iter().enumerate() {
                if !node.is_leaf() {
                    continue;
                }
                let lo = tile.dequantize(node.q_min);
                let hi = tile.dequantize(node.q_max);
                if lo[0] <= x && x <= hi[0] && lo[2] <= z && z <= hi[2] {
                    hits.push(ColumnHit {
                        tile_index: tile.index,
                        node_index,
                        triangle_index: node.triangle_index().expect("leaf"),
                        y_min: lo[1],
                        y_max: hi[1],
                    });
                }
            }
        }
        hits
    }
}

fn parse_tile(payload: &[u8], index: usize) -> Result<Tile, CollisionError> {
    let size = u32_at(payload, 0)? as usize;
    if size != payload.len() || size < PAYLOAD_PREFIX + NODE_SIZE + SUBTREE_SIZE + PAYLOAD_TRAILER {
        return Err(CollisionError(format!(
            "tile {index}: payload size {size} does not match"
        )));
    }
    if payload[4..32].iter().any(|b| *b != 0) {
        return Err(CollisionError(format!(
            "tile {index}: expected zero padding after the size"
        )));
    }
    let mut aabb_min = [0f32; 3];
    let mut aabb_max = [0f32; 3];
    let mut scale = [0f32; 3];
    for axis in 0..3 {
        aabb_min[axis] = f32_at(payload, 32 + 4 * axis)?;
        aabb_max[axis] = f32_at(payload, 48 + 4 * axis)?;
        scale[axis] = f32_at(payload, 64 + 4 * axis)?;
    }
    let span = [
        aabb_max[0] - aabb_min[0],
        aabb_max[1] - aabb_min[1],
        aabb_max[2] - aabb_min[2],
    ];
    if span.iter().any(|s| *s <= 0.5) {
        return Err(CollisionError(format!(
            "tile {index}: implausible span {span:?}"
        )));
    }
    for axis in 0..3 {
        let expected = QUANT_MAX / span[axis];
        if (scale[axis] - expected).abs() > 0.01 * expected {
            return Err(CollisionError(format!(
                "tile {index}: axis {axis} scale {} != 65535/span {expected}",
                scale[axis]
            )));
        }
    }

    let first_escape = i32_at(payload, PAYLOAD_PREFIX + 12)?;
    if first_escape >= 0 {
        return Err(CollisionError(format!(
            "tile {index}: first node is not an internal node"
        )));
    }
    let node_count = (-first_escape) as usize;
    if node_count < 2 || PAYLOAD_PREFIX + NODE_SIZE * node_count > size {
        return Err(CollisionError(format!(
            "tile {index}: root escape {node_count} out of range"
        )));
    }
    let mut nodes = Vec::with_capacity(node_count);
    for n in 0..node_count {
        let offset = PAYLOAD_PREFIX + NODE_SIZE * n;
        let q = u16x6_at(payload, offset)?;
        if q[0] > q[3] || q[1] > q[4] || q[2] > q[5] {
            return Err(CollisionError(format!(
                "tile {index}: node {n} has an inverted AABB"
            )));
        }
        nodes.push(QuantizedNode {
            q_min: [q[0], q[1], q[2]],
            q_max: [q[3], q[4], q[5]],
            escape_or_triangle: i32_at(payload, offset + 12)?,
        });
    }
    let boundary = PAYLOAD_PREFIX + NODE_SIZE * node_count;
    let head = u16x6_at(payload, boundary)?;
    if head[0] > head[3] || head[1] > head[4] || head[2] > head[5] {
        return Err(CollisionError(format!(
            "tile {index}: node array does not end at a subtree"
        )));
    }
    let subtree_bytes = size - boundary - PAYLOAD_TRAILER;
    if subtree_bytes == 0 || subtree_bytes % SUBTREE_SIZE != 0 {
        return Err(CollisionError(format!(
            "tile {index}: subtree area {subtree_bytes} is misaligned"
        )));
    }
    let mut subtrees = Vec::with_capacity(subtree_bytes / SUBTREE_SIZE);
    for s in 0..subtree_bytes / SUBTREE_SIZE {
        let offset = boundary + SUBTREE_SIZE * s;
        let q = u16x6_at(payload, offset)?;
        let root_node_index = u32_at(payload, offset + 12)?;
        let subtree_size = u32_at(payload, offset + 16)?;
        if payload[offset + 20..offset + 32].iter().any(|b| *b != 0) {
            return Err(CollisionError(format!(
                "tile {index}: subtree {s} padding is not zero"
            )));
        }
        if q[0] > q[3] || q[1] > q[4] || q[2] > q[5] {
            return Err(CollisionError(format!(
                "tile {index}: subtree {s} has an inverted AABB"
            )));
        }
        if subtree_size == 0 || root_node_index + subtree_size > node_count as u32 {
            return Err(CollisionError(format!(
                "tile {index}: subtree {s} spans {root_node_index}..{}",
                root_node_index + subtree_size
            )));
        }
        subtrees.push(SubtreeHeader {
            q_min: [q[0], q[1], q[2]],
            q_max: [q[3], q[4], q[5]],
            root_node_index,
            subtree_size,
        });
    }
    Ok(Tile {
        index,
        payload_size: size,
        aabb_min,
        aabb_max,
        scale,
        nodes,
        subtrees,
    })
}
