//! Private fixture checks that need server-internal movie clocks or private
//! appliers. The packet-path characterization (station refusals, the movie
//! gate, completion errors, gathering bonuses) lives in
//! `tests/s6_characterization.rs` and reuses the shared movie drivers. The
//! opening fixture and content paths are the shared `tests/common/s6.rs`
//! module; the request bodies are `tests/common/gateway_packets.rs`, so the
//! two files cannot drift.
//!
//! Wire prefixes here are the shipped client's request shapes: 26/10
//! constructor 0x00a36f80 and 26/8 builder 0x008b6bc0 for the NPC ordering
//! check; the chat request reader is 0x00b11600.

use super::*;
#[path = "../../tests/common/gateway_packets.rs"]
mod gateway_packets;
#[path = "../../tests/common/s6.rs"]
mod s6;
use crate::{
    account_store::AccountRole,
    recipes::CraftingJob,
    smelting::{SmeltingOutcome, SmeltingTransaction},
};
use gateway_packets::{chat, interact, select};
use s6::{chat_texts, w104_content};
use serde_json::Value;
use std::fs;

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

fn send(tutorial: &mut Tutorial, inner: &[u8], now: u64) -> io::Result<Outcome> {
    tutorial
        .game
        .handle_in_world(inner, &tutorial.character, now)
}

fn tutorial(case: Station, accepted: bool) -> Tutorial {
    let s6::OpeningFixture {
        content,
        state,
        actor,
        stage_id,
        objective_index,
    } = s6::opening_fixture(
        case.content,
        s6::TutorialGoal::Craft(case.recipe),
        &[case.profile],
        accepted,
        case.profile,
        &[],
    );
    let mut game = Gameplay::load(&content, Some(state)).unwrap();
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
    send(&mut tutorial, &select(actor, 1), NOW).unwrap();
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
#[ignore = "requires private smelting content"]
fn miner_gather_bonus_requires_the_active_miner_job() {
    // A wrong active job normally refuses before the opening engine counts, so
    // that negative never reaches the drop guard's own job clause. Force the
    // counted path instead: the opening engine still selects Miner (so the
    // gather counts), while Gameplay's active job is Adventurer.
    let fixture = s6::opening_fixture(
        SMELTING.content,
        s6::TutorialGoal::Gather(1003),
        &[1, 14],
        true,
        14,
        &[],
    );
    let mut game = Gameplay::load(&fixture.content, Some(fixture.state)).unwrap();
    let character = CharacterState {
        guid: 1,
        zone_id: fixture.actor.zone_id,
        position: fixture.actor.position,
        rotation: [0.0, 0.0, 1.0],
    };
    // initialize_world feeds the Miner profile into the opening engine.
    game.initialize_world(&character, NOW).unwrap();
    game.profiles.as_mut().unwrap().activate(1).unwrap();
    let ore = game.material_count(2739);
    let coal = game.material_count(879);
    let inventory = game.inventory.state().clone();
    let before = game.opening.as_ref().unwrap().state().stages[&fixture.stage_id].counts
        [fixture.objective_index];
    let out = game
        .handle_in_world(&select(fixture.actor.guid, 1), &character, NOW)
        .unwrap();
    assert!(out.changed, "the gather must count for this fixture");
    assert_eq!(
        game.inventory.state(),
        &inventory,
        "the wrong job changed the inventory"
    );
    assert_eq!(
        game.opening.as_ref().unwrap().state().stages[&fixture.stage_id].counts
            [fixture.objective_index],
        before + 1
    );
    assert_eq!(
        game.material_count(2739),
        ore,
        "wrong job paid the ore bonus"
    );
    assert_eq!(
        game.material_count(879),
        coal,
        "wrong job paid the coal bonus"
    );
}

#[test]
#[ignore = "requires private forging content and the W104 harvesting content"]
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
#[ignore = "requires private forging content"]
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
#[ignore = "requires private smelting content"]
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
#[ignore = "requires the private W104 content"]
fn real_regional_npc_reaches_the_npc_router_before_the_movie_gate() {
    // The router still runs before `handle_opening_inner`'s movie gate, but the lead
    // decision (2026-10-20) closes the interaction while an activity runs: the shared
    // busy gate refuses before any menu opens and before any quest progress event.
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
    let out = game
        .handle_in_world(&interact(npc), &character, NOW)
        .unwrap();
    let texts = chat_texts(&out.replies);
    assert!(
        !texts.contains(
            &"Close the forging game before changing jobs, equipment or interacting.".to_owned()
        ),
        "the movie gate must not pre-empt the real NPC router"
    );
    assert!(
        texts.contains(&"Finish the active activity before interacting.".to_owned()),
        "the real NPC router must refuse the busy interaction: {texts:?}"
    );
    assert!(
        out.replies
            .iter()
            .all(|packet| !packet.starts_with(&[26, 0, 9, 0])),
        "a busy interaction must not open a menu"
    );
}

const UNKNOWN_ACTOR: u64 = u64::MAX - 1;
/// The services' private profile constants: forging 16, cooking 45, smelting 14.
const FORGING_PROFILE: i32 = 16;
const COOKING_PROFILE: i32 = 45;
const SMELTING_PROFILE: i32 = 14;

/// The five crafting and gathering movies in the gate's intercept order. This
/// table is the test's own copy, so the same matrix characterized the five
/// per-engine blocks before they became one gate.
#[derive(Clone, Copy)]
enum Engine {
    Forging,
    Cooking,
    Smelting,
    Mining,
    Harvesting,
}

impl Engine {
    const ALL: [Engine; 5] = [
        Engine::Forging,
        Engine::Cooking,
        Engine::Smelting,
        Engine::Mining,
        Engine::Harvesting,
    ];

    fn noun(self) -> &'static str {
        match self {
            Engine::Forging => "forging",
            Engine::Cooking => "cooking",
            Engine::Smelting => "smelting",
            Engine::Mining => "mining",
            Engine::Harvesting => "harvesting",
        }
    }

    /// Whether this movie's engine still holds a run. Read from the saved state
    /// so the check does not copy the production `active()` detection.
    fn is_active(self, game: &Gameplay) -> bool {
        let state = game.state();
        match self {
            Engine::Forging => state
                .forging
                .is_some_and(|forging| forging.active.is_some()),
            Engine::Cooking => state
                .cooking
                .is_some_and(|cooking| cooking.active.is_some()),
            Engine::Smelting => state
                .smelting
                .is_some_and(|smelting| smelting.active.is_some()),
            Engine::Mining => state.mining.is_some_and(|mining| mining.active.is_some()),
            Engine::Harvesting => state
                .harvesting
                .is_some_and(|harvesting| harvesting.active.is_some()),
        }
    }
}

fn w104_game() -> Gameplay {
    Gameplay::load(&w104_content(), None).unwrap()
}

fn test_character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

/// Starts one movie on its engine as the production start path leaves it: the
/// engine run is live and the movie's own tick clock is stamped.
fn start(game: &mut Gameplay, engine: Engine) {
    match engine {
        Engine::Forging => {
            let key = game.forging.as_ref().unwrap().recipe_key().to_owned();
            let requirements = game.forging.as_ref().unwrap().requirements().to_vec();
            game.forging
                .as_mut()
                .unwrap()
                .begin_recipe(FORGING_PROFILE, &key, u8::MAX, true, &requirements)
                .unwrap();
            game.forging_tick_ms = Some(NOW);
        }
        Engine::Cooking => {
            let key = game.cooking.as_ref().unwrap().recipe_key().to_owned();
            let requirements = game.cooking.as_ref().unwrap().requirements().to_vec();
            game.cooking
                .as_mut()
                .unwrap()
                .begin_recipe(COOKING_PROFILE, &key, u8::MAX, true, &requirements)
                .unwrap();
        }
        Engine::Smelting => {
            let key = game.smelting.as_ref().unwrap().recipe_key().to_owned();
            let requirements = game.smelting.as_ref().unwrap().requirements().to_vec();
            game.smelting
                .as_mut()
                .unwrap()
                .begin_recipe(SMELTING_PROFILE, &key, u8::MAX, true, &requirements)
                .unwrap();
        }
        Engine::Mining => {
            game.type8_mut(Gather::Mining)
                .unwrap()
                .begin(mining::MINER_PROFILE_ID)
                .unwrap();
            game.type8_tick_ms = Some(NOW);
        }
        Engine::Harvesting => {
            game.type8_mut(Gather::Harvesting)
                .unwrap()
                .begin(harvesting::CHEF_PROFILE_ID)
                .unwrap();
            game.type8_tick_ms = Some(NOW);
        }
    }
}

fn running_movie_fixture(engine: Engine) -> (Gameplay, CharacterState) {
    let mut game = w104_game();
    start(&mut game, engine);
    (game, test_character())
}

/// The five families a live movie refuses, one per blocked request shape. The
/// ability writer 0x00a342e0 sends i32 bar, i32 slot, i32 target mode and a u64
/// target id for mode 0.
fn blocked_families() -> [Vec<u8>; 5] {
    let mut job = vec![26, 0, 13, 0];
    job.extend(1_i32.to_le_bytes());
    let mut attack = vec![36, 0, 10, 0];
    attack.extend([0; 12]);
    attack.extend(UNKNOWN_ACTOR.to_le_bytes());
    let mut equip = vec![42, 0, 3, 0];
    equip.extend(i32::MAX.to_le_bytes());
    equip.extend(1_i32.to_le_bytes());
    equip.extend(0_i32.to_le_bytes());
    [
        job,
        interact(UNKNOWN_ACTOR),
        select(UNKNOWN_ACTOR, 1),
        attack,
        equip,
    ]
}

#[test]
#[ignore = "requires the private W104 content"]
fn two_active_engines_keep_the_gate_intercept_order() {
    // Only a direct fixture can make two engines active at once: the busy table
    // and the load gate keep production to one. The first block in gate order
    // answers, as the five if-blocks did.
    let character = test_character();
    let mut game = w104_game();
    start(&mut game, Engine::Forging);
    start(&mut game, Engine::Mining);
    assert!(matches!(game.running_movie(), Some(Movie::Forging)));
    let out = game
        .handle_in_world(&interact(UNKNOWN_ACTOR), &character, NOW)
        .unwrap();
    assert_eq!(
        chat_texts(&out.replies),
        vec!["Close the forging game before changing jobs, equipment or interacting.".to_owned()]
    );

    let mut game = w104_game();
    start(&mut game, Engine::Cooking);
    start(&mut game, Engine::Smelting);
    assert!(matches!(game.running_movie(), Some(Movie::Cooking)));
    let out = game
        .handle_in_world(&interact(UNKNOWN_ACTOR), &character, NOW)
        .unwrap();
    assert_eq!(
        chat_texts(&out.replies),
        vec!["Close the cooking game before changing jobs, equipment or interacting.".to_owned()]
    );
}

#[test]
#[ignore = "requires the private W104 content"]
fn one_movie_gate_keeps_the_five_blocks_refusals_and_cancel() {
    for engine in Engine::ALL {
        let (mut game, character) = running_movie_fixture(engine);
        let noun = engine.noun();
        let before = game.state();

        // 26/42 with a wrong length is refused with that movie's error and
        // changes nothing.
        let error = game
            .handle_in_world(&[26, 0, 42, 0, 0], &character, NOW)
            .err()
            .expect("the malformed leave must be refused");
        assert_eq!(error.to_string(), format!("invalid {noun} leave request"));
        assert_eq!(
            game.state(),
            before,
            "{noun}: the malformed leave changed the state"
        );

        // Every blocked family answers with that movie's text and stages nothing.
        for packet in blocked_families() {
            let out = game.handle_in_world(&packet, &character, NOW).unwrap();
            assert_eq!(
                chat_texts(&out.replies),
                vec![format!(
                    "Close the {noun} game before changing jobs, equipment or interacting."
                )],
                "{noun}: {packet:?}"
            );
            assert!(!out.changed, "{noun}: a blocked family changed the state");
            assert_eq!(game.state(), before, "{noun}: {packet:?}");
        }

        // Other realm text and plain chat are answered and swallowed as before.
        // `status` is an ordinary command, so the developer gate in front of the
        // movie gate does not answer it first.
        let out = game
            .handle_in_world(&chat("realm status"), &character, NOW)
            .unwrap();
        assert_eq!(
            chat_texts(&out.replies),
            vec![format!(
                "Close the {noun} game before other opening actions."
            )]
        );
        assert_eq!(game.state(), before, "{noun}: realm text changed the state");
        let out = game.handle_in_world(&chat(""), &character, NOW).unwrap();
        assert!(
            chat_texts(&out.replies).is_empty(),
            "{noun}: plain chat was answered"
        );
        assert_eq!(game.state(), before, "{noun}: plain chat changed the state");

        // 39/x reaches that movie's engine: a truncated header is its own error
        // and leaves the run untouched.
        let error = game
            .handle_in_world(&[39, 0, 5], &character, NOW)
            .err()
            .expect("the truncated 39/x must reach the engine");
        assert_eq!(error.to_string(), "truncated minigame header", "{noun}");
        assert_eq!(game.state(), before, "{noun}: the 39/x error changed state");

        // `realm exit` cancels the engine, applies its outcome and clears the
        // movie's clock; the run grants nothing.
        let out = game
            .handle_in_world(&chat("realm exit"), &character, NOW)
            .unwrap();
        assert!(out.changed, "{noun}: exit changed nothing");
        assert!(
            !engine.is_active(&game),
            "{noun}: exit left the movie active"
        );
        assert_eq!(game.forging_tick_ms, None, "{noun}: forging clock left");
        assert_eq!(game.type8_tick_ms, None, "{noun}: type8 clock left");
        assert_eq!(
            &game.state().inventory,
            &before.inventory,
            "{noun}: exit paid an item"
        );
    }
}

#[test]
#[ignore = "requires the private W104 content"]
fn movie_tick_keeps_the_250ms_rule_for_every_engine() {
    for engine in [Engine::Forging, Engine::Mining, Engine::Harvesting] {
        let (mut game, character) = running_movie_fixture(engine);
        let noun = engine.noun();
        let clock = |game: &Gameplay| match engine {
            Engine::Forging => game.forging_tick_ms,
            _ => game.type8_tick_ms,
        };
        assert_eq!(clock(&game), Some(NOW), "{noun}: the start clock");
        let before = game.state();
        let out = game.tick(&character, NOW + 100).unwrap();
        assert!(out.replies.is_empty(), "{noun}: a not-due tick replied");
        assert_eq!(clock(&game), Some(NOW), "{noun}: a not-due tick stamped");
        assert_eq!(
            game.state(),
            before,
            "{noun}: a not-due tick advanced the run"
        );

        game.tick(&character, NOW + 250).unwrap();
        assert_eq!(
            clock(&game),
            Some(NOW + 250),
            "{noun}: the due tick did not stamp"
        );
        // Fifty milliseconds later is not due again; the stamp stays put.
        game.tick(&character, NOW + 300).unwrap();
        assert_eq!(clock(&game), Some(NOW + 250), "{noun}: an early restamp");
    }

    // Cooking and smelting run on client input alone: their world tick is the
    // same no-op and neither clock is touched.
    for engine in [Engine::Cooking, Engine::Smelting] {
        let (mut game, character) = running_movie_fixture(engine);
        let noun = engine.noun();
        let before = game.state();
        let out = game.tick(&character, NOW + 1_000).unwrap();
        assert!(out.replies.is_empty(), "{noun}: the no-op tick replied");
        assert_eq!(game.state(), before, "{noun}: the no-op tick changed state");
        assert_eq!(game.forging_tick_ms, None);
        assert_eq!(game.type8_tick_ms, None);
    }

    // A missing clock means no tick at all.
    let (mut game, character) = running_movie_fixture(Engine::Forging);
    game.forging_tick_ms = None;
    let before = game.state();
    let out = game.tick(&character, NOW + 5_000).unwrap();
    assert!(out.replies.is_empty());
    assert_eq!(game.forging_tick_ms, None);
    assert_eq!(game.state(), before);
}
