//! Harvesting border fix: the emitted board pads index 0 on both axes.
//!
//! ORIGINAL evidence (`notes/Research/Harvesting Border Compatibility.md`): the original
//! fall loops never visit index 0. `ServerGrid.as`/`Grid.as::checkFalls` scan columns with
//! `while (_loc8_ > 0)` and rows with `while (_loc6_ > 0)`, and `addColumnFall` marks rows
//! with `while (_loc3_ > 0)`; row 0 and column 0 can therefore never move. The reconstructed
//! v1 shape exposed index 0 as playable, so the fix pads the grid (11x11: row 0 and column 0
//! empty, the same 10x9 playable rectangle at x1..10 / y1..9, drop row 10).
//!
//! RED baseline: the current spec is 10x10 with playable x0..9 / y0..8 and drop row 9.
//!
//! Fixtures write the real writer output into a private temp directory; no shared content,
//! save, process or UI file is touched.
// (The crate builds cleanly for the private Mining replay; this target is Harvest-only.)

use osfr_tools::{
    harvesting::{
        self, CHEF_PROFILE_ID, CONTENT_FILE, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE,
        FIRST_INSTANCE_ID, RECONSTRUCTED_COLLECTABLE_PERCENT, RECONSTRUCTED_DURATION_SECONDS,
        RECONSTRUCTED_POWERUP_PERCENT, RECONSTRUCTED_SEED, RECONSTRUCTED_WILDCARD_PERCENT,
    },
    mining::{Type8Engine, Type8Phase},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const GAME_ID: i32 = 111;
const PADDED: usize = 11;
const FIRST_PLAYABLE: usize = 1;
const LAST_PLAYABLE: usize = 10;
const DROP_ROW: usize = 10;
const PLAYABLE_CELLS: usize = 90;
const SHAPE_EMPTY: i32 = 0;
const SHAPE_REGULAR: i32 = 1;
const SHAPE_DROPZONE: i32 = 4;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-harvesting-border-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn write(&self) -> std::io::Result<PathBuf> {
        harvesting::write_content(&self.directory, GAME_ID).map_err(std::io::Error::other)
    }

    fn load(&self) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            CONTENT_FILE,
            GAME_ID,
            FIRST_INSTANCE_ID,
            None,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join(CONTENT_FILE));
        let _ = fs::remove_dir(&self.directory);
    }
}

/// The shared engine's `board_bounds` rule over the shape: playable cells are every code
/// that is neither empty nor a dropzone, the rectangle must be full, and the dropzone row
/// sits directly below it.
fn derived_bounds(shape: &[i32]) -> (usize, usize, usize, usize, usize) {
    let mut min_x = usize::MAX;
    let mut min_y = usize::MAX;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut drop_row = usize::MAX;
    for index in 0..shape.len() {
        let (x, y) = (index % PADDED, index / PADDED);
        match shape[index] {
            SHAPE_EMPTY => {}
            SHAPE_DROPZONE => drop_row = drop_row.min(y),
            _ => {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    (min_x, min_y, max_x, max_y, drop_row)
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut bytes = 39_u16.to_le_bytes().to_vec();
    bytes.push(sub);
    for number in [id, -1, game] {
        bytes.extend(number.to_le_bytes());
    }
    bytes
}

fn client_payload(id: i32, text: &str) -> Vec<u8> {
    let mut bytes = header(14, id, -1);
    bytes.extend(((text.len() + 1) as i32).to_le_bytes());
    bytes.extend(text.as_bytes());
    bytes.push(0);
    bytes
}

/// Drives the engine to `Playing` exactly like the recovered handshake.
fn start_level(engine: &mut Type8Engine) {
    let id = FIRST_INSTANCE_ID;
    engine.begin(CHEF_PROFILE_ID).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    engine
        .handle(&client_payload(id, "OnStartLevelMsg"))
        .unwrap();
}

#[test]
fn emitted_shape_pads_the_original_zero_edges() {
    let spec = harvesting::board_spec(GAME_ID).expect("region 111 spec builds");
    assert_eq!(
        (spec.width, spec.height),
        (PADDED as i32, PADDED as i32),
        "the emitted grid carries the padded border"
    );
    assert!(
        spec.shape[0..PADDED]
            .iter()
            .all(|code| *code == SHAPE_EMPTY),
        "row 0 stays inert because checkFalls never scans it"
    );
    for row in 0..PADDED {
        assert_eq!(
            spec.shape[row * PADDED],
            SHAPE_EMPTY,
            "column 0 row {row} stays inert because the fall loops never visit index 0"
        );
    }
    let mut playable = Vec::new();
    for row in FIRST_PLAYABLE..=9 {
        for column in FIRST_PLAYABLE..=LAST_PLAYABLE {
            playable.push(spec.shape[row * PADDED + column]);
        }
    }
    assert_eq!(playable.len(), PLAYABLE_CELLS);
    assert!(
        playable.iter().all(|code| *code == SHAPE_REGULAR),
        "the 10x9 playable rectangle is unchanged"
    );
    assert_eq!(
        &spec.shape[DROP_ROW * PADDED..],
        &[SHAPE_EMPTY, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
        "row 10 is the padded drop row over x1..10"
    );
    assert_eq!(
        derived_bounds(&spec.shape),
        (FIRST_PLAYABLE, FIRST_PLAYABLE, LAST_PLAYABLE, 9, DROP_ROW),
        "the engine derives the same 10x9 rectangle from the padded shape"
    );
}

#[test]
fn padded_content_starts_on_the_real_engine_with_inert_edges() {
    let fixture = Fixture::new();
    fixture.write().expect("writer succeeds");
    let mut engine = fixture.load().expect("the padded content loads");
    assert_eq!((engine.spec().width, engine.spec().height), (11, 11));
    start_level(&mut engine);
    let run = engine.state().active.as_ref().expect("the run is active");
    assert_eq!(run.phase, Type8Phase::Playing);
    for index in 0..PADDED {
        assert_eq!(run.board[index], 0, "generated row 0 cell {index} is empty");
        assert_eq!(
            run.board[index * PADDED],
            0,
            "generated column 0 cell {index} is empty"
        );
    }
    for row in FIRST_PLAYABLE..=9 {
        for column in FIRST_PLAYABLE..=LAST_PLAYABLE {
            assert_ne!(
                run.board[row * PADDED + column],
                0,
                "generated playable cell ({column},{row}) is populated"
            );
        }
    }
}

#[test]
fn padded_geometry_preserves_parameters_and_playable_cell_count() {
    let spec = harvesting::board_spec(GAME_ID).expect("region 111 spec builds");
    assert_eq!(spec.selectable, vec![411, 413, 416, 403]);
    assert!(spec.collectable.is_empty());
    assert_eq!(spec.tiles.len(), 4);
    assert_eq!(spec.goals.len(), 4);
    assert_eq!(spec.outputs.len(), 4);
    assert_eq!(spec.duration_seconds, RECONSTRUCTED_DURATION_SECONDS);
    assert_eq!(spec.seed, RECONSTRUCTED_SEED);
    assert_eq!(spec.wildcard_percent, RECONSTRUCTED_WILDCARD_PERCENT);
    assert_eq!(spec.collectable_percent, RECONSTRUCTED_COLLECTABLE_PERCENT);
    assert_eq!(spec.powerup_percent, RECONSTRUCTED_POWERUP_PERCENT);
    assert_eq!(spec.meter_id, 0);
    assert_eq!(
        spec.shape
            .iter()
            .filter(|code| **code == SHAPE_REGULAR)
            .count(),
        PLAYABLE_CELLS,
        "the padded grid keeps exactly the 90 reconstructed playable cells"
    );
    assert_eq!(
        spec.shape
            .iter()
            .filter(|code| **code == SHAPE_DROPZONE)
            .count(),
        10,
        "the drop row keeps its ten cells"
    );
    assert!(spec.reconstructed_rules);
    let notice = spec.rule_notice.to_lowercase();
    assert!(
        notice.contains("screenshot"),
        "the screenshot evidence stays cited"
    );
    assert!(
        notice.contains("fidelity gap"),
        "the staggered-board gap stays labelled"
    );

    // The content document keeps the recovered identity fields.
    let document = harvesting::content(GAME_ID).expect("content builds");
    assert_eq!(document.game_id, GAME_ID);
    assert_eq!(document.profile_id, CHEF_PROFILE_ID);
    assert_eq!(document.preload_file, FARMING_PRELOAD_FILE);
    assert_eq!(document.launch_movie, FARMING_LAUNCH_MOVIE);
}
