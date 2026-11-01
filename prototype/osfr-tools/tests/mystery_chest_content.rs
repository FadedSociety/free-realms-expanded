//! L093: the shipped Mystery Chests. The five chests (reconstructed Type 28 definitions in
//! `data/release/item-table-supplement.json`) and their bindings (`data/release/mystery_chests.json`) load with their wiki prizes through the real
//! loaders, from the release's content (`data/content` with `data/release` laid over it).
#[path = "common/content_copy.rs"]
mod content_copy;

use osfr_tools::{inventory::Inventory, mystery_chest_loot::MysteryChests};
use serde_json::Value;
use std::{fs, path::PathBuf};

fn merged() -> PathBuf {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");
    let root = std::env::temp_dir().join(format!("osfr-chests-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    content_copy::copy_tree(&data.join("content"), &root);
    content_copy::copy_tree(&data.join("release"), &root);
    root
}

#[test]
fn the_five_chests_load_with_their_prizes() {
    let dir = merged();
    let inventory = Inventory::load(&dir, None).unwrap();
    assert!(MysteryChests::load(&dir, &inventory).unwrap().is_some());
    let chests: Value =
        serde_json::from_slice(&fs::read(dir.join("mystery_chests.json")).unwrap()).unwrap();
    let chests = chests["chests"].as_array().unwrap();
    let icons = [7666, 7723, 7777, 8134, 8500];
    assert_eq!(
        chests.len(),
        5,
        "Original, Sci-Fi, Clockwork, Snow Days, Chest of Wonders"
    );
    for (chest, icon) in chests.iter().zip(icons) {
        let id = chest["definition_id"].as_i64().unwrap() as i32;
        let shown = inventory.presentation(id).unwrap();
        assert_eq!((shown.item_type, shown.icon_id), (28, icon), "chest {id}");
    }
    let rule = |chest: &Value, key: &str| {
        chest["rewards"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p.get(key).is_some())
            .count()
    };
    assert_eq!(
        rule(&chests[1], "members"),
        1,
        "the Flying Saucer is the members' prize"
    );
    assert_eq!(
        rule(&chests[3], "members"),
        1,
        "Santa's Sleigh-mobile is the members' prize"
    );
    assert_eq!(
        rule(&chests[4], "gender"),
        10,
        "the Desert Traveler and Elegant Desert sets"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// L094: the Mystery Key sells in the Marketplace at the wiki's 180 SC on the client's own
/// "Mystery Chests" category shelf (408, name 436411).
#[test]
fn the_mystery_key_has_its_marketplace_shelf() {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json");
    let stores: Value = serde_json::from_slice(&fs::read(data).unwrap()).unwrap();
    let key = stores["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["item_id"] == 78137)
        .expect("the key row");
    assert_eq!(
        (key["sc_price"].as_u64(), key["category_id"].as_u64()),
        (Some(180), Some(408))
    );
    let shelves = stores["station_cash"]["item_shelves"].as_array().unwrap();
    assert!(shelves.iter().any(|shelf| {
        shelf["category_id"] == 408
            && shelf["category_name_id"] == 436411
            && shelf["item_category_ids"]
                .as_array()
                .unwrap()
                .contains(&Value::from(408))
    }));
}
