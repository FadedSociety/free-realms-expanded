//! Shipped id sets a graded quest objective target must resolve in.
//!
//! Test-only. [`Targets::load`] reads the same content root as the quest
//! catalogue through the loader that owns each id:
//!
//! * `dungeons`: `activity_portal::Catalogue` (the `field_battles.json` rows the
//!   encounter engine converts into battles) plus the raw `encounters.json`
//!   `dungeons[].id` rows the encounter loader owns but exposes no id list for.
//! * `minigames`: the game id of every server path that can emit a
//!   `FinishMinigame` quest event, one row per emitter, plus the result facts
//!   that emitter reports (`facts`, T787):
//!
//!   | Emitter | Id | Source |
//!   |---|---|---|
//!   | settled soccer (`Gameplay::tick_world` -> `npc_minigame_win`) | game id | `soccer.json` `games[].id` |
//!   | chess result (`Gameplay::apply_chess` -> `npc_minigame_win`) | row id | `chess.json` `rows[].row_id` |
//!   | craft win (`gameplay/crafting.rs`; facts only for a Blacksmith recipe) | recipe game id | `recipes.json` `records[].minigame_id` |
//!   | mining win (`Gameplay::apply_type8` -> `npc_minigame_win`) | 18 and each Games window ore row | `mining.json` `games[].game_id` (L934) |
//!   | harvesting win (`Gameplay::apply_type8` -> `npc_minigame_win`) | 111 and each region | `harvesting.json` `games[].game_id` |
//!   | mail sorting win (`Gameplay::apply_type8` -> `npc_minigame_win`) | each town | `mail_sorting.json` `games[].game_id` (L934) |
//!   | checkers win (`Gameplay::handle_checkers` -> `npc_minigame_win`) | 21 | `checkers::GAME_ID` (L934) |
//!   | paid fishing catch (`Gameplay::handle_fishing`) | spot game id | `fishing_spots.json` `spots[].game_id` |
//!   | kart claim (`Gameplay::handle_kart_entry`/tick) | entered row id (28, 628-634) | `KartGateway::minigame_ids` (`kart_gates.json` + the playable `kart_rows.json` rows) |
//!   | finished derby (`Gameplay::pay_derby_completion`) | entered row id | `DerbyGateway::rows` (`derby_entry.json` default + `derby_rows.json`) |
//!   | won song (`Gameplay::handle_music` -> `npc_minigame_win_id_only`, L961) | game id | `music_game.json` `games[].game_id` |
//!   | finished Pirate's Plunder match (`Gameplay::settle_pirates`) | arena's row | `ship_combat.json` `arenas[].minigame_id` |
//!   | won Tower Defense game (`Gameplay::handle_tower_defense` -> `npc_minigame_win_id_only`, L962) | game id | `tower_defense.json` `games[].game_id` |
//!
//!   Chess sends the row id as the 39/18 game id through its `RunSpec`; it reports
//!   an elapsed time and no score (the results page always carries zero). A craft
//!   win sends the selected recipe's `minigame_id`; a recipe without one falls
//!   back to the job content's game id (cooking 46, smelting 77, forging 139,
//!   each a private `const GAME` the content must match), and the shipped
//!   `recipes.json` rows carry all three ids. Only a Blacksmith recipe's movie
//!   measures its score and timer server-side (forging); the Chef and Miner
//!   movies take both from the client, so their ids report no result fact and a
//!   `require` on them is refused. A finished derby credits the entered row's
//!   MiniGameData id (`DerbyGateway::game_id`): the default row 39 "Wildwood Demo
//!   Derby" or an extra `derby_rows.json` row, never its type id 11, which is
//!   MiniGameData row 11 (the Trading Card Game). Mining,
//!   harvesting, forging, chess and soccer read their elapsed seconds and score
//!   from the server-built 39/18; kart reports its own elapsed time and no
//!   placement (a single-player time trial); the derby reports score, placement
//!   and knock-outs; a paid fishing catch reports no result fact.
//! * `items`: `Inventory::load`'s merged `inventory.json` +
//!   `item-catalogue.json` definitions. A `collect_item`, `equip_item` or
//!   `deliver_item` target must be one of them. An absent catalogue is an empty
//!   set, so a graded item target fails loudly instead of passing vacuously.
//! * `recipes`: `RecipeCatalog::load_runtime`'s stable `recipe_id` rows plus the
//!   three `CraftingJob::opening_recipe_id` fallbacks `Gameplay::apply_craft`
//!   emits when the learned recipe has no stable id.
//! * `gather`: mining/harvesting output definition ids plus the local opening
//!   drops (`Gameplay::local_gather_drop` match keys).
//! * `nodes`: the node-type keys of quest pickups with at least one placed spawn,
//!   read through `harvest_nodes::GatheringNodes::load` (the loader that owns
//!   `quest_nodes.json`, T503).
//! * `profiles`: the `profiles.json` `profiles[].id` rows `Profiles::load` owns
//!   (it exposes no id list), read from the profile content root: the opening
//!   content the session ranks jobs from, not the quest catalogue root.
//! * `pet_tricks`: `PetContent`'s authored `tricks[].id` rows, read through its
//!   accessor from the bundled `pets.json` the pet service serves. A
//!   `perform_pet_trick` target must be one of them.
//! * `abilities`: the `job_abilities.json` `jobs[].slots[].definition_id` rows
//!   `OpeningAbilities::load_job_abilities` validates. That loader exposes no id
//!   list, so the same authored file is read raw; a `use_ability` goal must name
//!   one of them.
//! * `enemy_kinds`: the `enemies.json` `definitions[].definition_id` rows
//!   `enemies::EnemyEngine::load` owns (it exposes no id list). The encounter
//!   engine reports this definition id on a kill and on an accepted cast, so a
//!   `use_ability` target must be one of them.
//! * `any_of` (T790): an alternative has no set of its own; each leaf id resolves
//!   in the set for its family above (talk in the draw set, collect in `items`,
//!   gather in `gather`), never in a combined set.
//! * `consumables`: the single-use rows of `item_bar::Effects` (the loaded
//!   `consumables.json`, or the built-in potion table when it is absent). A
//!   `use_item` goal only ever credits when `ItemBar::consume` actually spends
//!   the item, so only a single-use effect row can produce the event.

use super::quest_records::as_u32;
use osfr_tools::{
    activity_portal, checkers, chess_service,
    derby_gateway::DerbyGateway,
    harvest_nodes::GatheringNodes,
    harvesting,
    inventory::Inventory,
    item_bar::Effects,
    kart_gateway::KartGateway,
    mail_sorting,
    mining::{self, Type8Engine},
    music_game,
    pets::PetContent,
    recipes::{CraftingJob, RecipeCatalog},
    ship_combat, soccer_entry, tower_defense,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

/// `src/gameplay.rs`'s private `HARVESTING_GAME_ID`: the harvesting file's own game, which
/// `Type8Engine::load` binds (its `games` regions come from the loaded content).
pub const HARVESTING_GAME_ID: u32 = 111;
/// `Gameplay::local_gather_drop` match keys for the opening local drops. These
/// are gather event resource keys, not item definition ids.
pub const LOCAL_GATHER_RESOURCES: [u32; 3] = [1_001, 1_002, 1_003];

/// What the 39/18 result path reports for craft, mining, harvesting, chess and soccer.
const SCORE_AND_ELAPSED: [MinigameFact; 2] = [MinigameFact::Score, MinigameFact::Elapsed];
/// The derby's own per-driver result facts.
const DERBY_FACTS: [MinigameFact; 3] = [
    MinigameFact::Score,
    MinigameFact::Placement,
    MinigameFact::Knockout,
];

/// One fact a producer reports on its `MinigameResult` event (T787).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MinigameFact {
    Score,
    Elapsed,
    Placement,
    Knockout,
}

/// Target-id sets a graded objective must resolve in, loaded once from the same
/// content root as the quest catalogue.
#[derive(Debug, Default)]
pub struct Targets {
    /// `field_battles.json` battle ids plus `encounters.json` dungeon ids.
    pub dungeons: BTreeSet<u32>,
    /// The ids a server path can emit as a `FinishMinigame` event.
    pub minigames: BTreeSet<u32>,
    /// What the producer for each `minigames` id can report; an id with no row
    /// reports nothing (an empty set).
    pub facts: BTreeMap<u32, BTreeSet<MinigameFact>>,
    /// The merged `inventory.json` + `item-catalogue.json` definitions.
    pub items: BTreeSet<i32>,
    /// The loaded recipe book's stable server ids.
    pub recipes: BTreeSet<u32>,
    /// Mining/harvesting output definitions plus the local opening drops.
    pub gather: BTreeSet<u32>,
    /// Placed quest pickup node-type keys (`quest_nodes.json`, T503).
    pub nodes: BTreeSet<String>,
    /// Profiles the loaded profile content declares; a `job_level` rank is only
    /// real for one of them.
    pub profiles: BTreeSet<u32>,
    /// `pets.json` trick ids a `perform_pet_trick` step may name.
    pub pet_tricks: BTreeSet<u32>,
    /// `job_abilities.json` definition ids a `use_ability` step may name.
    pub abilities: BTreeSet<u32>,
    /// `enemies.json` definition ids a `use_ability` target may name.
    pub enemy_kinds: BTreeSet<u32>,
    /// Single-use `item_bar::Effects` definitions a `use_item` step may name.
    pub consumables: BTreeSet<i32>,
}

impl Targets {
    /// Register one producer: its id is a `finish_minigame` target and `facts` are
    /// the result facts it reports. Re-registering an id unions the facts, so an id
    /// two loaders can send never loses a reportable fact.
    fn producer(&mut self, id: u32, facts: impl IntoIterator<Item = MinigameFact>) {
        self.minigames.insert(id);
        self.facts.entry(id).or_default().extend(facts);
    }

    /// Whether the producer for `minigame` reports `fact`.
    #[allow(dead_code)]
    pub fn reports(&self, minigame: u32, fact: MinigameFact) -> bool {
        self.facts
            .get(&minigame)
            .is_some_and(|facts| facts.contains(&fact))
    }

    pub fn load(root: &Path, profile_root: &Path) -> io::Result<Self> {
        let mut targets = Self {
            gather: LOCAL_GATHER_RESOURCES.into_iter().collect(),
            profiles: read_ids(profile_root, "profiles.json", |document| {
                array_ids(document, "profiles", "id")
            }),
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
            // `run_spec` sends `row_id` as the game id; the loader validates it. Chess
            // settles with a server-measured elapsed time and a zero score, so it can
            // never meet a `min_score` bound.
            for row in chess_service::ChessContent::load(&chess)?.rows.iter() {
                targets.producer(row.row_id, [MinigameFact::Elapsed]);
            }
        }
        // `KartGateway::load` owns `kart_gates.json` and `kart_rows.json`; a kart claim credits the
        // MiniGameData row the race ran as (L930), one of `minigame_ids`. A time trial has no
        // rival, so no placement is reported.
        if let Some(kart) = KartGateway::load(root, None)? {
            for row in kart.minigame_ids() {
                targets.producer(row, [MinigameFact::Elapsed]);
            }
        }
        // L932: every `soccer.json` game is a producer, the Games window rows 42-44 and the
        // quest-only rows 322-333 alike; a win reports the team's goals and the match time.
        let soccer = root.join(soccer_entry::CONTENT_FILE);
        if soccer.is_file() {
            for game in soccer_entry::Content::load(&soccer)?.games() {
                targets.producer(game.id, SCORE_AND_ELAPSED);
            }
        }
        // L961: every `music_game.json` song; the 39/18 total is the client's, so no fact.
        if root.join(music_game::CONTENT_FILE).is_file() {
            for game in music_game::MusicContent::load(root)?.games {
                targets.producer(u32::try_from(game.game_id).map_err(io::Error::other)?, []);
            }
        }
        // L962: every `tower_defense.json` game; the 39/18 total is the client's, so no fact.
        if root.join(tower_defense::CONTENT_FILE).is_file() {
            for game in tower_defense::TdContent::load(root)?.games {
                targets.producer(u32::try_from(game.game_id).map_err(io::Error::other)?, []);
            }
        }
        // Every arena's row: a finished match reports the gold banked and first place.
        if let Some(content) = ship_combat::Content::load(root)? {
            for arena in &content.arenas {
                targets.producer(arena.minigame_id, [MinigameFact::Score, MinigameFact::Placement]);
            }
        }
        // `FishingSpots::load` owns the file; a paid catch credits the spot's
        // `game_id` to `finish_minigame` and reports no result fact.
        for spot in read_ids(root, "fishing_spots.json", |document| {
            array_ids(document, "spots", "game_id")
        }) {
            targets.producer(spot, []);
        }

        // L934: every game each Type8 file offers is a producer, read from the loaded content so a
        // new board registers itself: mining (18 and its ore rows), harvesting (111 and its
        // regions) and mail sorting (the six towns). Each win is a server-built 39/18 with the
        // run's score and time, and each output definition is a `gather` target.
        for (file, game, first_instance) in [
            (
                mining::CONTENT_FILE,
                mining::MINING_GAME_ID,
                mining::FIRST_MINING_INSTANCE,
            ),
            (
                harvesting::CONTENT_FILE,
                HARVESTING_GAME_ID as i32,
                harvesting::FIRST_INSTANCE_ID,
            ),
            (
                mail_sorting::CONTENT_FILE,
                mail_sorting::GAME_ID,
                mail_sorting::FIRST_INSTANCE_ID,
            ),
        ] {
            if !root.join(file).is_file() {
                continue;
            }
            let engine = Type8Engine::load(root, file, game, first_instance, None)?;
            for (id, spec) in engine.games() {
                if let Ok(id) = u32::try_from(id) {
                    targets.producer(id, SCORE_AND_ELAPSED);
                }
                targets.gather.extend(
                    spec.outputs
                        .iter()
                        .filter_map(|output| u32::try_from(output.definition_id).ok()),
                );
            }
        }
        // L934: the checkers table is always on (`portal_games::games`); a won game's 39/18
        // carries the measured time and a zero score, like chess.
        targets.producer(checkers::GAME_ID as u32, [MinigameFact::Elapsed]);

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
        // A selected recipe's minigame id is the 39/18 game id a craft win sends. Only
        // the blacksmith movie keeps the score and timer server-side; the chef and
        // miner movies take both from the client, so their ids report no result fact.
        for recipe in &recipes.records {
            let Some(minigame) = recipe.minigame_id else {
                continue;
            };
            match recipe.job {
                CraftingJob::Blacksmith => targets.producer(minigame, SCORE_AND_ELAPSED),
                CraftingJob::Chef | CraftingJob::Miner => targets.producer(minigame, []),
            }
        }

        // `DerbyGateway::load` owns the default package and every extra
        // `derby_rows.json` row; a finished match credits whichever row the
        // player entered and reports that row's driver result.
        if let Some(derby) = DerbyGateway::load(root, None)? {
            for row in derby.rows() {
                targets.producer(row.game_id() as u32, DERBY_FACTS);
            }
        }

        // T503: the loader owns `quest_nodes.json`; only placed pickups count, so a
        // configured type with no spawn is not reachable.
        if let Some(nodes) = GatheringNodes::load(root)? {
            targets
                .nodes
                .extend(nodes.pickup_kinds().into_iter().map(str::to_owned));
        }
        // T788: `PetContent` owns the trick table and the server serves it bundled, so the gate
        // reads the ids production does instead of re-parsing `pets.json`.
        targets
            .pet_tricks
            .extend(PetContent::bundled()?.trick_ids());
        // T789: the ability roster and the enemy catalogue own these files but expose no id
        // list, so the raw rows their loaders validate are read here.
        targets
            .abilities
            .extend(read_ids(root, "job_abilities.json", |document| {
                document["jobs"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|job| job["slots"].as_array().into_iter().flatten())
                    .filter_map(|slot| as_u32(&slot["definition_id"]))
                    .collect()
            }));
        targets
            .enemy_kinds
            .extend(read_ids(root, "enemies.json", |document| {
                array_ids(document, "definitions", "definition_id")
            }));
        // T789: the item bar owns the single-use rule (`consume` spends only a `single_use`
        // row), so its own loader decides which item ids can produce a `use_item` event.
        targets.consumables.extend(
            Effects::load(root)?
                .rows()
                .iter()
                .filter(|row| row.single_use)
                .map(|row| row.definition),
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
