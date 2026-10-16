//! Mining's shared lifecycle, payout and quest-event acceptance.
//!
//! Fixtures are private and synthetic. The seeded whole-round case drives the
//! production Type8 engine, lifecycle manager and reward API without a client.

use osfr_tools::{
    inventory::{Inventory, InventoryState},
    minigame::RunState,
    mining::{
        CONTENT_FILE, FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, SIDE, TILE_WILDCARD,
        Type8Engine, Type8Phase, Type8Run, lifecycle::MiningActivityOutcome,
        lifecycle::MiningLifecycle, write_content,
    },
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

const GUID: u64 = 800;
const ORE_ITEM: i32 = 2739;
const LEFT: usize = 3;
const TOP: usize = 3;
const BOTTOM: usize = 10;
const RIGHT: usize = 10;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-mining-lifecycle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        write_content(&path, MINING_GAME_ID).unwrap();
        let empty = 0_i32.to_le_bytes();
        let definition = [ORE_ITEM.to_le_bytes(), 8_i32.to_le_bytes()].concat();
        let content = json!({
            "schema_version": 1,
            "player_guid": GUID,
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
            "definitions": [{
                "id": ORE_ITEM,
                "slot": 0,
                "class_id": 0,
                "default_tint": 0,
                "max_stack": -1,
                "model": "",
                "texture": "",
                "tint_alias": "dyetint",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&definition),
                "sha256": digest(&definition)
            }]
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { path }
    }

    fn engine(&self) -> Type8Engine {
        Type8Engine::load(
            &self.path,
            CONTENT_FILE,
            MINING_GAME_ID,
            FIRST_MINING_INSTANCE,
            None,
        )
        .unwrap()
    }

    fn lifecycle(&self) -> MiningLifecycle {
        MiningLifecycle::new(self.engine(), GUID).unwrap()
    }

    fn rewards(&self, ore_count: u32) -> RewardState {
        let mut inventory = Inventory::load(&self.path, None).unwrap();
        if ore_count > 0 {
            inventory.grant(0, ORE_ITEM, ore_count).unwrap();
        }
        RewardState::new(
            inventory,
            ProfileState {
                version: 1,
                active: MINER_PROFILE_ID,
                unlocked: BTreeSet::from([MINER_PROFILE_ID]),
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

fn acknowledge_idle(lifecycle: &mut MiningLifecycle, id: i32) {
    let Some(run) = lifecycle.activity_state().active.as_ref() else {
        return;
    };
    if run.phase != Type8Phase::Playing {
        return;
    }
    let mut idle = String::new();
    for y in TOP..=BOTTOM {
        for x in LEFT..=RIGHT {
            let index = y * SIDE + x;
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

fn begin_play(lifecycle: &mut MiningLifecycle) -> i32 {
    let offer = lifecycle.begin(MINER_PROFILE_ID).unwrap();
    assert_eq!(offer.state, Some(RunState::Offered));
    assert_eq!(offer.replies[0][..3], [39, 0, 16]);
    let id = FIRST_MINING_INSTANCE;
    let start = lifecycle.handle(&header(5, id), None).unwrap();
    assert_eq!(start.state, Some(RunState::Starting));
    assert_eq!(start.replies.len(), 2);
    lifecycle.handle(&movie(id, "OnConnectMsg"), None).unwrap();
    lifecycle
        .handle(&movie(id, "OnStartGameMsg"), None)
        .unwrap();
    lifecycle
        .handle(&movie(id, "OnStartLevelMsg"), None)
        .unwrap();
    assert_eq!(lifecycle.run_state(), Some(RunState::Playing));
    id
}

fn find_chain(run: &Type8Run) -> Option<Vec<(usize, usize)>> {
    const MAX_PATH_LEN: usize = 12;
    const NODE_BUDGET: usize = 50_000;

    type Rank = (bool, usize, usize, i32, usize, usize);

    fn rank(run: &Type8Run, path: &[(usize, usize)], ores: &[(usize, usize)]) -> Rank {
        let mut collect_now = false;
        let mut helped = 0;
        let mut cleared_total = 0;
        let mut remaining_min = None;
        for &(ore_row, ore_col) in ores {
            let cleared = path
                .iter()
                .filter(|&&(row, col)| {
                    col == ore_col && row > ore_row && run.board[row * SIDE + col] != 0
                })
                .count();
            if cleared == 0 {
                continue;
            }
            helped += 1;
            cleared_total += cleared;
            let total = ((ore_row + 1)..=BOTTOM)
                .filter(|&row| run.board[row * SIDE + ore_col] != 0)
                .count();
            let remaining = total.saturating_sub(cleared);
            remaining_min = Some(remaining_min.map_or(remaining, |old: usize| old.min(remaining)));
            collect_now |= remaining == 0;
        }
        (
            collect_now,
            helped,
            cleared_total,
            -(remaining_min.unwrap_or(0) as i32),
            path.len(),
            path.iter().map(|(row, _)| *row).max().unwrap_or(0),
        )
    }

    fn explore(
        run: &Type8Run,
        ores: &[(usize, usize)],
        path: &mut Vec<(usize, usize)>,
        seen: &mut BTreeSet<(usize, usize)>,
        reference: Option<u16>,
        budget: &mut i32,
        best: &mut Option<Vec<(usize, usize)>>,
        best_rank: &mut Option<Rank>,
    ) {
        *budget -= 1;
        if *budget < 0 {
            return;
        }
        if path.len() >= 3 && reference.is_some() {
            let candidate = rank(run, path, ores);
            if best_rank.is_none_or(|current| candidate > current) {
                *best_rank = Some(candidate);
                *best = Some(path.clone());
            }
        }
        if path.len() >= MAX_PATH_LEN {
            return;
        }
        let &(row, col) = path.last().unwrap();
        for (next_row, next_col) in [
            (row + 1, col),
            (row, col + 1),
            (row, col.wrapping_sub(1)),
            (row.wrapping_sub(1), col),
        ] {
            if !(TOP..=BOTTOM).contains(&next_row)
                || !(LEFT..=RIGHT).contains(&next_col)
                || seen.contains(&(next_row, next_col))
            {
                continue;
            }
            let index = next_row * SIDE + next_col;
            let cell = run.board[index];
            if !run.idle[index]
                || (!(401..=405).contains(&cell) && cell != TILE_WILDCARD)
                || (cell != TILE_WILDCARD && reference.is_some_and(|value| value != cell))
            {
                continue;
            }
            seen.insert((next_row, next_col));
            path.push((next_row, next_col));
            explore(
                run,
                ores,
                path,
                seen,
                if cell == TILE_WILDCARD {
                    reference
                } else {
                    Some(cell)
                },
                budget,
                best,
                best_rank,
            );
            path.pop();
            seen.remove(&(next_row, next_col));
        }
    }

    let ores: Vec<_> = run
        .board
        .iter()
        .enumerate()
        .filter_map(|(index, &value)| {
            let goal = match value {
                601 if run.collected.first().copied().unwrap_or(0) < 6 => Some(601),
                602 if run.collected.get(1).copied().unwrap_or(0) < 6 => Some(602),
                _ => None,
            }?;
            let row = index / SIDE;
            let col = index % SIDE;
            if (TOP..=BOTTOM).contains(&row) && (LEFT..=RIGHT).contains(&col) {
                Some((row, col, goal))
            } else {
                None
            }
        })
        .map(|(row, col, _)| (row, col))
        .collect();
    let mut budget = NODE_BUDGET as i32;
    let mut best = None;
    let mut best_rank = None;
    for row in TOP..=BOTTOM {
        for col in LEFT..=RIGHT {
            if budget < 0 {
                break;
            }
            let index = row * SIDE + col;
            let cell = run.board[index];
            if !run.idle[index] || (!(401..=405).contains(&cell) && cell != TILE_WILDCARD) {
                continue;
            }
            let mut path = vec![(row, col)];
            let mut seen = BTreeSet::from([(row, col)]);
            explore(
                run,
                &ores,
                &mut path,
                &mut seen,
                (cell != TILE_WILDCARD).then_some(cell),
                &mut budget,
                &mut best,
                &mut best_rank,
            );
        }
    }
    best.map(|path| path.into_iter().map(|(row, col)| (col, row)).collect())
}

fn drive_seeded_round(
    lifecycle: &mut MiningLifecycle,
    reward_state: &RewardState,
) -> (u32, MiningActivityOutcome) {
    let id = begin_play(lifecycle);
    let mut actions = 0;
    loop {
        let run = lifecycle.activity_state().active.as_ref().unwrap();
        if run.phase == Type8Phase::LevelComplete {
            break;
        }
        assert_eq!(run.phase, Type8Phase::Playing);
        let path = find_chain(run).unwrap_or_else(|| {
            panic!(
                "no planner path at action {actions}; collected={:?} reshuffles={} idle={} tiles={:?}",
                run.collected,
                run.reshuffles,
                run.idle.iter().filter(|&&idle| idle).count(),
                &run.board[TOP * SIDE + LEFT..=BOTTOM * SIDE + RIGHT]
            )
        });
        let mut coordinates = String::new();
        for (x, y) in path {
            coordinates.push_str(&format!("{x},{y},"));
        }
        lifecycle
            .handle(&movie(id, &format!("OnChainMsg\t{coordinates}")), None)
            .unwrap();
        acknowledge_idle(lifecycle, id);

        for x in (LEFT..=RIGHT).rev() {
            let Some(run) = lifecycle.activity_state().active.as_ref() else {
                break;
            };
            if run.phase != Type8Phase::Playing {
                break;
            }
            let value = run.board[BOTTOM * SIDE + x];
            if [601, 602].contains(&value) {
                lifecycle
                    .handle(
                        &movie(id, &format!("OnLevelEventMsg\t50\t{x}\t{BOTTOM}")),
                        None,
                    )
                    .unwrap();
            }
        }
        acknowledge_idle(lifecycle, id);
        actions += 1;
        if lifecycle
            .activity_state()
            .active
            .as_ref()
            .is_some_and(|run| run.phase == Type8Phase::Playing)
        {
            lifecycle.tick(4_000).unwrap();
        }
        assert!(actions <= 200, "seeded Mining round exceeded 200 chains");
    }
    let run = lifecycle.activity_state().active.as_ref().unwrap();
    assert_eq!(run.collected, [6, 6]);
    assert!(
        run.remaining_ms > 0,
        "round completed before its game timer"
    );
    assert!(
        run.wall_ms <= 600_000,
        "round exceeded the 10-minute model cap"
    );
    let win = lifecycle
        .handle(&movie(id, "OnNextLevelMsg"), Some(reward_state))
        .unwrap();
    assert_eq!(win.state, Some(RunState::Won));
    (actions, win)
}

#[test]
fn seeded_full_round_wins_and_grants_ore_xp_and_quest_events_once() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let before = fixture.rewards(0);
    let (actions, win) = drive_seeded_round(&mut lifecycle, &before);
    assert_eq!(
        actions, 96,
        "the seeded 40% / 6+6 replay stays deterministic"
    );
    assert_eq!(win.replies.len(), 4); // OnGameWon, 39/18, 39/47, 39/19.
    let rewards = win.reward_state.expect("win stages RewardState");
    assert_eq!(count_item(&rewards.inventory.state(), ORE_ITEM), 12);
    assert_eq!(
        rewards.profiles.experience.get(&MINER_PROFILE_ID),
        Some(&25)
    );
    assert!(rewards.applied_grant_keys.contains("minigame-run:18000001"));
    assert_eq!(
        win.quest_events,
        vec![
            QuestEvent::CollectItem {
                item_id: ORE_ITEM,
                quantity: 12,
            },
            QuestEvent::FinishMinigame { minigame_id: 18 },
        ]
    );

    let mut restored = MiningLifecycle::load(
        &fixture.path,
        CONTENT_FILE,
        MINING_GAME_ID,
        FIRST_MINING_INSTANCE,
        GUID,
        Some(lifecycle.snapshot()),
    )
    .unwrap();
    let won = restored
        .handle(&movie(FIRST_MINING_INSTANCE, "FRServer_GameOver"), None)
        .unwrap();
    assert!(!won.changed);
    assert!(won.replies.is_empty());
    assert!(won.reward_packets.is_empty());
    assert!(won.quest_events.is_empty());
    let duplicate_next = restored
        .handle(
            &movie(FIRST_MINING_INSTANCE, "OnNextLevelMsg"),
            Some(&rewards),
        )
        .unwrap();
    assert!(!duplicate_next.changed);
    assert!(duplicate_next.replies.is_empty());
    assert!(duplicate_next.reward_state.is_none());
    let close = restored.handle(&[26, 0, 42, 0], None).unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    assert!(close.replies.is_empty());
    let retry = restored.begin(MINER_PROFILE_ID).unwrap();
    assert_eq!(retry.state, Some(RunState::Offered));
    assert_eq!(
        i32::from_le_bytes(retry.replies[0][3..7].try_into().unwrap()),
        FIRST_MINING_INSTANCE + 1
    );
}

#[test]
fn loss_has_no_payout_and_close_allows_try_again() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let id = begin_play(&mut lifecycle);
    let before = fixture.rewards(0);
    let saved_inventory = before.inventory.state().clone();
    let saved_profiles = before.profiles.clone();
    let lost = lifecycle.tick(180_000).unwrap();
    assert_eq!(lost.state, Some(RunState::Lost));
    assert!(lost.reward_state.is_none());
    assert!(lost.reward_packets.is_empty());
    assert!(lost.quest_events.is_empty());
    assert_eq!(before.inventory.state(), &saved_inventory);
    assert_eq!(before.profiles, saved_profiles);
    assert_eq!(lifecycle.run_state(), Some(RunState::Lost));

    let duplicate = lifecycle
        .handle(&movie(id, "FRServer_GameOver"), Some(&before))
        .unwrap();
    assert!(!duplicate.changed);
    assert!(duplicate.replies.is_empty());
    assert!(duplicate.reward_packets.is_empty());
    let close = lifecycle.handle(&[26, 0, 42, 0], None).unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    let retry = lifecycle.begin(MINER_PROFILE_ID).unwrap();
    assert_eq!(retry.state, Some(RunState::Offered));
    assert_eq!(
        i32::from_le_bytes(retry.replies[0][3..7].try_into().unwrap()),
        id + 1
    );
}

#[test]
fn quit_and_disconnect_preserve_materials_and_create_no_grant() {
    let fixture = Fixture::new();
    let mut lifecycle = fixture.lifecycle();
    let id = begin_play(&mut lifecycle);
    let before = fixture.rewards(4);
    let inventory = before.inventory.state().clone();
    let profiles = before.profiles.clone();

    let quit = lifecycle.handle(&header(6, id), Some(&before)).unwrap();
    assert_eq!(quit.state, Some(RunState::Cancelled));
    assert_eq!(quit.replies.len(), 1);
    assert_eq!(quit.replies[0][..3], [39, 0, 19]);
    assert!(quit.reward_state.is_none());
    assert!(quit.reward_packets.is_empty());
    assert!(quit.quest_events.is_empty());
    assert!(lifecycle.activity_state().active.is_none());
    assert_eq!(before.inventory.state(), &inventory);
    assert_eq!(before.profiles, profiles);
    assert!(
        !lifecycle
            .handle(&header(6, id), Some(&before))
            .unwrap()
            .changed
    );

    let retry_id = lifecycle.begin(MINER_PROFILE_ID).unwrap();
    assert_eq!(
        i32::from_le_bytes(retry_id.replies[0][3..7].try_into().unwrap()),
        id + 1
    );
    let cancelled = lifecycle.disconnect();
    assert_eq!(cancelled.state, Some(RunState::Cancelled));
    assert!(cancelled.replies.is_empty());
    assert!(cancelled.reward_state.is_none());
    assert_eq!(before.inventory.state(), &inventory);
    assert_eq!(before.profiles, profiles);
}

fn count_item(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}
