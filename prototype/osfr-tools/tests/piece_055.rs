//! Item 055: Ranged enemy attacks - cast/projectile vocabulary and the enemy cast lifecycle.
//!
//! These vectors run against `src/restoration/piece_055.rs`, which is not yet registered in
//! `src/lib.rs` (lead-owned), so the owned target includes the exact source file by `#[path]`.
//!
//! RED evidence: this file was written before the module existed; the first run failed with
//! `error[E0583]: file not found for module 'piece_055'` and the output is retained in this
//! item's lab report.
//!
//! ORIGINAL evidence behind the constants asserted here (all outside Git):
//! * `lab/decompiled/server/LibSOE/SOE/BaseCombatPackets.cs` and
//!   `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` - the sub-opcode tables.
//! * `lab/references/OpenSourceFreeRealms/ofrserver/Server/LibSOE/Core/PacketCodes.cs` - the
//!   family ids `BaseCombatPacket` 32, `BasePlayerUpdatePacket` 35, `BaseAbilityPacket` 36.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the supplied client's own constructor
//!   immediates: `AbilityPacketStartCasting` 3 (`FUN_00a2ed30`),
//!   `AbilityPacketDetonateProjectile` 0xe (`FUN_00a2ee30`),
//!   `PlayerUpdateLaunchProjectilePacket` 0x3e (`FUN_008d0b10`) and the seven combat packet
//!   constructors 1/3/4/5/6/7/9. The same file's `FUN_008d00e0` initialises the embedded
//!   `ProjectileParameters` with the `_DAT_0175b420` dword (`0x3f800000` = 1.0f) at +0x20/+0x24.
//!
//! RECONSTRUCTED: every timing, range and resolution rule the state machine enforces. No retail
//! enemy cast capture exists in the lab, so no original cast time, projectile speed or enemy reach
//! is claimed here; the caller supplies them and the module only sequences the recovered frames.

#[path = "../src/restoration/piece_055.rs"]
#[allow(dead_code)]
mod piece_055;

use piece_055::{
    ABILITY_FAMILY, ABILITY_SUB_CAST_INTERRUPT, ABILITY_SUB_DETONATE_PROJECTILE,
    ABILITY_SUB_FAILED, ABILITY_SUB_LAUNCH_AND_LAND, ABILITY_SUB_START_CASTING, COMBAT_FAMILY,
    COMBAT_SUB_ATTACK_PROCESSED, COMBAT_SUB_ATTACK_TARGET_DAMAGE, COMBAT_SUB_ATTACKER_MISSED,
    COMBAT_SUB_AUTO_ATTACK_TARGET, COMBAT_SUB_ENABLE_BOSS_DISPLAY, COMBAT_SUB_SINGLE_ATTACK_TARGET,
    COMBAT_SUB_TARGET_DODGED, CastError, CombatResolution, DETONATE_ID_DWORD_COUNT,
    DETONATE_INT32_FIELD_COUNT, EnemyAttackIntent, EnemyCastProfile, EnemyCastState,
    NO_CAST_TARGETING_ID, PLAYER_UPDATE_FAMILY, PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE,
    PROJECTILE_PARAMETERS_DWORD_UNIT_A, PROJECTILE_PARAMETERS_DWORD_UNIT_B,
    PROJECTILE_PARAMETERS_UNIT_DEFAULT, RangedEnemyAttack,
};

const CASTER: u64 = 0x5255_5354_0000_0101;
const TARGET: u64 = 0x5255_5354_0000_0202;

fn profile(cast_ms: u32, flight_ms: u32, range: f32) -> EnemyCastProfile {
    EnemyCastProfile {
        ability_id: 1010,
        cast_time_ms: cast_ms,
        flight_time_ms: flight_ms,
        range_units: range,
    }
}

/// The families and sub-opcodes are recovered data; a drift here means the client vocabulary
/// changed under the module and every intent below is no longer trustworthy.
#[test]
fn recovered_packet_vocabulary_is_pinned() {
    assert_eq!(COMBAT_FAMILY, 32);
    assert_eq!(COMBAT_SUB_AUTO_ATTACK_TARGET, 1);
    assert_eq!(COMBAT_SUB_SINGLE_ATTACK_TARGET, 3);
    assert_eq!(COMBAT_SUB_ATTACK_TARGET_DAMAGE, 4);
    assert_eq!(COMBAT_SUB_ATTACKER_MISSED, 5);
    assert_eq!(COMBAT_SUB_TARGET_DODGED, 6);
    assert_eq!(COMBAT_SUB_ATTACK_PROCESSED, 7);
    assert_eq!(COMBAT_SUB_ENABLE_BOSS_DISPLAY, 9);
    assert_eq!(ABILITY_FAMILY, 36);
    assert_eq!(ABILITY_SUB_FAILED, 1);
    assert_eq!(ABILITY_SUB_START_CASTING, 3);
    assert_eq!(ABILITY_SUB_LAUNCH_AND_LAND, 4);
    assert_eq!(ABILITY_SUB_DETONATE_PROJECTILE, 14);
    assert_eq!(ABILITY_SUB_CAST_INTERRUPT, 18);
    assert_eq!(PLAYER_UPDATE_FAMILY, 35);
    assert_eq!(PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE, 62);
    // `AbilityPacketFailed` carries `NoCastTargeting` when retail has no range message.
    assert_eq!(NO_CAST_TARGETING_ID, 288);
}

/// The partial `ProjectileParameters` / `DetonateProjectile` shapes recovered from the client
/// constructors, pinned so a later layout decode can extend rather than silently replace them.
#[test]
fn recovered_client_constructor_shapes_are_pinned() {
    assert_eq!(PROJECTILE_PARAMETERS_DWORD_UNIT_A, 8);
    assert_eq!(PROJECTILE_PARAMETERS_DWORD_UNIT_B, 9);
    assert_eq!(PROJECTILE_PARAMETERS_UNIT_DEFAULT, 1.0);
    assert_eq!(DETONATE_ID_DWORD_COUNT, 2);
    assert_eq!(DETONATE_INT32_FIELD_COUNT, 3);
}

#[test]
fn out_of_range_target_never_enters_the_cast() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1500, 400, 30.0));
    assert_eq!(attack.begin_cast(1_000, 30.5), Err(CastError::OutOfRange));
    assert_eq!(attack.state(), EnemyCastState::Idle);
    assert_eq!(attack.advance(9_999), Ok(None));
    // Non-finite input is rejected rather than admitted.
    assert_eq!(attack.begin_cast(1_000, f32::NAN), Err(CastError::Invalid));
}

#[test]
fn cast_launches_at_the_action_time_and_not_before() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1500, 400, 30.0));
    assert_eq!(
        attack.begin_cast(1_000, 12.0),
        Ok(EnemyAttackIntent::StartCasting {
            caster: CASTER,
            target: TARGET,
            ability_id: 1010,
            action_time_ms: 1500,
            has_action_progress: true,
        })
    );
    assert!(matches!(attack.state(), EnemyCastState::Casting { .. }));
    assert_eq!(attack.advance(2_499), Ok(None));
    assert_eq!(
        attack.advance(2_500),
        Ok(Some(EnemyAttackIntent::LaunchProjectile {
            caster: CASTER,
            target: TARGET,
        }))
    );
    assert_eq!(attack.advance(2_600), Ok(None));
}

#[test]
fn zero_flight_time_detonates_without_a_travel_intent() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(500, 0, 20.0));
    attack.begin_cast(100, 5.0).unwrap();
    assert_eq!(
        attack.advance(600),
        Ok(Some(EnemyAttackIntent::DetonateProjectile {
            caster: CASTER,
            target: TARGET,
        }))
    );
}

#[test]
fn detonation_lands_only_after_the_flight_time() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1_000, 750, 30.0));
    attack.begin_cast(0, 10.0).unwrap();
    assert!(matches!(
        attack.advance(1_000),
        Ok(Some(EnemyAttackIntent::LaunchProjectile { .. }))
    ));
    assert_eq!(attack.advance(1_749), Ok(None));
    assert_eq!(
        attack.advance(1_750),
        Ok(Some(EnemyAttackIntent::DetonateProjectile {
            caster: CASTER,
            target: TARGET,
        }))
    );
}

#[test]
fn resolution_applies_once_and_maps_to_the_recovered_combat_frames() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(0, 0, 30.0));
    attack.begin_cast(0, 4.0).unwrap();
    attack.advance(0).unwrap();
    assert_eq!(
        attack.resolve(CombatResolution::Damage),
        Ok(EnemyAttackIntent::AttackResolution {
            attacker: CASTER,
            target: TARGET,
            resolution: CombatResolution::Damage,
        })
    );
    assert_eq!(attack.state(), EnemyCastState::Resolved);
    assert_eq!(
        attack.resolve(CombatResolution::Damage),
        Err(CastError::AlreadyResolved)
    );

    let mut dodged = RangedEnemyAttack::new(CASTER, TARGET, profile(0, 0, 30.0));
    dodged.begin_cast(0, 4.0).unwrap();
    dodged.advance(0).unwrap();
    assert_eq!(
        dodged.resolve(CombatResolution::Dodged),
        Ok(EnemyAttackIntent::AttackResolution {
            attacker: CASTER,
            target: TARGET,
            resolution: CombatResolution::Dodged,
        })
    );
}

#[test]
fn resolving_before_detonation_is_refused() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1_000, 0, 30.0));
    attack.begin_cast(0, 4.0).unwrap();
    assert_eq!(
        attack.resolve(CombatResolution::Damage),
        Err(CastError::NotDetonated)
    );
}

#[test]
fn interrupt_is_only_possible_while_casting() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1_000, 100, 30.0));
    attack.begin_cast(0, 4.0).unwrap();
    assert_eq!(
        attack.interrupt(),
        Ok(EnemyAttackIntent::CastInterrupt { caster: CASTER })
    );
    assert_eq!(attack.state(), EnemyCastState::Idle);
    assert_eq!(attack.interrupt(), Err(CastError::NotCasting));
    // The same caster can restart after an interrupt.
    assert!(attack.begin_cast(10, 4.0).is_ok());
    attack.advance(1_010).unwrap();
    assert_eq!(attack.interrupt(), Err(CastError::NotCasting));
}

#[test]
fn a_second_cast_cannot_start_while_one_is_active() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(1_000, 0, 30.0));
    attack.begin_cast(0, 4.0).unwrap();
    assert_eq!(attack.begin_cast(1, 4.0), Err(CastError::Busy));
}

#[test]
fn timing_overflow_is_refused_instead_of_wrapping() {
    let mut attack = RangedEnemyAttack::new(CASTER, TARGET, profile(u32::MAX, 0, 30.0));
    assert_eq!(
        attack.begin_cast(u64::MAX - 10, 1.0),
        Err(CastError::Invalid)
    );
}

/// Every intent must be dispatchable to a recovered frame, and the three resolutions must land on
/// the three distinct family-32 frames rather than collapsing into one.
#[test]
fn intents_map_to_recovered_frames() {
    assert_eq!(
        EnemyAttackIntent::StartCasting {
            caster: CASTER,
            target: TARGET,
            ability_id: 1,
            action_time_ms: 0,
            has_action_progress: false,
        }
        .frame(),
        (ABILITY_FAMILY, ABILITY_SUB_START_CASTING)
    );
    assert_eq!(
        EnemyAttackIntent::LaunchProjectile {
            caster: CASTER,
            target: TARGET,
        }
        .frame(),
        (PLAYER_UPDATE_FAMILY, PLAYER_UPDATE_SUB_LAUNCH_PROJECTILE)
    );
    assert_eq!(
        EnemyAttackIntent::DetonateProjectile {
            caster: CASTER,
            target: TARGET,
        }
        .frame(),
        (ABILITY_FAMILY, ABILITY_SUB_DETONATE_PROJECTILE)
    );
    assert_eq!(
        EnemyAttackIntent::CastInterrupt { caster: CASTER }.frame(),
        (ABILITY_FAMILY, ABILITY_SUB_CAST_INTERRUPT)
    );
    for (resolution, sub) in [
        (CombatResolution::Damage, COMBAT_SUB_ATTACK_TARGET_DAMAGE),
        (CombatResolution::Missed, COMBAT_SUB_ATTACKER_MISSED),
        (CombatResolution::Dodged, COMBAT_SUB_TARGET_DODGED),
    ] {
        assert_eq!(resolution.sub_opcode(), sub);
        assert_eq!(
            EnemyAttackIntent::AttackResolution {
                attacker: CASTER,
                target: TARGET,
                resolution,
            }
            .frame(),
            (COMBAT_FAMILY, sub)
        );
    }
}
