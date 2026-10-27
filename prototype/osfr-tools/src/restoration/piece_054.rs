//! Piece 054 - melee attacks: the reviewed 3 m melee rule, the recovered
//! attack animation/timing contract and far-hit rejection.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/054 Melee attacks.md` and re-read from the preserved
//! original `Resources/` by the private recovery checker `piece_054.py`):
//!
//! * `CodeAnimationMappings.txt`: `33^1000^com_h2h_attack`,
//!   `26^1001^com_h2h_attack_01` and `121^1099^com_swing`.
//! * `WieldTypeAnimationMappings.txt`: `WIELD_TYPE^DESCRIPTION^MELEE_ANIMATION_GROUP`,
//!   so hand-to-hand (1) plays group 1000, the job rows (9..19, 22..24, 29, 30)
//!   play 1099 `com_swing`, bow (5) plays 1102 and thrown (6) has no melee group.
//! * `AnimationGroups.xml`: group `com_swing` id 1099 `fallback="1000"`, whose
//!   hand-to-hand slots are `com_h2h_attack_01..04`; group `com_h2h_attack` id 1000.
//! * `AnimationTypes.xml`: `com_h2h_attack_01` id 1001, type 2, priority 10,
//!   loadType 2, freezeMovement 0, doNotInterrupt 1, boneMaskSuffix `upperbody`,
//!   inside `AnimationTreeBranch name="Combat"`.
//! * `AnimationTransitionRules.xml`: `Any` -> `Combat` plays once (`numLoops="1"`)
//!   at `speed="1.0"`, `delay="0.0"`, `easeIn="0.2"`, `easeOut="0.2"`; the
//!   `Combat` -> `loc_stand` exit eases in over 0.6 s and out over 0.2 s.
//! * Reviewed native run `client-baseline-20261008T081219Z`: every one of the 20
//!   processed player hits follows a native `26/10` selection-103 request by
//!   less than 150 ms, the player swing animation is 1099 every time (the
//!   recovered Brawler row) and enemy attacks use the creature attack slot 1001.
//!
//! RECONSTRUCTED, labelled at each call site: the server-side reaches
//! (player 3.0 units, enemy 2.5 units), the 600 ms repeat gate and the pursuit
//! speed. The reviewed capture proves observed close hits, not every rejected
//! distance at the configured boundary.
//!
//! NOT CLAIMED: the original clip duration or hit frame window (a Granny GR2
//! read was not performed), original damage/balance, original action-bar
//! routing, or any native test owned by this module.

/// Reviewed reconstructed player melee reach in world units (metres).
///
/// This mirrors the production `profile_stats` reach rows in `src/encounter/combat_scale.rs`
/// (jobs 2/11/32/43 and the fallback all carry 3.0) and the accepted review
/// `notes/Reviews/Native Cave Completion 081219.md`.
pub const PLAYER_MELEE_REACH: f32 = 3.0;

/// Reviewed reconstructed enemy contact reach in world units.
pub const ENEMY_MELEE_REACH: f32 = 2.5;

/// Reconstructed minimum spacing between two accepted melee attacks.
pub const MELEE_REPEAT_GATE_MS: u64 = 600;

/// Measured bound between a native `26/10` selection-103 request and the
/// processed hit it produced in the reviewed capture (seconds converted to ms).
pub const OBSERVED_REQUEST_TO_HIT_BOUND_MS: u64 = 150;

/// Original client `com_swing` group that resolves the wield-type attack slot.
pub const COM_SWING_GROUP_ID: i32 = 1099;
/// Original `com_swing` fallback group from `AnimationGroups.xml`.
pub const COM_SWING_FALLBACK_ID: i32 = 1000;
/// Original `com_h2h_attack` group id.
pub const COM_H2H_ATTACK_GROUP_ID: i32 = 1000;
/// Original `com_h2h_attack_01` animation slot id.
pub const COM_H2H_ATTACK_01_SLOT_ID: i32 = 1001;

/// Which side of the reconstructed encounter issues the melee attack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatSide {
    Player,
    Enemy,
}

impl CombatSide {
    /// Reach used by this side. Both values are reconstructed; the player row is
    /// the reviewed 3 m melee rule.
    pub fn melee_reach(self) -> f32 {
        match self {
            CombatSide::Player => PLAYER_MELEE_REACH,
            CombatSide::Enemy => ENEMY_MELEE_REACH,
        }
    }
}

/// Why a melee request was refused. The production encounter server simply
/// drops refused attacks; this type keeps the refusal inspectable for tests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MeleeRefusal {
    /// A non-finite attacker or target coordinate.
    InvalidPosition,
    /// The target is already at zero health.
    DeadTarget,
    /// The target is beyond the side's reviewed reach.
    OutOfReach { distance: f32, reach: f32 },
    /// The previous accepted attack is inside [`MELEE_REPEAT_GATE_MS`].
    RepeatGate { since_last_ms: u64, minimum_ms: u64 },
}

/// An accepted melee attack.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeleeAttack {
    /// Solved three-dimensional distance the server measured.
    pub distance: f32,
    /// Animation the client plays: the recovered wield-type group for the
    /// player, the recovered creature attack slot for an enemy.
    pub swing_animation: i32,
}

/// Three-dimensional `f32` sum-of-squares then sqrt, identical to the
/// production `distance` helper in `src/encounter.rs`.
pub fn melee_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Recovered `WieldTypeAnimationMappings.txt` melee rows, in file order.
pub const WIELD_MELEE_ANIMATION_GROUPS: &[(u32, i32)] = &[
    (0, 0),
    (1, 1000),
    (2, 1020),
    (3, 1040),
    (4, 1060),
    (5, 1102),
    (6, 0),
    (7, 1133),
    (8, 1080),
    (9, 1099),
    (10, 1099),
    (11, 1099),
    (12, 1099),
    (13, 1099),
    (14, 1099),
    (15, 1099),
    (16, 1099),
    (17, 1099),
    (20, 0),
    (22, 1099),
    (23, 1099),
    (24, 1099),
    (27, 0),
    (28, 0),
    (29, 1099),
    (30, 1099),
];

/// Melee animation group the original client plays for a wield type.
///
/// Returns 0 for wield types whose recovered row has no melee group (unknown,
/// thrown, ultra-soaker, parachute, whip). Wield types outside the recovered
/// table are also 0: the original table does not cover them.
pub fn melee_animation_group(wield_type: u32) -> i32 {
    WIELD_MELEE_ANIMATION_GROUPS
        .iter()
        .find(|(row, _)| *row == wield_type)
        .map_or(0, |(_, group)| *group)
}

/// Animation played for an accepted attack of this side.
///
/// Players play their recovered wield-type melee group. Creatures play the
/// recovered hand-to-hand attack slot, matching the reviewed capture's
/// animation-1001 enemy attacks.
pub fn attack_animation(side: CombatSide, wield_type: u32) -> i32 {
    match side {
        CombatSide::Player => melee_animation_group(wield_type),
        CombatSide::Enemy => COM_H2H_ATTACK_01_SLOT_ID,
    }
}

/// One row of `AnimationTypes.xml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationSlotSpec {
    pub name: &'static str,
    pub id: i32,
    pub slot_type: u8,
    pub priority: u8,
    pub load_type: u8,
    pub freeze_movement: bool,
    pub do_not_interrupt: bool,
    pub bone_mask: &'static str,
}

/// `com_h2h_attack_01` row, re-read from the preserved original XML by
/// the private recovery checker `piece_054.py`.
pub const COM_H2H_ATTACK_01_SLOT: AnimationSlotSpec = AnimationSlotSpec {
    name: "com_h2h_attack_01",
    id: COM_H2H_ATTACK_01_SLOT_ID,
    slot_type: 2,
    priority: 10,
    load_type: 2,
    freeze_movement: false,
    do_not_interrupt: true,
    bone_mask: "upperbody",
};

/// Alias kept short for callers that already imported the group constant.
pub const H2H_ATTACK_01_SLOT: AnimationSlotSpec = COM_H2H_ATTACK_01_SLOT;

/// One timing row of `AnimationTransitionRules.xml`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombatAnimationTiming {
    /// -1 loops forever, 1 is a single play; the attack rows are single plays.
    pub num_loops: i32,
    pub speed: f32,
    pub delay_seconds: f32,
    pub ease_in_seconds: f32,
    pub ease_out_seconds: f32,
}

/// `Any` -> `Combat`: a combat animation starts immediately, plays once at
/// normal speed and blends in/out over 0.2 s.
pub const ANY_TO_COMBAT_TIMING: CombatAnimationTiming = CombatAnimationTiming {
    num_loops: 1,
    speed: 1.0,
    delay_seconds: 0.0,
    ease_in_seconds: 0.2,
    ease_out_seconds: 0.2,
};

/// `Combat` -> `loc_stand`: leaving the combat animation blends in over 0.6 s.
pub const COMBAT_TO_STAND_TIMING: CombatAnimationTiming = CombatAnimationTiming {
    num_loops: -1,
    speed: 1.0,
    delay_seconds: 0.0,
    ease_in_seconds: 0.6,
    ease_out_seconds: 0.2,
};

/// Decide one melee attack against the reviewed/reconstructed rules.
///
/// The gate mirrors the production encounter server: a target at exactly the
/// reach is accepted (`> reach` is refused), a zero-health target is refused,
/// and a second accepted attack inside [`MELEE_REPEAT_GATE_MS`] is refused.
/// Positions must be finite. The caller owns damage, rewards and packets.
#[allow(clippy::too_many_arguments)]
pub fn resolve_melee_attack(
    side: CombatSide,
    attacker_position: [f32; 3],
    target_position: [f32; 3],
    target_health: i32,
    now_ms: u64,
    last_attack_ms: Option<u64>,
    wield_type: u32,
) -> Result<MeleeAttack, MeleeRefusal> {
    if !attacker_position.iter().all(|value| value.is_finite())
        || !target_position.iter().all(|value| value.is_finite())
    {
        return Err(MeleeRefusal::InvalidPosition);
    }
    if target_health <= 0 {
        return Err(MeleeRefusal::DeadTarget);
    }
    let reach = side.melee_reach();
    let distance = melee_distance(attacker_position, target_position);
    if distance > reach {
        return Err(MeleeRefusal::OutOfReach { distance, reach });
    }
    if let Some(last) = last_attack_ms {
        let since_last_ms = now_ms.saturating_sub(last);
        if since_last_ms < MELEE_REPEAT_GATE_MS {
            return Err(MeleeRefusal::RepeatGate {
                since_last_ms,
                minimum_ms: MELEE_REPEAT_GATE_MS,
            });
        }
    }
    Ok(MeleeAttack {
        distance,
        swing_animation: attack_animation(side, wield_type),
    })
}
