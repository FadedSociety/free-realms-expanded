//! T484: the Marketplace refuses a Station Cash order for a ride the player already owns, and its
//! 66/39 reply marks owned rides view-only.
//!
//! 66/39 is read by `00b3e490`: a header, a counted list (`00b32d00`, i32 count) of a u32 bundle
//! id and a u8 flag (`00b2b590`), and for every bundle found it stores `flag == 0` at bundle+242,
//! the `IsViewOnly` column (`009a56d0` case 27). Orders go through `CoinShop::handle`, the path
//! the NPC route `npc_merchant_66` uses.

use osfr_tools::{
    coin_shop::{CoinShop, StoreCatalogue},
    inventory::{Inventory, InventoryState},
    station_cash::{self, Wallet},
};
use std::path::PathBuf;

const AIR_T_REX: i32 = 11_317; // max_stack 1: a second grant creates a second item
const SKELETAL_DRAGON: i32 = 10_530; // max_stack -1: a second grant raises the count
const STORE: u32 = 1;
const REFUSED: u32 = station_cash::ORDER_FAILURE;

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_STATION_CASH_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

fn shop(balance: u32) -> CoinShop {
    let directory = inventory_content();
    let inventory = Inventory::load(&directory, None).expect("item content loads");
    let stores = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json");
    let catalogue = StoreCatalogue::load(&stores).expect("store catalogue loads");
    let wallet = Wallet {
        balance,
        ..Wallet::default()
    };
    CoinShop::new(&directory, inventory, catalogue, 100, None, wallet).expect("shop loads")
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client order serializer `00b3fc60`/`00b3af80`: 66/3, or 66/1 with its extra string.
fn order(opcode: u8, tracking: u32, bundle: i32) -> Vec<u8> {
    let mut bytes = vec![66, 0, opcode, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(bundle.to_le_bytes());
    bytes.extend(STORE.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    put_string(&mut bytes, "0");
    put_string(&mut bytes, "");
    if opcode == 1 {
        put_string(&mut bytes, "");
    }
    bytes
}

fn buy(tracking: u32, bundle: i32) -> Vec<u8> {
    order(3, tracking, bundle)
}

fn word(packet: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

/// 66/39 as `00b3e490` reads it: the header, an i32 count, then (u32, u8) per entry, exact end.
fn requirements(packet: &[u8]) -> Vec<(u32, u8)> {
    assert_eq!(&packet[..4], &[66, 0, 39, 0]);
    let count = word(packet, 4) as usize;
    assert_eq!(packet.len(), 8 + 5 * count, "no unread bytes");
    packet[8..]
        .chunks_exact(5)
        .map(|entry| {
            assert!(entry[4] <= 1, "the flag is a boolean byte");
            (word(entry, 0), entry[4])
        })
        .collect()
}

/// The 66/39 in a reply stream, if any (there is at most one).
fn requirements_in(replies: &[Vec<u8>]) -> Option<Vec<(u32, u8)>> {
    let mut found = replies
        .iter()
        .filter(|packet| packet.starts_with(&[66, 0, 39, 0]));
    let list = found.next().map(|packet| requirements(packet));
    assert!(found.next().is_none(), "one 66/39 per reply stream");
    list
}

/// 66/4 as `00b2ded0` reads it (T453): tracking id, then result.
fn order_result(replies: &[Vec<u8>]) -> (u32, u32) {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 4, 0]))
        .expect("a 66/4 order response");
    (word(packet, 4), word(packet, 8))
}

fn preview_result(replies: &[Vec<u8>]) -> u32 {
    let packet = replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 2, 0]))
        .expect("a 66/2 preview response");
    word(packet, 8)
}

fn snapshot(shop: &CoinShop) -> (InventoryState, Wallet) {
    (shop.inventory().state().clone(), shop.sc_wallet().clone())
}

fn count(shop: &CoinShop, item_id: i32) -> u32 {
    shop.inventory()
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == item_id)
        .map(|item| item.count)
        .sum()
}

fn flags(shop: &mut CoinShop) -> Vec<(u32, u8)> {
    let outcome = shop.handle(&[66, 0, 38, 0]).unwrap();
    assert_eq!(outcome.replies.len(), 1, "66/38 gets exactly the 66/39");
    requirements_in(&outcome.replies).expect("a 66/39")
}

/// A ride granted without the shop, as `realm give` and a reward do, then handed to the shop the
/// way `npc_merchant_66` does.
fn give_outside_the_shop(shop: &mut CoinShop, item_id: i32) {
    let mut inventory = shop.inventory().clone();
    inventory.grant_with_tint(0, item_id, 1, 0).expect("grant");
    shop.sync_inventory(inventory);
}

#[test]
fn a_player_owning_nothing_sees_every_listing_purchasable() {
    let mut shop = shop(1_000);
    let list = flags(&mut shop);
    let listed: Vec<u32> = shop
        .sc_catalogue()
        .listings()
        .iter()
        .map(|listing| listing.bundle_id)
        .collect();
    assert_eq!(
        list.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        listed,
        "every listing, in catalogue order"
    );
    assert!(list.len() > 40, "the shipped Rides list is large");
    assert!(list.iter().all(|(_, flag)| *flag == 1));
    for ride in [AIR_T_REX, SKELETAL_DRAGON] {
        assert!(list.iter().any(|(id, _)| *id == ride as u32), "{ride}");
    }
    for earn_only in shop.sc_catalogue().earn_only() {
        assert!(
            list.iter().all(|(id, _)| *id != earn_only.bundle_id),
            "an earn-only ride is never named"
        );
    }
}

/// Gates (b), (c), (d) and (g): both stack kinds.
#[test]
fn buying_a_ride_greys_it_out_and_the_second_order_is_refused() {
    for ride in [AIR_T_REX, SKELETAL_DRAGON] {
        let mut shop = shop(1_000);
        let bought = shop.handle(&buy(1, ride)).unwrap();
        assert_eq!(
            order_result(&bought.replies),
            (1, station_cash::ORDER_SUCCESS)
        );
        assert!(bought.changed);
        assert!(
            bought.replies.last().unwrap().starts_with(&[66, 0, 39, 0]),
            "the 66/39 follows the 66/4 ({ride})"
        );
        let list = requirements_in(&bought.replies).unwrap();
        for (id, flag) in &list {
            assert_eq!(*flag, u8::from(*id != ride as u32), "{ride}: bundle {id}");
        }
        assert_eq!(count(&shop, ride), 1);
        assert_eq!(flags(&mut shop), list, "reopening the store says the same");

        let once = snapshot(&shop);
        let again = shop.handle(&buy(2, ride)).unwrap();
        assert_eq!(order_result(&again.replies), (2, REFUSED), "{ride}");
        assert!(!again.changed);
        assert!(requirements_in(&again.replies).is_none());
        assert_eq!(snapshot(&shop), once, "a refusal changes nothing ({ride})");
        assert_eq!(count(&shop, ride), 1, "never a second copy ({ride})");

        let replay = shop.handle(&buy(1, ride)).unwrap();
        assert_eq!(
            order_result(&replay.replies),
            (1, station_cash::ORDER_SUCCESS),
            "a paid packet replays as success ({ride})"
        );
        assert!(!replay.changed);
        assert!(requirements_in(&replay.replies).is_none());
        assert_eq!(snapshot(&shop), once, "a replay changes nothing ({ride})");
    }
}

/// Gate (e), plus the check order: an owned ride is refused as 2 even with too little Station
/// Cash, never as 5.
#[test]
fn a_ride_granted_outside_the_shop_is_owned() {
    let mut shop = shop(1_000);
    give_outside_the_shop(&mut shop, AIR_T_REX);
    let list = flags(&mut shop);
    assert!(list.contains(&(AIR_T_REX as u32, 0)));
    assert_eq!(list.iter().filter(|(_, flag)| *flag == 0).count(), 1);
    let before = snapshot(&shop);
    let refused = shop.handle(&buy(1, AIR_T_REX)).unwrap();
    assert_eq!(order_result(&refused.replies), (1, REFUSED));
    assert_eq!(snapshot(&shop), before);

    let mut poor = self::shop(1);
    give_outside_the_shop(&mut poor, AIR_T_REX);
    let refused = poor.handle(&buy(1, AIR_T_REX)).unwrap();
    assert_eq!(
        order_result(&refused.replies),
        (1, REFUSED),
        "owned wins over a short balance"
    );
    let unowned = poor.handle(&buy(2, SKELETAL_DRAGON)).unwrap();
    assert_eq!(
        order_result(&unowned.replies),
        (2, station_cash::ORDER_INSUFFICIENT_SC),
        "an unowned ride still reports the short balance"
    );
}

/// Gate (f): the preview shares `price`.
#[test]
fn previewing_an_owned_ride_answers_failure_and_changes_nothing() {
    let mut shop = shop(1_000);
    let open = shop.handle(&order(1, 1, AIR_T_REX)).unwrap();
    assert_eq!(preview_result(&open.replies), station_cash::ORDER_SUCCESS);
    give_outside_the_shop(&mut shop, AIR_T_REX);
    let before = snapshot(&shop);
    let owned = shop.handle(&order(1, 2, AIR_T_REX)).unwrap();
    assert_eq!(preview_result(&owned.replies), REFUSED);
    assert!(!owned.changed);
    assert_eq!(snapshot(&shop), before);
    let other = shop.handle(&order(1, 3, SKELETAL_DRAGON)).unwrap();
    assert_eq!(preview_result(&other.replies), station_cash::ORDER_SUCCESS);
}

/// A deed never enters the bag, so holding the deed item is not owning the lot: its flag stays 1
/// (`Listing::owned_by`).
#[test]
fn a_deed_item_in_the_bag_never_marks_its_listing_owned() {
    let mut shop = shop(1_000);
    let deed = shop
        .sc_catalogue()
        .listings()
        .iter()
        .find(|listing| listing.deed)
        .expect("the shipped catalogue lists deeds")
        .item_id;
    give_outside_the_shop(&mut shop, deed);
    assert!(flags(&mut shop).iter().all(|(_, flag)| *flag == 1));
}
