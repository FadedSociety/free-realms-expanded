//! Item 026: housing inventory transactions.
//!
//! These vectors run against `src/restoration/piece_026.rs`, which includes `src/housing.rs` by
//! `#[path]` so the pair compiles before the lead registers either module in `src/lib.rs`.
//!
//! RED evidence: the first revision of this file used the item 006 module directly and failed with
//! `assertion left == right failed: a fixture whose snapshot could not be written must not stay
//! placed`, `left: 1`, `right: 0`. The retained log is
//! `lab/reports/piece-026-20261009T025209Z/red-housing-only.log`.
//!
//! ORIGINAL evidence used by these vectors (all outside Git):
//! * `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` — housing is packet family 127 with
//!   `ClientHousingPacketPlaceFixtureRequest` 1, `ClientHousingPacketPlaceFixture` 2,
//!   `ClientHousingPacketPickupFixture` 3, `ClientHousingPacketSaveFixture` 5,
//!   `HousingPacketFixtureRemove` 41 and `HousingPacketFixtureItemList` 43.
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — the supplied client constructs the fixture
//!   item list (`HousingPacketFixtureItemList::vftable`) and reads `HousingMaxFixtures` from the
//!   property definition.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/*.cs` — a placed fixture
//!   carries the fixture guid, house guid, fixture definition id, position, rotation, tilt, tint id
//!   and scale; a housing instance carries `MaxFixtureCount` and `CurFixtureCount`; the fixture item
//!   list pairs an owned item definition with a fixture definition.
//! * `lab/references/FreeRealms-Legacy/src/Resources/Houses.json` row 1 — property 1, zone 1, name
//!   6514, build area x 322.0..=443.3, y -100.0..=60.0, z 392.1..=477.8.
//!
//! Everything the pool document, the commit ordering and the numeric bounds do is RECONSTRUCTED.

#[path = "../src/restoration/piece_026.rs"]
#[allow(dead_code)]
mod piece_026;

use piece_026::{
    Conservation, FixtureTransform, HousingError, HousingInventoryTransaction, LotSnapshot,
    OwnedFixture, PropertyDefinition, TransactionError,
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const OWNER: u64 = 0x1122_3344_5566_7788;
const OTHER: u64 = 0x8877_6655_4433_2211;
const HOUSE_TWO: u64 = 0x0f00_0000_0000_0002;
const CHAIR: i32 = 17_310;
const TABLE: i32 = 17_311;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-piece-026-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn housing_path(&self) -> PathBuf {
        self.directory.join("housing.snapshot.json")
    }

    fn stock_path(&self) -> PathBuf {
        self.directory.join("furniture-stock.json")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// A committed pair plus the lot guid item 006 derived for the owner and property.
struct Lot {
    transaction: HousingInventoryTransaction,
    house: u64,
}

fn transform(x: f32, y: f32, z: f32) -> FixtureTransform {
    FixtureTransform {
        position: [x, y, z],
        ..FixtureTransform::default()
    }
}

/// ORIGINAL: `Houses.json` row 1, the property item 006 pins in its own suite.
fn property() -> PropertyDefinition {
    PropertyDefinition {
        property_id: 1,
        zone_id: 1,
        name_id: 6514,
        max_fixtures: 400,
        max_landmarks: 0,
        build_areas: vec![[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]],
    }
}

fn unit(instance_id: u32, definition_id: i32, tint_id: i32) -> OwnedFixture {
    OwnedFixture {
        instance_id,
        definition_id,
        tint_id,
    }
}

fn open(fixture: &Fixture) -> HousingInventoryTransaction {
    HousingInventoryTransaction::open(fixture.housing_path(), fixture.stock_path())
        .expect("a missing pair is an empty state")
}

/// Four owned chairs and two owned tables, all in the default tint.
fn owned_lot(fixture: &Fixture) -> Lot {
    let mut transaction = open(fixture);
    let house = transaction
        .grant_property(OWNER, property())
        .expect("the property definition is valid");
    for instance_id in 1..=4 {
        transaction
            .grant_instance(unit(instance_id, CHAIR, 0))
            .expect("the pool accepts one new unit");
    }
    for instance_id in 1..=2 {
        transaction
            .grant_instance(unit(instance_id, TABLE, 0))
            .expect("the pool accepts one new unit");
    }
    Lot { transaction, house }
}

fn balance(conservation: &Conservation, definition_id: i32) -> (u32, u32, u32) {
    let row = conservation
        .definitions
        .iter()
        .find(|row| row.definition_id == definition_id)
        .expect("the definition has a balance row");
    (row.owned, row.placed, row.available)
}

fn read(path: &Path) -> Option<Vec<u8>> {
    fs::read(path).ok()
}

#[test]
fn failed_persistence_rolls_the_placement_back() {
    let fixture = Fixture::new();
    let untouched = owned_lot(&fixture);
    let stock_bytes = read(untouched.transaction.stock_path()).expect("the pool file exists");

    // A housing path whose parent does not exist opens as an empty half (NotFound) and can never be
    // written. The pool path stays writable, so the check really is about the failing commit.
    let blocked = fixture
        .directory
        .join("missing")
        .join("housing.snapshot.json");
    let pool_path = fixture.directory.join("blocked-pool.json");
    let mut transaction = HousingInventoryTransaction::open(&blocked, &pool_path)
        .expect("a missing parent is an empty half");
    let house = transaction
        .grant_property(OWNER, property())
        .expect("the property definition is valid");
    transaction
        .grant_instance(unit(1, CHAIR, 0))
        .expect("the pool file is still writable");

    let error = transaction
        .place_fixture(OWNER, house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .expect_err("the snapshot cannot be written");
    assert!(
        matches!(error, TransactionError::PersistHousing(_)),
        "a failed commit is reported as a persistence failure, got {error:?}"
    );

    // Nothing stayed in memory: the fixture is not placed and the unit is still available.
    let conservation = transaction
        .conservation()
        .expect("the rolled back pair conserves");
    assert_eq!(balance(&conservation, CHAIR), (1, 0, 1));
    assert_eq!(transaction.ledger().unwrap().available(CHAIR, 0), 1);
    assert!(
        transaction
            .housing()
            .saved_fixtures(house)
            .unwrap()
            .fixtures
            .is_empty()
    );
    assert!(!blocked.exists(), "no housing snapshot was created");
    assert_eq!(
        transaction.verify().unwrap(),
        transaction.conservation().unwrap()
    );

    // The pool file kept the one granted unit and nothing else.
    let pool: serde_json::Value =
        serde_json::from_slice(&fs::read(transaction.stock_path()).unwrap()).unwrap();
    assert_eq!(pool["instances"].as_array().unwrap().len(), 1);
    assert_eq!(
        read(untouched.transaction.stock_path()).as_deref(),
        Some(&stock_bytes[..])
    );
}

#[test]
fn placement_and_pickup_conserve_the_owned_pool() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let stock_bytes = read(lot.transaction.stock_path()).expect("the pool file exists");

    let placed = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .expect("the owner places an owned chair inside the build area")
        .placed;
    assert_eq!(placed.item_instance_id, 1);
    assert_eq!(placed.house_guid, lot.house);

    let conservation = lot.transaction.conservation().unwrap();
    assert_eq!(balance(&conservation, CHAIR), (4, 1, 3));
    assert_eq!(balance(&conservation, TABLE), (2, 0, 2));
    assert_eq!((conservation.owned, conservation.placed), (6, 1));
    assert_eq!(conservation.available, 5);
    assert_eq!(lot.transaction.ledger().unwrap().available(CHAIR, 0), 3);

    // ORIGINAL: `PlayerHousingInstanceData.CurFixtureCount` agrees with the stored fixture map.
    let stored: LotSnapshot = lot.transaction.housing().saved_fixtures(lot.house).unwrap();
    assert_eq!(stored.count, 1);
    assert_eq!(stored.fixtures.len(), 1);
    assert_eq!(stored.owner_guid, OWNER);
    assert_eq!(
        read(lot.transaction.stock_path()).as_deref(),
        Some(&stock_bytes[..]),
        "a placement consumes pool capacity but never rewrites the owned pool"
    );

    let removed = lot
        .transaction
        .pickup_fixture(OWNER, lot.house, placed.fixture_guid)
        .expect("the owner picks the fixture back up");
    assert_eq!(removed.removed.item_instance_id, 1);
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (4, 0, 4)
    );
    assert_eq!(
        read(lot.transaction.stock_path()).as_deref(),
        Some(&stock_bytes[..]),
        "an unrepainted pick-up does not rewrite the pool either"
    );

    // The pair survives a restart with the same numbers and the same owned pool.
    lot.transaction
        .place_fixture(OWNER, lot.house, TABLE, 1, transform(401.0, 4.0, 410.0))
        .expect("a table placement commits");
    let reopened = HousingInventoryTransaction::open(
        lot.transaction.housing_path(),
        lot.transaction.stock_path(),
    )
    .expect("the committed pair reloads");
    let after = reopened.housing().saved_fixtures(lot.house).unwrap();
    assert_eq!(after.count, 1);
    assert_eq!(after.fixtures[0].fixture_definition_id, TABLE);
    assert_eq!(balance(&reopened.conservation().unwrap(), TABLE), (2, 1, 1));
    assert_eq!(balance(&reopened.conservation().unwrap(), CHAIR), (4, 0, 4));
    assert_eq!(reopened.ledger().unwrap().available(CHAIR, 0), 4);
    assert_eq!(
        read(reopened.stock_path()).as_deref(),
        Some(&stock_bytes[..]),
        "the owned pool file is unchanged by placement and pick-up"
    );
}

#[test]
fn rejected_requests_leave_the_durable_pair_byte_identical() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    lot.transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .expect("the first placement commits the lot");
    let housing_bytes = read(lot.transaction.housing_path()).expect("the housing file exists");
    let stock_bytes = read(lot.transaction.stock_path()).expect("the pool file exists");

    // A stranger, an out-of-area transform and an unowned label are all refusals.
    let cases: Vec<TransactionError> = vec![
        lot.transaction
            .place_fixture(OTHER, lot.house, CHAIR, 2, transform(400.0, 4.0, 410.0))
            .unwrap_err(),
        lot.transaction
            .place_fixture(OWNER, lot.house, CHAIR, 2, transform(500.0, 4.0, 410.0))
            .unwrap_err(),
        lot.transaction
            .place_fixture(OWNER, lot.house, CHAIR, 99, transform(400.0, 4.0, 410.0))
            .unwrap_err(),
    ];
    assert_eq!(
        cases[0],
        TransactionError::Rejected(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        cases[1],
        TransactionError::Rejected(HousingError::OutsideBuildArea)
    );
    assert_eq!(
        cases[2],
        TransactionError::Rejected(HousingError::InstanceNotOwned { instance_id: 99 })
    );

    assert_eq!(
        read(lot.transaction.housing_path()).as_deref(),
        Some(&housing_bytes[..])
    );
    assert_eq!(
        read(lot.transaction.stock_path()).as_deref(),
        Some(&stock_bytes[..])
    );
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (4, 1, 3)
    );
}

#[test]
fn a_placed_unit_cannot_leave_the_owned_pool() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let stock_bytes = read(lot.transaction.stock_path()).expect("the pool file exists");
    let placed = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .unwrap()
        .placed;

    assert_eq!(
        lot.transaction.consume_instance(CHAIR, 1),
        Err(TransactionError::InstanceInUse { instance_id: 1 })
    );
    assert_eq!(
        read(lot.transaction.stock_path()).as_deref(),
        Some(&stock_bytes[..])
    );
    assert_eq!(lot.transaction.stock().owned(CHAIR), 4);

    // An unplaced unit may leave, and the pool file follows the in-memory pool.
    let removed = lot
        .transaction
        .consume_instance(CHAIR, 4)
        .expect("an unplaced owned unit can be consumed");
    assert_eq!(removed, unit(4, CHAIR, 0));
    assert_eq!(lot.transaction.stock().owned(CHAIR), 3);
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (3, 1, 2)
    );

    let mut reopened = HousingInventoryTransaction::open(
        lot.transaction.housing_path(),
        lot.transaction.stock_path(),
    )
    .expect("the pair reloads after a removal");
    assert_eq!(reopened.stock().owned(CHAIR), 3);
    assert_eq!(balance(&reopened.conservation().unwrap(), CHAIR), (3, 1, 2));
    let picked_up = reopened
        .pickup_fixture(OWNER, lot.house, placed.fixture_guid)
        .expect("the placed fixture is still pickable");
    assert_eq!(picked_up.removed.item_instance_id, 1);
    assert_eq!(reopened.stock().owned(CHAIR), 3);
}

#[test]
fn placing_a_unit_the_pool_does_not_own_is_rejected() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let housing_bytes = read(lot.transaction.housing_path());
    assert_eq!(
        lot.transaction
            .place_fixture(OWNER, lot.house, CHAIR, 9, transform(400.0, 4.0, 410.0)),
        Err(TransactionError::Rejected(HousingError::InstanceNotOwned {
            instance_id: 9
        }))
    );
    // `CHAIR` owns 1..=4 but `TABLE` owns only 1..=2, so the same numeric label under another
    // definition is still not owned.
    assert_eq!(
        lot.transaction
            .place_fixture(OWNER, lot.house, TABLE, 3, transform(400.0, 4.0, 410.0)),
        Err(TransactionError::Rejected(HousingError::InstanceNotOwned {
            instance_id: 3
        }))
    );
    assert_eq!(read(lot.transaction.housing_path()), housing_bytes);
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (4, 0, 4)
    );
}

#[test]
fn a_repainted_move_commits_and_the_pickup_keeps_the_new_tint() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let placed = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .unwrap()
        .placed;

    let moved = lot
        .transaction
        .move_fixture(
            OWNER,
            lot.house,
            placed.fixture_guid,
            FixtureTransform {
                tint_id: 7,
                ..transform(402.0, 4.0, 411.0)
            },
        )
        .expect("the owner may repaint an owned fixture");
    assert_eq!(moved.tint_id, 7);
    // While the unit is placed its tint follows the fixture, so the pool row is still the old tint.
    assert_eq!(lot.transaction.stock().find(CHAIR, 1).unwrap().tint_id, 0);
    let conservation = lot.transaction.conservation().unwrap();
    assert_eq!(balance(&conservation, CHAIR), (4, 1, 3));
    assert_eq!(lot.transaction.ledger().unwrap().available(CHAIR, 7), 0);

    // A second transaction sees the same placement through the committed housing snapshot.
    let reopened = HousingInventoryTransaction::open(
        lot.transaction.housing_path(),
        lot.transaction.stock_path(),
    )
    .expect("the repainted pair reloads");
    assert_eq!(reopened.stock().find(CHAIR, 1).unwrap().tint_id, 0);
    assert_eq!(
        reopened
            .housing()
            .saved_fixtures(lot.house)
            .unwrap()
            .fixtures[0]
            .transform
            .tint_id,
        7
    );

    lot.transaction
        .pickup_fixture(OWNER, lot.house, placed.fixture_guid)
        .expect("the owner picks the repainted fixture up");
    assert_eq!(
        lot.transaction.stock().find(CHAIR, 1).unwrap().tint_id,
        7,
        "the picked-up unit keeps the tint it was placed with"
    );
    assert_eq!(lot.transaction.ledger().unwrap().available(CHAIR, 7), 1);
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (4, 0, 4)
    );

    let reopened = HousingInventoryTransaction::open(
        lot.transaction.housing_path(),
        lot.transaction.stock_path(),
    )
    .expect("the pair reloads after the pick-up");
    assert_eq!(reopened.stock().find(CHAIR, 1).unwrap().tint_id, 7);
    assert_eq!(reopened.ledger().unwrap().available(CHAIR, 7), 1);
}

#[test]
fn double_spent_pairs_are_rejected_on_load() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    lot.transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .expect("the placement commits");

    // Case A: the pool lost the placed unit. The pair must not load at all.
    let pool: serde_json::Value =
        serde_json::from_slice(&fs::read(lot.transaction.stock_path()).unwrap()).unwrap();
    let mut emptied = pool.clone();
    emptied["instances"] = serde_json::json!([]);
    fs::write(
        lot.transaction.stock_path(),
        serde_json::to_vec_pretty(&emptied).unwrap(),
    )
    .unwrap();
    assert_eq!(
        HousingInventoryTransaction::open(
            lot.transaction.housing_path(),
            lot.transaction.stock_path()
        )
        .unwrap_err(),
        TransactionError::PlacedInstanceNotOwned { instance_id: 1 }
    );

    // Case B: two lots place the same single owned unit. Loading must refuse the pair.
    let mut housing: serde_json::Value =
        serde_json::from_slice(&fs::read(lot.transaction.housing_path()).unwrap()).unwrap();
    let mut second = housing["properties"][0].clone();
    second["lot_guid"] = serde_json::json!(HOUSE_TWO);
    second["property_id"] = serde_json::json!(2);
    second["fixtures"][0]["house_guid"] = serde_json::json!(HOUSE_TWO);
    second["fixtures"][0]["fixture_guid"] = serde_json::json!(0x0f00_0000_0000_0002u64);
    housing["properties"].as_array_mut().unwrap().push(second);
    fs::write(
        lot.transaction.housing_path(),
        serde_json::to_vec_pretty(&housing).unwrap(),
    )
    .unwrap();
    let mut single_unit = pool.clone();
    single_unit["instances"] = serde_json::json!([{
        "instance_id": 1,
        "definition_id": CHAIR,
        "tint_id": 0,
    }]);
    fs::write(
        lot.transaction.stock_path(),
        serde_json::to_vec_pretty(&single_unit).unwrap(),
    )
    .unwrap();
    assert_eq!(
        HousingInventoryTransaction::open(
            lot.transaction.housing_path(),
            lot.transaction.stock_path()
        )
        .unwrap_err(),
        TransactionError::PlacedExceedsOwned {
            definition_id: CHAIR,
            owned: 1,
            placed: 2,
        }
    );
}

#[test]
fn a_tint_skew_pair_still_loads_and_heals_on_pickup() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let placed = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .unwrap()
        .placed;

    // A pool row whose tint differs from the fixture it is placed as is the residue a failed
    // compensating write can leave. It conserves, because a placed unit's tint follows its fixture.
    let mut pool: serde_json::Value =
        serde_json::from_slice(&fs::read(lot.transaction.stock_path()).unwrap()).unwrap();
    pool["instances"][0]["tint_id"] = serde_json::json!(9);
    fs::write(
        lot.transaction.stock_path(),
        serde_json::to_vec_pretty(&pool).unwrap(),
    )
    .unwrap();

    let mut healed = HousingInventoryTransaction::open(
        lot.transaction.housing_path(),
        lot.transaction.stock_path(),
    )
    .expect("the tint skew pair still conserves");
    assert_eq!(balance(&healed.conservation().unwrap(), CHAIR), (4, 1, 3));
    // The placed unit is the tint-9 row, but a placed unit follows its fixture, so it still counts
    // inside the default tint bucket and the tint-9 bucket holds nothing.
    assert_eq!(healed.ledger().unwrap().available(CHAIR, 0), 3);
    assert_eq!(healed.ledger().unwrap().available(CHAIR, 9), 0);
    healed
        .pickup_fixture(OWNER, lot.house, placed.fixture_guid)
        .expect("the pick-up heals the row");
    assert_eq!(healed.stock().find(CHAIR, 1).unwrap().tint_id, 0);
    assert_eq!(healed.ledger().unwrap().available(CHAIR, 0), 4);
    let reopened = HousingInventoryTransaction::open(healed.housing_path(), healed.stock_path())
        .expect("the healed pair reloads");
    assert_eq!(reopened.stock().find(CHAIR, 1).unwrap().tint_id, 0);
    assert_eq!(reopened.ledger().unwrap().available(CHAIR, 0), 4);
}

#[test]
fn one_unit_placed_at_two_tints_is_refused_instead_of_folded() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let second_lot = lot
        .transaction
        .grant_property(
            OWNER,
            PropertyDefinition {
                property_id: 2,
                ..property()
            },
        )
        .expect("a second property is granted");
    let first = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .unwrap()
        .placed;
    let second = lot
        .transaction
        .place_fixture(OWNER, second_lot, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .expect("the counting model allows the same label in two lots");
    assert_eq!(
        balance(&lot.transaction.conservation().unwrap(), CHAIR),
        (4, 2, 2)
    );
    let housing_bytes = read(lot.transaction.housing_path()).expect("the housing file exists");

    // Repainting one of the two placements would need one unit to carry two tints at once.
    assert_eq!(
        lot.transaction.move_fixture(
            OWNER,
            second_lot,
            second.placed.fixture_guid,
            FixtureTransform {
                tint_id: 7,
                ..transform(402.0, 4.0, 411.0)
            },
        ),
        Err(TransactionError::AmbiguousPlacedTint { instance_id: 1 })
    );
    // The refusal rolled the move back: the transform and the file are unchanged.
    let stored = lot
        .transaction
        .housing()
        .saved_fixtures(second_lot)
        .unwrap();
    assert_eq!(stored.fixtures[0].transform.tint_id, 0);
    assert_eq!(stored.fixtures[0].transform.position, [400.0, 4.0, 410.0]);
    assert_eq!(
        read(lot.transaction.housing_path()).as_deref(),
        Some(&housing_bytes[..])
    );
    assert_ne!(first.fixture_guid, second.placed.fixture_guid);
}

#[test]
fn pool_writes_are_atomic_when_the_destination_cannot_be_written() {
    let fixture = Fixture::new();
    let blocked = fixture
        .directory
        .join("missing")
        .join("furniture-stock.json");
    let mut transaction = HousingInventoryTransaction::open(fixture.housing_path(), &blocked)
        .expect("a missing parent is an empty half");
    transaction.grant_property(OWNER, property()).unwrap();

    let error = transaction
        .grant_instance(unit(1, CHAIR, 0))
        .expect_err("the pool file cannot be written");
    assert_eq!(
        error,
        TransactionError::PersistStock(std::io::ErrorKind::NotFound)
    );
    assert!(
        transaction.stock().instances().is_empty(),
        "memory rolled back"
    );
    assert!(!blocked.exists());

    // The same holds for a removal: after the pool file exists, its directory is replaced by a file
    // so the destination can no longer be created, and the in-memory pool rolls back.
    let pool_directory = fixture.directory.join("pool");
    fs::create_dir(&pool_directory).unwrap();
    let stock_path = pool_directory.join("furniture-stock.json");
    let mut removal = HousingInventoryTransaction::open(fixture.housing_path(), &stock_path)
        .expect("the pool starts empty");
    removal.grant_instance(unit(1, TABLE, 0)).unwrap();
    let bytes = read(&stock_path).expect("the pool file exists");
    fs::remove_file(&stock_path).unwrap();
    fs::remove_dir(&pool_directory).unwrap();
    fs::write(&pool_directory, b"blocked").unwrap();
    assert!(
        matches!(
            removal.consume_instance(TABLE, 1),
            Err(TransactionError::PersistStock(_))
        ),
        "the blocked destination must report a pool write failure"
    );
    assert_eq!(removal.stock().owned(TABLE), 1, "memory rolled back");
    assert_eq!(
        fs::read(&pool_directory).unwrap(),
        b"blocked",
        "the failed write left the destination untouched"
    );
    assert_eq!(
        fs::read_dir(&fixture.directory)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count(),
        0,
        "the failed write left no temporary file behind"
    );
    fs::remove_file(&pool_directory).unwrap();
    fs::create_dir(&pool_directory).unwrap();
    fs::write(&stock_path, &bytes).unwrap();
    let reopened = HousingInventoryTransaction::open(fixture.housing_path(), &stock_path)
        .expect("the committed pool reloads");
    assert_eq!(reopened.stock().owned(TABLE), 1);
}

#[test]
fn unknown_or_duplicate_pool_rows_are_rejected() {
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    assert_eq!(
        lot.transaction.grant_instance(unit(1, CHAIR, 0)),
        Err(TransactionError::DuplicateStockInstance { instance_id: 1 })
    );
    assert_eq!(
        lot.transaction.grant_instance(unit(0, CHAIR, 0)),
        Err(TransactionError::InvalidStockInstance { instance_id: 0 })
    );
    assert_eq!(
        lot.transaction.consume_instance(CHAIR, 9),
        Err(TransactionError::UnknownStockInstance { instance_id: 9 })
    );

    // A pool document that repeats one unit, or that is not the expected schema, is corrupt.
    let mut pool: serde_json::Value =
        serde_json::from_slice(&fs::read(lot.transaction.stock_path()).unwrap()).unwrap();
    let repeated = pool["instances"][0].clone();
    pool["instances"].as_array_mut().unwrap().push(repeated);
    fs::write(
        lot.transaction.stock_path(),
        serde_json::to_vec_pretty(&pool).unwrap(),
    )
    .unwrap();
    assert_eq!(
        HousingInventoryTransaction::open(
            lot.transaction.housing_path(),
            lot.transaction.stock_path()
        )
        .unwrap_err(),
        TransactionError::DuplicateStockInstance { instance_id: 1 }
    );

    fs::write(
        lot.transaction.stock_path(),
        b"{\"schema_version\":99,\"instances\":[]}",
    )
    .unwrap();
    assert_eq!(
        HousingInventoryTransaction::open(
            lot.transaction.housing_path(),
            lot.transaction.stock_path()
        )
        .unwrap_err(),
        TransactionError::StockCorrupt("unsupported owned pool version")
    );
}

/// Writes the durable pair and the conservation report for the independent checker in
/// the private recovery checker `piece_026.py`. The evidence directory is private lab state, so the test skips the
/// export when the environment variable is absent.
#[test]
fn evidence_export_writes_the_durable_pair() {
    let Some(directory) = std::env::var_os("OSFR_PIECE_026_EVIDENCE_DIR").map(PathBuf::from) else {
        return;
    };
    let fixture = Fixture::new();
    let mut lot = owned_lot(&fixture);
    let chair = lot
        .transaction
        .place_fixture(OWNER, lot.house, CHAIR, 1, transform(400.0, 4.0, 410.0))
        .unwrap()
        .placed;
    lot.transaction
        .move_fixture(
            OWNER,
            lot.house,
            chair.fixture_guid,
            FixtureTransform {
                tint_id: 7,
                ..transform(402.0, 4.0, 411.0)
            },
        )
        .unwrap();
    lot.transaction
        .place_fixture(OWNER, lot.house, TABLE, 1, transform(401.0, 4.0, 410.0))
        .unwrap();
    let conservation = lot.transaction.conservation().unwrap();
    assert_eq!(balance(&conservation, CHAIR), (4, 1, 3));
    assert_eq!(balance(&conservation, TABLE), (2, 1, 1));

    fs::create_dir_all(&directory).unwrap();
    fs::copy(
        lot.transaction.housing_path(),
        directory.join("housing.snapshot.json"),
    )
    .unwrap();
    fs::copy(
        lot.transaction.stock_path(),
        directory.join("furniture-stock.json"),
    )
    .unwrap();
    let units = lot.transaction.placed_units().unwrap();
    let report = serde_json::json!({
        "consensus": "reconstructed server transaction over original packet and property facts",
        "owned": conservation.owned,
        "placed": conservation.placed,
        "available": conservation.available,
        "definitions": conservation.definitions.iter().map(|row| serde_json::json!({
            "definition_id": row.definition_id,
            "owned": row.owned,
            "placed": row.placed,
            "available": row.available,
        })).collect::<Vec<_>>(),
        "placed_units": units.iter().map(|unit| serde_json::json!({
            "lot_guid": unit.lot_guid,
            "fixture_guid": unit.fixture_guid,
            "definition_id": unit.definition_id,
            "instance_id": unit.instance_id,
            "tint_id": unit.tint_id,
        })).collect::<Vec<_>>(),
    });
    fs::write(
        directory.join("conservation.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
