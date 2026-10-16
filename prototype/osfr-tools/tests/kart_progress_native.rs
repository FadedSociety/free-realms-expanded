//! Kart native progress codec vectors — inner 33 subtype 14 (73-byte progress block) plus the tiny
//! subtype 1 (game state) and subtype 12 (destroy racer) payloads.
//!
//! ORIGINAL evidence used here (all outside this test):
//! * `notes/Research/Kart Race Finish Protocol.md` — `FUN_00ba74d0` dispatches the vehicle-race
//!   family (inner opcode 33); `FUN_00b9fc80` reads subtype 14 in this exact order:
//!   `u64 guid @0x00, i32 slot @0x08, f32 @0x0c, f32 @0x10, i32 laps @0x14, u8 local @0x18,
//!   i32 lap limit @0x19, f32 @0x1d, f32 best lap @0x21, f32 total @0x25, u64 x4 @0x29/0x31/0x39/0x41`
//!   — 73 bytes, every read bounds-checked. The wrapper (`FUN_00ba1230`-style reader) returns
//!   success only when the buffer was consumed exactly.
//! * the same note's native NaN detail, re-checked in the pinned export: the parser reads `0x0c`
//!   and `0x10` as `f32` and raises the reader's NaN flag (`packet+0x11`) for each; truncation
//!   raises `packet+0x10`. The wrapper's return value depends on truncation and exact consumption,
//!   not on the NaN flag, so NaN is **flagged, not refused**.
//! * `lab/reports/kart-finish-independent-worker-review-20261009T155247Z/capture-recompute.json` and
//!   `…T155257Z/review.json` — 9,358 subtype-14 frames (all 73-byte payloads, tunnel 84 bytes with
//!   `payload = raw[11:]`, `inner_length = payload_len + 4`), laps `{0: 8869, 1: 489}`, limit `3` in
//!   every frame, local flag `0` in every frame, slots `0..7`, 8 guids, `frames_at_or_above_limit 0`,
//!   state sequence `3,4,5,6,7,8,9` over 7 subtype-1 frames, no subtype-12 frame.
//! * a direct scan of the same private frames for this codec: FLT_MAX (`0x7f7fffff`) occurs at payload
//!   `0x10` in 6 frames and `0x21` in 5 frames and **never** as NaN, so FLT_MAX is observed valid.
//!
//! `Finished` is client-derived (`limit < laps.wrapping_add(1)` in the results row), while the
//! results-screen trigger uses `limit <= laps`. Neither is on the wire. These reporting helpers
//! authorizes nothing and no test here treats a decoded frame as a finish grant.
//!
//! This file exercises the registered production codec; framing is reused from
//! the already-verified `kart_racing::decode_racing_frame`.

use osfr_tools::kart_racing;

use osfr_tools::kart_progress_native::{
    DESTROY_RACER_PAYLOAD_LEN, GAME_STATE_PAYLOAD_LEN, NonFinitePolicy, ProgressCodecError,
    ProgressFloatField, ProgressPayload, decode_destroy_racer, decode_game_state, decode_progress,
    encode_destroy_racer, encode_game_state, encode_progress, results_row_finished,
    results_screen_trigger,
};

/// The recovered `progress_finish` vector from the finish note (synthetic, not captured):
/// guid `0x0000004000000001`, slot 0, laps 3 of limit 3, best 58.25 s, total 179.75 s, lap 61.5 s.
const FINISH_HEX: &str = concat!(
    "0100000040000000", // guid
    "00000000",         // slot
    "00000000",         // 0x0c
    "00000000",         // 0x10
    "03000000",         // laps
    "00",               // local flag
    "03000000",         // lap limit
    "00007642",         // 0x1d = 61.5
    "00006942",         // best lap = 58.25
    "00c03343",         // total = 179.75
    "0000000000000000",
    "0000000000000000",
    "0000000000000000",
    "0000000000000000",
);

fn finish_payload() -> ProgressPayload {
    ProgressPayload {
        racer_guid: 0x0000_0040_0000_0001,
        slot: 0,
        unknown_0c_bits: 0,
        unknown_10_bits: 0,
        laps_completed: 3,
        local_racer: 0,
        lap_limit: 3,
        unknown_1d_bits: 61.5f32.to_bits(),
        best_lap_bits: 58.25f32.to_bits(),
        total_time_bits: 179.75f32.to_bits(),
        tail_0x29: 0,
        tail_0x31: 0,
        tail_0x39: 0,
        tail_0x41: 0,
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[test]
fn finish_vector_matches_the_recovered_layout() {
    let payload = finish_payload();
    let bytes = encode_progress(&payload);
    assert_eq!(hex(&bytes), FINISH_HEX);
    assert_eq!(bytes.len(), 73);

    let decoded = decode_progress(&bytes, NonFinitePolicy::Preserve).expect("decodes");
    assert_eq!(decoded, payload);
    assert_eq!(decoded.best_lap_time(), 58.25);
    assert_eq!(decoded.total_time(), 179.75);
    assert_eq!(decoded.unknown_1d(), 61.5);
    assert!(results_row_finished(
        decoded.laps_completed,
        decoded.lap_limit
    ));
}

#[test]
fn mid_race_vector_roundtrips_and_is_not_finished() {
    let payload = ProgressPayload {
        racer_guid: 0x0000_0040_0000_0003,
        slot: 2,
        unknown_0c_bits: 1.0f32.to_bits(),
        unknown_10_bits: (-3.5f32).to_bits(),
        laps_completed: 1,
        local_racer: 1,
        lap_limit: 3,
        unknown_1d_bits: 12.5f32.to_bits(),
        best_lap_bits: 40.25f32.to_bits(),
        total_time_bits: 41.75f32.to_bits(),
        tail_0x29: 0x1122_3344_5566_7788,
        tail_0x31: 0x99aa_bbcc_ddee_ff00,
        tail_0x39: 0,
        tail_0x41: 1,
    };
    let decoded = decode_progress(&encode_progress(&payload), NonFinitePolicy::RefuseNaN)
        .expect("finite frame");
    assert_eq!(decoded, payload);
    assert!(!results_row_finished(
        decoded.laps_completed,
        decoded.lap_limit
    ));
}

#[test]
fn every_unknown_field_roundtrips_bit_for_bit() {
    let payload = ProgressPayload {
        racer_guid: u64::MAX,
        slot: i32::MIN,
        unknown_0c_bits: 0x7f7f_ffff, // FLT_MAX, observed valid in the capture
        unknown_10_bits: 0x8000_0000, // negative zero
        laps_completed: i32::MIN,
        local_racer: 0xff,
        lap_limit: i32::MAX,
        unknown_1d_bits: 0x0000_0001, // denormal
        best_lap_bits: 0x7f7f_ffff,
        total_time_bits: 0xdead_beef,
        tail_0x29: u64::MAX,
        tail_0x31: 0x0102_0304_0506_0708,
        tail_0x39: 0,
        tail_0x41: 0xffff_ffff_0000_0001,
    };
    let bytes = encode_progress(&payload);
    assert_eq!(bytes.len(), 73);
    assert_eq!(
        decode_progress(&bytes, NonFinitePolicy::RefuseNaN).expect("no NaN, so RefuseNaN passes"),
        payload
    );
    assert_eq!(
        decode_progress(&bytes, NonFinitePolicy::Preserve).expect("decodes"),
        payload
    );
}

#[test]
fn refuse_nan_flags_only_the_two_natively_checked_fields() {
    let mut payload = finish_payload();
    payload.unknown_0c_bits = 0x7fc0_0001;
    assert_eq!(
        decode_progress(&encode_progress(&payload), NonFinitePolicy::RefuseNaN),
        Err(ProgressCodecError::NonFinite {
            field: ProgressFloatField::Unknown0c
        })
    );
    assert_eq!(
        decode_progress(&encode_progress(&payload), NonFinitePolicy::Preserve)
            .expect("native policy preserves")
            .unknown_0c_bits,
        0x7fc0_0001
    );

    let mut payload = finish_payload();
    payload.unknown_10_bits = f32::NAN.to_bits();
    assert_eq!(
        decode_progress(&encode_progress(&payload), NonFinitePolicy::RefuseNaN),
        Err(ProgressCodecError::NonFinite {
            field: ProgressFloatField::Unknown10
        })
    );

    // The other three floats are not NaN-checked natively, so even RefuseNaN keeps them verbatim.
    let mut payload = finish_payload();
    payload.unknown_1d_bits = f32::NAN.to_bits();
    payload.best_lap_bits = f32::NAN.to_bits();
    payload.total_time_bits = f32::NAN.to_bits();
    let decoded = decode_progress(&encode_progress(&payload), NonFinitePolicy::RefuseNaN)
        .expect("only 0x0c and 0x10 are flagged");
    assert!(decoded.unknown_1d().is_nan() && decoded.best_lap_time().is_nan());
}

#[test]
fn progress_payload_must_be_exactly_73_bytes() {
    let bytes = encode_progress(&finish_payload());
    for length in [0usize, 72, 74] {
        let mut candidate = bytes.clone();
        candidate.resize(length, 0);
        assert_eq!(
            decode_progress(&candidate, NonFinitePolicy::Preserve),
            Err(ProgressCodecError::WrongLength {
                expected: 73,
                actual: length
            }),
            "payload length {length}"
        );
    }
}

#[test]
fn game_state_payload_is_exactly_four_bytes_and_preserves_any_state() {
    assert_eq!(GAME_STATE_PAYLOAD_LEN, 4);
    for state in [3i32, 4, 5, 6, 7, 8, 9, -1, i32::MAX] {
        let decoded = decode_game_state(&encode_game_state(state)).expect("decodes");
        assert_eq!(decoded, state);
    }
    for length in [0usize, 3, 5] {
        assert_eq!(
            decode_game_state(&vec![0u8; length]),
            Err(ProgressCodecError::WrongLength {
                expected: 4,
                actual: length
            })
        );
    }
}

#[test]
fn destroy_racer_payload_is_exactly_eight_bytes_and_carries_only_a_guid() {
    assert_eq!(DESTROY_RACER_PAYLOAD_LEN, 8);
    let guid = 0x0000_0040_0000_0007u64;
    assert_eq!(
        decode_destroy_racer(&encode_destroy_racer(guid)).expect("decodes"),
        guid
    );
    for length in [0usize, 7, 9] {
        assert_eq!(
            decode_destroy_racer(&vec![0u8; length]),
            Err(ProgressCodecError::WrongLength {
                expected: 8,
                actual: length
            })
        );
    }
}

#[test]
fn results_row_predicate_is_limit_less_than_laps_plus_one() {
    // VehicleRaceResultsDataSource row +0x20: `processor+0x1a0 < racer+0x15ec + 1`.
    assert!(!results_row_finished(0, 3));
    assert!(!results_row_finished(2, 3));
    assert!(results_row_finished(3, 3));
    assert!(results_row_finished(4, 3));
}

#[test]
fn signed_maximum_boundary_separates_the_two_recovered_predicates() {
    // `laps + 1` wraps to i32::MIN, so the row predicate is false at the maximum while the
    // screen trigger (a plain `<=`) is true. The two predicates must not be conflated.
    assert!(!results_row_finished(i32::MAX, 1));
    assert!(results_screen_trigger(i32::MAX, 1));
    assert!(!results_row_finished(i32::MAX, 0));
    assert!(results_screen_trigger(i32::MAX, 0));
    assert!(!results_row_finished(i32::MAX, i32::MIN));
    assert!(results_screen_trigger(i32::MAX, i32::MIN));
    // Below the wrap the recovered expressions agree for every signed lap count.
    for laps in [i32::MIN, -1, 0, 2, 3, 4, i32::MAX - 1] {
        for limit in [i32::MIN, -1, 0, 3, i32::MAX] {
            assert_eq!(
                results_row_finished(laps, limit),
                results_screen_trigger(laps, limit),
                "laps {laps}, limit {limit}"
            );
        }
    }
}

/// Round-trips every captured 33/14 (and 33/1) frame through the codec and the existing framing.
///
/// Run with:
/// `OSFR_KART_PROGRESS_CAPTURE_DIR=<...original-minigame-packets>
///  cargo test --test kart_progress_native -- --ignored --exact private_capture_roundtrips_all_progress_frames`
#[test]
#[ignore = "requires the private lab frame directory named by OSFR_KART_PROGRESS_CAPTURE_DIR"]
fn private_capture_roundtrips_all_progress_frames() {
    use std::collections::{BTreeMap, BTreeSet};

    let dir = std::env::var_os("OSFR_KART_PROGRESS_CAPTURE_DIR")
        .expect("OSFR_KART_PROGRESS_CAPTURE_DIR must point at the preserved frame directory");
    let dir = std::path::PathBuf::from(dir);

    let mut progress_files = Vec::new();
    let mut state_files = Vec::new();
    let mut destroy_files = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("readable capture directory") {
        let path = entry.expect("directory entry").path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // Restrict to the vehicle-race family: other minigames also archive `-sub14.bin` frames.
        if name.contains("-op33-sub14.bin") {
            progress_files.push(path);
        } else if name.contains("-op33-sub1.bin") {
            state_files.push(path);
        } else if name.contains("-op33-sub12.bin") {
            destroy_files.push(path);
        }
    }
    progress_files.sort();
    state_files.sort();

    assert_eq!(progress_files.len(), 9358, "captured subtype-14 frames");
    assert_eq!(state_files.len(), 7, "captured subtype-1 frames");
    assert_eq!(
        destroy_files.len(),
        0,
        "the capture has no subtype-12 frame"
    );

    let mut laps: BTreeMap<i32, usize> = BTreeMap::new();
    let mut limits: BTreeMap<i32, usize> = BTreeMap::new();
    let mut guids: BTreeSet<u64> = BTreeSet::new();
    let mut slots: BTreeSet<i32> = BTreeSet::new();
    let mut finished_rows = 0usize;
    let mut flt_max_10 = 0usize;
    let mut flt_max_21 = 0usize;
    let mut local_flag_values: BTreeSet<u8> = BTreeSet::new();

    for path in &progress_files {
        let raw = std::fs::read(path).expect("readable frame");
        let frame = kart_racing::decode_racing_frame(&raw).expect("existing framing");
        assert_eq!(frame.outer_opcode, 5);
        assert_eq!(frame.flag, 1);
        assert_eq!(frame.subtype, 14);
        assert_eq!(frame.payload.len(), 73);
        assert_eq!(
            &raw[11..],
            frame.payload,
            "payload starts after the 11-byte header"
        );

        let payload = decode_progress(frame.payload, NonFinitePolicy::Preserve).expect("decodes");
        assert_eq!(
            encode_progress(&payload)[..],
            frame.payload[..],
            "byte-exact payload"
        );

        // Reassemble the tunnel frame from its recovered parts and require the exact file bytes.
        let mut rebuilt = Vec::with_capacity(raw.len());
        rebuilt.extend(frame.outer_opcode.to_le_bytes());
        rebuilt.push(frame.flag);
        rebuilt.extend(((frame.payload.len() + 4) as u32).to_le_bytes());
        rebuilt.extend(33u16.to_le_bytes());
        rebuilt.extend(frame.subtype.to_le_bytes());
        rebuilt.extend(encode_progress(&payload));
        assert_eq!(rebuilt, raw, "byte-exact frame {}", path.display());

        *laps.entry(payload.laps_completed).or_default() += 1;
        *limits.entry(payload.lap_limit).or_default() += 1;
        guids.insert(payload.racer_guid);
        slots.insert(payload.slot);
        local_flag_values.insert(payload.local_racer);
        if results_row_finished(payload.laps_completed, payload.lap_limit) {
            finished_rows += 1;
        }
        assert_eq!(
            results_row_finished(payload.laps_completed, payload.lap_limit),
            results_screen_trigger(payload.laps_completed, payload.lap_limit),
            "both recovered predicates agree on the captured values"
        );
        if payload.unknown_10_bits == 0x7f7f_ffff {
            flt_max_10 += 1;
        }
        if payload.best_lap_bits == 0x7f7f_ffff {
            flt_max_21 += 1;
        }
    }

    assert_eq!(laps, BTreeMap::from([(0, 8869), (1, 489)]));
    assert_eq!(limits, BTreeMap::from([(3, 9358)]));
    assert_eq!(guids.len(), 8);
    assert_eq!(slots, (0..=7).collect());
    assert_eq!(local_flag_values, BTreeSet::from([0]));
    assert_eq!(finished_rows, 0, "no captured frame reaches the lap limit");
    assert_eq!(flt_max_10, 6, "FLT_MAX is observed valid at 0x10");
    assert_eq!(flt_max_21, 5, "FLT_MAX is observed valid at 0x21");

    let mut states = Vec::new();
    for path in &state_files {
        let raw = std::fs::read(path).expect("readable state frame");
        let frame = kart_racing::decode_racing_frame(&raw).expect("existing framing");
        assert_eq!(frame.subtype, 1);
        assert_eq!(frame.payload.len(), 4);
        let state = decode_game_state(frame.payload).expect("decodes");
        assert_eq!(encode_game_state(state), frame.payload);
        states.push(state);
    }
    states.sort_unstable();
    assert_eq!(
        states,
        vec![3, 4, 5, 6, 7, 8, 9],
        "recovered state sequence"
    );
}
