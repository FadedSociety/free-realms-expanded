use osfr_tools::recipe_book::{
    Station, WorldStationRule, load_world_station_rules, merge_stations, world_stations,
};
use osfr_tools::recipes::CraftingJob;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

#[test]
fn shipped_models_register_cooking_smelting_and_forging_stations() {
    let rules = load_world_station_rules(&content()).unwrap();
    let in_world = [
        (0, 419, [-1244.241, -14.34194, 405.1309]),
        (0, 1733, [-2631.933, 0.03999766, -216.5819]),
        (0, 273, [354.3665, 48.24031, 316.3203]),
        (7, 4010, [579.9434, 88.88337, 1974.708]),
    ];
    let stations = world_stations(&rules, in_world);
    assert_eq!(stations.len(), 4);
    assert_eq!(
        stations
            .iter()
            .map(|station| station.job)
            .collect::<Vec<_>>(),
        [
            CraftingJob::Chef,
            CraftingJob::Miner,
            CraftingJob::Blacksmith,
            CraftingJob::Blacksmith,
        ]
    );
    assert_eq!(stations[0].position, in_world[0].2);
    assert_eq!(stations[1].position, in_world[1].2);
    assert_eq!(stations[3].zone_id, 7);
    assert!(stations.iter().all(|station| station.radius == 12.0));
}

#[test]
fn unknown_non_finite_and_duplicate_world_models_are_ignored_or_deduplicated() {
    let rules = load_world_station_rules(&content()).unwrap();
    let stations = world_stations(
        &rules,
        [
            (0, 999, [1.0, 2.0, 3.0]),
            (0, 419, [f32::NAN, 0.0, 0.0]),
            (0, 419, [10.0, 20.0, 30.0]),
            (0, 419, [10.0, 20.0, 30.0]),
        ],
    );
    assert_eq!(stations.len(), 1);
    assert_eq!(stations[0].job, CraftingJob::Chef);
    assert_eq!(stations[0].position, [10.0, 20.0, 30.0]);

    let mut merged = stations;
    let opening = Station {
        job: CraftingJob::Chef,
        zone_id: 0,
        position: [10.0, 20.0, 30.0],
        radius: 12.0,
    };
    let other_job = Station {
        job: CraftingJob::Blacksmith,
        ..opening
    };
    merge_stations(&mut merged, [opening, other_job]);
    assert_eq!(merged.len(), 2);
}

#[test]
fn malformed_station_files_are_refused() {
    let directory = std::env::temp_dir().join(format!(
        "osfr-crafting-stations-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    assert!(load_world_station_rules(&directory).unwrap().is_empty());
    let duplicate = serde_json::json!({
        "schema_version": 1,
        "provenance": "test",
        "stations": [
            {"model_id": 1, "job": "chef", "interact_range": 12.0, "source": "test"},
            {"model_id": 1, "job": "miner", "interact_range": 12.0, "source": "test"}
        ]
    });
    fs::write(
        directory.join("crafting_stations.json"),
        serde_json::to_vec(&duplicate).unwrap(),
    )
    .unwrap();
    assert!(load_world_station_rules(&directory).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rules_preserve_the_world_zone() {
    let rules = [WorldStationRule {
        model_id: 1,
        job: CraftingJob::Miner,
        interact_range: 12.0,
        source: "test".into(),
    }];
    assert_eq!(
        world_stations(&rules, [(3, 1, [1.0, 2.0, 3.0])])[0].zone_id,
        3
    );
}
