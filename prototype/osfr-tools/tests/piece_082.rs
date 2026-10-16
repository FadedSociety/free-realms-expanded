//! Work item 082 acceptance: inventory-search preflight for the native crash.
//!
//! ORIGINAL native evidence this model follows (see
//! `notes/Research/Pieces/082 Inventory search crash.md`):
//! `lab/decompiled/client/FreeRealms/pseudocode.c:1170126-1170140` creates a
//! separate `BaseClient.SearchInventoryItemsData` data source and marks it with
//! a search flag at object offset `0x2e8`; `:1341483-1506` (`FUN_00a6e910`)
//! either rebuilds the plain row list or resolves the stored search string and
//! rebuilds the rows through the data source's per-row predicate virtual
//! (`:1341392`, `FUN_00a6e7b0`, slot `+0x3c`). The predicate's match rule is
//! NOT recovered, so this module must not reproduce a filtered result set.
//!
//! RECONSTRUCTED, and labelled as such at every call site: the row model below.
//! It projects each owned instance through the catalogue the client received and
//! reports which rows cannot resolve their definition, category or localized
//! name. It is a completeness preflight for a controlled native reproduction,
//! not a claim about the crash cause and not a fix.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module
//! before the lead registers `pub mod piece_082;` under `src/restoration/`.

#[path = "../src/restoration/piece_082.rs"]
#[allow(dead_code)]
mod piece_082;

use piece_082::{
    ContentIndex, InventorySearch, OwnedInstance, PreflightError, RowState, SEARCH_QUERY,
};

fn resolved_example() -> InventorySearch {
    let mut search = InventorySearch::new();
    search
        .add_instance(OwnedInstance {
            instance_id: 2,
            definition_id: 76476,
            rental: false,
        })
        .expect("instance");
    search
        .add_instance(OwnedInstance {
            instance_id: 1077,
            definition_id: 1195,
            rental: false,
        })
        .expect("instance");
    let content = search.content_mut();
    content
        .insert_definition(76476, 4, 8530)
        .expect("definition");
    content
        .insert_definition(1195, 3, 6278)
        .expect("definition");
    content.insert_category(3).expect("category");
    content.insert_category(4).expect("category");
    content.insert_localized_name(6278);
    content.insert_localized_name(8530);
    search
}

#[test]
fn fully_resolved_rows_report_no_gaps() {
    let report = resolved_example().report();
    assert_eq!(report.rows.len(), 2);
    assert_eq!(report.resolved_rows(), 2);
    assert_eq!(report.unresolved_rows(), 0);
    assert!(report.missing_definitions.is_empty());
    assert!(report.definitions_missing_category.is_empty());
    assert!(report.definitions_missing_name.is_empty());
    assert!(matches!(
        report.rows[0].state,
        RowState::Resolved { category_id: 4, .. }
    ));
}

#[test]
fn instance_without_a_definition_is_reported_with_both_ids() {
    let mut search = resolved_example();
    search
        .add_instance(OwnedInstance {
            instance_id: 4594,
            definition_id: 78398,
            rental: false,
        })
        .expect("instance");
    let report = search.report();
    assert_eq!(report.unresolved_rows(), 1);
    assert_eq!(report.missing_definitions[0].instance_id, 4594);
    assert_eq!(report.missing_definitions[0].definition_id, 78398);
    assert!(matches!(
        report.rows.last().unwrap().state,
        RowState::MissingDefinition
    ));
}

#[test]
fn dangling_definition_category_and_name_are_reported() {
    let mut search = InventorySearch::new();
    search
        .add_instance(OwnedInstance {
            instance_id: 7,
            definition_id: 900,
            rental: true,
        })
        .expect("instance");
    let content = search.content_mut();
    content
        .insert_definition(900, 127, 4242)
        .expect("definition");
    content.insert_category(1).expect("category");
    let report = search.report();
    assert_eq!(report.resolved_rows(), 1);
    assert_eq!(report.definitions_missing_category.get(&127), Some(&1));
    assert_eq!(report.definitions_missing_name, vec![4242]);
    assert!(report.missing_definitions.is_empty());
}

#[test]
fn duplicate_instance_and_conflicting_definition_are_rejected() {
    let mut search = resolved_example();
    let error = search
        .add_instance(OwnedInstance {
            instance_id: 2,
            definition_id: 1195,
            rental: false,
        })
        .unwrap_err();
    assert!(matches!(error, PreflightError::DuplicateInstanceId(2)));

    let content = search.content_mut();
    assert!(
        content.insert_definition(76476, 4, 8530).is_ok(),
        "identical repeats are allowed"
    );
    let error = content.insert_definition(76476, 5, 8530).unwrap_err();
    assert!(matches!(
        error,
        PreflightError::ConflictingDefinition(76476)
    ));
}

#[test]
fn zero_and_negative_identifiers_are_rejected() {
    let mut content = ContentIndex::new();
    assert!(matches!(
        content.insert_definition(0, 1, 1).unwrap_err(),
        PreflightError::InvalidDefinitionId(0)
    ));
    assert!(matches!(
        content.insert_definition(1, -3, 1).unwrap_err(),
        PreflightError::InvalidCategoryId(-3)
    ));
    assert!(matches!(
        content.insert_category(-1).unwrap_err(),
        PreflightError::InvalidCategoryId(-1)
    ));
    let mut search = InventorySearch::new();
    assert!(matches!(
        search
            .add_instance(OwnedInstance {
                instance_id: 0,
                definition_id: 5,
                rental: false
            })
            .unwrap_err(),
        PreflightError::InvalidInstanceId(0)
    ));
    assert!(matches!(
        search
            .add_instance(OwnedInstance {
                instance_id: 1,
                definition_id: 0,
                rental: false
            })
            .unwrap_err(),
        PreflightError::InvalidDefinitionId(0)
    ));
}

#[test]
fn fingerprint_is_deterministic_and_sensitive_to_the_input_set() {
    let first = resolved_example().report();
    let second = resolved_example().report();
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(first.fingerprint.len(), 64);

    let mut changed = resolved_example();
    changed.content_mut().insert_category(99).expect("category");
    assert_ne!(changed.report().fingerprint, first.fingerprint);

    let mut missing = resolved_example();
    missing
        .add_instance(OwnedInstance {
            instance_id: 4594,
            definition_id: 78398,
            rental: false,
        })
        .expect("instance");
    assert_ne!(missing.report().fingerprint, first.fingerprint);
}

#[test]
fn recovered_native_constants_stay_pinned() {
    assert_eq!(
        piece_082::SEARCH_DATA_SOURCE_NAME,
        "BaseClient.SearchInventoryItemsData"
    );
    assert_eq!(
        piece_082::PLAIN_ROWS_DATA_SOURCE_NAME,
        "BaseClient.InventoryItemsData"
    );
    assert_eq!(piece_082::SEARCH_FLAG_OFFSET, 0x2e8);
    assert_eq!(piece_082::SEARCH_TEXT_OFFSET, 0x2f0);
    assert_eq!(piece_082::APPLIED_SEARCH_TEXT_OFFSET, 0x2ec);
    assert_eq!(piece_082::SEARCH_ROW_PREDICATE_VTABLE_SLOT, 0x3c);
    assert_eq!(SEARCH_QUERY, "Wheeler");
}

/// Private oracle: run the preflight over the exact preserved crash-run inputs
/// and cross-check the counts and fingerprint produced by
/// `scripts/pieces/piece_082.py` (the two implementations must agree).
#[test]
#[ignore = "requires the private preflight-input.json produced by scripts/pieces/piece_082.py"]
fn private_crash_run_preflight_matches_the_python_checker() {
    let path = std::path::PathBuf::from(
        std::env::var_os("OSFR_PIECE_082_PREFLIGHT").expect("private preflight input required"),
    );
    let raw = std::fs::read_to_string(&path).expect("read preflight input");
    let input: serde_json::Value = serde_json::from_str(&raw).expect("parse preflight input");

    let mut search = InventorySearch::new();
    for instance in input["instances"].as_array().expect("instances") {
        search
            .add_instance(OwnedInstance {
                instance_id: instance["instance_id"].as_u64().expect("instance id") as u32,
                definition_id: instance["definition_id"].as_u64().expect("definition id") as u32,
                rental: instance["rental"].as_bool().expect("rental flag"),
            })
            .expect("instance");
    }
    let content = search.content_mut();
    for definition in input["definitions"].as_array().expect("definitions") {
        content
            .insert_definition(
                definition["definition_id"].as_u64().expect("definition id") as u32,
                definition["category_id"].as_i64().expect("category id") as i32,
                definition["name_id"].as_u64().expect("name id") as u32,
            )
            .expect("definition");
    }
    for category in input["categories"].as_array().expect("categories") {
        content
            .insert_category(category.as_i64().expect("category id") as i32)
            .expect("category");
    }
    for name in input["localized_name_ids"].as_array().expect("names") {
        content.insert_localized_name(name.as_u64().expect("name id") as u32);
    }

    let report = search.report();
    let expected = &input["expected"];
    assert_eq!(
        report.rows.len(),
        expected["rows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.definition_count,
        expected["definitions"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.category_count,
        expected["categories"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.unresolved_rows(),
        expected["unresolved_rows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        report.fingerprint,
        expected["fingerprint"].as_str().unwrap()
    );
    assert_eq!(report.missing_definitions.len(), 3);
    assert_eq!(
        report
            .definitions_missing_category
            .values()
            .copied()
            .sum::<u32>(),
        63
    );
    assert!(report.definitions_missing_name.is_empty());
}
