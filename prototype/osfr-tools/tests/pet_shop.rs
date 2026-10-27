//! T486: a Marketplace order for a pet bundle becomes a pet at once.
//!
//! The Pets category itself (59, listing 206 pet bundles at `round(sc_price * 0.25)`) is T657's and
//! is pinned by `tests/station_cash.rs`; this suite covers the order. The client sends 66/3 for a
//! bought bundle (`00b3fc60`) and reads 66/4 (`00b3b6d0`); the Pets window reads only the 53/5
//! roster (`00b596f0`), so the order's inventory stack is adopted by the same settlement the coin
//! store uses (`PetService::adopt_granted`, T772) and the roster is sent after it.
//!
//! Every request goes through the real gateway dispatch with the shipped `stores.json`, pets
//! content and Station Cash wallet. A refusal or a replay must leave the whole snapshot equal.

#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/vendor_fixture.rs"]
#[allow(dead_code)]
mod vendor_fixture;

use gateway_packets::{marketplace_order, order_result, wallet_balance};
use gateway_vendor::{
    connect_with, count_definition, fixture_root, handled, inventory_of, reload_saved, roster,
};
use osfr_tools::{chat, gateway::Gateway, station_cash};
use vendor_fixture::{VendorFixture, bag_item};

/// Dakota: a Station Cash pet (wiki 400 SC, `stores.json` row 2171), listed at 100 SC.
const DAKOTA: i32 = 2_171;
const DAKOTA_PRICE: u32 = 100;
/// A shipped ride (549 SC provenance), listed at 137 SC.
const RIDE: i32 = 76_856;
const RIDE_PRICE: u32 = 137;
/// An arbitrary non-zero tint the order names.
const TINT: i32 = 3;

/// Pipsqueak: an uncapped (`max_stack` -1) Station Cash pet (`stores.json` row 17840), so a
/// second unit merges into an existing same-tint stack instead of taking a new instance.
const PIPSQUEAK: i32 = 17_840;

/// A gateway on the shipped content whose character holds `balance` Station Cash.
fn shop(label: &str, balance: u32) -> (VendorFixture, Gateway) {
    shop_with(label, balance, None)
}

/// `shop`, with a bag stack of `(definition, tint, count)` already held under instance id 500.
fn shop_with(
    label: &str,
    balance: u32,
    stack: Option<(i32, i32, u32)>,
) -> (VendorFixture, Gateway) {
    let fixture = VendorFixture::new(&fixture_root(), label);
    let mut state = fixture.state(0, false);
    if let Some((definition, tint, count)) = stack {
        state
            .inventory
            .items
            .push(bag_item(definition, tint, 500, count));
        state.inventory.next_instance_id = Some(501);
    }
    state.sc_wallet = Some(station_cash::Wallet {
        balance,
        ..station_cash::Wallet::default()
    });
    let gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        state,
    );
    (fixture, gateway)
}

fn order(tracking: u32, item: i32, tint: i32) -> Vec<u8> {
    marketplace_order(tracking, item as u32, 1, &tint.to_string())
}

fn balance_of(gateway: &Gateway) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .expect("gameplay state exists")
        .sc_wallet
        .expect("the wallet is saved")
        .balance
}

#[test]
fn a_pet_order_debits_once_and_becomes_a_pet_with_the_ordered_tint() {
    let (fixture, mut gateway) = shop("t486-order", 250);
    let bag = inventory_of(&gateway);

    let (replies, outcome) = handled(&mut gateway, &order(1, DAKOTA, TINT));
    assert_eq!(order_result(&replies), station_cash::ORDER_SUCCESS);
    assert_eq!(wallet_balance(&replies), 250 - DAKOTA_PRICE);
    assert!(
        outcome.gameplay_changed,
        "the debit and the pet save together"
    );
    assert_eq!(balance_of(&gateway), 250 - DAKOTA_PRICE, "one debit");

    let after = inventory_of(&gateway);
    assert_eq!(count_definition(&after, DAKOTA), 0, "the item became a pet");
    assert_eq!(after.items, bag.items, "the bag is as it was");
    assert_eq!(after.coins, bag.coins, "an order creates no coins");

    let pets = roster(&gateway);
    assert_eq!(pets.len(), 1, "one order is one pet");
    assert_eq!(pets[0]["adoption_item_definition_id"], DAKOTA);
    assert_eq!(pets[0]["tint"], TINT, "the pet takes the ordered tint");
    assert!(
        replies.iter().any(|packet| packet.starts_with(&[53, 0, 5])),
        "the Pets window gets the 53/5 roster at once"
    );

    // After a reconnect the pet and the debit are still there.
    let restored = reload_saved(fixture.reference(), &gateway, fixture.content());
    assert_eq!(roster(&restored), pets, "reconnect keeps the bought pet");
    assert_eq!(balance_of(&restored), 250 - DAKOTA_PRICE);
}

#[test]
fn replaying_an_order_gives_no_second_pet_and_no_second_debit() {
    let (_fixture, mut gateway) = shop("t486-replay", 250);
    handled(&mut gateway, &order(1, DAKOTA, TINT));
    let once = gateway.snapshot();

    let (replies, outcome) = handled(&mut gateway, &order(1, DAKOTA, TINT));
    assert_eq!(
        order_result(&replies),
        station_cash::ORDER_SUCCESS,
        "a paid duplicate still answers success"
    );
    assert!(!outcome.gameplay_changed && !outcome.state_changed);
    assert!(
        !replies.iter().any(|packet| packet.starts_with(&[53, 0, 5])),
        "a replay sends no roster"
    );
    assert_eq!(gateway.snapshot(), once, "no second pet, no second debit");
    assert_eq!(roster(&gateway).len(), 1);

    // A new tracking id is a second purchase, so a second pet.
    handled(&mut gateway, &order(2, DAKOTA, TINT));
    assert_eq!(roster(&gateway).len(), 2);
    assert_eq!(balance_of(&gateway), 250 - 2 * DAKOTA_PRICE);
}

#[test]
fn too_little_station_cash_refuses_the_order_and_changes_nothing() {
    let (_fixture, mut gateway) = shop("t486-short", DAKOTA_PRICE - 1);
    let before = gateway.snapshot();

    let (replies, outcome) = handled(&mut gateway, &order(1, DAKOTA, TINT));
    assert_eq!(order_result(&replies), station_cash::ORDER_INSUFFICIENT_SC);
    assert!(!outcome.gameplay_changed && !outcome.state_changed);
    assert!(
        !replies.iter().any(|packet| packet.starts_with(&[53, 0, 5])),
        "a refusal sends no roster"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "wallet, bag and roster are equal"
    );
}

/// The brief's full-roster rule (R): the order still succeeds, so the item stays in the bag for
/// the Marketplace's own "Get it now" (53/32, `00a1ec00`), and a notice says why. Nothing is lost.
#[test]
fn a_full_roster_keeps_the_item_and_says_so() {
    let (_fixture, mut gateway) = shop("t486-full", 10_000);
    for tracking in 1..=50 {
        handled(&mut gateway, &order(tracking, DAKOTA, 0));
    }
    assert_eq!(roster(&gateway).len(), 50, "the roster is full");
    assert_eq!(count_definition(&inventory_of(&gateway), DAKOTA), 0);

    let (replies, outcome) = handled(&mut gateway, &order(51, DAKOTA, 0));
    assert_eq!(order_result(&replies), station_cash::ORDER_SUCCESS);
    assert!(outcome.gameplay_changed);
    assert_eq!(balance_of(&gateway), 10_000 - 51 * DAKOTA_PRICE, "paid");
    assert_eq!(roster(&gateway).len(), 50, "no 51st pet");
    assert_eq!(
        count_definition(&inventory_of(&gateway), DAKOTA),
        1,
        "the paid item is kept"
    );
    assert!(
        replies.contains(&chat::system("Your pet list is full; use the item later.")),
        "the player is told"
    );
}

/// A stackable pet merges into a held same-tint stack, so the granted instance survives one
/// adoption. The unit still became a pet, so the full-roster notice must not fire.
#[test]
fn an_adopted_unit_of_a_surviving_stack_sends_no_full_notice() {
    let (_fixture, mut gateway) = shop_with("t486-stack", 250, Some((PIPSQUEAK, TINT, 1)));

    let (replies, _) = handled(&mut gateway, &order(1, PIPSQUEAK, TINT));
    assert_eq!(order_result(&replies), station_cash::ORDER_SUCCESS);
    assert_eq!(roster(&gateway).len(), 1, "the ordered unit became a pet");
    assert_eq!(
        count_definition(&inventory_of(&gateway), PIPSQUEAK),
        1,
        "the held unit is untouched"
    );
    assert!(
        !replies.contains(&chat::system("Your pet list is full; use the item later.")),
        "an adoption that succeeded is not called a full roster"
    );
}

/// Only a pet bundle adopts: a ride order still puts the ride in the bag and sends no roster.
#[test]
fn a_ride_order_is_unchanged() {
    let (_fixture, mut gateway) = shop("t486-ride", 250);
    let (replies, _) = handled(&mut gateway, &order(1, RIDE, 0));
    assert_eq!(order_result(&replies), station_cash::ORDER_SUCCESS);
    assert_eq!(count_definition(&inventory_of(&gateway), RIDE), 1);
    assert_eq!(balance_of(&gateway), 250 - RIDE_PRICE);
    assert!(roster(&gateway).is_empty(), "a ride is not a pet");
    assert!(
        !replies.iter().any(|packet| packet.starts_with(&[53, 0, 5])),
        "no roster for a ride"
    );
}
