//! Shared helpers for the fight suites (`fights`, `fight_objectives`, `fight_frostfang`): parse
//! a fight written inline, and replay a run from facts without an engine.
#![allow(dead_code)] // each suite uses its own subset

use osfr_tools::encounter::{
    FightFact,
    brain::Act,
    director::{self, Catalogue, DirectorAct, Fight, Member, Party, Run, Signal},
};
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// The provenance every fight file must carry.
pub fn source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) test fight"}])
}

/// A small arena every inline fight can stand in.
pub fn arena() -> Value {
    json!({"zone": 9600, "arena": "sg_random_encounter_clearing", "sky": "sky_wilds24.xml",
           "geometry_id": 214, "arrive": [0, 0, 0], "exit_door": [0, 0, 20]})
}

pub fn kinds() -> Value {
    json!({
        "grunt": {"definition_id": 7001, "name": "Grunt", "name_id": 1, "model_id": 3},
        "mage": {"definition_id": 7002, "name": "Mage", "name_id": 1, "model_id": 3},
        "chief": {"definition_id": 7003, "name": "Chief", "name_id": 1, "model_id": 3, "rank": "boss", "boss": true},
        "crate": {"definition_id": 7004, "name": "Crate", "name_id": 1, "model_id": 3, "rank": "object"},
        "captive": {"definition_id": 7005, "name": "Captive", "name_id": 1, "model_id": 3, "side": "ally"},
        "tent": {"definition_id": 7006, "name": "Supply Tent", "name_id": 1, "model_id": 3, "side": "ally", "max_health": 100}
    })
}

pub fn try_parse(fight: &Value) -> Result<Fight, String> {
    Fight::parse(
        fight.to_string().as_bytes(),
        &Catalogue::default(),
        "test.json",
    )
    .map_err(|e| e.to_string())
}

pub fn parse(fight: &Value) -> Fight {
    try_parse(fight).unwrap_or_else(|e| panic!("{e}"))
}

/// The refusal a broken fight gets.
pub fn refusal(fight: &Value) -> String {
    try_parse(fight).expect_err("the loader accepted a broken fight")
}

/// One run replayed from facts: every act the run asked for is kept in order.
pub struct Replay {
    pub run: Run,
    pub now: u64,
    pub acts: Vec<DirectorAct>,
    pub plain: Vec<Act>,
}

impl Replay {
    pub fn start(fight: &Fight, players: usize) -> Self {
        let members = (0..players)
            .map(|i| Member {
                guid: i as u64 + 1,
                profile: 43,
                level: 1,
            })
            .collect();
        let mut replay = Self {
            run: director::begin(fight, &Party { members }),
            now: 1_000,
            acts: Vec::new(),
            plain: Vec::new(),
        };
        replay.step(0);
        replay
    }

    /// Move the clock on and collect what the run asks for.
    pub fn step(&mut self, ms: u64) -> Vec<DirectorAct> {
        self.now += ms;
        let mut out = Vec::new();
        director::tick(&mut self.run, self.now, &mut out);
        self.acts.extend(out.iter().cloned());
        out
    }

    pub fn fact(&mut self, fact: Signal) -> Vec<DirectorAct> {
        director::on_fact(&mut self.run, &fact, &mut self.plain);
        self.step(0)
    }

    /// Guids of the living actors that carry `tag`, in roster order.
    pub fn living(&self, tag: &str) -> Vec<u64> {
        let fight = self.run.definition();
        (0..fight.roster.len())
            .filter(|&i| fight.roster[i].tags.iter().any(|t| t == tag))
            .map(|i| fight.guid(Some(i)))
            .filter(|guid| self.run.life(*guid) == Some(director::Life::Alive))
            .collect()
    }

    pub fn guid(&self, actor: &str) -> u64 {
        let fight = self.run.definition();
        let i = fight
            .roster
            .iter()
            .position(|slot| slot.key == actor)
            .unwrap_or_else(|| panic!("no actor {actor}"));
        fight.guid(Some(i))
    }

    pub fn kill(&mut self, guid: u64) -> Vec<DirectorAct> {
        self.fact(Signal::Died {
            guid,
            kind: 0,
            tags: Vec::new(),
        })
    }

    /// Defeat every living actor with the tag; returns how many fell.
    pub fn clear(&mut self, tag: &str) -> usize {
        let guids = self.living(tag);
        for guid in &guids {
            self.kill(*guid);
        }
        guids.len()
    }

    pub fn talk(&mut self, guid: u64) -> Vec<DirectorAct> {
        self.fact(Signal::Talked { guid })
    }

    pub fn knock_out(&mut self) -> Vec<DirectorAct> {
        self.fact(Signal::PlayerDown { guid: 1 })
    }

    /// Leave through the open exit door.
    pub fn leave(&mut self) -> Vec<DirectorAct> {
        let door = self.run.definition().guid(None);
        self.talk(door)
    }

    pub fn waves(&self) -> Vec<String> {
        self.acts
            .iter()
            .filter_map(|act| match act {
                DirectorAct::Wave(id) => Some(id.clone()),
                _ => None,
            })
            .collect()
    }

    /// (goal, done, of) of every goal fact so far.
    pub fn goal_facts(&self) -> Vec<(String, u32, u32)> {
        self.acts
            .iter()
            .filter_map(|act| match act {
                DirectorAct::Fact(FightFact::Goal { goal, done, of, .. }) => {
                    Some((goal.clone(), *done, *of))
                }
                _ => None,
            })
            .collect()
    }

    pub fn count(&self, wanted: &DirectorAct) -> usize {
        self.acts.iter().filter(|act| *act == wanted).count()
    }
}

// ---------------------------------------------------------------------------
// A real engine over a scratch content directory that holds the given fights.
// ---------------------------------------------------------------------------

use osfr_tools::{character_store::CharacterState, encounter::EncounterEngine};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

pub const BRAWLER: i32 = 43;

pub fn home() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0, 0.0, 1.0],
    }
}

/// The smallest content the engine loads (one cave row), plus `fights/<id>.json` for each
/// fight given. The directory is the caller's to remove.
pub fn engine_with(fights: &[Value]) -> (PathBuf, EncounterEngine) {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "osfr-fights-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("fights")).unwrap();
    let mut template = vec![0_u8; 334];
    template[..4].copy_from_slice(&[35, 0, 2, 0]);
    template[133..137].copy_from_slice(&100_i32.to_le_bytes());
    template[224] = 1;
    std::fs::write(dir.join("npc.bin"), &template).unwrap();
    let zone = json!({"zone_id": 0, "name": "0", "sky": "", "geometry_id": 214,
                      "position": [0, 0, 0], "rotation": [0, 0, 1], "tutorial": false});
    let mut cave = zone.clone();
    cave["zone_id"] = json!(411);
    let actor = json!({"guid": 300, "definition_id": 930110, "model_id": 3, "name_id": 100,
                       "name": "930110", "texture_alias": "", "position": [0, 0, 0],
                       "max_health": 300, "damage": 0, "boss": false});
    let encounters = json!({
        "schema_version": 1,
        "template_file": "npc.bin",
        "template_sha256": format!("{:x}", Sha256::digest(&template)),
        "world": zone,
        "world_enemies": [],
        "dungeons": [{"id": 411, "title_id": 12, "description_id": 13, "objective_id": 14,
                      "zone": cave, "guide": null, "enemies": [actor],
                      "reward_coins": 0, "reward_items": []}]
    });
    std::fs::write(dir.join("encounters.json"), encounters.to_string()).unwrap();
    for fight in fights {
        let name = format!("fights/{}.json", fight["id"]);
        std::fs::write(dir.join(name), fight.to_string()).unwrap();
    }
    let engine = EncounterEngine::load(&dir, None).unwrap_or_else(|e| panic!("{e}"));
    (dir, engine)
}

/// Begin fight `id` and load its arena; returns the packets of the arrival.
pub fn enter(engine: &mut EncounterEngine, id: u32, now_ms: u64) -> Vec<Vec<u8>> {
    engine.begin(id, home(), BRAWLER).unwrap();
    engine.on_zone_ready(BRAWLER, now_ms).unwrap().replies
}

/// Guids of the actors a batch of packets adds (35/2 AddNpc).
pub fn spawned(replies: &[Vec<u8>]) -> Vec<u64> {
    replies
        .iter()
        .filter(|r| r.starts_with(&[35, 0, 2, 0]))
        .map(|r| u64::from_le_bytes(r[4..12].try_into().unwrap()))
        .collect()
}

/// 26/8: the player clicks an actor.
pub fn click(guid: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 8, 0];
    b.extend(guid.to_le_bytes());
    b.push(0);
    b
}

/// 41/122: the knockout window's free button (or its timer).
pub fn stand_up() -> Vec<u8> {
    let mut b = vec![41, 0, 122, 0];
    b.extend([0; 9]);
    b
}

/// The rows of a results card (39/47): name to (count, max, points).
pub fn score_rows(replies: &[Vec<u8>]) -> std::collections::BTreeMap<String, (i32, i32, i32)> {
    let card = replies
        .iter()
        .rev()
        .find(|r| r.starts_with(&[39, 0, 47]))
        .expect("no results card");
    let int = |at: usize| i32::from_le_bytes(card[at..at + 4].try_into().unwrap());
    let mut at = 15;
    let rows = int(at);
    at += 4;
    let mut out = std::collections::BTreeMap::new();
    for _ in 0..rows {
        let len = int(at) as usize;
        let name = String::from_utf8_lossy(&card[at + 4..at + 4 + len]).into_owned();
        at += 4 + len;
        out.insert(name, (int(at + 8), int(at + 12), int(at + 16)));
        at += 20;
    }
    out
}
