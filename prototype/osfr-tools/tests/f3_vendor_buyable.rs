//! F3 (rc-0136 w-vendors, m-garage): Andrea Motorspark, the garage's kart shop and most vendors
//! listed goods the client would not let anyone buy. The merchant window's Buy is live only for an
//! item the client's coin store knows (`00cff140` case 31 -> `00c70fe0`): the login 165/1 lists,
//! or the dynamic list the window asks for with 165/8 the moment it opens (`00c76100` reads the
//! 165/9 answer). The server answered that 165/9 empty, so 271 of Andrea's 280 rows, and every
//! row of 1,219 vendor rows missing from the 165/1 capture, stayed grey and no 165/4 was ever
//! sent; the logged `shop_txn subtype 8 refused` was that empty answer, not a purchase.
//!
//! Driven through the real gateway in the client's order: interact, Shop, the 165/10 window, its
//! 165/8, then the 165/4 buy naming the merchant the window carried (`00cf16c0`).

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "common/vendor_fixture.rs"]
mod vendor_fixture;
#[path = "common/vendor_purchase.rs"]
#[allow(dead_code)]
mod vendor_purchase;

use gateway_vendor::{
    coin_buy_packet, connect_with, count_definition, fixture_root, interact, inventory_of, select,
    send,
};
use osfr_tools::npc_interaction::decode_coin_merchant_list_165_10;
use vendor_fixture::VendorFixture;

const ANDREA: u64 = 9_000_000_008;
/// One of Andrea's shipped rows the login 165/1 capture does not carry.
const PART: i32 = 2_197;
/// A static 165/1 row, never repeated in the dynamic list.
const STATIC_ITEM: i32 = vendor_fixture::SEEDED_ITEM;
const PRICE: u32 = 15;

/// The (id, words, flags) records of a 165/9 (`00c74200`: a count, then id, three dwords and
/// three bytes each).
fn dynamic_records(packet: &[u8]) -> Vec<(i32, [u32; 3], [u8; 3])> {
    assert_eq!(&packet[..4], &[165, 0, 9, 0]);
    let count = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
    let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    let records: Vec<_> = (0..count)
        .map(|index| {
            let at = 8 + index * 19;
            let words = [word(at + 4), word(at + 8), word(at + 12)];
            (word(at) as i32, words, [packet[at + 16], packet[at + 17], packet[at + 18]])
        })
        .collect();
    assert_eq!(packet.len(), 8 + count * 19, "nothing after the records");
    records
}

#[test]
fn a_vendor_row_the_login_list_lacks_becomes_buyable_and_the_buy_pays_and_delivers() {
    let fixture = VendorFixture::for_vendor(&fixture_root(), "f3-andrea", ANDREA, "briarwood.json");
    fixture.stock_price_coins(&[(PART, PRICE), (STATIC_ITEM, PRICE)]);
    let mut gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        fixture.state(100, false),
    );

    // The window before anything opened: the login dynamic list alone, no vendor row.
    let idle = send(&mut gateway, &[165, 0, 8, 0]);
    let idle = dynamic_records(&idle[0]);
    assert!(idle.iter().all(|(id, _, _)| *id != PART), "nothing open, nothing added");

    send(&mut gateway, &interact(ANDREA));
    let opened = send(&mut gateway, &select(ANDREA, fixture.option()));
    let window = opened
        .iter()
        .find(|packet| packet.starts_with(&[165, 0, 10, 0]))
        .expect("Andrea opens her coin window");
    let list = decode_coin_merchant_list_165_10(window).unwrap();
    let rows: Vec<i32> = list.entries.iter().map(|row| row.item_id as i32).collect();
    assert!(rows.contains(&PART) && rows.contains(&STATIC_ITEM), "both rows listed: {rows:?}");
    let merchant = u64::from(list.fields[3]) | (u64::from(list.fields[4]) << 32);
    assert_eq!(merchant, ANDREA, "the window names its merchant");

    // The window's own 165/8: the part becomes known (and buyable, first flag clear) but stays
    // off the gamedock coin store (third flag); the static row is not repeated.
    let update = send(&mut gateway, &[165, 0, 8, 0]);
    let records = dynamic_records(&update[0]);
    let part = records.iter().find(|(id, _, _)| *id == PART).expect("the part is added");
    assert_eq!(part.2, [0, 0, 1], "buyable, unlisted in the gamedock store");
    assert!(records.iter().all(|(id, _, _)| *id != STATIC_ITEM), "a static row is never repeated");
    assert_eq!(records.len(), idle.len() + 1, "the login dynamic records stay");

    // The buy the now-live button sends: coins go down by the price and the part arrives.
    let before = inventory_of(&gateway);
    send(&mut gateway, &coin_buy_packet(merchant, list.fields[0], PART, 1));
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins - PRICE, "the price is paid");
    assert_eq!(
        count_definition(&after, PART),
        count_definition(&before, PART) + 1,
        "the part is in the bag"
    );
}

/// rc-0137 recheck (run 20261112T020333Z): with the shipped `bank.json` the bank answered every
/// 165/8 with its 11 upgrade rows alone, and the client swaps its whole dynamic list for each
/// 165/9 (`00c76100`), so Andrea's rows were never buyable and no 165/4 came. The client's order
/// from that run: 26/8 on Andrea, 26/10 Shop, two 26/20, then the window's 165/8.
#[test]
fn with_the_bank_installed_the_windows_list_keeps_the_vendor_rows_and_the_bank_rows() {
    let fixture = VendorFixture::for_vendor(&fixture_root(), "f3-bank", ANDREA, "briarwood.json");
    fixture.stock_price_coins(&[(PART, PRICE), (STATIC_ITEM, PRICE)]);
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    std::fs::copy(data.join("bank.json"), fixture.content().join("bank.json")).unwrap();
    let mut gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        fixture.state(100, false),
    );
    let idle = dynamic_records(&send(&mut gateway, &[165, 0, 8, 0])[0]);
    let bank: Vec<i32> = idle.iter().take_while(|row| row.1[1] == 62).map(|row| row.0).collect();
    assert!(!bank.is_empty(), "the bank's upgrade rows (category 62) lead the list");

    send(&mut gateway, &interact(ANDREA));
    let opened = send(&mut gateway, &select(ANDREA, fixture.option()));
    let window = opened.iter().find(|packet| packet.starts_with(&[165, 0, 10, 0])).unwrap();
    let list = decode_coin_merchant_list_165_10(window).unwrap();
    send(&mut gateway, &[26, 0, 20, 0]);
    send(&mut gateway, &[26, 0, 20, 0]);
    let update = send(&mut gateway, &[165, 0, 8, 0]);
    assert_eq!(update.len(), 1, "one 165/9: the client keeps only the last");
    let records = dynamic_records(&update[0]);
    let ids: Vec<i32> = records.iter().map(|row| row.0).collect();
    assert!(bank.iter().all(|id| ids.contains(id)), "the bank rows stay");
    let part = records.iter().find(|(id, _, _)| *id == PART).expect("Andrea's row is in");
    assert_eq!(part.2[0], 0, "buyable");
    assert_eq!(records.len(), idle.len() + 1, "the login rows stay too");

    let before = inventory_of(&gateway);
    send(&mut gateway, &coin_buy_packet(ANDREA, list.fields[0], PART, 1));
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins - PRICE, "the buy the live button sends is paid");
    assert_eq!(count_definition(&after, PART), count_definition(&before, PART) + 1);
}

/// C1 (rc-0137 known bugs): the gamedock Coin Shop window buys with no NPC open. Lua
/// `CoinStoreScreen.BuyItem(itemId, quantity, tintId)` (`00bffe30`) -> `BuyItemByItemId`
/// (`00c77a30`) sends 165/4 with the session merchant GUID (never stored: 0) and record -1. The
/// vendor checks refused it as "no open shop" and the client got no answer at all. Only rows the
/// window lists sell this way: a crafted packet for a vendor-only row or a row in no list is
/// refused and costs nothing.
#[test]
fn the_coin_shop_window_buys_a_priced_row_with_no_vendor_open() {
    let fixture = VendorFixture::for_vendor(&fixture_root(), "f3-window", ANDREA, "briarwood.json");
    fixture.stock_price_coins(&[(PART, PRICE)]);
    let mut gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        fixture.state(1_000, false),
    );
    let before = inventory_of(&gateway);
    // One of the rows L1172 moved from the Marketplace to coins for this window, a static row
    // no NPC stocks: the Camp Chatty Banner, 50 coins in the shipped `stores.json`.
    let banner = 14_299;
    let replies = send(&mut gateway, &coin_buy_packet(0, u32::MAX, banner, 1));
    assert!(replies.iter().any(|reply| reply.starts_with(&[165, 0, 6, 0])), "answered");
    let after = inventory_of(&gateway);
    assert_eq!(after.coins, before.coins - 50, "the banner is paid");
    assert_eq!(
        count_definition(&after, banner),
        count_definition(&before, banner) + 1,
        "and in the bag"
    );

    // (Event stock the login lists lack is a window row only while the session's calendar runs
    // its event, so it depends on today's date here; `event_shops.rs` checks it on a fixed
    // calendar.)

    // Crafted window buys for rows the window does not list are refused and cost nothing:
    // Andrea's part (vendor-only), a coin row in no list and at no vendor (49), and the Super Coin
    // booster (Station Cash only).
    for item in [PART, 49, 81] {
        send(&mut gateway, &coin_buy_packet(0, u32::MAX, item, 1));
        let now = inventory_of(&gateway);
        assert_eq!(now.coins, after.coins, "{item} costs nothing");
        assert_eq!(count_definition(&now, item), count_definition(&after, item), "{item}");
    }
    // A vendor's own buy (its GUID and window record) still needs the vendor open.
    send(&mut gateway, &coin_buy_packet(ANDREA, 7, PART, 1));
    assert_eq!(inventory_of(&gateway).coins, after.coins, "no Andrea window is open");
}

/// P-banking (L1161) stores banked units on the item record, so a record whose every unit is in
/// the vault stays in the save but is not in the bag (`InventoryState::bag_stacks`). A player with
/// 100 such records has an empty bag and must still be able to buy; with 100 shown stacks the bag
/// is full and the buy is refused, as before.
#[test]
fn banked_stacks_do_not_fill_the_bag_for_a_vendor_buy() {
    let fixture = VendorFixture::for_vendor(&fixture_root(), "f3-banked", ANDREA, "briarwood.json");
    fixture.stock_price_coins(&[(PART, PRICE)]);
    for banked in [true, false] {
        let mut state = fixture.state(100, true);
        for item in &mut state.inventory.items {
            item.stored_count = if banked { item.count } else { 0 };
        }
        let mut gateway =
            connect_with(fixture.reference(), fixture.character(), fixture.content(), state);
        send(&mut gateway, &interact(ANDREA));
        let opened = send(&mut gateway, &select(ANDREA, fixture.option()));
        let window = opened.iter().find(|packet| packet.starts_with(&[165, 0, 10, 0])).unwrap();
        let list = decode_coin_merchant_list_165_10(window).unwrap();
        send(&mut gateway, &[165, 0, 8, 0]);
        let before = inventory_of(&gateway);
        send(&mut gateway, &coin_buy_packet(ANDREA, list.fields[0], PART, 1));
        let after = inventory_of(&gateway);
        let bought = count_definition(&after, PART) == count_definition(&before, PART) + 1;
        assert_eq!(bought, banked, "banked {banked}: bought {bought}");
        assert_eq!(after.coins, before.coins - if banked { PRICE } else { 0 });
    }
}
