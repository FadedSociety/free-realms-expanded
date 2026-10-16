//! Brawler progression definitions and unlock / reward boundary validation (work item 032).
//!
//! ORIGINAL facts this module reads, each with the preserved source it comes from:
//! * the supplied client profile table (community conversion `Profiles.json`, AGPL-3.0,
//!   `raisingkaines/FreeRealms-Legacy`): Brawler `Id` 43, `NameId` 2873, `DescriptionId` 3234,
//!   `Type` 2, `Icon` 1337, `Hint` 384195, 25 item classes and 7 `DefaultItems`;
//! * `Resources/ItemClasses.txt`: `ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID`; five of
//!   Brawler's classes carry `PROFILE_NAME_ID` 2873 and the rest are borrowed or shared (`0`);
//! * `ClientItemDefinitions.json`: each row's `Class` and `MinProfileRank`, the rank the item
//!   unlocks at (`0` = no rank requirement);
//! * `Resources/EquipmentSlotDefinitions.txt`: slot 7 is `Weapon`, which is why the weapon buckets
//!   here are the ones compared against progression boundaries;
//! * `Resources/Requirements.txt` + `RequirementGroups.txt` + `RequirementSets.txt`: the recovered
//!   `CheckRankLastProfileByType` rows and the AND groups that combine them into rank windows;
//! * `Resources/ItemFilterOptions.txt`: row 11 binds `NAME_ID` 2873 to `VALUE` 43, independently
//!   confirming the profile identity;
//! * `lab/reports/locale-id-to-text.json`: trait name ids 420938..420941 and trait text ids
//!   420962..420965 (`420940` is absent from that export and stays a gap).
//!
//! INFERRED meaning, labelled as such: operator code 6 reads as `rank >= N` and code 4 as
//! `rank <= N`. The inference comes from the shipped AND groups — group 3 is `{>= 5, op4 12}` and
//! group 4 is `{>= 9, op4 16}`, which only describe a rank window if op4 is an upper bound. The
//! binary's comparator implementation is not recovered, so `RankComparator` carries the codes too.
//!
//! COMMUNITY facts, never mixed into the original tables: the wiki's declared Brawler equipment
//! levels and trait ranks. They arrive as caller input and are cross-checked, never trusted.
//!
//! NOT RECOVERED, and therefore never invented here: the experience needed per rank. The supplied
//! client receives `Rank` and `RankPercent` from the server; no preserved table carries thresholds.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// ORIGINAL: the Brawler profile row id. `ItemFilterOptions.txt` row 11 (`2873 -> 43`) confirms it.
pub const BRAWLER_PROFILE_ID: i32 = 43;
/// ORIGINAL: the Brawler locale name id (`"Brawler"`).
pub const BRAWLER_NAME_ID: i32 = 2873;
/// ORIGINAL: the Brawler locale description id (`"Brawlers use basic skills to help them in combat."`).
pub const BRAWLER_DESCRIPTION_ID: i32 = 3234;
/// ORIGINAL: the Brawler profile icon id.
pub const BRAWLER_ICON_ID: i32 = 1337;
/// ORIGINAL: every combat job in the profile table carries `Type` 2.
pub const COMBAT_PROFILE_TYPE: i32 = 2;
/// ORIGINAL ceiling: the shipped catalogue's largest `MinProfileRank` is 20 and
/// `EquipmentSlotDefinitions.txt` requires rank 20 for the MoonRune slot. The wiki's "job level 20"
/// agrees, but the ceiling is taken from the client tables.
pub const MAX_JOB_RANK: i32 = 20;
/// ORIGINAL: the client slot that holds a weapon.
pub const WEAPON_SLOT: i32 = 7;
/// ORIGINAL: the four Brawler trait name ids, in locale order.
pub const BRAWLER_TRAIT_NAME_IDS: [i32; 4] = [420938, 420939, 420940, 420941];
/// ORIGINAL: the four Brawler trait text ids, in the same order.
pub const BRAWLER_TRAIT_DESCRIPTION_IDS: [i32; 4] = [420962, 420963, 420964, 420965];
/// COMMUNITY (wiki revision 48172), not an original table: the ranks the wiki lists for the four
/// traits. Kept separate so a caller cannot mistake it for recovered data.
pub const BRAWLER_TRAIT_RANKS: [i32; 4] = [5, 10, 15, 20];
/// Row cap for every table this module accepts. The largest shipped table is 58,795 rows.
pub const MAX_ROWS: usize = 65_536;
/// Recovered marker the shipped catalogue uses for development rows ("Level 2 Brawler Test
/// Weapon", "Chris' Test Weapon", "Encounter 375 test weapon").
pub const DEVELOPMENT_ROW_MARKER: &str = "test";

/// The exact evidence this item could not recover.
pub const EXPERIENCE_GAPS: [&str; 1] = [
    "per-rank job experience thresholds: no preserved table carries them; the client receives \
     `Rank` and `RankPercent` (`ClientPcProfile`) from the server",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionError {
    HeaderMismatch { table: &'static str },
    MalformedRow { table: &'static str, line: usize },
    TooManyRows { table: &'static str, rows: usize },
    UnknownItemClass { class_id: i32 },
    MixedGateTypes { group_id: i32 },
    InvertedWindow { lower: i32, upper: i32 },
    RankOutOfRange { requirement_id: i32, rank: i32 },
}

impl std::fmt::Display for ProgressionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HeaderMismatch { table } => {
                write!(formatter, "{table} has an unrecognised header")
            }
            Self::MalformedRow { table, line } => {
                write!(formatter, "{table} row {line} is malformed")
            }
            Self::TooManyRows { table, rows } => {
                write!(
                    formatter,
                    "{table} has {rows} rows, above the {MAX_ROWS} row cap"
                )
            }
            Self::UnknownItemClass { class_id } => {
                write!(
                    formatter,
                    "item class {class_id} is absent from ItemClasses.txt"
                )
            }
            Self::MixedGateTypes { group_id } => write!(
                formatter,
                "requirement group {group_id} mixes profile types, so it is not one rank window"
            ),
            Self::InvertedWindow { lower, upper } => {
                write!(formatter, "rank window {lower}..={upper} is empty")
            }
            Self::RankOutOfRange {
                requirement_id,
                rank,
            } => write!(
                formatter,
                "requirement {requirement_id} names rank {rank}, outside 1..={MAX_JOB_RANK}"
            ),
        }
    }
}

impl std::error::Error for ProgressionError {}

/// One `Profiles.json` row. Unknown fields are ignored; every field this module uses is named.
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileRow {
    #[serde(rename = "Comment", default)]
    pub comment: String,
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "NameId", default)]
    pub name_id: i32,
    #[serde(rename = "DescriptionId", default)]
    pub description_id: i32,
    #[serde(rename = "Type", default)]
    pub profile_type: i32,
    #[serde(rename = "Icon", default)]
    pub icon: i32,
    #[serde(rename = "Hint", default)]
    pub hint: i32,
    #[serde(rename = "MembersOnly", default)]
    pub members_only: bool,
    #[serde(rename = "Trial", default)]
    pub trial: bool,
    #[serde(rename = "DefaultItems", default)]
    pub default_items: Vec<i32>,
    #[serde(rename = "ItemClasses", default)]
    pub item_classes: BTreeMap<String, ClassPermission>,
}

impl ProfileRow {
    /// The class ids the profile may equip, read from the keys of `ItemClasses`.
    pub fn class_ids(&self) -> BTreeSet<i32> {
        self.item_classes
            .iter()
            .filter_map(|(key, value)| value.class_id.or_else(|| key.trim().parse::<i32>().ok()))
            .collect()
    }
}

/// A `Profiles.json` `ItemClasses` entry.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ClassPermission {
    #[serde(rename = "Class", default)]
    pub class_id: Option<i32>,
    #[serde(rename = "Unknown", default)]
    pub permission: i32,
}

/// One `ItemClasses.txt` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemClassRow {
    pub id: i32,
    pub name_id: i32,
    pub icon_id: i32,
    pub wield_type: i32,
    pub stat_id: i32,
    pub profile_name_id: i32,
}

/// One `ClientItemDefinitions.json` row, reduced to the fields a progression boundary needs.
#[derive(Debug, Clone, Deserialize)]
pub struct ItemRow {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "Comment", default)]
    pub comment: String,
    #[serde(rename = "Class", default)]
    pub class_id: Option<i32>,
    #[serde(rename = "MinProfileRank", default)]
    pub min_profile_rank: i32,
    #[serde(rename = "Slot", default)]
    pub slot: Option<i32>,
    #[serde(rename = "NameId", default)]
    pub name_id: Option<i32>,
    #[serde(rename = "ProfileOverride", default)]
    pub profile_override: Option<i32>,
}

/// One `Requirements.txt` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementRow {
    pub id: i32,
    pub requirement_type: i32,
    pub factory: String,
    pub operator: i32,
    pub param_1: i32,
    pub param_2: i32,
    pub param_3: i32,
}

/// One `ItemFilterOptions.txt` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterOptionRow {
    pub id: i32,
    pub name_id: i32,
    pub value: i32,
}

/// The comparator a rank gate applies. INFERRED from the operator codes (see the module header).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankComparator {
    AtLeast,
    AtMost,
}

impl RankComparator {
    /// ORIGINAL operator codes as they appear in `Requirements.txt`.
    pub fn from_operator_code(code: i32) -> Option<Self> {
        match code {
            6 => Some(Self::AtLeast),
            4 => Some(Self::AtMost),
            _ => None,
        }
    }

    pub fn operator_code(self) -> i32 {
        match self {
            Self::AtLeast => 6,
            Self::AtMost => 4,
        }
    }
}

/// A recovered `CheckRankLastProfileByType` requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankGate {
    pub requirement_id: i32,
    pub profile_type: i32,
    pub comparator: RankComparator,
    pub rank: i32,
}

impl RankGate {
    /// Returns `None` for every requirement that is not a recovered rank gate.
    pub fn from_row(row: &RequirementRow) -> Option<Self> {
        if row.factory != "CheckRankLastProfileByType" {
            return None;
        }
        Some(Self {
            requirement_id: row.id,
            profile_type: row.param_1,
            comparator: RankComparator::from_operator_code(row.operator)?,
            rank: row.param_2,
        })
    }

    pub fn matches(&self, rank: i32) -> bool {
        match self.comparator {
            RankComparator::AtLeast => rank >= self.rank,
            RankComparator::AtMost => rank <= self.rank,
        }
    }

    pub fn matches_for(&self, profile_type: i32, rank: i32) -> bool {
        self.profile_type == profile_type && self.matches(rank)
    }
}

/// One rank window assembled from the AND groups of a requirement set. Bounds carry the
/// requirement id they came from so a reviewer can follow the evidence back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankWindow {
    pub set_id: i32,
    pub profile_type: i32,
    pub lower: Option<(i32, i32)>,
    pub upper: Option<(i32, i32)>,
}

impl RankWindow {
    pub fn contains(&self, rank: i32) -> bool {
        let above = self.lower.is_none_or(|(_, lower)| rank >= lower);
        let below = self.upper.is_none_or(|(_, upper)| rank <= upper);
        above && below
    }

    /// Builds one window per requirement set, using only gates of `profile_type`. A set with no such
    /// gate is skipped; a group that mixes profile types is refused instead of averaged.
    pub fn for_sets(
        requirements: &[RequirementRow],
        groups: &[(i32, i32)],
        sets: &[(i32, i32)],
        profile_type: i32,
    ) -> Result<Vec<Self>, ProgressionError> {
        let gates: BTreeMap<i32, RankGate> = requirements
            .iter()
            .filter_map(|row| RankGate::from_row(row).map(|gate| (gate.requirement_id, gate)))
            .collect();
        let mut grouped: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        for (group_id, requirement_id) in groups {
            grouped.entry(*group_id).or_default().push(*requirement_id);
        }

        let mut windows = Vec::new();
        let mut by_set: BTreeMap<i32, BTreeSet<i32>> = BTreeMap::new();
        for (set_id, group_id) in sets {
            by_set.entry(*set_id).or_default().insert(*group_id);
        }

        for (set_id, group_ids) in by_set {
            let mut lower: Option<(i32, i32)> = None;
            let mut upper: Option<(i32, i32)> = None;
            let mut found = false;
            for group_id in group_ids {
                let members = grouped.get(&group_id).cloned().unwrap_or_default();
                let group_gates: Vec<RankGate> = members
                    .iter()
                    .filter_map(|id| gates.get(id).copied())
                    .collect();
                let types: BTreeSet<i32> =
                    group_gates.iter().map(|gate| gate.profile_type).collect();
                if types.len() > 1 {
                    return Err(ProgressionError::MixedGateTypes { group_id });
                }
                for gate in group_gates {
                    if !(1..=MAX_JOB_RANK).contains(&gate.rank) {
                        return Err(ProgressionError::RankOutOfRange {
                            requirement_id: gate.requirement_id,
                            rank: gate.rank,
                        });
                    }
                    if gate.profile_type != profile_type {
                        continue;
                    }
                    found = true;
                    match gate.comparator {
                        RankComparator::AtLeast => {
                            if lower.is_none_or(|(_, rank)| gate.rank > rank) {
                                lower = Some((gate.requirement_id, gate.rank));
                            }
                        }
                        RankComparator::AtMost => {
                            if upper.is_none_or(|(_, rank)| gate.rank < rank) {
                                upper = Some((gate.requirement_id, gate.rank));
                            }
                        }
                    }
                }
            }
            if !found {
                continue;
            }
            if let (Some((_, lower_rank)), Some((_, upper_rank))) = (lower, upper) {
                if lower_rank > upper_rank {
                    return Err(ProgressionError::InvertedWindow {
                        lower: lower_rank,
                        upper: upper_rank,
                    });
                }
            }
            windows.push(Self {
                set_id,
                profile_type,
                lower,
                upper,
            });
        }
        Ok(windows)
    }
}

/// A class the profile can equip, joined against `ItemClasses.txt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WieldClass {
    pub id: i32,
    pub name_id: i32,
    pub wield_type: i32,
    /// ORIGINAL `Unknown` value from the profile's own `ItemClasses` entry, carried uninterpreted.
    pub permission: i32,
    /// True when `ItemClasses.txt` names this same profile as the class owner.
    pub owned: bool,
}

/// A row that breaks a recovered boundary. The row is reported and left out of the live buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryViolation {
    pub item_id: i32,
    pub item_class: i32,
    pub rank: i32,
}

/// A creation-kit entry from `DefaultItems`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KitEntry {
    pub item_id: i32,
    pub class_id: Option<i32>,
    pub slot: Option<i32>,
    pub rank: i32,
    pub class_wieldable: bool,
}

/// Cross-check between the community's declared levels and the original rows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RankCrossCheck {
    /// Declared levels with neither a live row nor a development row behind them.
    pub missing: BTreeSet<i32>,
    /// Live original levels the community list does not mention.
    pub unexpected: BTreeSet<i32>,
    /// Ranks whose original weapon rows are all development rows, so no live item unlocks there.
    pub development_only: BTreeSet<i32>,
}

/// The recovered Brawler (or any combat job's) progression boundaries.
#[derive(Debug, Clone)]
pub struct JobProgression {
    pub profile_id: i32,
    pub profile_type: i32,
    pub wield_classes: Vec<WieldClass>,
    /// Weapon rows by unlock rank, development rows excluded. Ranks are `1..=MAX_JOB_RANK`.
    pub equipment: BTreeMap<i32, Vec<i32>>,
    /// Weapon rows whose `MinProfileRank` is `0`: no rank requirement, available at every rank.
    pub unrestricted: Vec<i32>,
    /// Weapon rows the shipped catalogue marks as development rows. Never live boundaries.
    pub development_items: Vec<i32>,
    pub violations: Vec<BoundaryViolation>,
    development_by_rank: BTreeMap<i32, Vec<i32>>,
    kit: Vec<KitEntry>,
}

fn header_cells(line: &str) -> Vec<&str> {
    line.trim_start_matches('#')
        .split('^')
        .filter(|cell| !cell.is_empty())
        .collect()
}

fn row_cells(line: &str) -> Vec<&str> {
    let mut cells: Vec<&str> = line.split('^').collect();
    while cells.last() == Some(&"") {
        cells.pop();
    }
    cells
}

fn parse_cell<T: std::str::FromStr>(
    cells: &[&str],
    index: usize,
    table: &'static str,
    line: usize,
) -> Result<T, ProgressionError> {
    cells
        .get(index)
        .and_then(|cell| cell.parse::<T>().ok())
        .ok_or(ProgressionError::MalformedRow { table, line })
}

fn check_header(
    text: &str,
    expected: &[&str],
    table: &'static str,
) -> Result<(), ProgressionError> {
    let first = text.lines().find(|line| !line.trim().is_empty());
    match first {
        Some(line) if header_cells(line) == expected => Ok(()),
        _ => Err(ProgressionError::HeaderMismatch { table }),
    }
}

/// Parses `Resources/ItemClasses.txt`.
pub fn parse_item_class_table(text: &str) -> Result<Vec<ItemClassRow>, ProgressionError> {
    const TABLE: &str = "ItemClasses";
    const HEADER: [&str; 6] = [
        "ID",
        "NAME_ID",
        "ICON_ID",
        "WIELD_TYPE",
        "STAT_ID",
        "PROFILE_NAME_ID",
    ];
    check_header(text, &HEADER, TABLE)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        if rows.len() >= MAX_ROWS {
            return Err(ProgressionError::TooManyRows {
                table: TABLE,
                rows: rows.len(),
            });
        }
        let cells = row_cells(line);
        rows.push(ItemClassRow {
            id: parse_cell(&cells, 0, TABLE, index + 1)?,
            name_id: parse_cell(&cells, 1, TABLE, index + 1)?,
            icon_id: parse_cell(&cells, 2, TABLE, index + 1)?,
            wield_type: parse_cell(&cells, 3, TABLE, index + 1)?,
            stat_id: parse_cell(&cells, 4, TABLE, index + 1)?,
            profile_name_id: parse_cell(&cells, 5, TABLE, index + 1)?,
        });
    }
    Ok(rows)
}

/// Parses `Resources/Requirements.txt`.
pub fn parse_requirement_table(text: &str) -> Result<Vec<RequirementRow>, ProgressionError> {
    const TABLE: &str = "Requirements";
    const HEADER: [&str; 10] = [
        "client_requirements.ID",
        "client_requirements.TYPE",
        "ref_client_req_types.CODE_FACTORY_NAME",
        "client_requirements.OPERATOR",
        "client_requirements.PARAM_1",
        "client_requirements.PARAM_2",
        "client_requirements.PARAM_3",
        "client_requirements.STRING_ID",
        "client_requirements.PARAM_4",
        "client_requirements.PARAM_5",
    ];
    check_header(text, &HEADER, TABLE)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        if rows.len() >= MAX_ROWS {
            return Err(ProgressionError::TooManyRows {
                table: TABLE,
                rows: rows.len(),
            });
        }
        let cells = row_cells(line);
        let factory = cells
            .get(2)
            .ok_or(ProgressionError::MalformedRow {
                table: TABLE,
                line: index + 1,
            })?
            .to_string();
        rows.push(RequirementRow {
            id: parse_cell(&cells, 0, TABLE, index + 1)?,
            requirement_type: parse_cell(&cells, 1, TABLE, index + 1)?,
            factory,
            operator: parse_cell(&cells, 3, TABLE, index + 1)?,
            param_1: parse_cell(&cells, 4, TABLE, index + 1)?,
            param_2: parse_cell(&cells, 5, TABLE, index + 1)?,
            param_3: parse_cell(&cells, 6, TABLE, index + 1)?,
        });
    }
    Ok(rows)
}

/// Parses a two-column id mapping table such as `RequirementGroups.txt` or `RequirementSets.txt`.
pub fn parse_id_pairs(
    text: &str,
    expected: [&str; 2],
    table: &'static str,
) -> Result<Vec<(i32, i32)>, ProgressionError> {
    check_header(text, &expected, table)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        if rows.len() >= MAX_ROWS {
            return Err(ProgressionError::TooManyRows {
                table,
                rows: rows.len(),
            });
        }
        let cells = row_cells(line);
        rows.push((
            parse_cell(&cells, 0, table, index + 1)?,
            parse_cell(&cells, 1, table, index + 1)?,
        ));
    }
    Ok(rows)
}

/// Parses `Resources/ItemFilterOptions.txt`.
pub fn parse_item_filter_options(text: &str) -> Result<Vec<FilterOptionRow>, ProgressionError> {
    const TABLE: &str = "ItemFilterOptions";
    const HEADER: [&str; 6] = [
        "ID",
        "NAME_ID",
        "HAS_RANGE",
        "VALUE",
        "VALUE_RANGE_START",
        "VALUE_RANGE_END",
    ];
    check_header(text, &HEADER, TABLE)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        if rows.len() >= MAX_ROWS {
            return Err(ProgressionError::TooManyRows {
                table: TABLE,
                rows: rows.len(),
            });
        }
        let cells = row_cells(line);
        rows.push(FilterOptionRow {
            id: parse_cell(&cells, 0, TABLE, index + 1)?,
            name_id: parse_cell(&cells, 1, TABLE, index + 1)?,
            value: parse_cell(&cells, 3, TABLE, index + 1)?,
        });
    }
    Ok(rows)
}

fn is_development_row(comment: &str) -> bool {
    comment
        .to_ascii_lowercase()
        .contains(DEVELOPMENT_ROW_MARKER)
}

impl JobProgression {
    /// Joins the profile's `ItemClasses` against `ItemClasses.txt` and buckets every weapon row of
    /// those classes by its recovered unlock rank.
    pub fn build(
        profile: &ProfileRow,
        classes: &[ItemClassRow],
        items: &[ItemRow],
    ) -> Result<Self, ProgressionError> {
        if items.len() > MAX_ROWS {
            return Err(ProgressionError::TooManyRows {
                table: "ClientItemDefinitions",
                rows: items.len(),
            });
        }

        let by_id: BTreeMap<i32, &ItemClassRow> = classes.iter().map(|row| (row.id, row)).collect();
        let class_ids = profile.class_ids();
        let mut wield_classes = Vec::new();
        for class_id in &class_ids {
            let row = by_id
                .get(class_id)
                .ok_or(ProgressionError::UnknownItemClass {
                    class_id: *class_id,
                })?;
            let permission = profile
                .item_classes
                .values()
                .find(|value| value.class_id == Some(*class_id))
                .map(|value| value.permission)
                .unwrap_or_default();
            wield_classes.push(WieldClass {
                id: row.id,
                name_id: row.name_id,
                wield_type: row.wield_type,
                permission,
                owned: row.profile_name_id == profile.name_id,
            });
        }

        let mut equipment: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        let mut unrestricted = Vec::new();
        let mut development_items = Vec::new();
        let mut development_by_rank: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        let mut violations = Vec::new();
        for row in items {
            let Some(class_id) = row.class_id else {
                continue;
            };
            if !class_ids.contains(&class_id) {
                continue;
            }
            if row.slot != Some(WEAPON_SLOT) {
                continue;
            }
            if !(0..=MAX_JOB_RANK).contains(&row.min_profile_rank) {
                violations.push(BoundaryViolation {
                    item_id: row.id,
                    item_class: class_id,
                    rank: row.min_profile_rank,
                });
                continue;
            }
            if is_development_row(&row.comment) {
                development_items.push(row.id);
                development_by_rank
                    .entry(row.min_profile_rank)
                    .or_default()
                    .push(row.id);
                continue;
            }
            if row.min_profile_rank == 0 {
                unrestricted.push(row.id);
            } else {
                equipment
                    .entry(row.min_profile_rank)
                    .or_default()
                    .push(row.id);
            }
        }

        let mut kit = Vec::new();
        for item_id in &profile.default_items {
            let row = items.iter().find(|row| row.id == *item_id);
            kit.push(KitEntry {
                item_id: *item_id,
                class_id: row.and_then(|row| row.class_id),
                slot: row.and_then(|row| row.slot),
                rank: row.map(|row| row.min_profile_rank).unwrap_or_default(),
                class_wieldable: row
                    .and_then(|row| row.class_id)
                    .is_some_and(|class_id| class_ids.contains(&class_id)),
            });
        }

        Ok(Self {
            profile_id: profile.id,
            profile_type: profile.profile_type,
            wield_classes,
            equipment,
            unrestricted,
            development_items,
            violations,
            development_by_rank,
            kit,
        })
    }

    /// Weapon ids a job of `rank` may use, ordered by unlock rank and then by the order the rows
    /// appear in the supplied table. Rows without a rank requirement come last because they are
    /// available at every rank.
    pub fn unlocked_at(&self, rank: i32) -> Vec<i32> {
        let mut ids = Vec::new();
        for (bucket_rank, bucket) in &self.equipment {
            if *bucket_rank <= rank {
                ids.extend(bucket.iter().copied());
            }
        }
        ids.extend(self.unrestricted.iter().copied());
        ids
    }

    /// Cumulative unlock counts for ranks `1..=MAX_JOB_RANK`, including the unrestricted rows.
    pub fn unlock_curve(&self) -> Vec<usize> {
        (1..=MAX_JOB_RANK)
            .map(|rank| self.unlocked_at(rank).len())
            .collect()
    }

    /// Ranks that carry at least one live weapon row.
    pub fn observed_ranks(&self) -> BTreeSet<i32> {
        self.equipment.keys().copied().collect()
    }

    /// Ranks that carry at least one development row.
    pub fn development_ranks(&self) -> BTreeSet<i32> {
        self.development_by_rank.keys().copied().collect()
    }

    /// Ranks whose only weapon rows are development rows: no live item defines that boundary.
    pub fn development_only_ranks(&self) -> BTreeSet<i32> {
        self.development_by_rank
            .keys()
            .copied()
            .filter(|rank| !self.equipment.contains_key(rank))
            .collect()
    }

    /// Compares a community list with the original rows without preferring either.
    pub fn cross_check_declared_ranks(&self, declared: &BTreeSet<i32>) -> RankCrossCheck {
        let observed = self.observed_ranks();
        let development_only = self.development_only_ranks();
        RankCrossCheck {
            missing: declared
                .difference(&observed)
                .filter(|rank| !development_only.contains(rank))
                .copied()
                .collect(),
            unexpected: observed.difference(declared).copied().collect(),
            development_only,
        }
    }

    pub fn creation_kit(&self, _profile: &ProfileRow) -> Vec<KitEntry> {
        self.kit.clone()
    }

    /// The evidence this item could not recover. Reported, never guessed.
    pub fn experience_gaps(&self) -> &'static [&'static str] {
        &EXPERIENCE_GAPS
    }
}
