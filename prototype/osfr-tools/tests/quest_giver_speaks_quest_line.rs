//! Clicking a quest NPC opens its quest window: no wheel, no empty dialogue first
//! (`XzhxhHmpXfs` 0-2 s), and an offer left open never locks a character out.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, character_at_for, content_position, interact, talk, zone_player,
};

const DARREL: u64 = 9_500_000_118;

fn offers(replies: &[Vec<u8>]) -> usize {
    let offer = |packet: &&Vec<u8>| packet.starts_with(&[49, 0, 1, 0]);
    replies.iter().filter(offer).count()
}

#[test]
fn a_lone_action_skips_the_wheel_and_a_giver_opens_on_its_quest_window() {
    let fixture = Fixture::minimal(CHARACTER).with_bulk_quests();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    game.open_quests_directly();
    let character = character_at_for(CHARACTER, 0, content_position("briarwood.json", DARREL));

    let menu = interact(&mut game, &character, DARREL, 1_000);
    let list = menu
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("the click answers 26/9");
    assert_eq!(list[12], 1, "the 26/9 auto-select byte (client popup +0x18)");

    let talked = talk(&mut game, &character, DARREL, 1_100);
    assert_eq!(offers(&talked.replies), 1);
    assert!(
        !talked.replies.iter().any(|packet| packet.starts_with(&[26, 0, 3, 0])),
        "no dialogue before the quest window"
    );
}

#[test]
fn an_offer_left_open_is_offered_again_in_session_and_after_a_reload() {
    let fixture = Fixture::minimal(CHARACTER).with_bulk_quests();
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    game.open_quests_directly();
    let character = character_at_for(CHARACTER, 0, content_position("briarwood.json", DARREL));

    assert_eq!(offers(&talk(&mut game, &character, DARREL, 1_000).replies), 1);
    // Escape closes the client's window with no reply; the next click just offers again.
    assert_eq!(offers(&talk(&mut game, &character, DARREL, 2_000).replies), 1);

    // The game closed with the window open: the next session is not locked out.
    let saved = game.state();
    let mut back = fixture.game_as(CHARACTER, Some(saved));
    back.open_quests_directly();
    assert_eq!(offers(&talk(&mut back, &character, DARREL, 3_000).replies), 1);
}
