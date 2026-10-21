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
        orders: Vec::new(),
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
    // 66/4 full layout: result, tracking id, order-id string, discount, total.
    let reply = outcome.replies.last().expect("order response");
    assert_eq!(&reply[..4], &[66, 0, 4, 0]);
    assert_eq!(u32::from_le_bytes(reply[4..8].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(reply[8..12].try_into().unwrap()), 71);
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
            .any(|reply| reply.starts_with(&[66, 0, 4, 0, 1, 0, 0, 0]))
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
        .handle_marketplace(&buy(88), true)
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
    assert_eq!(u32::from_le_bytes(reply[4..8].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(reply[8..12].try_into().unwrap()), 91);
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
    let items: Vec<i32> = shop
        .sc_catalogue()
        .listings()
        .iter()
        .chain(shop.sc_catalogue().earn_only())
        .map(|listing| listing.item_id)
        .collect();
    assert_eq!(items.len(), 54, "every ride has a catalogue row");
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
        assert_eq!(reader.u32(), config.category_group_id);
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
        decoded.insert(key, price);
    }
    assert_eq!(
        reader.at,
        store.len(),
        "the store packet has trailing bytes"
    );
    assert_eq!(decoded.len(), 52, "the shipped listed ride count");

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
        decoded, expected,
        "the store lists the shipped rides at their SC prices"
    );
}

/// Walk a buffer exactly as the client's 66/5 reader does, refusing a short or trailing byte.
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
