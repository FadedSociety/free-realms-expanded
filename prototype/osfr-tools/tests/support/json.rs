//! One JSON reader for the content gates.
//!
//! The content tests (`content_npc_ambient.rs`) and the T507 gateway test both
//! read shipped JSON; one reader keeps the panic message and the read rule from
//! drifting apart. Test-only: production loaders have their own typed readers.

use serde::Serialize;
use serde_json::Value;
use std::{fs, path::Path};

/// The JSON document at `path`, panicking with the path on a read or parse error.
#[allow(dead_code)] // Some suites include this reader only to write.
pub fn read(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The shipped `pets.json` document (the pet suites read the same file).
#[allow(dead_code)] // Some suites include this reader only for other content.
pub fn pets_json() -> Value {
    read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/pets.json"))
}

/// Write a JSON document in the shipped two-space form with a trailing newline.
#[allow(dead_code)] // Some suites include this reader only to read content.
pub fn write_pretty(path: &Path, value: &impl Serialize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}
