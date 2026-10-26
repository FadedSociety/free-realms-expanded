//! Runtime Harvest border gate (gameplay load gate).
//!
//! ORIGINAL context (`notes/Research/Harvesting Border Compatibility.md`): the original fall loops
//! never visit index 0. `ServerGrid.as`/`Grid.as::checkFalls` scan columns with `while (... > 0)`
//! and rows with `while (... > 0)`, and `addColumnFall` marks rows with `while (... > 0)`, so a
//! playable cell on shape row 0 or shape column 0 can never move. The pre-border export
//! (`29538eb4…`, ten-by-ten with playable x0..9 / y0..8) therefore still loaded through
//! `Gameplay::load` even though its regular tiles cannot fall.
//!
//! RED baseline: that unpadded eight-kind document loads; GREEN refuses it (row-0 and column-0
//! cells both tested). The permissive side (padded writer output, Mining-only and no-content
//! legacy campaigns) is asserted in `harvesting_runtime_tiles.rs`.
//!
//! Fixtures copy the private harvesting campaign content into a temp directory and rewrite only
//! `harvesting.json`; a refusal must leave the tree byte-identical.

use osfr_tools::{gameplay::Gameplay, harvesting};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const GAME_ID: i32 = 111;

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
        entries.push((
            entry.file_name().to_string_lossy().into_owned(),
            format!("{:x}", Sha256::digest(fs::read(entry.path()).unwrap())),
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
            "osfr-harvesting-runtime-border-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        Self { directory }
    }

    fn harvesting_path(&self) -> PathBuf {
        self.directory.join(harvesting::CONTENT_FILE)
    }

    fn write_document(&self, document: &Value) {
        fs::write(
            self.harvesting_path(),
            serde_json::to_vec(document).unwrap(),
        )
        .unwrap();
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

/// The fixed writer's padded region 111 document.
fn valid_document() -> Value {
    serde_json::to_value(harvesting::content(GAME_ID).expect("content builds")).unwrap()
}

/// The pre-border ten-by-ten shape: playable x0..9 / y0..8, drop row 9.
fn unpadded_shape() -> Vec<Value> {
    let mut shape = vec![json!(1); 90];
    shape.extend(vec![json!(4); 10]);
    shape
}

fn expect_refused(fixture: &Fixture, case: &str) {
    let before = tree_fingerprint(&fixture.directory);
    match fixture.load() {
        Ok(_) => panic!("{case}: an index-0 playable cell loaded without refusal"),
        Err(error) => assert!(
            error.to_string().contains("border"),
            "{case}: expected a border refusal, saw {error}"
        ),
    }
    assert_eq!(
        before,
        tree_fingerprint(&fixture.directory),
        "{case}: a refused load must not mutate the save or content"
    );
}

#[test]
fn unpadded_eight_kind_export_is_refused_without_mutation() {
    // The piece-030 counterexample: eight valid kinds but the old 10x10 unpadded geometry.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["width"] = json!(10);
    document["spec"]["height"] = json!(10);
    document["spec"]["shape"] = Value::Array(unpadded_shape());
    fixture.write_document(&document);
    expect_refused(&fixture, "unpadded eight-kind export");

    // Row 0 only: an 11x10 board whose 10x9 rectangle sits at y0..8 with a left border. The
    // shared engine accepts this rectangle; only the border gate refuses it.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["width"] = json!(11);
    document["spec"]["height"] = json!(10);
    let mut shape = vec![json!(0); 110];
    for row in 0..9 {
        for column in 1..11 {
            shape[row * 11 + column] = json!(1);
        }
    }
    for column in 1..11 {
        shape[9 * 11 + column] = json!(4);
    }
    document["spec"]["shape"] = Value::Array(shape);
    fixture.write_document(&document);
    expect_refused(&fixture, "playable rectangle on row 0");

    // Column 0 only: a 10x11 board whose 10x9 rectangle sits at x0..9 with a top border.
    let fixture = Fixture::new();
    let mut document = valid_document();
    document["spec"]["width"] = json!(10);
    document["spec"]["height"] = json!(11);
    let mut shape = vec![json!(0); 110];
    for row in 1..10 {
        for column in 0..10 {
            shape[row * 10 + column] = json!(1);
        }
    }
    for column in 0..10 {
        shape[10 * 10 + column] = json!(4);
    }
    document["spec"]["shape"] = Value::Array(shape);
    fixture.write_document(&document);
    expect_refused(&fixture, "playable rectangle on column 0");
}
