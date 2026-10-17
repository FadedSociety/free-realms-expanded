//! Kart race service (inner opcode 33 lifecycle) - owned vectors.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `notes/Research/Kart Race Finish Protocol.md` - subtype 1 stores the game state at
//!   `processor+0x194`; subtype 14 carries `laps`, `lap limit`, `best lap`, `total time`; the
//!   results screen is client-derived from `lap limit <= laps`; there is no server finish packet.
//! * `notes/Research/Kart Local Racer and Gate Binding.md` - subtype 7 is `u64 guid | 3 x f32
//!   position | f32 1.0`, send-only, client-reported and **not** trustable; the 31 captured hits are
//!   one local guid over 51.6 s with a peak step speed of 40.5 m/s.
//! * `notes/Research/Kart Vehicle Configuration and Input.md` - subtype 6 is an 85-byte send-only
//!   bulk block with an unrecovered interior; subtype 17 shares the subtype-7 layout.
//! * the dispatcher `FUN_00ba74d0` (pinned export sha256 `f84a5867...`): case `0xa` parses the
//!   countdown status through `FUN_00ba1850` -> `FUN_00ba04f0` (`u64 | u32`, value stored at
//!   `processor+0x1b4`, pending flag `+0x1c2`); `FUN_00ba6300` shows `Countdown:SetCount`, clears
//!   the flag and echoes the value with the local racer guid. Client-sent subtypes 2/6/7/9/17/19/22
//!   have no client parse case.
//! * the capture cadence re-derived by
//!   `lab/reports/kart-service-20261009/probe-kart-service.py`: states `4..9` and countdown `4..0`
//!   at the offsets used by [`recovered_schedule`], 31 gate hits over 51.635 s.
//! * the live v12 capture
//!   `lab/reports/kart-playable-v4-20261009/client-baseline-20261010T005751Z/applications.json`
//!   holds 17 byte-identical `33/19` `VehicleRacePacketUseItemRequest` frames (payload
//!   `01 00 00 00 00 00 00 00`) and three `33/22` awards answered by three header-only `33/23`;
//!   there is no `33/17`, so no pickup claim was ever sent in that session.
//! * pseudocode.c (pinned export) `FUN_00b9c620` / `FUN_00b99a90` name subtypes 19 / 20 as
//!   `VehicleRacePacketUseItemRequest` / `VehicleRacePacketUseItemResponse`; dispatcher
//!   `FUN_00ba74d0` case `0x14` applies the response's single u32 item type (type 5 is the boost)
//!   and then pops the queue (`FUN_00fe94c0`), while `FUN_00fe9410` pushes into the three-slot
//!   queue. `33/15` (spawn pickup) staying unpinned is why a `33/17` claim awards nothing here.
//!
//! Exercises the registered production modules.
use osfr_tools::{kart_entry, kart_pickups, kart_progress_native, kart_racing, kart_service};

use kart_entry::EntryGate;
use kart_progress_native::ProgressPayload;
use kart_racing::{Phase, RewardSchedule};
use kart_service::{
    BOOST_INTERVAL_MS, BOOST_ITEM_TYPE, BOOST_REQUEST_SUBTYPE, BOOST_RESPONSE_SUBTYPE,
    CLAIM_CREDIT_CAP_SECONDS, CLAIM_CREDIT_WINDOW_MS, COUNTDOWN_SUBTYPE, GAME_STATE_SUBTYPE,
    GATE_CURTAIN_HALF_DEPTH_M, GATE_HIT_PAYLOAD_LEN, GATE_HIT_SUBTYPE, GateBand, GateCurtain,
    GateWaypoint, ITEM_QUEUE_CAP, KartService, KartServiceContent, MAX_GATE_BAND_HALF_M,
    MAX_GATE_CURTAIN_HALF_DEPTH_M, MAX_GATE_CURTAIN_LEAVES, NPC_COUNT_SUBTYPE,
    OWNED_ID_PAYLOAD_LEN, PICKUP_HIT_SUBTYPE, PLAYER_COUNT_SUBTYPE, PROGRESS_SUBTYPE, RaceBounds,
    RaceContext, RacePhase, RaceStep, SERVER_DATA_PAYLOAD_LEN, SERVER_DATA_SUBTYPE,
    SHIELD_DURATION_MS, ScheduledFrame, ServiceError, ServiceOutcome, ServiceRefusal,
    ServiceSnapshot, USE_ITEM_RESPONSE_SUBTYPE, USE_ITEM_SUBTYPE,
};

const LAPS: u32 = 2;

/// The service works in absolute server time: `begin` binds the race at this instant and every
/// schedule offset is added to it, so a late bind (a real gateway starts far from zero) is safe.
const BEGIN_MS: u64 = 100_000;

fn at(offset_ms: u64) -> u64 {
    BEGIN_MS + offset_ms
}

fn synthetic_progress_template() -> ProgressPayload {
    ProgressPayload {
        racer_guid: 0,
        slot: 0,
        unknown_0c_bits: 0,
        unknown_10_bits: 0x7f7f_ffff,
        laps_completed: 0,
        local_racer: 1,
        lap_limit: LAPS as i32,
        unknown_1d_bits: 0,
        best_lap_bits: 0,
        total_time_bits: 0,
        tail_0x29: 0,
        tail_0x31: 0,
        tail_0x39: 0,
        tail_0x41: 0,
    }
}

/// The recovered capture cadence, in ms relative to the `33/9` register frame. The entry handler
/// owns state 3 and the player count, so this service starts at state 4/npc count (+3026 ms).
fn recovered_schedule() -> Vec<RaceStep> {
    vec![
        RaceStep {
            at_ms: 3_026,
            frame: ScheduledFrame::GameState(4),
        },
        RaceStep {
            at_ms: 3_026,
            frame: ScheduledFrame::NpcCount(7),
        },
        RaceStep {
            at_ms: 3_353,
            frame: ScheduledFrame::GameState(5),
        },
        RaceStep {
            at_ms: 8_246,
            frame: ScheduledFrame::GameState(6),
        },
        RaceStep {
            at_ms: 8_694,
            frame: ScheduledFrame::GameState(7),
        },
        RaceStep {
            at_ms: 9_290,
            frame: ScheduledFrame::GameState(8),
        },
        RaceStep {
            at_ms: 9_290,
            frame: ScheduledFrame::Countdown(4),
        },
        RaceStep {
            at_ms: 10_074,
            frame: ScheduledFrame::Countdown(3),
        },
        RaceStep {
            at_ms: 10_529,
            frame: ScheduledFrame::Countdown(2),
        },
        RaceStep {
            at_ms: 11_954,
            frame: ScheduledFrame::Countdown(1),
        },
        RaceStep {
            at_ms: 13_241,
            frame: ScheduledFrame::Countdown(0),
        },
        RaceStep {
            at_ms: 14_505,
            frame: ScheduledFrame::GameState(9),
        },
    ]
}

/// Four reconstructed gate waypoints 10 m apart on a straight line; radius 2 m.
fn synthetic_gates() -> Vec<GateWaypoint> {
    (1..=4)
        .map(|index| GateWaypoint {
            position: [index as f32 * 10.0, 0.0, 0.0],
            radius: 3.0,
        })
        .collect()
}

fn synthetic_content() -> KartServiceContent {
    KartServiceContent {
        speedway: 31,
        laps: LAPS,
        reward: Some(RewardSchedule { coins: 40, xp: 60 }),
        gates: synthetic_gates(),
        schedule: recovered_schedule(),
        progress_template: synthetic_progress_template(),
        start_position: [0.0, 0.0, 0.0],
        bounds: RaceBounds {
            max_speed_mps: 60.0,
            max_step_ms: 60_000,
            max_race_ms: 600_000,
            max_claims: 64,
            max_observations: 4_096,
        },
        gate_band: None,
        gate_curtains: Vec::new(),
        pickups: Vec::new(),
        pickup_respawn: None,
        weapon_effects: false,
    }
}

fn weapon_test_content(kind: u32) -> KartServiceContent {
    let mut content = shield_content();
    content.weapon_effects = true;
    content.pickups[0].item_type = kind;
    content
}

fn weapon_pose_frame(position: [f32; 3]) -> Vec<u8> {
    let mut body = vec![0; 85];
    for (index, value) in [1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
        .iter()
        .chain(position.iter())
        .enumerate()
    {
        body[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
    }
    body[81..85].copy_from_slice(&1u32.to_le_bytes());
    inner(SERVER_DATA_SUBTYPE, &body)
}

#[test]
fn quiet_native_pose_still_allows_weapon_use_within_the_open_race() {
    for kind in [1, 3] {
        let (mut service, context) = racing(weapon_test_content(kind));
        start_box_claim(&mut service, &context, at(20_000));
        service
            .handle_inner(&weapon_pose_frame([0.0; 3]), at(20_020))
            .unwrap();
        // The native idle sender is quiet for over 40 s in the saved-wire diagnostic.
        // A valid cached placement remains the last known placement in this session.
        let outcome = service
            .handle_inner(
                &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
                at(65_000),
            )
            .unwrap();
        assert_eq!(outcome.replies().len(), 2, "quiet pose refused kind {kind}");
        assert_eq!(outcome.replies()[1][2], if kind == 1 { 24 } else { 29 });
        assert!(service.snapshot().unwrap().queued_items.is_empty());
        let updates = service.tick(at(65_100)).unwrap();
        assert!(
            updates
                .replies()
                .iter()
                .any(|r| r[2] == if kind == 1 { 26 } else { 31 })
        );
    }
}

#[test]
fn weapon_use_requires_owned_valid_reachable_pose_without_spending_item() {
    let (mut service, context) = racing(weapon_test_content(1));
    start_box_claim(&mut service, &context, at(20_000));
    let use_frame = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        refusal(&service.handle_inner(&use_frame, at(20_010)).unwrap()),
        &ServiceRefusal::WeaponUnavailable
    );
    assert_eq!(service.snapshot().unwrap().queued_items, vec![1]);
    service
        .handle_inner(&weapon_pose_frame([0.0; 3]), at(20_020))
        .unwrap();
    let foreign = inner(USE_ITEM_SUBTYPE, &(context.player_guid + 1).to_le_bytes());
    assert!(matches!(
        service.handle_inner(&foreign, at(20_021)).unwrap(),
        ServiceOutcome::Refused(_)
    ));
    assert!(matches!(
        service
            .handle_inner(&weapon_pose_frame([10_000.0, 0.0, 0.0]), at(20_030))
            .unwrap(),
        ServiceOutcome::Refused(_)
    ));
    service
        .handle_inner(&weapon_pose_frame([0.0; 3]), at(21_030))
        .unwrap();
    let outcome = service.handle_inner(&use_frame, at(21_040)).unwrap();
    assert_eq!(
        outcome.replies()[0],
        inner(USE_ITEM_RESPONSE_SUBTYPE, &1u32.to_le_bytes())
    );
    assert_eq!(&outcome.replies()[1][2..4], &24u16.to_le_bytes());
    assert_eq!(outcome.replies()[1].len(), 92);
    assert!(service.snapshot().unwrap().queued_items.is_empty());
    assert_eq!(
        refusal(&service.handle_inner(&use_frame, at(21_041)).unwrap()),
        &ServiceRefusal::NoItemQueued
    );
}

#[test]
fn weapon_pose_rejects_nan_invalid_basis_and_clock_regression() {
    let (mut service, _) = racing(weapon_test_content(3));
    let mut bad = weapon_pose_frame([0.0; 3]);
    bad[4..8].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(matches!(
        service.handle_inner(&bad, at(20_000)).unwrap(),
        ServiceOutcome::Refused(_)
    ));
    bad[4..8].copy_from_slice(&0f32.to_le_bytes());
    assert!(matches!(
        service.handle_inner(&bad, at(20_000)).unwrap(),
        ServiceOutcome::Refused(_)
    ));
    service
        .handle_inner(&weapon_pose_frame([0.0; 3]), at(20_000))
        .unwrap();
    assert!(matches!(
        service
            .handle_inner(&weapon_pose_frame([0.0; 3]), at(19_999))
            .unwrap(),
        ServiceOutcome::Refused(_)
    ));
}

#[test]
fn mine_use_emits_original_create_and_restores_bounded_lifecycle() {
    let content = weapon_test_content(3);
    let (mut service, context) = racing(content.clone());
    start_box_claim(&mut service, &context, at(20_000));
    service
        .handle_inner(&weapon_pose_frame([0.0; 3]), at(20_010))
        .unwrap();
    let outcome = service
        .handle_inner(
            &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
            at(20_020),
        )
        .unwrap();
    assert_eq!(&outcome.replies()[1][2..4], &29u16.to_le_bytes());
    assert_eq!(outcome.replies()[1].len(), 84);
    let before = service.snapshot().unwrap();
    let mut restored = KartService::restore(content, before.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(restored.snapshot().unwrap(), before);
    let ended = restored.tick(at(90_000)).unwrap();
    assert!(
        ended
            .replies()
            .iter()
            .any(|f| f[2..4] == 30u16.to_le_bytes())
    );
    assert!(
        !restored
            .tick(at(90_001))
            .unwrap()
            .replies()
            .iter()
            .any(|f| f[2..4] == 30u16.to_le_bytes())
    );
    assert_eq!(restored.snapshot().unwrap().session, before.session);
}

#[test]
fn weapon_state_is_content_bound_and_cannot_resume_foreign_actors() {
    let content = weapon_test_content(1);
    let (mut service, context) = racing(content.clone());
    start_box_claim(&mut service, &context, at(20_000));
    service
        .handle_inner(&weapon_pose_frame([0.0; 3]), at(20_010))
        .unwrap();
    service
        .handle_inner(
            &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
            at(20_020),
        )
        .unwrap();
    let snapshot = service.snapshot().unwrap();
    let mut disabled = content.clone();
    disabled.weapon_effects = false;
    assert!(KartService::restore(disabled, snapshot.clone()).is_err());
    let mut foreign = snapshot.clone();
    foreign.weapons.weapons[0].owner = context.player_guid + 1;
    assert!(KartService::restore(content.clone(), foreign).is_err());
    let mut aborted = service.clone();
    aborted.abort();
    let ended = aborted.snapshot().unwrap();
    assert!(ended.weapons.is_empty() && ended.weapon_pose.is_none());
    assert!(KartService::restore(content, ended).is_ok());
    assert_eq!(
        service.snapshot().unwrap(),
        snapshot,
        "clone cleanup must not change original state"
    );
}

/// RECONSTRUCTED band for the straight fixture: a 40 m wide lane (±20 m), 6 m deep, 8 m tall.
/// The travel axis is derived from the ordered waypoints (the fixture runs along +x), so the lane
/// opens across z exactly where the 3 m sphere rejects a different driving lane. Content owns these
/// dimensions; the service only validates and uses them.
fn synthetic_band() -> GateBand {
    GateBand {
        half_width: 20.0,
        half_depth: 3.0,
        half_height: 4.0,
        normal: None,
    }
}

fn with_band() -> KartServiceContent {
    let mut content = synthetic_content();
    content.gate_band = Some(synthetic_band());
    content
}

/// One authored curtain leaf: a vertical quad at `station` (+x), spanning `lane +/- 20 m` across
/// z and `0..8 m` up, with the recovered 3 m half-thickness. Ring order is the CDTA reader's.
fn synthetic_curtain(station: f32, lane: f32) -> GateCurtain {
    GateCurtain {
        vertices: [
            [station, 0.0, lane - 20.0],
            [station, 0.0, lane + 20.0],
            [station, 8.0, lane + 20.0],
            [station, 8.0, lane - 20.0],
        ],
        half_depth: GATE_CURTAIN_HALF_DEPTH_M,
    }
}

/// Curtain content for the straight fixture: one leaf per gate in the reconstructed lane.
fn with_curtains() -> KartServiceContent {
    let mut content = synthetic_content();
    content.gate_curtains = (1..=4)
        .map(|index| vec![synthetic_curtain(index as f32 * 10.0, 0.0)])
        .collect();
    content
}

fn synthetic_context() -> RaceContext {
    RaceContext {
        session: 0x4b52_cc6a_9573_ca91,
        // Deliberately different from the session and from the captured guid: the service must use
        // the real player's guid and never silently reuse the captured one.
        player_guid: 0x1122_3344_5566_7788,
        instance_id: 1008,
        encounter_id: 917_566,
    }
}

fn enabled(content: KartServiceContent) -> KartService {
    let mut service = KartService::new(content).expect("valid synthetic content");
    service.set_gate(EntryGate::Enabled);
    service
}

fn started() -> (KartService, RaceContext) {
    let context = synthetic_context();
    let mut service = enabled(synthetic_content());
    service.begin(context, BEGIN_MS).expect("begin");
    service.tick(at(14_505)).expect("schedule");
    assert_eq!(service.phase(), RacePhase::Racing);
    (service, context)
}

/// Same as [`started`] for arbitrary content.
fn racing(content: KartServiceContent) -> (KartService, RaceContext) {
    let context = synthetic_context();
    let mut service = enabled(content);
    service.begin(context, BEGIN_MS).expect("begin");
    service.tick(at(14_505)).expect("schedule");
    assert_eq!(service.phase(), RacePhase::Racing);
    (service, context)
}

#[test]
fn early_go_rebases_countdown_without_clock_underflow_and_restores() {
    let content = synthetic_content();
    let mut service = enabled(content.clone());
    service.prepare(synthetic_context(), 0).unwrap();
    service.start_countdown(100).unwrap();
    let started = service.tick(100).unwrap();
    assert_eq!(
        started.replies(),
        &[inner(GAME_STATE_SUBTYPE, &7u32.to_le_bytes())]
    );
    let snapshot = service.snapshot().unwrap();
    assert_eq!(snapshot.countdown_started_ms, 100);
    assert_eq!(snapshot.countdown_offset_ms, 8_694);
    let mut restored = KartService::restore(content.clone(), snapshot.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert!(restored.tick(100).unwrap().replies().is_empty());
    assert_eq!(restored.phase(), RacePhase::Countdown);
    restored.tick(100 + 13_241 - 8_694).unwrap();
    assert_eq!(restored.phase(), RacePhase::Racing);
    assert_eq!(restored.snapshot().unwrap().race_started_ms, 4_647);
    let mut corrupt = snapshot;
    corrupt.countdown_offset_ms += 1;
    assert!(matches!(
        KartService::restore(content, corrupt),
        Err(ServiceError::CorruptSnapshot {
            field: "countdown_offset_ms"
        })
    ));
    let mut empty = synthetic_content();
    empty.schedule.clear();
    let mut service = enabled(empty);
    service.prepare(synthetic_context(), 0).unwrap();
    service.start_countdown(1).unwrap();
    assert!(service.tick(1).unwrap().replies().is_empty());
}

#[test]
fn native_boost_award_is_owned_persistent_and_queue_bounded() {
    let (mut service, context) = started();
    let request = inner(22, &context.player_guid.to_le_bytes());
    let start = service.snapshot().unwrap().race_started_ms;
    let before = service.snapshot().unwrap();
    let foreign = inner(22, &(context.player_guid + 1).to_le_bytes());
    assert!(matches!(
        service.handle_inner(&foreign, start + 10_603).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::ForeignGuid { .. })
    ));
    assert_eq!(service.snapshot().unwrap(), before);
    // The v13 capture's first request arrives 10.603 s after Go, and v11/v12 legitimately range to
    // minutes: the client's config-driven accumulator paces refills, so no fixed interval gates it.
    let outcome = service.handle_inner(&request, start + 10_603).unwrap();
    assert_eq!(replies(&outcome), &[inner(23, &[])]);
    assert!(!outcome.reward_ready());
    let awarded = service.snapshot().unwrap();
    assert_eq!(awarded.queued_items, vec![BOOST_ITEM_TYPE]);
    assert_eq!(
        awarded.session, before.session,
        "a refill never moves the session"
    );
    assert!(
        service
            .handle_inner(&inner(22, &[0; 7]), start + 12_000)
            .is_err()
    );
    assert!(
        service
            .handle_inner(&inner(22, &[0; 9]), start + 12_000)
            .is_err()
    );
    let mut corrupt = awarded.clone();
    corrupt.last_boost_epoch = 31;
    assert!(matches!(
        KartService::restore(synthetic_content(), corrupt),
        Err(ServiceError::CorruptSnapshot {
            field: "last_boost_epoch"
        })
    ));
    let mut legacy = serde_json::to_value(&before).unwrap();
    legacy.as_object_mut().unwrap().remove("last_boost_epoch");
    let legacy: ServiceSnapshot = serde_json::from_value(legacy).unwrap();
    assert_eq!(
        KartService::restore(synthetic_content(), legacy)
            .unwrap()
            .snapshot()
            .unwrap(),
        before
    );
    // The granted entry persists across restore; the three-slot queue still bounds the refills.
    let mut restored = KartService::restore(synthetic_content(), awarded.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(
        restored.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE]
    );
    assert_eq!(
        replies(&restored.handle_inner(&request, start + 11_000).unwrap()),
        &[inner(23, &[])]
    );
    assert_eq!(
        replies(&restored.handle_inner(&request, start + 11_001).unwrap()),
        &[inner(23, &[])]
    );
    assert_eq!(
        restored.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE; ITEM_QUEUE_CAP]
    );
    let full = restored.snapshot().unwrap();
    assert_eq!(
        refusal(&restored.handle_inner(&request, start + 11_002).unwrap()),
        &ServiceRefusal::ItemQueueFull
    );
    assert_eq!(restored.snapshot().unwrap(), full);
    assert!(matches!(
        restored.handle_inner(&request, start + 600_001).unwrap(),
        ServiceOutcome::Refused(_)
    ));
    assert_eq!(restored.snapshot().unwrap(), full);
    restored.abort();
    assert!(matches!(
        restored.handle_inner(&request, start + 60_000).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::InvalidPhase { .. })
    ));
    let mut preparing = enabled(synthetic_content());
    preparing.prepare(context, BEGIN_MS).unwrap();
    assert!(matches!(
        preparing.handle_inner(&request, at(60_000)).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::InvalidPhase { .. })
    ));
}

/// A frame whose clock predates Go must not refill, use or collect: `saturating_sub` alone would
/// read it as elapsed zero. The refusal is the movement guard's `ClaimRate { delta_ms: 0 }`, and it
/// leaves the queue, the legacy epoch, the anchor and the pickups untouched - fresh and restored.
#[test]
fn before_go_clock_cannot_refill_use_or_collect() {
    let (mut service, context) = started();
    let start = service.snapshot().unwrap().race_started_ms;
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [0.0, 0.0, 0.0]),
    );
    let before = service.snapshot().unwrap();
    for frame in [&award, &use_item] {
        assert_eq!(
            refusal(&service.handle_inner(frame, start - 1).unwrap()),
            &ServiceRefusal::ClaimRate { delta_ms: 0 }
        );
    }
    assert_eq!(
        service.snapshot().unwrap(),
        before,
        "a pre-Go clock moves nothing"
    );
    assert_eq!(
        refusal(&service.handle_inner(&claim, start - 1).unwrap()),
        &ServiceRefusal::ClaimRate { delta_ms: 0 }
    );
    let after_claim = service.snapshot().unwrap();
    assert_eq!(after_claim.observations, before.observations + 1);
    assert_eq!(after_claim.queued_items, before.queued_items);
    assert_eq!(after_claim.last_boost_epoch, before.last_boost_epoch);
    assert_eq!(after_claim.last_position, before.last_position);
    assert_eq!(after_claim.last_claim_ms, before.last_claim_ms);
    assert_eq!(after_claim.pickups_consumed, before.pickups_consumed);
    assert_eq!(after_claim.session, before.session);
    // The exact Go boundary is in-window: the refill works and its entry is immediately usable.
    assert_eq!(
        replies(&service.handle_inner(&award, start).unwrap()),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
    assert_eq!(
        replies(&service.handle_inner(&use_item, start).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    // A restored race keeps the same lower edge.
    let snapshot = service.snapshot().unwrap();
    let mut restored = KartService::restore(synthetic_content(), snapshot.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(
        refusal(&restored.handle_inner(&award, start - 1).unwrap()),
        &ServiceRefusal::ClaimRate { delta_ms: 0 }
    );
    assert_eq!(restored.snapshot().unwrap(), snapshot);
}

/// The v13 native capture timing: Go at 265.585 s, `33/22` at +10.603 s, `33/19` at +22.447 s. The
/// refill must be accepted at that cadence, its granted entry must be usable, and the request's own
/// replay (no id exists) must be bounded by the recovered three-slot queue alone.
#[test]
fn native_v13_refill_timing_is_accepted_and_queue_bounded() {
    let (mut service, context) = started();
    let start = service.snapshot().unwrap().race_started_ms;
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        replies(&service.handle_inner(&award, start + 10_603).unwrap()),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
    assert_eq!(
        service.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE]
    );
    assert_eq!(
        replies(&service.handle_inner(&use_item, start + 22_447).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert!(service.snapshot().unwrap().queued_items.is_empty());
    assert_eq!(
        refusal(&service.handle_inner(&use_item, start + 22_500).unwrap()),
        &ServiceRefusal::NoItemQueued
    );
    // Replayed refills are admitted only while an entry is free; the fourth is refused.
    for offset in [22_600, 22_601, 22_602] {
        assert_eq!(
            replies(&service.handle_inner(&award, start + offset).unwrap()),
            &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
        );
    }
    assert_eq!(
        refusal(&service.handle_inner(&award, start + 22_603).unwrap()),
        &ServiceRefusal::ItemQueueFull
    );
    // A restored race keeps the same bounded rule.
    let snapshot = service.snapshot().unwrap();
    let mut restored = KartService::restore(synthetic_content(), snapshot).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(
        restored.snapshot().unwrap().queued_items.len(),
        ITEM_QUEUE_CAP
    );
    assert_eq!(
        refusal(&restored.handle_inner(&award, start + 23_000).unwrap()),
        &ServiceRefusal::ItemQueueFull
    );
    assert_eq!(
        replies(&restored.handle_inner(&use_item, start + 23_100).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert_eq!(
        replies(&restored.handle_inner(&award, start + 23_200).unwrap()),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
}

/// The real v12 client `33/19` `VehicleRacePacketUseItemRequest`, captured 17 times
/// byte-identically in `client-baseline-20261010T005751Z/applications.json`. The 7-byte application
/// envelope is stripped by the gateway, so the first seven bytes are dropped below; the sha256
/// pins the complete captured frame.
const CAPTURED_USE_ITEM_FRAME: &str = "0500010c000000210013000100000000000000";
const CAPTURED_USE_ITEM_SHA256: &str =
    "f8e9ceb075cdb65c4bfb883845540e0044cb961feb3879de9bc4fe31f8c12cec";
/// The real v12 client `33/22` `VehicleRacePacketBoostAwardRequest` (three identical frames).
const CAPTURED_BOOST_AWARD_FRAME: &str = "0500010c000000210016000100000000000000";

/// Decodes a captured application frame and drops the 7-byte envelope the gateway strips.
fn captured_inner(hex: &str) -> Vec<u8> {
    let frame: Vec<u8> = (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).unwrap())
        .collect();
    frame[7..].to_vec()
}

/// The v12 capture's character guid is 1 (`kart-playable-v12-20261009/identity.json`), which is why
/// every captured client frame carries the id pair `01 00 00 00 00 00 00 00`.
fn captured_context() -> RaceContext {
    RaceContext {
        session: 1,
        player_guid: 1,
        instance_id: 1008,
        encounter_id: 917_566,
    }
}

fn captured_racing() -> (KartService, RaceContext) {
    let context = captured_context();
    let mut service = enabled(synthetic_content());
    service.begin(context, BEGIN_MS).expect("begin");
    service.tick(at(14_505)).expect("schedule");
    assert_eq!(service.phase(), RacePhase::Racing);
    (service, context)
}

#[test]
fn native_use_item_request_consumes_one_queued_boost_and_replies_type_five() {
    use sha2::{Digest, Sha256};

    let frame: Vec<u8> = (0..CAPTURED_USE_ITEM_FRAME.len() / 2)
        .map(|index| {
            u8::from_str_radix(&CAPTURED_USE_ITEM_FRAME[index * 2..index * 2 + 2], 16).unwrap()
        })
        .collect();
    assert_eq!(
        frame.len(),
        19,
        "the captured application frame is 19 bytes"
    );
    assert_eq!(
        Sha256::digest(&frame)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        CAPTURED_USE_ITEM_SHA256
    );
    let request = captured_inner(CAPTURED_USE_ITEM_FRAME);
    assert_eq!(request, inner(USE_ITEM_SUBTYPE, &1u64.to_le_bytes()));

    let (mut service, _context) = captured_racing();
    let start = service.snapshot().unwrap().race_started_ms;
    // Empty mirror queue: the client asked, the server has nothing to name.
    let before = service.snapshot().unwrap();
    assert_eq!(
        refusal(&service.handle_inner(&request, start + 15_000).unwrap()),
        &ServiceRefusal::NoItemQueued
    );
    assert_eq!(service.snapshot().unwrap(), before);
    // The award path fills one slot: the client's own `33/23` case pushes type 5.
    let award = captured_inner(CAPTURED_BOOST_AWARD_FRAME);
    assert_eq!(
        replies(&service.handle_inner(&award, start + 20_000).unwrap()),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
    assert_eq!(
        service.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE]
    );
    let session = service.snapshot().unwrap().session;
    let outcome = service.handle_inner(&request, start + 20_100).unwrap();
    assert_eq!(
        replies(&outcome),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert!(!outcome.reward_ready());
    let after = service.snapshot().unwrap();
    assert!(
        after.queued_items.is_empty(),
        "the accepted use consumed the entry"
    );
    assert_eq!(after.session, session, "a use never advances gates");
    // The same bytes again: the single entry is spent, so the frame is indistinguishable from a
    // re-send and the empty queue is the bound that refuses it.
    assert_eq!(
        refusal(&service.handle_inner(&request, start + 20_200).unwrap()),
        &ServiceRefusal::NoItemQueued
    );
    assert_eq!(service.snapshot().unwrap(), after);
    // A second use needs a fresh award on a later epoch.
    assert_eq!(
        replies(&service.handle_inner(&award, start + 40_000).unwrap()),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
    assert_eq!(
        replies(&service.handle_inner(&request, start + 40_100).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert!(service.snapshot().unwrap().queued_items.is_empty());
    assert!(!service.finish_flag());
}

#[test]
fn use_item_refusals_are_typed_and_move_nothing() {
    let (mut service, context) = started();
    let request = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    let before = service.snapshot().unwrap();
    for width in [7usize, 9] {
        assert_eq!(
            service
                .handle_inner(&inner(USE_ITEM_SUBTYPE, &vec![0u8; width]), at(15_000))
                .unwrap_err(),
            ServiceError::PayloadLength {
                subtype: USE_ITEM_SUBTYPE,
                expected: OWNED_ID_PAYLOAD_LEN,
                actual: width,
            }
        );
    }
    assert_eq!(
        refusal(
            &service
                .handle_inner(
                    &inner(USE_ITEM_SUBTYPE, &(context.player_guid + 1).to_le_bytes()),
                    at(15_000),
                )
                .unwrap()
        ),
        &ServiceRefusal::ForeignGuid {
            offered: context.player_guid + 1,
            owned: context.player_guid,
        }
    );
    assert_eq!(
        refusal(&service.handle_inner(&request, at(15_000)).unwrap()),
        &ServiceRefusal::NoItemQueued
    );
    assert_eq!(service.snapshot().unwrap(), before);
    // Pre-race phases refuse a use without touching the queue.
    let mut preparing = enabled(synthetic_content());
    preparing.prepare(context, BEGIN_MS).unwrap();
    assert!(matches!(
        preparing.handle_inner(&request, at(60_000)).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::InvalidPhase { .. })
    ));
    let mut aborted = enabled(synthetic_content());
    aborted.begin(context, BEGIN_MS).unwrap();
    aborted.tick(at(14_505)).unwrap();
    aborted.abort();
    assert_eq!(
        refusal(&aborted.handle_inner(&request, at(15_000)).unwrap()),
        &ServiceRefusal::InvalidPhase {
            phase: RacePhase::Aborted
        }
    );
    // The race window still bounds a use.
    let (mut timed, context) = started();
    let start = timed.snapshot().unwrap().race_started_ms;
    let before = timed.snapshot().unwrap();
    assert!(matches!(
        timed
            .handle_inner(
                &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
                start + 600_001,
            )
            .unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::RaceTimeout { .. })
    ));
    assert_eq!(timed.snapshot().unwrap(), before);
    // Server-only subtypes stay unsupported on the client->server path.
    assert_eq!(
        refusal(
            &service
                .handle_inner(&inner(21, &0u32.to_le_bytes()), at(15_100))
                .unwrap()
        ),
        &ServiceRefusal::UnsupportedSubtype { subtype: 21 }
    );
}

#[test]
fn mirrored_item_queue_is_capped_at_three_and_a_full_refusal_costs_nothing() {
    let (mut service, context) = started();
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    let start = service.snapshot().unwrap().race_started_ms;
    for epoch in 1..=3u64 {
        assert_eq!(
            replies(
                &service
                    .handle_inner(&award, start + epoch * BOOST_INTERVAL_MS)
                    .unwrap()
            ),
            &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
        );
    }
    assert_eq!(
        service.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE; ITEM_QUEUE_CAP]
    );
    let full = service.snapshot().unwrap();
    assert_eq!(
        refusal(
            &service
                .handle_inner(&award, start + 4 * BOOST_INTERVAL_MS)
                .unwrap()
        ),
        &ServiceRefusal::ItemQueueFull
    );
    assert_eq!(
        service.snapshot().unwrap(),
        full,
        "a full refusal must not consume the epoch"
    );
    // Draining one slot frees the same epoch, so the refused award can be retried unchanged.
    assert_eq!(
        replies(
            &service
                .handle_inner(&use_item, start + 4 * BOOST_INTERVAL_MS)
                .unwrap()
        ),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert_eq!(
        replies(
            &service
                .handle_inner(&award, start + 4 * BOOST_INTERVAL_MS)
                .unwrap()
        ),
        &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
    );
    assert_eq!(
        service.snapshot().unwrap().queued_items.len(),
        ITEM_QUEUE_CAP
    );
    // No time floor exists any more: the queue cap is the whole bound, so the next refill request
    // is refused because the mirror is full, not because of a clock.
    assert_eq!(
        refusal(
            &service
                .handle_inner(&award, start + 4 * BOOST_INTERVAL_MS + 1)
                .unwrap()
        ),
        &ServiceRefusal::ItemQueueFull
    );
}

#[test]
fn legacy_snapshot_restores_an_empty_queue_and_a_doctored_queue_is_refused() {
    let (service, _) = started();
    let snapshot = service.snapshot().unwrap();
    let mut legacy = serde_json::to_value(&snapshot).unwrap();
    assert!(
        legacy
            .as_object_mut()
            .unwrap()
            .remove("queued_items")
            .is_some()
    );
    let legacy: ServiceSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(
        legacy.queued_items.is_empty(),
        "a legacy save grants nothing"
    );
    let restored = KartService::restore(synthetic_content(), legacy).unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    // A queue the service really produced round-trips and can still pay for exactly one use.
    let (mut awarded, context) = started();
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    let start = awarded.snapshot().unwrap().race_started_ms;
    for epoch in 1..=2u64 {
        awarded
            .handle_inner(&award, start + epoch * BOOST_INTERVAL_MS)
            .unwrap();
    }
    let queued = awarded.snapshot().unwrap();
    assert_eq!(queued.queued_items, vec![BOOST_ITEM_TYPE; 2]);
    let mut restored = KartService::restore(synthetic_content(), queued).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(
        replies(
            &restored
                .handle_inner(
                    &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
                    start + 40_100,
                )
                .unwrap()
        ),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert_eq!(
        restored.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE]
    );
    // Structural bounds only: an over-cap queue and unrecovered types are refused by shape, never
    // resumed into a grant.
    for queued in [vec![5, 5, 5, 5], vec![2], vec![9], vec![0]] {
        let mut corrupt = snapshot.clone();
        corrupt.queued_items = queued.clone();
        assert!(
            matches!(
                KartService::restore(synthetic_content(), corrupt),
                Err(ServiceError::CorruptSnapshot {
                    field: "queued_items"
                })
            ),
            "{queued:?} must be refused"
        );
    }
}

#[test]
fn captured_seventeen_use_requests_only_consume_awarded_items() {
    let request = captured_inner(CAPTURED_USE_ITEM_FRAME);
    let award = captured_inner(CAPTURED_BOOST_AWARD_FRAME);
    let (mut service, _) = captured_racing();
    let start = service.snapshot().unwrap().race_started_ms;
    for epoch in 1..=3u64 {
        assert_eq!(
            replies(
                &service
                    .handle_inner(&award, start + epoch * BOOST_INTERVAL_MS)
                    .unwrap()
            ),
            &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
        );
    }
    let session = service.snapshot().unwrap().session;
    let (mut accepted, mut refused) = (0usize, 0usize);
    for index in 0..17u64 {
        let now = start + 60_000 + index * 1_000;
        let before = service.snapshot().unwrap();
        let outcome = service.handle_inner(&request, now).unwrap();
        match &outcome {
            ServiceOutcome::Handled { .. } => {
                accepted += 1;
                assert_eq!(
                    replies(&outcome),
                    &[inner(
                        USE_ITEM_RESPONSE_SUBTYPE,
                        &BOOST_ITEM_TYPE.to_le_bytes()
                    )]
                );
                assert!(!outcome.reward_ready());
            }
            ServiceOutcome::Refused(refusal) => {
                refused += 1;
                assert_eq!(refusal, &ServiceRefusal::NoItemQueued);
                assert_eq!(
                    service.snapshot().unwrap(),
                    before,
                    "a refused use must move nothing"
                );
            }
        }
    }
    assert_eq!((accepted, refused), (ITEM_QUEUE_CAP, 17 - ITEM_QUEUE_CAP));
    let after = service.snapshot().unwrap();
    assert!(after.queued_items.is_empty());
    assert_eq!(
        after.session, session,
        "seventeen uses never advance a gate"
    );
    assert_eq!(service.phase(), RacePhase::Racing);
    assert!(!service.finish_flag());
}

/// Without pickup content a `33/17` claim is a bounded observation: it spends the observation
/// budget, refuses `NoPickupInRange` and moves nothing else. Foreign, malformed and over-budget
/// claims refuse without touching the queue or the session.
#[test]
fn pickup_claims_without_content_refuse_and_move_nothing() {
    let (mut service, context) = started();
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [1.0, 2.0, 3.0]),
    );
    let before = service.snapshot().unwrap();
    let outcome = service.handle_inner(&claim, at(15_000)).unwrap();
    assert!(outcome.replies().is_empty(), "no content, no grant");
    assert_eq!(refusal(&outcome), &ServiceRefusal::NoPickupInRange);
    assert!(!outcome.reward_ready());
    let after = service.snapshot().unwrap();
    assert_eq!(after.observations, before.observations + 1);
    assert!(
        after.queued_items.is_empty(),
        "no item is granted by a claim"
    );
    assert_eq!(after.session, before.session);
    assert_eq!(after.attempts, before.attempts);
    assert!(matches!(
        service
            .handle_inner(
                &inner(PICKUP_HIT_SUBTYPE, &gate_hit_payload(9, [1.0, 2.0, 3.0])),
                at(15_100),
            )
            .unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::ForeignGuid { .. })
    ));
    assert_eq!(
        service
            .handle_inner(&inner(PICKUP_HIT_SUBTYPE, &[0u8; 3]), at(15_200))
            .unwrap_err(),
        ServiceError::PayloadLength {
            subtype: PICKUP_HIT_SUBTYPE,
            expected: GATE_HIT_PAYLOAD_LEN,
            actual: 3,
        }
    );
    // The observation budget still stops a flood, and the refused observation changes nothing.
    let mut content = synthetic_content();
    content.bounds.max_observations = 1;
    let (mut limited, context) = racing(content);
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [1.0, 2.0, 3.0]),
    );
    assert_eq!(
        refusal(&limited.handle_inner(&claim, at(15_000)).unwrap()),
        &ServiceRefusal::NoPickupInRange
    );
    let before = limited.snapshot().unwrap();
    assert_eq!(
        refusal(&limited.handle_inner(&claim, at(15_100)).unwrap()),
        &ServiceRefusal::ObservationBudget { observations: 1 }
    );
    assert_eq!(limited.snapshot().unwrap(), before);
}

#[test]
fn preparation_loads_vehicle_but_waits_for_go_and_survives_restore() {
    let content = synthetic_content();
    let mut service = enabled(content.clone());
    service.prepare(synthetic_context(), BEGIN_MS).unwrap();
    let setup = service.tick(at(120_000)).unwrap();
    assert_eq!(service.phase(), RacePhase::Preparing);
    assert_eq!(setup.replies().len(), 4);
    assert!(!setup.replies().iter().any(|b| b[..4] == [33, 0, 10, 0]));
    let snapshot = service.snapshot().unwrap();
    let mut restored = KartService::restore(content, snapshot.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert!(restored.tick(at(120_100)).unwrap().replies().is_empty());
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    restored.start_countdown(at(130_000)).unwrap();
    let go = restored.tick(at(130_000)).unwrap();
    assert_eq!(
        go.replies(),
        &[inner(GAME_STATE_SUBTYPE, &7i32.to_le_bytes())]
    );
    let begun = restored.snapshot().unwrap();
    assert!(restored.start_countdown(at(131_000)).is_err());
    assert_eq!(restored.snapshot().unwrap(), begun);
    restored.tick(at(135_000)).unwrap();
    assert_eq!(restored.phase(), RacePhase::Racing);
}

fn gate_hit_payload(guid: u64, position: [f32; 3]) -> Vec<u8> {
    let mut payload = guid.to_le_bytes().to_vec();
    for value in position {
        payload.extend(value.to_le_bytes());
    }
    payload.extend(1.0f32.to_le_bytes());
    payload
}

fn countdown_payload(guid: u64, value: u32) -> Vec<u8> {
    let mut payload = guid.to_le_bytes().to_vec();
    payload.extend(value.to_le_bytes());
    payload
}

fn inner(subtype: u16, payload: &[u8]) -> Vec<u8> {
    let mut frame = 33u16.to_le_bytes().to_vec();
    frame.extend(subtype.to_le_bytes());
    frame.extend(payload);
    frame
}

fn replies(outcome: &ServiceOutcome) -> &[Vec<u8>] {
    match outcome {
        ServiceOutcome::Handled { replies, .. } => replies,
        ServiceOutcome::Refused(refusal) => {
            panic!("expected handled, got refusal {refusal:?}")
        }
    }
}

fn refusal(outcome: &ServiceOutcome) -> &ServiceRefusal {
    match outcome {
        ServiceOutcome::Refused(refusal) => refusal,
        ServiceOutcome::Handled { replies, .. } => {
            panic!("expected a refusal, got {} replies", replies.len())
        }
    }
}

fn subtypes(frames: &[Vec<u8>]) -> Vec<u16> {
    frames
        .iter()
        .map(|frame| u16::from_le_bytes([frame[2], frame[3]]))
        .collect()
}

fn u32_payload(frame: &[u8]) -> u32 {
    u32::from_le_bytes(frame[4..8].try_into().expect("4-byte payload"))
}

fn gate_claim(
    service: &mut KartService,
    context: &RaceContext,
    index: f32,
    now_ms: u64,
) -> ServiceOutcome {
    service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [index * 10.0, 0.0, 0.0]),
            ),
            now_ms,
        )
        .expect("well-formed claim")
}

/// One `33/7` claim at an arbitrary world position.
fn claim(
    service: &mut KartService,
    context: &RaceContext,
    position: [f32; 3],
    now_ms: u64,
) -> ServiceOutcome {
    service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, position),
            ),
            now_ms,
        )
        .expect("well-formed claim")
}

/// Drives one full lap in order: gates 1..=4, one second apart.
fn drive_lap(service: &mut KartService, context: &RaceContext, start_offset_ms: u64) {
    for index in 1..=4u64 {
        let outcome = gate_claim(
            service,
            context,
            index as f32,
            at(start_offset_ms + index * 1_000),
        );
        assert!(
            matches!(outcome, ServiceOutcome::Handled { .. }),
            "gate {index} must be accepted, got {outcome:?}"
        );
    }
}

#[test]
fn gate_defaults_off_and_nothing_moves() {
    let context = synthetic_context();
    let mut service = KartService::new(synthetic_content()).unwrap();
    assert_eq!(service.gate(), EntryGate::Disabled);
    service.begin(context.clone(), BEGIN_MS).unwrap();
    let frame = inner(
        GATE_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [10.0, 0.0, 0.0]),
    );
    assert_eq!(
        refusal(&service.handle_inner(&frame, at(15_000)).unwrap()),
        &ServiceRefusal::GateDisabled
    );
    assert_eq!(
        refusal(&service.tick(at(15_000)).unwrap()),
        &ServiceRefusal::GateDisabled
    );
    assert_eq!(service.phase(), RacePhase::Countdown);
    assert_eq!(service.entry_session(), Some(context.session));
    assert!(service.snapshot().is_some());
    assert!(
        KartService::new(synthetic_content())
            .unwrap()
            .snapshot()
            .is_none()
    );
}

#[test]
fn absolute_clock_binds_schedule_to_begin_time() {
    let mut service = enabled(synthetic_content());
    service.begin(synthetic_context(), BEGIN_MS).unwrap();
    // Nothing fires before its offset, even though the absolute clock is already at 100 s.
    assert!(replies(&service.tick(at(3_025)).unwrap()).is_empty());
    assert_eq!(service.phase(), RacePhase::Countdown);
    let due = service.tick(at(3_026)).unwrap();
    assert_eq!(
        subtypes(replies(&due)),
        vec![GAME_STATE_SUBTYPE, NPC_COUNT_SUBTYPE]
    );
    // The race starts at the countdown-0 instant, not at begin and not at tick time.
    let before_start = service.tick(at(13_240)).unwrap();
    assert_eq!(
        u16::from_le_bytes([
            replies(&before_start).last().expect("countdown 1")[2],
            replies(&before_start).last().unwrap()[3],
        ]),
        COUNTDOWN_SUBTYPE,
        "the last frame before the start is still a countdown"
    );
    assert_eq!(
        u32::from_le_bytes(
            replies(&before_start).last().unwrap()[12..16]
                .try_into()
                .unwrap()
        ),
        1
    );
    assert_eq!(service.phase(), RacePhase::Countdown);
    service.tick(at(13_241)).unwrap();
    assert_eq!(service.phase(), RacePhase::Racing);
    let started_at = service.snapshot().unwrap().race_started_ms;
    assert_eq!(started_at, at(13_241));
}

#[test]
fn schedule_emits_the_recovered_order_and_starts_the_race() {
    let context = synthetic_context();
    let mut service = enabled(synthetic_content());
    service.begin(context.clone(), BEGIN_MS).unwrap();
    assert_eq!(service.phase(), RacePhase::Countdown);

    let mut order = Vec::new();
    let mut countdown_sent = Vec::new();
    for step in recovered_schedule() {
        let outcome = service.tick(at(step.at_ms)).unwrap();
        for frame in replies(&outcome) {
            let subtype = u16::from_le_bytes([frame[2], frame[3]]);
            order.push(match subtype {
                GAME_STATE_SUBTYPE => format!("state({})", u32_payload(frame) as i32),
                PLAYER_COUNT_SUBTYPE => format!("players({})", u32_payload(frame)),
                NPC_COUNT_SUBTYPE => format!("npcs({})", u32_payload(frame)),
                COUNTDOWN_SUBTYPE => {
                    let guid = u64::from_le_bytes(frame[4..12].try_into().unwrap());
                    let value = u32::from_le_bytes(frame[12..16].try_into().unwrap());
                    assert_eq!(guid, context.player_guid, "countdown carries the real guid");
                    countdown_sent.push(value);
                    format!("countdown({value})")
                }
                other => panic!("unexpected subtype {other}"),
            });
        }
    }
    assert_eq!(
        order,
        vec![
            "state(4)",
            "npcs(7)",
            "state(5)",
            "state(6)",
            "state(7)",
            "state(8)",
            "countdown(4)",
            "countdown(3)",
            "countdown(2)",
            "countdown(1)",
            "countdown(0)",
            "state(9)",
        ]
    );
    assert_eq!(countdown_sent, vec![4, 3, 2, 1, 0]);
    assert_eq!(
        service.phase(),
        RacePhase::Racing,
        "countdown 0 starts the race"
    );
    assert!(
        replies(&service.tick(at(14_505)).unwrap()).is_empty(),
        "no step is re-emitted"
    );
}

#[test]
fn legitimate_ordered_race_finishes_and_offers_the_reward_once() {
    let (mut service, context) = started();
    drive_lap(&mut service, &context, 15_000);
    assert_eq!(service.phase(), RacePhase::Racing);
    assert!(!service.finish_flag());
    assert!(
        service.take_finished_session().is_none(),
        "no reward before the finish"
    );
    drive_lap(&mut service, &context, 20_000);
    assert_eq!(service.phase(), RacePhase::Finished);
    assert_eq!(service.session().unwrap().laps_completed(), LAPS);
    assert!(service.finish_flag());
    assert!(service.reward_priced());

    let session = service.take_finished_session().expect("finished session");
    assert_eq!(session.phase(), Phase::Finished);
    let claimed = session.claim_reward().expect("priced finish");
    assert!(matches!(
        claimed,
        kart_racing::RewardProposal::Granted { coins: 40, xp: 60 }
    ));
    assert!(
        service.take_finished_session().is_none(),
        "the offer is one shot"
    );
    assert!(!service.finish_flag());
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(30_000))),
        &ServiceRefusal::InvalidPhase {
            phase: RacePhase::Finished
        }
    );
    // Every accepted gate reported the local racer's progress block.
    assert_eq!(service.observations(), 0);
}

#[test]
fn refusals_move_nothing_but_the_attempt_budget() {
    let (mut service, context) = started();
    let before = service.snapshot().expect("snapshot");
    let foreign = inner(
        GATE_HIT_SUBTYPE,
        &gate_hit_payload(0xdead_beef_dead_beef, [10.0, 0.0, 0.0]),
    );
    assert!(matches!(
        service.handle_inner(&foreign, at(15_000)).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::ForeignGuid { .. })
    ));
    // A skipped gate and a non-finite claim both refuse; only the bounded attempt counter advances.
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 3.0, at(15_100))),
        &ServiceRefusal::SkippedGate {
            expected: 1,
            got: 3
        }
    );
    let nan = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [f32::NAN, 0.0, 0.0]),
            ),
            at(15_200),
        )
        .unwrap();
    assert_eq!(refusal(&nan), &ServiceRefusal::NonFinitePosition);
    let mut expected = before.clone();
    expected.attempts += 2;
    assert_eq!(
        service.snapshot().expect("snapshot"),
        expected,
        "only the bounded attempt counter may move on a refusal"
    );
}

#[test]
fn foreign_guid_session_and_countdown_are_refused() {
    let (mut service, context) = started();
    let foreign = 0x0bad_f00d_dead_beefu64;
    assert_eq!(
        refusal(&gate_claim(
            &mut service,
            &RaceContext {
                player_guid: foreign,
                ..context
            },
            1.0,
            at(15_000)
        )),
        &ServiceRefusal::ForeignGuid {
            offered: foreign,
            owned: context.player_guid,
        }
    );
    assert_eq!(
        refusal(
            &service
                .handle_inner(
                    &inner(COUNTDOWN_SUBTYPE, &countdown_payload(foreign, 3)),
                    at(15_000)
                )
                .unwrap()
        ),
        &ServiceRefusal::ForeignGuid {
            offered: foreign,
            owned: context.player_guid,
        }
    );
    assert_eq!(
        refusal(
            &service
                .handle_inner(
                    &inner(
                        COUNTDOWN_SUBTYPE,
                        &countdown_payload(context.player_guid, 9)
                    ),
                    at(15_000)
                )
                .unwrap()
        ),
        &ServiceRefusal::CountdownSpoof { value: 9, sent: 0 }
    );
    // An in-bound join (33/2) belongs to the entry handler, never to the race service.
    let mut join = context.session.to_le_bytes().to_vec();
    join.extend(2u32.to_le_bytes());
    assert_eq!(
        refusal(&service.handle_inner(&inner(2, &join), at(15_000)).unwrap()),
        &ServiceRefusal::UnsupportedSubtype { subtype: 2 }
    );
    assert_eq!(service.phase(), RacePhase::Racing);
    assert_eq!(service.session().unwrap().laps_completed(), 0);
    assert!(!service.finish_flag());
}

#[test]
fn out_of_order_skipped_and_stale_gates_are_refused() {
    let (mut service, context) = started();
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 3.0, at(15_000))),
        &ServiceRefusal::SkippedGate {
            expected: 1,
            got: 3
        }
    );
    assert_eq!(
        service.session().unwrap().next_checkpoint(),
        Some(1),
        "a skipped claim grants nothing"
    );
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(17_000))),
        &ServiceRefusal::StaleGate {
            expected: 2,
            got: 1
        }
    );
    // 55 m along the line is inside no gate volume.
    let outside = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [55.0, 0.0, 0.0]),
            ),
            at(18_000),
        )
        .unwrap();
    assert_eq!(refusal(&outside), &ServiceRefusal::OutsideGates);
    assert_eq!(service.session().unwrap().laps_completed(), 0);
}

#[test]
fn overlapping_gate_volumes_attribute_to_the_expected_gate() {
    // Overlapping physical volumes must not let a skipped or stale claim advance: the expected gate
    // wins, and anything else is typed.
    let mut content = synthetic_content();
    for gate in &mut content.gates {
        gate.radius = 6.0;
    }
    let context = synthetic_context();
    let mut service = enabled(content);
    service.begin(context.clone(), BEGIN_MS).unwrap();
    service.tick(at(14_505)).unwrap();
    // Gate 2's centre lies outside gate 1's volume while gate 1 is expected -> a skip.
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 2.0, at(15_000))),
        &ServiceRefusal::SkippedGate {
            expected: 1,
            got: 2
        }
    );
    // x = 15 m is inside both gate 1 and gate 2 (gates 10 m apart, radius 6 m): the expected gate 1
    // wins, so the overlap advances exactly one gate.
    let shared = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [15.0, 0.0, 0.0]),
            ),
            at(16_000),
        )
        .unwrap();
    assert!(matches!(shared, ServiceOutcome::Handled { .. }));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
    // The very same overlapping point now advances gate 2, because gate 2 is the expected gate.
    let shared_again = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [15.0, 0.0, 0.0]),
            ),
            at(17_000),
        )
        .unwrap();
    assert!(matches!(shared_again, ServiceOutcome::Handled { .. }));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(3));
    // A point inside gate 1 alone is stale while gate 3 is expected.
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(18_000))),
        &ServiceRefusal::StaleGate {
            expected: 3,
            got: 1
        }
    );
    assert_eq!(service.session().unwrap().laps_completed(), 0);
}

/// The recovered trigger latches on *entering* a type-3 volume and `33/7` reports the vehicle
/// position at that boundary, not a gate centre: a different driving lane is a legitimate hit.
/// The legacy sphere rejects the same lane offset.
#[test]
fn gate_band_accepts_a_wide_lane_claim_that_the_sphere_refuses() {
    // 15 m perpendicular to the +x travel axis of gate 1; the band half width is 20 m.
    let wide = [10.0, 0.0, 15.0];
    let (mut banded, banded_context) = racing(with_band());
    assert!(matches!(
        claim(&mut banded, &banded_context, wide, at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(banded.session().unwrap().next_checkpoint(), Some(2));

    // The same claim against the legacy 3 m sphere is outside every volume.
    let (mut legacy, legacy_context) = started();
    assert_eq!(
        refusal(&claim(&mut legacy, &legacy_context, wide, at(16_000))),
        &ServiceRefusal::OutsideGates
    );
    assert_eq!(
        legacy.session().unwrap().next_checkpoint(),
        Some(1),
        "legacy content keeps refusing the lane offset"
    );
}

/// `gate_band` is opt-in content: absent keeps the spherical volume, and older JSON without the
/// field still loads unchanged.
#[test]
fn gate_band_is_opt_in_and_legacy_content_stays_spherical() {
    assert!(synthetic_content().gate_band.is_none());
    let mut value = serde_json::to_value(synthetic_content()).unwrap();
    value.as_object_mut().unwrap().remove("gate_band");
    let loaded: KartServiceContent = serde_json::from_value(value).unwrap();
    assert!(
        loaded.gate_band.is_none(),
        "content written before the band still loads"
    );
    // The legacy sphere accepts its centre and refuses 3.5 m off axis.
    let (mut service, context) = racing(loaded);
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    let (mut off_axis, off_context) = racing(synthetic_content());
    assert_eq!(
        refusal(&claim(
            &mut off_axis,
            &off_context,
            [10.0, 0.0, 3.5],
            at(16_000)
        )),
        &ServiceRefusal::OutsideGates
    );
}

/// The band is an oriented box, not a bigger sphere: outside every face is refused and the faces
/// themselves are inclusive.
#[test]
fn gate_band_refuses_longitudinal_height_and_width_escapes() {
    // Local frame at gate 1 ([10,0,0]): +x along travel, z across the lane, y up.
    for (escape, why) in [
        (
            [14.0f32, 0.0, 0.0],
            "4 m along travel exceeds the 3 m half depth",
        ),
        ([10.0, 5.0, 0.0], "5 m up exceeds the 4 m half height"),
        ([10.0, 0.0, 21.0], "21 m across exceeds the 20 m half width"),
    ] {
        let (mut service, context) = racing(with_band());
        assert_eq!(
            refusal(&claim(&mut service, &context, escape, at(16_000))),
            &ServiceRefusal::OutsideGates,
            "{why}"
        );
        assert_eq!(
            service.session().unwrap().next_checkpoint(),
            Some(1),
            "{why}: a refused claim grants nothing"
        );
    }
    // On all three faces at once is still inside (inclusive bounds).
    let (mut service, context) = racing(with_band());
    assert!(matches!(
        claim(&mut service, &context, [13.0, 4.0, 20.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
}

/// The ordered-gate authority is unchanged by the volume shape: skips, replays and stale claims
/// stay typed refusals.
#[test]
fn gate_band_still_refuses_skipped_stale_and_replayed_claims() {
    let (mut service, context) = racing(with_band());
    // 10 m along travel is outside gate 1's 3 m half depth, so gate 2 is attributed as a skip.
    assert_eq!(
        refusal(&claim(&mut service, &context, [20.0, 0.0, 0.0], at(16_000))),
        &ServiceRefusal::SkippedGate {
            expected: 1,
            got: 2
        }
    );
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(1));
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 0.0], at(17_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(
        refusal(&claim(&mut service, &context, [10.0, 0.0, 0.0], at(18_000))),
        &ServiceRefusal::StaleGate {
            expected: 2,
            got: 1
        }
    );
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
}

/// Overlapping bands still attribute to the expected gate, so a wide lane can never let a skipped
/// or stale claim advance.
#[test]
fn gate_band_overlap_still_prefers_the_expected_gate() {
    let mut content = synthetic_content();
    content.gate_band = Some(GateBand {
        half_width: 20.0,
        half_depth: 12.0,
        half_height: 4.0,
        normal: None,
    });
    let (mut service, context) = racing(content);
    // Gates are 10 m apart and the half depth is 12 m: [20,0,0] sits in gate 1 and gate 2, and the
    // expected gate wins each time.
    assert!(matches!(
        claim(&mut service, &context, [20.0, 0.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
    assert!(matches!(
        claim(&mut service, &context, [20.0, 0.0, 0.0], at(17_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(3));
}

/// Guid binding, the countdown echo guard and the hard movement cap are unchanged by the band.
/// Movement authority is the bounded credit: a 10 m claim 60 ms after an accepted gate is paid
/// from the credit that leg banked, while a jump past the cap is still refused.
#[test]
fn gate_band_keeps_teleport_guid_and_countdown_guards() {
    let (mut service, context) = racing(with_band());
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    // 10 m in 60 ms implies 166 m/s, but the first leg banked ~80 m of spare, so the flush is paid
    // from savings. The bank, not the wall clock, is what the claim spends.
    assert!(matches!(
        claim(&mut service, &context, [20.0, 0.0, 0.0], at(16_060)),
        ServiceOutcome::Handled { .. }
    ));
    // A jump far past the hard cap (window + bank) is refused by the bound, not by the band.
    assert!(matches!(
        refusal(&claim(
            &mut service,
            &context,
            [1_000.0, 0.0, 0.0],
            at(16_120)
        )),
        ServiceRefusal::ImplausibleSpeed { .. }
    ));
    let foreign = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid ^ 0xff, [10.0, 0.0, 0.0]),
            ),
            at(17_000),
        )
        .unwrap();
    assert!(matches!(
        refusal(&foreign),
        ServiceRefusal::ForeignGuid { .. }
    ));
    let echo = service
        .handle_inner(
            &inner(
                COUNTDOWN_SUBTYPE,
                &countdown_payload(context.player_guid, 1),
            ),
            at(18_000),
        )
        .unwrap();
    assert_eq!(
        refusal(&echo),
        &ServiceRefusal::CountdownSpoof { value: 1, sent: 0 }
    );
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(3));
}

/// The travel axis is reconstructed from the ordered waypoints (`prev -> next`, `prev -> current`
/// for the last gate, `next -> current` for the first) or from one supplied horizontal normal;
/// neighbours that cannot define an axis are refused at construction.
#[test]
fn gate_band_axes_come_from_ordered_neighbours_or_a_supplied_normal() {
    // A single gate has no ordered neighbour, so the band must name the travel direction.
    let mut single = synthetic_content();
    single.gates.truncate(1);
    single.gate_band = Some(synthetic_band());
    assert_eq!(
        KartService::new(single.clone()).unwrap_err(),
        ServiceError::GateBandAxis { index: 0 }
    );
    // A supplied horizontal normal is enough for the same single-gate track...
    single.gate_band = Some(GateBand {
        normal: Some([0.0, 0.0, 1.0]),
        ..synthetic_band()
    });
    let (mut service, context) = racing(single);
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 2.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    // ...but a purely vertical normal has no horizontal travel axis.
    let mut vertical = with_band();
    vertical.gate_band = Some(GateBand {
        normal: Some([0.0, 1.0, 0.0]),
        ..synthetic_band()
    });
    assert_eq!(
        KartService::new(vertical).unwrap_err(),
        ServiceError::GateBand { field: "normal" }
    );
    // Coincident neighbours cannot define an axis either.
    let mut coincident = with_band();
    coincident.gates[1].position = coincident.gates[0].position;
    assert_eq!(
        KartService::new(coincident).unwrap_err(),
        ServiceError::GateBandAxis { index: 0 }
    );
    // A supplied normal must be finite and horizontal.
    let mut infinite = with_band();
    infinite.gate_band = Some(GateBand {
        normal: Some([f32::INFINITY, 0.0, 0.0]),
        ..synthetic_band()
    });
    assert_eq!(
        KartService::new(infinite).unwrap_err(),
        ServiceError::GateBand { field: "normal" }
    );
}

/// Band extents stay bounded and finite; a nonsense band is refused at construction, never clamped.
#[test]
fn gate_band_content_is_bounded() {
    let mut content = with_band();
    content.gate_band = Some(GateBand {
        half_width: 0.0,
        ..synthetic_band()
    });
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateBand {
            field: "half_width"
        }
    );
    let mut content = with_band();
    content.gate_band = Some(GateBand {
        half_depth: f32::NAN,
        ..synthetic_band()
    });
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateBand {
            field: "half_depth"
        }
    );
    let mut content = with_band();
    content.gate_band = Some(GateBand {
        half_height: MAX_GATE_BAND_HALF_M + 1.0,
        ..synthetic_band()
    });
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateBand {
            field: "half_height"
        }
    );
}

/// The snapshot mirrors the band, so a changed band cannot resume the race, while legacy
/// content/snapshot pairs keep round-tripping.
#[test]
fn restore_refuses_a_changed_gate_band() {
    let content = with_band();
    let (service, _) = racing(content.clone());
    let snapshot = service.snapshot().unwrap();
    assert_eq!(snapshot.gate_band, Some(synthetic_band()));
    assert!(KartService::restore(content.clone(), snapshot.clone()).is_ok());

    let mut changed = content.clone();
    changed.gate_band.as_mut().unwrap().half_width += 1.0;
    assert_eq!(
        KartService::restore(changed, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "gate_band" }
    );

    // A snapshot written before the band existed (field absent) must not resume into band content.
    let mut value = serde_json::to_value(&snapshot).unwrap();
    value.as_object_mut().unwrap().remove("gate_band");
    let legacy: ServiceSnapshot = serde_json::from_value(value).unwrap();
    assert!(legacy.gate_band.is_none());
    assert_eq!(
        KartService::restore(content, legacy).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "gate_band" }
    );
    // Legacy content keeps its own binding.
    let (legacy_service, _) = started();
    let legacy_snapshot = legacy_service.snapshot().unwrap();
    assert!(legacy_snapshot.gate_band.is_none());
    assert!(KartService::restore(synthetic_content(), legacy_snapshot).is_ok());
}

/// The authored curtain leaves are the gate volume. An ordered claim on a leaf is accepted; an
/// escape off the plane, out of the lane or below the leaf is refused with the cursor left alone.
#[test]
fn curtains_accept_the_authored_leaf_and_refuse_every_escape() {
    let (mut service, context) = racing(with_curtains());
    assert!(matches!(
        claim(&mut service, &context, [10.0, 4.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));

    for (escape, why) in [
        ([13.5f32, 4.0, 0.0], "3.5 m off the 3 m leaf"),
        ([10.0, 4.0, 25.0], "5 m past the 20 m lane edge"),
        ([10.0, -4.0, 0.0], "4 m below the leaf"),
    ] {
        let (mut escaped, escaped_context) = racing(with_curtains());
        assert_eq!(
            refusal(&claim(&mut escaped, &escaped_context, escape, at(16_000))),
            &ServiceRefusal::OutsideGates,
            "{why} must not be inside any authored leaf"
        );
        assert_eq!(
            escaped.session().unwrap().next_checkpoint(),
            Some(1),
            "{why} must leave the cursor where it was"
        );
    }
}

/// One gate may carry two leaves (the recovered aliases); either leaf is that gate.
#[test]
fn curtains_accept_either_leaf_of_one_gate() {
    let mut content = synthetic_content();
    content.gate_curtains = (1..=4)
        .map(|index| {
            vec![
                synthetic_curtain(index as f32 * 10.0, 30.0),
                synthetic_curtain(index as f32 * 10.0, -30.0),
            ]
        })
        .collect();
    for lane in [30.0f32, -30.0] {
        let (mut service, context) = racing(content.clone());
        assert!(
            matches!(
                claim(&mut service, &context, [10.0, 4.0, lane], at(16_000)),
                ServiceOutcome::Handled { .. }
            ),
            "the leaf at lane {lane} must accept"
        );
        assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
    }
}

/// Curtains shadow the reconstructed band: content carrying both may not fall back to the wide
/// lane when a claim misses the authored leaf.
#[test]
fn curtains_shadow_the_band_instead_of_widening_it() {
    let mut content = with_curtains();
    content.gate_band = Some(GateBand {
        half_width: 100.0,
        half_depth: 100.0,
        half_height: 100.0,
        normal: None,
    });
    let (mut service, context) = racing(content);
    // 5 m past the leaf's lane edge is comfortably inside the 100 m band and still refused.
    assert_eq!(
        refusal(&claim(
            &mut service,
            &context,
            [10.0, 0.0, 25.0],
            at(16_000)
        )),
        &ServiceRefusal::OutsideGates
    );
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(1));
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 0.0], at(17_000)),
        ServiceOutcome::Handled { .. }
    ));
}

/// Curtain content is validated: one group per gate, one or two leaves per group, and every leaf
/// finite, non-degenerate, vertical and bounded.
#[test]
fn curtain_content_is_bounded_and_well_formed() {
    let cases: Vec<(KartServiceContent, ServiceError)> = vec![
        (
            {
                let mut content = with_curtains();
                content.gate_curtains.pop();
                content
            },
            ServiceError::GateCurtainCount {
                groups: 3,
                gates: 4,
            },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0].clear();
                content
            },
            ServiceError::GateCurtainLeafCount {
                gate: 0,
                leaves: 0,
                max: MAX_GATE_CURTAIN_LEAVES,
            },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0] = vec![synthetic_curtain(10.0, 0.0); 3];
                content
            },
            ServiceError::GateCurtainLeafCount {
                gate: 0,
                leaves: 3,
                max: MAX_GATE_CURTAIN_LEAVES,
            },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0][0].vertices[0][0] = f32::NAN;
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0][0].vertices = [
                    [10.0, 0.0, 0.0],
                    [10.0, 0.0, 0.0],
                    [10.0, 4.0, 0.0],
                    [10.0, 8.0, 0.0],
                ];
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
        (
            {
                // A horizontal leaf: its normal points up, so it is not a vertical curtain.
                let mut content = with_curtains();
                content.gate_curtains[0][0].vertices = [
                    [0.0, 4.0, 0.0],
                    [10.0, 4.0, 0.0],
                    [10.0, 4.0, 10.0],
                    [0.0, 4.0, 10.0],
                ];
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0][0].half_depth = 0.0;
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0][0].half_depth = MAX_GATE_CURTAIN_HALF_DEPTH_M + 1.0;
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
        (
            {
                let mut content = with_curtains();
                content.gate_curtains[0][0].vertices = [
                    [10.0, 0.0, 0.0],
                    [10.0, 0.0, 500.0],
                    [10.0, 8.0, 500.0],
                    [10.0, 8.0, 0.0],
                ];
                content
            },
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
        ),
    ];
    for (content, expected) in cases {
        assert_eq!(KartService::new(content).unwrap_err(), expected);
    }
}

/// The validator must reject the four malformed shapes the audit found it accepting (or rejecting
/// only by accident): a folded leaf, a sub-floor sliver, a concave ring and a bow tie. Each vector
/// fails exactly one rule, so a regression in that rule shows up as this test failing.
#[test]
fn curtain_validator_rejects_folded_sliver_concave_and_bow_tie() {
    let vectors: [(&str, [[f32; 3]; 4]); 4] = [
        (
            // Second triangle 68.6 deg out of plane: the both-normals-vertical and planarity rules.
            "folded",
            [
                [10.0, 0.0, -20.0],
                [10.0, 0.0, 20.0],
                [10.0, 8.0, 20.0],
                [30.0, 8.0, -20.0],
            ],
        ),
        (
            // 0.3 m2 first triangle: below the documented 0.5 m2 floor once the area is halved.
            "sliver",
            [
                [10.0, 0.0, 0.0],
                [10.0, 0.0, 40.0],
                [10.0, 0.015, 40.0],
                [10.0, 0.06, 0.0],
            ],
        ),
        (
            // Interior diagonal at v2: the turn at the concave vertex flips sign.
            "concave",
            [
                [10.0, 0.0, 0.0],
                [10.0, 40.0, 0.0],
                [10.0, 10.0, 5.0],
                [10.0, 0.0, 40.0],
            ],
        ),
        (
            // Crossing edges: a turn flips sign even though both triangles look vertical.
            "bow tie",
            [
                [10.0, 0.0, -20.0],
                [10.0, 0.0, 20.0],
                [10.0, 8.0, -20.0],
                [10.0, 8.0, 20.0],
            ],
        ),
    ];
    for (name, vertices) in vectors {
        let mut content = with_curtains();
        content.gate_curtains[0][0].vertices = vertices;
        assert_eq!(
            KartService::new(content).unwrap_err(),
            ServiceError::GateCurtainGeometry { gate: 0, leaf: 0 },
            "{name} must be refused"
        );
    }
    // The authored shape itself still passes all four rules.
    assert!(KartService::new(with_curtains()).is_ok());
}

/// `gate_curtains` is opt-in content: absent keeps the legacy volume, a leaf without `half_depth`
/// takes the recovered 3 m default, and an unknown leaf key (the exporter's `normal`/`center`) is
/// refused instead of silently ignored.
#[test]
fn curtains_are_opt_in_with_the_recovered_half_depth_default() {
    assert!(synthetic_content().gate_curtains.is_empty());

    let mut value = serde_json::to_value(with_curtains()).unwrap();
    value.as_object_mut().unwrap().remove("gate_curtains");
    let legacy: KartServiceContent = serde_json::from_value(value).unwrap();
    assert!(
        legacy.gate_curtains.is_empty(),
        "content written before the curtains still loads"
    );

    let mut value = serde_json::to_value(with_curtains()).unwrap();
    value["gate_curtains"][0][0]
        .as_object_mut()
        .unwrap()
        .remove("half_depth");
    let loaded: KartServiceContent = serde_json::from_value(value).unwrap();
    assert_eq!(
        loaded.gate_curtains[0][0].half_depth,
        GATE_CURTAIN_HALF_DEPTH_M
    );

    let mut value = serde_json::to_value(with_curtains()).unwrap();
    value["gate_curtains"][0][0]["normal"] = serde_json::json!([0.0, 0.0, 1.0]);
    assert!(
        serde_json::from_value::<KartServiceContent>(value).is_err(),
        "an injected normal must not be silently ignored"
    );
}

/// The snapshot mirrors the curtains, so a changed leaf cannot resume the race; legacy pairs and
/// the saved credit keep their own bounds.
#[test]
fn restore_binds_the_curtains_and_bounds_the_saved_credit() {
    let content = with_curtains();
    let (service, _) = racing(content.clone());
    let snapshot = service.snapshot().unwrap();
    assert_eq!(snapshot.gate_curtains, content.gate_curtains);
    assert!(KartService::restore(content.clone(), snapshot.clone()).is_ok());

    let mut changed = content.clone();
    changed.gate_curtains[0][0].vertices[0][2] += 1.0;
    assert_eq!(
        KartService::restore(changed, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "gate_curtains"
        }
    );

    // A snapshot written before the curtains (field absent) must not resume into curtain content.
    let mut value = serde_json::to_value(&snapshot).unwrap();
    value.as_object_mut().unwrap().remove("gate_curtains");
    let legacy: ServiceSnapshot = serde_json::from_value(value).unwrap();
    assert!(legacy.gate_curtains.is_empty());
    assert_eq!(
        KartService::restore(content, legacy).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "gate_curtains"
        }
    );

    // Legacy content and a legacy snapshot still round-trip with an empty bank.
    let (legacy_service, _) = started();
    let legacy_snapshot = legacy_service.snapshot().unwrap();
    assert!(legacy_snapshot.gate_curtains.is_empty());
    assert_eq!(legacy_snapshot.claim_credit_m, 0.0);
    assert!(KartService::restore(synthetic_content(), legacy_snapshot).is_ok());

    // A doctored bank above the cap cannot resume either.
    let (service, _) = racing(with_curtains());
    let mut doctored = service.snapshot().unwrap();
    doctored.claim_credit_m = CLAIM_CREDIT_CAP_SECONDS * 60.0 + 0.5;
    assert_eq!(
        KartService::restore(with_curtains(), doctored).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "claim_credit_m"
        }
    );
}

/// Two ordered laps of authored curtain claims finish the race and offer the reward once.
#[test]
fn ordered_repeated_laps_finish_on_curtains() {
    let (mut service, context) = racing(with_curtains());
    drive_lap(&mut service, &context, 15_000);
    assert_eq!(service.session().unwrap().laps_completed(), 1);
    drive_lap(&mut service, &context, 20_000);
    assert_eq!(service.phase(), RacePhase::Finished);
    assert!(service.take_finished_session().is_some());
    assert!(
        service.take_finished_session().is_none(),
        "the reward offer is one-shot"
    );
}

/// A client-side flush spends banked credit instead of minting time: the first burst claim is
/// paid from savings, the next flush is refused once the bank is empty, and the cursor holds.
#[test]
fn flush_spends_banked_credit_and_cannot_mint_time() {
    let mut content = synthetic_content();
    content.gates = (1..=4)
        .map(|index| GateWaypoint {
            position: [index as f32 * 60.0, 0.0, 0.0],
            radius: 3.0,
        })
        .collect();
    content.start_position = [0.0, 0.0, 0.0];
    let (mut service, context) = racing(content);
    // Gate 1 at 60 m after the 2.759 s countdown-0-to-claim leg: ~105.5 m of spare is banked.
    assert!(matches!(
        claim(&mut service, &context, [60.0, 0.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    // 16 ms later: inside the rate floor, so only the bank can pay for the next 60 m.
    assert!(matches!(
        claim(&mut service, &context, [120.0, 0.0, 0.0], at(16_016)),
        ServiceOutcome::Handled { .. }
    ));
    // The bank holds ~45.5 m now; the next flush cannot pay for 60 m and is refused.
    assert!(matches!(
        refusal(&claim(
            &mut service,
            &context,
            [180.0, 0.0, 0.0],
            at(16_032)
        )),
        ServiceRefusal::ClaimRate { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(3));
}

/// Idling never buys more than the capped window plus the bank, a clock that goes backwards is a
/// rate refusal, and replaying a passed gate stays stale.
#[test]
fn teleports_are_capped_after_an_idle_and_clock_regressions_refuse() {
    let (mut service, context) = racing(synthetic_content());
    // 40 s of idle still only allows CLAIM_CREDIT_WINDOW_MS of travel; the bank is empty, so a
    // 240 m jump is refused.
    let idle_cap = 60.0 * (CLAIM_CREDIT_WINDOW_MS as f32 / 1_000.0);
    assert!(240.0 > idle_cap, "the jump is past the idle window itself");
    assert!(matches!(
        refusal(&claim(
            &mut service,
            &context,
            [240.0, 0.0, 0.0],
            at(54_505)
        )),
        ServiceRefusal::ImplausibleSpeed { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(1));

    let (mut service, context) = racing(synthetic_content());
    assert!(matches!(
        claim(&mut service, &context, [10.0, 0.0, 0.0], at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    // A claim whose clock predates the accepted one is a rate refusal, not free travel.
    assert!(matches!(
        refusal(&claim(&mut service, &context, [20.0, 0.0, 0.0], at(15_000))),
        ServiceRefusal::ClaimRate { .. }
    ));
    // Replaying the accepted gate does not advance twice.
    assert!(matches!(
        refusal(&claim(&mut service, &context, [10.0, 0.0, 0.0], at(17_000))),
        ServiceRefusal::StaleGate { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
}

#[test]
fn observation_budget_is_separate_from_the_gate_budget() {
    let mut content = synthetic_content();
    content.bounds.max_observations = 4;
    let context = synthetic_context();
    let mut service = enabled(content);
    service.begin(context.clone(), BEGIN_MS).unwrap();
    service.tick(at(14_505)).unwrap();
    let bulk = vec![0x2au8; SERVER_DATA_PAYLOAD_LEN];
    for step in 0..4u64 {
        let outcome = service
            .handle_inner(&inner(SERVER_DATA_SUBTYPE, &bulk), at(15_000 + step))
            .unwrap();
        assert!(
            matches!(outcome, ServiceOutcome::Handled { .. }),
            "observation {step} is accepted"
        );
    }
    assert_eq!(service.observations(), 4);
    assert_eq!(
        refusal(
            &service
                .handle_inner(&inner(SERVER_DATA_SUBTYPE, &bulk), at(15_004))
                .unwrap()
        ),
        &ServiceRefusal::ObservationBudget { observations: 4 }
    );
    // The high-rate bulk stream never consumed the gate budget: a real gate still advances.
    assert_eq!(service.snapshot().unwrap().attempts, 0);
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(2));
}

#[test]
fn teleports_rate_and_budget_are_refused() {
    // The bank is empty at the race start: a 1 ms flush cannot pay for a 10 m leg, so the rate
    // floor still refuses it.
    let (mut service, context) = started();
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(13_242))),
        &ServiceRefusal::ClaimRate { delta_ms: 1 }
    );
    // A normal claim banks the spare travel time of its leg.
    let (mut service, context) = started();
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
    // The bank pays for one flushed leg 1 ms later, exactly as the captured client's pair does.
    assert!(matches!(
        gate_claim(&mut service, &context, 2.0, at(16_001)),
        ServiceOutcome::Handled { .. }
    ));
    // A teleport is still refused by the movement bound.
    assert!(matches!(
        refusal(&claim(
            &mut service,
            &context,
            [500.0, 0.0, 0.0],
            at(16_100)
        )),
        ServiceRefusal::ImplausibleSpeed { .. }
    ));
    let nan = service
        .handle_inner(
            &inner(
                GATE_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [f32::NAN, 0.0, 0.0]),
            ),
            at(17_000),
        )
        .unwrap();
    assert_eq!(refusal(&nan), &ServiceRefusal::NonFinitePosition);

    // A flood of attempts is cut off even though every refusal grants nothing.
    let mut content = synthetic_content();
    content.bounds.max_claims = 12;
    let mut bounded = enabled(content);
    bounded.begin(synthetic_context(), BEGIN_MS).unwrap();
    bounded.tick(at(14_505)).unwrap();
    for step in 0..13u64 {
        let outcome = gate_claim(
            &mut bounded,
            &synthetic_context(),
            1.0,
            at(16_000 + step * 1_000),
        );
        if step == 12 {
            assert_eq!(
                refusal(&outcome),
                &ServiceRefusal::ClaimBudget { claims: 12 }
            );
        }
    }
    assert_eq!(bounded.phase(), RacePhase::Racing);
    assert!(!bounded.finish_flag());
}

#[test]
fn a_high_rate_gate_stream_cannot_finish_or_offer_a_reward() {
    // The bank is empty at the race start, so a stream 1 ms apart cannot pay for its 10 m legs:
    // the movement bound refuses every claim before attribution and the cursor never moves.
    let (mut service, context) = started();
    for step in 0..64u64 {
        let index = (step % 4) as f32 + 1.0;
        let _ = gate_claim(&mut service, &context, index, at(13_242 + step));
    }
    assert_eq!(service.session().unwrap().next_checkpoint(), Some(1));
    assert_eq!(service.phase(), RacePhase::Racing);
    assert!(!service.finish_flag());
    assert!(service.take_finished_session().is_none());
}

#[test]
fn timeout_aborts_without_a_reward() {
    let mut content = synthetic_content();
    content.bounds.max_race_ms = 60_000;
    let context = synthetic_context();
    let mut service = enabled(content);
    service.begin(context.clone(), BEGIN_MS).unwrap();
    service.tick(at(14_505)).unwrap();
    let outcome = service.tick(at(14_505 + 60_001)).unwrap();
    match outcome {
        ServiceOutcome::Handled {
            abort,
            reward_ready,
            ..
        } => {
            assert!(abort);
            assert!(!reward_ready);
        }
        other => panic!("expected an abort, got {other:?}"),
    }
    assert_eq!(service.phase(), RacePhase::Aborted);
    assert!(!service.finish_flag());
    assert!(service.take_finished_session().is_none());
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(80_000))),
        &ServiceRefusal::InvalidPhase {
            phase: RacePhase::Aborted
        }
    );
}

#[test]
fn gate_claim_after_the_race_window_is_refused_before_the_tick_aborts() {
    let mut content = synthetic_content();
    content.laps = 1;
    content.progress_template.lap_limit = 1;
    content.gates.truncate(1);
    content.bounds.max_race_ms = 60_000;
    // Keep the movement guard out of the way: only the whole-race window may refuse this claim.
    content.bounds.max_step_ms = 120_000;
    let (mut service, context) = racing(content);
    let before = service.snapshot().expect("snapshot before the timed claim");

    let timed = gate_claim(&mut service, &context, 1.0, at(13_241 + 60_001));
    assert_eq!(
        refusal(&timed),
        &ServiceRefusal::RaceTimeout {
            elapsed_ms: 60_001,
            limit_ms: 60_000,
        }
    );
    let mut expected = before.clone();
    expected.attempts += 1;
    assert_eq!(
        service.snapshot().expect("snapshot after the timed claim"),
        expected,
        "the timed claim must not finish, pay or move the race"
    );
    assert_eq!(service.phase(), RacePhase::Racing);
    assert!(!service.finish_flag());

    let aborted = service.tick(at(13_241 + 60_001)).unwrap();
    assert!(matches!(
        aborted,
        ServiceOutcome::Handled { abort: true, .. }
    ));
    assert_eq!(service.phase(), RacePhase::Aborted);
}

#[test]
fn bulk_observations_are_bounded_and_inert() {
    let (mut service, context) = started();
    let before = service.phase();
    let bulk = vec![0x2au8; SERVER_DATA_PAYLOAD_LEN];
    let outcome = service
        .handle_inner(&inner(SERVER_DATA_SUBTYPE, &bulk), at(15_000))
        .unwrap();
    assert!(replies(&outcome).is_empty(), "33/6 carries no reply");
    assert_eq!(service.phase(), before);
    assert_eq!(service.observations(), 1);
    assert_eq!(
        service
            .handle_inner(&inner(SERVER_DATA_SUBTYPE, &bulk[..10]), at(15_100))
            .unwrap_err(),
        ServiceError::PayloadLength {
            subtype: SERVER_DATA_SUBTYPE,
            expected: SERVER_DATA_PAYLOAD_LEN,
            actual: 10,
        }
    );
    let pickup = inner(
        kart_service::PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [1.0, 2.0, 3.0]),
    );
    // With no pickup content the claim is a bounded observation that still changes nothing: the
    // observation budget counts it, and the refusal carries no replies.
    let outcome = service.handle_inner(&pickup, at(15_200)).unwrap();
    assert!(outcome.replies().is_empty());
    assert_eq!(refusal(&outcome), &ServiceRefusal::NoPickupInRange);
    assert_eq!(service.observations(), 2);
    assert_eq!(
        refusal(
            &service
                .handle_inner(&inner(PROGRESS_SUBTYPE, &[0u8; 73]), at(15_300))
                .unwrap()
        ),
        &ServiceRefusal::UnsupportedSubtype {
            subtype: PROGRESS_SUBTYPE
        }
    );
    assert_eq!(
        service
            .handle_inner(&[0x20, 0x00, 0x07, 0x00], at(15_400))
            .unwrap_err(),
        ServiceError::UnsupportedOpcode { opcode: 32 }
    );
    assert_eq!(
        service.handle_inner(&[0x21, 0x00], at(15_500)).unwrap_err(),
        ServiceError::FrameTooShort { len: 2, needed: 4 }
    );
    assert_eq!(
        service
            .handle_inner(&inner(GATE_HIT_SUBTYPE, &[0u8; 20]), at(15_600))
            .unwrap_err(),
        ServiceError::PayloadLength {
            subtype: GATE_HIT_SUBTYPE,
            expected: GATE_HIT_PAYLOAD_LEN,
            actual: 20,
        }
    );
}

/// Pickup content for the straight fixture: one box per expected gate on the start lane.
fn pickup_content() -> KartServiceContent {
    let mut content = synthetic_content();
    content.pickups = vec![
        kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0]),
        kart_pickups::PickupSpec::native(2, [40.0, 0.0, 0.0]),
        kart_pickups::PickupSpec::native(3, [80.0, 0.0, 0.0]),
    ];
    content
}

fn is_create(frame: &[u8]) -> bool {
    frame[2..4] == [kart_pickups::CREATE_SUBTYPE as u8, 0]
}

#[test]
fn pickup_grants_wait_for_the_native_racing_toolbar_transition() {
    // Native035338: countdown0 precedes state9 by1.264seconds; a grid-box grant
    // in that interval disappears from the toolbar. Announce boxes after state9.
    let content = pickup_content();
    let context = synthetic_context();
    let mut service = enabled(content.clone());
    service.begin(context, BEGIN_MS).unwrap();
    let countdown_zero = service.tick(at(13_241)).unwrap();
    assert_eq!(service.phase(), RacePhase::Racing);
    assert!(
        !countdown_zero
            .replies()
            .iter()
            .any(|frame| is_create(frame))
    );
    assert!(!service.snapshot().unwrap().pickups_spawned);
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [0.0; 3]),
    );
    assert_eq!(
        refusal(&service.handle_inner(&claim, at(13_300)).unwrap()),
        &ServiceRefusal::PickupsNotSpawned
    );
    assert!(service.snapshot().unwrap().queued_items.is_empty());

    // Restore in the interval must preserve the pending announcement.
    let mut service = KartService::restore(content.clone(), service.snapshot().unwrap()).unwrap();
    service.set_gate(EntryGate::Enabled);
    let ready = service.tick(at(14_505)).unwrap();
    let mut expected = vec![inner(GAME_STATE_SUBTYPE, &9i32.to_le_bytes())];
    expected.extend(content.pickups.iter().map(kart_pickups::encode_create));
    assert_eq!(ready.replies(), expected.as_slice());
    assert!(service.tick(at(14_506)).unwrap().replies().is_empty());
}

#[test]
fn pickup_content_without_a_later_state9_keeps_countdown_zero_spawns() {
    let mut content = pickup_content();
    content
        .schedule
        .retain(|step| step.frame != ScheduledFrame::GameState(9));
    let mut service = enabled(content);
    service.begin(synthetic_context(), BEGIN_MS).unwrap();
    let go = service.tick(at(13_241)).unwrap();
    assert_eq!(
        go.replies().iter().filter(|frame| is_create(frame)).count(),
        3
    );
}

/// A refused claim may still spend the observation budget, so equality checks on "nothing moved"
/// normalise that one allowed difference.
fn ignoring_observations(mut snapshot: ServiceSnapshot) -> ServiceSnapshot {
    snapshot.observations = 0;
    snapshot
}

#[test]
fn pickups_spawn_once_at_state9_and_only_a_new_race_announces_them_again() {
    let content = pickup_content();
    let expected: Vec<Vec<u8>> = content
        .pickups
        .iter()
        .map(kart_pickups::encode_create)
        .collect();
    let mut service = enabled(content.clone());
    service.begin(synthetic_context(), BEGIN_MS).unwrap();
    let go = service.tick(at(14_505)).unwrap();
    let spawned: Vec<Vec<u8>> = go
        .replies()
        .iter()
        .filter(|f| is_create(f))
        .cloned()
        .collect();
    assert_eq!(
        spawned.as_slice(),
        expected.as_slice(),
        "the racing toolbar transition announces every live box once"
    );
    let later = service.tick(at(20_000)).unwrap();
    assert!(!later.replies().iter().any(|frame| is_create(frame)));
    // A new race is the only place consumed marks clear and the boxes are announced again.
    service.abort();
    service.begin(synthetic_context(), at(30_000)).unwrap();
    let second = service.tick(at(30_000 + 14_505)).unwrap();
    let respawned: Vec<Vec<u8>> = second
        .replies()
        .iter()
        .filter(|f| is_create(f))
        .cloned()
        .collect();
    assert_eq!(respawned.as_slice(), expected.as_slice());
}

#[test]
fn pickup_claim_awards_one_boost_and_never_moves_the_session() {
    let (mut service, context) = racing(pickup_content());
    let before = service.snapshot().unwrap();
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [0.0, 0.0, 0.0]),
    );
    let outcome = service.handle_inner(&claim, at(14_600)).unwrap();
    assert_eq!(
        replies(&outcome),
        &[
            kart_pickups::encode_grant(BOOST_ITEM_TYPE),
            kart_pickups::encode_destroy(1),
        ],
        "the grant comes first, then the destroy"
    );
    let after = service.snapshot().unwrap();
    assert_eq!(
        after.session, before.session,
        "a pickup never touches checkpoints, laps or times"
    );
    assert_eq!(after.last_position, [0.0, 0.0, 0.0]);
    assert_eq!(after.queued_items, vec![BOOST_ITEM_TYPE]);
    assert_eq!(after.pickups_consumed, vec![1]);
    assert_eq!(service.phase(), RacePhase::Racing);
    // The same claim cannot pay twice.
    let before = service.snapshot().unwrap();
    assert_eq!(
        refusal(&service.handle_inner(&claim, at(14_700)).unwrap()),
        &ServiceRefusal::NoPickupInRange
    );
    assert_eq!(
        ignoring_observations(service.snapshot().unwrap()),
        ignoring_observations(before)
    );
    // The granted entry is a normal use-item request away from the boost.
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        replies(&service.handle_inner(&use_item, at(14_800)).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert!(service.snapshot().unwrap().queued_items.is_empty());
}

#[test]
fn a_full_item_queue_refuses_before_any_box_is_consumed() {
    let (mut service, context) = racing(pickup_content());
    let start = service.snapshot().unwrap().race_started_ms;
    // A legitimate first checkpoint moves the movement anchor off the start line, so the later
    // collection is a normal bounded leg instead of an idle jump against the step window.
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, at(14_000)),
        ServiceOutcome::Handled { .. }
    ));
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    for epoch in 1..=3u64 {
        assert_eq!(
            replies(
                &service
                    .handle_inner(&award, start + epoch * BOOST_INTERVAL_MS)
                    .unwrap()
            ),
            &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
        );
    }
    assert_eq!(
        service.snapshot().unwrap().queued_items.len(),
        ITEM_QUEUE_CAP
    );
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [40.0, 0.0, 0.0]),
    );
    let before = service.snapshot().unwrap();
    assert_eq!(
        refusal(
            &service
                .handle_inner(&claim, start + 3 * BOOST_INTERVAL_MS)
                .unwrap()
        ),
        &ServiceRefusal::ItemQueueFull
    );
    assert_eq!(
        ignoring_observations(service.snapshot().unwrap()),
        ignoring_observations(before),
        "a full queue leaves the box, the queue and the credit alone"
    );
    // Draining one slot frees the same box.
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        replies(
            &service
                .handle_inner(&use_item, start + 3 * BOOST_INTERVAL_MS + 10)
                .unwrap()
        ),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &BOOST_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert_eq!(
        replies(
            &service
                .handle_inner(&claim, start + 3 * BOOST_INTERVAL_MS + 20)
                .unwrap()
        ),
        &[
            kart_pickups::encode_grant(BOOST_ITEM_TYPE),
            kart_pickups::encode_destroy(2),
        ]
    );
}

#[test]
fn pickup_refusals_move_nothing() {
    let (mut service, context) = racing(pickup_content());
    let start = service.snapshot().unwrap().race_started_ms;
    let before = service.snapshot().unwrap();
    let owned = context.player_guid;
    let foreign = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(owned + 1, [0.0, 0.0, 0.0]),
    );
    assert_eq!(
        refusal(&service.handle_inner(&foreign, start + 1_000).unwrap()),
        &ServiceRefusal::ForeignGuid {
            offered: owned + 1,
            owned
        }
    );
    assert_eq!(
        service
            .handle_inner(&inner(PICKUP_HIT_SUBTYPE, &[0u8; 23]), start + 1_100)
            .unwrap_err(),
        ServiceError::PayloadLength {
            subtype: PICKUP_HIT_SUBTYPE,
            expected: GATE_HIT_PAYLOAD_LEN,
            actual: 23,
        }
    );
    let nan = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(owned, [f32::NAN, 0.0, 0.0]),
    );
    assert_eq!(
        refusal(&service.handle_inner(&nan, start + 1_200).unwrap()),
        &ServiceRefusal::NonFinitePosition
    );
    // Inside the movement budget but outside every 3 m radius.
    let far = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(owned, [10.0, 0.0, 0.0]),
    );
    assert_eq!(
        refusal(&service.handle_inner(&far, start + 2_000).unwrap()),
        &ServiceRefusal::NoPickupInRange
    );
    // A 200 m jump in 1 s is beyond the 60 m the bounded window can pay for.
    let teleport = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(owned, [200.0, 0.0, 0.0]),
    );
    assert!(matches!(
        service.handle_inner(&teleport, start + 3_000).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::ImplausibleSpeed { .. })
    ));
    // The race window still bounds the claim.
    assert!(matches!(
        service.handle_inner(&foreign, start + 600_001).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::ForeignGuid { .. })
    ));
    let timed = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(owned, [0.0, 0.0, 0.0]),
    );
    assert!(matches!(
        service.handle_inner(&timed, start + 600_001).unwrap(),
        ServiceOutcome::Refused(ServiceRefusal::RaceTimeout { .. })
    ));
    let after = service.snapshot().unwrap();
    assert!(after.queued_items.is_empty());
    assert!(after.pickups_consumed.is_empty(), "no box was consumed");
    assert_eq!(after.last_position, before.last_position, "no anchor moved");
    assert_eq!(
        after.last_claim_ms, before.last_claim_ms,
        "no credit was spent"
    );
    assert_eq!(after.session, before.session);
    assert_eq!(
        after.attempts, before.attempts,
        "the gate budget is untouched"
    );
    // The four refused observation frames count against the observation budget only.
    assert_eq!(after.observations, before.observations + 4);
}

#[test]
fn pickup_snapshot_binds_definitions_and_restores_consumed_items_and_queue() {
    let content = pickup_content();
    let (mut service, context) = racing(content.clone());
    let claim = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [0.0, 0.0, 0.0]),
    );
    service.handle_inner(&claim, at(14_600)).unwrap();
    let snapshot = service.snapshot().unwrap();
    assert_eq!(snapshot.pickups, content.pickups);
    assert!(snapshot.pickups_spawned);
    assert_eq!(snapshot.pickups_consumed, vec![1]);
    assert_eq!(snapshot.queued_items, vec![BOOST_ITEM_TYPE]);

    let mut restored = KartService::restore(content.clone(), snapshot.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(restored.snapshot().unwrap().pickups_consumed, vec![1]);
    assert_eq!(
        restored.snapshot().unwrap().queued_items,
        vec![BOOST_ITEM_TYPE]
    );
    assert_eq!(
        refusal(&restored.handle_inner(&claim, at(14_700)).unwrap()),
        &ServiceRefusal::NoPickupInRange
    );
    // The next live box is still collectable after the restore.
    let next = inner(
        PICKUP_HIT_SUBTYPE,
        &gate_hit_payload(context.player_guid, [40.0, 0.0, 0.0]),
    );
    assert_eq!(
        replies(&restored.handle_inner(&next, at(15_500)).unwrap()),
        &[
            kart_pickups::encode_grant(BOOST_ITEM_TYPE),
            kart_pickups::encode_destroy(2),
        ]
    );
    // A different definition set must not resume a consumed mark.
    let mut other = content.clone();
    other.pickups = vec![kart_pickups::PickupSpec::native(1, [5.0, 0.0, 0.0])];
    assert!(matches!(
        KartService::restore(other, snapshot.clone()),
        Err(ServiceError::CorruptSnapshot { field: "pickups" })
    ));
    let mut unknown = snapshot.clone();
    unknown.pickups_consumed = vec![9];
    assert!(matches!(
        KartService::restore(content.clone(), unknown),
        Err(ServiceError::CorruptSnapshot {
            field: "pickups_consumed"
        })
    ));
    let mut unspawned = snapshot.clone();
    unspawned.pickups_spawned = false;
    assert!(matches!(
        KartService::restore(content.clone(), unspawned),
        Err(ServiceError::CorruptSnapshot {
            field: "pickups_spawned"
        })
    ));
    // Legacy saves carry none of the pickup fields and still resume.
    let mut legacy = serde_json::to_value(&snapshot).unwrap();
    for key in [
        "pickups",
        "pickups_consumed",
        "pickups_spawned",
        "pickup_respawn",
        "pickups_respawned",
        "pickups_pending",
    ] {
        assert!(legacy.as_object_mut().unwrap().remove(key).is_some());
    }
    let legacy: ServiceSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(legacy.pickups.is_empty());
    assert!(!legacy.pickups_spawned);
    assert!(legacy.pickups_consumed.is_empty());
    assert!(KartService::restore(synthetic_content(), legacy.clone()).is_ok());
    // A legacy save carries no definitions, so pickup content refuses to resume it.
    assert_eq!(
        KartService::restore(content.clone(), legacy).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "pickups" }
    );
    // Same definitions but never spawned: the race resumes and refuses to collect.
    let mut never_spawned = snapshot.clone();
    never_spawned.pickups_spawned = false;
    never_spawned.pickups_consumed.clear();
    let mut unspawned_race = KartService::restore(content, never_spawned).unwrap();
    unspawned_race.set_gate(EntryGate::Enabled);
    assert_eq!(
        refusal(&unspawned_race.handle_inner(&claim, at(16_000)).unwrap()),
        &ServiceRefusal::PickupsNotSpawned
    );
}

#[test]
fn legacy_content_without_pickups_emits_the_same_frames() {
    let mut value = serde_json::to_value(synthetic_content()).unwrap();
    value.as_object_mut().unwrap().remove("pickups");
    let legacy: KartServiceContent = serde_json::from_value(value).unwrap();
    assert!(legacy.pickups.is_empty());
    assert_eq!(legacy, synthetic_content());
    let mut service = enabled(legacy);
    service.begin(synthetic_context(), BEGIN_MS).unwrap();
    let go = service.tick(at(14_505)).unwrap();
    assert!(!go.replies().iter().any(|frame| is_create(frame)));
}

/// One box on the start line with an opt-in respawn policy.
fn respawn_content(delay_ms: u64, max_respawns: u32) -> KartServiceContent {
    let mut content = synthetic_content();
    content.pickups = vec![kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0])];
    content.pickup_respawn = Some(kart_pickups::PickupRespawn {
        delay_ms,
        max_respawns,
    });
    content
}

/// The one claim this fixture's box can receive: owned guid at the box position.
fn start_box_claim(
    service: &mut KartService,
    context: &RaceContext,
    now_ms: u64,
) -> ServiceOutcome {
    service
        .handle_inner(
            &inner(
                PICKUP_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [0.0, 0.0, 0.0]),
            ),
            now_ms,
        )
        .unwrap()
}

#[test]
fn opt_in_respawn_returns_the_box_after_the_delay_then_stops() {
    let (mut service, context) = racing(respawn_content(5_000, 1));
    let start = service.snapshot().unwrap().race_started_ms;
    assert_eq!(
        replies(&start_box_claim(&mut service, &context, start + 1_000)),
        &[
            kart_pickups::encode_grant(BOOST_ITEM_TYPE),
            kart_pickups::encode_destroy(1),
        ]
    );
    let scheduled = service.snapshot().unwrap();
    assert_eq!(scheduled.pickups_consumed, vec![1]);
    assert_eq!(
        scheduled.pickups_pending,
        vec![kart_pickups::PendingRespawn {
            handle: 1,
            due_ms: start + 6_000
        }]
    );
    // Before the due time the box stays away and nothing moves.
    let early = service.tick(start + 5_999).unwrap();
    assert!(!early.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(service.snapshot().unwrap(), scheduled);
    // At the due time exactly one create frame returns with the same handle.
    let due = service.tick(start + 6_000).unwrap();
    assert_eq!(
        due.replies(),
        &[kart_pickups::encode_create(
            &kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0])
        )]
    );
    let returned = service.snapshot().unwrap();
    assert!(returned.pickups_consumed.is_empty());
    assert_eq!(returned.pickups_respawned, vec![1]);
    assert!(returned.pickups_pending.is_empty());
    // The returned box is collectable once more; the budget then keeps it permanent.
    assert_eq!(
        replies(&start_box_claim(&mut service, &context, start + 6_100)),
        &[
            kart_pickups::encode_grant(BOOST_ITEM_TYPE),
            kart_pickups::encode_destroy(1),
        ]
    );
    assert!(service.snapshot().unwrap().pickups_pending.is_empty());
    assert_eq!(
        refusal(&start_box_claim(&mut service, &context, start + 6_200)),
        &ServiceRefusal::NoPickupInRange
    );
    assert!(
        service
            .tick(start + 60_000)
            .unwrap()
            .replies()
            .iter()
            .all(|frame| !is_create(frame))
    );
}

#[test]
fn respawn_never_fires_before_go_after_finish_or_past_the_window() {
    // A clock that predates Go must not return a box, even when its due time has passed.
    let (mut service, context) = racing(respawn_content(5_000, 2));
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 1_000);
    let before = service.snapshot().unwrap();
    let regressed = service.tick(start - 1).unwrap();
    assert!(!regressed.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(service.snapshot().unwrap(), before);
    // A timeout aborts the race; the pending return never fires.
    let timed = service.tick(start + 600_001).unwrap();
    match &timed {
        ServiceOutcome::Handled { abort, .. } => assert!(abort),
        other => panic!("expected an abort, got {other:?}"),
    }
    assert!(!timed.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(service.phase(), RacePhase::Aborted);
    // A finished race freezes its pending returns.
    let (mut finished, context) = racing(respawn_content(5_000, 2));
    let start = finished.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut finished, &context, start + 1_000);
    drive_lap(&mut finished, &context, 15_000);
    drive_lap(&mut finished, &context, 20_000);
    assert_eq!(finished.phase(), RacePhase::Finished);
    let pending = finished.snapshot().unwrap().pickups_pending;
    assert!(!pending.is_empty());
    let frozen = finished.tick(start + 60_000).unwrap();
    assert!(!frozen.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(finished.snapshot().unwrap().pickups_pending, pending);
}

/// A tick whose clock sits between a pending due time and the last accepted claim is a regression:
/// the older return must not fire under a clock the accepted stream has already moved past.
#[test]
fn respawn_tick_never_fires_on_a_regressed_clock() {
    let (mut service, context) = racing(respawn_content(5_000, 2));
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 1_000);
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, start + 9_000),
        ServiceOutcome::Handled { .. }
    ));
    let before = service.snapshot().unwrap();
    let stale = service.tick(start + 7_000).unwrap();
    assert!(!stale.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(service.snapshot().unwrap(), before);
    let due = service.tick(start + 9_500).unwrap();
    assert_eq!(
        due.replies()
            .iter()
            .filter(|frame| is_create(frame))
            .count(),
        1
    );
    assert!(service.snapshot().unwrap().pickups_pending.is_empty());
}

#[test]
fn full_queue_refusal_schedules_no_respawn() {
    let (mut service, context) = racing(respawn_content(5_000, 2));
    let start = service.snapshot().unwrap().race_started_ms;
    let award = inner(BOOST_REQUEST_SUBTYPE, &context.player_guid.to_le_bytes());
    for _ in 0..ITEM_QUEUE_CAP {
        assert_eq!(
            replies(&service.handle_inner(&award, start + 1_000).unwrap()),
            &[inner(BOOST_RESPONSE_SUBTYPE, &[])]
        );
    }
    let before = service.snapshot().unwrap();
    assert_eq!(
        refusal(&start_box_claim(&mut service, &context, start + 1_100)),
        &ServiceRefusal::ItemQueueFull
    );
    let after = service.snapshot().unwrap();
    assert_eq!(
        ignoring_observations(after),
        ignoring_observations(before),
        "a refused claim leaves the box, the queue and the schedule alone"
    );
}

#[test]
fn new_race_clears_respawn_state_and_announces_every_box() {
    let content = respawn_content(5_000, 2);
    let (mut service, context) = racing(content.clone());
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 1_000);
    service.tick(start + 6_000).unwrap();
    assert!(!service.snapshot().unwrap().pickups_respawned.is_empty());
    service.abort();
    service.begin(context, start + 10_000).unwrap();
    let go = service.tick(start + 10_000 + 14_505).unwrap();
    let spawns: Vec<Vec<u8>> = go
        .replies()
        .iter()
        .filter(|f| is_create(f))
        .cloned()
        .collect();
    assert_eq!(
        spawns,
        vec![kart_pickups::encode_create(
            &kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0])
        )]
    );
    let fresh = service.snapshot().unwrap();
    assert!(fresh.pickups_consumed.is_empty());
    assert!(fresh.pickups_respawned.is_empty());
    assert!(fresh.pickups_pending.is_empty());
}

#[test]
fn respawn_state_round_trips_and_tampered_saves_are_refused() {
    let content = respawn_content(5_000, 1);
    let (mut service, context) = racing(content.clone());
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 1_000);
    let snapshot = service.snapshot().unwrap();
    let mut restored = KartService::restore(content.clone(), snapshot.clone()).unwrap();
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(
        restored.snapshot().unwrap().pickups_pending,
        snapshot.pickups_pending
    );
    assert_eq!(
        restored.tick(start + 6_000).unwrap().replies(),
        &[kart_pickups::encode_create(
            &kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0])
        )]
    );
    // A different policy must not resume the race.
    let mut other = content.clone();
    other.pickup_respawn = Some(kart_pickups::PickupRespawn {
        delay_ms: 6_000,
        max_respawns: 1,
    });
    assert_eq!(
        KartService::restore(other, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "pickup_respawn"
        }
    );
    // A due time outside the race window, or an unknown handle, is a tampered save.
    let mut tampered = snapshot.clone();
    tampered.pickups_pending = vec![kart_pickups::PendingRespawn {
        handle: 1,
        due_ms: start + 600_001,
    }];
    assert_eq!(
        KartService::restore(content.clone(), tampered).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "pickups_consumed"
        }
    );
    let mut unknown = snapshot.clone();
    unknown.pickups_pending = vec![kart_pickups::PendingRespawn {
        handle: 9,
        due_ms: start + 6_000,
    }];
    assert_eq!(
        KartService::restore(content.clone(), unknown).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "pickups_consumed"
        }
    );
    // Consumed or scheduled state without a spawn is refused.
    let mut unspawned = snapshot.clone();
    unspawned.pickups_spawned = false;
    assert_eq!(
        KartService::restore(content, unspawned).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "pickups_spawned"
        }
    );
}

#[test]
fn legacy_content_keeps_permanent_consumption() {
    let (mut service, context) = racing(pickup_content());
    let start = service.snapshot().unwrap().race_started_ms;
    assert!(service.snapshot().unwrap().pickup_respawn.is_none());
    start_box_claim(&mut service, &context, start + 1_000);
    assert!(service.snapshot().unwrap().pickups_pending.is_empty());
    let later = service.tick(start + 30_000).unwrap();
    assert!(!later.replies().iter().any(|frame| is_create(frame)));
    assert_eq!(service.snapshot().unwrap().pickups_consumed, vec![1]);
}

#[test]
fn typed_pickup_items_flow_through_queue_use_and_restore() {
    let mut content = synthetic_content();
    let mut shield = kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0]);
    shield.item_type = kart_pickups::SHIELD_ITEM_TYPE;
    content.pickups = vec![shield];
    let (mut service, context) = racing(content.clone());
    let start = service.snapshot().unwrap().race_started_ms;
    assert_eq!(
        replies(&start_box_claim(&mut service, &context, start + 1_000)),
        &[
            kart_pickups::encode_grant(kart_pickups::SHIELD_ITEM_TYPE),
            kart_pickups::encode_destroy(1),
        ]
    );
    assert_eq!(
        service.snapshot().unwrap().queued_items,
        vec![kart_pickups::SHIELD_ITEM_TYPE]
    );
    // Using it names the recovered type; the client owns the effect and this service claims none.
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        replies(&service.handle_inner(&use_item, start + 1_100).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &kart_pickups::SHIELD_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert!(service.snapshot().unwrap().queued_items.is_empty());
    // Restore accepts the recovered queue types (within the three-slot cap) and refuses anything
    // else - the type foundation only names what the client applies, no effect is claimed here.
    let mut snapshot = service.snapshot().unwrap();
    snapshot.queued_items = vec![1, 3, 5];
    assert!(KartService::restore(content.clone(), snapshot.clone()).is_ok());
    snapshot.queued_items = vec![4];
    assert!(KartService::restore(content.clone(), snapshot.clone()).is_ok());
    for bad in [vec![2u32], vec![9u32], vec![0u32]] {
        let mut tampered = snapshot.clone();
        tampered.queued_items = bad;
        assert_eq!(
            KartService::restore(content.clone(), tampered).unwrap_err(),
            ServiceError::CorruptSnapshot {
                field: "queued_items"
            }
        );
    }
}

/// One shield box on the start line.
fn shield_content() -> KartServiceContent {
    let mut content = synthetic_content();
    let mut shield = kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0]);
    shield.item_type = kart_pickups::SHIELD_ITEM_TYPE;
    content.pickups = vec![shield];
    content
}

#[test]
fn shield_use_starts_a_bounded_deadline_and_ticks_one_expiry() {
    let (mut service, context) = racing(shield_content());
    let start = service.snapshot().unwrap().race_started_ms;
    let session = service.snapshot().unwrap().session;
    start_box_claim(&mut service, &context, start + 20_100);
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    assert_eq!(
        replies(&service.handle_inner(&use_item, start + 20_200).unwrap()),
        &[inner(
            USE_ITEM_RESPONSE_SUBTYPE,
            &kart_pickups::SHIELD_ITEM_TYPE.to_le_bytes()
        )]
    );
    assert_eq!(
        service.snapshot().unwrap().shield_expiry,
        Some(start + 20_200 + SHIELD_DURATION_MS)
    );
    // Before the deadline nothing fires; at the deadline exactly one 33/21(1) does.
    let early = service
        .tick(start + 20_200 + SHIELD_DURATION_MS - 1)
        .unwrap();
    assert!(
        !early
            .replies()
            .iter()
            .any(|frame| frame[2..4] == [kart_pickups::EXPIRE_SUBTYPE as u8, 0])
    );
    let due = service.tick(start + 20_200 + SHIELD_DURATION_MS).unwrap();
    assert_eq!(
        due.replies(),
        &[kart_pickups::encode_expire(
            kart_pickups::SHIELD_EXPIRE_VALUE
        )]
    );
    let after = service.snapshot().unwrap();
    assert!(after.shield_expiry.is_none());
    assert_eq!(after.session, session, "a shield never moves the session");
    assert!(!service.finish_flag());
    let again = service
        .tick(start + 20_200 + SHIELD_DURATION_MS + 1_000)
        .unwrap();
    assert!(
        !again
            .replies()
            .iter()
            .any(|frame| frame[2..4] == [kart_pickups::EXPIRE_SUBTYPE as u8, 0])
    );
}

#[test]
fn shield_use_refreshes_the_deadline_and_the_caller_clears_it_at_the_edges() {
    // Two shields let one use refresh the other's deadline.
    let mut content = synthetic_content();
    let mut first = kart_pickups::PickupSpec::native(1, [0.0, 0.0, 0.0]);
    first.item_type = kart_pickups::SHIELD_ITEM_TYPE;
    let mut second = kart_pickups::PickupSpec::native(2, [20.0, 0.0, 0.0]);
    second.item_type = kart_pickups::SHIELD_ITEM_TYPE;
    content.pickups = vec![first, second];
    let (mut service, context) = racing(content);
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 1_000);
    service
        .handle_inner(
            &inner(
                PICKUP_HIT_SUBTYPE,
                &gate_hit_payload(context.player_guid, [20.0, 0.0, 0.0]),
            ),
            start + 3_000,
        )
        .unwrap();
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    service.handle_inner(&use_item, start + 4_000).unwrap();
    assert_eq!(
        service.snapshot().unwrap().shield_expiry,
        Some(start + 9_000)
    );
    service.handle_inner(&use_item, start + 7_000).unwrap();
    assert_eq!(
        service.snapshot().unwrap().shield_expiry,
        Some(start + 12_000),
        "a repeated use refreshes the deadline"
    );
    let refreshed = service.tick(start + 9_500).unwrap();
    assert!(
        !refreshed
            .replies()
            .iter()
            .any(|frame| frame[2..4] == [kart_pickups::EXPIRE_SUBTYPE as u8, 0])
    );
    assert_eq!(
        service.tick(start + 12_000).unwrap().replies(),
        &[kart_pickups::encode_expire(
            kart_pickups::SHIELD_EXPIRE_VALUE
        )]
    );
    // Abort clears a live deadline without emitting anything.
    let (mut aborted, context) = racing(shield_content());
    let start = aborted.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut aborted, &context, start + 1_000);
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    aborted.handle_inner(&use_item, start + 2_000).unwrap();
    assert!(aborted.snapshot().unwrap().shield_expiry.is_some());
    aborted.abort();
    assert!(aborted.snapshot().unwrap().shield_expiry.is_none());
    let after_abort = aborted.tick(start + 20_000).unwrap();
    assert!(after_abort.replies().is_empty());
    // Finishing clears it too: no effect expires after the race.
    let (mut finished, context) = racing(shield_content());
    let start = finished.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut finished, &context, start + 1_000);
    finished
        .handle_inner(
            &inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes()),
            start + 2_000,
        )
        .unwrap();
    drive_lap(&mut finished, &context, 15_000);
    drive_lap(&mut finished, &context, 20_000);
    assert_eq!(finished.phase(), RacePhase::Finished);
    assert!(finished.snapshot().unwrap().shield_expiry.is_none());
    assert!(finished.tick(start + 30_000).unwrap().replies().is_empty());
}

#[test]
fn shield_deadline_is_clamped_to_the_window_and_restore_refuses_tampering() {
    // The clamp case reaches the end of the race window, so this fixture allows a long step.
    let mut content = shield_content();
    content.bounds.max_step_ms = 600_000;
    let (mut service, context) = racing(content.clone());
    let start = service.snapshot().unwrap().race_started_ms;
    start_box_claim(&mut service, &context, start + 597_000);
    let use_item = inner(USE_ITEM_SUBTYPE, &context.player_guid.to_le_bytes());
    service.handle_inner(&use_item, start + 598_000).unwrap();
    // 598 s + 5 s would pass the 600 s window: the deadline clamps to the window end.
    assert_eq!(
        service.snapshot().unwrap().shield_expiry,
        Some(start + 600_000)
    );
    let snapshot = service.snapshot().unwrap();
    let restored = KartService::restore(content.clone(), snapshot.clone()).unwrap();
    assert_eq!(
        restored.snapshot().unwrap().shield_expiry,
        Some(start + 600_000)
    );
    // A deadline past the window is a tampered save.
    let mut tampered = snapshot.clone();
    tampered.shield_expiry = Some(start + 600_001);
    assert_eq!(
        KartService::restore(content.clone(), tampered).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "shield_expiry"
        }
    );
    // A live deadline in a non-racing phase is incoherent too.
    let mut countdown = enabled(content.clone());
    countdown.begin(synthetic_context(), BEGIN_MS).unwrap();
    let mut countdown = countdown.snapshot().unwrap();
    assert_eq!(countdown.phase, RacePhase::Countdown);
    countdown.shield_expiry = Some(countdown.race_started_ms + 1_000);
    assert_eq!(
        KartService::restore(content.clone(), countdown).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "shield_expiry"
        }
    );
    // Legacy saves without the field restore with no shield running.
    let mut legacy = serde_json::to_value(&snapshot).unwrap();
    assert!(
        legacy
            .as_object_mut()
            .unwrap()
            .remove("shield_expiry")
            .is_some()
    );
    let legacy: ServiceSnapshot = serde_json::from_value(legacy).unwrap();
    assert!(legacy.shield_expiry.is_none());
    assert!(KartService::restore(content.clone(), legacy).is_ok());
}

#[test]
fn abort_and_replay_paths_never_offer_a_reward() {
    let (mut service, context) = started();
    drive_lap(&mut service, &context, 15_000);
    service.abort();
    assert_eq!(service.phase(), RacePhase::Aborted);
    assert!(!service.finish_flag());
    assert!(service.take_finished_session().is_none());
    assert_eq!(
        refusal(&gate_claim(&mut service, &context, 1.0, at(20_000))),
        &ServiceRefusal::InvalidPhase {
            phase: RacePhase::Aborted
        }
    );
}

#[test]
fn snapshot_restore_round_trips_and_preserves_the_one_shot() {
    let (mut service, context) = started();
    drive_lap(&mut service, &context, 15_000);
    let snapshot = service.snapshot().expect("snapshot");
    assert_eq!(snapshot.session.laps_completed, 1);
    let mut restored =
        KartService::restore(synthetic_content(), snapshot.clone()).expect("restore");
    restored.set_gate(EntryGate::Enabled);
    assert_eq!(restored.phase(), RacePhase::Racing);
    drive_lap(&mut restored, &context, 20_000);
    assert!(restored.finish_flag());

    let mut corrupt = snapshot.clone();
    corrupt.session.laps_completed = 99;
    assert!(matches!(
        KartService::restore(synthetic_content(), corrupt).unwrap_err(),
        ServiceError::CorruptSnapshot { .. }
    ));
    let mut corrupt = snapshot.clone();
    corrupt.attempts = u32::MAX;
    assert_eq!(
        KartService::restore(synthetic_content(), corrupt).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "attempts" }
    );

    let taken = restored.take_finished_session();
    assert!(taken.is_some());
    let after = restored.snapshot().expect("snapshot after claim");
    assert!(after.reward_taken);
    let mut restored =
        KartService::restore(synthetic_content(), after).expect("restore after claim");
    restored.set_gate(EntryGate::Enabled);
    assert!(!restored.finish_flag());
    assert!(restored.take_finished_session().is_none());
}

#[test]
fn restore_refuses_a_different_layout_or_binding() {
    let context = synthetic_context();
    let (mut service, _) = started();
    drive_lap(&mut service, &context, 15_000);
    let snapshot = service.snapshot().expect("snapshot");

    let mut priced = synthetic_content();
    priced.reward = Some(RewardSchedule {
        coins: 999,
        xp: 999,
    });
    assert_eq!(
        KartService::restore(priced, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "track" }
    );
    let mut laps = synthetic_content();
    laps.laps = 3;
    laps.progress_template.lap_limit = 3;
    assert_eq!(
        KartService::restore(laps, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "track" }
    );
    let mut speedway = synthetic_content();
    speedway.speedway = 32;
    assert_eq!(
        KartService::restore(speedway, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "track" }
    );
    let mut rebind = snapshot.clone();
    rebind.context.instance_id = 1009;
    assert_eq!(
        KartService::restore(synthetic_content(), rebind).unwrap_err(),
        ServiceError::CorruptSnapshot {
            field: "session_id"
        }
    );
    // The context is validated exactly like `begin`.
    let mut zero_session = snapshot.clone();
    zero_session.context.session = 0;
    assert_eq!(
        KartService::restore(synthetic_content(), zero_session).unwrap_err(),
        ServiceError::ZeroSession
    );
    let mut zero_guid = snapshot.clone();
    zero_guid.context.player_guid = 0;
    assert_eq!(
        KartService::restore(synthetic_content(), zero_guid).unwrap_err(),
        ServiceError::ZeroPlayerGuid
    );
    // Changed clock limits must not silently re-judge a saved race.
    let mut limited = synthetic_content();
    limited.bounds.max_race_ms = 30_000;
    assert_eq!(
        KartService::restore(limited, snapshot.clone()).unwrap_err(),
        ServiceError::CorruptSnapshot { field: "bounds" }
    );
}

#[test]
fn clone_preserves_gate_and_progress() {
    let (service, context) = started();
    let mut clone = service.clone();
    assert_eq!(clone.gate(), EntryGate::Enabled, "a clone stays enabled");
    assert_eq!(clone.phase(), RacePhase::Racing);
    drive_lap(&mut clone, &context, 15_000);
    assert_eq!(clone.session().unwrap().laps_completed(), 1);
    assert_eq!(
        service.session().unwrap().laps_completed(),
        0,
        "the original is untouched by the clone"
    );
}

#[test]
fn older_content_without_the_observation_budget_still_loads() {
    // Content written before `max_observations` existed must keep loading (schema compatibility):
    // the field defaults to 600 s at 200 Hz.
    let mut value = serde_json::to_value(synthetic_content()).unwrap();
    value["bounds"]
        .as_object_mut()
        .unwrap()
        .remove("max_observations");
    let loaded: KartServiceContent = serde_json::from_value(value).unwrap();
    assert_eq!(
        loaded.bounds.max_observations,
        kart_service::default_max_observations()
    );
    assert_eq!(loaded.bounds.max_observations, 120_000);
    let context = synthetic_context();
    let mut service = enabled(loaded);
    service.begin(context.clone(), BEGIN_MS).unwrap();
    service.tick(at(14_505)).unwrap();
    assert!(matches!(
        gate_claim(&mut service, &context, 1.0, at(16_000)),
        ServiceOutcome::Handled { .. }
    ));
}

#[test]
fn content_bounds_are_refused_at_construction() {
    let mut content = synthetic_content();
    content.gates.truncate(3);
    KartService::new(content).unwrap();

    let mut content = synthetic_content();
    content.gates.clear();
    assert!(matches!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateCount { count: 0, .. }
    ));
    let mut content = synthetic_content();
    content.gates[1].radius = 0.0;
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateRadius {
            index: 1,
            radius: 0.0
        }
    );
    // Below the pragmatic floor the captured sampling gap (9.8 m between neighbouring hits) could
    // not be covered at all.
    let mut content = synthetic_content();
    content.gates[0].radius = 2.5;
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::GateRadius {
            index: 0,
            radius: 2.5
        }
    );
    let mut content = synthetic_content();
    content.bounds.max_observations = 0;
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::Bounds {
            field: "max_observations"
        }
    );
    let mut content = synthetic_content();
    content.gates[2].position = [f32::INFINITY, 0.0, 0.0];
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::NonFiniteGate { index: 2 }
    );
    let mut content = synthetic_content();
    content.schedule.swap(3, 4);
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::ScheduleOrder { index: 4 }
    );
    let mut content = synthetic_content();
    content.schedule.push(RaceStep {
        at_ms: 20_000,
        frame: ScheduledFrame::Countdown(9),
    });
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::ScheduleCountdown {
            index: 12,
            value: 9
        }
    );
    let mut content = synthetic_content();
    content.bounds.max_speed_mps = 0.0;
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::Bounds {
            field: "max_speed_mps"
        }
    );
    let mut content = synthetic_content();
    content.progress_template.lap_limit = 5;
    assert_eq!(
        KartService::new(content).unwrap_err(),
        ServiceError::ProgressTemplate { field: "lap_limit" }
    );
    let mut content = synthetic_content();
    content.laps = 0;
    content.progress_template.lap_limit = 0;
    assert!(matches!(
        KartService::new(content).unwrap_err(),
        ServiceError::Track(_)
    ));
    let mut service = enabled(synthetic_content());
    let mut context = synthetic_context();
    context.session = 0;
    assert_eq!(
        service.begin(context, BEGIN_MS).unwrap_err(),
        ServiceError::ZeroSession
    );
    let mut context = synthetic_context();
    context.player_guid = 0;
    assert_eq!(
        service.begin(context, BEGIN_MS).unwrap_err(),
        ServiceError::ZeroPlayerGuid
    );
}

/// Private gate: replay the pinned capture's real 33/7 route and cadence through the service.
///
/// `OSFR_KART_SERVICE_CAPTURE` must point at `lab/reports/original-minigame-packets`. Optional
/// `OSFR_KART_SERVICE_CONTENT` / `OSFR_KART_SERVICE_CONTEXT` load the generated private package
/// (`build-private-content.py`); without them the content is derived here. Waypoints are
/// reconstructed from the captured trajectory itself - the original gate geometry is unrecovered -
/// so the test proves the validator accepts the real positions and timings, nothing more.
#[test]
#[ignore = "requires OSFR_KART_SERVICE_CAPTURE pointing at lab/reports/original-minigame-packets"]
fn private_capture_route_is_accepted() {
    use sha2::{Digest, Sha256};

    fn digest(data: &[u8]) -> String {
        Sha256::digest(data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    let root = std::path::PathBuf::from(
        std::env::var_os("OSFR_KART_SERVICE_CAPTURE")
            .expect("OSFR_KART_SERVICE_CAPTURE must name the preserved frame directory"),
    );
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("racing-application-index.json")).unwrap())
            .unwrap();
    let times: std::collections::HashMap<String, f64> = index
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            (
                entry["sha256"].as_str().unwrap().to_string(),
                entry["time"].as_f64().unwrap(),
            )
        })
        .collect();

    let mut register_time = None;
    let mut hits = Vec::new();
    for file in std::fs::read_dir(&root).unwrap() {
        let path = file.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let register = name.contains("op33-sub9");
        let gate = name.contains("op33-sub7");
        if !register && !gate {
            continue;
        }
        let raw = std::fs::read(&path).unwrap();
        let time = times[&digest(&raw)];
        if register {
            register_time = Some(time);
            continue;
        }
        assert_eq!(raw.len(), 35, "{name} is a 24-byte gate-hit payload");
        let guid = u64::from_le_bytes(raw[11..19].try_into().unwrap());
        hits.push((
            time,
            guid,
            [
                f32::from_le_bytes(raw[19..23].try_into().unwrap()),
                f32::from_le_bytes(raw[23..27].try_into().unwrap()),
                f32::from_le_bytes(raw[27..31].try_into().unwrap()),
            ],
        ));
    }
    hits.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(hits.len(), 31, "the capture holds 31 gate hits");
    let guid = hits[0].1;
    assert!(hits.iter().all(|hit| hit.1 == guid), "one local racer guid");
    let register_time = register_time.expect("the capture holds the 33/9 register");

    let derived = KartServiceContent {
        speedway: 31,
        laps: 3,
        reward: None,
        gates: hits
            .iter()
            .map(|hit| GateWaypoint {
                position: hit.2,
                // The derived fallback keeps the reconstructed sphere path valid: the service
                // requires at least the 3 m floor, and each captured hit is its own gate centre.
                radius: 3.0,
            })
            .collect(),
        schedule: recovered_schedule(),
        progress_template: ProgressPayload {
            lap_limit: 3,
            ..synthetic_progress_template()
        },
        start_position: hits[0].2,
        bounds: RaceBounds {
            max_speed_mps: 60.0,
            max_step_ms: 30_000,
            max_race_ms: 600_000,
            max_claims: 128,
            max_observations: 20_000,
        },
        gate_band: None,
        gate_curtains: Vec::new(),
        pickups: Vec::new(),
        pickup_respawn: None,
        weapon_effects: false,
    };
    let derived_context = RaceContext {
        session: guid,
        player_guid: guid,
        instance_id: 1008,
        encounter_id: 917_566,
    };
    // The generated private package, when present, must be usable as-is.
    let content = match std::env::var_os("OSFR_KART_SERVICE_CONTENT") {
        Some(path) => {
            let bytes = std::fs::read(path).expect("readable private content package");
            serde_json::from_slice::<KartServiceContent>(&bytes).expect("private content package")
        }
        None => derived,
    };
    let context = match std::env::var_os("OSFR_KART_SERVICE_CONTEXT") {
        Some(path) => {
            let bytes = std::fs::read(path).expect("readable private context package");
            serde_json::from_slice::<RaceContext>(&bytes).expect("private context package")
        }
        None => derived_context,
    };
    assert_eq!(content.gates.len(), 31);
    assert_eq!(content.laps, 3);
    assert_eq!(context.player_guid, guid);
    assert!(
        content.gates.iter().all(|gate| gate.radius >= 3.0),
        "private gate radii must clear the pragmatic 3 m floor"
    );

    let mut service = KartService::new(content).unwrap();
    service.set_gate(EntryGate::Enabled);
    service.begin(context, BEGIN_MS).unwrap();
    service.tick(at(14_505)).unwrap();
    assert_eq!(service.phase(), RacePhase::Racing);

    let mut progress_frames = 0;
    for (time, _, position) in &hits {
        let now_ms = BEGIN_MS + ((time - register_time) * 1_000.0).round() as u64;
        let outcome = service
            .handle_inner(
                &inner(GATE_HIT_SUBTYPE, &gate_hit_payload(guid, *position)),
                now_ms,
            )
            .unwrap_or_else(|error| panic!("gate at {now_ms} ms failed: {error:?}"));
        progress_frames += replies(&outcome)
            .iter()
            .filter(|frame| u16::from_le_bytes([frame[2], frame[3]]) == PROGRESS_SUBTYPE)
            .count();
    }
    assert_eq!(
        service.session().unwrap().laps_completed(),
        1,
        "the fixture circle of 31 captured samples closes exactly one lap; the real track geometry is \
         the lead's reconstruction, not this content"
    );
    assert!(progress_frames >= 31, "each accepted gate reports progress");
    assert_eq!(
        service.phase(),
        RacePhase::Racing,
        "3-lap race is still running"
    );
    assert!(!service.finish_flag());
    assert_eq!(service.observations(), 0);
}
