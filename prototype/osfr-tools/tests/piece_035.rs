//! Item 035: Archer progression - the recovered ranged-action wire and the bounded reach gate.
//!
//! These vectors run against `src/restoration/piece_035.rs`, which is not yet registered in
//! `src/lib.rs` (lead-owned), so the owned target includes the exact source file by `#[path]`.
//!
//! RED evidence: this file was written before the module existed and the first run failed to
//! compile with `error[E0583]: file not found for module`, retained in this item's lab report.
//!
//! ORIGINAL evidence behind the constants and layouts asserted here (all outside Git):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the supplied client's own sub-opcode
//!   immediates: `AbilityPacketStartCasting` is built with `3` (`FUN_00a2ed30`),
//!   `AbilityPacketDetonateProjectile` with `0xe` (`FUN_00a2ee30`), and
//!   `PlayerUpdateLaunchProjectilePacket` with `0x3e` (`FUN_008d0b10`).
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
//!   and `lab/references/OpenSourceFreeRealms/.../PacketCodes.cs` - the ability family (36) and the
//!   PlayerUpdate sub-code 62 name tables.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet.Common/Ability.cs` - the toolbar slot layout,
//!   including `float Range`, which PR 121 documents as reverse-engineered from the client and used
//!   as the auto-target radius.
//! * `lab/working/OSFR.Client/OSFR Client/Resources/CodeStringMappings.txt` - the `NoCast*`
//!   string-id vocabulary an `AbilityPacketFailed` frame carries.
//!
//! RECONSTRUCTED: the reach gate, the line-of-sight predicate and the ammo policy below. The
//! retail server's ability handler is not recovered (no ability capture exists in the lab and the
//! pinned community build ships serializers only), so those three are ours and are labelled.

#[path = "../src/restoration/piece_035.rs"]
#[allow(dead_code)]
mod piece_035;

use piece_035::{
    ABILITY_FAMILY, ABILITY_SLOT_COUNT, ABILITY_SLOT_TYPE_ABILITY, ABILITY_SLOT_TYPE_EMPTY,
    ABILITY_SLOT_TYPE_ITEM, ARCHER_AUTO_TARGET_REACH, ARCHER_BASIC_ABILITY_ID,
    ARCHER_BASIC_RECAST_MS, ARCHER_ENERGY_MAX, ARCHER_ENERGY_REGEN_PER_SECOND,
    ARCHER_FALLBACK_BASIC_ABILITY_ID, ARCHER_PROFILE_ID, ARCHER_SPECIAL_ABILITY_ID,
    ARCHER_STARTER_BOW_ITEM_ID, ARCHER_WIELD_TYPE, AmmoPolicy, ArcherKit, LineOfSight,
    NoCastReason, PLAYER_UPDATE_FAMILY, ProjectilePayload, RangedError, RangedTarget, ReachGate,
    SUB_ABILITY_DEFINITION, SUB_CLIENT_REQUEST_START_ABILITY, SUB_DETONATE_PROJECTILE, SUB_FAILED,
    SUB_LAUNCH_AND_LAND, SUB_LAUNCH_PROJECTILE, SUB_SET_DEFINITION, SUB_START_CASTING,
    StartAbilityRequest, StartCasting, Toolbar, WIZARD_AUTO_TARGET_REACH, admit_ranged_action,
};

/// `Ability.Serialize` for a Type-3 slot: nine ints, the fourth of which the client reads as a
/// float, and the trailing `ForceDismount` byte.
fn slot_ability(values: [i32; 10], range: f32) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (index, value) in values.into_iter().enumerate() {
        if index == 6 {
            bytes.extend(range.to_le_bytes());
        } else {
            bytes.extend(value.to_le_bytes());
        }
    }
    bytes.push(1); // ForceDismount
    bytes
}

fn toolbar_frame(profile_id: i32, slots: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![36, 0, 5, 0];
    bytes.extend(profile_id.to_le_bytes());
    bytes.extend((slots.len() as i32).to_le_bytes());
    for slot in slots {
        bytes.extend(slot);
    }
    bytes
}

#[test]
fn recovered_family_and_sub_opcodes_match_the_client_and_the_pinned_readers() {
    assert_eq!(ABILITY_FAMILY, 36);
    assert_eq!(PLAYER_UPDATE_FAMILY, 35);
    assert_eq!(SUB_FAILED, 1);
    assert_eq!(SUB_START_CASTING, 3);
    assert_eq!(SUB_LAUNCH_AND_LAND, 4);
    assert_eq!(SUB_SET_DEFINITION, 5);
    assert_eq!(SUB_CLIENT_REQUEST_START_ABILITY, 10);
    assert_eq!(SUB_ABILITY_DEFINITION, 13);
    assert_eq!(SUB_DETONATE_PROJECTILE, 14);
    assert_eq!(SUB_LAUNCH_PROJECTILE, 62);
    assert_eq!(ABILITY_SLOT_COUNT, 8);
    assert_eq!(ABILITY_SLOT_TYPE_EMPTY, 0);
    assert_eq!(ABILITY_SLOT_TYPE_ITEM, 2);
    assert_eq!(ABILITY_SLOT_TYPE_ABILITY, 3);
}

#[test]
fn archer_kit_is_the_recovered_ranged_kit_and_outreaches_the_melee_kits() {
    let kit = ArcherKit::RECOVERED;
    assert_eq!(kit.profile_id, ARCHER_PROFILE_ID);
    assert_eq!(kit.wield_type, ARCHER_WIELD_TYPE);
    assert_eq!(kit.auto_target_reach, ARCHER_AUTO_TARGET_REACH);
    assert_eq!(kit.starter_bow_item_id, ARCHER_STARTER_BOW_ITEM_ID);
    assert_eq!(kit.basic_ability_id, ARCHER_BASIC_ABILITY_ID);
    assert_eq!(kit.special_ability_id, ARCHER_SPECIAL_ABILITY_ID);
    assert_eq!(
        kit.fallback_basic_ability_id,
        ARCHER_FALLBACK_BASIC_ABILITY_ID
    );
    assert_eq!(kit.basic_recast_ms, ARCHER_BASIC_RECAST_MS);
    assert_eq!(kit.energy_max, ARCHER_ENERGY_MAX);
    assert_eq!(kit.energy_regen_per_second, ARCHER_ENERGY_REGEN_PER_SECOND);
    assert!(ARCHER_AUTO_TARGET_REACH > WIZARD_AUTO_TARGET_REACH);
    assert!(
        WIZARD_AUTO_TARGET_REACH > 7.0,
        "every melee kit is documented at 7u"
    );
}

#[test]
fn start_ability_request_decodes_a_position_target() {
    let mut bytes = vec![36, 0, 10, 0];
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(3_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    for value in [12.5_f32, 0.0, -8.25, 1.0] {
        bytes.extend(value.to_le_bytes());
    }

    let request = StartAbilityRequest::decode(&bytes).unwrap();
    assert_eq!(request.action_bar_id, 1);
    assert_eq!(request.slot, 3);
    assert_eq!(
        request.target,
        RangedTarget::Position([12.5, 0.0, -8.25, 1.0])
    );
}

#[test]
fn start_ability_request_decodes_a_guid_target() {
    let mut bytes = vec![36, 0, 10, 0];
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(0x5255_5354_0000_0007_u64.to_le_bytes());

    let request = StartAbilityRequest::decode(&bytes).unwrap();
    assert_eq!(request.action_bar_id, 0);
    assert_eq!(request.slot, 0);
    assert_eq!(request.target, RangedTarget::Guid(0x5255_5354_0000_0007));
}

#[test]
fn start_ability_request_rejects_wrong_family_truncation_and_trailing_bytes() {
    let mut good = vec![36, 0, 10, 0];
    good.extend(0_i32.to_le_bytes());
    good.extend(0_i32.to_le_bytes());
    good.extend(0_i32.to_le_bytes());
    good.extend(7_u64.to_le_bytes());
    assert!(StartAbilityRequest::decode(&good).is_ok());

    let mut wrong_family = good.clone();
    wrong_family[1] = 1;
    assert_eq!(
        StartAbilityRequest::decode(&wrong_family).unwrap_err(),
        RangedError::WrongFamily
    );

    let mut wrong_sub = good.clone();
    wrong_sub[2] = 9;
    assert_eq!(
        StartAbilityRequest::decode(&wrong_sub).unwrap_err(),
        RangedError::WrongSubOpcode
    );

    for cut in [4, 8, 20, 23] {
        let truncated = &good[..cut];
        assert_eq!(
            StartAbilityRequest::decode(truncated).unwrap_err(),
            RangedError::Truncated,
            "cut at {cut}"
        );
    }

    let mut trailing = good.clone();
    trailing.push(0);
    assert_eq!(
        StartAbilityRequest::decode(&trailing).unwrap_err(),
        RangedError::TrailingBytes
    );
}

#[test]
fn toolbar_decodes_eight_positional_slots_and_surfaces_the_range_float() {
    let mut slots = Vec::new();
    slots.push(ABILITY_SLOT_TYPE_EMPTY.to_le_bytes().to_vec());
    slots.push(slot_ability(
        [
            ABILITY_SLOT_TYPE_ABILITY,
            ARCHER_BASIC_ABILITY_ID,
            0,
            14134,
            420256,
            4,
            0,
            1,
            ARCHER_BASIC_ABILITY_ID,
            0,
        ],
        ARCHER_AUTO_TARGET_REACH,
    ));
    for _ in 0..6 {
        slots.push(ABILITY_SLOT_TYPE_EMPTY.to_le_bytes().to_vec());
    }

    let frame = toolbar_frame(ARCHER_PROFILE_ID, &slots);
    let toolbar = Toolbar::decode(&frame).unwrap();
    assert_eq!(toolbar.profile_id, ARCHER_PROFILE_ID);
    assert_eq!(toolbar.slots.len(), ABILITY_SLOT_COUNT);
    assert!(toolbar.slots[0].is_none());
    let basic = toolbar.slots[1].as_ref().unwrap();
    assert_eq!(basic.ability_definition_id, ARCHER_BASIC_ABILITY_ID);
    assert_eq!(basic.icon_id, 14134);
    assert_eq!(basic.range, ARCHER_AUTO_TARGET_REACH);
    assert_eq!(basic.force_dismount, 1);
    assert!(toolbar.slots[2..].iter().all(Option::is_none));
}

#[test]
fn toolbar_rejects_a_wrong_slot_count_a_bad_profile_and_short_frames() {
    let slots: Vec<Vec<u8>> = (0..ABILITY_SLOT_COUNT)
        .map(|_| ABILITY_SLOT_TYPE_EMPTY.to_le_bytes().to_vec())
        .collect();

    let short_count = toolbar_frame(ARCHER_PROFILE_ID, &slots[..ABILITY_SLOT_COUNT - 1]);
    assert_eq!(
        Toolbar::decode(&short_count).unwrap_err(),
        RangedError::BadSlotCount
    );

    let mut truncated = toolbar_frame(ARCHER_PROFILE_ID, &slots);
    truncated.truncate(truncated.len() - 4);
    assert_eq!(
        Toolbar::decode(&truncated).unwrap_err(),
        RangedError::Truncated
    );

    let bad_profile = toolbar_frame(0, &slots);
    assert_eq!(
        Toolbar::decode(&bad_profile).unwrap_err(),
        RangedError::BadProfile
    );
}

#[test]
fn ability_failure_uses_the_recovered_no_cast_vocabulary() {
    for (reason, string_id) in [
        (NoCastReason::KnockedOut, 280),
        (NoCastReason::Asleep, 281),
        (NoCastReason::Silenced, 282),
        (NoCastReason::Afraid, 283),
        (NoCastReason::Stunned, 284),
        (NoCastReason::InsufficientMana, 285),
        (NoCastReason::Default, 286),
        (NoCastReason::RecastTimer, 287),
        (NoCastReason::Targeting, 288),
        (NoCastReason::NotEnabled, 3079),
    ] as [(NoCastReason, i32); 10]
    {
        let mut expected = vec![36, 0, 1, 0];
        expected.extend(string_id.to_le_bytes());
        assert_eq!(reason.string_id(), string_id);
        assert_eq!(piece_035::encode_failure(reason), expected, "{reason:?}");
    }
}

#[test]
fn start_casting_frame_matches_the_client_constructor_defaults() {
    let cast = StartCasting {
        caster_guid: 0x1000_0000_0000_0001,
        target_guid: 0x1000_0000_0000_0002,
        composite_effect_id: 0,
        animation: -1,
        ability_id: ARCHER_BASIC_ABILITY_ID,
        action_time: 0.6,
        has_action_progress: false,
    };
    let bytes = piece_035::encode_start_casting(&cast);
    assert_eq!(&bytes[..4], &[36, 0, 3, 0]);
    assert_eq!(bytes.len(), 37);
    assert_eq!(&bytes[4..12], &0x1000_0000_0000_0001_u64.to_le_bytes());
    assert_eq!(&bytes[12..20], &0x1000_0000_0000_0002_u64.to_le_bytes());
    assert_eq!(&bytes[20..24], &0_i32.to_le_bytes());
    assert_eq!(&bytes[24..28], &(-1_i32).to_le_bytes());
    assert_eq!(&bytes[28..32], &ARCHER_BASIC_ABILITY_ID.to_le_bytes());
    assert_eq!(&bytes[32..36], &0.6_f32.to_le_bytes());
    assert_eq!(bytes[36], 0);
}

#[test]
fn reach_gate_admits_the_documented_ranged_reach_and_refuses_beyond_it() {
    let gate = piece_035::ReachGate::for_archer();
    let origin = [0.0_f32, 0.0, 0.0, 0.0];

    let inside = [ARCHER_AUTO_TARGET_REACH - 0.01, 0.0, 0.0, 0.0];
    assert_eq!(gate.admit_position(origin, inside), Ok(()));

    let exactly = [ARCHER_AUTO_TARGET_REACH, 0.0, 0.0, 0.0];
    assert_eq!(gate.admit_position(origin, exactly), Ok(()));

    let beyond = [ARCHER_AUTO_TARGET_REACH + 0.01, 0.0, 0.0, 0.0];
    assert_eq!(
        gate.admit_position(origin, beyond),
        Err(NoCastReason::Targeting)
    );
}

#[test]
fn line_of_sight_is_opt_in_and_never_fabricates_a_retail_rule() {
    let origin = [0.0_f32, 0.0, 0.0, 0.0];
    let target = [6.0_f32, 0.0, 0.0, 0.0];
    let wall = [[2.0_f32, -1.0, -1.0, 0.0], [4.0_f32, 1.0, 1.0, 0.0]];

    // With no supplied geometry the recovered evidence cannot justify a refusal.
    assert_eq!(LineOfSight::Unchecked.admit(origin, target), Ok(()));

    // With caller-supplied blockers the reconstructed predicate refuses the blocked segment...
    let blocked = LineOfSight::SegmentBlockers(&[wall]);
    assert_eq!(blocked.admit(origin, target), Err(NoCastReason::Targeting));

    // ...and still admits a segment the blockers do not cover.
    assert_eq!(blocked.admit(origin, [1.0, 0.0, 0.0, 0.0]), Ok(()));
    assert_eq!(blocked.admit(origin, [-6.0, 0.0, 0.0, 0.0]), Ok(()));
    let aside = LineOfSight::SegmentBlockers(&[wall]);
    assert_eq!(aside.admit(origin, [6.0, 4.0, 0.0, 0.0]), Ok(()));
}

#[test]
fn composed_gate_checks_reach_then_geometry_and_refuses_non_finite_positions() {
    let gate = ReachGate::for_archer();
    let origin = [0.0_f32, 0.0, 0.0, 0.0];
    let wall = [[1.0_f32, -1.0, -1.0, 0.0], [2.0_f32, 1.0, 1.0, 0.0]];
    let sight = LineOfSight::SegmentBlockers(&[wall]);

    assert_eq!(
        admit_ranged_action(origin, [3.0, 0.0, 0.0, 0.0], &gate, sight),
        Err(NoCastReason::Targeting),
        "a wall inside the reach refuses the shot"
    );
    assert_eq!(
        admit_ranged_action(origin, [0.5, 0.0, 0.0, 0.0], &gate, sight),
        Ok(()),
        "the same wall does not reach a nearer target"
    );
    assert_eq!(
        admit_ranged_action(
            origin,
            [100.0, 0.0, 0.0, 0.0],
            &gate,
            LineOfSight::Unchecked
        ),
        Err(NoCastReason::Targeting),
        "reach applies even without geometry"
    );
    assert_eq!(
        admit_ranged_action(
            origin,
            [f32::NAN, 0.0, 0.0, 0.0],
            &gate,
            LineOfSight::Unchecked
        ),
        Err(NoCastReason::Targeting)
    );
    assert_eq!(
        LineOfSight::Unchecked.admit([f32::INFINITY, 0.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]),
        Err(NoCastReason::Targeting),
        "an unchecked policy still refuses to reason about a non-finite position"
    );
}

#[test]
fn no_ammo_is_consumed_because_no_ammo_economy_was_recovered() {
    assert_eq!(AmmoPolicy::RECOVERED, AmmoPolicy::NotRecovered);
    assert!(!AmmoPolicy::NotRecovered.charges_items());
    assert!(!AmmoPolicy::NotRecovered.requires_equipped_ammo());
}

#[test]
fn non_finite_input_is_refused_before_any_policy_runs() {
    let mut bytes = vec![36, 0, 10, 0];
    bytes.extend(1_i32.to_le_bytes());
    bytes.extend(0_i32.to_le_bytes());
    bytes.extend(1_i32.to_le_bytes());
    for value in [f32::NAN, 0.0, 0.0, 0.0] {
        bytes.extend(value.to_le_bytes());
    }
    assert_eq!(
        StartAbilityRequest::decode(&bytes).unwrap_err(),
        RangedError::NonFinite
    );

    assert_eq!(
        ReachGate::new(f32::INFINITY).unwrap_err(),
        RangedError::NonFinite
    );
    assert_eq!(ReachGate::new(0.0).unwrap_err(), RangedError::NonFinite);
    assert_eq!(ReachGate::new(-1.0).unwrap_err(), RangedError::NonFinite);

    let gate = ReachGate::for_archer();
    assert_eq!(
        gate.admit_position([0.0, 0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0, 0.0]),
        Err(NoCastReason::Targeting)
    );

    let mut toolbar = vec![36, 0, 5, 0];
    toolbar.extend(ARCHER_PROFILE_ID.to_le_bytes());
    toolbar.extend((ABILITY_SLOT_COUNT as i32).to_le_bytes());
    toolbar.extend(slot_ability(
        [
            ABILITY_SLOT_TYPE_ABILITY,
            ARCHER_BASIC_ABILITY_ID,
            0,
            1,
            1,
            4,
            0,
            1,
            ARCHER_BASIC_ABILITY_ID,
            0,
        ],
        f32::NAN,
    ));
    for _ in 1..ABILITY_SLOT_COUNT {
        toolbar.extend(ABILITY_SLOT_TYPE_EMPTY.to_le_bytes());
    }
    assert_eq!(
        Toolbar::decode(&toolbar).unwrap_err(),
        RangedError::NonFinite
    );
}

#[test]
fn projectile_subs_are_known_but_their_payloads_stay_unrecovered() {
    assert_eq!(
        ProjectilePayload::launch_and_land(),
        ProjectilePayload::Unrecovered(SUB_LAUNCH_AND_LAND)
    );
    assert_eq!(
        ProjectilePayload::detonate(),
        ProjectilePayload::Unrecovered(SUB_DETONATE_PROJECTILE)
    );
    assert_eq!(
        ProjectilePayload::player_update_launch(),
        ProjectilePayload::Unrecovered(SUB_LAUNCH_PROJECTILE)
    );
    assert!(!ProjectilePayload::Unrecovered(SUB_DETONATE_PROJECTILE).has_recovered_layout());
}
