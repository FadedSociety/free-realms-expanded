//! W90 / Security Model T3. Deterministic routed application corpus, no sockets/client.
//! Threat model (R): a Player sends arbitrary bytes and replays valid requests.
//! Bounds and synthetic rewards below are test policy, not recovered server rules.
//! Uses the existing private content/reference fixtures, like coin_shop/cooking_gameplay.
//! No new original-client facts are asserted; route provenance is the source inventory test.

use osfr_tools::{
    account_store::AccountRole,
    chat,
    cooking::Cooking,
    derby::{DerbyMessage, ParticipantKey, VehicleHitReport, VehiclePose, encode_inner},
    forging::Forging,
    game_store::Snapshot,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    harvesting,
    inventory::InventoryItem,
    kart_gateway::KartGateway,
    mining::{self, Type8Engine},
    smelting::Smelting,
    startup_snapshot::StartupSnapshot,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const NOW: u64 = 10_000;
const OUTPUT_LIMIT: usize = 1_048_576; // (R) T3 per-call oracle, includes forwarded bytes.
const NPC: u64 = 700;
const QUEST: u32 = 701;
const CHEST: i32 = 910_190;
const LOOT: i32 = 910_191;
/// (R) Synthetic derby package payout, small enough for the fixture wallet.
const DERBY_COINS: u32 = 25;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn unwrap_reply(packet: &[u8]) -> Option<&[u8]> {
    if packet.starts_with(&[5, 0, 1]) {
        let length = u32::from_le_bytes(packet.get(3..7)?.try_into().ok()?) as usize;
        packet.get(7..7 + length)
    } else {
        None
    }
}

fn derby_pose(guid: u64, position: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key: ParticipantKey(guid),
        transform_bits: [
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            position[0],
            position[1],
            position[2],
            1.0,
        ]
        .map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

fn derby_start_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 5];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn derby_quit_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 6];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn derby_cancel_request(instance: i32) -> Vec<u8> {
    let mut bytes = vec![39, 0, 7];
    bytes.extend(instance.to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes.extend((-1_i32).to_le_bytes());
    bytes
}

fn words(family: u16, sub: u16, fields: &[u32]) -> Vec<u8> {
    let mut bytes = [family.to_le_bytes(), sub.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}

fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w90-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

fn bundle(children: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for child in children {
        assert!((2..255).contains(&child.len()));
        bytes.push(child.len() as u8);
        bytes.extend(child);
    }
    bytes
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

struct Fixture {
    content: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("reports/round59-native-20261015T124443Z/content"));
        let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| lab.join("references/rust-gateway-reference"));
        assert!(
            source.join("harvesting.json").is_file(),
            "W90 requires the prepared five-movie content fixture: {}",
            source.display()
        );
        // Keep private templates in lab. This directory is strictly test-owned.
        let content = lab.join("build/w90").join(format!(
            "app-fuzz-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&content).unwrap();
        for entry in fs::read_dir(&source).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file()
                && matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("json" | "bin")
                )
            {
                fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
            }
        }
        // Small local NPC/store catalogue avoids cloning the entire regional world per input.
        fs::create_dir(content.join("npcs")).unwrap();
        write_json(
            &content.join("npcs/synthetic.json"),
            &json!({"schema_version":1,"records":[
                {"npc_id":NPC,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
                 "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic",
                 "quest_ids_given":[QUEST],"quests_accepted":[{"quest_id":QUEST}]}
            ]}),
        );
        write_json(
            &content.join("npc_spawns_extra.json"),
            &json!({"schema_version":1,"records":[]}),
        );
        write_json(
            &content.join("zones.json"),
            &json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
        );
        write_json(
            &content.join("vendors.json"),
            &json!({"schema_version":1,"vendors":[{"npc_id":NPC,"item_ids":[8]}]}),
        );
        write_json(
            &content.join("stores.json"),
            &json!({"schema_version":1,"marketplace":{
                "enabled_on_handshake":false,"orders_enabled":false,"payment_sources_enabled":false,"not_available_message":"disabled"},"items":[
                {"item_id":8,"purchase_price_coins":15,"member_discount_percent":null,"member_price_coins":null,"sell_price_coins":7,"no_sale":false}
            ]}),
        );
        // Test-only chest presentation using Inventory::presentation's checked layout.
        let mut inventory = read_json(&content.join("inventory.json"));
        for (id, item_type) in [(CHEST, 28), (LOOT, 1)] {
            let mut bytes = vec![0u8; 62];
            for (offset, value) in [(0, id), (4, 1), (12, 1), (58, item_type)] {
                bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            }
            inventory["definitions"].as_array_mut().unwrap().push(json!({
                "id":id,"slot":0,"class_id":0,"default_tint":0,"max_stack":99,
                "model":"","texture":"","tint_alias":"","effect":0,"wield_type":0,
                "serialized_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"sha256":digest(&bytes)
            }));
        }
        write_json(&content.join("inventory.json"), &inventory);
        write_json(
            &content.join("mystery_chests.json"),
            &json!({"schema_version":1,"chests":[{"definition_id":CHEST,"rewards":[LOOT]}]}),
        );
        fs::create_dir(content.join("quests")).unwrap();
        let provenance = json!([{"kind":"reconstructed","evidence":"W90 synthetic test policy"}]);
        write_json(
            &content.join("quests/synthetic.json"),
            &json!({"schema_version":1,"records":[{
                "id":QUEST,"title_string_id":1,"giver_npc_id":NPC,"prerequisite_quest_ids":[],
                "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":NPC,"source":provenance}],
                "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
            }]}),
        );
        write_json(
            &content.join("jobs.json"),
            &json!({"schema_version":1,"records":[{
                "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
                "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[QUEST],"abilities":[],"source":provenance
            }]}),
        );
        let fixture = Self { content, reference };
        fixture.write_derby_content();
        fixture
    }

    fn write_derby_content(&self) {
        let transform = [
            1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
        .map(f32::to_bits);
        write_json(
            &self.content.join("derby_entry.json"),
            &json!({
                "schema_version": 1,
                "game_id": 39,
                "vehicle": {
                    "name": [],
                    "definition": [1, 2, 3, 4],
                    "start_transform_bits": transform,
                    "unknown_flag": false,
                    "unknown_value_bits": 0
                },
                "rules": {
                    "max_drivers": 8,
                    "start_health": 20,
                    "contact_damage_per_unit": 1.0,
                    "contact_cooldown_ms": 100,
                    "contact_radius_m": 5.0,
                    "max_impact": 50.0,
                    "max_pose_age_ms": 5000,
                    "max_damage_per_contact": 30,
                    "lobby_timeout_ms": 1000,
                    "pickup_radius": 3.0,
                    "health_pickup": 5,
                    "boost_pickup": 5.0,
                    "shield_duration_ms": 1000,
                    "attack_duration_ms": 1000,
                    "ultimate_duration_ms": 1000,
                    "attack_multiplier": 2.0,
                    "ultimate_multiplier": 2.0,
                    "respawn_delay_ms": 1000,
                    "results_duration_ms": 1000,
                    "knockout_points": 2,
                    "leader_knockout_points": 3,
                    "knockout_point_loss": 1,
                    "score_target": 2,
                    "time_limit_ms": null,
                    "config": {"value_a_bits": 0, "value_b": 0}
                },
                "completion_reward_coins": DERBY_COINS,
                "completion_reward_xp": 10,
                "pickups": [],
            }),
        );
        write_json(
            &self.content.join("derby_gates.json"),
            &json!({
                "schema_version": 1,
                "menu_string_id": 3053,
                "npcs": [NPC]
            }),
        );
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.content, None).unwrap().state();
        // (R) Test wallet/stock. Leave definitions intact; no player save is read.
        let equipped: BTreeSet<_> = state
            .inventory
            .equipment
            .iter()
            .map(|e| e.item_id)
            .collect();
        state
            .inventory
            .items
            .retain(|item| equipped.contains(&item.id));
        state.inventory.coins = 100;
        for (id, definition_id) in [(900_001, CHEST), (900_002, 78137)] {
            state.inventory.items.push(InventoryItem {
                id,
                definition_id,
                tint: 0,
                count: 2,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        }
        state.inventory.next_instance_id = Some(900_003);
        state
    }

    fn unloaded(&self, content: bool, trace: bool) -> Gateway {
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let mut gateway = if content {
            Gateway::load_gameplay(
                &self.reference,
                character,
                &self.content,
                Some(self.state()),
            )
            .unwrap()
        } else {
            Gateway::load(&self.reference, character).unwrap()
        };
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        gateway.set_trace_enabled(trace);
        gateway
    }

    fn gateway(&self, content: bool, trace: bool) -> Gateway {
        let mut gateway = self.unloaded(content, trace);
        gateway.handle_at(&login(), NOW).unwrap();
        gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
        gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
        gateway
    }

    fn active(&self, kind: &str) -> Gateway {
        let mut gateway = self.gateway(true, false);
        let mut state = gateway.snapshot().gameplay.unwrap();
        match kind {
            "forging" => {
                let mut e = Forging::load(&self.content, None).unwrap();
                e.begin(16).unwrap();
                state.forging = Some(e.state().clone());
            }
            "smelting" => {
                let mut e = Smelting::load(&self.content, None).unwrap();
                e.begin(14).unwrap();
                state.smelting = Some(e.state().clone());
            }
            "cooking" => {
                let mut e = Cooking::load(&self.content, None).unwrap();
                e.begin(45).unwrap();
                state.cooking = Some(e.state().clone());
            }
            "mining" => {
                let mut e = Type8Engine::load(
                    &self.content,
                    mining::CONTENT_FILE,
                    mining::MINING_GAME_ID,
                    mining::FIRST_MINING_INSTANCE,
                    None,
                )
                .unwrap();
                e.begin(14).unwrap();
                state.mining = Some(e.state().clone());
            }
            "harvesting" => {
                let mut e = Type8Engine::load(
                    &self.content,
                    harvesting::CONTENT_FILE,
                    111,
                    harvesting::FIRST_INSTANCE_ID,
                    None,
                )
                .unwrap();
                e.begin(45).unwrap();
                state.harvesting = Some(e.state().clone());
            }
            "kart" => {
                let mut e = KartGateway::load(&self.content, None).unwrap().unwrap();
                let (_, scene) = e.begin(gateway.state()).unwrap();
                state.kart_entry = Some(e.state().clone());
                // Reconnect deliberately abandons kart saves. Use a synthetic reference in
                // the active instance to test the routed handler with dev tools still OFF.
                let reference = self.content.join("kart-reference");
                fs::create_dir(&reference).unwrap();
                for entry in fs::read_dir(&self.reference).unwrap() {
                    let path = entry.unwrap().path();
                    if path.is_file()
                        && matches!(
                            path.extension().and_then(|s| s.to_str()),
                            Some("json" | "bin")
                        )
                    {
                        fs::copy(&path, reference.join(path.file_name().unwrap())).unwrap();
                    }
                }
                let mut manifest = read_json(&reference.join("manifest.json"));
                manifest["initial_state"]["zone_id"] = json!(scene.id);
                let zone_file = reference.join(manifest["templates"][3]["file"].as_str().unwrap());
                let mut zone = fs::read(&zone_file).unwrap();
                let offset = manifest["zone_layout"]["zone_id_offset"].as_u64().unwrap() as usize;
                zone[offset..offset + 4].copy_from_slice(&(scene.id as u32).to_le_bytes());
                fs::write(&zone_file, &zone).unwrap();
                manifest["templates"][3]["sha256"] = json!(digest(&zone));
                write_json(&reference.join("manifest.json"), &manifest);
                let mut character = gateway.state().clone();
                character.zone_id = scene.id as u32;
                character.position = scene.position[..3].try_into().unwrap();
                gateway = Gateway::load_with_startup_snapshot(
                    &reference,
                    character,
                    StartupSnapshot::empty(),
                )
                .unwrap();
                gateway.enable_gameplay(&self.content, Some(state)).unwrap();
                gateway.set_dev_tools(false);
                gateway.set_role(AccountRole::Player);
                for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
                    gateway.handle_at(&packet, NOW).unwrap();
                }
                return gateway;
            }
            _ => panic!("unknown test activity"),
        }
        // Use real loaders, with server-authored fixture state; no cheat command is enabled.
        gateway.enable_gameplay(&self.content, Some(state)).unwrap();
        gateway
    }

    fn active_derby(&self) -> (Gateway, ParticipantKey, i32) {
        self.active_derby_with(self.gateway(true, false))
    }

    /// Drive a gateway into a running derby match through the routed NPC menu.
    fn active_derby_with(&self, mut gateway: Gateway) -> (Gateway, ParticipantKey, i32) {
        let guid = gateway.state().guid;
        let mut menu = vec![26, 0, 8, 0];
        menu.extend(NPC.to_le_bytes());
        menu.push(0);
        gateway.handle_at(&wrap(&menu), 1_000).unwrap();
        let mut choice = vec![26, 0, 10, 0];
        choice.extend(NPC.to_le_bytes());
        choice.extend(3_i32.to_le_bytes());
        let entered = gateway.handle_at(&wrap(&choice), 1_100).unwrap();
        let instance = entered
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .find(|packet| packet.starts_with(&[39, 0, 16]))
            .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
            .expect("derby offer");
        gateway
            .handle_at(&wrap(&derby_start_request(instance)), 1_200)
            .unwrap();
        let register = encode_inner(&DerbyMessage::RegisterPlayer {
            key: ParticipantKey(guid),
            unknown_key: 0,
        })
        .unwrap();
        let setup = gateway.handle_at(&wrap(&register), 1_300).unwrap();
        let npc = setup
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .filter_map(|packet| osfr_tools::derby::decode_inner(packet).ok())
            .find_map(|message| match message {
                DerbyMessage::CreateVehicle(vehicle) if vehicle.key != ParticipantKey(guid) => {
                    Some(vehicle.key)
                }
                _ => None,
            })
            .expect("NPC driver");
        for value in 3..=8 {
            let packet = encode_inner(&DerbyMessage::PlayerPhase {
                key: ParticipantKey(guid),
                phase: value,
            })
            .unwrap();
            gateway
                .handle_at(&wrap(&packet), 2_000 + value as u64)
                .unwrap();
        }
        for value in (0..=4).rev() {
            let packet = encode_inner(&DerbyMessage::CountdownStatus {
                key: ParticipantKey(guid),
                count: value,
            })
            .unwrap();
            gateway
                .handle_at(&wrap(&packet), 3_000 + (4 - value) as u64)
                .unwrap();
        }
        assert_eq!(
            gateway.gameplay().unwrap().derby_phase(),
            Some(osfr_tools::derby::DerbyPhase::Running)
        );
        (gateway, npc, instance)
    }

    /// A logged-in gateway whose wallet is at the wire maximum, so any coin grant overflows.
    fn gateway_with_max_coins(&self) -> Gateway {
        let mut state = self.state();
        state.inventory.coins = i32::MAX as u32;
        let mut character = Gateway::reference_state(&self.reference).unwrap();
        character.position = [0.; 3];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.content, Some(state)).unwrap();
        gateway.set_dev_tools(false);
        gateway.set_role(AccountRole::Player);
        gateway.set_trace_enabled(false);
        for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
            gateway.handle_at(&packet, NOW).unwrap();
        }
        gateway
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/build/w90")
            .canonicalize()
            .unwrap();
        if self
            .content
            .canonicalize()
            .is_ok_and(|p| p.starts_with(&root) && p != root)
        {
            let _ = fs::remove_dir_all(&self.content);
        }
    }
}

fn bounded(out: &GatewayOutcome) {
    assert!(out.replies.len() <= 128 && out.applications.len() <= 128);
    let bytes: usize = out
        .replies
        .iter()
        .chain(&out.applications)
        .map(Vec::len)
        .sum();
    assert!(bytes <= OUTPUT_LIMIT, "output {bytes} exceeds W90 bound");
}

fn follow_up(gateway: &mut Gateway) {
    let before = gateway.snapshot();
    let mut time = 52u16.to_le_bytes().to_vec();
    time.extend([0; 12]);
    time.push(1);
    let out = gateway.handle_at(&wrap(&time), NOW).unwrap();
    bounded(&out);
    assert_eq!(out.replies.len(), 1, "valid time request stopped working");
    assert_eq!(&out.replies[0][7..9], &[52, 0]);
    assert_eq!(gateway.snapshot(), before);
}

fn unchanged(gateway: &mut Gateway, packet: &[u8], label: &str) {
    let before = gateway.snapshot();
    let result = catch_unwind(AssertUnwindSafe(|| gateway.handle_at(packet, NOW)));
    assert!(result.is_ok(), "panic: {label}, input={packet:02x?}");
    if let Ok(out) = result.unwrap() {
        bounded(&out);
    }
    assert!(
        gateway.snapshot() == before,
        "snapshot mutation: {label}, input={packet:02x?}"
    );
    follow_up(gateway);
}

// Exercise every truncation, trailing data, every sliding u32 length/count, and every
// byte as an invalid boolean/UTF-8 byte. Short seeds keep allocations/runtime bounded.
fn hostile(seed: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut cases = Vec::new();
    for cut in 0..seed.len() {
        cases.push((format!("cut {cut}"), seed[..cut].to_vec()));
    }
    for tail in [&[0][..], &[0xff][..], &[0xff; 8][..]] {
        let mut bytes = seed.to_vec();
        bytes.extend(tail);
        cases.push(("trailing".into(), bytes));
    }
    for offset in 4..seed.len().saturating_sub(3) {
        let mut bytes = seed.to_vec();
        bytes[offset..offset + 4].fill(0xff);
        cases.push((format!("u32::MAX at {offset}"), bytes));
    }
    for offset in 4..seed.len() {
        for value in [2, 0xff] {
            let mut bytes = seed.to_vec();
            bytes[offset] = value;
            if bytes != seed {
                cases.push((format!("byte {offset}={value}"), bytes));
            }
        }
    }
    let mut seen = BTreeSet::new();
    cases.retain(|(_, bytes)| bytes.as_slice() != seed && seen.insert(bytes.clone()));
    cases
}

fn seeds() -> Vec<(&'static str, Vec<u8>)> {
    let mut result = vec![("social_chat", chat::local_request(""))];
    let mut bad_utf8 = chat::local_request("");
    bad_utf8[62..66].copy_from_slice(&1u32.to_le_bytes());
    bad_utf8.insert(66, 0xff);
    result.push(("social_chat", bad_utf8));
    for sub in [2, 3] {
        result.push(("quick_chat", words(67, sub, &[u32::MAX; 4])));
    }
    for sub in [4, 6, 8, 10, 23] {
        let packet = match sub {
            4 => words(26, sub, &[]),
            6 => words(26, sub, &[u32::MAX]),
            8 => {
                let mut p = words(26, sub, &[u32::MAX; 2]);
                p.push(0);
                p
            }
            10 => words(26, sub, &[u32::MAX; 3]),
            _ => {
                let mut p = vec![26, 0, 23, 0, 0, 0];
                p.extend(u32::MAX.to_le_bytes());
                p
            }
        };
        result.push(("npc_interaction", packet));
    }
    for sub in [14, 15, 17, 21, 30] {
        result.push(("social_router", words(26, sub, &[u32::MAX; 5])));
    }
    for sub in [1, 3, 4, 6, 14, 16] {
        result.push(("social_router", words(40, sub, &[u32::MAX; 4])));
    }
    // 26/19 select-player: own guid then target guid (W143); unknown target.
    result.push(("player_radial", words(26, 19, &[0, 0, u32::MAX, u32::MAX])));
    for sub in [2, 14, 15] {
        let mut p = vec![49, 0, sub, 0, 0, 0];
        p.extend(u32::MAX.to_le_bytes());
        if sub == 2 {
            p.push(1);
        }
        result.push(("quest_engine", p));
    }
    // Family 47 uses a one-byte subtype: 47/12 select, 47/13 lock (2 is not a
    // valid lock byte), and the family wildcard's unknown subtype.
    let mut select = vec![47, 0, 12];
    select.extend(u32::MAX.to_le_bytes());
    result.push(("quest_engine", select));
    result.push(("quest_engine", vec![47, 0, 13, 2]));
    result.push(("quest_engine", vec![47, 0, 0xff, 0xff]));
    // Family 45 also uses a one-byte subtype: 45/7 is the client's goal-complete claim (i32 id).
    let mut claim = vec![45, 0, 7];
    claim.extend(u32::MAX.to_le_bytes());
    result.push(("quest_engine", claim));
    result.push(("quest_engine", vec![45, 0, 0xff, 0xff]));
    // Family 59 uses a one-byte subtype like 47: every client trade sub, then the wildcard.
    for sub in [3u8, 6, 7, 8, 9, 10] {
        let mut p = vec![59, 0, sub];
        p.extend(u32::MAX.to_le_bytes());
        p.extend(vec![0xff; 8]);
        result.push(("trade", p));
    }
    result.push(("trade", vec![59, 0, 0xff, 0xff]));
    result.push(("kart", words(26, 42, &[])));
    // 33/* routes as a family, even unknown subtypes. Include all implemented client cases.
    for sub in [2, 6, 7, 9, 10, 17, 19, 22, 0xffff] {
        use osfr_tools::{kart_entry, kart_service};
        let length = match sub {
            2 => kart_entry::PLAYER_STATE_PAYLOAD_LEN,
            9 => kart_entry::REGISTER_PAYLOAD_LEN,
            6 => kart_service::SERVER_DATA_PAYLOAD_LEN,
            7 | 17 => kart_service::GATE_HIT_PAYLOAD_LEN,
            10 => kart_service::COUNTDOWN_PAYLOAD_LEN,
            19 | 22 => kart_service::OWNED_ID_PAYLOAD_LEN,
            _ => 24,
        };
        let mut input = words(33, sub, &[]);
        input.extend(vec![0xff; length]);
        result.push(("kart", input));
    }
    result.push(("kart", words(41, 108, &[u32::MAX; 4])));
    result.push(("kart", words(102, 1, &[])));
    // Family 34 is the demolition derby processor. Cover every implemented client record and the
    // wildcard subtype required by the source inventory.
    for (sub, length) in [
        (2, 12),
        (5, 128),
        (6, 16),
        (7, 12),
        (14, 24),
        (16, 24),
        (0xffff, 16),
    ] {
        let mut input = words(34, sub, &[]);
        input.extend(vec![0xff; length]);
        result.push(("derby", input));
    }
    // Family 168 (mounts, W162) has a one-byte subtype: 1 board, 3 dismount, 6 spawn, 8 spawn by
    // item, and the family wildcard's unknown subtype.
    result.push(("mounts", [vec![168, 0, 1], vec![0xff; 8]].concat()));
    result.push(("mounts", vec![168, 0, 3, 0]));
    result.push(("mounts", [vec![168, 0, 6], vec![0xff; 4]].concat()));
    result.push(("mounts", [vec![168, 0, 8], vec![0xff; 4], vec![1]].concat()));
    result.push(("mounts", vec![168, 0, 0xff, 0xff]));
    // Family 53 (pets, W177) has a one-byte subtype: summon, the serializer-shaped
    // dismiss (u32 id + action), a truncated dismiss, UI mode, summon by item, and
    // the family wildcard retained as an ordinary unknown body.
    result.push(("pets", [vec![53, 0, 4], vec![0xff; 4], vec![1]].concat()));
    result.push(("pets", vec![53, 0, 4, 0, 0, 0, 0, 0]));
    result.push(("pets", vec![53, 0, 4, 0]));
    result.push(("pets", vec![53, 0, 6, 0]));
    result.push((
        "pets",
        [vec![53, 0, 6], 1u32.to_le_bytes().to_vec(), vec![b'A']].concat(),
    ));
    result.push(("pets", vec![53, 0, 24, 2, 0, 0, 0]));
    result.push(("pets", [vec![53, 0, 32], vec![0xff; 8]].concat()));
    result.push(("gameplay.handle_in_world", vec![53, 0, 0xff, 0xff]));
    result.push(("combat", words(36, 10, &[1, 0, 0, u32::MAX, u32::MAX])));
    result.push(("combat", words(36, 12, &[u32::MAX])));
    for (sub, length) in [(2, 2), (3, 3), (8, 4), (15, 1), (16, 1)] {
        result.push((
            if sub >= 15 { "chest" } else { "inventory" },
            words(42, sub, &vec![u32::MAX; length]),
        ));
    }
    result.push(("inventory", words(35, 34, &[u32::MAX, 0])));
    result.push(("merchant", words(66, 43, &[u32::MAX; 4])));
    result.push(("merchant", words(165, 4, &[u32::MAX; 6])));
    result.push(("merchant", words(165, 5, &[u32::MAX; 4])));
    result.push(("titles", words(152, 4, &[u32::MAX])));
    result.push(("titles", words(152, 5, &[])));
    result.push(("claims", words(108, 1, &[])));
    result.push(("claims", words(108, 3, &[u32::MAX; 3])));
    result.push(("claims", words(108, 5, &[u32::MAX; 3])));
    // W172: Coin Store siblings (definitions, list update, buy-back, gift) and the
    // Marketplace rows (bare header, locale string, unrecovered layouts).
    result.push(("merchant", words(165, 2, &[2, 8, u32::MAX])));
    result.push(("merchant", words(165, 8, &[])));
    result.push(("merchant", words(165, 12, &[u32::MAX, 0])));
    result.push(("merchant", words(165, 14, &[0, 0, 0, 0, 1, 8, 1])));
    for sub in [1, 3, 10, 12, 14, 18, 22, 27, 38] {
        result.push(("merchant", words(66, sub, &[])));
    }
    // F387: the store's close statistics (66/29) and one-time commerce session request
    // (117/41) stay unknown traffic; the hostile corpus keeps exercising both.
    result.push(("gameplay.handle_in_world", words(66, 29, &[0; 16])));
    result.push(("gameplay.handle_in_world", words(117, 41, &[])));
    for sub in [16, 20, 25] {
        let mut locale = words(66, sub, &[5]);
        locale.extend(b"en_US");
        result.push(("merchant", locale));
    }
    // T006: family 129 (guilds) is routed by the gateway's guild router; an unknown
    // subtype is understood and refused without a reply or a guild change.
    result.push(("guilds", words(129, 0xffff, &[u32::MAX; 2])));
    result.push(("class_selection", words(26, 13, &[u32::MAX])));
    result.push(("encounter_exit", words(41, 109, &[u32::MAX; 2])));
    for sub in [5, 6, 7, 14] {
        let mut p = vec![39, 0, sub];
        for _ in 0..3 {
            p.extend(u32::MAX.to_le_bytes());
        }
        if sub == 14 {
            p.extend(u32::MAX.to_le_bytes());
        }
        result.push(("movie", p));
    }
    // 98/1 path request (W164): handle, kind 1, aux, two vec4, tail; the same as Take Me
    // There (kind 2, T404); then the family's unknown subtype.
    let mut path = vec![98, 0, 1];
    path.extend(0u64.to_le_bytes());
    path.extend(1u32.to_le_bytes());
    path.extend([0; 4 + 32 + 4]);
    result.push(("pathing", path.clone()));
    path[11] = 2;
    result.push(("pathing", path));
    result.push(("pathing", vec![98, 0, 0xff, 0xff]));
    // W173: channel switch on/off by name, and the bubble colour (three i32 and a u64).
    for sub in [32u16, 33] {
        let mut switch = vec![26, 0, sub as u8, 0, 9, 0, 0, 0];
        switch.extend(b"WorldArea");
        result.push(("chat_settings", switch));
    }
    let mut bubble = vec![26, 0, 18, 0];
    bubble.extend(13_158i32.to_le_bytes());
    bubble.extend(13_434_879i32.to_le_bytes());
    bubble.extend(2i32.to_le_bytes());
    bubble.extend(0u64.to_le_bytes());
    result.push(("chat_settings", bubble));
    // W174: handled requests whose refusal sends nothing, then the quiet list (R). A wildcard
    // family needs a seed with an unknown subtype for the source inventory.
    result.push(("recipe_book", words(26, 24, &[u32::MAX])));
    result.push(("item_bar", words(42, 6, &[u32::MAX; 3])));
    result.push(("item_bar", words(42, 9, &[u32::MAX; 3])));
    result.push(("item_use", words(42, 15, &[u32::MAX])));
    result.push(("encounter", [words(41, 122, &[0, 0]), vec![0]].concat()));
    result.push(("activity_portal", vec![167, 0, 1, 4]));
    result.push((
        "activity_portal",
        [vec![167, 0, 1, 2], vec![0xff; 8]].concat(),
    ));
    result.push(("activity_portal", vec![167, 0, 0xff, 0xff]));
    // Family 76 is the native Soccer family; inactive sessions refuse every
    // subtype without a snapshot change. The wildcard covers the family route.
    result.push(("soccer", words(76, 14, &[u32::MAX; 15])));
    result.push(("soccer", vec![76, 0, 0xff, 0xff]));
    // T007: matchmaking list/stats and the explicit join refusal (queue id in the
    // native 141/3 prefix), the in-progress list request, and the lobby close
    // (family 101 has a three-byte header and a nonzero u32 lobby type).
    result.push(("matchmaking", words(141, 1, &[u32::MAX; 2])));
    result.push(("matchmaking", words(141, 3, &[u32::MAX; 6])));
    result.push(("matchmaking", words(141, 13, &[u32::MAX; 2])));
    result.push(("matchmaking", words(41, 129, &[u32::MAX; 4])));
    result.push(("matchmaking", vec![101, 0, 9, 3, 0, 0, 0]));
    // A guarded one-byte-sub family needs an unknown subtype seed for the source
    // inventory; 101/0xff contains no understood lobby close.
    result.push(("gameplay.handle_in_world", vec![101, 0, 0xff, 0xff]));
    result.push(("free_interaction", words(26, 20, &[])));
    result.push(("profile_notice", words(26, 26, &[0x30, 0x30])));
    result.push(("profile_notice", words(26, 37, &[0; 3])));
    result.push(("client_metrics", words(105, 1, &[0; 3])));
    result.push(("client_metrics", words(105, 0xffff, &[0; 2])));
    result.push(("client_log", words(109, 0xffff, &[0; 2])));
    result.push((
        "client_log",
        [
            vec![109, 0],
            vec![5, 0, 0, 0],
            b"a.log".to_vec(),
            vec![2, 0, 0, 0],
            b"hi".to_vec(),
        ]
        .concat(),
    ));
    result.push(("ui_telemetry", vec![194, 0, 2, 1, 0, 0, 0, 0x58, 0, 0, 0]));
    result.push(("ui_telemetry", vec![194, 0, 0xff, 0xff]));
    result.push(("announcements_request", vec![193, 0, 1, 0]));
    result.push(("announcements_request", vec![193, 0, 0xff, 0xff]));
    result.push(("unknown", words(0xffff, 0xffff, &[u32::MAX; 2])));
    let mut movement = 125u16.to_le_bytes().to_vec();
    movement.extend(u64::MAX.to_le_bytes());
    movement.extend([0; 24]);
    movement.extend([0, 0]);
    result.push(("movement", movement));
    // T008: the separate jump update is a 40-byte body; a foreign GUID must refuse atomically.
    let mut jump = 164u16.to_le_bytes().to_vec();
    jump.extend(u64::MAX.to_le_bytes());
    jump.extend([0; 30]);
    result.push(("movement", jump));
    let mut camera = 126u16.to_le_bytes().to_vec();
    camera.extend([0; 24]);
    result.push(("gateway.camera", camera));
    result
}

#[test]
fn t3_source_inventory_covers_all_gateway_labels_and_gameplay_dispatch_families() {
    let source = include_str!("../src/gateway.rs");
    let handler = source
        .split("fn application_handler(")
        .nth(1)
        .unwrap()
        .split("fn set_trace(")
        .next()
        .unwrap();
    let labels: BTreeSet<_> = handler
        .lines()
        .filter_map(|line| line.split("=> \"").nth(1))
        .map(|text| text.split('"').next().unwrap())
        .collect();
    let covered: BTreeSet<_> = seeds()
        .iter()
        .map(|(name, _)| *name)
        .chain(["gameplay.handle_in_world"])
        .filter(|name| labels.contains(name))
        .collect();
    assert_eq!(covered, labels, "new route needs a W90 seed");
    // Inventory the literal selector grammar in the live function, including pipes
    // and the 33 wildcard. A new subtype under an existing label must fail this gate.
    let heads: BTreeSet<_> = seeds()
        .iter()
        .map(|(_, p)| {
            (
                u16::from_le_bytes(p[..2].try_into().unwrap()),
                u16::from_le_bytes(p[2..4].try_into().unwrap()),
            )
        })
        .collect();
    let mut selectors = BTreeSet::new();
    for pattern in handler.split("(Some(").skip(1) {
        let (family, rest) = pattern.split_once("),").unwrap();
        let family = family.parse::<u16>().unwrap();
        let rest = rest.trim_start();
        if let Some(rest) = rest.strip_prefix("Some(") {
            for sub in rest.split(')').next().unwrap().split('|') {
                let sub = sub.trim().parse::<u16>().unwrap();
                selectors.insert((family, Some(sub)));
                assert!(
                    heads.contains(&(family, sub)),
                    "uncovered live selector {family}/{sub}"
                );
            }
        } else {
            assert!(rest.starts_with('_'), "review new selector grammar");
            selectors.insert((family, None));
            assert!(
                heads.contains(&(family, u16::MAX)),
                "wildcard family needs an unknown subtype"
            );
        }
    }
    let game = include_str!("../src/gameplay.rs");
    let compact: String = game.chars().filter(|ch| !ch.is_whitespace()).collect();
    for call in [
        "handle_kart_entry(",
        "handle_derby(",
        "handle_npc_packet(",
        "handle_opening_inner(",
        "handle_mystery_chest(",
        "self.inventory.handle(inner)",
        ".combat.handle(inner,",
        "self.encounter.as_mut().unwrap().handle(",
    ] {
        assert!(
            compact.contains(call),
            "review the live gameplay route inventory: {call}"
        );
    }
    println!(
        "W90 labels covered/routed: {}/{} {labels:?}; additional class/chest/39 movies/41 exit + fallback covered",
        covered.len(),
        labels.len()
    );
    println!(
        "W90 selectors covered/routed: {0}/{0} {selectors:?}",
        selectors.len()
    );
}

#[test]
fn t3_routed_hostile_corpus_content_and_trace_on_off() {
    let start = Instant::now();
    let fixture = Fixture::new();
    let mut count = 0;
    for content in [false, true] {
        for trace in [false, true] {
            let mut gateway = fixture.gateway(content, trace);
            for (handler, seed) in seeds() {
                for (mutation, input) in hostile(&seed) {
                    unchanged(
                        &mut gateway,
                        &wrap(&input),
                        &format!("{handler} {mutation} content={content} trace={trace}"),
                    );
                    count += 1;
                }
                unchanged(&mut gateway, &wrap(&seed), handler);
                // Family's unknown subtype, retaining original body.
                let mut unknown = seed.clone();
                unknown[2..4].fill(0xff);
                unchanged(&mut gateway, &wrap(&unknown), "unknown subtype");
            }
        }
    }
    println!(
        "W90 routed corpus: {count} mutations, elapsed {:?}",
        start.elapsed()
    );
    assert!(start.elapsed() < Duration::from_secs(60));
}

#[test]
fn active_derby_hostile_records_cancel_and_tick_transitions_are_bounded() {
    let fixture = Fixture::new();
    let (mut gateway, npc, instance) = fixture.active_derby();
    let guid = gateway.state().guid;
    let absent_pose_hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: ParticipantKey(guid),
        other: npc,
        impact_bits: 1.0_f32.to_bits(),
    }))
    .unwrap();
    for (mutation, input) in hostile(&absent_pose_hit) {
        let before = gateway.snapshot();
        let phase = gateway.gameplay().unwrap().derby_phase();
        let result = catch_unwind(AssertUnwindSafe(|| gateway.handle_at(&wrap(&input), NOW)));
        assert!(result.is_ok(), "panic: derby {mutation}");
        if let Ok(out) = result.unwrap() {
            bounded(&out);
        }
        assert_eq!(
            gateway.snapshot(),
            before,
            "derby mutation changed persisted state: {mutation}"
        );
        assert_eq!(
            gateway.gameplay().unwrap().derby_phase(),
            phase,
            "derby mutation changed the live phase: {mutation}"
        );
        follow_up(&mut gateway);
    }

    let before_stale = gateway.snapshot();
    let before_quit_coins = before_stale.gameplay.as_ref().unwrap().inventory.coins;
    let stale_phase = gateway.gameplay().unwrap().derby_phase();
    gateway
        .handle_at(&wrap(&derby_quit_request(instance + 1)), 4_100)
        .unwrap();
    assert_eq!(gateway.snapshot(), before_stale);
    assert_eq!(gateway.gameplay().unwrap().derby_phase(), stale_phase);

    let quit = gateway
        .handle_at(&wrap(&derby_quit_request(instance)), 4_200)
        .unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert!(
        quit.applications
            .iter()
            .any(|packet| packet.starts_with(&[39, 0, 6]))
    );
    assert!(
        quit.replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[39, 0, 19]))
    );
    let after_quit = gateway.snapshot();
    assert_eq!(
        after_quit.gameplay.as_ref().unwrap().inventory.coins,
        before_quit_coins
    );
    gateway
        .handle_at(&wrap(&derby_quit_request(instance)), 4_300)
        .unwrap();
    assert_eq!(gateway.snapshot(), after_quit);

    let (mut gateway, npc, instance) = fixture.active_derby();
    let guid = gateway.state().guid;
    gateway
        .handle_at(&wrap(&derby_pose(guid, [0.0, 0.0, 0.0])), 3_900)
        .unwrap();
    let before_limited = gateway.snapshot();
    let limited = gateway
        .handle_at(
            &wrap(
                &encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
                    reporter: ParticipantKey(guid),
                    other: npc,
                    impact_bits: 40.0_f32.to_bits(),
                }))
                .unwrap(),
            ),
            3_950,
        )
        .unwrap();
    assert_eq!(gateway.snapshot(), before_limited);
    assert!(
        !limited
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| *packet == [34, 0, 1, 0, 10, 0, 0, 0])
    );
    let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: ParticipantKey(guid),
        other: npc,
        impact_bits: 20.0_f32.to_bits(),
    }))
    .unwrap();
    let before_payout = gateway
        .snapshot()
        .gameplay
        .as_ref()
        .unwrap()
        .inventory
        .coins;
    let results = gateway.handle_at(&wrap(&hit), 4_000).unwrap();
    assert!(
        results
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| *packet == [34, 0, 1, 0, 10, 0, 0, 0])
    );
    let paid = gateway.snapshot();
    let paid_game = paid.gameplay.as_ref().unwrap();
    assert_eq!(
        paid_game.inventory.coins,
        before_payout + DERBY_COINS,
        "the results phase pays one flat completion reward"
    );
    assert_eq!(
        paid_game.derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "the paid match instance is recorded in the save"
    );
    assert!(
        results
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "the completion sends the reward bundle"
    );

    // Replaying the finished match's record while its results screen is up pays nothing more.
    let replay = gateway.handle_at(&wrap(&hit), 4_100).unwrap();
    assert!(!replay.gameplay_changed);
    assert_eq!(gateway.snapshot(), paid, "a replayed result pays nothing");

    let close_ready = gateway.tick_at(5_000).unwrap();
    assert!(
        close_ready
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| *packet == [34, 0, 1, 0, 11, 0, 0, 0])
    );
    let closed = gateway.handle_at(&wrap(&[26, 0, 42, 0]), 5_100).unwrap();
    assert!(closed.gameplay_changed);
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert!(
        !closed
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "closing the results screen pays nothing"
    );
    let after_close = gateway.snapshot();
    let after_close_game = after_close.gameplay.as_ref().unwrap();
    assert_eq!(
        after_close_game.inventory.coins,
        before_payout + DERBY_COINS,
        "the close pays nothing more"
    );
    assert_eq!(
        after_close_game.derby.as_ref().unwrap().paid_matches,
        vec![instance]
    );

    // Farming is allowed: the gate offers another match at once, and a new match is a new id.
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(NPC.to_le_bytes());
    menu.push(0);
    gateway.handle_at(&wrap(&menu), 5_200).unwrap();
    let mut choice = vec![26, 0, 10, 0];
    choice.extend(NPC.to_le_bytes());
    choice.extend(3_i32.to_le_bytes());
    let reentered = gateway.handle_at(&wrap(&choice), 5_300).unwrap();
    let next_instance = reentered
        .replies
        .iter()
        .filter_map(|packet| unwrap_reply(packet))
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .expect("a second derby offer");
    assert!(
        next_instance > instance,
        "the instance counter resumes above the paid ledger"
    );
    assert!(gateway.gameplay().unwrap().derby_active());
    let cancelled = gateway.handle_at(&wrap(&[26, 0, 42, 0]), 5_400).unwrap();
    assert!(cancelled.gameplay_changed);
    assert!(!gateway.gameplay().unwrap().derby_active());
    gateway.reset_session();
    let after_reset = gateway.snapshot();
    let after_reset_game = after_reset.gameplay.as_ref().unwrap();
    assert_eq!(
        after_reset_game.inventory.coins,
        before_payout + DERBY_COINS
    );
    assert_eq!(
        after_reset_game.derby.as_ref().unwrap().paid_matches,
        vec![instance],
        "a reset keeps the completion ledger"
    );
    assert!(!gateway.gameplay().unwrap().derby_active());
}

/// T219 gate 4: a completion payout that cannot be applied (this wallet is already at the wire
/// maximum) is a refusal. The application is dropped, the match stays running, the wallet and the
/// ledger are unchanged, and the session keeps serving this player; no other player is involved.
#[test]
fn a_refused_derby_completion_payout_changes_nothing() {
    let fixture = Fixture::new();
    let (mut gateway, npc, _) = fixture.active_derby_with(fixture.gateway_with_max_coins());
    let guid = gateway.state().guid;
    gateway
        .handle_at(&wrap(&derby_pose(guid, [0.0, 0.0, 0.0])), 3_900)
        .unwrap();
    let before = gateway.snapshot();
    let hit = encode_inner(&DerbyMessage::VehicleHit(VehicleHitReport {
        reporter: ParticipantKey(guid),
        other: npc,
        impact_bits: 20.0_f32.to_bits(),
    }))
    .unwrap();
    let refused = gateway.handle_at(&wrap(&hit), 4_000).unwrap();
    bounded(&refused);
    assert!(
        !refused
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[50, 0, 1])),
        "a refused payout sends no reward bundle"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a refused payout changes nothing"
    );
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(osfr_tools::derby::DerbyPhase::Running),
        "the refused results transition is rolled back, not half applied"
    );
    follow_up(&mut gateway);
}

#[test]
fn reset_during_a_running_derby_retires_it_and_restores_the_job() {
    let fixture = Fixture::new();
    let (mut gateway, _, _) = fixture.active_derby();
    let before = gateway.snapshot();
    let before_game = before.gameplay.as_ref().unwrap();
    let prior_job = before_game.profiles.as_ref().unwrap().active;
    assert_ne!(
        prior_job,
        osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE
    );
    assert!(gateway.gameplay().unwrap().derby_active());
    assert_eq!(
        gateway.gameplay().unwrap().active_profile(),
        Some(osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)
    );

    gateway.reset_session();
    let after = gateway.snapshot();
    let after_game = after.gameplay.as_ref().unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(
        after_game.profiles.as_ref().unwrap().active,
        prior_job,
        "reset restores the job the derby replaced"
    );
    assert_eq!(after_game.inventory.coins, before_game.inventory.coins);

    // A fresh login does not resume the retired match; only a new gate choice starts one.
    assert_eq!(gateway.gameplay().unwrap().derby_phase(), None);
}

#[test]
fn gateway_offer_cancel_restores_state_and_rejects_a_stale_quit() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    let before = gateway.snapshot();
    let before_coins = before.gameplay.as_ref().unwrap().inventory.coins;
    let before_job = gateway.gameplay().unwrap().active_profile();
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(NPC.to_le_bytes());
    menu.push(0);
    gateway.handle_at(&wrap(&menu), 1_000).unwrap();
    let mut choice = vec![26, 0, 10, 0];
    choice.extend(NPC.to_le_bytes());
    choice.extend(3_i32.to_le_bytes());
    let entered = gateway.handle_at(&wrap(&choice), 1_100).unwrap();
    let instance = entered
        .replies
        .iter()
        .filter_map(|packet| unwrap_reply(packet))
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[3..7].try_into().unwrap()))
        .expect("derby offer");
    assert!(gateway.gameplay().unwrap().derby_active());

    let cancelled = gateway.handle_at(&wrap(&[26, 0, 42, 0]), 1_200).unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(gateway.gameplay().unwrap().active_profile(), before_job);
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .inventory
            .coins,
        before_coins
    );
    assert!(
        cancelled
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[39, 0, 19]))
    );
    assert!(
        cancelled
            .nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 7, 0]))
    );

    let after = gateway.snapshot();
    gateway
        .handle_at(&wrap(&derby_quit_request(instance)), 1_300)
        .unwrap();
    assert_eq!(gateway.snapshot(), after);
}

#[test]
fn gateway_pending_game_cancel_retires_without_rewards() {
    let fixture = Fixture::new();
    let (mut gateway, _npc, instance) = fixture.active_derby();
    let before = gateway.snapshot();
    let before_job = before
        .gameplay
        .as_ref()
        .unwrap()
        .profiles
        .as_ref()
        .unwrap()
        .active;
    let phase = gateway.gameplay().unwrap().derby_phase();

    gateway
        .handle_at(&wrap(&derby_cancel_request(instance + 1)), 4_100)
        .unwrap();
    assert_eq!(gateway.snapshot(), before);
    assert_eq!(gateway.gameplay().unwrap().derby_phase(), phase);

    let cancelled = gateway
        .handle_at(&wrap(&derby_cancel_request(instance)), 4_200)
        .unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(
        gateway.gameplay().unwrap().active_profile(),
        Some(before_job)
    );
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .inventory
            .coins,
        before.gameplay.as_ref().unwrap().inventory.coins
    );
    assert!(
        cancelled
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[39, 0, 19]))
    );
    assert!(
        cancelled
            .nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 7, 0]))
    );
    let after = gateway.snapshot();
    gateway
        .handle_at(&wrap(&derby_cancel_request(instance)), 4_300)
        .unwrap();
    assert_eq!(gateway.snapshot(), after);
}

#[test]
fn unstarted_derby_offer_times_out_without_rewards() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    let before = gateway.snapshot();
    let before_job = before
        .gameplay
        .as_ref()
        .unwrap()
        .profiles
        .as_ref()
        .unwrap()
        .active;
    let before_coins = before.gameplay.as_ref().unwrap().inventory.coins;
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(NPC.to_le_bytes());
    menu.push(0);
    gateway.handle_at(&wrap(&menu), 1_000).unwrap();
    let mut choice = vec![26, 0, 10, 0];
    choice.extend(NPC.to_le_bytes());
    choice.extend(3_i32.to_le_bytes());
    gateway.handle_at(&wrap(&choice), 1_100).unwrap();

    gateway.tick_at(2_099).unwrap();
    assert!(gateway.gameplay().unwrap().derby_active());
    let timed_out = gateway.tick_at(2_100).unwrap();
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(
        gateway.gameplay().unwrap().active_profile(),
        Some(before_job)
    );
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .as_ref()
            .unwrap()
            .inventory
            .coins,
        before_coins
    );
    assert!(
        timed_out
            .replies
            .iter()
            .filter_map(|packet| unwrap_reply(packet))
            .any(|packet| packet.starts_with(&[39, 0, 19]))
    );
    assert!(
        timed_out
            .nearby
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 7, 0]))
    );
    let after = gateway.snapshot();
    gateway.tick_at(2_200).unwrap();
    assert_eq!(gateway.snapshot(), after);
}

#[test]
fn active_activity_refuses_derby_without_consuming_the_npc_menu() {
    let fixture = Fixture::new();
    let mut gateway = fixture.active("mining");
    let mut menu = vec![26, 0, 8, 0];
    menu.extend(NPC.to_le_bytes());
    menu.push(0);
    gateway.handle_at(&wrap(&menu), 1_000).unwrap();
    let choice = {
        let mut packet = vec![26, 0, 10, 0];
        packet.extend(NPC.to_le_bytes());
        packet.extend(3_i32.to_le_bytes());
        packet
    };
    let before = gateway.snapshot();
    let first = gateway.handle_at(&wrap(&choice), 1_100).unwrap();
    assert!(!first.replies.is_empty());
    assert_eq!(gateway.snapshot(), before);
    assert!(!gateway.gameplay().unwrap().derby_active());
    let retry = gateway.handle_at(&wrap(&choice), 1_200).unwrap();
    assert!(
        !retry.replies.is_empty(),
        "the same offered Derby choice must remain retryable"
    );
    assert_eq!(gateway.snapshot(), before);
    assert!(!gateway.gameplay().unwrap().derby_active());
}

#[test]
fn trade_requests_reach_the_binary_with_gameplay_loaded() {
    // W163: gameplay answers nothing for family 59, so only the handler label forwards it
    // to the gateway binary's trade route (otherwise it is rejected as unknown).
    let fixture = Fixture::new();
    for trace in [false, true] {
        let mut gateway = fixture.gateway(true, trace);
        let mut offer = vec![59, 0, 9];
        offer.extend(7u32.to_le_bytes());
        offer.extend([1, 0, 0, 0, 2, 0, 0, 0]);
        let out = gateway.handle_at(&wrap(&offer), NOW).unwrap();
        assert!(
            out.applications
                .iter()
                .any(|application| *application == offer),
            "59/9 must be forwarded, trace={trace}"
        );
    }
}

#[test]
fn t3_gateway_login_tunnel_and_bundle_lengths_are_atomic() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    for seed in [
        login(),
        wrap(&[13, 0]),
        wrap(&[10, 0]),
        wrap(&[52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
    ] {
        for (mutation, input) in hostile(&seed) {
            unchanged(&mut gateway, &input, &mutation);
        }
    }
    for input in [
        vec![0, 25, 255, 0, 255, 255, 255, 255, 255],
        // Group lengths (client GetVariableValue 0x010daea0): `FF 00 FF` short of its body,
        // `FF FF FF` + u32 70,000 with no body, and every truncated prefix.
        [vec![0, 25, 255, 0, 255], vec![0; 254]].concat(),
        vec![0, 25, 255, 255, 255, 0, 1, 17, 112, 0, 0],
        vec![0, 25, 255],
        vec![0, 25, 255, 0],
        vec![0, 25, 255, 255, 255],
        vec![0, 25, 255, 255, 255, 0, 1, 17],
        vec![0, 25, 2, 0, 25],
        vec![0; 1_048_577],
    ] {
        unchanged(&mut gateway, &input, "bundle/message limit");
    }
}

#[test]
fn t3_active_movies_reject_hostile_headers_and_out_of_phase_rewards() {
    let start = Instant::now();
    let fixture = Fixture::new();
    let mut count = 0;
    for kind in [
        "forging",
        "smelting",
        "cooking",
        "mining",
        "harvesting",
        "kart",
    ] {
        let mut gateway = fixture.active(kind);
        for (handler, seed) in seeds().into_iter().filter(|(h, _)| {
            matches!(
                *h,
                "movie" | "kart" | "inventory" | "class_selection" | "combat"
            )
        }) {
            // Exact Close is valid; hostile variants of Close must preserve the active run.
            for (mutation, input) in hostile(&seed) {
                unchanged(
                    &mut gateway,
                    &wrap(&input),
                    &format!("{kind} {handler} {mutation}"),
                );
                count += 1;
            }
            if seed != [26, 0, 42, 0] {
                unchanged(&mut gateway, &wrap(&seed), kind);
            }
        }
        // A client cannot claim a win while the server has only offered the movie.
        let state = gateway.snapshot().gameplay.unwrap();
        let instance = match kind {
            "forging" => state.forging.unwrap().active.unwrap().instance_id,
            "smelting" => state.smelting.unwrap().active.unwrap().instance_id,
            "cooking" => state.cooking.unwrap().active.unwrap().instance_id,
            "mining" => state.mining.unwrap().active.unwrap().instance_id,
            "harvesting" => state.harvesting.unwrap().active.unwrap().instance_id,
            _ => {
                let out = gateway
                    .handle_at(&wrap(&words(33, 9, &[1, 0, 0, 0])), NOW)
                    .unwrap();
                bounded(&out);
                assert!(
                    out.replies.iter().any(|p| p[7..].starts_with(&[33, 0])),
                    "valid kart registration failed after fuzz"
                );
                println!("W90 active kart routed, valid registration follows hostile inputs");
                continue;
            }
        };
        let game_id = match kind {
            "mining" => mining::MINING_GAME_ID,
            "harvesting" => 111,
            _ => read_json(&fixture.content.join(format!("{kind}.json")))["game_id"]
                .as_i64()
                .unwrap() as i32,
        };
        for event in [
            "OnGameWonMsg",
            "FRServer_GameWon\t0",
            "FRServer_GameWon\t4294967295",
            "FRServer_GameOver",
            "FRServer_ScoreSet\t4294967295",
            "OnPauseMsg\t2",
        ] {
            let mut input = osfr_tools::forging::payload_packet(instance, event).unwrap();
            input[11..15].copy_from_slice(&game_id.to_le_bytes());
            for _ in 0..2 {
                unchanged(&mut gateway, &wrap(&input), kind);
            }
        }
        let mut early_win =
            osfr_tools::forging::payload_packet(instance, "FRServer_GameWon\t0").unwrap();
        early_win[11..15].copy_from_slice(&game_id.to_le_bytes());
        for (label, input) in hostile(&early_win) {
            unchanged(
                &mut gateway,
                &wrap(&input),
                &format!("{kind} early win {label}"),
            );
            count += 1;
        }
        let mut valid = vec![39, 0, 5];
        for value in [instance, -1, -1] {
            valid.extend(value.to_le_bytes());
        }
        let before = gateway.snapshot().gameplay.unwrap().inventory;
        let out = gateway.handle_at(&wrap(&valid), NOW).unwrap();
        bounded(&out);
        assert!(
            out.gameplay_changed && !out.replies.is_empty(),
            "valid {kind} start failed after fuzz"
        );
        assert_eq!(
            gateway.snapshot().gameplay.unwrap().inventory,
            before,
            "start granted an early reward"
        );
        println!("W90 active {kind} routed, foreign ids and early reward replay rejected");
    }
    println!(
        "W90 active corpus: {count} mutations, elapsed {:?}",
        start.elapsed()
    );
    assert!(start.elapsed() < Duration::from_secs(60));
}

fn race_observations(snapshot: &Snapshot) -> u32 {
    snapshot
        .gameplay
        .as_ref()
        .unwrap()
        .kart_entry
        .as_ref()
        .unwrap()
        .race
        .as_ref()
        .unwrap()
        .observations
}

/// The same complete gateway snapshot with the two bulk-frame saved fields pinned to their fresh
/// values: the observation flood budget and the last weapon pose. A frame that only feeds those
/// must move nothing else in the saved state.
fn without_flood_state(snapshot: &Snapshot) -> Snapshot {
    let mut snapshot = snapshot.clone();
    let race = snapshot
        .gameplay
        .as_mut()
        .unwrap()
        .kart_entry
        .as_mut()
        .unwrap()
        .race
        .as_mut()
        .unwrap();
    race.observations = 0;
    race.weapon_pose = None;
    snapshot
}

#[test]
fn kart_timeout_refusal_is_save_inert_at_the_gateway() {
    let fixture = Fixture::new();
    let mut gateway = fixture.active("kart");
    let active = gateway
        .snapshot()
        .gameplay
        .as_ref()
        .unwrap()
        .kart_entry
        .as_ref()
        .unwrap()
        .active
        .clone()
        .unwrap();
    let guid = gateway.state().guid;

    assert!(
        !gateway
            .handle_at(
                &wrap(&words(33, 9, &[guid as u32, (guid >> 32) as u32, 0, 0])),
                NOW
            )
            .unwrap()
            .replies
            .is_empty()
    );
    let start_ms = NOW + 1_000;
    let mut start = words(
        41,
        108,
        &[active.context.encounter_id, active.context.instance_id],
    );
    start.extend(guid.to_le_bytes());
    gateway.handle_at(&wrap(&start), start_ms).unwrap();
    gateway.tick_at(start_ms + 20_000).unwrap();

    let race = gateway
        .snapshot()
        .gameplay
        .unwrap()
        .kart_entry
        .unwrap()
        .race
        .unwrap();
    assert_eq!(race.phase, osfr_tools::kart_service::RacePhase::Racing);
    let gate: [f32; 3] =
        read_json(&fixture.content.join("kart_entry.json"))["race"]["gates"][0]["position"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_f64().unwrap() as f32)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
    let mut claim = words(33, 7, &[guid as u32, (guid >> 32) as u32]);
    for value in gate {
        claim.extend(value.to_le_bytes());
    }
    claim.extend(1.0f32.to_le_bytes());

    let before = gateway.snapshot();
    let out = gateway
        .handle_at(
            &wrap(&claim),
            race.race_started_ms + u64::from(race.bounds.max_race_ms) + 1,
        )
        .unwrap();
    assert_eq!(out.applications, vec![claim]);
    assert!(out.replies.is_empty());
    assert!(
        !out.state_changed && !out.gameplay_changed,
        "a timeout refusal must not trigger a character save"
    );
    assert_eq!(
        gateway.snapshot(),
        before,
        "a timeout refusal must leave the complete gateway snapshot unchanged"
    );

    // The sibling `33/17` pickup claim carries the same 24-byte block and the same race window.
    // A post-window pickup still spends one observation slot that the saved race carries (a login
    // abandons the race, so this bounds only the race it belongs to) but must not trigger a
    // character save.
    let mut pickup = words(33, 17, &[guid as u32, (guid >> 32) as u32]);
    for _ in 0..3 {
        pickup.extend(0.0f32.to_le_bytes());
    }
    pickup.extend(1.0f32.to_le_bytes());
    let out = gateway
        .handle_at(
            &wrap(&pickup),
            race.race_started_ms + u64::from(race.bounds.max_race_ms) + 1,
        )
        .unwrap();
    assert_eq!(out.applications, vec![pickup]);
    assert!(out.replies.is_empty());
    assert!(
        !out.state_changed && !out.gameplay_changed,
        "a timed pickup refusal must not trigger a character save"
    );
    let after = gateway.snapshot();
    assert_eq!(
        race_observations(&after),
        1,
        "the routed timed frame must commit the spent observation slot"
    );
    assert_eq!(
        without_flood_state(&after),
        without_flood_state(&before),
        "a timed pickup refusal must move only the persisted observation budget"
    );

    // An in-window observation refusal is save-inert even though it commits the next budget slot.
    let mut refused = words(33, 17, &[guid as u32, (guid >> 32) as u32]);
    for value in [f32::NAN, 0.0, 0.0, 1.0] {
        refused.extend(value.to_le_bytes());
    }
    let out = gateway
        .handle_at(&wrap(&refused), race.race_started_ms + 1_000)
        .unwrap();
    assert_eq!(out.applications, vec![refused]);
    assert!(out.replies.is_empty());
    assert!(
        !out.state_changed && !out.gameplay_changed,
        "an in-window observation refusal must not trigger a character save"
    );
    let after = gateway.snapshot();
    assert_eq!(
        race_observations(&after),
        2,
        "the in-window refusal must commit its observation slot"
    );
    assert_eq!(
        without_flood_state(&after),
        without_flood_state(&before),
        "an in-window observation refusal must move only the persisted observation budget"
    );

    // An accepted `33/6` bulk observation is equally save-inert. The fixture keeps production's
    // weapon simulation on, so the frame also moves the saved pose: neither it nor the budget may
    // request a save.
    let mut bulk = words(33, 6, &[]);
    let mut body = vec![0u8; 85];
    for (index, value) in [1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
        .iter()
        .enumerate()
    {
        body[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (index, value) in race.last_position.iter().enumerate() {
        body[36 + index * 4..40 + index * 4].copy_from_slice(&value.to_le_bytes());
    }
    body[81..85].copy_from_slice(&1u32.to_le_bytes());
    bulk.extend(body);
    let out = gateway
        .handle_at(&wrap(&bulk), race.race_started_ms + 1_100)
        .unwrap();
    assert_eq!(out.applications, vec![bulk]);
    assert!(out.replies.is_empty());
    assert!(
        !out.state_changed && !out.gameplay_changed,
        "an accepted bulk observation must not trigger a character save"
    );
    let after = gateway.snapshot();
    assert_eq!(
        race_observations(&after),
        3,
        "the accepted bulk frame must commit its observation slot"
    );
    assert!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .kart_entry
            .as_ref()
            .unwrap()
            .race
            .as_ref()
            .unwrap()
            .weapon_pose
            .is_some(),
        "the production weapon simulation must move the committed pose"
    );
    assert_eq!(
        without_flood_state(&after),
        without_flood_state(&before),
        "an accepted bulk observation moves only the saved budget and weapon pose"
    );
}

#[test]
fn t3_application_requests_before_loaded_do_not_mutate_or_poison_entry() {
    let fixture = Fixture::new();
    let mut gateway = fixture.unloaded(true, false);
    for next in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
        let before = gateway.snapshot();
        for (handler, seed) in seeds()
            .into_iter()
            .filter(|(_, p)| !matches!(p[0], 125 | 126))
        {
            for cut in 0..=seed.len() {
                let result = catch_unwind(AssertUnwindSafe(|| {
                    gateway.handle_at(&wrap(&seed[..cut]), NOW)
                }));
                assert!(result.is_ok(), "pre-loaded panic {handler}/{cut}");
                if let Ok(out) = result.unwrap() {
                    bounded(&out);
                }
                assert!(
                    gateway.snapshot() == before,
                    "pre-loaded mutation {handler}/{cut}"
                );
            }
        }
        let out = gateway.handle_at(&next, NOW).unwrap();
        bounded(&out);
        assert!(
            !out.replies.is_empty() || out.entered_world,
            "valid world-entry follow-up stopped working"
        );
    }
    follow_up(&mut gateway);
}

#[test]
fn t3_non_opening_combat_inventory_and_quest_fallback_are_routed() {
    let fixture = Fixture::new();
    // Remove only files in this fixture, selecting the existing non-opening path.
    for file in [
        "opening.json",
        "smelting.json",
        "forging-campaign.json",
        "cooking-campaign.json",
        "cooking.json",
        "mining.json",
        "harvesting.json",
        "kart_entry.json",
    ] {
        fs::remove_file(fixture.content.join(file)).unwrap();
    }
    let mut gateway = fixture.gateway(true, false);
    assert!(!gateway.gameplay().unwrap().is_opening());
    let mut count = 0;
    for (handler, seed) in seeds() {
        for (mutation, input) in hostile(&seed) {
            unchanged(
                &mut gateway,
                &wrap(&input),
                &format!("fallback {handler} {mutation}"),
            );
            count += 1;
        }
    }
    println!("W90 non-opening fallback: {count} mutations");
}

#[test]
fn t3_pending_chest_survives_malformed_use_and_valid_reward_cannot_replay() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    let check = wrap(&words(42, 16, &[900_001]));
    let use_item = words(42, 15, &[900_001]);
    let before = gateway.snapshot();
    let out = gateway.handle_at(&check, NOW).unwrap();
    assert!(
        out.replies
            .iter()
            .any(|p| p[7..].starts_with(&[38, 0, 30, 0]))
    );
    assert_eq!(gateway.snapshot(), before);
    // Every malformed prefix/trailer must retain the pending confirmation, too.
    for (label, input) in hostile(&use_item)
        .into_iter()
        .filter(|(label, _)| label.starts_with("cut") || label == "trailing")
    {
        unchanged(&mut gateway, &wrap(&input), &label);
    }
    let out = gateway.handle_at(&wrap(&use_item), NOW).unwrap();
    assert!(
        out.gameplay_changed,
        "valid confirmation did not survive hostile requests"
    );
    assert_eq!(
        gateway
            .snapshot()
            .gameplay
            .unwrap()
            .inventory
            .items
            .iter()
            .filter(|i| i.definition_id == LOOT)
            .map(|i| i.count)
            .sum::<u32>(),
        1
    );
    unchanged(&mut gateway, &wrap(&use_item), "replayed chest reward");
    follow_up(&mut gateway);
}

#[test]
fn t3_snapshot_oracle_canary_detects_a_deliberate_economy_change() {
    let fixture = Fixture::new();
    let before = fixture.gateway(true, false).snapshot();
    let mut broken = before.clone();
    broken.gameplay.as_mut().unwrap().inventory.coins += 1;
    assert!(
        catch_unwind(|| assert!(broken == before, "W90 deliberate canary")).is_err(),
        "snapshot oracle failed to detect mutation"
    );
}

fn npc(sub: u16, choice: Option<u32>) -> Vec<u8> {
    let mut bytes = words(26, sub, &[]);
    bytes.extend(NPC.to_le_bytes());
    if let Some(choice) = choice {
        bytes.extend(choice.to_le_bytes());
    } else {
        bytes.push(0);
    }
    bytes
}

#[test]
fn t3_open_shop_rejects_hostile_buy_then_accepts_valid_follow_up() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    gateway.handle_at(&wrap(&npc(8, None)), NOW).unwrap();
    let opened = gateway.handle_at(&wrap(&npc(10, Some(2))), NOW).unwrap();
    assert!(
        opened
            .replies
            .iter()
            .any(|p| p[7..].starts_with(&[66, 0, 42, 0])),
        "fixture shop did not open"
    );
    for (label, input) in hostile(&npc(10, Some(u32::MAX))) {
        unchanged(
            &mut gateway,
            &wrap(&input),
            &format!("shared NPC state {label}"),
        );
    }
    let mut buy = words(165, 4, &[]);
    buy.extend(NPC.to_le_bytes());
    // Client writer 0x00c75ed0: record id, definition, tint, then quantity.
    for field in [71001u32, 8, 0, 1] {
        buy.extend(field.to_le_bytes());
    }
    // Length corruption and an excessive count must not close/poison shared shop state.
    for cut in 0..buy.len() {
        unchanged(&mut gateway, &wrap(&buy[..cut]), "shop truncation");
    }
    let mut huge = buy.clone();
    huge[24..28].fill(0xff);
    unchanged(&mut gateway, &wrap(&huge), "shop u32::MAX quantity");
    buy[12..16].copy_from_slice(&71002u32.to_le_bytes()); // A fresh transaction after a valid refusal.
    let before = gateway.snapshot().gameplay.unwrap().inventory.coins;
    let out = gateway.handle_at(&wrap(&buy), NOW).unwrap();
    bounded(&out);
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins,
        before - 15,
        "shared shop state poisoned"
    );
    unchanged(&mut gateway, &wrap(&buy), "replayed buy transaction");
}

#[test]
fn t3_quest_accept_turnin_and_reward_replays_are_idempotent() {
    use osfr_tools::quest_engine::{CharacterEligibility, QuestEngine, QuestEvent};
    let fixture = Fixture::new();
    let eligibility = CharacterEligibility {
        player_level: 1,
        nonmember_quest_limit: 20,
        member_quest_limit: 50,
        ..CharacterEligibility::default()
    };
    let mut engine =
        QuestEngine::from_split_content(&fixture.content, 1, eligibility.clone(), None).unwrap();
    assert!(engine.offer(QUEST, NPC).changed);
    let mut state = fixture.state();
    state.npc_quests = Some(engine.state().clone());
    let mut character = Gateway::reference_state(&fixture.reference).unwrap();
    character.position = [0.; 3];
    let mut gateway = Gateway::load_gameplay(
        &fixture.reference,
        character.clone(),
        &fixture.content,
        Some(state),
    )
    .unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, NOW).unwrap();
    }
    let mut accept = vec![49, 0, 2, 0, 0, 0];
    accept.extend(QUEST.to_le_bytes());
    accept.push(1);
    for (label, input) in hostile(&accept) {
        unchanged(
            &mut gateway,
            &wrap(&input),
            &format!("quest accept {label}"),
        );
    }
    let out = gateway.handle_at(&wrap(&accept), NOW).unwrap();
    assert!(out.gameplay_changed);
    unchanged(&mut gateway, &wrap(&accept), "quest accept replay");
    let mut engine = QuestEngine::from_split_content(
        &fixture.content,
        1,
        eligibility,
        gateway.snapshot().gameplay.unwrap().npc_quests,
    )
    .unwrap();
    assert!(
        engine
            .on_event(QuestEvent::TalkToNpc { npc_id: NPC })
            .unwrap()
            .changed
    );
    engine.begin_turnin(QUEST, NPC);
    assert_eq!(engine.state().pending_turnin, Some(QUEST));
    let mut state = gateway.snapshot().gameplay.unwrap();
    state.npc_quests = Some(engine.state().clone());
    let mut gateway =
        Gateway::load_gameplay(&fixture.reference, character, &fixture.content, Some(state))
            .unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    for packet in [login(), wrap(&[13, 0]), wrap(&[10, 0])] {
        gateway.handle_at(&packet, NOW).unwrap();
    }
    let mut finish = vec![49, 0, 14, 0, 0, 0];
    finish.extend(QUEST.to_le_bytes());
    for (label, input) in hostile(&finish) {
        unchanged(
            &mut gateway,
            &wrap(&input),
            &format!("quest turnin {label}"),
        );
    }
    let before = gateway.snapshot().gameplay.unwrap().inventory.coins;
    gateway.handle_at(&wrap(&finish), NOW).unwrap();
    assert_eq!(
        gateway.snapshot().gameplay.unwrap().inventory.coins,
        before + 7
    );
    for _ in 0..3 {
        unchanged(&mut gateway, &wrap(&finish), "quest reward replay");
    }
    // The parser rejects the entire combined frame before applying a valid first child.
    unchanged(
        &mut gateway,
        &bundle(&[wrap(&words(42, 16, &[900_001])), vec![5, 0]]),
        "valid + invalid bundle child",
    );
}

#[test]
fn finding_1_unknown_chest_use_must_preserve_pending_confirmation() {
    let fixture = Fixture::new();
    let mut gateway = fixture.gateway(true, false);
    gateway
        .handle_at(&wrap(&words(42, 16, &[900_001])), NOW)
        .unwrap();
    unchanged(
        &mut gateway,
        &wrap(&words(42, 15, &[900_100])),
        "unknown chest instance",
    );
    let result = gateway
        .handle_at(&wrap(&words(42, 15, &[900_001])), NOW)
        .unwrap();
    assert!(
        result.gameplay_changed,
        "unknown instance Use consumed the pending valid Check (mystery_chest_loot.rs pending.take())"
    );
}
