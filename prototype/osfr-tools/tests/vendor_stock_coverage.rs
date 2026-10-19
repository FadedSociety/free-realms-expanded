//! Vendor stock coverage for the playtest hubs.
//!
//! The pre-T108 runtime baseline was reproduced from `vendors.json`, `stores.json`,
//! and `Inventory::load`: 96 populated vendor records, 96 of them fully grantable,
//! and 1,081 grantable stock rows. The stale T019 note said 62 of 192; that number
//! predates the later ride-vendor additions and does not match the checked catalogue.

use osfr_tools::{
    account_store::AccountRole,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState},
    npc_interaction::{
        MerchantBuyRequestV2, NpcAction, NpcInteraction, NpcSelection, decode_merchant_list_66_42,
        encode_merchant_buy_66_43,
    },
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
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
/// Reproduced from the pre-T108 vendor records: 96 of the 98 records were
/// fully grantable. The task's stale "62 of 192" figure was 20 vendor records
/// short of the reproduced baseline, so the selected-hub fallback below is the
/// authorized completion path. A later content change may raise this floor but
/// must not lower it.
const REPRODUCED_FULLY_GRANTABLE_VENDORS: usize = 96;
const REQUIRED_ADDITIONAL_ROWS: usize = 20;
/// Resolved `vendors.json` records in the six selected hubs. Vendor-role NPCs
/// without a stock record are a separate content gap.
const SELECTED_HUB_VENDOR_COUNT: usize = 30;
const REPRESENTATIVE_VENDOR: u64 = 9_500_000_045;
const REPRESENTATIVE_ITEM: i32 = 76_427;

/// The selected hubs contain one vendor whose only named stock cannot be sold.
/// It is listed explicitly so a new empty vendor cannot silently inherit the
/// old `continue` and satisfy the playtest gate without stock.
struct StockException {
    npc_id: u64,
    source: &'static str,
    reason: &'static str,
}

const SELECTED_HUB_STOCK_EXCEPTIONS: &[StockException] = &[StockException {
    npc_id: 9_000_000_027,
    source: "data/content/vendors.json vendor_stock_description, recorded from \
             notes/Build Guides/Content/NPCs/NPCs - Snowhill.md#Big J and Bluerose",
    reason: "(R) Bait (57012) is the only named stock; its stores.json purchase price is null and \
             its client row is NoSale, so no purchasable item_id exists.",
}];

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
    let mut selected_vendors = 0usize;
    let mut empty = BTreeSet::new();
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
        selected_vendors += 1;
        let npc_id = vendor["npc_id"]
            .as_u64()
            .expect("vendor NPC ID is an integer");
        let ids: Vec<i64> = vendor["item_ids"]
            .as_array()
            .expect("runtime item_ids are an array")
            .iter()
            .map(|id| id.as_i64().expect("runtime item ID is an integer"))
            .collect();
        if ids.is_empty() {
            let exception = SELECTED_HUB_STOCK_EXCEPTIONS
                .iter()
                .find(|exception| exception.npc_id == npc_id)
                .unwrap_or_else(|| {
                    panic!("NPC {npc_id} exposes no stock and has no recorded exception")
                });
            assert!(!exception.source.is_empty() && !exception.reason.is_empty());
            let items = vendor["items"]
                .as_array()
                .expect("vendor items are an array");
            assert!(!items.is_empty(), "NPC {npc_id} has no named stock rows");
            assert!(
                items
                    .iter()
                    .all(|item| !grantable(item, &inventory, &stores)),
                "NPC {npc_id} has a grantable row but no runtime item_ids"
            );
            assert!(
                items.iter().any(|item| {
                    item["purchase_price_coins"].is_null() && item["runtime_eligible"] == false
                }),
                "NPC {npc_id} exception lacks the null-price NoSale row: {} ({})",
                exception.reason,
                exception.source
            );
            assert_eq!(vendor["npc_name"], "Bluerose");
            assert!(
                vendor["vendor_stock_description"]
                    .as_str()
                    .is_some_and(|description| description.starts_with("Bait.")),
                "NPC {npc_id} exception source no longer names the Bait stock"
            );
            assert_eq!(vendor["stock_completeness"], "partial_evidence_reference");
            empty.insert(npc_id);
            continue;
        }
        stocked += 1;
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
    let expected_exceptions: BTreeSet<u64> = SELECTED_HUB_STOCK_EXCEPTIONS
        .iter()
        .map(|exception| exception.npc_id)
        .collect();
    assert_eq!(
        selected_vendors, SELECTED_HUB_VENDOR_COUNT,
        "the selected-hub vendor set changed; update the enumeration deliberately"
    );
    assert_eq!(
        empty, expected_exceptions,
        "selected-hub empty-stock vendors changed; every exception needs a source and reason"
    );
    assert_eq!(
        stocked,
        SELECTED_HUB_VENDOR_COUNT - expected_exceptions.len(),
        "every selected-hub vendor must be stocked or explicitly excepted"
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
    // T108 could not add 20 more vendor records: the file has 98 records and 96
    // were already fully grantable. The selected-hub fallback above accounts
    // for every vendor in the playtest hubs, including the one sourced
    // exception. The improvement is therefore enforced on grantable stock rows,
    // while this floor prevents a vendor-coverage regression.
    assert!(
        fully_grantable_vendors >= REPRODUCED_FULLY_GRANTABLE_VENDORS,
        "expected at least {REPRODUCED_FULLY_GRANTABLE_VENDORS} fully grantable vendor records, \
         got {fully_grantable_vendors}"
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

#[test]
fn unresolved_vendor_names_keep_the_exact_utf8_evidence() {
    let text = fs::read_to_string(content_dir().join("vendors_unresolved.json"))
        .expect("unresolved vendor evidence is UTF-8");
    for name in [
        "Warrior\u{2019}s Double Axe of Berserking",
        "Warrior\u{2019}s Warlord Axe of Berserking",
    ] {
        assert!(text.contains(name), "missing exact evidence name {name:?}");
    }
    assert!(
        !text.contains("Warrior\u{00e2}\u{20ac}\u{2122}s"),
        "mojibaked Warrior name returned"
    );
}

fn item_id_value(item: &Value) -> i64 {
    item["item_id"]
        .as_i64()
        .expect("vendor item ID is an integer")
}

const NOW: u64 = 10_000;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["f143-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn inner_packets(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| packet[7..].to_vec())
        .collect()
}

fn send(gateway: &mut Gateway, packet: &[u8]) -> Vec<Vec<u8>> {
    inner_packets(
        &gateway
            .handle_at(&wrap(packet), NOW)
            .expect("gateway handles the request"),
    )
}

fn inventory_of(gateway: &Gateway) -> InventoryState {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay state exists")
        .inventory
}

fn npc_interact(npc_id: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.push(0);
    packet
}

fn npc_select(npc_id: u64, option: u32) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(npc_id.to_le_bytes());
    packet.extend(option.to_le_bytes());
    packet
}

fn vendor_buy_packet(transaction: u32, item: i32, quantity: u32) -> Vec<u8> {
    encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [item as u32, 0],
        context_dwords: [0; 3],
        context_bytes: Vec::new(),
        trailing_dwords: [quantity, 0, transaction],
    })
    .expect("66/43 request encodes")
}

#[test]
fn vendor_buy_wire_matches_the_client_reader_layout() {
    // Client serializer 0x00b3ab20: header, two dwords, three dwords, a
    // length-prefixed string, then three dwords. The empty context string makes
    // this request exactly 40 bytes.
    let packet = vendor_buy_packet(0x0102_0304, REPRESENTATIVE_ITEM, 1);
    assert_eq!(packet.len(), 40);
    assert_eq!(&packet[..4], &[66, 0, 43, 0]);
    assert_eq!(
        u32::from_le_bytes(packet[4..8].try_into().unwrap()),
        REPRESENTATIVE_ITEM as u32
    );
    assert_eq!(u32::from_le_bytes(packet[8..12].try_into().unwrap()), 0);
    assert_eq!(&packet[12..24], &[0; 12]);
    assert_eq!(u32::from_le_bytes(packet[24..28].try_into().unwrap()), 0);
    assert_eq!(u32::from_le_bytes(packet[28..32].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(packet[32..36].try_into().unwrap()), 0);
    assert_eq!(
        u32::from_le_bytes(packet[36..40].try_into().unwrap()),
        0x0102_0304
    );

    // Client reader 0x00b32430 reads seven dwords, an empty string, then five
    // dwords; object field +0x58 is wire offset 48 and is compared with 1.
    let response = osfr_tools::coin_shop::merchant_buy_result_packet(true);
    assert_eq!(response.len(), 56);
    assert_eq!(&response[..4], &[66, 0, 44, 0]);
    assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 1);
}

/// A complete gameplay fixture, built from the real vendor record and the
/// captured catalogue. Only the NPC's zone/position is translated to zone 0
/// because `Gateway::load_gameplay` without the opening campaign supports the
/// open-world zone. The stock row, prices, item definition and transaction path
/// are the shipped ones.
fn gateway_content() -> PathBuf {
    static CONTENT: OnceLock<PathBuf> = OnceLock::new();
    CONTENT
        .get_or_init(|| {
            let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
            let reference = lab.join("reports/round70-native-20261016T061000Z/content");
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock is after the epoch")
                .as_nanos();
            let content = lab.join("build").join(format!(
                "f143-vendor-gateway-{}-{stamp}",
                std::process::id()
            ));
            fs::create_dir_all(content.join("npcs")).expect("fixture content creates");
            for file in [
                "inventory.json",
                "quests.json",
                "quest-journal.bin",
                "combat.json",
                "combat-template.bin",
                "categories.json",
            ] {
                fs::copy(reference.join(file), content.join(file))
                    .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
            }
            let catalogue = content.join("item-catalogue.json");
            if fs::hard_link(reference.join("item-catalogue.json"), &catalogue).is_err() {
                fs::copy(reference.join("item-catalogue.json"), &catalogue)
                    .expect("item catalogue copies");
            }
            for file in ["stores.json", "zones.json"] {
                fs::copy(content_dir().join(file), content.join(file))
                    .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
            }

            let vendors = read_json(&content_dir().join("vendors.json"));
            let vendor = vendors["vendors"]
                .as_array()
                .expect("vendor records are an array")
                .iter()
                .find(|vendor| vendor["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
                .expect("the representative vendor exists")
                .clone();
            fs::write(
                content.join("vendors.json"),
                serde_json::to_vec(&json!({"schema_version": 1, "vendors": [vendor]}))
                    .expect("fixture vendors serialize"),
            )
            .expect("fixture vendors write");

            let npcs = read_json(&content_dir().join("npcs/blackspore_swamp.json"));
            let mut npc = npcs["records"]
                .as_array()
                .expect("NPC records are an array")
                .iter()
                .find(|npc| npc["npc_id"].as_u64() == Some(REPRESENTATIVE_VENDOR))
                .expect("the representative NPC exists")
                .clone();
            npc["zone_id"] = json!(0);
            npc["zone"] = json!("FabledRealms");
            npc["position"] = json!([0.0, 0.0, 0.0]);
            npc["heading"] = json!(0.0);
            npc["placement"] = json!("legacy_data");
            fs::write(
                content.join("npcs/melninster.json"),
                serde_json::to_vec(&json!({"schema_version": 1, "records": [npc]}))
                    .expect("fixture NPC serializes"),
            )
            .expect("fixture NPC writes");
            fs::write(
                content.join("npc_spawns_extra.json"),
                br#"{"schema_version":1,"records":[]}"#,
            )
            .expect("fixture spawn overlay writes");
            content
        })
        .clone()
}

struct GatewayFixture {
    content: PathBuf,
    reference: PathBuf,
    zone: u32,
    position: [f32; 3],
    option: u32,
    price: u32,
}

impl GatewayFixture {
    fn new() -> Self {
        let content = gateway_content();
        let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-gateway-reference");
        let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
            .expect("fixture NPC content loads");
        let (zone, position) = service
            .location(REPRESENTATIVE_VENDOR)
            .expect("the representative vendor has a location");
        let menu = service
            .interact(REPRESENTATIVE_VENDOR, zone, position, None)
            .expect("the representative vendor opens");
        let option = u32::try_from(
            menu.options
                .iter()
                .find(|option| option.action == NpcAction::Shop)
                .expect("the representative vendor exposes a shop option")
                .id,
        )
        .expect("the shop option id fits the client field");
        let price = prices(&content_dir())
            .get(&(REPRESENTATIVE_ITEM as i64))
            .copied()
            .expect("the representative item has a positive price") as u32;
        Self {
            content,
            reference,
            zone,
            position,
            option,
            price,
        }
    }

    fn state(&self, coins: u32, full_bag: bool) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("fixture gameplay content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = coins;
        let item = |id: i32| InventoryItem {
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
        };
        if full_bag {
            for id in 100_000..100_100 {
                state.inventory.items.push(item(id));
            }
            state.inventory.next_instance_id = Some(100_100);
        } else {
            state.inventory.items.push(item(1));
            state.inventory.next_instance_id = Some(2);
        }
        state
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = self.zone;
        character.position = self.position;
        character.rotation = [0.0; 3];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state))
                .expect("gateway loads the vendor content");
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway
                .handle_at(&packet, NOW)
                .expect("initialization message handles");
        }
        gateway
    }

    fn open_shop(&self, gateway: &mut Gateway) -> Vec<Vec<u8>> {
        send(gateway, &npc_interact(REPRESENTATIVE_VENDOR));
        let replies = send(gateway, &npc_select(REPRESENTATIVE_VENDOR, self.option));
        assert!(
            replies
                .iter()
                .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
            "the gateway did not open the representative shop"
        );
        replies
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
    let fixture = GatewayFixture::new();
    let price = fixture.price;
    let mut gateway = fixture.connect(fixture.state(price + 1_000, false));
    let opened = fixture.open_shop(&mut gateway);
    let list = opened
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("shop list is present");
    let list = decode_merchant_list_66_42(list).expect("shop list matches the client reader");
    assert!(
        list.group
            .entries
            .iter()
            .any(|entry| entry.first_six_dwords[0] == REPRESENTATIVE_ITEM as u32),
        "the gateway shop does not expose the new item"
    );

    let before = gateway.snapshot();
    for (label, request) in [
        ("malformed", vec![66, 0, 43, 0]),
        ("unknown item", vendor_buy_packet(108_002, i32::MAX, 1)),
        (
            "zero quantity",
            vendor_buy_packet(108_003, REPRESENTATIVE_ITEM, 0),
        ),
        (
            "unbounded quantity",
            vendor_buy_packet(108_004, REPRESENTATIVE_ITEM, u32::MAX),
        ),
    ] {
        let replies = send(&mut gateway, &request);
        assert_eq!(replies.len(), 1, "{label}: one client-visible reply");
        assert_eq!(merchant_status(&replies[0]), 0, "{label}: refused");
        assert_eq!(gateway.snapshot(), before, "{label}: gateway state changed");
    }

    // Canary: a refusal must not consume the open vendor menu.
    let bought = gateway
        .handle_at(
            &wrap(&vendor_buy_packet(108_001, REPRESENTATIVE_ITEM, 1)),
            NOW,
        )
        .expect("vendor buy handles");
    assert!(bought.gameplay_changed, "a successful buy requests a save");
    let replies = inner_packets(&bought);
    assert_eq!(
        merchant_status(replies.last().expect("vendor buy reply")),
        1
    );
    let after = gateway.snapshot();
    let before_inventory = before
        .gameplay
        .as_ref()
        .expect("pre-buy gameplay exists")
        .inventory
        .clone();
    let after_inventory = after
        .gameplay
        .as_ref()
        .expect("post-buy gameplay exists")
        .inventory
        .clone();
    assert_eq!(after_inventory.coins, before_inventory.coins - price);
    assert_eq!(
        count_definition(&after_inventory, REPRESENTATIVE_ITEM),
        count_definition(&before_inventory, REPRESENTATIVE_ITEM) + 1
    );
    assert_eq!(
        count_definition(&after_inventory, 75_060),
        1,
        "the pre-login item survived the purchase"
    );
    assert_eq!(
        after.character, before.character,
        "a vendor purchase moved the character"
    );

    // The saved GameplayState is what the session writes; reload it and
    // reconnect through the gateway rather than comparing inventory JSON.
    let saved = serde_json::to_vec(
        &after
            .gameplay
            .clone()
            .expect("successful gameplay state serializes"),
    )
    .expect("gameplay state serializes");
    let saved: GameplayState = serde_json::from_slice(&saved).expect("gameplay state parses");
    let mut reconnected = fixture.connect(saved);
    assert_eq!(inventory_of(&reconnected), after_inventory);
    let reconnect_before = reconnected.snapshot();
    let replies = send(
        &mut reconnected,
        &vendor_buy_packet(108_005, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("closed-shop reply")),
        0,
        "a reconnected player has no open shop"
    );
    assert_eq!(
        reconnected.snapshot(),
        reconnect_before,
        "closed-shop refusal changed the reconnected state"
    );

    let mut poor = fixture.connect(fixture.state(price - 1, false));
    fixture.open_shop(&mut poor);
    let poor_before = poor.snapshot();
    let replies = send(
        &mut poor,
        &vendor_buy_packet(108_006, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(merchant_status(replies.last().expect("poor reply")), 0);
    assert_eq!(
        poor.snapshot(),
        poor_before,
        "insufficient-coins refusal changed the gateway state"
    );

    let mut full = fixture.connect(fixture.state(price + 1_000, true));
    fixture.open_shop(&mut full);
    assert_eq!(
        inventory_of(&full)
            .items
            .iter()
            .filter(|item| item.count > 0)
            .count(),
        100
    );
    let full_before = full.snapshot();
    let replies = send(
        &mut full,
        &vendor_buy_packet(108_007, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(merchant_status(replies.last().expect("full-bag reply")), 0);
    assert_eq!(
        full.snapshot(),
        full_before,
        "full-bag refusal changed the gateway state"
    );
}
