//! The real-content gateway purchase/refusal proof for the representative
//! selected-hub vendor. Kept out of `vendor_stock_coverage.rs` so that file
//! stays a content-coverage suite; the protected test name in that file is a
//! one-line wrapper around this function.
//!
//! The fixture (content pack, prices, character and starting state) is the shared
//! `vendor_fixture.rs` module, which the gateway binary's L011 save-failure test
//! includes too, so both build the same representative vendor instead of copies.
//! The stock row, prices, item definition and transaction path are the shipped
//! ones. The directory is removed on drop.

#[path = "vendor_fixture.rs"]
mod vendor_fixture;

use crate::gateway_vendor::{
    NOW, connect_with, count_definition, fixture_root, inner_packets, interact, inventory_of,
    merchant_status, select, sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::GameplayState, gateway::Gateway,
    npc_interaction::decode_coin_merchant_list_165_10,
};
use vendor_fixture::{REPRESENTATIVE_ITEM, REPRESENTATIVE_VENDOR, SEEDED_ITEM, VendorFixture};

fn connect(fixture: &VendorFixture, state: GameplayState) -> Gateway {
    connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        state,
    )
}

/// The reconnect helper logs in with the saved character's own guid. The gateway
/// compares the ticket guid with the character it loaded, so the pre-fix `login(1)`
/// is refused as "login selects another character".
#[test]
fn connect_with_logs_in_the_saved_character_guid() {
    let fixture = VendorFixture::new(&fixture_root(), "f294-reconnect-guid");
    let mut character = fixture.character();
    character.guid = 7;
    let gateway = connect_with(
        fixture.reference(),
        character,
        fixture.content(),
        fixture.state(1_000, false),
    );
    assert_eq!(gateway.state().guid, 7);
    assert!(
        gateway.is_logged_in(),
        "the helper did not log in the saved character guid"
    );
}

fn open_shop(fixture: &VendorFixture, gateway: &mut Gateway) -> Vec<Vec<u8>> {
    send(gateway, &interact(REPRESENTATIVE_VENDOR));
    let replies = send(gateway, &select(REPRESENTATIVE_VENDOR, fixture.option()));
    assert!(
        replies
            .iter()
            .any(|packet| packet.starts_with(&[165, 0, 10, 0])),
        "the gateway did not open the representative coin shop"
    );
    replies
}

pub fn representative_new_stock_buys_once_and_refuses_atomically() {
    let fixture = VendorFixture::new(&fixture_root(), "f178-vendor-gateway");
    let price = fixture.price();
    let mut gateway = connect(&fixture, fixture.state(price + 1_000, false));
    let opened = open_shop(&fixture, &mut gateway);
    let list = opened
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("shop list is present");
    // 165/10 is decoded exactly as the client reads it (`00c78fb0` -> the shared record
    // reader `00b33020` and row reader `00b29cd0`). The displayed price is the item
    // definition's coin purchase price, so it is not repeated in the row.
    let list = decode_coin_merchant_list_165_10(list).expect("shop list matches the client reader");
    list.entries
        .iter()
        .find(|entry| entry.item_id == REPRESENTATIVE_ITEM as u32)
        .expect("the gateway shop does not expose the new item");

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
        count_definition(&after_inventory, SEEDED_ITEM),
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
    let mut reconnected = connect_with(
        fixture.reference(),
        saved_character.clone(),
        fixture.content(),
        saved,
    );
    let reconnect_snapshot = reconnected.snapshot();
    assert_eq!(
        reconnect_snapshot.character, saved_character,
        "the reconnect ignored the saved character half"
    );
    assert_eq!(inventory_of(&reconnected), after_inventory);

    // A reconnected player has no open merchant interaction: the same buy is
    // refused (66/44 status 0) and must not mutate the restored state.
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
        reconnect_snapshot,
        "the closed-shop refusal changed the reconnected state"
    );

    // Poor coins: the refusal changes nothing, and the shop survives it. Sell
    // the seeded item to top up, then the same buy lands.
    let mut poor = connect(&fixture, fixture.state(price - 1, false));
    open_shop(&fixture, &mut poor);
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
        price - 1 + fixture.sell_price(),
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
        price + 1_000 + fixture.sell_price(),
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
