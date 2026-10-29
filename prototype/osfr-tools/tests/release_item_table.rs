//! T877: a release knows every item the original item table names.
//!
//! `data/release/item-table.json` is generated from the original table alone
//! (`scripts/build_item_catalogue.py --release-table`). `Inventory::load` reads it in a second
//! optional slot after the lab's `item-catalogue.json`, so a lab package keeps its richer rows and
//! a release gets the full table. These checks pin what resolves from release content alone, that
//! the loader's encoder reproduces the generator's bytes for every row, which file wins when both
//! exist, and the refusals.
use osfr_tools::inventory::Inventory;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

#[path = "common/item_catalogue.rs"]
mod item_catalogue;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/scratch.rs"]
mod scratch;
use json::read;
use npc_visibility::content_dir;
use scratch::Scratch;

fn data(relative: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(relative)
}

/// A scratch content folder holding the release `inventory.json` with its captured definitions
/// emptied (so every definition must come from a catalogue file), plus the named release files.
fn scratch_with(tag: &str, keep_table: bool) -> Scratch {
    let scratch = Scratch::new(tag);
    let mut inventory = read(&data("release/inventory.json"));
    inventory["definitions"] = json!([]);
    json::write_pretty(&scratch.path().join("inventory.json"), &inventory);
    if keep_table {
        fs::copy(
            data("release/item-table.json"),
            scratch.path().join("item-table.json"),
        )
        .unwrap();
    }
    scratch
}

/// Every integer under any of `keys`, as one number or a list of numbers.
fn ids(value: &Value, keys: &[&str], out: &mut BTreeSet<i32>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                if keys.contains(&key.as_str()) {
                    let numbers = inner
                        .as_array()
                        .cloned()
                        .unwrap_or_else(|| vec![inner.clone()]);
                    out.extend(numbers.iter().filter_map(Value::as_i64).map(|n| n as i32));
                }
                ids(inner, keys, out);
            }
        }
        Value::Array(rows) => rows.iter().for_each(|row| ids(row, keys, out)),
        _ => {}
    }
}

/// (file, keys, ids, resolved from the captured `inventory.json` alone, resolved with the table).
const CONSUMERS: [(&str, &[&str], usize, usize, usize); 4] = [
    ("stores.json", &["item_id"], 17_623, 811, 16_910),
    ("vendors.json", &["item_id", "item_ids"], 1_166, 328, 1_166),
    (
        "recipes.json",
        &["definition_id", "recipe_item_definition_id"],
        104,
        37,
        104,
    ),
    (
        "collection_catalogue.json",
        &["item_definition_id"],
        1_553,
        231,
        1_552,
    ),
];

fn resolved(inventory: &Inventory, wanted: &BTreeSet<i32>) -> usize {
    wanted
        .iter()
        .filter(|id| inventory.has_definition(**id))
        .count()
}

#[test]
fn release_content_alone_resolves_the_shop_vendor_recipe_and_collection_ids() {
    let after = Inventory::load(&data("release"), None).unwrap();
    let reference = read(&data("reference/item_table_ps3.json"));
    let reconstruction_only: BTreeSet<i64> = reference["reconstruction_only_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_i64().unwrap())
        .collect();
    for (file, keys, total, _, with_table) in CONSUMERS {
        let mut wanted = BTreeSet::new();
        ids(&read(&content_dir().join(file)), keys, &mut wanted);
        assert_eq!(wanted.len(), total, "{file} ids");
        assert_eq!(
            resolved(&after, &wanted),
            with_table,
            "{file} with the table"
        );
        // What stays unresolved is only ids the original table lacks (Legacy-only; the lab keeps them).
        for id in wanted.iter().filter(|id| !after.has_definition(**id)) {
            assert!(
                reconstruction_only.contains(&i64::from(*id)),
                "{file}: item {id} is missing from the table and is not a known reconstruction-only id"
            );
        }
    }
}

#[test]
fn the_captured_rows_alone_resolve_only_the_counts_before_the_table() {
    // `inventory.json` of the overlay without `item-table.json`: the release as it was.
    let scratch = Scratch::new("t877-captured");
    fs::copy(
        data("release/inventory.json"),
        scratch.path().join("inventory.json"),
    )
    .unwrap();
    let before = Inventory::load(scratch.path(), None).unwrap();
    for (file, keys, _, captured, _) in CONSUMERS {
        let mut wanted = BTreeSet::new();
        ids(&read(&content_dir().join(file)), keys, &mut wanted);
        assert_eq!(
            resolved(&before, &wanted),
            captured,
            "{file} before the table"
        );
    }
}

#[test]
fn every_table_row_encodes_to_the_generators_bytes() {
    let table = read(&data("release/item-table.json"));
    let inventory = Inventory::load(scratch_with("t877-wire", true).path(), None).unwrap();
    let mut digest = Sha256::new();
    let rows = table["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 57_925);
    for row in rows {
        let id = row[0].as_i64().unwrap() as i32;
        let record = inventory
            .definition_record(id)
            .unwrap_or_else(|| panic!("item {id} is not loaded"));
        assert_eq!(record[..4], id.to_le_bytes());
        digest.update(record);
    }
    assert_eq!(
        format!("{:x}", digest.finalize()),
        table["wire_sha256"].as_str().unwrap()
    );
    // Sample rows: price from the wire row (reader 00cff140 case 27), and the model/texture/tint
    // the catalogue look reads.
    assert_eq!(inventory.definition_coin_price(2), Some(320));
    let look = inventory.look(2).unwrap();
    assert_eq!(look.texture, "miner-twotone-L3");
    assert_eq!(look.tint_alias, "dyetint");
    assert_eq!(look.default_tint, 0);
}

#[test]
fn the_table_file_is_publishable() {
    let path = data("release/item-table.json");
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.len() < 10 * 1024 * 1024, "{} bytes", text.len());
    let table: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        table["source"]["sha256"],
        "dd925781568a006a0d17d38b1ac98f752bdeaca0a067fbbb5693adba572fda49"
    );
    assert!(
        table["source"]["table"]
            .as_str()
            .unwrap()
            .contains("PS3 client")
    );
    // The rows are item data (a model name may say "sanctuary"); the file's own description names
    // only the original table.
    let mut description = table.clone();
    description["rows"] = Value::Null;
    let description = description.to_string().to_ascii_lowercase();
    for forbidden in ["legacy", "sanctuary", "agpl"] {
        assert!(
            !description.contains(forbidden),
            "item-table.json cites {forbidden}"
        );
    }
}

#[test]
fn the_lab_catalogue_wins_where_both_files_define_an_item() {
    let scratch = scratch_with("t877-wins", true);
    // Item 2 is in both files with different bytes; 999999 is only in the lab catalogue.
    item_catalogue::write(
        scratch.path(),
        &[
            item_catalogue::sized_row(2, 64),
            item_catalogue::sized_row(999_999, 64),
        ],
    );
    let inventory = Inventory::load(scratch.path(), None).unwrap();
    assert_eq!(inventory.definition_record(2).unwrap().len(), 64);
    assert_eq!(inventory.definition_record(999_999).unwrap().len(), 64);
    // A table-only item is still defined, and keeps its table bytes.
    assert!(inventory.definition_record(3).unwrap().len() > 64);
}

#[test]
fn a_malformed_table_is_refused() {
    let table = read(&data("release/item-table.json"));
    let mut broken = vec![];
    let mut wrong_schema = table.clone();
    wrong_schema["schema_version"] = json!(2);
    broken.push(("schema", wrong_schema));
    let mut wrong_columns = table.clone();
    wrong_columns["columns"].as_array_mut().unwrap().swap(0, 1);
    broken.push(("columns", wrong_columns));
    let mut duplicate = table.clone();
    let first = duplicate["rows"][0].clone();
    duplicate["rows"].as_array_mut().unwrap().push(first);
    broken.push(("duplicate id", duplicate));
    let mut bad_flags = table.clone();
    bad_flags["rows"][0][1][22] = json!(128);
    broken.push(("flag bits", bad_flags));
    let mut long_text = table.clone();
    long_text["rows"][0][2] = json!("x".repeat(4097));
    broken.push(("text length", long_text));
    let mut bad_id = table.clone();
    bad_id["rows"][0][0] = json!(0);
    broken.push(("id", bad_id));
    for (what, document) in broken {
        let scratch = scratch_with("t877-bad", false);
        fs::write(
            scratch.path().join("item-table.json"),
            serde_json::to_vec(&document).unwrap(),
        )
        .unwrap();
        assert!(
            Inventory::load(scratch.path(), None).is_err(),
            "a table with a bad {what} was accepted"
        );
    }
}
