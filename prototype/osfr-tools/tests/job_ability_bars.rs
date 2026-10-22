//! T011: every combat job's bar comes from its own job level. Switching away
//! must neither consume an unlocked skill nor reset the job's experience.
use osfr_tools::{
    inventory::Inventory,
    opening_abilities::OpeningAbilities,
    profiles::{ProfileState, Profiles},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/job_playtest.rs"]
mod job_playtest;

use job_playtest::bar_ids;

static NEXT: AtomicU64 = AtomicU64::new(0);
const JOBS: [i32; 7] = [1, 11, 32, 12, 2, 43, 35]; // Adventurer 1 has no combat bar.

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-job-ability-bars-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let zero = 0i32.to_le_bytes();
        let mut profiles = Vec::new();
        let mut inventory_profiles = Vec::new();
        let mut spans = vec![json!({"kind":"coins","offset":13,"length":4,
            "profile_id":0,"sha256":digest(&zero)})];
        let mut end = 17;
        for id in JOBS {
            // 12 rank bytes, 16 equipment bytes and four trailing bytes match
            // the profile proof layout used by the real loader.
            let mut raw = vec![0u8; 32];
            raw[0..4].copy_from_slice(&1i32.to_le_bytes());
            profiles.push(json!({
                "id": id,
                "offset": 0,
                "length": raw.len(),
                "sha256": digest(&raw),
                "serialized_hex": hex(&raw),
                "rank_offset": 0,
                "equipment_offset": 12,
                "equipment_length": 16
            }));
            inventory_profiles.push(json!({"id":id,"allowed_classes":[1],"slots":[]}));
            spans.push(json!({"kind":"equipment","offset":end,"length":4,
                "profile_id":id,"sha256":digest(&zero)}));
            end += 4;
        }
        spans.push(json!({"kind":"items","offset":end,"length":4,
            "profile_id":0,"sha256":digest(&zero)}));
        fs::write(
            path.join("profiles.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "active_offset": 0,
                "active_sha256": digest(&1i32.to_le_bytes()),
                "profiles": profiles,
                "default_state": {
                    "version": 1,
                    "active": 11,
                    "unlocked": JOBS,
                    "experience": {}
                },
                "starter_items": {}
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            path.join("inventory.json"),
            serde_json::to_vec(&json!({
                "schema_version": 1,
                "player_guid": 1,
                "default_state": {"coins":0,"items":[],"equipment":[]},
                "profiles": inventory_profiles,
                "spans": spans,
                "definitions": []
            }))
            .unwrap(),
        )
        .unwrap();
        Self(path)
    }

    fn profiles(&self, experience: &BTreeMap<i32, u32>) -> Profiles {
        Profiles::load(
            &self.0,
            Some(ProfileState {
                version: 1,
                active: 11,
                unlocked: JOBS.into_iter().collect::<BTreeSet<_>>(),
                experience: experience.clone(),
            }),
        )
        .unwrap()
    }

    fn inventory(&self) -> Inventory {
        Inventory::load(&self.0, None).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn roster() -> OpeningAbilities {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let mut abilities = OpeningAbilities::load(&content).unwrap();
    abilities
        .load_job_abilities(&content.join("job_abilities.json"))
        .unwrap();
    abilities
}

fn level(profiles: &Profiles, id: i32) -> u32 {
    (1 + profiles.state().experience.get(&id).copied().unwrap_or(0) / 100).min(20)
}

#[test]
fn switching_jobs_keeps_each_jobs_level_and_unlocked_bar() {
    let fixture = Fixture::new();
    let experience: BTreeMap<i32, u32> = [
        (1, 0),
        (11, 1400), // Medic 15
        (32, 900),  // Warrior 10
        (12, 1900), // Wizard 20
        (2, 900),   // Ninja 10
        (43, 400),  // Brawler 5
        (35, 1400), // Archer 15
    ]
    .into_iter()
    .collect();
    let mut profiles = fixture.profiles(&experience);
    let inventory = fixture.inventory();
    let abilities = roster();
    let levels: BTreeMap<i32, u32> = JOBS.iter().map(|id| (*id, level(&profiles, *id))).collect();
    let canonical: BTreeMap<i32, Vec<Vec<u8>>> = JOBS
        .iter()
        .map(|id| (*id, abilities.activation(*id, levels[id]).unwrap()))
        .collect();

    // T421: Brawler at 5 has Leg Sweep (row 1) and Rock Toss (row 2) with no
    // blank button before them; Pummel opens row 3 at 10, and rows 4-7, which
    // the client never draws, stay empty.
    let brawler = bar_ids(&canonical[&43]);
    assert_eq!(&brawler[..4], &[Some(4895), Some(62018), Some(62024), None]);
    assert!(brawler[4..].iter().all(Option::is_none));

    // Switching through every combat job and the non-combat Adventurer returns
    // exactly the same job bar each time; experience and levels stay intact.
    let switches = [
        11, 32, 12, 2, 43, 35, 1, 35, 43, 2, 12, 32, 11, 1, 11, 32, 12, 2, 43, 35,
    ];
    for id in switches {
        let packet = profiles.select(id, &inventory).unwrap();
        assert_eq!(&packet[..4], &[38, 0, 21, 0]);
        assert_eq!(profiles.active(), id);
        assert_eq!(level(&profiles, id), levels[&id], "job {id} level");
        assert_eq!(
            abilities.activation(id, level(&profiles, id)).unwrap(),
            canonical[&id],
            "job {id} bar after switching"
        );
        assert_eq!(
            profiles.state().experience,
            experience,
            "job {id} switch changed experience"
        );
    }

    // Adventurer clears all eight slots rather than leaving the Medic bar up.
    let empty = bar_ids(&canonical[&1]);
    assert!(empty.iter().all(Option::is_none));
    assert_eq!(canonical[&1].len(), 1);
    assert_eq!(&canonical[&1][0][12..], &[0; 32]);
}
