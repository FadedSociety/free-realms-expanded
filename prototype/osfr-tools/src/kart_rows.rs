//! L930: the MiniGameData rows the kart race serves (`kart_rows.json`, beside `kart_gates.json`).
//!
//! The shipped race package (`kart_entry.json`) is one track scene. A row names the track its
//! MiniGameData row races on; only a row whose track is that scene can run, the others stay listed
//! and closed until their track content exists (the Games window greys them out through the
//! portal's own `CanPlayerJoin`). A running row may override the package's lap count and race
//! window: the window is the row's time limit, and a race that runs past it ends unpaid
//! (`KartService::tick`). A finished race credits the row's own MiniGameData id, never a speedway.
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

const MAX_FILE: usize = 64 * 1024;
const MAX_ROWS: usize = 64;

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
    /// Seconds the race may last (the row's par or limit); unset keeps the package's window.
    #[serde(default)]
    pub time_limit_s: Option<u32>,
    pub source: Vec<SourceTag>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KartRowsFile {
    schema_version: u32,
    rows: Vec<KartRow>,
}

/// The loaded table, bound to the scene the race package runs.
#[derive(Clone, Debug)]
pub struct KartRows {
    scene: String,
    rows: Vec<KartRow>,
}

fn invalid(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

impl KartRows {
    /// Reads `kart_rows.json`; absent means the package serves only its default row. `scene` is the
    /// race package's track scene.
    pub fn load(directory: &Path, scene: &str) -> io::Result<Option<Self>> {
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
            return Err(invalid("kart rows exceed 64 KiB"));
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
            {
                return Err(invalid(format!("invalid kart row {}", row.game_id)));
            }
            check_sources(&format!("kart row {}", row.game_id), &row.source).map_err(invalid)?;
        }
        Ok(Some(Self {
            scene: scene.to_owned(),
            rows: file.rows,
        }))
    }

    fn playable(&self) -> impl Iterator<Item = &KartRow> {
        let scene = Some(self.scene.as_str());
        self.rows
            .iter()
            .filter(move |row| row.track.as_deref() == scene)
    }

    /// Ids of the rows that can run, in file order.
    pub fn playable_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.playable().map(|row| row.game_id)
    }

    pub fn is_playable(&self, id: u32) -> bool {
        self.playable_ids().any(|playable| playable == id)
    }

    /// The race package row `id` runs: `base` with the row's laps and race window.
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
