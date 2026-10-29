//! T673: zone quests round 2, Sanctuary. None of the 13 rows in
//! `lab/references/quest-lines/wiring/zone-quests-matched/sanctuary.json` is wired by this task,
//! and this suite keeps each one off behind a blocker that is looked at again the day the blocker
//! lifts: five Card Duelist rows by the lead's decision (`tcg_and_brawler_npc_placements.rs`
//! guards it), Chef: Reach Level 5 already enabled by T723, and the rest with the blocker named
//! below. The one row that looked wireable, Bordon Cloo's `Chef: Learning to Crave the Flave`
//! (3050021, quest-lines/quests.json q3050021: cook six dishes, then talk to Cloo), stays off
//! because nothing in the shipped content teaches an ordinary Chef those six recipes, so an
//! enabled quest would be accepted and could never be finished (26/24 on an unlearned recipe is
//! refused).

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{content_dir, read_json};
use quest_records::{as_u32, shipped_quest};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path};

/// The six dishes Learning to Crave the Flave asks for (recipes 43, 46, 50, 51, 56, 59).
const FLAVE_DISHES: [u32; 6] = [43, 46, 50, 51, 56, 59];

/// Single-item fields in the shipped content that are not the recipe book or the startup item
/// catalogue: store rows, quest rewards and objectives, node drop tables.
const ITEM_KEYS: [&str; 4] = [
    "item_id",
    "item_definition_id",
    "definition_id",
    "resource_id",
];

/// Whether `key` holds a list of items: vendor `item_ids`, pet `adoption_item_ids` and `items`,
/// encounter `reward_items`, staff `default_items`, and any sibling that ends the same way.
fn is_item_list_key(key: &str) -> bool {
    key == "items" || key.ends_with("_items") || key.ends_with("item_ids")
}

fn json_files(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            json_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            found.push(path);
        }
    }
}

/// Whether an item-list field holds one of `ids`: an array of bare ids, or of `[id, count]` pairs.
/// Arrays of objects are left to the walk, which finds their `item_id`-style keys.
fn list_holds_any(field: &Value, ids: &BTreeSet<u64>) -> bool {
    let Value::Array(rows) = field else {
        return false;
    };
    rows.iter().any(|row| {
        let id = match row {
            Value::Array(pair) => pair.first().and_then(Value::as_u64),
            other => other.as_u64(),
        };
        id.is_some_and(|id| ids.contains(&id))
    })
}

/// Whether `value` holds one of `ids` in an item-reference field, at any depth.
fn lists_any(value: &Value, ids: &BTreeSet<u64>) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, field)| {
            let single = ITEM_KEYS.contains(&key.as_str());
            (single && field.as_u64().is_some_and(|id| ids.contains(&id)))
                || (is_item_list_key(key) && list_holds_any(field, ids))
                || lists_any(field, ids)
        }),
        Value::Array(rows) => rows.iter().any(|row| lists_any(row, ids)),
        _ => false,
    }
}

/// The pattern item a recipe is learned from, read from the shipped recipe book.
fn pattern_items() -> BTreeSet<u64> {
    let book = read_json(&content_dir().join("recipes.json"));
    let rows = book["records"].as_array().unwrap();
    let pattern = |recipe_id: u32| {
        let row = rows
            .iter()
            .find(|row| as_u32(&row["recipe_id"]) == Some(recipe_id));
        row.unwrap_or_else(|| panic!("recipe {recipe_id} is in the shipped book"))
            ["recipe_item_definition_id"]
            .as_u64()
            .unwrap_or_else(|| panic!("recipe {recipe_id} names its pattern item"))
    };
    FLAVE_DISHES.into_iter().map(pattern).collect()
}

/// Learning to Crave the Flave stays off until a Chef can actually learn its six dishes. The
/// pattern items (the recipe book's `recipe_item_definition_id`) have no source in the shipped
/// data: no vendor, store, quest reward or node drop table lists them, and the client item
/// evidence does not say where a player gets them (citems.json: pattern acquisition not
/// evidenced). Learning needs them (`Gameplay::learn_recipe`), so a player who accepted the quest
/// could never cook the dishes. When this test fails, a source landed: re-check the row, then
/// enable it with a test that starts from an unseeded Chef and cooks all six through 26/24.
#[test]
fn learning_to_crave_the_flave_stays_off_until_its_patterns_have_a_source() {
    let record = shipped_quest(3_050_021);
    assert!(
        !record.enabled,
        "3050021 is enabled: a Chef must be able to learn recipes 43, 46, 50, 51, 56 and 59"
    );
    assert!(shipped_quest(3_050_020).enabled, "its prerequisite is on");

    let patterns = pattern_items();
    assert_eq!(
        patterns.len(),
        FLAVE_DISHES.len(),
        "one pattern item per dish"
    );
    let mut files = Vec::new();
    json_files(&content_dir(), &mut files);
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        // The recipe book names the patterns; the startup snapshot is the item catalogue.
        if name == "recipes.json" || name == "snapshot.json" {
            continue;
        }
        assert!(
            !lists_any(&read_json(&file), &patterns),
            "{name} now lists a pattern item {patterns:?}: re-check quest 3050021"
        );
    }
}

/// The walker sees every shape the shipped content uses for an item, so a source cannot land
/// unseen: vendor `item_ids`, encounter `reward_items` pairs, pet `items` and `adoption_item_ids`,
/// staff `default_items`, and the single-id keys. A count under an item-list key is not an id.
#[test]
fn the_pattern_scan_sees_every_item_shape() {
    let ids = BTreeSet::from([55_825_u64]);
    let seen = |json: &str| lists_any(&serde_json::from_str(json).unwrap(), &ids);
    for shape in [
        r#"{"item_id": 55825}"#,
        r#"{"vendors": [{"item_ids": [55, 55825]}]}"#,
        r#"{"encounters": [{"reward_items": [[7, 1], [55825, 2]]}]}"#,
        r#"{"pets": [{"items": [55825], "adoption_item_ids": [55825]}]}"#,
        r#"{"jobs": [{"default_items": [55825]}]}"#,
        r#"{"a": {"drops": [{"definition_id": 55825}]}}"#,
    ] {
        assert!(seen(shape), "not seen: {shape}");
    }
    for shape in [
        r#"{"vendors_with_item_ids": 55825}"#,
        r#"{"reward_items": [[7, 55825]]}"#,
        r#"{"price": 55825}"#,
    ] {
        assert!(!seen(shape), "wrongly seen: {shape}");
    }
}

/// Rows the matched list calls wireable that stay disabled, each with the blocker named.
/// A disabled prerequisite is checked from the shipped records, so the row is looked at
/// again the day its prerequisite is enabled.
#[test]
fn the_other_rows_stay_off_behind_their_blocker() {
    // (quest, prerequisite that is itself disabled, why the row is not wired)
    let gated: [(u32, u32, &str); 5] = [
        (
            3_000_031,
            3_000_030,
            "Find Guard Glibbin, Forbi and Nibbs have no NPC row; Verbi is an unplaced anchor",
        ),
        (
            3_000_046,
            3_000_045,
            "the box recovery has no mechanic and Darkthorne is an unplaced anchor",
        ),
        (
            3_050_033,
            3_050_032,
            "a text-only objective with no dish list: nothing to complete",
        ),
        (
            3_090_025,
            3_090_024,
            "Darvon's Descent is an unplaced anchor and the shock-paddle box has no item",
        ),
        (
            3_150_013,
            3_150_012,
            "its prerequisite Warrior: Knocking Some Sense In waits on the class condition",
        ),
    ];
    for (id, prerequisite, why) in gated {
        assert!(
            !shipped_quest(id).enabled,
            "{id} ({why}) is enabled: re-check the row"
        );
        assert!(
            !shipped_quest(prerequisite).enabled,
            "{id}: prerequisite {prerequisite} is now enabled, so the row can be reached; {why}"
        );
    }
    // Medic: Operation: Revival is reachable (prerequisite 3090001 is on) but has no mechanic:
    // reviving twelve wounded bixies is a counted ability use the server cannot count yet.
    let revival = shipped_quest(3_090_002);
    assert!(!revival.enabled, "revive the bixies: re-check the row");
    assert!(shipped_quest(3_090_001).enabled);
}
