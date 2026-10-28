//! T485/T488: the fixtures the pet care suites share (`tests/pet_care.rs`, `tests/pet_grooming.rs`):
//! the shipped kit rows, a bag holding them, a dog and the 53/* packet readers. T489/T531: edited
//! content and model rows. Includers declare `json`, `pet_inventory` and `pet_wire` first (`use
//! super::...` below).
#![allow(dead_code)] // each includer uses a subset

use super::{json, pet_inventory::InventoryFixture, pet_wire};
use osfr_tools::pets::{PetBusy, PetContent, PetService, PetTrickAnimation, PetWire};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};

/// Shipped catalogue rows (lab/reports/item-catalogue-20261015), all class 73: bowl 11 slot 204,
/// Simple Pet Food 1835 slot 205, Simple Pet Shampoo 1836 slot 200, Basic Pet Conditioner 10521
/// slot 201, brush 912 slot 203.
pub const BOWL: i32 = 11;
pub const FOOD: i32 = 1835;
pub const SHAMPOO: i32 = 1836;
pub const CONDITIONER: i32 = 10521;
pub const BRUSH: i32 = 912;
pub const KIT: [(i32, i32, i32); 5] = [
    (BOWL, 204, 73),
    (FOOD, 205, 73),
    (SHAMPOO, 200, 73),
    (CONDITIONER, 201, 73),
    (BRUSH, 203, 73),
];
pub const BOWL_ITEM: i32 = 21;
pub const FOOD_ITEM: i32 = 22;
pub const SHAMPOO_ITEM: i32 = 23;
pub const PLAIN_ITEM: i32 = 24;
pub const CONDITIONER_ITEM: i32 = 25;
pub const BRUSH_ITEM: i32 = 26;
pub const OWNER: u64 = 33;
/// The bundled Char species (1000015) is adopted from item 70352; `ADOPT_ITEM` is the fixture
/// instance the adoption consumes.
pub const ADOPT_DEF: i32 = 70_352;
pub const ADOPT_ITEM: i32 = 27;
/// 3,000 s after adoption: idle decay has lowered the needs, so a feed or a brushing visibly
/// raises them.
pub const T0: u64 = 3_000_000;

/// A bag with the whole kit: a bowl, two food, two shampoo, a conditioner, a brush and a plain item.
pub fn kit() -> InventoryFixture {
    InventoryFixture::with_kit_items(
        &[
            (BOWL_ITEM, BOWL, 1),
            (FOOD_ITEM, FOOD, 2),
            (SHAMPOO_ITEM, SHAMPOO, 2),
            (PLAIN_ITEM, 9001, 1),
            (CONDITIONER_ITEM, CONDITIONER, 1),
            (BRUSH_ITEM, BRUSH, 1),
        ],
        &KIT,
    )
}

/// A dog (model 364, which lists item class 73) adopted at time 0.
pub fn dog() -> (PetService, u32) {
    dog_with(PetContent::bundled().unwrap())
}

/// `dog` on edited content (T489: trick rules and animation rows).
pub fn dog_with(content: PetContent) -> (PetService, u32) {
    model_pet(content, 364)
}

/// A pet of client model `model_id` adopted at time 0 (T531: a cat, a bird, the dog without rows).
pub fn model_pet(content: PetContent, model_id: i32) -> (PetService, u32) {
    let mut service = PetService::new(content.clone()).unwrap();
    let mut definition = content.definition(1_000_015).unwrap();
    definition.model = content.model(model_id).unwrap().adr.clone();
    let id = service.adopt_vendor_grant(1, &definition, 0).unwrap();
    (service, id)
}

/// The shipped animation row of the dog (model 364) for `trick`. The trick and pet-trick
/// objective suites both time a trick from it, so a content edit that drops the model or the row
/// fails every suite that performs a trick.
pub fn trick_row(trick: u32) -> PetTrickAnimation {
    PetContent::bundled()
        .expect("pets.json loads")
        .model(364)
        .expect("the dog model row")
        .trick_animations
        .iter()
        .find(|row| row.trick_id == trick)
        .unwrap_or_else(|| panic!("the dog has an animation row for trick {trick}"))
        .clone()
}

/// The shipped `pets.json` document.
pub fn pets_json() -> Value {
    json::pets_json()
}

/// The shipped `data/content` root.
pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// A private-lab path (`lab/<path>`): the pinned client tables the pet gates check.
pub fn lab(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab")
        .join(path)
}

/// The client locale id -> text table (`lab/reports/locale-id-to-text.json`).
pub fn locale() -> BTreeMap<i64, String> {
    json::read(&lab("reports/locale-id-to-text.json"))
        .as_array()
        .expect("locale rows are an array")
        .iter()
        .filter_map(|row| Some((row["id"].as_i64()?, row["text"].as_str()?.to_owned())))
        .collect()
}

/// The shipped content after `edit`, or why it refuses to load.
pub fn loads(edit: impl FnOnce(&mut Value)) -> io::Result<PetContent> {
    let mut document = pets_json();
    edit(&mut document);
    PetContent::from_json(&serde_json::to_vec(&document).unwrap())
}

/// The shipped content after `edit` (it must load).
pub fn content_with(edit: impl FnOnce(&mut Value)) -> PetContent {
    loads(edit).unwrap()
}

/// The `models` row of client model `model_id` in a `pets.json` document.
pub fn model_row(document: &mut Value, model_id: i64) -> &mut Value {
    document["models"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|model| model["model_id"] == model_id)
        .unwrap()
}

pub fn send(
    service: &mut PetService,
    fixture: &mut InventoryFixture,
    packet: &[u8],
    now_ms: u64,
) -> PetWire {
    service
        .handle_at_ms(
            packet,
            OWNER,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            |_, _| None,
            now_ms,
        )
        .unwrap()
        .unwrap()
}

/// An expected server packet (or a malformed request): a header, then u32 words.
pub fn words(head: &[u8], values: &[u32]) -> Vec<u8> {
    let mut bytes = head.to_vec();
    for value in values {
        bytes.extend(value.to_le_bytes());
    }
    bytes
}

/// A 53/20 notify for `pet`: the kind, the timer word, three zero words.
pub fn notify(pet: u32, kind: u32, timer: u32) -> Vec<u8> {
    words(&[53, 0, 20], &[pet, kind, timer, 0, 0, 0])
}

pub fn fit(pet: u32, slot: u32, item: i32) -> Vec<u8> {
    pet_wire::utility(false, pet, slot, item as u32)
}

pub fn u32_at(packet: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

/// The kinds of every 53/20 PetUtilityNotify (pet, kind, four words).
pub fn notify_kinds(packets: &[Vec<u8>]) -> Vec<u32> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[53, 0, 20]))
        .map(|packet| {
            assert_eq!(packet.len(), 3 + 24, "six u32");
            u32_at(packet, 7)
        })
        .collect()
}

pub fn statuses(packets: &[Vec<u8>]) -> Vec<&Vec<u8>> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[53, 0, 3]))
        .collect()
}

/// T828: the 35/8 animation slots a packet list plays on `actor` (u16 35, u16 8, u64 guid, i32
/// slot, i32 0, u8 flag; native reader `00908a50`).
pub fn animations(packets: &[Vec<u8>], actor: u64) -> Vec<i32> {
    packets
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 8, 0]))
        .map(|packet| {
            assert_eq!(packet.len(), 21, "one animation frame");
            packet
        })
        .filter(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()) == actor)
        .map(|packet| u32_at(packet, 12) as i32)
        .collect()
}

pub fn count(fixture: &InventoryFixture, item: i32) -> u32 {
    fixture
        .inventory
        .state()
        .items
        .iter()
        .find(|owned| owned.id == item)
        .map_or(0, |owned| owned.count)
}
