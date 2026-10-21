//! One reader for the client's `Resources/Models.txt` (`id^filename^...`) table.
//!
//! `content_enemies.rs` and `content_npc_spawns.rs` both gate content against the client's
//! model table; one parser keeps a fix from landing in only one gate. The canonical Python
//! reader is `scripts/content_check.py::load_caret_ids` (trim both cells, skip a non-numeric
//! id). Test-only: production never reads the client table.

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

/// Model id -> every file name the client maps it to, trimmed and lowercased.
pub fn read_models() -> HashMap<i64, HashSet<String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/working/OSFR.Client/OSFR Client/Resources/Models.txt");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut models: HashMap<i64, HashSet<String>> = HashMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split('^').collect();
        let Ok(id) = fields[0].trim().parse::<i64>() else {
            continue;
        };
        let name = fields
            .get(1)
            .unwrap_or_else(|| panic!("Models.txt row for {id} has no filename"));
        models
            .entry(id)
            .or_default()
            .insert(name.trim().to_ascii_lowercase());
    }
    models
}
