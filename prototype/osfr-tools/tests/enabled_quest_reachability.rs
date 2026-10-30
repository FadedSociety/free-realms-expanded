//! T314/F343/T358/F394: structural reachability gate for the enabled split quest
//! catalogue.
//!
//! The shipped content tree is read through the production loaders:
//! `QuestEngine::from_split_content` (definitions, giver binding, reward bounds,
//! the enabled set and the 49/13 end-screen encoder) and `world::NpcVisibility`
//! (the 35/2 draw set through `teleport_targets` and `content_key`). A raw JSON
//! pass only reads what those loaders do not expose: the `enabled` flag,
//! prerequisite ids, objective target fields, the authored title id and the
//! authored NPC `quest_giver` role.
//!
//! Resolved objective kinds: `talk_to_npc`, `deliver_item`, `interact_actor`,
//! `reach_poi`, `defeat`, `collect_item`, `equip_item`, `craft_recipe`,
//! `complete_dungeon`, `finish_minigame`, `gather_resource`, `collect_node`,
//! `job_level`, `perform_pet_trick`, `use_ability`, `use_item` and `any_of`. An `any_of` has
//! no target of its own: every alternative is graded as its leaf kind (T790).
//! The item, recipe, dungeon, minigame, gather, quest-pickup, pet-trick, ability,
//! enemy and single-use consumable kinds additionally resolve their target ids in the id sets owned by
//! `tests/support/quest_target_sets.rs`, which names the owning loader and every
//! `FinishMinigame` emitter. A `deliver_item` step resolves its NPC in the production draw
//! set and its `item_id` in the catalogue set, so a hand-over cannot name an item no
//! player can hold. A
//! `collect_node` node type must have at least one placed spawn in
//! `quest_nodes.json` (`harvest_nodes::GatheringNodes::load`, T503). A `job_level`
//! profile must be declared by the loaded profile content (`Targets::profiles`);
//! its level bound (1..=20) and its jobs.json profile are the split loader's. A
//! `finish_minigame` id no server path emits is refused instead of being
//! certified, and `shipped_minigame_producers_are_pinned` fails if the accepted
//! set drifts. T787: a `require` on the step must name only result facts that id's
//! own producer reports (score/time for the 39/18 paths, placement/knock-outs for
//! the derby, time for kart, nothing for a paid fishing catch).
//!
//! `visit_position` geometry is rejected by `QuestEngine::from_split_content`
//! (`valid_objective_kind`), the bound's only authority; the audit does not
//! re-check it and `visit_geometry_bounds_are_enforced` drives the production
//! loader for it. `client_reported` stays a residual: the client judges it and
//! reports a `client_id` through 45/7; that trigger selector is not a row in a
//! shipped server id table. The target test prints the residual set so the gap
//! stays visible.
//!
//! A quest is *reachable* when every prerequisite, directly or transitively,
//! exists and is enabled. An enabled quest behind a disabled prerequisite is
//! reported and excluded from both gates; it is not turned on here.
//!
//! No new wire. A reachable quest's reward is proved through the production
//! 49/13 encoder (`quest_end_packet`, reader `0x00c7b990`: actor u64, quest, end
//! text, animation dword, reward bundle `0x008e7930`, trailing float). A
//! drawable giver is what the production 35/2 path sends (AddNpc reader
//! `0x00921a10`); the 26/8 click constructor `0x008b6bc0` resolves the same key
//! through `NpcVisibility::content_key`. The graded targets add no packet: they
//! are server-only ids. The 39/18 result reader wrapper `0x009b7ec0` (body
//! `0x009b48b0`: shared header via `0x009b2340`, `u8 won`, three dwords) and
//! the 49/9 objective reader wrapper `0x00c7b5e0` (body `0x00c7af60`: counters,
//! no target id) are the client side of that server-only identity.
//!
//! The shared `content_dir`/`read_json` helpers, the raw quest records, the
//! scratch fixture and the target-id sets each own their file in `tests/support/`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_catalogue_fixture.rs"]
mod quest_catalogue_fixture;
#[path = "support/quest_records.rs"]
mod quest_records;
#[path = "support/quest_target_sets.rs"]
mod quest_target_sets;

use npc_quest_chain::{content_dir, read_json, reference_content};
use osfr_tools::{
    chess_service,
    quest_engine::{CharacterEligibility, QuestEngine},
    world::NpcVisibility,
};
use quest_catalogue_fixture::{
    Defect, FIXTURE_ABILITY, FIXTURE_BATTLE, FIXTURE_DEFECT, FIXTURE_ENEMY,
    FIXTURE_GATHER_RESOURCE, FIXTURE_ITEM, FIXTURE_MINIGAME, FIXTURE_NODE_TYPE, FIXTURE_PET_TRICK,
    FIXTURE_QUEST, FIXTURE_RECIPE, FIXTURE_TARGET, FIXTURE_USE_ITEM, Scratch, fixture,
};
use quest_records::{RawQuest, as_u32, read_quests, sorted_json_paths};
use quest_target_sets::{MinigameFact, Targets};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};

const CHARACTER: u64 = 7;
const I32_MAX: u64 = i32::MAX as u64;
const MAX_OBJECTIVE_COUNT: u64 = 1_000_000;
/// `NpcVisibility::load` places every regional and extra-spawn actor here.
const OPEN_WORLD_ZONE: u32 = 0;

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    }
}

struct Catalogue {
    engine: QuestEngine,
    quests: Vec<RawQuest>,
    by_id: BTreeMap<u32, usize>,
    /// Content NPC ids the production 35/2 path can send.
    drawable: BTreeSet<u64>,
    /// Authored NPC ids whose `roles` array carries `quest_giver`.
    giver_role_npcs: BTreeSet<u64>,
    /// Shipped id sets the graded objective targets must resolve in.
    targets: Targets,
}

impl Catalogue {
    fn load(root: &Path) -> io::Result<Self> {
        let engine = QuestEngine::from_split_content(root, CHARACTER, eligibility(), None)?;
        let visibility = NpcVisibility::load(root)?;
        // The loader's own draw list: `teleport_targets` applies the same
        // placed/extra/excluded filters the 35/2 update uses, and `content_key`
        // maps the runtime guid to the content NPC. Re-filtering authored rows
        // here would silently fork a loader rule.
        let drawable = visibility
            .teleport_targets(OPEN_WORLD_ZONE)
            .into_iter()
            .filter_map(|target| visibility.content_key(target.guid))
            .collect();
        let giver_role_npcs = read_quest_giver_rows(root);
        let targets = Targets::load(root, &reference_content())?;
        let quests = read_quests(root);
        // `from_split_content` already rejected duplicate ids and an empty list.
        let by_id = quests
            .iter()
            .enumerate()
            .map(|(index, quest)| (quest.id, index))
            .collect();
        Ok(Self {
            engine,
            quests,
            by_id,
            drawable,
            giver_role_npcs,
            targets,
        })
    }

    fn enabled(&self) -> impl Iterator<Item = &RawQuest> {
        self.quests.iter().filter(|quest| quest.enabled)
    }

    fn quest(&self, id: u32) -> Option<&RawQuest> {
        self.by_id.get(&id).map(|index| &self.quests[*index])
    }

    /// `None` means reachable; `Some(reason)` names the first missing or
    /// disabled prerequisite on the chain. Disabled quests themselves are
    /// reported as unreachable, which is what a dependent sees.
    fn reachability(&self) -> BTreeMap<u32, Option<String>> {
        let mut memo = BTreeMap::new();
        for quest in &self.quests {
            self.reachability_of(quest.id, &mut memo, &mut BTreeSet::new());
        }
        memo
    }

    fn reachability_of(
        &self,
        id: u32,
        memo: &mut BTreeMap<u32, Option<String>>,
        visiting: &mut BTreeSet<u32>,
    ) -> Option<String> {
        if let Some(reason) = memo.get(&id) {
            return reason.clone();
        }
        if !visiting.insert(id) {
            return Some(format!("prerequisite cycle through quest {id}"));
        }
        let reason = match self.quest(id) {
            None => Some(format!("missing quest {id}")),
            Some(quest) if !quest.enabled => Some(format!("disabled quest {id}")),
            Some(quest) => {
                let mut reason = None;
                for prerequisite in &quest.prerequisite_quest_ids {
                    match self.quest(*prerequisite) {
                        None => {
                            reason = Some(format!("missing prerequisite {prerequisite}"));
                            break;
                        }
                        Some(row) if !row.enabled => {
                            reason = Some(format!("disabled prerequisite {prerequisite}"));
                            break;
                        }
                        Some(_) => {
                            if let Some(inner) = self.reachability_of(*prerequisite, memo, visiting)
                            {
                                reason = Some(format!(
                                    "prerequisite {prerequisite} is unreachable ({inner})"
                                ));
                                break;
                            }
                        }
                    }
                }
                reason
            }
        };
        visiting.remove(&id);
        memo.insert(id, reason.clone());
        reason
    }

    fn unreachable(&self) -> BTreeMap<u32, String> {
        let reachability = self.reachability();
        self.enabled()
            .filter_map(|quest| {
                reachability
                    .get(&quest.id)
                    .cloned()
                    .flatten()
                    .map(|reason| (quest.id, reason))
            })
            .collect()
    }

    fn reachable_enabled(&self) -> Vec<&RawQuest> {
        let reachability = self.reachability();
        self.enabled()
            .filter(|quest| {
                reachability
                    .get(&quest.id)
                    .is_some_and(|reason| reason.is_none())
            })
            .collect()
    }

    /// Objective kinds whose target this gate resolves. Everything else is only
    /// bounded by the split loader; the target test prints the gap.
    fn unchecked_objective_kinds(&self) -> BTreeSet<&str> {
        const CHECKED: [&str; 17] = [
            "talk_to_npc",
            "deliver_item",
            "interact_actor",
            "reach_poi",
            "defeat",
            "collect_item",
            "equip_item",
            "craft_recipe",
            "complete_dungeon",
            "finish_minigame",
            "gather_resource",
            "collect_node",
            "job_level",
            "perform_pet_trick",
            "use_ability",
            "use_item",
            "any_of",
        ];
        self.enabled()
            .flat_map(|quest| quest.objectives.iter())
            .filter_map(|goal| goal["kind"].as_str())
            .filter(|kind| !CHECKED.contains(kind))
            .collect()
    }

    /// Every enabled record has a drawable giver that the NPC content defines
    /// with the `quest_giver` role (the production Talk-offer rule), and the
    /// engine kept it (objectives are not empty).
    fn giver_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.enabled() {
            if !self.engine.is_enabled(quest.id) {
                violations.push(format!(
                    "quest {}: enabled record is not offered by the engine (disabled or empty objectives)",
                    quest.id
                ));
            }
            if !self.giver_role_npcs.contains(&quest.giver_npc_id) {
                violations.push(format!(
                    "quest {}: giver {} has no npcs/*.json row with the quest_giver role",
                    quest.id, quest.giver_npc_id
                ));
            }
            if !self.drawable.contains(&quest.giver_npc_id) {
                violations.push(format!(
                    "quest {}: giver {} is not in the production 35/2 draw set",
                    quest.id, quest.giver_npc_id
                ));
            }
        }
        violations
    }

    /// Every reachable enabled objective target this gate knows how to resolve:
    /// talk and interact actors are in the production draw set; `reach_poi` ids
    /// are positive; kill and item ids/counts are positive; and every target
    /// kind with a shipped id table resolves in it. `any_of` alternatives are
    /// graded as their leaf kind (T790). Visit geometry bounds are
    /// enforced by `from_split_content` and deliberately not repeated here.
    /// Unreachable enabled records are skipped, as the reachability rule says.
    fn target_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        // One hand-over event credits every open step with the same NPC, item and units, so two
        // enabled steps may not share an NPC and item (T786 review: no double credit per item).
        let mut handed_over = BTreeSet::new();
        for quest in self.reachable_enabled() {
            for (index, goal) in quest.objectives.iter().enumerate() {
                let objective = goal["id"].as_u64().unwrap_or(0);
                let owner = format!("quest {} objective {objective} (#{index})", quest.id);
                self.objective_violations(&mut violations, &owner, goal, false);
                if goal["kind"] == "deliver_item"
                    && !handed_over.insert((goal["npc_id"].as_u64(), goal["item_id"].as_i64()))
                {
                    violations.push(format!(
                        "{owner}: another enabled deliver_item step already takes this item from this NPC"
                    ));
                }
            }
        }
        violations
    }

    /// One goal's target report. An `any_of` leaf has no `quantity` of its own (the
    /// goal's `count` is the total), so only its id is graded.
    fn objective_violations(
        &self,
        violations: &mut Vec<String>,
        owner: &str,
        goal: &Value,
        any_of_leaf: bool,
    ) {
        match goal["kind"].as_str().unwrap_or_default() {
            "talk_to_npc" => {
                self.target_violation(violations, owner, goal["npc_id"].as_u64());
            }
            "deliver_item" => {
                let item = goal["item_id"]
                    .as_i64()
                    .and_then(|id| i32::try_from(id).ok())
                    .unwrap_or(0);
                if item <= 0
                    || !(1..=MAX_OBJECTIVE_COUNT).contains(&goal["quantity"].as_u64().unwrap_or(0))
                {
                    violations.push(format!(
                        "{owner}: deliver_item target and quantity must be positive"
                    ));
                }
                if item > 0 && !self.targets.items.contains(&item) {
                    violations.push(format!(
                        "{owner} (deliver_item): item {item} is not defined by the shipped item catalogue"
                    ));
                }
                self.target_violation(violations, owner, goal["npc_id"].as_u64());
            }
            "interact_actor" => {
                self.target_violation(violations, owner, goal["actor_guid"].as_u64());
            }
            "reach_poi" => {
                if goal["poi_id"].as_u64().unwrap_or(0) == 0 {
                    violations.push(format!("{owner}: reach_poi target is not positive"));
                }
            }
            "defeat" => {
                if goal["model_id"].as_u64().unwrap_or(0) == 0
                    || !(1..=MAX_OBJECTIVE_COUNT).contains(&goal["count"].as_u64().unwrap_or(0))
                {
                    violations.push(format!("{owner}: defeat target and count must be positive"));
                }
            }
            "collect_item" => {
                let item = goal["item_id"]
                    .as_i64()
                    .and_then(|id| i32::try_from(id).ok())
                    .unwrap_or(0);
                let quantity_positive = any_of_leaf
                    || (1..=MAX_OBJECTIVE_COUNT).contains(&goal["quantity"].as_u64().unwrap_or(0));
                if item <= 0 || !quantity_positive {
                    violations.push(format!(
                        "{owner}: collect_item target and quantity must be positive"
                    ));
                }
                if item > 0 && !self.targets.items.contains(&item) {
                    violations.push(format!(
                                "{owner} (collect_item): item {item} is not defined by the shipped item catalogue"
                            ));
                }
            }
            "equip_item" => {
                let item = goal["definition_id"]
                    .as_i64()
                    .and_then(|id| i32::try_from(id).ok())
                    .unwrap_or(0);
                if !self.targets.items.contains(&item) {
                    violations.push(format!(
                                "{owner} (equip_item): item {item} is not defined by the shipped item catalogue"
                            ));
                }
            }
            "craft_recipe" => {
                let recipe = as_u32(&goal["recipe_id"]).unwrap_or(0);
                if !self.targets.recipes.contains(&recipe) {
                    violations.push(format!(
                                "{owner} (craft_recipe): recipe {recipe} is not defined by the shipped recipe book"
                            ));
                }
            }
            "complete_dungeon" => {
                let encounter = as_u32(&goal["encounter_id"]).unwrap_or(0);
                if !self.targets.dungeons.contains(&encounter) {
                    violations.push(format!(
                                "{owner} (complete_dungeon): encounter {encounter} is not a shipped field battle or dungeon id"
                            ));
                }
            }
            "finish_minigame" => {
                let minigame = as_u32(&goal["minigame_id"]).unwrap_or(0);
                if !self.targets.minigames.contains(&minigame) {
                    violations.push(format!(
                                "{owner} (finish_minigame): minigame {minigame} is not a shipped minigame id"
                            ));
                }
                // T787: an enabled `require` must name only facts the id's own
                // producer reports. A bound on a fact it cannot report is a quest
                // no result can ever credit.
                if let Some(require) = goal.get("require") {
                    for (field, fact) in [
                        ("min_score", MinigameFact::Score),
                        ("max_elapsed_seconds", MinigameFact::Elapsed),
                        ("max_placement", MinigameFact::Placement),
                        ("no_knockout", MinigameFact::Knockout),
                    ] {
                        let demanded = if field == "no_knockout" {
                            require[field].as_bool() == Some(true)
                        } else {
                            require.get(field).is_some_and(|value| !value.is_null())
                        };
                        if demanded && !self.targets.reports(minigame, fact) {
                            violations.push(format!(
                                        "{owner} (finish_minigame): {field} needs a fact the producer for minigame {minigame} does not report"
                                    ));
                        }
                    }
                }
            }
            "gather_resource" => {
                let resource = as_u32(&goal["resource_id"]).unwrap_or(0);
                if !self.targets.gather.contains(&resource) {
                    violations.push(format!(
                                "{owner} (gather_resource): resource {resource} is not a shipped mining/harvesting output or local gather drop"
                            ));
                }
            }
            "collect_node" => {
                let node_type = goal["node_type"].as_str().unwrap_or_default();
                if node_type.is_empty() || !self.targets.nodes.contains(node_type) {
                    violations.push(format!(
                                "{owner} (collect_node): node type {node_type:?} has no placed spawn in quest_nodes.json"
                            ));
                }
            }
            "job_level" => {
                let profile = as_u32(&goal["profile_id"]).unwrap_or(0);
                if !self.targets.profiles.contains(&profile) {
                    violations.push(format!(
                                "{owner} (job_level): profile {profile} is not declared by the loaded profile content"
                            ));
                }
            }
            "perform_pet_trick" => {
                let trick = as_u32(&goal["trick_id"]).unwrap_or(0);
                if !self.targets.pet_tricks.contains(&trick) {
                    violations.push(format!(
                        "{owner} (perform_pet_trick): trick {trick} is not a pets.json trick id"
                    ));
                }
            }
            "use_ability" => {
                let ability = as_u32(&goal["ability_id"]).unwrap_or(0);
                if !self.targets.abilities.contains(&ability) {
                    violations.push(format!(
                                "{owner} (use_ability): ability {ability} is not a job_abilities.json definition id"
                            ));
                }
                // An absent target is legal: a self cast, or a goal that counts the
                // ability on any enemy. Present, it must name a shipped definition.
                if let Some(target) = goal.get("target_actor_kind_id") {
                    let target = as_u32(target).unwrap_or(0);
                    if !self.targets.enemy_kinds.contains(&target) {
                        violations.push(format!(
                                    "{owner} (use_ability): target {target} is not an enemies.json definition id"
                                ));
                    }
                }
            }
            "use_item" => {
                let item = goal["item_id"]
                    .as_i64()
                    .and_then(|id| i32::try_from(id).ok())
                    .unwrap_or(0);
                if item <= 0 {
                    violations.push(format!("{owner}: use_item target must be positive"));
                } else if !self.targets.items.contains(&item) {
                    violations.push(format!(
                                "{owner} (use_item): item {item} is not defined by the shipped item catalogue"
                            ));
                } else if !self.targets.consumables.contains(&item) {
                    // The item bar spends only a single-use effect row; a catalogue item
                    // without one can never produce the event the step waits for.
                    violations.push(format!(
                                "{owner} (use_item): item {item} has no single-use consumable row, so the bar can never spend it"
                            ));
                }
            }
            // T790: an any-of has no target of its own; walk every alternative as the leaf
            // kind it is, so each id resolves in its own shipped set.
            "any_of" => {
                for (leaf, alternative) in goal["alternatives"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    self.objective_violations(
                        violations,
                        &format!("{owner} (any_of) alternative {leaf}"),
                        alternative,
                        true,
                    );
                }
            }
            _ => {}
        }
    }

    fn target_violation(&self, violations: &mut Vec<String>, owner: &str, target: Option<u64>) {
        let Some(target) = target.filter(|target| *target != 0) else {
            violations.push(format!("{owner}: target id is not positive"));
            return;
        };
        if !self.drawable.contains(&target) {
            violations.push(format!(
                "{owner}: target {target} is not in the production 35/2 draw set"
            ));
        }
    }

    /// Every reachable enabled quest has a title the client's i32 fields can
    /// carry and a non-empty 49/13 reward bundle. Reward bounds and the giver
    /// listing are already proved by `from_split_content`; repeating them here
    /// would only mirror the loader.
    fn reward_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.reachable_enabled() {
            // `Quest.title_string_id` is a `u32`; `definition_from_content`
            // casts it to `i32`. A value above `i32::MAX` wraps negative in the
            // 49/1, 49/3 and 49/13 title fields, so reject it before the cast.
            if !(1..=I32_MAX).contains(&quest.title_string_id) {
                violations.push(format!(
                    "quest {}: title id {} is outside 1..=i32::MAX and would wrap in the client fields",
                    quest.id, quest.title_string_id
                ));
            }
            if self
                .engine
                .empty_reward_quest_end_packet(quest.id, quest.giver_npc_id)
                .is_some()
            {
                violations.push(format!(
                    "quest {}: the 49/13 end screen encodes an empty reward bundle",
                    quest.id
                ));
            } else if self
                .engine
                .quest_end_packet(quest.id, quest.giver_npc_id)
                .is_err()
            {
                violations.push(format!(
                    "quest {}: the 49/13 end screen cannot encode the reward",
                    quest.id
                ));
            }
        }
        violations
    }
}

/// Authored NPC ids with the `quest_giver` role. `NpcInteraction` owns this
/// table but exposes no role accessor; the split quest loader deliberately
/// accepts a giver that no NPC row lists.
fn read_quest_giver_rows(root: &Path) -> BTreeSet<u64> {
    let mut ids = BTreeSet::new();
    for path in sorted_json_paths(&root.join("npcs")) {
        let document = read_json(&path);
        let records = document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: no records array", path.display()));
        for record in records {
            let npc_id = record["npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{}: NPC row has no id", path.display()));
            let quest_giver = record["roles"].as_array().is_some_and(|roles| {
                roles
                    .iter()
                    .any(|role| role.as_str() == Some("quest_giver"))
            });
            if quest_giver {
                ids.insert(npc_id);
            }
        }
    }
    ids
}

#[test]
fn every_enabled_quest_has_a_drawable_giver() {
    let catalogue = Catalogue::load(&content_dir()).expect("the shipped split catalogue loads");
    let enabled: Vec<u32> = catalogue.enabled().map(|quest| quest.id).collect();
    assert!(
        !enabled.is_empty(),
        "the shipped catalogue has enabled quests"
    );
    assert_eq!(
        enabled.iter().copied().collect::<BTreeSet<_>>().len(),
        enabled.len(),
        "enabled quest ids are unique"
    );
    let violations = catalogue.giver_violations();
    assert!(
        violations.is_empty(),
        "enabled quests without a drawable giver:\n{}",
        violations.join("\n")
    );
    println!("enabled split quest records audited: {}", enabled.len());
}

#[test]
fn every_talk_target_and_visit_point_resolves() {
    let catalogue = Catalogue::load(&content_dir()).expect("the shipped split catalogue loads");
    let violations = catalogue.target_violations();
    assert!(
        violations.is_empty(),
        "reachable enabled quest targets that do not resolve:\n{}",
        violations.join("\n")
    );
    println!(
        "enabled objective kinds checked only by the split loader's field rules: {:?}",
        catalogue.unchecked_objective_kinds()
    );
    println!(
        "graded target sets: {} battle/dungeon, {} minigame, {} item, {} recipe, {} gather id(s), {} quest node type(s)",
        catalogue.targets.dungeons.len(),
        catalogue.targets.minigames.len(),
        catalogue.targets.items.len(),
        catalogue.targets.recipes.len(),
        catalogue.targets.gather.len(),
        catalogue.targets.nodes.len()
    );

    let reachable: BTreeSet<u32> = catalogue
        .reachable_enabled()
        .into_iter()
        .map(|quest| quest.id)
        .collect();
    let unreachable = catalogue.unreachable();
    for (id, reason) in &unreachable {
        assert!(
            !reachable.contains(id),
            "unreachable quest {id} must not be graded as reachable"
        );
        println!("unreachable enabled quest {id}: {reason}");
    }
    let classified: BTreeSet<u32> = reachable
        .iter()
        .copied()
        .chain(unreachable.keys().copied())
        .collect();
    let enabled: BTreeSet<u32> = catalogue.enabled().map(|quest| quest.id).collect();
    assert_eq!(
        classified, enabled,
        "every enabled quest is classified reachable or unreachable"
    );
    println!(
        "enabled quests with resolvable targets: {} reachable, {} unreachable",
        reachable.len(),
        unreachable.len()
    );
}

/// The accepted `finish_minigame` ids are the game ids one real server emitter
/// can send (the producer table in `tests/support/quest_target_sets.rs`).
/// Pinning the shipped set makes a lost or unmodelled producer fail here instead
/// of silently certifying or rejecting a quest target. Derby 39 is the shipped
/// default package's MiniGameData row ("Wildwood Demo Derby") and 604 is the
/// first `derby_rows.json` row ("Break 'er In"); 11 (the derby type id,
/// MiniGameData row 11 the Trading Card Game) is not.
#[test]
fn shipped_minigame_producers_are_pinned() {
    let catalogue = Catalogue::load(&content_dir()).expect("the shipped split catalogue loads");
    assert_eq!(
        catalogue
            .targets
            .minigames
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        vec![
            18, 20, 31, 39, 42, 43, 44, 46, 66, 67, 77, 111, 134, 135, 136, 137, 139, 596, 597,
            598, 599, 600, 601, 604,
        ],
        "the finish_minigame producer set changed; update the producer table and this pin"
    );
    assert_eq!(
        catalogue.targets.facts.len(),
        catalogue.targets.minigames.len(),
        "every finish_minigame producer must declare its result facts"
    );
    assert!(catalogue.targets.reports(39, MinigameFact::Placement));
    assert!(!catalogue.targets.reports(39, MinigameFact::Elapsed));
    // F854: the Chef (46) and Miner (77, 134-137) movies take their score and timer
    // from the client, so no result fact is certified for them; forging (139) keeps
    // both because its movie measures them server-side.
    for client_reported in [46, 77, 134, 135, 136, 137] {
        assert!(
            !catalogue
                .targets
                .reports(client_reported, MinigameFact::Score),
            "cooking/smelting id {client_reported} must not certify a client score"
        );
        assert!(
            !catalogue
                .targets
                .reports(client_reported, MinigameFact::Elapsed),
            "cooking/smelting id {client_reported} must not certify a client timer"
        );
    }
    assert!(catalogue.targets.reports(139, MinigameFact::Score));
    assert!(catalogue.targets.reports(139, MinigameFact::Elapsed));
    assert!(!catalogue.targets.reports(139, MinigameFact::Placement));
    // Chess settles with a zero score, so only its server-measured time is a fact.
    assert!(catalogue.targets.reports(66, MinigameFact::Elapsed));
    assert!(!catalogue.targets.reports(66, MinigameFact::Score));
    assert!(!catalogue.targets.reports(599, MinigameFact::Score));
}

#[test]
fn every_reachable_enabled_quest_has_a_real_reward_and_title() {
    let catalogue = Catalogue::load(&content_dir()).expect("the shipped split catalogue loads");
    let reachable = catalogue.reachable_enabled();
    assert!(
        !reachable.is_empty(),
        "the shipped catalogue has reachable enabled quests"
    );
    let violations = catalogue.reward_violations();
    assert!(
        violations.is_empty(),
        "reachable enabled quests without a real reward/title:\n{}",
        violations.join("\n")
    );
    // The one reachable enabled record this slice fixed must stay in the graded
    // set; disabling it would otherwise make the reward gate pass silently.
    assert!(
        reachable.iter().any(|quest| quest.id == 3_130_032),
        "Postmaster Lane's one-Talk referral 3130032 must stay reachable and enabled"
    );
    for (id, reason) in catalogue.unreachable() {
        println!("reward gate skipped unreachable enabled quest {id}: {reason}");
    }
    println!(
        "reachable enabled quests audited for reward and title: {}",
        reachable.len()
    );
}

#[test]
fn a_malformed_enabled_row_is_rejected_or_reported() {
    // Control: the fixture builder itself is clean, so a reported violation is
    // caused by the planted defect and not by the fixture shape.
    let clean = fixture("clean", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert_eq!(catalogue.enabled().count(), 1);
    assert!(catalogue.giver_violations().is_empty());
    assert!(catalogue.target_violations().is_empty());
    assert!(catalogue.reward_violations().is_empty());
    assert!(catalogue.unreachable().is_empty());

    // A missing giver is reported by the giver audit, not hidden.
    let missing_giver = fixture("missing-giver", Defect::MissingGiver);
    let catalogue = Catalogue::load(missing_giver.path()).expect("a missing giver still binds");
    let violations = catalogue.giver_violations();
    assert!(
        violations
            .iter()
            .any(|violation| violation.starts_with(&format!("quest {FIXTURE_DEFECT}:"))),
        "missing giver was not reported: {violations:?}"
    );
    assert!(catalogue.target_violations().is_empty());
    assert!(catalogue.reward_violations().is_empty());

    // An empty reward on a reachable enabled quest is reported.
    let empty_reward = fixture("empty-reward", Defect::EmptyReward);
    let catalogue = Catalogue::load(empty_reward.path()).expect("the empty-reward fixture loads");
    let violations = catalogue.reward_violations();
    assert!(
        violations
            .iter()
            .any(|violation| violation.starts_with(&format!("quest {FIXTURE_DEFECT}:"))),
        "empty reward was not reported: {violations:?}"
    );
    assert!(catalogue.giver_violations().is_empty());

    // A title above i32::MAX would wrap negative in the client's title fields;
    // the loader accepts it, so the audit must catch it.
    let wrapped_title = fixture("wrapped-title", Defect::WrappedTitle);
    let catalogue = Catalogue::load(wrapped_title.path()).expect("the wrapped-title fixture loads");
    let violations = catalogue.reward_violations();
    assert!(
        violations.iter().any(|violation| {
            violation.starts_with(&format!("quest {FIXTURE_DEFECT}:"))
                && violation.contains("outside 1..=i32::MAX")
        }),
        "wrapped title was not reported: {violations:?}"
    );

    // A disabled prerequisite makes the dependant unreachable: it is recorded
    // and not failed for its (deliberately empty) reward.
    let blocked = fixture("disabled-prerequisite", Defect::DisabledPrerequisite);
    let catalogue =
        Catalogue::load(blocked.path()).expect("the disabled-prerequisite fixture loads");
    let reason = catalogue
        .unreachable()
        .get(&FIXTURE_DEFECT)
        .cloned()
        .expect("the dependant is classified unreachable");
    assert!(
        reason.contains("disabled prerequisite"),
        "unexpected unreachable reason: {reason}"
    );
    assert!(
        !catalogue
            .reachable_enabled()
            .iter()
            .any(|quest| quest.id == FIXTURE_DEFECT),
        "an unreachable quest must not enter the reward gate"
    );
    assert!(
        catalogue.reward_violations().is_empty(),
        "the unreachable dependant must not fail the reward gate: {:?}",
        catalogue.reward_violations()
    );

    // A deliberately unshipped target on the unreachable dependant is not
    // graded either: the reachability rule skips both gates for it.
    patch_fixture_objective(
        &blocked,
        FIXTURE_DEFECT,
        "finish_minigame",
        "minigame_id",
        9_999,
    );
    let catalogue = Catalogue::load(blocked.path()).expect("the patched unreachable quest loads");
    assert!(catalogue.unreachable().contains_key(&FIXTURE_DEFECT));
    assert!(
        catalogue.target_violations().is_empty(),
        "an unreachable enabled quest's targets must not be graded: {:?}",
        catalogue.target_violations()
    );

    // The production loader refuses an absent prerequisite; that rule is not
    // re-implemented by the audit.
    let missing_prerequisite = fixture("missing-prerequisite", Defect::MissingPrerequisite);
    assert!(
        Catalogue::load(missing_prerequisite.path()).is_err(),
        "an absent prerequisite must be rejected by the split loader"
    );
}

/// Rewrite one target field of one fixture quest in its scratch tree.
fn patch_fixture_objective(scratch: &Scratch, quest: u32, kind: &str, field: &str, value: u64) {
    patch_fixture_field(scratch, quest, kind, field, json!(value));
}

/// Change the fixture recipe's job (and its matching profile) so the craft producer
/// reports the facts of that job.
fn patch_fixture_recipe_job(scratch: &Scratch, job: &str, profile: i32) {
    let path = scratch.path().join("recipes.json");
    let mut document = read_json(&path);
    let record = document["records"]
        .as_array_mut()
        .expect("fixture recipe records")
        .first_mut()
        .expect("the fixture recipe exists");
    record["job"] = json!(job);
    record["job_profile_id"] = json!(profile);
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).expect("fixture recipe JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

fn patch_fixture_field(scratch: &Scratch, quest: u32, kind: &str, field: &str, value: Value) {
    let path = scratch.path().join("quests/fixture.json");
    let mut document = read_json(&path);
    let record = document["records"]
        .as_array_mut()
        .expect("fixture records")
        .iter_mut()
        .find(|record| as_u32(&record["id"]) == Some(quest))
        .unwrap_or_else(|| panic!("the fixture quest {quest} exists"));
    let goal = record["objectives"]
        .as_array_mut()
        .expect("fixture objectives")
        .iter_mut()
        .find(|goal| goal["kind"].as_str() == Some(kind))
        .unwrap_or_else(|| panic!("the fixture quest {quest} has a {kind} objective"));
    goal[field] = value;
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).expect("fixture JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

/// Replace one alternative of the fixture's `any_of` goal (T790 canaries).
fn patch_fixture_alternative(scratch: &Scratch, quest: u32, index: usize, value: Value) {
    let path = scratch.path().join("quests/fixture.json");
    let mut document = read_json(&path);
    let record = document["records"]
        .as_array_mut()
        .expect("fixture records")
        .iter_mut()
        .find(|record| as_u32(&record["id"]) == Some(quest))
        .unwrap_or_else(|| panic!("the fixture quest {quest} exists"));
    let goal = record["objectives"]
        .as_array_mut()
        .expect("fixture objectives")
        .iter_mut()
        .find(|goal| goal["kind"].as_str() == Some("any_of"))
        .unwrap_or_else(|| panic!("the fixture quest {quest} has an any_of objective"));
    goal["alternatives"]
        .as_array_mut()
        .expect("fixture alternatives")[index] = value;
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).expect("fixture JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

#[test]
fn graded_target_kinds_fail_on_a_scratch_tree() {
    // Control: every graded kind has one resolving target in the fixture's own
    // production-loader sets, so each canary below breaks loaded content rather
    // than merely the fixture shape.
    let clean = fixture("graded-targets", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert!(
        catalogue.target_violations().is_empty(),
        "the clean fixture targets must resolve: {:?}",
        catalogue.target_violations()
    );
    assert!(catalogue.targets.dungeons.contains(&FIXTURE_BATTLE));
    assert!(catalogue.targets.minigames.contains(&FIXTURE_MINIGAME));
    assert!(catalogue.targets.items.contains(&FIXTURE_ITEM));
    assert!(catalogue.targets.recipes.contains(&FIXTURE_RECIPE));
    assert!(catalogue.targets.gather.contains(&FIXTURE_GATHER_RESOURCE));
    assert!(catalogue.targets.nodes.contains(FIXTURE_NODE_TYPE));

    // Canary A: a `complete_dungeon` encounter id no shipped battle or dungeon
    // owns. The quest stays reachable, so the reward gate cannot hide it.
    let dungeon = fixture("canary-dungeon", Defect::None);
    patch_fixture_objective(
        &dungeon,
        FIXTURE_QUEST,
        "complete_dungeon",
        "encounter_id",
        99_999,
    );
    let catalogue = Catalogue::load(dungeon.path()).expect("the altered dungeon quest loads");
    assert!(
        catalogue
            .reachable_enabled()
            .iter()
            .any(|quest| quest.id == FIXTURE_QUEST),
        "the altered quest must stay reachable"
    );
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 6 (#5)"))
            && violations[0].contains("complete_dungeon")
            && violations[0].contains("99999"),
        "canary A did not name the quest, objective, kind and target: {violations:?}"
    );

    // Canary B: a `finish_minigame` id no shipped minigame owns.
    let minigame = fixture("canary-minigame", Defect::None);
    patch_fixture_objective(
        &minigame,
        FIXTURE_QUEST,
        "finish_minigame",
        "minigame_id",
        9_999,
    );
    let catalogue = Catalogue::load(minigame.path()).expect("the altered minigame quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 5 (#4)"))
            && violations[0].contains("finish_minigame")
            && violations[0].contains("9999"),
        "canary B did not name the quest, objective, kind and target: {violations:?}"
    );

    // Canary C: drop the fixture item's definition row (keeping the catalogue and
    // the recipe row); all three item target kinds that pointed at it must fail
    // instead of passing because a catalogue file merely exists.
    let item = fixture("canary-item", Defect::None);
    let catalogue_path = item.path().join("item-catalogue.json");
    let mut document = read_json(&catalogue_path);
    let definitions = document["definitions"]
        .as_array_mut()
        .expect("fixture catalogue definitions");
    let before = definitions.len();
    definitions.retain(|definition| definition["id"].as_i64() != Some(FIXTURE_ITEM as i64));
    assert_eq!(
        definitions.len() + 1,
        before,
        "the fixture item definition row was removed"
    );
    fs::write(
        &catalogue_path,
        serde_json::to_vec_pretty(&document).expect("fixture catalogue JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", catalogue_path.display()));
    let catalogue = Catalogue::load(item.path()).expect("the fixture loads without the item row");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 3, "{violations:?}");
    for kind in ["collect_item", "equip_item", "deliver_item"] {
        assert!(
            violations.iter().any(|violation| {
                violation.contains(&format!("quest {FIXTURE_QUEST}"))
                    && violation.contains(kind)
                    && violation.contains(&FIXTURE_ITEM.to_string())
            }),
            "canary C did not report {kind} against its removed item: {violations:?}"
        );
    }

    // Canary D: a recipe key absent from the loaded recipe book.
    let recipe = fixture("canary-recipe", Defect::None);
    patch_fixture_objective(&recipe, FIXTURE_QUEST, "craft_recipe", "recipe_id", 999_999);
    let catalogue = Catalogue::load(recipe.path()).expect("the altered recipe quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("craft_recipe") && violations[0].contains("999999"),
        "canary D did not report the unknown recipe: {violations:?}"
    );

    // Canary E: a gather resource absent from the mining/harvesting outputs and
    // the documented local drops. Repeating the report proves deterministic order.
    let gather = fixture("canary-gather", Defect::None);
    patch_fixture_objective(
        &gather,
        FIXTURE_QUEST,
        "gather_resource",
        "resource_id",
        999_999,
    );
    let catalogue = Catalogue::load(gather.path()).expect("the altered gather quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("gather_resource") && violations[0].contains("999999"),
        "canary E did not report the unknown gather resource: {violations:?}"
    );
    assert_eq!(
        violations,
        catalogue.target_violations(),
        "the target report order must be deterministic"
    );

    // Canary F: a finished derby credits the row it entered, not the derby type
    // id 11 (MiniGameData row 11 is the Trading Card Game). The producer table
    // loads the shipped package through `DerbyGateway::load`, so the fixture
    // carries the real `derby_entry.json` rather than a bare game id.
    let derby = fixture("canary-derby", Defect::None);
    let derby_path = derby.path().join("derby_entry.json");
    fs::copy(content_dir().join("derby_entry.json"), &derby_path)
        .unwrap_or_else(|error| panic!("{}: {error}", derby_path.display()));
    patch_fixture_objective(&derby, FIXTURE_QUEST, "finish_minigame", "minigame_id", 39);
    let catalogue = Catalogue::load(derby.path()).expect("the derby-content quest loads");
    assert_eq!(catalogue.target_violations(), Vec::<String>::new());
    patch_fixture_objective(&derby, FIXTURE_QUEST, "finish_minigame", "minigame_id", 11);
    let catalogue = Catalogue::load(derby.path()).expect("the derby-type quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("finish_minigame") && violations[0].contains("minigame 11 "),
        "canary F accepted the derby type id as a game id: {violations:?}"
    );

    // Canary G: a `job_level` profile the split loader accepts (jobs.json names it)
    // but the loaded profile content does not declare, so no rank can reach it. The
    // control is the fixture job's own profile 4, which the profile content declares.
    let ranked = fixture("job-level-ranked", Defect::None);
    open_with_job_level(&ranked, 4);
    let catalogue = Catalogue::load(ranked.path()).expect("the ranked job-level quest loads");
    assert!(catalogue.targets.profiles.contains(&4));
    assert!(
        catalogue.target_violations().is_empty(),
        "{:?}",
        catalogue.target_violations()
    );
    let unranked = fixture("canary-job-level", Defect::None);
    open_with_job_level(&unranked, 99);
    let catalogue = Catalogue::load(unranked.path()).expect("a jobs.json profile loads");
    assert!(!catalogue.targets.profiles.contains(&99));
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 9 (#0)"))
            && violations[0].contains("job_level")
            && violations[0].contains("99"),
        "canary G did not name the undeclared job-level profile: {violations:?}"
    );

    // Canary H (T503): a collect_node naming a node type quest_nodes.json does not
    // place. The clean fixture's own type resolved above, so only the patch can fail.
    let pickup = fixture("canary-collect-node", Defect::None);
    patch_fixture_field(
        &pickup,
        FIXTURE_QUEST,
        "collect_node",
        "node_type",
        json!("missing-node"),
    );
    let catalogue = Catalogue::load(pickup.path()).expect("the altered pickup quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 8 (#7)"))
            && violations[0].contains("collect_node")
            && violations[0].contains("missing-node"),
        "canary H did not report the unknown node type: {violations:?}"
    );

    // Canary I (T790): one any_of alternative names an item the catalogue does not own.
    // The clean fixture's two collect alternatives resolve above, so only the patch fails.
    let any_of_item = fixture("canary-any-of-item", Defect::None);
    patch_fixture_alternative(
        &any_of_item,
        FIXTURE_QUEST,
        1,
        json!({"kind": "collect_item", "item_id": 999_999}),
    );
    let catalogue = Catalogue::load(any_of_item.path()).expect("the altered any-of quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 13 (#11)"))
            && violations[0].contains("any_of")
            && violations[0].contains("999999"),
        "canary I did not name the any_of item alternative: {violations:?}"
    );

    // Canary J: every talk alternative must resolve in the draw set, not only the first.
    let any_of_talk = fixture("canary-any-of-talk", Defect::None);
    patch_fixture_alternative(
        &any_of_talk,
        FIXTURE_QUEST,
        0,
        json!({"kind": "talk_to_npc", "npc_id": FIXTURE_TARGET}),
    );
    patch_fixture_alternative(
        &any_of_talk,
        FIXTURE_QUEST,
        1,
        json!({"kind": "talk_to_npc", "npc_id": 999_999}),
    );
    let catalogue =
        Catalogue::load(any_of_talk.path()).expect("the altered talk any_of quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("any_of") && violations[0].contains("999999"),
        "canary J did not name the missing talk alternative: {violations:?}"
    );

    // Canary K: every gather alternative must resolve in the gather set.
    let any_of_gather = fixture("canary-any-of-gather", Defect::None);
    patch_fixture_alternative(
        &any_of_gather,
        FIXTURE_QUEST,
        0,
        json!({"kind": "gather_resource", "resource_id": FIXTURE_GATHER_RESOURCE}),
    );
    patch_fixture_alternative(
        &any_of_gather,
        FIXTURE_QUEST,
        1,
        json!({"kind": "gather_resource", "resource_id": 999_999}),
    );
    let catalogue =
        Catalogue::load(any_of_gather.path()).expect("the altered gather any_of quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("any_of") && violations[0].contains("999999"),
        "canary K did not name the missing gather alternative: {violations:?}"
    );
}

/// T788: a `perform_pet_trick` target must be a `pets.json` trick id. The gate resolves it in
/// `Targets::pet_tricks`, so an id no trick row declares is refused instead of certifying a quest
/// no perform can ever credit. A zero id is refused earlier by the production split loader
/// (`valid_objective_kind`). Canary: remove the gate arm and the unknown id passes, failing here.
#[test]
fn unknown_trick_id_is_rejected_at_load() {
    let clean = fixture("pet-trick-clean", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert!(
        catalogue.targets.pet_tricks.contains(&FIXTURE_PET_TRICK),
        "the bundled pets.json declares the fixture trick id"
    );
    assert!(
        catalogue.target_violations().is_empty(),
        "the clean pet-trick target must resolve: {:?}",
        catalogue.target_violations()
    );

    // A trick id no pets.json row declares: the quest stays reachable, so the reward gate
    // cannot hide it and only the pet-trick check can report it.
    let unknown = fixture("pet-trick-unknown", Defect::None);
    patch_fixture_objective(
        &unknown,
        FIXTURE_QUEST,
        "perform_pet_trick",
        "trick_id",
        999_999,
    );
    let catalogue = Catalogue::load(unknown.path()).expect("the altered quest still loads");
    assert!(
        catalogue
            .reachable_enabled()
            .iter()
            .any(|quest| quest.id == FIXTURE_QUEST),
        "the altered quest must stay reachable"
    );
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 10 (#8)"))
            && violations[0].contains("perform_pet_trick")
            && violations[0].contains("999999"),
        "the unknown pet trick was not reported: {violations:?}"
    );

    // The loader's own sanity bound (non-zero) is the production refusal, not the audit's.
    let zero = fixture("pet-trick-zero", Defect::None);
    patch_fixture_objective(&zero, FIXTURE_QUEST, "perform_pet_trick", "trick_id", 0);
    assert!(
        Catalogue::load(zero.path()).is_err(),
        "from_split_content must reject a zero trick id"
    );
}

/// T788: every authored `perform_pet_trick` step, including the ten disabled Pet Trainer records
/// the reachability gate skips, names a bundled `pets.json` trick id. The enabled gate only
/// resolves reachable enabled quests, so a typo in a disabled record would otherwise ship.
#[test]
fn every_authored_pet_trick_step_names_a_bundled_trick() {
    let root = content_dir();
    let targets = Targets::load(&root, &reference_content()).expect("the target sets load");
    let mut steps = 0;
    for quest in read_quests(&root) {
        for goal in &quest.objectives {
            if goal["kind"].as_str() != Some("perform_pet_trick") {
                continue;
            }
            steps += 1;
            let trick = as_u32(&goal["trick_id"]).unwrap_or(0);
            assert!(
                targets.pet_tricks.contains(&trick),
                "quest {} objective {}: trick {trick} is not a bundled pets.json trick id",
                quest.id,
                goal["id"].as_u64().unwrap_or(0)
            );
        }
    }
    assert!(
        steps >= 13,
        "the authored pet-trick steps shrank to {steps}; the disabled records are no longer covered"
    );
}

/// T787: an enabled `finish_minigame` may only require result facts its own producer
/// reports. The fixture's target is a recipe's game id (a craft producer: score and
/// elapsed), never a placement or a knock-out; the derby canary's id reports score,
/// placement and knock-outs but no elapsed time. Both directions are checked, so a
/// gate that ignored the producer table would fail either accepted bound. Canary:
/// treat an unreportable field as acceptable and the placement case passes instead.
#[test]
fn requirement_needs_a_fact_the_producer_reports() {
    // The fixture's finish_minigame target resolves to a craft win.
    let craft = fixture("requirement-craft", Defect::None);
    patch_fixture_field(
        &craft,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"min_score": 20000, "max_elapsed_seconds": 90}),
    );
    let catalogue = Catalogue::load(craft.path()).expect("the craft requirement loads");
    assert!(
        catalogue.target_violations().is_empty(),
        "craft reports the score and elapsed bounds: {:?}",
        catalogue.target_violations()
    );

    patch_fixture_field(
        &craft,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"max_placement": 1}),
    );
    let catalogue = Catalogue::load(craft.path()).expect("the craft requirement loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 5 (#4)"))
            && violations[0].contains("max_placement")
            && violations[0].contains("does not report"),
        "the craft producer cannot report a placement: {violations:?}"
    );

    // F854: the cooking and smelting movies take the score and the timer from the
    // client, so neither id reports a result fact. The bound is refused instead of
    // certifying a client number as a result; the same fixture was accepted above as
    // a blacksmith recipe, so only the job patch can fail here.
    let client_craft = fixture("requirement-client-craft", Defect::None);
    patch_fixture_recipe_job(&client_craft, "chef", 45);
    patch_fixture_field(
        &client_craft,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"min_score": 20000, "max_elapsed_seconds": 90}),
    );
    let catalogue = Catalogue::load(client_craft.path()).expect("the chef requirement loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 2, "{violations:?}");
    for field in ["min_score", "max_elapsed_seconds"] {
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains(field) && violation.contains("does not report")),
            "a chef recipe must not certify {field}: {violations:?}"
        );
    }

    // F854: a chess row settles with a zero score, so a `min_score` bound on its game
    // id can never be met even though its elapsed time is real.
    let chess = fixture("requirement-chess", Defect::None);
    fs::copy(
        content_dir().join(chess_service::CONTENT_FILE),
        chess.path().join(chess_service::CONTENT_FILE),
    )
    .expect("the shipped chess content copies into the fixture");
    patch_fixture_objective(&chess, FIXTURE_QUEST, "finish_minigame", "minigame_id", 66);
    patch_fixture_field(
        &chess,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"min_score": 1}),
    );
    let catalogue = Catalogue::load(chess.path()).expect("the chess requirement loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("min_score") && violations[0].contains("minigame 66"),
        "a chess row must not certify a score: {violations:?}"
    );

    // The derby's own producer reports placement, score and knock-outs, but not a
    // finish time (its elapsed fact is never claimed). The producer table loads the
    // shipped package through `DerbyGateway::load`, so the fixture carries the real
    // `derby_entry.json` rather than a bare game id.
    let derby = fixture("requirement-derby", Defect::None);
    let path = derby.path().join("derby_entry.json");
    fs::copy(content_dir().join("derby_entry.json"), &path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    patch_fixture_objective(&derby, FIXTURE_QUEST, "finish_minigame", "minigame_id", 39);
    patch_fixture_field(
        &derby,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"max_placement": 1, "no_knockout": true}),
    );
    let catalogue = Catalogue::load(derby.path()).expect("the derby requirement loads");
    assert!(
        catalogue.target_violations().is_empty(),
        "the derby reports placement and knock-outs: {:?}",
        catalogue.target_violations()
    );

    patch_fixture_field(
        &derby,
        FIXTURE_QUEST,
        "finish_minigame",
        "require",
        json!({"max_elapsed_seconds": 60}),
    );
    let catalogue = Catalogue::load(derby.path()).expect("the derby requirement loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("max_elapsed_seconds") && violations[0].contains("minigame 39"),
        "the derby reports no elapsed time: {violations:?}"
    );
}

/// T786: a `deliver_item` step's item must be a shipped item definition, so a hand-over can
/// never name an item no player can hold. The clean fixture's own delivery step resolves; the
/// canary patches only its item id, so the quest stays reachable and only this gate can report it.
#[test]
fn unknown_delivery_item_is_rejected_at_load() {
    let clean = fixture("deliver-clean", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert!(
        catalogue.target_violations().is_empty(),
        "the clean delivery target must resolve: {:?}",
        catalogue.target_violations()
    );

    let unknown = fixture("deliver-unknown", Defect::None);
    patch_fixture_objective(&unknown, FIXTURE_QUEST, "deliver_item", "item_id", 999_999);
    let catalogue =
        Catalogue::load(unknown.path()).expect("the altered delivery quest still loads");
    assert!(
        catalogue
            .reachable_enabled()
            .iter()
            .any(|quest| quest.id == FIXTURE_QUEST),
        "the altered quest must stay reachable"
    );
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 14 (#12)"))
            && violations[0].contains("deliver_item")
            && violations[0].contains("999999"),
        "the unknown delivery item was not reported: {violations:?}"
    );

    // The production split loader's own bound is the refusal, not the audit's.
    let zero = fixture("deliver-zero", Defect::None);
    patch_fixture_objective(&zero, FIXTURE_QUEST, "deliver_item", "item_id", 0);
    assert!(
        Catalogue::load(zero.path()).is_err(),
        "from_split_content must reject a zero delivery item"
    );
}

/// T786 review finding 3: a second enabled delivery of the same item to the same NPC would be
/// credited by the first hand-over too, so the audit refuses it. The canary appends a copy of the
/// fixture's own delivery step to its quest.
#[test]
fn duplicate_delivery_target_is_rejected() {
    let scratch = fixture("deliver-duplicate", Defect::None);
    let path = scratch.path().join("quests/fixture.json");
    let mut document = read_json(&path);
    let objectives = document["records"]
        .as_array_mut()
        .expect("fixture records")
        .iter_mut()
        .find(|record| as_u32(&record["id"]) == Some(FIXTURE_QUEST))
        .expect("the fixture quest exists")["objectives"]
        .as_array_mut()
        .expect("fixture objectives");
    let mut copy = objectives
        .iter()
        .find(|goal| goal["kind"] == "deliver_item")
        .expect("the fixture quest has a delivery step")
        .clone();
    copy["id"] = json!(15);
    objectives.push(copy);
    fs::write(
        &path,
        serde_json::to_vec_pretty(&document).expect("fixture JSON"),
    )
    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));

    let catalogue = Catalogue::load(scratch.path()).expect("the altered fixture loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains("objective 15") && violations[0].contains("already takes"),
        "{violations:?}"
    );
}

/// Open the fixture quest with a `job_level` goal on `profile` (the first step, as the
/// split loader requires) and give that profile a jobs.json row, so the loader accepts
/// it and only the profile content can refuse it.
fn open_with_job_level(scratch: &Scratch, profile: u32) {
    let path = scratch.path().join("quests/fixture.json");
    let mut document = read_json(&path);
    document["records"]
        .as_array_mut()
        .expect("fixture records")
        .iter_mut()
        .find(|record| as_u32(&record["id"]) == Some(FIXTURE_QUEST))
        .expect("the fixture quest exists")["objectives"]
        .as_array_mut()
        .expect("fixture objectives")
        .insert(
            0,
            json!({
                "kind": "job_level",
                "id": 9,
                "profile_id": profile,
                "level": 5,
                "source": [{"kind": "reconstructed", "evidence": "(R) F576 scratch job_level goal"}],
            }),
        );
    fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
    let jobs_path = scratch.path().join("jobs.json");
    let mut jobs = read_json(&jobs_path);
    let rows = jobs["records"].as_array_mut().expect("fixture jobs");
    if !rows
        .iter()
        .any(|row| as_u32(&row["profile_id"]) == Some(profile))
    {
        let mut row = rows[0].clone();
        row["id"] = json!(format!("fixture_job_{profile}"));
        row["profile_id"] = json!(profile);
        row["quest_ids"] = json!([]);
        rows.push(row);
    }
    fs::write(&jobs_path, serde_json::to_vec_pretty(&jobs).unwrap()).unwrap();
}

/// `from_split_content` (`valid_objective_kind`) is the only authority for the
/// visit bound; this drives that production loader instead of testing a copied
/// helper, so a loosened or missing loader bound fails here.
#[test]
fn visit_geometry_bounds_are_enforced() {
    let valid = fixture(
        "visit-valid",
        Defect::VisitGeometry {
            position: [0.0, 0.0, 0.0],
            radius: 5.0,
        },
    );
    let catalogue = Catalogue::load(valid.path()).expect("valid visit geometry loads");
    assert_eq!(catalogue.enabled().count(), 2);

    for (name, position, radius) in [
        ("visit-zero-radius", [0.0, 0.0, 0.0], 0.0),
        ("visit-large-radius", [0.0, 0.0, 0.0], 1000.1),
        ("visit-position-bound", [1_000_001.0, 0.0, 0.0], 5.0),
    ] {
        let bad = fixture(name, Defect::VisitGeometry { position, radius });
        assert!(
            Catalogue::load(bad.path()).is_err(),
            "{name}: from_split_content must reject out-of-range visit geometry"
        );
    }
}

/// T789: a `use_ability` step must name a `job_abilities.json` definition id and, when it binds a
/// target, an `enemies.json` `definition_id`. The gate resolves both in
/// `Targets::{abilities, enemy_kinds}`, so a typo is refused instead of certifying a step no
/// accepted cast can credit. A zero id is refused earlier by the production split loader
/// (`valid_objective_kind`). Canary: patch either id and the clean fixture's empty report fails.
#[test]
fn unknown_ability_is_rejected_at_load() {
    let clean = fixture("use-ability-clean", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert!(catalogue.targets.abilities.contains(&FIXTURE_ABILITY));
    assert!(catalogue.targets.enemy_kinds.contains(&FIXTURE_ENEMY));
    assert!(
        catalogue.target_violations().is_empty(),
        "{:?}",
        catalogue.target_violations()
    );

    // An ability id no job_abilities.json row declares. The quest stays reachable, so only the
    // use_ability arm can report it.
    let unknown_ability = fixture("use-ability-unknown", Defect::None);
    patch_fixture_objective(
        &unknown_ability,
        FIXTURE_QUEST,
        "use_ability",
        "ability_id",
        999_999,
    );
    let catalogue = Catalogue::load(unknown_ability.path()).expect("the altered quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 11 (#9)"))
            && violations[0].contains("use_ability")
            && violations[0].contains("999999"),
        "the unknown ability was not reported: {violations:?}"
    );

    // A target that is not an enemies.json definition id (a field actor with no catalogue row).
    let unknown_target = fixture("use-ability-target", Defect::None);
    patch_fixture_objective(
        &unknown_target,
        FIXTURE_QUEST,
        "use_ability",
        "target_actor_kind_id",
        999_999,
    );
    let catalogue = Catalogue::load(unknown_target.path()).expect("the altered quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 11 (#9)"))
            && violations[0].contains("use_ability")
            && violations[0].contains("target 999999"),
        "the unknown ability target was not reported: {violations:?}"
    );
}

/// T789: a `use_item` step must name a shipped catalogue item with a single-use consumable
/// row, because the bar only credits an item it actually spent. The fixture's own item
/// resolves, so only the patches can fail.
#[test]
fn unknown_use_item_is_rejected_at_load() {
    let clean = fixture("use-item-clean", Defect::None);
    let catalogue = Catalogue::load(clean.path()).expect("the clean fixture loads");
    assert!(catalogue.targets.items.contains(&FIXTURE_USE_ITEM));
    assert!(catalogue.targets.consumables.contains(&FIXTURE_USE_ITEM));
    assert!(
        catalogue.target_violations().is_empty(),
        "{:?}",
        catalogue.target_violations()
    );

    let unknown = fixture("use-item-unknown", Defect::None);
    patch_fixture_objective(&unknown, FIXTURE_QUEST, "use_item", "item_id", 999_999);
    let catalogue = Catalogue::load(unknown.path()).expect("the altered quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 12 (#10)"))
            && violations[0].contains("use_item")
            && violations[0].contains("999999"),
        "the unknown use item was not reported: {violations:?}"
    );

    // Gate canary: a real catalogue item whose definition has no single-use consumable row is
    // bindable nowhere, so the bar can never spend it and the step could never tick.
    let non_consumable = fixture("use-item-non-consumable", Defect::None);
    patch_fixture_objective(
        &non_consumable,
        FIXTURE_QUEST,
        "use_item",
        "item_id",
        FIXTURE_ITEM as u64,
    );
    let catalogue = Catalogue::load(non_consumable.path()).expect("the altered quest loads");
    let violations = catalogue.target_violations();
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].contains(&format!("quest {FIXTURE_QUEST} objective 12 (#10)"))
            && violations[0].contains("use_item")
            && violations[0].contains("single-use consumable"),
        "a catalogued item with no consumable row was not reported: {violations:?}"
    );
}

/// T789: every authored `use_ability` step, including the disabled records the enabled gate
/// skips (3010011 still waits behind its hollow prerequisite chain), names a shipped ability and
/// enemy definition. Without this, a typo in a disabled record would ship unseen.
#[test]
fn every_authored_use_goal_resolves_in_the_shipped_tables() {
    let root = content_dir();
    let targets = Targets::load(&root, &reference_content()).expect("the target sets load");
    let mut ability_steps = 0;
    for quest in read_quests(&root) {
        for goal in &quest.objectives {
            match goal["kind"].as_str() {
                Some("use_ability") => {
                    ability_steps += 1;
                    let ability = as_u32(&goal["ability_id"]).unwrap_or(0);
                    assert!(
                        targets.abilities.contains(&ability),
                        "quest {} objective {}: ability {ability} is not a job_abilities.json definition id",
                        quest.id,
                        goal["id"].as_u64().unwrap_or(0)
                    );
                    if let Some(target) = goal.get("target_actor_kind_id") {
                        let target = as_u32(target).unwrap_or(0);
                        assert!(
                            targets.enemy_kinds.contains(&target),
                            "quest {} objective {}: target {target} is not an enemies.json definition id",
                            quest.id,
                            goal["id"].as_u64().unwrap_or(0)
                        );
                    }
                }
                Some("use_item") => {
                    let item = goal["item_id"]
                        .as_i64()
                        .and_then(|id| i32::try_from(id).ok())
                        .unwrap_or(0);
                    assert!(
                        targets.items.contains(&item),
                        "quest {} objective {}: item {item} is not a shipped catalogue item",
                        quest.id,
                        goal["id"].as_u64().unwrap_or(0)
                    );
                    assert!(
                        targets.consumables.contains(&item),
                        "quest {} objective {}: item {item} has no single-use consumable row, so the bar can never spend it",
                        quest.id,
                        goal["id"].as_u64().unwrap_or(0)
                    );
                }
                _ => {}
            }
        }
    }
    assert!(
        ability_steps >= 1,
        "the authored use_ability steps shrank to {ability_steps}; the disabled records are no longer covered"
    );
}

/// T895: a record that carries objectives but is switched off says why in its own source notes
/// ("stays disabled" or the W110 "DISABLED" tag), so a reviewer sees what is missing instead of
/// guessing. An unmarked `enabled` means on, so only an explicit `false` counts.
#[test]
fn every_disabled_record_with_objectives_says_why_it_stays_off() {
    let missing: Vec<u32> = read_quests(&content_dir())
        .into_iter()
        .filter(|quest| !quest.enabled && !quest.objectives.is_empty())
        .filter(|quest| {
            !quest
                .source_notes
                .iter()
                .any(|note| note.contains("stays disabled") || note.contains("DISABLED"))
        })
        .map(|quest| quest.id)
        .collect();
    assert!(
        missing.is_empty(),
        "disabled records with objectives and no stay-off reason: {missing:?}"
    );
}
