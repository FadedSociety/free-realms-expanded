use osfr_tools::opening::{OpeningEngine, OpeningEvent, OpeningState, OpeningTrigger};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Reference {
    directory: PathBuf,
    content: Value,
}

impl Reference {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-opening-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let actor = |guid, position, action: Value, profile: Value| json!({"guid":guid,"definition_id":20,"name_id":0,"name":"Synthetic guide","model_id":1,"model_file_name":"synthetic.adr","zone_id":5,"position":position,"heading":0.0,"interaction_radius":12.0,"kind":"guide","action":action,"required_profile":profile,"provenance":"synthetic"});
        let goal = |id, required, trigger: Value| json!({"id":id,"name_id":0,"description_id":0,"description":"Synthetic objective","required":required,"trigger":trigger,"recovery":"synthetic"});
        let content = json!({"schema_version":1,"scope":"synthetic","reconstruction_notice":"synthetic fixture","initial_profiles":[1,43],
          "actors":[actor(1,[0.0,0.0,0.0],Value::Null,Value::Null),actor(2,[5.0,0.0,0.0],json!({"kind":"gather","resource_id":7}),json!(45)),actor(3,[20.0,0.0,0.0],Value::Null,Value::Null),actor(4,[2.0,0.0,0.0],json!({"kind":"craft","recipe_id":9}),json!(45))],
          "discoveries":[{"id":1,"name_id":0,"name":"Synthetic location","zone_id":5,"position":[0.0,0.0,0.0],"radius":1.0,"reward":{"stage_id":99,"coins":5,"items":[]},"provenance":"synthetic"}],
          "stages":[
           {"id":10,"original_quest_id":null,"title_id":0,"description_id":0,"title":"Synthetic preparation","description":"Synthetic sequence","giver_guid":1,"turnin_guid":1,"ordered":true,
            "objectives":[goal(100,1,json!({"kind":"class_selected","profile_id":45})),goal(101,2,json!({"kind":"gather","resource_id":7})),goal(102,1,json!({"kind":"craft","recipe_id":9})),goal(103,1,json!({"kind":"cosmetic_equipped","definition_id":44})),goal(104,2,json!({"kind":"defeat","definition_id":9})),goal(105,1,json!({"kind":"visit","zone_id":5,"position":[20.0,0.0,0.0],"radius":2.0}))],
            "reward":{"stage_id":10,"coins":25,"items":[[44,1]]},"unlock_profiles":[45],"provenance":"synthetic"},
           {"id":11,"original_quest_id":null,"title_id":0,"description_id":0,"title":"Synthetic dungeon","description":"Synthetic ending","giver_guid":3,"turnin_guid":3,"ordered":true,"objectives":[goal(110,1,json!({"kind":"dungeon_completed","encounter_id":411})),goal(111,1,json!({"kind":"talk","actor_guid":3}))],"reward":{"stage_id":11,"coins":10,"items":[]},"unlock_profiles":[],"provenance":"synthetic"}],"provenance":{}});
        let value = Self { directory, content };
        value.write();
        value
    }
    fn write(&self) {
        fs::write(
            self.directory.join("opening.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
    fn engine(&self) -> OpeningEngine {
        OpeningEngine::load(&self.directory, None).unwrap()
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        if self.directory.parent() == Some(std::env::temp_dir().as_path()) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

fn accept(engine: &mut OpeningEngine) {
    engine.on_position(5, [0.0, 0.0, 0.0]).unwrap();
    let accepted = engine.on_interaction(1).unwrap();
    assert_eq!(accepted.unlock_profiles, vec![45]);
    assert!(accepted.rewards.is_empty());
}

#[test]
fn ordered_progress_survives_reload_and_turnin_rewards_only_once() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    assert!(
        !engine
            .on_event(OpeningEvent::Defeat { definition_id: 9 })
            .unwrap()
            .changed
    );
    accept(&mut engine);
    assert!(
        !engine
            .on_event(OpeningEvent::Gather {
                resource_id: 7,
                quantity: 2
            })
            .unwrap()
            .changed
    );
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    engine.on_interaction(2).unwrap();
    let state: OpeningState =
        serde_json::from_slice(&serde_json::to_vec(engine.state()).unwrap()).unwrap();
    let mut engine = OpeningEngine::load(&reference.directory, Some(state)).unwrap();
    assert!(!engine.on_interaction(1).unwrap().changed);
    engine.on_position(5, [0.0, 0.0, 0.0]).unwrap();
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    engine.on_interaction(2).unwrap();
    engine.on_interaction(4).unwrap();
    engine
        .on_event(OpeningEvent::CosmeticEquipped { definition_id: 44 })
        .unwrap();
    engine
        .on_event(OpeningEvent::Defeat { definition_id: 9 })
        .unwrap();
    engine
        .on_event(OpeningEvent::Defeat { definition_id: 9 })
        .unwrap();
    engine.on_position(5, [20.0, 0.0, 0.0]).unwrap();
    assert!(!engine.handle_command("opening turnin").changed);
    engine.on_position(5, [0.0, 0.0, 0.0]).unwrap();
    let result = engine.on_interaction(1).unwrap();
    assert_eq!(result.rewards.len(), 1);
    assert_eq!(result.rewards[0].coins, 25);
    assert_eq!(result.rewards[0].items, vec![(44, 1)]);
    assert_eq!(engine.active_stage().unwrap().id, 11);
    let state = engine.state().clone();
    let mut engine = OpeningEngine::load(&reference.directory, Some(state)).unwrap();
    engine.on_position(5, [0.0, 0.0, 0.0]).unwrap();
    assert!(engine.on_interaction(1).unwrap().rewards.is_empty());
    engine.on_position(5, [20.0, 0.0, 0.0]).unwrap();
    engine.on_interaction(3).unwrap();
    assert!(
        !engine
            .on_event(OpeningEvent::DungeonCompleted { encounter_id: 412 })
            .unwrap()
            .changed
    );
    engine
        .on_event(OpeningEvent::DungeonCompleted { encounter_id: 411 })
        .unwrap();
    let result = engine.on_interaction(3).unwrap();
    assert_eq!(result.rewards[0].coins, 10);
    assert!(result.completed_opening);
    assert!(engine.complete());
    assert!(engine.on_interaction(3).unwrap().rewards.is_empty());
}

#[test]
fn gathering_requires_correct_job_zone_and_proximity() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    accept(&mut engine);
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    engine.on_position(6, [0.0, 0.0, 0.0]).unwrap();
    assert!(
        !engine
            .on_event(OpeningEvent::Gather {
                resource_id: 7,
                quantity: 1
            })
            .unwrap()
            .changed
    );
    engine.on_position(5, [100.0, 0.0, 0.0]).unwrap();
    assert!(!engine.on_interaction(2).unwrap().changed);
    engine.on_position(5, [0.0, 0.0, 0.0]).unwrap();
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 43 })
        .unwrap();
    assert!(!engine.on_interaction(2).unwrap().changed);
    assert!(
        !engine
            .on_event(OpeningEvent::Gather {
                resource_id: 7,
                quantity: 1
            })
            .unwrap()
            .changed
    );
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    assert!(engine.on_interaction(2).unwrap().changed);
    assert_eq!(engine.state().stages[&10].counts, vec![1, 1, 0, 0, 0, 0]);
}

#[test]
fn discoveries_have_zone_identity_and_persistent_reward_once() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    assert!(
        engine
            .on_position(6, [0.0, 0.0, 0.0])
            .unwrap()
            .rewards
            .is_empty()
    );
    assert_eq!(
        engine.on_position(5, [0.0, 0.0, 0.0]).unwrap().rewards[0].coins,
        5
    );
    assert!(
        engine
            .on_position(5, [0.0, 0.0, 0.0])
            .unwrap()
            .rewards
            .is_empty()
    );
    let mut engine =
        OpeningEngine::load(&reference.directory, Some(engine.state().clone())).unwrap();
    assert!(
        engine
            .on_position(5, [0.0, 0.0, 0.0])
            .unwrap()
            .rewards
            .is_empty()
    );
    assert!(!engine.complete());
}

#[test]
fn chat_cannot_complete_skip_or_accept_a_remote_step() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    let before = engine.state().clone();
    for command in [
        "opening complete10",
        "opening accept",
        "opening turnin",
        "opening accept999999",
    ] {
        assert!(!engine.handle_command(command).changed);
    }
    assert_eq!(engine.state(), &before);
    accept(&mut engine);
    let before = engine.state().clone();
    assert!(!engine.handle_command("opening accept").changed);
    assert!(!engine.handle_command("opening complete").changed);
    assert_eq!(engine.state(), &before);
}

#[test]
fn invalid_events_reject_before_mutating_persisted_state() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    accept(&mut engine);
    let before = engine.state().clone();
    for event in [
        OpeningEvent::Gather {
            resource_id: 7,
            quantity: 0,
        },
        OpeningEvent::Gather {
            resource_id: 7,
            quantity: u32::MAX,
        },
        OpeningEvent::Craft {
            recipe_id: 0,
            quantity: 1,
        },
        OpeningEvent::ClassSelected { profile_id: -1 },
        OpeningEvent::Visit {
            zone_id: 5,
            position: [f32::NAN, 0.0, 0.0],
        },
    ] {
        assert!(engine.on_event(event).is_err());
        assert_eq!(engine.state(), &before);
    }
}

#[test]
fn corrupted_future_progress_reward_flags_and_content_change_are_rejected() {
    let mut reference = Reference::new();
    let engine = reference.engine();
    let mut state = engine.state().clone();
    state.stages.get_mut(&11).unwrap().accepted = true;
    assert!(OpeningEngine::load(&reference.directory, Some(state)).is_err());
    let mut state = engine.state().clone();
    state.stages.get_mut(&10).unwrap().rewarded = true;
    assert!(OpeningEngine::load(&reference.directory, Some(state)).is_err());
    let mut state = engine.state().clone();
    state.stages.get_mut(&10).unwrap().counts[0] = 2;
    assert!(OpeningEngine::load(&reference.directory, Some(state)).is_err());
    reference.content["scope"] = json!("changed");
    reference.write();
    assert!(OpeningEngine::load(&reference.directory, Some(engine.state().clone())).is_err());
}

#[test]
fn journal_exposes_only_saved_counters_and_session_reset_removes_remote_authority() {
    let reference = Reference::new();
    let mut engine = reference.engine();
    accept(&mut engine);
    engine
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    engine.on_interaction(2).unwrap();
    let journal = engine.journal_entries();
    assert!(journal[0].accepted);
    assert!(!journal[0].completed);
    assert_eq!(journal[0].objectives[1].count, 1);
    assert!(!journal[1].accepted);
    engine.reset_session();
    assert!(!engine.on_interaction(2).unwrap().changed);
    assert!(
        !engine
            .on_event(OpeningEvent::Gather {
                resource_id: 7,
                quantity: 1
            })
            .unwrap()
            .changed
    );
    assert_eq!(engine.journal_entries()[0].objectives[1].count, 1);
}

#[test]
#[ignore = "requires private recovered opening content"]
fn private_opening_content_loads_with_all_objectives_and_verified_actor_references() {
    let directory = PathBuf::from(
        std::env::var_os("OSFR_OPENING_REFERENCE").expect("OSFR_OPENING_REFERENCE is required"),
    );
    let engine = OpeningEngine::load(&directory, None).unwrap();
    assert_eq!(engine.journal_entries().len(), 17);
    assert_eq!(
        engine
            .journal_entries()
            .iter()
            .map(|q| q.objectives.len())
            .sum::<usize>(),
        51
    );
    assert_eq!(engine.actors().len(), 27);
    assert_eq!(engine.discoveries().len(), 4);
    assert_eq!(engine.initial_profiles(), &[1, 43]);
    let content: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("opening.json")).unwrap()).unwrap();
    let stages = content["stages"].as_array().unwrap();
    let weapon = stages.iter().find(|s| s["id"] == 910006).unwrap();
    let wolves = stages.iter().find(|s| s["id"] == 910007).unwrap();
    assert_eq!(weapon["reward"]["coins"], 16);
    assert_eq!(weapon["reward"]["items"], serde_json::json!([[75030, 1]]));
    assert_eq!(wolves["title"], "Fighting off the Pack");
    assert_eq!(wolves["reward"]["coins"], 18);
    assert_eq!(wolves["objectives"][2]["required"], 5);
    assert_eq!(
        content["provenance"]["wiki_farnum"]["original_material_grant"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["quantity"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![10, 5, 5, 5]
    );
    let encounters: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("encounters.json")).unwrap()).unwrap();
    let wolves: Vec<_> = encounters["world_enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["definition_id"] == 3156)
        .collect();
    assert_eq!(wolves.len(), 5);
    assert_eq!(
        wolves
            .iter()
            .map(|e| e["guid"].as_u64().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );
}

#[test]
#[ignore = "requires private recovered opening content; synthetic event traversal is not native gameplay proof"]
fn private_opening_every_stage_is_completable_through_typed_events_and_reward_once_after_reload() {
    let directory = PathBuf::from(
        std::env::var_os("OSFR_OPENING_REFERENCE").expect("OSFR_OPENING_REFERENCE is required"),
    );
    let mut engine = OpeningEngine::load(&directory, None).unwrap();
    let mut rewarded = std::collections::BTreeSet::new();
    for _ in 0..17 {
        let stage = engine.active_stage().unwrap().clone();
        let guide = engine
            .actors()
            .iter()
            .find(|a| a.guid == stage.giver_guid)
            .unwrap()
            .clone();
        engine.on_position(guide.zone_id, guide.position).unwrap();
        assert!(engine.on_interaction(guide.guid).unwrap().changed);
        for goal in &stage.objectives {
            for _ in 0..goal.required {
                if engine.state().stages[&stage.id].counts[stage
                    .objectives
                    .iter()
                    .position(|g| g.id == goal.id)
                    .unwrap()]
                    >= goal.required
                {
                    break;
                }
                let event = match goal.trigger {
                    OpeningTrigger::Talk { actor_guid }
                    | OpeningTrigger::Interact { actor_guid } => {
                        let actor = engine
                            .actors()
                            .iter()
                            .find(|a| a.guid == actor_guid)
                            .unwrap()
                            .clone();
                        engine.on_position(actor.zone_id, actor.position).unwrap();
                        if let Some(profile_id) = actor.required_profile {
                            engine
                                .on_event(OpeningEvent::ClassSelected { profile_id })
                                .unwrap();
                        }
                        match goal.trigger {
                            OpeningTrigger::Talk { .. } => OpeningEvent::Talk {
                                actor_id: actor_guid,
                            },
                            _ => OpeningEvent::Interact {
                                actor_id: actor_guid,
                            },
                        }
                    }
                    OpeningTrigger::Gather { resource_id }
                    | OpeningTrigger::Craft {
                        recipe_id: resource_id,
                    } => {
                        let actor = engine.actors().iter().find(|a|matches!((&a.action,&goal.trigger),(Some(OpeningTrigger::Gather { resource_id:a }),OpeningTrigger::Gather { resource_id:b })|(Some(OpeningTrigger::Craft { recipe_id:a }),OpeningTrigger::Craft { recipe_id:b }) if a==b)).unwrap().clone();
                        engine.on_position(actor.zone_id, actor.position).unwrap();
                        if let Some(profile_id) = actor.required_profile {
                            engine
                                .on_event(OpeningEvent::ClassSelected { profile_id })
                                .unwrap();
                        }
                        match goal.trigger {
                            OpeningTrigger::Gather { .. } => OpeningEvent::Gather {
                                resource_id,
                                quantity: 1,
                            },
                            _ => OpeningEvent::Craft {
                                recipe_id: resource_id,
                                quantity: 1,
                            },
                        }
                    }
                    OpeningTrigger::Visit {
                        zone_id, position, ..
                    } => OpeningEvent::Visit { zone_id, position },
                    OpeningTrigger::Defeat { definition_id } => {
                        OpeningEvent::Defeat { definition_id }
                    }
                    OpeningTrigger::ClassSelected { profile_id } => {
                        OpeningEvent::ClassSelected { profile_id }
                    }
                    OpeningTrigger::CosmeticEquipped { definition_id } => {
                        OpeningEvent::CosmeticEquipped { definition_id }
                    }
                    OpeningTrigger::DungeonCompleted { encounter_id } => {
                        OpeningEvent::DungeonCompleted { encounter_id }
                    }
                    OpeningTrigger::AbilityUsed { ability_id } => {
                        OpeningEvent::AbilityUsed { ability_id }
                    }
                };
                for reward in engine.on_event(event).unwrap().rewards {
                    assert!(rewarded.insert(reward.stage_id));
                }
            }
        }
        if !engine.state().stages[&stage.id].completed {
            let actor = engine
                .actors()
                .iter()
                .find(|a| a.guid == stage.turnin_guid)
                .unwrap()
                .clone();
            engine.on_position(actor.zone_id, actor.position).unwrap();
            for reward in engine.on_interaction(actor.guid).unwrap().rewards {
                assert!(rewarded.insert(reward.stage_id));
            }
        }
        assert!(
            engine.state().stages[&stage.id].completed,
            "stage{} failed",
            stage.id
        );
        engine = OpeningEngine::load(&directory, Some(engine.state().clone())).unwrap();
    }
    for discovery in engine.discoveries().to_vec() {
        engine
            .on_position(discovery.zone_id, discovery.position)
            .unwrap();
    }
    assert!(engine.complete());
    assert_eq!(
        engine
            .state()
            .stages
            .values()
            .filter(|p| p.completed && p.rewarded)
            .count(),
        17
    );
    let mut reloaded = OpeningEngine::load(&directory, Some(engine.state().clone())).unwrap();
    assert!(reloaded.handle_command("opening turnin").rewards.is_empty());
}
