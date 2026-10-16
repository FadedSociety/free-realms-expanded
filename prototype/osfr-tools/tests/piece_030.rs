//! Work item 030 acceptance: house native acceptance replay and gate report.
//!
//! These vectors run against `src/restoration/piece_030.rs`, which includes `src/housing.rs` by
//! `#[path]` so the module compiles before the lead registers either one in `src/lib.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside Git, see
//! `notes/Research/Pieces/030 House native acceptance.md`):
//! * housing is the inner-opcode family of base opcode 127
//!   (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`);
//! * the shipped client constructs the same members with `FUN_00aa4990(<byte>)` immediately before
//!   the matching `::vftable` (`lab/decompiled/client/FreeRealms/pseudocode.c`): enter request 19 at
//!   1386220, zone data 45 at 1393137, place fixture 2 at 1384411, fixture update 40 at 1390143,
//!   update fixture position 51 at 1386478, pickup 3 at 1386096, fixture remove 41 at 1386050,
//!   housing error 46 at 1384530 and instance data 28 at 1384498;
//! * the client's placement denial path prints `Fixture placement denied.` and reads
//!   `HousingMaxFixtures` from the property definition;
//! * permission denial is housing error code 2 with the recovered message
//!   "You do not have permission to enter that house." (locale id 421055);
//! * the acceptance lot is `Houses.json` row 1: property 1, zone 1, name id 6514, build area
//!   x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8.
//!
//! Everything the replay, the ordering rules and the error mapping do is RECONSTRUCTED. A local
//! replay can never report native acceptance; only a transcript captured from the running client
//! with the same origin can, and even then the lead confirms it.

#[path = "../src/restoration/piece_030.rs"]
#[allow(dead_code)]
mod piece_030;

use piece_030::{
    ACCEPTANCE_BUILD_AREA, ACCEPTANCE_NAME_ID, ACCEPTANCE_PROPERTY_ID, ACCEPTANCE_WIRE,
    ACCEPTANCE_ZONE_ID, AcceptanceExpectation, AcceptanceReport, AcceptanceVerb, BASE_OPCODE,
    EvidenceOrigin, FixtureRow, GateStatus, HOUSING_ERROR_PERMISSION_DENIED, ReplayReport, RowBody,
    SessionRow, TranscriptError, WireDirection, acceptance_property, evaluate_transcript,
    replay_reconstructed_session, wire_member,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn temp_snapshot_path(name: &str) -> PathBuf {
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    let directory =
        std::env::temp_dir().join(format!("osfr-piece-030-{}-{sequence}", std::process::id()));
    fs::create_dir_all(&directory).expect("temp directory");
    directory.join(name)
}

fn replay(path: &Path) -> ReplayReport {
    replay_reconstructed_session(path).expect("the reconstructed scenario replays")
}

fn gate_status(report: &AcceptanceReport, verb: AcceptanceVerb) -> GateStatus {
    report.gate(verb).status.clone()
}

fn assert_passed(report: &AcceptanceReport, verb: AcceptanceVerb) {
    match gate_status(report, verb) {
        GateStatus::Pass { .. } => {}
        other => panic!("{verb:?} should pass, got {other:?}"),
    }
}

fn fail_reason(status: &GateStatus) -> String {
    match status {
        GateStatus::Fail { reason } => reason.clone(),
        other => panic!("expected a failure, got {other:?}"),
    }
}

fn unproven_missing(status: &GateStatus) -> &'static str {
    match status {
        GateStatus::Unproven { missing } => missing,
        other => panic!("expected unproven evidence, got {other:?}"),
    }
}

fn without(rows: &[SessionRow], predicate: impl Fn(&SessionRow) -> bool) -> Vec<SessionRow> {
    rows.iter().filter(|row| !predicate(row)).cloned().collect()
}

fn expectation(report: &ReplayReport) -> AcceptanceExpectation {
    AcceptanceExpectation {
        house_guid: report.lot_guid,
        owner_guid: report.owner_guid,
        permitted_guest_guid: Some(report.guest_guid),
        zone_definition_id: ACCEPTANCE_ZONE_ID,
    }
}

fn evaluate(report: &ReplayReport, rows: &[SessionRow]) -> AcceptanceReport {
    evaluate_transcript(EvidenceOrigin::NativeCapture, &expectation(report), rows)
        .expect("the transcript is well formed")
}

fn instance_data(rows: &[SessionRow]) -> Vec<FixtureRow> {
    rows.iter()
        .filter_map(|row| match &row.body {
            RowBody::InstanceData { fixtures, .. } => Some(fixtures.clone()),
            _ => None,
        })
        .next_back()
        .expect("the transcript carries instance data")
}

/// The acceptance wire table is derived from the supplied server enum and the shipped client's own
/// constructors, so a wrong number here must fail rather than silently shift the gate.
#[test]
fn acceptance_wire_table_lists_the_recovered_members() {
    assert_eq!(BASE_OPCODE, 127);
    assert_eq!(HOUSING_ERROR_PERMISSION_DENIED, 2);
    assert_eq!(ACCEPTANCE_PROPERTY_ID, 1);
    assert_eq!(ACCEPTANCE_ZONE_ID, 1);
    assert_eq!(ACCEPTANCE_NAME_ID, 6514);
    assert_eq!(
        ACCEPTANCE_BUILD_AREA,
        [322.0, -100.0, 392.1, 443.3, 60.0, 477.8]
    );

    let expected = [
        (
            1,
            "ClientHousingPacketPlaceFixtureRequest",
            WireDirection::ClientToServer,
        ),
        (
            2,
            "ClientHousingPacketPlaceFixture",
            WireDirection::ServerToClient,
        ),
        (
            3,
            "ClientHousingPacketPickupFixture",
            WireDirection::ClientToServer,
        ),
        (
            5,
            "ClientHousingPacketSaveFixture",
            WireDirection::ClientToServer,
        ),
        (
            19,
            "ClientHousingPacketEnterRequest",
            WireDirection::ClientToServer,
        ),
        (
            28,
            "HousingPacketInstanceData",
            WireDirection::ServerToClient,
        ),
        (
            40,
            "HousingPacketFixtureUpdate",
            WireDirection::ServerToClient,
        ),
        (
            41,
            "HousingPacketFixtureRemove",
            WireDirection::ServerToClient,
        ),
        (45, "HousingPacketZoneData", WireDirection::ServerToClient),
        (
            46,
            "HousingPacketHousingError",
            WireDirection::ServerToClient,
        ),
        (
            51,
            "HousingPacketUpdateFixturePosition",
            WireDirection::ServerToClient,
        ),
    ];
    for (opcode, packet, direction) in expected {
        let member = wire_member(opcode).unwrap_or_else(|| panic!("missing opcode {opcode}"));
        assert_eq!(member.packet, packet, "opcode {opcode}");
        assert_eq!(member.direction, direction, "opcode {opcode}");
    }
    assert!(
        ACCEPTANCE_WIRE.iter().all(|member| member.opcode > 0),
        "every acceptance member carries a recovered inner opcode"
    );
    assert!(
        wire_member(0).is_none(),
        "opcode zero is reserved for the restart marker"
    );
}

#[test]
fn every_verb_requires_recovered_opcodes() {
    for verb in AcceptanceVerb::ALL {
        let required = verb.required_opcodes();
        assert!(!required.is_empty(), "{verb:?} needs evidence");
        for opcode in required {
            assert!(
                wire_member(*opcode).is_some(),
                "{verb:?} requires unrecovered opcode {opcode}"
            );
        }
        assert!(!verb.name().is_empty());
    }
}

#[test]
fn local_replay_passes_all_six_gates() {
    let path = temp_snapshot_path("housing-replay.json");
    let replay = replay(&path);
    for verb in AcceptanceVerb::ALL {
        assert_passed(&replay.report, verb);
    }
    assert!(replay.report.is_evidence_complete());
    assert!(
        !replay.report.native_evidence_complete(),
        "an in-process replay is not native evidence"
    );
    assert_eq!(replay.report.origin, EvidenceOrigin::LocalReplay);
    let _ = fs::remove_file(&path);
}

/// The restart gate is only as real as the state behind it, so the replay's own snapshot must
/// survive a fresh `Housing` load with the moved fixture present and the removed fixture gone.
#[test]
fn restart_round_trip_persists_the_moved_fixture_and_the_removal() {
    let path = temp_snapshot_path("housing-restart.json");
    let replay = replay(&path);
    assert!(replay.snapshot_path.exists(), "the replay wrote a snapshot");

    let mut restarted = piece_030::Housing::new();
    restarted
        .load_state(&replay.snapshot_path)
        .expect("the snapshot restores");
    let lot = restarted
        .saved_fixtures(replay.lot_guid)
        .expect("the lot survives the restart");
    assert_eq!(lot.count, 1, "exactly one fixture survives");
    let surviving = &lot.fixtures[0];
    assert_eq!(surviving.fixture_guid, replay.surviving_fixture_guid);
    assert_eq!(surviving.transform.position, replay.moved_position);

    let listed = instance_data(&replay.transcript);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].fixture_guid, replay.surviving_fixture_guid);
    assert_eq!(listed[0].position, replay.moved_position);
    assert!(
        !listed
            .iter()
            .any(|row| row.fixture_guid == replay.removed_fixture_guid),
        "a picked-up fixture must not come back after restart"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn native_capture_origin_is_the_only_route_to_complete_evidence() {
    let path = temp_snapshot_path("housing-native-origin.json");
    let replay = replay(&path);
    let native = evaluate(&replay, &replay.transcript);
    assert!(native.native_evidence_complete());
    assert_eq!(native.origin, EvidenceOrigin::NativeCapture);
    let _ = fs::remove_file(&path);
}

#[test]
fn restart_gate_is_unproven_without_instance_data() {
    let path = temp_snapshot_path("housing-no-instance.json");
    let replay = replay(&path);
    let rows = without(&replay.transcript, |row| {
        matches!(row.body, RowBody::InstanceData { .. })
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Restart);
    assert!(
        unproven_missing(&status).contains("instance data"),
        "restart must stay unproven, got {status:?}"
    );
    assert_passed(&report, AcceptanceVerb::Enter);
    assert_passed(&report, AcceptanceVerb::Place);
    let _ = fs::remove_file(&path);
}

#[test]
fn visit_gate_is_unproven_without_the_guest_enter() {
    let path = temp_snapshot_path("housing-no-guest.json");
    let replay = replay(&path);
    let guest = replay.guest_guid;
    let rows = without(&replay.transcript, |row| {
        row.actor == guest && matches!(row.body, RowBody::EnterRequest { .. })
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Visit);
    assert!(
        unproven_missing(&status).contains("guest"),
        "a visit needs a second actor, got {status:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn visit_gate_fails_when_an_unpermitted_actor_edits_successfully() {
    let path = temp_snapshot_path("housing-bad-guest.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    let seq = rows.last().map(|row| row.seq).unwrap_or(0) + 1;
    rows.push(SessionRow {
        seq,
        actor: replay.stranger_guid,
        opcode: 1,
        body: RowBody::PlaceFixtureRequest {
            house_guid: replay.lot_guid,
        },
    });
    rows.push(SessionRow {
        seq: seq + 1,
        actor: replay.stranger_guid,
        opcode: 2,
        body: RowBody::PlaceFixture {
            house_guid: replay.lot_guid,
            fixture_guid: 0x0f00_0000_0000_dead,
        },
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Visit);
    let reason = fail_reason(&status);
    assert!(
        reason.contains(&replay.stranger_guid.to_string()),
        "the offending actor must be named, got {reason}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn visit_gate_is_unproven_while_an_unauthorised_edit_is_unanswered() {
    let path = temp_snapshot_path("housing-unanswered-denial.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    let seq = rows.last().map(|row| row.seq).unwrap_or(0) + 1;
    rows.push(SessionRow {
        seq,
        actor: replay.stranger_guid,
        opcode: 3,
        body: RowBody::PickupFixture {
            fixture_guid: replay.removed_fixture_guid,
        },
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Visit);
    assert!(
        unproven_missing(&status).contains("unauthorised"),
        "an unanswered refusal keeps the gate unproven, got {status:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn move_gate_is_unproven_without_movement_evidence() {
    let path = temp_snapshot_path("housing-no-move.json");
    let replay = replay(&path);
    let rows = without(&replay.transcript, |row| {
        matches!(
            row.body,
            RowBody::FixtureUpdate { .. } | RowBody::UpdateFixturePosition { .. }
        )
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Move);
    assert!(
        unproven_missing(&status).contains("movement"),
        "a move needs a position update, got {status:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn move_gate_fails_on_a_repeated_transform() {
    let path = temp_snapshot_path("housing-repeated-move.json");
    let replay = replay(&path);
    let survivor = replay.surviving_fixture_guid;
    let mut rows = replay.transcript.clone();
    for row in rows.iter_mut() {
        if row.body
            == (RowBody::UpdateFixturePosition {
                fixture_guid: survivor,
                position: replay.moved_position,
            })
        {
            row.body = RowBody::UpdateFixturePosition {
                fixture_guid: survivor,
                position: replay.placed_position,
            };
        }
    }
    let report = evaluate(&replay, &rows);
    let reason = fail_reason(&gate_status(&report, AcceptanceVerb::Move));
    assert!(
        reason.contains("transform"),
        "a move must change the transform, got {reason}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn move_gate_fails_when_the_moved_guid_was_never_placed() {
    let path = temp_snapshot_path("housing-unknown-move.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    let seq = rows.last().map(|row| row.seq).unwrap_or(0) + 1;
    rows.push(SessionRow {
        seq,
        actor: replay.owner_guid,
        opcode: 51,
        body: RowBody::UpdateFixturePosition {
            fixture_guid: 0x0f00_0000_0000_beef,
            position: replay.moved_position,
        },
    });
    let report = evaluate(&replay, &rows);
    let reason = fail_reason(&gate_status(&report, AcceptanceVerb::Move));
    assert!(
        reason.contains("never placed"),
        "an unknown fixture cannot move, got {reason}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn remove_gate_requires_the_pickup_request_and_response() {
    let path = temp_snapshot_path("housing-no-pickup-request.json");
    let replay = replay(&path);
    let removed = replay.removed_fixture_guid;
    let rows = without(&replay.transcript, |row| {
        row.actor == replay.owner_guid
            && matches!(row.body, RowBody::PickupFixture { fixture_guid } if fixture_guid == removed)
    });
    let report = evaluate(&replay, &rows);
    let status = gate_status(&report, AcceptanceVerb::Remove);
    assert!(
        unproven_missing(&status).contains("pickup"),
        "removal needs the client request, got {status:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn restart_gate_fails_when_a_removed_fixture_reappears() {
    let path = temp_snapshot_path("housing-ghost-fixture.json");
    let replay = replay(&path);
    let removed = replay.removed_fixture_guid;
    let mut rows = replay.transcript.clone();
    for row in rows.iter_mut() {
        if let RowBody::InstanceData { fixtures, .. } = &mut row.body {
            fixtures.push(FixtureRow {
                fixture_guid: removed,
                position: replay.placed_position,
            });
        }
    }
    let report = evaluate(&replay, &rows);
    let reason = fail_reason(&gate_status(&report, AcceptanceVerb::Restart));
    assert!(
        reason.contains("appeared"),
        "a picked-up fixture must not return, got {reason}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn restart_gate_fails_when_a_moved_fixture_is_restored_at_the_old_transform() {
    let path = temp_snapshot_path("housing-stale-transform.json");
    let replay = replay(&path);
    let survivor = replay.surviving_fixture_guid;
    let mut rows = replay.transcript.clone();
    for row in rows.iter_mut() {
        if let RowBody::InstanceData { fixtures, .. } = &mut row.body {
            for fixture in fixtures.iter_mut().filter(|f| f.fixture_guid == survivor) {
                fixture.position = replay.placed_position;
            }
        }
    }
    let report = evaluate(&replay, &rows);
    let reason = fail_reason(&gate_status(&report, AcceptanceVerb::Restart));
    assert!(
        reason.contains("transform"),
        "the restart must carry the moved transform, got {reason}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn row_opcode_must_match_its_body() {
    let path = temp_snapshot_path("housing-opcode-mismatch.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    let row = rows
        .iter_mut()
        .find(|row| matches!(row.body, RowBody::EnterRequest { .. }))
        .expect("an enter request exists");
    row.opcode = 45;
    let error: TranscriptError =
        evaluate_transcript(EvidenceOrigin::LocalReplay, &expectation(&replay), &rows)
            .expect_err("a mismatched opcode is rejected");
    assert!(error.reason.contains("enter"), "got {}", error.reason);
    let _ = fs::remove_file(&path);
}

#[test]
fn rows_must_be_strictly_ordered() {
    let path = temp_snapshot_path("housing-unordered.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    rows.swap(0, 1);
    let error = evaluate_transcript(EvidenceOrigin::LocalReplay, &expectation(&replay), &rows)
        .expect_err("out-of-order rows are rejected");
    assert!(error.reason.contains("order"), "got {}", error.reason);
    let _ = fs::remove_file(&path);
}

#[test]
fn evidence_for_another_house_is_rejected() {
    let path = temp_snapshot_path("housing-foreign-house.json");
    let replay = replay(&path);
    let mut rows = replay.transcript.clone();
    let seq = rows.last().map(|row| row.seq).unwrap_or(0) + 1;
    rows.push(SessionRow {
        seq,
        actor: replay.owner_guid,
        opcode: 19,
        body: RowBody::EnterRequest {
            house_guid: replay.lot_guid ^ 0x1111,
        },
    });
    let error = evaluate_transcript(EvidenceOrigin::LocalReplay, &expectation(&replay), &rows)
        .expect_err("foreign evidence is rejected");
    assert!(error.reason.contains("house"), "got {}", error.reason);
    let _ = fs::remove_file(&path);
}

#[test]
fn acceptance_property_uses_the_recovered_row_one_bounds() {
    let property = acceptance_property();
    assert_eq!(property.property_id, ACCEPTANCE_PROPERTY_ID);
    assert_eq!(property.zone_id, ACCEPTANCE_ZONE_ID);
    assert_eq!(property.name_id, ACCEPTANCE_NAME_ID);
    assert_eq!(property.build_areas, vec![ACCEPTANCE_BUILD_AREA]);
    property
        .build_areas
        .iter()
        .for_each(|area| assert!(area.iter().all(|value| value.is_finite())));
}

/// Writes the golden replay evidence for the lead: the real snapshot, the six gate outcomes, and
/// the transcript as both text and JSON, so a native capture can be diffed against it.
#[test]
#[ignore = "requires OSFR_PIECE_030_REPORT_DIR to point at a writable evidence directory"]
fn exports_the_replay_evidence_for_the_lead() {
    let directory = std::env::var_os("OSFR_PIECE_030_REPORT_DIR")
        .map(PathBuf::from)
        .expect("set OSFR_PIECE_030_REPORT_DIR to the evidence directory");
    fs::create_dir_all(&directory).expect("the evidence directory is writable");

    let snapshot = directory.join("piece-030-replay-snapshot.json");
    let replay = replay(&snapshot);

    fs::write(
        directory.join("piece-030-gates.txt"),
        replay.report.render(),
    )
    .expect("the gate report is writable");
    fs::write(
        directory.join("piece-030-transcript.txt"),
        piece_030::render_rows(&replay.transcript),
    )
    .expect("the transcript is writable");
    fs::write(
        directory.join("piece-030-transcript.json"),
        serde_json::to_vec_pretty(&replay.transcript).expect("the transcript serializes"),
    )
    .expect("the transcript json is writable");
    fs::write(
        directory.join("piece-030-gates.json"),
        serde_json::to_vec_pretty(&replay.report).expect("the report serializes"),
    )
    .expect("the gate json is writable");

    assert!(
        replay.report.is_evidence_complete(),
        "the exported replay must pass every gate"
    );
    assert!(
        !replay.report.native_evidence_complete(),
        "exported local replay evidence is not a native acceptance"
    );
}
