//! T230: every ride is sourced from the Station Cash Marketplace, not a coin vendor.
//!
//! The two test names below are historical and deliberately retained for the protected-pass
//! list: before T230 the rides were sold by Delvin and the Mysterious Merchant for coins. T230
//! removed that stock and lists the rides for Station Cash instead. `Chrome Drake` and
//! `Polar Bear` carry `earn_only` and must never be listed (slice 2 rewards); T452 added six more
//! earn-only rides (loot-card, TCG-questline and collection rewards in the original).

use osfr_tools::{
    coin_shop::{CoinShop, StoreCatalogue},
    inventory::Inventory,
    mounts::{MountCatalog, MountSource},
    station_cash::{self, Wallet},
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf};

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

fn shop_with(wallet: Wallet, capacity: usize) -> CoinShop {
    let directory = inventory_content();
    let inventory = Inventory::load(&directory, None).expect("item content loads");
    let catalogue = StoreCatalogue::load(&content().join("stores.json")).expect("stores load");
    CoinShop::new(&directory, inventory, catalogue, capacity, None, wallet).expect("shop loads")
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client order serializer `00b3fc60`/`00b3af80` (66/3), with one detail.
fn order_packet(tracking: u32, bundle: u32, store: u32, quantity: u32) -> Vec<u8> {
    let mut bytes = vec![66, 0, 3, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(bundle.to_le_bytes());
    bytes.extend(store.to_le_bytes());
    bytes.extend(quantity.to_le_bytes());
    put_string(&mut bytes, "0");
    put_string(&mut bytes, "");
    bytes
}

fn store_row(stores: &Value, item_id: i32) -> Option<&Value> {
    stores["items"]
        .as_array()
        .expect("store items are an array")
        .iter()
        .find(|row| row["item_id"].as_i64() == Some(i64::from(item_id)))
}

#[test]
fn every_ride_is_sold_by_a_vendor_inside_the_price_bounds() {
    let stores = json("stores.json");
    let policy = &stores["station_cash"];
    let scale = policy["sc_price_scale"].as_f64().expect("scale");
    let low = policy["min_price_sc"].as_u64().expect("min_price_sc");
    let high = policy["max_price_sc"].as_u64().expect("max_price_sc");
    assert!(0 < low && low <= high && scale > 0.0);

    let vendors = json("vendors.json");
    let stocked: Vec<i64> = vendors["vendors"]
        .as_array()
        .expect("vendors are an array")
        .iter()
        .flat_map(|vendor| {
            vendor["item_ids"]
                .as_array()
                .expect("stock is an array")
                .clone()
        })
        .filter_map(|id| id.as_i64())
        .collect();
    for row in stores["items"].as_array().expect("store items") {
        if row["sc_price"].as_u64().is_some() {
            let id = row["item_id"].as_i64().expect("item id");
            assert!(
                !stocked.contains(&id),
                "ride item {id} is still coin vendor stock"
            );
            assert!(
                row["purchase_price_coins"].is_null(),
                "ride item {id} still has a coin price"
            );
            assert!(
                row["member_price_coins"].is_null(),
                "ride item {id} still has a member coin price"
            );
            assert!(row["sell_price_coins"].is_null());
            assert_eq!(row["no_sale"], true);
        }
    }

    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    let inventory = Inventory::load(&inventory_content(), None).expect("item content loads");
    let catalogue = StoreCatalogue::load(&content().join("stores.json")).expect("stores load");
    let shop = CoinShop::new(
        &inventory_content(),
        inventory,
        catalogue,
        usize::MAX,
        None,
        Wallet::default(),
    )
    .expect("shop loads");
    let sc = shop.sc_catalogue();
    let mut rides = 0usize;
    for ride in catalog.mounts() {
        rides += 1;
        let row = ride
            .item_definition_id_candidates
            .iter()
            .find_map(|&item| store_row(&stores, item).map(|row| (item, row)))
            .unwrap_or_else(|| panic!("{} has no Station Cash store row", ride.name));
        let (item, row) = row;
        let sc_price = row["sc_price"]
            .as_u64()
            .unwrap_or_else(|| panic!("{} has no sc_price", ride.name));
        let scaled = (sc_price as f64 * scale).round() as u64;
        assert!(
            (low..=high).contains(&scaled),
            "{} scaled price {scaled} is outside {low}..={high}",
            ride.name
        );
        if row["earn_only"].as_bool() == Some(true) {
            assert!(
                sc.listing(item as u32).is_none(),
                "{} is earn-only but listed",
                ride.name
            );
            assert!(
                sc.earn_only()
                    .iter()
                    .any(|listing| listing.item_id == item as i32),
                "{} is not in the earn-only catalogue",
                ride.name
            );
        } else {
            let listing = sc
                .listing(item as u32)
                .unwrap_or_else(|| panic!("{} is not listed", ride.name));
            assert_eq!(listing.price as u64, scaled, "{} price", ride.name);
        }
    }
    // T452 added 35 OSFR-parity rides (6 of them earn-only) to the 54 seeded ones.
    assert_eq!(rides, 89, "the ride catalogue changed");
    assert_eq!(shop.sc_catalogue().earn_only().len(), 8);
}

/// T452: the rides Sanctuary-main 10f129e `Mounts.json` ships and the seed lacked. The Futuristic
/// Bicycle (13464) waits for T429: its item 79417 is the only tradeable ride definition.
const OSFR_PARITY_RIDES: [i32; 35] = [
    12638, 520, 13562, 13563, 13590, 13591, 13592, 13566, 13578, 895, 13580, 13565, 8765, 13570,
    12636, 13569, 13573, 13575, 13582, 13593, 13588, 13585, 13586, 13577, 13579, 13572, 13571,
    13564, 681, 699, 13584, 13463, 13574, 13581, 926,
];

/// T452: each added ride's one item is a client ride definition that presents the ride's own name
/// and icon (definition bytes 4..8 and 12..16). Item type 19 is a ride: the client's item-record
/// action switch on the definition's type (+0x6c; `00a1ec00`, switch at `00a1edf0`) collects case
/// 19 at `00a1eec0` and spawns it through SpawnMountByItemDefinitionId (`00b06db0`).
#[test]
fn osfr_parity_rides_match_their_item_definitions() {
    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    let inventory = Inventory::load(&inventory_content(), None).expect("item content loads");
    for id in OSFR_PARITY_RIDES {
        let ride = catalog
            .mount(id)
            .unwrap_or_else(|| panic!("ride {id} is not catalogued"));
        let [item] = ride.item_definition_id_candidates[..] else {
            panic!("{} must have exactly one item", ride.name);
        };
        let presentation = inventory
            .presentation(item)
            .unwrap_or_else(|error| panic!("{} item {item}: {error}", ride.name));
        assert_eq!(
            presentation.name_id, ride.name_string_id,
            "{} name",
            ride.name
        );
        assert_eq!(
            presentation.icon_id, ride.image_set_id,
            "{} icon",
            ride.name
        );
        assert_eq!(presentation.item_type, 19, "{} item type", ride.name);
    }
}

/// T452 review: the item catalogue's `legacy+client-if-present` rows are FreeRealms-Legacy
/// `ClientItemDefinitions.json` data (none of these items is in the client overlay), so evidence
/// citing an added ride's catalogue row is `legacy` unless the row is a captured
/// `inventory-baseline` definition. Which source sets a ride's tint is open, so every added
/// ride's tint alias evidence carries (R).
#[test]
fn osfr_parity_ride_evidence_names_its_real_source() {
    #[derive(serde::Deserialize)]
    struct Row {
        id: i32,
        #[serde(default)]
        source: String,
    }
    #[derive(serde::Deserialize)]
    struct ItemCatalogue {
        definitions: Vec<Row>,
    }
    fn evidence(source: &MountSource) -> &str {
        match source {
            MountSource::Client(text)
            | MountSource::Legacy(text)
            | MountSource::Wiki(text)
            | MountSource::Reconstructed(text) => text,
        }
    }
    let bytes = fs::read(inventory_content().join("item-catalogue.json")).expect("catalogue reads");
    let items: ItemCatalogue = serde_json::from_slice(&bytes).expect("catalogue parses");
    let captured: BTreeSet<i32> = items
        .definitions
        .iter()
        .filter(|row| row.source == "inventory-baseline")
        .map(|row| row.id)
        .collect();
    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    for id in OSFR_PARITY_RIDES {
        let ride = catalog.mount(id).expect("ride catalogued");
        let item = ride.item_definition_id_candidates[0];
        let row = format!("item-catalogue.json row {item}");
        let cited: Vec<_> = (ride.source.iter())
            .chain(ride.field_sources.values().flatten())
            .filter(|source| evidence(source).contains(&row))
            .collect();
        assert!(!cited.is_empty(), "{} cites its item row", ride.name);
        for source in cited {
            let right = if captured.contains(&item) {
                matches!(source, MountSource::Client(_))
            } else {
                matches!(source, MountSource::Legacy(_))
            };
            assert!(right, "{} item {item}: {source:?}", ride.name);
        }
        let tint = &ride.field_sources["tint_alias"];
        assert!(
            !tint.is_empty() && tint.iter().all(|source| evidence(source).contains("(R)")),
            "{} tint alias evidence needs (R)",
            ride.name
        );
    }
}

#[test]
fn buying_each_ride_with_coins_grants_it() {
    // The historical name is retained; T230 pays with Station Cash and the old coin stock is
    // gone. Every listed ride must be granted once for exactly the listed price.
    let catalog = MountCatalog::load(content().join("mounts.json")).expect("mounts load");
    let mut wallet = Wallet::default();
    wallet.balance = 1_000_000;
    let mut shop = shop_with(wallet, usize::MAX);
    let store = shop.sc_catalogue().config().store_id;
    // L046: lot deeds share the listing; they grant a lot, never an item (tests/lot_deeds.rs).
    // T634: so do the wiki-priced items that are no ride (tests/station_cash.rs).
    let listings: Vec<_> = shop
        .sc_catalogue()
        .listings()
        .iter()
        .filter(|listing| {
            !listing.deed
                && catalog.mounts().any(|ride| {
                    ride.item_definition_id_candidates
                        .contains(&listing.item_id)
                })
        })
        .copied()
        .collect();
    // Every ride but the 8 earn-only ones is listed.
    assert_eq!(listings.len(), 81);
    let mut expected = 1_000_000u32;
    for (index, listing) in listings.iter().enumerate() {
        let ride = catalog
            .mounts()
            .find(|ride| {
                ride.item_definition_id_candidates
                    .contains(&listing.item_id)
            })
            .unwrap_or_else(|| panic!("item {} belongs to no ride", listing.item_id));
        let packet = order_packet(index as u32 + 1, listing.bundle_id, store, 1);
        let outcome = shop.handle(&packet).expect("order parses");
        assert_eq!(
            outcome.result_code,
            station_cash::ORDER_SUCCESS,
            "{}",
            ride.name
        );
        assert!(outcome.changed);
        expected -= listing.price;
        assert_eq!(shop.sc_wallet().balance, expected, "{}", ride.name);
        let owned = catalog.owned_mounts(shop.inventory().state());
        assert!(
            owned.iter().any(|owned| owned.id == ride.id),
            "{} is not owned after its order",
            ride.name
        );
    }
    assert!(listings.iter().all(|listing| listing.price > 0));
}
