use osfr_tools::{
    fishing::{Error, FishRun, Handle, Phase, ServerPacket, Sub, Vec4},
    fishing_service::{Change, FishingService},
    quest_engine::QuestEvent,
    rewards::RewardState,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const PLAYER: u64 = 0x0000_00a1_b2c3_d4e5;
const ACTOR: Handle = Handle {
    id: 0x1122_3344,
    seq: 0x5566_7788,
};
const T0: u64 = 1_000_000;
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

#[path = "support/fishing_packets.rs"]
mod fishing_packets;

use fishing_packets::{handle_bytes, request};

struct InventoryFixture(PathBuf);

impl InventoryFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-fishing-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fishing_packets::write_inventory(&path, PLAYER, &[fishing_packets::definition_row(100)]);
        Self(path)
    }

    fn reward_state(&self) -> RewardState {
        fishing_packets::reward_state(&self.0, 4)
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("inventory.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

fn register_request() -> Vec<u8> {
    fishing_packets::register(ACTOR)
}

fn cast_request() -> Vec<u8> {
    fishing_packets::cast_at(ACTOR, [0.0, 0.0, 0.0], 10.0)
}

fn reel_request() -> Vec<u8> {
    fishing_packets::reel(ACTOR)
}

fn packet_ids(change: &Change) -> Vec<(u16, u16)> {
    change
        .replies
        .iter()
        .map(|packet| {
            let family = u16::from_le_bytes([packet[0], packet[1]]);
            let subtype = if family == 39 || family == 50 {
                u16::from(packet[2])
            } else {
                u16::from_le_bytes([packet[2], packet[3]])
            };
            (family, subtype)
        })
        .collect()
}

fn running() -> (InventoryFixture, FishingService) {
    let fixture = InventoryFixture::new();
    let mut service = FishingService::new();
    let content: Value = serde_json::from_str(FishingService::content_json()).unwrap();
    for fish in content["species"].as_array().unwrap() {
        let key = fish["key"].as_str().unwrap();
        for grade in 1..=4 {
            service.bind_item_definition(key, grade, 100).unwrap();
        }
    }
    let offer = service
        .begin(
            PLAYER,
            595,
            9001,
            "sacred_grove_shallows",
            1,
            Vec4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
            fixture.reward_state(),
        )
        .unwrap();
    assert_eq!(packet_ids(&offer), vec![(39, 16), (41, 107)]);
    let instance = service.instance_id(PLAYER).unwrap();
    let mut start = vec![39, 0, 5];
    start.extend(instance.to_le_bytes());
    start.extend((-1i32).to_le_bytes());
    start.extend((-1i32).to_le_bytes());
    let starting = service.handle_minigame_lifecycle(PLAYER, &start).unwrap();
    assert_eq!(packet_ids(&starting), vec![(41, 2), (39, 17)]);
    assert!(service.mark_started(PLAYER));
    assert_eq!(
        service.run_state(PLAYER),
        Some(osfr_tools::minigame::RunState::Playing)
    );
    (fixture, service)
}

#[test]
fn registration_cast_bite_reel_catch_has_exact_messages_and_one_item() {
    let (_fixture, mut service) = running();
    let registered = service.handle(&register_request(), PLAYER, T0).unwrap();
    assert_eq!(
        packet_ids(&registered),
        vec![(138, 3), (138, 1), (138, 4), (138, 11), (138, 11)]
    );

    let cast = service.handle(&cast_request(), PLAYER, T0 + 10).unwrap();
    assert_eq!(packet_ids(&cast), vec![(138, 8), (138, 18)]);
    let bite = service.tick(PLAYER, T0 + 1510).unwrap();
    assert_eq!(packet_ids(&bite), vec![(138, 10)]);
    assert_eq!(service.phase(PLAYER), Some(Phase::Hook));
    let hook = service.tick(PLAYER, T0 + 2410).unwrap();
    assert_eq!(packet_ids(&hook), vec![(138, 10)]);
    assert_eq!(service.phase(PLAYER), Some(Phase::Reeling));

    let caught = service.handle(&reel_request(), PLAYER, T0 + 2420).unwrap();
    assert_eq!(
        packet_ids(&caught),
        vec![(138, 14), (38, 2), (50, 1)],
        "sub 14 drives the client catch card; rewards then grant inventory and display"
    );
    match ServerPacket::decode_known(Sub::Result, &caught.replies[0][4..]).unwrap() {
        ServerPacket::Result { body } => {
            assert_eq!(body.at_18, 5);
            assert_eq!(body.at_1c, false);
            assert_ne!(body.at_20, 0);
            assert!((1..=4).contains(&body.at_3c));
            assert!(body.at_38.is_finite());
        }
        packet => panic!("expected catch result, got {packet:?}"),
    }
    let reward = service.reward_state(PLAYER).unwrap();
    assert_eq!(reward.inventory.state().items.len(), 1);
    assert_eq!(reward.inventory.state().items[0].definition_id, 100);
    assert_eq!(reward.inventory.state().items[0].count, 1);
    assert_eq!(
        caught.quest_events,
        vec![
            QuestEvent::CollectItem {
                item_id: 100,
                quantity: 1
            },
            QuestEvent::FinishMinigame { minigame_id: 595 }
        ]
    );
}

/// The production path: `FishingService::route` with the player's live inventory and profiles.
fn route(
    service: &mut FishingService,
    fixture: &InventoryFixture,
    packet: &[u8],
    now: u64,
) -> Result<Change, Error> {
    let live = fixture.reward_state();
    service
        .route(PLAYER, packet, now, &live.inventory, &live.profiles)
        .map(|change| change.expect("a fishing packet"))
}

fn fish_run(change: &Change) -> FishRun {
    match ServerPacket::decode_known(Sub::SpawnFishRun, &change.replies[1][4..]).unwrap() {
        ServerPacket::SpawnFishRun { run } => run,
        packet => panic!("expected a fish run, got {packet:?}"),
    }
}

#[test]
fn a_cast_spawns_the_bobber_model_and_the_fish_the_bite_names() {
    let (fixture, mut service) = running();
    route(&mut service, &fixture, &register_request(), T0).unwrap();
    let cast = route(&mut service, &fixture, &cast_request(), T0 + 10).unwrap();
    assert_eq!(packet_ids(&cast), vec![(138, 8), (138, 18)]);
    match ServerPacket::decode_known(Sub::SpawnBobber, &cast.replies[0][4..]).unwrap() {
        ServerPacket::SpawnBobber { code, .. } => assert_eq!(
            code, 1063,
            "the bobber model (client 0x0107b6d0 +0x54); 0x00ccfdb0 looks it up unchecked"
        ),
        packet => panic!("expected a bobber, got {packet:?}"),
    }
    let run = fish_run(&cast);
    assert_eq!(run.fish.len(), 1, "one fish per attempt");
    let fish = &run.fish[0];
    assert!(
        fish.on_hook,
        "the record starts on the hook, so state 4 sends 138/7 past the +0xd9 gate (0x00b6cbd5)"
    );
    assert!((1..=4).contains(&fish.size_grade));
    assert!(
        run.model > 0 && fish.model > 0,
        "the client looks both up unchecked"
    );

    let bite = service
        .supervise(PLAYER, 0, [0.0; 3], T0 + 1_510)
        .unwrap()
        .unwrap();
    match ServerPacket::decode_known(Sub::UpdateBobber, &bite.replies[0][4..]).unwrap() {
        ServerPacket::UpdateBobber { code, .. } => assert_eq!(code, fish.id),
        packet => panic!("expected a bite, got {packet:?}"),
    }

    // After an escape the next cast gets a fresh run for its own fish.
    let escaped = route(&mut service, &fixture, &reel_request(), T0 + 1_520).unwrap();
    assert_eq!(packet_ids(&escaped), vec![(138, 10), (138, 14)]);
    let again = route(&mut service, &fixture, &cast_request(), T0 + 2_000).unwrap();
    assert_eq!(packet_ids(&again), vec![(138, 8), (138, 18)]);
    assert_ne!(fish_run(&again).fish[0].id, fish.id);
}

#[test]
fn registration_sends_the_client_compiled_player_defaults() {
    let (_fixture, mut service) = running();
    let registered = service.handle(&register_request(), PLAYER, T0).unwrap();
    let body = &registered.replies[0][4..];
    // Reader 0x00b65880 fills +0x20, +0x5c, +0x60, +0x64, +0x68, +0x6c, +0x58, +0x78, +0x7c,
    // +0x98, +0x9c, +0xa0, +0xa4, +0x4c, +0x50, +0x48; 0x0107b6d0 writes these defaults there.
    let words: Vec<u32> = body[..64]
        .chunks(4)
        .map(|word| u32::from_le_bytes(word.try_into().unwrap()))
        .collect();
    let expected: Vec<u32> = [0.5f32, 3.0, 20.0, 0.2, 5.0, 0.5]
        .map(f32::to_bits)
        .into_iter()
        .chain([u32::MAX])
        .chain([1.0f32, 2.0, 6.0, 0.445, 1.85, 0.2, 10.0, 5.0, 12.0].map(f32::to_bits))
        .collect();
    assert_eq!(words, expected);
    assert_eq!(
        body[24..28],
        [0xff; 4],
        "BobberSplashCompositeEffectId is -1"
    );
}

/// The Fish Finder reads only the rows the server sends, and the client resolves each name as
/// `Global.Text.<id>` (`0x0097bca0`): a raw locale hash renders nothing. Its rows are refreshed
/// only by 138/4 (case 4 calls `0x00cd4480`), so the register answer must be followed by the
/// area's own fish, each with a numeric name id and a real icon set.
#[test]
fn registration_followed_by_the_fish_finder_lists_the_areas_fish() {
    let (_fixture, mut service) = running();
    let registered = service.handle(&register_request(), PLAYER, T0).unwrap();
    assert_eq!(packet_ids(&registered)[..3], [(138, 3), (138, 1), (138, 4)]);
    let content: Value = serde_json::from_str(FishingService::content_json()).unwrap();
    let species: BTreeMap<&str, &Value> = content["species"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["key"].as_str().unwrap(), row))
        .collect();
    // `running()` begins the round in Sacred Grove Shallows.
    let area = content["areas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == "sacred_grove_shallows")
        .unwrap();
    let expected: Vec<&str> = area["fish"]
        .as_array()
        .unwrap()
        .iter()
        .map(|key| key.as_str().unwrap())
        .collect();
    let registered_rows = fishing_packets::register_fish_rows(&registered.replies[0]);
    let finder = registered
        .replies
        .iter()
        .find(|reply| reply.starts_with(&[138, 0, 4, 0]))
        .expect("the Fish Finder update follows registration");
    let finder_rows = fishing_packets::fish_info_rows(finder);
    assert_eq!(
        registered_rows, finder_rows,
        "138/3 and 138/4 list the same fish"
    );
    assert_eq!(finder_rows.len(), expected.len());
    let mut types = BTreeSet::new();
    for (row, key) in finder_rows.iter().zip(expected) {
        let fish = species[key];
        assert_eq!(
            u64::from(row.type_id),
            fish["type_id"].as_u64().unwrap(),
            "{key}: type"
        );
        assert_eq!(
            u64::from(row.name_id),
            fish["name_id"].as_u64().unwrap(),
            "{key}: name"
        );
        assert_eq!(
            u64::from(row.icon_id),
            fish["icon_id"].as_u64().unwrap(),
            "{key}: icon"
        );
        assert!(
            row.name_id > 0 && row.name_id < 6_000_000,
            "{key}: a numeric Global.Text id, not a raw locale hash"
        );
        assert!(row.icon_id > 0, "{key}: has a Fish Finder icon");
        assert!(types.insert(row.type_id), "{key}: unique type");
    }
    assert!(
        finder_rows.iter().any(|row| row.catchable),
        "a level 1 fisherman can catch at least one of the area's fish"
    );
    assert!(
        finder_rows.iter().any(|row| !row.catchable),
        "the row flag still follows the minimum level"
    );
}

#[test]
fn a_refused_cast_sends_nothing_and_changes_nothing() {
    let (fixture, mut service) = running();
    route(&mut service, &fixture, &register_request(), T0).unwrap();
    let mut twin = service.clone();
    // MinCastDistance 3 m and MaxCastDistance 20 m (client defaults, player words 1 and 2).
    for distance in [2.5, 20.5, 500.0] {
        let cast = fishing_packets::cast_at(ACTOR, [0.0, 0.0, 0.0], distance);
        assert!(
            route(&mut service, &fixture, &cast, T0 + 10).is_err(),
            "{distance} m"
        );
    }
    assert_eq!(service.phase(PLAYER), Some(Phase::Idle));
    // The refused service answers the next good cast exactly like its untouched twin, so the
    // attempt and the cast sequence (the fish id) were not consumed.
    assert_eq!(
        route(&mut service, &fixture, &cast_request(), T0 + 20).unwrap(),
        route(&mut twin, &fixture, &cast_request(), T0 + 20).unwrap()
    );
}

#[test]
fn early_reel_escapes_without_grant() {
    let (_fixture, mut service) = running();
    service.handle(&register_request(), PLAYER, T0).unwrap();
    service.handle(&cast_request(), PLAYER, T0 + 1).unwrap();
    let escaped = service.handle(&reel_request(), PLAYER, T0 + 2).unwrap();
    assert_eq!(packet_ids(&escaped), vec![(138, 10), (138, 14)]);
    match ServerPacket::decode_known(Sub::Result, &escaped.replies[1][4..]).unwrap() {
        ServerPacket::Result { body } => assert_eq!(body.at_18, 0),
        packet => panic!("expected escape result, got {packet:?}"),
    }
    assert!(escaped.quest_events.is_empty());
    assert!(
        service
            .reward_state(PLAYER)
            .unwrap()
            .inventory
            .state()
            .items
            .is_empty()
    );
}

#[test]
fn client_cannot_claim_a_catch_without_a_server_bite() {
    let (_fixture, mut service) = running();
    service.handle(&register_request(), PLAYER, T0).unwrap();
    let mut forged = request(14, &handle_bytes(ACTOR));
    forged.extend(5u32.to_le_bytes());
    assert!(service.handle(&forged, PLAYER, T0 + 1).is_err());
    assert!(
        service
            .reward_state(PLAYER)
            .unwrap()
            .inventory
            .state()
            .items
            .is_empty()
    );
}

#[test]
fn disconnect_mid_fight_clears_ephemeral_state_without_grant() {
    let (_fixture, mut service) = running();
    service.handle(&register_request(), PLAYER, T0).unwrap();
    service.handle(&cast_request(), PLAYER, T0 + 1).unwrap();
    service.tick(PLAYER, T0 + 1501).unwrap();
    let disconnected = service.disconnect(PLAYER);
    assert!(disconnected.changed);
    assert_eq!(packet_ids(&disconnected), vec![(138, 12), (138, 12)]);
    assert!(disconnected.quest_events.is_empty());
    assert_eq!(service.phase(PLAYER), None);
    assert_eq!(service.len(), 0);
}

#[test]
fn leave_despawns_schools_and_drops_attempt_without_reward() {
    let (_fixture, mut service) = running();
    let registered = service.handle(&register_request(), PLAYER, T0).unwrap();
    assert_eq!(registered.replies.len(), 5);
    service.handle(&cast_request(), PLAYER, T0 + 1).unwrap();
    let left = service.leave(PLAYER).unwrap();
    assert_eq!(packet_ids(&left), vec![(138, 12), (138, 12)]);
    assert!(left.quest_events.is_empty());
    assert_eq!(service.len(), 0);
}

#[test]
fn minigame_cancel_despawns_schools_and_drops_attempt_without_reward() {
    let (_fixture, mut service) = running();
    service.handle(&register_request(), PLAYER, T0).unwrap();
    service.handle(&cast_request(), PLAYER, T0 + 1).unwrap();
    let id = service.instance_id(PLAYER).unwrap();
    let mut cancel = vec![39, 0, 6];
    cancel.extend(id.to_le_bytes());
    cancel.extend((-1i32).to_le_bytes());
    cancel.extend((-1i32).to_le_bytes());
    let cancelled = service.handle_minigame_lifecycle(PLAYER, &cancel).unwrap();
    assert_eq!(packet_ids(&cancelled), vec![(39, 19), (138, 12), (138, 12)]);
    assert!(cancelled.quest_events.is_empty());
    assert_eq!(service.len(), 0);
}

#[test]
fn content_covers_six_areas_with_client_name_ids_and_reconstructed_tuning() {
    assert_eq!(FishingService::area_count(), 6);
    assert!(FishingService::validate_content());
    let json: Value = serde_json::from_str(FishingService::content_json()).unwrap();
    assert_eq!(json["minigame_type_id"], 21);
    assert_eq!(json["sizes"].as_array().unwrap().len(), 4);
    assert_eq!(json["lures"].as_array().unwrap().len(), 13);
    assert!(json["timing"]["source"].as_str().unwrap().contains("R:"));
}
