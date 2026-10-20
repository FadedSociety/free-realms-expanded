//! W169: every ride in `mounts.json` can be bought from a vendor with coins, at a price inside
//! the `ride_price_policy` bounds, and owning the bought item makes the mount service list the
//! ride. Before W169 only 4 of 54 rides had a store row and none had vendor stock.

use osfr_tools::{
    coin_shop::{self, CoinShop, StoreCatalogue},
    inventory::Inventory,
    mounts::MountCatalog,
};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_RIDE_SOURCES_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

fn json(name: &str) -> Value {
    serde_json::from_slice(&fs::read(content().join(name)).expect("content reads"))
        .expect("content parses")
}

#[test]
fn every_ride_is_sold_by_a_vendor_inside_the_price_bounds() {
    let stores = json("stores.json");
    let policy = &stores["ride_price_policy"];
    let (low, high) = (
        policy["min_coins"].as_u64().expect("min_coins"),
        policy["max_coins"].as_u64().expect("max_coins"),
    );
    assert!(0 < low && low < high && high <= i32::MAX as u64);
    let prices: BTreeMap<i64, &Value> = stores["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["item_id"].as_i64().unwrap(), row))
        .collect();
    let vendors = json("vendors.json");
    let stock: Vec<(u64, Vec<i64>)> = vendors["vendors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|vendor| {
            (
                vendor["npc_id"].as_u64().unwrap(),
                vendor["item_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_i64().unwrap())
                    .collect(),
            )
        })
        .collect();

    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    let inventory = Inventory::load(&inventory_content(), None).expect("item content loads");
    let mut unsold = Vec::new();
    for ride in catalog.mounts() {
        let sold = ride.item_definition_id_candidates.iter().any(|&item| {
            let item = i64::from(item);
            let on_sale = stock.iter().any(|(_, ids)| ids.contains(&item));
            let priced = prices.get(&item).is_some_and(|row| {
                row["purchase_price_coins"]
                    .as_u64()
                    .is_some_and(|p| (low..=high).contains(&p))
                    && row["member_price_coins"].as_u64() <= row["purchase_price_coins"].as_u64()
            });
            on_sale && priced && inventory.has_definition(item as i32)
        });
        if !sold {
            unsold.push(format!("{} ({})", ride.name, ride.id));
        }
    }
    assert!(unsold.is_empty(), "rides nobody sells: {unsold:?}");
}

#[test]
fn buying_each_ride_with_coins_grants_it() {
    let directory = inventory_content();
    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    let vendors = json("vendors.json");
    let sold: Vec<i64> = vendors["vendors"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|vendor| vendor["item_ids"].as_array().unwrap().clone())
        .map(|id| id.as_i64().unwrap())
        .collect();
    for ride in catalog.mounts() {
        let item = ride
            .item_definition_id_candidates
            .iter()
            .copied()
            .find(|&item| sold.contains(&i64::from(item)))
            .unwrap_or_else(|| panic!("{} has no vendor stock", ride.name));
        let loaded = Inventory::load(&directory, None).expect("item content loads");
        let mut state = loaded.state().clone();
        state.coins = 1_000_000;
        let inventory = Inventory::load(&directory, Some(state)).expect("coins are valid");
        let store = StoreCatalogue::load(&content().join("stores.json")).expect("stores load");
        let mut shop = CoinShop::new(&directory, inventory, store, usize::MAX, None).expect("shop");
        let bought = shop.buy_vendor(1, item, 1).expect("buy runs");
        assert_eq!(
            bought.result_code,
            coin_shop::RESULT_SUCCESS,
            "{}",
            ride.name
        );
        let owned = catalog.owned_mounts(shop.inventory().state());
        assert!(
            owned.iter().any(|mount| mount.id == ride.id),
            "{} is not owned after buying item {item}",
            ride.name
        );
    }
}
