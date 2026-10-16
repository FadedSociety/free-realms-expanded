//! Kart native race-entry handler (tunneled inner opcode 33) - owned vectors.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `notes/Research/Kart Native Race Entry Contract.md` - the `33/9` register starts the race; the
//!   server answers `33/34`, one `33/11` per entry vehicle, `33/5` and `33/14` in that order; the
//!   client `33/2` join is answered with `33/3` (player count) and `33/1` (game state); the later
//!   bundle repeats `33/5`/`33/14` with *different* payloads and carries `33/1` = 4 plus `33/4`.
//! * `notes/Research/Kart Community World Binding.md` - the scene is named by opcode 31; the
//!   captured `sg_kt_track` frame carries position `719.61, 72.0, 421.5, 1.0` and rotation
//!   `(0, 0, 1, 0)`; encounter `917566` / instance `1008` repeats the same spawn.
//! * `lab/reports/kart-race-entry-contract-20261009T1735Z/proof-v2.json` - field maps, subtype
//!   table (1, 2, 3, 4, 5, 9, 11, 14, 34) and the capture hashes pinned below.
//! * `notes/Reviews/Delegated Racing Readiness.md` - the recovery gap list this slice stays inside:
//!   no finish, no payout, no physics, no derby.
//!
//! This target exercises the registered module with its shared codec and zone types.
use osfr_tools::kart_entry;
pub use osfr_tools::{kart_progress_native, kart_racing, zone_transition};

use kart_entry::{
    CAPTURED_GAME_STATE_START, CAPTURED_GAME_STATE_STREAMING, CLIENT_DATA_PAYLOAD_LEN,
    CLIENT_DATA_SUBTYPE, CREATE_VEHICLE_SUBTYPE, EntryError, EntryGate, EntryOutcome, EntryPhase,
    EntryRefusal, GAME_STATE_SUBTYPE, KartEntryHandler, MAX_ENTRY_VEHICLES, MAX_RACERS,
    NPC_COUNT_SUBTYPE, PLAYER_COUNT_SUBTYPE, PLAYER_STATE_PAYLOAD_LEN, PROGRESS_SUBTYPE,
    PlayerState, ProxiedVehicle, RACE_CONFIG_WORDS, REGISTER_PAYLOAD_LEN, RaceConfig,
    RaceEntryContent, RaceEntryContext, RegisterVehicleRacePlayer, SET_RACE_CONFIG_SUBTYPE,
    VEHICLE_RACE_INNER_OPCODE, decode_inner, decode_player_state, decode_register, encode_inner,
};
use kart_progress_native::{NonFinitePolicy, ProgressPayload, decode_progress};
use zone_transition::{ZoneTransition, decode as decode_zone};

fn digest(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// ORIGINAL recovered scene context (`sg_kt_track`), including the rotation `(0, 0, 1, 0)` whose
/// `w` the encounter writer would force to `1.0`.
fn recovered_scene() -> ZoneTransition {
    ZoneTransition {
        envelope_flag: 1,
        scene: "sg_kt_track".to_string(),
        type_id: 2,
        position: [719.609985, 72.0, 421.5, 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
        sky: "sky.xml".to_string(),
        tutorial: false,
        unknown: 1,
        id: 0,
        update_radius: 0.0,
        geometry_id: 101,
        override_update_radius: true,
        wait_for_zone_ready: false,
    }
}

fn synthetic_progress(guid: u64, slot: i32) -> ProgressPayload {
    ProgressPayload {
        racer_guid: guid,
        slot,
        unknown_0c_bits: 0,
        unknown_10_bits: 0x7f7f_ffff,
        laps_completed: 0,
        local_racer: u8::from(slot == 0),
        lap_limit: 3,
        unknown_1d_bits: 0,
        best_lap_bits: 0,
        total_time_bits: 0,
        tail_0x29: guid,
        tail_0x31: 0,
        tail_0x39: 0,
        tail_0x41: 0,
    }
}

fn synthetic_content() -> RaceEntryContent {
    RaceEntryContent {
        race_config: RaceConfig {
            words: (1..=RACE_CONFIG_WORDS as u32).collect(),
        },
        entry_vehicles: vec![
            ProxiedVehicle {
                body_asset: "synthetic_body_a.cdt".to_string(),
                first_attribute: 400.0,
                attribute_block: vec![0x10, 0x20, 0x30],
            },
            ProxiedVehicle {
                body_asset: "synthetic_body_b.cdt".to_string(),
                first_attribute: 250.5,
                attribute_block: vec![0xaa, 0xbb],
            },
        ],
        client_data: {
            let mut bytes = vec![0x5a; CLIENT_DATA_PAYLOAD_LEN];
            bytes[77..81].copy_from_slice(&0x2a6ac44au32.to_le_bytes());
            bytes[81..85].copy_from_slice(&1u32.to_le_bytes());
            bytes
        },
        progress: vec![synthetic_progress(0x1122_3344_5566_7788, 0)],
        player_count: 1,
        npc_count: 7,
    }
}

fn synthetic_context() -> RaceEntryContext {
    RaceEntryContext {
        session: 0x91ca_7395_6acc_524b,
        instance_id: 1008,
        encounter_id: 917_566,
        scene: recovered_scene(),
    }
}

fn enabled(content: RaceEntryContent) -> KartEntryHandler {
    let mut handler = KartEntryHandler::new(content).expect("valid synthetic content");
    handler.set_gate(EntryGate::Enabled);
    handler
}

fn register_payload(session: u64) -> Vec<u8> {
    let mut payload = session.to_le_bytes().to_vec();
    payload.extend(0x2a6a_c44au32.to_le_bytes());
    payload.extend(1u32.to_le_bytes());
    payload
}

fn join_payload(session: u64, counter: u32) -> Vec<u8> {
    let mut payload = session.to_le_bytes().to_vec();
    payload.extend(counter.to_le_bytes());
    payload
}

fn replies(outcome: &EntryOutcome) -> &[Vec<u8>] {
    match outcome {
        EntryOutcome::Replies(replies) => replies,
        EntryOutcome::Refused(refusal) => panic!("expected replies, got refusal {refusal:?}"),
    }
}

fn refusal(outcome: &EntryOutcome) -> &EntryRefusal {
    match outcome {
        EntryOutcome::Refused(refusal) => refusal,
        EntryOutcome::Replies(replies) => {
            panic!("expected a refusal, got {} replies", replies.len())
        }
    }
}

fn subtypes(frames: &[Vec<u8>]) -> Vec<u16> {
    frames
        .iter()
        .map(|frame| decode_inner(frame).expect("inner frame").0)
        .collect()
}

fn u32_payload(frame: &[u8]) -> u32 {
    let (_, payload) = decode_inner(frame).expect("inner frame");
    u32::from_le_bytes(payload.try_into().expect("4-byte counter payload"))
}

#[test]
fn gate_defaults_off_and_refuses_without_any_reply_or_state() {
    let context = synthetic_context();
    let mut handler = KartEntryHandler::new(synthetic_content()).unwrap();
    assert_eq!(handler.gate(), EntryGate::Disabled);
    handler.open(context.clone()).unwrap();
    let frame = encode_inner(9, &register_payload(context.session));
    let outcome = handler.handle_inner(&frame).unwrap();
    assert_eq!(refusal(&outcome), &EntryRefusal::GateDisabled);
    assert_eq!(handler.phase(), EntryPhase::Waiting);
    assert_eq!(handler.session(), Some(context.session));
    assert_eq!(handler.instance_id(), Some(context.instance_id));
    assert_eq!(handler.encounter_id(), Some(context.encounter_id));
    assert_eq!(handler.scene(), Some(&recovered_scene()));
}

#[test]
fn unhandled_subtypes_and_missing_context_are_inert_refusals() {
    let mut handler = enabled(synthetic_content());
    let register = encode_inner(9, &register_payload(synthetic_context().session));
    assert_eq!(
        refusal(&handler.handle_inner(&register).unwrap()),
        &EntryRefusal::NoOpenContext
    );
    handler.open(synthetic_context()).unwrap();
    for subtype in [6u16, 7, 17, 10, 12] {
        let frame = encode_inner(subtype, &[]);
        assert_eq!(
            refusal(&handler.handle_inner(&frame).unwrap()),
            &EntryRefusal::UnsupportedSubtype { subtype }
        );
    }
    assert_eq!(handler.phase(), EntryPhase::Waiting);
}

#[test]
fn malformed_register_and_frame_widths_are_rejected() {
    let mut handler = enabled(synthetic_content());
    handler.open(synthetic_context()).unwrap();
    assert_eq!(
        handler.handle_inner(&[0x21]).unwrap_err(),
        EntryError::FrameTooShort { len: 1, needed: 4 }
    );
    let mut foreign_family = 32u16.to_le_bytes().to_vec();
    foreign_family.extend([0u8; 2]);
    assert_eq!(
        handler.handle_inner(&foreign_family).unwrap_err(),
        EntryError::UnsupportedOpcode { opcode: 32 }
    );
    for length in [REGISTER_PAYLOAD_LEN - 1, REGISTER_PAYLOAD_LEN + 1] {
        let frame = encode_inner(9, &vec![0u8; length]);
        assert_eq!(
            handler.handle_inner(&frame).unwrap_err(),
            EntryError::PayloadLength {
                subtype: 9,
                expected: REGISTER_PAYLOAD_LEN,
                actual: length,
            }
        );
    }
    for length in [PLAYER_STATE_PAYLOAD_LEN - 1, PLAYER_STATE_PAYLOAD_LEN + 1] {
        let frame = encode_inner(2, &vec![0u8; length]);
        assert_eq!(
            handler.handle_inner(&frame).unwrap_err(),
            EntryError::PayloadLength {
                subtype: 2,
                expected: PLAYER_STATE_PAYLOAD_LEN,
                actual: length,
            }
        );
    }
    assert_eq!(
        handler.phase(),
        EntryPhase::Waiting,
        "no phase moved on a refusal"
    );
}

#[test]
fn spoofed_session_is_refused_and_reported() {
    let context = synthetic_context();
    let mut handler = enabled(synthetic_content());
    handler.open(context.clone()).unwrap();
    let offered = 0x0bad_f00d_0bad_f00du64;
    let frame = encode_inner(9, &register_payload(offered));
    assert_eq!(
        refusal(&handler.handle_inner(&frame).unwrap()),
        &EntryRefusal::ForeignSession {
            offered,
            owned: context.session,
        }
    );
    assert_eq!(handler.phase(), EntryPhase::Waiting);
}

#[test]
fn stale_replay_and_closed_session_are_refused() {
    let context = synthetic_context();
    let mut handler = enabled(synthetic_content());
    handler.open(context.clone()).unwrap();
    let register = encode_inner(9, &register_payload(context.session));
    assert!(matches!(
        handler.handle_inner(&register).unwrap(),
        EntryOutcome::Replies(_)
    ));
    assert_eq!(handler.phase(), EntryPhase::Started);
    assert_eq!(
        refusal(&handler.handle_inner(&register).unwrap()),
        &EntryRefusal::StaleSession {
            phase: EntryPhase::Started
        }
    );
    handler.close();
    assert_eq!(handler.phase(), EntryPhase::Closed);
    assert_eq!(handler.session(), Some(context.session));
    assert_eq!(
        refusal(&handler.handle_inner(&register).unwrap()),
        &EntryRefusal::StaleSession {
            phase: EntryPhase::Closed
        }
    );
    let other = encode_inner(9, &register_payload(0x0102_0304_0506_0708));
    assert_eq!(
        refusal(&handler.handle_inner(&other).unwrap()),
        &EntryRefusal::NoOpenContext
    );
    // A closed handler opens again for the next speedway visit.
    handler.open(context).unwrap();
    assert_eq!(handler.phase(), EntryPhase::Waiting);
    assert_eq!(handler.session(), Some(synthetic_context().session));
}

#[test]
fn register_emits_the_ordered_start_bundle() {
    let content = synthetic_content();
    let context = synthetic_context();
    let mut handler = enabled(content.clone());
    handler.open(context.clone()).unwrap();
    let frame = encode_inner(9, &register_payload(context.session));
    let outcome = handler.handle_inner(&frame).unwrap();
    let frames = replies(&outcome);
    assert_eq!(
        subtypes(frames),
        vec![
            SET_RACE_CONFIG_SUBTYPE,
            CREATE_VEHICLE_SUBTYPE,
            CREATE_VEHICLE_SUBTYPE,
            CLIENT_DATA_SUBTYPE,
            PROGRESS_SUBTYPE,
        ]
    );
    assert_eq!(
        decode_inner(&frames[0]).unwrap().1,
        content.race_config.encode().as_slice()
    );
    assert_eq!(
        decode_inner(&frames[1]).unwrap().1,
        content.entry_vehicles[0].encode().as_slice()
    );
    assert_eq!(
        decode_inner(&frames[2]).unwrap().1,
        content.entry_vehicles[1].encode().as_slice()
    );
    assert_eq!(
        decode_inner(&frames[3]).unwrap().1,
        content.client_data.as_slice()
    );
    assert_eq!(
        decode_inner(&frames[4]).unwrap().1,
        kart_progress_native::encode_progress(&content.progress[0]).as_slice()
    );
    assert_eq!(handler.phase(), EntryPhase::Started);
    // Replaying the register never re-emits the bundle.
    assert_eq!(
        refusal(&handler.handle_inner(&frame).unwrap()),
        &EntryRefusal::StaleSession {
            phase: EntryPhase::Started
        }
    );
}

#[test]
fn player_state_requires_started_phase_and_matching_session() {
    let context = synthetic_context();
    let mut handler = enabled(synthetic_content());
    handler.open(context.clone()).unwrap();
    let join = encode_inner(2, &join_payload(context.session, 2));
    assert_eq!(
        refusal(&handler.handle_inner(&join).unwrap()),
        &EntryRefusal::InvalidPhase {
            phase: EntryPhase::Waiting
        }
    );
    let register = encode_inner(9, &register_payload(context.session));
    assert!(matches!(
        handler.handle_inner(&register).unwrap(),
        EntryOutcome::Replies(_)
    ));
    let foreign_join = encode_inner(2, &join_payload(0xdead_beef_dead_beef, 2));
    assert_eq!(
        refusal(&handler.handle_inner(&foreign_join).unwrap()),
        &EntryRefusal::ForeignSession {
            offered: 0xdead_beef_dead_beef,
            owned: context.session,
        }
    );
    assert_eq!(handler.phase(), EntryPhase::Started);
    let outcome = handler.handle_inner(&join).unwrap();
    let frames = replies(&outcome);
    assert_eq!(
        subtypes(frames),
        vec![PLAYER_COUNT_SUBTYPE, GAME_STATE_SUBTYPE]
    );
    assert_eq!(u32_payload(&frames[0]), 1);
    assert_eq!(u32_payload(&frames[1]) as i32, CAPTURED_GAME_STATE_START);
    assert_eq!(handler.phase(), EntryPhase::Joined);
    // The join is answered once; a replay is stale.
    assert_eq!(
        refusal(&handler.handle_inner(&join).unwrap()),
        &EntryRefusal::StaleSession {
            phase: EntryPhase::Joined
        }
    );
}

#[test]
fn race_ready_requires_joined_phase() {
    let context = synthetic_context();
    let mut handler = enabled(synthetic_content());
    assert_eq!(handler.on_race_ready().unwrap_err(), EntryError::NoContext);
    handler.open(context.clone()).unwrap();
    assert_eq!(
        refusal(&handler.on_race_ready().unwrap()),
        &EntryRefusal::InvalidPhase {
            phase: EntryPhase::Waiting
        }
    );
    let register = encode_inner(9, &register_payload(context.session));
    assert!(matches!(
        handler.handle_inner(&register).unwrap(),
        EntryOutcome::Replies(_)
    ));
    assert_eq!(
        refusal(&handler.on_race_ready().unwrap()),
        &EntryRefusal::InvalidPhase {
            phase: EntryPhase::Started
        }
    );
    let join = encode_inner(2, &join_payload(context.session, 2));
    assert!(matches!(
        handler.handle_inner(&join).unwrap(),
        EntryOutcome::Replies(_)
    ));
    let outcome = handler.on_race_ready().unwrap();
    let frames = replies(&outcome);
    assert_eq!(
        subtypes(frames),
        vec![GAME_STATE_SUBTYPE, NPC_COUNT_SUBTYPE],
        "the bulk 33/5 and 33/14 stream payloads are not re-authored here"
    );
    assert_eq!(
        u32_payload(&frames[0]) as i32,
        CAPTURED_GAME_STATE_STREAMING
    );
    assert_eq!(u32_payload(&frames[1]), 7);
    assert_eq!(handler.phase(), EntryPhase::Streaming);
    // Repeating a state transition is refused, not duplicated.
    assert_eq!(
        refusal(&handler.on_race_ready().unwrap()),
        &EntryRefusal::InvalidPhase {
            phase: EntryPhase::Streaming
        }
    );
}

#[test]
fn scene_frame_preserves_rotation_w_zero_through_the_zone_codec() {
    let mut handler = enabled(synthetic_content());
    assert_eq!(
        handler.scene_frame().unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
    handler.open(synthetic_context()).unwrap();
    let frame = handler.scene_frame().unwrap();
    let decoded = decode_zone(&frame).unwrap();
    assert_eq!(decoded, recovered_scene());
    assert_eq!(decoded.rotation, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(decoded.rotation[3], 0.0, "op31 scene keeps rotation w = 0");
    // The rotation quad occupies frame bytes 44..60; its last word must be 0.0, never the 1.0 the
    // encounter writer forces.
    assert_eq!(&frame[56..60], &0.0f32.to_le_bytes());
    assert_ne!(&frame[56..60], &1.0f32.to_le_bytes());
    assert_eq!(
        digest(&frame),
        digest(&zone_transition::encode(&recovered_scene()).unwrap())
    );
}

#[test]
fn content_bounds_are_refused() {
    let mut content = synthetic_content();
    content.race_config.words.pop();
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::ConfigWords {
            words: RACE_CONFIG_WORDS - 1,
            expected: RACE_CONFIG_WORDS,
        }
    );

    let mut content = synthetic_content();
    content.client_data.pop();
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::ClientData {
            bytes: CLIENT_DATA_PAYLOAD_LEN - 1,
            expected: CLIENT_DATA_PAYLOAD_LEN,
        }
    );

    let mut content = synthetic_content();
    content.entry_vehicles.clear();
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::VehicleCount {
            count: 0,
            max: MAX_ENTRY_VEHICLES,
        }
    );

    let mut content = synthetic_content();
    content.entry_vehicles = vec![ProxiedVehicle {
        body_asset: String::new(),
        first_attribute: 400.0,
        attribute_block: vec![0],
    }];
    assert!(matches!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::VehicleName { index: 0, .. }
    ));

    let mut content = synthetic_content();
    content.entry_vehicles[0].first_attribute = f32::NAN;
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::VehicleAttribute { index: 0 }
    );

    let mut content = synthetic_content();
    content.entry_vehicles[0].attribute_block.clear();
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::AttributeBlock { index: 0, bytes: 0 }
    );

    let mut content = synthetic_content();
    content.progress[0].racer_guid = 0;
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::RacerGuid { index: 0 }
    );

    let mut content = synthetic_content();
    content.progress[0].slot = -128;
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::RacerSlot {
            index: 0,
            slot: -128,
        }
    );

    let mut content = synthetic_content();
    content.progress = (0..MAX_RACERS + 1)
        .map(|index| synthetic_progress(0x1000 + index as u64, index as i32))
        .collect();
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::ProgressCount {
            count: MAX_RACERS + 1,
            max: MAX_RACERS,
        }
    );

    let mut content = synthetic_content();
    content.progress = vec![synthetic_progress(7, 0), synthetic_progress(7, 1)];
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::DuplicateRacerGuid { guid: 7 }
    );

    let mut content = synthetic_content();
    content.player_count = MAX_RACERS as u32 + 1;
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::Counter {
            field: "player_count",
            value: MAX_RACERS as u32 + 1,
        }
    );

    let mut content = synthetic_content();
    content.npc_count = MAX_RACERS as u32 + 1;
    assert_eq!(
        KartEntryHandler::new(content).unwrap_err(),
        EntryError::Counter {
            field: "npc_count",
            value: MAX_RACERS as u32 + 1,
        }
    );
}

#[test]
fn context_bounds_are_refused() {
    let open = |context: RaceEntryContext| {
        let mut handler = KartEntryHandler::new(synthetic_content()).unwrap();
        handler.open(context).unwrap_err()
    };
    let mut context = synthetic_context();
    context.session = 0;
    assert_eq!(open(context), EntryError::ZeroSession);
    let mut context = synthetic_context();
    context.instance_id = 0;
    assert_eq!(open(context), EntryError::ZeroInstance);
    let mut context = synthetic_context();
    context.encounter_id = 0;
    assert_eq!(open(context), EntryError::ZeroEncounter);
    let mut context = synthetic_context();
    context.scene.rotation = [0.0, 0.0, 1.0, f32::NAN];
    assert!(matches!(open(context), EntryError::SceneRefused { .. }));
    let mut handler = KartEntryHandler::new(synthetic_content()).unwrap();
    handler.open(synthetic_context()).unwrap();
    assert_eq!(
        handler.open(synthetic_context()).unwrap_err(),
        EntryError::AlreadyOpen {
            phase: EntryPhase::Waiting
        }
    );
}

#[test]
fn typed_decoders_match_the_recovered_layouts() {
    let session = 0x91ca_7395_6acc_524b;
    assert_eq!(
        VEHICLE_RACE_INNER_OPCODE,
        kart_racing::VEHICLE_RACE_INNER_OPCODE,
        "the entry handler shares the racing family's inner opcode"
    );
    assert_eq!(
        decode_register(&register_payload(session)).unwrap(),
        RegisterVehicleRacePlayer {
            session,
            word_8: 0x2a6a_c44a,
            word_12: 1,
        }
    );
    assert_eq!(
        decode_player_state(&join_payload(session, 2)).unwrap(),
        PlayerState {
            session,
            counter: 2,
        }
    );
    // Inner framing for the 33 family: `u16 opcode 33 | u16 subtype | payload`, no length field.
    let frame = encode_inner(9, &register_payload(session));
    assert_eq!(&frame[..4], &[0x21, 0x00, 0x09, 0x00]);
    assert_eq!(
        decode_inner(&frame).unwrap(),
        (9, register_payload(session).as_slice())
    );
    assert_eq!(
        decode_inner(&frame[..4]).unwrap(),
        (9, [].as_slice()),
        "header-only frame"
    );
    assert_eq!(
        decode_inner(&frame[..3]).unwrap_err(),
        EntryError::FrameTooShort { len: 3, needed: 4 }
    );
}

#[test]
fn private_content_package_format_round_trips_and_is_strict() {
    let content = synthetic_content();
    let json = serde_json::to_vec(&content).unwrap();
    let loaded: RaceEntryContent = serde_json::from_slice(&json).unwrap();
    assert_eq!(loaded, content);
    assert!(serde_json::from_slice::<RaceEntryContent>(&json).is_ok());
    let unknown_field = br#"{"race_config":{"words":[]},"entry_vehicles":[],
        "client_data":[],"progress":[],"player_count":0,"npc_count":0,"junk":1}"#;
    assert!(
        serde_json::from_slice::<RaceEntryContent>(unknown_field).is_err(),
        "the content package refuses unknown fields"
    );
    // Both paths drive the same ordered bundle from the same package.
    let mut derived = enabled(content);
    let mut packaged = enabled(loaded);
    for handler in [&mut derived, &mut packaged] {
        handler.open(synthetic_context()).unwrap();
    }
    let frame = encode_inner(9, &register_payload(synthetic_context().session));
    assert_eq!(
        derived.handle_inner(&frame).unwrap(),
        packaged.handle_inner(&frame).unwrap()
    );
}

/// Private gate: re-encode the original capture through the entry handler. Requires
/// `OSFR_KART_ENTRY_CAPTURE` to point at the preserved frame directory
/// (`lab/reports/original-minigame-packets`). No captured bytes are committed here.
#[test]
#[ignore = "requires OSFR_KART_ENTRY_CAPTURE pointing at lab/reports/original-minigame-packets"]
fn private_capture_reencodes_the_full_entry_bundle() {
    let root = std::path::PathBuf::from(
        std::env::var_os("OSFR_KART_ENTRY_CAPTURE")
            .expect("OSFR_KART_ENTRY_CAPTURE must name the preserved frame directory"),
    );
    const PINNED: [(&str, &str); 11] = [
        (
            "racing-24385-flow1-frame15480-op33-sub9.bin",
            "12c8181472fcab0aacb704ff80a073d28b0222bbe0a118f4708c07d2b37dfa73",
        ),
        (
            "racing-13265-flow0-frame15933-op33-sub34.bin",
            "3b0b36d42e20519fa70b2115a22793a661bd66578b2b007ac153cf262b7a6a55",
        ),
        (
            "racing-13266-flow0-frame15934-op33-sub11.bin",
            "22cb1fd8a3695dc525017cdefee2d96fae674f74cb44b6760b2b8787e391fcca",
        ),
        (
            "racing-13267-flow0-frame15948-op33-sub5.bin",
            "0c7e69b92b9714432770e08ed5581c58067cff47a3f52d80379ca8363b2d0aa6",
        ),
        (
            "racing-13268-flow0-frame15948-op33-sub14.bin",
            "25156096ff268a7ddcfd607f236e43fbaaaf1f546c416e2ccdc3a4e010a319ee",
        ),
        (
            "racing-24386-flow1-frame15950-op33-sub2.bin",
            "76e41a59dee208501fed65abcfa2d351b5eca6649b0b4d7604819b5b1d834ee8",
        ),
        (
            "racing-13269-flow0-frame16298-op33-sub3.bin",
            "41215bac51776fe6d891badc902989f3e9b39522282aefa567b8286fb62a2f48",
        ),
        (
            "racing-13270-flow0-frame16298-op33-sub1.bin",
            "0e5c9f444858e85418a0152fa7d1e73e9d0ff176e14715e87dd85c44295304e3",
        ),
        (
            "racing-13294-flow0-frame17362-op33-sub1.bin",
            "76379d5adbd76362715395ade0f74d395e0e7e41af44469fde6e83f9e0c0ac5b",
        ),
        (
            "racing-13293-flow0-frame17362-op33-sub4.bin",
            "ead42b6e3e4a53104d102bd15e4baf1f6207f2dce877042e9d5e47493925a965",
        ),
        (
            "racing-13071-flow0-frame723-op31-subNone.bin",
            "bace4cbaef5d50eb3601543847e77e1b4058ce765405f91556dd386f8e32c373",
        ),
    ];
    let mut frames: std::collections::BTreeMap<&str, Vec<u8>> = std::collections::BTreeMap::new();
    for (name, sha) in PINNED {
        let raw = std::fs::read(root.join(name)).expect("readable pinned capture frame");
        assert_eq!(digest(&raw), sha, "{name} must be the pinned capture");
        frames.insert(name, raw);
    }
    let frame = |name: &str| -> &[u8] { frames.get(name).expect("pinned frame").as_slice() };

    // The scene comes from the registered zone-transition codec, with rotation w = 0 preserved.
    let scene = decode_zone(frame("racing-13071-flow0-frame723-op31-subNone.bin")).unwrap();
    assert_eq!(scene.scene, "sg_kt_track");
    assert_eq!(scene.rotation, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(scene.sky, "sky.xml");

    // Encounter + instance are read from the captured 41/114 body, not invented here.
    let details = std::fs::read(root.join("racing-13236-flow0-frame15449-op41-sub114.bin")).ok();
    let (encounter_id, instance_id) = match &details {
        Some(raw) if raw.len() >= 19 => (
            u32::from_le_bytes(raw[11..15].try_into().unwrap()),
            u32::from_le_bytes(raw[15..19].try_into().unwrap()),
        ),
        _ => (917_566, 1008),
    };
    assert_ne!(encounter_id, 0);
    assert_ne!(instance_id, 0);

    let register =
        decode_register(&frame("racing-24385-flow1-frame15480-op33-sub9.bin")[7 + 4..]).unwrap();
    // The contract note renders the session as the wire byte order `91ca73956acc524b`; read little
    // endian (as every other field in this family) that is `0x4b52cc6a9573ca91`.
    assert_eq!(register.session, 0x4b52_cc6a_9573_ca91);
    assert_eq!(
        register.session.to_le_bytes(),
        [0x91, 0xca, 0x73, 0x95, 0x6a, 0xcc, 0x52, 0x4b]
    );
    assert_eq!(register.word_8, 0x2a6a_c44a);
    assert_eq!(register.word_12, 1);

    // Content is derived from the capture: config words, the entry kart, the bulk block, progress.
    let config_payload = &frame("racing-13265-flow0-frame15933-op33-sub34.bin")[11..];
    let words: Vec<u32> = config_payload
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();
    assert_eq!(words.len(), RACE_CONFIG_WORDS);
    let vehicle_payload = &frame("racing-13266-flow0-frame15934-op33-sub11.bin")[11..];
    let name_length = u32::from_le_bytes(vehicle_payload[0..4].try_into().unwrap()) as usize;
    let body_asset = String::from_utf8(vehicle_payload[4..4 + name_length].to_vec()).unwrap();
    assert_eq!(body_asset, "kt_kart08_body_bbe.cdt");
    let first_attribute = f32::from_le_bytes(
        vehicle_payload[4 + name_length..8 + name_length]
            .try_into()
            .unwrap(),
    );
    assert_eq!(first_attribute, 400.0);
    let progress = decode_progress(
        &frame("racing-13268-flow0-frame15948-op33-sub14.bin")[11..],
        NonFinitePolicy::Preserve,
    )
    .unwrap();
    assert_eq!(progress.racer_guid, register.session);
    let player_count = u32::from_le_bytes(
        frame("racing-13269-flow0-frame16298-op33-sub3.bin")[11..]
            .try_into()
            .unwrap(),
    );
    let npc_count = u32::from_le_bytes(
        frame("racing-13293-flow0-frame17362-op33-sub4.bin")[11..]
            .try_into()
            .unwrap(),
    );
    let content = RaceEntryContent {
        race_config: RaceConfig { words },
        entry_vehicles: vec![ProxiedVehicle {
            body_asset,
            first_attribute,
            attribute_block: vehicle_payload[8 + name_length..].to_vec(),
        }],
        client_data: frame("racing-13267-flow0-frame15948-op33-sub5.bin")[11..].to_vec(),
        progress: vec![progress],
        player_count,
        npc_count,
    };
    // Optional: the same content as a private JSON package must load to exactly this content.
    if let Some(path) = std::env::var_os("OSFR_KART_ENTRY_CONTENT") {
        let bytes = std::fs::read(&path).expect("readable private content package");
        let packaged: RaceEntryContent =
            serde_json::from_slice(&bytes).expect("private content package parses");
        assert_eq!(
            packaged, content,
            "private content package must match the capture"
        );
    }

    let mut handler = KartEntryHandler::new(content).unwrap();
    handler.set_gate(EntryGate::Enabled);
    handler
        .open(RaceEntryContext {
            session: register.session,
            instance_id,
            encounter_id,
            scene,
        })
        .unwrap();
    // Scene frame: byte-identical to the captured op31 frame, rotation w = 0 included.
    assert_eq!(
        handler.scene_frame().unwrap().as_slice(),
        frame("racing-13071-flow0-frame723-op31-subNone.bin")
    );

    // 33/9 -> 33/34, 33/11, 33/5, 33/14: every inner frame byte-identical to the capture.
    let start = handler
        .handle_inner(&frame("racing-24385-flow1-frame15480-op33-sub9.bin")[7..])
        .unwrap();
    let expected_start = [
        "racing-13265-flow0-frame15933-op33-sub34.bin",
        "racing-13266-flow0-frame15934-op33-sub11.bin",
        "racing-13267-flow0-frame15948-op33-sub5.bin",
        "racing-13268-flow0-frame15948-op33-sub14.bin",
    ];
    let start_replies = replies(&start);
    assert_eq!(start_replies.len(), expected_start.len());
    for (reply, name) in start_replies.iter().zip(expected_start) {
        assert_eq!(reply.as_slice(), &frame(name)[7..], "{name} re-encode");
    }

    // 33/2 -> 33/3 (player count 1), 33/1 (state 3).
    let join = handler
        .handle_inner(&frame("racing-24386-flow1-frame15950-op33-sub2.bin")[7..])
        .unwrap();
    let join_replies = replies(&join);
    assert_eq!(
        join_replies[0].as_slice(),
        &frame("racing-13269-flow0-frame16298-op33-sub3.bin")[7..]
    );
    assert_eq!(
        join_replies[1].as_slice(),
        &frame("racing-13270-flow0-frame16298-op33-sub1.bin")[7..]
    );

    // Race-ready bundle: only the recovered state/count transitions (33/1 = 4, 33/4 = 7).
    let ready = handler.on_race_ready().unwrap();
    let ready_replies = replies(&ready);
    assert_eq!(
        ready_replies[0].as_slice(),
        &frame("racing-13294-flow0-frame17362-op33-sub1.bin")[7..]
    );
    assert_eq!(
        ready_replies[1].as_slice(),
        &frame("racing-13293-flow0-frame17362-op33-sub4.bin")[7..]
    );
    assert_eq!(handler.phase(), EntryPhase::Streaming);
}
