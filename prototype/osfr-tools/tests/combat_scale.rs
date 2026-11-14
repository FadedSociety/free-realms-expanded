//! T639: combat on the live scale (`data/content/combat_scale.json`).
//! Tier ladder, health curve, heals and enemy rows all come from the data file;
//! the numbers pinned here are the live ones the note measured (889-based tier
//! ladder, 500 level-1 health), so a retune of the file moves the helpers.
use osfr_tools::{
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterOutcome, Rank, scale, wire::StatValue},
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
    /// Add a second world enemy (guid 300, same row as the cave's) so a save holds world enemies.
    fn world_enemy(&self) {
        let path = self.0.join("encounters.json");
        let mut content: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let mut actor = content["dungeons"][0]["enemies"][0].clone();
        actor["guid"] = json!(300);
        content["world_enemies"] = json!([actor]);
        fs::write(path, content.to_string()).unwrap();
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
    // 900 health) needed 3 hits to kill it and took 36 to die; with every job on the one
    // health curve (500 at level 1, no 0.9 for a Medic) it takes 39.
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
    // 3 and 39 before the owner's x2 health and x1.5 hits (rc-0137).
    let (health_x, hit_x) = combat_scale::multipliers();
    let (to_kill, to_die) = ((3.0 * health_x).round() as u64, (39.0 / hit_x).round() as u32);
    assert!(kills.abs_diff(to_kill) <= 1, "hits to kill {kills}");
    assert!(hits_to_die.abs_diff(to_die) <= 1, "hits to die {hits_to_die}");
}

#[test]
fn enemy_rows_are_scaled_once_where_they_load() {
    let f = Fixture::new(180, 25);
    let mut e = f.load();
    assert_eq!(
        combat_scale::enemy_health(180),
        (180.0 * 3.72 * combat_scale::multipliers().0).round() as u32,
        "180 x 3.72 = 669.6, times the owner's health multiplier"
    );
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
    let bad: [(&str, fn(&mut Value)); 25] = [
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
        ("wrong schema", |v| v["schema_version"] = json!(3)),
        ("the old schema", |v| v["schema_version"] = json!(1)),
        ("a rank is missing", |v| {
            v["ranks"].as_object_mut().unwrap().remove("elite");
        }),
        ("rank health 0", |v| v["ranks"]["boss"]["hp_basics"] = json!(0)),
        ("a ladder one tier short", |v| {
            v["ability_damage"]["super_heavy"] = json!([1143, 1998, 3492, 6107])
        }),
        ("no plain basic ladder", |v| v["plain_basic"] = json!("basic")),
        ("hit share over 1", |v| {
            v["enemy_hit"]["floor_health_share"] = json!(1.5)
        }),
        ("defense one tier short", |v| {
            v["defense"]["piece"] = json!([12, 23, 35, 47])
        }),
        ("unknown stat source", |v| v["stats"][0]["from"] = json!("luck")),
        ("ranks without both sources", |v| {
            let rows = v["source"].as_array_mut().unwrap();
            let captured = rows
                .iter()
                .position(|r| r["field"] == "ranks" && r["kind"] == "capture");
            rows.remove(captured.unwrap());
        }),
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
    fs::write(f.0.join("combat_scale.json"), vec![b' '; 17 * 1024]).unwrap();
    let large = EncounterEngine::load(&f.0, None).err().unwrap();
    assert!(large.to_string().contains("combat_scale.json"), "{large}");
    f.scale(|_| {}); // canary: the unedited shipped file loads
    assert!(EncounterEngine::load(&f.0, None).is_ok());
}

#[test]
fn saved_enemy_health_survives_a_scale_retune_in_both_directions() {
    let f = Fixture::new(180, 0);
    f.world_enemy();
    let mut e = EncounterEngine::load(&f.0, None).unwrap();
    let home = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0; 3],
    };
    e.begin(411, home, 43).unwrap();
    let mut saved = e.state().clone();
    let old_max = combat_scale::enemy_health(180);
    let hurt = 100; // below every retuned max below: kept as it is
    saved.world_enemies[0].health = hurt;
    assert_eq!(saved.active.as_ref().unwrap().enemies[0].health, old_max);
    for (scale, label) in [(2.0_f64, "down"), (8.0, "up")] {
        f.scale(|v| v["enemy_health_scale"] = json!(scale));
        let max = (180.0 * scale * combat_scale::multipliers().0).round() as u32;
        let loaded = EncounterEngine::load(&f.0, Some(saved.clone()))
            .unwrap_or_else(|e| panic!("retune {label}: {e}"));
        let state = loaded.state();
        let active = &state.active.as_ref().unwrap().enemies;
        assert_eq!(state.world_enemies.len() + active.len(), 2);
        assert_eq!(
            state.world_enemies[0].health, hurt,
            "{label}: a hurt enemy keeps its health"
        );
        assert_eq!(
            active[0].health,
            old_max.min(max),
            "{label}: a full-health cave enemy is clamped to the new max, never above it"
        );
        assert!(active[0].health <= max);
    }
}

// Combat P4: ranks, tiers, the enemy hit and Defense (combat design section 5).

fn shipped_scale() -> Value {
    serde_json::from_slice(
        &fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/content/combat_scale.json"
        ))
        .unwrap(),
    )
    .unwrap()
}

/// The three captured kinds, exactly: Bixie Soldier 260 (minion, tier 1), Frostfang Growler
/// 760 with its Alpha at 3,800 (normal and boss, tier 1), Feathered Drake 16,143 (tough,
/// tier 5). Both the free functions and a loaded engine answer them.
#[test]
fn the_rank_table_reproduces_the_three_captured_anchors() {
    let anchors = [
        (Rank::Minion, 1, 260),
        (Rank::Normal, 1, 760),
        (Rank::Boss, 1, 3_800),
        (Rank::Tough, 5, 16_143),
    ];
    let engine = Fixture::new(100, 0).load();
    for (rank, tier, captured) in anchors {
        // As captured, times the owner's x2 (rc-0137).
        let health = scale::live_health(captured);
        assert_eq!(scale::enemy_health(rank, tier, 1), health, "{rank:?}");
        let line = &engine.kill_table(WARRIOR, rank, Some(tier), (None, None), &[1])[0];
        assert_eq!(line.enemy_health, health, "{rank:?} through the engine");
    }
    assert_eq!(
        scale::enemy_health(Rank::Boss, 1, 1),
        5 * scale::enemy_health(Rank::Normal, 1, 1),
        "a battle boss is five of its regulars"
    );
    // Each further group member adds the group step.
    let step = shipped_scale()["group_step"].as_f64().unwrap();
    for group in 1..=4_u8 {
        // Times the owner's x2 (rc-0137).
        let expected = 279.0 * 2.724 * (1.0 + step * f64::from(group - 1)) * combat_scale::multipliers().0;
        assert_eq!(
            scale::enemy_health(Rank::Normal, 1, group),
            expected.round() as u32
        );
    }
    assert_eq!(scale::enemy_health(Rank::Normal, 1, 0), scale::live_health(760), "no group is one player");
    // Health rises with rank at every tier and with tier at every rank.
    let ranks = [
        Rank::Object,
        Rank::Normal,
        Rank::Tough,
        Rank::Elite,
        Rank::Boss,
        Rank::DungeonBoss,
        Rank::NamedBoss,
        Rank::WorldBoss,
    ];
    for tier in 1..=5 {
        let column = ranks.map(|rank| scale::enemy_health(rank, tier, 1));
        assert!(column.windows(2).all(|w| w[0] < w[1]), "tier {tier}: {column:?}");
    }
    for rank in ranks {
        let row = [1, 2, 3, 4, 5].map(|tier| scale::enemy_health(rank, tier, 1));
        assert!(row.windows(2).all(|w| w[0] < w[1]), "{rank:?}: {row:?}");
    }
    // A tier outside the table is the nearest end, never a panic.
    assert_eq!(scale::enemy_health(Rank::Minion, 0, 1), scale::live_health(260));
    assert_eq!(scale::enemy_health(Rank::Tough, 9, 1), scale::live_health(16_143));
}

/// The ability ladder is the file's, and each class steps by the tier ratio.
#[test]
fn the_ability_ladder_comes_from_the_file_and_steps_by_the_tier_ratio() {
    let file = shipped_scale();
    let ratio = file["tier_ratio"].as_f64().unwrap();
    let ladders = file["ability_damage"].as_object().unwrap();
    assert_eq!(ladders.len(), 5);
    for (class, ladder) in ladders {
        let ladder: Vec<u32> = (ladder.as_array().unwrap().iter())
            .map(|v| v.as_u64().unwrap() as u32)
            .collect();
        for (index, damage) in ladder.iter().enumerate() {
            assert_eq!(scale::ability_damage(class, index as u8 + 1), *damage, "{class}");
        }
        for pair in ladder.windows(2) {
            let step = f64::from(pair[1]) / f64::from(pair[0]);
            assert!((step - ratio).abs() < ratio * 0.01, "{class}: {pair:?}");
        }
    }
    assert_eq!(scale::ability_damage("basic_plain", 1), 279);
    assert_eq!(scale::ability_damage("super_standard", 2), 1_554);
    assert_eq!(scale::ability_damage("super_heavy", 5), 10_674);
    assert_eq!(scale::ability_damage("no_such_class", 3), 0);
}

/// An enemy's normal hit is 5 % of a player's base health at the first level of the tier
/// (levels 1, 4, 8, 12, 16), so 25 at tier 1 as the Bixie Soldier bit. The rule answers 204
/// at tier 5; the Feathered Drake's captured 206 is its own row (`enemy_catalogue.rs`).
#[test]
fn an_enemy_hit_is_five_percent_of_health_at_the_tier_floor() {
    let file = shipped_scale();
    let share = file["enemy_hit"]["floor_health_share"].as_f64().unwrap();
    let floors: Vec<u8> = (file["tier_min_levels"].as_array().unwrap().iter())
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    let hits = [1, 2, 3, 4, 5].map(|tier| scale::enemy_hit(Rank::Normal, tier));
    // The rule's hits times the owner's x1.5 (rc-0137).
    assert_eq!(hits, [25, 38, 67, 117, 204].map(scale::live_hit));
    for (tier, floor) in floors.iter().enumerate() {
        assert_eq!(scale::tier_of_level(*floor), tier as u8 + 1);
        let expected = (f64::from(scale::player_health(*floor)) * share).round() as u32;
        assert_eq!(hits[tier], scale::live_hit(expected), "tier {} from level {floor}", tier + 1);
    }
    assert_eq!(scale::enemy_hit(Rank::Minion, 1), scale::live_hit(25));
    assert_eq!(scale::enemy_hit(Rank::Tough, 5), scale::live_hit(204));
    assert_eq!(
        scale::enemy_hit(Rank::Elite, 1),
        (37.5 * combat_scale::multipliers().1).round() as u32,
        "25 x1.5 (elite), times the owner's damage multiplier"
    );
    assert_eq!(scale::enemy_hit(Rank::Boss, 5), scale::live_hit(408));
    assert_eq!(scale::special_hit(206), 309, "the captured special bite");
    assert_eq!(scale::enemy_hit_interval_ms(), 2_300);
    // A Bixie Soldier ticked +2 a second; a crate does not heal.
    assert_eq!(scale::enemy_regen(Rank::Minion, 260), 2);
    assert_eq!(scale::enemy_regen(Rank::Object, 260), 0);
    assert_eq!(scale::speeds("large"), Some([6.0, 12.0, 10.0]));
    assert_eq!(scale::speeds("flying"), None);
}

/// The one mitigation anchor: a raw 206 took 113 off a level 20 with Defense 348.
#[test]
fn defense_and_absorb_reduce_a_raw_hit() {
    assert_eq!(scale::mitigate(206, 348, 0, 5), 113);
    assert_eq!(scale::mitigate(206, 0, 0, 5), 206, "no Defense, the raw hit");
    assert_eq!(scale::mitigate(206, 348, 10, 5), 103, "Absorb comes off flat, last");
    assert_eq!(scale::mitigate(5, 348, 10, 5), 0, "never below nothing");
    // A full set of the tier removes the same share at every tier.
    for tier in 1..=5_u8 {
        let piece = shipped_scale()["defense"]["piece"][usize::from(tier) - 1]
            .as_u64()
            .unwrap() as u32;
        let taken = scale::mitigate(1_000, 6 * piece, 0, tier);
        assert!(taken.abs_diff(549) <= 1, "tier {tier}: {taken}");
    }
    // More Defense never means more damage.
    let curve: Vec<u32> = (0..=10).map(|n| scale::mitigate(1_000, n * 100, 0, 5)).collect();
    assert!(curve.windows(2).all(|w| w[0] > w[1]), "{curve:?}");
}

/// One health curve for every job, and the captured level-20 stat block built from it.
#[test]
fn player_health_and_the_stat_block_follow_the_one_curve() {
    assert_eq!(
        [1, 4, 16, 20].map(scale::player_health),
        [500, 761, 4_077, 7_135],
        "read on live: 500, 760, 4,069 and 7,116"
    );
    assert_eq!(scale::player_health(0), 500);
    assert_eq!(scale::player_health(99), scale::player_health(20));
    assert_eq!(
        [1, 3, 4, 7, 8, 11, 12, 15, 16, 20].map(scale::tier_of_level),
        [1, 1, 2, 2, 3, 3, 4, 4, 5, 5]
    );
    let rows = scale::stat_rows(scale::StatInput {
        level: 20,
        defense: 348,
        absorb: 10,
    });
    let row = |id: i32| rows.iter().find(|(stat, _)| *stat == id).map(|(_, v)| *v);
    assert_eq!(row(1), Some(StatValue::Int(7_135)), "Health");
    assert_eq!(row(7), Some(StatValue::Int(348)), "Defense");
    assert_eq!(row(16), Some(StatValue::Int(10)), "Absorb Damage");
    assert_eq!(row(3), Some(StatValue::Float(5.0)), "Weapon Range");
    assert_eq!(row(9), Some(StatValue::Int(5)), "melee crit chance");
    assert_eq!(row(10), Some(StatValue::Float(1.25)));
    assert_eq!(row(33), Some(StatValue::Int(3)), "ability crit chance");
    assert_eq!(row(34), Some(StatValue::Float(1.5)));
    assert_eq!(row(15), Some(StatValue::Int(1_000)), "Weapon Speed");
    // Live sent +71 and +356 on 7,116: 1 % in a fight, 5 % out of one.
    assert_eq!(row(65), Some(StatValue::Int(71)));
    assert_eq!(row(4), Some(StatValue::Int(357)));
    assert_eq!(rows.len(), shipped_scale()["stats"].as_array().unwrap().len());
}

/// The simulator's kill table: at levels 1, 5, 10, 15 and 20 every combat job kills an enemy
/// of each rank at its own tier inside the seconds the rank's row allows, and an ordinary
/// enemy needs 20 hits (46 s) to empty a level-1 player who has no Defense.
#[test]
fn the_kill_table_sits_in_each_ranks_band_at_every_level() {
    let engine = Fixture::new(100, 0).load();
    let ranks = [
        Rank::Object,
        Rank::Minion,
        Rank::Normal,
        Rank::Tough,
        Rank::Elite,
        Rank::Boss,
        Rank::DungeonBoss,
        Rank::NamedBoss,
        Rank::WorldBoss,
    ];
    for profile in [2, 11, 12, 32, 35, 43] {
        for rank in ranks {
            let [low, high] = engine.solo_seconds(rank);
            assert!(low > 0.0 && low < high, "{rank:?}");
            let table = engine.kill_table(profile, rank, None, (None, None), &[1, 5, 10, 15, 20]);
            assert_eq!(table.len(), 5);
            for line in table {
                assert_eq!(line.tier, scale::tier_of_level(line.level));
                assert_eq!(line.hits, line.enemy_health.div_ceil(line.strike));
                assert!(
                    (low..=high).contains(&line.seconds),
                    "job {profile} level {} against {rank:?}: {} hits, {:.2} s, outside {low}-{high}",
                    line.level,
                    line.hits,
                    line.seconds
                );
            }
        }
    }
    let line = &engine.kill_table(WARRIOR, Rank::Normal, None, (None, None), &[1])[0];
    // 25 a hit times the owner's damage multiplier, against a level-one 500, 2.3 s apart
    // (20 hits and 46 s at x1).
    let (hit, fall) = (scale::live_hit(25), 500_u32.div_ceil(scale::live_hit(25)));
    assert_eq!((line.enemy_hit, line.hits_to_fall), (hit, fall));
    assert!((line.seconds_to_fall - fall as f32 * 2.3).abs() < 0.01);
    // A kind's own numbers replace the rank's, as given.
    let own = &engine.kill_table(WARRIOR, Rank::Tough, Some(5), (Some(16_143), Some(206)), &[20])[0];
    assert_eq!((own.enemy_health, own.enemy_hit, own.hits), (16_143, 206, 7));
    // A retuned file moves the table.
    let f = Fixture::new(100, 0);
    f.scale(|v| v["ranks"]["tough"]["hp_basics"] = json!(12.375));
    let tuned = &f.load().kill_table(WARRIOR, Rank::Tough, Some(5), (None, None), &[20])[0];
    // 32_286 raw; the owner's x2 applies before the one rounding, so it can sit one above.
    let doubled = scale::live_health(32_286);
    assert!(tuned.enemy_health.abs_diff(doubled) <= 1, "{} vs {doubled}", tuned.enemy_health);
}

/// No combat number is a literal in the code that owns the scale: the numbers of design
/// section 5 and the rule constants the engine had before live only in the two data files.
#[test]
fn the_scale_code_holds_none_of_the_numbers() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let old = [
        // plain and rider basics, the supers, the tier ratio and the old enemy scales
        "254", "279", "488", "853", "1492", "2609", "889", "1143", "640", "1.75", "3.72",
        // rank multipliers and the group step
        "0.932", "2.724", "6.1875", "13.62", "60.0", "400.0", "1100.0",
        // enemy hits, rhythm, special
        "25.0", "44.0", "77.0", "133.0", "204.0", "206", "0.05", "2300", "2_300",
        // Defense
        "12.0", "23.0", "35.0", "47.0", "58.0", "1.215", "348",
        // player health and the rules the engine had as constants
        "500", "500.0", "600", "650", "0.85", "0.9", "1.15", "8_000", "15_000",
    ];
    for file in [
        "encounter/scale.rs",
        "encounter/scale/catalogue.rs",
        "encounter/combat_scale.rs",
        "encounter/combat_rules.rs",
        "gameplay/combat_sim.rs",
    ] {
        let text = fs::read_to_string(root.join(file)).unwrap();
        // Production code only: comments carry evidence, and the unit tests write fixtures.
        let code: String = text
            .split("#[cfg(test)]")
            .next()
            .unwrap()
            .lines()
            .map(|line| line.split("//").next().unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        let numbers = code
            .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '_'))
            .map(|token| token.trim_matches('.'))
            .filter(|token| token.starts_with(|c: char| c.is_ascii_digit()));
        for number in numbers {
            assert!(
                !old.contains(&number),
                "{file} holds the combat number {number}; it belongs in combat_scale.json or combat_rules.json"
            );
        }
    }
}
