//! Shared soccer match drivers for the suites that play a real match
//! (`soccer_entry`, `soccer_quest_credit`, `soccer_star_chapter`). Test-only.
//!
//! Every helper drives a production seam: `Gameplay::handle_in_world` for the
//! 167/1/2 portal Play, the family-39 lifecycle and the family-76 soccer
//! requests, `Gameplay::initialize_world` for the client's Loaded after the
//! scene change, and `Gameplay::tick` for the authoritative clock whose
//! `tick_world` settles the match. The family-39 envelope is the fishing
//! suites' builder (base constructor `008a4420`: family, subtype, then
//! instance, `-1`, game id).
#![allow(dead_code)]

#[path = "fishing_packets.rs"]
mod fishing_packets;

pub use fishing_packets::lifecycle;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::Gameplay,
    soccer::{self, SoccerPacket},
};

/// 167/1/2 activity-portal Play: activity id, then a zero dword.
pub fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

/// The run instance the server offered in its 39/16.
pub fn offer_instance(replies: &[Vec<u8>]) -> i32 {
    let offer = replies
        .iter()
        .find(|reply| reply.starts_with(&[39, 0, 16]))
        .expect("39/16 soccer offer");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

pub fn position(player: u64, position: [f32; 3]) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerPos {
        player,
        position,
        tail: [0; 45],
    })
}

pub fn shot(player: u64) -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::UpdatePlayerState {
        player,
        state: 17,
        aux: 255,
        state_counter: 1,
        vectors: [[0.0, 0.0, 1.0, 1.0], [0.0; 4], [0.0; 4]],
    })
}

/// A client's own `GoalMade`; the server refuses it.
pub fn client_goal() -> Vec<u8> {
    soccer::encode_inner(&SoccerPacket::GoalMade { fields: [0, 1, 1] })
}

/// Portal Play through Loaded and the start request. Returns the run instance.
pub fn enter_match(game: &mut Gameplay, character: &CharacterState, activity: u32, at: u64) -> i32 {
    let played = game
        .handle_in_world(&play(activity), character, at)
        .unwrap();
    assert!(played.scene_zoning.is_some(), "Play zones into sg_soccer");
    let loaded = game.initialize_world(character, at + 100).unwrap();
    let instance = offer_instance(&loaded.replies);
    let started = game
        .handle_in_world(&lifecycle(5, instance), character, at + 200)
        .unwrap();
    assert!(
        started
            .replies
            .iter()
            .any(|reply| reply.starts_with(&[41, 0, 2, 0])),
        "shared encounter start"
    );
    instance
}

/// The server-scored goal a real client produces: report the pose, let the
/// server step, then send the sub-14 charged shot.
pub fn shoot(game: &mut Gameplay, character: &CharacterState, at: u64) {
    game.handle_in_world(&position(character.guid, [0.0, 0.0, 0.0]), character, at)
        .unwrap();
    game.tick(character, at + 150).unwrap();
    game.handle_in_world(&shot(character.guid), character, at + 150)
        .unwrap();
}

/// Step the authoritative clock until full time settles the match. Returns the
/// settling tick's replies and the tick time. The bound covers the shipped
/// two 180 s halves at the 0.25 s physics step.
pub fn tick_to_full_time(
    game: &mut Gameplay,
    character: &CharacterState,
    mut now: u64,
) -> (Vec<Vec<u8>>, u64) {
    for _ in 0..1_600 {
        now += 250;
        let out = game.tick(character, now).unwrap();
        if out.replies.iter().any(|p| p.starts_with(&[39, 0, 18])) {
            return (out.replies, now);
        }
    }
    panic!("the soccer clock reached full time");
}

/// One full match: Play, start, server goal, full time. Returns the settling
/// tick's replies and its time.
pub fn win_match(
    game: &mut Gameplay,
    character: &CharacterState,
    activity: u32,
    at: u64,
) -> (Vec<Vec<u8>>, u64) {
    enter_match(game, character, activity, at);
    shoot(game, character, at + 200);
    tick_to_full_time(game, character, at + 350)
}
