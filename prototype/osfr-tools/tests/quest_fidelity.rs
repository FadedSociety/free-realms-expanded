//! QF (quest fidelity, owner 2026-11-11, after Going Batty): every live quest agrees with the
//! wiki quest table on its repeat, its event, its item rewards and its giver, or names its
//! reason in `tests/fixtures/quest_fidelity_exceptions.json`. The facts are
//! `tests/fixtures/quest_wiki_facts.json`, written by `scripts/quest_fidelity.py` from the wiki
//! quest table and the fanbyte harvest; the script also brings the rows in line, and runs again
//! after any merge.

use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn osfr() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn json(path: PathBuf) -> Value {
    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every quest row of the shipped catalogue.
fn quest_rows() -> Vec<Value> {
    let mut paths: Vec<PathBuf> = fs::read_dir(osfr().join("data/content/quests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths.into_iter().flat_map(|path| json(path)["records"].as_array().unwrap().clone()).collect()
}

fn live(row: &Value) -> bool {
    row["enabled"].as_bool().unwrap_or(true) && row["objectives"].as_array().is_some_and(|goals| !goals.is_empty())
}

fn ids(value: &Value) -> BTreeSet<u64> {
    value.as_array().into_iter().flatten().filter_map(Value::as_u64).collect()
}

/// (check, detail) for each of the four checks this live row fails against its wiki rows; a
/// check passes when any of a title's rows agrees.
fn mismatches(row: &Value, facts: &[&Value]) -> Vec<(String, String)> {
    let paid: BTreeSet<u64> = (row["rewards"].as_array().into_iter().flatten())
        .flat_map(|reward| reward["items"].as_array().into_iter().flatten())
        .filter_map(|item| item["definition_id"].as_u64())
        .collect();
    let key = row["live_event"].as_str();
    let repeatable = row["repeatable"].as_bool().unwrap_or(false);
    let rewards_ok = |fact: &&Value| {
        if !fact["item_cells"].as_bool().unwrap_or(false) {
            return true;
        }
        let cells = fact["rewards"].as_array().unwrap();
        let allowed: BTreeSet<u64> = cells.iter().flat_map(|cell| ids(&cell["ids"])).collect();
        paid.is_subset(&allowed) && cells.iter().all(|cell| !ids(&cell["ids"]).is_disjoint(&paid))
    };
    let giver = row["giver_npc_id"].as_u64().unwrap();
    let checks = [
        ("repeatable", facts.iter().any(|f| f["repeatable"].as_bool() == Some(repeatable))),
        ("event", facts.iter().any(|f| f["event"].as_str() == key)),
        ("rewards", facts.iter().any(rewards_ok)),
        ("giver", facts.iter().any(|f| ids(&f["givers"]).is_empty() || ids(&f["givers"]).contains(&giver))),
    ];
    let first = facts[0];
    (checks.into_iter())
        .filter(|(_, ok)| !ok)
        .map(|(check, _)| {
            let detail = format!(
                "wiki {:?} repeatable {} event {} givers {}; row repeatable {repeatable} event {key:?} giver {giver} items {paid:?}",
                first["title"], first["repeatable"], first["event"], first["givers"]
            );
            (check.to_owned(), detail)
        })
        .collect()
}

/// The check that would have caught Going Batty: every live quest with a wiki row agrees with it
/// on repeatable, event, item rewards by name and giver, or names its reason in the exceptions
/// file; an exception that no longer differs fails too, so the list only shrinks by fixing.
#[test]
fn every_live_quest_agrees_with_the_wiki_or_names_its_reason() {
    let facts = json(osfr().join("tests/fixtures/quest_wiki_facts.json"));
    let rows = facts["rows"].as_array().unwrap();
    let mut by_title: BTreeMap<u64, Vec<&Value>> = BTreeMap::new();
    for fact in rows {
        for title in ids(&fact["title_ids"]) {
            by_title.entry(title).or_default().push(fact);
        }
    }
    let exceptions = json(osfr().join("tests/fixtures/quest_fidelity_exceptions.json"));
    let excepted: BTreeMap<(u64, String), String> = (exceptions["exceptions"].as_array().unwrap().iter())
        .map(|row| {
            let reason = row["reason"].as_str().unwrap().trim().to_owned();
            assert!(reason.len() > 20, "exception {row} needs a reason");
            ((row["quest_id"].as_u64().unwrap(), row["check"].as_str().unwrap().to_owned()), reason)
        })
        .collect();
    let calendar = json(osfr().join("data/content/event_calendar.json"));
    let keys: BTreeSet<&str> = (calendar["events"].as_array().unwrap().iter())
        .map(|event| event["key"].as_str().unwrap())
        .collect();
    let (mut checked, mut failures, mut differing) = (0, Vec::new(), BTreeSet::new());
    for row in quest_rows().iter().filter(|row| live(row)) {
        let quest = row["id"].as_u64().unwrap();
        if let Some(key) = row["live_event"].as_str() {
            assert!(keys.contains(key), "quest {quest}: live_event {key} is not an event_calendar.json key");
        }
        if row["repeatable"].as_bool() == Some(true) {
            assert!(row["repeat_hours"].as_u64().is_some_and(|hours| hours >= 1), "quest {quest}: repeat_hours");
        }
        let Some(facts) = row["title_string_id"].as_u64().and_then(|title| by_title.get(&title)) else {
            continue;
        };
        checked += 1;
        for (check, detail) in mismatches(row, facts) {
            differing.insert((quest, check.clone()));
            if !excepted.contains_key(&(quest, check.clone())) {
                failures.push(format!("{quest} {check}: {detail}"));
            }
        }
    }
    assert!(checked > 0, "no live quest matched a wiki row: the check would pass vacuously");
    assert!(failures.is_empty(), "quests that differ from the wiki with no exception row:\n{}", failures.join("\n"));
    let stale: Vec<_> = excepted.keys().filter(|key| !differing.contains(key)).collect();
    assert!(stale.is_empty(), "exceptions that no longer differ (delete them): {stale:?}");
    // The shipped calendar lists no quests: each event quest's row carries the key.
    for event in calendar["events"].as_array().unwrap() {
        assert!(event.get("quest_ids").is_none(), "{} lists quests; tag the rows", event["key"]);
    }
}
