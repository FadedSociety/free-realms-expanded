//! One unique scratch content tree for the split-loader tests.
//!
//! Test-only: the tree lives under `CARGO_TARGET_TMPDIR` with the `quests/` and
//! `npcs/` directories the split loader reads, and is removed on drop. The name
//! carries the process id and a per-call counter, so two tests (or two suites)
//! never share a tree and the shipped content is never touched.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new(tag: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("quests")).expect("scratch quests dir");
        fs::create_dir_all(root.join("npcs")).expect("scratch npcs dir");
        Self(root)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
