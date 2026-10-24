//! Piece 054 (melee attacks) tests: the reviewed 3 m melee rule, the recovered
//! attack animation/timing contract and far-hit rejection.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `Resources/CodeAnimationMappings.txt`: `33^1000^com_h2h_attack`,
//!   `26^1001^com_h2h_attack_01` and `121^1099^com_swing`.
//! * `Resources/WieldTypeAnimationMappings.txt`: wield type 1 (Hand to Hand)
//!   plays melee group 1000, the job wield types 9..19/22..24/29..30 play 1099
//!   (`com_swing`), bow 5 plays 1102 and thrown 6 has no melee group.
//! * `Resources/AnimationGroups.xml`: `com_swing` is group 1099 with
//!   `fallback="1000"` and the hand-to-hand slot `com_h2h_attack_01`.
//! * `Resources/AnimationTypes.xml`: `com_h2h_attack_01` slot id 1001,
//!   type 2 priority 10 loadType 2 freezeMovement 0 doNotInterrupt 1
//!   boneMaskSuffix `upperbody`, inside `AnimationTreeBranch name="Combat"`.
//! * `Resources/AnimationTransitionRules.xml`: entering a combat animation is a
//!   single play (`Any` -> `Combat`, numLoops 1, speed 1.0, delay 0.0,
//!   easeIn 0.2, easeOut 0.2) and leaving it eases back over 0.6/0.2 s.
//! * Reviewed native run `client-baseline-20261008T081219Z`: all 20 processed
//!   player hits follow a native `26/10` selection-103 request by less than
//!   150 ms, at measured distances 1.970-2.434 units, with the player swing
//!   animation 1099 (the recovered Brawler row) every time.
//!
//! RECONSTRUCTED (labelled, not historical): the server-side melee reaches
//! (player 3.0, enemy 2.5), the 600 ms repeat gate and the pursuit speed. The
//! capture does not test every rejected distance at the configured boundary.
//!
//! NOT CLAIMED: the original clip duration/attack frame window (a Granny GR2
//! read was not performed), original damage, or original action-bar routing.
// The module is not registered in `src/lib.rs` (lead-owned). Including the exact
// source file keeps this target compiling against the real artifact. Integration
// step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_054.rs"]
#[allow(dead_code)]
mod piece_054;

use piece_054::{
    ANY_TO_COMBAT_TIMING, COM_H2H_ATTACK_01_SLOT, COM_H2H_ATTACK_01_SLOT_ID,
    COM_H2H_ATTACK_GROUP_ID, COM_SWING_FALLBACK_ID, COM_SWING_GROUP_ID, COMBAT_TO_STAND_TIMING,
    CombatSide, ENEMY_MELEE_REACH, H2H_ATTACK_01_SLOT, MELEE_REPEAT_GATE_MS, MeleeRefusal,
    PLAYER_MELEE_REACH, attack_animation, melee_animation_group, melee_distance,
    resolve_melee_attack,
};

const BRAWLER_WIELD_TYPE: u32 = 13;

/// Collapse whitespace so the production-rule assertions survive formatting.
fn normalized(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn player_attack(
    distance: f32,
    health: i32,
    now_ms: u64,
    last_attack_ms: Option<u64>,
) -> Result<piece_054::MeleeAttack, MeleeRefusal> {
    resolve_melee_attack(
        CombatSide::Player,
        [0.0, 0.0, 0.0],
        [distance, 0.0, 0.0],
        health,
        now_ms,
        last_attack_ms,
        BRAWLER_WIELD_TYPE,
    )
}

#[test]
fn production_melee_reach_is_still_the_reviewed_three_units() {
    // The production encounter server is read-only here; this test fails if the
    // reviewed melee reach is widened, if the ranged kits are folded into it, or
    // if the far-hit and repeat gates disappear.
    let encounter = normalized(include_str!("../src/encounter.rs"));
    // The enemy contact rule moved into the arena simulation (T439).
    let arena = normalized(include_str!("../src/encounter/arena.rs"));
    // Reach is code; damage per job moved to `combat_scale.json` (T639), so only
    // the reach rows are pinned here: melee 3.0 for every job but the two ranged kits.
    let scale = normalized(include_str!("../src/encounter/combat_scale.rs"));
    assert!(
        scale.contains("35 => 30.0, 12 => 20.0, _ => 3.0,"),
        "the Archer or Wizard ranged kit was folded into melee, or the melee reach changed"
    );
    assert!(
        encounter.contains("distance(position, s.position) > range"),
        "the production attack no longer rejects a target beyond reach"
    );
    assert!(
        encounter.contains("saturating_sub(last) < 600"),
        "the production repeat gate changed"
    );
    // Enemy reach is tunable data (behaviours.json, the owner 2026-10-15: "change
    // reach on the fly"); only the code default keeps the reviewed 2.5 m.
    assert!(
        arena.contains("distance(position, s.position) <= b.reach")
            && encounter.contains("reach: 2.5,"),
        "the reconstructed enemy contact rule changed"
    );

    assert_eq!(PLAYER_MELEE_REACH, 3.0);
    assert_eq!(ENEMY_MELEE_REACH, 2.5);
    assert_eq!(MELEE_REPEAT_GATE_MS, 600);
}

#[test]
fn far_hits_are_rejected_at_the_reviewed_boundary() {
    // The pre-fix eight-unit reach accepted this visibly distant hit.
    let far = player_attack(8.0, 180, 10_000, None).expect_err("8 units is out of reach");
    match far {
        MeleeRefusal::OutOfReach { distance, reach } => {
            assert_eq!(distance, 8.0);
            assert_eq!(reach, PLAYER_MELEE_REACH);
        }
        other => panic!("expected OutOfReach, got {other:?}"),
    }

    // Exactly at the reviewed reach is accepted (the production rule rejects
    // only `> range`), and the next representable float above it is rejected.
    assert!(player_attack(3.0, 180, 10_000, None).is_ok());
    let just_out = f32::from_bits(3.0_f32.to_bits() + 1);
    assert!(
        player_attack(just_out, 180, 10_000, None).is_err(),
        "the first float above 3.0 must be refused"
    );
}

#[test]
fn reviewed_capture_distances_are_accepted_and_the_old_reach_would_not_be() {
    // Measured server-observed distances from native run 081219 (min, winning
    // hits and the run maximum).
    for distance in [1.970_f32, 2.108, 2.434] {
        assert!(
            player_attack(distance, 180, 10_000, None).is_ok(),
            "captured hit at {distance} units was refused"
        );
        assert!(distance <= PLAYER_MELEE_REACH);
    }
    // The reconstructed review note's fix: the earlier eight-unit reach would
    // have accepted a hit this far away.
    assert!(player_attack(2.434, 180, 10_000, None).is_ok());
    assert!(player_attack(8.0, 180, 10_000, None).is_err());
}

#[test]
fn repeat_gate_refuses_a_second_attack_inside_six_hundred_ms() {
    let inside = player_attack(2.0, 180, 10_599, Some(10_000)).expect_err("599 ms is too soon");
    assert_eq!(
        inside,
        MeleeRefusal::RepeatGate {
            since_last_ms: 599,
            minimum_ms: MELEE_REPEAT_GATE_MS
        }
    );
    assert!(player_attack(2.0, 180, 10_600, Some(10_000)).is_ok());
}

#[test]
fn dead_targets_and_invalid_positions_are_refused() {
    assert_eq!(
        player_attack(2.0, 0, 10_000, None).expect_err("dead target"),
        MeleeRefusal::DeadTarget
    );
    assert_eq!(
        resolve_melee_attack(
            CombatSide::Player,
            [f32::NAN, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            180,
            10_000,
            None,
            BRAWLER_WIELD_TYPE,
        )
        .expect_err("invalid attacker position"),
        MeleeRefusal::InvalidPosition
    );
}

#[test]
fn swing_animation_follows_the_recovered_wield_table() {
    // Brawler is the reviewed native class; its original row plays com_swing.
    assert_eq!(
        melee_animation_group(BRAWLER_WIELD_TYPE),
        COM_SWING_GROUP_ID
    );
    assert_eq!(melee_animation_group(13), 1099);
    assert_eq!(melee_animation_group(1), COM_H2H_ATTACK_GROUP_ID);
    assert_eq!(melee_animation_group(2), 1020);
    assert_eq!(melee_animation_group(5), 1102);
    assert_eq!(
        melee_animation_group(6),
        0,
        "thrown has no recovered melee group"
    );
    assert_eq!(melee_animation_group(7), 1133);
    assert_eq!(melee_animation_group(10), 1099);

    let accepted = player_attack(2.0, 180, 10_000, None).expect("close hit is accepted");
    assert_eq!(accepted.swing_animation, COM_SWING_GROUP_ID);
    assert_eq!(accepted.distance, 2.0);
    // Creatures use their recovered attack slot, not a wield-type group.
    assert_eq!(
        attack_animation(CombatSide::Enemy, 0),
        COM_H2H_ATTACK_01_SLOT_ID
    );
    assert_eq!(attack_animation(CombatSide::Enemy, 0), 1001);
}

#[test]
fn animation_and_timing_evidence_matches_the_original_client_resources() {
    // Values re-read from the preserved original Resources by
    // `scripts/pieces/piece_054.py`; these constants pin the same rows.
    assert_eq!(COM_SWING_GROUP_ID, 1099);
    assert_eq!(COM_SWING_FALLBACK_ID, 1000);
    assert_eq!(COM_H2H_ATTACK_GROUP_ID, 1000);
    assert_eq!(COM_H2H_ATTACK_01_SLOT_ID, 1001);
    assert_eq!(H2H_ATTACK_01_SLOT, COM_H2H_ATTACK_01_SLOT);
    assert_eq!(COM_H2H_ATTACK_01_SLOT.name, "com_h2h_attack_01");
    assert_eq!(COM_H2H_ATTACK_01_SLOT.slot_type, 2);
    assert_eq!(COM_H2H_ATTACK_01_SLOT.priority, 10);
    assert_eq!(COM_H2H_ATTACK_01_SLOT.load_type, 2);
    assert!(!COM_H2H_ATTACK_01_SLOT.freeze_movement);
    assert!(COM_H2H_ATTACK_01_SLOT.do_not_interrupt);
    assert_eq!(COM_H2H_ATTACK_01_SLOT.bone_mask, "upperbody");

    assert_eq!(ANY_TO_COMBAT_TIMING.num_loops, 1);
    assert_eq!(ANY_TO_COMBAT_TIMING.speed, 1.0);
    assert_eq!(ANY_TO_COMBAT_TIMING.delay_seconds, 0.0);
    assert_eq!(ANY_TO_COMBAT_TIMING.ease_in_seconds, 0.2);
    assert_eq!(ANY_TO_COMBAT_TIMING.ease_out_seconds, 0.2);
    assert_eq!(COMBAT_TO_STAND_TIMING.ease_in_seconds, 0.6);
    assert_eq!(COMBAT_TO_STAND_TIMING.ease_out_seconds, 0.2);
}

#[test]
fn melee_distance_matches_the_production_encounter_math() {
    // Three-dimensional f32 sum-of-squares then sqrt, exactly like the
    // production `distance` helper.
    assert_eq!(melee_distance([0.0, 0.0, 0.0], [3.0, 0.0, 0.0]), 3.0);
    assert_eq!(melee_distance([1.0, 2.0, 2.0], [1.0, 2.0, 2.0]), 0.0);
    assert_eq!(melee_distance([0.0, 0.0, 0.0], [0.0, 4.0, 3.0]), 5.0);
}
