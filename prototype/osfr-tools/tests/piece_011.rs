//! Piece 011 (pet training activities) tests: recovered gesture rounds, validated
//! completion and the pet-trick wire family.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (shipped client data, hashed copies under the private lab):
//! * `Resources/PetTrickGesturePoints.txt` - `PET_TRICK_ID^SEQUENCE_ID^X^Y`, 23 trick
//!   ids, waypoints on a normalised plane.
//! * `Resources/PetTrickGestureSettings.txt` - `DRAW_DIST^0.1`, `SAMPLE_DRAW_RATE^0.0005`,
//!   `NUM_SEGMENTS^128`, four effect ids, `HIT_NODE_DIST_SQR^0.0025`,
//!   `TIMEOUT_SECONDS^10`, `FORCE_END_MILLISECONDS^20000` and six locale string ids.
//! * `PacketCodes.cs` (`PetBasePackets`, base opcode 53): 7 PetPerformTrickPacket,
//!   8 PetTrickGestureResult, 12 PetTrickSkillLevelPacket, 13 PetTrickAddPacket,
//!   14 PetTrickFavoriteSetPacket, 15 PetTrickNotifyPacket.
//! * Client profile pet record (community `ClientPcData.cs`): per trick a 32-bit key
//!   followed by eight 32-bit fields and one boolean flag.
//! * Ghidra pseudocode: `FUN_00b5ff50` writes one 32-bit payload for subtype 7;
//!   `FUN_00b5ffe0` writes one 32-bit payload for subtype 8; `FUN_00b5f670` rejects
//!   favourite slots outside `param_2 - 1U < 4` and writes pet, slot-1 and trick.
//!
//! RECONSTRUCTED (labelled below, not historical): the once-per-instance completion
//! ledger and the ordering rule that a node is only credited when every earlier node
//! in the same trick has already been hit.
//!
//! NOT CLAIMED: client animation, physics, reward values or the score-to-result-string
//! thresholds. The thresholds are absent from the recovered files and stay a gap.
// The module is not yet registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration step
// for the lead: add `pub mod piece_011;` to `src/restoration/mod.rs` (or register it
// from `src/lib.rs`) and drop this `#[path]` include.
#[path = "../src/restoration/piece_011.rs"]
#[allow(dead_code)]
mod piece_011;

use piece_011::{
    FAVORITE_SLOT_COUNT, GestureError, GestureNode, GestureSettings, PET_BASE_OPCODE,
    PROFILE_TRICK_FIELDS, RoundError, SUB_FAVORITE_SET, SUB_GESTURE_RESULT, SUB_NOTIFY,
    SUB_PERFORM_TRICK, SUB_SKILL_LEVEL, SUB_TRICK_ADD, SettingsError, TRICK_RECORD_LEN,
    TrickRecord, TrickRoundLedger, WireError, decode_favorite_set, decode_gesture_result,
    decode_perform_trick, decode_trick_record, encode_favorite_set, encode_gesture_result,
    encode_perform_trick, parse_gesture_points, parse_settings,
};

/// Recovered settings file, values copied from `PetTrickGestureSettings.txt`.
const RECOVERED_SETTINGS: &str = "#CONSTANT^VALUE\n\
DRAW_DIST^0.1\n\
SAMPLE_DRAW_RATE^0.0005\n\
NUM_SEGMENTS^128\n\
SAMPLE_DRAW_EFFECT^5511\n\
PLAYER_DRAW_EFFECT^5525\n\
SAMPLE_NODE_EFFECT^5526\n\
PLAYER_NODE_EFFECT^5527\n\
HIT_NODE_DIST_SQR^0.0025\n\
TIMEOUT_SECONDS^10\n\
RESULT_BAD_STRING^90155\n\
RESULT_OK_STRING^90157\n\
RESULT_GOOD_STRING^90161\n\
RESULT_GREAT_STRING^90163\n\
GET_READY_STRING^94617\n\
START_DRAW_STRING^94618\n\
FORCE_END_MILLISECONDS^20000\n";

/// Two gesture paths shaped like the shipped file (trick 1 and trick 12).
const SAMPLE_POINTS: &str = "#PET_TRICK_ID^SEQUENCE_ID^X^Y^\n\
1^0^0.200000^0.800000^\n\
1^1^0.600000^0.250000^\n\
12^0^0.700000^0.400000^\n\
12^1^0.700000^0.600000^\n\
12^2^0.300000^0.400000^\n";

fn settings() -> GestureSettings {
    parse_settings(RECOVERED_SETTINGS).expect("recovered settings parse")
}

fn node(x: f32, y: f32) -> GestureNode {
    GestureNode { x, y }
}

#[test]
fn settings_expose_the_recovered_constants() {
    let s = settings();
    assert_eq!(s.draw_dist, 0.1);
    assert_eq!(s.sample_draw_rate, 0.0005);
    assert_eq!(s.num_segments, 128);
    assert_eq!(s.sample_draw_effect, 5511);
    assert_eq!(s.player_draw_effect, 5525);
    assert_eq!(s.sample_node_effect, 5526);
    assert_eq!(s.player_node_effect, 5527);
    assert_eq!(s.hit_node_dist_sqr, 0.0025);
    assert_eq!(s.timeout_seconds, 10.0);
    assert_eq!(s.force_end_milliseconds, 20_000);
    assert_eq!(s.result_bad_string, 90155);
    assert_eq!(s.result_ok_string, 90157);
    assert_eq!(s.result_good_string, 90161);
    assert_eq!(s.result_great_string, 90163);
    assert_eq!(s.get_ready_string, 94617);
    assert_eq!(s.start_draw_string, 94618);
    // 0.0025 is the squared radius, so the drawn sample must land within 0.05 units.
    assert!((s.hit_radius() - 0.05).abs() < 1e-6);
    assert_eq!(s.timeout_milliseconds(), 10_000);
}

#[test]
fn settings_parser_rejects_missing_keys_and_garbage_numbers() {
    let missing = RECOVERED_SETTINGS.replace("HIT_NODE_DIST_SQR^0.0025\n", "");
    let err = parse_settings(&missing).expect_err("missing key must fail");
    assert!(matches!(
        err,
        SettingsError::MissingKey("HIT_NODE_DIST_SQR")
    ));

    let garbage = RECOVERED_SETTINGS.replace("NUM_SEGMENTS^128", "NUM_SEGMENTS^many");
    let err = parse_settings(&garbage).expect_err("garbage number must fail");
    assert!(matches!(err, SettingsError::BadNumber { .. }));

    let negative =
        RECOVERED_SETTINGS.replace("FORCE_END_MILLISECONDS^20000", "FORCE_END_MILLISECONDS^-1");
    let err = parse_settings(&negative).expect_err("negative millisecond budget must fail");
    assert!(matches!(err, SettingsError::BadNumber { .. }));
}

#[test]
fn gesture_points_group_by_trick_in_sequence_order() {
    let tricks = parse_gesture_points(SAMPLE_POINTS).expect("sample parses");
    assert_eq!(tricks.len(), 2);
    assert_eq!(tricks[0].trick_id, 1);
    assert_eq!(tricks[0].nodes, vec![node(0.2, 0.8), node(0.6, 0.25)]);
    assert_eq!(tricks[1].trick_id, 12);
    assert_eq!(
        tricks[1].nodes,
        vec![node(0.7, 0.4), node(0.7, 0.6), node(0.3, 0.4)]
    );
}

#[test]
fn gesture_points_reject_gaps_and_out_of_range_waypoints() {
    let gap = SAMPLE_POINTS.replace("1^1^0.600000^0.250000^\n", "1^2^0.600000^0.250000^\n");
    let err = parse_gesture_points(&gap).expect_err("sequence gap must fail");
    assert!(matches!(
        err,
        GestureError::NonContiguous { trick_id: 1, .. }
    ));

    let out_of_range =
        SAMPLE_POINTS.replace("1^0^0.200000^0.800000^\n", "1^0^1.200000^0.800000^\n");
    let err = parse_gesture_points(&out_of_range).expect_err("out-of-plane waypoint must fail");
    assert!(matches!(
        err,
        GestureError::WaypointOutOfRange { trick_id: 1, .. }
    ));

    // The shipped trick 103 revisits an earlier waypoint, so a repeat must parse.
    let duplicated = SAMPLE_POINTS.replace("1^1^0.600000^0.250000^\n", "1^1^0.200000^0.800000^\n");
    let tricks = parse_gesture_points(&duplicated).expect("repeated waypoint is real data");
    assert_eq!(tricks[0].nodes, vec![node(0.2, 0.8), node(0.2, 0.8)]);

    let zero_id = SAMPLE_POINTS.replace("12^0", "0^0");
    let err = parse_gesture_points(&zero_id).expect_err("non-positive trick id must fail");
    assert!(matches!(err, GestureError::BadTrickId { .. }));
}

#[test]
fn round_credits_nodes_only_in_order() {
    let s = settings();
    let tricks = parse_gesture_points(SAMPLE_POINTS).expect("sample parses");
    let ledger = TrickRoundLedger::new(&tricks);
    let mut t = ledger.begin(7, 1, 0, s).expect("round starts");
    // Crossing the second waypoint first earns nothing; waypoints are ordered.
    assert_eq!(t.record_sample(0.6, 0.25, 100, s), Ok(0));
    assert_eq!(t.record_sample(0.2, 0.8, 200, s), Ok(1));
    // Re-crossing the first waypoint does not double-count.
    assert_eq!(t.record_sample(0.21, 0.81, 300, s), Ok(1));
    assert_eq!(t.record_sample(0.6, 0.25, 400, s), Ok(2));
    let outcome = t.finish(500, s).expect("complete round accepted");
    assert_eq!(outcome.hit_nodes, 2);
    assert_eq!(outcome.total_nodes, 2);
    assert_eq!(outcome.elapsed_milliseconds, 500);
}

#[test]
fn round_rejects_incomplete_timeout_sample_overflow_and_late_finish() {
    let s = settings();

    let tricks = parse_gesture_points(SAMPLE_POINTS).expect("sample parses");
    let ledger = TrickRoundLedger::new(&tricks);
    let mut t = ledger.begin(1, 1, 0, s).expect("round starts");
    t.record_sample(0.2, 0.8, 100, s).unwrap();
    assert_eq!(
        t.finish(200, s),
        Err(RoundError::Incomplete {
            hit_nodes: 1,
            total_nodes: 2
        })
    );

    // Still inside the force-end window but past the recovered 10 s timeout.
    let mut t = ledger.begin(2, 1, 0, s).expect("round starts");
    t.record_sample(0.2, 0.8, 100, s).unwrap();
    t.record_sample(0.6, 0.25, 200, s).unwrap();
    assert_eq!(
        t.finish(10_500, s),
        Err(RoundError::TimedOut {
            elapsed_milliseconds: 10_500
        })
    );

    // The force-end budget is hard: samples after it are refused.
    let mut t = ledger.begin(3, 1, 0, s).expect("round starts");
    assert_eq!(
        t.record_sample(0.2, 0.8, 20_001, s),
        Err(RoundError::PastForceEnd {
            limit_milliseconds: 20_000
        })
    );

    // Sample count is bounded by the recovered NUM_SEGMENTS.
    let mut t = ledger.begin(4, 1, 0, s).expect("round starts");
    for i in 0..128 {
        assert_eq!(t.record_sample(0.5, 0.5, i, s), Ok(0));
    }
    assert_eq!(
        t.record_sample(0.5, 0.5, 500, s),
        Err(RoundError::TooManySamples { limit: 128 })
    );

    // Time never runs backwards inside one round.
    let mut t = ledger.begin(5, 1, 0, s).expect("round starts");
    t.record_sample(0.2, 0.8, 100, s).unwrap();
    assert_eq!(
        t.record_sample(0.2, 0.8, 99, s),
        Err(RoundError::TimeWentBackwards)
    );

    // An unknown trick cannot start a round.
    assert_eq!(
        ledger.begin(6, 999, 0, s),
        Err(RoundError::UnknownTrick(999))
    );
}

#[test]
fn completion_is_once_per_instance() {
    // Reconstructed policy: the ledger, not the client, decides completion.
    let s = settings();
    let tricks = parse_gesture_points(SAMPLE_POINTS).expect("sample parses");
    let mut ledger = TrickRoundLedger::new(&tricks);
    let mut first = ledger.begin(42, 1, 0, s).expect("round starts");
    first.record_sample(0.2, 0.8, 10, s).unwrap();
    first.record_sample(0.6, 0.25, 20, s).unwrap();
    let outcome = ledger
        .complete(&first, 30, s)
        .expect("first completion accepted");
    assert_eq!(outcome.total_nodes, 2);
    assert_eq!(
        ledger.complete(&first, 40, s),
        Err(RoundError::ReplayedInstance(42))
    );
    assert_eq!(
        ledger.begin(42, 1, 50, s),
        Err(RoundError::ReplayedInstance(42))
    );
}

#[test]
fn pet_wire_family_and_subtype_constants_match_the_recovered_table() {
    assert_eq!(PET_BASE_OPCODE, 53);
    assert_eq!(SUB_PERFORM_TRICK, 7);
    assert_eq!(SUB_GESTURE_RESULT, 8);
    assert_eq!(SUB_SKILL_LEVEL, 12);
    assert_eq!(SUB_TRICK_ADD, 13);
    assert_eq!(SUB_FAVORITE_SET, 14);
    assert_eq!(SUB_NOTIFY, 15);
}

#[test]
fn single_value_bodies_round_trip_and_reject_wrong_lengths() {
    let trick = encode_perform_trick(17);
    assert_eq!(trick, 17u32.to_le_bytes());
    assert_eq!(decode_perform_trick(&trick), Ok(17));
    assert_eq!(
        decode_perform_trick(&trick[..3]),
        Err(WireError::LengthMismatch {
            expected: 4,
            got: 3
        })
    );
    assert_eq!(
        decode_perform_trick(&[0, 0, 0, 0, 0]),
        Err(WireError::LengthMismatch {
            expected: 4,
            got: 5
        })
    );

    let result = encode_gesture_result(90163);
    assert_eq!(result, 90163u32.to_le_bytes());
    assert_eq!(decode_gesture_result(&result), Ok(90163));
    assert_eq!(
        decode_gesture_result(&[]),
        Err(WireError::LengthMismatch {
            expected: 4,
            got: 0
        })
    );
}

#[test]
fn favourite_slots_are_one_based_on_the_wire_as_zero_based() {
    // `FUN_00b5f670` accepts `param_2 - 1U < 4`, i.e. four player-facing slots.
    assert_eq!(FAVORITE_SLOT_COUNT, 4);
    let body = encode_favorite_set(77, 1, 14).expect("slot 1 encodes");
    assert_eq!(body.len(), 12);
    assert_eq!(&body[..4], &77u32.to_le_bytes());
    assert_eq!(&body[4..8], &0u32.to_le_bytes());
    assert_eq!(&body[8..], &14u32.to_le_bytes());
    assert_eq!(decode_favorite_set(&body), Ok((77, 0, 14)));
    let last = encode_favorite_set(77, 4, 17).expect("slot 4 encodes");
    assert_eq!(decode_favorite_set(&last), Ok((77, 3, 17)));
    assert_eq!(
        encode_favorite_set(77, 0, 14),
        Err(WireError::InvalidFavoriteSlot(0))
    );
    assert_eq!(
        encode_favorite_set(77, 5, 14),
        Err(WireError::InvalidFavoriteSlot(5))
    );
}

#[test]
fn profile_trick_records_use_the_community_layout() {
    assert_eq!(PROFILE_TRICK_FIELDS, 8);
    assert_eq!(TRICK_RECORD_LEN, 37);
    // Trick 17 from the recovered profile sample: Speak, skill 1, flag set. The
    // locale id sits in field index 2; that mapping is inferred, not original.
    let mut body = Vec::with_capacity(TRICK_RECORD_LEN);
    body.extend_from_slice(&17u32.to_le_bytes());
    for value in [17u32, 7532, 22714, 1, 0, 0, 0, 1] {
        body.extend_from_slice(&value.to_le_bytes());
    }
    body.push(1);
    let record = decode_trick_record(&body).expect("record decodes");
    assert_eq!(record.trick_id, 17);
    assert_eq!(record.fields[2], 22714);
    assert!(record.flag);
    assert_eq!(record.name_id_candidate(), Some(22714));
    assert_eq!(record.skill_level_candidate(), 1);
    assert_eq!(TrickRecord::encode(&record), body);

    let mut short = body.clone();
    short.pop();
    assert_eq!(
        decode_trick_record(&short),
        Err(WireError::LengthMismatch {
            expected: TRICK_RECORD_LEN,
            got: TRICK_RECORD_LEN - 1
        })
    );
}
