//! T454: the imported pet catalogue (`pets.json` models, items and tricks from
//! `scripts/pets_import.py`) checked against the client's own tables, and the real 53/32
//! adoption path for pet items the wiki species rows never listed.
#[path = "support/json.rs"]
mod json;
#[path = "support/models.rs"]
mod models;
#[path = "support/pet_inventory.rs"]
mod pet_inventory;
#[path = "../src/bin/osfr_gateway/pet_wire.rs"]
mod pet_wire;
#[path = "common/round70_content.rs"]
mod round70_content;

use osfr_tools::{
    inventory::Inventory,
    pets::{PetBusy, PetContent, PetService},
    startup_world,
};
use pet_inventory::InventoryFixture;
use pet_wire::summon_by_item;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Gloria's Pet Pals (vendors.json NPC 9500000246): Crab, Mini Cray, Shaggy Cow.
const GLORIA_PET_PALS: [i32; 3] = [7885, 10387, 17554];
const CRAB_ITEM: i32 = 7885;
/// Firefly: the client item whose `firefly.adr` has no Pets.json row (Models.txt 483 and 3243).
const FIREFLY_ITEM: i32 = 17_543;
/// Candy Corn: the client item whose `vanitypet_candycorn_critter.adr` is Models.txt 1530.
const CANDY_CORN_ITEM: i32 = 30_589;

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

fn summon(
    game: &mut PetService,
    fixture: &mut InventoryFixture,
    item: i32,
) -> startup_world::NpcSpawn {
    let wire = game
        .handle_with_inventory(
            &summon_by_item(item as u32, 0),
            33,
            [1.0, 2.0, 3.0],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .expect("53/32 is a pet request");
    assert!(wire.changed, "{:?}", wire.notice);
    let spawn = wire
        .private
        .iter()
        .chain(&wire.shared)
        .find(|packet| packet.starts_with(&[35, 0, 2, 0]))
        .expect("the pet's AddNpc");
    startup_world::parse_add_npc(spawn).unwrap()
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
    let texts: BTreeMap<i64, &str> = locale
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| Some((row.get("id")?.as_i64()?, row.get("text")?.as_str()?)))
        .collect();
    let content = shipped();
    for row in rows(&content, "items").iter().chain(
        rows(&content, "tricks")
            .iter()
            .filter(|t| t.get("name_id").is_some()),
    ) {
        let name = int(row, "name_id");
        assert!(
            name == 0 || texts.contains_key(&name),
            "name id {name} of {row}"
        );
    }
    // The item display name is that client text, not a second name source.
    for item in rows(&content, "items") {
        let name = int(item, "name_id");
        assert_eq!(
            item["name"].as_str(),
            Some(texts[&name]),
            "item name id {name} text"
        );
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
    let spawn = summon(&mut service, &mut fixture, CRAB_ITEM);
    assert!(!owns(&fixture, 10), "the Pet Pal item is consumed");
    assert!(
        service.pet(1).is_some() && service.pet(2).is_none(),
        "exactly one pet"
    );
    assert_eq!(spawn.model_id, 1667);
    // An item-only species has no wiki row; the item's client text names the actor.
    assert_eq!(spawn.npc_name, "Crab");
}

#[test]
fn a_candy_corn_item_adopts_with_its_client_name_and_texture() {
    // Candy Corn's client ModelName (`vanitypet_candycorn_critter.adr`) is not a Models.txt ADR;
    // its TextureAlias names `candycorn_critter.adr` (1530). Its pet has no wiki species row, so
    // the item's locale text and texture alias are the only display sources.
    let mut fixture = InventoryFixture::with_items(&[(11, CANDY_CORN_ITEM, 1)]);
    let mut service = PetService::new(bundled()).unwrap();
    let spawn = summon(&mut service, &mut fixture, CANDY_CORN_ITEM);
    assert_eq!(spawn.model_id, 1530);
    assert_eq!(spawn.npc_name, "Candy Corn");
    assert_eq!(spawn.texture_alias, "candycorn-critter");
    assert!(!owns(&fixture, 11), "the item is consumed");
    let pet = service.pet(1).expect("one pet");
    assert_eq!(pet.species_id, CANDY_CORN_ITEM as u32);
    assert_eq!(pet.model, "candycorn_critter.adr");
    assert_eq!(pet.adoption_item_definition_id, Some(CANDY_CORN_ITEM));
}

#[test]
fn every_client_type_2_item_is_a_pet_adoption_item() {
    // The pinned Legacy file is the source the catalogue was built from; a future drop must fail here,
    // not disappear between the source count and the shipped rows.
    let client = json::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
    ));
    let content = shipped();
    let bound: BTreeSet<i64> = rows(&content, "items")
        .iter()
        .map(|row| int(row, "item_id"))
        .collect();
    let mut count = 0;
    for item in client.as_array().unwrap() {
        if item.get("Type").and_then(Value::as_i64) != Some(2) {
            continue;
        }
        count += 1;
        let id = int(item, "Id");
        assert!(bound.contains(&id), "type-2 item {id} is not in pets.json");
    }
    assert_eq!(count, 528, "the pinned client file's Type-2 count changed");
    assert_eq!(bound.len(), count);
    let bundled = bundled();
    for (item_id, model_id) in [(FIREFLY_ITEM, 483), (CANDY_CORN_ITEM, 1530)] {
        let item = bundled
            .item(item_id)
            .unwrap_or_else(|| panic!("item {item_id}"));
        assert_eq!(item.model_id, model_id, "item {item_id}");
    }
}

#[test]
fn an_owned_item_that_is_not_a_pet_changes_nothing() {
    let mut fixture = InventoryFixture::with_items(&[(10, 1835, 1)]);
    let mut service = PetService::new(bundled()).unwrap();
    let before = service.roster_snapshot();
    let wire = service
        .handle_with_inventory(
            &summon_by_item(1835, 0),
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
    let rows: Vec<_> = (0..51).map(|n| (10 + n, CRAB_ITEM, 1)).collect();
    let mut fixture = InventoryFixture::with_items(&rows);
    let mut service = PetService::new(bundled()).unwrap();
    for n in 0..50 {
        let spawn = summon(&mut service, &mut fixture, CRAB_ITEM);
        assert_eq!(spawn.model_id, 1667, "adoption {n}");
    }
    assert_eq!(
        fixture.inventory.state().items.len(),
        1,
        "each request spent exactly one certificate"
    );
    let wire = service
        .handle_with_inventory(
            &summon_by_item(CRAB_ITEM as u32, 0),
            33,
            [0.0; 3],
            &mut fixture.inventory,
            PetBusy::default(),
            0,
        )
        .unwrap()
        .expect("53/32 is a pet request");
    assert!(!wire.changed);
    assert_eq!(
        fixture.inventory.state().items.len(),
        1,
        "the 51st item stays"
    );
    assert!(service.pet(51).is_none());
}

#[test]
fn every_shared_adr_resolves_to_its_lower_client_id() {
    // Seven ADRs have two or three client ids and the item and species rows name only the ADR; the
    // lower id is the deterministic choice (R), pinned here so a re-import cannot silently change it.
    let content = bundled();
    for (adr, model_id) in [
        ("snowhog.adr", 138),
        ("dog_large_floppy.adr", 169),
        ("dog_small_fluffy.adr", 364),
        ("dog_large_dalmation.adr", 514),
        ("dog_large_pointyghost.adr", 1018),
        ("dragon_m_basic.adr", 1756),
        ("tyrannosaurus_m.adr", 1759),
    ] {
        assert_eq!(content.model_id(adr), model_id, "{adr}");
    }
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
    // Firefly's only client item names its ADR; the lower of its two Models.txt ids is used (R).
    assert_eq!(
        content.definition_for_item(FIREFLY_ITEM).unwrap().model,
        "firefly.adr"
    );
    assert_eq!(content.model_id("firefly.adr"), 483);
}
