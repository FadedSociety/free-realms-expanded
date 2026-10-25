//! T514: Go fish! at a hole whose scene the client has (`client_scene_cached`) zones the player
//! into that scene, the round's start packets follow the scene load, and every end of the round
//! zones the player back to the world pose kept at Go fish!, once. Driven through the real NPC
//! menu (26/8, then the 26/10 Go fish! choice) on `Gameplay`, and through `Gateway` for the
//! opcode-31 frames, the save and the reconnect.
//!
//! Client evidence: 26/8 constructor `0x008b6bc0` and 26/10 writer `0x00a36f80` (see
//! `fisherman_entry_chain.rs`); opcode-31 reader `0x008e4ae0` (`src/zone_transition.rs`);
//! family-39 lifecycle `0x009bdc60` and the 138 shapes in `support/fishing_packets.rs`.

#[path = "support/fishing_packets.rs"]
mod fishing_packets;
#[path = "../src/bin/osfr_gateway/fishing_fixtures.rs"]
mod fixtures;

use fishing_packets::{cast_at, lifecycle, register};
use fixtures::{copy_tree, fish_option, npc, wrap};
use osfr_tools::{
    character_store::CharacterState,
    fishing::Handle,
    fishing_service::FishingSpots,
    gameplay::{Gameplay, Outcome},
    gateway::Gateway,
    housing_native::InstanceList,
    zone_transition::{self, ZoneTransition},
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GUID: u64 = 1;
/// Selvi Slowreel: Rainbow Lake (spot 598, hole 562, scene cached), drawn since T385.
const SELVI: u64 = 9_000_000_194;
/// Reed Stillwater: Sacred Grove Shallows (hole 563, scene not cached).
const REED: u64 = 9_000_000_181;
/// Lila Darklure: Darklit Lagoon (hole 560, scene not cached), drawable.
const LILA: u64 = 9_000_000_377;
const HOLE: u32 = 562;
const SPAWN: [f32; 3] = [441.42, -60.31, 475.08];
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
static NEXT: AtomicU64 = AtomicU64::new(0);

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The opening reference content (the world to come back to) plus the shipped spots file.
struct Content(PathBuf);

impl Content {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-fishing-scene-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_tree(
            &manifest().join("../../../lab/references/rust-opening-cooking-original-assets"),
            &dir,
        );
        fs::copy(
            manifest().join("data/content/fishing_spots.json"),
            dir.join("fishing_spots.json"),
        )
        .unwrap();
        Self(dir)
    }

    /// The same content with the shipped `houses.json`, so 127/19 reaches the housing glue.
    fn with_houses() -> Self {
        let content = Self::new();
        fs::copy(
            manifest().join("data/content/houses.json"),
            content.0.join("houses.json"),
        )
        .unwrap();
        content
    }

    fn game(&self) -> Gameplay {
        Gameplay::load(&self.0, None).unwrap()
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn menu_label() -> u32 {
    read_json(&manifest().join("data/content/fishing_spots.json"))["menu_string_id"]
        .as_u64()
        .unwrap() as u32
}

/// A guide's content position (`data/content/npcs`).
fn guide_position(guide: u64) -> [f32; 3] {
    for entry in fs::read_dir(manifest().join("data/content/npcs")).unwrap() {
        let document = read_json(&entry.unwrap().path());
        let rows = document["records"].as_array().unwrap();
        if let Some(row) = rows
            .iter()
            .find(|row| row["npc_id"].as_u64() == Some(guide))
        {
            return serde_json::from_value(row["position"].clone()).unwrap();
        }
    }
    panic!("guide {guide} has a content position");
}

/// The player in the open world (zone 0), beside a guide.
fn at(guide: u64) -> CharacterState {
    CharacterState {
        guid: GUID,
        zone_id: 0,
        position: guide_position(guide),
        rotation: [0.0, 0.0, 1.0],
    }
}

/// The player inside the hole, as the gateway stages it after the zoning.
fn in_hole(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: GUID,
        zone_id: HOLE,
        position,
        rotation: [0.0, 0.0, 0.0],
    }
}

fn starts(replies: &[Vec<u8>], prefix: &[u8]) -> bool {
    replies.iter().any(|reply| reply.starts_with(prefix))
}

fn status_flags(replies: &[Vec<u8>]) -> Vec<u32> {
    replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 20, 0]))
        .map(|p| u32::from_le_bytes(p[12..16].try_into().unwrap()))
        .collect()
}

/// The 39/16 offer's instance id (family-39 header: instance, -1, game).
fn offer_instance(replies: &[Vec<u8>]) -> i32 {
    let offer = replies
        .iter()
        .find(|reply| reply.starts_with(&[39, 0, 16]))
        .expect("a 39/16 offer");
    i32::from_le_bytes(offer[3..7].try_into().unwrap())
}

/// 26/8 at the guide, then its Go fish! choice (26/10), both through `handle_in_world`.
fn go_fish(game: &mut Gameplay, guide: u64, character: &CharacterState, now: u64) -> Outcome {
    let menu = game
        .handle_in_world(&npc(8, guide, None), character, now)
        .unwrap();
    let option = fish_option(&menu.replies, menu_label()).expect("the guide offers Go fish!");
    game.handle_in_world(&npc(10, guide, Some(option)), character, now + 1)
        .unwrap()
}

/// The hole scene, its start packets after the load, and the Go that starts the round.
fn enter_selvi(game: &mut Gameplay) -> (ZoneTransition, i32) {
    let out = go_fish(game, SELVI, &at(SELVI), 1_000);
    let scene = out.scene_zoning.expect("a cached hole zones the player in");
    let loaded = game.initialize_world(&in_hole(SPAWN), 2_000).unwrap();
    (scene, offer_instance(&loaded.replies))
}

fn start_selvi(game: &mut Gameplay) -> (ZoneTransition, i32) {
    let (scene, instance) = enter_selvi(game);
    let go = game
        .handle_in_world(&lifecycle(5, instance), &in_hole(SPAWN), 3_000)
        .unwrap();
    assert_eq!(
        status_flags(&go.replies),
        vec![0x40_0001],
        "fishing flag on"
    );
    let registered = game
        .handle_in_world(&register(ACTOR), &in_hole(SPAWN), 3_100)
        .unwrap();
    assert!(starts(&registered.replies, &[138, 0, 3, 0]), "138/3");
    (scene, instance)
}

/// The world return: the world scene's name and geometry, the kept zone id and pose.
fn assert_world_return(scene: &ZoneTransition, world: &osfr_tools::encounter::ZoneTransition) {
    let kept = at(SELVI);
    assert_eq!(scene.scene, world.name);
    assert_eq!(scene.geometry_id, world.geometry_id);
    assert_eq!(scene.id, kept.zone_id as i32);
    assert_eq!(&scene.position[..3], &kept.position);
    assert_eq!(&scene.rotation[..3], &kept.rotation);
}

#[test]
fn go_fish_at_a_cached_hole_zones_in_starts_after_the_load_and_comes_back_once() {
    let content = Content::new();
    let mut game = content.game();
    let world = game.current_zone().expect("the world scene");
    let out = go_fish(&mut game, SELVI, &at(SELVI), 1_000);
    assert!(out.changed);
    assert_eq!(
        out.replies,
        vec![vec![26, 0, 4, 0]],
        "only the dialog close goes now"
    );
    let scene = out.scene_zoning.expect("a cached hole zones the player in");
    assert_eq!(
        scene,
        ZoneTransition {
            envelope_flag: 1,
            scene: "sg_fishing_stream".into(),
            type_id: 2,
            position: [SPAWN[0], SPAWN[1], SPAWN[2], 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            unknown: 1,
            id: HOLE as i32,
            geometry_id: 214,
            override_update_radius: true,
            ..ZoneTransition::default()
        }
    );
    // The hole is the player's scene now: the startup zone record and membership use it.
    let zone = game.current_zone().unwrap();
    assert_eq!(
        (zone.zone_id, zone.name.as_str()),
        (HOLE, "sg_fishing_stream")
    );
    assert_eq!((zone.position, zone.geometry_id), (SPAWN, 214));
    assert!(game.supports_location(&in_hole(SPAWN)));
    assert_eq!(game.scene_return(), Some(at(SELVI)));

    // The start packets wait for the hole to load, and come only once.
    let loaded = game.initialize_world(&in_hole(SPAWN), 2_000).unwrap();
    assert!(
        loaded.replies[0].starts_with(&[39, 0, 16]),
        "the offer first"
    );
    assert!(
        loaded.replies.iter().all(|r| r[0] == 39 || r[0] == 41),
        "only the start packets, no world entry"
    );
    let game_id = i32::from_le_bytes(loaded.replies[0][11..15].try_into().unwrap());
    assert_eq!(game_id, 598, "Rainbow Lake");
    let again = game.initialize_world(&in_hole(SPAWN), 2_500).unwrap();
    assert!(
        again.replies.is_empty(),
        "a second load in the hole resends nothing"
    );

    let instance = offer_instance(&loaded.replies);
    let go = game
        .handle_in_world(&lifecycle(5, instance), &in_hole(SPAWN), 3_000)
        .unwrap();
    assert!(starts(&go.replies, &[41, 0, 2, 0]));
    assert_eq!(status_flags(&go.replies), vec![0x40_0001]);
    assert!(go.scene_zoning.is_none(), "Go stays in the hole");

    // 39/6 ends the round and brings the player back to the kept world pose.
    let quit = game
        .handle_in_world(&lifecycle(6, instance), &in_hole(SPAWN), 4_000)
        .unwrap();
    assert_eq!(status_flags(&quit.replies), vec![1], "fishing flag off");
    assert_world_return(&quit.scene_zoning.expect("the cancel zones back"), &world);
    assert_eq!(game.scene_return(), None);
    assert_eq!(game.current_zone(), Some(world));
    // Once: a repeated cancel and the next tick zone nothing.
    let repeat = game.handle_in_world(&lifecycle(6, instance), &at(SELVI), 4_100);
    assert!(repeat.map_or(true, |out| out.scene_zoning.is_none()));
    let tick = game.tick(&at(SELVI), 4_200).unwrap();
    assert!(tick.scene_zoning.is_none());
    let world_entry = game.initialize_world(&at(SELVI), 5_000).unwrap();
    assert!(
        !starts(&world_entry.replies, &[39, 0, 16]),
        "no held offer left"
    );
}

#[test]
fn the_hole_round_has_no_leash_casts_from_the_player_and_ends_with_its_scene() {
    let content = Content::new();
    let mut game = content.game();
    let world = game.current_zone().unwrap();
    start_selvi(&mut game);
    // 15 m along the bank, then a 10 m cast: 25 m from the spawn, past the 20 m reach, so it
    // is accepted only because the reach is measured from where the player stands (R).
    let walked = [SPAWN[0] + 15.0, SPAWN[1], SPAWN[2]];
    let tick = game.tick(&in_hole(walked), 4_000).unwrap();
    assert!(tick.scene_zoning.is_none() && status_flags(&tick.replies).is_empty());
    let cast = game
        .handle_in_world(&cast_at(ACTOR, walked, 10.0), &in_hole(walked), 4_100)
        .unwrap();
    assert!(starts(&cast.replies, &[138, 0, 8, 0]), "the bobber lands");
    // 65 m from the last supervised position is past the in-place leash (50 m); the hole
    // round keeps going.
    let far = [SPAWN[0] + 80.0, SPAWN[1], SPAWN[2]];
    let tick = game.tick(&in_hole(far), 4_200).unwrap();
    assert!(tick.scene_zoning.is_none() && status_flags(&tick.replies).is_empty());
    assert!(game.scene_return().is_some());
    // Leaving the hole's zone is a server end: the round ends and the player comes back.
    let gone = game.tick(&at(SELVI), 4_300).unwrap();
    assert_eq!(status_flags(&gone.replies), vec![1]);
    assert_world_return(&gone.scene_zoning.expect("a server end zones back"), &world);
    assert!(game.tick(&at(SELVI), 4_400).unwrap().scene_zoning.is_none());
}

#[test]
fn every_end_of_a_hole_round_zones_back_once() {
    // 26/42 cancels the offer before Go; 41/109 closes the running encounter.
    for started in [false, true] {
        let content = Content::new();
        let mut game = content.game();
        let world = game.current_zone().unwrap();
        let packet = if started {
            fishing_packets::encounter_close(start_selvi(&mut game).1)
        } else {
            enter_selvi(&mut game);
            vec![26, 0, 42, 0]
        };
        let out = game
            .handle_in_world(&packet, &in_hole(SPAWN), 5_000)
            .unwrap();
        assert_world_return(&out.scene_zoning.expect("the end zones back"), &world);
        assert_eq!(game.scene_return(), None);
        // The round and its visit are gone, so a disconnect after the cancel returns nobody.
        assert_eq!(game.abandon_scene_for_reconnect(), None);
        let after = game.tick(&at(SELVI), 5_100).unwrap();
        assert!(after.scene_zoning.is_none(), "once");
    }
}

#[test]
fn saves_and_reconnects_in_a_hole_keep_the_world_pose_and_drop_the_round() {
    let content = Content::new();
    let mut game = content.game();
    let world = game.current_zone().unwrap();
    assert_eq!(
        game.scene_return(),
        None,
        "outside, saves use the live pose"
    );
    start_selvi(&mut game);
    assert_eq!(
        game.scene_return(),
        Some(at(SELVI)),
        "a save stores the world pose"
    );
    assert_eq!(game.abandon_scene_for_reconnect(), Some(at(SELVI)));
    game.reset_session();
    assert_eq!(game.scene_return(), None);
    assert_eq!(game.current_zone(), Some(world));
    // The round went with the session: the guide offers a fresh trip.
    let again = go_fish(&mut game, SELVI, &at(SELVI), 9_000);
    assert!(again.scene_zoning.is_some(), "not 'already fishing'");
}

#[test]
fn a_hole_zone_id_must_fit_the_wire_id() {
    // The zone id becomes the opcode-31 i32 `id`, so the load refuses one past i32::MAX.
    let shipped = read_json(&manifest().join("data/content/fishing_spots.json"));
    for (zone, loads) in [(i64::from(i32::MAX), true), (1 << 31, false)] {
        let mut spots = shipped.clone();
        spots["spots"][0]["scene"]["zone_id"] = zone.into();
        let parsed = FishingSpots::parse(&serde_json::to_vec(&spots).unwrap());
        assert_eq!(parsed.is_ok(), loads, "zone id {zone}");
    }
}

#[test]
fn an_uncached_hole_keeps_the_in_place_round() {
    let content = Content::new();
    let mut game = content.game();
    let world = game.current_zone().unwrap();
    let out = go_fish(&mut game, REED, &at(REED), 1_000);
    assert!(out.scene_zoning.is_none(), "563's scene is not cached");
    assert!(starts(&out.replies, &[39, 0, 16]), "the offer goes at once");
    assert_eq!(game.scene_return(), None);
    assert_eq!(game.current_zone(), Some(world));
    let instance = offer_instance(&out.replies);
    let go = game
        .handle_in_world(&lifecycle(5, instance), &at(REED), 2_000)
        .unwrap();
    assert_eq!(status_flags(&go.replies), vec![0x40_0001]);
    let quit = game
        .handle_in_world(&lifecycle(6, instance), &at(REED), 3_000)
        .unwrap();
    assert_eq!(status_flags(&quit.replies), vec![1]);
    assert!(quit.scene_zoning.is_none());
}

#[test]
fn a_refused_go_fish_zones_nothing_and_keeps_no_pose() {
    let content = Content::new();
    let mut game = content.game();
    let menu = game
        .handle_in_world(&npc(8, SELVI, None), &at(SELVI), 1_000)
        .unwrap();
    let option = fish_option(&menu.replies, menu_label()).unwrap();
    let choose = npc(10, SELVI, Some(option));
    let before = game.state();
    // Out of reach of the guide: the NPC service rejects the choice (the gateway drops it).
    let mut away = at(SELVI);
    away.position[0] += 100.0;
    let far = game.handle_in_world(&choose, &away, 1_100);
    assert!(far.map_or(true, |out| out.scene_zoning.is_none() && !out.changed));
    assert_eq!(game.state(), before);
    assert_eq!(game.scene_return(), None);
    // Busy: an in-place round at Lila's uncached hole (560) is running.
    let lila = go_fish(&mut game, LILA, &at(LILA), 2_000);
    assert!(starts(&lila.replies, &[39, 0, 16]));
    let before = game.state();
    let busy = game.handle_in_world(&choose, &at(SELVI), 2_100).unwrap();
    assert!(busy.scene_zoning.is_none() && !busy.changed);
    assert_eq!(game.state(), before);
    assert_eq!(game.scene_return(), None);
    assert_ne!(game.current_zone().unwrap().zone_id, HOLE);
}

/// 127/19 for this house (the housing glue's enter request, as `tests/housing_session.rs`).
fn enter_house(house_guid: u64) -> Vec<u8> {
    let mut inner = vec![127, 0, 19, 0];
    inner.extend(house_guid.to_le_bytes());
    inner.extend(0i32.to_le_bytes());
    inner.push(0);
    inner
}

#[test]
fn a_house_cannot_be_entered_from_the_hole() {
    let content = Content::with_houses();
    let mut game = content.game();
    let world = game.current_zone().unwrap();
    let granted = game
        .handle_in_world(&[127, 0, 23, 0], &at(SELVI), 500)
        .unwrap();
    let list = granted
        .replies
        .iter()
        .find(|r| r.starts_with(&[127, 0, 39]));
    let lot = InstanceList::decode(&list.expect("127/39")[2..]).unwrap();
    let lot = lot.records[0].instance_guid;
    let (_, instance) = start_selvi(&mut game);

    // The fishing round holds the start gate: entering a lot would keep the hole as its way back.
    let before = game.state();
    let out = game
        .handle_in_world(&enter_house(lot), &in_hole(SPAWN), 3_500)
        .unwrap();
    assert!(out.scene_zoning.is_none() && !out.changed, "refused");
    assert!(!starts(&out.replies, &[127, 0]), "a chat line only");
    assert_eq!(game.state(), before);
    assert_eq!(game.scene_return(), Some(at(SELVI)), "still the world pose");
    assert_eq!(game.current_zone().unwrap().zone_id, HOLE);
    assert!(
        game.tick(&in_hole(SPAWN), 3_600)
            .unwrap()
            .scene_zoning
            .is_none()
    );
    let quit = game
        .handle_in_world(&lifecycle(6, instance), &in_hole(SPAWN), 4_000)
        .unwrap();
    assert_world_return(&quit.scene_zoning.expect("the cancel zones back"), &world);

    // Back in the world the lot opens, and housing's own gate still lets the player go from one
    // lot to another; the way back stays the world pose.
    let lot_scene = game
        .handle_in_world(&enter_house(lot), &at(SELVI), 5_000)
        .unwrap()
        .scene_zoning
        .expect("the lot opens once the round is over");
    let inside = CharacterState {
        guid: GUID,
        zone_id: lot_scene.id as u32,
        position: lot_scene.position[..3].try_into().unwrap(),
        rotation: [0.0, 0.0, 0.0],
    };
    let again = game
        .handle_in_world(&enter_house(lot), &inside, 6_000)
        .unwrap();
    assert!(again.scene_zoning.is_some(), "lot to lot stays open");
    assert_eq!(game.scene_return(), Some(at(SELVI)));
}

// ---- Gateway: the opcode-31 frames, the save and the reconnect, as the server runs them ----

fn login() -> Vec<u8> {
    fixtures::login("t514-fishing-scene")
}

fn inner(replies: &[Vec<u8>]) -> Vec<Vec<u8>> {
    replies
        .iter()
        .filter_map(|r| r.get(7..).map(<[u8]>::to_vec))
        .collect()
}

fn zoning(replies: &[Vec<u8>]) -> ZoneTransition {
    let frame = replies
        .iter()
        .find(|r| r.get(7..9) == Some(&[31, 0][..]))
        .expect("an opcode-31 begin-zoning frame");
    zone_transition::decode(frame).unwrap()
}

#[test]
fn the_gateway_zones_into_the_hole_saves_the_world_pose_and_brings_the_player_back() {
    let content = Content::new();
    let reference = manifest().join("../../../lab/references/rust-gateway-reference");
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.position = guide_position(SELVI);
    let kept = character.clone();
    let mut gateway = Gateway::load_gameplay(&reference, character, &content.0, None).unwrap();
    gateway.handle_at(&login(), 10_000).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), 10_000).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 10_000).unwrap();
    let menu = gateway
        .handle_at(&wrap(&npc(8, SELVI, None)), 11_000)
        .unwrap();
    let option = fish_option(&inner(&menu.replies), menu_label()).unwrap();
    let entered = gateway
        .handle_at(&wrap(&npc(10, SELVI, Some(option))), 11_100)
        .unwrap();
    assert!(entered.began_zoning);
    // The hole frame, field by field in the client reader `0x008e4ae0` order: opcode, name,
    // type, position, rotation, sky, tutorial, unknown, id, radius, geometry and the two
    // booleans. The Gameplay test checks the same struct before it is encoded; this asserts the
    // bytes the gateway actually sends, so a wrong field cannot pass here.
    let scene = zoning(&entered.replies);
    assert_eq!(
        scene,
        ZoneTransition {
            envelope_flag: 1,
            scene: "sg_fishing_stream".to_string(),
            type_id: 2,
            position: [SPAWN[0], SPAWN[1], SPAWN[2], 1.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            sky: String::new(),
            tutorial: false,
            unknown: 1,
            id: HOLE as i32,
            update_radius: 0.0,
            geometry_id: 214,
            override_update_radius: true,
            wait_for_zone_ready: false,
        },
        "the hole frame matches the client reader field for field"
    );
    assert!(
        !starts(&inner(&entered.replies), &[39, 0, 16]),
        "the offer waits"
    );
    assert_eq!(gateway.state().zone_id, HOLE);
    assert_eq!(gateway.state().position, SPAWN);
    // A save in the hole stores the world pose.
    let saved = gateway.snapshot();
    assert_eq!(saved.character, kept);

    // Membership: the hole is registered as the player's scene, and every visit gets its own
    // instance (R), so two fishers at the same hole do not see each other.
    let zone = gateway.runtime_zone().unwrap();
    assert_eq!(zone.id, HOLE);
    let mut membership = osfr_tools::world::World::runtime();
    membership.register_runtime_zone(zone).unwrap();
    let pose = osfr_tools::world::WorldPose {
        position: [SPAWN[0], SPAWN[1], SPAWN[2], 1.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
    };
    let first = membership
        .prepare_membership(1, HOLE, None, pose, false, 11_200)
        .unwrap();
    let first_instance = first.instance_id();
    membership.commit_membership(first);
    let second = membership
        .prepare_membership(2, HOLE, None, pose, false, 11_200)
        .unwrap();
    assert_ne!(second.instance_id(), first_instance);

    // The client loads the hole: the start packets follow.
    gateway.handle_at(&wrap(&[13, 0]), 12_000).unwrap();
    let loaded = gateway.handle_at(&wrap(&[10, 0]), 12_000).unwrap();
    let instance = offer_instance(&inner(&loaded.replies));

    // A reconnect from that save resumes at the world pose with no round.
    let mut resumed = Gateway::load_gameplay(
        &reference,
        saved.character.clone(),
        &content.0,
        saved.gameplay.clone(),
    )
    .unwrap();
    assert_eq!(resumed.state(), &kept);
    resumed.handle_at(&login(), 20_000).unwrap();
    resumed.handle_at(&wrap(&[13, 0]), 20_000).unwrap();
    resumed.handle_at(&wrap(&[10, 0]), 20_000).unwrap();
    let menu = resumed
        .handle_at(&wrap(&npc(8, SELVI, None)), 21_000)
        .unwrap();
    assert!(
        fish_option(&inner(&menu.replies), menu_label()).is_some(),
        "free to fish again"
    );

    // 39/6 in the first session brings the player back to the kept pose.
    let quit = gateway
        .handle_at(&wrap(&lifecycle(6, instance)), 13_000)
        .unwrap();
    assert!(quit.began_zoning);
    let back = zoning(&quit.replies);
    assert_eq!(back.id, kept.zone_id as i32);
    assert_eq!(&back.position[..3], &kept.position);
    assert_eq!(gateway.state(), &kept);
    assert_eq!(gateway.snapshot().character, kept);

    // Back in the hole, a session reset (`Gateway::reset_session`) also resumes at the world.
    gateway.handle_at(&wrap(&[13, 0]), 14_000).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), 14_000).unwrap();
    let menu = gateway
        .handle_at(&wrap(&npc(8, SELVI, None)), 15_000)
        .unwrap();
    let option = fish_option(&inner(&menu.replies), menu_label()).unwrap();
    let again = gateway
        .handle_at(&wrap(&npc(10, SELVI, Some(option))), 15_100)
        .unwrap();
    assert!(again.began_zoning);
    gateway.reset_session();
    assert_eq!(gateway.state(), &kept);
    assert_ne!(gateway.runtime_zone().unwrap().id, HOLE);
}
