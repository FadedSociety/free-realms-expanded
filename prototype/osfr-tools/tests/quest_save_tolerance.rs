//! (R) Content-edit compatibility and authored requirement policy, not retail save semantics.
use osfr_tools::{
    quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestDenial, QuestEngine, QuestEvent,
        QuestObjective, QuestProgress, QuestRequirements, QuestState, QuestStatus,
    },
    rewards::{RewardDisplay, RewardGrant},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

fn eligibility(level: u8) -> CharacterEligibility {
    CharacterEligibility {
        player_level: level,
        member: true,
        member_quest_limit: 30,
        nonmember_quest_limit: 30,
        ..Default::default()
    }
}

fn definition(id: u32, goals: usize) -> QuestDefinition {
    QuestDefinition {
        id,
        title: "Synthetic quest".into(),
        description: String::new(),
        giver_npc_id: 1,
        turnin_npc_id: 1,
        prerequisite_quest_ids: vec![],
        requirements: QuestRequirements::default(),
        ordered: true,
        objectives: (1..=goals as u32)
            .map(|id| QuestObjective {
                id,
                required: 2,
                description: "Synthetic objective".into(),
                kind: ObjectiveKind::CollectItem { item_id: 8 },
            })
            .collect(),
        reward: RewardGrant {
            coins: 0,
            items: vec![],
            job_xp: BTreeMap::new(),
            display: RewardDisplay {
                source_guid: 1,
                player_guid: 1,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: vec![],
            },
        },
    }
}

fn state(id: u32, counts: Vec<u32>, status: QuestStatus) -> QuestState {
    QuestState {
        version: 1,
        character_id: 1,
        quests: BTreeMap::from([(id, QuestProgress { status, counts })]),
        pending_offer: None,
        pending_turnin: None,
        pending_turnin_actor: None,
        selected_quest_id: None,
        auto_switch_selected: false,
    }
}

#[test]
fn older_save_without_the_pending_actor_falls_back_to_the_turnin_npc() {
    let mut saved = state(1, vec![2], QuestStatus::Active);
    saved.pending_turnin = Some(1);
    // A save written before F175 has no `pending_turnin_actor` key at all.
    let mut older = serde_json::to_value(&saved).unwrap();
    assert!(
        older
            .as_object_mut()
            .unwrap()
            .remove("pending_turnin_actor")
            .is_some()
    );
    let older: QuestState = serde_json::from_value(older).unwrap();
    assert_eq!(older.pending_turnin_actor, None);
    let engine = QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(older)).unwrap();
    assert_eq!(engine.state().pending_turnin, Some(1));
    assert_eq!(engine.pending_turnin_actor(), Some(1));
}

#[test]
fn more_and_fewer_objectives_reset_active_counters_and_clear_turnin() {
    for (old_counts, goals) in [(vec![2], 2), (vec![2, 2], 1), (vec![], 1), (vec![3], 1)] {
        let mut saved = state(1, old_counts, QuestStatus::Active);
        saved.pending_turnin = Some(1);
        saved.selected_quest_id = Some(1);
        let mut engine =
            QuestEngine::new(1, vec![definition(1, goals)], eligibility(1), Some(saved)).unwrap();
        assert_eq!(engine.progress(1).unwrap().counts, vec![0; goals]);
        assert_eq!(engine.progress(1).unwrap().status, QuestStatus::Active);
        assert_eq!(engine.state().pending_turnin, None);
        assert_eq!(engine.state().selected_quest_id, Some(1));
        assert!(
            !engine
                .active_login_packets(&BTreeSet::new())
                .unwrap()
                .is_empty()
        );
        assert!(
            engine
                .on_event(QuestEvent::CollectItem {
                    item_id: 8,
                    quantity: 1
                })
                .unwrap()
                .changed
        );
        let repaired = engine.state().clone();
        assert_eq!(
            QuestEngine::new(
                1,
                vec![definition(1, goals)],
                eligibility(1),
                Some(repaired.clone())
            )
            .unwrap()
            .state(),
            &repaired
        );
    }
}

#[test]
fn removed_quests_drop_active_state_and_keep_completion_history() {
    let mut saved = state(1, vec![2], QuestStatus::Completed);
    saved.quests.insert(
        3,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![2],
        },
    );
    saved.selected_quest_id = Some(3);
    saved.pending_turnin = Some(3);
    saved.quests.insert(
        4,
        QuestProgress {
            status: QuestStatus::Offered,
            counts: vec![0],
        },
    );
    saved.pending_offer = Some(4);
    let mut engine =
        QuestEngine::new(1, vec![definition(2, 1)], eligibility(1), Some(saved)).unwrap();
    assert_eq!(engine.progress(1).unwrap().status, QuestStatus::Completed);
    assert!(engine.progress(1).unwrap().counts.is_empty());
    assert!(engine.progress(3).is_none());
    assert!(engine.progress(4).is_none());
    assert_eq!(engine.state().pending_offer, None);
    assert_eq!(engine.state().pending_turnin, None);
    assert_eq!(engine.state().selected_quest_id, None);
    assert!(
        engine
            .active_login_packets(&BTreeSet::new())
            .unwrap()
            .is_empty()
    );
    assert!(
        !engine
            .on_event(QuestEvent::CollectItem {
                item_id: 8,
                quantity: 1
            })
            .unwrap()
            .changed
    );
    let history = engine.state().clone();
    let engine =
        QuestEngine::new(1, vec![definition(1, 2)], eligibility(1), Some(history)).unwrap();
    assert_eq!(engine.progress(1).unwrap().status, QuestStatus::Completed);
    assert_eq!(engine.progress(1).unwrap().counts, [2, 2]);
}

#[test]
fn completed_quests_stay_completed_and_unchanged_without_drift() {
    let saved = state(1, vec![2], QuestStatus::Completed);
    let engine = QuestEngine::new(
        1,
        vec![definition(1, 1)],
        eligibility(1),
        Some(saved.clone()),
    )
    .unwrap();
    assert_eq!(engine.state(), &saved);
    for goals in [0, 2] {
        let mut engine = QuestEngine::new(
            1,
            vec![definition(1, goals)],
            eligibility(1),
            Some(saved.clone()),
        )
        .unwrap();
        assert_eq!(engine.progress(1).unwrap().status, QuestStatus::Completed);
        assert_eq!(engine.progress(1).unwrap().counts, vec![2; goals]);
        assert!(!engine.offer(1, 1).changed);
        assert!(!engine.begin_turnin(1, 1).changed);
    }
    let mut changed = definition(1, 1);
    changed.objectives[0].required = 3;
    let engine = QuestEngine::new(1, vec![changed], eligibility(1), Some(saved)).unwrap();
    assert_eq!(engine.progress(1).unwrap().counts, [3]);
    assert_eq!(engine.progress(1).unwrap().status, QuestStatus::Completed);
}

#[test]
fn repair_does_not_accept_wrong_identity_version_or_unbounded_history() {
    let mut invalid = state(1, vec![0], QuestStatus::Active);
    invalid.pending_turnin = Some(999);
    assert!(QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(invalid)).is_err());
    let mut invalid = state(1, vec![0], QuestStatus::Active);
    invalid.selected_quest_id = Some(999);
    assert!(QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(invalid)).is_err());
    let mut invalid = state(1, vec![1], QuestStatus::Active);
    invalid.character_id = 2;
    assert!(QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(invalid)).is_err());
    let mut invalid = state(1, vec![], QuestStatus::Active);
    invalid.version = 2;
    assert!(QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(invalid)).is_err());
    let mut invalid = state(1, vec![], QuestStatus::Active);
    invalid.quests = (1..=1025)
        .map(|id| {
            (
                id,
                QuestProgress {
                    status: QuestStatus::Completed,
                    counts: vec![],
                },
            )
        })
        .collect();
    assert!(QuestEngine::new(1, vec![definition(1, 1)], eligibility(1), Some(invalid)).is_err());
}

struct SplitFixture(PathBuf);
impl SplitFixture {
    fn new(requirements: Value, enabled: bool) -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("quests")).unwrap();
        fs::create_dir_all(path.join("npcs")).unwrap();
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        fs::copy(content.join("jobs.json"), path.join("jobs.json")).unwrap();
        for folder in ["quests", "npcs"] {
            for entry in fs::read_dir(content.join(folder)).unwrap() {
                let input = entry.unwrap().path();
                let mut file: Value = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
                if folder == "quests" {
                    for quest in file["records"].as_array_mut().unwrap() {
                        if quest["id"] == 3_000_021 {
                            quest["enabled"] = enabled.into();
                            for (key, value) in requirements.as_object().unwrap() {
                                quest[key] = value.clone();
                            }
                        }
                    }
                }
                fs::write(
                    path.join(folder).join(input.file_name().unwrap()),
                    serde_json::to_vec(&file).unwrap(),
                )
                .unwrap();
            }
        }
        Self(path)
    }
    fn load(
        &self,
        context: CharacterEligibility,
        state: Option<QuestState>,
    ) -> std::io::Result<QuestEngine> {
        QuestEngine::from_split_content(&self.0, 1, context, state)
    }
}

impl Drop for SplitFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn disabled_quests_drop_active_counters_but_preserve_completed_history() {
    let fixture = SplitFixture::new(json!({}), false);
    for status in [QuestStatus::Active, QuestStatus::Completed] {
        let mut saved = state(3_000_021, vec![1, 1], status.clone());
        if status == QuestStatus::Active {
            saved.pending_turnin = Some(3_000_021);
            saved.selected_quest_id = Some(3_000_021);
        }
        let mut engine = fixture.load(eligibility(1), Some(saved)).unwrap();
        if status == QuestStatus::Completed {
            assert_eq!(
                engine.progress(3_000_021).unwrap().status,
                QuestStatus::Completed
            );
            assert!(engine.progress(3_000_021).unwrap().counts.is_empty());
            assert_eq!(
                fixture
                    .load(eligibility(1), Some(engine.state().clone()))
                    .unwrap()
                    .state(),
                engine.state()
            );
        } else {
            assert!(engine.progress(3_000_021).is_none());
        }
        assert_eq!(engine.state().pending_turnin, None);
        assert_eq!(engine.state().selected_quest_id, None);
        assert!(
            engine
                .active_login_packets(&BTreeSet::new())
                .unwrap()
                .is_empty()
        );
        assert!(!engine.begin_turnin(3_000_021, 9_000_000_189).changed);
    }
}

#[test]
fn split_level_and_job_gates_use_supplied_levels_and_recheck_accept() {
    let level = SplitFixture::new(json!({"min_level":5}), true);
    assert_eq!(
        level
            .load(eligibility(4), None)
            .unwrap()
            .offer(3_000_021, 9_000_000_189)
            .denial,
        Some(QuestDenial::Level)
    );
    assert!(
        level
            .load(eligibility(5), None)
            .unwrap()
            .offer(3_000_021, 9_000_000_189)
            .changed
    );
    let job = SplitFixture::new(json!({"min_level":5,"job":43}), true);
    let mut context = eligibility(20);
    context.job_levels.insert("chef".into(), 20);
    assert_eq!(
        job.load(context.clone(), None)
            .unwrap()
            .offer(3_000_021, 9_000_000_189)
            .denial,
        Some(QuestDenial::Job)
    );
    context.job_levels.insert("brawler".into(), 4);
    assert_eq!(
        job.load(context.clone(), None)
            .unwrap()
            .offer(3_000_021, 9_000_000_189)
            .denial,
        Some(QuestDenial::Job)
    );
    context.job_levels.insert("brawler".into(), 5);
    context.player_level = 1; // Other job/avatar level cannot override the required job's level.
    let mut engine = job.load(context, None).unwrap();
    assert!(engine.offer(3_000_021, 9_000_000_189).changed);
    engine.set_eligibility(eligibility(20));
    assert_eq!(engine.accept(3_000_021).denial, Some(QuestDenial::Job));
    let job_only = SplitFixture::new(json!({"job":43}), true);
    let mut context = eligibility(1);
    context.job_levels.insert("brawler".into(), 1);
    assert!(
        job_only
            .load(context, None)
            .unwrap()
            .offer(3_000_021, 9_000_000_189)
            .changed
    );
}

#[test]
fn content_db_uses_the_same_requirement_fields_and_validation() {
    use osfr_tools::content::ContentDb;
    let mut content =
        ContentDb::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content"))
            .unwrap();
    let id = content.quests[0].id;
    let giver = content.quests[0].giver_npc_id;
    content.quests[0].min_level = Some(5);
    content.quests[0].job = Some(35);
    let mut context = eligibility(20);
    context.job_levels.insert("archer".into(), 4);
    assert_eq!(
        QuestEngine::from_content_db(&content, 1, context.clone(), None)
            .unwrap()
            .offer(id, giver)
            .denial,
        Some(QuestDenial::Job)
    );
    context.job_levels.insert("archer".into(), 5);
    assert!(
        QuestEngine::from_content_db(&content, 1, context.clone(), None)
            .unwrap()
            .offer(id, giver)
            .changed
    );
    content.quests[0].min_level = Some(0);
    assert!(QuestEngine::from_content_db(&content, 1, context, None).is_err());
}

#[test]
fn invalid_authored_requirement_fields_fail_closed() {
    for fields in [
        json!({"min_level":0}),
        json!({"min_level":21}),
        json!({"min_level":true}),
        json!({"min_level":1.5}),
        json!({"job":0}),
        json!({"job":-1}),
        json!({"job":999999}),
        json!({"job":"brawler"}),
        json!({"job":true}),
    ] {
        assert!(
            SplitFixture::new(fields.clone(), true)
                .load(eligibility(20), None)
                .is_err(),
            "{fields}"
        );
    }
}

#[test]
#[ignore = "requires private round70 saves; copies inputs into W114 scratch storage"]
fn round70_saves_load_against_current_repository_content() {
    use osfr_tools::{game_store, gameplay::Gameplay};
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let package = manifest.join("../../../lab/reports/round70-native-20261016T061000Z");
    let scratch = manifest
        .join("../../../lab/build/w114/round70-saves")
        .join(std::process::id().to_string());
    fs::create_dir_all(&scratch).unwrap();
    let mut inputs = vec![package.join("character.json")];
    inputs.extend(
        fs::read_dir(package.join("characters"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "json")),
    );
    let mut checked = 0;
    for (index, input) in inputs.iter().enumerate() {
        let bytes = fs::read(input).unwrap();
        let target = scratch.join(format!("save-{index}.json"));
        fs::write(&target, &bytes).unwrap();
        let document: Value = serde_json::from_slice(&bytes).unwrap();
        if document.get("character").is_none() {
            assert!(
                document.get("profile").is_some(),
                "unrecognized private save sidecar"
            );
            assert_eq!(fs::read(input).unwrap(), bytes);
            continue;
        }
        let character = serde_json::from_value(document["character"].clone()).unwrap();
        let snapshot = game_store::load_or_create(&target, character).unwrap();
        let mut gameplay = Gameplay::load(&package.join("content"), snapshot.gameplay).unwrap();
        gameplay
            .bind_npc_quest_catalogue(&manifest.join("data/content"), snapshot.character.guid)
            .unwrap();
        assert_eq!(fs::read(input).unwrap(), bytes);
        checked += 1;
    }
    assert!(checked > 0);
    println!(
        "PASS round70: {checked} copied saves loaded with current repository quests; source bytes unchanged"
    );
}
