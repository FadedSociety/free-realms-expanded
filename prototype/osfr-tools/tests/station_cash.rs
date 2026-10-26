//! T230: the per-character Station Cash wallet, the Marketplace catalogue and ride orders.
//!
//! The wallet response, the order request/result and the store catalogue are pinned to the client
//! readers cited in `src/station_cash.rs` (`00b3e120`/`00b2b450`, `00b3fc60`/`00b3af80`,
//! `00b3b6d0`/`00b2ded0`, `00b48700`/`00b41860`). Everything a refusal may not change is
//! snapshotted before and compared after.

use osfr_tools::{
    coin_shop::{CoinShop, StoreCatalogue},
    inventory::{Inventory, InventoryState},
    station_cash::{self, Wallet},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;

/// A listed ride: item 76856 (the five-hundred-and-fifty Station Cash mount) is 549 SC of
/// provenance, so the shipped 0.25 scale lists it at 137.
const RIDE_ITEM: i32 = 76_856;
const RIDE_PRICE: u32 = 137;
const EARN_ONLY_ITEM: i32 = 79_454;
const STORE: u32 = 1;
/// T526: the four ride sub-categories: row tag, category id (also its 66/6 group key), client
/// locale name id and display order. The ids 2-5 are ours (R); the names are the ones Legacy
/// `StoreBundleCategories.json` gives 266 Winged, 269 Non-Winged, 268 Exotic and 267 Mechanical,
/// all children of Rides (139).
const RIDE_CATEGORIES: [(&str, u32, u32, u32); 4] = [
    ("winged", 2, 5_101_131, 10),
    ("non_winged", 3, 5_101_134, 20),
    ("exotic", 4, 5_101_132, 30),
    ("mechanical", 5, 5_101_133, 40),
];

fn inventory_content() -> PathBuf {
    std::env::var_os("OSFR_STATION_CASH_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/reports/round70-native-20261016T061000Z/content")
        })
}

fn stores() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json")
}

fn new_shop(wallet: Wallet, capacity: usize) -> CoinShop {
    let directory = inventory_content();
    let inventory = Inventory::load(&directory, None).expect("item content loads");
    let catalogue = StoreCatalogue::load(&stores()).expect("store catalogue loads");
    CoinShop::new(&directory, inventory, catalogue, capacity, None, wallet).expect("shop loads")
}

fn wallet(balance: u32) -> Wallet {
    Wallet {
        balance,
        ..Wallet::default()
    }
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// T657: the bundle image-set/tint strings; 0 writes an empty string (no icon).
fn decimal(value: u32) -> String {
    if value == 0 {
        String::new()
    } else {
        value.to_string()
    }
}

/// The client order serializer `00b3fc60`/`00b3af80` (66/1 or 66/3), with one detail.
fn order_packet(
    opcode: u16,
    tracking: u32,
    bundle: u32,
    store: u32,
    quantity: u32,
    customer: &str,
    recipient: &str,
    coupon: &str,
    tint: &str,
) -> Vec<u8> {
    let mut bytes = vec![66, 0, opcode as u8, (opcode >> 8) as u8];
    put_string(&mut bytes, customer);
    put_string(&mut bytes, recipient);
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    put_string(&mut bytes, coupon);
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(bundle.to_le_bytes());
    bytes.extend(store.to_le_bytes());
    bytes.extend(quantity.to_le_bytes());
    put_string(&mut bytes, tint);
    put_string(&mut bytes, "");
    bytes
}

fn buy(tracking: u32) -> Vec<u8> {
    order_packet(3, tracking, RIDE_ITEM as u32, STORE, 1, "", "", "", "0")
}

/// The 35/37 batch body (`00a3db70`): payload byte length (the count plus the records), record
/// count, then the serialized records back to back. Returns `(count, records bytes)`.
fn definition_batch(packet: &[u8]) -> (i32, &[u8]) {
    assert_eq!(&packet[..4], &[35, 0, 37, 0], "35/37 header");
    let length = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    let count = i32::from_le_bytes(packet[8..12].try_into().unwrap());
    assert_eq!(
        length as usize,
        packet.len() - 8,
        "35/37 payload length counts the record-count dword"
    );
    (count, &packet[12..])
}

/// The 66/28 reply (`00b2e2d0`): counted (store id, bundle id) pairs.
fn content_pairs(packet: &[u8]) -> Vec<(u32, u32)> {
    assert_eq!(&packet[..4], &[66, 0, 28, 0], "66/28 header");
    let count = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    assert_eq!(packet.len(), 8 + count as usize * 8, "66/28 pair count");
    packet[8..]
        .chunks_exact(8)
        .map(|pair| {
            (
                u32::from_le_bytes(pair[..4].try_into().unwrap()),
                u32::from_le_bytes(pair[4..].try_into().unwrap()),
            )
        })
        .collect()
}

fn snapshot(shop: &CoinShop) -> (InventoryState, Wallet) {
    (shop.inventory().state().clone(), shop.sc_wallet().clone())
}

fn count_definition(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn wallet_packet_matches_the_client_reader() {
    // Client reader `00b3e120` -> `00b3a6e0`/`00b2b450`: status u32, bool, balance u32,
    // credit-card u32, two strings, bool. `00b3e1a5` feeds the balance to `OnWalletInfo`.
    let packet = station_cash::wallet_packet(RIDE_PRICE);
    assert_eq!(&packet[..4], &[66, 0, 11, 0]);
    assert_eq!(u32::from_le_bytes(packet[4..8].try_into().unwrap()), 1);
    assert_eq!(packet[8], 0);
    assert_eq!(
        u32::from_le_bytes(packet[9..13].try_into().unwrap()),
        RIDE_PRICE
    );
    assert_eq!(u32::from_le_bytes(packet[13..17].try_into().unwrap()), 0);
    assert_eq!(i32::from_le_bytes(packet[17..21].try_into().unwrap()), 0);
    assert_eq!(i32::from_le_bytes(packet[21..25].try_into().unwrap()), 0);
    assert_eq!(packet[25], 0);
    assert_eq!(packet.len(), 26);
}

#[test]
fn order_packet_matches_the_client_writer() {
    let packet = buy(7);
    let order = station_cash::decode_order(&packet, false).expect("order parses");
    assert_eq!(order.tracking_id, 7);
    assert_eq!(order.details.len(), 1);
    assert_eq!(order.details[0].bundle_id, RIDE_ITEM as u32);
    assert_eq!(order.details[0].store_id, STORE);
    assert_eq!(order.details[0].quantity, 1);
    assert_eq!(order.details[0].tint, 0);
    assert!(!order.coupon && !order.gift);

    // 66/1 is the same order plus one trailing string; 66/3 must reject it.
    let mut preview = order_packet(1, 8, RIDE_ITEM as u32, STORE, 1, "", "", "", "0");
    put_string(&mut preview, "preview-extra");
    let decoded = station_cash::decode_order(&preview, true).expect("preview parses");
    assert_eq!(decoded.tracking_id, 8);
    assert!(station_cash::decode_order(&preview, false).is_err());
    // A bare order is also a valid preview (the extra template string is optional).
    let bare = order_packet(1, 13, RIDE_ITEM as u32, STORE, 1, "", "", "", "0");
    assert!(station_cash::decode_order(&bare, true).is_ok());
    let mut truncated_extra = bare.clone();
    truncated_extra.extend(5i32.to_le_bytes());
    truncated_extra.push(b'x');
    assert!(station_cash::decode_order(&truncated_extra, true).is_err());

    // The client's order debug string (`0x01843730`) names the fields customerUserKey,
    // recipientUserKey, orderTrackingId, customerLocaleString and couponCode in this order. The
    // Lua `CreateNewOrder` binding (`sub_C039F0`) substitutes the local player for the self
    // sentinel, and `sub_B376D0` writes the local player to customer and the requested player to
    // recipient, so an ordinary self-order repeats the customer key. Only a different recipient
    // is a gift.
    let self_order = order_packet(3, 14, RIDE_ITEM as u32, STORE, 1, "42", "42", "", "0");
    let decoded = station_cash::decode_order(&self_order, false).expect("self order parses");
    assert!(!decoded.gift && !decoded.coupon);
    let gift = order_packet(3, 15, RIDE_ITEM as u32, STORE, 1, "42", "43", "", "0");
    let decoded = station_cash::decode_order(&gift, false).expect("gift order parses");
    assert!(decoded.gift && !decoded.coupon);

    // Trailing bytes, a hostile string length and too many details are malformed, not orders.
    let mut trailing = buy(9);
    trailing.push(0);
    assert!(station_cash::decode_order(&trailing, false).is_err());
    let mut huge = buy(10);
    huge[4..8].copy_from_slice(&i32::MAX.to_le_bytes());
    assert!(station_cash::decode_order(&huge, false).is_err());
    let mut many = buy(11);
    let count_at = many.len() - 25; // detail (bundle, store, quantity, tint, unknown) + count
    many[count_at..count_at + 4].copy_from_slice(&17u32.to_le_bytes());
    assert!(station_cash::decode_order(&many, false).is_err());
    // A non-numeric tint string is refused before it can reach `grant_with_tint`.
    assert!(
        station_cash::decode_order(
            &order_packet(3, 12, RIDE_ITEM as u32, STORE, 1, "", "", "", "nope"),
            false
        )
        .is_err()
    );
}

/// The 66/4 in a reply stream. T484: a successful order is followed by a 66/39, so it is not last.
fn order_reply(replies: &[Vec<u8>]) -> &Vec<u8> {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 4, 0]))
        .expect("order response")
}

#[test]
fn order_debits_exact_price_and_grants_once() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    let before = snapshot(&shop);
    let packet = buy(71);
    let outcome = shop.handle(&packet).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    assert!(outcome.changed);
    assert_eq!(shop.sc_wallet().balance, 200 - RIDE_PRICE);
    assert_eq!(
        count_definition(shop.inventory().state(), RIDE_ITEM),
        count_definition(&before.0, RIDE_ITEM) + 1
    );
    // No coin path ran.
    assert_eq!(shop.inventory().state().coins, before.0.coins);
    assert_eq!(shop.sc_wallet().orders.len(), 1);
    // 66/4 full layout (`00b2ded0`, T453): tracking id, result, order-id string, discount, total.
    let reply = order_reply(&outcome.replies);
    assert_eq!(&reply[..4], &[66, 0, 4, 0]);
    assert_eq!(u32::from_le_bytes(reply[4..8].try_into().unwrap()), 71);
    assert_eq!(u32::from_le_bytes(reply[8..12].try_into().unwrap()), 1);
    assert_eq!(i32::from_le_bytes(reply[12..16].try_into().unwrap()), 2);
    assert_eq!(&reply[16..18], b"71");
    assert_eq!(u32::from_le_bytes(reply[18..22].try_into().unwrap()), 0);
    assert_eq!(
        u32::from_le_bytes(reply[22..26].try_into().unwrap()),
        RIDE_PRICE
    );
    assert_eq!(reply.len(), 26);
    let once = snapshot(&shop);

    // The same packet is paid at most once: success is replayed, state is untouched.
    let replay = shop.handle(&packet).expect("replay parses");
    assert_eq!(replay.result_code, station_cash::ORDER_SUCCESS);
    assert!(!replay.changed);
    assert!(
        replay
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[66, 0, 11, 0]))
    );
    assert!(
        replay
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[66, 0, 4, 0, 71, 0, 0, 0, 1, 0, 0, 0]))
    );
    assert_eq!(snapshot(&shop), once);
}

#[test]
fn refusals_change_nothing() {
    // Insufficient Station Cash (order failure 5).
    let mut poor = new_shop(wallet(100), usize::MAX);
    let before = snapshot(&poor);
    let short = poor.handle(&buy(80)).expect("order parses");
    assert_eq!(short.result_code, station_cash::ORDER_INSUFFICIENT_SC);
    assert!(!short.changed);
    assert_eq!(snapshot(&poor), before);

    // Full bag.
    let mut full = new_shop(wallet(200), 0);
    let before = snapshot(&full);
    let refused = full.handle(&buy(81)).expect("order parses");
    assert_eq!(refused.result_code, station_cash::ORDER_FAILURE);
    assert!(!refused.changed);
    assert_eq!(snapshot(&full), before);

    // An unlisted item (there is no Station Cash price for item 8), an unknown store, an
    // earn-only reward row, a quantity other than one, a gift recipient and a coupon.
    for (label, packet) in [
        (
            "unlisted",
            order_packet(3, 82, 8, STORE, 1, "", "", "", "0"),
        ),
        (
            "unknown store",
            order_packet(3, 83, RIDE_ITEM as u32, STORE + 1, 1, "", "", "", "0"),
        ),
        (
            "earn only",
            order_packet(3, 84, EARN_ONLY_ITEM as u32, STORE, 1, "", "", "", "0"),
        ),
        (
            "quantity",
            order_packet(3, 85, RIDE_ITEM as u32, STORE, 2, "", "", "", "0"),
        ),
        (
            "gift",
            order_packet(
                3,
                86,
                RIDE_ITEM as u32,
                STORE,
                1,
                "buyer",
                "friend",
                "",
                "0",
            ),
        ),
        (
            "coupon",
            order_packet(3, 87, RIDE_ITEM as u32, STORE, 1, "", "", "FREE", "0"),
        ),
    ] {
        let mut shop = new_shop(wallet(200), usize::MAX);
        let before = snapshot(&shop);
        let outcome = shop.handle(&packet).expect("order parses");
        assert_eq!(outcome.result_code, station_cash::ORDER_FAILURE, "{label}");
        assert!(!outcome.changed, "{label}");
        assert_eq!(snapshot(&shop), before, "{label}");
    }

    // The shared busy table refuses the order before any state is staged. `handle_marketplace`
    // takes that answer as an argument, so there is no second flag to keep in sync.
    let mut busy = new_shop(wallet(200), usize::MAX);
    let before = snapshot(&busy);
    let outcome = busy
        .handle_marketplace(&buy(88), true, &mut |_, _| None)
        .expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_FAILURE);
    assert_eq!(snapshot(&busy), before);

    // A malformed order is an error, never a zero-cost grant.
    let mut malformed = new_shop(wallet(200), usize::MAX);
    let before = snapshot(&malformed);
    assert!(malformed.handle(&buy(89)[..20]).is_err());
    assert_eq!(snapshot(&malformed), before);
}

#[test]
fn preview_prices_without_changing_state() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    let before = snapshot(&shop);
    let mut packet = order_packet(1, 91, RIDE_ITEM as u32, STORE, 1, "", "", "", "0");
    put_string(&mut packet, "");
    let outcome = shop.handle(&packet).expect("preview parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    let reply = outcome.replies.first().expect("preview reply");
    assert_eq!(&reply[..4], &[66, 0, 2, 0]);
    assert_eq!(u32::from_le_bytes(reply[4..8].try_into().unwrap()), 91);
    assert_eq!(u32::from_le_bytes(reply[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(reply[12..16].try_into().unwrap()), 0);
    assert_eq!(
        u32::from_le_bytes(reply[16..20].try_into().unwrap()),
        RIDE_PRICE
    );
    assert_eq!(reply.len(), 20);
    assert!(!outcome.changed);
    assert_eq!(snapshot(&shop), before);

    let mut poor = new_shop(wallet(100), usize::MAX);
    let before = snapshot(&poor);
    let mut packet = order_packet(1, 92, RIDE_ITEM as u32, STORE, 1, "", "", "", "0");
    put_string(&mut packet, "");
    let outcome = poor.handle(&packet).expect("preview parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_INSUFFICIENT_SC);
    assert_eq!(snapshot(&poor), before);
}

/// 66/4 exactly as the client reads it: `00b2dd60` (u16 family, u16 opcode), then `00b2ded0`
/// (tracking id to obj+0x18, result to obj+0x1C, i32-length order-id string, discount, total).
/// `00b3b6d0` drops the reply if any byte is left over.
fn client_order_response(reply: &[u8]) -> (u32, u32, String, u32, u32) {
    let mut reader = Reader {
        bytes: reply,
        at: 0,
    };
    assert_eq!((reader.u16(), reader.u16()), (66, 4), "66/4 header");
    let tracking = reader.u32();
    let result = reader.u32();
    let order_id = String::from_utf8(reader.string().to_vec()).expect("order id text");
    let response = (tracking, result, order_id, reader.u32(), reader.u32());
    assert_eq!(reader.at, reply.len(), "66/4 trailing bytes");
    response
}

/// `00b3b6d0`: the pending order (manager+0x3E8E8, tracking id at +0x44) completes only when the
/// reply carries its tracking id, and succeeds only when the result is 1.
fn client_outcome(pending: u32, reply: &[u8]) -> (bool, bool) {
    let (tracking, result, ..) = client_order_response(reply);
    (tracking == pending, result == station_cash::ORDER_SUCCESS)
}

/// 66/2 exactly as `00b2dd60` + `00b2de20` read it: four u32 into obj+0x18..+0x24, nothing after.
fn client_preview_response(reply: &[u8]) -> (u32, u32, u32, u32) {
    let mut reader = Reader {
        bytes: reply,
        at: 0,
    };
    assert_eq!((reader.u16(), reader.u16()), (66, 2), "66/2 header");
    let response = (reader.u32(), reader.u32(), reader.u32(), reader.u32());
    assert_eq!(reader.at, reply.len(), "66/2 trailing bytes");
    response
}

/// T453: the client's tracking ids restart at 1 every run (`00b28360`). With the result written
/// first, the first order worked by coincidence and the second read as tracking 1, result 2.
#[test]
fn order_response_tracking_id_precedes_result() {
    let mut shop = new_shop(wallet(1000), usize::MAX);
    let other = *shop
        .sc_catalogue()
        .listings()
        .iter()
        // L046: a deed needs housing, which `CoinShop::handle` does not have.
        .find(|listing| {
            !listing.deed
                && listing.bundle_id != RIDE_ITEM as u32
                && listing.price <= 1000 - RIDE_PRICE
        })
        .expect("a second affordable listed ride");

    // (a) Two different rides in one client run, tracking 1 then 2: both complete and succeed.
    let first = shop.handle(&buy(1)).expect("order parses");
    let reply = order_reply(&first.replies);
    assert_eq!(
        client_order_response(reply),
        (1, 1, "1".to_string(), 0, RIDE_PRICE)
    );
    assert_eq!(client_outcome(1, reply), (true, true));
    let second_packet = order_packet(3, 2, other.bundle_id, STORE, 1, "", "", "", "0");
    let second = shop.handle(&second_packet).expect("order parses");
    let reply = order_reply(&second.replies);
    assert_eq!(
        client_order_response(reply),
        (2, 1, "2".to_string(), 0, other.price)
    );
    assert_eq!(client_outcome(2, reply), (true, true));
    assert_eq!(shop.sc_wallet().balance, 1000 - RIDE_PRICE - other.price);

    // (c) A replay of tracking 2 echoes it and changes nothing.
    let paid = snapshot(&shop);
    let replay = shop.handle(&second_packet).expect("replay parses");
    assert!(!replay.changed);
    let reply = order_reply(&replay.replies);
    assert_eq!(
        client_order_response(reply),
        (2, 1, "2".to_string(), 0, other.price)
    );
    assert_eq!(snapshot(&shop), paid);

    // (d) An unknown bundle (item 8 has no Station Cash row) with tracking 3: order failure.
    let unknown = shop
        .handle(&order_packet(3, 3, 8, STORE, 1, "", "", "", "0"))
        .expect("order parses");
    let reply = order_reply(&unknown.replies);
    assert_eq!(client_order_response(reply), (3, 2, "3".to_string(), 0, 0));
    assert_eq!(client_outcome(3, reply), (true, false));
    assert_eq!(snapshot(&shop), paid);

    // (b) Too little Station Cash, tracking 1: balance-not-enough, never success.
    let mut poor = new_shop(wallet(RIDE_PRICE - 1), usize::MAX);
    let before = snapshot(&poor);
    let short = poor.handle(&buy(1)).expect("order parses");
    let reply = order_reply(&short.replies);
    assert_eq!(client_order_response(reply), (1, 5, "1".to_string(), 0, 0));
    assert_eq!(client_outcome(1, reply), (true, false));
    assert_eq!(snapshot(&poor), before);
}

/// T453: 66/2 carries the tracking id first too (`00b3b450` reads the result at obj+0x1C).
#[test]
fn preview_response_tracking_id_precedes_result() {
    let mut preview = order_packet(1, 9, RIDE_ITEM as u32, STORE, 1, "", "", "", "0");
    put_string(&mut preview, "");
    for (balance, expected) in [(200, (9, 1, 0, RIDE_PRICE)), (100, (9, 5, 0, 0))] {
        let mut shop = new_shop(wallet(balance), usize::MAX);
        let outcome = shop.handle(&preview).expect("preview parses");
        assert_eq!(outcome.replies.len(), 1, "balance {balance}");
        assert_eq!(
            client_preview_response(&outcome.replies[0]),
            expected,
            "balance {balance}"
        );
    }
}

#[test]
fn wallet_survives_serde_and_restores_under_the_data_ceiling() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    let outcome = shop.handle(&buy(60)).expect("order parses");
    assert!(outcome.changed);
    let saved = shop.sc_wallet().clone();
    let encoded = serde_json::to_vec(&saved).expect("wallet serializes");
    let decoded: Wallet = serde_json::from_slice(&encoded).expect("wallet parses");
    assert_eq!(decoded, saved);
    assert_eq!(decoded.balance, 200 - RIDE_PRICE);
    assert_eq!(decoded.orders.len(), 1);

    // The save field is the wallet itself; an over-ceiling or overlong history is repaired on
    // restore rather than trusted.
    let config = shop.sc_catalogue().config();
    let too_rich = Wallet {
        balance: u32::MAX,
        orders: saved.orders.clone(),
        ..Wallet::default()
    };
    let restored = Wallet::restore(Some(too_rich), config);
    assert_eq!(restored.balance, config.max_balance);
    let mut too_many = saved.clone();
    too_many.orders = (0..config.order_history + 10)
        .map(|index| {
            let mut record = saved.orders[0].clone();
            record.tracking_id = index as u32;
            record
        })
        .collect();
    let restored = Wallet::restore(Some(too_many), config);
    assert_eq!(restored.orders.len(), config.order_history);
    assert_eq!(restored.orders[0].tracking_id, 10);

    // A package with Station Cash off must not clamp or prune a wallet it does not serve.
    let mut disabled = (*config).clone();
    disabled.enabled = false;
    let untouched = Wallet {
        balance: u32::MAX,
        orders: saved.orders.clone(),
        ..Wallet::default()
    };
    assert_eq!(
        Wallet::restore(Some(untouched.clone()), &disabled),
        untouched
    );
}

/// The generated item catalogue marks every ride NO_TRADE (byte 44 of the serialized definition,
/// `piece_096`/`Inventory::trade_flags`). Ride rows are absent from the 302-row shipped
/// `ItemDefinitions.txt` and from the captured client rows, so this is Legacy-sourced catalogue
/// data, not a packet capture; T230 keeps the existing trade gate's refusal rather than
/// overriding item data. Goal B (tradeable SC rides) is explicitly deferred to the lead's economy
/// decision (F324 review item 9), not silently claimed.
#[test]
fn bought_ride_catalogue_row_is_no_trade() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    assert!(shop.handle(&buy(61)).expect("order parses").changed);
    // Rides are the rows whose provenance names a ride. L046 lot deeds and the T634 wiki-priced
    // items are listed too; deeds grant a lot and the wiki items are tradeable.
    let stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("stores.json reads"))
            .expect("stores.json parses");
    let ride_rows: Vec<i64> = stores["items"]
        .as_array()
        .expect("store rows")
        .iter()
        .filter(|row| !row["price_provenance"]["ride_id"].is_null())
        .filter_map(|row| row["item_id"].as_i64())
        .collect();
    let items: Vec<i32> = shop
        .sc_catalogue()
        .listings()
        .iter()
        .chain(shop.sc_catalogue().earn_only())
        .map(|listing| listing.item_id)
        .filter(|item| ride_rows.contains(&i64::from(*item)))
        .collect();
    // T452: 54 seeded plus 35 OSFR-parity rides.
    assert_eq!(items.len(), 89, "every ride has a catalogue row");
    for item in items {
        let (no_trade, _) = shop
            .inventory()
            .trade_flags(item)
            .unwrap_or_else(|| panic!("ride item {item} has no definition"));
        assert!(
            no_trade,
            "ride item {item} is tradeable in the item catalogue"
        );
    }
}

/// T528: the Training Saddle of Gliding is a Station Cash listing in the Winged ride sub-category.
/// Its sc_price is the (R) placeholder for the owner; the item's own catalogue row is NoSale, so the
/// 66/3 order is the only way to buy it and no coin path touches the row. The provenance names the
/// two references the brief cites: Sanctuary/Legacy `StoreBundles.json` bundle 5549 under
/// CategoryGroupId 266 (Winged), and the wiki Glide Training page.
#[test]
fn the_saddle_is_sold_for_station_cash() {
    const SADDLE: i32 = 79_150;
    const SADDLE_SC_PRICE: u64 = 200;
    let mut shop = new_shop(wallet(1_000), usize::MAX);
    let saddle = shop
        .sc_catalogue()
        .listing(SADDLE as u32)
        .expect("the saddle is listed");
    assert!(!saddle.deed, "a saddle is not a lot deed");
    assert!(!saddle.ride, "a saddle is not a ride");
    let scale = shop.sc_catalogue().config().sc_price_scale;
    let price = (SADDLE_SC_PRICE as f64 * scale).round() as u32;
    assert_eq!(saddle.price, price, "listed at the scaled sc_price");
    assert_eq!(
        saddle.group, RIDE_CATEGORIES[0].1,
        "the saddle lists under the Winged child of Rides"
    );

    let stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("stores.json reads"))
            .expect("stores.json parses");
    let row = stores["items"]
        .as_array()
        .expect("store rows")
        .iter()
        .find(|row| row["item_id"].as_i64() == Some(i64::from(SADDLE)))
        .expect("the saddle row");
    assert_eq!(row["sc_price"].as_u64(), Some(SADDLE_SC_PRICE));
    assert_eq!(
        row["no_sale"].as_bool(),
        Some(true),
        "NoSale, like its item row"
    );
    assert_eq!(row["ride_category"].as_str(), Some("winged"));
    let basis = row["price_provenance"]["basis"]
        .as_str()
        .expect("the row carries its provenance");
    assert!(
        basis.contains("5549") && basis.contains("266") && basis.contains("Glide Training"),
        "the provenance names its sources: {basis}"
    );

    let before = snapshot(&shop);
    let order = order_packet(3, 71, SADDLE as u32, STORE, 1, "", "", "", "0");
    let outcome = shop.handle(&order).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    assert!(outcome.changed);
    assert_eq!(shop.sc_wallet().balance, 1_000 - price, "the scaled price");
    assert_eq!(
        count_definition(shop.inventory().state(), SADDLE),
        count_definition(&before.0, SADDLE) + 1,
        "one saddle granted"
    );
    assert_eq!(
        shop.inventory().state().coins,
        before.0.coins,
        "no coin path ran"
    );
    // A second order sells another saddle: a stackable, not a single-copy ride (T484).
    let again = shop
        .handle(&order_packet(
            3,
            72,
            SADDLE as u32,
            STORE,
            1,
            "",
            "",
            "",
            "0",
        ))
        .expect("order parses");
    assert_eq!(again.result_code, station_cash::ORDER_SUCCESS);
    assert_eq!(
        count_definition(shop.inventory().state(), SADDLE),
        count_definition(&before.0, SADDLE) + 2,
        "the row sells again"
    );
}

/// T634: Dog Ears (a head accessory, three tints) is a wiki Marketplace item at 49 Station Cash
/// (page revision 36898); its store rows had no price. Castaway Hat spans two item classes under
/// one name (16031 is the class 1 one), so its wiki price cannot be assigned and its row stays
/// unpriced. Turtle is a wiki Coin Shop pet at 10000 coins (revision 49986), 9000 for a member;
/// the Legacy reference gave it no Cost.
const DOG_EARS: [i32; 3] = [22_927, 900_046, 900_047];
const DOG_EARS_WIKI_SC: f64 = 49.0;
const AMBIGUOUS_HAT: i32 = 16_031;
const TURTLE: i32 = 70_430;

#[test]
fn a_wiki_priced_clothing_item_lists_at_a_quarter_and_buying_it_grants_it() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    let scale = shop.sc_catalogue().config().sc_price_scale;
    assert_eq!(
        scale, 0.25,
        "the owner: the Marketplace sells at 1/4 of the live price"
    );
    let price = (DOG_EARS_WIKI_SC * scale).round() as u32;
    for item in DOG_EARS {
        let listing = shop
            .sc_catalogue()
            .listing(item as u32)
            .unwrap_or_else(|| panic!("Dog Ears tint {item} is not listed"));
        assert!(!listing.deed, "a head accessory is not a lot deed");
        assert_eq!(
            listing.price, price,
            "tint {item} lists at 1/4 of the wiki price"
        );
    }
    let before = snapshot(&shop);
    let order = order_packet(3, 101, DOG_EARS[0] as u32, STORE, 1, "", "", "", "0");
    let outcome = shop.handle(&order).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    assert!(outcome.changed);
    assert_eq!(shop.sc_wallet().balance, 200 - price);
    assert_eq!(
        count_definition(shop.inventory().state(), DOG_EARS[0]),
        count_definition(&before.0, DOG_EARS[0]) + 1
    );
    assert_eq!(
        shop.inventory().state().coins,
        before.0.coins,
        "no coin path ran"
    );
}

#[test]
fn an_ambiguous_wiki_name_stays_unpriced() {
    let mut shop = new_shop(wallet(200), usize::MAX);
    assert!(shop.sc_catalogue().listing(AMBIGUOUS_HAT as u32).is_none());
    let before = snapshot(&shop);
    let order = order_packet(3, 102, AMBIGUOUS_HAT as u32, STORE, 1, "", "", "", "0");
    let outcome = shop.handle(&order).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_FAILURE);
    assert!(!outcome.changed);
    assert_eq!(snapshot(&shop), before);
}

#[test]
fn a_coin_shop_item_without_a_price_sells_at_the_wiki_price() {
    let directory = inventory_content();
    let mut state = Inventory::load(&directory, None)
        .expect("item content loads")
        .state()
        .clone();
    state.coins = 20_000;
    for (member, price) in [(false, 10_000), (true, 9_000)] {
        let inventory = Inventory::load(&directory, Some(state.clone())).expect("coins are valid");
        let catalogue = StoreCatalogue::load(&stores()).expect("store catalogue loads");
        let mut shop = CoinShop::new(
            &directory,
            inventory,
            catalogue,
            usize::MAX,
            None,
            wallet(0),
        )
        .expect("shop loads");
        shop.set_member_status(member);
        let outcome = shop.buy(1, TURTLE, 1).expect("buy parses");
        assert_eq!(outcome.result_code, osfr_tools::coin_shop::RESULT_SUCCESS);
        assert_eq!(
            shop.inventory().state().coins,
            20_000 - price,
            "member {member}"
        );
        assert_eq!(count_definition(shop.inventory().state(), TURTLE), 1);
    }
}

/// T638: Dog Tail (three tints) is 49 Station Cash on the wiki (revision 36899, SC Marketplace and
/// not the Coin Shop) and no coin vendor stocks it; Legacy and the client definition give it a coin
/// Cost of 270 (client reader `008e3760`). It lists at 1/4 of the wiki price and no longer sells
/// for coins.
const DOG_TAIL: [i32; 3] = [79_511, 79_512, 79_513];

#[test]
fn a_station_cash_only_item_lists_at_a_quarter_and_no_longer_sells_for_coins() {
    let directory = inventory_content();
    let mut state = Inventory::load(&directory, None)
        .expect("item content loads")
        .state()
        .clone();
    state.coins = 20_000;
    let inventory = Inventory::load(&directory, Some(state)).expect("coins are valid");
    let catalogue = StoreCatalogue::load(&stores()).expect("store catalogue loads");
    let mut shop = CoinShop::new(
        &directory,
        inventory,
        catalogue,
        usize::MAX,
        None,
        wallet(200),
    )
    .expect("shop loads");
    let price = (49.0 * shop.sc_catalogue().config().sc_price_scale).round() as u32;
    assert_eq!(price, 12, "1/4 of the wiki price");
    for item in DOG_TAIL {
        let listing = shop.sc_catalogue().listing(item as u32).expect("listed");
        assert_eq!(listing.price, price, "tint {item}");
    }
    // The coin path is gone: a coin purchase is refused and changes nothing.
    let before = snapshot(&shop);
    let refused = shop.buy(1, DOG_TAIL[0], 1).expect("buy parses");
    assert_eq!(
        refused.result_code,
        osfr_tools::coin_shop::RESULT_SERVICE_UNAVAILABLE
    );
    assert!(!refused.changed);
    assert_eq!(snapshot(&shop), before);
    // The Station Cash order grants it and moves no coins.
    let order = order_packet(3, 103, DOG_TAIL[0] as u32, STORE, 1, "", "", "", "0");
    let outcome = shop.handle(&order).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    assert_eq!(shop.sc_wallet().balance, 200 - price);
    assert_eq!(
        count_definition(shop.inventory().state(), DOG_TAIL[0]),
        count_definition(&before.0, DOG_TAIL[0]) + 1
    );
    assert_eq!(shop.inventory().state().coins, 20_000);
}

/// T638: every row the move touched (tag T638) is a Station Cash row with no coin price, keeps the
/// Legacy price in its provenance, and is in no coin vendor's stock; every item a coin vendor
/// stocks keeps its coin price and is not a Station Cash listing.
#[test]
fn moved_rows_are_station_cash_only_and_vendor_stock_keeps_coins() {
    let read = |file: &str| -> Value {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/content")
            .join(file);
        serde_json::from_slice(&std::fs::read(path).expect("content reads")).expect("parses")
    };
    let stores = read("stores.json");
    let stock: std::collections::BTreeSet<i64> = read("vendors.json")["vendors"]
        .as_array()
        .expect("vendors")
        .iter()
        .flat_map(|vendor| vendor["item_ids"].as_array().expect("stock").clone())
        .filter_map(|id| id.as_i64())
        .collect();
    let mut moved = 0;
    for row in stores["items"].as_array().expect("rows") {
        let id = row["item_id"].as_i64().expect("id");
        if stock.contains(&id) {
            assert!(
                row["sc_price"].is_null() && !row["purchase_price_coins"].is_null(),
                "vendor-stocked item {id} lost its coin price"
            );
        }
        if row["source_tags"]
            .as_array()
            .expect("tags")
            .iter()
            .any(|tag| tag == "T638")
        {
            moved += 1;
            assert!(!stock.contains(&id), "{id} is in vendor stock");
            assert!(
                row["purchase_price_coins"].is_null(),
                "{id} still sells for coins"
            );
            assert!(row["member_price_coins"].is_null() && row["sell_price_coins"].is_null());
            assert_eq!(row["no_sale"], true);
            assert!(row["sc_price"].as_u64().expect("sc price") > 0, "{id}");
            assert!(
                row["price_provenance"]["legacy_coin_price"]
                    .as_u64()
                    .expect("kept")
                    > 0,
                "{id} lost the Legacy price record"
            );
        }
    }
    assert_eq!(moved, 663, "the Station Cash only items moved from coins");
}

#[test]
fn catalogue_packets_carry_the_scaled_price_and_skip_earn_only() {
    let shop = new_shop(Wallet::default(), usize::MAX);
    let packets = station_cash::catalogue_packets(shop.sc_catalogue());
    let store = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 5, 0]))
        .expect("store packet");
    // The exact field positions (including the listed price) are asserted by
    // `store_packet_matches_the_client_reader_field_for_field`; this test only checks the
    // catalogue-level rules: the earn-only row is absent and every navigation packet is pushed.
    assert!(!contains_u32(store, EARN_ONLY_ITEM as u32));
    assert!(
        packets
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 7, 0]))
    );
    assert!(
        packets
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 6, 0]))
    );
    assert!(
        packets
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 9, 0]))
    );
    assert!(
        packets
            .iter()
            .any(|packet| packet.starts_with(&[66, 0, 24, 0]))
    );
}

fn contains_u32(bytes: &[u8], value: u32) -> bool {
    bytes
        .windows(4)
        .any(|window| u32::from_le_bytes(window.try_into().unwrap()) == value)
}

/// F387: the Marketplace reply set (now sent on the 66/10 wallet request) carries the shipped
/// rides and their scaled SC prices. The pre-F387 name is kept because the integrator protects
/// every passing test by name. The 66/5 rows are walked exactly as `00b43170`/`00b41860` read
/// them and compared with every `stores.json` row that has an `sc_price` and is not earn-only.
/// T380: the rows are then listed per category the way the client's StoreItems filter does
/// (`0099f150`): a bundle's category group (+204) must be a 66/6 group, and one of that group's
/// categories must be the selected one or below it in the 66/7 tree (`0099d7e0`). The selected
/// category is the client's own Rides id, 139.
#[test]
fn world_entry_store_rows_are_the_shipped_rides_at_their_scaled_prices() {
    let shop = new_shop(Wallet::default(), usize::MAX);
    let packets = shop.marketplace_packets();
    assert_eq!(packets[0], vec![66, 0, 24, 0, 1], "66/24 handshake");
    assert_eq!(packets[1], vec![66, 0, 33, 0, 0, 0], "66/33 sources off");
    let store = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 5, 0]))
        .expect("66/5 store packet");
    let config = shop.sc_catalogue().config();
    let mut reader = Reader {
        bytes: store,
        at: 0,
    };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 5);
    assert_eq!(reader.u32(), 1, "inner type");
    assert_eq!(reader.u32(), config.store_id);
    assert_eq!(reader.u32(), config.store_id, "store id");
    assert_eq!(reader.u32(), config.store_name_id, "store name id");
    assert_eq!(
        reader.u32(),
        config.store_description_id,
        "store description id"
    );
    reader.string();
    reader.string();
    let count = reader.u32() as usize;
    let mut decoded = BTreeMap::new();
    for _ in 0..count {
        let key = reader.u32();
        assert_eq!(reader.u32(), key, "bundle id");
        reader.u32();
        reader.u32();
        reader.u32();
        reader.string();
        reader.string();
        reader.u8();
        reader.string();
        assert_eq!(reader.u32(), config.currency_type);
        let price = reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u64();
        reader.u64();
        reader.u32();
        reader.u8();
        reader.u64();
        reader.u64();
        assert_eq!(reader.u32(), 1, "one entry");
        assert_eq!(reader.u32(), 1, "quantity");
        let item = reader.u32();
        assert_eq!(reader.u32(), item, "marketing item id");
        reader.string();
        assert_eq!(reader.u32(), config.store_id);
        let group = reader.u32();
        reader.u8();
        for _ in 0..8 {
            reader.u32();
        }
        reader.u8();
        reader.u8();
        reader.u8();
        // `00b41860` continuation: u32, u32, u8, u32, u32, u32, u8, u8, u32, u32.
        reader.u32();
        reader.u32();
        reader.u8();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u8();
        reader.u8();
        reader.u32();
        reader.u32();
        assert_eq!(item, key, "the bundle's game item is its map key");
        decoded.insert(key, (price, group));
    }
    assert_eq!(
        reader.at,
        store.len(),
        "the store packet has trailing bytes"
    );
    // 52 rides plus the 9 lot deeds L046 lists, plus T452's 29 listed OSFR-parity rides (6 more
    // are earn-only), plus the 55 items T634 priced from the wiki, plus the 663 Station Cash only
    // items T638 moved from their Legacy coin price, plus the 4 boomboxes T495 priced, plus the
    // Training Saddle of Gliding T528 lists.
    assert_eq!(
        decoded.len(),
        813,
        "the shipped listed ride, deed, wiki item, boombox and saddle count"
    );
    assert!(
        store.len() < osfr_tools::soe::MAX_LOGICAL_MESSAGE / 2,
        "the store packet must stay well inside one logical message"
    );

    // 66/7 (`00b48040`/`00b43090`): counted category nodes, then the sorted byte.
    let tree = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 7, 0]))
        .expect("66/7 category tree");
    let mut reader = Reader { bytes: tree, at: 4 };
    let mut parents = BTreeMap::new();
    let mut names = BTreeMap::new();
    let mut counts = BTreeMap::new();
    let mut orders = BTreeMap::new();
    for _ in 0..reader.u32() {
        reader.u32();
        let id = reader.u32();
        names.insert(id, reader.u32());
        reader.string();
        reader.string();
        parents.insert(id, reader.u32());
        counts.insert(id, reader.u32());
        orders.insert(id, reader.u32());
    }
    reader.u8();
    assert_eq!(reader.at, tree.len(), "66/7 trailing bytes");
    // 66/6 (`00b44d70`/`00b3ad20`): group key, group id, counted category ids.
    let groups = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 6, 0]))
        .expect("66/6 category groups");
    let mut reader = Reader {
        bytes: groups,
        at: 4,
    };
    let mut categories_of = BTreeMap::new();
    for _ in 0..reader.u32() {
        let key = reader.u32();
        reader.u32();
        let ids: Vec<u32> = (0..reader.u32()).map(|_| reader.u32()).collect();
        categories_of.insert(key, ids);
    }
    assert_eq!(reader.at, groups.len(), "66/6 trailing bytes");
    // Lua `MarketplaceCategoryIds.RIDES`, the mount screen's buy-more-rides link and the Rides
    // portal tile (`ClientStorePortalCategories.txt`, name 27547) all open category 139.
    const RIDES: u32 = 139;
    assert_eq!(names.get(&RIDES), Some(&27_547), "the Rides category node");
    // T495: the Boombox category, from Legacy `StoreBundleCategories.json` (id 89, name 151365).
    const BOOMBOX: u32 = 89;
    assert_eq!(
        names.get(&BOOMBOX),
        Some(&151_365),
        "the Boombox category node"
    );
    assert_eq!(
        parents.get(&BOOMBOX),
        Some(&0),
        "Boombox is top level, like Rides"
    );
    // T526: the four ride sub-categories hang under Rides. The parent is record dword 3 (the first
    // `00b3b080` field), `00b43090` re-keys every node by it, and `0099d7e0` walks the resulting
    // index, so selecting 139 lists every child's bundles and selecting a child lists only its own.
    for (key, id, name_id, order) in RIDE_CATEGORIES {
        assert_eq!(names.get(&id), Some(&name_id), "{key} name");
        assert_eq!(parents.get(&id), Some(&RIDES), "{key} is a Rides child");
        assert_eq!(orders.get(&id), Some(&order), "{key} display order");
    }
    let listed_under = |selected: u32| -> BTreeMap<u32, u32> {
        let under = |mut category: u32| loop {
            if category == selected {
                return true;
            }
            match parents.get(&category) {
                Some(&parent) if parent != 0 => category = parent,
                _ => return false,
            }
        };
        decoded
            .iter()
            .filter(|(_, (_, group))| {
                categories_of
                    .get(group)
                    .is_some_and(|ids| ids.iter().any(|id| under(*id)))
            })
            .map(|(key, (price, _))| (*key, *price))
            .collect()
    };

    let stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("stores.json reads"))
            .expect("stores.json parses");
    let scale = stores["station_cash"]["sc_price_scale"]
        .as_f64()
        .expect("scale");
    let rows = stores["items"].as_array().expect("store rows");
    let listed = |row: &Value| {
        row["earn_only"].as_bool() != Some(true) && row["boombox"].as_bool() != Some(true)
    };
    let price_of =
        |row: &Value| (row["sc_price"].as_u64().expect("price") as f64 * scale).round() as u32;
    let expected = |boombox: bool| -> BTreeMap<u32, u32> {
        rows.iter()
            .filter(|row| row["sc_price"].as_u64().is_some())
            .filter(|row| row["earn_only"].as_bool() != Some(true))
            .filter(|row| (row["boombox"].as_bool() == Some(true)) == boombox)
            .filter_map(|row| Some((u32::try_from(row["item_id"].as_i64()?).ok()?, price_of(row))))
            .collect()
    };
    let expected_category = |key: &str| -> BTreeMap<u32, u32> {
        rows.iter()
            .filter(|row| row["sc_price"].as_u64().is_some())
            .filter(|row| listed(row))
            .filter(|row| row["ride_category"].as_str() == Some(key))
            .filter_map(|row| Some((u32::try_from(row["item_id"].as_i64()?).ok()?, price_of(row))))
            .collect()
    };
    // T657: the real client Marketplace categories for the non-ride rows, named by Legacy
    // `StoreBundleCategories.json` and resolved by the client locale.
    const CLOTHES: u32 = 45;
    const PETS: u32 = 59;
    const HOUSING: u32 = 116;
    for (id, name_id) in [(CLOTHES, 23_327), (PETS, 954), (HOUSING, 21_088)] {
        assert_eq!(names.get(&id), Some(&name_id), "category {id} name");
        assert_eq!(parents.get(&id), Some(&0), "category {id} is top level");
    }
    let expected_rides: BTreeMap<u32, u32> = rows
        .iter()
        .filter(|row| row["sc_price"].as_u64().is_some())
        .filter(|row| listed(row))
        .filter(|row| row["ride_category"].is_string())
        .filter_map(|row| Some((u32::try_from(row["item_id"].as_i64()?).ok()?, price_of(row))))
        .collect();
    let rides = listed_under(RIDES);
    assert_eq!(
        rides, expected_rides,
        "Rides lists the ride rows, and only them"
    );
    for (key, id, _, _) in RIDE_CATEGORIES {
        assert_eq!(
            listed_under(id),
            expected_category(key),
            "{key} lists exactly its tagged rows"
        );
        assert_eq!(
            counts.get(&id),
            Some(&(expected_category(key).len() as u32)),
            "{key} counts its own bundles"
        );
    }
    // T657: a ride and a clothing listing land in different categories. Turkey Hat (41) is item
    // category 101 (Hats), so the Clothes & Armor shelf 45; the sample ride is a Winged child.
    const TURKEY_HAT: u32 = 41;
    assert!(
        !rides.contains_key(&TURKEY_HAT),
        "clothing is not under Rides"
    );
    let clothes = listed_under(CLOTHES);
    assert_eq!(
        clothes.get(&TURKEY_HAT),
        Some(&25),
        "Turkey Hat is clothing at its scaled price"
    );
    assert!(
        clothes.contains_key(&(DOG_EARS[0] as u32)),
        "another clothing row is under the same shelf"
    );
    assert!(
        !clothes.contains_key(&(RIDE_ITEM as u32)),
        "the sample ride is not under Clothes"
    );
    let pets = listed_under(PETS);
    assert_eq!(pets.get(&2171), Some(&100), "Dakota is a pet");
    assert_eq!(
        pets.get(&27608),
        Some(&50),
        "a pet costume is not clothing (item category 6)"
    );
    let housing = listed_under(HOUSING);
    assert_eq!(housing.get(&11037), Some(&150), "a housing fixture");
    assert_eq!(housing.get(&17192), Some(&100), "a lot deed");
    // Every bundle is under exactly one leaf category; the leaves cover the whole catalogue.
    let mut covered = BTreeSet::new();
    for leaf in RIDE_CATEGORIES
        .iter()
        .map(|(_, id, _, _)| *id)
        .chain([BOOMBOX, CLOTHES, PETS, HOUSING])
    {
        for item in listed_under(leaf).keys() {
            assert!(
                covered.insert(*item),
                "listing {item} is in two leaf categories"
            );
        }
    }
    assert_eq!(
        covered,
        decoded.keys().copied().collect(),
        "every bundle has a leaf category"
    );
    // T657: the written count is the client's recompute (`00c37e70`): the node plus descendants.
    for id in names.keys() {
        assert_eq!(
            counts.get(id),
            Some(&(listed_under(*id).len() as u32)),
            "count of category {id}"
        );
    }
    // T526 gate: every listed ride appears under exactly one child, at its scaled price. T528: the
    // Training Saddle of Gliding is not a ride but lists in Winged beside the rides it trains.
    let mut tagged = 0;
    for row in rows.iter().filter(|row| listed(row)) {
        let Some(key) = row["ride_category"].as_str() else {
            continue;
        };
        tagged += 1;
        let item = u32::try_from(row["item_id"].as_i64().expect("item id")).expect("bounded");
        let children: Vec<&str> = RIDE_CATEGORIES
            .iter()
            .filter(|(_, id, _, _)| listed_under(*id).contains_key(&item))
            .map(|(key, _, _, _)| *key)
            .collect();
        assert_eq!(
            children,
            vec![key],
            "ride {item} lists under exactly one child"
        );
        assert_eq!(rides.get(&item), Some(&price_of(row)), "ride {item} price");
    }
    assert_eq!(
        tagged, 82,
        "every listed ride and the T528 saddle name their final-era type"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["ride_category"].is_string())
            .count(),
        90,
        "every shipped ride row and the T528 saddle name their final-era type, earn-only ones included"
    );
    assert_eq!(
        listed_under(RIDE_CATEGORIES[0].1).get(&(RIDE_ITEM as u32)),
        Some(&RIDE_PRICE),
        "the sample ride is Winged"
    );
    // Earn-only rides carry no bundle, so no category can list them.
    for row in rows
        .iter()
        .filter(|row| row["earn_only"].as_bool() == Some(true))
    {
        let item = u32::try_from(row["item_id"].as_i64().expect("item id")).expect("bounded");
        assert!(
            !decoded.contains_key(&item),
            "earn-only {item} is a 66/5 bundle"
        );
        for (key, id, _, _) in RIDE_CATEGORIES {
            assert!(
                !listed_under(id).contains_key(&item),
                "earn-only {item} is listed under {key}"
            );
        }
    }
    let boomboxes = listed_under(BOOMBOX);
    assert_eq!(
        boomboxes,
        expected(true),
        "Boombox lists exactly the tagged boomboxes at their SC prices"
    );
    assert_eq!(
        boomboxes.len(),
        11,
        "the wiki-documented Marketplace boomboxes"
    );
    assert!(
        boomboxes.keys().all(|id| !rides.contains_key(id)),
        "no boombox is listed under Rides"
    );
}

/// Walk a buffer exactly as the client's 66/x readers do, refusing a short or trailing byte.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> &'a [u8] {
        let end = self.at.checked_add(length).expect("cursor overflow");
        let bytes = self
            .bytes
            .get(self.at..end)
            .unwrap_or_else(|| panic!("packet ends at {} of {end}", self.bytes.len()));
        self.at = end;
        bytes
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn string(&mut self) -> &'a [u8] {
        let length = self.u32() as usize;
        self.take(length)
    }
}

/// T230: the 66/5 store packet is walked field by field in the exact order of the client readers
/// `00b48700` -> `00b2dfd0` -> `00b45340` -> `00b2ea50` -> `00b43170` -> `00b41860` ->
/// `00b3da50` -> `00b2ea50` -> `00b370d0` -> `00b2f390` -> `00b3fa00` -> `00b41860`'s ten-field
/// continuation. The tail after `00b3fa00` is the part a scan-for-a-u32 test cannot catch; a short
/// record misaligns every following bundle, so the last byte must be consumed exactly. T657: the
/// description, image-set, image-tint, price, IsOnSale and SalePrice values are compared with the
/// item catalogue, so the native `<STRING 0 NOT FOUND>`, empty icon, price 0 and SALE! badge cannot
/// come back. T744: the members-only price (+268) is read for every row and must equal the regular
/// price, because a 0 there is the 100% member discount the native run showed.
#[test]
fn store_packet_matches_the_client_reader_field_for_field() {
    let shop = new_shop(Wallet::default(), usize::MAX);
    let packets = station_cash::catalogue_packets(shop.sc_catalogue());
    let store = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 5, 0]))
        .expect("66/5 store packet");
    // Header (4) + inner type, store id and the store definition (two strings) + bundle count:
    // 36 bytes. Every bundle is a u32 map key plus a 181-byte record (definition 106, `00b3fa00`
    // tail 44, `00b41860` continuation 31); the two image strings add their payload length. Assert
    // the exact reader size as well as walking it, so a field added to or dropped from the writer
    // cannot pass by ending where the test stops.
    let listings = shop.sc_catalogue().listings();
    let inventory = shop.inventory();
    let expected_size = 36
        + listings
            .iter()
            .map(|listing| 185 + decimal(listing.icon_id).len() + decimal(listing.icon_tint).len())
            .sum::<usize>();
    assert_eq!(store.len(), expected_size, "66/5 packet size");
    let mut described = 0usize;
    let mut turkey_hat_description = None;
    // T744: every listing's (regular price, member price, IsOnSale, description) as the wire
    // carries them, so the boombox rows can be compared with the rest of the store.
    let mut tiles = BTreeMap::new();
    let mut reader = Reader {
        bytes: store,
        at: 0,
    };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 5);
    assert_eq!(reader.u32(), 1, "inner type");
    let store_id = reader.u32();
    let config = shop.sc_catalogue().config();
    assert_eq!(store_id, config.store_id);
    assert_eq!(reader.u32(), config.store_id, "store id");
    assert_eq!(reader.u32(), config.store_name_id, "store name id");
    assert_eq!(
        reader.u32(),
        config.store_description_id,
        "store description id"
    );
    reader.string();
    reader.string();
    assert_eq!(reader.u32() as usize, listings.len(), "bundle map count");
    for listing in listings {
        let presentation = inventory
            .presentation(listing.item_id)
            .unwrap_or_else(|error| panic!("item {}: {error}", listing.item_id));
        assert_eq!(reader.u32(), listing.bundle_id, "bundle map key");
        // MarketingBundleDefinition (`00b3da50`).
        assert_eq!(reader.u32(), listing.bundle_id, "bundle id");
        assert_eq!(reader.u32(), listing.name_id, "name id");
        // T657: tile Description (`009a56d0` case 4) from the item catalogue; an item without one
        // shows its name (F719), so no tile resolves locale id 0.
        let description = reader.u32();
        let real = u32::try_from(presentation.description_id).unwrap_or(0);
        assert_eq!(
            description,
            if real == 0 { listing.name_id } else { real },
            "description id (item {})",
            listing.item_id
        );
        assert_ne!(description, 0, "item {} description", listing.item_id);
        if real != 0 {
            described += 1;
        }
        if listing.item_id == 41 {
            turkey_hat_description = Some(description);
        }
        reader.u32();
        // T657: the image object at +0x18 (`00b2ea50`); the client `atoi`s both strings for
        // `ImageSetId`/`ImageTintValue` (`009a56d0` cases 6-7).
        assert_eq!(
            reader.string(),
            decimal(u32::try_from(presentation.icon_id).unwrap_or(0)).as_bytes(),
            "image set id (item {})",
            listing.item_id
        );
        assert_eq!(
            reader.string(),
            decimal(u32::try_from(presentation.icon_tint).unwrap_or(0)).as_bytes(),
            "image tint (item {})",
            listing.item_id
        );
        reader.u8();
        reader.string();
        assert_eq!(reader.u32(), config.currency_type);
        // T657: tile Price (`009a56d0` case 5). T744: no listed row's data price is 0.
        let price = reader.u32();
        assert_eq!(price, listing.price, "listed price");
        assert!(price > 0, "item {} listed price", listing.item_id);
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u64();
        reader.u64();
        reader.u32();
        reader.u8();
        reader.u64();
        reader.u64();
        // Counted entry list (`00b370d0`).
        assert_eq!(reader.u32(), 1, "one entry");
        assert_eq!(reader.u32(), 1, "quantity");
        assert_eq!(reader.u32(), listing.item_id as u32, "game item id");
        assert_eq!(reader.u32(), listing.item_id as u32, "marketing item id");
        reader.string();
        // StoreBundleDefinition tail (`00b3fa00`): StoreId, CategoryGroupId, IsOnSale (+243), the
        // terms id (+208), MaxPerOrder (+212), six more u32 and three bytes.
        assert_eq!(reader.u32(), config.store_id);
        assert_eq!(reader.u32(), listing.group, "bundle category group");
        let on_sale = reader.u8();
        assert_eq!(on_sale, 0, "no shipped row is on sale");
        assert_eq!(reader.u32(), 0, "no terms and conditions locale id");
        assert_eq!(reader.u32(), 1, "max per order");
        for _ in 0..6 {
            reader.u32();
        }
        reader.u8();
        reader.u8();
        reader.u8();
        // `00b41860` continuation: u32, u32, u8, u32, u32, u32, u8, u8, u32, u32 (31 bytes).
        assert_eq!(reader.u32(), 0, "00b41860 field 1");
        assert_eq!(reader.u32(), 0, "00b41860 field 2");
        assert_eq!(reader.u8(), 0, "00b41860 field 3");
        assert_eq!(reader.u32(), 0, "00b41860 field 4");
        // T744: the members-only price (+268, `009a56d0` case 32) is the regular price when the
        // row has no member price. A 0 here is the 100% member discount the native run showed.
        let member_price = reader.u32();
        assert_eq!(
            member_price, price,
            "00b41860 field 5 (members price, item {})",
            listing.item_id
        );
        tiles.insert(listing.item_id, (price, member_price, on_sale, description));
        assert_eq!(reader.u32(), 0, "00b41860 field 6 (sale price)");
        assert_eq!(reader.u8(), 0, "00b41860 field 7");
        assert_eq!(reader.u8(), 0, "00b41860 field 8");
        assert_eq!(reader.u32(), 0, "00b41860 field 9");
        assert_eq!(reader.u32(), 0, "00b41860 field 10");
    }
    assert!(described > 0, "some tiles carry a real description id");
    assert_eq!(
        turkey_hat_description,
        Some(4177),
        "Turkey Hat's own description (item catalogue bytes)"
    );
    // T744: the Boombox rows the native run showed (Jingle Bell 878, Realms Roll 1180, Rainbow
    // 1913, XVD-S 48244, Pop-Dance 76927, Tiki 76928) all carry the same shape and a real price.
    // A member and a non-member both pay the regular price: the client's `getItemPrice` returns
    // Price to a non-member and `getMOItemPrice` returns MembersOnlyPrice to a member, so equal
    // values mean a 0% discount. The three rows that used to show the "profile" placeholder are
    // items whose definitions were only missing on the client side; the 66/5 shape is identical.
    for id in [878, 1180, 1913, 48_244, 76_927, 76_928] {
        let (price, member_price, on_sale, description) = tiles
            .get(&id)
            .unwrap_or_else(|| panic!("boombox {id} is listed"));
        assert!(*price > 0, "boombox {id} regular price");
        assert_eq!(
            *member_price, *price,
            "boombox {id} member price is the regular price"
        );
        assert_eq!(*on_sale, 0, "boombox {id} is not on sale");
        assert_ne!(*description, 0, "boombox {id} description");
    }
    assert_eq!(
        reader.at,
        store.len(),
        "the store packet has trailing unread bytes"
    );
}

/// T744: the client asked for the bundle content of exactly these rows before the native run
/// (their definitions were absent from its local map), and the tile stayed on its "profile"
/// placeholder because the server never answered. The 66/27 reply now carries the 35/37 item
/// definitions first and the 66/28 pending-clearing pairs second, through the same
/// `CoinShop::handle` entry the gateway uses.
#[test]
fn content_request_sends_the_item_definitions_then_the_66_28_pairs() {
    let mut shop = new_shop(Wallet::default(), usize::MAX);
    let before = snapshot(&shop);
    let request =
        gateway_packets::marketplace_content(&[(STORE, 878, &[878]), (STORE, 76_927, &[76_927])]);
    let outcome = shop.handle(&request).expect("content request parses");
    assert_eq!(outcome.result_code, 1);
    assert!(!outcome.changed, "definitions are presentation, not state");
    assert_eq!(
        outcome.replies.len(),
        2,
        "the definitions must precede the 66/28 pending clear"
    );
    let (count, records) = definition_batch(&outcome.replies[0]);
    assert_eq!(count, 2);
    let expected = [
        shop.inventory().definition_record(878).expect("item 878"),
        shop.inventory()
            .definition_record(76_927)
            .expect("item 76927"),
    ];
    assert_eq!(
        records,
        expected.concat().as_slice(),
        "the two requested rows, in request order"
    );
    assert_eq!(
        content_pairs(&outcome.replies[1]),
        vec![(STORE, 878), (STORE, 76_927)]
    );
    assert_eq!(
        snapshot(&shop),
        before,
        "a content reply changes no session state"
    );
}

/// T744: a hostile or stale 66/27 cannot select content, cannot grow the reply and changes nothing.
/// Only a listed bundle of the configured store is confirmed, and the item ids in the request are
/// never used to choose a definition.
#[test]
fn content_request_refusals_and_unknown_bundles_change_nothing() {
    let mut shop = new_shop(Wallet::default(), usize::MAX);
    let before = snapshot(&shop);
    let empty = station_cash::content_response_packet(&[]);
    // A count above the parser bound, a truncated entry and a trailing byte are refused.
    assert!(shop.handle(&[66, 0, 27, 0, 9, 0, 0, 0]).is_err());
    let mut truncated = gateway_packets::marketplace_content(&[(STORE, 878, &[878])]);
    truncated.truncate(truncated.len() - 1);
    assert!(shop.handle(&truncated).is_err());
    let mut trailing = gateway_packets::marketplace_content(&[(STORE, 878, &[878])]);
    trailing.push(0);
    assert!(shop.handle(&trailing).is_err());
    // An unlisted bundle and a bundle from another store are not confirmed.
    for request in [
        gateway_packets::marketplace_content(&[(STORE, 999_999_999, &[1])]),
        gateway_packets::marketplace_content(&[(STORE + 1, 878, &[878])]),
    ] {
        let outcome = shop.handle(&request).expect("bounded request parses");
        assert!(!outcome.changed);
        assert_eq!(outcome.replies, vec![empty.clone()]);
    }
    // A listed bundle's reply is the server's item, not the client's ids: a bogus id list still
    // gets item 878's definition because 878 is what the bundle contains.
    let request = gateway_packets::marketplace_content(&[(STORE, 878, &[1, 2, 3])]);
    let outcome = shop.handle(&request).expect("listed request parses");
    assert_eq!(
        station_cash::content_response_packet(&[(STORE, 878)]),
        outcome.replies[1]
    );
    let (count, records) = definition_batch(&outcome.replies[0]);
    assert_eq!(count, 1);
    assert_eq!(
        &records[..4],
        &878i32.to_le_bytes(),
        "the server's listing chooses the definition"
    );
    assert_eq!(snapshot(&shop), before);
}

/// T744: the native client's own 66/27 bodies (trace `server-trace.jsonl` in the 2026-10-26 run,
/// `ts_ms` 539171 and 539184): one store-1 entry per missing Boombox, with the item id repeated as
/// the marketing item id. These exact bodies are decoded and answered here, so the parser is
/// tested against the shipped client, not only against the test helper.
#[test]
fn captured_66_27_requests_decode_and_are_answered() {
    let captured = [
        [
            66u8, 0, 27, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x6e, 0x03, 0, 0, 1, 0, 0, 0, 0x6e, 0x03, 0, 0,
        ],
        [
            66, 0, 27, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x7f, 0x2c, 0x01, 0, 1, 0, 0, 0, 0x7f, 0x2c,
            0x01, 0,
        ],
        [
            66, 0, 27, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x80, 0x2c, 0x01, 0, 1, 0, 0, 0, 0x80, 0x2c,
            0x01, 0,
        ],
    ];
    for (body, item) in captured.into_iter().zip([878i32, 76_927, 76_928]) {
        let decoded = station_cash::decode_content_request(&body).expect("captured request");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].store_id, STORE);
        assert_eq!(decoded[0].bundle_id, item as u32);
        assert_eq!(decoded[0].item_ids, vec![item as u32]);
        let mut shop = new_shop(Wallet::default(), usize::MAX);
        let outcome = shop.handle(&body).expect("captured request is answered");
        assert_eq!(outcome.replies.len(), 2);
        let (count, records) = definition_batch(&outcome.replies[0]);
        assert_eq!(count, 1);
        assert_eq!(records, shop.inventory().definition_record(item).unwrap());
        assert_eq!(
            content_pairs(&outcome.replies[1]),
            vec![(STORE, item as u32)]
        );
    }
}

/// T230: walk the catalogue navigation packets in their client reader order: category tree
/// `00b48040`/`00b43090`, category groups `00b44d70`/`00b3ad20`, bundle groups
/// `00b45560`/`00b3d770`/`00b36bb0`/`00b2a090`.
#[test]
fn catalogue_navigation_packets_match_the_client_readers() {
    let shop = new_shop(Wallet::default(), usize::MAX);
    let packets = station_cash::catalogue_packets(shop.sc_catalogue());
    let config = shop.sc_catalogue().config();
    let listings = shop.sc_catalogue().listings();
    // T495/T526/T657: Rides and Boombox are top level, then the three T657 item shelves, then the
    // four ride sub-categories under Rides. The ids and names are the client's real Marketplace
    // categories (Legacy `StoreBundleCategories.json`); each shelf id is its own 66/6 group key.
    let shelves = [
        (
            config.category_id,
            config.category_name_id,
            config.category_group_id,
            0,
            1,
        ),
        (89, 151_365, 89, 0, 2),
        (45, 23_327, 45, 0, 3),
        (59, 954, 59, 0, 4),
        (116, 21_088, 116, 0, 5),
        (2, 5_101_131, 2, config.category_id, 10),
        (3, 5_101_134, 3, config.category_id, 20),
        (4, 5_101_132, 4, config.category_id, 30),
        (5, 5_101_133, 5, config.category_id, 40),
    ];
    // T657: a node's count is its own bundles plus its descendants', the number the client
    // recomputes (`00c37e70`).
    let under = |selected: u32| -> u32 {
        let subtree: Vec<u32> = shelves
            .iter()
            .filter(|(category, _, _, _, _)| {
                let mut at = *category;
                loop {
                    if at == selected {
                        break true;
                    }
                    match shelves.iter().find(|(node, _, _, _, _)| *node == at) {
                        Some((_, _, _, parent, _)) if *parent != 0 => at = *parent,
                        _ => break false,
                    }
                }
            })
            .map(|(_, _, group, _, _)| *group)
            .collect();
        u32::try_from(
            listings
                .iter()
                .filter(|listing| subtree.contains(&listing.group))
                .count(),
        )
        .expect("bounded")
    };

    // 66/7: counted category nodes, then the sorted byte.
    let tree = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 7, 0]))
        .expect("66/7 category tree");
    let mut reader = Reader { bytes: tree, at: 0 };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 7);
    assert_eq!(
        reader.u32() as usize,
        shelves.len(),
        "nine categories (T657 adds three item shelves)"
    );
    for (category, name, _, parent, order) in shelves {
        assert_eq!(reader.u32(), category, "category map key");
        assert_eq!(reader.u32(), category, "category id");
        assert_eq!(reader.u32(), name, "category name id");
        reader.string();
        reader.string();
        assert_eq!(reader.u32(), parent, "parent id");
        assert_eq!(reader.u32(), under(category), "category count");
        assert_eq!(reader.u32(), order, "display order");
    }
    assert_eq!(reader.u8(), 1, "sorted");
    assert_eq!(reader.at, tree.len(), "66/7 trailing bytes");

    // 66/6: one counted group per shelf, each with its one category id.
    let groups = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 6, 0]))
        .expect("66/6 category groups");
    let mut reader = Reader {
        bytes: groups,
        at: 0,
    };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 6);
    assert_eq!(reader.u32() as usize, shelves.len(), "nine category groups");
    for (category, _, group, _, _) in shelves {
        assert_eq!(reader.u32(), group, "group map key");
        assert_eq!(reader.u32(), group, "group id");
        assert_eq!(reader.u32(), 1, "one category id");
        assert_eq!(reader.u32(), category);
    }
    assert_eq!(reader.at, groups.len(), "66/6 trailing bytes");

    // 66/9: one counted bundle group; each entry is the map key, the entry's own bundle id and
    // its display order.
    let bundle_groups = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 9, 0]))
        .expect("66/9 bundle groups");
    let mut reader = Reader {
        bytes: bundle_groups,
        at: 0,
    };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 9);
    assert_eq!(reader.u32(), 1, "one bundle group");
    assert_eq!(reader.u32(), config.category_group_id, "group map key");
    assert_eq!(reader.u32(), config.category_group_id, "group id");
    assert_eq!(reader.u32(), config.category_name_id, "group name id");
    assert_eq!(reader.u32(), 0, "group description id");
    reader.string();
    reader.string();
    assert_eq!(reader.u8(), 0, "exclusive");
    assert_eq!(reader.u32(), 1, "status");
    assert_eq!(reader.u32() as usize, listings.len(), "entry count");
    for (index, listing) in listings.iter().enumerate() {
        assert_eq!(reader.u32(), listing.bundle_id, "entry map key");
        assert_eq!(reader.u32(), listing.bundle_id, "entry bundle id");
        assert_eq!(reader.u32(), index as u32 + 1, "entry display order");
    }
    assert_eq!(reader.at, bundle_groups.len(), "66/9 trailing bytes");
}

/// T495: Realms Roll (wiki 1000 SC, 250 listed) is a boombox the Marketplace sells. The
/// reward-only boomboxes (TCG and Mystery Chest prizes) have no Marketplace price on the wiki.
const REALMS_ROLL: i32 = 1_180;
const REWARD_ONLY_BOOMBOXES: [i32; 4] = [8_185, 35_759, 56_774, 69_825];

fn buy_item(tracking: u32, item: i32) -> Vec<u8> {
    order_packet(3, tracking, item as u32, STORE, 1, "", "", "", "0")
}

/// T495 gate 2: one order grants exactly one boombox and debits once; a replay grants nothing.
#[test]
fn a_boombox_order_debits_once_and_grants_one_and_a_replay_grants_nothing() {
    let mut shop = new_shop(wallet(1_000), usize::MAX);
    let listing = *shop
        .sc_catalogue()
        .listing(REALMS_ROLL as u32)
        .expect("Realms Roll is listed");
    let shelf = shop.sc_catalogue().config().boombox_shelf.expect("shelf");
    assert_eq!(
        listing.group, shelf.category_group_id,
        "listed under Boombox"
    );
    let before = snapshot(&shop);
    let packet = buy_item(301, REALMS_ROLL);
    let outcome = shop.handle(&packet).expect("order parses");
    assert_eq!(outcome.result_code, station_cash::ORDER_SUCCESS);
    assert!(outcome.changed);
    assert_eq!(shop.sc_wallet().balance, 1_000 - listing.price);
    assert_eq!(
        count_definition(shop.inventory().state(), REALMS_ROLL),
        count_definition(&before.0, REALMS_ROLL) + 1,
        "exactly one boombox"
    );
    assert_eq!(shop.inventory().state().coins, before.0.coins);
    assert_eq!(shop.sc_wallet().orders.len(), 1);
    let once = snapshot(&shop);

    let replay = shop.handle(&packet).expect("replay parses");
    assert_eq!(replay.result_code, station_cash::ORDER_SUCCESS);
    assert!(!replay.changed);
    assert_eq!(
        snapshot(&shop),
        once,
        "a replayed order grants and debits nothing"
    );

    // F656: only rides are single-copy. No client item definition marks a boombox unique
    // (the Realms Roll row has MaxStackSize 1 and no unique flag), so a second order with a fresh tracking id is a second
    // sale: one more debit, one more boombox.
    let second = shop
        .handle(&buy_item(302, REALMS_ROLL))
        .expect("order parses");
    assert_eq!(second.result_code, station_cash::ORDER_SUCCESS);
    assert!(second.changed);
    assert_eq!(shop.sc_wallet().balance, 1_000 - 2 * listing.price);
    assert_eq!(
        count_definition(shop.inventory().state(), REALMS_ROLL),
        count_definition(&before.0, REALMS_ROLL) + 2,
        "a second copy"
    );
}

/// T495: a refused boombox order changes nothing, and a boombox the wiki does not sell for
/// Station Cash is not a listing.
#[test]
fn boombox_refusals_change_nothing() {
    let price = new_shop(wallet(0), usize::MAX)
        .sc_catalogue()
        .listing(REALMS_ROLL as u32)
        .expect("listed")
        .price;
    let mut cases = vec![
        (
            "insufficient",
            price - 1,
            usize::MAX,
            REALMS_ROLL,
            station_cash::ORDER_INSUFFICIENT_SC,
        ),
        (
            "full bag",
            1_000,
            0,
            REALMS_ROLL,
            station_cash::ORDER_FAILURE,
        ),
    ];
    for item in REWARD_ONLY_BOOMBOXES {
        cases.push((
            "reward only",
            1_000,
            usize::MAX,
            item,
            station_cash::ORDER_FAILURE,
        ));
    }
    for (label, balance, capacity, item, result) in cases {
        let mut shop = new_shop(wallet(balance), capacity);
        let before = snapshot(&shop);
        let outcome = shop.handle(&buy_item(310, item)).expect("order parses");
        assert_eq!(outcome.result_code, result, "{label} {item}");
        assert!(!outcome.changed, "{label} {item}");
        assert_eq!(snapshot(&shop), before, "{label} {item}");
    }
    let shop = new_shop(wallet(0), usize::MAX);
    for item in REWARD_ONLY_BOOMBOXES {
        assert!(
            shop.sc_catalogue().listing(item as u32).is_none(),
            "{item} is not sold"
        );
    }
}

/// Build a shop from the shipped `stores.json` after `edit`; the edited copy lives in `%TEMP%`.
fn shop_after(name: &str, edit: impl FnOnce(&mut Value)) -> std::io::Result<CoinShop> {
    let mut stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("stores.json reads"))
            .expect("stores.json parses");
    edit(&mut stores);
    let directory = std::env::temp_dir().join("op-T495").join(name);
    std::fs::create_dir_all(&directory).expect("temp directory");
    let path = directory.join("stores.json");
    std::fs::write(&path, serde_json::to_vec(&stores).expect("serializes")).expect("writes");
    let loaded = StoreCatalogue::load(&path);
    std::fs::remove_dir_all(&directory).expect("cleans up");
    let content = inventory_content();
    let inventory = Inventory::load(&content, None).expect("item content loads");
    CoinShop::new(
        &content,
        inventory,
        loaded?,
        usize::MAX,
        None,
        Wallet::default(),
    )
}

fn row_mut(stores: &mut Value, item: i32) -> &mut Value {
    stores["items"]
        .as_array_mut()
        .expect("rows")
        .iter_mut()
        .find(|row| row["item_id"] == item)
        .expect("row")
}

/// T495: bad boombox content is refused when the shop loads, never at purchase.
#[test]
fn bad_boombox_content_is_refused_at_load() {
    shop_after("good", |_| {}).expect("the shipped content loads");
    type Edit = Box<dyn FnOnce(&mut Value)>;
    let refused: [(&str, Edit); 4] = [
        (
            "no-shelf",
            Box::new(|s| s["station_cash"]["boombox_shelf"] = Value::Null),
        ),
        (
            "same-group",
            Box::new(|s| s["station_cash"]["boombox_shelf"]["category_group_id"] = 1.into()),
        ),
        (
            "overflow",
            Box::new(|s| row_mut(s, REALMS_ROLL)["sc_price"] = 500_000.into()),
        ),
        (
            "unpriced-tag",
            Box::new(|s| row_mut(s, REWARD_ONLY_BOOMBOXES[0])["boombox"] = true.into()),
        ),
    ];
    for (name, edit) in refused {
        assert!(
            shop_after(name, edit).is_err(),
            "{name} must be refused at load"
        );
    }
}

/// T526: a ride row's type must be one of the four keys, and a tagged row needs its category
/// configured. Bad content stops the store at load, never at a sale.
#[test]
fn bad_ride_category_content_is_refused_at_load() {
    shop_after("ride-category-good", |_| {}).expect("the shipped content loads");
    // The brief's canary, at `StoreCatalogue::load` itself: an unknown type is not a key.
    let mut stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("reads")).expect("parses");
    row_mut(&mut stores, RIDE_ITEM)["ride_category"] = "flying".into();
    let directory = std::env::temp_dir()
        .join("op-T526")
        .join("unknown-category");
    std::fs::create_dir_all(&directory).expect("temp directory");
    let path = directory.join("stores.json");
    std::fs::write(&path, serde_json::to_vec(&stores).expect("serializes")).expect("writes");
    let loaded = StoreCatalogue::load(&path);
    std::fs::remove_dir_all(&directory).expect("cleans up");
    assert!(
        loaded.is_err(),
        "ride_category flying must fail StoreCatalogue::load"
    );
    type Edit = Box<dyn FnOnce(&mut Value)>;
    let refused: [(&str, Edit); 3] = [
        (
            "category-not-configured",
            Box::new(|s| s["station_cash"]["ride_categories"] = Value::Array(Vec::new())),
        ),
        (
            "three-categories",
            Box::new(|s| {
                s["station_cash"]["ride_categories"]
                    .as_array_mut()
                    .expect("children")
                    .pop();
            }),
        ),
        (
            "reserved-id",
            Box::new(|s| s["station_cash"]["ride_categories"][0]["category_id"] = 1.into()),
        ),
    ];
    for (name, edit) in refused {
        assert!(
            shop_after(name, edit).is_err(),
            "{name} must be refused at load"
        );
    }
}

/// T526: a configured child with no bundles is still sent in 66/7 (count 0) and 66/6. The client
/// recomputes a category's count and deletes a zero-count tree node (`00c402b0` chain, T380), so
/// an empty child is harmless; its untagged ride rows list under the Rides parent again. T657:
/// the T528 Training Saddle is not a ride (item type 80001) and no item shelf names it, so it
/// cannot be untagged; it moves to another child to leave Winged empty.
#[test]
fn an_empty_ride_child_is_sent_and_lists_nothing() {
    let shop = shop_after("empty-winged", |s| {
        for row in s["items"].as_array_mut().expect("rows") {
            if row["ride_category"] != "winged" {
                continue;
            }
            if row["price_provenance"]["ride_id"].is_u64() {
                row.as_object_mut()
                    .expect("row object")
                    .remove("ride_category");
            } else {
                row["ride_category"] = "mechanical".into();
            }
        }
    })
    .expect("content loads");
    let config = shop.sc_catalogue().config();
    assert_eq!(
        shop.sc_catalogue()
            .listing(RIDE_ITEM as u32)
            .expect("the sample ride is listed")
            .group,
        config.category_group_id,
        "an untagged row lists under Rides"
    );
    assert!(
        shop.sc_catalogue()
            .listings()
            .iter()
            .all(|listing| listing.group != 2),
        "no bundle is keyed to the empty Winged child"
    );
    let packets = station_cash::catalogue_packets(shop.sc_catalogue());
    let tree = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 7, 0]))
        .expect("66/7 category tree");
    let mut reader = Reader { bytes: tree, at: 4 };
    let mut counts = BTreeMap::new();
    for _ in 0..reader.u32() {
        reader.u32();
        let id = reader.u32();
        reader.u32();
        reader.string();
        reader.string();
        reader.u32();
        counts.insert(id, reader.u32());
        reader.u32();
    }
    reader.u8();
    assert_eq!(reader.at, tree.len(), "66/7 trailing bytes");
    assert_eq!(counts.get(&2), Some(&0), "the empty child is sent with 0");
    let groups = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 6, 0]))
        .expect("66/6 category groups");
    let mut reader = Reader {
        bytes: groups,
        at: 4,
    };
    let mut categories_of = BTreeMap::new();
    for _ in 0..reader.u32() {
        let key = reader.u32();
        reader.u32();
        let ids: Vec<u32> = (0..reader.u32()).map(|_| reader.u32()).collect();
        categories_of.insert(key, ids);
    }
    assert_eq!(reader.at, groups.len(), "66/6 trailing bytes");
    assert_eq!(
        categories_of.get(&2),
        Some(&vec![2]),
        "the empty child still has its 66/6 group"
    );
}

/// F719: a ride row with no `ride_category` tag lists under Rides even when its row carries a
/// shelved item category (101 is the Clothes shelf's); it never lands on an item shelf.
#[test]
fn an_untagged_ride_with_a_shelved_item_category_still_lists_under_rides() {
    let shop = shop_after("ride-with-shelved-category", |s| {
        let row = row_mut(s, RIDE_ITEM);
        row.as_object_mut()
            .expect("row object")
            .remove("ride_category");
        row["category_id"] = 101.into();
    })
    .expect("content loads");
    let catalogue = shop.sc_catalogue();
    let ride = catalogue.listing(RIDE_ITEM as u32).expect("ride is listed");
    assert_eq!(ride.group, catalogue.config().category_group_id, "Rides");
    let clothes = catalogue
        .config()
        .item_shelves
        .iter()
        .find(|shelf| shelf.item_category_ids.contains(&101))
        .expect("the clothes shelf");
    assert!(
        catalogue
            .listings()
            .iter()
            .filter(|listing| listing.group == clothes.category_id)
            .all(|listing| listing.item_id != RIDE_ITEM),
        "the ride is not under Clothes"
    );
}

/// T657: the category table is content. A listed row no shelf names, a shelf list with a
/// duplicate item category, and a missing table are refused when the shop loads, never at a sale.
#[test]
fn bad_item_shelf_content_is_refused_at_load() {
    shop_after("item-shelf-good", |_| {}).expect("the shipped content loads");
    type Edit = Box<dyn FnOnce(&mut Value)>;
    let refused: [(&str, Edit); 5] = [
        (
            "no-shelves",
            Box::new(|s| s["station_cash"]["item_shelves"] = Value::Array(Vec::new())),
        ),
        (
            "duplicate-item-category",
            Box::new(|s| {
                // 101 is also the clothes shelf's; 6 stays so the build would still succeed if
                // only the duplicate check were missing.
                s["station_cash"]["item_shelves"][1]["item_category_ids"] =
                    serde_json::json!([6, 101]);
            }),
        ),
        (
            "unmapped-item-category",
            Box::new(|s| row_mut(s, 41)["category_id"] = 999.into()),
        ),
        (
            // 1 and 2 are the Rides and Boombox top-level orders (F719).
            "shelf-order-collides-with-rides",
            Box::new(|s| s["station_cash"]["item_shelves"][0]["display_order"] = 1.into()),
        ),
        (
            "shelf-order-collides-with-boombox",
            Box::new(|s| s["station_cash"]["item_shelves"][0]["display_order"] = 2.into()),
        ),
    ];
    for (name, edit) in refused {
        assert!(
            shop_after(name, edit).is_err(),
            "{name} must be refused at load"
        );
    }
    // A package with Station Cash off keeps the old payment-free behaviour even when it carries
    // priced rows: no catalogue is sent, so the category table is not required.
    shop_after("item-shelf-disabled", |s| {
        s["station_cash"]["enabled"] = false.into();
        s["station_cash"]["item_shelves"] = Value::Array(Vec::new());
    })
    .expect("a disabled Marketplace keeps an older content set loading");
}

/// T495: every boombox the store tags is a catalogued client boombox (`boomboxes.json`), none is
/// a reward-only box, and a wiki-priced row's number is on the cited wiki page revision.
#[test]
fn boombox_rows_match_the_catalogue_and_their_wiki_pages() {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let catalogued: Vec<i64> = serde_json::from_slice::<Value>(
        &std::fs::read(content.join("boomboxes.json")).expect("boomboxes.json reads"),
    )
    .expect("parses")["boomboxes"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| row["item_id"].as_i64().expect("item id"))
        .collect();
    let stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("reads")).expect("parses");
    let wiki = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/freerealms-wiki/wikitext");
    let mut tagged = 0;
    for row in stores["items"].as_array().expect("rows") {
        let item = row["item_id"].as_i64().expect("item id");
        if row["boombox"].as_bool() != Some(true) {
            continue;
        }
        tagged += 1;
        assert!(
            catalogued.contains(&item),
            "{item} is not a catalogued boombox"
        );
        assert!(
            !REWARD_ONLY_BOOMBOXES.contains(&(item as i32)),
            "{item} is reward only"
        );
        let provenance = &row["price_provenance"];
        if provenance["sc_basis"] != "W" {
            assert!(
                provenance["basis"]
                    .as_str()
                    .is_some_and(|text| text.starts_with("(R)")),
                "{item}: an estimated price is marked (R)"
            );
            continue;
        }
        let price = row["sc_price"].as_u64().expect("price");
        let revision = format!("-{}-", provenance["wiki_rev_id"]);
        let page = std::fs::read_dir(&wiki)
            .expect("wiki wikitext")
            .map(|entry| entry.expect("entry").path())
            .find(|path| path.to_string_lossy().contains(&revision))
            .unwrap_or_else(|| panic!("{item}: wiki revision {revision} is not in the harvest"));
        let text = String::from_utf8_lossy(&std::fs::read(page).expect("page reads")).into_owned();
        assert!(
            text.contains(&format!("SC|{price}}}")) || text.contains(&format!("{price} SC")),
            "{item}: the wiki page does not say {price} SC"
        );
    }
    assert_eq!(tagged, 11);
}
