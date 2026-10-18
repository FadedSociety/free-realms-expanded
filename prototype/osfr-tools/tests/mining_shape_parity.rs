//! Shape parity regression: original `core/ServerGrid.as` 357-361 maps shape
//! `1` to `TILE_BUILDREGULAR` and shape `2` to `TILE_BUILDANY`;
//! `core/ServerTileGenerator.as` 107-110, 126-137 builds the regular pool from
//! selectable/hazard/wildcard only, while the any pool includes collectables.
//! Rust `initialize_board` must therefore draw shape `1` cells with the
//! collectable-disabled pool and keep shape `2` plus the gravity refill queues
//! collectable-enabled.
//!
//! Every fixture is synthetic and private: no user save, native client, gateway state or shared content is read or written.
use osfr_tools::mining::{
    self, MINING_GAME_ID, SIDE, Type8BoardSpec, Type8Engine, Type8Goal, Type8Outcome, Type8Phase,
    Type8Run, Type8State,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const PROFILE: i32 = 14;
const FIRST_INSTANCE: i32 = 18_000_001;
const INSTANCE: i32 = 18_000_002;
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;
const CHAIN_MESSAGE: &str = "OnChainMsg\t3,3,4,3,5,3,";
const BUILD_REGULAR: i32 = 1;
const BUILD_ANY: i32 = 2;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(spec: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-shape-parity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let content = json!({
            "schema_version": 1,
            "game_id": MINING_GAME_ID,
            "profile_id": PROFILE,
            "name_id": 7910,
            "description_id": 7911,
            "icon_id": 909,
            "spec": spec,
        });
        fs::write(
            directory.join("mining.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        Self { directory }
    }

    fn load(&self, state: Option<Type8State>) -> Type8Engine {
        Type8Engine::load(
            &self.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            state,
        )
        .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("mining.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn spec_with(shape_code: i32, collectable_percent: u32, seed: u32) -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            shape[y * SIDE + x] = shape_code;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![401, 402, 403, 404],
        collectable: vec![ORE, ORE_TWO],
        tiles: vec![
            mining::Type8Kind {
                value: 401,
                category: 400,
                icon_id: 1,
            },
            mining::Type8Kind {
                value: ORE,
                category: 600,
                icon_id: 909,
            },
        ],
        goals: vec![goal(0, ORE, 15, 1), goal(1, ORE_TWO, 6, 1)],
        outputs: Vec::new(),
        duration_seconds: 180,
        seed,
        wildcard_percent: 0,
        collectable_percent,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Synthetic private fixture. Original server level tables unrecovered.".into(),
    }
}

fn goal(index: i32, value: u16, quota: u32, per_reward: u32) -> Type8Goal {
    Type8Goal {
        index,
        value,
        quota,
        per_reward,
        icon_id: 909,
    }
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [id, -1, game] {
        b.extend(n.to_le_bytes());
    }
    b
}

fn client_payload(id: i32, s: &str) -> Vec<u8> {
    let mut b = header(14, id, -1);
    b.extend(((s.len() + 1) as i32).to_le_bytes());
    b.extend(s.as_bytes());
    b.push(0);
    b
}

fn payload_text(packet: &[u8]) -> Option<String> {
    if packet.len() < 19 || packet[0..3] != [39, 0, 14] {
        return None;
    }
    let n = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    assert!(n >= 1);
    let end = 19 + n as usize - 1;
    assert_eq!(packet.len(), end + 1);
    assert_eq!(packet[end], 0);
    Some(String::from_utf8(packet[19..end].to_vec()).unwrap())
}

fn texts(outcome: &Type8Outcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter_map(|p| payload_text(p))
        .collect()
}

fn enter_playing(engine: &mut Type8Engine) {
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, FIRST_INSTANCE, -1)).unwrap();
    engine
        .handle(&client_payload(FIRST_INSTANCE, "OnConnectMsg"))
        .unwrap();
    engine
        .handle(&client_payload(FIRST_INSTANCE, "OnStartGameMsg"))
        .unwrap();
    engine
        .handle(&client_payload(FIRST_INSTANCE, "OnStartLevelMsg"))
        .unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Playing
    );
}

fn playable_cells(run: &Type8Run, spec: &Type8BoardSpec) -> Vec<u16> {
    let mut cells = Vec::new();
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            cells.push(run.board[y * spec.width + x]);
        }
    }
    cells
}

fn pattern(x: usize, y: usize) -> u16 {
    401 + ((x + y) % 4) as u16
}

/// Restored chainable run with a short refill queue, mirroring the captured
/// board's bottom ores so the post-chain gravity/reshuffle paths run.
fn short_queue_state(seed: u32, queue_len: usize) -> Type8State {
    let mut board = vec![0_u16; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            board[y * SIDE + x] = pattern(x, y);
        }
    }
    for x in LEFT..LEFT + 3 {
        board[TOP * SIDE + x] = 403;
    }
    board[(TOP + PLAYABLE - 1) * SIDE + 8] = ORE;
    Type8State {
        version: 1,
        next_instance_id: INSTANCE + 1,
        completed: 0,
        active: Some(Type8Run {
            instance_id: INSTANCE,
            phase: Type8Phase::Playing,
            idle: vec![true; board.len()],
            board,
            tile_queue: vec![402; queue_len],
            random_queue: Vec::new(),
            rng: seed.max(1),
            collected: vec![0, 0],
            score: 3000,
            remaining_ms: 172_329,
            wall_ms: 973_839,
            paused: false,
            chains: 6,
            reshuffles: 0,
            end_sent: false,
        }),
    }
}

/// Shape `1` is `TILE_BUILDREGULAR`: the original regular randomizer has no
/// collectable element, so a forced 100% collectable chance must still leave
/// every playable cell selectable.
#[test]
fn shape1_regular_cells_exclude_collectables_at_forced_percent() {
    let spec = spec_with(BUILD_REGULAR, 100, 18);
    let fixture = Fixture::new(&spec);
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let run = engine.state().active.as_ref().unwrap();
    let cells = playable_cells(run, &spec);
    assert!(
        cells.iter().all(|v| !spec.collectable.contains(v)),
        "shape 1 cells must never draw collectables (original regular pool)"
    );
    assert!(
        cells.iter().all(|v| spec.selectable.contains(v)),
        "shape 1 cells must all draw the regular selectable pool"
    );
}

/// Shape `2` is `TILE_BUILDANY`: collectables stay allowed.
#[test]
fn shape2_any_allows_collectables_at_forced_percent() {
    let spec = spec_with(BUILD_ANY, 100, 18);
    let fixture = Fixture::new(&spec);
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let run = engine.state().active.as_ref().unwrap();
    let cells = playable_cells(run, &spec);
    assert!(
        cells.iter().any(|v| spec.collectable.contains(v)),
        "shape 2 cells must still allow collectables"
    );
}

/// Gravity refill queues keep the collectable-enabled pool: a short queue is
/// topped up from the same `random_tile(true)` source, forced to 100%.
#[test]
fn gravity_refill_queue_still_allows_collectables() {
    let spec = spec_with(BUILD_REGULAR, 100, 18);
    let fixture = Fixture::new(&spec);
    let mut engine = fixture.load(Some(short_queue_state(18, 49)));
    let out = engine
        .handle(&client_payload(INSTANCE, CHAIN_MESSAGE))
        .unwrap();
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.chains, 7);
    assert!(
        run.tile_queue.iter().any(|v| spec.collectable.contains(v)),
        "refill queues must keep the collectable-enabled pool"
    );
    assert!(!texts(&out).is_empty());
}

/// Mixed seeds: the shape-1 refusal holds across seeds and percentages, while
/// shape 2 keeps producing collectables.
#[test]
fn mixed_seed_cases_keep_shape1_collectable_free() {
    for seed in [18_u32, 19, 20, 21] {
        let spec = spec_with(BUILD_REGULAR, 50, seed);
        let fixture = Fixture::new(&spec);
        let mut engine = fixture.load(None);
        enter_playing(&mut engine);
        let run = engine.state().active.as_ref().unwrap();
        let cells = playable_cells(run, &spec);
        assert!(
            cells.iter().all(|v| !spec.collectable.contains(v)),
            "shape 1 seed {seed} leaked a collectable"
        );
        assert!(
            run.tile_queue.iter().any(|v| spec.collectable.contains(v)),
            "shape 1 seed {seed} refill queue lost collectables"
        );
    }
    let spec = spec_with(BUILD_ANY, 100, 19);
    let fixture = Fixture::new(&spec);
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let run = engine.state().active.as_ref().unwrap();
    assert!(
        playable_cells(run, &spec)
            .iter()
            .any(|v| spec.collectable.contains(v)),
        "shape 2 mixed-seed case lost collectables"
    );
}
