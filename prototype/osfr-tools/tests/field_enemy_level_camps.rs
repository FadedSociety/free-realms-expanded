//! T245: level 5-10 free-roaming camps appended to the shipped field population.
//!
//! Content-only slice. The four new camps reuse the existing W102 centres, the
//! existing `field_roamer` behaviour, the existing `enemies.json` definitions and
//! the existing field loader/runtime; no production module changes.
//!
//! The client sees every field actor through the existing 35/2 AddNpc path
//! (reader 0x00921A10, `startup_world::NpcSpawn`), which carries no level field.
//! `level` is server-side input to the W105 HP/damage scaling and
//! `EncounterEngine::field_level` only.

use osfr_tools::encounter::{EncounterEngine, EncounterOutcome};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// The four mid-level camps in file order: group id, the two new member GUIDs,
/// level band and shared respawn timer.
const CAMPS: [(&str, [u64; 2], [u32; 2], u32); 4] = [
    (
        "mid_farnum_wolves",
        [131000102013, 131000102014],
        [5, 6],
        90,
    ),
    (
        "mid_highroad_hooligans",
        [131000102015, 131000102016],
        [7, 8],
        90,
    ),
    (
        "mid_wilds_shamans",
        [131000102017, 131000102018],
        [9, 10],
        120,
    ),
    (
        "mid_queensfields_bixies",
        [131000102019, 131000102020],
        [10, 10],
        120,
    ),
];

/// The 12 pre-change W102 centres, in actor order. A new camp may only repeat
/// one of these terrain-checked discs; moving one back onto new ground must fail
/// here and go through the W102 terrain audit first.
const W102_CENTRES: [(&str, [f32; 3]); 12] = [
    ("W102-F1", [-1899.204, -50.59, 307.902]),
    ("W102-F2", [-1887.867, -51.14, 291.159]),
    ("W102-F3", [-1962.89, -46.92, 284.723]),
    ("W102-H1", [-1779.455, -28.88, 562.833]),
    ("W102-H2", [-1792.921, -29.33, 579.55]),
    ("W102-H3", [-1789.626, -30.24, 616.51]),
    ("W102-Q1", [-664.762, 2.4, 299.608]),
    ("W102-Q2", [-692.141, 2.4, 220.373]),
    ("W102-Q3", [-651.667, 2.4, 231.947]),
    ("W102-T1", [-347.084, 0.61, 1068.348]),
    ("W102-T2", [-351.103, 2.33, 1048.674]),
    ("W102-T3", [-447.084, 3.03, 1128.348]),
];

const STARTER_LEVELS: [u32; 12] = [1, 1, 1, 3, 3, 3, 1, 1, 1, 1, 1, 1];
const STARTER_DEFINITIONS: [u64; 12] = [
    3156, 3156, 3156, 930050, 930050, 930050, 940102, 940102, 940102, 930025, 930025, 930025,
];
const STARTER_GUIDS: [u64; 12] = [
    131000102001,
    131000102002,
    131000102003,
    131000102004,
    131000102005,
    131000102006,
    131000102007,
    131000102008,
    131000102009,
    131000102010,
    131000102011,
    131000102012,
];

/// The four quest-3030007 Hooligans, in file order.
const QUEST_HOOLIGANS: [(u64, [f32; 3]); 4] = [
    (100000004154, [-1971.408, -34.51335, 539.8358]),
    (100000004155, [-1966.171, -35.09792, 544.2426]),
    (5064655448674664449, [-1975.2, -34.3, 548.0]),
    (5064655448674664450, [-1962.0, -35.5, 535.0]),
];
const QUEST_ID: u64 = 3_030_007;

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    /// Minimal production-shaped load: a captured AddNpc template, the world
    /// zone and one dungeon. The shipped content files are loaded from
    /// `data/content`, never copied or rewritten.
    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t245-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id":0,"name":"SyntheticWorld","sky":"","geometry_id":214,"position":[0,0,0],"rotation":[0,0,0],"tutorial":false});
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        let other = json!({"guid":200,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":100,"damage":7,"boss":false});
        fs::write(
            dir.join("encounters.json"),
            json!({
                "schema_version":1,
                "template_file":"npc.bin",
                "template_sha256":format!("{:x}",Sha256::digest(&template)),
                "world":zone,
                "world_enemies":[],
                "dungeons":[{
                    "id":411,
                    "title_id":12,
                    "description_id":13,
                    "objective_id":14,
                    "zone":cave,
                    "guide":null,
                    "enemies":[other],
                    "reward_coins":0,
                    "reward_items":[]
                }]
            })
            .to_string(),
        )
        .unwrap();
        Self(dir)
    }

    fn load(&self) -> EncounterEngine {
        let mut engine = EncounterEngine::load(&self.0, None).unwrap();
        // Production loads presets before the actors that name them.
        engine
            .load_behaviours(&content_dir().join("behaviours.json"))
            .unwrap();
        engine.load_field_enemies(&content_path()).unwrap();
        engine
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn content_path() -> PathBuf {
    content_dir().join("field_enemies.json")
}

fn field() -> Value {
    serde_json::from_slice(&fs::read(content_path()).unwrap()).unwrap()
}

fn actor<'a>(field: &'a Value, guid: u64) -> &'a Value {
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["guid"].as_u64() == Some(guid))
        .unwrap_or_else(|| panic!("field actor {guid} missing"))
}

fn group<'a>(field: &'a Value, id: &str) -> &'a Value {
    field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_str() == Some(id))
        .unwrap_or_else(|| panic!("field spawn group {id} missing"))
}

/// Compare authored decimal coordinates at the runtime's `f32` precision.
fn position(value: &Value) -> [f32; 3] {
    let points = value.as_array().unwrap();
    [
        points[0].as_f64().unwrap() as f32,
        points[1].as_f64().unwrap() as f32,
        points[2].as_f64().unwrap() as f32,
    ]
}

/// GUIDs in the 35/2 AddNpc packets a spawn emitted. The client body reader
/// `0x00921A10` reads the u64 actor id as the first field and carries no level.
fn spawned(out: &EncounterOutcome) -> BTreeSet<u64> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
        .map(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
        .collect()
}

#[test]
fn four_mid_level_camps_load_with_valid_bands_and_ids() {
    // The real loader is the gate: a duplicate GUID/area, an out-of-band level,
    // a missing group member or a malformed row fails here before any assertion.
    let mut engine = Fixture::new().load();
    let spawned = spawned(&engine.on_zone_ready(11, 0).unwrap());
    let field = field();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();

    let mut actor_guids = BTreeSet::new();
    let mut area_ids = BTreeSet::new();
    for row in enemies {
        assert!(
            actor_guids.insert(row["guid"].as_u64().unwrap()),
            "duplicate actor GUID {}",
            row["guid"]
        );
        if let Some(area) = row.get("spawn_area") {
            assert!(
                area_ids.insert(area["id"].as_str().unwrap().to_owned()),
                "duplicate spawn area id {}",
                area["id"]
            );
        }
    }

    let mut grouped: BTreeMap<u64, &str> = BTreeMap::new();
    for row in groups {
        for member in row["members"].as_array().unwrap() {
            let guid = member.as_u64().unwrap();
            assert!(
                grouped.insert(guid, row["id"].as_str().unwrap()).is_none(),
                "member {guid} is in more than one group"
            );
        }
    }

    for (id, members, band, respawn) in CAMPS {
        let row = group(&field, id);
        assert_eq!(row["zone_id"], 0);
        assert_eq!(row["level_band"], json!(band));
        assert_eq!(row["respawn_seconds"], respawn);
        assert_eq!(row["aggro_range"], 8.0);
        assert_eq!(row["leash_radius"], 12.0);
        assert_eq!(row["behaviour"], "field_roamer");
        assert_eq!(row["members"], json!(members));
        assert!(band[0] >= 1 && band[1] <= 20);
        for guid in members {
            assert_eq!(grouped.get(&guid).copied(), Some(id));
            let row = actor(&field, guid);
            let level = row["level"].as_u64().unwrap() as u32;
            assert!((5..=10).contains(&level), "{guid} level {level} is 5-10");
            assert!(
                band[0] <= level && level <= band[1],
                "{guid} level {level} outside its band {band:?}"
            );
            assert_eq!(row["quest_ids"], json!([]), "{guid} must be always-on");
            assert_ne!(
                row["definition_id"].as_u64(),
                Some(930050),
                "{guid} uses the boss definition"
            );
            assert_eq!(
                engine.field_level(guid),
                Some(level),
                "the loader must consume {guid}'s authored level"
            );
            assert!(
                spawned.contains(&guid),
                "{guid} must be visible through the existing 35/2 AddNpc spawn"
            );
            assert_eq!(engine.behaviour(guid).unwrap().leash, 12.0);
        }
    }
}

#[test]
fn every_new_home_is_a_verified_w102_centre() {
    let engine = Fixture::new().load();
    let field = field();
    let mut used = BTreeSet::new();
    for (id, members, ..) in CAMPS {
        assert_eq!(group(&field, id)["leash_radius"], 12.0);
        for guid in members {
            let row = actor(&field, guid);
            let area = row["spawn_area"]["id"].as_str().unwrap();
            let base_area = format!("W102-{}", &area[4..]);
            let (_, centre) = W102_CENTRES
                .iter()
                .find(|(name, _)| *name == base_area)
                .unwrap_or_else(|| panic!("{area} is not a W102 area"));
            assert!(used.insert(base_area), "{area} reuses a W102 centre twice");
            assert_eq!(
                position(&row["position"]),
                *centre,
                "{area} position must stay on its verified W102 centre"
            );
            assert_eq!(position(&row["spawn_area"]["centre"]), *centre);
            assert_eq!(row["spawn_area"]["radius"], 6.0);
            assert_eq!(row["spawn_area"]["count"], 1);
            assert_eq!(
                engine.behaviour(guid).unwrap().leash,
                12.0,
                "{area} keeps the existing home leash"
            );
        }
    }
    assert_eq!(
        used.len(),
        8,
        "the eight new camps sit on eight W102 centres"
    );
}

#[test]
fn the_actor_and_group_counts_stay_inside_the_loader_bounds() {
    let bytes = fs::read(content_path()).unwrap();
    assert!(bytes.len() <= 128 * 1024, "whole field file is 128 KiB max");
    let field: Value = serde_json::from_slice(&bytes).unwrap();
    let enemies = field["enemies"].as_array().unwrap();
    let groups = field["spawn_groups"].as_array().unwrap();
    assert!(enemies.len() <= 64, "64 actors max");
    assert!(groups.len() <= 64, "64 groups max");

    let sources = enemies
        .iter()
        .map(|row| row["source"].as_array().unwrap())
        .chain(groups.iter().map(|row| row["source"].as_array().unwrap()));
    for source_list in sources {
        assert!(!source_list.is_empty(), "every row needs a source");
        for source in source_list {
            let evidence = source["evidence"].as_str().unwrap_or("");
            assert!(
                !evidence.trim().is_empty(),
                "every source needs evidence: {source}"
            );
            if source["kind"] == "reconstructed" {
                assert!(
                    evidence.starts_with("(R)"),
                    "reconstructed source must start with (R): {evidence}"
                );
            }
        }
    }

    for (_, members, ..) in CAMPS {
        for guid in members {
            let kinds: BTreeSet<&str> = actor(&field, guid)["source"]
                .as_array()
                .unwrap()
                .iter()
                .map(|source| source["kind"].as_str().unwrap())
                .collect();
            for kind in ["client", "reference", "legacy-data", "reconstructed"] {
                assert!(kinds.contains(kind), "{guid} lost its {kind} provenance");
            }
        }
    }
}

#[test]
fn the_existing_starter_and_quest_populations_are_unchanged() {
    let field = field();

    // BEFORE snapshot (the rows this slice appends after): the 12 always-on
    // W102 actors keep their GUID, area, centre, level and definition, and stay
    // outside quest gating.
    for (index, guid) in STARTER_GUIDS.into_iter().enumerate() {
        let row = actor(&field, guid);
        let (area, centre) = &W102_CENTRES[index];
        assert_eq!(row["quest_ids"], json!([]));
        assert_eq!(row["level"], STARTER_LEVELS[index]);
        assert_eq!(row["definition_id"], STARTER_DEFINITIONS[index]);
        assert_eq!(row["spawn_area"]["id"], *area);
        assert_eq!(position(&row["position"]), *centre);
        assert_eq!(position(&row["spawn_area"]["centre"]), *centre);
    }

    // The four quest-gated Hooligans keep their identity, placement and gate;
    // the new always-on camps must not join quest 3030007.
    let mut grouped = BTreeSet::new();
    for row in field["spawn_groups"].as_array().unwrap() {
        for member in row["members"].as_array().unwrap() {
            grouped.insert(member.as_u64().unwrap());
        }
    }
    for (guid, centre) in QUEST_HOOLIGANS {
        let row = actor(&field, guid);
        assert_eq!(row["definition_id"], 930050);
        assert_eq!(row["quest_ids"], json!([QUEST_ID]));
        assert_eq!(position(&row["position"]), centre);
        assert_eq!(row["boss"], false);
        assert!(row.get("spawn_area").is_none());
        assert!(!grouped.contains(&guid), "quest spawn {guid} joined a camp");
    }
}
