//! T753: the world zone/region/place table (`data/content/regions.json`) loads through
//! the live `Gameplay::load` path, classifies the T749 section 3.4 sample points, covers
//! 1,091 of the 1,209 shipped NPC rows, and refuses malformed tables without changing
//! anything.
//!
//! The live table belongs to `world::NpcVisibility` (the NPC content owner `Gameplay::load`
//! calls), so `gameplay_dir()` has no `npcs/` directory and the NPC content — including
//! `regions.json` — comes from the bundled `data/content`. The coverage count is computed
//! from the shipped `npcs/*.json` rows, not from a copy of the bounds.
//!
//! The region bounds are recovered (R): nine-region shapes from `piece_076::REGION_AREAS`
//! and the client `FabledRealmsAreas.xml` place groups, projected to x/z. Client region ids,
//! banner movies and locale ids come from `ScriptsBase.lua:11642-11664` and the client
//! locale export; see `notes/Autopilot/T753.md`.

#[path = "support/json.rs"]
mod json;
#[path = "common/npc_greetings.rs"]
#[allow(dead_code)] // the shared harness carries helpers this suite does not use
mod npc_greetings;

use osfr_tools::{gameplay::Gameplay, world_regions::RegionTable};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "osfr-t753-{tag}-{}-{stamp}-{}",
            std::process::id(),
            NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write_table(&self, value: &Value) -> &Path {
        fs::write(
            self.0.join("regions.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn gameplay() -> Gameplay {
    Gameplay::load(&npc_greetings::gameplay_dir(), None).expect("gameplay content loads")
}

fn shipped() -> Value {
    json::read(&npc_greetings::content_dir().join("regions.json"))
}

fn point(table: &RegionTable, x: f32, z: f32) -> (Option<u32>, Option<&str>) {
    (
        table.region_at(0, x, z),
        table.place_at(0, x, z).map(|place| place.id.as_str()),
    )
}

fn load_mutated(shipped: &Value, mutate: impl FnOnce(&mut Value)) -> RegionTable {
    let mut value = shipped.clone();
    mutate(&mut value);
    let scratch = Scratch::new("load");
    let dir = scratch.write_table(&value);
    RegionTable::load(dir)
        .expect("mutated table loads")
        .expect("mutated table is present")
}

#[test]
fn client_name_ids_are_exactly_the_ids_the_rows_use() {
    let document = shipped();
    let listed = document["client_name_ids"]
        .as_array()
        .expect("client_name_ids")
        .iter()
        .map(|id| id.as_u64().expect("locale id"))
        .collect::<BTreeSet<_>>();
    let used = document["regions"]
        .as_array()
        .expect("regions")
        .iter()
        .chain(document["places"].as_array().expect("places"))
        .map(|row| row["name_string_id"].as_u64().expect("name id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        listed, used,
        "the loader's locale allowlist must be exactly the ids its rows name"
    );
    assert_eq!(listed.len(), 17);
}

#[test]
fn live_gameplay_load_classifies_section_3_4_points() {
    let game = gameplay();
    let table = game
        .region_table()
        .expect("regions.json is loaded through Gameplay::load");
    assert_eq!(table.regions().len(), 10);
    assert_eq!(table.places().len(), 19);
    assert_eq!(
        table.region(1).unwrap().banner_swf,
        None,
        "region 1 has no banner"
    );
    assert_eq!(
        table.region(8).unwrap().banner_swf.as_deref(),
        Some("snowhill_notification.swf")
    );
    assert_eq!(table.region(8).unwrap().name_string_id, 3961);
    assert_eq!(table.region(8).unwrap().parent_zone, 0);

    // T749 section 3.4: named sample points for step 1 (place group), step 2
    // (recovered region only), step 3 (Newbiezone fallback) and step 4 (none).
    let samples = [
        ((-2057.0, 430.0), (Some(1), Some("newbiezone"))),
        ((-1239.0, 533.0), (Some(5), Some("sanctuary"))),
        ((156.0, 390.0), (Some(8), Some("snowhill"))),
        ((-1832.0, 1054.0), (Some(2), Some("blackspore"))),
        ((-751.0, 765.0), (Some(1), Some("lakeshore"))),
        ((-1600.0, -600.0), (None, None)), // Greenwood Forest has no client bounds (R)
        ((-1040.0, -404.0), (Some(4), Some("merryvale"))),
        ((96.0, 1190.0), (Some(7), Some("shrouded_glade"))),
        ((711.0, 2143.0), (Some(10), Some("sunstone_valley"))),
        ((-2700.0, 1600.0), (Some(2), None)), // recovered Blackspore rect, outside its group
        // Section 3.4 priority: the banner region group (50) wins over the named
        // child place (30) where they overlap.
        ((-1403.0, 267.0), (Some(5), Some("sanctuary"))),
        ((9000.0, 9000.0), (None, None)),
    ];
    for ((x, z), expected) in samples {
        assert_eq!(point(table, x, z), expected, "at ({x}, {z})");
    }

    // Region ids are not zone ids: only the world zone 0 has regions.
    for zone in 1..=17 {
        assert_eq!(table.region_at(zone, -2057.0, 430.0), None, "zone {zone}");
        assert!(
            table.place_at(zone, -2057.0, 430.0).is_none(),
            "zone {zone}"
        );
    }
    assert_eq!(table.region_at(0, f32::NAN, 0.0), None);
    assert_eq!(table.region_at(0, 0.0, f32::INFINITY), None);
}

#[test]
fn shipped_npc_rows_cover_1091_of_1209() {
    let game = gameplay();
    let table = game.region_table().expect("live region table");
    let npc_dir = npc_greetings::content_dir().join("npcs");
    let mut paths = fs::read_dir(&npc_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();

    let (mut total, mut in_place, mut in_recovered_region, mut in_any_region, mut covered) =
        (0, 0, 0, 0, 0);
    for path in &paths {
        let document = json::read(path);
        for row in document["records"].as_array().expect("records array") {
            let position = row["position"].as_array().expect("position");
            let x = position[0].as_f64().expect("x") as f32;
            let z = position[2].as_f64().expect("z") as f32;
            let place = table.place_at(0, x, z).is_some();
            let region = table.region_at(0, x, z);
            // Data membership for the two note counts: the recovered nine shapes
            // (844) and their union with the region 1 fallback sphere (905).
            let in_recovered_shape = table
                .regions()
                .iter()
                .any(|row| row.id != 1 && row.contains(x, z));
            let in_any_shape = table.regions().iter().any(|row| row.contains(x, z));
            total += 1;
            if place {
                in_place += 1;
            }
            if in_recovered_shape {
                in_recovered_region += 1;
            }
            if in_any_shape {
                in_any_region += 1;
            }
            if place || region.is_some() {
                covered += 1;
            }
        }
    }

    assert_eq!(total, 1209, "shipped NPC rows");
    assert_eq!(in_place, 981, "rows in a first-party place group");
    assert_eq!(
        in_recovered_region, 844,
        "rows in the recovered nine regions"
    );
    assert_eq!(in_any_region, 905, "rows in a region (including region 1)");
    assert_eq!(covered, 1091, "rows in a place or a region");
}

#[test]
fn malformed_tables_are_refused_and_change_nothing() {
    let shipped = shipped();
    let good = RegionTable::load(&npc_greetings::content_dir())
        .unwrap()
        .unwrap();
    let before = point(&good, 156.0, 390.0);

    let refused = |mutate: fn(&mut Value)| -> String {
        let mut value = shipped.clone();
        mutate(&mut value);
        let scratch = Scratch::new("refused");
        let dir = scratch.write_table(&value);
        RegionTable::load(dir)
            .expect_err("malformed table was accepted")
            .to_string()
    };

    let cases: [(&str, fn(&mut Value)); 21] = [
        ("schema_version", |value| {
            value["schema_version"] = json!(2);
        }),
        ("unknown top-level field", |value| {
            value["unexpected"] = json!(1);
        }),
        ("duplicate place id", |value| {
            let id = value["places"][0]["id"].clone();
            value["places"][1]["id"] = id;
        }),
        ("unknown place region", |value| {
            value["places"][0]["region_id"] = json!(99);
        }),
        ("place name id", |value| {
            value["places"][0]["name_string_id"] = json!(1);
        }),
        ("region name id", |value| {
            value["regions"][0]["name_string_id"] = json!(1);
        }),
        ("duplicate region id", |value| {
            let id = value["regions"][0]["id"].clone();
            value["regions"][1]["id"] = id;
        }),
        ("region outside 1..=10", |value| {
            value["regions"][9]["id"] = json!(11);
        }),
        ("region 1 banner", |value| {
            value["regions"][0]["banner_swf"] = json!("snowhill_notification.swf");
        }),
        ("region missing banner", |value| {
            value["regions"][7]["banner_swf"] = Value::Null;
        }),
        ("world geometry 214", |value| {
            value["zones"][0]["geometry_id"] = json!(214);
        }),
        ("world zone id", |value| {
            value["zones"][0]["id"] = json!(1);
        }),
        ("world scene", |value| {
            value["zones"][0]["scene"] = json!("NotFabledRealms");
        }),
        ("duplicate zone scene", |value| {
            let mut instance = value["zones"][0].clone();
            instance["id"] = json!(17);
            instance["kind"] = json!("instance");
            instance["geometry_id"] = json!(96);
            value["zones"].as_array_mut().unwrap().push(instance);
        }),
        ("missing world zone", |value| {
            value["zones"][0]["id"] = json!(17);
            value["zones"][0]["kind"] = json!("instance");
        }),
        ("non-finite sphere", |value| {
            value["places"][0]["shapes"][0]["x"] = json!(1e40);
        }),
        ("negative radius", |value| {
            value["places"][0]["shapes"][0]["r"] = json!(-5);
        }),
        ("unordered rect", |value| {
            value["regions"][1]["shapes"][0]["x2"] = json!(-3000);
        }),
        ("empty shapes", |value| {
            value["places"][0]["shapes"] = json!([]);
        }),
        ("missing source", |value| {
            value["places"][0]["source"] = json!([]);
        }),
        ("unknown shape field", |value| {
            value["places"][0]["shapes"][0]["y"] = json!(1);
        }),
    ];
    for (what, mutate) in cases {
        let error = refused(mutate);
        assert!(!error.is_empty(), "{what}: refused without a message");
    }

    // Every refusal left the loaded table exactly as it was.
    assert_eq!(point(&good, 156.0, 390.0), before);

    // A content set without the file has no table, not an empty invented one.
    let empty = Scratch::new("missing");
    assert!(RegionTable::load(&empty.0).unwrap().is_none());
}

#[test]
fn section_3_4_priority_area_and_id_tie_breaks_are_applied() {
    let shipped = shipped();
    let template = shipped["places"][0].clone();
    let place = |id: &str, priority: u32, radius: u32| {
        let mut row = template.clone();
        row["id"] = json!(id);
        row["name"] = json!(id);
        row["priority"] = json!(priority);
        row["shapes"] = json!([{"shape": "sphere", "x": 0, "z": 0, "r": radius}]);
        row
    };
    let with_places = |rows: Vec<Value>| {
        load_mutated(&shipped, |value| {
            value["places"] = json!(rows);
        })
    };

    // Higher priority wins even when its group is larger.
    let priority = with_places(vec![place("alpha", 50, 10), place("beta", 30, 100)]);
    assert_eq!(point(&priority, 0.0, 0.0).1, Some("alpha"));
    // Same priority: the smaller group wins.
    let smaller = with_places(vec![place("alpha", 30, 100), place("beta", 30, 10)]);
    assert_eq!(point(&smaller, 0.0, 0.0).1, Some("beta"));
    // Same priority and area: the stable id wins regardless of data order.
    let stable = with_places(vec![place("beta", 30, 10), place("alpha", 30, 10)]);
    assert_eq!(point(&stable, 0.0, 0.0).1, Some("alpha"));

    // Region tie-breaks: smaller recovered region, then smaller region id; the
    // default region 1 is only the last resort.
    let regions = |r3: Value, r10: Value, fallback: Value| {
        load_mutated(&shipped, |value| {
            value["places"] = json!([]);
            value["regions"][0]["shapes"] = json!([fallback]);
            value["regions"][2]["shapes"] = json!([r3]);
            value["regions"][9]["shapes"] = json!([r10]);
        })
    };
    let at_origin = |x: i32, z: i32, r: i32| json!({"shape": "sphere", "x": x, "z": z, "r": r});
    let smaller_region = regions(
        at_origin(0, 0, 100),
        at_origin(0, 0, 50),
        at_origin(0, 0, 1000),
    );
    assert_eq!(point(&smaller_region, 0.0, 0.0), (Some(10), None));
    let same_area = regions(
        at_origin(0, 0, 50),
        at_origin(0, 0, 50),
        at_origin(0, 0, 1000),
    );
    assert_eq!(point(&same_area, 0.0, 0.0), (Some(3), None));
    let fallback_region = regions(
        at_origin(10_000, 10_000, 50),
        at_origin(10_000, 10_000, 50),
        at_origin(0, 0, 1000),
    );
    assert_eq!(point(&fallback_region, 0.0, 0.0), (Some(1), None));
}
