//! T507: passive content NPCs play the per-model ambient animation row, and
//! every viewer that knows the actor receives the same 35/8 (client header
//! reader `0x008d6830`, SetAnimation reader `0x008e5dd0`). Field actors are
//! excluded from the passive visibility table, so their model row never fires.
use super::*;
use serde_json::Value;

const CHUGAWUG_MODEL: u64 = 13;
const FIELD_ENEMY_MODEL: u64 = 203;

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// One bundled extra spawn of `model`, its actor GUID and position.
fn spawn_of_model(document: &Value, model: u64) -> (u64, [f32; 3]) {
    let row = document["records"]
        .as_array()
        .expect("extra rows")
        .iter()
        .find(|row| row["model_id"].as_u64() == Some(model))
        .unwrap_or_else(|| panic!("content has a model {model} extra spawn"));
    (
        row["spawn_guid"].as_u64().expect("a spawn GUID"),
        serde_json::from_value(row["position"].clone()).expect("an XYZ position"),
    )
}

/// A field enemy whose client model has an ambient row, so the exclusion is
/// observable: without the visibility exclusion it would fire.
fn field_enemy_with_ambient_model(document: &Value) -> (u64, [f32; 3]) {
    let row = document["enemies"]
        .as_array()
        .expect("field enemies")
        .iter()
        .find(|row| row["model_id"].as_u64() == Some(FIELD_ENEMY_MODEL))
        .expect("a field enemy uses an ambient model");
    (
        row["guid"].as_u64().expect("an enemy GUID"),
        serde_json::from_value(row["position"].clone()).expect("an XYZ position"),
    )
}

/// A viewer sender that never retries inside one test run: no client ACKs the
/// frames, so the shared tick's own retry window would otherwise exhaust them.
/// `max_queued_bytes` is the reliable queue's size; 1 refuses every frame.
fn sender(max_queued_bytes: usize) -> ReliableSender {
    ReliableSender::new(
        Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
        SenderConfig {
            datagram_limit: 60_000,
            window: 4_096,
            max_message: MAX_MESSAGE,
            max_queued_bytes,
            retry_ms: 1_000_000,
            max_retries: 40,
        },
    )
    .unwrap()
}

/// Load a real gameplay gateway, enter the world, and put the session in the
/// open-world instance at `position`, ready. The session GUID owns membership;
/// the gateway's reference character GUID owns the saved gameplay (F213).
fn load_world_view(server: &mut Server, key: SessionKey, position: [f32; 3], version: &str) {
    let content = server
        .options
        .content
        .clone()
        .expect("the field rig loads gameplay content");
    let mut character = server.sessions[&key].gateway.state().clone();
    character.zone_id = 0; // World::OPEN_WORLD_ZONE (crate-private).
    character.position = position;
    character.rotation = [0.0, 0.0, 1.0];
    let login_guid = character.guid;
    let mut gateway = Gateway::load_gameplay(&server.options.reference, character, &content, None)
        .expect("the open-world gameplay loads");
    gateway.take_shared_fields();
    let now = 1_000;
    gateway
        .handle_at(
            &super::f160_field_views::f213_login(login_guid, version, "t507-ambient"),
            now,
        )
        .unwrap();
    gateway
        .handle_at(&wrap_application(&[13, 0]), now + 1)
        .unwrap();
    gateway
        .handle_at(&wrap_application(&[10, 0]), now + 2)
        .expect("the world entry reaches the loaded event");
    let guid = server.sessions[&key].character_guid;
    let session = server.sessions.get_mut(&key).unwrap();
    session.gateway = gateway;
    session.sender = sender(32 * 1024 * 1024);
    let pose = WorldPose {
        position: [position[0], position[1], position[2], 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
    };
    let change = server
        .membership
        .prepare_membership(guid, 0, Some(0), pose, true, now + 3)
        .expect("the open world accepts the viewer");
    server.membership.commit_membership(change);
}

/// Every 35/8 body for `guid`, as `(guid, animation id)`.
fn ambient_for(bodies: &[Vec<u8>], guid: u64) -> Vec<(u64, i32)> {
    bodies
        .iter()
        .filter(|body| body.starts_with(&[35, 0, 8, 0]) && body.len() == 21)
        .filter_map(|body| {
            let actor = u64::from_le_bytes(body[4..12].try_into().unwrap());
            (actor == guid).then(|| (actor, i32::from_le_bytes(body[12..16].try_into().unwrap())))
        })
        .collect()
}

fn bodies(server: &mut Server, key: SessionKey, now: u64) -> Vec<Vec<u8>> {
    super::f160_field_views::f213_bodies(&mut server.sessions.get_mut(&key).unwrap().sender, now)
}

#[test]
fn t507_ambient_animations_follow_the_model_table_for_every_viewer() {
    if !super::tests::test_support::bin_test_target("t507") {
        return;
    }
    if !super::t020_group_routing::fixtures_present() {
        eprintln!("skip t507-ambient: the private lab fixtures are absent");
        return;
    }
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extras = read_json(&content.join("npc_spawns_extra.json"));
    let enemies = read_json(&content.join("field_enemies.json"));
    let (actor, actor_at) = spawn_of_model(&extras, CHUGAWUG_MODEL);
    let (field_enemy, field_at) = field_enemy_with_ambient_model(&enemies);
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("t507-ambient");
    let manifest: Value =
        serde_json::from_slice(&fs::read(server.options.reference.join("manifest.json")).unwrap())
            .unwrap();
    let version = manifest["client_version"].as_str().unwrap().to_owned();

    // Two viewers next to the chugawug row; the third stands beside a field
    // enemy whose model has an ambient row but is excluded from visibility.
    load_world_view(&mut server, keys[0], actor_at, &version);
    load_world_view(&mut server, keys[1], actor_at, &version);
    load_world_view(&mut server, keys[2], field_at, &version);

    let table = {
        let game = server.sessions[&keys[0]].gateway.gameplay().unwrap();
        assert!(game.npc_ambient().is_some(), "the ambient table loads");
        assert_eq!(game.npc_model(actor), Some(CHUGAWUG_MODEL as u32));
        assert_eq!(
            game.npc_model(field_enemy),
            None,
            "a field enemy is excluded from the passive visibility table"
        );
        game.npc_ambient().unwrap().clone()
    };

    // First shared tick reconciles every viewer's known set.
    server.tick_shared_fields(2_000);
    assert!(
        server.sessions[&keys[0]].known_npcs.contains(&actor)
            && server.sessions[&keys[1]].known_npcs.contains(&actor),
        "both viewers know the chugawug"
    );
    assert!(
        !server.sessions[&keys[2]].known_npcs.contains(&actor),
        "the distant viewer does not know the chugawug"
    );
    for key in keys {
        let _ = bodies(&mut server, key, 2_000);
    }

    let mut first = Vec::new();
    let mut second = Vec::new();
    let mut first_packet = None;
    for now in (2_050..=15_000).step_by(50) {
        server.tick_shared_fields(now);
        let seen = bodies(&mut server, keys[0], now);
        first.extend(ambient_for(&seen, actor));
        let packet = seen.iter().find(|body| {
            body.starts_with(&[35, 0, 8, 0])
                && body.len() == 21
                && u64::from_le_bytes(body[4..12].try_into().unwrap()) == actor
        });
        if first_packet.is_none() {
            first_packet = packet.cloned();
        }
        second.extend(ambient_for(&bodies(&mut server, keys[1], now), actor));
        assert!(
            ambient_for(&bodies(&mut server, keys[2], now), actor).is_empty(),
            "a viewer that does not know the actor receives none"
        );
    }
    assert!(
        !first.is_empty(),
        "the chugawug fires within two of its periods"
    );
    assert_eq!(first, second, "every viewer sees the same guid and id");

    // The wire body is the reader-derived 21 bytes: header (0x008d6830), GUID,
    // animation id, unknown zero and flags (0x008e5dd0).
    let packet = first_packet.expect("one ambient packet");
    let mut expected = vec![35, 0, 8, 0];
    expected.extend(actor.to_le_bytes());
    expected.extend(first[0].1.to_le_bytes());
    expected.extend([0; 4]);
    expected.push(0);
    assert_eq!(packet, expected);

    // A full reliable queue refuses only that viewer: its known set and saved
    // state stay put, the other viewer still gets the same firing, and the
    // shared tick finishes with no fault.
    let refused_firing = (15_001..=30_000)
        .find(|now| {
            table
                .due(actor, CHUGAWUG_MODEL as u32, now - 1, *now)
                .is_some()
        })
        .expect("the chugawug row fires again");
    let before_known = server.sessions[&keys[0]].known_npcs.clone();
    let before_state = server.sessions[&keys[0]].gateway.snapshot();
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(1);
    server.npc_ambient_ms = refused_firing - 1;
    server.tick_shared_fields(refused_firing);
    assert_eq!(
        server.sessions[&keys[0]].known_npcs, before_known,
        "a refused ambient packet changes nothing"
    );
    assert_eq!(
        server.sessions[&keys[0]].gateway.snapshot(),
        before_state,
        "a refused ambient packet changes no saved state"
    );
    assert!(
        server.faulted_fields.is_empty(),
        "one full queue must not fault the shared field"
    );
    assert!(
        !ambient_for(&bodies(&mut server, keys[1], refused_firing), actor).is_empty(),
        "the other viewer still receives the firing"
    );

    // A field enemy at a real firing time of its model's row: the GUID is in
    // the known set, the row would animate it, and visibility still refuses.
    let ambient = &table;
    let firing = (refused_firing + 1..=refused_firing + 120_000)
        .find(|now| {
            ambient
                .due(field_enemy, FIELD_ENEMY_MODEL as u32, now - 1, *now)
                .is_some()
        })
        .expect("the field enemy's model row has a firing");
    server
        .sessions
        .get_mut(&keys[2])
        .unwrap()
        .known_npcs
        .insert(field_enemy);
    server.npc_ambient_ms = firing - 1;
    server.tick_shared_fields(firing);
    let seen = bodies(&mut server, keys[2], firing);
    assert!(
        ambient_for(&seen, field_enemy).is_empty(),
        "a field enemy receives no ambient 35/8"
    );

    let _ = fs::remove_dir_all(scratch);
}
