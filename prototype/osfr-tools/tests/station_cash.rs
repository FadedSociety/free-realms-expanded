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
use std::{collections::BTreeMap, path::PathBuf};

/// A listed ride: item 76856 (the five-hundred-and-fifty Station Cash mount) is 549 SC of
/// provenance, so the shipped 0.25 scale lists it at 137.
const RIDE_ITEM: i32 = 76_856;
const RIDE_PRICE: u32 = 137;
const EARN_ONLY_ITEM: i32 = 79_454;
const STORE: u32 = 1;

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
    // items T638 moved from their Legacy coin price.
    assert_eq!(
        decoded.len(),
        808,
        "the shipped listed ride, deed and wiki item count"
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
    for _ in 0..reader.u32() {
        reader.u32();
        let id = reader.u32();
        names.insert(id, reader.u32());
        reader.string();
        reader.string();
        parents.insert(id, reader.u32());
        reader.u32();
        reader.u32();
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
    let under_rides = |mut category: u32| loop {
        if category == RIDES {
            return true;
        }
        match parents.get(&category) {
            Some(&parent) if parent != 0 => category = parent,
            _ => return false,
        }
    };
    let listed: BTreeMap<u32, u32> = decoded
        .iter()
        .filter(|(_, (_, group))| {
            categories_of
                .get(group)
                .is_some_and(|ids| ids.iter().any(|id| under_rides(*id)))
        })
        .map(|(key, (price, _))| (*key, *price))
        .collect();

    let stores: Value =
        serde_json::from_slice(&std::fs::read(stores()).expect("stores.json reads"))
            .expect("stores.json parses");
    let scale = stores["station_cash"]["sc_price_scale"]
        .as_f64()
        .expect("scale");
    let expected: BTreeMap<u32, u32> = stores["items"]
        .as_array()
        .expect("store rows")
        .iter()
        .filter(|row| row["earn_only"].as_bool() != Some(true))
        .filter_map(|row| {
            let price = row["sc_price"].as_u64()?;
            Some((
                u32::try_from(row["item_id"].as_i64()?).ok()?,
                (price as f64 * scale).round() as u32,
            ))
        })
        .collect();
    assert_eq!(
        listed, expected,
        "Rides lists exactly the shipped non-earn-only rides at their SC prices"
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
/// record misaligns every following bundle, so the last byte must be consumed exactly.
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
    // tail 44, `00b41860` continuation 31). Assert the exact reader size as well as walking it, so
    // a field added to or dropped from the writer cannot pass by ending where the test stops.
    let listings = shop.sc_catalogue().listings();
    assert_eq!(store.len(), 36 + listings.len() * 185, "66/5 packet size");
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
        assert_eq!(reader.u32(), listing.bundle_id, "bundle map key");
        // MarketingBundleDefinition (`00b3da50`).
        assert_eq!(reader.u32(), listing.bundle_id, "bundle id");
        assert_eq!(reader.u32(), listing.name_id, "name id");
        reader.u32();
        reader.u32();
        reader.string();
        reader.string();
        reader.u8();
        reader.string();
        assert_eq!(reader.u32(), config.currency_type);
        assert_eq!(reader.u32(), listing.price, "listed price");
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
        // StoreBundleDefinition tail (`00b3fa00`): StoreId, CategoryGroupId, visible, then eight
        // u32 and three bytes.
        assert_eq!(reader.u32(), config.store_id);
        assert_eq!(reader.u32(), config.category_group_id);
        reader.u8();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u32();
        reader.u8();
        reader.u8();
        reader.u8();
        // `00b41860` continuation: u32, u32, u8, u32, u32, u32, u8, u8, u32, u32 (31 bytes).
        assert_eq!(reader.u32(), 0, "00b41860 field 1");
        assert_eq!(reader.u32(), 0, "00b41860 field 2");
        assert_eq!(reader.u8(), 0, "00b41860 field 3");
        assert_eq!(reader.u32(), 0, "00b41860 field 4");
        assert_eq!(reader.u32(), 0, "00b41860 field 5");
        assert_eq!(reader.u32(), 0, "00b41860 field 6");
        assert_eq!(reader.u8(), 0, "00b41860 field 7");
        assert_eq!(reader.u8(), 0, "00b41860 field 8");
        assert_eq!(reader.u32(), 0, "00b41860 field 9");
        assert_eq!(reader.u32(), 0, "00b41860 field 10");
    }
    assert_eq!(
        reader.at,
        store.len(),
        "the store packet has trailing unread bytes"
    );
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

    // 66/7: one counted category node, then the sorted byte.
    let tree = packets
        .iter()
        .find(|packet| packet.starts_with(&[66, 0, 7, 0]))
        .expect("66/7 category tree");
    let mut reader = Reader { bytes: tree, at: 0 };
    assert_eq!(reader.u16(), 66);
    assert_eq!(reader.u16(), 7);
    assert_eq!(reader.u32(), 1, "one category");
    assert_eq!(reader.u32(), config.category_id, "category map key");
    assert_eq!(reader.u32(), config.category_id, "category id");
    assert_eq!(reader.u32(), config.category_name_id, "category name id");
    reader.string();
    reader.string();
    assert_eq!(reader.u32(), 0, "parent id");
    assert_eq!(reader.u32() as usize, listings.len(), "category count");
    assert_eq!(reader.u32(), 1, "display order");
    assert_eq!(reader.u8(), 1, "sorted");
    assert_eq!(reader.at, tree.len(), "66/7 trailing bytes");

    // 66/6: one counted group with one category id.
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
    assert_eq!(reader.u32(), 1, "one category group");
    assert_eq!(reader.u32(), config.category_group_id, "group map key");
    assert_eq!(reader.u32(), config.category_group_id, "group id");
    assert_eq!(reader.u32(), 1, "one category id");
    assert_eq!(reader.u32(), config.category_id);
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
