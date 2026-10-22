//! T415: a level 1-4 player only meets its own band at the starter camps.
//!
//! T245, T296 and T332 stacked the level 5-20 camps on the 12 W102 starter
//! centres; aggro is distance-only and the client shows no level (the 35/2
//! AddNpc reader `0x00921A10` has no level field), so a level-5 player at
//! W102-H1 took 46 and 141 hits from the stacked MID-H1/ELITE-H2 Hooligans. T415
//! moved those 18 rows to audited centres. These tests read the shipped content
//! through the real loader and the real shared-field tick.
//!
//! Enemy hits are 32/7 AttackProcessed (client reader `0x00A2A910`): attacker
//! u64 at 4..12, target u64 at 20..28, damage i32 at 28..32, remaining health
//! i32 at 46..50.

use osfr_tools::encounter::EncounterEngine;
use serde_json::Value;
use std::fs;

#[path = "common/field_camp_centres.rs"]
mod field_camp_centres;
#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "common/field_reward_fixture.rs"]
mod field_reward_fixture;

use field_camp_centres::VERIFIED_CENTRES;
use field_enemy_fixture::{Fixture as EngineFixture, content_dir, content_path};
use field_reward_fixture::{RewardFixture, character_at, player_state};

/// The 4 m chase hysteresis in the field tick (`encounter/field.rs`: a chasing
/// actor keeps a target out to `aggro_range + 4`).
const CHASE_MARGIN: f32 = 4.0;
/// (R) Extra clearance between a higher band's aggro disc and a difficulty 1-2
/// battle marker, so a low-level player walking up to the marker is not pulled.
const MARKER_MARGIN: f32 = 20.0;
/// The top of the starter bands (W102: 1-2 and 3-4).
const STARTER_TOP: u32 = 4;
const PLAYER: u64 = 101;
const BRAWLER: i32 = 43;
/// Level 3 on the local rank curve: one base rank plus one per 100 XP.
const LEVEL_THREE_XP: u32 = 200;
const TICK_MS: u64 = 250;

/// One shipped field actor as the separation rule sees it.
struct Camp {
    name: String,
    band: [u32; 2],
    leash: f32,
    aggro: f32,
    at: [f32; 2],
}

fn read(path: &std::path::Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Every shipped actor with its group's band, leash and aggro, as the loader
/// applies them (`load_field_enemies` gives each member its group's aggro and
/// leash). An ungrouped (quest-gated) row keeps its own level as its band, its
/// own `aggro_range` and the leash of its resolved behaviour.
fn camps(engine: &EncounterEngine) -> Vec<Camp> {
    let field = read(&content_path());
    let groups = field["spawn_groups"].as_array().unwrap();
    field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let guid = row["guid"].as_u64().unwrap();
            let group = groups
                .iter()
                .find(|g| g["members"].as_array().unwrap().contains(&row["guid"]));
            let level = row["level"].as_u64().unwrap_or(1) as u32;
            let number = |value: &Value| value.as_f64().unwrap() as f32;
            Camp {
                name: row["spawn_area"]["id"]
                    .as_str()
                    .map_or_else(|| guid.to_string(), str::to_owned),
                band: group.map_or([level, level], |g| {
                    serde_json::from_value(g["level_band"].clone()).unwrap()
                }),
                leash: group.map_or_else(
                    || engine.behaviour(guid).unwrap().leash,
                    |g| number(&g["leash_radius"]),
                ),
                aggro: number(group.map_or(&row["aggro_range"], |g| &g["aggro_range"])),
                at: [number(&row["position"][0]), number(&row["position"][2])],
            }
        })
        .collect()
}

fn horizontal(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Bands more than two levels apart.
fn far(a: [u32; 2], b: [u32; 2]) -> bool {
    b[0] > a[1] + 2 || a[0] > b[1] + 2
}

#[test]
fn no_field_camp_shares_ground_with_a_band_more_than_two_levels_away() {
    let engine = EngineFixture::shipped().load();
    let camps = camps(&engine);
    assert_eq!(camps.len(), engine.field_guids().len());
    for (i, a) in camps.iter().enumerate() {
        for b in &camps[i + 1..] {
            if !far(a.band, b.band) {
                continue;
            }
            let need = a.leash + b.leash + a.aggro.max(b.aggro) + CHASE_MARGIN;
            let apart = horizontal(a.at, b.at);
            assert!(
                apart >= need,
                "{} (band {:?}) and {} (band {:?}) are {apart:.1} m apart; bands this far apart need {need:.1} m",
                a.name,
                a.band,
                b.name,
                b.band
            );
        }
    }
}

#[test]
fn low_difficulty_battle_markers_are_clear_of_higher_bands() {
    let engine = EngineFixture::shipped().load();
    let battles = read(&content_dir().join("field_battles.json"));
    let markers: Vec<(u64, [f32; 2])> = battles["battles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| (1..=2).contains(&b["difficulty"].as_u64().unwrap()))
        .map(|b| {
            let at = &b["spawner"]["position"];
            (
                b["id"].as_u64().unwrap(),
                [
                    at[0].as_f64().unwrap() as f32,
                    at[2].as_f64().unwrap() as f32,
                ],
            )
        })
        .collect();
    assert!(
        markers.len() >= 2,
        "the shipped difficulty 1-2 markers load"
    );
    for camp in camps(&engine).iter().filter(|c| c.band[0] > STARTER_TOP) {
        for (battle, at) in &markers {
            let need = camp.leash + camp.aggro + MARKER_MARGIN;
            let apart = horizontal(camp.at, *at);
            assert!(
                apart >= need,
                "{} (band {:?}) is {apart:.1} m from battle {battle}'s marker; it needs {need:.1} m",
                camp.name,
                camp.band
            );
        }
    }
}

#[test]
fn a_level_three_player_at_each_starter_camp_is_hit_only_by_its_own_band() {
    let field = read(&content_path());
    let fixture = RewardFixture::new(Vec::new());
    fixture.write_field(&field);
    // Starter rows are levels 1-3, weapon tier 1, so their hit is the authored
    // damage times the player's mitigation; a level-4 row would change that.
    let starter: Vec<(u64, f32)> = field["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["spawn_area"]["id"]
                .as_str()
                .is_some_and(|area| area.starts_with("W102-"))
        })
        .map(|row| {
            assert!(row["level"].as_u64().unwrap() <= 3, "{row}");
            (
                row["guid"].as_u64().unwrap(),
                row["damage"].as_f64().unwrap() as f32,
            )
        })
        .collect();
    assert_eq!(starter.len(), 12);
    let state = player_state(
        &fixture.base_state(),
        BRAWLER,
        0,
        Some((BRAWLER, LEVEL_THREE_XP)),
    );
    for (area, home) in &VERIFIED_CENTRES[..12] {
        let character = character_at(PLAYER, *home);
        let mut game =
            osfr_tools::gameplay::Gameplay::load(fixture.dir(), Some(state.clone())).unwrap();
        game.initialize_world(&character, 0).unwrap();
        let mut field = game.take_shared_fields().unwrap();
        let mut hits = 0;
        for tick in 1..=20_000 / TICK_MS {
            let now = tick * TICK_MS;
            let me = game.field_participant(&character).unwrap();
            assert!(me.mitigation > 0.0, "god mode is off");
            let (out, damage) = game
                .tick_shared_motion(&mut field, &[me.clone()], now, TICK_MS)
                .unwrap();
            for packet in out.replies.iter().filter(|p| p.starts_with(&[32, 0, 7, 0])) {
                let word = |at: usize| u32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
                let attacker = u64::from_le_bytes(packet[4..12].try_into().unwrap());
                let (dealt, left) = (word(28), word(46));
                let full = starter
                    .iter()
                    .find(|(guid, _)| *guid == attacker)
                    .map(|(_, damage)| (damage * me.mitigation).round() as u32)
                    .unwrap_or_else(|| {
                        panic!("{area}: {attacker} outside the starter band hit for {dealt}")
                    });
                assert!(
                    dealt == full || (dealt < full && left == 0),
                    "{area}: {attacker} hit for {dealt}, its full hit is {full}"
                );
                hits += 1;
            }
            let taken = damage
                .iter()
                .filter(|(guid, _)| *guid == PLAYER)
                .map(|(_, dealt)| *dealt)
                .sum();
            game.receive_field_damage(taken, now, field.enemies_targeting(PLAYER))
                .unwrap();
            if game.field_participant(&character).unwrap().health == 0 {
                break;
            }
        }
        assert!(hits > 0, "{area}: its own band engages within 20 s");
    }
}
