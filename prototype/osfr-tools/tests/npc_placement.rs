//! E1, E2, E3 (PL-A npc-placement-a): a body whose spot was guessed (a POI anchor or the zone
//! centre) never stands inside a placed prop or on another body. (E3, the seatless sitters, is
//! `a_sitting_body_has_a_seat` in `npc_life.rs`, world-wide.)
//!
//! Owner, 2026-11-12: "One example of someone not floating but on top of an object is Vladimir
//! the bat quest guy ... plus the girl sitting next to him ... Thats the kind of stuff that
//! can't happen." Vladimir, Ladyslipper and Roxana (the girl) stood among the wrecked caravan
//! (`bw_cart_ransacked_01`, the `quest-merchant-caravan` nodes) that is also placed at the
//! Sanctuary zone centre. They stood on the terrain, not on a prop top: the fault is the spot.
//!
//! The suite reads the shipped data only (the drawn world table and `quest_nodes.json`), so it
//! runs without the lab terrain.

use osfr_tools::gameplay::Gameplay;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// (R) Two bodies closer than this on the ground plane stand on each other. The wagon's wreck
/// pieces and the people around it were 1.8 to 2.4 m apart.
const MIN_BODY_GAP_M: f32 = 2.5;
/// (R) A guessed body this close to a prop actor (a cart, a wagon, a stall) is inside it.
const MIN_PROP_GAP_M: f32 = 3.0;
/// Rows in the zones the other placement builder (PL-B) owns; they are checked and reported but
/// not asserted here until that builder's rows land (delete this list then).
const PL_B_FILES: [&str; 8] = [
    "blackspore_swamp",
    "highroad_junction",
    "quest_spawned",
    "shrouded_glade",
    "stillwater_crossing",
    "sunstone_valley",
    "wilds",
    "wugachug",
];

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// What a regional row says about where its spot came from.
struct Row {
    file: String,
    name: String,
    guessed: bool,
}

/// `npc_id` -> where its placement came from. Guessed: a POI anchor or the zone centre, with no
/// captured spawn behind it.
fn rows() -> HashMap<u64, Row> {
    let mut rows = HashMap::new();
    let mut paths: Vec<_> = fs::read_dir(content_dir().join("npcs"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    for path in paths {
        let file = path.file_stem().unwrap().to_string_lossy().into_owned();
        for record in read(&path)["records"].as_array().unwrap() {
            let id = record["npc_id"].as_u64().unwrap();
            let placement = record["placement"].as_str().unwrap_or("");
            let guessed = matches!(placement, "poi_anchor" | "unanchored")
                && record["capture_match"].is_null();
            rows.insert(
                id,
                Row {
                    file: file.clone(),
                    name: record["name"].as_str().unwrap_or("").to_owned(),
                    guessed,
                },
            );
        }
    }
    rows
}

/// A drawn actor: its guid, name, model, position and whether it is a prop model.
struct Actor {
    guid: u64,
    name: String,
    model: i32,
    position: [f32; 3],
    prop: bool,
}

/// Every actor the shipped world draws in the open world, plus the quest nodes' props.
fn world() -> (Vec<Actor>, Gameplay) {
    let game = Gameplay::load(&content_dir(), None).expect("the shipped content loads");
    let table = game
        .npc_table()
        .expect("the shipped content draws world actors");
    let life = table.life();
    let mut actors: Vec<Actor> = table
        .drawn_spawns()
        .filter(|(zone, _)| *zone == 0)
        .map(|(_, spawn)| Actor {
            guid: spawn.actor_id,
            name: spawn.npc_name.clone(),
            model: spawn.model_id,
            position: [spawn.position[0], spawn.position[1], spawn.position[2]],
            prop: life.is_prop(spawn.model_id),
        })
        .collect();
    let nodes = read(&content_dir().join("quest_nodes.json"));
    let types: HashMap<&str, i64> = nodes["node_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t["key"].as_str().unwrap(), t["model_id"].as_i64().unwrap()))
        .collect();
    let pools: HashMap<&str, i64> = nodes["pools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["key"].as_str().unwrap(),
                types[p["node_type"].as_str().unwrap()],
            )
        })
        .collect();
    for spawn in nodes["spawns"].as_array().unwrap() {
        let p: Vec<f32> = spawn["position"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let model = pools[spawn["pool_key"].as_str().unwrap()] as i32;
        actors.push(Actor {
            guid: 0x4E4F_4400_0000_0000 | 3 << 32 | spawn["id"].as_u64().unwrap(),
            name: spawn["pool_key"].as_str().unwrap().to_owned(),
            model,
            position: [p[0], p[1], p[2]],
            prop: life.is_prop(model),
        });
    }
    (actors, game)
}

fn gap(a: &Actor, b: &Actor) -> f32 {
    (a.position[0] - b.position[0]).hypot(a.position[2] - b.position[2])
}

/// The guessed bodies too close to another actor: (file, name, other, metres).
fn overlaps(actors: &[Actor], rows: &HashMap<u64, Row>) -> Vec<(String, String, String, f32)> {
    let mut found = Vec::new();
    for body in actors {
        let Some(row) = rows.get(&body.guid).filter(|row| row.guessed) else {
            continue;
        };
        if body.prop {
            continue;
        }
        for other in actors.iter().filter(|o| o.guid != body.guid) {
            let limit = if other.prop {
                MIN_PROP_GAP_M
            } else {
                MIN_BODY_GAP_M
            };
            let same_level = (other.position[1] - body.position[1]).abs() < 4.0;
            if same_level && gap(body, other) < limit {
                let other_name = rows
                    .get(&other.guid)
                    .map_or(other.name.clone(), |r| r.name.clone());
                found.push((
                    row.file.clone(),
                    row.name.clone(),
                    other_name,
                    gap(body, other),
                ));
                break;
            }
        }
    }
    found.sort_by(|a, b| a.partial_cmp(b).unwrap());
    found
}

/// E1 + E2: no guessed body inside a placed prop or on another body.
#[test]
fn guessed_npcs_stand_clear_of_props_and_each_other() {
    let (actors, _game) = world();
    let rows = rows();
    let found = overlaps(&actors, &rows);
    let (pending, owned): (Vec<_>, Vec<_>) = found
        .iter()
        .partition(|(file, ..)| PL_B_FILES.contains(&file.as_str()));
    println!(
        "{} overlaps in the other builder's zones (not asserted)",
        pending.len()
    );
    assert!(
        owned.is_empty(),
        "{} guessed NPCs stand inside a prop (< {MIN_PROP_GAP_M} m) or on another body (< {MIN_BODY_GAP_M} m):\n{}",
        owned.len(),
        owned
            .iter()
            .map(|(file, a, b, d)| format!("{file}: {a} is {d:.2} m from {b}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The owner's example: Vladimir, Ladyslipper and the girl who sat beside them (Roxana) in the
/// Sanctuary wreck.
#[test]
fn vladimir_and_ladyslipper_are_out_of_the_wrecked_wagon() {
    let (actors, _game) = world();
    let carts: Vec<&Actor> = actors.iter().filter(|a| a.model == 1050).collect();
    assert!(
        !carts.is_empty(),
        "the caravan wreck is placed (bw_cart_ransacked_01)"
    );
    for (guid, name) in [
        (9_500_000_542_u64, "Vladimir"),
        (9_500_000_503, "Ladyslipper"),
        (9_500_000_526, "Roxana"),
    ] {
        let body = actors
            .iter()
            .find(|a| a.guid == guid)
            .unwrap_or_else(|| panic!("{name} is drawn"));
        for cart in &carts {
            assert!(
                gap(body, cart) >= MIN_PROP_GAP_M,
                "{name} stands {:.2} m from a wrecked cart piece",
                gap(body, cart)
            );
        }
        for other in actors.iter().filter(|a| a.guid != guid && !a.prop) {
            assert!(
                gap(body, other) >= MIN_BODY_GAP_M,
                "{name} stands {:.2} m from actor {}",
                gap(body, other),
                other.guid
            );
        }
    }
}

/// F1: the wiki puts Vladimir "on the south side of Sunset Street". The project compass is +x
/// north, +z east, so the south side is the -x one, where the even-numbered signs stand (2 and 4
/// Sunset St.) and the odd ones (1, 3, 5) are on the north side.
#[test]
fn vladimir_stands_on_the_south_side_of_sunset_street() {
    let (actors, _game) = world();
    let vladimir = actors
        .iter()
        .find(|a| a.guid == 9_500_000_542)
        .expect("Vladimir is drawn");
    let extras = read(&content_dir().join("npc_spawns_extra.json"));
    // The mean x of the signs on each side of the street.
    let side_x = |numbers: &[&str]| {
        let xs: Vec<f32> = extras["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| {
                numbers
                    .iter()
                    .any(|n| row["name"] == format!("{n} Sunset St."))
            })
            .map(|row| row["position"][0].as_f64().unwrap() as f32)
            .collect();
        assert_eq!(
            xs.len(),
            numbers.len(),
            "the Sunset Street signs are placed"
        );
        xs.iter().sum::<f32>() / xs.len() as f32
    };
    let (south, north) = (side_x(&["2", "4"]), side_x(&["1", "3", "5"]));
    assert!(
        south < north,
        "+x is north: the even signs are the south side"
    );
    let x = vladimir.position[0];
    assert!(
        (x - south).abs() < (x - north).abs(),
        "Vladimir (x {x:.1}) is nearer the south signs (x {south:.1}) than the north ones (x {north:.1})"
    );
}
