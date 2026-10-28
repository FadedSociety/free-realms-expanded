//! One reader for the extracted client asset corpus (`lab/extracted/client-assets`, private).
//!
//! `catalog.tsv` columns are `pack entry name offset bytes crc32 extracted_relative_path`.
//! A suite looks an asset up by the name it already holds (`support/models.rs` lowercases the
//! `Models.txt` file names), so the column layout and the corpus root live here instead of in
//! every suite. With the lab absent [`Assets::open`] returns `None`: a private gate is
//! `#[ignore]`d rather than silently skipping. Test-only; production never reads the lab.
#![allow(dead_code)]

#[path = "paths.rs"]
mod paths;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

/// The extracted client assets `catalog.tsv` names, keyed by lowercase asset name.
pub struct Assets {
    by_name: HashMap<String, PathBuf>,
}

impl Assets {
    /// The corpus, or `None` on a checkout without the private lab files.
    pub fn open() -> Option<Self> {
        let root = paths::project_parent().join("lab/extracted/client-assets");
        let catalog = root.join("catalog.tsv");
        if !catalog.is_file() {
            return None;
        }
        let text = fs::read_to_string(&catalog)
            .unwrap_or_else(|error| panic!("{}: {error}", catalog.display()));
        let mut by_name = HashMap::new();
        for line in text.lines().skip(1) {
            let fields: Vec<&str> = line.split('\t').collect();
            if let [_, _, name, _, _, _, extracted] = fields[..] {
                by_name.insert(name.to_ascii_lowercase(), root.join(extracted));
            }
        }
        Some(Self { by_name })
    }

    /// Every extracted asset's lowercase name, for content gates that need only the set.
    pub fn names(&self) -> HashSet<String> {
        self.by_name.keys().cloned().collect()
    }

    /// The extracted bytes of the asset named `name` (case-insensitive).
    pub fn read(&self, name: &str) -> Vec<u8> {
        let path = self
            .by_name
            .get(&name.to_ascii_lowercase())
            .unwrap_or_else(|| panic!("client asset {name} is absent from catalog.tsv"));
        fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
    }
}
