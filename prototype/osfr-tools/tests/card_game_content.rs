//! T504: `data/content/card_game.json`, the card-game portal rows, against the client files they
//! were taken from, and the loader's refusals.
use osfr_tools::card_game::{Content, TREASURE_WAR_CATEGORY};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shipped_path() -> PathBuf {
    root().join("data/content/card_game.json")
}

fn client(file: &str) -> String {
    let path = root()
        .join("../../../lab/working/OSFR.Client/OSFR Client")
        .join(file);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `MiniGameData.txt` rows by id: `[TYPE, NAME_ID, DESCRIPTION_ID, ICON_ID, DIFFICULTY]`.
fn minigames() -> std::collections::HashMap<u32, [i64; 5]> {
    client("Resources/MiniGameData.txt")
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('^').collect();
            let id = cells.first()?.parse().ok()?;
            let n = |i: usize| cells[i].parse::<i64>().unwrap();
            Some((id, [n(1), n(2), n(3), n(4), n(5)]))
        })
        .collect()
}

/// Write `value` (plus `padding` trailing spaces) as a content file in a fresh temp directory and load it.
fn load_padded(value: &Value, padding: usize) -> std::io::Result<Content> {
    let dir = std::env::temp_dir().join(format!(
        "card-game-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("card_game.json");
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.resize(bytes.len() + padding, b' ');
    fs::write(&path, bytes).unwrap();
    let loaded = Content::load(&path);
    let _ = fs::remove_dir_all(&dir);
    loaded
}

fn load(value: &Value) -> std::io::Result<Content> {
    load_padded(value, 0)
}

fn shipped_value() -> Value {
    serde_json::from_str(&fs::read_to_string(shipped_path()).unwrap()).unwrap()
}

#[test]
fn the_shipped_file_loads_with_the_seventeen_rows() {
    let content = Content::load(&shipped_path()).unwrap();
    assert_eq!(content.rows().len(), 17);
    let ids: Vec<u32> = content.rows().iter().map(|row| row.id).collect();
    // The lobby, the 14 decks, the tutorial (the client's Tcg.ActivityId default 00c04970) and
    // Treasure War (Scg.ActivityId default 00c049f0). Garrison's 493 and 501 stay out.
    let mut expected = vec![7, 490, 491, 492, 494, 495, 496, 497, 498, 499, 500];
    expected.extend([502, 503, 504, 505, 1049, 1078]);
    assert_eq!(ids, expected);
    assert!(content.rows().iter().all(|row| row.can_player_join));
    assert_eq!(
        content
            .rows()
            .iter()
            .filter(|row| row.category == TREASURE_WAR_CATEGORY)
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [1078]
    );
}

#[test]
fn every_row_equals_its_client_minigame_row() {
    let minigames = minigames();
    for row in Content::load(&shipped_path()).unwrap().rows() {
        let [kind, name, description, _icon, difficulty] = *minigames
            .get(&row.minigame_id)
            .unwrap_or_else(|| panic!("row {}: MiniGameData has no {}", row.id, row.minigame_id));
        let expected_kind = if row.category == TREASURE_WAR_CATEGORY {
            23
        } else {
            16
        };
        assert_eq!(kind, expected_kind, "row {} TYPE", row.id);
        assert_eq!(i64::from(row.name_id), name, "row {} NAME_ID", row.id);
        assert_eq!(
            i64::from(row.description_id),
            description,
            "row {} DESCRIPTION_ID",
            row.id
        );
        assert_eq!(
            i64::from(row.difficulty),
            difficulty,
            "row {} DIFFICULTY",
            row.id
        );
    }
    // The two categories exist in the client table with the names the rows name.
    let categories = client("Resources/ActivityCategories.txt");
    for (id, name_id) in [(11, 3388), (26, 629)] {
        assert!(
            categories
                .lines()
                .any(|l| l.starts_with(&format!("{id}^36^{name_id}^"))),
            "category {id}"
        );
    }
}

#[test]
fn every_image_is_in_the_client_asset_manifest() {
    let manifest: HashSet<String> = client("Assets_manifest.txt")
        .lines()
        .filter_map(|line| line.split(',').next())
        .map(|name| name.trim_end_matches(".z").to_ascii_lowercase())
        .collect();
    for row in Content::load(&shipped_path()).unwrap().rows() {
        for name in [&row.detail_image, &row.thumbnail_image] {
            assert!(
                manifest.contains(&name.to_ascii_lowercase()),
                "row {}: {name} is not in Assets_manifest.txt",
                row.id
            );
        }
    }
}

#[test]
fn every_row_carries_legacy_and_client_provenance() {
    for row in Content::load(&shipped_path()).unwrap().rows() {
        let kinds: Vec<&str> = row.source.iter().map(|s| s.kind.as_str()).collect();
        assert!(kinds.contains(&"legacy-data"), "row {}", row.id);
        assert!(kinds.contains(&"client"), "row {}", row.id);
        // image_set_id 7591 is reconstructed (R); Treasure War ships none.
        assert_eq!(
            kinds.contains(&"reconstructed"),
            row.image_set_id != 0,
            "row {}",
            row.id
        );
    }
}

#[test]
fn malformed_files_are_refused() {
    assert!(
        load(&shipped_value()).is_ok(),
        "control: the shipped file loads"
    );
    let broken = |edit: &dyn Fn(&mut Value)| {
        let mut value = shipped_value();
        edit(&mut value);
        load(&value)
    };
    assert!(
        broken(&|v| v["rows"][1]["id"] = json!(7)).is_err(),
        "duplicate id"
    );
    assert!(
        broken(&|v| v["rows"][1]["id"] = json!(0)).is_err(),
        "zero id"
    );
    assert!(
        broken(&|v| v["rows"][1]["tutorial_activity_id"] = json!(1050)).is_err(),
        "unknown tutorial id"
    );
    assert!(
        broken(&|v| v["rows"][1]["tutorial_activity_id"] = json!(0)).is_err(),
        "zero tutorial id"
    );
    assert!(
        broken(&|v| v["rows"][1]["surprise"] = json!(1)).is_err(),
        "unknown field"
    );
    assert!(
        broken(&|v| v["rows"][1]["category"] = json!(18)).is_err(),
        "foreign category"
    );
    assert!(
        broken(&|v| v["rows"][1]["detail_image"] = json!("")).is_err(),
        "empty image name"
    );
    assert!(
        broken(&|v| v["rows"][1]["source"] = json!([])).is_err(),
        "no provenance"
    );
    assert!(broken(&|v| v["rows"] = json!([])).is_err(), "no rows");
    assert!(
        broken(&|v| v["schema_version"] = json!(2)).is_err(),
        "version"
    );
    assert!(
        load_padded(&shipped_value(), 1 << 20).is_err(),
        "oversize file"
    );
    assert!(Content::load(Path::new("no-such-card-game.json")).is_err());
}
