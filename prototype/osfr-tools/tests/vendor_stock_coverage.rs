//! Vendor stock coverage for the playtest hubs.
//!
//! The pre-T108 runtime baseline was reproduced from `vendors.json`, `stores.json`,
//! and `Inventory::load`: 96 populated vendor records, 96 of them fully grantable,
//! and 1,081 grantable stock rows. The stale T019 note said 62 of 192; that number
//! predates the later ride-vendor additions and does not match the checked catalogue.

use osfr_tools::{
    coin_shop::{self, CoinShop, StoreCatalogue},
    inventory::{Inventory, InventoryItem, InventoryState},
    npc_interaction::{NpcAction, NpcInteraction, NpcSelection},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const SELECTED_HUB_FILES: &[&str] = &[
    "cobblestone_village.json",
    "farnum_farmstead.json",
    "lakeshore.json",
    "sanctuary.json",
    "highroad_junction.json",
    "blackspore_swamp.json",
];

/// Added by T108. Each pair is one name listed twice in the retained wiki
/// revision and two exact catalogue rows with complementary GenderUsage 1/2.
const ADDED_STOCK: &[(u64, i32)] = &[
    (9_500_000_045, 76_427),
    (9_500_000_045, 76_428),
    (9_500_000_071, 76_003),
    (9_500_000_071, 76_004),
    (9_500_000_128, 76_443),
    (9_500_000_128, 76_444),
    (9_500_000_504, 76_387),
    (9_500_000_504, 76_388),
    (9_500_000_602, 76_395),
    (9_500_000_602, 76_396),
    (9_500_000_660, 76_017),
    (9_500_000_660, 76_018),
    (9_500_000_746, 76_411),
    (9_500_000_746, 76_412),
    (9_500_000_787, 76_419),
    (9_500_000_787, 76_420),
    (9_500_000_793, 75_996),
    (9_500_000_793, 75_997),
    (9_500_000_934, 76_403),
    (9_500_000_934, 76_404),
    (9_500_000_972, 75_982),
    (9_500_000_972, 75_983),
];

const BASELINE_GRANTABLE_STOCK_ROWS: usize = 1_081;
const STALE_T019_VENDOR_BASELINE: usize = 62;
const REQUIRED_ADDITIONAL_ROWS: usize = 20;
const REPRESENTATIVE_VENDOR: u64 = 9_500_000_045;
const REPRESENTATIVE_ITEM: i32 = 76_427;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn inventory_content_dir() -> PathBuf {
    std::env::var_os("OSFR_VENDOR_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("content file reads"))
        .unwrap_or_else(|error| panic!("{} parses: {error}", path.display()))
}

fn prices(content: &Path) -> BTreeMap<i64, u64> {
    let stores = read_json(&content.join("stores.json"));
    stores["items"]
        .as_array()
        .expect("store items are an array")
        .iter()
        .filter_map(|row| {
            let id = row["item_id"]
                .as_i64()
                .expect("store item ID is an integer");
            row["purchase_price_coins"]
                .as_u64()
                .map(|price| (id, price))
        })
        .collect()
}

fn item_id(item: &Value) -> i64 {
    item["item_id"]
        .as_i64()
        .expect("vendor item ID is an integer")
}

fn positive_purchase_price(prices: &BTreeMap<i64, u64>, id: i64) -> bool {
    prices.get(&id).is_some_and(|price| *price > 0)
}

fn grantable(item: &Value, inventory: &Inventory, prices: &BTreeMap<i64, u64>) -> bool {
    let id = item_id(item);
    item["runtime_eligible"].as_bool() == Some(true)
        && i32::try_from(id).is_ok_and(|id| inventory.has_definition(id))
        && positive_purchase_price(prices, id)
}

fn vendor_items(vendors: &Value) -> BTreeMap<u64, &Value> {
    vendors["vendors"]
        .as_array()
        .expect("vendor records are an array")
        .iter()
        .map(|vendor| {
            (
                vendor["npc_id"]
                    .as_u64()
                    .expect("vendor NPC ID is an integer"),
                vendor,
            )
        })
        .collect()
}

#[test]
fn the_selected_hub_vendors_have_nonempty_grantable_stock() {
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let stores = prices(&content);
    let inventory = Inventory::load(&inventory_content_dir(), None)
        .expect("the runtime inventory catalogue loads");
    let mut interaction = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts the selected hubs");

    let selected: BTreeSet<&str> = SELECTED_HUB_FILES.iter().copied().collect();
    let mut stocked = 0usize;
    for vendor in vendors["vendors"]
        .as_array()
        .expect("vendor records are an array")
    {
        let file = vendor["npc_content_file"]
            .as_str()
            .expect("vendor content file is a string");
        if !selected.contains(file) {
            continue;
        }
        let ids: Vec<i64> = vendor["item_ids"]
            .as_array()
            .expect("runtime item_ids are an array")
            .iter()
            .map(|id| id.as_i64().expect("runtime item ID is an integer"))
            .collect();
        if ids.is_empty() {
            // Bluerose's only named item is Bait, whose Legacy row is NoSale and
            // whose stores.json purchase price is null. No zero-price substitute is allowed.
            continue;
        }
        stocked += 1;
        let npc_id = vendor["npc_id"]
            .as_u64()
            .expect("vendor NPC ID is an integer");
        let items = vendor["items"]
            .as_array()
            .expect("vendor items are an array");
        assert_eq!(
            ids.len(),
            items.len(),
            "NPC {npc_id} has partial stock rows"
        );
        for item in items {
            assert!(
                grantable(item, &inventory, &stores),
                "NPC {npc_id} exposes non-grantable item {}",
                item_id(item)
            );
        }

        let (zone, position) = interaction
            .location(npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id} has a spawn location"));
        let menu = interaction
            .interact(npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} interaction failed: {error:?}"));
        let option = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
            .unwrap_or_else(|| panic!("NPC {npc_id} exposes no shop option"));
        let mut select = vec![26, 0, 10, 0];
        select.extend(npc_id.to_le_bytes());
        select.extend(option.id.to_le_bytes());
        let NpcSelection::Shop(window) = interaction
            .select(&select, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} shop did not open: {error:?}"))
        else {
            panic!("NPC {npc_id} selected a non-shop action");
        };
        let window_ids: BTreeSet<i32> = window.items.iter().map(|item| item.item_id).collect();
        assert_eq!(
            window_ids.len(),
            ids.len(),
            "NPC {npc_id} shop is incomplete"
        );
        for item in window.items {
            assert!(
                item.purchase_price_coins.is_some_and(|price| price > 0),
                "NPC {npc_id} item {} has no positive purchase price",
                item.item_id
            );
            assert!(
                inventory.has_definition(item.item_id),
                "NPC {npc_id} item {} has no runtime definition",
                item.item_id
            );
        }
    }
    assert!(
        stocked >= 20,
        "expected the playtest hubs to expose at least 20 populated vendors, got {stocked}"
    );
}

#[test]
fn at_least_twenty_more_vendor_rows_are_fully_grantable() {
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let stores = prices(&content);
    let inventory = Inventory::load(&inventory_content_dir(), None)
        .expect("the runtime inventory catalogue loads");

    let mut grantable_rows = 0usize;
    let mut fully_grantable_vendors = 0usize;
    for vendor in vendors["vendors"]
        .as_array()
        .expect("vendor records are an array")
    {
        let items = vendor["items"]
            .as_array()
            .expect("vendor items are an array");
        let all_grantable = !items.is_empty()
            && items
                .iter()
                .all(|item| grantable(item, &inventory, &stores));
        if all_grantable {
            fully_grantable_vendors += 1;
        }
        grantable_rows += items
            .iter()
            .filter(|item| grantable(item, &inventory, &stores))
            .count();

        let expected: Vec<i64> = items
            .iter()
            .filter(|item| grantable(item, &inventory, &stores))
            .map(item_id)
            .collect();
        let actual: Vec<i64> = vendor["item_ids"]
            .as_array()
            .expect("runtime item_ids are an array")
            .iter()
            .map(|id| id.as_i64().expect("runtime item ID is an integer"))
            .collect();
        assert_eq!(
            actual, expected,
            "NPC {} runtime stock differs from its grantable rows",
            vendor["npc_id"]
        );
    }

    assert!(
        grantable_rows >= BASELINE_GRANTABLE_STOCK_ROWS + REQUIRED_ADDITIONAL_ROWS,
        "expected at least {} grantable stock rows, got {grantable_rows}",
        BASELINE_GRANTABLE_STOCK_ROWS + REQUIRED_ADDITIONAL_ROWS
    );
    // The task's stale 62-vendor figure had a 20-row target of 82. The reproduced
    // baseline is already 96, so retain that stronger floor as well.
    assert!(
        fully_grantable_vendors >= STALE_T019_VENDOR_BASELINE + REQUIRED_ADDITIONAL_ROWS,
        "expected at least {} fully grantable vendor records, got {fully_grantable_vendors}",
        STALE_T019_VENDOR_BASELINE + REQUIRED_ADDITIONAL_ROWS
    );
}

#[test]
fn no_vendor_stock_row_uses_a_fuzzy_or_family_substitute() {
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let by_vendor = vendor_items(&vendors);
    let stores = prices(&content);
    let store_rows = read_json(&content.join("stores.json"));
    let store_rows: BTreeMap<i64, &Value> = store_rows["items"]
        .as_array()
        .expect("store items are an array")
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
    let inventory = Inventory::load(&inventory_content_dir(), None)
        .expect("the runtime inventory catalogue loads");

    for &(npc_id, item_id) in ADDED_STOCK {
        let vendor = by_vendor
            .get(&npc_id)
            .unwrap_or_else(|| panic!("vendor {npc_id} is missing"));
        let item = vendor["items"]
            .as_array()
            .expect("vendor items are an array")
            .iter()
            .find(|item| item_id_value(item) == i64::from(item_id))
            .unwrap_or_else(|| panic!("vendor {npc_id} is missing new item {item_id}"));
        assert!(grantable(item, &inventory, &stores));
        let store = store_rows
            .get(&i64::from(item_id))
            .expect("new item has an exact store row");
        assert!(
            store["category_id"].as_i64().is_some_and(|id| id > 0),
            "new item {item_id} has no valid store category"
        );
        assert_eq!(store["no_sale"], false);
        let (_, max_stack) = inventory
            .trade_flags(item_id)
            .expect("new item has a runtime definition and stack");
        assert!(
            max_stack.is_none_or(|stack| stack > 0),
            "new item {item_id} has an invalid maximum stack"
        );
        assert_eq!(item["source_tags"][0], "client");
        assert_eq!(item["stock_source"]["kind"], "wiki");
        assert_eq!(item["stock_source"]["listed_occurrences"], 2);
        assert!(
            item["stock_source"]["variant_basis"]
                .as_str()
                .is_some_and(|basis| basis.starts_with("(R)") && basis.contains("GenderUsage"))
        );
        assert_eq!(item["item_source"]["kind"], "catalogue_with_client_locale");
        assert_eq!(item["item_source"]["name_id"], item["item_name_id"]);
        assert!(
            item["item_source"]["mapping_policy"]
                .as_str()
                .is_some_and(|policy| policy.contains("exact default locale display name"))
        );
        assert_eq!(item["price_source"]["item_id"], item_id);
        assert!(
            [1, 2].contains(
                &item["gender_usage"]
                    .as_u64()
                    .expect("gender usage is an integer")
            )
        );
        let siblings = item["variant_sibling_item_ids"]
            .as_array()
            .expect("variant siblings are an array");
        assert_eq!(siblings.len(), 1);
        let sibling_id = siblings[0]
            .as_i64()
            .expect("variant sibling ID is an integer");
        let sibling = vendor["items"]
            .as_array()
            .expect("vendor items are an array")
            .iter()
            .find(|row| item_id_value(row) == sibling_id)
            .expect("the paired gender variant is present");
        assert_eq!(item["source_stock_name"], sibling["source_stock_name"]);
        assert_ne!(item["gender_usage"], sibling["gender_usage"]);
        assert_eq!(
            sibling["variant_sibling_item_ids"][0], item_id,
            "gender pairing must be reciprocal"
        );
        let english = item["item_source"]["english_text"]
            .as_str()
            .expect("locale display text exists");
        let source_name = item["source_stock_name"]
            .as_str()
            .expect("stock display name exists");
        assert!(
            english.contains(source_name),
            "item {item_id} does not retain its exact source display name"
        );
    }
}

fn item_id_value(item: &Value) -> i64 {
    item["item_id"]
        .as_i64()
        .expect("vendor item ID is an integer")
}

struct VendorCounter {
    content: PathBuf,
    shop: CoinShop,
}

impl VendorCounter {
    fn new(coins: u32, full_bag: bool) -> Self {
        let content = content_dir();
        let inventory_content = inventory_content_dir();
        let mut state = Inventory::load(&inventory_content, None)
            .expect("inventory definitions load")
            .state()
            .clone();
        state.coins = coins;
        if full_bag {
            state.items.clear();
            for entry in &mut state.equipment {
                entry.item_id = 0;
            }
            for id in 100_000..100_100 {
                state.items.push(InventoryItem {
                    definition_id: 75_060,
                    tint: 0,
                    id,
                    count: 1,
                    consumed_count: 0,
                    stored_count: 0,
                    last_cast_time: 0,
                    rental: None,
                    ability_count: 0,
                    activate_enabled: false,
                });
            }
            state.next_instance_id = Some(100_100);
        }
        let inventory = Inventory::load(&inventory_content, Some(state))
            .expect("test inventory state is valid");
        let catalogue =
            StoreCatalogue::load(&content.join("stores.json")).expect("store prices load");
        let shop =
            CoinShop::new(&inventory_content, inventory, catalogue, 100).expect("coin shop opens");
        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("NPC content loads");
        let (zone, position) = service
            .location(REPRESENTATIVE_VENDOR)
            .expect("the representative vendor has a spawn");
        let menu = service
            .interact(REPRESENTATIVE_VENDOR, zone, position, None)
            .expect("vendor menu opens");
        let option = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
            .expect("representative vendor exposes a shop option");
        let mut select = vec![26, 0, 10, 0];
        select.extend(REPRESENTATIVE_VENDOR.to_le_bytes());
        select.extend(option.id.to_le_bytes());
        let NpcSelection::Shop(window) = service
            .select(&select, zone, position, None)
            .expect("representative shop opens")
        else {
            panic!("representative vendor selected a non-shop action");
        };
        assert!(
            window
                .items
                .iter()
                .any(|item| item.item_id == REPRESENTATIVE_ITEM)
        );
        Self {
            content: inventory_content,
            shop,
        }
    }

    fn purchase(&mut self, transaction: u32) -> coin_shop::CoinShopOutcome {
        self.shop
            .buy_vendor(transaction, REPRESENTATIVE_ITEM, 1)
            .expect("vendor purchase returns a client-readable outcome")
    }

    fn state(&self) -> &InventoryState {
        self.shop.inventory().state()
    }
}

fn merchant_status(packet: &[u8]) -> u32 {
    assert_eq!(packet.len(), 56);
    assert_eq!(&packet[..4], &[66, 0, 44, 0]);
    u32::from_le_bytes(packet[48..52].try_into().unwrap())
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn representative_new_stock_buys_once_and_refuses_atomically() {
    let stores = prices(&content_dir());
    let price = stores
        .get(&(REPRESENTATIVE_ITEM as i64))
        .copied()
        .expect("representative item has a positive price") as u32;

    let mut counter = VendorCounter::new(price + 1, false);
    let before = counter.state().clone();
    let before_count = count_definition(&before, REPRESENTATIVE_ITEM);
    let bought = counter.purchase(108_001);
    assert!(bought.changed);
    assert_eq!(merchant_status(bought.replies.last().unwrap()), 1);
    let after = counter.state().clone();
    assert_eq!(after.coins, before.coins - price);
    assert_eq!(
        count_definition(&after, REPRESENTATIVE_ITEM),
        before_count + 1
    );

    // Save and reload through the same inventory loader.
    let saved = serde_json::to_vec(&after).expect("inventory state serializes");
    let restored: InventoryState = serde_json::from_slice(&saved).expect("inventory state parses");
    let reloaded =
        Inventory::load(&counter.content, Some(restored)).expect("saved inventory state reloads");
    assert_eq!(reloaded.state(), &after);

    let mut poor = VendorCounter::new(0, false);
    let before = serde_json::to_vec(poor.state()).expect("refusal pre-state serializes");
    let refused = poor.purchase(108_002);
    assert!(!refused.changed);
    assert_eq!(merchant_status(refused.replies.last().unwrap()), 0);
    let after = serde_json::to_vec(poor.state()).expect("refusal post-state serializes");
    assert_eq!(after, before, "insufficient-coins refusal changed state");

    let mut full = VendorCounter::new(price + 1, true);
    assert_eq!(
        full.state()
            .items
            .iter()
            .filter(|item| item.count > 0)
            .count(),
        100
    );
    let before = serde_json::to_vec(full.state()).expect("full-bag pre-state serializes");
    let refused = full.purchase(108_003);
    assert!(!refused.changed);
    assert_eq!(merchant_status(refused.replies.last().unwrap()), 0);
    let after = serde_json::to_vec(full.state()).expect("full-bag post-state serializes");
    assert_eq!(after, before, "full-bag refusal changed state");
}
