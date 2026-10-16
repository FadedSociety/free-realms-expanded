//! Work item 046 acceptance: consumable use requests, toolbar slot presentation and the
//! reconstructed once-only consumption / cooldown / expiry policy.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/server/LibSOE/SOE/BaseInventoryPackets.cs` - `InventoryPacketItemActionBarAssign = 6`,
//!   `InventoryPacketCheckImmediateActivationItem = 16`, `InventoryPacketUseImmediateActivationItem = 15`.
//! * `lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` - `AbilityPacketClientRequestStartAbility = 10`,
//!   `AbilityPacketFailed = 1`.
//! * `lab/decompiled/server/LibSOE/SOE/BaseClientUpdatePackets.cs` -
//!   `ClientUpdatePacketUpdateActionBarSlot = 25`.
//! * `lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs` - the original 15-field
//!   `ActionBarSlot.Serialize` order (51 bytes) used by the 38/25 reply.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - the recovered `activateItem` / `equipItem` dock calls
//!   (`Ui.ClickedAbility(tonumber(b) - 5, 2)`, `Ui.AssignToItemActionBar(..., b)`).
//! * `lab/reports/original-inventory-use-v1/proof.json` (SHA-256
//!   `262d05bd51b55719ad93e27ddb988f7a22eebf9821ce8d1458ac1895cdacb9cf`) - the request schemas, the
//!   cached wiki potion revisions and the item metadata rows.
//! * `lab/reports/original-item-toolbar-v1/proof.json` (SHA-256
//!   `eb1fe14305fb2e769beda8486bf8cf5414c5c24c9f617e68f64f5361fd47c24e`) - the native 38/25 reader
//!   semantics and the four captured wire vectors embedded below (hex copied from the proof; the test
//!   re-reads the preserved `.bin` files when the lab is present).
//! * `lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json` (pinned AGPL-3.0
//!   revision `9e082025391177d5a33ee18c714ddd06b457a695`) - the consumable item records;
//!   `lab/reports/locale-id-to-text.json` resolves every string id.
//!
//! The ledger policy (cooldown window, effect expiry, replay refusal) is reconstructed: every duration is
//! supplied by the caller and no historical balance number is baked in. Nothing here grants items,
//! currency or experience, and no native request was sent.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers `pub mod piece_046;` under `src/restoration/mod.rs`/`src/lib.rs`.
#[path = "../src/restoration/piece_046.rs"]
#[allow(dead_code)]
mod piece_046;

use piece_046::{
    ABILITY_FAILED_SUB, ABILITY_MAIN, ACTION_BAR_SLOT_WIRE_LEN, AssignError,
    CAST_BLOCK_STATUS_MASK, CHECK_IMMEDIATE_ACTIVATION_SUB, CLIENT_REQUEST_START_ABILITY_SUB,
    CLIENT_UPDATE_MAIN, CONSUMABLE_CLASS, ConsumableLedger, FOOD_CATEGORY, FOOD_CATEGORY_NAME_ID,
    INVENTORY_MAIN, ITEM_ACTION_BAR_ASSIGN_SUB, ITEM_ACTION_BAR_ID, ImmediateActivation,
    NEVER_CAST_SENTINEL, POTION_CATEGORY, POTION_CATEGORY_NAME_ID, Target,
    UPDATE_ACTION_BAR_SLOT_LEN, UPDATE_ACTION_BAR_SLOT_SUB, USE_FAILURE_STRING_ID,
    USE_IMMEDIATE_ACTIVATION_SUB, UseOutcome, UsePolicy, UseRejection, UseRequest, consumable,
    consumable_definitions, decode_ability_failed, decode_client_request_start_ability,
    decode_immediate_activation, decode_item_action_bar_assign, decode_slot_update,
    encode_client_request_start_ability, encode_immediate_activation,
    encode_item_action_bar_assign,
};

fn unhex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "odd hex length");
    (0..hex.len() / 2)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).expect("hex byte"))
        .collect()
}

/// Captured 38/25 bar2/slot0 disabled Medium Health Potion template (63 bytes), SHA-256
/// `0a4e41c4420868a044b6fb2caf3b1dfc7088013b4c4c67ba66205d16b79e1cee`.
const CAPTURED_MEDIUM_TEMPLATE: &str = "260019000200000000000000001f02000000000000fe73060001000000020000000000000000000000000000000000000000000000003b00000001a962ad6a";
/// Captured 38/25 empty bar2/slot0 slot (63 bytes), SHA-256
/// `2d4e53d0ba2746a248ec6a89b070ff02b75157f96468737273ef6e09f2fb14ea`.
const CAPTURED_EMPTY_SLOT: &str = "260019000200000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
/// The private candidate's Stew inspection slot: verified icon/name/quantity, never-cast sentinel,
/// SHA-256 `c7a8fa1df3c97d6862df8218c1ba59634022b7db43df88ac2e37bc7ed0726467`.
const CAPTURED_STEW_SLOT: &str = "26001900020000000000000000f803000000000000d004000001000000020000000000000000000000010000000000000000000000000100000001ffffff7f";
/// Captured 36/1 failure with localized string id 3079 (8 bytes).
const CAPTURED_FAILURE_3079: &str = "24000100070c0000";

#[test]
fn recovered_opcodes_and_lengths_match_the_pinned_sources() {
    assert_eq!((INVENTORY_MAIN, ITEM_ACTION_BAR_ASSIGN_SUB), (42, 6));
    assert_eq!((INVENTORY_MAIN, CHECK_IMMEDIATE_ACTIVATION_SUB), (42, 16));
    assert_eq!((INVENTORY_MAIN, USE_IMMEDIATE_ACTIVATION_SUB), (42, 15));
    assert_eq!((ABILITY_MAIN, CLIENT_REQUEST_START_ABILITY_SUB), (36, 10));
    assert_eq!((ABILITY_MAIN, ABILITY_FAILED_SUB), (36, 1));
    assert_eq!((CLIENT_UPDATE_MAIN, UPDATE_ACTION_BAR_SLOT_SUB), (38, 25));
    assert_eq!(ITEM_ACTION_BAR_ID, 2);
    assert_eq!(ACTION_BAR_SLOT_WIRE_LEN, 51);
    assert_eq!(UPDATE_ACTION_BAR_SLOT_LEN, 63);
    assert_eq!(CAST_BLOCK_STATUS_MASK, 0x0015_42C6);
    assert_eq!(NEVER_CAST_SENTINEL, i32::MAX);
    assert_eq!(USE_FAILURE_STRING_ID, 3079);
}

#[test]
fn assign_request_is_the_exact_recovered_twelve_byte_form() {
    let encoded = encode_item_action_bar_assign(0, 11);
    assert_eq!(encoded, [0x2a, 0x00, 0x06, 0x00, 0, 0, 0, 0, 11, 0, 0, 0]);
    assert_eq!(unhex("2a000600000000000b000000"), encoded.to_vec());
    let decoded = decode_item_action_bar_assign(&encoded).expect("assign");
    assert_eq!((decoded.slot, decoded.instance_id), (0, 11));
    // The recovered reader rejects any trailing byte.
    assert!(decode_item_action_bar_assign(&[0x2a, 0x00, 0x06, 0x00]).is_none());
    let mut extra = encoded.to_vec();
    extra.push(0);
    assert!(decode_item_action_bar_assign(&extra).is_none());
}

#[test]
fn activate_request_is_the_exact_recovered_guid_and_position_forms() {
    let guid = encode_client_request_start_ability(2, 3, Target::Guid(0x1122_3344_5566_7788));
    assert_eq!(guid.len(), 24);
    assert_eq!(
        unhex("24000a000200000003000000000000008877665544332211"),
        guid
    );
    let decoded = decode_client_request_start_ability(&guid).expect("guid activate");
    assert_eq!(
        (decoded.bar, decoded.slot, decoded.target),
        (2, 3, Target::Guid(0x1122_3344_5566_7788))
    );

    let position =
        encode_client_request_start_ability(2, 0, Target::Position([1.5, -2.0, 3.0, 0.0]));
    assert_eq!(position.len(), 32);
    let decoded = decode_client_request_start_ability(&position).expect("position activate");
    assert_eq!(decoded.target, Target::Position([1.5, -2.0, 3.0, 0.0]));
    // Truncated forms fail closed.
    assert!(decode_client_request_start_ability(&guid[..guid.len() - 1]).is_none());
    assert!(decode_client_request_start_ability(&position[..31]).is_none());
}

#[test]
fn immediate_activation_forms_are_distinct_eight_byte_requests() {
    let check = encode_immediate_activation(ImmediateActivation::Check, 15);
    let use_item = encode_immediate_activation(ImmediateActivation::Use, 15);
    assert_eq!(unhex("2a0010000f000000"), check);
    assert_eq!(unhex("2a000f000f000000"), use_item);
    assert_eq!(
        decode_immediate_activation(&check),
        Some((ImmediateActivation::Check, 15))
    );
    assert_eq!(
        decode_immediate_activation(&use_item),
        Some((ImmediateActivation::Use, 15))
    );
    // Any other sub or length is refused rather than guessed.
    assert!(decode_immediate_activation(&unhex("2a0011000f000000")).is_none());
    assert!(decode_immediate_activation(&[0x2a, 0x00, 0x10, 0x00]).is_none());
}

#[test]
fn captured_failure_vector_decodes_to_the_recovered_string_id() {
    let decoded = decode_ability_failed(&unhex(CAPTURED_FAILURE_3079)).expect("failure");
    assert_eq!(decoded, USE_FAILURE_STRING_ID);
    assert!(decode_ability_failed(&unhex("24000100070c00")).is_none());
}

#[test]
fn captured_slot_vectors_parse_with_the_recovered_field_names() {
    let slot = decode_slot_update(&unhex(CAPTURED_MEDIUM_TEMPLATE)).expect("medium template");
    assert_eq!((slot.bar, slot.slot), (2, 0));
    assert!(!slot.is_empty);
    assert_eq!(slot.icon, 543);
    assert_eq!(slot.icon_tint, 0);
    assert_eq!(slot.name, 422910);
    assert_eq!((slot.selector_a, slot.target_selector), (1, 2));
    assert_eq!(slot.numeric_target_parameter, 0.0);
    assert_eq!(slot.mana_cost, 0);
    assert!(!slot.server_enabled);
    assert_eq!(slot.total_refresh_time, 0);
    assert_eq!(slot.quantity, 59);
    assert!(slot.force_dismount);
    assert_eq!(slot.cast_age_ms, 1_789_747_881);

    let empty = decode_slot_update(&unhex(CAPTURED_EMPTY_SLOT)).expect("empty slot");
    assert!(empty.is_empty);
    assert_eq!(empty.quantity, 0);
    assert_eq!(empty.icon, 0);

    let stew = decode_slot_update(&unhex(CAPTURED_STEW_SLOT)).expect("stew slot");
    assert!(!stew.is_empty);
    assert_eq!((stew.icon, stew.name, stew.quantity), (1016, 1232, 1));
    assert!(stew.server_enabled);
    assert!(stew.force_dismount);
    assert_eq!(stew.cast_age_ms, NEVER_CAST_SENTINEL);
}

#[test]
fn slot_parser_fails_closed_on_wrong_header_or_length() {
    let bytes = unhex(CAPTURED_MEDIUM_TEMPLATE);
    let mut wrong_main = bytes.clone();
    wrong_main[0] = 0x25;
    assert!(decode_slot_update(&wrong_main).is_none());
    let mut wrong_sub = bytes.clone();
    wrong_sub[2] = 0x18;
    assert!(decode_slot_update(&wrong_sub).is_none());
    assert!(decode_slot_update(&bytes[..62]).is_none());
    let mut longer = bytes.clone();
    longer.push(0);
    assert!(decode_slot_update(&longer).is_none());
}

#[test]
fn captured_vectors_are_re_read_from_the_preserved_lab_when_present() {
    let lab = std::path::Path::new("../../../lab/reports/original-item-toolbar-v1");
    for (name, hex) in [
        ("captured-medium-template.bin", CAPTURED_MEDIUM_TEMPLATE),
        ("empty-slot.bin", CAPTURED_EMPTY_SLOT),
        ("stew-inspection-slot.bin", CAPTURED_STEW_SLOT),
        ("failure3079.bin", CAPTURED_FAILURE_3079),
    ] {
        let path = lab.join(name);
        if path.is_file() {
            let preserved = std::fs::read(&path).expect("read preserved vector");
            assert_eq!(
                preserved,
                unhex(hex),
                "{name} drifted from the embedded vector"
            );
        }
    }
}

#[test]
fn recast_remaining_follows_the_recovered_client_semantics() {
    let mut bytes = unhex(CAPTURED_MEDIUM_TEMPLATE);
    // total_refresh_time at slot offset 34, cast age at slot offset 47 (12-byte prefix included).
    bytes[46..50].copy_from_slice(&5_000_i32.to_le_bytes());
    bytes[59..63].copy_from_slice(&1_000_i32.to_le_bytes());
    let slot = decode_slot_update(&bytes).expect("mid-cooldown slot");
    assert_eq!(slot.recast_remaining_ms(0), 4_000);
    assert_eq!(slot.recast_remaining_ms(4_000), 0);
    assert_eq!(slot.recast_remaining_ms(50_000), 0);

    // Zero refresh time means no recast delay even with an age present.
    let zero = decode_slot_update(&unhex(CAPTURED_MEDIUM_TEMPLATE)).expect("zero refresh");
    assert_eq!(zero.recast_remaining_ms(0), 0);

    // The recovered never-cast sentinel keeps the slot free.
    let never = decode_slot_update(&unhex(CAPTURED_STEW_SLOT)).expect("never cast");
    assert_eq!(never.recast_remaining_ms(0), 0);
}

#[test]
fn slot_input_gate_matches_the_recovered_client_checks() {
    let stew = decode_slot_update(&unhex(CAPTURED_STEW_SLOT)).expect("stew slot");
    assert!(stew.usable_now(0, 0));
    assert!(!stew.usable_now(CAST_BLOCK_STATUS_MASK, 0));
    assert!(
        !stew.usable_now(0x0000_0002, 0),
        "a masked status bit blocks input"
    );

    let mut costly = stew;
    costly.mana_cost = 40;
    assert!(costly.usable_now(0, 40));
    assert!(!costly.usable_now(0, 39));

    let empty = decode_slot_update(&unhex(CAPTURED_EMPTY_SLOT)).expect("empty slot");
    assert!(!empty.usable_now(0, 100));
    let mut disabled = decode_slot_update(&unhex(CAPTURED_STEW_SLOT)).expect("stew slot");
    disabled.server_enabled = false;
    assert!(!disabled.usable_now(0, 100));
}

#[test]
fn catalogue_is_the_recovered_consumable_rows() {
    assert_eq!(CONSUMABLE_CLASS, 9);
    assert_eq!((POTION_CATEGORY, FOOD_CATEGORY), (9, 10));
    assert_eq!((POTION_CATEGORY_NAME_ID, FOOD_CATEGORY_NAME_ID), (961, 962));
    assert_eq!(consumable_definitions().len(), 7);
    let rows: Vec<(i32, i32, i32, i32, i32, i32)> = consumable_definitions()
        .iter()
        .map(|item| {
            assert!(item.single_use);
            assert_eq!(item.class_id, CONSUMABLE_CLASS);
            assert!(item.category_id == POTION_CATEGORY || item.category_id == FOOD_CATEGORY);
            (
                item.id,
                item.icon_id,
                item.name_id,
                item.description_id,
                item.category_id,
                item.ability_id,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            (35, 1016, 1232, 3172, 10, 3264),
            (1195, 1241, 6278, 6282, 10, 232),
            (2173, 730, 422909, 422916, 9, 267),
            (3021, 543, 422910, 422916, 9, 4697),
            (11002, 730, 422909, 422916, 9, 4696),
            (65820, 749, 4531, 4532, 9, 4696),
            (68989, 543, 3966, 3966, 9, 4340),
        ]
    );
    let stew = consumable(35).expect("stew");
    assert_eq!(stew.name, "Strong Arm Stew");
    assert_eq!(
        stew.description,
        "Eating this food increases the health and size of the player."
    );
    assert_eq!(stew.power_rating, 1);
    assert_eq!(stew.cost, 180);
    assert_eq!(
        consumable(3021).expect("medium potion").name,
        "Medium Health Potion"
    );
    assert_eq!(
        consumable(3021).expect("medium potion").description,
        "A potion that restores some of your health over 5 seconds."
    );
    assert_eq!(consumable(11002).expect("small potion").power_rating, 1);
    assert_eq!(
        consumable(2173).expect("legacy small potion").ability_id,
        267
    );
    assert!(consumable(999_999).is_none());
}

#[test]
fn ledger_consumes_once_and_gates_on_the_caller_supplied_cooldown() {
    let mut ledger = ConsumableLedger::new();
    ledger.seed_owned(15, 35, 2);
    ledger.assign(2, 0, 15).expect("assign stew");
    let policy = UsePolicy {
        cooldown_ms: 5_000,
        effect_duration_ms: 30_000,
    };

    let first = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 1_000,
        },
        policy,
    );
    assert_eq!(
        first,
        UseOutcome::Consumed {
            item_id: 35,
            instance_id: 15,
            ability_id: 3264,
            remaining: 1,
            cooldown_ready_at_ms: 6_000,
            effect_expires_at_ms: Some(31_000),
        }
    );
    assert_eq!(ledger.owned_count(15), Some(1));
    assert_eq!(ledger.effects().len(), 1);

    // A retransmitted click inside the recast window is refused without consuming.
    let blocked = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 2_000,
        },
        policy,
    );
    assert_eq!(
        blocked,
        UseOutcome::Refused(UseRejection::OnCooldown {
            ready_at_ms: 6_000,
            remaining_ms: 4_000,
        })
    );
    assert_eq!(ledger.owned_count(15), Some(1));

    let second = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 6_000,
        },
        policy,
    );
    assert_eq!(
        second,
        UseOutcome::Consumed {
            item_id: 35,
            instance_id: 15,
            ability_id: 3264,
            remaining: 0,
            cooldown_ready_at_ms: 11_000,
            effect_expires_at_ms: Some(36_000),
        }
    );
    assert_eq!(ledger.owned_count(15), Some(0));
    assert_eq!(ledger.cooldown_ready_at(2, 0), Some(11_000));
}

#[test]
fn ledger_refuses_stale_or_replayed_ticks_without_consuming() {
    let mut ledger = ConsumableLedger::new();
    ledger.seed_owned(11, 1195, 3);
    ledger.seed_owned(12, 1195, 3);
    ledger.assign(2, 1, 11).expect("assign corn");
    ledger.assign(2, 2, 12).expect("assign second corn");
    let policy = UsePolicy {
        cooldown_ms: 0,
        effect_duration_ms: 0,
    };

    let accepted = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 1,
            status_mask: 0,
            now_ms: 700,
        },
        policy,
    );
    assert!(matches!(
        accepted,
        UseOutcome::Consumed { remaining: 2, .. }
    ));

    let replay = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 1,
            status_mask: 0,
            now_ms: 700,
        },
        policy,
    );
    assert_eq!(
        replay,
        UseOutcome::Refused(UseRejection::ReplayedOrStaleRequest {
            last_accept_ms: 700
        })
    );
    // A fresh slot has no recast record yet, so an older tick reaches the stale guard itself.
    let stale = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 2,
            status_mask: 0,
            now_ms: 699,
        },
        policy,
    );
    assert_eq!(
        stale,
        UseOutcome::Refused(UseRejection::ReplayedOrStaleRequest {
            last_accept_ms: 700
        })
    );
    assert_eq!(ledger.owned_count(11), Some(2));
    assert_eq!(
        ledger.owned_count(12),
        Some(3),
        "the stale slot is untouched"
    );
    assert!(ledger.effects().is_empty());
}

#[test]
fn every_rejection_leaves_quantity_effects_and_cooldowns_untouched() {
    let mut ledger = ConsumableLedger::new();
    ledger.seed_owned(11, 1195, 1);
    ledger.seed_owned(77, 424_242, 4);
    ledger.assign(2, 1, 11).expect("assign corn");
    ledger.assign(2, 2, 77).expect("assign unknown item");
    let policy = UsePolicy {
        cooldown_ms: 1_000,
        effect_duration_ms: 1_000,
    };

    let refusals = [
        (
            UseRequest {
                bar: 3,
                slot: 1,
                status_mask: 0,
                now_ms: 100,
            },
            UseRejection::WrongBar { bar: 3 },
        ),
        (
            UseRequest {
                bar: 2,
                slot: 4,
                status_mask: 0,
                now_ms: 100,
            },
            UseRejection::SlotNotAssigned { bar: 2, slot: 4 },
        ),
        (
            UseRequest {
                bar: 2,
                slot: 2,
                status_mask: 0,
                now_ms: 100,
            },
            UseRejection::UncataloguedItem { item_id: 424_242 },
        ),
        (
            UseRequest {
                bar: 2,
                slot: 1,
                status_mask: CAST_BLOCK_STATUS_MASK,
                now_ms: 100,
            },
            UseRejection::BlockedStatus {
                blocked_bits: CAST_BLOCK_STATUS_MASK,
            },
        ),
    ];
    for (request, expected) in refusals {
        assert_eq!(
            ledger.use_item(request, policy),
            UseOutcome::Refused(expected)
        );
    }
    assert_eq!(ledger.owned_count(11), Some(1));
    assert_eq!(ledger.owned_count(77), Some(4));
    assert!(ledger.effects().is_empty());
    assert_eq!(ledger.cooldown_ready_at(2, 1), None);

    // The single-use item disappears after its one accepted use.
    let used = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 1,
            status_mask: 0,
            now_ms: 100,
        },
        policy,
    );
    assert!(matches!(used, UseOutcome::Consumed { remaining: 0, .. }));
    assert_eq!(ledger.owned_count(11), Some(0));
    // The last accepted tick was 100; a later request reaches the depleted check first.
    let depleted = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 1,
            status_mask: 0,
            now_ms: 5_000,
        },
        policy,
    );
    assert_eq!(
        depleted,
        UseOutcome::Refused(UseRejection::Depleted { instance_id: 11 })
    );
}

#[test]
fn active_effects_expire_on_the_exact_caller_supplied_boundary() {
    let mut ledger = ConsumableLedger::new();
    ledger.seed_owned(15, 35, 5);
    ledger.assign(2, 0, 15).expect("assign stew");
    let policy = UsePolicy {
        cooldown_ms: 0,
        effect_duration_ms: 3_000,
    };
    let consumed = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 10_000,
        },
        policy,
    );
    assert!(matches!(
        consumed,
        UseOutcome::Consumed {
            effect_expires_at_ms: Some(13_000),
            ..
        }
    ));
    let effect = ledger.effects()[0];
    assert!(!effect.expired(12_999));
    assert!(effect.expired(13_000));
    assert_eq!(ledger.live_effects(12_999).count(), 1);
    assert_eq!(ledger.live_effects(13_000).count(), 0);

    // A second accepted use replaces that instance's window instead of stacking duplicates.
    let second = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 20_000,
        },
        policy,
    );
    assert!(matches!(
        second,
        UseOutcome::Consumed {
            effect_expires_at_ms: Some(23_000),
            ..
        }
    ));
    assert_eq!(ledger.effects().len(), 1);

    // A zero-duration policy records no effect at all.
    let mut plain = ConsumableLedger::new();
    plain.seed_owned(11, 1195, 1);
    plain.assign(2, 1, 11).expect("assign corn");
    let outcome = plain.use_item(
        UseRequest {
            bar: 2,
            slot: 1,
            status_mask: 0,
            now_ms: 1,
        },
        UsePolicy {
            cooldown_ms: 0,
            effect_duration_ms: 0,
        },
    );
    assert!(matches!(
        outcome,
        UseOutcome::Consumed {
            effect_expires_at_ms: None,
            ..
        }
    ));
    assert!(plain.effects().is_empty());
}

#[test]
fn assignment_refuses_unknown_instances_and_invalid_slots() {
    let mut ledger = ConsumableLedger::new();
    ledger.seed_owned(15, 35, 1);
    assert_eq!(
        ledger.assign(2, 0, 99),
        Err(AssignError::UnknownInstance { instance_id: 99 })
    );
    assert_eq!(
        ledger.assign(2, -1, 15),
        Err(AssignError::NegativeSlot { slot: -1 })
    );
    assert_eq!(
        ledger.assign(1, 0, 15),
        Err(AssignError::WrongBar { bar: 1 })
    );
    assert_eq!(ledger.assign(2, 0, 15), Ok(()));
    // Re-assigning the same slot replaces the tracked instance.
    ledger.seed_owned(16, 3021, 1);
    assert_eq!(ledger.assign(2, 0, 16), Ok(()));
    let outcome = ledger.use_item(
        UseRequest {
            bar: 2,
            slot: 0,
            status_mask: 0,
            now_ms: 50,
        },
        UsePolicy {
            cooldown_ms: 0,
            effect_duration_ms: 0,
        },
    );
    assert!(matches!(
        outcome,
        UseOutcome::Consumed {
            item_id: 3021,
            instance_id: 16,
            remaining: 0,
            ..
        }
    ));
    assert_eq!(
        ledger.owned_count(15),
        Some(1),
        "the replaced instance is untouched"
    );
}
