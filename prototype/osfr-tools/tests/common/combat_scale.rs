//! The live-scale numbers (`data/content/combat_scale.json`) for tests that
//! assert on them; read from the data so tuning the file moves the tests too.
#![allow(dead_code)]
use serde_json::Value;

fn data() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/content/combat_scale.json"
    );
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

/// Level-1 maximum health before the job's health multiplier.
pub fn base_health() -> u32 {
    data()["player_base_health"].as_u64().unwrap() as u32
}

/// A profile's level-1 basic attack (key 0 answers unlisted profiles).
pub fn basic(profile: i32) -> u32 {
    let table = &data()["basic_damage"];
    table[profile.to_string()]
        .as_u64()
        .or(table["0"].as_u64())
        .unwrap() as u32
}

/// An authored (old-scale) enemy health as the loader scales it at level one.
pub fn enemy_health(authored: u32) -> u32 {
    (authored as f64 * data()["enemy_health_scale"].as_f64().unwrap()).round() as u32
}

/// An authored (old-scale) enemy damage as the loader scales it at level one.
pub fn enemy_damage(authored: u32) -> u32 {
    (authored as f64 * data()["enemy_damage_scale"].as_f64().unwrap()).round() as u32
}

/// Damage and heal multiplier of the weapon tier a job level wields.
pub fn tier_factor(level: u32) -> f64 {
    let d = data();
    let tiers = d["tier_min_levels"].as_array().unwrap();
    let tier = tiers
        .iter()
        .filter(|l| l.as_u64().unwrap() <= level as u64)
        .count()
        .max(1);
    d["tier_ratio"].as_f64().unwrap().powi(tier as i32 - 1)
}

/// A profile's basic attack at a job level: the basic times its tier factor.
pub fn strike(profile: i32, level: u32) -> u32 {
    (basic(profile) as f64 * tier_factor(level)).round() as u32
}

/// Growth per weapon tier, and per four levels of health.
pub fn tier_ratio() -> f64 {
    data()["tier_ratio"].as_f64().unwrap()
}

/// A field enemy's authored damage at its level: the tier factor, then the enemy scale.
pub fn enemy_hit(authored: u32, level: u32) -> u32 {
    let scale = data()["enemy_damage_scale"].as_f64().unwrap();
    (authored as f64 * tier_factor(level) * scale).round() as u32
}

/// An authored enemy health at a level or difficulty level: the health growth
/// first, then the enemy scale, rounded once like the loader.
pub fn enemy_health_at(authored: u32, level: u32) -> u32 {
    let d = data();
    let growth = d["tier_ratio"]
        .as_f64()
        .unwrap()
        .powf((level - 1) as f64 / 4.0);
    (authored as f64 * growth * d["enemy_health_scale"].as_f64().unwrap()).round() as u32
}

/// The first authored enemy health whose loader-scaled value reaches the middle of the
/// `swings`-th swing of `strike` damage: a fixture that dies on exactly that swing, so tuning
/// the scale keeps its hit count.
pub fn authored_for(strike: u32, swings: u32) -> u32 {
    let target = (swings - 1) * strike + strike / 2;
    (1..).find(|&h| enemy_health(h) >= target).unwrap()
}
