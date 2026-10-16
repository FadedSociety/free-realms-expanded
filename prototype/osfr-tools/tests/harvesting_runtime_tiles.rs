//! Runtime Harvest tile-registration guard (gameplay load gate).
//!
//! ORIGINAL context (see `notes/Research/Harvesting Runtime Tile Guard.md`):
//! `core/TileGenerator.as::SetTileInfo(value, index)` is the only place the client learns a
//! selectable/collectable value, and `generate_selectable` dereferences `aNames[value]`. The
//! shared engine emits `OnTileInfoMsg\t<value>\t<value - category>` for every tile-list entry,
//! so a persisted harvesting config that advertises recovered produce values (`400 + type`,
//! `farmingValue401..416`) but does not register them silently leaves every regular tile
//! undrawable and every basket unresolved.
//!
//! RED baseline: `Gameplay::load` accepts those malformed persisted configs (they parse and the
//! engine stays permissive for Mining), so the malformed cases below fail until the optional
//! Harvest load gate refuses them.
//!
//! Fixtures copy the private harvesting campaign content into a temp directory and rewrite only
//! `harvesting.json`; the save/content on disk must be byte-identical after a refusal.

use osfr_tools::{gameplay::Gameplay, harvesting, mining};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const HARVESTING_GAME_ID: i32 = 111;
const CHEF_PROFILE_ID: i32 = 45;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn base_content() -> PathBuf {
    std::env::var_os("OSFR_HARVEST_RUNTIME_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-harvesting-native-20261009T132512Z/content")
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

fn tree_fingerprint(directory: &Path) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            continue;
        }
        let bytes = fs::read(entry.path()).unwrap();
        entries.push((
            entry.file_name().to_string_lossy().into_owned(),
            format!("{:x}", Sha256::digest(&bytes)),
        ));
    }
    entries.sort();
    entries
}

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-harvesting-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        Self { directory }
    }

    fn harvesting_path(&self) -> PathBuf {
        self.directory.join(harvesting::CONTENT_FILE)
    }

    fn write_harvesting_document(&self, document: &Value) {
        fs::write(
            self.harvesting_path(),
            serde_json::to_vec(document).unwrap(),
        )
        .unwrap();
    }

    fn read_harvesting_document(&self) -> Value {
        serde_json::from_slice(&fs::read(self.harvesting_path()).unwrap()).unwrap()
    }

    fn load(&self) -> std::io::Result<Gameplay> {
        Gameplay::load(&self.directory, None)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// The fixed writer's region 111 document, the valid four-kind reference.
fn valid_document() -> Value {
    let content = harvesting::content(HARVESTING_GAME_ID).expect("region 111 content builds");
    serde_json::to_value(content).expect("content serializes")
}

fn kind(value: u16, category: u16, icon_id: i32) -> Value {
    json!({"value": value, "category": category, "icon_id": icon_id})
}

fn load_refusal_matches_tiles(fixture: &Fixture, case: &str) {
    let before = tree_fingerprint(&fixture.directory);
    match fixture.load() {
        Ok(_) => panic!("{case}: a malformed tile registration loaded without refusal"),
        Err(error) => assert!(
            error.to_string().contains("tile"),
            "{case}: expected a tile-registration refusal, saw {error}"
        ),
    }
    let after = tree_fingerprint(&fixture.directory);
    assert_eq!(
        before, after,
        "{case}: a refused load must not mutate the save or content"
    );
}

#[test]
fn malformed_persisted_harvesting_configs_are_refused_without_mutation() {
    // Missing produce mapping: a recovered goal value absent from the tile list.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["tiles"]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry["value"] != 411);
    fixture.write_harvesting_document(&document);
    load_refusal_matches_tiles(&fixture, "missing produce 411");

    // Duplicate produce mapping.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["tiles"]
        .as_array_mut()
        .unwrap()
        .push(kind(411, 400, 11));
    fixture.write_harvesting_document(&document);
    load_refusal_matches_tiles(&fixture, "duplicate produce 411");

    // Wrong category for a produce value.
    let fixture = Fixture::new();
    let mut document = valid_document();
    for entry in document["spec"]["tiles"].as_array_mut().unwrap() {
        if entry["value"] == 411 {
            entry["category"] = json!(600);
        }
    }
    fixture.write_harvesting_document(&document);
    load_refusal_matches_tiles(&fixture, "wrong category for 411");

    // A kind that belongs to neither recovered harvesting set.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["tiles"]
        .as_array_mut()
        .unwrap()
        .push(kind(505, 500, 0));
    fixture.write_harvesting_document(&document);
    load_refusal_matches_tiles(&fixture, "out-of-set kind 505");
}

#[test]
fn valid_four_kind_content_loads() {
    let fixture = Fixture::new();
    harvesting::write_content(&fixture.directory, HARVESTING_GAME_ID).expect("writer succeeds");
    let document = fixture.read_harvesting_document();
    assert_eq!(document["spec"]["tiles"].as_array().unwrap().len(), 4);
    let gameplay = fixture
        .load()
        .expect("the fixed four-kind content loads through the gameplay gate");
    assert!(
        gameplay.state().harvesting.is_some(),
        "the loaded gameplay keeps its harvesting engine"
    );
}

#[test]
fn missing_content_legacy_and_mining_stay_permissive() {
    // Legacy campaign without harvesting content still loads.
    let fixture = Fixture::new();
    fs::remove_file(fixture.harvesting_path()).unwrap();
    let gameplay = fixture
        .load()
        .expect("a campaign without harvesting content loads");
    assert!(gameplay.state().harvesting.is_none());

    // Mining keeps the generic permissive engine contract; the Harvest gate must not apply.
    let fixture = Fixture::new();
    fs::remove_file(fixture.harvesting_path()).unwrap();
    mining::write_content(&fixture.directory, mining::MINING_GAME_ID).expect("mining writer");
    let gameplay = fixture.load().expect("mining content loads unchanged");
    assert!(gameplay.state().mining.is_some());
    assert!(gameplay.state().harvesting.is_none());

    // The Chef profile and game id stay the recovered ones.
    let fixture = Fixture::new();
    let document = valid_document();
    assert_eq!(document["game_id"], json!(HARVESTING_GAME_ID));
    assert_eq!(document["profile_id"], json!(CHEF_PROFILE_ID));
    fixture.write_harvesting_document(&document);
    fixture.load().expect("the valid document loads");
}
