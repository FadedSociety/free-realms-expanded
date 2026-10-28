//! T507/T552: per-model ambient animations for passive content NPCs.
//!
//! The rows are Sanctuary's `Resources/NpcAmbient.json`
//! (`Sanctuary-main-20261019@10f129e`, AGPL-3.0 values only, no code). Upstream
//! records measured-looking intervals but names no source, so every schedule
//! value is (R). The schedules are stateless: an actor's firing times depend only
//! on its GUID and the row, so every viewer that knows the actor plays the same
//! animation in the same tick window.
//!
//! T552 adds the three row kinds slice 1 skipped: conversation groups (talkers
//! linked through any chain of neighbours within 6 m XZ form one group), idle
//! loops (35/26 plus a base 35/8), and look-at (35/17, target resolved by the
//! gateway from the player grid). Groups, idle actors and look-at actors are
//! fixed at load (`schedule.rs`); the wire bodies are in `wire.rs`.
//!
//! Animation slot ids are client data and production must not read the client
//! resources, so the loader rejects only structural errors: the content gate
//! (`tests/content_npc_ambient.rs`) checks every id against the client
//! `AnimationTypes.xml`. A hand-edited play-package table with an unknown slot
//! is server-operator content, not client input; the client ignores the id.

use crate::hash::mix64;
use schedule::{ConversationGroup, IdleSchedule, LookAtSchedule};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

mod schedule;
#[cfg(test)]
mod tests;
mod wire;

pub use wire::{AmbientUpdate, idle_animation, look_at};

const CONTENT_FILE: &str = "npc_ambient.json";
const MAX_CONTENT_BYTES: u64 = 1024 * 1024;
const MAX_ROWS: usize = 512;
const MAX_IDS_PER_ROW: usize = 16;
const MIN_INTERVAL_MS: u64 = 500;
const MAX_INTERVAL_MS: u64 = 600_000;
const MAX_CONVERSATION_GAP_MS: u64 = 60_000;
const MIN_LOOK_AT_RADIUS: f32 = 1.0;
const MAX_LOOK_AT_RADIUS: f32 = 30.0;
const DEFAULT_CONVERSATION_RADIUS_M: f32 = 6.0;
const MAX_PACKETS_PER_VIEWER_PER_TICK: std::ops::RangeInclusive<usize> = 1..=16;
/// Distinct domain separators for the independent hashes.
const PHASE_SALT: u64 = 0x9e37_79b9_7f4a_7c15;
const FIRING_SALT: u64 = 0xd1b5_4a32_d192_ed03;
const IDLE_SALT: u64 = 0x94d0_49bb_1331_11eb;
const LOOK_AT_SALT: u64 = 0x2545_f491_4f6c_dd1d;
const CONVERSATION_GAP_SALT: u64 = 0x8ebc_6af0_9c88_c6e3;
const CONVERSATION_PHASE_SALT: u64 = 0xd6e8_feb8_6659_fd93;
const CONVERSATION_ID_SALT: u64 = 0xbf58_476d_1ce4_e5b9;

#[derive(Clone, Debug)]
pub struct NpcAmbient {
    rows: BTreeMap<u32, AmbientRow>,
    max_packets_per_viewer_per_tick: usize,
    conversation_radius_sq: f32,
    conversation_groups: Vec<ConversationGroup>,
    /// Members of a conversation group: their turns replace the random emote.
    talking: BTreeSet<u64>,
    idles: BTreeMap<u64, IdleSchedule>,
    look_at: BTreeMap<u64, LookAtSchedule>,
}

#[derive(Clone, Debug)]
struct AmbientRow {
    animation_ids: Vec<i32>,
    min_interval_ms: u64,
    max_interval_ms: u64,
    conversation_animation_ids: Vec<i32>,
    conversation_min_gap_ms: u64,
    conversation_max_gap_ms: u64,
    idle_animation_ids: Vec<i32>,
    idle_interval_ms: u64,
    look_at_radius: f32,
    look_at_interval_ms: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    source: String,
    max_packets_per_viewer_per_tick: usize,
    #[serde(default = "default_conversation_radius_m")]
    conversation_radius_m: f32,
    rows: Vec<RowDocument>,
}

fn default_conversation_radius_m() -> f32 {
    DEFAULT_CONVERSATION_RADIUS_M
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowDocument {
    model_ids: Vec<u32>,
    model_file_name: String,
    #[serde(default)]
    animation_ids: Vec<i32>,
    #[serde(default)]
    min_interval_ms: u64,
    #[serde(default)]
    max_interval_ms: u64,
    #[serde(default)]
    conversation_animation_ids: Vec<i32>,
    #[serde(default)]
    conversation_min_gap_ms: u64,
    #[serde(default)]
    conversation_max_gap_ms: u64,
    #[serde(default)]
    idle_animation_ids: Vec<i32>,
    #[serde(default)]
    idle_interval_ms: u64,
    #[serde(default)]
    look_at_radius: f32,
    #[serde(default)]
    look_at_interval_ms: u64,
    source: String,
}

impl NpcAmbient {
    /// The table beside `directory`, or `None` when the content set has none.
    /// `actors` is every shown content actor's `(runtime GUID, client model,
    /// position)`; grouping is fixed here and stays stable for the load.
    pub fn load(directory: &Path, actors: &[(u64, u32, [f32; 3])]) -> io::Result<Option<Self>> {
        let path = directory.join(CONTENT_FILE);
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if !metadata.is_file() || metadata.len() > MAX_CONTENT_BYTES {
            return Err(invalid(
                "npc ambient content must be a regular file under 1 MiB",
            ));
        }
        fs::read(path)
            .and_then(|bytes| Self::from_json(&bytes))
            .map(|mut ambient| {
                ambient.place_actors(actors);
                Some(ambient)
            })
    }

    fn from_json(bytes: &[u8]) -> io::Result<Self> {
        let document: Document = serde_json::from_slice(bytes).map_err(invalid)?;
        if document.schema_version != 1 {
            return Err(invalid("unsupported npc ambient content schema"));
        }
        if document.source.trim().is_empty()
            || !MAX_PACKETS_PER_VIEWER_PER_TICK.contains(&document.max_packets_per_viewer_per_tick)
        {
            return Err(invalid(
                "npc ambient needs a source and a 1..=16 packet cap",
            ));
        }
        if !(MIN_LOOK_AT_RADIUS..=MAX_LOOK_AT_RADIUS).contains(&document.conversation_radius_m) {
            return Err(invalid(
                "npc ambient conversation radius is outside 1..=30 m",
            ));
        }
        if document.rows.is_empty() || document.rows.len() > MAX_ROWS {
            return Err(invalid("npc ambient has no rows or too many rows"));
        }
        let mut rows = BTreeMap::new();
        for row in document.rows {
            if row.model_ids.is_empty()
                || row.model_file_name.trim().is_empty()
                || row.source.trim().is_empty()
            {
                return Err(invalid(
                    "npc ambient row needs model ids, a model file name and a source",
                ));
            }
            check_ids(&row.animation_ids, "animation")?;
            check_ids(&row.conversation_animation_ids, "conversation")?;
            check_ids(&row.idle_animation_ids, "idle")?;
            if row.animation_ids.is_empty() {
                if row.min_interval_ms != 0 || row.max_interval_ms != 0 {
                    return Err(invalid(
                        "npc ambient row without animation ids must have zero intervals",
                    ));
                }
            } else if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&row.min_interval_ms)
                || row.max_interval_ms < row.min_interval_ms
                || row.max_interval_ms > MAX_INTERVAL_MS
            {
                return Err(invalid(
                    "npc ambient row intervals are outside 500..=600000 ms",
                ));
            }
            if row.conversation_animation_ids.is_empty() {
                if row.conversation_min_gap_ms != 0 || row.conversation_max_gap_ms != 0 {
                    return Err(invalid(
                        "npc ambient row without conversation ids must have zero gaps",
                    ));
                }
            } else if row.conversation_max_gap_ms < row.conversation_min_gap_ms
                || row.conversation_max_gap_ms > MAX_CONVERSATION_GAP_MS
            {
                return Err(invalid(
                    "npc ambient conversation gaps are outside 0..=60000 ms",
                ));
            }
            if row.idle_animation_ids.is_empty() {
                if row.idle_interval_ms != 0 {
                    return Err(invalid(
                        "npc ambient row without idle ids must have a zero interval",
                    ));
                }
            } else if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&row.idle_interval_ms) {
                return Err(invalid(
                    "npc ambient idle interval is outside 500..=600000 ms",
                ));
            }
            if row.look_at_radius == 0.0 {
                if row.look_at_interval_ms != 0 {
                    return Err(invalid(
                        "npc ambient row without a look-at radius must have a zero interval",
                    ));
                }
            } else if !(MIN_LOOK_AT_RADIUS..=MAX_LOOK_AT_RADIUS).contains(&row.look_at_radius)
                || !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&row.look_at_interval_ms)
            {
                return Err(invalid(
                    "npc ambient look-at radius/interval is outside 1..=30 m / 500..=600000 ms",
                ));
            }
            if row.animation_ids.is_empty()
                && row.conversation_animation_ids.is_empty()
                && row.idle_animation_ids.is_empty()
                && row.look_at_radius == 0.0
            {
                return Err(invalid("npc ambient row has no animation, idle or look-at"));
            }
            let ambient = AmbientRow {
                animation_ids: row.animation_ids,
                min_interval_ms: row.min_interval_ms,
                max_interval_ms: row.max_interval_ms,
                conversation_animation_ids: row.conversation_animation_ids,
                conversation_min_gap_ms: row.conversation_min_gap_ms,
                conversation_max_gap_ms: row.conversation_max_gap_ms,
                idle_animation_ids: row.idle_animation_ids,
                idle_interval_ms: row.idle_interval_ms,
                look_at_radius: row.look_at_radius,
                look_at_interval_ms: row.look_at_interval_ms,
            };
            for model_id in row.model_ids {
                if rows.insert(model_id, ambient.clone()).is_some() {
                    return Err(invalid("npc ambient binds one client model twice"));
                }
            }
        }
        Ok(Self {
            rows,
            max_packets_per_viewer_per_tick: document.max_packets_per_viewer_per_tick,
            conversation_radius_sq: document.conversation_radius_m.powi(2),
            conversation_groups: Vec::new(),
            talking: BTreeSet::new(),
            idles: BTreeMap::new(),
            look_at: BTreeMap::new(),
        })
    }

    /// (R) The id one actor plays when a firing falls in `(from_ms, to_ms]`.
    ///
    /// The period is `min + h(guid) % (max - min + 1)`, the phase is a second
    /// hash modulo the period, and the k-th firing picks `ids[h3(guid, k)]`.
    /// When a long stall spans several firings, only the latest is returned so
    /// the caller never replays a backlog. No `RandomState`: every viewer and
    /// every restart sees the same schedule.
    pub fn due(&self, guid: u64, model_id: u32, from_ms: u64, to_ms: u64) -> Option<i32> {
        let row = self.rows.get(&model_id)?;
        if row.animation_ids.is_empty() {
            return None;
        }
        let span = row.max_interval_ms - row.min_interval_ms + 1;
        let period = row.min_interval_ms + mix64(guid) % span;
        let phase = mix64(guid ^ PHASE_SALT) % period;
        let last = phase + to_ms.checked_sub(phase)? / period * period;
        if last <= from_ms {
            return None;
        }
        let firing = (last - phase) / period;
        let index = mix64(guid ^ firing.wrapping_add(1).wrapping_mul(FIRING_SALT));
        Some(row.animation_ids[(index % row.animation_ids.len() as u64) as usize])
    }

    /// Up to `max_packets_per_viewer_per_tick` due updates for the known content
    /// actors, lowest GUID first for the slice-1 emote rows (not for a conversation
    /// member, whose turns are its animation), then idle loops, conversation
    /// speakers and look-at actors. An idle update costs two
    /// packets because it sends 35/26 and the base 35/8.
    pub fn due_for_known(
        &self,
        known: &BTreeSet<u64>,
        mut model_of: impl FnMut(u64) -> Option<u32>,
        from_ms: u64,
        to_ms: u64,
    ) -> Vec<AmbientUpdate> {
        let cap = self.max_packets_per_viewer_per_tick;
        let mut updates = Vec::new();
        let mut used = 0;
        for &guid in known {
            if self.talking.contains(&guid) {
                continue;
            }
            let Some(model_id) = model_of(guid) else {
                continue;
            };
            let Some(animation_id) = self.due(guid, model_id, from_ms, to_ms) else {
                continue;
            };
            if used == cap {
                return updates;
            }
            updates.push(AmbientUpdate::Animation { guid, animation_id });
            used += 1;
        }
        for &guid in known {
            let Some(model_id) = model_of(guid) else {
                continue;
            };
            let Some(schedule) = self.idles.get(&guid) else {
                continue;
            };
            if schedule.model_id != model_id {
                continue;
            }
            let Some(animation_id) = schedule.due(from_ms, to_ms) else {
                continue;
            };
            if used + 2 > cap {
                continue;
            }
            updates.push(AmbientUpdate::Idle { guid, animation_id });
            used += 2;
        }
        for group in &self.conversation_groups {
            if used == cap {
                break;
            }
            let Some((guid, model_id, animation_id)) = group.due(from_ms, to_ms) else {
                continue;
            };
            if !known.contains(&guid) || model_of(guid) != Some(model_id) {
                continue;
            }
            updates.push(AmbientUpdate::Conversation { guid, animation_id });
            used += 1;
        }
        for &guid in known {
            if used == cap {
                break;
            }
            let Some(model_id) = model_of(guid) else {
                continue;
            };
            let Some(schedule) = self.look_at.get(&guid) else {
                continue;
            };
            if schedule.model_id != model_id || !schedule.due(from_ms, to_ms) {
                continue;
            }
            updates.push(AmbientUpdate::LookAt {
                guid,
                position: schedule.position,
                radius: schedule.radius,
            });
            used += 1;
        }
        updates
    }
}

fn check_ids(ids: &[i32], what: &str) -> io::Result<()> {
    if ids.len() > MAX_IDS_PER_ROW || ids.iter().any(|&id| id <= 0) {
        return Err(invalid(format!(
            "npc ambient {what} row needs 0..=16 positive ids"
        )));
    }
    let mut seen = BTreeSet::new();
    for &id in ids {
        if !seen.insert(id) {
            return Err(invalid(format!("npc ambient {what} row repeats an id")));
        }
    }
    Ok(())
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
