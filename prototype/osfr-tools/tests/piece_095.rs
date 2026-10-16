//! Piece 095 (multi-character persistence) tests: reversible single-character
//! migration, per-character isolation and cross-process restart proofs.
//!
//! Evidence boundary, read before the assertions:
//!
//! ORIGINAL (first-party client inputs; hashes pinned in the lab bundle):
//! * `decompiled/client-ui/ScriptsBase.lua`: `AccountInfo_DS.MaxCharacters = 1`,
//!   `AccountInfo_DS.CharacterGuid = 4`, `AccountInfo_DS.CharacterName = 5`;
//!   `CharacterSelect:Populate` invokes `addCharacter` once per account-info row
//!   and always appends the localized `Ui.GetStringById(436106)` row with GUID 0;
//!   `CharacterSelect:GetMaxCharacters` reads `ds:GetData(0, AccountInfo_DS.MaxCharacters)`;
//!   `OnUserEvent("addCharacter")` continues only when `GetMaxCharacters() > 1`
//!   and otherwise shows locale 435820.
//! * Locale rows 436106 "Add a Character" and 435820 "Get more Characters!
//!   Become A Member!".
//!
//! COMMUNITY (pinned `lab/references/FreeRealms-Legacy`, cross-check only):
//! `AccountInfo` writes IsMember/MaxCharacters/IsAdminAccount in that order,
//! `CharacterSelectInfoReply` (opcode 12) carries an entity list, the create
//! handler refuses beyond `DbUser.MaxCharacters` with result 6, `DbUser.MaxCharacters`
//! defaults to 10, `DbCharacter` has a `UserId` owner and `GuidHelper` encodes a
//! player guid as `id << 4 | 1`.
//!
//! RECONSTRUCTED here: the version-2 multi-character document, the local slot
//! bound, the migration provenance digest and the rollback rule. The original
//! per-account slot count is unrecovered; 10 is only the community default.
//!
//! The module is not registered in `src/lib.rs` (lead-owned). Including the exact
//! source file keeps this target compiling against the real artifact. Integration
//! step for the lead: register the module and drop this `#[path]` include.
#[path = "../src/restoration/piece_095.rs"]
#[allow(dead_code)]
mod piece_095;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use piece_095::{
    CharacterState, MAX_CHARACTERS, MAX_STORE_BYTES, Migration, MultiCharacterStore, STORE_VERSION,
    load, load_legacy, migrate_legacy, rollback_to_legacy, save, save_legacy,
};

static DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct StoreDirectory {
    directory: PathBuf,
    legacy: PathBuf,
    store: PathBuf,
}

impl StoreDirectory {
    fn new() -> Self {
        let sequence = DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-095-{}-{}",
            std::process::id(),
            sequence
        ));
        fs::create_dir(&directory).unwrap();
        Self {
            legacy: directory.join("character.json"),
            store: directory.join("characters.json"),
            directory,
        }
    }
}

impl Drop for StoreDirectory {
    fn drop(&mut self) {
        // These fixtures create only files in their uniquely owned directory.
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries.flatten() {
                let _ = fs::remove_file(entry.path());
            }
        }
        let _ = fs::remove_dir(&self.directory);
    }
}

fn character(guid: u64) -> CharacterState {
    CharacterState {
        guid,
        zone_id: 2,
        position: [234.5 + guid as f32, -15.25, 800.0],
        rotation: [-1.0, 2.0, 0.125],
    }
}

fn two_characters() -> (CharacterState, CharacterState) {
    let first = character(1);
    let mut second = character(0x1_0000_0021);
    second.zone_id = 12;
    second.position = [1000.5, 200.25, -300.75];
    second.rotation = [0.0, -0.25, 3.0];
    (first, second)
}

#[test]
fn migration_imports_the_legacy_character_and_keeps_the_source_bytes() {
    let files = StoreDirectory::new();
    let legacy = character(1);
    save_legacy(&files.legacy, &legacy).unwrap();
    let source = fs::read(&files.legacy).unwrap();

    let migration = migrate_legacy(&files.legacy, &files.store, character(1)).unwrap();
    assert_eq!(migration, Migration::Imported { guid: 1 });
    // The single-character save is the rollback source; migration never rewrites it.
    assert_eq!(fs::read(&files.legacy).unwrap(), source);

    let store = load(&files.store).unwrap();
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(1), Some(&legacy));
    assert_eq!(store.active(), Some(1));

    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(&files.store).unwrap()).unwrap();
    assert_eq!(document["version"], STORE_VERSION);

    // A second start of an unchanged pair is not a second import.
    assert_eq!(
        migrate_legacy(&files.legacy, &files.store, character(1)).unwrap(),
        Migration::Existing
    );
}

#[test]
fn a_fresh_install_creates_both_saves_and_imports_once() {
    let files = StoreDirectory::new();
    let default = character(1);

    assert_eq!(
        migrate_legacy(&files.legacy, &files.store, default.clone()).unwrap(),
        Migration::Imported { guid: 1 }
    );
    // Both files exist: the version-1 save keeps the single-character contract,
    // and the version-2 store records the digest of exactly these bytes.
    let legacy_bytes = fs::read(&files.legacy).unwrap();
    assert_eq!(load_legacy(&files.legacy, character(1)).unwrap(), default);
    assert_eq!(
        migrate_legacy(&files.legacy, &files.store, character(1)).unwrap(),
        Migration::Existing
    );
    assert_eq!(fs::read(&files.legacy).unwrap(), legacy_bytes);

    // Because the digest is recorded, even a fresh install refuses divergence.
    let mut rewritten = default;
    rewritten.position = [1.5, 2.5, 3.5];
    save_legacy(&files.legacy, &rewritten).unwrap();
    assert!(migrate_legacy(&files.legacy, &files.store, character(1)).is_err());
}

#[test]
fn characters_share_no_state_and_survive_a_save_load_cycle() {
    let files = StoreDirectory::new();
    let (first, second) = two_characters();
    let mut store = MultiCharacterStore::new();
    store.insert(first.clone()).unwrap();
    store.insert(second.clone()).unwrap();
    store.select(second.guid).unwrap();

    let mut moved = first.clone();
    moved.position = [1.0, 2.0, 3.0];
    moved.rotation = [4.0, 5.0, 6.0];
    store.update(moved.clone()).unwrap();
    // Isolation: updating one entry leaves every field of the other untouched.
    assert_eq!(store.get(second.guid), Some(&second));

    save(&files.store, &store).unwrap();
    let loaded = load(&files.store).unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get(moved.guid), Some(&moved));
    assert_eq!(loaded.get(second.guid), Some(&second));
    assert_eq!(loaded.active(), Some(second.guid));

    // Isolation across files: a second store keeps its own characters for the
    // same guids and is not touched by the first store's save.
    let other_path = files.directory.join("other-characters.json");
    let mut other = MultiCharacterStore::new();
    let mut relocated = second.clone();
    relocated.position = [7.0, 8.0, 9.0];
    other.insert(relocated.clone()).unwrap();
    save(&other_path, &other).unwrap();
    let other_bytes = fs::read(&other_path).unwrap();
    save(&files.store, &store).unwrap();
    assert_eq!(fs::read(&other_path).unwrap(), other_bytes);
    assert_eq!(
        load(&other_path).unwrap().get(second.guid),
        Some(&relocated)
    );
}

#[test]
fn two_readers_of_the_same_file_share_no_state() {
    let files = StoreDirectory::new();
    let original = character(1);
    let mut store = MultiCharacterStore::new();
    store.insert(original.clone()).unwrap();
    save(&files.store, &store).unwrap();

    let mut first = load(&files.store).unwrap();
    let second = load(&files.store).unwrap();
    let mut moved = original.clone();
    moved.position = [11.0, 12.0, 13.0];
    first.update(moved.clone()).unwrap();
    assert_eq!(first.get(1), Some(&moved));
    // A second reader is an independent value, not a view of the first.
    assert_eq!(second.get(1), Some(&original));
    assert_eq!(second.active(), Some(1));
}

#[test]
fn migration_rolls_back_byte_for_byte() {
    let files = StoreDirectory::new();
    let legacy = character(1);
    save_legacy(&files.legacy, &legacy).unwrap();
    let source = fs::read(&files.legacy).unwrap();

    migrate_legacy(&files.legacy, &files.store, character(1)).unwrap();
    let store = load(&files.store).unwrap();
    rollback_to_legacy(&store, &files.legacy, 1).unwrap();
    // The real version-1 writer produces the identical document, so a rollback
    // can restore the exact single-character save that was migrated.
    assert_eq!(fs::read(&files.legacy).unwrap(), source);
    assert_eq!(load_legacy(&files.legacy, character(1)).unwrap(), legacy);
}

#[test]
fn rollback_refuses_to_drop_a_second_character() {
    let files = StoreDirectory::new();
    let (first, second) = two_characters();
    save_legacy(&files.legacy, &first).unwrap();
    let source = fs::read(&files.legacy).unwrap();

    migrate_legacy(&files.legacy, &files.store, character(1)).unwrap();
    let mut store = load(&files.store).unwrap();
    store.insert(second).unwrap();
    let error = rollback_to_legacy(&store, &files.legacy, first.guid).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("more than one character"));
    assert_eq!(fs::read(&files.legacy).unwrap(), source);
}

#[test]
fn a_diverged_legacy_save_is_refused_and_leaves_the_store_bytes_alone() {
    let files = StoreDirectory::new();
    save_legacy(&files.legacy, &character(1)).unwrap();
    migrate_legacy(&files.legacy, &files.store, character(1)).unwrap();
    let store_bytes = fs::read(&files.store).unwrap();

    let mut rewritten = character(1);
    rewritten.position = [999.0, 0.0, 0.0];
    save_legacy(&files.legacy, &rewritten).unwrap();
    let error = migrate_legacy(&files.legacy, &files.store, character(1)).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("changed after migration"));
    assert_eq!(fs::read(&files.store).unwrap(), store_bytes);
}

#[test]
fn invalid_or_duplicate_characters_are_refused() {
    let mut store = MultiCharacterStore::new();
    store.insert(character(1)).unwrap();

    let error = store.insert(character(1)).unwrap_err();
    assert!(error.to_string().contains("duplicate"));

    let mut zero_guid = character(1);
    zero_guid.guid = 0;
    assert!(store.insert(zero_guid).is_err());

    let mut not_finite = character(2);
    not_finite.position[1] = f32::NAN;
    assert!(store.insert(not_finite).is_err());

    assert!(store.update(character(3)).is_err());
    assert!(store.select(3).is_err());

    for guid in 2..=MAX_CHARACTERS as u64 {
        store.insert(character(guid)).unwrap();
    }
    assert_eq!(store.len(), MAX_CHARACTERS);
    let error = store
        .insert(character(MAX_CHARACTERS as u64 + 1))
        .unwrap_err();
    assert!(error.to_string().contains(&MAX_CHARACTERS.to_string()));
}

#[test]
fn corrupt_unsupported_or_oversized_stores_are_preserved() {
    let files = StoreDirectory::new();
    let valid = br#"{"version":2,"active":1,"characters":[{"guid":1,"zone_id":2,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0]}]}"#;
    fs::write(&files.store, valid).unwrap();
    assert_eq!(load(&files.store).unwrap().active(), Some(1));

    let duplicate = br#"{"version":2,"active":1,"characters":[{"guid":1,"zone_id":2,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0]},{"guid":1,"zone_id":3,"position":[1.0,0.0,0.0],"rotation":[0.0,0.0,0.0]}]}"#;
    let cases: Vec<Vec<u8>> = vec![
        b"not JSON".to_vec(),
        b"{}".to_vec(),
        br#"{"version":3,"active":1,"characters":[]}"#.to_vec(),
        br#"{"version":2,"active":9,"characters":[{"guid":1,"zone_id":2,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0,0.0]}]}"#.to_vec(),
        br#"{"version":2,"active":1,"characters":[{"guid":1,"zone_id":2,"position":[0.0,0.0,0.0],"rotation":[0.0,0.0]}]}"#.to_vec(),
        duplicate.to_vec(),
        vec![b' '; MAX_STORE_BYTES + 1],
    ];
    for bytes in cases {
        fs::write(&files.store, &bytes).unwrap();
        let error = load(&files.store).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(fs::read(&files.store).unwrap(), bytes);
    }
}

#[test]
fn a_fresh_process_restores_every_character_and_the_active_selection() {
    const CHILD_PATH: &str = "OSFR_PIECE_095_RESTART_TEST";
    let (first, second) = two_characters();
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        let store = load(Path::new(&path)).unwrap();
        assert_eq!(store.len(), 2);
        assert_eq!(store.get(first.guid), Some(&first));
        assert_eq!(store.get(second.guid), Some(&second));
        assert_eq!(store.active(), Some(second.guid));
        return;
    }
    let files = StoreDirectory::new();
    let mut store = MultiCharacterStore::new();
    store.insert(first.clone()).unwrap();
    store.insert(second.clone()).unwrap();
    store.select(second.guid).unwrap();
    save(&files.store, &store).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "a_fresh_process_restores_every_character_and_the_active_selection",
            "--nocapture",
        ])
        .env(CHILD_PATH, &files.store)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh test process failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
