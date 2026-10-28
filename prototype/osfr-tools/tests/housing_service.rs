//! Housing integration service acceptance for `src/housing_service.rs`.
//!
//! Client packet facts follow `notes/Build Guides/Gameplay/Housing.md` and its native-map citations.
//! The selector interpretation follows `notes/Reviews/Housing Placement Selection Identity Review.md`.
//! `FixtureConfig::capture_defaults()` is a reconstructed test fallback with older packet-derived
//! provenance; its tuples are not client-table evidence or verified retail content. House rows are
//! a Legacy data reference and remain disabled until client scene mappings exist.
//!
//! This target imports the registered housing service and its shared crate types.

#[path = "support/housing_move.rs"]
mod housing_move;

use osfr_tools::{housing, housing_native, housing_service, inventory, zone_transition};

use housing::{
    FixtureTransform, HousingError, PERMISSION_CAN_EDIT, PERMISSION_CAN_VISIT, PropertyDefinition,
    PropertyGrant, SavedFixture,
};
use housing_service::{
    BASE_OPCODE, FixtureConfig, FixtureConfigRow, FurnitureStock, HousingContent,
    HousingLotDefinition, HousingReply, HousingScene, HousingService, HousingServiceState,
    HousingZoneChange, OwnedInstance, REPLY_FIXTURE_REMOVE, REPLY_FIXTURE_UPDATE, REPLY_HOUSE_INFO,
    REPLY_INSTANCE_DATA, REPLY_PLACE_REQUEST, REPLY_POSITION, REPLY_ZONE_DATA, Request,
    RequestError, SERVICE_SCHEMA_VERSION, ServiceError, StateError, StockOperation, WireTransform,
    decode_request,
};
use inventory::{Inventory, InventoryItem, InventoryState};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use zone_transition::ZoneTransition;

const OWNER: u64 = 0x1122_3344_5566_7788;
const GUEST: u64 = 0x8877_6655_4433_2211;
const STRANGER: u64 = 0x8877_6655_4433_2212;
const HOUSE: u64 = 0x0f00_0000_0000_0001;
/// Test item definition from the reconstructed fixture configuration.
const BLOCK_ITEM: i32 = 7_466;
/// Test fixture definition paired with `BLOCK_ITEM` in the reconstructed configuration.
const BLOCK_FIXTURE: i32 = 34;
/// Test model id paired with `BLOCK_FIXTURE` in the reconstructed configuration.
const BLOCK_MODEL: i32 = 1_926;
/// The character's real furniture instance id in the reconstructed inventory content below.
const INSTANCE: u32 = 7;
/// Opaque client selection id used by the test; it is never treated as an inventory identity.
const SELECTION: u32 = 4_260;
const ZERO_ROTATION: [f32; 4] = [0.0; 4];

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn position() -> [f32; 4] {
    [10.0, 0.0, 20.0, 1.0]
}

fn block_transform(position: [f32; 4], rotation: [f32; 4], scale: f32) -> WireTransform {
    WireTransform {
        position,
        rotation,
        scale,
    }
}

fn place_frame(selection_id: u32, transform: &WireTransform) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&selection_id.to_le_bytes());
    for value in transform
        .position
        .iter()
        .chain(transform.rotation.iter())
        .chain(std::iter::once(&transform.scale))
    {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

fn place_request_frame(selection_id: u32) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&selection_id.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.push(0);
    bytes
}

fn append_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as i32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn append_float(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn pickup_frame(fixture_guid: u64) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&3u16.to_le_bytes());
    bytes.extend_from_slice(&fixture_guid.to_le_bytes());
    bytes
}

fn enter_frame(house_guid: u64) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&19u16.to_le_bytes());
    bytes.extend_from_slice(&house_guid.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.push(0);
    bytes
}

fn edit_frame(in_edit_mode: bool) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&6u16.to_le_bytes());
    bytes.push(u8::from(in_edit_mode));
    bytes
}

fn leave_frame() -> Vec<u8> {
    [BASE_OPCODE.to_le_bytes(), 10u16.to_le_bytes()].concat()
}

fn pickup_all_frame() -> Vec<u8> {
    [BASE_OPCODE.to_le_bytes(), 4u16.to_le_bytes()].concat()
}

fn move_frame(fixture_guid: u64, transform: &WireTransform) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&5u16.to_le_bytes());
    bytes.extend_from_slice(&fixture_guid.to_le_bytes());
    for value in transform
        .position
        .iter()
        .chain(transform.rotation.iter())
        .chain(std::iter::once(&transform.scale))
    {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());
    Ok::<(), ()>(()).unwrap();
    bytes
}

fn preview_frame(item_definition_id: u32) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&20u16.to_le_bytes());
    bytes.extend_from_slice(&item_definition_id.to_le_bytes());
    bytes
}

/// A legacy test move frame: fixture guid, two four-float groups and a 1.0 dword (the `127/2`
/// placement shape). A real move sends 0 there; use `housing_move::move_frame` for that shape.
fn save_frame(fixture_guid: u64) -> Vec<u8> {
    let mut bytes = BASE_OPCODE.to_le_bytes().to_vec();
    bytes.extend_from_slice(&5u16.to_le_bytes());
    bytes.extend_from_slice(&fixture_guid.to_le_bytes());
    for value in [11.0f32, 0.0, 21.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes
}

/// The live client's `127/5` body, byte for byte from the community-preserved client capture
/// `p2.pcap` (frame 32782; `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap`, sha256
/// `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`). After the two transform
/// groups the client sends a zero dword (where `127/2` carries the placement scale) and a 24-byte
/// tail whose last float is 1.0.
const RETAIL_MOVE: [u8; 72] = [
    0x7f, 0x00, 0x05, 0x00, 0x09, 0xc0, 0xf5, 0x7b, 0xa1, 0x4c, 0xf5, 0x07, 0xff, 0xbf, 0xdb, 0x43,
    0xff, 0xc7, 0xe2, 0xbd, 0xff, 0x7f, 0xd5, 0x43, 0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f,
];

fn property() -> PropertyDefinition {
    PropertyDefinition {
        property_id: 5,
        zone_id: 645,
        name_id: 6_514,
        max_fixtures: 4,
        max_landmarks: 2,
        build_areas: vec![[0.0, -20.0, 0.0, 100.0, 20.0, 100.0]],
    }
}

fn lot_definition(property: PropertyDefinition) -> HousingLotDefinition {
    HousingLotDefinition {
        property_id: property.property_id,
        zone_id: property.zone_id,
        name_id: property.name_id,
        icon_id: 33_439,
        max_fixtures: property.max_fixtures,
        max_landmarks: property.max_landmarks,
        build_areas: property.build_areas,
        spawn_position: [10.0, 0.0, 20.0, 1.0],
        spawn_rotation: [-1.0, 0.0, 0.0, 0.0],
        scene: Some(HousingScene {
            name: "hsg_emptylot_test".to_string(),
            zone_type: 2,
            sky: "sky_test.xml".to_string(),
            tutorial: false,
            unknown: 1,
            zone_id: 1,
            update_radius: 0.0,
            geometry_id: 214,
            override_update_radius: true,
            wait_for_zone_ready: false,
        }),
        scene_source: Some("test fixture only".to_string()),
        bounds_source: "test fixture only".to_string(),
        disabled_reason: None,
        deed: None,
        source_tag: "test fixture only".to_string(),
        enabled: true,
    }
}

fn service_with_config(config: FixtureConfig) -> HousingService {
    let mut service = HousingService::new(config);
    service
        .register_house_definition(lot_definition(property()))
        .unwrap();
    service
        .grant_property(PropertyGrant {
            owner_guid: OWNER,
            property: property(),
            lot_guid: HOUSE,
        })
        .unwrap();
    service
}

fn service_with_lot() -> HousingService {
    service_with_config(FixtureConfig::capture_defaults())
}

fn test_housing_content() -> HousingContent {
    HousingContent {
        schema_version: 1,
        provenance: "synthetic test content".to_string(),
        starter_property_id: 5,
        starter_kit: None,
        client_evidence: vec!["synthetic test row".to_string()],
        lots: vec![lot_definition(property())],
    }
}

fn owned(instance_id: u32, definition_id: i32, count: u32) -> OwnedInstance {
    OwnedInstance {
        instance_id,
        definition_id,
        tint_id: 0,
        count,
    }
}

/// A stand-in for the live inventory. The real `Inventory` needs private content, so the service
/// is generic over this seam and the real inventory is exercised separately below.
#[derive(Clone, Debug, Default)]
struct TestStock {
    instances: Vec<OwnedInstance>,
    refuse_consume: bool,
    refuse_return: bool,
}

impl TestStock {
    fn new(instances: Vec<OwnedInstance>) -> Self {
        Self {
            instances,
            refuse_consume: false,
            refuse_return: false,
        }
    }
}

impl FurnitureStock for TestStock {
    fn instances(&self) -> Vec<OwnedInstance> {
        self.instances.clone()
    }

    fn consume(&mut self, instance_id: u32, quantity: u32) -> Result<(), ServiceError> {
        if self.refuse_consume {
            return Err(ServiceError::StockRejected {
                operation: StockOperation::Consume,
                instance_id,
            });
        }
        let Some(entry) = self
            .instances
            .iter_mut()
            .find(|entry| entry.instance_id == instance_id)
        else {
            return Err(ServiceError::UnknownInstance { instance_id });
        };
        if entry.count < quantity {
            return Err(ServiceError::StockRejected {
                operation: StockOperation::Consume,
                instance_id,
            });
        }
        entry.count -= quantity;
        self.instances.retain(|entry| entry.count > 0);
        Ok(())
    }

    fn return_instance(&mut self, definition_id: i32, tint_id: i32) -> Result<(), ServiceError> {
        if self.refuse_return {
            return Err(ServiceError::StockRejected {
                operation: StockOperation::Return,
                instance_id: 0,
            });
        }
        let next = self
            .instances
            .iter()
            .map(|entry| entry.instance_id)
            .max()
            .unwrap_or(0)
            + 1;
        self.instances.push(OwnedInstance {
            instance_id: next,
            definition_id,
            tint_id,
            count: 1,
        });
        Ok(())
    }

    fn return_instances(&mut self, items: &[(i32, i32)]) -> Result<(), ServiceError> {
        let mut staged = self.clone();
        for &(definition_id, tint_id) in items {
            staged.return_instance(definition_id, tint_id)?;
        }
        *self = staged;
        Ok(())
    }

    fn first_free_instance_id(&self) -> Result<u32, ServiceError> {
        Ok(self
            .instances
            .iter()
            .map(|e| e.instance_id)
            .max()
            .unwrap_or(0)
            + 1)
    }

    /// Only the unit is recorded; this stand-in keeps no counter for the standing fixtures' ids.
    fn grant_kit(&mut self, item_id: i32, ids: &[u32]) -> Result<(), ServiceError> {
        if self.refuse_return {
            return Err(ServiceError::StockRejected {
                operation: StockOperation::Return,
                instance_id: 0,
            });
        }
        self.instances.push(owned(ids[0], item_id, 1));
        Ok(())
    }
}

fn opcodes(outcome: &housing_service::ServiceOutcome) -> Vec<u16> {
    outcome.replies.iter().map(HousingReply::opcode).collect()
}

fn decode_house_info(reply: &HousingReply) -> housing_native::HouseInfo {
    housing_native::HouseInfo::decode(&reply.encode().unwrap()).unwrap()
}

/// Reconstructed inventory content, built the same way `tests/inventory.rs` does: one synthetic
/// self-view packet whose coins/items/equipment spans are hash-pinned in `inventory.json`.
struct Content {
    directory: PathBuf,
}

impl Content {
    fn new(items: Vec<InventoryItem>, definitions: &[i32]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-housing-service-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let state = InventoryState {
            coins: 5,
            items,
            equipment: Vec::new(),
            next_instance_id: None,
        };
        let mut packet = vec![5, 0, 1, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0];
        let coins_offset = packet.len();
        packet.extend((state.coins as i32).to_le_bytes());
        let equipment_offset = packet.len();
        packet.extend(0i32.to_le_bytes());
        let items_offset = packet.len();
        packet.extend((state.items.len() as i32).to_le_bytes());
        for item in &state.items {
            for value in [
                item.definition_id,
                item.tint,
                item.id,
                item.count as i32,
                item.consumed_count as i32,
                i32::try_from(item.last_cast_time).unwrap(),
            ] {
                packet.extend(value.to_le_bytes());
            }
            packet.push(u8::from(item.rental.is_some()));
            packet.extend(item.ability_count.to_le_bytes());
            packet.push(u8::from(item.activate_enabled));
        }
        let items_length = packet.len() - items_offset;
        let length = packet.len();
        packet[3..7].copy_from_slice(&((length - 7) as u32).to_le_bytes());
        packet[9..13].copy_from_slice(&((length - 13) as u32).to_le_bytes());
        let span = |kind: &str, offset: usize, length: usize, profile_id: i32| json!({"kind":kind,"offset":offset,"length":length,"profile_id":profile_id,"sha256":digest(&packet[offset..offset+length])});
        let definitions: Vec<_> = definitions
            .iter()
            .map(|id| {
                let bytes = [id.to_le_bytes(), 8i32.to_le_bytes()].concat();
                json!({"id":id,"slot":3,"class_id":1,"default_tint":5,"max_stack":10,"model":"hsg_block_01.adr","texture":"plain","tint_alias":"dye","effect":0,"wield_type":0,"serialized_hex":bytes.iter().map(|byte|format!("{byte:02x}")).collect::<String>(),"sha256":digest(&bytes)})
            })
            .collect();
        let content = json!({
            "schema_version": 1,
            "player_guid": 1,
            "default_state": state,
            "profiles": [{"id":1,"allowed_classes":[1],"slots":[]}],
            "spans": [
                span("coins", coins_offset, 4, 0),
                span("equipment", equipment_offset, 4, 1),
                span("items", items_offset, items_length, 0),
            ],
            "definitions": definitions,
        });
        fs::write(
            directory.join("inventory.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { directory }
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.directory, None).unwrap()
    }
}

impl Drop for Content {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn item(instance_id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id: instance_id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

fn enter(
    service: &mut HousingService,
    stock: &mut dyn FurnitureStock,
) -> housing_service::ServiceOutcome {
    let outcome = service.handle(OWNER, &enter_frame(HOUSE), stock).unwrap();
    service.handle(OWNER, &edit_frame(true), stock).unwrap();
    outcome
}

fn placed_guid(service: &HousingService) -> u64 {
    service.lot(HOUSE).unwrap().fixtures[0].fixture_guid
}

fn place(
    service: &mut HousingService,
    stock: &mut dyn FurnitureStock,
) -> housing_service::ServiceOutcome {
    service.offer(OWNER, SELECTION, INSTANCE, stock).unwrap();
    service
        .handle(
            OWNER,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            stock,
        )
        .unwrap()
}

fn units(inventory: &Inventory, definition_id: i32) -> u32 {
    inventory
        .state()
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id)
        .map(|item| item.count)
        .sum()
}

#[test]
fn decode_request_keeps_the_selection_id_separate_from_the_transform() {
    let frame = place_frame(
        SELECTION,
        &block_transform([10.0, 0.0, 20.0, 1.0], ZERO_ROTATION, 1.0),
    );
    assert_eq!(frame.len(), 4 + 40);
    assert_eq!(&frame[..4], &[0x7f, 0x00, 0x02, 0x00]);
    match decode_request(&frame).unwrap() {
        Request::Place {
            selection_id,
            transform,
        } => {
            assert_eq!(selection_id, SELECTION);
            assert_eq!(
                transform,
                block_transform([10.0, 0.0, 20.0, 1.0], ZERO_ROTATION, 1.0)
            );
        }
        other => panic!("expected a place request, got {other:?}"),
    }
    assert_eq!(
        decode_request(&pickup_frame(9)).unwrap(),
        Request::Pickup { fixture_guid: 9 }
    );
    assert_eq!(
        decode_request(&enter_frame(HOUSE)).unwrap(),
        Request::Enter { house_guid: HOUSE }
    );
    assert_eq!(
        decode_request(&edit_frame(true)).unwrap(),
        Request::SetEditMode { in_edit_mode: true }
    );
    assert!(matches!(
        decode_request(&save_frame(9)),
        Ok(Request::Move {
            fixture_guid: 9,
            ..
        })
    ));
}

/// T847: the live client's `127/5`, byte for byte from the retail capture, decodes to the move's
/// transform with a zero dword, not the `127/2` placement scale.
#[test]
fn a_captured_move_frame_decodes_its_transform_and_zero_dword() {
    let request = decode_request(&RETAIL_MOVE).unwrap();
    let Request::Move {
        fixture_guid,
        transform,
    } = request
    else {
        panic!("a captured 127/5 is a move: {request:?}");
    };
    assert_eq!(fixture_guid, 0x07f5_4ca1_7bf5_c009);
    assert_eq!(transform.position[0], f32::from_bits(0x43db_bfff));
    assert_eq!(transform.position[1], f32::from_bits(0xbde2_c7ff));
    assert_eq!(transform.position[2], f32::from_bits(0x43d5_7fff));
    assert_eq!(transform.position[3], 1.0);
    assert_eq!(transform.rotation, [0.0; 4]);
    assert_eq!(
        transform.scale, 0.0,
        "the move dword is not a placement scale"
    );
    // The same captured tail in this suite's shape: the scale lane is not read from the frame.
    let frame = housing_move::move_frame(9, [10.0, 3.0, 20.0, 1.0], ZERO_ROTATION);
    assert_eq!(frame.len(), RETAIL_MOVE.len());
    assert_eq!(
        decode_request(&frame).unwrap(),
        Request::Move {
            fixture_guid: 9,
            transform: block_transform([10.0, 3.0, 20.0, 1.0], ZERO_ROTATION, 0.0),
        }
    );
}

#[test]
fn a_127_2_without_its_127_1_places_nothing() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    // The dword is the real item id, but only 127/1 binds it: a 127/2 alone places nothing.
    let error = service
        .handle(
            OWNER,
            &place_frame(INSTANCE, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap_err();
    assert_eq!(
        error,
        ServiceError::UnresolvedSelection {
            selection_id: INSTANCE
        }
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(stock.instances.len(), 1);
}

#[test]
fn a_native_placement_consumes_the_real_inventory_instance() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    let outcome = place(&mut service, &mut inventory);
    assert!(outcome.changed);
    assert_eq!(
        opcodes(&outcome),
        vec![REPLY_FIXTURE_UPDATE, REPLY_HOUSE_INFO]
    );

    let lot = service.lot(HOUSE).unwrap();
    assert_eq!(lot.count, 1);
    assert_eq!(lot.fixtures[0].fixture_definition_id, BLOCK_FIXTURE);
    assert_eq!(lot.fixtures[0].item_instance_id, INSTANCE);
    assert_eq!(lot.fixtures[0].transform.position, [10.0, 0.0, 20.0]);
    // The client sent no rotation, and retail keeps that as zeros facing +Z (T806).
    assert_eq!(lot.fixtures[0].transform.rotation, [0.0; 4]);
    assert_eq!(lot.fixtures[0].transform.tilt, [0.0, 0.0, 1.0, 0.0]);
    assert_eq!(
        units(&inventory, BLOCK_ITEM),
        0,
        "the real instance left the backpack"
    );

    let body = outcome.replies[0].encode_inner().unwrap();
    assert_eq!(&body[..4], &[0x7f, 0x00, 0x28, 0x00]);
    // 229 subtype-level bytes: 2 subtype + 133 fixture record + 94 tail. With the family prefix the
    // complete inner packet is 231 bytes under the current local `127/40` codec.
    assert_eq!(body.len(), 231);
    let update = housing_native::FixtureUpdate::decode(&body[2..]).unwrap();
    assert_eq!(update.instance.fixture_definition_id, BLOCK_FIXTURE);
    assert_eq!(update.info.item_definition_id, BLOCK_ITEM);
    assert_eq!(update.definition.id, BLOCK_FIXTURE);
    assert_eq!(update.definition.model_id, BLOCK_MODEL);
    assert_eq!(update.instance.tint_id, 0);

    let info = decode_house_info(&outcome.replies[1]);
    assert_eq!(info.cur_fixture_count, 1);
    assert!(info.in_edit_mode);
}

#[test]
fn pickup_returns_one_unit_and_conserves_the_owned_item() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let guid = placed_guid(&service);

    let outcome = service
        .handle(OWNER, &pickup_frame(guid), &mut inventory)
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(
        opcodes(&outcome),
        vec![REPLY_FIXTURE_REMOVE, REPLY_HOUSE_INFO]
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(units(&inventory, BLOCK_ITEM), 1);

    let mut expected = BASE_OPCODE.to_le_bytes().to_vec();
    expected.extend_from_slice(&41u16.to_le_bytes());
    expected.extend_from_slice(&guid.to_le_bytes());
    assert_eq!(outcome.replies[0].encode_inner().unwrap(), expected);
}

#[test]
fn a_stranger_cannot_enter_or_place_on_someone_elses_lot() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    assert_eq!(
        service
            .handle(GUEST, &enter_frame(HOUSE), &mut stock)
            .unwrap_err(),
        ServiceError::Housing(HousingError::NotPermitted)
    );
    assert_eq!(
        service
            .handle(
                GUEST,
                &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock,
            )
            .unwrap_err(),
        ServiceError::NotInHouse { actor_guid: GUEST }
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(stock.instances.len(), 1);
}

#[test]
fn a_guest_with_edit_permission_can_build_but_cannot_reassign_the_lot() {
    let mut service = service_with_lot();
    service
        .set_permission(OWNER, HOUSE, GUEST, PERMISSION_CAN_EDIT)
        .unwrap();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    service
        .handle(GUEST, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    service
        .handle(GUEST, &edit_frame(true), &mut stock)
        .unwrap();
    service.offer(GUEST, SELECTION, INSTANCE, &stock).unwrap();
    let outcome = service
        .handle(
            GUEST,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(service.lot(HOUSE).unwrap().count, 1);
    assert_eq!(
        service
            .set_permission(GUEST, HOUSE, STRANGER, PERMISSION_CAN_EDIT)
            .unwrap_err(),
        ServiceError::Housing(HousingError::NotPropertyOwner)
    );
}

#[test]
fn a_refused_stock_mutation_rolls_the_staged_placement_back() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    stock.refuse_consume = true;
    enter(&mut service, &mut stock);
    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    let error = service
        .handle(
            OWNER,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap_err();
    assert_eq!(
        error,
        ServiceError::StockRejected {
            operation: StockOperation::Consume,
            instance_id: INSTANCE,
        }
    );
    assert_eq!(
        service.lot(HOUSE).unwrap().count,
        0,
        "a refused inventory mutation must not leave a staged fixture behind"
    );
    assert_eq!(stock.instances.len(), 1);
    assert_eq!(service.state().lots[0].fixtures.len(), 0);
}

#[test]
fn furniture_the_backpack_does_not_own_is_refused_without_a_staged_change() {
    let mut service = service_with_lot();
    let mut empty = TestStock::default();
    enter(&mut service, &mut empty);
    assert_eq!(
        service
            .offer(OWNER, SELECTION, INSTANCE, &empty)
            .unwrap_err(),
        ServiceError::UnknownInstance {
            instance_id: INSTANCE
        }
    );

    // The instance disappears between the preview and the placement commit.
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    stock.instances.clear();
    assert_eq!(
        service
            .handle(
                OWNER,
                &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock,
            )
            .unwrap_err(),
        ServiceError::UnknownInstance {
            instance_id: INSTANCE
        }
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
}

#[test]
fn a_furniture_stack_cannot_back_a_placed_fixture() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 2)]);
    enter(&mut service, &mut stock);
    assert_eq!(
        service
            .offer(OWNER, SELECTION, INSTANCE, &stock)
            .unwrap_err(),
        ServiceError::StackedInstanceUnsupported {
            instance_id: INSTANCE,
            count: 2,
        }
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
}

#[test]
fn furniture_without_one_unambiguous_fixture_row_is_refused() {
    let mut service = service_with_lot();
    let mut unknown = TestStock::new(vec![owned(INSTANCE, 4_242, 1)]);
    enter(&mut service, &mut unknown);
    assert_eq!(
        service
            .offer(OWNER, SELECTION, INSTANCE, &unknown)
            .unwrap_err(),
        ServiceError::FixtureDefinitionUnknown {
            definition_id: 4_242
        }
    );

    let mut config = FixtureConfig::capture_defaults();
    config.rows.push(FixtureConfigRow {
        fixture_definition_id: 900,
        item_definition_id: BLOCK_ITEM,
        model_id: 1,
        category: String::new(),
        lua_call: String::new(),
        texture_alias: String::new(),
        tint_alias: String::new(),
        default_tint_id: 0,
        provenance: "test-only duplicate row".to_string(),
    });
    let mut ambiguous = HousingService::new(config);
    ambiguous
        .register_house_definition(lot_definition(property()))
        .unwrap();
    ambiguous
        .grant_property(PropertyGrant {
            owner_guid: OWNER,
            property: property(),
            lot_guid: HOUSE,
        })
        .unwrap();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut ambiguous, &mut stock);
    assert_eq!(
        ambiguous
            .offer(OWNER, SELECTION, INSTANCE, &stock)
            .unwrap_err(),
        ServiceError::FixtureDefinitionAmbiguous {
            definition_id: BLOCK_ITEM
        }
    );
}

#[test]
fn malformed_requests_leave_the_service_state_intact() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    let before = service.state();
    let cases = vec![
        (vec![0x7f, 0x00], RequestError::TooShort),
        (
            vec![0x7e, 0x00, 0x03, 0x00],
            RequestError::WrongBaseOpcode { found: 126 },
        ),
        (
            vec![0x7f, 0x00, 0x63, 0x00],
            RequestError::UnknownOpcode { opcode: 99 },
        ),
        (
            place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0))[..20].to_vec(),
            RequestError::BodyRejected,
        ),
        (
            {
                let mut frame = pickup_frame(9);
                frame.push(0);
                frame
            },
            RequestError::TrailingBytes { count: 1 },
        ),
    ];
    for (frame, expected) in cases {
        assert_eq!(
            service.handle(OWNER, &frame, &mut stock).unwrap_err(),
            ServiceError::MalformedRequest(expected)
        );
    }
    assert_eq!(service.state(), before);
}

#[test]
fn recovered_entry_packets_encode_and_preview_without_selector_stays_unrecovered() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    let entered = service
        .handle(OWNER, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    assert_eq!(
        opcodes(&entered),
        vec![45, REPLY_INSTANCE_DATA, REPLY_HOUSE_INFO, 43]
    );
    assert!(matches!(
        entered.zone_change,
        Some(HousingZoneChange::EnterHouse {
            house_guid: HOUSE,
            ..
        })
    ));
    assert!(!entered.changed);
    assert!(
        entered
            .replies
            .iter()
            .all(|reply| reply.encode_inner().is_ok())
    );

    let preview = service
        .handle(OWNER, &preview_frame(BLOCK_ITEM as u32), &mut stock)
        .unwrap();
    assert_eq!(opcodes(&preview), vec![REPLY_PLACE_REQUEST]);
    assert!(!preview.changed);

    assert_eq!(
        decode_request(&save_frame(9)).unwrap(),
        Request::Move {
            fixture_guid: 9,
            transform: block_transform([11.0, 0.0, 21.0, 1.0], ZERO_ROTATION, 1.0),
        }
    );
}

#[test]
fn fixture_asset_request_resolves_only_a_bound_owned_selection() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    // An unbound id is tried as the owned item guid; SELECTION names no owned item.
    assert_eq!(
        service
            .handle(OWNER, &place_request_frame(SELECTION), &mut stock)
            .unwrap_err(),
        ServiceError::UnknownInstance {
            instance_id: SELECTION
        }
    );

    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    let response = service
        .handle(OWNER, &place_request_frame(SELECTION), &mut stock)
        .unwrap();
    assert_eq!(opcodes(&response), vec![REPLY_PLACE_REQUEST]);
    let asset =
        housing_native::FixtureAsset::decode(&response.replies[0].encode().unwrap()).unwrap();
    assert_eq!(asset.selection_id, SELECTION);
    assert_eq!(asset.model_id, BLOCK_MODEL as u32);
    assert_eq!(asset.definition.id, BLOCK_FIXTURE);
    assert_eq!(asset.definition.item_definition_id, BLOCK_ITEM);
    assert_eq!(asset.definition.model_id, BLOCK_MODEL);
    // T783: the client's validity check (`sub_ABB630` `0x00ABB6AD`) needs placement type 1, 0xE
    // or 0xF, or no preview is ever valid and no 127/2 is sent; retail sent 1.
    assert_eq!(asset.definition.unknown3, 1, "placement type");
    assert_eq!(
        asset.definition.flags,
        [false, false, false, true, false, false]
    );
    assert_eq!(
        (asset.definition.unknown14, asset.definition.unknown15),
        (1.0, 1.0)
    );
    assert_eq!(
        stock.instances.len(),
        1,
        "request does not consume the owned item"
    );
}

#[test]
fn fixture_item_list_contains_only_configured_owned_items() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    let entered = service
        .handle(OWNER, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    let list =
        housing_native::FixtureItemList::decode(&entered.replies[3].encode().unwrap()).unwrap();
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].fixture_guid, u64::from(INSTANCE));
    assert_eq!(list.items[0].item_definition_id, BLOCK_ITEM);
    assert_eq!(list.definitions.len(), 1);
    assert_eq!(list.definitions[0].id, BLOCK_FIXTURE);
}

#[test]
fn edit_mode_is_reported_in_typed_house_info() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    let entered = service
        .handle(OWNER, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    let info = decode_house_info(&entered.replies[2]);
    assert!(!info.in_edit_mode);
    assert_eq!(info.unused, 10);
    assert!(info.is_flora_allowed);
    assert_eq!(info.cur_fixture_count, 0);

    let outcome = service
        .handle(OWNER, &edit_frame(true), &mut stock)
        .unwrap();
    assert!(!outcome.changed);
    assert_eq!(opcodes(&outcome), vec![REPLY_HOUSE_INFO]);
    assert!(decode_house_info(&outcome.replies[0]).in_edit_mode);
}

#[test]
fn a_denied_request_encodes_the_recovered_53_body() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    let error = service
        .handle(
            OWNER,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap_err();

    let body = error.denial_reply().encode_inner().unwrap();
    assert_eq!(&body[..4], &[0x7f, 0x00, 0x35, 0x00]);
    let message = housing_native::PlacementDenied::decode(&body[2..]).unwrap();
    assert_eq!(message.message, error.denial_message());
    assert!(!message.message.is_empty());
}

#[test]
fn a_non_finite_rotation_is_refused_and_rolled_back() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    let error = service
        .handle(
            OWNER,
            &place_frame(
                SELECTION,
                &block_transform(position(), [f32::NAN, 0.0, 0.0, 0.0], 1.0),
            ),
            &mut stock,
        )
        .unwrap_err();
    assert_eq!(
        error,
        ServiceError::MalformedRequest(RequestError::BodyRejected)
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(stock.instances.len(), 1);
}

/// The client's rotation lanes are Euler angles, not a quaternion: a quarter turn and a tipped
/// turn place (T806; both were refused as a "non-unit rotation"), the record keeps the lanes and
/// the tilt is the facing they imply, as in all 2,015 instances of the community-preserved p2.pcap.
#[test]
fn a_client_euler_rotation_places_and_the_tilt_is_its_facing() {
    for (yaw, pitch, facing) in [
        (
            std::f32::consts::FRAC_PI_2,
            0.0_f32,
            [1.0_f32, 0.0, 0.0, 0.0],
        ),
        (0.2618, 0.0, [0.2588, 0.0, 0.9659, 0.0]),
        (-1.5708, -0.3285, [-0.9465, 0.3226, 0.0, 0.0]),
    ] {
        let mut service = service_with_lot();
        let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
        enter(&mut service, &mut stock);
        service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
        let rotation = [yaw, pitch, 0.0, 0.0];
        let placed = service
            .handle(
                OWNER,
                &place_frame(SELECTION, &block_transform(position(), rotation, 1.0)),
                &mut stock,
            )
            .unwrap();
        let update =
            housing_native::FixtureUpdate::decode(&placed.replies[0].encode().unwrap()).unwrap();
        assert_eq!(update.instance.rotation, rotation);
        for (lane, (got, want)) in update.instance.tilt.iter().zip(facing).enumerate() {
            assert!(
                (got - want).abs() < 1e-3,
                "{rotation:?} tilt lane {lane}: {got} vs {want}"
            );
        }
        let [housing_service::FixtureActor::Spawn(npc)] = &placed.actors[..] else {
            panic!("one AddNpc: {:?}", placed.actors)
        };
        assert_eq!((npc.rotation, npc.tilt), (update.instance.tilt, rotation));
    }
}

#[test]
fn a_typed_move_stages_the_transform_and_keeps_the_item_placed() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let guid = placed_guid(&service);

    let outcome = service
        .handle(
            OWNER,
            &move_frame(
                guid,
                &block_transform([11.0, 0.0, 21.0, 1.0], ZERO_ROTATION, 1.0),
            ),
            &mut inventory,
        )
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(opcodes(&outcome), vec![REPLY_POSITION, REPLY_HOUSE_INFO]);
    let position =
        housing_native::PositionUpdate::decode(&outcome.replies[0].encode().unwrap()).unwrap();
    // The capture's 127/51 names the NPC that draws the fixture, not the fixture (T806).
    assert_eq!(
        position.actor_guid,
        housing_service::fixture_actor_guid(guid)
    );
    assert_ne!(position.actor_guid, guid);
    assert_eq!(position.position, [11.0, 0.0, 21.0, 1.0]);
    assert_eq!(position.orientation, ZERO_ROTATION);
    let lot = service.lot(HOUSE).unwrap();
    assert_eq!(lot.count, 1);
    assert_eq!(lot.fixtures[0].transform.position, [11.0, 0.0, 21.0]);
    assert_eq!(units(&inventory, BLOCK_ITEM), 0);

    assert_eq!(
        service
            .move_fixture(
                STRANGER,
                HOUSE,
                guid,
                block_transform([12.0, 0.0, 21.0, 1.0], ZERO_ROTATION, 1.0),
                &mut inventory,
            )
            .unwrap_err(),
        ServiceError::Housing(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        service
            .handle(
                OWNER,
                &move_frame(
                    guid,
                    &block_transform([900.0, 0.0, 21.0, 1.0], ZERO_ROTATION, 1.0)
                ),
                &mut inventory
            )
            .unwrap_err(),
        ServiceError::Housing(HousingError::OutsideBuildArea)
    );
    assert_eq!(
        service.lot(HOUSE).unwrap().fixtures[0].transform.position,
        [11.0, 0.0, 21.0]
    );
}

/// T847: the live client's move dword is 0 where `127/2` carries the scale. The service must keep
/// the placed fixture's scale (a 0 scale is out of bounds), so a real frame moves the fixture and
/// the same fixture can then be lowered again.
#[test]
fn a_real_move_frame_with_its_zero_dword_moves_and_keeps_the_placed_scale() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let guid = placed_guid(&service);

    let raised = [11.0, 5.0, 21.0, 1.0];
    let up = service
        .handle(
            OWNER,
            &housing_move::move_frame(guid, raised, ZERO_ROTATION),
            &mut inventory,
        )
        .unwrap();
    assert!(up.changed);
    assert_eq!(opcodes(&up), vec![REPLY_POSITION, REPLY_HOUSE_INFO]);
    let position =
        housing_native::PositionUpdate::decode(&up.replies[0].encode().unwrap()).unwrap();
    assert_eq!(position.position, raised);
    let lot = service.lot(HOUSE).unwrap();
    assert_eq!(lot.fixtures[0].transform.position, [11.0, 5.0, 21.0]);
    assert_eq!(
        lot.fixtures[0].transform.scale, 1.0,
        "a move keeps the scale it was placed with"
    );

    // The raised fixture can come down again through the same path.
    let lowered = [11.0, 0.0, 21.0, 1.0];
    let down = service
        .handle(
            OWNER,
            &housing_move::move_frame(guid, lowered, ZERO_ROTATION),
            &mut inventory,
        )
        .unwrap();
    assert!(down.changed);
    assert_eq!(
        service.lot(HOUSE).unwrap().fixtures[0].transform.position,
        [11.0, 0.0, 21.0]
    );
    assert_eq!(units(&inventory, BLOCK_ITEM), 0, "the item stays placed");
}

/// T826 (native 2026-10-27): every radial-menu action (move, rotate, remove) reached the client's
/// housing code as guid 0 and rotating crashed it. The menu's Lua gets the selected fixture's guid
/// as the data source's `%lld` print (`00cbd700`) and its guid reader `00bfea90` returns 0 for a
/// string that starts with `-`, so a guid with bit 63 set can never be acted on. Bit 55 is the
/// actor's (`fixture_actor_guid`).
#[test]
fn a_minted_fixture_guid_prints_as_a_positive_decimal() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let guid = placed_guid(&service);
    assert_eq!(guid >> 63, 0, "bit 63 clear: {guid:#x}");
    assert_eq!((guid >> 55) & 1, 0, "bit 55 is the actor's: {guid:#x}");
    let printed = format!("{}", guid as i64);
    assert!(!printed.starts_with('-'), "the client prints {printed}");
    let actor = housing_service::fixture_actor_guid(guid);
    assert!(!format!("{}", actor as i64).starts_with('-'));
    assert_ne!(actor, guid);
}

/// A lot saved before T806/T826 holds guids with bit 63 set and, for a fixture the client placed
/// unrotated, the identity-mapped pair rotation `[1, 0, 0, 0]` / tilt `[1, 0, 0, 0]`. Loading
/// brings both to the current records, keeps everything else, and a second load changes nothing.
#[test]
fn a_lot_saved_with_the_old_guids_and_rotation_loads_in_the_current_form() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let current = service.state();
    let new_guid = current.lots[0].fixtures[0].fixture_guid;

    let mut old = current.clone();
    old.lots[0].fixtures[0].fixture_guid = new_guid | (1 << 63);
    old.lots[0].fixtures[0].transform.rotation = [1.0, 0.0, 0.0, 0.0];
    old.lots[0].fixtures[0].transform.tilt = [1.0, 0.0, 0.0, 0.0];
    let restored =
        HousingService::from_state(FixtureConfig::capture_defaults(), &old, inventory.state())
            .unwrap();
    assert_eq!(
        restored.state(),
        current,
        "the old form loads as the current one"
    );
    let fixture = &restored.lot(HOUSE).unwrap().fixtures[0];
    assert_eq!(fixture.fixture_guid, new_guid);
    assert_eq!(fixture.transform.rotation, [0.0; 4]);
    assert_eq!(fixture.transform.tilt, [0.0, 0.0, 1.0, 0.0]);
    let again = HousingService::from_state(
        FixtureConfig::capture_defaults(),
        &restored.state(),
        inventory.state(),
    )
    .unwrap();
    assert_eq!(again.state(), restored.state());

    // A rotation the client really sent is never mistaken for the old pair: one radian of yaw
    // comes with its own facing, not the tilt [1, 0, 0, 0].
    let mut turned = current.clone();
    turned.lots[0].fixtures[0].transform.rotation = [1.0, 0.0, 0.0, 0.0];
    turned.lots[0].fixtures[0].transform.tilt = [1.0_f32.sin(), 0.0, 1.0_f32.cos(), 0.0];
    let kept = HousingService::from_state(
        FixtureConfig::capture_defaults(),
        &turned,
        inventory.state(),
    )
    .unwrap();
    assert_eq!(kept.state(), turned);
}

/// T806 review finding 2: a saved tilt can be a unit vector and still disagree with its rotation.
/// The rotation is the one source, so load derives the tilt instead of trusting the stored pair.
#[test]
fn a_saved_unit_tilt_is_derived_from_its_rotation_on_load() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    place(&mut service, &mut inventory);
    let mut state = service.state();
    let rotation = [0.5, 0.5, 0.25, 0.0];
    state.lots[0].fixtures[0].transform.rotation = rotation;
    state.lots[0].fixtures[0].transform.tilt = [1.0, 0.0, 0.0, 0.0];

    let restored =
        HousingService::from_state(FixtureConfig::capture_defaults(), &state, inventory.state())
            .unwrap();
    let transform = &restored.lot(HOUSE).unwrap().fixtures[0].transform;
    assert_eq!(transform.tilt, FixtureTransform::facing(rotation));
}

#[test]
fn the_serialized_service_state_round_trips_and_rejects_invalid_documents() {
    let content = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content.inventory();
    let mut service = service_with_lot();
    enter(&mut service, &mut inventory);
    let before_place = inventory.state().clone();
    place(&mut service, &mut inventory);

    let state = service.state();
    let json = serde_json::to_string(&state).unwrap();
    let decoded: HousingServiceState = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, state);
    let restored = HousingService::from_state(
        FixtureConfig::capture_defaults(),
        &decoded,
        inventory.state(),
    )
    .unwrap();
    assert_eq!(restored.state(), state);
    assert_eq!(restored.lot(HOUSE).unwrap().count, 1);
    assert_eq!(
        restored.lot(HOUSE).unwrap().fixtures[0].item_instance_id,
        INSTANCE
    );

    // The placed instance is no longer in the backpack, so it must not still be owned.
    assert_eq!(
        HousingService::from_state(FixtureConfig::capture_defaults(), &state, &before_place)
            .unwrap_err(),
        StateError::PlacedInstanceStillOwned {
            instance_id: INSTANCE
        }
    );

    let mut version = state.clone();
    version.schema_version = 99;
    assert_eq!(
        HousingService::from_state(
            FixtureConfig::capture_defaults(),
            &version,
            inventory.state()
        )
        .unwrap_err(),
        StateError::SchemaVersion { found: 99 }
    );

    let mut heavy = state.clone();
    heavy.lots = vec![state.lots[0].clone(); 65];
    assert_eq!(
        HousingService::from_state(FixtureConfig::capture_defaults(), &heavy, inventory.state())
            .unwrap_err(),
        StateError::TooManyLots { count: 65 }
    );

    let mut corrupt = state.clone();
    corrupt.lots[0].count = 2;
    assert_eq!(
        HousingService::from_state(
            FixtureConfig::capture_defaults(),
            &corrupt,
            inventory.state()
        )
        .unwrap_err(),
        StateError::Housing(HousingError::CorruptSnapshot)
    );

    let mut duplicated = state.clone();
    let mut second = state.lots[0].clone();
    second.lot_guid = HOUSE + 1;
    second.property_id = 6;
    second.fixtures[0].fixture_guid += 1;
    second.fixtures[0].house_guid = HOUSE + 1;
    duplicated.lots.push(second);
    assert_eq!(
        HousingService::from_state(
            FixtureConfig::capture_defaults(),
            &duplicated,
            inventory.state(),
        )
        .unwrap_err(),
        StateError::PlacedInstanceDuplicated {
            instance_id: INSTANCE
        }
    );
}

#[test]
fn the_reconstructed_default_catalog_has_source_tagged_fixture_rows() {
    let config = FixtureConfig::capture_defaults();
    let row = config.row_for_item_definition(BLOCK_ITEM).unwrap();
    assert_eq!(row.fixture_definition_id, BLOCK_FIXTURE);
    assert_eq!(row.model_id, BLOCK_MODEL);
    assert!(
        row.provenance
            .starts_with("R:unverified-legacy-packet-derived-artifact")
    );
    assert_eq!(
        config.row_for_item_definition(4_242).unwrap_err(),
        ServiceError::FixtureDefinitionUnknown {
            definition_id: 4_242
        }
    );

    // An unusable table is refused before any state is adopted, including the empty-state path.
    let mut empty_config = FixtureConfig::capture_defaults();
    empty_config.rows.clear();
    let empty_state = HousingServiceState {
        schema_version: SERVICE_SCHEMA_VERSION,
        lots: Vec::new(),
    };
    let empty_inventory = InventoryState {
        coins: 0,
        items: Vec::new(),
        equipment: Vec::new(),
        next_instance_id: None,
    };
    assert_eq!(
        HousingService::from_state(empty_config, &empty_state, &empty_inventory).unwrap_err(),
        StateError::Config(ServiceError::FixtureConfigInvalid(
            "empty fixture configuration"
        ))
    );
    // A row without a model id could put 0 in the 127/42 model dword; it is refused at load.
    let mut no_model = FixtureConfig::capture_defaults();
    no_model.rows[0].model_id = 0;
    assert_eq!(
        HousingService::from_state(no_model, &empty_state, &empty_inventory).unwrap_err(),
        StateError::Config(ServiceError::FixtureConfigInvalid(
            "invalid fixture configuration id"
        ))
    );
    assert_eq!(
        HousingService::from_state(
            FixtureConfig::capture_defaults(),
            &empty_state,
            &empty_inventory,
        )
        .unwrap()
        .state(),
        empty_state
    );
}

/// The generated table the server ships in its content directory (T518/T869).
fn bundled_fixtures() -> FixtureConfig {
    FixtureConfig::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/housing_fixtures.json"),
    )
    .unwrap()
}

#[test]
fn the_bundled_fixture_table_covers_the_placeable_items() {
    let config = bundled_fixtures();
    config.validate().expect("the generated table validates");
    assert_eq!(
        config.rows.len(),
        1_198,
        "one row per placeable item: T518's item rows plus T869's model-joined placement rows"
    );
    // The generator sorts by item id, so a regenerated table reviews as a small diff.
    assert!(
        config
            .rows
            .windows(2)
            .all(|pair| pair[0].item_definition_id < pair[1].item_definition_id),
        "rows ascend by item id"
    );
    // The seven packet-captured rows keep their captured fixture and model ids.
    for (item, fixture, model) in [
        (7_466, 34, 1_926),
        (14_300, 902, 2_159),
        (77_259, 490, 1_988),
        (7_462, 323, 1_945),
        (16_039, 68, 3_002),
        (18_110, 1_074, 3_058),
        (16_040, 69, 3_003),
    ] {
        let row = config.row_for_item_definition(item).unwrap();
        assert_eq!(
            (row.fixture_definition_id, row.model_id),
            (fixture, model),
            "item {item}"
        );
    }
    assert_eq!(
        config.row_for_item_definition(70_429).unwrap().model_id,
        535
    );
    assert_eq!(
        config.row_for_item_definition(32_647).unwrap().model_id,
        1_926
    );
    // An item without an icon tint or a texture alias keeps empty/zero values, not an error.
    let chicken = config.row_for_item_definition(70_429).unwrap();
    assert_eq!(
        (chicken.texture_alias.as_str(), chicken.default_tint_id),
        ("", 0)
    );
    // The Basic Square Block carries the decorate preview's aliases and its icon tint.
    let block = config.row_for_item_definition(32_647).unwrap();
    assert_eq!(
        (
            block.texture_alias.as_str(),
            block.tint_alias.as_str(),
            block.default_tint_id,
        ),
        ("fun-block-L1", "dyetint", 227)
    );
    // One row per item definition and per fixture definition, never a model id of 0, and a
    // source for every row.
    let items: BTreeSet<i32> = config
        .rows
        .iter()
        .map(|row| row.item_definition_id)
        .collect();
    let fixtures: BTreeSet<i32> = config
        .rows
        .iter()
        .map(|row| row.fixture_definition_id)
        .collect();
    assert_eq!(items.len(), config.rows.len(), "unique item ids");
    assert_eq!(fixtures.len(), config.rows.len(), "unique fixture ids");
    assert!(config.rows.iter().all(|row| row.model_id > 0));
    assert!(config.rows.iter().all(|row| !row.provenance.is_empty()));
    // T869: the Lxzxrus placement table is joined by model name, never by its Id column. Its
    // extra ids are runtime-generated Lxzxrus items that collide with real client items, so an
    // item is a fixture only when its own ModelName equals a reference ASSET_NAME. That finds the
    // boombox items below; item 560 (military boots) and 563 (a helmet) are not fixtures even
    // though the reference maps their ids to a brazier and a palisade.
    let placement_rows: Vec<&FixtureConfigRow> = config
        .rows
        .iter()
        .filter(|row| row.provenance.contains("HousingPlacementData.txt"))
        .collect();
    assert_eq!(placement_rows.len(), 12, "model-joined placement rows");
    assert_eq!(
        placement_rows
            .iter()
            .map(|row| row.item_definition_id)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            878, 1_180, 1_913, 8_185, 35_759, 56_774, 69_825, 76_928, 76_930, 76_933, 76_935,
            77_329,
        ]),
        "only the items whose own ModelName is a reference asset are added"
    );
    let placement_pin = concat!(
        "Lxzxrus-Free-Realms-Server@37ef17587d43e9b33bfb10e5484518f61a82c6be ",
        "(AGPL-3.0) src/Resources/HousingPlacementData.txt"
    );
    assert!(
        placement_rows
            .iter()
            .all(|row| row.provenance.contains(placement_pin)),
        "every placement row carries repository, licence, revision and file"
    );
    for item in [560, 563] {
        assert!(
            config.row_for_item_definition(item).is_err(),
            "item {item} is not a fixture"
        );
    }
    for item in [10_451, 11_494, 15_971, 16_193, 76_878] {
        assert!(
            config.row_for_item_definition(item).is_err(),
            "item {item} is not in the pinned item data"
        );
    }
    // The new rows carry the model of the item's own ModelName (the same client Models.txt id).
    for (item, model) in [(878, 1_936), (69_825, 4_428), (76_928, 2_201)] {
        assert_eq!(
            config.row_for_item_definition(item).unwrap().model_id,
            model,
            "item {item}"
        );
    }
}

/// Canary: a duplicated row must fail validation (the generator refuses the same case).
#[test]
fn a_duplicated_fixture_row_is_refused_at_load() {
    let mut config = bundled_fixtures();
    let first = config.rows[0].clone();
    config.rows.push(first.clone());
    assert_eq!(
        config.validate().unwrap_err(),
        ServiceError::FixtureConfigInvalid("duplicate fixture definition id")
    );
    config.rows.pop();
    let mut same_item = first;
    same_item.fixture_definition_id = i32::MAX;
    config.rows.push(same_item);
    assert_eq!(
        config.validate().unwrap_err(),
        ServiceError::FixtureConfigInvalid("duplicate item definition id")
    );
    // The load path refuses the whole table before any lot is adopted.
    let empty = HousingServiceState {
        schema_version: SERVICE_SCHEMA_VERSION,
        lots: Vec::new(),
    };
    assert!(matches!(
        HousingService::from_state(config, &empty, &no_items()),
        Err(StateError::Config(ServiceError::FixtureConfigInvalid(
            "duplicate item definition id"
        )))
    ));
}

#[test]
fn a_save_made_with_the_capture_defaults_reloads_with_the_generated_table() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    place(&mut service, &mut stock);
    let state = service.state();
    assert_eq!(
        state.lots[0].fixtures[0].fixture_definition_id, BLOCK_FIXTURE,
        "the capture-defaults save holds fixture 34"
    );
    let restored = HousingService::from_state(bundled_fixtures(), &state, &no_items()).unwrap();
    assert_eq!(
        restored.state(),
        state,
        "the placed fixture survives the swap"
    );
    let row = restored
        .config()
        .row_for_item_definition(BLOCK_ITEM)
        .unwrap();
    assert_eq!(
        (row.fixture_definition_id, row.model_id),
        (BLOCK_FIXTURE, BLOCK_MODEL),
        "fixture 34 stays the Blue Square Block's definition"
    );
}

#[test]
fn a_127_42_for_a_generated_row_carries_its_model_aliases_and_tint() {
    let mut service = service_with_config(bundled_fixtures());
    let mut stock = TestStock::new(vec![owned(INSTANCE, KIT_BLOCK, 1)]);
    enter(&mut service, &mut stock);
    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    let response = service
        .handle(OWNER, &place_request_frame(SELECTION), &mut stock)
        .unwrap();
    assert_eq!(opcodes(&response), vec![REPLY_PLACE_REQUEST]);
    let asset =
        housing_native::FixtureAsset::decode(&response.replies[0].encode().unwrap()).unwrap();
    assert_eq!(asset.model_id, BLOCK_MODEL as u32);
    assert_eq!(asset.definition.model_id, BLOCK_MODEL);
    assert_eq!(asset.texture_alias, "fun-block-L1");
    assert_eq!(asset.tint_alias, "dyetint");
    assert_eq!(asset.tint_id, 227);
    // The same row dresses the fixture's actor; the 127/40 record itself carries no look.
    let placed = service
        .handle(
            OWNER,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap();
    assert_eq!(
        opcodes(&placed),
        vec![REPLY_FIXTURE_UPDATE, REPLY_HOUSE_INFO]
    );
    let update =
        housing_native::FixtureUpdate::decode(&placed.replies[0].encode().unwrap()).unwrap();
    assert_eq!(update.instance.customization.texture_alias, "");
    assert_eq!(update.instance.customization.tint_alias, "");
    let [housing_service::FixtureActor::Spawn(npc)] = &placed.actors[..] else {
        panic!("one AddNpc: {:?}", placed.actors)
    };
    assert_eq!(
        (npc.texture_alias.as_str(), npc.tint_alias.as_str()),
        ("fun-block-L1", "dyetint")
    );
    assert_eq!(
        npc.tint_id, 227,
        "the item's default tint, as the capture's fun-block actors"
    );
}

#[test]
fn the_housing_snapshot_seam_is_path_free() {
    // The service state never names a file: the lead serializes it into the canonical snapshot.
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    place(&mut service, &mut stock);
    let state = service.state();
    let encoded = serde_json::to_value(&state).unwrap();
    assert_eq!(encoded["schema_version"], SERVICE_SCHEMA_VERSION);
    assert!(encoded.get("path").is_none());
    let round_trip: HousingServiceState = serde_json::from_value(encoded).unwrap();
    assert_eq!(round_trip, state);
}

/// T516/F689: the row provenance names only sources this table is allowed to carry.
/// `scene_source` says where the property-to-scene association came from (enabled rows);
/// `bounds_source` says where the six-float box came from. A new source must be added here
/// deliberately, so a typo or a fabricated source fails this test.
const SCENE_SOURCES: [&str; 4] = [
    "client-manifest (R)",
    "community-manifest (R)",
    "lxzxrus-scout (R)",
    "lxzxrus-zone (R)",
];
const BOUNDS_SOURCES: [&str; 3] = ["legacy (R)", "lxzxrus-whereami (R)", "lxzxrus-zone (R)"];

#[test]
fn enabled_bundled_lots_have_scene_bounds_and_source_tags() {
    let content = HousingContent::bundled().unwrap();
    assert_eq!(content.lots.len(), 31);
    assert_eq!(content.lots.iter().filter(|lot| lot.enabled).count(), 23);
    for lot in &content.lots {
        assert!(!lot.source_tag.is_empty());
        assert!(!lot.build_areas.is_empty());
        assert!(
            BOUNDS_SOURCES.contains(&lot.bounds_source.as_str()),
            "property {} bounds_source {:?}",
            lot.property_id,
            lot.bounds_source
        );
        if lot.enabled {
            assert!(
                lot.scene
                    .as_ref()
                    .is_some_and(|scene| !scene.name.is_empty())
            );
            let scene_source = lot
                .scene_source
                .as_deref()
                .expect("an enabled lot has a scene source");
            assert!(
                SCENE_SOURCES.contains(&scene_source),
                "property {} scene_source {:?}",
                lot.property_id,
                scene_source
            );
            assert!(lot.disabled_reason.is_none());
        } else {
            assert!(lot.scene.is_none());
            assert!(lot.scene_source.is_none());
            assert!(
                lot.disabled_reason
                    .as_ref()
                    .is_some_and(|reason| !reason.is_empty())
            );
        }
    }
    assert!(
        content
            .client_evidence
            .iter()
            .any(|source| source.contains("no client lot/zone/build-box table"))
    );
}

/// T516/F689: the client's `Resources/SKY` files that enabled lots name (working client
/// `lab/working/OSFR.Client/OSFR Client/Resources/SKY`, listed 2026-10-25). A new or changed sky
/// must be re-listed there and added here, so an invented name cannot pass.
const SHIPPED_SKIES: [&str; 15] = [
    "sky_bat_cave.xml",
    "sky_blackspore24.xml",
    "sky_briarwood24.xml",
    "sky_bristlewood_glade.xml",
    "sky_deep_mines.xml",
    "sky_hidden_valley24.xml",
    "sky_housing_wilds.xml",
    "sky_merryvale24.xml",
    "sky_seaside24.xml",
    "sky_shrouded_gloam.xml",
    "sky_snowhill24.xml",
    "sky_snowhill_housinglot_01.xml",
    "sky_sunset.xml",
    "sky_sunstone_valley24.xml",
    "sky_wugachug24.xml",
];

/// T516: every enabled lot has its own scene, and every enabled lot but the lead-owned starter
/// has a real sky file, never the placeholder.
///
/// F689: every enabled row except the lead-owned starter names a sky from `SHIPPED_SKIES`. The
/// starter (property 25, L030) keeps `sky_test.xml` until the lead accepts the scout's
/// `sky_sunstone_valley24`; pinning the placeholder here makes that a deliberate change.
#[test]
fn enabled_lots_have_distinct_scenes_and_shipped_skies() {
    let content = HousingContent::bundled().unwrap();
    let mut scenes = BTreeSet::new();
    for lot in content.lots.iter().filter(|lot| lot.enabled) {
        let scene = lot.scene.as_ref().expect("an enabled lot has a scene");
        assert!(
            scenes.insert(scene.name.as_str()),
            "property {} reuses scene {}",
            lot.property_id,
            scene.name
        );
        if lot.property_id == content.starter_property_id {
            assert_eq!(
                scene.sky, "sky_test.xml",
                "the lead-owned starter keeps its placeholder sky"
            );
        } else {
            assert!(
                SHIPPED_SKIES.contains(&scene.sky.as_str()),
                "property {} names sky {} which the client's Resources/SKY does not ship",
                lot.property_id,
                scene.sky
            );
        }
    }
    assert_eq!(scenes.len(), 23);
}

#[test]
fn bundled_enabled_starter_enters_its_scene_before_the_full_reply_sequence() {
    let content = HousingContent::bundled().unwrap();
    let starter = content
        .lots
        .iter()
        .find(|lot| lot.property_id == content.starter_property_id)
        .unwrap();
    assert!(starter.enabled);
    let expected_scene = starter.scene.as_ref().unwrap().name.clone();
    assert_eq!(expected_scene, "hsg_hum_condo_w_lot");
    let mut service =
        HousingService::with_content(FixtureConfig::capture_defaults(), content).unwrap();
    let house_guid = service.grant_starter_lot(OWNER).unwrap();
    let mut stock = TestStock::default();
    let outcome = service
        .handle(OWNER, &enter_frame(house_guid), &mut stock)
        .unwrap();
    match outcome.zone_change.as_ref().unwrap() {
        HousingZoneChange::EnterHouse { scene, .. } => assert_eq!(scene.name, expected_scene),
        other => panic!("expected house scene transition, got {other:?}"),
    }
    assert_eq!(opcodes(&outcome), vec![45, 28, 44, 43]);
}

#[test]
fn bundled_disabled_lots_refuse_entry_with_a_reason() {
    let content = HousingContent::bundled().unwrap();
    let disabled_ids: Vec<_> = content
        .lots
        .iter()
        .filter(|lot| !lot.enabled)
        .map(|lot| lot.property_id)
        .collect();
    assert!(!disabled_ids.is_empty());
    for property_id in disabled_ids {
        let mut disabled_content = content.clone();
        disabled_content.starter_property_id = property_id;
        // The kit's positions are the Wilds Condo's; another starter type stands its own.
        disabled_content.starter_kit = None;
        let mut service =
            HousingService::with_content(FixtureConfig::capture_defaults(), disabled_content)
                .unwrap();
        let house_guid = service.grant_starter_lot(OWNER).unwrap();
        let mut stock = TestStock::default();
        assert_eq!(
            service
                .handle(OWNER, &enter_frame(house_guid), &mut stock)
                .unwrap_err(),
            ServiceError::HouseSceneUnavailable { property_id }
        );
    }
}

/// T516: a lot saved on a now-closed farm row still loads, and entering it is refused without
/// changing the lots or the bag.
#[test]
fn a_saved_disabled_farm_lot_loads_and_entry_changes_nothing() {
    let content = HousingContent::bundled().unwrap();
    let farm = content
        .lots
        .iter()
        .find(|lot| lot.property_id == 33)
        .expect("the Briarwood farm row");
    assert!(!farm.enabled && farm.scene.is_none() && farm.scene_source.is_none());

    // A save made before the row was closed: the character already owns the lot.
    let mut seeded = content.clone();
    seeded.starter_property_id = 33;
    seeded.starter_kit = None;
    let mut before_close =
        HousingService::with_content(FixtureConfig::capture_defaults(), seeded).unwrap();
    let house_guid = before_close.grant_starter_lot(OWNER).unwrap();
    let saved: HousingServiceState =
        serde_json::from_slice(&serde_json::to_vec(&before_close.state()).unwrap()).unwrap();

    let content_dir = Content::new(vec![item(INSTANCE as i32, BLOCK_ITEM, 1)], &[BLOCK_ITEM]);
    let mut inventory = content_dir.inventory();
    let mut service = HousingService::from_state_with_content(
        FixtureConfig::capture_defaults(),
        content,
        &saved,
        inventory.state(),
    )
    .unwrap();
    assert_eq!(service.lot(house_guid).unwrap().property_id, 33);
    let before_state = serde_json::to_vec(&service.state()).unwrap();
    let before_inventory = serde_json::to_vec(inventory.state()).unwrap();
    assert_eq!(
        service
            .handle(OWNER, &enter_frame(house_guid), &mut inventory)
            .unwrap_err(),
        ServiceError::HouseSceneUnavailable { property_id: 33 }
    );
    assert_eq!(serde_json::to_vec(&service.state()).unwrap(), before_state);
    assert_eq!(
        serde_json::to_vec(inventory.state()).unwrap(),
        before_inventory
    );
}

#[test]
fn starter_lot_grant_is_per_character_and_idempotent() {
    let mut service = HousingService::with_content(
        FixtureConfig::capture_defaults(),
        HousingContent::bundled().unwrap(),
    )
    .unwrap();
    let owner_lot = service.grant_starter_lot(OWNER).unwrap();
    assert_eq!(service.grant_starter_lot(OWNER).unwrap(), owner_lot);
    let guest_lot = service.grant_starter_lot(GUEST).unwrap();
    assert_ne!(guest_lot, owner_lot);
    let lots = service.state().lots;
    assert_eq!(lots.len(), 2);
    assert!(lots.iter().all(|lot| lot.property_id == 25));
}

/// houses.json `starter_kit` (L040): the Chicken for the bag and the block standing in the lot.
const KIT_CHICKEN: i32 = 70_429;
const KIT_BLOCK: i32 = 32_647;

#[test]
fn the_starter_lot_and_its_kit_commit_together() {
    let mut service = HousingService::with_content(
        FixtureConfig::capture_defaults(),
        HousingContent::bundled().unwrap(),
    )
    .unwrap();
    let grant = [BASE_OPCODE.to_le_bytes(), 23u16.to_le_bytes()].concat();
    let mut stock = TestStock {
        refuse_return: true,
        ..TestStock::default()
    };
    let before = service.state();
    assert!(matches!(
        service.handle(OWNER, &grant, &mut stock),
        Err(ServiceError::StockRejected { .. })
    ));
    assert_eq!(
        service.state(),
        before,
        "a refused kit drops the staged lot"
    );
    stock.refuse_return = false;
    assert!(service.handle(OWNER, &grant, &mut stock).unwrap().changed);
    assert_eq!(stock.instances, [owned(1, KIT_CHICKEN, 1)]);
    let standing = service.state().lots[0].fixtures.clone();
    let ids: Vec<_> = standing.iter().map(|f| f.item_instance_id).collect();
    assert_eq!(ids.len(), 4);
    assert!(
        ids.iter().all(|&id| id > 1),
        "no fixture shares the unit's id"
    );
    assert!(
        standing
            .iter()
            .all(|f| f.fixture_definition_id == KIT_BLOCK)
    );
    assert!(!service.handle(OWNER, &grant, &mut stock).unwrap().changed);
    assert_eq!(
        stock.instances.len(),
        1,
        "a repeat ask grants no second unit"
    );
    assert_eq!(service.state().lots[0].fixtures, standing);
}

#[test]
fn the_inventory_hands_out_each_kit_id_once() {
    let content = Content::new(
        vec![item(INSTANCE as i32, BLOCK_ITEM, 1)],
        &[BLOCK_ITEM, KIT_CHICKEN],
    );
    let mut inventory = content.inventory();
    let first = inventory.first_free_instance_id().unwrap();
    assert_eq!(first, INSTANCE + 1);
    let before = inventory.state().clone();
    for (what, ids) in [
        ("an id a row holds", vec![INSTANCE]),
        ("a repeated id", vec![first, first]),
        ("ids out of order", vec![first + 1, first]),
        ("no id", Vec::new()),
    ] {
        assert!(inventory.grant_kit(KIT_CHICKEN, &ids).is_err(), "{what}");
        assert_eq!(inventory.state(), &before, "{what}: the bag is unchanged");
    }
    inventory
        .grant_kit(KIT_CHICKEN, &[first, first + 1, first + 2])
        .unwrap();
    assert_eq!(units(&inventory, KIT_CHICKEN), 1);
    assert_eq!(inventory.first_free_instance_id().unwrap(), first + 3);
    let granted = inventory.state().clone();
    assert!(
        inventory.grant_kit(KIT_CHICKEN, &[first + 1]).is_err(),
        "an id already handed to a standing fixture"
    );
    assert_eq!(inventory.state(), &granted);
}

#[test]
fn a_starter_kit_that_cannot_stand_fails_at_load() {
    let bundled = HousingContent::bundled().unwrap();
    let mut config = FixtureConfig::capture_defaults();
    config
        .rows
        .retain(|row| row.item_definition_id != KIT_CHICKEN);
    assert_eq!(
        HousingService::with_content(config, bundled.clone()).err(),
        Some(ServiceError::FixtureDefinitionUnknown {
            definition_id: KIT_CHICKEN
        })
    );
    let refused = |edit: fn(&mut housing_service::StarterKit)| {
        let mut content = bundled.clone();
        edit(content.starter_kit.as_mut().unwrap());
        HousingContent::parse(&serde_json::to_vec(&content).unwrap()).is_err()
    };
    assert!(
        refused(|kit| kit.placed_positions[0][0] = 87.0),
        "outside the build area"
    );
    assert!(
        refused(|kit| kit.placed_positions = vec![[92.0, 0.0, -30.0]; 17]),
        "over 16"
    );
    assert!(refused(|kit| kit.item_id = 0), "a zero item id");
    assert!(
        !refused(|kit| kit.placed_positions.clear()),
        "a kit may stand nothing"
    );
}

#[test]
fn enter_sends_zoning_then_exact_45_28_44_and_item_list_bytes() {
    let mut service = service_with_lot();
    let mut stock = TestStock::default();
    let outcome = service
        .handle(OWNER, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    let zone_frame = match outcome.zone_change.as_ref().unwrap() {
        HousingZoneChange::EnterHouse {
            actor_guid,
            house_guid,
            scene,
            spawn_position,
            spawn_rotation,
        } => {
            assert_eq!(*actor_guid, OWNER);
            assert_eq!(*house_guid, HOUSE);
            assert_eq!(scene.name, "hsg_emptylot_test");
            let transition = ZoneTransition {
                envelope_flag: 1,
                scene: scene.name.clone(),
                type_id: i32::from(scene.zone_type),
                position: *spawn_position,
                rotation: *spawn_rotation,
                sky: scene.sky.clone(),
                tutorial: scene.tutorial,
                unknown: scene.unknown as u8,
                id: scene.zone_id as i32,
                update_radius: scene.update_radius,
                geometry_id: scene.geometry_id as i32,
                override_update_radius: scene.override_update_radius,
                wait_for_zone_ready: scene.wait_for_zone_ready,
            };
            let mut inner = 31u16.to_le_bytes().to_vec();
            append_text(&mut inner, "hsg_emptylot_test");
            inner.extend_from_slice(&2i32.to_le_bytes());
            for value in [10.0, 0.0, 20.0, 1.0, -1.0, 0.0, 0.0, 0.0] {
                append_float(&mut inner, value);
            }
            append_text(&mut inner, "sky_test.xml");
            inner.push(0);
            inner.push(1);
            inner.extend_from_slice(&1i32.to_le_bytes());
            append_float(&mut inner, 0.0);
            inner.extend_from_slice(&214i32.to_le_bytes());
            inner.extend_from_slice(&[1, 0]);
            let mut expected = 5u16.to_le_bytes().to_vec();
            expected.push(1);
            expected.extend_from_slice(&(inner.len() as u32).to_le_bytes());
            expected.extend_from_slice(&inner);
            assert_eq!(zone_transition::encode(&transition).unwrap(), expected);
            expected
        }
        other => panic!("expected entry transition, got {other:?}"),
    };
    assert!(
        !zone_frame.is_empty(),
        "opcode-31 zoning frame must precede replies"
    );
    assert_eq!(
        opcodes(&outcome),
        vec![REPLY_ZONE_DATA, REPLY_INSTANCE_DATA, REPLY_HOUSE_INFO, 43]
    );

    let mut expected_45 = vec![127, 0, 45, 0, 0, 0];
    expected_45.extend_from_slice(&10i32.to_le_bytes()); // head size in tenths (00ab07a0 default)
    expected_45.extend_from_slice(&OWNER.to_le_bytes());
    expected_45.extend_from_slice(&HOUSE.to_le_bytes());
    expected_45.extend_from_slice(&6_514i32.to_le_bytes());
    append_text(&mut expected_45, "");
    append_text(&mut expected_45, "");
    for value in [33_439i32, 0, 0, 0, 0] {
        expected_45.extend_from_slice(&value.to_le_bytes());
    }
    expected_45.extend_from_slice(&[0, 0, 1]);
    append_text(&mut expected_45, "");
    append_text(&mut expected_45, "");
    append_float(&mut expected_45, 0.0);
    expected_45.extend_from_slice(&0i32.to_le_bytes());
    expected_45.extend_from_slice(&[0, 0]);
    expected_45.extend_from_slice(&0i32.to_le_bytes());
    append_text(&mut expected_45, ""); // directory snapshot, after factory_plot_id
    expected_45.extend_from_slice(&[0u8; 8]);
    assert_eq!(outcome.replies[0].encode_inner().unwrap(), expected_45);

    let mut expected_28 = vec![127, 0, 28, 0];
    expected_28.extend_from_slice(&HOUSE.to_le_bytes());
    expected_28.extend_from_slice(&OWNER.to_le_bytes());
    append_text(&mut expected_28, "");
    expected_28.extend_from_slice(&0u64.to_le_bytes());
    expected_28.extend_from_slice(&6_514u32.to_le_bytes());
    append_text(&mut expected_28, "");
    for value in [0u32, 4, 2, 0, 0] {
        expected_28.extend_from_slice(&value.to_le_bytes());
    }
    expected_28.extend_from_slice(&[0, 1]);
    expected_28.extend_from_slice(&[0u8; 8]);
    expected_28.push(0);
    expected_28.extend_from_slice(&0u32.to_le_bytes());
    expected_28.extend_from_slice(&1i32.to_le_bytes());
    for value in [0.0, -20.0, 0.0, 1.0, 100.0, 20.0, 100.0, 1.0] {
        append_float(&mut expected_28, value);
    }
    expected_28.extend_from_slice(&33_439u32.to_le_bytes());
    expected_28.extend_from_slice(&[0, 0]);
    expected_28.extend_from_slice(&0u32.to_le_bytes());
    expected_28.extend_from_slice(&0u64.to_le_bytes());
    expected_28.push(0);
    append_text(&mut expected_28, "");
    append_text(&mut expected_28, "");
    expected_28.extend_from_slice(&[0, 0]);
    assert_eq!(outcome.replies[1].encode_inner().unwrap(), expected_28);

    let mut expected_44 = vec![127, 0, 44, 0, 0];
    expected_44.extend_from_slice(&10i32.to_le_bytes());
    expected_44.extend_from_slice(&[0, 1, 0]);
    expected_44.extend_from_slice(&[0u8; 12]);
    assert_eq!(outcome.replies[2].encode_inner().unwrap(), expected_44);
    let mut expected_43 = vec![127, 0, 43, 0];
    expected_43.extend_from_slice(&[0u8; 12]);
    assert_eq!(outcome.replies[3].encode_inner().unwrap(), expected_43);
}

/// T799 finding 4: the 127/28 payload byte-for-byte with one standing fixture. The client reads a
/// `u32` key then a bare `FixtureInstance` (`00ab8c50` -> `00ab6f40`) between the caps and the
/// permission count; a whole 127/40 record would shift the build box. The community-preserved
/// `p2.pcap` capture carries the same key + instance shape (reliable record seq 12619).
#[test]
fn instance_data_encodes_a_fixture_as_key_and_bare_instance_bytes() {
    let service = service_with_lot();
    let mut state = service.state();
    let fixture_guid = 0x00a0_0000_0000_0001;
    let fixture = SavedFixture {
        fixture_guid,
        house_guid: HOUSE,
        item_instance_id: INSTANCE,
        fixture_definition_id: BLOCK_FIXTURE,
        transform: FixtureTransform {
            position: [10.0, 0.0, 20.0],
            rotation: ZERO_ROTATION,
            tilt: [0.0, 0.0, 1.0, 0.0],
            scale: 1.0,
            tint_id: 0,
        },
        placed_by: OWNER,
    };
    state.lots[0].fixtures.push(fixture);
    state.lots[0].count = 1;
    let mut service = HousingService::from_state_with_content(
        FixtureConfig::capture_defaults(),
        test_housing_content(),
        &state,
        &no_items(),
    )
    .unwrap();
    let mut stock = TestStock::default();
    let outcome = service
        .handle(OWNER, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    assert_eq!(
        opcodes(&outcome)[..2],
        [REPLY_ZONE_DATA, REPLY_INSTANCE_DATA]
    );

    let mut expected = vec![127, 0, 28, 0];
    expected.extend_from_slice(&HOUSE.to_le_bytes());
    expected.extend_from_slice(&OWNER.to_le_bytes());
    append_text(&mut expected, "");
    expected.extend_from_slice(&0u64.to_le_bytes());
    expected.extend_from_slice(&6_514u32.to_le_bytes());
    append_text(&mut expected, "");
    for value in [0u32, 4, 2, 1] {
        expected.extend_from_slice(&value.to_le_bytes());
    }
    // The keyed list entry: key, then the bare FixtureInstance the client reads.
    expected.extend_from_slice(&1u32.to_le_bytes());
    expected.extend_from_slice(&fixture_guid.to_le_bytes());
    expected.extend_from_slice(&HOUSE.to_le_bytes());
    expected.extend_from_slice(&BLOCK_FIXTURE.to_le_bytes());
    expected.extend_from_slice(&0i32.to_le_bytes());
    for value in [10.0, 0.0, 20.0, 1.0] {
        append_float(&mut expected, value);
    }
    for value in [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0] {
        append_float(&mut expected, value);
    }
    expected.extend_from_slice(&housing_service::fixture_actor_guid(fixture_guid).to_le_bytes());
    expected.extend_from_slice(&0i32.to_le_bytes()); // tint
    expected.extend_from_slice(&0i32.to_le_bytes()); // unknown10
    expected.extend_from_slice(&0i32.to_le_bytes()); // customization kind
    append_text(&mut expected, "");
    append_text(&mut expected, "");
    expected.extend_from_slice(&0i32.to_le_bytes()); // customization tint
    append_text(&mut expected, "");
    append_text(&mut expected, "");
    append_text(&mut expected, "");
    expected.extend_from_slice(&0i32.to_le_bytes()); // unknown13
    append_text(&mut expected, "");
    append_float(&mut expected, 1.0);
    expected.push(0);
    expected.extend_from_slice(&0i32.to_le_bytes()); // unknown17
    // Permissions, flags and counts, then the build box the placement is tested against.
    expected.extend_from_slice(&0u32.to_le_bytes());
    expected.extend_from_slice(&[0, 1]);
    expected.extend_from_slice(&1u32.to_le_bytes());
    expected.extend_from_slice(&0u32.to_le_bytes());
    expected.push(0);
    expected.extend_from_slice(&0u32.to_le_bytes());
    expected.extend_from_slice(&1i32.to_le_bytes());
    for value in [0.0, -20.0, 0.0, 1.0, 100.0, 20.0, 100.0, 1.0] {
        append_float(&mut expected, value);
    }
    expected.extend_from_slice(&33_439u32.to_le_bytes());
    expected.extend_from_slice(&[0, 0]);
    expected.extend_from_slice(&0u32.to_le_bytes());
    expected.extend_from_slice(&0u64.to_le_bytes());
    expected.push(0);
    append_text(&mut expected, "");
    append_text(&mut expected, "");
    expected.extend_from_slice(&[0, 0]);
    assert_eq!(outcome.replies[1].encode_inner().unwrap(), expected);
}

#[test]
fn leave_clears_session_selection_and_blocks_further_edits() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    service.offer(OWNER, SELECTION, INSTANCE, &stock).unwrap();
    let outcome = service.handle(OWNER, &leave_frame(), &mut stock).unwrap();
    assert!(!outcome.changed);
    assert!(outcome.replies.is_empty());
    assert_eq!(
        outcome.zone_change,
        Some(HousingZoneChange::ReturnToPreviousZone {
            actor_guid: OWNER,
            house_guid: HOUSE,
        })
    );
    assert_eq!(
        service
            .handle(
                OWNER,
                &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock
            )
            .unwrap_err(),
        ServiceError::NotInHouse { actor_guid: OWNER }
    );
}

#[test]
fn leaving_removes_every_fixture_actor_the_visit_spawned() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    let entry = enter(&mut service, &mut stock);
    let placed = place(&mut service, &mut stock);
    // The removal set comes from the actors the visit actually spawned, not from re-deriving the
    // fixture guids: a lot that spawned nothing would otherwise pass this test too.
    let mut spawned = Vec::new();
    for outcome in [&entry, &placed] {
        for actor in &outcome.actors {
            match actor {
                housing_service::FixtureActor::Spawn(npc) => spawned.push(npc.actor_id),
                other => panic!("entering and placing only spawn actors: {other:?}"),
            }
        }
    }
    assert!(!spawned.is_empty(), "the placement spawned its actor");
    let outcome = service.handle(OWNER, &leave_frame(), &mut stock).unwrap();
    let removed: Vec<u64> = outcome
        .actors
        .iter()
        .map(|actor| match actor {
            housing_service::FixtureActor::Remove(guid) => *guid,
            other => panic!("leaving only removes actors: {other:?}"),
        })
        .collect();
    assert_eq!(
        removed, spawned,
        "one 35/3 per actor the visit spawned, none left in the world"
    );
}

#[test]
fn pickup_all_returns_every_owned_item_or_keeps_both_sides_unchanged() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![
        owned(INSTANCE, BLOCK_ITEM, 1),
        owned(INSTANCE + 1, BLOCK_ITEM, 1),
    ]);
    enter(&mut service, &mut stock);
    for (selection, instance) in [(SELECTION, INSTANCE), (SELECTION + 1, INSTANCE + 1)] {
        service.offer(OWNER, selection, instance, &stock).unwrap();
        service
            .handle(
                OWNER,
                &place_frame(selection, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock,
            )
            .unwrap();
    }
    assert_eq!(service.lot(HOUSE).unwrap().count, 2);
    assert_eq!(stock.instances.len(), 0);
    let outcome = service
        .handle(OWNER, &pickup_all_frame(), &mut stock)
        .unwrap();
    assert!(outcome.changed);
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(
        stock
            .instances
            .iter()
            .map(|item| item.definition_id)
            .collect::<Vec<_>>(),
        vec![BLOCK_ITEM, BLOCK_ITEM]
    );

    let returned_ids: Vec<u32> = stock
        .instances
        .iter()
        .map(|item| item.instance_id)
        .collect();
    for (index, instance) in returned_ids.into_iter().enumerate() {
        let selection = SELECTION + 2 + index as u32;
        service.offer(OWNER, selection, instance, &stock).unwrap();
        service
            .handle(
                OWNER,
                &place_frame(selection, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock,
            )
            .unwrap();
    }
    stock.refuse_return = true;
    let before = service.state();
    assert!(
        service
            .handle(OWNER, &pickup_all_frame(), &mut stock)
            .is_err()
    );
    assert_eq!(service.state(), before);
    assert_eq!(stock.instances.len(), 0);
}

#[test]
fn cap_and_unowned_selector_refusals_consume_no_stock() {
    let mut service = service_with_lot();
    let stock_items = (0..5)
        .map(|index| owned(INSTANCE + index, BLOCK_ITEM, 1))
        .collect();
    let mut stock = TestStock::new(stock_items);
    enter(&mut service, &mut stock);
    for index in 0..4 {
        let selection = SELECTION + index;
        let instance = INSTANCE + index;
        service.offer(OWNER, selection, instance, &stock).unwrap();
        service
            .handle(
                OWNER,
                &place_frame(selection, &block_transform(position(), ZERO_ROTATION, 1.0)),
                &mut stock,
            )
            .unwrap();
    }
    let selection = SELECTION + 4;
    let instance = INSTANCE + 4;
    service.offer(OWNER, selection, instance, &stock).unwrap();
    let denied = service
        .handle(
            OWNER,
            &place_frame(selection, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap_err();
    assert_eq!(
        denied,
        ServiceError::Housing(HousingError::FixtureLimitReached)
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 4);
    assert_eq!(stock.instances.len(), 1);

    let mut unowned_service = service_with_lot();
    let mut unowned_stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut unowned_service, &mut unowned_stock);
    let denied = unowned_service
        .handle(
            OWNER,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut unowned_stock,
        )
        .unwrap_err();
    assert_eq!(
        denied,
        ServiceError::UnresolvedSelection {
            selection_id: SELECTION
        }
    );
    assert_eq!(unowned_stock.instances.len(), 1);
}

#[test]
fn guest_can_visit_but_cannot_toggle_edit_or_place() {
    let mut service = service_with_lot();
    service
        .set_permission(OWNER, HOUSE, GUEST, PERMISSION_CAN_VISIT)
        .unwrap();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    let entered = service
        .handle(GUEST, &enter_frame(HOUSE), &mut stock)
        .unwrap();
    assert!(entered.zone_change.is_some());
    assert_eq!(
        service
            .handle(GUEST, &edit_frame(true), &mut stock)
            .unwrap_err(),
        ServiceError::Housing(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        service
            .offer(GUEST, SELECTION, INSTANCE, &stock)
            .unwrap_err(),
        ServiceError::Housing(HousingError::NotPropertyOwner)
    );
    let denied = service
        .handle(
            GUEST,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut stock,
        )
        .unwrap_err();
    assert_eq!(
        denied,
        ServiceError::UnresolvedSelection {
            selection_id: SELECTION
        }
    );
    assert_eq!(service.lot(HOUSE).unwrap().count, 0);
    assert_eq!(stock.instances.len(), 1);
}

#[test]
fn housing_service_reload_keeps_lots_and_inventory_conservation() {
    let content = test_housing_content();
    let mut service =
        HousingService::with_content(FixtureConfig::capture_defaults(), content.clone()).unwrap();
    service.grant_starter_lot(OWNER).unwrap();
    let state = service.state();
    let empty_inventory = InventoryState {
        coins: 0,
        items: Vec::new(),
        equipment: Vec::new(),
        next_instance_id: None,
    };
    let restored = HousingService::from_state_with_content(
        FixtureConfig::capture_defaults(),
        content,
        &state,
        &empty_inventory,
    )
    .unwrap();
    assert_eq!(restored.state(), state);
}

fn no_items() -> InventoryState {
    InventoryState {
        coins: 0,
        items: Vec::new(),
        equipment: Vec::new(),
        next_instance_id: None,
    }
}

/// L045 placed-by: each fixture records the character whose placement created it, a co-builder's
/// pick-up returns the unit to the picker (R), and the record survives a save and reload.
#[test]
fn placed_by_records_each_placer_and_survives_a_reload() {
    const GUEST_INSTANCE: u32 = INSTANCE + 1;
    let mut service = service_with_lot();
    service
        .set_permission(OWNER, HOUSE, GUEST, PERMISSION_CAN_EDIT)
        .unwrap();
    let mut owner_stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut owner_stock);
    place(&mut service, &mut owner_stock);
    let owner_fixture = placed_guid(&service);

    let mut guest_stock = TestStock::new(vec![owned(GUEST_INSTANCE, BLOCK_ITEM, 1)]);
    service
        .handle(GUEST, &enter_frame(HOUSE), &mut guest_stock)
        .unwrap();
    service
        .handle(GUEST, &edit_frame(true), &mut guest_stock)
        .unwrap();
    service
        .offer(GUEST, SELECTION, GUEST_INSTANCE, &guest_stock)
        .unwrap();
    service
        .handle(
            GUEST,
            &place_frame(SELECTION, &block_transform(position(), ZERO_ROTATION, 1.0)),
            &mut guest_stock,
        )
        .unwrap();
    let placers = |service: &HousingService| -> Vec<(u32, u64)> {
        let mut rows: Vec<_> = service
            .lot(HOUSE)
            .unwrap()
            .fixtures
            .iter()
            .map(|fixture| (fixture.item_instance_id, fixture.placed_by))
            .collect();
        rows.sort();
        rows
    };
    assert_eq!(
        placers(&service),
        vec![(INSTANCE, OWNER), (GUEST_INSTANCE, GUEST)],
        "the server records the acting character, not the lot owner"
    );

    // (R) Pick-up returns the unit to the picker: the co-builder's backpack, never the owner's.
    service
        .handle(GUEST, &pickup_frame(owner_fixture), &mut guest_stock)
        .unwrap();
    assert!(owner_stock.instances.is_empty());
    assert_eq!(guest_stock.instances.len(), 1);
    assert_eq!(guest_stock.instances[0].definition_id, BLOCK_ITEM);
    assert_eq!(placers(&service), vec![(GUEST_INSTANCE, GUEST)]);

    let state = service.state();
    let saved = serde_json::to_string(&state).unwrap();
    assert!(saved.contains(&format!("\"placed_by\":{GUEST}")), "{saved}");
    let decoded: HousingServiceState = serde_json::from_str(&saved).unwrap();
    let restored =
        HousingService::from_state(FixtureConfig::capture_defaults(), &decoded, &no_items())
            .unwrap();
    assert_eq!(restored.state(), state);
    assert_eq!(placers(&restored), vec![(GUEST_INSTANCE, GUEST)]);
}

/// L045 migration: a schema-1 document (no `placed_by`) loads, attributes every fixture to the
/// lot owner and is written back as the current schema. Unknown versions are refused whole.
#[test]
fn a_schema_one_document_migrates_placed_by_to_the_owner() {
    let mut service = service_with_lot();
    let mut stock = TestStock::new(vec![owned(INSTANCE, BLOCK_ITEM, 1)]);
    enter(&mut service, &mut stock);
    place(&mut service, &mut stock);
    let current = service.state();
    assert_eq!(current.schema_version, 2);

    let mut old = serde_json::to_value(&current).unwrap();
    old["schema_version"] = json!(1);
    for fixture in old["lots"][0]["fixtures"].as_array_mut().unwrap() {
        let fixture = fixture.as_object_mut().unwrap();
        assert!(fixture.remove("placed_by").is_some());
    }
    let old: HousingServiceState = serde_json::from_value(old).unwrap();
    let unrecorded = old.lots[0].fixtures[0].placed_by;
    assert_eq!(unrecorded, 0, "schema 1 has no placer");
    let migrated =
        HousingService::from_state(FixtureConfig::capture_defaults(), &old, &no_items()).unwrap();
    // Written back as schema 2, with the owner as the placer.
    assert_eq!(migrated.state(), current);
    assert_eq!(migrated.lot(HOUSE).unwrap().fixtures[0].placed_by, OWNER);

    for found in [0, SERVICE_SCHEMA_VERSION + 1] {
        let mut unknown = current.clone();
        unknown.schema_version = found;
        assert_eq!(
            HousingService::from_state(FixtureConfig::capture_defaults(), &unknown, &no_items())
                .unwrap_err(),
            StateError::SchemaVersion { found }
        );
    }
}
