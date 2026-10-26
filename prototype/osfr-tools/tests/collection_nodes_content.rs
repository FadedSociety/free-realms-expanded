//! T523: the community collection-node data is imported whole and agrees with the collection
//! catalogue (T447). T524: the gameplay node loader reads it as the third node kind.

use osfr_tools::{
    collection_catalogue::CollectionCatalogueContent,
    harvest_nodes::{GatheringNodes, HarvestNodesContent},
};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const FILE: &str = "collection_nodes.json";
const MAX_FILE_BYTES: u64 = 1 << 20;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn data_file(name: &str) -> PathBuf {
    content_dir().join(name)
}

fn load() -> HarvestNodesContent {
    HarvestNodesContent::load_from(data_file(FILE)).expect("collection nodes load")
}

fn type_keys(content: &HarvestNodesContent) -> BTreeSet<&str> {
    content
        .node_types
        .iter()
        .map(|node_type| node_type.key.as_str())
        .collect()
}

fn pool_keys(content: &HarvestNodesContent) -> BTreeSet<&str> {
    content.pools.iter().map(|pool| pool.key.as_str()).collect()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-collection-nodes-{}-{}",
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

#[test]
fn collection_nodes_load_with_the_imported_counts_and_unique_ids() {
    let size = fs::metadata(data_file(FILE)).unwrap().len();
    assert!(size < MAX_FILE_BYTES, "collection nodes are {size} bytes");
    let content = load();
    assert_eq!(content.schema_version, 1);
    assert_eq!(
        (
            content.node_types.len(),
            content.pools.len(),
            content.spawns.len()
        ),
        (12, 34, 847)
    );
    assert_eq!(type_keys(&content).len(), content.node_types.len());
    assert_eq!(pool_keys(&content).len(), content.pools.len());
    let ids: BTreeSet<u32> = content.spawns.iter().map(|spawn| spawn.id).collect();
    assert_eq!(ids.len(), content.spawns.len());
    for pool in &content.pools {
        assert_eq!(pool.zone_definition_id, 1);
    }
    for spawn in &content.spawns {
        assert_eq!(spawn.zone_definition_id, 1);
        assert!(
            spawn.position.iter().all(|value| value.is_finite()),
            "spawn {} position",
            spawn.id
        );
        assert!(spawn.heading.is_finite(), "spawn {} heading", spawn.id);
    }
}

#[test]
fn pools_reference_types_and_spawns_fit_the_caps_and_respawn_policy() {
    let content = load();
    let types = type_keys(&content);
    let mut pools = HashMap::new();
    for pool in &content.pools {
        assert!(
            types.contains(pool.node_type.as_str()),
            "{} names an unknown type",
            pool.key
        );
        assert!(pools.insert(pool.key.as_str(), pool).is_none());
        assert!(pool.max_active_nodes > 0, "{} max", pool.key);
        let rare = pool.node_type.starts_with("rare-");
        assert_eq!(
            pool.respawn_seconds,
            if rare { 300 } else { 60 },
            "{} respawn",
            pool.key
        );
        assert!(
            pool.drop_table.iter().all(|drop| drop.weight > 0),
            "{} drop weight",
            pool.key
        );
    }
    for spawn in &content.spawns {
        let pool = pools
            .get(spawn.pool_key.as_str())
            .unwrap_or_else(|| panic!("spawn {} names an unknown pool", spawn.id));
        assert_eq!(spawn.zone_definition_id, pool.zone_definition_id);
    }
    let common = content
        .pools
        .iter()
        .filter(|pool| !pool.node_type.starts_with("rare-"))
        .count();
    let rare = content
        .pools
        .iter()
        .filter(|pool| pool.node_type.starts_with("rare-"))
        .count();
    assert_eq!((common, rare), (17, 17));

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for spawn in &content.spawns {
        *counts.entry(spawn.pool_key.as_str()).or_default() += 1;
    }
    for pool in &content.pools {
        let count = counts.get(pool.key.as_str()).copied().unwrap_or(0);
        assert!(count > 0, "{} has no spawns", pool.key);
        assert!(
            pool.max_active_nodes as usize <= count,
            "{} cap {} over {count} spawns",
            pool.key,
            pool.max_active_nodes
        );
    }
}

#[test]
fn data_canary_central_flowers_and_the_sanctuary_only_fields() {
    let content = load();
    let pool = content
        .pools
        .iter()
        .find(|pool| pool.key == "sanctuary-central-flowers")
        .expect("central flowers pool");
    assert_eq!(pool.node_type, "flowers");
    assert_eq!(pool.max_active_nodes, 22);
    assert_eq!(pool.respawn_seconds, 60);
    assert_eq!(
        content
            .spawns
            .iter()
            .filter(|spawn| spawn.pool_key == pool.key)
            .count(),
        111
    );

    let find = |key: &str| {
        content
            .node_types
            .iter()
            .find(|node_type| node_type.key == key)
            .unwrap_or_else(|| panic!("{key} type"))
    };
    let water = find("sparkling-water");
    assert_eq!(water.model_id, 271);
    assert_eq!(water.composite_effect_id, 15381);
    assert_eq!(water.extra_composite_effect_ids, vec![5742]);
    let rare_water = find("rare-sparkling-water");
    assert_eq!(rare_water.composite_effect_id, 15382);
    assert_eq!(rare_water.extra_composite_effect_ids, vec![5740]);
    let rare = find("rare-flowers");
    assert_eq!(rare.model_id, 670);
    assert_eq!(rare.sub_text_name_id, Some(-2118222202));
    assert_eq!(find("flowers").sub_text_name_id, None);
}

#[test]
fn every_drop_item_resolves_to_one_entry_or_is_a_listed_gap() {
    let content = load();
    let catalogue = CollectionCatalogueContent::load(&content_dir())
        .expect("catalogue load")
        .expect("collection catalogue file");
    let drops: BTreeSet<u32> = content
        .pools
        .iter()
        .flat_map(|pool| &pool.drop_table)
        .map(|drop| drop.item_definition_id)
        .collect();
    assert!(!drops.is_empty());
    let expected: Vec<u32> = drops
        .iter()
        .copied()
        .filter(|item| catalogue.entry_for_item(*item as i32).is_none())
        .collect();
    assert_eq!(content.gaps.drop_items_without_entry, expected);
    let gaps: BTreeSet<u32> = content
        .gaps
        .drop_items_without_entry
        .iter()
        .copied()
        .collect();
    assert_eq!(gaps.len(), content.gaps.drop_items_without_entry.len());
    assert!(gaps.is_subset(&drops));
    for item in &drops {
        assert_eq!(
            catalogue.entry_for_item(*item as i32).is_some(),
            !gaps.contains(item),
            "drop item {item}"
        );
    }
}

#[test]
fn legacy_mushroom_and_mining_files_still_load_with_the_new_fields_defaulted() {
    let harvest = HarvestNodesContent::load_from(data_file("harvest_nodes.json")).unwrap();
    assert_eq!(
        (
            harvest.node_types.len(),
            harvest.pools.len(),
            harvest.spawns.len()
        ),
        (2, 2, 76)
    );
    assert!(harvest.gaps.drop_items_without_entry.is_empty());
    assert!(
        harvest
            .node_types
            .iter()
            .all(|row| row.extra_composite_effect_ids.is_empty()
                && row.sub_text_name_id.is_none()
                && !row.drop_table.is_empty())
    );
    assert!(harvest.pools.iter().all(|pool| pool.drop_table.is_empty()));

    let mining = HarvestNodesContent::load_from(data_file("mining_nodes.json")).unwrap();
    assert_eq!(
        (
            mining.node_types.len(),
            mining.pools.len(),
            mining.spawns.len()
        ),
        (1, 1, 5)
    );
    assert_eq!(mining.node_types[0].model_id, 259);
    assert!(mining.node_types[0].drop_table.is_empty());
    assert!(mining.pools.iter().all(|pool| pool.drop_table.is_empty()));
}

#[test]
fn the_gathering_loader_reads_collection_nodes() {
    let dir = TempDir::new();
    fs::copy(data_file(FILE), dir.0.join(FILE)).unwrap();
    let nodes = GatheringNodes::load(&dir.0)
        .unwrap()
        .expect("T524 loads collection_nodes.json as sparkling nodes");
    let actors: Vec<_> = nodes
        .actors()
        .filter(|a| nodes.is_collect(a.guid))
        .collect();
    let content = load();
    let expected: usize = content
        .pools
        .iter()
        .map(|pool| pool.max_active_nodes as usize)
        .sum();
    assert_eq!(
        actors.len(),
        expected,
        "one placed actor per active pool slot"
    );
    assert!(
        actors
            .iter()
            .all(|a| a.presentation.composite_effect_id.is_some_and(|id| id != 0)),
        "every collection type sparkles"
    );
    // T524 review: the full AddNpc presentation is the collection kind's; a mining or quest
    // actor keeps the captured template fields, and the Chef mushrooms only gain their sparkle.
    for file in ["mining_nodes.json", "quest_nodes.json"] {
        let dir = TempDir::new();
        fs::copy(data_file(file), dir.0.join(file)).unwrap();
        let nodes = GatheringNodes::load(&dir.0).unwrap().expect("a node file");
        assert!(
            nodes.actors().all(|actor| actor.presentation
                == osfr_tools::harvest_nodes::NodePresentation::default()),
            "{file} actors must keep the captured AddNpc template fields"
        );
    }
    let dir = TempDir::new();
    fs::copy(
        data_file("harvest_nodes.json"),
        dir.0.join("harvest_nodes.json"),
    )
    .unwrap();
    let harvest = GatheringNodes::load(&dir.0)
        .unwrap()
        .expect("mushroom nodes");
    assert!(harvest.actors().all(|actor| {
        actor
            .presentation
            .composite_effect_id
            .is_some_and(|id| id != 0)
            && actor.presentation.interact_range.is_none()
            && actor.presentation.sub_text_name_id.is_none()
            && actor.presentation.extra_composite_effect_id.is_none()
            && actor.companion_guid().is_none()
    }));
    // Control: an empty directory still places nothing.
    let empty = TempDir::new();
    assert!(GatheringNodes::load(&empty.0).unwrap().is_none());
}

#[test]
fn provenance_and_policy_tags_are_present() {
    let content = load();
    let revision = content.provenance.get("source_repository").unwrap();
    assert!(revision.contains("10f129e7f3e2094304d20ccee6e3f7136a5e889d"));
    assert!(content.provenance.get("license").unwrap().contains("AGPL"));
    for pool in &content.pools {
        assert!(
            pool.source_tag
                .as_deref()
                .unwrap_or("")
                .starts_with("sanctuary@10f129e:")
        );
        assert_eq!(
            pool.source_tags.get("max_active_nodes").map(String::as_str),
            Some("community_policy_R")
        );
        assert_eq!(
            pool.source_tags.get("respawn_seconds").map(String::as_str),
            Some("community_policy_R")
        );
        for drop in &pool.drop_table {
            assert_eq!(
                drop.source_tags.get("weight").map(String::as_str),
                Some("community_policy_R")
            );
        }
    }
    for spawn in &content.spawns {
        assert!(
            spawn
                .source_tag
                .as_deref()
                .unwrap_or("")
                .starts_with("sanctuary@10f129e:")
        );
        assert_eq!(
            spawn.source_tags.get("position").map(String::as_str),
            Some("community_policy_R")
        );
        assert_eq!(
            spawn.source_tags.get("heading").map(String::as_str),
            Some("community_policy_R")
        );
    }
}
