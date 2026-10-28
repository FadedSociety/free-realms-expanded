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

use osfr_tools::{harvest_nodes::GatheringNodes, world::NpcVisibility};
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

/// The reconstructed-evidence prefix that marks the rows this suite owns for T550.
const T550_MARKER: &str = "(R) T550:";

/// T550 imported every harvest plant and static prop among Sanctuary-main's 895 new spawns
/// that nothing else draws. Six source props were already drawn (four clickable quest props
/// from T739 and two card tables from T467) and are skipped, leaving 115 props below.
const T550_PLANTS: usize = 194;
const T550_PROPS: usize = 115;
const T550_ROWS: usize = T550_PLANTS + T550_PROPS;

/// The 20 house mailboxes in those spawns: 19 imported here and the Snowhill one T739 draws.
const T550_HOUSE_MAILBOXES: usize = 20;

/// Every harvest-plant model in the 895 new spawns with its row count. The models are the
/// client's `overworld_*_harvestable_01` plant bodies; a model outside this list would be a
/// plant family Sanctuary does not place here.
const T550_PLANT_SPECIES: &[(&str, usize)] = &[
    ("overworld_bush_bumbleberry_harvestable_01.adr", 19),
    ("overworld_bush_snowberry_harvestable_01.adr", 27),
    ("overworld_bush_stainberry_harvestable_01.adr", 15),
    ("overworld_bush_stickfruit_harvestable_01.adr", 1),
    ("overworld_root_chugaroot_harvestable_01.adr", 3),
    ("overworld_root_wildroot_harvestable_01.adr", 1),
    ("overworld_spiralmint_bush_harvestable_01.adr", 21),
    ("overworld_tree_honeytwig_harvestable_01.adr", 18),
    ("overworld_tree_rainleaf_harvestable_01.adr", 22),
    ("overworld_tree_starnut_harvestable_01.adr", 15),
    ("overworld_tree_sunleaf_harvestable_01.adr", 10),
    ("overworld_vinehigh_moonbean_harvestable_01.adr", 1),
    ("overworld_vinelow_gummyfruit_harvestable_01.adr", 30),
    ("overworld_vinelow_punkin_harvestable_01.adr", 11),
];

/// The static prop models T550 draws. Anything outside this list and the plant list would be
/// a creature, person, helper, marker, coin or gather node, which another feature owns.
const T550_PROP_MODELS: &[(&str, usize)] = &[
    ("ball_m_bowlingball.adr", 1),
    ("bw_flag_01_green.adr", 1),
    ("evnt_halloween_candybag_01.adr", 1),
    ("evnt_winter_holiday_party_banner_01.adr", 3),
    ("evnt_winter_holiday_snowballs_01.adr", 3),
    ("farming_dirt_mound_01.adr", 1),
    ("farming_farmers_market_lv1_01.adr", 1),
    ("gw_roadhouse_01_gas.adr", 2),
    ("hsg_checkpoint_track_01.adr", 1),
    ("hsg_checkpoint_track_02.adr", 7),
    ("mail_01.adr", 2),
    ("sg_anvil_stump_01.adr", 1),
    ("sg_chugawug_sacks_01.adr", 1),
    ("sg_chugawug_sacks_02.adr", 1),
    ("sg_cooking_table_nodes_01.adr", 2),
    ("sg_fairy_flora_03_flower03.adr", 20),
    ("sg_foodtray_01.adr", 1),
    ("sg_gametable_01.adr", 4),
    ("sg_mail_package_01.adr", 1),
    ("sg_mailbox_01.adr", 19),
    ("sg_pies.adr", 1),
    ("sg_practicedummy_nodes_01.adr", 1),
    ("sg_rock_01.adr", 3),
    ("sg_sign01.adr", 1),
    ("sg_spot_the_difference_01.adr", 1),
    ("sg_troll_hut_02_snow.adr", 1),
    ("sh_barrel_01.adr", 8),
    ("sh_bucket_01.adr", 2),
    ("sh_cave_entrance_bat_cave_01.adr", 1),
    ("sh_crate_01.adr", 2),
    ("sh_icecube_01.adr", 1),
    ("sh_mailbox_blue_group_01.adr", 1),
    ("sh_sign01.adr", 1),
    ("sh_table_battlechair01.adr", 8),
    ("sh_table_wartable01.adr", 8),
    ("snowman.adr", 1),
    ("ss_deco_flower_04.adr", 1),
];

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

/// The rows whose reconstructed evidence carries `marker`.
fn marked_rows<'a>(rows: &'a [Value], marker: &str) -> Vec<&'a Value> {
    rows.iter()
        .filter(|row| {
            row["source"].as_array().is_some_and(|sources| {
                sources.iter().any(|source| {
                    source["evidence"]
                        .as_str()
                        .is_some_and(|evidence| evidence.starts_with(marker))
                })
            })
        })
        .collect()
}

/// The rows whose reconstructed evidence carries the T549 marker.
fn t549_rows(rows: &[Value]) -> Vec<&Value> {
    marked_rows(rows, T549_MARKER)
}

/// The rows whose reconstructed evidence carries the T550 marker.
fn t550_rows(rows: &[Value]) -> Vec<&Value> {
    marked_rows(rows, T550_MARKER)
}

/// The client locale text by Global.Text id.
fn locale_text() -> HashMap<u64, String> {
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
fn t550_rows_are_harvest_plants_and_static_props() {
    let rows = extras_rows();
    let imported = t550_rows(&rows);

    let mut guids = HashSet::new();
    for row in &rows {
        assert!(
            guids.insert(row["spawn_guid"].as_u64().unwrap()),
            "npc_spawns_extra.json has a duplicate spawn guid"
        );
    }
    let regional_guids: HashSet<u64> = regional_rows()
        .iter()
        .map(|row| row["npc_id"].as_u64().unwrap())
        .collect();

    let mut plants: HashMap<String, usize> = HashMap::new();
    let mut props: HashMap<String, usize> = HashMap::new();
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
            "{name}: a T550 row binds no content key"
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
        assert!(
            !model.starts_with("sg_mining_node_") && !model.starts_with("sg_farming_node_"),
            "{name}: a gather node model leaked into the decoration rows"
        );
        if model.ends_with("_harvestable_01.adr") {
            *plants.entry(model).or_default() += 1;
        } else {
            *props.entry(model).or_default() += 1;
        }
    }

    let expected_plants: HashMap<String, usize> = T550_PLANT_SPECIES
        .iter()
        .map(|(model, count)| (model.to_string(), *count))
        .collect();
    let expected_props: HashMap<String, usize> = T550_PROP_MODELS
        .iter()
        .map(|(model, count)| (model.to_string(), *count))
        .collect();
    assert_eq!(plants, expected_plants, "plant species counts changed");
    assert_eq!(props, expected_props, "prop model counts changed");
    assert_eq!(plants.values().sum::<usize>(), T550_PLANTS);
    assert_eq!(props.values().sum::<usize>(), T550_PROPS);
    assert_eq!(imported.len(), T550_ROWS);
}

#[test]
fn t550_house_mailboxes_keep_their_locale_address_names() {
    let locale = locale_text();
    let rows = extras_rows();
    let mut addresses = Vec::new();
    for row in t550_rows(&rows) {
        if row["model_file_name"] == "sg_mailbox_01.adr" {
            let id = row["name_string_id"].as_u64().expect("name_string_id");
            let name = row["name"].as_str().expect("name");
            assert_eq!(
                locale.get(&id).map(String::as_str),
                Some(name),
                "{name}: mailbox name is not locale id {id}"
            );
            addresses.push(name.to_string());
        }
    }
    // The twentieth house mailbox is the Snowhill prop T739 already draws and names.
    for row in regional_rows() {
        if row["placement"] == "legacy_data" && row["model_file_name"] == "sg_mailbox_01.adr" {
            let id = row["name_string_id"]
                .as_u64()
                .expect("regional mailbox name id");
            let name = row["name"].as_str().expect("regional mailbox name");
            assert_eq!(
                locale.get(&id).map(String::as_str),
                Some(name),
                "{name}: mailbox name is not locale id {id}"
            );
            addresses.push(name.to_string());
        }
    }
    assert_eq!(addresses.len(), T550_HOUSE_MAILBOXES);
    assert_eq!(
        addresses.iter().collect::<HashSet<_>>().len(),
        T550_HOUSE_MAILBOXES,
        "two house mailboxes share an address"
    );
}

#[test]
fn t550_plants_are_not_gather_nodes() {
    let content = content_dir();
    let nodes = GatheringNodes::load(&content).unwrap().unwrap();
    let rows = extras_rows();
    let imported = t550_rows(&rows);
    assert!(!imported.is_empty());
    for row in &imported {
        let guid = row["spawn_guid"].as_u64().unwrap();
        let name = row["name"].as_str().unwrap();
        assert!(
            !nodes.is_node(guid),
            "{name}: a decoration row became a gather node"
        );
    }
    // Neither node table names a harvest plant model, so no code path can start a gather run
    // on one even if an actor guid is forged.
    for file in ["harvest_nodes.json", "mining_nodes.json"] {
        let text = fs::read_to_string(content.join(file))
            .unwrap_or_else(|error| panic!("{file}: {error}"))
            .to_ascii_lowercase();
        for (model, _) in T550_PLANT_SPECIES {
            assert!(!text.contains(model), "{file} names plant model {model}");
        }
    }
}

#[test]
fn t549_names_are_client_locale_text() {
    let locale = locale_text();
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

/// Every actor the server draws by proximity: extras rows that pass `ExtraNpc::drawn` and the
/// `legacy_data` regional placements. The knocker row is filtered at load because the client
/// has no base model for it; a row with content key 0 is filtered too.
fn drawn_actors(rows: &[Value]) -> Vec<(u64, String, [f32; 3])> {
    let mut drawn: Vec<(u64, String, [f32; 3])> = rows
        .iter()
        .filter(|row| row["capture_match"].is_null() && row["content_key"] != 0)
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
    drawn
}

/// No marked row duplicates a drawn actor of the same model within one metre. Staging a new
/// row over an existing body would draw two actors in the same place and double the click
/// candidates there, so this is the cross-feature dedupe gate.
fn assert_no_same_model_neighbour(imported: &[&Value], drawn: &[(u64, String, [f32; 3])]) {
    for row in imported {
        let guid = row["spawn_guid"].as_u64().unwrap();
        let name = row["name"].as_str().unwrap();
        let model = row["model_file_name"]
            .as_str()
            .unwrap()
            .to_ascii_lowercase();
        let at = position(row);
        for (other, other_model, other_at) in drawn {
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
fn t549_rows_do_not_duplicate_a_drawn_actor_within_one_metre() {
    let rows = extras_rows();
    let imported = t549_rows(&rows);
    assert_no_same_model_neighbour(&imported, &drawn_actors(&rows));
}

#[test]
fn t550_rows_do_not_duplicate_a_drawn_actor_within_one_metre() {
    let rows = extras_rows();
    let imported = t550_rows(&rows);
    assert_no_same_model_neighbour(&imported, &drawn_actors(&rows));
}

/// Each row is drawn by the same shared nearest-64 fan-out the gateway uses, from its own
/// position, with the content model, name and transform.
fn assert_rows_are_drawn_from_their_own_position(imported: &[&Value]) {
    let visibility = NpcVisibility::load(&content_dir()).unwrap();
    assert!(!imported.is_empty());
    for row in imported {
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
fn t549_rows_reach_the_shared_add_npc_view() {
    let rows = extras_rows();
    assert_rows_are_drawn_from_their_own_position(&t549_rows(&rows));
}

#[test]
fn t550_rows_reach_the_shared_add_npc_view() {
    let rows = extras_rows();
    assert_rows_are_drawn_from_their_own_position(&t550_rows(&rows));
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
