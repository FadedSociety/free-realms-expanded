#![allow(dead_code)]

#[path = "../src/npc_interaction.rs"]
mod npc_interaction;

use npc_interaction::NpcInteraction;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("content file reads"))
        .unwrap_or_else(|error| panic!("{} parses: {error}", path.display()))
}

#[test]
fn vendor_stock_is_loadable_and_supported_by_content_evidence() {
    let content = content_dir();
    let startup_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/content/startup-snapshot/startup.json");
    let startup = read_json(&startup_path);
    let catalogue_ids: BTreeSet<i64> = startup["catalogues"]["item_definitions"]
        .as_array()
        .expect("startup item catalogue is an array")
        .iter()
        .map(|row| row["id"].as_i64().expect("item ID is an integer"))
        .collect();

    let stores = read_json(&content.join("stores.json"));
    let prices: BTreeMap<i64, &Value> = stores["items"]
        .as_array()
        .expect("stores items are an array")
        .iter()
        .map(|row| {
            (
                row["item_id"]
                    .as_i64()
                    .expect("store item ID is an integer"),
                row,
            )
        })
        .collect();

    // The production loader also validates NPC references, vendor roles, unique IDs, and prices.
    NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("vendor content loads through the production NPC loader");

    let mut npc_roles = BTreeMap::<u64, BTreeSet<String>>::new();
    for entry in fs::read_dir(content.join("npcs")).expect("NPC content directory reads") {
        let path = entry.expect("NPC directory entry reads").path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let region = read_json(&path);
            for npc in region["records"]
                .as_array()
                .expect("NPC records are an array")
            {
                let id = npc["npc_id"].as_u64().expect("NPC ID is an integer");
                let roles = npc["roles"]
                    .as_array()
                    .expect("NPC roles are an array")
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect();
                assert!(npc_roles.insert(id, roles).is_none(), "duplicate NPC {id}");
            }
        }
    }

    let vendor_data = read_json(&content.join("vendors.json"));
    let vendor_rows = vendor_data["vendors"]
        .as_array()
        .expect("resolved vendors are an array");
    let mut vendor_keys = BTreeSet::new();
    let mut item_rows = 0usize;
    let mut runtime_items = 0usize;
    let mut runtime_vendors = 0usize;

    for vendor in vendor_rows {
        let npc_id = vendor["npc_id"]
            .as_u64()
            .expect("vendor NPC ID is an integer");
        assert!(vendor_keys.insert(npc_id), "duplicate vendor {npc_id}");
        assert_eq!(
            npc_roles.get(&npc_id).map(|roles| roles.contains("vendor")),
            Some(true)
        );

        let items = vendor["items"]
            .as_array()
            .expect("vendor items are an array");
        let mut expected_runtime_ids = Vec::new();
        let mut seen_ids = BTreeSet::new();
        for item in items {
            item_rows += 1;
            let item_id = item["item_id"]
                .as_i64()
                .expect("mapped item ID is an integer");
            assert!(
                catalogue_ids.contains(&item_id),
                "item {item_id} is absent from startup catalogue"
            );
            assert!(
                seen_ids.insert(item_id),
                "duplicate item {item_id} for vendor {npc_id}"
            );
            assert!(
                item["source_tags"]
                    .as_array()
                    .is_some_and(|tags| !tags.is_empty())
            );
            assert_eq!(
                item["source_tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|tag| tag == "client"),
                true
            );
            assert!(item["stock_source"].is_object());
            assert!(item["item_source"]["name_id"].as_i64().is_some());
            assert!(item["item_source"]["english_text"].as_str().is_some());

            let has_store_price = prices.contains_key(&item_id);
            assert_eq!(item["runtime_eligible"].as_bool(), Some(has_store_price));
            if let Some(price) = prices.get(&item_id) {
                for field in [
                    "purchase_price_coins",
                    "member_price_coins",
                    "sell_price_coins",
                ] {
                    assert_eq!(
                        item[field], price[field],
                        "{field} mismatch for item {item_id}"
                    );
                }
                assert_eq!(item["price_source"]["kind"], "stores.json");
                expected_runtime_ids.push(item_id);
                runtime_items += 1;
            } else {
                for field in [
                    "purchase_price_coins",
                    "member_price_coins",
                    "sell_price_coins",
                ] {
                    assert!(item[field].is_null(), "unpriced item {item_id} has {field}");
                }
                assert!(item["price_source"].is_null());
            }
            assert!(item["member_only"].is_null() || item["member_only"].is_boolean());
        }

        let actual_ids: Vec<i64> = vendor["item_ids"]
            .as_array()
            .expect("runtime item_ids are an array")
            .iter()
            .map(|value| value.as_i64().expect("runtime item ID is an integer"))
            .collect();
        assert_eq!(
            actual_ids, expected_runtime_ids,
            "runtime list differs for NPC {npc_id}"
        );
        if !actual_ids.is_empty() {
            runtime_vendors += 1;
        }
    }

    let unresolved = read_json(&content.join("vendors_unresolved.json"));
    let unresolved_rows = unresolved["unresolved"]
        .as_array()
        .expect("unresolved vendor records are an array");
    let mut unresolved_keys = BTreeSet::new();
    for row in unresolved_rows {
        let npc_id = row["npc_id"]
            .as_u64()
            .expect("unresolved vendor NPC ID is an integer");
        assert!(
            unresolved_keys.insert(npc_id),
            "duplicate unresolved vendor {npc_id}"
        );
        assert!(
            npc_roles
                .get(&npc_id)
                .is_some_and(|roles| roles.contains("vendor"))
        );
        assert!(
            row["reason"]
                .as_str()
                .is_some_and(|reason| !reason.trim().is_empty())
        );
        for issue in row["unresolved_items"]
            .as_array()
            .expect("unresolved items are an array")
        {
            assert!(
                issue["reason"]
                    .as_str()
                    .is_some_and(|reason| !reason.trim().is_empty())
            );
            assert!(issue["status"].as_str().is_some());
        }
    }
    assert_eq!(unresolved_keys.len(), 192);

    let candidate_vendor_ids: BTreeSet<u64> = unresolved["unresolved_stock_claims"]
        .as_array()
        .expect("unresolved cross-source claims are an array")
        .iter()
        .flat_map(|claim| claim["candidate_npc_ids"].as_array().into_iter().flatten())
        .map(|value| value.as_u64().expect("candidate NPC ID is an integer"))
        .collect();
    assert!(
        candidate_vendor_ids
            .iter()
            .all(|id| unresolved_keys.contains(id))
    );

    let coverage = &vendor_data["coverage"];
    assert_eq!(coverage["items_resolved"].as_u64(), Some(item_rows as u64));
    assert_eq!(
        coverage["items_with_stores_price"].as_u64(),
        Some(runtime_items as u64)
    );
    assert_eq!(
        coverage["vendors_with_runtime_item_ids"].as_u64(),
        Some(runtime_vendors as u64)
    );
    assert_eq!(coverage["member_only_flags_recovered"].as_u64(), Some(0));
    println!(
        "vendor coverage: {} vendor keys; {} have runtime stock; {} mapped items ({} priced/runtime-ready, {} ambiguous, {} unmatched)",
        unresolved_keys.len(),
        runtime_vendors,
        item_rows,
        runtime_items,
        coverage["items_ambiguous"].as_u64().unwrap_or_default(),
        coverage["items_unmatched"].as_u64().unwrap_or_default()
    );
}
