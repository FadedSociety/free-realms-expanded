//! The live-world forge placement shared by the station refresh unit test and
//! the gateway route test.
//!
//! A live-world `sh_forge_02` placement (model 1733, `data/content/crafting_stations.json`)
//! at a position no shipped station occupies.
use osfr_tools::world::{PLACEMENT_GUID_BASE, Placement, PlacementFile};
use std::{fs, path::Path};

pub const NEAR: [f32; 3] = [321.0, 0.0, 654.0];
pub const AWAY: [f32; 3] = [1_321.0, 0.0, 1_654.0];

/// Write the live-world forge placement, or an empty placement file.
pub fn write_placements(dir: &Path, at: Option<[f32; 3]>) {
    let records = at
        .map(|position| Placement {
            guid: PLACEMENT_GUID_BASE + 1,
            model_id: 1733,
            name_string_id: 0,
            name: None,
            position,
            heading: 0.0,
            scale: 1.0,
            extra: serde_json::Map::new(),
        })
        .into_iter()
        .collect();
    fs::write(
        dir.join("npc_placements.json"),
        serde_json::to_vec(&PlacementFile {
            schema_version: 1,
            records,
        })
        .unwrap(),
    )
    .unwrap();
}
