//! T491 (R): a content row may name a `client_areas.json` season; `NpcVisibility` drops it when
//! that season is over. The 12 Halloween decoration rows were retired by T468 as out-of-bounds
//! housing-demo props, so they are tagged in `npc_spawns_retired.json` (the file that still holds
//! them, and which `world.rs` never reads); this test rebuilds the production extra-spawn table
//! from those exact rows. No test reads today's date: every clock below is fixed.
use osfr_tools::world::NpcVisibility;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

/// 2026-10-31 12:00 UTC, inside Halloween (9/20 to 11/11), and 2026-01-15 12:00 UTC, outside.
const IN_SEASON_MS: u64 = 1_793_448_000_000;
const OFF_SEASON_MS: u64 = 1_768_478_400_000;
const CAKE: u64 = 100_000_012_182;
/// A row with no season: every filter below must keep it.
const CONTROL: u64 = 424_242;

fn content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn rows_of(file: &str) -> Vec<Value> {
    let bytes = fs::read(content().join(file)).unwrap();
    serde_json::from_slice::<Value>(&bytes).unwrap()["records"]
        .as_array()
        .unwrap()
        .clone()
}

fn guid(row: &Value) -> u64 {
    row["spawn_guid"].as_u64().unwrap()
}

/// The shipped Halloween decoration rows, from whichever table still carries them.
fn halloween_rows() -> Vec<Value> {
    let mut rows: Vec<Value> = ["npc_spawns_extra.json", "npc_spawns_retired.json"]
        .into_iter()
        .flat_map(rows_of)
        .filter(|row| {
            row["model_file_name"]
                .as_str()
                .is_some_and(|name| name.to_ascii_lowercase().contains("halloween"))
        })
        .collect();
    rows.sort_by_key(guid);
    rows
}

fn all_guids(rows: &[Value]) -> BTreeSet<u64> {
    rows.iter().map(guid).chain([CONTROL]).collect()
}

fn shown(visibility: &NpcVisibility, guids: &BTreeSet<u64>) -> BTreeSet<u64> {
    guids
        .iter()
        .copied()
        .filter(|guid| visibility.model_of(*guid).is_some())
        .collect()
}

/// The shipped content plus the Halloween rows, in a scratch directory.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(tag: &str, rows: &[Value]) -> Self {
        let dir = std::env::temp_dir().join(format!("osfr-t491-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("npcs")).unwrap();
        let write = |name: &str, value: Value| {
            fs::write(dir.join(name), serde_json::to_vec(&value).unwrap()).unwrap()
        };
        write(
            "zones.json",
            json!({"schema_version": 1, "records": [{"id": 0, "name": "Z"}]}),
        );
        write(
            "npcs/anchor.json",
            json!({"schema_version": 1, "records": [{
                "npc_id": 1,
                "name": "Anchor",
                "zone_id": 0,
                "position": [0.0, 0.0, 0.0],
                "placement": "legacy_data",
                "model_id": 1
            }]}),
        );
        let mut records = rows.to_vec();
        records.push(json!({"spawn_guid": CONTROL, "model_id": 7, "position": [1.0, 0.0, 1.0]}));
        write(
            "npc_spawns_extra.json",
            json!({"schema_version": 1, "records": records}),
        );
        fs::copy(
            content().join("client_areas.json"),
            dir.join("client_areas.json"),
        )
        .unwrap();
        Self { dir }
    }

    fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn the_twelve_halloween_rows_are_tagged_and_leave_only_when_the_season_is_over() {
    let rows = halloween_rows();
    assert_eq!(rows.len(), 12, "expected the 12 Halloween decoration rows");
    let tagged: BTreeSet<u64> = rows.iter().map(guid).collect();
    assert_eq!(tagged.len(), 12, "a Halloween spawn guid repeats");
    for row in &rows {
        assert_eq!(
            row["season"].as_str(),
            Some("Halloween"),
            "spawn {} is not tagged",
            guid(row)
        );
    }

    let fixture = Fixture::new("season", &rows);
    let all = all_guids(&rows);
    let loaded = NpcVisibility::load(fixture.path()).unwrap();
    assert_eq!(shown(&loaded, &all), all, "load keeps every row");

    let mut active = loaded.clone();
    active.retain_in_season(|_| Some(true)).unwrap();
    assert_eq!(shown(&active, &all), all);

    let mut over = loaded.clone();
    over.retain_in_season(|_| Some(false)).unwrap();
    assert_eq!(shown(&over, &all), BTreeSet::from([CONTROL]));
    assert!(over.model_of(CAKE).is_none());
}

#[test]
fn an_unknown_season_fails_naming_the_row_and_drops_nothing() {
    let rows = halloween_rows();
    let fixture = Fixture::new("unknown", &rows);
    let all = all_guids(&rows);
    let mut visibility = NpcVisibility::load(fixture.path()).unwrap();
    let before = shown(&visibility, &all);
    let first = guid(&rows[0]);
    let error = visibility.retain_in_season(|_| None).unwrap_err();
    assert!(error.to_string().contains(&first.to_string()), "{error}");
    assert_eq!(shown(&visibility, &all), before);
}

#[test]
fn loading_for_play_follows_the_shipped_season_dates() {
    let rows = halloween_rows();
    let tagged: BTreeSet<u64> = rows.iter().map(guid).collect();
    let all = all_guids(&rows);
    let fixture = Fixture::new("dates", &rows);

    let in_season = NpcVisibility::load_in_season_at(fixture.path(), IN_SEASON_MS).unwrap();
    assert_eq!(in_season.model_of(CAKE), Some(3202));
    assert_eq!(shown(&in_season, &all), all);

    let off_season = NpcVisibility::load_in_season_at(fixture.path(), OFF_SEASON_MS).unwrap();
    assert!(off_season.model_of(CAKE).is_none());
    assert_eq!(shown(&off_season, &all), BTreeSet::from([CONTROL]));
    assert_eq!(
        shown(&in_season, &all)
            .difference(&shown(&off_season, &all))
            .copied()
            .collect::<BTreeSet<_>>(),
        tagged
    );
}

#[test]
fn a_row_naming_a_season_the_content_lacks_fails_the_load() {
    let mut rows = halloween_rows();
    rows[0]["season"] = json!("Halloweeen");
    let fixture = Fixture::new("typo", &rows);
    let first = guid(&rows[0]);
    let error = NpcVisibility::load_in_season_at(fixture.path(), IN_SEASON_MS)
        .err()
        .unwrap();
    assert!(error.to_string().contains(&first.to_string()), "{error}");
    // `load` stays date independent: the same rows load unchanged without the season rule.
    assert_eq!(
        shown(
            &NpcVisibility::load(fixture.path()).unwrap(),
            &all_guids(&rows)
        )
        .len(),
        13
    );
}
