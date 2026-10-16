//! Work item 058: death and revive (KO / revive / location / health) tests.
//!
//! The module is included by path until the lead registers it in `src/lib.rs`.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (supplied client binary and decompiled client Lua; every hash is
//! re-checked by `scripts/pieces/piece_058.py`):
//! * `EncounterShowRespawnWindowPacket` is the encounter family
//!   (`BaseEncounterPacket` = outer opcode 41) sub-opcode **125**: the client constructor
//!   `FUN_008a33b0` passes `0x7d` to the family base constructor `FUN_008a2fd0` (which writes
//!   outer `0x29`), and the class carries exactly **two** extra `i32` fields after the 12-byte
//!   family header.
//! * The native dispatcher `FUN_0090d830` handles sub `0x7d` by calling
//!   `GroupHandler:SetKO` and then `RespawnWindow:DisplayRespawn`; sub `0x7e` (= 126) calls
//!   `RespawnWindow:Hide`.
//! * The display handler `FUN_00904300` treats its timer argument as milliseconds
//!   (`param_2 / 1000` for the label) and falls back to a client-side default when it is below 1.
//! * `ScriptsBase.lua` `RespawnWindow:DisplayRespawn(timer, isOverworld, reviveCost)` shows the
//!   `Revive here:\n %1 coins` dialog only when `isOverworld == 1`, otherwise calls
//!   `StartTimer(timer)`; the entry points are `respawn` -> `Ui.RevivePlayer()` and
//!   `OnPaidRevive(b)`.
//!
//! COMMUNITY (pinned Sanctuary and OSFR sources, labelled `community` in the checker):
//! * `PacketZoneSafeTeleportRequest` is top-level opcode **122** with an empty body, and both
//!   community servers answer it with `ClientUpdatePacketUpdateLocation` **38/12** and
//!   `Teleport = true` at a safe hub point of interest.
//! * The wiki "Combat" page documents the ten-second recovery, the brief invulnerability and the
//!   full health/energy restore; "Sunstone Valley" documents the free safe-zone revive against a
//!   1,000-coin revive on the spot of defeat.
//!
//! RECONSTRUCTED (labelled in the module): the once-only KO/revive arbiter, the relocation policy
//! and the terminal settlement ledger. No retail capture of 41/125 or 41/126 exists in the lab, so
//! the packet bodies are proven from the client class fields and the native reader, not from a
//! frame.
#[path = "../src/restoration/piece_058.rs"]
#[allow(dead_code)]
mod piece_058;

use piece_058::{
    ENCOUNTER_OPCODE, RECOVER_DELAY_MS, RESPAWN_HIDE_SUB, RESPAWN_WINDOW_SUB, RespawnOffer,
    RespawnWindow, ReviveArbiter, ReviveChoice, ReviveError, SAFE_TELEPORT_REQUEST_OPCODE,
    TerminalLedger, decode_respawn_window, decode_safe_teleport_request, encode_respawn_hide,
};

fn i32_bytes(value: i32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// The recovered family header: outer 41, `u16` sub-opcode, `i32` encounter id, `i32` instance.
fn family_header(sub: u16, encounter_id: u32, instance_id: i32) -> Vec<u8> {
    let mut bytes = ENCOUNTER_OPCODE.to_le_bytes().to_vec();
    bytes.extend(sub.to_le_bytes());
    bytes.extend((encounter_id as i32).to_le_bytes());
    bytes.extend(instance_id.to_le_bytes());
    bytes
}

#[test]
fn respawn_window_packet_uses_the_encounter_family_and_two_trailing_fields() {
    let window = RespawnWindow {
        encounter_id: 17,
        instance_id: 3,
        timer_ms: RECOVER_DELAY_MS,
        revive_cost: 1000,
    };
    let mut expected = family_header(RESPAWN_WINDOW_SUB, 17, 3);
    expected.extend(i32_bytes(RECOVER_DELAY_MS as i32));
    expected.extend(i32_bytes(1000));
    let encoded = window.encode();
    assert_eq!(
        encoded, expected,
        "41/125 body order: id, instance, timer, cost"
    );
    assert_eq!(
        encoded.len(),
        20,
        "12-byte family header plus two i32 fields"
    );
    assert_eq!(decode_respawn_window(&encoded).unwrap(), window);
}

#[test]
fn respawn_window_zero_timer_keeps_the_client_default() {
    // The native display handler substitutes a client-side default when the timer is below 1,
    // so zero is a legal "use the default" offer and must survive a round trip.
    let window = RespawnWindow {
        encounter_id: 4,
        instance_id: 1,
        timer_ms: 0,
        revive_cost: 0,
    };
    let encoded = window.encode();
    assert_eq!(&encoded[12..16], &[0, 0, 0, 0]);
    assert_eq!(decode_respawn_window(&encoded).unwrap(), window);
}

#[test]
fn respawn_window_decode_rejects_other_packets_and_short_bodies() {
    assert_eq!(
        decode_respawn_window(&family_header(RESPAWN_HIDE_SUB, 17, 3)),
        Err(ReviveError::WrongPacket {
            expected: RESPAWN_WINDOW_SUB,
            found: RESPAWN_HIDE_SUB,
        })
    );
    assert_eq!(
        decode_respawn_window(&[]),
        Err(ReviveError::MalformedWire(
            "expected at least 4 header bytes"
        ))
    );
    assert_eq!(
        decode_respawn_window(&family_header(RESPAWN_WINDOW_SUB, 17, 3)),
        Err(ReviveError::MalformedWire(
            "expected the two respawn fields"
        ))
    );
}

#[test]
fn respawn_hide_packet_is_the_bare_family_header() {
    // Sub 126 only asks the client to hide the window; the native reader consumes the standard
    // 12-byte family header before it branches.
    assert_eq!(
        encode_respawn_hide(17, 3),
        family_header(RESPAWN_HIDE_SUB, 17, 3)
    );
    assert_eq!(encode_respawn_hide(17, 3).len(), 12);
}

#[test]
fn safe_teleport_request_is_a_bare_122() {
    assert_eq!(SAFE_TELEPORT_REQUEST_OPCODE, 122);
    assert_eq!(decode_safe_teleport_request(&[122, 0]), Ok(()));
    assert_eq!(
        decode_safe_teleport_request(&[122, 0, 1]),
        Err(ReviveError::MalformedWire(
            "safe teleport request has no body"
        ))
    );
    assert_eq!(
        decode_safe_teleport_request(&[121, 0]),
        Err(ReviveError::WrongPacket {
            expected: SAFE_TELEPORT_REQUEST_OPCODE,
            found: 121,
        })
    );
}

#[test]
fn first_knockout_emits_one_window_and_repeats_are_refused() {
    let mut arbiter = ReviveArbiter::new(1000, 2500, 1000);
    assert!(
        !arbiter.apply_damage(600),
        "a survivable hit is not a knockout"
    );
    assert_eq!(arbiter.health(), 400);
    let offer = RespawnOffer {
        encounter_id: 17,
        instance_id: 1,
        timer_ms: RECOVER_DELAY_MS,
    };
    assert!(
        arbiter.apply_damage(400),
        "the hit that lands on zero is the knockout"
    );
    assert_eq!(arbiter.health(), 0);
    assert!(arbiter.knocked_out());
    let window = arbiter.knock_out(&offer);
    assert_eq!(
        window,
        Some(
            RespawnWindow {
                encounter_id: 17,
                instance_id: 1,
                timer_ms: RECOVER_DELAY_MS,
                revive_cost: 1000,
            }
            .encode()
        )
    );
    assert_eq!(
        arbiter.knock_out(&offer),
        None,
        "a repeated KO message is a no-op"
    );
    assert_eq!(arbiter.knockouts(), 1);
}

#[test]
fn knockout_offer_reports_the_knockout_only_once_per_call() {
    let mut arbiter = ReviveArbiter::new(1000, 0, 0);
    let offer = RespawnOffer {
        encounter_id: 9,
        instance_id: 2,
        timer_ms: 0,
    };
    assert!(arbiter.apply_damage(1000));
    assert!(arbiter.knock_out(&offer).is_some());
    // Health damage while already knocked out cannot re-arm the offer.
    assert!(!arbiter.apply_damage(50));
    assert_eq!(arbiter.health(), 0);
    assert_eq!(arbiter.knock_out(&offer), None);
}

#[test]
fn safe_revive_is_free_relocates_and_restores_full_health() {
    let mut arbiter = ReviveArbiter::new(1000, 2500, 1000);
    let offer = RespawnOffer {
        encounter_id: 17,
        instance_id: 1,
        timer_ms: RECOVER_DELAY_MS,
    };
    assert!(arbiter.apply_damage(1000));
    assert!(arbiter.knock_out(&offer).is_some());
    let destination = [12.5_f32, -3.0, 44.0];
    let rotation = [1.0_f32, 0.0, 0.0];
    let outcome = arbiter
        .revive(ReviveChoice::Safe {
            position: destination,
            rotation,
        })
        .expect("the free safe revive must succeed");
    assert_eq!(outcome.charged_coins, 0, "the safe revive is free");
    assert_eq!(outcome.health, 1000);
    assert_eq!(arbiter.coins(), 2500);
    assert!(!arbiter.knocked_out());
    assert_eq!(
        outcome.hide,
        Some(family_header(RESPAWN_HIDE_SUB, 17, 1)),
        "the revive hides the window with 41/126"
    );
    let relocation = outcome.relocation.expect("the safe revive relocates");
    assert_eq!(
        &relocation[..4],
        &[38, 0, 12, 0],
        "38/12 ClientUpdatePacketUpdateLocation"
    );
    assert_eq!(
        relocation.len(),
        38,
        "eight floats, the teleport flag and the unknown byte"
    );
    assert_eq!(relocation[36], 1, "Teleport is set for a revive relocation");
    assert_eq!(relocation[37], 0, "the writer's default trailing byte");
    assert_eq!(
        relocation,
        osfr_tools::dev_teleport::location_packet(destination, rotation),
        "the relocation matches the pinned reviewed teleport builder byte for byte"
    );
}

#[test]
fn paid_revive_charges_the_cost_once_and_keeps_the_spot() {
    let mut arbiter = ReviveArbiter::new(1000, 2500, 1000);
    let offer = RespawnOffer {
        encounter_id: 17,
        instance_id: 1,
        timer_ms: RECOVER_DELAY_MS,
    };
    assert!(arbiter.apply_damage(1000));
    assert!(arbiter.knock_out(&offer).is_some());
    let outcome = arbiter.revive(ReviveChoice::Paid).expect("paid revive");
    assert_eq!(outcome.charged_coins, 1000);
    assert_eq!(arbiter.coins(), 1500);
    assert_eq!(outcome.health, 1000);
    assert_eq!(
        outcome.relocation, None,
        "the coin revive keeps the spot of defeat"
    );
    assert_eq!(outcome.hide, Some(family_header(RESPAWN_HIDE_SUB, 17, 1)));
    // A replayed revive message must not charge a second time.
    assert_eq!(
        arbiter.revive(ReviveChoice::Paid),
        Err(ReviveError::NotKnockedOut)
    );
    assert_eq!(arbiter.coins(), 1500);
    assert_eq!(arbiter.revives(), 1);
}

#[test]
fn paid_revive_below_cost_changes_nothing() {
    let mut arbiter = ReviveArbiter::new(1000, 999, 1000);
    let offer = RespawnOffer {
        encounter_id: 17,
        instance_id: 1,
        timer_ms: RECOVER_DELAY_MS,
    };
    assert!(arbiter.apply_damage(1000));
    assert!(arbiter.knock_out(&offer).is_some());
    assert_eq!(
        arbiter.revive(ReviveChoice::Paid),
        Err(ReviveError::NotEnoughCoins {
            wallet: 999,
            cost: 1000,
        })
    );
    assert_eq!(arbiter.coins(), 999);
    assert_eq!(arbiter.health(), 0);
    assert!(arbiter.knocked_out());
}

#[test]
fn safe_revive_requires_a_finite_destination() {
    let mut arbiter = ReviveArbiter::new(1000, 0, 0);
    assert!(arbiter.apply_damage(1000));
    assert_eq!(
        arbiter.revive(ReviveChoice::Safe {
            position: [f32::NAN, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
        }),
        Err(ReviveError::InvalidPosition)
    );
    assert!(arbiter.knocked_out());
}

#[test]
fn revive_without_a_knockout_is_refused() {
    let mut arbiter = ReviveArbiter::new(1000, 5000, 1000);
    assert_eq!(
        arbiter.revive(ReviveChoice::Paid),
        Err(ReviveError::NotKnockedOut)
    );
    assert_eq!(arbiter.coins(), 5000);
    assert_eq!(arbiter.health(), 1000);
    assert_eq!(arbiter.revives(), 0);
}

#[test]
fn terminal_ledger_settles_each_instance_once_and_never_rewards_a_loss() {
    let mut ledger = TerminalLedger::default();
    assert!(ledger.settle(7, true), "the first won settlement grants");
    assert!(
        !ledger.settle(7, true),
        "a replayed terminal message grants nothing"
    );
    assert!(!ledger.settle(7, false));
    assert_eq!(ledger.settled(7), Some(true));
    assert!(
        !ledger.settle(8, false),
        "a knocked-out instance never grants"
    );
    assert!(
        !ledger.settle(8, true),
        "a lost instance cannot be re-settled as a win"
    );
    assert_eq!(ledger.settled(8), Some(false));
    assert!(
        ledger.settle(9, true),
        "a new instance settles independently"
    );
    assert_eq!(ledger.settled(10), None);
}

#[test]
fn revive_ledger_survives_a_second_knockout_in_the_same_instance() {
    // The exploit this guards: replaying the terminal messages of a finished encounter to be
    // paid or rewarded twice. The ledger keeps one verdict per instance.
    let mut arbiter = ReviveArbiter::new(1000, 0, 0);
    let offer = RespawnOffer {
        encounter_id: 17,
        instance_id: 5,
        timer_ms: RECOVER_DELAY_MS,
    };
    assert!(arbiter.apply_damage(1000));
    assert!(arbiter.knock_out(&offer).is_some());
    arbiter
        .revive(ReviveChoice::Safe {
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
        })
        .expect("safe revive");
    assert!(arbiter.apply_damage(1000));
    assert!(
        arbiter.knock_out(&offer).is_some(),
        "a fresh knockout offers again"
    );
    assert_eq!(arbiter.knockouts(), 2);
    assert_eq!(arbiter.revives(), 1);
}
