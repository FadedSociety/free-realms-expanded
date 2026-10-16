//! Piece 094 (reliable transport stress) tests: loss, reordering, duplicates,
//! sequence rollover and session replacement across several clients.
//!
//! Evidence boundary for these assertions (details in
//! `notes/Research/Pieces/094 Reliable transport stress.md`):
//!
//! RECOVERED / EXISTING PRODUCTION (read-only here, exercised through the harness):
//! * `src/soe.rs` framing, CRC and bounded ordered receive; `src/soe_send.rs`
//!   retransmission and bounded send queue. Both are already covered by
//!   `tests/soe_transport.rs` and `tests/soe_send.rs` for a single link.
//! * `src/bin/osfr-gateway.rs` derives a per-generation key
//!   `CRC_SEED ^ generation` on session replacement and refuses a negotiated
//!   `crc_length == 0` because a zero trailer cannot isolate stale frames.
//!
//! RECONSTRUCTED (stress policy only, no invented wire facts): the deterministic
//! loss/duplicate/reorder schedule, the bounded latency model and the retained
//! wire-byte accounting. No sockets, timers, gameplay or economy are involved.
//!
//! NOT CLAIMED: this is not native acceptance and not a multi-client gateway.
//! The gateway is single-peer by design; several clients here means several
//! independent link pairs stressed in one process.
// The crate's own transport modules, declared at this test root so `crate::soe`,
// `crate::soe_send` and `crate::session` resolve exactly as they will inside
// `src/lib.rs` once the lead registers the module below.
#[path = "../src/session.rs"]
#[allow(dead_code)]
mod session;
#[path = "../src/soe.rs"]
#[allow(dead_code)]
mod soe;
#[path = "../src/soe_send.rs"]
#[allow(dead_code)]
mod soe_send;

// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact.
#[path = "../src/restoration/piece_094.rs"]
#[allow(dead_code)]
mod piece_094;

use piece_094::{SendError, SessionPolicy, StressConfig, StressError, TransportStress};

/// Deterministic message body for `client`/`index`. Lengths straddle the
/// serializer's 100-byte compression threshold so fragmentation is exercised.
fn message(client: usize, index: usize) -> Vec<u8> {
    let filler = [0usize, 4, 12, 40, 90, 140, 220][index % 7];
    let mut bytes = vec![client as u8, (index >> 8) as u8, index as u8];
    let mut state = 0x9e37_79b9u32 ^ (index as u32).wrapping_mul(2_654_435_761);
    for _ in 0..filler {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        bytes.push(state as u8);
    }
    bytes
}

fn messages(client: usize, range: std::ops::Range<usize>) -> Vec<Vec<u8>> {
    range.map(|index| message(client, index)).collect()
}

#[test]
fn several_clients_deliver_exactly_once_through_loss_reordering_and_duplicates() {
    let config = StressConfig::new(4);
    let mut stress = TransportStress::new(config.clone()).unwrap();
    for client in 0..config.clients {
        for index in 0..300 {
            stress.queue(client, &message(client, index)).unwrap();
        }
    }
    let ticks = stress.run_until_idle(200_000).unwrap();
    assert!(ticks > 1);

    let mut totals = [0usize; 4];
    for client in 0..config.clients {
        assert_eq!(
            stress.delivered(client),
            messages(client, 0..300).as_slice(),
            "client {client}"
        );
        let stats = stress.stats(client);
        assert_eq!(
            stats.corrupt_rejected, 0,
            "client {client} rejected live frames"
        );
        assert_eq!(
            stats.receive_errors, 0,
            "client {client} hit a receive error"
        );
        assert_eq!(
            stats.stale_accepted, 0,
            "client {client} accepted a stale frame"
        );
        assert_eq!(stress.backlog(client), 0);
        assert!(stress.retained_peak(client) <= config.max_queued_bytes);
        totals[0] += stats.loss;
        totals[1] += stats.duplicate;
        totals[2] += stats.reordered;
        totals[3] += stats.retransmits;
    }
    assert!(totals[0] > 0, "no frame was dropped");
    assert!(totals[1] > 0, "no frame was duplicated");
    assert!(totals[2] > 0, "no frame was reordered");
    assert!(totals[3] > 0, "no frame was retransmitted");
    assert!(stress.is_idle());
    println!("{}", stress.summary_line("several_clients"));
}

#[test]
fn session_replacement_rejects_stale_datagrams_and_restarts_exactly() {
    let config = StressConfig {
        clients: 2,
        latency_ticks: 3,
        loss_percent: 0,
        duplicate_percent: 0,
        reorder_percent: 0,
        ack_loss_percent: 0,
        ack_duplicate_percent: 0,
        ..StressConfig::new(2)
    };
    let mut stress = TransportStress::new(config).unwrap();
    for index in 0..40 {
        stress.queue(0, &message(0, index)).unwrap();
    }
    // A second client keeps delivering across the replacement.
    for index in 0..25 {
        stress.queue(1, &message(1, index)).unwrap();
    }
    // Fill the send window, then stop with the pipeline drained of backlog and
    // frames still in flight: those frames belong to the replaced session.
    stress
        .run_until(
            |stress| {
                stress.backlog(0) == 0
                    && stress.in_flight_for(0) > 0
                    && !stress.delivered(0).is_empty()
            },
            10_000,
        )
        .unwrap();
    let boundary = stress.delivered(0).len();
    assert!(boundary > 0, "no message landed before the replacement");
    assert!(stress.in_flight_for(0) > 0, "nothing stayed in flight");

    stress.replace_session(0).unwrap();
    assert_eq!(stress.stats(0).stale_rejected, 0);
    for index in 0..10 {
        stress.queue(0, &message(0, 100 + index)).unwrap();
    }
    stress.run_until_idle(50_000).unwrap();

    let delivered = stress.delivered(0).to_vec();
    let stats = stress.stats(0);
    assert_eq!(
        stats.stale_accepted, 0,
        "a replaced session's frame was accepted"
    );
    assert!(stats.stale_rejected > 0, "no stale frame was rejected");
    assert!(stats.discarded > 0, "the replaced sender retained nothing");
    assert_eq!(stats.corrupt_rejected, 0);
    assert_eq!(delivered[..boundary], messages(0, 0..boundary)[..]);
    assert_eq!(delivered[boundary..], messages(0, 100..110)[..]);
    // The concurrent client is untouched by the replacement.
    assert_eq!(stress.delivered(1), messages(1, 0..25).as_slice());
    assert_eq!(stress.stats(1).stale_accepted, 0);
    assert_eq!(stress.stats(1).corrupt_rejected, 0);
    assert_eq!(stress.stats(1).receive_errors, 0);
    assert!(stress.is_idle());
    println!("{}", stress.summary_line("session_replacement"));
}

#[test]
fn reused_key_or_zero_crc_admits_stale_datagrams() {
    // Negative control: the harness must be able to observe stale acceptance,
    // otherwise the replacement test above passes vacuously. Both policies below
    // are exactly what `osfr-gateway.rs` refuses to run.
    let cases = [
        ("zero_crc", SessionPolicy::RotateSeed, 0u8),
        ("reused_key", SessionPolicy::ReuseSeed, 3u8),
    ];
    for (name, policy, crc_len) in cases {
        let config = StressConfig {
            clients: 1,
            crc_len,
            session_policy: policy,
            latency_ticks: 3,
            loss_percent: 0,
            duplicate_percent: 0,
            reorder_percent: 0,
            ack_loss_percent: 0,
            ack_duplicate_percent: 0,
            ..StressConfig::new(1)
        };
        let mut stress = TransportStress::new(config).unwrap();
        for index in 0..40 {
            stress.queue(0, &message(0, index)).unwrap();
        }
        stress
            .run_until(
                |stress| {
                    stress.backlog(0) == 0
                        && stress.in_flight_for(0) > 0
                        && !stress.delivered(0).is_empty()
                },
                10_000,
            )
            .unwrap();
        let boundary = stress.delivered(0).len();
        assert!(
            stress.in_flight_for(0) > 0,
            "{name}: nothing stayed in flight"
        );
        stress.replace_session(0).unwrap();
        for index in 0..10 {
            stress.queue(0, &message(0, 100 + index)).unwrap();
        }
        stress.run_until_idle(50_000).unwrap();

        let delivered = stress.delivered(0).to_vec();
        let stats = stress.stats(0);
        assert!(
            stats.stale_accepted > 0,
            "{name}: stale frames were not admitted"
        );
        assert!(
            delivered[boundary..] != messages(0, 100..110)[..],
            "{name}: exact delivery survived an unguarded replacement"
        );
        println!("{}", stress.summary_line(&format!("unsafe_{name}")));
    }
}

#[test]
fn sequence_rollover_with_a_small_queue_delivers_exactly_once_and_stays_bounded() {
    const COUNT: usize = 65_600;
    let config = StressConfig {
        clients: 1,
        window: 24,
        max_queued_bytes: 4096,
        loss_percent: 0,
        duplicate_percent: 0,
        reorder_percent: 0,
        ack_loss_percent: 0,
        ..StressConfig::new(1)
    };
    let mut stress = TransportStress::new(config.clone()).unwrap();
    for index in 0..COUNT {
        stress.queue(0, &message(0, index)).unwrap();
    }
    stress.run_until_idle(2_000_000).unwrap();

    assert_eq!(stress.delivered(0), messages(0, 0..COUNT).as_slice());
    let stats = stress.stats(0);
    assert_eq!(stats.corrupt_rejected, 0);
    assert_eq!(stats.stale_accepted, 0);
    assert!(
        stats.queue_full > 0,
        "a 4 KiB queue accepted the whole stream"
    );
    assert!(stress.retained_peak(0) <= config.max_queued_bytes);
    assert!(stress.retained_peak(0) > config.max_queued_bytes / 2);
    assert_eq!(stress.backlog(0), 0);
    println!("{}", stress.summary_line("rollover_bounded_queue"));
}

#[test]
fn a_stalled_client_fails_at_its_retry_limit_without_disturbing_others() {
    let config = StressConfig {
        clients: 2,
        retry_ms: 10,
        max_retries: 2,
        tick_ms: 10,
        loss_percent: 0,
        duplicate_percent: 0,
        reorder_percent: 0,
        ack_loss_percent: 0,
        ack_duplicate_percent: 0,
        ..StressConfig::new(2)
    };
    let mut stress = TransportStress::new(config.clone()).unwrap();
    for client in 0..2 {
        for index in 0..60 {
            stress.queue(client, &message(client, index)).unwrap();
        }
    }
    stress.set_partitioned(0, true).unwrap();

    let mut error = None;
    for _ in 0..1_000 {
        match stress.tick() {
            Ok(()) => {}
            Err(reported) => {
                error = Some(reported);
                break;
            }
        }
    }
    assert!(matches!(
        error,
        Some(StressError::Sender {
            client: 0,
            error: SendError::RetryLimit { .. }
        })
    ));
    let stalled_prefix = stress.delivered(0).len();
    stress.retire_client(0).unwrap();
    stress.set_partitioned(0, false).unwrap();
    stress.run_until_idle(50_000).unwrap();

    assert_eq!(stress.delivered(0).len(), stalled_prefix);
    assert_eq!(stress.delivered(1), messages(1, 0..60).as_slice());
    let stats = stress.stats(1);
    assert_eq!(stats.corrupt_rejected, 0);
    assert_eq!(stats.receive_errors, 0);
    assert!(stats.loss > 0 || stress.stats(0).loss > 0);
    assert!(stress.retained_peak(1) <= config.max_queued_bytes);
    assert!(stress.is_idle());
    println!("{}", stress.summary_line("stalled_client_isolation"));
}
