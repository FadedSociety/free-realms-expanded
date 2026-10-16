//! Independent review and repair guard suite for the Kart Racing bounded contract (MMO004).
//!
//! Every test here started as a counterexample in `notes/Reviews/Kart Independent Review.md` and now
//! guards the repaired behavior: no caller-supplied reward authority, session-local claims, lossless
//! tunnel-flag handling, clock-regression rejection, coherent snapshot restore with a kept price
//! schedule, and strict asset-name validation. The capture vectors are re-verified against archived
//! file hashes in `lab/reports/kart-review-repair-20261009T031700Z/`.
//!
use osfr_tools::kart_racing::{
    CheckpointOutcome, FrameError, KART_RACE_GAME_TYPE, KART_RACE_LOBBY_NAME_ID, KartRacingError,
    LobbyGameDefinition, LobbyGameEntry, ObservationOutcome, Phase, RACE_PROGRESS_BODY_LEN,
    RACER_STATE_BODY_LEN, RaceSession, RewardProposal, RewardSchedule, TUNNEL_OUTER_OPCODES,
    TrackLayout, decode_racing_frame, vehicle_body_asset,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

fn priced_track(laps: u32) -> TrackLayout {
    TrackLayout::new(
        31,
        laps,
        vec![11, 12],
        Some(RewardSchedule { coins: 10, xp: 25 }),
    )
    .unwrap()
}

fn unpriced_track(laps: u32) -> TrackLayout {
    TrackLayout::new(31, laps, vec![11, 12], None).unwrap()
}

/// Community serializer oracle, re-derived independently from the pinned Sanctuary sources
/// (`lobby-game-definition-community.bin`, verified hash `e088a1f9…`) in
/// `lab/reports/kart-review-20261009T024747Z/verify-kart-capture.py`. This is the community subset,
/// not the native 102/2 wire framing.
const LOBBY_ORACLE_HEX: &str = "0300000001000000010000000100000025000000d60b000002000000020000000100000025000000ec0b000003000000030000000200000025000000ed0b000000000000000000000000000000000000";
const LOBBY_ORACLE_SHA256: &str =
    "e088a1f906696bf853fffa5766afebf7a990e31635210f7b7cb2a26309f7e113";

/// The six builder vectors, matched one-to-one to archived capture files by SHA-256 during review
/// (see `lab/reports/kart-review-20261009T024747Z/capture-verification.json`). The sub2 vector is a
/// *client->server* frame (`racing-24386-flow1-frame15950-op33-sub2.bin`, UDP flow
/// 192.168.1.145:61759 -> 64.37.170.6:20267) and still carries outer opcode 5, so outer opcode does
/// not encode direction.
const CAPTURED_FRAMES: [(&str, u16, usize, &str); 6] = [
    (
        "050001080000002100030001000000",
        3,
        4,
        "41215bac51776fe6d891badc902989f3e9b39522282aefa567b8286fb62a2f48",
    ),
    (
        "050001100000002100020091ca73956acc524b02000000",
        2,
        12,
        "76e41a59dee208501fed65abcfa2d351b5eca6649b0b4d7604819b5b1d834ee8",
    ),
    (
        "05000159000000210005000000803f0000000000000000000000000000803f0000000000000000000000000000803f00c046444861ac420080b64300000000000000000000000000000000000000000000000000000000004ac46a2a01000000",
        5,
        85,
        "0c7e69b92b9714432770e08ed5581c58067cff47a3f52d80379ca8363b2d0aa6",
    ),
    (
        "0500014d00000021000e0091ca73956acc524b0000000000000000ffff7f7f00000000000300000000000000000000000000000091ca73956acc524b00000000000000000000803f000000000000000000000000",
        14,
        73,
        "25156096ff268a7ddcfd607f236e43fbaaaf1f546c416e2ccdc3a4e010a319ee",
    ),
    (
        "0500010400000021001700",
        23,
        0,
        "a461cc08479de74dc96d8a588e9d0f2403359eebd44981bd6a541746b63f5a4d",
    ),
    (
        "0500012400000021000f0001000000f1010000bc8c204400008d42d17205440000803f0000803f2c050000",
        15,
        32,
        "27a773b09822e8eaa89d7cc07d39bd1ac8da7aaacaf87a04de90903baefda9fb",
    ),
];

/// Flag-0 bulk racer streams the repaired decoder accepts losslessly. Files:
/// `racing-01135-flow0-frame27068-op33-sub5.bin` (8,176 of 8,182 sub5 frames are flag 0) and
/// `racing-00002-flow1-frame21194-op33-sub6.bin` (all 4,679 sub6 client frames are flag 0).
const FLAG_ZERO_SUB5_HEX: &str = "05000059000000210005000000803fcb4f64348f1d4734498761b4b3f97f3fb13563bc33434ab4b135633cb3f97f3f004044444698aa42bd7fcf436cee96357eceac3e2c3c833980b5a4ba00004031000085b600000000381a73beab01000000";
const FLAG_ZERO_SUB5_SHA256: &str =
    "87f13f699f565289bb596ef8b66d50dac080d3dbe25cf0d963a72f2757d84ea7";
const FLAG_ZERO_SUB6_HEX: &str = "0500005900000021000600feff7f3f000000000000000000000000fdff7f3f000000000000000000000000fbff7f3f00c046444861ac420080b64300000000000000000000000000000000000000000000000000000000004ac46a2a01000000";
const FLAG_ZERO_SUB6_SHA256: &str =
    "990ccb07bb94499681d95f53f77fa45496cd3444256e6da9d3129cd406fe865e";

/// The captured opcode 102/2 lobby catalogue payload carries a length prefix and a six-row body
/// before the community subset. First 48 bytes of
/// `racing-23867-flow0-frame44523-op102-sub2.bin` (full frame SHA-256
/// `4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932`, 2,393 bytes).
const OP102_SUB2_HEAD_HEX: &str = "05000152090000660002004a0900000600000001000000010000000100000025000000d60b0000020000000200000002";

#[test]
fn builder_capture_vectors_are_real_archive_bytes() {
    for (hex, subtype, payload_len, sha256) in CAPTURED_FRAMES {
        let packet = bytes(hex);
        assert_eq!(
            digest(&packet),
            sha256,
            "capture hash for subtype {subtype}"
        );
        let frame = decode_racing_frame(&packet).unwrap();
        assert!(TUNNEL_OUTER_OPCODES.contains(&frame.outer_opcode));
        assert_eq!(frame.flag, 1, "subtype {subtype}");
        assert_eq!(frame.subtype, subtype);
        assert_eq!(frame.payload.len(), payload_len);
        assert_eq!(frame.payload.len(), packet.len() - 11);
    }

    // The sub2 vector is a client->server frame by UDP flow and still uses outer opcode 5. No op33
    // frame in the archive carries outer 6 (review bundle `capture-verification.json`).
    let client_packet = bytes(CAPTURED_FRAMES[1].0);
    let client_frame = decode_racing_frame(&client_packet).unwrap();
    assert_eq!(client_frame.outer_opcode, 5);
    assert_eq!(client_frame.flag, 1);
}

#[test]
fn flag_zero_and_one_frames_decode_losslessly() {
    for (hex, sha256, label) in [
        (FLAG_ZERO_SUB5_HEX, FLAG_ZERO_SUB5_SHA256, "sub5"),
        (FLAG_ZERO_SUB6_HEX, FLAG_ZERO_SUB6_SHA256, "sub6"),
    ] {
        let packet = bytes(hex);
        assert_eq!(digest(&packet), sha256, "{label} archive hash");
        assert_eq!(packet[2], 0, "{label} tunnel flag");
        let frame = decode_racing_frame(&packet).unwrap();
        assert_eq!(frame.flag, 0, "{label}: flag is preserved, not interpreted");
        assert_eq!(frame.payload.len(), RACER_STATE_BODY_LEN);
    }

    let flag_one_packet = bytes(CAPTURED_FRAMES[2].0);
    let flag_one = decode_racing_frame(&flag_one_packet).unwrap();
    assert_eq!(flag_one.flag, 1);

    let mut bad_flag = bytes(CAPTURED_FRAMES[0].0);
    bad_flag[2] = 2;
    assert_eq!(
        decode_racing_frame(&bad_flag),
        Err(FrameError::UnsupportedTunnelFlag(2))
    );
}

#[test]
fn lobby_oracle_re_derives_from_the_community_layout() {
    // Re-derive the community writer's bytes from its pinned source layout instead of trusting the
    // constants: i32 count, then per entry [key][id][type][unknown][name_id], then four zero counts.
    let rows = [
        (1i32, 1i32, KART_RACE_GAME_TYPE, 37i32, 3030i32),
        (2, 2, KART_RACE_GAME_TYPE, 37, KART_RACE_LOBBY_NAME_ID),
        (3, 3, 2, 37, 3053),
    ];
    let mut expected = Vec::new();
    expected.extend((rows.len() as i32).to_le_bytes());
    for (key, id, game_type, unknown, name_id) in rows {
        for field in [key, id, game_type, unknown, name_id] {
            expected.extend(field.to_le_bytes());
        }
    }
    expected.extend([0u8; 16]);
    assert_eq!(expected, bytes(LOBBY_ORACLE_HEX));
    assert_eq!(digest(&expected), LOBBY_ORACLE_SHA256);

    let module = LobbyGameDefinition::new(vec![
        LobbyGameEntry {
            id: 1,
            game_type: KART_RACE_GAME_TYPE,
            unknown: 37,
            name_id: 3030,
        },
        LobbyGameEntry {
            id: 2,
            game_type: KART_RACE_GAME_TYPE,
            unknown: 37,
            name_id: KART_RACE_LOBBY_NAME_ID,
        },
        LobbyGameEntry {
            id: 3,
            game_type: 2,
            unknown: 37,
            name_id: 3053,
        },
    ]);
    assert_eq!(module.encode(), expected);
}

#[test]
fn captured_lobby_payload_has_a_length_prefix_and_six_rows() {
    let prefix = bytes(OP102_SUB2_HEAD_HEX);
    assert_eq!(prefix.len(), 48);
    let outer = u16::from_le_bytes([prefix[0], prefix[1]]);
    let flag = prefix[2];
    let declared = u32::from_le_bytes([prefix[3], prefix[4], prefix[5], prefix[6]]) as usize;
    let inner = u16::from_le_bytes([prefix[7], prefix[8]]);
    let subtype = u16::from_le_bytes([prefix[9], prefix[10]]);
    assert_eq!(
        (outer, flag, declared, inner, subtype),
        (5, 1, 2386, 102, 2)
    );
    // Full frame 2,393 bytes -> payload 2,382 bytes; the module only supports inner opcode 33.
    assert_eq!(2393 - 11, 2382);
    assert_eq!(
        decode_racing_frame(&prefix),
        Err(FrameError::LengthMismatch {
            declared: 2386,
            available: 41
        })
    );

    let payload = &prefix[11..];
    let leading_field = u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize;
    let row_count = u32::from_le_bytes(payload[4..8].try_into().unwrap());
    let row0 = (
        i32::from_le_bytes(payload[8..12].try_into().unwrap()),
        i32::from_le_bytes(payload[12..16].try_into().unwrap()),
        i32::from_le_bytes(payload[16..20].try_into().unwrap()),
        i32::from_le_bytes(payload[20..24].try_into().unwrap()),
        i32::from_le_bytes(payload[24..28].try_into().unwrap()),
    );
    assert_eq!(leading_field, 2382 - 4, "u32 remaining-length prefix");
    assert_eq!(row_count, 6);
    assert_eq!(row0, (1, 1, 1, 37, 3030));
}

#[test]
fn checkpoint_skips_stale_replays_and_unknowns_never_advance_progress() {
    let track = TrackLayout::new(31, 2, vec![11, 12, 13], None).unwrap();
    let mut session = RaceSession::open(0x4B52_CC6A, track, 1_000).unwrap();

    assert_eq!(
        session.on_checkpoint(1_050, 11),
        CheckpointOutcome::NotRacing
    );
    session.begin(1_100).unwrap();
    assert_eq!(
        session.on_checkpoint(1_150, 11),
        CheckpointOutcome::NotRacing,
        "countdown frames are not race frames"
    );
    session.start(1_200).unwrap();

    let clean = session.snapshot();
    assert_eq!(
        session.on_checkpoint(1_300, 12),
        CheckpointOutcome::OutOfOrder {
            expected: 11,
            got: 12
        }
    );
    assert_eq!(
        session.on_checkpoint(1_350, 13),
        CheckpointOutcome::OutOfOrder {
            expected: 11,
            got: 13
        }
    );
    assert_eq!(
        session.on_checkpoint(1_400, 99),
        CheckpointOutcome::UnknownCheckpoint(99)
    );
    assert_eq!(
        session.snapshot(),
        clean,
        "rejected gates must not mutate state"
    );

    assert_eq!(
        session.on_checkpoint(1_500, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(1_550, 11),
        CheckpointOutcome::Duplicate
    );
    assert_eq!(
        session.on_checkpoint(1_600, 13),
        CheckpointOutcome::OutOfOrder {
            expected: 12,
            got: 13
        },
        "skipping the next gate is refused"
    );
    assert_eq!(
        session.on_checkpoint(1_650, 12),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 1 }
    );
    assert_eq!(
        session.on_checkpoint(1_700, 13),
        CheckpointOutcome::LapCompleted {
            lap: 1,
            lap_ms: 500
        }
    );

    // Stale delivery after the lap rolls over: the last gate of lap 1 is a duplicate, the middle
    // gate of the completed lap is out of order, and only gate 11 starts lap 2.
    assert_eq!(
        session.on_checkpoint(1_750, 13),
        CheckpointOutcome::Duplicate
    );
    assert_eq!(
        session.on_checkpoint(1_800, 12),
        CheckpointOutcome::OutOfOrder {
            expected: 11,
            got: 12
        }
    );
    assert_eq!(
        session.on_checkpoint(1_900, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 2, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(2_100, 12),
        CheckpointOutcome::CheckpointAccepted { lap: 2, index: 1 }
    );
    assert_eq!(
        session.on_checkpoint(2_300, 13),
        CheckpointOutcome::LapCompleted {
            lap: 2,
            lap_ms: 600
        }
    );
    assert_eq!(session.phase(), Phase::Finished);

    // Replayed checkpoints after the finish are inert, and the recorded progress is unchanged.
    assert_eq!(
        session.on_checkpoint(2_400, 11),
        CheckpointOutcome::NotRacing
    );
    let final_state = session.snapshot();
    assert_eq!(final_state.laps_completed, 2);
    assert_eq!(final_state.best_lap_ms, Some(500));
    assert_eq!(final_state.next_index, 0);
    assert_eq!(
        session.on_checkpoint(2_500, 12),
        CheckpointOutcome::NotRacing
    );
    assert_eq!(session.snapshot(), final_state);
}

#[test]
fn session_claims_are_local_and_the_id_is_exposed_for_provenance() {
    let mut finisher = RaceSession::open(9, priced_track(1), 0).unwrap();
    assert_eq!(finisher.session_id(), 9);
    finisher.begin(0).unwrap();
    finisher.start(0).unwrap();
    finisher.on_checkpoint(10, 11);
    finisher.on_checkpoint(20, 12);
    let record = finisher.finish(20).unwrap();
    assert_eq!(record.session_id, 9);
    assert_eq!(
        finisher.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 })
    );
    assert_eq!(
        finisher.claim_reward(),
        Err(KartRacingError::RewardAlreadyClaimed)
    );

    // A different session object that merely reuses session id 9 has no claim authority: the grant
    // is derived from the object's own validated state, never from a shared result record.
    let mut impostor = RaceSession::open(9, unpriced_track(1), 0).unwrap();
    impostor.begin(0).unwrap();
    impostor.start(0).unwrap();
    impostor.abandon();
    assert_eq!(
        impostor.claim_reward(),
        Err(KartRacingError::NotFinished),
        "claims are session-local"
    );

    let mut stranger = RaceSession::open(10, unpriced_track(1), 0).unwrap();
    stranger.begin(0).unwrap();
    stranger.start(0).unwrap();
    stranger.abandon();
    assert_eq!(stranger.claim_reward(), Err(KartRacingError::NotFinished));
}

#[test]
fn clock_regressions_are_rejected_before_mutations() {
    let mut session = RaceSession::open(5, unpriced_track(1), 5_000).unwrap();
    session.begin(5_000).unwrap();

    // Start cannot move the timing base backwards.
    assert_eq!(session.start(4_000), Err(KartRacingError::ClockRegression));
    assert_eq!(session.phase(), Phase::Countdown);
    session.start(5_000).unwrap();

    let clean = session.snapshot();
    assert_eq!(
        session.on_checkpoint(4_000, 11),
        CheckpointOutcome::ClockRegression {
            now_ms: 4_000,
            since_ms: 5_000
        }
    );
    assert_eq!(
        session.snapshot(),
        clean,
        "a regressed clock changes nothing"
    );

    assert_eq!(
        session.on_checkpoint(5_000, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(4_000, 12),
        CheckpointOutcome::ClockRegression {
            now_ms: 4_000,
            since_ms: 5_000
        }
    );
    assert_eq!(session.laps_completed(), 0);
    assert_eq!(session.best_lap_ms(), None);

    assert_eq!(
        session.on_checkpoint(6_000, 12),
        CheckpointOutcome::LapCompleted {
            lap: 1,
            lap_ms: 1_000
        }
    );
    assert_eq!(session.phase(), Phase::Finished);
    assert_eq!(session.finish(4_000), Err(KartRacingError::ClockRegression));
    assert_eq!(session.finish(6_000).unwrap().total_ms, 1_000);
}

#[test]
fn reward_authority_is_derived_from_the_priced_run_only() {
    // Abandoned and mid-race sessions have no grant authority, priced or not.
    let mut abandoned = RaceSession::open(7, unpriced_track(2), 0).unwrap();
    abandoned.begin(0).unwrap();
    abandoned.start(0).unwrap();
    abandoned.abandon();
    assert_eq!(abandoned.claim_reward(), Err(KartRacingError::NotFinished));

    let mut racer = RaceSession::open(8, priced_track(2), 0).unwrap();
    racer.begin(0).unwrap();
    racer.start(0).unwrap();
    racer.on_checkpoint(10, 11);
    racer.on_checkpoint(20, 12);
    assert_eq!(racer.phase(), Phase::Racing);
    assert_eq!(racer.finish(30), Err(KartRacingError::NotFinished));
    assert_eq!(racer.claim_reward(), Err(KartRacingError::NotFinished));

    racer.on_checkpoint(30, 11);
    racer.on_checkpoint(40, 12);
    assert_eq!(racer.phase(), Phase::Finished);
    assert_eq!(
        racer.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 }),
        "the grant is the layout's price, not a caller's number"
    );
    assert_eq!(
        racer.claim_reward(),
        Err(KartRacingError::RewardAlreadyClaimed)
    );

    // A finished but unpriced run never grants and never consumes the one-shot marker.
    let mut unpriced = RaceSession::open(12, unpriced_track(1), 0).unwrap();
    unpriced.begin(0).unwrap();
    unpriced.start(0).unwrap();
    unpriced.on_checkpoint(10, 11);
    unpriced.on_checkpoint(20, 12);
    assert_eq!(unpriced.claim_reward(), Ok(RewardProposal::Unpriced));
    assert_eq!(unpriced.claim_reward(), Ok(RewardProposal::Unpriced));
    assert!(!unpriced.snapshot().reward_claimed);
}

#[test]
fn finish_is_only_valid_on_a_finished_race_and_repeats_are_reads() {
    let mut session = RaceSession::open(4, priced_track(1), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    session.on_checkpoint(10, 11);
    assert_eq!(session.finish(20), Err(KartRacingError::NotFinished));

    session.on_checkpoint(20, 12);
    let first = session.finish(100).unwrap();
    let second = session.finish(200).unwrap();
    assert_eq!(first.reward, RewardProposal::Granted { coins: 10, xp: 25 });
    assert_eq!(second.reward, RewardProposal::Granted { coins: 10, xp: 25 });
    assert!(second.total_ms > first.total_ms);
    assert_eq!(
        session.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 })
    );
    assert_eq!(
        session.claim_reward(),
        Err(KartRacingError::RewardAlreadyClaimed),
        "the one-shot guard lives in claim_reward, not in finish"
    );
}

fn restorable_snapshot() -> Value {
    json!({
        "session_id": 3,
        "speedway": 31,
        "laps": 2,
        "checkpoints": [11, 12],
        "reward": { "coins": 10, "xp": 25 },
        "phase": "racing",
        "started_ms": 0,
        "lap_started_ms": 0,
        "next_index": 0,
        "laps_completed": 0,
        "best_lap_ms": null,
        "last_checkpoint": null,
        "reward_claimed": false,
        "observed_racer_states": 0
    })
}

#[test]
fn restore_rejects_incoherent_snapshots_and_keeps_the_reward_schedule() {
    // A valid mid-race snapshot keeps its price schedule across the restart.
    let mut session = RaceSession::open(3, priced_track(2), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    session.on_checkpoint(10, 11);
    let snapshot = session.snapshot();
    assert_eq!(snapshot.reward, Some(RewardSchedule { coins: 10, xp: 25 }));
    let mut resumed = RaceSession::restore(snapshot).unwrap();
    resumed.on_checkpoint(20, 12);
    resumed.on_checkpoint(30, 11);
    resumed.on_checkpoint(40, 12);
    assert_eq!(resumed.phase(), Phase::Finished);
    assert_eq!(
        resumed.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 }),
        "restore keeps the schedule instead of dropping to Unpriced"
    );

    // A coherent mid-race snapshot restores (positive control).
    let mut mid_lap = restorable_snapshot();
    mid_lap["next_index"] = json!(1);
    mid_lap["last_checkpoint"] = json!(11);
    let restored = RaceSession::restore(serde_json::from_value(mid_lap).unwrap()).unwrap();
    assert_eq!(restored.next_checkpoint(), Some(12));
    assert_eq!(restored.laps_completed(), 0);

    let cases = [
        ("next gate without a recorded last checkpoint", {
            let mut v = restorable_snapshot();
            v["next_index"] = json!(1);
            v
        }),
        ("finished with zero laps", {
            let mut v = restorable_snapshot();
            v["phase"] = json!("finished");
            v
        }),
        ("best lap without any completed lap", {
            let mut v = restorable_snapshot();
            v["best_lap_ms"] = json!(1);
            v
        }),
        ("completed lap without a best lap", {
            let mut v = restorable_snapshot();
            v["laps_completed"] = json!(1);
            v["last_checkpoint"] = json!(12);
            v
        }),
        ("claimed reward while still racing", {
            let mut v = restorable_snapshot();
            v["reward_claimed"] = json!(true);
            v
        }),
        ("empty price schedule", {
            let mut v = restorable_snapshot();
            v["reward"] = json!({ "coins": 0, "xp": 0 });
            v
        }),
        ("lap started before the session", {
            let mut v = restorable_snapshot();
            v["started_ms"] = json!(10);
            v
        }),
        ("next index past the last gate", {
            let mut v = restorable_snapshot();
            v["next_index"] = json!(2);
            v
        }),
        ("more laps than the track", {
            let mut v = restorable_snapshot();
            v["laps_completed"] = json!(3);
            v
        }),
    ];
    for (label, value) in cases {
        assert_eq!(
            RaceSession::restore(serde_json::from_value(value).unwrap()).err(),
            Some(KartRacingError::CorruptSnapshot),
            "{label}"
        );
    }
}

#[test]
fn opaque_racer_streams_never_advance_progress_even_with_archived_flag_zero_bytes() {
    let mut session = RaceSession::open(11, unpriced_track(2), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();

    let server_stream = bytes(FLAG_ZERO_SUB5_HEX);
    let frame = decode_racing_frame(&server_stream).unwrap();
    assert_eq!(frame.flag, 0);
    assert_eq!(
        session.observe_racer_state(frame.payload),
        ObservationOutcome::Recorded {
            bytes: RACER_STATE_BODY_LEN
        }
    );
    let progress_stream = bytes(CAPTURED_FRAMES[3].0);
    let progress_frame = decode_racing_frame(&progress_stream).unwrap();
    assert_eq!(
        session.observe_racer_state(progress_frame.payload),
        ObservationOutcome::Recorded {
            bytes: RACE_PROGRESS_BODY_LEN
        }
    );
    assert_eq!(
        session.observe_racer_state(&[0u8; 8]),
        ObservationOutcome::UnexpectedLength { bytes: 8 }
    );
    assert_eq!(session.observed_racer_states(), 2);
    assert_eq!(session.next_checkpoint(), Some(11));
    assert_eq!(session.laps_completed(), 0);
    assert_eq!(session.phase(), Phase::Racing);
}

#[test]
fn asset_name_decoder_rejects_empty_truncated_nul_and_non_utf8_names() {
    // The captured subtype-11 encoding is length-prefixed, not NUL-terminated (the bytes after the
    // name are the 400.0 attribute), so a complete window without a NUL is a valid name.
    assert_eq!(
        vehicle_body_asset(&bytes("0000000000000000")),
        Err(FrameError::InvalidVehicleName)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("0100000000000000")),
        Err(FrameError::InvalidVehicleName)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("0500000061626364")),
        Err(FrameError::TooShort),
        "a declared window that runs past the payload is truncated"
    );
    assert_eq!(
        vehicle_body_asset(&bytes("050000006162006465")),
        Err(FrameError::InvalidVehicleName),
        "a NUL inside the declared window is not a usable name"
    );
    assert_eq!(
        vehicle_body_asset(&bytes("02000000fffe0000")),
        Err(FrameError::InvalidUtf8)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("ff00000000000000")),
        Err(FrameError::TooShort)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("050000006162636465")),
        Ok("abcde")
    );
    assert_eq!(
        vehicle_body_asset(&bytes(
            "160000006b745f6b61727430385f626f64795f6262652e6364740000c843"
        )),
        Ok("kt_kart08_body_bbe.cdt")
    );
}
