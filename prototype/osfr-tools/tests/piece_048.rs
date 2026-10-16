//! Work item 048 acceptance: original action-bar/hotbar binding packets and the
//! "persist only owned available actions" rule.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` - base inventory constructor
//!   `00a10400` writes main `0x2a` (42) and copies the sub-opcode; `00c14720`
//!   constructs `InventoryPacketItemActionBarAssign` with sub 6, a slot and an item
//!   id; `00a122e0` constructs the sub-9 by-item-record variant with three copied
//!   record fields; the header writer `00a1ada0` emits both 16-bit headers; the
//!   slot reader `008dae30` consumes three 1-byte bools, eleven 4-byte values (one
//!   of them float-compared for NaN) and one trailing 4-byte cast age with
//!   `0x7fffffff` as the never-cast sentinel; `008dadd0` consumes the bar id and
//!   slot index. Layout: 4 header + 8 bar/slot + (3 + 44 + 4) slot = 63 bytes.
//! * `lab/working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json` -
//!   the preserved captured self packet: one `ItemActionBar` (bar `2`) with exactly
//!   four slots `0..3`, three bound slots with quantities 59/28/33, disabled
//!   `Usable` flags and the never-cast sentinel on slot 1.
//! * `lab/decompiled/client-ui/ScriptsBase.lua` - `ItemActionBarCol`
//!   (IsEmpty/IconID/Name/IsEnabled/RefreshTimeLeft/TotalRefreshTime/Quantity/
//!   IsTargeting/IconTintValue), dock `EquipItem` -> `AssignToItemActionBar(c - 5, b)`,
//!   `activateItem` -> `ClickedAbility(b - 5, 2)` and fishing `AssignToItemActionBar(0, 0)`.
//! * pinned Sanctuary revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`
//!   (AGPL-3.0, inspected as facts): `InventoryPacketItemActionBarAssign` opcode 6
//!   reads `Slot` then `Guid` and requires the remaining length to be zero;
//!   `ClientUpdatePacketUpdateActionBarSlot` is opcode 25 with `ActionBarData`
//!   (bar id + slot) and a 15-field, 51-byte `ActionBarSlot`; the handler hardcodes
//!   bar 2, clears the slot on `Guid == 0`, ignores an unknown item instance without
//!   replying and uses the item's own count as the slot quantity.
//!
//! The four bounded slots, reconciliation of persisted bindings against the live
//! inventory and `serverEnabled = true` on a fresh bind are reconstructed policy
//! labelled in the module and the research note. No native input is sent and no
//! inventory, save, content or shared source file is touched.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module
//! before the lead registers `pub mod piece_048;` under `src/restoration/`.
#[path = "../src/restoration/piece_048.rs"]
#[allow(dead_code)]
mod piece_048;

use piece_048::{
    ABILITY_BAR_ID, ASSIGN_BY_RECORD_SUB, ASSIGN_MAIN, ASSIGN_REQUEST_BYTES, ASSIGN_SUB,
    ActionBarBinding, ActionBarItems, ActionBarSlot, ActionBarState, AssignOutcome, AssignParse,
    AssignRejection, AssignRequest, CAPTURED_EMPTY_SLOT, CAPTURED_SLOT_0, CAPTURED_SLOT_1,
    CAPTURED_SLOT_2, ITEM_BAR_ID, ITEM_BAR_SLOTS, NEVER_CAST_AGE, OwnedAction, SLOT_BYTES,
    SLOT_FIELDS, SLOT_UPDATE_BYTES, SLOT_UPDATE_MAIN, SLOT_UPDATE_SUB, STATE_VERSION, slot_field,
};
use std::collections::BTreeMap;
use std::io;

struct Items(BTreeMap<i32, OwnedAction>);

impl ActionBarItems for Items {
    fn owned_action(&self, item_instance_id: i32) -> Option<OwnedAction> {
        self.0.get(&item_instance_id).copied()
    }
}

fn items(list: &[OwnedAction]) -> Items {
    Items(
        list.iter()
            .map(|action| (action.item_instance_id, *action))
            .collect(),
    )
}

/// The immutable opening checkpoint owns one Strong Arm Stew instance 15.
fn stew() -> OwnedAction {
    OwnedAction {
        item_instance_id: 15,
        definition_id: 35,
        count: 1,
        icon_id: 1016,
        icon_tint_id: 0,
        name_id: 1232,
    }
}

/// The same checkpoint owns one Wheeler on the Cob instance 11.
fn corn() -> OwnedAction {
    OwnedAction {
        item_instance_id: 11,
        definition_id: 1195,
        count: 1,
        icon_id: 401,
        icon_tint_id: 0,
        name_id: 1195,
    }
}

fn decode_update(bytes: &[u8]) -> (i32, i32, ActionBarSlot) {
    assert_eq!(bytes.len(), SLOT_UPDATE_BYTES);
    assert_eq!(&bytes[..2], &SLOT_UPDATE_MAIN.to_le_bytes());
    assert_eq!(&bytes[2..4], &SLOT_UPDATE_SUB.to_le_bytes());
    let bar = i32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let slot = i32::from_le_bytes(bytes[8..12].try_into().unwrap());
    let data = ActionBarSlot::from_bytes(&bytes[12..]).expect("rendered slot decodes");
    (bar, slot, data)
}

#[test]
fn assign_request_matches_the_recovered_twelve_byte_form() {
    let request = [
        ASSIGN_MAIN as u8,
        0,
        ASSIGN_SUB as u8,
        0,
        0,
        0,
        0,
        0,
        15,
        0,
        0,
        0,
    ];
    assert_eq!(request.len(), ASSIGN_REQUEST_BYTES);
    match piece_048::parse_assign(&request).expect("valid assign") {
        AssignParse::Request(parsed) => assert_eq!(
            parsed,
            AssignRequest {
                slot: 0,
                item_instance_id: 15
            }
        ),
        other => panic!("unexpected parse: {other:?}"),
    }

    let mut dock = vec![42, 0, 6, 0];
    dock.extend_from_slice(&3i32.to_le_bytes());
    dock.extend_from_slice(&21i32.to_le_bytes());
    match piece_048::parse_assign(&dock).expect("valid dock assign") {
        AssignParse::Request(parsed) => {
            assert_eq!(parsed.slot, 3);
            assert_eq!(parsed.item_instance_id, 21);
        }
        other => panic!("unexpected parse: {other:?}"),
    }

    // The unconsumed fishing lure unassign is the same packet with item id 0.
    let mut unassign = vec![42, 0, 6, 0];
    unassign.extend_from_slice(&0i32.to_le_bytes());
    unassign.extend_from_slice(&0i32.to_le_bytes());
    match piece_048::parse_assign(&unassign).expect("valid unassign") {
        AssignParse::Request(parsed) => assert_eq!(parsed.item_instance_id, 0),
        other => panic!("unexpected parse: {other:?}"),
    }
}

#[test]
fn assign_request_rejects_lengths_and_families_outside_the_original_form() {
    let mut short = vec![42, 0, 6, 0];
    short.extend_from_slice(&[0; 7]);
    assert!(piece_048::parse_assign(&short).is_err());

    let mut long = vec![42, 0, 6, 0];
    long.extend_from_slice(&[0; 9]);
    assert!(piece_048::parse_assign(&long).is_err());

    let mut wrong_family = vec![41, 0, 6, 0];
    wrong_family.extend_from_slice(&[0; 8]);
    assert!(piece_048::parse_assign(&wrong_family).is_err());

    assert!(piece_048::parse_assign(&[42, 0]).is_err());

    let mut other_sub = vec![42, 0, 3, 0];
    other_sub.extend_from_slice(&[0; 12]);
    assert_eq!(
        piece_048::parse_assign(&other_sub).expect("known family"),
        AssignParse::OtherSubOpcode
    );
}

#[test]
fn by_record_sub_opcode_is_reported_and_never_guessed() {
    // The native constructor 00a122e0 proves sub 9 exists (`FUN_00a10400(9)` plus a
    // slot and three copied record fields), but the serialized record field order is
    // not recovered. The module must refuse to parse or mutate it.
    let mut request = vec![42, 0, ASSIGN_BY_RECORD_SUB as u8, 0];
    request.extend_from_slice(&[0; 14]);
    match piece_048::parse_assign(&request).expect("known family") {
        AssignParse::ByRecordUnrecovered { observed_len } => {
            assert_eq!(observed_len, request.len());
        }
        other => panic!("unexpected parse: {other:?}"),
    }

    let mut state = ActionBarState::new();
    let outcome = state
        .apply_parse(
            AssignParse::ByRecordUnrecovered {
                observed_len: request.len(),
            },
            &items(&[stew()]),
        )
        .expect("by-record rejection is not an io error");
    assert_eq!(
        outcome,
        AssignOutcome {
            changed: false,
            reply: None,
            rejection: Some(AssignRejection::ByRecordUnrecovered),
        }
    );
    assert!(state.bindings().is_empty());
}

#[test]
fn slot_layout_covers_every_recovered_field_exactly_once() {
    assert_eq!(SLOT_FIELDS.len(), 15);
    let mut expected_offset = 0usize;
    for field in SLOT_FIELDS {
        assert_eq!(field.offset, expected_offset, "field {}", field.name);
        assert!(field.width == 1 || field.width == 4);
        expected_offset += field.width;
    }
    assert_eq!(expected_offset, SLOT_BYTES);
    assert_eq!(piece_048::slot_layout_bytes(), SLOT_BYTES);
    assert_eq!(SLOT_BYTES + 12, SLOT_UPDATE_BYTES);

    assert_eq!(slot_field::IS_EMPTY, 0);
    assert_eq!(slot_field::ICON_ID, 1);
    assert_eq!(slot_field::ICON_TINT_ID, 5);
    assert_eq!(slot_field::NAME_ID, 9);
    assert_eq!(slot_field::SELECTOR_A, 13);
    assert_eq!(slot_field::TARGET_SELECTOR, 17);
    assert_eq!(slot_field::TARGETING_PARAMETER, 21);
    assert_eq!(slot_field::MANA_COST, 25);
    assert_eq!(slot_field::ENABLED, 29);
    assert_eq!(slot_field::UNKNOWN_10, 30);
    assert_eq!(slot_field::TOTAL_REFRESH_TIME, 34);
    assert_eq!(slot_field::UNKNOWN_12, 38);
    assert_eq!(slot_field::QUANTITY, 42);
    assert_eq!(slot_field::FORCE_DISMOUNT, 46);
    assert_eq!(slot_field::CAST_AGE, 47);
}

#[test]
fn slot_renderer_writes_each_recovered_field_at_its_recovered_offset() {
    let slot = ActionBarSlot {
        is_empty: false,
        icon_id: 0x11,
        icon_tint_id: 0x22,
        name_id: 0x33,
        selector_a: 1,
        target_selector: 2,
        targeting_parameter: 0.5,
        mana_cost: 0x44,
        enabled: true,
        unknown_10: 0x55,
        total_refresh_time: 0x66,
        unknown_12: 0x77,
        quantity: 0x88,
        force_dismount: true,
        cast_age: NEVER_CAST_AGE,
    };
    let bytes = slot.to_bytes();
    assert_eq!(bytes.len(), SLOT_BYTES);
    for field in SLOT_FIELDS {
        let value = &bytes[field.offset..field.offset + field.width];
        let expected: Vec<u8> = match field.name {
            "is_empty" => vec![0],
            "icon_id" => 0x11i32.to_le_bytes().to_vec(),
            "icon_tint_id" => 0x22i32.to_le_bytes().to_vec(),
            "name_id" => 0x33i32.to_le_bytes().to_vec(),
            "selector_a" => 1i32.to_le_bytes().to_vec(),
            "target_selector" => 2i32.to_le_bytes().to_vec(),
            "targeting_parameter" => 0.5f32.to_le_bytes().to_vec(),
            "mana_cost" => 0x44i32.to_le_bytes().to_vec(),
            "enabled" => vec![1],
            "unknown_10" => 0x55i32.to_le_bytes().to_vec(),
            "total_refresh_time" => 0x66i32.to_le_bytes().to_vec(),
            "unknown_12" => 0x77i32.to_le_bytes().to_vec(),
            "quantity" => 0x88i32.to_le_bytes().to_vec(),
            "force_dismount" => vec![1],
            "cast_age" => NEVER_CAST_AGE.to_le_bytes().to_vec(),
            other => panic!("unexpected field {other}"),
        };
        assert_eq!(value, expected.as_slice(), "field {}", field.name);
    }
    assert_eq!(ActionBarSlot::from_bytes(&bytes).unwrap(), slot);
}

#[test]
fn captured_reference_slots_round_trip_through_the_layout() {
    let empty = ActionBarSlot::from_bytes(&CAPTURED_EMPTY_SLOT).expect("captured empty slot");
    assert!(empty.is_empty);
    assert!(!empty.enabled);
    assert!(!empty.force_dismount);
    assert_eq!(empty.quantity, 0);

    let first = ActionBarSlot::from_bytes(&CAPTURED_SLOT_0).expect("captured slot 0");
    assert!(!first.is_empty);
    assert_eq!(first.icon_id, 543);
    assert_eq!(first.icon_tint_id, 0);
    assert_eq!(first.name_id, 422_910);
    assert_eq!(first.selector_a, 1);
    assert_eq!(first.target_selector, 2);
    assert_eq!(first.targeting_parameter, 0.0);
    assert_eq!(first.mana_cost, 0);
    assert!(!first.enabled);
    assert_eq!(first.quantity, 59);
    assert!(first.force_dismount);
    assert_eq!(first.cast_age, 1_789_747_881);

    let second = ActionBarSlot::from_bytes(&CAPTURED_SLOT_1).expect("captured slot 1");
    assert_eq!(second.icon_id, 777);
    assert_eq!(second.quantity, 28);
    assert_eq!(second.cast_age, NEVER_CAST_AGE);

    let third = ActionBarSlot::from_bytes(&CAPTURED_SLOT_2).expect("captured slot 2");
    assert_eq!(third.icon_id, 544);
    assert_eq!(third.quantity, 33);

    for captured in [
        CAPTURED_EMPTY_SLOT,
        CAPTURED_SLOT_0,
        CAPTURED_SLOT_1,
        CAPTURED_SLOT_2,
    ] {
        let slot = ActionBarSlot::from_bytes(&captured).expect("captured slot");
        assert_eq!(slot.to_bytes(), captured);
    }
}

#[test]
fn slot_decoder_rejects_nan_targeting_and_wrong_lengths() {
    let mut nan = CAPTURED_EMPTY_SLOT;
    nan[slot_field::TARGETING_PARAMETER] = 0xff;
    nan[slot_field::TARGETING_PARAMETER + 1] = 0xff;
    nan[slot_field::TARGETING_PARAMETER + 2] = 0xff;
    nan[slot_field::TARGETING_PARAMETER + 3] = 0x7f;
    assert!(ActionBarSlot::from_bytes(&nan).is_err());
    assert!(ActionBarSlot::from_bytes(&CAPTURED_SLOT_0[..50]).is_err());
    assert!(ActionBarSlot::from_bytes(&[0u8; 52]).is_err());
}

#[test]
fn slot_update_matches_the_captured_reference_packet_frame() {
    let update = piece_048::item_slot_update(0, &ActionBarSlot::empty()).expect("empty update");
    let mut expected = vec![38, 0, 25, 0, 2, 0, 0, 0, 0, 0, 0, 0];
    expected.extend_from_slice(&CAPTURED_EMPTY_SLOT);
    assert_eq!(update, expected);
    assert_eq!(update.len(), SLOT_UPDATE_BYTES);

    let (bar, slot, data) = decode_update(&update);
    assert_eq!(bar, ITEM_BAR_ID);
    assert_eq!(slot, 0);
    assert!(data.is_empty);

    let captured = ActionBarSlot::from_bytes(&CAPTURED_SLOT_0).unwrap();
    let (bar, slot, data) = decode_update(&piece_048::item_slot_update(2, &captured).unwrap());
    assert_eq!(bar, ITEM_BAR_ID);
    assert_eq!(slot, 2);
    assert_eq!(data, captured);

    assert!(piece_048::item_slot_update(-1, &ActionBarSlot::empty()).is_err());
    assert!(piece_048::item_slot_update(ITEM_BAR_SLOTS, &ActionBarSlot::empty()).is_err());
    assert!(piece_048::empty_item_slot_update(ITEM_BAR_SLOTS).is_err());
    assert!(piece_048::empty_item_slot_update(ITEM_BAR_SLOTS - 1).is_ok());
}

#[test]
fn bind_uses_the_owned_instance_presentation_and_quantity() {
    let owned = items(&[stew()]);
    let mut state = ActionBarState::new();
    let outcome = state
        .assign(
            AssignRequest {
                slot: 2,
                item_instance_id: 15,
            },
            &owned,
        )
        .expect("assign");
    assert!(outcome.changed);
    assert_eq!(outcome.rejection, None);
    assert_eq!(
        state.bindings(),
        &[ActionBarBinding {
            slot: 2,
            item_instance_id: 15,
            definition_id: 35,
        }]
    );

    let reply = outcome.reply.expect("accepted assign replies");
    let (bar, slot, data) = decode_update(&reply);
    assert_eq!(bar, ITEM_BAR_ID);
    assert_eq!(slot, 2);
    assert!(!data.is_empty);
    assert_eq!(data.icon_id, 1016);
    assert_eq!(data.icon_tint_id, 0);
    assert_eq!(data.name_id, 1232);
    assert_eq!(data.quantity, 1);
    assert!(data.enabled);
    assert_eq!(data.selector_a, 1);
    assert_eq!(data.target_selector, 2);
    assert_eq!(data.targeting_parameter, 0.0);
    assert_eq!(data.mana_cost, 0);
    assert_eq!(data.total_refresh_time, 0);
    assert!(data.force_dismount);
    assert_eq!(data.cast_age, NEVER_CAST_AGE);

    // Repeating the same assign keeps the binding and reports no change.
    let repeat = state
        .assign(
            AssignRequest {
                slot: 2,
                item_instance_id: 15,
            },
            &owned,
        )
        .expect("repeat assign");
    assert!(!repeat.changed);
    assert_eq!(state.bindings().len(), 1);
}

#[test]
fn unassign_emits_the_empty_slot_and_clears_the_binding() {
    let owned = items(&[stew()]);
    let mut state = ActionBarState::new();
    state
        .assign(
            AssignRequest {
                slot: 1,
                item_instance_id: 15,
            },
            &owned,
        )
        .expect("assign");

    let outcome = state
        .assign(
            AssignRequest {
                slot: 1,
                item_instance_id: 0,
            },
            &owned,
        )
        .expect("unassign");
    assert!(outcome.changed);
    assert_eq!(outcome.rejection, None);
    assert!(state.bindings().is_empty());
    let reply = outcome.reply.expect("unassign replies");
    let (bar, slot, data) = decode_update(&reply);
    assert_eq!(bar, ITEM_BAR_ID);
    assert_eq!(slot, 1);
    assert!(data.is_empty);
    assert_eq!(&reply[12..], &CAPTURED_EMPTY_SLOT[..]);

    let again = state
        .assign(
            AssignRequest {
                slot: 1,
                item_instance_id: 0,
            },
            &owned,
        )
        .expect("repeat unassign");
    assert!(!again.changed);
    assert_eq!(
        &again.reply.expect("repeat unassign still replies")[12..],
        &CAPTURED_EMPTY_SLOT[..]
    );
}

#[test]
fn unowned_and_unavailable_items_are_rejected_without_state_change() {
    // A persisted binding the live inventory no longer owns is pruned by the same
    // call, and the unknown request itself never writes a slot.
    let mut state: ActionBarState = serde_json::from_str(
        r#"{"version":1,"bindings":[{"slot":0,"item_instance_id":11,"definition_id":1195}]}"#,
    )
    .expect("stale binding fixture");
    let unknown = state
        .assign(
            AssignRequest {
                slot: 1,
                item_instance_id: 99,
            },
            &items(&[stew()]),
        )
        .expect("unknown item");
    assert_eq!(unknown.rejection, Some(AssignRejection::UnknownItem));
    assert_eq!(unknown.reply, None);
    assert!(!unknown.changed);
    assert!(state.bindings().is_empty(), "stale binding is pruned");

    let mut zeroed = stew();
    zeroed.count = 0;
    let empty_stack = state
        .assign(
            AssignRequest {
                slot: 1,
                item_instance_id: 15,
            },
            &items(&[zeroed]),
        )
        .expect("zero stack");
    assert_eq!(empty_stack.rejection, Some(AssignRejection::Unavailable));
    assert_eq!(empty_stack.reply, None);
    assert!(state.bindings().is_empty());
}

#[test]
fn slot_bounds_are_enforced_on_both_the_request_and_the_reply() {
    let owned = items(&[stew()]);
    let mut state = ActionBarState::new();
    for slot in [-1, ITEM_BAR_SLOTS, ITEM_BAR_SLOTS + 7, i32::MIN] {
        let outcome = state
            .assign(
                AssignRequest {
                    slot,
                    item_instance_id: 15,
                },
                &owned,
            )
            .expect("out of range slot");
        assert_eq!(outcome.rejection, Some(AssignRejection::SlotOutOfRange));
        assert_eq!(outcome.reply, None);
        assert!(!outcome.changed);
        assert!(state.bindings().is_empty());
    }
}

#[test]
fn persisted_bindings_keep_only_owned_available_actions() {
    let owned = items(&[stew(), corn()]);
    let mut state = ActionBarState::new();
    state
        .assign(
            AssignRequest {
                slot: 3,
                item_instance_id: 11,
            },
            &owned,
        )
        .expect("corn bind");
    state
        .assign(
            AssignRequest {
                slot: 0,
                item_instance_id: 15,
            },
            &owned,
        )
        .expect("stew bind");
    assert_eq!(
        state.bindings(),
        &[
            ActionBarBinding {
                slot: 0,
                item_instance_id: 15,
                definition_id: 35,
            },
            ActionBarBinding {
                slot: 3,
                item_instance_id: 11,
                definition_id: 1195,
            },
        ]
    );

    let json = serde_json::to_string(&state).expect("serialize");
    assert!(json.contains("\"version\":1"));
    let mut reloaded: ActionBarState = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(reloaded, state);

    // Definition drift on the same instance id drops just that binding on reload.
    let mut drifted = corn();
    drifted.definition_id = 999;
    assert_eq!(reloaded.reconcile(&items(&[stew(), drifted])), 1);
    assert_eq!(
        reloaded.bindings(),
        &[ActionBarBinding {
            slot: 0,
            item_instance_id: 15,
            definition_id: 35,
        }]
    );
    // A consumed instance with no live item drops the remaining binding.
    assert_eq!(reloaded.reconcile(&items(&[corn()])), 1);
    assert!(reloaded.bindings().is_empty());
    reloaded.validate().expect("reconciled state is valid");

    // A live item that drops to an empty stack cannot stay persisted: the seeded
    // corn binding is pruned by the same call that binds the live stew.
    let mut pruned: ActionBarState = serde_json::from_str(
        r#"{"version":1,"bindings":[{"slot":3,"item_instance_id":11,"definition_id":1195}]}"#,
    )
    .expect("stale stack fixture");
    let mut gone = corn();
    gone.count = 0;
    pruned
        .assign(
            AssignRequest {
                slot: 2,
                item_instance_id: 15,
            },
            &items(&[stew(), gone]),
        )
        .expect("bind prunes first");
    assert_eq!(
        pruned.bindings(),
        &[ActionBarBinding {
            slot: 2,
            item_instance_id: 15,
            definition_id: 35,
        }]
    );
}

#[test]
fn persisted_state_rejects_tampering() {
    let state = ActionBarState::new();
    assert_eq!(state.version(), STATE_VERSION);
    state.validate().expect("fresh state");

    let duplicate: ActionBarState = serde_json::from_str(
        r#"{"version":1,"bindings":[
            {"slot":1,"item_instance_id":15,"definition_id":35},
            {"slot":1,"item_instance_id":11,"definition_id":1195}]}"#,
    )
    .expect("shape parses");
    assert!(duplicate.validate().is_err());

    let out_of_range: ActionBarState = serde_json::from_str(
        r#"{"version":1,"bindings":[{"slot":4,"item_instance_id":15,"definition_id":35}]}"#,
    )
    .expect("shape parses");
    assert!(out_of_range.validate().is_err());

    let zero_instance: ActionBarState = serde_json::from_str(
        r#"{"version":1,"bindings":[{"slot":0,"item_instance_id":0,"definition_id":35}]}"#,
    )
    .expect("shape parses");
    assert!(zero_instance.validate().is_err());

    let unknown_version: ActionBarState =
        serde_json::from_str(r#"{"version":2,"bindings":[]}"#).expect("shape parses");
    assert!(unknown_version.validate().is_err());

    let unknown_field: Result<ActionBarState, _> =
        serde_json::from_str(r#"{"version":1,"slots":[]}"#);
    assert!(unknown_field.is_err(), "unknown fields must be rejected");
}

#[test]
fn reconnect_updates_follow_the_live_inventory() {
    let mut state = ActionBarState::new();
    state
        .assign(
            AssignRequest {
                slot: 3,
                item_instance_id: 15,
            },
            &items(&[stew()]),
        )
        .expect("bind");

    let mut stocked = stew();
    stocked.count = 7;
    let updates = state
        .slot_updates(&items(&[stocked]))
        .expect("reconnect updates");
    assert_eq!(updates.len(), 1);
    let (bar, slot, data) = decode_update(&updates[0]);
    assert_eq!(bar, ITEM_BAR_ID);
    assert_eq!(slot, 3);
    assert_eq!(data.quantity, 7);

    let updates = state
        .slot_updates(&items(&[]))
        .expect("stale binding produces no update");
    assert!(updates.is_empty());
}

#[test]
fn the_item_bar_is_not_the_combat_bar() {
    assert_eq!(ITEM_BAR_ID, 2);
    assert_eq!(ABILITY_BAR_ID, 1);
    assert_eq!(ITEM_BAR_SLOTS, 4);
    assert_eq!(ASSIGN_MAIN, 42);
    assert_eq!(ASSIGN_SUB, 6);
    assert_eq!(ASSIGN_BY_RECORD_SUB, 9);
    assert_eq!(SLOT_UPDATE_MAIN, 38);
    assert_eq!(SLOT_UPDATE_SUB, 25);
    assert_eq!(ASSIGN_REQUEST_BYTES, 12);
    assert_eq!(NEVER_CAST_AGE, i32::MAX);

    // The community tint rule: a zero instance tint falls back to the definition.
    assert_eq!(piece_048::resolve_tint(0, 77), 77);
    assert_eq!(piece_048::resolve_tint(3, 77), 3);

    // No input can invent state from an empty request.
    let error: io::Error = piece_048::parse_assign(&[]).expect_err("empty request");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}
