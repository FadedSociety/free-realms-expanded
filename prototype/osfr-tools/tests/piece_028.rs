//! Item 028: housing save migration.
//!
//! These vectors run against `src/restoration/piece_028.rs`, which includes `src/housing.rs` by
//! `#[path]` so the pair compiles before the lead registers either module in `src/lib.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside Git):
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs`
//!   — a housing instance carries the house guid, owner guid, name id/name, max fixture and landmark
//!   counts, the fixture map, the permission map, `CurFixtureCount` and `BuildAreas`; the local
//!   snapshot mirrors that field set.
//! * `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` — the packet family names
//!   `ClientHousingPacketSaveFixture` as inner opcode 5; `lab/decompiled/client/FreeRealms/pseudocode.c`
//!   names the shipped client's writer `FUN_00abf620`. Its body beyond the leading fields is
//!   undecoded (item 025), so nothing here is wire-level.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Database/DatabaseContext.cs`, its MySql
//!   migrations and `.../Sanctuary.Gateway/Handlers/BaseHousingPacketHandler.cs` — the community
//!   server stores no housing row anywhere (`housing not implemented`), so no original save
//!   document exists to migrate; the legacy shape here is explicit local reconstruction.
//! * `lab/references/FreeRealms-Legacy/src/Resources/Houses.json` (sha256
//!   `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`) — rows 1, 2 and 3 are used
//!   verbatim, including the build areas that a legacy lot does not carry.
//!
//! Recipe ids 17310/17311, fixture guids and the 400 fixture cap are local vector values, not
//! recovered content. `count`, `locked` and level-3 permissions exist in the current schema; the
//! legacy shape here is that schema without `count` and `build_areas`.

#[path = "../src/restoration/piece_028.rs"]
#[allow(dead_code)]
mod piece_028;

use piece_028::{
    CURRENT_SCHEMA_VERSION, FixtureTransform, Housing, HousingError, LEGACY_SCHEMA_VERSION,
    LotPermission, LotSnapshot, MigrationError, PropertyDefinition, PropertyGrant, SavedFixture,
    migrate, restore,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const OWNER: u64 = 0x1122_3344_5566_7788;
const OTHER: u64 = 0x8877_6655_4433_2211;
const CHAIR: i32 = 17_310;
const TABLE: i32 = 17_311;
const FIXTURE_ONE: u64 = 0x0f00_0000_0000_1001;
const FIXTURE_TWO: u64 = 0x0f00_0000_0000_1002;
const FIXTURE_THREE: u64 = 0x0f00_0000_0000_1003;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-028-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }

    fn write_bytes(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn write_json(&self, name: &str, value: &Value) -> PathBuf {
        self.write_bytes(name, &serde_json::to_vec_pretty(value).unwrap())
    }

    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// ORIGINAL: `Houses.json` row 1 (`Id` 1, `ZoneId` 1, `NameId` 6514, build area
/// x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8). RECONSTRUCTED: the fixture and landmark
/// caps are local vector values; only the `HousingMaxFixtures` string was recovered, not a number.
fn property_one() -> PropertyDefinition {
    PropertyDefinition {
        property_id: 1,
        zone_id: 1,
        name_id: 6514,
        max_fixtures: 400,
        max_landmarks: 0,
        build_areas: vec![[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]],
    }
}

/// ORIGINAL: `Houses.json` row 2 (build area x 218.8..=481.3, y -100.0..=80.0, z 346.1..=505.7).
fn property_two() -> PropertyDefinition {
    PropertyDefinition {
        property_id: 2,
        zone_id: 2,
        name_id: 5_103_265,
        max_fixtures: 400,
        max_landmarks: 0,
        build_areas: vec![[218.8, -100.0, 346.1, 481.3, 80.0, 505.7]],
    }
}

/// ORIGINAL: `Houses.json` row 3 (build area x 467.2..=581.7, y -100.0..=60.0, z 514.2..=604.5).
fn property_three() -> PropertyDefinition {
    PropertyDefinition {
        property_id: 3,
        zone_id: 3,
        name_id: 69_974,
        max_fixtures: 400,
        max_landmarks: 0,
        build_areas: vec![[467.2, -100.0, 514.2, 581.7, 60.0, 604.5]],
    }
}

/// The recovered property rows this module is allowed to fill build areas from.
fn table() -> Vec<PropertyDefinition> {
    vec![property_one(), property_two(), property_three()]
}

fn transform(x: f32, y: f32, z: f32, tint_id: i32) -> FixtureTransform {
    FixtureTransform {
        position: [x, y, z],
        tint_id,
        ..FixtureTransform::default()
    }
}

fn fixture(
    fixture_guid: u64,
    house_guid: u64,
    instance_id: u32,
    definition_id: i32,
    tint_id: i32,
) -> SavedFixture {
    SavedFixture {
        fixture_guid,
        house_guid,
        item_instance_id: instance_id,
        fixture_definition_id: definition_id,
        transform: transform(400.0, 4.0, 410.0, tint_id),
        placed_by: 0,
    }
}

fn grant_lot(housing: &mut Housing, owner: u64, property: PropertyDefinition) -> u64 {
    let lot_guid = housing.lot_guid(owner, property.property_id).unwrap();
    housing
        .grant_property(PropertyGrant {
            owner_guid: owner,
            property,
            lot_guid,
        })
        .unwrap();
    lot_guid
}

/// Two owned lots with placed furniture, one of them repainted and one carrying a guest permission
/// (`InstancePermission.Level` 3, the recovered "can edit" level).
fn seed_housing() -> (Housing, [u64; 2]) {
    let mut housing = Housing::new();
    let first = grant_lot(&mut housing, OWNER, property_one());
    let second = grant_lot(&mut housing, OTHER, property_two());
    let first_snapshot = LotSnapshot {
        owner_guid: OWNER,
        property_id: 1,
        lot_guid: first,
        zone_id: 1,
        name_id: 6514,
        max_fixtures: 400,
        max_landmarks: 0,
        count: 2,
        build_areas: vec![[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]],
        fixtures: vec![
            fixture(FIXTURE_ONE, first, 1, CHAIR, 7),
            fixture(FIXTURE_TWO, first, 1, TABLE, 0),
        ],
        permissions: vec![LotPermission {
            guid: OTHER,
            level: 3,
        }],
        locked: false,
    };
    housing
        .replace_saved_fixtures(first, first_snapshot)
        .unwrap();
    let second_snapshot = LotSnapshot {
        owner_guid: OTHER,
        property_id: 2,
        lot_guid: second,
        zone_id: 2,
        name_id: 5_103_265,
        max_fixtures: 400,
        max_landmarks: 0,
        count: 1,
        build_areas: vec![[218.8, -100.0, 346.1, 481.3, 80.0, 505.7]],
        fixtures: vec![fixture(FIXTURE_THREE, second, 2, CHAIR, 2)],
        permissions: Vec::new(),
        locked: true,
    };
    housing
        .replace_saved_fixtures(second, second_snapshot)
        .unwrap();
    (housing, [first, second])
}

fn save_current(housing: &Housing, path: &Path) {
    housing.save_state(path).unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The reconstructed legacy shape: the current document without the version and without the two
/// fields the item 006 review added to a lot (`count`, `build_areas`).
fn legacy_form(current: &Value) -> Value {
    let mut value = current.clone();
    let object = value.as_object_mut().unwrap();
    object.remove("schema_version");
    for lot in object
        .get_mut("properties")
        .unwrap()
        .as_array_mut()
        .unwrap()
    {
        let lot = lot.as_object_mut().unwrap();
        lot.remove("count");
        lot.remove("build_areas");
    }
    value
}

fn sha256(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(fs::read(path).unwrap());
    format!("{:x}", hasher.finalize())
}

fn assert_same_lots(left: &Housing, right: &Housing, lots: &[u64]) {
    for lot in lots {
        assert_eq!(
            left.saved_fixtures(*lot).unwrap(),
            right.saved_fixtures(*lot).unwrap(),
            "lot {lot:#x} must be preserved exactly"
        );
    }
}

#[test]
fn current_snapshot_restores_every_lot_and_fixture() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let path = fixture.path("housing.snapshot.json");
    save_current(&source, &path);

    let (migrated, report) = migrate(&fs::read(&path).unwrap(), &table()).unwrap();
    assert_eq!(report.from_version, u64::from(CURRENT_SCHEMA_VERSION));
    assert_eq!(report.to_version, CURRENT_SCHEMA_VERSION);
    assert!(!report.migrated);
    assert_eq!(report.lots, 2);
    assert_eq!(report.fixtures, 3);
    assert_eq!(report.derived_counts, 0);
    assert_eq!(report.repaired_counts, 0);
    assert_eq!(report.filled_build_areas, 0);

    let canonical = fixture.write_bytes("migrated.snapshot.json", &migrated);
    let mut restored = Housing::new();
    let report = restore(&mut restored, &canonical, &table()).unwrap();
    assert!(!report.migrated);
    assert_same_lots(&source, &restored, &lots);
}

#[test]
fn legacy_versionless_snapshot_migrates_and_keeps_every_owned_lot_and_fixture() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let legacy = fixture.write_json(
        "legacy.snapshot.json",
        &legacy_form(&read_json(&current_path)),
    );
    let before = sha256(&legacy);

    let mut restored = Housing::new();
    let report = restore(&mut restored, &legacy, &table()).unwrap();

    assert!(report.migrated);
    assert_eq!(report.from_version, LEGACY_SCHEMA_VERSION);
    assert_eq!(report.to_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(report.lots, 2);
    assert_eq!(report.fixtures, 3);
    assert_eq!(report.derived_counts, 2);
    assert_eq!(report.repaired_counts, 0);
    assert_eq!(report.filled_build_areas, 2);
    assert_same_lots(&source, &restored, &lots);

    // restore reads the source and stages in the same directory; it must not rewrite the source and
    // it must not leave the staging file behind.
    assert_eq!(sha256(&legacy), before, "the source must not be rewritten");
    assert_eq!(
        fixture.names(),
        vec!["current.snapshot.json", "legacy.snapshot.json"],
        "no staging file may survive a restore"
    );
}

#[test]
fn explicit_version_zero_migrates_to_the_same_bytes_as_the_versionless_form() {
    let fixture = Fixture::new();
    let (source, _) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let versionless = legacy_form(&read_json(&current_path));

    let mut explicit = versionless.clone();
    explicit
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_owned(), json!(LEGACY_SCHEMA_VERSION));

    let (versionless_bytes, versionless_report) =
        migrate(&serde_json::to_vec(&versionless).unwrap(), &table()).unwrap();
    let (explicit_bytes, explicit_report) =
        migrate(&serde_json::to_vec(&explicit).unwrap(), &table()).unwrap();
    assert_eq!(versionless_bytes, explicit_bytes);
    assert_eq!(versionless_report, explicit_report);
}

#[test]
fn legacy_build_areas_come_from_the_recovered_property_rows() {
    let fixture = Fixture::new();
    let (source, _) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let legacy = legacy_form(&read_json(&current_path));

    let (migrated, _) = migrate(&serde_json::to_vec(&legacy).unwrap(), &table()).unwrap();
    let document: Value = serde_json::from_slice(&migrated).unwrap();
    assert_eq!(document["schema_version"], json!(CURRENT_SCHEMA_VERSION));
    let lots = document["properties"].as_array().unwrap();
    assert_eq!(lots.len(), 2);
    assert_eq!(
        lots[0]["build_areas"],
        json!([[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]])
    );
    assert_eq!(lots[0]["zone_id"], json!(1));
    assert_eq!(lots[0]["name_id"], json!(6514));
    assert_eq!(
        lots[1]["build_areas"],
        json!([[218.8, -100.0, 346.1, 481.3, 80.0, 505.7]])
    );
    assert_eq!(lots[1]["zone_id"], json!(2));
    assert_eq!(lots[1]["name_id"], json!(5_103_265));
}

#[test]
fn legacy_lot_without_a_recovered_property_row_is_refused() {
    let fixture = Fixture::new();
    let (source, _) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let legacy = fixture.write_json(
        "legacy.snapshot.json",
        &legacy_form(&read_json(&current_path)),
    );
    let before = sha256(&legacy);

    // A live lot that already exists must not be lost or replaced by a refused migration.
    let mut live = Housing::new();
    let existing = grant_lot(&mut live, OWNER, property_three());
    let existing_snapshot = live.saved_fixtures(existing).unwrap();

    assert_eq!(
        restore(&mut live, &legacy, &[property_three()]),
        Err(MigrationError::UnknownProperty { property_id: 1 })
    );
    assert_eq!(sha256(&legacy), before);
    assert_eq!(
        live.saved_fixtures(existing).unwrap(),
        existing_snapshot,
        "the live lot must survive a refused migration unchanged"
    );
}

#[test]
fn legacy_property_row_that_disagrees_with_the_lot_is_refused() {
    let fixture = Fixture::new();
    let (source, _) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let legacy = fixture.write_json(
        "legacy.snapshot.json",
        &legacy_form(&read_json(&current_path)),
    );

    let mut wrong_zone = property_one();
    wrong_zone.zone_id = 9;
    let mut restored = Housing::new();
    assert_eq!(
        restore(
            &mut restored,
            &legacy,
            &[wrong_zone, property_two(), property_three()]
        ),
        Err(MigrationError::PropertyMismatch {
            property_id: 1,
            field: "zone_id",
            stored: 1,
            source: 9,
        })
    );

    let mut wrong_name = property_two();
    wrong_name.name_id = 1;
    assert_eq!(
        restore(
            &mut restored,
            &legacy,
            &[property_one(), wrong_name, property_three()]
        ),
        Err(MigrationError::PropertyMismatch {
            property_id: 2,
            field: "name_id",
            stored: 5_103_265,
            source: 1,
        })
    );
}

#[test]
fn legacy_declared_count_is_repaired_from_the_fixture_list() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let mut legacy = legacy_form(&read_json(&current_path));
    // A stale derived count in a legacy row: the fixture list is authoritative.
    legacy["properties"][0]
        .as_object_mut()
        .unwrap()
        .insert("count".to_owned(), json!(5));
    let path = fixture.write_json("legacy.snapshot.json", &legacy);

    let mut restored = Housing::new();
    let report = restore(&mut restored, &path, &table()).unwrap();
    assert_eq!(report.derived_counts, 1);
    assert_eq!(report.repaired_counts, 1);
    assert_eq!(report.filled_build_areas, 2);
    assert_same_lots(&source, &restored, &lots);
}

#[test]
fn future_schema_version_is_refused_and_the_source_is_never_rewritten() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let mut future = read_json(&current_path);
    future
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_owned(), json!(7));
    let path = fixture.write_json("future.snapshot.json", &future);
    let before = sha256(&path);

    let mut live = Housing::new();
    assert_eq!(
        restore(&mut live, &path, &table()),
        Err(MigrationError::UnsupportedVersion { found: 7 })
    );
    assert_eq!(
        sha256(&path),
        before,
        "a newer document must survive intact"
    );
    for lot in lots {
        assert_eq!(live.saved_fixtures(lot), Err(HousingError::LotNotFound));
    }

    // An integer above `u32::MAX` is still a newer version, not a malformed integer.
    let mut enormous = read_json(&current_path);
    enormous
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_owned(), json!(u64::MAX));
    assert_eq!(
        migrate(&serde_json::to_vec(&enormous).unwrap(), &table()),
        Err(MigrationError::UnsupportedVersion { found: u64::MAX })
    );
}

#[test]
fn malformed_and_non_document_inputs_are_refused() {
    assert!(matches!(
        migrate(b"{ not json", &table()),
        Err(MigrationError::Malformed { .. })
    ));
    assert_eq!(migrate(b"[]", &table()), Err(MigrationError::NotDocument));
    assert_eq!(
        migrate(br#"{"schema_version": "1", "properties": []}"#, &table()),
        Err(MigrationError::VersionNotInteger)
    );
    assert_eq!(
        migrate(br#"{"schema_version": -1, "properties": []}"#, &table()),
        Err(MigrationError::VersionNotInteger)
    );
    assert_eq!(
        migrate(br#"{"properties": []}"#, &table())
            .unwrap()
            .1
            .migrated,
        true
    );
}

#[test]
fn current_documents_missing_the_added_fields_are_invalid_not_repaired() {
    let fixture = Fixture::new();
    let (source, _) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let current = read_json(&current_path);

    let mut without_areas = current.clone();
    without_areas["properties"][0]
        .as_object_mut()
        .unwrap()
        .remove("build_areas");
    assert!(matches!(
        migrate(&serde_json::to_vec(&without_areas).unwrap(), &table()),
        Err(MigrationError::InvalidDocument { .. })
    ));

    let mut without_count = current.clone();
    without_count["properties"][0]
        .as_object_mut()
        .unwrap()
        .remove("count");
    assert!(matches!(
        migrate(&serde_json::to_vec(&without_count).unwrap(), &table()),
        Err(MigrationError::InvalidDocument { .. })
    ));
}

#[test]
fn a_document_the_real_loader_refuses_leaves_the_live_state_intact() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let mut legacy = legacy_form(&read_json(&current_path));
    // The first lot's chair claims the other lot's house guid: the migration can fill fields, but
    // the item 006 loader owns identity validation and must refuse the document.
    legacy["properties"][0]["fixtures"][0]["house_guid"] = json!(lots[1]);
    let path = fixture.write_json("legacy.snapshot.json", &legacy);

    let mut live = Housing::new();
    let error = restore(&mut live, &path, &table()).unwrap_err();
    assert_eq!(
        error,
        MigrationError::Refused(HousingError::CorruptSnapshot),
        "unexpected refusal: {error:?}"
    );
    for lot in lots {
        assert_eq!(
            live.saved_fixtures(lot),
            Err(HousingError::LotNotFound),
            "a refused restore must not adopt the other lot"
        );
    }
}

#[test]
fn duplicate_property_rows_are_refused_without_partial_adoption() {
    let fixture = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = fixture.path("current.snapshot.json");
    save_current(&source, &current_path);
    let mut legacy = legacy_form(&read_json(&current_path));
    // Both rows claim property 1: one property id may be owned by one lot only.
    legacy["properties"][1]["property_id"] = json!(1);
    legacy["properties"][1]["zone_id"] = json!(1);
    legacy["properties"][1]["name_id"] = json!(6514);
    let path = fixture.write_json("legacy.snapshot.json", &legacy);

    let mut live = Housing::new();
    let first = grant_lot(&mut live, OWNER, property_one());
    let before = live.saved_fixtures(first).unwrap();
    let error = restore(&mut live, &path, &table()).unwrap_err();
    // `Housing::load_state` deliberately collapses every snapshot validation failure into
    // `CorruptSnapshot`; the point of this vector is that nothing was partially adopted.
    assert_eq!(
        error,
        MigrationError::Refused(HousingError::CorruptSnapshot)
    );
    assert_eq!(
        live.saved_fixtures(first).unwrap(),
        before,
        "the live lot must survive a refused migration unchanged"
    );
    assert_eq!(live.saved_fixtures(lots[1]), Err(HousingError::LotNotFound));
}

#[test]
fn oversized_documents_are_refused_before_parsing() {
    let bytes = vec![b' '; piece_028::MAX_SNAPSHOT_BYTES + 1];
    assert_eq!(migrate(&bytes, &table()), Err(MigrationError::TooLarge));
}

#[test]
fn evidence_export_writes_migration_artifacts_for_the_independent_checker() {
    let Some(directory) = std::env::var_os("OSFR_PIECE_028_EVIDENCE_DIR").map(PathBuf::from) else {
        // The private export is optional; the vectors above always run.
        return;
    };
    fs::create_dir_all(&directory).unwrap();
    let work = Fixture::new();
    let (source, lots) = seed_housing();
    let current_path = work.path("current.snapshot.json");
    save_current(&source, &current_path);
    let legacy_value = legacy_form(&read_json(&current_path));
    let legacy_path = work.write_json("legacy.snapshot.json", &legacy_value);
    let before = sha256(&legacy_path);

    let mut restored = Housing::new();
    let report = restore(&mut restored, &legacy_path, &table()).unwrap();
    let after = sha256(&legacy_path);
    let (migrated_bytes, _) = migrate(&fs::read(&legacy_path).unwrap(), &table()).unwrap();

    let mut future = read_json(&current_path);
    future
        .as_object_mut()
        .unwrap()
        .insert("schema_version".to_owned(), json!(7));
    let future_path = work.write_json("future.snapshot.json", &future);
    let future_before = sha256(&future_path);
    let mut untouched = Housing::new();
    let future_error = format!(
        "{:?}",
        restore(&mut untouched, &future_path, &table()).unwrap_err()
    );
    let future_after = sha256(&future_path);

    // Owned furniture in the item 026 schema: every placed fixture backed by a pool row, plus two
    // chairs that stay available. Rows are sorted by (definition id, instance id).
    let stock = json!({
        "schema_version": 1,
        "instances": [
            {"instance_id": 1, "definition_id": CHAIR, "tint_id": 7},
            {"instance_id": 2, "definition_id": CHAIR, "tint_id": 2},
            {"instance_id": 3, "definition_id": CHAIR, "tint_id": 0},
            {"instance_id": 4, "definition_id": CHAIR, "tint_id": 0},
            {"instance_id": 1, "definition_id": TABLE, "tint_id": 0}
        ]
    });

    fs::write(
        directory.join("legacy.snapshot.json"),
        &serde_json::to_vec_pretty(&legacy_value).unwrap(),
    )
    .unwrap();
    fs::write(directory.join("migrated.snapshot.json"), &migrated_bytes).unwrap();
    fs::write(
        directory.join("future.snapshot.json"),
        &serde_json::to_vec_pretty(&future).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("furniture-stock.json"),
        &serde_json::to_vec_pretty(&stock).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("migration-report.json"),
        &serde_json::to_vec_pretty(&json!({
            "report": report,
            "legacy_sha256_before": before,
            "legacy_sha256_after": after,
            "future_case": {
                "error": future_error,
                "sha256_before": future_before,
                "sha256_after": future_after,
            },
            "lots": lots,
        }))
        .unwrap(),
    )
    .unwrap();
}
