//! F701: every live node source and its effect companions share one guid namespace.
//!
//! A node guid is "NOD", one tag byte, then the content spawn id. T524's invisible effect
//! companions also use the tag byte (tag 4), so a source that reuses a tag aliases its spawns.
//! Each node file is loaded on its own through the real `GatheringNodes::load`, which makes the
//! tag the production loader assigns that source observable, then all guids and tags are
//! combined.

use osfr_tools::harvest_nodes::GatheringNodes;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const FILES: [&str; 4] = [
    "harvest_nodes.json",
    "mining_nodes.json",
    "quest_nodes.json",
    "collection_nodes.json",
];
/// T524: the invisible effect companion of a collection node keeps the spawn id and uses tag 4.
const COMPANION_TAG: u64 = 4;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-node-guid-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn tag(guid: u64) -> u64 {
    (guid >> 32) & 0xff
}

/// The guids the real loader places from `file` alone. `None` means the file is not wired into
/// `GatheringNodes::load`.
fn load_one(file: &str) -> Option<Vec<u64>> {
    let dir = TempDir::new();
    fs::copy(content_dir().join(file), dir.0.join(file)).unwrap();
    GatheringNodes::load(&dir.0)
        .unwrap()
        .map(|nodes| nodes.actors().map(|actor| actor.guid).collect())
}

#[test]
fn no_two_live_node_guids_share_a_tag_or_a_value() {
    let mut all = BTreeSet::new();
    let mut tags = BTreeSet::new();
    for file in FILES {
        let guids = load_one(file).unwrap_or_else(|| panic!("{file} is missing from the loader"));
        let source_tags: BTreeSet<u64> = guids.iter().map(|guid| tag(*guid)).collect();
        assert_eq!(
            source_tags.len(),
            1,
            "{file} was placed under more than one tag: {source_tags:?}"
        );
        let source_tag = *source_tags
            .first()
            .expect("a loaded node source has actors");
        assert!(
            source_tag < 256,
            "{file} tag {source_tag} does not fit one byte"
        );
        assert!(
            tags.insert(source_tag),
            "{file} reuses node tag {source_tag} already owned by another source"
        );
        for guid in &guids {
            assert!(
                all.insert(*guid),
                "{file} guid {guid:#x} is also another source's guid"
            );
        }
        if file == "collection_nodes.json" {
            // T524: every collection node's effect companion keeps the spawn id and uses tag 4.
            assert_ne!(
                source_tag, COMPANION_TAG,
                "collection tag aliases its own effect companions"
            );
            for guid in &guids {
                let companion = (*guid & !(0xff_u64 << 32)) | (COMPANION_TAG << 32);
                assert!(
                    !all.contains(&companion),
                    "collection node {guid:#x} and its companion {companion:#x} collide"
                );
            }
        }
    }
    assert!(
        !tags.contains(&COMPANION_TAG),
        "a live node source uses the tag-4 effect-companion namespace"
    );
}
