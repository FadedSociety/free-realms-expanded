//! T663: "Postman: Achieve Level 10" (3130017), the one Seaside / Thunder Falls
//! quest of the data-pass wiring lists that the server can run end to end today.
//!
//! The record is content only (`data/content/quests/postman.json`): Enve LeStamp
//! (9000000080, a drawn `legacy_data` Seaside row) offers it, one `job_level`
//! objective (T361) completes from the server's own Postman rank, and Enve's
//! 49/13 end screen pays the reward row's coins once. No packet, handler or
//! selector is new, so no wire is re-traced beyond the readers T361 and T466 cite:
//! 49/1 `0x00c7bb60`, 49/9 `0x00c7af60`, 49/13 `QuestEndPacket::ReadFields`
//! `0x00c7b990` (actor `6..14`, quest `14..18`, end text `18..22`, reward bundle
//! `0x008e7930`), 49/14 `0x00a864c0`.
//!
//! Every test drives the production in-world router through
//! `tests/support/npc_quest_chain.rs`. The reward is read from the data, never
//! pinned here. The Postman level target is (R) server policy: the client text
//! names level 10 but the client never sends a rank.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    CHARACTER, Fixture, accept, assert_no_offer, assert_packet_changes_nothing, character_at_for,
    coins, content_dir, content_position, counts, end_screen_reward, finish_quest, has_family,
    objective_complete, offer, pending_turnin, profile_xp, quest_end_reply, quest_reply,
    quest_status, read_json, release_end_screen, saved, seed_active_profile, send, talk,
};
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    npc_interaction::decode_quest_offer_49_1,
    quest_engine::QuestStatus,
};

const ENVE: u64 = 9_000_000_080;
const QUEST: u32 = 3_130_017;
const POSTMAN: i32 = 4;
const ADVENTURER: i32 = 1;
const LEVEL_GATE: u32 = 10;
/// Title, intro and description ids the 49/1 card carries (client locale, T466 shape).
const CARD: [u32; 3] = [387_935, 387_934, 387_936];
const END_TEXT: u32 = 387_937;

fn enve() -> CharacterState {
    character_at_for(CHARACTER, 0, content_position("seaside.json", ENVE))
}

/// The coins the quest's reward row pays, read from the shipped record.
fn reward_coins() -> u32 {
    let postman = read_json(&content_dir().join("quests/postman.json"));
    let record = postman["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_u64() == Some(u64::from(QUEST)))
        .expect("the record ships");
    assert!(record["enabled"].as_bool().unwrap_or(true), "enabled");
    record["rewards"][0]["coins"].as_u64().expect("coins") as u32
}

fn player(fixture: &Fixture, active: i32) -> Gameplay {
    let mut state = fixture.game(None).state();
    seed_active_profile(&mut state, active, &[ADVENTURER, POSTMAN]);
    fixture.game(Some(state))
}

fn realm_level(game: &mut Gameplay, level: u32, now: u64) -> Outcome {
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    send(
        game,
        &enve(),
        &chat::local_request(&format!("realm level {level}")),
        now,
    )
}

fn accept_gate(game: &mut Gameplay, now: &mut u64) -> Outcome {
    let offered = offer(game, &enve(), ENVE, QUEST, *now);
    let card = offered
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 1, Some(QUEST)))
        .and_then(|packet| decode_quest_offer_49_1(packet).ok())
        .expect("the 49/1 card decodes");
    assert_eq!(
        [
            card.name_string_id,
            card.intro_text_string_id,
            card.description_string_id
        ],
        CARD,
        "49/1 title, intro and description"
    );
    *now += 10;
    let accepted = accept(game, &enve(), QUEST, *now);
    *now += 10;
    accepted
}

/// The final Talk, the end screen and the 49/14 settle: coins from the data, no job XP.
fn turn_in(game: &mut Gameplay, now: &mut u64) {
    talk(game, &enve(), ENVE, *now);
    assert_eq!(pending_turnin(game), Some(QUEST));
    let end = release_end_screen(game, &enve(), QUEST, ENVE, END_TEXT, *now + 2);
    assert_eq!(end_screen_reward(&end), (reward_coins(), 0));
    finish_quest(game, &enve(), QUEST, POSTMAN, reward_coins(), 0, *now + 4);
    *now += 10;
}

#[test]
fn a_postman_runs_the_level_ten_gate_and_is_paid_once() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, POSTMAN);
    let mut now = 1_000;
    accept_gate(&mut game, &mut now);
    assert_eq!(counts(&game, QUEST), [0], "rank 1 is short of the gate");

    let level = realm_level(&mut game, LEVEL_GATE, now);
    now += 10;
    assert!(
        objective_complete(&level, QUEST, 1),
        "reaching the rank sends the existing 49/10"
    );
    assert_eq!(counts(&game, QUEST), [1]);

    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, POSTMAN));
    turn_in(&mut game, &mut now);
    assert_eq!(coins(&game), coins_before + reward_coins());
    assert_eq!(profile_xp(&game, POSTMAN), xp_before, "the row pays no XP");

    let done = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &enve(),
        &quest_end_reply(QUEST),
        now,
        "a duplicate 49/14",
    );
    assert_no_offer(&mut game, &enve(), ENVE, &mut now, "Enve after the gate");
    assert_eq!(saved(&game), done);
}

#[test]
fn short_of_the_gate_nothing_completes_or_pays() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, POSTMAN);
    let mut now = 1_000;
    accept_gate(&mut game, &mut now);

    realm_level(&mut game, LEVEL_GATE - 1, now);
    now += 10;
    assert_eq!(counts(&game, QUEST), [0], "one rank short never credits");

    // A Talk at Enve with the goal open neither ends the quest nor offers another.
    assert_no_offer(&mut game, &enve(), ENVE, &mut now, "Enve one rank short");
    assert_eq!(pending_turnin(&game), None);
    let before = saved(&game);
    assert_packet_changes_nothing(
        &mut game,
        &enve(),
        &quest_end_reply(QUEST),
        now,
        "an early 49/14",
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(saved(&game), before);
}

#[test]
fn another_job_is_not_offered_the_postman_gate() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, ADVENTURER);
    let mut now = 1_000;
    assert_no_offer(&mut game, &enve(), ENVE, &mut now, "Enve for an Adventurer");
    let forged = quest_reply(QUEST, true);
    assert_packet_changes_nothing(&mut game, &enve(), &forged, now, "a forged 49/2");
}

#[test]
fn the_gate_survives_a_reconnect() {
    let fixture = Fixture::new();
    let mut game = player(&fixture, POSTMAN);
    let mut now = 1_000;
    accept_gate(&mut game, &mut now);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(counts(&game, QUEST), [0]);
    realm_level(&mut game, LEVEL_GATE, now);
    now += 10;
    assert_eq!(counts(&game, QUEST), [1]);

    // The held turn-in (end screen shown, 49/14 not yet sent) also survives a save.
    talk(&mut game, &enve(), ENVE, now);
    release_end_screen(&mut game, &enve(), QUEST, ENVE, END_TEXT, now + 2);
    let mut game = fixture.reload_serialized(&game);
    assert_eq!(pending_turnin(&game), Some(QUEST));
    finish_quest(
        &mut game,
        &enve(),
        QUEST,
        POSTMAN,
        reward_coins(),
        0,
        now + 10,
    );
}

#[test]
fn accepting_at_the_gate_rank_completes_at_once() {
    let fixture = Fixture::new();
    let mut ranked = player(&fixture, POSTMAN);
    realm_level(&mut ranked, LEVEL_GATE, 500);
    let mut game = fixture.reload(&ranked);
    let mut now = 1_000;

    let accepted = accept_gate(&mut game, &mut now);
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(objective_complete(&accepted, QUEST, 1));
    turn_in(&mut game, &mut now);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Completed);
}
