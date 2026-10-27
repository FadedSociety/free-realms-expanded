//! Scratch split-content tree for the enabled quest catalogue audit.
//!
//! Test-only: the fixture is built under `CARGO_TARGET_TMPDIR` and removed on
//! drop, so the shipped tree is never touched. The audit loads it through the
//! production `QuestEngine::from_split_content`, `world::NpcVisibility::load`,
//! `Inventory::load`, `RecipeCatalog::load_runtime` and
//! `activity_portal::Catalogue::load`; this module only writes the minimal
//! authored rows those loaders need, one resolving row per graded kind.

#[path = "../common/item_catalogue.rs"]
mod item_catalogue;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const FIXTURE_DEFECT: u32 = 9_100_002;
pub const FIXTURE_QUEST: u32 = 9_100_001;
pub const FIXTURE_ITEM: i32 = 9_100_100;
pub const FIXTURE_RECIPE: u32 = 9_100_101;
/// The recipe row's `minigame_id`: the game id a craft win sends as 39/18.
pub const FIXTURE_MINIGAME: u32 = 9_100_102;
pub const FIXTURE_BATTLE: u32 = 9_100_200;
/// The documented local opening drop key (`Gameplay::local_gather_drop`).
pub const FIXTURE_GATHER_RESOURCE: u32 = 1_001;
/// T503: the node type `quest_nodes.json` places for the graded `collect_node` goal.
pub const FIXTURE_NODE_TYPE: &str = "fixture-node";
/// T788: a bundled `pets.json` trick id (11, the dog's "Sit Up"), used as a
/// `perform_pet_trick` target the reachability gate resolves.
pub const FIXTURE_PET_TRICK: u32 = 11;
const FIXTURE_GIVER: u64 = 9_000_100_001;
const FIXTURE_TARGET: u64 = 9_000_100_002;
const FIXTURE_MISSING_GIVER: u64 = 9_000_100_099;
const FIXTURE_MISSING_PREREQUISITE: u32 = 4_000_100_098;
const FIXTURE_BLOCKER: u32 = 9_100_003;
const FIXTURE_TITLE: u32 = 700_001;

#[derive(Clone, Copy)]
pub enum Defect {
    None,
    MissingGiver,
    EmptyReward,
    DisabledPrerequisite,
    MissingPrerequisite,
    WrappedTitle,
    /// A `visit_position` objective with the given transform, so the production
    /// split loader's geometry bound can be exercised directly.
    VisitGeometry {
        position: [f64; 3],
        radius: f64,
    },
}

pub struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t314-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("quests")).expect("fixture quests dir");
        fs::create_dir_all(dir.join("npcs")).expect("fixture npcs dir");
        Self(dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture_source() -> Value {
    json!([{"kind": "reconstructed", "evidence": "(R) T314 scratch fixture"}])
}

fn fixture_quest(
    id: u32,
    title_string_id: u32,
    giver_npc_id: u64,
    objectives: Value,
    rewards: Value,
    prerequisite_quest_ids: Value,
) -> Value {
    json!({
        "id": id,
        "title_string_id": title_string_id,
        "giver_npc_id": giver_npc_id,
        "prerequisite_quest_ids": prerequisite_quest_ids,
        "objectives": objectives,
        "rewards": rewards,
        "source": fixture_source(),
    })
}

fn fixture_talk_objective() -> Value {
    json!([{
        "kind": "talk_to_npc",
        "id": 1,
        "npc_id": FIXTURE_TARGET,
        "source": fixture_source(),
    }])
}

fn fixture_visit_objective(position: [f64; 3], radius: f64) -> Value {
    json!([{
        "kind": "visit_position",
        "id": 1,
        "zone_id": 1,
        "position": position,
        "radius": radius,
        "source": fixture_source(),
    }])
}

/// One resolving target per graded kind: the item catalogue, recipe book, the
/// recipe's `minigame_id`, battle set and local gather-drop list this fixture
/// writes. The `finish_minigame` target is the recipe's game id so the craft
/// producer is exercised through the production `RecipeCatalog` loader.
fn fixture_objectives() -> Value {
    let mut objectives = fixture_talk_objective();
    objectives
        .as_array_mut()
        .expect("the talk fixture is an objective array")
        .extend([
            json!({
                "kind": "collect_item",
                "id": 2,
                "item_id": FIXTURE_ITEM,
                "quantity": 1,
                "source": fixture_source(),
            }),
            json!({
                "kind": "equip_item",
                "id": 3,
                "definition_id": FIXTURE_ITEM,
                "source": fixture_source(),
            }),
            json!({
                "kind": "craft_recipe",
                "id": 4,
                "recipe_id": FIXTURE_RECIPE,
                "quantity": 1,
                "source": fixture_source(),
            }),
            json!({
                "kind": "finish_minigame",
                "id": 5,
                "minigame_id": FIXTURE_MINIGAME,
                "count": 1,
                "source": fixture_source(),
            }),
            json!({
                "kind": "complete_dungeon",
                "id": 6,
                "encounter_id": FIXTURE_BATTLE,
                "source": fixture_source(),
            }),
            json!({
                "kind": "gather_resource",
                "id": 7,
                "resource_id": FIXTURE_GATHER_RESOURCE,
                "quantity": 1,
                "source": fixture_source(),
            }),
            json!({
                "kind": "collect_node",
                "id": 8,
                "node_type": FIXTURE_NODE_TYPE,
                "quantity": 1,
                "source": fixture_source(),
            }),
            json!({
                "kind": "perform_pet_trick",
                "id": 10,
                "trick_id": FIXTURE_PET_TRICK,
                "count": 1,
                "source": fixture_source(),
            }),
        ]);
    objectives
}

fn fixture_reward() -> Value {
    json!([{"coins": 5, "items": [], "job_xp": {"4": 5}, "source": fixture_source()}])
}

/// The minimal valid merged inventory catalogue: no state rows, no inventory
/// definitions, and one `item-catalogue.json` row for each target definition.
fn fixture_inventory() -> Value {
    let empty = format!("{:x}", Sha256::digest(0i32.to_le_bytes()));
    let span = |kind: &str, offset: usize, profile_id: i32| {
        json!({
            "kind": kind,
            "offset": offset,
            "length": 4,
            "profile_id": profile_id,
            "sha256": empty.as_str(),
        })
    };
    json!({
        "schema_version": 1,
        "player_guid": 1,
        "default_state": {"coins": 0, "items": [], "equipment": []},
        "profiles": [{"id": 1, "allowed_classes": [1], "slots": []}],
        "spans": [
            span("coins", 13, 0),
            span("equipment", 17, 1),
            span("items", 21, 0),
        ],
        "definitions": [],
    })
}

fn fixture_recipe_book() -> Value {
    let source = || json!({"kind": "reconstructed", "evidence": "(R) T358 scratch recipe"});
    let mut fields = serde_json::Map::new();
    for field in [
        "key",
        "name",
        "recipe_id",
        "job",
        "job_profile_id",
        "minigame_type_id",
        "minigame_id",
        "experience_points",
        "output.definition_id",
        "output.count",
        "ingredients[0].definition_id",
        "ingredients[0].count",
    ] {
        fields.insert(field.into(), source());
    }
    json!({
        "schema_version": 1,
        "records": [{
            "key": "fixture.recipe",
            "name": "Fixture Recipe",
            "recipe_id": FIXTURE_RECIPE,
            "job": "chef",
            "job_profile_id": 45,
            "minigame_type_id": 1,
            "minigame_id": FIXTURE_MINIGAME,
            "ingredients": [{"definition_id": FIXTURE_ITEM, "count": 1}],
            "output": {"definition_id": FIXTURE_RECIPE, "count": 1},
            "experience_points": 0,
            "field_sources": fields,
            "source": [source()],
        }],
    })
}

fn fixture_field_battles() -> Value {
    json!({
        "schema_version": 1,
        "battles": [{
            "id": FIXTURE_BATTLE,
            "title_id": 1,
            "description_id": 2,
            "difficulty": 1,
        }],
    })
}

/// T503: one placed quest pickup so the graded `collect_node` target resolves through
/// the production `GatheringNodes::load` (the loader that owns `quest_nodes.json`).
/// The fixture needs no provenance tags; the loader reads only the typed rows.
fn fixture_quest_nodes() -> Value {
    json!({
        "schema_version": 1,
        "node_types": [{
            "key": FIXTURE_NODE_TYPE,
            "name": "Fixture Node",
            "model_id": 611,
            "scale": 1.0,
            "composite_effect_id": 0,
            "interact_range": 12,
            "cursor_id": 0,
            "placement_y_offset": 0.0,
            "drop_table": [],
        }],
        "pools": [{
            "key": "fixture-node-pool",
            "zone_definition_id": 1,
            "node_type": FIXTURE_NODE_TYPE,
            "max_active_nodes": 1,
            "respawn_seconds": 30,
        }],
        "spawns": [{
            "id": 1,
            "pool_key": "fixture-node-pool",
            "zone_definition_id": 1,
            "position": [0.0, 0.0, 0.0],
            "heading": 0.0,
        }],
    })
}

fn fixture_npc(npc_id: u64, name: &str, name_string_id: u32, model_id: u32, x: f64) -> Value {
    json!({
        "npc_id": npc_id,
        "name": name,
        "name_string_id": name_string_id,
        "zone_id": 1,
        "position": [x, 0.0, 0.0],
        "placement": "legacy_data",
        "model_id": model_id,
        "roles": ["quest_giver"],
        "quest_ids_given": [],
        "quests_given": [],
        "quests_accepted": [],
    })
}

fn write_json(path: &Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("fixture JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

pub fn fixture(name: &str, defect: Defect) -> Scratch {
    let scratch = Scratch::new(name);
    let root = scratch.path();
    let mut quests = vec![fixture_quest(
        FIXTURE_QUEST,
        FIXTURE_TITLE,
        FIXTURE_GIVER,
        fixture_objectives(),
        fixture_reward(),
        json!([]),
    )];
    let mut quest_ids = vec![FIXTURE_QUEST];
    let mut blocker = None;
    match defect {
        Defect::None => {}
        Defect::MissingGiver => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_MISSING_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::WrappedTitle => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                u32::MAX,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::EmptyReward => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                json!([]),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::DisabledPrerequisite => {
            // The defect quest would fail the reward gate if it were graded, so
            // an empty reward proves the audit classified it unreachable instead.
            // Its graded objectives let the target test patch one to an
            // unshipped id and prove the unreachable record is skipped there too.
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_objectives(),
                json!([]),
                json!([FIXTURE_BLOCKER]),
            ));
            blocker = Some(json!({
                "id": FIXTURE_BLOCKER,
                "title_string_id": FIXTURE_TITLE,
                "giver_npc_id": FIXTURE_GIVER,
                "prerequisite_quest_ids": [],
                "objectives": [],
                "rewards": [],
                "source": fixture_source(),
                "enabled": false,
            }));
            quest_ids.extend([FIXTURE_DEFECT, FIXTURE_BLOCKER]);
        }
        Defect::MissingPrerequisite => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([FIXTURE_MISSING_PREREQUISITE]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::VisitGeometry { position, radius } => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_TITLE,
                FIXTURE_GIVER,
                fixture_visit_objective(position, radius),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
    }
    if let Some(blocker) = blocker {
        quests.push(blocker);
    }
    let jobs = json!({
        "schema_version": 1,
        "records": [{
            "id": "fixture_job",
            "name": "Fixture Job",
            "profile_id": 4,
            "category": "combat",
            "max_level": 20,
            "members_only": false,
            "trainer_npc_ids": [],
            "quest_ids": quest_ids,
            "abilities": [],
            "source": fixture_source(),
        }],
    });
    write_json(
        &root.join("zones.json"),
        &json!({"schema_version": 1, "records": [{"id": 1, "name": "Fixture Zone"}]}),
    );
    write_json(&root.join("npc_spawns_extra.json"), &json!({"records": []}));
    write_json(&root.join("jobs.json"), &jobs);
    write_json(&root.join("inventory.json"), &fixture_inventory());
    item_catalogue::write(
        root,
        &[
            item_catalogue::sized_row(FIXTURE_ITEM as u32, 8),
            item_catalogue::sized_row(FIXTURE_RECIPE, 8),
        ],
    );
    write_json(&root.join("recipes.json"), &fixture_recipe_book());
    write_json(&root.join("field_battles.json"), &fixture_field_battles());
    write_json(&root.join("quest_nodes.json"), &fixture_quest_nodes());
    write_json(
        &root.join("quests/fixture.json"),
        &json!({"schema_version": 1, "records": quests}),
    );
    write_json(
        &root.join("npcs/fixture.json"),
        &json!({"schema_version": 1, "records": [
            fixture_npc(FIXTURE_GIVER, "Fixture giver", 700_002, 101, 10.0),
            fixture_npc(FIXTURE_TARGET, "Fixture target", 700_003, 102, 12.0),
        ]}),
    );
    scratch
}
