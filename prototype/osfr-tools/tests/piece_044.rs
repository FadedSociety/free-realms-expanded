//! Item 044: class switching wire - strict request parsing, the recovered failure
//! command, and the preservation certificate for health, gear and abilities.
//!
//! These vectors run against `src/restoration/piece_044.rs`, which is not yet registered in
//! `src/lib.rs` (lead-owned), so the owned target includes the exact source file by `#[path]`.
//!
//! RED evidence: this file was written before the module existed, so its first run failed to
//! compile with `error[E0583]: file not found for module`; the log is retained in this item's
//! lab report.
//!
//! ORIGINAL evidence behind the constants and layouts asserted here (all outside Git):
//! * `lab/decompiled/server/LibSOE/SOE/BaseCommandPackets.cs:7,38,39` - `CommandPacketSetProfile
//!   = 13`, `CommandPacketActivateProfileFailed = 26`, `CommandPacketSetProfileByItemDefinitionId
//!   = 36` under the recovered `BaseCommandPacket` family, which the sibling enum in
//!   `BasePackets.cs` pins to opcode 26.
//! * `lab/decompiled/server/LibSOE/SOE/BaseClientUpdatePackets.cs:1,21` - `ClientUpdatePacketHitpoints
//!   = 1` and `ClientUpdatePacketActivateProfile = 21` under `BaseClientUpdatePacket` = 38.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - the supplied client's own constructors:
//!   `CommandPacketSetProfile` is built with immediate `0xd` (`FUN_008a28e0`),
//!   `CommandPacketSetProfileByItemDefinitionId` with `0x24` (`FUN_008a2b20`),
//!   `CommandPacketActivateProfileFailed` with `0x1a` (`FUN_008a2ca0`) and
//!   `ClientUpdatePacketActivateProfile` with `0x15` (`FUN_0091c05c`). The shared base constructor
//!   `FUN_008a2820` stores the command family `0x1a` in `BasePacket` and the immediate in
//!   `BaseCommandPacket`, so the command frame is `[26,0][sub,0][fields...]`.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` `FUN_008a2ca0` - the failure command
//!   initialises exactly two `int` fields beyond the base command packet.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs:101,114,122`
//!   - the pinned community build names the same three command sub-opcodes, and names
//!   `ClientUpdatePacketHitpoints` and `ClientUpdatePacketActivateProfile` at 328 and 323.
//! * `lab/references/Sanctuary/src/Sanctuary.Packet/BaseCommandPacket/CommandPacketSetProfile.cs:20`
//!   - the community reader accepts `Id` only and requires `RemainingLength == 0`, so the request
//!   carries exactly one `int32`.
//! * `lab/references/Sanctuary/src/Sanctuary.Gateway/Handlers/BaseCommandPacket/CommandPacketSetProfileHandler.cs:45-60`
//!   - the recovered flow: a found profile serializes into the activate payload, attachments are
//!   sent, then `Animation = 3001` and `CompositeEffect = 4005`; an absent profile returns without
//!   mutating the active profile.
//! * `lab/references/freerealms-wiki/wiki.sqlite` page "Game Interface" - "Quick Job Change/Profile
//!   Info: Hitting the arrows will let you change your job without having to go to the About Me
//!   menu", the community description of this exact request.
//!
//! RECONSTRUCTED: the refusal choice itself. The native handler is not recovered, no class-switch
//! frame exists in the lab captures, and no error-code vocabulary is recovered for this command, so
//! the two failure-command fields are emitted as the zeroes the recovered client constructor
//! installs rather than as an invented reason.

#[path = "../src/restoration/piece_044.rs"]
#[allow(dead_code)]
mod piece_044;

use piece_044::{
    ACTIVATE_PROFILE_ANIMATION, ACTIVATE_PROFILE_COMPOSITE_EFFECT, BASE_CLIENT_UPDATE_OPCODE,
    BASE_COMMAND_OPCODE, ClientUpdate, Command, CommandPacketActivateProfileFailed, PreserveError,
    PreservedValues, ProfileRefusal, SwitchOutcome, SwitchRequestError, decide_switch,
    decode_switch_request, preserve,
};

fn request(profile: i32) -> Vec<u8> {
    let mut bytes = vec![26, 0, 13, 0];
    bytes.extend(profile.to_le_bytes());
    bytes
}

#[test]
fn recovered_opcodes_and_animation_immediates_match_the_original_evidence() {
    assert_eq!(BASE_COMMAND_OPCODE, 26);
    assert_eq!(BASE_CLIENT_UPDATE_OPCODE, 38);
    assert_eq!(Command::SetProfile.wire(), 13);
    assert_eq!(Command::SetProfileByItemDefinitionId.wire(), 36);
    assert_eq!(CommandPacketActivateProfileFailed::WIRE, 26);
    assert_eq!(ClientUpdate::Hitpoints.wire(), 1);
    assert_eq!(ClientUpdate::ActivateProfile.wire(), 21);
    assert_eq!(ACTIVATE_PROFILE_ANIMATION, 3001);
    assert_eq!(ACTIVATE_PROFILE_COMPOSITE_EFFECT, 4005);
}

#[test]
fn switch_request_parses_only_the_recovered_four_byte_payload() {
    let parsed = decode_switch_request(&request(43)).unwrap();
    assert_eq!(parsed.profile_id, 43);

    let mut wrong_command_family = request(43);
    wrong_command_family[0] = 25;
    assert_eq!(
        decode_switch_request(&wrong_command_family),
        Err(SwitchRequestError::WrongCommandFamily)
    );

    let mut wrong_sub_command = request(43);
    wrong_sub_command[2] = 36;
    assert_eq!(
        decode_switch_request(&wrong_sub_command),
        Err(SwitchRequestError::WrongSubCommand)
    );

    assert_eq!(
        decode_switch_request(&[26, 0, 13, 0]),
        Err(SwitchRequestError::WrongLength)
    );

    let mut trailing = request(43);
    trailing.push(0);
    assert_eq!(
        decode_switch_request(&trailing),
        Err(SwitchRequestError::WrongLength)
    );

    for refused in [-1, i32::MIN] {
        assert_eq!(
            decode_switch_request(&request(refused)),
            Err(SwitchRequestError::NonPositiveProfile),
            "profile {refused}"
        );
    }
}

#[test]
fn unlocked_profile_switches_and_an_unowned_profile_is_refused_without_mutation() {
    let unlocked = [1, 43];

    let SwitchOutcome::Activate { profile_id } = decide_switch(&request(43), &unlocked).unwrap()
    else {
        panic!("an owned profile must activate");
    };
    assert_eq!(profile_id, 43);

    assert_eq!(
        decide_switch(&request(99), &unlocked),
        Ok(SwitchOutcome::Refuse(ProfileRefusal::NotOwned))
    );

    assert_eq!(
        decide_switch(&request(58), &[1, 58]),
        Ok(SwitchOutcome::Activate { profile_id: 58 }),
        "the module rejects what the caller does not own, not a hard-coded class list"
    );
}

#[test]
fn refusal_selects_the_recovered_failure_command_and_never_fabricates_a_reason() {
    let mut expected = vec![26, 0, 26, 0];
    for value in [99i32, 0] {
        expected.extend(value.to_le_bytes());
    }

    let outcome = decide_switch(&request(99), &[1, 43]).unwrap();
    let packet = match &outcome {
        SwitchOutcome::Refuse(refusal) => refusal.packet(99),
        SwitchOutcome::Activate { profile_id } => panic!("profile {profile_id} must not activate"),
    };

    assert_eq!(
        packet, expected,
        "the failure command is [26,0][26,0][id][0]"
    );
    assert_eq!(
        outcome.activated(),
        None,
        "a refusal must not hand the caller an activation profile"
    );
}

#[test]
fn switching_carries_health_gear_and_the_active_profile_through_unchanged() {
    let before = PreservedValues {
        health: 2_137,
        active_profile_id: 1,
        equipment_definition_ids: vec![4914, 100],
        ability_count: 8,
    };
    let after = PreservedValues {
        active_profile_id: 43,
        ..before.clone()
    };

    let guard = preserve(&before, &after).unwrap();
    assert_eq!(guard.health, before.health);
    assert_eq!(guard.equipment_definition_ids, vec![4914, 100]);
    assert_eq!(guard.ability_count, 8);
    assert_eq!(guard.profile_id, 43);
    assert_ne!(guard.profile_id, before.active_profile_id);
}

#[test]
fn preservation_guard_reports_the_exact_field_that_changed() {
    let before = PreservedValues {
        health: 2_137,
        active_profile_id: 1,
        equipment_definition_ids: vec![4914],
        ability_count: 8,
    };

    assert_eq!(
        preserve(
            &before,
            &PreservedValues {
                health: 2_500,
                active_profile_id: 43,
                ..before.clone()
            }
        ),
        Err(PreserveError::HealthChanged)
    );
    assert_eq!(
        preserve(
            &before,
            &PreservedValues {
                active_profile_id: 43,
                equipment_definition_ids: vec![4914, 100],
                ..before.clone()
            }
        ),
        Err(PreserveError::EquipmentChanged)
    );
    assert_eq!(
        preserve(
            &before,
            &PreservedValues {
                active_profile_id: 43,
                ability_count: 6,
                ..before.clone()
            }
        ),
        Err(PreserveError::AbilitiesChanged)
    );
    assert_eq!(
        preserve(&before, &before),
        Err(PreserveError::ProfileUnchanged)
    );
}

#[test]
fn switch_decision_and_guard_compose_into_one_accepted_activation() {
    let before = PreservedValues {
        health: 640,
        active_profile_id: 1,
        equipment_definition_ids: vec![4914],
        ability_count: 8,
    };
    let outcome = decide_switch(&request(43), &[1, 43]).unwrap();
    let after = PreservedValues {
        active_profile_id: 43,
        ..before.clone()
    };
    let guard = preserve(&before, &after).unwrap();

    assert_eq!(guard.profile_id, after.active_profile_id);
    assert_eq!(guard.health, before.health);
    assert!(matches!(
        outcome,
        SwitchOutcome::Activate { profile_id: 43 }
    ));
}
