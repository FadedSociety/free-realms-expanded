//! Piece 019 behavioral tests for the activity leaderboard store.
//!
//! The owned module is included by path so this file exercises
//! `src/restoration/piece_019.rs` without touching `lib.rs`. Fixtures are
//! synthetic and private: no save, gateway or native client state is read.
//!
//! Recovered client contract under test (see notes/Research/Pieces/019):
//! - row payload uses `##` between rows, `::` between properties and `^`
//!   between a property and its value, with properties
//!   `Rank`, `Name`, `Guid`, `Value`, `Id`, `Type`;
//! - `Type` 1 is a points score and `Type` 2 is elapsed seconds;
//! - the high-score list renders ten rows per page.
#[path = "../src/restoration/piece_019.rs"]
mod piece_019;

use piece_019::{
    CommitOutcome, HIGH_SCORES_PAGE_ROWS, LeaderboardStore, MAX_DISPLAYED_VALUE, PROP_DELIMITER,
    ROW_DELIMITER, SCORE_TYPE_ELAPSED_SECONDS, SCORE_TYPE_POINTS, ScoreKind, StoreError,
    VALUE_DELIMITER,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const BOARD: &str = "91";
const TIME_BOARD: &str = "93";

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-piece019-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn save_file(&self) -> PathBuf {
        self.path.join("leaderboards.json")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Server-side session that ends successfully and commits a value.
fn commit_on(
    store: &mut LeaderboardStore,
    board: &str,
    guid: &str,
    name: &str,
    kind: ScoreKind,
    value: i64,
) -> CommitOutcome {
    let session = store.begin_session(board, guid, name, kind).unwrap();
    store.finish_session(session).unwrap();
    store.commit(session, value).unwrap()
}

fn commit(
    store: &mut LeaderboardStore,
    guid: &str,
    name: &str,
    kind: ScoreKind,
    value: i64,
) -> CommitOutcome {
    commit_on(store, BOARD, guid, name, kind, value)
}

/// Replica of the client `ItemDisplay.deserialize` split rules.
fn parse_payload(payload: &str) -> Vec<Vec<(String, String)>> {
    if payload.is_empty() {
        return Vec::new();
    }
    payload
        .split(ROW_DELIMITER)
        .map(|row| {
            row.split(PROP_DELIMITER)
                .map(|prop| {
                    let (key, value) = prop.split_once(VALUE_DELIMITER).unwrap();
                    (key.to_string(), value.to_string())
                })
                .collect()
        })
        .collect()
}

#[test]
fn payload_matches_the_recovered_client_row_schema() {
    let mut store = LeaderboardStore::default();
    commit(&mut store, "P0001", "Ayla", ScoreKind::Points, 1200);
    commit(&mut store, "P0002", "Bram", ScoreKind::Points, 900);

    let (payload, total) = store.page_payload(BOARD, 0, HIGH_SCORES_PAGE_ROWS).unwrap();

    assert_eq!(total, 2);
    assert_eq!(
        payload,
        "Rank^1::Name^Ayla::Guid^P0001::Value^1200::Id^91::Type^1##\
         Rank^2::Name^Bram::Guid^P0002::Value^900::Id^91::Type^1"
    );

    let rows = parse_payload(&payload);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], ("Rank".to_string(), "1".to_string()));
    assert_eq!(
        rows[0][5],
        ("Type".to_string(), SCORE_TYPE_POINTS.to_string())
    );
    assert_eq!(rows[1][1], ("Name".to_string(), "Bram".to_string()));
}

#[test]
fn points_rank_high_to_low_and_ties_share_a_rank() {
    let mut store = LeaderboardStore::default();
    commit(&mut store, "A", "Ayla", ScoreKind::Points, 100);
    commit(&mut store, "B", "Bram", ScoreKind::Points, 100);
    commit(&mut store, "C", "Cora", ScoreKind::Points, 90);

    let rows = store.rows(BOARD);
    let ranks: Vec<usize> = rows.iter().map(|row| row.rank).collect();
    assert_eq!(ranks, vec![1, 1, 3]);
    assert_eq!(rows[0].value, 100);
    assert_eq!(rows[2].character_guid, "C");
}

#[test]
fn elapsed_seconds_rank_low_to_high_with_the_time_type_code() {
    let mut store = LeaderboardStore::default();
    commit(&mut store, "A", "Ayla", ScoreKind::ElapsedSeconds, 305);
    commit(&mut store, "B", "Bram", ScoreKind::ElapsedSeconds, 121);

    let rows = store.rows(BOARD);
    assert_eq!(rows[0].character_guid, "B");
    assert_eq!(rows[1].character_guid, "A");

    let (payload, _) = store.page_payload(BOARD, 0, HIGH_SCORES_PAGE_ROWS).unwrap();
    assert!(payload.contains("Value^121"));
    assert!(payload.contains(&format!("Type^{SCORE_TYPE_ELAPSED_SECONDS}")));
}

#[test]
fn a_score_is_only_written_from_a_finished_server_session() {
    let mut store = LeaderboardStore::default();

    assert_eq!(
        store.begin_session("", "A", "Ayla", ScoreKind::Points),
        Err(StoreError::InvalidBoard)
    );
    assert_eq!(
        store.begin_session("0", "A", "Ayla", ScoreKind::Points),
        Err(StoreError::InvalidBoard)
    );
    assert_eq!(
        store.begin_session(BOARD, "A", "Ayl^a", ScoreKind::Points),
        Err(StoreError::InvalidField)
    );

    // A raw value without a session cannot enter the board.
    assert_eq!(store.commit(7, 999), Err(StoreError::UnknownSession));

    let session = store
        .begin_session(BOARD, "A", "Ayla", ScoreKind::Points)
        .unwrap();
    assert_eq!(
        store.commit(session, 10),
        Err(StoreError::SessionNotFinished)
    );

    store.finish_session(session).unwrap();
    assert_eq!(store.commit(session, -1), Err(StoreError::InvalidValue));
    assert_eq!(
        store.commit(session, MAX_DISPLAYED_VALUE + 1),
        Err(StoreError::InvalidValue)
    );
    assert_eq!(store.commit(session, 10), Ok(CommitOutcome::Recorded));
    assert_eq!(
        store.commit(session, 50),
        Err(StoreError::SessionAlreadyCommitted)
    );
    assert_eq!(store.finish_session(session), Ok(()));
}

#[test]
fn one_entry_per_character_keeps_the_better_value() {
    let mut store = LeaderboardStore::default();
    commit(&mut store, "A", "Ayla", ScoreKind::Points, 100);
    assert_eq!(
        commit(&mut store, "A", "Ayla", ScoreKind::Points, 150),
        CommitOutcome::Recorded
    );
    assert_eq!(
        commit(&mut store, "A", "Ayla", ScoreKind::Points, 120),
        CommitOutcome::KeptBest
    );
    let rows = store.rows(BOARD);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, 150);

    // Elapsed seconds are better when they are smaller.
    commit_on(
        &mut store,
        TIME_BOARD,
        "T",
        "Tia",
        ScoreKind::ElapsedSeconds,
        300,
    );
    assert_eq!(
        commit_on(
            &mut store,
            TIME_BOARD,
            "T",
            "Tia",
            ScoreKind::ElapsedSeconds,
            200
        ),
        CommitOutcome::Recorded
    );
    assert_eq!(
        commit_on(
            &mut store,
            TIME_BOARD,
            "T",
            "Tia",
            ScoreKind::ElapsedSeconds,
            250
        ),
        CommitOutcome::KeptBest
    );
}

#[test]
fn pages_are_sliced_by_row_offset_and_report_the_total_count() {
    let mut store = LeaderboardStore::default();
    for (guid, name, value) in [("A", "Ayla", 300), ("B", "Bram", 200), ("C", "Cora", 100)] {
        commit(&mut store, guid, name, ScoreKind::Points, value);
    }

    let (first, total) = store.page_payload(BOARD, 0, 2).unwrap();
    assert_eq!(total, 3);
    assert_eq!(parse_payload(&first).len(), 2);
    assert!(first.starts_with("Rank^1::Name^Ayla"));

    let (second, total) = store.page_payload(BOARD, 2, HIGH_SCORES_PAGE_ROWS).unwrap();
    assert_eq!(total, 3);
    assert!(second.starts_with("Rank^3::Name^Cora"));
    assert_eq!(parse_payload(&second).len(), 1);

    assert_eq!(
        store.page_payload(BOARD, 0, HIGH_SCORES_PAGE_ROWS + 1),
        Err(StoreError::InvalidPage)
    );
    assert_eq!(store.page_payload("93", 0, 10).unwrap(), (String::new(), 0));
}

#[test]
fn persistence_round_trips_and_preserves_a_corrupt_file() {
    let scratch = Scratch::new();
    let path = scratch.save_file();

    let mut store = LeaderboardStore::default();
    commit(&mut store, "A", "Ayla", ScoreKind::Points, 42);
    commit_on(
        &mut store,
        TIME_BOARD,
        "B",
        "Bram",
        ScoreKind::ElapsedSeconds,
        61,
    );
    store.save(&path).unwrap();

    let loaded = LeaderboardStore::load(&path).unwrap();
    assert_eq!(loaded.rows(BOARD).len(), 1);
    assert_eq!(loaded.rows(BOARD)[0].character_guid, "A");
    assert_eq!(loaded.rows(TIME_BOARD), store.rows(TIME_BOARD));
    assert_eq!(
        loaded
            .page_payload(BOARD, 0, HIGH_SCORES_PAGE_ROWS)
            .unwrap(),
        store.page_payload(BOARD, 0, HIGH_SCORES_PAGE_ROWS).unwrap()
    );

    fs::write(&path, b"{not json").unwrap();
    assert!(matches!(
        LeaderboardStore::load(&path),
        Err(StoreError::Corrupt(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"{not json");
}

#[test]
fn a_wrong_persistence_version_is_rejected() {
    let scratch = Scratch::new();
    let path = scratch.save_file();
    fs::write(&path, br#"{"version":99,"boards":{}}"#).unwrap();
    assert!(matches!(
        LeaderboardStore::load(&path),
        Err(StoreError::Corrupt(_))
    ));
}
