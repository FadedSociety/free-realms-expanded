//! T374: the vendor round trip through the real gateway. A player opens the shipped
//! representative vendor, buys one stock row (66/43), the canonical save holds the new
//! wallet/bag, a relog reads it back, and the same item sells once through 165/5 — with every
//! refusal asserted to change neither `Gateway::snapshot()` nor the serialized save.
//!
//! Wire bodies are the shipped client's, re-checked in IDA:
//! - 26/8 interact constructor `0x008b6bc0` (u64 target + bool);
//! - 26/10 select constructor `0x00a36f80` (u64 target + i32 choice);
//! - 66/42 list: dispatcher `0x00b48a10` case 0x2a -> `0x00b40a60` -> `0x00b3d5f0`; the row
//!   reader `0x00b29cd0` reads six dwords, a bool, three dwords and a bool;
//! - 66/43 buy serializer `0x00b3ab20`; its request constructor is `0x00b43310`;
//! - 66/44 result reader `0x00b32430`; `0x00b48a10` tests field `+0x58` (wire offset 48)
//!   against 1;
//! - 165/5 sell writer `0x00c75a70`, fields set by `0x00c76dd0`, Lua entry `0x00c77030`
//!   (which passes the null merchant GUIDs `0x01beea08`);
//! - 165/6 completion reader `0x00c74ef0` -> `0x00c71f30`; `0x00c78220` compares the first
//!   record dword (wire offset 4) with 1.
//!
//! No new wire shape is introduced: every request comes from the shared
//! `tests/common/gateway_vendor.rs` helpers, priced stock comes from the shared
//! `tests/common/vendor_fixture.rs`, and the save/relog is the real `game_store` envelope plus
//! the shared `connect_with` helper.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
// The shared purchase module carries helpers and a test this suite reuses rather than copies;
// the few functions it does not call stay unused here.
#[path = "common/vendor_purchase.rs"]
#[allow(dead_code)]
mod vendor_purchase;

use gateway_vendor::{
    NOW, count_definition, interact, inventory_of, merchant_status, select, sell_packet, send,
    vendor_buy_packet, wrap,
};
use osfr_tools::{
    account_store::AccountRole,
    coin_shop,
    game_store::{self, Snapshot},
    gameplay::GameplayState,
    gateway::Gateway,
    inventory::InventoryState,
    npc_interaction::{
        MerchantBuyRequestV2, decode_merchant_list_66_42, encode_merchant_buy_66_43,
    },
    world::NpcVisibility,
};
use std::{fs, path::PathBuf};
use vendor_purchase::{
    REPRESENTATIVE_ITEM, REPRESENTATIVE_VENDOR, SEEDED_ITEM, VendorFixture, connect_with,
    fixture_root,
};

const RESULT_SUCCESS: u32 = coin_shop::RESULT_SUCCESS;

fn connect(fixture: &VendorFixture, state: GameplayState) -> Gateway {
    connect_with(fixture, fixture.character(), state)
}

/// Open the representative vendor's 66/42 window and return the list body.
fn open_shop(fixture: &VendorFixture, gateway: &mut Gateway) -> Vec<u8> {
    send(gateway, &interact(REPRESENTATIVE_VENDOR));
    let replies = send(gateway, &select(REPRESENTATIVE_VENDOR, fixture.option()));
    replies
        .into_iter()
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("the representative vendor opens its 66/42 shop")
}

fn buy(gateway: &mut Gateway, transaction: u32, item: i32, quantity: u32) -> Vec<Vec<u8>> {
    send(gateway, &vendor_buy_packet(transaction, item, quantity))
}

/// 165/6 record sizes and order from `0x00c74ef0` -> `0x00c71f30`: header, result, leading
/// item, then kind, u64 merchant, transaction, u64 timestamp, item pair, quantity, unit price
/// and remaining.
#[derive(Debug, PartialEq)]
struct SellCompletion {
    result: u32,
    kind: u32,
    merchant: u64,
    transaction: u32,
    definition: i32,
    quantity: u32,
    price: u32,
    remaining: u32,
}

fn sell_completion(packet: &[u8]) -> SellCompletion {
    assert_eq!(
        packet.len(),
        56,
        "165/6 must match the 0x00c74ef0/0x00c71f30 reader size"
    );
    assert_eq!(&packet[..4], &[165, 0, 6, 0]);
    let u32_at = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let u64_at = |at: usize| u64::from_le_bytes(packet[at..at + 8].try_into().unwrap());
    SellCompletion {
        result: u32_at(4),
        kind: u32_at(12),
        merchant: u64_at(16),
        transaction: u32_at(24),
        definition: u32_at(36) as i32,
        quantity: u32_at(44),
        price: u32_at(48),
        remaining: u32_at(52),
    }
}

fn completions(replies: &[Vec<u8>]) -> Vec<SellCompletion> {
    replies
        .iter()
        .filter(|packet| packet.starts_with(&[165, 0, 6, 0]))
        .map(|packet| sell_completion(packet))
        .collect()
}

fn buy_status(replies: &[Vec<u8>]) -> u32 {
    merchant_status(replies.last().expect("a 66/43 buy always answers 66/44"))
}

/// The one stack the purchase granted (the fixture seeds only `SEEDED_ITEM`).
fn bought_instance(state: &InventoryState) -> (i32, u32) {
    let mut matches = state
        .items
        .iter()
        .filter(|item| item.definition_id == REPRESENTATIVE_ITEM)
        .map(|item| (item.id, item.count));
    let first = matches.next().expect("the purchase granted the item");
    assert!(
        matches.next().is_none(),
        "the purchase granted more than one stack"
    );
    first
}

fn save_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("osfr-t374-{}-{label}.json", std::process::id()))
}

/// Serialize the whole gateway snapshot through the real save envelope and return its bytes.
fn saved_bytes(label: &str, snapshot: &Snapshot) -> Vec<u8> {
    let path = save_path(label);
    game_store::save(&path, snapshot).expect("the canonical game save writes");
    fs::read(&path).expect("the canonical game save reads")
}

fn read_save(label: &str, snapshot: &Snapshot) -> Snapshot {
    let path = save_path(label);
    game_store::save(&path, snapshot).expect("the canonical game save writes");
    game_store::load_or_create(&path, snapshot.character.clone()).expect("the canonical save loads")
}

/// A refused buy answers exactly one 66/44 status 0 and changes neither the snapshot nor the
/// save bytes.
fn refused_buy(gateway: &mut Gateway, label: &str, packet: &[u8]) {
    let before = gateway.snapshot();
    let before_save = saved_bytes(&format!("{label}-before"), &before);
    let replies = send(gateway, packet);
    assert_eq!(
        replies.len(),
        1,
        "{label}: a refused buy answers exactly one 66/44"
    );
    assert_eq!(merchant_status(&replies[0]), 0, "{label}: not refused");
    assert_eq!(
        gateway.snapshot(),
        before,
        "{label}: the refused buy changed the snapshot"
    );
    assert_eq!(
        saved_bytes(&format!("{label}-after"), &gateway.snapshot()),
        before_save,
        "{label}: the refused buy changed the save bytes"
    );
}

/// A refused sell answers one 165/6 with the production result code and changes nothing.
fn refused_sell(gateway: &mut Gateway, label: &str, packet: &[u8], code: u32) {
    let before = gateway.snapshot();
    let before_save = saved_bytes(&format!("{label}-before"), &before);
    let replies = send(gateway, packet);
    let done = completions(&replies);
    assert_eq!(done.len(), 1, "{label}: exactly one 165/6 completion");
    assert_eq!(done[0].result, code, "{label}: result code");
    assert_eq!(done[0].kind, 2, "{label}: completion kind is sell");
    assert_eq!(done[0].remaining, 0, "{label}: nothing to buy back");
    assert_eq!(
        gateway.snapshot(),
        before,
        "{label}: the refused sell changed the snapshot"
    );
    assert_eq!(
        saved_bytes(&format!("{label}-after"), &gateway.snapshot()),
        before_save,
        "{label}: the refused sell changed the save bytes"
    );
}

#[test]
fn a_player_buys_a_stock_item_and_the_save_holds_the_new_wallet() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-buy");
    let price = fixture.price();
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));

    let list = decode_merchant_list_66_42(&open_shop(&fixture, &mut gateway))
        .expect("the 66/42 window matches the client row reader");
    let entry = list
        .group
        .entries
        .iter()
        .find(|entry| entry.first_six_dwords[0] == REPRESENTATIVE_ITEM as u32)
        .expect("the representative stock row is in the window");
    // (R) Local convention for the opaque row: dword 1 is the purchase price; the server value
    // from stores.json, never a client field, decides the debit.
    assert_eq!(
        entry.first_six_dwords[1], price,
        "the window does not quote the stores.json price"
    );

    let before = gateway.snapshot();
    let before_inventory = inventory_of(&gateway);
    assert_eq!(
        count_definition(&before_inventory, REPRESENTATIVE_ITEM),
        0,
        "fixture: the bought item is not owned yet"
    );
    assert_eq!(count_definition(&before_inventory, SEEDED_ITEM), 1);

    let replies = buy(&mut gateway, 101, REPRESENTATIVE_ITEM, 1);
    assert_eq!(buy_status(&replies), 1, "the buy was refused");
    let after = gateway.snapshot();
    let after_inventory = inventory_of(&gateway);
    assert_eq!(
        after_inventory.coins,
        before_inventory.coins - price,
        "the debit is not the stores.json purchase price"
    );
    assert_eq!(
        count_definition(&after_inventory, REPRESENTATIVE_ITEM),
        1,
        "the purchase did not grant exactly one item"
    );
    assert_eq!(
        count_definition(&after_inventory, SEEDED_ITEM),
        1,
        "the pre-existing stack did not survive the purchase"
    );
    assert_eq!(
        after.character, before.character,
        "a vendor purchase moved the character"
    );
    let (instance, count) = bought_instance(&after_inventory);
    assert_eq!(count, 1);
    assert_ne!(instance, 1, "the purchase aliased the seeded instance");
    let mut ids: Vec<i32> = after_inventory.items.iter().map(|item| item.id).collect();
    ids.sort_unstable();
    let mut unique = ids.clone();
    unique.dedup();
    assert_eq!(
        unique.len(),
        ids.len(),
        "inventory instance ids are not unique"
    );

    // The authored stock is not decremented by a purchase: reopening the window lists the same
    // vendor state (there is no per-row stock counter in the production path).
    let relisted = decode_merchant_list_66_42(&open_shop(&fixture, &mut gateway))
        .expect("the reopened 66/42 window matches the client reader");
    assert_eq!(relisted, list, "the vendor stock changed after a purchase");

    // The canonical save holds the same wallet and bag.
    let loaded = read_save("buy", &after);
    assert_eq!(loaded, after, "the save envelope lost the purchase");
    let saved_inventory = loaded
        .gameplay
        .as_ref()
        .expect("the save carries gameplay state")
        .inventory
        .clone();
    assert_eq!(saved_inventory.coins, after_inventory.coins);
    assert_eq!(
        count_definition(&saved_inventory, REPRESENTATIVE_ITEM),
        count_definition(&after_inventory, REPRESENTATIVE_ITEM)
    );
}

#[test]
fn the_purchase_survives_a_real_relog() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-relog");
    let price = fixture.price();
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    open_shop(&fixture, &mut gateway);
    assert_eq!(
        buy_status(&buy(&mut gateway, 201, REPRESENTATIVE_ITEM, 1)),
        1
    );

    let committed = gateway.snapshot();
    let loaded = read_save("relog", &committed);
    assert_eq!(loaded, committed, "the save envelope lost the purchase");
    let saved_inventory = loaded
        .gameplay
        .as_ref()
        .expect("the save carries gameplay state")
        .inventory
        .clone();
    assert_eq!(saved_inventory.coins, 1_000);
    assert_eq!(count_definition(&saved_inventory, REPRESENTATIVE_ITEM), 1);

    let mut relogged = connect_with(
        &fixture,
        loaded.character.clone(),
        loaded.gameplay.clone().expect("the save carries gameplay"),
    );
    assert_eq!(
        inventory_of(&relogged),
        saved_inventory,
        "the relog did not read the saved wallet and bag"
    );

    // An open shop is session state: the relog starts with none, and the buy is refused without
    // changing the restored character.
    let closed = relogged.snapshot();
    let replies = buy(&mut relogged, 202, REPRESENTATIVE_ITEM, 1);
    assert_eq!(replies.len(), 1, "closed shop answers one 66/44");
    assert_eq!(
        merchant_status(&replies[0]),
        0,
        "a relog kept the shop open"
    );
    assert_eq!(
        relogged.snapshot(),
        closed,
        "the closed-shop refusal changed state"
    );

    // A fresh 26/8 -> 26/10 window spends the saved wallet, not the fixture's entry value.
    open_shop(&fixture, &mut relogged);
    assert_eq!(
        buy_status(&buy(&mut relogged, 203, REPRESENTATIVE_ITEM, 1)),
        1,
        "a second buy after relog was refused"
    );
    assert_eq!(inventory_of(&relogged).coins, 1_000 - price);
    assert_eq!(
        count_definition(&inventory_of(&relogged), REPRESENTATIVE_ITEM),
        2,
        "the relog double-granted or lost the first item"
    );

    // The live balance (not the fixture entry balance) refuses the next buy.
    refused_buy(
        &mut relogged,
        "relog-poor",
        &vendor_buy_packet(204, REPRESENTATIVE_ITEM, 1),
    );
    assert_eq!(inventory_of(&relogged).coins, 1_000 - price);
    assert_eq!(
        count_definition(&inventory_of(&relogged), REPRESENTATIVE_ITEM),
        2
    );
}

#[test]
fn the_same_item_can_be_sold_once_through_165_5() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-sell");
    let price = fixture.price();
    // F228 zeroes only free job-starter outfit rows; this stock row resells at its store value.
    let resale = fixture.sell_price_of(REPRESENTATIVE_ITEM);
    assert!(resale > 0);
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    open_shop(&fixture, &mut gateway);
    assert_eq!(
        buy_status(&buy(&mut gateway, 301, REPRESENTATIVE_ITEM, 1)),
        1
    );

    let before = gateway.snapshot();
    let before_inventory = inventory_of(&gateway);
    let (instance, count) = bought_instance(&before_inventory);
    assert_eq!(count, 1);

    let replies = send(&mut gateway, &sell_packet(0, instance, 1));
    let done = completions(&replies);
    assert_eq!(done.len(), 1, "one 165/6 completion");
    assert_eq!(done[0].result, RESULT_SUCCESS);
    assert_eq!(done[0].kind, 2);
    assert_eq!(done[0].merchant, 0, "the native sell names the open shop");
    assert_eq!(done[0].definition, REPRESENTATIVE_ITEM);
    assert_eq!((done[0].quantity, done[0].price), (1, resale));
    assert_ne!(
        done[0].transaction, 0,
        "the sale did not allocate a ledger transaction"
    );
    let after_sell = gateway.snapshot();
    let sold = inventory_of(&gateway);
    assert_eq!(
        sold.coins,
        before_inventory.coins + resale,
        "the sale did not pay the stores.json resale price"
    );
    assert_eq!(
        count_definition(&sold, REPRESENTATIVE_ITEM),
        0,
        "the sold item is still held"
    );
    assert!(
        sold.items.iter().all(|item| item.id != instance),
        "the sold instance is still in the bag"
    );
    assert!(
        sold.items
            .iter()
            .all(|item| item.count > 0 && item.count >= item.consumed_count + item.stored_count),
        "the sale left an invalid stack count"
    );
    assert_eq!(after_sell.character, before.character);

    // A replay finds no such owned instance: it pays nothing and changes neither state nor save.
    refused_sell(
        &mut gateway,
        "sell-replay",
        &sell_packet(0, instance, 1),
        coin_shop::RESULT_INVALID_ITEM,
    );

    // Interleaving: the second purchase and sale read the live wallet and a new instance, never
    // the fixture's entry snapshot.
    let live = inventory_of(&gateway);
    assert_eq!(
        buy_status(&buy(&mut gateway, 302, REPRESENTATIVE_ITEM, 1)),
        1
    );
    let (second, second_count) = bought_instance(&inventory_of(&gateway));
    assert_eq!(second_count, 1);
    assert_ne!(
        second, instance,
        "the second purchase reused the sold instance id"
    );
    let replies = send(&mut gateway, &sell_packet(0, second, 1));
    assert_eq!(completions(&replies)[0].result, RESULT_SUCCESS);
    let final_inventory = inventory_of(&gateway);
    assert_eq!(
        final_inventory.coins,
        live.coins - price + resale,
        "the live buy/sell interleaving did not apply deltas"
    );
    assert_eq!(
        count_definition(&final_inventory, REPRESENTATIVE_ITEM),
        0,
        "the second item was not sold"
    );

    // The vendor's authored stock is unaffected by either sale.
    let relisted = decode_merchant_list_66_42(&open_shop(&fixture, &mut gateway))
        .expect("the reopened 66/42 window matches the client reader");
    assert!(
        relisted
            .group
            .entries
            .iter()
            .any(|entry| entry.first_six_dwords[0] == REPRESENTATIVE_ITEM as u32),
        "a sale removed the vendor's authored stock row"
    );
}

#[test]
fn buy_and_sell_refusals_change_nothing() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-refusals");
    let price = fixture.price();

    // Not enough coins: a distinct wallet, refused, then the shop tops itself up through a real
    // sale and the same buy lands.
    let mut poor = connect(&fixture, fixture.state(price - 1, false));
    open_shop(&fixture, &mut poor);
    refused_buy(
        &mut poor,
        "poor",
        &vendor_buy_packet(401, REPRESENTATIVE_ITEM, 1),
    );
    let sold = completions(&send(&mut poor, &sell_packet(0, 1, 1)));
    assert_eq!(
        sold[0].result, RESULT_SUCCESS,
        "the poor refusal closed the shop"
    );
    assert_eq!(inventory_of(&poor).coins, price - 1 + fixture.sell_price());
    assert_eq!(
        buy_status(&buy(&mut poor, 402, REPRESENTATIVE_ITEM, 1)),
        1,
        "the same open shop did not serve the topped-up buy"
    );
    assert_eq!(
        inventory_of(&poor).coins,
        price - 1 + fixture.sell_price() - price
    );

    // A full bag refuses the purchase whole; freeing one slot through the real 165/5 route lets
    // the same open shop buy.
    let mut full = connect(&fixture, fixture.state(price + 1_000, true));
    open_shop(&fixture, &mut full);
    assert_eq!(
        inventory_of(&full)
            .items
            .iter()
            .filter(|item| item.count > 0)
            .count(),
        100
    );
    refused_buy(
        &mut full,
        "full-bag",
        &vendor_buy_packet(403, REPRESENTATIVE_ITEM, 1),
    );
    let freed = completions(&send(&mut full, &sell_packet(0, 100_000, 1)));
    assert_eq!(
        freed[0].result, RESULT_SUCCESS,
        "the full-bag refusal closed the shop"
    );
    assert_eq!(
        buy_status(&buy(&mut full, 404, REPRESENTATIVE_ITEM, 1)),
        1,
        "the full-bag retry was refused"
    );
    let filled = inventory_of(&full);
    assert_eq!(filled.items.len(), 100);
    assert_eq!(count_definition(&filled, REPRESENTATIVE_ITEM), 1);
    assert_eq!(filled.coins, price + 1_000 + fixture.sell_price() - price);

    // One well-funded, open shop serves every remaining refusal; the specific 165/6 codes prove
    // the coin-store admission checks ran, and the final sale proves the shop stayed usable.
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    open_shop(&fixture, &mut gateway);
    refused_buy(
        &mut gateway,
        "unknown-item",
        &vendor_buy_packet(405, i32::MAX, 1),
    );
    refused_buy(
        &mut gateway,
        "unbounded-quantity",
        // The authored stock has no per-row count; the server's quantity bound is the real gate.
        &vendor_buy_packet(406, REPRESENTATIVE_ITEM, u32::MAX),
    );
    refused_sell(
        &mut gateway,
        "foreign-merchant",
        &sell_packet(REPRESENTATIVE_VENDOR + 1, 1, 1),
        coin_shop::RESULT_INVALID_MERCHANT,
    );
    refused_sell(
        &mut gateway,
        "unowned-instance",
        &sell_packet(0, 999_999, 1),
        coin_shop::RESULT_INVALID_ITEM,
    );
    refused_sell(
        &mut gateway,
        "over-owned-quantity",
        &sell_packet(0, 1, 2),
        coin_shop::RESULT_INVALID_QUANTITY,
    );
    let sold = completions(&send(&mut gateway, &sell_packet(0, 1, 1)));
    assert_eq!(
        sold[0].result, RESULT_SUCCESS,
        "a refusal consumed the open shop"
    );
    assert_eq!(sold[0].price, fixture.sell_price());
    assert_eq!(
        inventory_of(&gateway).coins,
        price + 1_000 + fixture.sell_price()
    );

    // Out of range: the entry path is gated before a shop exists. A single (non-bundled) request
    // whose gameplay handler refuses is dropped by the gateway with no reply, and it must change
    // nothing.
    let mut far = fixture.character();
    far.position = [1_000.0, 0.0, 0.0];
    let mut far_gateway = connect_with(&fixture, far, fixture.state(price + 1_000, false));
    let far_before = far_gateway.snapshot();
    let refused = far_gateway
        .handle_at(&wrap(&interact(REPRESENTATIVE_VENDOR)), NOW)
        .expect("a single refused interaction is not a session fault");
    assert!(
        refused.replies.is_empty() && !refused.gameplay_changed,
        "the out-of-range interaction answered {refused:?}"
    );
    assert_eq!(
        far_gateway.snapshot(),
        far_before,
        "the out-of-range refusal changed the snapshot"
    );
    let replies = buy(&mut far_gateway, 407, REPRESENTATIVE_ITEM, 1);
    assert_eq!(replies.len(), 1);
    assert_eq!(
        merchant_status(&replies[0]),
        0,
        "a shop opened out of range"
    );
}

#[test]
fn the_round_trip_uses_no_dev_commands() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-ordinary");
    let price = fixture.price();
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    assert_eq!(
        gateway.role(),
        AccountRole::Player,
        "the round trip must run as an ordinary player"
    );

    struct Flow<'a> {
        gateway: &'a mut Gateway,
        requests: Vec<Vec<u8>>,
        replies: Vec<Vec<u8>>,
    }
    impl Flow<'_> {
        fn request(&mut self, packet: Vec<u8>) -> Vec<Vec<u8>> {
            self.requests.push(packet.clone());
            let replies = send(self.gateway, &packet);
            self.replies.extend(replies.clone());
            replies
        }
    }
    let mut flow = Flow {
        gateway: &mut gateway,
        requests: Vec::new(),
        replies: Vec::new(),
    };
    flow.request(interact(REPRESENTATIVE_VENDOR));
    let shop = flow.request(select(REPRESENTATIVE_VENDOR, fixture.option()));
    assert!(
        shop.iter()
            .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
        "the ordinary interaction did not open the shop"
    );
    assert_eq!(
        buy_status(&flow.request(vendor_buy_packet(501, REPRESENTATIVE_ITEM, 1))),
        1
    );
    let (instance, count) = bought_instance(&inventory_of(flow.gateway));
    assert_eq!(count, 1);
    let sold = flow.request(sell_packet(0, instance, 1));
    assert_eq!(completions(&sold)[0].result, RESULT_SUCCESS);

    let headers: Vec<[u8; 4]> = flow
        .requests
        .iter()
        .map(|packet| packet[..4].try_into().unwrap())
        .collect();
    assert_eq!(
        headers,
        vec![
            [26, 0, 8, 0],
            [26, 0, 10, 0],
            [66, 0, 43, 0],
            [165, 0, 5, 0],
        ],
        "the round trip sent a packet outside the ordinary interaction/buy/sell set"
    );
    assert!(
        flow.requests
            .iter()
            .all(|packet| packet[0] != 15 && !packet.windows(5).any(|window| window == b"realm")),
        "a chat or realm command entered the round trip"
    );
    assert!(
        flow.replies.iter().all(|packet| packet[0] != 15),
        "a chat/dev reply entered the ordinary round trip"
    );
    for expected in [[66u8, 0, 42, 0], [66, 0, 44, 0], [165, 0, 6, 0]] {
        assert!(
            flow.replies
                .iter()
                .any(|packet| packet.starts_with(&expected)),
            "the ordinary flow did not answer {expected:?}"
        );
    }
    assert_eq!(flow.gateway.role(), AccountRole::Player);

    // The vendor is spawned from the fixture content by the same visibility loader the world
    // entry uses; no admin spawn or realm command is needed.
    let visibility = NpcVisibility::load(fixture.content()).expect("the fixture NPC content loads");
    assert_eq!(
        visibility.content_key(REPRESENTATIVE_VENDOR),
        Some(REPRESENTATIVE_VENDOR)
    );
    assert!(
        visibility
            .teleport_targets(0)
            .iter()
            .any(|target| target.guid == REPRESENTATIVE_VENDOR),
        "the representative vendor is not in the zone-0 spawn list"
    );
}

#[test]
fn a_client_supplied_price_field_cannot_change_the_server_price() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-forged");
    let price = fixture.price();
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    open_shop(&fixture, &mut gateway);
    let before = inventory_of(&gateway);

    // Same 40-byte shape as the shared builder, but every field the server does not read
    // (the second initial dword, the context dwords and the second trailing dword) forges a
    // 1-coin price. Item, quantity and transaction stay the real fields.
    let forged = encode_merchant_buy_66_43(&MerchantBuyRequestV2 {
        initial_dwords: [REPRESENTATIVE_ITEM as u32, 1],
        context_dwords: [1, 1, 1],
        context_bytes: Vec::new(),
        trailing_dwords: [1, 1, 601],
    })
    .expect("the forged 66/43 encodes");
    assert_eq!(forged.len(), 40);
    assert_ne!(
        forged,
        vendor_buy_packet(601, REPRESENTATIVE_ITEM, 1),
        "the forged price fields did not change the body"
    );

    let replies = send(&mut gateway, &forged);
    assert_eq!(
        buy_status(&replies),
        1,
        "the item/quantity fields did not drive a valid purchase"
    );
    let after = inventory_of(&gateway);
    assert_eq!(
        after.coins,
        before.coins - price,
        "the server used a client-supplied price"
    );
    assert_eq!(count_definition(&after, REPRESENTATIVE_ITEM), 1);
    assert_eq!(
        count_definition(&after, SEEDED_ITEM),
        1,
        "the forged request disturbed the existing bag"
    );
}

#[test]
fn two_players_at_the_same_vendor_keep_independent_wallets_and_shops() {
    let fixture = VendorFixture::new(&fixture_root(), "t374-two");
    let price = fixture.price();
    let mut first = connect(&fixture, fixture.state(price + 1_000, false));
    let mut second = connect(&fixture, fixture.state(price + 1_000, false));
    let first_list = decode_merchant_list_66_42(&open_shop(&fixture, &mut first))
        .expect("first shop matches the client reader");
    let second_list = decode_merchant_list_66_42(&open_shop(&fixture, &mut second))
        .expect("second shop matches the client reader");
    assert_eq!(
        first_list, second_list,
        "the same content priced the shops differently"
    );

    let second_before = second.snapshot();
    assert_eq!(buy_status(&buy(&mut first, 701, REPRESENTATIVE_ITEM, 1)), 1);
    assert_eq!(
        second.snapshot(),
        second_before,
        "one player's purchase changed the other player's session"
    );
    assert_eq!(inventory_of(&second).coins, price + 1_000);
    assert_eq!(
        count_definition(&inventory_of(&second), REPRESENTATIVE_ITEM),
        0
    );

    // The other player's already-open shop and own wallet still serve its own purchase.
    assert_eq!(
        buy_status(&buy(&mut second, 702, REPRESENTATIVE_ITEM, 1)),
        1
    );
    assert_eq!(inventory_of(&second).coins, price + 1_000 - price);
    assert_eq!(inventory_of(&first).coins, price + 1_000 - price);
    assert_eq!(
        count_definition(&inventory_of(&first), REPRESENTATIVE_ITEM),
        1
    );
    assert_eq!(
        count_definition(&inventory_of(&second), REPRESENTATIVE_ITEM),
        1
    );
    let (first_instance, _) = bought_instance(&inventory_of(&first));
    let (second_instance, _) = bought_instance(&inventory_of(&second));
    // Instance ids are per-character namespaces; each purchase still allocated its own new id
    // in its own bag (never the seeded instance 1).
    assert_ne!(first_instance, 1);
    assert_ne!(second_instance, 1);
}
