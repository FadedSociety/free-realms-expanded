//! Developer-only `realm goto` relocation against the private opening campaign.
//! No native client, release build, user save or launcher script is touched.
use osfr_tools::{
    dev_teleport::STANDOFF,
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
    opening::OpeningEngine,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const WORLD_ZONE: u32 = 0;
const MAE_GUID: u64 = 5_715_144_128_442_925_060;
const QUEEN_GUID: u64 = 5_715_144_128_442_925_063;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}

fn inner(reply: &[u8]) -> &[u8] {
    assert_eq!(&reply[..3], &[5, 0, 1], "reply is not a tunneled message");
    let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
    assert_eq!(length, reply.len() - 7);
    &reply[7..]
}

/// Accepts both gateway replies (tunneled) and direct gameplay replies.
fn body(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn chat_inner(text: &str) -> Vec<u8> {
    let mut p = vec![15, 0, 1, 0, 0, 0];
    p.extend(1u64.to_le_bytes());
    p.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        p.extend([0; 12]);
        p.extend(0u32.to_le_bytes());
        p.extend(0u32.to_le_bytes());
    }
    p.extend((text.len() as u32).to_le_bytes());
    p.extend(text.as_bytes());
    p.extend([0; 28]);
    p
}

fn say(gateway: &mut Gateway, text: &str) -> osfr_tools::gateway::GatewayOutcome {
    gateway.handle_at(&wrap(&chat_inner(text)), 2_000).unwrap()
}

fn interaction(guid: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 10, 0];
    b.extend(guid.to_le_bytes());
    b.extend(1i32.to_le_bytes());
    b
}

fn enter_world(gateway: &mut Gateway) {
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        login.extend((s.len() as u32).to_le_bytes());
        login.extend(s.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    gateway.handle_at(&login, 1_000).unwrap();
    gateway
        .handle_at(&wrap(&13u16.to_le_bytes()), 1_000)
        .unwrap();
    gateway
        .handle_at(&wrap(&10u16.to_le_bytes()), 1_000)
        .unwrap();
}

fn assert_position_close(left: [f32; 3], right: [f32; 3]) {
    for (a, b) in left.into_iter().zip(right) {
        assert!((a - b).abs() < 0.01, "{a} vs {b}");
    }
}

fn paths() -> (PathBuf, PathBuf) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    (
        std::env::var_os("OSFR_DEV_TELEPORT_CONTENT")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("opening-anvil-placement-v1/content")),
        std::env::var_os("OSFR_GATEWAY_REFERENCE")
            .map(PathBuf::from)
            .unwrap_or_else(|| base.join("rust-gateway-reference")),
    )
}

fn content_actor(content: &Path, guid: u64) -> Value {
    let opening: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    opening["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["guid"].as_u64() == Some(guid))
        .expect("campaign actor")
        .clone()
}

fn content_actor_named(content: &Path, name: &str) -> Value {
    let opening: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    opening["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["name"].as_str() == Some(name))
        .expect("named campaign actor")
        .clone()
}

fn horizontal(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn float3(value: &Value) -> [f32; 3] {
    let a = value.as_array().unwrap();
    [
        a[0].as_f64().unwrap() as f32,
        a[1].as_f64().unwrap() as f32,
        a[2].as_f64().unwrap() as f32,
    ]
}

fn packet_f32(packet: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(packet[offset..offset + 4].try_into().unwrap())
}

fn chat_text(replies: &[Vec<u8>]) -> String {
    let mut text = String::new();
    for reply in replies {
        let inner = body(reply);
        if inner.starts_with(&[15, 0, 3, 0]) {
            let length = u32::from_le_bytes(inner[4..8].try_into().unwrap()) as usize;
            text.push_str(std::str::from_utf8(&inner[8..8 + length]).unwrap());
            text.push('\n');
        }
    }
    text
}

/// Logs in a character standing at the Queen, matching the live campaign save.
fn fixture(dev_tools: bool) -> (Gateway, PathBuf, PathBuf) {
    let (content, reference) = paths();
    assert!(content.join("opening.json").is_file());
    let queen = content_actor(&content, QUEEN_GUID);
    assert_eq!(queen["zone_id"].as_u64(), Some(u64::from(WORLD_ZONE)));
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = WORLD_ZONE;
    character.position = float3(&queen["position"]);
    character.rotation = [1.0, 0.0, 0.0];
    let state = Gameplay::load(&content, None).unwrap().state();
    let mut gateway = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    enter_world(&mut gateway);
    gateway.set_dev_tools(dev_tools);
    (gateway, content, reference)
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_tools_stay_off_until_the_launcher_opts_in() {
    let (mut gateway, content, _reference) = fixture(false);
    let mae = content_actor(&content, MAE_GUID);
    let mae_position = float3(&mae["position"]);
    let before = gateway.snapshot();

    let outcome = say(&mut gateway, "realm goto mae farnum");

    assert!(
        !outcome
            .replies
            .iter()
            .map(|reply| inner(reply))
            .any(|inner| inner.starts_with(&[38, 0, 12, 0])),
        "developer relocation must stay off by default"
    );
    assert_eq!(gateway.snapshot(), before);
    assert!(gateway.state().position != mae_position);
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_teleport_moves_to_named_entity_and_preserves_progress() {
    let (mut gateway, content, _reference) = fixture(true);
    let mae = content_actor(&content, MAE_GUID);
    let mae_position = float3(&mae["position"]);
    let before = gateway.snapshot();

    let outcome = say(&mut gateway, "realm goto mae farnum");

    let packet = outcome
        .replies
        .iter()
        .map(|reply| inner(reply))
        .find(|inner| inner.starts_with(&[38, 0, 12, 0]))
        .expect("relocation must emit the pinned 38/12 location-update packet");
    let packet_index = outcome
        .replies
        .iter()
        .position(|reply| inner(reply).starts_with(&[38, 0, 12, 0]))
        .unwrap();
    let reply_index = outcome
        .replies
        .iter()
        .position(|reply| inner(reply).starts_with(&[15, 0, 3, 0]))
        .unwrap();
    assert!(
        packet_index < reply_index,
        "the pinned handler sends the location packet before the chat reply"
    );
    assert_eq!(packet.len(), 38, "writer-exact packet size");
    assert_eq!(packet[36], 1, "Teleport flag");
    assert_eq!(packet[37], 0, "Unknown byte");
    let position = gateway.state().position;
    let rotation = gateway.state().rotation;
    for (offset, value) in position.into_iter().enumerate() {
        assert_eq!(packet_f32(packet, 4 + offset * 4), value);
    }
    assert_eq!(packet_f32(packet, 16), 1.0, "position W");
    for (offset, value) in rotation.into_iter().enumerate() {
        assert_eq!(packet_f32(packet, 20 + offset * 4), value);
    }
    assert_eq!(packet_f32(packet, 32), 0.0, "facing W");
    let dx = mae_position[0] - position[0];
    let dz = mae_position[2] - position[2];
    let distance = (dx * dx + (mae_position[1] - position[1]).powi(2) + dz * dz).sqrt();
    assert!(
        (distance - STANDOFF).abs() < 0.01,
        "expected the bounded standoff, got {distance}"
    );
    assert!(distance < mae["interaction_radius"].as_f64().unwrap() as f32);
    let facing = (rotation[0].powi(2) + rotation[2].powi(2)).sqrt();
    assert!((facing - 1.0).abs() < 1e-6);
    assert!(
        rotation[0] * dx + rotation[2] * dz > 0.0,
        "must face the actor"
    );
    assert!(position.iter().all(|v| v.is_finite()));
    assert!(chat_text(&outcome.replies).contains("Developer teleport: Mae Farnum"));
    assert_eq!(gateway.snapshot().gameplay, before.gameplay);
    assert_ne!(before.character.position, position);

    // The relocation refreshes the interaction origin, so talking to Mae works
    // immediately without a dummy movement key. Standing at the Queen without
    // the relocation is the negative control for the same request.
    let (mut queen, _queen_content, _queen_reference) = fixture(true);
    let at_queen = queen
        .handle_at(&wrap(&interaction(MAE_GUID)), 3_000)
        .unwrap();
    assert!(
        chat_text(&at_queen.replies).contains("Move closer"),
        "the unrelocated character is out of range: {}",
        chat_text(&at_queen.replies)
    );
    let at_mae = gateway
        .handle_at(&wrap(&interaction(MAE_GUID)), 3_000)
        .unwrap();
    let text = chat_text(&at_mae.replies);
    assert!(
        !text.contains("Move closer"),
        "the relocation must refresh the interaction origin: {text}"
    );
}

#[test]
fn content_npcs_support_filtered_listing_name_and_guid_teleports_with_visibility_refresh() {
    let (mut gateway, _content, _reference) = fixture(true);

    let listing = chat_text(&say(&mut gateway, "realm goto list sam").replies);
    assert!(
        listing.contains("Samantha"),
        "filtered content list: {listing}"
    );
    assert!(
        listing.contains("9000000189"),
        "filtered content list: {listing}"
    );

    let moved = say(&mut gateway, "realm goto samantha");
    assert!(
        chat_text(&moved.replies).contains("Developer teleport: Samantha"),
        "case-insensitive content name lookup: {}",
        chat_text(&moved.replies)
    );

    let remote = say(&mut gateway, "realm goto 100000001308");
    assert!(
        remote
            .replies
            .iter()
            .filter_map(|reply| { osfr_tools::startup_world::parse_add_npc(body(reply)).ok() })
            .any(|npc| npc.actor_id == 100000001308),
        "relocation must immediately add the target NPC to visibility"
    );
    assert!(
        chat_text(&remote.replies).contains("Developer teleport: Gus Galewind"),
        "content GUID lookup: {}",
        chat_text(&remote.replies)
    );
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_teleport_arrival_keeps_the_target_closest_in_the_farm_cluster() {
    let (mut gateway, content, _reference) = fixture(true);
    let mae = content_actor(&content, MAE_GUID);
    let table = content_actor_named(&content, "Cooking Table");
    let mae_position = float3(&mae["position"]);
    let table_position = float3(&table["position"]);

    say(&mut gateway, "realm goto mae farnum");
    let arrival = gateway.state().position;
    let to_target = horizontal(arrival, mae_position);
    let to_table = horizontal(arrival, table_position);
    eprintln!(
        "mae={mae_position:?} cooking_table={table_position:?} arrival={arrival:?} \
         to_target={to_target:.3} to_table={to_table:.3}"
    );
    assert!((to_target - STANDOFF).abs() < 0.01, "{to_target}");
    assert!(
        to_table >= to_target + 0.5,
        "the arrival must not hand the prompt to the Cooking Table: \
         target={to_target:.3} table={to_table:.3}"
    );

    // The whole loaded zone-0 cluster must stay at least the 0.5m margin away,
    // so no other farm resource can win the prompt either.
    let actors: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    for actor in actors["actors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["zone_id"].as_u64() == Some(0) && a["guid"].as_u64() != Some(MAE_GUID))
    {
        let distance = horizontal(arrival, float3(&actor["position"]));
        assert!(
            distance >= to_target + 0.5,
            "{} at {distance:.3} beats target {to_target:.3} from {arrival:?}",
            actor["name"]
        );
    }
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_teleport_list_ambiguity_unknown_and_malformed_targets_never_move() {
    let (mut gateway, _content, _reference) = fixture(true);
    let before = gateway.snapshot();

    let list = say(&mut gateway, "realm goto list");
    let text = chat_text(&list.replies);
    assert!(text.contains("Mae Farnum"), "{text}");
    assert!(text.contains(&MAE_GUID.to_string()), "{text}");
    assert_eq!(gateway.snapshot(), before);

    // Three zone 0 fires make the bare name ambiguous; the cave-only name,
    // an unused GUID, an empty argument and a name with extra text refuse.
    for query in ["fire", "gloamling", "999999", "", "mae farnum extra"] {
        let outcome = say(&mut gateway, &format!("realm goto {query}"));
        let text = chat_text(&outcome.replies);
        assert!(!text.is_empty(), "{query} must answer");
        assert_eq!(gateway.snapshot(), before, "{query} must not mutate");
    }

    // A `goto` prefix without a keyword boundary is an ordinary unknown
    // command: it must not relocate, and the opening handler still answers.
    for query in ["gotomae", "gotolist"] {
        let outcome = say(&mut gateway, &format!("realm {query}"));
        assert_eq!(gateway.snapshot(), before, "{query} must not mutate");
        assert!(
            !chat_text(&outcome.replies).contains("Developer teleport"),
            "{query} must not run the developer command"
        );
    }

    // The same guard applies to a replayed relocation: it is idempotent.
    let relocated = say(&mut gateway, "realm goto 5715144128442925060");
    assert!(chat_text(&relocated.replies).contains("Mae Farnum"));
    let moved = gateway.snapshot();
    let replay = say(&mut gateway, "realm goto mae farnum");
    assert!(chat_text(&replay.replies).contains("Developer teleport"));
    // Re-normalizing the already-relocated origin is idempotent to f32 rounding.
    assert_position_close(gateway.state().position, moved.character.position);
    assert_position_close(gateway.state().rotation, moved.character.rotation);
    assert_eq!(gateway.snapshot().gameplay, moved.gameplay);
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_teleport_refuses_live_encounter_and_knockout() {
    let (content, _reference) = paths();
    let world =
        serde_json::from_slice::<Value>(&fs::read(content.join("encounters.json")).unwrap())
            .unwrap();
    let dungeon = &world["dungeons"][0];
    assert_eq!(dungeon["id"].as_u64(), Some(411));
    let enemies: Vec<Value> = dungeon["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            json!({
                "guid": a["guid"],
                "health": a["max_health"],
                "position": a["position"],
            })
        })
        .collect();
    let return_location = json!({
        "guid": 1,
        "zone_id": 0,
        "position": world["world"]["position"],
        "rotation": [1.0, 0.0, 0.0],
    });
    let character = {
        let queen = content_actor(&content, QUEEN_GUID);
        let c = osfr_tools::character_store::CharacterState {
            guid: 1,
            zone_id: WORLD_ZONE,
            position: float3(&queen["position"]),
            rotation: [1.0, 0.0, 0.0],
        };
        c.validate().unwrap();
        c
    };

    for (phase, health, expected) in [
        ("fighting", 775_u32, "encounter"),
        ("lost", 0_u32, "knocked out"),
    ] {
        let mut value =
            serde_json::to_value(Gameplay::load(&content, None).unwrap().state()).unwrap();
        value["encounter"]["active"] = json!({
            "encounter_id": 411,
            "instance_id": 1,
            "phase": phase,
            "return_location": return_location.clone(),
            "enemies": enemies.clone(),
        });
        value["encounter"]["player_health"] = json!(health);
        value["encounter"]["next_instance_id"] = json!(2);
        let state: GameplayState = serde_json::from_value(value).unwrap();
        let mut game = Gameplay::load(&content, Some(state)).unwrap();
        game.set_dev_tools(true);
        let before = game.state();

        let outcome = game
            .handle_in_world(&chat_inner("realm goto mae farnum"), &character, 2_000)
            .unwrap();

        let text = chat_text(&outcome.replies);
        assert!(text.contains(expected), "{phase}: {text}");
        assert!(outcome.relocation.is_none(), "{phase}");
        assert_eq!(game.state(), before, "{phase} must not mutate");
    }
}

#[test]
#[ignore = "requires private opening campaign and gateway reference; no native launch"]
fn private_dev_teleport_bundle_is_atomic_and_restart_keeps_the_position() {
    let (mut gateway, content, reference) = fixture(true);
    let mae = content_actor(&content, MAE_GUID);
    let mae_position = float3(&mae["position"]);
    let before = gateway.snapshot();

    // A relocation bundled with a semantically invalid child must not commit.
    let mut invalid_equip = vec![42, 0, 3, 0];
    invalid_equip.extend((-1i32).to_le_bytes());
    invalid_equip.extend([0; 8]);
    let children = [
        wrap(&chat_inner("realm goto mae farnum")),
        wrap(&invalid_equip),
    ];
    let mut bundle = vec![0, 25];
    for child in children {
        assert!(child.len() < 255);
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(gateway.handle_at(&bundle, 2_500).is_err());
    assert_eq!(gateway.snapshot(), before);

    say(&mut gateway, "realm goto mae farnum");
    let moved = gateway.snapshot();
    assert_ne!(moved.character.position, before.character.position);
    let dx = mae_position[0] - moved.character.position[0];
    let dz = mae_position[2] - moved.character.position[2];
    assert!((dx * dx + dz * dz).sqrt() < STANDOFF + 0.01);

    let directory = std::env::temp_dir().join(format!("osfr-dev-teleport-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("snapshot.json");
    game_store::save(&path, &moved).unwrap();
    let loaded = game_store::load_or_create(&path, moved.character.clone()).unwrap();
    assert_eq!(loaded, moved);
    let mut fresh = Gateway::load(&reference, loaded.character.clone()).unwrap();
    fresh
        .enable_gameplay(&content, loaded.gameplay.clone())
        .unwrap();
    assert_eq!(fresh.state(), &loaded.character);
    assert_eq!(fresh.snapshot().gameplay, loaded.gameplay);
    // Entering the world normally may fire position rules for the relocated
    // spot; the developer command itself must still work after the restart.
    enter_world(&mut fresh);
    fresh.set_dev_tools(true);
    let after_restart = say(&mut fresh, "realm goto mae farnum");
    assert!(chat_text(&after_restart.replies).contains("Developer teleport"));
    assert_position_close(fresh.state().position, loaded.character.position);
    let dx = mae_position[0] - fresh.state().position[0];
    let dz = mae_position[2] - fresh.state().position[2];
    assert!((dx * dx + dz * dz).sqrt() < STANDOFF + 0.01);
    fs::remove_file(path).unwrap();
    fs::remove_dir(directory).unwrap();
}

#[test]
#[ignore = "requires private opening campaign; no native launch"]
fn private_dev_teleport_location_setter_is_progress_free_but_visit_rules_are_not() {
    let (content, _reference) = paths();
    let opening: Value =
        serde_json::from_slice(&fs::read(content.join("opening.json")).unwrap()).unwrap();
    let discovery = &opening["discoveries"][0];
    let zone = discovery["zone_id"].as_u64().unwrap() as u32;
    let position = float3(&discovery["position"]);

    // The ordinary visit path records the discovery and its message.
    let mut visited = OpeningEngine::load(&content, None).unwrap();
    let before_visit = visited.state().clone();
    let outcome = visited.on_position(zone, position).unwrap();
    assert!(outcome.changed, "the visit must fire the discovery");
    assert!(
        !outcome.messages.is_empty(),
        "the visit must explain itself"
    );
    assert_ne!(visited.state(), &before_visit);

    // The location-only setter used by developer relocation moves the origin
    // without touching any persisted opening progress.
    let mut relocated = OpeningEngine::load(&content, None).unwrap();
    let before = relocated.state().clone();
    relocated
        .set_location_without_events(zone, position)
        .unwrap();
    assert_eq!(relocated.state(), &before);
    assert!(
        relocated
            .set_location_without_events(zone, [f32::NAN, 0.0, 0.0])
            .is_err()
    );
    assert_eq!(relocated.state(), &before);
}
