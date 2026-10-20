//! The private-lab round70 content base shared by the gateway fixture rigs: the reference
//! character directory, the authored content files and the 69 MB item catalogue that the
//! vendor, quest and combat fixtures all draw on. The caller adds its own NPC, quest, store
//! and job rows on top. This module has no `crate::` items so `#[path]`-inclusion works in
//! both the integration harnesses and the gateway binary's tests.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// The authoring content package the fixtures copy their base files from.
pub fn source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/round70-native-20261016T061000Z/content")
}

/// The reference character/catalogue directory every gateway fixture loads.
pub fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references/rust-gateway-reference")
}

/// Copy the base files plus `extra` into `root/<label>-<pid>-<n>` and return it. The
/// catalogue is hard-linked rather than copied, so a fixture costs one 69 MB file per test
/// process. The directory is left for the caller's scratch root to remove.
pub fn build(root: &Path, label: &str, extra: &[&str]) -> PathBuf {
    let source = source();
    let content = root.join(format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(content.join("npcs")).expect("fixture content creates");
    for file in [
        "inventory.json",
        "quests.json",
        "quest-journal.bin",
        "combat.json",
        "combat-template.bin",
        "categories.json",
    ]
    .into_iter()
    .chain(extra.iter().copied())
    {
        fs::copy(source.join(file), content.join(file))
            .unwrap_or_else(|error| panic!("fixture {file} copies: {error}"));
    }
    if fs::hard_link(
        source.join("item-catalogue.json"),
        content.join("item-catalogue.json"),
    )
    .is_err()
    {
        fs::copy(
            source.join("item-catalogue.json"),
            content.join("item-catalogue.json"),
        )
        .expect("item catalogue copies");
    }
    content
}
