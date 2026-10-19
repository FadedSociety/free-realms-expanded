use osfr_tools::recipes::{self, CraftingJob, RecipeCatalog, UnresolvedRecipeCatalog};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    fs::File,
    io::BufReader,
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

#[derive(serde::Deserialize)]
struct DefinitionFile {
    definitions: Vec<DefinitionRow>,
}

#[derive(serde::Deserialize)]
struct DefinitionRow {
    id: i32,
}

fn definition_ids(path: &std::path::Path) -> HashSet<i32> {
    let file = File::open(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let document: DefinitionFile = serde_json::from_reader(BufReader::new(file))
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    document.definitions.into_iter().map(|row| row.id).collect()
}

#[derive(serde::Deserialize)]
struct ClientSelfPacket {
    #[serde(rename = "Recipes")]
    recipes: Vec<ClientRecipeRow>,
}

#[derive(serde::Deserialize)]
struct ClientRecipeRow {
    #[serde(rename = "Unknown")]
    id: u32,
    #[serde(rename = "Unknown6")]
    family: u32,
    #[serde(rename = "Unknown7")]
    output_definition_id: i32,
    #[serde(rename = "ComponentData")]
    components: Vec<ClientRecipeComponent>,
}

#[derive(serde::Deserialize)]
struct ClientRecipeComponent {
    #[serde(rename = "Unknown3")]
    count: u32,
    #[serde(rename = "Unknown6")]
    definition_id: i32,
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
    assert_eq!(catalog.records.len(), 32);

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
    assert_eq!(unresolved.records.len(), 82);
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
        18
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
fn every_loaded_recipe_item_exists_in_runtime_item_catalogue() {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/round70-native-20261016T061000Z/content");
    let mut ids = definition_ids(&content.join("inventory.json"));
    ids.extend(definition_ids(&content.join("item-catalogue.json")));
    assert!(
        ids.len() >= 58_000,
        "runtime item catalogue unexpectedly small: {}",
        ids.len()
    );

    let catalog = RecipeCatalog::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json"),
        &ids,
    )
    .expect("every loaded recipe item must have a runtime definition");
    assert_eq!(catalog.records.len(), 32);
}

#[test]
fn client_cooking_and_smelting_rows_match_loaded_recipes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json");
    let file = File::open(&path).expect("preserved client self packet");
    let packet: ClientSelfPacket =
        serde_json::from_reader(BufReader::new(file)).expect("client self packet JSON");
    let catalog = shipped();
    let mut matched = HashSet::new();

    for row in packet
        .recipes
        .iter()
        .filter(|row| matches!(row.family, 9 | 10))
    {
        let recipe = catalog
            .records
            .iter()
            .find(|recipe| recipe.recipe_id == Some(row.id))
            .unwrap_or_else(|| panic!("client recipe row {} is not loaded", row.id));
        assert_eq!(
            recipe.output.definition_id, row.output_definition_id,
            "client recipe row {} output",
            row.id
        );
        let expected: BTreeMap<i32, u32> = row
            .components
            .iter()
            .map(|component| (component.definition_id, component.count))
            .collect();
        let actual: BTreeMap<i32, u32> = recipe
            .ingredients
            .iter()
            .map(|ingredient| (ingredient.definition_id, ingredient.count))
            .collect();
        assert_eq!(actual, expected, "client recipe row {} ingredients", row.id);
        matched.insert(row.id);
    }

    assert_eq!(
        matched,
        HashSet::from([4, 5, 33, 34, 43, 46, 50, 51, 56, 59]),
        "every cooking and smelting row in the client recipe table must be loaded"
    );
}

#[test]
fn unresolved_candidate_ids_do_not_silently_shadow_loaded_rows() {
    let catalog = shipped();
    let unresolved = UnresolvedRecipeCatalog::load(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes_unresolved.json"),
        &item_definition_ids(),
    )
    .unwrap();
    let collisions: Vec<(String, u32)> = unresolved
        .records
        .iter()
        .filter_map(|row| {
            let id = row.candidate_recipe_id?;
            catalog
                .records
                .iter()
                .find(|recipe| recipe.recipe_id == Some(id))
                .map(|_| (row.key.clone(), id))
        })
        .collect();
    assert_eq!(
        collisions,
        vec![("chef.pepper_punkin".to_owned(), 46)],
        "only the documented stale Pepper Punkin candidate may collide; see its missing_fields"
    );
}

#[test]
fn lookups_cover_job_rank_and_output() {
    let catalog = shipped();
    assert_eq!(catalog.by_job(CraftingJob::Chef).count(), 26);
    assert_eq!(catalog.by_job(CraftingJob::Blacksmith).count(), 1);
    assert_eq!(catalog.by_job(CraftingJob::Miner).count(), 5);
    assert_eq!(catalog.by_output(21785).next().unwrap().name, "Copper Bar");
    assert_eq!(
        catalog.by_key("miner.tin_bar").unwrap().output.count,
        5,
        "client locale 451207523 states five tin bars in one session"
    );
    let copper = catalog.by_key("miner.copper_bar.sample").unwrap();
    assert_eq!(copper.recipe_id, Some(4));
    assert_eq!(copper.recipe_item_definition_id, Some(381));
    assert_eq!(copper.required_job_rank, Some(1));
    let seared = catalog.by_key("chef.seared_stainberry").unwrap();
    assert_eq!(seared.recipe_id, Some(51));
    assert_eq!(seared.recipe_item_definition_id, Some(861));
    assert_eq!(seared.output.definition_id, 21825);
    assert_eq!(
        seared
            .ingredients
            .iter()
            .map(|ingredient| (ingredient.definition_id, ingredient.count))
            .collect::<Vec<_>>(),
        vec![(2880, 3), (2884, 3), (2887, 3), (2906, 7)]
    );
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
