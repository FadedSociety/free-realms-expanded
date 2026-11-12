//! B2 event shops (2026-11-11 event audits; owner: everything available, limited stock).
//!
//! Event stock is sold only while its `live_event` runs (the calendar's one gate), on the
//! currency and at the price `stores.json` gives it (`scripts/event_shops.py` writes them from the
//! audits), and a limited row keeps one shared stock counter per event run plus an optional
//! per-player cap.
use osfr_tools::{
    character_store::CharacterState,
    coin_shop::{self, CoinShop, StoreCatalogue},
    gameplay::Gameplay,
    inventory::Inventory,
    shop_stock::{Ledger, Limit},
    station_cash::{self, Wallet},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// 2026-10-15 12:00 UTC: inside Halloween (Oct 1 - Nov 12).
const OCTOBER: u64 = 1_792_065_600_000;
/// 2026-07-15 12:00 UTC: Summer Camp only.
const JULY: u64 = 1_784_116_800_000;
/// Graveyard Fence: a Halloween Coin Shop housing piece (fanbyte: only in the Coin Shop).
const FENCE: i32 = 16_840;
/// Broomstick: a ride the fandom Rides table sells only during the Super Spooktacular.
const BROOMSTICK: i32 = 79_433;
/// Spooky Chair: one of B3's new Halloween item rows, priced by `scripts/event_shops.py`.
const SPOOKY_CHAIR: i32 = 22_940;
/// Camp Chatty Banner: Summer Camp Coin Shop stock, year-round (the camp is permanent).
const BANNER: i32 = 14_299;
/// LIMITED EDITION Werewolf Outfit (client NameId 119208): its three sold rows share one stock.
const LIMITED: [i32; 3] = [69_193, 69_194, 69_195];
const LIMITED_GROUP: &str = "limited_edition_werewolf_outfit";

fn release() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release")
}

fn stores() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json")
}

fn rows() -> BTreeMap<i64, Value> {
    let stores: Value = serde_json::from_slice(&fs::read(stores()).unwrap()).unwrap();
    (stores["items"].as_array().unwrap().iter())
        .map(|row| (row["item_id"].as_i64().unwrap(), row.clone()))
        .collect()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("osfr-b2-shops-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn shop_from(catalogue: &Path, coins: u32, sc: u32) -> CoinShop {
    let directory = release();
    let mut state = Inventory::load(&directory, None).unwrap().state().clone();
    state.coins = coins;
    let inventory = Inventory::load(&directory, Some(state)).unwrap();
    let catalogue = StoreCatalogue::load(catalogue).expect("store catalogue loads");
    let wallet = Wallet {
        balance: sc,
        ..Wallet::default()
    };
    CoinShop::new(&directory, inventory, catalogue, usize::MAX, None, wallet).unwrap()
}

/// The calendar's word, as `Gameplay` passes it: `halloween` runs as run `run`, or nothing runs.
fn halloween(shop: &mut CoinShop, run: Option<u64>) {
    shop.follow_events(|key| run.filter(|_| key == "halloween"));
}

fn coin_buy(item: i32, quantity: u32) -> Vec<u8> {
    let mut packet = vec![165, 0, 4, 0];
    packet.extend(0u64.to_le_bytes());
    packet.extend(0u32.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(0i32.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// The client's 66/3 order (`00b3fc60`/`00b3af80`) for one bundle of store 1.
fn order(tracking: u32, bundle: i32) -> Vec<u8> {
    let mut bytes = vec![66, 0, 3, 0];
    put_string(&mut bytes, "");
    put_string(&mut bytes, "");
    bytes.extend(tracking.to_le_bytes());
    put_string(&mut bytes, "en_US");
    for _ in 0..3 {
        put_string(&mut bytes, "");
    }
    bytes.extend(1u32.to_le_bytes());
    bytes.extend((bundle as u32).to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(1u32.to_le_bytes());
    put_string(&mut bytes, "0");
    put_string(&mut bytes, "");
    bytes
}

fn says(replies: &[Vec<u8>], text: &str) -> bool {
    replies
        .iter()
        .any(|reply| String::from_utf8_lossy(reply).contains(text))
}

fn contains_u32(packet: &[u8], value: u32) -> bool {
    packet
        .windows(4)
        .any(|window| window == value.to_le_bytes())
}

#[test]
fn an_event_row_is_hidden_out_of_season_and_offered_in_season() {
    let mut shop = shop_from(&stores(), 100_000, 100_000);
    // No calendar has spoken yet: a bare shop sells everything, as before.
    assert!(shop.sale_block(FENCE).is_none());
    assert!(shop.sc_catalogue().listing(BROOMSTICK as u32).is_some());

    halloween(&mut shop, None);
    assert_eq!(shop.sale_block(FENCE), Some("event stock is out of season"));
    assert!(shop.sale_block(BANNER).is_none(), "untagged stock stays");
    assert!(
        shop.sc_catalogue().listing(BROOMSTICK as u32).is_none(),
        "off the Marketplace"
    );
    let coins = shop.inventory().state().coins;
    let refused = shop.handle(&coin_buy(FENCE, 1)).unwrap();
    assert_eq!(refused.result_code, coin_shop::RESULT_INVALID_ITEM);
    assert!(!refused.changed);
    let refused = shop.handle(&order(1, BROOMSTICK)).unwrap();
    assert_eq!(
        refused.result_code,
        station_cash::ORDER_FAILURE,
        "an out-of-season order"
    );
    assert!(!refused.changed);
    assert_eq!(shop.inventory().state().coins, coins);

    halloween(&mut shop, Some(1));
    assert!(shop.sale_block(FENCE).is_none());
    let bought = shop.handle(&coin_buy(FENCE, 1)).unwrap();
    assert_eq!(bought.result_code, coin_shop::RESULT_SUCCESS, "in season");
    assert_eq!(
        shop.inventory().state().coins,
        coins - 50,
        "the Coin Shop price"
    );
    let listing = *shop
        .sc_catalogue()
        .listing(BROOMSTICK as u32)
        .expect("listed again");
    assert_eq!(listing.price, 137, "549 SC at the 0.25 scale");
    // A row B3's item table added (the Spooky Chair, 299 SC) lists on the Housing shelf in season.
    let chair = *shop
        .sc_catalogue()
        .listing(SPOOKY_CHAIR as u32)
        .expect("B3's row is listed");
    assert_eq!(chair.price, 75);
    halloween(&mut shop, None);
    assert!(
        shop.sc_catalogue().listing(SPOOKY_CHAIR as u32).is_none(),
        "and leaves after"
    );
}

/// The real entry path: a session's calendar (dates and hand toggles, `set_world_dir`) drives
/// the shop, and the store screen's 66/10 lists the Halloween ride only in October.
#[test]
fn the_session_calendar_gates_the_marketplace_screen() {
    let character = CharacterState {
        guid: 7,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [1.0, 0.0, 0.0],
    };
    for (clock, listed) in [(OCTOBER, true), (JULY, false)] {
        let live = Scratch::new(&format!("live-{clock}"));
        let mut game = Gameplay::load(&release(), None).unwrap();
        game.set_event_clock(Some(clock));
        game.set_world_dir(Some(live.0.clone())).unwrap();
        let out = game
            .handle_in_world(&[66, 0, 10, 0], &character, 1_000)
            .unwrap();
        let store = (out.replies.iter())
            .find(|packet| packet.starts_with(&[66, 0, 5, 0]))
            .expect("the store's bundles");
        assert_eq!(
            contains_u32(store, BROOMSTICK as u32),
            listed,
            "clock {clock}"
        );
        // A year-round ride (76856, the Station Cash suite's ride) is listed either way.
        assert!(contains_u32(store, 76_856), "clock {clock}");
    }
}

#[test]
fn currency_and_price_come_from_the_audited_rows() {
    let rows = rows();
    let coins = |id: i64| rows[&id]["purchase_price_coins"].as_u64();
    let sc = |id: i64| {
        rows[&id]["sc_price"]
            .as_u64()
            .filter(|_| rows[&id]["earn_only"] != true)
    };
    let event = |id: i64| rows[&id]["live_event"].as_str().map(str::to_owned);
    // Coin Shop rows back on coins at the Coin Shop price, not Station Cash.
    for (id, price) in [
        (16_840, 50),
        (16_858, 5_000),
        (16_039, 750),
        (14_299, 50),
        (14_308, 20_000),
    ] {
        assert_eq!(
            (coins(id), sc(id)),
            (Some(price), None),
            "{id} is a Coin Shop row"
        );
    }
    // A Coin Shop row no coin window can show (not in the client's captured Coin Shop list, no
    // vendor) stays on the Marketplace, so it stays buyable, and says why.
    for id in [76_822, 36_154, 77_451] {
        assert_eq!(coins(id), None, "{id}");
        assert!(sc(id).is_some(), "{id} keeps its Marketplace price");
        let basis = rows[&id]["price_provenance"]["basis"].as_str().unwrap();
        assert!(basis.contains("kept on the Marketplace"), "{id}: {basis}");
    }
    // Wiki Coin Shop prices: the Halloween and Snow Days pet pals, the Lovely T-Shirt.
    for (id, price) in [
        (17_520, 45_000),
        (30_589, 44_999),
        (10_469, 9_000),
        (17_521, 45_000),
        (76_826, 34),
    ] {
        assert_eq!(coins(id), Some(price), "{id}");
    }
    // Marketplace rows off coins.
    for (id, price) in [
        (69_836, 199),
        (69_158, 199),
        (76_548, 199),
        (4_826, 250),
        (7_882, 349),
        (77_460, 49),
    ] {
        assert_eq!(
            (coins(id), sc(id)),
            (None, Some(price)),
            "{id} is a Marketplace row"
        );
        assert_eq!(rows[&id]["no_sale"], true);
    }
    // Reward-only items are not sold; a chest ride keeps its value as earn-only.
    for id in [9_091, 77_450, 77_445, 76_707, 76_538, 76_592, 14_309] {
        assert_eq!((coins(id), sc(id)), (None, None), "{id} is only a reward");
    }
    assert_eq!(rows[&79_451]["earn_only"], true);
    // Event stock is tagged with its event; the permanent camp is not.
    assert_eq!(event(16_840).as_deref(), Some("halloween"));
    assert_eq!(event(41).as_deref(), Some("turkey_dance"));
    assert_eq!(event(76_613).as_deref(), Some("childs_play"));
    assert_eq!(event(14_299), None);
    assert_eq!(event(7_882), None, "the regular Bat colours are year-round");
}

#[test]
fn every_event_tag_and_limit_is_well_formed() {
    let calendar: Value =
        serde_json::from_slice(&fs::read(stores().with_file_name("event_calendar.json")).unwrap())
            .unwrap();
    let keys: BTreeSet<&str> = (calendar["events"].as_array().unwrap().iter())
        .map(|event| event["key"].as_str().unwrap())
        .collect();
    let rows = rows();
    let mut tagged = 0;
    for (id, row) in &rows {
        if let Some(key) = row["live_event"].as_str() {
            tagged += 1;
            assert!(
                keys.contains(key),
                "{id}: {key} is not an event_calendar.json key"
            );
            assert!(
                row["source_tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|tag| tag == "B2S"),
                "{id}: an event row names its audit (B2S)"
            );
        }
        if row["stock_per_event"].is_u64() {
            assert!(
                row["live_event"].is_string(),
                "{id}: a per-event stock needs its event"
            );
        }
    }
    assert!(tagged >= 260, "the audited event stock is tagged: {tagged}");
    for id in LIMITED {
        assert_eq!(rows[&(id as i64)]["stock_per_event"], 200);
        assert_eq!(rows[&(id as i64)]["stock_group"], LIMITED_GROUP);
        assert_eq!(rows[&(id as i64)]["live_event"], "halloween");
    }
}

/// The LIMITED EDITION Werewolf Outfit: 200 per Halloween across all its rows, and a new
/// Halloween (a new run) starts the count again.
#[test]
fn a_stock_cap_is_shared_across_rows_and_resets_at_the_next_event() {
    let scratch = Scratch::new("stock");
    let ledger_path = scratch.0.join("shop-stock.json");
    let limit = Limit {
        group: LIMITED_GROUP.into(),
        event: Some("halloween".into()),
        per_event: Some(200),
        per_player: None,
    };
    // 199 already sold this run, by other characters.
    let mut ledger = Ledger::default();
    ledger.take(&limit, 1, 99, 199);
    ledger.save(&ledger_path).unwrap();

    let mut shop = shop_from(&stores(), 0, 10_000);
    shop.set_stock_file(ledger_path.clone());
    shop.set_buyer(7);
    halloween(&mut shop, Some(1));
    let bought = shop.handle(&order(1, LIMITED[1])).unwrap();
    assert_eq!(bought.result_code, station_cash::ORDER_SUCCESS, "the 200th");
    assert!(bought.changed);
    assert_eq!(
        Ledger::load(&ledger_path).unwrap().remaining(&limit, 1),
        Some(0)
    );

    let wallet = shop.sc_wallet().balance;
    let refused = shop.handle(&order(2, LIMITED[2])).unwrap();
    assert_eq!(
        refused.result_code,
        station_cash::ORDER_FAILURE,
        "another colour shares it"
    );
    assert!(!refused.changed);
    assert!(
        says(&refused.replies, "sold out"),
        "the player is told it is gone"
    );
    assert_eq!(shop.sc_wallet().balance, wallet, "nothing was paid");

    // The next Halloween is a new run: the counter starts again.
    halloween(&mut shop, Some(2));
    let bought = shop.handle(&order(3, LIMITED[0])).unwrap();
    assert_eq!(
        bought.result_code,
        station_cash::ORDER_SUCCESS,
        "next year's first"
    );
    let ledger = Ledger::load(&ledger_path).unwrap();
    assert_eq!(ledger.remaining(&limit, 2), Some(199));
    assert_eq!(ledger.counters[LIMITED_GROUP].players[&7], 1);
}

/// `per_player` counts one character's buys; another character still may.
#[test]
fn a_per_player_cap_refuses_the_second_buy() {
    let scratch = Scratch::new("per-player");
    let mut stores: Value = serde_json::from_slice(&fs::read(stores()).unwrap()).unwrap();
    for row in stores["items"].as_array_mut().unwrap() {
        if row["item_id"] == FENCE {
            row["per_player"] = 1.into();
        }
    }
    let path = scratch.0.join("stores.json");
    fs::write(&path, serde_json::to_vec(&stores).unwrap()).unwrap();
    let mut shop = shop_from(&path, 10_000, 0);
    shop.set_stock_file(scratch.0.join("shop-stock.json"));
    halloween(&mut shop, Some(1));
    shop.set_buyer(7);
    let first = shop.handle(&coin_buy(FENCE, 1)).unwrap();
    assert_eq!(first.result_code, coin_shop::RESULT_SUCCESS);
    let coins = shop.inventory().state().coins;
    let second = shop.handle(&coin_buy(FENCE, 1)).unwrap();
    assert_eq!(
        second.result_code,
        coin_shop::RESULT_INVALID_QUANTITY,
        "a second for the same buyer"
    );
    assert!(!second.changed);
    assert!(says(&second.replies, "as many of that item"), "told why");
    assert_eq!(shop.inventory().state().coins, coins, "nothing was paid");
    shop.set_buyer(8);
    let other = shop.handle(&coin_buy(FENCE, 1)).unwrap();
    assert_eq!(
        other.result_code,
        coin_shop::RESULT_SUCCESS,
        "another character may"
    );
}

/// A malformed limit is refused when the catalogue loads.
#[test]
fn a_malformed_event_or_stock_field_is_refused_at_load() {
    let scratch = Scratch::new("refused");
    let edits: [(&str, fn(&mut Value)); 4] = [
        ("stock without an event", |row| {
            row.as_object_mut().unwrap().remove("live_event");
            row["stock_per_event"] = 5.into();
        }),
        ("zero stock", |row| row["stock_per_event"] = 0.into()),
        ("a group without a cap", |row| {
            row["stock_group"] = "lonely".into()
        }),
        ("an empty event key", |row| row["live_event"] = "".into()),
    ];
    for (label, edit) in edits {
        let mut stores: Value = serde_json::from_slice(&fs::read(stores()).unwrap()).unwrap();
        for row in stores["items"].as_array_mut().unwrap() {
            if row["item_id"] == FENCE {
                edit(row);
            }
        }
        let path = scratch.0.join("stores.json");
        fs::write(&path, serde_json::to_vec(&stores).unwrap()).unwrap();
        assert!(
            StoreCatalogue::load(&path).is_err(),
            "{label} must be refused"
        );
    }
}
