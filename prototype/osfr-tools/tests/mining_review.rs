//! Independent review probes for Mining18 (TYPE8), owned by the mining review
//! worker. Production code is read-only here: this file only drives
//! `src/mining.rs` and records what it actually does against the recovered
//! original client (`lab/decompiled/minigames/mining/game-scripts`).
//!
//! The two counterexamples this file originally carried as `#[ignore]`d RED
//! probes (native trailing-comma wire text, unusable specs accepted at load)
//! are now ordinary passing regressions for the authorized repair, together
//! with the boundary probes that keep the new caps honest.
use osfr_tools::mining::{
    self, Type8BoardSpec, Type8Engine, Type8Goal, Type8Outcome, Type8Phase, Type8Run, Type8State,
    MINING_GAME_ID, SIDE,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const PROFILE: i32 = 14;
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;

/// Pinned community oracle vectors (`community-serializer-vectors.json`,
/// unmodified Sanctuary BaseMiniGamePacket writer compiled with the arguments
/// proven by the original x86 constructors). Client frames carry GameId=-1.
const ORACLE_FORGING_START: &str = "270005c1f84808ffffffffffffffff";
const ORACLE_FORGING_CONNECT: &str =
    "27000ec1f84808ffffffffffffffff0d0000004f6e436f6e6e6563744d736700";

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(spec: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-review-{}-{}",
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
        self.try_load(18_000_001, state).unwrap()
    }

    fn try_load(
        &self,
        first_instance: i32,
        state: Option<Type8State>,
    ) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            "mining.json",
            MINING_GAME_ID,
            first_instance,
            state,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("mining.json"));
        let _ = fs::remove_dir(&self.directory);
    }
}

fn hex(text: &str) -> Vec<u8> {
    assert!(text.len() % 2 == 0);
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn rows_all(code: i32) -> [[i32; PLAYABLE]; PLAYABLE] {
    [[code; PLAYABLE]; PLAYABLE]
}

fn spec_with(
    rows: [[i32; PLAYABLE]; PLAYABLE],
    goals: Vec<Type8Goal>,
    duration_seconds: u32,
) -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, value) in row.iter().enumerate() {
            shape[(TOP + dy) * SIDE + LEFT + dx] = *value;
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
        goals,
        outputs: Vec::new(),
        duration_seconds,
        seed: 18,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Review fixture. Original server level tables unrecovered.".into(),
    }
}

fn spec(rows: [[i32; PLAYABLE]; PLAYABLE], goals: Vec<Type8Goal>) -> Type8BoardSpec {
    spec_with(rows, goals, 180)
}

/// 14x32 = 448 cells: the largest board the run state accepts. Rows 1..=30
/// form the playable rectangle, row 31 is the dropzone row and row 0 is empty.
fn tall_spec() -> Type8BoardSpec {
    let mut spec = spec(rows_all(1), vec![goal(0, ORE, 1, 1)]);
    spec.height = 32;
    let mut shape = vec![0_i32; SIDE * 32];
    for y in 1..=30 {
        for x in LEFT..LEFT + PLAYABLE {
            shape[y * SIDE + x] = 1;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[31 * SIDE + x] = 4;
    }
    spec.shape = shape;
    spec
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

fn client_payload(id: i32, text: &str) -> Vec<u8> {
    let mut b = header(14, id, -1);
    b.extend(((text.len() + 1) as i32).to_le_bytes());
    b.extend(text.as_bytes());
    b.push(0);
    b
}

fn payload_text(packet: &[u8]) -> Option<String> {
    if packet.len() < 20 || packet[0..3] != [39, 0, 14] {
        return None;
    }
    let n = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    assert!(n >= 1);
    let end = 19 + n as usize - 1;
    assert_eq!(packet.len(), end + 1);
    assert_eq!(packet[end], 0);
    Some(String::from_utf8(packet[19..end].to_vec()).unwrap())
}

fn has_text(outcome: &Type8Outcome, expected: &str) -> bool {
    outcome
        .replies
        .iter()
        .filter_map(|p| payload_text(p))
        .any(|t| t == expected || t.starts_with(&format!("{expected}\t")))
}

fn enter_playing(engine: &mut Type8Engine, id: i32) {
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg\t0"))
        .unwrap();
    engine
        .handle(&client_payload(id, "OnStartLevelMsg\t0"))
        .unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Playing
    );
}

/// Provenance: the engine accepts the exact pinned community client frames
/// (op39/sub5 with GameId -1, op39/sub14 with a NUL-terminated `len+1` text)
/// and answers with the real game id in the server-direction header.
#[test]
fn pinned_oracle_client_frames_drive_the_handshake() {
    let start = hex(ORACLE_FORGING_START);
    let connect = hex(ORACLE_FORGING_CONNECT);
    let instance = i32::from_le_bytes(start[3..7].try_into().unwrap());
    assert_eq!(&start[0..3], &[39u8, 0, 5][..]);
    assert_eq!(
        &start[7..15],
        &[0xffu8; 8][..],
        "GroupId and GameId are both -1"
    );
    assert_eq!(&connect[0..3], &[39u8, 0, 14][..]);
    assert_eq!(&connect[7..15], &[0xffu8; 8][..]);
    assert_eq!(
        i32::from_le_bytes(connect[15..19].try_into().unwrap()),
        13,
        "Lua writer includes the NUL in len+1"
    );
    assert_eq!(&connect[19..31], b"OnConnectMsg".as_slice());
    assert_eq!(connect[31], 0);
    assert_eq!(connect.len(), 32, "frame is header + len + text + NUL");

    let fixture = Fixture::new(&spec(rows_all(1), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.try_load(instance, None).unwrap();
    let info = engine.begin(PROFILE).unwrap();
    assert_eq!(&info.replies[0][3..7], &instance.to_le_bytes());

    let start_reply = engine.handle(&start).unwrap();
    assert!(
        start_reply
            .replies
            .iter()
            .any(|p| p == &header(17, instance, MINING_GAME_ID)),
        "sub 5 must be acked with sub 17 carrying the mining game id"
    );
    let connect_reply = engine.handle(&connect).unwrap();
    assert!(has_text(&connect_reply, "OnServerReadyMsg"));
}

/// The recovered client appends a trailing comma to every coordinate list
/// (`core/Grid.as` builds `x,y,` per tile; the original server pops the empty
/// tail in `GameLevel.OnChainMsg`). Exactly one trailing empty field is
/// tolerated; every other empty field stays invalid.
#[test]
fn trailing_comma_is_tolerated_once_and_internal_empties_stay_invalid() {
    let mut rows = rows_all(1);
    rows[5][0] = -401;
    rows[5][1] = -401;
    rows[5][2] = -401;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    let native = format!(
        "OnChainMsg\t{},{},{},{},{},{},",
        LEFT,
        TOP + 5,
        LEFT + 1,
        TOP + 5,
        LEFT + 2,
        TOP + 5
    );
    let accepted = engine
        .handle(&client_payload(id, &native))
        .expect("native chain text validates against the board");
    assert!(accepted.changed);
    assert_eq!(engine.state().active.as_ref().unwrap().chains, 1);
    for text in [
        // Internal empty field (between two real coordinates).
        format!(
            "OnChainMsg\t{},{},{},,{},{},{},",
            LEFT,
            TOP + 6,
            LEFT + 1,
            TOP + 6,
            LEFT + 2,
            TOP + 6
        ),
        // Two trailing empties: only a single original tail is accepted.
        format!(
            "OnChainMsg\t{},{},{},{},{},{},,",
            LEFT,
            TOP + 6,
            LEFT + 1,
            TOP + 6,
            LEFT + 2,
            TOP + 6
        ),
    ] {
        let before = engine.state().clone();
        assert!(
            engine.handle(&client_payload(id, &text)).is_err(),
            "empty field other than the single original tail must stay invalid: {text}"
        );
        assert_eq!(before, *engine.state());
    }
}

#[test]
fn native_client_chain_idle_and_verify_wire_text_is_accepted() {
    let mut rows = rows_all(1);
    rows[5][0] = -401;
    rows[5][1] = -401;
    rows[5][2] = -401;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    let chained = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{},",
                LEFT,
                TOP + 5,
                LEFT + 1,
                TOP + 5,
                LEFT + 2,
                TOP + 5
            ),
        ))
        .expect("native chain text must validate against the board");
    assert!(has_text(&chained, "OnLevelEventMsg\t21\t400"));
    let idled = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnIdleMsg\t{},{},{},{},{},{},",
                LEFT,
                TOP + 5,
                LEFT + 1,
                TOP + 5,
                LEFT + 2,
                TOP + 5
            ),
        ))
        .expect("native idle text must be accepted");
    assert!(idled.changed);
    // Zero means "do not check" in the recovered ServerGrid.CheckGrid, so the
    // native 196-triple verify tail is exercised without guessing refilled
    // tile values.
    let verified = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnVerifyMsg\t{},{},0,{},{},0,",
                LEFT,
                TOP + 5,
                LEFT + 2,
                TOP + 5
            ),
        ))
        .expect("native verify text must be accepted");
    assert!(!verified.changed);
    let mismatch = engine
        .handle(&client_payload(
            id,
            &format!("OnVerifyMsg\t{},{},999,", LEFT, TOP + 5),
        ))
        .expect_err("a checked value that cannot match the board must still fail");
    assert!(mismatch.to_string().contains("verification"));
}

#[test]
fn native_client_verify_tail_and_non_numeric_fields_are_handled() {
    let mut rows = rows_all(1);
    rows[5][0] = -401;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    // A whole-board verify payload (14x14 triples) with the recovered tail.
    let mut text = String::from("OnVerifyMsg\t");
    for y in 0..SIDE {
        for x in 0..SIDE {
            text.push_str(&format!("{x},{y},0,"));
        }
    }
    let out = engine
        .handle(&client_payload(id, &text))
        .expect("full native verify payload is accepted");
    assert!(!out.changed);
    let before = engine.state().clone();
    for bad in [
        "OnChainMsg\tnan,1,2,1,3,1,",
        "OnChainMsg\t1.5,1,2,1,3,1",
        "OnChainMsg\t4097,1,2,1,3,1",
        "OnLevelEventMsg\t50\tinf\t3",
        "OnLevelEventMsg\t50\t-1000001\t3",
        "FRServer_Pause\tmaybe",
        "OnVerifyMsg\t1,1,65536",
    ] {
        assert!(
            engine.handle(&client_payload(id, bad)).is_err(),
            "non-finite or out-of-range field must stay invalid: {bad}"
        );
        assert_eq!(before, *engine.state());
    }
}

/// Counterexample: `validate_spec` accepts durations and shape codes that the
/// run validator and tile factory cannot execute, so `load()` succeeds for
/// content that can never start a level.
#[test]
fn unusable_specs_are_rejected_at_load() {
    let long = Fixture::new(&spec_with(rows_all(1), vec![goal(0, ORE, 1, 1)], 601));
    assert!(
        long.try_load(18_000_001, None).is_err(),
        "duration 601s exceeds the 600s state cap; load must not accept it silently"
    );
    assert!(
        Fixture::new(&spec_with(rows_all(1), vec![goal(0, ORE, 1, 1)], 600))
            .try_load(18_000_001, None)
            .is_ok(),
        "duration 600s equals the cap and stays accepted"
    );
    let mut category = rows_all(1);
    category[0][0] = 200;
    assert!(
        Fixture::new(&spec(category, vec![goal(0, ORE, 1, 1)]))
            .try_load(18_000_001, None)
            .is_err(),
        "shape code 200 maps to an unsupported category"
    );
    let mut specific = rows_all(1);
    specific[0][0] = -101;
    assert!(
        Fixture::new(&spec(specific, vec![goal(0, ORE, 1, 1)]))
            .try_load(18_000_001, None)
            .is_err(),
        "shape code -101 yields board value 101 which is not a valid tile"
    );
    for code in [500, 700, 800, -1002] {
        let mut rows = rows_all(1);
        rows[0][0] = code;
        assert!(
            Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]))
                .try_load(18_000_001, None)
                .is_err(),
            "shape code {code} is not executable by the engine"
        );
    }
    for code in [400, 499, 600, 699, -200, -401, -601] {
        let mut rows = rows_all(1);
        rows[0][0] = code;
        assert!(
            Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]))
                .try_load(18_000_001, None)
                .is_ok(),
            "shape code {code} maps to a tile the engine can build and hold"
        );
    }
    let mut oversized = spec(rows_all(1), vec![goal(0, ORE, 1, 1)]);
    oversized.width = 32;
    oversized.height = 32;
    oversized.shape = vec![1; 32 * 32];
    assert!(
        Fixture::new(&oversized).try_load(18_000_001, None).is_err(),
        "boards above the 448-cell run cap must not load"
    );
    assert!(
        Fixture::new(&tall_spec())
            .try_load(18_000_001, None)
            .is_ok(),
        "a 448-cell board stays loadable"
    );
}

#[test]
fn duration_ceiling_matches_the_run_state_cap() {
    let mut rows = rows_all(1);
    rows[5][0] = -401;
    rows[5][1] = -401;
    rows[5][2] = -401;
    let fixture = Fixture::new(&spec_with(rows, vec![goal(0, ORE, 1, 1)], 600));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    assert_eq!(
        engine.state().active.as_ref().unwrap().remaining_ms,
        600_000
    );
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{},",
                LEFT,
                TOP + 5,
                LEFT + 1,
                TOP + 5,
                LEFT + 2,
                TOP + 5
            ),
        ))
        .unwrap();
    // At the cap the chain bonus is bounded to zero instead of overflowing.
    assert!(has_text(&out, "OnLevelEventMsg\t25\t0\t1"));
    assert_eq!(
        engine.state().active.as_ref().unwrap().remaining_ms,
        600_000
    );
    Type8Engine::validate_state(engine.state()).unwrap();
    let expired = engine.tick(600_000).unwrap();
    assert!(has_text(&expired, "OnStopLevel\t1\tfalse"));
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Lost
    );
}

#[test]
fn board_ceiling_matches_the_run_state_cap() {
    let fixture = Fixture::new(&tall_spec());
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.board.len(), 448);
    assert_eq!(run.idle.len(), 448);
    assert!(run.remaining_ms <= 600_000);
    Type8Engine::validate_state(engine.state()).unwrap();
}

#[test]
fn client_reported_score_and_win_texts_never_change_authoritative_state() {
    let mut rows = rows_all(1);
    rows[7][0] = -i32::from(ORE);
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 2, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    let before = engine.state().clone();
    for text in [
        "FRServer_ScoreSet\t900000",
        "FRServer_GameWon\t900000\t50",
        "FRServer_GameLost\t900000",
        "FRServer_GameQuit\t900000",
    ] {
        let out = engine.handle(&client_payload(id, text)).unwrap();
        assert!(!out.completed);
        assert!(out.transaction.is_none());
    }
    assert_eq!(before, *engine.state());
    // A fall claim for a tile that is not a bottom-row collectable is refused.
    assert!(engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, TOP)
        ))
        .is_err());
    assert_eq!(before, *engine.state());
}

#[test]
fn reconnect_cancel_drops_the_run_and_stale_instance_messages_are_ignored() {
    let fixture = Fixture::new(&spec(rows_all(1), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let cancelled = engine.cancel();
    assert!(cancelled.changed);
    assert!(!engine.active());
    assert!(
        cancelled
            .replies
            .iter()
            .any(|p| p == &header(19, 18_000_001, MINING_GAME_ID)),
        "reconnect teardown sends the state-removal frame"
    );
    let stale = engine
        .handle(&client_payload(18_000_001, "OnStartGameMsg\t0"))
        .unwrap();
    assert!(stale.replies.is_empty());
    assert!(!stale.changed);
    let replay = engine.begin(PROFILE).unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().instance_id,
        18_000_002,
        "a new session allocates a fresh instance id"
    );
    assert_eq!(replay.replies.len(), 1);
    assert!(
        engine.state().active.as_ref().unwrap().board.is_empty(),
        "the new session does not inherit the dropped board"
    );
    let stale_cancel = engine.cancel();
    assert!(stale_cancel
        .replies
        .iter()
        .any(|p| p == &header(19, 18_000_002, MINING_GAME_ID)));
}

#[test]
fn queue_and_board_stay_bounded_across_a_scripted_chain_run() {
    let mut rows = rows_all(1);
    for x in 0..PLAYABLE {
        rows[0][x] = -401;
        rows[1][x] = -401;
        rows[2][x] = -401;
    }
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    {
        let run = engine.state().active.as_ref().unwrap();
        assert_eq!(
            run.tile_queue.len(),
            70,
            "queue seeds to the original target"
        );
        assert_eq!(run.random_queue.len(), 70);
        assert_eq!(run.board.len(), SIDE * SIDE);
        assert_eq!(run.idle.len(), run.board.len());
    }
    for x in 0..PLAYABLE {
        let text = format!(
            "OnChainMsg\t{},{},{},{},{},{}",
            LEFT + x,
            TOP,
            LEFT + x,
            TOP + 1,
            LEFT + x,
            TOP + 2
        );
        let out = engine.handle(&client_payload(id, &text)).unwrap();
        for reply in out.replies.iter().filter_map(|p| payload_text(p)) {
            if let Some(list) = reply.strip_prefix("OnTileQueueMsg\t-1\t") {
                let values = list.split(',').count();
                assert!(
                    values <= 196,
                    "queue replies are either the 196-cell board or a <=20 batch, saw {values}"
                );
            }
        }
        let run = engine.state().active.as_ref().unwrap();
        assert!(
            run.tile_queue.len() <= 70,
            "tile queue never exceeds the target"
        );
        assert_eq!(run.board.len(), SIDE * SIDE);
        assert_eq!(run.idle.len(), run.board.len());
        assert!(run.remaining_ms <= 600_000);
        assert_eq!(run.chains as usize, x + 1);
        Type8Engine::validate_state(engine.state()).unwrap();
    }
}

#[test]
fn tampered_state_outside_the_declared_bounds_is_rejected() {
    let fixture = Fixture::new(&spec(rows_all(1), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    engine.begin(PROFILE).unwrap();
    let active: Type8Run = engine.state().active.clone().unwrap();
    Type8Engine::validate_state(engine.state()).unwrap();

    let mut oversized_queue = active.clone();
    oversized_queue.tile_queue = vec![401; 141];
    let mut state = engine.state().clone();
    state.active = Some(oversized_queue);
    assert!(
        Type8Engine::validate_state(&state).is_err(),
        "tile queue above QUEUE_TARGET * 2 must be refused"
    );

    let mut oversized_board = active;
    oversized_board.board = vec![401; SIDE * 32 + 1];
    oversized_board.idle = vec![true; oversized_board.board.len()];
    let mut state = engine.state().clone();
    state.active = Some(oversized_board);
    assert!(
        Type8Engine::validate_state(&state).is_err(),
        "board above SIDE * 32 cells must be refused"
    );
}

#[test]
fn default_spec_completes_with_the_reconstructed_ore_mapping() {
    let spec = mining::mining_spec();
    assert_eq!(spec.collectable_percent, 40);
    assert_eq!(
        spec.goals.iter().map(|goal| goal.quota).collect::<Vec<_>>(),
        [6, 6]
    );
    assert_eq!(spec.outputs.len(), 2);
    assert!(spec.reconstructed_rules);
    assert!(
        spec.goals.len() >= 2,
        "the default Mining18 spec carries two collectable goals"
    );
    let mut rows = rows_all(1);
    let mut goals = Vec::new();
    for (index, goal) in spec.goals.iter().enumerate() {
        let mut reachable = goal.clone();
        reachable.quota = 1;
        reachable.per_reward = 1;
        goals.push(reachable);
        rows[7][index] = -i32::from(goal.value);
    }
    let mut fixture_spec = spec_with(rows, goals, spec.duration_seconds);
    fixture_spec.outputs = spec.outputs.clone();
    let fixture = Fixture::new(&fixture_spec);
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    for column in 0..spec.goals.len() {
        engine
            .handle(&client_payload(
                id,
                &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + column, TOP + 7),
            ))
            .unwrap();
    }
    let out = engine
        .handle(&client_payload(id, "OnNextLevelMsg"))
        .unwrap();
    assert!(out.completed);
    assert_eq!(
        out.transaction.unwrap().grants,
        vec![(2739, 1), (2739, 2)],
        "default output mapping is reconstructed policy, not a recovered drop table"
    );
}

#[test]
fn malformed_payloads_are_rejected_without_mutating_state() {
    let fixture = Fixture::new(&spec(rows_all(1), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine, 18_000_001);
    let id = 18_000_001;
    let before = engine.state().clone();
    let cases: Vec<Vec<u8>> = vec![
        vec![39, 0, 5],
        {
            let mut b = header(5, id, -1);
            b.push(0);
            b
        },
        {
            let mut b = client_payload(id, "OnChainMsg\t1,1,2,1,3,1");
            let n = i32::from_le_bytes(b[15..19].try_into().unwrap()) + 4;
            b[15..19].copy_from_slice(&n.to_le_bytes());
            b
        },
        client_payload(id, "bad\npayload"),
        client_payload(id, "OnChainMsg\t5000,0,1,0,2,0"),
        client_payload(id, "OnChainMsg\t1,1,2,2,3"),
        client_payload(id, "OnVerifyMsg\t1,1,70000"),
        client_payload(id, "NotAMessage"),
        client_payload(id, ""),
    ];
    for case in cases {
        assert!(
            engine.handle(&case).is_err(),
            "malformed frame must be refused: {case:?}"
        );
        assert_eq!(
            before,
            *engine.state(),
            "a refused frame must not mutate state"
        );
    }
    // Another minigame family and an unknown sub-op are ignored, not errors.
    let mut other_family = header(5, id, -1);
    other_family[1] = 41;
    let ignored = engine.handle(&other_family).unwrap();
    assert!(!ignored.changed);
    let unknown_sub = engine.handle(&header(3, id, MINING_GAME_ID)).unwrap();
    assert!(!unknown_sub.changed);
    let wrong_instance = engine.handle(&header(5, id + 1, -1)).unwrap();
    assert!(!wrong_instance.changed);
    assert_eq!(before, *engine.state());
}
