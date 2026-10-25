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
//! Resolved objective kinds: `talk_to_npc`, `interact_actor`, `reach_poi`,
//! `defeat`, `collect_item`, `equip_item`, `craft_recipe`, `complete_dungeon`,
//! `finish_minigame`, `gather_resource`, `collect_node` and `job_level`. The item,
//! recipe, dungeon, minigame, gather and quest-pickup kinds additionally resolve
//! their target ids in the id sets owned by `tests/support/quest_target_sets.rs`,
//! which names the owning loader and every `FinishMinigame` emitter. A
//! `collect_node` node type must have at least one placed spawn in
//! `quest_nodes.json` (`harvest_nodes::GatheringNodes::load`, T503). A `job_level`
//! profile must be declared by the loaded profile content (`Targets::profiles`);
//! its level bound (1..=20) and its jobs.json profile are the split loader's. A
//! `finish_minigame` id no server path emits is refused instead of being
//! certified, and `shipped_minigame_producers_are_pinned` fails if the accepted
//! set drifts.
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
    quest_engine::{CharacterEligibility, QuestEngine},
    world::NpcVisibility,
};
use quest_catalogue_fixture::{
    Defect, FIXTURE_BATTLE, FIXTURE_DEFECT, FIXTURE_GATHER_RESOURCE, FIXTURE_ITEM,
    FIXTURE_MINIGAME, FIXTURE_NODE_TYPE, FIXTURE_QUEST, FIXTURE_RECIPE, Scratch, fixture,
};
use quest_records::{RawQuest, as_u32, read_quests, sorted_json_paths};
use quest_target_sets::Targets;
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
        const CHECKED: [&str; 12] = [
            "talk_to_npc",
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
    /// kind with a shipped id table resolves in it. Visit geometry bounds are
    /// enforced by `from_split_content` and deliberately not repeated here.
    /// Unreachable enabled records are skipped, as the reachability rule says.
    fn target_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.reachable_enabled() {
            for (index, goal) in quest.objectives.iter().enumerate() {
                let objective = goal["id"].as_u64().unwrap_or(0);
                let owner = format!("quest {} objective {objective} (#{index})", quest.id);
                match goal["kind"].as_str().unwrap_or_default() {
                    "talk_to_npc" => {
                        self.target_violation(&mut violations, &owner, goal["npc_id"].as_u64());
                    }
                    "interact_actor" => {
                        self.target_violation(&mut violations, &owner, goal["actor_guid"].as_u64());
                    }
                    "reach_poi" => {
                        if goal["poi_id"].as_u64().unwrap_or(0) == 0 {
                            violations.push(format!("{owner}: reach_poi target is not positive"));
                        }
                    }
                    "defeat" => {
                        if goal["model_id"].as_u64().unwrap_or(0) == 0
                            || !(1..=MAX_OBJECTIVE_COUNT)
                                .contains(&goal["count"].as_u64().unwrap_or(0))
                        {
                            violations
                                .push(format!("{owner}: defeat target and count must be positive"));
                        }
                    }
                    "collect_item" => {
                        let item = goal["item_id"]
                            .as_i64()
                            .and_then(|id| i32::try_from(id).ok())
                            .unwrap_or(0);
                        if item <= 0
                            || !(1..=MAX_OBJECTIVE_COUNT)
                                .contains(&goal["quantity"].as_u64().unwrap_or(0))
                        {
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
                    _ => {}
                }
            }
        }
        violations
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
/// package's MiniGameData row ("Wildwood Demo Derby") that a finished match
/// credits; 11 (its type id, MiniGameData row 11 the Trading Card Game) is not.
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
            598, 599, 600, 601,
        ],
        "the finish_minigame producer set changed; update the producer table and this pin"
    );
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
    // the recipe row); both item target kinds that pointed at it must fail
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
    assert_eq!(violations.len(), 2, "{violations:?}");
    for kind in ["collect_item", "equip_item"] {
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

    // Canary F: a finished derby credits the package's `game_id`, not the
    // derby type id 11 (MiniGameData row 11 is the Trading Card Game).
    let derby = fixture("canary-derby", Defect::None);
    let derby_path = derby.path().join("derby_entry.json");
    fs::write(&derby_path, br#"{"schema_version":1,"game_id":39}"#)
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
