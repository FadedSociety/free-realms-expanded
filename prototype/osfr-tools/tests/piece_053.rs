//! Work item 053 acceptance: aggro and leash - acquisition, pursuit, disengage/reset and
//! invalid/out-of-zone target rejection.
//!
//! ORIGINAL evidence behind the recovered inputs (all outside this test):
//! * `lab/reports/native-combat-controls.json` function `00a2ac20` requires a hostile
//!   `Disposition == 0` at NPC `+0x178`, rejects the dead flag at `+0x6f8` bit 7, then applies a
//!   facing dot product and a line-of-sight ray. That is native target *selection*; this module
//!   consumes the same eligibility facts and never reimplements the facing/LOS test.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketAddNpc.cs`
//!   carries `Disposition` on the AddNpc wire record with 0 hostile / 1 neutral / 2 ally comments.
//!
//! RECONSTRUCTED rules exercised here (labelled, not historical): the pursuit radius, melee reach,
//! movement speed, 250 ms step clamp, 100 ms minimum tick interval, stop margin and the
//! return-to-anchor leash, all mirroring the registered `encounter.rs` behaviour this module is
//! intended to replace. The original server's aggro and leash numbers were not recovered.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_053;` under `src/restoration/`.
#[path = "../src/restoration/piece_053.rs"]
#[allow(dead_code)]
mod piece_053;

use piece_053::{
    AGGRO_MELEE_REACH, AGGRO_MOVE_SPEED, AGGRO_PURSUIT_RANGE, AGGRO_STEP_CLAMP_MS,
    AGGRO_STOP_MARGIN, AGGRO_TICK_INTERVAL_MS, AggroActor, AggroEvent, AggroPhase,
    DISPOSITION_ALLY, DISPOSITION_HOSTILE, DISPOSITION_NEUTRAL, DisengageReason, TargetCandidate,
    TargetRejection,
};

const PLAYER: u64 = 1;
const NPC: u64 = 0x4f50_454e_0000_0053;
const ZONE: i32 = 43;

fn player(guid: u64, position: [f32; 3]) -> TargetCandidate {
    TargetCandidate {
        guid,
        zone: ZONE,
        position,
        alive: true,
    }
}

fn actor() -> AggroActor {
    AggroActor::new(NPC, DISPOSITION_HOSTILE, ZONE, [0.0, 0.0, 0.0])
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn recovered_disposition_values_and_reconstructed_rules_are_pinned() {
    assert_eq!(
        (DISPOSITION_HOSTILE, DISPOSITION_NEUTRAL, DISPOSITION_ALLY),
        (0, 1, 2)
    );
    assert_eq!(AGGRO_PURSUIT_RANGE, 40.0);
    assert_eq!(AGGRO_MELEE_REACH, 2.5);
    assert_eq!(AGGRO_MOVE_SPEED, 3.0);
    assert_eq!(AGGRO_STEP_CLAMP_MS, 250);
    assert_eq!(AGGRO_TICK_INTERVAL_MS, 100);
    assert_eq!(AGGRO_STOP_MARGIN, 2.4);
}

#[test]
fn hostile_candidate_inside_the_pursuit_range_is_acquired() {
    let mut npc = actor();
    let events = npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    assert_eq!(events, vec![AggroEvent::Acquired { target: PLAYER }]);
    assert_eq!(npc.phase(), AggroPhase::Pursuing);
    assert_eq!(npc.target(), Some(PLAYER));
}

#[test]
fn non_hostile_actors_never_acquire_a_target() {
    for disposition in [DISPOSITION_NEUTRAL, DISPOSITION_ALLY] {
        let mut npc = AggroActor::new(NPC, disposition, ZONE, [0.0, 0.0, 0.0]);
        let candidate = player(PLAYER, [5.0, 0.0, 0.0]);
        assert!(npc.tick(&[candidate], 0).is_empty());
        assert_eq!(npc.phase(), AggroPhase::Idle);
        assert_eq!(npc.target(), None);
        assert_eq!(
            piece_053::rejection_for(&candidate, disposition, ZONE, [0.0, 0.0, 0.0]),
            Some(TargetRejection::NotHostile)
        );
    }
}

#[test]
fn invalid_candidates_are_rejected_with_typed_reasons() {
    let mut dead = player(PLAYER, [5.0, 0.0, 0.0]);
    dead.alive = false;
    let mut other_zone = player(PLAYER, [5.0, 0.0, 0.0]);
    other_zone.zone = ZONE + 1;
    let far = player(PLAYER, [AGGRO_PURSUIT_RANGE + 0.001, 0.0, 0.0]);
    let mut nan = player(PLAYER, [5.0, 0.0, 0.0]);
    nan.position[0] = f32::NAN;

    for (candidate, expected) in [
        (dead, TargetRejection::Dead),
        (other_zone, TargetRejection::OutOfZone),
        (far, TargetRejection::OutOfRange),
        (nan, TargetRejection::NonFinitePosition),
    ] {
        let mut npc = actor();
        let events = npc.tick(&[candidate], 0);
        assert!(events.is_empty(), "unexpected events for {expected:?}");
        assert_eq!(npc.phase(), AggroPhase::Idle);
        assert_eq!(npc.target(), None);
        assert_eq!(
            piece_053::rejection_for(&candidate, DISPOSITION_HOSTILE, ZONE, [0.0, 0.0, 0.0]),
            Some(expected)
        );
    }
}

#[test]
fn nearest_eligible_player_is_acquired_when_several_are_present() {
    let mut npc = actor();
    let events = npc.tick(
        &[
            player(7, [30.0, 0.0, 0.0]),
            player(9, [4.0, 0.0, 0.0]),
            player(8, [20.0, 0.0, 0.0]),
        ],
        0,
    );
    assert_eq!(events, vec![AggroEvent::Acquired { target: 9 }]);
}

#[test]
fn pursuit_steps_toward_the_target_without_overshooting_the_stop_margin() {
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    let mut now = 0_u64;
    loop {
        now += AGGRO_TICK_INTERVAL_MS;
        let events = npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], now);
        let position = npc.position();
        assert!(position.iter().all(|coord| coord.is_finite()));
        if events.is_empty() || matches!(events[0], AggroEvent::InReach { .. }) {
            break;
        }
        let head = match events[0] {
            AggroEvent::Pursuing {
                position, heading, ..
            } => {
                assert_eq!(position, npc.position());
                assert_eq!(heading[1], 0.0);
                position
            }
            ref other => panic!("unexpected pursuit event {other:?}"),
        };
        let remaining = distance(head, [10.0, 0.0, 0.0]);
        assert!(
            remaining >= AGGRO_STOP_MARGIN,
            "overshot the stop margin: {remaining}"
        );
    }
    let remaining = distance(npc.position(), [10.0, 0.0, 0.0]);
    assert!(
        remaining <= AGGRO_MELEE_REACH,
        "stopped outside reach: {remaining}"
    );
}

#[test]
fn one_step_is_clamped_to_the_reconstructed_speed() {
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    let events = npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 1000);
    let moved = match events[0] {
        AggroEvent::Pursuing { position, .. } => position[0],
        ref other => panic!("unexpected event {other:?}"),
    };
    assert_eq!(
        moved,
        AGGRO_MOVE_SPEED * AGGRO_STEP_CLAMP_MS as f32 / 1000.0
    );
}

#[test]
fn leaving_the_pursuit_range_disengages_and_returns_to_the_anchor() {
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 100);
    assert_ne!(npc.position(), [0.0, 0.0, 0.0]);
    let escaped = npc.tick(&[player(PLAYER, [100.0, 0.0, 0.0])], 200);
    assert_eq!(
        escaped,
        vec![AggroEvent::Disengaged {
            target: PLAYER,
            reason: DisengageReason::OutOfRange,
        }]
    );
    assert_eq!(npc.phase(), AggroPhase::Returning);
    assert_eq!(npc.target(), None);
    let mut now = 200_u64;
    loop {
        now += AGGRO_TICK_INTERVAL_MS;
        if npc
            .tick(&[], now)
            .iter()
            .any(|event| matches!(event, AggroEvent::ReturnedHome { .. }))
        {
            break;
        }
        assert!(now < 100_000, "actor never returned home");
    }
    assert_eq!(npc.phase(), AggroPhase::Idle);
    assert_eq!(npc.position(), [0.0, 0.0, 0.0]);
    assert!(
        npc.tick(&[], now + AGGRO_TICK_INTERVAL_MS).is_empty(),
        "an idle actor keeps emitting events"
    );
}

#[test]
fn dead_out_of_zone_and_missing_targets_disengage_with_typed_reasons() {
    for (candidate, expected) in [
        (
            {
                let mut dead = player(PLAYER, [10.0, 0.0, 0.0]);
                dead.alive = false;
                dead
            },
            DisengageReason::Dead,
        ),
        (
            {
                let mut zoned = player(PLAYER, [10.0, 0.0, 0.0]);
                zoned.zone = ZONE + 1;
                zoned
            },
            DisengageReason::OutOfZone,
        ),
    ] {
        let mut npc = actor();
        npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
        let events = npc.tick(&[candidate], 100);
        assert_eq!(
            events,
            vec![AggroEvent::Disengaged {
                target: PLAYER,
                reason: expected,
            }]
        );
        assert_eq!(npc.phase(), AggroPhase::Returning);
    }
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    assert_eq!(
        npc.tick(&[], 100),
        vec![AggroEvent::Disengaged {
            target: PLAYER,
            reason: DisengageReason::TargetGone,
        }]
    );
}

#[test]
fn fast_ticks_and_non_finite_anchors_do_not_move_the_actor() {
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    assert!(npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 50).is_empty());
    assert_eq!(npc.position(), [0.0, 0.0, 0.0]);
    let mut broken = AggroActor::new(NPC, DISPOSITION_HOSTILE, ZONE, [f32::NAN, 0.0, 0.0]);
    assert!(
        broken
            .tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0)
            .is_empty()
    );
    assert_eq!(broken.phase(), AggroPhase::Idle);
}

#[test]
fn boundaries_include_the_pursuit_range_and_the_melee_reach() {
    let mut npc = actor();
    let edge = player(PLAYER, [AGGRO_PURSUIT_RANGE, 0.0, 0.0]);
    assert_eq!(
        npc.tick(&[edge], 0),
        vec![AggroEvent::Acquired { target: PLAYER }]
    );
    assert!(matches!(
        npc.tick(&[edge], 100).as_slice(),
        [AggroEvent::Pursuing { target: PLAYER, .. }]
    ));

    let mut close = actor();
    close.tick(&[player(PLAYER, [5.0, 0.0, 0.0])], 0);
    let reach = player(PLAYER, [AGGRO_MELEE_REACH, 0.0, 0.0]);
    assert_eq!(
        close.tick(&[reach], 100),
        vec![AggroEvent::InReach { target: PLAYER }]
    );
    assert_eq!(close.position(), [0.0, 0.0, 0.0]);

    let mut out = actor();
    let beyond = player(PLAYER, [AGGRO_PURSUIT_RANGE + 0.001, 0.0, 0.0]);
    assert!(out.tick(&[beyond], 0).is_empty());
    assert_eq!(out.phase(), AggroPhase::Idle);
}

#[test]
fn returning_actor_reacquires_only_after_reaching_home() {
    let mut npc = actor();
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 0);
    npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 100);
    npc.tick(&[player(PLAYER, [60.0, 0.0, 0.0])], 200);
    assert_eq!(npc.phase(), AggroPhase::Returning);
    let events = npc.tick(&[player(PLAYER, [10.0, 0.0, 0.0])], 300);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, AggroEvent::Acquired { .. })),
        "reacquired while still returning: {events:?}"
    );
    let mut now = 300_u64;
    loop {
        now += AGGRO_TICK_INTERVAL_MS;
        if npc
            .tick(&[player(PLAYER, [10.0, 0.0, 0.0])], now)
            .iter()
            .any(|event| matches!(event, AggroEvent::Acquired { .. }))
        {
            break;
        }
        assert!(now < 100_000, "actor never returned home");
    }
    assert_eq!(npc.phase(), AggroPhase::Pursuing);
}
