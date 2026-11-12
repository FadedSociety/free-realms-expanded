//! Shared test helper: copy a private content tree into a scratch directory.
//!
//! Include with `#[path = "common/content_copy.rs"] mod content_copy;`. Test-only; the production
//! path never copies content.
use std::fs;

pub fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// Hard-link one file into place, copying only when the filesystem refuses the
/// link (a different volume, or a permission the sandbox withholds).
#[allow(dead_code)] // the field-battle suites link; the other includes use another helper
pub fn hard_link_or_copy(from: &std::path::Path, to: &std::path::Path) {
    if fs::hard_link(from, to).is_err() {
        fs::copy(from, to).unwrap();
    }
}

/// The opening reference (`lab/references/rust-opening-cooking-original-assets`) copied to
/// `CARGO_TARGET_TMPDIR/<name>`, with the release item table beside it as a release ships one:
/// the reference's own `inventory.json` lacks one of the Brawler's battle-96 prizes (rc-0137).
#[allow(dead_code)] // only the suites that start battle 96 on the opening reference
pub fn opening_with_release_items(name: &str) -> std::path::PathBuf {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let reference = manifest.join("../../../lab/references/rust-opening-cooking-original-assets");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&root);
    copy_tree(&reference, &root);
    let table = manifest.join("data/release/item-table.json");
    hard_link_or_copy(&table, &root.join("item-table.json"));
    root
}

/// Copy every entry in `from` that `to` does not already supply, recursively.
/// The destination's existing files win, so a fixture's own profile/inventory
/// files survive an overlay of the repository content.
#[allow(dead_code)] // only the quest, reward and soccer fixtures overlay; other includes use `copy_tree`
pub fn copy_missing(from: &std::path::Path, to: &std::path::Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_missing(&entry.path(), &target);
        } else if !target.exists() {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
