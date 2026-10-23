//! T275: Bordon Cloo's recovered cooking-supply stock.
//!
//! The Chef guide Vendors/Ingredients list and video `ukTXIyG6EKI/00006` recover a
//! 12-row cooking-supply shop at 12 coins each: Spice, Bat Meat, Mystery Meat,
//! Spider Meat, Bear Meat, Butter, Egg, Broth, Sugar, Chocolate Bar and the two
//! Snowhill Ice Creams. The video's name plate is unreadable, so the Bordon Cloo
//! identity is reconstructed from the client Chef-vendor text (2840821030, store
//! names 102334807 and 1311255679) and the Sanctuary map frame
//! `ukTXIyG6EKI/00320` in the same video.
//!
//! The stock rows are shipped content. The tests drive the production path:
//! `NpcVisibility`/`Gameplay::load` -> 26/8 (`008b6bc0`) -> 26/10
//! (`00a36f80`) -> 165/10 -> 66/43 (`00b3ab20`) -> 66/44 (`00b32430`), plus
//! 165/5 (`00c75a70`) -> 165/6 and 165/12 buy-back. The item ids are the ones
//! the Chef recipes in `data/content/recipes.json` already bind; duplicate
//! catalogue printings are excluded by that binding.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/resale.rs"]
mod resale;

use gateway_vendor::{
    NOW, count_definition, inner_packets, interact, inventory_of, login, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    coin_shop,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::{InventoryItem, InventoryState},
    npc_interaction::{
        MerchantBuyRequestV2, NpcAction, NpcInteraction, decode_coin_merchant_list_165_10,
        encode_merchant_buy_66_43,
    },
    world::NpcVisibility,
};
use resale::resale;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

const BORDON: u64 = 9_000_000_028;
const BORDON_NAME: &str = "Bordon Cloo";
const PRICE: u32 = 12;
/// A pre-existing sellable item used to free a bag slot or top up coins.
const SEED_DEFINITION: i32 = 75_060;

/// The recovered shop rows in video order: (display name, definition id).
const INGREDIENTS: [(&str, i32); 12] = [
    ("Spice", 2900),
    ("Bat Meat", 2862),
    ("Mystery Meat", 2859),
    ("Spider Meat", 2861),
    ("Bear Meat", 2860),
    ("Butter", 2902),
    ("Egg", 2897),
    ("Broth", 2899),
    ("Sugar", 2906),
    ("Chocolate Bar", 2898),
    ("Snowhill Ice Cream: Chocolate", 2896),
    ("Snowhill Ice Cream: Vanilla", 2895),
];

/// Exact-name duplicates that the Chef recipe binding excludes: the NoSale
/// printings (30573, 30576) and the category-42 quest copies (10005-10008).
const DUPLICATE_PRINTINGS: [i32; 6] = [30_573, 30_576, 10_005, 10_006, 10_007, 10_008];

/// Harvested crops named by the Chef guide. The recovered shop sells none of
/// them; a later harvesting/quest slice owns those sources.
const HARVESTED: [&str; 18] = [
    "Spiralmint",
    "Bumbleberry",
    "Gummyfruit",
    "Rainleaf",
    "Briarmelon",
    "Moonbean",
    "Stainberry",
    "Choychoy",
    "Chugaroot",
    "Sunleaf",
    "Punkin",
    "Stickfruit",
    "Starnut",
    "Snowberry",
    "Honeytwig",
    "Tumtumnip",
    "Bundle of Rainleaf",
    "Bundle of Gummyfruit",
];

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn lab_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

/// The prepared full-content pack with the 58,835-row item catalogue. It has no
/// `npcs/`, so `Gameplay::load` uses the bundled `data/content` NPC rows, vendor
/// stock and stores this test exercises.
fn gameplay_dir() -> PathBuf {
    std::env::var_os("OSFR_COOKING_SUPPLIES_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| lab_dir().join("reports/round70-native-20261016T061000Z/content"))
}

fn reference_dir() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| lab_dir().join("references/rust-gateway-reference"))
}

fn read_json(path: &Path) -> Value {
    let bytes =
        std::fs::read(path).unwrap_or_else(|error| panic!("{} reads: {error}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!("{} parses: {error}", path.display());
    })
}

fn bordon_record() -> Value {
    let document = read_json(&content_dir().join("npcs/sanctuary.json"));
    document["records"]
        .as_array()
        .expect("Sanctuary records are an array")
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(BORDON))
        .expect("Bordon Cloo exists in Sanctuary")
        .clone()
}

fn bordon_position() -> [f32; 3] {
    let record = bordon_record();
    let position = record["position"]
        .as_array()
        .expect("Bordon Cloo has a position");
    assert_eq!(position.len(), 3);
    [
        position[0].as_f64().expect("x") as f32,
        position[1].as_f64().expect("y") as f32,
        position[2].as_f64().expect("z") as f32,
    ]
}

/// The vendor record the production loader will serve, exactly once.
fn vendor_record() -> Value {
    let document = read_json(&content_dir().join("vendors.json"));
    let matches: Vec<&Value> = document["vendors"]
        .as_array()
        .expect("vendor records are an array")
        .iter()
        .filter(|vendor| vendor["npc_id"].as_u64() == Some(BORDON))
        .collect();
    assert_eq!(matches.len(), 1, "one Bordon Cloo vendor record");
    matches[0].clone()
}

/// The 66/10 choice id the production menu offers for the shop. Loading the
/// service also proves the shipped content passes vendor-role, price and stock
/// admission.
fn shop_choice() -> u32 {
    let content = content_dir();
    let mut service = NpcInteraction::load(&content.join("npcs"), &content.join("stores.json"))
        .expect("the production NPC loader accepts Bordon's vendor record");
    let (zone, position) = service.location(BORDON).expect("Bordon has a location");
    let menu = service
        .interact(BORDON, zone, position, None)
        .expect("Bordon offers a menu");
    let choice = menu
        .options
        .iter()
        .find(|option| option.action == NpcAction::Shop)
        .expect("Bordon offers the shop");
    u32::try_from(choice.id).expect("the shop choice fits the client field")
}

fn base_state() -> &'static GameplayState {
    static BASE: std::sync::OnceLock<GameplayState> = std::sync::OnceLock::new();
    BASE.get_or_init(|| {
        Gameplay::load(&gameplay_dir(), None)
            .expect("the prepared gameplay content loads")
            .state()
    })
}

fn state_with(coins: u32, stacks: &[(i32, i32, u32)]) -> GameplayState {
    let mut state = base_state().clone();
    state.inventory.items.clear();
    for slot in &mut state.inventory.equipment {
        slot.item_id = 0;
    }
    state.inventory.coins = coins;
    let mut next = 100_000i64;
    for &(instance, definition, count) in stacks {
        state.inventory.items.push(InventoryItem {
            id: instance,
            definition_id: definition,
            tint: 0,
            count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        next = next.max(i64::from(instance) + 1);
    }
    state.inventory.next_instance_id = Some(next);
    state
}

fn character_at(position: [f32; 3]) -> CharacterState {
    let mut character = Gateway::reference_state(&reference_dir()).expect("reference character");
    character.zone_id = 0;
    character.position = position;
    character.rotation = [1.0, 0.0, 0.0];
    character
}

fn connect(character: CharacterState, state: GameplayState) -> Gateway {
    let mut gateway = Gateway::load_gameplay(
        &reference_dir(),
        character.clone(),
        &gameplay_dir(),
        Some(state),
    )
    .expect("the gateway loads the prepared content");
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    for packet in [login(character.guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway
            .handle_at(&packet, NOW)
            .expect("gateway initialization handles");
    }
    gateway
}

fn player(coins: u32, stacks: &[(i32, i32, u32)]) -> Gateway {
    connect(character_at(bordon_position()), state_with(coins, stacks))
}

/// The two save halves as the production save serializes them. A refusal must
/// leave both files byte-identical.
fn save_bytes(gateway: &Gateway) -> Vec<u8> {
    let snapshot = gateway.snapshot();
    let mut bytes = serde_json::to_vec(&snapshot.character).expect("character save serializes");
    bytes.extend(
        serde_json::to_vec(&snapshot.gameplay.expect("gameplay state"))
            .expect("gameplay save serializes"),
    );
    bytes
}

fn open_shop(gateway: &mut Gateway) -> Vec<Vec<u8>> {
    let menu = send(gateway, &interact(BORDON));
    assert!(
        menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "26/8 must answer with the 26/9 interaction list"
    );
    let replies = send(gateway, &select(BORDON, shop_choice()));
    assert!(
        replies
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "26/10 must open Bordon's 165/10 coin merchant list"
    );
    replies
}

fn shop_entries(replies: &[Vec<u8>]) -> Vec<i32> {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("the 165/10 window is present");
    let decoded =
        decode_coin_merchant_list_165_10(packet).expect("165/10 matches the client reader");
    let mut rows: Vec<i32> = decoded
        .entries
        .iter()
        .map(|entry| entry.item_id as i32)
        .collect();
    rows.sort_unstable();
    rows
}

fn buy(gateway: &mut Gateway, transaction: u32, item: i32, quantity: u32) -> Vec<Vec<u8>> {
    send(gateway, &vendor_buy_packet(transaction, item, quantity))
}

fn buy_result(replies: &[Vec<u8>]) -> u32 {
    merchant_status(replies.last().expect("a 66/44 result follows a buy"))
}

fn stack_of(state: &InventoryState, definition: i32) -> Option<(i32, u32)> {
    state
        .items
        .iter()
        .find(|item| item.definition_id == definition)
        .map(|item| (item.id, item.count))
}

fn sell_completion(replies: &[Vec<u8>]) -> (u32, u32, u32, u32) {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .expect("a 165/6 completion follows a sell");
    assert_eq!(packet.len(), 56, "165/6 matches the client reader");
    let word = |offset: usize| u32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap());
    (word(4), word(24), word(48), word(52))
}

#[test]
fn bordon_sells_only_the_resolved_recovered_ingredients() {
    let vendor = vendor_record();
    assert_eq!(vendor["npc_name"], BORDON_NAME);
    assert_eq!(vendor["npc_content_file"], "sanctuary.json");
    assert_eq!(
        vendor["stock_completeness"],
        "all_named_evidence_items_mapped"
    );

    let items = vendor["items"].as_array().expect("items are an array");
    let item_ids: Vec<i32> = vendor["item_ids"]
        .as_array()
        .expect("item_ids are an array")
        .iter()
        .map(|id| id.as_i64().expect("item id is an integer") as i32)
        .collect();
    assert_eq!(items.len(), INGREDIENTS.len(), "all twelve rows resolved");
    let expected: Vec<i32> = INGREDIENTS.iter().map(|(_, id)| *id).collect();
    assert_eq!(item_ids, expected, "video order and no invented id");

    let unique: BTreeSet<i32> = item_ids.iter().copied().collect();
    assert_eq!(unique.len(), item_ids.len(), "no duplicate id");
    assert!(
        DUPLICATE_PRINTINGS.iter().all(|id| !unique.contains(id)),
        "a duplicate exact-name printing was chosen"
    );
    for name in HARVESTED {
        assert!(
            !items.iter().any(|item| item["item_name"] == name),
            "harvested item {name} must not be sold by this vendor"
        );
    }

    let stores = read_json(&content_dir().join("stores.json"));
    let store_rows: BTreeMap<i32, &Value> = stores["items"]
        .as_array()
        .expect("store rows are an array")
        .iter()
        .map(|row| (row["item_id"].as_i64().expect("store item id") as i32, row))
        .collect();
    let locale_doc = read_json(&lab_dir().join("reports/locale-id-to-text.json"));
    let locale: BTreeMap<u64, String> = locale_doc
        .as_array()
        .expect("locale rows are an array")
        .iter()
        .map(|row| {
            (
                row["id"].as_u64().expect("locale id"),
                row["text"].as_str().expect("locale text").to_owned(),
            )
        })
        .collect();
    let recipes = read_json(&content_dir().join("recipes.json"));
    let chef_ingredients: BTreeSet<i32> = recipes["records"]
        .as_array()
        .expect("recipe records are an array")
        .iter()
        .filter(|recipe| recipe["job"] == "chef")
        .flat_map(|recipe| {
            recipe["ingredients"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|ingredient| ingredient["definition_id"].as_i64().map(|id| id as i32))
        })
        .collect();

    for ((name, expected_id), item) in INGREDIENTS.iter().zip(items) {
        assert_eq!(item["item_id"].as_i64(), Some(i64::from(*expected_id)));
        assert_eq!(item["item_name"], *name);
        assert_eq!(item["source_stock_name"], *name);
        assert_eq!(item["runtime_eligible"], true);
        assert!(
            item["source_tags"]
                .as_array()
                .expect("source tags")
                .iter()
                .any(|tag| tag == "video"),
            "{name} must carry the video stock tag"
        );

        let name_id = item["item_name_id"].as_u64().expect("name id");
        let locale_text = locale
            .get(&name_id)
            .unwrap_or_else(|| panic!("{name}: locale id {name_id}"));
        assert_eq!(
            locale_text, name,
            "{name}: exact default-locale display name"
        );

        let store = store_rows
            .get(expected_id)
            .unwrap_or_else(|| panic!("{name}: store row missing"));
        assert_eq!(
            store["purchase_price_coins"], PRICE,
            "{name}: 12-coin price"
        );
        assert_eq!(store["member_price_coins"], 10, "{name}: member price");
        assert_eq!(
            store["sell_price_coins"],
            resale(*expected_id),
            "{name}: resale"
        );
        assert_eq!(store["no_sale"], false, "{name}: sellable policy");
        assert_eq!(
            item["purchase_price_coins"], store["purchase_price_coins"],
            "{name}: vendor price is the store price"
        );

        assert!(
            chef_ingredients.contains(expected_id),
            "{name} ({expected_id}) is not an ingredient bound by a Chef recipe"
        );
    }
}

#[test]
fn buying_each_ingredient_grants_one_stack_and_debits_twelve_coins() {
    let mut gateway = player(200, &[]);
    let before = inventory_of(&gateway);
    let entries = shop_entries(&open_shop(&mut gateway));
    let mut expected: Vec<i32> = INGREDIENTS.iter().map(|(_, id)| *id).collect();
    expected.sort_unstable();
    assert_eq!(entries, expected, "165/10 lists exactly the twelve rows");

    let mut coins = before.coins;
    for (index, (name, item_id)) in INGREDIENTS.iter().enumerate() {
        let transaction = 5_000 + index as u32;
        let replies = buy(&mut gateway, transaction, *item_id, 1);
        assert_eq!(buy_result(&replies), 1, "{name}: 66/44 success");
        coins -= PRICE;
        let after = inventory_of(&gateway);
        assert_eq!(after.coins, coins, "{name}: exact coin debit");
        assert_eq!(
            stack_of(&after, *item_id),
            Some((100_000 + index as i32, 1)),
            "{name}: one new stack"
        );
        assert_eq!(count_definition(&after, *item_id), 1, "{name}: one item");
        assert_eq!(
            after.items.iter().filter(|item| item.count > 0).count(),
            index + 1,
            "{name}: one slot consumed"
        );
    }

    // A second buy of the same row stacks on the first stack instead of
    // inventing a slot. The client cannot influence the price: 66/43 carries no
    // price field, and forged context words are ignored.
    let (name, item_id) = INGREDIENTS[0];
    let replies = buy(&mut gateway, 5_100, item_id, 1);
    assert_eq!(buy_result(&replies), 1, "{name}: stacking buy success");
    let forged = encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [item_id as u32, u32::MAX],
        context_dwords: [7, 8, 9],
        context_bytes: b"forged".to_vec(),
        trailing_dwords: [1, 0, 5_101],
    })
    .expect("the forged-context 66/43 encodes");
    let replies = send(&mut gateway, &forged);
    assert_eq!(buy_result(&replies), 1, "{name}: forged context ignored");
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, coins - 2 * PRICE, "server price, not client");
    assert_eq!(
        stack_of(&after, item_id),
        Some((100_000, 3)),
        "{name}: stacked, one slot"
    );
    assert_eq!(
        after.items.iter().filter(|item| item.count > 0).count(),
        INGREDIENTS.len(),
        "stacking did not add a slot"
    );
}

#[test]
fn insufficient_coins_full_bag_and_unknown_item_refuse_and_change_nothing() {
    // Insufficient coins: 11 coins cannot buy a 12-coin row.
    let mut poor = player(11, &[(100_000, SEED_DEFINITION, 1)]);
    open_shop(&mut poor);
    let before = poor.snapshot();
    let before_bytes = save_bytes(&poor);
    let replies = buy(&mut poor, 6_001, INGREDIENTS[0].1, 1);
    assert_eq!(replies.len(), 1, "one client-visible refusal");
    assert_eq!(buy_result(&replies), 0, "insufficient coins refused");
    assert_eq!(poor.snapshot(), before, "insufficient coins changed state");
    assert_eq!(
        save_bytes(&poor),
        before_bytes,
        "insufficient coins changed the save"
    );
    // Canary: the refusal did not consume the shop. Selling the seeded stack
    // tops up the balance and the same buy lands.
    let (instance, _) = stack_of(&inventory_of(&poor), SEED_DEFINITION).unwrap();
    let sold = send(&mut poor, &sell_packet(0, instance, 1));
    assert_eq!(sell_completion(&sold).0, 1, "the refusal closed the shop");
    let topped_up = inventory_of(&poor).coins;
    let replies = buy(&mut poor, 6_002, INGREDIENTS[0].1, 1);
    assert_eq!(buy_result(&replies), 1, "buy after the refusal");
    assert_eq!(inventory_of(&poor).coins, topped_up - PRICE);

    // Full bag: 100 occupied stacks is the local bag cap.
    let full_stacks: Vec<(i32, i32, u32)> = (0..100)
        .map(|index| (100_000 + index, SEED_DEFINITION, 1))
        .collect();
    let mut full = player(1_000, &full_stacks);
    open_shop(&mut full);
    let before = full.snapshot();
    let before_bytes = save_bytes(&full);
    let replies = buy(&mut full, 6_003, INGREDIENTS[0].1, 1);
    assert_eq!(replies.len(), 1);
    assert_eq!(buy_result(&replies), 0, "full bag refused");
    assert_eq!(full.snapshot(), before, "full bag changed state");
    assert_eq!(save_bytes(&full), before_bytes, "full bag changed the save");
    let sold = send(&mut full, &sell_packet(0, 100_000, 1));
    assert_eq!(sell_completion(&sold).0, 1, "the refusal closed the shop");
    let freed = inventory_of(&full);
    let replies = buy(&mut full, 6_004, INGREDIENTS[0].1, 1);
    assert_eq!(buy_result(&replies), 1, "buy after freeing a slot");
    assert_eq!(inventory_of(&full).items.len(), 100);
    assert_eq!(
        inventory_of(&full).coins,
        freed.coins.saturating_sub(PRICE),
        "full-bag retry debits once"
    );

    // Malformed bodies, unknown item, zero and overflowing quantity: each is a
    // 66/44 failure and none changes the gateway state or closes the shop.
    let mut gateway = player(100, &[]);
    open_shop(&mut gateway);
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("header only", vec![66, 0, 43, 0]),
        ("unknown item", vendor_buy_packet(6_010, i32::MAX, 1)),
        (
            "zero quantity",
            vendor_buy_packet(6_011, INGREDIENTS[0].1, 0),
        ),
        (
            "overflow quantity",
            vendor_buy_packet(6_012, INGREDIENTS[0].1, u32::MAX),
        ),
        ("long body", {
            let mut packet = vendor_buy_packet(6_013, INGREDIENTS[0].1, 1);
            packet.push(0);
            packet
        }),
    ];
    for (label, packet) in cases {
        let before = gateway.snapshot();
        let before_bytes = save_bytes(&gateway);
        let replies = send(&mut gateway, &packet);
        assert_eq!(replies.len(), 1, "{label}: one client-visible refusal");
        assert_eq!(buy_result(&replies), 0, "{label}: refused");
        assert_eq!(gateway.snapshot(), before, "{label}: state changed");
        assert_eq!(
            save_bytes(&gateway),
            before_bytes,
            "{label}: save bytes changed"
        );
    }
    // Canary: the shop still works after every refusal.
    let replies = buy(&mut gateway, 6_020, INGREDIENTS[0].1, 1);
    assert_eq!(buy_result(&replies), 1, "the refusals consumed the shop");

    // A fresh session has no open shop; the buy is refused and changes nothing.
    let mut closed = player(100, &[]);
    let before = closed.snapshot();
    let before_bytes = save_bytes(&closed);
    let replies = buy(&mut closed, 6_030, INGREDIENTS[0].1, 1);
    assert_eq!(replies.len(), 1);
    assert_eq!(buy_result(&replies), 0, "closed shop refused");
    assert_eq!(closed.snapshot(), before, "closed shop changed state");
    assert_eq!(
        save_bytes(&closed),
        before_bytes,
        "closed shop changed the save"
    );

    // One player's error never stops the server: a second session still buys.
    let mut other = player(100, &[]);
    open_shop(&mut other);
    let replies = buy(&mut other, 6_040, INGREDIENTS[1].1, 1);
    assert_eq!(buy_result(&replies), 1, "the other player is unaffected");
    assert_eq!(inventory_of(&other).coins, 100 - PRICE);
}

#[test]
fn selling_an_ingredient_back_uses_the_store_price_and_is_once() {
    let mut gateway = player(100, &[]);
    open_shop(&mut gateway);
    let replies = buy(&mut gateway, 7_001, INGREDIENTS[0].1, 1);
    assert_eq!(buy_result(&replies), 1);
    let bought = inventory_of(&gateway);
    assert_eq!(bought.coins, 100 - PRICE);
    let (instance, _) = stack_of(&bought, INGREDIENTS[0].1).expect("the bought stack");

    // 165/5 names no merchant (null session GUID); 165/6 pays the store row (a tenth, T633).
    let sell_price = resale(INGREDIENTS[0].1);
    let sold = send(&mut gateway, &sell_packet(0, instance, 1));
    let (result, transaction, price, remaining) = sell_completion(&sold);
    assert_eq!(result, coin_shop::RESULT_SUCCESS, "sale accepted");
    assert_eq!(price, sell_price, "resale price is the store row");
    assert_eq!(remaining, 1, "one copy can be bought back");
    let after_sale = inventory_of(&gateway);
    assert_eq!(after_sale.coins, bought.coins + sell_price);
    assert_eq!(stack_of(&after_sale, INGREDIENTS[0].1), None);

    // The same sell again fails and pays nothing.
    let before_replay = gateway.snapshot();
    let replay = send(&mut gateway, &sell_packet(0, instance, 1));
    assert_ne!(sell_completion(&replay).0, coin_shop::RESULT_SUCCESS);
    assert_eq!(
        gateway.snapshot(),
        before_replay,
        "replayed sell paid twice"
    );

    // 165/12 buys the sold copy back at the sale price exactly once.
    let mut buy_back = vec![165, 0, 12, 0];
    buy_back.extend(transaction.to_le_bytes());
    buy_back.extend(0u32.to_le_bytes());
    let restored = send(&mut gateway, &buy_back);
    let reply = restored
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 13, 0]))
        .expect("165/13 buy-back result");
    assert_eq!(
        u32::from_le_bytes(reply[8..12].try_into().unwrap()),
        coin_shop::RESULT_SUCCESS
    );
    let after_back = inventory_of(&gateway);
    assert_eq!(after_back.coins, after_sale.coins - sell_price);
    assert_eq!(count_definition(&after_back, INGREDIENTS[0].1), 1);
    let before_second = gateway.snapshot();
    let again = send(&mut gateway, &buy_back);
    let reply = again
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 13, 0]))
        .expect("a second 165/13 result");
    assert_ne!(
        u32::from_le_bytes(reply[8..12].try_into().unwrap()),
        coin_shop::RESULT_SUCCESS,
        "buy-back is once"
    );
    assert_eq!(gateway.snapshot(), before_second);
}

#[test]
fn the_shop_survives_a_relog() {
    let mut gateway = player(100, &[]);
    open_shop(&mut gateway);
    for (index, (name, item_id)) in INGREDIENTS.iter().take(2).enumerate() {
        let replies = buy(&mut gateway, 8_000 + index as u32, *item_id, 1);
        assert_eq!(buy_result(&replies), 1, "{name}: bought before relog");
    }
    let before = inventory_of(&gateway);
    assert_eq!(before.coins, 100 - 2 * PRICE);
    let expected_entries = shop_entries(&open_shop(&mut gateway));

    // Serialize both save halves, then reconnect through the real loader.
    let snapshot = gateway.snapshot();
    let character: CharacterState =
        serde_json::from_slice(&serde_json::to_vec(&snapshot.character).unwrap()).unwrap();
    let state: GameplayState = serde_json::from_slice(
        &serde_json::to_vec(&snapshot.gameplay.expect("gameplay state")).unwrap(),
    )
    .unwrap();
    let mut reconnected = connect(character, state);
    assert_eq!(
        inventory_of(&reconnected),
        before,
        "coins and stacks survived the relog"
    );
    let entries = shop_entries(&open_shop(&mut reconnected));
    assert_eq!(
        entries, expected_entries,
        "the stock list is the same resolved rows"
    );
    let replies = buy(&mut reconnected, 8_010, INGREDIENTS[2].1, 1);
    assert_eq!(buy_result(&replies), 1, "the relogged shop still buys");
    assert_eq!(inventory_of(&reconnected).coins, before.coins - PRICE);
}

#[test]
fn the_vendor_is_keyed_to_the_drawable_bordon() {
    let record = bordon_record();
    assert_eq!(record["name"], BORDON_NAME);
    assert_eq!(record["name_string_id"], 15_300);
    assert_eq!(record["model_id"], 355);
    assert_eq!(record["placement"], "legacy_data");
    let position = bordon_position();
    for (value, expected) in position.iter().zip([-1243.191, -14.34194, 408.6057]) {
        assert!((value - expected).abs() < 1e-3, "Bordon position drifted");
    }
    assert!(
        record["roles"]
            .as_array()
            .expect("roles are an array")
            .iter()
            .any(|role| role == "vendor"),
        "the production loader requires the vendor role"
    );
    assert_eq!(record["greeting_string_id"], 387246);
    let vendor = vendor_record();
    assert_eq!(vendor["npc_name"], BORDON_NAME);
    assert_eq!(vendor["npc_content_file"], "sanctuary.json");

    // The spawn overlay publishes Bordon at the content position, and the
    // runtime GUID maps back to the same 9000000028 content key.
    let visibility = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    assert_eq!(visibility.content_key(BORDON), Some(BORDON));
    let spawned = visibility
        .teleport_targets(0)
        .into_iter()
        .find(|target| target.guid == BORDON)
        .expect("Bordon is spawned");
    for axis in 0..3 {
        assert!((spawned.position[axis] - position[axis]).abs() < 1e-3);
    }

    // Real in-range 26/8 -> 26/10 opens the 165/10 coin window and 66/43 buys.
    let mut gateway = player(100, &[]);
    let menu = send(&mut gateway, &interact(BORDON));
    assert!(menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])));
    let replies = send(&mut gateway, &select(BORDON, shop_choice()));
    assert_eq!(shop_entries(&replies).len(), INGREDIENTS.len());
    let replies = buy(&mut gateway, 9_001, INGREDIENTS[0].1, 1);
    assert_eq!(buy_result(&replies), 1);

    // Out of range: the character is 100 m away; the interaction fails and
    // nothing changes.
    let mut far = connect(
        character_at([position[0] + 100.0, position[1], position[2]]),
        state_with(100, &[]),
    );
    let before = far.snapshot();
    let outcome = far
        .handle_at(&wrap(&interact(BORDON)), NOW)
        .expect("an out-of-range request is a per-session refusal");
    assert!(
        inner_packets(&outcome).is_empty(),
        "an out-of-range interaction must send nothing"
    );
    assert_eq!(far.snapshot(), before, "out-of-range changed state");

    // A non-Bordon target cannot open the shop.
    let mut other = player(100, &[]);
    let before = other.snapshot();
    let outcome = other
        .handle_at(&wrap(&interact(BORDON + 1)), NOW)
        .expect("an unknown target is ignored");
    assert!(
        inner_packets(&outcome)
            .iter()
            .all(|packet| !packet.starts_with(&[165, 0, 10, 0])),
        "a non-Bordon target opened a shop"
    );
    assert_eq!(other.snapshot(), before);

    // A 26/10 choice without the 26/8 menu is refused, and the gateway keeps
    // running.
    let outcome = other
        .handle_at(&wrap(&select(BORDON, shop_choice())), NOW)
        .expect("the session survives a refused choice");
    assert!(
        inner_packets(&outcome).is_empty(),
        "a choice without the menu must send nothing"
    );
    assert_eq!(other.snapshot(), before);
    open_shop(&mut other);
    let replies = buy(&mut other, 9_002, INGREDIENTS[1].1, 1);
    assert_eq!(buy_result(&replies), 1);
}

#[test]
fn a_second_player_does_not_share_bordon_state() {
    let mut first = player(100, &[]);
    let mut second = player(100, &[]);
    let list = shop_entries(&open_shop(&mut first));
    assert_eq!(list, shop_entries(&open_shop(&mut second)));

    assert_eq!(buy_result(&buy(&mut first, 10_001, INGREDIENTS[0].1, 1)), 1);
    let first_state = inventory_of(&first);
    assert_eq!(first_state.coins, 100 - PRICE);
    assert_eq!(stack_of(&first_state, INGREDIENTS[0].1), Some((100_000, 1)));
    assert_eq!(stack_of(&inventory_of(&second), INGREDIENTS[0].1), None);

    let second_before = second.snapshot();
    assert_eq!(
        buy_result(&buy(&mut second, 10_002, INGREDIENTS[1].1, 1)),
        1
    );
    let second_state = inventory_of(&second);
    assert_eq!(second_state.coins, 100 - PRICE);
    assert_eq!(
        stack_of(&second_state, INGREDIENTS[1].1),
        Some((100_000, 1))
    );
    assert_eq!(
        stack_of(&inventory_of(&second), INGREDIENTS[0].1),
        None,
        "the second player must not receive the first player's item"
    );
    assert_eq!(inventory_of(&first), first_state, "the first player moved");
    assert_ne!(
        second.snapshot(),
        second_before,
        "the second buy must commit"
    );

    // The stock is content, not a per-player depletion: both windows still
    // list the twelve rows after the buys.
    assert_eq!(shop_entries(&open_shop(&mut first)), list);
    assert_eq!(shop_entries(&open_shop(&mut second)), list);
}
