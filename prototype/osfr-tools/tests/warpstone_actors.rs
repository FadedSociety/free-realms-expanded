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
//! model-280 actor, the stock-locale name behind the row's own NameId, and the
//! 35/2 packet the production visibility loader draws.
//!
//! The `animation_id` in 35/2 is the client's `AnimationTypes.xml` slot selector
//! (1 = `loc_stand`), and every content actor carries the template's 1, so that
//! pin alone does not show a visible spin. The ignored private asset test below
//! records the model's stand-clip bank (`sg_warpstone.gr2`) and spin rig; that is
//! evidence the model can animate, not that the client plays the spin, so it is
//! not spin coverage. Whether the stone visibly rotates stays an open native
//! task for the lead (`notes/Autopilot/F780.md`, `notes/Autopilot/F831.md`).

#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;
#[path = "../src/restoration/piece_098.rs"]
#[allow(dead_code)]
mod piece_098;
// Canonical placement helpers (position reader and planar distance).
#[path = "support/placement.rs"]
mod placement;
// The canonical 35/2 view of a fresh `NpcVisibility` update.
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
// The extracted client corpus for the private gate below.
#[path = "support/client_assets.rs"]
mod client_assets;
// One JSON reader for the content gates (`tests/support/json.rs`).
#[path = "support/json.rs"]
mod json;
// The stock `en_us_data.dat` rows behind the row NameIds (`tests/common/npc_greetings.rs`).
#[path = "common/npc_greetings.rs"]
mod npc_greetings;

use osfr_tools::{world::NpcVisibility, world_text::locale_hash};
use piece_077::{OPEN_WORLD_ZONE, curated_definitions};
use piece_098::{load_actor, load_skeleton, probe_granny};
use std::{collections::BTreeSet, path::Path};

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
    let visibility =
        NpcVisibility::load(&content).expect("the production visibility loader accepts it");
    // The stock client locale resolves each pinned NameId the way the client's
    // `Global.Text.<id>` lookup does; the row's own id is checked against this text below.
    let wanted: BTreeSet<u32> = NEW_STONES
        .iter()
        .map(|stone| locale_hash(i32::try_from(stone.name_id).expect("stock locale id fits i32")))
        .collect();
    let locale = npc_greetings::locale_rows(&wanted);

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
        // The client draws the nameplate from the row's own NameId, so the id must
        // name this stone, not merely exist in the locale: `content_npc_spawns` only
        // gates presence, and a pinned id alone would pass if the pin drifted with it.
        let name_id = row["name_string_id"]
            .as_u64()
            .unwrap_or_else(|| panic!("{}: NameId is missing", stone.name));
        let (line, locale_name) = locale
            .get(&locale_hash(
                i32::try_from(name_id).expect("a stock locale id fits i32"),
            ))
            .unwrap_or_else(|| panic!("{}: NameId {name_id} has no stock locale row", stone.name));
        assert_eq!(
            locale_name.as_str(),
            stone.name,
            "{}: NameId {name_id} names {locale_name:?} at en_us_data.dat:{line}",
            stone.name
        );
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
        // Lua heading. `seen_from` is the canonical 35/2 view the other placement
        // suites use (`tests/support/npc_visibility.rs`); `animation_id` only
        // selects the client's stand slot, and the model binding that could spin
        // is the ignored asset test below.
        let spawn = npc_visibility::seen_from(&visibility, stone.position)
            .into_iter()
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

/// Private gate: the stone's actor definition binds one `loc_stand` clip over a rig with the
/// spin bones, while the dais it stands on has no animation bank. Model-bank evidence only,
/// not the native spin.
#[test]
#[ignore = "requires the private extracted client assets (lab/extracted/client-assets)"]
fn the_stone_has_a_stand_animation_bank_and_the_still_dais_does_not() {
    let assets =
        client_assets::Assets::open().expect("the private extracted client assets are absent");

    // The stone: one stand slot, on its own clip, over a rig with the spin bones.
    let stone = load_actor(&assets.read("sg_warpstone_01.adr")).expect("sg_warpstone_01.adr loads");
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
    let rig = load_skeleton(&assets.read("sg_warpstone_01.dsk")).expect("the stone skeleton loads");
    for bone in ["SPIN_BASE", "SPIN_EYE"] {
        assert!(
            rig.bone_names.iter().any(|name| name == bone),
            "the warpstone rig carries {bone}"
        );
    }
    let clip =
        probe_granny(&assets.read("sg_warpstone.gr2")).expect("the stand clip is a Granny file");
    assert!(clip.bytes > 0 && clip.section_count > 0);

    // The dais the stone stands on: no animation bank and no skeleton, so the
    // same stand slot selects nothing. This is the still control for the test.
    let dais =
        load_actor(&assets.read("sg_warpstone_dais_01.adr")).expect("the dais definition loads");
    assert!(
        dais.animation_slots.is_empty(),
        "the still dais must not carry a stand animation"
    );
    assert!(
        dais.skeleton_file.is_none(),
        "the still dais is a static mesh"
    );
}
