//! 098: bounded render-slice asset loading for a future replacement client.
//!
//! ORIGINAL facts this module loads (all preserved outside Git; see
//! `notes/Research/Pieces/098 Rust rendering/client feasibility.md`):
//! * Terrain tiles are `GCNK`: magic, version 2-6, a zlib chunk and a zlib collision blob.
//!   The chunk holds tiles (coords, placed runtime objects, lights, areas, groups), render
//!   batches, an optional height map and the shared 32-byte vertex buffer. The layout here
//!   mirrors the unmodified pinned community reader
//!   `ForgeLightToolkit/Editor/FileTypes/Gcnk*.cs` at revision
//!   `9c06714c5d79d9152c322814bc904abad944e06e`.
//! * Models are `DMOD` version 2-3 with an embedded `DMAT` material block and 52-byte
//!   `ClrNrmUVSkin` / 36-byte `ClrNrmUV` vertex layouts, mirroring the pinned FR ADR Toolkit
//!   `fr_formats/{dme,dma}.py` at revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72`.
//! * Skeletons are `DSKE` version 2 (bone names plus 48-byte bind-pose rows at the tail) and
//!   actors are `ADR` section streams; animation slots (slot name, `.gr2` clip, big-endian
//!   playback scalar) are recovered with the same pattern scan as the pinned toolkit.
//! * Animation clips are Granny `GR2` containers. `probe_granny` reads only the container
//!   header described by the pinned community reader `carbon-granny`
//!   (`carbonenginejs/tools-blender` revision `732265cff0bbe4535dacb9297a1c0794a65ba267`):
//!   a 16-byte magic that selects the 4- or 8-byte pointer size, version, and the 44-byte
//!   section table.
//!
//! RECONSTRUCTED here: this loader slice, its limits and its refusal of malformed input.
//! NOT implemented, and deliberately left as gaps: Granny transform-curve evaluation,
//! GCNK collision semantics (the collision blob is only bounds-checked), the `GCNK` v2-v4
//! runtime-object layout (the pinned reader throws there), model material parameter
//! decoding, GPU upload, skinning and rendering. Nothing here runs the native client.
//!
//! INTEGRATION: this file is included by `tests/piece_098.rs` with `#[path]` until the lead
//! registers `pub mod piece_098;`. It has no dependencies beyond the crate's `miniz_oxide`.

use std::fmt;

/// Granny container magic for 4-byte pointer files (7,783 preserved clips use one of two).
pub const GRANNY_MAGIC_POINTER32: [u8; 16] = [
    0x29, 0xde, 0x6c, 0xc0, 0xba, 0xa4, 0x53, 0x2b, 0x25, 0xf5, 0xb7, 0xa5, 0xf6, 0x66, 0xe2, 0xee,
];

/// Granny container magic for 8-byte pointer files.
pub const GRANNY_MAGIC_POINTER64: [u8; 16] = [
    0xe5, 0x9b, 0x49, 0x5e, 0x6f, 0x63, 0x1f, 0x14, 0x1e, 0x13, 0xeb, 0xa9, 0x90, 0xbe, 0xed, 0xc4,
];

const MAX_INFLATED_BYTES: usize = 64 << 20;
const MAX_TERRAIN_TILES: i32 = 4096;
const MAX_TERRAIN_OBJECTS: i32 = 100_000;
const MAX_TERRAIN_VERTICES: i32 = 1_000_000;
const MAX_TERRAIN_INDICES: i32 = 4_000_000;
const MAX_TERRAIN_BATCHES: i32 = 4096;
const MAX_MESH_CHUNKS: i32 = 4096;
const MAX_MESH_VERTICES: i32 = 1_000_000;
const MAX_MESH_INDICES: i32 = 8_000_000;
const MAX_BONES: usize = 4096;
const MAX_ANIMATION_SLOTS: usize = 4096;
const MAX_GRANNY_SECTIONS: u32 = 64;
const MAX_SECTION_EXPANDED: u32 = 256 << 20;
const MAX_STRING_BYTES: usize = 1024;
const MAX_ACTOR_SECTIONS: usize = 4096;

/// Why a preserved byte string could not be loaded into a render-slice structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// The file ended before a declared structure did.
    Truncated(&'static str),
    /// A container magic does not match the recovered value.
    BadMagic(&'static str),
    /// A container version outside the recovered range.
    UnsupportedVersion { version: i32 },
    /// A layout this slice does not implement (for example a non-recovered vertex stride).
    UnsupportedLayout(&'static str),
    /// A count or size beyond the loader's bounded envelope.
    Limit(&'static str),
    /// The zlib stream failed to inflate.
    Inflate(&'static str),
    /// A float that is NaN or infinite; refuse it before it reaches a renderer.
    NonFinite(&'static str),
    /// An index outside its buffer, or a render batch outside its ranges.
    IndexOutOfRange(&'static str),
    /// Declared lengths or counts disagree with one another.
    Inconsistent(&'static str),
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Truncated(context) => write!(formatter, "truncated {context}"),
            LoadError::BadMagic(context) => write!(formatter, "unexpected magic for {context}"),
            LoadError::UnsupportedVersion { version } => {
                write!(formatter, "unsupported container version {version}")
            }
            LoadError::UnsupportedLayout(context) => {
                write!(formatter, "unsupported layout {context}")
            }
            LoadError::Limit(context) => {
                write!(formatter, "value beyond the loader envelope: {context}")
            }
            LoadError::Inflate(context) => write!(formatter, "zlib inflate failed for {context}"),
            LoadError::NonFinite(context) => write!(formatter, "non-finite float in {context}"),
            LoadError::IndexOutOfRange(context) => {
                write!(formatter, "index out of range in {context}")
            }
            LoadError::Inconsistent(context) => write!(formatter, "inconsistent {context}"),
        }
    }
}

impl std::error::Error for LoadError {}

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len() - self.offset
    }

    fn take(&mut self, count: usize, context: &'static str) -> Result<&'a [u8], LoadError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(LoadError::Truncated(context))?;
        if end > self.data.len() {
            return Err(LoadError::Truncated(context));
        }
        let slice = &self.data[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn skip(&mut self, count: usize, context: &'static str) -> Result<(), LoadError> {
        self.take(count, context).map(|_| ())
    }

    fn u8(&mut self, context: &'static str) -> Result<u8, LoadError> {
        Ok(self.take(1, context)?[0])
    }

    fn u16(&mut self, context: &'static str) -> Result<u16, LoadError> {
        let bytes = self.take(2, context)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn i32(&mut self, context: &'static str) -> Result<i32, LoadError> {
        Ok(self.u32(context)? as i32)
    }

    fn u32(&mut self, context: &'static str) -> Result<u32, LoadError> {
        let bytes = self.take(4, context)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn f32(&mut self, context: &'static str) -> Result<f32, LoadError> {
        let bytes = self.take(4, context)?;
        let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if !value.is_finite() {
            return Err(LoadError::NonFinite(context));
        }
        Ok(value)
    }

    fn vec3(&mut self, context: &'static str) -> Result<[f32; 3], LoadError> {
        Ok([self.f32(context)?, self.f32(context)?, self.f32(context)?])
    }

    fn vec4(&mut self, context: &'static str) -> Result<[f32; 4], LoadError> {
        Ok([
            self.f32(context)?,
            self.f32(context)?,
            self.f32(context)?,
            self.f32(context)?,
        ])
    }

    /// Null-terminated ASCII string, mirroring the pinned readers' byte-wise read.
    fn string(&mut self, context: &'static str) -> Result<String, LoadError> {
        let window = self
            .data
            .len()
            .saturating_sub(self.offset)
            .min(MAX_STRING_BYTES);
        let end = self.data[self.offset..self.offset + window]
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(if window == MAX_STRING_BYTES {
                LoadError::Limit(context)
            } else {
                LoadError::Truncated(context)
            })?;
        let bytes = self.take(end, context)?;
        self.skip(1, context)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    /// The pinned readers' three-case compressed length: one byte, two bytes, or `0xff` + u32.
    fn compressed_length(&mut self, context: &'static str) -> Result<usize, LoadError> {
        let first = self.u8(context)?;
        if first < 0x80 {
            return Ok(first as usize);
        }
        if first != 0xff {
            let second = self.u8(context)?;
            return Ok((((first & 0x7f) as usize) << 8) | second as usize);
        }
        Ok(self.u32(context)? as usize)
    }
}

fn inflate_bounded(
    bytes: &[u8],
    expanded: usize,
    context: &'static str,
) -> Result<Vec<u8>, LoadError> {
    if expanded == 0 || expanded > MAX_INFLATED_BYTES {
        return Err(LoadError::Limit(context));
    }
    let inflated = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(bytes, expanded)
        .map_err(|_| LoadError::Inflate(context))?;
    if inflated.len() != expanded {
        return Err(LoadError::Inconsistent(context));
    }
    Ok(inflated)
}

// --- terrain -------------------------------------------------------------------------

/// One 32-byte `GCNK` terrain vertex. Colour bytes are kept in file order; the pinned reader
/// maps the first group as (r, b, g, a).
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [u8; 4],
    pub color2: [u8; 4],
    pub uv0: [u16; 2],
    pub uv1: [u16; 2],
}

/// One contiguous index range over the shared terrain vertex buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderBatch {
    pub index_offset: i32,
    pub index_count: i32,
    pub vertex_offset: i32,
    pub vertex_count: i32,
}

/// A placed object recorded in a terrain tile (its `.adr`/`.agr` name and world transform).
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainObject {
    pub file_name: String,
    pub alias: String,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: f32,
    pub material_name: Option<String>,
    pub tint_alias: Option<String>,
    pub object_id: i32,
    pub unknown12_entries: usize,
}

/// One `.gcnk` tile record: coordinates, position and everything placed on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainRecord {
    pub coords: (i32, i32),
    pub position: [f32; 4],
    pub objects: Vec<TerrainObject>,
    pub raw_lights: usize,
    pub raw_areas: usize,
    pub raw_groups: usize,
    pub unknown13: i32,
}

/// Height-map metadata. The payload is skipped, not decoded; the pinned reader leaves it as
/// a TODO as well.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeightMap {
    pub bits_per_pixel: i32,
    pub size: i32,
}

/// A loaded `GCNK` terrain file: the CPU-side mesh plus its placement records.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainTile {
    pub version: i32,
    pub coords: (i32, i32),
    pub position: [f32; 4],
    pub tiles: Vec<TerrainRecord>,
    pub render_batches: Vec<RenderBatch>,
    pub height_map: Option<HeightMap>,
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u16>,
    pub collision_bytes: usize,
    pub trailing_bytes: usize,
}

fn read_terrain_object(cursor: &mut Cursor<'_>, version: i32) -> Result<TerrainObject, LoadError> {
    cursor.i32("GCNK object type")?;
    let file_name = cursor.string("GCNK object name")?;
    let alias = cursor.string("GCNK object alias")?;
    let position = cursor.vec4("GCNK object position")?;
    let rotation = cursor.vec4("GCNK object rotation")?;
    let scale = cursor.f32("GCNK object scale")?;
    let mut material_name = None;
    let mut tint_alias = None;
    if version >= 6 {
        let material = cursor.string("GCNK object material")?;
        let tint = cursor.string("GCNK object tint alias")?;
        if !tint.is_empty() {
            cursor.vec4("GCNK object tint vector")?;
        }
        material_name = Some(material);
        tint_alias = Some(tint);
    }
    cursor.skip(4, "GCNK object separator")?;
    if version <= 4 {
        // The pinned reader has no recovered layout for v2-v4 runtime objects.
        return Err(LoadError::UnsupportedVersion { version });
    }
    let object_id = cursor.i32("GCNK object id")?;
    cursor.i32("GCNK object unknown")?;
    let mut unknown12_entries = 0;
    if version > 2 {
        let count = cursor.i32("GCNK object list count")?;
        if !(0..=MAX_TERRAIN_OBJECTS).contains(&count) {
            return Err(LoadError::Limit("GCNK object list count"));
        }
        for _ in 0..count {
            let size = cursor.i32("GCNK object list entry size")?;
            if size < 0 {
                return Err(LoadError::Inconsistent("GCNK object list entry size"));
            }
            cursor.skip(size as usize * 4, "GCNK object list entry")?;
            unknown12_entries += 1;
        }
    }
    Ok(TerrainObject {
        file_name,
        alias,
        position,
        rotation,
        scale,
        material_name,
        tint_alias,
        object_id,
        unknown12_entries,
    })
}

fn read_terrain_record(cursor: &mut Cursor<'_>, version: i32) -> Result<TerrainRecord, LoadError> {
    let coords = (cursor.i32("GCNK tile x")?, cursor.i32("GCNK tile y")?);
    let position = cursor.vec4("GCNK tile position")?;
    let unknown7 = cursor.i32("GCNK tile tag")?;
    if unknown7 > 0 {
        for _ in 0..4 {
            cursor.i32("GCNK tile bounds")?;
        }
    }
    cursor.f32("GCNK tile scalar")?;
    let eco_count = cursor.i32("GCNK eco data count")?;
    if !(0..=MAX_TERRAIN_OBJECTS).contains(&eco_count) {
        return Err(LoadError::Limit("GCNK eco data count"));
    }
    cursor.skip(eco_count as usize * 4, "GCNK eco data")?;

    let object_count = cursor.i32("GCNK object count")?;
    if !(0..=MAX_TERRAIN_OBJECTS).contains(&object_count) {
        return Err(LoadError::Limit("GCNK object count"));
    }
    let mut objects = Vec::with_capacity(object_count as usize);
    for _ in 0..object_count {
        objects.push(read_terrain_object(cursor, version)?);
    }

    let light_count = cursor.i32("GCNK light count")?;
    if !(0..=MAX_TERRAIN_OBJECTS).contains(&light_count) {
        return Err(LoadError::Limit("GCNK light count"));
    }
    for _ in 0..light_count {
        cursor.string("GCNK light name")?;
        cursor.string("GCNK light colour name")?;
        cursor.u8("GCNK light type")?;
        cursor.vec4("GCNK light position")?;
        cursor.f32("GCNK light range")?;
        cursor.f32("GCNK light intensity")?;
        cursor.skip(4, "GCNK light colour")?;
    }

    let area_count = cursor.i32("GCNK area count")?;
    if !(0..=MAX_TERRAIN_OBJECTS).contains(&area_count) {
        return Err(LoadError::Limit("GCNK area count"));
    }
    for _ in 0..area_count {
        cursor.string("GCNK area name")?;
        cursor.i32("GCNK area kind")?;
        cursor.string("GCNK area alias")?;
        cursor.vec4("GCNK area bounds")?;
        cursor.vec4("GCNK area bounds 2")?;
        cursor.i32("GCNK area flag")?;
        cursor.vec3("GCNK area vector")?;
    }

    let group_count = cursor.i32("GCNK group count")?;
    if !(0..=MAX_TERRAIN_OBJECTS).contains(&group_count) {
        return Err(LoadError::Limit("GCNK group count"));
    }
    for _ in 0..group_count {
        cursor.string("GCNK group name")?;
        cursor.vec4("GCNK group bounds")?;
        cursor.vec4("GCNK group bounds 2")?;
        cursor.i32("GCNK group flag")?;
    }

    let unknown13 = cursor.i32("GCNK tile unknown")?;
    cursor.skip(4, "GCNK tile trailing word")?;
    Ok(TerrainRecord {
        coords,
        position,
        objects,
        raw_lights: light_count as usize,
        raw_areas: area_count as usize,
        raw_groups: group_count as usize,
        unknown13,
    })
}

fn read_terrain_vertex(cursor: &mut Cursor<'_>) -> Result<TerrainVertex, LoadError> {
    let position = cursor.vec3("GCNK vertex position")?;
    let normal = [
        cursor.u8("GCNK vertex normal")? as f32 / 127.5 - 1.0,
        cursor.u8("GCNK vertex normal")? as f32 / 127.5 - 1.0,
        cursor.u8("GCNK vertex normal")? as f32 / 127.5 - 1.0,
    ];
    cursor.skip(1, "GCNK vertex padding")?;
    let mut color = [0u8; 4];
    for slot in color.iter_mut() {
        *slot = cursor.u8("GCNK vertex colour")?;
    }
    let mut color2 = [0u8; 4];
    for slot in color2.iter_mut() {
        *slot = cursor.u8("GCNK vertex colour 2")?;
    }
    let uv0 = [cursor.u16("GCNK vertex uv")?, cursor.u16("GCNK vertex uv")?];
    let uv1 = [
        cursor.u16("GCNK vertex uv 2")?,
        cursor.u16("GCNK vertex uv 2")?,
    ];
    Ok(TerrainVertex {
        position,
        normal,
        color,
        color2,
        uv0,
        uv1,
    })
}

/// Load a preserved `GCNK` terrain tile into CPU-side mesh and placement data.
pub fn load_terrain(bytes: &[u8]) -> Result<TerrainTile, LoadError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(4, "GCNK magic")? != b"GCNK" {
        return Err(LoadError::BadMagic("GCNK"));
    }
    let version = cursor.i32("GCNK version")?;
    if !(0..=6).contains(&version) {
        return Err(LoadError::UnsupportedVersion { version });
    }
    let chunk_expanded = cursor.u32("GCNK chunk length")? as usize;
    let chunk_stored = cursor.u32("GCNK chunk stored length")? as usize;
    let stored = cursor.take(chunk_stored, "GCNK chunk payload")?;
    let chunk = inflate_bounded(stored, chunk_expanded, "GCNK chunk")?;

    let collision_expanded = cursor.u32("GCNK collision length")? as usize;
    let collision_stored = cursor.u32("GCNK collision stored length")? as usize;
    let collision = cursor.take(collision_stored, "GCNK collision payload")?;
    let collision_bytes = inflate_bounded(collision, collision_expanded, "GCNK collision")?.len();

    let mut chunk = Cursor::new(&chunk);
    let tile_count = chunk.i32("GCNK tile count")?;
    if !(0..=MAX_TERRAIN_TILES).contains(&tile_count) {
        return Err(LoadError::Limit("GCNK tile count"));
    }
    let mut tiles = Vec::with_capacity(tile_count as usize);
    for _ in 0..tile_count {
        tiles.push(read_terrain_record(&mut chunk, version)?);
    }
    let coords = tiles.first().map(|tile| tile.coords).unwrap_or((0, 0));
    let position = tiles.first().map(|tile| tile.position).unwrap_or([0.0; 4]);

    let batch_count = chunk.i32("GCNK render batch count")?;
    if !(0..=MAX_TERRAIN_BATCHES).contains(&batch_count) {
        return Err(LoadError::Limit("GCNK render batch count"));
    }
    let mut render_batches = Vec::with_capacity(batch_count as usize);
    for _ in 0..batch_count {
        render_batches.push(RenderBatch {
            index_offset: chunk.i32("GCNK batch index offset")?,
            index_count: chunk.i32("GCNK batch index count")?,
            vertex_offset: chunk.i32("GCNK batch vertex offset")?,
            vertex_count: chunk.i32("GCNK batch vertex count")?,
        });
    }

    let height_bits = chunk.i32("GCNK height map bits")?;
    let mut height_map = None;
    if height_bits > 0 && version > 4 {
        let size = chunk.i32("GCNK height map size")?;
        if size < 0 || height_bits > 64 {
            return Err(LoadError::Inconsistent("GCNK height map"));
        }
        chunk.skip(
            size as usize * size as usize * height_bits as usize / 2,
            "GCNK height map payload",
        )?;
        height_map = Some(HeightMap {
            bits_per_pixel: height_bits,
            size,
        });
    }

    let index_count = chunk.i32("GCNK index count")?;
    if !(0..=MAX_TERRAIN_INDICES).contains(&index_count) {
        return Err(LoadError::Limit("GCNK index count"));
    }
    let mut indices = Vec::with_capacity(index_count as usize);
    for _ in 0..index_count {
        indices.push(chunk.u16("GCNK index")?);
    }

    let vertex_count = chunk.i32("GCNK vertex count")?;
    if !(0..=MAX_TERRAIN_VERTICES).contains(&vertex_count) {
        return Err(LoadError::Limit("GCNK vertex count"));
    }
    let mut vertices = Vec::with_capacity(vertex_count as usize);
    for _ in 0..vertex_count {
        vertices.push(read_terrain_vertex(&mut chunk)?);
    }

    for batch in &render_batches {
        if batch.index_offset < 0
            || batch.index_count < 0
            || batch.vertex_offset < 0
            || batch.index_offset + batch.index_count > indices.len() as i32
        {
            return Err(LoadError::IndexOutOfRange("GCNK render batch"));
        }
        for index in
            &indices[batch.index_offset as usize..(batch.index_offset + batch.index_count) as usize]
        {
            if *index as i64 + batch.vertex_offset as i64 >= vertices.len() as i64 {
                return Err(LoadError::IndexOutOfRange("GCNK render batch index"));
            }
        }
    }

    Ok(TerrainTile {
        version,
        coords,
        position,
        tiles,
        render_batches,
        height_map,
        vertices,
        indices,
        collision_bytes,
        trailing_bytes: chunk.remaining(),
    })
}

// --- models --------------------------------------------------------------------------

/// One decoded model vertex; unused fields stay at their defaults for the 36-byte layout.
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_weights: [f32; 3],
    pub bone_indices: [u8; 4],
    pub color: [u8; 4],
}

/// One `DMOD` mesh chunk with its material index and decoded buffers.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshChunk {
    pub material_index: i32,
    pub unknown: [i32; 3],
    pub vertex_stride: i32,
    pub index_stride: i32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// One embedded `DMAT` material row. Parameters are counted, not decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialEntry {
    pub hash: u32,
    pub parameter_bytes: usize,
}

/// A loaded `DMOD` model: embedded materials, bounds, chunks and trailing metadata size.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub version: u32,
    pub textures: Vec<String>,
    pub materials: Vec<MaterialEntry>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub chunks: Vec<MeshChunk>,
    pub trailing_bytes: usize,
}

impl Model {
    /// The pinned readers require one material row per mesh chunk.
    pub fn material_count_matches(&self) -> bool {
        self.chunks.len() == self.materials.len()
    }
}

fn parse_materials(bytes: &[u8]) -> Result<(Vec<String>, Vec<MaterialEntry>), LoadError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(4, "DMAT magic")? != b"DMAT" {
        return Err(LoadError::BadMagic("DMAT"));
    }
    let version = cursor.u32("DMAT version")?;
    if version != 1 {
        return Err(LoadError::UnsupportedVersion {
            version: version as i32,
        });
    }
    let texture_bytes = cursor.u32("DMAT texture block length")? as usize;
    let textures_end = cursor
        .offset
        .checked_add(texture_bytes)
        .ok_or(LoadError::Truncated("DMAT texture block"))?;
    if textures_end > bytes.len() {
        return Err(LoadError::Truncated("DMAT texture block"));
    }
    let mut textures = Vec::new();
    while cursor.offset < textures_end {
        textures.push(cursor.string("DMAT texture name")?);
    }
    let material_count = cursor.u32("DMAT material count")?;
    if material_count > MAX_MESH_CHUNKS as u32 {
        return Err(LoadError::Limit("DMAT material count"));
    }
    let mut materials = Vec::with_capacity(material_count as usize);
    for _ in 0..material_count {
        let hash = cursor.u32("DMAT material hash")?;
        let parameter_bytes = cursor.u32("DMAT parameter length")? as usize;
        cursor.skip(parameter_bytes, "DMAT parameter block")?;
        materials.push(MaterialEntry {
            hash,
            parameter_bytes,
        });
    }
    Ok((textures, materials))
}

fn read_mesh_vertex(cursor: &mut Cursor<'_>, stride: i32) -> Result<Vertex, LoadError> {
    match stride {
        52 => {
            let position = cursor.vec3("DMOD vertex position")?;
            let bone_weights = cursor.vec3("DMOD vertex weights")?;
            let mut bone_indices = [0u8; 4];
            for slot in bone_indices.iter_mut() {
                *slot = cursor.u8("DMOD vertex bone indices")?;
            }
            let normal = cursor.vec3("DMOD vertex normal")?;
            let mut color = [0u8; 4];
            for slot in color.iter_mut() {
                *slot = cursor.u8("DMOD vertex colour")?;
            }
            let uv = [cursor.f32("DMOD vertex uv")?, cursor.f32("DMOD vertex uv")?];
            Ok(Vertex {
                position,
                normal,
                uv,
                bone_weights,
                bone_indices,
                color,
            })
        }
        36 => {
            let position = cursor.vec3("DMOD vertex position")?;
            let normal = cursor.vec3("DMOD vertex normal")?;
            let mut color = [0u8; 4];
            for slot in color.iter_mut() {
                *slot = cursor.u8("DMOD vertex colour")?;
            }
            let uv = [cursor.f32("DMOD vertex uv")?, cursor.f32("DMOD vertex uv")?];
            Ok(Vertex {
                position,
                normal,
                uv,
                bone_weights: [1.0, 0.0, 0.0],
                bone_indices: [0, 0, 0, 0],
                color,
            })
        }
        _ => Err(LoadError::UnsupportedLayout("DMOD vertex stride")),
    }
}

/// Load a preserved `DMOD` model (versions 2 and 3; the corpus carries no version 4).
pub fn load_model(bytes: &[u8]) -> Result<Model, LoadError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(4, "DMOD magic")? != b"DMOD" {
        return Err(LoadError::BadMagic("DMOD"));
    }
    let version = cursor.u32("DMOD version")?;
    if !(2..=3).contains(&version) {
        return Err(LoadError::UnsupportedVersion {
            version: version as i32,
        });
    }
    let material_bytes = cursor.u32("DMOD material block length")? as usize;
    let material_block = cursor.take(material_bytes, "DMOD material block")?;
    let (textures, materials) = parse_materials(material_block)?;

    let bounds_min = cursor.vec3("DMOD bounds min")?;
    let bounds_max = cursor.vec3("DMOD bounds max")?;
    let mesh_count = if version >= 3 {
        cursor.u32("DMOD mesh count")?
    } else {
        materials.len() as u32
    };
    if mesh_count > MAX_MESH_CHUNKS as u32 {
        return Err(LoadError::Limit("DMOD mesh count"));
    }
    if mesh_count as usize != materials.len() {
        return Err(LoadError::Inconsistent("DMOD mesh count against materials"));
    }

    let mut chunks = Vec::with_capacity(mesh_count as usize);
    for _ in 0..mesh_count {
        let material_index = cursor.i32("DMOD material index")?;
        let unknown = [
            cursor.i32("DMOD chunk tag")?,
            cursor.i32("DMOD chunk tag")?,
            cursor.i32("DMOD chunk tag")?,
        ];
        let vertex_stride = cursor.i32("DMOD vertex stride")?;
        let vertex_count = cursor.i32("DMOD vertex count")?;
        let index_stride = cursor.i32("DMOD index stride")?;
        let index_count = cursor.i32("DMOD index count")?;
        if !(0..=MAX_MESH_VERTICES).contains(&vertex_count)
            || !(0..=MAX_MESH_INDICES).contains(&index_count)
            || vertex_stride < 0
            || index_stride < 0
        {
            return Err(LoadError::Limit("DMOD chunk buffer size"));
        }
        let vertex_bytes = cursor.take(
            vertex_stride as usize * vertex_count as usize,
            "DMOD vertex buffer",
        )?;
        let index_bytes = cursor.take(
            index_stride as usize * index_count as usize,
            "DMOD index buffer",
        )?;
        if !matches!(index_stride, 2 | 4) {
            return Err(LoadError::UnsupportedLayout("DMOD index stride"));
        }
        let mut vertices = Cursor::new(vertex_bytes);
        let mut decoded = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            decoded.push(read_mesh_vertex(&mut vertices, vertex_stride)?);
        }
        let mut indices = Vec::with_capacity(index_count as usize);
        for offset in 0..index_count as usize {
            let start = offset * index_stride as usize;
            let value = if index_stride == 2 {
                u16::from_le_bytes([index_bytes[start], index_bytes[start + 1]]) as u32
            } else {
                u32::from_le_bytes([
                    index_bytes[start],
                    index_bytes[start + 1],
                    index_bytes[start + 2],
                    index_bytes[start + 3],
                ])
            };
            if value as usize >= decoded.len() {
                return Err(LoadError::IndexOutOfRange("DMOD chunk index"));
            }
            indices.push(value);
        }
        chunks.push(MeshChunk {
            material_index,
            unknown,
            vertex_stride,
            index_stride,
            vertices: decoded,
            indices,
        });
    }

    Ok(Model {
        version,
        textures,
        materials,
        bounds_min,
        bounds_max,
        chunks,
        trailing_bytes: cursor.remaining(),
    })
}

// --- skeletons and actors --------------------------------------------------------------

/// A loaded `DSKE` skeleton: bone names, tail bind-pose rows and the unexplained middle blob.
#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    pub version: u32,
    pub bone_names: Vec<String>,
    pub bind_pose: Vec<[f32; 12]>,
    pub hierarchy_bytes: usize,
}

impl Skeleton {
    pub fn is_finite(&self) -> bool {
        self.bind_pose
            .iter()
            .all(|rows| rows.iter().all(|value| value.is_finite()))
    }
}

/// Load a preserved `DSKE` skeleton.
pub fn load_skeleton(bytes: &[u8]) -> Result<Skeleton, LoadError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(4, "DSKE magic")? != b"DSKE" {
        return Err(LoadError::BadMagic("DSKE"));
    }
    let version = cursor.u32("DSKE version")?;
    let names_size = cursor.u32("DSKE name block length")? as usize;
    let names_end = cursor
        .offset
        .checked_add(names_size)
        .ok_or(LoadError::Truncated("DSKE name block"))?;
    if names_end > bytes.len() {
        return Err(LoadError::Truncated("DSKE name block"));
    }
    let mut bone_names = Vec::new();
    while cursor.offset < names_end {
        if bone_names.len() == MAX_BONES {
            return Err(LoadError::Limit("DSKE bone count"));
        }
        bone_names.push(cursor.string("DSKE bone name")?);
    }
    let tail = bone_names.len() * 48;
    if bytes.len() < names_end + tail {
        return Err(LoadError::Inconsistent("DSKE bind-pose block"));
    }
    let matrix_start = bytes.len() - tail;
    if matrix_start <= names_end {
        return Err(LoadError::Inconsistent("DSKE bind-pose block"));
    }
    let hierarchy_bytes = matrix_start - names_end;
    for name in &bone_names {
        if name.is_empty() {
            return Err(LoadError::Inconsistent("DSKE bone name"));
        }
    }
    let mut matrices = Cursor::new(&bytes[matrix_start..]);
    let mut bind_pose = Vec::with_capacity(bone_names.len());
    for _ in 0..bone_names.len() {
        let mut rows = [0f32; 12];
        for value in rows.iter_mut() {
            *value = matrices.f32("DSKE bind pose")?;
        }
        bind_pose.push(rows);
    }
    Ok(Skeleton {
        version,
        bone_names,
        bind_pose,
        hierarchy_bytes,
    })
}

/// One `ADR` animation binding: slot name, `.gr2` clip and the recovered playback scalar.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationSlot {
    pub slot_name: String,
    pub gr2_file: String,
    pub speed: Option<f32>,
}

/// A loaded `ADR` actor definition.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Actor {
    pub skeleton_file: Option<String>,
    pub mesh_file: Option<String>,
    pub material_file: Option<String>,
    pub model_scale: Option<f32>,
    pub animation_slots: Vec<AnimationSlot>,
    pub section_types: Vec<u8>,
}

fn read_records(data: &[u8]) -> Vec<(u8, &[u8])> {
    let mut cursor = Cursor::new(data);
    let mut records = Vec::new();
    while cursor.remaining() > 0 {
        let Ok(record_type) = cursor.u8("ADR record type") else {
            break;
        };
        let Ok(size) = cursor.compressed_length("ADR record length") else {
            break;
        };
        let Ok(payload) = cursor.take(size, "ADR record payload") else {
            break;
        };
        records.push((record_type, payload));
    }
    records
}

fn payload_string(payload: &[u8]) -> String {
    let end = payload
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(payload.len());
    String::from_utf8_lossy(&payload[..end]).into_owned()
}

/// The pinned toolkit's animation-bank scan: `0x01` length-prefixed slot name, `0x02`
/// length-prefixed `.gr2` name, optional `0x04 0x04` big-endian playback scalar.
pub fn scan_animation_slots(section: &[u8]) -> Result<Vec<AnimationSlot>, LoadError> {
    let printable = |byte: u8| (0x20..=0x7e).contains(&byte);
    let mut slots: Vec<AnimationSlot> = Vec::new();
    let mut index = 0;
    while index < section.len() {
        if section[index] != 0x01 {
            index += 1;
            continue;
        }
        let Some(length_byte) = section.get(index + 1).copied() else {
            break;
        };
        if length_byte > 0x7f {
            index += 1;
            continue;
        }
        let name_start = index + 2;
        let mut name_end = name_start;
        while name_end < section.len() && printable(section[name_end]) && name_end - name_start < 64
        {
            name_end += 1;
        }
        if name_end == name_start || name_end - name_start >= 64 {
            index += 1;
            continue;
        }
        if section.get(name_end) != Some(&0x00) || section.get(name_end + 1) != Some(&0x02) {
            index += 1;
            continue;
        }
        let Some(file_length) = section.get(name_end + 2).copied() else {
            break;
        };
        if file_length > 0x7f {
            index += 1;
            continue;
        }
        let file_start = name_end + 3;
        let mut file_end = file_start;
        while file_end < section.len() && printable(section[file_end]) && file_end - file_start < 96
        {
            file_end += 1;
        }
        if file_end - file_start < 4 || file_end - file_start >= 96 {
            index += 1;
            continue;
        }
        let file = &section[file_start..file_end];
        if !file.ends_with(b".gr2") || section.get(file_end) != Some(&0x00) {
            index += 1;
            continue;
        }
        let slot_name = String::from_utf8_lossy(&section[name_start..name_end]).into_owned();
        let gr2_file = String::from_utf8_lossy(file).into_owned();
        let mut speed = None;
        let mut next = file_end + 1;
        if section.get(next) == Some(&0x04) && section.get(next + 1) == Some(&0x04) {
            let bytes = section
                .get(next + 2..next + 6)
                .ok_or(LoadError::Truncated("ADR playback scalar"))?;
            let value = f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            if !value.is_finite() {
                return Err(LoadError::NonFinite("ADR playback scalar"));
            }
            speed = Some(value);
            next += 6;
        }
        if slots
            .iter()
            .all(|slot| slot.slot_name != slot_name || slot.gr2_file != gr2_file)
        {
            if slots.len() == MAX_ANIMATION_SLOTS {
                return Err(LoadError::Limit("ADR animation slots"));
            }
            slots.push(AnimationSlot {
                slot_name,
                gr2_file,
                speed,
            });
        }
        index = next;
    }
    Ok(slots)
}

/// Load a preserved `ADR` actor definition (skeleton/mesh/material block and animation bank).
pub fn load_actor(bytes: &[u8]) -> Result<Actor, LoadError> {
    let mut cursor = Cursor::new(bytes);
    let mut actor = Actor::default();
    while cursor.remaining() > 0 {
        if actor.section_types.len() == MAX_ACTOR_SECTIONS {
            return Err(LoadError::Limit("ADR section count"));
        }
        let section_type = cursor.u8("ADR section type")?;
        let size = cursor.compressed_length("ADR section length")?;
        let payload = cursor.take(size, "ADR section payload")?;
        actor.section_types.push(section_type);
        match section_type {
            1 => {
                let records = read_records(payload);
                let source = records
                    .iter()
                    .find(|(record_type, _)| *record_type == 1)
                    .map(|(_, record)| *record)
                    .unwrap_or(payload);
                let name = payload_string(source);
                if !name.is_empty() {
                    actor.skeleton_file = Some(name);
                }
            }
            2 => {
                for (record_type, record) in read_records(payload) {
                    match record_type {
                        1 => actor.mesh_file = Some(payload_string(record)),
                        2 => actor.material_file = Some(payload_string(record)),
                        4 if record.len() == 4 => {
                            let value =
                                f32::from_be_bytes([record[0], record[1], record[2], record[3]]);
                            if value.is_finite() {
                                actor.model_scale = Some(value);
                            }
                        }
                        _ => {}
                    }
                }
            }
            9 => {
                actor.animation_slots.extend(scan_animation_slots(payload)?);
            }
            _ => {}
        }
    }
    Ok(actor)
}

// --- Granny containers -----------------------------------------------------------------

/// One 44-byte Granny section-table entry (fields unused by this slice are still recorded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrannySection {
    pub format: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub expanded_size: u32,
    pub pointer_fixup_offset: u32,
    pub pointer_fixup_count: u32,
}

/// The container header of one preserved animation clip. Curves are not decoded here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrannyContainer {
    pub pointer_size: u8,
    pub version: u32,
    pub section_count: u32,
    pub sections: Vec<GrannySection>,
    pub bytes: usize,
}

fn u32_at(bytes: &[u8], offset: usize, context: &'static str) -> Result<u32, LoadError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or(LoadError::Truncated(context))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

/// Read the Granny 2 container header so a caller can confirm a clip is loadable before
/// committing to curve decoding.
pub fn probe_granny(bytes: &[u8]) -> Result<GrannyContainer, LoadError> {
    let magic: [u8; 16] = bytes
        .get(..16)
        .ok_or(LoadError::Truncated("GR2 magic"))?
        .try_into()
        .map_err(|_| LoadError::Truncated("GR2 magic"))?;
    let pointer_size = if magic == GRANNY_MAGIC_POINTER32 {
        4u8
    } else if magic == GRANNY_MAGIC_POINTER64 {
        8u8
    } else {
        return Err(LoadError::BadMagic("GR2 magic"));
    };
    bytes.get(..68).ok_or(LoadError::Truncated("GR2 header"))?;
    let version = u32_at(bytes, 32, "GR2 version")?;
    let table_offset = 32usize
        .checked_add(u32_at(bytes, 44, "GR2 section table offset")? as usize)
        .ok_or(LoadError::Inconsistent("GR2 section table"))?;
    let section_count = u32_at(bytes, 48, "GR2 section count")?;
    if section_count > MAX_GRANNY_SECTIONS {
        return Err(LoadError::Limit("GR2 section count"));
    }
    let mut sections = Vec::with_capacity(section_count as usize);
    for index in 0..section_count as usize {
        let base = table_offset + index * 44;
        let header = bytes
            .get(base..base + 44)
            .ok_or(LoadError::Truncated("GR2 section table"))?;
        let section = GrannySection {
            format: u32::from_le_bytes([header[0], header[1], header[2], header[3]]),
            data_offset: u32::from_le_bytes([header[4], header[5], header[6], header[7]]),
            data_size: u32::from_le_bytes([header[8], header[9], header[10], header[11]]),
            expanded_size: u32::from_le_bytes([header[12], header[13], header[14], header[15]]),
            pointer_fixup_offset: u32::from_le_bytes([
                header[28], header[29], header[30], header[31],
            ]),
            pointer_fixup_count: u32::from_le_bytes([
                header[32], header[33], header[34], header[35],
            ]),
        };
        let end = (section.data_offset as usize)
            .checked_add(section.data_size as usize)
            .ok_or(LoadError::Inconsistent("GR2 section payload"))?;
        if end > bytes.len() {
            return Err(LoadError::Inconsistent("GR2 section payload"));
        }
        if section.expanded_size > MAX_SECTION_EXPANDED {
            return Err(LoadError::Limit("GR2 section expanded size"));
        }
        sections.push(section);
    }
    Ok(GrannyContainer {
        pointer_size,
        version,
        section_count,
        sections,
        bytes: bytes.len(),
    })
}
