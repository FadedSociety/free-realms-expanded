use osfr_tools::quest_engine::{
    CharacterEligibility, ObjectiveKind, OutboundMessage, QuestDefinition, QuestEngine, QuestEvent,
    QuestMessage, QuestObjective, QuestRequirements,
};
use osfr_tools::rewards::{RewardDisplay, RewardGrant};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

fn engine(kind: ObjectiveKind, required: u32) -> QuestEngine {
    let definitions = (1..=3)
        .map(|id| QuestDefinition {
            id,
            title: "Synthetic objective (R)".into(),
            description: String::new(),
            giver_npc_id: 1,
            turnin_npc_id: 1,
            prerequisite_quest_ids: vec![],
            requirements: QuestRequirements::default(),
            ordered: true,
            objectives: vec![QuestObjective {
                id: 1,
                required,
                description: "Synthetic (R)".into(),
                kind: kind.clone(),
            }],
            reward: RewardGrant {
                coins: 0,
                items: vec![],
                job_xp: BTreeMap::new(),
                sc: 0,
                display: RewardDisplay {
                    source_guid: 1,
                    player_guid: 1,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: vec![],
                },
            },
        })
        .collect();
    let mut engine = QuestEngine::new(
        1,
        definitions,
        CharacterEligibility {
            member: true,
            member_quest_limit: 10,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(engine.offer(1, 1).changed);
    assert!(engine.accept(1).changed);
    assert!(engine.offer(2, 1).changed); // Offered, never active.
    engine
}

fn advances(kind: ObjectiveKind, wrong: QuestEvent, right: QuestEvent) {
    let mut engine = engine(kind, 2);
    assert!(!engine.on_event(wrong).unwrap().changed);
    assert!(engine.on_event(right.clone()).unwrap().changed);
    assert_eq!(engine.progress(1).unwrap().counts, [1]);
    let done = engine.on_event(right.clone()).unwrap();
    assert!(done.outbound.iter().any(|m| matches!(
        m,
        OutboundMessage::Quest(QuestMessage::ObjectiveComplete { quest_id: 1, .. })
    )));
    assert_eq!(engine.progress(1).unwrap().counts, [2]);
    assert!(!engine.on_event(right).unwrap().changed);
    assert_eq!(engine.progress(2).unwrap().counts, [0]);
    assert!(engine.progress(3).is_none());
}

#[test]
fn collect() {
    advances(
        ObjectiveKind::CollectItem { item_id: 8 },
        QuestEvent::CollectItem {
            item_id: 9,
            quantity: 1,
        },
        QuestEvent::CollectItem {
            item_id: 8,
            quantity: 1,
        },
    );
}
#[test]
fn equip() {
    advances(
        ObjectiveKind::EquipItem { definition_id: 8 },
        QuestEvent::EquipItem { definition_id: 9 },
        QuestEvent::EquipItem { definition_id: 8 },
    );
}
#[test]
fn gather() {
    advances(
        ObjectiveKind::GatherResource { resource_id: 8 },
        QuestEvent::GatherResource {
            resource_id: 9,
            quantity: 1,
        },
        QuestEvent::GatherResource {
            resource_id: 8,
            quantity: 1,
        },
    );
}
#[test]
fn craft() {
    advances(
        ObjectiveKind::CraftRecipe { recipe_id: 8 },
        QuestEvent::CraftRecipe {
            recipe_id: 9,
            quantity: 1,
        },
        QuestEvent::CraftRecipe {
            recipe_id: 8,
            quantity: 1,
        },
    );
}
#[test]
fn minigame() {
    advances(
        ObjectiveKind::FinishMinigame { minigame_id: 8 },
        QuestEvent::FinishMinigame { minigame_id: 9 },
        QuestEvent::FinishMinigame { minigame_id: 8 },
    );
}
#[test]
fn area() {
    advances(
        ObjectiveKind::EnterArea { area_id: 0 },
        QuestEvent::EnterArea { area_id: 1 },
        QuestEvent::EnterArea { area_id: 0 },
    );
}
#[test]
fn reach() {
    advances(
        ObjectiveKind::ReachPoi { poi_id: 8 },
        QuestEvent::ReachPoi { poi_id: 9 },
        QuestEvent::ReachPoi { poi_id: 8 },
    );
}
#[test]
fn position_radius_and_zone() {
    advances(
        ObjectiveKind::VisitPosition {
            zone_id: 0,
            position: [1., 0., 0.],
            radius: 1.,
        },
        QuestEvent::VisitPosition {
            zone_id: 1,
            position: [1., 0., 0.],
        },
        QuestEvent::VisitPosition {
            zone_id: 0,
            position: [0., 0., 0.],
        },
    );
    let mut engine = engine(
        ObjectiveKind::VisitPosition {
            zone_id: 0,
            position: [0.; 3],
            radius: 1.,
        },
        1,
    );
    assert!(
        !engine
            .on_event(QuestEvent::VisitPosition {
                zone_id: 0,
                position: [1.01, 0., 0.]
            })
            .unwrap()
            .changed
    );
    assert!(
        engine
            .on_event(QuestEvent::VisitPosition {
                zone_id: 0,
                position: [f32::NAN, 0., 0.]
            })
            .is_err()
    );
    assert_eq!(engine.progress(1).unwrap().counts, [0]);
}

#[test]
fn no_offered_content_quest_has_zero_objectives() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    for path in fs::read_dir(content.join("quests")).unwrap() {
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(path.unwrap().path()).unwrap()).unwrap();
        for quest in value["records"].as_array().unwrap() {
            if quest["enabled"].as_bool().unwrap_or(true) {
                assert!(
                    !quest["objectives"].as_array().unwrap().is_empty(),
                    "hollow quest {}",
                    quest["id"]
                );
            }
        }
    }
}

/// Per-run split-content copy; removed on drop, including while a test unwinds.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        Self(PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
    }
}

impl std::ops::Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn split_fixture(goal: serde_json::Value, enabled: bool) -> Scratch {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let target = Scratch::new();
    fs::create_dir_all(target.join("quests")).unwrap();
    fs::create_dir_all(target.join("npcs")).unwrap();
    fs::copy(content.join("jobs.json"), target.join("jobs.json")).unwrap();
    for folder in ["quests", "npcs"] {
        for entry in fs::read_dir(content.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            if folder == "quests" {
                for quest in value["records"].as_array_mut().unwrap() {
                    if quest["id"] == 3_000_021 {
                        quest["objectives"] = serde_json::json!([goal.clone()]);
                        quest["enabled"] = enabled.into();
                    }
                }
            }
            fs::write(
                target.join(folder).join(path.file_name().unwrap()),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
    }
    target
}

#[test]
fn split_json_kinds_reach_active_engine_and_disabled_flag_blocks_offers() {
    use serde_json::json;
    let pairs = [
        (
            json!({"kind":"collect_item","item_id":8,"quantity":2}),
            QuestEvent::CollectItem {
                item_id: 8,
                quantity: 2,
            },
        ),
        (
            json!({"kind":"gather_resource","resource_id":1001,"quantity":2}),
            QuestEvent::GatherResource {
                resource_id: 1001,
                quantity: 2,
            },
        ),
        (
            json!({"kind":"craft_recipe","recipe_id":2001,"quantity":2}),
            QuestEvent::CraftRecipe {
                recipe_id: 2001,
                quantity: 2,
            },
        ),
        (
            json!({"kind":"finish_minigame","minigame_id":111,"count":1}),
            QuestEvent::FinishMinigame { minigame_id: 111 },
        ),
        (
            json!({"kind":"equip_item","definition_id":8}),
            QuestEvent::EquipItem { definition_id: 8 },
        ),
        (
            json!({"kind":"visit_zone","zone_id":0}),
            QuestEvent::VisitPosition {
                zone_id: 0,
                position: [2., 0., 0.],
            },
        ),
        (
            json!({"kind":"visit_position","zone_id":0,"position":[1.,0.,0.],"radius":1.}),
            QuestEvent::VisitPosition {
                zone_id: 0,
                position: [2., 0., 0.],
            },
        ),
        (
            json!({"kind":"reach_poi","poi_id":7,"zone_id":0,"position":[1.,0.,0.],"radius":1.}),
            QuestEvent::VisitPosition {
                zone_id: 0,
                position: [2., 0., 0.],
            },
        ),
    ];
    for (mut goal, event) in pairs {
        goal["id"] = 1.into();
        goal["source"] =
            json!([{"kind":"reconstructed","evidence":"(R) synthetic quest event test"}]);
        let path = split_fixture(goal.clone(), true);
        let eligibility = CharacterEligibility {
            member: true,
            member_quest_limit: 30,
            ..Default::default()
        };
        let mut engine =
            QuestEngine::from_split_content(&path, 1, eligibility.clone(), None).unwrap();
        assert!(!engine.on_event(event.clone()).unwrap().changed);
        assert!(engine.offer(3_000_021, 9_000_000_189).changed);
        assert!(engine.accept(3_000_021).changed);
        if matches!(event, QuestEvent::VisitPosition { .. }) {
            assert!(
                !engine
                    .on_event(QuestEvent::VisitPosition {
                        zone_id: 1,
                        position: [1., 0., 0.]
                    })
                    .unwrap()
                    .changed
            );
            if goal["kind"] != "visit_zone" {
                assert!(
                    !engine
                        .on_event(QuestEvent::VisitPosition {
                            zone_id: 0,
                            position: [2.01, 0., 0.]
                        })
                        .unwrap()
                        .changed
                );
            }
        }
        assert!(engine.on_event(event.clone()).unwrap().changed, "{goal}");
        assert!(!engine.on_event(event).unwrap().changed);
        let state = engine.state().clone();
        let restored =
            QuestEngine::from_split_content(&path, 1, eligibility.clone(), Some(state.clone()))
                .unwrap();
        assert_eq!(restored.state(), &state);
        let disabled = split_fixture(goal, false);
        let mut engine = QuestEngine::from_split_content(&disabled, 1, eligibility, None).unwrap();
        assert!(!engine.is_enabled(3_000_021));
        assert!(!engine.offer(3_000_021, 9_000_000_189).changed);
        assert!(!engine.dev_mark_offered(3_000_021));
    }
}

#[test]
fn invalid_runtime_json_is_rejected() {
    use serde_json::json;
    for mut goal in [
        json!({"kind":"gather_resource","resource_id":1,"quantity":0}),
        json!({"kind":"craft_recipe","recipe_id":0,"quantity":1}),
        json!({"kind":"finish_minigame","minigame_id":1,"count":1_000_001}),
        json!({"kind":"equip_item","definition_id":-1}),
        json!({"kind":"visit_position","zone_id":0,"position":[0.,0.,0.],"radius":0.}),
        json!({"kind":"reach_poi","poi_id":1,"zone_id":0,"position":[0.,0.,0.],"radius":1001.}),
    ] {
        goal["id"] = 1.into();
        goal["source"] = json!([{"kind":"reconstructed","evidence":"(R) invalid test"}]);
        let path = split_fixture(goal, true);
        assert!(
            QuestEngine::from_split_content(&path, 1, CharacterEligibility::default(), None)
                .is_err()
        );
    }
}

#[test]
fn former_hollow_save_preserves_completion_and_resets_only_unearned_progress() {
    use osfr_tools::quest_engine::QuestStatus;
    use serde_json::json;
    let path = split_fixture(
        json!({"kind":"collect_item","id":1,"item_id":1195,"quantity":2,
        "source":[{"kind":"reconstructed","evidence":"(R) rollout test"}]}),
        true,
    );
    let eligibility = CharacterEligibility {
        member: true,
        member_quest_limit: 30,
        ..Default::default()
    };
    let mut engine = QuestEngine::from_split_content(&path, 1, eligibility.clone(), None).unwrap();
    assert!(engine.offer(3_000_021, 9_000_000_189).changed);
    assert!(engine.accept(3_000_021).changed);
    let mut old = engine.state().clone();
    old.quests.get_mut(&3_000_021).unwrap().counts.clear();
    old.pending_turnin = Some(3_000_021);
    let migrated =
        QuestEngine::from_split_content(&path, 1, eligibility.clone(), Some(old.clone())).unwrap();
    assert_eq!(migrated.progress(3_000_021).unwrap().counts, [0]);
    assert_eq!(migrated.state().pending_turnin, None);
    old.pending_turnin = None;
    old.quests.get_mut(&3_000_021).unwrap().status = QuestStatus::Completed;
    let completed =
        QuestEngine::from_split_content(&path, 1, eligibility.clone(), Some(old.clone())).unwrap();
    assert_eq!(completed.progress(3_000_021).unwrap().counts, [2]);
    assert_eq!(
        completed.progress(3_000_021).unwrap().status,
        QuestStatus::Completed
    );
    old.quests.get_mut(&3_000_021).unwrap().counts = vec![1, 1];
    let repaired = QuestEngine::from_split_content(&path, 1, eligibility, Some(old)).unwrap();
    assert_eq!(repaired.progress(3_000_021).unwrap().counts, [2]);
    assert_eq!(
        repaired.progress(3_000_021).unwrap().status,
        QuestStatus::Completed
    );
}

#[test]
fn disabling_a_pending_offer_clears_it_before_reload_acceptance() {
    use serde_json::json;
    let goal = json!({"kind":"collect_item","id":1,"item_id":1195,"quantity":2,
        "source":[{"kind":"reconstructed","evidence":"(R) disabled offer test"}]});
    let path = split_fixture(goal.clone(), true);
    let eligibility = CharacterEligibility {
        member: true,
        member_quest_limit: 30,
        ..Default::default()
    };
    let mut engine = QuestEngine::from_split_content(&path, 1, eligibility.clone(), None).unwrap();
    assert!(engine.offer(3_000_021, 9_000_000_189).changed);
    let disabled = split_fixture(goal, false);
    let mut restored =
        QuestEngine::from_split_content(&disabled, 1, eligibility, Some(engine.state().clone()))
            .unwrap();
    assert_eq!(restored.state().pending_offer, None);
    assert!(!restored.accept(3_000_021).changed);
    assert!(restored.progress(3_000_021).is_none());
}
