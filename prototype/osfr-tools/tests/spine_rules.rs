//! Spine rules ratchet (lead): numbers that may only go down. `spine_rules.ceilings` holds the
//! ceilings; builders may not edit it. After every integrated batch `integrate.sh` runs this
//! test with `SPINE_RATCHET_LOWER=1`, which lowers each ceiling to today's value (hubs keep
//! `HUB_HEADROOM` lines of slack) and prints the table for the drift log.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const HUBS: [&str; 4] = [
    "src/gameplay.rs",
    "src/bin/osfr-gateway.rs",
    "src/encounter.rs",
    "src/gateway.rs",
];
/// Matches the per-task hub budget in `tick.sh`.
const HUB_HEADROOM: usize = 40;
/// Activity state read by hand instead of through `Gameplay::is_occupied`.
const BUSY_TERMS: [&str; 7] = [
    "Forging::active",
    "Smelting::active",
    "Cooking::active",
    "Type8Engine::active",
    "ChessService::is_active",
    "EncounterEngine::active",
    "DerbyGateway::active",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every production source under `src`, with test code removed: files declared as
/// `#[cfg(test)] mod x;` (or `#[path]`) are skipped, and an inline `#[cfg(test)] mod x {`
/// cuts the rest of its file.
fn production_sources(root: &Path) -> BTreeMap<PathBuf, Vec<String>> {
    let mut paths = Vec::new();
    rust_files(&root.join("src"), &mut paths);
    let mut tests = Vec::new();
    let mut sources = BTreeMap::new();
    for path in paths {
        let text = fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let mut keep = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if line.trim() == "#[cfg(test)]" {
                let mut next = index + 1;
                let mut path_attr = None;
                while lines.get(next).is_some_and(|l| l.trim().starts_with("#[")) {
                    let attr = lines[next].trim();
                    if let Some(rest) = attr.strip_prefix("#[path = \"") {
                        path_attr = rest.strip_suffix("\"]").map(str::to_owned);
                    }
                    next += 1;
                }
                let decl = lines.get(next).map_or("", |l| l.trim());
                let decl = decl.strip_prefix("pub(super) ").unwrap_or(decl);
                if let Some(name) = decl.strip_prefix("mod ").and_then(|d| d.strip_suffix(';')) {
                    let dir = path.parent().unwrap();
                    let stem = path.file_stem().unwrap();
                    tests.push(match &path_attr {
                        Some(relative) => dir.join(relative),
                        None if stem == "lib" || stem == "mod" => dir.join(format!("{name}.rs")),
                        None => dir.join(stem).join(format!("{name}.rs")),
                    });
                } else if decl.starts_with("mod ") && decl.ends_with('{') {
                    break;
                }
            }
            keep.push(line.to_string());
        }
        sources.insert(path, keep);
    }
    for test in tests {
        let test = fs::canonicalize(&test).unwrap_or(test);
        sources.retain(|path, _| fs::canonicalize(path).map_or(true, |p| p != test));
    }
    sources
}

fn measure(root: &Path) -> BTreeMap<String, usize> {
    let sources = production_sources(root);
    let count = |skip: Option<&str>, needles: &[&str]| -> usize {
        sources
            .iter()
            .filter(|(path, _)| skip.is_none_or(|skip| !path.ends_with(skip)))
            .flat_map(|(_, lines)| lines)
            .map(|line| {
                needles
                    .iter()
                    .map(|n| line.matches(n).count())
                    .sum::<usize>()
            })
            .sum()
    };
    let mut values = BTreeMap::new();
    for hub in HUBS {
        let lines = fs::read_to_string(root.join(hub)).unwrap().lines().count();
        values.insert(format!("hub:{hub}"), lines);
    }
    // Plain writes can leave a torn file after a crash; use `character_store::replace_bytes`.
    values.insert("plain_fs_write".into(), count(None, &["fs::write("]));
    // Hex goes through `trace::hex_prefix`, the one redaction rule.
    values.insert(
        "hex_outside_trace".into(),
        count(Some("trace.rs"), &[":02x}", ":02X}"]),
    );
    // Busy rules live in `gameplay/occupation.rs`; ask `is_occupied` or `blocks`.
    values.insert(
        "busy_outside_occupation".into(),
        count(Some("occupation.rs"), &BUSY_TERMS),
    );
    values
}

#[test]
fn spine_rules_only_ratchet_down() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(
        !manifest.contains("panic = \"abort\""),
        "the isolation boundary (`contain` in osfr-gateway.rs) needs panic = \"unwind\""
    );
    let file = root.join("tests/spine_rules.ceilings");
    let text = fs::read_to_string(&file).unwrap();
    let ceilings: BTreeMap<String, usize> = text
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let (key, value) = line.rsplit_once(' ').unwrap();
            (key.to_owned(), value.parse().unwrap())
        })
        .collect();
    let values = measure(root);
    assert_eq!(
        ceilings.keys().collect::<Vec<_>>(),
        values.keys().collect::<Vec<_>>(),
        "spine_rules.ceilings must list exactly the measured rules"
    );
    for (key, value) in &values {
        println!("spine {key} {value} (ceiling {})", ceilings[key]);
    }
    let over: Vec<_> = values
        .iter()
        .filter(|(key, value)| **value > ceilings[*key])
        .map(|(key, value)| format!("{key}: {value} > {}", ceilings[key]))
        .collect();
    assert!(
        over.is_empty(),
        "spine rules grew (move code into a module, use replace_bytes, trace::hex_prefix or \
         Gameplay::is_occupied): {over:?}"
    );
    if std::env::var_os("SPINE_RATCHET_LOWER").is_some() {
        let lowered = text
            .lines()
            .map(|line| match line.rsplit_once(' ') {
                Some((key, _)) if values.contains_key(key) => {
                    let slack = if key.starts_with("hub:") {
                        HUB_HEADROOM
                    } else {
                        0
                    };
                    format!("{key} {}", ceilings[key].min(values[key] + slack))
                }
                _ => line.to_owned(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file, lowered + "\n").unwrap();
    }
}
