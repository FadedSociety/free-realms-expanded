//! Travel through the real `Gameplay` and `Gateway` with the shipped content: the Atlas table at
//! world entry, finding stones by walking, a teleport to every point of `data/content/travel.json`
//! that lands on walkable ground, the friend teleport's host lend, Teleport to Safety off the open
//! world, and the login that never refuses a pose (the stranding audit of the travel piece).
//!
//! Evidence for each rule is in `src/gameplay/travel.rs` and `src/restoration/piece_077.rs`.
//! Raw inner packets as the client sends them (op 56, 58, 90, 122); no player data.
#[path = "../src/restoration/piece_077.rs"]
#[allow(dead_code)]
mod piece_077;
#[path = "support/terrain.rs"]
mod terrain_support;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{FriendLend, Gameplay, Outcome},
    gateway::Gateway,
};
use piece_077::{PointOfInterestDefinition, decode_point_of_interest_message};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

fn content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn points() -> Vec<Value> {
    let file: Value =
        serde_json::from_slice(&fs::read(content().join("travel.json")).unwrap()).unwrap();
    file["points"].as_array().unwrap().clone()
}

fn vec3(value: &Value) -> [f32; 3] {
    let v: Vec<f32> = (value.as_array().unwrap().iter())
        .map(|c| c.as_f64().unwrap() as f32)
        .collect();
    [v[0], v[1], v[2]]
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position,
        rotation: [0.0, 0.0, 1.0],
    }
}

fn game() -> Gameplay {
    Gameplay::load(&content(), None).unwrap()
}

fn atlas(out: &Outcome) -> Vec<PointOfInterestDefinition> {
    let message = (out.replies.iter())
        .find(|reply| reply.starts_with(&[57, 0]))
        .expect("a 57 destination list");
    decode_point_of_interest_message(message).expect("the strict client framing")
}

fn op90(id: i64) -> Vec<u8> {
    [90u16.to_le_bytes().as_slice(), &(id as i32).to_le_bytes()].concat()
}

fn op58(guid: u64) -> Vec<u8> {
    [58u16.to_le_bytes().as_slice(), &guid.to_le_bytes()].concat()
}

fn text(out: &Outcome) -> String {
    (out.replies.iter())
        .map(|reply| String::from_utf8_lossy(reply).into_owned())
        .collect()
}

const START: [f32; 3] = [-1904.0, -42.0, 420.0];

/// Client 26/8 (`008b6bc0`): the actor guid and a zero flag. Pressing X on an actor sends it.
fn click(guid: u64) -> Vec<u8> {
    [&[26, 0, 8, 0][..], &guid.to_le_bytes(), &[0]].concat()
}

/// BaseUi 47/7 `Atlas.Show` with no ints: the answer Sanctuary's warpstone gives a click.
fn atlas_script() -> Vec<u8> {
    let mut packet = vec![47, 0, 7, 10, 0, 0, 0];
    packet.extend(b"Atlas.Show");
    packet.extend([0, 0, 0, 0]);
    packet
}

/// The shipped `npc_spawns_extra.json` rows of one model.
fn extra_rows(model_id: i64) -> Vec<Value> {
    let file: Value =
        serde_json::from_slice(&fs::read(content().join("npc_spawns_extra.json")).unwrap())
            .unwrap();
    (file["records"].as_array().unwrap().iter())
        .filter(|row| row["model_id"] == model_id)
        .cloned()
        .collect()
}

#[test]
fn world_entry_sends_the_whole_atlas_and_walking_to_a_stone_finds_it() {
    let mut game = game();
    let entry = game.initialize_world(&at(START), 500).unwrap();
    let rows = atlas(&entry);
    // EV: an event marker (Halloween Harry's, row 140) stays closed until the session's calendar
    // runs its event; with no quest catalogue bound here, no calendar runs (`event_assets`).
    let year_round = points().iter().filter(|p| p.get("live_event").is_none()).count();
    assert_eq!(rows.len(), year_round);
    assert!(
        rows.iter().all(|row| row.notification_type != 7),
        "a fresh character found none"
    );
    assert_eq!(
        rows.iter().filter(|row| row.notification_type == 8).count(),
        18
    );
    for atlas_name in ["sacredgrove", "sunstone"] {
        assert!(
            rows.iter().any(|row| row.atlas_name == atlas_name),
            "{atlas_name}"
        );
    }
    // The client's own scripts teleport to these ids, so they must be in its table.
    for id in [14, 16, 17, 153] {
        assert!(rows.iter().any(|row| row.id == id), "row {id}");
    }
    // Twenty metres from the Sanctuary stone's row (the bodies stand up to 17 m from their rows).
    let near = [-1414.636 + 20.0, -27.631, 351.567];
    let found = game.on_location(&at(near), 1_000).unwrap();
    assert!(found.changed, "a found stone is saved");
    assert!(text(&found).contains("attuned to the Sanctuary Warpstone"));
    let rows = atlas(&found);
    assert_eq!(
        rows.iter()
            .find(|row| row.id == 5)
            .unwrap()
            .notification_type,
        7
    );
    assert_eq!(
        rows.iter().filter(|row| row.notification_type == 8).count(),
        17
    );
    // It survives a relog, and 56 answers with the same list.
    let mut relogged = Gameplay::load(&content(), Some(game.state())).unwrap();
    let again = relogged
        .handle_in_world(&[56, 0], &at(START), 2_000)
        .unwrap();
    assert_eq!(atlas(&again), rows);
}

/// The stranding audit for arrivals: every point, reached by the real request, puts the
/// character on the terrain or, for the eight cave mouths, at the point's own floor.
#[test]
fn every_point_is_a_teleport_that_lands_on_walkable_ground() {
    let terrain = terrain_support::load();
    let grid = terrain.grid(0).expect("zone 0 heights");
    let mut game = game();
    game.set_terrain(Some(Arc::new(terrain_support::load())));
    let points = points();
    let mut now = 1_000;
    for point in points.iter().filter(|p| p["notification_type"] == 7) {
        let here = vec3(&point["position"]);
        game.on_location(&at(here), now).unwrap();
        now += 1;
    }
    let (mut on_ground, mut interior) = (0, 0);
    // EV: an event marker is closed here (no calendar runs); `gameplay/travel.rs` tests it.
    for point in points.iter().filter(|p| p.get("live_event").is_none()) {
        let (id, name) = (
            point["id"].as_i64().unwrap(),
            point["name"].as_str().unwrap(),
        );
        now += 6_000;
        let out = game.handle_in_world(&op90(id), &at(START), now).unwrap();
        // The Atlas offers no teleport for types 8, 10 and 11 (Precursor Ruins), nor does the server.
        if [8, 10, 11]
            .iter()
            .any(|kind| point["notification_type"] == *kind)
        {
            assert!(out.relocation.is_none(), "{id} {name}");
            continue;
        }
        let to = (out.relocation.clone()).unwrap_or_else(|| panic!("{id} {name}: {}", text(&out)));
        let spawn = vec3(point.get("spawn").unwrap_or(&point["position"]));
        assert_eq!(
            [to.position[0], to.position[2]],
            [spawn[0], spawn[2]],
            "{id} {name}"
        );
        assert!(
            to.position
                .iter()
                .chain(&to.rotation)
                .all(|v| v.is_finite()),
            "{id} {name}"
        );
        match grid.height_at(spawn[0], spawn[2]) {
            // Deeper than 3 m under the highest surface: a cave mouth, kept at its own floor.
            Some(ground) if ground - spawn[1] > 3.0 => {
                assert_eq!(to.position[1], spawn[1], "{id} {name}");
                assert_eq!(
                    point["notification_type"], 3,
                    "{id} {name}: only dungeon gates"
                );
                interior += 1;
            }
            Some(ground) => {
                let above = to.position[1] - ground;
                assert!(
                    (0.0..=10.0).contains(&above),
                    "{id} {name}: {above} m over the ground"
                );
                on_ground += 1;
            }
            // No terrain under the point: its own height.
            None => {
                assert_eq!(to.position[1], spawn[1], "{id} {name}");
                interior += 1;
            }
        }
        if point["notification_type"] == 7 {
            let over = to.position[1] - grid.height_at(spawn[0], spawn[2]).unwrap();
            assert!(
                (0.0..=3.0).contains(&over),
                "{id} {name}: a stone stands on the ground"
            );
        }
    }
    // EV: 127 on the ground with Halloween Harry's event marker (row 140) closed here.
    assert_eq!(
        (on_ground, interior),
        (127, 8),
        "measured against the zone 0 grid"
    );
}

#[test]
fn a_stone_not_found_yet_refuses_and_nothing_forged_moves_anyone() {
    let mut game = game();
    for (request, why) in [
        (op90(16), "Blackspore, not found"),
        (
            op90(153),
            "the comic alias, before the Sunstone Valley stone is found",
        ),
        (op90(147), "Precursor Ruins: no teleport on the Atlas"),
        (op90(99_999), "no such point"),
        (op58(7), "no host lend"),
        (op58(1), "yourself"),
    ] {
        let out = game.handle_in_world(&request, &at(START), 5_000).unwrap();
        assert!(
            out.relocation.is_none() && out.scene_zoning.is_none(),
            "{why}"
        );
        assert!(text(&out).contains("not attuned") || text(&out).contains("cannot teleport"));
    }
    assert!(
        game.handle_in_world(&[90, 0, 1], &at(START), 6_000)
            .is_err(),
        "a short body"
    );
}

/// Bug of 2026-11-10: X on a warpstone got no answer, because the click router dropped every
/// actor without a content key. Every stone row now opens the Atlas, and a row picked on it
/// travels through the same op 90.
#[test]
fn pressing_x_on_any_warpstone_opens_the_atlas_and_a_found_row_then_teleports() {
    let stones = extra_rows(280);
    assert_eq!(
        stones.len(),
        18,
        "the warpstone actors (sg_warpstone_01.adr)"
    );
    let mut game = game();
    let mut now = 1_000;
    for stone in &stones {
        let name = stone["name"].as_str().unwrap();
        assert!(
            stone["content_key"].is_null(),
            "{name}: no content key to route by"
        );
        let guid = stone["spawn_guid"].as_u64().unwrap();
        let spot = vec3(&stone["position"]);
        now += 1_000;
        let out = game
            .handle_in_world(&click(guid), &at([spot[0] + 3.0, spot[1], spot[2]]), now)
            .unwrap();
        assert_eq!(out.replies, [atlas_script()], "{name}");
        assert!(!out.changed && out.relocation.is_none(), "{name}");
        // Beyond the 12 m NPC reach, or off the open world: no answer, as any NPC click.
        let mut away = at(spot);
        away.zone_id = 77;
        for refused in [at([spot[0] + 13.0, spot[1], spot[2]]), away] {
            let out = game.handle_in_world(&click(guid), &refused, now).unwrap();
            assert!(out.replies.is_empty() && out.relocation.is_none(), "{name}");
        }
    }
    // The opened map's "Teleport Now": Snowhill (row 3) was found by walking, so the Sanctuary
    // stone's Atlas travels there by op 90.
    game.on_location(&at([51.391, 30.987, 363.156]), now + 1)
        .unwrap();
    let beside = at([-1415.3, -27.0, 347.8]);
    let out = game
        .handle_in_world(&click(100000033167), &beside, now + 2)
        .unwrap();
    assert_eq!(out.replies, [atlas_script()]);
    let out = game.handle_in_world(&op90(3), &beside, now + 3).unwrap();
    let to = out.relocation.expect("a found row teleports");
    assert_eq!([to.position[0], to.position[2]], [51.391, 363.156]);
}

#[test]
fn a_keyless_actor_that_is_not_a_warpstone_is_still_dropped() {
    let mut game = game();
    let squirrel = &extra_rows(402)[0];
    assert!(squirrel["content_key"].is_null());
    let guid = squirrel["spawn_guid"].as_u64().unwrap();
    assert_eq!(game.npc_model(guid), Some(402), "a drawn, passive actor");
    let spot = vec3(&squirrel["position"]);
    let out = game
        .handle_in_world(&click(guid), &at([spot[0] + 3.0, spot[1], spot[2]]), 5_000)
        .unwrap();
    assert!(out.replies.is_empty() && out.relocation.is_none() && !out.changed);
}

#[test]
fn friend_teleport_moves_only_with_the_hosts_lend_for_that_guid() {
    let friend = [-1400.0, -27.0, 360.0];
    let mut game = game();
    // The host found the friend free in the open world.
    game.lend_friend(Some(FriendLend::Go {
        guid: 7,
        position: friend,
    }));
    let out = game.handle_in_world(&op58(7), &at(START), 10_000).unwrap();
    assert_eq!(out.relocation.expect("joins the friend").position, friend);
    // The lend is spent: the same request again is refused.
    let out = game.handle_in_world(&op58(7), &at(START), 20_000).unwrap();
    assert!(out.relocation.is_none(), "one lend, one teleport");
    // A lend for one guid never serves another.
    game.lend_friend(Some(FriendLend::Go {
        guid: 7,
        position: friend,
    }));
    let out = game.handle_in_world(&op58(8), &at(START), 30_000).unwrap();
    assert!(out.relocation.is_none());
    // A refusal is the host's own 74/8, sent alone; nobody moves.
    let packet = vec![74, 0, 8, 0, 11];
    game.lend_friend(Some(FriendLend::Refuse {
        guid: 7,
        packet: Some(packet.clone()),
    }));
    let out = game.handle_in_world(&op58(7), &at(START), 40_000).unwrap();
    assert!(out.relocation.is_none());
    assert_eq!(out.replies, [packet]);
    // From another zone (a house, an arena) the requester is refused, lend or not.
    game.lend_friend(Some(FriendLend::Go {
        guid: 7,
        position: friend,
    }));
    let mut away = at(START);
    away.zone_id = 77;
    let out = game.handle_in_world(&op58(7), &away, 50_000).unwrap();
    assert!(
        out.relocation.is_none() && text(&out).contains("open world"),
        "{}",
        text(&out)
    );
}

#[test]
fn teleport_to_safety_goes_to_a_found_stone_and_names_the_way_out_of_a_minigame() {
    let mut game = game();
    // Found: Snowhill only. Asked from beside the unfound Blackspore stone.
    game.on_location(&at([51.391, 30.987, 363.156]), 1_000)
        .unwrap();
    let out = game
        .handle_in_world(&[122, 0], &at([-2140.0, -32.9, 1040.0]), 10_000)
        .unwrap();
    let to = out.relocation.expect("the open world always has a way out");
    assert_eq!([to.position[0], to.position[2]], [51.391, 363.156]);
    // Off the open world with no house and no battle instance: the client's own wording.
    let mut scene = at(START);
    scene.zone_id = 77;
    let out = game.handle_in_world(&[122, 0], &scene, 400_000).unwrap();
    assert!(out.relocation.is_none() && out.scene_zoning.is_none());
    assert!(
        text(&out).contains("Exit the current minigame"),
        "{}",
        text(&out)
    );
}

/// The one way a player was stranded for good: a save in a zone nothing claims refused the
/// login. It now loads at the reference's open-world start.
#[test]
fn a_save_in_a_zone_nothing_claims_still_logs_in() {
    let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        });
    let start = Gateway::reference_state(&reference).expect("the private gateway reference");
    let mut lost = start.clone();
    lost.guid = 42;
    lost.zone_id = 4_242;
    lost.position = [9.0, 9.0, 9.0];
    let mut gateway = Gateway::load_gameplay(&reference, lost, &content(), None)
        .expect("a pose never refuses a login");
    // The host's L039 block sees the rescue, so it saves and logs it at once.
    assert_eq!(
        gateway.recover_login_pose().map(|(_, why)| why),
        Some("unclaimed_zone")
    );
    let state = gateway.state();
    assert_eq!((state.guid, state.zone_id), (42, start.zone_id));
    assert_eq!(state.position, start.position);
    // A pose in the open world is left alone.
    let mut fine = start.clone();
    fine.position[0] += 3.0;
    let mut gateway = Gateway::load_gameplay(&reference, fine.clone(), &content(), None).unwrap();
    assert_eq!(gateway.recover_login_pose(), None);
    assert_eq!(gateway.state(), &fine);
}
