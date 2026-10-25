//! Light NPC-visibility helpers: the content root and a viewer's 35/2 AddNpc view.
//!
//! `npc_quest_chain.rs` re-exports these, so the quest-chain suites keep one rule;
//! a test that needs only the draw path includes this file directly and skips the
//! heavy fixture. Test-only.
#![allow(dead_code)]

use osfr_tools::{
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// A fresh open-world view from `position`: every actor `visibility` sends, as parsed 35/2
/// AddNpc bodies (client reader `0x00921a10`).
pub fn seen_from(visibility: &NpcVisibility, position: [f32; 3]) -> Vec<NpcSpawn> {
    visibility
        .clone()
        .update(0, position)
        .unwrap()
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).unwrap())
        .collect()
}

/// The same view through the live gateway fan-out: `Gameplay::shared_npc_view` calls
/// `NpcVisibility::shared_view` (a shared-mode `update` sends nothing), here with no
/// field actors and nothing known yet.
pub fn shared_seen_from(visibility: &NpcVisibility, position: [f32; 3]) -> Vec<NpcSpawn> {
    visibility
        .shared_view(0, position, &[], &BTreeSet::new())
        .unwrap()
        .1
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .map(|packet| parse_add_npc(packet).unwrap())
        .collect()
}
