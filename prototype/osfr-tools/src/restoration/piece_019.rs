//! Piece 019: activity leaderboards.
//!
//! This module is self-contained and is exercised by `tests/piece_019.rs`
//! through a `#[path]` include, so it does not touch shared registration.
//!
//! Evidence classes (see `notes/Research/Pieces/019 Activity leaderboards.md`):
//! - Recovered from pinned artifacts: the client row schema (`Rank`, `Name`,
//!   `Guid`, `Value`, `Id`, `Type`; `##` rows, `::` properties, `^` values),
//!   `Type` 1 points versus `Type` 2 elapsed seconds, ten rows per high-score
//!   page, the per-game `LEADERBOARD_ID` bindings, the web endpoint key
//!   `GameLeaderBoardUrl`, and mini-game end-score packet 47.
//! - Inferred: points rank high to low ("Top Score"); elapsed seconds rank low
//!   to high.
//! - Reconstructed: one best entry per character, competition tie ranks, the
//!   ten-digit value ceiling from the client's roll-up fields, and the session
//!   gate that keeps a value off the board unless the server finished a session
//!   for it.
//! - Not recovered: the original web query parameters and the original ranking
//!   service. No leaderboard reward rule is claimed.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Client `HighScoreItem.TYPE` value for a points score.
pub const SCORE_TYPE_POINTS: i32 = 1;
/// Client `HighScoreItem.TYPE` value for elapsed seconds.
pub const SCORE_TYPE_ELAPSED_SECONDS: i32 = 2;
/// `ItemDisplay.DELIM_ITEM`.
pub const ROW_DELIMITER: &str = "##";
/// `ItemDisplay.DELIM_PROP`.
pub const PROP_DELIMITER: &str = "::";
/// `ItemDisplay.DELIM_VALUE`.
pub const VALUE_DELIMITER: &str = "^";
/// `ListPanel` high-score mode is ten rows in one column.
pub const HIGH_SCORES_PAGE_ROWS: usize = 10;
/// `HighScoreItem.setScore` writes `txt0..txt9`, so ten digits is the display ceiling.
pub const MAX_DISPLAYED_VALUE: i64 = 9_999_999_999;

const SNAPSHOT_VERSION: u32 = 1;
const MAX_FIELD_BYTES: usize = 64;
const MAX_FILE_BYTES: usize = 1 << 20;
const FIELD_DELIMITERS: [&str; 3] = [ROW_DELIMITER, PROP_DELIMITER, VALUE_DELIMITER];

/// The `Type` property of a board row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreKind {
    Points,
    ElapsedSeconds,
}

impl ScoreKind {
    pub fn type_code(self) -> i32 {
        match self {
            ScoreKind::Points => SCORE_TYPE_POINTS,
            ScoreKind::ElapsedSeconds => SCORE_TYPE_ELAPSED_SECONDS,
        }
    }

    fn is_better(self, candidate: i64, current: i64) -> bool {
        match self {
            ScoreKind::Points => candidate > current,
            ScoreKind::ElapsedSeconds => candidate < current,
        }
    }
}

/// One ranked row exactly as the client renders it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardRow {
    pub rank: usize,
    pub character_guid: String,
    pub character_name: String,
    pub value: i64,
    pub kind: ScoreKind,
}

/// Result of committing a finished session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitOutcome {
    /// The value replaced the stored entry.
    Recorded,
    /// The stored entry was already better; the session is still consumed.
    KeptBest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreError {
    InvalidBoard,
    InvalidField,
    InvalidValue,
    InvalidPage,
    UnknownSession,
    SessionNotFinished,
    SessionAlreadyCommitted,
    Corrupt(String),
    Io(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            StoreError::InvalidBoard => "leaderboard id must be a positive decimal id",
            StoreError::InvalidField => "character field is empty or contains a wire delimiter",
            StoreError::InvalidValue => "score is negative or above the display ceiling",
            StoreError::InvalidPage => "page count must be between one and ten rows",
            StoreError::UnknownSession => "no server session exists for this commit",
            StoreError::SessionNotFinished => "the activity session has not finished",
            StoreError::SessionAlreadyCommitted => "the activity session already committed",
            StoreError::Corrupt(_) => "leaderboard snapshot is not readable",
            StoreError::Io(_) => "leaderboard snapshot could not be read or written",
        };
        write!(formatter, "{message}")
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(error: io::Error) -> Self {
        StoreError::Io(error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ScoreRecord {
    guid: String,
    name: String,
    value: i64,
}

#[derive(Clone, Debug)]
struct Session {
    board: String,
    guid: String,
    name: String,
    kind: ScoreKind,
    finished: bool,
    committed: bool,
}

/// Server-side high-score boards for the activity ids that ship a
/// `LEADERBOARD_ID`. Entries only enter through a finished server session.
#[derive(Clone, Debug, Default)]
pub struct LeaderboardStore {
    boards: BTreeMap<String, Board>,
    sessions: BTreeMap<u64, Session>,
    next_session: u64,
}

#[derive(Clone, Debug)]
struct Board {
    kind: ScoreKind,
    entries: BTreeMap<String, ScoreRecord>,
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    version: u32,
    boards: BTreeMap<String, BoardSnapshot>,
}

#[derive(Serialize, Deserialize)]
struct BoardSnapshot {
    kind: ScoreKind,
    entries: Vec<ScoreRecord>,
}

impl LeaderboardStore {
    /// Opens a server-side activity session that may later commit one score.
    pub fn begin_session(
        &mut self,
        board: &str,
        guid: &str,
        name: &str,
        kind: ScoreKind,
    ) -> Result<u64, StoreError> {
        validate_board(board)?;
        validate_field(guid)?;
        validate_field(name)?;
        self.next_session += 1;
        let id = self.next_session;
        self.sessions.insert(
            id,
            Session {
                board: board.to_string(),
                guid: guid.to_string(),
                name: name.to_string(),
                kind,
                finished: false,
                committed: false,
            },
        );
        Ok(id)
    }

    /// Marks a session as finished. Repeated calls are harmless.
    pub fn finish_session(&mut self, session: u64) -> Result<(), StoreError> {
        let session = self
            .sessions
            .get_mut(&session)
            .ok_or(StoreError::UnknownSession)?;
        session.finished = true;
        Ok(())
    }

    /// Commits the value of a finished session. The session is consumed even
    /// when the stored entry was already better, so a replay cannot add twice.
    pub fn commit(&mut self, session: u64, value: i64) -> Result<CommitOutcome, StoreError> {
        if !(0..=MAX_DISPLAYED_VALUE).contains(&value) {
            return Err(StoreError::InvalidValue);
        }
        let active = self
            .sessions
            .get(&session)
            .ok_or(StoreError::UnknownSession)?
            .clone();
        if active.committed {
            return Err(StoreError::SessionAlreadyCommitted);
        }
        if !active.finished {
            return Err(StoreError::SessionNotFinished);
        }
        let outcome = match self.boards.get(&active.board) {
            Some(board) if board.kind != active.kind => return Err(StoreError::InvalidBoard),
            Some(board) => match board.entries.get(&active.guid) {
                Some(record) if !active.kind.is_better(value, record.value) => {
                    CommitOutcome::KeptBest
                }
                Some(_) => CommitOutcome::Recorded,
                None => CommitOutcome::Recorded,
            },
            None => CommitOutcome::Recorded,
        };
        if outcome == CommitOutcome::Recorded {
            self.boards
                .entry(active.board.clone())
                .or_insert_with(|| Board {
                    kind: active.kind,
                    entries: BTreeMap::new(),
                })
                .entries
                .insert(
                    active.guid.clone(),
                    ScoreRecord {
                        guid: active.guid.clone(),
                        name: active.name.clone(),
                        value,
                    },
                );
        }
        self.sessions
            .get_mut(&session)
            .expect("session was just read")
            .committed = true;
        Ok(outcome)
    }

    /// Ranked rows for one board. Ties share a rank (1, 1, 3).
    pub fn rows(&self, board: &str) -> Vec<LeaderboardRow> {
        let Some(board) = self.boards.get(board) else {
            return Vec::new();
        };
        let mut records: Vec<&ScoreRecord> = board.entries.values().collect();
        records.sort_by(|left, right| match board.kind {
            ScoreKind::Points => right
                .value
                .cmp(&left.value)
                .then(left.guid.cmp(&right.guid)),
            ScoreKind::ElapsedSeconds => left
                .value
                .cmp(&right.value)
                .then(left.guid.cmp(&right.guid)),
        });
        let mut rows: Vec<LeaderboardRow> = Vec::with_capacity(records.len());
        for record in records {
            let rank = match rows.last() {
                Some(previous) if previous.value == record.value => previous.rank,
                _ => rows.len() + 1,
            };
            rows.push(LeaderboardRow {
                rank,
                character_guid: record.guid.clone(),
                character_name: record.name.clone(),
                value: record.value,
                kind: board.kind,
            });
        }
        rows
    }

    /// Builds the `ListPanel.setData` payload for one page plus the total row
    /// count the client needs for its scrollbar.
    pub fn page_payload(
        &self,
        board: &str,
        offset: usize,
        count: usize,
    ) -> Result<(String, usize), StoreError> {
        if count == 0 || count > HIGH_SCORES_PAGE_ROWS {
            return Err(StoreError::InvalidPage);
        }
        let rows = self.rows(board);
        let total = rows.len();
        let page: Vec<LeaderboardRow> = rows.into_iter().skip(offset).take(count).collect();
        Ok((encode_rows(board, &page), total))
    }

    /// Atomically replaces the snapshot file. A failed write preserves the
    /// previous file.
    pub fn save(&self, path: &Path) -> Result<(), StoreError> {
        let snapshot = Snapshot {
            version: SNAPSHOT_VERSION,
            boards: self
                .boards
                .iter()
                .map(|(id, board)| {
                    (
                        id.clone(),
                        BoardSnapshot {
                            kind: board.kind,
                            entries: board.entries.values().cloned().collect(),
                        },
                    )
                })
                .collect(),
        };
        let bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(StoreError::Corrupt("snapshot exceeds 1 MiB".to_string()));
        }
        replace_bytes(path, &bytes)
    }

    /// Loads a snapshot. Unreadable or unsupported files are rejected without
    /// touching the file.
    pub fn load(path: &Path) -> Result<Self, StoreError> {
        let bytes = fs::read(path)?;
        if bytes.len() > MAX_FILE_BYTES {
            return Err(StoreError::Corrupt("snapshot exceeds 1 MiB".to_string()));
        }
        let snapshot: Snapshot = serde_json::from_slice(&bytes)
            .map_err(|error| StoreError::Corrupt(error.to_string()))?;
        if snapshot.version != SNAPSHOT_VERSION {
            return Err(StoreError::Corrupt(format!(
                "unsupported snapshot version {}",
                snapshot.version
            )));
        }
        let mut store = LeaderboardStore::default();
        for (id, board) in snapshot.boards {
            validate_board(&id).map_err(|_| StoreError::Corrupt("invalid board id".to_string()))?;
            let mut entries = BTreeMap::new();
            for record in board.entries {
                validate_field(&record.guid)
                    .and_then(|()| validate_field(&record.name))
                    .map_err(|_| StoreError::Corrupt("invalid entry field".to_string()))?;
                if !(0..=MAX_DISPLAYED_VALUE).contains(&record.value) {
                    return Err(StoreError::Corrupt("invalid entry value".to_string()));
                }
                entries.insert(record.guid.clone(), record);
            }
            store.boards.insert(
                id,
                Board {
                    kind: board.kind,
                    entries,
                },
            );
        }
        Ok(store)
    }
}

fn encode_rows(board: &str, rows: &[LeaderboardRow]) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "Rank{vd}{rank}{pd}Name{vd}{name}{pd}Guid{vd}{guid}{pd}Value{vd}{value}\
                 {pd}Id{vd}{board}{pd}Type{vd}{kind}",
                vd = VALUE_DELIMITER,
                rank = row.rank,
                pd = PROP_DELIMITER,
                name = row.character_name,
                guid = row.character_guid,
                value = row.value,
                board = board,
                kind = row.kind.type_code(),
            )
        })
        .collect::<Vec<String>>()
        .join(ROW_DELIMITER)
}

fn validate_board(board: &str) -> Result<(), StoreError> {
    if board.is_empty() || board.len() > 18 || !board.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(StoreError::InvalidBoard);
    }
    if board.bytes().all(|byte| byte == b'0') {
        return Err(StoreError::InvalidBoard);
    }
    Ok(())
}

fn validate_field(field: &str) -> Result<(), StoreError> {
    if field.is_empty() || field.len() > MAX_FIELD_BYTES {
        return Err(StoreError::InvalidField);
    }
    if field.chars().any(|character| character.is_control()) {
        return Err(StoreError::InvalidField);
    }
    if FIELD_DELIMITERS
        .iter()
        .any(|delimiter| field.contains(delimiter))
    {
        return Err(StoreError::InvalidField);
    }
    Ok(())
}

fn replace_bytes(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let temporary = temporary_path(path)?;
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    let mut retries = 0;
    loop {
        match fs::rename(&temporary, path) {
            Ok(()) => return Ok(()),
            // A concurrent Windows reader can deny replacement briefly.
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(5 | 32))
                    && retries < 20 =>
            {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(StoreError::Io(error.to_string()));
            }
        }
    }
}

fn temporary_path(path: &Path) -> Result<PathBuf, StoreError> {
    let file_name = path
        .file_name()
        .ok_or_else(|| StoreError::Io("snapshot path has no file name".to_string()))?;
    let mut name = file_name.to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    Ok(path.with_file_name(name))
}
