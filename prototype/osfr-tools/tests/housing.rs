//! Housing / property / furniture placement acceptance for `src/housing.rs`.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — the client owns the housing packet family as
//!   inner opcodes of base opcode 127: PlaceFixtureRequest = 1, PlaceFixture = 2, PickupFixture = 3,
//!   SaveFixture = 5, SetEditMode = 6, InstanceData = 28, FixtureUpdate = 40, FixtureRemove = 41,
//!   InstanceList = 39. Confirmed at client constructors `FUN_00aa5340` (`FUN_00aa4990(1)`),
//!   `FUN_00aa53f0` (`FUN_00aa4990(2)`), `FUN_00aa81a0` (`FUN_00aa4990(3)`).
//! * `lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs` — the same numbers as opcodes, plus
//!   `BasePackets.BaseHousingPacket = 127`.
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/*.cs` — the community
//!   reconstruction of `PlayerHousingInstanceData` (house guid, owner guid, owner name, name id/name,
//!   max fixture count, max landmark count, fixtures, permissions, build areas, furniture score,
//!   members-only, factory instance) and of `FixtureInstance` (fixture guid, house guid, fixture
//!   definition id, position, rotation, tint id, customization, scale, tint, npc guid, xml).
//! * `lab/references/FreeRealms-Legacy/src/Resources/Houses.json` (sha256
//!   `854b2d3f009b916a928f66d57a42e2bf685dcc853e898d149580eefd26196d3c`, 31 house rows) — property
//!   definitions carry a zone id, name id, icon, spawn position/rotation and `BuildAreas` min/max.
//! * `lab/references/Sanctuary/src/Sanctuary.Game/ZoneManager.cs` — `PlayerOwnsHouse(ownerId,
//!   zoneDefinitionId)` gates housing access on the house zone definition.
//! * `lab/references/freerealms-wiki/wiki.sqlite` — "Lots" (rev 49783) describes lots as owned
//!   housing areas the player builds on.
//!
use osfr_tools::housing::{
    self, FixtureInstance, FixtureItemLedger, FixturePlacement, FixtureTransform, Housing,
    HousingError, PlacedFixture, PropertyDefinition, PropertyGrant, SaveFixtureCommand,
    SavedFixture, HOUSE_GUID_BASE, MAX_FIXTURE_MAGNITUDE,
};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const OWNER: u64 = 0x1122_3344_5566_7788;
const OTHER: u64 = 0x8877_6655_4433_2211;
const HOUSE: u64 = HOUSE_GUID_BASE | 1;
const HOUSE_TWO: u64 = HOUSE_GUID_BASE | 2;
const CHAIR: i32 = 17_310;
const TABLE: i32 = 17_311;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-housing-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    fn save_path(&self) -> PathBuf {
        self.directory.join("housing.snapshot.json")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn transform(x: f32, y: f32, z: f32) -> FixtureTransform {
    FixtureTransform {
        position: [x, y, z],
        rotation: [1.0, 0.0, 0.0, 0.0],
        tilt: [1.0, 0.0, 0.0, 0.0],
        scale: 1.0,
        tint_id: 0,
    }
}

fn ledger() -> FixtureItemLedger {
    let mut ledger = FixtureItemLedger::default();
    // Fungible stock: the captured instance ids are assigned per placement, so the ledger only has
    // to hold enough seeded instances for the vectors below, in the default tint.
    assert!(ledger.seed(CHAIR, 0, [1, 2, 3, 4]));
    assert!(ledger.seed(TABLE, 0, [1, 2]));
    ledger
}

/// ORIGINAL: the property id, zone id, name id and build area come from `Houses.json` row 1 (`Id` 1,
/// `ZoneId` 1, `NameId` 6514, `BuildAreas[0]`). RECONSTRUCTED: the fixture and landmark caps are
/// local test values; only the `HousingMaxFixtures` string id was recovered, not a number.
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

/// The same row with a different property id, so a lot guid stays distinct per property.
fn property_with(property_id: i32) -> PropertyDefinition {
    PropertyDefinition {
        property_id,
        ..property()
    }
}

fn owned(housing: &mut Housing, owner_guid: u64, property_id: i32, lot_guid: u64) {
    housing
        .grant_property(PropertyGrant {
            owner_guid,
            property: property_with(property_id),
            lot_guid,
        })
        .expect("the property definition is valid");
}

fn owned_with_cap(
    housing: &mut Housing,
    owner_guid: u64,
    property_id: i32,
    lot_guid: u64,
    max_fixtures: u32,
) {
    housing
        .grant_property(PropertyGrant {
            owner_guid,
            property: PropertyDefinition {
                max_fixtures,
                ..property_with(property_id)
            },
            lot_guid,
        })
        .expect("the property definition is valid");
}

#[test]
fn lot_guid_cannot_collide_with_a_property_zone_guid() {
    // ORIGINAL: the housing lot guid and the housing zone guid are distinct identities
    // (`PlayerHousingInstanceData.HouseGuid` versus the zone's owner-keyed instance). A property
    // restored from a malformed snapshot must not share a guid with a live gameplay zone.
    let housing = Housing::new();
    assert_ne!(housing.lot_guid(OWNER, 1).unwrap(), HOUSE);
    assert_ne!(housing.lot_guid(OWNER, 1).unwrap(), HOUSE_TWO);
    assert_eq!(
        housing.lot_guid(OWNER, 1).unwrap(),
        housing.lot_guid(OWNER, 1).unwrap()
    );
    assert_ne!(
        housing.lot_guid(OWNER, 1).unwrap(),
        housing.lot_guid(OTHER, 1).unwrap()
    );
}

#[test]
fn placement_grants_ownership_and_reports_the_granted_limits() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    let outcome = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("owner places an owned chair inside the build area");
    assert_eq!(outcome.placed.item_instance_id, 1);
    assert_eq!(outcome.placed.fixture_definition_id, CHAIR);
    assert_eq!(outcome.placed.house_guid, HOUSE);
    assert_eq!(outcome.placed.transform.position, [400.0, 4.0, 410.0]);
    assert_eq!(outcome.saved.count, 1);
    assert_eq!(outcome.saved.max_fixtures, 400);
    assert_eq!(
        inventory.available(CHAIR, 0),
        3,
        "one seeded chair is now placed"
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 1);
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().owner_guid, OWNER);
}

#[test]
fn a_fixture_instance_cannot_be_placed_twice_in_one_lot() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap();
    let duplicate = housing.place_fixture(
        OWNER,
        HOUSE,
        CHAIR,
        1,
        transform(401.0, 4.0, 410.0),
        &mut inventory,
    );
    assert_eq!(
        duplicate,
        Err(HousingError::InstanceAlreadyPlaced { instance_id: 1 })
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 1);

    // The same captured instance may live in a different owned lot.
    owned(&mut housing, OWNER, 2, HOUSE_TWO);
    housing
        .place_fixture(
            OWNER,
            HOUSE_TWO,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("the same instance may be placed in the other owned lot");
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 1);
    assert_eq!(housing.saved_fixtures(HOUSE_TWO).unwrap().fixtures.len(), 1);
}

#[test]
fn placement_rejects_unowned_and_unteamed_fixtures() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    assert_eq!(
        housing.place_fixture(
            OTHER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            99,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::InstanceNotOwned { instance_id: 99 })
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().count, 0);
}

#[test]
fn placement_rejects_non_finite_and_out_of_range_transforms() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    for (name, value, expected) in [
        (
            "nan position",
            FixtureTransform {
                position: [f32::NAN, 4.0, 410.0],
                ..transform(0.0, 0.0, 0.0)
            },
            HousingError::NonFiniteTransform,
        ),
        (
            "infinite position",
            FixtureTransform {
                position: [f32::INFINITY, 4.0, 410.0],
                ..transform(0.0, 0.0, 0.0)
            },
            HousingError::NonFiniteTransform,
        ),
        (
            "position beyond bound",
            transform(MAX_FIXTURE_MAGNITUDE * 2.0, 4.0, 410.0),
            HousingError::TransformOutOfBounds,
        ),
        (
            "zero scale",
            FixtureTransform {
                scale: 0.0,
                ..transform(400.0, 4.0, 410.0)
            },
            HousingError::TransformOutOfBounds,
        ),
        (
            "non-unit facing",
            FixtureTransform {
                tilt: [0.25, 0.0, 0.0, 0.0],
                ..transform(400.0, 4.0, 410.0)
            },
            HousingError::NonUnitRotation,
        ),
        (
            "rotation angle out of bounds",
            FixtureTransform {
                rotation: [2.0e6, 0.0, 0.0, 0.0],
                ..transform(400.0, 4.0, 410.0)
            },
            HousingError::TransformOutOfBounds,
        ),
    ] {
        assert_eq!(
            housing.place_fixture(OWNER, HOUSE, CHAIR, 1, value, &mut inventory),
            Err(expected),
            "{name}"
        );
    }
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().count, 0);
    assert_eq!(inventory.available(CHAIR, 0), 4);
}

#[test]
fn move_and_pickup_keep_ownership_and_release_the_snapshot() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let placed = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;

    let moved = housing
        .move_fixture(
            OWNER,
            HOUSE,
            placed.fixture_guid,
            FixtureTransform {
                rotation: [0.0, 1.0, 0.0, 0.0],
                ..transform(402.5, 4.0, 411.0)
            },
            &mut inventory,
        )
        .expect("the owner may move an owned fixture");
    assert_eq!(moved.position, [402.5, 4.0, 411.0]);
    assert_eq!(moved.rotation, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(
        inventory.available(CHAIR, 0),
        3,
        "a move must not consume the instance twice"
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 1);

    assert_eq!(
        housing.move_fixture(
            OTHER,
            HOUSE,
            placed.fixture_guid,
            transform(403.0, 4.0, 411.0),
            &mut inventory
        ),
        Err(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        housing.move_fixture(
            OWNER,
            HOUSE,
            placed.fixture_guid | 0xdead_beef_0000_0000,
            transform(403.0, 4.0, 411.0),
            &mut inventory
        ),
        Err(HousingError::UnknownFixture)
    );

    housing
        .pickup_fixture(OWNER, HOUSE, placed.fixture_guid, &mut inventory)
        .expect("the owner may pick the fixture back up");
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 0);
    assert_eq!(
        inventory.available(CHAIR, 0),
        4,
        "pick-up returns the instance to stock"
    );
}

#[test]
fn placement_limits_are_enforced_and_never_silently_exceeded() {
    let mut housing = Housing::new();
    let mut inventory = FixtureItemLedger::default();
    assert!(inventory.seed(CHAIR, 0, [1, 2, 3, 4, 5, 6, 7, 8]));
    owned_with_cap(&mut housing, OWNER, 1, HOUSE, 2);

    for instance in 1..=2 {
        housing
            .place_fixture(
                OWNER,
                HOUSE,
                CHAIR,
                instance,
                transform(400.0, 4.0, 410.0),
                &mut inventory,
            )
            .unwrap();
    }
    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            3,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::FixtureLimitReached)
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().fixtures.len(), 2);
    assert_eq!(inventory.available(CHAIR, 0), 6);
}

#[test]
fn placement_outside_every_build_area_is_denied() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    // ORIGINAL: `Houses.json` gives property 1 the build area x 322.0..=443.3, y -100.0..=60.0,
    // z 392.1..=477.8. `HousingMaxFixtures` is the client's only recovered denial string.
    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(500.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::OutsideBuildArea)
    );
    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 70.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::OutsideBuildArea)
    );
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(322.0, -100.0, 392.1),
            &mut inventory,
        )
        .expect("the build-area corner is inclusive");
}

#[test]
fn place_fixture_request_is_rejected_on_a_locked_lot() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    housing.set_locked(OWNER, HOUSE, true).unwrap();

    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::LotLocked)
    );
    // ORIGINAL: `ClientHousingPacketLockHouse` is inner opcode 22; the owner may unlock.
    housing.set_locked(OWNER, HOUSE, false).unwrap();
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("unlocking restores placement");
}

#[test]
fn corrupted_or_duplicate_snapshots_are_rejected_on_load() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let placed = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;

    // A second fixture guid for the same captured instance. This is what a double-spend of one
    // furniture stack looks like in the restored ledger, and it must never be written to a save.
    let mut double_spent = housing.saved_fixtures(HOUSE).unwrap();
    let mut twin = SavedFixture::from(placed.to_placement());
    twin.fixture_guid ^= 0x2000;
    double_spent.fixtures.push(twin);
    double_spent.count = double_spent.fixtures.len() as u32;
    housing
        .replace_saved_fixtures(HOUSE, double_spent.clone())
        .expect("replace keeps the double-spent value in memory");
    assert_eq!(
        housing.save_state(&fixture.save_path()),
        Err(HousingError::InstanceAlreadyPlaced { instance_id: 1 })
    );

    // A snapshot document that repeats a fixture guid must not restore either copy.
    let document = serde_json::json!({
        "schema_version": housing::SNAPSHOT_SCHEMA_VERSION,
        "properties": [
            {
                "owner_guid": OWNER,
                "property_id": 1,
                "lot_guid": HOUSE,
                "zone_id": 1,
                "name_id": 6514,
                "max_fixtures": 400,
                "max_landmarks": 0,
                "count": 2,
                "build_areas": [[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]],
                "fixtures": [
                    SavedFixture::from(placed.to_placement()),
                    SavedFixture::from(placed.to_placement())
                ],
                "permissions": [],
                "locked": false,
            }
        ],
    });
    fs::write(fixture.save_path(), serde_json::to_vec(&document).unwrap()).unwrap();
    assert!(matches!(
        housing.load_state(&fixture.save_path()),
        Err(HousingError::CorruptSnapshot)
    ));
    // The rejected document leaves the live in-memory state untouched.
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().count, 2);
}

#[test]
fn saved_fixtures_survive_a_serialized_restart() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let chair = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            TABLE,
            1,
            transform(401.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap();
    housing
        .move_fixture(
            OWNER,
            HOUSE,
            chair.fixture_guid,
            FixtureTransform {
                scale: 1.5,
                ..transform(402.0, 4.5, 411.0)
            },
            &mut inventory,
        )
        .unwrap();
    housing.set_locked(OWNER, HOUSE, true).unwrap();

    let before = housing.saved_fixtures(HOUSE).unwrap();
    housing
        .save_state(&fixture.save_path())
        .expect("snapshot saves");
    let bytes = fs::read(fixture.save_path()).unwrap();
    assert!(
        bytes.len() < housing::MAX_SNAPSHOT_BYTES,
        "the snapshot stays inside its documented bound"
    );

    let mut restored = Housing::new();
    restored
        .load_state(&fixture.save_path())
        .expect("snapshot restores");
    let after = restored.saved_fixtures(HOUSE).unwrap();
    assert_eq!(before, after);
    assert!(after.locked);
    assert_eq!(after.fixtures.len(), 2);
    let moved = after
        .fixtures
        .iter()
        .find(|fixture| fixture.fixture_guid == chair.fixture_guid)
        .expect("the moved fixture restores by guid");
    assert_eq!(moved.transform.position, [402.0, 4.5, 411.0]);
    assert_eq!(moved.transform.scale, 1.5);
    assert_eq!(moved.item_instance_id, 1);

    // A restart must not mint a second guid for the same captured instance.
    let mut inventory = ledger();
    restored
        .set_locked(OWNER, HOUSE, false)
        .expect("the restored owner may reopen the lot");
    assert_eq!(
        restored.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::InstanceAlreadyPlaced { instance_id: 1 })
    );
}

#[test]
fn a_restored_lot_keeps_its_ownership_and_accepts_an_edit_after_restart() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let chair = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    housing.save_state(&fixture.save_path()).unwrap();

    let mut restored = Housing::new();
    restored.load_state(&fixture.save_path()).unwrap();
    restored
        .move_fixture(
            OTHER,
            HOUSE,
            chair.fixture_guid,
            transform(200.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect_err("a stranger must not edit a restored lot");
    let moved = restored
        .move_fixture(
            OWNER,
            HOUSE,
            chair.fixture_guid,
            FixtureTransform {
                rotation: [0.0, 1.0, 0.0, 0.0],
                ..transform(402.0, 4.0, 411.0)
            },
            &mut inventory,
        )
        .expect("the restored owner keeps edit authority");
    assert_eq!(moved.position, [402.0, 4.0, 411.0]);

    // Saving the moved fixture again replaces it in place; the count never grows.
    let stored = restored.saved_fixtures(HOUSE).unwrap();
    assert_eq!(stored.count, 1);
    restored
        .save_state(&fixture.save_path())
        .expect("the moved snapshot saves");
    let mut reloaded = Housing::new();
    reloaded.load_state(&fixture.save_path()).unwrap();
    let stored = reloaded.saved_fixtures(HOUSE).unwrap();
    assert_eq!(stored.count, 1);
    assert_eq!(stored.fixtures[0].transform.position, [402.0, 4.0, 411.0]);
    assert_eq!(stored.fixtures[0].item_instance_id, 1);
}

#[test]
fn a_rejected_save_leaves_the_previous_snapshot_file_byte_identical() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let placed = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    housing.save_state(&fixture.save_path()).unwrap();
    let committed = fs::read(fixture.save_path()).unwrap();

    let mut corrupt_housing = Housing::new();
    let mut corrupt_inventory = ledger();
    owned(&mut corrupt_housing, OWNER, 1, HOUSE);
    corrupt_housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut corrupt_inventory,
        )
        .unwrap();
    let mut double_spent = corrupt_housing.saved_fixtures(HOUSE).unwrap();
    let mut twin = SavedFixture::from(placed.to_placement());
    twin.fixture_guid ^= 0x4000;
    double_spent.fixtures.push(twin);
    double_spent.count = double_spent.fixtures.len() as u32;
    corrupt_housing
        .replace_saved_fixtures(HOUSE, double_spent)
        .unwrap();

    assert!(matches!(
        corrupt_housing.save_state(&fixture.save_path()),
        Err(HousingError::InstanceAlreadyPlaced { instance_id: 1 })
    ));
    assert_eq!(
        fs::read(fixture.save_path()).unwrap(),
        committed,
        "a rejected save must not replace the committed snapshot"
    );
    let mut restored = Housing::new();
    restored.load_state(&fixture.save_path()).unwrap();
    assert_eq!(restored.saved_fixtures(HOUSE).unwrap(), {
        let mut expected = housing.saved_fixtures(HOUSE).unwrap();
        expected
            .fixtures
            .sort_by_key(|fixture| fixture.fixture_guid);
        expected
    });
}

#[test]
fn fixture_guids_are_unique_per_lot_placement() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let first = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    let second = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            3,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    assert_ne!(first.fixture_guid, second.fixture_guid);

    // A rejected request must not consume a guid, so the next accepted placement is deterministic.
    let mut probe = Housing::new();
    let mut probe_inventory = ledger();
    owned(&mut probe, OWNER, 1, HOUSE);
    assert_eq!(
        probe.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(900.0, 4.0, 410.0),
            &mut probe_inventory
        ),
        Err(HousingError::OutsideBuildArea)
    );
    let retry = probe
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut probe_inventory,
        )
        .unwrap()
        .placed;
    assert_eq!(retry.fixture_guid, first.fixture_guid);
}

#[test]
fn staged_inventory_stays_consistent_with_the_fixture_ledger() {
    let mut housing = Housing::new();
    let mut inventory = FixtureItemLedger::default();
    assert!(inventory.seed(CHAIR, 0, [1, 2]));
    owned(&mut housing, OWNER, 1, HOUSE);

    let proposal = housing
        .save_fixture_placement(SaveFixtureCommand {
            owner_guid: OWNER,
            house_guid: HOUSE,
            placed: SavedFixture {
                fixture_guid: 0,
                house_guid: HOUSE,
                item_instance_id: 1,
                fixture_definition_id: CHAIR,
                transform: transform(400.0, 4.0, 410.0),
                placed_by: 0,
            },
        })
        .expect("the server copies the client transform before committing");
    assert_eq!(proposal.owner_guid, OWNER);
    assert_eq!(proposal.house_guid, HOUSE);
    assert_eq!(proposal.instance_id, 1);
    assert!(proposal.created, "a first save mints one fixture guid");
    let stored = housing.saved_fixtures(HOUSE).unwrap();
    assert_eq!(stored.count, 1);
    let fixture_guid = stored.fixtures[0].fixture_guid;

    // Replaying the same client save is idempotent; it must not stage a second output instance.
    let replay = housing
        .save_fixture_placement(SaveFixtureCommand {
            owner_guid: OWNER,
            house_guid: HOUSE,
            placed: housing.saved_fixtures(HOUSE).unwrap().fixtures[0].clone(),
        })
        .unwrap();
    assert!(!replay.created);
    let stored = housing.saved_fixtures(HOUSE).unwrap();
    assert_eq!(stored.count, 1);
    assert_eq!(stored.fixtures[0].fixture_guid, fixture_guid);

    // A rejected request must leave both ledgers exactly as they were.
    let placements_before = housing.saved_fixtures(HOUSE).unwrap();
    let available_before = inventory.available(CHAIR, 0);
    assert_eq!(
        housing.save_fixture_placement(SaveFixtureCommand {
            owner_guid: OTHER,
            house_guid: HOUSE,
            placed: placements_before.fixtures[0].clone(),
        }),
        Err(HousingError::NotPropertyOwner)
    );
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap(), placements_before);
    assert_eq!(inventory.available(CHAIR, 0), available_before);
}

#[test]
fn typed_fixture_view_matches_the_recovered_serialization_order() {
    // ORIGINAL: `FixtureInstance.Serialize` writes guid, house guid, fixture definition id,
    // position, rotation, tilt, npc guid, tint id, customization, scale. We keep that field set so
    // the lead's packet writer can serialize a placement without inventing a second schema.
    let placement = FixturePlacement {
        fixture_guid: 9,
        house_guid: HOUSE,
        item_instance_id: 4,
        fixture_definition_id: TABLE,
        transform: FixtureTransform {
            position: [1.0, 2.0, 3.0],
            rotation: [0.5, 0.5, 0.5, 0.5],
            tilt: [1.0, 0.0, 0.0, 0.0],
            scale: 2.0,
            tint_id: 7,
        },
    };
    let stored = SavedFixture::from(placement.clone());
    let round_trip: FixturePlacement = stored.to_placement().expect("stored fixture is valid");
    assert_eq!(round_trip, placement);

    let json = serde_json::to_value(&stored).unwrap();
    let mut keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    // L045 adds the server-only `placed_by` (not part of the wire record).
    assert_eq!(keys.len(), 6);
    for expected in [
        "fixture_guid",
        "house_guid",
        "item_instance_id",
        "fixture_definition_id",
        "transform",
        "placed_by",
    ] {
        assert!(
            keys.contains(&expected),
            "missing serialized field {expected}"
        );
    }
    let mut transform_keys: Vec<&str> = json["transform"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    transform_keys.sort_unstable();
    assert_eq!(transform_keys.len(), 5);
    for expected in ["position", "rotation", "scale", "tilt", "tint_id"] {
        assert!(
            transform_keys.contains(&expected),
            "missing serialized transform field {expected}"
        );
    }
    assert_eq!(SavedFixture::from(placement).house_guid(), HOUSE);
    assert!(std::mem::size_of::<PlacedFixture>() > 0);
}

#[test]
fn a_rejected_snapshot_load_leaves_the_live_state_intact() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let before = housing.saved_fixtures(HOUSE).unwrap();

    // `load_state` promises that a rejected document leaves the in-memory state untouched. This
    // document deserializes and rebuilds, then fails snapshot validation because the lot has no
    // owner; it also carries limits the live lot must not adopt.
    let document = serde_json::json!({
        "schema_version": housing::SNAPSHOT_SCHEMA_VERSION,
        "properties": [
            {
                "owner_guid": 0,
                "property_id": 1,
                "lot_guid": HOUSE,
                "zone_id": 1,
                "name_id": 6514,
                "max_fixtures": 0,
                "max_landmarks": 0,
                "count": 0,
                "build_areas": [[1.0, -100.0, 392.1, 2.0, 60.0, 477.8]],
                "fixtures": [],
                "permissions": [],
                "locked": false,
            }
        ],
    });
    fs::write(fixture.save_path(), serde_json::to_vec(&document).unwrap()).unwrap();
    assert_eq!(
        housing.load_state(&fixture.save_path()),
        Err(HousingError::CorruptSnapshot)
    );

    assert_eq!(housing.saved_fixtures(HOUSE).unwrap(), before);
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("the live lot keeps its own limits and build area after a rejected load");
}

#[test]
fn duplicate_lot_guids_in_a_snapshot_are_rejected() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let placed = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;

    // Two lots sharing one guid: a rebuild that simply inserts would drop the first lot and its
    // fixtures without an error, which is silent furniture loss in a save.
    let document = serde_json::json!({
        "schema_version": housing::SNAPSHOT_SCHEMA_VERSION,
        "properties": [
            {
                "owner_guid": OWNER,
                "property_id": 1,
                "lot_guid": HOUSE,
                "zone_id": 1,
                "name_id": 6514,
                "max_fixtures": 400,
                "max_landmarks": 0,
                "count": 1,
                "build_areas": [[322.0, -100.0, 392.1, 443.3, 60.0, 477.8]],
                "fixtures": [SavedFixture::from(placed.to_placement())],
                "permissions": [],
                "locked": false,
            },
            {
                "owner_guid": OWNER,
                "property_id": 2,
                "lot_guid": HOUSE,
                "zone_id": 2,
                "name_id": 5103265,
                "max_fixtures": 400,
                "max_landmarks": 0,
                "count": 0,
                "build_areas": [[218.8, -100.0, 346.1, 481.3, 80.0, 505.7]],
                "fixtures": [],
                "permissions": [],
                "locked": false,
            }
        ],
    });
    fs::write(fixture.save_path(), serde_json::to_vec(&document).unwrap()).unwrap();

    let mut restored = Housing::new();
    assert_eq!(
        restored.load_state(&fixture.save_path()),
        Err(HousingError::CorruptSnapshot)
    );
    assert_eq!(
        restored.saved_fixtures(HOUSE),
        Err(HousingError::LotNotFound),
        "a rejected document must not restore either of the duplicate lots"
    );
}

#[test]
fn a_lot_limit_change_is_enforced_by_placement_and_still_saves() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    // The migration hook replaces the lot's own cap; placement, save and restore must all agree on
    // the replaced value instead of a stale property-table copy.
    let mut snapshot = housing.saved_fixtures(HOUSE).unwrap();
    snapshot.max_fixtures = 1;
    housing.replace_saved_fixtures(HOUSE, snapshot).unwrap();

    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("the first placement stays inside the replaced cap");
    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            2,
            transform(401.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::FixtureLimitReached)
    );
    housing
        .save_state(&fixture.save_path())
        .expect("an accepted placement must always be saveable");
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().max_fixtures, 1);
}

#[test]
fn a_lot_build_area_change_is_enforced_by_placement() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    let mut snapshot = housing.saved_fixtures(HOUSE).unwrap();
    snapshot.build_areas = vec![[200.0, -100.0, 392.1, 260.0, 60.0, 477.8]];
    housing.replace_saved_fixtures(HOUSE, snapshot).unwrap();

    assert_eq!(
        housing.place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::OutsideBuildArea),
        "the replacement area is the one the lot now reports"
    );
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(250.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("a position inside the replacement area is accepted");
}

#[test]
fn save_fixture_placement_stamps_the_resolved_lot_identity() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    owned(&mut housing, OWNER, 1, HOUSE);

    // The server resolves the lot from the command's house guid; a client-supplied house guid on the
    // placement must never be able to persist a fixture that the snapshot validator rejects.
    let proposal = housing
        .save_fixture_placement(SaveFixtureCommand {
            owner_guid: OWNER,
            house_guid: HOUSE,
            placed: SavedFixture {
                fixture_guid: 0,
                house_guid: HOUSE_TWO,
                item_instance_id: 1,
                fixture_definition_id: CHAIR,
                transform: transform(400.0, 4.0, 410.0),
                placed_by: OTHER,
            },
        })
        .expect("the owner may save a placement into the lot the command names");
    assert!(proposal.created);

    let stored = housing.saved_fixtures(HOUSE).unwrap();
    assert_eq!(stored.fixtures.len(), 1);
    assert_eq!(stored.fixtures[0].house_guid, HOUSE);
    // L045: the placer is the command's authorized actor, never the caller-supplied guid.
    assert_eq!(stored.fixtures[0].placed_by, OWNER);
    housing
        .save_state(&fixture.save_path())
        .expect("a stamped placement saves and reloads");
}

#[test]
fn save_fixture_placement_rejects_an_empty_instance() {
    let mut housing = Housing::new();
    owned(&mut housing, OWNER, 1, HOUSE);

    // `place_fixture` rejects an empty instance or definition; the save path must not admit what the
    // placement path calls invalid, or a save would later fail snapshot validation.
    for (item_instance_id, fixture_definition_id) in [(0_u32, CHAIR), (1_u32, 0)] {
        assert_eq!(
            housing.save_fixture_placement(SaveFixtureCommand {
                owner_guid: OWNER,
                house_guid: HOUSE,
                placed: SavedFixture {
                    fixture_guid: 0,
                    house_guid: HOUSE,
                    item_instance_id,
                    fixture_definition_id,
                    transform: transform(400.0, 4.0, 410.0),
                    placed_by: 0,
                },
            }),
            Err(HousingError::InvalidPlacement),
            "instance {item_instance_id} / definition {fixture_definition_id}"
        );
    }
    assert_eq!(housing.saved_fixtures(HOUSE).unwrap().count, 0);
}

#[test]
fn saving_an_existing_fixture_keeps_the_build_area_bound() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let placed = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    let before = housing.saved_fixtures(HOUSE).unwrap();

    // A save that rewrites an existing fixture is a move, so it must respect the same bound as
    // `move_fixture`; otherwise a client save can persist an out-of-area fixture.
    assert_eq!(
        housing.save_fixture_placement(SaveFixtureCommand {
            owner_guid: OWNER,
            house_guid: HOUSE,
            placed: SavedFixture {
                transform: transform(900.0, 4.0, 410.0),
                ..SavedFixture::from(placed.to_placement())
            },
        }),
        Err(HousingError::OutsideBuildArea)
    );
    assert_eq!(
        housing.saved_fixtures(HOUSE).unwrap(),
        before,
        "a denied save must leave the stored transform untouched"
    );
}

#[test]
fn a_rejected_seed_leaves_the_ledger_unchanged() {
    let mut inventory = FixtureItemLedger::default();
    assert!(
        !inventory.seed(CHAIR, 0, [1, 1]),
        "a duplicated captured instance id is rejected"
    );
    assert!(
        inventory.seed(CHAIR, 0, [1]),
        "the rejected call must not have consumed the id"
    );
    assert_eq!(inventory.available(CHAIR, 0), 1);
}

#[test]
fn a_guest_with_edit_permission_can_edit_but_a_stranger_cannot() {
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);

    // ORIGINAL: `InstancePermission` level 3 is "can edit". Only the owner may hand that level out
    // or lock the lot.
    assert_eq!(
        housing.set_permission(OTHER, HOUSE, OTHER, housing::PERMISSION_CAN_EDIT),
        Err(HousingError::NotPropertyOwner)
    );
    assert_eq!(
        housing.set_locked(OTHER, HOUSE, true),
        Err(HousingError::NotPropertyOwner)
    );
    housing
        .set_permission(OWNER, HOUSE, OTHER, housing::PERMISSION_CAN_EDIT)
        .expect("the owner may grant an edit level");

    let placed = housing
        .place_fixture(
            OTHER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("a permitted guest may place furniture")
        .placed;
    housing
        .move_fixture(
            OWNER,
            HOUSE,
            placed.fixture_guid,
            transform(401.0, 4.0, 410.0),
            &mut inventory,
        )
        .expect("the owner keeps edit authority on their own lot");
    housing
        .set_permission(OWNER, HOUSE, OTHER, 0)
        .expect("a zero level removes the permission");
    assert_eq!(
        housing.move_fixture(
            OTHER,
            HOUSE,
            placed.fixture_guid,
            transform(402.0, 4.0, 410.0),
            &mut inventory
        ),
        Err(HousingError::NotPropertyOwner)
    );
}

#[test]
fn restart_conservation_requires_restaging_the_captured_instances() {
    let fixture = Fixture::new();
    let mut housing = Housing::new();
    let mut inventory = ledger();
    owned(&mut housing, OWNER, 1, HOUSE);
    let chair = housing
        .place_fixture(
            OWNER,
            HOUSE,
            CHAIR,
            1,
            transform(400.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap()
        .placed;
    housing
        .place_fixture(
            OWNER,
            HOUSE,
            TABLE,
            1,
            transform(401.0, 4.0, 410.0),
            &mut inventory,
        )
        .unwrap();
    housing.save_state(&fixture.save_path()).unwrap();

    let mut restored = Housing::new();
    restored.load_state(&fixture.save_path()).unwrap();

    // The lot snapshot is restored; the placement ledger is the owning inventory's state, so a
    // restart reseeds stock and then re-stages the fixtures the lot still holds.
    let mut restock = FixtureItemLedger::default();
    assert!(restock.seed(CHAIR, 0, [1, 2, 3, 4]));
    assert!(restock.seed(TABLE, 0, [1, 2]));
    assert_eq!(
        restored.pickup_fixture(OWNER, HOUSE, chair.fixture_guid, &mut restock),
        Err(HousingError::InventoryNotOwned { instance_id: 1 }),
        "a pickup with no recorded placed instance must not mint stock"
    );
    assert_eq!(restored.saved_fixtures(HOUSE).unwrap().count, 2);
    for stored in restored.saved_fixtures(HOUSE).unwrap().fixtures {
        restock
            .stage(FixtureInstance {
                instance_id: stored.item_instance_id,
                definition_id: stored.fixture_definition_id,
                tint_id: stored.transform.tint_id,
            })
            .expect("a re-seeded captured instance restages");
    }
    assert_eq!(restock.available(CHAIR, 0), 3);
    assert_eq!(restock.available(TABLE, 0), 1);

    restored
        .move_fixture(
            OWNER,
            HOUSE,
            chair.fixture_guid,
            transform(402.0, 4.5, 411.0),
            &mut restock,
        )
        .expect("the restored owner may move an owned fixture");
    restored
        .pickup_fixture(OWNER, HOUSE, chair.fixture_guid, &mut restock)
        .expect("the restored owner may pick the fixture back up");
    assert_eq!(restored.saved_fixtures(HOUSE).unwrap().count, 1);
    assert_eq!(
        restock.available(CHAIR, 0),
        4,
        "pick-up after a restart returns the instance to stock"
    );
}
