//! L930: the MiniGameData rows the kart race serves (`kart_rows.json`, beside `kart_gates.json`).
//!
//! A row names the track scene its MiniGameData row races on. A row runs when a race package serves
//! that scene: the shipped Wildwood package (`kart_entry.json`) or one of the per-track packages
//! (`kart_tracks/<scene>.json`, P-kart-racing). Rows on a scene with no package stay listed and
//! closed (the Games window greys them out through the portal's own `CanPlayerJoin`). A running row
//! may override the package's lap count; its clock lives on the quest goal (`require.max_elapsed_seconds`),
//! never on the race, so a slow racer still finishes and sees the board (sheet C13). A finished
//! race credits the row's own MiniGameData id, never a speedway.
//!
//! The file is deliberately outside `kart_entry.json`'s content hash, so editing a rule never
//! invalidates a saved race. Every rule is reconstructed policy (R); each row's `source` says which
//! numbers are the client's own.
use crate::{
    content::{SourceTag, check_sources},
    kart_racing::MAX_LAPS,
    kart_service::KartServiceContent,
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    io::{self, Read},
    path::Path,
};

const MAX_FILE: usize = 128 * 1024;
const MAX_ROWS: usize = 128;

/// The client's own text ids for a row (`Resources/MiniGameData.txt`): the offer window's title,
/// picture, description and difficulty dots when the race runs as this row.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RowTexts {
    pub name_id: u32,
    pub description_id: u32,
    pub icon_id: u32,
    pub difficulty: u32,
}

/// One MiniGameData row the kart race knows.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KartRow {
    pub game_id: u32,
    /// The track scene the row races on; `None` when the client names no track for it.
    #[serde(default)]
    pub track: Option<String>,
    /// Laps to run; unset keeps the package's.
    #[serde(default)]
    pub laps: Option<u32>,
    /// Seconds the race window may last; unset keeps the package's. A quest's clock is a goal, not
    /// a window: leave this unset unless a row must really cut a race short.
    #[serde(default)]
    pub time_limit_s: Option<u32>,
    /// The client's offer texts (`RowTexts`); all four or none. Unset keeps the package's (the
    /// Wildwood free race).
    #[serde(default)]
    pub name_id: Option<u32>,
    #[serde(default)]
    pub description_id: Option<u32>,
    #[serde(default)]
    pub icon_id: Option<u32>,
    #[serde(default)]
    pub difficulty: Option<u32>,
    pub source: Vec<SourceTag>,
}

impl KartRow {
    fn texts(&self) -> Result<Option<RowTexts>, ()> {
        match (
            self.name_id,
            self.description_id,
            self.icon_id,
            self.difficulty,
        ) {
            (None, None, None, None) => Ok(None),
            (Some(name_id), Some(description_id), Some(icon_id), Some(difficulty))
                if name_id > 0 && description_id > 0 && icon_id > 0 =>
            {
                Ok(Some(RowTexts {
                    name_id,
                    description_id,
                    icon_id,
                    difficulty,
                }))
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KartRowsFile {
    schema_version: u32,
    rows: Vec<KartRow>,
}

/// The loaded table, bound to the scenes a race package serves.
#[derive(Clone, Debug)]
pub struct KartRows {
    scenes: BTreeSet<String>,
    rows: Vec<KartRow>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

impl KartRows {
    /// Reads `kart_rows.json`; absent means the package serves only its default row. `scenes` are
    /// the track scenes a race package serves.
    pub fn load(directory: &Path, scenes: BTreeSet<String>) -> io::Result<Option<Self>> {
        let path = directory.join("kart_rows.json");
        let metadata = match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
            Ok(m) => m,
        };
        if !metadata.is_file() {
            return Err(invalid("kart rows must be a regular file"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((MAX_FILE + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_FILE {
            return Err(invalid("kart rows exceed 128 KiB"));
        }
        let file: KartRowsFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        let mut seen = BTreeSet::new();
        if file.schema_version != 1 || file.rows.len() > MAX_ROWS {
            return Err(invalid("invalid kart rows file"));
        }
        for row in &file.rows {
            if row.game_id == 0
                || !seen.insert(row.game_id)
                || row.laps == Some(0)
                || row.time_limit_s == Some(0)
                || row.texts().is_err()
            {
                return Err(invalid(format!("invalid kart row {}", row.game_id)));
            }
            check_sources(&format!("kart row {}", row.game_id), &row.source).map_err(invalid)?;
        }
        Ok(Some(Self {
            scenes,
            rows: file.rows,
        }))
    }

    fn playable(&self) -> impl Iterator<Item = &KartRow> {
        self.rows
            .iter()
            .filter(|row| row.track.as_ref().is_some_and(|t| self.scenes.contains(t)))
    }

    /// Ids of the rows that can run, in file order.
    pub fn playable_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.playable().map(|row| row.game_id)
    }

    pub fn is_playable(&self, id: u32) -> bool {
        self.playable_ids().any(|playable| playable == id)
    }

    /// The track scene row `id` races on, when the row can run.
    pub fn track(&self, id: u32) -> Option<&str> {
        self.playable()
            .find(|row| row.game_id == id)?
            .track
            .as_deref()
    }

    /// The client's offer texts of row `id`, when the table carries them.
    pub fn texts(&self, id: u32) -> Option<RowTexts> {
        self.rows
            .iter()
            .find(|row| row.game_id == id)?
            .texts()
            .ok()
            .flatten()
    }

    /// The race package row `id` runs: `base` (the package of the row's track) with the row's laps
    /// and race window.
    pub fn race(&self, id: u32, base: &KartServiceContent) -> io::Result<KartServiceContent> {
        let row = (self.playable())
            .find(|row| row.game_id == id)
            .ok_or_else(|| invalid(format!("kart row {id} cannot run")))?;
        let mut race = base.clone();
        if let Some(laps) = row.laps {
            if laps > MAX_LAPS {
                return Err(invalid(format!("kart row {id} has too many laps")));
            }
            race.laps = laps;
            race.progress_template.lap_limit = laps as i32;
        }
        if let Some(seconds) = row.time_limit_s {
            let limit = u64::from(seconds) * 1000;
            if limit > base.bounds.max_race_ms {
                return Err(invalid(format!("kart row {id} outlasts the race window")));
            }
            race.bounds.max_race_ms = limit;
        }
        Ok(race)
    }
}
