//! T454: the imported pet catalogue (`pets.json` models, items and tricks from
//! `scripts/pets_import.py`) checked against the client's own tables, and the real 53/32
//! adoption path for pet items the wiki species rows never listed.
#[path = "support/json.rs"]
mod json;
#[path = "support/models.rs"]
mod models;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "common/round70_content.rs"]
mod round70_content;

use osfr_tools::{
    inventory::Inventory,
    pets::{PetBusy, PetContent, PetService},
    startup_world,
};
use pet_inventory::InventoryFixture;
use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

/// Gloria's Pet Pals (vendors.json NPC 9500000246): Crab, Mini Cray, Shaggy Cow.
const GLORIA_PET_PALS: [i32; 3] = [7885, 10387, 17554];
const CRAB_ITEM: i32 = 7885;

fn bundled() -> PetContent {
    PetContent::bundled().unwrap()
}

fn shipped() -> Value {
    serde_json::from_str(include_str!("../data/content/pets.json")).unwrap()
}

fn rows<'a>(content: &'a Value, table: &str) -> &'a Vec<Value> {
    content[table].as_array().unwrap()
}

fn int(row: &Value, key: &str) -> i64 {
    row[key]
        .as_i64()
        .unwrap_or_else(|| panic!("{key} in {row}"))
}

fn summon_by_item(definition_id: i32) -> Vec<u8> {
    [
        vec![53, 0, 32],
        (definition_id as u32).to_le_bytes().to_vec(),
        0u32.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn owns(fixture: &InventoryFixture, instance: i32) -> bool {
    fixture
        .inventory
        .state()
        .items
        .iter()
        .any(|item| item.id == instance)
}

#[test]
fn every_model_and_item_resolves_through_client_models_txt() {
    let client = models::read_models();
    let content = shipped();
    let ids: BTreeSet<i64> = rows(&content, "models")
        .iter()
        .map(|m| int(m, "model_id"))
        .collect();
    for model in rows(&content, "models") {
        let adr = model["adr"].as_str().unwrap();
        assert!(
            client
                .get(&int(model, "model_id"))
                .is_some_and(|files| files.contains(adr)),
            "{adr} is not Models.txt {}",
            model["model_id"]
        );
    }
    for item in rows(&content, "items") {
        assert!(ids.contains(&int(item, "model_id")), "{item}");
    }
}

#[test]
fn every_trick_and_item_name_id_is_client_text() {
    let locale = json::read(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/reports/locale-id-to-text.json"),
    );
    let ids: BTreeSet<i64> = locale
        .as_array()
        .unwrap()
        .iter()
        .map(|row| int(row, "id"))
        .collect();
    let content = shipped();
    for row in rows(&content, "items").iter().chain(
        rows(&content, "tricks")
            .iter()
            .filter(|t| t.get("name_id").is_some()),
    ) {
        let name = int(row, "name_id");
        assert!(name == 0 || ids.contains(&name), "name id {name} of {row}");
    }
}

#[test]
fn every_item_row_matches_the_shipped_item_catalogue() {
    let inventory = Inventory::load(&round70_content::source(), None).expect("catalogue loads");
    for item in rows(&shipped(), "items") {
        let id = int(item, "item_id") as i32;
        let shown = inventory
            .presentation(id)
            .unwrap_or_else(|e| panic!("item {id}: {e}"));
        assert_eq!(shown.item_type, 2, "item {id} type");
        assert_eq!(
            i64::from(shown.name_id),
            int(item, "name_id"),
            "item {id} name"
        );
        if let Some(icon) = item["image_set_id"].as_i64() {
            assert_eq!(i64::from(shown.icon_id), icon, "item {id} icon");
        }
        if let Some(tint) = item["icon_tint_id"].as_i64() {
            assert_eq!(i64::from(shown.icon_tint), tint, "item {id} icon tint");
        }
    }
}

#[test]
fn gloria_pet_pals_are_vanity_pets() {
    let content = bundled();
    for item_id in GLORIA_PET_PALS {
        let item = content
            .item(item_id)
            .unwrap_or_else(|| panic!("item {item_id}"));
        assert!(
            content.is_vanity(item.model_id),
            "item {item_id} model {}",
            item.model_id
        );
    }
}

#[test]
fn an_owned_pet_pal_adopts_through_53_32_and_spawns_its_model() {
    let mut fixture = InventoryFixture::with_items(&[(10, CRAB_ITEM, 1)]);
    let mut service = PetService::new(bundled()).unwrap();
    let wire = service
        .handle_with_inventory(
            &summon_by_item(CRAB_ITEM),
            33,
            [1.0, 2.0, 3.0],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .expect("53/32 is a pet request");
    assert!(wire.changed, "{:?}", wire.notice);
    assert!(!owns(&fixture, 10), "the Pet Pal item is consumed");
    assert!(
        service.pet(1).is_some() && service.pet(2).is_none(),
        "exactly one pet"
    );
    let spawn = wire
        .private
        .iter()
        .chain(&wire.shared)
        .find(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .expect("the pet's AddNpc");
    assert_eq!(startup_world::parse_add_npc(spawn).unwrap().model_id, 1667);
}

#[test]
fn an_owned_item_that_is_not_a_pet_changes_nothing() {
    let mut fixture = InventoryFixture::with_items(&[(10, 1835, 1)]);
    let mut service = PetService::new(bundled()).unwrap();
    let before = service.roster_snapshot();
    let wire = service
        .handle_with_inventory(
            &summon_by_item(1835),
            33,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .expect("53/32 is a pet request");
    assert!(!wire.changed);
    assert!(owns(&fixture, 10));
    assert_eq!(service.roster_snapshot(), before);
}

#[test]
fn a_full_roster_refuses_the_item_and_keeps_it() {
    let content = bundled();
    let crab = content.definition_for_item(CRAB_ITEM).unwrap();
    let rows: Vec<_> = (0..51).map(|n| (10 + n, CRAB_ITEM, 1)).collect();
    let mut fixture = InventoryFixture::with_items(&rows);
    let mut service = PetService::new(content).unwrap();
    for n in 0..50 {
        service
            .adopt_from_owned_item(&mut fixture.inventory, 10 + n, &crab, 0)
            .unwrap();
    }
    let wire = service
        .handle_with_inventory(
            &summon_by_item(CRAB_ITEM),
            33,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .expect("53/32 is a pet request");
    assert!(!wire.changed);
    assert!(owns(&fixture, 60), "the 51st item stays");
    assert!(service.pet(51).is_none());
}

#[test]
fn wiki_species_items_keep_their_species_and_adrs_resolve_deterministically() {
    let content = bundled();
    assert_eq!(
        content.definition_for_item(70352).unwrap().species_id,
        1_000_015
    );
    assert_eq!(content.model_id("dragon_m_basic.adr"), 1756);
    // A bound ADR with no Pets.json block adopts with no tricks (R) and keeps its model id.
    let bat = content.model(4566).expect("bat_m_pinkeye.adr row");
    assert!(bat.tricks.is_empty());
    assert_eq!(content.model_id("bat_m_pinkeye.adr"), 4566);
}
