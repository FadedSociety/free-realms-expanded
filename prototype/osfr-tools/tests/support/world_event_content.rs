//! The shared content gate for `data/content/world_events.json`.
//!
//! `bruce_concert.rs` (T493) and `halloween_dance_off.rs` (T464) both gate a shipped world
//! event against the client's tables. One collector and one gate keep a fix to the locale,
//! area XML path, animation tables or Models check from landing in only one suite, and stop a
//! wrong id passing a second copy of the gate. The rows are read as raw JSON, never through
//! `world_events::load`, so the gate cannot share a mistake with the loader.
//!
//! Test-only: production never reads the client tables.

#[path = "models.rs"]
mod models;
pub use models::read_models;
#[path = "xml.rs"]
mod xml;

use osfr_tools::world_events;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn content() -> PathBuf {
    crate_root().join("data/content")
}

pub fn lab(path: &str) -> PathBuf {
    crate_root().join("../../../lab").join(path)
}

pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

pub fn shipped() -> Value {
    serde_json::from_str(&read(&content().join(world_events::FILE))).unwrap()
}

/// The shipped row of `key`.
pub fn row<'a>(file: &'a Value, key: &str) -> &'a Value {
    file["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["key"] == key)
        .unwrap_or_else(|| panic!("{key} is not shipped"))
}

pub fn int(value: &Value) -> i32 {
    i32::try_from(value.as_i64().unwrap()).unwrap()
}

pub fn resources(file: &str) -> PathBuf {
    lab("working/OSFR.Client/OSFR Client/Resources").join(file)
}

pub fn locale() -> BTreeMap<i64, String> {
    let rows: Value = serde_json::from_str(&read(&lab("reports/locale-id-to-text.json"))).unwrap();
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap(),
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// Every step's op of the timeline, the `one_of` branches' included: `(op name, its fields)`.
pub fn ops(timeline: &Value) -> Vec<(String, Value)> {
    let mut found = Vec::new();
    for step in timeline.as_array().unwrap() {
        let (name, fields) = step["op"].as_object().unwrap().iter().next().unwrap();
        match name.as_str() {
            "one_of" => {
                for branch in fields["branches"].as_array().unwrap() {
                    found.extend(ops(branch));
                }
            }
            _ => found.push((name.clone(), fields.clone())),
        }
    }
    found
}

/// Every id the row names, gathered from the raw file so the check cannot share a mistake
/// with the loader.
#[derive(Debug, Default)]
pub struct Ids {
    pub models: BTreeSet<i32>,
    pub strings: BTreeSet<i32>,
    /// `(locale id, line)` for each broadcast that names a client line.
    pub broadcasts: Vec<(i32, String)>,
    pub areas: BTreeSet<i32>,
    pub animations: BTreeSet<i32>,
    pub effects: BTreeSet<i32>,
}

fn collect(event: &Value) -> Ids {
    let mut ids = Ids::default();
    for actor in event["actors"].as_array().unwrap() {
        ids.models.insert(int(&actor["model_id"]));
        let name = int(&actor["name_string_id"]);
        if name != 0 {
            ids.strings.insert(name);
        }
        ids.animations.insert(int(&actor["stand_animation_id"]));
    }
    let announce = &event["schedule"]["announce"];
    if !announce.is_null() {
        ids.strings.insert(int(&announce["locale_id"]));
    }
    // T538: the participation block's prompt, line and costume name ids (client `Global.Text`
    // rows). The item name ids are locale rows too; the shop/definition join is checked by the
    // item-catalogue gates, not here.
    let dance_off = &event["dance_off"];
    if !dance_off.is_null() {
        for key in [
            "prompt_string_id",
            "already_voted_string_id",
            "not_participant_string_id",
            "used_cake_string_id",
        ] {
            ids.strings.insert(int(&dance_off[key]));
        }
        for team in dance_off["teams"].as_array().unwrap() {
            ids.strings.insert(int(&team["vote_label_string_id"]));
            ids.strings.insert(int(&team["cake_say_string_id"]));
            ids.strings
                .extend(team["costume_name_ids"].as_array().unwrap().iter().map(int));
        }
    }
    for (name, op) in ops(&event["timeline"]) {
        match name.as_str() {
            "say" => drop(ids.strings.insert(int(&op["string_id"]))),
            "broadcast" => {
                if let Some(id) = op["locale_id"].as_i64() {
                    let id = i32::try_from(id).unwrap();
                    ids.strings.insert(id);
                    ids.broadcasts
                        .push((id, op["text"].as_str().unwrap().to_owned()));
                }
            }
            "area" => drop(ids.areas.insert(int(&op["area_id"]))),
            "base_anim" => drop(ids.animations.insert(int(&op["id"]))),
            "cycle_anim" => ids
                .animations
                .extend(op["ids"].as_array().unwrap().iter().map(int)),
            "tag_on" | "tag_off" => drop(ids.effects.insert(int(&op["effect_id"]))),
            "spawn" | "remove" if !op["effect_id"].is_null() => {
                ids.effects.insert(int(&op["effect_id"]));
            }
            "spawn" | "remove" | "move" | "face" => {}
            other => panic!("unexpected op {other}"),
        }
    }
    ids
}

/// Gate every id `event` names against the client's tables, returning the ids for the
/// caller's exact-set assertions. Panics on the first id that does not resolve.
pub fn assert_every_id_resolves(event: &Value) -> Ids {
    let ids = collect(event);
    let locale = locale();
    for id in &ids.strings {
        assert!(
            locale.contains_key(&i64::from(*id)),
            "string {id} is not in the locale"
        );
    }
    for actor in event["actors"].as_array().unwrap() {
        assert!(
            !actor["source"].as_str().unwrap_or_default().is_empty(),
            "actor {} has no source",
            actor["key"]
        );
        let name = int(&actor["name_string_id"]);
        if name != 0 {
            assert_eq!(
                actor["npc_name"],
                locale[&i64::from(name)],
                "actor {}",
                actor["key"]
            );
        }
    }
    for (id, line) in &ids.broadcasts {
        assert!(
            line.ends_with(locale[&i64::from(*id)].as_str()),
            "{line} is not the client's locale {id}"
        );
    }
    let announce = &event["schedule"]["announce"];
    if !announce.is_null() {
        let id = i64::from(int(&announce["locale_id"]));
        assert!(
            announce["text"]
                .as_str()
                .unwrap()
                .ends_with(locale[&id].as_str()),
            "the kickoff line is not the client's locale {id}"
        );
    }
    let models = read_models();
    for model in &ids.models {
        assert!(
            models.contains_key(&i64::from(*model)),
            "model {model} is not in Models.txt"
        );
    }
    let known = xml::ids(
        &lab("extracted/named/Assets_000/Assets_000/FabledRealmsAreas.xml"),
        "AreaDefinition",
    );
    assert!(
        ids.areas.iter().all(|id| known.contains(id)),
        "areas {:?}",
        ids.areas
    );
    let mut known = xml::ids(&resources("AnimationTypes.xml"), "AnimationSlot");
    known.extend(xml::ids(
        &resources("AnimationGroups.xml"),
        "AnimationGroup",
    ));
    assert!(
        ids.animations.iter().all(|id| known.contains(id)),
        "animations {:?}",
        ids.animations
    );
    let known = xml::ids(
        &resources("ActorCompositeEffectDefinitions.xml"),
        "EffectDefinition",
    );
    assert!(
        ids.effects.iter().all(|id| known.contains(id)),
        "effects {:?}",
        ids.effects
    );
    ids
}
