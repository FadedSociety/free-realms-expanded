//! T314: structural reachability gate for the enabled split quest catalogue.
//!
//! The shipped content tree is read through the production loaders:
//! `QuestEngine::from_split_content` (definitions, giver listing, reward bounds
//! and the 49/13 end-screen encoder) and `world::NpcVisibility::load` (the 35/2
//! draw set and the 26/8 content-key map). A raw JSON pass only enumerates the
//! `enabled` records and objective target fields those loaders do not expose.
//!
//! A quest is *reachable* when every prerequisite, directly or transitively,
//! exists and is enabled. An enabled quest behind a disabled prerequisite is
//! reported and excluded from the reward gate; it is not turned on here.
//!
//! No new wire. A reachable quest's reward is proved through the production
//! 49/13 encoder (`quest_end_packet`, reader `0x00c7b990`: actor u64, quest, end
//! text, animation dword, reward bundle `0x008e7930`, trailing float). A
//! drawable giver is a `legacy_data` regional row with a positive model/name or
//! an `npc_spawns_extra.json` `content_key`, exactly what the 35/2 AddNpc reader
//! `0x00921a10` consumes; the 26/8 click reader `0x008b6bc0` resolves it through
//! `NpcVisibility::content_key`.

use osfr_tools::{
    quest_engine::{CharacterEligibility, QuestEngine},
    world::NpcVisibility,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

const CHARACTER: u64 = 7;
const I32_MAX: u64 = i32::MAX as u64;
const MAX_OBJECTIVE_COUNT: u64 = 1_000_000;

fn content_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn eligibility() -> CharacterEligibility {
    CharacterEligibility {
        player_level: 20,
        job_levels: BTreeMap::new(),
        member: true,
        nonmember_quest_limit: 20,
        member_quest_limit: 20,
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn read_json(path: &Path) -> io::Result<Value> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|error| invalid(format!("{}: {error}", path.display())))
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

/// One `enabled` split quest record. Only the fields the production loaders do
/// not expose are parsed here; everything else is the loader's answer.
#[derive(Debug)]
struct RawQuest {
    id: u32,
    enabled: bool,
    giver_npc_id: u64,
    prerequisite_quest_ids: Vec<u32>,
    objectives: Vec<Value>,
    rewards: Vec<Value>,
}

struct Catalogue {
    engine: QuestEngine,
    quests: Vec<RawQuest>,
    by_id: BTreeMap<u32, usize>,
    npcs: BTreeMap<u64, Value>,
    drawable: BTreeSet<u64>,
    /// NPC content ids whose only placement is a `poi_anchor` row and which have
    /// no drawable extra spawn.
    anchor_only: BTreeSet<u64>,
    job_quests: BTreeSet<u32>,
    job_profiles: BTreeSet<i32>,
}

impl Catalogue {
    fn load(root: &Path) -> io::Result<Self> {
        let engine = QuestEngine::from_split_content(root, CHARACTER, eligibility(), None)?;
        let visibility = NpcVisibility::load(root)?;
        let npcs = read_npc_rows(root)?;
        let drawable = drawable_sites(root, &visibility, &npcs)?;
        let anchor_only = npcs
            .iter()
            .filter(|(npc_id, row)| {
                row["placement"].as_str() == Some("poi_anchor") && !drawable.contains(npc_id)
            })
            .map(|(npc_id, _)| *npc_id)
            .collect();
        let quests = read_quests(root)?;
        let mut by_id = BTreeMap::new();
        for (index, quest) in quests.iter().enumerate() {
            if by_id.insert(quest.id, index).is_some() {
                return Err(invalid(format!("duplicate split quest id {}", quest.id)));
            }
        }
        let jobs = read_json(&root.join("jobs.json"))?;
        let records = jobs["records"]
            .as_array()
            .ok_or_else(|| invalid("split jobs.json has no records array"))?;
        let mut job_quests = BTreeSet::new();
        let mut job_profiles = BTreeSet::new();
        for job in records {
            job_quests.extend(
                job["quest_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(as_u32),
            );
            job_profiles.extend(
                job["profile_id"]
                    .as_i64()
                    .and_then(|profile| i32::try_from(profile).ok()),
            );
        }
        Ok(Self {
            engine,
            quests,
            by_id,
            npcs,
            drawable,
            anchor_only,
            job_quests,
            job_profiles,
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

    /// Every enabled record has a drawable giver that the NPC content defines
    /// with the `quest_giver` role (the production Talk-offer rule), the engine
    /// kept it (objectives are not empty), and no giver is a bare `poi_anchor`
    /// placement with no drawable extra spawn.
    fn giver_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.enabled() {
            if !self.engine.is_enabled(quest.id) {
                violations.push(format!(
                    "quest {}: enabled record is not offered by the engine (disabled or empty objectives)",
                    quest.id
                ));
            }
            let Some(row) = self.npcs.get(&quest.giver_npc_id) else {
                violations.push(format!(
                    "quest {}: giver {} has no npcs/*.json row",
                    quest.id, quest.giver_npc_id
                ));
                continue;
            };
            // `npc_interaction::has_role` reads the same `roles` array; a giver
            // without this string never builds the 26/9 Talk offer.
            if !row["roles"].as_array().is_some_and(|roles| {
                roles
                    .iter()
                    .any(|role| role.as_str() == Some("quest_giver"))
            }) {
                violations.push(format!(
                    "quest {}: giver {} has no quest_giver role",
                    quest.id, quest.giver_npc_id
                ));
            }
            if !self.drawable.contains(&quest.giver_npc_id) {
                let anchor = if self.anchor_only.contains(&quest.giver_npc_id) {
                    " (poi_anchor-only row with no drawable extra spawn)"
                } else {
                    ""
                };
                violations.push(format!(
                    "quest {}: giver {}{anchor} is not drawable by NpcVisibility",
                    quest.id, quest.giver_npc_id
                ));
            }
        }
        violations
    }

    /// Every enabled objective target resolves: talk/interact actors are
    /// drawable content NPCs; visit geometry is finite and bounded; kill and
    /// item ids/counts are positive; prerequisites exist.
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
            for prerequisite in &quest.prerequisite_quest_ids {
                if self.quest(*prerequisite).is_none() {
                    violations.push(format!(
                        "quest {}: prerequisite {prerequisite} does not exist",
                        quest.id
                    ));
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
        if !self.npcs.contains_key(&target) {
            violations.push(format!("{owner}: target {target} has no npcs/*.json row"));
        } else if !self.drawable.contains(&target) {
            violations.push(format!("{owner}: target {target} is not drawable"));
        }
    }

    /// Every reachable enabled quest has a positive title, a non-empty reward
    /// inside the loader bounds, its giver's listing, the jobs.json union and an
    /// encodable non-empty 49/13 end screen.
    fn reward_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();
        for quest in self.reachable_enabled() {
            if self.engine.title_string_id(quest.id).is_none() {
                violations.push(format!(
                    "quest {}: title is not positive or missing",
                    quest.id
                ));
            }
            let mut coins = 0u64;
            let mut job_xp = 0u64;
            let mut grant = false;
            for (index, reward) in quest.rewards.iter().enumerate() {
                let owner = format!("quest {} reward {index}", quest.id);
                let row_coins = reward["coins"].as_u64().unwrap_or(0);
                coins = coins.saturating_add(row_coins);
                grant |= row_coins > 0;
                for item in reward["items"].as_array().into_iter().flatten() {
                    let definition_id = item["definition_id"].as_i64().unwrap_or(0);
                    let count = item["count"].as_u64().unwrap_or(0);
                    if definition_id <= 0 || !(1..=I32_MAX).contains(&count) {
                        violations.push(format!(
                            "{owner}: item grant {definition_id} x {count} is outside the loader bounds"
                        ));
                    } else {
                        grant = true;
                    }
                }
                for (profile, amount) in reward["job_xp"].as_object().into_iter().flatten() {
                    let known = profile
                        .parse::<i32>()
                        .ok()
                        .is_some_and(|profile| profile > 0 && self.job_profiles.contains(&profile));
                    if !known {
                        violations.push(format!(
                            "{owner}: job XP profile {profile:?} is not a positive jobs.json profile"
                        ));
                        continue;
                    }
                    let amount = amount.as_u64().unwrap_or(0);
                    if amount > I32_MAX {
                        violations.push(format!(
                            "{owner}: job XP amount {amount} exceeds the reward bundle i32 field"
                        ));
                        continue;
                    }
                    grant |= amount > 0;
                    job_xp = job_xp.saturating_add(amount);
                }
            }
            if !grant {
                violations.push(format!(
                    "quest {}: reachable quest has no reward grant (coins, items and job_xp are zero or empty)",
                    quest.id
                ));
            }
            if coins > I32_MAX {
                violations.push(format!(
                    "quest {}: coin total {coins} exceeds the loader bound",
                    quest.id
                ));
            }
            if job_xp > I32_MAX {
                violations.push(format!(
                    "quest {}: job XP total {job_xp} exceeds the loader bound",
                    quest.id
                ));
            }
            if !self
                .engine
                .quest_ids_for_giver(quest.giver_npc_id)
                .any(|id| id == quest.id)
            {
                violations.push(format!(
                    "quest {}: giver {} does not list the quest",
                    quest.id, quest.giver_npc_id
                ));
            }
            if !self.job_quests.contains(&quest.id) {
                violations.push(format!(
                    "quest {}: jobs.json does not list the quest in its quest_ids union",
                    quest.id
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

fn read_npc_rows(root: &Path) -> io::Result<BTreeMap<u64, Value>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(root.join("npcs"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<_>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    let mut rows = BTreeMap::new();
    for path in paths {
        let document = read_json(&path)?;
        let records = document["records"]
            .as_array()
            .ok_or_else(|| invalid(format!("{}: no records array", path.display())))?;
        for record in records {
            let npc_id = record["npc_id"]
                .as_u64()
                .ok_or_else(|| invalid(format!("{}: NPC row has no id", path.display())))?;
            if rows.insert(npc_id, record.clone()).is_some() {
                return Err(invalid(format!("duplicate split NPC id {npc_id}")));
            }
        }
    }
    Ok(rows)
}

/// The production draw set: positioned `legacy_data` rows with a positive model
/// or name id, plus extra spawns whose `content_key` names a content NPC. Each
/// candidate is checked against `NpcVisibility` itself so the audit cannot claim
/// an actor the 35/2 path never sends.
fn drawable_sites(
    root: &Path,
    visibility: &NpcVisibility,
    npcs: &BTreeMap<u64, Value>,
) -> io::Result<BTreeSet<u64>> {
    let mut sites = BTreeSet::new();
    for (npc_id, row) in npcs {
        if row["placement"].as_str() != Some("legacy_data") || !finite_position(&row["position"]) {
            continue;
        }
        let model_id = row["model_id"].as_u64().unwrap_or(0);
        let name_id = row["name_string_id"].as_u64().unwrap_or(0);
        if model_id == 0 && name_id == 0 {
            continue;
        }
        if visibility.content_key(*npc_id) != Some(*npc_id) {
            return Err(invalid(format!(
                "legacy_data NPC {npc_id} is not in the production draw set"
            )));
        }
        sites.insert(*npc_id);
    }
    let extras = read_json(&root.join("npc_spawns_extra.json"))?;
    let extras = extras["records"]
        .as_array()
        .ok_or_else(|| invalid("npc_spawns_extra.json has no records array"))?;
    for row in extras {
        let Some(content_key) = row["content_key"].as_u64().filter(|key| *key != 0) else {
            continue;
        };
        // Mirror the loader's own skip list: no captured body, no unshipped
        // knocker model and no non-finite position.
        if !row["capture_match"].is_null()
            || row["model_file_name"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("knocker_m_boss.adr"))
            || !finite_position(&row["position"])
        {
            continue;
        }
        let spawn_guid = row["spawn_guid"]
            .as_u64()
            .ok_or_else(|| invalid("extra NPC spawn has no guid"))?;
        if visibility.content_key(spawn_guid) != Some(content_key) {
            return Err(invalid(format!(
                "extra spawn {spawn_guid} with content key {content_key} is not in the production draw set"
            )));
        }
        sites.insert(content_key);
    }
    Ok(sites)
}

fn read_quests(root: &Path) -> io::Result<Vec<RawQuest>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(root.join("quests"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<_>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    let mut quests = Vec::new();
    for path in paths {
        let document = read_json(&path)?;
        let records = document["records"]
            .as_array()
            .ok_or_else(|| invalid(format!("{}: no records array", path.display())))?;
        for record in records {
            let id = as_u32(&record["id"])
                .ok_or_else(|| invalid(format!("{}: invalid quest id", path.display())))?;
            let enabled = match record.get("enabled") {
                None => true,
                Some(value) => value.as_bool().ok_or_else(|| {
                    invalid(format!(
                        "{}: quest {id} enabled is not boolean",
                        path.display()
                    ))
                })?,
            };
            let giver_npc_id = record["giver_npc_id"]
                .as_u64()
                .ok_or_else(|| invalid(format!("{}: quest {id} has no giver", path.display())))?;
            let prerequisite_quest_ids = record["prerequisite_quest_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| {
                    as_u32(value).ok_or_else(|| {
                        invalid(format!(
                            "{}: quest {id} prerequisite id is invalid",
                            path.display()
                        ))
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            let objectives = record["objectives"].as_array().cloned().ok_or_else(|| {
                invalid(format!("{}: quest {id} has no objectives", path.display()))
            })?;
            let rewards = record["rewards"]
                .as_array()
                .cloned()
                .ok_or_else(|| invalid(format!("{}: quest {id} has no rewards", path.display())))?;
            quests.push(RawQuest {
                id,
                enabled,
                giver_npc_id,
                prerequisite_quest_ids,
                objectives,
                rewards,
            });
        }
    }
    if quests.is_empty() {
        return Err(invalid("split quest catalogue is empty"));
    }
    Ok(quests)
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
    for id in &enabled {
        assert!(
            catalogue.engine.giver_npc_id(*id).is_some(),
            "enabled quest {id} was silently dropped by QuestEngine"
        );
    }
    let violations = catalogue.giver_violations();
    assert!(
        violations.is_empty(),
        "enabled quests without a drawable giver:\n{}",
        violations.join("\n")
    );
    let anchor_only: Vec<u32> = catalogue
        .enabled()
        .filter(|quest| catalogue.anchor_only.contains(&quest.giver_npc_id))
        .map(|quest| quest.id)
        .collect();
    assert!(
        anchor_only.is_empty(),
        "enabled quests name a poi_anchor-only giver: {anchor_only:?}"
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

    // A reachable quest's direct prerequisites are enabled; reachability already
    // proves the transitive chain, so a disabled ancestor cannot hide here.
    for quest in catalogue.reachable_enabled() {
        for prerequisite in &quest.prerequisite_quest_ids {
            let row = catalogue.quest(*prerequisite).unwrap_or_else(|| {
                panic!("quest {} prerequisite {prerequisite} is missing", quest.id)
            });
            assert!(
                row.enabled,
                "reachable quest {} has disabled prerequisite {prerequisite}",
                quest.id
            );
        }
    }
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

// -- scratch fixtures -------------------------------------------------------

const FIXTURE_GIVER: u64 = 9_000_100_001;
const FIXTURE_TARGET: u64 = 9_000_100_002;
const FIXTURE_MISSING_GIVER: u64 = 9_000_100_099;
const FIXTURE_MISSING_PREREQUISITE: u32 = 4_000_100_098;
const FIXTURE_QUEST: u32 = 9_100_001;
const FIXTURE_DEFECT: u32 = 9_100_002;
const FIXTURE_BLOCKER: u32 = 9_100_003;
const FIXTURE_TITLE: u32 = 700_001;

#[derive(Clone, Copy)]
enum Defect {
    None,
    MissingGiver,
    NonFiniteVisit,
    EmptyReward,
    DisabledPrerequisite,
    MissingPrerequisite,
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("t314-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("quests")).expect("fixture quests dir");
        fs::create_dir_all(dir.join("npcs")).expect("fixture npcs dir");
        Self(dir)
    }

    fn path(&self) -> &Path {
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
    giver_npc_id: u64,
    objectives: Value,
    rewards: Value,
    prerequisite_quest_ids: Value,
) -> Value {
    json!({
        "id": id,
        "title_string_id": FIXTURE_TITLE,
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

fn fixture_reward() -> Value {
    json!([{"coins": 5, "items": [], "job_xp": {"4": 5}, "source": fixture_source()}])
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

fn fixture(name: &str, defect: Defect) -> Scratch {
    let scratch = Scratch::new(name);
    let root = scratch.path();
    let mut quests = vec![fixture_quest(
        FIXTURE_QUEST,
        FIXTURE_GIVER,
        fixture_talk_objective(),
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
                FIXTURE_MISSING_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::NonFiniteVisit => {
            let objectives = json!([{
                "kind": "visit_position",
                "id": 1,
                "zone_id": 1,
                "position": [1e300, 0.0, 0.0],
                "radius": 5.0,
                "source": fixture_source(),
            }]);
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_GIVER,
                objectives,
                fixture_reward(),
                json!([]),
            ));
            quest_ids.push(FIXTURE_DEFECT);
        }
        Defect::EmptyReward => {
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
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
            quests.push(fixture_quest(
                FIXTURE_DEFECT,
                FIXTURE_GIVER,
                fixture_talk_objective(),
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
                FIXTURE_GIVER,
                fixture_talk_objective(),
                fixture_reward(),
                json!([FIXTURE_MISSING_PREREQUISITE]),
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

    // The production loader already refuses an absent prerequisite and invalid
    // visit geometry; the audit's matching checks are the defence in depth.
    let missing_prerequisite = fixture("missing-prerequisite", Defect::MissingPrerequisite);
    assert!(
        Catalogue::load(missing_prerequisite.path()).is_err(),
        "an absent prerequisite must be rejected by the split loader"
    );
    let non_finite_visit = fixture("non-finite-visit", Defect::NonFiniteVisit);
    assert!(
        Catalogue::load(non_finite_visit.path()).is_err(),
        "non-finite visit geometry must be rejected by the split loader"
    );
}
