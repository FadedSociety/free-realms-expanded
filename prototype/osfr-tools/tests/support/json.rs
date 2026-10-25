//! One JSON reader for the content gates.
//!
//! The content tests (`content_npc_ambient.rs`) and the T507 gateway test both
//! read shipped JSON; one reader keeps the panic message and the read rule from
//! drifting apart. Test-only: production loaders have their own typed readers.

use serde_json::Value;
use std::{fs, path::Path};

/// The JSON document at `path`, panicking with the path on a read or parse error.
pub fn read(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
