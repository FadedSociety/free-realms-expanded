//! Harvesting tile registration regression: `OnTileInfoMsg` per-kind index.
//!
//! ORIGINAL evidence (decompiled farming script package; hashes in
//! `lab/reports/harvesting-mouse-driver-20261009-1337Z/hashes.txt`):
//! * `core/TileGenerator.as::Initialize` fills `aNames` for empty (0), wildcard (200),
//!   blocker (800), hazard-lock (301), the powerups (900..953), the lootdrops
//!   (1000..1002) and the dropzone (100); it names neither 401..404 nor 601..604.
//! * `core/TileGenerator.as::SetTileInfo(value, index)` is the only writer of those
//!   entries. It derives the branch as `Math.floor(value / 100) * 100` and stores
//!   `"SelectableTile" + index` for selectables and `aCollectableNames[index * 2]` for
//!   collectables, so the server's second field is the one-based kind index.
//! * `core/TileGenerator.as::generate_selectable` dereferences `aNames[value]`, so a value
//!   the server spawns but never registers cannot be drawn.
//! The shared engine emits `OnTileInfoMsg\t<value>\t<value - category>` for every kind in
//! the content tile list, so that list must register all four recovered regional produce
//! values; the goal value is the same `400 + type` selectable value and the frame index is
//! the produce type (`farmingValue401..416` / `SelectableTile1..16`).
//!
//! Native RED (run `client-baseline-20261009T202011Z`): the regional specs advertised
//! `selectable = [401..404]` while the goals and the tile list carried 601..604, so the
//! baskets resolved to the client placeholder `##farmingValue60…` with blank produce icons.
//!
//! Fixtures are synthetic and written to a private temp directory; no shared content, save,
//! process or UI file is touched.

use osfr_tools::{
    harvesting::{
        CHEF_PROFILE_ID, CONTENT_FILE, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE,
        FIRST_INSTANCE_ID, HarvestContent, HarvestError, RECONSTRUCTED_BUNDLES_PER_CLUSTER,
        RECONSTRUCTED_COLLECTABLE_PERCENT, RECONSTRUCTED_DURATION_SECONDS,
        RECONSTRUCTED_PER_REWARD_TILES, RECONSTRUCTED_POWERUP_PERCENT, RECONSTRUCTED_QUOTA_TILES,
        RECONSTRUCTED_SEED, RECONSTRUCTED_WILDCARD_PERCENT, RECOVERED_REGION_GAME_IDS,
        SCHEMA_VERSION, TILE_SELECTABLE_CATEGORY, Type8Kind, content, produce, produce_value_for,
        region, validate_content, write_content,
    },
    mining::Type8Engine,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const PROFILE: i32 = CHEF_PROFILE_ID;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-harvesting-tiles-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn write(&self, game_id: i32) -> std::io::Result<PathBuf> {
        write_content(&self.directory, game_id).map_err(std::io::Error::other)
    }

    fn load(&self, game_id: i32) -> std::io::Result<Type8Engine> {
        Type8Engine::load(
            &self.directory,
            CONTENT_FILE,
            game_id,
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

/// Recovered produce values of one region, in board order.
fn selectable_values(game_id: i32) -> Vec<u16> {
    region(game_id)
        .expect("recovered region row")
        .produce
        .iter()
        .map(|slug| produce_value_for(slug).expect("recovered produce value"))
        .collect()
}

/// Registered `(value, frame)` pairs, in the order the engine emits them.
fn expected_pairs(game_id: i32) -> Vec<(u16, u16)> {
    selectable_values(game_id)
        .into_iter()
        .map(|value| (value, value - TILE_SELECTABLE_CATEGORY))
        .collect()
}

fn refusal(document: &HarvestContent, needle: &str) {
    match validate_content(document) {
        Err(HarvestError::Content(message)) => assert!(
            message.contains(needle),
            "expected a refusal naming {needle:?}, saw {message:?}"
        ),
        other => panic!("expected a content refusal naming {needle:?}, saw {other:?}"),
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

fn payload_text(packet: &[u8]) -> Option<String> {
    if packet.len() < 19 || packet[0..3] != [39, 0, 14] {
        return None;
    }
    let length = i32::from_le_bytes(packet[15..19].try_into().unwrap());
    assert!(length >= 1);
    let end = 19 + length as usize - 1;
    assert_eq!(packet.len(), end + 1);
    assert_eq!(packet[end], 0);
    String::from_utf8(packet[19..end].to_vec()).ok()
}

/// Drives a fresh engine to `OnStartLevelMsg` and returns that frame's replies.
fn start_level(engine: &mut Type8Engine) -> std::io::Result<Vec<Vec<u8>>> {
    let id = FIRST_INSTANCE_ID;
    engine.begin(PROFILE)?;
    engine.handle(&header(5, id, -1))?;
    engine.handle(&client_payload(id, "OnConnectMsg"))?;
    engine.handle(&client_payload(id, "OnStartGameMsg"))?;
    let out = engine.handle(&client_payload(id, "OnStartLevelMsg"))?;
    Ok(out.replies)
}

fn tile_info_pairs(replies: &[Vec<u8>]) -> Vec<(u16, u16)> {
    let mut pairs = Vec::new();
    for reply in replies {
        let Some(text) = payload_text(reply) else {
            continue;
        };
        let Some(rest) = text.strip_prefix("OnTileInfoMsg\t") else {
            continue;
        };
        let fields: Vec<&str> = rest.split('\t').collect();
        assert_eq!(fields.len(), 2, "malformed OnTileInfoMsg: {text:?}");
        pairs.push((fields[0].parse().unwrap(), fields[1].parse().unwrap()));
    }
    pairs
}

#[test]
fn regional_specs_register_each_recovered_tile_value_once() {
    for game_id in RECOVERED_REGION_GAME_IDS {
        let document = content(game_id).expect("regional content builds");
        let values: Vec<u16> = document.spec.tiles.iter().map(|kind| kind.value).collect();
        assert_eq!(
            values,
            selectable_values(game_id),
            "game {game_id} registers exactly its recovered produce values"
        );
        assert_eq!(document.spec.tiles.len(), 4, "game {game_id} kind count");
        for kind in &document.spec.tiles {
            assert_eq!(
                kind.category,
                (kind.value / 100) * 100,
                "game {game_id} kind {} carries the recovered category",
                kind.value
            );
        }
        assert_eq!(document.spec.selectable, selectable_values(game_id));
        assert!(document.spec.collectable.is_empty());
        validate_content(&document).expect("regional content validates");
    }
}

#[test]
fn start_level_wire_registers_the_produce_frame_indices() {
    for game_id in RECOVERED_REGION_GAME_IDS {
        let fixture = Fixture::new();
        fixture.write(game_id).expect("content writes");
        let mut engine = fixture
            .load(game_id)
            .expect("engine loads the written content");
        let replies = start_level(&mut engine).expect("level start emits replies");
        let registered = tile_info_pairs(&replies);
        assert_eq!(
            registered,
            expected_pairs(game_id),
            "game {game_id} OnTileInfoMsg set"
        );
        for (value, index) in &registered {
            assert!(
                (1..=16).contains(index),
                "game {game_id} kind {value} must carry its SelectableTile frame, saw {index}"
            );
            assert_eq!(*index, value - TILE_SELECTABLE_CATEGORY);
        }
    }
}

#[test]
fn validate_refuses_a_missing_regular_mapping() {
    let mut document = content(111).expect("region 111 content");
    document.spec.tiles.retain(|kind| kind.value != 411);
    refusal(&document, "411");
}

#[test]
fn validate_refuses_a_duplicate_regular_mapping() {
    let mut document = content(111).expect("region 111 content");
    let duplicate = document.spec.tiles[0];
    document.spec.tiles.push(duplicate);
    refusal(&document, "duplicate");
    refusal(&document, "411");
}

#[test]
fn validate_refuses_a_wrong_category_regular_mapping() {
    let mut document = content(111).expect("region 111 content");
    document.spec.tiles[0].category = 600;
    refusal(&document, "411");
    refusal(&document, "implies");
}

#[test]
fn validate_refuses_a_conflicting_tile_kind() {
    // A 600-block value claiming the selectable category (the unnamed family must
    // never come back as a farming basket).
    let mut conflicting = content(111).expect("region 111 content");
    conflicting.spec.tiles.push(Type8Kind {
        value: 601,
        category: TILE_SELECTABLE_CATEGORY,
        icon_id: 0,
    });
    refusal(&conflicting, "601");
    refusal(&conflicting, "implies");

    // A kind that belongs to neither recovered harvesting set.
    let mut outsider = content(111).expect("region 111 content");
    outsider.spec.tiles.push(Type8Kind {
        value: 505,
        category: 500,
        icon_id: 0,
    });
    refusal(&outsider, "505");
    refusal(&outsider, "neither selectable nor collectable");
}

#[test]
fn regional_specs_preserve_recovered_goals_outputs_and_settings() {
    for game_id in RECOVERED_REGION_GAME_IDS {
        let document = content(game_id).expect("regional content builds");
        let recovered = region(game_id).expect("recovered region row");
        assert_eq!(document.schema_version, SCHEMA_VERSION);
        assert_eq!(document.game_id, game_id);
        assert_eq!(document.profile_id, CHEF_PROFILE_ID);
        assert_eq!(document.preload_file, FARMING_PRELOAD_FILE);
        assert_eq!(document.launch_movie, FARMING_LAUNCH_MOVIE);
        assert_eq!(document.spec.goals.len(), recovered.produce.len());
        assert_eq!(document.spec.outputs.len(), recovered.produce.len());
        for (slot, slug) in recovered.produce.iter().enumerate() {
            let pinned = produce(slug).expect("pinned produce row");
            let value = produce_value_for(slug).expect("recovered produce value");
            assert_eq!(document.spec.goals[slot].index, slot as i32);
            assert_eq!(document.spec.goals[slot].value, value);
            assert_eq!(document.spec.goals[slot].quota, RECONSTRUCTED_QUOTA_TILES);
            assert_eq!(
                document.spec.goals[slot].per_reward,
                RECONSTRUCTED_PER_REWARD_TILES
            );
            assert_eq!(document.spec.goals[slot].icon_id, pinned.icon_id);
            assert_eq!(document.spec.outputs[slot].value, value);
            assert_eq!(
                document.spec.outputs[slot].definition_id,
                pinned.item_definition_id
            );
            assert_eq!(
                document.spec.outputs[slot].cluster_quantity,
                RECONSTRUCTED_BUNDLES_PER_CLUSTER
            );
        }
        assert!(document.spec.collectable.is_empty());
        assert_eq!(document.spec.selectable, selectable_values(game_id));
        assert_eq!(
            document.spec.duration_seconds,
            RECONSTRUCTED_DURATION_SECONDS
        );
        assert_eq!(document.spec.seed, RECONSTRUCTED_SEED);
        assert_eq!(
            document.spec.wildcard_percent,
            RECONSTRUCTED_WILDCARD_PERCENT
        );
        assert_eq!(
            document.spec.collectable_percent,
            RECONSTRUCTED_COLLECTABLE_PERCENT
        );
        assert_eq!(document.spec.powerup_percent, RECONSTRUCTED_POWERUP_PERCENT);
        assert!(document.spec.reconstructed_rules);
        assert!(!document.spec.rule_notice.trim().is_empty());
    }
}

#[test]
fn writer_emits_the_registered_kinds_for_every_region() {
    for game_id in RECOVERED_REGION_GAME_IDS {
        let fixture = Fixture::new();
        let path = fixture.write(game_id).expect("writer succeeds");
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let values: Vec<u16> = value["spec"]["tiles"]
            .as_array()
            .expect("written tiles array")
            .iter()
            .map(|kind| kind["value"].as_u64().unwrap() as u16)
            .collect();
        assert_eq!(
            values,
            selectable_values(game_id),
            "written game {game_id} tile values"
        );
        let engine = fixture
            .load(game_id)
            .expect("engine loads the written content");
        assert_eq!(engine.spec().tiles.len(), 4);
        assert_eq!(engine.spec().selectable, selectable_values(game_id));
        assert!(engine.spec().collectable.is_empty());
    }
}

/// Optional private export for the root's versioned content packaging: the six freshly
/// written regional documents plus the `OnTileInfoMsg` pairs each one produces. Skips
/// silently when `OSFR_HARVESTING_TILE_EVIDENCE_DIR` is unset.
#[test]
fn evidence_export_writes_the_fixed_regional_content() {
    let Some(directory) = std::env::var_os("OSFR_HARVESTING_TILE_EVIDENCE_DIR").map(PathBuf::from)
    else {
        return;
    };
    fs::create_dir_all(&directory).unwrap();
    let mut summary = Vec::new();
    for game_id in RECOVERED_REGION_GAME_IDS {
        let fixture = Fixture::new();
        let path = fixture.write(game_id).expect("writer succeeds");
        let document = fs::read(&path).unwrap();
        fs::write(
            directory.join(format!("harvesting-{game_id}.json")),
            &document,
        )
        .unwrap();
        let mut engine = fixture
            .load(game_id)
            .expect("engine loads the written content");
        let replies = start_level(&mut engine).expect("level start emits replies");
        let registered = tile_info_pairs(&replies);
        assert_eq!(registered, expected_pairs(game_id), "game {game_id} pairs");
        summary.push(serde_json::json!({
            "game_id": game_id,
            "sha256": format!("{:x}", Sha256::digest(&document)),
            "tiles": registered
                .iter()
                .map(|(value, index)| serde_json::json!({"value": value, "index": index}))
                .collect::<Vec<_>>(),
        }));
    }
    fs::write(
        directory.join("tile-registration.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "driver": "harvesting_tile_registration",
            "produce_values": "400 + 1-based produce type (farmingValue401..416)",
            "regions": summary,
        }))
        .unwrap(),
    )
    .unwrap();
}
