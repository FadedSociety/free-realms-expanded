//! F228: a job starter outfit item is free issuance, so it resells for 0 coins wherever the
//! player got that copy. The regression drives the real gateway path: learn a job from its
//! trainer, unequip and sell the starter weapon, reselect the job (which re-grants it) and sell
//! again, then buy the same item from a merchant and sell that copy. A non-starter item still
//! pays its catalogued price, and (L040) the Wilds Condo starter kit resells for 0 as well.
//!
//! Client facts: 165/5 names no price (sender `00c76dd0`/`00c75a70` write the merchant, item
//! instance and quantity only), and 165/6 carries the server's unit price (`00c71f30` reads
//! quantity at record +0xc, price at +0xe and remaining at +0xd). The 0-coin rule is server
//! policy (R). Synthetic content only; no player data or chat text.

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;

use gateway_vendor::{
    NOW, count_definition, inner_packets, interact, inventory_of, login, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    coin_shop,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    inventory::InventoryState,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const ARCHER: i32 = 35;
const ARCHER_TRAINER: u64 = 9_000_000_387;
const ARCHER_LABEL: u32 = 412_187;
/// The Archer starter bow from the content's `starter_items[35]`.
const STARTER_BOW: i32 = 4_266;
/// An ordinary catalogued item that is no job's starter.
const NORMAL_ITEM: i32 = 8;
const VENDOR: u64 = 9_500_000_339;
const STARTER_SELL_PRICE: u32 = 375;
const NORMAL_SELL_PRICE: u32 = 7;
const RESULT_OK: u32 = coin_shop::RESULT_SUCCESS;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The prepared play package whose `profiles.json` carries the trainer jobs' starter outfits.
fn reference_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn reference_gateway() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        })
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn store_row(item_id: i32, buy: u32, sell: u32) -> Value {
    json!({
        "item_id": item_id,
        "purchase_price_coins": buy,
        "member_discount_percent": null,
        "member_price_coins": null,
        "sell_price_coins": sell,
        "no_sale": false
    })
}

/// The 165/6 record as `00c74ef0`/`00c71f30` read it. Field names beyond the result are (R).
#[derive(Debug, PartialEq)]
struct Completion {
    result: u32,
    definition: i32,
    quantity: u32,
    price: u32,
}

fn completions(replies: &[Vec<u8>]) -> Vec<Completion> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(|packet| {
            assert_eq!(packet.len(), 56, "completion must match the reader size");
            let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
            Completion {
                result: word(4),
                definition: word(36) as i32,
                quantity: word(44),
                price: word(48),
            }
        })
        .collect()
}

fn one_completion(replies: &[Vec<u8>]) -> Completion {
    let mut done = completions(replies);
    assert_eq!(done.len(), 1, "one 165/6 completion per sell");
    done.remove(0)
}

fn instance_of(state: &InventoryState, definition: i32) -> i32 {
    state
        .items
        .iter()
        .find(|item| item.definition_id == definition)
        .unwrap_or_else(|| panic!("definition {definition} is not owned"))
        .id
}

fn equipped_item(state: &InventoryState, job: i32, slot: i32) -> i32 {
    state
        .equipment
        .iter()
        .find(|entry| entry.profile_id == job && entry.slot == slot)
        .map_or(0, |entry| entry.item_id)
}

fn active_job(gateway: &Gateway) -> i32 {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay state exists")
        .profiles
        .expect("class snapshot exists")
        .active
}

fn has_system_line(replies: &[Vec<u8>], needle: &str) -> bool {
    replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(needle))
}

/// The option id of the 26/9 menu entry whose label is `string_id`.
fn option_with_label(menu: &[Vec<u8>], string_id: u32) -> Option<u32> {
    let packet = menu.iter().find(|p| p.starts_with(&[26, 0, 9, 0]))?;
    let count = i32::from_le_bytes(packet[13..17].try_into().ok()?) as usize;
    (0..count).find_map(|index| {
        let at = 17 + index * 28;
        let id = i32::from_le_bytes(packet[at..at + 4].try_into().ok()?);
        let label = u32::from_le_bytes(packet[at + 8..at + 12].try_into().ok()?);
        (label == string_id).then_some(id as u32)
    })
}

fn request(gateway: &mut Gateway, inner: &[u8]) -> Vec<Vec<u8>> {
    inner_packets(
        &gateway
            .handle_at(&wrap(inner), NOW)
            .unwrap_or_else(|error| panic!("gateway refused the request: {error}")),
    )
}

fn learn_archer(gateway: &mut Gateway) {
    let menu = request(gateway, &interact(ARCHER_TRAINER));
    let option = option_with_label(&menu, ARCHER_LABEL).expect("the Archer trainer offers Learn");
    let learned = request(gateway, &select(ARCHER_TRAINER, option));
    assert!(
        has_system_line(&learned, "You learned the Archer job"),
        "the trainer did not teach Archer: {:?}",
        String::from_utf8_lossy(&learned.concat())
    );
}

fn switch_to_archer(gateway: &mut Gateway) -> Vec<Vec<u8>> {
    let mut packet = vec![26, 0, 13, 0];
    packet.extend(ARCHER.to_le_bytes());
    request(gateway, &packet)
}

fn unequip_weapon(gateway: &mut Gateway) {
    let mut packet = vec![42, 0, 2, 0];
    packet.extend(7i32.to_le_bytes());
    packet.extend(ARCHER.to_le_bytes());
    let replies = request(gateway, &packet);
    assert!(
        replies
            .iter()
            .any(|packet| packet.starts_with(&[38, 0, 6, 0])),
        "the unequip was not acknowledged"
    );
}

/// Interact with the synthetic vendor and choose its Shop option (Talk is option 1). Returns the
/// 66/42 window as sent, so a test can read the advertised prices.
fn open_shop(gateway: &mut Gateway) -> Vec<Vec<u8>> {
    let menu = request(gateway, &interact(VENDOR));
    assert!(
        menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the vendor opened no menu"
    );
    let opened = request(gateway, &select(VENDOR, 2));
    assert!(
        opened
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
        "the vendor opened no shop window"
    );
    opened
}

/// The 66/42 rows `00b29cd0`/`00b2e620` read: item id, purchase price and sell price in the
/// first three entry dwords (semantic values are (R)).
fn merchant_rows(replies: &[Vec<u8>]) -> Vec<(i32, u32, u32)> {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("the shop window reply");
    let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let count = word(32) as usize;
    assert_eq!(packet.len(), 36 + count * 38, "66/42 must match its reader");
    (0..count)
        .map(|index| {
            let at = 36 + index * 38;
            (word(at) as i32, word(at + 4), word(at + 8))
        })
        .collect()
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    fn new() -> Self {
        let source = reference_content();
        assert!(
            source.join("profiles.json").is_file() && source.join("inventory.json").is_file(),
            "prepared content fixture missing: {}",
            source.display()
        );
        let content = std::env::temp_dir().join(format!(
            "osfr-f228-starter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        content_copy::copy_tree(&source, &content);
        fs::copy(
            content_dir().join("job_trainers.json"),
            content.join("job_trainers.json"),
        )
        .unwrap();
        fs::create_dir(content.join("npcs")).unwrap();
        write_json(
            &content.join("npcs/synthetic.json"),
            &json!({"schema_version":1,"records":[
                {"npc_id":ARCHER_TRAINER,"name":"Synthetic trainer","zone_id":0,
                 "position":[0.0,0.0,0.0],"placement":"legacy_data","model_id":1,
                 "roles":["trainer"]},
                {"npc_id":VENDOR,"name":"Synthetic vendor","zone_id":0,
                 "position":[1.0,0.0,0.0],"placement":"legacy_data","model_id":1,
                 "roles":["vendor"],"vendor_stock":"synthetic"}
            ]}),
        );
        write_json(
            &content.join("npc_spawns_extra.json"),
            &json!({"schema_version":1,"records":[]}),
        );
        write_json(
            &content.join("zones.json"),
            &json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
        );
        write_json(
            &content.join("vendors.json"),
            &json!({"schema_version":1,"vendors":[
                {"npc_id":VENDOR,"item_ids":[STARTER_BOW, NORMAL_ITEM]}
            ]}),
        );
        // The starter row deliberately keeps a nonzero reference sell price, so the gateway test
        // proves the server rule (not the store data) is what pays 0 for the free issuance.
        write_json(
            &content.join("stores.json"),
            &json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[
                store_row(STARTER_BOW, 10, STARTER_SELL_PRICE),
                store_row(NORMAL_ITEM, 15, NORMAL_SELL_PRICE)
            ]}),
        );
        Self {
            content,
            reference: reference_gateway(),
        }
    }

    /// A bare character: equipment and bag are empty so the 100-stack capacity cannot refuse a
    /// purchase, and the trainer's starter grant is what first creates the Archer outfit.
    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None)
            .expect("the prepared content loads")
            .state();
        state.inventory.items.clear();
        for entry in &mut state.inventory.equipment {
            entry.item_id = 0;
        }
        state.inventory.coins = 100;
        state
    }

    fn connect(&self) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.zone_id = 0;
        character.position = [0.0; 3];
        let mut gateway = Gateway::load_gameplay(
            &self.reference,
            character,
            &self.content,
            Some(self.state()),
        )
        .unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        for packet in [login(1), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway.handle_at(&packet, NOW).unwrap();
        }
        gateway
    }
}

#[test]
fn a_regranted_starter_weapon_sells_for_zero_and_a_normal_item_keeps_its_price() {
    let fixture = Fixture::new();
    let mut gateway = fixture.connect();
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 0);

    // Learn Archer; its starter table grants the bow and equips it.
    learn_archer(&mut gateway);
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 1);
    let learned_bow = instance_of(&inventory_of(&gateway), STARTER_BOW);
    assert_eq!(
        equipped_item(&inventory_of(&gateway), ARCHER, 7),
        learned_bow
    );

    // Selecting the job keeps that same starter weapon.
    switch_to_archer(&mut gateway);
    assert_eq!(active_job(&gateway), ARCHER);
    assert_eq!(
        equipped_item(&inventory_of(&gateway), ARCHER, 7),
        learned_bow
    );

    let shop_window = open_shop(&mut gateway);
    // F228: the 66/42 window advertises the same resale price the sale pays. The fixture keeps
    // the raw 375 in its synthetic stores.json, so only the shared starter rule can make this 0.
    let rows = merchant_rows(&shop_window);
    assert_eq!(rows.len(), 2, "both vendor items are listed: {rows:?}");
    assert!(
        rows.contains(&(STARTER_BOW, 10, 0)),
        "the starter row must advertise 0, not {STARTER_SELL_PRICE}: {rows:?}"
    );
    assert!(
        rows.contains(&(NORMAL_ITEM, 15, NORMAL_SELL_PRICE)),
        "a non-starter keeps its advertised price: {rows:?}"
    );
    // The equipped weapon is refused, and the refusal changes nothing (the shop stays open in
    // the canary below). A starter item is only sellable once it is off the body.
    let before_refusal = gateway.snapshot();
    let refused = one_completion(&send(&mut gateway, &sell_packet(0, learned_bow, 1)));
    assert_eq!(refused.result, coin_shop::RESULT_SERVICE_UNAVAILABLE);
    assert_eq!(gateway.snapshot(), before_refusal);

    // Sell the starter weapon: it leaves the bag and pays 0, not the catalogued 375.
    let coins_before = inventory_of(&gateway).coins;
    unequip_weapon(&mut gateway);
    let sold = one_completion(&send(&mut gateway, &sell_packet(0, learned_bow, 1)));
    assert_eq!(sold.result, RESULT_OK);
    assert_eq!(sold.definition, STARTER_BOW);
    assert_eq!(sold.quantity, 1);
    assert_eq!(sold.price, 0, "a starter item must not pay its store price");
    assert_eq!(inventory_of(&gateway).coins, coins_before);
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 0);

    // Reselect the job while it is already active: ensure_outfit re-grants the weapon, and the
    // fresh copy is still worth 0. The re-grant itself is unchanged (the owner, 2026-10-20).
    switch_to_archer(&mut gateway);
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 1);
    let regranted = instance_of(&inventory_of(&gateway), STARTER_BOW);
    assert_eq!(equipped_item(&inventory_of(&gateway), ARCHER, 7), regranted);
    open_shop(&mut gateway);
    unequip_weapon(&mut gateway);
    let sold = one_completion(&send(&mut gateway, &sell_packet(0, regranted, 1)));
    assert_eq!(sold.result, RESULT_OK);
    assert_eq!(sold.price, 0, "a re-granted starter must also sell for 0");
    assert_eq!(inventory_of(&gateway).coins, coins_before);
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 0);

    // Buy the same item from the merchant: the purchase price is unchanged, but a merchant copy
    // is the same starter item and still sells for 0.
    open_shop(&mut gateway);
    let bought = send(&mut gateway, &vendor_buy_packet(91_001, STARTER_BOW, 1));
    assert_eq!(merchant_status(bought.last().expect("vendor buy reply")), 1);
    let after_buy = inventory_of(&gateway);
    assert_eq!(after_buy.coins, coins_before - 10);
    assert_eq!(count_definition(&after_buy, STARTER_BOW), 1);
    let bought_bow = instance_of(&after_buy, STARTER_BOW);
    let sold = one_completion(&send(&mut gateway, &sell_packet(0, bought_bow, 1)));
    assert_eq!(sold.result, RESULT_OK);
    assert_eq!(
        sold.price, 0,
        "a merchant copy of a starter sells for 0 too"
    );
    assert_eq!(inventory_of(&gateway).coins, coins_before - 10);
    assert_eq!(count_definition(&inventory_of(&gateway), STARTER_BOW), 0);

    // A non-starter item still sells at its catalogued price.
    open_shop(&mut gateway);
    let bought = send(&mut gateway, &vendor_buy_packet(91_002, NORMAL_ITEM, 1));
    assert_eq!(merchant_status(bought.last().expect("vendor buy reply")), 1);
    let after_buy = inventory_of(&gateway);
    assert_eq!(after_buy.coins, coins_before - 25);
    let normal = instance_of(&after_buy, NORMAL_ITEM);
    let sold = one_completion(&send(&mut gateway, &sell_packet(0, normal, 1)));
    assert_eq!(sold.result, RESULT_OK);
    assert_eq!(sold.price, NORMAL_SELL_PRICE);
    assert_eq!(
        inventory_of(&gateway).coins,
        coins_before - 25 + NORMAL_SELL_PRICE
    );
    assert_eq!(count_definition(&inventory_of(&gateway), NORMAL_ITEM), 0);
}

/// F228 regression: the store rule reuses the session's already-loaded profiles. It used to
/// parse `profiles.json` a second time with no saved state, so a package whose saved progression
/// was valid but whose unused `default_state` was not failed at shop construction.
#[test]
fn a_saved_package_loads_with_an_invalid_unused_default_state() {
    let fixture = Fixture::new();
    let saved = Gameplay::load(&fixture.content, None)
        .expect("the fixture's default state loads")
        .state();
    let path = fixture.content.join("profiles.json");
    let mut document: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    document["default_state"]["active"] = json!(999_999);
    fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(
        Gameplay::load(&fixture.content, None).is_err(),
        "canary: an invalid default_state must be refused when it is the state"
    );
    let loaded = Gameplay::load(&fixture.content, Some(saved))
        .expect("a valid saved progression must load even when default_state is unused");
    assert!(loaded.state().profiles.is_some());
}

/// L040: the Wilds Condo's starter kit (houses.json `starter_kit`) is free issuance too. The 66/42
/// window advertises 0 for both kit items, and the Chicken the 127/23 grant put in the bag sells
/// for 0, although the synthetic store keeps their reference resale prices (500 and 2).
#[test]
fn the_condo_starter_kit_resells_for_zero() {
    const CHICKEN: i32 = 70_429;
    const KIT_BLOCK: i32 = 32_647;
    let fixture = Fixture::new();
    fs::copy(
        content_dir().join("houses.json"),
        fixture.content.join("houses.json"),
    )
    .unwrap();
    write_json(
        &fixture.content.join("vendors.json"),
        &json!({"schema_version":1,"vendors":[
            {"npc_id":VENDOR,"item_ids":[CHICKEN, KIT_BLOCK, NORMAL_ITEM]}
        ]}),
    );
    write_json(
        &fixture.content.join("stores.json"),
        &json!({"schema_version":1,"marketplace":{
        "enabled_on_handshake":false,"orders_enabled":false,
        "payment_sources_enabled":false,"not_available_message":"disabled"},
        "items":[
            store_row(CHICKEN, 1_000, 500),
            store_row(KIT_BLOCK, 5, 2),
            store_row(NORMAL_ITEM, 15, NORMAL_SELL_PRICE)
        ]}),
    );
    let mut gateway = fixture.connect();
    request(&mut gateway, &[127, 0, 23, 0]);
    let chicken = instance_of(&inventory_of(&gateway), CHICKEN);

    let rows = merchant_rows(&open_shop(&mut gateway));
    assert!(rows.contains(&(CHICKEN, 1_000, 0)), "{rows:?}");
    assert!(rows.contains(&(KIT_BLOCK, 5, 0)), "{rows:?}");
    assert!(
        rows.contains(&(NORMAL_ITEM, 15, NORMAL_SELL_PRICE)),
        "{rows:?}"
    );
    let coins = inventory_of(&gateway).coins;
    let sold = one_completion(&send(&mut gateway, &sell_packet(0, chicken, 1)));
    assert_eq!((sold.result, sold.definition), (RESULT_OK, CHICKEN));
    assert_eq!(
        sold.price, 0,
        "the free Chicken must not pay its store price"
    );
    assert_eq!(inventory_of(&gateway).coins, coins);
    assert_eq!(count_definition(&inventory_of(&gateway), CHICKEN), 0);
}
