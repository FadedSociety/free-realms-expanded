//! T639: combat on the live scale (`data/content/combat_scale.json`).
//! Tier ladder, health curve, heals and enemy rows all come from the data file;
//! the numbers pinned here are the live ones the note measured (889-based tier
//! ladder, 500 level-1 health), so a retune of the file moves the helpers.
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

#[path = "common/combat_scale.rs"]
mod combat_scale;

static SERIAL: AtomicU64 = AtomicU64::new(0);
const GUID: u64 = 100;
const MEDIC: i32 = 11;
const WARRIOR: i32 = 32;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    /// One field enemy (`health` authored on the old scale, `damage` per hit) and
    /// a cave; no combat_scale.json, so the bundled one applies.
    fn new(health: u32, damage: u32) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "t639-{}-{}",
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
        let actor = json!({"guid":GUID,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic wolf","texture_alias":"","position":[0,0,0],"max_health":health,"damage":damage,"boss":false,"level":1,"level_band":[1,20],"zone_id":0,"region":"Synthetic region","spawn_area":{"id":"test","centre":[0,0,0],"radius":0,"count":1},"leash_radius":12,"respawn_seconds":3600,"aggro_range":8});
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
    /// Write a combat_scale.json next to the encounter content (it overrides the bundled one).
    fn scale(&self, edit: impl FnOnce(&mut Value)) {
        let mut file: Value = serde_json::from_slice(
            &fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/content/combat_scale.json"
            ))
            .unwrap(),
        )
        .unwrap();
        edit(&mut file);
        fs::write(self.0.join("combat_scale.json"), file.to_string()).unwrap();
    }
}

fn damages(out: &EncounterOutcome) -> Vec<u32> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()))
        .collect()
}

/// The lowest of six swings on a fresh engine, so a x1.5 critical never reads
/// as the basic and the enemy outlives the swings.
fn basic_hit(f: &Fixture, profile: i32, level: u32) -> u32 {
    let mut e = f.load();
    e.set_player_job(profile, level);
    (0..6)
        .map(|i| damages(&e.attack(GUID, [0.0; 3], profile, 0, i * 700).unwrap())[0])
        .min()
        .unwrap()
}

fn near(value: u32, target: f64, tolerance: f64) -> bool {
    (f64::from(value) - target).abs() <= target * tolerance
}

fn heal(amount: u32) -> JobAbility {
    serde_json::from_value(json!({"slot":1,"definition_id":62000,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":0,"damage_multiplier":0.0,"heal":amount,"effects":[]})).unwrap()
}

#[test]
fn a_level_eight_medic_hits_for_the_live_tier_three_basic() {
    let f = Fixture::new(10_000, 0);
    let hit = basic_hit(&f, MEDIC, 8);
    assert!(
        near(hit, 854.0, 0.01),
        "889 x 11/35 x 1.75^2 = 854, got {hit}"
    );
}

#[test]
fn tiers_open_at_levels_1_4_8_12_16() {
    let f = Fixture::new(10_000, 0);
    let tier1 = f64::from(combat_scale::basic(MEDIC));
    let ratio = combat_scale::tier_ratio();
    // (level, tier): level 5 is tier 2 (it was tier 3), 8-11 tier 3, 12-15 tier 4.
    for (level, tier) in [
        (1, 1),
        (3, 1),
        (4, 2),
        (5, 2),
        (7, 2),
        (8, 3),
        (11, 3),
        (12, 4),
        (15, 4),
        (16, 5),
        (20, 5),
    ] {
        let hit = basic_hit(&f, MEDIC, level);
        assert!(
            near(hit, tier1 * ratio.powi(tier - 1), 0.01),
            "level {level} should hit as tier {tier}, got {hit}"
        );
    }
}

#[test]
fn damage_steps_by_the_tier_ratio_and_health_follows_the_curve() {
    let f = Fixture::new(10_000, 0);
    let mut e = f.load();
    let (base, ratio) = (
        f64::from(combat_scale::base_health()),
        combat_scale::tier_ratio(),
    );
    let tiers = [1, 4, 8, 12, 16].map(|level| f64::from(basic_hit(&f, WARRIOR, level)));
    for pair in tiers.windows(2) {
        assert!((pair[1] / pair[0] - ratio).abs() < ratio * 0.01, "{pair:?}");
    }
    for level in 1..=20_u32 {
        e.set_player_job(WARRIOR, level);
        let max = e.player_health().1;
        let curve = base * ratio.powf(f64::from(level - 1) / 4.0);
        assert!(
            near(max, curve, 0.10),
            "level {level}: {max} against {curve}"
        );
    }
}

#[test]
fn a_level_one_player_has_the_base_health() {
    let f = Fixture::new(100, 0);
    let mut e = f.load();
    assert_eq!(e.player_health(), (500, 500));
    e.set_player_job(WARRIOR, 1);
    assert_eq!(
        e.player_health(),
        (500, 500),
        "a Warrior has the plain base"
    );
    assert_eq!(combat_scale::base_health(), 500);
}

#[test]
fn a_saved_player_above_the_new_maximum_is_clamped() {
    let f = Fixture::new(100, 0);
    let mut state = f.load().state().clone();
    state.player_health = 1_000; // the old level-1 maximum
    let mut e = EncounterEngine::load(&f.0, Some(state)).unwrap();
    let packets = e.set_player_job(WARRIOR, 1);
    assert_eq!(e.player_health(), (500, 500));
    assert!(!packets.is_empty(), "the client is told the new health");
}

#[test]
fn heals_follow_the_tier_ladder() {
    let f = Fixture::new(10_000, 0);
    let mut state = f.load().state().clone();
    state.player_health = 1;
    for level in [1, 4, 8, 12, 16] {
        let mut e = EncounterEngine::load(&f.0, Some(state.clone())).unwrap();
        e.load_field_enemies(&f.0.join("field_enemies.json"))
            .unwrap();
        e.set_player_job(MEDIC, level);
        e.job_ability(&heal(254), GUID, [0.0; 3], MEDIC, 0).unwrap();
        let healed = f64::from(e.player_health().0 - 1);
        let expected = 254.0 * combat_scale::tier_factor(level);
        assert!(
            (healed - expected).abs() <= 1.0,
            "level {level}: {healed} against {expected}"
        );
    }
}

#[test]
fn a_field_enemy_keeps_its_hits_to_kill_and_hits_to_die() {
    // Authored 180 health / 25 damage. On the old scale a level-1 Medic (75 a hit,
    // 900 health) needed 3 hits to kill it and took 36 to die.
    let f = Fixture::new(180, 25);
    let mut e = f.load();
    e.set_player_job(MEDIC, 1);
    let max = e.player_health().1;
    let before = e.player_health().0;
    e.tick([0.0; 3], MEDIC, 100).unwrap();
    let taken = before - e.player_health().0;
    let hits_to_die = max.div_ceil(taken);
    let mut kills = 0;
    while e.field_teleport_targets().iter().any(|a| a.guid == GUID) {
        e.attack(GUID, [0.0; 3], MEDIC, 0, 1_000 + kills * 700)
            .unwrap();
        kills += 1;
        assert!(kills < 20);
    }
    assert!(kills.abs_diff(3) <= 1, "hits to kill {kills}");
    assert!(hits_to_die.abs_diff(36) <= 1, "hits to die {hits_to_die}");
}

#[test]
fn enemy_rows_are_scaled_once_where_they_load() {
    let f = Fixture::new(180, 25);
    let mut e = f.load();
    assert_eq!(combat_scale::enemy_health(180), 670);
    e.set_player_job(MEDIC, 1);
    let first = damages(&e.attack(GUID, [0.0; 3], MEDIC, 0, 0).unwrap());
    assert_eq!(first, [combat_scale::basic(MEDIC)]);
    let hp = e
        .attack(GUID, [0.0; 3], MEDIC, 0, 700)
        .unwrap()
        .replies
        .iter()
        .find(|p| p.starts_with(&[35, 0, 5, 0]))
        .map(|p| u32::from_le_bytes(p[16..20].try_into().unwrap()));
    assert_eq!(
        hp,
        Some(combat_scale::enemy_health(180)),
        "max health sent to the client"
    );
}

#[test]
fn the_data_file_tunes_the_scale_without_code() {
    let f = Fixture::new(10_000, 0);
    f.scale(|v| v["tier_min_levels"] = json!([1, 2, 3, 4, 5]));
    let tier5 = f64::from(combat_scale::basic(MEDIC)) * combat_scale::tier_ratio().powi(4);
    assert!(
        near(basic_hit(&f, MEDIC, 5), tier5, 0.01),
        "level 5 is now tier 5"
    );
}

#[test]
fn an_invalid_scale_file_refuses_the_load() {
    let f = Fixture::new(100, 0);
    let bad: [(&str, fn(&mut Value)); 16] = [
        ("tiers must start at 1", |v| {
            v["tier_min_levels"] = json!([2, 4, 8])
        }),
        ("tiers must increase", |v| {
            v["tier_min_levels"] = json!([1, 4, 4, 12])
        }),
        ("tiers at most level 20", |v| {
            v["tier_min_levels"] = json!([1, 4, 21])
        }),
        ("no tiers", |v| v["tier_min_levels"] = json!([])),
        ("basic damage 0", |v| v["basic_damage"]["11"] = json!(0)),
        ("basic damage too big", |v| {
            v["basic_damage"]["11"] = json!(100_001)
        }),
        ("no fallback profile", |v| {
            v["basic_damage"].as_object_mut().unwrap().remove("0");
        }),
        ("base health too small", |v| {
            v["player_base_health"] = json!(99)
        }),
        ("base health too big", |v| {
            v["player_base_health"] = json!(100_001)
        }),
        ("health scale too small", |v| {
            v["enemy_health_scale"] = json!(0.001)
        }),
        ("damage scale too big", |v| {
            v["enemy_damage_scale"] = json!(101)
        }),
        ("tier ratio below 1", |v| v["tier_ratio"] = json!(0.5)),
        ("a value without a source", |v| v["source"] = json!([])),
        ("unknown field", |v| v["surprise"] = json!(1)),
        ("wrong schema", |v| v["schema_version"] = json!(2)),
        ("not a number", |v| v["tier_ratio"] = json!("high")),
    ];
    for (why, edit) in bad {
        f.scale(edit);
        let error = EncounterEngine::load(&f.0, None).err();
        assert!(error.is_some(), "{why} must refuse the load, not fall back");
        assert!(
            error.unwrap().to_string().contains("combat_scale.json"),
            "{why}: the error names the file"
        );
    }
    fs::write(f.0.join("combat_scale.json"), b"not json").unwrap();
    assert!(EncounterEngine::load(&f.0, None).is_err());
    f.scale(|_| {}); // canary: the unedited shipped file loads
    assert!(EncounterEngine::load(&f.0, None).is_ok());
}
