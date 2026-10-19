use osfr_tools::{
    fishing::{Handle, Phase, ServerPacket, Sub, Vec4},
    fishing_service::{Change, FishingService},
    inventory::Inventory,
    profiles::ProfileState,
    quest_engine::QuestEvent,
    rewards::RewardState,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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

struct InventoryFixture(PathBuf);

impl InventoryFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-fishing-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let coins = 0u32.to_le_bytes();
        let empty = 0i32.to_le_bytes();
        let definition_bytes = [100i32.to_le_bytes(), 8i32.to_le_bytes()].concat();
        let definition = json!({
            "id": 100,
            "slot": 3,
            "class_id": 1,
            "default_tint": 5,
            "max_stack": 10,
            "model": "fixture.adr",
            "texture": "fixture",
            "tint_alias": "fixture",
            "effect": 0,
            "wield_type": 0,
            "serialized_hex": hex(&definition_bytes),
            "sha256": digest(&definition_bytes)
        });
        let content = json!({
            "schema_version": 1,
            "player_guid": PLAYER,
            "default_state": {
                "coins": 0,
                "items": [],
                "equipment": [],
                "next_instance_id": null
            },
            "profiles": [{"id": 1, "allowed_classes": [1], "slots": [3]}],
            "spans": [
                span("coins", 13, 0, &coins),
                span("items", 17, 0, &empty),
                span("equipment", 21, 1, &empty)
            ],
            "definitions": [definition]
        });
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self(path)
    }

    fn reward_state(&self) -> RewardState {
        let inventory = Inventory::load(&self.0, None).unwrap();
        let profiles = ProfileState {
            version: 1,
            active: 1,
            unlocked: BTreeSet::from([1]),
            experience: BTreeMap::new(),
        };
        RewardState::new(inventory, profiles, 4)
    }
}

impl Drop for InventoryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("inventory.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

fn span(kind: &str, offset: usize, profile_id: usize, bytes: &[u8]) -> Value {
    json!({
        "kind": kind,
        "offset": offset,
        "length": bytes.len(),
        "profile_id": profile_id,
        "sha256": digest(bytes)
    })
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn handle_bytes(handle: Handle) -> Vec<u8> {
    [handle.id.to_le_bytes(), handle.seq.to_le_bytes()].concat()
}

fn request(sub: u16, body: &[u8]) -> Vec<u8> {
    let mut packet = vec![138, 0];
    packet.extend(sub.to_le_bytes());
    packet.extend(body);
    packet
}

fn register_request() -> Vec<u8> {
    let mut body = handle_bytes(ACTOR);
    body.push(0);
    request(2, &body)
}

fn cast_request() -> Vec<u8> {
    let mut body = handle_bytes(ACTOR);
    for coordinate in [10.0f32, 0.0, 0.0, 1.0] {
        body.extend(coordinate.to_le_bytes());
    }
    body.push(0);
    request(6, &body)
}

fn reel_request() -> Vec<u8> {
    let mut body = handle_bytes(ACTOR);
    body.push(1);
    request(7, &body)
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
    assert_eq!(packet_ids(&offer), vec![(39, 16)]);
    let instance = service.instance_id(PLAYER).unwrap();
    let mut start = vec![39, 0, 5];
    start.extend(instance.to_le_bytes());
    start.extend((-1i32).to_le_bytes());
    start.extend((-1i32).to_le_bytes());
    let starting = service.handle_minigame_lifecycle(PLAYER, &start).unwrap();
    assert_eq!(packet_ids(&starting), vec![(41, 2)]);
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
        vec![(138, 3), (138, 1), (138, 11), (138, 11)]
    );

    let cast = service.handle(&cast_request(), PLAYER, T0 + 10).unwrap();
    assert_eq!(packet_ids(&cast), vec![(138, 8)]);
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
    assert_eq!(registered.replies.len(), 4);
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
