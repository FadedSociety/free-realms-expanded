//! T658: a field-battle win can add a chance item drop beside its prize wheel.
//!
//! Every test drives the production route: marker click -> Go -> melee kills ->
//! results. The drop rows are the shipped `drops` of `field_battles.json`; only
//! the chance is rewritten in a private content copy (1.0 forces the drop, 0.0
//! retires it), so no pinned rate and no test seam is involved. The shipped
//! content is never edited.

use osfr_tools::{
    account_store::AccountRole,
    gateway::Gateway,
    inventory::{Inventory, InventoryState, MAX_ITEMS},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[allow(dead_code)] // this suite only overlays; the helper also has `copy_tree`
#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::*;

const BRAWLER: i32 = 43;
/// One battle win pays 50 job XP (src/encounter.rs).
const BATTLE_JOB_XP: u32 = 50;
/// The shipped battle whose row carries the Yeti drop, and one without a drop.
const YETI_BATTLE: u32 = 12_310;
const PLAIN_BATTLE: u32 = 12_350;

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// The shipped `drops` rows of a battle.
fn shipped_drops(id: u32) -> Vec<Value> {
    battle(&battles(), id)["drops"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// The item the Yeti battle drops, read from the shipped row.
fn yeti_item() -> i32 {
    let drops = shipped_drops(YETI_BATTLE);
    assert!(!drops.is_empty(), "battle {YETI_BATTLE} ships a drop row");
    i32_field(&drops[0], "item_id")
}

/// A private production-shaped content tree: the round70 runtime package
/// (hard-linked, read-only) overlaid with the bundled content, and every
/// battle's drop `chance` set to `chance`.
fn content_with_chance(chance: f64) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "t658-drops-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for entry in fs::read_dir(manifest_dir().join(RUNTIME_CONTENT)).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let target = root.join(entry.file_name());
            if fs::hard_link(entry.path(), &target).is_err() {
                fs::copy(entry.path(), &target).unwrap();
            }
        }
    }
    content_copy::copy_missing(&content_dir(), &root);
    let mut file = battles();
    for row in file["battles"].as_array_mut().unwrap() {
        // `get_mut`: indexing a missing key would insert a null.
        for drop in row
            .get_mut("drops")
            .and_then(Value::as_array_mut)
            .into_iter()
            .flatten()
        {
            drop["chance"] = json!(chance);
        }
    }
    fs::write(
        root.join("field_battles.json"),
        serde_json::to_vec(&file).unwrap(),
    )
    .unwrap();
    root
}

fn count(gateway: &Gateway, item: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|stack| stack.definition_id == item)
        .map(|stack| stack.count)
        .sum()
}

/// Win `battle_id` once from its marker: (gateway, economy before, after).
fn win(
    content: &Path,
    battle_id: u32,
    prizes: &BTreeSet<i32>,
    guid: u64,
) -> (Gateway, Economy, Economy) {
    win_with_bag(content, battle_id, prizes, guid, |bag| bag)
}

/// `win`, with the bag replaced by `shape(bag)` before the fight (reloaded
/// through the production `load_gameplay` restore path, as a save would be).
fn win_with_bag(
    content: &Path,
    battle_id: u32,
    prizes: &BTreeSet<i32>,
    guid: u64,
    shape: impl FnOnce(InventoryState) -> InventoryState,
) -> (Gateway, Economy, Economy) {
    let mut fixture = Fixture::new();
    fixture.content = content.to_path_buf();
    let file = battles();
    let row = battle(&file, battle_id);
    let mut gateway = fixture.gateway(position(&row["spawner"]["position"]), guid);
    assert!(send(&mut gateway, &switch_job(BRAWLER), 200).state_changed);
    let mut snapshot = gateway.snapshot();
    let gameplay = snapshot.gameplay.as_mut().unwrap();
    gameplay.inventory = shape(gameplay.inventory.clone());
    let bag = gameplay.inventory.clone();
    gateway = Gateway::load_gameplay(
        &fixture.reference,
        gateway.state().clone(),
        &fixture.content,
        snapshot.gameplay,
    )
    .unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(AccountRole::Admin);
    fixture.login(&mut gateway);
    send(&mut gateway, &13_u16.to_le_bytes(), 250);
    send(&mut gateway, &10_u16.to_le_bytes(), 260);
    assert_eq!(gateway.snapshot().gameplay.unwrap().inventory, bag);
    let before = economy(&gateway, prizes);
    let screen = open_battle(&mut gateway, u64_field(&row["spawner"], "guid"), row, 300);
    enter_battle(&mut gateway, &screen, row, 400);
    let mut now = 2_000;
    let replies = fight_to_win(&mut gateway, &mut now, row);
    assert_eq!(won_flag(&replies), Some(1));
    let after = economy(&gateway, prizes);
    (gateway, before, after)
}

#[test]
fn a_forced_drop_lands_with_the_win_and_the_prize_stays_one() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let (gateway, before, after) = win(&content, YETI_BATTLE, &prizes, 8_601);
    let quantity = u32_field(&shipped_drops(YETI_BATTLE)[0], "quantity");
    assert_eq!(count(&gateway, item), quantity, "the drop is in the bag");
    // The wheel still pays coins, one prize and the active job's XP.
    assert_payout(&before, &after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

#[test]
fn a_zero_chance_drops_nothing_and_the_win_pays_as_before() {
    let content = content_with_chance(0.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let (gateway, before, after) = win(&content, YETI_BATTLE, &prizes, 8_602);
    assert_eq!(count(&gateway, item), 0, "chance 0 never drops");
    assert_payout(&before, &after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

#[test]
fn a_battle_without_a_drop_row_awards_nothing_new() {
    let content = content_with_chance(1.0);
    assert!(shipped_drops(PLAIN_BATTLE).is_empty());
    let file = battles();
    let row = battle(&file, PLAIN_BATTLE);
    let prizes = prize_ids(row);
    let (gateway, before, after) = win(&content, PLAIN_BATTLE, &prizes, 8_603);
    assert_eq!(count(&gateway, yeti_item()), 0, "no drop row, no drop");
    assert_payout(&before, &after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

/// The bag holds one stack of every prize (so the prize merges instead of
/// needing a slot) and `filler` stacks of an uncapped item that is neither a
/// prize nor the drop, up to `capacity` stacks.
fn bag_holding_prizes(
    base: InventoryState,
    prizes: &BTreeSet<i32>,
    item: i32,
    capacity: usize,
) -> InventoryState {
    let inventory = Inventory::load(&manifest_dir().join(RUNTIME_CONTENT), None).unwrap();
    let filler = inventory
        .definition_ids()
        .find(|id| {
            *id != item
                && !prizes.contains(id)
                && inventory
                    .trade_flags(*id)
                    .is_some_and(|(_, max_stack)| max_stack.is_none())
        })
        .expect("an uncapped filler item");
    let mut state = base;
    state
        .items
        .retain(|stack| stack.definition_id != item && !prizes.contains(&stack.definition_id));
    let mut id = state.items.iter().map(|stack| stack.id).max().unwrap_or(0) + 1;
    for &prize in prizes {
        state.items.push(stack(id, prize, 1));
        id += 1;
    }
    while state.items.len() < capacity {
        state.items.push(stack(id, filler, 1));
        id += 1;
    }
    state.next_instance_id = Some(i64::from(id));
    state
}

#[test]
fn a_bag_with_no_slot_loses_only_the_drop() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    // One free slot: the prize merges into its stack, the drop takes the slot.
    let (open, before, after) = win_with_bag(&content, YETI_BATTLE, &prizes, 8_604, |bag| {
        bag_holding_prizes(bag, &prizes, item, MAX_ITEMS - 1)
    });
    assert_eq!(count(&open, item), 1, "the drop fits the free slot");
    assert_payout(&before, &after, row, &prizes, BATTLE_JOB_XP);
    // No free slot: the same win pays coins, prize and XP and loses the drop.
    let (full, before, after) = win_with_bag(&content, YETI_BATTLE, &prizes, 8_605, |bag| {
        bag_holding_prizes(bag, &prizes, item, MAX_ITEMS)
    });
    assert_eq!(count(&full, item), 0, "a full bag cannot take the drop");
    assert_payout(&before, &after, row, &prizes, BATTLE_JOB_XP);
    let bag = full.snapshot().gameplay.unwrap().inventory;
    assert_eq!(bag.items.len(), MAX_ITEMS, "the bag did not grow");
    let _ = fs::remove_dir_all(content);
}

#[test]
fn the_shipped_rows_are_data_driven_and_grantable() {
    let inventory = Inventory::load(&manifest_dir().join(RUNTIME_CONTENT), None).unwrap();
    let file = battles();
    let mut with_drops = 0;
    for row in file["battles"].as_array().unwrap() {
        for drop in row["drops"].as_array().into_iter().flatten() {
            with_drops += 1;
            let item = i32_field(drop, "item_id");
            let quantity = drop["quantity"].as_u64().unwrap_or(1) as u32;
            assert!(
                inventory.accepts_reward(item, quantity),
                "battle {} drop {item} is not grantable by the catalogue",
                row["id"]
            );
            let chance = drop["chance"].as_f64().unwrap();
            assert!((0.0..=1.0).contains(&chance), "chance is a probability");
            let tags = drop["source"].as_array().unwrap();
            assert!(!tags.is_empty(), "every drop row carries provenance");
            for tag in tags.iter().filter(|t| t["kind"] == "reconstructed") {
                assert!(tag["evidence"].as_str().unwrap().starts_with("(R)"));
            }
        }
    }
    assert!(
        with_drops > 0,
        "the shipped content wires at least one drop"
    );
}
