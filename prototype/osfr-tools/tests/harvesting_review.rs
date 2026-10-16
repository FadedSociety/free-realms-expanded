//! Independent post-repair review execution for Harvesting (TYPE8 piece 002).
//!
//! Read-only with respect to production: both the harvesting module and the
//! shared type-8 engine are path-included and driven as real code. These tests
//! execute every supported harvesting spec through the engine loader,
//! handshake and reward path instead of only rendering JSON.
use osfr_tools::{harvesting, mining};

use mining::{Type8Engine, Type8Phase, Type8Run, Type8State};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const GAP_PROFILES: [i32; 8] = [58, 110, 112, 114, 116, 118, 120, 217];

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "osfr-harvesting-review-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn header(sub: u8, id: i32, game: i32) -> Vec<u8> {
    let mut b = mining::TYPE8_GAME_FAMILY.to_le_bytes().to_vec();
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
    Some(String::from_utf8(packet[19..end].to_vec()).unwrap())
}

fn has_text(outcome: &mining::Type8Outcome, prefix: &str) -> bool {
    outcome
        .replies
        .iter()
        .filter_map(|p| payload_text(p))
        .any(|t| t == prefix || t.starts_with(&format!("{prefix}\t")))
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Every supported regional spec must load, bind its own content ids, request
/// the original farming movie/preload list and reach the Playing phase.
#[test]
fn every_supported_region_spec_executes_and_requests_farming_assets() {
    let scratch = Scratch::new("regions");
    for game_id in harvesting::RECOVERED_REGION_GAME_IDS {
        let dir = scratch.0.join(game_id.to_string());
        fs::create_dir_all(&dir).unwrap();
        let path = harvesting::write_content(&dir, game_id)
            .unwrap_or_else(|e| panic!("game {game_id} content: {e}"));
        assert_eq!(path.file_name().unwrap(), harvesting::CONTENT_FILE);

        let mut engine = Type8Engine::load(
            &dir,
            harvesting::CONTENT_FILE,
            game_id,
            harvesting::FIRST_INSTANCE_ID,
            None,
        )
        .unwrap_or_else(|e| panic!("game {game_id} rejected by the shared engine: {e}"));

        let info = engine
            .begin(harvesting::CHEF_PROFILE_ID)
            .unwrap()
            .replies
            .remove(0);
        let region = harvesting::region(game_id).unwrap();
        assert!(
            contains(&info, &region.name_id.to_le_bytes()),
            "game {game_id} info packet must carry name id {}",
            region.name_id
        );
        assert!(
            contains(&info, &region.description_id.to_le_bytes()),
            "game {game_id} info packet must carry description id {}",
            region.description_id
        );
        assert!(
            contains(&info, b"FarmingAssets.txt"),
            "game {game_id} must request the recovered farming preload list"
        );
        assert!(
            !contains(&info, b"MiningAssets.txt"),
            "game {game_id} must not advertise the mining preload list"
        );

        let first_instance = harvesting::FIRST_INSTANCE_ID;
        let start = engine.handle(&header(5, first_instance, -1)).unwrap();
        let launch = start
            .replies
            .iter()
            .find(|p| {
                p.len() >= 4 && p[0..2] == 26u16.to_le_bytes() && p[2..4] == 12u16.to_le_bytes()
            })
            .unwrap_or_else(|| panic!("game {game_id} did not emit a 26/12 launch"));
        assert!(
            contains(launch, b"game_farming.gfx"),
            "game {game_id} must launch the original farming movie"
        );
        assert!(
            !contains(launch, b"game_mining.gfx"),
            "game {game_id} must not launch the mining movie"
        );

        engine
            .handle(&client_payload(first_instance, "OnConnectMsg"))
            .unwrap();
        engine
            .handle(&client_payload(first_instance, "OnStartGameMsg"))
            .unwrap();
        engine
            .handle(&client_payload(first_instance, "OnStartLevelMsg"))
            .unwrap();
        assert_eq!(
            engine.state().active.as_ref().unwrap().phase,
            Type8Phase::Playing,
            "game {game_id} must reach the playing phase"
        );
        let spec = engine.spec();
        assert_eq!(
            spec.goals.iter().map(|g| g.value).collect::<Vec<_>>(),
            region
                .produce
                .iter()
                .map(|slug| harvesting::produce_value_for(slug).unwrap())
                .collect::<Vec<u16>>(),
            "game {game_id} goals stay the recovered produce values (farmingValue401..416)"
        );
        let expected: Vec<i32> = region
            .produce
            .iter()
            .map(|slug| harvesting::produce(slug).unwrap().item_definition_id)
            .collect();
        assert_eq!(
            spec.outputs
                .iter()
                .map(|o| o.definition_id)
                .collect::<Vec<_>>(),
            expected,
            "game {game_id} bundle ids stay bound to the recovered produce order"
        );
    }
}

/// Gap profiles carry no recovered produce set, so they must refuse to build
/// or write a spec at all rather than emit an engine-refused configuration.
#[test]
fn gap_profiles_refuse_unsupported_launch() {
    let scratch = Scratch::new("gaps");
    for game_id in GAP_PROFILES {
        let board = harvesting::board_spec(game_id);
        assert!(board.is_err(), "game {game_id} must not propose a board");
        let content = harvesting::content(game_id);
        assert!(content.is_err(), "game {game_id} must not build content");
        let message = content.unwrap_err().to_string();
        assert!(
            message.contains("unsupported") || message.contains("unrecovered"),
            "game {game_id} refusal must be explicit, got {message:?}"
        );
        let dir = scratch.0.join(game_id.to_string());
        fs::create_dir_all(&dir).unwrap();
        assert!(
            harvesting::write_content(&dir, game_id).is_err(),
            "game {game_id} must not write a content file"
        );
        assert!(
            !dir.join(harvesting::CONTENT_FILE).exists(),
            "game {game_id} must leave no launchable harvesting.json"
        );
    }
    assert!(
        harvesting::content(9999).is_err(),
        "unknown ids stay refused"
    );
}

/// The executable board must satisfy the engine's rectangle + dropzone grammar
/// and label the fidelity gap against the screenshot-observed staggered board.
#[test]
fn emitted_geometry_is_engine_compatible_and_labels_the_fidelity_gap() {
    for game_id in harvesting::RECOVERED_REGION_GAME_IDS {
        let spec = harvesting::board_spec(game_id).unwrap();
        let width = spec.width as usize;
        let height = spec.height as usize;
        assert_eq!(
            spec.shape.len(),
            width * height,
            "game {game_id} shape size"
        );
        assert_eq!(
            height, 11,
            "game {game_id} keeps the padded reconstructed grid height"
        );
        assert_eq!(
            width, 11,
            "game {game_id} keeps the padded reconstructed grid width"
        );
        assert!(
            spec.shape[(height - 1) * width + 1..]
                .iter()
                .all(|code| *code == 4),
            "game {game_id} needs a full dropzone row over the playable columns"
        );
        assert!(
            spec.shape[0..width].iter().all(|code| *code == 0),
            "game {game_id} keeps row 0 inert for the original fall loops"
        );
        for row in 0..height {
            assert_eq!(
                spec.shape[row * width],
                0,
                "game {game_id} keeps column 0 inert for the original fall loops"
            );
        }
        assert!(
            (1..=height - 2)
                .all(|row| (1..width).all(|column| spec.shape[row * width + column] == 1)),
            "game {game_id} keeps the 10x9 padded playable rectangle"
        );
        let observed = harvesting::diamond_shape(10, 10);
        assert_eq!(
            observed.iter().filter(|code| **code != 0).count(),
            76,
            "the screenshot-observed staggered artifact stays recorded"
        );
        let notice = spec.rule_notice.to_lowercase();
        assert!(
            notice.contains("screenshot"),
            "game {game_id} notice must keep the screenshot evidence reference"
        );
        assert!(
            notice.contains("fidelity gap"),
            "game {game_id} notice must label the rectangular fidelity gap"
        );
    }
}

/// Serialized restart / reward path: resume a completed harvesting run from a
/// persisted state and check the engine grants the recovered bundle counts.
#[test]
fn harvested_bundles_grant_once_from_a_serialized_completion_state() {
    let scratch = Scratch::new("reward");
    let dir = scratch.0.join("111");
    fs::create_dir_all(&dir).unwrap();
    harvesting::write_content(&dir, 111).unwrap();
    let spec = harvesting::board_spec(111).unwrap();
    let width = spec.width as usize;
    let height = spec.height as usize;
    let instance_id = harvesting::FIRST_INSTANCE_ID;
    let mut board = vec![0u16; width * height];
    for (slot, goal) in spec.goals.iter().enumerate() {
        board[(height - 2) * width + slot] = goal.value;
    }
    let run = Type8Run {
        instance_id,
        phase: Type8Phase::LevelComplete,
        board,
        idle: vec![true; width * height],
        tile_queue: Vec::new(),
        random_queue: Vec::new(),
        rng: 1,
        collected: spec.goals.iter().map(|goal| goal.quota).collect(),
        score: 12_345,
        remaining_ms: 30_000,
        wall_ms: 60_000,
        paused: false,
        chains: 4,
        reshuffles: 0,
        end_sent: false,
    };
    let state = Type8State {
        version: 1,
        next_instance_id: instance_id + 1,
        completed: 0,
        active: Some(run),
    };
    let mut engine = Type8Engine::load(
        &dir,
        harvesting::CONTENT_FILE,
        111,
        harvesting::FIRST_INSTANCE_ID,
        Some(state),
    )
    .expect("persisted completion state must resume");
    let out = engine
        .handle(&client_payload(instance_id, "OnNextLevelMsg"))
        .unwrap();
    assert!(out.completed);
    let transaction = out
        .transaction
        .as_ref()
        .expect("completed harvest must grant");
    assert!(transaction.consumes.is_empty());
    // Reconstructed quota 25 tiles / 5 tiles per bundle = 5 bundles per goal.
    assert_eq!(
        transaction.grants,
        vec![(2885, 5), (2887, 5), (2890, 5), (2877, 5)]
    );
    assert!(has_text(&out, "OnGameWonMsg"));
    let replay = engine
        .handle(&client_payload(instance_id, "OnNextLevelMsg"))
        .unwrap();
    assert!(replay.transaction.is_none(), "replay must not grant twice");
    assert_eq!(engine.state().completed, 1);
}
