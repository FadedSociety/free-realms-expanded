//! T507: passive content NPCs play the per-model ambient animation row, and
//! every viewer that knows the actor receives the same 35/8 (client header
//! reader `0x008d6830`, SetAnimation reader `0x008e5dd0`). Field actors are
//! excluded from the passive visibility table, so their model row never fires.
use super::*;
use osfr_tools::npc_ambient::{AmbientUpdate, NpcAmbient};
use serde_json::Value;

#[path = "../../../tests/support/json.rs"]
mod json;

const CHUGAWUG_MODEL: u64 = 13;
const FIELD_ENEMY_MODEL: u64 = 203;

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
    let extras = json::read(&content.join("npc_spawns_extra.json"));
    let enemies = json::read(&content.join("field_enemies.json"));
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
    {
        let refused = server.sessions.get_mut(&keys[0]).unwrap();
        refused.sender = sender(1);
        // The Studio inspector records a shared message before the reliable
        // queue decides, so a refused attempt stays observable; without it this
        // test cannot tell a refusal from the hook skipping the viewer.
        refused.recent_on = true;
        refused.recent.clear();
    }
    server.npc_ambient_ms = refused_firing - 1;
    server.tick_shared_fields(refused_firing);
    let actor_hex: String = actor
        .to_le_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let refused_attempts = server.sessions[&keys[0]]
        .recent
        .iter()
        .filter(|message| {
            // `hex` is the whole inner packet: the 35/8 header, then the GUID.
            message["dir"] == "out"
                && message["family"] == 35
                && message["sub"] == 8
                && message["hex"].as_str().is_some_and(|body| {
                    body.starts_with("23000800")
                        && body
                            .get(8..)
                            .is_some_and(|body| body.starts_with(&actor_hex))
                })
        })
        .count();
    assert_eq!(
        refused_attempts, 1,
        "the due firing reached the 1-byte queue once and was refused"
    );
    assert_eq!(
        server.sessions[&keys[0]].sender.pending_packets(),
        0,
        "the full queue accepted nothing"
    );
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

    // Regression (T507 review): a fired window queues once even when its tick
    // takes the resync branch. `tick_shared_instance` replays
    // `sync_field_views(..., false, now)` after a committed change, so queue a
    // committed lethal HP packet (35/5 with hp 0) to force that branch; the
    // ambient `hold_deaths` gate is what stops the window firing twice. Then
    // repeat the same `sync_field_views(false)` call with the tick's own
    // window, which is deterministic even if the field has no due simulation.
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(32 * 1024 * 1024);
    let resync_firing = (firing + 1..=firing + 120_000)
        .find(|now| {
            ambient
                .due(actor, CHUGAWUG_MODEL as u32, now - 1, *now)
                .is_some()
        })
        .expect("the chugawug row fires again");
    let mut lethal = vec![35, 0, 5, 0];
    lethal.extend(actor.to_le_bytes());
    lethal.extend(0_i32.to_le_bytes()); // hp zero: the held-corpse resync trigger
    lethal.extend(1000_i32.to_le_bytes());
    lethal.extend(0_i32.to_le_bytes());
    server.membership.queue_field_packets(0, vec![lethal]);
    server.npc_ambient_ms = resync_firing - 1;
    server.tick_shared_fields(resync_firing);
    assert!(
        !server.faulted_fields.contains(&0),
        "the resync tick must finish, or its second view pass never runs"
    );
    let seen = bodies(&mut server, keys[0], resync_firing);
    assert!(
        seen.iter()
            .any(|body| body.starts_with(&[35, 0, 5, 0]) && body[4..12] == actor.to_le_bytes()),
        "the committed lethal HP packet reaches the viewer and forces the resync"
    );
    assert_eq!(
        ambient_for(&seen, actor).len(),
        1,
        "one 35/8 for a firing whose tick also resyncs the field"
    );
    let field = server
        .membership
        .field(0)
        .cloned()
        .expect("the open-world shared field");
    server.npc_ambient_ms = resync_firing - 1;
    server
        .sync_field_views(0, &field, false, resync_firing)
        .expect("the resync pass reconciles");
    let seen = bodies(&mut server, keys[0], resync_firing);
    assert!(
        seen.is_empty(),
        "a resync pass must not replay the ambient window: {seen:?}"
    );

    let _ = fs::remove_dir_all(scratch);
}

const CHILD_F_MODEL: u64 = 7;
const CHILD_M_MODEL: u64 = 9;
const DALMATION_MODEL: u64 = 514;

/// Two placed child models whose ambient rows have conversation and idle
/// fields, within the 6 m XZ grouping radius.
fn conversation_pair(document: &Value) -> ((u64, u32, [f32; 3]), (u64, u32, [f32; 3])) {
    let actors: Vec<_> = document["records"]
        .as_array()
        .expect("extra rows")
        .iter()
        .filter_map(|row| {
            let model_id = row["model_id"].as_u64()?;
            if model_id != CHILD_F_MODEL && model_id != CHILD_M_MODEL {
                return None;
            }
            Some((
                row["spawn_guid"].as_u64().expect("a spawn GUID"),
                u32::try_from(model_id).ok()?,
                serde_json::from_value::<[f32; 3]>(row["position"].clone())
                    .expect("an XYZ position"),
            ))
        })
        .collect();
    for first in &actors {
        for second in &actors {
            if first.0 == second.0 {
                continue;
            }
            let dx = first.2[0] - second.2[0];
            let dz = first.2[2] - second.2[2];
            if dx * dx + dz * dz <= 36.0 {
                return (*first, *second);
            }
        }
    }
    panic!("content has a child conversation pair");
}

/// `(at, update)` for the first window in `[start, start + limit)` that
/// contains an update the predicate accepts. The real hook calls the same
/// `due_for_known` with the same known set and window.
fn find_due(
    table: &NpcAmbient,
    known: &BTreeSet<u64>,
    models: &BTreeMap<u64, u32>,
    want: impl Fn(&AmbientUpdate) -> bool,
    start: u64,
    limit: u64,
) -> Option<(u64, AmbientUpdate)> {
    for at in start..start.saturating_add(limit) {
        let updates = table.due_for_known(known, |guid| models.get(&guid).copied(), at - 1, at);
        if let Some(update) = updates.into_iter().find(|update| want(update)) {
            return Some((at, update));
        }
    }
    None
}

/// Put the session in the player registry at the open-world position, as
/// `Session::advance` does on world entry and movement.
fn register_pose(server: &mut Server, key: SessionKey, position: [f32; 3]) {
    let id = server.sessions[&key].id;
    server
        .world
        .players
        .update_pose_with_state(
            id,
            ZoneInstance {
                zone_id: 0,
                instance_id: 0,
            },
            position,
            0.0,
            0,
            0,
        )
        .expect("the player registry accepts the test pose");
}

/// Tick the shared field with this viewer's known set pinned, so the ambient
/// pass sees exactly the actors under test.
fn tick_window(
    server: &mut Server,
    keys: [SessionKey; 2],
    known: &BTreeSet<u64>,
    from: u64,
    to: u64,
) {
    for key in keys {
        server.sessions.get_mut(&key).unwrap().known_npcs = known.clone();
    }
    server.npc_ambient_ms = from;
    server.tick_shared_fields(to);
}

fn tick_with_known(server: &mut Server, keys: [SessionKey; 2], known: &BTreeSet<u64>, at: u64) {
    tick_window(server, keys, known, at - 1, at);
}

fn assert_packets(server: &mut Server, key: SessionKey, at: u64, packets: &[Vec<u8>], why: &str) {
    let seen = bodies(server, key, at);
    for packet in packets {
        assert!(
            seen.iter().any(|body| body == packet),
            "{why}: expected {packet:?} in {seen:?}"
        );
    }
}

fn look_at_for(bodies: &[Vec<u8>], guid: u64) -> Vec<u64> {
    bodies
        .iter()
        .filter(|body| body.starts_with(&[35, 0, 17, 0]) && body.len() == 20)
        .filter_map(|body| {
            let actor = u64::from_le_bytes(body[4..12].try_into().unwrap());
            (actor == guid).then(|| u64::from_le_bytes(body[12..20].try_into().unwrap()))
        })
        .collect()
}

#[test]
fn t552_conversation_turns_and_idle_rows_reach_every_viewer() {
    if !super::tests::test_support::bin_test_target("t552") {
        return;
    }
    if !super::t020_group_routing::fixtures_present() {
        eprintln!("skip t552-conversation: the private lab fixtures are absent");
        return;
    }
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extras = json::read(&content.join("npc_spawns_extra.json"));
    let (first, second) = conversation_pair(&extras);
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("t552-conversation");
    let manifest: Value =
        serde_json::from_slice(&fs::read(server.options.reference.join("manifest.json")).unwrap())
            .unwrap();
    let version = manifest["client_version"].as_str().unwrap().to_owned();
    load_world_view(&mut server, keys[0], first.2, &version);
    load_world_view(&mut server, keys[1], second.2, &version);

    let (table, models) = {
        let game = server.sessions[&keys[0]].gateway.gameplay().unwrap();
        let table = game.npc_ambient().expect("the ambient table loads").clone();
        let models = BTreeMap::from([
            (
                first.0,
                game.npc_model(first.0).expect("the first child is visible"),
            ),
            (
                second.0,
                game.npc_model(second.0)
                    .expect("the second child is visible"),
            ),
        ]);
        (table, models)
    };
    let known: BTreeSet<u64> = [first.0, second.0].into_iter().collect();

    let (first_at, first_update) = find_due(
        &table,
        &known,
        &models,
        |update| matches!(update, AmbientUpdate::Conversation { .. }),
        2_000,
        60_000,
    )
    .expect("the child pair has a conversation firing");
    let AmbientUpdate::Conversation {
        guid: first_speaker,
        ..
    } = &first_update
    else {
        unreachable!("the predicate selected a conversation");
    };
    let first_speaker = *first_speaker;
    tick_with_known(&mut server, [keys[0], keys[1]], &known, first_at);
    let expected = first_update.packets(0);
    assert_eq!(expected.len(), 1);
    for key in [keys[0], keys[1]] {
        assert_packets(
            &mut server,
            key,
            first_at,
            &expected,
            "conversation reaches both viewers",
        );
    }

    let (second_at, second_update) = find_due(
        &table,
        &known,
        &models,
        |update| matches!(update, AmbientUpdate::Conversation { .. }),
        first_at + 1,
        60_000,
    )
    .expect("the child pair speaks again");
    let AmbientUpdate::Conversation {
        guid: second_speaker,
        ..
    } = &second_update
    else {
        unreachable!("the predicate selected a conversation");
    };
    assert_ne!(
        first_speaker, *second_speaker,
        "a group of two takes turns: {first_speaker} then {second_speaker}"
    );
    tick_with_known(&mut server, [keys[0], keys[1]], &known, second_at);
    for key in [keys[0], keys[1]] {
        assert_packets(
            &mut server,
            key,
            second_at,
            &second_update.packets(0),
            "the second speaker reaches both viewers",
        );
    }

    let (idle_at, idle_update) = find_due(
        &table,
        &known,
        &models,
        |update| matches!(update, AmbientUpdate::Idle { .. }),
        second_at + 1,
        60_000,
    )
    .expect("a child idle row fires");
    let idle_packets = idle_update.packets(0);
    assert_eq!(idle_packets.len(), 2, "35/26 and the base 35/8");
    tick_with_known(&mut server, [keys[0], keys[1]], &known, idle_at);
    for key in [keys[0], keys[1]] {
        assert_packets(
            &mut server,
            key,
            idle_at,
            &idle_packets,
            "the idle loop reaches both viewers",
        );
    }

    let _ = fs::remove_dir_all(scratch);
}

#[test]
fn t552_look_at_target_is_shared_and_clears_out_of_range() {
    if !super::tests::test_support::bin_test_target("t552") {
        return;
    }
    if !super::t020_group_routing::fixtures_present() {
        eprintln!("skip t552-look-at: the private lab fixtures are absent");
        return;
    }
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extras = json::read(&content.join("npc_spawns_extra.json"));
    let (dog, dog_at) = spawn_of_model(&extras, DALMATION_MODEL);
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("t552-look-at");
    let manifest: Value =
        serde_json::from_slice(&fs::read(server.options.reference.join("manifest.json")).unwrap())
            .unwrap();
    let version = manifest["client_version"].as_str().unwrap().to_owned();
    let near = [dog_at[0] + 1.0, dog_at[1], dog_at[2]];
    let farther = [dog_at[0] + 2.0, dog_at[1], dog_at[2]];
    load_world_view(&mut server, keys[0], near, &version);
    load_world_view(&mut server, keys[1], farther, &version);
    register_pose(&mut server, keys[0], near);
    register_pose(&mut server, keys[1], farther);
    let viewer_guids = [
        server.sessions[&keys[0]].character_guid,
        server.sessions[&keys[1]].character_guid,
    ];
    let known: BTreeSet<u64> = [dog].into_iter().collect();

    let mut first_at = None;
    for now in (2_050..=30_000).step_by(50) {
        tick_window(&mut server, [keys[0], keys[1]], &known, now - 50, now);
        let first = look_at_for(&bodies(&mut server, keys[0], now), dog);
        let second = look_at_for(&bodies(&mut server, keys[1], now), dog);
        if let (Some(&first_target), Some(&second_target)) = (first.last(), second.last()) {
            assert_eq!(
                first_target, second_target,
                "every viewer resolves the same look-at target"
            );
            assert_ne!(first_target, 0, "a nearby player is the target");
            first_at = Some((now, first_target));
            break;
        }
    }
    let (target_at, target) = first_at.expect("the dog looks at a nearby player");
    assert_eq!(
        target, viewer_guids[0],
        "the nearest ready player is the target"
    );

    let far = [dog_at[0] + 50.0, dog_at[1], dog_at[2]];
    register_pose(&mut server, keys[0], far);
    register_pose(&mut server, keys[1], [far[0] + 1.0, far[1], far[2]]);
    let mut cleared = false;
    for now in (target_at + 50..=target_at + 30_000).step_by(50) {
        tick_window(&mut server, [keys[0], keys[1]], &known, now - 50, now);
        let first = look_at_for(&bodies(&mut server, keys[0], now), dog);
        let second = look_at_for(&bodies(&mut server, keys[1], now), dog);
        if let (Some(&first_target), Some(&second_target)) = (first.last(), second.last()) {
            assert_eq!(first_target, second_target);
            if first_target == 0 {
                cleared = true;
                break;
            }
        }
    }
    assert!(
        cleared,
        "the dog clears its look-at when no player is in range"
    );

    let _ = fs::remove_dir_all(scratch);
}
