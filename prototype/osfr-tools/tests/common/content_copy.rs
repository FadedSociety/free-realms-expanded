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
