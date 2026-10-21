use osfr_tools::{
    encounter::{EncounterEngine, EncounterOutcome},
    opening_abilities::JobAbility,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/job_playtest.rs"]
mod job_playtest;

static SERIAL: AtomicU64 = AtomicU64::new(0);
const GUID: u64 = 100;
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    fn new(level: u32, radius: f32) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w105-{}-{}",
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
        let actor = json!({"guid":GUID,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":100,"damage":7,"boss":false,"level":level,"level_band":[1,20],"zone_id":0,"region":"Synthetic region","spawn_area":{"id":"test","centre":[0,0,0],"radius":radius,"count":1},"leash_radius":12,"respawn_seconds":1,"aggro_range":8});
        let mut other = actor.clone();
        other["guid"] = json!(200);
        fs::write(dir.join("encounters.json"), json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[other],"reward_coins":0,"reward_items":[]}]}).to_string()).unwrap();
        fs::write(
            dir.join("field_enemies.json"),
            json!({"schema_version":1,"enemies":[actor]}).to_string(),
        )
        .unwrap();
        Self(dir)
    }
    fn load(&self) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, None).unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        e
    }
    fn edit(&self, key: &str, value: Value) {
        let path = self.0.join("field_enemies.json");
        let mut file: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        file["enemies"][0][key] = value;
        fs::write(path, file.to_string()).unwrap();
    }
}
fn at(e: &EncounterEngine) -> [f32; 3] {
    e.field_teleport_targets()
        .into_iter()
        .find(|a| a.guid == GUID)
        .unwrap()
        .position
}
fn hits_from(out: &EncounterOutcome, attacker: u64) -> Vec<u32> {
    job_playtest::hits_from(&out.replies, attacker)
}
fn timed_effect(effects: Value) -> JobAbility {
    serde_json::from_value(json!({"slot":1,"definition_id":62001,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":0,"damage_multiplier":0.0,"heal":0,"effects":effects})).unwrap()
}
fn hp(out: &EncounterOutcome) -> Option<(u32, u32)> {
    out.replies
        .iter()
        .find(|p| p.starts_with(&[35, 0, 5, 0]) && p[4..12] == GUID.to_le_bytes())
        .map(|p| {
            (
                u32::from_le_bytes(p[12..16].try_into().unwrap()),
                u32::from_le_bytes(p[16..20].try_into().unwrap()),
            )
        })
}
fn radius(p: [f32; 3]) -> f32 {
    p[0].hypot(p[2])
}

#[test]
fn home_leash_returns_and_heals_while_ignoring_reaggro() {
    let f = Fixture::new(1, 0.0);
    // Non-divisor of 12 m: an unchecked step crosses the leash clearly,
    // rather than landing at 12 m within floating-point tolerance.
    f.edit("behaviour", json!({"move_speed":3.1}));
    let mut e = f.load();
    assert!(hp(&e.attack(GUID, [0.0; 3], 11, 0, 0).unwrap()).unwrap().0 < 100);
    let mut returned = false;
    let mut farthest = 0.0_f32;
    for t in 1..=400 {
        let p = at(&e);
        // Keep within aggro while pulling past home leash. The distance give-up
        // alone cannot pass this test (canary: remove the home leash check).
        let out = e.tick([p[0] + 6.0, 0.0, 0.0], 11, t * 100).unwrap();
        farthest = farthest.max(radius(at(&e)));
        assert!(radius(at(&e)) <= 12.001);
        if at(&e)[0] < p[0] && p[0] > 1.0 {
            assert!(
                e.attack(GUID, at(&e), 11, 0, t * 100 + 1)
                    .unwrap()
                    .replies
                    .is_empty()
            );
        }
        if hp(&out) == Some((100, 100)) {
            returned = true;
            break;
        }
    }
    assert!(farthest > 10.0, "must pull to the home boundary");
    assert!(returned, "must heal on reaching home despite nearby player");
    assert!(radius(at(&e)) < 0.001);
}

#[test]
fn lost_target_returns_at_run_speed_then_heals() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    e.attack(GUID, [0.0; 3], 11, 0, 0).unwrap();
    for t in 1..=10 {
        e.tick([7.0, 0.0, 0.0], 11, t * 100).unwrap();
    }
    let before = at(&e)[0];
    assert!(before > 2.0);
    e.tick([100.0, 0.0, 0.0], 11, 1100).unwrap();
    assert!((before - at(&e)[0] - 0.35).abs() < 0.001);
    let mut healed = false;
    for t in 12..=30 {
        let out = e.tick(at(&e), 11, t * 100).unwrap();
        healed |= hp(&out) == Some((100, 100));
        if healed {
            break;
        }
    }
    assert!(healed);
}

#[test]
fn kill_then_respawn_forgets_skill_effects() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let stun = timed_effect(json!([{"kind":"stun","duration_ms":30000}]));
    let debuff = timed_effect(
        json!([{"kind":"buff","recipient":"target","duration_ms":30000,"damage_multiplier":1.0,"damage_taken_multiplier":2.0,"crit_bonus":0,"crit_damage_bonus":0.0}]),
    );
    let dot = timed_effect(
        json!([{"kind":"damage_over_time","multiplier":0.5,"duration_ms":30000,"interval_ms":1000}]),
    );
    e.job_ability(&stun, GUID, [0.0; 3], 11, 0).unwrap();
    e.job_ability(&debuff, GUID, [0.0; 3], 11, 700).unwrap();
    e.job_ability(&dot, GUID, [0.0; 3], 11, 1400).unwrap();
    let out = e.attack(GUID, [0.0; 3], 11, 0, 2100).unwrap();
    assert_eq!(hp(&out), Some((0, 100)), "the debuff lands the killing hit");
    let out = e.tick([100.0, 0.0, 0.0], 11, 3100).unwrap();
    assert_eq!(hp(&out), Some((100, 100)), "the kill respawns");
    let out = e.tick([100.0, 0.0, 0.0], 11, 4100).unwrap();
    assert!(
        hits_from(&out, 1).is_empty(),
        "damage over time was cleared"
    );
    let out = e.tick([0.0; 3], 11, 4200).unwrap();
    assert!(!hits_from(&out, GUID).is_empty(), "the stun was cleared");
    let out = e.attack(GUID, [0.0; 3], 11, 0, 4300).unwrap();
    assert_eq!(hp(&out), Some((25, 100)), "the target buff was cleared");
}

#[test]
fn evade_completion_forgets_skill_effects() {
    let f = Fixture::new(1, 0.0);
    f.edit("max_health", json!(1000));
    let mut e = f.load();
    let stun = timed_effect(json!([{"kind":"stun","duration_ms":30000}]));
    let debuff = timed_effect(
        json!([{"kind":"buff","recipient":"target","duration_ms":30000,"damage_multiplier":1.0,"damage_taken_multiplier":2.0,"crit_bonus":0,"crit_damage_bonus":0.0}]),
    );
    let dot = timed_effect(
        json!([{"kind":"damage_over_time","multiplier":0.5,"duration_ms":30000,"interval_ms":1000}]),
    );
    e.job_ability(&stun, GUID, [0.0; 3], 11, 0).unwrap();
    e.job_ability(&debuff, GUID, [0.0; 3], 11, 700).unwrap();
    e.job_ability(&dot, GUID, [0.0; 3], 11, 1400).unwrap();
    let out = e.attack(GUID, [0.0; 3], 11, 0, 2100).unwrap();
    assert_eq!(hp(&out).unwrap().0, 850, "the debuff doubles the hit");
    let out = e.tick([100.0, 0.0, 0.0], 11, 2200).unwrap();
    assert!(hits_from(&out, 1).is_empty(), "the dot was not due yet");
    assert_eq!(hp(&out), Some((1000, 1000)), "the evading actor heals home");
    let out = e.tick([100.0, 0.0, 0.0], 11, 3500).unwrap();
    assert!(
        hits_from(&out, 1).is_empty(),
        "damage over time was cleared"
    );
    let out = e.tick([0.0; 3], 11, 4200).unwrap();
    assert!(!hits_from(&out, GUID).is_empty(), "the stun was cleared");
    let out = e.attack(GUID, [0.0; 3], 11, 0, 4300).unwrap();
    assert_eq!(hp(&out), Some((925, 1000)), "the target buff was cleared");
}

#[test]
fn reload_field_enemies_forgets_skill_effects() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let stun = timed_effect(json!([{"kind":"stun","duration_ms":30000}]));
    let debuff = timed_effect(
        json!([{"kind":"buff","recipient":"target","duration_ms":30000,"damage_multiplier":1.0,"damage_taken_multiplier":2.0,"crit_bonus":0,"crit_damage_bonus":0.0}]),
    );
    let dot = timed_effect(
        json!([{"kind":"damage_over_time","multiplier":0.5,"duration_ms":30000,"interval_ms":1000}]),
    );
    e.job_ability(&stun, GUID, [0.0; 3], 11, 0).unwrap();
    e.job_ability(&debuff, GUID, [0.0; 3], 11, 700).unwrap();
    e.job_ability(&dot, GUID, [0.0; 3], 11, 1400).unwrap();
    e.reload_field_enemies(&f.0.join("field_enemies.json"))
        .unwrap();
    let out = e.tick([100.0, 0.0, 0.0], 11, 2200).unwrap();
    assert!(hits_from(&out, 1).is_empty(), "the dot was not due yet");
    let out = e.tick([100.0, 0.0, 0.0], 11, 3500).unwrap();
    assert!(
        hits_from(&out, 1).is_empty(),
        "damage over time was cleared"
    );
    let out = e.tick([0.0; 3], 11, 4200).unwrap();
    assert!(!hits_from(&out, GUID).is_empty(), "the stun was cleared");
    let out = e.attack(GUID, [0.0; 3], 11, 0, 4300).unwrap();
    assert_eq!(hp(&out), Some((25, 100)), "the target buff was cleared");
}

#[test]
fn roaming_is_seeded_bounded_and_pauses() {
    let f = Fixture::new(1, 6.0);
    let mut a = f.load();
    let mut b = f.load();
    let mut moved = 0;
    let mut paused = 0;
    for t in 0..3000 {
        let previous = at(&a);
        a.tick([100.0, 0.0, 0.0], 11, t * 100).unwrap();
        b.tick([100.0, 0.0, 0.0], 11, t * 100).unwrap();
        assert_eq!(at(&a), at(&b));
        assert!(radius(at(&a)) <= 6.001);
        if at(&a) != previous {
            moved += 1;
        } else {
            paused += 1;
        }
    }
    assert!(moved > 200, "idle actors must roam");
    assert!(paused > 100, "pause between goals");
}

#[test]
fn levels_scale_hp_damage_and_respawn() {
    let f = Fixture::new(1, 0.0);
    let mut one = f.load();
    let five = Fixture::new(5, 0.0);
    let mut high = five.load();
    assert_eq!(one.field_level(GUID), Some(1));
    assert_eq!(high.field_level(GUID), Some(5));
    assert_eq!(hp(&one.on_zone_ready(11, 0).unwrap()), Some((100, 100)));
    assert_eq!(hp(&high.on_zone_ready(11, 0).unwrap()), Some((175, 175)));
    let before = high.state().player_health;
    high.tick([0.0; 3], 11, 100).unwrap();
    assert_eq!(before - high.state().player_health, 21); // 7 * 1.75^(tier 3 - 1)
    for t in 0..3 {
        high.attack(GUID, [0.0; 3], 2, 0, 1000 + t * 700).unwrap();
    }
    assert!(!high.field_teleport_targets().iter().any(|a| a.guid == GUID));
    let out = high.tick([100.0; 3], 11, 4000).unwrap();
    assert_eq!(hp(&out), Some((175, 175)));
    assert_eq!(at(&high), [0.0; 3]);
}

#[test]
fn behaviour_leash_fallback_and_honest_stationary_reach() {
    let f = Fixture::new(1, 0.0);
    f.edit("leash_radius", Value::Null);
    f.edit("behaviour", json!({"leash":5.0,"move_speed":3.1}));
    let mut e = f.load();
    let mut turning = false;
    for t in 0..150 {
        let before = at(&e);
        e.tick([before[0] + 6.0, 0.0, 0.0], 11, t * 100).unwrap();
        assert!(radius(at(&e)) <= 5.001);
        turning |= at(&e)[0] < before[0];
    }
    assert!(turning);
    f.edit("behaviour", json!({"mode":"stationary","reach":2.5}));
    let mut e = f.load();
    let before = e.state().player_health;
    e.tick([2.6, 0.0, 0.0], 11, 100).unwrap();
    assert_eq!(e.state().player_health, before);
    e.tick([2.5, 0.0, 0.0], 11, 200).unwrap();
    assert_eq!(e.state().player_health, before - 7);
}

#[test]
fn roaming_follows_existing_terrain_heights() {
    use osfr_tools::terrain::{HeightGrid, Terrain};
    let f = Fixture::new(1, 6.0);
    let mut grid = HeightGrid::new([-20.0, -20.0], [20.0, 20.0], 1.0).unwrap();
    let a = [-20.0, -4.0, -20.0];
    let b = [20.0, 4.0, -20.0];
    let c = [20.0, 4.0, 20.0];
    let d = [-20.0, -4.0, 20.0];
    grid.add_triangle([a, b, c]);
    grid.add_triangle([a, c, d]);
    fs::write(f.0.join("zone_0.hgt"), grid.to_bytes()).unwrap();
    let mut e = f.load();
    e.set_terrain(Some(std::sync::Arc::new(Terrain::load(&f.0).unwrap())));
    let mut moved = false;
    for t in 0..500 {
        e.tick([100.0; 3], 11, t * 100).unwrap();
        let p = at(&e);
        moved |= radius(p) > 1.0;
        assert!((p[1] - grid.height_at(p[0], p[2]).unwrap()).abs() < 0.001);
        assert!(radius(p) <= 6.001);
    }
    assert!(moved);
}

#[test]
fn malformed_metadata_and_positions_fail_without_replacing_field() {
    let f = Fixture::new(1, 6.0);
    let mut e = f.load();
    for (key, value) in [
        ("level", json!(21)),
        ("level_band", json!([2, 3])),
        ("zone_id", json!(10)),
        ("leash_radius", json!(-1)),
        (
            "spawn_area",
            json!({"id":"bad","centre":[1,0,0],"radius":6,"count":1}),
        ),
    ] {
        let fresh = Fixture::new(1, 6.0);
        fresh.edit(key, value);
        assert!(
            e.reload_field_enemies(&fresh.0.join("field_enemies.json"))
                .is_err(),
            "{key}"
        );
        assert_eq!(at(&e), [0.0; 3]);
    }
    assert!(e.tick([f32::NAN, 0.0, 0.0], 11, 100).is_err());
    assert!(
        e.attack(GUID, [f32::INFINITY, 0.0, 0.0], 11, 0, 100)
            .is_err()
    );
    e.tick([0.0; 3], 11, 100).unwrap();
    let health = e.state().player_health;
    f.edit("level", json!(21));
    assert!(
        e.reload_field_enemies(&f.0.join("field_enemies.json"))
            .is_err()
    );
    e.tick([0.0; 3], 11, 200).unwrap();
    assert_eq!(
        e.state().player_health,
        health,
        "rejected reload must preserve enemy attack cooldown"
    );
}
