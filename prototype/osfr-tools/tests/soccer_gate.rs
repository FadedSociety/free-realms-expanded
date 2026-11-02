//! L932: the Snowhill soccer gates start a match from the world.
//!
//! Client facts: Global.Text 416257 / 416258 are "Right Side Soccer Gate" / "Left Side Soccer
//! Gate"; the Soccer Star quest objectives tell the player to "Click the soccer gate"
//! (402694, 402788, 402796). The gate is the client terrain object
//! `sg_soccer_entrance_gate_01.adr`, drawn as the invisible cube `Models.txt` row 271. The click is
//! the ordinary NPC path: 26/8 interact -> 26/9 menu -> 26/10 select; the 35/2 AddNpc reader is
//! `0x00921a10`, the 39/16 offer names the game at bytes 11..15.
//!
//! The tests drive the production seams: `Gameplay::update_npc_visibility` for the draw,
//! `Gameplay::handle_in_world` for the click and the choice, `Gameplay::initialize_world` for the
//! client's Loaded after the scene change.
//!
//! No player names or chat text are embedded.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    Fixture, character_at_for, content_dir, content_position, interact, menu_option,
    menu_option_opt, npc_select, seed_quest, send,
};
use osfr_tools::{
    character_store::CharacterState, gameplay::Gameplay, quest_engine::QuestStatus, soccer_entry,
    startup_world::parse_add_npc,
};
use serde_json::{Value, json};
use std::fs;

const GUID: u64 = 93_202;
const RIGHT_GATE: u64 = 9_500_001_022;
const LEFT_GATE: u64 = 9_500_001_023;
const NOT_A_GATE: u64 = 9_000_000_140;
/// Global.Text 3212 "Soccer Lobby", the gates' menu label.
const SOCCER_LABEL: u32 = 3_212;
/// `Models.txt` row 271 = invisible_cube.adr.
const INVISIBLE_CUBE: u32 = 271;
const SNOWHILL_SOCCER: i32 = 43;
const SLOWING_THE_SNOWSPURS: i32 = 326;
const SCRIMMAGE: u32 = 3_140_004;

/// The opening fixture with the shipped soccer package (rows, gates, pitch).
fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fs::copy(
        content_dir().join("soccer.json"),
        fixture.path().join("soccer.json"),
    )
    .unwrap();
    fixture
}

fn at(gate: u64) -> CharacterState {
    character_at_for(GUID, 0, content_position("snowhill.json", gate))
}

fn gateway_player(fixture: &Fixture, gate: u64, quest_open: bool) -> Gameplay {
    let mut state = fixture.game_as(GUID, None).state();
    if quest_open {
        seed_quest(&mut state, SCRIMMAGE, QuestStatus::Active, &[0]);
    }
    let mut game = fixture.game_as(GUID, Some(state));
    game.initialize_world(&at(gate), 1_000).unwrap();
    game
}

fn offered_game(replies: &[Vec<u8>]) -> i32 {
    let offer = replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("the 39/16 soccer offer");
    i32::from_le_bytes(offer[11..15].try_into().unwrap())
}

/// Click the gate, take its soccer option, answer the scene change with Loaded. Returns the
/// game id the offer names.
fn enter_by_gate(game: &mut Gameplay, gate: u64) -> i32 {
    let character = at(gate);
    let menu = interact(game, &character, gate, 2_000);
    let option = menu_option(&menu, SOCCER_LABEL);
    let entered = send(game, &character, &npc_select(gate, option), 2_100);
    assert!(
        entered.scene_zoning.is_some(),
        "the gate zones into sg_soccer"
    );
    assert!(game.is_soccer_active());
    let loaded = game.initialize_world(&character, 2_200).unwrap();
    offered_game(&loaded.replies)
}

#[test]
fn both_gates_are_drawn_as_invisible_cubes_at_the_snowhill_field() {
    let fixture = fixture();
    for (gate, name_id) in [(RIGHT_GATE, 416_257), (LEFT_GATE, 416_258)] {
        let mut game = gateway_player(&fixture, gate, false);
        let character = at(gate);
        let packets = game.update_npc_visibility(&character).unwrap();
        let drawn: Vec<_> = packets
            .iter()
            .filter_map(|packet| parse_add_npc(packet).ok())
            .filter(|spawn| spawn.actor_id == gate)
            .collect();
        assert_eq!(drawn.len(), 1, "gate {gate} is drawn once");
        assert_eq!(drawn[0].model_id, INVISIBLE_CUBE as i32);
        assert_eq!(drawn[0].name_string_id, name_id);
        assert_eq!(&drawn[0].position[..3], &character.position);
    }
}

#[test]
fn a_gate_offers_the_soccer_lobby_and_enters_snowhill_soccer() {
    let fixture = fixture();
    for gate in [RIGHT_GATE, LEFT_GATE] {
        let mut game = gateway_player(&fixture, gate, false);
        assert_eq!(
            enter_by_gate(&mut game, gate),
            SNOWHILL_SOCCER,
            "gate {gate}"
        );
    }
}

#[test]
fn an_open_quest_objective_picks_the_row_the_gate_enters() {
    let fixture = fixture();
    fixture.patch_quest("soccer_star.json", SCRIMMAGE, |record| {
        record["objectives"][0]["minigame_id"] = json!(SLOWING_THE_SNOWSPURS);
    });
    let mut game = gateway_player(&fixture, RIGHT_GATE, true);
    assert_eq!(enter_by_gate(&mut game, RIGHT_GATE), SLOWING_THE_SNOWSPURS);
}

#[test]
fn only_the_gates_offer_the_soccer_lobby() {
    let fixture = fixture();
    let mut game = gateway_player(&fixture, RIGHT_GATE, false);
    let menu = interact(&mut game, &at(NOT_A_GATE), NOT_A_GATE, 2_000);
    let packet = menu
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("Landon's 26/9 menu");
    assert_eq!(menu_option_opt(packet, SOCCER_LABEL), None);
}

/// A gate enters a Games window row; naming a quest-only row, or no NPC, is refused at load.
#[test]
fn a_gate_must_name_npcs_and_a_games_window_row() {
    let shipped: Value =
        serde_json::from_slice(&fs::read(content_dir().join("soccer.json")).unwrap()).unwrap();
    let directory = std::env::temp_dir().join(format!("osfr-l932-gate-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("soccer.json");
    let mut refusals = Vec::new();
    for (field, value) in [
        ("game_id", json!(SLOWING_THE_SNOWSPURS)),
        ("game_id", json!(7)),
        ("npcs", json!([])),
        ("npcs", json!([RIGHT_GATE, RIGHT_GATE])),
    ] {
        let mut document = shipped.clone();
        document["gates"][field] = value;
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        refusals.push(
            soccer_entry::Content::load(&path)
                .map(|_| ())
                .map_err(|e| e.to_string()),
        );
    }
    let _ = fs::remove_dir_all(&directory);
    for refusal in refusals {
        assert_eq!(
            refusal,
            Err("soccer gates need NPCs and a Games window row".to_owned())
        );
    }
}
