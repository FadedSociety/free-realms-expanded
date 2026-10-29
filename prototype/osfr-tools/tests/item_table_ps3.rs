//! T875: the pass that checked our item catalogue against the original item table.
//!
//! `scripts/build_item_catalogue.py` writes `data/reference/item_table_ps3.json` (ids and numbers
//! only). These checks pin its counts and three sample corrections. The extra columns are carried
//! raw; no server value reads them yet (the unit of ACTIVATABLE_RECAST_SECONDS is unproven).
use serde_json::Value;
use std::path::Path;

#[path = "support/json.rs"]
mod json;

fn data(relative: &str) -> Value {
    json::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
}

fn correction<'a>(reference: &'a Value, id: i64, field: &str) -> &'a Value {
    reference["corrections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id && row["field"] == field)
        .unwrap_or_else(|| panic!("no correction for item {id} {field}"))
}

#[test]
fn the_pass_pins_its_row_count_correction_count_and_sample_corrections() {
    let reference = data("data/reference/item_table_ps3.json");
    assert_eq!(reference["schema_version"], 1);
    let counts = &reference["counts"];
    // The original table has 57,925 data rows; 57,878 ids are in both tables, 47 only in the
    // original (added) and 957 only in ours (kept as they were).
    assert_eq!(counts["original_rows"], 57_925);
    assert_eq!(counts["common"], 57_878);
    assert_eq!(counts["original_only"], 47);
    assert_eq!(counts["reconstruction_only"], 957);
    assert_eq!(counts["catalogue_rows"], 58_882);
    assert_eq!(counts["corrections"], 145);
    assert_eq!(counts["corrected_rows"], 127);
    assert_eq!(counts["held_rows"], 5);

    // Each count is what the lists below it hold.
    let list = |key: &str| reference[key].as_array().unwrap().len();
    assert_eq!(list("corrections"), 145);
    assert_eq!(list("held"), 5);
    assert_eq!(list("original_only_ids"), 47);
    assert_eq!(list("reconstruction_only_ids"), 957);
    let rows: std::collections::BTreeSet<_> = reference["corrections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    assert_eq!(rows.len(), 127);
    for row in reference["corrections"].as_array().unwrap() {
        assert_ne!(
            row["was"], row["now"],
            "a correction changes its value: {row}"
        );
    }

    // Three sample corrections: a type, a price and a name.
    assert_eq!(correction(&reference, 37010, "Type")["was"], 6);
    assert_eq!(correction(&reference, 37010, "Type")["now"], 1);
    assert_eq!(correction(&reference, 76751, "Cost")["was"], 270);
    assert_eq!(correction(&reference, 76751, "Cost")["now"], 9453);
    assert_eq!(correction(&reference, 14114, "NameId")["was"], 1670);
    assert_eq!(correction(&reference, 14114, "NameId")["now"], 1671);

    // The boomboxes (and one placeholder name) the original table blanks are held, not corrected.
    let held: Vec<_> = reference["held"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_i64().unwrap())
        .collect();
    assert_eq!(held, [878, 2215, 56774, 69825, 77329]);
    assert!(held.iter().all(|id| !rows.contains(id)));
}
