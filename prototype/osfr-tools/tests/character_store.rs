use std::{
    fs, io,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use osfr_tools::character_store::{
    CharacterState, MAX_COMPONENT_MAGNITUDE, MAX_FILE_BYTES, load_or_create, save,
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
            "osfr-character-store-{}-{}",
            std::process::id(),
            sequence
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("character.json");
        Self { directory, path }
    }
}

impl Drop for SaveFile {
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

fn character() -> CharacterState {
    CharacterState {
        guid: 123456789,
        zone_id: 2,
        position: [234.5, -15.25, 800.0],
        rotation: [-1.0, 2.0, 0.125],
    }
}

#[test]
fn creates_a_save_and_ignores_a_new_default_on_load() {
    let file = SaveFile::new();
    let initial = character();
    assert_eq!(
        load_or_create(&file.path, initial.clone()).unwrap(),
        initial
    );
    let mut moved = initial;
    moved.position = [1000.5, 200.25, -300.75];
    moved.rotation = [0.0, -0.25, 3.0];
    save(&file.path, &moved).unwrap();
    // A fresh load has no shared in-memory state and ignores the new default.
    let mut different_default = character();
    different_default.guid = 999;
    assert_eq!(
        load_or_create(&file.path, different_default).unwrap(),
        moved
    );
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(&file.path).unwrap()).unwrap();
    assert_eq!(document["schema_version"], 2);
    assert!(document.get("version").is_none());
}

#[test]
fn a_fresh_process_restores_saved_coordinates() {
    const CHILD_PATH: &str = "OSFR_CHARACTER_STORE_RESTART_TEST";
    let mut moved = character();
    moved.position = [1000.5, 200.25, -300.75];
    moved.rotation = [0.0, -0.25, 3.0];
    if let Some(path) = std::env::var_os(CHILD_PATH) {
        assert_eq!(
            load_or_create(&PathBuf::from(path), character()).unwrap(),
            moved
        );
        return;
    }
    let file = SaveFile::new();
    save(&file.path, &moved).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "a_fresh_process_restores_saved_coordinates",
            "--nocapture",
        ])
        .env(CHILD_PATH, &file.path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh test process failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn successive_saves_replace_the_file_and_leave_no_temporary_files() {
    let file = SaveFile::new();
    let mut state = character();
    for index in 0..10 {
        state.position[0] = index as f32 * 12.5;
        save(&file.path, &state).unwrap();
        assert_eq!(load_or_create(&file.path, character()).unwrap(), state);
    }
    assert_eq!(fs::read_dir(&file.directory).unwrap().count(), 1);
}

#[test]
fn corrupt_and_unsupported_existing_files_are_not_reset() {
    let file = SaveFile::new();
    for bytes in [
        b"not JSON".to_vec(),
        b"{}".to_vec(),
        br#"{"version":2,"character":{"guid":1,"zone_id":2,"position":[0,0,0],"rotation":[0,0,0]}}"#.to_vec(),
        br#"{"version":1,"character":{"guid":0,"zone_id":2,"position":[0,0,0],"rotation":[0,0,0]}}"#.to_vec(),
        br#"{"version":1,"character":{"guid":1,"zone_id":2,"position":[1000001,0,0],"rotation":[0,0,0]}}"#.to_vec(),
        br#"{"version":1,"character":{"guid":1,"zone_id":2,"position":[0,0,0],"rotation":[0,0]}}"#.to_vec(),
    ] {
        fs::write(&file.path, &bytes).unwrap();
        let error = load_or_create(&file.path, character()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read(&file.path).unwrap(), bytes);
    }
}

#[test]
fn oversized_existing_save_is_bounded_and_preserved() {
    let file = SaveFile::new();
    let bytes = vec![b' '; MAX_FILE_BYTES + 1];
    fs::write(&file.path, &bytes).unwrap();
    let error = load_or_create(&file.path, character()).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("16 KiB"));
    assert_eq!(fs::read(&file.path).unwrap(), bytes);
}

#[test]
fn invalid_updates_preserve_the_prior_valid_save() {
    let file = SaveFile::new();
    let valid = character();
    save(&file.path, &valid).unwrap();
    let previous = fs::read(&file.path).unwrap();
    let mut invalid = valid.clone();
    invalid.guid = 0;
    assert!(save(&file.path, &invalid).is_err());
    for value in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        MAX_COMPONENT_MAGNITUDE + 1.0,
        -MAX_COMPONENT_MAGNITUDE - 1.0,
    ] {
        invalid = valid.clone();
        invalid.position[1] = value;
        assert!(save(&file.path, &invalid).is_err());
        invalid = valid.clone();
        invalid.rotation[2] = value;
        assert!(save(&file.path, &invalid).is_err());
        assert_eq!(fs::read(&file.path).unwrap(), previous);
    }
    assert_eq!(load_or_create(&file.path, character()).unwrap(), valid);
    assert_eq!(fs::read_dir(&file.directory).unwrap().count(), 1);
}

#[cfg(windows)]
#[test]
fn transient_windows_reader_without_delete_sharing_allows_atomic_save_after_release() {
    use std::{fs::OpenOptions, io::Read, os::windows::fs::OpenOptionsExt, thread, time::Duration};

    let file = SaveFile::new();
    let initial = character();
    save(&file.path, &initial).unwrap();
    let previous = fs::read(&file.path).unwrap();
    // Ordinary observers can read while denying replacement of this pathname.
    // Keep read/write sharing; omit only FILE_SHARE_DELETE to reproduce OS5.
    let mut reader = OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(&file.path)
        .unwrap();
    let observed = previous.clone();
    let released = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, observed);
        drop(reader);
    });
    let mut moved = initial.clone();
    moved.position[0] += 10.0;
    let result = save(&file.path, &moved);
    released.join().unwrap();
    result.expect("atomic save should retry the transient reader's replacement denial");
    assert_eq!(load_or_create(&file.path, initial).unwrap(), moved);
    assert_eq!(fs::read_dir(&file.directory).unwrap().count(), 1);
}

#[cfg(windows)]
#[test]
fn failed_windows_replacement_preserves_the_previous_file() {
    use std::{fs::OpenOptions, os::windows::fs::OpenOptionsExt};

    let file = SaveFile::new();
    let valid = character();
    save(&file.path, &valid).unwrap();
    let previous = fs::read(&file.path).unwrap();
    // Denying delete sharing forces MoveFileEx replacement to fail on Windows.
    let locked = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&file.path)
        .unwrap();
    let mut moved = valid.clone();
    moved.position[0] += 10.0;
    assert!(save(&file.path, &moved).is_err());
    drop(locked);
    assert_eq!(fs::read(&file.path).unwrap(), previous);
    assert_eq!(load_or_create(&file.path, character()).unwrap(), valid);
    assert_eq!(fs::read_dir(&file.directory).unwrap().count(), 1);
}
