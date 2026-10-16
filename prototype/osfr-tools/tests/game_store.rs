use osfr_tools::{
    character_store::CharacterState,
    game_store::{
        BASE_SCHEMA_VERSION, MAX_SNAPSHOT_BYTES, PET_SCHEMA_VERSION, Snapshot, load_or_create,
        save, save_pets_state,
    },
};
use std::fs;

fn initial() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

#[test]
fn version_one_migrates_without_changing_coordinates_and_corruption_is_preserved() {
    let dir = std::env::temp_dir().join(format!("osfr-game-store-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("state.json");
    let legacy = serde_json::json!({
        "version": 1,
        "character": initial(),
    });
    fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
    let before = fs::read(&path).unwrap();
    let loaded = load_or_create(&path, initial()).unwrap();
    assert_eq!(
        loaded,
        Snapshot {
            character: initial(),
            gameplay: None
        }
    );
    assert_eq!(fs::read(&path).unwrap(), before); // Loading is not an implicit write.
    save(&path, &loaded).unwrap();
    let upgraded: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // Saves without a pet payload retain the version-five envelope.
    assert_eq!(upgraded["schema_version"], BASE_SCHEMA_VERSION);
    assert!(upgraded.get("version").is_none());
    assert_eq!(load_or_create(&path, initial()).unwrap(), loaded);
    for bytes in [b"not-json".to_vec(), vec![b' ';MAX_SNAPSHOT_BYTES+1],br#"{"version":99,"character":{"guid":1,"zone_id":0,"position":[1,2,3],"rotation":[0,0,1]}}"#.to_vec()] {
        fs::write(&path,&bytes).unwrap();
        assert!(load_or_create(&path,initial()).is_err());
        assert_eq!(fs::read(&path).unwrap(),bytes);
    }
    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}

#[test]
fn pet_state_uses_the_current_schema_constant() {
    let dir = std::env::temp_dir().join(format!("osfr-game-store-pets-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("state.json");
    load_or_create(&path, initial()).unwrap();
    save_pets_state(&path, &serde_json::json!({"roster": []})).unwrap();

    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["schema_version"], PET_SCHEMA_VERSION);

    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}
