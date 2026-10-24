//! T398: the shipped starter field enemies stay inside the reconstructed (R)
//! final-era budget. Bounds only (the owner: tunable over locked); the numbers come
//! from `data/content/field_enemies.json` (each row) and `combat_scale.json`
//! (player health, tier factors, enemy scales).
//!
//! (R) Budget: at its own level, a lone starter enemy needs over 90 s to knock
//! out an idle player of the weakest job (Wizard, health x0.85), and a whole
//! starter camp, every member swinging at once, over 30 s. A camp is a spawn
//! group, or the ungrouped spawns of one quest (they appear together).
//! Starter means an authored level of 5 or less. Health and hits come from the
//! real engine: the player's max health after `set_player_job`, and each hit
//! from its 32/7 `attack_processed` (client reader `0x00a2a910`, damage dword
//! at +28, `common/job_playtest.rs`).

use std::collections::{BTreeMap, BTreeSet};

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "common/job_playtest.rs"]
mod job_playtest;

use field_enemy_fixture::{Fixture, content_path};
use serde_json::Value;

const WIZARD: i32 = 12;
const STARTER_MAX_LEVEL: u32 = 5;
const LONE_FLOOR_S: f32 = 90.0;
const CAMP_FLOOR_S: f32 = 30.0;

/// The shipped engine with every quest-gated enemy present too.
fn loaded(field: &Value) -> osfr_tools::encounter::EncounterEngine {
    let mut engine = Fixture::shipped().load();
    let quests: BTreeSet<u32> = field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|row| row["quest_ids"].as_array().into_iter().flatten())
        .map(|id| id.as_u64().unwrap() as u32)
        .collect();
    engine.set_active_quests(&quests);
    engine
}

/// The weakest level-matched player's max health and the damage per second
/// `row` deals it, both read from a fresh engine over the shipped file.
fn measure(field: &Value, row: &Value, level: u32) -> (u32, f32) {
    let guid = row["guid"].as_u64().unwrap();
    let mut engine = loaded(field);
    engine.set_player_job(WIZARD, level);
    engine.on_zone_ready(WIZARD, 0).unwrap();
    let at: Vec<f32> = row["position"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let out = engine.tick([at[0], at[1], at[2]], WIZARD, 100).unwrap();
    let hits = job_playtest::hits_from(&out.replies, guid);
    assert!(
        hits.len() == 1 && hits[0] > 0,
        "{guid} must swing once at a player in reach: {hits:?}"
    );
    let attack_ms = engine.behaviour(guid).unwrap().attack_ms;
    let dps = hits[0] as f32 * 1000.0 / attack_ms as f32;
    (engine.player_health().1, dps)
}

#[test]
fn starter_enemies_and_camps_stay_inside_the_final_era_budget() {
    let field: Value = serde_json::from_slice(&std::fs::read(content_path()).unwrap()).unwrap();
    let mut camp_of: BTreeMap<u64, String> = field["spawn_groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| {
            let id = group["id"].as_str().unwrap().to_owned();
            group["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |m| (m.as_u64().unwrap(), id.clone()))
        })
        .collect();
    // camp -> (lowest member level, summed damage per second)
    let mut camps: BTreeMap<String, (u32, f32)> = BTreeMap::new();
    let mut starters = 0;
    let levels = loaded(&field);
    for row in field["enemies"].as_array().unwrap() {
        let guid = row["guid"].as_u64().unwrap();
        let level = levels.field_level(guid).unwrap();
        if level > STARTER_MAX_LEVEL {
            continue;
        }
        starters += 1;
        let (health, dps) = measure(&field, row, level);
        if !camp_of.contains_key(&guid)
            && row["quest_ids"].as_array().is_some_and(|q| !q.is_empty())
        {
            camp_of.insert(guid, format!("quest {}", row["quest_ids"]));
        }
        let lone = health as f32 / dps;
        assert!(
            lone > LONE_FLOOR_S,
            "{guid} (level {level}) knocks out a level-{level} Wizard ({health} HP) in {lone:.0} s"
        );
        if let Some(camp) = camp_of.get(&guid) {
            let entry = camps.entry(camp.clone()).or_insert((level, 0.0));
            entry.0 = entry.0.min(level);
            entry.1 += dps;
        }
    }
    assert!(starters >= 14, "the starter rows were found: {starters}");
    assert!(camps.len() >= 5, "the starter camps were found: {camps:?}");
    for (camp, (level, dps)) in camps {
        let mut engine = Fixture::shipped().load();
        engine.set_player_job(WIZARD, level);
        let seconds = engine.player_health().1 as f32 / dps;
        assert!(
            seconds > CAMP_FLOOR_S,
            "{camp}: all members together knock out a level-{level} Wizard in {seconds:.0} s"
        );
    }
}
