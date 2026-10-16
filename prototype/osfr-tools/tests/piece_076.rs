//! Work item 076 acceptance: source-backed regional actor placements, zone membership and
//! cross-zone identifier / asset validation.
//!
//! Evidence behind the vectors below (see
//! `notes/Research/Pieces/076 Regional NPC and spawn content.md`):
//!
//! RECOVERED (pinned community sources, AGPL-3.0, read as facts, all outside this test):
//! * `Sanctuary/src/Scripts/Zone/FabledRealms.lua` - 3426
//!   `zone:spawnNpcWithGuid(npcId, guid, x, y, z, heading)` placements for world zone 1;
//!   3421 use `100000000000 + npc id`, four use the npc row's `CaptureGuid`, one (33081)
//!   matches neither.
//! * `Sanctuary/src/Resources/Zones/FabledRealms.json` - zone id 1, tile size 64, and the
//!   nine `AreaDefinitions` embedded in `REGION_AREAS`.
//! * `Sanctuary/src/Resources/Npcs.json` - one row per spawned npc id; four rows carry a
//!   `TerrainObjectId` with `ModelId` 0 (interaction actors bound to terrain objects).
//! * `lab/extracted/client-assets/catalog.tsv` - 737 of the 793 distinct model files resolve;
//!   the other 56 are explicit gaps.
//! * `FreeRealms-Legacy/src/Scripts/Zone/FabledRealms.lua` (4253 placements) and its
//!   `Npcs.json` - the second pinned revision for the cross-revision identifier check.
//!
//! RECONSTRUCTED policy: membership is inferred from the recovered area shapes; a
//! contradiction breaks the catalogue while a finding is reported and reviewed. Nothing here
//! spawns an actor or claims native acceptance.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the
//! lead registers `pub mod piece_076;` under `src/restoration/`.
#[path = "../src/restoration/piece_076.rs"]
#[allow(dead_code)]
mod piece_076;

use piece_076::{
    AREA_COUNT, ASSET, AssetBinding, CROSS_REVISION_MODEL_CONFLICTS, CROSS_REVISION_SHARED_IDS,
    GuidStatus, LEGACY_PLACEMENTS, NPC_GUID_BASE, NpcRow, Place, PlacementIssue,
    RECOVERED_CAPTURE_GUIDS, RECOVERED_CONVENTIONAL_GUIDS, RECOVERED_DISTINCT_NPC_IDS,
    RECOVERED_MODEL_ASSETS, RECOVERED_PLACEMENTS, RECOVERED_SAMPLE_PLACEMENTS,
    RECOVERED_SAMPLE_ROWS, RECOVERED_TERRAIN_BOUND_INTERACTION_NPCS, RECOVERED_UNEXPECTED_GUIDS,
    RECOVERED_UNRESOLVED_MODEL_ASSETS, REGION_AREAS, Severity, ZONE_ID, ZONE_NAME, ZONE_TILE_SIZE,
    area_contains, asset_binding, audit, classify_guid, guid_for_npc, model_conflicts, regions_at,
};

fn row(npc_id: i32) -> NpcRow {
    *RECOVERED_SAMPLE_ROWS
        .iter()
        .find(|row| row.id == npc_id)
        .expect("recovered sample row")
}

#[test]
fn zone_record_and_area_table_keep_the_recovered_values() {
    assert_eq!(ZONE_ID, 1);
    assert_eq!(ZONE_NAME, "FabledRealms");
    assert_eq!(ZONE_TILE_SIZE, 64);
    assert_eq!(REGION_AREAS.len(), AREA_COUNT);
    assert_eq!(AREA_COUNT, 9);
    let names: Vec<&str> = REGION_AREAS.iter().map(|area| area.name).collect();
    for expected in [
        "Blackspore",
        "Briarwood",
        "Merryvale",
        "Sanctuary",
        "Seaside",
        "Shrouded Glades",
        "Snowhill",
        "Wugachug",
        "Sunstone Valley",
    ] {
        assert!(names.contains(&expected), "missing region {expected}");
    }
    assert_eq!(
        REGION_AREAS.iter().map(|area| area.id).collect::<Vec<_>>(),
        vec![2, 3, 4, 5, 6, 7, 8, 9, 10]
    );
    let seaside = REGION_AREAS
        .iter()
        .find(|area| area.name == "Seaside")
        .unwrap();
    assert!(area_contains(seaside, -955.8561, -1170.239));
    assert!(!area_contains(seaside, -1418.3218, 347.82056));
}

#[test]
fn region_membership_matches_the_recovered_placements() {
    let sanctuary = regions_at([-1418.3218, -27.002932, 347.82056]);
    assert_eq!(
        sanctuary.iter().map(|area| area.name).collect::<Vec<_>>(),
        vec!["Sanctuary"]
    );

    let snowhill = regions_at([50.92852, 32.425617, 370.30267]);
    assert_eq!(
        snowhill.iter().map(|area| area.name).collect::<Vec<_>>(),
        vec!["Snowhill"]
    );

    let sunstone = regions_at([501.101013, 69.9525299, 1779.1543]);
    assert_eq!(
        sunstone.iter().map(|area| area.name).collect::<Vec<_>>(),
        vec!["Sunstone Valley"]
    );

    let blackspore = regions_at([-2128.67554, -32.8866158, 1044.89343]);
    assert_eq!(
        blackspore.iter().map(|area| area.name).collect::<Vec<_>>(),
        vec!["Blackspore"]
    );

    assert!(regions_at([-1270.01453, -49.5988922, 785.111572]).is_empty());
}

#[test]
fn guid_convention_and_capture_guids_are_classified_against_the_npc_table() {
    assert_eq!(guid_for_npc(1201), 100000001201);
    assert_eq!(NPC_GUID_BASE, 100_000_000_000);
    assert_eq!(
        classify_guid(1201, 100000001201, None),
        GuidStatus::Conventional
    );
    assert_eq!(
        classify_guid(33167, 14293651161170, Some(14293651161170)),
        GuidStatus::CaptureGuid
    );
    assert_eq!(
        classify_guid(33081, 100000033190, None),
        GuidStatus::Unexpected
    );
    assert_eq!(
        classify_guid(1201, 100000001202, None),
        GuidStatus::Unexpected
    );
    assert_eq!(
        classify_guid(36143, 13125420056642, Some(13125420056642)),
        GuidStatus::CaptureGuid
    );
}

#[test]
fn terrain_bound_interaction_npcs_resolve_to_a_terrain_object_instead_of_a_model() {
    let warpstone = row(36143);
    assert_eq!(warpstone.model_id, 0);
    assert!(warpstone.model_file.is_none());
    assert_eq!(
        asset_binding(&warpstone),
        Some(AssetBinding::TerrainObject(1346725607))
    );

    let model = row(1201);
    assert_eq!(
        asset_binding(&model),
        Some(AssetBinding::Model(ASSET.dwarf_therin))
    );

    let unbound = NpcRow {
        id: 1,
        model_id: 0,
        model_file: None,
        terrain_object_id: None,
        capture_guid: None,
    };
    assert_eq!(asset_binding(&unbound), None);
}

#[test]
fn audit_reproduces_the_recovered_identifier_and_region_counts() {
    let resolved = [
        ASSET.dwarf_therin,
        ASSET.painting_chaos_laughter,
        ASSET.warpstone,
    ];
    let audit = audit(
        RECOVERED_SAMPLE_PLACEMENTS,
        RECOVERED_SAMPLE_ROWS,
        &resolved,
    );

    assert_eq!(RECOVERED_PLACEMENTS, 3426);
    assert_eq!(RECOVERED_DISTINCT_NPC_IDS, 3426);
    assert_eq!(RECOVERED_CONVENTIONAL_GUIDS, 3421);
    assert_eq!(RECOVERED_CAPTURE_GUIDS, 4);
    assert_eq!(RECOVERED_UNEXPECTED_GUIDS, &[33081]);
    assert_eq!(
        RECOVERED_TERRAIN_BOUND_INTERACTION_NPCS,
        &[36143, 36307, 36308, 36309]
    );
    assert_eq!(RECOVERED_MODEL_ASSETS, 793);
    assert_eq!(RECOVERED_UNRESOLVED_MODEL_ASSETS, 56);

    // 1201 and the three conventional terrain-bound warpstones follow the recovered base+id
    // convention; the four capture-guid rows carry their npc row's CaptureGuid.
    assert_eq!(audit.conventional, 4);
    assert_eq!(audit.capture, 4);
    assert_eq!(audit.unexpected, vec![33081]);
    assert_eq!(audit.terrain_bound, vec![36143, 36307, 36308, 36309]);
    assert_eq!(audit.issues, vec![PlacementIssue::UnexpectedGuid(33081)]);
    assert!(audit.contradictions().is_empty());
    assert_eq!(audit.findings().len(), 1);
    assert_eq!(audit.findings()[0].severity(), Severity::Finding);
    assert!(audit.unresolved_models.is_empty());
    assert_eq!(audit.outside_declared_areas, 3);
    assert_eq!(audit.region_counts.get("Sunstone Valley"), Some(&2));
    assert_eq!(audit.region_counts.get("Seaside"), Some(&1));
    assert_eq!(audit.region_counts.get("Blackspore"), Some(&1));
    assert_eq!(audit.region_counts.get("Sanctuary"), Some(&1));
    assert_eq!(audit.region_counts.get("Snowhill"), Some(&1));
}

#[test]
fn audit_reports_unresolved_assets_as_gaps_not_success() {
    let audit = audit(RECOVERED_SAMPLE_PLACEMENTS, RECOVERED_SAMPLE_ROWS, &[]);
    assert_eq!(
        audit.unresolved_models,
        vec![
            ASSET.dwarf_therin,
            ASSET.painting_chaos_laughter,
            ASSET.warpstone
        ]
    );
    assert_eq!(audit.findings().len(), 1);
    assert!(audit.contradictions().is_empty());
}

#[test]
fn audit_rejects_duplicate_guids_unknown_rows_and_unbound_rows() {
    let placements = [
        Place {
            npc_id: 1201,
            guid: 100000001201,
            position: [-955.8, -44.6, -1170.2],
            heading: 2.3,
        },
        Place {
            npc_id: 1203,
            guid: 100000001201,
            position: [-956.3, -44.7, -1171.8],
            heading: 2.2,
        },
        Place {
            npc_id: 999999,
            guid: 1000000999999,
            position: [0.0, 0.0, 0.0],
            heading: 0.0,
        },
        Place {
            npc_id: 36307,
            guid: 100000036307,
            position: [-1270.0, -49.6, 785.1],
            heading: -0.4,
        },
        Place {
            npc_id: 1202,
            guid: 100000001202,
            position: [-955.9, -44.6, -1170.2],
            heading: 2.4,
        },
    ];
    let rows = [
        NpcRow {
            id: 1201,
            model_id: 46,
            model_file: Some(ASSET.dwarf_therin),
            terrain_object_id: None,
            capture_guid: None,
        },
        NpcRow {
            id: 1202,
            model_id: 0,
            model_file: None,
            terrain_object_id: None,
            capture_guid: None,
        },
        NpcRow {
            id: 1203,
            model_id: 280,
            model_file: Some(ASSET.warpstone),
            terrain_object_id: None,
            capture_guid: None,
        },
        NpcRow {
            id: 36307,
            model_id: 0,
            model_file: None,
            terrain_object_id: Some(1254775835),
            capture_guid: None,
        },
    ];
    let audit = audit(&placements, &rows, &[ASSET.dwarf_therin, ASSET.warpstone]);
    assert!(
        audit
            .issues
            .contains(&PlacementIssue::DuplicateGuid(100000001201))
    );
    // The second placement carrying 1201's guid also names 1201's npc row, so the
    // collision is reported next to the duplicate.
    assert!(audit.issues.contains(&PlacementIssue::GuidCollision {
        npc_id: 1203,
        aliases: 1201
    }));
    assert!(
        audit
            .issues
            .contains(&PlacementIssue::UnknownNpcRow(999999))
    );
    assert!(
        audit
            .issues
            .contains(&PlacementIssue::MissingModelBinding(1202))
    );
    assert_eq!(audit.unknown_ids, vec![999999]);
    assert_eq!(audit.contradictions().len(), 4);
    assert!(audit.findings().is_empty());
}

#[test]
fn guid_aliasing_another_recovered_npc_is_a_contradiction() {
    let placements = [Place {
        npc_id: 33081,
        guid: 100000001201,
        position: [-1933.2, -43.6, -133.9],
        heading: 1.4,
    }];
    let rows = [
        NpcRow {
            id: 33081,
            model_id: 942,
            model_file: Some(ASSET.painting_chaos_laughter),
            terrain_object_id: None,
            capture_guid: None,
        },
        NpcRow {
            id: 1201,
            model_id: 46,
            model_file: Some(ASSET.dwarf_therin),
            terrain_object_id: None,
            capture_guid: None,
        },
    ];
    let audit = audit(&placements, &rows, &[ASSET.painting_chaos_laughter]);
    assert!(audit.issues.contains(&PlacementIssue::GuidCollision {
        npc_id: 33081,
        aliases: 1201
    }));
    assert_eq!(audit.contradictions().len(), 1);
    assert!(audit.findings().is_empty());
}

#[test]
fn cross_revision_comparison_keeps_one_model_per_recovered_npc_id() {
    assert_eq!(LEGACY_PLACEMENTS, 4253);
    assert_eq!(CROSS_REVISION_SHARED_IDS, 3419);
    assert_eq!(CROSS_REVISION_MODEL_CONFLICTS, 0);

    let a = [NpcRow {
        id: 1201,
        model_id: 46,
        model_file: Some(ASSET.dwarf_therin),
        terrain_object_id: None,
        capture_guid: None,
    }];
    let b = [NpcRow {
        id: 1201,
        model_id: 47,
        model_file: Some("dwarf_m_other.adr"),
        terrain_object_id: None,
        capture_guid: None,
    }];
    assert_eq!(
        model_conflicts(&a, &b),
        vec![(1201, ASSET.dwarf_therin, "dwarf_m_other.adr")]
    );
    assert!(model_conflicts(&a, &a).is_empty());
    assert!(model_conflicts(&a, &b[..0]).is_empty());
}
