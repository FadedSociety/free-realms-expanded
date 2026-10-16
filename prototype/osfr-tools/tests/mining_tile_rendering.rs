//! Mining18 tile rendering regression (`OnTileInfoMsg` per-kind index).
//!
//! Native run `client-baseline-20261009T121834Z` captured
//! `OnTileInfoMsg\t401\t400` … `\t405\t400` and `\t601\t600`/`\t602\t600`
//! (frames 003562–003568). The client resolves the second field as the tile-kind
//! index (`"SelectableTile" + index`, `aCollectableNames[index * 2]`), so the
//! category left every regular tile on the combined clip's default frame.
//! These tests pin the recovered index (`value - category`) and the refusal for
//! content whose value sits below its category.
//!
//! Fixtures are synthetic and private.
use osfr_tools::mining::{
    FIRST_MINING_INSTANCE, MINING_GAME_ID, Type8BoardSpec, Type8Engine, Type8Run, Type8State,
    mining_spec,
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const PROFILE: i32 = 14;
const FIRST_INSTANCE: i32 = 18_000_001;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(spec: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-mining-tiles-{}-{}",
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

    fn load(&self) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            "mining.json",
            MINING_GAME_ID,
            FIRST_MINING_INSTANCE,
            None,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.directory.join("mining.json"));
        let _ = fs::remove_dir(&self.directory);
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

/// Client `39/14` payload: current instance, two `-1` sentinels, length + text + NUL.
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
    String::from_utf8(packet[19..end].to_vec()).ok()
}

/// Drives a fresh engine to `OnStartLevelMsg` and returns that frame's replies.
fn start_level(engine: &mut Type8Engine) -> std::io::Result<Vec<Vec<u8>>> {
    let id = FIRST_INSTANCE;
    engine.begin(PROFILE).unwrap();
    engine.handle(&header(5, id, -1)).unwrap();
    engine.handle(&client_payload(id, "OnConnectMsg")).unwrap();
    engine
        .handle(&client_payload(id, "OnStartGameMsg"))
        .unwrap();
    let out = engine.handle(&client_payload(id, "OnStartLevelMsg"))?;
    Ok(out.replies)
}

fn tile_info(replies: &[Vec<u8>]) -> Vec<String> {
    replies
        .iter()
        .filter_map(|packet| payload_text(packet))
        .filter(|text| text.starts_with("OnTileInfoMsg"))
        .collect()
}

#[test]
fn initialize_board_sends_the_recovered_tile_index() {
    let fixture = Fixture::new(&mining_spec());
    let mut engine = fixture.load().unwrap();
    let replies = start_level(&mut engine).unwrap();

    assert_eq!(
        tile_info(&replies),
        vec![
            "OnTileInfoMsg\t401\t1",
            "OnTileInfoMsg\t402\t2",
            "OnTileInfoMsg\t403\t3",
            "OnTileInfoMsg\t404\t4",
            "OnTileInfoMsg\t405\t5",
            "OnTileInfoMsg\t601\t1",
            "OnTileInfoMsg\t602\t2",
        ],
        "OnTileInfoMsg must carry the per-kind index (value - category), not the category"
    );
}

#[test]
fn tile_index_never_underflows_on_malformed_content() {
    let mut spec = mining_spec();
    spec.tiles[0].value = 399; // currently permitted content: value below its category
    let fixture = Fixture::new(&spec);
    // The malformed kind still loads (validate_spec does not police spec.tiles)…
    let mut engine = fixture.load().unwrap();
    // …and the board build refuses it instead of wrapping, panicking or emitting a bad index.
    let error = start_level(&mut engine).unwrap_err();
    assert!(
        error.to_string().contains("below its category"),
        "unexpected error: {error}"
    );
}

#[test]
fn tile_index_fix_keeps_run_state_and_snapshot_bytes() {
    let fixture = Fixture::new(&mining_spec());
    let mut engine = fixture.load().unwrap();
    let _ = start_level(&mut engine).unwrap();

    let state: Type8State = engine.state().clone();
    assert_eq!(state.version, 1);
    assert_eq!(state.next_instance_id, FIRST_INSTANCE + 1);
    let text = serde_json::to_vec(&state).unwrap();
    let round_trip: Type8State = serde_json::from_slice(&text).unwrap();
    assert_eq!(round_trip, state);
    let run: &Type8Run = state.active.as_ref().unwrap();
    assert_eq!(run.instance_id, FIRST_INSTANCE);
    assert_eq!(run.board.len(), 14 * 14);
    assert!(run.board.iter().any(|value| (401..=405).contains(value)));
}
