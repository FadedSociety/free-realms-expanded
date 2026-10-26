//! Shared test helper: a minimal production-shaped field-enemy load.
//!
//! Include with `#[path = "common/field_enemy_fixture.rs"] mod field_enemy_fixture;`.
//! Test-only; the production path never builds this template.
#![allow(dead_code)] // each suite uses one field source and a subset of the helpers

use osfr_tools::encounter::{EncounterEngine, EncounterOutcome};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// A temp content root holding the captured-shaped AddNpc template, the world
/// zone and one dungeon. `field` is the field file the engine loads: the
/// shipped content, or a private copy a test may rewrite.
pub struct Fixture {
    dir: PathBuf,
    field: PathBuf,
}

impl Fixture {
    /// Load the shipped `data/content/field_enemies.json` in place. What the
    /// production loader reads is what the test asserts.
    pub fn shipped() -> Self {
        Self::new()
    }

    /// Put `rows` in the encounter content's `world_enemies`, so the production
    /// `EncounterEngine::load` scaling and the real attack path see them. Rows
    /// need `"stages": [0]` to be reachable at world stage 0.
    pub fn with_world_enemies(rows: Vec<Value>) -> Self {
        let fixture = Self::new();
        let path = fixture.dir.join("encounters.json");
        let mut content: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        content["world_enemies"] = json!(rows);
        fs::write(&path, content.to_string()).unwrap();
        fixture
    }

    /// Write a `combat_scale.json` beside the encounter content; it overrides
    /// the bundled file for the next load (`EncounterEngine::load`).
    pub fn scale(&self, edit: impl FnOnce(&mut Value)) {
        let mut file: Value =
            serde_json::from_slice(&fs::read(content_dir().join("combat_scale.json")).unwrap())
                .unwrap();
        edit(&mut file);
        fs::write(self.dir.join("combat_scale.json"), file.to_string()).unwrap();
    }

    /// Load a private copy of the shipped field file, so a test may rewrite it
    /// to exercise the reload path.
    pub fn copied() -> Self {
        let mut fixture = Self::new();
        let copy = fixture.dir.join("field_enemies.json");
        fs::copy(&fixture.field, &copy).unwrap();
        fixture.field = copy;
        fixture
    }

    fn new() -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "field-enemy-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let mut template = vec![0; 334];
        template[..4].copy_from_slice(&[35, 0, 2, 0]);
        template[133..137].copy_from_slice(&100_i32.to_le_bytes());
        template[224] = 1;
        fs::write(dir.join("npc.bin"), &template).unwrap();
        let zone = json!({"zone_id":0,"name":"SyntheticWorld","sky":"","geometry_id":214,"position":[0,0,0],"rotation":[0,0,0],"tutorial":false});
        let mut cave = zone.clone();
        cave["zone_id"] = json!(411);
        let other = json!({"guid":200,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":100,"damage":7,"boss":false});
        fs::write(
            dir.join("encounters.json"),
            json!({
                "schema_version":1,
                "template_file":"npc.bin",
                "template_sha256":format!("{:x}",Sha256::digest(&template)),
                "world":zone,
                "world_enemies":[],
                "dungeons":[{
                    "id":411,
                    "title_id":12,
                    "description_id":13,
                    "objective_id":14,
                    "zone":cave,
                    "guide":null,
                    "enemies":[other],
                    "reward_coins":0,
                    "reward_items":[]
                }]
            })
            .to_string(),
        )
        .unwrap();
        Self {
            dir,
            field: content_path(),
        }
    }

    /// The production load order: the encounter content, then the behaviour
    /// presets, then the field actors that name them.
    pub fn load(&self) -> EncounterEngine {
        let mut engine = self.load_encounters().unwrap();
        engine
            .load_behaviours(&content_dir().join("behaviours.json"))
            .unwrap();
        engine.load_field_enemies(&self.field).unwrap();
        engine
    }

    /// The first production load step alone, so a refusal test sees the error
    /// (`combat_scale.json` and `encounters.json` are read here).
    pub fn load_encounters(&self) -> std::io::Result<EncounterEngine> {
        EncounterEngine::load(&self.dir, None)
    }

    /// [`Fixture::load`] with the shipped field kill payouts and power-up drop
    /// tables, for a suite that must see what a kill pays or drops.
    pub fn load_paying(&self) -> EncounterEngine {
        fs::copy(
            content_dir().join("field_kill_rewards.json"),
            self.dir.join("field_kill_rewards.json"),
        )
        .unwrap();
        let mut engine = self.load();
        engine
            .load_power_ups(&content_dir().join("power_ups.json"))
            .unwrap();
        engine
    }

    /// The field file this fixture loads (the private copy in [`Fixture::copied`]).
    pub fn field_path(&self) -> &Path {
        &self.field
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

pub fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

pub fn content_path() -> PathBuf {
    content_dir().join("field_enemies.json")
}

/// A row of the shipped `field_enemies.json` by GUID; the file is parsed once
/// per test binary.
pub fn shipped_row(guid: u64) -> Option<&'static Value> {
    static FIELD: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_slice(&fs::read(content_path()).unwrap()).unwrap());
    FIELD["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["guid"].as_u64() == Some(guid))
}

/// GUIDs in the 35/2 AddNpc packets a spawn emitted. The client body reader
/// `0x00921A10` reads the u64 actor id as the first field and carries no level.
pub fn spawned(out: &EncounterOutcome) -> BTreeSet<u64> {
    out.replies
        .iter()
        .filter(|packet| packet.starts_with(&[35, 0, 2, 0]) && packet.len() >= 12)
        .map(|packet| u64::from_le_bytes(packet[4..12].try_into().unwrap()))
        .collect()
}
