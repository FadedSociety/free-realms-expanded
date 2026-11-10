//! P-games-window: the loading screen table (`splash_screens.json`), the `LoadingScreen.xml` it
//! renders and the dock movies ship combat's ZoneConfig carries.
use osfr_tools::splash_screens::{Rotation, Table};
use serde_json::Value;
use std::{collections::BTreeSet, path::PathBuf};

fn data() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn json(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(data().join(name)).unwrap()).unwrap()
}

fn table() -> Table {
    Table::load(&data()).unwrap().expect("splash_screens.json ships")
}

/// Every destination a player can load into: zones, Games window categories, world events and
/// Pirate's Plunder docks.
fn destinations() -> Vec<String> {
    let mut keys = Vec::new();
    for zone in json("zones.json")["records"].as_array().unwrap() {
        keys.push(format!("zone:{}", zone["id"]));
    }
    let rows = json("activity_catalogue.json");
    let categories: BTreeSet<u64> =
        (rows["rows"].as_array().unwrap().iter()).map(|row| row["category"].as_u64().unwrap()).collect();
    keys.extend(categories.iter().map(|id| format!("category:{id}")));
    for event in json("world_events.json")["events"].as_array().unwrap() {
        keys.push(format!("event:{}", event["key"].as_str().unwrap()));
    }
    let ships = json("ship_combat.json");
    for arena in ships["arenas"].as_array().unwrap() {
        for colour in ships["colours"].as_array().unwrap() {
            keys.push(format!("dock:{}:{}", arena["scene"].as_str().unwrap(), colour.as_str().unwrap()));
        }
    }
    keys
}

#[test]
fn every_destination_resolves_and_the_bare_ones_are_reported() {
    let table = table();
    let all = destinations();
    let mut bare = Vec::new();
    for key in &all {
        for turn in 0..40 {
            assert!(table.pick(&[key], turn).is_some(), "{key} shows a screen");
        }
        if table.own(&[key]).is_none() {
            bare.push(key.as_str());
        }
    }
    println!("{} destinations; no screen of their own (default rotation): {bare:?}", all.len());
    // Every dock, every Games window category and the three themed zones have their own.
    assert!(bare.iter().all(|key| key.starts_with("zone:") || key.starts_with("event:")), "{bare:?}");
    assert_eq!(bare.iter().filter(|key| key.starts_with("zone:")).count(), 14);
    // No row names a destination that does not exist.
    for key in table.destinations.keys() {
        assert!(all.contains(key), "{key} is not a destination");
    }
}

#[test]
fn every_screen_is_in_the_players_install() {
    // The launcher's install: `OSFR_CLIENT_DIR`, else the launcher's default folder.
    let install = std::env::var_os("OSFR_CLIENT_DIR").map(PathBuf::from).or_else(|| {
        let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
        Some(PathBuf::from(home).join("Documents/Free Realms/client"))
    });
    let manifest = install.map(|folder| folder.join("Assets_manifest.txt"));
    let Some(text) = manifest.as_ref().and_then(|path| std::fs::read(path).ok()) else {
        println!("SKIPPED every_screen_is_in_the_players_install: no launcher install at {manifest:?} (set OSFR_CLIENT_DIR)");
        return;
    };
    println!("checked against {manifest:?}");
    let text = String::from_utf8_lossy(&text).to_lowercase();
    let held: BTreeSet<&str> = text.lines().filter_map(|line| line.split(',').next()).collect();
    for screen in &table().screens {
        let stem = screen.file.trim_end_matches(".swf").to_lowercase();
        for kind in ["swf", "gfx", "lst.z"] {
            assert!(held.contains(format!("{stem}.{kind}").as_str()), "{stem}.{kind}");
        }
    }
}

#[test]
fn the_xml_lists_the_default_rotation_the_live_events_and_the_tips() {
    let mut table = table();
    let xml = table.loading_screen_xml(&[]);
    assert!(xml.starts_with("<LoadingScreens>\n\t<LoadingScreen Id=1 Filename=\"loadingscreen.swf\" ShowToMembers=1 ShowToNonMembers=1 Weight=15/>\n"));
    assert_eq!(xml.matches("<LoadingScreen Id=").count(), table.default.len());
    assert!(!xml.contains("membershipupsell"), "everything is free: no membership advert");
    assert!(!xml.contains("_bbe"), "a dock movie is not a loading-screen row");
    assert_eq!(xml.matches("<Text StringId=").count(), table.tips.len());
    assert!(xml.ends_with("</LoadingScreenText>\n"));
    assert!(!xml.contains("StartDate") && !xml.contains("Locale"), "no row is date or locale bound");

    // An event's screens join while it runs, each id once.
    let pirates = table.destinations["dock:ss_pirate_arena:pir_red"].clone();
    table.destinations.insert("event:pirate_week".into(), [pirates, table.default[..1].to_vec()].concat());
    let live = table.loading_screen_xml(&["pirate_week".to_owned(), "unknown".to_owned()]);
    assert_eq!(live.matches("<LoadingScreen Id=").count(), table.default.len() + 1);
    assert!(live.contains("Filename=\"loadingscreen_pirTI_red_bbe.swf\""));
}

#[test]
fn a_destination_rotates_by_the_tables_rule() {
    let mut table = table();
    let seaside = ["zone:11"];
    let seen: BTreeSet<&str> = (0..40).map(|turn| table.pick(&seaside, turn).unwrap().file.as_str()).collect();
    assert_eq!(seen, BTreeSet::from(["loadingscreen.swf", "loadscreen_seaside.swf"]));
    let first = (0..40).filter(|turn| table.pick(&seaside, *turn).unwrap().id == 87).count();
    assert_eq!(first, 30, "weights 30 and 10 share 40 turns");
    table.rotation = Rotation::Sequence;
    let order: Vec<u32> = (0..4).map(|turn| table.pick(&seaside, turn).unwrap().id).collect();
    assert_eq!(order, [87, 1, 87, 1]);
    // The most specific key wins; an unknown one falls back to the default rotation.
    assert_eq!(table.pick(&["game:1", "category:11"], 0).unwrap().id, 82);
    assert_eq!(table.pick(&["zone:999"], 0).unwrap().id, 1);
}

#[test]
fn zone_config_names_each_docks_movie() {
    use osfr_tools::ship_combat;
    let mut table = table();
    let content = ship_combat::Content::load(&data()).unwrap().expect("ship_combat.json ships");
    // Shipped off: the client draws the movie only in a Tutorial state the server does not hold.
    assert!(!table.dock_movies);
    for arena in &content.arenas {
        let names = table.dock_movies(&arena.scene, &content.colours, 0);
        assert!(names.iter().all(String::is_empty), "switched off, no dock names a movie");
    }
    table.dock_movies = true;
    let mut movies = BTreeSet::new();
    for arena in &content.arenas {
        let names = table.dock_movies(&arena.scene, &content.colours, 0);
        assert_eq!(names.len(), content.colours.len());
        let with = ship_combat::zone_config(arena, &content.zone_config, &content.colours, &content.treasure, &names);
        let without = ship_combat::zone_config(arena, &content.zone_config, &content.colours, &content.treasure, &[]);
        let extra: usize = names.iter().take(arena.docks.len()).map(String::len).sum();
        assert_eq!(with.len(), without.len() + extra, "only the movie strings are added");
        for name in names.iter().take(arena.docks.len()) {
            assert!(with.windows(name.len()).any(|window| window == name.as_bytes()), "{name}");
        }
        movies.extend(names);
    }
    assert_eq!(movies.len(), 18, "three maps, six dock colours");
    assert!(movies.contains("loadingscreen_pirAB_purple_bbe.swf"));
}
