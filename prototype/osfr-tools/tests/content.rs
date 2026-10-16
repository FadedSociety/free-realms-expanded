use osfr_tools::content;

use content::{ContentDb, ContentError};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct FixtureDir(PathBuf);

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content")
}

fn copied_fixture() -> FixtureDir {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target = std::env::temp_dir().join(format!("osfr-content-{}-{stamp}", std::process::id()));
    fs::create_dir(&target).unwrap();
    for entry in fs::read_dir(fixture_dir()).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
    }
    FixtureDir(target)
}

fn edit_json(dir: &Path, file: &str, edit: impl FnOnce(&mut Value)) {
    let path = dir.join(file);
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

#[test]
fn validated_content_set_loads() {
    let db = ContentDb::load(fixture_dir()).unwrap();

    assert_eq!(db.zones.len(), 1);
    assert_eq!(db.npcs.len(), 1);
    assert_eq!(db.pois.len(), 1);
    assert_eq!(db.items.len(), 1);
    assert_eq!(db.quests.len(), 1);
    assert_eq!(db.jobs.len(), 1);
    assert_eq!(db.dungeons.len(), 1);
    assert_eq!(db.minigames.len(), 1);
}

#[test]
fn duplicate_ids_fail_with_the_dataset_and_id() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "zones.json", |value| {
        let row = value["records"][0].clone();
        value["records"].as_array_mut().unwrap().push(row);
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::DuplicateId { kind, id } => {
            assert_eq!(kind, "zone");
            assert_eq!(id, "7");
        }
        error => panic!("expected duplicate zone id, got {error}"),
    }
}

#[test]
fn dangling_references_fail_with_the_owner_field_and_target() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "quests.json", |value| {
        value["records"][0]["giver_npc_id"] = 999999.into();
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::DanglingReference {
            owner,
            field,
            target,
        } => {
            assert_eq!(owner, "quest 101");
            assert_eq!(field, "giver_npc_id");
            assert_eq!(target, "999999");
        }
        error => panic!("expected dangling quest giver, got {error}"),
    }
}

#[test]
fn missing_reward_items_fail_with_the_grant_field_and_target() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "quests.json", |value| {
        value["records"][0]["rewards"][0]["items"][0]["definition_id"] = (-999).into();
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::DanglingReference {
            owner,
            field,
            target,
        } => {
            assert_eq!(owner, "quest 101");
            assert_eq!(field, "reward.items.definition_id");
            assert_eq!(target, "-999");
        }
        error => panic!("expected dangling reward item, got {error}"),
    }
}

#[test]
fn missing_zone_references_fail_with_the_owner_field_and_target() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "npcs.json", |value| {
        value["records"][0]["zone_id"] = 999.into();
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::DanglingReference {
            owner,
            field,
            target,
        } => {
            assert_eq!(owner, "npc 2732686529148769137");
            assert_eq!(field, "zone_id");
            assert_eq!(target, "999");
        }
        error => panic!("expected dangling NPC zone, got {error}"),
    }
}

#[test]
fn records_without_provenance_fail_with_the_file_and_index() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "items.json", |value| {
        value["records"][0]
            .as_object_mut()
            .unwrap()
            .remove("source");
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::MissingProvenance { file, record_index } => {
            assert_eq!(file, "items.json");
            assert_eq!(record_index, 0);
        }
        error => panic!("expected missing provenance, got {error}"),
    }
}

#[test]
fn unsupported_schema_versions_fail_precisely() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "zones.json", |value| {
        value["schema_version"] = 2.into();
    });

    assert!(matches!(
        ContentDb::load(&fixture.0),
        Err(ContentError::UnsupportedSchemaVersion {
            file,
            found: 2,
            expected: 1
        }) if file == "zones.json"
    ));
}

#[test]
fn unknown_record_fields_fail_with_the_file_name() {
    let fixture = copied_fixture();
    edit_json(&fixture.0, "items.json", |value| {
        value["records"][0]["nmae_string_id"] = 123.into();
    });

    match ContentDb::load(&fixture.0).unwrap_err() {
        ContentError::InvalidJson { file, message } => {
            assert_eq!(file, "items.json");
            assert!(message.contains("nmae_string_id"));
        }
        error => panic!("expected unknown-field error, got {error}"),
    }
}
