//! T491 (R): a content row may name a `client_areas.json` season; `NpcVisibility` drops it when
//! that season is over. T468 retired the 12 Halloween decoration rows to
//! `npc_spawns_retired.json` as out-of-bounds housing-demo props, and `world.rs` never reads that
//! archive, so production draws none of them today. These tests copy the real live table and add
//! those archived rows, so the rule is proven on the shipped 2,675 rows plus the 12 candidates.
//! No test reads today's date: every clock below is fixed.
use osfr_tools::{gameplay::Gameplay, world::NpcVisibility};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[allow(dead_code)] // only `copy_missing` is used here
#[path = "common/content_copy.rs"]
mod content_copy;

/// 2026-10-31 12:00 UTC, inside Halloween (9/20 to 11/11), and 2026-01-15 12:00 UTC, outside.
const IN_SEASON_MS: u64 = 1_793_448_000_000;
const OFF_SEASON_MS: u64 = 1_768_478_400_000;
const CAKE: u64 = 100_000_012_182;
/// A content key no live row uses, put on the cake to prove the load-side map follows the row.
const CAKE_KEY: u64 = 491_000_001;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn bundled() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The opening package supplies the opening, inventory and quest files `Gameplay::load` needs;
/// the bundled content then fills in the world, NPC and area tables.
fn opening() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

fn rows_of(path: &Path) -> Vec<Value> {
    serde_json::from_slice::<Value>(&fs::read(path).unwrap()).unwrap()["records"]
        .as_array()
        .unwrap()
        .clone()
}

fn guid(row: &Value) -> u64 {
    row["spawn_guid"].as_u64().unwrap()
}

fn live_rows() -> Vec<Value> {
    rows_of(&bundled().join("npc_spawns_extra.json"))
}

/// T468's archive: the 12 shipped Halloween decoration rows, sorted by GUID.
fn archived_halloween_rows() -> Vec<Value> {
    let mut rows: Vec<Value> = rows_of(&bundled().join("npc_spawns_retired.json"))
        .into_iter()
        .filter(|row| {
            row["model_file_name"]
                .as_str()
                .is_some_and(|name| name.to_ascii_lowercase().contains("halloween"))
        })
        .collect();
    rows.sort_by_key(guid);
    rows
}

/// The 12 shipped rows, tagged, unique, and absent from the live table (T468).
fn tagged_rows(live: &[Value], archived: &[Value]) -> BTreeSet<u64> {
    assert_eq!(
        archived.len(),
        12,
        "the archive holds the 12 Halloween decoration rows"
    );
    assert!(
        live.iter().all(|row| row.get("season").is_none()),
        "the live table carries no season tag: production draws none of these props (T468)"
    );
    let tagged: BTreeSet<u64> = archived.iter().map(guid).collect();
    assert_eq!(tagged.len(), 12, "a Halloween spawn GUID repeats");
    assert!(
        archived
            .iter()
            .all(|row| row["season"].as_str() == Some("Halloween")),
        "every archived row names the Halloween season"
    );
    let live_guids: BTreeSet<u64> = live.iter().map(guid).collect();
    assert!(
        tagged.is_disjoint(&live_guids),
        "an archived row is live again; T468's exit pin would fail too"
    );
    tagged
}

fn cake_key(archived: &mut [Value]) {
    archived.iter_mut().find(|row| guid(row) == CAKE).unwrap()["content_key"] = json!(CAKE_KEY);
}

fn present(visibility: &NpcVisibility, guids: &BTreeSet<u64>) -> BTreeSet<u64> {
    guids
        .iter()
        .copied()
        .filter(|guid| visibility.actor_position(*guid).is_some())
        .collect()
}

/// The production-shaped content tree with `live` plus `archived` in the live NPC table.
struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new(tag: &str, live: &[Value], archived: &[Value]) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "f683-t491-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        content_copy::copy_missing(&opening(), &dir);
        content_copy::copy_missing(&bundled(), &dir);
        // This suite exercises NPC visibility, not harvesting; the opening inventory does not
        // carry every optional farming reward the bundled content defines (same rule as the
        // soccer and field-combat fixtures).
        let _ = fs::remove_file(dir.join("harvesting.json"));
        let fixture = Self { dir };
        let mut rows = live.to_vec();
        rows.extend(archived.iter().cloned());
        fixture.write_rows(&rows);
        fixture
    }

    fn path(&self) -> &Path {
        &self.dir
    }

    fn write_rows(&self, rows: &[Value]) {
        fs::write(
            self.dir.join("npc_spawns_extra.json"),
            serde_json::to_vec(&json!({"schema_version": 1, "records": rows})).unwrap(),
        )
        .unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn the_twelve_halloween_rows_are_tagged_and_leave_only_when_the_season_is_over() {
    let live = live_rows();
    let mut archived = archived_halloween_rows();
    let tagged = tagged_rows(&live, &archived);
    cake_key(&mut archived);
    let guids: BTreeSet<u64> = live.iter().chain(&archived).map(guid).collect();

    let fixture = Fixture::new("retain", &live, &archived);
    let loaded = NpcVisibility::load(fixture.path()).unwrap();
    let shown = present(&loaded, &guids);
    assert!(
        tagged.is_subset(&shown),
        "every archived row loads before the filter"
    );

    let mut active = loaded.clone();
    active.retain_in_season(|_| Some(true)).unwrap();
    assert_eq!(
        present(&active, &guids),
        shown,
        "an active season keeps every row"
    );

    let mut over = loaded.clone();
    over.retain_in_season(|_| Some(false)).unwrap();
    let dropped: BTreeSet<u64> = shown.difference(&present(&over, &guids)).copied().collect();
    assert_eq!(
        dropped, tagged,
        "the over season drops exactly the tagged rows and leaves every live row"
    );
    assert_eq!(active.content_key(CAKE), Some(CAKE_KEY));
    assert_eq!(over.content_key(CAKE), None);
    let (control, key) = live
        .iter()
        .find_map(|row| {
            let guid = guid(row);
            let key = row["content_key"].as_u64()?;
            loaded.actor_position(guid).is_some().then_some((guid, key))
        })
        .expect("the live table has a content-keyed row");
    assert_eq!(active.content_key(control), Some(key));
    assert_eq!(over.content_key(control), Some(key));
}

#[test]
fn an_unknown_season_fails_naming_the_row_and_drops_nothing() {
    let live = live_rows();
    let archived = archived_halloween_rows();
    let first = guid(&archived[0]);
    let guids: BTreeSet<u64> = live.iter().chain(&archived).map(guid).collect();
    let fixture = Fixture::new("unknown", &live, &archived);
    let mut visibility = NpcVisibility::load(fixture.path()).unwrap();
    let before = present(&visibility, &guids);
    let error = visibility.retain_in_season(|_| None).unwrap_err();
    assert!(error.to_string().contains(&first.to_string()), "{error}");
    assert_eq!(
        present(&visibility, &guids),
        before,
        "the refused season dropped a row"
    );
}

#[test]
fn loading_for_play_follows_the_shipped_season_dates() {
    let live = live_rows();
    let mut archived = archived_halloween_rows();
    let tagged = tagged_rows(&live, &archived);
    cake_key(&mut archived);
    let guids: BTreeSet<u64> = live.iter().chain(&archived).map(guid).collect();
    let fixture = Fixture::new("dates", &live, &archived);

    let loaded = NpcVisibility::load(fixture.path()).unwrap();
    let in_season = NpcVisibility::load_in_season_at(fixture.path(), IN_SEASON_MS).unwrap();
    assert_eq!(in_season.model_of(CAKE), Some(3202));
    assert_eq!(in_season.content_key(CAKE), Some(CAKE_KEY));
    assert_eq!(
        present(&in_season, &guids),
        present(&loaded, &guids),
        "the shipped dates keep every row inside Halloween"
    );

    let off_season = NpcVisibility::load_in_season_at(fixture.path(), OFF_SEASON_MS).unwrap();
    assert!(off_season.model_of(CAKE).is_none());
    assert_eq!(off_season.content_key(CAKE), None);
    let dropped: BTreeSet<u64> = present(&loaded, &guids)
        .difference(&present(&off_season, &guids))
        .copied()
        .collect();
    assert_eq!(
        dropped, tagged,
        "outside the shipped dates exactly the tagged rows go"
    );

    // The production caller loads the same tree (either side of the season); the undefined-season
    // test below proves it applies the rule instead of `NpcVisibility::load`.
    Gameplay::load(fixture.path(), None).expect("the shipped content loads with seasonal rows");
}

#[test]
fn a_row_naming_a_season_the_content_lacks_fails_the_load() {
    let live = live_rows();
    let mut archived = archived_halloween_rows();
    let first = guid(&archived[0]);
    let guids: BTreeSet<u64> = live.iter().chain(&archived).map(guid).collect();
    let fixture = Fixture::new("typo", &live, &archived);
    let loaded = present(&NpcVisibility::load(fixture.path()).unwrap(), &guids);
    Gameplay::load(fixture.path(), None).expect("the production caller loads a defined season");

    // A season name `client_areas.json` does not define fails the fixed-clock load and the
    // production caller, both naming the row. `NpcVisibility::load` alone ignores the tag, so a
    // call site reverted to `load` would accept this table.
    archived[0]["season"] = json!("Halloweeen");
    let mut rows = live;
    rows.extend(archived);
    fixture.write_rows(&rows);
    let error = NpcVisibility::load_in_season_at(fixture.path(), IN_SEASON_MS)
        .err()
        .unwrap();
    assert!(error.to_string().contains(&first.to_string()), "{error}");
    let error = Gameplay::load(fixture.path(), None)
        .err()
        .expect("the production caller must apply the season rule");
    assert!(error.to_string().contains(&first.to_string()), "{error}");
    assert_eq!(
        present(&NpcVisibility::load(fixture.path()).unwrap(), &guids),
        loaded,
        "`load` stays season independent: the typo drops no row"
    );
}
