//! B3 (event items): the rows `scripts/event_items.py` adds to `data/release/item-table-supplement.json`
//! for the event items the 2026-11-11 event audits found with no item row. A launcher install's release
//! content alone defines each one with its own name, description and icon, a pet row adopts through
//! `pets.json`, and a placeable row has its `housing_fixtures.json` row.
use osfr_tools::{inventory::Inventory, pets::PetContent};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

fn data(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(relative)
}

fn json(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(data(relative)).unwrap()).unwrap()
}

/// The B3 rows by id (`source.event_item_ids` names them).
fn event_rows() -> BTreeMap<i64, Value> {
    let supplement = json("release/item-table-supplement.json");
    let ids: BTreeSet<i64> = supplement["source"]["event_item_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap())
        .collect();
    let rows: BTreeMap<i64, Value> = supplement["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| ids.contains(&row[0].as_i64().unwrap()))
        .map(|row| (row[0].as_i64().unwrap(), row.clone()))
        .collect();
    assert_eq!(rows.len(), ids.len(), "every listed id has a supplement row");
    rows
}

fn int(row: &Value, column: usize) -> i32 {
    row[1][column].as_i64().unwrap() as i32
}

#[test]
fn every_event_item_is_defined_by_release_content_with_its_own_text_and_icon() {
    let rows = event_rows();
    assert_eq!(rows.len(), 393);
    let inventory = Inventory::load(&data("release"), None).unwrap();
    for (&id, row) in &rows {
        let id = i32::try_from(id).unwrap();
        assert!(inventory.has_definition(id), "{id} has no definition");
        let shown = inventory.presentation(id).unwrap();
        assert_eq!(
            (shown.name_id, shown.description_id, shown.icon_id, shown.item_type),
            (int(row, 0), int(row, 1), int(row, 2), int(row, 9)),
            "{id} shows the row's own name, description, icon and type"
        );
        assert!(shown.name_id > 0 && shown.icon_id > 0, "{id} names itself and has an icon");
    }
}

#[test]
fn an_event_pet_adopts_and_an_event_decoration_places() {
    let rows = event_rows();
    let pets = PetContent::bundled().unwrap();
    let fixtures: BTreeSet<i64> = json("content/housing_fixtures.json")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["item_definition_id"].as_i64().unwrap())
        .collect();
    let (mut adopted, mut placed) = (0, 0);
    for (&id, row) in &rows {
        if int(row, 9) == 2 {
            assert!(
                pets.definition_for_item(i32::try_from(id).unwrap()).is_some(),
                "pet {id} has no pets.json binding"
            );
            adopted += 1;
        }
        if int(row, 9) == 29 {
            assert!(fixtures.contains(&id), "{id} has no housing fixture row");
            placed += 1;
        }
    }
    assert_eq!(adopted, 18);
    assert!(placed > 0);
    // The Love Struck and Stone Hearted sets (seven pieces each) place like the rest of the furniture.
    let furniture = rows
        .values()
        .filter(|row| (3_544..=3_570).contains(&int(row, 0)))
        .filter(|row| fixtures.contains(&row[0].as_i64().unwrap()))
        .count();
    assert_eq!(furniture, 14);
}
