//! Work item 073 acceptance: discoveries and exploration triggers.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c`: `PacketPointOfInterestDefinitionReply`'s
//!   constructor writes opcode `0x39` (57) into the packet header, the client ships the
//!   `PointOfInterestDefinitionManager` / `ClientPointOfInterestManager` / `CurrentPOIDataSource`
//!   vtables, and the client asks for `BaseClient.PointOfInterestData` by that data-source name.
//! * `lab/decompiled/client-ui/ScriptsBase.lua`: the POI data source exposes the columns
//!   `Id, Name, ZoneID, X, y, z, Heading, IconId, NotificationType, SubName, BreadcrumbQuestId,
//!   AtlasName`; `BaseClient.CurrentPOI` exposes `POIName, POIId, AreaId` and its update handler
//!   calls `NotificationHandler:LoadingZone(poiId)` for `1 < poiId < 11`.
//! * `Resources/CollectionCategories.txt` row `1^93636^9535` and the shipped locale name it
//!   resolves to, `Exploration`.
//! * the captured gateway oracle `messages/0009-ready.bin` (inner opcode 57, 10,637 bytes) is the
//!   real wire form: per definition a `true` flag byte, 13 fixed fields, a length-prefixed ASCII
//!   atlas name, one trailing int, then a `false` terminator. The Rust private gate decodes that
//!   payload and compares it with the supplied emulator's `PointOfInterestDefinitions.json`.
//!
//! COMMUNITY (pinned references, not first-party): Sanctuary's `PointOfInterestDefinition.Serialize`
//! writes the same fields in the same order and labels notification types `Dungeon=3`, `Local=6`,
//! `ZoneHub=7`, `Ruins=10`; the Free Realms wiki documents that an exploration collection entry is
//! unlocked by approaching its location and lists the rare-exploration clothing rewards.
//!
//! RECONSTRUCTED here (this module's own policy, labelled): planar (X/Z) proximity, nearest-candidate
//! selection, the caller-supplied radius, the per-character once-only ledger, and the zone filter.
//! The activation radius itself is NOT recovered; it is a parameter.
//!
//! NOT CLAIMED: any reward grant, any numeric experience value, any historical activation radius,
//! and any native in-game discovery run. Those stay explicit gaps.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact source file keeps
// this target compiling against the real artifact. Integration step for the lead: register
// `pub mod piece_073;` under `src/restoration/` and drop this `#[path]` include.
#[path = "../src/restoration/piece_073.rs"]
#[allow(dead_code)]
mod piece_073;

use piece_073::{
    COLLECTION_CATEGORY_EXPLORATION, CURRENT_POI_COLUMNS, CURRENT_POI_DATASOURCE_NAME,
    DiscoveryError, DiscoveryOutcome, DiscoveryTracker, PACKET_POI_DEFINITION_REPLY,
    PACKET_POI_DEFINITION_REQUEST, POI_DATASOURCE_COLUMNS, POI_DATASOURCE_NAME, POI_ENTRY_PRESENT,
    POI_LIST_TERMINATOR, POI_NOTIFICATION_DUNGEON, POI_NOTIFICATION_LOCAL, POI_NOTIFICATION_RUINS,
    POI_NOTIFICATION_ZONE_HUB, PoiDecodeError, PoiDefinition, PoiTable, PoiTableError, Position,
    ZONE_NOTIFICATION_MAX_POI_ID, ZONE_NOTIFICATION_MIN_POI_ID, decode_poi_definitions,
    poi_for_exploration_entry,
};

fn definition(id: i32, name_id: i32, location_id: i32, x: f32, y: f32, z: f32) -> PoiDefinition {
    PoiDefinition {
        id,
        name_id,
        location_id,
        x,
        y,
        z,
        heading: 1.0,
        icon_id: 1345,
        notification_type: POI_NOTIFICATION_ZONE_HUB,
        sub_name_id: 382845,
        unknown_11: 651,
        breadcrumb_quest_id: 0,
        teleport_location_id: 210020,
        atlas_name: "sacredgrove".to_owned(),
        unknown_15: 0,
    }
}

/// Two synthetic definitions in the captured wire form. Values are a test fixture, not recovered
/// content: a `sacredgrove` hub and an empty-atlas local point.
fn synthetic_wire() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(POI_ENTRY_PRESENT);
    bytes.extend_from_slice(&3i32.to_le_bytes());
    bytes.extend_from_slice(&3961i32.to_le_bytes());
    bytes.extend_from_slice(&281i32.to_le_bytes());
    bytes.extend_from_slice(&51.391f32.to_le_bytes());
    bytes.extend_from_slice(&30.987f32.to_le_bytes());
    bytes.extend_from_slice(&363.156f32.to_le_bytes());
    bytes.extend_from_slice(&1.487f32.to_le_bytes());
    bytes.extend_from_slice(&1345i32.to_le_bytes());
    bytes.extend_from_slice(&7i32.to_le_bytes());
    bytes.extend_from_slice(&382845i32.to_le_bytes());
    bytes.extend_from_slice(&651i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&210020i32.to_le_bytes());
    bytes.extend_from_slice(&11u32.to_le_bytes());
    bytes.extend_from_slice(b"sacredgrove");
    bytes.extend_from_slice(&60i32.to_le_bytes());
    bytes.push(POI_ENTRY_PRESENT);
    bytes.extend_from_slice(&11i32.to_le_bytes());
    bytes.extend_from_slice(&2061i32.to_le_bytes());
    bytes.extend_from_slice(&644i32.to_le_bytes());
    bytes.extend_from_slice(&89.0f32.to_le_bytes());
    bytes.extend_from_slice(&42.0f32.to_le_bytes());
    bytes.extend_from_slice(&1020.0f32.to_le_bytes());
    bytes.extend_from_slice(&2.5f32.to_le_bytes());
    bytes.extend_from_slice(&7546i32.to_le_bytes());
    bytes.extend_from_slice(&6i32.to_le_bytes());
    bytes.extend_from_slice(&382845i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&646i32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.push(POI_LIST_TERMINATOR);
    bytes
}

#[test]
fn recovered_packet_and_data_source_constants() {
    assert_eq!(PACKET_POI_DEFINITION_REQUEST, 56);
    assert_eq!(PACKET_POI_DEFINITION_REPLY, 57);
    assert_eq!(POI_ENTRY_PRESENT, 1);
    assert_eq!(POI_LIST_TERMINATOR, 0);
    assert_eq!(POI_DATASOURCE_NAME, "BaseClient.PointOfInterestData");
    assert_eq!(CURRENT_POI_DATASOURCE_NAME, "BaseClient.CurrentPOI");
    assert_eq!(
        POI_DATASOURCE_COLUMNS,
        [
            "Id",
            "Name",
            "ZoneID",
            "X",
            "y",
            "z",
            "Heading",
            "IconId",
            "NotificationType",
            "SubName",
            "BreadcrumbQuestId",
            "AtlasName",
        ]
    );
    assert_eq!(CURRENT_POI_COLUMNS, ["POIName", "POIId", "AreaId"]);
    assert_eq!(COLLECTION_CATEGORY_EXPLORATION, 1);
    assert_eq!(ZONE_NOTIFICATION_MIN_POI_ID, 2);
    assert_eq!(ZONE_NOTIFICATION_MAX_POI_ID, 10);
    // Community labels for the notification-type column, not first-party names.
    assert_eq!(
        [
            POI_NOTIFICATION_DUNGEON,
            POI_NOTIFICATION_LOCAL,
            POI_NOTIFICATION_ZONE_HUB,
            POI_NOTIFICATION_RUINS,
        ],
        [3, 6, 7, 10]
    );
}

#[test]
fn decodes_the_captured_wire_shape() {
    let decoded = decode_poi_definitions(&synthetic_wire()).expect("synthetic payload decodes");
    assert_eq!(decoded.len(), 2);
    let first = PoiDefinition {
        heading: 1.487,
        unknown_15: 60,
        ..definition(3, 3961, 281, 51.391, 30.987, 363.156)
    };
    assert_eq!(decoded[0], first);
    assert_eq!(decoded[0].teleport_location_id, 210020);
    assert_eq!(decoded[1].id, 11);
    assert_eq!(decoded[1].notification_type, POI_NOTIFICATION_LOCAL);
    assert_eq!(decoded[1].atlas_name, "");
    assert_eq!(decoded[1].unknown_15, 0);
}

#[test]
fn decode_rejects_a_missing_terminator_and_trailing_bytes() {
    let mut unterminated = synthetic_wire();
    unterminated.pop();
    assert_eq!(
        decode_poi_definitions(&unterminated),
        Err(PoiDecodeError::UnexpectedEof)
    );

    let mut trailing = synthetic_wire();
    trailing.push(0x7f);
    assert_eq!(
        decode_poi_definitions(&trailing),
        Err(PoiDecodeError::TrailingBytes)
    );
    assert!(decode_poi_definitions(&[]).is_err());
}

#[test]
fn decode_rejects_an_unknown_entry_flag_and_a_non_ascii_name() {
    let mut bad_flag = synthetic_wire();
    bad_flag[0] = 2;
    assert_eq!(
        decode_poi_definitions(&bad_flag),
        Err(PoiDecodeError::UnknownFlag(2))
    );

    let mut non_ascii = synthetic_wire();
    let offset = non_ascii
        .windows(11)
        .position(|window| window == b"sacredgrove")
        .expect("fixture contains the atlas name");
    non_ascii[offset] = 0xc3;
    assert_eq!(
        decode_poi_definitions(&non_ascii),
        Err(PoiDecodeError::NonAsciiString)
    );
}

#[test]
fn table_indexes_ids_and_names_and_refuses_duplicates() {
    let table = PoiTable::new(vec![
        definition(3, 3961, 281, 51.391, 30.987, 363.156),
        definition(11, 2061, 644, 89.0, 42.0, 1020.0),
    ])
    .expect("unique fixture");
    assert_eq!(table.len(), 2);
    assert_eq!(table.by_id(11).map(|poi| poi.name_id), Some(2061));
    assert_eq!(
        table
            .by_name_id(3961)
            .iter()
            .map(|poi| poi.id)
            .collect::<Vec<_>>(),
        vec![3]
    );
    assert!(table.by_name_id(99).is_empty());
    assert!(table.by_id(99).is_none());

    let duplicate_id = PoiTable::new(vec![
        definition(3, 3961, 281, 0.0, 0.0, 0.0),
        definition(3, 2061, 644, 1.0, 0.0, 0.0),
    ]);
    assert_eq!(duplicate_id, Err(PoiTableError::DuplicateId(3)));

    // The real captured table repeats a display name, so a shared name id is legal and indexed
    // as a multi-map; the exploration link refuses the ambiguous case.
    let shared_name = PoiTable::new(vec![
        definition(3, 3961, 281, 0.0, 0.0, 0.0),
        definition(4, 3961, 282, 1.0, 0.0, 0.0),
    ])
    .expect("name ids may repeat");
    assert_eq!(
        shared_name
            .by_name_id(3961)
            .iter()
            .map(|poi| poi.id)
            .collect::<Vec<_>>(),
        vec![3, 4]
    );
    assert!(poi_for_exploration_entry(&shared_name, 3961).is_none());
}

#[test]
fn nearest_candidate_inside_the_caller_radius_is_discovered_once() {
    let table = PoiTable::new(vec![
        definition(3, 3961, 281, 0.0, 0.0, 0.0),
        definition(11, 2061, 644, 30.0, 0.0, 0.0),
    ])
    .expect("unique fixture");
    let mut tracker = DiscoveryTracker::new(7);

    let outcome = tracker
        .evaluate(&table, Position::new(2.0, 5.0, 2.0), None, 10.0)
        .expect("valid request");
    match outcome {
        DiscoveryOutcome::Discovered(poi) => assert_eq!(poi.id, 3),
        other => panic!("expected a discovery, got {other:?}"),
    }
    assert!(tracker.is_discovered(3));
    assert_eq!(tracker.discovered_count(), 1);

    // A replayed visit is ignored: discovery is a once-only event per character.
    let replay = tracker
        .evaluate(&table, Position::new(2.0, 5.0, 2.0), None, 10.0)
        .expect("valid request");
    assert_eq!(replay, DiscoveryOutcome::AlreadyDiscovered(3));
    assert_eq!(tracker.discovered_count(), 1);

    // The nearest in-range definition wins over a farther one.
    let nearer = tracker
        .evaluate(&table, Position::new(29.0, 0.0, 0.0), None, 10.0)
        .expect("valid request");
    match nearer {
        DiscoveryOutcome::Discovered(poi) => assert_eq!(poi.id, 11),
        other => panic!("expected the nearer definition, got {other:?}"),
    }
}

#[test]
fn out_of_range_reports_the_nearest_candidate_without_granting() {
    let table = PoiTable::new(vec![definition(3, 3961, 281, 0.0, 0.0, 0.0)]).expect("fixture");
    let mut tracker = DiscoveryTracker::new(7);
    let outcome = tracker
        .evaluate(&table, Position::new(100.0, 0.0, 0.0), None, 10.0)
        .expect("valid request");
    match outcome {
        DiscoveryOutcome::NoCandidate(Some(nearest)) => {
            assert_eq!(nearest.poi_id, 3);
            assert!((nearest.distance - 100.0).abs() < 1e-3);
        }
        other => panic!("expected no candidate, got {other:?}"),
    }
    assert_eq!(tracker.discovered_count(), 0);
}

#[test]
fn distance_is_planar_so_height_does_not_decide_the_trigger() {
    let table = PoiTable::new(vec![definition(3, 3961, 281, 0.0, 0.0, 0.0)]).expect("fixture");
    let mut tracker = DiscoveryTracker::new(7);
    // Directly above the definition: planar distance is zero, so the reconstructed policy is in
    // range. This pins the planar choice instead of leaving it implicit.
    let above = tracker
        .evaluate(&table, Position::new(0.0, 40.0, 0.0), None, 10.0)
        .expect("valid request");
    assert!(matches!(above, DiscoveryOutcome::Discovered(_)));

    // A 3-4-5 ground offset measures 5: inside a 5.0 radius, outside a 4.9 one.
    let mut inside = DiscoveryTracker::new(7);
    let hit = inside
        .evaluate(&table, Position::new(3.0, 0.0, 4.0), None, 5.0)
        .expect("valid request");
    assert!(matches!(hit, DiscoveryOutcome::Discovered(_)));
    let mut outside = DiscoveryTracker::new(7);
    match outside
        .evaluate(&table, Position::new(3.0, 0.0, 4.0), None, 4.9)
        .expect("valid request")
    {
        DiscoveryOutcome::NoCandidate(Some(nearest)) => {
            assert!((nearest.distance - 5.0).abs() < 1e-4);
        }
        other => panic!("expected the planar distance to miss a 4.9 radius, got {other:?}"),
    }
}

#[test]
fn zone_filter_keeps_unrelated_locations_out_of_the_candidate_set() {
    let table = PoiTable::new(vec![
        definition(3, 3961, 281, 0.0, 0.0, 0.0),
        definition(11, 2061, 644, 1.0, 0.0, 0.0),
    ])
    .expect("fixture");
    let mut tracker = DiscoveryTracker::new(7);
    let outcome = tracker
        .evaluate(&table, Position::new(0.5, 0.0, 0.0), Some(644), 10.0)
        .expect("valid request");
    match outcome {
        DiscoveryOutcome::Discovered(poi) => assert_eq!(poi.id, 11),
        other => panic!("expected the location 644 definition, got {other:?}"),
    }
}

#[test]
fn invalid_radius_and_non_finite_positions_are_refused() {
    let table = PoiTable::new(vec![definition(3, 3961, 281, 0.0, 0.0, 0.0)]).expect("fixture");
    let mut tracker = DiscoveryTracker::new(7);
    assert_eq!(
        tracker.evaluate(&table, Position::new(0.0, 0.0, 0.0), None, -1.0),
        Err(DiscoveryError::InvalidRadius)
    );
    assert_eq!(
        tracker.evaluate(&table, Position::new(0.0, 0.0, 0.0), None, f32::NAN),
        Err(DiscoveryError::InvalidRadius)
    );
    assert_eq!(
        tracker.evaluate(&table, Position::new(f32::NAN, 0.0, 0.0), None, 5.0),
        Err(DiscoveryError::InvalidPosition)
    );
    assert_eq!(tracker.discovered_count(), 0);
}

#[test]
fn exploration_entries_link_to_pois_by_the_recovered_name_id() {
    let table = PoiTable::new(vec![
        definition(78, 5133, 2451, -710.791, -49.829, 1754.531),
        definition(11, 2061, 644, 89.0, 42.0, 1020.0),
    ])
    .expect("fixture");
    // Captured exploration entry "Treeleaf's Retreat" carries name id 5133.
    let linked = poi_for_exploration_entry(&table, 5133).expect("entry links to its POI");
    assert_eq!(linked.id, 78);
    assert_eq!(linked.location_id, 2451);
    assert!(poi_for_exploration_entry(&table, 1834).is_none());
}

/// Private gate: decode the real captured opcode-57 payload, compare it with the supplied
/// emulator's POI table, and export the vectors the Python checker re-evaluates.
///
/// ```text
/// $env:OSFR_PIECE_073_MESSAGE = '...\lab\captures\legacy-gateway-20261008T014620082859Z\messages\0009-ready.bin'
/// $env:OSFR_PIECE_073_JSON    = '...\lab\working\OSFR.Server\OSFR Server\Customize\PointOfInterestDefinitions.json'
/// $env:OSFR_PIECE_073_REPORT  = '...\lab\reports\piece-073-<UTC>'
/// .\scripts\cargo.ps1 test --test piece_073 -- --ignored --exact private_captured_poi_payload_matches_supplied_table
/// ```
#[test]
#[ignore = "requires the preserved capture and the supplied emulator's Customize data"]
fn private_captured_poi_payload_matches_supplied_table() {
    use serde_json::json;

    let message_path = std::env::var_os("OSFR_PIECE_073_MESSAGE")
        .expect("OSFR_PIECE_073_MESSAGE must point at messages/0009-ready.bin");
    let json_path = std::env::var_os("OSFR_PIECE_073_JSON")
        .expect("OSFR_PIECE_073_JSON must point at PointOfInterestDefinitions.json");
    let report_dir = std::env::var_os("OSFR_PIECE_073_REPORT")
        .expect("OSFR_PIECE_073_REPORT must point at a directory");

    let frame = std::fs::read(&message_path).expect("readable captured message");
    // Outer tunnel header: 1 byte opcode, 4 byte length, 1 byte flag, 2 byte inner opcode,
    // 4 byte inner length. Body starts after that.
    assert_eq!(frame[0], 0x05, "outer tunneled opcode");
    assert_eq!(
        u16::from_le_bytes([frame[7], frame[8]]),
        PACKET_POI_DEFINITION_REPLY
    );
    let body_len = u32::from_le_bytes([frame[9], frame[10], frame[11], frame[12]]) as usize;
    let body = &frame[13..13 + body_len];
    assert_eq!(body.len(), frame.len() - 13);

    let decoded = decode_poi_definitions(body).expect("captured body decodes");

    let table_json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&json_path).expect("readable POI table"))
            .expect("POI table parses");
    let expected = table_json.as_array().expect("POI table is an array");
    assert_eq!(decoded.len(), expected.len(), "decoded definition count");
    for (decoded_row, expected_row) in decoded.iter().zip(expected) {
        assert_eq!(
            decoded_row.id,
            expected_row["Id"].as_i64().expect("Id") as i32
        );
        assert_eq!(
            decoded_row.name_id,
            expected_row["NameId"].as_i64().expect("NameId") as i32
        );
        assert_eq!(
            decoded_row.location_id,
            expected_row["LocationId"].as_i64().expect("LocationId") as i32
        );
        for (field, key) in [("x", "X"), ("y", "Y"), ("z", "Z"), ("heading", "Heading")] {
            let decoded_value = match field {
                "x" => decoded_row.x,
                "y" => decoded_row.y,
                "z" => decoded_row.z,
                _ => decoded_row.heading,
            };
            let expected_value = expected_row[key].as_f64().expect(key) as f32;
            assert!(
                (decoded_value - expected_value).abs() < 1e-4,
                "{key} row {}: {decoded_value} != {expected_value}",
                decoded_row.id
            );
        }
        assert_eq!(
            decoded_row.icon_id,
            expected_row["IconId"].as_i64().expect("IconId") as i32
        );
        assert_eq!(
            decoded_row.notification_type,
            expected_row["NotificationType"]
                .as_i64()
                .expect("NotificationType") as i32
        );
        assert_eq!(
            decoded_row.sub_name_id,
            expected_row["SubNameId"].as_i64().expect("SubNameId") as i32
        );
        assert_eq!(
            decoded_row.unknown_11,
            expected_row["Unknown11"].as_i64().expect("Unknown11") as i32
        );
        assert_eq!(
            decoded_row.breadcrumb_quest_id,
            expected_row["BreadcrumbQuestId"]
                .as_i64()
                .expect("BreadcrumbQuestId") as i32
        );
        assert_eq!(
            decoded_row.teleport_location_id,
            expected_row["TeleportLocationId"]
                .as_i64()
                .expect("TeleportLocationId") as i32
        );
        assert_eq!(
            decoded_row.atlas_name,
            expected_row["AtlasName"].as_str().expect("AtlasName")
        );
        assert_eq!(
            decoded_row.unknown_15,
            expected_row["Unknown15"].as_i64().expect("Unknown15") as i32
        );
    }

    // Export the evaluation vectors the Python checker re-runs against the same decoded table.
    // The captured table repeats one display name (382885); ids stay unique, so the table is keyed
    // by id and shared names are reported rather than hidden.
    let table = PoiTable::new(decoded).expect("captured table has unique ids");
    let vectors = vec![
        // POI 3 (`Snowhill`, location 281) and POI 11 (`Cooking Table`, location 644) are captured
        // definitions; the third case uses POI 78 (`Treeleaf's Retreat`, location 2451).
        (
            11u64,
            281i32,
            Position::new(51.391, 30.987, 363.156),
            5.0f32,
        ),
        // The same character and place again: the ledger must answer `AlreadyDiscovered`.
        (11, 281, Position::new(51.391, 30.987, 363.156), 5.0),
        (12, 281, Position::new(0.0, 0.0, 0.0), 25.0),
        (12, 644, Position::new(89.0, 42.0, 1020.0), 0.5),
        (12, 2451, Position::new(-710.79, -49.0, 1754.0), 3.0),
    ];
    let mut exported = Vec::new();
    for (index, (character, location, position, radius)) in vectors.iter().copied().enumerate() {
        let mut tracker = DiscoveryTracker::new(character);
        // Replay this character's earlier requests so the once-only rule is exercised.
        for (earlier_character, earlier_location, earlier_position, earlier_radius) in
            &vectors[..index]
        {
            if *earlier_character == character {
                tracker
                    .evaluate(
                        &table,
                        *earlier_position,
                        Some(*earlier_location),
                        *earlier_radius,
                    )
                    .expect("valid earlier vector");
            }
        }
        let outcome = tracker
            .evaluate(&table, position, Some(location), radius)
            .expect("valid vector");
        exported.push(json!({
            "character": character,
            "location_id": location,
            "position": {"x": position.x, "y": position.y, "z": position.z},
            "radius": radius,
            "outcome": match &outcome {
                DiscoveryOutcome::Discovered(poi) => json!({"kind": "discovered", "poi_id": poi.id}),
                DiscoveryOutcome::AlreadyDiscovered(poi_id) => json!({"kind": "already_discovered", "poi_id": poi_id}),
                DiscoveryOutcome::NoCandidate(nearest) => json!({
                    "kind": "none",
                    "nearest": nearest.map(|n| json!({"poi_id": n.poi_id, "distance": n.distance})),
                }),
            },
        }));
    }
    let exported = json!({
        "piece": "073",
        "source": "captured opcode 57 payload",
        "definitions": table.len(),
        "shared_name_ids": shared_name_ids(&table),
        "vectors": exported,
    });
    let report_path = std::path::Path::new(&report_dir).join("rust-vectors.json");
    std::fs::write(
        &report_path,
        serde_json::to_vec_pretty(&exported).expect("vectors serialise"),
    )
    .expect("report directory is writable");

    println!(
        "piece 073 private gate: {} definitions decoded, {} bytes, vectors -> {}",
        table.len(),
        body.len(),
        report_path.display()
    );
}

fn shared_name_ids(table: &piece_073::PoiTable) -> Vec<i32> {
    let mut shared: Vec<i32> = table
        .definitions()
        .iter()
        .map(|definition| definition.name_id)
        .filter(|name_id| table.by_name_id(*name_id).len() > 1)
        .collect();
    shared.sort_unstable();
    shared.dedup();
    shared
}
