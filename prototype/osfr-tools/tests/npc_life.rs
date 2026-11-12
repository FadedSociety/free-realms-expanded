//! F6 (op-P-npc-life): every passive world actor (NPCs, vendors, quest and gather nodes, props,
//! battle doors) stands on its floor and idles in a pose its own model has.
//!
//! Private gates: the floor comes from the lab terrain bake (`lab/terrain`: the height grids and
//! the `.obs` placed-object collision), the slot tables from the extracted client assets and
//! `Resources/AnimationTypes.xml`. Production reads none of them (`src/npc_life.rs`). The heights
//! are written into the content at rest by `scripts/npc_floor.py`, so a realm without terrain
//! draws them right; whenever `lab/terrain` is present these suites check the shipped data.

#[path = "support/client_assets.rs"]
mod client_assets;
#[path = "support/models.rs"]
mod models;

use osfr_tools::{
    gameplay::Gameplay,
    npc_life::STAND,
    startup_world::{NpcSpawn, parse_add_npc},
    terrain::{FLOOR_CAST_LIFT_M, Terrain},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

/// The owner's bar (rc-0136): no spawn more than half a metre off its floor.
const TOLERANCE_M: f32 = 0.5;
/// Spawns under the terrain grid with no baked floor beneath them: the mines and caves (the
/// Snowhill mine, the Briarwood and Mushroom caverns, Sanctuary's and Merry Vale's caves). The
/// bake keeps only the top layer of the terrain mesh, so their floor is unknown here and they
/// keep their authored height. A ceiling, not a target: a new spawn sunk under the ground
/// fails this suite until it is moved or this bake learns the lower layers.
const UNDERGROUND_MAX: usize = 169;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn lab() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

/// The lab terrain bake, or `None` on a checkout without it (the floor checks then skip).
fn terrain() -> Option<Arc<Terrain>> {
    static TERRAIN: OnceLock<Option<Arc<Terrain>>> = OnceLock::new();
    let terrain = TERRAIN.get_or_init(|| {
        let dir = lab().join("terrain");
        if !dir.join("zone_0.hgt").is_file() {
            eprintln!("lab/terrain is absent: the floor checks skip");
            return None;
        }
        let terrain = Terrain::load(&dir).expect("the lab terrain bake loads");
        assert!(
            terrain.obstacles().has(0),
            "the bake has the zone 0 collision (.obs)"
        );
        Some(Arc::new(terrain))
    });
    terrain.clone()
}

/// The shipped world as a session loads it (`Gameplay::load` on `data/content`).
fn shipped() -> Gameplay {
    Gameplay::load(&content_dir(), None).expect("the shipped content loads")
}

fn at(spawn: &NpcSpawn) -> [f32; 3] {
    [spawn.position[0], spawn.position[1], spawn.position[2]]
}

fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The region a spawn stands in, from the table's own `regions.json`.
fn region(game: &Gameplay, zone: u32, p: [f32; 3]) -> String {
    game.region_table()
        .and_then(|regions| regions.region(regions.region_at(zone, p[0], p[2])?))
        .map_or_else(|| format!("zone {zone}"), |region| region.name.clone())
}

/// The shipped content stands on the floor at rest: every drawn body is within half a metre of
/// it, and the load-time snap (the same rule, `scripts/npc_floor.py` offline) moves nothing.
#[test]
fn every_drawn_spawn_stands_within_half_a_metre_of_its_floor() {
    let Some(terrain) = terrain() else { return };
    let mut game = shipped();
    let authored: HashMap<u64, f32> = game
        .npc_table()
        .expect("the shipped content draws world actors")
        .drawn_spawns()
        .map(|(_, spawn)| (spawn.actor_id, spawn.position[1]))
        .collect();
    game.set_terrain(Some(terrain.clone()));
    let table = game.npc_table().unwrap();
    let warpstones: Vec<[f32; 3]> = table
        .drawn_spawns()
        .filter(|(_, spawn)| spawn.model_id == 280)
        .map(|(_, spawn)| at(spawn))
        .collect();
    let mut moved: BTreeMap<String, [usize; 2]> = BTreeMap::new();
    let mut by_warpstones = 0;
    let mut kept = BTreeMap::<&str, usize>::new();
    let mut underground = BTreeMap::<String, usize>::new();
    let mut off = Vec::new();
    let (mut moves, mut poses) = (String::new(), String::new());
    for (zone, spawn) in table.drawn_spawns() {
        let p = at(spawn);
        let before = authored[&spawn.actor_id];
        if spawn.animation_id != STAND {
            let (guid, model, id) = (spawn.actor_id, spawn.model_id, spawn.animation_id);
            poses.push_str(&format!("{guid}\t{model}\t{id}\n"));
        }
        if (p[1] - before).abs() > f32::EPSILON {
            moved.entry(region(&game, zone, p)).or_default()[usize::from(p[1] > before)] += 1;
            by_warpstones += usize::from(
                warpstones
                    .iter()
                    .any(|w| (w[0] - p[0]).hypot(w[2] - p[2]) < 25.0),
            );
            moves.push_str(&format!(
                "{}\t{}\t{}\t{before}\t{}\t{}\n",
                spawn.actor_id,
                spawn.model_id,
                region(&game, zone, p),
                p[1],
                spawn.npc_name
            ));
        }
        if spawn.terrain_object_id != 0 {
            *kept.entry("zone object").or_default() += 1;
            continue;
        }
        if table.life().is_prop(spawn.model_id) {
            *kept.entry("prop").or_default() += 1;
            continue;
        }
        if table.life().keeps_height(spawn.actor_id) {
            *kept.entry("kept height").or_default() += 1;
            continue;
        }
        let Some(ground) = terrain.height_at(zone, p[0], p[2]) else {
            *kept.entry("no terrain for its zone").or_default() += 1;
            continue;
        };
        match terrain.floor_at(zone, p) {
            Some(floor) if (floor - p[1]).abs() <= TOLERANCE_M => {}
            None if ground > p[1] + FLOOR_CAST_LIFT_M => {
                *underground.entry(region(&game, zone, p)).or_default() += 1;
            }
            floor => off.push(format!(
                "{} {:?} model {} at {:?} ({}): floor {floor:?}",
                spawn.actor_id,
                spawn.npc_name,
                spawn.model_id,
                p,
                region(&game, zone, p),
            )),
        }
    }
    if let Ok(path) = std::env::var("F6_MOVES") {
        fs::write(&path, &moves).unwrap();
        fs::write(format!("{path}.poses"), poses).unwrap();
    }
    let total: usize = moved.values().flatten().sum();
    let caves: usize = underground.values().sum();
    println!(
        "F6 floor: moved {total} spawns, {by_warpstones} of them within 25 m of a warpstone; \
         (down, up) by region: {moved:?}; kept: {kept:?}; under the ground grid: {underground:?}"
    );
    assert!(
        off.is_empty(),
        "{} spawns are more than {TOLERANCE_M} m off the floor:\n{}",
        off.len(),
        off.join("\n")
    );
    assert!(
        caves <= UNDERGROUND_MAX,
        "{caves} spawns stand under the ground grid with no floor below (at most {UNDERGROUND_MAX})"
    );
    assert!(
        total == 0 && by_warpstones == 0,
        "the shipped data is on the floor at rest (run scripts/npc_floor.py); the load moved:\n{moves}"
    );
}

/// The owner's report: NPCs float beside the warpstones. The reconstructed rows around the
/// Sanctuary and Snowhill stones stood at the stone's plinth height (1 to 4.5 m up); the shipped
/// rows now stand on the ground, as the original server's own Warpwatcher did (captured at
/// -28.32 beside the Sanctuary stone at -27.00, `startup/snapshot.json`), at least 0.8 m below
/// the stone, and the load leaves them there.
#[test]
fn the_npcs_beside_the_warpstones_come_down_to_the_ground() {
    let Some(terrain) = terrain() else { return };
    let mut game = shipped();
    let authored = |game: &Gameplay, guid: u64| {
        let (_, spawn) = game
            .npc_table()
            .unwrap()
            .drawn_spawns()
            .find(|(_, spawn)| spawn.actor_id == guid)
            .unwrap_or_else(|| panic!("{guid} is drawn"));
        at(spawn)
    };
    // Doctor Gordon and Pux Frillwillow by the Sanctuary stone, Trina Turtledove by Snowhill's.
    let guids = [9_500_000_574, 9_500_000_517, 9_000_000_492];
    let before: Vec<[f32; 3]> = guids.iter().map(|&guid| authored(&game, guid)).collect();
    game.set_terrain(Some(terrain.clone()));
    let stones: Vec<[f32; 3]> = game
        .npc_table()
        .unwrap()
        .drawn_spawns()
        .filter(|(_, spawn)| spawn.model_id == 280)
        .map(|(_, spawn)| at(spawn))
        .collect();
    for (guid, before) in guids.into_iter().zip(before) {
        let after = authored(&game, guid);
        let floor = terrain.floor_at(0, after).expect("open ground");
        let stone = stones
            .iter()
            .min_by(|a, b| {
                let d = |s: &&[f32; 3]| (s[0] - after[0]).hypot(s[2] - after[2]);
                d(a).total_cmp(&d(b))
            })
            .expect("a warpstone");
        assert!(
            stone[1] - after[1] >= 0.8,
            "{guid} stands on the ground, below the stone at {stone:?}: {after:?}"
        );
        assert_eq!(
            before, after,
            "{guid}: the shipped row is already on the floor"
        );
        assert!(
            (after[1] - floor).abs() < 0.01,
            "{guid} stands on the floor"
        );
        assert_eq!(
            (after[0], after[2]),
            (before[0], before[2]),
            "only the height moves"
        );
    }
}

/// The floor the server sets bodies onto agrees with the original server's own spawns: of the
/// 23 AddNpc rows captured at login (`startup/snapshot.json`), every body stands within half a
/// metre of `Terrain::floor_at`. Its props do not (the 14 warpstones stand 0.8-1.0 m above it,
/// the radio 1.6 m, the Halloween dispenser 1.4 m), which is why prop models keep their height.
#[test]
fn the_floor_agrees_with_the_original_servers_captured_spawns() {
    let Some(terrain) = terrain() else { return };
    let life = osfr_tools::npc_life::Life::load(&content_dir()).unwrap();
    let snapshot = read(&content_dir().join("startup/snapshot.json"));
    let spawns = snapshot["world"]["npc_spawns"].as_array().unwrap();
    assert_eq!(spawns.len(), 23);
    let (mut bodies, mut props, mut off) = (0, 0, Vec::new());
    for spawn in spawns {
        let p: Vec<f32> = spawn["position"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let model = spawn["model_id"].as_i64().unwrap() as i32;
        if life.is_prop(model) {
            props += 1;
            continue;
        }
        bodies += 1;
        let floor = terrain.floor_at(0, [p[0], p[1], p[2]]);
        if floor.is_none_or(|floor| (floor - p[1]).abs() > TOLERANCE_M) {
            off.push(format!("model {model} at {p:?}: floor {floor:?}"));
        }
    }
    assert!(
        off.is_empty(),
        "captured bodies off the floor:\n{}",
        off.join("\n")
    );
    assert_eq!(
        (bodies, props),
        (6, 17),
        "two humans, a skeleton, a ghost cat, two druids"
    );
}

/// The `props` list is exactly the drawn models with no body clip: a model whose own clips
/// (its `.adr`, or an `.agr`'s parts) include a walk, run, emote, fidget or combat slot is a
/// body and is grounded; any other model is a thing and keeps its height. With `F6_DUMP` set
/// to a file, the suite writes every drawn model and its class there (`f6_poses.py` reads it).
#[test]
fn every_drawn_model_is_a_body_or_a_listed_prop_by_its_own_clips() {
    let assets = client_assets::Assets::open().expect("the extracted client assets");
    let names = assets.names();
    let slots = slot_table();
    let models = models::read_models();
    let game = shipped();
    let table = game.npc_table().unwrap();
    // Every drawn model, and the original server's captured login spawns.
    let snapshot = read(&content_dir().join("startup/snapshot.json"));
    let captured = snapshot["world"]["npc_spawns"].as_array().unwrap().iter();
    let drawn: BTreeSet<i32> = table
        .drawn_spawns()
        .map(|(_, spawn)| spawn.model_id)
        .chain(captured.map(|spawn| spawn["model_id"].as_i64().unwrap() as i32))
        .collect();
    let mut wrong = Vec::new();
    let mut dump = String::new();
    for model in drawn {
        let files = models.get(&i64::from(model)).cloned().unwrap_or_default();
        let clips: HashSet<String> = files
            .iter()
            .flat_map(|file| model_slots(&assets, &names, file))
            .collect();
        let body = clips.iter().any(|clip| {
            clip == "loc_walk"
                || clip == "loc_run"
                || slots.get(clip).is_some_and(|(_, kind)| {
                    matches!(kind.as_str(), "Emote" | "Fidgets" | "Combat")
                })
        });
        dump.push_str(&format!(
            "{model}\t{}\t{body}\n",
            files.iter().next().map_or("", String::as_str)
        ));
        if body == table.life().is_prop(model) {
            wrong.push(format!("{model} {files:?} body {body}"));
        }
    }
    if let Ok(path) = std::env::var("F6_DUMP") {
        fs::write(path, dump).unwrap();
    }
    assert!(
        wrong.is_empty(),
        "props list out of step with the clips:\n{}",
        wrong.join("\n")
    );
}

/// Field enemy homes (`field_enemies.json` and the camp bodies) stand on the collision floor at
/// rest, within half a metre, unless `npc_poses.json` keeps the body's height with its evidence.
/// The encounter grounds a home by the same floor (`Terrain::snapped`) and moves none of those
/// on it (`tests/rc0136_combat.rs`); a kept body's leash home is still the floor below it.
#[test]
fn field_enemy_homes_stand_on_the_floor() {
    let Some(terrain) = terrain() else { return };
    let content = content_dir();
    let life = osfr_tools::npc_life::Life::load(&content).unwrap();
    let mut homes: Vec<(u64, [f32; 3])> = Vec::new();
    for enemy in read(&content.join("field_enemies.json"))["enemies"]
        .as_array()
        .unwrap()
    {
        let p: Vec<f32> = enemy["position"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        homes.push((enemy["guid"].as_u64().unwrap(), [p[0], p[1], p[2]]));
    }
    let members: HashSet<u64> = read(&content.join("field_camps.json"))["camps"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|camp| {
            camp["members"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_u64)
        })
        .collect();
    for row in read(&content.join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
    {
        let guid = row["spawn_guid"].as_u64().unwrap();
        if members.contains(&guid) {
            let p: Vec<f32> = row["position"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect();
            homes.push((guid, [p[0], p[1], p[2]]));
        }
    }
    let (mut on, mut underground, mut held, mut off) = (0, 0, 0, Vec::new());
    for (guid, p) in homes {
        match terrain.floor_at(0, p) {
            _ if life.keeps_height(guid) => held += 1,
            Some(floor) if (floor - p[1]).abs() <= TOLERANCE_M => on += 1,
            None => underground += 1,
            floor => off.push(format!("{guid} at {p:?}: floor {floor:?}")),
        }
    }
    println!(
        "F6 enemies: {on} on the floor, {underground} under the ground grid, {held} kept at their height"
    );
    assert!(
        off.is_empty(),
        "field enemy homes off the floor:\n{}",
        off.join("\n")
    );
}

/// Slot name -> (id, type name) from the client's `AnimationTypes.xml`.
fn slot_table() -> HashMap<String, (i32, String)> {
    let path = lab().join("working/OSFR.Client/OSFR Client/Resources/AnimationTypes.xml");
    let text = fs::read_to_string(&path).unwrap();
    let attribute = |element: &str, name: &str| {
        let needle = format!(" {name}=\"");
        let start = element.find(&needle)? + needle.len();
        Some(element[start..start + element[start..].find('"')?].to_string())
    };
    let types: HashMap<String, String> = text
        .split("<AnimationType ")
        .skip(1)
        .filter_map(|e| {
            let e = format!(" {}", e.split('>').next()?);
            Some((attribute(&e, "id")?, attribute(&e, "name")?))
        })
        .collect();
    text.split("<AnimationSlot ")
        .skip(1)
        .filter_map(|element| {
            let element = format!(" {}", element.split('>').next()?);
            let id = attribute(&element, "id")?.parse().ok()?;
            let kind = types.get(&attribute(&element, "type")?)?.clone();
            Some((attribute(&element, "name")?, (id, kind)))
        })
        .collect()
}

/// The slot names a model file binds to a clip: an `.adr`'s section-9 records (`0x01` name,
/// `0x02` `.gr2`; `AnimationSlotRecordRead` `007e6190`), an `.agr`'s parts together.
fn model_slots(
    assets: &client_assets::Assets,
    names: &HashSet<String>,
    file: &str,
) -> HashSet<String> {
    if !names.contains(&file.to_ascii_lowercase()) {
        return HashSet::new();
    }
    let bytes = assets.read(file);
    if file.ends_with(".agr") {
        let text = String::from_utf8_lossy(&bytes);
        return text
            .split("name=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|part| part.to_ascii_lowercase().ends_with(".adr"))
            .flat_map(|part| model_slots(assets, names, part))
            .collect();
    }
    // The scan `restoration/piece_098.rs::scan_animation_slots` makes: a printable name after
    // `0x01 <len>`, then `0x00 0x02 <len>`, then a printable `.gr2` name and `0x00`.
    let printable = |from: usize| {
        let from = from.min(bytes.len());
        let end = (from..bytes.len())
            .find(|&i| !(0x20..0x7f).contains(&bytes[i]))
            .unwrap_or(bytes.len());
        (&bytes[from..end], end)
    };
    let mut slots = HashSet::new();
    for index in (0..bytes.len()).filter(|&i| bytes[i] == 0x01) {
        let (name, end) = printable(index + 2);
        if name.is_empty() || bytes.get(end..end + 2) != Some(&[0x00, 0x02]) {
            continue;
        }
        let (clip, clip_end) = printable(end + 3);
        if clip.ends_with(b".gr2") && bytes.get(clip_end) == Some(&0x00) {
            slots.insert(String::from_utf8_lossy(name).into_owned());
        }
    }
    slots
}

/// Every pose `npc_poses.json` names is a looping idle (AmbientLoops, Stand, Sit, Lie or
/// `loc_stand`) that the model of every row it reaches really binds to a clip. A model the
/// ambient table also idles (`npc_ambient.json` `idle_animation_ids`) poses only within those,
/// and every id the ambient table sends is a clip its row's models bind.
#[test]
fn every_pose_is_a_looping_idle_its_own_model_has() {
    let Some(assets) = client_assets::Assets::open() else {
        panic!("the extracted client assets are missing (lab/extracted/client-assets)");
    };
    let names = assets.names();
    let slots = slot_table();
    let by_id: HashMap<i32, (&String, &String)> = slots
        .iter()
        .map(|(name, (id, kind))| (*id, (name, kind)))
        .collect();
    let models = models::read_models();
    let poses = read(&content_dir().join("npc_poses.json"));
    let ambient = read(&content_dir().join("npc_ambient.json"));
    let ambient_idles: HashMap<i64, BTreeSet<i64>> = ambient["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| {
            let ids: BTreeSet<i64> = row["idle_animation_ids"]
                .as_array()?
                .iter()
                .filter_map(Value::as_i64)
                .collect();
            Some(
                row["model_ids"]
                    .as_array()?
                    .iter()
                    .filter_map(Value::as_i64)
                    .map(move |m| (m, ids.clone())),
            )
        })
        .flatten()
        .collect();
    let looping = ["AmbientLoops", "Stand", "Sit", "Lie"];
    let check = |model: i64, id: i64, owner: &str| {
        let (name, kind) = by_id
            .get(&(id as i32))
            .unwrap_or_else(|| panic!("{owner}: {id} is no AnimationTypes slot"));
        assert!(
            id == i64::from(STAND) || looping.contains(&kind.as_str()),
            "{owner}: {id} {name} is a {kind} slot, not a looping idle"
        );
        let files = models
            .get(&model)
            .unwrap_or_else(|| panic!("{owner}: model {model} is not in Models.txt"));
        assert!(
            files
                .iter()
                .any(|file| model_slots(&assets, &names, file).contains(name.as_str())),
            "{owner}: model {model} {files:?} has no clip for {id} {name}"
        );
        if let Some(idles) = ambient_idles.get(&model) {
            // A stand row (loc_stand) beats the table: the world skips the ambient idle of an
            // actor with its own pose row, so a child with no seat can stand.
            assert!(
                id == i64::from(STAND) || idles.contains(&id),
                "{owner}: npc_ambient idles model {model} on {idles:?}"
            );
        }
    };
    let mut posed_models = HashMap::new();
    for row in poses["models"].as_array().unwrap() {
        let owner = row["model_file_name"].as_str().unwrap();
        for model in row["model_ids"].as_array().unwrap() {
            let model = model.as_i64().unwrap();
            for id in row["animation_ids"].as_array().unwrap() {
                check(model, id.as_i64().unwrap(), owner);
            }
            posed_models.insert(model, owner);
        }
    }
    // A spawn row's id must suit the model the row draws with.
    let mut model_of: HashMap<u64, i64> = HashMap::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        for row in read(&entry.unwrap().path())["records"].as_array().unwrap() {
            if let (Some(guid), Some(model)) = (row["npc_id"].as_u64(), row["model_id"].as_i64()) {
                model_of.insert(guid, model);
            }
        }
    }
    for row in read(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
    {
        model_of.insert(
            row["spawn_guid"].as_u64().unwrap(),
            row["model_id"].as_i64().unwrap_or(0),
        );
    }
    let mut spawn_rows = 0;
    for row in poses["spawns"].as_array().unwrap() {
        let guid = row["guid"].as_u64().unwrap();
        let Some(id) = row["animation_id"].as_i64() else {
            continue;
        };
        let model = *model_of
            .get(&guid)
            .unwrap_or_else(|| panic!("pose spawn {guid} is no NPC row"));
        check(model, id, &format!("spawn {guid}"));
        spawn_rows += 1;
    }
    assert!(
        posed_models.contains_key(&382),
        "the dwarf miners the owner saw are posed"
    );
    assert!(spawn_rows > 0);

    // The ambient table (`npc_ambient.json`, 35/8 emotes, conversation turns and 35/26 idles)
    // names only clips each of its row's models binds, by the same `.adr` rule.
    let mut missing = Vec::new();
    for row in ambient["rows"].as_array().unwrap() {
        let file = row["model_file_name"].as_str().unwrap();
        for model in row["model_ids"].as_array().unwrap() {
            let model = model.as_i64().unwrap();
            let files = models.get(&model).cloned().unwrap_or_default();
            let clips: HashSet<String> = files
                .iter()
                .flat_map(|file| model_slots(&assets, &names, file))
                .collect();
            for list in [
                "animation_ids",
                "conversation_animation_ids",
                "idle_animation_ids",
            ] {
                for id in row[list].as_array().into_iter().flatten() {
                    let id = id.as_i64().unwrap();
                    let name = by_id.get(&(id as i32)).map(|(name, _)| name.as_str());
                    if !name.is_some_and(|name| clips.contains(name)) {
                        missing.push(format!("{file} model {model} {list} {id} {name:?}"));
                    }
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "ambient ids the model lacks:\n{}",
        missing.join("\n")
    );
}

/// The way in: the session's own view (`NpcVisibility::shared_view`, the gateway fan-out)
/// sends the Snowhill mine's dwarf miners swinging their picks: each unnamed miner's 35/2
/// carries one of its model's pick-swing loops, the miners do not all swing the same way, and
/// the vendor among them (Bruno Battlerock <Crafting Supplies>) still stands.
#[test]
fn the_snowhill_miners_swing_their_picks_in_the_session_view() {
    let mut game = shipped();
    game.set_terrain(terrain());
    let table = game.npc_table().unwrap();
    let mut swings = BTreeMap::new();
    let mut stands = BTreeSet::new();
    for spot in [
        [450.0, 47.1, 320.0],
        [440.0, 50.0, 280.0],
        [357.0, 48.4, 321.0],
    ] {
        let (_, packets) = table.shared_view(0, spot, &[], &BTreeSet::new()).unwrap();
        for packet in packets.iter().filter(|p| p.starts_with(&[35, 0, 2, 0])) {
            let spawn = parse_add_npc(packet).unwrap();
            if spawn.model_id != 382 {
                continue;
            }
            if spawn.actor_id == 9_000_000_034 {
                stands.insert(spawn.animation_id);
            } else if spawn.animation_id != STAND {
                swings.insert(spawn.actor_id, spawn.animation_id);
            }
        }
    }
    assert!(
        swings.len() >= 6,
        "the mine's unnamed miners are in view: {swings:?}"
    );
    let kinds: BTreeSet<i32> = swings.values().copied().collect();
    assert!(
        kinds.is_subset(&BTreeSet::from([2100, 2110, 2120])) && kinds.len() > 1,
        "pick swings at the ground, a rock and sideways: {kinds:?}"
    );
    assert_eq!(
        stands,
        BTreeSet::from([STAND]),
        "the vendor stands at his stall"
    );
}

// ---- placement faults the owner reported on 2026-11-12 (npc-placement-b) ------------------------
//
// "those on objects like npcs on carts appear floating on top of whatever they were inside" and
// "the girl sitting outside the cemetery gate": a reconstructed spot (a POI anchor or a zone
// centre, never a captured transform) must not pile bodies on one point, put one on top of an
// object or the water, or sit a child on nothing. These cover the regional files this review
// owns; the other regional files have their own review.

const REVIEWED_FILES: [&str; 8] = [
    "wilds",
    "blackspore_swamp",
    "quest_spawned",
    "shrouded_glade",
    "sunstone_valley",
    "wugachug",
    "stillwater_crossing",
    "highroad_junction",
];

struct Body {
    file: String,
    id: u64,
    name: String,
    pos: [f32; 3],
    guessed: bool,
    drawn: bool,
    /// The Wilds zone-centre pile is another review's (its heights and spots are re-derived there).
    exempt: bool,
}

/// Every body the world draws, at the position it is drawn from: the regional row, or the
/// `npc_spawns_extra.json` body that draws it. `guessed` is a reconstructed POI-anchor or
/// zone-centre spot (the body's own source says so when it has one). With `hidden`, regional rows
/// with no model (nothing drawn) are listed too: an anchor stacked with them is still a pile.
fn drawn_bodies(hidden: bool) -> Vec<Body> {
    let extras = read(&content_dir().join("npc_spawns_extra.json"));
    let rows = extras["records"].as_array().unwrap();
    let bound: HashMap<u64, &Value> = rows
        .iter()
        .filter_map(|row| Some((row["content_key"].as_u64()?, row)))
        .collect();
    let xyz = |row: &Value| {
        let p = row["position"].as_array().unwrap();
        [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32)
    };
    let mut bodies = Vec::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_stem().unwrap().to_string_lossy().into_owned();
        for row in read(&path)["records"].as_array().unwrap() {
            if !row["house_zone_id"].is_null() {
                continue;
            }
            let id = row["npc_id"].as_u64().unwrap();
            let body = bound.get(&id).copied();
            let source = body.unwrap_or(row);
            let drawn = source["model_id"].as_i64().unwrap_or(0) > 0
                && (body.is_some() || row["name_string_id"].as_i64().unwrap_or(0) > 0);
            let reconstructed =
                matches!(row["placement"].as_str(), Some("poi_anchor" | "unanchored"))
                    && !row["position_reason"]
                        .as_str()
                        .is_some_and(|reason| reason.contains("LIVE CAPTURE"))
                    && body.map_or(true, |body| {
                        body["source"].as_array().unwrap().iter().any(|s| {
                            s["kind"] == "reconstructed"
                                && s["evidence"].as_str().is_some_and(|e| {
                                    e.contains("reconstructed poi_anchor")
                                        || e.contains("reconstructed unanchored")
                                })
                        })
                    });
            if drawn || hidden {
                bodies.push(Body {
                    file: file.clone(),
                    id,
                    name: row["name"].as_str().unwrap_or("?").to_owned(),
                    pos: xyz(source),
                    guessed: reconstructed,
                    drawn,
                    exempt: file == "wilds"
                        && row["placement_source"] == "reconstructed_zone_center",
                });
            }
        }
    }
    for row in rows.iter().filter(|row| row["content_key"].is_null()) {
        if row["model_id"].as_i64().unwrap_or(0) > 0 {
            bodies.push(Body {
                file: "extras".to_owned(),
                id: row["spawn_guid"].as_u64().unwrap(),
                name: row["name"].as_str().unwrap_or("?").to_owned(),
                pos: xyz(row),
                guessed: false,
                drawn: true,
                exempt: false,
            });
        }
    }
    bodies
}

/// Bodies on one guessed point read as a pile (Lucinda, the Banshee and the Pumpkin Prince on
/// the Bone Bog Cemetery anchor, 3.3 and 4.6 m apart): the POI spiral steps about 2 to 4 m, so
/// no guessed body stands within 3.5 m of another drawn body or another row of the same files.
#[test]
fn guessed_bodies_do_not_pile_on_one_point() {
    let bodies = drawn_bodies(true);
    let mut piles = Vec::new();
    for a in bodies
        .iter()
        .filter(|b| b.guessed && !b.exempt && REVIEWED_FILES.contains(&b.file.as_str()))
    {
        for b in bodies.iter().filter(|b| {
            b.id != a.id && !b.exempt && (b.drawn || REVIEWED_FILES.contains(&b.file.as_str()))
        }) {
            let near = (a.pos[0] - b.pos[0]).hypot(a.pos[2] - b.pos[2]);
            if near < 3.5 && (a.pos[1] - b.pos[1]).abs() < 3.0 {
                piles.push(format!(
                    "{} {:?} ({}) and {} {:?} are {near:.2} m apart at {:?}",
                    a.id, a.name, a.file, b.id, b.name, a.pos
                ));
            }
        }
    }
    assert!(
        piles.is_empty(),
        "guessed bodies on one point:\n{}",
        piles.join("\n")
    );
}

/// A reconstructed spot is on the ground: not on a prop or the water surface above the terrain
/// (the Sacred Grove Shallows people stood on the water of the river, the Treasure Trove
/// merchant on a raised object), and not buried in it. World-wide; a guessed body kept at its
/// authored height (`keep_height`: a pier, a dock, paving, a cave) is the row's own evidence.
/// The under-the-terrain check covers the files outside `REVIEWED_FILES`: the Wilds, Blackspore
/// and quest-spawned rows are the other placement review's.
#[test]
fn guessed_bodies_do_not_stand_on_an_object_above_the_ground() {
    let Some(terrain) = terrain() else { return };
    let poses = read(&content_dir().join("npc_poses.json"));
    let kept: HashSet<u64> = poses["spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["keep_height"].is_string())
        .filter_map(|row| row["guid"].as_u64())
        .collect();
    let extras = read(&content_dir().join("npc_spawns_extra.json"));
    let drawn_as: HashMap<u64, u64> = extras["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| Some((row["content_key"].as_u64()?, row["spawn_guid"].as_u64()?)))
        .collect();
    let (mut floating, mut buried) = (Vec::new(), Vec::new());
    for body in drawn_bodies(false).iter().filter(|b| b.guessed) {
        if kept.contains(&body.id) || drawn_as.get(&body.id).is_some_and(|g| kept.contains(g)) {
            continue;
        }
        let Some(ground) = terrain.height_at(0, body.pos[0], body.pos[2]) else {
            continue;
        };
        let floor = terrain.floor_at(0, body.pos);
        let line = |what: &str| {
            format!(
                "{} {:?} ({}) stands at {:?}, {what}",
                body.id, body.name, body.file, body.pos
            )
        };
        match floor {
            Some(floor) if floor - ground > 0.4 => {
                floating.push(line(&format!("{:.2} m above the ground", floor - ground)))
            }
            None if ground - body.pos[1] > 0.5
                && !REVIEWED_FILES.contains(&body.file.as_str())
                && body.file != "durango_zoo" =>
            {
                buried.push(line(&format!("{:.2} m under the terrain", ground - body.pos[1])))
            }
            _ => {}
        }
    }
    assert!(
        floating.is_empty(),
        "bodies on an object:\n{}",
        floating.join("\n")
    );
    assert!(
        buried.is_empty(),
        "bodies buried in the terrain:\n{}",
        buried.join("\n")
    );
}

/// A child's only idle is the sit loop, so a child with no seat sits on the ground (Lucinda at
/// the Bone Bog Cemetery gate, whose stand-in body was a child's). Every sitting body in the world has a 2014 seat placement
/// (a bench, chair, stool, stump, sofa, cot, bedroll or log) within 2 m, or evidence for its
/// height (`keep_height`: something the bake lacks holds it up), or it stands.
#[test]
fn a_sitting_body_has_a_seat() {
    let seats_file = lab().join("references/fr-spatial-20261029/prop_placements_2014.tsv");
    let Ok(seats_tsv) = fs::read_to_string(&seats_file) else {
        eprintln!("the 2014 prop placements are absent: the seat check skips");
        return;
    };
    const SEAT: [&str; 11] = [
        "bench", "chair", "stool", "stump", "sofa", "loveseat", "cot_", "bedroll", "logs_01",
        "throne", "lounge",
    ];
    let seats: Vec<[f32; 3]> = seats_tsv
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut c = line.split('\t').skip(1);
            let asset = c.next()?;
            SEAT.iter().any(|s| asset.contains(s)).then_some(())?;
            Some([
                c.next()?.parse().ok()?,
                c.next()?.parse().ok()?,
                c.next()?.parse().ok()?,
            ])
        })
        .collect();
    assert!(
        seats.len() > 100,
        "the seat placements parse: {}",
        seats.len()
    );
    // The child bodies sit on every loop of their pose row; a drinker sits on `sitdrink` (2140),
    // its other loop (2150) stands.
    let pose_rows = read(&content_dir().join("npc_poses.json"));
    let sits = |model: i32, id: i32| {
        pose_rows["models"].as_array().unwrap().iter().any(|row| {
            row["model_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m == model)
                && match row["role"].as_str() {
                    Some("child") => row["animation_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|a| a == id),
                    Some("drinker") => id == 2140,
                    _ => false,
                }
        })
    };
    let game = shipped();
    let table = game.npc_table().unwrap();
    let mut seatless = Vec::new();
    for (zone, spawn) in table.drawn_spawns() {
        let p = at(spawn);
        let area = region(&game, zone, p);
        if !sits(spawn.model_id, spawn.animation_id) || table.life().keeps_height(spawn.actor_id)
        {
            continue;
        }
        let near = seats
            .iter()
            .any(|s| (s[0] - p[0]).hypot(s[2] - p[2]) <= 2.0 && (s[1] - p[1]).abs() < 1.5);
        if !near {
            seatless.push(format!(
                "{} {:?} model {} sits (loop {}) at {p:?} ({area}) with no seat within 2 m",
                spawn.actor_id, spawn.npc_name, spawn.model_id, spawn.animation_id
            ));
        }
    }
    assert!(
        seatless.is_empty(),
        "sitting on nothing:\n{}",
        seatless.join("\n")
    );
}
