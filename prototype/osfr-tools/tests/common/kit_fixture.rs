//! A synthetic campaign with the shipped weapon kits and status table installed: three field
//! enemies (definition 400, guids 100 to 102) two metres in front of the origin.
#![allow(dead_code)]
use osfr_tools::encounter::{EncounterEngine, EncounterOutcome};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

pub struct Fixture(pub PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn shipped(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content").join(name)
}

impl Fixture {
    /// `health` and `damage` are authored values for the field enemies; `boss` marks the cave's
    /// one enemy (guid 200, dungeon 411).
    pub fn new(health: u32, damage: u32, boss: bool) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "kits-{}-{}",
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
        cave["name"] = json!("SyntheticCave");
        let content = json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[{"guid":200,"definition_id":401,"model_id":3,"name_id":100,"name":"Synthetic cave enemy","texture_alias":"","position":[0,0,0],"max_health":50000,"damage":0,"boss":boss}],"reward_coins":0,"reward_items":[]}]});
        fs::write(dir.join("encounters.json"), serde_json::to_vec(&content).unwrap()).unwrap();
        let enemies: Vec<_> = (100..103)
            .map(|guid| json!({"guid":guid,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic enemy","texture_alias":"","position":[(guid-101) as f32,0.0,2.0],"max_health":health,"damage":damage,"boss":false,"respawn_seconds":3600,"aggro_range":10}))
            .collect();
        fs::write(
            dir.join("field_enemies.json"),
            serde_json::to_vec(&json!({"schema_version":1,"enemies":enemies})).unwrap(),
        )
        .unwrap();
        for name in ["weapon_kits.json", "status_effects.json"] {
            fs::copy(shipped(name), dir.join(name)).unwrap();
        }
        Self(dir)
    }

    /// Rewrite one of the fixture's JSON files in place.
    pub fn patch(&self, name: &str, change: impl FnOnce(&mut Value)) {
        let path = self.0.join(name);
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        change(&mut value);
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }

    pub fn load(&self, profile: i32, level: u32) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, None).unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json")).unwrap();
        e.set_player_job(profile, level);
        e
    }
}

/// (attacker, target, damage) of every 32/7 in the outcome.
pub fn hits(out: &EncounterOutcome) -> Vec<(u64, u64, u32)> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| {
            (
                u64::from_le_bytes(p[4..12].try_into().unwrap()),
                u64::from_le_bytes(p[20..28].try_into().unwrap()),
                u32::from_le_bytes(p[28..32].try_into().unwrap()),
            )
        })
        .collect()
}

/// Damage of the player's hits (attacker 1), in order.
pub fn dealt(out: &EncounterOutcome) -> Vec<u32> {
    hits(out).into_iter().filter(|h| h.0 == 1).map(|h| h.2).collect()
}

/// Damage the enemies dealt to the player.
pub fn taken(out: &EncounterOutcome) -> Vec<u32> {
    hits(out).into_iter().filter(|h| h.0 != 1).map(|h| h.2).collect()
}

/// The current energy of the last 38/13 in the outcome.
pub fn energy(out: &EncounterOutcome) -> Option<i32> {
    out.replies
        .iter()
        .rev()
        .find(|p| p.starts_with(&[38, 0, 13, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
}

/// The string id of a 36/1 refusal in the outcome.
pub fn refusal(out: &EncounterOutcome) -> Option<i32> {
    out.replies
        .iter()
        .find(|p| p.starts_with(&[36, 0, 1, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
}

/// (guid, bits) of every 35/20 in the outcome.
pub fn states(out: &EncounterOutcome) -> Vec<(u64, u32)> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 20, 0]))
        .map(|p| {
            (
                u64::from_le_bytes(p[4..12].try_into().unwrap()),
                u32::from_le_bytes(p[12..16].try_into().unwrap()),
            )
        })
        .collect()
}
