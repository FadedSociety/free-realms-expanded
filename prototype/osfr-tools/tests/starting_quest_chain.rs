use osfr_tools::{
    inventory::Inventory,
    profiles::ProfileState,
    quest_engine::{CharacterEligibility, QuestDenial, QuestEngine, QuestEvent, QuestStatus},
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

const CHARACTER: u64 = 42_017;
const MERIGOLD: u64 = 9_000_000_402;
const RAINA_RUSH: u64 = 9_000_000_177;
const CHAD_FASTFLUTTER: u64 = 9_500_000_735;
const MUSTANG_ALLI: u64 = 9_500_000_759;
const BURT_CREWE: u64 = 9_500_000_731;
const SAMANTHA: u64 = 9_000_000_189;
const RICKY_DANGER: u64 = 9_500_000_254;
const SHAKEY: u64 = 9_500_000_941;
const KART_TUTORIAL_SPEEDWAY: u32 = 31;

fn content_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: false,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn reward_state() -> RewardState {
    let directory = std::env::temp_dir().join(format!(
        "osfr-starting-chain-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let zero = 0i32.to_le_bytes();
    let content = json!({
        "schema_version": 1,
        "player_guid": CHARACTER,
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
    });
    fs::write(
        directory.join("inventory.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    let inventory = Inventory::load(&directory, None).unwrap();
    let _ = fs::remove_dir_all(&directory);
    RewardState::new(
        inventory,
        ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::new(),
            experience: BTreeMap::new(),
        },
        100_000,
    )
}

fn assert_refusal_changes_nothing(
    engine: &mut QuestEngine,
    rewards: &RewardState,
    quest_id: u32,
    giver_npc_id: u64,
    expected: QuestDenial,
) {
    let before = engine.state().clone();
    let coins = rewards.inventory.state().coins;
    let outcome = engine.offer(quest_id, giver_npc_id);
    assert_eq!(outcome.denial, Some(expected));
    assert!(!outcome.changed);
    assert!(outcome.outbound.is_empty());
    assert!(outcome.rewards.is_empty());
    assert_eq!(engine.state(), &before);
    assert_eq!(rewards.inventory.state().coins, coins);
}

fn finish_quest(
    engine: &mut QuestEngine,
    rewards: &RewardState,
    quest_id: u32,
    giver_npc_id: u64,
    events: &[QuestEvent],
    expected_reward: u32,
) -> RewardState {
    assert!(engine.offer(quest_id, giver_npc_id).changed);
    assert!(engine.accept(quest_id).changed);
    for (index, event) in events.iter().enumerate() {
        let outcome = engine.on_event(event.clone()).unwrap();
        assert!(outcome.changed, "quest {quest_id} event {index}");
        let counts = &engine.progress(quest_id).unwrap().counts;
        assert!(counts[..=index].iter().all(|count| *count == 1));
        assert!(counts[index + 1..].iter().all(|count| *count == 0));
    }
    assert_eq!(engine.state().pending_turnin, Some(quest_id));

    let coins_before = rewards.inventory.state().coins;
    let (outcome, settled) = engine.finish_turnin(quest_id, rewards).unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.rewards.len(), 1);
    assert_eq!(outcome.rewards[0].quest_id, quest_id);
    assert_eq!(outcome.rewards[0].coins, expected_reward);
    assert_eq!(
        settled.inventory.state().coins,
        coins_before + expected_reward
    );
    assert_eq!(
        engine.progress(quest_id).unwrap().status,
        QuestStatus::Completed
    );
    assert_eq!(engine.state().pending_turnin, None);

    // A repeated 49/14 cannot pay the chain twice.
    let state_before = engine.state().clone();
    let coins_before = settled.inventory.state().coins;
    let (duplicate, unchanged) = engine.finish_turnin(quest_id, &settled).unwrap();
    assert_eq!(duplicate.denial, Some(QuestDenial::NoPendingTurnin));
    assert!(!duplicate.changed);
    assert!(duplicate.outbound.is_empty());
    assert!(duplicate.rewards.is_empty());
    assert_eq!(engine.state(), &state_before);
    assert_eq!(unchanged.inventory.state().coins, coins_before);
    assert_eq!(unchanged.inventory.state(), settled.inventory.state());
    assert_eq!(unchanged.applied_grant_keys, settled.applied_grant_keys);
    settled
}

fn assert_chain_record_is_sourced(path: &Path, quest_id: u32, objective_count: usize) {
    let document: Value =
        serde_json::from_slice(&fs::read(path).unwrap()).expect("quest file is valid JSON");
    let quest = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["id"] == quest_id)
        .unwrap_or_else(|| panic!("quest {quest_id} exists"));
    assert_eq!(
        quest["enabled"].as_bool().unwrap_or(true),
        true,
        "quest {quest_id} is enabled"
    );
    let objectives = quest["objectives"].as_array().unwrap();
    assert_eq!(objectives.len(), objective_count);
    assert!(
        objectives.iter().all(|goal| {
            goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())
        }),
        "quest {quest_id} objectives have provenance"
    );
    let rewards = quest["rewards"].as_array().unwrap();
    assert!(!rewards.is_empty(), "quest {quest_id} has a reward");
    assert!(
        rewards.iter().all(|reward| {
            reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())
        }),
        "quest {quest_id} rewards have provenance"
    );
}

fn assert_npc_binds_quest(path: &Path, quest_id: u32, npc_id: u64) {
    let document: Value =
        serde_json::from_slice(&fs::read(path).unwrap()).expect("NPC file is valid JSON");
    let npc = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"] == npc_id)
        .unwrap_or_else(|| panic!("NPC {npc_id} exists"));
    assert!(
        npc["quest_ids_given"]
            .as_array()
            .unwrap()
            .contains(&json!(quest_id)),
        "NPC {npc_id} gives quest {quest_id}"
    );
    for relation in ["quests_given", "quests_accepted"] {
        assert!(
            npc[relation]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["quest_id"] == quest_id),
            "NPC {npc_id} has {relation} binding for quest {quest_id}"
        );
    }
}

#[test]
fn highroad_entry_chain_finishes_through_the_real_quest_engine() {
    let content = content_root();
    assert_chain_record_is_sourced(&content.join("quests/adventurer.json"), 3_000_020, 1);
    assert_chain_record_is_sourced(&content.join("quests/kart_driver.json"), 3_080_001, 4);
    assert_chain_record_is_sourced(&content.join("quests/kart_driver.json"), 3_080_002, 2);
    assert_npc_binds_quest(&content.join("npcs/snowhill.json"), 3_080_002, MUSTANG_ALLI);

    let mut engine =
        QuestEngine::from_split_content(&content, CHARACTER, eligibility(), None).unwrap();
    let mut rewards = reward_state();
    let starting_coins = rewards.inventory.state().coins;

    assert_refusal_changes_nothing(
        &mut engine,
        &rewards,
        3_080_001,
        RAINA_RUSH,
        QuestDenial::Prerequisite,
    );
    assert_refusal_changes_nothing(
        &mut engine,
        &rewards,
        3_080_002,
        MUSTANG_ALLI,
        QuestDenial::Prerequisite,
    );

    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_000_020,
        MERIGOLD,
        &[QuestEvent::TalkToNpc { npc_id: RAINA_RUSH }],
        5,
    );
    let loaded = engine.state().clone();
    engine =
        QuestEngine::from_split_content(&content, CHARACTER, eligibility(), Some(loaded)).unwrap();

    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_080_001,
        RAINA_RUSH,
        &[
            QuestEvent::TalkToNpc {
                npc_id: CHAD_FASTFLUTTER,
            },
            QuestEvent::TalkToNpc {
                npc_id: MUSTANG_ALLI,
            },
            QuestEvent::TalkToNpc { npc_id: BURT_CREWE },
            QuestEvent::TalkToNpc { npc_id: RAINA_RUSH },
        ],
        5,
    );
    let loaded = engine.state().clone();
    engine =
        QuestEngine::from_split_content(&content, CHARACTER, eligibility(), Some(loaded)).unwrap();

    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_080_002,
        MUSTANG_ALLI,
        &[
            QuestEvent::FinishMinigame {
                minigame_id: KART_TUTORIAL_SPEEDWAY,
            },
            QuestEvent::TalkToNpc {
                npc_id: MUSTANG_ALLI,
            },
        ],
        5,
    );

    assert_eq!(rewards.inventory.state().coins, starting_coins + 15);
    assert_eq!(
        engine.progress(3_000_020).unwrap().status,
        QuestStatus::Completed
    );
    assert_eq!(
        engine.progress(3_080_001).unwrap().status,
        QuestStatus::Completed
    );
    assert_eq!(
        engine.progress(3_080_002).unwrap().status,
        QuestStatus::Completed
    );
}

#[test]
fn cobblestone_post_opening_chain_finishes_and_is_not_rewarded_twice() {
    let content = content_root();
    assert_chain_record_is_sourced(&content.join("quests/adventurer.json"), 3_000_021, 1);
    assert_chain_record_is_sourced(&content.join("quests/adventurer.json"), 3_000_059, 2);
    assert_chain_record_is_sourced(&content.join("quests/adventurer.json"), 3_000_060, 1);

    let mut engine =
        QuestEngine::from_split_content(&content, CHARACTER, eligibility(), None).unwrap();
    let mut rewards = reward_state();
    let starting_coins = rewards.inventory.state().coins;
    assert_refusal_changes_nothing(
        &mut engine,
        &rewards,
        3_000_060,
        RICKY_DANGER,
        QuestDenial::Prerequisite,
    );

    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_000_021,
        SAMANTHA,
        &[QuestEvent::TalkToNpc {
            npc_id: RICKY_DANGER,
        }],
        5,
    );
    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_000_059,
        RICKY_DANGER,
        &[
            QuestEvent::TalkToNpc { npc_id: SHAKEY },
            QuestEvent::TalkToNpc {
                npc_id: RICKY_DANGER,
            },
        ],
        5,
    );
    rewards = finish_quest(
        &mut engine,
        &rewards,
        3_000_060,
        RICKY_DANGER,
        &[QuestEvent::TalkToNpc { npc_id: SAMANTHA }],
        5,
    );

    assert_eq!(rewards.inventory.state().coins, starting_coins + 15);
    for quest_id in [3_000_021, 3_000_059, 3_000_060] {
        assert_eq!(
            engine.progress(quest_id).unwrap().status,
            QuestStatus::Completed
        );
    }
}
