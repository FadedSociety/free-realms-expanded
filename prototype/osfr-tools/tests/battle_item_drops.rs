//! T658: a field-battle win can add a chance item drop beside its prize wheel.
//!
//! Every test drives the production route: marker click -> Go -> melee kills ->
//! results. The drop rows are the shipped `drops` of `field_battles.json`; only
//! the chance is rewritten in a private content copy (1.0 forces the drop, 0.0
//! retires it), so no pinned rate and no test seam is involved. The shipped
//! content is never edited.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    coin_shop::CoinShop,
    gateway::Gateway,
    inventory::{Inventory, InventoryState, MAX_ITEMS},
    profiles::Profiles,
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
            content_copy::hard_link_or_copy(&entry.path(), &root.join(entry.file_name()));
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

/// The NPC content `Gameplay::load` binds for this runtime package: its own
/// `npcs/` when it has one, else the bundled `data/content`.
fn npc_content() -> PathBuf {
    let runtime = manifest_dir().join(RUNTIME_CONTENT);
    if runtime.join("npc_spawns_extra.json").is_file() && runtime.join("npcs").is_dir() {
        runtime
    } else {
        content_dir()
    }
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

/// How a test wins the battle: the production melee route, or the developer
/// `realm win`, which never grants a chance drop.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Route {
    Fight,
    DevWin,
}

/// One finished run: everything a test asserts about the production route.
struct BattleRun {
    fixture: Fixture,
    gateway: Gateway,
    /// The bag the fight started with, after `shape`.
    bag: InventoryState,
    before: Economy,
    after: Economy,
    replies: Vec<Vec<u8>>,
}

/// Win `battle_id` once from its marker through the melee route.
fn win(content: &Path, battle_id: u32, prizes: &BTreeSet<i32>, guid: u64) -> BattleRun {
    run_battle(content, battle_id, prizes, guid, Route::Fight, |bag| bag)
}

/// `win`, with the bag replaced by `shape(bag)` before the fight (reloaded
/// through the production `load_gameplay` restore path, as a save would be).
fn run_battle(
    content: &Path,
    battle_id: u32,
    prizes: &BTreeSet<i32>,
    guid: u64,
    route: Route,
    shape: impl FnOnce(InventoryState) -> InventoryState,
) -> BattleRun {
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
    let replies = match route {
        Route::Fight => fight_to_win(&mut gateway, &mut now, row),
        Route::DevWin => send(&mut gateway, &chat::local_request("realm win"), now).replies,
    };
    assert_eq!(won_flag(&replies), Some(1));
    let after = economy(&gateway, prizes);
    BattleRun {
        fixture,
        gateway,
        bag,
        before,
        after,
        replies,
    }
}

#[test]
fn a_forced_drop_lands_with_the_win_and_the_prize_stays_one() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let run = win(&content, YETI_BATTLE, &prizes, 8_601);
    let quantity = u32_field(&shipped_drops(YETI_BATTLE)[0], "quantity");
    assert_eq!(
        count(&run.gateway, item),
        quantity,
        "the drop is in the bag"
    );
    // The wheel still pays coins, one prize and the active job's XP.
    assert_payout(&run.before, &run.after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

#[test]
fn a_zero_chance_drops_nothing_and_the_win_pays_as_before() {
    let content = content_with_chance(0.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let run = win(&content, YETI_BATTLE, &prizes, 8_602);
    assert_eq!(count(&run.gateway, item), 0, "chance 0 never drops");
    assert_payout(&run.before, &run.after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

#[test]
fn a_battle_without_a_drop_row_awards_nothing_new() {
    let content = content_with_chance(1.0);
    assert!(shipped_drops(PLAIN_BATTLE).is_empty());
    let file = battles();
    let row = battle(&file, PLAIN_BATTLE);
    let prizes = prize_ids(row);
    let run = win(&content, PLAIN_BATTLE, &prizes, 8_603);
    assert_eq!(count(&run.gateway, yeti_item()), 0, "no drop row, no drop");
    assert_payout(&run.before, &run.after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

#[test]
fn a_bag_with_no_slot_loses_only_the_drop() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    // One free slot: the prize merges into its stack, the drop takes the slot.
    let open = run_battle(&content, YETI_BATTLE, &prizes, 8_604, Route::Fight, |bag| {
        full_bag(&bag, MAX_ITEMS - 1, YETI_BATTLE, PrizeStacks::Keep)
    });
    assert_eq!(count(&open.gateway, item), 1, "the drop fits the free slot");
    assert_payout(&open.before, &open.after, row, &prizes, BATTLE_JOB_XP);
    // No free slot: the same win pays coins, prize and XP and loses the drop.
    let full = run_battle(&content, YETI_BATTLE, &prizes, 8_605, Route::Fight, |bag| {
        full_bag(&bag, MAX_ITEMS, YETI_BATTLE, PrizeStacks::Keep)
    });
    assert_eq!(
        count(&full.gateway, item),
        0,
        "a full bag cannot take the drop"
    );
    assert!(
        full.bag
            .items
            .iter()
            .all(|stack| stack.definition_id != item),
        "the shaped bag starts without the chance drop"
    );
    assert_payout(&full.before, &full.after, row, &prizes, BATTLE_JOB_XP);
    let bag = full.gateway.snapshot().gameplay.unwrap().inventory;
    assert_eq!(bag.items.len(), MAX_ITEMS, "the bag did not grow");
    let _ = fs::remove_dir_all(content);
}

/// (R) The refusal line `encounter_rewards.rs` owns; no recovered locale row
/// names it, and T658 review item 8 added the chance drop to its wording.
const REWARD_REFUSED: &str = "This reward, any prize item and any chance drop it carried were not granted: your coin balance or active job is at its limit.";

/// T658 review item 1/2: `realm win` is the developer shortcut, so it pays the
/// deterministic prize, coins and XP but never mints the chance drop. A test
/// that inspects the bag through the dev route must not be a coin flip.
#[test]
fn the_dev_win_never_grants_the_chance_drop() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let run = run_battle(
        &content,
        YETI_BATTLE,
        &prizes,
        8_606,
        Route::DevWin,
        |bag| bag,
    );
    let held: u32 = run
        .bag
        .items
        .iter()
        .filter(|stack| stack.definition_id == item)
        .map(|stack| stack.count)
        .sum();
    assert_eq!(
        count(&run.gateway, item),
        held,
        "realm win never mints the drop"
    );
    assert_payout(&run.before, &run.after, row, &prizes, BATTLE_JOB_XP);
    let _ = fs::remove_dir_all(content);
}

/// T658 review item 8: a player at the coin cap loses the whole win reward,
/// including the chance drop, is told so, and no inventory byte changes.
#[test]
fn a_capped_win_refuses_the_drop_with_the_rest_and_changes_nothing() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let mut run = run_battle(
        &content,
        YETI_BATTLE,
        &prizes,
        8_607,
        Route::Fight,
        |mut bag| {
            bag.coins = i32::MAX as u32;
            bag
        },
    );
    assert_eq!(
        count(&run.gateway, item),
        0,
        "the refused drop is not granted"
    );
    assert_eq!(run.after, run.before, "the refused reward changes nothing");
    assert_eq!(
        run.gateway.snapshot().gameplay.unwrap().inventory,
        run.bag,
        "no stack, coin or the drop was touched"
    );
    assert!(
        run.replies.iter().any(|reply| reply
            .windows(REWARD_REFUSED.len())
            .any(|window| window == REWARD_REFUSED.as_bytes())),
        "the player is told the chance drop went with the refused reward"
    );
    // F681 review item 6: a refused win still completes, so the results Close
    // must free the player and leave the bag and economy exactly as they were.
    let closed = send(&mut run.gateway, &[26, 0, 42, 0], 5_000);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(
        run.gateway.snapshot().gameplay.unwrap().inventory,
        run.bag,
        "the results Close does not alter the refused bag"
    );
    assert_eq!(
        economy(&run.gateway, &prizes),
        run.before,
        "the refused reward stays refused after Close"
    );
    let _ = fs::remove_dir_all(content);
}

/// T658 review item 3: the drop lands in the saved bag, survives the results
/// Close, and reloads through the production `game_store` reconnect.
#[test]
fn a_forced_drop_survives_the_results_close_and_a_reconnect() {
    let content = content_with_chance(1.0);
    let item = yeti_item();
    let quantity = u32_field(&shipped_drops(YETI_BATTLE)[0], "quantity");
    let file = battles();
    let row = battle(&file, YETI_BATTLE);
    let prizes = prize_ids(row);
    let mut run = win(&content, YETI_BATTLE, &prizes, 8_608);
    assert_eq!(count(&run.gateway, item), quantity, "the win pays the drop");
    let closed = send(&mut run.gateway, &[26, 0, 42, 0], 5_000);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_eq!(
        count(&run.gateway, item),
        quantity,
        "the results Close keeps the drop"
    );
    let reopened = reopen(&run.fixture, &run.gateway, "drop-save", 6_000);
    assert_eq!(
        count(&reopened, item),
        quantity,
        "the saved bag keeps the drop"
    );
    let _ = fs::remove_dir_all(content);
}

#[test]
fn the_shipped_rows_are_data_driven_and_grantable() {
    let runtime = manifest_dir().join(RUNTIME_CONTENT);
    let inventory = Inventory::load(&runtime, None).unwrap();
    let profiles = Profiles::load(&runtime, None).unwrap();
    // T658 review item 9: a repeatable 25% drop must not be a coin faucet.
    // `CoinShop::sell_unit_price` is the one effective sale price the Sell tab
    // uses (store row, free issuance or the client Worth fallback), so a
    // runtime catalogue or code-level price fails here before it can be farmed.
    let shop = CoinShop::load(
        &runtime,
        &npc_content(),
        true,
        inventory.clone(),
        Some(&profiles),
        None,
    )
    .expect("the shipped shop loads")
    .expect("the shipped content enables the shop");
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
            let price = shop.sell_unit_price(item).unwrap_or(0);
            assert_eq!(
                price, 0,
                "drop {item} sells for {price} coins: 25% per win becomes a coin faucet"
            );
        }
    }
    assert!(
        with_drops > 0,
        "the shipped content wires at least one drop"
    );
}
