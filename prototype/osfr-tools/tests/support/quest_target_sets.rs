//! Shipped id sets a graded quest objective target must resolve in.
//!
//! Test-only. [`Targets::load`] reads the same content root as the quest
//! catalogue through the loader that owns each id:
//!
//! * `dungeons`: `activity_portal::Catalogue` (the `field_battles.json` rows the
//!   encounter engine converts into battles) plus the raw `encounters.json`
//!   `dungeons[].id` rows the encounter loader owns but exposes no id list for.
//! * `minigames`: the game id of every server path that can emit a
//!   `FinishMinigame` quest event, one row per emitter:
//!
//!   | Emitter | Id | Source |
//!   |---|---|---|
//!   | settled soccer (`Gameplay::tick_world` -> `npc_minigame_win`) | game id | `soccer.json` `games[].id` |
//!   | chess result (`Gameplay::apply_chess` -> `npc_minigame_win`) | row id | `chess.json` `rows[].row_id` |
//!   | craft win (`gameplay/crafting.rs` -> `npc_minigame_win`) | recipe game id | `recipes.json` `records[].minigame_id` |
//!   | mining win (`Gameplay::apply_type8` -> `npc_minigame_win`) | 18 | `mining::MINING_GAME_ID` |
//!   | harvesting win (`Gameplay::apply_type8` -> `npc_minigame_win`) | 111 | `Gameplay`'s `HARVESTING_GAME_ID` |
//!   | paid fishing catch (`Gameplay::handle_fishing`) | spot game id | `fishing_spots.json` `spots[].game_id` |
//!   | kart claim (`Gameplay::handle_kart_entry`/tick) | 31 | `kart_entry.json` `race.speedway` |
//!
//!   Chess sends the row id as the 39/18 game id through its `RunSpec`. A craft
//!   win sends the selected recipe's `minigame_id`; a recipe without one falls
//!   back to the job content's game id (cooking 46, smelting 77, forging 139,
//!   each a private `const GAME` the content must match), and the shipped
//!   `recipes.json` rows carry all three ids.
//! * derby: `derby_entry.json` `game_id` (39) is a shipped minigame id, but no
//!   server path emits `FinishMinigame` for it
//!   (`Gameplay::pay_derby_completion` never calls `npc_minigame_win`, and the
//!   derby sends no 39/18 or `OnGameWonMsg` 39/14), so it is reported in
//!   [`Targets::unproduced_minigames`] instead of being accepted.
//! * `items`: `Inventory::load`'s merged `inventory.json` +
//!   `item-catalogue.json` definitions. An absent catalogue is an empty set, so
//!   a graded item target fails loudly instead of passing vacuously.
//! * `recipes`: `RecipeCatalog::load_runtime`'s stable `recipe_id` rows plus the
//!   three `CraftingJob::opening_recipe_id` fallbacks `Gameplay::apply_craft`
//!   emits when the learned recipe has no stable id.
//! * `gather`: mining/harvesting output definition ids plus the local opening
//!   drops (`Gameplay::local_gather_drop` match keys).

use osfr_tools::{
    activity_portal, chess_service, harvesting,
    inventory::Inventory,
    mining::{self, Type8Engine},
    recipes::{CraftingJob, RecipeCatalog},
    soccer_entry,
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, io, path::Path};

/// `src/gameplay.rs`'s private `HARVESTING_GAME_ID`: the one harvesting game the
/// server loads, so the only harvesting id `finish_minigame` can receive.
pub const HARVESTING_GAME_ID: u32 = 111;
/// `Gameplay::local_gather_drop` match keys for the opening local drops. These
/// are gather event resource keys, not item definition ids.
pub const LOCAL_GATHER_RESOURCES: [u32; 3] = [1_001, 1_002, 1_003];

pub fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|value| u32::try_from(value).ok())
}

/// Target-id sets a graded objective must resolve in, loaded once from the same
/// content root as the quest catalogue.
#[derive(Debug, Default)]
pub struct Targets {
    /// `field_battles.json` battle ids plus `encounters.json` dungeon ids.
    pub dungeons: BTreeSet<u32>,
    /// The ids a server path can emit as a `FinishMinigame` event.
    pub minigames: BTreeSet<u32>,
    /// Shipped minigame ids no server path can emit; a target on one can never
    /// advance and is refused with the reason instead of the unknown-id message.
    pub unproduced_minigames: BTreeSet<u32>,
    /// The merged `inventory.json` + `item-catalogue.json` definitions.
    pub items: BTreeSet<i32>,
    /// The loaded recipe book's stable server ids.
    pub recipes: BTreeSet<u32>,
    /// Mining/harvesting output definitions plus the local opening drops.
    pub gather: BTreeSet<u32>,
}

impl Targets {
    pub fn load(root: &Path) -> io::Result<Self> {
        let mut targets = Self {
            gather: LOCAL_GATHER_RESOURCES.into_iter().collect(),
            ..Self::default()
        };

        let battles = root.join("field_battles.json");
        if battles.is_file() {
            // The activity portal reads the same battle rows the encounter
            // engine converts into dungeons, and exposes their ids.
            targets.dungeons.extend(
                activity_portal::Catalogue::load(&battles)?
                    .activities()
                    .iter()
                    .map(|activity| activity.id),
            );
        }
        // `EncounterEngine::load` owns the dungeon table but exposes no id list;
        // the raw `dungeons` array is the same authored rows it loads.
        targets
            .dungeons
            .extend(read_ids(root, "encounters.json", |document| {
                array_ids(document, "dungeons", "id")
            }));

        let chess = root.join(chess_service::CONTENT_FILE);
        if chess.is_file() {
            // `run_spec` sends `row_id` as the game id; the loader validates it.
            targets.minigames.extend(
                chess_service::ChessContent::load(&chess)?
                    .rows
                    .iter()
                    .map(|row| row.row_id),
            );
        }
        // `KartGateway::load` owns the file; the kart claim credits the authored
        // speedway id, and the service exposes no id accessor.
        targets
            .minigames
            .extend(read_ids(root, "kart_entry.json", |document| {
                as_u32(&document["race"]["speedway"]).into_iter().collect()
            }));
        let soccer = root.join(soccer_entry::CONTENT_FILE);
        if soccer.is_file() {
            targets.minigames.extend(
                soccer_entry::Content::load(&soccer)?
                    .games()
                    .iter()
                    .map(|game| game.id),
            );
        }
        // `FishingSpots::load` owns the file; a paid catch credits the spot's
        // `game_id` to `finish_minigame`.
        targets
            .minigames
            .extend(read_ids(root, "fishing_spots.json", |document| {
                array_ids(document, "spots", "game_id")
            }));

        let mining = root.join(mining::CONTENT_FILE);
        if mining.is_file() {
            let engine = Type8Engine::load(
                root,
                mining::CONTENT_FILE,
                mining::MINING_GAME_ID,
                mining::FIRST_MINING_INSTANCE,
                None,
            )?;
            targets.minigames.insert(mining::MINING_GAME_ID as u32);
            targets.gather.extend(
                engine
                    .spec()
                    .outputs
                    .iter()
                    .filter_map(|output| u32::try_from(output.definition_id).ok()),
            );
        }
        let harvest = root.join(harvesting::CONTENT_FILE);
        if harvest.is_file() {
            let engine = Type8Engine::load(
                root,
                harvesting::CONTENT_FILE,
                HARVESTING_GAME_ID as i32,
                harvesting::FIRST_INSTANCE_ID,
                None,
            )?;
            targets.minigames.insert(HARVESTING_GAME_ID);
            targets.gather.extend(
                engine
                    .spec()
                    .outputs
                    .iter()
                    .filter_map(|output| u32::try_from(output.definition_id).ok()),
            );
        }

        // `Gameplay::load` merges the optional catalogue under the same root.
        if root.join("inventory.json").is_file() {
            targets
                .items
                .extend(Inventory::load(root, None)?.definition_ids());
        }
        let recipes = RecipeCatalog::load_runtime(root)?;
        targets
            .recipes
            .extend(recipes.records.iter().filter_map(|recipe| recipe.recipe_id));
        // `Gameplay::apply_craft` falls back to the job's local opening recipe id
        // when the learned recipe has no stable id, so those three server ids are
        // valid `craft_recipe` events even though the recipe book does not row them.
        targets.recipes.extend([
            CraftingJob::Chef.opening_recipe_id(),
            CraftingJob::Blacksmith.opening_recipe_id(),
            CraftingJob::Miner.opening_recipe_id(),
        ]);
        // A selected recipe's minigame id is the 39/18 game id a craft win sends.
        targets.minigames.extend(
            recipes
                .records
                .iter()
                .filter_map(|recipe| recipe.minigame_id),
        );

        // Derby is the one shipped minigame content row with no producer: its
        // completion pays but sends no result packet the quest glue reads.
        targets.unproduced_minigames.extend(
            read_ids(root, "derby_entry.json", |document| {
                as_u32(&document["game_id"]).into_iter().collect()
            })
            .into_iter()
            .filter(|id| !targets.minigames.contains(id)),
        );
        Ok(targets)
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// Ids read from one optional content file. An absent file yields none, so a
/// content tree without that activity stays empty instead of falling back.
fn read_ids(root: &Path, file: &str, ids: impl FnOnce(&Value) -> BTreeSet<u32>) -> BTreeSet<u32> {
    let path = root.join(file);
    if !path.is_file() {
        return BTreeSet::new();
    }
    ids(&read_json(&path))
}

/// The `u32` values of `field` in one JSON array.
fn array_ids(document: &Value, array: &str, field: &str) -> BTreeSet<u32> {
    document[array]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| as_u32(&row[field]))
        .collect()
}
