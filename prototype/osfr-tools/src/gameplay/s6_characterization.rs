//! Characterization net for the S6 crafting, gathering and movie merges.
//!
//! These tests observe the unmodified production path. They are ignored by the
//! ordinary suite because they require private lab content, and panic rather
//! than silently skipping when that content is absent.
//!
//! Wire prefixes are the shipped client's request shapes: 26/8 builder
//! 0x008b6bc0, 26/10 constructor 0x00a36f80, 26/13 sender 0x00938e10,
//! 36/10 writer 0x00a342e0 and 42/3 writer 0x00bdb450. Results Close is the
//! four-byte writer 0x0092f300. The 39/16 info reader is 0x009bfc50 and the
//! 39/14 payload writer is 0x009be6b0. The chat request reader is 0x00b11600.

use super::*;
use crate::{
    account_store::AccountRole,
    forging::ForgingPhase,
    smelting::{SmeltingOutcome, SmeltingTransaction},
};
use serde_json::Value;
use std::{fs, path::PathBuf};

const NOW: u64 = 1_000;
const UNKNOWN_ACTOR: u64 = u64::MAX - 1;

#[derive(Clone, Copy)]
struct Station {
    content: &'static str,
    profile: i32,
    recipe: u32,
    noun: &'static str,
    wrong_job: &'static str,
    pending: &'static str,
    shortage: &'static str,
    outside: &'static str,
    changed: &'static str,
    success: Option<&'static str>,
}

const FORGING: Station = Station {
    content: "rust-opening-forging",
    profile: 16,
    recipe: 2003,
    noun: "forging",
    wrong_job: "Switch to Blacksmith before forging.",
    pending: "Accept the forging objective and finish its earlier steps first.",
    shortage: "Forging requires 10 Copper Bars, 5 Turquoise, 5 Wooden Planks and 5 Thorny Fasteners.",
    outside: "forging completion is outside a Blacksmith station",
    changed: "forging ingredients changed before completion",
    success: Some(
        "Original Spunky Scrapper Hammer controls; board tuning, consumption and tint choice are reconstructed.",
    ),
};

const COOKING: Station = Station {
    content: "rust-opening-cooking-original-assets",
    profile: 45,
    recipe: 2001,
    noun: "cooking",
    wrong_job: "Switch to Chef before cooking.",
    pending: "Accept the cooking objective and finish its earlier steps first.",
    shortage: "StrongArm Stew requires 4 Bat Meat, 4 Gummyfruit, 4 Rainleaf and 4 Spice.",
    outside: "cooking completion is outside a Chef station",
    changed: "cooking ingredients changed before completion",
    success: None,
};

const SMELTING: Station = Station {
    content: "rust-opening",
    profile: 14,
    recipe: 2002,
    noun: "smelting",
    wrong_job: "Switch to Miner before smelting.",
    pending: "Accept the smelting objective and finish its earlier steps first.",
    shortage: "Smelting requires 5 Copper Ore and 3 Singing Coal. Mine the accepted nearby ore objective first.",
    outside: "smelting completion is outside a Miner station",
    changed: "smelting ingredients changed before completion",
    success: Some(
        "Original four-stage smelting controls; phase tuning and this local recipe output are reconstructed.",
    ),
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

fn chat_texts(replies: &[Vec<u8>]) -> Vec<String> {
    let mut texts = Vec::new();
    for reply in replies {
        let packet = if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
            let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
            if length == reply.len() - 7 {
                &reply[7..]
            } else {
                reply
            }
        } else {
            reply
        };
        if packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8 {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            texts.push(
                std::str::from_utf8(&packet[8..8 + length])
                    .unwrap()
                    .to_owned(),
            );
        }
    }
    texts
}

fn assert_empty(out: &Outcome) {
    assert!(!out.dev_command_denied);
    assert!(out.shared_field.is_none());
    assert!(out.replies.is_empty());
    assert!(!out.changed);
    assert!(out.zoning.is_none());
    assert!(out.scene_zoning.is_none());
    assert!(out.relocation.is_none());
    assert!(out.group_heal.is_none());
    assert!(out.nearby.is_empty());
    assert!(!out.pets_changed);
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
        45 => game.cooking.as_ref().unwrap().requirements().to_vec(),
        14 => game.smelting.as_ref().unwrap().requirements().to_vec(),
        _ => unreachable!(),
    };
    for (definition, count) in required {
        game.inventory.grant(0, definition, count).unwrap();
    }
}

fn send(tutorial: &mut Tutorial, inner: &[u8], now: u64) -> io::Result<Outcome> {
    tutorial
        .game
        .handle_in_world(inner, &tutorial.character, now)
}

fn assert_station_refusals(case: Station) {
    for (active, accepted, supplies, expected) in [
        (1, true, true, case.wrong_job),
        (case.profile, false, true, case.pending),
        (case.profile, true, false, case.shortage),
    ] {
        let mut tutorial = tutorial(case, accepted);
        tutorial
            .game
            .profiles
            .as_mut()
            .unwrap()
            .select(active, &tutorial.game.inventory)
            .unwrap();
        if supplies {
            grant_recipe_materials(&mut tutorial.game, case);
        }
        let before = tutorial.game.state();
        let actor = tutorial.actor;
        let out = send(&mut tutorial, &interact(actor), NOW).unwrap();
        assert_eq!(chat_texts(&out.replies), vec![expected.to_owned()]);
        assert_eq!(tutorial.game.state(), before);
    }
}

fn payload(case: Station, instance: i32, text: &str) -> Vec<u8> {
    let mut packet = match case.profile {
        16 => crate::forging::payload_packet(instance, text),
        45 => crate::cooking::payload_packet(instance, text),
        14 => crate::smelting::payload_packet(instance, text),
        _ => unreachable!(),
    }
    .unwrap();
    packet[11..15].copy_from_slice(&(-1_i32).to_le_bytes());
    packet
}

fn instance(game: &Gameplay, case: Station) -> i32 {
    match case.profile {
        16 => {
            game.forging
                .as_ref()
                .unwrap()
                .state()
                .active
                .as_ref()
                .unwrap()
                .instance_id
        }
        45 => {
            game.cooking
                .as_ref()
                .unwrap()
                .state()
                .active
                .as_ref()
                .unwrap()
                .instance_id
        }
        14 => {
            game.smelting
                .as_ref()
                .unwrap()
                .state()
                .active
                .as_ref()
                .unwrap()
                .instance_id
        }
        _ => unreachable!(),
    }
}

fn send_as(
    tutorial: &mut Tutorial,
    inner: &[u8],
    character: &CharacterState,
) -> io::Result<Outcome> {
    tutorial.game.handle_in_world(inner, character, NOW)
}

fn prepare_terminal(tutorial: &mut Tutorial, case: Station) {
    let id = instance(&tutorial.game, case);
    let character = tutorial.character.clone();
    let mut start = vec![39, 0, 5];
    for value in [id, -1, -1] {
        start.extend(value.to_le_bytes());
    }
    send_as(tutorial, &start, &character).unwrap();
    match case.profile {
        16 => {
            for text in ["OnConnectMsg", "OnStartGameMsg\t0", "OnStartLevelMsg\t0"] {
                send_as(tutorial, &payload(case, id, text), &character).unwrap();
            }
            for _ in 0..2_000 {
                let (phase, board) = {
                    let active = tutorial
                        .game
                        .forging
                        .as_ref()
                        .unwrap()
                        .state()
                        .active
                        .as_ref()
                        .unwrap();
                    (active.phase, active.board.clone())
                };
                if phase == ForgingPhase::LevelComplete {
                    return;
                }
                assert_eq!(phase, ForgingPhase::Playing);
                let mut idle = String::from("OnIdleMsg\t");
                for y in 0..14 {
                    for x in 0..14 {
                        if (401..=405).contains(&board[y * 14 + x]) || board[y * 14 + x] == 606 {
                            idle += &format!("{x},{y},");
                        }
                    }
                }
                send_as(tutorial, &payload(case, id, &idle), &character).unwrap();
                let board = tutorial
                    .game
                    .forging
                    .as_ref()
                    .unwrap()
                    .state()
                    .active
                    .as_ref()
                    .unwrap()
                    .board
                    .clone();
                if let Some((x, y)) = (0..13)
                    .flat_map(|y| (0..14).map(move |x| (x, y)))
                    .find(|&(x, y)| board[y * 14 + x] == 606 && board[(y + 1) * 14 + x] == 100)
                {
                    send_as(
                        tutorial,
                        &payload(case, id, &format!("OnLevelEventMsg\t50\t{x}\t{y}")),
                        &character,
                    )
                    .unwrap();
                    continue;
                }
                let mut found = None;
                'outer: for y in (1..13).rev() {
                    for x in 1..13 {
                        let value = board[y * 14 + x];
                        if !(401..=405).contains(&value) {
                            continue;
                        }
                        for (x2, y2) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                            if board[y2 * 14 + x2] != value {
                                continue;
                            }
                            for (x3, y3) in [
                                (x2.saturating_sub(1), y2),
                                (x2 + 1, y2),
                                (x2, y2.saturating_sub(1)),
                                (x2, y2 + 1),
                            ] {
                                if x3 < 14
                                    && y3 < 14
                                    && (x3, y3) != (x, y)
                                    && board[y3 * 14 + x3] == value
                                {
                                    found =
                                        Some(format!("OnChainMsg\t{x},{y},{x2},{y2},{x3},{y3},"));
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                send_as(
                    tutorial,
                    &payload(
                        case,
                        id,
                        &found.expect("server must reshuffle a board without a move"),
                    ),
                    &character,
                )
                .unwrap();
            }
            panic!("bounded forging transcript did not reach the terminal level");
        }
        45 => {
            send_as(
                tutorial,
                &payload(case, id, "FRServer_ReadyForGameData"),
                &character,
            )
            .unwrap();
            for stage in [1, 2, 3, 6, 17, 7] {
                for kind in 0..4 {
                    send_as(
                        tutorial,
                        &payload(
                            case,
                            id,
                            &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"),
                        ),
                        &character,
                    )
                    .unwrap();
                }
            }
            for kind in 4..8 {
                send_as(
                    tutorial,
                    &payload(case, id, &format!("FRServer_StatUpdate\t-1\t{kind}\t6)")),
                    &character,
                )
                .unwrap();
            }
        }
        14 => {
            send_as(
                tutorial,
                &payload(case, id, "FRServer_ReadyForGameData"),
                &character,
            )
            .unwrap();
            for stage in 101..=104 {
                for kind in 0..=3 {
                    send_as(
                        tutorial,
                        &payload(
                            case,
                            id,
                            &format!("FRServer_StatUpdate\t{stage}\t{kind}\t1)"),
                        ),
                        &character,
                    )
                    .unwrap();
                }
            }
        }
        _ => unreachable!(),
    }
}

fn terminal(
    tutorial: &mut Tutorial,
    case: Station,
    character: &CharacterState,
) -> io::Result<Outcome> {
    let text = match case.profile {
        16 => "OnNextLevelMsg",
        45 => "FRServer_GameWon\t900\t0",
        14 => "FRServer_GameWon\t900\t1",
        _ => unreachable!(),
    };
    send_as(
        tutorial,
        &payload(case, instance(&tutorial.game, case), text),
        character,
    )
}

fn assert_completion_errors(case: Station) {
    let mut tutorial = active_for_test(case);
    prepare_terminal(&mut tutorial, case);
    let far = CharacterState {
        guid: 1,
        zone_id: tutorial.character.zone_id,
        position: [1_000_000.0; 3],
        rotation: tutorial.character.rotation,
    };
    let before = tutorial.game.state();
    let outside = terminal(&mut tutorial, case, &far)
        .err()
        .expect("completion outside the station must fail");
    assert_eq!(outside.to_string(), case.outside);
    assert_eq!(tutorial.game.state(), before);

    let required = match case.profile {
        16 => tutorial
            .game
            .forging
            .as_ref()
            .unwrap()
            .requirements()
            .to_vec(),
        45 => tutorial
            .game
            .cooking
            .as_ref()
            .unwrap()
            .requirements()
            .to_vec(),
        14 => tutorial
            .game
            .smelting
            .as_ref()
            .unwrap()
            .requirements()
            .to_vec(),
        _ => unreachable!(),
    };
    for (definition, count) in required {
        let mut remaining = count;
        while remaining > 0 {
            let item = tutorial
                .game
                .inventory
                .state()
                .items
                .iter()
                .find(|item| {
                    item.definition_id == definition && item.count > 0 && item.rental.is_none()
                })
                .unwrap()
                .clone();
            let take = remaining.min(item.count);
            tutorial.game.inventory.consume(item.id, take).unwrap();
            remaining -= take;
        }
    }
    let before = tutorial.game.state();
    let character = tutorial.character.clone();
    let changed = terminal(&mut tutorial, case, &character)
        .err()
        .expect("changed ingredients must fail");
    assert_eq!(changed.to_string(), case.changed);
    assert_eq!(tutorial.game.state(), before);
}

fn assert_station_success(case: Station) {
    let mut tutorial = tutorial(case, true);
    grant_recipe_materials(&mut tutorial.game, case);
    let actor = tutorial.actor;
    let out = send(&mut tutorial, &interact(actor), NOW).unwrap();
    assert_eq!(
        chat_texts(&out.replies),
        case.success
            .map(|text| vec![text.to_owned()])
            .unwrap_or_default()
    );
    let active = match case.profile {
        16 => tutorial.game.forging.as_ref().is_some_and(Forging::active),
        45 => tutorial.game.cooking.as_ref().is_some_and(Cooking::active),
        14 => tutorial
            .game
            .smelting
            .as_ref()
            .is_some_and(Smelting::active),
        _ => unreachable!(),
    };
    assert!(active);
    if case.profile == 16 {
        assert_eq!(tutorial.game.forging_tick_ms, Some(NOW));
    }
}

fn active_for_test(case: Station) -> Tutorial {
    let mut tutorial = tutorial(case, true);
    grant_recipe_materials(&mut tutorial.game, case);
    let actor = tutorial.actor;
    send(&mut tutorial, &interact(actor), NOW).unwrap();
    tutorial
}

fn assert_movie_gate(case: Station) {
    let mut tutorial = active_for_test(case);
    let expected = format!(
        "Close the {} game before changing jobs, equipment or interacting.",
        case.noun
    );
    let attack = [
        vec![36, 0, 10, 0],
        UNKNOWN_ACTOR.to_le_bytes().to_vec(),
        1_000_000_i64.to_le_bytes().to_vec(),
    ]
    .concat();
    let equip = [
        vec![42, 0, 3, 0],
        UNKNOWN_ACTOR.to_le_bytes().to_vec(),
        i32::MAX.to_le_bytes().to_vec(),
    ]
    .concat();
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(UNKNOWN_ACTOR.to_le_bytes());
    menu.push(0);
    for packet in [
        vec![26, 0, 13, 0, 1, 0, 0, 0],
        menu,
        interact(UNKNOWN_ACTOR),
        attack,
        equip,
    ] {
        let before = tutorial.game.state();
        let out = send(&mut tutorial, &packet, NOW).unwrap();
        assert_eq!(chat_texts(&out.replies), vec![expected.clone()]);
        assert_eq!(tutorial.game.state(), before);
    }

    let before = tutorial.game.state();
    let error = tutorial
        .game
        .handle_in_world(&[26, 0, 42, 0, 0], &tutorial.character, NOW)
        .err()
        .expect("malformed leave must fail");
    assert_eq!(
        error.to_string(),
        format!("invalid {} leave request", case.noun)
    );
    assert_eq!(tutorial.game.state(), before);
}

fn assert_forging_chat_gate() {
    let mut tutorial = active_for_test(FORGING);
    tutorial.game.dev_tools = true;
    tutorial.game.set_role(AccountRole::Admin);
    let before = tutorial.game.state();
    let out = send(&mut tutorial, &chat("realm x"), NOW).unwrap();
    assert_eq!(
        chat_texts(&out.replies),
        vec!["Close the forging game before other opening actions.".to_owned()]
    );
    assert_eq!(tutorial.game.state(), before);

    let out = send(&mut tutorial, &chat(""), NOW).unwrap();
    assert_empty(&out);
    assert_eq!(tutorial.game.state(), before);

    let out = send(&mut tutorial, &chat("realm exit"), NOW).unwrap();
    assert!(out.changed);
    assert!(
        tutorial
            .game
            .forging
            .as_ref()
            .is_none_or(|forging| !forging.active())
    );
    assert_eq!(tutorial.game.forging_tick_ms, None);
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn forging_characterization_pins_movie_tutorial_gate_and_errors() {
    assert_station_refusals(FORGING);
    assert_station_success(FORGING);
    assert_movie_gate(FORGING);
    assert_completion_errors(FORGING);
    assert_forging_chat_gate();
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn cooking_characterization_pins_movie_tutorial_gate_and_errors() {
    assert_station_refusals(COOKING);
    assert_station_success(COOKING);
    assert_movie_gate(COOKING);
    assert_completion_errors(COOKING);
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn smelting_characterization_pins_movie_tutorial_gate_and_errors() {
    assert_station_refusals(SMELTING);
    assert_station_success(SMELTING);
    assert_movie_gate(SMELTING);
    assert_completion_errors(SMELTING);
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn smelting_completion_advances_opening_craft_through_movie_packets() {
    let mut tutorial = active_for_test(SMELTING);
    prepare_terminal(&mut tutorial, SMELTING);
    let before = tutorial.game.state();
    let character = tutorial.character.clone();
    terminal(&mut tutorial, SMELTING, &character).unwrap();
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

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn opening_craft_quantity_sums_every_copper_bar_grant_row() {
    // The shipped fixture has one 10-bar product row, so the packet path cannot
    // distinguish a sum from taking the first row. Pin that branch directly.
    let mut tutorial = active_for_test(SMELTING);
    let before = tutorial.game.state();
    tutorial
        .game
        .apply_smelting(
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

fn gather_case(
    resource_id: u32,
    profile: i32,
    content: &'static str,
) -> (Gameplay, CharacterState, u64, u32, usize) {
    let directory = reference(content);
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
                        OpeningTrigger::Gather { resource_id: id } if id == resource_id
                    )
                })
                .map(|index| (stage.id, index))
        })
        .expect("tutorial Gather objective");
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
            progress.accepted = true;
            progress.completed = false;
            progress.rewarded = false;
            progress.counts = stage
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
                .collect();
        }
    }
    state.profiles.as_mut().unwrap().unlocked.insert(profile);
    let mut game = Gameplay::load(&directory, Some(state)).unwrap();
    game.profiles
        .as_mut()
        .unwrap()
        .select(profile, &game.inventory)
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
                Some(OpeningTrigger::Gather { resource_id: id }) if id == resource_id
            )
        })
        .expect("tutorial Gather actor")
        .clone();
    let character = CharacterState {
        guid: 1,
        zone_id: actor.zone_id,
        position: actor.position,
        rotation: [0.0, 0.0, 1.0],
    };
    game.initialize_world(&character, NOW).unwrap();
    (game, character, actor.guid, stage_id, objective_index)
}

fn item_count(game: &Gameplay, definition: i32) -> u32 {
    game.inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

fn assert_bonus_drop(
    resource_id: u32,
    profile: i32,
    content: &'static str,
    expected: &[(i32, u32)],
) {
    let (mut game, character, actor, stage_id, objective_index) =
        gather_case(resource_id, profile, content);
    let before = expected
        .iter()
        .map(|(definition, _)| (*definition, item_count(&game, *definition)))
        .collect::<Vec<_>>();
    game.handle_in_world(&interact(actor), &character, NOW)
        .unwrap();
    for (definition, count) in expected {
        assert_eq!(
            item_count(&game, *definition),
            before.iter().find(|(id, _)| id == definition).unwrap().1 + count
        );
    }
    assert_eq!(
        game.state().opening.as_ref().unwrap().stages[&stage_id].counts[objective_index],
        1
    );
}

#[test]
#[ignore = "requires private forging cooking smelting and W104 content"]
fn opening_gather_bonus_drops_are_counted_only_with_valid_job_progress() {
    assert_bonus_drop(1003, 14, "rust-opening", &[(2739, 1), (879, 1)]);
    assert_bonus_drop(
        1001,
        45,
        "rust-opening-cooking-original-assets",
        &[(2884, 1)],
    );
    assert_bonus_drop(
        1002,
        45,
        "rust-opening-cooking-original-assets",
        &[(2875, 1)],
    );
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
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(npc.to_le_bytes());
    menu.push(0);
    let out = game.handle_in_world(&menu, &character, NOW).unwrap();
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
