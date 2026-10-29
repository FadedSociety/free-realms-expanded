//! T753: the world zone/region/place table (`data/content/regions.json`) loads through
//! the live `Gameplay::load` path, classifies the T749 section 3.4 sample points, covers
//! 1,193 of the 1,209 shipped NPC rows, and refuses malformed tables without changing
//! anything.
//!
//! The live table belongs to `world::NpcVisibility` (the NPC content owner `Gameplay::load`
//! calls), so `gameplay_dir()` has no `npcs/` directory and the NPC content — including
//! `regions.json` — comes from the bundled `data/content`. The coverage count is computed
//! from the shipped `npcs/*.json` rows, not from a copy of the bounds.
//!
//! The region and place bounds are client rows of `FabledRealmsAreas.xml` (pinned in
//! `data/reference/area_definitions.json` by `scripts/build_region_areas.py`), projected to
//! x/z; T876 replaced the community nine-region estimates. Client region ids, banner movies
//! and locale ids come from `ScriptsBase.lua:11642-11664` and the client locale export; see
//! `notes/Autopilot/T753.md` and `notes/Autopilot/T876.md`.

#[path = "support/json.rs"]
mod json;
#[path = "common/npc_greetings.rs"]
#[allow(dead_code)] // the shared harness carries helpers this suite does not use
mod npc_greetings;

use osfr_tools::{gameplay::Gameplay, world_regions::RegionTable};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    assert_eq!(listed.len(), 18);
}

#[test]
fn live_gameplay_load_classifies_section_3_4_points() {
    let game = gameplay();
    let table = game
        .region_table()
        .expect("regions.json is loaded through Gameplay::load");
    assert_eq!(table.regions().len(), 10);
    assert_eq!(table.places().len(), 20);
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
        // T876: Greenwood Forest has no area of its own; the Wilds music bed's child TheWild_Bed_3 (area 99,
        // sphere (-1224, -431) r673) covers it, so it is the Wilds place in the default region.
        ((-1600.0, -600.0), (Some(1), Some("wilds"))),
        ((-1040.0, -404.0), (Some(4), Some("merryvale"))),
        ((96.0, 1190.0), (Some(7), Some("shrouded_glade"))),
        ((711.0, 2143.0), (Some(10), Some("sunstone_valley"))),
        // T876: the Blackspore region is the client bed (area 545) plus its child 546 (sphere (-2195, 1189)
        // r480). The old community rectangle also took this corner; the client rows do not.
        ((-2700.0, 1600.0), (None, None)),
        ((-2195.0, 1189.0), (Some(2), Some("blackspore"))),
        // Child beds extend the places: SnowHill_Bed_4 (area 78, sphere (251, 762) r266) was in no region before.
        ((251.0, 762.0), (Some(8), Some("snowhill"))),
        // Section 3.4 priority: the banner region group (50) wins over the named
        // child place (30) where they overlap.
        ((-1403.0, 267.0), (Some(5), Some("sanctuary"))),
        // T876: a place the client defines and the table lacked (area Pixie_Nursery, id 90).
        ((-356.0, -695.0), (Some(1), Some("pixie_nursery"))),
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
fn shipped_place_group_counts_match_the_research_table() {
    // F752 (review finding 4): the per-group counts in notes/Research/Zones and
    // Regions.md section 3.2 are the shipped regions.json grouping, not raw prefixes.
    // The 697 top-level always-on spheres split into 590 grouped + 107 unassigned
    // (T753 open gap); the 10 top-level box rows are likewise unassigned.
    let document = shipped();
    let counts = document["places"]
        .as_array()
        .expect("places")
        .iter()
        .map(|place| {
            (
                place["id"].as_str().expect("place id"),
                place["shapes"].as_array().expect("shapes").len(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let expected = BTreeMap::from([
        ("blackspore", 50),
        ("briarheart", 7),
        ("briarwood", 75),
        ("crossroads", 17),
        ("farnum_farmstead", 6),
        ("highroad_junction", 51),
        ("lakeshore", 18),
        ("memorial_caverns", 21),
        ("merryvale", 57),
        ("newbiezone", 1),
        ("pixie_nursery", 12),
        ("queensfields", 4),
        ("sanctuary", 52),
        ("seaside", 31),
        ("shrouded_glade", 19),
        ("snowhill", 53),
        ("stillwater_crossing", 51),
        ("sunstone_valley", 9),
        ("wilds", 170),
        ("wugachug", 4),
    ]);
    assert_eq!(counts, expected, "per-group client sphere counts");
    // T876: 602 top-level spheres (590 grouped by T753 plus the 12 Pixie Nursery rows) and 106 rows of the
    // places' music/ambience area families (children extend their parent: client Enable 0x00bed0c0).
    assert_eq!(counts.values().sum::<usize>(), 708, "grouped shapes");
    let region_shapes = document["regions"]
        .as_array()
        .expect("regions")
        .iter()
        .skip(1)
        .map(|region| region["shapes"].as_array().expect("shapes").len())
        .sum::<usize>();
    // T876: regions 2..=10 are the area families of their places (client rows, no estimated shape left).
    assert_eq!(region_shapes, 107, "banner region shapes");

    // The live owner loads the same 19 rows through Gameplay::load.
    let live = gameplay();
    assert_eq!(
        live.region_table().expect("live table").places().len(),
        expected.len(),
        "live place rows"
    );
}

#[test]
fn shipped_npc_rows_cover_1193_of_1209() {
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
            // Data membership for the two note counts: the banner-region shapes (892)
            // and their union with the region 1 fallback sphere (953).
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
    // T876 (was 981 / 844 / 905 / 1091 with the T753 grouping and the community region shapes): the places'
    // music/ambience area families and the client-row regions cover 1,193 rows, of which every one is in a place.
    assert_eq!(in_place, 1193, "rows in a first-party place group");
    assert_eq!(
        in_recovered_region, 892,
        "rows in the banner regions 2..=10"
    );
    assert_eq!(in_any_region, 953, "rows in a region (including region 1)");
    assert_eq!(covered, 1193, "rows in a place or a region");
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
            // Park every region far away so only the three under test can match.
            for region in value["regions"].as_array_mut().unwrap() {
                region["shapes"] = json!([{"shape": "sphere", "x": 15000, "z": 15000, "r": 1}]);
            }
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

fn pinned_zones() -> Vec<Value> {
    let path = npc_greetings::content_dir().join("../reference/area_definitions.json");
    json::read(&path)["zones"]
        .as_array()
        .expect("zones")
        .clone()
}

fn world_rows() -> Vec<Value> {
    let zones = pinned_zones();
    let world = zones.iter().find(|zone| zone["zone"] == "FabledRealms");
    world.expect("FabledRealms zone")["rows"]
        .as_array()
        .unwrap()
        .clone()
}

fn area<'a>(rows: &'a [Value], id: u64, name: &str) -> &'a Value {
    let row = rows.iter().find(|row| row["id"] == id).expect("area id");
    assert_eq!(row["name"], name, "area {id}");
    row
}

/// Whether a table shape is exactly this client row (sphere centre/radius, or box corners as a rect).
fn is_row(shape: &Value, row: &Value) -> bool {
    let same = |a: &str, b: &str| shape[a].as_f64() == row[b].as_f64();
    match shape["shape"].as_str().unwrap() {
        "sphere" => row["shape"] == "sphere" && same("x", "x") && same("z", "z") && same("r", "r"),
        _ => {
            let (x, z) = (["x1", "x2"], ["z1", "z2"]);
            row["shape"] == "box" && x.iter().all(|k| same(k, k)) && z.iter().all(|k| same(k, k))
        }
    }
}

/// Every shape of every region and place that is not a client row, as "owner: shape" (the
/// Sanctuary-style reconstructed shapes would be listed here).
fn shapes_without_a_client_row(document: &Value, rows: &[Value]) -> Vec<String> {
    let mut found = Vec::new();
    for key in ["regions", "places"] {
        for owner in document[key].as_array().unwrap() {
            for shape in owner["shapes"].as_array().unwrap() {
                if !rows.iter().any(|row| is_row(shape, row)) {
                    found.push(format!("{}: {shape}", owner["id"]));
                }
            }
        }
    }
    found
}

#[test]
fn pinned_client_area_rows_carry_their_provenance() {
    let zones = pinned_zones();
    // The packs of this build: 130 zone files, 2,376 rows (the community bundle's 2,323 is an older build).
    assert_eq!(zones.len(), 130);
    let mut total = 0;
    for zone in &zones {
        let rows = zone["rows"].as_array().unwrap();
        total += rows.len();
        let name = zone["zone"].as_str().unwrap();
        assert_eq!(zone["file"], format!("{name}Areas.xml"));
        assert!(zone["pack"].as_str().unwrap().ends_with(".pack"), "{name}");
        assert!(zone["entry"].is_u64(), "{name}");
        assert_eq!(zone["crc32"].as_str().unwrap().len(), 8, "{name}");
        assert!(
            rows.iter().all(|row| row["line"].as_u64().unwrap() > 0),
            "{name}"
        );
    }
    assert_eq!(total, 2376);
    // The world zone's area ids are unique (one zone, ss_cray_caves, reuses seven ids in the client itself).
    let world = world_rows();
    let ids = world.iter().map(|row| row["id"].as_u64().unwrap());
    assert_eq!(ids.collect::<BTreeSet<_>>().len(), 1279);
    assert_eq!(world.len(), 1279);
}

#[test]
fn region_and_place_shapes_are_client_rows_except_where_listed() {
    let document = shipped();
    let rows = world_rows();
    assert_eq!(
        shapes_without_a_client_row(&document, &rows),
        Vec::<String>::new(),
        "every shape of the table is a pinned FabledRealms row"
    );
    // The region extents are the places' music/ambience families, no longer the community estimates.
    for region in document["regions"].as_array().unwrap().iter().skip(1) {
        let kinds = region["source"].as_array().unwrap();
        let text = |row: &Value| row["evidence"].as_str().unwrap().to_owned();
        assert!(
            kinds
                .iter()
                .any(|tag| tag["kind"] == "client" && text(tag).contains("T876"))
        );
        assert!(
            !kinds
                .iter()
                .any(|tag| text(tag).contains("T749 section 3.1")),
            "region {}",
            region["id"]
        );
    }

    // Sampled replaced shapes, pinned against the client rows by area id.
    let owner = |key: &str, id: Value| {
        document[key]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()
            .clone()
    };
    let has = |owner: &Value, row: &Value| {
        owner["shapes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| is_row(s, row))
    };
    let snowhill_bed = area(&rows, 3, "Snowhill_Bed_1");
    let snowhill_child = area(&rows, 78, "SnowHill_Bed_4");
    let blackspore_child = area(&rows, 546, "BlackSpore_Swamp_Bed_2");
    let brewery = area(&rows, 91, "TheWild_WaguChugBrewery");
    let seaside_box = area(&rows, 2250, "Seaside_area_1");
    for row in [snowhill_bed, snowhill_child] {
        assert!(
            has(&owner("regions", json!(8)), row) && has(&owner("places", json!("snowhill")), row)
        );
    }
    assert!(has(&owner("regions", json!(2)), blackspore_child));
    assert!(has(&owner("places", json!("blackspore")), blackspore_child));
    assert_eq!(
        owner("regions", json!(9))["shapes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        has(&owner("regions", json!(9)), brewery),
        "Wugachug is the client brewery area, radius 150 not 165"
    );
    assert!(
        has(&owner("regions", json!(6)), seaside_box),
        "Seaside stays the client box"
    );

    // Canary: a shape nudged off its client row is reported (the check is not vacuous).
    let mut broken = document.clone();
    broken["regions"][7]["shapes"][0]["r"] = json!(136);
    assert_eq!(shapes_without_a_client_row(&broken, &rows).len(), 1);
}
