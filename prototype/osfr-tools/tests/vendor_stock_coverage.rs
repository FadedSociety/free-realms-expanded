//! Vendor stock coverage for the playtest hubs.
//!
//! The pre-T108 runtime baseline was reproduced from `vendors.json`, `stores.json`,
//! and `Inventory::load`: 96 populated vendor records, 96 of them fully grantable,
//! and 1,081 grantable stock rows. The stale T019 note said 62 of 192; that number
//! predates the later ride-vendor additions and does not match the checked catalogue.
//!
//! The six selected-hub NPC files contain 59 vendor-role records: 30 fully
//! grantable `vendors.json` records (T275 made Bordon Cloo a vendor), one resolved
//! record whose only named stock has no purchasable price, and 28 records with no
//! item-level stock evidence.
//! Every one is enumerated and either stocked or listed as an explicit,
//! evidence-backed exception, so a new vendor or a newly empty window fails here
//! instead of passing silently.
//!
//! This enumeration is **not** the T108 completion target. The brief asks for
//! 20 additional fully grantable vendor records (or every reachable selected-hub
//! vendor when fewer than 20 remain unresolved). Twenty-eight vendor rows are
//! still unresolved, so the fallback cannot apply and the target is unmet; the
//! ignored `at_least_twenty_more_selected_hub_vendors_are_fully_grantable` test
//! keeps that target runnable. Neither an empty exception nor the one empty
//! `vendors.json` record counts as a completed vendor. See
//! `notes/Autopilot/F192.md` for the evidence search and the lead's accepted
//! decision (2026-10-19): no stock is invented beyond retained evidence.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/vendor_purchase.rs"]
mod vendor_purchase;

use gateway_vendor::{select, vendor_buy_packet};
use osfr_tools::{
    inventory::Inventory,
    npc_interaction::{MerchantWindow, NpcAction, NpcInteraction, NpcSelection},
    world::NpcVisibility,
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

/// T230 moved every ride row to the Station Cash Marketplace: 54 Seaside ride rows and 3
/// Mysterious Merchant ride rows stopped being grantable coin stock. This tip measures 1,049
/// after the move (1,103 before), so the fixed baseline stays 23 rows below the measured floor.
const BASELINE_GRANTABLE_STOCK_ROWS: usize = 1_026;
/// The brief's row floor. T108 added 22 rows; this only guards the row-level
/// content, not the completion target (see the selected-hub test below).
const REQUIRED_ADDITIONAL_ROWS: usize = 20;
/// Reproduced from the pre-T108 vendor records: 96 of the 98 records were
/// fully grantable. T230 deliberately emptied Delvin and the Mysterious
/// Merchant, both of which were fully grantable, so the reproduced floor drops
/// to 94; a later content change may raise it but must not lower it.
const REPRODUCED_FULLY_GRANTABLE_VENDORS: usize = 94;
/// Every vendor-role record in the six selected-hub NPC files, split by the
/// evidence available for its stock. The test asserts each count, so adding,
/// removing or emptying a vendor needs a deliberate content change.
const SELECTED_HUB_VENDOR_COUNT: usize = 59;
const SELECTED_HUB_VENDOR_RECORD_COUNT: usize = 31;
const SELECTED_HUB_STOCKED_VENDOR_COUNT: usize = 30;
const SELECTED_HUB_RESOLVED_EMPTY_VENDOR_COUNT: usize = 1;
const SELECTED_HUB_UNRESOLVED_VENDOR_COUNT: usize = 28;
/// Only data-backed `legacy_data` rows are sent by `NpcVisibility::load`; the
/// other 29 vendor rows are POI-anchor/unanchored reconstruction scaffolding.
/// Pin the stocked/unresolved split too, so a placement swap between the two
/// groups cannot keep the total while hiding a reachable-stock regression.
const SELECTED_HUB_SPAWNED_VENDOR_COUNT: usize = 30;
const SELECTED_HUB_SPAWNED_STOCKED_VENDOR_COUNT: usize = 20;
const SELECTED_HUB_SPAWNED_UNSTOCKED_VENDOR_COUNT: usize = 10;
/// Stocked `vendors.json` records whose NPC placement is not `legacy_data`, so
/// the production visibility loader never spawns them. They remain unreachable
/// and must not be counted toward the T108 completion target.
const SELECTED_HUB_STOCKED_UNSPAWNED_VENDOR_COUNT: usize = 10;
const SELECTED_HUB_STOCKED_UNSPAWNED_VENDOR_IDS: &[u64] = &[
    9_500_000_032,
    9_500_000_067,
    9_500_000_339,
    9_500_000_344,
    9_500_000_348,
    9_500_000_349,
    9_500_000_354,
    9_500_000_355,
    9_500_000_482,
    9_500_000_543,
];
/// The brief's 20-additional-vendor target leaves 49 of the 59 selected-hub
/// vendor rows fully grantable. Nothing sources that yet; the ignored target
/// test below records the real requirement instead of treating the pre-T275
/// 29/1/28 split as completion. Reachability is checked separately: 20 more
/// *spawned* fully grantable vendors than the pre-T275 19, so unspawned stock
/// cannot make the target look complete.
const T108_REQUIRED_STOCKED_VENDOR_COUNT: usize = 49;
const T108_REQUIRED_SPAWNED_STOCKED_VENDOR_COUNT: usize = 39;
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

fn fully_grantable(vendor: &Value, inventory: &Inventory, prices: &BTreeMap<i64, u64>) -> bool {
    let items = vendor["items"]
        .as_array()
        .expect("vendor items are an array");
    !items.is_empty() && items.iter().all(|item| grantable(item, inventory, prices))
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
/// `00a36f80`); the reply is the 66/42 list. Dispatcher `00b48a10` selects
/// `00b40a60`; `00b3d5f0` reads the header and nested record, `00b33020` reads
/// one scalar, two dword pairs and a final scalar (six dwords), and `00b2e620`
/// then reads the entry count and rows through `00b29cd0`.
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

/// Legacy protected name. It enumerates the 59 selected-hub vendor-role rows
/// and pins the known 30 stocked / 1 resolved-empty / 28 unresolved split; it
/// is a content-completeness audit, **not** evidence that the T108 20-vendor
/// target is met. The ignored target test below states that requirement.
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
    // These are the known content-gap counts, not the brief's 49-vendor
    // completion target: T275 stocked Bordon Cloo, so 30 records are stocked; 28
    // rows still have no item-level final-era evidence
    // and one resolved record exposes only an unbuyable Bait row. The ignored
    // target test above keeps the real requirement visible.
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

/// The production loader (`world::NpcVisibility::load`, `src/world.rs:280`)
/// spawns only data-backed `legacy_data` rows; `reconstructed_spawn`, POI-anchor
/// and unanchored rows are reconstruction scaffolding and stay off the wire.
/// Pin both outcomes so a placement or anchor regression cannot silently drop a
/// visible vendor, and confirm the 35/2 spawn list carries the content position.
/// Also pin the stocked rows the loader cannot spawn: their stock is real but
/// unreachable, so it must not count toward the T108 completion target.
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
    let mut stocked_unspawned = BTreeSet::new();
    for vendor in vendors {
        let spawned_placement = vendor.placement == "legacy_data";
        if spawned_placement {
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
            if stocked_ids.contains(&vendor.npc_id) {
                stocked_unspawned.insert(vendor.npc_id);
            }
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
    assert_eq!(
        stocked_unspawned.len(),
        SELECTED_HUB_STOCKED_UNSPAWNED_VENDOR_COUNT,
        "the stocked-but-unspawned vendor set changed; these rows are unreachable"
    );
    assert_eq!(
        stocked_unspawned.iter().copied().collect::<Vec<_>>(),
        SELECTED_HUB_STOCKED_UNSPAWNED_VENDOR_IDS,
        "a stocked vendor's placement changed; do not count an unreachable vendor as complete"
    );
}

/// The brief's completion target, kept runnable but ignored: 20 additional
/// fully grantable vendor records would leave 49 of 59 stocked, and 20 more
/// spawned fully grantable vendors would leave 39 reachable. No retained source
/// proves stock for 20 of the 28 unresolved rows (the wiki pages name no
/// final-era items; the few item-level mentions are retired-2009 or
/// variant/composite claims), so the lead accepted this unmet target on
/// 2026-10-19 instead of invented content. See `notes/Autopilot/F192.md`.
#[test]
#[ignore = "T108 target unmet: 20 of 39 spawned (30 of 49 records) selected-hub vendors are stocked"]
fn at_least_twenty_more_selected_hub_vendors_are_fully_grantable() {
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let stores = prices(&content);
    let inventory = Inventory::load(&inventory_content_dir(), None)
        .expect("the runtime inventory catalogue loads");
    let placements: BTreeMap<u64, String> = selected_hub_vendors(&content)
        .into_iter()
        .map(|vendor| (vendor.npc_id, vendor.placement))
        .collect();
    let selected: Vec<(u64, &Value)> = vendor_items(&vendors)
        .into_iter()
        .filter(|(_, vendor)| {
            vendor["npc_content_file"]
                .as_str()
                .is_some_and(|file| SELECTED_HUB_FILES.contains(&file))
        })
        .filter(|(_, vendor)| fully_grantable(vendor, &inventory, &stores))
        .collect();
    let fully_grantable_count = selected.len();
    let spawned_count = selected
        .iter()
        .filter(|(npc_id, _)| {
            placements
                .get(npc_id)
                .is_some_and(|placement| placement == "legacy_data")
        })
        .count();
    assert!(
        spawned_count >= T108_REQUIRED_SPAWNED_STOCKED_VENDOR_COUNT,
        "expected {T108_REQUIRED_SPAWNED_STOCKED_VENDOR_COUNT} spawned fully grantable \
         selected-hub vendors, got {spawned_count}; unspawned stock is not reachable completion"
    );
    assert!(
        fully_grantable_count >= T108_REQUIRED_STOCKED_VENDOR_COUNT,
        "expected {T108_REQUIRED_STOCKED_VENDOR_COUNT} fully grantable selected-hub vendors, \
         got {fully_grantable_count}"
    );
}

#[test]
fn at_least_twenty_more_vendor_rows_are_fully_grantable() {
    // Row-level content regression only. Neither this row count nor the
    // 96-vendor baseline proves a newly completed vendor; the T108
    // 20-vendor target is the ignored test above and remains unmet. T108 added
    // 22 rows to existing records, which is exactly why rows alone cannot be
    // the completion gate.
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
        let all_grantable = fully_grantable(vendor, &inventory, &stores);
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

/// Protected name. The real-content purchase/refusal proof lives in
/// `common/vendor_purchase.rs` so this file stays the content-coverage suite.
#[test]
fn representative_new_stock_buys_once_and_refuses_atomically() {
    vendor_purchase::representative_new_stock_buys_once_and_refuses_atomically();
}
