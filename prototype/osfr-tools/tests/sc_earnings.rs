//! T229: Station Cash earned by verified completions and the earn-only ride awards.
//!
//! These tests cover the data table, the one-shots it decides (first activity of the day and the
//! first-clear ride), the Station Cash reward kind in the one reward applier and the wallet save.
//! The per-activity completion hooks are driven through their real handlers in the owning test
//! files (`kart_npc_entry`, `derby_route`, `chess_route`, `soccer_quest_credit`,
//! `starting_quest_chain`, `opening_cave_replay`, `thugamug_battle`); the first-clear ride is
//! driven through the gateway below.
//!
//! Every amount below is (R) content; no test copies the production decision logic.

use osfr_tools::{
    chat,
    gateway::Gateway,
    inventory::Inventory,
    profiles::ProfileState,
    rewards::{GrantFailure, RewardDisplay, RewardGrant, RewardState, apply_grant},
    station_cash::{self, EarnSource, EarnTable, RideOffer, Wallet},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture as fb;

static NEXT: AtomicU64 = AtomicU64::new(0);

const DAY: u64 = 20_000;
const CHROME_DRAKE: i32 = 79_454;
const POLAR_BEAR: i32 = 79_447;

fn table_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/sc_rewards.json")
}

fn shipped() -> EarnTable {
    EarnTable::load(&table_path())
        .expect("shipped earn table parses")
        .expect("shipped earn table exists")
}

fn scratch(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "osfr-sc-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// An empty inventory definition set, like the reward fixtures in `starting_quest_chain`.
fn inventory() -> Inventory {
    let directory = scratch("inventory");
    let zero = 0i32.to_le_bytes();
    fs::write(
        directory.join("inventory.json"),
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "player_guid": 1,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": 1
            },
            "profiles": [],
            "spans": [
                {"kind":"coins","offset":13,"length":4,"profile_id":0,"sha256":digest(&zero)},
                {"kind":"items","offset":17,"length":4,"profile_id":0,"sha256":digest(&zero)}
            ],
            "definitions": []
        }))
        .unwrap(),
    )
    .unwrap();
    let inventory = Inventory::load(&directory, None).expect("fixture inventory loads");
    let _ = fs::remove_dir_all(&directory);
    inventory
}

fn profiles() -> ProfileState {
    ProfileState {
        version: 1,
        active: 1,
        unlocked: BTreeSet::from([1]),
        experience: Default::default(),
    }
}

fn reward_state(wallet: Wallet, ceiling: u32) -> RewardState {
    RewardState::new(inventory(), profiles(), 100).with_wallet(wallet, ceiling)
}

fn sc_grant(sc: u32) -> RewardGrant {
    RewardGrant {
        coins: 0,
        items: Vec::new(),
        job_xp: Default::default(),
        sc,
        display: RewardDisplay {
            source_guid: 0,
            player_guid: 0,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn wallet_balance(packet: &[u8]) -> Option<u32> {
    (packet.len() == 26 && packet[..4] == [66, 0, 11, 0])
        .then(|| u32::from_le_bytes(packet[9..13].try_into().unwrap()))
}

fn capped_notice(packet: &[u8]) -> bool {
    packet.starts_with(&[15, 0, 3, 0]) && packet.windows(12).any(|w| w == &b"Station Cash"[..])
}

/// The amounts are tunable data (sanity bounds only, enforced by the loader); this pins only the
/// shape the brief asks for: earning is on, the hardest tier is a separate amount, Chrome Drake
/// is the first-clear ride and Polar Bear stays dormant.
#[test]
fn shipped_table_loads_with_the_earn_only_rides() {
    let table = shipped();
    assert!(table.enabled);
    let ride = table.first_clear_ride.expect("a first-clear ride ships");
    assert_eq!(ride.item_id, CHROME_DRAKE);
    assert_eq!(
        table.amount(EarnSource::DungeonClear {
            id: ride.dungeon_id
        }),
        table.hardest_dungeon_clear,
        "the first-clear dungeon is in the hardest tier"
    );
    assert_eq!(
        table
            .seasonal_rides
            .iter()
            .map(|ride| (ride.item_id, ride.enabled))
            .collect::<Vec<_>>(),
        vec![(POLAR_BEAR, false)]
    );
}

#[test]
fn earn_table_refuses_out_of_bounds_and_active_seasonal_rows() {
    let directory = scratch("bad-tables");
    let write = |name: &str, value: serde_json::Value| {
        fs::write(directory.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    let source = json!([{"kind":"reconstructed","evidence":"(R) synthetic bound test"}]);
    let base = json!({
        "schema_version": 1,
        "enabled": true,
        "kart_race": 3,
        "derby_match": 3,
        "soccer_match": 3,
        "chess_game": 2,
        "dungeon_clear": 10,
        "hardest_dungeon_clear": 25,
        "hardest_dungeons": [411],
        "quest_complete": 1,
        "daily_first_activity": 5,
        "source": source,
    });
    assert!(
        EarnTable::load(&directory.join("absent.json"))
            .unwrap()
            .is_none(),
        "a missing table keeps earning off"
    );

    let mut active = base.clone();
    active["seasonal_rides"] = json!([{"item_id": POLAR_BEAR, "event": "event", "enabled": true}]);
    write("active-seasonal.json", active);
    assert!(EarnTable::load(&directory.join("active-seasonal.json")).is_err());

    let mut huge = base.clone();
    huge["kart_race"] = json!(station_cash::MAX_EARN_SC + 1);
    write("huge.json", huge);
    assert!(EarnTable::load(&directory.join("huge.json")).is_err());

    let mut stray_ride = base.clone();
    stray_ride["first_clear_ride"] = json!({"dungeon_id": 999, "item_id": CHROME_DRAKE});
    write("stray-ride.json", stray_ride);
    assert!(
        EarnTable::load(&directory.join("stray-ride.json")).is_err(),
        "the first-clear ride must name a hardest dungeon"
    );

    let mut unknown = base.clone();
    unknown["mystery"] = json!(1);
    write("unknown.json", unknown);
    assert!(EarnTable::load(&directory.join("unknown.json")).is_err());

    let mut schema = base;
    schema["schema_version"] = json!(2);
    write("schema.json", schema);
    assert!(EarnTable::load(&directory.join("schema.json")).is_err());
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn first_activity_of_the_day_pays_the_bonus_once() {
    let table = shipped();
    let chess_win = EarnSource::ChessGame {
        won_or_drew: true,
        plies: 0,
    };
    let (chess, kart, bonus) = (
        table.amount(chess_win),
        table.amount(EarnSource::KartRace),
        table.daily_first_activity,
    );
    let mut wallet = Wallet::default();
    let first = table.earned(&mut wallet, chess_win, DAY, None);
    assert_eq!(
        first.sc,
        chess + bonus,
        "the first completion adds the daily bonus"
    );
    assert_eq!(wallet.last_activity_day, Some(DAY));
    let second = table.earned(&mut wallet, chess_win, DAY, None);
    assert_eq!(second.sc, chess, "the same day pays the base amount only");
    let next_day = table.earned(&mut wallet, EarnSource::KartRace, DAY + 1, None);
    assert_eq!(
        next_day.sc,
        kart + bonus,
        "a new UTC day pays the bonus again"
    );
    assert_eq!(wallet.last_activity_day, Some(DAY + 1));
}

#[test]
fn first_clear_ride_is_settled_once_and_an_owned_ride_pays_its_marketplace_value() {
    let table = shipped();
    let hardest = EarnSource::DungeonClear {
        id: table
            .first_clear_ride
            .expect("a first-clear ride ships")
            .dungeon_id,
    };
    let clear = table.amount(hardest);
    let mut wallet = Wallet {
        last_activity_day: Some(DAY),
        ..Wallet::default()
    };
    let offer = RideOffer {
        item_id: CHROME_DRAKE,
        sc_equivalent: 125,
        owned: false,
        room: true,
    };
    let first = table.earned(&mut wallet, hardest, DAY, Some(offer));
    assert_eq!(first.ride, Some(CHROME_DRAKE));
    assert_eq!(
        first.sc, clear,
        "the clear pays even when the ride is granted"
    );
    assert_eq!(wallet.earned_rides, vec![CHROME_DRAKE as u32]);

    let second = table.earned(&mut wallet, hardest, DAY, Some(offer));
    assert_eq!(
        second.ride, None,
        "a later clear never grants the ride again"
    );
    assert_eq!(
        second.sc, clear,
        "a later clear pays the dungeon clear only"
    );

    let mut owned = Wallet {
        last_activity_day: Some(DAY),
        ..Wallet::default()
    };
    let paid = table.earned(
        &mut owned,
        hardest,
        DAY,
        Some(RideOffer {
            owned: true,
            ..offer
        }),
    );
    assert_eq!(paid.ride, None);
    assert_eq!(
        paid.sc,
        clear + 125,
        "an owned ride pays its Station Cash value"
    );
    assert_eq!(
        owned.earned_rides,
        vec![CHROME_DRAKE as u32],
        "the award is consumed even when paid in Station Cash"
    );

    // A full bag pays the clear only and does not settle the ride, so a later clear grants it.
    let mut full = Wallet {
        last_activity_day: Some(DAY),
        ..Wallet::default()
    };
    let no_room = RideOffer {
        room: false,
        ..offer
    };
    let paid = table.earned(&mut full, hardest, DAY, Some(no_room));
    assert_eq!(paid.ride, None);
    assert_eq!(paid.sc, clear, "a full bag pays the clear only");
    assert!(paid.bag_full, "the player is told to make room");
    assert!(full.earned_rides.is_empty(), "the ride stays unsettled");
    let again = table.earned(&mut full, hardest, DAY, Some(no_room));
    assert!(again.bag_full && again.ride.is_none() && full.earned_rides.is_empty());
    let freed = table.earned(&mut full, hardest, DAY, Some(offer));
    assert_eq!(freed.ride, Some(CHROME_DRAKE), "room later grants the ride");
    assert_eq!(freed.sc, clear);
    assert!(!freed.bag_full);
    assert_eq!(full.earned_rides, vec![CHROME_DRAKE as u32]);
    let after = table.earned(&mut full, hardest, DAY, Some(offer));
    assert_eq!(after.ride, None, "the ride is granted once");

    // An owned ride keeps the SC-equivalent rule even with a full bag.
    let mut owned_full = Wallet {
        last_activity_day: Some(DAY),
        ..Wallet::default()
    };
    let paid = table.earned(
        &mut owned_full,
        hardest,
        DAY,
        Some(RideOffer {
            owned: true,
            ..no_room
        }),
    );
    assert_eq!(paid.sc, clear + 125);
    assert!(!paid.bag_full);
    assert_eq!(owned_full.earned_rides, vec![CHROME_DRAKE as u32]);
}

fn chess(won_or_drew: bool, plies: u32) -> EarnSource {
    EarnSource::ChessGame { won_or_drew, plies }
}

/// Chess pays only a game that reached a result through play (R): a win, a draw, or a loss after
/// `chess_min_plies`. An instant resign pays nothing, not even the daily bonus.
#[test]
fn chess_pays_only_a_game_that_reached_a_result_through_play() {
    let mut table = shipped();
    table.chess_min_plies = 30;
    let (game, bonus) = (table.chess_game, table.daily_first_activity);
    assert!(game > 0 && bonus > 0, "the shipped table pays chess");

    let mut wallet = Wallet::default();
    let resigned = table.earned(&mut wallet, chess(false, 0), DAY, None);
    assert_eq!(
        resigned,
        Default::default(),
        "an instant resign pays nothing"
    );
    assert_eq!(
        wallet,
        Wallet::default(),
        "and does not start the daily bonus"
    );
    let early = table.earned(&mut wallet, chess(false, 29), DAY, None);
    assert_eq!(early.sc, 0, "a loss one ply short pays nothing");
    assert_eq!(wallet.last_activity_day, None);

    let lost = table.earned(&mut wallet, chess(false, 30), DAY, None);
    assert_eq!(lost.sc, game + bonus, "a loss at min_plies pays");
    let mut fresh = Wallet::default();
    let won = table.earned(&mut fresh, chess(true, 0), DAY, None);
    assert_eq!(won.sc, game + bonus, "a win pays at any length");
    let mut fresh = Wallet::default();
    let drawn = table.earned(&mut fresh, chess(true, 1), DAY, None);
    assert_eq!(drawn.sc, game + bonus, "a draw pays at any length");
}

/// `chess_min_plies` is data: retuning it changes the outcome, and the loader bounds it.
#[test]
fn retuning_chess_min_plies_in_data_changes_the_outcome() {
    let paid = |min: u32| {
        let mut table = shipped();
        table.chess_min_plies = min;
        table
            .earned(&mut Wallet::default(), chess(false, 10), DAY, None)
            .sc
    };
    assert_eq!(paid(11), 0, "a stricter minimum refuses the same game");
    assert!(paid(10) > 0, "the same game pays at its own length");
    assert!(paid(0) > 0, "0 pays every settled game");

    let directory = scratch("chess-min-plies");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(table_path()).unwrap()).unwrap();
    let load = |name: &str, value: &serde_json::Value| {
        fs::write(directory.join(name), serde_json::to_vec(value).unwrap()).unwrap();
        EarnTable::load(&directory.join(name))
    };
    value["chess_min_plies"] = json!(400);
    assert_eq!(
        load("max.json", &value).unwrap().unwrap().chess_min_plies,
        400
    );
    value["chess_min_plies"] = json!(401);
    assert!(load("over.json", &value).is_err(), "above the sane bound");
    value.as_object_mut().unwrap().remove("chess_min_plies");
    assert_eq!(
        load("default.json", &value)
            .unwrap()
            .unwrap()
            .chess_min_plies,
        20,
        "an omitted value keeps the (R) default"
    );
    let _ = fs::remove_dir_all(&directory);
}

/// With every amount tuned to 0 and only the ride flag set, the payout is the ride alone: the
/// applier sends the item event but no empty 50/1 reward bundle.
#[test]
fn a_zeroed_table_with_a_ride_sends_no_empty_reward_bundle() {
    let mut table = shipped();
    let first = table.first_clear_ride.expect("a first-clear ride ships");
    table.hardest_dungeon_clear = 0;
    table.daily_first_activity = 0;
    let offer = RideOffer {
        item_id: first.item_id,
        sc_equivalent: 0,
        owned: false,
        room: true,
    };
    let mut wallet = Wallet::default();
    let earned = table.earned(
        &mut wallet,
        EarnSource::DungeonClear {
            id: first.dungeon_id,
        },
        DAY,
        Some(offer),
    );
    assert_eq!((earned.sc, earned.ride), (0, Some(first.item_id)));

    let state = RewardState::new(fb::runtime_inventory().clone(), profiles(), 100)
        .with_wallet(wallet, 1_000);
    let (next, replies) = apply_grant(&state, &earned.grant(), "sc-zeroed").unwrap();
    assert_eq!(
        next.inventory
            .state()
            .items
            .iter()
            .filter(|item| item.definition_id == first.item_id)
            .count(),
        1,
        "the ride is granted"
    );
    assert!(
        !replies.is_empty() && !replies.iter().any(|p| p.starts_with(&[50, 0, 1])),
        "an item event, and no empty reward bundle: {replies:?}"
    );
}

#[test]
fn station_cash_grant_pays_once_through_the_one_applier() {
    let state = reward_state(Wallet::default(), 1_000);
    let (next, replies) = apply_grant(&state, &sc_grant(4), "sc-test-once").unwrap();
    assert_eq!(next.wallet.as_ref().unwrap().balance, 4);
    assert_eq!(
        replies
            .iter()
            .filter_map(|packet| wallet_balance(packet))
            .collect::<Vec<_>>(),
        vec![4],
        "the wallet response carries the new balance"
    );
    assert!(
        !replies.iter().any(|packet| packet.starts_with(&[50, 0, 1])),
        "a Station Cash-only grant has no coin/XP reward bundle"
    );

    // The same completion key is paid once by the applier's own replay guard.
    let (again, replies) = apply_grant(&next, &sc_grant(4), "sc-test-once").unwrap();
    assert_eq!(again.wallet.as_ref().unwrap().balance, 4);
    assert!(replies.is_empty());

    // A coin grant still carries its reward bundle.
    let mut grant = sc_grant(1);
    grant.coins = 2;
    let (_, replies) = apply_grant(&next, &grant, "sc-test-mixed").unwrap();
    assert!(replies.iter().any(|packet| packet.starts_with(&[50, 0, 1])));
}

#[test]
fn station_cash_grant_caps_at_the_wallet_ceiling_and_tells_the_player() {
    let wallet = Wallet {
        balance: 8,
        ..Wallet::default()
    };
    let state = reward_state(wallet, 10);
    let (next, replies) = apply_grant(&state, &sc_grant(5), "sc-test-cap").unwrap();
    assert_eq!(next.wallet.as_ref().unwrap().balance, 10);
    assert!(
        replies
            .iter()
            .any(|packet| wallet_balance(packet) == Some(10)),
        "the player sees the capped balance"
    );
    assert!(
        replies.iter().any(|packet| capped_notice(packet)),
        "the player is told the rest was not paid"
    );
}

#[test]
fn station_cash_grant_without_a_wallet_is_refused() {
    let state = RewardState::new(inventory(), profiles(), 100);
    assert_eq!(
        apply_grant(&state, &sc_grant(1), "sc-test-no-wallet").err(),
        Some(GrantFailure::NoWallet)
    );
}

#[test]
fn wallet_save_round_trips_the_earned_ledger_and_prunes_it() {
    let wallet = Wallet {
        balance: 30,
        orders: Vec::new(),
        earned_rides: vec![CHROME_DRAKE as u32],
        last_activity_day: Some(DAY),
    };
    let bytes = serde_json::to_vec(&wallet).unwrap();
    let decoded: Wallet = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, wallet);

    let config = station_cash::Config {
        enabled: true,
        max_balance: 20,
        order_history: 2,
        ..station_cash::Config::default()
    };
    let corrupt = Wallet {
        balance: 99,
        earned_rides: vec![CHROME_DRAKE as u32, 0, CHROME_DRAKE as u32],
        ..Wallet::default()
    };
    let restored = Wallet::restore(Some(corrupt.clone()), &config);
    assert_eq!(restored.balance, 20);
    assert_eq!(restored.earned_rides, vec![CHROME_DRAKE as u32]);

    let mut disabled = config.clone();
    disabled.enabled = false;
    assert_eq!(Wallet::restore(Some(corrupt.clone()), &disabled), corrupt);
}

const BRAWLER: i32 = 43;

/// The shipped first-clear ride and its dungeon, read from the table so a data edit moves the test.
fn first_clear() -> (u32, i32) {
    let ride = shipped()
        .first_clear_ride
        .expect("a first-clear ride ships");
    (ride.dungeon_id, ride.item_id)
}

/// A level-20 Brawler beside the first-clear dungeon's marker (any level gate is below 20; the
/// gate itself is pinned by `dungeon_stone_heart`).
fn hardest_session(fixture: &fb::Fixture, guid: u64) -> Gateway {
    let file = fb::battles();
    let row = fb::battle(&file, first_clear().0);
    let mut gateway = fixture.gateway(fb::position(&row["spawner"]["position"]), guid);
    assert!(fb::send(&mut gateway, &fb::switch_job(BRAWLER), 300).state_changed);
    fb::send(&mut gateway, &chat::local_request("realm level 20"), 350);
    assert_eq!(fb::level(&gateway, BRAWLER), 20);
    gateway
}

/// Open, enter and win the first-clear dungeon through the production route (26/8 marker, 41/108
/// Go, 36/10 attacks and the world tick). Returns the time after the win.
fn win_hardest(gateway: &mut Gateway, now_ms: u64) -> u64 {
    win_hardest_replies(gateway, now_ms).0
}

/// [`win_hardest`] plus every reply the fight produced.
fn win_hardest_replies(gateway: &mut Gateway, now_ms: u64) -> (u64, Vec<Vec<u8>>) {
    let file = fb::battles();
    let row = fb::battle(&file, first_clear().0);
    let screen = fb::open_battle(gateway, fb::u64_field(&row["spawner"], "guid"), row, now_ms);
    fb::enter_battle(gateway, &screen, row, now_ms + 500);
    let mut now = now_ms + 1_000;
    let replies = fb::fight_to_win(gateway, &mut now, row);
    (now, replies)
}

fn saved_wallet(gateway: &Gateway) -> Option<Wallet> {
    gateway.snapshot().gameplay.unwrap().sc_wallet
}

fn rides_owned(gateway: &Gateway, item_id: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == item_id)
        .map(|item| item.count)
        .sum()
}

/// The ride's Marketplace value from bundled `stores.json`: `round(sc_price * sc_price_scale)`,
/// the T230 listing formula.
fn marketplace_value(item_id: i32) -> u32 {
    let stores: serde_json::Value = serde_json::from_slice(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/stores.json"))
            .unwrap(),
    )
    .unwrap();
    let price = stores["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["item_id"] == item_id)
        .and_then(|row| row["sc_price"].as_f64())
        .expect("stores.json prices the ride");
    let scale = stores["station_cash"]["sc_price_scale"].as_f64().unwrap();
    (price * scale).round() as u32
}

/// The hardest dungeon's first clear grants the earn-only ride with the hardest-tier Station Cash,
/// at the win. A disconnect after the win keeps both, the results Close pays nothing more, a
/// second clear pays the clear only, and the wallet survives a reload.
#[test]
fn hardest_dungeon_first_clear_grants_the_ride_once_then_pays_station_cash() {
    let (dungeon, ride) = first_clear();
    let table = shipped();
    let clear = table.amount(EarnSource::DungeonClear { id: dungeon });
    assert_eq!(
        clear, table.hardest_dungeon_clear,
        "the ride dungeon is the hardest tier"
    );
    let fixture = fb::Fixture::new();
    let mut gateway = hardest_session(&fixture, 2_001);
    assert_eq!(saved_wallet(&gateway), None);
    assert_eq!(rides_owned(&gateway, ride), 0);

    let now = win_hardest(&mut gateway, 500);
    let won = saved_wallet(&gateway).expect("the win writes the Station Cash wallet");
    assert_eq!(won.balance, clear + table.daily_first_activity);
    assert_eq!(won.earned_rides, vec![ride as u32]);
    assert_eq!(
        rides_owned(&gateway, ride),
        1,
        "the first clear grants the ride"
    );

    // The run is connection-local, but the clear was paid at the win: a reconnect keeps it.
    let reconnected = fb::reopen(&fixture, &gateway, "won.json", now + 100);
    assert_eq!(saved_wallet(&reconnected), Some(won.clone()));
    assert_eq!(rides_owned(&reconnected, ride), 1);

    let closed = fb::send(&mut gateway, &[26, 0, 42, 0], now + 700);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(
        saved_wallet(&gateway),
        Some(won.clone()),
        "the results Close pays nothing more"
    );
    // The client's zone-load handshake (13, then 10) after zoning back to the marker.
    fb::arena_spawned(&mut gateway, now + 800);

    let now = win_hardest(&mut gateway, now + 1_000);
    let second = saved_wallet(&gateway).expect("the second win keeps the wallet");
    assert_eq!(
        second.balance,
        won.balance + clear,
        "a later clear pays the clear only (no ride, no second daily bonus)"
    );
    assert_eq!(second.earned_rides, won.earned_rides);
    assert_eq!(
        rides_owned(&gateway, ride),
        1,
        "the ride is never granted twice"
    );

    let reloaded = fb::reopen(&fixture, &gateway, "second.json", now + 100);
    assert_eq!(saved_wallet(&reloaded), Some(second));
    assert_eq!(rides_owned(&reloaded, ride), 1);
}

/// Leaving the hardest dungeon before the win pays nothing. A character who already owns the
/// earn-only ride is paid its Marketplace Station Cash value on the first clear instead of a
/// second copy, and the award is settled once.
#[test]
fn an_owned_ride_pays_its_marketplace_value_and_leaving_early_pays_nothing() {
    let (dungeon, ride) = first_clear();
    let table = shipped();
    let fixture = fb::Fixture::new();
    let seed = hardest_session(&fixture, 2_002);
    let mut gateway = fb::reopen_with(&fixture, &seed, "owned.json", 400, |state| {
        let id = state
            .inventory
            .items
            .iter()
            .map(|item| item.id)
            .max()
            .unwrap_or(0)
            + 1;
        state.inventory.items.push(fb::stack(id, ride, 1));
        state.inventory.next_instance_id = Some(i64::from(id) + 1);
    });
    assert_eq!(rides_owned(&gateway, ride), 1);

    let file = fb::battles();
    let row = fb::battle(&file, dungeon);
    let screen = fb::open_battle(
        &mut gateway,
        fb::u64_field(&row["spawner"], "guid"),
        row,
        500,
    );
    fb::enter_battle(&mut gateway, &screen, row, 1_000);
    let left = fb::send(&mut gateway, &chat::local_request("realm exit"), 1_500);
    assert!(
        left.began_zoning,
        "the ordinary leave returns to the open world"
    );
    assert_eq!(
        saved_wallet(&gateway),
        None,
        "leaving early pays no Station Cash"
    );
    fb::arena_spawned(&mut gateway, 1_600);

    win_hardest(&mut gateway, 2_000);
    let paid = saved_wallet(&gateway).expect("the win writes the Station Cash wallet");
    assert_eq!(
        paid.balance,
        table.amount(EarnSource::DungeonClear { id: dungeon })
            + table.daily_first_activity
            + marketplace_value(ride),
        "an owned ride pays its Marketplace value instead of a second copy"
    );
    assert_eq!(rides_owned(&gateway, ride), 1, "no second copy");
    assert_eq!(paid.earned_rides, vec![ride as u32]);
}

const MAKE_ROOM: &str = "Make room in your bag";

/// Every system chat line in a set of gateway replies.
fn said(replies: &[Vec<u8>]) -> String {
    replies
        .iter()
        .map(|packet| fb::unframe(packet))
        .filter(|packet| packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8)
        .map(|packet| {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&packet[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

/// A first clear whose payout finds the bag full pays the clear only: the ride is not granted and
/// not settled, the player is told to make room, and a later clear with room grants it once.
#[test]
fn a_full_bag_defers_the_first_clear_ride_until_a_later_clear_has_room() {
    let (dungeon, ride) = first_clear();
    let table = shipped();
    let clear = table.amount(EarnSource::DungeonClear { id: dungeon });
    let fixture = fb::Fixture::new();
    let seed = hardest_session(&fixture, 2_003);
    // 99 stacks: the win's prize takes the 100th, so no stack is free when the clear pays.
    let mut gateway = fb::reopen_with(&fixture, &seed, "full.json", 400, |state| {
        state.inventory = fb::full_bag(&state.inventory, 99, dungeon);
    });
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.items.len(),
        99
    );

    let (now, replies) = win_hardest_replies(&mut gateway, 500);
    let first = saved_wallet(&gateway).expect("the win pays the clear");
    assert_eq!(first.balance, clear + table.daily_first_activity);
    assert!(
        first.earned_rides.is_empty(),
        "a full bag does not settle the ride"
    );
    assert_eq!(rides_owned(&gateway, ride), 0, "and does not grant it");
    assert!(
        said(&replies).contains(MAKE_ROOM),
        "the player is told to make room"
    );

    // Back in the open world, free three stacks and clear again.
    assert!(fb::send(&mut gateway, &[26, 0, 42, 0], now + 700).began_zoning);
    fb::arena_spawned(&mut gateway, now + 800);
    let mut gateway = fb::reopen_with(&fixture, &gateway, "freed.json", now + 900, |state| {
        let keep = state.inventory.items.len() - 3;
        state.inventory.items.truncate(keep);
    });
    let (now, replies) = win_hardest_replies(&mut gateway, now + 1_000);
    let second = saved_wallet(&gateway).expect("the second win pays");
    assert_eq!(
        second.balance,
        first.balance + clear,
        "the clear pays again"
    );
    assert_eq!(second.earned_rides, vec![ride as u32]);
    assert_eq!(rides_owned(&gateway, ride), 1, "room grants the ride");
    assert!(!said(&replies).contains(MAKE_ROOM));

    assert!(fb::send(&mut gateway, &[26, 0, 42, 0], now + 700).began_zoning);
    fb::arena_spawned(&mut gateway, now + 800);
    win_hardest(&mut gateway, now + 1_000);
    assert_eq!(rides_owned(&gateway, ride), 1, "granted once");
}
