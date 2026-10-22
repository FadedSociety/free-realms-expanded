use osfr_tools::{
    character_store::{self, CharacterState},
    game_store,
    pets::PetService,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct SaveFile {
    directory: PathBuf,
    path: PathBuf,
}

impl SaveFile {
    fn new() -> Self {
        let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-save-schema-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("save.json");
        Self { directory, path }
    }
}

impl Drop for SaveFile {
    fn drop(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir(&self.directory);
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

#[test]
fn legacy_character_save_reads_and_next_write_uses_schema_version() {
    let file = SaveFile::new();
    let state = character();
    let legacy = serde_json::json!({
        "version": 1,
        "character": state.clone(),
    });
    fs::write(&file.path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

    assert_eq!(
        character_store::load_or_create(&file.path, character()).unwrap(),
        state
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&file.path).unwrap()).unwrap(),
        legacy
    );
    character_store::save(&file.path, &state).unwrap();

    let migrated: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(migrated["schema_version"], 2);
    assert!(migrated.get("version").is_none());
    assert_eq!(
        character_store::load_or_create(&file.path, character()).unwrap(),
        state
    );
}

#[test]
fn current_game_save_migrates_envelope_and_preserves_all_instance_counters() {
    let file = SaveFile::new();
    let current = include_str!("fixtures/save_schema/current-v2.json");
    let before: Value = serde_json::from_str(current).unwrap();
    assert_eq!(before["version"], 2);
    assert_eq!(before["character"]["guid"], 900000001_u64);
    assert_eq!(
        before["gameplay"]["identity_fingerprint"],
        "synthetic-identity-fingerprint-0001"
    );
    let counters = instance_counters(&before);
    for field in [
        "gameplay.inventory.next_instance_id",
        "gameplay.encounter.next_instance_id",
        "gameplay.cooking.next_instance_id",
        "gameplay.forging.next_instance_id",
    ] {
        assert!(counters.contains_key(field), "fixture missing {field}");
    }
    fs::write(&file.path, current).unwrap();
    let old_bytes = fs::read(&file.path).unwrap();

    let loaded = game_store::load_or_create(&file.path, character()).unwrap();
    assert_eq!(loaded.character.guid, 900000001);
    assert_eq!(fs::read(&file.path).unwrap(), old_bytes);
    game_store::save(&file.path, &loaded).unwrap();

    let migrated: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(
        migrated["schema_version"],
        game_store::GAMEPLAY_SCHEMA_VERSION
    );
    assert!(migrated.get("version").is_none());
    assert_eq!(instance_counters(&migrated), counters);
    let reloaded = game_store::load_or_create(&file.path, character()).unwrap();
    assert_eq!(reloaded, loaded);
}

#[test]
fn schema_three_inventory_migrates_stored_count_and_preserves_64_bit_last_cast() {
    let file = SaveFile::new();
    let mut legacy: Value =
        serde_json::from_str(include_str!("fixtures/save_schema/current-v2.json")).unwrap();
    legacy.as_object_mut().unwrap().remove("version");
    legacy["schema_version"] = serde_json::json!(3);
    let item = &mut legacy["gameplay"]["inventory"]["items"][0];
    item.as_object_mut().unwrap().remove("stored_count");
    item["last_cast_time"] = serde_json::json!(0x1122_3344_5566_7788i64);
    fs::write(&file.path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
    let old_bytes = fs::read(&file.path).unwrap();

    let loaded = game_store::load_or_create(&file.path, character()).unwrap();
    let item = &loaded.gameplay.as_ref().unwrap().inventory.items[0];
    assert_eq!(item.stored_count, 0);
    assert_eq!(item.last_cast_time, 0x1122_3344_5566_7788);
    assert_eq!(fs::read(&file.path).unwrap(), old_bytes);

    game_store::save(&file.path, &loaded).unwrap();
    let migrated: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(
        migrated["schema_version"],
        game_store::GAMEPLAY_SCHEMA_VERSION
    );
    assert_eq!(
        migrated["gameplay"]["inventory"]["items"][0]["stored_count"],
        0
    );
    assert_eq!(
        migrated["gameplay"]["inventory"]["items"][0]["last_cast_time"],
        0x1122_3344_5566_7788i64
    );
    assert_eq!(
        game_store::load_or_create(&file.path, character()).unwrap(),
        loaded
    );
}

#[test]
fn schema_four_migrates_housing_in_the_canonical_envelope_and_preserves_counters() {
    let file = SaveFile::new();
    let mut schema_four: Value =
        serde_json::from_str(include_str!("fixtures/save_schema/current-v2.json")).unwrap();
    schema_four.as_object_mut().unwrap().remove("version");
    schema_four["schema_version"] = serde_json::json!(4);
    let housing = serde_json::json!({
        "schema_version": 1,
        "lots": [{"owner_guid": 123, "lot_guid": 456, "fixtures": []}]
    });
    schema_four["housing"] = housing.clone();
    let counters = instance_counters(&schema_four);
    fs::write(&file.path, serde_json::to_vec_pretty(&schema_four).unwrap()).unwrap();
    let original = fs::read(&file.path).unwrap();

    let loaded = game_store::load_or_create(&file.path, character()).unwrap();
    assert_eq!(fs::read(&file.path).unwrap(), original);
    assert_eq!(
        game_store::load_housing_state::<Value>(&file.path).unwrap(),
        Some(housing.clone())
    );

    game_store::save(&file.path, &loaded).unwrap();
    let migrated: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(
        migrated["schema_version"],
        game_store::GAMEPLAY_SCHEMA_VERSION
    );
    assert_eq!(migrated["housing"], housing);
    assert_eq!(instance_counters(&migrated), counters);

    let replacement = serde_json::json!({"schema_version": 1, "lots": []});
    game_store::save_housing_state(&file.path, &replacement).unwrap();
    assert_eq!(
        game_store::load_housing_state::<Value>(&file.path).unwrap(),
        Some(replacement.clone())
    );
    let after: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(after["housing"], replacement);
    assert_eq!(instance_counters(&after), counters);
    assert_eq!(
        game_store::load_or_create(&file.path, character()).unwrap(),
        loaded
    );
}

fn instance_counters(value: &Value) -> BTreeMap<String, Value> {
    fn visit(value: &Value, path: &str, counters: &mut BTreeMap<String, Value>) {
        match value {
            Value::Object(object) => {
                for (key, child) in object {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    if key == "next_instance_id" {
                        counters.insert(child_path.clone(), child.clone());
                    }
                    visit(child, &child_path, counters);
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    visit(child, &format!("{path}[{index}]"), counters);
                }
            }
            _ => {}
        }
    }

    let mut counters = BTreeMap::new();
    visit(value, "", &mut counters);
    counters
}

#[test]
fn schema_five_migrates_pet_roster_and_preserves_housing_and_counters() {
    let file = SaveFile::new();
    let mut legacy: Value =
        serde_json::from_str(include_str!("fixtures/save_schema/current-v2.json")).unwrap();
    legacy.as_object_mut().unwrap().remove("version");
    legacy["schema_version"] = serde_json::json!(5);
    let housing = serde_json::json!({"schema_version": 1, "lots": []});
    legacy["housing"] = housing.clone();
    let counters = instance_counters(&legacy);
    fs::write(&file.path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

    let roster = serde_json::json!({
        "version": 1,
        "next_pet_id": 2,
        "pets": [],
        "vendor_grants": {},
    });
    let mut loaded = game_store::load_or_create(&file.path, character()).unwrap();
    loaded.pets = Some(roster.clone());
    game_store::save(&file.path, &loaded).unwrap();
    let migrated: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(
        migrated["schema_version"],
        game_store::GAMEPLAY_SCHEMA_VERSION
    );
    assert!(migrated.get("version").is_none());
    assert_eq!(migrated["housing"], housing);
    assert_eq!(migrated["pets"], roster);
    assert_eq!(instance_counters(&migrated), counters);
    let mut loaded = game_store::load_or_create(&file.path, character()).unwrap();
    assert_eq!(loaded.pets, Some(roster));

    // `None` (an empty live roster) keeps the roster already on disk.
    loaded.pets = None;
    game_store::save(&file.path, &loaded).unwrap();
    let rewritten: Value = serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(
        rewritten["schema_version"],
        game_store::GAMEPLAY_SCHEMA_VERSION
    );
    assert_eq!(rewritten["pets"]["next_pet_id"], 2);
    assert_eq!(rewritten["housing"], housing);
    assert_eq!(instance_counters(&rewritten), counters);
}

/// S17: a save in the shape the pre-S17 `save_with_pets` wrote (fc39966 built the same
/// `Document` with the roster from `serde_json::to_value`) loads with the same roster, and
/// re-saving the loaded snapshot rewrites the same bytes.
#[test]
fn a_pre_s17_pet_save_loads_the_same_roster_and_resaves_identically() {
    let file = SaveFile::new();
    let fixture = include_bytes!("fixtures/save_schema/pets-save-with-pets.json");
    fs::write(&file.path, fixture).unwrap();
    let loaded = game_store::load_or_create(&file.path, character()).unwrap();
    let pets =
        PetService::from_document(PetService::bundled_content().unwrap(), loaded.pets.clone())
            .unwrap();
    let pet = pets.pet(1).unwrap();
    assert_eq!((pet.species_id, pet.name.as_str()), (1_000_015, "Ace"));
    assert!(pet.learned_tricks.contains(&1));
    assert_eq!(pet.favorite_tricks[0], Some(1));
    assert_eq!(pets.document_value(), loaded.pets);
    game_store::save(&file.path, &loaded).unwrap();
    assert_eq!(fs::read(&file.path).unwrap(), fixture);
}
