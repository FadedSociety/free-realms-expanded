use osfr_tools::pets::{
    PetBusy, PetContent, PetDefinition, PetEquipmentSlot, PetPresenceEvent, PetService,
};
use osfr_tools::{character_store::CharacterState, game_store};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[path = "support/pet_inventory.rs"]
mod pet_inventory;
use pet_inventory::InventoryFixture;

fn definition() -> PetDefinition {
    PetDefinition {
        species_id: 652,
        model: "pet_test_model".into(),
        adoption_item_definition_id: Some(9001),
        initial_tricks: Vec::new(),
    }
}

fn character() -> CharacterState {
    CharacterState {
        guid: 123,
        zone_id: 7,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

struct SaveFile {
    directory: PathBuf,
    path: PathBuf,
}

impl SaveFile {
    fn new() -> Self {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("osfr-pets-save-{}-{sequence}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        Self {
            path: directory.join("save.json"),
            directory,
        }
    }
}

impl Drop for SaveFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_dir(&self.directory);
    }
}

#[test]
fn adopts_owned_item_and_generated_name_validation() {
    let mut fixture = InventoryFixture::new();
    let content = PetService::bundled_content().unwrap();
    assert!(content.definition(652).is_none());
    let mut service = PetService::new(content).unwrap();
    let id = service
        .adopt_from_owned_item(&mut fixture.inventory, 10, &definition(), 0)
        .unwrap();
    assert_eq!(
        fixture
            .inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == 10),
        None
    );

    assert_eq!(service.rename(id, 1, 0).unwrap(), "Admiral Ace");
    let old_name = service.pet(id).unwrap().name.clone();
    assert!(service.rename(id, 60, 0).is_err());
    assert_eq!(service.pet(id).unwrap().name, old_name);
}

#[test]
fn summon_and_dismiss_return_client_pet_active_bytes_and_presence_events() {
    let mut fixture = InventoryFixture::new();
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let id = service
        .adopt_from_owned_item(&mut fixture.inventory, 10, &definition(), 0)
        .unwrap();
    let summoned = service
        .summon(id, 44, 33, [1.0, 2.0, 3.0], PetBusy::default(), 0)
        .unwrap();
    let mut expected = vec![53, 0, 9];
    expected.extend(id.to_le_bytes());
    expected.extend(44u64.to_le_bytes());
    assert_eq!(summoned.owner_message, expected);
    assert!(
        matches!(summoned.broadcasts.as_slice(), [PetPresenceEvent::Spawn(row)] if row.actor_guid == 44 && row.owner_guid == 33)
    );

    let dismissed = service.dismiss(0).unwrap();
    assert_eq!(
        dismissed.owner_message,
        [53, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert!(matches!(
        dismissed.broadcasts.as_slice(),
        [PetPresenceEvent::Despawn { actor_guid: 44, .. }]
    ));
}

#[test]
fn follow_and_equipment_state_validate_owned_items_and_slots() {
    let mut fixture = InventoryFixture::new();
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let pet_id = service
        .adopt_from_owned_item(&mut fixture.inventory, 10, &definition(), 0)
        .unwrap();
    service
        .summon(pet_id, 44, 33, [0.0; 3], PetBusy::default(), 0)
        .unwrap();
    let followed = service.follow_location([4.0, 5.0, 6.0]).unwrap();
    assert_eq!(followed.owner_message[0..3], [53, 0, 26]);
    assert_eq!(
        followed.broadcasts,
        [PetPresenceEvent::Follow {
            actor_guid: 44,
            target: [4.0, 5.0, 6.0]
        }]
    );
    service.set_in_water(true).unwrap();
    assert_eq!(
        service.active_follow_state(),
        Some((44, Some([4.0, 5.0, 6.0]), true))
    );

    service
        .equip_item(pet_id, 11, PetEquipmentSlot::Utility(0), &fixture.inventory)
        .unwrap();
    assert_eq!(
        service.pet(pet_id).unwrap().equipment.utilities[0],
        Some(11)
    );
    service
        .unequip_item(pet_id, PetEquipmentSlot::Utility(0))
        .unwrap();
    assert_eq!(service.pet(pet_id).unwrap().equipment.utilities[0], None);
    assert!(
        service
            .equip_item(pet_id, 999, PetEquipmentSlot::Head, &fixture.inventory)
            .is_err()
    );
    for slot in [
        PetEquipmentSlot::Head,
        PetEquipmentSlot::Collar,
        PetEquipmentSlot::Chest,
        PetEquipmentSlot::Feet,
    ] {
        service
            .equip_item(pet_id, 13, slot, &fixture.inventory)
            .unwrap();
        service.unequip_item(pet_id, slot).unwrap();
    }
    service
        .equip_item(pet_id, 13, PetEquipmentSlot::Toy, &fixture.inventory)
        .unwrap();
    service.advance_time(1_000).unwrap();
    let play_before = service.pet(pet_id).unwrap().needs.play;
    assert_eq!(service.play(pet_id, 1_000).unwrap()[0..3], [53, 0, 3]);
    assert!(service.pet(pet_id).unwrap().needs.play > play_before);
}

#[test]
fn care_consumes_one_owned_food_and_restores_food_need() {
    let mut fixture = InventoryFixture::new();
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let pet_id = service
        .adopt_from_owned_item(&mut fixture.inventory, 10, &definition(), 0)
        .unwrap();
    service
        .summon(pet_id, 44, 33, [1.0, 2.0, 3.0], PetBusy::default(), 0)
        .unwrap();
    service
        .equip_item(pet_id, 11, PetEquipmentSlot::Utility(0), &fixture.inventory)
        .unwrap();
    service.advance_time(1_000).unwrap();
    let before = service.pet(pet_id).unwrap().needs.food;
    let outcome = service
        .care(
            pet_id,
            11,
            &mut fixture.inventory,
            PetBusy::default(),
            1_000,
        )
        .unwrap();
    assert_eq!(
        fixture
            .inventory
            .state()
            .items
            .iter()
            .find(|item| item.id == 11)
            .unwrap()
            .count,
        1
    );
    assert!(service.pet(pet_id).unwrap().needs.food > before);
    assert_eq!(outcome.pet_status_packet[0..3], [53, 0, 3]);
}

#[test]
fn needs_decay_faster_while_summoned() {
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let pet_id = service.adopt_vendor_grant(1, &definition(), 0).unwrap();
    service.advance_time(600).unwrap();
    let idle_food = service.pet(pet_id).unwrap().needs.food;
    assert!((idle_food - 94.0).abs() < 0.01);
    service
        .summon(pet_id, 44, 33, [0.0; 3], PetBusy::default(), 600)
        .unwrap();
    service.advance_time(1_200).unwrap();
    assert!(service.pet(pet_id).unwrap().needs.food < idle_food - 10.0);
}

#[test]
fn trick_list_favourite_and_roster_reload_persist() {
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let pet_id = service.adopt_vendor_grant(2, &definition(), 0).unwrap();
    service.learn_trick(pet_id, 1).unwrap();
    // The wire slot is zero-based (the client sender subtracts one from the UI slot).
    let favorite = service.set_favorite(pet_id, 0, 1).unwrap();
    assert_eq!(
        favorite,
        [
            vec![53, 0, 14],
            pet_id.to_le_bytes().to_vec(),
            0u32.to_le_bytes().to_vec(),
            1u32.to_le_bytes().to_vec()
        ]
        .concat()
    );

    let file = SaveFile::new();
    game_store::save(
        &file.path,
        &game_store::Snapshot {
            character: character(),
            gameplay: None,
            pets: service.document_value(),
        },
    )
    .unwrap();
    let saved = game_store::load_or_create(&file.path, character()).unwrap();
    let reloaded =
        PetService::from_document(PetService::bundled_content().unwrap(), saved.pets).unwrap();
    let pet = reloaded.pet(pet_id).unwrap();
    assert!(pet.learned_tricks.contains(&1));
    assert_eq!(pet.favorite_tricks[0], Some(1));
    assert_eq!(reloaded.active_pet_id(), None);
    let document: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], 6);
}

#[test]
fn minigame_busy_rule_rejects_summon_without_mutation() {
    let mut service = PetService::new(PetService::bundled_content().unwrap()).unwrap();
    let pet_id = service.adopt_vendor_grant(3, &definition(), 0).unwrap();
    assert!(
        service
            .summon(
                pet_id,
                44,
                33,
                [0.0; 3],
                PetBusy {
                    minigame_active: true
                },
                0
            )
            .is_err()
    );
    assert_eq!(service.active_pet_id(), None);
}

#[test]
fn species_catalog_has_field_sources_and_resolved_or_unresolved_item_ids() {
    let content: Value = serde_json::from_str(include_str!("../data/content/pets.json")).unwrap();
    let startup: Value = serde_json::from_str(include_str!(
        "../../../../lab/content/startup-snapshot/startup.json"
    ))
    .unwrap();
    let species = content["species"].as_array().unwrap();
    let item_ids: std::collections::BTreeSet<_> = startup["catalogues"]["item_definitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|definition| definition["id"].as_i64())
        .collect();
    let mut resolved_item_ids = 0usize;
    let mut unresolved_item_ids = 0usize;

    for row in species {
        assert!(!row["source"].as_array().unwrap().is_empty());
        for field in [
            "name",
            "wiki_title",
            "client_name_string_id",
            "client_name_string_id_candidates",
            "species_id",
            "species_kind",
            "breed",
            "model",
            "model_files",
            "texture_alias",
            "icon",
            "adoption_item_ids",
            "member_only",
            "adoption_source",
        ] {
            assert!(
                row["field_sources"][field]
                    .as_array()
                    .is_some_and(|sources| !sources.is_empty()),
                "missing field sources for {field} on {}",
                row["wiki_title"]
            );
        }
        for adoption in row["adoption_source"].as_array().unwrap() {
            assert!(!adoption["source"].as_array().unwrap().is_empty());
            for field in [
                "kind",
                "store_item_id",
                "store_price_coins",
                "price",
                "member_price",
                "currency",
                "reward",
                "promotion",
            ] {
                assert!(
                    adoption["field_sources"][field]
                        .as_array()
                        .is_some_and(|sources| !sources.is_empty()),
                    "missing adoption field sources for {field} on {}",
                    row["wiki_title"]
                );
            }
            let item_id = adoption["store_item_id"].as_i64();
            if let Some(item_id) = item_id {
                if item_ids.contains(&item_id) {
                    resolved_item_ids += 1;
                } else {
                    assert!(
                        adoption["unknown_reasons"]["store_item_id"]
                            .as_str()
                            .is_some_and(|reason| !reason.is_empty())
                    );
                    unresolved_item_ids += 1;
                }
            } else {
                assert!(
                    adoption["unknown_reasons"]["store_item_id"]
                        .as_str()
                        .is_some_and(|reason| !reason.is_empty())
                );
                unresolved_item_ids += 1;
            }
        }
    }

    let named_pets = content["named_pets"].as_array().unwrap();
    for pet in named_pets {
        assert!(!pet["source"].as_array().unwrap().is_empty());
        assert!(pet["field_sources"].as_object().is_some());
    }

    println!(
        "pet species: {}; named NPC pets: {}; adoption item IDs matched: {resolved_item_ids}; unresolved: {unresolved_item_ids}",
        species.len(),
        named_pets.len()
    );
    assert_eq!(
        resolved_item_ids + unresolved_item_ids,
        content["species"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["adoption_source"].as_array().unwrap().len())
            .sum::<usize>()
    );
}

#[test]
fn client_pet_bindings_exist_and_species_ids_are_unique() {
    let content: Value = serde_json::from_str(include_str!("../data/content/pets.json")).unwrap();
    let startup: Value = serde_json::from_str(include_str!(
        "../../../../lab/content/startup-snapshot/startup.json"
    ))
    .unwrap();
    let asset_names: std::collections::BTreeSet<_> =
        include_str!("../../../../lab/extracted/client-assets/catalog.tsv")
            .lines()
            .skip(1)
            .filter_map(|line| line.split('\t').nth(2))
            .map(str::to_ascii_lowercase)
            .collect();
    let item_ids: std::collections::BTreeSet<_> = startup["catalogues"]["item_definitions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["id"].as_u64())
        .collect();
    let pet_content = PetContent::bundled().unwrap();
    let mut species_ids = std::collections::BTreeSet::new();
    let mut bound_models = 0usize;
    let mut adoption_items = 0usize;

    for row in content["species"].as_array().unwrap() {
        let species_id = row["species_id"].as_u64().unwrap();
        assert!(species_id > 0 && species_ids.insert(species_id));

        if let Some(model) = row["model"].as_str() {
            bound_models += 1;
            assert!(model.ends_with(".adr"));
            assert!(asset_names.contains(&model.to_ascii_lowercase()), "{model}");
            let files = row["model_files"].as_array().unwrap();
            assert!(!files.is_empty());
            for file in files {
                let file = file.as_str().unwrap();
                assert!(asset_names.contains(&file.to_ascii_lowercase()), "{file}");
            }
            let definition = pet_content.definition(species_id as u32).unwrap();
            assert_eq!(definition.model, model);
            let item_id = row["adoption_item_ids"]
                .as_array()
                .unwrap()
                .first()
                .and_then(Value::as_u64);
            assert_eq!(
                definition.adoption_item_definition_id,
                item_id.map(|id| id as i32)
            );
        }

        for item_id in row["adoption_item_ids"].as_array().unwrap() {
            let item_id = item_id.as_u64().unwrap();
            assert!(item_ids.contains(&item_id), "unknown pet item {item_id}");
            adoption_items += 1;
        }
    }

    println!(
        "pet bindings: {} species; {bound_models} models; {adoption_items} adoption items; {} unique species IDs",
        content["species"].as_array().unwrap().len(),
        species_ids.len()
    );
    assert!(bound_models > 0);
    assert!(adoption_items > 0);
    assert_eq!(
        species_ids.len(),
        content["species"].as_array().unwrap().len()
    );
}

#[test]
fn every_bound_pet_model_has_a_models_txt_id() {
    let content: Value = serde_json::from_str(include_str!("../data/content/pets.json")).unwrap();
    let models: Vec<_> = content["species"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row["model"].as_str())
        .collect();
    assert_eq!(models.len(), 26);
    let bundled = PetContent::bundled().unwrap();
    for model in models {
        assert!(
            bundled.model_id(model) > 0,
            "missing Models.txt id for {model}"
        );
    }
}
