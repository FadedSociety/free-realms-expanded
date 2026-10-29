use osfr_tools::harvest_nodes::HarvestNodesContent;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

#[derive(Deserialize)]
struct ItemDefinitionRow {
    #[serde(rename = "Id")]
    id: u32,
}

fn project_path(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn lab_path(path: &str) -> PathBuf {
    project_path("../../../lab").join(path)
}

#[test]
fn loaded_harvest_nodes_have_valid_references_and_finite_spawns() {
    let content =
        HarvestNodesContent::load_from(project_path("data/content/harvest_nodes.json")).unwrap();
    let item_bytes = fs::read(lab_path(
        "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
    ))
    .unwrap();
    let item_ids: HashSet<u32> = serde_json::from_slice::<Vec<ItemDefinitionRow>>(&item_bytes)
        .unwrap()
        .into_iter()
        .map(|row| row.id)
        .collect();
    let model_rows = fs::read_to_string(lab_path(
        "working/OSFR.Client/OSFR Client/Resources/Models.txt",
    ))
    .unwrap();
    let model_ids: HashSet<u32> = model_rows
        .lines()
        .filter_map(|line| line.split('^').next()?.parse().ok())
        .collect();

    // Mushrooms and rare mushrooms (2 pools, 76 spawns), then T553's farm plot (3 pools, 13).
    assert_eq!(content.node_types.len(), 3);
    assert_eq!(content.pools.len(), 5);
    assert_eq!(content.spawns.len(), 89);

    let type_keys: HashSet<&str> = content
        .node_types
        .iter()
        .map(|row| row.key.as_str())
        .collect();
    assert_eq!(type_keys.len(), content.node_types.len());
    let pool_keys: HashSet<&str> = content.pools.iter().map(|row| row.key.as_str()).collect();
    assert_eq!(pool_keys.len(), content.pools.len());
    let spawn_ids: HashSet<u32> = content.spawns.iter().map(|row| row.id).collect();
    assert_eq!(spawn_ids.len(), content.spawns.len());

    for node_type in &content.node_types {
        assert!(model_ids.contains(&node_type.model_id));
        assert!(
            node_type
                .drop_table
                .iter()
                .all(|drop| item_ids.contains(&drop.item_definition_id))
        );
        assert!(node_type.drop_table.iter().all(|drop| drop.weight > 0));
    }
    for pool in &content.pools {
        assert!(type_keys.contains(pool.node_type.as_str()));
    }

    let pools: HashMap<&str, _> = content
        .pools
        .iter()
        .map(|pool| (pool.key.as_str(), pool))
        .collect();
    for spawn in &content.spawns {
        let pool = pools[spawn.pool_key.as_str()];
        assert_eq!(spawn.zone_definition_id, pool.zone_definition_id);
        assert!(spawn.position.iter().all(|value| value.is_finite()));
        assert!(spawn.heading.is_finite());
    }
}
