//! T877: the release item table, `item-table.json` beside `inventory.json`.
//!
//! One compact row per item, generated from the original item table alone by
//! `scripts/build_item_catalogue.py --release-table`. The loader rebuilds each item's serialized
//! `ClientItemDefinition` row (field order: reader `008e3760`, extension `00919e10`) the way the
//! generator's established encoder does; `wire_sha256` pins the two against each other.
//! Fields the original table does not name stay zero (R): the tint, resell value, VIP and race-set
//! requirements, `PARAM3`, and the stat and ability lists.
use std::{collections::BTreeMap, io};

use serde::Deserialize;

use super::{Definition, MAX_CATALOGUE_BYTES, MAX_CATALOGUE_ROWS, Requirement, invalid, put};

/// Positional columns of a row's integer list, then its flag bits, then its three texts. The file
/// repeats both lists, and the loader refuses a file whose lists differ.
const COLUMNS: [&str; 26] = [
    "NameId",
    "DescriptionId",
    "IconData.Id",
    "IconData.TintId",
    "Cost",
    "Class",
    "ProfileOverride",
    "Slot",
    "GenderUsage",
    "Type",
    "CategoryId",
    "WeaponTrailEffectId",
    "CompositeEffectId",
    "PowerRating",
    "MinProfileRank",
    "Rarity",
    "ActivatableAbilityId",
    "PassiveAbilityId",
    "MaxStackSize",
    "MemberDiscount",
    "Unknown36",
    "Unknown37",
    "Flags",
    "ModelName",
    "TextureAlias",
    "TintAlias",
];
const FLAGS: [&str; 7] = [
    "NoTrade",
    "NoSale",
    "MembersOnly",
    "NonMiniGame",
    "SingleUse",
    "IsTintable",
    "ForceDisablePreview",
];
const INTS: usize = 23;
const TEXT_MAX: usize = 4096;

#[derive(Deserialize)]
struct Table {
    schema_version: u32,
    columns: Vec<String>,
    flags: Vec<String>,
    /// Wield type per item class, from captured rows; a class that is absent reads 0 (R).
    wield_by_class: BTreeMap<i32, i32>,
    rows: Vec<Row>,
}

/// `[id, [ints and the flag bits], model, texture, tint alias]`.
#[derive(Deserialize)]
struct Row(i32, [i32; INTS], String, String, String);

impl Row {
    /// The serialized `ClientItemDefinition` row; zeros stand for the fields the table lacks.
    fn wire(&self) -> Vec<u8> {
        let Self(id, v, model, texture, alias) = self;
        let flag = |bit: usize| u8::from((v[22] >> bit) & 1 != 0);
        let mut out = Vec::new();
        let words = |out: &mut Vec<u8>, words: &[i32]| words.iter().for_each(|w| put(out, *w));
        let text = |out: &mut Vec<u8>, text: &str| {
            put(out, text.len() as i32);
            out.extend(text.as_bytes());
        };
        words(
            &mut out,
            &[*id, v[0], v[1], v[2], v[3], 0, 0, v[4], v[5], v[6], v[7]],
        );
        out.extend([flag(0), flag(1)]);
        text(&mut out, model);
        text(&mut out, texture);
        words(&mut out, &[v[8], v[9], v[10]]);
        out.extend([flag(2), flag(3)]);
        words(&mut out, &v[11..18]);
        out.push(flag(4));
        words(&mut out, &[v[18]]);
        out.push(flag(5));
        text(&mut out, alias);
        out.push(flag(6));
        words(&mut out, &[v[19], 0, 0, 0, 0, v[20], v[21], 0, 0, 0]);
        out
    }
}

/// Every definition in a table file, bounded like the lab catalogue.
pub(super) fn load(bytes: &[u8]) -> io::Result<BTreeMap<i32, Definition>> {
    let table: Table = serde_json::from_slice(bytes).map_err(invalid)?;
    if table.schema_version != 1
        || table.columns != COLUMNS
        || table.flags != FLAGS
        || table.rows.len() > MAX_CATALOGUE_ROWS
    {
        return Err(invalid("unsupported item table"));
    }
    let (mut definitions, mut total) = (BTreeMap::new(), 0usize);
    for row in &table.rows {
        let Row(id, v, model, texture, alias) = row;
        if *id <= 0 || v[22] >> FLAGS.len() != 0 || v[22] < 0 {
            return Err(invalid("invalid item table row"));
        }
        if [model, texture, alias]
            .iter()
            .any(|text| text.len() > TEXT_MAX)
        {
            return Err(invalid("item table text too long"));
        }
        let bytes = row.wire();
        total += bytes.len();
        if total > MAX_CATALOGUE_BYTES {
            return Err(invalid("item table byte limit exceeded"));
        }
        let definition = Definition {
            id: *id,
            slot: v[7],
            class_id: v[5],
            default_tint: v[3],
            max_stack: v[18],
            model: model.clone(),
            texture: texture.clone(),
            tint_alias: alias.clone(),
            effect: v[12],
            wield_type: table.wield_by_class.get(&v[5]).copied().unwrap_or(0),
            serialized_hex: String::new(),
            sha256: String::new(),
            requirement: Requirement::from_wire(&bytes),
            bytes,
        };
        if definitions.insert(*id, definition).is_some() {
            return Err(invalid("duplicate item table row"));
        }
    }
    Ok(definitions)
}
