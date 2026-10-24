//! T474: the consumable effect stack through the real `Gameplay` route: applied with
//! `apply_consumable_effect`, expired by `Gameplay::tick`, cancelled by the client's 35/33
//! (`handle_in_world`), resent after a scene load, and never saved.
use osfr_tools::{
    character_store::CharacterState,
    consumable_effects::{Kind, Refusal, Spec},
    gameplay::{Gameplay, Outcome},
};
use std::path::PathBuf;

const GUID: u64 = 1;
const MIN: u64 = 60_000;

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}
fn who() -> CharacterState {
    CharacterState {
        guid: GUID,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}
fn game() -> Gameplay {
    Gameplay::load(&reference(), None).unwrap()
}
/// A test-only transform: no catalogue entry is needed, the route only sees the `Spec`.
fn transform(duration_ms: u64) -> Spec {
    Spec {
        kind: Kind::Transform {
            model_id: 4321,
            poof_effect_id: 77,
        },
        duration_ms,
        icon_id: 3843,
        name_id: 100,
    }
}
fn heads(packets: &[Vec<u8>]) -> Vec<(u16, u16)> {
    packets
        .iter()
        .map(|p| {
            (
                u16::from_le_bytes([p[0], p[1]]),
                u16::from_le_bytes([p[2], p[3]]),
            )
        })
        .collect()
}
/// The replies that belong to the effect stack (the world tick may add unrelated ones).
fn effect_heads(out: &Outcome) -> Vec<(u16, u16)> {
    heads(&out.replies)
        .into_iter()
        .filter(|h| [(35, 14), (35, 15), (35, 16), (38, 16), (38, 17)].contains(h))
        .collect()
}
/// 35/33: `u16 35, u16 33, i32 id` (client sender `00934740`, serializer `00913b80`).
fn strip(id: i32) -> Vec<u8> {
    [vec![35, 0, 33, 0], id.to_le_bytes().to_vec()].concat()
}
/// The buff id the 38/16 row carries (`i32` tag id right after the header).
fn buff_id(out: &Outcome) -> i32 {
    let row = out.replies.iter().find(|p| p.starts_with(&[38, 0, 16, 0]));
    i32::from_le_bytes(row.unwrap()[4..8].try_into().unwrap())
}

#[test]
fn a_transform_goes_to_the_owner_with_its_row_and_to_neighbours_without_it() {
    let mut game = game();
    let out = game
        .apply_consumable_effect(GUID, who().position, transform(30 * MIN), 1_000)
        .unwrap();
    assert_eq!(heads(&out.replies), [(35, 16), (35, 14), (38, 16)]);
    assert_eq!(heads(&out.nearby), [(35, 16), (35, 14)]);
    assert!(out.changed, "a visible change refreshes the AddPc snapshot");
}

#[test]
fn a_refused_spec_sends_and_changes_nothing() {
    let mut game = game();
    let before = game.consumable_effects().clone();
    assert!(matches!(
        game.apply_consumable_effect(GUID, who().position, transform(0), 0),
        Err(Refusal::Duration)
    ));
    assert_eq!(*game.consumable_effects(), before);
}

#[test]
fn expiry_is_sent_once_by_the_tick() {
    let mut game = game();
    game.apply_consumable_effect(GUID, who().position, transform(MIN), 0)
        .unwrap();
    let early = game.tick(&who(), MIN - 1).unwrap();
    assert!(effect_heads(&early).is_empty() && early.nearby.is_empty());
    let out = game.tick(&who(), MIN).unwrap();
    assert_eq!(effect_heads(&out), [(35, 15), (38, 17)]);
    assert_eq!(heads(&out.nearby), [(35, 15)]);
    assert!(out.changed);
    let again = game.tick(&who(), MIN + 1).unwrap();
    assert!(effect_heads(&again).is_empty() && again.nearby.is_empty() && !again.changed);
}

#[test]
fn the_buff_bar_cancel_ends_only_an_owned_effect() {
    let mut game = game();
    let applied = game
        .apply_consumable_effect(GUID, who().position, transform(30 * MIN), 0)
        .unwrap();
    let id = buff_id(&applied);

    // Another id, a job or power-up tag id and a negative id: nothing is sent, nothing changes.
    let before = game.consumable_effects().clone();
    for other in [id + 1, 2, 15_255, -1, 0] {
        let out = game.handle_in_world(&strip(other), &who(), 5_000).unwrap();
        assert!(out.replies.is_empty() && out.nearby.is_empty() && !out.changed);
        assert_eq!(*game.consumable_effects(), before);
    }

    // The owned id reverts the model and drops the row, for the owner and for neighbours.
    let out = game.handle_in_world(&strip(id), &who(), 5_000).unwrap();
    assert_eq!(effect_heads(&out), [(35, 15), (38, 17)]);
    assert_eq!(heads(&out.nearby), [(35, 15)]);
    assert!(out.changed);

    // A repeat is the unknown-id case; so is the expiry that would have come later.
    let repeat = game.handle_in_world(&strip(id), &who(), 6_000).unwrap();
    assert!(repeat.replies.is_empty() && repeat.nearby.is_empty());
    assert!(effect_heads(&game.tick(&who(), 30 * MIN).unwrap()).is_empty());
}

#[test]
fn a_malformed_cancel_is_an_error_that_changes_nothing() {
    let mut game = game();
    game.apply_consumable_effect(GUID, who().position, transform(30 * MIN), 0)
        .unwrap();
    let before = game.consumable_effects().clone();
    let saved = serde_json::to_string(&game.state()).unwrap();
    for bad in [
        vec![35, 0, 33, 0],
        vec![35, 0, 33, 0, 1, 0, 0],
        [strip(1_000_000_000), vec![0]].concat(),
        [strip(1_000_000_000), vec![0; 4]].concat(),
    ] {
        assert!(game.handle_in_world(&bad, &who(), 1).is_err(), "{bad:?}");
        assert_eq!(*game.consumable_effects(), before);
    }
    assert_eq!(serde_json::to_string(&game.state()).unwrap(), saved);
}

#[test]
fn the_stack_is_session_only_and_follows_a_scene_load() {
    let mut game = game();
    let saved = serde_json::to_string(&game.state()).unwrap();
    game.apply_consumable_effect(GUID, who().position, transform(30 * MIN), 0)
        .unwrap();
    assert_eq!(serde_json::to_string(&game.state()).unwrap(), saved);

    // A scene load rebuilds the client's own actor: model and buff row come again, remaining time.
    let loaded = game.initialize_world(&who(), 10 * MIN).unwrap();
    let heads = heads(&loaded.replies);
    assert!(heads.contains(&(35, 14)) && heads.contains(&(38, 16)));

    // A relog starts with nothing and sends nothing.
    let mut relog = Gameplay::load(&reference(), Some(game.state())).unwrap();
    let login = relog.initialize_world(&who(), 0).unwrap();
    assert!(effect_heads(&login).is_empty());
    assert_eq!(
        *relog.consumable_effects(),
        *Gameplay::load(&reference(), None)
            .unwrap()
            .consumable_effects()
    );
}
