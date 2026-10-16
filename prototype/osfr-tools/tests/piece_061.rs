//! Piece 061 (dungeon waves and gates) tests: recovered tutorial-cave room
//! boxes, door instances and gate clips, plus the reconstructed ordered
//! completion policy.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client data; hashes pinned in the lab bundle and in
//! `notes/Research/Pieces/061 Dungeon waves and gates.md`):
//! * `sg_npx_cavern_01Areas.xml` (pack_14/003872.xml, 8600 bytes): the tutorial
//!   room boxes `AMB_FIRST_ROOM` (6116), `AMB_TUT_SECOND_ROOM` (6114),
//!   `AMB_TUT_FINAL_ROOM` (6113) and the `STING_TUTORIAL` sting (6115).
//! * Original terrain runtime objects in `sg_npx_cavern_01_4_4.gcnk`
//!   (pack_01/001617.gcnk): `sg_npx_cavern_01_door_01.adr`
//!   (object 1358893701) and `sg_npx_cavern_01_door_02.adr`
//!   (object 1362704680) with their recorded positions/rotations.
//! * Original asset manifest rows: the door_01 clip set
//!   {stand, open, open_stand, close} under `sg_npx_cavern_01_door_01*`, the
//!   rock-barrier deploy/retract clips under
//!   `_npx_cavern_01_rockbarrier_01_loc_*`, and the interactive door verb
//!   sounds `OBJ_DOOR_LRG_OPEN_VERB.mp3` / `OBJ_DOOR_LRG_CLOSE_VERB.mp3`.
//!
//! RECONSTRUCTED, labelled at each call site: the ordered-completion policy,
//! the caller-supplied wave sizes (the original per-room actor counts are
//! unrecovered) and the server-side gate/position enforcement.
//!
//! NOT CLAIMED: any recovered trigger volume, wave composition or door-opening
//! script; the areas table carries no event records, and no NPX server script
//! was recovered. No native run was performed for this piece.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_061.rs"]
#[allow(dead_code)]
mod piece_061;

use piece_061::{
    ClearOutcome, DOOR_01, DOOR_01_CLIPS, DOOR_02, DOOR_VERB_SOUNDS, DungeonChain, FINAL_ROOM,
    FIRST_ROOM, GateError, GateId, GateState, ROCK_BARRIER_CLIPS, RoomId, SECOND_ROOM,
    TUTORIAL_STING, ValidatedClear, WaveSpec,
};

/// Synthetic fixture wave sizes. The original per-room actor counts are not
/// recovered, so tests use explicit small numbers instead of a historical claim.
fn fixture_chain() -> DungeonChain {
    DungeonChain::new(vec![
        WaveSpec::new(RoomId::First, 3),
        WaveSpec::new(RoomId::Second, 2),
        WaveSpec::new(RoomId::Final, 1),
    ])
    .expect("fixture waves cover every tutorial room once")
}

fn clear_room(chain: &mut DungeonChain, room: RoomId, count: u32) -> Option<GateId> {
    let mut opened = None;
    for index in 0..count {
        let instance = 1_000 + room.index() as u64 * 100 + index as u64;
        let outcome = chain
            .record_clear(ValidatedClear::enemy_defeated(room, instance))
            .expect("validated clear of the active room");
        if let Some(gate) = outcome.gate_opened {
            opened = Some(gate);
        }
    }
    opened
}

#[test]
fn recovered_room_boxes_come_from_the_original_area_table() {
    assert_eq!(FIRST_ROOM.area_name, "AMB_FIRST_ROOM");
    assert_eq!(FIRST_ROOM.area_id, 6116);
    assert_eq!(FIRST_ROOM.min, [261.0, 12.0, 268.0]);
    assert_eq!(FIRST_ROOM.max, [316.0, 42.0, 304.0]);

    assert_eq!(SECOND_ROOM.area_name, "AMB_TUT_SECOND_ROOM");
    assert_eq!(SECOND_ROOM.area_id, 6114);
    assert_eq!(SECOND_ROOM.min, [316.0, 8.0, 224.0]);
    assert_eq!(SECOND_ROOM.max, [446.0, 55.0, 340.0]);

    assert_eq!(FINAL_ROOM.area_name, "AMB_TUT_FINAL_ROOM");
    assert_eq!(FINAL_ROOM.area_id, 6113);
    assert_eq!(FINAL_ROOM.min, [292.0, -6.0, 351.0]);
    assert_eq!(FINAL_ROOM.max, [432.0, 79.0, 486.0]);

    assert_eq!(TUTORIAL_STING.area_name, "STING_TUTORIAL");
    assert_eq!(TUTORIAL_STING.area_id, 6115);
    assert_eq!(TUTORIAL_STING.min, [273.0, 25.0, 287.0]);
    assert_eq!(TUTORIAL_STING.max, [274.0, 29.0, 288.0]);

    // The recorded first-room spawn sample sits inside the recovered first-room box.
    assert!(FIRST_ROOM.contains([287.0, 28.0, 286.0]));
    assert!(!SECOND_ROOM.contains([287.0, 28.0, 286.0]));
}

#[test]
fn tutorial_rooms_are_ordered_by_both_name_and_axis() {
    let order = RoomId::tutorial_order();
    assert_eq!(order, [RoomId::First, RoomId::Second, RoomId::Final]);
    // The recovered names carry the room order; the boxes share a wall plane at
    // x = 316 and the final room sits beyond the second room's far wall in Z.
    assert_eq!(FIRST_ROOM.max[0], SECOND_ROOM.min[0]);
    assert_eq!(SECOND_ROOM.max[2], 340.0);
    assert_eq!(FINAL_ROOM.min[2], 351.0);
    assert!(FINAL_ROOM.min[2] > SECOND_ROOM.max[2]);
    // door_02 sits inside that 11-unit passage, so the gate count matches the
    // two room transitions rather than the box extents.
    assert!(DOOR_02.position[2] > SECOND_ROOM.max[2]);
    assert!(DOOR_02.position[2] < FINAL_ROOM.min[2]);
}

#[test]
fn recovered_doors_sit_on_the_recovered_room_transitions() {
    assert_eq!(DOOR_01.actor, "sg_npx_cavern_01_door_01.adr");
    assert_eq!(DOOR_01.object_id, 1358893701);
    assert_eq!(DOOR_01.scale, 1.0);
    assert_eq!(DOOR_01.rotation, [-1.5707964, 0.0, 0.0, 0.0]);
    let [x, y, z] = DOOR_01.position;
    // door_01 stands on the shared room1/room2 wall plane x=316.
    assert!((x - FIRST_ROOM.max[0]).abs() <= 1.5, "door_01 x {x}");
    assert!(y >= FIRST_ROOM.min[1] && y <= FIRST_ROOM.max[1]);
    assert!(z >= FIRST_ROOM.min[2] && z <= FIRST_ROOM.max[2]);

    assert_eq!(DOOR_02.actor, "sg_npx_cavern_01_door_02.adr");
    assert_eq!(DOOR_02.object_id, 1362704680);
    assert_eq!(DOOR_02.rotation, [-3.1415927, 0.0, 0.0, 0.0]);
    let [x2, y2, z2] = DOOR_02.position;
    // door_02 stands in the passage between the second room's far wall and the
    // final room's near wall.
    assert!(z2 > SECOND_ROOM.max[2], "door_02 z {z2}");
    assert!(z2 < FINAL_ROOM.min[2], "door_02 z {z2}");
    assert!(x2 >= SECOND_ROOM.min[0] && x2 <= SECOND_ROOM.max[0]);
    assert!(y2 >= SECOND_ROOM.min[1] && y2 <= SECOND_ROOM.max[1]);
}

#[test]
fn recovered_gate_clips_and_verb_sounds_are_listed() {
    assert_eq!(
        DOOR_01_CLIPS,
        [
            "sg_npx_cavern_01_door_01_stand.gr2",
            "sg_npx_cavern_01_door_01_open.gr2",
            "sg_npx_cavern_01_door_01_open_stand.gr2",
            "sg_npx_cavern_01_door_01_close.gr2",
        ]
    );
    assert_eq!(
        ROCK_BARRIER_CLIPS,
        [
            "_npx_cavern_01_rockbarrier_01_loc_stand_retract.gr2",
            "_npx_cavern_01_rockbarrier_01_loc_shoot_up.gr2",
            "_npx_cavern_01_rockbarrier_01_loc_stand_deployed.gr2",
        ]
    );
    assert_eq!(
        DOOR_VERB_SOUNDS,
        ["OBJ_DOOR_LRG_OPEN_VERB.mp3", "OBJ_DOOR_LRG_CLOSE_VERB.mp3"]
    );
}

#[test]
fn chain_opens_each_gate_only_after_its_room_is_cleared() {
    let mut chain = fixture_chain();
    assert_eq!(chain.gate_state(GateId::Door01), GateState::Closed);
    assert_eq!(chain.gate_state(GateId::Door02), GateState::Closed);
    assert_eq!(chain.active_room(), Some(RoomId::First));

    let first = chain
        .record_clear(ValidatedClear::enemy_defeated(RoomId::First, 1))
        .expect("first clear");
    assert_eq!(
        first,
        ClearOutcome {
            room: RoomId::First,
            cleared_actors: 1,
            total_actors: 3,
            room_cleared: false,
            gate_opened: None
        }
    );
    assert_eq!(chain.gate_state(GateId::Door01), GateState::Closed);

    assert_eq!(
        clear_room(&mut chain, RoomId::First, 2),
        Some(GateId::Door01)
    );
    assert_eq!(chain.gate_state(GateId::Door01), GateState::Open);
    assert_eq!(chain.gate_state(GateId::Door02), GateState::Closed);
    assert_eq!(chain.active_room(), Some(RoomId::Second));

    assert_eq!(
        clear_room(&mut chain, RoomId::Second, 2),
        Some(GateId::Door02)
    );
    assert_eq!(chain.gate_state(GateId::Door02), GateState::Open);
    assert_eq!(chain.active_room(), Some(RoomId::Final));
    assert!(!chain.is_complete());

    assert_eq!(clear_room(&mut chain, RoomId::Final, 1), None);
    assert!(chain.is_complete());
    assert_eq!(chain.active_room(), None);
}

#[test]
fn gate_requests_are_refused_until_the_server_unlocks_them() {
    let mut chain = fixture_chain();
    // An arbitrary client door event cannot open anything on its own.
    assert_eq!(
        chain.request_gate(GateId::Door01),
        Err(GateError::GateLocked(GateId::Door01))
    );
    assert_eq!(
        chain.request_gate(GateId::Door02),
        Err(GateError::GateLocked(GateId::Door02))
    );
    assert_eq!(chain.gate_state(GateId::Door01), GateState::Closed);

    clear_room(&mut chain, RoomId::First, 3);
    // Once the room clear unlocked it, a repeated request is idempotent.
    assert_eq!(chain.request_gate(GateId::Door01), Ok(GateState::Open));
    assert_eq!(chain.request_gate(GateId::Door01), Ok(GateState::Open));
    assert_eq!(
        chain.request_gate(GateId::Door02),
        Err(GateError::GateLocked(GateId::Door02))
    );
}

#[test]
fn duplicate_and_out_of_order_clears_are_rejected() {
    let mut chain = fixture_chain();
    assert!(
        chain
            .record_clear(ValidatedClear::enemy_defeated(RoomId::First, 7))
            .is_ok()
    );
    assert_eq!(
        chain.record_clear(ValidatedClear::enemy_defeated(RoomId::First, 7)),
        Err(GateError::DuplicateClear)
    );
    assert_eq!(
        chain.record_clear(ValidatedClear::enemy_defeated(RoomId::Second, 8)),
        Err(GateError::OutOfOrderClear(RoomId::Second))
    );
    assert_eq!(
        chain.record_clear(ValidatedClear::enemy_defeated(RoomId::First, 0)),
        Err(GateError::InvalidActorInstance)
    );
    assert_eq!(chain.cleared_actors(RoomId::First), 1);
}

#[test]
fn chain_refuses_wave_specs_that_do_not_match_the_recovered_rooms() {
    assert_eq!(
        DungeonChain::new(vec![WaveSpec::new(RoomId::First, 3)]),
        Err(GateError::InvalidWaveSpec)
    );
    assert_eq!(
        DungeonChain::new(vec![
            WaveSpec::new(RoomId::First, 3),
            WaveSpec::new(RoomId::First, 2),
            WaveSpec::new(RoomId::Final, 1),
        ]),
        Err(GateError::InvalidWaveSpec)
    );
    assert_eq!(
        DungeonChain::new(vec![
            WaveSpec::new(RoomId::First, 0),
            WaveSpec::new(RoomId::Second, 2),
            WaveSpec::new(RoomId::Final, 1),
        ]),
        Err(GateError::InvalidWaveSpec)
    );
}

#[test]
fn position_reports_reject_bypass_into_a_locked_room() {
    let mut chain = fixture_chain();
    assert_eq!(
        chain.report_position([287.0, 28.0, 286.0]),
        Ok(Some(RoomId::First))
    );
    // The corridor with door_02 belongs to no recovered room box.
    assert_eq!(chain.report_position([360.0, 33.0, 345.0]), Ok(None));
    // Walking past a closed gate is refused with the gate that blocks it.
    assert_eq!(
        chain.report_position([380.0, 30.0, 280.0]),
        Err(GateError::BypassAttempt {
            room: RoomId::Second,
            blocked_by: GateId::Door01
        })
    );
    assert_eq!(
        chain.report_position([360.0, 30.0, 400.0]),
        Err(GateError::BypassAttempt {
            room: RoomId::Final,
            blocked_by: GateId::Door01
        })
    );

    clear_room(&mut chain, RoomId::First, 3);
    assert_eq!(
        chain.report_position([380.0, 30.0, 280.0]),
        Ok(Some(RoomId::Second))
    );
    assert_eq!(
        chain.report_position([360.0, 30.0, 400.0]),
        Err(GateError::BypassAttempt {
            room: RoomId::Final,
            blocked_by: GateId::Door02
        })
    );

    clear_room(&mut chain, RoomId::Second, 2);
    assert_eq!(
        chain.report_position([360.0, 30.0, 400.0]),
        Ok(Some(RoomId::Final))
    );
    assert_eq!(chain.report_position([100.0, 0.0, 100.0]), Ok(None));
}

#[test]
fn extra_clears_after_a_completed_room_are_rejected_and_change_nothing() {
    let mut chain = DungeonChain::new(vec![
        WaveSpec::new(RoomId::First, 1),
        WaveSpec::new(RoomId::Second, 1),
        WaveSpec::new(RoomId::Final, 1),
    ])
    .expect("valid fixture");
    assert!(
        chain
            .record_clear(ValidatedClear::enemy_defeated(RoomId::First, 1))
            .is_ok()
    );
    assert_eq!(
        chain.record_clear(ValidatedClear::enemy_defeated(RoomId::First, 2)),
        Err(GateError::OutOfOrderClear(RoomId::First))
    );
    assert_eq!(chain.cleared_actors(RoomId::First), 1);
    assert_eq!(chain.active_room(), Some(RoomId::Second));
}
