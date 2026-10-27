//! T543: multi-line quest talk conversations with player responses and quiz retries.
//! F810: the review blockers (range per answer, the end screen after a finished
//! conversation) and the gateway-level close, switch and abandon paths.
//!
//! Every test drives the real 26/8 -> 26/10 -> 26/3 -> 26/6 path through the shared
//! `tests/support/npc_quest_chain.rs` walkers (`ZoneQuests`, `Fixture::session`) and the real
//! split quest content. The full Seaside offer/visit/turn-in chain is pinned by
//! `zone_quests_seaside.rs`; these tests seed the quest at its conversation step instead of
//! replaying it. Client evidence: the 26/3 reader `0x00a9ef10` (`ShowDialogFromPacket`
//! `0x009f5ae0` copies each row's first dword into the open dialog's click list) and the 26/6
//! sender `ChoiceResponseByIndex` `0x009f6aa0` (constructor `0x009f5890`), which writes the
//! chosen row's first dword, the same id these tests send. The 49/13 reader `0x00c7b990`
//! reads the actor at `6..14`; `ShowQuestEndScreen` `0x00a95420` resolves it through
//! `0x00984960`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, ZoneQuest, ZoneQuests, coins, counts, dialogue_body,
    dialogue_choice, dialogue_closed, dialogue_line, finish_quest, gateway_send, gateway_talk,
    has_family, npc_interact, objective_complete, pending_turnin, profile_xp, quest_abandon,
    quest_status, saved, seed_quest, send, zone_player,
};
use osfr_tools::{gameplay::Gameplay, quest_engine::QuestStatus};
use serde_json::json;
use std::fs;

const SPRITZY: u64 = 9_500_000_629;
const COLE: u64 = 9_500_000_567;
const BRENNA: u64 = 9_500_000_562;
const YANCY: u64 = 9_500_000_645;
const TAZINNA: u64 = 9_500_000_633;
const EMUZZ: u64 = 9_500_000_580;
const OTHER: u64 = CHARACTER + 1;

const NOMI: u64 = 9_500_000_509;
/// Hasti's content row and the extra spawn the client clicks (`npc_spawns_extra.json`).
const HASTI: u64 = 9_500_001_008;
const HASTI_SPAWN: u64 = 100_000_003_016;

/// The fork's coin reward on every quest here (R).
const COINS: u32 = 30;

const SEASIDE: ZoneQuests = ZoneQuests {
    default_file: "seaside.json",
    files: &[],
    runtime: &[],
};
const SANCTUARY: ZoneQuests = ZoneQuests {
    default_file: "sanctuary.json",
    files: &[],
    runtime: &[(HASTI, HASTI_SPAWN)],
};

const WELCOME: ZoneQuest = ZoneQuest {
    id: 3_000_320,
    giver: SPRITZY,
    card: [93_283, 93_282, 93_284],
    steps: &[(SPRITZY, 93_316)],
    turnin: COLE,
    end_text: 93_335,
    coins: COINS,
    xp: 20,
    profile_xp: 20,
};
const TESTING: ZoneQuest = ZoneQuest {
    id: 3_000_323,
    giver: COLE,
    card: [93_447, 93_449, 93_454],
    steps: &[(BRENNA, 93_464)],
    turnin: COLE,
    end_text: 93_451,
    coins: COINS,
    xp: 30,
    profile_xp: 30,
};
const FEMININE: ZoneQuest = ZoneQuest {
    id: 3_000_360,
    giver: YANCY,
    card: [101_400, 101_408, 101_410],
    steps: &[(TAZINNA, 101_412), (EMUZZ, 101_414)],
    turnin: YANCY,
    end_text: 101_411,
    coins: COINS,
    xp: 20,
    profile_xp: 20,
};
const LITTLE_BROTHER: ZoneQuest = ZoneQuest {
    id: 3_000_301,
    giver: NOMI,
    card: [46_280, 46_279, 46_621],
    steps: &[(HASTI, 140_679)],
    turnin: NOMI,
    end_text: 73_239,
    coins: 20,
    xp: 20,
    profile_xp: 20,
};

/// A player with `quest` active at `counts`, as its giver and earlier steps would have left it.
fn seeded(fixture: &Fixture, guid: u64, quest: &ZoneQuest, counts: &[u32]) -> Gameplay {
    let mut state = zone_player(fixture, guid, ADVENTURER).state();
    seed_quest(&mut state, quest.id, QuestStatus::Active, counts);
    fixture.game_as(guid, Some(state))
}

fn answer_labels(rows: &[[u32; 5]]) -> Vec<u32> {
    rows.iter().map(|row| row[2]).collect()
}

fn answer_rows(rows: &[[u32; 5]]) -> Vec<u32> {
    rows.iter().map(|row| row[0]).collect()
}

/// One Talk at Brenna opens the quiz; it does not complete the step.
fn start_quiz(game: &mut Gameplay, guid: u64, now: u64) {
    let talked = SEASIDE.talk(game, guid, BRENNA, now);
    assert_eq!(
        dialogue_body(&talked),
        93_464,
        "the quiz starts at its first line"
    );
    assert!(
        !talked.changed,
        "complete_after_dialogue must not complete on the Talk"
    );
    assert_eq!(counts(game, TESTING.id), [0, 0]);
}

/// The right answers: continue, the Diggy question's row 2, its "correct" continue, then
/// the Seaspray question's row 3 (which ends the quiz).
const RIGHT: [(i32, u32); 3] = [(1, 93_466), (2, 93_470), (1, 93_474)];

#[test]
fn brennas_quiz_needs_the_right_answers() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let now = 1_000;

    let started = SEASIDE.talk(&mut game, CHARACTER, BRENNA, now);
    let (text, rows) = dialogue_line(&started.replies);
    assert_eq!(text, 93_464);
    assert!(!started.changed, "the Talk does not complete the step");
    assert_eq!(answer_rows(&rows), [1]);
    assert_eq!(answer_labels(&rows), [93_465]);

    let question = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 1);
    let (text, rows) = dialogue_line(&question.replies);
    assert_eq!(text, 93_466, "the first question");
    assert_eq!(answer_rows(&rows), [1, 2, 3]);
    assert_eq!(answer_labels(&rows), [21_311, 21_302, 93_337]);

    // A wrong answer plays the try-again line and changes no saved state.
    let before = saved(&game);
    let wrong = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 2);
    let (text, rows) = dialogue_line(&wrong.replies);
    assert_eq!(text, 93_471, "the try-again line");
    assert_eq!(answer_labels(&rows), [93_472]);
    assert_eq!(saved(&game), before, "a wrong answer changed the save");

    // Its continue row re-asks the same question.
    let again = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 3);
    assert_eq!(dialogue_body(&again), 93_466, "the retry re-asks");

    let correct = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 2, now + 4);
    assert_eq!(dialogue_body(&correct), 93_470, "the correct answer's line");
    assert_eq!(counts(&game, TESTING.id), [0, 0], "not the end yet");

    let next = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 5);
    let (text, rows) = dialogue_line(&next.replies);
    assert_eq!(text, 93_474, "the second question");
    assert_eq!(answer_labels(&rows), [93_475, 93_476, 93_477]);

    let before = saved(&game);
    let wrong = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 2, now + 6);
    assert_eq!(dialogue_body(&wrong), 93_479, "the second try-again line");
    assert_eq!(saved(&game), before, "a wrong answer changed the save");
    let again = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 7);
    assert_eq!(dialogue_body(&again), 93_474, "the retry re-asks");

    let before = saved(&game);
    let finished = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 3, now + 8);
    assert!(dialogue_closed(&finished.replies), "the last answer closes");
    assert!(
        objective_complete(&finished, TESTING.id, 1),
        "the step completes only after the last line"
    );
    assert_eq!(counts(&game, TESTING.id), [1, 0]);
    assert_ne!(saved(&game), before, "the finished quiz did not save");
    assert!(
        finished
            .replies
            .iter()
            .all(|packet| !has_family(packet, 49, 3, None) && !has_family(packet, 49, 4, None)),
        "a conversation adds and completes no quest"
    );

    // Cole's turn-in works after the quiz.
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    SEASIDE.turn_in(&mut game, CHARACTER, &TESTING, ADVENTURER, &mut (now + 20));
    assert_eq!(coins(&game), coins_before + COINS);
    assert_eq!(profile_xp(&game, ADVENTURER), xp_before + 30);
    assert_eq!(quest_status(&game, TESTING.id), QuestStatus::Completed);
}

#[test]
fn spritzy_lines_play_after_the_step_completes() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &WELCOME, &[1, 0, 0]);
    let now = 1_000;

    let talked = SEASIDE.talk(&mut game, CHARACTER, SPRITZY, now);
    assert!(talked.changed, "the step completes on the first Talk");
    assert_eq!(counts(&game, WELCOME.id), [1, 1, 0]);
    assert!(
        objective_complete(&talked, WELCOME.id, 2),
        "the talk completes objective 2"
    );
    let (text, rows) = dialogue_line(&talked.replies);
    assert_eq!(text, 93_316, "her first line plays after the step");
    assert_eq!(answer_labels(&rows), [93_320, 93_322, 93_323]);

    let before = saved(&game);
    let answered = SEASIDE.answer(&mut game, CHARACTER, SPRITZY, 1, now + 1);
    let (text, rows) = dialogue_line(&answered.replies);
    assert_eq!(text, 93_324, "the chosen reply's follow-up line");
    assert_eq!(
        answer_labels(&rows),
        [103_085],
        "the last-line fallback label"
    );

    let finished = SEASIDE.answer(&mut game, CHARACTER, SPRITZY, 1, now + 2);
    assert!(dialogue_closed(&finished.replies), "the follow-up ends it");
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
    let after = SEASIDE.answer(&mut game, CHARACTER, SPRITZY, 300, now + 3);
    assert!(dialogue_closed(&after.replies));
    assert!(!after.changed);
    assert_eq!(saved(&game), before);
    assert_eq!(counts(&game, WELCOME.id), [1, 1, 0]);
}

#[test]
fn feminine_touch_replies_play_after_each_step() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &FEMININE, &[0, 0, 0]);
    let now = 1_000;

    let tazinna = SEASIDE.talk(&mut game, CHARACTER, TAZINNA, now);
    assert!(tazinna.changed);
    let (text, rows) = dialogue_line(&tazinna.replies);
    assert_eq!((text, answer_labels(&rows)), (101_412, vec![101_413]));
    let before = saved(&game);
    let done = SEASIDE.answer(&mut game, CHARACTER, TAZINNA, 1, now + 1);
    assert!(dialogue_closed(&done.replies));
    assert_eq!(saved(&game), before, "her reply changed the save");
    assert_eq!(counts(&game, FEMININE.id), [1, 0, 0]);

    let emuzz = SEASIDE.talk(&mut game, CHARACTER, EMUZZ, now + 10);
    assert!(emuzz.changed);
    let (text, rows) = dialogue_line(&emuzz.replies);
    assert_eq!((text, answer_labels(&rows)), (101_414, vec![101_415]));
    let before = saved(&game);
    let second = SEASIDE.answer(&mut game, CHARACTER, EMUZZ, 1, now + 11);
    let (text, rows) = dialogue_line(&second.replies);
    assert_eq!((text, answer_labels(&rows)), (101_416, vec![103_597]));
    let done = SEASIDE.answer(&mut game, CHARACTER, EMUZZ, 1, now + 12);
    assert!(dialogue_closed(&done.replies));
    assert_eq!(saved(&game), before, "his replies changed the save");
    assert_eq!(counts(&game, FEMININE.id), [1, 1, 0]);
}

#[test]
fn disconnect_mid_quiz_restarts_it() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let now = 1_000;
    start_quiz(&mut game, CHARACTER, now);
    let question = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 1);
    assert_eq!(dialogue_body(&question), 93_466);

    let mut back = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&back, TESTING.id), QuestStatus::Active);
    assert_eq!(counts(&back, TESTING.id), [0, 0], "the step is still open");
    assert_eq!(pending_turnin(&back), None);
    let restarted = SEASIDE.talk(&mut back, CHARACTER, BRENNA, now + 20);
    let (text, rows) = dialogue_line(&restarted.replies);
    assert_eq!(text, 93_464, "the reconnect restarts at the first line");
    assert_eq!(answer_labels(&rows), [93_465]);

    // The quiz still needs its right answers after the restart.
    for (row, text) in RIGHT {
        let next = SEASIDE.answer(&mut back, CHARACTER, BRENNA, row, now + 21);
        assert_eq!(dialogue_body(&next), text);
    }
    let finished = SEASIDE.answer(&mut back, CHARACTER, BRENNA, 3, now + 24);
    assert!(objective_complete(&finished, TESTING.id, 1));
    assert_eq!(counts(&back, TESTING.id), [1, 0]);
}

#[test]
fn unknown_choice_closes_without_change() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let now = 1_000;
    start_quiz(&mut game, CHARACTER, now);
    let question = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, now + 1);
    assert_eq!(dialogue_body(&question), 93_466);

    let before = saved(&game);
    let unknown = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 99, now + 2);
    assert!(
        dialogue_closed(&unknown.replies),
        "an unknown answer closes"
    );
    assert!(!unknown.changed, "an unknown answer changed gameplay");
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
    let later = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 2, now + 3);
    assert!(dialogue_closed(&later.replies));
    assert!(!later.changed);
    assert_eq!(saved(&game), before);

    // The step is still open and a new Talk restarts the quiz.
    start_quiz(&mut game, CHARACTER, now + 4);
}

#[test]
fn quest_offer_choice_still_works() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let mut now = 1_000;
    let talked = SEASIDE.talk(&mut game, CHARACTER, SPRITZY, now);
    let (_, rows) = dialogue_line(&talked.replies);
    assert_eq!(
        answer_labels(&rows),
        [93_283, 8_075],
        "the quest row and Goodbye, unchanged"
    );
    assert_eq!(answer_rows(&rows), [301, 300]);
    SEASIDE.accept(&mut game, CHARACTER, &WELCOME, &mut now);
    assert_eq!(quest_status(&game, WELCOME.id), QuestStatus::Active);
    // A fresh Talk with no open step at Spritzy still builds the ordinary single
    // line dialog, not a conversation.
    let again = SEASIDE.talk(&mut game, CHARACTER, SPRITZY, now + 20);
    let (text, rows) = dialogue_line(&again.replies);
    assert_eq!(text, 93_282, "Spritzy's greeting body");
    assert_eq!(answer_rows(&rows), [300], "only the Goodbye row");
}

#[test]
fn bad_dialogue_data_is_refused() {
    let fixture = Fixture::minimal(CHARACTER);
    let catalogue = fixture.quest_catalogue();

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
        let original = fixture.patch_quest("zone_seaside.json", WELCOME.id, |record| {
            record["objectives"][1]["lines"] = lines;
        });
        let mut game = Gameplay::load(fixture.path(), None).expect("opening fixture loads");
        let error = game
            .bind_npc_quest_catalogue(&catalogue, CHARACTER)
            .expect_err(label);
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData, "{label}");
        fs::write(catalogue.join("quests/zone_seaside.json"), &original).unwrap();
    }

    // The restored file binds again, so the refusals were about the data.
    let mut game = Gameplay::load(fixture.path(), None).expect("opening fixture loads");
    game.bind_npc_quest_catalogue(&catalogue, CHARACTER)
        .expect("the restored quest content binds");
}

#[test]
fn two_players_quiz_independently() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut a = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let mut b = seeded(&fixture, OTHER, &TESTING, &[0, 0]);

    let b_before = saved(&b);
    start_quiz(&mut a, CHARACTER, 1_000);
    for (row, text) in RIGHT {
        let next = SEASIDE.answer(&mut a, CHARACTER, BRENNA, row, 1_001);
        assert_eq!(dialogue_body(&next), text);
    }
    assert_eq!(saved(&b), b_before, "A's quiz changed B");
    let finished = SEASIDE.answer(&mut a, CHARACTER, BRENNA, 3, 1_004);
    assert!(objective_complete(&finished, TESTING.id, 1));
    assert_eq!(counts(&a, TESTING.id), [1, 0]);
    assert_eq!(counts(&b, TESTING.id), [0, 0], "B did not answer A's quiz");

    // B's own quiz is untouched and still needs its right answers.
    start_quiz(&mut b, OTHER, 1_010);
    for (row, text) in RIGHT {
        let next = SEASIDE.answer(&mut b, OTHER, BRENNA, row, 1_011);
        assert_eq!(dialogue_body(&next), text);
    }
    let finished = SEASIDE.answer(&mut b, OTHER, BRENNA, 3, 1_014);
    assert!(objective_complete(&finished, TESTING.id, 1));
    assert_eq!(counts(&a, TESTING.id), [1, 0], "B's quiz changed A");
    let (a_coins, b_coins) = (coins(&a), coins(&b));
    SEASIDE.turn_in(&mut a, CHARACTER, &TESTING, ADVENTURER, &mut 2_000);
    SEASIDE.turn_in(&mut b, OTHER, &TESTING, ADVENTURER, &mut 2_000);
    assert_eq!(coins(&a), a_coins + COINS);
    assert_eq!(coins(&b), b_coins + COINS);
}

/// F810: the start gate checked range once; every later answer repeats it. A player who walked
/// or zoned away cannot keep answering, and the last answer from afar completes nothing.
#[test]
fn answers_from_out_of_reach_end_the_quiz() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let brenna = SEASIDE.at(CHARACTER, BRENNA);
    let mut walked = brenna.clone();
    walked.position[0] += 40.0;
    let mut zoned = brenna.clone();
    zoned.zone_id = 5;

    for (label, away) in [("walked", walked), ("zoned", zoned)] {
        let mut now = 1_000;
        start_quiz(&mut game, CHARACTER, now);
        // Up to the last question, in reach.
        for (row, text) in RIGHT {
            now += 10;
            let next = SEASIDE.answer(&mut game, CHARACTER, BRENNA, row, now);
            assert_eq!(dialogue_body(&next), text, "{label}");
        }
        // The last, right answer from afar ends the quiz and completes nothing.
        let before = saved(&game);
        let far = send(&mut game, &away, &dialogue_choice(3), now + 10);
        assert!(dialogue_closed(&far.replies), "{label}: the dialog closes");
        assert!(!far.changed, "{label}");
        assert_eq!(saved(&game), before, "{label}: an away answer saved");
        assert_eq!(counts(&game, TESTING.id), [0, 0], "{label}");
        assert!(
            far.replies
                .iter()
                .all(|packet| !has_family(packet, 49, 4, None)
                    && !packet.starts_with(&[26, 0, 3, 0])),
            "{label}: nothing but the close"
        );

        // Back in reach the cursor is gone: the same answer closes and credits nothing.
        let later = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 3, now + 20);
        assert!(dialogue_closed(&later.replies), "{label}");
        assert_eq!(counts(&game, TESTING.id), [0, 0], "{label}");
        assert_eq!(saved(&game), before, "{label}");
    }

    // A fresh Talk in reach still works and finishes the quiz.
    start_quiz(&mut game, CHARACTER, 5_000);
    for (row, text) in RIGHT {
        let next = SEASIDE.answer(&mut game, CHARACTER, BRENNA, row, 5_010);
        assert_eq!(dialogue_body(&next), text);
    }
    let finished = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 3, 5_020);
    assert!(objective_complete(&finished, TESTING.id, 1));
}

/// F810: a `complete_after_dialogue` conversation on the last step opens the 49/13 end screen
/// when it finishes. Like the held Talk turn-in it names the actor the client clicked (an
/// extra spawn's runtime GUID, not the content key) and no 26/4 closes the dialog first.
/// Nomi's Little Brother is cut down to Hasti's conversation, so Hasti is its last step.
#[test]
fn a_finished_final_conversation_opens_the_end_screen_at_the_clicked_actor() {
    let fixture = Fixture::minimal(CHARACTER);
    fixture.patch_quest("zone_sanctuary.json", LITTLE_BROTHER.id, |record| {
        let objectives = record["objectives"].as_array_mut().unwrap();
        objectives.truncate(1);
        objectives[0]["complete_after_dialogue"] = json!(true);
        objectives[0]["lines"] = json!([{"text_id": 140_679}]);
    });
    let mut game = zone_player(&fixture, CHARACTER, ADVENTURER);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, ADVENTURER));
    let mut now = 1_000;
    SANCTUARY.accept(&mut game, CHARACTER, &LITTLE_BROTHER, &mut now);

    let talked = SANCTUARY.talk(&mut game, CHARACTER, HASTI, now);
    assert_eq!(dialogue_body(&talked), 140_679, "the conversation starts");
    assert!(!talked.changed, "the Talk does not complete the step");
    assert_eq!(pending_turnin(&game), None);

    let finished = SANCTUARY.answer(&mut game, CHARACTER, HASTI, 1, now + 1);
    assert!(finished.changed);
    assert_eq!(pending_turnin(&game), Some(LITTLE_BROTHER.id));
    let end_screen = finished
        .replies
        .iter()
        .find(|packet| has_family(packet, 49, 13, Some(LITTLE_BROTHER.id)))
        .expect("the finished conversation opens the end screen");
    assert_eq!(
        end_screen[6..14],
        HASTI_SPAWN.to_le_bytes(),
        "the 49/13 names the clicked runtime actor, not the content key"
    );
    assert!(
        !dialogue_closed(&finished.replies),
        "the end screen opens over the dialog, as the held Talk path does"
    );

    finish_quest(
        &mut game,
        &SANCTUARY.at(CHARACTER, HASTI),
        LITTLE_BROTHER.id,
        ADVENTURER,
        LITTLE_BROTHER.coins,
        LITTLE_BROTHER.profile_xp,
        now + 4,
    );
    assert_eq!(coins(&game), coins_before + LITTLE_BROTHER.coins);
    assert_eq!(
        profile_xp(&game, ADVENTURER),
        xp_before + LITTLE_BROTHER.profile_xp
    );
}

/// F810: a real 26/23 of the conversation's own quest ends it; a refused one changes nothing.
#[test]
fn abandoning_the_quest_ends_its_conversation_only_when_it_abandons() {
    let fixture = Fixture::minimal(CHARACTER);
    let mut game = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]);
    let brenna = SEASIDE.at(CHARACTER, BRENNA);
    start_quiz(&mut game, CHARACTER, 1_000);
    let question = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, 1_001);
    assert_eq!(dialogue_body(&question), 93_466);

    // An abandon of a quest that is not active changes nothing, the quiz goes on.
    let before = saved(&game);
    let refused = send(&mut game, &brenna, &quest_abandon(WELCOME.id), 1_002);
    assert!(!refused.changed);
    assert_eq!(saved(&game), before, "a refused abandon changed the save");
    let still = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 2, 1_003);
    assert_eq!(dialogue_body(&still), 93_470, "the quiz survived");

    let abandoned = send(&mut game, &brenna, &quest_abandon(TESTING.id), 1_004);
    assert!(abandoned.changed, "the real abandon took");
    let dead = SEASIDE.answer(&mut game, CHARACTER, BRENNA, 1, 1_005);
    assert!(dialogue_closed(&dead.replies), "no quiz left to continue");
    assert!(
        dead.replies
            .iter()
            .all(|packet| !packet.starts_with(&[26, 0, 3, 0])),
        "an answer after the abandon shows no line"
    );
}

/// F810: the 26/4 close and another NPC's 26/8 each end the conversation, and the staged clone
/// that holds the cleared cursor is committed by the real `Gateway::handle_at` although the
/// outcome carries no change (a gateway that dropped it would let the next 26/6 continue).
#[test]
fn close_and_another_menu_end_the_conversation_through_the_gateway() {
    let fixture = Fixture::minimal(CHARACTER);
    let seed = seeded(&fixture, CHARACTER, &TESTING, &[0, 0]).state();
    let at = SEASIDE.at(CHARACTER, BRENNA).position;
    let quiz_right = |gateway: &mut osfr_tools::gateway::Gateway, now: u64| {
        let replies = gateway_talk(gateway, BRENNA, now);
        assert_eq!(dialogue_line(&replies).0, 93_464);
        let replies = gateway_send(gateway, &dialogue_choice(1), now + 2);
        assert_eq!(dialogue_line(&replies).0, 93_466, "the question is open");
    };

    // Control: without a close the same answer continues the quiz.
    let mut gateway = fixture.session(CHARACTER, at, Some(seed.clone()));
    quiz_right(&mut gateway, 1_000);
    let next = gateway_send(&mut gateway, &dialogue_choice(2), 1_010);
    assert_eq!(dialogue_line(&next).0, 93_470, "the cursor is live");

    // 26/4: the player closes the dialog; the next answer finds no conversation.
    let mut gateway = fixture.session(CHARACTER, at, Some(seed.clone()));
    quiz_right(&mut gateway, 1_000);
    gateway_send(&mut gateway, &[26, 0, 4, 0], 1_010);
    let dead = gateway_send(&mut gateway, &dialogue_choice(2), 1_020);
    assert!(dialogue_closed(&dead), "the close was committed");
    assert!(
        dead.iter()
            .all(|packet| !packet.starts_with(&[26, 0, 3, 0]))
    );

    // 26/8 at Cole (within reach of Brenna): his menu ends the conversation.
    let mut gateway = fixture.session(CHARACTER, at, Some(seed.clone()));
    quiz_right(&mut gateway, 1_000);
    let menu = gateway_send(&mut gateway, &npc_interact(COLE), 1_010);
    assert!(menu.iter().any(|packet| packet.starts_with(&[26, 0, 9, 0])));
    let dead = gateway_send(&mut gateway, &dialogue_choice(2), 1_020);
    assert!(
        dead.iter()
            .all(|packet| !packet.starts_with(&[26, 0, 3, 0]))
    );
    let last = gateway_send(&mut gateway, &dialogue_choice(3), 1_030);
    assert!(last.iter().all(|packet| !has_family(packet, 49, 10, None)));
    assert_eq!(
        counts(gateway.gameplay().unwrap(), TESTING.id),
        [0, 0],
        "answers after the switch completed nothing"
    );

    // 26/23 of the quest ends the conversation too.
    let mut gateway = fixture.session(CHARACTER, at, Some(seed));
    quiz_right(&mut gateway, 1_000);
    gateway_send(&mut gateway, &quest_abandon(TESTING.id), 1_010);
    let dead = gateway_send(&mut gateway, &dialogue_choice(2), 1_020);
    assert!(
        dead.iter()
            .all(|packet| !packet.starts_with(&[26, 0, 3, 0]))
    );
}
