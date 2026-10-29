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
#[path = "json.rs"]
mod json;
#[path = "scratch.rs"]
mod scratch;

pub use scratch::Scratch;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const FIXTURE_DEFECT: u32 = 9_100_002;
pub const FIXTURE_QUEST: u32 = 9_100_001;
pub const FIXTURE_ITEM: i32 = 9_100_100;
pub const FIXTURE_RECIPE: u32 = 9_100_101;
/// T790: a second item definition so the fixture `any_of` has two independent
/// alternatives, and canary C (which removes `FIXTURE_ITEM`) does not disturb it.
pub const FIXTURE_ANY_OF_ITEM: i32 = 9_100_106;
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
/// T789: a `job_abilities.json` definition id and an `enemies.json` `definition_id`
/// a graded `use_ability` step names.
pub const FIXTURE_ABILITY: u32 = 9_100_103;
pub const FIXTURE_ENEMY: u32 = 9_100_104;
/// T789: a second catalogue item, so Canary C (which removes `FIXTURE_ITEM`) keeps
/// its exact collect_item/equip_item violation count. It is also the built-in potion row
/// (`item_bar::Effects::default`) the fixture's absent `consumables.json` is loaded with, so a
/// `use_item` step can name an id the bar can actually spend.
pub const FIXTURE_USE_ITEM: i32 = 11_002;
const FIXTURE_GIVER: u64 = 9_000_100_001;
pub const FIXTURE_TARGET: u64 = 9_000_100_002;
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
/// producer is exercised through the production `RecipeCatalog` loader. The recipe
/// is a blacksmith row: only forging keeps its score and timer server-side, so it is
/// the craft producer a `require` may bound (F854).
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
            json!({
                "kind": "use_ability",
                "id": 11,
                "ability_id": FIXTURE_ABILITY,
                "count": 1,
                "target_actor_kind_id": FIXTURE_ENEMY,
                "source": fixture_source(),
            }),
            json!({
                "kind": "use_item",
                "id": 12,
                "item_id": FIXTURE_USE_ITEM,
                "count": 1,
                "source": fixture_source(),
            }),
            // T790: one any-of over two item definitions the fixture catalogue owns.
            json!({
                "kind": "any_of",
                "id": 13,
                "count": 1,
                "alternatives": [
                    {"kind": "collect_item", "item_id": FIXTURE_ANY_OF_ITEM},
                    {"kind": "collect_item", "item_id": FIXTURE_RECIPE},
                ],
                "source": fixture_source(),
            }),
            // T786: one Talk step that hands the fixture item over.
            json!({
                "kind": "deliver_item",
                "id": 14,
                "npc_id": FIXTURE_TARGET,
                "item_id": FIXTURE_ITEM,
                "quantity": 1,
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
            "job": "blacksmith",
            "job_profile_id": 16,
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

/// T789: the one authored ability the graded `use_ability` step names. Production
/// `OpeningAbilities::load_job_abilities` validates this file; the audit reads its ids. The row
/// keeps a positive damage multiplier so that loader would accept it.
fn fixture_job_abilities() -> Value {
    json!({
        "schema_version": 1,
        "jobs": [{
            "profile_id": 4,
            "slots": [{
                "slot": 1,
                "definition_id": FIXTURE_ABILITY,
                "name_id": 700_010,
                "icon_id": 1,
                "damage_multiplier": 1.0,
                "cooldown_ms": 1000,
                "min_level": 1,
            }],
        }],
    })
}

/// T789: the one enemy definition the graded `use_ability` step targets. `enemies::EnemyEngine`
/// owns this file but is not bound by this audit; only `definitions[].definition_id` is read.
fn fixture_enemies() -> Value {
    json!({
        "schema_version": 1,
        "definitions": [{
            "id": "fixture_enemy",
            "kind": "Fixture Enemy",
            "actor_kind_id": FIXTURE_ENEMY,
            "definition_id": FIXTURE_ENEMY,
            "model_id": 3,
            "name_string_id": 700_011,
            "texture_alias": "",
            "max_health": 100,
            "damage": 1,
            "source": "(R) T789 scratch enemy definition",
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
    json::write_pretty(
        &root.join("zones.json"),
        &json!({"schema_version": 1, "records": [{"id": 1, "name": "Fixture Zone"}]}),
    );
    json::write_pretty(&root.join("npc_spawns_extra.json"), &json!({"records": []}));
    json::write_pretty(&root.join("jobs.json"), &jobs);
    json::write_pretty(&root.join("inventory.json"), &fixture_inventory());
    item_catalogue::write(
        root,
        &[
            item_catalogue::sized_row(FIXTURE_ITEM as u32, 8),
            item_catalogue::sized_row(FIXTURE_RECIPE, 8),
            item_catalogue::sized_row(FIXTURE_USE_ITEM as u32, 8),
            item_catalogue::sized_row(FIXTURE_ANY_OF_ITEM as u32, 8),
        ],
    );
    json::write_pretty(&root.join("recipes.json"), &fixture_recipe_book());
    json::write_pretty(&root.join("field_battles.json"), &fixture_field_battles());
    json::write_pretty(&root.join("quest_nodes.json"), &fixture_quest_nodes());
    json::write_pretty(&root.join("job_abilities.json"), &fixture_job_abilities());
    json::write_pretty(&root.join("enemies.json"), &fixture_enemies());
    json::write_pretty(
        &root.join("quests/fixture.json"),
        &json!({"schema_version": 1, "records": quests}),
    );
    json::write_pretty(
        &root.join("npcs/fixture.json"),
        &json!({"schema_version": 1, "records": [
            fixture_npc(FIXTURE_GIVER, "Fixture giver", 700_002, 101, 10.0),
            fixture_npc(FIXTURE_TARGET, "Fixture target", 700_003, 102, 12.0),
        ]}),
    );
    scratch
}
