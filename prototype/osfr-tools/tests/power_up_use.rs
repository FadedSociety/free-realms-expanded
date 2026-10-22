//! T222/F263: the held combat power-up on its own action-bar key.
//!
//! The client's `ActionBarSlot3` (default `3`) dispatches job-bar slot 2 and
//! sends 36/10 bar 1 slot 2 (see `notes/Research/Power-up Use Key.md`). These
//! tests drive that real request through `Gameplay::handle_in_world`, with the
//! shipped field Hooligans and a guaranteed drop loaded by the production
//! `Gameplay::load` content root. Nothing here calls the power-up code directly.
use osfr_tools::{
    character_store::CharacterState,
    derby::{DerbyConfig, DerbyRules, DerbyVehicleTemplate},
    derby_gateway::{DerbyGates, DerbyGatewayContent},
    encounter::SharedFieldAction,
    gameplay::{Gameplay, Outcome},
    power_ups::{PowerUps, Slot},
};
#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;

use field_enemy_fixture::shipped_row;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Shipped `data/content/field_enemies.json` Hooligan Wolf whose `standard`
/// table this fixture replaces with a guaranteed drop. It is always present
/// (no quest gate).
const TARGET: u64 = 131_000_102_001;
const TARGET_AT: [f32; 3] = [-1899.204, -50.59, 307.902];
/// A clear point 40 m east of the F1 camp where nothing is in Flame Wave range;
/// T415 keeps every moved camp at least leash + radius + 4 m from it.
const CLEAR_AT: [f32; 3] = [-1859.204, -50.59, 307.902];
/// T245's level-5 MID-F1 wolf. T415 moved it off the F1 centre; its home is
/// read from the shipped file.
const MID_F1: u64 = 131_000_102_013;
/// A second always-present Hooligan, where the player walks after the pickup;
/// its own position is inside the Flame Wave radius.
const SECOND: u64 = 131_000_102_004;
const SECOND_AT: [f32; 3] = [-1779.455, -28.88, 562.833];
/// F263 raises this Hooligan above one level-15 Flame Wave, so its damage
/// packet reports the authored hit instead of the target's health cap.
const SECOND_HEALTH: u32 = 5_000;
/// Brawler XP for level 15, where its slot-2 job ability is unlocked.
const LEVEL_15_XP: u32 = 1400;
/// The fixture's only NPC, registered as the derby gate (`derby_gates.json`).
const GATE: u64 = 1;
/// Bundled `field_battles.json` markers this fixture keeps: the difficulty-5
/// arena and the Troll Summoner fight whose minions shield it.
const TIER_FIVE_MARKER: u64 = 100_000_004_750;
const TIER_FIVE_AT: [f32; 3] = [-950.582, -34.42554, -651.9259];
const SUMMONER_MARKER: u64 = 100_000_096_000;
const SUMMONER_AT: [f32; 3] = [-440.0, 0.8, 1290.0];
const STONE_HEART: u64 = 100_001_230_301;
const SUMMONER: u64 = 100_000_096_001;
const SUMMONER_MINION: u64 = 100_000_096_002;
const SUMMONER_MINION_B: u64 = 100_000_096_003;

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    fn new() -> Self {
        Self::with_drop("flame_wave")
    }

    fn with_drop(kind: &str) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t222-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        for entry in fs::read_dir(reference()).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                fs::copy(&source, dir.join(source.file_name().unwrap())).unwrap();
            }
        }
        let bundled = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
        // The canonical combat roster (starter weapon classes), as the other
        // job tests do; the pinned opening package predates some weapons.
        fs::copy(
            bundled.join("opening-abilities.json"),
            dir.join("opening-abilities.json"),
        )
        .unwrap();
        // Own NPC content root: this makes `Gameplay::load` read the fixture's
        // field enemies, job bar and power-up tables (the production loader),
        // instead of the bundled `data/content` fallback.
        fs::create_dir_all(dir.join("npcs")).unwrap();
        fs::write(dir.join("npc_spawns_extra.json"), r#"{"records":[]}"#).unwrap();
        // The interaction router (and the derby gate's menu) needs these.
        fs::write(
            dir.join("stores.json"),
            r#"{"schema_version":1,"marketplace":{"enabled_on_handshake":false,"orders_enabled":false,"payment_sources_enabled":false,"not_available_message":"disabled"},"items":[]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("vendors.json"),
            r#"{"schema_version":1,"vendors":[]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("npcs/anchor.json"),
            r#"{"schema_version":1,"records":[{"npc_id":1,"name":"T222 anchor","zone_id":0,"model_id":1,"name_string_id":1,"position":[0.0,0.0,0.0],"placement":"legacy_data","roles":["derby_guide"]}]}"#,
        )
        .unwrap();
        for name in ["zones.json", "job_abilities.json", "behaviours.json"] {
            fs::copy(bundled.join(name), dir.join(name)).unwrap();
        }
        // The shipped field roster with one surviving Hooligan for the damage
        // assertions; everything else stays byte-identical.
        let mut field: Value =
            serde_json::from_slice(&fs::read(bundled.join("field_enemies.json")).unwrap()).unwrap();
        for enemy in field["enemies"].as_array_mut().unwrap() {
            if enemy["guid"].as_u64() == Some(SECOND) {
                enemy["max_health"] = json!(SECOND_HEALTH);
            }
        }
        fs::write(dir.join("field_enemies.json"), field.to_string()).unwrap();
        // Two real battle rows: difficulty 5 indexes the power-up's battle
        // tier; battle 96 lets a summoner shield itself behind its minions.
        // Prizes are stripped so no item catalogue is needed, and the level
        // floor drops so a level-1 test can enter the difficulty-5 arena.
        let battles: Value =
            serde_json::from_str(include_str!("../data/content/field_battles.json")).unwrap();
        let kept: Vec<Value> = battles["battles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["id"].as_u64() == Some(12_303) || row["id"].as_u64() == Some(96))
            .map(|row| {
                let mut row = row.clone();
                row["prizes"] = json!([]);
                if row["id"].as_u64() == Some(12_303) {
                    row["required_combat_level"] = json!(0);
                }
                row
            })
            .collect();
        let battles = json!({"schema_version": 1, "battles": kept});
        fs::write(dir.join("field_battles.json"), battles.to_string()).unwrap();
        // Every kind and table stays byte-identical; only the shipped standard
        // table becomes a guaranteed `kind`.
        let mut power_ups: Value =
            serde_json::from_str(include_str!("../data/content/power_ups.json")).unwrap();
        power_ups["tables"]["standard"] = json!({"guaranteed":[kind],"source":"T222 test table"});
        fs::write(dir.join("power_ups.json"), power_ups.to_string()).unwrap();
        // A valid derby package so the real NPC offer/cancel route can run.
        fs::write(
            dir.join("derby_entry.json"),
            serde_json::to_vec(&derby_content()).unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("derby_gates.json"),
            serde_json::to_vec(&DerbyGates {
                schema_version: 1,
                menu_string_id: 3053,
                npcs: vec![GATE],
                reward_cooldown_s: None,
            })
            .unwrap(),
        )
        .unwrap();
        Self(dir)
    }

    fn content(&self) -> &Path {
        &self.0
    }

    fn slot(&self, kind: &str) -> Slot {
        PowerUps::load(self.0.join("power_ups.json"))
            .unwrap()
            .slot(kind)
            .unwrap()
    }
}

/// A valid solo derby package (the same rules the derby route tests use).
fn derby_content() -> DerbyGatewayContent {
    DerbyGatewayContent {
        schema_version: 1,
        game_id: 39,
        vehicle: DerbyVehicleTemplate {
            name: b"synthetic-derby".to_vec(),
            definition: vec![1, 2, 3, 4],
            start_transform_bits: [0; 16],
            unknown_flag: false,
            unknown_value_bits: 0,
        },
        rules: DerbyRules {
            max_drivers: 8,
            start_health: 20,
            contact_damage_per_unit: 1.0,
            contact_cooldown_ms: 100,
            contact_radius_m: 5.0,
            max_impact: 50.0,
            max_pose_age_ms: 5_000,
            max_damage_per_contact: 100,
            lobby_timeout_ms: 120_000,
            pickup_radius: 3.0,
            health_pickup: 5,
            boost_pickup: 5.0,
            shield_duration_ms: 1_000,
            attack_duration_ms: 1_000,
            ultimate_duration_ms: 1_000,
            attack_multiplier: 2.0,
            ultimate_multiplier: 2.0,
            respawn_delay_ms: 1_000,
            results_duration_ms: 1_000,
            knockout_points: 2,
            leader_knockout_points: 3,
            knockout_point_loss: 1,
            score_target: Some(2),
            time_limit_ms: None,
            config: DerbyConfig {
                value_a_bits: 0,
                value_b: 0,
            },
        },
        pickups: Vec::new(),
        completion_reward_coins: 0,
        completion_reward_xp: 0,
        reward: None,
    }
}

fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: TARGET_AT,
        rotation: [0.0; 3],
    }
}

fn command(family: u16, sub: u16, fields: &[i32]) -> Vec<u8> {
    [
        family.to_le_bytes().to_vec(),
        sub.to_le_bytes().to_vec(),
        fields
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect(),
    ]
    .concat()
}

/// Bar 1, slot 0 (the basic attack) with a target guid.
fn basic(target: u64) -> Vec<u8> {
    [command(36, 10, &[1, 0, 0]), target.to_le_bytes().to_vec()].concat()
}

fn at(position: [f32; 3]) -> CharacterState {
    CharacterState { position, ..who() }
}

/// Bar 1, slot `slot`, target mode 0 (self cast uses the player's own guid).
fn cast(slot: i32, target: u64) -> Vec<u8> {
    [
        command(36, 10, &[1, slot, 0]),
        target.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn failed(out: &Outcome) -> bool {
    out.replies.len() == 1 && out.replies[0].starts_with(&[36, 0, 1, 0])
}

fn toolbar(out: &Outcome) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .rev()
        .find(|p| p.starts_with(&[36, 0, 5, 0]))
}

/// Decode the eight toolbar slots; index 2 is the power-up's key (`3`).
fn slot_ids(bar: &[u8]) -> [Option<i32>; 8] {
    assert!(bar.starts_with(&[36, 0, 5, 0]));
    assert_eq!(i32::from_le_bytes(bar[8..12].try_into().unwrap()), 8);
    let mut at = 12;
    let mut ids = [None; 8];
    for id in &mut ids {
        let kind = i32::from_le_bytes(bar[at..at + 4].try_into().unwrap());
        match kind {
            0 => at += 4,
            3 => {
                *id = Some(i32::from_le_bytes(bar[at + 4..at + 8].try_into().unwrap()));
                at += 41;
            }
            other => panic!("unexpected toolbar entry type {other}"),
        }
    }
    assert_eq!(at, bar.len());
    ids
}

fn definition_ids(out: &Outcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|p| p.len() == 160 && p.starts_with(&[36, 0, 13, 0]))
        .map(|p| i32::from_le_bytes(p[4..8].try_into().unwrap()))
        .collect()
}

fn definition(out: &Outcome, id: i32) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .find(|p| p.starts_with(&[36, 0, 13, 0]) && p[4..8] == id.to_le_bytes())
}

/// Model ids of the pickups (`AddNpc`) in one outcome.
fn pickup_models(out: &Outcome) -> Vec<i32> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .filter(|p| {
            u64::from_le_bytes(p[4..12].try_into().unwrap()) & 0xffff_ffff_0000_0000
                == 0x5057_5550_0000_0000
        })
        .map(|p| i32::from_le_bytes(p[16..20].try_into().unwrap()))
        .collect()
}

/// Targets of the combat damage results in one outcome.
fn hit_targets(out: &Outcome) -> Vec<u64> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| u64::from_le_bytes(p[20..28].try_into().unwrap()))
        .collect()
}

/// The damage one `32/7` result reports for `guid`.
fn hit_damage(out: &Outcome, guid: u64) -> Option<i32> {
    out.replies
        .iter()
        .find(|p| p.starts_with(&[32, 0, 7, 0]) && p[20..28] == guid.to_le_bytes())
        .map(|p| i32::from_le_bytes(p[28..32].try_into().unwrap()))
}

/// True when `guid` attacked in this outcome (`32/7` attacker).
fn attacked(out: &Outcome, guid: u64) -> bool {
    out.replies
        .iter()
        .any(|p| p.starts_with(&[32, 0, 7, 0]) && p[4..12] == guid.to_le_bytes())
}

/// True when the outcome carries the client's `com_stunned` clip for `guid`.
fn stunned(out: &Outcome, guid: u64) -> bool {
    out.replies.iter().any(|p| {
        p.starts_with(&[35, 0, 8, 0])
            && p[4..12] == guid.to_le_bytes()
            && i32::from_le_bytes(p[12..16].try_into().unwrap()) == 1401
    })
}

fn enemy_health(out: &Outcome, guid: u64) -> Option<i32> {
    out.replies
        .iter()
        .rev()
        .find(|p| p.starts_with(&[35, 0, 5, 0]) && p[4..12] == guid.to_le_bytes())
        .map(|p| i32::from_le_bytes(p[12..16].try_into().unwrap()))
}

fn player_health(game: &Gameplay) -> u32 {
    game.state().encounter.unwrap().player_health
}

/// Load the reference package, force a Brawler level and switch to it through
/// the real 26/13 request. Returns the bar right after the switch.
fn fresh_game(fixture: &Fixture, xp: u32) -> (Gameplay, CharacterState, Vec<u8>) {
    let base = Gameplay::load(fixture.content(), None).unwrap();
    let mut seed = base.state();
    seed.profiles.as_mut().unwrap().experience.insert(43, xp);
    let mut game = Gameplay::load(fixture.content(), Some(seed)).unwrap();
    let location = who();
    let switched = game
        .handle_in_world(&command(26, 13, &[43]), &location, 100)
        .unwrap();
    let bar = toolbar(&switched)
        .expect("a job switch sends the job bar")
        .clone();
    game.initialize_world(&location, 200).unwrap();
    (game, location, bar)
}

fn interact(guid: u64) -> Vec<u8> {
    [vec![26, 0, 8, 0], guid.to_le_bytes().to_vec(), vec![0]].concat()
}

/// Click a battle marker (26/8 -> 41/114), send Go (41/108) and perform the
/// arena zone change the gateway owns, then spawn the arena.
fn enter_battle(
    game: &mut Gameplay,
    marker: [f32; 3],
    spawner: u64,
    now_ms: u64,
) -> CharacterState {
    let here = at(marker);
    let screen = game
        .handle_in_world(&interact(spawner), &here, now_ms)
        .unwrap();
    let details = screen
        .replies
        .iter()
        .find(|p| p.starts_with(&[41, 0, 114, 0]))
        .expect("the marker opens the battle start screen");
    let mut go = vec![41, 0, 108, 0];
    go.extend(&details[4..12]);
    go.extend(0_u64.to_le_bytes());
    let entered = game.handle_in_world(&go, &here, now_ms + 100).unwrap();
    let zone = entered.zoning.expect("Go zones into the arena");
    let arena = CharacterState {
        zone_id: zone.zone_id,
        position: zone.position,
        rotation: zone.rotation,
        ..at([0.0; 3])
    };
    game.initialize_world(&arena, now_ms + 200).unwrap();
    arena
}

/// Hit the always-present Hooligan Wolf until the guaranteed drop appears.
fn kill_target(game: &mut Gameplay, location: &CharacterState, start: u64) -> u64 {
    for attempt in 0..12 {
        let now = start + attempt * 700;
        let out = game.handle_in_world(&basic(TARGET), location, now).unwrap();
        if !pickup_models(&out).is_empty() {
            return now;
        }
    }
    panic!("the Hooligan Wolf never dropped the guaranteed power-up");
}

/// Pick up the guaranteed kind at `TARGET_AT` and return the collection time.
fn hold(game: &mut Gameplay, location: &CharacterState) -> u64 {
    let kill = kill_target(game, location, 1_000);
    game.tick(location, kill + 200).unwrap();
    assert!(game.held_power_up().is_some(), "the pickup is held");
    kill
}

#[test]
fn a_held_power_up_uses_its_own_key_once_and_restores_the_job_bar() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, LEVEL_15_XP);
    let before = slot_ids(&before_bar);
    assert!(
        before[2].is_some(),
        "the level-15 Brawler has a slot-2 job ability"
    );
    let slot = fixture.slot("flame_wave");

    // The pickup puts the kind on its own key slot and leaves every other
    // job slot exactly where it was.
    let kill = kill_target(&mut game, &location, 1_000);
    let picked = game.tick(&location, kill + 200).unwrap();
    assert_eq!(game.held_power_up(), Some("flame_wave"));
    let held_bar = toolbar(&picked).expect("the pickup resends the bar");
    let held = slot_ids(held_bar);
    assert_eq!(held[2], Some(slot.definition_id));
    assert_eq!(held[..2], before[..2]);
    assert_eq!(held[3..], before[3..]);
    let power_up_definition = definition(&picked, slot.definition_id)
        .expect("the power-up definition precedes its toolbar");
    assert_eq!(
        i32::from_le_bytes(power_up_definition[10..14].try_into().unwrap()),
        slot.name_id
    );
    assert_eq!(
        i32::from_le_bytes(power_up_definition[18..22].try_into().unwrap()),
        slot.icon_id
    );

    // The client's key request (bar 1, slot 2) consumes the kind and restores
    // the authored job row. This cast happens at a clear point with no target
    // in the Flame Wave radius.
    let use_at = kill + 1_000;
    let used = game
        .handle_in_world(&cast(2, 1), &at(CLEAR_AT), use_at)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    let restored = toolbar(&used).expect("the use resends the restored bar");
    assert_eq!(slot_ids(restored), before);
    assert!(!definition_ids(&used).contains(&slot.definition_id));
    assert!(hit_targets(&used).is_empty(), "no target in range");
}

#[test]
fn the_f1_pickup_point_hits_the_co_located_mid_camp_wolf() {
    // T415 re-scope: the pickup is still taken at F1, but T245's level-5 MID-F1
    // wolf no longer stands there. A cast at the F1 centre reaches no actor more
    // than two levels from the dead starter wolf; a cast at MID-F1's new home
    // hits it, leaves it alive and fabricates no drop.
    let fixture = Fixture::new();
    // A shipped actor's authored level (1 when absent); `None` for the player.
    let shipped_level = |guid| shipped_row(guid).map(|row| row["level"].as_i64().unwrap_or(1));
    let starter = shipped_level(TARGET).unwrap();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let at_f1 = game
        .handle_in_world(&cast(2, 1), &at(TARGET_AT), kill + 1_000)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    assert!(
        !hit_targets(&at_f1).contains(&TARGET),
        "the dead starter wolf is not hit again"
    );
    for guid in hit_targets(&at_f1) {
        if let Some(level) = shipped_level(guid) {
            assert!(
                (level - starter).abs() <= 2,
                "the F1 cast hit {guid} at level {level}, beside the level-{starter} starter camp"
            );
        }
    }

    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let home: [f32; 3] =
        serde_json::from_value(shipped_row(MID_F1).unwrap()["position"].clone()).unwrap();
    let used = game
        .handle_in_world(&cast(2, 1), &at(home), kill + 1_000)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    let hits = hit_targets(&used);
    assert!(
        hits.contains(&MID_F1),
        "a cast at MID-F1's home hits it: {hits:?}"
    );
    let health = enemy_health(&used, MID_F1).expect("the hit reports the wolf's health");
    assert!(
        health > 0,
        "a level-1 Flame Wave leaves the level-5 wolf alive"
    );
    assert!(
        pickup_models(&used).is_empty(),
        "a surviving wolf drops nothing"
    );
}

#[test]
fn the_flame_wave_burn_ticks_and_a_repeat_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    // At level 1 the tier-1 Flame Wave (damage_by_tier[0] = 1.0) leaves the
    // Hooligan alive, so the authored burn can tick on it.
    let moved = at(SECOND_AT);
    let use_at = kill + 1_000;
    let used = game.handle_in_world(&cast(2, 1), &moved, use_at).unwrap();
    assert_eq!(game.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&used).expect("the bar")),
        slot_ids(&before_bar)
    );
    let hits = hit_targets(&used);
    let mut unique = hits.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), hits.len(), "one initial hit per target");
    assert!(unique.contains(&SECOND), "the Flame Wave hits {hits:?}");
    let after_hit = enemy_health(&used, SECOND).expect("the target's health");
    assert!(after_hit > 0, "the target survives the initial hit");
    let ticked = game.tick(&moved, use_at + 1_100).unwrap();
    let after_burn = enemy_health(&ticked, SECOND).expect("the burn tick reports health");
    assert!(
        after_burn < after_hit,
        "the burn from power_ups.json ticks: {after_hit} -> {after_burn}"
    );

    // The kind is gone; the same key is now the (empty) job slot and a repeat
    // request is refused without consuming or damaging anything.
    let before = game.state();
    let replay = game
        .handle_in_world(&cast(2, 1), &moved, use_at + 2_000)
        .unwrap();
    assert!(failed(&replay), "nothing is held");
    assert_eq!(game.state(), before);
}

/// F263 blocker: the battle tier indexes the authored curve, and the level's
/// weapon scale is applied once (the old code multiplied both a second time).
#[test]
fn a_level_fifteen_flame_wave_uses_the_tier_one_basic_once() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, LEVEL_15_XP);
    let kill = hold(&mut game, &location);
    let moved = at(SECOND_AT);
    let used = game
        .handle_in_world(&cast(2, 1), &moved, kill + 1_000)
        .unwrap();
    let dealt = hit_damage(&used, SECOND).expect("the Flame Wave reaches the Hooligan");
    // The overworld is battle tier 1: a level-15 Brawler's basic is 938 and
    // the authored tier-1 multiplier is 1.0; a hit may crit for x1.5.
    assert!(
        dealt == 938 || dealt == 1_407,
        "a level-15 overworld Flame Wave hit for {dealt}"
    );
}

/// F263 blocker: a difficulty-5 fight indexes `damage_by_tier` with the
/// battle's tier, not with the player's level.
#[test]
fn a_difficulty_five_battle_scales_the_flame_wave_by_its_tier() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let arena = enter_battle(&mut game, TIER_FIVE_AT, TIER_FIVE_MARKER, kill + 300);
    assert!(
        game.state().encounter.as_ref().unwrap().active.is_some(),
        "the difficulty-5 battle starts"
    );
    let in_arena = CharacterState {
        position: [136.0, -0.6, 173.0],
        ..arena
    };
    let used = game
        .handle_in_world(&cast(2, 1), &in_arena, kill + 600)
        .unwrap();
    let dealt = hit_damage(&used, STONE_HEART).expect("the Flame Wave reaches Stone Heart");
    // Level-1 basic 100 x the authored tier-5 multiplier 9.38 (x1.5 on a crit).
    assert!(
        dealt == 938 || dealt == 1_407,
        "a tier-5 Flame Wave hit for {dealt}; the level-1 basic is 100"
    );
}

/// F263 finding 4: a shielded summoner takes the 0-damage hit but no stun.
#[test]
fn a_shielded_summoner_is_not_stunned_by_earth_shard() {
    let fixture = Fixture::with_drop("earth_shard");
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let arena = enter_battle(&mut game, SUMMONER_AT, SUMMONER_MARKER, kill + 300);
    let in_arena = CharacterState {
        position: [136.0, -0.6, 175.0],
        ..arena
    };
    // Three basics cross the summoner's 75 % threshold (and never its 50 %
    // threshold), so exactly the first minion pair stands.
    let mut now = kill + 500;
    for _ in 0..3 {
        now += 700;
        game.handle_in_world(&basic(SUMMONER), &in_arena, now)
            .unwrap();
    }
    assert!(
        game.state()
            .encounter
            .unwrap()
            .active
            .unwrap()
            .enemies
            .iter()
            .any(|s| s.guid == SUMMONER_MINION && !s.dormant),
        "the summoner's minions stand, so it is shielded"
    );
    let used = game
        .handle_in_world(&cast(2, 1), &in_arena, now + 700)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    assert!(
        !stunned(&used, SUMMONER),
        "a shielded summoner cannot be stunned"
    );
    assert_eq!(
        hit_damage(&used, SUMMONER),
        Some(0),
        "the shielded summoner takes the 0-damage hit"
    );
    assert!(
        hit_targets(&used).contains(&SUMMONER_MINION),
        "the exposed minion is in the area"
    );
}

/// F263 finding 4: a shielded summoner takes no burn, so once its minions die
/// (the shield drops) no authored tick has been waiting on it.
#[test]
fn a_shielded_summoner_is_not_burned_by_flame_wave() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let arena = enter_battle(&mut game, SUMMONER_AT, SUMMONER_MARKER, kill + 300);
    let in_arena = CharacterState {
        position: [136.0, -0.6, 175.0],
        ..arena
    };
    // Three basics bring the summoner below 75 % (and never below 50 %), so
    // exactly its first minion pair stands and shields it.
    let mut now = kill + 500;
    for _ in 0..3 {
        now += 700;
        game.handle_in_world(&basic(SUMMONER), &in_arena, now)
            .unwrap();
    }
    let use_at = now + 700;
    let used = game
        .handle_in_world(&cast(2, 1), &in_arena, use_at)
        .unwrap();
    let shielded_health = enemy_health(&used, SUMMONER).expect("the shielded hit reports health");
    // Kill the two minions from in reach of each; the summoner is then exposed.
    let mut exposed_at = use_at;
    for (guid, spot) in [
        (SUMMONER_MINION, [134.5, -0.6, 174.0]),
        (SUMMONER_MINION_B, [137.5, -0.6, 174.0]),
    ] {
        for _ in 0..2 {
            exposed_at += 700;
            let here = CharacterState {
                position: spot,
                ..in_arena.clone()
            };
            game.handle_in_world(&basic(guid), &here, exposed_at)
                .unwrap();
        }
    }
    let ticked = game.tick(&in_arena, exposed_at + 300).unwrap();
    assert_eq!(
        enemy_health(&ticked, SUMMONER).unwrap_or(shielded_health),
        shielded_health,
        "no burn was applied while the summoner was shielded"
    );
}

#[test]
fn earth_shard_stuns_an_enemy_and_the_stun_expires() {
    let fixture = Fixture::with_drop("earth_shard");
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let moved = at(SECOND_AT);
    let use_at = kill + 1_000;
    let used = game.handle_in_world(&cast(2, 1), &moved, use_at).unwrap();
    assert_eq!(game.held_power_up(), None);
    assert!(stunned(&used, SECOND), "Earth Shard stuns its area");
    let healthy = player_health(&game);
    // 2.5 s: the stunned Hooligan neither moves nor swings.
    let during = game.tick(&moved, use_at + 1_200).unwrap();
    assert!(!attacked(&during, SECOND), "a stunned enemy does not swing");
    assert_eq!(player_health(&game), healthy, "the player is untouched");
    // After the window the same enemy is free to attack again.
    let after = game.tick(&moved, use_at + 3_000).unwrap();
    assert!(attacked(&after, SECOND), "the stun expires");
    assert!(
        player_health(&game) < healthy,
        "the freed enemy deals damage"
    );
}

#[test]
fn super_shield_blocks_damage_for_its_window_then_expires() {
    let fixture = Fixture::with_drop("super_shield");
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let moved = at(SECOND_AT);
    let use_at = kill + 1_000;
    let used = game.handle_in_world(&cast(2, 1), &moved, use_at).unwrap();
    assert_eq!(game.held_power_up(), None);
    assert!(
        used.replies.iter().any(|p| p.starts_with(&[38, 0, 16, 0])),
        "the HUD receives the shield's effect tag"
    );
    let healthy = player_health(&game);
    // Within the 5 s window the enemy still swings but deals no damage.
    let during = game.tick(&moved, use_at + 1_200).unwrap();
    assert!(attacked(&during, SECOND), "the shielded enemy still swings");
    assert_eq!(player_health(&game), healthy, "the shield takes no damage");
    // After the window the same swing lands.
    let after = game.tick(&moved, use_at + 6_200).unwrap();
    assert!(attacked(&after, SECOND));
    assert!(
        player_health(&game) < healthy,
        "the shield expired and the hit landed"
    );
}

/// F263 finding 7: the same 0-damage mitigation fold applies to an instanced
/// fight's enemy attacks, not only to the shared field.
#[test]
fn super_shield_blocks_instance_damage_for_its_window_then_expires() {
    let fixture = Fixture::with_drop("super_shield");
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let arena = enter_battle(&mut game, TIER_FIVE_AT, TIER_FIVE_MARKER, kill + 300);
    let in_arena = CharacterState {
        position: [136.0, -0.6, 173.0],
        ..arena
    };
    let use_at = kill + 700;
    let used = game
        .handle_in_world(&cast(2, 1), &in_arena, use_at)
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    assert!(
        used.replies.iter().any(|p| p.starts_with(&[38, 0, 16, 0])),
        "the HUD receives the shield's effect tag"
    );
    let healthy = player_health(&game);
    // Stone Heart stands 2 m away, inside the 2.5 m reach, so the next tick
    // swings inside the 5 s window and the mitigation fold reports 0 damage.
    let during = game.tick(&in_arena, use_at + 1_200).unwrap();
    assert!(attacked(&during, STONE_HEART), "the boss still swings");
    assert_eq!(player_health(&game), healthy, "the shield takes no damage");
    // After the window the same boss lands its hit.
    let after = game.tick(&in_arena, use_at + 6_200).unwrap();
    assert!(attacked(&after, STONE_HEART));
    assert!(
        player_health(&game) < healthy,
        "the shield expired and the hit landed"
    );
}

#[test]
fn a_use_request_with_nothing_held_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    assert_eq!(
        slot_ids(&before_bar)[2],
        None,
        "level-1 Brawler has no slot-2 job ability"
    );
    let before = game.state();
    let out = game.handle_in_world(&cast(2, 1), &location, 1_000).unwrap();
    assert!(failed(&out), "an empty key slot is refused");
    assert_eq!(game.state(), before, "a refused use changes nothing");
    assert_eq!(game.held_power_up(), None);
}

#[test]
fn a_knocked_out_player_keeps_the_power_up_and_changes_nothing() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    game.receive_field_damage(u32::MAX / 2, kill + 300, true)
        .unwrap();
    assert_eq!(game.state().encounter.unwrap().player_health, 0);
    let before = game.state();
    let refused = game
        .handle_in_world(&cast(2, 1), &location, kill + 400)
        .unwrap();
    assert!(failed(&refused), "a knocked-out player gets 36/1");
    assert_eq!(game.state(), before, "the refusal changes nothing");
    assert_eq!(
        game.held_power_up(),
        Some("flame_wave"),
        "the kind stays held"
    );
}

#[test]
fn a_held_power_up_is_session_state_and_a_reconnect_clears_it() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let _ = hold(&mut game, &location);

    // The held kind is never written to the character save (W158 design): a
    // reconnect starts with the job bar and no kind.
    let saved = game.state();
    let mut reconnected = Gameplay::load(fixture.content(), Some(saved)).unwrap();
    let init = reconnected.initialize_world(&location, 5_000).unwrap();
    assert_eq!(reconnected.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&init).expect("world entry sends the bar"))[2],
        slot_ids(&before_bar)[2],
        "the reconnect shows the authored job bar"
    );
    let before = reconnected.state();
    let out = reconnected
        .handle_in_world(&cast(2, 1), &location, 6_000)
        .unwrap();
    assert!(failed(&out) && !out.changed);
    assert_eq!(reconnected.state(), before);
}

#[test]
fn the_shared_field_route_uses_the_same_key_and_consumes_once() {
    let fixture = Fixture::new();
    let (mut game, location, before_bar) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    // Move the field simulation to its runtime, exactly as the gateway does.
    let mut field = game
        .take_shared_fields()
        .expect("the field enemies move to the shared runtime");
    let moved = at(SECOND_AT);
    let action = SharedFieldAction::Ability { slot: 2, target: 1 };
    let used = game
        .handle_shared_field(
            &action,
            &mut field,
            &BTreeSet::from([TARGET, SECOND]),
            &moved,
            kill + 1_000,
        )
        .unwrap();
    assert_eq!(game.held_power_up(), None);
    assert_eq!(
        slot_ids(toolbar(&used).expect("the use resends the restored bar")),
        slot_ids(&before_bar)
    );
    let hit = hit_targets(&used);
    assert!(
        hit.contains(&SECOND),
        "the nearby shared enemy takes the hit: {hit:?}"
    );
    assert!(
        hit.iter().all(|guid| *guid != TARGET),
        "the dead enemy is not hit again"
    );
}

/// F263 finding 6: the power-up's area obeys the sender's interest set, like
/// a job ability's area.
#[test]
fn a_shared_field_use_skips_an_enemy_outside_the_senders_interest() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let mut field = game
        .take_shared_fields()
        .expect("the field enemies move to the shared runtime");
    let moved = at(SECOND_AT);
    let action = SharedFieldAction::Ability { slot: 2, target: 1 };
    let health_before = field.health(SECOND);
    // The sender has been told about the dead wolf only; the second Hooligan
    // stands inside the radius but outside their interest.
    let known = BTreeSet::from([TARGET]);
    let used = game
        .handle_shared_field(&action, &mut field, &known, &moved, kill + 1_000)
        .unwrap();
    assert_eq!(game.held_power_up(), None, "the kind is still consumed");
    assert!(
        !hit_targets(&used).contains(&SECOND),
        "an unseen enemy is not hit"
    );
    assert_eq!(field.health(SECOND), health_before, "and takes no damage");
}

/// F263 blocker: a derby return used to send the raw job bar, hiding a held
/// kind while the server still held it.
#[test]
fn a_derby_return_keeps_the_held_power_up_on_its_slot() {
    let fixture = Fixture::new();
    let (mut game, location, _) = fresh_game(&fixture, 0);
    let kill = hold(&mut game, &location);
    let slot = fixture.slot("flame_wave");
    let near = at([0.0, 0.0, 0.0]);
    let menu = game
        .handle_in_world(&npc_menu(), &near, kill + 300)
        .unwrap();
    assert!(
        menu.replies.iter().any(|p| p.starts_with(&[26, 0, 9])),
        "the gate opens its menu"
    );
    let entered = game
        .handle_in_world(&npc_choice(2), &near, kill + 400)
        .unwrap();
    assert!(
        game.derby_active() && entered.replies.iter().any(|p| p.starts_with(&[39, 0, 16])),
        "the gate's derby choice opens the offer"
    );
    let cancelled = game
        .handle_in_world(&[26, 0, 42, 0], &near, kill + 500)
        .unwrap();
    assert!(!game.derby_active());
    assert_eq!(
        game.held_power_up(),
        Some("flame_wave"),
        "the job switch keeps the held kind"
    );
    let bar = toolbar(&cancelled).expect("the derby return resends the toolbar");
    assert_eq!(
        slot_ids(bar)[2],
        Some(slot.definition_id),
        "slot 2 still shows the held kind"
    );
}

fn npc_menu() -> Vec<u8> {
    [vec![26, 0, 8, 0], GATE.to_le_bytes().to_vec(), vec![0]].concat()
}

fn npc_choice(choice: i32) -> Vec<u8> {
    [
        vec![26, 0, 10, 0],
        GATE.to_le_bytes().to_vec(),
        choice.to_le_bytes().to_vec(),
    ]
    .concat()
}
