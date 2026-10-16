//! Bounded regression for the captured dead board (native run 125556): a
//! chainless mining board with a collectable parked on the bottom playable
//! row must still rebuild like the original
//! `ServerGrid.updateCurrentGridArray` (`if(!HasMove()) LevelReshuffle()`),
//! on both the chain-update and collect-update paths. `has_bottom_collectable`
//! must never suppress the rebuild the original performs.
//!
//! Every fixture is synthetic and private: no user save, native client, gateway state or shared content is read or written.
use osfr_tools::mining::{
    Type8BoardSpec, Type8Engine, Type8Goal, Type8Kind, Type8Outcome, Type8Phase, Type8Run,
    Type8State, MINING_GAME_ID, SIDE,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const BOTTOM: usize = TOP + PLAYABLE - 1;
const PROFILE: i32 = 14;
const FIRST_INSTANCE: i32 = 18_000_001;
const INSTANCE: i32 = 18_000_002;
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;
const ORE_X: usize = 8;
const ORE_TWO_X: usize = 10;
const COLLECT_MESSAGE: &str = "OnLevelEventMsg\t50\t8\t10";
const CHAIN_MESSAGE: &str = "OnChainMsg\t3,3,4,3,5,3,";

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(spec: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-dead-board-{}-{}",
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

fn spec() -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            shape[y * SIDE + x] = 1;
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
            Type8Kind {
                value: 401,
                category: 400,
                icon_id: 1,
            },
            Type8Kind {
                value: ORE,
                category: 600,
                icon_id: 909,
            },
        ],
        goals: vec![goal(0, ORE, 15, 1), goal(1, ORE_TWO, 6, 1)],
        outputs: Vec::new(),
        duration_seconds: 180,
        seed: 18,
        wildcard_percent: 0,
        collectable_percent: 0,
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

/// Pattern with period four along both axes: orthogonal neighbours never match,
/// so the playable area holds no connected selectable group by construction.
fn pattern(x: usize, y: usize) -> u16 {
    401 + ((x + y) % 4) as u16
}

fn board_with(chain: bool) -> Vec<u16> {
    let mut board = vec![0_u16; SIDE * SIDE];
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            board[y * SIDE + x] = pattern(x, y);
        }
    }
    if chain {
        // Exactly one connected group over two tiles: the 403 chain that the
        // accepted `OnChainMsg` consumes, mirroring the captured run's single
        // legal chain on an otherwise chainless board.
        for x in LEFT..LEFT + 3 {
            board[TOP * SIDE + x] = 403;
        }
    }
    // Both captured bottom-row ores: (10,8)=602-style goal collectable and
    // (10,10)=601 directly above dropzone cells (row 11).
    board[BOTTOM * SIDE + ORE_X] = ORE;
    board[BOTTOM * SIDE + ORE_TWO_X] = ORE_TWO;
    board
}

/// Queue with a controlled refill prefix; the length stays above `QUEUE_MIN`
/// (50) so gravity never appends an rng batch and the refill stays exact.
fn queue(prefix: &[u16]) -> Vec<u16> {
    let mut queue = prefix.to_vec();
    queue.resize(60, 402);
    queue
}

fn dead_state(board: Vec<u16>, tile_queue: Vec<u16>) -> Type8State {
    Type8State {
        version: 1,
        next_instance_id: INSTANCE + 1,
        completed: 0,
        active: Some(Type8Run {
            instance_id: INSTANCE,
            phase: Type8Phase::Playing,
            idle: vec![true; board.len()],
            board,
            tile_queue,
            random_queue: Vec::new(),
            rng: 3_309_872_643,
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

fn has_text(outcome: &Type8Outcome, prefix: &str) -> bool {
    texts(outcome)
        .iter()
        .any(|t| t == prefix || t.starts_with(&format!("{prefix}\t")))
}

/// Captured dead-board regression, chain-update path: an accepted chain leaves
/// a chainless board with a collectable on the bottom playable row. The
/// original rebuilds (`if(!HasMove()) LevelReshuffle()`); the bottom ore must
/// not suppress that rebuild.
#[test]
fn chainless_board_with_bottom_ore_rebuilds_after_chain() {
    let fixture = Fixture::new(&spec());
    // Refill for columns 5, 4, 3 (gravity scans right to left): keeps the
    // quarter pattern intact, so the post-chain board is genuinely chainless.
    let mut engine = fixture.load(Some(dead_state(board_with(true), queue(&[404, 403, 401]))));
    let out = engine
        .handle(&client_payload(INSTANCE, CHAIN_MESSAGE))
        .unwrap();
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.chains, 7);
    assert_eq!(run.collected, vec![0, 0]);
    assert_eq!(
        run.reshuffles, 1,
        "chainless board with a bottom collectable must rebuild like the original"
    );
    assert!(
        has_text(&out, "OnLevelEventMsg\t11"),
        "reshuffle notice must reach the client"
    );
    assert!(
        has_text(&out, "OnBuildMapMsg"),
        "rebuilt board must be sent to the client"
    );
}

/// Captured dead-board regression, collect-update path: a client fall for one
/// bottom ore leaves a second bottom ore and a chainless board. The original
/// still rebuilds; the remaining bottom ore must not suppress it.
#[test]
fn chainless_board_with_bottom_ore_rebuilds_after_collect() {
    let fixture = Fixture::new(&spec());
    // Column 8 refill after the collected tile shifts the column down one row.
    let mut engine = fixture.load(Some(dead_state(board_with(false), queue(&[402]))));
    let out = engine
        .handle(&client_payload(INSTANCE, COLLECT_MESSAGE))
        .unwrap();
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.collected, vec![1, 0]);
    assert_eq!(run.score, 3500);
    assert!(
        has_text(&out, "OnQuotaUpdateMsg\t0"),
        "collect must update the goal quota"
    );
    assert_eq!(
        run.reshuffles, 1,
        "chainless board with a remaining bottom collectable must rebuild"
    );
    assert!(
        has_text(&out, "OnLevelEventMsg\t11"),
        "reshuffle notice must reach the client"
    );
}

/// Guard: the rebuild still requires a chainless board. A refill that leaves a
/// connected selectable group must not reshuffle even with a bottom ore.
#[test]
fn board_with_a_move_does_not_rebuild() {
    let fixture = Fixture::new(&spec());
    let mut engine = fixture.load(Some(dead_state(board_with(true), queue(&[401, 401, 401]))));
    let out = engine
        .handle(&client_payload(INSTANCE, CHAIN_MESSAGE))
        .unwrap();
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.chains, 7);
    assert_eq!(
        run.reshuffles, 0,
        "a board with a legal move must not rebuild"
    );
    assert!(!has_text(&out, "OnLevelEventMsg\t11"));
}
