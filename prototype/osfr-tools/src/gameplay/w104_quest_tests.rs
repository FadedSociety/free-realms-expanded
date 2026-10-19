use super::*;
use crate::{
    cooking::CookingOutcome,
    forging::ForgingOutcome,
    quest_engine::{
        CharacterEligibility, ObjectiveKind, QuestDefinition, QuestEngine, QuestObjective,
        QuestRequirements,
    },
    rewards::{RewardDisplay, RewardGrant},
    smelting::SmeltingOutcome,
};

fn content() -> PathBuf {
    std::env::var_os("W104_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/build/w104/scratch/content")
        })
}

fn game(kind: ObjectiveKind, required: u32) -> Gameplay {
    let mut game = Gameplay::load(&content(), None).unwrap();
    let definition = QuestDefinition {
        id: 1,
        title: "Synthetic hook (R)".into(),
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
            kind,
        }],
        reward: RewardGrant {
            coins: 0,
            items: vec![],
            job_xp: Default::default(),
            display: RewardDisplay {
                source_guid: 1,
                player_guid: 1,
                reward_kind: 0,
                icon_id: 0,
                name_id: 0,
                rows: vec![],
            },
        },
    };
    let mut inactive = definition.clone();
    inactive.id = 2;
    let mut engine = QuestEngine::new(
        1,
        vec![definition, inactive],
        CharacterEligibility {
            member: true,
            member_quest_limit: 30,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(engine.offer(1, 1).changed);
    assert!(engine.accept(1).changed);
    game.npc_quests = Some(engine);
    game
}

fn count(game: &Gameplay, expected: u32) {
    let engine = game.npc_quests.as_ref().unwrap();
    assert_eq!(engine.progress(1).unwrap().counts, [expected]);
    assert!(engine.progress(2).is_none());
}

fn stage(
    game: &mut Gameplay,
    stage_id: u32,
    profile: i32,
    trigger: OpeningEvent,
) -> CharacterState {
    let mut state = game.opening.as_ref().unwrap().state().clone();
    for definition in game.opening.as_ref().unwrap().stages() {
        let progress = state.stages.get_mut(&definition.id).unwrap();
        if definition.id < stage_id {
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = definition.objectives.iter().map(|g| g.required).collect();
        } else if definition.id == stage_id {
            progress.accepted = true;
            progress.counts = definition.objectives.iter().map(|g| g.required).collect();
            *progress.counts.last_mut().unwrap() = 0;
        }
    }
    // Memory-only fixture; no campaign content or user save changes.
    game.opening = Some(OpeningEngine::load(&content(), Some(state)).unwrap());
    let profiles = game.profiles.as_mut().unwrap();
    profiles.unlock(profile).unwrap();
    profiles.select(profile, &game.inventory).unwrap();
    let target = game
        .opening
        .as_ref()
        .unwrap()
        .actors()
        .iter()
        .find(|a| match (&a.action, &trigger) {
            (
                Some(crate::opening::OpeningTrigger::Craft { recipe_id: a }),
                OpeningEvent::Craft { recipe_id: b, .. },
            ) => a == b,
            _ => false,
        })
        .unwrap();
    CharacterState {
        guid: 1,
        zone_id: target.zone_id,
        position: target.position,
        rotation: [0., 0., 1.],
    }
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn collect_hook_grant_failure_and_inactive_quest() {
    let mut game = game(ObjectiveKind::CollectItem { item_id: 1195 }, 2);
    let mut out = Outcome::default();
    assert!(game.grant_reward(0, &[(i32::MAX, 1)], &mut out).is_err());
    count(&game, 0);
    game.grant_reward(0, &[(1195, 1)], &mut out).unwrap();
    count(&game, 1);
    game.grant_reward(0, &[(1195, 5)], &mut out).unwrap();
    count(&game, 2);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn equip_hook_accepts_owned_slot_and_ignores_replay() {
    let mut game = game(
        ObjectiveKind::EquipItem {
            definition_id: 68878,
        },
        1,
    );
    let mut out = Outcome::default();
    game.grant_reward(0, &[(68878, 1)], &mut out).unwrap();
    let item = game
        .inventory
        .state()
        .items
        .iter()
        .filter(|i| i.definition_id == 68878)
        .max_by_key(|i| i.id)
        .unwrap()
        .id;
    let mut packet = vec![42, 0, 3, 0];
    packet.extend(item.to_le_bytes());
    packet.extend(1i32.to_le_bytes());
    packet.extend(4i32.to_le_bytes());
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    };
    game.handle_in_world(&packet, &character, 1_000).unwrap();
    count(&game, 1);
    game.handle_in_world(&packet, &character, 1_001).unwrap();
    count(&game, 1);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn gathering_payout_hook_and_minigame_hook() {
    let mut game = game(ObjectiveKind::GatherResource { resource_id: 2884 }, 2);
    game.profiles.as_mut().unwrap().unlock(45).unwrap();
    game.profiles
        .as_mut()
        .unwrap()
        .select(45, &game.inventory)
        .unwrap();
    let mut out = Outcome::default();
    game.apply_type8(Gather::Harvesting, Type8Outcome::default(), &mut out)
        .unwrap();
    count(&game, 0);
    let mut win = mining::payload_packet(111_000_001, "OnGameWonMsg").unwrap();
    win[11..15].copy_from_slice(&111i32.to_le_bytes());
    game.apply_type8(
        Gather::Harvesting,
        Type8Outcome {
            transaction: Some(crate::mining::Type8Transaction {
                consumes: vec![],
                grants: vec![(2884, 2)],
            }),
            completed: true,
            replies: vec![win.clone()],
            ..Default::default()
        },
        &mut out,
    )
    .unwrap();
    count(&game, 2);
    let mut minigame = self::game(ObjectiveKind::FinishMinigame { minigame_id: 111 }, 1);
    minigame.profiles.as_mut().unwrap().unlock(45).unwrap();
    minigame
        .profiles
        .as_mut()
        .unwrap()
        .select(45, &minigame.inventory)
        .unwrap();
    // A server result without a payout never emits a completion event.
    minigame
        .apply_type8(
            Gather::Harvesting,
            Type8Outcome {
                replies: vec![win.clone()],
                ..Default::default()
            },
            &mut out,
        )
        .unwrap();
    count(&minigame, 0);
    minigame
        .apply_type8(
            Gather::Harvesting,
            Type8Outcome {
                transaction: Some(crate::mining::Type8Transaction {
                    consumes: vec![],
                    grants: vec![(2884, 1)],
                }),
                completed: true,
                replies: vec![win],
                ..Default::default()
            },
            &mut out,
        )
        .unwrap();
    count(&minigame, 1);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn gather_node_hook_checks_accepted_progress() {
    let mut game = game(ObjectiveKind::GatherResource { resource_id: 1001 }, 2);
    stage(
        &mut game,
        910004,
        45,
        OpeningEvent::Craft {
            recipe_id: 2001,
            quantity: 1,
        },
    );
    let opening = game.opening.as_ref().unwrap();
    let definition = opening.active_stage().unwrap();
    let index = definition
        .objectives
        .iter()
        .position(|g| {
            matches!(
                g.trigger,
                crate::opening::OpeningTrigger::Gather { resource_id: 1001 }
            )
        })
        .unwrap();
    let mut state = opening.state().clone();
    state.stages.get_mut(&definition.id).unwrap().counts = definition
        .objectives
        .iter()
        .enumerate()
        .map(|(i, g)| if i < index { g.required } else { 0 })
        .collect();
    let actor = opening
        .actors()
        .iter()
        .find(|a| {
            matches!(
                a.action,
                Some(crate::opening::OpeningTrigger::Gather { resource_id: 1001 })
            )
        })
        .unwrap()
        .clone();
    game.opening = Some(OpeningEngine::load(&content(), Some(state)).unwrap());
    let mut character = CharacterState {
        guid: 1,
        zone_id: actor.zone_id,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    };
    game.opening
        .as_mut()
        .unwrap()
        .on_event(OpeningEvent::ClassSelected { profile_id: 45 })
        .unwrap();
    game.on_location(&character, 999).unwrap();
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(actor.guid.to_le_bytes());
    packet.extend(1i32.to_le_bytes());
    game.handle_in_world(&packet, &character, 1_000).unwrap();
    count(&game, 0);
    character.position = actor.position;
    game.on_location(&character, 1_001).unwrap();
    game.handle_in_world(&packet, &character, 1_001).unwrap();
    count(&game, 1);
    game.handle_in_world(&packet, &character, 1_002).unwrap();
    count(&game, 2);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn cooking_craft_hook() {
    let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 70 }, 1);
    let character = stage(
        &mut game,
        910004,
        45,
        OpeningEvent::Craft {
            recipe_id: 2001,
            quantity: 1,
        },
    );
    let mut out = Outcome::default();
    game.apply_craft(
        crate::recipes::CraftingJob::Chef,
        CookingOutcome::default(),
        &character,
        &mut out,
    )
    .unwrap();
    count(&game, 0);
    game.apply_craft(
        crate::recipes::CraftingJob::Chef,
        CookingOutcome {
            transaction: Some(crate::cooking::CookingTransaction {
                consumes: vec![],
                grants: vec![(2884, 1)],
                experience_points: 0,
            }),
            completed: true,
            ..Default::default()
        },
        &character,
        &mut out,
    )
    .unwrap();
    count(&game, 1);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn smelting_craft_hook() {
    let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 2002 }, 1);
    let character = stage(
        &mut game,
        910005,
        14,
        OpeningEvent::Craft {
            recipe_id: 2002,
            quantity: 1,
        },
    );
    let mut out = Outcome::default();
    game.apply_craft(
        crate::recipes::CraftingJob::Miner,
        SmeltingOutcome {
            transaction: Some(crate::smelting::SmeltingTransaction {
                consumes: vec![],
                grants: vec![(21785, 1)],
                experience_points: 0,
            }),
            completed: true,
            ..Default::default()
        },
        &character,
        &mut out,
    )
    .unwrap();
    count(&game, 1);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn forging_craft_hook() {
    let mut game = game(ObjectiveKind::CraftRecipe { recipe_id: 11 }, 1);
    let character = stage(
        &mut game,
        910006,
        16,
        OpeningEvent::Craft {
            recipe_id: 2003,
            quantity: 1,
        },
    );
    let mut out = Outcome::default();
    game.apply_craft(
        crate::recipes::CraftingJob::Blacksmith,
        ForgingOutcome {
            transaction: Some(crate::forging::ForgingTransaction {
                consumes: vec![],
                grants: vec![(1195, 1)],
                experience_points: 0,
            }),
            completed: true,
            ..Default::default()
        },
        &character,
        &mut out,
    )
    .unwrap();
    count(&game, 1);
}

#[test]
#[ignore = "private W104 scratch package; no native client"]
fn movement_hook_area_and_radius() {
    for kind in [
        ObjectiveKind::EnterArea { area_id: 0 },
        ObjectiveKind::VisitPosition {
            zone_id: 0,
            position: [0.; 3],
            radius: 1.,
        },
    ] {
        let mut game = game(kind, 1);
        let mut character = CharacterState {
            guid: 1,
            zone_id: 1,
            position: [0.; 3],
            rotation: [0., 0., 1.],
        };
        game.on_location(&character, 1_000).unwrap();
        count(&game, 0);
        character.zone_id = 0;
        game.on_location(&character, 1_001).unwrap();
        count(&game, 1);
    }
}
