//! W148: native vendor SELL (165/5) and its 165/6 completion, at the service layer and routed
//! through the gateway. Synthetic content only; no player data, names or chat.
//!
//! Recovered client facts (see `coin_shop::transaction_complete` and
//! `npc_interaction::check_coin_store`): the completion's first dword is the result (`00c78220`
//! tests it against 1), and the V2 merchant window sells with the null session merchant GUID
//! (`00c77030`). Everything else asserted here is server policy (R).

use osfr_tools::{
    account_store::AccountRole,
    coin_shop::{self, CoinShop, CoinShopOutcome, StoreCatalogue},
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::{Inventory, InventoryItem, InventoryState},
    npc_interaction::{
        MerchantBuyRequestV2, NpcAction, NpcInteraction, NpcInteractionError, NpcSelection,
        decode_merchant_list_66_42, encode_merchant_buy_66_43,
    },
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

// ---------------------------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------------------------

const VENDOR_ID: u64 = 9_500_000_339;
/// Default-state fixtures from `lab/references/rust-gameplay/inventory.json`.
const SELLABLE_INSTANCE: i32 = 14; // definition 75060, sell price 375, not equipped
const EQUIPPED_INSTANCE: i32 = 34; // definition 21683, sell price 5, equipped
const UNCATALOGUED_INSTANCE: i32 = 4; // definition 19522, no store row
const NO_SALE_DEFINITION: i32 = 1180; // store row flagged no_sale
const RESULT_OK: u32 = coin_shop::RESULT_SUCCESS;

fn content_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_COIN_SHOP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gameplay")
        })
}

fn sell_packet(merchant: u64, instance: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 5, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(instance.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

fn buy_packet(merchant: u64, transaction: u32, item: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(merchant.to_le_bytes());
    packet.extend(transaction.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
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

fn merchant_status(packet: &[u8]) -> u32 {
    assert_eq!(packet.len(), 56);
    assert_eq!(&packet[..4], &[66, 0, 44, 0]);
    u32::from_le_bytes(packet[48..52].try_into().unwrap())
}

fn bundle(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for child in children {
        assert!(child.len() < 255);
        bytes.push(child.len() as u8);
        bytes.extend(child);
    }
    bytes
}

/// The 165/6 record as `00c74ef0`/`00c71f30` read it. Field names beyond the result are (R).
#[derive(Debug, PartialEq)]
struct Completion {
    result: u32,
    leading_item: i32,
    kind: u32,
    merchant: u64,
    transaction: u32,
    timestamp: u64,
    definition: i32,
    tint: i32,
    quantity: u32,
    price: u32,
    remaining: u32,
}

fn parse_completion(packet: &[u8]) -> Completion {
    // header 4 + result 4 + item 4 + record (4+8+4+8+4+4+4+4+4) = 56: the client reader
    // overruns on anything shorter, which is how the earlier 52-byte frame failed.
    assert_eq!(packet.len(), 56, "completion must match the reader size");
    assert_eq!(&packet[..4], &[165, 0, 6, 0]);
    let u32_at = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let u64_at = |at: usize| u64::from_le_bytes(packet[at..at + 8].try_into().unwrap());
    Completion {
        result: u32_at(4),
        leading_item: u32_at(8) as i32,
        kind: u32_at(12),
        merchant: u64_at(16),
        transaction: u32_at(24),
        timestamp: u64_at(28),
        definition: u32_at(36) as i32,
        tint: u32_at(40) as i32,
        quantity: u32_at(44),
        price: u32_at(48),
        remaining: u32_at(52),
    }
}

fn completions(replies: &[Vec<u8>]) -> Vec<Completion> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(|packet| parse_completion(packet))
        .collect()
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

/// Instance ids whose stack differs between two states (added, removed or modified).
fn changed_stacks(before: &InventoryState, after: &InventoryState) -> Vec<i32> {
    let ids: BTreeSet<i32> = before
        .items
        .iter()
        .chain(&after.items)
        .map(|item| item.id)
        .collect();
    ids.into_iter()
        .filter(|id| {
            before.items.iter().find(|item| item.id == *id)
                != after.items.iter().find(|item| item.id == *id)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Service layer: NpcInteraction + CoinShop
// ---------------------------------------------------------------------------------------------

struct Counter {
    service: NpcInteraction,
    shop: CoinShop,
    zone: u32,
    position: [f32; 3],
}

impl Counter {
    fn new(coins: u32, tweak: impl FnOnce(&mut InventoryState)) -> Self {
        let directory = inventory_content();
        let mut state = Inventory::load(&directory, None)
            .expect("inventory definitions load")
            .state()
            .clone();
        state.coins = coins;
        tweak(&mut state);
        let inventory = Inventory::load(&directory, Some(state)).expect("test state is valid");
        let catalogue = StoreCatalogue::load(&content_dir().join("stores.json")).unwrap();
        let shop = CoinShop::new(&directory, inventory, catalogue, usize::MAX).unwrap();
        let data = content_dir();
        let mut service =
            NpcInteraction::load(&data.join("npcs"), &data.join("stores.json")).unwrap();
        // (R) Item 8 is an explicit fixture mapping for the vendor's stock.
        service.set_vendor_stock_items(VENDOR_ID, &[8]).unwrap();
        let (zone, position) = service.location(VENDOR_ID).expect("content vendor exists");
        Self {
            service,
            shop,
            zone,
            position,
        }
    }

    fn open(&mut self) {
        let menu = self
            .service
            .interact(VENDOR_ID, self.zone, self.position, None)
            .unwrap();
        let option = menu
            .options
            .iter()
            .find(|option| option.action == NpcAction::Shop)
            .expect("vendor exposes a shop option");
        let mut request = vec![26, 0, 10, 0];
        request.extend(VENDOR_ID.to_le_bytes());
        request.extend(option.id.to_le_bytes());
        let selected = self
            .service
            .select(&request, self.zone, self.position, None)
            .unwrap();
        assert!(matches!(selected, NpcSelection::Shop(_)));
    }

    fn state(&self) -> InventoryState {
        self.shop.inventory().state().clone()
    }

    fn send(&mut self, packet: &[u8]) -> Result<CoinShopOutcome, NpcInteractionError> {
        self.service
            .handle_coin_store(&mut self.shop, packet, self.zone, self.position)
    }

    /// Sends a sell that must be refused: one failure completion carrying `code`, no other
    /// reply, no state change, and the outcome is flagged unchanged.
    fn refused(&mut self, label: &str, packet: &[u8], code: u32) {
        let before = self.state();
        let outcome = self
            .send(packet)
            .unwrap_or_else(|error| panic!("{label}: refusal must be a reply, got {error:?}"));
        assert!(!outcome.changed, "{label}: refusal flagged a change");
        assert_eq!(outcome.result_code, code, "{label}: result code");
        assert_eq!(outcome.replies.len(), 1, "{label}: exactly one reply");
        let completion = parse_completion(&outcome.replies[0]);
        assert_eq!(completion.result, code, "{label}: completion result");
        assert_eq!(completion.kind, 2, "{label}: completion kind is sell");
        assert_eq!(completion.remaining, 0, "{label}: nothing to buy back");
        assert_eq!(self.state(), before, "{label}: state changed");
    }
}

#[test]
fn native_sell_changes_exactly_one_stack_and_the_coin_balance() {
    let mut counter = Counter::new(100, |_| {});
    counter.open();
    // Ordinary native BUY first: one stack, exact price.
    let before_buy = counter.state();
    let bought = counter
        .send(&buy_packet(VENDOR_ID, 90_001, 8, 1))
        .expect("buy is accepted");
    assert_eq!(bought.result_code, RESULT_OK);
    let after_buy = counter.state();
    assert_eq!(after_buy.coins, 100 - 15);
    assert_eq!(changed_stacks(&before_buy, &after_buy).len(), 1);
    let completion = parse_completion(bought.replies.last().unwrap());
    assert_eq!(completion.result, RESULT_OK, "a buy reads as success");
    assert_eq!((completion.kind, completion.definition), (1, 8));

    let instance = after_buy
        .items
        .iter()
        .find(|item| item.definition_id == 8)
        .unwrap()
        .id;
    // Native SELL names no merchant (null session GUID, `00c77030`).
    let sold = counter
        .send(&sell_packet(0, instance, 1))
        .expect("sell is accepted");
    assert!(sold.changed);
    assert_eq!(sold.result_code, RESULT_OK);
    let after_sell = counter.state();
    assert_eq!(after_sell.coins, after_buy.coins + 7);
    assert_eq!(
        count_definition(&after_sell, 8),
        count_definition(&after_buy, 8) - 1
    );
    assert_eq!(changed_stacks(&after_buy, &after_sell), vec![instance]);

    let completion = parse_completion(sold.replies.last().unwrap());
    assert_eq!(completion.result, RESULT_OK, "success must lead with 1");
    assert_eq!(completion.kind, 2);
    assert_eq!(completion.merchant, 0);
    assert_eq!(completion.timestamp, 0);
    assert_eq!(completion.definition, 8);
    assert_eq!(
        (completion.quantity, completion.price, completion.remaining),
        (1, 7, 1)
    );
    // 38/19 coin balance carries the new total.
    let balance = sold
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 19, 0]))
        .expect("coin balance reply");
    assert_eq!(
        i32::from_le_bytes(balance[4..8].try_into().unwrap()),
        after_sell.coins as i32
    );
}

#[test]
fn sale_completion_id_drives_buy_back() {
    let mut counter = Counter::new(100, |_| {});
    counter.open();
    let before = counter.state();
    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    let completion = parse_completion(sold.replies.last().unwrap());
    assert_eq!(completion.result, RESULT_OK);
    assert_eq!(completion.price, 375);
    // The history row id the client sends back is the record's id field.
    let mut buy_back = vec![165, 0, 12, 0];
    buy_back.extend(completion.transaction.to_le_bytes());
    buy_back.extend(0u32.to_le_bytes());
    let restored = counter.shop.handle(&buy_back).unwrap();
    assert_eq!(restored.result_code, RESULT_OK);
    assert_eq!(counter.state().coins, before.coins);
    assert_eq!(
        count_definition(&counter.state(), 75060),
        count_definition(&before, 75060)
    );
}

#[test]
fn the_content_key_is_still_accepted_and_foreign_merchants_are_refused() {
    let mut counter = Counter::new(100, |_| {});
    counter.open();
    let sold = counter
        .send(&sell_packet(VENDOR_ID, SELLABLE_INSTANCE, 1))
        .unwrap();
    assert_eq!(sold.result_code, RESULT_OK);
    assert_eq!(
        parse_completion(sold.replies.last().unwrap()).merchant,
        VENDOR_ID
    );
    // Another vendor's id, a runtime-looking id and an off-by-one are not this shop.
    for merchant in [VENDOR_ID + 1, 1, u64::MAX, VENDOR_ID << 1] {
        counter.refused(
            "foreign merchant",
            &sell_packet(merchant, 2, 1),
            coin_shop::RESULT_INVALID_MERCHANT,
        );
    }
}

#[test]
fn sell_without_an_open_shop_or_out_of_range_is_refused_visibly() {
    // Never opened.
    let mut counter = Counter::new(100, |_| {});
    counter.refused(
        "never opened",
        &sell_packet(0, SELLABLE_INSTANCE, 1),
        coin_shop::RESULT_INVALID_MERCHANT,
    );
    // Opened, then closed by the native 66/46 hide.
    counter.open();
    counter.service.close(VENDOR_ID).unwrap();
    counter.refused(
        "after close",
        &sell_packet(0, SELLABLE_INSTANCE, 1),
        coin_shop::RESULT_INVALID_MERCHANT,
    );
    // Opened, then the player walks away.
    counter.open();
    counter.position = [
        counter.position[0] + 500.0,
        counter.position[1],
        counter.position[2],
    ];
    counter.refused(
        "out of range",
        &sell_packet(0, SELLABLE_INSTANCE, 1),
        coin_shop::RESULT_INVALID_MERCHANT,
    );
    counter.zone += 1;
    counter.refused(
        "other zone",
        &sell_packet(0, SELLABLE_INSTANCE, 1),
        coin_shop::RESULT_INVALID_MERCHANT,
    );
}

#[test]
fn malformed_sells_get_a_failure_reply_and_other_packets_stay_errors() {
    let mut counter = Counter::new(100, |_| {});
    counter.open();
    let good = sell_packet(0, SELLABLE_INSTANCE, 1);
    for cut in 4..good.len() {
        counter.refused("truncated sell", &good[..cut], coin_shop::RESULT_FAILURE);
    }
    let mut long = good.clone();
    long.push(0);
    counter.refused("long sell", &long, coin_shop::RESULT_FAILURE);
    // A header-only or foreign-family packet is not a sell; it keeps the old error.
    assert!(matches!(
        counter.send(&[165, 0, 99, 0]),
        Err(NpcInteractionError::InvalidPacket)
    ));
    assert!(matches!(
        counter.send(&[165, 0]),
        Err(NpcInteractionError::InvalidPacket)
    ));
}

#[test]
fn every_item_refusal_leaves_state_untouched_with_a_canary_that_still_sells() {
    let mut counter = Counter::new(100, |state| {
        // A no-sale item the player owns.
        state.items.push(InventoryItem {
            definition_id: NO_SALE_DEFINITION,
            tint: 0,
            id: 900_001,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        // 3 owned, 1 consumed, 1 stored: only 1 copy is available to sell.
        let stack = state
            .items
            .iter_mut()
            .find(|item| item.id == SELLABLE_INSTANCE)
            .unwrap();
        stack.count = 3;
        stack.consumed_count = 1;
        stack.stored_count = 1;
    });
    counter.open();
    let state = counter.state();
    assert!(
        state
            .equipment
            .iter()
            .any(|entry| entry.item_id == EQUIPPED_INSTANCE),
        "fixture: instance {EQUIPPED_INSTANCE} is equipped"
    );
    assert!(
        state
            .items
            .iter()
            .any(|item| item.id == UNCATALOGUED_INSTANCE),
        "fixture: instance {UNCATALOGUED_INSTANCE} exists"
    );
    let invalid_item = coin_shop::RESULT_INVALID_ITEM;
    let invalid_quantity = coin_shop::RESULT_INVALID_QUANTITY;
    let unavailable = coin_shop::RESULT_SERVICE_UNAVAILABLE;
    let cases: Vec<(&str, i32, u32, u32)> = vec![
        ("instance zero", 0, 1, invalid_item),
        ("negative instance", -5, 1, invalid_item),
        ("unknown instance", 123_456_789, 1, invalid_item),
        ("no store row", UNCATALOGUED_INSTANCE, 1, invalid_item),
        ("no-sale item", 900_001, 1, unavailable),
        ("equipped item", EQUIPPED_INSTANCE, 1, unavailable),
        ("zero quantity", SELLABLE_INSTANCE, 0, invalid_quantity),
        ("past the stack", SELLABLE_INSTANCE, 4, invalid_quantity),
        // Consumed + stored copies are not for sale: 3 owned, only 1 available.
        (
            "into consumed copies",
            SELLABLE_INSTANCE,
            2,
            invalid_quantity,
        ),
        ("u32 max", SELLABLE_INSTANCE, u32::MAX, invalid_quantity),
        (
            "past i32",
            SELLABLE_INSTANCE,
            i32::MAX as u32 + 1,
            invalid_quantity,
        ),
    ];
    for (label, instance, quantity, code) in cases {
        counter.refused(label, &sell_packet(0, instance, quantity), code);
    }
    // Canary: the same instance sells once the request is within what is available, and the
    // stack keeps count >= consumed + stored afterwards.
    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    assert_eq!(sold.result_code, RESULT_OK);
    let stack = counter
        .state()
        .items
        .iter()
        .find(|item| item.id == SELLABLE_INSTANCE)
        .cloned()
        .unwrap();
    assert_eq!(stack.count, 2);
    assert!(stack.count >= stack.consumed_count + stack.stored_count);
    assert_eq!(counter.state().coins, 100 + 375);
}

#[test]
fn coin_cap_blocks_a_sale_without_taking_the_item() {
    let cap = i32::MAX as u32;
    let mut counter = Counter::new(cap - 374, |_| {});
    counter.open();
    counter.refused(
        "over the cap",
        &sell_packet(0, SELLABLE_INSTANCE, 1),
        coin_shop::RESULT_SERVICE_UNAVAILABLE,
    );
    // Exactly reaching the cap is fine.
    let mut counter = Counter::new(cap - 375, |_| {});
    counter.open();
    let sold = counter.send(&sell_packet(0, SELLABLE_INSTANCE, 1)).unwrap();
    assert_eq!(sold.result_code, RESULT_OK);
    assert_eq!(counter.state().coins, cap);
}

#[test]
fn replay_and_reload_cannot_pay_twice() {
    let mut counter = Counter::new(0, |_| {});
    counter.open();
    let packet = sell_packet(0, SELLABLE_INSTANCE, 1);
    let sold = counter.send(&packet).unwrap();
    assert_eq!(sold.result_code, RESULT_OK);
    let after = counter.state();
    assert_eq!(after.coins, 375);
    assert!(after.items.iter().all(|item| item.id != SELLABLE_INSTANCE));
    // The identical packet again finds no such item.
    counter.refused("replay", &packet, coin_shop::RESULT_INVALID_ITEM);
    assert_eq!(counter.state().coins, 375);

    // Save and reload: the state round-trips and a replay against it still pays nothing.
    let saved = serde_json::to_vec(&after).unwrap();
    let restored: InventoryState = serde_json::from_slice(&saved).unwrap();
    assert_eq!(restored, after);
    let mut reloaded = Counter::new(0, |state| *state = restored.clone());
    reloaded.open();
    assert_eq!(reloaded.state(), after);
    reloaded.refused(
        "replay after reload",
        &packet,
        coin_shop::RESULT_INVALID_ITEM,
    );
}

// ---------------------------------------------------------------------------------------------
// Routed through the gateway (the real gameplay.rs 165/5 path)
// ---------------------------------------------------------------------------------------------

const NOW: u64 = 10_000;
const NPC: u64 = 700;
const STACK_MANY: i32 = 900_001; // definition 75060 x3, sell 375 each
const STACK_NO_SALE: i32 = 900_003; // definition 1180
const STACK_ONE: i32 = 900_004; // definition 8 x1, sell 7
static NEXT: AtomicU64 = AtomicU64::new(0);

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w148-synthetic-ticket", "1.910.1.530630", ""]
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

fn npc_request(sub: u16, choice: Option<u32>) -> Vec<u8> {
    let mut bytes = [26u16.to_le_bytes(), sub.to_le_bytes()].concat();
    bytes.extend(NPC.to_le_bytes());
    match choice {
        Some(choice) => bytes.extend(choice.to_le_bytes()),
        None => bytes.push(0),
    }
    bytes
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
    /// An item instance that is equipped in the prepared default state.
    equipped: i32,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.content);
    }
}

impl Fixture {
    fn new() -> Self {
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("reports/round59-native-20261015T124443Z/content"));
        let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("references/rust-gateway-reference"));
        assert!(
            source.join("inventory.json").is_file(),
            "prepared content fixture missing: {}",
            source.display()
        );
        let content = lab.join("build/w148-fixtures").join(format!(
            "vendor-sell-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&content).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "bin")
                )
            {
                fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
            }
        }
        fs::create_dir(content.join("npcs")).unwrap();
        write_json(
            &content.join("npcs/synthetic.json"),
            &json!({"schema_version":1,"records":[
                {"npc_id":NPC,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
                 "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic"}
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
            &json!({"schema_version":1,"vendors":[{"npc_id":NPC,"item_ids":[8]}]}),
        );
        // The quests/jobs files the gameplay loader expects, kept empty of this test's concerns.
        fs::create_dir(content.join("quests")).unwrap();
        let provenance = json!([{"kind":"reconstructed","evidence":"W148 synthetic test policy"}]);
        write_json(
            &content.join("quests/synthetic.json"),
            &json!({"schema_version":1,"records":[{
                "id":701,"title_string_id":1,"giver_npc_id":NPC,"prerequisite_quest_ids":[],
                "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":NPC,"source":provenance}],
                "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
            }]}),
        );
        write_json(
            &content.join("jobs.json"),
            &json!({"schema_version":1,"records":[{
                "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
                "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[701],"abilities":[],"source":provenance
            }]}),
        );
        // Pick an equipped item from the state the gameplay loader actually produces.
        let loaded = Gameplay::load(&content, None).unwrap().state().inventory;
        let (equipped, equipped_definition) = loaded
            .equipment
            .iter()
            .filter(|entry| entry.item_id != 0)
            .find_map(|entry| {
                let item = loaded.items.iter().find(|item| item.id == entry.item_id)?;
                Some((item.id, item.definition_id))
            })
            .expect("the prepared state has an equipped item");
        let row = |id: i32, buy: Option<u32>, sell: Option<u32>, no_sale: bool| {
            json!({"item_id":id,"purchase_price_coins":buy,"member_discount_percent":null,
                   "member_price_coins":null,"sell_price_coins":sell,"no_sale":no_sale})
        };
        write_json(
            &content.join("stores.json"),
            &json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[
                row(8, Some(15), Some(7), false),
                row(75060, Some(750), Some(375), false),
                row(equipped_definition, Some(10), Some(5), false),
                row(NO_SALE_DEFINITION, Some(50), None, true),
            ]}),
        );
        Self {
            content,
            reference,
            equipped,
        }
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None).unwrap().state();
        let equipped: BTreeSet<_> = state
            .inventory
            .equipment
            .iter()
            .map(|entry| entry.item_id)
            .collect();
        assert!(equipped.contains(&self.equipped));
        state
            .inventory
            .items
            .retain(|item| equipped.contains(&item.id));
        state.inventory.coins = 100;
        for (id, definition_id, count) in [
            (STACK_MANY, 75060, 3),
            (STACK_NO_SALE, NO_SALE_DEFINITION, 1),
            (STACK_ONE, 8, 1),
        ] {
            state.inventory.items.push(InventoryItem {
                id,
                definition_id,
                tint: 0,
                count,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        }
        state.inventory.next_instance_id = Some(900_005);
        state
    }

    fn connect(&self, state: GameplayState) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state)).unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway.handle_at(&packet, NOW).unwrap();
        }
        gateway
    }

    fn at_open_shop_with(&self, state: GameplayState) -> (Gateway, Vec<Vec<u8>>) {
        let mut gateway = self.connect(state);
        gateway
            .handle_at(&wrap(&npc_request(8, None)), NOW)
            .unwrap();
        let opened = gateway
            .handle_at(&wrap(&npc_request(10, Some(2))), NOW)
            .unwrap();
        let inner = inner_packets(&opened);
        assert!(
            inner.iter().any(|p| p.starts_with(&[66, 0, 42, 0])),
            "the shop did not open"
        );
        (gateway, inner)
    }

    /// A fresh player standing at the vendor with the shop window open.
    fn at_open_shop(&self) -> (Gateway, Vec<Vec<u8>>) {
        self.at_open_shop_with(self.state())
    }
}

/// Tunnel payloads of every reply, with the 7-byte application wrapper removed.
fn inner_packets(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome.replies.iter().map(|p| p[7..].to_vec()).collect()
}

fn inventory_of(gateway: &Gateway) -> InventoryState {
    gateway.snapshot().gameplay.unwrap().inventory
}

fn send(gateway: &mut Gateway, packet: &[u8]) -> Vec<Vec<u8>> {
    inner_packets(&gateway.handle_at(&wrap(packet), NOW).unwrap())
}

#[test]
fn gateway_shop_list_is_the_client_readable_66_42() {
    let fixture = Fixture::new();
    let (_, inner) = fixture.at_open_shop();
    let list = inner
        .iter()
        .find(|p| p.starts_with(&[66, 0, 42, 0]))
        .unwrap();
    let decoded = decode_merchant_list_66_42(list).expect("client layout");
    assert_eq!(decoded.group.entries.len(), 1);
    assert_eq!(decoded.group.entries[0].first_six_dwords[0], 8);
}

#[test]
fn gateway_native_sell_pays_once_saves_and_survives_a_reconnect() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = inventory_of(&gateway);

    let replies = send(&mut gateway, &sell_packet(0, STACK_MANY, 2));
    let done = completions(&replies);
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].result, RESULT_OK);
    assert_eq!((done[0].quantity, done[0].price), (2, 375));
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins + 750);
    assert_eq!(changed_stacks(&before, &after), vec![STACK_MANY]);
    assert_eq!(
        after
            .items
            .iter()
            .find(|i| i.id == STACK_MANY)
            .unwrap()
            .count,
        1
    );

    // Disconnect and reload from the saved state: both the stack and the coins persist.
    let saved = gateway.snapshot().gameplay.unwrap();
    let mut again = fixture.connect(saved);
    assert_eq!(inventory_of(&again), after);
    // A reconnected player has no open shop, so the same sell is refused and pays nothing.
    let replies = send(&mut again, &sell_packet(0, STACK_MANY, 1));
    let done = completions(&replies);
    assert_eq!(done.len(), 1);
    assert_ne!(done[0].result, RESULT_OK);
    assert_eq!(inventory_of(&again), after);
}

#[test]
fn gateway_refusals_answer_the_client_and_change_nothing() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = gateway.snapshot();
    let table: Vec<(&str, Vec<u8>, u32)> = vec![
        (
            "foreign merchant",
            sell_packet(NPC + 1, STACK_MANY, 1),
            coin_shop::RESULT_INVALID_MERCHANT,
        ),
        (
            "unknown instance",
            sell_packet(0, 777_777, 1),
            coin_shop::RESULT_INVALID_ITEM,
        ),
        (
            "too many",
            sell_packet(0, STACK_MANY, 4),
            coin_shop::RESULT_INVALID_QUANTITY,
        ),
        (
            "zero",
            sell_packet(0, STACK_MANY, 0),
            coin_shop::RESULT_INVALID_QUANTITY,
        ),
        (
            "no sale",
            sell_packet(0, STACK_NO_SALE, 1),
            coin_shop::RESULT_SERVICE_UNAVAILABLE,
        ),
        (
            "equipped",
            sell_packet(0, fixture.equipped, 1),
            coin_shop::RESULT_SERVICE_UNAVAILABLE,
        ),
        (
            "truncated",
            sell_packet(0, STACK_MANY, 1)[..14].to_vec(),
            coin_shop::RESULT_FAILURE,
        ),
    ];
    for (label, packet, code) in table {
        let replies = send(&mut gateway, &packet);
        let done = completions(&replies);
        assert_eq!(done.len(), 1, "{label}: one completion");
        assert_eq!(done[0].result, code, "{label}: result");
        assert_eq!(done[0].kind, 2, "{label}: sell kind");
        assert_eq!(gateway.snapshot(), before, "{label}: gateway state changed");
    }
    // Canary: the shop is still usable after every refusal.
    let replies = send(&mut gateway, &sell_packet(0, STACK_ONE, 1));
    assert_eq!(completions(&replies)[0].result, RESULT_OK);
    assert_eq!(inventory_of(&gateway).coins, 100 + 7);
}

#[test]
fn gateway_sell_replay_of_a_whole_stack_pays_once() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let packet = sell_packet(0, STACK_ONE, 1);
    assert_eq!(
        completions(&send(&mut gateway, &packet))[0].result,
        RESULT_OK
    );
    let paid = inventory_of(&gateway);
    assert_eq!(paid.coins, 107);
    for _ in 0..3 {
        let done = completions(&send(&mut gateway, &packet));
        assert_eq!(done[0].result, coin_shop::RESULT_INVALID_ITEM);
        assert_eq!(inventory_of(&gateway), paid);
    }
}

#[test]
fn gateway_buy_then_sell_round_trip_from_the_native_window() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = inventory_of(&gateway);
    // Ordinary BUY through the same window (165/4 names the shop by its content key).
    let replies = send(&mut gateway, &buy_packet(NPC, 31_001, 8, 1));
    assert_eq!(completions(&replies)[0].result, RESULT_OK);
    let bought = inventory_of(&gateway);
    assert_eq!(bought.coins, before.coins - 15);
    assert_eq!(
        count_definition(&bought, 8),
        count_definition(&before, 8) + 1
    );
    // Sell what was bought: the new stack, not the one the fixture already owned.
    let bought_stack = bought
        .items
        .iter()
        .find(|item| item.definition_id == 8 && item.id != STACK_ONE)
        .expect("the purchase created a stack")
        .id;
    let replies = send(&mut gateway, &sell_packet(0, bought_stack, 1));
    assert_eq!(completions(&replies)[0].result, RESULT_OK);
    let sold = inventory_of(&gateway);
    assert_eq!(sold.coins, bought.coins + 7);
    assert_eq!(count_definition(&sold, 8), count_definition(&before, 8));
}

#[test]
fn gateway_vendor_buy_debits_grants_and_replies() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = inventory_of(&gateway);

    let replies = send(&mut gateway, &vendor_buy_packet(91_001, 8, 1));
    let status = merchant_status(replies.last().expect("vendor buy reply"));
    assert_eq!(status, 1);
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins - 15);
    assert_eq!(
        count_definition(&after, 8),
        count_definition(&before, 8) + 1
    );
}

#[test]
fn gateway_vendor_buy_insufficient_coins_replies_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut state = fixture.state();
    state.inventory.coins = 5;
    let (mut gateway, _) = fixture.at_open_shop_with(state);
    let before = gateway.snapshot();

    let replies = send(&mut gateway, &vendor_buy_packet(91_002, 8, 1));
    assert_eq!(replies.len(), 1, "refusal has one client-visible reply");
    assert_eq!(merchant_status(&replies[0]), 0);
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn gateway_vendor_buy_full_bag_replies_and_changes_nothing() {
    let fixture = Fixture::new();
    let mut state = fixture.state();
    state.inventory.items.clear();
    for entry in &mut state.inventory.equipment {
        entry.item_id = 0;
    }
    for id in 100_000..100_100 {
        state.inventory.items.push(InventoryItem {
            definition_id: 75060,
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
    state.inventory.next_instance_id = Some(100_100);
    let (mut gateway, _) = fixture.at_open_shop_with(state);
    let before = gateway.snapshot();

    let replies = send(&mut gateway, &vendor_buy_packet(91_003, 8, 1));
    assert_eq!(replies.len(), 1, "full bag has one client-visible reply");
    assert_eq!(merchant_status(&replies[0]), 0);
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn gateway_vendor_buy_refusals_always_answer_with_66_44() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = gateway.snapshot();
    let cases = [
        ("malformed", vec![66, 0, 43, 0]),
        ("unknown item", vendor_buy_packet(91_010, i32::MAX, 1)),
        ("zero quantity", vendor_buy_packet(91_011, 8, 0)),
        ("unbounded quantity", vendor_buy_packet(91_012, 8, u32::MAX)),
    ];
    for (label, request) in cases {
        let replies = send(&mut gateway, &request);
        assert_eq!(replies.len(), 1, "{label}: one client-visible reply");
        assert_eq!(merchant_status(&replies[0]), 0, "{label}: refused");
        assert_eq!(gateway.snapshot(), before, "{label}: state changed");
    }

    let mut closed = fixture.connect(fixture.state());
    let before = closed.snapshot();
    let replies = send(&mut closed, &vendor_buy_packet(91_013, 8, 1));
    assert_eq!(replies.len(), 1, "closed shop: one client-visible reply");
    assert_eq!(merchant_status(&replies[0]), 0, "closed shop: refused");
    assert_eq!(closed.snapshot(), before, "closed shop: state changed");
}

#[test]
fn gateway_vendor_buy_reads_the_live_inventory_after_a_reward() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let mut live = inventory_of(&gateway);
    live.coins += 100;
    live.items.push(InventoryItem {
        definition_id: 75060,
        tint: 0,
        id: 900_010,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    live.next_instance_id = Some(900_011);
    gateway.adopt_trade_inventory(live).unwrap();
    let before = inventory_of(&gateway);

    let replies = send(&mut gateway, &vendor_buy_packet(91_004, 8, 1));
    assert_eq!(
        merchant_status(replies.last().expect("vendor buy reply")),
        1
    );
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins - 15);
    assert_eq!(
        count_definition(&after, 75060),
        count_definition(&before, 75060)
    );
}

#[test]
fn gateway_bundle_failure_does_not_commit_a_vendor_buy() {
    let fixture = Fixture::new();
    let (mut gateway, _) = fixture.at_open_shop();
    let before = gateway.snapshot();
    let request = vendor_buy_packet(91_005, 8, 1);

    let failed = gateway.handle_at(&bundle(&[request.clone(), vec![26, 0, 8, 0]]), NOW);
    assert!(
        failed.is_err(),
        "the malformed child must reject the bundle"
    );
    assert_eq!(gateway.snapshot(), before, "failed bundle changed gameplay");

    let replies = send(&mut gateway, &request);
    assert_eq!(
        merchant_status(replies.last().expect("vendor buy reply")),
        1
    );
    assert_eq!(
        inventory_of(&gateway).coins,
        before.gameplay.unwrap().inventory.coins - 15
    );
}
