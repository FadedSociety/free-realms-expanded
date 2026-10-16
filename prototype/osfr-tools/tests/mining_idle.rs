//! Mining idle-acknowledgement regression: `OnIdleMsg` accepts any non-empty
//! coordinate list.
//!
//! Recovered wiring:
//! - `core/Grid.UpdateIdle` (1356-1363) sends the pending `sNowIdle` string as
//!   soon as it is non-empty;
//! - `GameClientNetwork.SendIdle` (26-29) wraps it as `OnIdleMsg`;
//! - `GameLevel.OnIdleMsg` (543-552) splits on `,`, pops the trailing empty
//!   field and forwards every remaining pair to `ServerGrid.NowIdle`;
//! - `ServerGrid.NowIdle` (78-102) loops `length / 2` entries with no minimum.
//!
//! A single fired tile (`"4,4,"`) is therefore a complete, legal message; only
//! the chain path has the recovered three-tile minimum.
//!
//! Fixtures are synthetic: the content file comes from the production
//! `write_content` and every state is built in this file. No save, native client
//! or gateway state is read.

use osfr_tools::mining::{
    CONTENT_FILE, FIRST_MINING_INSTANCE, MINING_GAME_ID, SIDE, TILE_SELECTABLE_START, Type8Engine,
    Type8Phase, Type8Run, Type8State, write_content,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const BACKGROUND: u16 = 601;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-idle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        write_content(&directory, MINING_GAME_ID).expect("mining content writes");
        Self { directory }
    }

    fn load(&self, state: Type8State) -> Type8Engine {
        Type8Engine::load(
            &self.directory,
            CONTENT_FILE,
            MINING_GAME_ID,
            FIRST_MINING_INSTANCE,
            Some(state),
        )
        .expect("the synthetic state loads")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join(CONTENT_FILE));
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

/// Playable rectangle filled with one goal collectable so no accidental chain
/// exists; the dropzone row stays as the content writes it.
fn background() -> Vec<u16> {
    let mut board = vec![0_u16; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            board[index(x, y)] = BACKGROUND;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        board[index(x, TOP + PLAYABLE)] = 100;
    }
    board
}

fn state_with(
    board: Vec<u16>,
    idle_marks: &[(usize, usize)],
    phase: Type8Phase,
    paused: bool,
) -> Type8State {
    let mut idle = vec![true; SIDE * SIDE];
    for &(x, y) in idle_marks {
        idle[index(x, y)] = false;
    }
    Type8State {
        version: 1,
        next_instance_id: FIRST_MINING_INSTANCE + 1,
        completed: u32::from(matches!(phase, Type8Phase::Won)),
        active: Some(Type8Run {
            instance_id: FIRST_MINING_INSTANCE,
            phase,
            board,
            idle,
            tile_queue: vec![TILE_SELECTABLE_START; 64],
            random_queue: Vec::new(),
            rng: 1,
            collected: vec![0, 0],
            score: 0,
            remaining_ms: 30_000,
            wall_ms: 0,
            paused,
            chains: 0,
            reshuffles: 0,
            end_sent: false,
        }),
    }
}

fn idle_at(engine: &Type8Engine, x: usize, y: usize) -> bool {
    engine.state().active.as_ref().unwrap().idle[index(x, y)]
}

#[test]
fn single_tile_idle_ack_is_accepted() {
    // RED before the fix: the shared chain parser demanded three pairs.
    let fixture = Fixture::new();
    let mut engine = fixture.load(state_with(
        background(),
        &[(4, 4)],
        Type8Phase::Playing,
        false,
    ));
    assert!(!idle_at(&engine, 4, 4), "fixture starts busy");
    engine
        .handle(&client_payload(FIRST_MINING_INSTANCE, "OnIdleMsg\t4,4,"))
        .expect("a single fired tile is a complete idle acknowledgement");
    assert!(idle_at(&engine, 4, 4));
}

#[test]
fn two_tile_idle_ack_is_accepted() {
    let fixture = Fixture::new();
    let mut engine = fixture.load(state_with(
        background(),
        &[(4, 4), (5, 4)],
        Type8Phase::Playing,
        false,
    ));
    engine
        .handle(&client_payload(
            FIRST_MINING_INSTANCE,
            "OnIdleMsg\t4,4,5,4,",
        ))
        .expect("two fired tiles are a complete idle acknowledgement");
    assert!(idle_at(&engine, 4, 4));
    assert!(idle_at(&engine, 5, 4));
}

#[test]
fn chain_still_requires_three_tiles() {
    // The three-tile minimum belongs to the chain path only.
    let fixture = Fixture::new();
    let mut board = background();
    for x in [4, 5, 6] {
        board[index(x, 4)] = TILE_SELECTABLE_START;
    }
    let mut engine = fixture.load(state_with(board, &[], Type8Phase::Playing, false));
    let before = engine.state().clone();
    assert!(
        engine
            .handle(&client_payload(
                FIRST_MINING_INSTANCE,
                "OnChainMsg\t4,4,5,4,"
            ))
            .is_err(),
        "a two-tile chain must stay refused"
    );
    assert!(
        engine
            .handle(&client_payload(FIRST_MINING_INSTANCE, "OnChainMsg\t4,4,"))
            .is_err(),
        "a one-tile chain must stay refused"
    );
    assert_eq!(engine.state(), &before);
    // The legal three-tile chain is still accepted.
    engine
        .handle(&client_payload(
            FIRST_MINING_INSTANCE,
            "OnChainMsg\t4,4,5,4,6,4,",
        ))
        .expect("the recovered three-tile chain still works");
}

#[test]
fn idle_ack_outside_the_board_is_refused_atomically() {
    let fixture = Fixture::new();
    let mut engine = fixture.load(state_with(
        background(),
        &[(4, 4)],
        Type8Phase::Playing,
        false,
    ));
    let before = engine.state().clone();
    assert!(
        engine
            .handle(&client_payload(
                FIRST_MINING_INSTANCE,
                "OnIdleMsg\t0,0,4,4,"
            ))
            .is_err(),
        "an out-of-board coordinate must be refused"
    );
    assert_eq!(engine.state(), &before, "refusal must not mark any tile");
}

#[test]
fn malformed_idle_lists_are_refused_and_change_nothing() {
    let fixture = Fixture::new();
    let mut engine = fixture.load(state_with(
        background(),
        &[(4, 4)],
        Type8Phase::Playing,
        false,
    ));
    let before = engine.state().clone();
    for malformed in [
        "OnIdleMsg\t4,4,5,",    // odd field count
        "OnIdleMsg\t4,4,,5,4,", // empty field
        "OnIdleMsg\t9999,4,",   // coordinate above the recovered bound
        "OnIdleMsg\tx,4,",      // non-numeric
        "OnIdleMsg\t,",         // no coordinate at all
    ] {
        assert!(
            engine
                .handle(&client_payload(FIRST_MINING_INSTANCE, malformed))
                .is_err(),
            "{malformed} must be refused"
        );
        assert_eq!(
            engine.state(),
            &before,
            "{malformed} must not mutate the run"
        );
    }
}

#[test]
fn idle_acks_keep_their_non_playing_behaviour() {
    let fixture = Fixture::new();
    // Terminal run: the message parses and is accepted, but no tile changes.
    let mut engine = fixture.load(state_with(background(), &[(4, 4)], Type8Phase::Won, false));
    let before = engine.state().clone();
    engine
        .handle(&client_payload(FIRST_MINING_INSTANCE, "OnIdleMsg\t4,4,"))
        .expect("a terminal run still accepts a well-formed idle request");
    assert_eq!(engine.state(), &before);
    assert!(
        engine
            .handle(&client_payload(FIRST_MINING_INSTANCE, "OnIdleMsg\t4,4,,"))
            .is_err(),
        "malformed input is refused in every phase"
    );
    assert_eq!(engine.state(), &before);

    // Paused play keeps applying idle acknowledgements, as before the fix.
    let mut engine = fixture.load(state_with(
        background(),
        &[(4, 4)],
        Type8Phase::Playing,
        true,
    ));
    engine
        .handle(&client_payload(FIRST_MINING_INSTANCE, "OnIdleMsg\t4,4,"))
        .expect("a paused level still acknowledges idle tiles");
    assert!(idle_at(&engine, 4, 4));
}
