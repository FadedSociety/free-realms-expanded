use osfr_tools::{content, inventory, opening, profiles, quest_engine, rewards, startup_world};

use content::ContentDb;
use inventory::Inventory;
use opening::{OpeningEngine, OpeningEvent, OpeningOutcome, OpeningTrigger};
use profiles::ProfileState;
use quest_engine::{
    CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestDenial,
    QuestEngine, QuestEvent, QuestMessage, QuestObjective, QuestRequirements, QuestRewardSummary,
    QuestStatus,
};
use rewards::{ItemGrant, RewardDisplay, RewardGrant, RewardState};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const CHARACTER: u64 = 800;
const OPENING_ITEM: i32 = 14_794;

struct InventoryFixture(PathBuf);

impl InventoryFixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-quest-engine-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let zero = 0i32.to_le_bytes();
        let definitions = [500, OPENING_ITEM]
            .into_iter()
            .map(|id| {
                let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
                json!({
                    "id": id,
                    "slot": 3,
                    "class_id": 1,
                    "default_tint": 0,
                    "max_stack": 99,
                    "model": "synthetic.adr",
                    "texture": "synthetic",
                    "tint_alias": "synthetic",
                    "effect": 0,
                    "wield_type": 0,
                    "serialized_hex": hex(&bytes),
                    "sha256": digest(&bytes)
                })
            })
            .collect::<Vec<_>>();
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
            "definitions": definitions
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(directory)
    }

    fn reward_state(&self) -> RewardState {
        RewardState::new(
            Inventory::load(&self.0, None).unwrap(),
            ProfileState {
                version: 1,
                active: 1,
                unlocked: BTreeSet::new(),
                experience: BTreeMap::new(),
            },
            16,
        )
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn content_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content")
}

fn opening_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/quests")
}

fn split_quest_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/quests")
}

fn eligibility(limit: usize) -> CharacterEligibility {
    CharacterEligibility {
        player_level: 10,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: limit,
        member_quest_limit: limit,
    }
}

fn reward(coins: u32, items: Vec<ItemGrant>, character_id: u64) -> RewardGrant {
    RewardGrant {
        coins,
        items,
        job_xp: BTreeMap::new(),
        display: RewardDisplay {
            source_guid: 1,
            player_guid: character_id,
            reward_kind: 0,
            icon_id: 0,
            name_id: 0,
            rows: Vec::new(),
        },
    }
}

fn objective(id: u32, required: u32, kind: ObjectiveKind) -> QuestObjective {
    QuestObjective {
        id,
        required,
        description: format!("Goal {id}"),
        kind,
    }
}

fn definition(
    id: u32,
    giver: u64,
    turnin: u64,
    objectives: Vec<QuestObjective>,
    coins: u32,
) -> QuestDefinition {
    QuestDefinition {
        id,
        title: format!("Quest {id}"),
        description: format!("Description {id}"),
        giver_npc_id: giver,
        turnin_npc_id: turnin,
        prerequisite_quest_ids: Vec::new(),
        requirements: QuestRequirements::default(),
        ordered: false,
        objectives,
        reward: reward(coins, Vec::new(), CHARACTER),
    }
}

fn quest_opcodes(outcome: &quest_engine::QuestOutcome) -> Vec<(u16, u32)> {
    outcome
        .outbound
        .iter()
        .filter_map(|message| match message {
            OutboundMessage::Quest(message) => Some(message.opcode()),
            OutboundMessage::Reward(_) => None,
        })
        .collect()
}

#[test]
fn content_db_definitions_drive_accept_progress_and_turnin_packets() {
    let content = ContentDb::load(content_fixture()).unwrap();
    let giver = content.quests[0].giver_npc_id;
    let mut engine =
        QuestEngine::from_content_db(&content, CHARACTER, eligibility(8), None).unwrap();
    let offered = engine.offer(101, giver);
    assert_eq!(quest_opcodes(&offered), vec![(49, 1)]);
    assert!(matches!(
        &offered.outbound[0],
        OutboundMessage::Quest(message) if message.encode_exact().is_none()
    ));

    let accepted = engine.accept(101);
    assert_eq!(quest_opcodes(&accepted), vec![(49, 3), (49, 8)]);
    let accepted_record = engine.quest_add_packet(101).unwrap();
    let login = engine.active_login_packets(&BTreeSet::new()).unwrap();
    assert_eq!(login[0], accepted_record);
    assert_eq!(
        login[0],
        match &accepted.outbound[0] {
            OutboundMessage::Quest(QuestMessage::QuestAdd { quest_id }) => {
                engine.quest_add_packet(*quest_id).unwrap()
            }
            _ => panic!("accept starts with the quest record"),
        }
    );
    let accepted_activation = match &accepted.outbound[1] {
        OutboundMessage::Quest(message) => message.encode_exact().unwrap(),
        _ => panic!("accept sends an objective activation"),
    };
    assert_eq!(login[1], accepted_activation);
    match &accepted.outbound[1] {
        OutboundMessage::Quest(message) => {
            let encoded = message.encode_exact().unwrap();
            assert_eq!(encoded.len(), 19);
            assert_eq!(&encoded[..6], &[49, 0, 8, 0, 0, 0]);
        }
        _ => panic!("objective activation must be a quest packet"),
    }

    let progress = engine
        .on_event(QuestEvent::CollectItem {
            item_id: 500,
            quantity: 1,
        })
        .unwrap();
    assert_eq!(quest_opcodes(&progress), vec![(49, 9), (49, 10)]);
    match &progress.outbound[0] {
        OutboundMessage::Quest(message) => assert_eq!(message.encode_exact().unwrap().len(), 27),
        _ => panic!("objective update must be a quest packet"),
    }
    match &progress.outbound[1] {
        OutboundMessage::Quest(message) => assert_eq!(message.encode_exact().unwrap().len(), 23),
        _ => panic!("objective completion must be a quest packet"),
    }

    let end = engine.begin_turnin(101, giver);
    assert_eq!(quest_opcodes(&end), vec![(49, 13)]);
    assert!(matches!(
        &end.outbound[0],
        OutboundMessage::Quest(message) if message.encode_exact().is_none()
    ));
    assert!(engine.empty_reward_quest_end_packet(101).is_none());
    let inventory = InventoryFixture::new();
    let (settled, state) = engine
        .finish_turnin(101, &inventory.reward_state())
        .unwrap();
    assert_eq!(quest_opcodes(&settled), vec![(49, 4), (49, 12)]);
    assert_eq!(state.inventory.state().items[0].definition_id, 500);
    assert_eq!(engine.progress(101).unwrap().status, QuestStatus::Completed);
}

#[test]
fn empty_objective_quest_offers_accepts_and_completes_on_talk_back() {
    let giver = 9_000_000_189;
    let mut engine = QuestEngine::new(
        CHARACTER,
        vec![definition(3_000_021, giver, giver, Vec::new(), 0)],
        eligibility(8),
        None,
    )
    .unwrap();

    assert!(engine.can_offer(3_000_021, giver));
    assert_eq!(
        quest_opcodes(&engine.offer(3_000_021, giver)),
        vec![(49, 1)]
    );
    assert_eq!(quest_opcodes(&engine.accept(3_000_021)), vec![(49, 3)]);
    let end = engine
        .on_event(QuestEvent::TalkToNpc { npc_id: giver })
        .unwrap();
    assert_eq!(quest_opcodes(&end), vec![(49, 13)]);
    let packet = engine.empty_reward_quest_end_packet(3_000_021).unwrap();
    assert_eq!(packet.len(), 99);
    assert_eq!(&packet[..10], &[49, 0, 13, 0, 0, 0, 0xd5, 0xc6, 0x2d, 0]);
    // Reply/end-screen id at offset 14; everything else after the header is zero.
    assert_eq!(&packet[14..18], &3_000_021_i32.to_le_bytes());
    assert!(
        packet[10..14]
            .iter()
            .chain(&packet[18..])
            .all(|byte| *byte == 0)
    );

    let inventory = InventoryFixture::new();
    let (completed, _) = engine
        .finish_turnin(3_000_021, &inventory.reward_state())
        .unwrap();
    assert_eq!(quest_opcodes(&completed), vec![(49, 4), (49, 12)]);
    assert_eq!(
        engine.progress(3_000_021).unwrap().status,
        QuestStatus::Completed
    );
}

#[test]
fn all_six_requested_objective_kinds_increment_counters() {
    let objectives = vec![
        objective(1, 1, ObjectiveKind::TalkToNpc { npc_id: 10 }),
        objective(2, 2, ObjectiveKind::KillActorKind { actor_kind_id: 20 }),
        objective(3, 3, ObjectiveKind::CollectItem { item_id: 30 }),
        objective(4, 1, ObjectiveKind::EnterArea { area_id: 40 }),
        objective(5, 1, ObjectiveKind::FinishMinigame { minigame_id: 50 }),
        objective(6, 2, ObjectiveKind::UseItem { item_id: 60 }),
    ];
    let mut engine = QuestEngine::new(
        CHARACTER,
        vec![definition(200, 1, 99, objectives, 0)],
        eligibility(8),
        None,
    )
    .unwrap();
    engine.offer(200, 1);
    engine.accept(200);
    let events = [
        QuestEvent::TalkToNpc { npc_id: 10 },
        QuestEvent::KillActorKind {
            actor_kind_id: 20,
            count: 2,
        },
        QuestEvent::CollectItem {
            item_id: 30,
            quantity: 3,
        },
        QuestEvent::EnterArea { area_id: 40 },
        QuestEvent::FinishMinigame { minigame_id: 50 },
        QuestEvent::UseItem {
            item_id: 60,
            quantity: 2,
        },
    ];
    let mut sequence = Vec::new();
    for event in events {
        sequence.extend(quest_opcodes(&engine.on_event(event).unwrap()));
    }
    assert_eq!(engine.progress(200).unwrap().counts, vec![1, 2, 3, 1, 1, 2]);
    assert_eq!(
        sequence,
        (0..6).flat_map(|_| [(49, 9), (49, 10)]).collect::<Vec<_>>()
    );
}

#[test]
fn prerequisite_level_job_member_and_limit_gates_are_server_owned() {
    let first = definition(
        1,
        1,
        1,
        vec![objective(11, 1, ObjectiveKind::EnterArea { area_id: 3 })],
        0,
    );
    let mut gated = definition(
        2,
        2,
        2,
        vec![objective(22, 1, ObjectiveKind::TalkToNpc { npc_id: 77 })],
        0,
    );
    gated.prerequisite_quest_ids = vec![1];
    gated.requirements = QuestRequirements {
        player_level: Some(3),
        job: Some(("chef".into(), 2)),
        members_only: true,
    };
    let extra = definition(
        3,
        3,
        3,
        vec![objective(33, 1, ObjectiveKind::EnterArea { area_id: 9 })],
        0,
    );
    let definitions = vec![first.clone(), gated.clone(), extra];
    let inventory = InventoryFixture::new();
    let mut limit_context = eligibility(1);
    limit_context.member = false;
    let mut engine = QuestEngine::new(CHARACTER, definitions.clone(), limit_context, None).unwrap();
    assert_eq!(engine.offer(2, 2).denial, Some(QuestDenial::Prerequisite));
    engine.offer(1, 1);
    engine.accept(1);
    let limited = engine.offer(3, 3);
    assert_eq!(limited.denial, Some(QuestDenial::Limit));
    match &limited.outbound[0] {
        OutboundMessage::Quest(message) => {
            assert_eq!(message.encode_exact().unwrap(), vec![47, 0, 15, 2])
        }
        _ => panic!("quest limit must be sent as family 47/15"),
    }
    engine
        .on_event(QuestEvent::EnterArea { area_id: 3 })
        .unwrap();
    engine.begin_turnin(1, 1);
    let (_, reward_state) = engine.finish_turnin(1, &inventory.reward_state()).unwrap();
    let saved = engine.state().clone();

    let mut low_context = eligibility(1);
    low_context.player_level = 1;
    let mut low = QuestEngine::new(
        CHARACTER,
        definitions.clone(),
        low_context,
        Some(saved.clone()),
    )
    .unwrap();
    assert_eq!(low.offer(2, 2).denial, Some(QuestDenial::Level));
    let mut levelled_context = eligibility(1);
    levelled_context.player_level = 3;
    levelled_context.job_levels.insert("chef".into(), 1);
    let mut low_job = QuestEngine::new(
        CHARACTER,
        definitions.clone(),
        levelled_context,
        Some(saved.clone()),
    )
    .unwrap();
    assert_eq!(low_job.offer(2, 2).denial, Some(QuestDenial::Job));
    let mut job_context = eligibility(1);
    job_context.player_level = 3;
    job_context.job_levels.insert("chef".into(), 2);
    job_context.member = false;
    let mut nonmember = QuestEngine::new(
        CHARACTER,
        definitions.clone(),
        job_context,
        Some(saved.clone()),
    )
    .unwrap();
    assert_eq!(nonmember.offer(2, 2).denial, Some(QuestDenial::Membership));
    let mut qualified_context = eligibility(1);
    qualified_context.player_level = 3;
    qualified_context.job_levels.insert("chef".into(), 2);
    let mut qualified =
        QuestEngine::new(CHARACTER, definitions, qualified_context, Some(saved)).unwrap();
    assert!(qualified.offer(2, 2).changed);
    assert_eq!(reward_state.inventory.state().coins, 0);
}

#[test]
fn decline_abandon_and_reload_preserve_only_valid_progress() {
    let mut def = definition(
        77,
        7,
        9,
        vec![
            objective(1, 3, ObjectiveKind::KillActorKind { actor_kind_id: 8 }),
            objective(2, 1, ObjectiveKind::EnterArea { area_id: 9 }),
        ],
        25,
    );
    def.ordered = true;
    let definitions = vec![def];
    let mut engine =
        QuestEngine::new(CHARACTER, definitions.clone(), eligibility(8), None).unwrap();
    engine.offer(77, 7);
    assert!(engine.decline(77).changed);
    assert!(engine.progress(77).is_none());
    engine.offer(77, 7);
    engine.accept(77);
    assert!(engine.select_quest(77).changed);
    assert!(engine.set_auto_switch_selected(true).changed);
    engine
        .on_event(QuestEvent::KillActorKind {
            actor_kind_id: 8,
            count: 1,
        })
        .unwrap();
    let saved: quest_engine::QuestState =
        serde_json::from_slice(&serde_json::to_vec(engine.state()).unwrap()).unwrap();
    let mut resumed =
        QuestEngine::new(CHARACTER, definitions.clone(), eligibility(8), Some(saved)).unwrap();
    assert_eq!(resumed.progress(77).unwrap().counts, vec![1, 0]);
    assert_eq!(resumed.state().selected_quest_id, Some(77));
    assert!(resumed.state().auto_switch_selected);
    assert!(
        !resumed
            .on_event(QuestEvent::EnterArea { area_id: 9 })
            .unwrap()
            .changed
    );
    let completed_goal = resumed
        .on_event(QuestEvent::KillActorKind {
            actor_kind_id: 8,
            count: 2,
        })
        .unwrap();
    assert_eq!(
        quest_opcodes(&completed_goal),
        vec![(49, 9), (49, 10), (49, 8)]
    );
    let abandoned = resumed.abandon(77);
    assert_eq!(quest_opcodes(&abandoned), vec![(49, 6)]);
    assert_eq!(resumed.progress(77).unwrap().status, QuestStatus::Abandoned);
    assert_eq!(resumed.state().selected_quest_id, None);
    assert!(resumed.offer(77, 7).changed);
    assert_eq!(resumed.progress(77).unwrap().counts, vec![0, 0]);
}

#[test]
fn duplicate_turnin_retry_uses_the_character_quest_grant_key_once() {
    let mut def = definition(
        88,
        8,
        9,
        vec![objective(1, 1, ObjectiveKind::EnterArea { area_id: 5 })],
        5,
    );
    def.reward.items = vec![ItemGrant {
        definition_id: OPENING_ITEM,
        count: 1,
    }];
    let definitions = vec![def.clone()];
    let inventory = InventoryFixture::new();
    let empty_reward_state = inventory.reward_state();
    let mut engine =
        QuestEngine::new(CHARACTER, definitions.clone(), eligibility(8), None).unwrap();
    engine.offer(88, 8);
    engine.accept(88);
    engine
        .on_event(QuestEvent::EnterArea { area_id: 5 })
        .unwrap();
    let began = engine.begin_turnin(88, 9);
    assert_eq!(quest_opcodes(&began), vec![(49, 13)]);
    let pending = engine.state().clone();
    let (first, paid) = engine.finish_turnin(88, &empty_reward_state).unwrap();
    assert_eq!(paid.inventory.state().coins, 5);
    assert_eq!(paid.inventory.state().items[0].count, 1);
    assert_eq!(
        paid.applied_grant_keys,
        BTreeSet::from(["quest:800:88".into()])
    );
    assert_eq!(
        first.rewards,
        vec![QuestRewardSummary {
            quest_id: 88,
            coins: 5,
            items: vec![(OPENING_ITEM, 1)]
        }]
    );
    let done_order: Vec<_> = first
        .outbound
        .iter()
        .filter_map(|message| match message {
            OutboundMessage::Quest(message) => Some(message.opcode()),
            _ => None,
        })
        .collect();
    assert_eq!(done_order, vec![(49, 4), (49, 12)]);
    assert!(matches!(
        first.outbound.last(),
        Some(OutboundMessage::Quest(QuestMessage::CompletedQuestCount {
            count: 1
        }))
    ));

    // Simulate a retry snapshot with the grant key already committed but the quest still pending.
    let mut retry =
        QuestEngine::new(CHARACTER, definitions, eligibility(8), Some(pending)).unwrap();
    let (retried, paid_again) = retry.finish_turnin(88, &paid).unwrap();
    assert_eq!(paid_again.inventory.state().coins, 5);
    assert_eq!(paid_again.inventory.state().items[0].count, 1);
    assert!(
        !retried
            .outbound
            .iter()
            .any(|message| matches!(message, OutboundMessage::Reward(_)))
    );
    assert_eq!(retry.progress(88).unwrap().status, QuestStatus::Completed);
    let duplicate = retry.finish_turnin(88, &paid_again).unwrap().0;
    assert_eq!(duplicate.denial, Some(QuestDenial::NoPendingTurnin));
}

fn fixture_actors() -> (PathBuf, OpeningEngine) {
    let directory = opening_fixture();
    let opening = OpeningEngine::load(&directory, None).unwrap();
    (directory, opening)
}

fn set_legacy_location(opening: &mut OpeningEngine, actor_id: u64) {
    let actor = opening
        .actors()
        .iter()
        .find(|actor| actor.guid == actor_id)
        .unwrap();
    opening
        .set_location_without_events(actor.zone_id, actor.position)
        .unwrap();
}

fn legacy_objective_event(opening: &mut OpeningEngine, trigger: &OpeningTrigger) -> OpeningOutcome {
    match trigger {
        OpeningTrigger::Talk { actor_guid } | OpeningTrigger::Interact { actor_guid } => {
            set_legacy_location(opening, *actor_guid);
            opening.on_interaction(*actor_guid).unwrap()
        }
        OpeningTrigger::Gather { resource_id } => {
            let actor = opening.actors().iter().find(|actor| matches!(
                actor.action.as_ref(), Some(OpeningTrigger::Gather { resource_id: id }) if id == resource_id
            )).unwrap();
            let actor_id = actor.guid;
            set_legacy_location(opening, actor_id);
            opening.on_interaction(actor_id).unwrap()
        }
        OpeningTrigger::Craft { recipe_id } => {
            let actor = opening.actors().iter().find(|actor| matches!(
                actor.action.as_ref(), Some(OpeningTrigger::Craft { recipe_id: id }) if id == recipe_id
            )).unwrap();
            let actor_id = actor.guid;
            set_legacy_location(opening, actor_id);
            opening.on_interaction(actor_id).unwrap()
        }
        OpeningTrigger::Visit {
            zone_id, position, ..
        } => opening.on_position(*zone_id, *position).unwrap(),
        trigger => opening
            .on_event(match trigger {
                OpeningTrigger::Defeat { definition_id } => OpeningEvent::Defeat {
                    definition_id: *definition_id,
                },
                OpeningTrigger::ClassSelected { profile_id } => OpeningEvent::ClassSelected {
                    profile_id: *profile_id,
                },
                OpeningTrigger::CosmeticEquipped { definition_id } => {
                    OpeningEvent::CosmeticEquipped {
                        definition_id: *definition_id,
                    }
                }
                OpeningTrigger::DungeonCompleted { encounter_id } => {
                    OpeningEvent::DungeonCompleted {
                        encounter_id: *encounter_id,
                    }
                }
                OpeningTrigger::AbilityUsed { ability_id } => OpeningEvent::AbilityUsed {
                    ability_id: *ability_id,
                },
                _ => unreachable!(),
            })
            .unwrap(),
    }
}

fn reward_summaries(outcome: &OpeningOutcome) -> Vec<QuestRewardSummary> {
    outcome
        .rewards
        .iter()
        .map(|reward| QuestRewardSummary {
            quest_id: reward.stage_id,
            coins: reward.coins,
            items: reward.items.clone(),
        })
        .collect()
}

#[test]
fn source_tagged_opening_fixture_matches_the_existing_seventeen_stage_engine() {
    let (directory, mut legacy) = fixture_actors();
    let fixture: Value =
        serde_json::from_slice(&fs::read(directory.join("opening.json")).unwrap()).unwrap();
    assert_eq!(fixture["stages"].as_array().unwrap().len(), 17);
    assert_eq!(fixture["provenance"]["source"][0]["kind"], "reconstructed");
    assert!(
        fixture["provenance"]["quests"]
            .as_array()
            .unwrap()
            .iter()
            .all(|quest| {
                quest["source"].is_array()
                    && quest["objectives"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|goal| goal["source"].is_array())
            })
    );

    let mut engine = QuestEngine::from_opening(&legacy, CHARACTER, eligibility(30), None).unwrap();
    let stages = legacy.stages().to_vec();
    let mut old_messages = Vec::new();
    let mut new_messages = Vec::new();
    let mut old_rewards = Vec::new();
    let mut new_rewards = Vec::new();
    let inventory = InventoryFixture::new();
    let mut reward_state = inventory.reward_state();

    for stage in &stages {
        assert_eq!(legacy.active_stage().unwrap().id, stage.id);
        set_legacy_location(&mut legacy, stage.giver_guid);
        let old_accept = legacy.on_interaction(stage.giver_guid).unwrap();
        old_messages.extend(old_accept.messages);
        let offered = engine.offer(stage.id, stage.giver_guid);
        assert_eq!(offered.denial, None, "offer {}", stage.id);
        let accepted = engine.accept(stage.id);
        new_messages.extend(accepted.messages);

        for goal in &stage.objectives {
            for _ in 0..goal.required {
                let old_step = legacy_objective_event(&mut legacy, &goal.trigger);
                old_rewards.extend(reward_summaries(&old_step));
                old_messages.extend(old_step.messages);
                let new_step = engine
                    .on_event(QuestEvent::from(goal.trigger.clone()))
                    .unwrap();
                new_messages.extend(new_step.messages);
                if engine.state().pending_turnin == Some(stage.id) {
                    let (settled, next) = engine.finish_turnin(stage.id, &reward_state).unwrap();
                    reward_state = next;
                    new_messages.extend(settled.messages);
                    new_rewards.extend(settled.rewards);
                }
            }
        }

        if !legacy.state().stages[&stage.id].completed {
            set_legacy_location(&mut legacy, stage.turnin_guid);
            let old_turnin = legacy.on_interaction(stage.turnin_guid).unwrap();
            old_rewards.extend(reward_summaries(&old_turnin));
            old_messages.extend(old_turnin.messages);
            let new_turnin = engine
                .on_event(QuestEvent::TalkToNpc {
                    npc_id: stage.turnin_guid,
                })
                .unwrap();
            new_messages.extend(new_turnin.messages);
            assert_eq!(engine.state().pending_turnin, Some(stage.id));
            let (settled, next) = engine.finish_turnin(stage.id, &reward_state).unwrap();
            reward_state = next;
            new_messages.extend(settled.messages);
            new_rewards.extend(settled.rewards);
        }
        assert_eq!(
            legacy.state().stages[&stage.id].completed,
            true,
            "legacy stage {}",
            stage.id
        );
        assert_eq!(
            engine.progress(stage.id).unwrap().status,
            QuestStatus::Completed
        );
    }

    assert_eq!(old_messages, new_messages);
    assert_eq!(old_rewards, new_rewards);
    assert!(legacy.complete());
    assert_eq!(reward_state.applied_grant_keys.len(), 17);
}

#[test]
fn protocol_message_builders_leave_contextual_quest_end_unencoded() {
    let offer = QuestMessage::QuestInfo { quest_id: 1 };
    let add = QuestMessage::QuestAdd { quest_id: 1 };
    let end = QuestMessage::QuestEnd { quest_id: 1 };
    assert_eq!(offer.opcode(), (49, 1));
    assert_eq!(add.opcode(), (49, 3));
    assert_eq!(end.opcode(), (49, 13));
    assert!(offer.encode_exact().is_none());
    assert!(add.encode_exact().is_none());
    assert!(end.encode_exact().is_none());
}

#[test]
fn login_journal_builder_orders_209_1_before_209_2() {
    let definitions = startup_world::JournalDefinitions {
        regions: Vec::new(),
        hubs: Vec::new(),
        hub_quests: Vec::new(),
        stickers: Vec::new(),
    };
    let packets = QuestEngine::journal_login_packets(&definitions, &[(7, 1)]);
    assert_eq!(packets.len(), 2);
    assert_eq!(&packets[0][..7], &[5, 0, 1, 22, 0, 0, 0]);
    assert_eq!(&packets[0][7..13], &[209, 0, 1, 0, 0, 0]);
    assert_eq!(&packets[1][..7], &[5, 0, 1, 18, 0, 0, 0]);
    assert_eq!(&packets[1][7..13], &[209, 0, 2, 0, 0, 0]);
}

#[test]
fn split_regional_quest_catalogue_accepts_and_restores_an_npc_offer() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let mut quest_files: Vec<_> = fs::read_dir(directory.join("quests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    quest_files.sort();
    let quests: Vec<Value> = quest_files
        .into_iter()
        .flat_map(|path| {
            serde_json::from_slice::<Value>(&fs::read(path).unwrap()).unwrap()["records"]
                .as_array()
                .unwrap()
                .clone()
        })
        .collect();
    let mut npc_files: Vec<_> = fs::read_dir(directory.join("npcs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    npc_files.sort();
    let mut giver_relations = BTreeSet::new();
    for path in npc_files {
        let document: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        for npc in document["records"].as_array().unwrap() {
            let npc_id = npc["npc_id"].as_u64().unwrap();
            let mut ids = npc["quest_ids_given"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_u64)
                .collect::<Vec<_>>();
            ids.extend(
                npc["quests_given"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|quest| quest["quest_id"].as_u64()),
            );
            giver_relations.extend(ids.into_iter().map(|id| (id as u32, npc_id)));
        }
    }
    let quest = quests
        .iter()
        .find(|quest| {
            quest["prerequisite_quest_ids"]
                .as_array()
                .is_some_and(Vec::is_empty)
                && quest["id"].as_u64().is_some_and(|id| {
                    quest["giver_npc_id"]
                        .as_u64()
                        .is_some_and(|giver| giver_relations.contains(&(id as u32, giver)))
                })
        })
        .expect("split NPC content has a root quest giver");
    let quest_id = quest["id"].as_u64().unwrap() as u32;
    let giver_npc_id = quest["giver_npc_id"].as_u64().unwrap();
    let eligibility = CharacterEligibility {
        player_level: 1,
        job_levels: BTreeMap::new(),
        member: false,
        nonmember_quest_limit: 100,
        member_quest_limit: 100,
    };
    let mut engine =
        QuestEngine::from_split_content(&directory, CHARACTER, eligibility.clone(), None).unwrap();
    let offer = engine.offer(quest_id, giver_npc_id);
    assert!(offer.changed);
    let accepted = engine.accept(quest_id);
    assert!(accepted.changed);
    let packet = engine.quest_add_packet(quest_id).unwrap();
    assert_eq!(&packet[..6], &[49, 0, 3, 0, 0, 0]);
    assert!(packet.len() >= 6 + 4 * 3 + 1);

    let saved = serde_json::from_slice(&serde_json::to_vec(engine.state()).unwrap()).unwrap();
    let restored =
        QuestEngine::from_split_content(&directory, CHARACTER, eligibility, Some(saved)).unwrap();
    assert_eq!(
        restored.progress(quest_id).unwrap().status,
        QuestStatus::Active
    );
}

#[test]
fn split_quest_locale_ids_resolve_to_exported_rows() {
    let locale_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/reports/locale-id-to-text.json");
    let locale: Vec<Value> = serde_json::from_slice(&fs::read(locale_path).unwrap()).unwrap();
    let locale_ids: BTreeSet<u32> = locale
        .iter()
        .filter_map(|row| row["id"].as_u64().and_then(|id| u32::try_from(id).ok()))
        .collect();
    let mut files: Vec<_> = fs::read_dir(split_quest_directory())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    let mut highroad_intro = None;
    let mut samantha = None;
    for path in files {
        let document: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        for quest in document["records"].as_array().unwrap() {
            for field in [
                "title_string_id",
                "intro_text_string_id",
                "description_string_id",
            ] {
                if let Some(id) = quest[field].as_u64() {
                    let id = u32::try_from(id).unwrap();
                    assert!(
                        locale_ids.contains(&id),
                        "quest {} {field} {id} is absent from locale export",
                        quest["id"]
                    );
                }
            }
            match quest["id"].as_u64() {
                Some(3_000_020) => highroad_intro = Some(quest.clone()),
                Some(3_000_021) => samantha = Some(quest.clone()),
                _ => {}
            }
        }
    }
    let highroad_intro = highroad_intro.expect("Highroad Introduce Yourself quest exists");
    assert_eq!(highroad_intro["title_string_id"].as_u64(), Some(94_247));
    assert_eq!(
        highroad_intro["intro_text_string_id"].as_u64(),
        Some(104_097)
    );
    assert_eq!(
        highroad_intro["description_string_id"].as_u64(),
        Some(104_099)
    );
    let samantha = samantha.expect("Samantha quest exists");
    assert_eq!(samantha["title_string_id"].as_u64(), Some(94_247));
    assert_eq!(samantha["intro_text_string_id"].as_u64(), Some(94_246));
    assert_eq!(samantha["description_string_id"].as_u64(), Some(94_248));
}
