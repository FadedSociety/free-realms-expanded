//! T507: per-model ambient idle and emote animations for passive content NPCs.
//!
//! The rows are Sanctuary's `Resources/NpcAmbient.json`
//! (`Sanctuary-main-20261019@10f129e`, AGPL-3.0 values only, no code). Upstream
//! records measured-looking Min/MaxIntervalMs but names no source, so the
//! intervals are (R). The schedule is stateless: an actor's firing times depend
//! only on its GUID, so every viewer that knows the actor plays the same
//! animation in the same tick window.

use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

const CONTENT_FILE: &str = "npc_ambient.json";
const MAX_CONTENT_BYTES: u64 = 1024 * 1024;
const MAX_ROWS: usize = 512;
const MAX_IDS_PER_ROW: usize = 16;
const MIN_INTERVAL_MS: u64 = 500;
const MAX_INTERVAL_MS: u64 = 600_000;
const MAX_PACKETS_PER_VIEWER_PER_TICK: std::ops::RangeInclusive<usize> = 1..=16;
/// Distinct domain separators for the three independent hashes.
const PHASE_SALT: u64 = 0x9e37_79b9_7f4a_7c15;
const FIRING_SALT: u64 = 0xd1b5_4a32_d192_ed03;

#[derive(Clone, Debug)]
pub struct NpcAmbient {
    rows: BTreeMap<u32, AmbientRow>,
    max_packets_per_viewer_per_tick: usize,
}

#[derive(Clone, Debug)]
struct AmbientRow {
    animation_ids: Vec<i32>,
    min_interval_ms: u64,
    max_interval_ms: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    source: String,
    max_packets_per_viewer_per_tick: usize,
    rows: Vec<RowDocument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowDocument {
    model_ids: Vec<u32>,
    model_file_name: String,
    animation_ids: Vec<i32>,
    min_interval_ms: u64,
    max_interval_ms: u64,
    source: String,
}

impl NpcAmbient {
    /// The table beside `directory`, or `None` when the content set has none.
    pub fn load(directory: &Path) -> io::Result<Option<Self>> {
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
            .map(Some)
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
            if row.animation_ids.is_empty() || row.animation_ids.len() > MAX_IDS_PER_ROW {
                return Err(invalid("npc ambient row needs 1..=16 animation ids"));
            }
            let mut ids = BTreeSet::new();
            for &id in &row.animation_ids {
                if id <= 0 || !ids.insert(id) {
                    return Err(invalid(
                        "npc ambient row repeats or has a nonpositive animation id",
                    ));
                }
            }
            if !(MIN_INTERVAL_MS..=MAX_INTERVAL_MS).contains(&row.min_interval_ms)
                || row.max_interval_ms < row.min_interval_ms
                || row.max_interval_ms > MAX_INTERVAL_MS
            {
                return Err(invalid(
                    "npc ambient row intervals are outside 500..=600000 ms",
                ));
            }
            let ambient = AmbientRow {
                animation_ids: row.animation_ids,
                min_interval_ms: row.min_interval_ms,
                max_interval_ms: row.max_interval_ms,
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
        let span = row.max_interval_ms - row.min_interval_ms + 1;
        let period = row.min_interval_ms + splitmix64(guid) % span;
        let phase = splitmix64(guid ^ PHASE_SALT) % period;
        let last = phase + to_ms.checked_sub(phase)? / period * period;
        if last <= from_ms {
            return None;
        }
        let firing = (last - phase) / period;
        let index = splitmix64(guid ^ firing.wrapping_add(1).wrapping_mul(FIRING_SALT));
        Some(row.animation_ids[(index % row.animation_ids.len() as u64) as usize])
    }

    /// Up to `max_packets_per_viewer_per_tick` due `(guid, id)` pairs for the
    /// known content actors, lowest GUID first. `model_of` owns the passive
    /// content-actor rule (excluded field actors have no model).
    pub fn due_for_known(
        &self,
        known: &BTreeSet<u64>,
        mut model_of: impl FnMut(u64) -> Option<u32>,
        from_ms: u64,
        to_ms: u64,
    ) -> Vec<(u64, i32)> {
        let mut due = Vec::new();
        for &guid in known {
            let Some(model_id) = model_of(guid) else {
                continue;
            };
            let Some(id) = self.due(guid, model_id, from_ms, to_ms) else {
                continue;
            };
            due.push((guid, id));
            if due.len() == self.max_packets_per_viewer_per_tick {
                break;
            }
        }
        due
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const GUID: u64 = 100_000_001_631;

    fn row(
        model_ids: serde_json::Value,
        ids: serde_json::Value,
        min: u64,
        max: u64,
    ) -> serde_json::Value {
        json!({
            "model_ids": model_ids,
            "model_file_name": "test.adr",
            "animation_ids": ids,
            "min_interval_ms": min,
            "max_interval_ms": max,
            "source": "test (R)",
        })
    }

    fn content(rows: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "source": "test content",
            "max_packets_per_viewer_per_tick": 4,
            "rows": rows,
        }))
        .unwrap()
    }

    fn table(rows: serde_json::Value) -> NpcAmbient {
        NpcAmbient::from_json(&content(rows)).unwrap()
    }

    fn one_row(ids: serde_json::Value, min: u64, max: u64) -> NpcAmbient {
        table(json!([row(json!([1]), ids, min, max)]))
    }

    #[test]
    fn the_same_guid_and_window_are_deterministic() {
        let ambient = one_row(json!([3302, 3306]), 500, 5000);
        for from in [0, 1, 999, 4_000] {
            let first = ambient.due(GUID, 1, from, from + 6_000);
            let second = ambient.due(GUID, 1, from, from + 6_000);
            assert_eq!(first, second);
            assert!(first.is_some(), "a six-period window holds a firing");
        }
    }

    #[test]
    fn ten_periods_fire_ten_times_and_never_twice_in_the_next_window() {
        let ambient = one_row(json!([3302]), 1_000, 1_000);
        let mut firings = 0;
        for t in 0..10_000 {
            if let Some(id) = ambient.due(GUID, 1, t, t + 1) {
                assert_eq!(id, 3302);
                firings += 1;
                assert!(
                    ambient.due(GUID, 1, t + 1, t + 2).is_none(),
                    "a firing must not repeat in the next window"
                );
            }
        }
        assert_eq!(firings, 10, "one firing per period");
    }

    #[test]
    fn the_per_viewer_cap_keeps_the_lowest_guids() {
        let capped = NpcAmbient::from_json(
            &serde_json::to_vec(&json!({
                "schema_version": 1,
                "source": "test content",
                "max_packets_per_viewer_per_tick": 2,
                "rows": [row(json!([1]), json!([3302]), 1_000, 1_000)],
            }))
            .unwrap(),
        )
        .unwrap();
        let known: BTreeSet<u64> = [3, 1, 2].into_iter().collect();
        let due = capped.due_for_known(&known, |_| Some(1), 0, 3_000);
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].0, 1);
        assert_eq!(due[1].0, 2);
    }

    #[test]
    fn unknown_models_are_not_scheduled() {
        let ambient = one_row(json!([3302]), 1_000, 2_000);
        assert_eq!(ambient.due(GUID, 99, 0, 10_000), None);
        let known: BTreeSet<u64> = [GUID].into_iter().collect();
        assert!(
            ambient
                .due_for_known(&known, |_| None, 0, 10_000)
                .is_empty()
        );
    }

    #[test]
    fn invalid_rows_are_a_load_error() {
        let cases = [
            row(json!([1]), json!([3302]), 2_000, 1_000),
            row(json!([1]), json!([]), 1_000, 1_000),
            row(json!([1]), json!([0]), 1_000, 1_000),
            row(json!([1]), json!([3302, 3302]), 1_000, 1_000),
            row(json!([1]), json!([3302]), 499, 1_000),
            row(json!([1]), json!([3302]), 1_000, 600_001),
            {
                let mut too_many = Vec::new();
                for id in 1..=17 {
                    too_many.push(id);
                }
                json!({
                    "model_ids": [1],
                    "model_file_name": "test.adr",
                    "animation_ids": too_many,
                    "min_interval_ms": 1_000,
                    "max_interval_ms": 1_000,
                    "source": "test (R)",
                })
            },
            json!({
                "model_ids": [],
                "model_file_name": "test.adr",
                "animation_ids": [3302],
                "min_interval_ms": 1_000,
                "max_interval_ms": 1_000,
                "source": "test (R)",
            }),
        ];
        for case in cases {
            let bytes = content(json!([case]));
            assert!(NpcAmbient::from_json(&bytes).is_err(), "{case}");
        }
        // Two rows binding the same client model are ambiguous.
        let duplicate = content(json!([
            row(json!([1]), json!([3302]), 1_000, 1_000),
            row(json!([1]), json!([3306]), 1_000, 1_000),
        ]));
        assert!(NpcAmbient::from_json(&duplicate).is_err());
        // The per-viewer cap is bounded too.
        for cap in [0, 17] {
            let bytes = serde_json::to_vec(&json!({
                "schema_version": 1,
                "source": "test content",
                "max_packets_per_viewer_per_tick": cap,
                "rows": [row(json!([1]), json!([3302]), 1_000, 1_000)],
            }))
            .unwrap();
            assert!(NpcAmbient::from_json(&bytes).is_err());
        }
    }

    /// The client reads 35/8 as a 4-byte header (`0x008d6830`: u16 opcode, u16
    /// subtype), u64 GUID, i32 animation, i32 zero, u8 flags (`0x008e5dd0`).
    #[test]
    fn ambient_packets_match_the_client_reader_layout() {
        let packet = crate::encounter::animation(GUID, 3302, false);
        let mut expected = vec![35, 0, 8, 0];
        expected.extend(GUID.to_le_bytes());
        expected.extend(3302_i32.to_le_bytes());
        expected.extend([0; 4]);
        expected.push(0);
        assert_eq!(packet, expected);
    }
}
