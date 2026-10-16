//! W142 patch proof: the `gameplay.rs` wiring of piece_077. Drop this file in as
//! `tests/warpstone_gameplay.rs` AFTER `apply_gameplay_patch.py` is applied; it
//! does not compile before (it needs the persisted `warpstones` field).
//!
//! It drives the real `Gameplay` with raw inner packets: world entry sends the
//! attuned list, standing near a warpstone attunes it and saves, `90` relocates
//! only to an attuned point, and forged / locked / active-battle requests move
//! nobody. Needs the private opening content like `tests/role_enforcement.rs`.
#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, GameplayState, Outcome},
};
use std::path::PathBuf;

const SANCTUARY: [f32; 3] = [-1414.636, -27.631, 351.567];
const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];

fn content() -> PathBuf {
    std::env::var_os("OSFR_WARP_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}

fn character(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn game(state: Option<GameplayState>) -> Gameplay {
    let mut game = Gameplay::load(&content(), state).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game
}

fn travel(id: i32) -> Vec<u8> {
    let mut inner = 90u16.to_le_bytes().to_vec();
    inner.extend(id.to_le_bytes());
    inner
}

fn definitions(out: &Outcome) -> Vec<Vec<i32>> {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[57, 0]))
        .map(|reply| {
            piece_077::decode_point_of_interest_message(reply)
                .unwrap()
                .iter()
                .map(|row| row.id)
                .collect()
        })
        .collect()
}

fn chat_text(out: &Outcome) -> String {
    out.replies
        .iter()
        .filter(|reply| reply.starts_with(&[15, 0, 3, 0]))
        .map(|reply| {
            let length = u32::from_le_bytes(reply[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&reply[8..8 + length]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Attunes Sanctuary the way a player does: the server sees them nearby.
fn attune_sanctuary(game: &mut Gameplay) {
    let out = game.on_location(&character(SANCTUARY), 1_000).unwrap();
    assert!(out.changed, "a new attunement is saved");
    assert_eq!(definitions(&out), [vec![5]], "57 refresh lists the new stone");
    assert!(chat_text(&out).contains("attuned to the Sanctuary Warpstone"));
}

#[test]
fn world_entry_lists_only_attuned_stones_and_attunement_survives_relog() {
    let mut fresh = game(None);
    let entry = fresh.initialize_world(&character([0.0; 3]), 500).unwrap();
    assert_eq!(definitions(&entry), [Vec::<i32>::new()]);
    assert!(fresh.state().warpstones.is_none(), "nothing saved before a visit");

    attune_sanctuary(&mut fresh);
    let json = serde_json::to_string(&fresh.state()).unwrap();
    assert!(json.contains(r#""warpstones":{"attuned":[5]}"#));

    // Relog: the saved state loads, world entry lists the stone, travel works.
    let state: GameplayState = serde_json::from_str(&json).unwrap();
    let mut relogged = game(Some(state));
    let entry = relogged.initialize_world(&character([0.0; 3]), 500).unwrap();
    assert_eq!(definitions(&entry), [vec![5]]);
    let out = relogged
        .handle_in_world(&travel(5), &character(LAKESHORE), 10_000)
        .unwrap();
    let relocation = out.relocation.expect("travel after relog");
    assert_eq!(relocation.position, SANCTUARY);
}

#[test]
fn travel_goes_only_to_attuned_points_and_forged_requests_move_nobody() {
    let mut game = game(None);
    attune_sanctuary(&mut game);
    let from = character(LAKESHORE);
    for id in [9, 0, -1, 99_999, i32::MIN] {
        let out = game.handle_in_world(&travel(id), &from, 5_000).unwrap();
        assert!(out.relocation.is_none(), "id {id}");
        assert!(
            chat_text(&out).contains("have not attuned"),
            "id {id} gets feedback"
        );
    }
    // A body of the wrong size is an error, not a silent no-op.
    assert!(
        game.handle_in_world(&[90, 0, 1, 0], &from, 5_000).is_err(),
        "malformed 90"
    );
    let ok = game.handle_in_world(&travel(5), &from, 6_000).unwrap();
    assert_eq!(ok.relocation.unwrap().position, SANCTUARY);
    assert!(!ok.changed, "travel saves nothing itself");
}

#[test]
fn travel_is_refused_in_other_zones_and_during_a_battle() {
    let mut game = game(None);
    attune_sanctuary(&mut game);

    let mut cave = character(LAKESHORE);
    cave.zone_id = 411;
    let out = game.handle_in_world(&travel(5), &cave, 5_000).unwrap();
    assert!(out.relocation.is_none());
    assert!(chat_text(&out).contains("open world"));

    let at_lakeshore = character(LAKESHORE);
    game.handle_in_world(&chat::local_request("realm battle 411"), &at_lakeshore, 6_000)
        .unwrap();
    let out = game
        .handle_in_world(&travel(5), &at_lakeshore, 7_000)
        .unwrap();
    assert!(out.relocation.is_none(), "an active battle blocks travel");
    assert!(chat_text(&out).contains("Finish what you are doing"));
}

#[test]
fn friend_teleport_is_refused_until_the_player_registry_supplies_friends() {
    let mut game = game(None);
    let mut inner = 58u16.to_le_bytes().to_vec();
    inner.extend(7u64.to_le_bytes());
    let out = game
        .handle_in_world(&inner, &character(LAKESHORE), 5_000)
        .unwrap();
    assert!(out.relocation.is_none());
    assert!(chat_text(&out).contains("cannot teleport to that player"));
}

#[test]
fn destination_request_answers_with_the_current_list() {
    let mut game = game(None);
    attune_sanctuary(&mut game);
    let out = game
        .handle_in_world(&[56, 0], &character(LAKESHORE), 5_000)
        .unwrap();
    assert_eq!(definitions(&out), [vec![5]]);
}
