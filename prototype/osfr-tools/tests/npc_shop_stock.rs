//! T774: an NPC shop only opens when it has stock to list.
//!
//! A vendor-role NPC with no priced `vendors.json` stock used to offer the Shop
//! choice, and the client opened an empty merchant window (native 2026-10-26, Mr.
//! Twinkle). `npc_interaction::build_options` now offers Shop only when the
//! merchant window has a priced row, so the 165/10 list the client receives is
//! never empty. This suite is the content gate:
//!
//! - every vendor-role NPC in `npcs/*.json` either offers Shop and advertises its
//!   whole mapped stock through the real in-world dispatch, or offers no Shop at
//!   all; both sets are pinned by id, so a content edit cannot silently empty a
//!   shop or add one without stock;
//! - Mr. Twinkle (9_500_000_758) offers no Shop because no item-level stock is
//!   recovered for him (see `notes/Autopilot/T774.md`), a choice the menu does not
//!   offer is refused without consuming the menu, and his Talk choice still opens
//!   the dialogue through the real gateway.
//!
//! Client evidence: the 26/9 menu reader chain `00aa2360 -> 00aa2220 -> 00aa1600`
//! with the option reader `00a9dca0`; the 26/8/26/10 constructors `008b6bc0` /
//! `00a36f80`; the 165/10 coin merchant list `00c78fb0`, whose record/row readers
//! `00b33020`/`00b29cd0` `decode_coin_merchant_list_165_10` mirrors.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/npc_greetings.rs"]
mod npc_greetings;
#[path = "common/round70_content.rs"]
mod round70_content;

use gateway_vendor::{NOW, connect_with, send};
use npc_greetings::{content_dir, interact, npc_position, npc_record, read_json, select, service};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    gateway::Gateway,
    inventory::Inventory,
    npc_interaction::{NpcAction, decode_coin_merchant_list_165_10},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// The vendor the owner found with an empty window (native 2026-10-26).
const MR_TWINKLE: u64 = 9_500_000_758;
/// The NPC file that carries Mr. Twinkle's row.
const MR_TWINKLE_FILE: &str = "snowhill.json";
/// Vendor-role NPCs with at least one priced `vendors.json` row.
const SHOP_VENDORS: usize = 96;
/// Vendor-role NPCs with no priced stock: no Shop choice, so no empty window.
const NO_SHOP_VENDORS: usize = 98;
/// The vendor-role NPCs that offer no Shop, pinned so a stock change or a new
/// vendor without stock has to be a deliberate content edit. 94 have no resolved
/// stock record (no item-level final-era evidence, a retired-only item, or an
/// ambiguous variant); 2 name only an unpriced `Bait` row (Big J, Bluerose); the
/// other 2 are the T230 ride vendors whose rides moved to the Marketplace.
const NO_SHOP: &[u64] = &[
    9_000_000_016,
    9_000_000_021,
    9_000_000_024,
    9_000_000_027,
    9_000_000_048,
    9_000_000_079,
    9_000_000_092,
    9_000_000_101,
    9_000_000_106,
    9_000_000_122,
    9_000_000_196,
    9_000_000_334,
    9_000_000_387,
    9_000_000_487,
    9_500_000_005,
    9_500_000_014,
    9_500_000_051,
    9_500_000_069,
    9_500_000_072,
    9_500_000_074,
    9_500_000_113,
    9_500_000_231,
    9_500_000_235,
    9_500_000_247,
    9_500_000_279,
    9_500_000_283,
    9_500_000_284,
    9_500_000_285,
    9_500_000_286,
    9_500_000_288,
    9_500_000_289,
    9_500_000_290,
    9_500_000_291,
    9_500_000_292,
    9_500_000_309,
    9_500_000_313,
    9_500_000_328,
    9_500_000_331,
    9_500_000_332,
    9_500_000_350,
    9_500_000_358,
    9_500_000_376,
    9_500_000_380,
    9_500_000_385,
    9_500_000_388,
    9_500_000_391,
    9_500_000_419,
    9_500_000_442,
    9_500_000_445,
    9_500_000_454,
    9_500_000_455,
    9_500_000_457,
    9_500_000_463,
    9_500_000_466,
    9_500_000_473,
    9_500_000_492,
    9_500_000_515,
    9_500_000_518,
    9_500_000_529,
    9_500_000_540,
    9_500_000_545,
    9_500_000_549,
    9_500_000_555,
    9_500_000_563,
    9_500_000_566,
    9_500_000_569,
    9_500_000_572,
    9_500_000_584,
    9_500_000_589,
    9_500_000_593,
    9_500_000_596,
    9_500_000_601,
    9_500_000_608,
    9_500_000_609,
    9_500_000_616,
    9_500_000_625,
    9_500_000_653,
    9_500_000_708,
    9_500_000_728,
    9_500_000_736,
    9_500_000_739,
    9_500_000_748,
    9_500_000_749,
    9_500_000_750,
    9_500_000_751,
    9_500_000_758,
    9_500_000_774,
    9_500_000_805,
    9_500_000_811,
    9_500_000_852,
    9_500_000_878,
    9_500_000_880,
    9_500_000_908,
    9_500_000_924,
    9_500_000_928,
    9_500_000_936,
    9_500_000_987,
    9_500_001_000,
];

/// Every vendor-role NPC row, keyed by id, with its name for failure messages.
/// There is no shared enumerator: `vendor_stock_coverage` scopes itself to six
/// hubs, and the content gate needs every file.
fn vendor_npcs(content: &Path) -> BTreeMap<u64, String> {
    let mut npcs = BTreeMap::new();
    for entry in fs::read_dir(content.join("npcs")).expect("NPC content directory reads") {
        let path = entry.expect("NPC directory entry reads").path();
        if !path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            continue;
        }
        for record in read_json(&path)["records"]
            .as_array()
            .expect("NPC records are an array")
        {
            let vendor = record["roles"]
                .as_array()
                .is_some_and(|roles| roles.iter().any(|role| role.as_str() == Some("vendor")));
            if !vendor {
                continue;
            }
            npcs.insert(
                record["npc_id"].as_u64().expect("NPC id is an integer"),
                record["name"]
                    .as_str()
                    .expect("NPC name is a string")
                    .into(),
            );
        }
    }
    npcs
}

/// The mapped stock the window must list, with the server's store price checked:
/// a row without a positive purchase price is not stock.
fn priced_stock(vendors: &Value, stores: &Value, npc_id: u64) -> BTreeSet<i32> {
    let prices: BTreeMap<i64, &Value> = stores["items"]
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
    vendors["vendors"]
        .as_array()
        .expect("vendor records are an array")
        .iter()
        .find(|row| row["npc_id"].as_u64() == Some(npc_id))
        .into_iter()
        .flat_map(|row| {
            row["item_ids"]
                .as_array()
                .expect("runtime item_ids are an array")
        })
        .map(|value| {
            let item_id = i32::try_from(value.as_i64().expect("item ID is an integer"))
                .expect("item ID fits i32");
            let price = prices
                .get(&i64::from(item_id))
                .unwrap_or_else(|| panic!("NPC {npc_id} item {item_id} has no store row"));
            assert!(
                price["purchase_price_coins"]
                    .as_u64()
                    .is_some_and(|price| price > 0),
                "NPC {npc_id} item {item_id} has no positive purchase price"
            );
            item_id
        })
        .collect()
}

/// The gateway reference character at `position` in the open world. The vendor
/// fixture cannot host a stockless vendor (`VendorFixture::for_vendor` requires a
/// Shop option), so this suite places the reference character itself.
fn reference_character_at(position: [f32; 3]) -> CharacterState {
    let mut character = Gateway::reference_state(&round70_content::reference())
        .expect("the gateway reference character loads");
    character.zone_id = 0;
    character.position = position;
    character.rotation = [1.0, 0.0, 0.0];
    character
}

/// Every vendor-role NPC: the production menu offers Shop exactly when the
/// vendor has priced stock, and each offered shop advertises that whole stock
/// through the real in-world dispatch (26/8 -> 26/10 -> 165/10).
#[test]
fn every_vendor_shop_the_menu_offers_lists_its_priced_stock() {
    let content = content_dir();
    let vendors = read_json(&content.join("vendors.json"));
    let stores = read_json(&content.join("stores.json"));
    let npcs = vendor_npcs(&content);
    assert_eq!(npcs.len(), SHOP_VENDORS + NO_SHOP_VENDORS);
    let mut service = service();
    let inventory = Inventory::load(&round70_content::source(), None)
        .expect("the runtime item catalogue loads");
    let mut game = Gameplay::load(&round70_content::source(), None)
        .expect("the prepared gameplay content loads");

    let mut shops = BTreeSet::new();
    let mut no_shop = BTreeSet::new();
    for (npc_id, name) in &npcs {
        let (zone, position) = service
            .location(*npc_id)
            .unwrap_or_else(|| panic!("NPC {npc_id} ({name}) has a location"));
        let menu = service
            .interact(*npc_id, zone, position, None)
            .unwrap_or_else(|error| panic!("NPC {npc_id} ({name}) interaction: {error:?}"));
        let Some(shop) = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
        else {
            no_shop.insert(*npc_id);
            continue;
        };
        let expected = priced_stock(&vendors, &stores, *npc_id);
        assert!(
            !expected.is_empty(),
            "NPC {npc_id} ({name}) offers Shop without priced stock"
        );
        for item_id in &expected {
            assert!(
                inventory.has_definition(*item_id),
                "NPC {npc_id} ({name}) item {item_id} has no runtime definition"
            );
        }

        let character = npc_greetings::character_at(position);
        game.handle_in_world(&interact(*npc_id), &character, NOW)
            .unwrap_or_else(|error| panic!("NPC {npc_id} ({name}) 26/8 through gameplay: {error}"));
        let outcome = game
            .handle_in_world(&select(*npc_id, shop.id as u32), &character, NOW)
            .unwrap_or_else(|error| {
                panic!("NPC {npc_id} ({name}) 26/10 through gameplay: {error}")
            });
        let packet = outcome
            .replies
            .iter()
            .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
            .unwrap_or_else(|| panic!("NPC {npc_id} ({name}) answered no 165/10 window"));
        let group = decode_coin_merchant_list_165_10(packet)
            .unwrap_or_else(|error| panic!("NPC {npc_id} ({name}) 165/10 layout: {error:?}"));
        let advertised: BTreeSet<i32> = group
            .entries
            .iter()
            .map(|entry| {
                assert!(
                    entry.can_buy && entry.name_id != 0,
                    "NPC {npc_id} ({name}) row {} cannot render a buyable name",
                    entry.item_id
                );
                i32::try_from(entry.item_id).expect("shop item id fits i32")
            })
            .collect();
        assert!(
            !advertised.is_empty(),
            "NPC {npc_id} ({name}) opened an empty 165/10 window"
        );
        assert_eq!(
            advertised, expected,
            "NPC {npc_id} ({name}) 165/10 window does not list its priced stock"
        );
        shops.insert(*npc_id);
    }

    assert_eq!(
        shops.len(),
        SHOP_VENDORS,
        "the set of NPCs that offer a shop changed"
    );
    assert_eq!(
        no_shop.len(),
        NO_SHOP_VENDORS,
        "the set of vendor-role NPCs without a shop changed"
    );
    let pinned: BTreeSet<u64> = NO_SHOP.iter().copied().collect();
    assert_eq!(
        pinned.len(),
        NO_SHOP.len(),
        "the pinned no-shop list has duplicates"
    );
    assert_eq!(
        no_shop, pinned,
        "the vendor NPCs without a shop changed; every one needs retained stock evidence"
    );
}

/// Mr. Twinkle through the real gateway: no Shop choice (he has no recovered
/// stock), a choice the menu does not offer is refused without consuming the
/// menu, and Talk still opens his dialogue.
#[test]
fn the_gateway_offers_mr_twinkle_no_empty_shop_and_keeps_his_dialogue() {
    let content = content_dir();
    let reference = round70_content::reference();
    let gameplay = round70_content::source();
    let position = npc_position(&npc_record(MR_TWINKLE_FILE, MR_TWINKLE), MR_TWINKLE);
    let state = Gameplay::load(&gameplay, None)
        .expect("the prepared gameplay content loads")
        .state()
        .clone();
    let mut gateway = connect_with(
        &reference,
        reference_character_at(position),
        &gameplay,
        state,
    );

    let menu = send(&mut gateway, &interact(MR_TWINKLE));
    assert!(
        menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "Mr. Twinkle answered no 26/9 menu"
    );
    assert!(
        !menu
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "Mr. Twinkle advertised a merchant window on interact"
    );

    // Shop used to be choice 2 (Talk is 1); it no longer exists, so the gateway
    // rejects the choice and the open menu survives for the Talk that follows.
    let before = gateway.snapshot();
    let refused = send(&mut gateway, &select(MR_TWINKLE, 2));
    assert!(
        refused.is_empty(),
        "a choice the menu does not offer sent a reply"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a refused menu choice changed the saved state"
    );

    let talk = send(&mut gateway, &select(MR_TWINKLE, 1));
    assert!(
        talk.iter().any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "Talk no longer opens Mr. Twinkle's dialogue after the refusal"
    );
    assert!(
        !talk
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "Talk answered with a merchant window"
    );
}
