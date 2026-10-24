//! T383: the shipped field-battle catalogue gate.
//!
//! Every row of `data/content/field_battles.json` is loaded through the real
//! `EncounterEngine` loader and checked for what a player needs before a friend
//! clicks it: a unique, drawable open-world marker at the authored position, a
//! non-empty arena roster the loader resolves, a prize wheel whose definitions
//! the merged item catalogue grants, and a published 167/1/1 activity row. One
//! marker battle and one start-screen battle then run the real marker -> Go ->
//! kill -> results -> Close route. A malformed copy of a row proves the loader
//! and the prize preflight refuse without replacing the loaded catalogue.
//!
//! Rows are derived from the shipped file, never a fixed list, so a row added by
//! a later content slice is audited automatically and a failure names its id.
//!
//! The wire shapes are the existing pinned ones, not new protocol: the shared
//! fixture carries 41/108 (ctor `0x008b6e70`, sender `0x00938f00`), the 41/114
//! start screen (reader chain `0x00aa32d0` -> `0x00a29120` -> `0x009bdd70`),
//! 36/10 (writer `0x00a342e0`), 26/42 close (sender `0x0092f300`), the 35/2
//! AddNpc reader `0x00921a10` and the 167/1/1 list reader `0x00be79d0` ->
//! `0x00be63e0`; 167/1/2 is the client Play sender `0x00be7d80` (verified in
//! IDA for this slice). This file adds no production behaviour.

use osfr_tools::{
    activity_portal::{Catalogue, Viewer},
    character_store::CharacterState,
    encounter::{EncounterEngine, EncounterPhase, MAX_PRIZE_QUANTITY, ZoneTransition},
    gateway::Gateway,
    inventory::{Inventory, MAX_ITEMS},
    startup_world::{NpcSpawn, parse_add_npc},
    world::NpcVisibility,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
use field_battle_fixture::*;

const BRAWLER: i32 = 43;
/// One battle win pays 50 job XP (src/encounter.rs).
const BATTLE_JOB_XP: u32 = 50;
/// `Profiles::rank` caps the job level at 20 (src/profiles.rs) and
/// `set_player_job` clamps 1..=20, so a row above the cap can never be played.
const MAX_JOB_LEVEL: u32 = 20;
/// `EncounterEngine::load` refuses a dungeon row above this coin reward
/// (src/encounter.rs); a battle row becomes a dungeon row.
const ENCOUNTER_REWARD_COINS_MAX: u32 = 1_000_000;
/// 26/42 `MiniGameEndScreenClose` (client sender `0x0092f300`).
const CLOSE_RESULTS: [u8; 4] = [26, 0, 42, 0];

/// Spawners allowed to reuse an open-world content NPC GUID because they *are*
/// that marker: `(spawner GUID, battle id, why)`. `Gameplay::load` adds every
/// battle spawner through `NpcVisibility::add_static_actor`, which removes the
/// NPC with the same GUID, so exactly one actor is drawn; the replaced row must
/// stand at the same authored position (the test checks the shared position, and
/// this table's evidence names the matching model/name row).
const MARKER_REPLACEMENTS: &[(u64, u32, &str)] = &[(
    100_000_004_750,
    12_303,
    "Stone Heart's Hideaway entrance: npc_spawns_extra.json spawn 100000004750 \
     (legacy NPC 4750, model 2056, name 426971) is the same entrance at the same \
     position; battle 12303 reuses its GUID so the marker takes over the entrance \
     actor (the row's 'legacy-data' evidence).",
)];

/// Rows the 167 list intentionally leaves unpublished, with the evidence for
/// each. Every shipped row at this tip is advertised: `Catalogue::load` turns
/// every `battles[]` row into a portal row, so a row listed here must be kept
/// out by a deliberate, reviewed policy (never to hide a broken row).
const GUIDE_ONLY: &[(u32, &str)] = &[];

// ---------------------------------------------------------------------------
// Shared per-row rules.
// ---------------------------------------------------------------------------

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn client_models() -> BTreeSet<i32> {
    fs::read_to_string(lab("working/OSFR.Client/OSFR Client/Resources/Models.txt"))
        .expect("the client model table is readable")
        .lines()
        .filter_map(|line| line.split('^').next()?.trim().parse().ok())
        .collect()
}

fn client_text() -> BTreeMap<i32, String> {
    read_json(&lab("reports/locale-id-to-text.json"))
        .as_array()
        .expect("the locale report is an array")
        .iter()
        .map(|row| {
            (
                row["id"].as_i64().unwrap() as i32,
                row["text"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

/// Load the catalogue exactly as `Gameplay::load` does: the runtime package,
/// then behaviours, the shipped field enemies and the shipped battles. The
/// loader's cross-file GUID rules run in this order and refuse the whole file.
fn loaded_engine(fixture: &Fixture) -> EncounterEngine {
    let mut engine =
        EncounterEngine::load(&fixture.content, None).expect("the runtime encounter package loads");
    engine
        .load_behaviours(&content_dir().join("behaviours.json"))
        .expect("the shipped behaviour presets load");
    engine
        .load_field_enemies(&content_dir().join("field_enemies.json"))
        .expect("the shipped field enemies load");
    engine
        .load_field_battles(&battle_path())
        .expect("the shipped field battle catalogue loads through the production loader");
    engine
}

fn number(value: &Value, field: &str) -> i64 {
    value[field].as_i64().unwrap_or(0)
}

fn unsigned(value: &Value, field: &str) -> u32 {
    value[field].as_u64().map_or(0, |number| number as u32)
}

fn prize_quantity(prize: &Value) -> u32 {
    prize["quantity"]
        .as_u64()
        .map_or(1, |quantity| quantity as u32)
}

fn text_id_violation(
    id: u32,
    field: &str,
    value: i64,
    locale: &BTreeMap<i32, String>,
    out: &mut Vec<String>,
) {
    if value <= 0 {
        out.push(format!(
            "battle {id}: {field} {value} is not a positive client text id"
        ));
    } else if !locale.contains_key(&(value as i32)) {
        out.push(format!(
            "battle {id}: {field} {value} does not resolve in the client locale"
        ));
    }
}

fn actor_violations(
    id: u32,
    label: &str,
    actor: &Value,
    locale: &BTreeMap<i32, String>,
    models: &BTreeSet<i32>,
    out: &mut Vec<String>,
) {
    let guid = number(actor, "guid");
    if guid <= 1 {
        out.push(format!(
            "battle {id}: {label} GUID {guid} is not a usable actor id"
        ));
    }
    let model = number(actor, "model_id");
    if model <= 0 {
        out.push(format!("battle {id}: {label} has no model"));
    } else if !models.contains(&(model as i32)) {
        out.push(format!(
            "battle {id}: {label} model {model} is absent from the client model table"
        ));
    }
    text_id_violation(
        id,
        &format!("{label} name_id"),
        number(actor, "name_id"),
        locale,
        out,
    );
    if number(actor, "definition_id") < 0 {
        out.push(format!("battle {id}: {label} has a negative definition id"));
    }
    let name = actor["name"].as_str().unwrap_or_default();
    if name.is_empty() || name.chars().any(char::is_control) {
        out.push(format!("battle {id}: {label} has no printable name"));
    }
    let position = actor["position"]
        .as_array()
        .map(|points| {
            points
                .iter()
                .map(|value| value.as_f64())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if position.len() != 3
        || !position
            .iter()
            .all(|value| value.is_some_and(|v| v.is_finite() && v.abs() <= 1_000_000.0))
    {
        out.push(format!("battle {id}: {label} position is not finite"));
    }
    let health = number(actor, "max_health");
    if !(1..=100_000).contains(&health) {
        out.push(format!(
            "battle {id}: {label} health {health} is outside 1..=100000"
        ));
    }
    if number(actor, "damage") > 1_000 {
        out.push(format!("battle {id}: {label} damage is above 1000"));
    }
    if let Some(scale) = actor["scale"].as_f64() {
        if !(scale.is_finite() && scale > 0.0) {
            out.push(format!(
                "battle {id}: {label} scale {scale} is not a positive finite value"
            ));
        }
    }
}

/// Every authored rule a shipped row must satisfy, with the row id in each
/// message. The production loader owns the wire/serde bounds and the prize
/// preflight; this mirrors the *content* rules on top of it (positive text ids,
/// the job-level cap, the encounter coin bound, the prize wheel and its slots,
/// the summon/dormant rules and the actor bounds).
fn row_violations(
    row: &Value,
    locale: &BTreeMap<i32, String>,
    models: &BTreeSet<i32>,
) -> Vec<String> {
    let id = unsigned(row, "id");
    let mut out = Vec::new();
    for field in ["title_id", "description_id", "objective_id"] {
        text_id_violation(id, field, number(row, field), locale, &mut out);
    }
    let required = unsigned(row, "required_combat_level");
    if required > MAX_JOB_LEVEL {
        out.push(format!(
            "battle {id}: required_combat_level {required} exceeds the level-{MAX_JOB_LEVEL} job cap"
        ));
    }
    let coins = unsigned(row, "reward_coins");
    if coins > ENCOUNTER_REWARD_COINS_MAX {
        out.push(format!(
            "battle {id}: reward_coins {coins} exceeds the encounter content bound {ENCOUNTER_REWARD_COINS_MAX}"
        ));
    }
    let difficulty = unsigned(row, "difficulty");
    if !(1..=5).contains(&difficulty) {
        out.push(format!(
            "battle {id}: difficulty {difficulty} is outside the loader's 1..=5"
        ));
    }
    match serde_json::from_value::<ZoneTransition>(row["zone"].clone()) {
        Ok(zone) if zone.zone_id == 0 => {
            out.push(format!("battle {id}: arena zone 0 is the open world"));
        }
        Ok(_) => {}
        Err(error) => out.push(format!(
            "battle {id}: zone does not match the encounter ZoneTransition schema: {error}"
        )),
    }

    // The loader caps the wheel at 4 rows but admits an empty one; a shipped
    // battle with no prize is a content defect, so the audit requires one.
    let prizes = row["prizes"].as_array().cloned().unwrap_or_default();
    if prizes.is_empty() || prizes.len() > 4 {
        out.push(format!(
            "battle {id}: the prize wheel has {} rows; the audit requires 1..=4",
            prizes.len()
        ));
    }
    for (index, prize) in prizes.iter().enumerate() {
        for field in ["item_id", "name_id", "icon_id"] {
            if number(prize, field) <= 0 {
                out.push(format!(
                    "battle {id}: prize {} {field} {} is not a positive id",
                    index + 1,
                    number(prize, field)
                ));
            }
        }
        let quantity = prize_quantity(prize);
        if !(1..=MAX_PRIZE_QUANTITY).contains(&quantity) {
            out.push(format!(
                "battle {id}: prize {} quantity {quantity} is outside 1..={MAX_PRIZE_QUANTITY}",
                index + 1
            ));
        }
    }

    actor_violations(id, "marker", &row["spawner"], locale, models, &mut out);
    if unsigned(row, "nameplate_image_id") == 0 {
        out.push(format!(
            "battle {id}: the marker sends no crossed-swords nameplate image"
        ));
    }

    let enemies = row["enemies"].as_array().cloned().unwrap_or_default();
    if enemies.is_empty() || enemies.len() > 16 {
        out.push(format!(
            "battle {id}: the roster has {} enemies; the loader allows 1..=16",
            enemies.len()
        ));
    }
    let goals = enemies
        .iter()
        .filter(|enemy| enemy["summon_below_percent"].is_null())
        .count();
    for (index, enemy) in enemies.iter().enumerate() {
        let label = format!("enemy {}", index + 1);
        actor_violations(id, &label, enemy, locale, models, &mut out);
        let name = enemy["name"].as_str().unwrap_or_default();
        let name_id = number(enemy, "name_id") as i32;
        if locale.get(&name_id).map(String::as_str) != Some(name) {
            out.push(format!(
                "battle {id}: {label} name_id {name_id} does not resolve to {name:?}"
            ));
        }
        if let Some(below) = enemy["summon_below_percent"].as_u64() {
            // The runtime calls a dormant minion in when the fight drops *below*
            // this percent, so 0 never summons and 100 is already crossed.
            if !(1..=99).contains(&below) {
                out.push(format!(
                    "battle {id}: {label} summon_below_percent {below} is outside 1..=99"
                ));
            }
            if enemy["boss"].as_bool().unwrap_or(false) {
                out.push(format!(
                    "battle {id}: {label} is a summoned minion and cannot also be a boss"
                ));
            }
            if enemy["skittish"].as_bool().unwrap_or(false) {
                out.push(format!(
                    "battle {id}: {label} is a summoned minion and cannot be the skittish summoner"
                ));
            }
        }
    }
    if !enemies.is_empty() && goals == 0 {
        out.push(format!(
            "battle {id}: every enemy is a summoned minion, so the roster has no summoner and no goal"
        ));
    }
    out
}

// ---------------------------------------------------------------------------
// 1. Every row loads with one unique drawable marker at its authored position.
// ---------------------------------------------------------------------------

#[test]
fn every_shipped_field_battle_loads_with_a_unique_drawable_marker() {
    let fixture = Fixture::new();
    let file = battles();
    let rows = file["battles"]
        .as_array()
        .expect("the shipped battle list is an array")
        .clone();
    assert!(!rows.is_empty(), "the shipped catalogue has a row");
    let locale = client_text();
    let models = client_models();

    // Per-row loader attribution: a single-row file makes the loader's refusal
    // name the row even though the production load reads the whole catalogue.
    for row in &rows {
        let id = unsigned(row, "id");
        let path = fixture.root.join(format!("audit-row-{id}.json"));
        fs::write(
            &path,
            serde_json::to_vec(&json!({"schema_version": 1, "battles": [row]})).unwrap(),
        )
        .expect("the scratch row file is writable");
        let mut probe =
            EncounterEngine::load(&fixture.content, None).expect("the runtime package loads");
        probe.load_field_battles(&path).unwrap_or_else(|error| {
            panic!("battle {id} is refused by the production loader: {error}")
        });
    }

    // The whole catalogue in production order: field enemies first, then the
    // battles, so the loader's cross-file GUID rules run for every actor.
    let engine = loaded_engine(&fixture);
    let spawners = engine.battle_spawners();
    assert_eq!(
        spawners.len(),
        rows.len(),
        "every shipped row publishes exactly one marker"
    );
    let field_guids: BTreeSet<u64> = engine.field_guids().into_iter().collect();

    let mut violations: Vec<String> = Vec::new();
    for row in &rows {
        violations.extend(row_violations(row, &locale, &models));
    }
    assert!(
        violations.is_empty(),
        "shipped field battle rows violate the catalogue rules:\n{}",
        violations.join("\n")
    );

    // Cross-row GUID uniqueness. The loader refuses a repeat; this names both
    // rows.
    let mut guids: BTreeMap<u64, u32> = BTreeMap::new();
    for row in &rows {
        let id = unsigned(row, "id");
        for actor in std::iter::once(&row["spawner"]).chain(
            row["enemies"]
                .as_array()
                .expect("the roster is an array")
                .iter(),
        ) {
            let guid = u64_field(actor, "guid");
            if let Some(other) = guids.insert(guid, id) {
                violations.push(format!(
                    "battles {other} and {id} both use actor GUID {guid}"
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "the shipped catalogue repeats an actor GUID:\n{}",
        violations.join("\n")
    );

    // The marker must report the authored fields through the engine accessor,
    // and the portal row that starts it must carry the authored gates.
    let catalogue = Catalogue::load(&battle_path()).expect("the shipped portal catalogue loads");
    for row in &rows {
        let id = unsigned(row, "id");
        let spawner = &row["spawner"];
        let guid = u64_field(spawner, "guid");
        let loaded = spawners
            .iter()
            .find(|entry| entry.0 == guid)
            .unwrap_or_else(|| panic!("battle {id}: battle_spawners has no marker {guid}"));
        assert_eq!(
            loaded.1,
            spawner["name"].as_str().unwrap(),
            "battle {id}: the marker tool name"
        );
        assert_eq!(
            loaded.2,
            number(spawner, "name_id") as i32,
            "battle {id}: the marker client name id"
        );
        assert_eq!(
            loaded.3,
            number(spawner, "model_id") as i32,
            "battle {id}: the marker model"
        );
        assert_eq!(
            loaded.4,
            position(&spawner["position"]),
            "battle {id}: the marker position"
        );
        assert_eq!(
            loaded.5,
            number(row, "nameplate_image_id") as i32,
            "battle {id}: the crossed-swords nameplate"
        );
        assert!(loaded.5 > 0, "battle {id}: the marker is not drawable");
        assert!(
            engine.is_battle_spawner(guid),
            "battle {id}: the marker is not the engine's battle spawner"
        );
        assert!(
            !field_guids.contains(&guid),
            "battle {id}: marker {guid} aliases a field enemy"
        );
        let activity = catalogue
            .activities()
            .iter()
            .find(|activity| activity.id == id)
            .unwrap_or_else(|| panic!("battle {id} is missing from the portal catalogue"));
        assert_eq!(
            activity.difficulty,
            unsigned(row, "difficulty"),
            "battle {id}: the portal difficulty differs from the authored row"
        );
        assert_eq!(
            activity.required_level,
            unsigned(row, "required_combat_level"),
            "battle {id}: the portal level gate differs from the authored row"
        );
    }

    // The open-world visibility set `Gameplay::load` builds: NPC content, field
    // enemies removed, then every battle spawner added through
    // `add_static_actor` (which replaces an NPC with the same GUID).
    let mut visibility =
        NpcVisibility::load(&content_dir()).expect("the shipped NPC content loads");
    visibility.exclude_actor_guids(field_guids.iter().copied());
    let npc_positions: BTreeMap<u64, Vec<[f32; 3]>> = {
        let mut positions: BTreeMap<u64, Vec<[f32; 3]>> = BTreeMap::new();
        for target in visibility.teleport_targets(0) {
            positions
                .entry(target.guid)
                .or_default()
                .push(target.position);
        }
        positions
    };
    for (guid, _battle, reason) in MARKER_REPLACEMENTS {
        assert!(
            !reason.trim().is_empty(),
            "a marker replacement needs recorded evidence"
        );
        assert!(
            npc_positions.contains_key(guid),
            "documented NPC replacement {guid} is no longer a loaded NPC"
        );
    }
    let mut collisions: Vec<(u64, u32)> = Vec::new();
    for row in &rows {
        let guid = u64_field(&row["spawner"], "guid");
        if npc_positions.contains_key(&guid) {
            collisions.push((guid, unsigned(row, "id")));
        }
    }
    let mut documented: Vec<(u64, u32)> = MARKER_REPLACEMENTS
        .iter()
        .map(|(guid, battle, _)| (*guid, *battle))
        .collect();
    collisions.sort();
    documented.sort();
    assert_eq!(
        collisions, documented,
        "a spawner may reuse an open-world NPC GUID only for a documented same-marker replacement"
    );
    for (guid, battle_id, _) in MARKER_REPLACEMENTS {
        let row = battle(&file, *battle_id);
        let marker = position(&row["spawner"]["position"]);
        let replaced = &npc_positions[guid];
        assert!(
            replaced.iter().any(|npc| {
                (npc[0] - marker[0]).abs() < 1e-4
                    && (npc[1] - marker[1]).abs() < 1e-4
                    && (npc[2] - marker[2]).abs() < 1e-4
            }),
            "battle {battle_id} replaces NPC {guid} but the two are not the same marker"
        );
    }
    for row in &rows {
        let spawner = &row["spawner"];
        visibility.add_static_actor(
            u64_field(spawner, "guid"),
            spawner["name"].as_str().unwrap(),
            number(spawner, "name_id") as i32,
            number(spawner, "model_id") as i32,
            position(&spawner["position"]),
            number(row, "nameplate_image_id") as i32,
        );
    }
    let targets = visibility.teleport_targets(0);
    for row in &rows {
        let id = unsigned(row, "id");
        let guid = u64_field(&row["spawner"], "guid");
        let marker = position(&row["spawner"]["position"]);
        let matches: Vec<_> = targets
            .iter()
            .filter(|target| target.guid == guid)
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "battle {id}: marker {guid} must be exactly one open-world actor"
        );
        assert_eq!(
            matches[0].position, marker,
            "battle {id}: the marker is not visible at its authored position"
        );
    }

    // The client's 35/2 AddNpc (reader `0x00921a10`): the marker is drawn in
    // open-world zone 0 with the authored model, name id and crossed-swords
    // icon.
    for row in &rows {
        let id = unsigned(row, "id");
        let guid = u64_field(&row["spawner"], "guid");
        let marker = position(&row["spawner"]["position"]);
        // Reset the view so a marker within 100 m of the previous one cannot be
        // skipped as already visible; no shipped marker is near the origin.
        visibility.update(0, [0.0, 0.0, 0.0]).unwrap();
        let packets = visibility.update(0, marker).unwrap();
        let packet = packets
            .iter()
            .find(|packet| {
                packet.starts_with(&[35, 0, 2, 0])
                    && packet.len() >= 12
                    && u64::from_le_bytes(packet[4..12].try_into().unwrap()) == guid
            })
            .unwrap_or_else(|| {
                panic!("battle {id}: marker {guid} was not sent as 35/2 AddNpc at its position")
            });
        let spawn: NpcSpawn = parse_add_npc(packet).expect("the 35/2 AddNpc body parses");
        assert_eq!(spawn.actor_id, guid);
        assert_eq!(
            spawn.name_string_id,
            number(&row["spawner"], "name_id") as i32,
            "battle {id}: the marker's client name id"
        );
        assert_eq!(
            spawn.model_id,
            number(&row["spawner"], "model_id") as i32,
            "battle {id}: the marker's drawable model"
        );
        assert_eq!(
            spawn.nameplate_image_id,
            number(row, "nameplate_image_id") as i32,
            "battle {id}: the crossed-swords nameplate"
        );
        assert_eq!(
            spawn.position,
            [marker[0], marker[1], marker[2], 1.0],
            "battle {id}: AddNpc carries the authored position"
        );
        // add_static_actor leaves the literal tool name empty: the client
        // nameplate has to come from NameStringId (see battle 12400's note).
        assert_eq!(
            spawn.npc_name, "",
            "battle {id}: the marker has no tool name"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Every roster resolves through the production loader (the arena state).
// ---------------------------------------------------------------------------

#[test]
fn every_battle_has_a_valid_non_empty_roster() {
    let fixture = Fixture::new();
    let file = battles();
    let rows = file["battles"].as_array().unwrap().clone();
    let engine = loaded_engine(&fixture);
    let field_guids: BTreeSet<u64> = engine.field_guids().into_iter().collect();
    let mut visibility =
        NpcVisibility::load(&content_dir()).expect("the shipped NPC content loads");
    visibility.exclude_actor_guids(field_guids.iter().copied());
    let npc_guids: BTreeSet<u64> = visibility
        .teleport_targets(0)
        .into_iter()
        .map(|target| target.guid)
        .collect();

    let mut enemy_guids = BTreeSet::new();
    // The authored roster rules (bounds, text ids, summon/dormant agreement)
    // are this test's contract; then the production loader must resolve the
    // same roster into the arena state.
    let locale = client_text();
    let models = client_models();
    let mut violations: Vec<String> = rows
        .iter()
        .flat_map(|row| row_violations(row, &locale, &models))
        .collect();
    assert!(
        violations.is_empty(),
        "shipped field battle rosters violate the catalogue rules:\n{}",
        violations.join("\n")
    );
    for row in &rows {
        let id = unsigned(row, "id");
        let enemies = row["enemies"].as_array().unwrap();
        let mut engine = engine.clone();
        engine.set_player_job(BRAWLER, MAX_JOB_LEVEL);
        let character = CharacterState {
            guid: 9_000_000 + u64::from(id),
            zone_id: 0,
            position: position(&row["spawner"]["position"]),
            rotation: [0.0, 0.0, 1.0],
        };
        engine
            .begin(id, character, BRAWLER)
            .unwrap_or_else(|error| panic!("battle {id} cannot begin: {error}"));
        let zone: ZoneTransition = serde_json::from_value(row["zone"].clone())
            .expect("the authored zone matches ZoneTransition");
        assert_eq!(
            engine.zone().as_ref(),
            Some(&zone),
            "battle {id}: the engine zones into the authored arena"
        );
        assert!(
            engine.in_battle_arena(),
            "battle {id}: begin did not start a battle arena"
        );
        let active = engine
            .state()
            .active
            .as_ref()
            .unwrap_or_else(|| panic!("battle {id}: begin leaves no active encounter"));
        assert_eq!(active.encounter_id, id, "battle {id}: wrong encounter");
        assert_eq!(
            active.enemies.len(),
            enemies.len(),
            "battle {id}: the loaded roster size differs from the authored one"
        );
        let mut ratios = Vec::new();
        for (index, (authored, state)) in enemies.iter().zip(&active.enemies).enumerate() {
            let guid = u64_field(authored, "guid");
            if !enemy_guids.insert(guid) {
                violations.push(format!(
                    "battle {id}: enemy {guid} repeats across the catalogue"
                ));
            }
            if field_guids.contains(&guid) {
                violations.push(format!("battle {id}: enemy {guid} aliases a field enemy"));
            }
            if npc_guids.contains(&guid) {
                violations.push(format!(
                    "battle {id}: enemy {guid} aliases an open-world NPC"
                ));
            }
            assert_eq!(state.guid, guid, "battle {id}: roster order changed");
            assert_eq!(
                state.position,
                position(&authored["position"]),
                "battle {id}: enemy {} loaded at the wrong position",
                index + 1
            );
            assert_eq!(
                state.dormant,
                !authored["summon_below_percent"].is_null(),
                "battle {id}: enemy {} summon/dormant disagreement",
                index + 1
            );
            let authored_health = u32_field(authored, "max_health");
            // The loader scales every arena actor by the combat scale and the
            // difficulty's level factor (R). Difficulty 1 has factor x1, so the
            // scaled health is exact there; above it, every actor uses the same
            // factor.
            let scaled = combat_scale::enemy_health(authored_health);
            assert!(
                state.health >= scaled,
                "battle {id}: enemy {} scaled below its combat-scale health",
                index + 1
            );
            ratios.push(f64::from(state.health) / f64::from(scaled));
        }
        if u32_field(row, "difficulty") == 1 {
            assert!(
                ratios.iter().all(|ratio| (ratio - 1.0).abs() < 1e-6),
                "battle {id}: difficulty 1 must not scale the authored roster"
            );
        } else if let (Some(min), Some(max)) = (
            ratios.iter().copied().reduce(f64::min),
            ratios.iter().copied().reduce(f64::max),
        ) {
            assert!(
                max - min < 0.02,
                "battle {id}: enemies were scaled by different factors ({min}..{max})"
            );
        }
    }
    assert!(
        violations.is_empty(),
        "shipped roster GUID collisions:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 3. Every prize and coin reward resolves; a full bag is refused.
// ---------------------------------------------------------------------------

#[test]
fn every_battle_prize_and_coin_reward_resolves() {
    let fixture = Fixture::new();
    let file = battles();
    let rows = file["battles"].as_array().unwrap().clone();
    let engine = loaded_engine(&fixture);
    let inventory = runtime_inventory();
    let mut violations: Vec<String> = Vec::new();
    for row in &rows {
        let id = unsigned(row, "id");
        let prizes = row["prizes"].as_array().unwrap();
        for (index, prize) in prizes.iter().enumerate() {
            let item = number(prize, "item_id") as i32;
            let quantity = prize_quantity(prize);
            if !inventory.has_definition(item) {
                violations.push(format!(
                    "battle {id}: prize {} item {item} is absent from the merged item catalogue",
                    index + 1
                ));
            } else if !inventory.accepts_reward(item, quantity) {
                violations.push(format!(
                    "battle {id}: prize {} item {item} x{quantity} cannot be granted by a bag with room",
                    index + 1
                ));
            }
        }
        if !engine.battle_prizes_available(id, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }) {
            violations.push(format!(
                "battle {id}: the production prize preflight refuses the shipped preview bundle"
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "shipped field battle prizes do not resolve:\n{}",
        violations.join("\n")
    );

    // The same preflight through the portal Play on a full bag: the whole
    // bundle is refused before the encounter, bag, coins or save can change.
    let row = battle(&file, 12_350);
    let marker = position(&row["spawner"]["position"]);
    let (mut gateway, bag) =
        fixture.gateway_with_bag_seeded(marker, 7_301, MAX_ITEMS, 12_350, |session| {
            send(session, &switch_job(BRAWLER), 250);
        });
    assert_eq!(bag.items.len(), MAX_ITEMS, "the fixture bag is full");
    let bundle: Vec<(i32, u32)> = row["prizes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|prize| (number(prize, "item_id") as i32, prize_quantity(prize)))
        .collect();
    assert!(
        !Inventory::load(&fixture.content, Some(bag))
            .unwrap()
            .accepts_reward_bundle(&bundle),
        "the full bag must have no room for the previewed bundle"
    );
    let before = gateway.snapshot();
    let refused = send(&mut gateway, &portal_play(12_350), 500);
    assert!(!refused.began_zoning, "a refused Play zones nowhere");
    assert!(!refused.state_changed, "a refused Play changes nothing");
    assert!(
        chat_text(&refused).contains("one of its rewards cannot be granted"),
        "{}",
        chat_text(&refused)
    );
    assert!(
        details(&refused).is_none(),
        "a refused Play opens no start screen"
    );
    let encounter = gateway
        .snapshot()
        .gameplay
        .expect("the session has gameplay")
        .encounter
        .expect("the session has the encounter engine");
    assert!(encounter.active.is_none(), "no encounter began");
    assert_eq!(
        gateway.snapshot(),
        before,
        "a full-bag refusal changes no state"
    );
}

// ---------------------------------------------------------------------------
// 4. Every row is published on the activity portal (or documented guide-only).
// ---------------------------------------------------------------------------

#[test]
fn every_battle_is_published_or_documented_guide_only() {
    let fixture = Fixture::new();
    let file = battles();
    let rows = file["battles"].as_array().unwrap().clone();
    let marker = position(&battle(&file, 12_350)["spawner"]["position"]);
    let (_gateway, loaded) = fixture.session(marker, 7_401);
    let list = loaded
        .replies
        .iter()
        .map(|packet| unframe(packet))
        .find(|packet| packet.starts_with(&[167, 0, 1, 1]))
        .expect("world entry sends the 167/1/1 activity list");
    let published: BTreeMap<u32, PortalRow> = decode_list(list)
        .into_iter()
        .map(|row| (row.id, row))
        .collect();
    let catalogue = Catalogue::load(&battle_path()).expect("the shipped portal catalogue loads");

    let mut violations: Vec<String> = Vec::new();
    for row in &rows {
        let id = unsigned(row, "id");
        match published.get(&id) {
            Some(portal) => {
                if portal.title != unsigned(row, "title_id")
                    || portal.description != unsigned(row, "description_id")
                    || portal.difficulty != unsigned(row, "difficulty")
                {
                    violations.push(format!(
                        "battle {id}: the 167 list row differs from the authored title/description/difficulty"
                    ));
                }
            }
            None if !GUIDE_ONLY.iter().any(|(guide, _)| *guide == id) => violations.push(format!(
                "battle {id} is not published by 167/1/1 and is not a documented guide-only row"
            )),
            None => {}
        }
        if let Err(refusal) = catalogue.admit(
            id,
            Viewer {
                combat_job: true,
                level: MAX_JOB_LEVEL,
            },
            false,
        ) {
            violations.push(format!(
                "battle {id} is not playable from the portal for a qualified player: {refusal:?}"
            ));
        }
    }
    // `GUIDE_ONLY` must not hide a row that is in fact published, and every
    // entry needs a reason.
    for (id, reason) in GUIDE_ONLY {
        assert!(
            !reason.trim().is_empty(),
            "guide-only battle {id} needs recorded evidence"
        );
        assert!(
            !published.contains_key(id),
            "guide-only battle {id} is actually published"
        );
        assert!(
            rows.iter().any(|row| unsigned(row, "id") == *id),
            "guide-only battle {id} is not a shipped row"
        );
    }
    assert!(
        violations.is_empty(),
        "shipped field battles are not published:\n{}",
        violations.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 5. Marker + start-screen routes: start, win, pay once, return, replay inert.
// ---------------------------------------------------------------------------

#[test]
fn one_marker_battle_and_one_guide_battle_start_win_pay_once_and_return() {
    let fixture = Fixture::new();
    let file = battles();
    let marker_row = battle(&file, 12_350);
    let guide_row = battle(&file, 96);
    let marker_prizes = prize_ids(marker_row);
    let guide_prizes = prize_ids(guide_row);

    // Two independent in-process sessions (two players at their own markers).
    let mut marker = fixture.gateway(position(&marker_row["spawner"]["position"]), 7_501);
    let mut guide = fixture.gateway(position(&guide_row["spawner"]["position"]), 7_502);
    assert!(send(&mut marker, &switch_job(BRAWLER), 200).state_changed);
    assert!(send(&mut guide, &switch_job(BRAWLER), 200).state_changed);
    let marker_before = economy(&marker, &marker_prizes);
    let guide_before = economy(&guide, &guide_prizes);

    let marker_screen = open_battle(
        &mut marker,
        u64_field(&marker_row["spawner"], "guid"),
        marker_row,
        300,
    );
    let guide_screen = open_battle(
        &mut guide,
        u64_field(&guide_row["spawner"], "guid"),
        guide_row,
        300,
    );
    let marker_guids = enter_battle(&mut marker, &marker_screen, marker_row, 400);

    // The guide battle's six summoned minions are dormant at the arena load:
    // only the Troll Summoner (the goal and the summoner) is spawned there.
    let entered = send(&mut guide, &go(&guide_screen), 400);
    assert!(entered.began_zoning, "Go enters the guide battle arena");
    assert_eq!(
        guide.state().zone_id,
        u32_field(&guide_row["zone"], "zone_id")
    );
    let ready = arena_spawned(&mut guide, 500);
    let summoned: BTreeSet<u64> = guide_row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| !enemy["summon_below_percent"].is_null())
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let boss = guide_row["enemies"][0].clone();
    assert!(
        ready
            .replies
            .iter()
            .map(|packet| unframe(packet))
            .any(|packet| packet.starts_with(&[35, 0, 2, 0])
                && packet[4..12] == u64_field(&boss, "guid").to_le_bytes()),
        "the summoner spawns at arena load"
    );
    let guide_active = guide
        .snapshot()
        .gameplay
        .unwrap()
        .encounter
        .unwrap()
        .active
        .expect("the guide battle runs");
    assert_eq!(guide_active.phase, EncounterPhase::Fighting);
    assert_eq!(
        guide_active.enemies.len(),
        guide_row["enemies"].as_array().unwrap().len()
    );
    for (authored, state) in guide_row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&guide_active.enemies)
    {
        assert_eq!(
            state.dormant,
            summoned.contains(&u64_field(authored, "guid")),
            "the summon threshold and the loaded dormant flag agree"
        );
    }

    let mut now = 2_000;
    // Win the marker battle first; the other player's run must be untouched.
    let marker_replies = fight_to_win(&mut marker, &mut now, marker_row);
    assert_eq!(results_lifecycle(&marker_replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&marker_replies), Some(1));
    let marker_paid = economy(&marker, &marker_prizes);
    assert_payout(
        &marker_before,
        &marker_paid,
        marker_row,
        &marker_prizes,
        BATTLE_JOB_XP,
    );
    assert_eq!(
        economy(&guide, &guide_prizes),
        guide_before,
        "the other player's wallet, bag and XP are untouched"
    );
    assert_eq!(
        guide
            .snapshot()
            .gameplay
            .unwrap()
            .encounter
            .unwrap()
            .active
            .map(|active| active.phase),
        Some(EncounterPhase::Fighting),
        "the other player's battle is still running"
    );

    // A replayed attack on the won marker fight pays nothing.
    let won = marker.snapshot();
    let repeated = send(&mut marker, &melee(marker_guids[0]), now + 600);
    assert!(!repeated.state_changed, "a won fight cannot pay twice");
    assert_eq!(marker.snapshot(), won);

    // Close the marker results: the pre-entry pose/zone return, once.
    let closed = send(&mut marker, &CLOSE_RESULTS, now + 900);
    assert!(closed.began_zoning, "Close returns to the open world");
    assert_returned(&marker, position(&marker_row["spawner"]["position"]));
    assert_eq!(economy(&marker, &marker_prizes), marker_paid);
    let after_close = marker.snapshot();
    let replay = send(&mut marker, &CLOSE_RESULTS, now + 1_200);
    assert!(!replay.began_zoning && !replay.state_changed);
    let replay_go = send(&mut marker, &go(&marker_screen), now + 1_400);
    assert!(!replay_go.began_zoning && !replay_go.state_changed);
    assert_eq!(
        marker.snapshot(),
        after_close,
        "a replayed Close or Go pays nothing"
    );

    // The start-screen battle wins through the same server-observed damage.
    let mut guide_now = 2_000;
    let guide_replies = fight_summoner(&mut guide, &mut guide_now, guide_row);
    assert_eq!(results_lifecycle(&guide_replies), vec![18, 47, 19]);
    assert_eq!(won_flag(&guide_replies), Some(1));
    let guide_paid = economy(&guide, &guide_prizes);
    assert_payout(
        &guide_before,
        &guide_paid,
        guide_row,
        &guide_prizes,
        BATTLE_JOB_XP,
    );
    let guide_closed = send(&mut guide, &CLOSE_RESULTS, guide_now + 900);
    assert!(guide_closed.began_zoning, "Close returns to the open world");
    assert_returned(&guide, position(&guide_row["spawner"]["position"]));
    assert_eq!(economy(&guide, &guide_prizes), guide_paid);
}

/// The Troll Summoner is shielded while a summoned minion stands
/// (`src/encounter/skills.rs::shielded`), so the arena route has to clear each
/// wave before it can damage the summoner. `fight_to_win` targets the first
/// living actor, which suits the marker battles; this is the same 36/10 attack
/// path with the summoner's own target rule.
fn fight_summoner(gateway: &mut Gateway, now_ms: &mut u64, row: &Value) -> Vec<Vec<u8>> {
    let player = gateway.state().guid;
    let arrival = position(&row["zone"]["position"]);
    let summons: BTreeSet<u64> = row["enemies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|enemy| !enemy["summon_below_percent"].is_null())
        .map(|enemy| u64_field(enemy, "guid"))
        .collect();
    let mut replies = Vec::new();
    for _ in 0..400 {
        let Some(active) = arena_active(gateway) else {
            break;
        };
        if active.phase != EncounterPhase::Fighting {
            break;
        }
        let standing: Vec<_> = active
            .enemies
            .iter()
            .filter(|enemy| enemy.health > 0 && !enemy.dormant)
            .collect();
        let Some(target) = standing
            .iter()
            .find(|enemy| summons.contains(&enemy.guid))
            .copied()
            .or_else(|| standing.first().copied())
        else {
            break;
        };
        walk(
            gateway,
            player,
            [
                target.position[0] + 1.5,
                target.position[1],
                target.position[2],
            ],
            now_ms,
        );
        *now_ms += 700;
        let hit = send(gateway, &melee(target.guid), *now_ms);
        replies.extend(hit.replies);
        walk(gateway, player, arrival, now_ms);
        *now_ms += 100;
        replies.extend(gateway.tick_at(*now_ms).unwrap().replies);
    }
    assert_eq!(
        arena_active(gateway).map(|active| active.phase),
        Some(EncounterPhase::Won),
        "the summoner waves die through the real attack path"
    );
    replies
}

fn arena_active(gateway: &Gateway) -> Option<osfr_tools::encounter::ActiveEncounter> {
    gateway
        .snapshot()
        .gameplay
        .and_then(|gameplay| gameplay.encounter)
        .and_then(|encounter| encounter.active)
}

// ---------------------------------------------------------------------------
// 6. A malformed row is refused without replacing the loaded catalogue.
// ---------------------------------------------------------------------------

#[test]
fn a_malformed_battle_row_is_refused_atomically() {
    let fixture = Fixture::new();
    let shipped = fs::read(battle_path()).expect("the shipped battle file is readable");
    let shipped_hash = sha256(&shipped);
    let mut engine = loaded_engine(&fixture);
    let spawners = engine.battle_spawners();
    let state = engine.state().clone();
    let inventory = runtime_inventory();
    let prizes_before = engine.battle_prizes_available(96, |item, quantity| {
        inventory.accepts_reward(item, quantity)
    });
    let scratch = fixture.root.join("field_battles.json");
    let write = |doc: &Value| {
        fs::write(&scratch, serde_json::to_vec(doc).unwrap())
            .expect("the scratch copy is writable");
        scratch.clone()
    };

    // Duplicate GUID: the row reuses another row's spawner GUID.
    let mut doc: Value = serde_json::from_slice(&shipped).unwrap();
    let duplicated = u64_field(&battle(&doc, 12_301)["spawner"], "guid");
    doc["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == 96)
        .unwrap()["spawner"]["guid"] = json!(duplicated);
    let path = write(&doc);
    let error = engine.reload_field_battles(&path).unwrap_err().to_string();
    assert!(error.contains("duplicate field battle GUID"), "{error}");

    // Missing enemy: an empty roster.
    let mut doc: Value = serde_json::from_slice(&shipped).unwrap();
    doc["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == 12_370)
        .unwrap()["enemies"] = json!([]);
    let path = write(&doc);
    let error = engine.reload_field_battles(&path).unwrap_err().to_string();
    assert!(error.contains("invalid field battle"), "{error}");

    // Level out of bounds: a difficulty the loader does not admit.
    let mut doc: Value = serde_json::from_slice(&shipped).unwrap();
    doc["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == 12_303)
        .unwrap()["difficulty"] = json!(6);
    let path = write(&doc);
    let error = engine.reload_field_battles(&path).unwrap_err().to_string();
    assert!(error.contains("invalid field battle"), "{error}");

    // Every refused reload leaves the previously loaded battles, encounter
    // state and prize preflight exactly as they were.
    assert_eq!(
        engine.battle_spawners(),
        spawners,
        "a refusal replaced a battle row"
    );
    assert_eq!(engine.state(), &state, "a refusal changed encounter state");
    assert_eq!(
        engine.battle_prizes_available(96, |item, quantity| inventory
            .accepts_reward(item, quantity)),
        prizes_before,
        "a refusal changed the prize preflight"
    );

    // Unknown prize: the loader owns no item catalogue, so the production
    // preflight is the refusal layer (T243). The row loads and the preflight
    // refuses it before any encounter or payout can start.
    let mut doc: Value = serde_json::from_slice(&shipped).unwrap();
    doc["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == 12_350)
        .unwrap()["prizes"][0]["item_id"] = json!(i32::MAX);
    let path = write(&doc);
    engine
        .reload_field_battles(&path)
        .expect("the loader accepts a well-formed unknown prize id");
    assert!(
        !engine.battle_prizes_available(12_350, |item, quantity| {
            inventory.accepts_reward(item, quantity)
        }),
        "the production preflight must refuse an unknown prize definition"
    );

    // Zero title: not a loader rule (the loader only bounds the wire shape), so
    // the catalogue gate itself refuses it. It must name the row and field.
    let mut doc: Value = serde_json::from_slice(&shipped).unwrap();
    doc["battles"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == 12_400)
        .unwrap()["title_id"] = json!(0);
    let path = write(&doc);
    engine
        .reload_field_battles(&path)
        .expect("the loader does not validate the title id");
    let violations = row_violations(&battle(&doc, 12_400), &client_text(), &client_models());
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("battle 12400") && violation.contains("title_id")),
        "the audit must name the zero title: {violations:?}"
    );

    // Restore the scratch copy byte-identically, prove the shipped file was
    // never touched, and prove the restored catalogue loads again.
    fs::write(&scratch, &shipped).expect("the scratch copy is restorable");
    assert_eq!(sha256(&fs::read(&scratch).unwrap()), shipped_hash);
    assert_eq!(sha256(&fs::read(battle_path()).unwrap()), shipped_hash);
    engine
        .reload_field_battles(&scratch)
        .expect("the restored catalogue loads again");
    assert_eq!(
        engine.battle_spawners(),
        spawners,
        "the restored catalogue is the shipped one"
    );
    assert_eq!(engine.state(), &state);
}

// ---------------------------------------------------------------------------
// 7. The audit runs on the shipped content and sends no developer command.
// ---------------------------------------------------------------------------

#[test]
fn the_audit_uses_the_shipped_content_and_no_dev_commands() {
    let shipped = fs::read(battle_path()).expect("the shipped battle file is readable");
    let file: Value = serde_json::from_slice(&shipped).expect("the shipped file is JSON");
    let rows = file["battles"].as_array().unwrap().clone();
    assert!(!rows.is_empty());
    println!(
        "field battles: {} rows, sha256 {}",
        rows.len(),
        sha256(&shipped)
    );

    let fixture = Fixture::new();
    let engine = loaded_engine(&fixture);
    assert_eq!(
        engine.battle_spawners().len(),
        rows.len(),
        "the engine loads exactly the shipped rows"
    );

    // The route's packet builders are ordinary client messages. A realm order
    // or the developer teleport would ride the local-chat family (15/1); none
    // of the route helpers can carry one.
    let route: [(&str, Vec<u8>); 6] = [
        ("interact 26/8", interact(0)),
        ("go 41/108", go(&[0_u8; 71])),
        ("attack 36/10", melee(0)),
        ("move 125", move_to(0, [0.0; 3])),
        ("results close 26/42", CLOSE_RESULTS.to_vec()),
        ("portal play 167/1/2", portal_play(0)),
    ];
    for (label, packet) in route {
        assert_ne!(
            &packet[..2],
            &[15, 0][..],
            "{label} must not be a local-chat/developer command"
        );
    }
}
