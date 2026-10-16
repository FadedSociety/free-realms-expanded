//! Mining `has_move` regression: wildcard false positives after a chain.
//!
//! Native run 143211 paused with 21 chains, every tile idle and one reshuffle already applied;
//! the client showed no legal path while the engine believed a move existed. Its snapshot
//! (`lab/runs/client-baseline-20261009T143211Z/character-chain21-no-move.json`, sha256
//! `e3885ebeaab33c9f15232f13a9e70333b4d8a8ccd24db8077b30d14927b1bd24`) has a wildcard 200 at
//! (3,3), a collectable 602 at (3,4) and a 403 at (4,3).
//!
//! The old `has_move` seeds a flood from the wildcard and treats *any* neighbour as a match
//! (`value == TILE_WILDCARD` short-circuits the value test), so the flood counts goals and
//! mixed-colour tiles and reports a move that does not exist. An exhaustive three-tile finder
//! over the snapshot returns zero legal paths.
//!
//! These vectors drive the real engine's post-chain path (`OnLevelEventMsg 50` -> collect ->
//! `gravity` -> `has_move`): a board with no legal three-tile path must reshuffle, a board with
//! one must not. `has_move` is a future-move check, so idle flags are ignored on purpose.
//!
//! Fixtures copy nothing shared: a private temp directory gets the standard `mining.json`, and
//! every state is either the frozen snapshot or a synthetic board built in this file.

use osfr_tools::mining::{
    self, FIRST_MINING_INSTANCE, MINING_GAME_ID, Type8Engine, Type8Phase, Type8Run, Type8State,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const SIDE: usize = 14;
const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const BOTTOM: usize = TOP + PLAYABLE - 1;
const TRIGGER_COLUMN: usize = 7;
const SNAPSHOT_SHA256: &str = "e3885ebeaab33c9f15232f13a9e70333b4d8a8ccd24db8077b30d14927b1bd24";

static NEXT: AtomicU64 = AtomicU64::new(0);

fn snapshot_path() -> PathBuf {
    std::env::var_os("OSFR_MINING_NO_MOVE_SNAPSHOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../../lab/runs/client-baseline-20261009T143211Z/character-chain21-no-move.json",
            )
        })
}

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-move-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        mining::write_content(&directory, MINING_GAME_ID).expect("mining content writes");
        Self { directory }
    }

    fn load(&self, state: Type8State) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            mining::CONTENT_FILE,
            MINING_GAME_ID,
            FIRST_MINING_INSTANCE,
            Some(state),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join(mining::CONTENT_FILE));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut bytes = 39_u16.to_le_bytes().to_vec();
    bytes.push(sub);
    for number in [id, -1, game] {
        bytes.extend(number.to_le_bytes());
    }
    bytes
}

/// Client `39/14` payload: current instance, two `-1` sentinels, length + text + NUL.
fn client_payload(id: i32, text: &str) -> Vec<u8> {
    let mut bytes = header(14, id, -1);
    bytes.extend(((text.len() + 1) as i32).to_le_bytes());
    bytes.extend(text.as_bytes());
    bytes.push(0);
    bytes
}

fn index(x: usize, y: usize) -> usize {
    y * SIDE + x
}

fn set(board: &mut [u16], x: usize, y: usize, value: u16) {
    board[index(x, y)] = value;
}

/// Playable area filled with a goal collectable so no accidental chain exists there.
fn background() -> Vec<u16> {
    let mut board = vec![0_u16; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            set(&mut board, x, y, 601);
        }
    }
    board
}

fn synthetic_state(board: Vec<u16>, idle: Vec<bool>, reshuffles: u32) -> Type8State {
    Type8State {
        version: 1,
        next_instance_id: FIRST_MINING_INSTANCE + 1,
        completed: 0,
        active: Some(Type8Run {
            instance_id: FIRST_MINING_INSTANCE,
            phase: Type8Phase::Playing,
            board,
            idle,
            tile_queue: vec![401; 64],
            random_queue: Vec::new(),
            rng: 1,
            collected: vec![0, 0],
            score: 0,
            remaining_ms: 30_000,
            wall_ms: 0,
            paused: true,
            chains: 1,
            reshuffles,
            end_sent: false,
        }),
    }
}

/// Sends the recovered fall event and returns the reshuffle counter afterwards.
fn fall_and_reshuffles(engine: &mut Type8Engine) -> u32 {
    let instance = engine
        .state()
        .active
        .as_ref()
        .expect("active run")
        .instance_id;
    engine
        .handle(&client_payload(
            instance,
            &format!("OnLevelEventMsg\t50\t{TRIGGER_COLUMN}\t{BOTTOM}"),
        ))
        .expect("fall event is accepted");
    engine.state().active.as_ref().unwrap().reshuffles
}

#[test]
#[ignore = "requires private native Mining snapshot"]
fn chain21_snapshot_with_no_legal_path_must_reshuffle() {
    let raw = fs::read(snapshot_path()).expect("the chain21 snapshot is readable");
    assert_eq!(
        format!("{:x}", Sha256::digest(&raw)),
        SNAPSHOT_SHA256,
        "the frozen chain21 snapshot changed"
    );
    let document: Value = serde_json::from_slice(&raw).unwrap();
    let mut state: Type8State = serde_json::from_value(document["gameplay"]["mining"].clone())
        .expect("mining state parses");
    let run = state
        .active
        .as_mut()
        .expect("the snapshot has an active run");
    assert!(run.idle.iter().all(|flag| *flag), "every tile is idle");
    assert_eq!(run.reshuffles, 1, "the snapshot already reshuffled once");
    // Trigger reachability: the engine only collects a goal collectable from the bottom row, and
    // column 7 still has no three-tile path after the refill.
    set(&mut run.board, TRIGGER_COLUMN, BOTTOM, 601);
    run.tile_queue = vec![401; 64];

    let fixture = Fixture::new();
    let mut engine = fixture.load(state).expect("snapshot state loads");
    assert_eq!(
        fall_and_reshuffles(&mut engine),
        2,
        "a board with no legal three-tile path must reshuffle"
    );
}

#[test]
fn wildcard_star_between_two_colours_has_no_move() {
    let mut board = background();
    set(&mut board, 3, 3, 200);
    set(&mut board, 3, 4, 401);
    set(&mut board, 4, 3, 402);
    let fixture = Fixture::new();
    let mut engine = fixture
        .load(synthetic_state(board, vec![true; SIDE * SIDE], 1))
        .unwrap();
    assert_eq!(fall_and_reshuffles(&mut engine), 2);
}

#[test]
fn wildcard_next_to_a_goal_has_no_move() {
    let mut board = background();
    set(&mut board, 3, 3, 200);
    set(&mut board, 3, 4, 602);
    set(&mut board, 4, 3, 403);
    let fixture = Fixture::new();
    let mut engine = fixture
        .load(synthetic_state(board, vec![true; SIDE * SIDE], 1))
        .unwrap();
    assert_eq!(fall_and_reshuffles(&mut engine), 2);
}

#[test]
fn valid_three_tile_paths_keep_the_board_and_ignore_idle() {
    let cases: [(&str, [(usize, usize, u16); 3]); 4] = [
        ("straight", [(5, 4, 401), (5, 5, 401), (5, 6, 401)]),
        ("bent", [(5, 4, 402), (5, 5, 402), (6, 5, 402)]),
        ("wildcard bridge", [(5, 4, 401), (5, 5, 200), (5, 6, 401)]),
        ("pure wildcard", [(5, 4, 200), (5, 5, 200), (5, 6, 200)]),
    ];
    for (name, cells) in cases {
        let mut board = background();
        let mut idle = vec![true; SIDE * SIDE];
        for (x, y, value) in cells {
            set(&mut board, x, y, value);
            idle[index(x, y)] = false; // future moves ignore idle
        }
        let fixture = Fixture::new();
        let mut engine = fixture
            .load(synthetic_state(board, idle, 1))
            .unwrap_or_else(|error| panic!("{name}: state loads: {error}"));
        assert_eq!(
            fall_and_reshuffles(&mut engine),
            1,
            "{name}: a legal three-tile path must not reshuffle"
        );
    }
}
