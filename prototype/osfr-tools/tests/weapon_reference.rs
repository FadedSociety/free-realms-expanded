//! T635: the weapon and attack reference tables (wiki harvest checked against the client item catalogue)
//! load, every row names its source, and every job ability multiplier matches the live ratio.
use serde_json::{Value, json};
use std::{fs, path::Path};

fn load(relative: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn sources(row: &Value) -> Vec<&str> {
    row["source"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect()
}

#[test]
fn every_weapon_and_attack_row_has_a_source() {
    let weapons = load("data/reference/weapons.json");
    let attacks = load("data/reference/attacks.json");
    assert_eq!(weapons["schema_version"], 1);
    assert_eq!(attacks["schema_version"], 1);
    let rows = weapons["weapons"].as_array().unwrap();
    assert_eq!(rows.len(), 653, "one row per wiki weapon page");
    for row in rows {
        let name = row["name"].as_str().unwrap();
        assert!(
            row["url"].as_str().unwrap().starts_with("https://"),
            "{name}"
        );
        assert!(row["rev_id"].as_i64().unwrap() > 0, "{name}");
        // A row cites the client only when it matched client items, and then it lists them.
        let from_client = sources(row).contains(&"client");
        assert!(sources(row).contains(&"wiki"), "{name}");
        assert_eq!(
            from_client,
            !row["client_items"].as_array().unwrap().is_empty(),
            "{name}"
        );
        // The power flag is recomputable from the row itself.
        let items = row["client_items"].as_array().unwrap();
        let power = row["wiki"]["power"].as_i64();
        let hits = items
            .iter()
            .filter(|i| i["power_rating"].as_i64() == power)
            .count();
        let expect = match (items.len(), hits) {
            (0, _) => Value::Null,
            (n, h) if h == n => json!("all"),
            (_, 0) => json!("none"),
            _ => json!("some"),
        };
        assert_eq!(row["agree"]["power"], expect, "{name}");
    }
    let unmatched = weapons["unmatched"].as_array().unwrap();
    let matched = rows
        .iter()
        .filter(|r| sources(r).contains(&"client"))
        .count();
    assert_eq!(unmatched.len() + matched, rows.len());
    assert!(
        unmatched
            .iter()
            .all(|u| !u["reason"].as_str().unwrap().is_empty())
    );
    for attack in attacks["attacks"].as_array().unwrap() {
        let name = attack["name"].as_str().unwrap();
        assert!(sources(attack).contains(&"wiki"), "{name}");
        assert!(attack["weapon_count"].as_u64().unwrap() > 0, "{name}");
    }
    assert_eq!(attacks["job_basic"].as_object().unwrap().len(), 6);
}

/// Job ability slots whose multiplier is neither within 2% of the live ratio (wiki attack damage as a
/// multiple of the tier base, over the job's basic attack) nor explained by a source naming the wiki ratio.
fn unexplained(abilities: &Value, attacks: &Value) -> Vec<String> {
    let basic = |profile: i64| {
        attacks["job_basic"]
            .as_object()
            .unwrap()
            .values()
            .find(|b| b["profile_id"] == profile)
            .map(|b| b["damage_x"].as_f64().unwrap())
            .unwrap()
    };
    let mut bad = Vec::new();
    for job in abilities["jobs"].as_array().unwrap() {
        for slot in job["slots"].as_array().unwrap() {
            let id = slot["definition_id"].as_i64().unwrap();
            let multiplier = slot["damage_multiplier"].as_f64().unwrap();
            let name_id = slot["name_id"].as_i64().unwrap();
            let attack = attacks["attacks"].as_array().unwrap().iter().find(|a| {
                a["name_locale_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|i| *i == name_id)
            });
            let source = slot["source"].to_string();
            let ok = match attack.and_then(|a| a["damage_x"]["median"].as_f64()) {
                Some(x) => {
                    let ratio = x / basic(job["profile_id"].as_i64().unwrap());
                    (multiplier / ratio - 1.0).abs() <= 0.02 || source.contains("wiki ratio")
                }
                // The wiki attack has no damage (buff) or no wiki attack carries this name:
                // a zero multiplier agrees, anything else needs its explanation or an (R) tag.
                None if attack.is_some() => multiplier == 0.0 || source.contains("wiki ratio"),
                None => source.contains("(R)"),
            };
            if !ok {
                bad.push(format!("definition {id}: multiplier {multiplier}"));
            }
        }
    }
    bad
}

#[test]
fn job_ability_multipliers_match_the_live_ratios() {
    let attacks = load("data/reference/attacks.json");
    let abilities = load("data/content/job_abilities.json");
    assert_eq!(unexplained(&abilities, &attacks), Vec::<String>::new());
    // Canary: a multiplier that disagrees with its attack and carries no wiki explanation is caught.
    let mut broken = abilities.clone();
    let zap = broken["jobs"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .flat_map(|j| j["slots"].as_array_mut().unwrap())
        .find(|s| s["definition_id"] == 62008)
        .unwrap();
    zap["damage_multiplier"] = json!(1.8);
    zap["source"] = json!({"damage_multiplier": "(R) guess"});
    assert_eq!(
        unexplained(&broken, &attacks),
        ["definition 62008: multiplier 1.8"]
    );
}
