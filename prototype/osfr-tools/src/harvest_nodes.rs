//! Harvest-node content imported from the pinned Legacy data tables.

use serde::Deserialize;
use std::{collections::BTreeMap, fs, io, path::Path};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodesContent {
    pub schema_version: u32,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
    pub node_types: Vec<HarvestNodeType>,
    pub pools: Vec<HarvestNodePool>,
    pub spawns: Vec<HarvestNodeSpawn>,
}

impl HarvestNodesContent {
    pub fn load_from(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeType {
    pub key: String,
    pub name: String,
    pub model_id: u32,
    pub scale: f32,
    pub composite_effect_id: u32,
    pub interact_range: u32,
    pub cursor_id: u32,
    pub placement_y_offset: f32,
    pub drop_table: Vec<HarvestNodeDrop>,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeDrop {
    pub item_definition_id: u32,
    pub weight: u32,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodePool {
    pub key: String,
    pub zone_definition_id: u32,
    pub node_type: String,
    pub max_active_nodes: u32,
    pub respawn_seconds: u32,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct HarvestNodeSpawn {
    pub id: u32,
    pub pool_key: String,
    pub zone_definition_id: u32,
    pub position: [f32; 3],
    pub heading: f32,
    #[serde(default)]
    pub source_tag: Option<String>,
    #[serde(default)]
    pub source_tags: BTreeMap<String, String>,
}
