//! T544: the six Card Duelist and Snowhill Brawler NPCs are drawn from their regional rows
//! with client name ids, and the two Card Duelist records cite their original quests.
//!
//! Content only; no production Rust changed. `world.rs` draws a regional row only when its
//! `placement` is `legacy_data`, and the 35/2 AddNpc reader `0x00921a10` reads the u64 actor
//! id first, then the i32 name id and the i32 model id (`startup_world::parse_add_npc`).
//! T467 drew the four Card Duelist trainers as `legacy_data` rows; T470 drew Gerold and Tevin
//! as a `reference_data` row plus a duplicate `npc_spawns_extra.json` body. This slice converts
//! those two rows and removes their duplicate bodies, so every one of the six is a single
//! drawn actor at its Sanctuary-main spawn.
//!
//! Quest records stay disabled: no server path emits a card-duel win (T505). The dev teleport
//! target is the visibility index, so it moves with each row (checked below).

#[path = "common/npc_greetings.rs"]
mod npc_greetings;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_records.rs"]
mod quest_records;

use npc_quest_chain::{
    CHARACTER, Fixture, assert_no_offer, character_at_for, content_dir, content_position,
    read_json, seen_from,
};
use osfr_tools::world::NpcVisibility;
use osfr_tools::{
    quest_engine::{CharacterEligibility, QuestEngine},
    startup_world::NpcSpawn,
};
use serde_json::json;
use std::collections::BTreeMap;

const SAM: u64 = 9_000_000_187;
const POE: u64 = 9_000_000_170;
const BRY: u64 = 9_000_000_035;
const GARRISON: u64 = 9_000_000_094;
const GEROLD: u64 = 9_000_000_098;
const TEVIN: u64 = 9_000_000_215;

struct Row {
    npc: u64,
    file: &'static str,
    name: &'static str,
    /// The client `Global.Text` id the client draws as the nameplate.
    name_id: u32,
    /// The row's former raw locale hash; the reason must record it.
    old_hash: u64,
    model_id: u32,
    model_file: &'static str,
    position: [f64; 3],
    heading: f64,
}

/// Sanctuary-main@10f129e spawns (`FabledRealms.lua:3570,3585,3762,3852,3950,4163`) and
/// `Npcs.json` ids 33340, 33355, 33540, 33633, 33741 and 34006.
const ROWS: [Row; 6] = [
    Row {
        npc: SAM,
        file: "sanctuary.json",
        name: "Sam Potts",
        name_id: 74_093,
        old_hash: 1_159_662_914,
        model_id: 646,
        model_file: "human_f_tcg_sam_potts.agr",
        position: [-1290.0687, -14.91131, 291.80383],
        heading: 1.08519005,
    },
    Row {
        npc: POE,
        file: "sanctuary.json",
        name: "Poe Tatters",
        name_id: 38_111,
        old_hash: 4_287_171_865,
        model_id: 648,
        model_file: "human_m_tcg_poe_tatters.agr",
        position: [-1282.9117, -14.34194, 353.48657],
        heading: -3.09744,
    },
    Row {
        npc: BRY,
        file: "lakeshore.json",
        name: "Bry",
        name_id: 38_107,
        old_hash: 677_847_228,
        model_id: 643,
        model_file: "fairy_f_tcg_bry.agr",
        position: [-742.321, 1.05391, 844.33936],
        heading: -1.71367001,
    },
    Row {
        npc: GARRISON,
        file: "snowhill.json",
        name: "Garrison Gold",
        name_id: 38_092,
        old_hash: 4_090_496_530,
        model_id: 644,
        model_file: "fairy_m_tcg_rick_garfield.agr",
        position: [192.64722, 23.345482, 353.1109],
        heading: -0.00549800034,
    },
    Row {
        npc: GEROLD,
        file: "snowhill.json",
        name: "Gerold",
        name_id: 104_054,
        old_hash: 752_220_903,
        model_id: 966,
        model_file: "human_m_gerold.agr",
        position: [-207.1201, -19.14542, 290.6031],
        heading: -1.39930999,
    },
    Row {
        npc: TEVIN,
        file: "snowhill.json",
        name: "Tevin",
        name_id: 79_022,
        old_hash: 3_316_189_035,
        model_id: 657,
        model_file: "fairy_m_freestyle_08.agr",
        position: [303.51282, 27.23725, 410.2941],
        heading: -1.07071997,
    },
];

fn expected_position(row: &Row) -> [f32; 3] {
    row.position.map(|value| value as f32)
}

#[test]
fn six_npcs_are_drawn_with_client_names() {
    let base = NpcVisibility::load(&content_dir()).expect("NPC visibility loads");
    let extras = read_json(&content_dir().join("npc_spawns_extra.json"));
    let extra_rows = extras["records"].as_array().unwrap();
    let targets: BTreeMap<u64, [f32; 3]> = base
        .teleport_targets(0)
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    for row in &ROWS {
        let record = npc_greetings::npc_record(row.file, row.npc);
        assert_eq!(record["name"], row.name);
        assert_eq!(record["placement"], "legacy_data", "{}", row.name);
        assert_eq!(
            record["placement_source"], "legacy_data_spawn",
            "{}",
            row.name
        );
        assert_eq!(record["name_string_id"].as_u64(), Some(row.name_id as u64));
        assert_eq!(
            record["name_string_ids"],
            json!([{"table": "client_locale", "id": row.name_id}]),
            "{}",
            row.name
        );
        assert_eq!(record["model_id"].as_u64(), Some(row.model_id as u64));
        assert_eq!(record["model_file_name"], row.model_file);
        assert_eq!(record["heading"].as_f64(), Some(row.heading));
        let position = expected_position(row);
        assert!(
            position.iter().all(|value| value.is_finite()),
            "{}",
            row.name
        );
        assert_eq!(
            serde_json::from_value::<[f32; 3]>(record["position"].clone()).unwrap(),
            position,
            "{}",
            row.name
        );

        // One body per actor: a second extras row would be a duplicate AddNpc.
        assert!(
            !extra_rows.iter().any(|extra| {
                extra["spawn_guid"].as_u64() == Some(row.npc)
                    || extra["content_key"].as_u64() == Some(row.npc)
            }),
            "{}: a second extras body is drawn beside the regional row",
            row.name
        );

        // The loader draws the regional row as itself, and the dev-teleport index follows it.
        assert_eq!(
            base.content_key(row.npc),
            Some(row.npc),
            "{}: the actor binds its own content row",
            row.name
        );
        assert_eq!(base.actor_position(row.npc), Some(position), "{}", row.name);
        assert_eq!(
            targets.get(&row.npc),
            Some(&position),
            "{}: the dev-teleport target moved with the row",
            row.name
        );

        let seen: Vec<NpcSpawn> = seen_from(&base, position)
            .into_iter()
            .filter(|npc| npc.actor_id == row.npc)
            .collect();
        assert_eq!(seen.len(), 1, "{}: drawn {} times", row.name, seen.len());
        let npc = &seen[0];
        let drawn = [npc.position[0], npc.position[1], npc.position[2]];
        assert_eq!(drawn, position, "{}: AddNpc position", row.name);
        assert_eq!(
            npc.name_string_id, row.name_id as i32,
            "{}: the AddNpc name is a Global.Text id",
            row.name
        );
        assert_eq!(npc.model_id, row.model_id as i32, "{}", row.name);
        let heading = row.heading as f32;
        assert_eq!(
            npc.rotation,
            [heading.sin(), 0.0, heading.cos(), 0.0],
            "{}: AddNpc facing",
            row.name
        );
    }
}

#[test]
fn the_six_name_ids_resolve() {
    let locale = npc_greetings::locale_id_to_text();
    for row in &ROWS {
        let record = npc_greetings::npc_record(row.file, row.npc);
        let name_id = record["name_string_id"]
            .as_u64()
            .unwrap_or_else(|| panic!("{}: name_string_id", row.name));
        assert_eq!(
            locale.get(&name_id).map(String::as_str),
            Some(row.name),
            "{}: Global.Text {}",
            row.name,
            name_id
        );
        // The replaced value was a raw locale hash, not a Global.Text id: the client's
        // nameplate lookup by id would draw nothing.
        assert!(
            !locale.contains_key(&row.old_hash),
            "{}: the old hash {} is still a locale key",
            row.name,
            row.old_hash
        );
        let reason = record["name_string_id_reason"]
            .as_str()
            .unwrap_or_else(|| panic!("{}: name_string_id_reason", row.name));
        assert!(
            reason.contains(&row.old_hash.to_string()),
            "{}: the reason does not record the replaced hash {}",
            row.name,
            row.old_hash
        );
    }
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: false,
        nonmember_quest_limit: 30,
        member_quest_limit: 30,
    }
}

#[test]
fn card_duelist_and_brawler_quests_stay_disabled() {
    let quests = quest_records::read_quests(&content_dir());
    // All 63 Card Duelist quests and the Brawler line's disabled records (3030014 is Tevin's).
    let disabled: Vec<u32> = (3_040_001..=3_040_063)
        .chain(3_030_010..=3_030_014)
        .collect();
    for id in &disabled {
        assert!(
            !quest_records::find_quest(&quests, *id).enabled,
            "{id} is enabled"
        );
    }
    let engine = QuestEngine::from_split_content(&content_dir(), CHARACTER, eligibility(), None)
        .expect("the shipped quest content loads");
    for id in disabled {
        assert!(!engine.is_enabled(id), "{id} is offered");
    }

    // Sam Potts is the Card Duelist trainer; with every quest disabled, Talk offers none.
    let fixture = Fixture::new();
    let mut game = fixture.game(None);
    let sam = character_at_for(CHARACTER, 0, content_position("sanctuary.json", SAM));
    let mut now = 1_000;
    assert_no_offer(&mut game, &sam, SAM, &mut now, "Sam Potts");
}

/// One client source tag that records every fact about the original quest.
fn source_text<'a>(record: &'a serde_json::Value, facts: &[&str]) -> &'a str {
    record["source"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|tag| tag["kind"] == "client")
        .filter_map(|tag| tag["evidence"].as_str())
        .find(|text| facts.iter().all(|fact| text.contains(fact)))
        .unwrap_or_else(|| panic!("no client source tag records {facts:?}"))
}

#[test]
fn original_ids_are_recorded() {
    let document = read_json(&content_dir().join("quests/card_duelist.json"));
    let records = document["records"].as_array().unwrap();
    let record = |id: u64| {
        records
            .iter()
            .find(|record| record["id"].as_u64() == Some(id))
            .unwrap_or_else(|| panic!("quest {id}"))
    };

    // 1304 'Poe's Best Deck' (objective 12597, reward item 51187).
    source_text(
        record(3_040_011),
        &[
            "original quest 1304",
            "12597",
            "103739",
            "51187",
            "original-quest-coverage.json",
        ],
    );

    // 1875 'Esther, the Brains' (objectives 15573 and 15574).
    source_text(
        record(3_040_034),
        &["original quest 1875", "15573", "407045", "15574", "407047"],
    );

    // The link is provenance only: both records stay disabled.
    for id in [3_040_011_u64, 3_040_034] {
        assert_eq!(record(id)["enabled"].as_bool(), Some(false), "{id}");
    }
}
