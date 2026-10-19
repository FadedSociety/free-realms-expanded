//! Vendor stock coverage for the playtest hubs.
//!
//! The pre-T108 runtime baseline was reproduced from `vendors.json`, `stores.json`,
//! and `Inventory::load`: 96 populated vendor records, 96 of them fully grantable,
//! and 1,081 grantable stock rows. The stale T019 note said 62 of 192; that number
//! predates the later ride-vendor additions and does not match the checked catalogue.
//!
//! The six selected-hub NPC files contain 58 vendor-role records: 29 fully
//! grantable `vendors.json` records, one resolved record whose only named stock
//! has no purchasable price, and 28 records with no item-level stock evidence.
//! Every one is enumerated and either stocked or listed as an explicit,
//! evidence-backed exception, so a new vendor or a newly empty window fails here
//! instead of passing silently.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;

use gateway_vendor::{
    NOW, count_definition, inner_packets, interact, inventory_of, login, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::{Inventory, InventoryItem},
    npc_interaction::{
        MerchantWindow, NpcAction, NpcInteraction, NpcSelection, decode_merchant_list_66_42,
    },
    world::NpcVisibility,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
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
/// The brief's row floor. T108 added 22 rows; this only guards the row-level
/// content, not the completion target (see the selected-hub test below).
const REQUIRED_ADDITIONAL_ROWS: usize = 20;
/// Reproduced from the pre-T108 vendor records: 96 of the 98 records were
/// fully grantable. A later content change may raise this floor but must not
/// lower it.
const REPRODUCED_FULLY_GRANTABLE_VENDORS: usize = 96;
/// Every vendor-role record in the six selected-hub NPC files, split by the
/// evidence available for its stock. The test asserts each count, so adding,
/// removing or emptying a vendor needs a deliberate content change.
const SELECTED_HUB_VENDOR_COUNT: usize = 58;
const SELECTED_HUB_VENDOR_RECORD_COUNT: usize = 30;
const SELECTED_HUB_STOCKED_VENDOR_COUNT: usize = 29;
const SELECTED_HUB_RESOLVED_EMPTY_VENDOR_COUNT: usize = 1;
const SELECTED_HUB_UNRESOLVED_VENDOR_COUNT: usize = 28;
/// Only data-backed `legacy_data` rows are sent by `NpcVisibility::load`; the
/// other 29 vendor rows are POI-anchor/unanchored reconstruction scaffolding.
/// Pin the stocked/unresolved split too, so a placement swap between the two
/// groups cannot keep the total while hiding a reachable-stock regression.
const SELECTED_HUB_SPAWNED_VENDOR_COUNT: usize = 29;
const SELECTED_HUB_SPAWNED_STOCKED_VENDOR_COUNT: usize = 19;
const SELECTED_HUB_SPAWNED_UNSTOCKED_VENDOR_COUNT: usize = 10;
const REPRESENTATIVE_VENDOR: u64 = 9_500_000_045;
const REPRESENTATIVE_ITEM: i32 = 76_427;

/// A vendor that cannot expose grantable stock yet. Every entry is a positioned
/// vendor-role NPC in a selected hub. The test resolves `source` against the
/// content: a `vendors.json` record with an empty `item_ids`, or a
/// `vendors_unresolved.json` record whose `resolved_item_ids` is empty and whose
/// own `source`/`reason` carry the per-NPC evidence.
struct StockException {
    npc_id: u64,
    source: &'static str,
    reason: &'static str,
}

const UNRESOLVED_SOURCE: &str = "data/content/vendors_unresolved.json record";

const fn unresolved(npc_id: u64) -> StockException {
    StockException {
        npc_id,
        source: UNRESOLVED_SOURCE,
        reason: "No item-level final-era stock evidence; no resolved_item_ids.",
    }
}

const fn unresolved_named(npc_id: u64) -> StockException {
    StockException {
        npc_id,
        source: UNRESOLVED_SOURCE,
        reason: "Named stock is unmatched or variant-ambiguous; no resolved_item_ids.",
    }
}

/// Every empty selected-hub vendor, with the evidence the counts rely on.
const SELECTED_HUB_STOCK_EXCEPTIONS: &[StockException] = &[
    unresolved(9_000_000_021),
    StockException {
        npc_id: 9_000_000_027,
        source: "data/content/vendors.json record",
        reason: "(R) Bait (57012) has a null stores.json purchase price and a NoSale client row.",
    },
    unresolved(9_500_000_005),
    unresolved(9_500_000_014),
    unresolved(9_500_000_051),
    unresolved(9_500_000_069),
    unresolved(9_500_000_072),
    unresolved(9_500_000_074),
    unresolved(9_500_000_247),
    unresolved(9_500_000_279),
    unresolved(9_500_000_328),
    unresolved(9_500_000_331),
    unresolved(9_500_000_332),
    unresolved(9_500_000_350),
    unresolved(9_500_000_358),
    unresolved(9_500_000_442),
    unresolved(9_500_000_445),
    unresolved_named(9_500_000_454),
    unresolved(9_500_000_455),
    unresolved(9_500_000_457),
    unresolved(9_500_000_463),
    unresolved(9_500_000_466),
    unresolved_named(9_500_000_473),
    unresolved(9_500_000_492),
    unresolved(9_500_000_515),
    unresolved(9_500_000_518),
    unresolved(9_500_000_529),
    unresolved(9_500_000_540),
    unresolved(9_500_000_545),
];

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

struct VendorNpc {
    file: String,
    npc_id: u64,
    name: String,
    placement: String,
    position: [f32; 3],
}

/// Enumerate the vendor-role records in the six selected-hub NPC files. The
/// role test mirrors `npc_interaction::has_role` (the exact `vendor` value).
fn selected_hub_vendors(content: &Path) -> Vec<VendorNpc> {
    let mut vendors = Vec::new();
    for file in SELECTED_HUB_FILES {
        let document = read_json(&content.join("npcs").join(file));
        for record in document["records"]
            .as_array()
            .expect("NPC records are an array")
        {
            let vendor_role = record["roles"]
                .as_array()
                .is_some_and(|roles| roles.iter().any(|role| role.as_str() == Some("vendor")));
            if !vendor_role {
                continue;
            }
            vendors.push(VendorNpc {
                file: (*file).to_string(),
                npc_id: record["npc_id"].as_u64().expect("NPC ID is an integer"),
                name: record["name"]
                    .as_str()
                    .expect("NPC name is a string")
                    .to_string(),
                placement: record["placement"]
                    .as_str()
                    .expect("NPC placement is a string")
                    .to_string(),
                position: {
                    let position = record["position"]
                        .as_array()
                        .expect("vendor NPC has a position");
                    assert_eq!(position.len(), 3, "vendor NPC position is XYZ");
                    [
                        position[0].as_f64().expect("X is a number") as f32,
                        position[1].as_f64().expect("Y is a number") as f32,
                        position[2].as_f64().expect("Z is a number") as f32,
                    ]
                },
            });
        }
    }
    vendors.sort_by_key(|vendor| vendor.npc_id);
    vendors
}

fn exception(npc_id: u64) -> Option<&'static StockException> {
    SELECTED_HUB_STOCK_EXCEPTIONS
        .iter()
        .find(|exception| exception.npc_id == npc_id)
}

/// Open one vendor's 66/42 window through the production interaction service.
/// The 26/10 body is the u64 NPC GUID plus the i32 choice (client constructor
/// `00a36f80`); the reply is the 66/42 list (client readers
/// `00b48a10`/`00b40a60`/`00b3d5f0`/`00b33020`).
fn open_shop(interaction: &mut NpcInteraction, npc_id: u64) -> MerchantWindow {
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
    let choice = u32::try_from(option.id).expect("the shop choice id fits the wire field");
    let NpcSelection::Shop(window) = interaction
        .select(&select(npc_id, choice), zone, position, None)
        .unwrap_or_else(|error| panic!("NPC {npc_id} shop did not open: {error:?}"))
    else {
        panic!("NPC {npc_id} selected a non-shop action");
    };
    window
}

#[test]
fn the_selected_hub_vendors_have_nonempty_grantable_stock() {
    let content = content_dir();
    let vendor_document = read_json(&content.join("vendors.json"));
    let selected_records: BTreeMap<u64, &Value> = vendor_items(&vendor_document)
        .into_iter()
        .filter(|(_, vendor)| {
            vendor["npc_content_file"]
                .as_str()
                .is_some_and(|file| SELECTED_HUB_FILES.contains(&file))
        })
        .collect();
    let unresolved = read_json(&content.join("vendors_unresolved.json"));
    let unresolved: BTreeMap<u64, &Value> = unresolved["unresolved"]
        .as_array()
        .expect("unresolved vendor records are an array")
        .iter()
        .map(|record| {
            (
                record["npc_id"]
                    .as_u64()
                    .expect("unresolved NPC ID is an integer"),
                record,
            )
        })
        .collect();
    let stores = prices(&content);
    let catalogue = Inventory::load(&inventory_content_dir(), None)
        .expect("the runtime inventory catalogue loads");
    let mut interaction = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts the selected hubs");

    let vendors = selected_hub_vendors(&content);
    assert_eq!(
        vendors.len(),
        SELECTED_HUB_VENDOR_COUNT,
        "the selected-hub vendor set changed; update the enumeration deliberately"
    );
    assert_eq!(
        selected_records.len(),
        SELECTED_HUB_VENDOR_RECORD_COUNT,
        "the selected-hub vendor-record set changed"
    );
    let vendor_ids: BTreeSet<u64> = vendors.iter().map(|vendor| vendor.npc_id).collect();
    for npc_id in selected_records.keys() {
        assert!(
            vendor_ids.contains(npc_id),
            "vendors.json records NPC {npc_id}, but its NPC file no longer has the vendor role"
        );
    }

    let mut stocked = 0usize;
    let mut stocked_ids = BTreeSet::new();
    let mut resolved_empty = 0usize;
    let mut unresolved_empty = 0usize;
    let mut empty = BTreeSet::new();
    for vendor in &vendors {
        let npc_id = vendor.npc_id;
        if let Some(record) = selected_records.get(&npc_id) {
            let ids: Vec<i64> = record["item_ids"]
                .as_array()
                .expect("runtime item_ids are an array")
                .iter()
                .map(|id| id.as_i64().expect("runtime item ID is an integer"))
                .collect();
            if !ids.is_empty() {
                stocked += 1;
                stocked_ids.insert(npc_id);
                let items = record["items"]
                    .as_array()
                    .expect("vendor items are an array");
                assert_eq!(
                    ids.len(),
                    items.len(),
                    "NPC {npc_id} has partial stock rows"
                );
                for item in items {
                    assert!(
                        grantable(item, &catalogue, &stores),
                        "NPC {npc_id} exposes non-grantable item {}",
                        item_id(item)
                    );
                }
                let window = open_shop(&mut interaction, npc_id);
                let window_ids: BTreeSet<i32> =
                    window.items.iter().map(|item| item.item_id).collect();
                let ids: BTreeSet<i32> = ids
                    .iter()
                    .map(|id| i32::try_from(*id).expect("vendor item ID fits i32"))
                    .collect();
                assert_eq!(window_ids, ids, "NPC {npc_id} shop is incomplete");
                for item in &window.items {
                    assert!(
                        item.purchase_price_coins.is_some_and(|price| price > 0),
                        "NPC {npc_id} item {} has no positive purchase price",
                        item.item_id
                    );
                    assert!(
                        catalogue.has_definition(item.item_id),
                        "NPC {npc_id} item {} has no runtime definition",
                        item.item_id
                    );
                }
            } else {
                resolved_empty += 1;
                assert_exception(vendor, exception(npc_id));
                let items = record["items"]
                    .as_array()
                    .expect("vendor items are an array");
                assert!(!items.is_empty(), "NPC {npc_id} has no named stock rows");
                assert!(
                    items
                        .iter()
                        .all(|item| !grantable(item, &catalogue, &stores)),
                    "NPC {npc_id} has a grantable row but no runtime item_ids"
                );
                assert!(
                    items.iter().any(|item| {
                        item["purchase_price_coins"].is_null() && item["runtime_eligible"] == false
                    }),
                    "NPC {npc_id} exception lacks the null-price NoSale row"
                );
                assert_eq!(record["npc_name"], "Bluerose");
                assert!(
                    record["vendor_stock_description"]
                        .as_str()
                        .is_some_and(|description| description.starts_with("Bait.")),
                    "NPC {npc_id} exception source no longer names the Bait stock"
                );
                assert_eq!(record["stock_completeness"], "partial_evidence_reference");
                empty.insert(npc_id);
                assert!(
                    open_shop(&mut interaction, npc_id).items.is_empty(),
                    "NPC {npc_id} resolution exception still exposes stock"
                );
            }
            continue;
        }

        unresolved_empty += 1;
        let recorded = exception(npc_id);
        assert_exception(vendor, recorded);
        let record = unresolved.get(&npc_id).unwrap_or_else(|| {
            panic!(
                "NPC {npc_id} ({}) has no vendors.json record and no vendors_unresolved.json record",
                vendor.name
            )
        });
        assert_eq!(
            record["source"].as_str(),
            Some(vendor.file.as_str()),
            "NPC {npc_id} unresolved evidence points at another content file"
        );
        assert!(
            record["resolved_item_ids"]
                .as_array()
                .is_some_and(|ids| ids.is_empty()),
            "NPC {npc_id} has resolved item IDs but no runtime stock"
        );
        assert!(
            record["reason"]
                .as_str()
                .is_some_and(|reason| !reason.trim().is_empty()),
            "NPC {npc_id} unresolved record has no reason"
        );
        empty.insert(npc_id);
        assert!(
            open_shop(&mut interaction, npc_id).items.is_empty(),
            "NPC {npc_id} unresolved exception still exposes stock"
        );
    }

    let expected_exceptions: BTreeSet<u64> = SELECTED_HUB_STOCK_EXCEPTIONS
        .iter()
        .map(|exception| exception.npc_id)
        .collect();
    assert_eq!(
        empty, expected_exceptions,
        "the selected-hub empty vendors changed; every exception needs a source and reason"
    );
    assert_eq!(stocked, SELECTED_HUB_STOCKED_VENDOR_COUNT);
    assert_eq!(resolved_empty, SELECTED_HUB_RESOLVED_EMPTY_VENDOR_COUNT);
    assert_eq!(unresolved_empty, SELECTED_HUB_UNRESOLVED_VENDOR_COUNT);

    assert_selected_hub_vendor_spawns(&content, &vendors, &stocked_ids);
}

/// Every exception must name its evidence and pass the non-empty sanity check.
/// A stale exception (a vendor that gained stock) fails the caller's set check;
/// the unresolved record itself is checked against the NPC file in the test.
fn assert_exception(vendor: &VendorNpc, recorded: Option<&StockException>) {
    let exception = recorded.unwrap_or_else(|| {
        panic!(
            "NPC {} ({}) exposes no stock and has no recorded exception",
            vendor.npc_id, vendor.name
        )
    });
    assert!(
        !exception.source.is_empty() && !exception.reason.is_empty(),
        "NPC {} exception needs a source and reason",
        vendor.npc_id
    );
    assert!(
        exception.source.starts_with("data/content/"),
        "NPC {} exception source must name a content file",
        vendor.npc_id
    );
}

/// The production loader (`world::NpcVisibility::load`) sends only data-backed
/// `legacy_data`/`reconstructed_spawn` rows; POI-anchor and unanchored rows are
/// reconstruction scaffolding and stay off the wire. Pin both outcomes so a
/// placement or anchor regression cannot silently drop a visible vendor, and
/// confirm the 35/2 spawn list carries the content position.
fn assert_selected_hub_vendor_spawns(
    content: &Path,
    vendors: &[VendorNpc],
    stocked_ids: &BTreeSet<u64>,
) {
    let visibility = NpcVisibility::load(content)
        .expect("the production NPC visibility loader accepts the selected hubs");
    let spawned: BTreeMap<u64, [f32; 3]> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    let mut data_backed = 0usize;
    let mut data_backed_stocked = 0usize;
    let mut data_backed_unstocked = 0usize;
    for vendor in vendors {
        let data_backed_placement = matches!(
            vendor.placement.as_str(),
            "legacy_data" | "reconstructed_spawn"
        );
        if data_backed_placement {
            data_backed += 1;
            if stocked_ids.contains(&vendor.npc_id) {
                data_backed_stocked += 1;
            } else {
                data_backed_unstocked += 1;
            }
            assert_eq!(
                visibility.content_key(vendor.npc_id),
                Some(vendor.npc_id),
                "vendor NPC {} ({}) is data-backed but not spawned",
                vendor.npc_id,
                vendor.name
            );
            let position = spawned.get(&vendor.npc_id).unwrap_or_else(|| {
                panic!(
                    "vendor NPC {} ({}) is missing from the 35/2 spawn list",
                    vendor.npc_id, vendor.name
                )
            });
            for axis in 0..3 {
                assert!(
                    (position[axis] - vendor.position[axis]).abs() < 1e-3,
                    "vendor NPC {} spawn position differs from its content row",
                    vendor.npc_id
                );
            }
        } else {
            assert_eq!(
                visibility.content_key(vendor.npc_id),
                None,
                "vendor NPC {} ({}) has placement {} and must not be spawned",
                vendor.npc_id,
                vendor.name,
                vendor.placement
            );
        }
    }
    assert_eq!(
        data_backed, SELECTED_HUB_SPAWNED_VENDOR_COUNT,
        "the selected-hub data-backed vendor spawn set changed"
    );
    assert_eq!(
        data_backed_stocked, SELECTED_HUB_SPAWNED_STOCKED_VENDOR_COUNT,
        "the data-backed stocked-vendor spawn set changed"
    );
    assert_eq!(
        data_backed_unstocked, SELECTED_HUB_SPAWNED_UNSTOCKED_VENDOR_COUNT,
        "the data-backed non-stocked vendor spawn set changed"
    );
}

#[test]
fn at_least_twenty_more_vendor_rows_are_fully_grantable() {
    // Row-level content regression only. The completion target is enforced by
    // `the_selected_hub_vendors_have_nonempty_grantable_stock`, which accounts
    // for every reachable vendor in the selected hubs. T108 added 22 rows to
    // existing records because the file already had 96 of 98 fully grantable
    // records; rows alone cannot prove a new vendor was completed.
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

/// 66/43 serializer `00b3ab20` writes two dwords, three dwords, a
/// length-prefixed string and three dwords. 66/44 reader `00b32430` reads seven
/// dwords, the string and five dwords, and compares object field `+0x58`
/// (wire offset 48) with 1.
#[test]
fn vendor_buy_wire_matches_the_client_reader_layout() {
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

    let response = osfr_tools::coin_shop::merchant_buy_result_packet(true);
    assert_eq!(response.len(), 56);
    assert_eq!(&response[..4], &[66, 0, 44, 0]);
    assert_eq!(u32::from_le_bytes(response[48..52].try_into().unwrap()), 1);
}

static NEXT: AtomicU64 = AtomicU64::new(0);

/// A complete gameplay fixture built from the real vendor record and the
/// captured catalogue. Only the NPC's zone/position is translated to zone 0
/// because `Gateway::load_gameplay` without the opening campaign supports the
/// open-world zone. The stock row, prices, item definition and transaction path
/// are the shipped ones. The directory is removed on drop.
struct GatewayFixture {
    content: PathBuf,
    reference: PathBuf,
    zone: u32,
    position: [f32; 3],
    option: u32,
    price: u32,
}

impl Drop for GatewayFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl GatewayFixture {
    fn new() -> Self {
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let reference = lab.join("references/rust-gateway-reference");
        let source = lab.join("reports/round70-native-20261016T061000Z/content");
        let content = lab.join("build/w178-vendor-gateway").join(format!(
            "f178-vendor-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
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
            fs::copy(source.join(file), content.join(file))
                .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
        }
        let catalogue = content.join("item-catalogue.json");
        if fs::hard_link(source.join("item-catalogue.json"), &catalogue).is_err() {
            fs::copy(source.join("item-catalogue.json"), &catalogue)
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

    fn character(&self) -> CharacterState {
        let mut character =
            Gateway::reference_state(&self.reference).expect("reference character loads");
        character.zone_id = self.zone;
        character.position = self.position;
        character.rotation = [0.0; 3];
        character
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        self.connect_with(self.character(), state)
    }

    /// Reconnect through the saved character half of the save envelope, not a
    /// fresh reference character.
    fn connect_with(&self, character: CharacterState, state: GameplayState) -> Gateway {
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
        send(gateway, &interact(REPRESENTATIVE_VENDOR));
        let replies = send(gateway, &select(REPRESENTATIVE_VENDOR, self.option));
        assert!(
            replies
                .iter()
                .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
            "the gateway did not open the representative shop"
        );
        replies
    }
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
    // 66/42 is decoded exactly as the client reads it: dispatcher `00b48a10`
    // -> `00b40a60` -> `00b3d5f0` (header + nested record), group reader
    // `00b33020` (seven dwords + counted rows), row reader `00b29cd0`.
    let list = decode_merchant_list_66_42(list).expect("shop list matches the client reader");
    let entry = list
        .group
        .entries
        .iter()
        .find(|entry| entry.first_six_dwords[0] == REPRESENTATIVE_ITEM as u32)
        .expect("the gateway shop does not expose the new item");
    // (R) Local convention for the opaque row: dword 1 is the purchase price.
    assert_eq!(entry.first_six_dwords[1], price);

    let before = gateway.snapshot();
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

    // Reconnect through both saved halves, not a fresh reference character.
    let saved_character: CharacterState = serde_json::from_slice(
        &serde_json::to_vec(&after.character).expect("character state serializes"),
    )
    .expect("character state parses");
    let saved: GameplayState = serde_json::from_slice(
        &serde_json::to_vec(
            &after
                .gameplay
                .clone()
                .expect("successful gameplay state serializes"),
        )
        .expect("gameplay state serializes"),
    )
    .expect("gameplay state parses");
    let reconnected = fixture.connect_with(saved_character.clone(), saved);
    let reconnect_snapshot = reconnected.snapshot();
    assert_eq!(
        reconnect_snapshot.character, saved_character,
        "the reconnect ignored the saved character half"
    );
    assert_eq!(inventory_of(&reconnected), after_inventory);

    // Poor coins: the refusal changes nothing, and the shop survives it. Sell
    // the seeded item to top up, then the same buy lands.
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
    send(&mut poor, &sell_packet(0, 1, 1));
    let topped_up = inventory_of(&poor).coins;
    assert_eq!(
        topped_up,
        price - 1 + 375,
        "the poor-coins refusal consumed the open shop"
    );
    let replies = send(
        &mut poor,
        &vendor_buy_packet(108_020, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("poor retry reply")),
        1
    );
    assert_eq!(inventory_of(&poor).coins, topped_up - price);
    assert_eq!(
        count_definition(&inventory_of(&poor), REPRESENTATIVE_ITEM),
        1
    );

    // Full bag: the refusal changes nothing, and freeing one slot through the
    // real sell route lets the same buy land.
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
    send(&mut full, &sell_packet(0, 100_000, 1));
    let freed = inventory_of(&full);
    assert_eq!(
        freed.coins,
        price + 1_000 + 375,
        "the full-bag refusal consumed the open shop"
    );
    assert_eq!(freed.items.len(), 99);
    let replies = send(
        &mut full,
        &vendor_buy_packet(108_021, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(
        merchant_status(replies.last().expect("full-bag retry reply")),
        1
    );
    let purchased = inventory_of(&full);
    assert_eq!(purchased.coins, freed.coins - price);
    assert_eq!(purchased.items.len(), 100);
    assert_eq!(count_definition(&purchased, REPRESENTATIVE_ITEM), 1);
}
