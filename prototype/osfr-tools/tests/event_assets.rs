//! EV (owner 2026-11-11): "this needs to be done right so Halloween assets and others in the world
//! after their events aren't stuck and forgotten about." Every placed world row may carry
//! `live_event`, an `event_calendar.json` key; the one gate `EventCalendar::is_running` decides
//! whether the row is in the world, and a row tagged with an event that ended is gone for every
//! viewer through the normal 35/3 diff. `scripts/event_tags.py` keeps the tags and
//! `event_tag_exceptions.json` (the shared signal rule and the rows that stay all year).
//!
//! Row kinds proven here: regional NPC rows, extras (props and dressing), Studio placements, coin
//! bodies, quest and gathering nodes, entrances and battle starters, and the client's own seasonal
//! areas. Field enemies, camps, permanent boomboxes, Atlas markers and scheduled shows are proven
//! beside their modules (`encounter/shared.rs`, `boombox.rs`, `gameplay/travel.rs`,
//! `world_events/schedule.rs`). No test reads today's date: every clock is fixed.
#[path = "common/content_copy.rs"]
#[allow(dead_code)]
mod content_copy;
#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use npc_visibility::{content_dir, seen_from};
use osfr_tools::{
    client_areas::ClientAreas,
    encounter::EncounterEngine,
    event_calendar::Mode,
    harvest_nodes::{GatheringNodes, NodeChange},
    world::{NpcVisibility, Placement},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// 2026-10-31 12:00 UTC: inside Halloween (Oct 1 to Nov 12) and no other shipped event.
const HALLOWEEN_MS: u64 = 1_793_448_000_000;
/// 2026-11-13 00:00 UTC: the first minute after Halloween's last day.
const HALLOWEEN_OVER_MS: u64 = 1_794_528_000_000;
/// 2026-01-15 12:00 UTC: after Snow Days and before the Festival of Hearts: no event runs.
const NO_EVENT_MS: u64 = 1_768_478_400_000;
const HARRY: u64 = 9_500_000_493;
const HARRY_AT: [f32; 3] = [-1307.4, -15.3, 301.8];
const GOOD_CANDY: u64 = 100_000_033_337;
/// The Halloween client areas: Pixiewood 1-10, Briarwood, Snowhill and Stillwater.
const HALLOWEEN_AREAS: std::ops::RangeInclusive<i32> = 3992..=4004;

fn rows(path: &Path, key: &str) -> Vec<Value> {
    json::read(path)[key].as_array().cloned().unwrap_or_default()
}

/// A fixture content tree: the shipped content, with `edit` applied to one file.
fn fixture(tag: &str, file: &str, edit: impl FnOnce(&mut Value)) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("ev-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    content_copy::copy_missing(&content_dir(), &dir);
    let mut doc = json::read(&content_dir().join(file));
    edit(&mut doc);
    let _ = fs::remove_file(dir.join(file));
    fs::write(dir.join(file), serde_json::to_vec(&doc).unwrap()).unwrap();
    dir
}

/// The shared rule of `event_tag_exceptions.json`: the event a row looks like it dresses for.
struct Signals {
    prefixes: Vec<(String, String)>,
    words: Vec<(String, String)>,
}

impl Signals {
    fn load(doc: &Value) -> Self {
        let pairs = |key: &str| -> Vec<(String, String)> {
            (doc["signals"][key].as_object().unwrap().iter())
                .flat_map(|(event, list)| {
                    let list = list.as_array().unwrap();
                    list.iter().map(move |w| (event.clone(), w.as_str().unwrap().to_owned()))
                })
                .collect()
        };
        Self {
            prefixes: pairs("prefixes"),
            words: pairs("words"),
        }
    }

    fn flags(&self, model: &str, names: &[&str]) -> Option<String> {
        let model = model.to_lowercase();
        if let Some((event, _)) = self.prefixes.iter().find(|(_, p)| model.starts_with(p.as_str())) {
            return Some(event.clone());
        }
        for text in std::iter::once(model.as_str()).chain(names.iter().copied()) {
            let lower = text.to_lowercase();
            let tokens: Vec<&str> = lower.split(|c: char| !c.is_ascii_alphanumeric()).filter(|t| !t.is_empty()).collect();
            let squashed: String = tokens.concat();
            let hit = |(_, w): &&(String, String)| tokens.contains(&w.as_str()) || (w.len() > 8 && squashed.contains(w.as_str()));
            if let Some((event, _)) = self.words.iter().find(hit) {
                return Some(event.clone());
            }
        }
        None
    }
}

/// The untagged-event check: no row that looks like event dressing stands untagged unless the
/// exceptions file says why it stays all year; every tag names a calendar event; and the counts.
#[test]
fn every_event_looking_row_is_tagged_or_excused() {
    let dir = content_dir();
    let doc = json::read(&dir.join("event_tag_exceptions.json"));
    let signals = Signals::load(&doc);
    let excused: BTreeMap<String, String> = (doc["rows"].as_array().unwrap().iter())
        .map(|r| (r["id"].as_str().unwrap().to_owned(), r["reason"].as_str().unwrap().to_owned()))
        .collect();
    assert!(excused.values().all(|reason| !reason.trim().is_empty()), "an exception has no reason");
    let areas = ClientAreas::load(&dir).unwrap();
    let mut untagged = Vec::new();
    let mut counts: BTreeMap<(String, &str), usize> = BTreeMap::new();
    let mut seen_ids = BTreeSet::new();
    let mut check = |kind: &'static str, id: String, tag: &Value, model: &str, names: &[&str]| {
        seen_ids.insert(id.clone());
        if let Some(key) = tag.as_str() {
            assert!(areas.calendar().has_event(key), "{id}: live_event {key:?} is not a calendar event");
            *counts.entry((key.to_owned(), kind)).or_default() += 1;
        } else if let Some(event) = signals.flags(model, names) {
            if !excused.contains_key(&id) {
                untagged.push(format!("{id} looks like {event} ({model} {names:?})"));
            }
        }
    };
    for row in rows(&dir.join("npc_spawns_extra.json"), "records") {
        let model = row["model_file_name"].as_str().unwrap_or_default();
        check("spawn", format!("spawn:{}", row["spawn_guid"]), &row["live_event"], model, &[row["name"].as_str().unwrap_or_default()]);
    }
    for entry in fs::read_dir(dir.join("npcs")).unwrap() {
        for row in rows(&entry.unwrap().path(), "records") {
            let model = row["model_file_name"].as_str().unwrap_or_default();
            check("npc", format!("npc:{}", row["npc_id"]), &row["live_event"], model, &[row["name"].as_str().unwrap_or_default()]);
        }
    }
    for (file, kind) in [("quest_nodes.json", "node"), ("collection_nodes.json", "collection_node"), ("harvest_nodes.json", "harvest_node"), ("mining_nodes.json", "mining_node")] {
        let doc = json::read(&dir.join(file));
        let types: BTreeMap<&str, &Value> = (doc["node_types"].as_array().unwrap().iter()).map(|t| (t["key"].as_str().unwrap(), t)).collect();
        let pools: BTreeMap<&str, &str> = (doc["pools"].as_array().unwrap().iter()).map(|p| (p["key"].as_str().unwrap(), p["node_type"].as_str().unwrap())).collect();
        for row in doc["spawns"].as_array().unwrap() {
            let pool = row["pool_key"].as_str().unwrap();
            let node_type = types[pools[pool]];
            let names = [node_type["key"].as_str().unwrap_or_default(), node_type["name"].as_str().unwrap_or_default(), pool];
            check(kind, format!("{kind}:{}", row["id"]), &row["live_event"], "", &names);
        }
    }
    for row in rows(&dir.join("field_enemies.json"), "enemies") {
        check("enemy", format!("enemy:{}", row["guid"]), &row["live_event"], "", &[row["name"].as_str().unwrap_or_default()]);
    }
    for row in rows(&dir.join("entrances.json"), "entrances") {
        let names = [row["title"].as_str().unwrap_or_default(), row["name"].as_str().unwrap_or_default()];
        check("entrance", format!("entrance:{}", row["id"].as_str().unwrap()), &row["live_event"], "", &names);
    }
    for row in rows(&dir.join("field_battles.json"), "battles") {
        let spawner = &row["spawner"];
        check("battle", format!("battle:{}", row["id"]), &spawner["live_event"], "", &[spawner["name"].as_str().unwrap_or_default()]);
    }
    for row in rows(&dir.join("travel.json"), "points") {
        check("travel", format!("travel:{}", row["id"]), &row["live_event"], "", &[row["name"].as_str().unwrap_or_default()]);
    }
    for row in rows(&dir.join("world_events.json"), "events") {
        check("world_event", format!("world_event:{}", row["key"].as_str().unwrap()), &row["schedule"]["live_event"], "", &[]);
    }
    assert!(untagged.is_empty(), "event dressing with no live_event and no exception:\n{}", untagged.join("\n"));
    let stray: Vec<_> = excused.keys().filter(|id| !seen_ids.contains(*id)).collect();
    assert!(stray.is_empty(), "exceptions name rows no placed file has: {stray:?}");
    println!("tagged rows per event and kind: {counts:?}");
    for event in ["halloween", "snow_days", "festival_of_hearts", "birthday_bash", "saint_patricks", "spring_eggstravaganza", "twelve_days"] {
        assert!(counts.keys().any(|(key, _)| key == event), "no row carries {event}");
    }
    assert!(counts[&("halloween".to_owned(), "npc")] >= 15, "{counts:?}");
}

/// The migration: no row reads the old `season` display name any more; each became its key.
#[test]
fn the_season_field_moved_to_live_event_everywhere() {
    let dir = content_dir();
    let mut files: Vec<PathBuf> = ["npc_spawns_extra.json", "npc_spawns_retired.json"].iter().map(|f| dir.join(f)).collect();
    files.extend(fs::read_dir(dir.join("npcs")).unwrap().map(|e| e.unwrap().path()));
    for path in &files {
        for row in rows(path, "records") {
            assert!(row.get("season").is_none(), "{}: {} still has season", path.display(), row);
            assert!(row["field_sources"].get("season").is_none(), "{}: a season source tag", path.display());
        }
    }
    for row in rows(&dir.join("world_events.json"), "events") {
        assert!(row["schedule"].get("season").is_none(), "{} schedule keeps season", row["key"]);
    }
    let dance = rows(&dir.join("world_events.json"), "events").into_iter().find(|r| r["key"] == "halloween_dance_off").unwrap();
    assert_eq!(dance["schedule"]["live_event"], "halloween");
    // The archive: the 12 Halloween decoration rows (T468) keep their event for a future rehome.
    let retired = rows(&dir.join("npc_spawns_retired.json"), "records");
    let halloween = retired.iter().filter(|r| r["live_event"] == "halloween").count();
    assert_eq!(halloween, 12, "the archive's Halloween rows");
}

/// Regional NPC rows: Halloween Harry stands at Halloween, is gone after it, and the admin's
/// toggle brings him out of season and takes him away in season.
#[test]
fn a_regional_event_npc_follows_its_event_and_the_toggle() {
    let seen = |v: &NpcVisibility| seen_from(v, HARRY_AT).iter().any(|npc| npc.actor_id == HARRY);
    assert!(seen(&NpcVisibility::load_in_season_at(&content_dir(), HALLOWEEN_MS).unwrap()));
    assert!(!seen(&NpcVisibility::load_in_season_at(&content_dir(), NO_EVENT_MS).unwrap()));
    let mut forced = NpcVisibility::load_in_season_at(&content_dir(), NO_EVENT_MS).unwrap();
    forced.set_event_modes([("halloween".to_owned(), Mode::On)].into());
    assert!(seen(&forced), "realm halloween on brings Harry out of season");
    let mut stopped = NpcVisibility::load_in_season_at(&content_dir(), HALLOWEEN_MS).unwrap();
    stopped.set_event_modes([("halloween".to_owned(), Mode::Off)].into());
    assert!(!seen(&stopped), "realm halloween off takes him away");
    assert_eq!(stopped.content_key(HARRY), None, "and no click reaches him");
}

/// Props and dressing: a viewer standing by the Goo'd Candy bag at Halloween's end is told to
/// remove it (35/3), and every untagged neighbour stays.
#[test]
fn event_dressing_leaves_a_nearby_viewer_at_the_event_edge() {
    let mut view = NpcVisibility::load_in_season_at(&content_dir(), HALLOWEEN_MS).unwrap();
    let (wanted, _) = view.shared_view(0, HARRY_AT, &[], &BTreeSet::new()).unwrap();
    assert!(wanted.contains(&GOOD_CANDY) && wanted.contains(&HARRY));
    view.set_season_clock(Some(HALLOWEEN_OVER_MS));
    let (after, packets) = view.shared_view(0, HARRY_AT, &[], &wanted).unwrap();
    let gone: BTreeSet<u64> = wanted.difference(&after).copied().collect();
    assert!(gone.contains(&GOOD_CANDY) && gone.contains(&HARRY), "{gone:?}");
    let removed: BTreeSet<u64> = (packets.iter())
        .filter(|p| p.starts_with(&[35, 0, 3, 0]))
        .filter_map(|p| Some(u64::from_le_bytes(p.get(6..14)?.try_into().ok()?)))
        .collect();
    assert_eq!(removed, gone, "each leaving row gets its 35/3");
}

/// The captured 2014 login stream sends a "Happy Halloween!" boombox, a Halloween dispenser and a
/// ghost cat at the Blackspore warpstone. Rows of the same models at the same spot, tagged
/// halloween, make the world table draw them instead (`Gameplay::draws_startup_npc`) all year,
/// and the table shows them only while Halloween runs: no captured copy stands after it.
#[test]
fn the_login_streams_halloween_actors_leave_with_the_event() {
    let snapshot = json::read(&content_dir().join("startup/snapshot.json"));
    let captured: Vec<osfr_tools::startup_world::NpcSpawn> = (snapshot["world"]["npc_spawns"].as_array().unwrap().iter())
        .filter(|npc| [1062, 4579, 1013].contains(&npc["model_id"].as_i64().unwrap()))
        .map(|npc| {
            let p: Vec<f32> = (npc["position"].as_array().unwrap().iter()).map(|v| v.as_f64().unwrap() as f32).collect();
            osfr_tools::startup_world::NpcSpawn {
                model_id: npc["model_id"].as_i64().unwrap() as i32,
                position: [p[0], p[1], p[2], 1.0],
                ..Default::default()
            }
        })
        .collect();
    assert_eq!(captured.len(), 3, "the three captured Halloween actors");
    let game = osfr_tools::gameplay::Gameplay::load(&content_dir(), None).unwrap();
    assert!(captured.iter().all(|npc| game.draws_startup_npc(npc)), "the table replaces every captured copy");
    let rows = [100_000_090_110_u64, 100_000_090_111, 100_000_090_112];
    let inside = NpcVisibility::load_in_season_at(&content_dir(), HALLOWEEN_MS).unwrap();
    let outside = NpcVisibility::load_in_season_at(&content_dir(), NO_EVENT_MS).unwrap();
    assert!(rows.iter().all(|guid| inside.actor_position(*guid).is_some()));
    assert!(rows.iter().all(|guid| outside.actor_position(*guid).is_none()));
}

/// Studio placements: a placement carrying `live_event` stands only while its event runs.
#[test]
fn a_studio_placement_with_an_event_stands_only_while_it_runs() {
    let placement = Placement {
        guid: osfr_tools::world::PLACEMENT_GUID_BASE + 77,
        model_id: 1722,
        name_string_id: 0,
        name: Some("Jack-o-lantern".into()),
        position: [-1300.0, -15.0, 300.0],
        heading: 0.0,
        scale: 1.0,
        extra: serde_json::Map::from_iter([("live_event".to_owned(), json!("halloween"))]),
    };
    let mut view = NpcVisibility::load_in_season_at(&content_dir(), HALLOWEEN_MS).unwrap();
    view.place(&placement);
    assert!(view.actor_position(placement.guid).is_some());
    view.set_season_clock(Some(HALLOWEEN_OVER_MS));
    assert!(view.actor_position(placement.guid).is_none());
}

/// Coins are extras bodies: a coin whose body names an event cannot be seen or picked up after it
/// (the pickup asks `actor_position`, `gameplay/collection_pickup.rs`).
#[test]
fn a_coin_body_with_an_event_is_gone_after_it() {
    let coin = rows(&content_dir().join("collection_coins.json"), "records")[0]["spawn_guid"].as_u64().unwrap();
    let dir = fixture("coin", "npc_spawns_extra.json", |doc| {
        let row = (doc["records"].as_array_mut().unwrap().iter_mut()).find(|r| r["spawn_guid"] == coin);
        row.expect("the coin has a body")["live_event"] = json!("halloween");
    });
    assert!(NpcVisibility::load_in_season_at(&dir, HALLOWEEN_MS).unwrap().actor_position(coin).is_some());
    assert!(NpcVisibility::load_in_season_at(&dir, NO_EVENT_MS).unwrap().actor_position(coin).is_none());
    let _ = fs::remove_dir_all(&dir);
}

/// Quest and gathering nodes: Bag Snatchers' candy bags are parked until Halloween runs, shown
/// while it does, hidden again at its end, and a click on a parked one is refused as gone.
#[test]
fn event_pickups_stand_only_while_their_event_runs() {
    let tagged: BTreeSet<u64> = (rows(&content_dir().join("quest_nodes.json"), "spawns").iter())
        .filter(|r| r["live_event"] == "halloween")
        .map(|r| 0x4e4f_4400_0000_0000 | 3 << 32 | r["id"].as_u64().unwrap())
        .collect();
    let mut nodes = GatheringNodes::load(&content_dir()).unwrap().unwrap();
    // Only `max_active_nodes` rows of a pool are placed.
    let candy: BTreeSet<u64> = nodes.actors().map(|n| n.guid).filter(|g| tagged.contains(g)).collect();
    assert!(!candy.is_empty());
    let shown = |changes: &[NodeChange]| -> BTreeSet<u64> {
        (changes.iter()).filter_map(|c| if let NodeChange::Show(n) = c { Some(n.guid) } else { None }).collect()
    };
    let first = nodes.sync(0, false);
    assert!(shown(&first).is_disjoint(&candy), "closed until the calendar names the event");
    let any = *candy.iter().next().unwrap();
    assert!(nodes.admit_pickup(any, 0, [0.0; 3], false, true).is_err());
    nodes.set_running_events(&["halloween".to_owned()].into());
    assert!(candy.is_subset(&shown(&nodes.sync(1, false))), "the event brings them");
    nodes.set_running_events(&BTreeSet::new());
    let hidden: BTreeSet<u64> = (nodes.sync(2, false).iter())
        .filter_map(|c| if let NodeChange::Hide { guid, .. } = c { Some(*guid) } else { None })
        .collect();
    assert_eq!(hidden, candy, "the end takes exactly them");
}

/// Entrances and battle starters: the Crafty Robgoblins holiday battle (Snow Days) and Stone
/// Heart's Hideaway (Festival of Hearts) are candidates for a viewer only while their event runs.
#[test]
fn event_entrances_and_battle_starters_follow_their_event() {
    let mut engine = EncounterEngine::load(&content_dir(), None).unwrap();
    engine.load_field_battles(&content_dir().join("field_battles.json")).unwrap();
    let mut field = engine.take_shared_fields().unwrap();
    let crafty = 100_003_000_063_u64;
    let hideaway = (rows(&content_dir().join("field_battles.json"), "battles").iter())
        .find(|b| b["id"] == 12303)
        .map(|b| b["spawner"]["guid"].as_u64().unwrap())
        .unwrap();
    let candidates = |field: &osfr_tools::encounter::SharedFieldState| -> BTreeSet<u64> {
        field.interest_candidates(false).iter().map(|c| c.guid).collect()
    };
    field.set_active_quests(&BTreeSet::new(), &BTreeSet::new());
    let off = candidates(&field);
    assert!(!off.contains(&crafty) && !off.contains(&hideaway));
    assert!(field.initialization(crafty).unwrap().is_empty(), "nothing to draw out of its event");
    field.set_active_quests(&BTreeSet::new(), &["snow_days".to_owned(), "festival_of_hearts".to_owned()].into());
    let on = candidates(&field);
    assert!(on.contains(&crafty) && on.contains(&hideaway));
    assert!(off.is_subset(&on), "untagged starters stand either way");
}

/// The client's own seasonal areas (opcode 89) follow the same gate: Halloween's 13 areas are on
/// exactly when `is_running("halloween")` is, by date and by the admin's toggle, and turn off for
/// loaded clients when it ends.
#[test]
fn the_client_seasonal_areas_follow_the_same_gate() {
    let enabled = |areas: &ClientAreas, at: u64| -> BTreeSet<i32> {
        (areas.world_ready_packets(at).iter()).map(|p| i32::from_le_bytes(p[2..6].try_into().unwrap())).collect()
    };
    let halloween: BTreeSet<i32> = HALLOWEEN_AREAS.collect();
    let mut areas = ClientAreas::load(&content_dir()).unwrap();
    for (at, mode) in [(HALLOWEEN_MS, None), (NO_EVENT_MS, None), (NO_EVENT_MS, Some(Mode::On)), (HALLOWEEN_MS, Some(Mode::Off))] {
        areas.set_hand_modes(mode.map(|m| ("halloween".to_owned(), m)).into_iter().collect());
        let running = areas.calendar().is_running("halloween", &areas, at);
        assert_eq!(halloween.is_subset(&enabled(&areas, at)), running, "at {at} with {mode:?}");
        assert_eq!(enabled(&areas, at).is_disjoint(&halloween), !running);
    }
    areas.set_hand_modes(BTreeMap::new());
    assert!(areas.season_changes(HALLOWEEN_MS).is_empty(), "the first call is the baseline");
    let off: BTreeSet<i32> = (areas.season_changes(HALLOWEEN_OVER_MS).iter())
        .filter(|p| p[6] == 0)
        .map(|p| i32::from_le_bytes(p[2..6].try_into().unwrap()))
        .collect();
    assert_eq!(off, halloween, "the end turns every Halloween area off");
}
