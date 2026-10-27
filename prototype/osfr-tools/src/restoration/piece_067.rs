//! Piece 067 - quest prerequisite engine: source-defined requirements, branch
//! availability and invalid acceptance rejection.
//!
//! ORIGINAL facts implemented here (provenance in
//! `notes/Research/Pieces/067 Quest prerequisite engine.md`):
//!
//! * The revision-pinned quest pages (`lab/references/freerealms-wiki/
//!   quest-pages.json`) state requirements verbatim:
//!   - "Player must complete Quest:X in order to do Quest:Y",
//!   - "... and have an active Membership ...",
//!   - "... waiting at least one day after completion of Quest:X ...",
//!   - "Player must own a usable glide-trainable ride ...",
//!   - "Player must achieve at least one Level 8 or higher Combat Job",
//!   - "... only available during ... Community Events".
//!   the private recovery checker `piece_067.py` re-derives all 36 requirement rows from that
//!   corpus and fails on drift; the two Farnum's Farm rows are embedded below.
//! * First-party client locale 401549 "In order to accept more quests, you must
//!   first become a member." and 71311 "Sorry, you don't meet the requirements
//!   for that action!" anchor the accept-time gate and its rejection message.
//!   The client `QuestData_DS` carries `MembersOnly` and `JobLevel` fields.
//!
//! COMMUNITY RECONSTRUCTION (labelled, not first-party): the field shape
//! `PrerequisiteQuestId` / `ExcludesQuestIds` and the ordering "exclusions are
//! checked before the prerequisite" come from FreeRealms-Legacy
//! (`Sanctuary.Game/Resources/Definitions/QuestDefinition.cs`, AGPL-3.0, pinned
//! revision). No first-party exclusion table was recovered, so branch exclusions
//! stay a reconstructed policy with the documented starter-branch rationale.
//!
//! NOT CLAIMED: any native accept/turn-in run, a numeric quest id for the wiki
//! pages, or a complete historical requirement table. The corpus carries 52
//! quest pages against tens of thousands of original quests, and six prerequisite
//! pages are absent from it; those stay explicit gaps.
//!
//! RECONSTRUCTED decisions labelled here:
//! * A row may only name a prerequisite that is present in the table or listed in
//!   the caller's explicit external list; an undeclared reference is a load error
//!   so a typo cannot silently disable a gate.
//! * Self-prerequisites and self-exclusions are rejected. Longer prerequisite
//!   cycles are not scanned: they simply keep both quests blocked forever, which
//!   is a visible data bug rather than an unsafe acceptance.
//! * `Availability` reports every unmet condition at once, ordered as the engine
//!   checks them: branch conflicts, prerequisite completion, then the remaining
//!   requirements in source order. `accept` rejects with the first of those.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

const DAY_MS: u64 = 86_400_000;

/// A source-defined requirement beyond the single prerequisite quest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    /// "Player must ... have an active Membership ..." (quest page), corroborated
    /// by the first-party accept-time gate locale 401549.
    MembershipActive,
    /// "Player must achieve at least one Level N or higher Combat Job".
    CombatJobLevelAtLeast { level: u32 },
    /// "Player must own a usable glide-trainable ride ...".
    OwnsGlideTrainableRide,
    /// "... waiting at least N day(s) after completion of" the prerequisite.
    PrerequisiteCooldownDays { days: u32 },
    /// "... only available during the <name> ... event".
    EventWindow { name: &'static str },
}

/// One quest definition row: the recovered prerequisite, branch exclusions and
/// further requirements, with its provenance string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestRow<K> {
    pub quest: K,
    /// The single recovered prerequisite ("Player must complete Quest:X ...").
    pub prereq: Option<K>,
    /// Reconstructed branch exclusions: taking any of these rules the quest out.
    pub excludes: Vec<K>,
    /// Further source-defined requirements beyond the prerequisite.
    pub requirements: Vec<Requirement>,
    /// Where this row comes from, e.g. "wiki rev 27815 (2013-07-12)".
    pub evidence: &'static str,
}

/// A player's progress for one quest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestRecord {
    pub accepted: bool,
    /// Completion time in milliseconds; `Some` means completed.
    pub completed_ms: Option<u64>,
}

/// Everything the engine needs to evaluate a requirement set.
#[derive(Clone, Debug)]
pub struct PlayerFacts<K: Ord> {
    pub quests: BTreeMap<K, QuestRecord>,
    pub membership_active: bool,
    pub best_combat_job_level: u32,
    pub owns_glide_trainable_ride: bool,
    pub active_event: Option<String>,
    /// Caller clock in milliseconds, used for the completion-cooldown rule.
    pub now_ms: u64,
}

impl<K: Ord> PlayerFacts<K> {
    pub fn new(now_ms: u64) -> Self {
        Self {
            quests: BTreeMap::new(),
            membership_active: false,
            best_combat_job_level: 0,
            owns_glide_trainable_ride: false,
            active_event: None,
            now_ms,
        }
    }

    /// Records a quest the player is on but has not completed.
    pub fn record_active(&mut self, quest: K) {
        self.quests.insert(
            quest,
            QuestRecord {
                accepted: true,
                completed_ms: None,
            },
        );
    }

    /// Records a completed quest. A quest outside the table (an unresolved
    /// prerequisite page, for example) can still be recorded and satisfy a gate.
    pub fn record_completed(&mut self, quest: K, completed_ms: u64) {
        self.quests.insert(
            quest,
            QuestRecord {
                accepted: true,
                completed_ms: Some(completed_ms),
            },
        );
    }
}

/// Why a quest is not offered, in the order the engine checks conditions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block<K> {
    /// The player already took the other branch of this choice.
    BranchTaken {
        taken: K,
    },
    Unmet(UnmetRequirement<K>),
}

/// A single unmet source-defined requirement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnmetRequirement<K> {
    PrerequisiteNotCompleted {
        prerequisite: K,
    },
    PrerequisiteCooldown {
        required_days: u32,
        remaining_ms: u64,
    },
    MembershipRequired,
    CombatJobLevel {
        required: u32,
        actual: u32,
    },
    GlideTrainableRideRequired,
    EventNotActive {
        name: &'static str,
    },
}

/// Availability of one quest for one player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability<K> {
    /// No definition row exists for this quest.
    Unknown,
    Active,
    Completed,
    Available,
    Blocked(Vec<Block<K>>),
}

/// Why an accept request was rejected. Rejections never mutate player state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection<K> {
    UnknownQuest(K),
    AlreadyActive(K),
    AlreadyCompleted(K),
    BranchTaken { quest: K, taken: K },
    Unmet(UnmetRequirement<K>),
}

/// A definition table that cannot be trusted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError<K> {
    DuplicateQuest(K),
    SelfPrerequisite(K),
    SelfExclusion(K),
    DuplicateExclusion { quest: K, excluded: K },
    UnknownPrerequisite { quest: K, prerequisite: K },
    UnknownExclusion { quest: K, excluded: K },
    InvalidRequirement { quest: K, requirement: Requirement },
}

fn valid_requirement(requirement: &Requirement) -> bool {
    match requirement {
        Requirement::MembershipActive | Requirement::OwnsGlideTrainableRide => true,
        Requirement::CombatJobLevelAtLeast { level } => *level > 0,
        Requirement::PrerequisiteCooldownDays { days } => *days > 0,
        Requirement::EventWindow { name } => !name.trim().is_empty(),
    }
}

/// A validated set of quest requirement rows.
#[derive(Clone, Debug)]
pub struct PrerequisiteTable<K> {
    rows: BTreeMap<K, QuestRow<K>>,
    external: BTreeSet<K>,
}

impl<K: Clone + Ord + Debug> PrerequisiteTable<K> {
    /// Validates and indexes definition rows. `external_prerequisites` lists
    /// prerequisite ids documented as unresolved (their page is missing from the
    /// recovered corpus); every other referenced id must exist in `rows`.
    pub fn new(
        rows: Vec<QuestRow<K>>,
        external_prerequisites: Vec<K>,
    ) -> Result<Self, TableError<K>> {
        let external: BTreeSet<K> = external_prerequisites.into_iter().collect();
        let mut table: BTreeMap<K, QuestRow<K>> = BTreeMap::new();
        for row in rows {
            if table.contains_key(&row.quest) {
                return Err(TableError::DuplicateQuest(row.quest));
            }
            if row.prereq.as_ref() == Some(&row.quest) {
                return Err(TableError::SelfPrerequisite(row.quest));
            }
            let mut seen = BTreeSet::new();
            for excluded in &row.excludes {
                if *excluded == row.quest {
                    return Err(TableError::SelfExclusion(row.quest));
                }
                if !seen.insert(excluded.clone()) {
                    return Err(TableError::DuplicateExclusion {
                        quest: row.quest.clone(),
                        excluded: excluded.clone(),
                    });
                }
            }
            for requirement in &row.requirements {
                if !valid_requirement(requirement) {
                    return Err(TableError::InvalidRequirement {
                        quest: row.quest.clone(),
                        requirement: *requirement,
                    });
                }
            }
            table.insert(row.quest.clone(), row);
        }
        for row in table.values() {
            if let Some(prerequisite) = &row.prereq {
                if !table.contains_key(prerequisite) && !external.contains(prerequisite) {
                    return Err(TableError::UnknownPrerequisite {
                        quest: row.quest.clone(),
                        prerequisite: prerequisite.clone(),
                    });
                }
            }
            for excluded in &row.excludes {
                if !table.contains_key(excluded) {
                    return Err(TableError::UnknownExclusion {
                        quest: row.quest.clone(),
                        excluded: excluded.clone(),
                    });
                }
            }
        }
        Ok(Self {
            rows: table,
            external,
        })
    }

    pub fn row(&self, quest: &K) -> Option<&QuestRow<K>> {
        self.rows.get(quest)
    }

    /// Prerequisite ids documented as unresolved (their quest page is absent
    /// from the recovered corpus).
    pub fn unresolved_prerequisites(&self) -> impl Iterator<Item = &K> {
        self.external.iter()
    }

    pub fn availability(&self, facts: &PlayerFacts<K>, quest: &K) -> Availability<K> {
        let Some(row) = self.rows.get(quest) else {
            return Availability::Unknown;
        };
        if let Some(record) = facts.quests.get(quest) {
            if record.completed_ms.is_some() {
                return Availability::Completed;
            }
            if record.accepted {
                return Availability::Active;
            }
        }
        let blocks = self.blocks(facts, row);
        if blocks.is_empty() {
            Availability::Available
        } else {
            Availability::Blocked(blocks)
        }
    }

    /// Accepts a quest when every source-defined requirement holds. Every
    /// rejection leaves player state untouched.
    pub fn accept(&self, facts: &mut PlayerFacts<K>, quest: &K) -> Result<(), Rejection<K>> {
        let Some(row) = self.rows.get(quest) else {
            return Err(Rejection::UnknownQuest(quest.clone()));
        };
        if let Some(record) = facts.quests.get(quest) {
            if record.completed_ms.is_some() {
                return Err(Rejection::AlreadyCompleted(quest.clone()));
            }
            if record.accepted {
                return Err(Rejection::AlreadyActive(quest.clone()));
            }
        }
        if let Some(block) = self.blocks(facts, row).into_iter().next() {
            return match block {
                Block::BranchTaken { taken } => Err(Rejection::BranchTaken {
                    quest: quest.clone(),
                    taken,
                }),
                Block::Unmet(requirement) => Err(Rejection::Unmet(requirement)),
            };
        }
        facts.record_active(quest.clone());
        Ok(())
    }

    fn blocks(&self, facts: &PlayerFacts<K>, row: &QuestRow<K>) -> Vec<Block<K>> {
        let mut blocks = Vec::new();
        // Recovered community ordering: exclusions are checked before the
        // prerequisite, so a taken branch rules the quest out either way.
        for excluded in &row.excludes {
            if facts.quests.contains_key(excluded) {
                blocks.push(Block::BranchTaken {
                    taken: excluded.clone(),
                });
            }
        }
        let mut prerequisite_completed_ms = None;
        if let Some(prerequisite) = &row.prereq {
            match facts
                .quests
                .get(prerequisite)
                .and_then(|record| record.completed_ms)
            {
                Some(completed_ms) => prerequisite_completed_ms = Some(completed_ms),
                None => blocks.push(Block::Unmet(UnmetRequirement::PrerequisiteNotCompleted {
                    prerequisite: prerequisite.clone(),
                })),
            }
        }
        for requirement in &row.requirements {
            match requirement {
                Requirement::PrerequisiteCooldownDays { days } => {
                    if let Some(completed_ms) = prerequisite_completed_ms {
                        let required_ms = (*days as u64).saturating_mul(DAY_MS);
                        let elapsed_ms = facts.now_ms.saturating_sub(completed_ms);
                        if elapsed_ms < required_ms {
                            blocks.push(Block::Unmet(UnmetRequirement::PrerequisiteCooldown {
                                required_days: *days,
                                remaining_ms: required_ms - elapsed_ms,
                            }));
                        }
                    }
                }
                Requirement::MembershipActive => {
                    if !facts.membership_active {
                        blocks.push(Block::Unmet(UnmetRequirement::MembershipRequired));
                    }
                }
                Requirement::CombatJobLevelAtLeast { level } => {
                    if facts.best_combat_job_level < *level {
                        blocks.push(Block::Unmet(UnmetRequirement::CombatJobLevel {
                            required: *level,
                            actual: facts.best_combat_job_level,
                        }));
                    }
                }
                Requirement::OwnsGlideTrainableRide => {
                    if !facts.owns_glide_trainable_ride {
                        blocks.push(Block::Unmet(UnmetRequirement::GlideTrainableRideRequired));
                    }
                }
                Requirement::EventWindow { name } => {
                    if facts.active_event.as_deref() != Some(*name) {
                        blocks.push(Block::Unmet(UnmetRequirement::EventNotActive { name }));
                    }
                }
            }
        }
        blocks
    }
}

/// The two recovered Farnum's Farm opening rows. Page revisions are pinned in
/// the evidence strings; the prerequisite page of the first row is absent from
/// the corpus and stays an explicit gap.
pub const OPENING_QUEST_PREREQUISITES: [QuestRow<&'static str>; 2] = [
    QuestRow {
        quest: "Quest:Your First Weapon",
        prereq: Some("Quest:Learn to Mine & Smelt Ore"),
        excludes: Vec::new(),
        requirements: Vec::new(),
        evidence: "wiki rev 27815 (2013-07-12)",
    },
    QuestRow {
        quest: "Quest:Fighting off the Pack",
        prereq: Some("Quest:Your First Weapon"),
        excludes: Vec::new(),
        requirements: Vec::new(),
        evidence: "wiki rev 28955 (2013-07-21)",
    },
];

/// Prerequisite ids referenced by the opening rows whose pages are missing from
/// the recovered corpus.
pub const OPENING_UNRESOLVED_PREREQUISITES: [&str; 1] = ["Quest:Learn to Mine & Smelt Ore"];

/// Builds the pinned opening-arc table. The constants above are asserted by
/// `tests/piece_067.rs`; the private recovery checker `piece_067.py` re-derives them from the
/// corpus, so a drift in either place fails a real check.
pub fn opening_table() -> PrerequisiteTable<&'static str> {
    PrerequisiteTable::new(
        OPENING_QUEST_PREREQUISITES.to_vec(),
        OPENING_UNRESOLVED_PREREQUISITES.to_vec(),
    )
    .expect("pinned opening rows are valid")
}
