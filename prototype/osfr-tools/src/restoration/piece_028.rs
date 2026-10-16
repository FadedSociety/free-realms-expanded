//! 028: housing save migration for one restored server.
//!
//! This module is the version boundary in front of the item 006 housing snapshot
//! (`src/housing.rs`, `SNAPSHOT_SCHEMA_VERSION` = 1). It reads a snapshot document, dispatches on
//! its declared `schema_version`, and hands the real `Housing::load_state` a current-version
//! document. Its job is that a migration can only ever *add* derived data: it never drops a lot, a
//! fixture, a permission or an ownership row, and it never rewrites the source file.
//!
//! ORIGINAL facts used here (all outside Git, see `notes/Research/Pieces/028 Housing save
//! migration.md`):
//! * `PlayerHousingInstanceData` carries `HouseGuid`, `OwnerGuid`, `NameId`, `MaxFixtureCount`,
//!   `MaxLandmarkCount`, `CurFixtureCount`, the fixture map, the permission map and `BuildAreas`
//!   (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs`),
//!   and `FixtureInstance` carries fixture guid, house guid, fixture definition id, position,
//!   rotation, tilt, tint id and scale. The local snapshot mirrors those fields; this module's
//!   version dispatch is over that local document, not over the wire.
//! * the recovered `Houses.json` rows (`Id`, `ZoneId`, `NameId`, `BuildAreas`) are the only
//!   original source of a lot's build rectangle; row 1 is `Id` 1, `ZoneId` 1, `NameId` 6514,
//!   x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8
//!   (`lab/references/FreeRealms-Legacy/src/Resources/Houses.json`, sha256
//!   `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`).
//! * the supplied client has a save action on the wire (`ClientHousingPacketSaveFixture`, inner
//!   opcode 5, writer `FUN_00abf620`) and the community server has **no** housing persistence at
//!   all: its `DatabaseContext` has no housing `DbSet`, its migrations add no housing table, and
//!   `BaseHousingPacketHandler` answers every unimplemented housing opcode with the literal comment
//!   `housing not implemented` (`lab/references/FreeRealms-Legacy/src/Sanctuary.Database/`,
//!   `.../Sanctuary.Database.MySql/Migrations/`, `.../Sanctuary.Gateway/Handlers/BaseHousingPacketHandler.cs`).
//!   No original server save layout was recovered, so there is no real save *format* to migrate
//!   from. That is an explicit gap, not something this module pretends to close.
//! * `src/game_store.rs` already establishes the repo convention this module follows for the
//!   housing document: an older version is upgraded in memory and the player's progress is never
//!   reset by the upgrade.
//!
//! RECONSTRUCTED here:
//! * the legacy schema version is `0` and its shape is the current per-lot snapshot without the two
//!   fields the item 006 review added (`count` and `build_areas`): a legacy document either omits
//!   `schema_version` or declares `0`, and either omits `count`/`build_areas` or carries them. No
//!   historical file of that shape is claimed to exist; it is the bounded, mechanical repair rule
//!   this module implements.
//! * a migration may only recompute `count` (the derived `CurFixtureCount`, whose authoritative
//!   source inside the document is the fixture list) and fill `build_areas` from the recovered
//!   property rows when the legacy lot has none. Identities, transforms, permissions, locks,
//!   fixture guids, ownership and property ids are copied verbatim.
//! * an unknown or newer `schema_version` is refused as `UnsupportedVersion` and must never be
//!   overwritten; a current-version document that is missing the current required fields is
//!   `InvalidDocument` and is never repaired.
//! * the staging write used to hand the migrated bytes to the real loader.
//!
//! Deliberate boundaries:
//! * `restore` does not write the source file. The caller decides when the upgraded document is
//!   persisted, and `Housing::save_state` writes the canonical v1 form atomically.
//! * this module never touches the item 026 owned-furniture pool; a placed fixture stays placed and
//!   an unplaced unit stays unplaced, so a migration cannot move a unit between those states.
//! * there is no wire-level migration: the recovered `ClientHousingPacketSaveFixture` tail is still
//!   undecoded (item 025), so no packet is translated here.
//!
//! ponytail: one version dispatch, one legacy shape, one property lookup. The canonical output is
//! serialized from the real `housing::LotSnapshot` types so it cannot drift from what item 006
//! writes; only the legacy reader is a local mirror.

use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "../housing.rs"]
#[allow(dead_code)]
mod housing;

// INTEGRATION: when the lead registers `pub mod housing;` in `src/lib.rs`, replace the `#[path]`
// include above with `use crate::housing;` and drop this re-export block. Until then the owned test
// target includes the module by path exactly as `tests/housing.rs` does.
pub use housing::{
    FixtureTransform, Housing, HousingError, LotPermission, LotSnapshot, PropertyDefinition,
    PropertyGrant, SavedFixture,
};

/// RECONSTRUCTED: the legacy document did not declare a version.
pub const LEGACY_SCHEMA_VERSION: u64 = 0;
/// The version this module writes. Mirrors `housing::SNAPSHOT_SCHEMA_VERSION` by construction.
pub const CURRENT_SCHEMA_VERSION: u32 = housing::SNAPSHOT_SCHEMA_VERSION;
/// Same bound as the item 006 loader, so both entry points refuse oversized documents identically.
pub const MAX_SNAPSHOT_BYTES: usize = housing::MAX_SNAPSHOT_BYTES;

// The canonical output must be the version item 006 writes; a bump there without a migration arm
// here has to fail at compile time rather than silently writing a document the loader rejects.
const _: () = assert!(CURRENT_SCHEMA_VERSION == 1);

static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Every way a snapshot can fail to become a current-version document. No variant is a success, and
/// no variant has touched the live state or the source file when it is returned.
#[derive(Clone, Debug, PartialEq)]
pub enum MigrationError {
    /// The source could not be read, or the staging file could not be created in its directory.
    Io(io::ErrorKind),
    /// The document is larger than `MAX_SNAPSHOT_BYTES`.
    TooLarge,
    /// The bytes are not JSON.
    Malformed { line: usize, column: usize },
    /// The JSON root is not an object, so it cannot be a snapshot document.
    NotDocument,
    /// `schema_version` is present but is not a non-negative integer.
    VersionNotInteger,
    /// `schema_version` is newer than this build writes. The source must not be overwritten.
    UnsupportedVersion { found: u64 },
    /// A document of a supported version does not match that version's shape.
    InvalidDocument { reason: String },
    /// A legacy lot has no build areas and its property id has no recovered `Houses.json` row.
    UnknownProperty { property_id: i32 },
    /// The recovered property row disagrees with the legacy lot's own zone or name id.
    PropertyMismatch {
        property_id: i32,
        field: &'static str,
        stored: i64,
        source: i64,
    },
    /// The recovered property row cannot be used as content (empty, non-finite or inverted area).
    InvalidPropertyRow { property_id: i32 },
    /// The real item 006 loader refused the migrated document. Live state is untouched.
    Refused(HousingError),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(kind) => write!(formatter, "housing snapshot io error: {kind:?}"),
            Self::TooLarge => formatter.write_str("housing snapshot exceeds the bounded size"),
            Self::Malformed { line, column } => {
                write!(formatter, "housing snapshot is not JSON at {line}:{column}")
            }
            Self::NotDocument => formatter.write_str("housing snapshot root is not an object"),
            Self::VersionNotInteger => {
                formatter.write_str("housing snapshot schema_version is not a non-negative integer")
            }
            Self::UnsupportedVersion { found } => write!(
                formatter,
                "housing snapshot schema_version {found} is newer than {CURRENT_SCHEMA_VERSION}; \
                 refusing to overwrite it"
            ),
            Self::InvalidDocument { reason } => {
                write!(formatter, "housing snapshot is invalid: {reason}")
            }
            Self::UnknownProperty { property_id } => write!(
                formatter,
                "legacy housing lot for property {property_id} has no recovered build area"
            ),
            Self::PropertyMismatch {
                property_id,
                field,
                stored,
                source,
            } => write!(
                formatter,
                "legacy housing lot for property {property_id} carries {field} {stored} but the \
                 recovered row has {source}"
            ),
            Self::InvalidPropertyRow { property_id } => write!(
                formatter,
                "recovered housing property row {property_id} has no usable build area"
            ),
            Self::Refused(error) => {
                write!(formatter, "housing loader refused the document: {error:?}")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// What a migration did, for operator reports and the independent checker. `from_version` is `0`
/// when the legacy document did not declare one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    pub from_version: u64,
    pub to_version: u32,
    pub migrated: bool,
    pub lots: usize,
    pub fixtures: usize,
    /// Legacy lots whose `count` was absent and was derived from the fixture list.
    pub derived_counts: usize,
    /// Legacy lots whose declared `count` disagreed with the fixture list and was corrected.
    pub repaired_counts: usize,
    /// Legacy lots whose build areas were absent or empty and were filled from `Houses.json` rows.
    pub filled_build_areas: usize,
}

/// The canonical current-version document. `properties` are the real item 006 snapshot rows, so the
/// serialized bytes are exactly what `Housing::save_state` writes.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CurrentDocument {
    schema_version: u32,
    properties: Vec<LotSnapshot>,
}

/// Detect the declared version without trusting the body, then migrate. A document of the current
/// version is canonicalized and returned with an empty report; a legacy document is upgraded by
/// adding only the fields the legacy shape lacked; anything else is refused.
pub fn migrate(
    bytes: &[u8],
    table: &[PropertyDefinition],
) -> Result<(Vec<u8>, MigrationReport), MigrationError> {
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(MigrationError::TooLarge);
    }
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| MigrationError::Malformed {
            line: error.line(),
            column: error.column(),
        })?;
    let object = value.as_object().ok_or(MigrationError::NotDocument)?;
    let version = match object.get("schema_version") {
        None => LEGACY_SCHEMA_VERSION,
        Some(serde_json::Value::Number(number)) => {
            number.as_u64().ok_or(MigrationError::VersionNotInteger)?
        }
        Some(_) => return Err(MigrationError::VersionNotInteger),
    };
    if version == LEGACY_SCHEMA_VERSION {
        let document: LegacyDocument = serde_json::from_slice(bytes).map_err(invalid_document)?;
        // Re-check the parsed value so the declared version can never disagree with the dispatch.
        if let Some(found) = document.schema_version {
            if found != LEGACY_SCHEMA_VERSION {
                return Err(MigrationError::UnsupportedVersion { found });
            }
        }
        let mut report = MigrationReport {
            from_version: LEGACY_SCHEMA_VERSION,
            to_version: CURRENT_SCHEMA_VERSION,
            migrated: true,
            lots: document.properties.len(),
            fixtures: 0,
            derived_counts: 0,
            repaired_counts: 0,
            filled_build_areas: 0,
        };
        let mut properties = Vec::with_capacity(document.properties.len());
        for lot in document.properties {
            // `count` is `PlayerHousingInstanceData.CurFixtureCount`: derived data whose
            // authoritative source inside the document is the fixture list.
            let count = lot.fixtures.len() as u32;
            report.fixtures += lot.fixtures.len();
            match lot.count {
                None => report.derived_counts += 1,
                Some(declared) if declared != count => report.repaired_counts += 1,
                Some(_) => {}
            }
            let build_areas = match lot.build_areas {
                Some(areas) if !areas.is_empty() => areas,
                _ => {
                    report.filled_build_areas += 1;
                    recovered_build_areas(table, &lot)?
                }
            };
            properties.push(LotSnapshot {
                owner_guid: lot.owner_guid,
                property_id: lot.property_id,
                lot_guid: lot.lot_guid,
                zone_id: lot.zone_id,
                name_id: lot.name_id,
                max_fixtures: lot.max_fixtures,
                max_landmarks: lot.max_landmarks,
                count,
                build_areas,
                fixtures: lot.fixtures,
                permissions: lot.permissions,
                locked: lot.locked,
            });
        }
        return Ok((canonical_bytes(properties)?, report));
    }
    if version != u64::from(CURRENT_SCHEMA_VERSION) {
        return Err(MigrationError::UnsupportedVersion { found: version });
    }
    let document: CurrentDocument = serde_json::from_slice(bytes).map_err(invalid_document)?;
    let report = MigrationReport {
        from_version: version,
        to_version: CURRENT_SCHEMA_VERSION,
        migrated: false,
        lots: document.properties.len(),
        fixtures: document
            .properties
            .iter()
            .map(|lot| lot.fixtures.len())
            .sum(),
        derived_counts: 0,
        repaired_counts: 0,
        filled_build_areas: 0,
    };
    Ok((canonical_bytes(document.properties)?, report))
}

/// The legacy per-lot row: the current row without the two fields the item 006 review added. Both
/// are optional here because a legacy writer may have carried either one, and a legacy lot without
/// build areas is filled from the recovered property rows rather than invented.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDocument {
    /// Present only when the legacy document declared version 0 explicitly. The dispatch above has
    /// already checked that any present value is 0.
    #[serde(default)]
    schema_version: Option<u64>,
    properties: Vec<LegacyLot>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyLot {
    owner_guid: u64,
    property_id: i32,
    lot_guid: u64,
    zone_id: i32,
    name_id: i32,
    max_fixtures: u32,
    max_landmarks: u32,
    #[serde(default)]
    count: Option<u32>,
    #[serde(default)]
    build_areas: Option<Vec<[f32; 6]>>,
    fixtures: Vec<SavedFixture>,
    #[serde(default)]
    permissions: Vec<LotPermission>,
    #[serde(default)]
    locked: bool,
}

/// A legacy lot can only borrow the build rectangle its own property id names. A row that disagrees
/// with the lot's zone or name id is not the writer's content, so it is refused instead of applied.
fn recovered_build_areas(
    table: &[PropertyDefinition],
    lot: &LegacyLot,
) -> Result<Vec<[f32; 6]>, MigrationError> {
    let row = table
        .iter()
        .find(|row| row.property_id == lot.property_id)
        .ok_or(MigrationError::UnknownProperty {
            property_id: lot.property_id,
        })?;
    for (field, stored, source) in [
        ("zone_id", i64::from(lot.zone_id), i64::from(row.zone_id)),
        ("name_id", i64::from(lot.name_id), i64::from(row.name_id)),
    ] {
        if stored != source {
            return Err(MigrationError::PropertyMismatch {
                property_id: lot.property_id,
                field,
                stored,
                source,
            });
        }
    }
    if row.build_areas.is_empty() || row.build_areas.len() > 64 {
        return Err(MigrationError::InvalidPropertyRow {
            property_id: lot.property_id,
        });
    }
    for area in &row.build_areas {
        if area.iter().any(|value| !value.is_finite()) {
            return Err(MigrationError::InvalidPropertyRow {
                property_id: lot.property_id,
            });
        }
        if area
            .iter()
            .any(|value| value.abs() > housing::MAX_FIXTURE_MAGNITUDE)
            || area[0] > area[3]
            || area[1] > area[4]
            || area[2] > area[5]
        {
            return Err(MigrationError::InvalidPropertyRow {
                property_id: lot.property_id,
            });
        }
    }
    Ok(row.build_areas.clone())
}

fn invalid_document(error: serde_json::Error) -> MigrationError {
    MigrationError::InvalidDocument {
        reason: error.to_string(),
    }
}

/// Read, migrate and adopt a snapshot. The item 006 loader runs against a staged copy of the
/// migrated bytes, so a refused document leaves the live housing state exactly as it was; the source
/// file is only ever read and never rewritten.
pub fn restore(
    housing: &mut Housing,
    path: &Path,
    table: &[PropertyDefinition],
) -> Result<MigrationReport, MigrationError> {
    let bytes = read_bounded(path)?;
    let (migrated, report) = migrate(&bytes, table)?;
    let (staging_path, mut staging) = create_staging(path)?;
    if let Err(error) = staging
        .write_all(&migrated)
        .and_then(|()| staging.sync_all())
    {
        drop(staging);
        let _ = fs::remove_file(&staging_path);
        return Err(MigrationError::Io(error.kind()));
    }
    // Windows requires closing our staging handle before the loader opens it.
    drop(staging);
    let loaded = housing.load_state(&staging_path);
    let _ = fs::remove_file(&staging_path);
    loaded.map_err(MigrationError::Refused)?;
    Ok(report)
}

fn canonical_bytes(properties: Vec<LotSnapshot>) -> Result<Vec<u8>, MigrationError> {
    let document = CurrentDocument {
        schema_version: CURRENT_SCHEMA_VERSION,
        properties,
    };
    let bytes =
        serde_json::to_vec_pretty(&document).map_err(|error| MigrationError::InvalidDocument {
            reason: error.to_string(),
        })?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(MigrationError::TooLarge);
    }
    Ok(bytes)
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, MigrationError> {
    let file = File::open(path).map_err(|error| MigrationError::Io(error.kind()))?;
    let mut bytes = Vec::new();
    file.take((MAX_SNAPSHOT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| MigrationError::Io(error.kind()))?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return Err(MigrationError::TooLarge);
    }
    Ok(bytes)
}

/// A same-directory staging file. The migrated bytes go through the real loader from here, so the
/// restore path needs a writable directory: the caller's snapshot directory is the one the server
/// already saves into.
fn create_staging(destination: &Path) -> Result<(PathBuf, File), MigrationError> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("housing");
    for _ in 0..64 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.piece-028.{}.{sequence}.staging.tmp",
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(MigrationError::Io(error.kind())),
        }
    }
    Err(MigrationError::Io(io::ErrorKind::AlreadyExists))
}
