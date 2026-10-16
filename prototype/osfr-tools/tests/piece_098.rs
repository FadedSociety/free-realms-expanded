//! Work item 098 acceptance: render-slice asset loading for a future replacement client.
//!
//! ORIGINAL facts used by these vectors (all preserved outside Git, see
//! `notes/Research/Pieces/098 Rust rendering/client feasibility.md`):
//! * The preserved corpus carries 2,344 `GCNK` terrain tiles, 5,969 `DMOD` models,
//!   865 `DSKE` skeletons, 8,631 `ADR` actors and 7,783 Granny `GR2` clips
//!   (`lab/extracted/client-assets/catalog.tsv`).
//! * Container versions actually present: `GCNK` 2-6, `DMOD` 2-3, `DSKE` 2.
//! * The unmodified pinned ForgeLightToolkit reader
//!   (`9c06714c5d79d9152c322814bc904abad944e06e`) produced the terrain numbers and
//!   the pinned FR ADR Toolkit (`5acfdf8d6006fa6f43d1d3605f076a6be7393e72`) produced
//!   the model/skeleton/animation-slot numbers the private gate below compares against.
//!
//! RECONSTRUCTED here: the loader slice itself, its bound checks and its refusal of
//! malformed/unsupported input. Nothing in this file renders, runs the native client or
//! proves gameplay; the private gate loads preserved bytes and compares container facts.
//!
//! This file is a `#[path]` include so it exercises the real module before the lead
//! registers `pub mod piece_098;` under `src/restoration/mod.rs`/`src/lib.rs`.

#[path = "../src/restoration/piece_098.rs"]
#[allow(dead_code)]
mod piece_098;

use piece_098::{
    GRANNY_MAGIC_POINTER32, GRANNY_MAGIC_POINTER64, LoadError, load_actor, load_model,
    load_skeleton, load_terrain, probe_granny,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// Environment variable used by the private gate: path to the oracle JSON produced by
/// `scripts/pieces/piece_098.py` from the preserved lab inputs.
const ORACLE_ENV: &str = "OSFR_RENDER_SLICE";

/// Optional override for the private gate's evidence output path; defaults to
/// `rust-slice-check.json` beside the oracle.
const ORACLE_OUT_ENV: &str = "OSFR_RENDER_SLICE_OUT";

fn zlib(data: &[u8]) -> Vec<u8> {
    miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}

fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend(value.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend(value.to_le_bytes());
}

fn push_str(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

fn push_compressed_length(out: &mut Vec<u8>, value: usize) {
    if value < 0x80 {
        out.push(value as u8);
    } else if value < 0x10000 {
        out.push(0x80 | ((value >> 8) as u8));
        out.push((value & 0xff) as u8);
    } else {
        out.push(0xff);
        out.extend((value as u32).to_le_bytes());
    }
}

fn push_section(out: &mut Vec<u8>, section_type: u8, payload: &[u8]) {
    out.push(section_type);
    push_compressed_length(out, payload.len());
    out.extend_from_slice(payload);
}

fn push_record(out: &mut Vec<u8>, record_type: u8, payload: &[u8]) {
    out.push(record_type);
    push_compressed_length(out, payload.len());
    out.extend_from_slice(payload);
}

/// One 32-byte terrain vertex in the pinned reader's `Vertex.Deserialize` layout.
/// Normal bytes decode as `byte / 127.5 - 1`.
fn terrain_vertex(position: [f32; 3], normal: [u8; 3]) -> Vec<u8> {
    let mut out = Vec::new();
    for value in position {
        push_f32(&mut out, value);
    }
    out.extend(normal);
    out.push(0);
    out.extend([10, 20, 30, 255]);
    out.extend([1, 2, 3, 255]);
    out.extend(16u16.to_le_bytes());
    out.extend(32u16.to_le_bytes());
    out.extend(64u16.to_le_bytes());
    out.extend(128u16.to_le_bytes());
    out
}

/// Options that let the rejection tests rebuild the same synthetic file with one
/// deliberately broken field instead of guessing byte offsets in a finished buffer.
#[derive(Default)]
struct GcnkOptions {
    version: i32,
    first_index: u16,
    first_position_x: f32,
    collision_expanded: Option<u32>,
}

/// Minimal `GCNK` file: one tile, one runtime object, one render batch, three vertices.
fn build_gcnk(options: GcnkOptions) -> Vec<u8> {
    let version = if options.version == 0 {
        6
    } else {
        options.version
    };
    let mut chunk = Vec::new();
    push_i32(&mut chunk, 1); // tile count
    push_i32(&mut chunk, 8);
    push_i32(&mut chunk, -32);
    for value in [0.0f32, 0.0, -1000.0, 1.0] {
        push_f32(&mut chunk, value);
    }
    push_i32(&mut chunk, 1); // Unknown7 > 0 -> four extra words
    for value in [1, 2, 3, 4] {
        push_i32(&mut chunk, value);
    }
    push_f32(&mut chunk, 0.25); // Unknown12
    push_i32(&mut chunk, 0); // eco data count
    push_i32(&mut chunk, 1); // runtime object count
    push_i32(&mut chunk, 7); // object Unknown
    push_str(&mut chunk, "sg_tree_giant_01.adr");
    push_str(&mut chunk, "alias");
    for value in [-2003.4474f32, -40.07, 544.33, 1.0] {
        push_f32(&mut chunk, value);
    }
    for value in [-2.79f32, 0.0, 0.0, 0.0] {
        push_f32(&mut chunk, value);
    }
    push_f32(&mut chunk, 1.5); // scale
    push_str(&mut chunk, "material_alias"); // version >= 6
    push_str(&mut chunk, "tint_alias"); // version >= 6
    for value in [0.1f32, 0.2, 0.3, 0.4] {
        push_f32(&mut chunk, value); // tint vector because alias is non-empty
    }
    push_i32(&mut chunk, 0); // skipped word
    push_i32(&mut chunk, 1254773387); // object id
    push_i32(&mut chunk, 0); // Unknown11
    push_i32(&mut chunk, 0); // unknown list count
    push_i32(&mut chunk, 0); // raw light count
    push_i32(&mut chunk, 0); // raw area count
    push_i32(&mut chunk, 0); // raw group count
    push_i32(&mut chunk, 9); // Unknown13
    push_i32(&mut chunk, 0); // trailing skipped word
    push_i32(&mut chunk, 1); // render batch count
    for value in [0, 3, 0, 3] {
        push_i32(&mut chunk, value);
    }
    push_i32(&mut chunk, 0); // height map bits per pixel
    let first_x = if options.first_position_x == 0.0 {
        1.0
    } else {
        options.first_position_x
    };
    push_i32(&mut chunk, 3); // index buffer count
    for index in [options.first_index, 1, 2] {
        chunk.extend(index.to_le_bytes());
    }
    push_i32(&mut chunk, 3); // vertex buffer count
    chunk.extend(terrain_vertex([first_x, 2.0, 3.0], [255, 128, 0]));
    chunk.extend(terrain_vertex([4.0, 5.0, 6.0], [128, 255, 128]));
    chunk.extend(terrain_vertex([7.0, 8.0, 9.0], [128, 128, 255]));

    let collision_payload = b"collision placeholder";
    let collision = zlib(collision_payload);
    let collision_expanded = options
        .collision_expanded
        .unwrap_or(collision_payload.len() as u32);

    let mut file = Vec::new();
    file.extend(b"GCNK");
    push_i32(&mut file, version);
    push_u32(&mut file, chunk.len() as u32);
    let compressed = zlib(&chunk);
    push_u32(&mut file, compressed.len() as u32);
    file.extend(compressed);
    push_u32(&mut file, collision_expanded);
    push_u32(&mut file, collision.len() as u32);
    file.extend(collision);
    file
}

fn synthetic_gcnk() -> Vec<u8> {
    build_gcnk(GcnkOptions::default())
}

fn synthetic_dmat() -> Vec<u8> {
    let mut dma = Vec::new();
    dma.extend(b"DMAT");
    push_u32(&mut dma, 1);
    let mut names = Vec::new();
    push_str(&mut names, "CHATDY.DDS");
    push_u32(&mut dma, names.len() as u32);
    dma.extend(names);
    push_u32(&mut dma, 1); // material count
    push_u32(&mut dma, 0x1234_5678);
    push_u32(&mut dma, 0); // parameter block size
    dma
}

fn skinned_vertex(position: [f32; 3], uv: [f32; 2]) -> Vec<u8> {
    let mut out = Vec::new();
    for value in position {
        push_f32(&mut out, value);
    }
    out.extend(
        [
            1.0f32.to_le_bytes(),
            0.0f32.to_le_bytes(),
            0.0f32.to_le_bytes(),
        ]
        .concat(),
    );
    out.extend([0u8, 0, 0, 0]);
    for value in [0.0f32, 1.0, 0.0] {
        push_f32(&mut out, value);
    }
    out.extend([255u8, 255, 255, 255]);
    push_f32(&mut out, uv[0]);
    push_f32(&mut out, uv[1]);
    out
}

/// Minimal `DMOD` version 3 file with one chunk and an embedded `DMAT`.
fn synthetic_dmod() -> Vec<u8> {
    let dma = synthetic_dmat();
    let mut file = Vec::new();
    file.extend(b"DMOD");
    push_u32(&mut file, 3);
    push_u32(&mut file, dma.len() as u32);
    file.extend(&dma);
    for value in [-1.0f32, -2.0, -3.0] {
        push_f32(&mut file, value);
    }
    for value in [1.0f32, 2.0, 3.0] {
        push_f32(&mut file, value);
    }
    push_u32(&mut file, 1); // mesh count
    for value in [0, 7, 24, -1] {
        push_i32(&mut file, value);
    }
    push_i32(&mut file, 52); // vertex stride
    push_i32(&mut file, 3); // vertex count
    push_i32(&mut file, 2); // index stride
    push_i32(&mut file, 3); // index count
    file.extend(skinned_vertex([0.0, 0.0, 0.0], [0.0, 0.0]));
    file.extend(skinned_vertex([1.0, 0.0, 0.0], [1.0, 0.0]));
    file.extend(skinned_vertex([0.0, 1.0, 0.0], [0.0, 1.0]));
    for index in [0u16, 1, 2] {
        file.extend(index.to_le_bytes());
    }
    file.extend(b"trailing");
    file
}

fn synthetic_dske() -> Vec<u8> {
    let mut names = Vec::new();
    push_str(&mut names, "ROOT");
    push_str(&mut names, "COG");
    let mut file = Vec::new();
    file.extend(b"DSKE");
    push_u32(&mut file, 2);
    push_u32(&mut file, names.len() as u32);
    file.extend(&names);
    file.extend([0u8, 1, 2, 3, 4, 5, 6, 7]); // hierarchy blob
    for bone in 0..2 {
        for value in 0..12 {
            push_f32(&mut file, (bone * 12 + value) as f32);
        }
    }
    file
}

fn synthetic_adr() -> Vec<u8> {
    let mut file = Vec::new();
    let mut skeleton = Vec::new();
    push_record(&mut skeleton, 1, b"treeble.dsk\0");
    push_section(&mut file, 1, &skeleton);

    let mut model = Vec::new();
    push_record(&mut model, 1, b"chatdy_lod0.dme\0");
    push_record(&mut model, 2, b"chatdy.dma\0");
    push_record(&mut model, 4, &0.65f32.to_be_bytes());
    push_section(&mut file, 2, &model);

    let mut bank = Vec::new();
    bank.extend([0xfe, 0x03]);
    for (slot, clip, speed) in [
        ("loc_stand", "chatdy_loc_stand.gr2", 3.0f32),
        ("loc_walk", "chatdy_loc_walk.gr2", 0.6666670f32),
    ] {
        bank.push(1);
        bank.push(slot.len() as u8);
        bank.extend_from_slice(slot.as_bytes());
        bank.push(0);
        bank.push(2);
        bank.push(clip.len() as u8);
        bank.extend_from_slice(clip.as_bytes());
        bank.push(0);
        bank.extend([4, 4]);
        bank.extend(speed.to_be_bytes());
    }
    push_section(&mut file, 9, &bank);
    file
}

fn synthetic_granny(pointer_size: u8, version: u32, sections: u32) -> Vec<u8> {
    let mut file = Vec::new();
    file.extend(match pointer_size {
        4 => GRANNY_MAGIC_POINTER32,
        8 => GRANNY_MAGIC_POINTER64,
        other => panic!("unsupported pointer size {other}"),
    });
    for _ in 0..4 {
        push_u32(&mut file, 0); // header words up to offset 32
    }
    push_u32(&mut file, version);
    push_u32(&mut file, 0); // file size word
    push_f32(&mut file, 2.0);
    push_u32(&mut file, 56); // section table offset relative to the header
    push_u32(&mut file, sections);
    push_u32(&mut file, 0); // root type section
    push_u32(&mut file, 0); // root type offset
    push_u32(&mut file, 0); // root object section
    push_u32(&mut file, 0); // root object offset
    file.resize(88, 0); // section table starts where the header says it does
    for index in 0..sections {
        push_u32(&mut file, 0); // format: uncompressed
        push_u32(&mut file, 88); // data offset
        push_u32(&mut file, 4 * (index + 1));
        push_u32(&mut file, 4 * (index + 1));
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
        push_u32(&mut file, 0);
    }
    file.resize(88 + sections as usize * 44 + 8, 0);
    file
}

fn oracle_path() -> Option<PathBuf> {
    std::env::var_os(ORACLE_ENV).map(PathBuf::from)
}

fn load_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap_or_else(|error| {
        panic!("cannot read {}: {error}", path.display());
    }))
    .expect("oracle JSON")
}

#[test]
fn terrain_chunk_loads_tiles_batches_and_vertices() {
    let tile = load_terrain(&synthetic_gcnk()).expect("synthetic GCNK loads");
    assert_eq!(tile.version, 6);
    assert_eq!(tile.tiles.len(), 1);
    assert_eq!(tile.tiles[0].coords, (8, -32));
    assert_eq!(tile.tiles[0].objects.len(), 1);
    let object = &tile.tiles[0].objects[0];
    assert_eq!(object.file_name, "sg_tree_giant_01.adr");
    assert_eq!(object.object_id, 1254773387);
    assert!((object.position[0] + 2003.4474).abs() < 0.001);
    assert_eq!(object.scale, 1.5);
    assert!(tile.height_map.is_none());
    assert_eq!(tile.render_batches.len(), 1);
    assert_eq!(tile.render_batches[0].index_count, 3);
    assert_eq!(tile.indices, vec![0, 1, 2]);
    assert_eq!(tile.vertices.len(), 3);
    assert_eq!(tile.vertices[0].position, [1.0, 2.0, 3.0]);
    assert_eq!(tile.vertices[0].uv0, [16, 32]);
    assert_eq!(tile.vertices[0].uv1, [64, 128]);
    assert!((tile.vertices[0].normal[0] - 1.0).abs() < 1e-6);
    assert!((tile.vertices[0].normal[2] + 1.0).abs() < 1e-6);
    assert!((tile.vertices[1].normal[1] - 1.0).abs() < 1e-6);
    assert!((tile.vertices[2].normal[2] - 1.0).abs() < 1e-6);
    assert_eq!(tile.coords, (8, -32));
}

#[test]
fn terrain_rejects_malformed_and_unsupported_input() {
    let mut bad_magic = synthetic_gcnk();
    bad_magic[0] = b'X';
    assert!(matches!(
        load_terrain(&bad_magic),
        Err(LoadError::BadMagic(_))
    ));

    let bad_version = build_gcnk(GcnkOptions {
        version: 9,
        ..Default::default()
    });
    assert!(matches!(
        load_terrain(&bad_version),
        Err(LoadError::UnsupportedVersion { version: 9 })
    ));

    let mut truncated = synthetic_gcnk();
    truncated.truncate(truncated.len() - 12);
    assert!(matches!(
        load_terrain(&truncated),
        Err(LoadError::Truncated(_))
    ));

    let mut bad_inflate = synthetic_gcnk();
    bad_inflate[16] ^= 0xff;
    assert!(matches!(
        load_terrain(&bad_inflate),
        Err(LoadError::Inflate(_))
    ));

    // Index buffer references a vertex the vertex buffer does not contain.
    let out_of_range = build_gcnk(GcnkOptions {
        first_index: 9,
        ..Default::default()
    });
    let result = load_terrain(&out_of_range);
    assert!(
        matches!(result, Err(LoadError::IndexOutOfRange(_))),
        "{result:?}"
    );

    // Non-finite terrain position must be refused before it reaches a renderer.
    let non_finite = build_gcnk(GcnkOptions {
        first_position_x: f32::NAN,
        ..Default::default()
    });
    assert!(matches!(
        load_terrain(&non_finite),
        Err(LoadError::NonFinite(_))
    ));

    // Collision payload that does not expand to its declared length.
    let bad_collision = build_gcnk(GcnkOptions {
        collision_expanded: Some(4096),
        ..Default::default()
    });
    assert!(matches!(
        load_terrain(&bad_collision),
        Err(LoadError::Inconsistent(_))
    ));
}

#[test]
fn model_loads_materials_chunks_and_bounds() {
    let model = load_model(&synthetic_dmod()).expect("synthetic DMOD loads");
    assert_eq!(model.version, 3);
    assert_eq!(model.textures, vec!["CHATDY.DDS".to_string()]);
    assert_eq!(model.materials.len(), 1);
    assert_eq!(model.materials[0].hash, 0x1234_5678);
    assert_eq!(model.bounds_min, [-1.0, -2.0, -3.0]);
    assert_eq!(model.bounds_max, [1.0, 2.0, 3.0]);
    assert_eq!(model.chunks.len(), 1);
    let chunk = &model.chunks[0];
    assert_eq!(chunk.vertex_stride, 52);
    assert_eq!(chunk.vertices.len(), 3);
    assert_eq!(chunk.indices, vec![0, 1, 2]);
    assert_eq!(chunk.vertices[2].uv, [0.0, 1.0]);
    assert_eq!(model.trailing_bytes, 8);
    assert!(model.material_count_matches());
}

#[test]
fn model_rejects_malformed_input() {
    let mut bad_magic = synthetic_dmod();
    bad_magic[0] = b'X';
    assert!(matches!(
        load_model(&bad_magic),
        Err(LoadError::BadMagic(_))
    ));

    let mut unsupported = synthetic_dmod();
    unsupported[4..8].copy_from_slice(&7u32.to_le_bytes());
    assert!(matches!(
        load_model(&unsupported),
        Err(LoadError::UnsupportedVersion { version: 7 })
    ));

    // Chunk header advertises one material more than the embedded DMAT carries.
    let mut mismatch = synthetic_dmod();
    let mesh_count_offset = 12 + synthetic_dmat().len() + 24;
    mismatch[mesh_count_offset..mesh_count_offset + 4].copy_from_slice(&2u32.to_le_bytes());
    assert!(matches!(
        load_model(&mismatch),
        Err(LoadError::Inconsistent(_))
    ));

    // Unsupported vertex stride.
    let mut stride = synthetic_dmod();
    let stride_offset = mesh_count_offset + 4 + 16;
    stride[stride_offset..stride_offset + 4].copy_from_slice(&48i32.to_le_bytes());
    assert!(matches!(
        load_model(&stride),
        Err(LoadError::UnsupportedLayout(_))
    ));

    // Index pointing past the decoded vertex list.
    let mut index = synthetic_dmod();
    let index_offset = index.len() - 8 - 6;
    index[index_offset..index_offset + 2].copy_from_slice(&7u16.to_le_bytes());
    assert!(matches!(
        load_model(&index),
        Err(LoadError::IndexOutOfRange(_))
    ));
}

#[test]
fn skeleton_loads_bones_and_bind_pose() {
    let skeleton = load_skeleton(&synthetic_dske()).expect("synthetic DSKE loads");
    assert_eq!(skeleton.version, 2);
    assert_eq!(
        skeleton.bone_names,
        vec!["ROOT".to_string(), "COG".to_string()]
    );
    assert_eq!(skeleton.bind_pose.len(), 2);
    assert_eq!(skeleton.bind_pose[1][0], 12.0);
    assert_eq!(skeleton.hierarchy_bytes, 8);
    assert!(skeleton.is_finite());
}

#[test]
fn skeleton_rejects_malformed_input() {
    let mut bad_magic = synthetic_dske();
    bad_magic[0] = b'X';
    assert!(matches!(
        load_skeleton(&bad_magic),
        Err(LoadError::BadMagic(_))
    ));

    // Bone-name block longer than the file.
    let mut oversized = synthetic_dske();
    oversized[8..12].copy_from_slice(&4096u32.to_le_bytes());
    assert!(matches!(
        load_skeleton(&oversized),
        Err(LoadError::Truncated(_))
    ));

    // No room left for the bind-pose block.
    let mut empty = Vec::new();
    empty.extend(b"DSKE");
    push_u32(&mut empty, 2);
    push_u32(&mut empty, 0);
    assert!(matches!(
        load_skeleton(&empty),
        Err(LoadError::Inconsistent(_))
    ));
}

#[test]
fn actor_loads_model_skeleton_and_animation_slots() {
    let actor = load_actor(&synthetic_adr()).expect("synthetic ADR loads");
    assert_eq!(actor.skeleton_file.as_deref(), Some("treeble.dsk"));
    assert_eq!(actor.mesh_file.as_deref(), Some("chatdy_lod0.dme"));
    assert_eq!(actor.material_file.as_deref(), Some("chatdy.dma"));
    assert_eq!(actor.model_scale, Some(0.65));
    assert_eq!(actor.animation_slots.len(), 2);
    assert_eq!(actor.animation_slots[0].slot_name, "loc_stand");
    assert_eq!(actor.animation_slots[0].gr2_file, "chatdy_loc_stand.gr2");
    assert_eq!(actor.animation_slots[0].speed, Some(3.0));
    assert!((actor.animation_slots[1].speed.unwrap() - 0.666667).abs() < 1e-5);
    assert!(actor.section_types.contains(&9));
}

#[test]
fn actor_rejects_malformed_input() {
    let mut truncated = synthetic_adr();
    truncated.truncate(truncated.len() - 5);
    assert!(matches!(
        load_actor(&truncated),
        Err(LoadError::Truncated(_))
    ));

    let mut bad_length = synthetic_adr();
    bad_length[1] = 0xff;
    bad_length[2..6].copy_from_slice(&0xffff_ffffu32.to_le_bytes());
    assert!(matches!(
        load_actor(&bad_length),
        Err(LoadError::Truncated(_))
    ));
}

#[test]
fn granny_probe_reports_container_header() {
    let clip = probe_granny(&synthetic_granny(4, 7, 2)).expect("pointer32 clip");
    assert_eq!(clip.pointer_size, 4);
    assert_eq!(clip.version, 7);
    assert_eq!(clip.section_count, 2);
    assert_eq!(clip.sections.len(), 2);
    assert_eq!(clip.sections[0].data_size, 4);
    assert_eq!(clip.sections[1].expanded_size, 8);

    let clip = probe_granny(&synthetic_granny(8, 6, 1)).expect("pointer64 clip");
    assert_eq!(clip.pointer_size, 8);
    assert_eq!(clip.version, 6);
    assert_eq!(clip.section_count, 1);
}

#[test]
fn granny_probe_rejects_malformed_input() {
    let mut bad_magic = synthetic_granny(4, 7, 1);
    bad_magic[0] = 0;
    assert!(matches!(
        probe_granny(&bad_magic),
        Err(LoadError::BadMagic(_))
    ));

    let mut truncated = synthetic_granny(4, 7, 1);
    truncated.truncate(40);
    assert!(matches!(
        probe_granny(&truncated),
        Err(LoadError::Truncated(_))
    ));

    // Section table claims a section count the file cannot hold.
    let mut oversized = synthetic_granny(4, 7, 1);
    oversized[48..52].copy_from_slice(&1024u32.to_le_bytes());
    assert!(matches!(probe_granny(&oversized), Err(LoadError::Limit(_))));

    // A section payload outside the file.
    let mut outside = synthetic_granny(4, 7, 1);
    let section_base = 32 + 56;
    outside[section_base + 4..section_base + 8].copy_from_slice(&0xffff_0000u32.to_le_bytes());
    assert!(matches!(
        probe_granny(&outside),
        Err(LoadError::Inconsistent(_))
    ));
}

#[test]
#[ignore = "requires the private preserved assets and oracle; set OSFR_RENDER_SLICE"]
fn preserved_assets_match_community_readers() {
    let Some(oracle_path) = oracle_path() else {
        panic!("set {ORACLE_ENV} to the piece 098 oracle JSON");
    };
    let oracle = load_json(&oracle_path);
    let mut report = json!({"piece": "098", "oracle": oracle_path.display().to_string()});

    let terrain_rows = oracle["terrain"].as_array().expect("terrain oracle rows");
    let mut terrain_report = Vec::new();
    for row in terrain_rows {
        let path = PathBuf::from(row["path"].as_str().expect("terrain path"));
        let bytes = fs::read(&path).expect("preserved terrain payload");
        let tile = load_terrain(&bytes).unwrap_or_else(|error| {
            panic!("{} failed to load: {error}", path.display());
        });
        assert_eq!(tile.version, row["version"].as_i64().unwrap() as i32);
        assert_eq!(tile.tiles.len(), row["tiles"].as_u64().unwrap() as usize);
        assert_eq!(
            tile.coords,
            (
                row["tile0_coords"][0].as_i64().unwrap() as i32,
                row["tile0_coords"][1].as_i64().unwrap() as i32
            )
        );
        assert_eq!(
            tile.vertices.len(),
            row["vertices"].as_u64().unwrap() as usize
        );
        assert_eq!(
            tile.indices.len(),
            row["indices"].as_u64().unwrap() as usize
        );
        let objects: usize = tile.tiles.iter().map(|entry| entry.objects.len()).sum();
        assert_eq!(objects, row["objects"].as_u64().unwrap() as usize);
        let first = &tile.tiles[0].objects[0];
        assert_eq!(
            first.file_name,
            row["first_object"]["file"].as_str().unwrap()
        );
        assert_eq!(
            first.object_id,
            row["first_object"]["object_id"].as_i64().unwrap() as i32
        );
        let expected = row["first_object"]["position"][0].as_f64().unwrap();
        assert!((first.position[0] as f64 - expected).abs() < 1e-3);
        terrain_report.push(json!({
            "name": row["name"],
            "version": tile.version,
            "tiles": tile.tiles.len(),
            "vertices": tile.vertices.len(),
            "indices": tile.indices.len(),
            "objects": objects,
            "render_batches": tile.render_batches.len(),
            "collision_bytes": tile.collision_bytes,
        }));
    }

    let actor_rows = oracle["actors"].as_array().expect("actor oracle rows");
    let mut actor_report = Vec::new();
    for row in actor_rows {
        let read =
            |key: &str| fs::read(row[key].as_str().expect(key)).expect("preserved actor input");
        let actor = load_actor(&read("adr")).expect("actor definition");
        assert_eq!(actor.mesh_file.as_deref(), row["mesh_file"].as_str());
        assert_eq!(
            actor.skeleton_file.as_deref(),
            row["skeleton_file"].as_str()
        );
        assert_eq!(
            actor.material_file.as_deref(),
            row["material_file"].as_str()
        );

        let model = load_model(&read("mesh")).expect("model");
        let vertices: usize = model.chunks.iter().map(|chunk| chunk.vertices.len()).sum();
        let indices: usize = model.chunks.iter().map(|chunk| chunk.indices.len()).sum();
        assert_eq!(model.version, row["mesh_version"].as_u64().unwrap() as u32);
        assert_eq!(model.chunks.len(), row["chunks"].as_u64().unwrap() as usize);
        assert_eq!(vertices, row["vertices"].as_u64().unwrap() as usize);
        assert_eq!(indices / 3, row["triangles"].as_u64().unwrap() as usize);
        for axis in 0..3 {
            let expected = row["bounds_min"][axis].as_f64().unwrap();
            assert!((model.bounds_min[axis] as f64 - expected).abs() < 1e-6);
            let expected = row["bounds_max"][axis].as_f64().unwrap();
            assert!((model.bounds_max[axis] as f64 - expected).abs() < 1e-6);
        }

        let skeleton = load_skeleton(&read("skeleton")).expect("skeleton");
        assert_eq!(
            skeleton.bone_names.len(),
            row["bones"].as_u64().unwrap() as usize
        );
        for (index, expected) in row["first_bones"].as_array().unwrap().iter().enumerate() {
            assert_eq!(Some(skeleton.bone_names[index].as_str()), expected.as_str());
        }

        let expected_slots = row["slots"].as_array().expect("animation slots");
        assert_eq!(actor.animation_slots.len(), expected_slots.len());
        for (slot, expected) in actor.animation_slots.iter().zip(expected_slots) {
            assert_eq!(Some(slot.slot_name.as_str()), expected["name"].as_str());
            assert_eq!(Some(slot.gr2_file.as_str()), expected["file"].as_str());
            match expected["speed"].as_f64() {
                Some(value) => {
                    let speed = slot.speed.expect("expected playback scalar");
                    assert!((speed as f64 - value).abs() < 1e-6, "{slot:?} vs {value}");
                }
                None => assert!(slot.speed.is_none()),
            }
        }

        let textures: Vec<&str> = model.textures.iter().map(String::as_str).collect();
        let expected: Vec<&str> = row["textures"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        assert_eq!(textures, expected);

        actor_report.push(json!({
            "name": row["name"],
            "mesh_version": model.version,
            "chunks": model.chunks.len(),
            "vertices": vertices,
            "triangles": indices / 3,
            "bones": skeleton.bone_names.len(),
            "animation_slots": actor.animation_slots.len(),
            "textures": textures,
        }));
    }

    let clip_rows = oracle["clips"].as_array().expect("clip oracle rows");
    let mut clip_report = Vec::new();
    for row in clip_rows {
        let path = row["path"].as_str().expect("clip path");
        let bytes = fs::read(path).expect("preserved clip");
        let clip = probe_granny(&bytes).unwrap_or_else(|error| panic!("{path}: {error}"));
        assert_eq!(
            clip.pointer_size,
            row["pointer_size"].as_u64().unwrap() as u8
        );
        assert_eq!(clip.version, row["version"].as_u64().unwrap() as u32);
        assert_eq!(
            clip.section_count,
            row["section_count"].as_u64().unwrap() as u32
        );
        assert_eq!(clip.sections.len(), clip.section_count as usize);
        clip_report.push(json!({
            "path": path,
            "pointer_size": clip.pointer_size,
            "version": clip.version,
            "section_count": clip.section_count,
        }));
    }

    report["terrain"] = Value::Array(terrain_report);
    report["actors"] = Value::Array(actor_report);
    report["clips"] = Value::Array(clip_report);
    let output = std::env::var_os(ORACLE_OUT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| oracle_path.with_file_name("rust-slice-check.json"));
    assert!(
        !output.exists(),
        "{} already exists; refusing to overwrite evidence",
        output.display()
    );
    fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).expect("write report");
    println!("wrote {}", output.display());
}
