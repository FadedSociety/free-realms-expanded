use super::*;
use std::path::PathBuf;

fn content_directory() -> PathBuf {
    std::env::var_os("OSFR_OPENING_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-opening-cooking-original-assets")
        })
}
fn character() -> CharacterState {
    CharacterState {
        guid: 7,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0; 3],
    }
}
fn quit(instance: i32) -> Vec<u8> {
    let mut b = vec![39, 0, 6];
    for n in [instance, -1, -1] {
        b.extend(n.to_le_bytes());
    }
    b
}
fn remove(instance: i32) -> Vec<u8> {
    let mut b = vec![39, 0, 19];
    for n in [instance, -1, 46] {
        b.extend(n.to_le_bytes());
    }
    b
}
fn snapshot(game: &Gameplay) -> serde_json::Value {
    serde_json::to_value(game.state()).unwrap()
}

#[test]
fn ordinary_cooking_quit_cancels_atomically_and_malformed_replays_roll_back() {
    let content = content_directory();
    assert!(content.join("cooking-campaign.json").is_file());
    let mut game = Gameplay::load(&content, None).unwrap();
    let character = character();
    game.cooking.as_mut().unwrap().begin(45).unwrap();
    let instance = game
        .cooking
        .as_ref()
        .unwrap()
        .state()
        .active
        .as_ref()
        .unwrap()
        .instance_id;
    let before = snapshot(&game);
    let out = game
        .handle_in_world(&quit(instance), &character, 1_000)
        .unwrap();
    let after = snapshot(&game);
    assert_eq!(out.replies, vec![remove(instance)]);
    assert!(after["cooking"]["active"].is_null());
    assert_eq!(before["inventory"], after["inventory"]);
    assert_eq!(before["quests"], after["quests"]);
    assert_eq!(before["opening"], after["opening"]);
    assert_eq!(
        before["cooking"]["completed"],
        after["cooking"]["completed"]
    );
    assert_eq!(
        before["cooking"]["next_instance_id"],
        after["cooking"]["next_instance_id"]
    );

    // A replay of the stale packet is not a second cancellation.
    let replay = game
        .handle_in_world(&quit(instance), &character, 2_000)
        .unwrap();
    assert!(replay.replies.is_empty());
    assert_eq!(snapshot(&game), after);

    // A malformed recognized packet rejects before the staged bundle commits.
    game.cooking.as_mut().unwrap().begin(45).unwrap();
    let live = snapshot(&game);
    let mut malformed = quit(instance + 1);
    malformed.push(0);
    assert!(game.handle_in_world(&malformed, &character, 3_000).is_err());
    assert_eq!(snapshot(&game), live);
}
