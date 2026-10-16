//! Independent economy review for work items 081 (inventory instance lifecycle), 069 (quest
//! turn-in transactions) and 086 (currency and economy audit), 2026-10-09.
//!
//! This file is read-only over production. Tests named `integration_` drive the real production
//! call sites (`Inventory`, `Gameplay`, `game_store`) and are expected to FAIL until the lead
//! patches those call sites: they assert the invariant the item claims, not the current behavior.
//! Tests named `candidate_` and `guard_` pin the candidate modules and the surviving invariants
//! and must pass today.
//!
//! Production call sites under review:
//! * `src/inventory.rs:455` `grant_with_tint` allocates `max(item.id) + 1` and `src/inventory.rs:530`
//!   `consume` deletes a row at zero, so a retired top id is handed out again (item 081).
//! * `src/gameplay.rs:1775` `apply_quest` credits `grant_coins` and then each item as separate
//!   commits; a later failure leaves the coins and the `rewarded` flag without the item (piece 069).

#[path = "../src/restoration/piece_069.rs"]
#[allow(dead_code)]
mod piece_069;
#[path = "../src/restoration/piece_081.rs"]
#[allow(dead_code)]
mod piece_081;

use osfr_tools::{
    game_store,
    gameplay::{Gameplay, GameplayState},
    inventory::{Inventory, InventoryItem, InventoryState},
    quests::{QuestProgress, QuestState},
};
use piece_069::{
    QuestProgress as CandidateProgress, QuestState as CandidateQuestState, TurnInDecision,
    TurnInLedger, TurnInReceipt, TurnInRecovery, TurnInRejection, TurnInRequest, TurnInReward,
};
use piece_081::{CounterError, InstanceCounter};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// `src/inventory.rs:14` `MAX_ITEMS`. Read from production, pinned here so a change to the bound is
/// visible in this review instead of silently resizing the vector.
const PRODUCTION_MAX_ITEMS: usize = 4096;
/// `quests.json` quest 900001: 25 coins, item 1195 x1 (the quest the private golden turn-in uses).
const QUEST_ID: u32 = 900001;
const QUEST_COINS: u32 = 25;
const QUEST_ITEM: i32 = 1195;
/// Enemy position from the content's `combat.json`; attacks require being within 8 units.
const ENEMY_POSITION: [f32; 3] = [-1407.0, -28.2, 344.0];

static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-economy-review-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// The prepared private content. `OSFR_GAMEPLAY_CONTENT` wins; the lab default is the canonical
/// 1386-instance character that the private golden turn-in uses.
fn content_directory() -> PathBuf {
    std::env::var_os("OSFR_GAMEPLAY_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gameplay")
        })
}

fn chat(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1u64.to_le_bytes());
    packet.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0u32.to_le_bytes());
        packet.extend(0u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

/// One authored row for a definition and tint that no captured row uses, so the grant must mint a
/// new instance instead of merging. Returns the minted instance id.
fn grant_new_row(inventory: &mut Inventory, definition_id: i32, tint: i32) -> i32 {
    inventory
        .grant_with_tint(0, definition_id, 1, tint)
        .expect("the content has the definition");
    inventory
        .state()
        .items
        .iter()
        .find(|item| item.definition_id == definition_id && item.tint == tint)
        .expect("the grant minted the row")
        .id
}

fn item_count(inventory: &InventoryState, definition_id: i32) -> u32 {
    inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

fn quest_state() -> QuestState {
    let mut quests = BTreeMap::new();
    quests.insert(
        QUEST_ID,
        QuestProgress {
            accepted: true,
            counts: vec![0],
            completed: false,
            rewarded: false,
        },
    );
    QuestState {
        version: 1,
        quests,
        trial_best_ms: None,
        trial_completions: 0,
    }
}

/// A full bag that cannot merge the quest reward: every row that could merge with item 1195 is
/// moved off tint 0, and the bag is filled to the production `MAX_ITEMS` bound.
fn full_inventory_state(content: &Path) -> InventoryState {
    let inventory = Inventory::load(content, None).expect("content loads");
    let mut state = inventory.state().clone();
    for item in &mut state.items {
        if item.definition_id == QUEST_ITEM && item.tint == 0 {
            item.tint = 1;
        }
    }
    let next_id = state.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    while state.items.len() < PRODUCTION_MAX_ITEMS {
        let offset = (state.items.len() - inventory.state().items.len()) as i32;
        state.items.push(InventoryItem {
            definition_id: QUEST_ITEM,
            tint: 2,
            id: next_id + offset,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
    }
    assert_eq!(state.items.len(), PRODUCTION_MAX_ITEMS);
    state
}

/// Drive one training kill: respawn when the enemy is down, then land the four 25-damage hits.
fn kill_training_enemy(gameplay: &mut Gameplay, clock: &mut u64) -> io::Result<()> {
    gameplays_handle(gameplay, "realm respawn", *clock)?;
    *clock += 800;
    for _ in 0..4 {
        gameplays_handle(gameplay, "realm attack", *clock)?;
        *clock += 800;
    }
    Ok(())
}

fn gameplays_handle(gameplay: &mut Gameplay, command: &str, now_ms: u64) -> io::Result<()> {
    gameplay
        .handle(&chat(command), 1, ENEMY_POSITION, now_ms)
        .map(|_| ())
}

fn load_gameplay(content: &Path, inventory: InventoryState, quests: QuestState) -> Gameplay {
    let base = Gameplay::load(content, None).expect("content loads");
    let mut state = base.state();
    state.inventory = inventory;
    state.quests = quests;
    Gameplay::load(content, Some(state)).expect("the crafted state is valid production state")
}

// ---------------------------------------------------------------------------------------------
// Item 081: production instance-id lifecycle. These fail until the allocator is replaced.
// ---------------------------------------------------------------------------------------------

#[test]
fn integration_instance_ids_are_never_reused_after_a_top_row_retires() {
    let content = content_directory();
    let mut inventory = Inventory::load(&content, None).unwrap();

    // The minted row is the current maximum, so retiring it frees exactly that id.
    let retired = grant_new_row(&mut inventory, QUEST_ITEM, 17);
    inventory.consume(retired, 1).unwrap();
    let granted = grant_new_row(&mut inventory, QUEST_ITEM, 18);
    assert!(
        granted > retired,
        "a retired instance id must never be handed out again: {granted} == {retired}"
    );
}

#[test]
fn integration_restart_never_reuses_a_retired_top_instance_id() {
    let content = content_directory();
    let mut inventory = Inventory::load(&content, None).unwrap();
    let retired = grant_new_row(&mut inventory, QUEST_ITEM, 17);
    inventory.consume(retired, 1).unwrap();

    // A restart reloads the same rows; nothing durable records the retired id.
    let mut restarted = Inventory::load(&content, Some(inventory.state().clone())).unwrap();
    let granted = grant_new_row(&mut restarted, QUEST_ITEM, 18);
    assert!(
        granted > retired,
        "a restart must not hand a retired id out again: {granted} == {retired}"
    );
}

#[test]
fn integration_a_replayed_consume_cannot_retarget_a_reused_id() {
    let content = content_directory();
    let mut inventory = Inventory::load(&content, None).unwrap();

    // The client saw the first instance under `retired`, then the server reported it removed.
    let retired = grant_new_row(&mut inventory, QUEST_ITEM, 17);
    inventory.consume(retired, 1).unwrap();
    // A new, unrelated instance is then minted.
    let current = grant_new_row(&mut inventory, QUEST_ITEM, 18);

    // A replayed client consume for the retired id must not damage the new instance.
    let _ = inventory.consume(retired, 1);
    let survives = inventory
        .state()
        .items
        .iter()
        .any(|item| item.id == current && item.count == 1);
    assert!(
        survives,
        "a stale consume for {retired} retargeted the reused instance {current}"
    );
}

// ---------------------------------------------------------------------------------------------
// Item 081 candidate: the durable counter. These must pass today and are the lead's patch basis.
// ---------------------------------------------------------------------------------------------

#[test]
fn candidate_counter_is_monotonic_across_retire_and_restart() {
    let directory = TempDir::new("counter-monotonic");
    let path = directory.path().join("instance-counter.json");

    let mut counter = InstanceCounter::open(&path, &[3]).unwrap();
    let first = counter.allocate().unwrap();
    assert_eq!(first, 4);
    // The caller grants 4, then the row is consumed and the live set no longer holds it.
    let mut reopened = InstanceCounter::open(&path, &[3]).unwrap();
    let second = reopened.allocate().unwrap();
    assert_eq!(second, 5, "a retired id is never handed out twice");
    let mut reopened = InstanceCounter::open(&path, &[2, 3, 5]).unwrap();
    assert_eq!(
        reopened.allocate().unwrap(),
        6,
        "the counter floors above every live id and keeps increasing"
    );
}

#[test]
fn candidate_counter_burns_an_id_when_the_grant_never_lands() {
    let directory = TempDir::new("counter-burn");
    let path = directory.path().join("instance-counter.json");

    let mut counter = InstanceCounter::open(&path, &[]).unwrap();
    let burned = counter.allocate().unwrap();
    assert_eq!(burned, 1);
    // The caller's grant failed and no row exists, so the id stays burned.
    let mut reopened = InstanceCounter::open(&path, &[]).unwrap();
    assert_eq!(reopened.allocate().unwrap(), 2);
}

#[test]
fn candidate_counter_repairs_the_floor_and_exhausts_without_writing() {
    let directory = TempDir::new("counter-floor");
    let path = directory.path().join("instance-counter.json");
    fs::write(&path, r#"{"schema_version":1,"next_instance_id":5}"#).unwrap();

    // A live row above the stored value repairs the counter before the next hand-out.
    let counter = InstanceCounter::open(&path, &[9]).unwrap();
    assert_eq!(counter.next_instance_id(), 10);
    assert_eq!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("\"next_instance_id\": 10"),
        true,
        "the repaired floor is durable"
    );
    assert_eq!(counter.verify(&[9]).unwrap(), 9);

    let max = directory.path().join("exhausted.json");
    fs::write(
        &max,
        r#"{"schema_version":1,"next_instance_id":2147483647}"#,
    )
    .unwrap();
    let mut counter = InstanceCounter::open(&max, &[]).unwrap();
    assert_eq!(counter.allocate().unwrap(), i32::MAX);
    let exhausted = counter.allocate();
    assert_eq!(exhausted, Err(CounterError::Exhausted));
    assert_eq!(
        fs::read_to_string(&max)
            .unwrap()
            .contains("\"next_instance_id\": 2147483648"),
        true,
        "exhaustion must not rewrite the document"
    );
}

#[test]
fn candidate_counter_refuses_ambiguous_or_corrupt_ids() {
    let directory = TempDir::new("counter-reject");
    let path = directory.path().join("instance-counter.json");
    assert_eq!(
        InstanceCounter::open(&path, &[4, 4]).unwrap_err(),
        CounterError::DuplicateInstanceId { id: 4 }
    );
    assert_eq!(
        InstanceCounter::open(&path, &[0]).unwrap_err(),
        CounterError::InvalidInstanceId { id: 0 }
    );
    fs::write(&path, r#"{"schema_version":9,"next_instance_id":5}"#).unwrap();
    assert!(matches!(
        InstanceCounter::open(&path, &[]).unwrap_err(),
        CounterError::Corrupt(_)
    ));
    fs::write(&path, r#"{"schema_version":1,"next_instance_id":7}"#).unwrap();
    let counter = InstanceCounter::open(&path, &[]).unwrap();
    assert_eq!(
        counter.verify(&[7]).unwrap_err(),
        CounterError::BehindHighestInstance {
            next: 7,
            highest: 7
        }
    );
}

// ---------------------------------------------------------------------------------------------
// Item 069: production turn-in atomicity. The first two fail until gameplay.rs stages the reward.
// ---------------------------------------------------------------------------------------------

#[test]
fn integration_quest_turn_in_is_atomic_when_the_bag_is_full() {
    let content = content_directory();
    let full = full_inventory_state(&content);
    let coins_before = full.coins;
    let items_before = full.items.len();
    let reward_before = item_count(&full, QUEST_ITEM);
    let mut gameplay = load_gameplay(&content, full, quest_state());
    gameplays_handle(&mut gameplay, "realm quest accept 900001", 100).unwrap();

    let mut clock = 1_000;
    let first = kill_training_enemy(&mut gameplay, &mut clock);
    assert!(first.is_ok());
    let second = kill_training_enemy(&mut gameplay, &mut clock);
    assert!(second.is_ok());
    let third = kill_training_enemy(&mut gameplay, &mut clock);

    let state = gameplay.state();
    let progress = state.quests.quests.get(&QUEST_ID).unwrap();
    assert!(
        third.is_ok(),
        "a full bag must give a retryable refusal without ending the session"
    );
    assert_eq!(
        state.inventory.coins, coins_before,
        "coins must not land without the full bundle"
    );
    assert!(
        !progress.rewarded,
        "a partial turn-in must stay pending so it can be retried"
    );
    assert_eq!(state.inventory.items.len(), items_before);
    assert_eq!(item_count(&state.inventory, QUEST_ITEM), reward_before);
}

#[test]
fn integration_a_failed_turn_in_is_not_persisted_as_a_partial_reward() {
    let content = content_directory();
    let full = full_inventory_state(&content);
    let coins_before = full.coins;
    let mut gameplay = load_gameplay(&content, full, quest_state());
    gameplays_handle(&mut gameplay, "realm quest accept 900001", 100).unwrap();

    let mut clock = 1_000;
    kill_training_enemy(&mut gameplay, &mut clock).unwrap();
    kill_training_enemy(&mut gameplay, &mut clock).unwrap();
    let _ = kill_training_enemy(&mut gameplay, &mut clock);

    // Whatever the process reports is what a restart would reload from the same JSON.
    let bytes = serde_json::to_vec(&gameplay.state()).unwrap();
    let reloaded: GameplayState = serde_json::from_slice(&bytes).unwrap();
    let restarted = Gameplay::load(&content, Some(reloaded)).unwrap();
    let state = restarted.state();
    let progress = state.quests.quests.get(&QUEST_ID).unwrap();
    assert_eq!(
        state.inventory.coins, coins_before,
        "a restart must not reload coins from a turn-in whose items never landed"
    );
    assert!(
        !progress.rewarded,
        "a restart must not reload a rewarded quest whose items never landed"
    );
}

// ---------------------------------------------------------------------------------------------
// Item 069 candidate: the staged intent ledger. These must pass today.
// ---------------------------------------------------------------------------------------------

fn candidate_reward() -> TurnInReward {
    TurnInReward {
        coins: QUEST_COINS,
        experience: 0,
        items: vec![(QUEST_ITEM, 1)],
    }
}

fn candidate_progress(completed: bool, rewarded: bool) -> CandidateProgress {
    CandidateProgress {
        accepted: true,
        counts: vec![3],
        completed,
        rewarded,
    }
}

fn candidate_state(progress: CandidateProgress) -> CandidateQuestState {
    let mut quests = BTreeMap::new();
    quests.insert(QUEST_ID, progress);
    CandidateQuestState {
        version: 1,
        quests,
        trial_best_ms: None,
        trial_completions: 0,
    }
}

#[test]
fn candidate_ledger_records_intent_before_apply_and_settles_after() {
    let directory = TempDir::new("ledger-intent");
    let path = directory.path().join("turn-in-ledger.jsonl");
    let request = TurnInRequest {
        quest_id: QUEST_ID,
        attempt: 1,
        request_id: 7,
    };
    let ready = candidate_progress(true, false);

    let mut ledger = TurnInLedger::open(&path).unwrap();
    let decision = ledger
        .consider(request, Some(&ready), &candidate_reward())
        .unwrap();
    let staged = match decision {
        TurnInDecision::Apply(staged) => staged,
        other => panic!("expected an apply, got {other:?}"),
    };
    assert!(
        fs::read_to_string(&path)
            .unwrap()
            .contains("\"kind\":\"intent\""),
        "the intent must be durable before the caller applies it"
    );
    assert_eq!(staged.reward().coins, QUEST_COINS);

    // Crash before the snapshot landed: recovery replays the exact bundle once.
    let mut reopened = TurnInLedger::open(&path).unwrap();
    let actions = reopened.recover(&candidate_state(ready.clone())).unwrap();
    assert_eq!(actions.len(), 1);
    let replayed = match &actions[0] {
        TurnInRecovery::Replay(staged) => staged.clone(),
        other => panic!("expected a replay, got {other:?}"),
    };
    assert_eq!(replayed.request_id(), 7);
    assert_eq!(replayed.reward(), &candidate_reward());

    // The same request again is the same staged bundle, not a second intent.
    let records_before = reopened.records();
    let decision = reopened
        .consider(request, Some(&ready), &candidate_reward())
        .unwrap();
    assert!(matches!(decision, TurnInDecision::Apply(_)));
    assert_eq!(reopened.records(), records_before);

    // Commit once the snapshot with the reward has landed; repeats are idempotent.
    let receipt = reopened.commit(replayed.clone()).unwrap();
    assert_eq!(
        receipt,
        TurnInReceipt {
            quest_id: QUEST_ID,
            attempt: 1,
            request_id: 7,
            settled_from_state: false
        }
    );
    assert_eq!(reopened.commit(replayed).unwrap(), receipt);
    let mut reopened = TurnInLedger::open(&path).unwrap();
    assert!(
        reopened
            .recover(&candidate_state(candidate_progress(true, true)))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn candidate_ledger_settles_from_persisted_state_and_refuses_unproven_rewards() {
    let directory = TempDir::new("ledger-settle");
    let path = directory.path().join("turn-in-ledger.jsonl");
    let request = TurnInRequest {
        quest_id: QUEST_ID,
        attempt: 1,
        request_id: 7,
    };
    let mut ledger = TurnInLedger::open(&path).unwrap();
    ledger
        .consider(
            request,
            Some(&candidate_progress(true, false)),
            &candidate_reward(),
        )
        .unwrap();

    // The snapshot landed before the applied record: recovery settles without a second grant.
    let actions = ledger
        .recover(&candidate_state(candidate_progress(true, true)))
        .unwrap();
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        TurnInRecovery::Settled(receipt) => assert!(receipt.settled_from_state),
        other => panic!("expected a settle, got {other:?}"),
    }
    let decision = ledger
        .consider(
            request,
            Some(&candidate_progress(true, true)),
            &candidate_reward(),
        )
        .unwrap();
    assert!(matches!(decision, TurnInDecision::Replay(_)));

    // A rewarded quest this ledger never recorded is refused, not replayed.
    let unproven = TempDir::new("ledger-unproven");
    let unproven_path = unproven.path().join("turn-in-ledger.jsonl");
    let mut ledger = TurnInLedger::open(&unproven_path).unwrap();
    let decision = ledger
        .consider(
            TurnInRequest {
                quest_id: QUEST_ID,
                attempt: 2,
                request_id: 9,
            },
            Some(&candidate_progress(true, true)),
            &candidate_reward(),
        )
        .unwrap();
    assert_eq!(
        decision,
        TurnInDecision::Reject(TurnInRejection::AlreadyRewarded)
    );
    assert_eq!(ledger.records(), 0, "a refused request records nothing");
}

#[test]
fn candidate_ledger_refuses_stale_attempts_without_a_second_intent() {
    let directory = TempDir::new("ledger-stale");
    let path = directory.path().join("turn-in-ledger.jsonl");
    let mut ledger = TurnInLedger::open(&path).unwrap();
    let ready = candidate_progress(true, false);
    let first = TurnInRequest {
        quest_id: QUEST_ID,
        attempt: 1,
        request_id: 7,
    };
    ledger
        .consider(first, Some(&ready), &candidate_reward())
        .unwrap();
    let stale = TurnInRequest {
        quest_id: QUEST_ID,
        attempt: 2,
        request_id: 8,
    };
    assert_eq!(
        ledger
            .consider(stale, Some(&ready), &candidate_reward())
            .unwrap(),
        TurnInDecision::Reject(TurnInRejection::StaleAttempt { recorded: 1 })
    );
    let reused = TurnInRequest {
        quest_id: QUEST_ID,
        attempt: 1,
        request_id: 8,
    };
    assert!(matches!(
        ledger
            .consider(reused, Some(&ready), &candidate_reward())
            .unwrap(),
        TurnInDecision::Reject(TurnInRejection::StaleAttempt { .. })
    ));
    assert_eq!(ledger.records(), 1);
}

// ---------------------------------------------------------------------------------------------
// Item 086: the economy guard. These must pass today.
// ---------------------------------------------------------------------------------------------

#[test]
fn guard_coin_balance_refuses_overflow_and_keeps_the_bundle_atomic() {
    let content = content_directory();
    let inventory = Inventory::load(&content, None).unwrap();
    let mut state = inventory.state().clone();
    state.coins = i32::MAX as u32;
    let mut inventory = Inventory::load(&content, Some(state)).unwrap();
    assert!(inventory.grant_coins(1).is_err(), "i32 coin overflow");
    assert_eq!(inventory.state().coins, i32::MAX as u32);

    let mut state = inventory.state().clone();
    state.coins = i32::MAX as u32 - 5;
    let items_before = state.items.len();
    let mut inventory = Inventory::load(&content, Some(state)).unwrap();
    assert!(
        inventory.grant_with_tint(10, QUEST_ITEM, 1, 31).is_err(),
        "a bundle whose coins overflow must fail before the item lands"
    );
    assert_eq!(inventory.state().coins, i32::MAX as u32 - 5);
    assert_eq!(inventory.state().items.len(), items_before);
    assert!(
        !inventory
            .state()
            .items
            .iter()
            .any(|item| item.definition_id == QUEST_ITEM && item.tint == 31),
        "the refused bundle must not mint the item"
    );
}

#[test]
fn guard_full_bag_observation_for_the_086_bounds() {
    // Measurement, not a pass/fail gate: record how a production-legal full bag interacts with the
    // 1 MiB snapshot bound that `game_store::save` enforces, so the lead can size the fix.
    let content = content_directory();
    let full = full_inventory_state(&content);
    let inventory_bytes = serde_json::to_vec(&full).unwrap().len();
    let gameplay_bytes = serde_json::to_vec(&load_gameplay(&content, full, quest_state()).state())
        .unwrap()
        .len();
    println!(
        "full-bag measurement: inventory={inventory_bytes} bytes gameplay={gameplay_bytes} bytes \
         snapshot_bound={} bytes",
        game_store::MAX_SNAPSHOT_BYTES
    );
    assert!(gameplay_bytes > 0);
}

#[test]
fn guard_reward_producers_stay_once_only_in_the_golden_path() {
    // The observable once-only guarantee the private capture already accepts: three defeats land
    // exactly one bundle, and a trailing defeat grants nothing.
    let content = content_directory();
    let base = Inventory::load(&content, None).unwrap();
    let coins_before = base.state().coins;
    let reward_before = item_count(base.state(), QUEST_ITEM);
    let mut gameplay = Gameplay::load(&content, None).unwrap();
    gameplays_handle(&mut gameplay, "realm quest accept 900001", 100).unwrap();

    let mut clock = 1_000;
    for _ in 0..3 {
        kill_training_enemy(&mut gameplay, &mut clock).unwrap();
    }
    let state = gameplay.state();
    assert_eq!(state.inventory.coins, coins_before + QUEST_COINS);
    assert_eq!(
        item_count(&state.inventory, QUEST_ITEM),
        reward_before + 1,
        "exactly one reward instance"
    );
    assert!(state.quests.quests[&QUEST_ID].rewarded);

    kill_training_enemy(&mut gameplay, &mut clock).unwrap();
    let state = gameplay.state();
    assert_eq!(state.inventory.coins, coins_before + QUEST_COINS);
    assert_eq!(
        item_count(&state.inventory, QUEST_ITEM),
        reward_before + 1,
        "a trailing defeat must not pay twice"
    );
}

#[test]
fn integration_pending_reward_survives_restart_and_retries_once_after_space_is_freed() {
    let content = content_directory();
    let full = full_inventory_state(&content);
    let coins_before = full.coins;
    let mut gameplay = load_gameplay(&content, full, quest_state());
    let mut clock = 1_000;
    for _ in 0..3 {
        kill_training_enemy(&mut gameplay, &mut clock).unwrap();
    }
    let mut state = gameplay.state();
    assert!(state.quests.quests[&QUEST_ID].completed);
    assert!(!state.quests.quests[&QUEST_ID].rewarded);
    assert_eq!(state.quests.quests[&QUEST_ID].counts, vec![3]);
    state.inventory.items.pop();
    let before_items = item_count(&state.inventory, QUEST_ITEM);
    let bytes = serde_json::to_vec(&state).unwrap();
    let mut restarted =
        Gameplay::load(&content, Some(serde_json::from_slice(&bytes).unwrap())).unwrap();
    kill_training_enemy(&mut restarted, &mut clock).unwrap();
    let awarded = restarted.state();
    assert!(awarded.quests.quests[&QUEST_ID].rewarded);
    assert_eq!(awarded.inventory.coins, coins_before + QUEST_COINS);
    assert_eq!(item_count(&awarded.inventory, QUEST_ITEM), before_items + 1);
    kill_training_enemy(&mut restarted, &mut clock).unwrap();
    assert_eq!(restarted.state().inventory, awarded.inventory);
}

#[test]
fn integration_corrupt_counter_is_refused_and_exhaustion_preserves_the_whole_inventory() {
    let content = content_directory();
    let base = Inventory::load(&content, None).unwrap().state().clone();
    for invalid in [0, 1, i64::from(i32::MAX) + 2] {
        let mut state = base.clone();
        state.next_instance_id = Some(invalid);
        assert!(Inventory::load(&content, Some(state)).is_err());
    }
    let mut state = base;
    state.next_instance_id = Some(i64::from(i32::MAX) + 1);
    let mut inventory = Inventory::load(&content, Some(state)).unwrap();
    let before = inventory.state().clone();
    assert!(inventory.grant_with_tint(10, QUEST_ITEM, 1, 999).is_err());
    assert_eq!(inventory.state(), &before);
}

#[test]
fn integration_bundle_failure_keeps_coins_items_and_counter_unchanged() {
    let content = content_directory();
    let mut inventory = Inventory::load(&content, None).unwrap();
    let before = inventory.state().clone();
    assert!(
        inventory
            .grant_bundle(25, &[(QUEST_ITEM, 1), (i32::MAX, 1)])
            .is_err()
    );
    assert_eq!(inventory.state(), &before);
}
