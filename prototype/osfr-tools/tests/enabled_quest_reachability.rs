//! T314/F343: structural reachability gate for the enabled split quest catalogue.
//!
//! The shipped content tree is read through the production loaders:
//! `QuestEngine::from_split_content` (definitions, giver binding, reward bounds,
//! the enabled set and the 49/13 end-screen encoder) and `world::NpcVisibility`
//! (the 35/2 draw set through `teleport_targets` and `content_key`). A raw JSON
//! pass only reads what those loaders do not expose: the `enabled` flag,
//! prerequisite ids, objective target fields, the authored title id and the
//! authored NPC `quest_giver` role.
//!
//! Resolved objective kinds: `talk_to_npc`, `interact_actor`, `visit_position`,
//! `reach_poi`, `defeat` and `collect_item`. Other kinds (`gather_resource`,
//! `complete_dungeon`, `finish_minigame`, `client_reported`, `craft_recipe`,
//! `equip_item`) are only bounded by the loader's field rules, not proved to
//! exist; the target test prints them so the gap stays visible.
//!
//! A quest is *reachable* when every prerequisite, directly or transitively,
//! exists and is enabled. An enabled quest behind a disabled prerequisite is
//! reported and excluded from the reward gate; it is not turned on here.
//!
//! No new wire. A reachable quest's reward is proved through the production
//! 49/13 encoder (`quest_end_packet`, reader `0x00c7b990`: actor u64, quest, end
//! text, animation dword, reward bundle `0x008e7930`, trailing float). A
//! drawable giver is what the production 35/2 path sends (AddNpc reader
//! `0x00921a10`); the 26/8 click constructor `0x008b6bc0` resolves the same key
//! through `NpcVisibility::content_key`.
//!
//! The shared `content_dir`/`read_json` helpers and the scratch fixture live in
//! `tests/support/`.

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/quest_catalogue_fixture.rs"]
mod quest_catalogue_fixture;

use npc_quest_chain::{content_dir, read_json};
use osfr_tools::{
    quest_engine::{CharacterEligibility, QuestEngine},
    world::NpcVisibility,
};
use quest_catalogue_fixture::{Defect, FIXTURE_DEFECT, fixture};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
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

fn as_u32(value: &Value) -> Option<u32> {
    value.as_u64().and_then(|value| u32::try_from(value).ok())
}

/// A finite xyz transform component.
fn finite_position(value: &Value) -> bool {
    value.as_array().is_some_and(|axis| {
        axis.len() == 3
            && axis
                .iter()
                .all(|value| value.as_f64().is_some_and(f64::is_finite))
    })
}

/// The quest engine's `visit_position` bounds (`valid_objective_kind`).
fn valid_visit_geometry(position: &Value, radius: &Value) -> bool {
    finite_position(position)
        && position.as_array().is_some_and(|axis| {
            axis.iter().all(|value| {
                value
                    .as_f64()
                    .is_some_and(|value| value.abs() <= 1_000_000.0)
            })
        })
        && radius
            .as_f64()
            .is_some_and(|radius| radius.is_finite() && (0.1..=1000.0).contains(&radius))
}

/// One split quest record. Only fields the production loaders do not expose are
/// parsed here; the loader remains the authority for rewards, bounds and
/// enabled/disabled behaviour.
#[derive(Debug)]
struct RawQuest {
    id: u32,
    enabled: bool,
    giver_npc_id: u64,
    /// Authored title id before `definition_from_content` casts it to `i32`.
    title_string_id: u64,
    prerequisite_quest_ids: Vec<u32>,
    objectives: Vec<Value>,
}

struct Catalogue {
    engine: QuestEngine,
    quests: Vec<RawQuest>,
    by_id: BTreeMap<u32, usize>,
    /// Content NPC ids the production 35/2 path can send.
    drawable: BTreeSet<u64>,
    /// Authored NPC ids whose `roles` array carries `quest_giver`.
    giver_role_npcs: BTreeSet<u64>,
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
        const CHECKED: [&str; 6] = [
            "talk_to_npc",
            "interact_actor",
            "visit_position",
            "reach_poi",
            "defeat",
            "collect_item",
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

    /// Every enabled objective target this gate knows how to resolve: talk and
    /// interact actors are in the production draw set; visit geometry is finite
    /// and bounded; kill and item ids/counts are positive.
    fn target_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.enabled() {
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
                    "visit_position" => {
                        if !valid_visit_geometry(&goal["position"], &goal["radius"]) {
                            violations.push(format!(
                                "{owner}: visit geometry is not finite with a positive radius"
                            ));
                        }
                    }
                    "reach_poi" => {
                        if goal["poi_id"].as_u64().unwrap_or(0) == 0 {
                            violations.push(format!("{owner}: reach_poi target is not positive"));
                        }
                        if goal.get("position").is_some()
                            && !valid_visit_geometry(&goal["position"], &goal["radius"])
                        {
                            violations.push(format!(
                                "{owner}: reach geometry is not finite with a positive radius"
                            ));
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
                        if goal["item_id"].as_i64().unwrap_or(0) <= 0
                            || !(1..=MAX_OBJECTIVE_COUNT)
                                .contains(&goal["quantity"].as_u64().unwrap_or(0))
                        {
                            violations.push(format!(
                                "{owner}: collect_item target and quantity must be positive"
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

fn sorted_json_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
                .path()
        })
        .collect();
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    paths
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

fn read_quests(root: &Path) -> Vec<RawQuest> {
    let mut quests = Vec::new();
    for path in sorted_json_paths(&root.join("quests")) {
        let document = read_json(&path);
        let records = document["records"]
            .as_array()
            .unwrap_or_else(|| panic!("{}: no records array", path.display()));
        for record in records {
            let id = as_u32(&record["id"])
                .unwrap_or_else(|| panic!("{}: invalid quest id", path.display()));
            let enabled = match record.get("enabled") {
                None => true,
                Some(value) => value.as_bool().unwrap_or_else(|| {
                    panic!("{}: quest {id} enabled is not boolean", path.display())
                }),
            };
            let giver_npc_id = record["giver_npc_id"]
                .as_u64()
                .unwrap_or_else(|| panic!("{}: quest {id} has no giver", path.display()));
            let prerequisite_quest_ids = record["prerequisite_quest_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| {
                    as_u32(value).unwrap_or_else(|| {
                        panic!("{}: quest {id} prerequisite id is invalid", path.display())
                    })
                })
                .collect();
            let objectives = record["objectives"]
                .as_array()
                .cloned()
                .unwrap_or_else(|| panic!("{}: quest {id} has no objectives", path.display()));
            quests.push(RawQuest {
                id,
                enabled,
                giver_npc_id,
                title_string_id: record["title_string_id"].as_u64().unwrap_or(0),
                prerequisite_quest_ids,
                objectives,
            });
        }
    }
    quests
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
        "enabled quest targets that do not resolve:\n{}",
        violations.join("\n")
    );
    println!(
        "enabled objective kinds checked only by the split loader's field rules: {:?}",
        catalogue.unchecked_objective_kinds()
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

    // The production loader refuses an absent prerequisite; that rule is not
    // re-implemented by the audit.
    let missing_prerequisite = fixture("missing-prerequisite", Defect::MissingPrerequisite);
    assert!(
        Catalogue::load(missing_prerequisite.path()).is_err(),
        "an absent prerequisite must be rejected by the split loader"
    );
}

#[test]
fn visit_geometry_bounds_are_enforced() {
    assert!(valid_visit_geometry(&json!([0.0, 0.0, 0.0]), &json!(5.0)));
    assert!(valid_visit_geometry(
        &json!([1_000_000.0, -1_000_000.0, 0.0]),
        &json!(1000.0)
    ));
    for position in [
        json!([1_000_001.0, 0.0, 0.0]),
        json!([null, 0.0, 0.0]),
        json!([0.0, 0.0]),
        json!([0.0, 0.0, "x"]),
    ] {
        assert!(
            !valid_visit_geometry(&position, &json!(5.0)),
            "position {position} must be refused"
        );
    }
    for radius in [
        json!(0.0),
        json!(-1.0),
        json!(0.05),
        json!(1000.1),
        json!("5"),
        json!(null),
    ] {
        assert!(
            !valid_visit_geometry(&json!([0.0, 0.0, 0.0]), &radius),
            "radius {radius} must be refused"
        );
    }
}
