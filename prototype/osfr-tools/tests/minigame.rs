use osfr_tools::{inventory, minigame, profiles, rewards};

use inventory::Inventory;
use minigame::{Activity, ActivityFinish, MinigameRuns, RunMode, RunSpec, RunState};
use profiles::ProfileState;
use rewards::{RewardBundleRow, RewardDisplay, RewardGrant, RewardState};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new(coins: u32) -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-minigame-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let coin_bytes = (coins as i32).to_le_bytes();
        let empty_count = 0_i32.to_le_bytes();
        let definition_bytes = [100_i32.to_le_bytes(), 8_i32.to_le_bytes()].concat();
        let content = json!({
            "schema_version": 1,
            "player_guid": 800,
            "default_state": {
                "coins": coins,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coin_bytes),
                span("items", 17, 0, &empty_count),
                span("equipment", 21, 1, &empty_count)
            ],
            "definitions": [{
                "id": 100,
                "slot": 3,
                "class_id": 1,
                "default_tint": 5,
                "max_stack": 10,
                "model": "synthetic.adr",
                "texture": "synthetic",
                "tint_alias": "synthetic",
                "effect": 0,
                "wield_type": 0,
                "serialized_hex": hex(&definition_bytes),
                "sha256": digest(&definition_bytes)
            }]
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { path }
    }

    fn reward_state(&self) -> RewardState {
        RewardState::new(
            Inventory::load(&self.path, None).unwrap(),
            ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::from([1]),
                experience: BTreeMap::from([(1, 20)]),
            },
            4,
        )
    }

    fn reward_state_with_materials(&self) -> RewardState {
        let mut inventory = Inventory::load(&self.path, None).unwrap();
        inventory.grant(0, 100, 3).unwrap();
        RewardState::new(
            inventory,
            ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::from([1]),
                experience: BTreeMap::from([(1, 20)]),
            },
            4,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("inventory.json"));
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

fn spec() -> RunSpec {
    let mut spec = RunSpec::empty(
        46,
        9,
        RunMode::Flash {
            movie: "game_cooking.gfx".into(),
        },
    );
    spec.name_id = 11;
    spec.icon_id = 12;
    spec.description_id = 13;
    spec.profile_type = 1;
    spec.preload_file = "CookingAssets.txt".into();
    spec
}

#[derive(Clone, Copy)]
struct TestActivity {
    score: u32,
    finish: Option<ActivityFinish>,
}

impl Activity for TestActivity {
    fn score(&self) -> u32 {
        self.score
    }

    fn finish(&self) -> Option<ActivityFinish> {
        self.finish
    }
}

fn request(sub: u8, instance_id: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, sub];
    for id in [instance_id, -1, -1] {
        bytes.extend(id.to_le_bytes());
    }
    bytes
}

fn expected_info(instance_id: i32, spec: &RunSpec) -> Vec<u8> {
    let mut bytes = vec![39, 0, 16];
    for id in [instance_id, -1, spec.game_id] {
        bytes.extend(id.to_le_bytes());
    }
    for value in [
        spec.name_id,
        spec.icon_id,
        spec.description_id,
        spec.difficulty,
        spec.profile_type,
        spec.type_id,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(0);
    bytes.extend(&spec.reward_bundles);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(spec.flags);
    bytes.extend((spec.preload_file.len() as i32).to_le_bytes());
    bytes.extend(spec.preload_file.as_bytes());
    bytes.extend(1_i32.to_le_bytes());
    bytes.push(1);
    bytes.extend(spec.game_id.to_le_bytes());
    bytes.extend([0; 4]);
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes
}

fn expected_launch(instance_id: i32, game_id: i32) -> Vec<Vec<u8>> {
    let mut started = vec![39, 0, 17];
    for id in [instance_id, -1, game_id] {
        started.extend(id.to_le_bytes());
    }
    let mut flash = vec![26, 0, 12, 0];
    for value in ["MiniGameFlash", "game_cooking.gfx"] {
        flash.extend((value.len() as i32).to_le_bytes());
        flash.extend(value.as_bytes());
    }
    flash.push(0);
    vec![started, flash]
}

fn expected_finish(instance_id: i32, game_id: i32, won: bool) -> Vec<Vec<u8>> {
    let mut movie = vec![39, 0, 14];
    for id in [instance_id, -1, game_id] {
        movie.extend(id.to_le_bytes());
    }
    let event = if won { "OnGameWonMsg" } else { "OnGameLostMsg" };
    movie.extend(((event.len() + 1) as i32).to_le_bytes());
    movie.extend(event.as_bytes());
    movie.push(0);

    let mut game_over = vec![39, 0, 18];
    for id in [instance_id, -1, game_id] {
        game_over.extend(id.to_le_bytes());
    }
    game_over.push(u8::from(won));
    for value in [3_i32, 12, 900] {
        game_over.extend(value.to_le_bytes());
    }

    let mut scores = vec![39, 0, 47];
    for id in [instance_id, -1, game_id] {
        scores.extend(id.to_le_bytes());
    }
    scores.extend(1_i32.to_le_bytes());
    let label = b"scoreTotalScore";
    scores.extend((label.len() as i32).to_le_bytes());
    scores.extend(label);
    for value in [-1_i32, 4, -1, -1, 900] {
        scores.extend(value.to_le_bytes());
    }

    let mut remove = vec![39, 0, 19];
    for id in [instance_id, -1, game_id] {
        remove.extend(id.to_le_bytes());
    }
    vec![movie, game_over, scores, remove]
}

fn grant() -> RewardGrant {
    RewardGrant {
        coins: 5,
        items: Vec::new(),
        job_xp: BTreeMap::new(),
        sc: 0,
        display: RewardDisplay {
            source_guid: 0,
            player_guid: 800,
            reward_kind: 2,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

#[test]
fn flash_start_play_win_close_has_exact_packet_bytes_and_once_only_reward() {
    let fixture = Fixture::new(10);
    let initial_reward = fixture.reward_state();
    let payout = grant();
    let mut runs = MinigameRuns::new(123).unwrap();
    let offer_spec = spec();
    let offered = runs.offer(offer_spec.clone()).unwrap();
    assert_eq!(offered.replies, vec![expected_info(123, &offer_spec)]);
    assert_eq!(offered.state, Some(RunState::Offered));

    let started = runs.handle_client(&request(5, 123)).unwrap();
    assert_eq!(started.replies, expected_launch(123, 46));
    assert_eq!(started.state, Some(RunState::Starting));
    assert_eq!(runs.started(123).state, Some(RunState::Playing));

    let activity = TestActivity {
        score: 900,
        finish: Some(ActivityFinish {
            won: true,
            stars: 3,
            elapsed_seconds: 12,
        }),
    };
    assert!(runs.score(123, &activity).changed);
    let won = runs
        .finish(123, &activity, Some((&initial_reward, &payout)))
        .unwrap();
    assert_eq!(won.replies, expected_finish(123, 46, true));
    assert_eq!(won.state, Some(RunState::Won));
    assert_eq!(won.reward_packets[0], [38, 0, 19, 0, 15, 0, 0, 0]);
    assert_eq!(&won.reward_packets[1][..4], &[50, 0, 1, 1]);
    let committed_reward = won.reward_state.as_ref().unwrap();
    assert_eq!(committed_reward.inventory.state().coins, 15);
    assert!(
        committed_reward
            .applied_grant_keys
            .contains("minigame-run:123")
    );

    let duplicate_win = runs
        .finish(123, &activity, Some((&initial_reward, &payout)))
        .unwrap();
    assert!(!duplicate_win.accepted);
    assert!(duplicate_win.replies.is_empty());
    assert!(duplicate_win.reward_packets.is_empty());
    assert!(duplicate_win.reward_state.is_none());

    let ack = runs.handle_client(&request(7, 123)).unwrap();
    assert!(ack.accepted);
    assert!(!ack.changed);
    assert!(ack.replies.is_empty());
    assert_eq!(runs.state(123), Some(RunState::Won));

    let close = runs.handle_client(&[26, 0, 42, 0]).unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    assert!(close.replies.is_empty());
    let duplicate_close = runs.handle_client(&[26, 0, 42, 0]).unwrap();
    assert!(!duplicate_close.accepted);
    assert!(!duplicate_close.changed);
    assert_eq!(runs.state(123), Some(RunState::Closed));
}

#[test]
fn loss_sends_results_without_settling_reward() {
    let fixture = Fixture::new(10);
    let initial_reward = fixture.reward_state();
    let mut runs = MinigameRuns::new(124).unwrap();
    runs.offer(spec()).unwrap();
    runs.handle_client(&request(5, 124)).unwrap();
    runs.started(124);
    let activity = TestActivity {
        score: 900,
        finish: Some(ActivityFinish {
            won: false,
            stars: 3,
            elapsed_seconds: 12,
        }),
    };
    let lost = runs.finish(124, &activity, None).unwrap();
    assert_eq!(lost.replies, expected_finish(124, 46, false));
    assert_eq!(lost.state, Some(RunState::Lost));
    assert!(lost.reward_packets.is_empty());
    assert!(lost.reward_state.is_none());
    assert_eq!(initial_reward.inventory.state().coins, 10);
}

#[test]
fn a_win_may_settle_without_a_grant_and_a_loss_still_may_not() {
    let fixture = Fixture::new(10);
    let initial_reward = fixture.reward_state();
    let mut runs = MinigameRuns::new(131).unwrap();
    runs.offer(spec()).unwrap();
    runs.handle_client(&request(5, 131)).unwrap();
    runs.started(131);
    let won_activity = TestActivity {
        score: 900,
        finish: Some(ActivityFinish {
            won: true,
            stars: 3,
            elapsed_seconds: 12,
        }),
    };
    // A win through `finish` still needs its grant: a caller that forgets it is
    // refused, not settled without a reward.
    assert!(
        runs.finish(131, &won_activity, None).is_err(),
        "a won minigame must carry its grant"
    );
    assert_eq!(runs.state(131), Some(RunState::Playing));
    // A capped or zero-award win says so explicitly.
    let won = runs.finish_win_without_reward(131, &won_activity).unwrap();
    assert_eq!(won.replies, expected_finish(131, 46, true));
    assert_eq!(won.state, Some(RunState::Won));
    assert!(won.reward_packets.is_empty());
    assert!(won.reward_state.is_none());
    assert_eq!(initial_reward.inventory.state().coins, 10);

    let mut lost = MinigameRuns::new(132).unwrap();
    lost.offer(spec()).unwrap();
    lost.handle_client(&request(5, 132)).unwrap();
    lost.started(132);
    let loss = TestActivity {
        score: 0,
        finish: Some(ActivityFinish {
            won: false,
            stars: 0,
            elapsed_seconds: 1,
        }),
    };
    let payout = grant();
    assert!(
        lost.finish(132, &loss, Some((&initial_reward, &payout)))
            .is_err(),
        "a loss must never carry a grant"
    );
    assert_eq!(lost.state(132), Some(RunState::Playing));
}

#[test]
fn leave_and_disconnect_cancel_without_reward_or_activity_mutation() {
    let fixture = Fixture::new(10);
    let reward_state = fixture.reward_state_with_materials();
    let mut leave_runs = MinigameRuns::new(125).unwrap();
    leave_runs.offer(spec()).unwrap();
    leave_runs.handle_client(&request(5, 125)).unwrap();
    leave_runs.started(125);
    let left = leave_runs.handle_client(&request(6, 125)).unwrap();
    let mut expected_remove = vec![39, 0, 19];
    for id in [125_i32, -1, 46] {
        expected_remove.extend(id.to_le_bytes());
    }
    assert_eq!(left.replies, vec![expected_remove]);
    assert_eq!(left.state, Some(RunState::Cancelled));
    assert!(left.reward_packets.is_empty());
    assert_eq!(reward_state.inventory.state().coins, 10);
    assert_eq!(reward_state.inventory.state().items[0].count, 3);
    assert!(left.reward_state.is_none());

    let mut disconnect_runs = MinigameRuns::new(126).unwrap();
    disconnect_runs.offer(spec()).unwrap();
    disconnect_runs.handle_client(&request(5, 126)).unwrap();
    disconnect_runs.started(126);
    let disconnected = disconnect_runs.disconnect();
    assert_eq!(disconnected.state, Some(RunState::Cancelled));
    assert!(disconnected.replies.is_empty());
    assert!(disconnected.reward_packets.is_empty());
    assert_eq!(reward_state.inventory.state().coins, 10);
    assert_eq!(reward_state.inventory.state().items[0].count, 3);
    assert!(disconnected.reward_state.is_none());
}

#[test]
fn encounter_start_and_close_use_encounter_family_packets() {
    let mut runs = MinigameRuns::new(127).unwrap();
    let mut encounter = RunSpec::empty(
        4,
        4,
        RunMode::Encounter {
            encounter_id: 77,
            player_guid: 0x0102_0304_0506_0708,
        },
    );
    encounter.flags = [1; 5];
    let offered = runs.offer(encounter).unwrap();
    // 39/16 then the 41/107 ready (`00a9f990`: the bare encounter header) that turns the
    // start screen's "Loading..." into Go.
    assert_eq!(offered.replies.len(), 2);
    assert_eq!(offered.replies[0][..3], [39, 0, 16]);
    assert_eq!(
        offered.replies[1],
        vec![41, 0, 107, 0, 77, 0, 0, 0, 127, 0, 0, 0]
    );
    let started = runs.handle_client(&request(5, 127)).unwrap();
    // 41/2 as before, then the 39/17 that starts a 39/16-created state (`009b9550`).
    assert_eq!(
        started.replies,
        vec![
            vec![
                41, 0, 2, 0, 77, 0, 0, 0, 127, 0, 0, 0, 8, 7, 6, 5, 4, 3, 2, 1, 0
            ],
            vec![39, 0, 17, 127, 0, 0, 0, 255, 255, 255, 255, 4, 0, 0, 0],
        ]
    );
    runs.started(127);
    let loss = TestActivity {
        score: 0,
        finish: Some(ActivityFinish {
            won: false,
            stars: 0,
            elapsed_seconds: 1,
        }),
    };
    runs.finish(127, &loss, None).unwrap();
    let close = runs
        .handle_client(&[41, 0, 109, 0, 0, 0, 0, 0, 127, 0, 0, 0])
        .unwrap();
    assert_eq!(close.state, Some(RunState::Closed));
    assert!(close.replies.is_empty());
}

#[test]
fn prize_wheel_landing_is_server_selected_and_stop_report_cannot_pay() {
    let fixture = Fixture::new(10);
    let state = fixture.reward_state();
    let payout = grant();
    let mut runs = MinigameRuns::new(130).unwrap();
    runs.offer(spec()).unwrap();
    runs.handle_client(&request(5, 130)).unwrap();
    runs.started(130);
    let selected = runs
        .choose_wheel_prize(
            130,
            RewardBundleRow {
                type_id: 3,
                body: vec![0xaa, 0xbb, 0xcc],
            },
        )
        .unwrap();
    let mut expected = vec![39, 0, 45];
    for id in [130_i32, -1, 46] {
        expected.extend(id.to_le_bytes());
    }
    expected.extend(3_u32.to_le_bytes());
    expected.extend([0xaa, 0xbb, 0xcc]);
    assert_eq!(selected.replies, vec![expected]);

    let activity = TestActivity {
        score: 500,
        finish: Some(ActivityFinish {
            won: true,
            stars: 1,
            elapsed_seconds: 5,
        }),
    };
    runs.finish(130, &activity, Some((&state, &payout)))
        .unwrap();
    let stopped = runs.handle_client(&request(46, 130)).unwrap();
    assert!(stopped.accepted);
    assert!(stopped.reward_packets.is_empty());
    assert!(stopped.reward_state.is_none());
    let duplicate = runs.handle_client(&request(46, 130)).unwrap();
    assert!(!duplicate.accepted);
    assert!(!duplicate.changed);
    assert!(duplicate.reward_packets.is_empty());
}

/// T795: flag `+616` (end-of-game dialog on 39/19, `009bf190`) stays on for every shared run;
/// only a card run, whose result belongs to the card window, may leave it off. A card run's
/// start waits for its ticket request (39/37), which the offered run already answers.
#[test]
fn only_a_card_run_may_skip_the_end_dialog_and_it_starts_after_its_ticket() {
    let mut dialogless = spec();
    dialogless.flags[4] = 0;
    assert!(MinigameRuns::new(1).unwrap().offer(dialogless).is_err());

    let mut card = RunSpec::empty(500, 16, RunMode::CardClient);
    card.flags[4] = 0;
    let mut runs = MinigameRuns::new(61_000_001).unwrap();
    assert!(runs.offer(card).unwrap().accepted);
    let before = runs.snapshot().unwrap();
    let early = runs.handle_client(&request(5, 61_000_001)).unwrap();
    assert!(!early.accepted && early.replies.is_empty());
    assert_eq!(
        runs.snapshot().unwrap(),
        before,
        "a refused start changes nothing"
    );

    let asked = runs.handle_client(&request(37, 61_000_001)).unwrap();
    assert_eq!(asked.state, Some(RunState::Offered));
    assert_eq!(asked.card_ticket.map(|t| t.game_id), Some(500));
    let started = runs.handle_client(&request(5, 61_000_001)).unwrap();
    assert_eq!(started.state, Some(RunState::Starting));
    assert_eq!(started.replies.len(), 1, "39/17 alone");

    // Another mode never answers 39/37.
    let mut shared = MinigameRuns::new(1).unwrap();
    shared.offer(spec()).unwrap();
    assert!(
        shared
            .handle_client(&request(37, 1))
            .unwrap()
            .card_ticket
            .is_none()
    );
}

#[test]
fn persisted_counter_never_reuses_ids_and_wrong_instance_is_rejected() {
    let mut runs = MinigameRuns::new(128).unwrap();
    let first = runs.offer(spec()).unwrap();
    assert_eq!(first.state, Some(RunState::Offered));
    let wrong_start = runs.handle_client(&request(5, 129)).unwrap();
    assert!(!wrong_start.accepted);
    assert!(wrong_start.replies.is_empty());
    assert_eq!(runs.state(128), Some(RunState::Offered));

    let wrong_activity = TestActivity {
        score: 1,
        finish: Some(ActivityFinish {
            won: false,
            stars: 0,
            elapsed_seconds: 1,
        }),
    };
    let wrong_finish = runs.finish(129, &wrong_activity, None).unwrap();
    assert!(!wrong_finish.accepted);
    assert_eq!(runs.state(128), Some(RunState::Offered));

    let cancelled = runs.handle_client(&[26, 0, 42, 0]).unwrap();
    assert_eq!(cancelled.state, Some(RunState::Cancelled));
    let restarted = MinigameRuns::restore(&runs.snapshot().unwrap()).unwrap();
    assert_eq!(restarted.next_instance_id(), 129);
    let mut restarted = restarted;
    let second = restarted.offer(spec()).unwrap();
    assert_eq!(restarted.current_instance_id(), Some(129));
    assert_ne!(restarted.current_instance_id(), Some(128));
    assert!(!second.replies.is_empty());
}
