//! T788: a `perform_pet_trick` quest objective credits when the server accepts the owner's pet
//! trick.
//!
//! Everything here runs through the real request path: `Gameplay::handle_in_world` (the gateway's
//! game handler) reads the 53/7 `PetPerformTrick` body (one u32 trick id, client
//! `Pet.PerformTrick` `00c06890` -> `DispatchPetActionRecord` `00b5fb30` ->
//! `UnlockPetTricksAfterGesture` `00cc7420` -> `sub_B5FF50`) and `Gameplay::pet_tick` runs the
//! trick's session. The pet roster comes from `adopt_pets`; the quest catalogue is the shipped
//! split content, copied into the fixture and with the Pet Trainer quest under test enabled (the
//! record ships disabled: its reward is unrecovered). No pet trick packet byte changes.
//!
//! The objective credits the *accepted* request only. The client-computed grade never reaches the
//! server, so a refused 53/7 (unknown trick, no summoned pet, a blocked pet) and a late 53/8 credit
//! nothing; the tests assert the serialized quest state is byte-equal after every refusal.

#[path = "support/pet_care.rs"]
mod care;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;

use npc_quest_chain::{
    ADVENTURER, CHARACTER, Fixture, accept, character_at_for, content_dir, counts, finish_quest,
    offer, pending_turnin, quest_status, read_json, release_end_screen, saved, send, talk,
};
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    pets::{PetContent, PetTrickAnimation},
    quest_engine::QuestStatus,
};
use pet_wire::{perform_trick, summon};
use serde_json::json;
use std::fs;

/// The disabled Pet Trainer record under test: Roscoe's dog trick "Sit Up".
const QUEST: u32 = 3_120_007;
/// Roscoe in `data/content/npcs/sanctuary.json`; his extra spawn keeps the same runtime guid
/// (`npc_spawns_extra.json` `spawn_guid`), so the menu actor is the content NPC id.
const GIVER: u64 = 9_000_000_184;
/// The quest's end text (locale 90033): "Great! You're becoming a better pet trainer!".
const END_TEXT: u32 = 90033;
/// `data/content/pets.json` trick 11, name_id 62 -> "Sit Up"; the dog model rows carry it.
const TRICK: u32 = 11;
/// Trick 12 "Roll Over": also a dog trick, but not the one this quest names.
const OTHER_TRICK: u32 = 12;
/// A trick id no `pets.json` row has.
const UNKNOWN_TRICK: u32 = 999_999;
/// The bundled dog's model (`data/content/pets.json` models row 364).
const DOG_MODEL: i32 = 364;
/// A chess activity row in the shipped `data/content/chess.json`; entering it closes the
/// occupation table's pets gate.
const CHESS_ROW: u32 = 20;
/// A moment well after every pet's adoption, so needs and care clocks are live.
const START: u64 = 6_000_000;

/// A fixture whose copied split catalogue has the Pet Trainer record under test enabled.
///
/// The shipped record stays `enabled: false` because its reward rows are unrecovered; enabling it
/// here is the smallest way to drive the production loader and the real dispatch. `count` rewrites
/// the one `perform_pet_trick` step's required count (the duplicate/complete tests need 2 and 1).
fn fixture(count: Option<u32>) -> Fixture {
    let fixture = Fixture::new();
    fixture.overlay_content(&content_dir());
    // The reference opening inventory has none of the harvesting board's reward rows; this
    // slice never starts a gather, so drop the overlayed opt-in board (the T503 fixture rule).
    let harvesting = fixture.path().join("harvesting.json");
    if harvesting.is_file() {
        fs::remove_file(&harvesting).expect("drop the unused harvesting board");
    }
    let catalogue = fixture.quest_catalogue();
    let path = catalogue.join("quests/pet_trainer.json");
    let mut document = read_json(&path);
    let record = document["records"]
        .as_array_mut()
        .expect("pet_trainer records")
        .iter_mut()
        .find(|record| record["id"].as_u64() == Some(u64::from(QUEST)))
        .expect("the shipped Pet Trainer record exists");
    record["enabled"] = json!(true);
    if let Some(count) = count {
        for goal in record["objectives"]
            .as_array_mut()
            .expect("the record has objectives")
        {
            if goal["kind"].as_str() == Some("perform_pet_trick") {
                goal["count"] = json!(count);
            }
        }
    }
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).expect("quest JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    fixture
}

/// The content row position Roscoe is talked to at. The player stands at the row, so the 12 m
/// interaction radius always holds; the row's regional zone label is inside the one world zone.
fn giver_position() -> [f32; 3] {
    let document = read_json(&content_dir().join("npc_spawns_extra.json"));
    let row = document["records"]
        .as_array()
        .expect("npc_spawns_extra records")
        .iter()
        .find(|row| row["spawn_guid"].as_u64() == Some(GIVER))
        .expect("Roscoe has an extra spawn");
    serde_json::from_value(row["position"].clone()).expect("Roscoe has a position")
}

/// A game that owns one adopted dog and has accepted the quest under test.
fn armed(count: Option<u32>) -> (Fixture, Gameplay, CharacterState, u32) {
    let fixture = fixture(count);
    let (service, pet) = care::dog();
    let mut game = fixture.game(None);
    game.adopt_pets(service.document_value())
        .expect("the dog roster loads");
    let who = character_at_for(CHARACTER, 0, giver_position());
    let mut now = START;
    offer(&mut game, &who, GIVER, QUEST, now);
    accept(&mut game, &who, QUEST, now + 2);
    now += 10;
    assert_eq!(counts(&game, QUEST), [0], "the trick step starts at 0");
    let summoned = send(&mut game, &who, &summon(pet), now);
    assert!(
        summoned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[53, 0, 9])),
        "the dog is summoned (53/9 PetActive, reader 00b556c0)"
    );
    (fixture, game, who, pet)
}

/// The trick's shipped animation row on the dog model.
fn trick_row(trick: u32) -> PetTrickAnimation {
    PetContent::bundled()
        .expect("pets.json loads")
        .model(DOG_MODEL)
        .expect("the dog model row")
        .trick_animations
        .iter()
        .find(|row| row.trick_id == trick)
        .unwrap_or_else(|| panic!("the dog has an animation row for trick {trick}"))
        .clone()
}

/// When the running trick has fully ended: its pet animation delay plus its run time.
fn trick_end(started: u64) -> u64 {
    let row = trick_row(TRICK);
    started + u64::from(row.start_delay_ms) + u64::from(row.duration_ms)
}

fn perform(game: &mut Gameplay, who: &CharacterState, trick: u32, now: u64) -> Outcome {
    game.handle_in_world(&perform_trick(trick), who, now)
        .unwrap_or_else(|error| panic!("53/7 for trick {trick} is a handling error: {error}"))
}

/// The server accepted the 53/7: it answered the trick-name line (53/15 kind 1).
fn accepted(outcome: &Outcome) -> bool {
    outcome.replies.iter().any(|packet| {
        packet.len() >= 11
            && packet.starts_with(&[53, 0, 15])
            && u32::from_le_bytes(packet[7..11].try_into().unwrap()) == 1
    })
}

/// Every quest-family packet (49/*) in one outcome.
fn quest_packets(outcome: &Outcome) -> Vec<&Vec<u8>> {
    outcome
        .replies
        .iter()
        .filter(|packet| packet.starts_with(&[49, 0]))
        .collect()
}

/// End the trick at its shipped end time so the pet is free again.
fn end_trick(game: &mut Gameplay, who: &CharacterState, started: u64) {
    let ended = game
        .pet_tick(who, trick_end(started))
        .expect("the pet is still out");
    assert!(
        ended.replies.iter().any(|packet| {
            packet.len() >= 11
                && packet.starts_with(&[53, 0, 15])
                && u32::from_le_bytes(packet[7..11].try_into().unwrap()) == 2
        }),
        "the trick ends with its 53/15 kind 2 result"
    );
}

/// The whole walk: one accepted trick credits, then the held 49/13 and the 49/14 turn it in. The
/// record ships with no reward rows, so the end screen's coin and star words are 0.
fn turn_in(game: &mut Gameplay, who: &CharacterState, now: u64) {
    let talked = talk(game, who, GIVER, now);
    assert!(talked.changed, "the final Talk opens the held end screen");
    assert_eq!(pending_turnin(game), Some(QUEST));
    release_end_screen(game, who, QUEST, GIVER, END_TEXT, now + 2);
    finish_quest(game, who, QUEST, ADVENTURER, 0, 0, now + 4);
    assert_eq!(quest_status(game, QUEST), QuestStatus::Completed);
}

#[test]
fn an_accepted_trick_credits_the_quest_objective_and_the_turnin_pays() {
    let (_fixture, mut game, who, _pet) = armed(None);
    let performed = perform(&mut game, &who, TRICK, START + 20);
    assert!(accepted(&performed), "the dog knows Sit Up and has a row");
    assert_eq!(counts(&game, QUEST), [1], "the accepted trick credits once");
    assert!(performed.changed, "crediting the step marks the save");
    assert!(
        performed.replies.iter().any(|packet| {
            packet.starts_with(&[49, 0, 9, 0])
                && u32::from_le_bytes(packet[14..18].try_into().unwrap()) == 1
        }),
        "the client gets the counted 49/9 update"
    );
    assert!(
        performed
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 10, 0])),
        "the one-count step completes"
    );

    end_trick(&mut game, &who, START + 20);
    turn_in(&mut game, &who, START + 30_000);
}

#[test]
fn credit_does_not_wait_for_the_animation() {
    let (_fixture, mut game, who, _pet) = armed(None);
    let started = START + 100;
    let performed = perform(&mut game, &who, TRICK, started);
    // The credit lands in the same outcome as the accepted request: no tick, no animation and no
    // client timing is involved. The pet animation is due `start_delay_ms` later.
    assert_eq!(counts(&game, QUEST), [1]);
    assert!(quest_packets(&performed).iter().any(|packet| {
        packet.starts_with(&[49, 0, 10, 0]) || packet.starts_with(&[49, 0, 9, 0])
    }));
    assert!(
        !performed.replies.iter().any(|packet| {
            packet.len() >= 11
                && packet.starts_with(&[53, 0, 15])
                && u32::from_le_bytes(packet[7..11].try_into().unwrap()) == 2
        }),
        "the result line only comes when the trick's own timer ends"
    );
    assert!(
        trick_row(TRICK).start_delay_ms > 0,
        "the animation is delayed"
    );
}

#[test]
fn each_accepted_perform_counts_once() {
    let (_fixture, mut game, who, _pet) = armed(Some(2));
    let first = START + 200;
    perform(&mut game, &who, TRICK, first);
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "one accepted perform is one count"
    );

    // A second 53/7 while the trick runs is refused by the pet session and credits nothing.
    let during = START + 300;
    let refused = perform(&mut game, &who, TRICK, during);
    assert!(!accepted(&refused), "a running trick refuses another");
    assert!(
        quest_packets(&refused).is_empty(),
        "the refused perform sends no quest packet"
    );
    assert_eq!(counts(&game, QUEST), [1]);

    end_trick(&mut game, &who, first);
    let second = trick_end(first) + 1;
    let performed = perform(&mut game, &who, TRICK, second);
    assert!(accepted(&performed), "the pet is free again");
    assert_eq!(
        counts(&game, QUEST),
        [2],
        "each accepted perform counts once"
    );
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);

    // A third accepted perform after the step is complete changes nothing.
    end_trick(&mut game, &who, second);
    let before = saved(&game);
    let after = perform(&mut game, &who, TRICK, trick_end(second) + 1);
    assert!(accepted(&after), "the pet still performs the known trick");
    assert!(
        quest_packets(&after).is_empty(),
        "a completed step sends no update"
    );
    assert_eq!(saved(&game), before, "a completed step changes nothing");
}

#[test]
fn perform_after_completion_changes_nothing() {
    let (_fixture, mut game, who, _pet) = armed(None);
    let started = START + 400;
    perform(&mut game, &who, TRICK, started);
    assert_eq!(counts(&game, QUEST), [1]);
    end_trick(&mut game, &who, started);
    let completed = saved(&game);
    let again = perform(&mut game, &who, TRICK, trick_end(started) + 1);
    assert!(accepted(&again), "the trick itself is still allowed");
    assert!(!again.changed, "a completed objective marks no save");
    assert!(
        quest_packets(&again).is_empty(),
        "a completed objective sends no quest packet"
    );
    assert_eq!(saved(&game), completed, "the quest state is unchanged");
}

#[test]
fn other_trick_changes_nothing() {
    let (_fixture, mut game, who, _pet) = armed(None);
    let before = saved(&game);
    let performed = perform(&mut game, &who, OTHER_TRICK, START + 500);
    assert!(
        accepted(&performed),
        "the dog knows Roll Over too, so only the objective can refuse it"
    );
    assert!(
        quest_packets(&performed).is_empty(),
        "a different trick sends no quest packet"
    );
    assert!(!performed.changed);
    assert_eq!(saved(&game), before, "a different trick changes nothing");
}

#[test]
fn unknown_trick_credits_nothing() {
    let (_fixture, mut game, who, _pet) = armed(None);
    let before = saved(&game);
    let refused = perform(&mut game, &who, UNKNOWN_TRICK, START + 600);
    assert!(!accepted(&refused), "no model row declares this trick");
    assert!(
        quest_packets(&refused).is_empty(),
        "an unknown trick sends no quest packet"
    );
    assert!(!refused.changed);
    assert_eq!(saved(&game), before, "an unknown trick changes nothing");
}

#[test]
fn blocked_pet_credits_nothing() {
    let (_fixture, mut game, who, _pet) = armed(None);
    // Enter a shipped chess row through the activity portal's real 167/1 Play packet; chess is an
    // Occupation, so the shared busy table closes the pets gate.
    let mut play = vec![167, 0, 1, 2];
    play.extend(CHESS_ROW.to_le_bytes());
    play.extend(0u32.to_le_bytes());
    let entered = game
        .handle_in_world(&play, &who, START + 700)
        .expect("the activity portal handles the request");
    assert!(
        entered
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 16])),
        "the chess row is offered: {:?}",
        entered.replies
    );

    let before = saved(&game);
    let refused = perform(&mut game, &who, TRICK, START + 710);
    assert!(!accepted(&refused), "a blocked pet refuses the trick");
    assert!(
        quest_packets(&refused).is_empty(),
        "a blocked pet sends no quest packet"
    );
    assert!(!refused.changed);
    assert_eq!(saved(&game), before, "a blocked pet changes nothing");
}

#[test]
fn another_players_pet_does_not_credit_me() {
    let fixture = fixture(None);
    let (service, _pet) = care::dog();
    let mut mine = fixture.game(None);
    mine.adopt_pets(service.document_value()).unwrap();
    let who = character_at_for(CHARACTER, 0, giver_position());
    offer(&mut mine, &who, GIVER, QUEST, START);
    accept(&mut mine, &who, QUEST, START + 2);
    let before = saved(&mine);

    // The other session owns its own dog and performs the same trick at the same spot.
    let other = CHARACTER + 1;
    let (other_service, other_pet) = care::dog();
    let mut theirs = fixture.game_as(other, None);
    theirs
        .adopt_pets(other_service.document_value())
        .expect("the other dog roster loads");
    let their_who = character_at_for(other, 0, giver_position());
    let summoning = send(&mut theirs, &their_who, &summon(other_pet), START + 10);
    assert!(
        summoning
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[53, 0, 9]))
    );
    let performed = perform(&mut theirs, &their_who, TRICK, START + 20);
    assert!(accepted(&performed), "the other player's dog performs");

    assert_eq!(
        saved(&mine),
        before,
        "another player's pet performs on their own session"
    );
    assert_eq!(counts(&mine, QUEST), [0]);
}

#[test]
fn reload_between_trick_and_turnin_keeps_progress() {
    let (fixture, mut game, who, _pet) = armed(None);
    let started = START + 800;
    perform(&mut game, &who, TRICK, started);
    assert_eq!(counts(&game, QUEST), [1]);

    let mut game = fixture.reload_serialized(&game);
    assert_eq!(quest_status(&game, QUEST), QuestStatus::Active);
    assert_eq!(
        counts(&game, QUEST),
        [1],
        "the trick credit is saved progress"
    );

    turn_in(&mut game, &who, START + 40_000);
}
