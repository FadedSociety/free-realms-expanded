//! Private fixture checks that need server-internal movie clocks or private
//! appliers. The packet-path characterization (station refusals, the movie
//! gate, completion errors, gathering bonuses) lives in
//! `tests/s6_characterization.rs` and reuses the shared movie drivers.
//!
//! Wire prefixes here are the shipped client's request shapes: 26/10
//! constructor 0x00a36f80 and 26/8 builder 0x008b6bc0 for the NPC ordering
//! check; the chat request reader is 0x00b11600.

use super::*;
use crate::{
    account_store::AccountRole,
    recipes::CraftingJob,
    smelting::{SmeltingOutcome, SmeltingTransaction},
};
use serde_json::Value;
use std::{fs, path::PathBuf};

const NOW: u64 = 1_000;

#[derive(Clone, Copy)]
struct Station {
    content: &'static str,
    profile: i32,
    recipe: u32,
}

const FORGING: Station = Station {
    content: "rust-opening-forging",
    profile: 16,
    recipe: 2003,
};

const SMELTING: Station = Station {
    content: "rust-opening",
    profile: 14,
    recipe: 2002,
};

struct Tutorial {
    game: Gameplay,
    character: CharacterState,
    actor: u64,
    stage_id: u32,
    objective_index: usize,
}

fn reference(content: &str) -> PathBuf {
    let root = std::env::var_os("OSFR_S6_REFERENCE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references")
        });
    let path = root.join(content);
    assert!(
        path.join("opening.json").is_file(),
        "S6 requires the private content at {}",
        path.display()
    );
    path
}

fn w104_content() -> PathBuf {
    let path = std::env::var_os("OSFR_S6_W104_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/build/w104/scratch/content")
        });
    assert!(
        path.join("opening.json").is_file() && path.join("harvesting.json").is_file(),
        "S6 requires the private W104 content at {}",
        path.display()
    );
    path
}

fn chat(text: &str) -> Vec<u8> {
    let mut packet = vec![15, 0, 1, 0, 0, 0];
    packet.extend(1_u64.to_le_bytes());
    packet.extend(0_u64.to_le_bytes());
    for _ in 0..2 {
        packet.extend([0; 12]);
        packet.extend(0_u32.to_le_bytes());
        packet.extend(0_u32.to_le_bytes());
    }
    packet.extend((text.len() as u32).to_le_bytes());
    packet.extend(text.as_bytes());
    packet.extend([0; 28]);
    packet
}

fn interact(guid: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 10, 0];
    packet.extend(guid.to_le_bytes());
    packet.extend(1_i32.to_le_bytes());
    packet
}

fn menu(guid: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(guid.to_le_bytes());
    packet.push(0);
    packet
}

fn chat_texts(replies: &[Vec<u8>]) -> Vec<String> {
    let mut texts = Vec::new();
    for packet in replies {
        if packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8 {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            if let Some(body) = packet.get(8..8 + length)
                && let Ok(text) = std::str::from_utf8(body)
            {
                texts.push(text.to_owned());
            }
        }
    }
    texts
}

fn assert_empty(out: &Outcome) {
    assert!(!out.changed);
    assert!(out.replies.is_empty());
}

fn send(tutorial: &mut Tutorial, inner: &[u8], now: u64) -> io::Result<Outcome> {
    tutorial
        .game
        .handle_in_world(inner, &tutorial.character, now)
}

fn tutorial(case: Station, accepted: bool) -> Tutorial {
    let directory = reference(case.content);
    let base = Gameplay::load(&directory, None).unwrap();
    let engine = OpeningEngine::load(&directory, base.state().opening.clone()).unwrap();
    let (stage_id, objective_index) = engine
        .stages()
        .iter()
        .find_map(|stage| {
            stage
                .objectives
                .iter()
                .position(|goal| {
                    matches!(
                        goal.trigger,
                        OpeningTrigger::Craft { recipe_id } if recipe_id == case.recipe
                    )
                })
                .map(|index| (stage.id, index))
        })
        .expect("tutorial Craft objective");
    let mut state = base.state();
    for stage in engine.stages() {
        let progress = state
            .opening
            .as_mut()
            .unwrap()
            .stages
            .get_mut(&stage.id)
            .unwrap();
        if stage.id < stage_id {
            progress.accepted = true;
            progress.completed = true;
            progress.rewarded = true;
            progress.counts = stage.objectives.iter().map(|goal| goal.required).collect();
        } else if stage.id == stage_id {
            progress.accepted = accepted;
            progress.completed = false;
            progress.rewarded = false;
            progress.counts = if accepted {
                stage
                    .objectives
                    .iter()
                    .enumerate()
                    .map(|(index, goal)| {
                        if index < objective_index {
                            goal.required
                        } else {
                            0
                        }
                    })
                    .collect()
            } else {
                vec![0; stage.objectives.len()]
            };
        }
    }
    state
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .insert(case.profile);
    let mut game = Gameplay::load(&directory, Some(state)).unwrap();
    game.profiles
        .as_mut()
        .unwrap()
        .select(case.profile, &game.inventory)
        .unwrap();
    let actor = game
        .opening
        .as_ref()
        .unwrap()
        .actors()
        .iter()
        .find(|actor| {
            matches!(
                actor.action,
                Some(OpeningTrigger::Craft { recipe_id }) if recipe_id == case.recipe
            )
        })
        .expect("tutorial Craft actor")
        .clone();
    let character = CharacterState {
        guid: 1,
        zone_id: actor.zone_id,
        position: actor.position,
        rotation: [0.0, 0.0, 1.0],
    };
    game.initialize_world(&character, NOW).unwrap();
    Tutorial {
        character,
        game,
        actor: actor.guid,
        stage_id,
        objective_index,
    }
}

fn grant_recipe_materials(game: &mut Gameplay, case: Station) {
    let required = match case.profile {
        16 => game.forging.as_ref().unwrap().requirements().to_vec(),
        14 => game.smelting.as_ref().unwrap().requirements().to_vec(),
        _ => unreachable!(),
    };
    for (definition, count) in required {
        game.inventory.grant(0, definition, count).unwrap();
    }
}

fn active_for_test(case: Station) -> Tutorial {
    let mut tutorial = tutorial(case, true);
    grant_recipe_materials(&mut tutorial.game, case);
    let actor = tutorial.actor;
    send(&mut tutorial, &interact(actor), NOW).unwrap();
    let active = match case.profile {
        16 => tutorial.game.forging.as_ref().is_some_and(Forging::active),
        14 => tutorial
            .game
            .smelting
            .as_ref()
            .is_some_and(Smelting::active),
        _ => unreachable!(),
    };
    assert!(active);
    tutorial
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn movie_ticks_wait_for_250ms_and_stamp_their_own_clock() {
    let mut forging = active_for_test(FORGING);
    let before = forging.game.state();
    let out = forging.game.tick(&forging.character, NOW + 100).unwrap();
    assert_empty(&out);
    assert_eq!(forging.game.forging_tick_ms, Some(NOW));
    assert_eq!(forging.game.state(), before);
    let out = forging.game.tick(&forging.character, NOW + 250).unwrap();
    assert!(out.replies.is_empty());
    assert_eq!(forging.game.forging_tick_ms, Some(NOW + 250));

    let directory = w104_content();
    let mut harvesting = Gameplay::load(&directory, None).unwrap();
    harvesting.profiles.as_mut().unwrap().unlock(45).unwrap();
    harvesting
        .profiles
        .as_mut()
        .unwrap()
        .select(45, &harvesting.inventory)
        .unwrap();
    harvesting.dev_tools = true;
    harvesting.set_role(AccountRole::Admin);
    let character = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    };
    harvesting
        .handle_in_world(&chat("realm harvest"), &character, NOW)
        .unwrap();
    assert!(
        harvesting
            .harvesting
            .as_ref()
            .is_some_and(Type8Engine::active)
    );
    assert_eq!(harvesting.type8_tick_ms, Some(NOW));
    let before = harvesting.state();
    let out = harvesting.tick(&character, NOW + 100).unwrap();
    assert_empty(&out);
    assert_eq!(harvesting.type8_tick_ms, Some(NOW));
    assert_eq!(harvesting.state(), before);
    harvesting.tick(&character, NOW + 250).unwrap();
    assert_eq!(harvesting.type8_tick_ms, Some(NOW + 250));
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn forging_exit_clears_the_movie_tick_clock() {
    let mut forging = active_for_test(FORGING);
    assert_eq!(forging.game.forging_tick_ms, Some(NOW));
    forging.game.dev_tools = true;
    forging.game.set_role(AccountRole::Admin);
    let out = send(&mut forging, &chat("realm exit"), NOW).unwrap();
    assert!(out.changed);
    assert!(
        forging
            .game
            .forging
            .as_ref()
            .is_none_or(|forging| !forging.active())
    );
    assert_eq!(forging.game.forging_tick_ms, None);
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn opening_craft_quantity_sums_every_copper_bar_grant_row() {
    // Smelting content models exactly one product row, so the packet path
    // cannot distinguish a sum from the first row. This calls the production
    // applier with a two-row transaction; the packet-path grant is pinned by
    // `smelting_completion_advances_opening_craft_and_grants_every_bar`.
    let mut tutorial = active_for_test(SMELTING);
    let before = tutorial.game.state();
    tutorial
        .game
        .apply_craft(
            CraftingJob::Miner,
            SmeltingOutcome {
                transaction: Some(SmeltingTransaction {
                    consumes: Vec::new(),
                    grants: vec![(21785, 2), (21785, 3)],
                    experience_points: 0,
                }),
                ..Default::default()
            },
            &tutorial.character,
            &mut Outcome::default(),
        )
        .unwrap();
    let after = tutorial.game.state();
    let mut expected = before.opening.as_ref().unwrap().stages[&tutorial.stage_id]
        .counts
        .clone();
    expected[tutorial.objective_index] += 5;
    assert_eq!(
        after.opening.as_ref().unwrap().stages[&tutorial.stage_id].counts,
        expected
    );
}

fn first_regional_npc(game: &Gameplay) -> (u64, u32, [f32; 3]) {
    let directory = w104_content();
    let raw = fs::read(directory.join("npcs/farnum_farmstead.json")).unwrap();
    let content: Value = serde_json::from_slice(&raw).unwrap();
    for record in content["records"].as_array().unwrap() {
        let Some(id) = record["npc_id"].as_u64() else {
            continue;
        };
        let Some(zone_id) = record["zone_id"].as_u64() else {
            continue;
        };
        let Some(position) = record["position"].as_array() else {
            continue;
        };
        let Some(position) = [
            position[0].as_f64(),
            position[1].as_f64(),
            position[2].as_f64(),
        ]
        .into_iter()
        .collect::<Option<Vec<_>>>() else {
            continue;
        };
        let visible = game
            .npc_visibility
            .as_ref()
            .and_then(|visibility| visibility.content_key(id))
            .is_some();
        let interactable = game
            .npc_interaction
            .as_ref()
            .is_some_and(|interaction| interaction.location(id).is_some());
        if visible && interactable {
            return (
                id,
                zone_id as u32,
                [position[0] as f32, position[1] as f32, position[2] as f32],
            );
        }
    }
    panic!("no positioned regional NPC is available in the W104 content");
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn real_regional_npc_reaches_the_npc_router_before_the_movie_gate() {
    // This pins the current (pre-existing) ordering for the lead's production
    // decision: a real regional NPC answers 26/8 while a forging movie runs,
    // because `handle_npc_packet` runs before `handle_opening_inner`. It needs
    // the private `npc_visibility`/`npc_interaction` lookup and an active movie
    // away from a station, so it cannot drive the packet path.
    let directory = w104_content();
    let mut game = Gameplay::load(&directory, None).unwrap();
    let key = game.forging.as_ref().unwrap().recipe_key().to_owned();
    let requirements = game.forging.as_ref().unwrap().requirements().to_vec();
    game.forging
        .as_mut()
        .unwrap()
        .begin_recipe(16, &key, u8::MAX, true, &requirements)
        .unwrap();
    assert!(game.forging.as_ref().is_some_and(Forging::active));
    let (npc, zone_id, position) = first_regional_npc(&game);
    let character = CharacterState {
        guid: 1,
        zone_id,
        position,
        rotation: [0.0, 0.0, 1.0],
    };
    let out = game.handle_in_world(&menu(npc), &character, NOW).unwrap();
    assert!(
        !chat_texts(&out.replies).contains(
            &"Close the forging game before changing jobs, equipment or interacting.".to_owned()
        ),
        "the movie gate must not pre-empt the real NPC router"
    );
    assert!(
        out.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "the real NPC router must produce its menu"
    );
}
