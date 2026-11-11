//! Mounts piece: every ride can be held in a launcher install, and its sources name real items.
//!
//! The release item table is the original (PS3-era) table, which lacks the rides added later.
//! `data/release/item-table-supplement.json` carries those rows, so the Marketplace, the collection
//! prize and the three ride quests hand over an item the release content defines. Each ride also
//! names the mount type its model animates with (the model's actor definition binds one
//! `<prefix>_loc_*` animation set; `AnimationTypes.xml` numbers the prefixes).
use osfr_tools::{inventory::Inventory, mounts::MountCatalog};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn data(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(relative)
}

fn json(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(data(relative)).unwrap()).unwrap()
}

fn catalog() -> MountCatalog {
    MountCatalog::load(data("content/mounts.json")).unwrap()
}

#[test]
fn every_ride_item_the_shop_names_is_defined_by_release_content_alone() {
    let inventory = Inventory::load(&data("release"), None).unwrap();
    let catalog = catalog();
    let stores = json("content/stores.json");
    let mut sourced = BTreeMap::new();
    for row in stores["items"].as_array().unwrap() {
        let item = row["item_id"].as_i64().unwrap() as i32;
        let Some(mount) = catalog.mount_for_item(item) else {
            continue;
        };
        assert!(
            inventory.has_definition(item),
            "{} (item {item}) has no release item row",
            mount.name
        );
        sourced.insert(mount.id, item);
    }
    for mount in catalog.mounts() {
        assert!(
            sourced.contains_key(&mount.id),
            "{} has no shop or earn-only row",
            mount.name
        );
    }
}

#[test]
fn a_supplement_ride_row_carries_its_mount_id_like_the_original_table() {
    let catalog = catalog();
    let supplement = json("release/item-table-supplement.json");
    let columns: Vec<&str> = supplement["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    let column = |name: &str| columns.iter().position(|c| *c == name).unwrap();
    let (kind, param) = (column("Type"), column("Unknown36"));
    let mut rides = 0;
    for row in supplement["rows"].as_array().unwrap() {
        let item = row[0].as_i64().unwrap() as i32;
        let Some(mount) = catalog.mount_for_item(item) else {
            continue;
        };
        assert_eq!(row[1][kind], 19, "{item} is a ride item (Type 19)");
        assert_eq!(row[1][param], mount.id, "{item} names its mount");
        rides += 1;
    }
    assert!(rides > 0, "the supplement holds ride rows");
}

#[test]
fn every_ride_names_the_mount_type_its_model_animates_with() {
    let catalog = catalog();
    for mount in catalog.mounts() {
        let type_id = mount
            .mount_type_id
            .unwrap_or_else(|| panic!("{} has no mount type", mount.name));
        assert!(
            catalog.mount_types().any(|t| t.id == type_id),
            "{} names an unknown mount type {type_id}",
            mount.name
        );
        assert!(mount.model_id.is_some(), "{} has no model", mount.name);
    }
}

/// The wiki's three ride quests pay their ride (fanbyte quest pages; the item is the ride's own).
#[test]
fn the_three_ride_quests_pay_their_rides() {
    let inventory = Inventory::load(&data("release"), None).unwrap();
    let catalog = catalog();
    let quests = json("content/quests/card_duelist.json");
    for (quest, ride) in [
        (3_040_052, "Gloamstone Hoverboard"),
        (3_040_057, "Sparky"),
        (3_040_063, "Skullz Chopper"),
    ] {
        let record = quests["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["id"] == quest)
            .unwrap();
        assert_eq!(record["enabled"], true, "quest {quest} is live");
        let items: Vec<i32> = record["rewards"][0]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["definition_id"].as_i64().unwrap() as i32)
            .collect();
        let paid: Vec<&str> = items
            .iter()
            .filter_map(|item| catalog.mount_for_item(*item))
            .map(|mount| mount.name.as_str())
            .collect();
        assert_eq!(paid, [ride], "quest {quest}");
        assert!(items.iter().all(|item| inventory.has_definition(*item)));
    }
}
