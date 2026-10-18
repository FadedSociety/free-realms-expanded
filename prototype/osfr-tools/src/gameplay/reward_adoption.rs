use super::*;
use crate::fishing_service::{FISHERMAN_PROFILE, FishingSpots};
use serde_json::json;
use std::path::PathBuf;

const SPOTS_JSON: &str = include_str!("../../data/content/fishing_spots.json");
const FISHING_GATE: u64 = 9_000_000_181;
const FISHING_T0: u64 = 1_000_000;
const FISHING_ACTOR: crate::fishing::Handle = crate::fishing::Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn game() -> Gameplay {
    let path = reference();
    assert!(
        path.join("opening.json").is_file(),
        "reward adoption requires the private opening reference at {}",
        path.display()
    );
    Gameplay::load(&path, None).expect("opening reference loads")
}

fn character() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.; 3],
        rotation: [0., 0., 1.],
    }
}

fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    // Common header writer 0x00b69550 emits the family and subtype words.
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

fn actor_bytes() -> Vec<u8> {
    [
        FISHING_ACTOR.id.to_le_bytes(),
        FISHING_ACTOR.seq.to_le_bytes(),
    ]
    .concat()
}

fn register() -> Vec<u8> {
    // Client writer 0x00b6a400: header, handle, boolean.
    let mut body = actor_bytes();
    body.push(0);
    request(2, &body)
}

fn cast() -> Vec<u8> {
    // Client writer 0x00b6a460: header, handle, vector, boolean.
    let mut body = actor_bytes();
    for coordinate in [10.0f32, 0.0, 0.0, 1.0] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    request(6, &body)
}

fn reel() -> Vec<u8> {
    // Client writer 0x00b6a4d0: header, handle, boolean.
    let mut body = actor_bytes();
    body.push(1);
    request(7, &body)
}

fn start(game: &mut Gameplay, at: u64) {
    let instance = game.fishing.instance_id(1).expect("fishing round");
    let mut packet = vec![39, 0, 5];
    packet.extend(instance.to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    packet.extend((-1i32).to_le_bytes());
    let outcome = game
        .handle_in_world(&packet, &character(), at)
        .expect("fishing lifecycle starts");
    assert!(!outcome.replies.is_empty());
    game.handle_in_world(&register(), &character(), at + 1)
        .expect("fishing registration");
}

fn begin(game: &mut Gameplay) {
    let mut spots: serde_json::Value = serde_json::from_str(SPOTS_JSON).unwrap();
    for sizes in spots["items"].as_object_mut().unwrap().values_mut() {
        for item in sizes.as_array_mut().unwrap() {
            if item.as_i64().unwrap() > 0 {
                *item = json!(2884);
            }
        }
    }
    game.fishing_spots = Some(Arc::new(
        FishingSpots::parse(&serde_json::to_vec(&spots).unwrap()).unwrap(),
    ));
    let outcome = game
        .begin_fishing(FISHING_GATE, &character())
        .expect("fishing begins");
    assert!(outcome.changed);
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .unlocked
            .contains(&FISHERMAN_PROFILE)
    );
    start(game, FISHING_T0);
}

fn catch(game: &mut Gameplay, at: u64) -> io::Result<Outcome> {
    game.handle_in_world(&cast(), &character(), at + 10)?;
    game.tick(&character(), at + 1_510)?;
    game.tick(&character(), at + 2_410)?;
    game.handle_in_world(&reel(), &character(), at + 2_420)
}

fn item_count(game: &Gameplay, definition_id: i32) -> u32 {
    game.inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn fishing_catch_reloads_unlocked_fisherman_profile() {
    let mut game = game();
    begin(&mut game);
    let active = game.profiles.as_ref().unwrap().active();
    assert!(
        game.profiles
            .as_ref()
            .unwrap()
            .state()
            .experience
            .get(&FISHERMAN_PROFILE)
            .is_none()
    );
    game.inventory.grant(0, 2884, 3).expect("live item grant");
    let held = item_count(&game, 2884);

    let outcome = catch(&mut game, FISHING_T0 + 10_000).expect("paid catch");
    assert!(outcome.changed);
    let profiles = game.profiles.as_ref().unwrap();
    assert_eq!(profiles.active(), active, "fishing does not change jobs");
    assert_eq!(
        profiles.state().experience.get(&FISHERMAN_PROFILE),
        Some(&10),
        "the catch reloads the Fisherman XP snapshot"
    );
    assert_eq!(
        item_count(&game, 2884),
        held + 1,
        "adoption keeps items granted after the round began"
    );
}

#[test]
fn fishing_catch_without_profile_content_changes_no_reward_state() {
    let mut game = game();
    begin(&mut game);
    game.profile_content_directory = None;
    let before = (
        game.inventory.state().clone(),
        game.inventory.state().coins,
        game.profiles.as_ref().unwrap().state().clone(),
        game.profiles.as_ref().unwrap().active(),
    );

    let error = match catch(&mut game, FISHING_T0 + 10_000) {
        Ok(_) => panic!("profile reload is required"),
        Err(error) => error,
    };
    assert_eq!(error.to_string(), "minigame XP needs profile content");
    assert_eq!(
        (
            game.inventory.state().clone(),
            game.inventory.state().coins,
            game.profiles.as_ref().unwrap().state().clone(),
            game.profiles.as_ref().unwrap().active(),
        ),
        before,
        "a refused catch adopts neither inventory nor profile state"
    );
}
