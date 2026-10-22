use osfr_tools::{
    character_store::CharacterState,
    game_store::{
        BASE_SCHEMA_VERSION, MAX_SNAPSHOT_BYTES, PET_SCHEMA_VERSION, Snapshot, load_or_create,
        save, save_housing_state,
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
            gameplay: None,
            pets: None,
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
    let with_pets = Snapshot {
        character: initial(),
        gameplay: None,
        pets: Some(serde_json::json!({"roster": []})),
    };
    save(&path, &with_pets).unwrap();

    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["schema_version"], PET_SCHEMA_VERSION);

    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}

/// Lead (persistence), S17: a pet change that also changes the character (adoption consumes an
/// item) is written as one atomic document, keeps housing, a `None` roster keeps the one on disk,
/// and a refused write changes nothing. The production adoption path (session commit, restart,
/// refused replacement) is pinned by the gateway binary's `f161_pet_adoption_persist` tests;
/// this one covers the envelope itself.
#[test]
fn one_save_writes_character_and_roster_and_none_keeps_the_disk_roster() {
    let dir =
        std::env::temp_dir().join(format!("osfr-game-store-with-pets-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("state.json");
    load_or_create(&path, initial()).unwrap();
    save_housing_state(&path, &serde_json::json!({"rooms": 1})).unwrap();

    let roster = serde_json::json!({"roster": [{"id": 7}]});
    let moved = Snapshot {
        character: CharacterState {
            position: [9.0, 9.0, 9.0],
            ..initial()
        },
        gameplay: None,
        pets: Some(roster.clone()),
    };
    save(&path, &moved).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        saved["character"]["position"],
        serde_json::json!([9.0, 9.0, 9.0])
    );
    assert_eq!(saved["pets"], roster);
    assert_eq!(saved["housing"], serde_json::json!({"rooms": 1}));
    assert_eq!(saved["schema_version"], PET_SCHEMA_VERSION);

    // A later save without a roster (`None`, as an empty live roster sends) keeps it on disk.
    let moved = Snapshot {
        pets: None,
        ..moved
    };
    save(&path, &moved).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["pets"], roster);

    // A refused write leaves the file byte-identical: no half-applied character or roster.
    let before = fs::read(&path).unwrap();
    let invalid = Snapshot {
        character: CharacterState {
            position: [f32::NAN, 0.0, 0.0],
            ..initial()
        },
        gameplay: None,
        pets: Some(serde_json::json!({"roster": []})),
    };
    assert!(save(&path, &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);

    fs::remove_file(path).unwrap();
    fs::remove_dir(dir).unwrap();
}

/// L043: an admin moving an offline character rewrites only its position and rotation; a save
/// in another zone or a missing save is refused and left exactly as it was.
#[test]
fn replace_pose_changes_only_the_pose_and_refusals_change_nothing() {
    let dir = std::env::temp_dir().join(format!("osfr-game-store-pose-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("state.json");
    load_or_create(&path, initial()).unwrap();
    save_housing_state(&path, &serde_json::json!({"rooms": [1, 2]})).unwrap();
    let with_pets = Snapshot {
        character: initial(),
        gameplay: None,
        pets: Some(serde_json::json!({"roster": [7]})),
    };
    save(&path, &with_pets).unwrap();
    let before: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let (from, to) = osfr_tools::game_store::replace_pose(
        &path,
        0,
        [-1904.46, -39.29, 413.316],
        Some([0.0, 0.0, -1.0]),
    )
    .unwrap();
    assert_eq!(from, initial());
    assert_eq!(to.position, [-1904.46, -39.29, 413.316]);
    let mut after: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        after["character"]["rotation"],
        serde_json::json!([0.0, 0.0, -1.0])
    );
    let mut expected = before.clone();
    for document in [&mut after, &mut expected] {
        let character = document["character"].as_object_mut().unwrap();
        character.remove("position");
        character.remove("rotation");
    }
    assert_eq!(after, expected, "only the pose may change");
    // `None` keeps the saved facing.
    let (_, kept) = osfr_tools::game_store::replace_pose(&path, 0, [1.0, 1.0, 1.0], None).unwrap();
    assert_eq!(kept.rotation, [0.0, 0.0, -1.0]);

    let saved = fs::read(&path).unwrap();
    let elsewhere = osfr_tools::game_store::replace_pose(&path, 411, [0.0; 3], None).unwrap_err();
    assert_eq!(elsewhere.kind(), std::io::ErrorKind::InvalidInput);
    let far = osfr_tools::game_store::replace_pose(&path, 0, [2_000_000.0, 0.0, 0.0], None);
    assert!(far.is_err());
    assert_eq!(
        fs::read(&path).unwrap(),
        saved,
        "a refusal rewrote the save"
    );
    let missing = dir.join("never-entered.json");
    let absent = osfr_tools::game_store::replace_pose(&missing, 0, [0.0; 3], None).unwrap_err();
    assert_eq!(absent.kind(), std::io::ErrorKind::NotFound);
    assert!(!missing.exists(), "a move must never create a save");
    fs::remove_dir_all(dir).unwrap();
}
