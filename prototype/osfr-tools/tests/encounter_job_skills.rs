use osfr_tools::{encounter::EncounterEngine, opening_abilities::JobAbility};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SERIAL: AtomicU64 = AtomicU64::new(0);

#[test]
fn unbuffed_critical_damage_keeps_existing_integer_rounding() {
    let f = Fixture::new(10000, 0);
    let mut e = f.load(11, 1); // existing odd basic damage 75
    let mut critical = false;
    for t in (0..44800).step_by(700) {
        let damage = hit_damages(&e.attack(100, [0.0; 3], 11, 0, t).unwrap())[0];
        if damage > 75 {
            assert_eq!(damage, 112, "75 * 3 / 2 rounds down");
            critical = true;
        }
    }
    assert!(critical);
}
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    fn new(health: u32, damage: u32) -> Self {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "w101-{}-{}",
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
        cave["name"] = json!("SyntheticCave");
        let content = json!({"schema_version":1,"template_file":"npc.bin","template_sha256":format!("{:x}",Sha256::digest(template)),"world":zone,"world_enemies":[],"dungeons":[{"id":411,"title_id":12,"description_id":13,"objective_id":14,"zone":cave,"guide":null,"enemies":[{"guid":200,"definition_id":401,"model_id":3,"name_id":100,"name":"Synthetic cave enemy","texture_alias":"","position":[0,0,0],"max_health":100,"damage":1,"boss":false}],"reward_coins":0,"reward_items":[]}]});
        fs::write(
            dir.join("encounters.json"),
            serde_json::to_vec(&content).unwrap(),
        )
        .unwrap();
        let enemies:Vec<_>=(100..103).map(|guid| json!({"guid":guid,"definition_id":400,"model_id":3,"name_id":100,"name":"Synthetic enemy","texture_alias":"","position":[2.0,0.0,(guid-100) as f32],"max_health":health,"damage":damage,"boss":false,"respawn_seconds":3600,"aggro_range":10})).collect();
        fs::write(
            dir.join("field_enemies.json"),
            serde_json::to_vec(&json!({"schema_version":1,"enemies":enemies})).unwrap(),
        )
        .unwrap();
        Self(dir)
    }
    fn load(&self, profile: i32, level: u32) -> EncounterEngine {
        let mut e = EncounterEngine::load(&self.0, None).unwrap();
        e.load_field_enemies(&self.0.join("field_enemies.json"))
            .unwrap();
        e.set_player_job(profile, level);
        e
    }
}
fn skill(effects: Value, damage: f32, heal: u32) -> JobAbility {
    serde_json::from_value(json!({"slot":1,"definition_id":62000,"name_id":1,"icon_id":1,"min_level":1,"cooldown_ms":5000,"damage_multiplier":damage,"heal":heal,"effects":effects})).unwrap()
}
fn hit_damages(out: &osfr_tools::encounter::EncounterOutcome) -> Vec<u32> {
    out.replies
        .iter()
        .filter(|p| p.starts_with(&[32, 0, 7, 0]))
        .map(|p| u32::from_le_bytes(p[28..32].try_into().unwrap()))
        .collect()
}
#[test]
fn self_heal_clamps_and_scales_with_the_existing_hp_curve() {
    let f = Fixture::new(10000, 100);
    let mut e = f.load(11, 5);
    e.tick([0.0; 3], 11, 0).unwrap();
    let before = e.player_health();
    assert!(before.0 < before.1);
    let a = skill(json!([]), 0.0, 250);
    e.job_ability(&a, 1, [0.0; 3], 11, 600).unwrap();
    assert_eq!(e.player_health().0, e.player_health().1);
    let again = e.job_ability(&a, 1, [0.0; 3], 11, 700).unwrap();
    assert!(again.replies.contains(&vec![36, 0, 1, 0, 31, 1, 0, 0])); // 287 cooldown
}
#[test]
fn area_damage_hits_each_in_radius_once_and_honours_target_limit() {
    let f = Fixture::new(10000, 1);
    let mut e = f.load(32, 5);
    let a = skill(
        json!([{"kind":"area_damage","radius":6,"max_targets":2,"centered_on_caster":true}]),
        1.0,
        0,
    );
    let out = e.job_ability(&a, 100, [0.0; 3], 32, 0).unwrap();
    assert_eq!(hit_damages(&out).len(), 2);
    assert!(hit_damages(&out).iter().all(|x| *x >= 337));
}
#[test]
fn dot_ticks_on_deadlines_and_does_not_consume_basic_swing_gate() {
    let f = Fixture::new(10000, 1);
    let mut e = f.load(12, 1);
    let a = skill(
        json!([{"kind":"damage_over_time","multiplier":0.5,"duration_ms":2000,"interval_ms":1000}]),
        1.0,
        0,
    );
    e.job_ability(&a, 100, [0.0; 3], 12, 0).unwrap();
    assert!(
        hit_damages(&e.tick([0.0; 3], 12, 999).unwrap())
            .iter()
            .all(|x| *x == 1)
    );
    let dot = e.tick([0.0; 3], 12, 1100).unwrap();
    assert!(hit_damages(&dot).iter().any(|x| *x >= 65));
    assert!(
        !e.attack(100, [0.0; 3], 12, 0, 1100)
            .unwrap()
            .replies
            .is_empty()
    );
    let late = e.tick([0.0; 3], 12, 10000).unwrap();
    assert_eq!(hit_damages(&late).iter().filter(|x| **x >= 65).count(), 1);
    assert!(
        hit_damages(&e.tick([0.0; 3], 12, 11000).unwrap())
            .iter()
            .all(|x| *x == 1)
    );
}
#[test]
fn stun_stops_attacks_and_movement_until_exact_expiry() {
    let f = Fixture::new(10000, 10);
    let mut e = f.load(32, 1);
    let a = skill(
        json!([{"kind":"area_damage","radius":6,"max_targets":16,"centered_on_caster":true},{"kind":"stun","duration_ms":2000}]),
        0.0,
        0,
    );
    e.job_ability(&a, 100, [0.0; 3], 32, 0).unwrap();
    let before = e.player_health();
    assert!(hit_damages(&e.tick([0.0; 3], 32, 1900).unwrap()).is_empty());
    assert_eq!(e.player_health(), before);
    assert!(!hit_damages(&e.tick([0.0; 3], 32, 2000).unwrap()).is_empty());
}

#[test]
fn target_vitals_debuff_only_increases_damage_on_its_target_and_expires() {
    let f = Fixture::new(10000, 0);
    let mut e = f.load(11, 5);
    let a = skill(
        json!([{"kind":"buff","recipient":"target","duration_ms":2000,"damage_multiplier":1.0,"damage_taken_multiplier":2.0,"crit_bonus":0,"crit_damage_bonus":0.0}]),
        0.0,
        0,
    );
    e.job_ability(&a, 100, [0.0; 3], 11, 0).unwrap();
    let marked = hit_damages(&e.attack(100, [0.0; 3], 11, 0, 600).unwrap())[0];
    let other = hit_damages(&e.attack(101, [0.0; 3], 11, 0, 1200).unwrap())[0];
    assert!(marked >= 458 && other <= 344);
    let expired = hit_damages(&e.attack(100, [0.0; 3], 11, 0, 2000).unwrap())[0];
    assert!(expired <= 344);
}

#[test]
fn critical_buff_affects_chance_and_damage_and_clears_on_job_change() {
    let f = Fixture::new(10000, 0);
    let mut normal = f.load(43, 1);
    let mut boosted = normal.clone();
    let a = skill(
        json!([{"kind":"buff","recipient":"caster","duration_ms":30000,"damage_multiplier":1.0,"damage_taken_multiplier":1.0,"crit_bonus":50,"crit_damage_bonus":1.0}]),
        0.0,
        0,
    );
    boosted.job_ability(&a, 1, [0.0; 3], 43, 0).unwrap();
    let mut normal_damage = 0;
    let mut boosted_damage = 0;
    for t in (0..14000).step_by(700) {
        normal_damage += hit_damages(&normal.attack(100, [0.0; 3], 43, 0, t).unwrap())[0];
        boosted_damage += hit_damages(&boosted.attack(100, [0.0; 3], 43, 0, t).unwrap())[0];
    }
    assert!(boosted_damage > normal_damage + 500);
    boosted.set_player_job(32, 1);
    let hit = hit_damages(&boosted.attack(100, [0.0; 3], 32, 0, 15000).unwrap())[0];
    assert!(hit <= 165);
}

#[test]
fn pure_dot_arms_an_arena_and_stops_at_victory_without_repeating_rewards() {
    use osfr_tools::{character_store::CharacterState, encounter::EncounterPhase};
    let f = Fixture::new(10000, 0);
    let mut e = f.load(12, 1);
    e.begin(
        411,
        CharacterState {
            guid: 1,
            zone_id: 0,
            position: [0.0; 3],
            rotation: [0.0; 3],
        },
        12,
    )
    .unwrap();
    e.on_zone_ready(12, 0).unwrap();
    let a = skill(
        json!([{"kind":"damage_over_time","multiplier":1.0,"duration_ms":2000,"interval_ms":1000}]),
        0.0,
        0,
    );
    e.job_ability(&a, 200, [0.0; 3], 12, 0).unwrap();
    let win = e.tick([0.0; 3], 12, 1000).unwrap();
    assert_eq!(
        e.state().active.as_ref().unwrap().phase,
        EncounterPhase::Won
    );
    assert_eq!(win.defeated, [401]);
    assert_eq!(win.rewards.len(), 1);
    assert!(e.tick([0.0; 3], 12, 2000).unwrap().rewards.is_empty());
}
#[test]
fn buff_changes_damage_and_mitigation_then_expires_without_stacking() {
    let f = Fixture::new(10000, 100);
    let mut e = f.load(43, 1);
    let a = skill(
        json!([{"kind":"buff","recipient":"caster","duration_ms":2000,"damage_multiplier":2.0,"damage_taken_multiplier":0.5,"crit_bonus":0,"crit_damage_bonus":0.0}]),
        0.0,
        0,
    );
    e.job_ability(&a, 1, [0.0; 3], 43, 0).unwrap();
    let hit = e.attack(100, [0.0; 3], 43, 0, 0).unwrap();
    assert!(hit_damages(&hit)[0] >= 200);
    let before = e.player_health().0;
    let incoming = e.tick([0.0; 3], 43, 100).unwrap();
    let taken = hit_damages(&incoming).iter().sum::<u32>();
    assert_eq!(before - e.player_health().0, taken);
    assert!(taken <= 150);
    let hit = e.attack(100, [0.0; 3], 43, 0, 2100).unwrap();
    assert!(hit_damages(&hit)[0] <= 150);
}
#[test]
fn invalid_range_cooldown_and_unavailable_summon_do_not_spend_or_damage() {
    let f = Fixture::new(10000, 1);
    let mut e = f.load(2, 10);
    let mut a = skill(json!([]), 1.0, 0);
    let out = e.job_ability(&a, 100, [1000.0; 3], 2, 0).unwrap();
    assert!(out.replies[0].starts_with(&[36, 0, 1, 0]));
    assert!(!hit_damages(&e.job_ability(&a, 100, [0.0; 3], 2, 0).unwrap()).is_empty());
    assert!(hit_damages(&e.job_ability(&a, 100, [0.0; 3], 2, 600).unwrap()).is_empty());
    a.unavailable = true;
    assert!(hit_damages(&e.job_ability(&a, 100, [0.0; 3], 2, 10000).unwrap()).is_empty());
}

fn job_wins(profile: i32) {
    let content: Value =
        serde_json::from_str(include_str!("../data/content/job_abilities.json")).unwrap();
    let row = content["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["profile_id"] == profile)
        .unwrap();
    let skills: Vec<JobAbility> = serde_json::from_value(row["slots"].clone()).unwrap();
    // (R) Same-level field packs use the existing HP curve, three enemies,
    // no god mode, no basic attack; repeat at every ladder level.
    for level in [1_u32, 5, 10, 15, 20] {
        let scale = 1.75_f32.powf((level - 1) as f32 / 4.0);
        let f = Fixture::new((240.0 * scale) as u32, (12.0 * scale) as u32);
        let mut e = f.load(profile, level);
        let mut defeated = 0;
        let mut casts = 0;
        for t in (0..120000).step_by(700) {
            for a in skills.iter().filter(|a| a.min_level <= level) {
                let out = e.job_ability(a, 0, [0.0; 3], profile, t).unwrap();
                defeated += out.defeated.len();
                casts += usize::from(!hit_damages(&out).is_empty());
            }
            if defeated >= 3 {
                break;
            }
            let out = e.tick([0.0; 3], profile, t).unwrap();
            defeated += out.defeated.len();
            assert!(e.player_health().0 > 0, "profile {profile}, level {level}");
            assert!(
                !out.feedback
                    .as_deref()
                    .unwrap_or("")
                    .contains("Knocked out"),
                "profile {profile}, level {level}"
            );
        }
        assert_eq!(
            defeated, 3,
            "profile {profile}, level {level}, casts {casts}"
        );
        assert!(casts > 0);
    }
}
#[test]
fn medic_wins_using_skills() {
    job_wins(11)
}
#[test]
fn warrior_wins_using_skills() {
    job_wins(32)
}
#[test]
fn wizard_wins_using_skills() {
    job_wins(12)
}
#[test]
fn ninja_wins_using_skills() {
    job_wins(2)
}
#[test]
fn brawler_wins_using_skills() {
    job_wins(43)
}
#[test]
fn archer_wins_using_skills() {
    job_wins(35)
}
