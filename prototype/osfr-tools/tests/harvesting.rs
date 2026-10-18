// Focused proof for the Harvesting (MatchThree "Farming") module.
use harvesting::{
    GoalProgress, HarvestContent, HarvestError, LevelRow, PRODUCE, ProgressionRow, REGIONS,
    SpawnRow, TimerRow, Type8Output, board_spec, chain_score, chain_time_bonus_seconds,
    collectable_score, content_json, diamond_shape, evaluate, parse_caret, parse_grid_templates,
    produce, region, validate_content,
};
use osfr_tools::harvesting;
use serde_json::Value;

/// The exact `MatchThreeSpawnData.txt` column order from the original
/// GameServer.as SD_* constants (14 caret-separated columns).
const SPAWN_GOLDEN: &str = "7^0^1^25^40^12^15^3^706^0^0^5^0^0";

#[test]
fn original_spawn_row_roundtrips_with_original_column_order() {
    let row = SpawnRow::parse(SPAWN_GOLDEN).expect("golden spawn row parses");
    assert_eq!(row.id, 7);
    assert_eq!(row.category, 0);
    assert_eq!(row.spawn_kind, 1);
    assert_eq!(row.spawn_chance, 25);
    assert_eq!(row.create_maximum, 40);
    assert_eq!(row.collect_maximum, 12);
    assert_eq!(row.quota, 15);
    assert_eq!(row.reward_set, 3);
    assert_eq!(row.icon_id, 706);
    assert_eq!(row.duration, 0);
    assert_eq!(row.power, 0);
    assert_eq!(row.per_reward, 5);
    assert_eq!(row.collection_type, 0);
    assert_eq!(row.timer_id, 0);
    assert_eq!(row.to_row(), SPAWN_GOLDEN);
}

#[test]
fn original_level_row_roundtrips_with_original_column_order() {
    // LD_ID..LD_TYPEDATA, 18 columns (GameServer.as).
    let golden = "58^58^1^180^910003^0^0^0^10^0^80^10^0^5^5^0^2^0";
    let row = LevelRow::parse(golden).expect("golden level row parses");
    assert_eq!(row.id, 58);
    assert_eq!(row.game, 58);
    assert_eq!(row.level_index, 1);
    assert_eq!(row.duration, 180);
    assert_eq!(row.objective_id, 910003);
    assert_eq!(row.collectable_spawn, 10);
    assert_eq!(row.bonus_spawn, 0);
    assert_eq!(row.selectable_spawn, 80);
    assert_eq!(row.powerup_spawn, 10);
    assert_eq!(row.hazard_spawn, 0);
    assert_eq!(row.blocker_spawn, 5);
    assert_eq!(row.wildcard_spawn, 5);
    assert_eq!(row.reward, 0);
    assert_eq!(row.kind, 2);
    assert_eq!(row.type_data, 0);
    assert_eq!(row.to_row(), golden);
}

#[test]
fn original_progression_and_timer_rows_roundtrip() {
    let golden = "1^111^5^20^4^60";
    let row = ProgressionRow::parse(golden).unwrap();
    assert_eq!((row.id, row.level, row.property), (1, 111, 5));
    assert_eq!((row.init, row.delta, row.max), (20, 4, 60));
    assert_eq!(row.to_row(), golden);

    // TimerData.SetChain divides the raw columns by 10 (TimerData.as).
    let timer = TimerRow::parse("3^2^30^2^0^1").unwrap();
    assert_eq!(timer.id, 3);
    assert_eq!(timer.timer_kind, 2);
    assert!((timer.chain_m - 3.0).abs() < f64::EPSILON);
    assert!((timer.chain_f - 0.2).abs() < f64::EPSILON);
    assert_eq!(timer.per_tile, 1);
    assert_eq!(timer.to_row(), "3^2^30^2^0^1");
}

#[test]
fn original_grid_templates_parse_rows_of_codes() {
    let text = "grid^0\n1^1^1\n1^4^1\n0^1^0\n# comment\ngrid^1\n2^2\n";
    let templates = parse_grid_templates(text).expect("two templates");
    assert_eq!(templates.len(), 2);
    assert_eq!(templates[0].id, 0);
    assert_eq!(
        templates[0].rows,
        vec![vec![1, 1, 1], vec![1, 4, 1], vec![0, 1, 0]]
    );
    assert_eq!(templates[1].rows, vec![vec![2, 2]]);
}

#[test]
fn schema_parsers_reject_wrong_column_counts() {
    assert!(matches!(
        SpawnRow::parse("1^2^3"),
        Err(HarvestError::Schema(_))
    ));
    assert!(matches!(
        SpawnRow::parse("1^2^3^4^5^6^7^8^9^10^11^12^13^14^15"),
        Err(HarvestError::Schema(_))
    ));
    assert!(matches!(
        LevelRow::parse("1^2"),
        Err(HarvestError::Schema(_))
    ));
    assert!(matches!(
        TimerRow::parse("1^2^3"),
        Err(HarvestError::Schema(_))
    ));
    assert!(matches!(parse_caret("1^x^3"), Err(HarvestError::Schema(_))));
}

#[test]
fn recovered_produce_sets_match_original_locale_descriptions() {
    // Locale 94149..94154 are the original in-client region descriptions.
    let expected: [(i32, [&str; 4]); 6] = [
        (109, ["gummyfruit", "rainleaf", "bumbleberry", "honeytwig"]),
        (111, ["stickfruit", "stainberry", "tumtumnip", "spiralmint"]),
        (113, ["rainleaf", "chugaroot", "choychoy", "gummyfruit"]),
        (115, ["snowberry", "stainberry", "starnut", "spiralmint"]),
        (117, ["punkin", "sunleaf", "rainleaf", "gummyfruit"]),
        (119, ["spiralmint", "briarmelon", "stainberry", "moonbean"]),
    ];
    for (game_id, set) in expected {
        let r = region(game_id).unwrap_or_else(|| panic!("region {game_id} exists"));
        assert_eq!(r.produce, set, "region {game_id} produce set");
    }
    // The screenshot on the pinned wiki page shows the Lakesfield bins.
    let lakesfield = region(117).unwrap();
    for slug in ["punkin", "rainleaf", "sunleaf", "gummyfruit"] {
        assert!(lakesfield.produce.contains(&slug));
    }
}

#[test]
fn recovered_item_ids_match_pinned_client_definitions() {
    // FreeRealms-Legacy ClientItemDefinitions.json: NameId -> Id for 23285..23300.
    let expected: [(&str, i32, i32, i32); 16] = [
        ("gummyfruit", 23285, 2875, 706),
        ("snowberry", 23286, 2876, 711),
        ("spiralmint", 23287, 2877, 712),
        ("moonbean", 23288, 2878, 708),
        ("choychoy", 23289, 2879, 704),
        ("sunleaf", 23290, 2880, 716),
        ("honeytwig", 23291, 2881, 707),
        ("punkin", 23292, 2882, 709),
        ("briarmelon", 23293, 2883, 703),
        ("rainleaf", 23294, 2884, 710),
        ("stickfruit", 23295, 2885, 715),
        ("starnut", 23296, 2886, 714),
        ("stainberry", 23297, 2887, 713),
        ("chugaroot", 23298, 2888, 705),
        ("bumbleberry", 23299, 2889, 667),
        ("tumtumnip", 23300, 2890, 717),
    ];
    assert_eq!(PRODUCE.len(), expected.len());
    for (slug, name_locale_id, item_definition_id, icon_id) in expected {
        let p = produce(slug).unwrap_or_else(|| panic!("produce {slug} exists"));
        assert_eq!(p.name_locale_id, name_locale_id, "{slug} locale id");
        assert_eq!(p.item_definition_id, item_definition_id, "{slug} item id");
        assert_eq!(p.icon_id, icon_id, "{slug} icon");
    }
    assert_eq!(REGIONS.len(), 6);
}

#[test]
fn score_formulas_match_original_gamedata_as() {
    // nCHAIN_SCORE=400, nCHAIN_BONUS_TILE_SCORE=200, nMINIMUM_CHAIN_LENGTH=3.
    assert_eq!(chain_score(3), 400);
    assert_eq!(chain_score(4), 600);
    assert_eq!(chain_score(12), 2200);
    // nMINIMUM_CHAIN_TIME_BONUS=1, 2 points per chain tile.
    assert_eq!(chain_time_bonus_seconds(3), 6);
    assert_eq!(chain_time_bonus_seconds(0), 0);
    // nCOLLECTABLE_SCORE=500, doubled once the quota is reached.
    assert_eq!(collectable_score(1, 15), 500);
    assert_eq!(collectable_score(15, 15), 1000);
    assert_eq!(collectable_score(16, 15), 1000);
    assert_eq!(collectable_score(1, 0), 500);
}

#[test]
fn result_requires_every_quota_goal_and_counts_bundles() {
    let result = evaluate(&[(14, 15, 5), (10, 10, 5)]);
    assert!(!result.success, "14/15 tiles is not complete");
    assert_eq!(result.total_bundles, 14 / 5 + 10 / 5);

    let result = evaluate(&[(15, 15, 5), (10, 10, 5)]);
    assert!(result.success);
    assert_eq!(result.total_bundles, 3 + 2);

    // CheckCollectableQuota(successOnly=true) returns true with no quota goals.
    assert!(evaluate(&[(0, 0, 5)]).success);
}

#[test]
fn goal_progress_reports_clusters_like_the_original_bins() {
    let goal = GoalProgress {
        index: 0,
        value: 601,
        collected_tiles: 7,
        quota: 15,
        per_reward: 5,
    };
    assert_eq!(goal.clusters(), 1);
    assert!(!goal.complete());
    let done = GoalProgress {
        collected_tiles: 15,
        ..goal
    };
    assert!(done.complete());
    assert_eq!(done.clusters(), 3);
}

#[test]
fn observed_diamond_shape_and_engine_rectangle_stay_separate() {
    // The screenshot-observed staggered artifact stays recorded...
    let shape = diamond_shape(10, 10);
    assert_eq!(shape.len(), 100);
    let widths: Vec<usize> = (0..10)
        .map(|row| {
            shape[row * 10..row * 10 + 10]
                .iter()
                .filter(|c| **c != 0)
                .count()
        })
        .collect();
    assert_eq!(widths, vec![4, 6, 8, 10, 10, 10, 10, 8, 6, 4]);
    assert_eq!(shape.iter().filter(|c| **c != 0).count(), 76);
    // The bottom row of the playable rectangle must sit above the drop zone.
    assert!(shape.iter().all(|c| *c == 0 || *c == 1));

    // ...while the emitted geometry is the engine-compatible padded rectangle: the same 10x9
    // playable area, with row 0 and column 0 kept empty because the original fall loops never
    // visit index 0.
    let rectangle = harvesting::engine_rectangle_shape(10, 9);
    assert_eq!(rectangle.len(), 100);
    assert!(rectangle[..90].iter().all(|c| *c == 1));
    assert!(rectangle[90..].iter().all(|c| *c == 4));
    let emitted = harvesting::padded_engine_shape(11, 11, 1, 9);
    assert_eq!(emitted.len(), 121);
    assert_eq!(emitted.iter().filter(|c| **c == 1).count(), 90);
    assert_eq!(emitted.iter().filter(|c| **c == 4).count(), 10);
    assert!(emitted[0..11].iter().all(|c| *c == 0));
    assert!((0..11).all(|row| emitted[row * 11] == 0));
    let spec = board_spec(109).unwrap();
    assert_eq!(spec.shape, emitted);
    assert!(spec.rule_notice.contains("Fidelity gap"));
}

#[test]
fn recovered_constants_match_the_original_movie_and_inventory() {
    use harvesting::{
        BONUS_SCORE, CHAIN_BONUS_TILE_SCORE, CHAIN_SCORE, COLLECTABLE_SCORE, CONTENT_FILE,
        FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE, FIRST_INSTANCE_ID, GENERIC_HARVESTING_GAME_ID,
        HARVESTING_ICON_ID, MIN_CHAIN_TILES, POWERUP_SCORE, RECONSTRUCTED_PLAYABLE_ROWS,
        RECOVERED_REGION_GAME_IDS, REMAINING_TIME_SCORE, SCHEMA_VERSION, SHAPE_DROPZONE,
        TILE_SIZE_PIXELS, TYPE8_GAME_FAMILY, UNNAMED_VARIANT_GAME_IDS,
    };
    // GameData.as values.
    assert_eq!(MIN_CHAIN_TILES, 3);
    assert_eq!(CHAIN_SCORE, 400);
    assert_eq!(CHAIN_BONUS_TILE_SCORE, 200);
    assert_eq!(COLLECTABLE_SCORE, 500);
    assert_eq!(BONUS_SCORE, 2000);
    assert_eq!(POWERUP_SCORE, 250);
    assert_eq!(REMAINING_TIME_SCORE, 25);
    assert_eq!(TILE_SIZE_PIXELS, 40);
    // Shared type-8 contract and MiniGameData inventory facts.
    assert_eq!(TYPE8_GAME_FAMILY, 39);
    assert_eq!(CONTENT_FILE, "harvesting.json");
    assert_eq!(SCHEMA_VERSION, 1);
    assert_eq!(FIRST_INSTANCE_ID, 58_000_001);
    // Recovered farming launch assets: Assets_000.pack 50 and AssetsW_009.pack
    // 1448 in the pinned manifests.
    assert_eq!(FARMING_PRELOAD_FILE, "FarmingAssets.txt");
    assert_eq!(FARMING_LAUNCH_MOVIE, "game_farming.gfx");
    assert_eq!(RECONSTRUCTED_PLAYABLE_ROWS, 9);
    assert_eq!(SHAPE_DROPZONE, 4);
    assert_eq!(GENERIC_HARVESTING_GAME_ID, 58);
    assert_eq!(HARVESTING_ICON_ID, 908);
    assert_eq!(RECOVERED_REGION_GAME_IDS, [109, 111, 113, 115, 117, 119]);
    assert_eq!(
        UNNAMED_VARIANT_GAME_IDS,
        [110, 112, 114, 116, 118, 120, 217]
    );
    for game_id in RECOVERED_REGION_GAME_IDS {
        assert_eq!(region(game_id).unwrap().icon_id, 908);
    }
}

#[test]
fn content_json_matches_the_frozen_type8_contract() {
    let json = content_json(111).expect("farm content");
    let top: Value = serde_json::from_str(&json).unwrap();
    for key in [
        "schema_version",
        "game_id",
        "profile_id",
        "name_id",
        "description_id",
        "icon_id",
        "spec",
    ] {
        assert!(top.get(key).is_some(), "wrapper field {key}");
    }
    assert_eq!(top["game_id"], 111);
    assert_eq!(top["name_id"], 100834);
    assert_eq!(top["description_id"], 94150);
    assert_eq!(top["icon_id"], 908);
    assert_eq!(top["preload_file"], "FarmingAssets.txt");
    assert_eq!(top["launch_movie"], "game_farming.gfx");

    let spec = &top["spec"];
    for key in [
        "width",
        "height",
        "shape",
        "selectable",
        "collectable",
        "tiles",
        "goals",
        "outputs",
        "duration_seconds",
        "seed",
        "wildcard_percent",
        "collectable_percent",
        "blocker_percent",
        "powerup_percent",
        "meter_id",
        "reconstructed_rules",
        "rule_notice",
    ] {
        assert!(spec.get(key).is_some(), "spec field {key}");
    }
    assert_eq!(spec["reconstructed_rules"], true);
    assert!(!spec["rule_notice"].as_str().unwrap().is_empty());
    let shape = spec["shape"].as_array().unwrap();
    let width = spec["width"].as_i64().unwrap();
    let height = spec["height"].as_i64().unwrap();
    assert_eq!(shape.len() as i64, width * height);
    assert!(spec["collectable"].as_array().unwrap().is_empty());
    assert_eq!(spec["selectable"].as_array().unwrap().len(), 4);
    assert_eq!(spec["goals"].as_array().unwrap().len(), 4);
    let outputs = spec["outputs"].as_array().unwrap();
    assert_eq!(outputs.len(), 4);
    let farm_ids: Vec<i64> = outputs
        .iter()
        .map(|o| o["definition_id"].as_i64().unwrap())
        .collect();
    assert_eq!(farm_ids, vec![2885, 2887, 2890, 2877]);
    let emitted_shape = spec["shape"].as_array().unwrap();
    assert!(
        emitted_shape[emitted_shape.len() - 10..]
            .iter()
            .all(|c| c.as_i64() == Some(4)),
        "emitted board must end in a dropzone row"
    );
}

#[test]
fn content_guards_reject_unknown_or_mutated_configuration() {
    let mut content = harvesting::content(117).unwrap();
    validate_content(&content).expect("recovered Lakesfield content is valid");

    content.game_id = 9999;
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    content.spec.shape.truncate(10);
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    content.spec.outputs[0].definition_id = 1;
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    content.spec.goals.clear();
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    content.launch_movie = "game_mining.gfx".into();
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    content.preload_file = "../MiningAssets.txt".into();
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));

    let mut content = harvesting::content(117).unwrap();
    // Row 0 is the inert border the fall loops require; a non-zero border breaks the
    // engine's playable rectangle and must be refused.
    content.spec.shape[0] = 1;
    assert!(matches!(
        validate_content(&content),
        Err(HarvestError::Content(_))
    ));
}

#[test]
fn generic_and_unnamed_entries_refuse_an_unsupported_launch() {
    // 58 has only the shared description; 110/112/114/116/118/120/217 have no
    // produce evidence. They must refuse to build or write a board instead of
    // emitting config the shared engine rejects.
    assert!(region(58).is_none());
    for game_id in [110, 112, 114, 116, 118, 120, 217] {
        assert!(region(game_id).is_none(), "{game_id} is an unnamed variant");
    }
    for game_id in [58, 110, 112, 114, 116, 118, 120, 217] {
        let error = board_spec(game_id).expect_err("gap profiles must not propose a board");
        assert!(
            error.to_string().contains("unsupported"),
            "{game_id} refusal must be explicit, got {error}"
        );
        assert!(matches!(
            harvesting::content(game_id),
            Err(HarvestError::Content(_))
        ));
        assert!(harvesting::content_json(game_id).is_err());
    }
    assert!(matches!(board_spec(9999), Err(HarvestError::Content(_))));
}

#[test]
fn emitted_spec_is_consumable_by_a_type8_engine_loader() {
    // Mirror of mining_restore's frozen `Content` + Type8BoardSpec serde shape
    // (contract v1). Keeps the data contract honest without a compile link.
    #[derive(serde::Deserialize)]
    struct ContentMirror {
        schema_version: u32,
        game_id: i32,
        profile_id: i32,
        name_id: i32,
        description_id: i32,
        icon_id: i32,
        preload_file: String,
        launch_movie: String,
        spec: SpecMirror,
    }
    #[derive(serde::Deserialize)]
    struct SpecMirror {
        width: i32,
        height: i32,
        shape: Vec<i32>,
        selectable: Vec<u16>,
        collectable: Vec<u16>,
        tiles: Vec<KindMirror>,
        goals: Vec<GoalMirror>,
        outputs: Vec<OutputMirror>,
        duration_seconds: u32,
        seed: u32,
        wildcard_percent: u32,
        collectable_percent: u32,
        blocker_percent: u32,
        powerup_percent: u32,
        meter_id: i32,
        reconstructed_rules: bool,
        rule_notice: String,
    }
    #[derive(serde::Deserialize)]
    struct KindMirror {
        value: u16,
        category: u16,
        icon_id: i32,
    }
    #[derive(serde::Deserialize)]
    struct GoalMirror {
        index: i32,
        value: u16,
        quota: u32,
        per_reward: u32,
        icon_id: i32,
    }
    #[derive(serde::Deserialize)]
    struct OutputMirror {
        value: u16,
        definition_id: i32,
        cluster_quantity: u32,
    }

    let json = content_json(109).unwrap();
    let c: ContentMirror = serde_json::from_str(&json).expect("contract mirror parses");
    assert_eq!(c.schema_version, 1);
    assert_eq!(c.game_id, 109);
    assert_eq!(
        (c.name_id, c.description_id, c.icon_id),
        (100833, 94149, 908)
    );
    assert_eq!(
        (c.preload_file.as_str(), c.launch_movie.as_str()),
        ("FarmingAssets.txt", "game_farming.gfx")
    );
    assert_eq!(c.profile_id, harvesting::CHEF_PROFILE_ID);
    assert_eq!(c.spec.goals.len(), 4);
    // Region 109 = gummyfruit, rainleaf, bumbleberry, honeytwig -> the client's
    // farmingValue401/410/415/407 produce values.
    assert_eq!(c.spec.goals[0].value, 401);
    assert_eq!(c.spec.goals[3].value, 407);
    assert!(c.spec.goals.iter().all(|g| g.quota > 0 && g.per_reward > 0));
    assert_eq!(
        c.spec
            .goals
            .iter()
            .map(|g| (g.index, g.icon_id))
            .collect::<Vec<_>>(),
        vec![(0, 706), (1, 710), (2, 667), (3, 707)]
    );
    // Recovered registration order: one selectable kind per produce, category 400,
    // with `icon_id` = the movie's `SelectableTile<type>` frame index.
    assert_eq!(c.spec.tiles.len(), 4);
    assert_eq!(
        c.spec
            .tiles
            .iter()
            .map(|t| (t.value, t.category, t.icon_id))
            .collect::<Vec<_>>(),
        vec![(401, 400, 1), (410, 400, 10), (415, 400, 15), (407, 400, 7),]
    );
    // The tile list is exactly the selectable spawn set; the unnamed 600 block
    // must never come back as a farming basket.
    assert_eq!(
        c.spec.tiles.iter().map(|t| t.value).collect::<Vec<_>>(),
        c.spec.selectable
    );
    assert!(c.spec.collectable.is_empty());
    assert_eq!(c.spec.goals[0].icon_id, 706, "Gummyfruit bundle icon");
    assert_eq!(c.spec.shape.len() as i32, c.spec.width * c.spec.height);
    assert_eq!(c.spec.selectable, vec![401, 410, 415, 407]);
    assert_eq!(c.spec.outputs[0].value, 401);
    assert_eq!(c.spec.outputs[0].cluster_quantity, 1);
    assert_eq!(
        c.spec
            .outputs
            .iter()
            .map(|o| o.definition_id)
            .collect::<Vec<_>>(),
        vec![2875, 2884, 2889, 2881]
    );
    assert_eq!(
        c.spec.duration_seconds,
        harvesting::RECONSTRUCTED_DURATION_SECONDS
    );
    assert_eq!(c.spec.seed, harvesting::RECONSTRUCTED_SEED);
    assert_eq!(c.spec.wildcard_percent, 5);
    assert_eq!(c.spec.collectable_percent, 0);
    assert_eq!(c.spec.blocker_percent, 0);
    assert_eq!(c.spec.powerup_percent, 0);
    assert_eq!(c.spec.meter_id, 0);
    assert!(c.spec.reconstructed_rules, "reconstruction label must ship");
    assert!(c.spec.rule_notice.contains("Reconstructed"));

    let typed: HarvestContent = serde_json::from_str(&json).unwrap();
    assert_eq!(typed.game_id, 109);
    let typed_outputs: Vec<Type8Output> = typed.spec.outputs.clone();
    assert_eq!(typed_outputs[0].definition_id, 2875);
}

#[test]
fn write_content_lands_in_the_shared_content_directory() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "osfr-harvesting-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = harvesting::write_content(&dir, 119).expect("content file written");
    assert_eq!(path.file_name().unwrap(), harvesting::CONTENT_FILE);
    let text = std::fs::read_to_string(&path).unwrap();
    let parsed: HarvestContent = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed.game_id, 119);
    assert_eq!(parsed.spec.goals.len(), 4);
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir(&dir);
}
