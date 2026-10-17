//! Kart Racing (original lobby game type 1) bounded-contract tests.
//!
//! Original evidence vs reconstruction is labelled per vector. Captured frame bytes and hashes come
//! from the archived community capture `references/OpenSourceFreeRealms/ofrserver/Packets/
//! packets_racing_minigame.pcapng`; the extracted per-frame bodies live outside the repository under
//! `lab/reports/original-minigame-packets/racing-*.bin` and the recovered schema is recorded in
//! `lab/reports/kart-racing-20261009T0158Z/kart-racing-wire-schema.json`.
use osfr_tools::kart_racing::{
    self, CheckpointOutcome, FrameError, KART_DRIVER_JOB_GUID, KART_DRIVER_JOB_NAME_ID,
    KART_RACE_GAME_TYPE, KART_RACE_LOBBY_NAME_ID, KartRacingError, LobbyGameDefinition,
    LobbyGameEntry, ObservationOutcome, Phase, RACE_PROGRESS_BODY_LEN, RACER_STATE_BODY_LEN,
    RaceSession, RewardProposal, RewardSchedule, SPEEDWAYS, TrackLayout, decode_racing_frame,
    vehicle_body_asset,
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

/// Unchanged-community-serializer oracle output: `LobbyGameDefinition` with three entries
/// (1/type1/37/3030, 2/type1/37/3052, 3/type2/37/3053) written by the pinned Sanctuary writer.
/// Produced 2026-10-09T02:0xZ by `lab/reports/kart-racing-20261009T0158Z/oracle` (SHA-256 recorded).
/// This is the community serializer subset; it is not the native opcode 102/2 wire framing.
const LOBBY_ORACLE_HEX: &str = "0300000001000000010000000100000025000000d60b000002000000020000000100000025000000ec0b000003000000030000000200000025000000ed0b000000000000000000000000000000000000";
const LOBBY_ORACLE_SHA256: &str =
    "e088a1f906696bf853fffa5766afebf7a990e31635210f7b7cb2a26309f7e113";

/// Captured racing frames: (frame hex, subtype, payload length, capture SHA-256). Payload length
/// excludes the 7-byte tunnel header, the 2-byte inner opcode and the 2-byte subtype; the archived
/// file length is therefore payload + 11. All six carry tunnel flag 1.
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

/// Captured flag-0 racer streams, matched to their archived files by SHA-256:
/// `racing-01135-flow0-frame27068-op33-sub5.bin` (8,176 of 8,182 sub5 frames are flag 0) and
/// `racing-00002-flow1-frame21194-op33-sub6.bin` (all 4,679 sub6 frames are flag 0). The decoder
/// accepts both flags losslessly and does not interpret them as direction.
const FLAG_ZERO_FRAMES: [(&str, u16, &str); 2] = [
    (
        "05000059000000210005000000803fcb4f64348f1d4734498761b4b3f97f3fb13563bc33434ab4b135633cb3f97f3f004044444698aa42bd7fcf436cee96357eceac3e2c3c833980b5a4ba00004031000085b600000000381a73beab01000000",
        5,
        "87f13f699f565289bb596ef8b66d50dac080d3dbe25cf0d963a72f2757d84ea7",
    ),
    (
        "0500005900000021000600feff7f3f000000000000000000000000fdff7f3f000000000000000000000000fbff7f3f00c046444861ac420080b64300000000000000000000000000000000000000000000000000000000004ac46a2a01000000",
        6,
        "990ccb07bb94499681d95f53f77fa45496cd3444256e6da9d3129cd406fe865e",
    ),
];

#[test]
fn kart_race_lobby_definition_matches_the_unchanged_community_serializer() {
    let definition = LobbyGameDefinition::new(vec![
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
    let encoded = definition.encode();
    assert_eq!(encoded.len(), 80);
    assert_eq!(digest(&encoded), LOBBY_ORACLE_SHA256);
    assert_eq!(encoded, bytes(LOBBY_ORACLE_HEX));
}

#[test]
fn captured_racing_frames_decode_with_their_recorded_flags_and_lengths() {
    for (hex, subtype, payload_len, sha256) in CAPTURED_FRAMES {
        let packet = bytes(hex);
        assert_eq!(
            digest(&packet),
            sha256,
            "captured frame hash for subtype {subtype}"
        );
        let frame = decode_racing_frame(&packet).unwrap();
        assert_eq!(frame.outer_opcode, 5);
        assert_eq!(frame.flag, 1, "captured flag for subtype {subtype}");
        assert_eq!(frame.subtype, subtype);
        assert_eq!(frame.payload.len(), payload_len);
    }

    for (hex, subtype, sha256) in FLAG_ZERO_FRAMES {
        let packet = bytes(hex);
        assert_eq!(
            digest(&packet),
            sha256,
            "captured flag-0 frame hash for subtype {subtype}"
        );
        assert_eq!(packet[2], 0);
        let frame = decode_racing_frame(&packet).unwrap();
        assert_eq!(frame.flag, 0, "flag 0 must decode losslessly");
        assert_eq!(frame.subtype, subtype);
        assert_eq!(frame.payload.len(), RACER_STATE_BODY_LEN);
    }
}

#[test]
fn racing_frame_decoding_rejects_malformed_tunnels() {
    let packet = bytes(CAPTURED_FRAMES[0].0);
    assert!(decode_racing_frame(&packet[..6]).is_err());
    for cut in 0..packet.len() {
        assert!(
            decode_racing_frame(&packet[..cut]).is_err(),
            "prefix of length {cut} must not decode"
        );
    }

    let mut wrong_outer = packet.clone();
    wrong_outer[0] = 7;
    assert_eq!(
        decode_racing_frame(&wrong_outer),
        Err(FrameError::UnsupportedOuterOpcode(7))
    );

    // Only 0 and 1 are captured tunnel flags; anything else is rejected.
    let mut wrong_flag = packet.clone();
    wrong_flag[2] = 2;
    assert_eq!(
        decode_racing_frame(&wrong_flag),
        Err(FrameError::UnsupportedTunnelFlag(2))
    );

    let mut wrong_inner = packet.clone();
    wrong_inner[7] = 0x22;
    assert_eq!(
        decode_racing_frame(&wrong_inner),
        Err(FrameError::UnsupportedInnerOpcode(0x22))
    );

    let mut short_inner = packet.clone();
    short_inner[3] = short_inner[3].wrapping_sub(1);
    assert_eq!(
        decode_racing_frame(&short_inner),
        Err(FrameError::LengthMismatch {
            declared: 7,
            available: 8
        })
    );

    let mut long_inner = packet.clone();
    long_inner[3] = long_inner[3].wrapping_add(1);
    assert_eq!(
        decode_racing_frame(&long_inner),
        Err(FrameError::LengthMismatch {
            declared: 9,
            available: 8
        })
    );
}

#[test]
fn vehicle_config_payload_yields_the_original_kart_body_asset() {
    // First 41 bytes of `racing-13266-flow0-frame15934-op33-sub11.bin`
    // (full-frame capture SHA-256 22cb1fd8a3695dc525017cdefee2d96fae674f74cb44b6760b2b8787e391fcca).
    let payload = bytes("160000006b745f6b61727430385f626f64795f6262652e6364740000c843");
    assert_eq!(
        vehicle_body_asset(&payload).unwrap(),
        "kt_kart08_body_bbe.cdt"
    );
    let declared = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
    assert_eq!(
        declared, 22,
        "the u32 is the name byte count; the captured name is not NUL-terminated"
    );
    // The bytes after the name are the attribute block, which starts with the 400.0 f32.
    assert_eq!(
        &payload[4 + declared as usize..8 + declared as usize],
        &[0x00, 0x00, 0xc8, 0x43]
    );
    assert_eq!(vehicle_body_asset(&payload[..4]), Err(FrameError::TooShort));

    // Empty, truncated and NUL-carrying windows are not usable names; a complete length-prefixed
    // window without a NUL is valid, because the captured encoding has no terminator.
    assert_eq!(
        vehicle_body_asset(&bytes("0100000000000000")),
        Err(FrameError::InvalidVehicleName)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("0000000000000000")),
        Err(FrameError::InvalidVehicleName)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("0500000061626364")),
        Err(FrameError::TooShort)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("050000006162006465")),
        Err(FrameError::InvalidVehicleName)
    );
    assert_eq!(
        vehicle_body_asset(&bytes("050000006162636465")),
        Ok("abcde")
    );
}

#[test]
fn checkpoint_progress_must_follow_the_configured_order() {
    let mut session = RaceSession::open(0x4B52_CC6A, track(), 1_000).unwrap();
    assert_eq!(
        session.on_checkpoint(1_100, 11),
        CheckpointOutcome::NotRacing
    );
    session.begin(1_100).unwrap();
    session.start(1_200).unwrap();
    assert_eq!(
        session.on_checkpoint(1_300, 12),
        CheckpointOutcome::OutOfOrder {
            expected: 11,
            got: 12
        }
    );
    assert_eq!(
        session.on_checkpoint(1_400, 99),
        CheckpointOutcome::UnknownCheckpoint(99)
    );
    assert_eq!(
        session.on_checkpoint(1_500, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 0 }
    );
    assert_eq!(session.laps_completed(), 0);
    assert_eq!(session.next_checkpoint(), Some(12));
}

#[test]
fn clock_regressions_are_rejected_before_any_progress() {
    let mut session = RaceSession::open(5, track(), 5_000).unwrap();
    session.begin(5_000).unwrap();
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
    assert_eq!(session.snapshot(), clean);

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
}

#[test]
fn duplicate_checkpoint_delivery_is_idempotent_and_survives_reconnect() {
    let mut session = RaceSession::open(7, track(), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    assert_eq!(
        session.on_checkpoint(1_000, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(1_050, 11),
        CheckpointOutcome::Duplicate
    );
    assert_eq!(
        session.on_checkpoint(2_000, 12),
        CheckpointOutcome::LapCompleted {
            lap: 1,
            lap_ms: 2_000
        }
    );

    let text = serde_json::to_vec(&session.snapshot()).unwrap();
    let mut resumed = RaceSession::restore(serde_json::from_slice(&text).unwrap()).unwrap();
    assert_eq!(resumed.phase(), Phase::Racing);
    assert_eq!(resumed.laps_completed(), 1);
    assert_eq!(resumed.best_lap_ms(), Some(2_000));
    assert_eq!(resumed.next_checkpoint(), Some(11));
    assert_eq!(
        resumed.on_checkpoint(3_100, 12),
        CheckpointOutcome::Duplicate
    );
    assert_eq!(resumed.laps_completed(), 1);
    assert_eq!(resumed.best_lap_ms(), Some(2_000));
}

#[test]
fn laps_record_the_best_time_and_finish_only_after_the_last_lap() {
    let mut session = RaceSession::open(9, track(), 0).unwrap();
    session.begin(0).unwrap();
    session.start(500).unwrap();

    assert_eq!(
        session.on_checkpoint(1_500, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 1, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(2_500, 12),
        CheckpointOutcome::LapCompleted {
            lap: 1,
            lap_ms: 2_000
        }
    );
    assert_eq!(session.phase(), Phase::Racing);
    assert_eq!(
        session.finish(2_600),
        Err(KartRacingError::NotFinished),
        "a mid-race finish is not a result"
    );
    assert_eq!(
        session.on_checkpoint(3_000, 11),
        CheckpointOutcome::CheckpointAccepted { lap: 2, index: 0 }
    );
    assert_eq!(
        session.on_checkpoint(3_800, 12),
        CheckpointOutcome::LapCompleted {
            lap: 2,
            lap_ms: 1_300
        }
    );
    assert_eq!(session.phase(), Phase::Finished);
    assert_eq!(session.laps_completed(), 2);
    assert_eq!(session.best_lap_ms(), Some(1_300));
    assert_eq!(
        session.on_checkpoint(4_000, 11),
        CheckpointOutcome::NotRacing
    );

    let proposal = session.finish(4_000).unwrap();
    assert_eq!(proposal.laps_completed, 2);
    assert_eq!(proposal.total_ms, 3_500);
    assert_eq!(proposal.best_lap_ms, Some(1_300));
    assert_eq!(proposal.placement, None);
    assert_eq!(proposal.reward, RewardProposal::Unpriced);
}

#[test]
fn client_racer_state_is_opaque_and_never_advances_progress() {
    let mut session = RaceSession::open(11, track(), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    assert_eq!(session.next_checkpoint(), Some(11));

    let captured_state = bytes(CAPTURED_FRAMES[2].0);
    let frame = decode_racing_frame(&captured_state).unwrap();
    assert_eq!(frame.payload.len(), RACER_STATE_BODY_LEN);
    assert_eq!(
        session.observe_racer_state(frame.payload),
        ObservationOutcome::Recorded {
            bytes: RACER_STATE_BODY_LEN
        }
    );
    assert_eq!(
        session.observe_racer_state(&[0u8; 8]),
        ObservationOutcome::UnexpectedLength { bytes: 8 }
    );
    assert_eq!(session.observed_racer_states(), 1);
    assert_eq!(session.next_checkpoint(), Some(11));
    assert_eq!(session.laps_completed(), 0);
    assert_eq!(
        session.observe_racer_state(&[0xAAu8; RACE_PROGRESS_BODY_LEN]),
        ObservationOutcome::Recorded {
            bytes: RACE_PROGRESS_BODY_LEN
        }
    );
    assert_eq!(session.phase(), Phase::Racing);
}

#[test]
fn reward_stays_unpriced_without_evidence_and_is_granted_only_once() {
    let mut unpriced = RaceSession::open(1, track(), 0).unwrap();
    unpriced.begin(0).unwrap();
    unpriced.start(0).unwrap();
    unpriced.on_checkpoint(10, 11);
    unpriced.on_checkpoint(20, 12);
    unpriced.on_checkpoint(30, 11);
    unpriced.on_checkpoint(40, 12);
    assert_eq!(
        unpriced.finish(40).unwrap().reward,
        RewardProposal::Unpriced
    );
    assert_eq!(unpriced.claim_reward(), Ok(RewardProposal::Unpriced));
    assert_eq!(unpriced.claim_reward(), Ok(RewardProposal::Unpriced));

    let mut scheduled = RaceSession::open(
        2,
        TrackLayout::new(
            31,
            1,
            vec![11, 12],
            Some(kart_racing::RewardSchedule { coins: 10, xp: 25 }),
        )
        .unwrap(),
        0,
    )
    .unwrap();
    scheduled.begin(0).unwrap();
    scheduled.start(0).unwrap();
    scheduled.on_checkpoint(10, 11);
    assert_eq!(
        scheduled.claim_reward(),
        Err(KartRacingError::NotFinished),
        "the grant is not available before the validated finish"
    );
    scheduled.on_checkpoint(20, 12);
    assert_eq!(
        scheduled.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 })
    );
    assert_eq!(
        scheduled.claim_reward(),
        Err(KartRacingError::RewardAlreadyClaimed)
    );

    let mut abandoned = RaceSession::open(
        3,
        TrackLayout::new(
            31,
            2,
            vec![11, 12],
            Some(kart_racing::RewardSchedule { coins: 10, xp: 25 }),
        )
        .unwrap(),
        0,
    )
    .unwrap();
    abandoned.begin(0).unwrap();
    abandoned.start(0).unwrap();
    abandoned.abandon();
    assert_eq!(
        abandoned.claim_reward(),
        Err(KartRacingError::NotFinished),
        "an abandoned run never grants"
    );
}

#[test]
fn abandoning_after_finish_keeps_the_result_and_reward() {
    let mut session = RaceSession::open(
        34,
        TrackLayout::new(31, 1, vec![11], Some(RewardSchedule { coins: 10, xp: 25 })).unwrap(),
        0,
    )
    .unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    session.on_checkpoint(10, 11);
    assert_eq!(session.phase(), Phase::Finished);

    session.abandon();
    assert_eq!(
        session.phase(),
        Phase::Finished,
        "a finished result must survive late teardown"
    );
    assert_eq!(
        session.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 })
    );
}

#[test]
fn track_definitions_and_snapshots_reject_ambiguous_state() {
    assert_eq!(
        TrackLayout::new(31, 2, vec![], None),
        Err(KartRacingError::NoCheckpoints)
    );
    assert_eq!(
        TrackLayout::new(31, 2, vec![11, 11], None),
        Err(KartRacingError::DuplicateCheckpoint(11))
    );
    assert_eq!(
        TrackLayout::new(31, 0, vec![11], None),
        Err(KartRacingError::ZeroLaps)
    );
    assert_eq!(
        TrackLayout::new(31, 2, vec![0], None),
        Err(KartRacingError::ZeroCheckpoint)
    );
    assert_eq!(
        TrackLayout::new(31, 2, vec![11], Some(RewardSchedule { coins: 0, xp: 0 })),
        Err(KartRacingError::EmptyRewardSchedule),
        "a priced track must price something"
    );

    let mut session = RaceSession::open(3, track(), 0).unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    session.on_checkpoint(10, 11);
    let snapshot: Value =
        serde_json::from_slice(&serde_json::to_vec(&session.snapshot()).unwrap()).unwrap();
    let corrupt = json!({
        "session_id": snapshot["session_id"],
        "speedway": snapshot["speedway"],
        "laps": snapshot["laps"],
        "checkpoints": snapshot["checkpoints"],
        "reward": snapshot["reward"],
        "phase": "racing",
        "started_ms": snapshot["started_ms"],
        "lap_started_ms": snapshot["lap_started_ms"],
        "next_index": 99,
        "laps_completed": 0,
        "best_lap_ms": null,
        "last_checkpoint": 11,
        "reward_claimed": false,
        "observed_racer_states": 0
    });
    assert_eq!(
        RaceSession::restore(serde_json::from_value(corrupt).unwrap()).err(),
        Some(KartRacingError::CorruptSnapshot)
    );
}

#[test]
fn snapshots_keep_the_reward_schedule_and_reject_incoherent_progress() {
    let mut session = RaceSession::open(
        4,
        TrackLayout::new(
            31,
            2,
            vec![11, 12],
            Some(kart_racing::RewardSchedule { coins: 10, xp: 25 }),
        )
        .unwrap(),
        0,
    )
    .unwrap();
    session.begin(0).unwrap();
    session.start(0).unwrap();
    session.on_checkpoint(10, 11);

    let snapshot = session.snapshot();
    assert_eq!(snapshot.reward, Some(RewardSchedule { coins: 10, xp: 25 }));
    let text = serde_json::to_vec(&snapshot).unwrap();
    let mut resumed = RaceSession::restore(serde_json::from_slice(&text).unwrap()).unwrap();
    resumed.on_checkpoint(20, 12);
    resumed.on_checkpoint(30, 11);
    resumed.on_checkpoint(40, 12);
    assert_eq!(resumed.phase(), Phase::Finished);
    assert_eq!(
        resumed.claim_reward(),
        Ok(RewardProposal::Granted { coins: 10, xp: 25 }),
        "the restored price schedule is still payable"
    );

    let mut missing_last = snapshot.clone();
    missing_last.last_checkpoint = None;
    assert_eq!(
        RaceSession::restore(missing_last).err(),
        Some(KartRacingError::CorruptSnapshot),
        "next_index>0 requires the gate that produced it"
    );

    let mut finished_early = snapshot.clone();
    finished_early.phase = Phase::Finished;
    assert_eq!(
        RaceSession::restore(finished_early).err(),
        Some(KartRacingError::CorruptSnapshot)
    );

    let mut best_without_laps = snapshot.clone();
    best_without_laps.best_lap_ms = Some(1);
    assert_eq!(
        RaceSession::restore(best_without_laps).err(),
        Some(KartRacingError::CorruptSnapshot)
    );

    let mut empty_schedule = snapshot.clone();
    empty_schedule.reward = Some(RewardSchedule { coins: 0, xp: 0 });
    assert_eq!(
        RaceSession::restore(empty_schedule).err(),
        Some(KartRacingError::CorruptSnapshot)
    );
}

#[test]
fn recovered_original_identifiers_are_stable() {
    assert_eq!(KART_RACE_GAME_TYPE, 1);
    assert_eq!(KART_RACE_LOBBY_NAME_ID, 3052);
    assert_eq!(KART_DRIVER_JOB_GUID, 48);
    assert_eq!(KART_DRIVER_JOB_NAME_ID, 2917);
    assert_eq!(
        SPEEDWAYS
            .iter()
            .map(|speedway| speedway.id)
            .collect::<Vec<_>>(),
        vec![31, 32, 33, 34, 39]
    );
    assert_eq!(
        SPEEDWAYS
            .iter()
            .map(|speedway| speedway.name_id)
            .collect::<Vec<_>>(),
        vec![18206, 3606, 382855, 382856, 401623]
    );
}

fn track() -> TrackLayout {
    TrackLayout::new(31, 2, vec![11, 12], None).unwrap()
}
