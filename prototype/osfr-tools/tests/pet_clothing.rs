//! T530: pet clothing (53/11) — the equip panel's fit/clear with the client's item-class and
//! slot rules, the saved slots, and the attachment every spawn and every change carries.
//!
//! Client sources: the senders `00b60220` (fit) and `00b602c0` (clear) through the Lua callbacks
//! `sub_C07990`/`sub_C07AE0`, the wrapper `00b5c7e0`, and the reader `00b55c30` (flag, then the
//! pane slot, then the item). The pane slots come from `ScriptsBase.lua` `PetInventorySlotMap`
//! (head 1, chest 3, feet 5, collar 6, toy 7) and the shipped catalogue's pet rows use the same
//! item `Slot` values. The live actor change reuses the one 35/6 encoder (reader `0090a1f0`).
//!
//! Most tests drive `PetService::handle_at_ms`, the handler the gateway's pet route calls, with
//! real request bytes. `the_gateway_fits_real_catalogue_clothing` drives the whole
//! gateway -> gameplay -> service path with the shipped catalogue and a saved character.

#[path = "common/gateway_vendor.rs"]
mod gateway_vendor;
#[path = "support/json.rs"]
mod json;
#[path = "support/pet_care.rs"]
mod pet_care;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "support/pet_wire.rs"]
mod pet_wire;
#[path = "common/vendor_fixture.rs"]
#[allow(dead_code)]
mod vendor_fixture;

use gateway_vendor::{NOW, connect_with, fixture_root, handled, login, reload_saved, roster, wrap};
use osfr_tools::pets::{
    DECODED_SUBTYPES, PetBusy, PetEquipmentSlot, PetRequest, PetService, PetWire,
};
use osfr_tools::startup_world::{self, NpcAttachment};
use pet_care::{FOOD, FOOD_ITEM, KIT, OWNER, content_with, dog, model_pet, model_row, send};
use pet_inventory::{InventoryFixture, Wearable};
use serde_json::json;
use std::collections::BTreeSet;
use vendor_fixture::{VendorFixture, bag_item};

/// The client equipment pane slots (`PetInventorySlotMap`), 1-based.
const HEAD: u32 = 1;
const CHEST: u32 = 3;
const FEET: u32 = 5;
const COLLAR: u32 = 6;
const TOY: u32 = 7;

/// Synthetic definitions: the dog class 31 wearables, a cat class 61 chest piece (wrong class for
/// the dog), a head piece worn in the chest slot (wrong slot), and a type-4 kit row (wrong type).
const HEAD_DEF: i32 = 9_101;
const CHEST_DEF: i32 = 9_102;
const CAT_CHEST_DEF: i32 = 9_103;
const HEAD_AS_CHEST_DEF: i32 = 9_104;
const KIT_TYPE_DEF: i32 = 9_105;
const TOY_DEF: i32 = 9_106;
const COLLAR_DEF: i32 = 9_107;
const FEET_DEF: i32 = 9_108;
/// A chest piece whose catalogue row draws nothing (an empty model).
const DRAWSLESS_DEF: i32 = 9_109;

const HEAD_ITEM: i32 = 41;
const CHEST_ITEM: i32 = 42;
const CAT_ITEM: i32 = 43;
const HEAD_AS_CHEST_ITEM: i32 = 44;
const KIT_TYPE_ITEM: i32 = 45;
const TOY_ITEM: i32 = 46;
const COLLAR_ITEM: i32 = 47;
const FEET_ITEM: i32 = 48;
const DRAWSLESS_ITEM: i32 = 49;
/// An instance no fixture bag holds.
const UNOWNED_ITEM: i32 = 999;

/// The shipped look of every test garment (the fixture's undyed instances use the definition
/// default tint).
const CHEST_MODEL: &str = "pet_dog_small_chest_tshirt.adr";
const GARMENT_TEXTURE: &str = "casual-treelogo-P";
const GARMENT_TINT: i32 = 263;

fn wearables() -> Vec<Wearable> {
    vec![
        Wearable::new(HEAD_DEF, HEAD as i32, 31, "pet_dog_small_head_turban.adr"),
        Wearable::new(CHEST_DEF, CHEST as i32, 31, CHEST_MODEL),
        Wearable::new(
            CAT_CHEST_DEF,
            CHEST as i32,
            61,
            "pet_cat_longhair_chest_tshirt.adr",
        ),
        Wearable::new(
            HEAD_AS_CHEST_DEF,
            HEAD as i32,
            31,
            "pet_dog_small_head_cap.adr",
        ),
        Wearable::new(TOY_DEF, TOY as i32, 31, "tool_ar_ag_weapon_eggbeater.adr"),
        Wearable::new(
            COLLAR_DEF,
            COLLAR as i32,
            31,
            "pet_dog_small_shoulders_scarf.adr",
        ),
        Wearable::new(FEET_DEF, FEET as i32, 31, "pet_dog_small_feet_boots.adr"),
        Wearable::new(DRAWSLESS_DEF, CHEST as i32, 31, ""),
    ]
}

/// A bag holding every test garment, plus a kit row of the chest slot/class but type 4.
fn dressed() -> InventoryFixture {
    let mut kit = KIT.to_vec();
    kit.push((KIT_TYPE_DEF, CHEST as i32, 31));
    InventoryFixture::with_pet_items(
        &[
            (HEAD_ITEM, HEAD_DEF, 1),
            (CHEST_ITEM, CHEST_DEF, 1),
            (CAT_ITEM, CAT_CHEST_DEF, 1),
            (HEAD_AS_CHEST_ITEM, HEAD_AS_CHEST_DEF, 1),
            (KIT_TYPE_ITEM, KIT_TYPE_DEF, 1),
            (TOY_ITEM, TOY_DEF, 1),
            (COLLAR_ITEM, COLLAR_DEF, 1),
            (FEET_ITEM, FEET_DEF, 1),
            (DRAWSLESS_ITEM, DRAWSLESS_DEF, 1),
            (FOOD_ITEM, FOOD, 2),
        ],
        &kit,
        &wearables(),
    )
}

/// The shipped dog (model 364: class 73 for the kit and class 31 for clothing), summoned through
/// the real 53/4 request.
fn summoned() -> (PetService, InventoryFixture, u32) {
    let (mut service, pet_id) = dog();
    let mut fixture = dressed();
    let wire = send(&mut service, &mut fixture, &pet_wire::summon(pet_id), 0);
    assert!(wire.changed, "the dog summons");
    (service, fixture, pet_id)
}

fn expected_row(model: &str, texture: &str, tint: i32, slot: u32) -> NpcAttachment {
    NpcAttachment {
        model_name: model.to_owned(),
        texture_alias: texture.to_owned(),
        tint_alias: "dyetint".to_owned(),
        tint_id: tint,
        composite_effect_id: 0,
        slot: slot as i32,
    }
}

fn chest_row() -> NpcAttachment {
    expected_row(CHEST_MODEL, GARMENT_TEXTURE, GARMENT_TINT, CHEST)
}

fn head_row() -> NpcAttachment {
    expected_row(
        "pet_dog_small_head_turban.adr",
        GARMENT_TEXTURE,
        GARMENT_TINT,
        HEAD,
    )
}

/// The 35/6 `PlayerUpdatePacketEquipItemChange` the client's reader `0090a1f0` consumes: actor,
/// item id, the three attachment strings, tint, effect, slot, profile, an ignored word and the
/// wield type. `None` builds the clear row (empty strings, item 0).
fn equip_change(
    actor: u64,
    item: i32,
    row: Option<&NpcAttachment>,
    slot: u32,
    profile: i32,
) -> Vec<u8> {
    let mut bytes = vec![35, 0, 6, 0];
    bytes.extend(actor.to_le_bytes());
    bytes.extend(item.to_le_bytes());
    let empty = NpcAttachment {
        slot: slot as i32,
        ..NpcAttachment::default()
    };
    let row = row.unwrap_or(&empty);
    for text in [&row.model_name, &row.texture_alias, &row.tint_alias] {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
    }
    for value in [
        row.tint_id,
        row.composite_effect_id,
        row.slot,
        profile,
        0,
        0,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// The 53/11 echo the client's writer sends: clear flag, pane slot, item.
fn equip_echo(clear: bool, slot: u32, item: i32) -> Vec<u8> {
    let mut bytes = vec![53, 0, 11, u8::from(clear)];
    bytes.extend(slot.to_le_bytes());
    bytes.extend(item.to_le_bytes());
    bytes
}

/// The pet actor's AddNpc profile: the 35/6 change carries it so the client's `SetProfile`
/// (`sub_96A3E0`, from the same case) changes nothing on the actor.
fn actor_profile(service: &PetService) -> i32 {
    let spawn = service
        .view(OWNER)
        .expect("the pet is out")
        .spawn_packet()
        .expect("the pet actor spawns");
    startup_world::parse_add_npc(&spawn)
        .expect("the AddNpc parses")
        .active_profile
}

fn wear(
    service: &mut PetService,
    fixture: &mut InventoryFixture,
    clear: bool,
    slot: u32,
    item: i32,
    now_ms: u64,
) -> PetWire {
    send(
        service,
        fixture,
        &pet_wire::wear(clear, slot, item as u32),
        now_ms,
    )
}

fn spawn_of(wire: &PetWire) -> Vec<u8> {
    wire.private
        .iter()
        .chain(wire.shared.iter())
        .find(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .expect("the summon carries the pet's AddNpc")
        .clone()
}

#[test]
fn pet_equip_requests_decode_with_an_exact_length() {
    assert_eq!(
        DECODED_SUBTYPES,
        [4, 6, 7, 8, 11, 14, 16, 18, 19, 22, 24, 26, 28, 32],
        "53/11 is a decoded pet request"
    );
    assert_eq!(
        PetRequest::decode(&pet_wire::wear(false, CHEST, CHEST_ITEM as u32)).unwrap(),
        Some(PetRequest::Equip {
            clear: false,
            slot: CHEST,
            item: CHEST_ITEM as u32,
        })
    );
    assert_eq!(
        PetRequest::decode(&pet_wire::wear(true, TOY, 0)).unwrap(),
        Some(PetRequest::Equip {
            clear: true,
            slot: TOY,
            item: 0,
        })
    );
    let mut short = pet_wire::wear(false, CHEST, CHEST_ITEM as u32);
    short.pop();
    assert!(PetRequest::decode(&short).is_err(), "8-byte body");
    let mut long = pet_wire::wear(false, CHEST, CHEST_ITEM as u32);
    long.push(0);
    assert!(PetRequest::decode(&long).is_err(), "10-byte body");
    let mut action = pet_wire::wear(false, CHEST, CHEST_ITEM as u32);
    action[3] = 2;
    assert!(PetRequest::decode(&action).is_err(), "unknown flag");
}

#[test]
fn a_wrong_class_slot_or_type_item_is_refused_and_changes_nothing() {
    let (mut service, mut fixture, pet_id) = summoned();
    let roster_before = service.roster_snapshot();
    let inventory_before = fixture.inventory.state().clone();
    // The wrong class (a cat piece on a dog), the wrong slot (a head piece in the chest row),
    // the wrong item type (a type-4 kit row), an instance nobody owns, and a head piece in the
    // chest row: each is refused and echoes the chest row as it stands (empty).
    for item in [
        CAT_ITEM,
        HEAD_AS_CHEST_ITEM,
        KIT_TYPE_ITEM,
        DRAWSLESS_ITEM,
        UNOWNED_ITEM,
        HEAD_ITEM,
    ] {
        let refused = wear(&mut service, &mut fixture, false, CHEST, item, 1_000);
        assert!(!refused.changed, "item {item} is refused");
        assert!(refused.shared.is_empty(), "item {item} reaches no viewer");
        assert_eq!(
            refused.private,
            vec![equip_echo(false, CHEST, 0)],
            "item {item} echoes the unchanged chest row"
        );
        assert_eq!(service.roster_snapshot(), roster_before);
        assert_eq!(fixture.inventory.state(), &inventory_before);
    }
    assert_eq!(service.pet(pet_id).unwrap().equipment.chest, None);
    // A pane row the client does not have (2 and 4 are empty, 200 and up are the utility kit).
    for slot in [0, 2, 4, 8, 200] {
        let refused = wear(
            &mut service,
            &mut fixture,
            false,
            slot as u32,
            CHEST_ITEM,
            1_000,
        );
        assert!(!refused.changed, "slot {slot} is refused");
        assert!(
            refused.private.is_empty() && refused.shared.is_empty(),
            "slot {slot} has no pane row to echo"
        );
    }
    assert_eq!(service.roster_snapshot(), roster_before);
}

#[test]
fn a_vanity_pet_refuses_clothing() {
    // Give the vanity turtle (model 6) the dog clothing class it otherwise lacks, so only the
    // vanity rule can refuse the fit.
    let content = content_with(|document| {
        model_row(document, 6)["item_classes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key": 31, "id": 31, "unknown": 5}));
    });
    let (mut service, pet_id) = model_pet(content, 6);
    let mut fixture = dressed();
    assert!(send(&mut service, &mut fixture, &pet_wire::summon(pet_id), 0).changed);
    let before = service.roster_snapshot();
    let refused = wear(&mut service, &mut fixture, false, CHEST, CHEST_ITEM, 1_000);
    assert!(!refused.changed, "a vanity pet wears no clothes");
    assert!(refused.shared.is_empty());
    assert_eq!(service.roster_snapshot(), before);
}

#[test]
fn an_equipped_item_is_saved_and_dresses_every_spawn() {
    let (mut service, mut fixture, pet_id) = summoned();
    let actor = PetService::pet_guid(OWNER, pet_id);
    let profile = actor_profile(&service);
    let row = chest_row();

    let equipped = wear(&mut service, &mut fixture, false, CHEST, CHEST_ITEM, 1_000);
    assert!(equipped.changed);
    assert_eq!(
        service.pet(pet_id).unwrap().equipment.chest,
        Some(CHEST_ITEM)
    );
    assert!(
        equipped
            .private
            .contains(&equip_echo(false, CHEST, CHEST_ITEM))
    );
    assert!(
        equipped
            .private
            .iter()
            .any(|packet| packet.starts_with(&[53, 0, 5])),
        "the roster follows the fit"
    );
    let change = equip_change(actor, CHEST_ITEM, Some(&row), CHEST, profile);
    assert_eq!(
        equipped
            .private
            .iter()
            .filter(|packet| packet.starts_with(&[35, 0, 6, 0]))
            .count(),
        1,
        "one actor change for the owner"
    );
    assert!(equipped.private.contains(&change));
    assert_eq!(
        equipped.shared,
        vec![change],
        "and the same one for viewers"
    );

    // A save and reload keep the fitted instance.
    let saved = service.document_value();
    let mut reloaded =
        PetService::from_document(PetService::bundled_content().unwrap(), saved).unwrap();
    assert_eq!(
        reloaded.pet(pet_id).unwrap().equipment.chest,
        Some(CHEST_ITEM)
    );

    // The next summon (the zoning resummon path) carries the row in the AddNpc.
    let wire = send(
        &mut reloaded,
        &mut fixture,
        &pet_wire::summon(pet_id),
        2_000,
    );
    let npc = startup_world::parse_add_npc(&spawn_of(&wire)).expect("the AddNpc parses");
    assert_eq!(npc.attachments, vec![row.clone()]);
    // A late viewer's presence gets the same rows from the live inventory (`pet_view`).
    let mut late = reloaded.view(OWNER).unwrap();
    late.attachments = reloaded.worn_attachments(pet_id, &fixture.inventory);
    let npc = startup_world::parse_add_npc(&late.spawn_packet().unwrap()).unwrap();
    assert_eq!(npc.attachments, vec![row.clone()]);

    // The zoning resummon path carries them too (`Gameplay::resummon_pet` resolves the same
    // rows for `PetService::resummon`).
    reloaded.zone_away(3_000).unwrap();
    let wire = reloaded
        .resummon(
            actor,
            OWNER,
            [0.0; 3],
            PetBusy::default(),
            4_000,
            reloaded.worn_attachments(pet_id, &fixture.inventory),
        )
        .unwrap()
        .expect("the remembered pet comes back");
    let npc = startup_world::parse_add_npc(&spawn_of(&wire)).expect("the resummon AddNpc parses");
    assert_eq!(npc.attachments, vec![row]);
}

#[test]
fn a_viewer_receives_the_fit_and_the_clear() {
    let (mut service, mut fixture, pet_id) = summoned();
    let actor = PetService::pet_guid(OWNER, pet_id);
    let profile = actor_profile(&service);
    let row = expected_row(
        "pet_dog_small_shoulders_scarf.adr",
        GARMENT_TEXTURE,
        GARMENT_TINT,
        COLLAR,
    );
    let fitted = wear(
        &mut service,
        &mut fixture,
        false,
        COLLAR,
        COLLAR_ITEM,
        1_000,
    );
    assert_eq!(
        fitted.shared,
        vec![equip_change(
            actor,
            COLLAR_ITEM,
            Some(&row),
            COLLAR,
            profile
        )],
        "viewers are sent the fit"
    );
    let cleared = wear(&mut service, &mut fixture, true, COLLAR, COLLAR_ITEM, 2_000);
    assert_eq!(
        cleared.shared,
        vec![equip_change(actor, 0, None, COLLAR, profile)],
        "viewers are sent the clear"
    );
    assert_eq!(service.pet(pet_id).unwrap().equipment.collar, None);
}

#[test]
fn clearing_a_slot_removes_the_attachment_and_repeats_without_change() {
    let (mut service, mut fixture, pet_id) = summoned();
    let actor = PetService::pet_guid(OWNER, pet_id);
    let profile = actor_profile(&service);
    assert!(wear(&mut service, &mut fixture, false, HEAD, HEAD_ITEM, 1_000).changed);
    assert!(wear(&mut service, &mut fixture, false, CHEST, CHEST_ITEM, 1_100).changed);

    let cleared = wear(&mut service, &mut fixture, true, CHEST, CHEST_ITEM, 2_000);
    assert!(cleared.changed);
    assert_eq!(
        cleared.shared,
        vec![equip_change(actor, 0, None, CHEST, profile)]
    );
    assert!(cleared.private.contains(&equip_echo(true, CHEST, 0)));
    let equipment = service.pet(pet_id).unwrap().equipment.clone();
    assert_eq!(equipment.chest, None);
    assert_eq!(equipment.head, Some(HEAD_ITEM), "the other row stays");

    // The next spawn carries only the head attachment.
    let wire = send(&mut service, &mut fixture, &pet_wire::summon(pet_id), 3_000);
    let npc = startup_world::parse_add_npc(&spawn_of(&wire)).unwrap();
    assert_eq!(npc.attachments, vec![head_row()]);

    // Clearing the empty row again changes nothing and reaches no viewer.
    let again = wear(&mut service, &mut fixture, true, CHEST, CHEST_ITEM, 4_000);
    assert!(!again.changed);
    assert!(again.shared.is_empty());
    assert_eq!(again.private, vec![equip_echo(true, CHEST, 0)]);
}

#[test]
fn attachments_are_bounded_at_the_five_wearable_rows() {
    let (mut service, mut fixture, pet_id) = summoned();
    for (slot, item) in [
        (HEAD, HEAD_ITEM),
        (CHEST, CHEST_ITEM),
        (FEET, FEET_ITEM),
        (COLLAR, COLLAR_ITEM),
        (TOY, TOY_ITEM),
    ] {
        let fitted = wear(
            &mut service,
            &mut fixture,
            false,
            slot,
            item,
            1_000 + slot as u64,
        );
        assert!(fitted.changed, "slot {slot} fits");
    }
    let equipment = service.pet(pet_id).unwrap().equipment.clone();
    assert_eq!(equipment.instances().count(), 5);
    assert_eq!(
        equipment.utilities, [None; 8],
        "clothes stay out of the kit"
    );
    let wire = send(&mut service, &mut fixture, &pet_wire::summon(pet_id), 2_000);
    let npc = startup_world::parse_add_npc(&spawn_of(&wire)).unwrap();
    let slots: Vec<i32> = npc.attachments.iter().map(|row| row.slot).collect();
    assert_eq!(
        slots,
        vec![
            HEAD as i32,
            CHEST as i32,
            FEET as i32,
            COLLAR as i32,
            TOY as i32
        ],
        "the pane-slot order: head, chest, feet, collar, toy"
    );
    assert_eq!(npc.attachments.len(), 5);
}

#[test]
fn an_item_consumed_by_care_is_cleared_from_the_pet() {
    let (mut service, mut fixture, pet_id) = summoned();
    // A stack that a stale save could have fitted in both a utility row and a wearable row: the
    // low-level write is the one `wear` uses, and only the 53/11 rules now stop a non-clothing
    // item from taking a clothing slot.
    service
        .equip_item(
            pet_id,
            FOOD_ITEM,
            PetEquipmentSlot::Utility(6),
            &fixture.inventory,
        )
        .unwrap();
    service
        .equip_item(
            pet_id,
            FOOD_ITEM,
            PetEquipmentSlot::Head,
            &fixture.inventory,
        )
        .unwrap();
    for now in [1_000, 2_000] {
        service
            .care(
                pet_id,
                FOOD_ITEM,
                &mut fixture.inventory,
                PetBusy::default(),
                now,
            )
            .expect("the fitted food is eaten");
    }
    assert!(
        !fixture
            .inventory
            .state()
            .items
            .iter()
            .any(|item| item.id == FOOD_ITEM),
        "both units were eaten, so the stack is gone"
    );
    let equipment = service.pet(pet_id).unwrap().equipment.clone();
    assert_eq!(
        equipment.head, None,
        "the spent instance leaves the head row"
    );
    assert_eq!(equipment.utilities[6], None, "and the utility row");
}

#[test]
fn only_owned_worn_items_enter_the_trade_gate() {
    let (mut service, mut fixture, pet_id) = summoned();
    assert!(wear(&mut service, &mut fixture, false, CHEST, CHEST_ITEM, 1_000).changed);
    assert_eq!(
        service.equipped_instances(fixture.inventory.state()),
        BTreeSet::from([CHEST_ITEM])
    );
    // A stale id no bag holds must not enter the set: `TradeInventory` refuses a set naming an
    // item its inventory lacks, which would block trading for the whole character. Edit the save
    // document, which is the only way a pre-T530 roster can name such an instance.
    let mut document = service.document_value().unwrap();
    document["pets"][0]["equipment"]["head"] = json!(UNOWNED_ITEM);
    let stale =
        PetService::from_document(PetService::bundled_content().unwrap(), Some(document)).unwrap();
    assert_eq!(
        stale.pet(pet_id).unwrap().equipment.head,
        Some(UNOWNED_ITEM)
    );
    assert_eq!(
        stale.equipped_instances(fixture.inventory.state()),
        BTreeSet::from([CHEST_ITEM])
    );
    // A real, owned instance does enter it, so the trade gate blocks the item itself.
    let mut document = service.document_value().unwrap();
    document["pets"][0]["equipment"]["head"] = json!(HEAD_ITEM);
    let dressed =
        PetService::from_document(PetService::bundled_content().unwrap(), Some(document)).unwrap();
    assert_eq!(
        dressed.equipped_instances(fixture.inventory.state()),
        BTreeSet::from([HEAD_ITEM, CHEST_ITEM])
    );
}

#[test]
fn the_gateway_fits_real_catalogue_clothing() {
    // Shipped rows from the private item catalogue: the Black Lab Whistle (model 515, class 31)
    // and a dog head turban (type 3, slot 1, class 31).
    const ADOPTION: i32 = 2_644;
    const TURBAN: i32 = 2_163;
    const CAT_SHIRT: i32 = 1_193;
    let fixture = VendorFixture::new(&fixture_root(), "t530-clothing");
    let mut state = fixture.state(0, false);
    state.inventory.items.push(bag_item(ADOPTION, 0, 700, 1));
    state.inventory.items.push(bag_item(TURBAN, 0, 701, 1));
    state.inventory.items.push(bag_item(CAT_SHIRT, 0, 702, 1));
    state.inventory.next_instance_id = Some(703);
    let mut gateway = connect_with(
        fixture.reference(),
        fixture.character(),
        fixture.content(),
        state,
    );
    let (adopted, _) = handled(&mut gateway, &pet_wire::summon_by_item(ADOPTION as u32, 0));
    assert!(
        adopted
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 2, 0])),
        "the gateway adopts and summons the lab"
    );

    let (fitted, outcome) = handled(&mut gateway, &pet_wire::wear(false, HEAD, 701u32));
    assert!(outcome.gameplay_changed);
    assert!(
        fitted.iter().any(|packet| packet.starts_with(&[53, 0, 11])
            && packet.get(4..8) == Some(HEAD.to_le_bytes().as_slice())),
        "the equip panel echo names the head row"
    );
    assert!(fitted.iter().any(|packet| packet.starts_with(&[53, 0, 5])));
    let change = fitted
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 6, 0]))
        .expect("the owner's 35/6");
    assert_eq!(&change[12..16], &701i32.to_le_bytes());
    assert!(
        change
            .windows("pet_dog_<descriptor>_head_turban.adr".len())
            .any(|window| window == b"pet_dog_<descriptor>_head_turban.adr"),
        "the attachment names the catalogue model"
    );
    assert!(
        outcome.nearby.iter().any(|packet| packet == change),
        "viewers fan out the same change"
    );
    // The production trade gate takes the pet's fitted instances (`Gameplay::trade_inventory`),
    // so the worn turban cannot be offered in a player trade.
    let trade = gateway
        .trade_inventory()
        .expect("the character's inventory validates for trade");
    assert!(
        trade.equipped_on_pet.contains(&701),
        "the pet-worn instance is blocked from trade"
    );
    let saved = roster(&gateway);
    assert_eq!(saved[0]["equipment"]["head"], 701);

    // A wrong-class request through the gateway changes nothing at all.
    let before = gateway.snapshot();
    let (refused, outcome) = handled(&mut gateway, &pet_wire::wear(false, HEAD, 702u32));
    assert!(!outcome.gameplay_changed);
    assert!(outcome.nearby.is_empty());
    assert_eq!(&refused[0][..3], &[53u8, 0, 11]);
    assert_eq!(gateway.snapshot(), before);

    // A reconnect keeps the roster, and the next summon dresses the actor. `reload_saved`
    // restores the saved halves only, so the handshake `connect_with` runs comes next.
    let mut restored = reload_saved(fixture.reference(), &gateway, fixture.content());
    let guid = restored.state().guid;
    for packet in [login(guid), wrap(&[13, 0]), wrap(&[10, 0])] {
        restored
            .handle_at(&packet, NOW)
            .expect("the reconnected handshake handles");
    }
    assert_eq!(roster(&restored)[0]["equipment"]["head"], 701);
    let (summoned, _) = handled(&mut restored, &pet_wire::summon(1));
    let spawn = summoned
        .iter()
        .find(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .expect("the reconnected pet spawns");
    let npc = startup_world::parse_add_npc(spawn).expect("the AddNpc parses");
    assert_eq!(npc.attachments.len(), 1);
    assert_eq!(
        npc.attachments[0].model_name,
        "pet_dog_<descriptor>_head_turban.adr"
    );
}
