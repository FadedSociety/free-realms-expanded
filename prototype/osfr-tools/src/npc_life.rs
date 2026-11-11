//! F6 (op-P-npc-life): how a passive world actor stands: its idle pose and its floor.
//!
//! **Pose.** The AddNpc `animation_id` (35/2, reader `00921a10`; the case-2 dispatch `0092f460`
//! stores it at actor `+0x51c`, the idle base `00958290` and 35/26 also write) is what an actor
//! plays at rest: `ProxiedCharacter__Process` (`00973200`) reads it through `00958240` whenever
//! the actor is not moving. The original server set it per actor: three of the 23 captured
//! AddNpc rows (`startup/snapshot.json`) carry 2100 `amb_loop_01`. `npc_poses.json` names the
//! loop per client model (a role the model's own clips show, such as a dwarf miner's pick
//! swings) and per spawn. A model with several loops gives each spawn one of them by GUID, so
//! every viewer and every restart sees the same pose. A row whose AddNpc already carries
//! another id (a boombox box) keeps it. With the base left at `loc_stand` (1) the client's own
//! fidget runs: `AccumulateTimersAndDispatchMotion` (`00970b60`) swaps to code enum 31
//! (`loc_fidget`) 15% of the time once the base has played 4 s, so townsfolk need no row.
//!
//! **Floor.** [`Terrain::floor_at`] (the client's own Y+2 cast) gives the surface under a spawn.
//! A body further than [`SETTLED_M`] from it is set onto it, unless its row carries
//! `keep_height` evidence that the authored height is right: a Legacy spawn-script height, a
//! 2014-placed prop standing level with it (a deck the bake has no collision for), a flyer or
//! a swimmer. Props keep the height they were placed at: the original server's own warpstones
//! stand 0.8-1.0 m above this floor and its radio 1.6 m (`startup/snapshot.json`), so a prop's
//! origin is not its foot. The `props` list names those client models (a model with no walk,
//! run, emote, fidget or combat clip). A row that is one of the zone's placed objects
//! (`terrain_object_id`) never moves. The shipped content already carries these heights
//! (`scripts/npc_floor.py` applies the same rule offline, since a public realm runs without
//! the client-derived terrain), so for it the load-time snap is a check that moves nothing; it
//! grounds what is added later (Studio placements, `realm place`).
//!
//! Client data stays out of production: the loader checks only the shape, and
//! `tests/npc_life.rs` checks every id against the model's own `.adr` slot table.

use crate::terrain::Terrain;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::Path,
    sync::Arc,
};

pub const CONTENT_FILE: &str = "npc_poses.json";
/// AddNpc `animation_id` of a content actor with no pose: `loc_stand`, as 18 of the 23 captured
/// AddNpc rows send it.
pub const STAND: i32 = 1;
/// (R) A spawn this close to its floor keeps its authored height: Legacy rows sit within a few
/// centimetres, and node types sink their models on purpose (`placement_y_offset` -0.05).
pub const SETTLED_M: f32 = 0.1;
const MAX_CONTENT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_IDS: usize = 16;

/// The pose table and, once the session's terrain is known, the floor.
#[derive(Clone, Debug, Default)]
pub struct Life {
    by_model: HashMap<u32, Vec<i32>>,
    by_spawn: HashMap<u64, i32>,
    kept: HashSet<u64>,
    props: HashSet<u32>,
    floor: Option<Arc<Terrain>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    source: Vec<String>,
    models: Vec<ModelRow>,
    spawns: Vec<SpawnRow>,
    props: Vec<PropRow>,
}

/// Client models that are things, not bodies: they keep their authored height.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropRow {
    model_ids: Vec<u32>,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRow {
    model_ids: Vec<u32>,
    model_file_name: String,
    role: String,
    animation_ids: Vec<i32>,
    source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpawnRow {
    guid: u64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    animation_id: Option<i32>,
    /// Evidence that this spawn stands above (or below) the floor on purpose.
    #[serde(default)]
    keep_height: Option<String>,
    source: String,
}

impl Life {
    /// The table beside `directory`; a content set without the file poses nothing.
    pub fn load(directory: &Path) -> io::Result<Self> {
        let path = directory.join(CONTENT_FILE);
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error),
        };
        if !metadata.is_file() || metadata.len() > MAX_CONTENT_BYTES {
            return Err(invalid("npc poses must be a regular file under 4 MiB"));
        }
        Self::from_json(&fs::read(path)?)
    }

    fn from_json(bytes: &[u8]) -> io::Result<Self> {
        let document: Document = serde_json::from_slice(bytes).map_err(invalid)?;
        if document.schema_version != 1 || document.source.iter().all(|s| s.trim().is_empty()) {
            return Err(invalid("npc poses need schema 1 and a source"));
        }
        let mut life = Self::default();
        for row in document.models {
            if row.model_ids.is_empty()
                || [&row.model_file_name, &row.role, &row.source]
                    .iter()
                    .any(|text| text.trim().is_empty())
            {
                return Err(invalid(format!(
                    "npc pose model row {:?} needs model ids, a file, a role and a source",
                    row.model_file_name
                )));
            }
            check_ids(&row.animation_ids, &row.model_file_name)?;
            for model in row.model_ids {
                if life
                    .by_model
                    .insert(model, row.animation_ids.clone())
                    .is_some()
                {
                    return Err(invalid(format!("npc pose model {model} is listed twice")));
                }
            }
        }
        for row in document.spawns {
            let keep = row.keep_height.as_deref().map(str::trim);
            if row.source.trim().is_empty()
                || keep.is_some_and(str::is_empty)
                || (row.animation_id.is_none() && keep.is_none())
            {
                return Err(invalid(format!(
                    "npc pose spawn {} needs a source and an animation id or keep_height evidence",
                    row.guid
                )));
            }
            if let Some(id) = row.animation_id {
                check_ids(&[id], row.name.as_deref().unwrap_or("spawn"))?;
                if life.by_spawn.insert(row.guid, id).is_some() {
                    return Err(invalid(format!(
                        "npc pose spawn {} is listed twice",
                        row.guid
                    )));
                }
            }
            if keep.is_some() && !life.kept.insert(row.guid) {
                return Err(invalid(format!(
                    "npc pose spawn {} keeps its height twice",
                    row.guid
                )));
            }
        }
        for row in document.props {
            if row.model_ids.is_empty() || row.source.trim().is_empty() {
                return Err(invalid("npc pose prop row needs model ids and a source"));
            }
            for model in row.model_ids {
                if !life.props.insert(model) {
                    return Err(invalid(format!(
                        "npc pose prop model {model} is listed twice"
                    )));
                }
            }
        }
        Ok(life)
    }

    /// The idle id `guid` stands in: its own row, else one of its model's loops by GUID.
    pub fn pose(&self, guid: u64, model_id: i32) -> Option<i32> {
        if let Some(id) = self.by_spawn.get(&guid) {
            return Some(*id);
        }
        let ids = self.by_model.get(&u32::try_from(model_id).ok()?)?;
        let index = crate::hash::mix64(guid) % ids.len() as u64;
        ids.get(usize::try_from(index).ok()?).copied()
    }

    /// The height `guid` at `p` is set to: its floor, when the terrain is known, the actor is a
    /// body (not a prop model), the row does not keep its height and the floor is more than [`SETTLED_M`]
    /// away. `None` keeps the authored height.
    pub fn floor(&self, zone: u32, guid: u64, model_id: i32, p: [f32; 3]) -> Option<f32> {
        if self.kept.contains(&guid) || self.is_prop(model_id) {
            return None;
        }
        let floor = self.floor.as_ref()?.floor_at(zone, p)?;
        ((floor - p[1]).abs() > SETTLED_M).then_some(floor)
    }

    /// Whether the floor is known; [`Self::set_floor`] takes only the first terrain.
    pub fn has_floor(&self) -> bool {
        self.floor.is_some()
    }

    pub fn set_floor(&mut self, terrain: Arc<Terrain>) {
        self.floor.get_or_insert(terrain);
    }

    /// Whether this spawn's row gives evidence that its authored height is right.
    pub fn keeps_height(&self, guid: u64) -> bool {
        self.kept.contains(&guid)
    }

    /// Whether this client model is a thing (a warpstone, a mailbox, a plant node) that keeps
    /// the height it was placed at: the original server's warpstones stand 0.8-1.0 m above the
    /// floor and its radio 1.6 m (on a crate), so a prop's origin is not its foot.
    pub fn is_prop(&self, model_id: i32) -> bool {
        u32::try_from(model_id).is_ok_and(|model| self.props.contains(&model))
    }
}

fn check_ids(ids: &[i32], owner: &str) -> io::Result<()> {
    let mut seen = HashSet::new();
    if ids.is_empty() || ids.len() > MAX_IDS || ids.iter().any(|&id| id <= 0 || !seen.insert(id)) {
        return Err(invalid(format!(
            "npc pose {owner:?} needs 1..=16 distinct positive animation ids"
        )));
    }
    Ok(())
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn table(models: serde_json::Value, spawns: serde_json::Value) -> io::Result<Life> {
        let props = json!([{"model_ids": [280], "source": "test"}]);
        let document = json!({"schema_version": 1, "source": ["test"], "models": models, "spawns": spawns, "props": props});
        Life::from_json(document.to_string().as_bytes())
    }

    fn miner() -> serde_json::Value {
        json!([{"model_ids": [382], "model_file_name": "dwarf_miner_m.adr", "role": "miner",
                "animation_ids": [2100, 2110, 2120], "source": "test (R)"}])
    }

    #[test]
    fn a_spawn_row_beats_its_model_and_a_model_pose_is_stable_per_guid() {
        let life = table(
            miner(),
            json!([{"guid": 7, "animation_id": 2140, "source": "test"},
                   {"guid": 8, "keep_height": "a flyer", "source": "test"}]),
        )
        .unwrap();
        assert_eq!(life.pose(7, 382), Some(2140));
        let picks: HashSet<_> = (100..400).filter_map(|guid| life.pose(guid, 382)).collect();
        assert_eq!(
            picks,
            HashSet::from([2100, 2110, 2120]),
            "every loop is used"
        );
        assert_eq!(life.pose(123, 382), life.pose(123, 382));
        assert_eq!(
            life.pose(123, 999),
            None,
            "an unlisted model keeps its stand"
        );
        assert_eq!(life.pose(123, -1), None);
        assert!(life.keeps_height(8) && !life.keeps_height(7));
    }

    #[test]
    fn the_loader_refuses_rows_that_cannot_be_sent() {
        let bad_model = |ids: serde_json::Value| json!([{"model_ids": [1], "model_file_name": "a.adr", "role": "r", "animation_ids": ids, "source": "s"}]);
        for ids in [
            json!([]),
            json!([0]),
            json!([5, 5]),
            json!((1..=17).collect::<Vec<_>>()),
        ] {
            assert!(table(bad_model(ids), json!([])).is_err());
        }
        assert!(
            table(json!([]), json!([{"guid": 1, "source": "s"}])).is_err(),
            "no pose, no evidence"
        );
        assert!(
            table(
                json!([]),
                json!([{"guid": 1, "keep_height": " ", "source": "s"}])
            )
            .is_err()
        );
        assert!(
            table(
                json!([]),
                json!([{"guid": 1, "animation_id": 3, "source": "s"},
                                    {"guid": 1, "animation_id": 4, "source": "s"}])
            )
            .is_err()
        );
        let twice = json!([miner()[0].clone(), miner()[0].clone()]);
        assert!(table(twice, json!([])).is_err(), "one model in two rows");
        assert!(
            table(miner(), json!([{"guid": 1, "pose": 3, "source": "s"}])).is_err(),
            "unknown field"
        );
        let props = |ids: serde_json::Value| json!({"schema_version": 1, "source": ["t"], "models": [], "spawns": [], "props": [{"model_ids": ids, "source": "s"}]});
        for ids in [json!([]), json!([280, 280])] {
            assert!(
                Life::from_json(props(ids).to_string().as_bytes()).is_err(),
                "empty or repeated prop"
            );
        }
        assert_eq!(
            Life::load(Path::new("no such dir")).unwrap().pose(1, 382),
            None
        );
    }

    #[test]
    fn the_floor_moves_only_unkept_bodies_off_by_more_than_the_tolerance() {
        let mut grid = crate::terrain::HeightGrid::new([0.0, 0.0], [10.0, 10.0], 2.0).unwrap();
        grid.add_triangle([[0.0, 5.0, 0.0], [10.0, 5.0, 0.0], [10.0, 5.0, 10.0]]);
        grid.add_triangle([[0.0, 5.0, 0.0], [10.0, 5.0, 10.0], [0.0, 5.0, 10.0]]);
        let terrain = Arc::new(Terrain::from_grids([(0, grid)]));
        let mut life = table(
            json!([]),
            json!([{"guid": 9, "keep_height": "a flyer", "source": "s"}]),
        )
        .unwrap();
        let body = 382;
        assert_eq!(
            life.floor(0, 1, body, [5.0, 9.0, 5.0]),
            None,
            "no terrain yet"
        );
        life.set_floor(terrain);
        assert_eq!(
            life.floor(0, 1, body, [5.0, 9.0, 5.0]),
            Some(5.0),
            "a floater comes down"
        );
        assert_eq!(
            life.floor(0, 1, body, [5.0, 3.5, 5.0]),
            Some(5.0),
            "a body 1.5 m under comes up"
        );
        assert_eq!(
            life.floor(0, 1, body, [5.0, 5.05, 5.0]),
            None,
            "within tolerance keeps its height"
        );
        assert_eq!(
            life.floor(0, 1, body, [5.0, -20.0, 5.0]),
            None,
            "a cave under the hills keeps its height"
        );
        assert_eq!(
            life.floor(0, 9, body, [5.0, 9.0, 5.0]),
            None,
            "a kept row keeps its height"
        );
        assert_eq!(
            life.floor(0, 1, 280, [5.0, 5.9, 5.0]),
            None,
            "a prop keeps its height"
        );
        assert_eq!(
            life.floor(3, 1, body, [5.0, 9.0, 5.0]),
            None,
            "no grid for the zone"
        );
    }
}
