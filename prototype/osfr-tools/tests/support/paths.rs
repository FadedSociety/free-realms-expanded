//! The CartoonMMO root that holds `lab/`, for the suites that read private lab files.
//!
//! Test-only. One parent chain keeps a suite from hardcoding its own `../../../lab` and
//! land beside the corpus it means to read.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The CartoonMMO root, the parent of the repository checkout and `lab/`.
pub fn project_parent() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}
