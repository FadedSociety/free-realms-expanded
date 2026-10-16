//! Piece 052 (enemy patrol and idle) tests: the recovered movement/animation
//! contract plus a bounded reconstructed patrol controller.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party native code and shipped resources; hashes and offsets
//! in `notes/Research/Native NPC Motion and Animation.md` and re-verified by
//! `scripts/pieces/piece_052.py` against the preserved lab inputs):
//! * Movement is the standalone opcode 125 application message,
//!   `u16 opcode, u64 guid, 3f position, 3f rotation, u8 state, u8 unknown`
//!   = 36 bytes (native reader 008e5940; no timestamp field). The three rotation
//!   floats are a direction vector, not a truncated quaternion: native 0095dae0
//!   requires a nonzero horizontal X/Z length, normalizes XYZ and derives yaw
//!   with atan2(X, Z). Community NPC movement uses State 1 / Unknown 0.
//! * Animation is `u16 opcode(35), u16 subtype(8), u64 guid, i32 animationId,
//!   i32 unknown, u8 flags` = 21 bytes. Flags bit 0 set writes the entity base
//!   animation; clear plays it immediately.
//! * `Resources/CodeAnimationMappings.txt` maps data ids 1=loc_stand,
//!   2=loc_walk, 3=loc_run, 1000=com_h2h_attack, 1001=com_h2h_attack_01. The
//!   AddNpc template carries Walk/Run/Stand at 137/141/145.
//!
//! RECONSTRUCTED (labelled, not historical): waypoint order, dwell time, the
//! patrol speed ceiling, arrival radius and route extent bounds. The original
//! per-actor patrol routes and dwell timings are not recovered; this module
//! never invents them, it validates a caller-supplied route and steps it.
//!
//! NOT CLAIMED: rendered native idle/run playback, per-model animation support,
//! or that any recovered enemy historically patrolled this way. The native gate
//! stays with the lead and the replay checker
//! (`scripts/pieces/piece_052.py --run <capture>`).
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_052.rs"]
#[allow(dead_code)]
mod piece_052;

use piece_052::{
    ANIM_ID_ATTACK, ANIM_ID_RUN, ANIM_ID_STAND, ANIM_ID_WALK, ANIMATION_BYTES, ANIMATION_FLAG_BASE,
    ANIMATION_OPCODE, ANIMATION_SUBTYPE, MAX_ARRIVAL_RADIUS, MAX_DWELL_MS, MAX_PATROL_SPEED,
    MAX_ROUTE_EXTENT_M, MAX_WAYPOINTS, MOVEMENT_BYTES, MOVEMENT_OPCODE, MOVEMENT_STATE_MOVING,
    PatrolController, PatrolError, PatrolPhase, PatrolRoute, Waypoint, animation_packet,
    locomotion_animation_ids, movement_packet,
};

const GUID: u64 = 0x1122_3344_5566_7788;

fn unit_route() -> PatrolRoute {
    // A small synthetic route: two waypoints 10 m apart on the X axis. This is a
    // test fixture, not recovered patrol data.
    PatrolRoute::new(
        vec![
            Waypoint {
                position: [0.0, 0.0, 0.0],
                dwell_ms: 500,
            },
            Waypoint {
                position: [10.0, 0.0, 0.0],
                dwell_ms: 500,
            },
        ],
        3.0,
        0.5,
    )
    .expect("fixture route is valid")
}

fn controller(start_ms: u64) -> PatrolController {
    PatrolController::new(GUID, unit_route(), start_ms).expect("fixture controller is valid")
}

#[test]
fn recovered_contract_constants() {
    assert_eq!(MOVEMENT_OPCODE, 125);
    assert_eq!(MOVEMENT_BYTES, 36);
    assert_eq!(MOVEMENT_STATE_MOVING, 1);
    assert_eq!(ANIMATION_OPCODE, 35);
    assert_eq!(ANIMATION_SUBTYPE, 8);
    assert_eq!(ANIMATION_BYTES, 21);
    assert_eq!(ANIMATION_FLAG_BASE, 1);
    assert_eq!((ANIM_ID_STAND, ANIM_ID_WALK, ANIM_ID_RUN), (1, 2, 3));
    assert_eq!(ANIM_ID_ATTACK, 1001);
    // The AddNpc template stores Walk/Run/Stand in that order at 137/141/145.
    assert_eq!(locomotion_animation_ids(), (2, 3, 1));
}

#[test]
fn movement_packet_matches_the_recovered_125_layout() {
    let packet = movement_packet(GUID, [1.0, 2.0, 3.0], [0.0, 0.0, -1.0]);
    assert_eq!(packet.len(), MOVEMENT_BYTES);
    let mut expected = Vec::new();
    expected.extend(125_u16.to_le_bytes());
    expected.extend(GUID.to_le_bytes());
    for value in [1.0_f32, 2.0, 3.0, 0.0, 0.0, -1.0] {
        expected.extend(value.to_le_bytes());
    }
    expected.push(1); // community NPC state
    expected.push(0); // unknown
    assert_eq!(packet, expected);
}

#[test]
fn animation_packet_matches_the_recovered_35_8_layout() {
    let run = animation_packet(GUID, ANIM_ID_RUN, true);
    assert_eq!(run.len(), ANIMATION_BYTES);
    let mut expected = Vec::new();
    expected.extend(35_u16.to_le_bytes());
    expected.extend(8_u16.to_le_bytes());
    expected.extend(GUID.to_le_bytes());
    expected.extend(3_i32.to_le_bytes());
    expected.extend(0_i32.to_le_bytes());
    expected.push(1);
    assert_eq!(run, expected);

    // Immediate (non-base) animation: the same layout with flags cleared.
    let attack = animation_packet(GUID, ANIM_ID_ATTACK, false);
    assert_eq!(attack.len(), ANIMATION_BYTES);
    assert_eq!(attack[12..16], 1001_i32.to_le_bytes());
    assert_eq!(attack[16..20], 0_i32.to_le_bytes());
    assert_eq!(attack[20], 0);
}

#[test]
fn patrol_cycles_run_and_stand_between_waypoints() {
    let mut patrol = controller(0);
    assert_eq!(patrol.phase(), PatrolPhase::Idle);
    assert_eq!(patrol.position(), [0.0, 0.0, 0.0]);

    // Still inside the first dwell window: no wire traffic.
    assert!(patrol.tick(400).replies.is_empty());

    // Dwell expired: the enemy starts running toward waypoint 1.
    let start = patrol.tick(500);
    assert_eq!(patrol.phase(), PatrolPhase::Running);
    // Still on waypoint 0: the index advances on arrival, not on departure.
    assert_eq!(patrol.waypoint_index(), 0);
    assert_eq!(start.replies.len(), 2, "movement plus run-base animation");
    assert_eq!(start.replies[0].len(), MOVEMENT_BYTES);
    assert_eq!(start.replies[1].len(), ANIMATION_BYTES);
    assert_eq!(&start.replies[1][..2], &35_u16.to_le_bytes());
    assert_eq!(&start.replies[1][2..4], &8_u16.to_le_bytes());
    assert_eq!(start.replies[1][12..16], 3_i32.to_le_bytes());
    assert_eq!(start.replies[1][20], ANIMATION_FLAG_BASE);
    assert!(patrol.position()[0] > 0.0);

    // Later ticks only carry movement until the arrival radius is reached.
    let mut now = 600;
    let mut arrivals = 0;
    let mut previous = patrol.position()[0];
    while arrivals == 0 && now < 20_000 {
        let update = patrol.tick(now);
        if !update.replies.is_empty() {
            assert_eq!(update.replies[0].len(), MOVEMENT_BYTES);
            assert!(patrol.position()[0] >= previous - f32::EPSILON);
            previous = patrol.position()[0];
            if update.replies.len() > 1 {
                assert_eq!(update.replies[1][12..16], 1_i32.to_le_bytes());
                assert_eq!(update.replies[1][20], ANIMATION_FLAG_BASE);
                arrivals += 1;
            }
        }
        now += 100;
    }
    assert_eq!(arrivals, 1, "arrival emits a stand-base animation");
    assert_eq!(patrol.phase(), PatrolPhase::Idle);
    assert_eq!(patrol.waypoint_index(), 1);
    assert_eq!(patrol.position(), [10.0, 0.0, 0.0]);

    // The next dwell window is respected before the loop heads back.
    assert!(patrol.tick(now + 100).replies.is_empty());
    let back = patrol.tick(now + 500);
    assert_eq!(patrol.waypoint_index(), 1);
    assert_eq!(back.replies[1][12..16], 3_i32.to_le_bytes());
    // Returning to waypoint 0 faces negative X: the heading is a direction
    // vector with a negative X component and no Y component.
    assert!(
        f32::from_le_bytes(back.replies[0][22..26].try_into().unwrap()) < 0.0,
        "return leg must face negative X"
    );
    assert_eq!(
        f32::from_le_bytes(back.replies[0][26..30].try_into().unwrap()),
        0.0
    );
}

#[test]
fn patrol_heading_is_always_a_unit_horizontal_vector() {
    let mut patrol = controller(0);
    let mut now = 0;
    for _ in 0..400 {
        now += 100;
        for reply in patrol.tick(now).replies {
            if reply.len() == MOVEMENT_BYTES {
                let heading = [
                    f32::from_le_bytes(reply[22..26].try_into().unwrap()),
                    f32::from_le_bytes(reply[26..30].try_into().unwrap()),
                    f32::from_le_bytes(reply[30..34].try_into().unwrap()),
                ];
                assert_eq!(heading[1], 0.0);
                let length = (heading[0] * heading[0] + heading[2] * heading[2]).sqrt();
                assert!(
                    (length - 1.0).abs() < 1e-5,
                    "heading {heading:?} is not unit length"
                );
                assert_eq!(reply[34], MOVEMENT_STATE_MOVING);
                assert_eq!(reply[35], 0);
            }
        }
    }
    // The loop is bounded: every visited position stays inside the route box.
    let position = patrol.position();
    assert!((-0.001..=10.001).contains(&position[0]));
    assert_eq!(position[1], 0.0);
    assert_eq!(position[2], 0.0);
}

#[test]
fn patrol_step_is_time_bounded() {
    let mut patrol = controller(0);
    patrol.tick(500);
    let before = patrol.position();
    // A huge wall-clock jump must not teleport the actor across the route.
    patrol.tick(60_000);
    let after = patrol.position();
    let travelled = (after[0] - before[0]).abs();
    let ceiling = 3.0 * 0.25 + 1e-3;
    assert!(
        travelled <= ceiling,
        "travelled {travelled} exceeds {ceiling}"
    );
}

#[test]
fn patrol_rejects_malformed_routes() {
    let waypoint = |x: f32| Waypoint {
        position: [x, 0.0, 0.0],
        dwell_ms: 100,
    };
    let cases: Vec<(Result<PatrolRoute, PatrolError>, PatrolError)> = vec![
        (
            PatrolRoute::new(vec![waypoint(0.0)], 3.0, 0.5),
            PatrolError::TooFewWaypoints,
        ),
        (
            PatrolRoute::new(vec![waypoint(10.0); MAX_WAYPOINTS + 1], 3.0, 0.5),
            PatrolError::TooManyWaypoints,
        ),
        (
            PatrolRoute::new(
                vec![
                    waypoint(0.0),
                    Waypoint {
                        position: [f32::NAN, 0.0, 0.0],
                        dwell_ms: 100,
                    },
                ],
                3.0,
                0.5,
            ),
            PatrolError::NonFinitePosition,
        ),
        (
            PatrolRoute::new(
                vec![
                    waypoint(0.0),
                    Waypoint {
                        position: [2_000_000.0, 0.0, 0.0],
                        dwell_ms: 100,
                    },
                ],
                3.0,
                0.5,
            ),
            PatrolError::PositionOutOfRange,
        ),
        (
            PatrolRoute::new(vec![waypoint(0.0), waypoint(0.0)], 3.0, 0.5),
            PatrolError::SegmentInsideArrivalRadius,
        ),
        (
            PatrolRoute::new(
                vec![
                    waypoint(0.0),
                    Waypoint {
                        position: [0.0, 4.0, 0.0],
                        dwell_ms: 100,
                    },
                ],
                3.0,
                0.5,
            ),
            PatrolError::VerticalOnlySegment,
        ),
        (
            PatrolRoute::new(vec![waypoint(0.0), waypoint(10.0)], f32::INFINITY, 0.5),
            PatrolError::NonFiniteSpeed,
        ),
        (
            PatrolRoute::new(vec![waypoint(0.0), waypoint(10.0)], 0.0, 0.5),
            PatrolError::SpeedOutOfRange,
        ),
        (
            PatrolRoute::new(
                vec![waypoint(0.0), waypoint(10.0)],
                MAX_PATROL_SPEED + 0.01,
                0.5,
            ),
            PatrolError::SpeedOutOfRange,
        ),
        (
            PatrolRoute::new(
                vec![waypoint(0.0), waypoint(10.0)],
                3.0,
                MAX_ARRIVAL_RADIUS + 0.01,
            ),
            PatrolError::InvalidArrivalRadius,
        ),
        (
            PatrolRoute::new(
                vec![
                    waypoint(0.0),
                    Waypoint {
                        position: [10.0, 0.0, 0.0],
                        dwell_ms: MAX_DWELL_MS + 1,
                    },
                ],
                3.0,
                0.5,
            ),
            PatrolError::DwellOutOfRange,
        ),
        (
            PatrolRoute::new(
                vec![
                    waypoint(0.0),
                    Waypoint {
                        position: [MAX_ROUTE_EXTENT_M * 2.0, 0.0, 0.0],
                        dwell_ms: 100,
                    },
                ],
                3.0,
                0.5,
            ),
            PatrolError::RouteTooLarge,
        ),
    ];
    for (result, expected) in cases {
        assert_eq!(result.unwrap_err(), expected);
    }
}

#[test]
fn patrol_never_targets_the_player_guid() {
    assert_eq!(
        PatrolController::new(0, unit_route(), 0).unwrap_err(),
        PatrolError::InvalidGuid
    );
    assert_eq!(
        PatrolController::new(1, unit_route(), 0).unwrap_err(),
        PatrolError::InvalidGuid
    );
}
