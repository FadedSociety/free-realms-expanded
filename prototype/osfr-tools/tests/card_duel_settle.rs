//! L935: a Skirmish or Scenario win the card window reports (`AccountCommand_RecordSinglePlayerGame`,
//! class 487; spec lab/autopilot/opus/minigame-notes/L935-tcg-protocol.md) credits the Games-window row
//! the player launched, once, and pays the duel purse: the row's MiniGameData id raises
//! `FinishMinigame` for the Card Duelist quest chain, the purse is the launch block's coins plus the
//! Card Duelist `activity_xp` of `job_xp_curve.json`.
//!
//! The rows are the shipped Games-window rows: activity 499 is "Poe's Tournament Deck" (MiniGameData
//! 507, the id the shipped quest 3040011 asks for) and activity 491 "Poe's Trick Deck" (504).

#[path = "support/card_duelist.rs"]
mod card_duelist;

use card_duelist::{
    CARD_DUELIST, CHARACTER, Fixture, POE, POES_BEST_DECK, accept, at, card_duelist_as, coins,
    counts, fixture_with, offer, profile_xp, quest_status, send,
};
use osfr_tools::{
    card_game::Win,
    card_service::SinglePlayerRecord,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use std::{fs, path::Path};

const POES_TOURNAMENT_ROW: u32 = 499;
const POES_TRICK_ROW: u32 = 491;
const WIN: i32 = SinglePlayerRecord::WON;
const SKIRMISH: i32 = 3;
const SCENARIO: i32 = 2;
const TUTORIAL: i32 = 1;
/// The shipped `launch.duel` purse and the Card Duelist rate in `job_xp_curve.json`.
const WIN_COINS: u32 = 25;
const DUEL_XP: u32 = 466;
const FLOOR_MS: u64 = 60_000;

fn play(activity: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(activity.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

fn client(sub: u8, instance: i32) -> Vec<u8> {
    let mut packet = vec![39, 0, sub];
    for value in [instance, -1, -1] {
        packet.extend(value.to_le_bytes());
    }
    packet
}

/// The run's instance: the id inside the 39/16 offer a Play answers.
fn instance_of(out: &Outcome) -> i32 {
    let offer = out
        .replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("Play offers the card run (39/16)");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

fn record(phase: i32, game_type: i32, result: i32, started: i32) -> SinglePlayerRecord {
    SinglePlayerRecord {
        account: 1,
        phase,
        game_type,
        result,
        started,
    }
}

fn start(game: &mut Gameplay, game_type: i32, started: i32, now: u64) {
    let record = record(SinglePlayerRecord::STARTED, game_type, 0, started);
    let out = game.card_single_player(CHARACTER, &record, now).unwrap();
    assert!(out.is_none(), "a start record pays nothing");
}

fn finish(
    game: &mut Gameplay,
    game_type: i32,
    result: i32,
    started: i32,
    now: u64,
) -> Option<(Win, Outcome)> {
    let record = record(SinglePlayerRecord::ENDED, game_type, result, started);
    game.card_single_player(CHARACTER, &record, now).unwrap()
}

/// A Card Duelist on Poe's quest 3040011 (asks for MiniGameData 507) who has pressed Play on `row`
/// and opened the card window (the 39/37 ticket request).
fn launched(row: u32) -> (Fixture, Gameplay) {
    let fixture = fixture_with(&[POES_BEST_DECK]);
    // The release curve file carries the job's `activity_xp`, as in the other L927 suites.
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release/job_xp_curve.json"),
        fixture.path().join("job_xp_curve.json"),
    )
    .unwrap();
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let poe = at(POE);
    offer(&mut game, &poe, POE, POES_BEST_DECK, 2_000);
    accept(&mut game, &poe, POES_BEST_DECK, 2_010);
    let played = send(&mut game, &poe, &play(row), 3_000);
    let instance = instance_of(&played);
    let ticket = send(&mut game, &poe, &client(37, instance), 3_100);
    assert!(
        ticket.replies.iter().any(|p| p.starts_with(&[39, 0, 0x28])),
        "the launch ticket answers the 39/37"
    );
    // The fixture's content directory is deleted with it, and the game reads it when it pays XP.
    (fixture, game)
}

#[test]
fn a_skirmish_win_credits_the_launched_row_and_pays_the_purse() {
    let (_fixture, mut game) = launched(POES_TOURNAMENT_ROW);
    let (coins_before, xp_before) = (coins(&game), profile_xp(&game, CARD_DUELIST));
    start(&mut game, SKIRMISH, 1_793_000_000, 10_000);
    assert_eq!(counts(&game, POES_BEST_DECK), [0], "a start is not a win");

    let (win, out) = finish(
        &mut game,
        SKIRMISH,
        WIN,
        1_793_000_000,
        10_000 + FLOOR_MS + 5_000,
    )
    .expect("a win after the floor settles");
    assert_eq!((win.minigame_id, win.elapsed_seconds), (507, 65));
    assert!(out.changed, "the credit and the purse are saved");
    assert_eq!(
        counts(&game, POES_BEST_DECK),
        [1],
        "MiniGameData 507 was credited"
    );
    assert_eq!(quest_status(&game, POES_BEST_DECK), QuestStatus::Active);
    assert_eq!(coins(&game) - coins_before, WIN_COINS);
    assert_eq!(profile_xp(&game, CARD_DUELIST) - xp_before, DUEL_XP);
    assert!(
        !out.replies.is_empty(),
        "the reward toast and the quest update reach the player"
    );

    // The client repeating its end record, or a replayed start and end, pays nothing more.
    let (coins_after, xp_after) = (coins(&game), profile_xp(&game, CARD_DUELIST));
    assert!(finish(&mut game, SKIRMISH, WIN, 1_793_000_000, 200_000).is_none());
    start(&mut game, SKIRMISH, 1_793_000_000, 210_000);
    assert!(finish(&mut game, SKIRMISH, WIN, 1_793_000_000, 300_000).is_none());
    assert_eq!(coins(&game), coins_after);
    assert_eq!(profile_xp(&game, CARD_DUELIST), xp_after);
}

/// A deck picked from its NPC's menu is the same run as a Games-window row: the launch survives
/// the selection, its window gets a ticket, and its win credits the quest.
#[test]
fn a_deck_picked_from_its_npc_launches_and_credits_the_quest() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/card_game.json");
    let content: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let row = content["rows"].as_array().unwrap().iter().find(|row| row["minigame_id"] == 507);
    let label = row.expect("Poe's Tournament Deck has a row")["name_id"].as_u64().unwrap() as u32;
    let fixture = fixture_with(&[POES_BEST_DECK]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let poe = at(POE);
    offer(&mut game, &poe, POE, POES_BEST_DECK, 2_000);
    accept(&mut game, &poe, POES_BEST_DECK, 2_010);

    let menu = card_duelist::interact(&mut game, &poe, POE, 3_000);
    let choice = card_duelist::menu_option(&menu, label);
    let picked = send(&mut game, &poe, &card_duelist::npc_select(POE, choice), 3_010);
    let instance = instance_of(&picked);
    let ticket = send(&mut game, &poe, &client(37, instance), 3_100);
    assert!(
        ticket.replies.iter().any(|p| p.starts_with(&[39, 0, 0x28])),
        "the run survived the menu: its 39/37 gets a ticket"
    );

    start(&mut game, SKIRMISH, 1_793_000_000, 10_000);
    let (win, _) = finish(&mut game, SKIRMISH, WIN, 1_793_000_000, 10_000 + FLOOR_MS + 5_000)
        .expect("a win after the floor settles");
    assert_eq!(win.minigame_id, 507);
    assert_eq!(counts(&game, POES_BEST_DECK), [1], "MiniGameData 507 was credited");
}

#[test]
fn a_scenario_win_credits_the_row_it_was_launched_from() {
    let (_fixture, mut game) = launched(POES_TRICK_ROW);
    start(&mut game, SCENARIO, 1_793_000_100, 10_000);
    let coins_before = coins(&game);
    let (win, out) = finish(&mut game, SCENARIO, WIN, 1_793_000_100, 100_000).expect("settles");
    assert!(out.changed);
    // Row 491 is MiniGameData 504 (Poe's Trick Deck), not the 507 the quest asks for.
    assert_eq!(win.minigame_id, 504);
    assert_eq!(counts(&game, POES_BEST_DECK), [0]);
    assert_eq!(
        coins(&game) - coins_before,
        WIN_COINS,
        "the win still pays the purse"
    );
}

#[test]
fn losses_early_wins_and_tutorials_change_nothing() {
    let (_fixture, mut game) = launched(POES_TOURNAMENT_ROW);
    let before = (coins(&game), profile_xp(&game, CARD_DUELIST));
    for (kind, result, started, seconds) in [
        (SKIRMISH, 2, 1_793_000_001_i32, 600_u64),
        (SKIRMISH, WIN, 1_793_000_002, 30),
        (TUTORIAL, WIN, 1_793_000_003, 600),
    ] {
        let at = u64::from(started.unsigned_abs());
        start(&mut game, kind, started, at * 1_000);
        assert!(
            finish(
                &mut game,
                kind,
                result,
                started,
                at * 1_000 + seconds * 1_000
            )
            .is_none(),
            "type {kind} result {result} after {seconds} s"
        );
    }
    assert_eq!(counts(&game, POES_BEST_DECK), [0]);
    assert_eq!(
        (coins(&game), profile_xp(&game, CARD_DUELIST)),
        before,
        "nothing was paid"
    );
}

#[test]
fn a_win_without_a_launched_card_window_pays_nothing() {
    let fixture = fixture_with(&[POES_BEST_DECK]);
    let mut game = card_duelist_as(&fixture, CHARACTER);
    let before = (coins(&game), profile_xp(&game, CARD_DUELIST));
    start(&mut game, SKIRMISH, 1_793_000_000, 10_000);
    assert!(finish(&mut game, SKIRMISH, WIN, 1_793_000_000, 100_000).is_none());
    assert_eq!((coins(&game), profile_xp(&game, CARD_DUELIST)), before);
}
