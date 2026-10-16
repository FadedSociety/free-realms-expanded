use osfr_tools::power_ups::{PowerUps, Rng, roll};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SHIPPED: &str = include_str!("../data/content/power_ups.json");

fn parse(value: &Value) -> std::io::Result<PowerUps> {
    PowerUps::from_json(value.to_string().as_bytes())
}

fn shipped() -> Value {
    serde_json::from_str(SHIPPED).unwrap()
}

#[test]
fn shipped_file_loads_and_references_resolve() {
    let p = PowerUps::from_json(SHIPPED.as_bytes()).unwrap();
    for name in ["standard", "elite", "boss"] {
        assert!(p.table(name).is_some(), "missing table {name}");
    }
    assert!(p.kind("flame_wave").is_some_and(|k| !k.auto));
    assert!(p.kind("energy").is_some_and(|k| k.auto));
}

/// One canary per bound: each edit must be rejected, and the untouched file is accepted.
#[test]
fn every_bound_rejects_a_bad_file() {
    let cases: &[(&str, Value)] = &[
        ("/schema_version", json!(2)),
        ("/pickup/ttl_ms", json!(4_999)),
        ("/pickup/ttl_ms", json!(120_001)),
        ("/pickup/radius", json!(0.4)),
        ("/pickup/radius", json!(5.1)),
        ("/pickup/max_alive_per_instance", json!(0)),
        ("/pickup/max_alive_per_instance", json!(33)),
        ("/pickup/source", json!("")),
        ("/kinds/health_small/model_id", json!(0)),
        ("/kinds/health_small/heal_pct_max", json!(0)),
        ("/kinds/health_small/heal_pct_max", json!(101)),
        ("/kinds/health_big/scale", json!(5.0)),
        ("/kinds/energy/energy", json!(0)),
        ("/kinds/energy/energy", json!(101)),
        ("/kinds/flame_wave/area/radius", json!(0.4)),
        ("/kinds/flame_wave/area/radius", json!(30.1)),
        ("/kinds/flame_wave/area/max_targets", json!(0)),
        ("/kinds/flame_wave/area/max_targets", json!(17)),
        ("/kinds/flame_wave/burn/multiplier", json!(0.0)),
        ("/kinds/flame_wave/burn/duration_ms", json!(499)),
        ("/kinds/flame_wave/burn/duration_ms", json!(30_001)),
        ("/kinds/flame_wave/burn/interval_ms", json!(499)),
        ("/kinds/flame_wave/burn/interval_ms", json!(4_001)),
        ("/kinds/flame_wave/damage_by_tier", json!([1.0, 2.0])),
        ("/kinds/flame_wave/damage_by_tier/0", json!(0.0)),
        ("/kinds/earth_shard/stun_ms", json!(499)),
        ("/kinds/earth_shard/stun_ms", json!(10_001)),
        ("/kinds/super_shield/duration_ms", json!(499)),
        ("/kinds/super_shield/duration_ms", json!(30_001)),
        ("/kinds/super_shield/speed_pct", json!(101)),
        ("/kinds/super_shield/source", json!("  ")),
        ("/tables/standard/health_roll/chance", json!(0.51)),
        ("/tables/standard/health_roll/chance", json!(-0.1)),
        (
            "/tables/standard/health_roll/pool/health_small",
            json!(1001),
        ),
        (
            "/tables/standard/health_roll/pool",
            json!({ "health_small": 85, "no_such_kind": 5 }),
        ),
        (
            "/tables/standard/health_roll/pool",
            json!({ "health_small": 0 }),
        ),
        ("/tables/standard/health_roll/pool", json!({})),
        ("/tables/boss/guaranteed", json!(["no_such_kind"])),
        ("/tables/boss/guaranteed", json!(vec!["health_big"; 9])),
        ("/tables/boss/source", json!("")),
    ];
    assert!(parse(&shipped()).is_ok());
    for (pointer, bad) in cases {
        let mut doc = shipped();
        *doc.pointer_mut(pointer)
            .unwrap_or_else(|| panic!("no {pointer}")) = bad.clone();
        assert!(parse(&doc).is_err(), "{pointer} = {bad} was accepted");
    }
}

#[test]
fn non_finite_and_unknown_fields_and_oversize_are_rejected() {
    // JSON cannot carry NaN, so a string in a number slot stands for a hostile value.
    let mut doc = shipped();
    doc["tables"]["standard"]["health_roll"]["chance"] = json!("NaN");
    assert!(parse(&doc).is_err());
    let mut doc = shipped();
    doc["pickup"]["surprise"] = json!(1);
    assert!(parse(&doc).is_err());
    assert!(PowerUps::from_json(&vec![b' '; 1_048_577]).is_err());
    assert!(PowerUps::from_json(b"not json").is_err());
}

#[test]
fn roll_is_deterministic_for_a_fixed_seed() {
    let p = PowerUps::from_json(SHIPPED.as_bytes()).unwrap();
    let table = p.table("elite").unwrap();
    let run = |seed| {
        let mut rng = Rng::seeded(seed);
        (0..64).map(|_| roll(table, &mut rng)).collect::<Vec<_>>()
    };
    assert_eq!(run(7), run(7));
    assert_ne!(run(7), run(8));
    // Pinned splitmix64 output: guards against an accidental generator change.
    assert_eq!(Rng::seeded(0).next_u64(), 0xe220_a839_7b1d_cdaf);
}

#[test]
fn guaranteed_drops_always_come_first_and_skip_the_rolls() {
    let p = PowerUps::from_json(SHIPPED.as_bytes()).unwrap();
    let boss = p.table("boss").unwrap();
    let mut rng = Rng::seeded(1);
    for _ in 0..100 {
        assert_eq!(roll(boss, &mut rng), ["health_big"]);
    }
}

/// Large sample: drop rate and pool shares land near the configured numbers.
#[test]
fn roll_distribution_matches_chances_and_weights() {
    let p = PowerUps::from_json(SHIPPED.as_bytes()).unwrap();
    let table = p.table("standard").unwrap();
    let (health, other) = (
        table.health_roll.as_ref().unwrap(),
        table.other_roll.as_ref().unwrap(),
    );
    let n = 400_000u32;
    let mut rng = Rng::seeded(0x5eed);
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    for _ in 0..n {
        for kind in roll(table, &mut rng) {
            *counts.entry(kind).or_default() += 1;
        }
    }
    for r in [health, other] {
        let total_weight: u32 = r.pool.values().sum();
        let drops: u32 = r
            .pool
            .keys()
            .map(|k| counts.get(k.as_str()).copied().unwrap_or(0))
            .sum();
        let rate = f64::from(drops) / f64::from(n);
        assert!(
            (rate - r.chance).abs() < 0.005,
            "drop rate {rate} vs {}",
            r.chance
        );
        for (kind, &w) in &r.pool {
            let share = f64::from(counts[kind.as_str()]) / f64::from(drops);
            let want = f64::from(w) / f64::from(total_weight);
            assert!((share - want).abs() < 0.02, "{kind}: {share} vs {want}");
        }
    }
}

#[test]
fn zero_chance_never_drops() {
    let mut doc = shipped();
    doc["tables"]["standard"]["health_roll"]["chance"] = json!(0.0);
    doc["tables"]["standard"]["other_roll"]["chance"] = json!(0.0);
    let p = parse(&doc).unwrap();
    let mut rng = Rng::seeded(3);
    assert!((0..10_000).all(|_| roll(p.table("standard").unwrap(), &mut rng).is_empty()));
}
