//! T699: the enemy definitions and field spawns that back the kill objectives
//! whose target had no `enemies.json` row.
//!
//! The worklist is `lab/references/quest-lines/wiring/enemy-rows-needed.json`
//! (66 objectives, 22 distinct targets recovered here). Every row's
//! `actor_kind_id` is a FreeRealms-Legacy `Npcs.json` definition id, and the
//! field spawn sits on that definition's `FabledRealms.lua` spawn line.
//!
//! Included: the strict catalogue loader (`EnemyEngine::load`), the client
//! `Models.txt`/locale tables the names and models must resolve in, and the real
//! `EncounterEngine` spawn/kill path (a defeat reports the row's definition id,
//! which `Gameplay` turns into `QuestEvent::KillActorKind`).

use osfr_tools::encounter::EncounterOutcome;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/field_enemy_fixture.rs"]
mod field_enemy_fixture;
#[path = "support/json.rs"]
mod support_json;
#[allow(dead_code)] // Only the locale and Models.txt readers are used here.
#[path = "support/world_event_content.rs"]
mod world_event_content;

use field_enemy_fixture::{Fixture, content_dir, spawned};
use support_json::read as read_json;
use world_event_content::{locale, read_models};

/// The `EncounterEngine` profile ids used by the shipped content tests
/// (Wizard 12, the weakest job's health multiplier).
const WIZARD: i32 = 12;

/// Every definition T699 appended: id, actor_kind_id/definition_id, model id,
/// name string id, boss. The five bosses cannot be field enemies
/// (`load_field_enemies` refuses `boss`), so they are row-only.
const ROWS: [(&str, i64, i64, u64, bool); 22] = [
    ("thugawug_thug", 4115, 199, 75450, false),
    ("thugawug_fumbler", 1638, 199, 75454, false),
    ("thugawug_sneak", 1639, 199, 75449, false),
    ("thugawug_bumbler", 3491, 200, 75453, false),
    ("robgoblin_pondblaster", 1294, 4, 95047, false),
    ("robgoblin_initiate", 1474, 4, 39582, false),
    ("fungus_golem", 2660, 235, 4477, false),
    ("vicious_vinewolf", 2872, 142, 103401, false),
    ("hedge_guardian", 2644, 74, 4084, false),
    ("smuggler_han", 2834, 669, 77317, true),
    ("smuggler_hannah", 1551, 685, 77319, true),
    ("skullz_brawler", 3965, 4011, 439761, false),
    ("hewey_the_bear", 1237, 91, 5734, true),
    ("frostfang_growler", 2529, 3284, 104067, false),
    ("troll_roughguard", 3115, 77, 36237, false),
    ("willow_slamdash", 3066, 1296, 19366, false),
    ("troll_spider_trainer", 3148, 77, 5135, false),
    ("troll_naturalist", 3135, 77, 5234, false),
    ("frostfang_snarler", 3592, 1938, 78096, false),
    ("battle_warthog", 4857, 139, 23196, false),
    ("foreman_ironfist", 1495, 10, 71199, true),
    ("queen_arachnia", 2714, 53, 3592, true),
];

/// Fresh actor ids for the quest actors: "FIEL" plus a counter (the two T245
/// quest Hooligans hold counters 1 and 2). They must not be the retail actor
/// GUIDs `npc_spawns_extra.json` draws for the same NPCs: a quest end removes
/// the field actor's GUID from the client.
const FRESH_GUID: u64 = 0x4649_454c_0000_0000;

/// Every field spawn T699 appended: GUID, definition id, authored level and the
/// quests that make it present. Willow Slamdash (q3060010, a derby knock-out) and
/// the Frostfang Growler (inside the opening wolf walk's clearance) keep a
/// catalogue row but no spawn.
const SPAWNS: [(u64, i64, u32, &[u32]); 15] = [
    (FRESH_GUID + 3, 4115, 5, &[3000024]),
    (FRESH_GUID + 4, 1638, 5, &[3010005]),
    (FRESH_GUID + 5, 1639, 5, &[3010018, 3150012]),
    (FRESH_GUID + 6, 3491, 9, &[3150012]),
    (FRESH_GUID + 7, 1294, 5, &[3010008, 3010011, 3090007]),
    (FRESH_GUID + 8, 1474, 9, &[3160006]),
    (FRESH_GUID + 9, 2660, 5, &[3000040]),
    (FRESH_GUID + 10, 2872, 5, &[3010025]),
    (FRESH_GUID + 11, 2644, 7, &[3010025]),
    (FRESH_GUID + 12, 3965, 7, &[3020035]),
    (FRESH_GUID + 13, 3115, 9, &[3030040]),
    (FRESH_GUID + 14, 3148, 13, &[3110025]),
    (FRESH_GUID + 15, 3135, 13, &[3110025]),
    (FRESH_GUID + 16, 3592, 13, &[3150002]),
    (FRESH_GUID + 17, 4857, 13, &[3160031]),
];

/// Catalogue rows that never spawn in the field.
const NO_FIELD_SPAWN: [&str; 2] = ["frostfang_growler", "willow_slamdash"];

fn quest_ids() -> BTreeSet<u32> {
    SPAWNS
        .iter()
        .flat_map(|(_, _, _, quests)| quests.iter().copied())
        .collect()
}

fn definition<'a>(defs: &'a [Value], id: i64) -> &'a Value {
    defs.iter()
        .find(|d| d["definition_id"].as_i64() == Some(id))
        .unwrap_or_else(|| panic!("enemies.json has no definition {id}"))
}

#[test]
fn the_new_rows_load_with_unique_actor_kinds_and_client_names_and_models() {
    let root = content_dir();
    // The real catalogue loader: strict schema, `deny_unknown_fields`, bounds.
    osfr_tools::enemies::EnemyEngine::load(&root).expect("the shipped enemy catalogue loads");

    let file = read_json(&root.join("enemies.json"));
    let defs = file["definitions"].as_array().unwrap();
    let mut actor_kinds = BTreeSet::new();
    let mut definition_ids = BTreeSet::new();
    for def in defs {
        let actor_kind = def["actor_kind_id"].as_i64().unwrap();
        assert!(
            actor_kinds.insert(actor_kind),
            "duplicate actor_kind_id {} ({})",
            actor_kind,
            def["id"]
        );
        assert!(
            definition_ids.insert(def["definition_id"].as_i64().unwrap()),
            "duplicate definition_id {}",
            def["definition_id"]
        );
        // The kill event carries `definition_id` as the objective's actor kind.
        assert_eq!(def["actor_kind_id"], def["definition_id"], "{}", def["id"]);
        // Every stat stays inside the catalogue's sanity bounds.
        let level = def["level"].as_u64();
        assert!(level.is_none_or(|level| (1..=20).contains(&level)), "{def}");
        assert!(def["max_health"].as_i64().unwrap() > 0, "{def}");
        assert!(
            (1..=10_000).contains(&def["max_health"].as_i64().unwrap()),
            "{def}"
        );
        assert!(
            (0..=500).contains(&def["damage"].as_i64().unwrap()),
            "{def}"
        );
        assert!(
            (0.0..=20.0).contains(&def["move_speed"].as_f64().unwrap()),
            "{def}"
        );
        assert!(
            (0.0..=100.0).contains(&def["aggro_radius"].as_f64().unwrap()),
            "{def}"
        );
    }

    let names = locale();
    let client_models = read_models();
    let crosswalk = read_json(&root.join("enemies_unresolved.json"));
    let bindings: BTreeMap<i64, &Value> = crosswalk["model_asset_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|binding| (binding["definition_id"].as_i64().unwrap(), binding))
        .collect();

    for (key, actor_kind, model_id, name_id, boss) in ROWS {
        let def = definition(defs, actor_kind);
        assert_eq!(def["id"], key);
        assert_eq!(def["actor_kind_id"].as_i64(), Some(actor_kind), "{key}");
        assert_eq!(def["definition_id"].as_i64(), Some(actor_kind), "{key}");
        assert_eq!(def["model_id"].as_i64(), Some(model_id), "{key}");
        assert_eq!(def["name_string_id"].as_u64(), Some(name_id), "{key}");
        assert_eq!(def["boss"].as_bool(), Some(boss), "{key}");
        // The client must be able to name the actor: the locale resolves the id
        // to the row's kind (the catalogue's own content test relies on this).
        assert_eq!(
            names
                .get(&i64::try_from(name_id).unwrap())
                .map(String::as_str),
            def["kind"].as_str(),
            "{key}: name id {name_id} must resolve to the kind"
        );
        // The model must exist in the client model table, bound to the row's ADR.
        let binding = bindings.get(&actor_kind).expect("crosswalk binding");
        let adr = binding["adr"].as_str().unwrap().to_ascii_lowercase();
        assert!(
            client_models
                .get(&model_id)
                .is_some_and(|files| files.contains(&adr)),
            "{key}: Models.txt has no {model_id} -> {adr}"
        );
        assert_eq!(binding["enemy_id"], key);
        // The worklist entry is cited on the row itself.
        let source = def["source"].as_str().unwrap();
        assert!(
            source.contains("enemy-rows-needed.json"),
            "{key} must cite the worklist: {source}"
        );
    }
}

#[test]
fn every_new_field_spawn_is_present_only_with_its_quest_and_live_in_the_field() {
    let quests = quest_ids();
    let content = content_dir();
    let field = read_json(&content.join("field_enemies.json"));
    let rows = field["enemies"].as_array().unwrap();
    let catalogue = read_json(&content.join("enemies.json"));
    let defs = catalogue["definitions"].as_array().unwrap();
    let names = locale();
    // Actor ids the NPC placements already draw: a shared id would let a quest
    // end remove the world NPC.
    let npc_guids: BTreeSet<u64> = read_json(&content.join("npc_spawns_extra.json"))["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|record| record["spawn_guid"].as_u64().unwrap())
        .collect();
    for (guid, definition_id, level, quest_ids) in SPAWNS {
        let row = rows
            .iter()
            .find(|row| row["guid"].as_u64() == Some(guid))
            .unwrap_or_else(|| panic!("field row {guid} is missing"));
        assert!(
            !npc_guids.contains(&guid),
            "{guid} is also an NPC placement"
        );
        assert_eq!(row["definition_id"].as_i64(), Some(definition_id));
        assert_eq!(row["level"], json!(level));
        assert_eq!(row["quest_ids"], json!(quest_ids));
        assert_eq!(
            row["boss"],
            json!(false),
            "{guid}: a field actor has no boss"
        );
        assert!(!row["source"].as_array().unwrap().is_empty(), "{guid}");
        // The client draws the actor from the row: model, name id and name must
        // be the catalogue definition's and the locale's, not just load.
        let def = definition(defs, definition_id);
        assert_eq!(row["model_id"], def["model_id"], "{guid} model");
        assert_eq!(row["name_id"], def["name_string_id"], "{guid} name id");
        assert_eq!(
            names.get(&i64::try_from(row["name_id"].as_u64().unwrap()).unwrap()),
            row["name"].as_str().map(str::to_owned).as_ref(),
            "{guid} name must be the locale text of its name id"
        );
        // A quest target is not a faucet: no drop table on the row, and the
        // payout table pays it nothing (checked end to end in the defeat test).
        assert!(
            row.get("power_up_table").is_none(),
            "{guid} drops a power-up"
        );
    }
    // A row without a reference spawn, or a boss, stays catalogue-only.
    let spawned_defs: BTreeSet<i64> = SPAWNS.iter().map(|(_, def, _, _)| *def).collect();
    for (key, actor_kind, _, _, boss) in ROWS {
        assert_eq!(
            spawned_defs.contains(&actor_kind),
            !boss && !NO_FIELD_SPAWN.contains(&key),
            "{key}"
        );
    }

    // The real engine: nothing is present without the quests, everything with them.
    let mut quiet = Fixture::shipped().load();
    quiet.set_player_job(WIZARD, 1);
    quiet.on_zone_ready(WIZARD, 0).unwrap();
    let quiet = quiet.set_active_quests(&BTreeSet::new());
    let quiet = spawned(&quiet);
    let mut live = Fixture::shipped().load();
    live.set_player_job(WIZARD, 1);
    live.on_zone_ready(WIZARD, 0).unwrap();
    let out = live.set_active_quests(&quests);
    let present = spawned(&out);
    for (guid, _, level, _) in SPAWNS {
        assert!(!quiet.contains(&guid), "{guid} must stay quest-gated");
        assert!(present.contains(&guid), "{guid} must spawn with its quest");
        assert_eq!(live.field_level(guid), Some(level), "{guid}");
        assert_eq!(
            live.field_teleport_targets()
                .into_iter()
                .find(|target| target.guid == guid)
                .map(|target| target.position),
            rows.iter()
                .find(|row| row["guid"].as_u64() == Some(guid))
                .map(|row| serde_json::from_value::<[f32; 3]>(row["position"].clone()).unwrap()),
            "{guid} must stand on its reference spawn"
        );
    }
}

#[test]
fn a_defeat_of_a_new_spawn_reports_its_actor_kind_and_pays_nothing() {
    // The shipped field payout table and drop tables, so a reward or a
    // power-up would show; the quest event itself is wired by step 5.
    let mut engine = Fixture::shipped().load_paying();
    engine.set_player_job(WIZARD, 20);
    engine.on_zone_ready(WIZARD, 0).unwrap();
    engine.set_active_quests(&quest_ids());
    // One monotonic clock: `strike` refuses a second swing within 600 ms.
    let mut now = 10_000_u64;
    for (guid, definition_id, _, _) in SPAWNS {
        let at = engine
            .field_teleport_targets()
            .into_iter()
            .find(|target| target.guid == guid)
            .unwrap_or_else(|| panic!("{guid} is not live"))
            .position;
        let mut kill: Option<EncounterOutcome> = None;
        for _ in 0..16 {
            now += 1_000;
            let out = engine.attack(guid, at, WIZARD, 0, now).unwrap();
            if out
                .defeated
                .contains(&i32::try_from(definition_id).unwrap())
            {
                kill = Some(out);
                break;
            }
        }
        let kill = kill.unwrap_or_else(|| panic!("{guid} survived sixteen level-20 swings"));
        assert_eq!(
            kill.defeated,
            vec![i32::try_from(definition_id).unwrap()],
            "{guid} defeat must carry the row's definition id, the quest event's actor_kind_id"
        );
        assert!(kill.rewards.is_empty(), "{guid} paid {:?}", kill.rewards);
        assert_eq!(engine.pickup_count(), 0, "{guid} dropped a power-up");
    }
}

#[test]
fn a_refused_new_row_leaves_the_live_field_unchanged() {
    let fixture = Fixture::copied();
    let mut engine = fixture.load();
    engine.set_active_quests(&quest_ids());
    let before: Vec<(u64, [f32; 3])> = engine
        .field_teleport_targets()
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    let (first, ..) = SPAWNS[0];
    assert!(before.iter().any(|(guid, _)| *guid == first));

    // Break one T699 row in a private copy: the loader refuses level 0 and the
    // engine keeps every live actor and its position.
    let path = fixture.field_path().to_path_buf();
    let mut field: Value = read_json(&path);
    let rows = field["enemies"].as_array_mut().unwrap();
    let row = rows
        .iter_mut()
        .find(|row| row["guid"].as_u64() == Some(first))
        .expect("the thugawug thug spawn");
    row["level"] = json!(0);
    fs::write(&path, field.to_string()).unwrap();

    assert!(engine.reload_field_enemies(&path).is_err());
    let after: Vec<(u64, [f32; 3])> = engine
        .field_teleport_targets()
        .into_iter()
        .map(|target| (target.guid, target.position))
        .collect();
    assert_eq!(after, before, "a refused reload changes nothing");
}
