use osfr_tools::recipes::{self, CraftingJob, RecipeCatalog, UnresolvedRecipeCatalog};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

fn item_definition_ids() -> HashSet<i32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json");
    let raw = fs::read_to_string(path).expect("pinned item definitions should be available in lab");
    recipes::ids_from_item_definitions_json(&raw).unwrap()
}

fn shipped() -> RecipeCatalog {
    RecipeCatalog::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json"),
        &item_definition_ids(),
    )
    .unwrap()
}

fn recipe_object(source: &str) -> Value {
    let marker = source.find("\"recipe\":").expect("test recipe macro");
    let start = source[marker..]
        .find('{')
        .map(|offset| marker + offset)
        .expect("recipe object start");
    let bytes = source.as_bytes();
    let mut depth = 0_u32;
    let mut in_string = false;
    let mut escaped = false;
    let mut end = None;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
        } else if *byte == b'"' {
            in_string = true;
        } else if *byte == b'{' {
            depth += 1;
        } else if *byte == b'}' {
            depth -= 1;
            if depth == 0 {
                end = Some(index + 1);
                break;
            }
        }
    }
    serde_json::from_str(&source[start..end.expect("recipe object end")]).unwrap()
}

#[test]
fn shipped_recipes_load_and_all_item_references_exist() {
    let ids = item_definition_ids();
    let catalog = RecipeCatalog::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json"),
        &ids,
    )
    .unwrap();
    assert_eq!(catalog.records.len(), 31);

    for recipe in &catalog.records {
        assert!(ids.contains(&recipe.output.definition_id));
        assert!(
            recipe
                .ingredients
                .iter()
                .all(|ingredient| ids.contains(&ingredient.definition_id))
        );
        if let Some(id) = recipe.recipe_item_definition_id {
            assert!(ids.contains(&id));
        }
    }

    let unresolved = UnresolvedRecipeCatalog::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes_unresolved.json"),
        &ids,
    )
    .unwrap();
    assert_eq!(unresolved.records.len(), 83);
    assert!(
        unresolved
            .records
            .iter()
            .all(|row| catalog.by_key(&row.key).is_none())
    );
    assert_eq!(catalog.records.len() + unresolved.records.len(), 114);
    assert_eq!(
        unresolved
            .records
            .iter()
            .filter(|row| row.job == CraftingJob::Chef)
            .count(),
        19
    );
    assert_eq!(
        unresolved
            .records
            .iter()
            .filter(|row| row.job == CraftingJob::Blacksmith)
            .count(),
        64
    );
    assert_eq!(
        unresolved
            .records
            .iter()
            .filter(|row| row.job == CraftingJob::Miner)
            .count(),
        0
    );
}

#[test]
fn lookups_cover_job_rank_and_output() {
    let catalog = shipped();
    assert_eq!(catalog.by_job(CraftingJob::Chef).count(), 25);
    assert_eq!(catalog.by_job(CraftingJob::Blacksmith).count(), 1);
    assert_eq!(catalog.by_job(CraftingJob::Miner).count(), 5);
    assert_eq!(catalog.by_output(21785).next().unwrap().name, "Copper Bar");
    assert_eq!(
        catalog
            .by_key("blacksmith.spunky_scrapper_hammer.sample")
            .unwrap()
            .output
            .definition_id,
        30280
    );

    let mut ranked = catalog.by_job(CraftingJob::Chef).next().unwrap().clone();
    ranked.required_job_rank = Some(5);
    let ranked_catalog = RecipeCatalog {
        records: vec![ranked],
    };
    assert_eq!(ranked_catalog.by_rank(CraftingJob::Chef, 5).count(), 1);
    assert_eq!(ranked_catalog.by_rank(CraftingJob::Chef, 4).count(), 0);
}

#[test]
fn current_module_recipe_samples_match_shipped_rows() {
    let catalog = shipped();
    let samples = [
        (include_str!("../src/cooking.rs"), CraftingJob::Chef, 35),
        (
            include_str!("../src/smelting.rs"),
            CraftingJob::Miner,
            21785,
        ),
        (
            include_str!("../src/forging.rs"),
            CraftingJob::Blacksmith,
            30280,
        ),
    ];

    for (source, job, output_id) in samples {
        let selector = recipe_object(source);
        let key = selector["key"].as_str().expect("module recipe key");
        let record = catalog
            .by_key(key)
            .unwrap_or_else(|| panic!("missing current {job:?} recipe key {key}"));
        assert_eq!(record.job, job);
        assert_eq!(record.output.definition_id, output_id);
        if job == CraftingJob::Miner {
            // Client tutorial locale 1772250646 says Copper smelting yields five bars.
            assert_eq!(record.output.count, 5);
        }
    }
}

#[test]
fn every_recipe_minigame_type_exists_in_client_data() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt");
    let raw = fs::read_to_string(path).expect("preserved client MiniGameData.txt");
    let rows: HashMap<u32, u32> = raw
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split('^').collect();
            Some((fields.first()?.parse().ok()?, fields.get(1)?.parse().ok()?))
        })
        .collect();
    let catalog = shipped();
    let types: HashSet<_> = rows.values().copied().collect();

    for recipe in &catalog.records {
        assert!(
            types.contains(&recipe.minigame_type_id),
            "{} uses absent MiniGameData type {}",
            recipe.key,
            recipe.minigame_type_id
        );
        if let Some(game_id) = recipe.minigame_id {
            assert_eq!(
                rows.get(&game_id),
                Some(&recipe.minigame_type_id),
                "{} has a minigame id with the wrong client type",
                recipe.key
            );
        }
    }
}

#[test]
fn recipe_schema_rejects_unknown_fields_and_dangling_items() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json");
    let mut raw = fs::read_to_string(&path).unwrap();
    let with_unknown = raw.replacen(
        "\"schema_version\": 1,",
        "\"schema_version\": 1, \"unknown\": true,",
        1,
    );
    assert!(serde_json::from_str::<recipes::RecipeFile>(&with_unknown).is_err());

    raw = raw.replacen("\"definition_id\": 35", "\"definition_id\": 99999999", 1);
    let temp = std::env::temp_dir().join(format!("recipes-invalid-{}.json", std::process::id()));
    fs::write(&temp, raw).unwrap();
    let result = RecipeCatalog::load(&temp, &item_definition_ids());
    let _ = fs::remove_file(temp);
    assert!(result.is_err());
}
