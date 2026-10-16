//! Mining18 behavioral regression for the shared TYPE8 match-three engine.
//!
//! Every fixture is synthetic and private: no user save, native client or gateway state is read or written.
use osfr_tools::mining::{
    self, payload_packet, Type8BoardSpec, Type8Engine, Type8Goal, Type8Outcome, Type8Output,
    Type8Phase, Type8State, COPPER_MINING_GAME_ID, MINING_GAME_ID, SIDE, TILE_COLLECTABLE_START,
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
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;
const ORE_DEFINITION: i32 = 920011;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(spec: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-{}-{}",
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

fn empty_rows() -> [[i32; PLAYABLE]; PLAYABLE] {
    // The derived playable rectangle must be hole-free, like a full aMyGrid
    // region; individual tests overwrite the cells they control.
    [[1; PLAYABLE]; PLAYABLE]
}

fn spec(rows: [[i32; PLAYABLE]; PLAYABLE], goals: Vec<Type8Goal>) -> Type8BoardSpec {
    spec_with(rows, goals, Vec::new())
}

fn spec_with(
    rows: [[i32; PLAYABLE]; PLAYABLE],
    goals: Vec<Type8Goal>,
    outputs: Vec<Type8Output>,
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
        outputs,
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

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn text_with<'a>(outcome: &'a Type8Outcome, prefix: &'a str) -> Option<String> {
    texts(outcome)
        .into_iter()
        .find(|t| t == prefix || t.starts_with(&format!("{prefix}\t")))
}

fn enter_playing(engine: &mut Type8Engine) {
    let id = FIRST_INSTANCE;
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    engine
        .handle(&client_payload(id, "OnStartLevelMsg"))
        .unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Playing
    );
}

#[test]
fn payload_packet_matches_pinned_original_framing() {
    let packet = payload_packet(FIRST_INSTANCE, "OnConnectMsg").unwrap();
    let mut expected = vec![39, 0, 14];
    expected.extend(FIRST_INSTANCE.to_le_bytes());
    expected.extend((-1_i32).to_le_bytes());
    expected.extend(MINING_GAME_ID.to_le_bytes());
    expected.extend(13_i32.to_le_bytes());
    expected.extend(b"OnConnectMsg");
    expected.push(0);
    assert_eq!(packet, expected);
    assert_eq!(payload_text(&packet).unwrap(), "OnConnectMsg");
    // Compiled unchanged-community writer vector form (pinned Sanctuary
    // BaseMiniGamePacket, native-proven GameId=-1 sentinel) reproduced for
    // the mining instance: 27000e <instance> ffffffff ffffffff <len+1> text 00.
    let mut oracle = vec![39, 0, 14];
    oracle.extend(FIRST_INSTANCE.to_le_bytes());
    oracle.extend((-1_i32).to_le_bytes());
    oracle.extend((-1_i32).to_le_bytes());
    oracle.extend(13_i32.to_le_bytes());
    oracle.extend(b"OnConnectMsg");
    oracle.push(0);
    assert_eq!(client_payload(FIRST_INSTANCE, "OnConnectMsg"), oracle);
    assert!(payload_packet(FIRST_INSTANCE, "").is_err());
    assert!(payload_packet(FIRST_INSTANCE, "has\nnewline").is_err());
}

#[test]
fn begin_emits_info_packet_and_blocks_a_second_run() {
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    let out = engine.begin(PROFILE).unwrap();
    assert!(out.changed);
    assert_eq!(out.replies.len(), 1);
    let info = &out.replies[0];
    assert_eq!(info[0..3], [39, 0, 16]);
    assert_eq!(
        i32::from_le_bytes(info[3..7].try_into().unwrap()),
        FIRST_INSTANCE
    );
    assert!(engine.begin(PROFILE).is_err());
    let wrong = engine.begin(PROFILE + 1);
    assert!(wrong.is_err());
}

#[test]
fn handshake_reaches_playing_with_the_original_message_set() {
    let fixture = Fixture::new(&spec(
        empty_rows(),
        vec![goal(0, ORE, 4, 1), goal(1, ORE_TWO, 4, 2)],
    ));
    let mut engine = fixture.load(None);
    let id = FIRST_INSTANCE;
    engine.begin(PROFILE).unwrap();

    let start = engine.handle(&header(5, id, -1)).unwrap();
    assert!(start
        .replies
        .iter()
        .any(|p| p == &header(17, id, MINING_GAME_ID)));
    let launch = start
        .replies
        .iter()
        .find(|p| p[0..4] == [26, 0, 12, 0])
        .expect("launch packet");
    let movie = b"game_mining.gfx";
    assert!(launch.windows(movie.len()).any(|w| w == movie));

    let connect = engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    assert!(has_text(&connect, "OnServerReadyMsg"));
    assert!(engine
        .handle(&header(5, id, -1))
        .unwrap()
        .replies
        .is_empty());

    let start_game = engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    assert!(has_text(&start_game, "OnNextLevelMsg"));

    let start_level = engine
        .handle(&client_payload(id, "OnStartLevelMsg"))
        .unwrap();
    assert!(has_text(&start_level, "OnLevelInitMsg\t1\t180"));
    assert!(has_text(&start_level, "OnBuildMapMsg\t14\t14"));
    assert_eq!(
        texts(&start_level)
            .iter()
            .filter(|t| t.starts_with("OnTileQueueMsg\t-1"))
            .count(),
        2
    );
    assert!(has_text(&start_level, "OnRandomQueueMsg"));
    assert!(has_text(&start_level, "OnQuotaInitMsg\t0\t601\t4\t1"));
    assert!(has_text(&start_level, "OnQuotaInitMsg\t1\t602\t4\t2"));
    assert!(has_text(&start_level, "OnLevelEventMsg\t32"));
    assert!(has_text(&start_level, "OnActiveMsg\ttrue"));
    let board_message = text_with(&start_level, "OnBuildMapMsg").unwrap();
    let shape: Vec<i32> = board_message
        .split('\t')
        .nth(3)
        .unwrap()
        .split(',')
        .map(|v| v.parse().unwrap())
        .collect();
    assert_eq!(shape.len(), SIDE * SIDE);
    assert_eq!(shape[(TOP + PLAYABLE) * SIDE + LEFT], 4);
}

#[test]
fn chain_must_be_orthogonal_distinct_matching_and_on_the_board() {
    let mut rows = empty_rows();
    rows[5][0] = -401;
    rows[5][1] = -401;
    rows[5][2] = -401;
    rows[5][3] = -402;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let before = engine.state().clone();
    let x = LEFT;
    let y = TOP + 5;
    let rejected = [
        // Two tiles is below the original minimum chain length.
        format!("OnChainMsg\t{x},{y},{},{y}", x + 1),
        // Repeated coordinate.
        format!("OnChainMsg\t{x},{y},{x},{y},{},{y}", x + 2),
        // Repeated tile through a longer path.
        format!(
            "OnChainMsg\t{x},{y},{},{y},{},{y},{},{y}",
            x + 1,
            x + 2,
            x + 1
        ),
        // Non-orthogonal diagonal step.
        format!("OnChainMsg\t{x},{y},{},{y},{},{}", x + 1, x + 2, y + 1),
        // Orthogonal but a different selectable value.
        format!("OnChainMsg\t{x},{y},{},{y},{},{y}", x + 1, x + 3),
        // Outside the playable rectangle.
        format!("OnChainMsg\t{x},{y},{},{y},0,0", x + 1),
    ];
    for message in rejected {
        assert!(
            engine.handle(&client_payload(id, &message)).is_err(),
            "chain {message} must be rejected"
        );
    }
    let after = engine.state().clone();
    assert_eq!(before, after);
}

#[test]
fn valid_chain_clears_tiles_scores_and_extends_the_timer() {
    let mut rows = empty_rows();
    rows[6][0] = -403;
    rows[6][1] = -403;
    rows[6][2] = -403;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let before = engine.state().active.as_ref().unwrap().remaining_ms;
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 6,
                LEFT + 1,
                TOP + 6,
                LEFT + 2,
                TOP + 6
            ),
        ))
        .unwrap();
    assert!(has_text(&out, "OnLevelEventMsg\t21\t400"));
    assert!(has_text(&out, "OnLevelEventMsg\t25\t6"));
    let run = engine.state().active.as_ref().unwrap();
    assert_eq!(run.chains, 1);
    assert_eq!(run.remaining_ms, before + 6000);
    assert_eq!(run.score, 400);
}

#[test]
fn idle_tiles_accept_a_resumed_chain_after_the_client_reports_the_fall() {
    let mut rows = empty_rows();
    for row in rows.iter_mut().take(6) {
        row[0] = -403;
        row[1] = -403;
        row[2] = -403;
    }
    rows[6][0] = -402;
    rows[6][1] = -402;
    rows[6][2] = -402;
    rows[7][0] = -401;
    rows[7][1] = -401;
    rows[7][2] = -401;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 7,
                LEFT + 1,
                TOP + 7,
                LEFT + 2,
                TOP + 7
            ),
        ))
        .unwrap();
    let chain = format!(
        "OnChainMsg\t{},{},{},{},{},{}",
        LEFT,
        TOP + 7,
        LEFT + 1,
        TOP + 7,
        LEFT + 2,
        TOP + 7
    );
    // The 402 row above falls into the cleared bottom row; those tiles are
    // not idle until the client reports the finished fall.
    assert!(engine.handle(&client_payload(id, &chain)).is_err());
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnIdleMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 7,
                LEFT + 1,
                TOP + 7,
                LEFT + 2,
                TOP + 7
            ),
        ))
        .unwrap();
    assert!(out.changed);
    engine.handle(&client_payload(id, &chain)).unwrap();
}

#[test]
fn wildcard_matches_the_chain_value_and_all_wildcards_are_legal() {
    let mut rows = empty_rows();
    rows[7][0] = -401;
    rows[7][1] = -200;
    rows[7][2] = -401;
    rows[7][3] = -200;
    rows[7][4] = -200;
    rows[7][5] = -200;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT,
                TOP + 7,
                LEFT + 1,
                TOP + 7,
                LEFT + 2,
                TOP + 7
            ),
        ))
        .unwrap();
    let out = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnChainMsg\t{},{},{},{},{},{}",
                LEFT + 3,
                TOP + 7,
                LEFT + 4,
                TOP + 7,
                LEFT + 5,
                TOP + 7
            ),
        ))
        .unwrap();
    assert!(has_text(&out, "OnLevelEventMsg\t21\t400"));
}

#[test]
fn collectable_fall_updates_goal_progress_and_reaching_quota_stops_the_level() {
    let mut rows = empty_rows();
    rows[7][0] = -i32::from(ORE);
    rows[7][1] = -i32::from(ORE);
    rows[7][2] = -i32::from(ORE_TWO);
    rows[7][3] = -i32::from(ORE_TWO);
    let fixture = Fixture::new(&spec(
        rows,
        vec![goal(0, ORE, 2, 2), goal(1, ORE_TWO, 2, 1)],
    ));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let first = engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, TOP + 7),
        ))
        .unwrap();
    assert!(has_text(&first, "OnQuotaUpdateMsg\t0\t1\t2\t2"));
    assert!(has_text(&first, "OnLevelEventMsg\t21\t500\t3"));
    assert!(!has_text(&first, "OnStopLevel"));
    let second = engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + 1, TOP + 7),
        ))
        .unwrap();
    assert!(has_text(&second, "OnQuotaUpdateMsg\t0\t2\t2\t2"));
    assert!(!has_text(&second, "OnStopLevel"));
    let third = engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + 2, TOP + 7),
        ))
        .unwrap();
    assert!(has_text(&third, "OnQuotaUpdateMsg\t1\t1\t2\t1"));
    assert!(!has_text(&third, "OnStopLevel"));
    let fourth = engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + 3, TOP + 7),
        ))
        .unwrap();
    assert!(has_text(&fourth, "OnQuotaUpdateMsg\t1\t2\t2\t1"));
    assert!(has_text(&fourth, "OnStopLevel\t1\ttrue"));
    assert!(
        !has_text(&fourth, "OnGameWonMsg"),
        "the reward intent waits for the native next-level request"
    );
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::LevelComplete
    );
    let after = engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + 4, TOP + 7),
        ))
        .unwrap();
    assert!(!after.changed);
    assert_eq!(
        engine.state().active.as_ref().unwrap().collected,
        vec![2, 2]
    );
}

#[test]
fn a_fall_that_is_not_the_bottom_row_or_not_a_collectable_is_rejected() {
    let mut rows = empty_rows();
    rows[7][0] = -i32::from(ORE);
    rows[0][1] = 403;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    assert!(engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, TOP),
        ))
        .is_err());
    assert!(engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + 1, TOP + 7),
        ))
        .is_err());
}

#[test]
fn win_grants_one_reconstructed_output_transaction_and_replay_is_monotonic() {
    let mut rows = empty_rows();
    rows[7][0] = -i32::from(ORE);
    let fixture = Fixture::new(&spec_with(
        rows,
        vec![goal(0, ORE, 1, 1)],
        vec![Type8Output {
            value: ORE,
            definition_id: ORE_DEFINITION,
            cluster_quantity: 1,
        }],
    ));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    engine
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, TOP + 7),
        ))
        .unwrap();
    let out = engine
        .handle(&client_payload(id, "OnNextLevelMsg"))
        .unwrap();
    assert!(out.completed);
    let transaction = out.transaction.as_ref().unwrap();
    assert_eq!(transaction.grants, vec![(ORE_DEFINITION, 1)]);
    assert!(transaction.consumes.is_empty());
    assert_eq!(engine.state().completed, 1);
    let duplicate = engine
        .handle(&client_payload(id, "OnNextLevelMsg"))
        .unwrap();
    assert!(duplicate.transaction.is_none());
    assert_eq!(engine.state().completed, 1);
    let over = engine
        .handle(&client_payload(id, "FRServer_GameOver"))
        .unwrap();
    assert!(over.replies.iter().any(|p| p[0..3] == [39, 0, 18]));
    engine
        .handle(&client_payload(id, "FRServer_GameClose"))
        .unwrap();
    assert!(!engine.active());
    let replay = engine.begin(PROFILE).unwrap();
    let instance = engine.state().active.as_ref().unwrap().instance_id;
    assert_eq!(instance, FIRST_INSTANCE + 1);
    assert_eq!(engine.state().next_instance_id, FIRST_INSTANCE + 2);
    assert!(!replay.replies.is_empty());
    assert_eq!(engine.state().completed, 1);
}

#[test]
fn loss_on_timer_expiry_is_authoritative_and_quit_only_cancels_before_a_win() {
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let out = engine.tick(180_000).unwrap();
    assert!(has_text(&out, "OnStopLevel\t1\tfalse"));
    assert_eq!(
        engine.state().active.as_ref().unwrap().phase,
        Type8Phase::Lost
    );
    let quit = engine.handle(&header(6, id, -1)).unwrap();
    assert!(quit.replies.is_empty());
    assert!(engine.active());
    let results = engine
        .handle(&client_payload(id, "FRServer_GameOver"))
        .unwrap();
    let game_over = results
        .replies
        .iter()
        .find(|p| p[0..3] == [39, 0, 18])
        .unwrap();
    assert_eq!(game_over[15], 0);
    assert!(results.replies.iter().any(|p| p[0..3] == [39, 0, 47]));
    assert!(results.replies.iter().any(|p| p[0..3] == [39, 0, 19]));
}

#[test]
fn ordinary_quit_cancels_only_a_pre_win_active_run() {
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let wrong = engine.handle(&header(6, id, MINING_GAME_ID)).unwrap();
    assert!(wrong.replies.is_empty());
    assert!(engine.active());
    let quit = engine.handle(&header(6, id, -1)).unwrap();
    assert!(quit.changed);
    assert!(!engine.active());
    assert!(quit.replies.iter().any(|p| p[0..3] == [39, 0, 19]));
    let again = engine.handle(&header(6, id, -1)).unwrap();
    assert!(again.replies.is_empty());
}

#[test]
fn serialized_state_round_trips_and_resumes_mid_run() {
    let mut rows = empty_rows();
    rows[7][0] = -i32::from(ORE);
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let state = engine.state().clone();
    let encoded = serde_json::to_vec(&state).unwrap();
    let decoded: Type8State = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(state, decoded);
    Type8Engine::validate_state(&decoded).unwrap();
    let mut resumed = fixture.load(Some(decoded));
    assert_eq!(resumed.state(), &state);
    resumed
        .handle(&client_payload(
            id,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, TOP + 7),
        ))
        .unwrap();
    let out = resumed
        .handle(&client_payload(id, "OnNextLevelMsg"))
        .unwrap();
    assert!(out.completed);
    assert_eq!(resumed.state().completed, 1);
}

#[test]
fn older_state_without_mining_keeps_serde_compatibility_and_tampering_is_rejected() {
    // An older gameplay snapshot has no mining record at all. Only this new
    // field stores mining progress, and an absent `active` run is idle state.
    let older = json!({
        "version": 1,
        "next_instance_id": FIRST_INSTANCE,
        "completed": 0,
    });
    let state: Type8State = serde_json::from_value(older).unwrap();
    assert!(state.active.is_none());
    assert_eq!(state, Type8State::default());
    Type8Engine::validate_state(&state).unwrap();
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let engine = fixture.load(None);
    let mut tampered = engine.state().clone();
    tampered.version = 99;
    assert!(Type8Engine::validate_state(&tampered).is_err());
    let mut tampered = engine.state().clone();
    tampered.next_instance_id = 0;
    assert!(Type8Engine::validate_state(&tampered).is_err());
    let mut tampered = engine.state().clone();
    tampered.completed = 2_000_000;
    assert!(Type8Engine::validate_state(&tampered).is_err());
    let mut outside = Type8State::default();
    outside.next_instance_id = 5;
    assert!(Type8Engine::load(
        &fixture.directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        Some(outside),
    )
    .is_err());
}

#[test]
fn verify_reports_are_checked_against_the_board_and_ignored_zero_values() {
    let mut rows = empty_rows();
    rows[2][2] = -404;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let ok = engine
        .handle(&client_payload(
            id,
            &format!(
                "OnVerifyMsg\t{},{},0,{},{},{}",
                LEFT,
                TOP + 2,
                LEFT + 2,
                TOP + 2,
                404
            ),
        ))
        .unwrap();
    assert!(!ok.changed);
    assert!(engine
        .handle(&client_payload(
            id,
            &format!("OnVerifyMsg\t{},{},999", LEFT, TOP + 2),
        ))
        .is_err());
}

#[test]
fn copper_mining_gets_its_own_instance_namespace() {
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let engine = Type8Engine::load(
        &fixture.directory,
        "mining.json",
        COPPER_MINING_GAME_ID,
        91_000_001,
        None,
    );
    assert!(engine.is_err());
    assert_ne!(MINING_GAME_ID, COPPER_MINING_GAME_ID);
    assert_eq!(TILE_COLLECTABLE_START, 600);
}

#[test]
fn default_mining_spec_is_consistent_and_marks_reconstructed_rules() {
    let spec = mining::mining_spec();
    assert_eq!(spec.width, SIDE);
    assert_eq!(spec.height, SIDE);
    assert_eq!(spec.shape.len(), SIDE * SIDE);
    assert!(spec.reconstructed_rules);
    assert!(spec.rule_notice.len() > 10);
    assert!(spec.selectable.len() >= 3);
    assert!(!spec.collectable.is_empty());
    assert!(!spec.goals.is_empty());
    for goal in &spec.goals {
        assert!(spec.collectable.contains(&goal.value));
        assert!(goal.per_reward >= 1);
        assert!(goal.quota >= goal.per_reward);
    }
    assert_eq!(spec.collectable_percent, 40);
    assert_eq!(
        spec.goals.iter().map(|goal| goal.quota).collect::<Vec<_>>(),
        [6, 6]
    );
    assert_eq!(spec.outputs.len(), 2);
    let fixture = Fixture::new(&spec);
    let engine = fixture.load(None);
    assert!(!engine.active());
    assert_eq!(engine.spec().goals.len(), spec.goals.len());
}

#[test]
fn native_client_coordinate_lists_tolerate_one_trailing_comma() {
    // The recovered client appends "x,y," per tile (`core/Grid.doMatch3`,
    // `core/Grid.UpdateIdle`, `core/Grid.VerifyGridWithServer`) and the
    // recovered server drops exactly that single empty tail
    // (`GameLevel.OnChainMsg` splits and pops). All three native lists must
    // parse; any other empty field stays invalid.
    let mut rows = empty_rows();
    rows[5][0] = -401;
    rows[5][1] = -401;
    rows[5][2] = -401;
    let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    enter_playing(&mut engine);
    let id = FIRST_INSTANCE;
    let chain = engine
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
    assert!(has_text(&chain, "OnLevelEventMsg\t21\t400"));
    assert_eq!(engine.state().active.as_ref().unwrap().chains, 1);
    assert!(
        engine
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
            .unwrap()
            .changed
    );
    assert!(
        !engine
            .handle(&client_payload(
                id,
                &format!("OnVerifyMsg\t{},{},0,", LEFT, TOP + 5),
            ))
            .unwrap()
            .changed
    );
    let before = engine.state().clone();
    for invalid in [
        "OnChainMsg\t3,8,,4,8,5,8,",
        "OnChainMsg\t3,8,4,8,5,8,,",
        "OnVerifyMsg\t3,8,0,,",
    ] {
        assert!(
            engine.handle(&client_payload(id, invalid)).is_err(),
            "only the single recovered tail is tolerated: {invalid}"
        );
        assert_eq!(before, *engine.state());
    }
}

#[test]
fn content_bounds_align_with_the_run_state_bounds() {
    // 600 s is the largest loadable timer; 601 s is refused at load instead of
    // producing a run that rejects every message. The same constant bounds the
    // chain time bonus.
    let mut duration = spec(empty_rows(), vec![goal(0, ORE, 1, 1)]);
    duration.duration_seconds = 601;
    let refused = Fixture::new(&duration);
    assert!(Type8Engine::load(
        &refused.directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        None,
    )
    .is_err());
    duration.duration_seconds = 600;
    let accepted = Fixture::new(&duration);
    let mut engine = Type8Engine::load(
        &accepted.directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        None,
    )
    .unwrap();
    engine.begin(PROFILE).unwrap();
    assert_eq!(
        engine.state().active.as_ref().unwrap().remaining_ms,
        600_000
    );

    // Every loadable shape code must be executable by `initialize_board` and
    // accepted by the run validator.
    for (code, loadable) in [
        (200, false),
        (500, false),
        (700, false),
        (-101, false),
        (-1002, false),
        (400, true),
        (600, true),
        (-200, true),
        (-401, true),
        (-601, true),
    ] {
        let mut rows = empty_rows();
        rows[0][0] = code;
        let fixture = Fixture::new(&spec(rows, vec![goal(0, ORE, 1, 1)]));
        let loaded = Type8Engine::load(
            &fixture.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            None,
        );
        assert_eq!(
            loaded.is_ok(),
            loadable,
            "shape code {code} loadability must match the engine's tile factory"
        );
    }

    // 448 cells is the largest board the run state holds; 32x32 is refused.
    let oversized = {
        let mut s = spec(empty_rows(), vec![goal(0, ORE, 1, 1)]);
        s.width = 32;
        s.height = 32;
        s.shape = vec![1; 32 * 32];
        s
    };
    let fixture = Fixture::new(&oversized);
    assert!(Type8Engine::load(
        &fixture.directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        None,
    )
    .is_err());
    let tall = {
        let mut s = spec(empty_rows(), vec![goal(0, ORE, 1, 1)]);
        s.height = 32;
        let mut shape = vec![0_i32; SIDE * 32];
        for y in 1..=30 {
            for x in LEFT..LEFT + PLAYABLE {
                shape[y * SIDE + x] = 1;
            }
        }
        for x in LEFT..LEFT + PLAYABLE {
            shape[31 * SIDE + x] = 4;
        }
        s.shape = shape;
        s
    };
    let fixture = Fixture::new(&tall);
    let engine = Type8Engine::load(
        &fixture.directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        None,
    )
    .unwrap();
    assert!(!engine.active());
}

#[test]
fn launch_and_preload_names_default_to_the_recovered_mining_assets() {
    // Older mining.json files carry neither field; the recovered mining
    // resource names must stay the default.
    let fixture = Fixture::new(&spec(empty_rows(), vec![goal(0, ORE, 1, 1)]));
    let mut engine = fixture.load(None);
    assert_eq!(engine.preload_file(), "MiningAssets.txt");
    assert_eq!(engine.launch_movie(), "game_mining.gfx");
    let info = engine.begin(PROFILE).unwrap().replies.remove(0);
    assert!(contains(&info, b"MiningAssets.txt"));
    assert!(!contains(&info, b"FarmingAssets.txt"));
    let start = engine.handle(&header(5, FIRST_INSTANCE, -1)).unwrap();
    let launch = start
        .replies
        .iter()
        .find(|p| p.len() >= 4 && p[0..2] == 26u16.to_le_bytes() && p[2..4] == 12u16.to_le_bytes())
        .expect("26/12 launch packet");
    assert!(contains(launch, b"game_mining.gfx"));
    assert!(!contains(launch, b"game_farming.gfx"));
}

#[test]
fn launch_names_are_selectable_and_orphan_outputs_are_refused() {
    // Explicit launch configuration wins over the mining defaults.
    let directory = std::env::temp_dir().join(format!(
        "osfr-mining-launch-{}-{}",
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
        "preload_file": "FarmingAssets.txt",
        "launch_movie": "game_farming.gfx",
        "spec": spec(empty_rows(), vec![goal(0, ORE, 1, 1)]),
    });
    fs::write(
        directory.join("mining.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    let mut engine = Type8Engine::load(
        &directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_INSTANCE,
        None,
    )
    .unwrap();
    assert_eq!(engine.preload_file(), "FarmingAssets.txt");
    assert_eq!(engine.launch_movie(), "game_farming.gfx");
    let info = engine.begin(PROFILE).unwrap().replies.remove(0);
    assert!(contains(&info, b"FarmingAssets.txt"));
    assert!(!contains(&info, b"MiningAssets.txt"));
    let start = engine.handle(&header(5, FIRST_INSTANCE, -1)).unwrap();
    assert!(start
        .replies
        .iter()
        .any(|p| contains(p, b"game_farming.gfx")));
    assert!(!start
        .replies
        .iter()
        .any(|p| contains(p, b"game_mining.gfx")));
    engine.handle(&header(6, FIRST_INSTANCE, -1)).unwrap();
    let _ = fs::remove_file(directory.join("mining.json"));
    let _ = fs::remove_dir(&directory);

    // Path-like launch names fail the load closed.
    let path_like = {
        let mut s = spec(empty_rows(), vec![goal(0, ORE, 1, 1)]);
        s.rule_notice = "Synthetic private fixture.".into();
        s
    };
    let directory = std::env::temp_dir().join(format!(
        "osfr-mining-launch-bad-{}-{}",
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
        "launch_movie": "../game_mining.gfx",
        "spec": path_like,
    });
    fs::write(
        directory.join("mining.json"),
        serde_json::to_vec(&content).unwrap(),
    )
    .unwrap();
    assert!(
        Type8Engine::load(
            &directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            None,
        )
        .is_err(),
        "path-like launch names must be refused"
    );
    let _ = fs::remove_file(directory.join("mining.json"));
    let _ = fs::remove_dir(&directory);

    // An output whose tile value has no goal would silently lose its grant;
    // the proposal is refused at load instead.
    let orphan = spec_with(
        empty_rows(),
        vec![goal(0, ORE, 1, 1)],
        vec![Type8Output {
            value: ORE_TWO,
            definition_id: ORE_DEFINITION,
            cluster_quantity: 1,
        }],
    );
    let fixture = Fixture::new(&orphan);
    assert!(
        Type8Engine::load(
            &fixture.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_INSTANCE,
            None,
        )
        .is_err(),
        "an output without a matching goal must be refused, not silently dropped"
    );
}
