use osfr_tools::{
    encounter::{EncounterEngine, EncounterOutcome},
    item_bar::Strike,
    opening_abilities::JobAbility,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "common/job_playtest.rs"]
mod job_playtest;

static SERIAL: AtomicU64 = AtomicU64::new(0);
const GUID: u64 = 100;
/// The fixture wolf's authored 100 health on the live scale.
fn full() -> u32 {
    combat_scale::enemy_health(100)
}
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
    assert!(hp(&e.attack(GUID, [0.0; 3], 11, 0, 0).unwrap()).unwrap().0 < full());
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
        if hp(&out) == Some((full(), full())) {
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
        healed |= hp(&out) == Some((full(), full()));
        if healed {
            break;
        }
    }
    assert!(healed);
}

/// L084 (Sanctuary port P3, loot tables): a field kill reward row can roll a loot table. The roll's items
/// ride the reward as bonus items (granted only with bag room), its coins and XP add to the row's pair, and a
/// row naming a table that does not exist refuses to load.
#[test]
fn a_kill_reward_row_with_loot_pays_the_rolled_items() {
    let f = Fixture::new(1, 0.0);
    let source = json!([{"kind": "reconstructed", "evidence": "(R) test table"}]);
    let tables = json!({"schema_version": 1, "tables": [{"key": "wolf-drops", "source": source, "entries": [
        {"kind": "item", "weight": 1, "definition_id": 2001, "quantity": 2}
    ]}]});
    fs::write(f.0.join("loot_tables.json"), tables.to_string()).unwrap();
    let rewards = |table: &str| {
        json!({"schema_version": 1, "level_rewards": [{"min_level": 1, "max_level": 1, "coins": 3, "job_xp": 4,
            "loot": {"table": table, "percent": 100}, "source": source}]})
        .to_string()
    };
    fs::write(f.0.join("field_kill_rewards.json"), rewards("wolf-drops")).unwrap();
    let mut e = f.load();
    let mut paid = Vec::new();
    for t in 0..300 {
        paid.extend(e.attack(GUID, at(&e), 11, 0, t * 700).unwrap().rewards);
        if !paid.is_empty() {
            break;
        }
    }
    assert_eq!(paid.len(), 1, "one kill, one payout");
    assert_eq!((paid[0].coins, paid[0].job_xp), (3, 4));
    assert_eq!(paid[0].bonus_items, vec![(2001, 2)]);

    fs::write(f.0.join("field_kill_rewards.json"), rewards("missing")).unwrap();
    assert!(
        EncounterEngine::load(&f.0, None).is_err(),
        "an unknown loot table refuses to load"
    );
}

/// L085 (Sanctuary port P4a): a potion's heal over time comes a step a second, in equal steps, and stops
/// once spent.
#[test]
fn a_heal_over_time_comes_a_step_a_second_and_stops() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let max = e.state().player_health;
    let mut now = 0;
    // The wolf takes some health first; then the player walks off past its leash.
    while e.state().player_health > max * 6 / 10 {
        now += 100;
        e.tick([0.0; 3], 11, now).unwrap();
        assert!(now < 120_000, "the wolf never hit");
    }
    for _ in 0..40 {
        now += 100;
        e.tick([100.0, 0.0, 0.0], 11, now).unwrap();
    }
    let start = e.state().player_health;
    e.heal_player_over_time(30, 3_000, now);
    let mut steps = Vec::new();
    for second in 1..=4 {
        e.tick([100.0, 0.0, 0.0], 11, now + second * 1_000).unwrap();
        steps.push(e.state().player_health);
    }
    let step = max * 10 / 100;
    assert_eq!(steps[0], start + step, "the first step a second in");
    assert_eq!(steps[2], start + 3 * step, "three steps of 10%");
    assert_eq!(steps[3], steps[2], "nothing after the heal is spent");
}

/// L088: battle items (wiki numbers). A Frag Orb hits the clicked wolf for its 254 (or a critical);
/// a clicked wolf out of reach is no target; a Flabbergast Orb stuns it for 5 s, so it cannot hit back.
#[test]
fn a_battle_item_strikes_or_stuns_the_wolf_in_reach() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let near = [5.0, 0.0, 0.0];
    assert_eq!(e.battle_item_target([60.0, 0.0, 0.0], Some(GUID)), None);
    assert_eq!(
        e.battle_item_target(near, None),
        Some(GUID),
        "no click: the nearest"
    );
    let frag = Strike {
        damage: 254,
        ..Strike::default()
    };
    let thrown = e.use_battle_item(frag, Some(GUID), near, 0).unwrap();
    let (health, _) = hp(&thrown).expect("the wolf's health after the orb");
    assert!(full() - health >= 254, "a 254 hit, or its critical");
    let stunned = e
        .use_battle_item(
            Strike {
                stun_ms: 5_000,
                ..Strike::default()
            },
            Some(GUID),
            near,
            100,
        )
        .unwrap();
    assert!(hp(&stunned).is_none(), "a stun deals no damage");
    for t in 2..50 {
        let out = e.tick(near, 12, t * 100).unwrap();
        assert!(
            hits_from(&out, GUID).is_empty(),
            "stunned at {} ms",
            t * 100
        );
    }
    let mut hit_back = false;
    for t in 52..=120 {
        hit_back |= !hits_from(&e.tick(near, 12, t * 100).unwrap(), GUID).is_empty();
    }
    assert!(hit_back, "it fights again once the stun ends");
}

/// L089: a Sleep Orb holds the wolf at home until a hit wakes it; an Unmoving Orb roots it until a hit
/// frees it (wiki: "Hitting your target will wake them" / "will allow them to move").
#[test]
fn a_sleeping_or_rooted_wolf_moves_again_once_hit() {
    let f = Fixture::new(1, 0.0);
    // Inside the wolf's 8 m aggro, out of its reach: an awake wolf walks over.
    let player = [6.0, 0.0, 0.0];
    for hold in [
        Strike {
            sleep_ms: 10_000,
            ..Strike::default()
        },
        Strike {
            root_ms: 10_000,
            ..Strike::default()
        },
    ] {
        let mut e = f.load();
        e.use_battle_item(hold, Some(GUID), player, 0).unwrap();
        for t in 1..30 {
            e.tick(player, 12, t * 100).unwrap();
        }
        assert!(radius(at(&e)) < 0.5, "held at home: {hold:?}");
        assert!(hp(&e.attack(GUID, player, 12, 0, 3_000).unwrap()).is_some());
        for t in 31..60 {
            e.tick(player, 12, t * 100).unwrap();
        }
        assert!(radius(at(&e)) > 1.0, "the hit lets it walk: {hold:?}");
    }
}

/// L101 (review 522c6f8-L089): a root that replaces a sleep takes the sleep with it, so one hit
/// frees the wolf; and waking a sleeper leaves a separate 5 s stun running to its own end.
#[test]
fn a_hold_and_a_stun_keep_their_own_timers() {
    let f = Fixture::new(1, 0.0);
    let player = [6.0, 0.0, 0.0];
    let sleep = Strike {
        sleep_ms: 10_000,
        ..Strike::default()
    };
    let root = Strike {
        root_ms: 10_000,
        ..Strike::default()
    };
    let stun = Strike {
        stun_ms: 5_000,
        ..Strike::default()
    };

    let mut e = f.load();
    e.use_battle_item(sleep, Some(GUID), player, 0).unwrap();
    e.use_battle_item(root, Some(GUID), player, 0).unwrap();
    assert!(hp(&e.attack(GUID, player, 12, 0, 1_000).unwrap()).is_some());
    for t in 11..40 {
        e.tick(player, 12, t * 100).unwrap();
    }
    assert!(
        radius(at(&e)) > 1.0,
        "the root's hit frees it, no sleep stun left"
    );

    let mut e = f.load();
    e.use_battle_item(stun, Some(GUID), player, 0).unwrap();
    e.use_battle_item(sleep, Some(GUID), player, 0).unwrap();
    assert!(hp(&e.attack(GUID, player, 12, 0, 1_000).unwrap()).is_some());
    for t in 11..48 {
        e.tick(player, 12, t * 100).unwrap();
    }
    assert!(radius(at(&e)) < 0.5, "the stun still holds it until 5 s");
    for t in 52..80 {
        e.tick(player, 12, t * 100).unwrap();
    }
    assert!(radius(at(&e)) > 1.0, "free once the stun ends");
}

/// L091: a Blast Orb knocks the wolf 6 yd (5.5 m) straight away from the thrower and it stays down
/// for 2 s before it comes back (wiki: "Knocks your target back 6 yards").
#[test]
fn a_blast_orb_knocks_the_wolf_away_from_the_thrower() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let player = [3.0, 0.0, 0.0];
    let blast = Strike {
        knockback: 5.5,
        ..Strike::default()
    };
    let out = e.use_battle_item(blast, Some(GUID), player, 0).unwrap();
    let [x, _, z] = at(&e);
    assert!(
        (x + 5.5).abs() < 0.01 && z.abs() < 0.01,
        "pushed away from the thrower: {x} {z}"
    );
    assert!(
        out.replies
            .iter()
            .any(|p| p.starts_with(&125_u16.to_le_bytes())),
        "a movement update"
    );
    for t in 1..19 {
        let out = e.tick(player, 12, t * 100).unwrap();
        assert!(hits_from(&out, GUID).is_empty(), "down at {} ms", t * 100);
    }
}

/// L092: a scare sends the wolf running straight away from the player for 4 s, striking nobody;
/// a hit does not end it (wiki: "causing opponents to panic and flee in terror").
#[test]
fn a_scared_wolf_runs_from_the_player_and_strikes_nobody() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    let player = [2.0, 0.0, 0.0];
    let out = e.scare(10.0, 4_000, player, 0);
    assert!(!out.replies.is_empty(), "the wolf in range is scared");
    for t in 1..38 {
        let out = e.tick(player, 12, t * 100).unwrap();
        assert!(hits_from(&out, GUID).is_empty(), "scared at {} ms", t * 100);
    }
    let [x, _, _] = at(&e);
    assert!(x < -1.0, "it ran away from the player: x {x}");
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
    assert_eq!(
        hp(&out),
        Some((0, full())),
        "the debuff lands the killing hit"
    );
    let out = e.tick([100.0, 0.0, 0.0], 11, 3100).unwrap();
    assert_eq!(hp(&out), Some((full(), full())), "the kill respawns");
    let out = e.tick([100.0, 0.0, 0.0], 11, 4100).unwrap();
    assert!(
        hits_from(&out, 1).is_empty(),
        "damage over time was cleared"
    );
    let out = e.tick([0.0; 3], 11, 4200).unwrap();
    assert!(!hits_from(&out, GUID).is_empty(), "the stun was cleared");
    let out = e.attack(GUID, [0.0; 3], 11, 0, 4300).unwrap();
    assert_eq!(
        hp(&out),
        Some((full() - combat_scale::basic(11), full())),
        "the target buff was cleared"
    );
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
    let big = combat_scale::enemy_health(1000);
    assert_eq!(
        hp(&out).unwrap().0,
        big - 2 * combat_scale::basic(11),
        "the debuff doubles the hit"
    );
    let out = e.tick([100.0, 0.0, 0.0], 11, 2200).unwrap();
    assert!(hits_from(&out, 1).is_empty(), "the dot was not due yet");
    assert_eq!(hp(&out), Some((big, big)), "the evading actor heals home");
    let out = e.tick([100.0, 0.0, 0.0], 11, 3500).unwrap();
    assert!(
        hits_from(&out, 1).is_empty(),
        "damage over time was cleared"
    );
    let out = e.tick([0.0; 3], 11, 4200).unwrap();
    assert!(!hits_from(&out, GUID).is_empty(), "the stun was cleared");
    let out = e.attack(GUID, [0.0; 3], 11, 0, 4300).unwrap();
    assert_eq!(
        hp(&out),
        Some((big - combat_scale::basic(11), big)),
        "the target buff was cleared"
    );
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
    assert_eq!(
        hp(&out),
        Some((full() - combat_scale::basic(11), full())),
        "the target buff was cleared"
    );
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
    assert_eq!(
        hp(&one.on_zone_ready(11, 0).unwrap()),
        Some((full(), full()))
    );
    // Level 5 is one four-level health step.
    let health = (full() as f64 * combat_scale::tier_ratio()).round() as u32;
    assert_eq!(
        hp(&high.on_zone_ready(11, 0).unwrap()),
        Some((health, health))
    );
    let before = high.state().player_health;
    high.tick([0.0; 3], 11, 100).unwrap();
    assert_eq!(
        before - high.state().player_health,
        combat_scale::enemy_hit(7, 5)
    );
    for t in 0..3 {
        high.attack(GUID, [0.0; 3], 2, 0, 1000 + t * 700).unwrap();
    }
    assert!(!high.field_teleport_targets().iter().any(|a| a.guid == GUID));
    let out = high.tick([100.0; 3], 11, 4000).unwrap();
    assert_eq!(hp(&out), Some((health, health)));
    assert_eq!(at(&high), [0.0; 3]);
}

/// T398: a field enemy's hit is its row's `damage` through the data scale, not a hidden
/// constant: editing the row (or its level) moves the hit to the data-derived value.
#[test]
fn a_field_enemy_hit_follows_its_row_damage_and_level() {
    let hit = |level: u32, damage: u32| {
        let f = Fixture::new(level, 0.0);
        f.edit("damage", json!(damage));
        let mut e = f.load();
        e.on_zone_ready(11, 0).unwrap();
        let hits = hits_from(&e.tick([0.0; 3], 11, 100).unwrap(), GUID);
        assert_eq!(hits.len(), 1, "one swing");
        hits[0]
    };
    let mut seen = BTreeSet::new();
    for (level, damage) in [(1, 7), (1, 40), (5, 7), (5, 40), (8, 40)] {
        let got = hit(level, damage);
        assert_eq!(
            got,
            combat_scale::enemy_hit(damage, level),
            "{damage} at {level}"
        );
        seen.insert(got);
    }
    assert_eq!(
        seen.len(),
        5,
        "every row or level change moves the hit: {seen:?}"
    );
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
    assert_eq!(
        e.state().player_health,
        before - combat_scale::enemy_damage(7)
    );
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

/// L083 (Sanctuary port P2, threat): an enemy shot from beyond its aggro range used to ignore its attacker;
/// now the attacker is on its threat table and is fought anywhere within the leash, then forgotten once the
/// enemy evades home.
#[test]
fn a_hit_from_beyond_aggro_range_is_fought_within_the_leash() {
    let f = Fixture::new(1, 0.0);
    let mut e = f.load();
    // Profile 12's basic attack reaches 20 m; the fixture wolf aggroes at 8 m and leashes at 12 m.
    let archer = [10.0, 0.0, 0.0];
    assert!(hp(&e.attack(GUID, archer, 12, 0, 0).unwrap()).unwrap().0 < full());
    let mut hit_back = false;
    for t in 1..=60 {
        let out = e.tick(archer, 12, t * 100).unwrap();
        hit_back |= !hits_from(&out, GUID).is_empty();
    }
    assert!(hit_back, "the wolf closes in and fights its attacker");
    assert!(radius(at(&e)) > 5.0, "it walked out to the archer");

    // Out of the leash the attacker is forgotten: the wolf goes home, heals and stays idle.
    let mut healed = false;
    for t in 61..=200 {
        let out = e.tick([60.0, 0.0, 0.0], 12, t * 100).unwrap();
        healed |= hp(&out) == Some((full(), full()));
    }
    assert!(healed, "it evades home and heals");
    for t in 201..=230 {
        let out = e.tick(archer, 12, t * 100).unwrap();
        assert!(
            hits_from(&out, GUID).is_empty(),
            "threat was cleared on the way home"
        );
    }
}
