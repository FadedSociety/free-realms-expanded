//! T543: multi-line quest talk conversations with player responses and quiz retries.
//!
//! Every test drives the real 26/8 -> 26/10 -> 26/3 -> 26/6 path in
//! `tests/support/npc_quest_chain.rs` and the real split quest content. Client
//! evidence: the 26/3 reader `0x00a9ef10` (`ShowDialogFromPacket` `0x009f5ae0`
//! copies each row's first dword into the open dialog's click list) and the 26/6
//! sender `ChoiceResponseByIndex` `0x009f6aa0` (constructor `0x009f5890`), which
//! writes the chosen row's first dword, the same id these tests send.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, accept, character_at_for, coins, content_position, counts,
    dialogue_choice, end_screen_reward, finish_quest, has_family, objective_complete, offer,
    pending_turnin, profile_xp, quest_status, release_end_screen, saved, seed_active_profile, send,
    talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    npc_interaction::decode_dialogue_26_3,
    quest_engine::QuestStatus,
};
use serde_json::{Value, json};
use std::fs;

const SPRITZY: u64 = 9_500_000_629;
const COLE: u64 = 9_500_000_567;
const SEASPRAY: u64 = 9_500_000_623;
const DIGGY: u64 = 9_000_000_070;
const BRENNA: u64 = 9_500_000_562;
const OTHER: u64 = CHARACTER + 1;

const WELCOME: u32 = 3_000_320;
const OLDEST: u32 = 3_000_321;
const TOURING: u32 = 3_000_322;
const TESTING: u32 = 3_000_323;

/// The fork's coin reward on every quest here (R).
const COINS: u32 = 30;
const GAZEBO: [f32; 3] = [-690.38, 2.3, -1060.25];
const KOODINI: [f32; 3] = [-379.92, -6.58, -1168.87];
const BOARDWALK: [f32; 3] = [-426.24, -18.72, -1223.69];

fn at(guid: u64, npc: u64) -> CharacterState {
    character_at_for(guid, 0, content_position("seaside.json", npc))
}

fn player(fixture: &Fixture, guid: u64) -> Gameplay {
    let mut state = fixture.game_as(guid, None).state();
    seed_active_profile(&mut state, ADVENTURER, &[ADVENTURER]);
    fixture.game_as(guid, Some(state))
}

/// The 26/3 body and its response rows, decoded with the production reader.
fn line(outcome: &Outcome) -> (u32, Vec<[u32; 5]>) {
    let packet = outcome
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 3, 0]))
        .expect("a conversation step answers 26/3");
    let decoded = decode_dialogue_26_3(packet).expect("the 26/3 decodes");
    (decoded.initial_dwords[0], decoded.choices)
}

fn closed(outcome: &Outcome) -> bool {
    outcome
        .replies
        .iter()
        .any(|packet| packet == &[26, 0, 4, 0])
}

fn answer(game: &mut Gameplay, character: &CharacterState, row: i32, now: u64) -> Outcome {
    send(game, character, &dialogue_choice(row), now)
}

fn answer_rows(rows: &[[u32; 5]]) -> Vec<u32> {
    rows.iter().map(|row| row[0]).collect()
}

fn answer_labels(rows: &[[u32; 5]]) -> Vec<u32> {
    rows.iter().map(|row| row[2]).collect()
}

fn accept_quest(game: &mut Gameplay, guid: u64, giver: u64, quest_id: u32, now: &mut u64) {
    offer(game, &at(guid, giver), giver, quest_id, *now);
    accept(game, &at(guid, giver), quest_id, *now + 2);
    *now += 10;
}

fn turn_in_quest(
    game: &mut Gameplay,
    guid: u64,
    quest_id: u32,
    end_text: u32,
    xp: u32,
    now: &mut u64,
) {
    let cole = at(guid, COLE);
    let talked = talk(game, &cole, COLE, *now);
    assert!(talked.changed, "quest {quest_id}: the final Talk");
    assert_eq!(pending_turnin(game), Some(quest_id));
    let end = release_end_screen(game, &cole, quest_id, COLE, end_text, *now + 2);
    assert_eq!(end_screen_reward(&end), (COINS, xp));
    finish_quest(game, &cole, quest_id, ADVENTURER, COINS, xp, *now + 4);
    *now += 10;
}

/// Walk The Seaside Tour through its real offers and turn-ins, leaving Testing
/// the Tour (`3000323`) accepted at Brenna's quiz step.
fn tour_to_quiz(game: &mut Gameplay, guid: u64, now: &mut u64) {
    accept_quest(game, guid, SPRITZY, WELCOME, now);
    let visited = game
        .on_location(&character_at_for(guid, 0, GAZEBO), *now)
        .expect("the gazebo visit is accepted");
    assert!(visited.changed);
    *now += 10;
    // Spritzy's step completes on this Talk and starts her follow-up lines; the
    // 26/8 at Cole below ends that conversation without completing anything else.
    let talked = talk(game, &at(guid, SPRITZY), SPRITZY, *now);
    assert!(talked.changed);
    *now += 10;
    turn_in_quest(game, guid, WELCOME, 93_335, 20, now);

    accept_quest(game, guid, COLE, OLDEST, now);
    let talked = talk(game, &at(guid, SEASPRAY), SEASPRAY, *now);
    assert!(talked.changed);
    *now += 10;
    turn_in_quest(game, guid, OLDEST, 93_401, 20, now);

    accept_quest(game, guid, COLE, TOURING, now);
    for point in [KOODINI, BOARDWALK] {
        let visited = game
            .on_location(&character_at_for(guid, 0, point), *now)
            .expect("the tour visit is accepted");
        assert!(visited.changed);
        *now += 10;
    }
    let talked = talk(game, &at(guid, DIGGY), DIGGY, *now);
    assert!(talked.changed);
    *now += 10;
    turn_in_quest(game, guid, TOURING, 93_422, 30, now);

    accept_quest(game, guid, COLE, TESTING, now);
}

/// One Talk at Brenna opens the quiz; it does not complete the step.
fn start_quiz(game: &mut Gameplay, guid: u64, now: u64) -> Outcome {
    let talked = talk(game, &at(guid, BRENNA), BRENNA, now);
    assert_eq!(line(&talked).0, 93_464, "the quiz starts at its first line");
    assert!(
        !talked.changed,
        "complete_after_dialogue must not complete on the Talk"
    );
    assert_eq!(counts(game, TESTING), [0, 0]);
    talked
}

#[test]
fn brennas_quiz_needs_the_right_answers() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER);
    let mut now = 1_000;
    tour_to_quiz(&mut game, CHARACTER, &mut now);
    let brenna = at(CHARACTER, BRENNA);

    let started = start_quiz(&mut game, CHARACTER, now);
    let (text, rows) = line(&started);
    assert_eq!(text, 93_464);
    assert_eq!(answer_rows(&rows), [1]);
    assert_eq!(answer_labels(&rows), [93_465]);

    let question = answer(&mut game, &brenna, 1, now + 1);
    let (text, rows) = line(&question);
    assert_eq!(text, 93_466, "the first question");
    assert_eq!(answer_rows(&rows), [1, 2, 3]);
    assert_eq!(answer_labels(&rows), [21_311, 21_302, 93_337]);

    // A wrong answer plays the try-again line and changes no saved state.
    let before = saved(&game);
    let wrong = answer(&mut game, &brenna, 1, now + 2);
    let (text, rows) = line(&wrong);
    assert_eq!(text, 93_471, "the try-again line");
    assert_eq!(answer_labels(&rows), [93_472]);
    assert_eq!(counts(&game, TESTING), [0, 0]);
    assert_eq!(saved(&game), before, "a wrong answer changed the save");

    // Its continue row re-asks the same question.
    let again = answer(&mut game, &brenna, 1, now + 3);
    assert_eq!(line(&again).0, 93_466, "the retry re-asks the question");
    assert_eq!(counts(&game, TESTING), [0, 0]);

    let correct = answer(&mut game, &brenna, 2, now + 4);
    assert_eq!(line(&correct).0, 93_470, "the correct answer's line");
    assert_eq!(
        counts(&game, TESTING),
        [0, 0],
        "the first answer is not the end"
    );

    let next = answer(&mut game, &brenna, 1, now + 5);
    let (text, rows) = line(&next);
    assert_eq!(text, 93_474, "the second question");
    assert_eq!(answer_labels(&rows), [93_475, 93_476, 93_477]);

    let before = saved(&game);
    let wrong = answer(&mut game, &brenna, 2, now + 6);
    assert_eq!(line(&wrong).0, 93_479, "the second try-again line");
    assert_eq!(saved(&game), before, "a wrong answer changed the save");
    let again = answer(&mut game, &brenna, 1, now + 7);
    assert_eq!(
        line(&again).0,
        93_474,
        "the retry re-asks the second question"
    );

    let before = saved(&game);
    let finished = answer(&mut game, &brenna, 3, now + 8);
    assert!(closed(&finished), "the last answer closes the dialog");
    assert!(
        objective_complete(&finished, TESTING, 1),
        "the step completes only after the last line"
    );
    assert_eq!(counts(&game, TESTING), [1, 0]);
    assert_ne!(saved(&game), before, "the finished quiz did not save");
    assert!(
        finished
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, None) && !has_family(packet, 49, 4, None)),
        "a conversation adds and completes no quest"
    );

    // Cole's turn-in works after the quiz.
    let cole = at(CHARACTER, COLE);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let talked = talk(&mut game, &cole, COLE, now + 20);
    assert!(talked.changed, "Cole accepts the finished quest");
    assert_eq!(pending_turnin(&game), Some(TESTING));
    let end = release_end_screen(&mut game, &cole, TESTING, COLE, 93_451, now + 22);
    assert_eq!(end_screen_reward(&end), (COINS, 30));
    finish_quest(&mut game, &cole, TESTING, ADVENTURER, COINS, 30, now + 24);
    assert_eq!(coins(&game), coins_before + COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 30);
    assert_eq!(quest_status(&game, TESTING), QuestStatus::Completed);
}

#[test]
fn spritzy_lines_play_after_the_step_completes() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER);
    let mut now = 1_000;
    accept_quest(&mut game, CHARACTER, SPRITZY, WELCOME, &mut now);
    let visited = game
        .on_location(&character_at_for(CHARACTER, 0, GAZEBO), now)
        .expect("the gazebo visit is accepted");
    assert!(visited.changed);
    now += 10;

    let talked = talk(&mut game, &at(CHARACTER, SPRITZY), SPRITZY, now);
    assert!(talked.changed, "the step completes on the first Talk");
    assert_eq!(counts(&game, WELCOME), [1, 1, 0]);
    assert!(
        objective_complete(&talked, WELCOME, 2),
        "the talk completes objective 2"
    );
    let (text, rows) = line(&talked);
    assert_eq!(text, 93_316, "her first line plays after the step");
    assert_eq!(answer_labels(&rows), [93_320, 93_322, 93_323]);

    let spritzy = at(CHARACTER, SPRITZY);
    let before = saved(&game);
    let answered = answer(&mut game, &spritzy, 1, now + 1);
    let (text, rows) = line(&answered);
    assert_eq!(text, 93_324, "the chosen reply's follow-up line");
    assert_eq!(
        answer_labels(&rows),
        [103_085],
        "the last-line fallback label"
    );

    let finished = answer(&mut game, &spritzy, 1, now + 2);
    assert!(
        closed(&finished),
        "the follow-up line ends the conversation"
    );
    assert!(!finished.changed, "the step already completed on the Talk");
    assert_eq!(saved(&game), before, "the follow-up lines changed the save");
    assert!(
        finished
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, None) && !has_family(packet, 49, 4, None)),
        "follow-up lines add and complete no quest"
    );

    // The dialog is gone; another 26/6 closes nothing and changes nothing.
    let after = answer(&mut game, &spritzy, 300, now + 3);
    assert!(closed(&after));
    assert!(!after.changed);
    assert_eq!(saved(&game), before);
    assert_eq!(counts(&game, WELCOME), [1, 1, 0]);
}

#[test]
fn disconnect_mid_quiz_restarts_it() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER);
    let mut now = 1_000;
    tour_to_quiz(&mut game, CHARACTER, &mut now);
    start_quiz(&mut game, CHARACTER, now);
    let brenna = at(CHARACTER, BRENNA);
    let question = answer(&mut game, &brenna, 1, now + 1);
    assert_eq!(line(&question).0, 93_466);

    let mut back = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&back, TESTING), QuestStatus::Active);
    assert_eq!(counts(&back, TESTING), [0, 0], "the step is still open");
    assert_eq!(pending_turnin(&back), None);
    let around = at(CHARACTER, BRENNA);
    let restarted = talk(&mut back, &around, BRENNA, now + 20);
    let (text, rows) = line(&restarted);
    assert_eq!(text, 93_464, "the reconnect restarts at the first line");
    assert_eq!(answer_labels(&rows), [93_465]);

    // The quiz still needs its right answers after the restart.
    let question = answer(&mut back, &around, 1, now + 21);
    assert_eq!(line(&question).0, 93_466);
    let digest = answer(&mut back, &around, 2, now + 22);
    assert_eq!(line(&digest).0, 93_470);
    let next = answer(&mut back, &around, 1, now + 23);
    assert_eq!(line(&next).0, 93_474);
    let finished = answer(&mut back, &around, 3, now + 24);
    assert!(objective_complete(&finished, TESTING, 1));
    assert_eq!(counts(&back, TESTING), [1, 0]);
}

#[test]
fn unknown_choice_closes_without_change() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER);
    let mut now = 1_000;
    tour_to_quiz(&mut game, CHARACTER, &mut now);
    start_quiz(&mut game, CHARACTER, now);
    let brenna = at(CHARACTER, BRENNA);
    let question = answer(&mut game, &brenna, 1, now + 1);
    assert_eq!(line(&question).0, 93_466);

    let before = saved(&game);
    let unknown = answer(&mut game, &brenna, 99, now + 2);
    assert!(closed(&unknown), "an unknown answer closes the dialog");
    assert!(!unknown.changed, "an unknown answer changed gameplay");
    assert_eq!(counts(&game, TESTING), [0, 0]);
    assert_eq!(saved(&game), before, "an unknown answer changed the save");
    assert!(
        unknown
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, None)
                && !has_family(packet, 49, 4, None)
                && !has_family(packet, 49, 10, None)),
        "an unknown answer completes nothing"
    );

    // The conversation is closed; a later valid row does nothing either.
    let later = answer(&mut game, &brenna, 2, now + 3);
    assert!(closed(&later));
    assert!(!later.changed);
    assert_eq!(saved(&game), before);

    // The step is still open and a new Talk restarts the quiz.
    let restarted = talk(&mut game, &brenna, BRENNA, now + 4);
    assert_eq!(line(&restarted).0, 93_464);
    assert_eq!(counts(&game, TESTING), [0, 0]);
}

#[test]
fn quest_offer_choice_still_works() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = player(&fixture, CHARACTER);
    let now = 1_000;
    let spritzy = at(CHARACTER, SPRITZY);
    let talked = talk(&mut game, &spritzy, SPRITZY, now);
    let (_, rows) = line(&talked);
    assert_eq!(
        answer_labels(&rows),
        [93_283, 8_075],
        "the quest row and Goodbye, unchanged"
    );
    assert_eq!(answer_rows(&rows), [301, 300]);
    let offered = offer(&mut game, &spritzy, SPRITZY, WELCOME, now + 10);
    assert!(
        offered
            .replies
            .iter()
            .any(|packet| has_family(packet, 49, 1, Some(WELCOME)))
    );
    accept(&mut game, &spritzy, WELCOME, now + 12);
    assert_eq!(quest_status(&game, WELCOME), QuestStatus::Active);
    // A fresh Talk with no open step at Spritzy still builds the ordinary single
    // line dialog, not a conversation.
    let again = talk(&mut game, &spritzy, SPRITZY, now + 20);
    let (text, rows) = line(&again);
    assert_eq!(text, 93_282, "Spritzy's greeting body");
    assert_eq!(answer_rows(&rows), [300], "only the Goodbye row");
}

#[test]
fn bad_dialogue_data_is_refused() {
    let fixture = Fixture::minimal(CHARACTER);
    let catalogue = fixture.quest_catalogue();
    let path = catalogue.join("quests/zone_seaside.json");
    let original = fs::read(&path).expect("the fixture copies the quest file");

    let bad = [
        (
            "six choices",
            json!([{
                "text_id": 93_316,
                "choices": (0..6)
                    .map(|_| json!({"response_text_id": 93_320}))
                    .collect::<Vec<_>>(),
            }]),
        ),
        (
            "seventeen lines",
            json!(
                (0..17)
                    .map(|index| json!({"text_id": 1000 + index}))
                    .collect::<Vec<_>>()
            ),
        ),
        (
            "then depth three",
            json!([{
                "text_id": 1,
                "choices": [{
                    "response_text_id": 2,
                    "then": [{
                        "text_id": 3,
                        "choices": [{
                            "response_text_id": 4,
                            "then": [{
                                "text_id": 5,
                                "choices": [{
                                    "response_text_id": 6,
                                    "then": [{"text_id": 7}],
                                }],
                            }],
                        }],
                    }],
                }],
            }]),
        ),
    ];
    for (label, lines) in bad {
        let mut document: Value = serde_json::from_slice(&original).expect("the quest file parses");
        let goal = document["records"]
            .as_array_mut()
            .expect("records")
            .iter_mut()
            .find(|record| record["id"] == json!(WELCOME))
            .and_then(|record| {
                record["objectives"]
                    .as_array_mut()?
                    .iter_mut()
                    .find(|objective| objective["id"] == json!(2))
            })
            .expect("3000320 objective 2");
        goal["lines"] = lines;
        fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
        let mut game = Gameplay::load(fixture.path(), None).expect("opening fixture loads");
        let error = game
            .bind_npc_quest_catalogue(&catalogue, CHARACTER)
            .expect_err(label);
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData, "{label}");
        fs::write(&path, &original).unwrap();
    }

    // The restored file binds again, so the refusals were about the data.
    let mut game = Gameplay::load(fixture.path(), None).expect("opening fixture loads");
    game.bind_npc_quest_catalogue(&catalogue, CHARACTER)
        .expect("the restored quest content binds");
}

#[test]
fn two_players_quiz_independently() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = player(&fixture, CHARACTER);
    let mut b = player(&fixture, OTHER);
    let mut now = 1_000;
    tour_to_quiz(&mut a, CHARACTER, &mut now);
    tour_to_quiz(&mut b, OTHER, &mut now);

    let b_before = saved(&b);
    start_quiz(&mut a, CHARACTER, now);
    let a_brenna = at(CHARACTER, BRENNA);
    let question = answer(&mut a, &a_brenna, 1, now + 1);
    assert_eq!(line(&question).0, 93_466);
    assert_eq!(saved(&b), b_before, "A's quiz changed B");

    let diggy = answer(&mut a, &a_brenna, 2, now + 2);
    assert_eq!(line(&diggy).0, 93_470);
    let next = answer(&mut a, &a_brenna, 1, now + 3);
    assert_eq!(line(&next).0, 93_474);
    let finished = answer(&mut a, &a_brenna, 3, now + 4);
    assert!(objective_complete(&finished, TESTING, 1));
    assert_eq!(counts(&a, TESTING), [1, 0]);
    assert_eq!(counts(&b, TESTING), [0, 0], "B did not answer A's quiz");

    // B's own quiz is untouched and still needs its right answers.
    let b_brenna = at(OTHER, BRENNA);
    let b_started = start_quiz(&mut b, OTHER, now + 10);
    assert_eq!(line(&b_started).0, 93_464);
    let b_question = answer(&mut b, &b_brenna, 1, now + 11);
    assert_eq!(line(&b_question).0, 93_466);
    let b_diggy = answer(&mut b, &b_brenna, 2, now + 12);
    assert_eq!(line(&b_diggy).0, 93_470);
    let b_next = answer(&mut b, &b_brenna, 1, now + 13);
    assert_eq!(line(&b_next).0, 93_474);
    let b_finished = answer(&mut b, &b_brenna, 3, now + 14);
    assert!(objective_complete(&b_finished, TESTING, 1));
    assert_eq!(counts(&b, TESTING), [1, 0]);
    assert_eq!(counts(&a, TESTING), [1, 0], "B's quiz changed A");
    let (a_coins, b_coins) = (coins(&a), coins(&b));
    turn_in_quest(&mut a, CHARACTER, TESTING, 93_451, 30, &mut now);
    turn_in_quest(&mut b, OTHER, TESTING, 93_451, 30, &mut now);
    assert_eq!(coins(&a), a_coins + COINS);
    assert_eq!(coins(&b), b_coins + COINS);
}
