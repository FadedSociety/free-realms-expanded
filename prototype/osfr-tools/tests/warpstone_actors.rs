//! T546/F780: the Sanctuary, Lakeshore and Snowhill warpstone actors.
//!
//! Sanctuary-main (20261019@10f129e) places three `sg_warpstone_01.adr` NPCs at
//! those hubs; our 11 Legacy stones cover the other towns. These are
//! `npc_spawns_extra.json` actors, not travel data: `piece_077` is unchanged and
//! attunement still comes from the character's position near the curated hub.
//!
//! Content checks that every extra row already gets (client model table, locale
//! id presence, source kind) stay in `tests/content_npc_spawns.rs`. This suite
//! owns only the three placements: the curated hub they stand at, the unique
//! model-280 actor, and the 35/2 packet the production visibility loader draws.
//!
//! The `animation_id` in 35/2 is the client's `AnimationTypes.xml` slot selector
//! (1 = `loc_stand`), and every content actor carries the template's 1, so that
//! pin alone does not show a visible spin. The private asset test below pins the
//! client model's stand-animation bank instead: the stone has one `loc_stand`
//! slot on its own `sg_warpstone.gr2` clip and a rig with `SPIN_BASE`/`SPIN_EYE`
//! bones, while the still dais beside it has no animation bank at all. The
//! intended spin is the client playing that clip; only a native run can confirm
//! it and that remains a lead task (see `notes/Autopilot/F780.md`).

#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;
#[path = "../src/restoration/piece_098.rs"]
#[allow(dead_code)]
mod piece_098;
// Canonical placement helpers (position reader and planar distance).
#[path = "support/placement.rs"]
mod placement;
// One JSON reader for the content gates (`tests/support/json.rs`).
#[path = "support/json.rs"]
mod json;

use osfr_tools::{startup_world, world::NpcVisibility};
use piece_077::{OPEN_WORLD_ZONE, curated_definitions};
use piece_098::{load_actor, load_skeleton, probe_granny};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// One new stone: the actor guid the row spawns, the curated hub it belongs to,
/// its client name and locale id, and the Lua placement.
struct Stone {
    guid: u64,
    hub_id: i32,
    hub_name_id: i32,
    name: &'static str,
    name_id: u32,
    position: [f32; 3],
    heading: f32,
}

const NEW_STONES: [Stone; 3] = [
    Stone {
        guid: 100000033167,
        hub_id: 5,
        hub_name_id: 3499,
        name: "Sanctuary Warpstone",
        name_id: 20831,
        position: [-1418.3218, -27.002932, 347.82056],
        heading: -0.0622950028,
    },
    Stone {
        guid: 100000033558,
        hub_id: 9,
        hub_name_id: 18899,
        name: "Lakeshore Warpstone",
        name_id: 41298,
        position: [-721.7169, 9.560479, 948.71606],
        heading: -1.81613004,
    },
    Stone {
        guid: 100000033602,
        hub_id: 3,
        hub_name_id: 3961,
        name: "Snowhill Warpstone",
        name_id: 20830,
        position: [50.92852, 32.425617, 370.30267],
        heading: 1.21874107,
    },
];

#[test]
fn the_three_zone_warpstones_are_drawn_at_their_curated_hubs() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extra = json::read(&content.join("npc_spawns_extra.json"));
    let rows = extra["records"].as_array().expect("extra rows");
    let hubs = curated_definitions();
    let mut visibility =
        NpcVisibility::load(&content).expect("the production visibility loader accepts it");

    for stone in &NEW_STONES {
        let row = rows
            .iter()
            .find(|row| row["spawn_guid"].as_u64() == Some(stone.guid))
            .unwrap_or_else(|| panic!("{} stone {} is absent", stone.name, stone.guid));
        // Placement facts not covered by the all-row content gate: a zone
        // placement is drawn, not a clickable menu NPC, and it is the Legacy
        // warpstone model at the client name the locale row supplies.
        assert_eq!(
            row["legacy_npc_id"],
            serde_json::Value::Null,
            "{}: a zone placement gets no Legacy NPC id",
            stone.name
        );
        assert!(
            row.get("content_key").is_none(),
            "{}: a stone is drawn, not a menu NPC",
            stone.name
        );
        assert_eq!(row["model_id"].as_u64(), Some(280), "{}", stone.name);
        assert_eq!(
            row["model_file_name"].as_str(),
            Some("sg_warpstone_01.adr"),
            "{}",
            stone.name
        );
        assert_eq!(row["name"].as_str(), Some(stone.name));
        assert_eq!(
            row["name_string_id"].as_u64(),
            Some(u64::from(stone.name_id)),
            "{}: client NameId",
            stone.name
        );
        let written = placement::xyz(&row["position"]);

        // The stone stands at the curated hub of the same town. Hubs and their
        // name ids are travel data pinned by `piece_077`; comparing them here
        // keeps this suite free of the private locale report.
        let hub = hubs
            .iter()
            .find(|hub| hub.id == stone.hub_id)
            .unwrap_or_else(|| panic!("{}: curated hub {}", stone.name, stone.hub_id));
        assert_eq!(hub.name_id, stone.hub_name_id, "{}: hub name", stone.name);
        let distance = placement::planar_distance(written, [hub.x, hub.y, hub.z]);
        assert!(
            distance <= 15.0,
            "{} stands {distance:.2} m from the {town} hub",
            stone.name,
            town = stone.name.trim_end_matches(" Warpstone")
        );

        // The row's position and heading are the Lua line's own values.
        for axis in 0..3 {
            assert!(
                (written[axis] - stone.position[axis]).abs() < 0.001,
                "{}: axis {axis} differs from the Lua line",
                stone.name
            );
        }
        assert!(
            (row["heading"].as_f64().expect("heading") as f32 - stone.heading).abs() < 0.000_001,
            "{}: heading differs from the Lua line",
            stone.name
        );

        // No second model-280 actor stands at the same town: exactly one (this
        // stone) is within 50 m in the production table, which covers the
        // regional rows as well as the 11 Legacy stones in other towns.
        let neighbours = visibility
            .model_actors()
            .filter(|(zone, model, other)| {
                *zone == OPEN_WORLD_ZONE
                    && *model == 280
                    && placement::planar_distance(written, *other) <= 50.0
            })
            .count();
        assert_eq!(
            neighbours, 1,
            "{}: one model-280 actor within 50 m",
            stone.name
        );

        // The production loader draws this actor, at this position, facing the
        // Lua heading. `animation_id` only selects the client's stand slot; the
        // model-to-clip binding that can actually spin is checked below.
        let spawn = visibility
            .update(OPEN_WORLD_ZONE, stone.position)
            .expect("visibility update")
            .iter()
            .filter_map(|packet| startup_world::parse_add_npc(packet).ok())
            .find(|spawn| spawn.actor_id == stone.guid)
            .unwrap_or_else(|| {
                panic!(
                    "{} actor {} is not drawn at its position",
                    stone.name, stone.guid
                )
            });
        assert_eq!(spawn.model_id, 280);
        assert_eq!(spawn.name_string_id, stone.name_id as i32);
        // `loc_stand`, the slot the client's own actor definition binds.
        assert_eq!(spawn.animation_id, 1);
        let facing = [stone.heading.sin(), 0.0, stone.heading.cos(), 0.0];
        for axis in 0..4 {
            assert!(
                (spawn.rotation[axis] - facing[axis]).abs() < 0.000_001,
                "{}: rotation axis {axis}",
                stone.name
            );
        }
        for axis in 0..3 {
            assert!(
                (spawn.position[axis] - stone.position[axis]).abs() < 0.01,
                "{}: drawn position axis {axis}",
                stone.name
            );
        }
    }
}

/// The private extracted client assets, when the lab is present.
fn client_assets() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/extracted/client-assets");
    root.join("catalog.tsv").is_file().then_some(root)
}

/// Client asset name (lowercase) -> extracted file path, from the pack catalogue.
fn asset_index(root: &Path) -> Option<HashMap<String, PathBuf>> {
    let catalog = fs::read_to_string(root.join("catalog.tsv")).ok()?;
    let mut index = HashMap::new();
    for line in catalog.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() == 7 {
            index.insert(fields[2].to_ascii_lowercase(), root.join(fields[6]));
        }
    }
    Some(index)
}

#[test]
fn the_stone_has_a_stand_animation_bank_and_the_still_dais_does_not() {
    let Some(root) = client_assets() else {
        eprintln!("skip the warpstone stand-clip check: the private client assets are absent");
        return;
    };
    let index = asset_index(&root).expect("the client pack catalogue parses");
    let read = |name: &str| -> Vec<u8> {
        let path = index
            .get(name)
            .unwrap_or_else(|| panic!("client asset {name} is absent from catalog.tsv"));
        fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    };

    // The stone: one stand slot, on its own clip, over a rig with the spin bones.
    let stone = load_actor(&read("sg_warpstone_01.adr")).expect("sg_warpstone_01.adr loads");
    assert_eq!(
        stone.skeleton_file.as_deref(),
        Some("sg_warpstone_01.dsk"),
        "the stone is skinned for animation"
    );
    assert_eq!(
        stone.animation_slots.len(),
        1,
        "the actor definition's only bank entry is loc_stand"
    );
    let slot = &stone.animation_slots[0];
    assert_eq!(slot.slot_name, "loc_stand");
    assert_eq!(slot.gr2_file, "sg_warpstone.gr2");
    assert_eq!(slot.speed, Some(40.0));
    let rig = load_skeleton(&read("sg_warpstone_01.dsk")).expect("the stone skeleton loads");
    for bone in ["SPIN_BASE", "SPIN_EYE"] {
        assert!(
            rig.bone_names.iter().any(|name| name == bone),
            "the warpstone rig carries {bone}"
        );
    }
    let clip = probe_granny(&read("sg_warpstone.gr2")).expect("the stand clip is a Granny file");
    assert!(clip.bytes > 0 && clip.section_count > 0);

    // The dais the stone stands on: no animation bank and no skeleton, so the
    // same stand slot selects nothing. This is the still control for the test.
    let dais = load_actor(&read("sg_warpstone_dais_01.adr")).expect("the dais definition loads");
    assert!(
        dais.animation_slots.is_empty(),
        "the still dais must not carry a stand animation"
    );
    assert!(
        dais.skeleton_file.is_none(),
        "the still dais is a static mesh"
    );
}
