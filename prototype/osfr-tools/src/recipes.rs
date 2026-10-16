//! Crafting recipe content. Recipe ids and server rules remain unresolved
//! unless the record carries direct source evidence.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs, io,
    path::Path,
};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CraftingJob {
    Chef,
    Blacksmith,
    Miner,
}

impl CraftingJob {
    pub const fn profile_id(self) -> i32 {
        match self {
            Self::Chef => 45,
            Self::Blacksmith => 16,
            Self::Miner => 14,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
    tag = "kind",
    content = "evidence",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum SourceTag {
    Client(String),
    Video(String),
    Wiki(String),
    Legacy(String),
    Reconstructed(String),
}

impl SourceTag {
    fn evidence(&self) -> &str {
        match self {
            Self::Client(value)
            | Self::Video(value)
            | Self::Wiki(value)
            | Self::Legacy(value)
            | Self::Reconstructed(value) => value,
        }
    }

    fn is_reconstructed(&self) -> bool {
        matches!(self, Self::Reconstructed(_))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeIngredient {
    pub definition_id: i32,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeOutput {
    pub definition_id: i32,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// Stable content key; server recipe id may be absent when unrecovered.
    pub key: String,
    pub name: String,
    pub recipe_id: Option<u32>,
    pub candidate_recipe_id: Option<u32>,
    pub recipe_item_definition_id: Option<i32>,
    pub job: CraftingJob,
    pub job_profile_id: i32,
    pub required_job_rank: Option<u8>,
    pub minigame_type_id: u32,
    pub minigame_id: Option<u32>,
    pub ingredients: Vec<RecipeIngredient>,
    pub output: RecipeOutput,
    /// (R) Local XP policy; the native recipe row has no XP column and no
    /// per-recipe XP amount was recovered from the wiki.
    pub experience_points: u32,
    /// Provenance for each populated field, including each ingredient value.
    pub field_sources: BTreeMap<String, SourceTag>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeFile {
    pub schema_version: u32,
    pub records: Vec<Recipe>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedRecipe {
    pub key: String,
    pub job: CraftingJob,
    pub name: String,
    pub candidate_recipe_id: Option<u32>,
    pub candidate_recipe_item_definition_id: Option<i32>,
    pub candidate_output_definition_id: Option<i32>,
    pub candidate_required_job_rank: Option<u8>,
    pub minigame_type_id: u32,
    pub minigame_id: Option<u32>,
    pub missing_fields: Vec<String>,
    /// Provenance for each populated candidate field.
    pub field_sources: BTreeMap<String, SourceTag>,
    pub source: Vec<SourceTag>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedRecipeFile {
    pub schema_version: u32,
    pub records: Vec<UnresolvedRecipe>,
}

#[derive(Clone, Debug)]
pub struct RecipeCatalog {
    pub records: Vec<Recipe>,
}

#[derive(Clone, Debug)]
pub struct UnresolvedRecipeCatalog {
    pub records: Vec<UnresolvedRecipe>,
}

impl RecipeCatalog {
    /// Load the typed runtime catalogue. Its references are rechecked against
    /// the item ids declared by the same content file; the independent build
    /// gate checks those ids against the pinned client item definitions.
    pub fn load_runtime(directory: &Path) -> io::Result<Self> {
        let packaged = directory.join("recipes.json");
        let path = if packaged.is_file() {
            packaged
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/recipes.json")
        };
        let file: RecipeFile = read_json(&path)?;
        let ids: HashSet<i32> = file
            .records
            .iter()
            .flat_map(|recipe| {
                recipe
                    .ingredients
                    .iter()
                    .map(|item| item.definition_id)
                    .chain([recipe.output.definition_id])
                    .chain(recipe.recipe_item_definition_id)
            })
            .collect();
        Self::load(path, &ids)
    }

    pub fn load(path: impl AsRef<Path>, item_definitions: &HashSet<i32>) -> io::Result<Self> {
        let file: RecipeFile = read_json(path)?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(invalid(format!(
                "schema_version {}, expected {SCHEMA_VERSION}",
                file.schema_version
            )));
        }

        let mut keys = HashSet::new();
        let mut recipe_ids = HashSet::new();
        for recipe in &file.records {
            validate_recipe(recipe, item_definitions)?;
            if !keys.insert(recipe.key.as_str()) {
                return Err(invalid(format!("duplicate recipe key {}", recipe.key)));
            }
            if let Some(id) = recipe.recipe_id {
                if !recipe_ids.insert(id) {
                    return Err(invalid(format!("duplicate recipe id {id}")));
                }
            }
        }
        Ok(Self {
            records: file.records,
        })
    }

    pub fn by_job(&self, job: CraftingJob) -> impl Iterator<Item = &Recipe> {
        self.records.iter().filter(move |recipe| recipe.job == job)
    }

    /// Exact required-rank lookup. Unknown ranks are not treated as rank 1.
    pub fn by_rank(&self, job: CraftingJob, rank: u8) -> impl Iterator<Item = &Recipe> {
        self.by_job(job)
            .filter(move |recipe| recipe.required_job_rank == Some(rank))
    }

    pub fn by_output(&self, definition_id: i32) -> impl Iterator<Item = &Recipe> {
        self.records
            .iter()
            .filter(move |recipe| recipe.output.definition_id == definition_id)
    }

    pub fn by_key(&self, key: &str) -> Option<&Recipe> {
        self.records.iter().find(|recipe| recipe.key == key)
    }

    /// Encode learned records for the native recipe table. The client row has
    /// no recovered policy fields for description, tint, membership or filter;
    /// those values are local defaults (R). Missing pattern presentation falls
    /// back to the output item so a learned row remains displayable.
    pub fn wire_rows(
        &self,
        learned: &HashSet<u32>,
        inventory: &crate::inventory::Inventory,
    ) -> io::Result<Vec<(u32, RecipeWireRow)>> {
        let mut rows = Vec::new();
        for recipe in &self.records {
            let Some(id) = recipe.recipe_id.filter(|id| learned.contains(id)) else {
                continue;
            };
            let Some(pattern_id) = recipe.recipe_item_definition_id else {
                continue;
            };
            let presentation = inventory
                .presentation(pattern_id)
                .or_else(|_| inventory.presentation(recipe.output.definition_id))
                .ok();
            let components = recipe
                .ingredients
                .iter()
                .map(|ingredient| {
                    let item = inventory.presentation(ingredient.definition_id).ok();
                    Ok(RecipeWireComponent {
                        name_id: item.map_or(0, |item| item.name_id.max(0) as u32),
                        icon_id: item.map_or(0, |item| item.icon_id.max(0) as u32),
                        tint_key: item.map_or(0, |item| item.icon_tint.max(0) as u32),
                        description_id: 0, // (R) No recipe-component description mapping.
                        required_count: ingredient.count,
                        inventory_key: ingredient.definition_id as u32, // (R) Content definition key.
                        unknown_20: 0,
                        item_definition_id: ingredient.definition_id as u32,
                    })
                })
                .collect::<io::Result<Vec<_>>>()?;
            rows.push((
                id,
                RecipeWireRow {
                    id,
                    name_id: presentation.map_or(0, |item| item.name_id.max(0) as u32),
                    icon_id: presentation.map_or(0, |item| item.icon_id.max(0) as u32),
                    tint_key: presentation.map_or(0, |item| item.icon_tint.max(0) as u32),
                    description_id: 0, // (R) Recipe description string mapping is unavailable.
                    unknown_1c: 0,
                    members_only: false,
                    filter_type: recipe.job_profile_id as u32, // (R) Stable local filter mapping.
                    components,
                    item_definition_id: pattern_id as u32,
                    passes_requirement: true,
                    unknown_2c: 0,
                },
            ));
        }
        rows.sort_by_key(|(id, _)| *id);
        Ok(rows)
    }
}

impl UnresolvedRecipeCatalog {
    pub fn load(path: impl AsRef<Path>, item_definitions: &HashSet<i32>) -> io::Result<Self> {
        let file: UnresolvedRecipeFile = read_json(path)?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(invalid(format!(
                "schema_version {}, expected {SCHEMA_VERSION}",
                file.schema_version
            )));
        }

        let mut keys = HashSet::new();
        for recipe in &file.records {
            if recipe.key.trim().is_empty() || recipe.name.trim().is_empty() {
                return Err(invalid("unresolved recipe key and name must not be empty"));
            }
            if recipe.minigame_type_id == 0 || recipe.missing_fields.is_empty() {
                return Err(invalid(format!(
                    "unresolved recipe {} needs a minigame type and missing fields",
                    recipe.key
                )));
            }
            if recipe.candidate_recipe_id.is_some_and(|id| id == 0)
                || recipe.minigame_id.is_some_and(|id| id == 0)
                || recipe
                    .candidate_required_job_rank
                    .is_some_and(|rank| !(1..=20).contains(&rank))
            {
                return Err(invalid(format!(
                    "unresolved recipe {} has an invalid candidate value",
                    recipe.key
                )));
            }
            for (field, id) in [
                (
                    "candidate_recipe_item_definition_id",
                    recipe.candidate_recipe_item_definition_id,
                ),
                (
                    "candidate_output_definition_id",
                    recipe.candidate_output_definition_id,
                ),
            ] {
                if let Some(id) = id {
                    item_reference(&recipe.key, field, id, item_definitions)?;
                }
            }
            validate_field_sources(&recipe.key, &recipe.field_sources)?;
            for field in ["key", "name", "job", "minigame_type_id"] {
                require_field_source(&recipe.key, field, &recipe.field_sources)?;
            }
            for (field, present) in [
                ("candidate_recipe_id", recipe.candidate_recipe_id.is_some()),
                (
                    "candidate_recipe_item_definition_id",
                    recipe.candidate_recipe_item_definition_id.is_some(),
                ),
                (
                    "candidate_output_definition_id",
                    recipe.candidate_output_definition_id.is_some(),
                ),
                (
                    "candidate_required_job_rank",
                    recipe.candidate_required_job_rank.is_some(),
                ),
                ("minigame_id", recipe.minigame_id.is_some()),
            ] {
                if present {
                    require_field_source(&recipe.key, field, &recipe.field_sources)?;
                }
            }
            validate_sources(&recipe.key, &recipe.source)?;
            if !keys.insert(recipe.key.as_str()) {
                return Err(invalid(format!(
                    "duplicate unresolved recipe key {}",
                    recipe.key
                )));
            }
        }
        Ok(Self {
            records: file.records,
        })
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> io::Result<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)?;
    serde_json::from_str(&raw).map_err(|error| invalid(format!("{}: {error}", path.display())))
}

fn validate_recipe(recipe: &Recipe, item_definitions: &HashSet<i32>) -> io::Result<()> {
    if recipe.key.trim().is_empty() || recipe.name.trim().is_empty() {
        return Err(invalid("recipe key and name must not be empty"));
    }
    if recipe.job_profile_id != recipe.job.profile_id() {
        return Err(invalid(format!(
            "recipe {} has profile {}, expected {} for {:?}",
            recipe.key,
            recipe.job_profile_id,
            recipe.job.profile_id(),
            recipe.job
        )));
    }
    if recipe.minigame_type_id == 0
        || recipe.minigame_id.is_some_and(|id| id == 0)
        || recipe.recipe_id.is_some_and(|id| id == 0)
        || recipe.candidate_recipe_id.is_some_and(|id| id == 0)
        || recipe
            .required_job_rank
            .is_some_and(|rank| !(1..=20).contains(&rank))
    {
        return Err(invalid(format!(
            "recipe {} has an invalid id, type, or job rank",
            recipe.key
        )));
    }
    if recipe.ingredients.is_empty() || recipe.output.count == 0 {
        return Err(invalid(format!(
            "recipe {} needs ingredients and a positive output count",
            recipe.key
        )));
    }
    let mut ingredients = HashSet::new();
    for ingredient in &recipe.ingredients {
        if ingredient.definition_id <= 0 || ingredient.count == 0 {
            return Err(invalid(format!(
                "recipe {} has an invalid ingredient",
                recipe.key
            )));
        }
        if !ingredients.insert(ingredient.definition_id) {
            return Err(invalid(format!(
                "recipe {} repeats ingredient {}",
                recipe.key, ingredient.definition_id
            )));
        }
        item_reference(
            &recipe.key,
            "ingredients.definition_id",
            ingredient.definition_id,
            item_definitions,
        )?;
    }
    if recipe.output.definition_id <= 0 {
        return Err(invalid(format!(
            "recipe {} has invalid output definition id",
            recipe.key
        )));
    }
    item_reference(
        &recipe.key,
        "output.definition_id",
        recipe.output.definition_id,
        item_definitions,
    )?;
    if let Some(id) = recipe.recipe_item_definition_id {
        item_reference(
            &recipe.key,
            "recipe_item_definition_id",
            id,
            item_definitions,
        )?;
    }
    validate_field_sources(&recipe.key, &recipe.field_sources)?;
    for field in [
        "key",
        "name",
        "job",
        "job_profile_id",
        "minigame_type_id",
        "experience_points",
        "output.definition_id",
        "output.count",
    ] {
        require_field_source(&recipe.key, field, &recipe.field_sources)?;
    }
    for (field, present) in [
        ("recipe_id", recipe.recipe_id.is_some()),
        ("candidate_recipe_id", recipe.candidate_recipe_id.is_some()),
        (
            "recipe_item_definition_id",
            recipe.recipe_item_definition_id.is_some(),
        ),
        ("required_job_rank", recipe.required_job_rank.is_some()),
        ("minigame_id", recipe.minigame_id.is_some()),
    ] {
        if present {
            require_field_source(&recipe.key, field, &recipe.field_sources)?;
        }
    }
    for index in 0..recipe.ingredients.len() {
        require_field_source(
            &recipe.key,
            &format!("ingredients[{index}].definition_id"),
            &recipe.field_sources,
        )?;
        require_field_source(
            &recipe.key,
            &format!("ingredients[{index}].count"),
            &recipe.field_sources,
        )?;
    }
    validate_sources(&recipe.key, &recipe.source)
}

fn item_reference(
    key: &str,
    field: &str,
    id: i32,
    item_definitions: &HashSet<i32>,
) -> io::Result<()> {
    if item_definitions.contains(&id) {
        Ok(())
    } else {
        Err(invalid(format!(
            "{key}.{field} references missing item {id}"
        )))
    }
}

fn validate_sources(key: &str, sources: &[SourceTag]) -> io::Result<()> {
    if sources.is_empty() {
        return Err(invalid(format!("{key} has no source provenance")));
    }
    for source in sources {
        let evidence = source.evidence().trim();
        if evidence.is_empty() || (source.is_reconstructed() && !evidence.starts_with("(R)")) {
            return Err(invalid(format!("{key} has invalid source provenance")));
        }
    }
    Ok(())
}

fn validate_field_sources(key: &str, sources: &BTreeMap<String, SourceTag>) -> io::Result<()> {
    for (field, source) in sources {
        if field.trim().is_empty() {
            return Err(invalid(format!("{key} has an empty field provenance key")));
        }
        validate_sources(key, std::slice::from_ref(source))?;
    }
    Ok(())
}

fn require_field_source(
    key: &str,
    field: &str,
    sources: &BTreeMap<String, SourceTag>,
) -> io::Result<()> {
    if sources.contains_key(field) {
        Ok(())
    } else {
        Err(invalid(format!("{key}.{field} has no field provenance")))
    }
}

/// Native recipe messages use client message ID 0x003f followed by a byte
/// operation code. The parser chain is 009471a0 -> 00a12740 -> 00c8d170.
pub const RECIPE_MESSAGE_ID: u16 = 0x003f;

/// Values are kept by native row offset where the client reader does not give
/// the field a semantic label. Order follows 0090a720 and 008db930.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeWireComponent {
    pub name_id: u32,
    pub icon_id: u32,
    pub tint_key: u32,
    pub description_id: u32,
    pub required_count: u32,
    pub inventory_key: u32,
    pub unknown_20: u32,
    pub item_definition_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeWireRow {
    pub id: u32,
    pub name_id: u32,
    pub icon_id: u32,
    pub tint_key: u32,
    pub description_id: u32,
    pub unknown_1c: u32,
    pub members_only: bool,
    pub filter_type: u32,
    pub components: Vec<RecipeWireComponent>,
    pub item_definition_id: u32,
    pub passes_requirement: bool,
    pub unknown_2c: u32,
}

/// RecipeAddPacket, operation 1. `RecipeData` field order comes from the
/// bounded reader at 0090a720; component fields are read at 008db930.
pub fn encode_recipe_add(row: &RecipeWireRow) -> io::Result<Vec<u8>> {
    let mut bytes = recipe_header(1);
    encode_recipe_row(&mut bytes, row)?;
    Ok(bytes)
}

/// RecipeComponentUpdatePacket, operation 2. The last word updates the
/// component value stored at row offset +0x1c; its higher-level meaning varies
/// with the inventory lookup and is intentionally not renamed as a count.
pub fn encode_component_update(recipe_id: u32, component_index: u32, value: u32) -> Vec<u8> {
    let mut bytes = recipe_header(2);
    write_u32(&mut bytes, recipe_id);
    write_u32(&mut bytes, component_index);
    write_u32(&mut bytes, value);
    bytes
}

/// RecipeRemovePacket, operation 3.
pub fn encode_recipe_remove(recipe_id: u32) -> Vec<u8> {
    let mut bytes = recipe_header(3);
    write_u32(&mut bytes, recipe_id);
    bytes
}

/// RecipeUpdatePacket, operation 5: u32 row count, then repeated hash key and
/// RecipeData body. The client's 00c8beb0 reader replaces the old list.
pub fn encode_recipe_replace_all(rows: &[(u32, RecipeWireRow)]) -> io::Result<Vec<u8>> {
    let count = u32::try_from(rows.len())
        .map_err(|_| invalid("recipe replacement contains more than u32::MAX rows"))?;
    let mut bytes = recipe_header(5);
    write_u32(&mut bytes, count);
    for (key, row) in rows {
        write_u32(&mut bytes, *key);
        encode_recipe_row(&mut bytes, row)?;
    }
    Ok(bytes)
}

fn recipe_header(operation: u8) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(3);
    bytes.extend_from_slice(&RECIPE_MESSAGE_ID.to_le_bytes());
    bytes.push(operation);
    bytes
}

fn encode_recipe_row(bytes: &mut Vec<u8>, row: &RecipeWireRow) -> io::Result<()> {
    let count = u32::try_from(row.components.len())
        .map_err(|_| invalid("recipe contains more than u32::MAX components"))?;
    write_u32(bytes, row.id);
    write_u32(bytes, row.name_id);
    write_u32(bytes, row.icon_id);
    write_u32(bytes, row.tint_key);
    write_u32(bytes, row.description_id);
    write_u32(bytes, row.unknown_1c);
    bytes.push(u8::from(row.members_only));
    write_u32(bytes, row.filter_type);
    write_u32(bytes, count);
    for component in &row.components {
        write_u32(bytes, component.name_id);
        write_u32(bytes, component.icon_id);
        write_u32(bytes, component.tint_key);
        write_u32(bytes, component.description_id);
        write_u32(bytes, component.required_count);
        write_u32(bytes, component.inventory_key);
        write_u32(bytes, component.unknown_20);
        write_u32(bytes, component.item_definition_id);
    }
    write_u32(bytes, row.item_definition_id);
    bytes.push(u8::from(row.passes_requirement));
    write_u32(bytes, row.unknown_2c);
    Ok(())
}

fn write_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub fn ids_from_item_definitions_json(raw: &str) -> io::Result<HashSet<i32>> {
    let rows: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(raw)
        .map_err(|error| invalid(format!("invalid item definitions: {error}")))?;
    let mut ids = HashSet::with_capacity(rows.len());
    for row in rows {
        let id = row
            .get("Id")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .ok_or_else(|| invalid("item definition is missing an i32 Id"))?;
        ids.insert(id);
    }
    Ok(ids)
}
