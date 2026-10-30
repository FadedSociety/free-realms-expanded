//! T878: positions solved from the original-game video archive (placement index, 2026-10-29).
//!
//! Two kinds of row carry them. Twelve regional NPC rows that stood at a reconstructed POI
//! offset now stand at the solved point (`placement: video_index`); they stay undrawn like
//! every reconstructed row (`world.rs` spawns only `legacy_data` regional rows and the extras
//! bodies). Seventeen dungeon files gain `dungeon_data.entrance.position`, the entrance (or the
//! enemy that starts the battle) in the same world frame. Each pin below is the index value,
//! kept apart from the data so a drifted row fails against it.
//!
//! T883 adds the drawn side: Felipe, Murphy and Natali (row plus extras body) and Lucky Chance
//! McXander (row only), and two single-instance props (Hard Briarwood Defense, Shrouded Glade
//! Postboxes), all within the index error radius plus 2 m.

#[path = "support/json.rs"]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;
#[path = "support/placement.rs"]
mod placement;

use osfr_tools::world::NpcVisibility;
use placement::{planar_distance, regional_rows, xyz};
use serde_json::Value;

/// (npc id, name, index x, index z, sightings, videos).
const NPCS: [(u64, &str, f32, f32, u32, u32); 12] = [
    (9_500_000_584, "Foreman Moser", -550.6, -1321.5, 28, 9),
    (9_500_000_380, "Duana", -1094.5, -430.5, 25, 12),
    (9_500_000_609, "Munchi Elwood", -547.0, -1171.1, 22, 17),
    (9_500_000_031, "Grave Elemental", -2331.3, 1234.6, 18, 10),
    (9_500_000_026, "Finn", -1379.4, 318.8, 16, 10),
    (9_500_000_748, "Foreman Hetfield", 130.4, 226.2, 10, 6),
    (9_500_000_008, "Burke", -2459.6, 1164.6, 7, 3),
    (9_500_000_073, "Tormented Spirit", -2305.8, 1079.9, 11, 4),
    (9_500_000_425, "Sous-Chef Elden", -1031.0, -550.3, 8, 6),
    (9_500_000_728, "Boomi McBam", 230.5, 430.5, 5, 3),
    (9_500_000_998, "Sparkolot", 35.8, -416.8, 9, 6),
    (9_500_000_555, "Bandy Randi", -486.6, -1199.8, 4, 3),
];

/// (dungeon file id, index x, index z, sightings, videos, medium confidence).
const DUNGEONS: [(&str, f32, f32, u32, u32, bool); 17] = [
    ("tanglewood_fort", -358.1, 2404.1, 45, 18, false),
    ("bone_bog_cemetery", -2386.9, 1239.0, 14, 6, false),
    ("darvon_s_descent", -901.9, 235.2, 14, 6, false),
    ("forest_troll_fort", -431.5, 1346.3, 12, 5, false),
    ("briar_patch", -406.9, 2094.4, 10, 6, false),
    ("misty_mountain", 160.5, 1072.5, 10, 7, false),
    ("the_snarling_hedges", -900.4, 1836.9, 9, 6, false),
    ("hot_springs_haven", 264.1, 159.9, 8, 7, false),
    ("haunted_mines", -2590.3, 1022.5, 7, 3, false),
    ("sheep_watch", -2114.2, 280.0, 7, 5, false),
    ("snowy_canyon", 421.0, 645.2, 4, 2, false),
    ("bixie_hive", -1355.0, 234.8, 5, 2, true),
    ("forgotten_caves", -940.7, 387.9, 5, 4, true),
    ("prince_of_low_tide", -727.4, -1122.9, 6, 5, false),
    ("snatching_snappers", -172.4, -1198.1, 10, 6, false),
    ("robgoblin_creek", -172.5, 979.5, 5, 3, false),
    ("snakes_in_a_maze", -884.9, 1912.5, 11, 7, false),
];

fn assert_at(position: [f32; 3], x: f32, z: f32, tolerance: f32, owner: &str) {
    assert!(position[1].is_finite(), "{owner}: height");
    let off = planar_distance(position, [x, position[1], z]);
    assert!(off <= tolerance, "{owner}: {off} m from the index point");
}

#[test]
fn moved_npcs_stand_at_their_index_positions() {
    let rows = regional_rows();
    for (id, name, x, z, sightings, videos) in NPCS {
        let row = rows
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("{id}: no regional row"));
        let owner = format!("{id} ({name})");
        assert_eq!(row["name"], name, "{owner}");
        assert_at(xyz(&row["position"]), x, z, 1.0, &owner);
        assert_eq!(row["placement"], "video_index", "{owner}");
        assert_eq!(row["placement_confidence"], "high", "{owner}");
        let reason = row["position_reason"].as_str().unwrap();
        assert!(
            reason.contains(&format!("{sightings} sightings in {videos} videos"))
                && !reason.starts_with("(R)"),
            "{owner}: {reason}"
        );
    }
    let moved = rows.iter().filter(|row| row["placement"] == "video_index");
    assert_eq!(
        moved.count(),
        NPCS.len() + MOVED_NPCS.len(),
        "only the listed rows moved"
    );
}

#[test]
fn moved_npcs_stay_undrawn_and_unbound() {
    let base = NpcVisibility::load(&npc_visibility::content_dir()).unwrap();
    let rows = regional_rows();
    for (id, name, ..) in NPCS {
        let row = rows
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
            .unwrap();
        assert_eq!(base.content_key(id), None, "{name} has a body");
        let seen = npc_visibility::seen_from(&base, xyz(&row["position"]));
        assert!(seen.iter().all(|npc| npc.actor_id != id), "{name} drawn");
        assert!(
            row["quest_ids_given"].as_array().unwrap().is_empty(),
            "{name}"
        );
    }
}

#[test]
fn dungeon_entrances_carry_the_index_position() {
    let dir = npc_visibility::content_dir().join("dungeons");
    for (id, x, z, sightings, videos, medium) in DUNGEONS {
        let file = json::read(&dir.join(format!("{id}.json")));
        let position = &file["records"][0]["dungeon_data"]["entrance"]["position"];
        assert_at(xyz(&position["value"]), x, z, 1.0, id);
        assert!(position["reason"].is_null(), "{id}");
        let tags = position["source"].as_array().unwrap();
        assert!(
            tags[0]["kind"] == "video"
                && tags[0]["evidence"]
                    .as_str()
                    .unwrap()
                    .contains(&format!("{sightings} sightings in {videos} videos")),
            "{id}: first tag cites the index row"
        );
        let reconstructed = |tag: &&Value| tag["kind"] == "reconstructed";
        assert_eq!(
            tags.iter().filter(reconstructed).count() > 0,
            medium || id == "prince_of_low_tide" || id == "sheep_watch",
            "{id}: (R) tags mark medium confidence and the two unreconciled rows"
        );
    }
    let positioned = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| json::read(&entry.unwrap().path()))
        .filter(|file| !file["records"][0]["dungeon_data"]["entrance"]["position"].is_null())
        .count();
    assert_eq!(positioned, DUNGEONS.len(), "only the listed dungeons");
}

/// T883: (name, actor id, index x, index z, error m, sightings, videos, drawn). The ids are
/// the regional `npc_id` (which is also the extras `spawn_guid` of the three drawn NPCs). The
/// index error plus 2 m is the brief's tolerance.
const MOVED_NPCS: [(&str, u64, f32, f32, f32, u32, u32, bool); 4] = [
    ("Felipe", 9_000_000_086, -1270.1, 286.1, 2.5, 15, 6, true),
    ("Murphy", 9_000_000_158, 259.7, 260.0, 4.0, 7, 3, true),
    ("Natali", 9_500_000_360, -698.1, 986.3, 4.0, 2, 2, true),
    (
        "Lucky Chance McXander",
        9_000_000_134,
        -1045.9,
        -462.1,
        4.0,
        2,
        1,
        false,
    ),
];

/// T883 single-instance minigame props: (name, extras spawn_guid, index x, index z, error m,
/// sightings, videos). Both are medium or high, two or more videos, and one row in the data.
const MOVED_PROPS: [(&str, u64, f32, f32, f32, u32, u32); 2] = [
    (
        "Hard Briarwood Defense",
        100_000_002_793,
        -640.7,
        2296.4,
        3.1,
        7,
        3,
    ),
    (
        "Shrouded Glade Postboxes",
        100_000_001_426,
        68.5,
        1181.4,
        3.2,
        4,
        2,
    ),
];

#[test]
fn t883_moved_npcs_and_props_stand_within_the_index_error() {
    let rows = regional_rows();
    let base = NpcVisibility::load(&npc_visibility::content_dir()).unwrap();
    let drawn = placement::drawn_npcs(&base);
    for (name, id, x, z, error, sightings, videos, is_drawn) in MOVED_NPCS {
        let row = rows
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("{id}: no regional row"));
        assert_eq!(row["name"], name);
        let at = xyz(&row["position"]);
        assert_at(at, x, z, error + 2.0, name);
        assert_eq!(row["placement"], "video_index", "{name}");
        assert_eq!(row["placement_source"], "video_archive_index", "{name}");
        let reason = row["position_reason"].as_str().unwrap();
        assert!(
            reason.contains(&format!("{sightings} sighting"))
                && reason.contains(&format!("in {videos} video")),
            "{name}: {reason}"
        );
        // (R) marks the single-video solve and only that one.
        assert_eq!(reason.starts_with("(R)"), videos == 1, "{name}: {reason}");
        // Interaction range reads the row and the client draws the body: they must agree.
        let body = drawn.iter().find(|npc| npc.actor == id);
        assert_eq!(body.is_some(), is_drawn, "{name}: drawn or not");
        if let Some(body) = body {
            assert_at(body.position, x, z, error + 2.0, name);
            assert!(
                planar_distance(body.position, at) < 0.01,
                "{name}: body vs row"
            );
        }
    }
    for (name, guid, x, z, error, ..) in MOVED_PROPS {
        let prop = drawn
            .iter()
            .find(|npc| npc.actor == guid)
            .unwrap_or_else(|| panic!("{name}: not drawn"));
        assert_at(prop.position, x, z, error + 2.0, name);
    }
}
