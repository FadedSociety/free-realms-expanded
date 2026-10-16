//! Work item 097: catalogue asset-reference validation.
//!
//! Every input below is an ORIGINAL preserved source; this module invents no catalogue row, no
//! price, no model and no locale string. It measures the references the recovered stock item
//! catalogue makes and reports the ones that do not resolve:
//!
//! * `ClientItemDefinitions.json` (AGPL-3.0, `raisingkaines/FreeRealms-Legacy` revision
//!   `9e082025391177d5a33ee18c714ddd06b457a695`, sha256
//!   `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`) is the 58,795 row stock
//!   item catalogue. Its `NameId`, `DescriptionId`, `Class`, `CategoryId`, `ModelName`, `Type` and
//!   `Param1` cells are the references under test.
//! * `ItemClasses.txt`, `ItemCategories.txt` and `Models.txt` are the sibling reference tables the
//!   item rows point at (`Models.txt` is `ID^MODEL_FILE_NAME^...^IS_VALID_FOR_PC^...`).
//! * `rust-asset-catalog.tsv` is the Rust pack reader's name list over the 24 preserved packs.
//! * `locale-id-to-text.json` is the unmodified community locale reader export of the supplied
//!   client `locale/en_us_data.dat`/`.dir`.
//!
//! RECONSTRUCTED, labelled at each use site:
//!
//! * The four PC model variants (`human`/`fairy` x `m`/`f`) used to expand `<race>`/`<gender>`
//!   templates. No recovered table declares that set; it is the only expansion under which the
//!   preserved catalogue nearly closes (31,957 of 31,962 templates resolve), and it matches the
//!   four character-creation rows recorded in `notes/Research/Character Creation Audit.md`.
//! * The `Param1`-to-`Models.txt` relationship is derived from the catalogue's own coverage rather
//!   than asserted: a `Param1` value that does not resolve is reported as a conflict only for item
//!   types that resolve at least one other `Param1` value. Types that resolve none stay coverage.
//!
//! ponytail: one pass over each table, `BTreeMap`/`BTreeSet` only, no index files and no embedded
//! catalogue dump. The Python checker `scripts/pieces/piece_097.py` is the independent oracle over
//! the same inputs.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Recovered naming convention for the player-race placeholder in a model file name.
pub const MODEL_RACE_TOKEN: &str = "<race>";
/// Recovered naming convention for the player-gender placeholder in a model file name.
pub const MODEL_GENDER_TOKEN: &str = "<gender>";
/// The recovered playable race tokens. RECONSTRUCTED set; see the module docs.
pub const PC_RACES: [&str; 2] = ["human", "fairy"];
/// The recovered playable gender tokens. RECONSTRUCTED set; see the module docs.
pub const PC_GENDERS: [&str; 2] = ["m", "f"];

/// One stock catalogue row, deserialized straight from `ClientItemDefinitions.json`.
///
/// The shipped rows carry far more cells than these; only the reference-bearing ones are kept.
/// Rows whose `Icon` block holds only a `TintId` still parse because unknown cells are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawItem {
    #[serde(rename = "Id", default)]
    pub id: i32,
    #[serde(rename = "NameId", default)]
    pub name_id: Option<i32>,
    #[serde(rename = "DescriptionId", default)]
    pub description_id: Option<i32>,
    #[serde(rename = "ModelName", default)]
    pub model: Option<String>,
    #[serde(rename = "Class", default)]
    pub class_id: Option<i32>,
    #[serde(rename = "CategoryId", default)]
    pub category_id: Option<i32>,
    #[serde(rename = "Type", default)]
    pub item_type: Option<i32>,
    #[serde(rename = "Param1", default)]
    pub param1: Option<i32>,
}

/// One `ItemClasses.txt` row: `ID^NAME_ID^ICON_ID^WIELD_TYPE^STAT_ID^PROFILE_NAME_ID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassRow {
    pub id: i32,
    pub name_id: i32,
    pub profile_name_id: i32,
}

/// One `ItemCategories.txt` row: `ID^NAME_ID^ICON_ID^UNKNOWN^UNKNOWN_2^DISPLAY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryRow {
    pub id: i32,
    pub name_id: i32,
}

/// One `Models.txt` row, trimmed to the reference-bearing cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRow {
    pub id: i32,
    pub file: String,
    pub race_id: i32,
    pub is_valid_for_pc: bool,
}

/// What a declared `ModelName` references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelReference {
    /// No template token: the exact lowercased file name must exist in the packs.
    Literal(String),
    /// Only `<race>`/`<gender>` tokens, which the recovered PC variants expand.
    RaceGender(String),
    /// A token with no recovered expansion table; the relationship stays unresolved.
    Unmapped { name: String, tokens: Vec<String> },
}

/// A malformed reference table row. Duplicate ids are rejected as conflicting entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    MalformedRow {
        table: &'static str,
        line: usize,
        fields: usize,
        expected: usize,
    },
    BadNumber {
        table: &'static str,
        line: usize,
        field: usize,
        value: String,
    },
    DuplicateId {
        table: &'static str,
        line: usize,
        id: i32,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MalformedRow {
                table,
                line,
                fields,
                expected,
            } => write!(
                f,
                "{table} line {line} has {fields} fields, expected at least {expected}"
            ),
            ParseError::BadNumber {
                table,
                line,
                field,
                value,
            } => {
                write!(
                    f,
                    "{table} line {line} field {field} is not a number: {value:?}"
                )
            }
            ParseError::DuplicateId { table, line, id } => {
                write!(f, "{table} line {line} repeats id {id}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Split one `^`-delimited reference table, skipping blank lines and `#` headers.
///
/// The shipped tables end every row with a trailing delimiter, so a single trailing empty field is
/// dropped rather than counted as a column.
fn delimited_rows<'a>(
    table: &'static str,
    text: &'a str,
    expected: usize,
) -> Result<Vec<(usize, Vec<&'a str>)>, ParseError> {
    let mut rows = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut fields: Vec<&str> = trimmed.split('^').collect();
        if fields.last() == Some(&"") {
            fields.pop();
        }
        if fields.len() < expected {
            return Err(ParseError::MalformedRow {
                table,
                line,
                fields: fields.len(),
                expected,
            });
        }
        rows.push((line, fields));
    }
    Ok(rows)
}

fn number(table: &'static str, line: usize, field: usize, value: &str) -> Result<i32, ParseError> {
    value.trim().parse().map_err(|_| ParseError::BadNumber {
        table,
        line,
        field,
        value: value.to_owned(),
    })
}

/// Parse `ItemClasses.txt`; a repeated class id is rejected as a conflicting entry.
pub fn parse_item_classes(text: &str) -> Result<BTreeMap<i32, ClassRow>, ParseError> {
    const TABLE: &str = "ItemClasses.txt";
    let mut rows = BTreeMap::new();
    for (line, fields) in delimited_rows(TABLE, text, 6)? {
        let id = number(TABLE, line, 0, fields[0])?;
        let name_id = number(TABLE, line, 1, fields[1])?;
        let profile_name_id = number(TABLE, line, 5, fields[5])?;
        if rows
            .insert(
                id,
                ClassRow {
                    id,
                    name_id,
                    profile_name_id,
                },
            )
            .is_some()
        {
            return Err(ParseError::DuplicateId {
                table: TABLE,
                line,
                id,
            });
        }
    }
    Ok(rows)
}

/// Parse `ItemCategories.txt`; a repeated category id is rejected as a conflicting entry.
pub fn parse_item_categories(text: &str) -> Result<BTreeMap<i32, CategoryRow>, ParseError> {
    const TABLE: &str = "ItemCategories.txt";
    let mut rows = BTreeMap::new();
    for (line, fields) in delimited_rows(TABLE, text, 6)? {
        let id = number(TABLE, line, 0, fields[0])?;
        let name_id = number(TABLE, line, 1, fields[1])?;
        if rows.insert(id, CategoryRow { id, name_id }).is_some() {
            return Err(ParseError::DuplicateId {
                table: TABLE,
                line,
                id,
            });
        }
    }
    Ok(rows)
}

/// Parse `Models.txt`; a repeated model id is rejected as a conflicting entry.
pub fn parse_models(text: &str) -> Result<BTreeMap<i32, ModelRow>, ParseError> {
    const TABLE: &str = "Models.txt";
    let mut rows = BTreeMap::new();
    for (line, fields) in delimited_rows(TABLE, text, 14)? {
        let id = number(TABLE, line, 0, fields[0])?;
        let race_id = number(TABLE, line, 2, fields[2])?;
        let is_valid_for_pc = number(TABLE, line, 13, fields[13])? != 0;
        let row = ModelRow {
            id,
            file: fields[1].to_owned(),
            race_id,
            is_valid_for_pc,
        };
        if rows.insert(id, row).is_some() {
            return Err(ParseError::DuplicateId {
                table: TABLE,
                line,
                id,
            });
        }
    }
    Ok(rows)
}

/// Classify a declared `ModelName` by the template tokens it carries.
///
/// The returned name is lowercased because the pack catalogue comparison is case-insensitive.
pub fn classify_model(name: &str) -> ModelReference {
    let lowered = name.to_ascii_lowercase();
    let tokens = template_tokens(&lowered);
    if tokens.is_empty() {
        ModelReference::Literal(lowered)
    } else if tokens
        .iter()
        .all(|token| token == MODEL_RACE_TOKEN || token == MODEL_GENDER_TOKEN)
    {
        ModelReference::RaceGender(lowered)
    } else {
        ModelReference::Unmapped {
            name: lowered,
            tokens,
        }
    }
}

/// The `<...>` tokens of a lowercased model file name, in order of appearance.
fn template_tokens(lowered: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = lowered;
    while let Some(start) = rest.find('<') {
        let after = &rest[start..];
        let Some(end) = after.find('>') else {
            break;
        };
        tokens.push(after[..=end].to_owned());
        rest = &after[end + 1..];
    }
    tokens
}

/// Expand a `<race>`/`<gender>` template over the recovered PC variants.
///
/// Tokens without a recovered expansion stay in place and the surviving variants are
/// deduplicated, so `<race>_m_hair_afro_<hairtype>.adr` yields the two male variants.
pub fn expand_race_gender(name: &str) -> Vec<String> {
    let lowered = name.to_ascii_lowercase();
    let mut variants = Vec::new();
    for race in PC_RACES {
        for gender in PC_GENDERS {
            let expanded = lowered
                .replace(MODEL_RACE_TOKEN, race)
                .replace(MODEL_GENDER_TOKEN, gender);
            if !variants.contains(&expanded) {
                variants.push(expanded);
            }
        }
    }
    variants
}

/// Lowercased asset names from the pack catalogue TSV (`name` column).
pub fn asset_names(tsv: &str) -> BTreeSet<String> {
    let mut lines = tsv.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let column = header.iter().position(|name| *name == "name").unwrap_or(2);
    lines
        .filter_map(|line| line.split('\t').nth(column))
        .filter(|name| !name.is_empty())
        .map(|name| name.to_ascii_lowercase())
        .collect()
}

/// Locale ids from the community reader export (`[{"id":..,"text":".."}]`).
///
/// A malformed export yields an empty set, which makes every name a reported gap instead of a
/// silent pass.
pub fn locale_ids(json: &str) -> BTreeSet<i32> {
    #[derive(Deserialize)]
    struct Entry {
        id: i32,
    }
    serde_json::from_str::<Vec<Entry>>(json)
        .map(|entries| entries.into_iter().map(|entry| entry.id).collect())
        .unwrap_or_default()
}

/// The parsed reference tables the item catalogue points at.
#[derive(Debug, Clone, Default)]
pub struct CatalogueTables {
    pub classes: BTreeMap<i32, ClassRow>,
    pub categories: BTreeMap<i32, CategoryRow>,
    pub models: BTreeMap<i32, ModelRow>,
    pub assets: BTreeSet<String>,
    pub locale: BTreeSet<i32>,
}

/// Bundle the parsed tables for [`analyze`].
pub fn analyze_tables(
    classes: BTreeMap<i32, ClassRow>,
    categories: BTreeMap<i32, CategoryRow>,
    models: BTreeMap<i32, ModelRow>,
    assets: BTreeSet<String>,
    locale: BTreeSet<i32>,
) -> CatalogueTables {
    CatalogueTables {
        classes,
        categories,
        models,
        assets,
        locale,
    }
}

/// How many `Param1` values one item type carries and how many resolve to a `Models.txt` row.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Param1Coverage {
    pub present: usize,
    pub resolved: usize,
}

/// Sorted relationship results. Every list stays sorted so two runs compare byte for byte.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Findings {
    pub items: usize,
    pub duplicate_item_ids: Vec<i32>,
    /// Rows with no `NameId` cell at all.
    pub name_id_absent: usize,
    /// Item ids whose `NameId` is a positive or sentinel value that names nothing.
    pub name_id_unset: Vec<i32>,
    /// `(item id, locale id)` pairs whose id is absent from the locale export.
    pub missing_name_ids: Vec<(i32, i32)>,
    /// Rows with no `DescriptionId` cell at all.
    pub description_id_absent: usize,
    pub description_id_unset: Vec<i32>,
    pub missing_description_ids: Vec<(i32, i32)>,
    pub literal_model_rows: usize,
    /// `(item id, model file name)` for literal names absent from the packs.
    pub missing_literal_models: Vec<(i32, String)>,
    pub race_gender_model_rows: usize,
    /// `(item id, model template)` whose four PC variants are all absent from the packs.
    pub missing_race_gender_models: Vec<(i32, String)>,
    pub unmapped_template_rows: usize,
    /// Unrecovered token -> row count. The recovered `<race>`/`<gender>` tokens are not counted
    /// here even when they share a row with an unrecovered token.
    pub unmapped_template_tokens: BTreeMap<String, usize>,
    /// `Class` ids the items reference but `ItemClasses.txt` does not define.
    pub missing_item_classes: Vec<i32>,
    /// `CategoryId` ids the items reference but `ItemCategories.txt` does not define.
    pub missing_item_categories: Vec<i32>,
    /// Class rows whose `NAME_ID` is absent from the locale export.
    pub class_locale_gaps: Vec<i32>,
    pub category_locale_gaps: Vec<i32>,
    pub model_rows: usize,
    pub models_valid_for_pc: usize,
    /// `Models.txt` ids whose declared file is absent from the packs.
    pub model_asset_gaps: Vec<i32>,
    /// Item type -> `Param1` coverage; `None` is a row with no `Type` cell.
    pub param1_coverage: BTreeMap<Option<i32>, Param1Coverage>,
    /// Types that resolve at least one `Param1` value and leave at least one unresolved.
    pub param1_conflict_types: Vec<i32>,
    /// `(item id, item type, param1)` for the conflicting rows, sorted by item id.
    pub param1_conflicts: Vec<(i32, i32, i32)>,
}

/// Measure every reference the stock catalogue makes.
pub fn analyze(items: &[RawItem], tables: &CatalogueTables) -> Findings {
    let mut findings = Findings {
        items: items.len(),
        ..Findings::default()
    };
    let mut id_counts: BTreeMap<i32, usize> = BTreeMap::new();
    let mut missing_class_ids = BTreeSet::new();
    let mut missing_category_ids = BTreeSet::new();
    let mut coverage: BTreeMap<Option<i32>, Param1Coverage> = BTreeMap::new();

    for item in items {
        *id_counts.entry(item.id).or_default() += 1;
        findings.account_locale(item, &tables.locale);
        if let Some(class_id) = item.class_id {
            if !tables.classes.contains_key(&class_id) {
                missing_class_ids.insert(class_id);
            }
        }
        if let Some(category_id) = item.category_id {
            if !tables.categories.contains_key(&category_id) {
                missing_category_ids.insert(category_id);
            }
        }
        if let Some(model) = item.model.as_deref().filter(|model| !model.is_empty()) {
            match classify_model(model) {
                ModelReference::Literal(_) => {
                    findings.literal_model_rows += 1;
                    if !tables.assets.contains(&model.to_ascii_lowercase()) {
                        findings
                            .missing_literal_models
                            .push((item.id, model.to_owned()));
                    }
                }
                ModelReference::RaceGender(_) => {
                    findings.race_gender_model_rows += 1;
                    let resolves = expand_race_gender(model)
                        .iter()
                        .any(|variant| tables.assets.contains(variant));
                    if !resolves {
                        findings
                            .missing_race_gender_models
                            .push((item.id, model.to_owned()));
                    }
                }
                ModelReference::Unmapped { tokens, .. } => {
                    findings.unmapped_template_rows += 1;
                    for token in tokens {
                        if token == MODEL_RACE_TOKEN || token == MODEL_GENDER_TOKEN {
                            continue;
                        }
                        *findings.unmapped_template_tokens.entry(token).or_default() += 1;
                    }
                }
            }
        }
        if let Some(param1) = item.param1 {
            if param1 > 0 {
                let cell = coverage.entry(item.item_type).or_default();
                cell.present += 1;
                if tables.models.contains_key(&param1) {
                    cell.resolved += 1;
                }
            }
        }
    }

    findings.duplicate_item_ids = id_counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(id, _)| *id)
        .collect();
    findings.missing_item_classes = missing_class_ids.into_iter().collect();
    findings.missing_item_categories = missing_category_ids.into_iter().collect();
    findings.class_locale_gaps = tables
        .classes
        .values()
        .filter(|row| !tables.locale.contains(&row.name_id))
        .map(|row| row.id)
        .collect();
    findings.category_locale_gaps = tables
        .categories
        .values()
        .filter(|row| !tables.locale.contains(&row.name_id))
        .map(|row| row.id)
        .collect();
    findings.model_rows = tables.models.len();
    findings.models_valid_for_pc = tables
        .models
        .values()
        .filter(|row| row.is_valid_for_pc)
        .count();
    findings.model_asset_gaps = tables
        .models
        .values()
        .filter(|row| !tables.assets.contains(&row.file.to_ascii_lowercase()))
        .map(|row| row.id)
        .collect();

    let conflict_types: BTreeSet<i32> = coverage
        .iter()
        .filter(|(key, cell)| key.is_some() && cell.resolved > 0 && cell.resolved < cell.present)
        .filter_map(|(key, _)| *key)
        .collect();
    findings.param1_conflicts = items
        .iter()
        .filter_map(|item| {
            let value = item.param1?;
            if value <= 0 || tables.models.contains_key(&value) {
                return None;
            }
            let item_type = item.item_type?;
            conflict_types
                .contains(&item_type)
                .then_some((item.id, item_type, value))
        })
        .collect();
    findings.param1_conflicts.sort_unstable();
    findings.param1_conflict_types = conflict_types.into_iter().collect();
    findings.param1_coverage = coverage;
    findings
}

impl Findings {
    /// Locale accounting shared by the name and description cells.
    fn account_locale(&mut self, item: &RawItem, locale: &BTreeSet<i32>) {
        account_id(
            item.name_id,
            item.id,
            locale,
            &mut self.name_id_absent,
            &mut self.name_id_unset,
            &mut self.missing_name_ids,
        );
        account_id(
            item.description_id,
            item.id,
            locale,
            &mut self.description_id_absent,
            &mut self.description_id_unset,
            &mut self.missing_description_ids,
        );
    }
}

fn account_id(
    value: Option<i32>,
    item_id: i32,
    locale: &BTreeSet<i32>,
    absent: &mut usize,
    unset: &mut Vec<i32>,
    missing: &mut Vec<(i32, i32)>,
) {
    match value {
        None => *absent += 1,
        Some(id) if id <= 0 => unset.push(item_id),
        Some(id) if !locale.contains(&id) => {
            missing.push((item_id, id));
        }
        Some(_) => {}
    }
}
