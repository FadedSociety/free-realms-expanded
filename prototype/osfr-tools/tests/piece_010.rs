//! Trading card game (queue item 010) behavioral regression for
//! `src/restoration/piece_010.rs`.
//!
//! The module is included by path so this file exercises the recovered rules
//! without touching `lib.rs`. Every match here is synthetic: no user save,
//! native client or gateway state is read or written.
//!
//! Recovered expectations come from the shipped client tables and locale
//! (`MiniGameData.txt` TYPE 16 rows and their `en_us` descriptions), the
//! community minigame packet catalogue and the native client's
//! `TradingCardGameHandler`; provenance and limits are recorded in
//! `notes/Research/Pieces/010 Trading card game.md`.
#[path = "../src/restoration/piece_010.rs"]
mod piece_010;

use piece_010::RowKind;
use piece_010::Side::{Challenger, Opponent};
use piece_010::{
    AppliedPoints, Archetype, DeckRow, MINIGAME_BASE_PACKET, MINIGAME_GAME_END_SCORE_PACKET,
    MINIGAME_GAME_OVER_PACKET, MINIGAME_STARS_EARNED_PACKET, MatchError, Phase,
    REQUEST_TCG_CHALLENGE_PACKET, TRADING_CARD_START_GAME_PACKET, TcgMatch, UNDECLARED_DIFFICULTY,
    WIN_SCORE,
};

const HOST: i32 = 10_010_001;

fn deck(id: i32, name: &str, description: &str, difficulty: &str) -> DeckRow {
    DeckRow::parse(id, Some(name), Some(description), difficulty)
}

fn reward_deck(id: i32, name: &str, reward: &str, difficulty: &str) -> DeckRow {
    deck(
        id,
        name,
        &format!(
            "Battle an opponent. Be the first to score 12 points to win the \
             match and a {reward} from his deck."
        ),
        difficulty,
    )
}

#[test]
fn recovered_packet_opcodes_match_the_client_and_catalogue() {
    assert_eq!(MINIGAME_BASE_PACKET, 39);
    assert_eq!(REQUEST_TCG_CHALLENGE_PACKET, 38);
    assert_eq!(TRADING_CARD_START_GAME_PACKET, 40);
    assert_eq!(MINIGAME_GAME_OVER_PACKET, 18);
    assert_eq!(MINIGAME_GAME_END_SCORE_PACKET, 47);
    assert_eq!(MINIGAME_STARS_EARNED_PACKET, 21);
    // 116 of the 124 shipped TYPE 16 rows say "score 12 points".
    assert_eq!(WIN_SCORE, 12);
    assert_eq!(UNDECLARED_DIFFICULTY, -1);
}

#[test]
fn deck_names_classify_into_the_recovered_row_families() {
    assert_eq!(RowKind::classify("Tyrone's trick deck"), RowKind::Deck);
    assert_eq!(RowKind::classify("Slowhands' First Round"), RowKind::Round);
    assert_eq!(RowKind::classify("Battle Riley"), RowKind::Battle);
    assert_eq!(RowKind::classify("Kai's Challenge"), RowKind::Challenge);
    assert_eq!(
        RowKind::classify("Free Realms Trading Card Game Lobby"),
        RowKind::Lobby
    );
    assert_eq!(
        RowKind::classify("Free Realms Trading Card Game Tutorial"),
        RowKind::Tutorial
    );
    assert_eq!(RowKind::classify("Creepy Crawly Cards"), RowKind::Other);
    assert_eq!(RowKind::classify(""), RowKind::Unnamed);
}

#[test]
fn archetype_vocabulary_is_the_four_shipped_decks() {
    assert_eq!(Archetype::from_deck_name("trick"), Some(Archetype::Trick));
    assert_eq!(
        Archetype::from_deck_name("Resource"),
        Some(Archetype::Resource)
    );
    assert_eq!(
        Archetype::from_deck_name("creature"),
        Some(Archetype::Creature)
    );
    assert_eq!(
        Archetype::from_deck_name("Tournament"),
        Some(Archetype::Tournament)
    );
    assert_eq!(Archetype::from_deck_name("mixed"), None);
    assert_eq!(
        Archetype::all().map(Archetype::as_str),
        ["resource", "trick", "creature", "tournament"]
    );
}

#[test]
fn deck_row_parses_opponent_archetype_difficulty_and_reward() {
    let row = reward_deck(387, "Tyrone's trick deck", "Pummel trick card", "1");
    assert_eq!(row.id, 387);
    assert_eq!(row.opponent.as_deref(), Some("Tyrone"));
    assert_eq!(row.kind, RowKind::Deck);
    assert_eq!(row.archetype, Some(Archetype::Trick));
    assert_eq!(row.difficulty, Some(1));
    assert_eq!(row.score_target, Some(WIN_SCORE));
    assert_eq!(row.reward_card.as_deref(), Some("Pummel trick card"));
    assert!(row.reward_eligible());
    assert!(row.gaps.is_empty());
}

#[test]
fn round_rows_keep_their_opponent_and_target() {
    let round = reward_deck(
        415,
        "Slowhands' First Round",
        "Weathered Shield resource card",
        "2",
    );
    assert_eq!(round.kind, RowKind::Round);
    assert_eq!(round.opponent.as_deref(), Some("Slowhands"));
    assert_eq!(round.archetype, None);
    assert_eq!(round.score_target, Some(WIN_SCORE));
    assert!(round.reward_eligible());
}

#[test]
fn a_challenge_without_a_target_keeps_its_gaps() {
    let challenge = deck(
        1036,
        "Kai's Challenge",
        "Ready your best deck and challenge Kai to a single-player match.",
        "1",
    );
    assert_eq!(challenge.kind, RowKind::Challenge);
    assert_eq!(challenge.opponent.as_deref(), Some("Kai"));
    assert!(challenge.score_target.is_none());
    assert!(!challenge.reward_eligible());
    assert_eq!(
        challenge.gaps,
        vec!["score_target".to_string(), "reward_card".to_string()]
    );
}

#[test]
fn undeclared_difficulty_is_not_a_valid_difficulty() {
    let row = deck(9997, "", "", &UNDECLARED_DIFFICULTY.to_string());
    assert_eq!(row.difficulty, None);
    assert_eq!(row.kind, RowKind::Unnamed);
    assert!(row.score_target.is_none());
}

#[test]
fn a_match_only_counts_points_after_the_start_packet() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).expect("bounded target");
    assert_eq!(game.phase(), Phase::WaitingToStart);
    assert_eq!(
        game.apply_points(Challenger, 1, 3),
        Err(MatchError::NotStarted)
    );
    game.start();
    assert_eq!(game.phase(), Phase::Playing);
    assert_eq!(
        game.apply_points(Challenger, 1, 3),
        Ok(AppliedPoints {
            side: Challenger,
            score: 3,
            winner: None,
        })
    );
}

#[test]
fn the_first_player_to_twelve_points_wins_and_nothing_scores_after() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    game.apply_points(Challenger, 1, 5).unwrap();
    game.apply_points(Opponent, 2, 6).unwrap();
    game.apply_points(Challenger, 3, 6).unwrap();
    game.apply_points(Opponent, 4, 5).unwrap();
    assert_eq!(game.scores(), [11, 11]);
    assert_eq!(
        game.phase(),
        Phase::Playing,
        "neither side has reached 12 yet"
    );

    let last = game.apply_points(Challenger, 5, 1).unwrap();
    assert_eq!(last.winner, Some(Challenger));
    assert_eq!(game.phase(), Phase::Finished);
    assert_eq!(game.winner(), Some(Challenger));
    assert_eq!(
        game.apply_points(Opponent, 6, 1),
        Err(MatchError::MatchFinished)
    );
    assert_eq!(
        game.scores(),
        [WIN_SCORE, 11],
        "the losing score is preserved"
    );
}

#[test]
fn a_side_that_reaches_the_target_first_wins_the_rest_is_rejected() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    let update = game.apply_points(Opponent, 1, WIN_SCORE).unwrap();
    assert_eq!(update.winner, Some(Opponent));
    assert_eq!(game.winner(), Some(Opponent));
    assert_eq!(
        game.apply_points(Challenger, 2, 1),
        Err(MatchError::MatchFinished)
    );
}

#[test]
fn point_events_are_positive_and_bounded_by_the_target() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    assert_eq!(
        game.apply_points(Challenger, 1, 0),
        Err(MatchError::InvalidPoints)
    );
    assert_eq!(
        game.apply_points(Challenger, 1, -3),
        Err(MatchError::InvalidPoints)
    );
    assert_eq!(
        game.apply_points(Challenger, 1, WIN_SCORE + 1),
        Err(MatchError::PointsExceedTarget)
    );
    assert_eq!(game.scores(), [0, 0], "rejected events change nothing");
}

#[test]
fn event_ids_must_strictly_increase_so_a_replay_cannot_score_again() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    game.apply_points(Challenger, 7, 4).unwrap();
    let after = game.clone();

    assert_eq!(
        game.apply_points(Challenger, 7, 4),
        Err(MatchError::ReplayedEvent)
    );
    assert_eq!(
        game.apply_points(Opponent, 6, 4),
        Err(MatchError::EventOutOfOrder)
    );
    assert_eq!(game, after, "a rejected event leaves the match untouched");

    game.apply_points(Opponent, 8, 4).unwrap();
    assert_eq!(game.scores(), [4, 4]);
}

#[test]
fn a_match_target_must_be_reachable_in_whole_point_events() {
    assert_eq!(TcgMatch::new(HOST, 0), Err(MatchError::InvalidTarget));
    assert_eq!(
        TcgMatch::new(HOST, WIN_SCORE + 1),
        Err(MatchError::InvalidTarget)
    );
    assert!(TcgMatch::new(HOST, WIN_SCORE).is_ok());
}

#[test]
fn the_winner_receives_the_reward_named_by_that_deck_row_only_once() {
    let challenger = reward_deck(387, "Tyrone's trick deck", "Pummel trick card", "1");
    let opponent = reward_deck(
        388,
        "Tyrone's resource deck",
        "Hot Springs Haven resource card",
        "1",
    );
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    assert_eq!(game.instance_id(), HOST);
    assert_eq!(game.target(), WIN_SCORE);
    game.start();
    game.apply_points(Opponent, 1, WIN_SCORE).unwrap();
    assert_eq!(
        game.reward_card([&challenger, &opponent]),
        Some("Hot Springs Haven resource card")
    );
    assert_eq!(game.score(Opponent), WIN_SCORE);
    assert_eq!(game.applied().len(), 1);

    assert_eq!(
        game.apply_points(Opponent, 2, 1),
        Err(MatchError::MatchFinished)
    );
    assert_eq!(
        game.reward_card([&challenger, &opponent]),
        Some("Hot Springs Haven resource card"),
        "a rejected post-game event cannot re-grant the reward"
    );
    assert_eq!(
        game.applied().len(),
        1,
        "the rejected event is not recorded"
    );

    let rewardless = deck(1036, "Kai's Challenge", "Challenge Kai.", "1");
    let mut solo = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    solo.start();
    solo.apply_points(Challenger, 1, WIN_SCORE).unwrap();
    assert_eq!(solo.reward_card([&rewardless, &opponent]), None);
}

#[test]
fn an_unfinished_match_grants_nothing() {
    let challenger = reward_deck(387, "Tyrone's trick deck", "Pummel trick card", "1");
    let opponent = reward_deck(388, "Tyrone's resource deck", "Hot Springs Haven", "1");
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    game.apply_points(Challenger, 1, WIN_SCORE - 1).unwrap();
    assert_eq!(game.reward_card([&challenger, &opponent]), None);
}

#[test]
fn a_session_round_trips_through_its_snapshot_for_reconnect() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    game.apply_points(Challenger, 1, 4).unwrap();
    game.apply_points(Opponent, 2, 9).unwrap();

    let snapshot = game.snapshot();
    let restored = TcgMatch::from_snapshot(HOST, WIN_SCORE, snapshot).unwrap();
    assert_eq!(restored.snapshot(), game.snapshot());
    assert_eq!(restored.scores(), [4, 9]);
    assert_eq!(restored.phase(), Phase::Playing);
    assert_eq!(
        restored.last_event_id(),
        2,
        "the restored session still rejects replayed events"
    );
    let mut restored = restored;
    assert_eq!(
        restored.apply_points(Opponent, 2, 3),
        Err(MatchError::ReplayedEvent),
        "a replayed post-reconnect event cannot score again"
    );
    assert_eq!(restored.scores(), [4, 9]);
}

#[test]
fn a_finished_snapshot_restores_with_its_winner() {
    let mut game = TcgMatch::new(HOST, WIN_SCORE).unwrap();
    game.start();
    game.apply_points(Challenger, 1, WIN_SCORE).unwrap();
    let restored =
        TcgMatch::from_snapshot(HOST, WIN_SCORE, game.snapshot()).expect("valid snapshot");
    assert_eq!(restored.winner(), Some(Challenger));
    assert_eq!(restored.phase(), Phase::Finished);
}

#[test]
fn an_out_of_range_snapshot_is_rejected() {
    assert_eq!(
        TcgMatch::from_snapshot(
            HOST,
            WIN_SCORE,
            piece_010::MatchSnapshot {
                scores: [WIN_SCORE + 1, 0],
                finished: false,
                winner: None,
                last_event_id: 0,
            },
        ),
        Err(MatchError::InvalidSnapshot)
    );
    assert_eq!(
        TcgMatch::from_snapshot(
            HOST,
            WIN_SCORE,
            piece_010::MatchSnapshot {
                scores: [3, 3],
                finished: true,
                winner: None,
                last_event_id: 2,
            },
        ),
        Err(MatchError::InvalidSnapshot),
        "a finished match without a winner is not a state the rules can reach"
    );
}
