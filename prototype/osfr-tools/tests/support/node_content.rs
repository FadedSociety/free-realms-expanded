//! The shipped gather-node tables, read through the production loader once for every node
//! suite. `npc_visibility.rs` and `pickup_fixture.rs` include this file. Test-only.
#![allow(dead_code)]

use osfr_tools::harvest_nodes::{GatheringNodes, NodeActor};
use std::path::{Path, PathBuf};

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The shipped node tables (`data/content`), read through the production loader.
pub fn shipped_nodes() -> GatheringNodes {
    GatheringNodes::load(&content_dir())
        .expect("the shipped node tables load")
        .expect("the shipped node tables are present")
}

/// The placed pickups of one `quest_nodes.json` node type, in guid order.
pub fn pickups(node_type: &str) -> Vec<NodeActor> {
    let nodes = shipped_nodes();
    let mut rows: Vec<NodeActor> = nodes
        .actors()
        .filter(|actor| {
            nodes
                .pickup_kind(actor.guid)
                .is_some_and(|kind| kind == node_type)
        })
        .cloned()
        .collect();
    rows.sort_by_key(|actor| actor.guid);
    rows
}
