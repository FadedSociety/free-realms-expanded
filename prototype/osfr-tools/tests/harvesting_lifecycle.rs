//! Harvesting's shared lifecycle, seeded payout and cancellation acceptance.
//!
//! The seeded replay drives production type-8 and reward code. Timing, quotas,
//! seed and the simulated four-second input interval are reconstructed policy.

use osfr_tools::{
    harvesting::{
        self, CHEF_PROFILE_ID, CONTENT_FILE, FIRST_INSTANCE_ID, RECONSTRUCTED_PER_REWARD_TILES,
        RECONSTRUCTED_QUOTA_TILES, RECONSTRUCTED_SEED,
        lifecycle::{HarvestingActivityOutcome, HarvestingLifecycle},
    },
    inventory::{Inventory, InventoryState},
    minigame::RunState,
    mining::{TILE_WILDCARD, Type8Phase, Type8Run},
    profiles::ProfileState,
    quest_engine::QuestEvent,
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const GAME_ID: i32 = 111;
const PLAYER_GUID: u64 = 801;
const SIDE: usize = 11;
const LEFT: usize = 1;
const RIGHT: usize = 10;
const TOP: usize = 1;
const BOTTOM: usize = 9;
const MAX_CHAIN: usize = 12;
const NODE_BUDGET: usize = 100_000;
const INPUT_INTERVAL_MS: u32 = 4_000;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-harvesting-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        harvesting::write_content(&path, GAME_ID).unwrap();

        let item_ids: Vec<i32> = harvesting::region(GAME_ID)
            .unwrap()
            .produce
            .iter()
            .map(|slug| harvesting::produce(slug).unwrap().item_definition_id)
            .collect();
        let definitions: Vec<Value> = item_ids
            .iter()
            .map(|&id| {
                let bytes = [id.to_le_bytes(), 8_i32.to_le_bytes()].concat();
                json!({
                    "id": id,
                    "slot": 0,
                    "class_id": 0,
                    "default_tint": 0,
                    "max_stack": -1,
                    "model": "",
                    "texture": "",
                    "tint_alias": "dyetint",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": hex(&bytes),
                    "sha256": digest(&bytes)
                })
            })
            .collect();
        let empty = 0_i32.to_le_bytes();
        let inventory = json!({
            "schema_version": 1,
            "player_guid": PLAYER_GUID,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &0_i32.to_le_bytes()),
                span("items", 17, 0, &empty),
                span("equipment", 21, 1, &empty)
            ],
            "definitions": definitions
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&inventory).unwrap(),
        )
        .unwrap();
        Self { path }
    }

    fn lifecycle(&self) -> HarvestingLifecycle {
        HarvestingLifecycle::load(
            &self.path,
            CONTENT_FILE,
            GAME_ID,
            FIRST_INSTANCE_ID,
            PLAYER_GUID,
            None,
        )
        .unwrap()
    }

    fn rewards(&self) -> RewardState {
        let mut inventory = Inventory::load(&self.path, None).unwrap();
        for slug in harvesting::region(GAME_ID).unwrap().produce {
            inventory
                .grant(0, harvesting::produce(slug).unwrap().item_definition_id, 2)
                .unwrap();
        }
        RewardState::new(
            inventory,
            ProfileState {
                version: 1,
                active: CHEF_PROFILE_ID,
                unlocked: BTreeSet::from([CHEF_PROFILE_ID]),
                experience: BTreeMap::new(),
            },
            8,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("inventory.json"));
        let _ = fs::remove_file(self.path.join(CONTENT_FILE));
        let _ = fs::remove_dir(&self.path);
    }
}

fn span(kind: &str, offset: usize, profile_id: i32, bytes: &[u8]) -> Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": digest(bytes)
    })
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn header(sub: u8, id: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, sub];
    for value in [id, -1, -1] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

fn movie(id: i32, message: &str) -> Vec<u8> {
    let mut bytes = header(14, id);
    bytes.extend(((message.len() + 1) as i32).to_le_bytes());
    bytes.extend(message.as_bytes());
    bytes.push(0);
    bytes
}

fn begin_play(lifecycle: &mut HarvestingLifecycle) -> (i32, Vec<String>) {
    let offer = lifecycle.begin(CHEF_PROFILE_ID).unwrap();
    assert_eq!(offer.state, Some(RunState::Offered));
    assert_eq!(offer.replies[0][..3], [39, 0, 16]);
    let id = FIRST_INSTANCE_ID;
    let start = lifecycle.handle(&header(5, id), None).unwrap();
    assert_eq!(start.state, Some(RunState::Starting));
    assert_eq!(start.replies.len(), 2);

    let mut messages = Vec::new();
    for command in ["OnConnectMsg", "OnStartGameMsg", "OnStartLevelMsg"] {
        let out = lifecycle.handle(&movie(id, command), None).unwrap();
        messages.extend(movie_messages(&out));
    }
    assert_eq!(lifecycle.run_state(), Some(RunState::Playing));
    (id, messages)
}

fn movie_messages(out: &HarvestingActivityOutcome) -> Vec<String> {
    out.replies
        .iter()
        .filter(|packet| packet.len() >= 20 && packet[..3] == [39, 0, 14])
        .map(|packet| {
            let size = i32::from_le_bytes(packet[15..19].try_into().unwrap()) as usize;
            assert_eq!(size, packet.len() - 19);
            assert_eq!(packet.last(), Some(&0));
            String::from_utf8(packet[19..packet.len() - 1].to_vec()).unwrap()
        })
        .collect()
}

fn acknowledge_idle(lifecycle: &mut HarvestingLifecycle, id: i32) {
    let Some(run) = lifecycle.activity_state().active.as_ref() else {
        return;
    };
    if run.phase != Type8Phase::Playing {
        return;
    }
    let width = harvesting::board_spec(GAME_ID).unwrap().width as usize;
    let mut idle = String::new();
    for y in TOP..=BOTTOM {
        for x in LEFT..=RIGHT {
            let index = y * width + x;
            if run.board[index] != 0 && !run.idle[index] {
                idle.push_str(&format!("{x},{y},"));
            }
        }
    }
    if !idle.is_empty() {
        lifecycle
            .handle(&movie(id, &format!("OnIdleMsg\t{idle}")), None)
            .unwrap();
    }
}

fn find_chain(run: &Type8Run) -> Option<Vec<(usize, usize)>> {
    type Rank = (u32, usize);

    fn explore(
        run: &Type8Run,
        target: u16,
        max_len: usize,
        path: &mut Vec<(usize, usize)>,
        seen: &mut BTreeSet<(usize, usize)>,
        budget: &mut usize,
        best: &mut Option<Vec<(usize, usize)>>,
    ) {
        if *budget == 0 {
            return;
        }
        *budget -= 1;
        if path.len() >= 3 && best.as_ref().is_none_or(|old| path.len() > old.len()) {
            *best = Some(path.clone());
        }
        if path.len() >= max_len {
            return;
        }
        let &(x, y) = path.last().unwrap();
        for (nx, ny) in [
            (x + 1, y),
            (x, y + 1),
            (x.wrapping_sub(1), y),
            (x, y.wrapping_sub(1)),
        ] {
            if !(LEFT..=RIGHT).contains(&nx)
                || !(TOP..=BOTTOM).contains(&ny)
                || seen.contains(&(nx, ny))
            {
                continue;
            }
            let index = ny * SIDE + nx;
            let value = run.board[index];
            if !run.idle[index] || (value != target && value != TILE_WILDCARD) {
                continue;
            }
            seen.insert((nx, ny));
            path.push((nx, ny));
            explore(run, target, max_len, path, seen, budget, best);
            path.pop();
            seen.remove(&(nx, ny));
        }
    }

    let spec = harvesting::board_spec(GAME_ID).ok()?;
    let mut best_rank: Option<Rank> = None;
    let mut best_path = None;
    for (index, goal) in spec.goals.iter().enumerate() {
        let collected = *run.collected.get(index)?;
        if collected >= goal.quota {
            continue;
        }
        // (R) Permit one final chain to cross the tile quota, while bounding
        // the overshoot to less than one additional bundle.
        let max_count = goal.quota + goal.per_reward - 1;
        let max_len = (max_count.saturating_sub(collected) as usize).min(MAX_CHAIN);
        if max_len < 3 {
            continue;
        }
        let mut best_for_goal = None;
        let mut budget = NODE_BUDGET;
        for y in TOP..=BOTTOM {
            for x in LEFT..=RIGHT {
                let cell = run.board[y * SIDE + x];
                if !run.idle[y * SIDE + x] || (cell != goal.value && cell != TILE_WILDCARD) {
                    continue;
                }
                let mut path = vec![(x, y)];
                let mut seen = BTreeSet::from([(x, y)]);
                explore(
                    run,
                    goal.value,
                    max_len,
                    &mut path,
                    &mut seen,
                    &mut budget,
                    &mut best_for_goal,
                );
            }
        }
        let Some(path) = best_for_goal else {
            continue;
        };
        let rank = (u32::MAX - collected, path.len());
        if best_rank.is_none_or(|old| rank > old) {
            best_rank = Some(rank);
            best_path = Some(path);
        }
    }
    best_path
}

fn drive_round(lifecycle: &mut HarvestingLifecycle) -> Option<(i32, u32)> {
    let (id, messages) = begin_play(lifecycle);
    let spec = harvesting::board_spec(GAME_ID).unwrap();
    for (index, goal) in spec.goals.iter().enumerate() {
        let quota = format!(
            "OnQuotaInitMsg\t{index}\t{}\t{}\t{}\t{}",
            goal.value, goal.quota, goal.per_reward, goal.icon_id
        );
        assert!(messages.iter().any(|message| message == &quota), "{quota}");
        let frame = i32::from(goal.value - 400);
        assert!(
            messages
                .iter()
                .any(|message| message == &format!("OnTileInfoMsg\t{}\t{frame}", goal.value)),
            "missing SelectableTile{frame} registration for {}",
            goal.value
        );
    }

    let mut actions = 0;
    loop {
        let run = lifecycle.activity_state().active.as_ref().unwrap();
        if run.phase == Type8Phase::LevelComplete {
            break;
        }
        assert_eq!(run.phase, Type8Phase::Playing);
        let Some(path) = find_chain(run) else {
            return None;
        };
        let coords = path
            .iter()
            .map(|(x, y)| format!("{x},{y},"))
            .collect::<String>();
        lifecycle
            .handle(&movie(id, &format!("OnChainMsg\t{coords}")), None)
            .unwrap();
        acknowledge_idle(lifecycle, id);
        if lifecycle
            .activity_state()
            .active
            .as_ref()
            .is_some_and(|run| run.phase == Type8Phase::Playing)
        {
            lifecycle.tick(INPUT_INTERVAL_MS).unwrap();
        }
        actions += 1;
        if actions > 40 {
            return None;
        }
    }

    let run = lifecycle.activity_state().active.as_ref().unwrap();
    assert_eq!(run.collected.len(), 4);
    assert!(
        run.collected
            .iter()
            .all(|&count| count >= RECONSTRUCTED_QUOTA_TILES)
    );
    assert!(run.remaining_ms > 0, "round exhausted its timer");
    assert!(
        run.wall_ms <= spec.duration_seconds * 1000,
        "round exceeded its reconstructed timer"
    );
    Some((id, actions))
}

fn count_item(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn seeded_full_round_grants_produce_chef_xp_and_quest_events_once() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let before = fixture.rewards();
    let before_inventory = before.inventory.state().clone();
    let (id, actions) = drive_round(&mut lifecycle)
        .unwrap_or_else(|| panic!("seeded Harvesting round reached a board with no legal chain"));
    assert_eq!(
        actions, 22,
        "the region-111 seed-2 replay stays deterministic"
    );
    assert!(actions * INPUT_INTERVAL_MS <= 120_000);

    let win = lifecycle
        .handle(&movie(id, "OnNextLevelMsg"), Some(&before))
        .unwrap();
    assert_eq!(win.state, Some(RunState::Won));
    assert_eq!(win.replies.len(), 4);
    let rewards = win.reward_state.expect("win stages one RewardState");
    assert_ne!(rewards.inventory.state(), &before_inventory);
    assert_eq!(
        rewards.profiles.experience.get(&CHEF_PROFILE_ID),
        Some(&25),
        "(R) one local Chef XP award"
    );
    assert!(
        rewards
            .applied_grant_keys
            .contains(&format!("minigame-run:{id}"))
    );

    let spec = harvesting::board_spec(GAME_ID).unwrap();
    let run = lifecycle.activity_state().active.as_ref().unwrap();
    assert_eq!(
        run.wall_ms,
        (actions - 1) as u32 * INPUT_INTERVAL_MS,
        "the final chain completes before another modeled timer tick"
    );
    let mut expected_items = BTreeMap::new();
    for ((goal, output), &collected) in spec.goals.iter().zip(&spec.outputs).zip(&run.collected) {
        let bundles = collected / goal.per_reward * output.cluster_quantity;
        assert_eq!(bundles, 5, "one five-bundle completion per regional goal");
        let item_id = output.definition_id;
        assert_eq!(
            count_item(&rewards.inventory.state(), item_id),
            count_item(&before_inventory, item_id) + bundles
        );
        expected_items.insert(item_id, bundles);
    }
    let mut expected_events: Vec<_> = expected_items
        .into_iter()
        .map(|(item_id, quantity)| QuestEvent::CollectItem { item_id, quantity })
        .collect();
    expected_events.push(QuestEvent::FinishMinigame {
        minigame_id: GAME_ID as u32,
    });
    assert_eq!(win.quest_events, expected_events);

    let mut restored = HarvestingLifecycle::load(
        &fixture.path,
        CONTENT_FILE,
        GAME_ID,
        FIRST_INSTANCE_ID,
        PLAYER_GUID,
        Some(lifecycle.snapshot()),
    )
    .unwrap();
    let duplicate = restored
        .handle(&movie(id, "FRServer_GameOver"), None)
        .unwrap();
    assert!(!duplicate.changed);
    assert!(duplicate.replies.is_empty());
    assert!(duplicate.reward_packets.is_empty());
    assert!(duplicate.reward_state.is_none());
    assert!(duplicate.quest_events.is_empty());
    let duplicate_ack = restored.handle(&header(7, id), None).unwrap();
    assert!(!duplicate_ack.changed);
    assert!(duplicate_ack.replies.is_empty());
    let duplicate_win = restored
        .handle(&movie(id, "OnNextLevelMsg"), Some(&rewards))
        .unwrap();
    assert!(!duplicate_win.changed);
    assert!(duplicate_win.replies.is_empty());
    assert!(duplicate_win.reward_packets.is_empty());
    assert!(duplicate_win.reward_state.is_none());
    assert!(duplicate_win.quest_events.is_empty());
    let close = restored.handle(&[26, 0, 42, 0], None).unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    let retry = restored.begin(CHEF_PROFILE_ID).unwrap();
    assert_eq!(retry.state, Some(RunState::Offered));
    assert_eq!(
        i32::from_le_bytes(retry.replies[0][3..7].try_into().unwrap()),
        id + 1
    );
}

#[test]
fn loss_has_no_payout_and_close_allows_try_again() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let (id, _) = begin_play(&mut lifecycle);
    let before = fixture.rewards();
    let inventory = before.inventory.state().clone();
    let profiles = before.profiles.clone();

    let lost = lifecycle.tick(120_000).unwrap();
    assert_eq!(lost.state, Some(RunState::Lost));
    assert!(lost.reward_state.is_none());
    assert!(lost.reward_packets.is_empty());
    assert!(lost.quest_events.is_empty());
    assert_eq!(before.inventory.state(), &inventory);
    assert_eq!(before.profiles, profiles);

    let duplicate = lifecycle
        .handle(&movie(id, "FRServer_GameOver"), Some(&before))
        .unwrap();
    assert!(!duplicate.changed);
    assert!(duplicate.replies.is_empty());
    assert!(duplicate.reward_packets.is_empty());
    assert_eq!(lifecycle.run_state(), Some(RunState::Lost));
    let close = lifecycle.handle(&[26, 0, 42, 0], None).unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    let retry = lifecycle.begin(CHEF_PROFILE_ID).unwrap();
    assert_eq!(
        i32::from_le_bytes(retry.replies[0][3..7].try_into().unwrap()),
        id + 1
    );
}

#[test]
fn leave_and_disconnect_preserve_materials_and_create_no_grant() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let (id, _) = begin_play(&mut lifecycle);
    let before = fixture.rewards();
    let inventory = before.inventory.state().clone();
    let profiles = before.profiles.clone();

    let leave = lifecycle.handle(&header(6, id), Some(&before)).unwrap();
    assert_eq!(leave.state, Some(RunState::Cancelled));
    assert_eq!(leave.replies.len(), 1);
    assert_eq!(leave.replies[0][..3], [39, 0, 19]);
    assert!(leave.reward_state.is_none());
    assert!(leave.reward_packets.is_empty());
    assert!(leave.quest_events.is_empty());
    assert!(lifecycle.activity_state().active.is_none());
    assert_eq!(before.inventory.state(), &inventory);
    assert_eq!(before.profiles, profiles);
    assert!(
        !lifecycle
            .handle(&header(6, id), Some(&before))
            .unwrap()
            .changed
    );

    let retry = lifecycle.begin(CHEF_PROFILE_ID).unwrap();
    let retry_id = i32::from_le_bytes(retry.replies[0][3..7].try_into().unwrap());
    assert_eq!(retry_id, id + 1);
    let cancelled = lifecycle.disconnect();
    assert_eq!(cancelled.state, Some(RunState::Cancelled));
    assert!(cancelled.replies.is_empty());
    assert!(cancelled.reward_state.is_none());
    assert!(cancelled.reward_packets.is_empty());
    assert!(cancelled.quest_events.is_empty());
    assert_eq!(before.inventory.state(), &inventory);
    assert_eq!(before.profiles, profiles);
}

#[test]
fn reconstructed_quota_and_time_match_visible_candidate() {
    let spec = harvesting::board_spec(GAME_ID).unwrap();
    assert_eq!(spec.duration_seconds, 120);
    assert_eq!(spec.seed, RECONSTRUCTED_SEED);
    assert_eq!(
        RECONSTRUCTED_QUOTA_TILES / RECONSTRUCTED_PER_REWARD_TILES,
        5
    );
    assert!(spec.goals.iter().all(|goal| {
        goal.quota == RECONSTRUCTED_QUOTA_TILES && goal.per_reward == RECONSTRUCTED_PER_REWARD_TILES
    }));
}
