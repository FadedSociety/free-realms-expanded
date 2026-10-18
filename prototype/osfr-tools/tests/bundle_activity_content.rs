//! T026: the playtest bundle's frozen activity content loads without `lab`.
//!
//! The temp directory below holds only files copied from `data/content`. The
//! loaders read that directory; none of them falls back to a lab package. The
//! hash pins protect the exact bytes because `kart_entry.json` and
//! `encounters.json` hashes are also embedded in saved character state.

use osfr_tools::{
    encounter::EncounterEngine,
    fishing_service::FishingSpots,
    harvest_nodes::GatheringNodes,
    harvesting::{
        self, CHEF_PROFILE_ID, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE, HarvestContent,
    },
    kart_gateway::KartGateway,
    kart_lobby_native::{self, LobbyNativeBody},
    mining::{self, FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, Type8Engine},
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

const FILES: &[&str] = &[
    "encounters.json",
    "encounter-template.bin",
    "kart_entry.json",
    "kart_lobby.json",
    "kart_gates.json",
    "mining.json",
    "harvesting.json",
    "fishing_spots.json",
    "mining_nodes.json",
    "harvest_nodes.json",
];

/// The reviewed T017 playtest bytes copied into `data/content` on 2026-10-17.
const FROZEN_SHA256: &[(&str, &str)] = &[
    (
        "encounters.json",
        "2d465c7e70edcd725345f512c26883fb0d31124e2af89d1b103e432927b5d9a4",
    ),
    (
        "encounter-template.bin",
        "602c66b1a4ea9fac97eaec2d64c380bee17c33ff86e0aa9fe0b10c1da3bbfc19",
    ),
    (
        "kart_entry.json",
        "b01169529a08a9314dc921860fa9c543eeb7772ee64f31d4f070cc9790c8a62a",
    ),
    (
        "kart_lobby.json",
        "c1045046bc94277ca45fbb08725242316461c000f8ffb6b79c707fda6b846897",
    ),
    (
        "kart_gates.json",
        "478efd568ed52bff7f6a0cb46988656d18e2c59dc9f12178bd68bde1b2fd2cee",
    ),
    (
        "mining.json",
        "24568d64f7d32dcd7e9b025c963e85333bc0895ab4cc0207844d0549626358f8",
    ),
    (
        "harvesting.json",
        "bfe6897fe00aac118b0c9c92c8528e91cc4ef713e1e3b0fdd61f1690e8e42532",
    ),
];

fn data_content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn tree_hashes(directory: &Path) -> BTreeMap<String, String> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, digest(&fs::read(path).unwrap()))
        })
        .collect()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-bundle-activity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        for name in FILES {
            fs::copy(data_content().join(name), path.join(name)).unwrap();
        }
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn frozen_activity_bytes_are_the_reviewed_snapshot() {
    for (name, expected) in FROZEN_SHA256 {
        let bytes = fs::read(data_content().join(name)).unwrap();
        assert_eq!(digest(&bytes), *expected, "{name} changed");
    }
}

#[test]
fn bundled_activity_content_loads_without_lab() {
    let scratch = Scratch::new();
    let directory = scratch.path();
    assert!(
        !directory.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab")),
        "the fixture must not be the lab package"
    );

    let kart = KartGateway::load(directory, None)
        .expect("kart content loads")
        .expect("kart content is present");
    assert_eq!(
        kart.gate_npcs(),
        &[9000000273u64, 9000000039u64, 9500000759u64][..]
    );

    let lobby: LobbyNativeBody =
        serde_json::from_slice(&fs::read(directory.join("kart_lobby.json")).unwrap()).unwrap();
    let encoded = kart_lobby_native::encode_inner(&lobby).expect("lobby body encodes");
    assert_eq!(&encoded[..4], &[102, 0, 2, 0]);

    let mining = Type8Engine::load(
        directory,
        mining::CONTENT_FILE,
        MINING_GAME_ID,
        FIRST_MINING_INSTANCE,
        None,
    )
    .expect("mining content loads");
    assert_eq!(mining.profile_id(), MINER_PROFILE_ID);

    let harvest_content: HarvestContent =
        serde_json::from_slice(&fs::read(directory.join(harvesting::CONTENT_FILE)).unwrap())
            .unwrap();
    harvesting::validate_content(&harvest_content).expect("harvesting content is playable");
    let harvest = Type8Engine::load(
        directory,
        harvesting::CONTENT_FILE,
        111,
        harvesting::FIRST_INSTANCE_ID,
        None,
    )
    .expect("harvesting content loads");
    assert_eq!(harvest.profile_id(), CHEF_PROFILE_ID);
    assert_eq!(harvest.preload_file(), FARMING_PRELOAD_FILE);
    assert_eq!(harvest.launch_movie(), FARMING_LAUNCH_MOVIE);

    let encounter = EncounterEngine::load(directory, None).expect("encounter content loads");
    assert_eq!(encounter.world().zone_id, 0);
    assert_eq!(encounter.world().name, "FabledRealms");
    let value: Value =
        serde_json::from_slice(&fs::read(directory.join("encounters.json")).unwrap()).unwrap();
    assert_eq!(value["dungeons"][0]["id"], 411);
    assert_eq!(value["world_enemies"].as_array().unwrap().len(), 7);

    let spots = FishingSpots::load(directory)
        .expect("fishing spots load")
        .expect("fishing spots are present");
    assert_eq!(spots.gate_npcs().len(), 6);
    assert!(spots.menu_string_id() > 0);

    let nodes = GatheringNodes::load(directory)
        .expect("node tables load")
        .expect("node tables are present");
    assert!(nodes.actors().count() >= 2);
}

#[test]
fn activity_refusals_leave_the_content_directory_unchanged() {
    let scratch = Scratch::new();
    let directory = scratch.path();
    let before = tree_hashes(directory);

    let mut mining: Value =
        serde_json::from_slice(&fs::read(directory.join("mining.json")).unwrap()).unwrap();
    mining["game_id"] = Value::from(17);
    fs::write(
        directory.join("mining.json"),
        serde_json::to_vec(&mining).unwrap(),
    )
    .unwrap();
    let after_bad_mining = tree_hashes(directory);
    assert_ne!(after_bad_mining, before);
    assert!(
        Type8Engine::load(
            directory,
            mining::CONTENT_FILE,
            MINING_GAME_ID,
            FIRST_MINING_INSTANCE,
            None,
        )
        .is_err(),
        "a board for another game must be refused"
    );
    assert_eq!(
        tree_hashes(directory),
        after_bad_mining,
        "a refused board changed the content"
    );

    fs::copy(
        data_content().join("mining.json"),
        directory.join("mining.json"),
    )
    .unwrap();
    let mut gates: Value =
        serde_json::from_slice(&fs::read(directory.join("kart_gates.json")).unwrap()).unwrap();
    gates["npcs"] = Value::Array(vec![Value::from(9000000273u64), Value::from(9000000273u64)]);
    fs::write(
        directory.join("kart_gates.json"),
        serde_json::to_vec(&gates).unwrap(),
    )
    .unwrap();
    let before_bad_gates = tree_hashes(directory);
    assert!(
        KartGateway::load(directory, None).is_err(),
        "duplicate kart gates must be refused"
    );
    assert_eq!(
        tree_hashes(directory),
        before_bad_gates,
        "a refused gate list changed the content"
    );
}
