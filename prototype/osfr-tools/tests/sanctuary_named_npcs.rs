//! T470: 90 named NPCs that were only undrawn regional rows now stand at Sanctuary-main's
//! placements (Card Duelist givers Senari and Mayor Crystalline, the Sanctuary, Snowhill and
//! Lakeshore townsfolk and merchants, and more).
//!
//! Two records per NPC: an `npc_spawns_extra.json` row (the body the client draws) and the
//! regional row, now `placement: reference_data` at the same position. NPC clicks are
//! range-checked against the regional row (`npc_interaction.rs` `check_range`), so the two
//! must agree or a click lands on empty ground. Client evidence: 35/2 AddNpc reader
//! `0x00921a10` (u64 actor, i32 name id, i32 model id); 26/8 constructor `0x008b6bc0`
//! (u64 target + bool); 26/9 chain `0x00aa2360 -> 0x00aa2220 -> 0x00aa1600 -> 0x00aa04d0`.
//! No new wire shape.
//!
//! T506 moved 19 more drawn NPCs (the Lakeshore merchants, Skye, Drill Sergeant Dewey and
//! others) from their POI guesses to the same placements, so 109 rows are `reference_data`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/placement.rs"]
mod placement;
#[path = "support/terrain.rs"]
mod terrain;

use npc_quest_chain::{
    CHARACTER, Fixture, TALK_LABEL, character_at_for, content_dir, interact, menu_option,
    npc_interact, npc_select, read_json, saved, seen_from,
};
use osfr_tools::{startup_world::parse_add_npc, world::NpcVisibility};
use placement::{distance, xyz};
use serde_json::Value;
use std::{collections::BTreeSet, fs};

const REF: &str = "Sanctuary-main-20261019@10f129e";
/// Mayor Crystalline: Snowhill quest giver and trainer, moved from the Snowhill zone-centre
/// guess to her Sanctuary-main spawn (FabledRealms.lua:3946).
const MAYOR: u64 = 9_000_000_143;
const MAYOR_OLD_POSITION: [f32; 3] = [52.757_717, 30.987, 367.513_31];
/// T506: Drill Sergeant Dewey, and where the POI guess put him (the Sanctuary POI).
const DEWEY: u64 = 9_000_000_076;
const DEWEY_OLD_POSITION: [f32; 3] = [-1412.821_604, -27.631, 353.933_558];
/// Legacy-data anchors that mark the towns: Ty (Lakeshore) and Brody Sparfist (Snowhill).
const TY: u64 = 9_000_000_223;
const BRODY: u64 = 9_000_000_033;
/// Names the lead held back (the regions disagree, so it may be another NPC) plus the two
/// duplicate rows of NPCs that are already drawn: Morninglory (extras 9000000155) and Chip
/// Numbwing (extras 9500000736).
const HELD_BACK: [(&str, u64); 6] = [
    ("Clark", 9_500_000_342),
    ("Finn", 9_500_000_026),
    ("Little J", 9_500_000_699),
    ("Raven", 9_500_000_336),
    ("Morninglory duplicate", 9_500_000_506),
    ("Chip Numbwing seaside duplicate", 9_500_000_566),
];
const NOW: u64 = 10_000;

fn regional_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for entry in fs::read_dir(content_dir().join("npcs")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            rows.extend(read_json(&path)["records"].as_array().unwrap().clone());
        }
    }
    rows
}

fn extra_rows() -> Vec<Value> {
    read_json(&content_dir().join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .clone()
}

/// The regional rows this task moved, each with its one extras body.
fn moved() -> Vec<(Value, Value)> {
    let extras = extra_rows();
    regional_rows()
        .into_iter()
        .filter(|row| row["placement"] == "reference_data")
        .map(|row| {
            let id = row["npc_id"].as_u64().unwrap();
            let body = extras
                .iter()
                .find(|extra| extra["content_key"].as_u64() == Some(id))
                .unwrap_or_else(|| panic!("NPC {id} has no extras body"))
                .clone();
            (row, body)
        })
        .collect()
}

fn body_position(id: u64) -> [f32; 3] {
    extra_rows()
        .iter()
        .find(|row| row["content_key"].as_u64() == Some(id))
        .map(|row| xyz(&row["position"]))
        .unwrap()
}

#[test]
fn every_reference_row_is_drawn_once() {
    let rows = moved();
    assert_eq!(
        rows.len(),
        109,
        "T470 drew 90 named NPCs, T506 moved 19 more"
    );
    let extras = extra_rows();
    let guids: BTreeSet<_> = extras
        .iter()
        .map(|extra| extra["spawn_guid"].as_u64().unwrap())
        .collect();
    assert_eq!(guids.len(), extras.len(), "every spawn_guid is unique");
    let keys: Vec<_> = extras
        .iter()
        .filter_map(|extra| extra["content_key"].as_u64())
        .collect();
    assert_eq!(
        keys.iter().collect::<BTreeSet<_>>().len(),
        keys.len(),
        "every content_key is unique"
    );
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for (regional, body) in &rows {
        let id = regional["npc_id"].as_u64().unwrap();
        let owner = format!("{id} ({})", regional["name"]);
        assert_eq!(body["spawn_guid"].as_u64(), Some(id), "{owner}");
        assert!(body["legacy_npc_id"].is_null(), "{owner}");
        assert_eq!(
            extras
                .iter()
                .filter(|extra| extra["spawn_guid"].as_u64() == Some(id)
                    || extra["content_key"].as_u64() == Some(id))
                .count(),
            1,
            "{owner}: the id names one extras row"
        );
        assert_eq!(base.content_key(id), Some(id), "{owner}");
        let position = xyz(&body["position"]);
        assert_eq!(xyz(&regional["position"]), position, "{owner}");

        // The regional rotation is the Y quaternion of the spawn heading (world.rs yaw).
        let heading = body["heading"].as_f64().unwrap();
        assert_eq!(regional["heading"].as_f64(), Some(heading), "{owner}");
        let rotation = &regional["rotation"];
        let yaw = 2.0
            * rotation[1]
                .as_f64()
                .unwrap()
                .atan2(rotation[3].as_f64().unwrap());
        assert!((yaw - heading).abs() < 1e-9, "{owner}: rotation yaw");

        let seen = seen_from(&base, position);
        let drawn: Vec<_> = seen.iter().filter(|npc| npc.actor_id == id).collect();
        assert_eq!(drawn.len(), 1, "{owner}: exactly one drawn actor");
        assert_eq!(
            drawn[0].model_id,
            body["model_id"].as_i64().unwrap() as i32,
            "{owner}"
        );
        assert_eq!(
            drawn[0].name_string_id,
            body["name_string_id"].as_i64().unwrap() as i32,
            "{owner}: the AddNpc name is a Global.Text id"
        );
        assert_eq!(base.actor_position(id), Some(position), "{owner}");
    }
    let names: Vec<_> = extras
        .iter()
        .filter(|extra| extra["name"] == "Morninglory")
        .collect();
    assert_eq!(names.len(), 1, "there is only one Morninglory");
}

#[test]
fn held_back_rows_stay_undrawn() {
    let regional = regional_rows();
    let base = NpcVisibility::load(&content_dir()).unwrap();
    for (label, id) in HELD_BACK {
        let row = regional
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("{label} row {id}"));
        assert_ne!(row["placement"], "reference_data", "{label}");
        assert_ne!(row["placement"], "legacy_data", "{label}");
        assert_eq!(base.content_key(id), None, "{label} has a body");
        assert_eq!(base.actor_position(id), None, "{label} is drawn");
        let seen = seen_from(&base, xyz(&row["position"]));
        assert!(seen.iter().all(|npc| npc.actor_id != id), "{label} drawn");
    }
}

#[test]
fn gulsh_is_the_cave_seller() {
    let rows = moved();
    // Names that match several regional rows: only the chosen one is drawn.
    for (name, chosen) in [
        ("Gulsh", 9_500_000_492_u64),
        ("Cragara", 9_500_000_739),
        ("Tiger the Party Animal", 9_500_000_774),
    ] {
        let ids: Vec<_> = rows
            .iter()
            .filter(|(row, _)| row["name"] == name)
            .map(|(row, _)| row["npc_id"].as_u64().unwrap())
            .collect();
        assert_eq!(ids, [chosen], "{name}");
    }
    let (gulsh, body) = rows
        .iter()
        .find(|(row, _)| row["npc_id"].as_u64() == Some(9_500_000_492))
        .unwrap();
    assert_eq!(gulsh["zone"], "Sanctuary");
    assert_eq!(body["name"], "Gulsh");
    // His spawn is 20.7 m under the terrain grid: the cave below the Sanctuary bridge.
    // Measured on the shipped grid, not read from the extras row's own source text.
    let [x, y, z] = xyz(&body["position"]);
    let ground = terrain::load()
        .height_at(0, x, z)
        .expect("terrain covers his spawn");
    assert!(
        (y - (ground - 20.7)).abs() < 1.0,
        "Gulsh stands at y {y}, ground {ground}: expected about 20.7 m below"
    );
}

/// A 26/8 click from 2 m of the drawn body opens the menu; the same click and a 26/10 choice from
/// the old guess are out of range and change nothing.
fn click_works_only_at_the_new_position(npc: u64, old_position: [f32; 3]) {
    let fixture = Fixture::for_character(CHARACTER);
    let mut game = fixture.game(None);
    // Where the client draws the NPC: the extras body, not the regional row.
    let position = body_position(npc);

    let near = character_at_for(CHARACTER, 0, [position[0] + 2.0, position[1], position[2]]);
    let menu = game
        .handle_in_world(&npc_interact(npc), &near, NOW)
        .unwrap();
    assert!(
        menu.replies
            .iter()
            .any(|packet| packet.starts_with(&[26, 0, 9, 0])),
        "a player 2 m from {npc}'s new position gets the menu"
    );

    // The old position is out of range. With the menu open, a far 26/8 and a far 26/10 are
    // both refused and change nothing: the open menu still answers from beside the NPC, exactly
    // as it does in a game that never saw the far clicks.
    let mut control = fixture.game(None);
    let menu = interact(&mut control, &near, npc, NOW);
    let talk = menu_option(&menu, TALK_LABEL);
    let expected = control
        .handle_in_world(&npc_select(npc, talk), &near, NOW + 3)
        .unwrap();
    assert!(
        !expected.replies.is_empty(),
        "Talk answers with the dialogue"
    );

    let mut game = fixture.game(None);
    interact(&mut game, &near, npc, NOW);
    let state = game.state();
    let bytes = saved(&game);
    let far = character_at_for(CHARACTER, 0, old_position);
    for (packet, message) in [
        (npc_interact(npc), "NPC interaction is unavailable"),
        (npc_select(npc, talk), "NPC menu choice is unavailable"),
    ] {
        let refusal = game
            .handle_in_world(&packet, &far, NOW + 1)
            .err()
            .expect("a far click is refused");
        assert_eq!(refusal.to_string(), message, "{:?}", &packet[..4]);
    }
    assert_eq!(game.state(), state, "a refused far click changes nothing");
    assert_eq!(saved(&game), bytes, "nor the serialized state");
    let survived = game
        .handle_in_world(&npc_select(npc, talk), &near, NOW + 3)
        .unwrap();
    assert_eq!(
        survived.replies, expected.replies,
        "the open menu survived the refused far clicks"
    );
}

#[test]
fn a_click_works_where_she_stands_and_not_at_her_old_guess() {
    click_works_only_at_the_new_position(MAYOR, MAYOR_OLD_POSITION);
}

/// T506: five Lakeshore merchants and Skye stood at the Cooking Table POI near (88, 42, 1020),
/// 680-930 m from their towns; Dewey stood at the Sanctuary POI, 1.7 km from Snowhill, though
/// the client says he is near the Snowhill town hall (string 414618, wording as shipped).
#[test]
fn lakeshore_merchants_and_dewey_stand_in_their_towns() {
    let regional = regional_rows();
    let position = |id: u64| {
        let row = regional.iter().find(|row| row["npc_id"] == id).unwrap();
        xyz(&row["position"])
    };
    // Lorraine, Grasberg, Kimi, Kristane, Alexander. The brief asked for 150 m, but Sanctuary's
    // own Lakeshore cluster stands 137-193 m from Ty (193.4 m for Grasberg), so this coarse
    // town gate is 200 m; `the_nineteen_stand_at_the_cited_sanctuary_placements` pins the
    // exact positions, so no merchant can drift inside the gate. (R) the 200 m is a lead call.
    for merchant in [
        9_500_000_354_u64,
        9_500_000_344,
        9_500_000_348,
        9_500_000_349,
        9_500_000_339,
    ] {
        assert!(
            distance(position(merchant), position(TY)) < 200.0,
            "merchant {merchant} is not in Lakeshore"
        );
    }
    // Skye is 196.6 m from Brody, inside the bound by 3.4 m.
    for snowhill in [DEWEY, 9_500_000_769] {
        assert!(
            distance(position(snowhill), position(BRODY)) < 200.0,
            "NPC {snowhill} is not in Snowhill"
        );
    }
    click_works_only_at_the_new_position(DEWEY, DEWEY_OLD_POSITION);
}

/// T506: the 19 moved NPCs as `FabledRealms.lua` places them (Sanctuary-main, `spawnNpcWithGuid`
/// position and heading; line numbers in `notes/Autopilot/T506.md`). Pinned here so the data
/// and its regional copy cannot both drift to a wrong spot or facing.
const T506_PLACEMENTS: [(u64, [f32; 3], f64); 19] = [
    (9_500_000_354, [-816.7909, 2.009025, 839.6132], 0.83435801),
    (9_500_000_344, [-784.2302, 5.083301, 896.4583], 1.73828101),
    (9_500_000_348, [-774.88715, 0.230093, 861.7282], 2.34716702),
    (9_500_000_349, [-763.6466, 0.075717, 836.9289], -0.780399958),
    (9_500_000_339, [-759.3783, 0.020418, 859.9176], -1.99780999),
    (
        9_500_000_769,
        [203.77641, 23.083303, 352.81158],
        -0.237981002,
    ),
    (DEWEY, [317.7469, 26.46915, 424.5907], -1.12853004),
    (
        9_500_000_482,
        [-1399.3837, -29.449934, 278.26196],
        -0.901700003,
    ),
    (9_500_000_543, [-1266.9845, -14.225539, 290.2942], -1.237623),
    (
        9_000_000_208,
        [319.51495, 26.05179, 430.15677],
        -0.890770018,
    ),
    (9_500_000_775, [235.03586, 23.664875, 384.34042], 0.56215605),
    (9_500_000_741, [237.1437, 26.234308, 436.1044], 1.38882005),
    (
        9_500_000_763,
        [215.32555, 28.237514, 441.73898],
        -2.74149799,
    ),
    (
        9_500_000_746,
        [230.88478, 25.246117, 430.04623],
        -3.07482004,
    ),
    (9_500_000_772, [228.21536, 23.48963, 385.24973], -1.24267901),
    (9_500_000_771, [233.37015, 23.61321, 387.91855], 1.08215001),
    (
        9_500_000_742,
        [-33.279736, 10.13236, 259.21094],
        -1.07136001,
    ),
    (9_500_000_355, [-782.4307, 5.557206, 892.12964], 0.731347972),
    (9_000_000_155, [261.79507, 48.96282, 211.63281], 0.349949977),
];

#[test]
fn the_nineteen_stand_at_the_cited_sanctuary_placements() {
    let regional = regional_rows();
    let extras = extra_rows();
    for (id, position, heading) in T506_PLACEMENTS {
        let row = regional.iter().find(|row| row["npc_id"] == id).unwrap();
        let body = extras
            .iter()
            .find(|row| row["content_key"].as_u64() == Some(id))
            .unwrap();
        for (label, record) in [("regional", row), ("extras", body)] {
            assert_eq!(xyz(&record["position"]), position, "{id} {label} position");
            assert_eq!(record["heading"].as_f64(), Some(heading), "{id} {label}");
        }
    }
}

/// Each session builds its own visibility from the same content: both draw her identically.
#[test]
fn two_sessions_both_see_her() {
    let position = body_position(MAYOR);
    let mut first = Fixture::for_character(CHARACTER).game(None);
    let mut second = Fixture::for_character(CHARACTER + 1).game(None);
    let adds = |game: &mut osfr_tools::gameplay::Gameplay, guid| {
        game.update_npc_visibility(&character_at_for(guid, 0, position))
            .unwrap()
            .into_iter()
            .find(|packet| {
                packet.starts_with(&[35, 0, 2, 0])
                    && parse_add_npc(packet).is_ok_and(|npc| npc.actor_id == MAYOR)
            })
            .expect("her 35/2")
    };
    assert_eq!(
        adds(&mut first, CHARACTER),
        adds(&mut second, CHARACTER + 1)
    );
}

/// The nearest-64 cap must not push a neighbour out of view: from every content-keyed
/// actor, every other content-keyed actor within 30 m is still in the visible set.
#[test]
fn the_visibility_cap_keeps_neighbours_in_view() {
    let base = NpcVisibility::load(&content_dir()).unwrap();
    let keyed: Vec<(u64, [f32; 3])> = extra_rows()
        .iter()
        .filter(|row| row["content_key"].as_u64().is_some())
        .map(|row| (row["spawn_guid"].as_u64().unwrap(), xyz(&row["position"])))
        .collect();
    assert!(keyed.len() >= 90);
    for (guid, position) in &keyed {
        let visible: BTreeSet<u64> = seen_from(&base, *position)
            .iter()
            .map(|npc| npc.actor_id)
            .collect();
        for (other, other_position) in &keyed {
            let metres = distance(*position, *other_position);
            if metres <= 30.0 {
                assert!(
                    visible.contains(other),
                    "actor {other} within {metres:.1} m of {guid} is past the 64-actor cap"
                );
            }
        }
    }
}

#[test]
fn the_reference_tag_names_the_commit() {
    for (row, body) in moved() {
        for record in [&row["field_sources"]["position"], &body["source"]] {
            assert!(record.as_array().unwrap().iter().any(|tag| {
                tag["kind"] == "reference"
                    && tag["evidence"]
                        .as_str()
                        .is_some_and(|text| text.contains(REF))
            }));
        }
    }
}
