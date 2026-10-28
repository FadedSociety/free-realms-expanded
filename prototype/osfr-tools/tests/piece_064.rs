//! Piece 064 (dungeon collision/reachability) tests: the recovered ForgeLight
//! `CDTA` collision container prefix and the render-vs-collision boundary.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (preserved client assets, hashes pinned in
//! `notes/Research/Pieces/064 Dungeon collision/reachability.md`):
//! * `sg_npx_cavern_01_room_01.adr` (pack_19/001776.adr) ends with the ASCII
//!   string `sg_npx_cavern_01_room_01.cdt`: the room actor binds its collision
//!   asset by name.
//! * Every cave room/door ships a `*.cdt` asset whose bytes begin `CDTA`.
//! * The parsed container prefix is `CDTA | u32 version=1 | u32 a | u32 b |
//!   u32 c | u32 vertex_count | f32[vertex_count*3] | u32 index_count |
//!   u16[index_count]`, where `a = 1, b = 1, c = 0` for the room/door assets
//!   (the water asset differs: `a = 8`), every index is `< vertex_count`, and
//!   the remaining tail bytes are not decoded by this piece.
//! * `check` on `pack_01/001632.cdt` (room_01): 2280 vertices, 3048 indices,
//!   1007 unique triangles of which 994 are byte-equal render triangles of the
//!   community reader's `sg_npx_cavern_01_room_01_lod0.dme` export.
//!
//! RECONSTRUCTED (labelled, not historical): the slope threshold used to
//! classify a triangle as walkable surface, and the "highest surface below the
//! probe" floor query. They are an analysis policy for the checker, not
//! recovered game physics.
//!
//! NOT CLAIMED: that the parsed prefix is the complete collision mesh, that a
//! route is natively walkable, or that GCNK collision payloads are decoded.
//! The 456 recorded native cave positions are compared with this module by
//! the private recovery checker `piece_064.py`, not here.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_064.rs"]
#[allow(dead_code)]
mod piece_064;

use osfr_tools::obstacles::MAX_SLOPE_DEGREES;
use piece_064::{Aabb, BoundsRelation, CdtaError, CdtaMesh, classify_bounds, horizontal_floor};

/// Build a synthetic CDTA container: header, vertices, index block, tail bytes.
fn synthetic(header: [u32; 3], vertices: &[[f32; 3]], indices: &[u16], tail: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"CDTA");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    for value in header {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&(vertices.len() as u32).to_le_bytes());
    for vertex in vertices {
        for component in vertex {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    bytes.extend_from_slice(&(indices.len() as u32).to_le_bytes());
    for index in indices {
        bytes.extend_from_slice(&index.to_le_bytes());
    }
    bytes.extend_from_slice(tail);
    bytes
}

/// Two triangles forming a small upward-facing floor plus a vertical wall.
fn floor_and_wall() -> (Vec<[f32; 3]>, Vec<u16>) {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 0.0, 4.0],
        [0.0, 0.0, 4.0],
        [0.0, 6.0, 0.0],
        [4.0, 6.0, 0.0],
        [0.0, 6.0, 4.0],
    ];
    let indices = vec![0u16, 1, 2, 0, 2, 3, 0, 1, 5, 0, 5, 4];
    (vertices, indices)
}

#[test]
fn parses_the_recovered_prefix_and_keeps_the_tail_unparsed() {
    let (vertices, indices) = floor_and_wall();
    let bytes = synthetic([1, 1, 0], &vertices, &indices, &[0xAA; 7]);
    let mesh = CdtaMesh::parse(&bytes).expect("valid prefix");
    assert_eq!(mesh.version, 1);
    assert_eq!(mesh.header, [1, 1, 0]);
    assert_eq!(mesh.vertices.len(), 7);
    assert_eq!(mesh.indices.len(), 12);
    assert_eq!(mesh.tail_len, 7);
    assert_eq!(mesh.triangle_count(), 4);
    assert_eq!(mesh.triangles().count(), 4);
}

#[test]
fn rejects_a_bad_magic_truncation_and_out_of_range_index() {
    let (vertices, indices) = floor_and_wall();
    let mut bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    bytes[0] = b'X';
    assert_eq!(CdtaMesh::parse(&bytes).unwrap_err(), CdtaError::BadMagic);

    let bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    let truncated = &bytes[..bytes.len() - 2];
    assert_eq!(
        CdtaMesh::parse(truncated).unwrap_err(),
        CdtaError::TruncatedIndices
    );

    let mut bad = indices.clone();
    bad[3] = 9;
    let bytes = synthetic([1, 1, 0], &vertices, &bad, &[]);
    assert_eq!(
        CdtaMesh::parse(&bytes).unwrap_err(),
        CdtaError::IndexOutOfRange {
            index: 9,
            vertices: 7
        }
    );
}

#[test]
fn rejects_an_unsupported_version() {
    let (vertices, indices) = floor_and_wall();
    let mut bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(
        CdtaMesh::parse(&bytes).unwrap_err(),
        CdtaError::UnsupportedVersion(2)
    );
}

#[test]
fn bounds_cover_every_vertex() {
    let (vertices, indices) = floor_and_wall();
    let bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    let mesh = CdtaMesh::parse(&bytes).unwrap();
    let bounds = mesh.bounds().expect("non-empty mesh");
    assert_eq!(
        bounds,
        Aabb {
            min: [0.0, 0.0, 0.0],
            max: [4.0, 6.0, 4.0],
        }
    );
}

#[test]
fn classifies_collision_bounds_against_render_bounds() {
    let render = Aabb {
        min: [0.0, 0.0, 0.0],
        max: [4.0, 6.0, 4.0],
    };
    let identical = Aabb {
        min: [0.0, 0.0, 0.0],
        max: [4.0, 6.0, 4.0],
    };
    let subset = Aabb {
        min: [0.0, 1.0, 1.0],
        max: [3.0, 5.0, 3.0],
    };
    let exceeds = Aabb {
        min: [-1.0, 0.0, 0.0],
        max: [4.0, 6.0, 4.0],
    };
    assert_eq!(
        classify_bounds(&identical, &render, 1e-4),
        BoundsRelation::Identical
    );
    assert_eq!(
        classify_bounds(&subset, &render, 1e-4),
        BoundsRelation::CollisionSubsetOfRender
    );
    assert_eq!(
        classify_bounds(&exceeds, &render, 1e-4),
        BoundsRelation::CollisionExceedsRender
    );
}

#[test]
fn floor_query_returns_the_highest_walkable_surface_below_the_probe() {
    let (vertices, indices) = floor_and_wall();
    let bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    let mesh = CdtaMesh::parse(&bytes).unwrap();
    // Inside the floor quad, above the floor but below the ceiling height.
    let floor = horizontal_floor(&mesh, 1.0, 1.0, 5.0, MAX_SLOPE_DEGREES).expect("floor surface");
    assert!(floor.abs() < 1e-6, "floor y {floor}");
    // Outside every triangle and beyond the wall there is no surface.
    assert!(horizontal_floor(&mesh, 20.0, 20.0, 5.0, MAX_SLOPE_DEGREES).is_none());
    // The probe below the floor cannot see it.
    assert!(horizontal_floor(&mesh, 1.0, 1.0, -1.0, MAX_SLOPE_DEGREES).is_none());
}

#[test]
fn floor_query_ignores_vertical_surfaces() {
    // Wall-only fixture: both triangles lie in the z = 0 plane, so the XZ
    // projection is degenerate and no walkable surface may be reported.
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 6.0, 0.0],
        [0.0, 6.0, 0.0],
    ];
    let indices = vec![0u16, 1, 2, 0, 2, 3];
    let bytes = synthetic([1, 1, 0], &vertices, &indices, &[]);
    let mesh = CdtaMesh::parse(&bytes).unwrap();
    assert!(horizontal_floor(&mesh, 2.0, 0.0, 5.0, MAX_SLOPE_DEGREES).is_none());
}

#[test]
fn horizontal_floor_ignores_the_unparsed_tail() {
    let (vertices, indices) = floor_and_wall();
    let tail = vec![0u8; 4096];
    let bytes = synthetic([1, 1, 0], &vertices, &indices, &tail);
    let mesh = CdtaMesh::parse(&bytes).unwrap();
    assert_eq!(mesh.tail_len, 4096);
    assert_eq!(
        horizontal_floor(&mesh, 1.0, 1.0, 5.0, MAX_SLOPE_DEGREES),
        Some(0.0)
    );
}
