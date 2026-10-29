use osfr_tools::inventory::{EquippedItem, Inventory, InventoryItem, InventoryState, RentalState};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn item(id: i32, definition_id: i32) -> InventoryItem {
    InventoryItem {
        id,
        definition_id,
        tint: 0,
        count: 2,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}
fn command(family: u16, opcode: u16, fields: &[i32]) -> Vec<u8> {
    let mut bytes = [family.to_le_bytes(), opcode.to_le_bytes()].concat();
    for field in fields {
        bytes.extend(field.to_le_bytes());
    }
    bytes
}
fn signed(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn serialize_item(item: &InventoryItem) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in [
        item.definition_id,
        item.tint,
        item.id,
        item.count as i32,
        item.consumed_count as i32,
        i32::try_from(item.last_cast_time).unwrap(),
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.push(u8::from(item.rental.is_some()));
    if let Some(rental) = &item.rental {
        bytes.extend(rental.expiration.to_le_bytes());
        bytes.extend(rental.upsells.to_le_bytes());
        bytes.extend(rental.bundle_id.to_le_bytes());
    }
    bytes.extend(item.ability_count.to_le_bytes());
    bytes.push(u8::from(item.activate_enabled));
    bytes
}

struct Fixture {
    path: PathBuf,
    content: Value,
    packet: Vec<u8>,
    items_offset: usize,
    equipment_offset: usize,
    coins_offset: usize,
}
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-inventory-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let mut stack = item(7, 100);
        stack.stored_count = 1;
        stack.ability_count = 6;
        let mut rental = item(8, 101);
        rental.count = 2;
        rental.stored_count = 1;
        rental.ability_count = 6;
        rental.last_cast_time = 987654321;
        rental.tint = 23;
        rental.rental = Some(RentalState {
            expiration: 123456789,
            upsells: 2,
            bundle_id: 8,
        });
        let state = InventoryState {
            next_instance_id: None,
            coins: 4,
            items: vec![stack, rental],
            equipment: vec![EquippedItem {
                profile_id: 1,
                slot: 3,
                item_id: 7,
                category: 3,
            }],
        };
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        packet.extend([0xA5; 8]);
        let coins_offset = packet.len();
        packet.extend(4i32.to_le_bytes());
        let equipment_offset = packet.len();
        for value in [1i32, 3, 7, 3] {
            packet.extend(value.to_le_bytes());
        }
        let items_offset = packet.len();
        packet.extend(2i32.to_le_bytes());
        for item in &state.items {
            packet.extend(serialize_item(item));
        }
        packet.extend(b"quest-tail");
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind, offset, length, profile_id| json!({"kind":kind,"offset":offset,"length":length,"profile_id":profile_id,"sha256":digest(&packet[offset..offset+length])});
        let definitions: Vec<_> = (100i32..=102).map(|id| {
            let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
            json!({"id":id,"slot":3,"class_id":1,"default_tint":5,"max_stack":10,"model":"shirt.adr","texture":"plain","tint_alias":"dye","effect":0,"wield_type":0,"serialized_hex":bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"sha256":digest(&bytes)})
        }).collect();
        let content = json!({"schema_version":1,"player_guid":1,"default_state":state,"profiles":[{"id":1,"allowed_classes":[1],"slots":[3]}],"spans":[span("coins",coins_offset,4,0),span("equipment",equipment_offset,16,1),span("items",items_offset,80,0)],"definitions":definitions});
        let fixture = Self {
            path,
            content,
            packet,
            items_offset,
            equipment_offset,
            coins_offset,
        };
        fixture.write();
        fixture
    }
    fn write(&self) {
        fs::write(
            self.path.join("inventory.json"),
            serde_json::to_vec(&self.content).unwrap(),
        )
        .unwrap();
    }
    fn set_default_state(&mut self, state: &InventoryState) {
        self.content["default_state"] = serde_json::to_value(state).unwrap();
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        packet.extend([0xA5; 8]);

        let coins_offset = packet.len();
        packet.extend((state.coins as i32).to_le_bytes());

        let equipment_offset = packet.len();
        let equipment: Vec<_> = state
            .equipment
            .iter()
            .filter(|entry| entry.profile_id == 1)
            .collect();
        packet.extend((equipment.len() as i32).to_le_bytes());
        for entry in equipment {
            packet.extend(entry.slot.to_le_bytes());
            packet.extend(entry.item_id.to_le_bytes());
            packet.extend(entry.category.to_le_bytes());
        }

        let items_offset = packet.len();
        packet.extend((state.items.len() as i32).to_le_bytes());
        for item in &state.items {
            packet.extend(serialize_item(item));
        }
        packet.extend(b"quest-tail");
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());

        for span in self.content["spans"].as_array_mut().unwrap() {
            let (offset, length) = match span["kind"].as_str().unwrap() {
                "coins" => (coins_offset, 4),
                "equipment" => (equipment_offset, 4 + 12 * state.equipment.len()),
                "items" => (
                    items_offset,
                    4 + state
                        .items
                        .iter()
                        .map(|item| serialize_item(item).len())
                        .sum::<usize>(),
                ),
                _ => unreachable!(),
            };
            span["offset"] = json!(offset);
            span["length"] = json!(length);
            span["sha256"] = json!(digest(&packet[offset..offset + length]));
        }
        self.packet = packet;
        self.items_offset = items_offset;
        self.equipment_offset = equipment_offset;
        self.coins_offset = coins_offset;
        self.write();
    }
    fn inventory(&self) -> Inventory {
        Inventory::load(&self.path, None).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path.join("inventory.json"));
        let _ = fs::remove_dir(&self.path);
    }
}

#[test]
fn definitions_catalog_matches_original_community_payload_framing() {
    let mut f = Fixture::new();
    // Loading keeps sorted unique definitions despite unordered duplicate input.
    let definitions = f.content["definitions"].as_array_mut().unwrap();
    definitions.reverse();
    definitions.push(definitions[0].clone());
    f.write();
    let inventory = f.inventory();
    let before = inventory.state().clone();
    let vectors: Value = serde_json::from_str(include_str!(
        "fixtures/inventory-catalog-community-vectors.json"
    ))
    .unwrap();
    let vector = vectors["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["count"] == 3)
        .unwrap();
    let hex = vector["hex"].as_str().unwrap();
    let expected: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap())
        .collect();
    let packet = inventory.definitions_packet();
    assert_eq!(packet, expected);
    assert_eq!(digest(&packet), vector["sha256"].as_str().unwrap());
    assert_eq!(&packet[..4], &[35, 0, 37, 0]);
    assert_eq!(signed(&packet, 4) as usize, packet.len() - 8);
    assert_eq!(signed(&packet, 8), 3);
    assert_eq!(inventory.state(), &before);
}

#[test]
fn startup_withholds_three_owned_items_missing_from_catalogue_and_keeps_saved_state() {
    let mut f = Fixture::new();
    let mut state = f.inventory().state().clone();
    for (instance_id, definition_id) in [(1145, 7960), (4594, 78398), (4595, 78399)] {
        state.items.push(item(instance_id, definition_id));
    }
    state.equipment[0].item_id = 1145;
    f.set_default_state(&state);

    let inventory = f.inventory();
    let definitions = inventory.definitions_packet();
    assert_eq!(signed(&definitions, 8), 3);
    for index in 0..3 {
        let id = signed(&definitions, 12 + index * 8);
        assert!(![7960, 78398, 78399].contains(&id));
    }

    let patched = inventory.initialization_patch(&f.packet).unwrap();
    assert_eq!(signed(&patched, f.items_offset), 2);
    assert_eq!(signed(&patched, f.items_offset + 12), 7);
    assert_eq!(signed(&patched, f.items_offset + 42), 8);
    assert_eq!(signed(&patched, f.equipment_offset + 8), 0);
    assert_eq!(signed(&inventory.profile_equipment(1), 8), 0);
    assert_eq!(signed(&inventory.profile_attachments(1).unwrap(), 0), 0);
    assert_eq!(patched.len(), f.packet.len() - 3 * 30);
    assert_eq!(inventory.state().items.len(), 5);
    assert_eq!(inventory.state().equipment[0].item_id, 1145);
    for (instance_id, definition_id) in [(1145, 7960), (4594, 78398), (4595, 78399)] {
        assert!(inventory
            .state()
            .items
            .iter()
            .any(|item| { item.id == instance_id && item.definition_id == definition_id }));
    }
}

#[test]
fn presentation_uses_serialized_byte_lengths_and_refuses_truncated_fields() {
    let mut f = Fixture::new();
    let mut bytes = Vec::new();
    for value in [100i32, 123, 0, 456, 789, 0, 0, 0, 1, 0, 3] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend([0, 0]);
    for text in ["modèle.adr", "plain"] {
        bytes.extend((text.len() as i32).to_le_bytes());
        bytes.extend(text.as_bytes());
    }
    bytes.extend(0i32.to_le_bytes());
    bytes.extend(28i32.to_le_bytes());
    let replace = |f: &mut Fixture, data: &[u8]| {
        let d = &mut f.content["definitions"][0];
        d["serialized_hex"] =
            serde_json::json!(data.iter().map(|b| format!("{b:02x}")).collect::<String>());
        d["sha256"] = serde_json::json!(digest(data));
        f.write();
    };
    replace(&mut f, &bytes);
    let metadata = f.inventory().presentation(100).unwrap();
    assert_eq!(
        (
            metadata.name_id,
            metadata.icon_id,
            metadata.icon_tint,
            metadata.item_type
        ),
        (123, 456, 789, 28)
    );
    for length in 4..bytes.len() {
        replace(&mut f, &bytes[..length]);
        assert!(
            f.inventory().presentation(100).is_err(),
            "accepted length {length}"
        );
    }
    bytes[46..50].copy_from_slice(&(-1i32).to_le_bytes());
    replace(&mut f, &bytes);
    assert!(f.inventory().presentation(100).is_err());
}

#[test]
fn default_inventory_roundtrip_preserves_rental_and_other_packet_bytes() {
    let f = Fixture::new();
    let inventory = f.inventory();
    assert_eq!(inventory.initialization_patch(&f.packet).unwrap(), f.packet);
    assert_eq!(
        inventory.state().items[1]
            .rental
            .as_ref()
            .unwrap()
            .expiration,
        123456789
    );
    let state: InventoryState =
        serde_json::from_slice(&serde_json::to_vec(inventory.state()).unwrap()).unwrap();
    assert_eq!(
        Inventory::load(&f.path, Some(state)).unwrap().state(),
        inventory.state()
    );
}

#[test]
fn existing_reward_38_3_uses_stored_count_and_last_cast_not_ability_or_rental_fields() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let outcome = inventory.grant(50, 100, 3).unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.replies.len(), 2);
    assert_eq!(outcome.replies[0], command(38, 19, &[54]));
    assert_eq!(
        &outcome.replies[1][..24],
        &command(38, 3, &[7, 5, -1, 0, 1])
    );
    assert_eq!(outcome.replies[1].len(), 32);
    assert_eq!(
        &outcome.replies[1][24..],
        &i64::from(i32::MAX).to_le_bytes()
    );
    assert_eq!(inventory.state().items[0].ability_count, 6);
    let patched = inventory.initialization_patch(&f.packet).unwrap();
    assert_eq!(signed(&patched, f.coins_offset), 54);
    assert_eq!(signed(&patched, f.items_offset + 4 + 12), 5);
    assert_eq!(patched.len(), f.packet.len());
}

#[test]
fn partial_rental_update_uses_last_cast_time_instead_of_rental_expiration() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let update = inventory.consume(8, 1).unwrap();
    assert_eq!(
        update.replies[0],
        [
            command(38, 3, &[8, 1, -1, 0, 1]),
            987654321i64.to_le_bytes().to_vec()
        ]
        .concat()
    );
    assert_ne!(&update.replies[0][24..], &123456789i64.to_le_bytes());
    assert_eq!(inventory.state().items[1].ability_count, 6);
}

#[test]
fn legacy_item_snapshot_defaults_stored_count_and_keeps_64_bit_last_cast() {
    let mut legacy = serde_json::to_value(item(7, 100)).unwrap();
    legacy.as_object_mut().unwrap().remove("stored_count");
    legacy["last_cast_time"] = json!(0x1122_3344_5566_7788i64);
    let migrated: InventoryItem = serde_json::from_value(legacy).unwrap();
    assert_eq!(migrated.stored_count, 0);
    assert_eq!(migrated.last_cast_time, 0x1122_3344_5566_7788);
}

#[test]
fn new_reward_has_length_prefixed_item_and_definition_and_adjusts_self_lengths() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let outcome = inventory.grant(0, 102, 1).unwrap();
    let reply = &outcome.replies[0];
    assert_eq!(&reply[..4], &command(38, 2, &[]));
    assert_eq!(signed(reply, 4), 38);
    assert_eq!(signed(reply, 8), 102);
    assert_eq!(signed(reply, 16), 9);
    assert_eq!(signed(reply, 38), 102);
    let patched = inventory.initialization_patch(&f.packet).unwrap();
    assert_eq!(patched.len(), f.packet.len() + 30);
    assert_eq!(signed(&patched, f.items_offset), 3);
    assert_eq!(signed(&patched, 3) as usize, patched.len() - 7);
    assert_eq!(signed(&patched, 9) as usize, patched.len() - 13);
    assert!(patched.ends_with(b"quest-tail"));
}

#[test]
fn equip_by_id_and_record_and_unequip_persist_profile_slots() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    inventory.grant(0, 102, 1).unwrap();
    let outcome = inventory.handle(&command(42, 3, &[9, 1, 3])).unwrap();
    assert!(outcome.changed);
    assert_eq!(outcome.replies.len(), 2);
    assert_eq!(&outcome.replies[0][..8], &command(38, 5, &[9]));
    assert_eq!(&outcome.replies[1][..4], &command(35, 6, &[]));
    let patched = inventory.initialization_patch(&f.packet).unwrap();
    assert_eq!(signed(&patched, f.equipment_offset + 8), 9);
    assert!(
        !inventory
            .handle(&command(42, 8, &[102, 0, 1, 3]))
            .unwrap()
            .changed
    );
    let unequip = inventory.handle(&command(42, 2, &[3, 1])).unwrap();
    assert_eq!(unequip.replies, vec![command(38, 6, &[3, 1])]);
    assert!(unequip.changed);
    assert_eq!(inventory.state().equipment[0].item_id, 0);
    assert!(!inventory.handle(&command(42, 2, &[3, 1])).unwrap().changed);
}

#[test]
fn invalid_requests_and_failed_grants_leave_all_state_unchanged() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let previous = inventory.state().clone();
    for request in [
        command(42, 3, &[999, 1, 3]),
        command(42, 3, &[7, 99, 3]),
        command(42, 3, &[7, 1, 9]),
        command(42, 2, &[9, 1]),
        command(42, 8, &[102, 0, 1, 3]),
        vec![42, 0, 3, 0],
    ] {
        assert!(inventory.handle(&request).is_err());
        assert_eq!(inventory.state(), &previous);
    }
    for (coins, definition, quantity) in
        [(5, 999, 1), (0, 100, 0), (0, 100, 11), (u32::MAX, 100, 1)]
    {
        assert!(inventory.grant(coins, definition, quantity).is_err());
        assert_eq!(inventory.state(), &previous);
    }
    assert!(inventory.grant_coins(u32::MAX).is_err());
    assert_eq!(inventory.state(), &previous);
    assert!(!inventory.handle(&command(42, 10, &[7])).unwrap().changed);
    assert_eq!(inventory.state(), &previous);
}

#[test]
fn authorized_consumption_updates_or_deletes_and_clears_equipment() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let update = inventory.consume(7, 1).unwrap();
    assert_eq!(signed(&update.replies[0], 8), 1);
    let deleted = inventory.consume(7, 1).unwrap();
    assert_eq!(
        deleted.replies,
        vec![command(38, 6, &[3, 1]), command(38, 4, &[7])]
    );
    assert_eq!(inventory.state().equipment[0].item_id, 0);
    assert_eq!(inventory.state().items.len(), 1);
    let previous = inventory.state().clone();
    assert!(inventory.consume(8, 3).is_err());
    assert_eq!(inventory.state(), &previous);
    let patched = inventory.initialization_patch(&f.packet).unwrap();
    assert_eq!(signed(&patched, f.items_offset), 1);
}

#[test]
fn definition_consumption_skips_rentals_and_is_all_or_nothing() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let previous = inventory.state().clone();
    // Definition 101 is held only as a rental stack, and definition 100 only twice over.
    for (definition, quantity) in [(101, 1), (100, 3), (999, 1)] {
        assert!(inventory.consume_definition(definition, quantity).is_err());
        assert_eq!(inventory.state(), &previous);
    }
    let taken = inventory.consume_definition(100, 2).unwrap();
    assert!(taken.changed);
    assert_eq!(
        taken.replies,
        vec![command(38, 6, &[3, 1]), command(38, 4, &[7])]
    );
    assert_eq!(inventory.state().equipment[0].item_id, 0);
    assert_eq!(inventory.state().items.len(), 1, "the rental stack stays");
}

#[test]
fn definition_request_is_bounded_and_returns_verified_definition() {
    let f = Fixture::new();
    let mut inventory = f.inventory();
    let reply = inventory.handle(&command(35, 34, &[100, 0])).unwrap();
    assert_eq!(reply.replies, vec![command(35, 34, &[100, 8, 100, 8])]);
    assert!(!reply.changed);
    assert!(inventory.handle(&command(35, 34, &[100, 1])).is_err());
    assert!(inventory.handle(&command(35, 34, &[100, -1])).is_err());
    assert!(inventory
        .handle(&command(35, 34, &[999, 0]))
        .unwrap()
        .replies
        .is_empty());
}

#[test]
fn patch_preserves_other_layers_and_rejects_changed_proven_spans() {
    let f = Fixture::new();
    let inventory = f.inventory();
    let mut layered = f.packet.clone();
    layered[13] = 33;
    let last = layered.len() - 1;
    layered[last] = 44;
    assert_eq!(inventory.initialization_patch(&layered).unwrap(), layered);
    layered[f.items_offset + 4] ^= 1;
    assert!(inventory.initialization_patch(&layered).is_err());
    assert!(inventory.initialization_patch(&f.packet[..12]).is_err());
}

#[test]
fn corrupt_content_and_invalid_saved_state_are_rejected() {
    let mut f = Fixture::new();
    f.content["definitions"][0]["sha256"] = json!("00".repeat(32));
    f.write();
    assert!(Inventory::load(&f.path, None).is_err());
    let mut f = Fixture::new();
    f.content["spans"][1]["offset"] = json!(0);
    f.write();
    assert!(Inventory::load(&f.path, None).is_err());
    let f = Fixture::new();
    let inventory = f.inventory();
    let mut state = inventory.state().clone();
    state.items.push(state.items[0].clone());
    assert!(Inventory::validate_state(&state).is_err());
    let mut state = inventory.state().clone();
    state.items[0].count = 0;
    assert!(Inventory::validate_state(&state).is_err());
    let mut state = inventory.state().clone();
    state.items[0].definition_id = 999;
    assert!(Inventory::load(&f.path, Some(state)).is_err());
    let mut state = inventory.state().clone();
    state.equipment[0].item_id = 99;
    assert!(Inventory::validate_state(&state).is_err());
    let mut state = inventory.state().clone();
    state.equipment.clear();
    assert!(Inventory::load(&f.path, Some(state)).is_err());
    let mut state = inventory.state().clone();
    state.equipment[0].category = 123;
    assert!(Inventory::load(&f.path, Some(state)).is_err());
}

#[test]
fn identical_catalog_duplicates_are_allowed_but_conflicts_reject() {
    let mut f = Fixture::new();
    let duplicate = f.content["definitions"][0].clone();
    f.content["definitions"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    f.write();
    assert!(Inventory::load(&f.path, None).is_ok());
    f.content["definitions"][3]["slot"] = json!(4);
    f.write();
    assert!(Inventory::load(&f.path, None).is_err());
}

#[test]
fn saved_equipment_cannot_bypass_class_or_slot_requirements() {
    for (field, value) in [("class_id", 2), ("slot", 4)] {
        let mut f = Fixture::new();
        let mut state = f.inventory().state().clone();
        state.equipment[0].item_id = 8;
        f.content["definitions"][1][field] = json!(value);
        f.write();
        assert!(Inventory::load(&f.path, Some(state)).is_err());
    }
}

#[test]
#[ignore = "requires prepared private inventory content and compiled unchanged community writer"]
fn private_catalog_matches_compiled_original_community_serializer() {
    let content = PathBuf::from(
        std::env::var_os("OSFR_GAMEPLAY_CONTENT").expect("private gameplay content required"),
    );
    let oracle = fs::read(
        std::env::var_os("OSFR_INVENTORY_CATALOG_ORACLE")
            .expect("compiled catalog oracle required"),
    )
    .unwrap();
    let inventory = Inventory::load(&content, None).unwrap();
    let before = inventory.state().clone();
    let packet = inventory.definitions_packet();
    assert_eq!(packet, oracle);
    assert_eq!(signed(&packet, 4) as usize, packet.len() - 8);
    let manifest: Value =
        serde_json::from_slice(&fs::read(content.join("inventory.json")).unwrap()).unwrap();
    let unique: std::collections::BTreeSet<_> = manifest["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_i64().unwrap())
        .collect();
    assert_eq!(signed(&packet, 8) as usize, unique.len());
    assert_eq!(inventory.state(), &before);
}

#[test]
#[ignore = "requires prepared private inventory content and captured self template"]
fn private_captured_inventory_roundtrip_and_reward() {
    let content = PathBuf::from(
        std::env::var_os("OSFR_GAMEPLAY_CONTENT").expect("private gameplay content required"),
    );
    let reference = PathBuf::from(
        std::env::var_os("OSFR_GATEWAY_REFERENCE").expect("private gateway reference required"),
    );
    let packet = fs::read(reference.join("0007-login.bin")).unwrap();
    let mut inventory = Inventory::load(&content, None).unwrap();
    assert_eq!(inventory.state().items.len(), 1386);
    let missing_item_bytes: usize = inventory
        .state()
        .items
        .iter()
        .filter(|item| !inventory.has_definition(item.definition_id))
        .map(|item| serialize_item(item).len())
        .sum();
    let patched = inventory.initialization_patch(&packet).unwrap();
    assert_eq!(patched.len(), packet.len() - missing_item_bytes);
    let outcome = inventory.grant(50, 1195, 1).unwrap();
    assert_eq!(outcome.replies.len(), 2);
    assert_eq!(
        inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == 1077)
            .unwrap()
            .count,
        10
    );
    let restored = Inventory::load(&content, Some(inventory.state().clone())).unwrap();
    assert_eq!(restored.state(), inventory.state());
    let patched = restored.initialization_patch(&packet).unwrap();
    assert_eq!(signed(&patched, 238), 50);
    assert_eq!(patched.len(), packet.len() - missing_item_bytes);
    println!(
        "1386 saved items, {missing_item_bytes} bytes withheld from startup self data; reward count and coins restored"
    );
}
