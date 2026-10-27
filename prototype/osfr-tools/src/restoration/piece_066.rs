//! Piece 066: the historical quest catalogue for the selected Free Realms
//! opening arc (Briarwood Caverns -> Farnum Farmstead -> Cobblestone Village ->
//! Royal Palace).
//!
//! The catalogue is recovered evidence, not a reconstructed quest system. This
//! module reads `notes/Research/original-quest-coverage.json` (produced from the
//! pinned supplied/community/wiki sources, see
//! the private recovery checker `piece_066.py`) and gives the lead a typed, validated view of
//! what is original, what is community evidence and what is missing:
//!
//! * ORIGINAL: 25 supplied numeric quest records, 13 region-1 journal hub
//!   registrations, 6 region-1 stickers, 52 revision-pinned wiki pages and the
//!   bounded local engine definitions.
//! * RECONSTRUCTED: the 17 reserved `910xxx` local opening stages; every one
//!   keeps `original_quest_id` null.
//! * GAPS: hub registrations and most stickers carry no recovered title,
//!   objective, prerequisite, NPC or reward. Those stay listed as missing
//!   originals instead of being invented.
//!
//! `validate()` separates integrity errors (duplicate ids, a local stage that
//! claims a binding, an enabled blocked claim) from documented warnings (missing
//! originals, disabled claims, ids shared across corpora, unresolved wiki
//! prerequisites). Nothing in this module grants money, items or quest state.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Schema version of the catalogue document this module understands.
pub const SCHEMA_VERSION: i64 = 1;

pub const QUEST_SOURCE_SUPPLIED_JOURNAL: &str = "supplied-journal";
pub const QUEST_SOURCE_COMMUNITY_JOURNAL: &str = "community-journal";
pub const QUEST_SOURCE_WIKI: &str = "wiki";
pub const QUEST_SOURCE_LOCAL_OPENING: &str = "local-opening";

/// The catalogue lives in the checkout; the lab copy stays private.
pub const CATALOGUE_RELATIVE_PATH: &str = "notes/Research/original-quest-coverage.json";

/// Opening sticker ids whose titles are recovered but whose objectives and
/// rewards are not. Recorded here so a later edit cannot silently drop them.
pub const MISSING_REGION_ONE_STICKERS: [i64; 3] = [2572, 2573, 2587];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogueError {
    Io(String),
    Json(String),
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogueError::Io(detail) => write!(formatter, "catalogue io error: {detail}"),
            CatalogueError::Json(detail) => write!(formatter, "catalogue parse error: {detail}"),
        }
    }
}

impl std::error::Error for CatalogueError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(untagged)]
pub enum QuestId {
    Int(i64),
    Text(String),
}

impl QuestId {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            QuestId::Int(value) => Some(*value),
            QuestId::Text(_) => None,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            QuestId::Int(_) => None,
            QuestId::Text(value) => Some(value),
        }
    }

    pub fn display(&self) -> String {
        match self {
            QuestId::Int(value) => value.to_string(),
            QuestId::Text(value) => value.clone(),
        }
    }
}

impl fmt::Display for QuestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.display())
    }
}

impl From<i64> for QuestId {
    fn from(value: i64) -> Self {
        QuestId::Int(value)
    }
}

impl From<&str> for QuestId {
    fn from(value: &str) -> Self {
        QuestId::Text(value.to_string())
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ObjectiveRow {
    #[serde(default)]
    pub id: Option<QuestId>,
    #[serde(default)]
    pub name_locale_id: Option<i64>,
    #[serde(default)]
    pub name_locale: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub required: Option<i64>,
    #[serde(default)]
    pub progress: Option<i64>,
}

/// A recorded reward reference. The catalogue keeps references only; no item
/// definition corpus is recovered, so nothing here may be paid out directly.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct RewardRefRow {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub item_definition: Option<i64>,
    #[serde(default)]
    pub quantity: Option<i64>,
    #[serde(default)]
    pub amount: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestRow {
    pub source: String,
    pub id: QuestId,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub title_locale_id: Option<i64>,
    #[serde(default)]
    pub title_locale: Option<String>,
    #[serde(default)]
    pub description_locale_id: Option<i64>,
    #[serde(default)]
    pub description_locale: Option<String>,
    #[serde(default)]
    pub objectives: Vec<ObjectiveRow>,
    #[serde(default)]
    pub reward_refs: Vec<RewardRefRow>,
    #[serde(default)]
    pub hub_id: Option<i64>,
    #[serde(default)]
    pub display_order: Option<i64>,
    #[serde(default)]
    pub sticker_id: Option<i64>,
    #[serde(default)]
    pub region_id: Option<i64>,
    #[serde(default)]
    pub original_quest_id: Option<i64>,
    #[serde(default)]
    pub revision_id: Option<i64>,
    #[serde(default)]
    pub revision_timestamp: Option<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub previous: Option<String>,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub npc: Option<String>,
    #[serde(default)]
    pub goals: Vec<String>,
    #[serde(default)]
    pub coin_reward: Option<i64>,
    #[serde(default)]
    pub quest_series: Option<String>,
    #[serde(default)]
    pub opening_arc: Option<bool>,
    #[serde(default)]
    pub evidence_class: Option<String>,
    #[serde(default)]
    pub implementation_state: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EngineDefinition {
    pub id: i64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub original: bool,
    #[serde(default)]
    pub enemy_definitions: Vec<i64>,
    #[serde(default)]
    pub goal_texts: Vec<String>,
    #[serde(default)]
    pub coins: Option<i64>,
    #[serde(default)]
    pub items: Vec<(i64, i64)>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BlockedClaim {
    pub quest_id: i64,
    #[serde(default)]
    pub claim: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub objective: Option<String>,
    #[serde(default)]
    pub required: Option<i64>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MissingOriginal {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub quest_id: Option<i64>,
    #[serde(default)]
    pub quest_title: Option<String>,
    #[serde(default)]
    pub hub_id: Option<i64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub referenced_by: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

impl MissingOriginal {
    fn label(&self) -> String {
        if let Some(quest_id) = self.quest_id {
            return quest_id.to_string();
        }
        if let Some(title) = &self.quest_title {
            return title.clone();
        }
        if let Some(title) = &self.title {
            return title.clone();
        }
        "?".to_string()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CrossCheck {
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct CatalogueSource {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LocalBinding {
    #[serde(default)]
    pub bound: i64,
    #[serde(default)]
    pub total: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct NativeAcceptance {
    #[serde(default)]
    pub local_stages_native_accepted: i64,
    #[serde(default)]
    pub local_stages_code_only: i64,
    #[serde(default)]
    pub original_quests_native_accepted: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Coverage {
    #[serde(default)]
    pub local_binding: LocalBinding,
    #[serde(default)]
    pub native_acceptance: NativeAcceptance,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuestCatalogue {
    pub schema_version: i64,
    #[serde(default)]
    pub generated_utc: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub sources: Vec<CatalogueSource>,
    #[serde(default)]
    pub corpus_totals: BTreeMap<String, i64>,
    #[serde(default)]
    pub quests: Vec<QuestRow>,
    #[serde(default)]
    pub engine_definitions: Vec<EngineDefinition>,
    #[serde(default)]
    pub coverage: Coverage,
    #[serde(default)]
    pub blocked_claims: Vec<BlockedClaim>,
    #[serde(default)]
    pub missing_originals: Vec<MissingOriginal>,
    #[serde(default)]
    pub cross_checks: Vec<CrossCheck>,
    #[serde(default)]
    pub limitations: Vec<String>,
}

/// Validation result: errors are integrity violations, warnings are the
/// explicit gaps the catalogue is expected to carry.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogueReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CatalogueReport {
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

pub struct ObjectiveRef<'a> {
    pub source: &'a str,
    pub quest_id: String,
    pub text: Option<&'a str>,
    pub required: Option<i64>,
    pub objective: &'a ObjectiveRow,
}

pub struct RewardRowRef<'a> {
    pub source: &'a str,
    pub quest_id: String,
    pub reward: &'a RewardRefRow,
}

pub struct PrerequisiteLink<'a> {
    pub page: &'a str,
    pub prerequisite: &'a str,
}

/// Absolute path of the workspace catalogue (`CARGO_MANIFEST_DIR/../../`).
pub fn catalogue_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(CATALOGUE_RELATIVE_PATH)
}

impl QuestCatalogue {
    pub fn from_json_str(text: &str) -> Result<Self, CatalogueError> {
        serde_json::from_str(text).map_err(|error| CatalogueError::Json(error.to_string()))
    }

    pub fn load(path: &Path) -> Result<Self, CatalogueError> {
        let text = std::fs::read_to_string(path)
            .map_err(|error| CatalogueError::Io(format!("{}: {error}", path.display())))?;
        Self::from_json_str(&text)
    }

    pub fn load_workspace() -> Result<Self, CatalogueError> {
        Self::load(&catalogue_path())
    }

    pub fn rows(&self, source: &str) -> Vec<&QuestRow> {
        self.quests
            .iter()
            .filter(|row| row.source == source)
            .collect()
    }

    pub fn count(&self, source: &str) -> usize {
        self.rows(source).len()
    }

    pub fn objective_count(&self, source: &str) -> usize {
        self.rows(source)
            .iter()
            .map(|row| row.objectives.len())
            .sum()
    }

    pub fn quest(&self, source: &str, id: &QuestId) -> Option<&QuestRow> {
        self.quests
            .iter()
            .find(|row| row.source == source && &row.id == id)
    }

    pub fn supplied_records(&self) -> Vec<&QuestRow> {
        self.rows(QUEST_SOURCE_SUPPLIED_JOURNAL)
    }

    /// Region-1 journal hub registrations. They carry no recovered title,
    /// objective, prerequisite or reward, so the caller only learns the ids.
    pub fn opening_hub_quests(&self) -> Vec<&QuestRow> {
        let mut rows: Vec<&QuestRow> = self
            .rows(QUEST_SOURCE_COMMUNITY_JOURNAL)
            .into_iter()
            .filter(|row| row.hub_id.is_some() && row.region_id == Some(1))
            .collect();
        rows.sort_by_key(|row| row.id.clone());
        rows
    }

    pub fn region_one_stickers(&self) -> Vec<&QuestRow> {
        let mut rows: Vec<&QuestRow> = self
            .rows(QUEST_SOURCE_COMMUNITY_JOURNAL)
            .into_iter()
            .filter(|row| row.sticker_id.is_some() && row.region_id == Some(1))
            .collect();
        rows.sort_by_key(|row| row.id.clone());
        rows
    }

    pub fn wiki_opening_pages(&self) -> Vec<&QuestRow> {
        self.rows(QUEST_SOURCE_WIKI)
            .into_iter()
            .filter(|row| row.opening_arc == Some(true))
            .collect()
    }

    pub fn local_stages(&self) -> Vec<&QuestRow> {
        self.rows(QUEST_SOURCE_LOCAL_OPENING)
    }

    /// (bound, total) original bindings across the local reconstructed stages.
    pub fn local_binding(&self) -> (usize, usize) {
        let stages = self.local_stages();
        let bound = stages
            .iter()
            .filter(|stage| stage.original_quest_id.is_some())
            .count();
        (bound, stages.len())
    }

    pub fn native_acceptance(&self) -> &NativeAcceptance {
        &self.coverage.native_acceptance
    }

    /// Flattened objective rows, so prerequisite/objective/reward analysts do
    /// not have to re-walk the corpus.
    pub fn objective_rows(&self) -> Vec<ObjectiveRef<'_>> {
        let mut rows = Vec::new();
        for quest in &self.quests {
            for objective in &quest.objectives {
                rows.push(ObjectiveRef {
                    source: quest.source.as_str(),
                    quest_id: quest.id.display(),
                    text: objective.text.as_deref(),
                    required: objective.required,
                    objective,
                });
            }
        }
        rows
    }

    pub fn reward_rows(&self) -> Vec<RewardRowRef<'_>> {
        let mut rows = Vec::new();
        for quest in &self.quests {
            for reward in &quest.reward_refs {
                rows.push(RewardRowRef {
                    source: quest.source.as_str(),
                    quest_id: quest.id.display(),
                    reward,
                });
            }
        }
        rows
    }

    pub fn prerequisite_links(&self) -> Vec<PrerequisiteLink<'_>> {
        let mut rows = Vec::new();
        for quest in self.rows(QUEST_SOURCE_WIKI) {
            for prerequisite in &quest.prerequisites {
                rows.push(PrerequisiteLink {
                    page: quest.id.as_text().unwrap_or_default(),
                    prerequisite: prerequisite.as_str(),
                });
            }
        }
        rows
    }

    pub fn start_npcs(&self) -> Vec<(&str, &str)> {
        self.rows(QUEST_SOURCE_WIKI)
            .into_iter()
            .filter_map(|quest| {
                quest
                    .npc
                    .as_deref()
                    .map(|npc| (quest.id.as_text().unwrap_or_default(), npc))
            })
            .collect()
    }

    pub fn validate(&self) -> CatalogueReport {
        let mut report = CatalogueReport::default();

        if self.schema_version != SCHEMA_VERSION {
            report.errors.push(format!(
                "unsupported catalogue schema version {}",
                self.schema_version
            ));
        }

        // Duplicate ids inside one corpus are an integrity error; the same id in
        // two corpora is expected and reported as a shared-id warning.
        let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
        let mut shared: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for row in &self.quests {
            let id = row.id.display();
            let key = (row.source.clone(), id.clone());
            if !seen.insert(key) {
                report
                    .errors
                    .push(format!("duplicate quest id {} in {}", id, row.source));
            }
            shared.entry(id).or_default().insert(row.source.clone());
            if !matches!(
                row.source.as_str(),
                QUEST_SOURCE_SUPPLIED_JOURNAL
                    | QUEST_SOURCE_COMMUNITY_JOURNAL
                    | QUEST_SOURCE_WIKI
                    | QUEST_SOURCE_LOCAL_OPENING
            ) {
                report
                    .errors
                    .push(format!("unknown quest source {}", row.source));
            }
            match row.source.as_str() {
                QUEST_SOURCE_COMMUNITY_JOURNAL => {
                    let has_hub = row.hub_id.is_some();
                    let has_sticker = row.sticker_id.is_some();
                    if has_hub == has_sticker {
                        report.errors.push(format!(
                            "community row {} must be either a hub registration or a sticker",
                            row.id.display()
                        ));
                    }
                }
                QUEST_SOURCE_LOCAL_OPENING => {
                    if let Some(original) = row.original_quest_id {
                        report.errors.push(format!(
                            "local stage {} claims original binding {}",
                            row.id.display(),
                            original
                        ));
                    }
                }
                _ => {}
            }
        }
        for (id, sources) in shared {
            if sources.len() > 1 {
                report.warnings.push(format!(
                    "shared quest id {id} appears in {} corpora (kept separate)",
                    sources.len()
                ));
            }
        }

        // The coverage block must agree with the rows it summarises.
        let (bound, total) = self.local_binding();
        if self.coverage.local_binding.bound as usize != bound
            || self.coverage.local_binding.total as usize != total
        {
            report.errors.push(format!(
                "coverage.local_binding {} / {} disagrees with the {} / {} stage rows",
                self.coverage.local_binding.bound, self.coverage.local_binding.total, bound, total
            ));
        }

        for claim in &self.blocked_claims {
            if claim.status != "disabled" {
                report.errors.push(format!(
                    "blocked claim for quest {} is not disabled (status {})",
                    claim.quest_id, claim.status
                ));
            } else {
                report.warnings.push(format!(
                    "disabled blocked claim: quest {} - {}",
                    claim.quest_id, claim.claim
                ));
            }
        }

        for missing in &self.missing_originals {
            report.warnings.push(format!(
                "missing original: {} {} ({})",
                missing.kind,
                missing.label(),
                missing.reason.as_deref().unwrap_or("no reason recorded")
            ));
        }

        // Wiki prerequisite tokens must resolve to a page title, to a chain link
        // or to a named missing original; otherwise the extraction is suspect.
        let page_titles: BTreeSet<&str> = self
            .rows(QUEST_SOURCE_WIKI)
            .into_iter()
            .filter_map(|row| row.id.as_text())
            .collect();
        let mut chain: BTreeSet<&str> = BTreeSet::new();
        for row in self.rows(QUEST_SOURCE_WIKI) {
            if let Some(previous) = row.previous.as_deref() {
                chain.insert(previous);
            }
            if let Some(next) = row.next.as_deref() {
                chain.insert(next);
            }
        }
        for missing in &self.missing_originals {
            if let Some(title) = missing.quest_title.as_deref() {
                chain.insert(title);
            }
            if let Some(title) = missing.title.as_deref() {
                chain.insert(title);
            }
        }
        for link in self.prerequisite_links() {
            if !page_titles.contains(link.prerequisite) && !chain.contains(link.prerequisite) {
                report.warnings.push(format!(
                    "unresolved wiki prerequisite \"{}\" on {}",
                    link.prerequisite, link.page
                ));
            }
        }

        report
    }
}
