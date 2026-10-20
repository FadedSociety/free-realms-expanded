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
    NOW, count_definition, inner_packets, interact, inventory_of, login, merchant_status, select,
    sell_packet, send, vendor_buy_packet, wrap,
};
use osfr_tools::{
    account_store::AccountRole, character_store::CharacterState, gameplay::GameplayState,
    gateway::Gateway, npc_interaction::decode_merchant_list_66_42,
};
use std::path::PathBuf;
use vendor_fixture::{REPRESENTATIVE_ITEM, REPRESENTATIVE_VENDOR, SEEDED_ITEM, VendorFixture};

/// The integration suite's private build directory for the fixture pack.
fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/build/w178-vendor-gateway")
}

fn connect(fixture: &VendorFixture, state: GameplayState) -> Gateway {
    connect_with(fixture, fixture.character(), state)
}

/// Reconnect through the saved character half of the save envelope, not a fresh
/// reference character.
fn connect_with(
    fixture: &VendorFixture,
    character: CharacterState,
    state: GameplayState,
) -> Gateway {
    let guid = character.guid;
    let mut gateway = Gateway::load_gameplay(
        fixture.reference(),
        character,
        fixture.content(),
        Some(state),
    )
    .expect("gateway loads the vendor content");
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway
            .handle_at(&packet, NOW)
            .expect("initialization message handles");
    }
    gateway
}

/// The reconnect helper logs in with the saved character's own guid. The gateway
/// compares the ticket guid with the character it loaded, so the pre-fix `login(1)`
/// is refused as "login selects another character".
#[test]
fn connect_with_logs_in_the_saved_character_guid() {
    let fixture = VendorFixture::new(&fixture_root(), "f294-reconnect-guid");
    let mut character = fixture.character();
    character.guid = 7;
    let gateway = connect_with(&fixture, character, fixture.state(1_000, false));
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
            .any(|packet| packet.starts_with(&[66, 0, 42, 0])),
        "the gateway did not open the representative shop"
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
        .find(|packet| packet.starts_with(&[66, 0, 42, 0]))
        .expect("shop list is present");
    // 66/42 is decoded exactly as the client reads it: dispatcher `00b48a10`
    // -> `00b40a60` -> `00b3d5f0` (header + nested record). Group reader
    // `00b33020` reads one scalar, two dword pairs and a final scalar (six
    // dwords), then `00b2e620` reads the entry count and rows via `00b29cd0`.
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
    let mut reconnected = connect_with(&fixture, saved_character.clone(), saved);
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
