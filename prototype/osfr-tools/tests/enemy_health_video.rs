//! T737: enemy health fitted to the archived gameplay video's hits-to-kill.
//!
//! The lead's frame pass (`lab/references/quest-lines/enemy-hp-video.json`, read
//! per frame in `lab/fleet/out/hp-frame/`) gives, per fight, the target health
//! fraction, the floating damage, the player's level and job, and the enemy's
//! content (the dungeon the fight happens in). It is a range, not a pin: frames
//! are two seconds apart, so hits go missing and one rising number can span two
//! frames.
//!
//! The fit lives in two places:
//! - `data/content/enemies.json`: each observed row's `level` is the level band
//!   of the content it was filmed in (a difficulty-5 dungeon enemy is level 20,
//!   the open-world level-4 fight's Cray Marauder is level 3).
//! - `EncounterEngine::load`: a world or dungeon row's health scales by that
//!   level exactly like a field enemy's (`scale_enemy`); before T737 the level
//!   was ignored at that load point, so every level-20 row died in one hit.
//!
//! Each fight below drives the shipped row through the real loader and
//! `EncounterEngine::attack` at the video's own player level and job. A data
//! retune moves the count, so the assertion is the video's observed range.

use osfr_tools::encounter::EncounterEngine;
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;

/// One observed fight: definition id, the video's player profile and level, the
/// basic-hit range read from the fights, and the source (video id and seconds).
struct Fight {
    definition_id: i64,
    profile: i32,
    level: u32,
    hits: std::ops::RangeInclusive<u32>,
    source: &'static str,
}

/// The observed fights (`enemy-hp-video.json`, 2026-10-26). The ranges are the
/// estimate-health divided by the fight's own per-hit damage, widened to the
/// number of hits the evidence actually counted; a blank frame never narrows a
/// range. Brawler 43, Ninja 2.
const FIGHTS: [Fight; 10] = [
    Fight {
        definition_id: 930038,
        profile: 43,
        level: 4,
        hits: 2..=4,
        source: "Cray Marauder SUgDAmJpyVs 44-52s, 176-202s (L4 Brawler, 73-98 a hit, ~200-300 health)",
    },
    Fight {
        definition_id: 930078,
        profile: 2,
        level: 2,
        hits: 2..=5,
        source: "Elder Swamp Cray -5UUuwaVXnM 324-334s, n9NH7VCyJrU 150-152s (L2/3 Ninja, 254/381, 635-1,058)",
    },
    Fight {
        definition_id: 1237,
        profile: 43,
        level: 5,
        hits: 4..=12,
        source: "Hewey the Bear _PEkuJgkSDw 178-204s (L1/5 Brawler, 4 counted hits for half a 4,190 bar)",
    },
    Fight {
        definition_id: 930057,
        profile: 2,
        level: 20,
        hits: 5..=9,
        source: "Elder Mugwort OawnLys6j-o 400-422s (L20 Ninja, ~17,729 health, 7 counted hits)",
    },
    Fight {
        definition_id: 2660,
        profile: 2,
        level: 20,
        hits: 4..=8,
        source: "Fungus Golem OrfNBeSJByM 254-276s (L20 Ninja, ~17,430 health, 3 counted hits for half)",
    },
    Fight {
        definition_id: 930016,
        profile: 2,
        level: 20,
        hits: 3..=6,
        source: "Nettleseed Pest OWvui60zJQw 134-154s (L20 Ninja, ~8,351 health, 5 counted hits)",
    },
    Fight {
        definition_id: 930076,
        profile: 2,
        level: 20,
        hits: 4..=7,
        source: "Ocean Cray Defender BJy3sgpdqpw 76-102s, 162-188s (L20 Ninja, 11,045-17,430, 2,490 a hit)",
    },
    Fight {
        definition_id: 930077,
        profile: 2,
        level: 20,
        hits: 2..=13,
        source: "Ocean Cray Brood Mother BJy3sgpdqpw 306-318s, 400-428s (L20 Ninja, 4,980-33,003)",
    },
    Fight {
        definition_id: 930045,
        profile: 2,
        level: 20,
        hits: 3..=7,
        source: "Queen Verda FiWCkh0St38 362-384s, 428-468s (L20 Ninja, 7,468-14,666, 2 counted fights)",
    },
    Fight {
        definition_id: 1495,
        profile: 2,
        level: 20,
        hits: 18..=30,
        source: "Foreman Ironfist X6zfuEFRcQk 188-212s (L20 Ninja x2, ~61,741 health over 24.4s)",
    },
];

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

/// A shipped `enemies.json` definition by definition id.
fn definition(definition_id: i64) -> Value {
    let file: Value =
        serde_json::from_slice(&fs::read(content_dir().join("enemies.json")).unwrap()).unwrap();
    file["definitions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["definition_id"].as_i64() == Some(definition_id))
        .unwrap_or_else(|| panic!("enemies.json has no definition {definition_id}"))
        .clone()
}

/// The observed rows as world enemies at world stage 0, the same authored
/// `max_health`, `damage` and `level` the shipped catalogue holds; the row at
/// `index` has the GUID `guid(index)`.
fn world_rows() -> Vec<Value> {
    FIGHTS
        .iter()
        .enumerate()
        .map(|(index, fight)| {
            let def = definition(fight.definition_id);
            serde_json::json!({
                "guid": 9_000_000_001 + index as u64,
                "definition_id": def["definition_id"],
                "model_id": def["model_id"],
                "name_id": def["name_string_id"],
                "name": def["kind"],
                "texture_alias": "",
                "position": [1.0, 0.0, 0.0],
                "max_health": def["max_health"],
                "damage": def["damage"],
                "level": def["level"].as_u64().unwrap_or(1),
                "boss": def["boss"],
                "stages": [0],
            })
        })
        .collect()
}

fn guid(index: usize) -> u64 {
    9_000_000_001 + index as u64
}

/// Basic swings on one target at the video's job and level, counting every
/// swing the production gate admits. The clock is shared across the fights in a
/// test: `strike` refuses a swing within 600 ms of `last_attack`, so a clock
/// restarted below it would silently refuse the first swings. The enemy must be
/// defeated.
fn swings_to_kill(
    engine: &mut EncounterEngine,
    guid: u64,
    profile: i32,
    level: u32,
    now: &mut u64,
) -> u32 {
    engine.set_player_job(profile, level);
    let mut swings = 0;
    loop {
        *now += 1_000;
        let out = engine
            .attack(guid, [0.0, 0.0, 0.0], profile, 0, *now)
            .unwrap();
        assert!(out.changed, "swing {swings} on {guid} was refused");
        swings += 1;
        let health = engine
            .state()
            .world_enemies
            .iter()
            .find(|enemy| enemy.guid == guid)
            .unwrap()
            .health;
        if health == 0 {
            return swings;
        }
        assert!(swings < 100, "enemy {guid} survived 100 basic swings");
    }
}

#[test]
fn observed_rows_die_in_the_videos_basic_hit_ranges() {
    let fixture = field_enemy_fixture::Fixture::with_world_enemies(world_rows());
    let mut engine = fixture.load();
    let mut now = 0;
    for (index, fight) in FIGHTS.iter().enumerate() {
        let guid = guid(index);
        let def = definition(fight.definition_id);
        let health = engine
            .state()
            .world_enemies
            .iter()
            .find(|enemy| enemy.guid == guid)
            .unwrap()
            .health;
        let swings = swings_to_kill(&mut engine, guid, fight.profile, fight.level, &mut now);
        eprintln!(
            "{}: authored {} at level {} -> {} health, {} level-{} swings (range {:?}); {}",
            def["kind"],
            def["max_health"],
            def["level"],
            health,
            swings,
            fight.level,
            fight.hits,
            fight.source
        );
        assert!(
            fight.hits.contains(&swings),
            "{}: {} swings is outside the video's {:?}; {}",
            def["kind"],
            swings,
            fight.hits,
            fight.source
        );
    }
}

#[test]
fn the_enemy_health_scale_is_tunable_from_data() {
    let rows = world_rows();
    let row = rows
        .iter()
        .position(|row| row["definition_id"] == serde_json::json!(930076))
        .unwrap();
    let guid = guid(row);
    let fixture = field_enemy_fixture::Fixture::with_world_enemies(vec![rows[row].clone()]);
    let scale = |fixture: &field_enemy_fixture::Fixture| {
        let mut engine = fixture.load();
        let health = engine
            .state()
            .world_enemies
            .iter()
            .find(|enemy| enemy.guid == guid)
            .unwrap()
            .health;
        let mut now = 0;
        (health, swings_to_kill(&mut engine, guid, 2, 20, &mut now))
    };
    let (base_health, base_swings) = scale(&fixture);
    fixture.scale(|file| file["enemy_health_scale"] = serde_json::json!(7.44));
    let (tuned_health, tuned_swings) = scale(&fixture);
    assert!(
        tuned_health > base_health && tuned_swings > base_swings,
        "x2 enemy_health_scale must lengthen the fight: {base_health}/{base_swings} -> {tuned_health}/{tuned_swings}"
    );
    // A data reload changes the fight, and it changes it by the file's factor:
    // compare with the shipped factor rather than pinning the tuned number.
    let shipped: f64 = serde_json::from_slice::<Value>(
        &fs::read(content_dir().join("combat_scale.json")).unwrap(),
    )
    .unwrap()["enemy_health_scale"]
        .as_f64()
        .unwrap();
    let expected = (f64::from(tuned_health) * shipped / 7.44).round() as u32;
    assert!(
        base_health.abs_diff(expected) <= 1,
        "base {base_health} against the same row at x{shipped}: {expected}"
    );
}

#[test]
fn an_out_of_bounds_enemy_scale_refuses_the_load() {
    let rows = world_rows();
    for bad in [0.001, 101.0] {
        let fixture = field_enemy_fixture::Fixture::with_world_enemies(rows.clone());
        fixture.scale(|file| file["enemy_health_scale"] = serde_json::json!(bad));
        let error = fixture
            .load_encounters()
            .err()
            .unwrap_or_else(|| panic!("enemy_health_scale {bad} must refuse the load"));
        assert!(
            error.to_string().contains("combat_scale.json"),
            "the error names the file: {error}"
        );
    }
    // Canary: the same fixture with the shipped scale loads.
    let fixture = field_enemy_fixture::Fixture::with_world_enemies(rows);
    fixture.scale(|_| {});
    assert!(fixture.load_encounters().is_ok());
}
