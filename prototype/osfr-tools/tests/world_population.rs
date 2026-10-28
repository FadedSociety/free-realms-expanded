//! T549 world population: Sanctuary-main's ambient critters and townsfolk.
//!
//! The rows are plain data in `npc_spawns_extra.json` (no production code changed). This
//! suite proves they stay inside the brief's living families, bind the client locale names,
//! reach the shared AddNpc fan-out, and keep the content-key actors visible through the
//! nearest-64 cap (`world.rs:22`).

#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/paths.rs"]
mod paths;

use osfr_tools::world::NpcVisibility;
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::PathBuf,
};

/// The reconstructed-evidence prefix that marks the rows this suite owns.
const T549_MARKER: &str = "(R) T549:";

/// The living families this feature may draw: the critter set from the brief and the
/// ambient people families (farmers, shoppers, guards, children). A row outside both is
/// another brief's or a named NPC's.
const CRITTER_PREFIXES: &[&str] = &[
    "antelope",
    "forest_buck",
    "forest_doe",
    "rabbit",
    "squirrel",
    "cow",
    "billygoat",
    "nannygoat",
    "duck_f_01",
    "vulture_sunstone",
    "dog_large_",
    "dog_small_",
];
const TOWNSFOLK_PREFIXES: &[&str] = &[
    "human_",
    "fairy_",
    "fairychild_",
    "humanchild_",
    "changeling_",
    "dwarf_miner_m",
    "chugawug_",
    "troll_peasant",
    "sogg_",
];

/// Categories held back by the brief. Helpers and markers, plants and props (T550), coins,
/// ore veins and farm plots, field-enemies creatures, and world-event dressing each belong
/// to another brief and must never carry the T549 marker.
const HELD_BACK_PREFIXES: &[&str] = &[
    "invisible_cube",
    "sg_spawner_",
    "hsg_checkpoint_",
    "overworld_",
    "farming_",
    "sg_farming_node",
    "sg_mailbox",
    "sh_mailbox",
    "mail_",
    "evnt_",
    "hsg_",
    "sg_",
    "sh_",
    "ss_deco",
    "gw_roadhouse",
    "env_icon_",
    "feathereddrake_",
    "bixie_",
    "robgoblin_",
    "wolf",
    "troll_shaman",
    "trex_",
    "tyrannosaurus_",
    "wyvern_",
    "drake_",
    "panthera_",
    "human_m_soccerblue",
    "human_m_soccerred",
    "human_f_cheetahs_teammate",
];
const HELD_BACK_NAMES: &[&str] = &["soccer", "goalie", "partying", "party pixie"];

// The 141 rows this feature imported: 71 critters and 70 townsfolk.
const IMPORTED_ROWS: usize = 141;
const IMPORTED_CRITTERS: usize = 71;
const IMPORTED_TOWNSFOLK: usize = 70;

/// The densest Sanctuary content-actor cluster (Sam Potts at the market square); a fresh
/// viewer here sees 53 AddNpc bodies, including T549 shoppers and royal guards.
const SANCTUARY_SQUARE: [f32; 3] = [-1290.0687, -14.91131, 291.80383];

fn content_dir() -> PathBuf {
    npc_visibility::content_dir()
}

fn extras_rows() -> Vec<Value> {
    json::read(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .expect("npc_spawns_extra.json records")
        .clone()
}

/// The rows whose reconstructed evidence carries the T549 marker.
fn t549_rows(rows: &[Value]) -> Vec<&Value> {
    rows.iter()
        .filter(|row| {
            row["source"].as_array().is_some_and(|sources| {
                sources.iter().any(|source| {
                    source["evidence"]
                        .as_str()
                        .is_some_and(|evidence| evidence.starts_with(T549_MARKER))
                })
            })
        })
        .collect()
}

fn position(row: &Value) -> [f32; 3] {
    let values = row["position"].as_array().expect("position");
    [
        values[0].as_f64().unwrap() as f32,
        values[1].as_f64().unwrap() as f32,
        values[2].as_f64().unwrap() as f32,
    ]
}

fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (*a - b) * (*a - b))
        .sum::<f32>()
        .sqrt()
}

/// The `spawnNpcWithGuid(<id>)` argument in the row's reference evidence.
fn lua_spawn_id(row: &Value) -> Option<u64> {
    let evidence = row["source"].as_array()?.iter().find_map(|source| {
        (source["kind"] == "reference")
            .then(|| source["evidence"].as_str())
            .flatten()
    })?;
    let rest = evidence.split_once("spawnNpcWithGuid(")?.1;
    let digits: String = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn regional_rows() -> Vec<Value> {
    let mut files: Vec<PathBuf> = fs::read_dir(content_dir().join("npcs"))
        .expect("data/content/npcs")
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
        .iter()
        .flat_map(|path| json::read(path)["records"].as_array().unwrap().clone())
        .collect()
}

#[test]
fn t549_rows_are_only_the_critter_and_townsfolk_families() {
    let rows = extras_rows();
    let imported = t549_rows(&rows);

    let extras_guids: HashSet<u64> = rows
        .iter()
        .map(|row| row["spawn_guid"].as_u64().unwrap())
        .collect();
    assert_eq!(
        extras_guids.len(),
        rows.len(),
        "npc_spawns_extra.json has a duplicate spawn guid"
    );
    let mut regional_guids = HashSet::new();
    for row in regional_rows() {
        assert!(
            regional_guids.insert(row["npc_id"].as_u64().unwrap()),
            "a regional actor id repeats"
        );
    }

    let mut critters = 0;
    let mut townsfolk = 0;
    for row in &imported {
        let name = row["name"].as_str().expect("name");
        let guid = row["spawn_guid"].as_u64().expect("spawn_guid");
        assert!(
            !regional_guids.contains(&guid),
            "{name}: spawn guid collides with a regional actor"
        );
        assert_eq!(row["legacy_npc_id"], Value::Null, "{name}");
        assert!(
            row.get("content_key").is_none(),
            "{name}: a T549 row binds no content key"
        );
        let id = lua_spawn_id(row).unwrap_or_else(|| panic!("{name}: no reference spawn id"));
        assert_eq!(
            guid,
            100_000_000_000 + id,
            "{name}: spawn guid is not 100000000000 + the Npcs.json Id"
        );

        let model = row["model_file_name"]
            .as_str()
            .expect("model_file_name")
            .to_ascii_lowercase();
        let lowered = name.to_ascii_lowercase();
        assert!(
            !HELD_BACK_PREFIXES
                .iter()
                .any(|prefix| model.starts_with(prefix)),
            "{name}: held-back model {model}"
        );
        assert!(
            !HELD_BACK_NAMES.iter().any(|word| lowered.contains(word)),
            "{name}: event dressing"
        );
        if CRITTER_PREFIXES
            .iter()
            .any(|prefix| model.starts_with(prefix))
        {
            critters += 1;
        } else if TOWNSFOLK_PREFIXES
            .iter()
            .any(|prefix| model.starts_with(prefix))
        {
            townsfolk += 1;
        } else {
            panic!("{name}: model {model} is neither a critter nor a townsfolk family");
        }
    }
    assert_eq!(imported.len(), IMPORTED_ROWS);
    assert_eq!(
        (critters, townsfolk),
        (IMPORTED_CRITTERS, IMPORTED_TOWNSFOLK)
    );
}

#[test]
fn t549_names_are_client_locale_text() {
    let locale: HashMap<u64, String> =
        json::read(&paths::project_parent().join("lab/reports/locale-id-to-text.json"))
            .as_array()
            .expect("locale-id-to-text.json rows")
            .iter()
            .map(|entry| {
                (
                    entry["id"].as_u64().unwrap(),
                    entry["text"].as_str().unwrap().to_string(),
                )
            })
            .collect();
    let rows = extras_rows();
    let imported = t549_rows(&rows);
    assert!(!imported.is_empty());
    for row in imported {
        let id = row["name_string_id"].as_u64().expect("name_string_id");
        let name = row["name"].as_str().expect("name");
        assert_eq!(
            locale.get(&id).map(String::as_str),
            Some(name),
            "{name}: client locale id {id} does not say that"
        );
    }
}

#[test]
fn t549_rows_do_not_duplicate_a_drawn_actor_within_one_metre() {
    let rows = extras_rows();
    let imported = t549_rows(&rows);
    let mut drawn: Vec<(u64, String, [f32; 3])> = rows
        .iter()
        .filter(|row| row["capture_match"].is_null())
        .filter(|row| {
            !row["model_file_name"]
                .as_str()
                .is_some_and(|model| model.eq_ignore_ascii_case("knocker_m_boss.adr"))
        })
        .map(|row| {
            (
                row["spawn_guid"].as_u64().unwrap(),
                row["model_file_name"]
                    .as_str()
                    .unwrap()
                    .to_ascii_lowercase(),
                position(row),
            )
        })
        .collect();
    for row in regional_rows() {
        if row["placement"] == "legacy_data" {
            drawn.push((
                row["npc_id"].as_u64().unwrap(),
                row["model_file_name"]
                    .as_str()
                    .unwrap()
                    .to_ascii_lowercase(),
                position(&row),
            ));
        }
    }
    for row in &imported {
        let guid = row["spawn_guid"].as_u64().unwrap();
        let name = row["name"].as_str().unwrap();
        let model = row["model_file_name"]
            .as_str()
            .unwrap()
            .to_ascii_lowercase();
        let at = position(row);
        for (other, other_model, other_at) in &drawn {
            if *other == guid || *other_model != model {
                continue;
            }
            assert!(
                distance(at, *other_at) > 1.0,
                "{name}: within 1 m of drawn actor {other} with the same model"
            );
        }
    }
}

#[test]
fn t549_rows_reach_the_shared_add_npc_view() {
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    let rows = extras_rows();
    for row in t549_rows(&rows) {
        let guid = row["spawn_guid"].as_u64().unwrap();
        let at = position(row);
        let seen = npc_visibility::shared_seen_from(&visibility, at);
        let npc = seen
            .iter()
            .find(|npc| npc.actor_id == guid)
            .unwrap_or_else(|| panic!("{guid} is not drawn from its own position"));
        assert_eq!(
            npc.model_id,
            row["model_id"].as_u64().unwrap() as i32,
            "{guid}"
        );
        assert_eq!(
            npc.name_string_id,
            row["name_string_id"].as_u64().unwrap() as i32,
            "{guid}"
        );
        assert_eq!(
            [npc.position[0], npc.position[1], npc.position[2]],
            at,
            "{guid}"
        );
        let scale = row["scale"].as_f64().unwrap_or(1.0) as f32;
        assert!((npc.scale - scale).abs() <= f32::EPSILON, "{guid}: scale");
        assert_eq!(visibility.content_key(guid), None, "{guid}: content key");
    }
}

#[test]
fn content_key_actors_keep_their_near_neighbours_in_the_capped_view() {
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    let mut anchors = Vec::new();
    for row in extras_rows() {
        let key = row["content_key"].as_u64();
        if row["capture_match"].is_null()
            && key.is_some_and(|key| key != 0)
            && !row["model_file_name"]
                .as_str()
                .is_some_and(|model| model.eq_ignore_ascii_case("knocker_m_boss.adr"))
        {
            anchors.push((row["spawn_guid"].as_u64().unwrap(), position(&row)));
        }
    }
    for row in regional_rows() {
        if row["placement"] == "legacy_data" {
            anchors.push((row["npc_id"].as_u64().unwrap(), position(&row)));
        }
    }
    assert!(anchors.len() > 500, "anchor set is only {}", anchors.len());

    let mut near_pairs = 0;
    let mut capped = 0;
    for (guid, at) in &anchors {
        let seen = npc_visibility::shared_seen_from(&visibility, *at);
        assert!(seen.len() <= 64);
        if seen.len() == 64 {
            capped += 1;
        }
        let ids: BTreeSet<u64> = seen.iter().map(|npc| npc.actor_id).collect();
        assert!(
            ids.contains(guid),
            "content actor {guid} is not drawn from its own position"
        );
        for (other, other_at) in &anchors {
            if other == guid {
                continue;
            }
            if distance(*at, *other_at) <= 30.0 {
                assert!(
                    ids.contains(other),
                    "content actor {other} within 30 m of {guid} is outside the nearest-64 view"
                );
                near_pairs += 1;
            }
        }
    }
    assert!(near_pairs > 0, "no near content-actor pair was exercised");
    assert!(capped > 0, "the nearest-64 cap never engaged");
}

#[test]
fn two_sessions_in_the_sanctuary_square_receive_the_same_view() {
    let first = NpcVisibility::load(&content_dir()).unwrap();
    let second = NpcVisibility::load(&content_dir()).unwrap();
    let left = npc_visibility::shared_seen_from(&first, SANCTUARY_SQUARE);
    let right = npc_visibility::shared_seen_from(&second, SANCTUARY_SQUARE);
    assert_eq!(left, right);
    assert!(!left.is_empty() && left.len() <= 64);
    let ids: BTreeSet<u64> = left.iter().map(|npc| npc.actor_id).collect();
    assert!(
        ids.contains(&9000000187),
        "the square's content actor is not drawn"
    );
    assert!(
        ids.contains(&100000033368),
        "the T549 shopper at the square is not drawn"
    );
}
