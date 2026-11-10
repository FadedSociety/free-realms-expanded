//! The floating rock over every warpstone (owner report 2026-11-09: no stone showed it).
//!
//! The rock is the stone's own actor (model 280, `sg_warpstone_01.adr`); the dais under it is
//! part of the zone. Cause, seen natively on 2026-11-09: nine of the fourteen stone rows of
//! `npc_spawns_extra.json` were marked as "already drawn by the login stream" and skipped by
//! the world table, while the login stream leaves its NPC rows (startup messages 10 to 32) out
//! whenever the opening campaign is loaded, which is every release. Those nine stones were
//! never sent. A stone the world table sends (Sanctuary, Snowhill) drew its rock; the dais at
//! Cobblestone Village stood bare, also when logging in beside it.
//!
//! These tests fail if that comes back: every stone is sent, once, by the world table, after
//! the world has loaded, with the fields the original server sent (2014-03 captures), followed
//! by its minimap row (35/10) and its use cursor (35/12, cursor 17).

#[path = "common/content_copy.rs"]
mod content_copy;
#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;
#[path = "support/json.rs"]
#[allow(dead_code)]
mod json;
#[path = "support/npc_visibility.rs"]
mod npc_visibility;

use gateway_packets::{login_with_version, wrap};
use npc_visibility::content_dir;
use osfr_tools::{
    gateway::Gateway,
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const STONE_MODEL: i32 = 280;
/// The stones that are actors: the other four rocks (Blackspore, Stillwater Crossing, Rumble
/// Ridge, Sunstone Valley) are part of the zone and need none to be seen.
const STONES: usize = 14;
/// The dais is about 3 m across; nobody may stand on it or inside the rock.
const DAIS_CLEAR_M: f32 = 2.5;

fn stone_rows() -> Vec<serde_json::Value> {
    let extra = json::read(&content_dir().join("npc_spawns_extra.json"));
    extra["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["model_id"] == STONE_MODEL)
        .cloned()
        .collect()
}

fn spot(row: &serde_json::Value) -> [f32; 3] {
    let at = |index: usize| row["position"][index].as_f64().unwrap() as f32;
    [at(0), at(1), at(2)]
}

fn level_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[2] - b[2])
}

/// The unframed world packets a fresh viewer standing at `position` is sent.
fn view(table: &NpcVisibility, position: [f32; 3]) -> Vec<Vec<u8>> {
    table.clone().update(0, position).unwrap()
}

fn stones_in(packets: &[Vec<u8>]) -> Vec<(usize, NpcSpawn)> {
    packets
        .iter()
        .enumerate()
        .filter(|(_, packet)| packet.starts_with(&[35, 0, 2, 0]))
        .map(|(index, packet)| (index, parse_add_npc(packet).unwrap()))
        .filter(|(_, npc)| npc.model_id == STONE_MODEL)
        .collect()
}

#[test]
fn every_stone_is_sent_once_by_the_world_table_with_the_fields_the_client_draws() {
    let rows = stone_rows();
    assert_eq!(rows.len(), STONES, "the stone actors of npc_spawns_extra.json");
    let table = NpcVisibility::load(&content_dir()).unwrap();
    for row in &rows {
        let name = row["name"].as_str().unwrap();
        let here = spot(row);
        let packets = view(&table, here);
        let near: Vec<_> = stones_in(&packets)
            .into_iter()
            .filter(|(_, npc)| {
                level_distance([npc.position[0], npc.position[1], npc.position[2]], here) < 30.0
            })
            .collect();
        assert_eq!(near.len(), 1, "{name}: one rock, not none and not two");
        let (index, npc) = &near[0];
        // The original server's stone packets (Sanctuary, Lakeshore, Snowhill, Lavender Coast):
        // a plain interactable actor with no terrain link, at scale 1, in its stand animation.
        assert_eq!(npc.actor_id, row["spawn_guid"].as_u64().unwrap(), "{name}");
        assert_eq!(npc.name_string_id as i64, row["name_string_id"].as_i64().unwrap(), "{name}");
        assert_eq!(
            (npc.scale, npc.animation_id, npc.terrain_object_id, npc.is_interactable),
            (1.0, 1, 0, 1),
            "{name}"
        );
        assert_eq!(
            (npc.composite_effect_id, npc.hide_nameplate, npc.temporary_appearance),
            (0, 0, 0),
            "{name}"
        );
        assert_eq!(npc.position[3], 1.0, "{name}");
        // Then its minimap row and its hover cursor, both naming the actor just spawned (the
        // client skips a 35/10 or 35/12 guid it has not spawned).
        let guid = npc.actor_id.to_le_bytes();
        let after = |subtype: u8| {
            packets[index + 1..]
                .iter()
                .take_while(|packet| !packet.starts_with(&[35, 0, 2, 0]))
                .find(|packet| packet.starts_with(&[35, 0, subtype, 0]))
                .unwrap_or_else(|| panic!("{name}: no 35/{subtype} after its AddNpc"))
                .clone()
        };
        let notification = after(10);
        assert_eq!(&notification[8..16], &guid, "{name}: 35/10 guid");
        let cursor = after(12);
        // i32 count, u64 guid, has-cursor, cursor id 17 (use), flag 0: the captured entry.
        assert_eq!(&cursor[4..8], &1i32.to_le_bytes(), "{name}");
        assert_eq!(&cursor[8..16], &guid, "{name}: 35/12 guid");
        assert_eq!(&cursor[16..], &[1, 17, 0], "{name}: the use cursor");
    }
}

#[test]
fn no_stone_row_waits_for_the_login_stream() {
    // The login stream's NPC rows are not sent while the opening campaign is loaded, so a row
    // left to it is a stone nobody sees. `capture_match` is the loader's "someone else draws
    // this" mark; no stone may carry it.
    for row in stone_rows() {
        assert!(
            row["capture_match"].is_null(),
            "{} is left to the login stream",
            row["name"]
        );
    }
    // And every login-stream stone is one the world table draws, so it is never sent twice.
    let table = NpcVisibility::load(&content_dir()).unwrap();
    let snapshot = json::read(&content_dir().join("startup/snapshot.json"));
    let login_stones: Vec<_> = snapshot["world"]["npc_spawns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|npc| npc["model_id"] == STONE_MODEL)
        .collect();
    assert_eq!(login_stones.len(), STONES);
    for npc in login_stones {
        let at = |index: usize| npc["position"][index].as_f64().unwrap() as f32;
        assert!(
            table.draws_model_near(STONE_MODEL, [at(0), at(1), at(2)], 2.0),
            "login-stream stone {} has no world-table row",
            npc["name_string_id"]
        );
    }
}

#[test]
fn nobody_stands_on_a_stone() {
    let rows = stone_rows();
    let table = NpcVisibility::load(&content_dir()).unwrap();
    // The four rocks that are part of the zone (the dais placements of the 2014 zone chunks).
    let baked = [
        [-2128.74, -32.96, 1045.24],
        [-1270.03, -49.75, 785.18],
        [501.07, 69.84, 1779.16],
        [901.90, 2.05, 2235.69],
    ];
    let spots = rows.iter().map(spot).chain(baked);
    for stone in spots {
        for packet in view(&table, stone) {
            if !packet.starts_with(&[35, 0, 2, 0]) {
                continue;
            }
            let npc = parse_add_npc(&packet).unwrap();
            let at = [npc.position[0], npc.position[1], npc.position[2]];
            assert!(
                npc.model_id == STONE_MODEL || level_distance(at, stone) >= DAIS_CLEAR_M,
                "actor {} (name id {}) stands {:.2} m from the stone at {stone:?}",
                npc.actor_id,
                npc.name_string_id,
                level_distance(at, stone)
            );
        }
    }
}

fn release_content(name: &str, opening: bool) -> PathBuf {
    let root = std::env::temp_dir().join(format!("osfr-stone-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    content_copy::copy_tree(&content_dir(), &root);
    content_copy::copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release"),
        &root,
    );
    if !opening {
        fs::remove_file(root.join("opening.json")).unwrap();
    }
    root
}

/// Model-280 AddNpc messages among framed gateway replies: `(index, guid, position)`.
fn framed_stones(replies: &[Vec<u8>]) -> Vec<(usize, u64, [f32; 3])> {
    replies
        .iter()
        .enumerate()
        .filter(|(_, reply)| reply.get(7..11) == Some(&[35, 0, 2, 0][..]))
        .map(|(index, reply)| (index, parse_add_npc(&reply[7..]).unwrap()))
        .filter(|(_, npc)| npc.model_id == STONE_MODEL)
        .map(|(index, npc)| {
            (
                index,
                npc.actor_id,
                [npc.position[0], npc.position[1], npc.position[2]],
            )
        })
        .collect()
}

/// The real entry path: login, the client's ready signal, then its loaded signal, standing
/// 7 m from a stone. Returns the stones sent before the world loaded and after it.
fn enter_beside(content: &Path, stone: [f32; 3]) -> (Vec<u64>, Vec<u64>) {
    let mut state = Gateway::content_state(content).unwrap();
    state.zone_id = 0;
    state.position = [stone[0] + 6.0, stone[1], stone[2] - 3.0];
    let version = json::read(&content.join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut gateway = Gateway::load_gameplay_from_content(state.clone(), content, None).unwrap();
    let mut early = gateway
        .handle(&login_with_version(state.guid, &version))
        .unwrap()
        .replies;
    early.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    let world = gateway.handle(&wrap(&[10, 0])).unwrap();
    assert!(world.entered_world);
    let near = |found: Vec<(usize, u64, [f32; 3])>| {
        found
            .into_iter()
            .filter(|(_, _, at)| level_distance(*at, stone) < 30.0)
            .map(|(_, guid, _)| guid)
            .collect::<Vec<_>>()
    };
    (near(framed_stones(&early)), near(framed_stones(&world.replies)))
}

#[test]
fn entering_the_world_beside_a_stone_sends_its_rock_once_after_the_world_loads() {
    let rows = stone_rows();
    // A stone the login stream alone used to own (never drawn), one both paths used to send,
    // and one whose two recovered placements are 0.75 m apart.
    for (opening, name) in [
        (false, "Cobblestone Village Warpstone"),
        (false, "Sanctuary Warpstone"),
        (false, "Wugachug Warpstone"),
    ] {
        let row = rows.iter().find(|row| row["name"] == name).unwrap();
        let content = release_content(&name.replace(' ', "-"), opening);
        let (early, loaded) = enter_beside(&content, spot(row));
        let _ = fs::remove_dir_all(&content);
        assert!(early.is_empty(), "{name}: sent before the world loaded: {early:?}");
        assert_eq!(
            loaded,
            vec![row["spawn_guid"].as_u64().unwrap()],
            "{name}: the stone's actor, once"
        );
    }
}

#[test]
fn the_release_with_its_opening_sends_no_stone_in_the_login_stream() {
    // With the opening campaign loaded the login stream carries no NPC rows at all; that is
    // why a stone must not depend on it.
    let content = release_content("opening", true);
    let state = Gateway::content_state(&content).unwrap();
    let version = json::read(&content.join("startup/manifest.json"))["client_version"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut gateway = Gateway::load_gameplay_from_content(state.clone(), &content, None).unwrap();
    assert!(gateway.gameplay().unwrap().is_opening());
    let mut early = gateway
        .handle(&login_with_version(state.guid, &version))
        .unwrap()
        .replies;
    early.extend(gateway.handle(&wrap(&[13, 0])).unwrap().replies);
    let _ = fs::remove_dir_all(&content);
    assert!(framed_stones(&early).is_empty());
}
