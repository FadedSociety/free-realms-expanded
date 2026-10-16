//! Work item 016: minigame loss and retry (`op39`) tests.
//!
//! The module is included by path until the lead registers it in `src/lib.rs`.
//!
//! ORIGINAL vectors embedded here are the nine preserved decoded minigame frames under
//! `lab/reports/original-minigame-packets/` (seven server->client `39/23` knockouts, two
//! client->server `39/7` leave requests). Each byte string below was re-hashed against those
//! files: six knockouts share `a6aa79938ed60a9255d0a51df9e20a02d1d2b065f97fecdbd0971f5487bdbbf9`
//! (also the `sha256` of the soccer `39/23` row in `soccer-application-index.json`),
//! `demoderby-31557-flow0-frame29641` is `08e233c11bd705f63ab943439d5d3559707358926b1244445f2a9223fde45721`,
//! `demoderby-43469-flow1-frame41374` is `2f37481af7abca619b5754b35dc2ed02efea21c473f125e3a0098d862ea73309`
//! and `racing-24992-flow1-frame42933` is `2a3d4fd4ca2b7e8a3b1ff662c88eee151c5166ebbf7d1bfc624be59a7a95b75c`.
//! A capture frame includes the 7-byte tunnel envelope; `encode_*` helpers return the inner
//! message only, so those tests compare the tail.
#[path = "../src/restoration/piece_016.rs"]
mod piece_016;

use piece_016::{
    BASE_HEADER_LEN, CURRENT_STATE, FrameError, Ledger, MINIGAME_FRAME_HEADER_LEN,
    MINIGAME_INNER_OPCODE, MinigameFrame, Outcome, Phase, Reject, SUB_GAME_OVER, SUB_GAME_START,
    SUB_KNOCK_OUT, SUB_LEAVE_REQUEST, SUB_RESTART_REQUEST, SUB_START_REQUEST, SUB_STATE_REMOVE,
    decode_base_header, decode_frame, encode_game_over, encode_game_start, encode_knock_out,
    encode_state_remove,
};
use sha2::{Digest, Sha256};

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

/// Six identical `demoderby`/`racing`/`soccer` knockouts: all ids `-1`, counter `0/0`.
const KNOCKOUT_ALL_CURRENT: &str = "05000117000000270017ffffffffffffffffffffffff0000000000000000";
/// `demoderby-31557-flow0-frame29641`: all ids `-1`, counter `1/0`.
const KNOCKOUT_COUNTER_ONE: &str = "05000117000000270017ffffffffffffffffffffffff0100000000000000";
/// `demoderby-43469-flow1-frame41374`: client leave request, state `0x140041`.
const LEAVE_DERBY: &str = "0500010f00000027000741001400ffffffffffffffff";
/// `racing-24992-flow1-frame42933`: client leave request, state `0x0e003e`.
const LEAVE_RACING: &str = "0500010f0000002700073e000e00ffffffffffffffff";

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[test]
fn captured_knockout_frames_decode_and_re_encode_byte_exact() {
    for (hex, current, digest) in [
        (
            KNOCKOUT_ALL_CURRENT,
            0,
            "a6aa79938ed60a9255d0a51df9e20a02d1d2b065f97fecdbd0971f5487bdbbf9",
        ),
        (
            KNOCKOUT_COUNTER_ONE,
            1,
            "08e233c11bd705f63ab943439d5d3559707358926b1244445f2a9223fde45721",
        ),
    ] {
        let frame = bytes(hex);
        assert_eq!(sha256_hex(&frame), digest);
        let decoded = decode_frame(&frame).expect("captured knockout frame must decode");
        assert_eq!(decoded.subtype, SUB_KNOCK_OUT);
        let body = decode_base_header(decoded.inner).expect("base header");
        assert_eq!(body.state_id, CURRENT_STATE);
        assert_eq!(body.group_id, CURRENT_STATE);
        assert_eq!(body.game_id, CURRENT_STATE);
        let knockout = decoded.knock_out().expect("knockout body");
        assert_eq!((knockout.current, knockout.max), (current, 0));
        assert_eq!(decode_frame(&frame).unwrap().knock_out().unwrap(), knockout);
        assert_eq!(
            encode_knock_out(&knockout),
            frame[MINIGAME_FRAME_HEADER_LEN - 3..].to_vec()
        );
    }
}

#[test]
fn captured_leave_frames_are_bare_base_headers() {
    for (hex, state_id, digest) in [
        (
            LEAVE_DERBY,
            0x0014_0041,
            "2f37481af7abca619b5754b35dc2ed02efea21c473f125e3a0098d862ea73309",
        ),
        (
            LEAVE_RACING,
            0x000e_003e,
            "2a3d4fd4ca2b7e8a3b1ff662c88eee151c5166ebbf7d1bfc624be59a7a95b75c",
        ),
    ] {
        let frame = bytes(hex);
        assert_eq!(sha256_hex(&frame), digest);
        let decoded = decode_frame(&frame).expect("captured leave frame must decode");
        assert_eq!(decoded.subtype, SUB_LEAVE_REQUEST);
        assert_eq!(decoded.inner.len(), BASE_HEADER_LEN);
        let body = decode_base_header(decoded.inner).expect("base header");
        assert_eq!(body.state_id, state_id);
        assert_eq!(body.group_id, CURRENT_STATE);
        assert_eq!(body.game_id, CURRENT_STATE);
    }
}

#[test]
fn game_over_frame_carries_the_won_byte_and_three_ints() {
    let lost = encode_game_over(false, 41, -1, 18);
    assert_eq!(lost.len(), BASE_HEADER_LEN + 13);
    assert_eq!(&lost[..2], &MINIGAME_INNER_OPCODE.to_le_bytes());
    assert_eq!(lost[2], SUB_GAME_OVER);
    assert_eq!(decode_base_header(&lost).unwrap().state_id, 41);
    assert_eq!(lost[BASE_HEADER_LEN], 0, "loss sets Won = false");
    assert_eq!(
        &lost[BASE_HEADER_LEN + 1..],
        &[0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0]
    );

    let won = encode_game_over(true, 41, -1, 18);
    assert_eq!(won[BASE_HEADER_LEN], 1, "win sets Won = true");
    let frame = [bytes("0500011c000000"), won.clone()].concat();
    let decoded = decode_frame(&frame).unwrap();
    assert_eq!(decoded.game_over().unwrap().won, true);
    assert_eq!(decoded.game_over().unwrap().state_id, 41);
}

#[test]
fn state_remove_and_game_start_are_bare_headers() {
    for frame in [encode_state_remove(9, -1, 18), encode_game_start(9, -1, 18)] {
        assert_eq!(frame.len(), BASE_HEADER_LEN);
    }
    assert_eq!(encode_state_remove(9, -1, 18)[2], SUB_STATE_REMOVE);
    assert_eq!(encode_game_start(9, -1, 18)[2], SUB_GAME_START);
    assert_eq!(encode_game_start(9, -1, 18)[3..7], 9_i32.to_le_bytes());
}

#[test]
fn start_and_restart_subtypes_are_the_recovered_requests() {
    assert_eq!(SUB_START_REQUEST, 5, "native RequestStartGame sends sub 5");
    assert_eq!(
        SUB_RESTART_REQUEST, 11,
        "native RequestRestartGame sends sub 11"
    );
    assert_eq!(SUB_LEAVE_REQUEST, 7, "native LeaveMiniGame sends sub 7");
}

#[test]
fn frame_decoder_rejects_malformed_input() {
    let frame = bytes(KNOCKOUT_ALL_CURRENT);
    assert!(matches!(
        decode_frame(&frame[..MINIGAME_FRAME_HEADER_LEN - 1]),
        Err(FrameError::TooShort)
    ));
    let mut wrong_outer = frame.clone();
    wrong_outer[0] = 9;
    wrong_outer[1] = 0;
    assert!(matches!(
        decode_frame(&wrong_outer),
        Err(FrameError::UnsupportedOuterOpcode(9))
    ));
    let mut wrong_inner = frame.clone();
    wrong_inner[7] = 40;
    assert!(matches!(
        decode_frame(&wrong_inner),
        Err(FrameError::UnsupportedInnerOpcode(40))
    ));
    let mut bad_length = frame.clone();
    bad_length[3] = 0x18;
    assert!(matches!(
        decode_frame(&bad_length),
        Err(FrameError::LengthMismatch { .. })
    ));
    let mut truncated_knockout = frame.clone();
    truncated_knockout.pop();
    assert!(matches!(
        decode_frame(&truncated_knockout),
        Err(FrameError::LengthMismatch { .. })
    ));
    assert!(matches!(
        decode_base_header(&frame[..BASE_HEADER_LEN - 1]),
        Err(FrameError::MalformedBody(_))
    ));
    let mut wrong_flag = frame.clone();
    wrong_flag[2] = 2;
    assert!(matches!(
        decode_frame(&wrong_flag),
        Err(FrameError::UnsupportedTunnelFlag(2))
    ));
    // A `39/18` body is longer than a knockout body and must not decode as one.
    let game_over = encode_game_over(false, 1, -1, 18);
    assert!(matches!(
        MinigameFrame {
            outer_opcode: 5,
            tunnel_flag: 1,
            subtype: SUB_GAME_OVER,
            inner: &game_over,
        }
        .knock_out(),
        Err(FrameError::MalformedBody(_))
    ));
}

#[test]
fn knockout_remaining_is_max_minus_current() {
    let knockout = decode_frame(&bytes(KNOCKOUT_ALL_CURRENT))
        .unwrap()
        .knock_out()
        .unwrap();
    assert_eq!(knockout.remaining(), 0);
    let synthetic = piece_016::KnockOut {
        state_id: 4,
        group_id: -1,
        game_id: 2,
        current: 3,
        max: 5,
    };
    assert_eq!(synthetic.remaining(), 2);
}

#[test]
fn start_mints_a_fresh_instance_id_and_acks_it() {
    let mut ledger = Ledger::new(1_000);
    assert!(ledger.current().is_none());
    let first = ledger.start(-1, 18).expect("first start");
    assert_eq!(first.state_id, 1_000);
    assert_eq!(first.replies, vec![encode_game_start(1_000, -1, 18)]);
    assert_eq!(ledger.current().unwrap().phase, Phase::Playing);

    assert!(matches!(ledger.start(-1, 18), Err(Reject::AlreadyPlaying)));
    ledger
        .finish(first.state_id, Outcome::Lost)
        .expect("loss settles");
    assert!(matches!(ledger.start(-1, 18), Err(Reject::AlreadyPlaying)));
}

#[test]
fn loss_is_terminal_and_proposes_no_reward() {
    let mut ledger = Ledger::new(7);
    let start = ledger.start(-1, 31).unwrap();
    let finish = ledger.finish(start.state_id, Outcome::Lost).unwrap();
    assert!(!finish.won);
    assert!(
        finish.reward.is_none(),
        "a loss must never propose a reward"
    );
    assert_eq!(
        finish.replies,
        vec![
            encode_game_over(false, start.state_id, -1, 31),
            encode_state_remove(start.state_id, -1, 31),
        ]
    );
    assert_eq!(ledger.current().unwrap().phase, Phase::Lost);
    assert!(matches!(
        ledger.finish(start.state_id, Outcome::Lost),
        Err(Reject::AlreadySettled)
    ));
}

#[test]
fn a_win_proposes_exactly_one_reward() {
    let mut ledger = Ledger::new(7);
    let start = ledger.start(-1, 31).unwrap();
    let finish = ledger.finish(start.state_id, Outcome::Won).unwrap();
    assert!(finish.won);
    let reward = finish.reward.expect("a win proposes a reward");
    assert_eq!(reward.state_id, start.state_id);
    assert_eq!(reward.game_id, 31);
    assert_eq!(finish.replies[0][BASE_HEADER_LEN], 1);
    assert!(matches!(
        ledger.finish(start.state_id, Outcome::Won),
        Err(Reject::AlreadySettled)
    ));
}

#[test]
fn stale_finish_after_retry_is_rejected_without_reward() {
    let mut ledger = Ledger::new(7);
    let first = ledger.start(-1, 31).unwrap();
    ledger.finish(first.state_id, Outcome::Lost).unwrap();

    let retry = ledger.retry(first.state_id).expect("retry after loss");
    assert!(retry.state_id > first.state_id, "retry mints a fresh state");
    assert_eq!(
        retry.replies,
        vec![encode_game_start(retry.state_id, -1, 31)]
    );

    match ledger.finish(first.state_id, Outcome::Won) {
        Err(Reject::StaleState { offered, current }) => {
            assert_eq!(offered, first.state_id);
            assert_eq!(current, retry.state_id);
        }
        other => panic!("stale finish must be rejected, got {other:?}"),
    }
    let win = ledger.finish(retry.state_id, Outcome::Won).unwrap();
    assert_eq!(win.reward.unwrap().state_id, retry.state_id);
}

#[test]
fn retry_requires_a_terminal_instance() {
    let mut ledger = Ledger::new(7);
    assert!(matches!(ledger.retry(7), Err(Reject::NoCurrentState)));
    let start = ledger.start(-1, 31).unwrap();
    assert!(matches!(
        ledger.retry(start.state_id),
        Err(Reject::AlreadyPlaying)
    ));
    assert!(matches!(ledger.retry(999), Err(Reject::StaleState { .. })));
}

#[test]
fn finish_without_a_started_instance_is_rejected() {
    let mut ledger = Ledger::new(7);
    assert!(matches!(
        ledger.finish(7, Outcome::Won),
        Err(Reject::NoCurrentState)
    ));
    let start = ledger.start(-1, 31).unwrap();
    assert!(matches!(
        ledger.finish(start.state_id + 1, Outcome::Won),
        Err(Reject::StaleState { .. })
    ));
}

#[test]
fn quitting_abandons_the_instance_without_reward() {
    let mut ledger = Ledger::new(7);
    let start = ledger.start(-1, 31).unwrap();
    let finish = ledger.finish(start.state_id, Outcome::Quit).unwrap();
    assert!(!finish.won);
    assert!(finish.reward.is_none());
    assert_eq!(
        finish.replies,
        vec![encode_state_remove(start.state_id, -1, 31)]
    );
    assert_eq!(ledger.current().unwrap().phase, Phase::Quit);
}

#[test]
fn state_ids_are_never_reused_across_many_retries() {
    let mut ledger = Ledger::new(-5);
    let mut ids = Vec::new();
    let start = ledger.start(-1, 18).unwrap();
    ids.push(start.state_id);
    let mut current = start.state_id;
    for round in 0..12 {
        let outcome = if round % 3 == 0 {
            Outcome::Won
        } else {
            Outcome::Lost
        };
        ledger.finish(current, outcome).unwrap();
        let retry = ledger.retry(current).unwrap();
        assert!(retry.state_id > current);
        ids.push(retry.state_id);
        current = retry.state_id;
    }
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "state ids must never be reused");
}
