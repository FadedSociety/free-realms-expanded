//! P-banking through the real `Gateway` on the shipped content alone (`data/content` with the
//! `data/release` overlay, no lab file): a banker click opens storage, deposits and withdrawals
//! round-trip, items stack by item and tint, a full bank refuses, the ten Coin Shop upgrades are
//! bought in order and pay once, the Storage Vault opens storage from a lot, storage survives a
//! relog, and a failed save loses nothing.
//!
//! Every request is the client's own body (`src/bank/mod.rs` cites the readers and writers):
//! 26/8 interact `008b6bc0` (u64 target + bool), 26/44 `RequestStorageCheck` `00c24370` (bare
//! header), 42/17 `MoveItemToStorage` writer `00c20b30` (u64 guid, i32 item, signed count), 165/4
//! buy writer `00c75ed0` (merchant, context, item, tint, quantity) and 165/8 `00c76d20` (header).
#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;
#[path = "support/json.rs"]
#[allow(dead_code)]
mod json;

use gateway_packets::{body, interact, login_with_version, wrap};
use osfr_tools::{
    character_store::CharacterState,
    game_store::{self, Snapshot},
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    housing_native::{FixtureItemList, FixtureUpdate},
    housing_service::fixture_actor_guid,
    inventory::{InventoryItem, InventoryState},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const BEZZLE: u64 = 9_500_000_450;
const BURGLZ: u64 = 9_500_000_563;
const YOINK: u64 = 9_600_200_000;
const BEZZLE_AT: [f32; 3] = [-1283.2, -14.39, 341.4];
const UPGRADE_1: i32 = 35_853;
const UPGRADE_2: i32 = 35_854;
/// The 50-slot upgrade: repeatable, outside the order (R, `bank.json`).
const UPGRADE_50: i32 = 35_852;
const VAULT: i32 = 910_900;
/// A real uncapped catalogue stack (`max_stack` -1).
const STACKABLE: i32 = 75_060;
/// A real one-per-stack housing item (`max_stack` 1).
const SINGLE: i32 = 16_014;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// `data/content` with the `data/release` overlay, hard-linked into a scratch folder; the opening
/// campaign is left out so the character stands in the open world. Removed on drop.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-bank-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        link_tree(&data.join("content"), &dir);
        link_tree(&data.join("release"), &dir);
        fs::remove_file(dir.join("opening.json")).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Link every file of `from` into `to` (the overlay replaces a file of the same name).
fn link_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            link_tree(&entry.path(), &target);
        } else {
            let _ = fs::remove_file(&target);
            if fs::hard_link(entry.path(), &target).is_err() {
                fs::copy(entry.path(), &target).unwrap();
            }
        }
    }
}

fn record(id: i32, definition_id: i32, tint: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

/// A saved character holding exactly `items` and `coins`.
fn saved(content: &Content, coins: u32, items: Vec<InventoryItem>) -> GameplayState {
    let mut state = Gameplay::load(content.path(), None).unwrap().state();
    for entry in &mut state.inventory.equipment {
        entry.item_id = 0;
    }
    let next = items
        .iter()
        .map(|item| i64::from(item.id))
        .max()
        .unwrap_or(0)
        + 1;
    state.inventory.items = items;
    state.inventory.coins = coins;
    state.inventory.next_instance_id = Some(next);
    state
}

fn character(content: &Content, position: [f32; 3]) -> CharacterState {
    let mut character = Gateway::content_state(content.path()).unwrap();
    character.zone_id = 0;
    character.position = position;
    character
}

/// Login, ready and world entry; returns the session and the login replies (the self data).
fn connect(
    content: &Content,
    who: CharacterState,
    state: GameplayState,
) -> (Gateway, Vec<Vec<u8>>) {
    let version = json::read(&content.path().join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned();
    let guid = who.guid;
    let mut gateway =
        Gateway::load_gameplay_from_content(who, content.path(), Some(state)).unwrap();
    let mut early = Vec::new();
    for message in [
        login_with_version(guid, &version),
        wrap(&[13, 0]),
        wrap(&[10, 0]),
    ] {
        let out = gateway.handle(&message).unwrap();
        early.extend(out.replies.iter().map(|reply| body(reply).to_vec()));
    }
    assert!(gateway.is_world_loaded());
    (gateway, early)
}

fn at_bezzle(content: &Content, state: GameplayState) -> Gateway {
    connect(content, character(content, BEZZLE_AT), state).0
}

/// One client message; the inner replies.
fn send(gateway: &mut Gateway, inner: &[u8]) -> Vec<Vec<u8>> {
    let out = gateway.handle(&wrap(inner)).unwrap();
    out.replies
        .iter()
        .map(|reply| body(reply).to_vec())
        .collect()
}

fn move_item(item: i32, quantity: i32) -> Vec<u8> {
    let mut packet = vec![42, 0, 17, 0];
    packet.extend(1u64.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(quantity.to_le_bytes());
    packet
}

/// 165/4 from the Coin Shop window: no merchant, the client's transaction id, the item, tint 0,
/// one copy.
fn buy(transaction: u32, item: i32) -> Vec<u8> {
    let mut packet = vec![0xa5, 0, 4, 0];
    packet.extend(0u64.to_le_bytes());
    packet.extend(transaction.to_le_bytes());
    packet.extend(item.to_le_bytes());
    packet.extend(0i32.to_le_bytes());
    packet.extend(1u32.to_le_bytes());
    packet
}

fn word(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

/// The last 38/31 among `replies`: (slots used, slots owned).
fn usage(replies: &[Vec<u8>]) -> (i32, i32) {
    let packet = (replies.iter().rev())
        .find(|reply| reply.starts_with(&[38, 0, 31, 0]))
        .expect("a 38/31 storage count");
    assert_eq!(packet.len(), 12);
    (word(packet, 4), word(packet, 8))
}

fn opens(replies: &[Vec<u8>]) -> bool {
    replies.iter().any(|reply| reply == &[42, 0, 18, 0])
}

fn chat_text(replies: &[Vec<u8>]) -> String {
    (replies.iter())
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| String::from_utf8_lossy(reply).into_owned())
        .collect()
}

fn inventory(gateway: &Gateway) -> InventoryState {
    gateway.snapshot().gameplay.unwrap().inventory
}

fn stack(gateway: &Gateway, id: i32) -> InventoryItem {
    inventory(gateway)
        .items
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
}

/// The 38/3 update for item `id` among `replies`: (count, consumed, stored).
fn update_of(replies: &[Vec<u8>], id: i32) -> (i32, i32, i32) {
    let packet = (replies.iter())
        .find(|reply| reply.starts_with(&[38, 0, 3, 0]) && word(reply, 4) == id)
        .unwrap_or_else(|| panic!("a 38/3 for item {id}"));
    (word(packet, 8), word(packet, 16), word(packet, 20))
}

#[test]
fn a_banker_click_opens_storage_and_counts_its_slots() {
    let content = Content::new();
    let state = saved(&content, 0, vec![record(1, STACKABLE, 0, 5)]);
    let mut gateway = at_bezzle(&content, state.clone());
    let replies = send(&mut gateway, &interact(BEZZLE));
    assert!(opens(&replies), "Bezzle opens storage: {replies:?}");
    assert_eq!(
        usage(&replies),
        (0, 25),
        "a new bank: nothing stored, 25 slots"
    );
    // The window's own question (`StorageHandler:Show` -> 26/44) is answered the same way.
    assert_eq!(usage(&send(&mut gateway, &[26, 0, 44, 0])), (0, 25));
    // The first visit hands over the Storage Vault, once.
    let vaults = |g: &Gateway| {
        inventory(g)
            .items
            .iter()
            .filter(|i| i.definition_id == VAULT)
            .count()
    };
    assert_eq!(vaults(&gateway), 1);
    assert!(chat_text(&replies).contains("Storage Vault"));
    let again = send(&mut gateway, &interact(BEZZLE));
    assert!(opens(&again) && !chat_text(&again).contains("Storage Vault"));
    assert_eq!(vaults(&gateway), 1, "a second visit gives no second vault");

    // Out of reach, nothing opens and nothing changes.
    let far = character(&content, [BEZZLE_AT[0] + 40.0, BEZZLE_AT[1], BEZZLE_AT[2]]);
    let (mut away, _) = connect(&content, far, state.clone());
    let before = inventory(&away);
    let replies = send(&mut away, &interact(BEZZLE));
    assert!(!opens(&replies), "Bezzle is 40 m away");
    assert_eq!(inventory(&away), before);

    // Yoink (Snowhill) and Burglz (Seaside) bank too, each where they stand.
    for (banker, file) in [(YOINK, "npcs/bankers.json"), (BURGLZ, "npcs/seaside.json")] {
        let rows = json::read(&content.path().join(file));
        let row = (rows["records"].as_array().unwrap().iter())
            .find(|row| row["npc_id"].as_u64() == Some(banker))
            .unwrap();
        let at = |i: usize| row["position"][i].as_f64().unwrap() as f32;
        let who = character(&content, [at(0) + 2.0, at(1), at(2)]);
        let (mut gateway, _) = connect(&content, who, state.clone());
        assert!(
            opens(&send(&mut gateway, &interact(banker))),
            "banker {banker}"
        );
    }
}

#[test]
fn a_deposit_and_a_withdrawal_round_trip_without_changing_what_is_owned() {
    let content = Content::new();
    let mut gateway = at_bezzle(
        &content,
        saved(&content, 0, vec![record(1, STACKABLE, 0, 5)]),
    );
    let replies = send(&mut gateway, &move_item(1, 3));
    assert_eq!(
        update_of(&replies, 1),
        (5, 0, 3),
        "count 5, 3 of them stored"
    );
    assert_eq!(usage(&replies), (1, 25));
    assert_eq!(
        (stack(&gateway, 1).count, stack(&gateway, 1).stored_count),
        (5, 3)
    );

    // The bag cannot spend what is banked: more than the 2 left is refused.
    let refused = send(&mut gateway, &move_item(1, 3));
    assert!(chat_text(&refused).contains("do not have that many"));
    assert_eq!(stack(&gateway, 1).stored_count, 3);

    let replies = send(&mut gateway, &move_item(1, -3));
    assert_eq!(update_of(&replies, 1), (5, 0, 0));
    assert_eq!(usage(&replies), (0, 25));
    let refused = send(&mut gateway, &move_item(1, -1));
    assert!(chat_text(&refused).contains("holds fewer"));
    assert_eq!(
        (stack(&gateway, 1).count, stack(&gateway, 1).stored_count),
        (5, 0)
    );
}

#[test]
fn identical_items_with_one_tint_share_a_slot() {
    let content = Content::new();
    let bag = vec![
        record(1, SINGLE, 0, 1),
        record(2, SINGLE, 0, 1),
        record(3, SINGLE, 7, 1),
        record(4, STACKABLE, 0, 9),
    ];
    let mut gateway = at_bezzle(&content, saved(&content, 0, bag));
    assert_eq!(usage(&send(&mut gateway, &move_item(1, 1))), (1, 25));
    assert_eq!(
        usage(&send(&mut gateway, &move_item(2, 1))),
        (1, 25),
        "same item, same tint"
    );
    assert_eq!(
        usage(&send(&mut gateway, &move_item(3, 1))),
        (2, 25),
        "another tint"
    );
    assert_eq!(usage(&send(&mut gateway, &move_item(4, 4))), (3, 25));
    assert_eq!(
        usage(&send(&mut gateway, &move_item(4, 5))),
        (3, 25),
        "more of a stored stack"
    );
}

#[test]
fn a_full_bank_refuses_a_new_slot_and_changes_nothing() {
    let content = Content::new();
    let bag: Vec<_> = (0..26)
        .map(|tint| record(tint + 1, STACKABLE, tint, 2))
        .collect();
    let mut gateway = at_bezzle(&content, saved(&content, 0, bag));
    for id in 1..=25 {
        send(&mut gateway, &move_item(id, 1));
    }
    assert_eq!(usage(&send(&mut gateway, &[26, 0, 44, 0])), (25, 25));
    let before = inventory(&gateway);
    let refused = send(&mut gateway, &move_item(26, 1));
    assert!(chat_text(&refused).contains("bank is full"), "{refused:?}");
    assert_eq!(inventory(&gateway), before);
    // A slot already in use still takes more.
    assert_eq!(usage(&send(&mut gateway, &move_item(1, 1))), (25, 25));
    assert_eq!(stack(&gateway, 1).stored_count, 2);
}

#[test]
fn upgrades_are_refused_out_of_order_and_take_their_coins_once() {
    let content = Content::new();
    let mut gateway = at_bezzle(&content, saved(&content, 10_000, Vec::new()));
    // The Coin Shop's list: upgrade 1 open, the other nine locked by the first flag, the
    // repeatable 50-slot upgrade (listed last) open.
    let list = send(&mut gateway, &[0xa5, 0, 8, 0]);
    let rows = list
        .iter()
        .find(|r| r.starts_with(&[0xa5, 0, 9, 0]))
        .unwrap();
    assert_eq!(word(rows, 4), 11);
    let flag = |row: usize| rows[8 + row * 19 + 16];
    assert_eq!((word(rows, 8), flag(0), flag(1)), (UPGRADE_1, 0, 1));
    assert_eq!((word(rows, 8 + 10 * 19), flag(10)), (UPGRADE_50, 0));

    // Upgrade 2 first: refused, nothing spent.
    let before = inventory(&gateway);
    let replies = send(&mut gateway, &buy(7, UPGRADE_2));
    let done = replies
        .iter()
        .find(|r| r.starts_with(&[0xa5, 0, 6, 0]))
        .unwrap();
    assert_ne!(word(done, 4), 1, "the 165/6 completion is a failure");
    assert!(chat_text(&replies).contains("in order"));
    assert_eq!(inventory(&gateway), before);

    // Upgrade 1: 1,000 coins once, 2 slots more, and upgrade 2 unlocks.
    let replies = send(&mut gateway, &buy(8, UPGRADE_1));
    let done = replies
        .iter()
        .find(|r| r.starts_with(&[0xa5, 0, 6, 0]))
        .unwrap();
    assert_eq!(word(done, 4), 1, "success");
    assert_eq!(inventory(&gateway).coins, 9_000);
    assert_eq!(usage(&replies), (0, 27));
    let rows = replies
        .iter()
        .find(|r| r.starts_with(&[0xa5, 0, 9, 0]))
        .unwrap();
    assert_eq!((rows[8 + 16], rows[8 + 19 + 16]), (1, 0), "1 owned, 2 open");
    // Applied at once: the record is kept (owned) but is not in the bag.
    let owned: Vec<_> = (inventory(&gateway).items.into_iter())
        .filter(|item| item.definition_id == UPGRADE_1)
        .collect();
    assert_eq!(owned.len(), 1);
    assert_eq!((owned[0].count, owned[0].consumed_count), (1, 1));
    assert_eq!(inventory(&gateway).bag_stacks(), 0);

    // The same upgrade again: refused, the coins stay.
    let replies = send(&mut gateway, &buy(9, UPGRADE_1));
    assert!(chat_text(&replies).contains("already own"));
    assert_eq!(inventory(&gateway).coins, 9_000);
    // Upgrade 2 is next and costs 2,000.
    send(&mut gateway, &buy(10, UPGRADE_2));
    assert_eq!(inventory(&gateway).coins, 7_000);
    assert_eq!(usage(&send(&mut gateway, &[26, 0, 44, 0])), (0, 31));
    // Without the coins: refused, nothing changes.
    let mut poor = at_bezzle(&content, saved(&content, 999, Vec::new()));
    let before = inventory(&poor);
    let replies = send(&mut poor, &buy(1, UPGRADE_1));
    assert!(chat_text(&replies).contains("more coins"));
    assert_eq!(inventory(&poor), before);
    // The 50-slot upgrade needs no order and has no limit: 25,000 coins and 50 slots a copy.
    let mut rich = at_bezzle(&content, saved(&content, 60_000, Vec::new()));
    for (transaction, coins, slots) in [(1, 35_000, 75), (2, 10_000, 125)] {
        let replies = send(&mut rich, &buy(transaction, UPGRADE_50));
        assert_eq!(inventory(&rich).coins, coins);
        assert_eq!(usage(&replies), (0, slots));
    }
    assert_eq!(inventory(&rich).bag_stacks(), 0, "applied, not in the bag");
}

#[test]
fn storage_survives_a_relog_and_the_self_data_carries_it() {
    let content = Content::new();
    let mut gateway = at_bezzle(
        &content,
        saved(&content, 5_000, vec![record(1, STACKABLE, 0, 5)]),
    );
    send(&mut gateway, &move_item(1, 4));
    send(&mut gateway, &buy(1, UPGRADE_1));
    let path = std::env::temp_dir().join(format!("osfr-bank-relog-{}.json", std::process::id()));
    game_store::save(&path, &gateway.snapshot()).unwrap();
    let loaded = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    let _ = fs::remove_file(&path);

    let (mut again, early) = connect(&content, loaded.character, loaded.gameplay.unwrap());
    assert_eq!(
        (stack(&again, 1).count, stack(&again, 1).stored_count),
        (5, 4)
    );
    assert_eq!(usage(&send(&mut again, &[26, 0, 44, 0])), (1, 27));
    // The self data's item record ends with the stored count (+0x1c, reader `008fa4b0`).
    let mut expected = Vec::new();
    for value in [STACKABLE, 0, 1, 5, 0, i32::MAX] {
        expected.extend(value.to_le_bytes());
    }
    expected.push(0);
    expected.extend(4i32.to_le_bytes());
    let self_data = early
        .iter()
        .find(|reply| reply.starts_with(&[12, 0]))
        .unwrap();
    assert!(
        self_data
            .windows(expected.len())
            .any(|window| window == expected),
        "the banked stack logs in with its stored count"
    );
}

#[test]
fn a_failed_save_loses_nothing() {
    let content = Content::new();
    let path = std::env::temp_dir().join(format!("osfr-bank-fail-{}.json", std::process::id()));
    let _ = fs::remove_file(&path);
    let mut gateway = at_bezzle(
        &content,
        saved(&content, 0, vec![record(1, STACKABLE, 0, 5)]),
    );
    game_store::save(&path, &gateway.snapshot()).unwrap();
    // The host stages each message on a copy and installs it only after its save
    // (`Session::commit_outcome`); a save that cannot be written drops the copy.
    let mut staged = gateway.clone();
    send(&mut staged, &move_item(1, 5));
    assert_eq!(stack(&staged, 1).stored_count, 5);
    let blocked = path.join("blocked.json");
    assert!(game_store::save(&blocked, &staged.snapshot()).is_err());
    drop(staged);
    let disk = game_store::load_or_create(&path, gateway.state().clone()).unwrap();
    let _ = fs::remove_file(&path);
    let item = disk
        .gameplay
        .unwrap()
        .inventory
        .items
        .into_iter()
        .find(|i| i.id == 1)
        .unwrap();
    assert_eq!(
        (item.count, item.stored_count),
        (5, 0),
        "the save on disk is the last good one"
    );
    assert_eq!(
        stack(&gateway, 1).stored_count,
        0,
        "the live session never saw the move"
    );
    // Whatever was saved, a move only ever shifts units inside the record: two racing moves keep
    // the five units owned whichever lands.
    let first = send(&mut gateway, &move_item(1, 3));
    let second = send(&mut gateway, &move_item(1, 3));
    assert!(chat_text(&second).contains("do not have that many") && !first.is_empty());
    let item = stack(&gateway, 1);
    assert_eq!((item.count, item.stored_count), (5, 3));
}

#[test]
fn the_storage_vault_opens_storage_from_a_lot() {
    let content = Content::new();
    let mut gateway = at_bezzle(
        &content,
        saved(&content, 0, vec![record(1, STACKABLE, 0, 3)]),
    );
    send(&mut gateway, &interact(BEZZLE));
    let vault = (inventory(&gateway).items.into_iter())
        .find(|item| item.definition_id == VAULT)
        .expect("the first visit gave a vault")
        .id;
    // The starter lot, entered and loaded, then the vault placed in edit mode.
    let list = send(&mut gateway, &[127, 0, 23, 0]);
    let lot = list
        .iter()
        .find(|r| r.starts_with(&[127, 0, 39, 0]))
        .unwrap();
    let records = osfr_tools::housing_native::InstanceList::decode(&lot[2..])
        .unwrap()
        .records;
    let house = (records.iter())
        .find(|r| r.name_id == 296)
        .unwrap()
        .instance_guid;
    let mut enter = vec![127, 0, 19, 0];
    enter.extend(house.to_le_bytes());
    enter.extend(0i32.to_le_bytes());
    enter.push(0);
    send(&mut gateway, &enter);
    // The lot is a scene change: the client reports ready, then loaded, again.
    let mut loaded = send(&mut gateway, &[13, 0]);
    loaded.extend(send(&mut gateway, &[10, 0]));
    let tray = loaded
        .iter()
        .find(|r| r.starts_with(&[127, 0, 43, 0]))
        .expect("the 127/43 tray");
    // The shipped creative tray lists the vault's fixture row; the owned vault is placed from the
    // bag, by its item id, as the decorate bar does.
    let tray = FixtureItemList::decode(&tray[2..]).unwrap();
    assert!(tray.items.iter().any(|item| item.item_definition_id == VAULT));
    send(&mut gateway, &[127, 0, 6, 0, 1]);
    let mut ask = vec![127, 0, 1, 0];
    ask.extend((vault as u32).to_le_bytes());
    ask.extend((-1i32).to_le_bytes());
    ask.extend((-1i32).to_le_bytes());
    ask.push(0);
    send(&mut gateway, &ask);
    let mut place = vec![127, 0, 2, 0];
    place.extend((vault as u32).to_le_bytes());
    for value in [100.0f32, 0.0, -30.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
        place.extend(value.to_le_bytes());
    }
    let placed = send(&mut gateway, &place);
    let update = placed
        .iter()
        .find(|r| r.starts_with(&[127, 0, 40, 0]))
        .expect("placed");
    let fixture = FixtureUpdate::decode(&update[2..]).unwrap().instance.guid;
    send(&mut gateway, &[127, 0, 6, 0, 0]);

    let replies = send(&mut gateway, &interact(fixture_actor_guid(fixture)));
    assert!(opens(&replies), "the vault opens storage: {replies:?}");
    assert_eq!(usage(&replies), (0, 25));
    assert!(
        !chat_text(&replies).contains("Storage Vault"),
        "a placed vault counts as owned"
    );
    let replies = send(&mut gateway, &move_item(1, 2));
    assert_eq!(update_of(&replies, 1), (3, 0, 2));
}

#[test]
fn the_shipped_bank_table_names_standing_bankers_and_atlas_rows() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let bank = osfr_tools::bank::Bank::load(&content)
        .unwrap()
        .expect("bank.json ships");
    let travel = json::read(&content.join("travel.json"));
    let atlas: Vec<[f64; 3]> = (travel["points"].as_array().unwrap().iter())
        .filter(|point| point["name"] == "Banker" && point["notification_type"] == 6)
        .map(|point| [0, 1, 2].map(|i| point["position"][i].as_f64().unwrap()))
        .collect();
    assert_eq!(atlas.len(), 3, "a teleportable atlas row per banker");
    for (banker, file) in [
        (BEZZLE, "sanctuary.json"),
        (YOINK, "bankers.json"),
        (BURGLZ, "seaside.json"),
    ] {
        assert!(bank.is_banker(banker));
        let rows = json::read(&content.join("npcs").join(file));
        let row = (rows["records"].as_array().unwrap().iter())
            .find(|row| row["npc_id"].as_u64() == Some(banker))
            .unwrap();
        let at = [0, 1, 2].map(|i| row["position"][i].as_f64().unwrap());
        let nearest = (atlas.iter())
            .map(|point| ((point[0] - at[0]).powi(2) + (point[2] - at[2]).powi(2)).sqrt())
            .fold(f64::MAX, f64::min);
        assert!(
            nearest < 12.0,
            "banker {banker} is {nearest} m from an atlas row"
        );
    }
    // Every upgrade is a defined coin item whose price is the wiki's.
    let release = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release");
    let items = osfr_tools::inventory::Inventory::load(&release, None).unwrap();
    let prices: Vec<_> = (35_852..=35_862)
        .map(|id| (bank.is_upgrade(id), items.definition_coin_price(id)))
        .collect();
    // The ten from the wiki; the 50-slot one (R) at their 500 coins a slot.
    let wiki = [
        25_000, 1_000, 2_000, 3_000, 4_000, 5_000, 10_000, 20_000, 30_000, 40_000, 50_000,
    ];
    assert_eq!(prices, wiki.map(|price| (true, Some(price))));
    assert!(items.has_definition(VAULT));
}
