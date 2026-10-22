#[path = "../src/movement.rs"]
mod movement;
#[allow(dead_code)]
#[path = "../src/soe.rs"]
mod soe;
#[allow(dead_code)]
#[path = "../src/zone_transition.rs"]
mod zone_transition;

use movement::{
    Decision, ExpectedDiscontinuity, MovementState, MovementUpdate, Reason, SpeedModifiers,
    validate,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use soe::{Codec, ReliableReceiver};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_MESSAGE: usize = 1_048_576;
const MAX_BUFFERED: usize = 4_194_304;

fn update(position: [f32; 3]) -> MovementUpdate {
    update_with_state(position, 2)
}

fn update_with_state(position: [f32; 3], state: u8) -> MovementUpdate {
    MovementUpdate {
        position,
        direction: [1.0, 0.0, 0.0],
        state,
        unknown: 0,
    }
}

fn seed() -> MovementState {
    MovementState::new(0, [0.0; 3], [0.0, 0.0, 1.0])
}

// (R) Test profile only; no universal player speed constant was recovered.
fn test_policy() -> SpeedModifiers {
    SpeedModifiers {
        base_speed_per_second: 12.0,
        mount_multiplier: 1.0,
        buff_multiplier: 1.0,
    }
}

#[test]
fn ordinary_motion_clamps_soft_overage_and_rejects_gross_speed_hacks() {
    let previous = seed();
    let policy = test_policy();
    let accepted = validate(
        &previous,
        update([1.0, 0.0, 0.0]),
        0,
        Duration::from_millis(100),
        policy,
        None,
    );
    assert_eq!(accepted.decision, Decision::Accept);

    let clamped = validate(
        &previous,
        update([60.0, 0.0, 0.0]),
        0,
        Duration::from_millis(100),
        policy,
        None,
    );
    assert_eq!(clamped.decision, Decision::Clamp);
    assert_eq!(clamped.reason, Reason::SoftHorizontalOverspeed);
    assert!((clamped.state.position[0] - 48.0).abs() < 0.001);

    let rejected = validate(
        &previous,
        update([100.0, 0.0, 0.0]),
        0,
        Duration::from_millis(100),
        policy,
        None,
    );
    assert_eq!(rejected.decision, Decision::RejectAndSnapBack);
    assert_eq!(rejected.reason, Reason::GrossHorizontalOverspeed);
    assert_eq!(rejected.state, previous);
}

#[test]
fn unauthorized_teleport_hack_is_rejected() {
    let result = validate(
        &seed(),
        update([5_000.0, 0.0, -5_000.0]),
        0,
        Duration::from_millis(50),
        test_policy(),
        None,
    );
    assert_eq!(result.decision, Decision::RejectAndSnapBack);
    assert_eq!(result.reason, Reason::GrossHorizontalOverspeed);
}

#[test]
fn server_issued_zone_and_developer_teleport_tokens_authorize_only_their_target() {
    let previous = seed();
    let zone_change = validate(
        &previous,
        update([500.0, 30.0, -20.0]),
        4,
        Duration::from_millis(50),
        test_policy(),
        Some(ExpectedDiscontinuity::zone_change(4, [500.0, 30.0, -20.0])),
    );
    assert_eq!(zone_change.decision, Decision::Accept);
    assert_eq!(zone_change.reason, Reason::ExpectedZoneChange);
    assert_eq!(zone_change.state.zone_id, 4);
    assert!(zone_change.expected_discontinuity.is_none());

    let teleport = validate(
        &previous,
        update([80.0, 4.0, 12.0]),
        0,
        Duration::from_millis(50),
        test_policy(),
        Some(ExpectedDiscontinuity::developer_teleport(
            0,
            [80.0, 4.0, 12.0],
        )),
    );
    assert_eq!(teleport.decision, Decision::Accept);
    assert_eq!(teleport.reason, Reason::ExpectedDeveloperTeleport);

    let target_state = MovementState::new(0, [80.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let wrong_target = validate(
        &target_state,
        update([900.0, 0.0, 0.0]),
        0,
        Duration::from_millis(50),
        test_policy(),
        Some(ExpectedDiscontinuity::developer_teleport(
            0,
            [80.0, 0.0, 0.0],
        )),
    );
    assert_eq!(wrong_target.decision, Decision::Ignore);
    assert_eq!(wrong_target.reason, Reason::ExpectedDeveloperTeleport);
    assert_eq!(wrong_target.state, target_state);
    assert!(wrong_target.expected_discontinuity.is_some());
}

#[test]
fn stale_teleport_reports_are_ignored_then_matching_target_resumes_validation() {
    let target = MovementState::new(0, [80.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let ignored = validate(
        &target,
        update([405.27, 0.0, 0.0]),
        0,
        Duration::from_millis(15),
        test_policy(),
        Some(ExpectedDiscontinuity::developer_teleport(
            0,
            [80.0, 0.0, 0.0],
        )),
    );
    assert_eq!(ignored.decision, Decision::Ignore);
    assert_eq!(ignored.state, target);
    assert_eq!(ignored.reason, Reason::ExpectedDeveloperTeleport);

    let resumed = validate(
        &ignored.state,
        update([80.0, 0.0, 0.0]),
        0,
        Duration::from_millis(50),
        test_policy(),
        ignored.expected_discontinuity,
    );
    assert_eq!(resumed.decision, Decision::Accept);
    assert_eq!(resumed.state.position, target.position);
    assert!(resumed.expected_discontinuity.is_none());
}

#[test]
fn post_teleport_suppression_expires_and_does_not_authorize_a_hack() {
    let target = MovementState::new(0, [80.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let pending = validate(
        &target,
        update([900.0, 0.0, 0.0]),
        0,
        Duration::from_millis(4_900),
        test_policy(),
        Some(ExpectedDiscontinuity::developer_teleport(
            0,
            [80.0, 0.0, 0.0],
        )),
    );
    assert_eq!(pending.decision, Decision::Ignore);
    let expired = validate(
        &target,
        update([900.0, 0.0, 0.0]),
        0,
        Duration::from_millis(100),
        test_policy(),
        pending.expected_discontinuity,
    );
    assert_eq!(expired.decision, Decision::RejectAndSnapBack);
    assert_eq!(expired.reason, Reason::GrossHorizontalOverspeed);
    assert_eq!(expired.state, target);
    assert!(expired.expected_discontinuity.is_none());
}

#[test]
fn native_slow_speed_flag_reduces_horizontal_tolerance() {
    let result = validate(
        &seed(),
        update_with_state([6.1, 0.0, 0.0], 1 << 4),
        0,
        Duration::from_millis(50),
        test_policy(),
        None,
    );
    assert_eq!(result.decision, Decision::Clamp);
    assert!((result.state.position[0] - 6.0).abs() < 0.001);
}

#[test]
fn invalid_positions_and_unexpected_zone_changes_snap_back() {
    let previous = seed();
    for (position, expected) in [
        ([f32::NAN, 0.0, 0.0], Reason::NonFinitePosition),
        ([0.0, f32::INFINITY, 0.0], Reason::NonFinitePosition),
        ([1_000_001.0, 0.0, 0.0], Reason::OutOfBoundsPosition),
    ] {
        let result = validate(
            &previous,
            update(position),
            0,
            Duration::from_secs(1),
            test_policy(),
            None,
        );
        assert_eq!(result.decision, Decision::RejectAndSnapBack);
        assert_eq!(result.reason, expected);
        assert_eq!(result.state, previous);
    }

    let mut bad_direction = update([0.0, 0.0, 0.0]);
    bad_direction.direction[2] = f32::NEG_INFINITY;
    let result = validate(
        &previous,
        bad_direction,
        0,
        Duration::from_secs(1),
        test_policy(),
        None,
    );
    assert_eq!(result.decision, Decision::RejectAndSnapBack);
    assert_eq!(result.reason, Reason::NonFiniteDirection);

    let result = validate(
        &previous,
        update([0.0, 0.0, 0.0]),
        9,
        Duration::from_secs(30),
        test_policy(),
        None,
    );
    assert_eq!(result.decision, Decision::RejectAndSnapBack);
    assert_eq!(result.reason, Reason::ZoneChangeWithoutMatchingToken);
}

#[test]
fn mount_and_buff_policy_does_not_stack_and_vertical_motion_has_wide_tolerance() {
    let previous = seed();
    let result = validate(
        &previous,
        update([100.0, 0.0, 0.0]),
        0,
        Duration::from_millis(100),
        SpeedModifiers {
            base_speed_per_second: 12.0,
            mount_multiplier: 1.5,
            buff_multiplier: 1.4,
        },
        None,
    );
    assert_eq!(result.decision, Decision::Clamp);
    assert!((result.state.position[0] - 72.0).abs() < 0.001);

    let fall_or_jump = validate(
        &previous,
        update([0.0, 40.0, 0.0]),
        0,
        Duration::from_millis(250),
        test_policy(),
        None,
    );
    assert_eq!(fall_or_jump.decision, Decision::Accept);

    let vertical_hack = validate(
        &previous,
        update([0.0, 10_000.0, 0.0]),
        0,
        Duration::from_millis(100),
        test_policy(),
        None,
    );
    assert_eq!(vertical_hack.decision, Decision::RejectAndSnapBack);
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)]
    frames: Vec<FrameMeta>,
}

#[derive(Deserialize)]
struct FrameMeta {
    #[serde(default)]
    time: f64,
    #[serde(default)]
    direction: String,
    #[serde(default)]
    service: String,
    #[serde(default)]
    opcode: u16,
    #[serde(default)]
    path: String,
    #[serde(default)]
    bytes: usize,
    #[serde(default)]
    sha256: String,
}

#[derive(Default)]
struct ReplayStats {
    position_updates: usize,
    state_counts: BTreeMap<u8, usize>,
    unknown_counts: BTreeMap<u8, usize>,
    fragmented_position_updates: usize,
    bundled_position_updates: usize,
    out_of_order_sequences: usize,
    out_of_order_position_sequences: usize,
    rejected: usize,
    clamped: usize,
    ignored: usize,
    expected_discontinuities: usize,
    teleport_tokens: usize,
    accepted_teleports: usize,
    ignored_teleports: usize,
    teleport_stale_distance: Option<f64>,
    teleport_stale_delay: Option<f64>,
    zone_tokens: usize,
}

#[test]
fn replay_all_recorded_native_baselines_without_rejecting_legitimate_positions() {
    let runs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/runs");
    let mut directories = fs::read_dir(&runs)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("client-baseline-"))
                && (path.join("manifest.json").is_file()
                    || path.join("frames/live-index.jsonl").is_file())
        })
        .collect::<Vec<_>>();
    directories.sort();
    assert!(
        !directories.is_empty(),
        "no recorded baseline manifests found"
    );
    let manifest_runs = directories
        .iter()
        .filter(|path| path.join("manifest.json").is_file())
        .count();
    let live_index_runs = directories.len() - manifest_runs;
    let capture_runs = directories.len();
    assert!(manifest_runs >= 92 && live_index_runs >= 4);

    let required_run = directories
        .iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "client-baseline-20261014T011719Z")
        })
        .expect("the 2026-10-14 developer-teleport trace is missing")
        .clone();

    let mut total = ReplayStats::default();
    let mut replayed_runs = 0;
    let mut required_stats = ReplayStats::default();
    for directory in directories {
        let manifest = read_run_manifest(&directory);
        let stats = replay_run(&directory, manifest);
        if stats.position_updates != 0 {
            replayed_runs += 1;
        }
        total.position_updates += stats.position_updates;
        for (state, count) in &stats.state_counts {
            *total.state_counts.entry(*state).or_default() += count;
        }
        for (flag, count) in &stats.unknown_counts {
            *total.unknown_counts.entry(*flag).or_default() += count;
        }
        total.fragmented_position_updates += stats.fragmented_position_updates;
        total.bundled_position_updates += stats.bundled_position_updates;
        total.out_of_order_sequences += stats.out_of_order_sequences;
        total.out_of_order_position_sequences += stats.out_of_order_position_sequences;
        total.rejected += stats.rejected;
        total.clamped += stats.clamped;
        total.ignored += stats.ignored;
        total.expected_discontinuities += stats.expected_discontinuities;
        total.teleport_tokens += stats.teleport_tokens;
        total.accepted_teleports += stats.accepted_teleports;
        total.ignored_teleports += stats.ignored_teleports;
        total.zone_tokens += stats.zone_tokens;
        eprintln!(
            "{}: updates={}, rejects={}, clamps={}, ignores={}, teleports={}, zones={}",
            directory.file_name().unwrap().to_string_lossy(),
            stats.position_updates,
            stats.rejected,
            stats.clamped,
            stats.ignored,
            stats.teleport_tokens,
            stats.zone_tokens,
        );
        if directory == required_run {
            required_stats = stats;
        }
    }

    assert!(
        replayed_runs > 0,
        "no baseline position updates were decoded"
    );
    assert!(
        replayed_runs >= 50 && total.position_updates >= 10_000,
        "capture decoder regressed to sparse movement coverage: runs={replayed_runs}, updates={}",
        total.position_updates
    );
    assert!(
        required_stats.position_updates > 0,
        "required trace had no positions"
    );
    assert!(
        required_stats.teleport_tokens > 0,
        "required trace had no server teleport token"
    );
    assert!(
        required_stats.ignored_teleports > 0,
        "required trace did not ignore its stale teleport report: tokens={}, updates={}, expected={}, ignored={}, rejected={}, clamped={}",
        required_stats.teleport_tokens,
        required_stats.position_updates,
        required_stats.expected_discontinuities,
        required_stats.ignored_teleports,
        required_stats.rejected,
        required_stats.clamped
    );
    assert_eq!(
        required_stats.rejected, 0,
        "legitimate position rejected in required trace"
    );
    assert_eq!(total.rejected, 0, "legitimate baseline position rejected");
    assert_eq!(total.clamped, 0, "legitimate baseline position clamped");
    assert_eq!(
        total.out_of_order_position_sequences, 0,
        "captured movement update sequence order changed"
    );
    // Aggregate counts are intentionally limited to movement outcomes. The
    // replay never decodes or prints chat/application text or player identity.
    let states = &total.state_counts;
    let flags = &total.unknown_counts;
    eprintln!(
        "movement replay: capture_runs={capture_runs}, manifest_runs={manifest_runs}, live_index_runs={live_index_runs}, position_runs={replayed_runs}, updates={}, rejects={}, clamps={}, ignores={}, fragmented_updates={}, bundled_updates={}, out_of_order_sequences={}, out_of_order_position_sequences={}, authorized_discontinuities={}, teleport_tokens={}, accepted_teleports={}, ignored_teleports={}, zone_tokens={}, states={states:?}, second_bytes={flags:?}",
        total.position_updates,
        total.rejected,
        total.clamped,
        total.ignored,
        total.fragmented_position_updates,
        total.bundled_position_updates,
        total.out_of_order_sequences,
        total.out_of_order_position_sequences,
        total.expected_discontinuities,
        total.teleport_tokens,
        total.accepted_teleports,
        total.ignored_teleports,
        total.zone_tokens,
    );
}

#[test]
fn real_developer_teleport_ignores_the_15ms_stale_position() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/runs/client-baseline-20261014T011719Z");
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(directory.join("manifest.json")).expect("read developer-teleport trace"),
    )
    .expect("parse developer-teleport trace");
    let stats = replay_run(&directory, manifest);

    assert_eq!(stats.position_updates, 3);
    assert_eq!(stats.teleport_tokens, 1);
    assert_eq!(stats.ignored_teleports, 1);
    assert!((stats.teleport_stale_delay.unwrap() - 0.015).abs() < 0.002);
    assert!((stats.teleport_stale_distance.unwrap() - 325.270).abs() < 0.1);
    assert_eq!(stats.rejected, 0);
    assert_eq!(stats.clamped, 0);
}

fn replay_run(directory: &Path, manifest: Manifest) -> ReplayStats {
    let mut stats = ReplayStats::default();
    let mut codec: Option<Codec> = None;
    let mut receivers: [Option<ReliableReceiver>; 2] = [None, None];
    let mut state: Option<MovementState> = None;
    let mut current_zone = 0u32;
    let mut last_position_time: Option<f64> = None;
    let mut expected: Option<ExpectedDiscontinuity> = None;
    let mut client_sequences = std::collections::HashSet::new();
    let mut last_client_sequence: Option<u16> = None;
    let mut last_position_sequence: Option<u16> = None;
    let policy = test_policy();
    let mut previous_capture_time = None;

    for meta in manifest.frames {
        if meta.service != "gateway" || !matches!(meta.opcode, 1 | 2 | 9 | 13 | 21) {
            continue;
        }
        assert!(meta.time.is_finite(), "non-finite capture timestamp");
        if let Some(previous) = previous_capture_time {
            assert!(
                meta.time >= previous,
                "capture timestamps moved backwards in {}: {previous} then {}",
                directory.display(),
                meta.time
            );
        }
        previous_capture_time = Some(meta.time);
        let path = directory.join(&meta.path);
        let bytes = fs::read(&path).unwrap_or_else(|_| {
            panic!(
                "missing captured gateway datagram in {}",
                directory.display()
            )
        });
        assert_eq!(bytes.len(), meta.bytes, "captured datagram length mismatch");
        let digest = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(digest, meta.sha256, "captured datagram hash mismatch");

        if meta.opcode == 2 && meta.direction == "server-to-client" {
            if bytes.len() < 13 || bytes[..2] != [0, 2] || bytes[12] != 0 {
                continue;
            }
            let seed = u32::from_be_bytes(bytes[6..10].try_into().unwrap());
            let crc = bytes[10];
            let compression = bytes[11] == 1;
            codec = Codec::new(seed, crc, compression, MAX_MESSAGE).ok();
            receivers = [None, None];
            client_sequences.clear();
            last_client_sequence = None;
            last_position_sequence = None;
            continue;
        }
        let Some(active_codec) = codec.as_ref() else {
            continue;
        };
        if !matches!(meta.opcode, 9 | 13) {
            continue;
        }
        let direction_index = if meta.direction == "client-to-server" {
            0
        } else if meta.direction == "server-to-client" {
            1
        } else {
            continue;
        };
        let packet = match active_codec.decode(&bytes) {
            Ok(packet) if matches!(packet.opcode, 9 | 13) => packet,
            _ => continue,
        };
        let direct_message = if direction_index == 0 && packet.opcode == 9 {
            if packet.payload.len() < 2 {
                continue;
            }
            let sequence = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
            if let Some(last) = last_client_sequence {
                let forward = sequence.wrapping_sub(last);
                if (0 < forward) && (forward < 0x8000) && sequence < last {
                    client_sequences.clear();
                }
            }
            if !client_sequences.insert(sequence) {
                continue;
            }
            if let Some(last) = last_client_sequence {
                if sequence.wrapping_sub(last) >= 0x8000 {
                    stats.out_of_order_sequences += 1;
                    eprintln!(
                        "movement sequence out of order run={} time={:.3} previous={} current={}",
                        directory.file_name().unwrap().to_string_lossy(),
                        meta.time,
                        last,
                        sequence,
                    );
                } else {
                    last_client_sequence = Some(sequence);
                }
            } else {
                last_client_sequence = Some(sequence);
            }
            Some(packet.payload[2..].to_vec())
        } else {
            None
        };
        if receivers[direction_index].is_none() {
            if packet.payload.len() < 2 {
                continue;
            }
            let initial = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
            receivers[direction_index] =
                ReliableReceiver::new(initial, 1024, MAX_MESSAGE, MAX_BUFFERED).ok();
        }
        let Some(receiver) = receivers[direction_index].as_mut() else {
            continue;
        };
        let outcome = receiver.receive(&packet).ok();
        let messages = if let Some(message) = direct_message {
            vec![message]
        } else if let Some(outcome) = outcome {
            outcome.messages
        } else {
            continue;
        };
        for message in messages {
            let children = match application_messages(&message) {
                Ok(children) => children,
                Err(_) => continue,
            };
            for child in children {
                let Some((outer, inner)) = tunnel(child) else {
                    continue;
                };
                if direction_index == 1 {
                    if let Some(zone) = parse_zone_details(inner) {
                        current_zone = zone;
                    }
                    if inner.get(..2) == Some(&31u16.to_le_bytes()) {
                        if let Ok(transition) = zone_transition::decode(child) {
                            if transition.id >= 0 {
                                current_zone = transition.id as u32;
                                expected = Some(ExpectedDiscontinuity::zone_change(
                                    current_zone,
                                    [
                                        transition.position[0],
                                        transition.position[1],
                                        transition.position[2],
                                    ],
                                ));
                                stats.zone_tokens += 1;
                                set_server_pose(
                                    &mut state,
                                    &mut last_position_time,
                                    current_zone,
                                    [
                                        transition.position[0],
                                        transition.position[1],
                                        transition.position[2],
                                    ],
                                    meta.time,
                                );
                            }
                        }
                    }
                    if let Some((position, teleport)) = parse_location(inner) {
                        if teleport {
                            expected = Some(ExpectedDiscontinuity::developer_teleport(
                                current_zone,
                                position,
                            ));
                            stats.teleport_tokens += 1;
                            set_server_pose(
                                &mut state,
                                &mut last_position_time,
                                current_zone,
                                position,
                                meta.time,
                            );
                        } else if state.is_none() {
                            set_server_pose(
                                &mut state,
                                &mut last_position_time,
                                current_zone,
                                position,
                                meta.time,
                            );
                        }
                    }
                } else if outer == 5 {
                    if let Some(update) = parse_position(inner) {
                        if let (0, Some(sequence)) = (
                            direction_index,
                            packet
                                .payload
                                .get(..2)
                                .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]])),
                        ) {
                            if let Some(last) = last_position_sequence {
                                let delta = sequence.wrapping_sub(last);
                                if sequence != last && delta >= 0x8000 {
                                    stats.out_of_order_position_sequences += 1;
                                    eprintln!(
                                        "movement position sequence out of order run={} time={:.6} previous={} current={}",
                                        directory.file_name().unwrap().to_string_lossy(),
                                        meta.time,
                                        last,
                                        sequence,
                                    );
                                } else if sequence != last {
                                    last_position_sequence = Some(sequence);
                                }
                            } else {
                                last_position_sequence = Some(sequence);
                            }
                        }
                        stats.position_updates += 1;
                        *stats.state_counts.entry(update.state).or_default() += 1;
                        *stats.unknown_counts.entry(update.unknown).or_default() += 1;
                        if direction_index == 0 && packet.opcode == 13 {
                            stats.fragmented_position_updates += 1;
                        }
                        if direction_index == 0 && message.get(..2) == Some(&[0, 25]) {
                            stats.bundled_position_updates += 1;
                        }
                        match state.as_ref() {
                            None => {
                                // The first captured client update establishes
                                // the trace baseline; no earlier accepted pose
                                // exists in this datagram capture.
                                state = Some(MovementState {
                                    zone_id: current_zone,
                                    position: update.position,
                                    direction: update.direction,
                                    state: update.state,
                                    unknown: update.unknown,
                                });
                                last_position_time = Some(meta.time);
                            }
                            Some(previous) => {
                                let dt = last_position_time
                                    .map(|last| (meta.time - last).max(0.0))
                                    .unwrap_or(0.0);
                                last_position_time = Some(meta.time);
                                let result = validate(
                                    previous,
                                    update,
                                    current_zone,
                                    Duration::from_secs_f64(dt),
                                    policy,
                                    expected.take(),
                                );
                                if matches!(
                                    result.decision,
                                    Decision::Clamp | Decision::RejectAndSnapBack
                                ) {
                                    let dx = f64::from(update.position[0] - previous.position[0]);
                                    let dy = f64::from(update.position[1] - previous.position[1]);
                                    let dz = f64::from(update.position[2] - previous.position[2]);
                                    eprintln!(
                                        "movement outlier run={} time={:.3} state={} h={:.3} v={:.3} dt={:.3} decision={:?} reason={:?}",
                                        directory.file_name().unwrap().to_string_lossy(),
                                        meta.time,
                                        update.state,
                                        dx.hypot(dz),
                                        dy.abs(),
                                        dt,
                                        result.decision,
                                        result.reason,
                                    );
                                }
                                if matches!(
                                    result.reason,
                                    Reason::ExpectedZoneChange | Reason::ExpectedDeveloperTeleport
                                ) {
                                    stats.expected_discontinuities += 1;
                                }
                                let is_teleport =
                                    result.reason == Reason::ExpectedDeveloperTeleport;
                                match result.decision {
                                    Decision::Accept => {
                                        if is_teleport {
                                            stats.accepted_teleports += 1;
                                        }
                                        state = Some(result.state);
                                        expected = result.expected_discontinuity;
                                    }
                                    Decision::Clamp => {
                                        stats.clamped += 1;
                                        state = Some(result.state);
                                        expected = result.expected_discontinuity;
                                    }
                                    Decision::RejectAndSnapBack => {
                                        stats.rejected += 1;
                                        expected = result.expected_discontinuity;
                                    }
                                    Decision::Ignore => {
                                        stats.ignored += 1;
                                        if is_teleport {
                                            stats.ignored_teleports += 1;
                                            let dx = f64::from(
                                                update.position[0] - previous.position[0],
                                            );
                                            let dy = f64::from(
                                                update.position[1] - previous.position[1],
                                            );
                                            let dz = f64::from(
                                                update.position[2] - previous.position[2],
                                            );
                                            stats.teleport_stale_distance =
                                                Some(dx.hypot(dz).hypot(dy));
                                            stats.teleport_stale_delay = Some(dt);
                                        }
                                        expected = result.expected_discontinuity;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    stats
}

fn read_run_manifest(directory: &Path) -> Manifest {
    let manifest_path = directory.join("manifest.json");
    if manifest_path.is_file() {
        return serde_json::from_slice(&fs::read(manifest_path).expect("read baseline manifest"))
            .expect("parse baseline manifest");
    }

    let index_path = directory.join("frames/live-index.jsonl");
    let index = fs::read_to_string(index_path).expect("read live capture index");
    let frames = index
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse live capture index record"))
        .collect();
    Manifest { frames }
}

fn set_server_pose(
    state: &mut Option<MovementState>,
    last_position_time: &mut Option<f64>,
    zone_id: u32,
    position: [f32; 3],
    time: f64,
) {
    let (direction, movement_state, unknown) =
        state.as_ref().map_or(([0.0, 0.0, 1.0], 0, 0), |previous| {
            (previous.direction, previous.state, previous.unknown)
        });
    *state = Some(MovementState {
        zone_id,
        position,
        direction,
        state: movement_state,
        unknown,
    });
    *last_position_time = Some(time);
}

fn tunnel(message: &[u8]) -> Option<(u16, &[u8])> {
    if message.len() < 9 || message[2] > 1 {
        return None;
    }
    let outer = u16::from_le_bytes(message[..2].try_into().ok()?);
    if !matches!(outer, 5 | 6) {
        return None;
    }
    let length = u32::from_le_bytes(message[3..7].try_into().ok()?) as usize;
    if !(2..=MAX_MESSAGE).contains(&length) || length != message.len() - 7 {
        return None;
    }
    Some((outer, &message[7..]))
}

fn application_messages(message: &[u8]) -> Result<Vec<&[u8]>, ()> {
    if message.len() < 2 || message[..2] != [0, 25] {
        return Ok(vec![message]);
    }
    let mut offset = 2;
    let mut output = Vec::new();
    while offset < message.len() {
        let mut length = usize::from(*message.get(offset).ok_or(())?);
        offset += 1;
        if length == 255 {
            let short = u16::from_be_bytes(
                message
                    .get(offset..offset + 2)
                    .ok_or(())?
                    .try_into()
                    .unwrap(),
            );
            offset += 2;
            length = usize::from(short);
            if short == 255 {
                length = u32::from_be_bytes(
                    message
                        .get(offset..offset + 4)
                        .ok_or(())?
                        .try_into()
                        .unwrap(),
                ) as usize;
                offset += 4;
            }
        }
        let end = offset
            .checked_add(length)
            .filter(|end| *end <= message.len())
            .ok_or(())?;
        if length < 2 {
            return Err(());
        }
        let child = &message[offset..end];
        if child.get(..2) == Some(&[0, 25]) || output.len() >= 128 {
            return Err(());
        }
        output.push(child);
        offset = end;
    }
    if output.is_empty() {
        Err(())
    } else {
        Ok(output)
    }
}

fn parse_position(inner: &[u8]) -> Option<MovementUpdate> {
    if inner.len() != 36 || u16::from_le_bytes(inner[..2].try_into().ok()?) != 125 {
        return None;
    }
    let mut floats = [0.0f32; 6];
    for (index, value) in floats.iter_mut().enumerate() {
        let offset = 10 + index * 4;
        *value = f32::from_le_bytes(inner.get(offset..offset + 4)?.try_into().ok()?);
    }
    Some(MovementUpdate {
        position: [floats[0], floats[1], floats[2]],
        direction: [floats[3], floats[4], floats[5]],
        state: inner[34],
        unknown: inner[35],
    })
}

fn parse_location(inner: &[u8]) -> Option<([f32; 3], bool)> {
    if inner.len() != 38 || inner[..4] != [38, 0, 12, 0] {
        return None;
    }
    let mut position = [0.0; 3];
    for (index, value) in position.iter_mut().enumerate() {
        let offset = 4 + index * 4;
        *value = f32::from_le_bytes(inner.get(offset..offset + 4)?.try_into().ok()?);
    }
    Some((position, inner[36] & 1 != 0))
}

fn parse_zone_details(inner: &[u8]) -> Option<u32> {
    if inner.len() < 2 + 4 || u16::from_le_bytes(inner[..2].try_into().ok()?) != 43 {
        return None;
    }
    let mut offset = 2usize;
    let scene_len = u32::from_le_bytes(inner.get(offset..offset + 4)?.try_into().ok()?) as usize;
    offset = offset.checked_add(4 + scene_len)?;
    offset = offset.checked_add(4 + 2)?;
    let sky_len = u32::from_le_bytes(inner.get(offset..offset + 4)?.try_into().ok()?) as usize;
    offset = offset.checked_add(4 + sky_len + 1)?;
    Some(u32::from_le_bytes(
        inner.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

/// T451: a boarded ride scales the base by its speed over the base, held to the mount range.
#[test]
fn a_ride_scales_the_allowance_within_the_mount_range() {
    let multiplier = |speed| SpeedModifiers::riding(speed).mount_multiplier;
    assert_eq!(multiplier(None), 1.0);
    assert_eq!(multiplier(Some(0.0)), 1.0, "a ride that stands still");
    assert_eq!(multiplier(Some(12.5)), 1.25);
    assert_eq!(multiplier(Some(64.0)), 1.5, "capped");
    assert_eq!(SpeedModifiers::riding(None).base_speed_per_second, 10.0);
}
