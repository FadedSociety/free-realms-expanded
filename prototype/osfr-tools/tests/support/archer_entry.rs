//! Shared fixture and packet helpers for the Archer entry chapter tests.
//!
//! `archer_entry_chain.rs` and `archer_entry_content.rs` include this file with
//! `#[path = "support/archer_entry.rs"] mod archer_entry;`. Test-only; production never copies
//! content. The fixture supplies profile/inventory content; the repository's
//! `job_trainers.json`, `npcs/` and split quests are bound from `data/content` exactly as the
//! bundled play package does. The generic quest-chain harness lives in
//! `tests/support/npc_quest_chain.rs`, shared with the Medic chapter tests.
#![allow(dead_code)]

#[path = "npc_quest_chain.rs"]
mod npc_quest_chain;

pub use npc_quest_chain::*;

use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, Outcome},
    quest_engine::QuestStatus,
};
use serde_json::Value;
use std::collections::BTreeSet;

pub const LUGABOW: u64 = 9_000_000_387;
pub const BOLLYLOG: u64 = 9_500_000_289;
pub const HRUGAGU: u64 = 9_500_000_309;
pub const BRUGADUG: u64 = 9_500_000_290;
pub const HILFORN: u64 = 9_000_000_106;
pub const JONES: u64 = 9_000_000_121;
pub const BARTLE: u64 = 9_000_000_021;
pub const PATH: u32 = 3_010_003;
pub const OATH: u32 = 3_010_006;
pub const ARCHER: i32 = 35;
pub const BRAWLER: i32 = 43;
/// `job_trainers.json` row for Lugabow (profile 35).
pub const LEARN_LABEL: u32 = 412_187;
pub const PATH_COINS: u32 = 5;
pub const PATH_XP: u32 = 15;
/// The Path debrief locale id the 49/13 carries at byte 18..22.
pub const PATH_END_TEXT: u32 = 9_701;
pub const OATH_COINS: u32 = 10;
pub const OATH_XP: u32 = 20;
/// The Oath debrief locale id the 49/13 carries at byte 18..22.
pub const OATH_END_TEXT: u32 = 77_038;
pub const GREENWOOD: u32 = 6;
pub const COBBLESTONE: u32 = 3;

/// One runtime actor bound to a quest content key. The binding is the only way a
/// client click on an extra-spawn actor reaches the recruit's content row.
pub fn recruit_spawn(content_key: u64) -> (u64, [f32; 3]) {
    let document = read_json(&content_dir().join("npc_spawns_extra.json"));
    let rows: Vec<&Value> = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["content_key"].as_u64() == Some(content_key))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "content key {content_key} must bind exactly one runtime actor"
    );
    let row = rows[0];
    (
        row["spawn_guid"].as_u64().expect("bound spawn GUID"),
        serde_json::from_value(row["position"].clone()).expect("bound spawn position"),
    )
}

pub fn learn_archer(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    let menu = interact(game, character, LUGABOW, *now);
    let learn = menu_option(&menu, LEARN_LABEL);
    *now += 10;
    let learned = send(game, character, &npc_select(LUGABOW, learn), *now);
    assert!(learned.changed, "Lugabow's Learn choice must commit");
    assert!(
        game.state()
            .profiles
            .as_ref()
            .unwrap()
            .unlocked
            .contains(&ARCHER),
        "Lugabow must unlock Archer"
    );
    *now += 10;
    let switched = send(game, character, &select_job(ARCHER), *now);
    assert!(
        switched.changed,
        "Archer selection must activate; its starter outfit must be complete"
    );
    assert_eq!(
        game.state().profiles.as_ref().unwrap().active,
        ARCHER,
        "the job gate needs Archer active"
    );
}

pub fn activate_brawler(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    if game.state().profiles.as_ref().unwrap().active == BRAWLER {
        return;
    }
    *now += 10;
    let switched = send(game, character, &select_job(BRAWLER), *now);
    assert!(switched.changed, "Brawler must activate");
    assert_eq!(game.state().profiles.as_ref().unwrap().active, BRAWLER);
}

pub fn start_path(game: &mut Gameplay, character: &CharacterState, now: &mut u64) {
    *now += 10;
    offer(game, character, LUGABOW, PATH, *now);
    *now += 10;
    accept(game, character, PATH, *now);
}

pub fn talk_recruit(
    game: &mut Gameplay,
    content_key: u64,
    now: &mut u64,
) -> (Outcome, u64, [f32; 3]) {
    let (runtime_guid, position) = recruit_spawn(content_key);
    let character = character_at(GREENWOOD, position);
    *now += 10;
    let outcome = talk(game, &character, runtime_guid, *now);
    (outcome, runtime_guid, position)
}

/// The client's fourth Path step is "Return to Lugabow" (locale 22083), so the
/// turn-in is at the giver, not at the third recruit.
pub fn complete_path_via_recruits(game: &mut Gameplay, now: &mut u64) {
    assert_eq!(quest_status(game, PATH), QuestStatus::Active);
    let before = counts(game, PATH);
    if before[0] == 0 {
        let (first, _, _) = talk_recruit(game, BOLLYLOG, now);
        assert!(first.changed);
        assert_eq!(counts(game, PATH), vec![1, 0, 0, 0]);
        assert_eq!(pending_turnin(game), None);
    }
    if before[1] == 0 {
        let (second, _, _) = talk_recruit(game, HRUGAGU, now);
        assert!(second.changed);
        assert_eq!(counts(game, PATH), vec![1, 1, 0, 0]);
        assert_eq!(pending_turnin(game), None);
    }
    if before[2] == 0 {
        let (third, _, _) = talk_recruit(game, BRUGADUG, now);
        assert!(third.changed);
        assert_eq!(counts(game, PATH), vec![1, 1, 1, 0]);
        assert_eq!(pending_turnin(game), None);
        assert!(
            !third
                .replies
                .iter()
                .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
            "the third recruit is not the turn-in; the client returns to Lugabow"
        );
    }

    let position = content_position("greenwood_forest.json", LUGABOW);
    let character = character_at(GREENWOOD, position);
    *now += 10;
    let returned = talk(game, &character, LUGABOW, *now);
    assert!(returned.changed);
    assert_eq!(counts(game, PATH), vec![1, 1, 1, 1]);
    assert_eq!(pending_turnin(game), Some(PATH));
    assert!(
        !returned
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the turn-in screen waits for the 26/6 dialogue choice"
    );

    *now += 10;
    let _ = release_end_screen(game, &character, PATH, LUGABOW, PATH_END_TEXT, *now);
    *now += 10;
    finish_quest(game, &character, PATH, ARCHER, PATH_COINS, PATH_XP, *now);
}

pub fn complete_oath_via_hilforn(game: &mut Gameplay, now: &mut u64) {
    let position = content_position("greenwood_forest.json", HILFORN);
    let character = character_at(GREENWOOD, position);
    *now += 10;
    let reported = talk(game, &character, HILFORN, *now);
    assert!(reported.changed);
    assert_eq!(counts(game, OATH), vec![1]);
    assert_eq!(pending_turnin(game), Some(OATH));
    assert!(
        !reported
            .replies
            .iter()
            .any(|packet| packet.starts_with(&[49, 0, 13, 0])),
        "the turn-in screen waits for the 26/6 dialogue choice"
    );

    *now += 10;
    let _ = release_end_screen(game, &character, OATH, HILFORN, OATH_END_TEXT, *now);
    *now += 10;
    finish_quest(game, &character, OATH, ARCHER, OATH_COINS, OATH_XP, *now);
}

pub fn archer_xp(game: &Gameplay) -> u32 {
    profile_xp(game, ARCHER)
}

/// Shared by the protected chain test (the integrator pins that target/name) and the
/// focused content target, so both run the same checks without a second copy.
pub fn assert_enabled_archer_chapter_records() {
    let document = read_json(&content_dir().join("quests/archer.json"));
    let records = document["records"].as_array().unwrap();
    let enabled: Vec<&Value> = records
        .iter()
        .filter(|record| record["enabled"].as_bool().unwrap_or(false))
        .collect();
    let enabled_ids: Vec<u64> = enabled
        .iter()
        .map(|record| record["id"].as_u64().unwrap())
        .collect();
    assert_eq!(enabled_ids, vec![u64::from(PATH), u64::from(OATH)]);

    for record in &enabled {
        assert_eq!(record["giver_npc_id"].as_u64(), Some(LUGABOW));
        assert_eq!(record["job"].as_i64(), Some(i64::from(ARCHER)));
        let objectives = record["objectives"].as_array().unwrap();
        assert!(!objectives.is_empty(), "an enabled quest needs objectives");
        assert!(
            objectives.iter().all(|goal| goal["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every objective needs provenance"
        );
        let rewards = record["rewards"].as_array().unwrap();
        assert!(!rewards.is_empty(), "every enabled Archer record pays");
        assert!(
            rewards.iter().all(|reward| reward["source"]
                .as_array()
                .is_some_and(|source| !source.is_empty())),
            "every reward needs provenance"
        );
    }

    let path = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(PATH)))
        .unwrap();
    assert_eq!(path["title_string_id"].as_u64(), Some(22_075));
    assert_eq!(path["prerequisite_quest_ids"], serde_json::json!([]));
    let path_targets: Vec<u64> = path["objectives"]
        .as_array()
        .unwrap()
        .iter()
        .map(|goal| goal["npc_id"].as_u64().unwrap())
        .collect();
    assert_eq!(
        path_targets,
        vec![BOLLYLOG, HRUGAGU, BRUGADUG, LUGABOW],
        "Path is the three recruit talks, then the client's Return to Lugabow step"
    );
    assert_eq!(
        path["end_text_string_id"].as_u64(),
        Some(u64::from(PATH_END_TEXT)),
        "the Path end screen must carry the recovered debrief text"
    );
    assert_eq!(
        path["rewards"][0]["coins"].as_u64(),
        Some(u64::from(PATH_COINS))
    );
    assert_eq!(
        path["rewards"][0]["job_xp"]["35"].as_u64(),
        Some(u64::from(PATH_XP))
    );
    assert_eq!(path["rewards"][0]["items"], serde_json::json!([]));

    let oath = records
        .iter()
        .find(|record| record["id"].as_u64() == Some(u64::from(OATH)))
        .unwrap();
    assert_eq!(oath["title_string_id"].as_u64(), Some(77_036));
    assert_eq!(oath["prerequisite_quest_ids"], serde_json::json!([PATH]));
    assert_eq!(oath["objectives"][0]["npc_id"].as_u64(), Some(HILFORN));
    assert_eq!(
        oath["end_text_string_id"].as_u64(),
        Some(u64::from(OATH_END_TEXT))
    );
    assert_eq!(
        oath["rewards"][0]["coins"].as_u64(),
        Some(u64::from(OATH_COINS))
    );
    assert_eq!(
        oath["rewards"][0]["job_xp"]["35"].as_u64(),
        Some(u64::from(OATH_XP))
    );
    assert_eq!(oath["rewards"][0]["items"], serde_json::json!([]));

    // The chapter deliberately leaves the minigame/field-enemy records disabled.
    for quest_id in [3_010_004_u64, 3_010_005, 3_010_007] {
        let record = records
            .iter()
            .find(|record| record["id"].as_u64() == Some(quest_id))
            .unwrap();
        assert_eq!(record["enabled"].as_bool(), Some(false));
    }

    // Lugabow (the Path giver and fourth objective) and Hilforn (the Oath
    // target) are existing Legacy actors. Every Path target must have exactly
    // one regional row: a duplicate would let `find` silently pick a different
    // spawn/click anchor while the chain test stayed green.
    let npcs = read_json(&content_dir().join("npcs/greenwood_forest.json"));
    let regional_rows = |content_key: u64| -> Vec<&Value> {
        npcs["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["npc_id"].as_u64() == Some(content_key))
            .collect()
    };
    for content_key in &path_targets {
        let rows = regional_rows(*content_key);
        assert_eq!(
            rows.len(),
            1,
            "Path target {content_key} must have exactly one regional row"
        );
        assert_eq!(rows[0]["zone_id"].as_u64(), Some(u64::from(GREENWOOD)));
        assert!(
            rows[0]["roles"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| role == "quest_giver"),
            "Path target {content_key} must be a quest giver"
        );
    }

    let lugabow_row = regional_rows(LUGABOW)[0];
    assert_eq!(lugabow_row["placement"].as_str(), Some("legacy_data"));
    assert!(lugabow_row["position"].is_array());

    // The Oath objective's report target is a data-backed spawned quest giver.
    let hilforn_rows = regional_rows(HILFORN);
    assert_eq!(
        hilforn_rows.len(),
        1,
        "Oath target {HILFORN} must have exactly one regional row"
    );
    let hilforn_row = hilforn_rows[0];
    assert_eq!(hilforn_row["placement"].as_str(), Some("legacy_data"));
    assert_eq!(hilforn_row["zone_id"].as_u64(), Some(u64::from(GREENWOOD)));
    assert!(
        hilforn_row["roles"]
            .as_array()
            .unwrap()
            .iter()
            .any(|role| role == "quest_giver")
    );

    // Each enabled recruit also binds exactly one existing runtime actor, so the
    // client can click the recruit it sees; Lugabow is spawned from its regional
    // legacy_data row, never an extra spawn.
    let extra = read_json(&content_dir().join("npc_spawns_extra.json"));
    assert!(
        !extra["records"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["content_key"].as_u64() == Some(LUGABOW)),
        "Path giver {LUGABOW} must not bind an extra spawn"
    );
    let mut runtime_guids = BTreeSet::new();
    for (content_key, legacy_id, locale_id) in [
        (BOLLYLOG, 3_468_u64, 22_168_u64),
        (HRUGAGU, 3_469, 22_169),
        (BRUGADUG, 3_470, 22_170),
    ] {
        let (runtime_guid, runtime_position) = recruit_spawn(content_key);
        assert!(runtime_guid > 1_000_000_000, "runtime actor id range");
        assert!(
            runtime_guids.insert(runtime_guid),
            "recruit {content_key} reuses runtime actor id {runtime_guid}"
        );
        let recruit = regional_rows(content_key)[0];
        assert_eq!(recruit["name_string_id"].as_u64(), Some(locale_id));
        assert_eq!(recruit["placement"].as_str(), Some("unanchored"));
        let position: [f32; 3] = serde_json::from_value(recruit["position"].clone()).unwrap();
        assert_eq!(
            position, runtime_position,
            "the range anchor must match the bound runtime actor"
        );
        let bound: Vec<&Value> = extra["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["spawn_guid"].as_u64() == Some(runtime_guid))
            .collect();
        assert_eq!(
            bound.len(),
            1,
            "runtime actor {runtime_guid} must have one binding row"
        );
        assert_eq!(bound[0]["legacy_npc_id"].as_u64(), Some(legacy_id));
    }
    assert_eq!(
        runtime_guids.len(),
        3,
        "the three recruits need distinct runtime actors"
    );
}
