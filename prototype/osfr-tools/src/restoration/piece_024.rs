//! Work item 024: furniture catalogue relationships (asset / item / model / locale).
//!
//! Every field here is an ORIGINAL fact read from a preserved source; this module invents no
//! catalogue record, price, stack size or placement rule:
//!
//! * `ClientItemDefinitions.json` (AGPL-3.0, `raisingkaines/FreeRealms-Legacy`, revision
//!   `9e082025391177d5a33ee18c714ddd06b457a695`, sha256
//!   `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`) supplies the stock item
//!   rows: `Id`, `NameId`, `DescriptionId`, `Icon.Id`, `CategoryId`, `ModelName`.
//! * `rust-asset-catalog.tsv` (the Rust pack reader over the 24 preserved client packs, 108,543
//!   distinct names) supplies the model and icon asset names.
//! * `locale-id-to-text.json` (unmodified FreeRealmsLocaleTools export of the supplied client
//!   `locale/en_us_data.dat`/`.dir`) supplies the name and description strings.
//! * the current reconstructed `inventory.json` supplies the server-side definitions the
//!   relationship delta is measured against.
//!
//! The furniture selector is the recovered `hsg_` model naming convention, not a server rule: no
//! recovered table states "this item is furniture". Housing categories are accepted as a caller
//! input rather than hard-coded, so the same checker can be rerun when the categories change.
//!
//! ponytail: one pass over each table, sets and vectors only. No index files, no packet writer, no
//! embedded catalogue dump. Add a database or cache only when a caller proves the scan is hot.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// Recovered naming convention for housing/furniture model files.
pub const FURNITURE_MODEL_PREFIX: &str = "hsg_";
/// Recovered naming convention for item icon textures.
pub const ITEM_ICON_PREFIX: &str = "icon_item_";

/// Stock catalogue row, deserialized straight from `ClientItemDefinitions.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct RawItem {
    #[serde(rename = "Id")]
    pub id: i32,
    #[serde(rename = "NameId", default)]
    pub name_id: Option<i32>,
    #[serde(rename = "DescriptionId", default)]
    pub description_id: Option<i32>,
    #[serde(rename = "Icon", default)]
    pub icon: Option<RawIcon>,
    #[serde(rename = "CategoryId", default)]
    pub category_id: Option<i32>,
    #[serde(rename = "ModelName", default)]
    pub model: Option<String>,
}

/// Stock catalogue icon block.
///
/// The shipped catalogue contains rows whose `Icon` block carries only a `TintId` (for example
/// item 15490 "Wolf"), so the icon id is optional rather than assumed present.
#[derive(Debug, Clone, Deserialize)]
pub struct RawIcon {
    #[serde(rename = "Id", default)]
    pub id: Option<i32>,
}

/// One reconstructed definition from the server content file.
#[derive(Debug, Clone, Deserialize)]
pub struct ReconstructedItem {
    pub id: i32,
    #[serde(default)]
    pub model: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReconstructedContent {
    #[serde(default)]
    definitions: Vec<ReconstructedItem>,
}

/// The furniture subset of the stock catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FurnitureItem {
    pub id: i32,
    pub name_id: Option<i32>,
    pub description_id: Option<i32>,
    pub icon_id: Option<i32>,
    pub category_id: Option<i32>,
    pub model: String,
}

/// Sorted relationship results. Ids stay sorted so two runs compare byte for byte.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Findings {
    pub furniture: usize,
    pub missing_models: Vec<i32>,
    pub missing_locale: Vec<i32>,
    pub missing_icon_assets: Vec<i32>,
    pub stock_only: Vec<i32>,
    pub reconstructed_furniture: usize,
    pub reconstructed_unknown: Vec<i32>,
    pub model_mismatches: Vec<i32>,
    pub duplicate_reconstructed: Vec<i32>,
}

/// True when the model file name follows the recovered `hsg_` furniture convention.
pub fn is_furniture_model(model: &str) -> bool {
    model
        .to_ascii_lowercase()
        .starts_with(FURNITURE_MODEL_PREFIX)
}

/// `icon_item_<model stem>` prefix for a furniture model, or `None` when there is no stem.
pub fn icon_asset_prefix(model: &str) -> Option<String> {
    if !is_furniture_model(model) {
        return None;
    }
    let model = model.to_ascii_lowercase();
    let stem = model.split('.').next()?;
    if stem.is_empty() {
        return None;
    }
    Some(format!("{ITEM_ICON_PREFIX}{stem}"))
}

/// Collect the furniture subset, preserving stock order.
pub fn collect_furniture(items: &[RawItem]) -> Vec<FurnitureItem> {
    items
        .iter()
        .filter_map(|item| {
            let model = item.model.clone()?;
            if !is_furniture_model(&model) {
                return None;
            }
            Some(FurnitureItem {
                id: item.id,
                name_id: item.name_id,
                description_id: item.description_id,
                icon_id: item.icon.as_ref().and_then(|icon| icon.id),
                category_id: item.category_id,
                model,
            })
        })
        .collect()
}

/// Measure the stock catalogue against the preserved assets, the locale export and the
/// reconstructed content definitions.
pub fn analyze(
    stock: &[RawItem],
    reconstructed: &[ReconstructedItem],
    assets: &BTreeSet<String>,
    locale: &BTreeSet<i32>,
) -> Findings {
    let assets: BTreeSet<String> = assets
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect();
    let icon_assets: BTreeSet<String> = assets
        .iter()
        .filter(|name| name.starts_with(ITEM_ICON_PREFIX))
        .cloned()
        .collect();
    let furniture = collect_furniture(stock);

    let mut missing_models = Vec::new();
    let mut missing_locale = Vec::new();
    let mut missing_icon_assets = Vec::new();
    for item in &furniture {
        if !assets.contains(&item.model.to_ascii_lowercase()) {
            missing_models.push(item.id);
        }
        let named = item.name_id.is_some_and(|id| locale.contains(&id));
        let described = item.description_id.is_none_or(|id| locale.contains(&id));
        if !named || !described {
            missing_locale.push(item.id);
        }
        let icon = icon_asset_prefix(&item.model)
            .is_some_and(|prefix| has_prefixed_name(&icon_assets, &prefix));
        if !icon {
            missing_icon_assets.push(item.id);
        }
    }

    let stock_models = stock_models(stock);
    let mut reconstructed_counts: BTreeMap<i32, usize> = BTreeMap::new();
    let mut reconstructed_models: BTreeMap<i32, BTreeSet<String>> = BTreeMap::new();
    for definition in reconstructed
        .iter()
        .filter(|definition| is_furniture_model(&definition.model))
    {
        *reconstructed_counts.entry(definition.id).or_default() += 1;
        reconstructed_models
            .entry(definition.id)
            .or_default()
            .insert(definition.model.to_ascii_lowercase());
    }

    let duplicate_reconstructed = reconstructed_counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(id, _)| *id)
        .collect();

    let mut reconstructed_unknown = Vec::new();
    let mut model_mismatches = Vec::new();
    for id in reconstructed_counts.keys() {
        match stock_models.get(id) {
            None => reconstructed_unknown.push(*id),
            Some(stock_model) => {
                let models = &reconstructed_models[id];
                let single = models.len() == 1;
                let matches = single
                    && models
                        .iter()
                        .all(|model| model.eq_ignore_ascii_case(stock_model));
                if !matches {
                    model_mismatches.push(*id);
                }
            }
        }
    }

    let stock_only = furniture
        .iter()
        .map(|item| item.id)
        .filter(|id| !reconstructed_counts.contains_key(id))
        .collect();

    Findings {
        furniture: furniture.len(),
        missing_models,
        missing_locale,
        missing_icon_assets,
        stock_only,
        reconstructed_furniture: reconstructed_counts.len(),
        reconstructed_unknown,
        model_mismatches,
        duplicate_reconstructed,
    }
}

/// First asset name at or after `prefix`. A `BTreeSet` range keeps the icon lookup logarithmic
/// instead of scanning 108k names per furniture row.
fn has_prefixed_name(names: &BTreeSet<String>, prefix: &str) -> bool {
    names
        .range(prefix.to_owned()..)
        .next()
        .is_some_and(|name| name.starts_with(prefix))
}

/// Asset names from the pack catalogue TSV (`name` column).
pub fn asset_names(tsv: &str) -> BTreeSet<String> {
    let mut lines = tsv.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let column = header.iter().position(|name| *name == "name").unwrap_or(2);
    lines
        .filter_map(|line| line.split('\t').nth(column))
        .filter(|name| !name.is_empty())
        .map(|name| name.to_owned())
        .collect()
}

/// Locale ids from the community reader export (`[{"id":..,"text":".."}]`).
pub fn locale_ids(json: &str) -> BTreeSet<i32> {
    #[derive(Deserialize)]
    struct Entry {
        id: i32,
    }
    serde_json::from_str::<Vec<Entry>>(json)
        .map(|entries| entries.into_iter().map(|entry| entry.id).collect())
        .unwrap_or_default()
}

/// Reconstructed definitions from the server content file.
pub fn reconstructed_definitions(json: &str) -> Vec<ReconstructedItem> {
    serde_json::from_str::<ReconstructedContent>(json)
        .map(|content| content.definitions)
        .unwrap_or_default()
}

/// Stock item id to declared model file name, for the whole catalogue.
pub fn stock_models(stock: &[RawItem]) -> BTreeMap<i32, String> {
    stock
        .iter()
        .filter_map(|item| item.model.clone().map(|model| (item.id, model)))
        .collect()
}
