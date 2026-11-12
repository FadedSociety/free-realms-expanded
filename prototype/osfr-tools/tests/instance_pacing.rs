//! The owner's instance pacing (rc-0136, 2026-11-11): every instance fight a player enters
//! from the overworld (a battle starter's fight, or a field battle's own marker) is won solo
//! at its recommended level in 60 to 150 seconds, by the simulator's player with the job's
//! real rotation (`EncounterEngine::simulate_clear`, as `realm sim` runs it), on the shipped
//! engine and the job's shipped bar (`opening-abilities.json`, `job_abilities.json`).
//! "Won" is the run's own verdict: a fight file lets a knocked-out player get up (its
//! `knockouts`), a `field_battles.json` row ends at the first. Dungeons keep their own pace
//! and carry no `instance_pacing.json` row. The table of every fight's time, the health left,
//! the hearts taken and the knockouts prints with `--nocapture`.
use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, pacing::PacingTable, sim::SimEnd},
    opening_abilities::OpeningAbilities,
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf};

const BRAWLER: i32 = 43;
/// Seconds one player should need, lowest and highest (the owner, 2026-11-11).
const BAND: (f32, f32) = (60.0, 150.0);

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn json(rel: &str) -> Value {
    serde_json::from_slice(&fs::read(content().join(rel)).unwrap()).unwrap()
}

/// Recommended level: the fight's own gate, or level 1 where it gives none.
fn level(gate: &Value) -> u32 {
    gate.as_u64().unwrap_or(0).max(1) as u32
}

/// (fight id, name, recommended level) of every overworld-entered instance fight that runs
/// today: each battle starter's fight file or battle row, and every `field_battles.json` row
/// whose own marker stands (no entrance runs or counts as it).
fn overworld_fights() -> Vec<(u32, String, u32)> {
    let battles = json("field_battles.json");
    let rows = battles["battles"].as_array().unwrap();
    let row = |id: u64| rows.iter().find(|b| b["id"].as_u64() == Some(id));
    let entrances = json("entrances.json");
    let entrances = entrances["entrances"].as_array().unwrap();
    let mut fights = Vec::new();
    for e in entrances.iter().filter(|e| e["kind"] == "starter") {
        let Some(id) = e["fight"].as_u64() else {
            continue;
        };
        // A starter that runs a battle row runs it under its own encounter id, as a click on it
        // does.
        let run = if row(id).is_some() { e["encounter_id"].as_u64().unwrap() } else { id };
        let file = content().join(format!("fights/{id}.json"));
        let gate = if let Some(b) = row(id).filter(|_| !file.is_file()) {
            b["required_combat_level"].clone()
        } else if file.is_file() {
            let fight: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
            fight["entry"]["min_combat_level"].clone()
        } else {
            continue; // a starter whose fight is not built yet stands closed
        };
        fights.push((run as u32, e["title"].as_str().unwrap().to_owned(), level(&gate)));
    }
    let taken: BTreeSet<u64> = (entrances.iter())
        .flat_map(|e| [e["fight"].as_u64(), e["counts_as"].as_u64()])
        .flatten()
        .collect();
    for b in rows.iter().filter(|b| !taken.contains(&b["id"].as_u64().unwrap())) {
        let name = format!("battle {}", b["id"]);
        fights.push((b["id"].as_u64().unwrap() as u32, name, level(&b["required_combat_level"])));
    }
    fights
}

/// The shipped engine as `Gameplay::load` builds it: behaviours, field enemies, camps, battles.
fn engine() -> EncounterEngine {
    let dir = content();
    let mut engine = EncounterEngine::load(&dir, None).unwrap_or_else(|e| panic!("{e}"));
    engine.load_behaviours(&dir.join("behaviours.json")).unwrap();
    engine.load_field_enemies(&dir.join("field_enemies.json")).unwrap();
    engine.load_field_camps(&dir).unwrap();
    engine.load_field_battles(&dir.join("field_battles.json")).unwrap();
    engine.load_power_ups(&dir.join("power_ups.json")).unwrap();
    engine
}

/// The job's bar at a level, as the server gives it (slots 1 to 7).
fn bar(abilities: &OpeningAbilities, level: u32) -> Vec<osfr_tools::opening_abilities::JobAbility> {
    (1..=7)
        .filter_map(|slot| abilities.job_slot(BRAWLER, slot, level).cloned())
        .collect()
}

fn home() -> CharacterState {
    let world = json("encounters.json");
    let at = |v: &Value| [0, 1, 2].map(|i| v[i].as_f64().unwrap() as f32);
    CharacterState {
        guid: 1,
        zone_id: world["world"]["zone_id"].as_u64().unwrap_or(0) as u32,
        position: at(&world["world"]["position"]),
        rotation: [0.0, 0.0, 1.0],
    }
}

#[test]
fn every_overworld_instance_fight_takes_one_to_two_and_a_half_minutes_solo() {
    let engine = engine();
    let mut abilities = OpeningAbilities::load(&content()).unwrap();
    abilities.load_job_abilities(&content().join("job_abilities.json")).unwrap();
    let (mut table, mut bad) = (String::new(), Vec::new());
    table.push_str(
        "fight  name                        level  result   seconds  health left  hearts  knockouts\n",
    );
    let fights = overworld_fights();
    assert!(fights.len() >= 20, "{} overworld instance fights", fights.len());
    for (id, name, level) in fights {
        let line = match engine.simulate_clear(id, level, BRAWLER, &bar(&abilities, level), home()) {
            Err(why) => {
                let why = why.to_string();
                bad.push(format!("{id} {name}: {why}"));
                format!("{id:<6} {name:<27} {level:>5}  {why}\n")
            }
            Ok(sim) => {
                let paced = sim.end == SimEnd::Won && (BAND.0..=BAND.1).contains(&sim.seconds);
                if !paced {
                    bad.push(format!("{id} {name}: {:?} in {:.0} s", sim.end, sim.seconds));
                }
                format!(
                    "{id:<6} {name:<27} {level:>5}  {:<7} {:>7.0}  {:>5}/{:<5}  {:>6}  {:>9}\n",
                    format!("{:?}", sim.end),
                    sim.seconds,
                    sim.health,
                    sim.max_health,
                    sim.hearts,
                    sim.knockouts
                )
            }
        };
        table.push_str(&line);
    }
    println!("{table}");
    assert!(
        bad.is_empty(),
        "{} fights are not won solo in {}-{} s:\n{}",
        bad.len(),
        BAND.0,
        BAND.1,
        bad.join("\n")
    );
}

/// The owner: pacing is for the fights entered from the overworld; dungeons keep their pace.
#[test]
fn pacing_rows_are_overworld_fights_only() {
    // A starter that runs a battle row is paced under the row it counts as.
    let entrances = json("entrances.json");
    let ran: BTreeSet<u32> = (entrances["entrances"].as_array().unwrap().iter())
        .filter(|e| e["kind"] == "starter")
        .filter_map(|e| e["fight"].as_u64())
        .map(|id| id as u32)
        .collect();
    let overworld: BTreeSet<u32> = overworld_fights().into_iter().map(|f| f.0).collect();
    let overworld: BTreeSet<u32> = overworld.union(&ran).copied().collect();
    let table = PacingTable::load(&content()).unwrap();
    let ids: Vec<u32> = table.ids().collect();
    assert!(!ids.is_empty(), "the shipped table paces the overworld fights");
    for id in ids {
        assert!(overworld.contains(&id), "{id} has a pacing row but is no overworld instance fight");
    }
}
