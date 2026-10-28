//! T870 creative decorate catalog: every row of `housing_fixtures.json` is placeable without
//! owning it, and a piece placed that way never becomes an item.
//!
//! The setting is `housing_settings.json` next to `houses.json` (`creative_catalog`). A content
//! directory without the file keeps the owned-item flow exactly; the shipped `data/content` turns
//! it on, and a server turns it off by editing the file.
//!
//! Ids (R, server-side only; the client treats both as opaque guids and dwords). Real inventory
//! ids are positive `i32`s counted up from a small number, so the upper half of that range is
//! reserved here, in two bands:
//! * a **selector** names one catalog row in the tray and in 127/1 and 127/2:
//!   `SELECTOR_BASE + item definition id`;
//! * a **placed id** is the `item_instance_id` a catalog placement saves, one per piece (a lot
//!   and a character's lots hold each only once), taken above every placed id in the lots.
//!
//! Whether a saved fixture is creative is therefore its `item_instance_id` alone: there is no
//! second flag to keep in sync, a pickup needs no setting, and a server that turns the setting
//! off later still never turns a creative piece into an item.

use super::ServiceError;
use crate::housing::LotSnapshot;
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path};

/// First reserved id. Bit 30 keeps every id a positive `i32`, the type of an inventory item id.
pub const SELECTOR_BASE: u32 = 1 << 30;
/// Selectors cover item definition ids `1..SELECTOR_SPAN` (the table's largest is under 80,000).
const SELECTOR_SPAN: u32 = 1 << 20;
/// First placed id: above the selector band.
const PLACED_BASE: u32 = SELECTOR_BASE + SELECTOR_SPAN;
/// One past the last reserved id (`i32::MAX + 1`).
const RESERVED_END: u32 = 1 << 31;
/// RECONSTRUCTED bound for the settings file.
const MAX_SETTINGS_BYTES: u64 = 4096;

/// The tray and selection id of the catalog row for an item. `None` when the id has no selector.
pub fn selector(item_definition_id: i32) -> Option<u32> {
    let id = u32::try_from(item_definition_id).ok()?;
    (1..SELECTOR_SPAN).contains(&id).then(|| SELECTOR_BASE + id)
}

/// The item a selector names; `None` for any other id.
pub fn selected_item(id: u32) -> Option<i32> {
    let item = id
        .checked_sub(SELECTOR_BASE)
        .filter(|&n| 0 < n && n < SELECTOR_SPAN)?;
    i32::try_from(item).ok()
}

/// True for the ids a catalog placement saves; a saved fixture with one is creative.
pub fn is_placed_id(id: u32) -> bool {
    (PLACED_BASE..RESERVED_END).contains(&id)
}

/// True for every id the catalog owns. An inventory row with one is not furniture the character
/// owns (`owned_instance`), so a catalog id can never alias a real item.
pub fn is_reserved_id(id: u32) -> bool {
    (SELECTOR_BASE..RESERVED_END).contains(&id)
}

/// The id for the next catalog placement: above every placed id the lots hold. `None` only when
/// the band is full.
pub fn next_placed_id(lots: &[LotSnapshot]) -> Option<u32> {
    let held = lots
        .iter()
        .flat_map(|lot| &lot.fixtures)
        .map(|fixture| fixture.item_instance_id)
        .filter(|&id| is_placed_id(id))
        .max();
    let next = held.map_or(Some(PLACED_BASE), |id| id.checked_add(1))?;
    is_placed_id(next).then_some(next)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    schema_version: u32,
    creative_catalog: bool,
}

/// `creative_catalog` from `directory/housing_settings.json`; off without that file. A file that
/// is present but unusable is refused rather than read as off, so a typo cannot silently change
/// what a server offers.
pub fn catalog_setting(directory: &Path) -> Result<bool, ServiceError> {
    const INVALID: ServiceError = ServiceError::HousingContentInvalid("invalid housing settings");
    let path = directory.join("housing_settings.json");
    if !path.is_file() {
        return Ok(false);
    }
    let mut bytes = Vec::new();
    File::open(&path)
        .and_then(|file| file.take(MAX_SETTINGS_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| INVALID)?;
    match serde_json::from_slice::<Settings>(&bytes) {
        Ok(settings)
            if bytes.len() as u64 <= MAX_SETTINGS_BYTES && settings.schema_version == 1 =>
        {
            Ok(settings.creative_catalog)
        }
        _ => Err(INVALID),
    }
}
