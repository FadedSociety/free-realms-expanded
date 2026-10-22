//! T459: the boombox catalogue (`boomboxes.json`), one row per client boombox item: the box model,
//! its dances and songs, and the tunables a placed box will use. Catalogue only in this slice: the
//! item bar lets a listed box sit on the bar, shown disabled until boombox use spawns the box.
//!
//! # Recovered original facts
//!
//! * The client offers an item for the item bar (`BaseClient.Inventory.Consumables`, row
//!   predicate `00c31de0`) when its definition's type (+0x6c) is 1, its activatable ability
//!   (+0x8c) is positive, its gender usage fits (`00bae6a0`) and a unit is left. All 15 boombox
//!   definitions the server loads pass, so the client sends 42/6 for them.
//! * Every model, animation and song effect id is checked against the client's own tables by
//!   `tests/boombox_catalogue.rs`; each row's `source` cites where its values come from.
//!
//! # Reconstructed (R)
//!
//! * Dances, clip lengths, songs, priority and transforms come from Sanctuary's measured rows;
//!   duration, range, spawn offset and recast are Sanctuary's defaults. All are tunable here.

use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::Path,
};

const FILE: &str = "boomboxes.json";
const MAX_FILE: u64 = 256 * 1024;
const MAX_ROWS: usize = 64;
const MAX_MS: u32 = 600_000;
const MIN_RECAST_MS: u32 = 3_000;

/// How the dancers around a box move: all in step, each on its own clips, or transformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Synchronized,
    Independent,
    /// Dancers take `transform_model_id`; only this kind has one.
    Transform,
}

/// Where a row's values come from. Read by people and content gates, never by the server.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: String,
    pub evidence: String,
}

/// One boombox. Ids of 0 mean "none" for `standing_animation_id` and `transform_model_id`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boombox {
    pub item_id: i32,
    pub model_id: i32,
    pub kind: Kind,
    pub dance_sequence: Vec<i32>,
    /// Player model id -> one clip length per dance step. Empty for `Independent` boxes, whose
    /// per-dancer random clips are a later slice.
    pub clip_ms: BTreeMap<i32, Vec<u32>>,
    pub blend_ms: u32,
    pub song_effect_ids: Vec<i32>,
    pub standing_animation_id: i32,
    pub priority: i32,
    pub transform_model_id: i32,
    /// 0 unless `kind` is `Transform`.
    pub transform_reapply_ms: u32,
    pub duration_ms: u32,
    pub range_m: f32,
    pub spawn_offset_m: f32,
    pub recast_ms: u32,
    pub texture_alias: String,
    pub tint_alias: String,
    pub source: Vec<Source>,
}

impl Boombox {
    /// Sanity bounds, not tuned values.
    fn valid(&self) -> bool {
        let ms = |v: u32| (1..=MAX_MS).contains(&v);
        let transform = self.kind == Kind::Transform;
        self.item_id > 0
            && self.model_id > 0
            && !self.dance_sequence.is_empty()
            && self.dance_sequence.iter().all(|id| *id > 0)
            && self.song_effect_ids.iter().all(|id| *id > 0)
            && self.standing_animation_id >= 0
            && (self.kind == Kind::Independent) == self.clip_ms.is_empty()
            && self.clip_ms.iter().all(|(model, clips)| {
                *model > 0
                    && clips.len() == self.dance_sequence.len()
                    && clips.iter().all(|v| ms(*v))
            })
            && ms(self.blend_ms)
            && self.transform_model_id >= 0
            && transform == (self.transform_model_id > 0)
            && if transform {
                ms(self.transform_reapply_ms)
            } else {
                self.transform_reapply_ms == 0
            }
            && ms(self.duration_ms)
            && (1.0..=50.0).contains(&self.range_m)
            && (0.0..=10.0).contains(&self.spawn_offset_m)
            && (MIN_RECAST_MS..=MAX_MS).contains(&self.recast_ms)
            && !self.source.is_empty()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogueFile {
    schema_version: u32,
    boomboxes: Vec<Boombox>,
}

/// Every boombox row, read-only and shared. Nothing about it is saved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Catalogue(Vec<Boombox>);

impl Catalogue {
    /// `boomboxes.json` from the content directory; a missing file is an empty catalogue.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let file = match fs::File::open(directory.join(FILE)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(MAX_FILE + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE {
            return Err(invalid("boomboxes.json exceeds 256 KiB"));
        }
        let parsed: CatalogueFile = serde_json::from_slice(&bytes).map_err(invalid)?;
        if parsed.schema_version != 1 || parsed.boomboxes.len() > MAX_ROWS {
            return Err(invalid("unsupported boomboxes.json"));
        }
        let rows = parsed.boomboxes;
        for (index, row) in rows.iter().enumerate() {
            if !row.valid() || rows[..index].iter().any(|r| r.item_id == row.item_id) {
                return Err(invalid(format!("invalid boombox row {}", row.item_id)));
            }
        }
        Ok(Self(rows))
    }

    pub fn get(&self, item_id: i32) -> Option<&Boombox> {
        self.0.iter().find(|row| row.item_id == item_id)
    }
}

fn invalid(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
