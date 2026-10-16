//! Piece 080 (world collision data) tests: recovered GCNK collision-section schema,
//! quantised BVH nodes, subtree headers and the column query.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (preserved first-party inputs):
//! * `lab/extracted/client-assets/pack_01/001956.gcnk` (FabledRealms_4_-32,
//!   SHA256 8c3cad84fdf029a4d14c08c2a93791b37aa98b30154a1c8cb496b672ddef5abd):
//!   16 collision payloads, first payload 15408 bytes with 931 nodes and 9 subtree
//!   headers, per-axis scale equal to 65535 / (max - min).
//! * The original client binary ships `btQuantizedBvhFloatData`,
//!   `btQuantizedBvhNodeData`, `btBvhSubtreeInfoData`, `btOptimizedBvh::build` and
//!   `AppPhysics::CollisionData(Asset)`, and its grounding path casts `Y+2` to
//!   `Y-2000` through collision query `00fd4290` with filter `0x143`.
//!
//! RECONSTRUCTED for the synthetic fixture: every byte of `synthetic_section()`.
//! RECORDED, not re-observed here: the native world-ready position used by the
//! column assertion (run 20261008T072258Z).
//!
//! NOT CLAIMED: exact triangle hit heights, the triangle/vertex payload, native
//! acceptance, or any gameplay effect.
#[path = "../src/restoration/piece_080.rs"]
#[allow(dead_code)]
mod piece_080;

use piece_080::{
    CollisionError, CollisionSection, PAYLOAD_PREFIX, PAYLOAD_TRAILER, QUANT_MAX, SUBTREE_SIZE,
    TILE_CELL, TILE_COUNT,
};
use std::path::Path;

const NATIVE_WORLD_READY: (f32, f32, f32) = (-1904.883_057, -39.709_801, 412.602_386);

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Synthetic payload: 208-byte prefix, three nodes (root + two leaves) and one
/// subtree header. Every byte here is a test fixture, not recovered data.
fn synthetic_payload(
    min: [f32; 3],
    span: [f32; 3],
    leaf_min: [u16; 3],
    leaf_max: [u16; 3],
) -> Vec<u8> {
    let node_count = 3usize;
    let total = PAYLOAD_PREFIX + 16 * node_count + SUBTREE_SIZE + PAYLOAD_TRAILER;
    let mut out = Vec::with_capacity(total);
    push_u32(&mut out, total as u32);
    out.extend_from_slice(&[0u8; 28]);
    for axis in 0..3 {
        push_f32(&mut out, min[axis]);
    }
    push_f32(&mut out, 0.0);
    for axis in 0..3 {
        push_f32(&mut out, min[axis] + span[axis]);
    }
    push_f32(&mut out, 0.0);
    for axis in 0..3 {
        push_f32(&mut out, QUANT_MAX / span[axis]);
    }
    push_f32(&mut out, 0.0);
    out.resize(PAYLOAD_PREFIX, 0);
    // root
    for value in [0u16, 0, 0, 65535, 65535, 65535] {
        push_u16(&mut out, value);
    }
    push_i32(&mut out, -(node_count as i32));
    // leaf 1
    for value in leaf_min {
        push_u16(&mut out, value);
    }
    for value in leaf_max {
        push_u16(&mut out, value);
    }
    push_i32(&mut out, 7);
    // leaf 2
    for value in [30000u16, 0, 30000, 65535, 65535, 65535] {
        push_u16(&mut out, value);
    }
    push_i32(&mut out, 8);
    // subtree header covering both leaves
    for value in leaf_min {
        push_u16(&mut out, value);
    }
    for value in leaf_max {
        push_u16(&mut out, value);
    }
    push_u32(&mut out, 1);
    push_u32(&mut out, 2);
    out.extend_from_slice(&[0u8; 12]);
    // payload trailer
    out.extend_from_slice(&[0u8; PAYLOAD_TRAILER]);
    assert_eq!(out.len(), total);
    out
}

/// Synthetic 16-tile section: a 4x4 grid of 66-unit cells, each with the same
/// three-node fixture tree. The leaf column covers the cell's north-west corner.
fn synthetic_section() -> Vec<u8> {
    let mut out = Vec::new();
    for iz in 0..4 {
        for ix in 0..4 {
            let min = [ix as f32 * TILE_CELL, -60.0, iz as f32 * TILE_CELL];
            let span = [TILE_CELL, 20.0, TILE_CELL];
            let payload = synthetic_payload(min, span, [0, 0, 0], [4000, 20000, 4000]);
            out.extend_from_slice(&payload);
            // 16-byte trailer block that follows every payload.
            push_u32(&mut out, ix as u32 * 4 + iz as u32);
            out.extend_from_slice(&[0u8; 12]);
        }
    }
    out
}

#[test]
fn synthetic_section_matches_the_recovered_schema() {
    let section = CollisionSection::parse(&synthetic_section()).expect("synthetic section parses");
    assert_eq!(section.tiles.len(), TILE_COUNT);
    for tile in &section.tiles {
        assert_eq!(tile.nodes.len(), 3);
        assert_eq!(tile.leaf_count(), 2);
        assert_eq!(tile.subtrees.len(), 1);
        assert_eq!(tile.subtrees[0].root_node_index, 1);
        assert_eq!(tile.subtrees[0].subtree_size, 2);
        assert!((tile.span()[0] - TILE_CELL).abs() < 1e-4);
        assert!((tile.span()[2] - TILE_CELL).abs() < 1e-4);
        assert!(tile.nodes[0].escape_index().is_some());
        assert_eq!(tile.nodes[1].triangle_index(), Some(7));
    }
}

#[test]
fn synthetic_column_query_reports_leaf_bands_only() {
    let section = CollisionSection::parse(&synthetic_section()).expect("synthetic section parses");
    // A column inside tile (ix=0, iz=0)'s leaf box (q <= 4000 -> 4.03 world units wide).
    let hits = section.column(1.0, 1.0);
    assert_eq!(hits.len(), 1, "exactly one leaf covers the column corner");
    let hit = hits[0];
    assert_eq!(hit.triangle_index, 7);
    assert!(hit.y_min >= -60.0 && hit.y_max <= -40.0 + 1e-3);
    // Outside every cell: no columns.
    assert!(section.column(-500.0, -500.0).is_empty());
    // The south-east leaf answers the far corner of the same cell.
    let corner = section.column(60.0, 60.0);
    assert_eq!(corner.len(), 1);
    assert_eq!(corner[0].triangle_index, 8);
    // Inside the cell but in the gap between both leaf boxes: no column.
    assert!(section.column(15.0, 15.0).is_empty());
}

#[test]
fn malformed_sections_are_rejected() {
    let good = synthetic_section();
    assert!(CollisionSection::parse(&good[..good.len() - 1]).is_err());

    let mut bad_size = good.clone();
    bad_size[0..4].copy_from_slice(&123u32.to_le_bytes());
    assert!(matches!(
        CollisionSection::parse(&bad_size),
        Err(CollisionError(_))
    ));

    let mut bad_scale = good.clone();
    bad_scale[64..68].copy_from_slice(&1.0f32.to_le_bytes());
    assert!(matches!(
        CollisionSection::parse(&bad_scale),
        Err(CollisionError(_))
    ));

    let mut bad_trailer = good;
    let last = bad_trailer.len() - 4;
    bad_trailer[last..].copy_from_slice(&7u32.to_le_bytes());
    assert!(matches!(
        CollisionSection::parse(&bad_trailer),
        Err(CollisionError(_))
    ));
}

#[test]
fn preserved_original_chunk_parses_with_the_same_schema() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(piece_080::STARTING_CHUNK);
    if !path.is_file() {
        eprintln!("preserved chunk not present, skipping: {}", path.display());
        return;
    }
    let bytes = std::fs::read(&path).expect("read preserved chunk");
    assert_eq!(bytes.len(), piece_080::STARTING_CHUNK_BYTES);
    let (version, terrain, section) = CollisionSection::from_gcnk(&bytes).expect("parse GCNK");
    assert_eq!(version, 6);
    assert_eq!(u32::from_le_bytes(terrain[0..4].try_into().unwrap()), 16);
    assert_eq!(section.tiles.len(), TILE_COUNT);
    let first = &section.tiles[0];
    assert_eq!(first.payload_size, 15408);
    assert_eq!(first.nodes.len(), 931);
    assert_eq!(first.subtrees.len(), 9);
    assert_eq!(first.subtrees[0].root_node_index, 4);
    assert_eq!(first.subtrees[0].subtree_size, 81);
    assert!((first.span()[0] - TILE_CELL).abs() < 0.05);
    assert!((first.span()[2] - TILE_CELL).abs() < 0.05);

    let (x, y, z) = NATIVE_WORLD_READY;
    let hits = section.column(x, z);
    assert!(
        !hits.is_empty(),
        "collision has a leaf column at the native start position"
    );
    assert!(
        hits.iter()
            .any(|h| h.y_min - 4.0 <= y && y <= h.y_max + 4.0),
        "the recorded native Y sits inside a collision leaf band"
    );
}
