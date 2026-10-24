use osfr_tools::{
    account_store::AccountRole,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    inventory::Inventory,
    profiles::ProfileState,
    quest_engine::{
        CharacterEligibility, QuestDenial, QuestEngine, QuestEvent, QuestState, QuestStatus,
    },
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
#[path = "support/sc_earn.rs"]
mod sc_earn;

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
const MUSTANG_POSITION: [f32; 3] = [-237.1163, 1.1, -46.70612];
const TALK_MENU_STRING_ID: u32 = 3300;
const RACE_MENU_STRING_ID: u32 = 3052;
const TUTORIAL_INTRO_STRING_ID: u32 = 117327;
const TUTORIAL_DESCRIPTION_STRING_ID: u32 = 117329;
const TUTORIAL_END_STRING_ID: u32 = 117330;
const QUEST_CHOICE_RESPONSE: u32 = 301;
const ROUTE_NOW: u64 = 10_000;

fn content_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        // T202: the Kart Driver entry quests are offered only to their job.
        job_levels: BTreeMap::from([("kart_driver".into(), 20)]),
        member: false,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

/// T202: the playable Kart Driver line is offered only while Kart Driver is active and
/// unlocked, and the turn-in pays profile 48. A saved Kart Driver is that production state.
fn kart_driver_profiles(state: &mut GameplayState) {
    state.profiles = Some(ProfileState {
        version: 1,
        active: 48,
        unlocked: BTreeSet::from([1, 43, 48]),
        experience: BTreeMap::new(),
    });
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
            // T202: the chapter rows pay Adventurer (1) and Kart Driver (48) job XP.
            unlocked: BTreeSet::from([1, 48]),
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
    assert!(
        npc["quests_given"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["quest_id"] == quest_id),
        "NPC {npc_id} has a quests_given binding for quest {quest_id}"
    );
}

struct RouteContent(PathBuf);

impl RouteContent {
    fn new() -> Self {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = std::env::var_os("OSFR_ROUTE_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                manifest.join("../../../lab/reports/round59-native-20261015T124443Z/content")
            });
        assert!(
            source.join("opening.json").is_file() && source.join("kart_entry.json").is_file(),
            "packet route tests require the prepared content fixture: {}",
            source.display()
        );
        let root = std::env::temp_dir().join(format!(
            "osfr-racing-route-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                fs::copy(&path, root.join(path.file_name().unwrap())).unwrap();
            }
        }
        fs::copy(
            content_root().join("kart_gates.json"),
            root.join("kart_gates.json"),
        )
        .unwrap();
        Self(root)
    }
}

impl Drop for RouteContent {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn reference_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gateway-reference")
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u32).to_le_bytes());
    bytes.extend(value.as_bytes());
}

fn application(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn login(guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    put_string(&mut bytes, "F128-route-test");
    bytes.extend(guid.to_le_bytes());
    put_string(&mut bytes, "1.910.1.530630");
    put_string(&mut bytes, "");
    bytes
}

fn npc_interact(guid: u64) -> Vec<u8> {
    // Client 26/8 constructor 0x008b6bc0; sender 0x0092efe0.
    let mut bytes = vec![26, 0, 8, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.push(0);
    bytes
}

fn npc_select(guid: u64, option: i32) -> Vec<u8> {
    // Client 26/10 constructor 0x00a36f80; sender 0x00a39090.
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(option.to_le_bytes());
    bytes
}

fn quest_choice() -> Vec<u8> {
    // Client 26/6 carries the selected dialogue row (reader 0x009f5890).
    let mut bytes = vec![26, 0, 6, 0];
    bytes.extend(QUEST_CHOICE_RESPONSE.to_le_bytes());
    bytes
}

fn quest_reply(quest_id: u32) -> Vec<u8> {
    // 49/2 is the captured QuestReply: quest id followed by the accept flag.
    let mut bytes = vec![49, 0, 2, 0, 0, 0];
    bytes.extend(quest_id.to_le_bytes());
    bytes.push(1);
    bytes
}

fn quest_end_reply(quest_id: u32) -> Vec<u8> {
    // Client QuestEndReplyPacket constructor 0x00a864c0; sender 0x00a97ef0.
    let mut bytes = vec![49, 0, 14, 0, 0, 0];
    bytes.extend(quest_id.to_le_bytes());
    bytes
}

fn seed_tutorial_prerequisites(
    content: &Path,
    character_id: u64,
    race_complete: bool,
) -> QuestState {
    let mut engine =
        QuestEngine::from_split_content(content, character_id, eligibility(), None).unwrap();
    assert!(engine.offer(3_000_020, MERIGOLD).changed);
    assert!(engine.accept(3_000_020).changed);
    assert!(
        engine
            .on_event(QuestEvent::TalkToNpc { npc_id: RAINA_RUSH })
            .unwrap()
            .changed
    );
    let mut state = engine.state().clone();
    state.pending_turnin = None;
    state.quests.get_mut(&3_000_020).unwrap().status = QuestStatus::Completed;

    let mut engine =
        QuestEngine::from_split_content(content, character_id, eligibility(), Some(state)).unwrap();
    assert!(engine.offer(3_080_001, RAINA_RUSH).changed);
    assert!(engine.accept(3_080_001).changed);
    for npc_id in [CHAD_FASTFLUTTER, MUSTANG_ALLI, BURT_CREWE, RAINA_RUSH] {
        assert!(
            engine
                .on_event(QuestEvent::TalkToNpc { npc_id })
                .unwrap()
                .changed
        );
    }
    let mut state = engine.state().clone();
    state.pending_turnin = None;
    state.quests.get_mut(&3_080_001).unwrap().status = QuestStatus::Completed;

    if race_complete {
        let mut engine =
            QuestEngine::from_split_content(content, character_id, eligibility(), Some(state))
                .unwrap();
        assert!(engine.offer(3_080_002, MUSTANG_ALLI).changed);
        assert!(engine.accept(3_080_002).changed);
        assert!(
            engine
                .on_event(QuestEvent::FinishMinigame {
                    minigame_id: KART_TUTORIAL_SPEEDWAY,
                })
                .unwrap()
                .changed
        );
        state = engine.state().clone();
    }
    state
}

fn packet_gateway(
    content: &Path,
    character: osfr_tools::character_store::CharacterState,
    gameplay: GameplayState,
) -> Gateway {
    let mut gateway = Gateway::load_gameplay(
        &reference_root(),
        character.clone(),
        content,
        Some(gameplay),
    )
    .unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway
        .handle_at(&login(character.guid), ROUTE_NOW)
        .unwrap();
    for packet in [application(&[13, 0]), application(&[10, 0])] {
        gateway.handle_at(&packet, ROUTE_NOW).unwrap();
    }
    gateway
}

fn send(gateway: &mut Gateway, inner: &[u8]) -> GatewayOutcome {
    gateway.handle_at(&application(inner), ROUTE_NOW).unwrap()
}

fn bodies(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome
        .replies
        .iter()
        .map(|packet| {
            assert!(packet.len() >= 7 && packet[..3] == [5, 0, 1]);
            packet[7..].to_vec()
        })
        .collect()
}

fn interaction_options(gateway: &mut Gateway) -> Vec<(i32, u32)> {
    // Client 26/9 reader 0x00aa2360 -> 0x00aa2220 -> 0x00aa1600/0x00aa04d0.
    let replies = bodies(&send(gateway, &npc_interact(MUSTANG_ALLI)));
    let menu = replies
        .iter()
        .find(|body| body.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 returns the 26/9 interaction menu");
    let count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    let mut offset = 17;
    let mut options = Vec::new();
    for _ in 0..count {
        options.push((
            i32::from_le_bytes(menu[offset..offset + 4].try_into().unwrap()),
            u32::from_le_bytes(menu[offset + 8..offset + 12].try_into().unwrap()),
        ));
        offset += 28;
    }
    options
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

#[test]
fn mustang_racing_tutorial_offer_and_race_entry_use_npc_packets() {
    let content = content_root();
    let route = RouteContent::new();
    let mut character = Gateway::reference_state(&reference_root()).unwrap();
    character.zone_id = 0;
    character.position = MUSTANG_POSITION;
    let mut gameplay = Gameplay::load(&route.0, None).unwrap().state();
    kart_driver_profiles(&mut gameplay);
    gameplay.npc_quests = Some(seed_tutorial_prerequisites(&content, character.guid, false));
    let mut gateway = packet_gateway(&route.0, character, gameplay);

    let options = interaction_options(&mut gateway);
    let talk = options
        .iter()
        .find_map(|(id, label)| (*label == TALK_MENU_STRING_ID).then_some(*id))
        .expect("Mustang Alli exposes the generic Talk option");

    let talk_replies = bodies(&send(&mut gateway, &npc_select(MUSTANG_ALLI, talk)));
    assert!(
        talk_replies
            .iter()
            .any(|body| body.starts_with(&[26, 0, 3, 0])),
        "Talk must open the dialogue carrying the quest choice"
    );
    let offer_replies = bodies(&send(&mut gateway, &quest_choice()));
    let offer = offer_replies
        .iter()
        .find(|body| body.starts_with(&[49, 0, 1, 0]))
        .expect("choosing the quest row opens the 49/1 offer");
    assert_eq!(
        u32::from_le_bytes(offer[6..10].try_into().unwrap()),
        3_080_002
    );
    assert_eq!(
        u32::from_le_bytes(offer[10..14].try_into().unwrap()),
        TUTORIAL_INTRO_STRING_ID
    );
    assert_eq!(
        u32::from_le_bytes(offer[22..26].try_into().unwrap()),
        TUTORIAL_DESCRIPTION_STRING_ID
    );

    let accepted = send(&mut gateway, &quest_reply(3_080_002));
    assert!(accepted.gameplay_changed);
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .npc_quests
            .unwrap()
            .quests[&3_080_002]
            .status,
        QuestStatus::Active
    );

    let options = interaction_options(&mut gateway);
    let race = options
        .iter()
        .find_map(|(id, label)| (*label == RACE_MENU_STRING_ID).then_some(*id))
        .expect("the configured Mustang Alli gate exposes the supported race");
    assert!(
        !options.iter().any(|(_, label)| *label == 117323),
        "the active return objective must continue to use Talk"
    );

    let before_refusal = gateway.snapshot();
    let refused = send(&mut gateway, &npc_select(MUSTANG_ALLI, 99));
    assert!(bodies(&refused).is_empty());
    assert_eq!(
        gateway.snapshot(),
        before_refusal,
        "an invalid menu choice must change nothing"
    );

    let started = send(&mut gateway, &npc_select(MUSTANG_ALLI, race));
    assert!(started.gameplay_changed);
    assert!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .kart_entry
            .unwrap()
            .active
            .is_some(),
        "selecting the race option must enter the configured kart race"
    );
}

#[test]
fn mustang_talk_completes_return_objective_and_pays_once_after_reconnect() {
    let content = content_root();
    let route = RouteContent::new();
    let mut character = Gateway::reference_state(&reference_root()).unwrap();
    character.zone_id = 0;
    character.position = MUSTANG_POSITION;
    let mut gameplay = Gameplay::load(&route.0, None).unwrap().state();
    kart_driver_profiles(&mut gameplay);
    gameplay.npc_quests = Some(seed_tutorial_prerequisites(&content, character.guid, true));
    let mut gateway = packet_gateway(&route.0, character, gameplay);

    let options = interaction_options(&mut gateway);
    let talk = options
        .iter()
        .find_map(|(id, label)| (*label == TALK_MENU_STRING_ID).then_some(*id))
        .expect("Mustang Alli exposes the generic Talk option");
    assert!(
        !options.iter().any(|(_, label)| *label == 117323),
        "an active Racing Tutorial must not expose begin_turnin as a menu shortcut"
    );

    let talk_replies = bodies(&send(&mut gateway, &npc_select(MUSTANG_ALLI, talk)));
    assert!(
        talk_replies
            .iter()
            .any(|body| body.starts_with(&[26, 0, 3, 0]))
    );
    assert!(
        !talk_replies
            .iter()
            .any(|body| body.starts_with(&[49, 0, 13, 0])),
        "the end screen opens only after the client chooses the Talk result"
    );
    let progress = &gateway
        .snapshot()
        .gameplay
        .unwrap()
        .npc_quests
        .unwrap()
        .quests[&3_080_002];
    assert_eq!(progress.status, QuestStatus::Active);
    assert_eq!(progress.counts, [1, 1]);

    let end_replies = bodies(&send(&mut gateway, &quest_choice()));
    let end = end_replies
        .iter()
        .find(|body| body.starts_with(&[49, 0, 13, 0]))
        .expect("choosing the Talk row opens the 49/13 end screen");
    assert_eq!(
        u32::from_le_bytes(end[18..22].try_into().unwrap()),
        TUTORIAL_END_STRING_ID
    );

    let character = gateway.state().clone();
    let gameplay = gateway.snapshot().gameplay.unwrap();
    let mut gateway = packet_gateway(&route.0, character, gameplay);
    let coins_before = gateway.snapshot().gameplay.unwrap().inventory.coins;
    let completed = send(&mut gateway, &quest_end_reply(3_080_002));
    let completion = bodies(&completed);
    assert!(
        completion
            .iter()
            .any(|body| body.starts_with(&[49, 0, 4, 0]))
    );
    let after = gateway.snapshot().gameplay.unwrap();
    assert_eq!(after.inventory.coins, coins_before + 5);
    assert_eq!(
        after.npc_quests.as_ref().unwrap().quests[&3_080_002].status,
        QuestStatus::Completed
    );
    assert_eq!(after.npc_quests.as_ref().unwrap().pending_turnin, None);

    let snapshot = gateway.snapshot();
    let replay = send(&mut gateway, &quest_end_reply(3_080_002));
    assert!(!replay.gameplay_changed);
    assert_eq!(
        gateway.snapshot(),
        snapshot,
        "a repeated 49/14 must not restore or change live state"
    );
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins,
        after.inventory.coins
    );
}

/// T229: the same routed turn-in pays the quest's Station Cash kind once. The first completion of
/// the UTC day also pays the data bonus; a repeated 49/14 re-sends nothing and pays nothing.
#[test]
fn mustang_turnin_pays_the_quest_station_cash_once_and_survives_reconnect() {
    let content = content_root();
    let route = RouteContent::new();
    let mut character = Gateway::reference_state(&reference_root()).unwrap();
    character.zone_id = 0;
    character.position = MUSTANG_POSITION;
    let mut gameplay = Gameplay::load(&route.0, None).unwrap().state();
    kart_driver_profiles(&mut gameplay);
    gameplay.npc_quests = Some(seed_tutorial_prerequisites(&content, character.guid, true));
    let mut gateway = packet_gateway(&route.0, character, gameplay);

    let options = interaction_options(&mut gateway);
    let talk = options
        .iter()
        .find_map(|(id, label)| (*label == TALK_MENU_STRING_ID).then_some(*id))
        .expect("Mustang Alli exposes the generic Talk option");
    let _ = send(&mut gateway, &npc_select(MUSTANG_ALLI, talk));
    let _ = send(&mut gateway, &quest_choice());

    // Reconnect with the open end screen, like the completion test above.
    let character = gateway.state().clone();
    let gameplay = gateway.snapshot().gameplay.unwrap();
    let mut gateway = packet_gateway(&route.0, character, gameplay);
    let completed = send(&mut gateway, &quest_end_reply(3_080_002));
    let completion = bodies(&completed);
    assert!(
        completion
            .iter()
            .any(|body| body.starts_with(&[66, 0, 11, 0])),
        "the earned Station Cash pushes the wallet response"
    );
    let wallet = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .sc_wallet
        .expect("a paid quest writes the Station Cash wallet");
    assert_eq!(
        wallet.balance,
        sc_earn::first_of_day(osfr_tools::station_cash::EarnSource::Quest { id: 3_080_002 }),
        "one quest completion plus its first-activity-of-the-day bonus"
    );
    assert!(wallet.last_activity_day.is_some());
    let saved = gateway.snapshot();

    // The same turn-in reply is inert: no second grant, no wallet change.
    let replay = send(&mut gateway, &quest_end_reply(3_080_002));
    assert!(!replay.gameplay_changed);
    assert_eq!(gateway.snapshot(), saved);

    // The wallet is in the character save, so a reconnect keeps exactly one payout.
    let character = gateway.state().clone();
    let gameplay = gateway.snapshot().gameplay.unwrap();
    let mut restarted = packet_gateway(&route.0, character, gameplay);
    assert_eq!(
        restarted.snapshot().gameplay.unwrap().sc_wallet,
        Some(wallet.clone())
    );
    let replay = send(&mut restarted, &quest_end_reply(3_080_002));
    assert!(!replay.gameplay_changed);
    assert_eq!(
        restarted.snapshot().gameplay.unwrap().sc_wallet,
        Some(wallet)
    );
}
