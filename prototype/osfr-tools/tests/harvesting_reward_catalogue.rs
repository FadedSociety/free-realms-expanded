//! Missing reward-definition guard: a harvesting content whose configured
//! outputs are not defined in the loaded inventory catalogue must refuse
//! `Gameplay::load` before any offer is possible.
//!
//! Fixtures copy the private campaign content into a temp directory and write
//! `harvesting.json` there; no shared content, save or process is touched.

use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    harvesting,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// Four definitions the frozen catalogue does define.
const PRESENT: [i32; 4] = [2739, 879, 65834, 2436];
static NEXT: AtomicU64 = AtomicU64::new(0);

fn base_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-harvesting-reward-catalogue-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        Self(directory)
    }

    /// The shipped region-111 configuration (`harvesting::write_content`).
    fn write_shipped(&self) {
        harvesting::write_content(&self.0, 111).unwrap();
    }

    /// The same configuration with its four output definitions replaced.
    fn write_outputs(&self, definitions: [i32; 4]) {
        let path = self.0.join(harvesting::CONTENT_FILE);
        let mut value: Value = if path.is_file() {
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap()
        } else {
            serde_json::from_str(&harvesting::content_json(111).unwrap()).unwrap()
        };
        for (slot, definition) in definitions.iter().enumerate() {
            value["spec"]["outputs"][slot]["definition_id"] = json!(definition);
        }
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn load_error(directory: &Path, state: Option<GameplayState>) -> String {
    match Gameplay::load(directory, state) {
        Ok(_) => panic!("Gameplay::load accepted unusable harvesting rewards"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn real_package_missing_bundle_definitions_refuse_gameplay_load() {
    let fixture = Fixture::new();
    fixture.write_shipped();
    let error = load_error(&fixture.0, None);
    assert!(
        error.contains("harvesting reward definition")
            && (error.contains("2885") || error.contains("2890")),
        "the refusal must name a missing bundle definition: {error}"
    );
}

#[test]
fn wrong_or_late_output_definition_is_refused() {
    let fixture = Fixture::new();
    fixture.write_outputs([2885, 2887, 2890, 2877]);
    let error = load_error(&fixture.0, None);
    assert!(error.contains("reward definition"), "{error}");

    // A late (fourth) output is caught too.
    let fixture = Fixture::new();
    fixture.write_outputs([2887, 2877, 2739, 99999]);
    let error = load_error(&fixture.0, None);
    assert!(
        error.contains("99999"),
        "a late unknown definition must be named: {error}"
    );

    // And a first-position unknown definition.
    let fixture = Fixture::new();
    fixture.write_outputs([99999, 2877, 2739, 879]);
    let error = load_error(&fixture.0, None);
    assert!(error.contains("99999"), "{error}");
}

#[test]
fn valid_definitions_load_and_legacy_save_bytes_stay_compatible() {
    let fixture = Fixture::new();
    fixture.write_outputs(PRESENT);
    let state = Gameplay::load(&fixture.0, None)
        .expect("the four manifest-present definitions must load")
        .state();
    assert!(state.harvesting.is_some());

    let old = json!({
        "inventory": {"coins": 0, "items": [], "equipment": []},
        "quests": {"version": 1, "quests": {}, "trial_best_ms": null, "trial_completions": 0},
        "combat": {"enemy_health": 100, "defeats": 0}
    });
    let legacy: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(legacy.harvesting.is_none());
    assert_eq!(serde_json::to_value(&legacy).unwrap(), old);
    Gameplay::load(&fixture.0, Some(state)).expect("the loaded state remains valid");
}

#[test]
fn refused_load_preserves_the_caller_state_and_content_bytes() {
    let fixture = Fixture::new();
    fixture.write_outputs(PRESENT);
    let state = Gameplay::load(&fixture.0, None).unwrap().state();
    let before_state = serde_json::to_value(&state).unwrap();

    fixture.write_shipped();
    let path = fixture.0.join(harvesting::CONTENT_FILE);
    let before_bytes = fs::read(&path).unwrap();
    let error = load_error(&fixture.0, Some(state.clone()));
    assert!(error.contains("reward definition"), "{error}");
    assert_eq!(
        serde_json::to_value(&state).unwrap(),
        before_state,
        "a refused load must not mutate the caller's save"
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before_bytes,
        "a refused load must not rewrite the content"
    );
}
