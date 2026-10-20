use gateway_reference::{Reference, initial_state};
use osfr_tools::gateway::Gateway;
use osfr_tools::startup_selfdata as typed_selfdata;
use osfr_tools::startup_snapshot::StartupSnapshot;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "common/gateway_reference.rs"]
mod gateway_reference;

const VERSION: &str = "synthetic.1";

fn string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn wrapped(outer: u16, inner: &[u8]) -> Vec<u8> {
    let mut bytes = outer.to_le_bytes().to_vec();
    bytes.push(1);
    bytes.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    bytes.extend_from_slice(inner);
    bytes
}

fn packet(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend_from_slice(fields);
    wrapped(5, &inner)
}

fn login(guid: u64, version: &str, ticket: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    string(&mut bytes, ticket);
    bytes.extend_from_slice(&guid.to_le_bytes());
    string(&mut bytes, version);
    string(&mut bytes, "");
    bytes
}

fn position(guid: u64, xyz: [f32; 3], rotation: [f32; 3]) -> Vec<u8> {
    let mut fields = guid.to_le_bytes().to_vec();
    for value in xyz.into_iter().chain(rotation) {
        fields.extend_from_slice(&value.to_le_bytes());
    }
    fields.extend_from_slice(&[2, 255]);
    packet(125, &fields)
}

fn jump(guid: u64, xyz: [f32; 3], velocity: f32) -> Vec<u8> {
    let mut fields = guid.to_le_bytes().to_vec();
    for value in xyz.into_iter().chain([0.0, 0.0, 1.0]) {
        fields.extend_from_slice(&value.to_le_bytes());
    }
    fields.extend_from_slice(&[4, 255]);
    fields.extend_from_slice(&velocity.to_le_bytes());
    packet(164, &fields)
}

fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        if message.len() < 255 {
            bytes.push(message.len() as u8);
        } else if message.len() <= 65_535 && message.len() != 255 {
            bytes.push(255);
            bytes.extend_from_slice(&(message.len() as u16).to_be_bytes());
        } else {
            bytes.extend_from_slice(&[255, 0, 255]);
            bytes.extend_from_slice(&(message.len() as u32).to_be_bytes());
        }
        bytes.extend_from_slice(message);
    }
    bytes
}

impl Reference {
    fn gateway(&self) -> Gateway {
        Gateway::load_with_startup_snapshot(
            &self.directory,
            initial_state(),
            StartupSnapshot::empty(),
        )
        .unwrap()
    }
    fn ready(&self) -> Gateway {
        let mut gateway = self.gateway();
        gateway.handle(&login(1, VERSION, "local-ticket")).unwrap();
        gateway.handle(&packet(13, &[])).unwrap();
        gateway
    }
}

#[test]
fn synthetic_startup_sequence_has_the_expected_phase_order_and_lengths() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.gateway();
    let login_replies = gateway
        .handle(&login(1, VERSION, "ticket"))
        .unwrap()
        .replies;
    let ready_replies = gateway.handle(&packet(13, &[])).unwrap().replies;

    let opcodes = |messages: &[Vec<u8>]| {
        messages
            .iter()
            .map(|message| {
                assert_eq!(&message[..3], &[5, 0, 1]);
                assert_eq!(
                    message.len(),
                    7 + u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize
                );
                u16::from_le_bytes(message[7..9].try_into().unwrap())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(opcodes(&login_replies), [41, 2, 166, 43, 144, 35, 193, 12]);
    assert_eq!(login_replies.len(), 8);
    assert_eq!(ready_replies.len(), 38);
    let expected_ready = [
        67, 57, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35, 35,
        35, 35, 38, 38, 36, 38, 38, 209, 209, 165, 93, 14, 38, 38, 87,
    ];
    assert_eq!(opcodes(&ready_replies), expected_ready);
}

#[test]
#[ignore = "requires private startup reference and typed lab snapshot"]
fn built_startup_sequence_matches_the_captured_character_byte_for_byte() {
    let reference = PathBuf::from(
        std::env::var("OSFR_GATEWAY_REFERENCE").expect("private reference path required"),
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    let version = manifest["client_version"].as_str().unwrap();
    let state = Gateway::reference_state(&reference).unwrap();
    let mut gateway = Gateway::load(&reference, state.clone()).unwrap();
    let mut actual = gateway
        .handle(&login(state.guid, version, "startup-oracle"))
        .unwrap()
        .replies;
    actual.extend(gateway.handle(&packet(13, &[])).unwrap().replies);
    assert_eq!(actual.len(), 46);
    for (index, message) in actual.iter().enumerate() {
        let phase = if index < 8 { "login" } else { "ready" };
        let expected = fs::read(reference.join(format!("{index:04}-{phase}.bin"))).unwrap();
        assert_eq!(message, &expected, "startup message index {index:02}");
    }
}

#[test]
fn typed_startup_builder_serializes_session_position_and_preserves_tail() {
    let reference = Reference::new(VERSION, None);
    assert_eq!(
        Gateway::reference_state(&reference.directory).unwrap(),
        initial_state()
    );
    let mut state = initial_state();
    state.position = [-400.5, 30.0, 500.25];
    state.rotation = [0.8, 0.0, -0.4];
    let startup = StartupSnapshot::empty();
    let mut gateway =
        Gateway::load_with_startup_snapshot(&reference.directory, state.clone(), startup.clone())
            .unwrap();
    let result = gateway.handle(&login(1, VERSION, "local-ticket")).unwrap();
    assert_eq!(result.replies.len(), 8);
    assert!(!result.entered_world && !result.state_changed);
    for (index, reply) in result.replies.iter().enumerate() {
        let expected = if index == 3 {
            startup.build_zone(state.zone_id, None)
        } else if index == 7 {
            let original = fs::read(reference.directory.join(format!("{index:04}.bin"))).unwrap();
            let mut data = typed_selfdata::parse_self_with_quest_bindings(&original, &[]);
            data.identity.guid = state.guid;
            for component in 0..3 {
                data.transform.position[component] = state.position[component].to_bits();
                data.transform.rotation[component] = state.rotation[component].to_bits();
            }
            let built = typed_selfdata::build_self(&data);
            let decoded = typed_selfdata::parse_self_with_quest_bindings(&built, &[]);
            assert_eq!(
                decoded.transform.position[..3],
                state.position.map(f32::to_bits)
            );
            assert_eq!(
                decoded.transform.rotation[..3],
                state.rotation.map(f32::to_bits)
            );
            assert_eq!(decoded.post_effect_tail, [90, 91, 92]);
            built
        } else {
            startup.build_packet(index, state.guid).unwrap()
        };
        assert_eq!(*reply, expected);
    }
    assert_eq!(gateway.state(), &state);
}

#[test]
fn login_and_world_entry_are_validated_once_per_session() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.gateway();
    for bytes in [
        login(2, VERSION, "ticket"),
        login(1, "wrong-build", "ticket"),
        login(1, VERSION, ""),
        login(1, VERSION, &"a".repeat(257)),
    ] {
        assert!(gateway.handle(&bytes).is_err());
    }
    assert!(gateway.handle(&packet(13, &[])).is_err());
    assert!(gateway.handle(&packet(10, &[])).is_err());
    assert_eq!(
        gateway
            .handle(&login(1, VERSION, "ticket"))
            .unwrap()
            .replies
            .len(),
        8
    );
    assert!(
        gateway
            .handle(&login(1, VERSION, "ticket"))
            .unwrap()
            .replies
            .is_empty()
    );
    assert_eq!(gateway.handle(&packet(13, &[])).unwrap().replies.len(), 38);
    assert!(gateway.handle(&packet(13, &[])).unwrap().replies.is_empty());
    assert!(gateway.handle(&packet(10, &[])).unwrap().entered_world);
    assert!(!gateway.handle(&packet(10, &[])).unwrap().entered_world);
    gateway.reset_session();
    assert_eq!(
        gateway
            .handle(&login(1, VERSION, "ticket"))
            .unwrap()
            .replies
            .len(),
        8
    );
}

#[test]
fn position_updates_character_but_camera_does_not() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    let xyz = [-100.0, 42.0, 30.0];
    let rotation = [0.8, 0.0, -0.4];
    assert!(
        gateway
            .handle(&position(1, xyz, rotation))
            .unwrap()
            .state_changed
    );
    assert_eq!(gateway.state().position, xyz);
    assert_eq!(gateway.state().rotation, [0.894_427_2, 0.0, -0.447_213_6]);
    assert!(
        !gateway
            .handle(&position(1, xyz, rotation))
            .unwrap()
            .state_changed
    );
    let mut camera = Vec::new();
    for value in [9.0f32; 6] {
        camera.extend_from_slice(&value.to_le_bytes());
    }
    assert!(!gateway.handle(&packet(126, &camera)).unwrap().state_changed);
    assert_eq!(gateway.state().position, xyz);
    gateway.reset_session();
    let replies = gateway
        .handle(&login(1, VERSION, "ticket"))
        .unwrap()
        .replies;
    let offset = reference.manifest["self_layout"]["position_offset"]
        .as_u64()
        .unwrap() as usize;
    assert_eq!(&replies[7][offset..offset + 4], &xyz[0].to_le_bytes());
}

#[test]
fn reset_session_drops_a_staged_jump_relay() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    let xyz = [4.0, 2.0, 3.0];
    gateway.handle(&jump(1, xyz, 4.5)).unwrap();
    assert!(gateway.take_jump_relay().is_some());
    gateway.handle(&jump(1, xyz, 4.5)).unwrap();
    gateway.reset_session();
    assert!(
        gateway.take_jump_relay().is_none(),
        "a reset session must not keep a staged jump body"
    );
}

#[test]
fn an_application_child_after_a_jump_drops_the_staged_relay() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    gateway.handle(&jump(1, [4.0, 2.0, 3.0], 4.5)).unwrap();
    assert!(gateway.take_jump_relay().is_some());
    // A later application child can relocate or zone the character (the
    // gameplay path returns `relocation`/`zoning` there); the impulse must not
    // survive it. Here gameplay is absent, so the child is forwarded instead.
    let out = gateway
        .handle(&bundle(&[
            jump(1, [5.0, 2.0, 3.0], 4.5),
            wrapped(5, &[16, 0]),
        ]))
        .unwrap();
    assert_eq!(out.applications, vec![vec![16, 0]]);
    assert_eq!(gateway.state().position, [5.0, 2.0, 3.0]);
    assert!(
        gateway.take_jump_relay().is_none(),
        "a later application child must drop the jump relay body"
    );
}

#[test]
fn rejected_position_snaps_back_with_the_server_location_message() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    let original = gateway.state().clone();
    let outcome = gateway
        .handle(&position(1, [10_000.0, 2.0, 3.0], [1.0, 0.0, 0.0]))
        .unwrap();
    assert!(outcome.state_changed);
    assert_eq!(gateway.state().position, original.position);
    assert_eq!(outcome.replies.len(), 1);
    assert_eq!(&outcome.replies[0][7..11], &[38, 0, 12, 0]);
    assert_eq!(
        &outcome.replies[0][11..23],
        &original
            .position
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>()
    );
}

#[test]
fn malformed_movement_and_bundle_failure_cannot_partially_commit() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    let original = gateway.state().clone();
    let good = position(1, [50.0, 60.0, 70.0], [0.0; 3]);
    for cut in 0..good.len() {
        assert!(gateway.handle(&good[..cut]).is_err(), "cut {cut}");
    }
    for bad in [
        position(2, [50.0; 3], [0.0; 3]),
        position(1, [f32::NAN; 3], [0.0; 3]),
        position(1, [f32::INFINITY; 3], [0.0; 3]),
        position(1, [1_000_001.0; 3], [0.0; 3]),
    ] {
        assert!(gateway.handle(&bundle(&[good.clone(), bad])).is_err());
        assert_eq!(gateway.state(), &original);
    }
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(gateway.handle(&trailing).is_err());
    let mut fresh = reference.gateway();
    assert!(
        fresh
            .handle(&bundle(&[login(1, VERSION, "ticket"), good]))
            .is_err()
    );
    assert_eq!(
        fresh
            .handle(&login(1, VERSION, "ticket"))
            .unwrap()
            .replies
            .len(),
        8
    );
}

#[test]
fn real_bundle_prefix_variants_and_tunnel_lengths_are_bounded() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.gateway();
    let mut messages = vec![
        login(1, VERSION, "ticket"),
        packet(13, &[]),
        packet(10, &[]),
    ];
    // Both extended lengths used by SOE application bundles, including 255.
    messages.push(wrapped(6, &vec![0; 248]));
    messages.push(wrapped(6, &vec![0; 293]));
    let outcome = gateway.handle(&bundle(&messages)).unwrap();
    assert_eq!(outcome.replies.len(), 46);
    assert!(outcome.entered_world);
    for bad in [
        vec![0, 25],
        vec![0, 25, 255],
        vec![0, 25, 255, 0, 255, 0],
        bundle(&[vec![0, 25]]),
        bundle(&vec![packet(200, &[]); 129]),
    ] {
        assert!(gateway.handle(&bad).is_err());
    }
    let mut bad_bool = packet(200, &[]);
    bad_bool[2] = 2;
    assert!(gateway.handle(&bad_bool).is_err());
    let mut bad_size = packet(200, &[]);
    bad_size[3..7].copy_from_slice(&1u32.to_le_bytes());
    assert!(gateway.handle(&bad_size).is_err());
}

#[test]
fn time_response_uses_current_seconds_and_original_packet_shape() {
    let reference = Reference::new(VERSION, None);
    let mut gateway = reference.ready();
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let outcome = gateway.handle(&packet(52, &[0; 13])).unwrap();
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    assert_eq!(outcome.replies.len(), 1);
    let bytes = &outcome.replies[0];
    assert_eq!(&bytes[..9], &[5, 0, 1, 15, 0, 0, 0, 52, 0]);
    let seconds = i64::from_le_bytes(bytes[9..17].try_into().unwrap());
    assert!((before..=after).contains(&seconds));
    assert_eq!(&bytes[17..], &[0, 0, 0, 0, 1]);
}

#[test]
fn corrupt_or_unproven_reference_is_rejected() {
    let mut reference = Reference::new(VERSION, None);
    reference.manifest["templates"][0]["sha256"] = json!("00".repeat(32));
    reference.write_manifest();
    assert!(Gateway::reference_state(&reference.directory).is_err());
    let mut reference = Reference::new(VERSION, None);
    reference.manifest["templates"][0]["file"] = json!("../outside.bin");
    reference.write_manifest();
    assert!(Gateway::reference_state(&reference.directory).is_err());
    let mut reference = Reference::new(VERSION, None);
    reference.manifest["self_layout"]["position_offset"] = json!(21);
    reference.write_manifest();
    assert!(Gateway::reference_state(&reference.directory).is_err());
    let mut other = initial_state();
    other.guid = 3;
    let mut gateway = Gateway::load_with_startup_snapshot(
        &Reference::new(VERSION, None).directory,
        other,
        StartupSnapshot::empty(),
    )
    .unwrap();
    assert_eq!(
        gateway
            .handle(&login(3, VERSION, "ticket"))
            .unwrap()
            .replies
            .len(),
        8
    );
    let reference = Reference::new(VERSION, None);
    let mut other_zone = initial_state();
    other_zone.zone_id = 19;
    let error = Gateway::load_with_startup_snapshot(
        &reference.directory,
        other_zone,
        StartupSnapshot::empty(),
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("unsupported saved zone"));
}

/// Opt-in oracle: never embeds original game bytes in the repository.
/// Set OSFR_GATEWAY_REFERENCE and OSFR_GATEWAY_CAPTURE to the private lab paths.
#[test]
#[ignore = "requires private reference and original client capture"]
fn actual_original_client_capture_when_explicitly_configured() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for the original-client oracle");
    let capture = PathBuf::from(
        std::env::var("OSFR_GATEWAY_CAPTURE").expect("capture path required with reference"),
    );
    let analysis: Value =
        serde_json::from_slice(&fs::read(capture.join("analysis/packet-analysis.json")).unwrap())
            .unwrap();
    let state = Gateway::reference_state(reference.as_ref()).unwrap();
    let mut gateway = Gateway::load(reference.as_ref(), state).unwrap();
    let mut replies = Vec::new();
    let mut entered_world = 0;
    for message in analysis["directions"]["client-to-server"]["reliable_messages"]
        .as_array()
        .unwrap()
    {
        let bytes = fs::read(capture.join(message["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            message["sha256"].as_str().unwrap()
        );
        let result = gateway.handle(&bytes).unwrap();
        replies.extend(result.replies);
        entered_world += usize::from(result.entered_world);
    }
    assert_eq!(entered_world, 1);
    // Initial 46 templates + seven time replies; legacy area-of-interest reply
    // 104 is beyond this one-character gateway slice.
    assert_eq!(replies.len(), 53);
    for (reply, original) in replies[..46].iter().zip(
        analysis["directions"]["server-to-client"]["messages"]
            .as_array()
            .unwrap(),
    ) {
        let expected = fs::read(capture.join(original["path"].as_str().unwrap())).unwrap();
        assert_eq!(reply, &expected);
    }
}

/// W139 (brief #15): the gateway must hand the emote policy the startup rows'
/// animation ids and the last accepted movement time. Fails to compile before
/// `Gateway::emote_context` exists and fails the `13` assert with the old
/// `from_ids` catalog.
#[test]
fn quick_chat_catalog_carries_animation_ids_and_emote_context_follows_movement() {
    use osfr_tools::startup_world::QuickChatRow;
    let row = |id, animation_id| QuickChatRow {
        key: id,
        id,
        display_string_id: 0,
        say_string_id: 0,
        animation_id,
        parent_id: 0,
        admin_only: 0,
        menu_icon_id: 0,
        item_id: 0,
        item_category: 0,
    };
    let reference = Reference::new(VERSION, None);
    let mut snapshot = StartupSnapshot::empty();
    snapshot.world.quick_chat = vec![row(11, 0), row(12, 3306), row(13, 3200)];
    let mut gateway =
        Gateway::load_with_startup_snapshot(&reference.directory, initial_state(), snapshot)
            .unwrap();
    let catalog = gateway.quick_chat_catalog();
    let standing = osfr_tools::social_chat::piece_093::EmoteContext::standing();
    assert!(catalog.authorize_emote(11, standing).is_ok());
    assert!(catalog.authorize_emote(12, standing).is_ok());
    assert!(catalog.authorize_emote(13, standing).is_err()); // Dance 3200 stays unavailable

    gateway.handle(&login(1, VERSION, "ticket")).unwrap();
    gateway.handle(&packet(13, &[])).unwrap();
    assert!(!gateway.emote_context(1_000).moving); // nothing accepted yet
    gateway
        .handle_at(&position(1, [-100.0, 42.0, 30.0], [0.8, 0.0, -0.4]), 1_000)
        .unwrap();
    assert!(gateway.emote_context(1_200).moving);
    assert!(!gateway.emote_context(1_600).moving);
    assert!(!gateway.emote_context(1_200).mounted); // (R) no mount service routed yet
}
