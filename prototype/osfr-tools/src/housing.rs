//! Housing / property / furniture placement state for one restored server.
//!
//! Client packet names, opcodes, and verified field prefixes are documented in
//! `notes/Build Guides/Gameplay/Housing.md` and its native-map citations. FreeRealms-Legacy is a
//! behavior/data reference only: its packet models, resource rows and server checks do not establish
//! retail server behavior.
//!
//! Legacy reference shapes used to guide this local model:
//! * fixture and lot-instance fields are based on
//!   `FreeRealms-Legacy/src/Sanctuary.Packet.Common/Housing/FixtureInstance.cs` and
//!   `PlayerHousingInstanceData.cs`;
//! * property zone/name/icon/spawn/build-area rows are transcribed from
//!   `FreeRealms-Legacy/src/Resources/Houses.json` (31 rows);
//! * Legacy `ZoneManager.PlayerOwnsHouse` is consulted only as a behavior reference.
//!
//! Reconstructed policy (R): local numeric bounds, identity allocation, starter grants, permission
//! levels beyond the client-verified edit value, inventory conservation and error behavior. Original
//! placement, pickup, saving, collision and server handlers remain unverified, so this module is a
//! bounded local state machine rather than a native acceptance claim.
//!
//! Deliberate boundaries:
//! * no collision, snap, drag preview, landmark scoring, upkeep billing, visitor access or house
//!   rating is modelled;
//! * the fixture item ledger is conservation-only. The lead's inventory owns real stacks and
//!   currencies; this module never mints an item and never writes a shared save file;
//! * `HousingMaxFixtures` is the only recovered denial string, so limit semantics stay with the
//!   property definition rather than an invented global constant.
//!
//! ponytail: one property table, one fixture table and one ledger pass. No trait objects, no packet
//! writer, no per-frame tick. Add shape-layout or collision only when the native client demands it.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

/// RECONSTRUCTED local defensive bound, matching the existing `character_store` convention. This is
/// not a recovered world or angle limit.
pub const MAX_FIXTURE_MAGNITUDE: f32 = 1_000_000.0;
/// RECONSTRUCTED tolerance for the stored facing against the one the rotation implies.
pub const FACING_TOLERANCE: f32 = 1e-3;
/// RECONSTRUCTED snapshot bound, in the same spirit as `character_store::MAX_FILE_BYTES`.
pub const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;
/// RECONSTRUCTED schema version for the local housing snapshot.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED fixture-guid namespace. Real fixture guids are server-assigned and unknown; this
/// block keeps generated guids distinguishable from character, item and zone guids in a local test.
pub const HOUSE_GUID_BASE: u64 = 0x0f00_0000_0000_0000;
/// The namespace of minted fixture guids. Bit 63 stays clear: the client formats the selected
/// fixture's guid with `%lld` (`SelectedFixtureDataSource::GetValue`, `00cbd700`) and the Lua
/// guid reader `00bfea90` returns 0 for a string that starts with `-`, so a guid with bit 63 set
/// reached every radial-menu action (move, rotate, remove) as guid 0 (T826). Bit 55 stays clear
/// too: the fixture's actor guid sets it (`housing_service::fixture_actor_guid`).
const FIXTURE_GUID_MASK: u64 = 0x7f00_0000_0000_0000;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Legacy reference: its packet model assigns level 3 to edit permission. Retail semantics remain
/// unverified; this mapping is reconstructed policy (R).
pub const PERMISSION_CAN_EDIT: i32 = 3;
/// RECONSTRUCTED visitor grant level; the client's original permission numbers are unknown.
pub const PERMISSION_CAN_VISIT: i32 = 1;

/// Local transform projection based on the Legacy `FixtureInstance` reference shape.
///
/// `rotation` is the client's own form, Euler angles in radians: yaw, pitch, roll, then 0. The
/// client sends those lanes in 127/2 and a retail 127/28 carries them (1.5708, 3.1416, ...), never
/// a quaternion. It is the one source of the facing: `tilt` is its projection, and a stored pair
/// that disagrees is refused (T806/F838).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureTransform {
    pub position: [f32; 3],
    /// The client's Euler lanes in radians; changing these changes the facing.
    pub rotation: [f32; 4],
    /// `FixtureTransform::facing(rotation)`. Older snapshots persisted it separately, so load
    /// normalizes it from the rotation and validation refuses a pair that disagrees.
    pub tilt: [f32; 4],
    pub scale: f32,
    pub tint_id: i32,
}

impl FixtureTransform {
    /// The facing a client-form rotation (yaw, pitch, roll in radians) implies:
    /// `(sin yaw cos pitch, -sin pitch, cos yaw cos pitch, 0)`. A 127/40 record carries it as
    /// `tilt`, and its AddNpc as `rotation` (community-preserved `p2.pcap`: all 2,015 instances;
    /// the client computes it itself in `sub_105CD30` through `sub_776970`).
    pub fn facing(rotation: [f32; 4]) -> [f32; 4] {
        let [yaw, pitch, ..] = rotation;
        [
            yaw.sin() * pitch.cos(),
            0.0 - pitch.sin(),
            yaw.cos() * pitch.cos(),
            0.0,
        ]
    }

    pub fn validate(&self) -> Result<(), HousingError> {
        validate_transform(self)
    }
}

impl Default for FixtureTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0; 4],
            tilt: [0.0, 0.0, 1.0, 0.0],
            scale: 1.0,
            tint_id: 0,
        }
    }
}

/// Local placed-fixture state. Server fixture IDs and the item-instance association are reconstructed
/// policy (R), guided by the client update fields and Legacy fixture reference shape.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedFixture {
    pub fixture_guid: u64,
    pub house_guid: u64,
    pub item_instance_id: u32,
    pub fixture_definition_id: i32,
    pub transform: FixtureTransform,
}

impl PlacedFixture {
    pub fn validate(&self) -> Result<(), HousingError> {
        self.to_placement().validate()
    }

    pub fn to_placement(&self) -> FixturePlacement {
        FixturePlacement {
            fixture_guid: self.fixture_guid,
            house_guid: self.house_guid,
            item_instance_id: self.item_instance_id,
            fixture_definition_id: self.fixture_definition_id,
            transform: self.transform.clone(),
        }
    }
}

/// Persisted local form. The Legacy `FixtureInstance` also includes a house guid; this shape retains
/// it as an integrity check (R), not as a claim about the retail save format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedFixture {
    pub fixture_guid: u64,
    pub house_guid: u64,
    pub item_instance_id: u32,
    pub fixture_definition_id: i32,
    pub transform: FixtureTransform,
    /// L045 (R): the character whose placement created this fixture. Set by the server from the
    /// acting guid (`place_fixture`, `save_fixture_placement`), never from a packet, and not on the
    /// wire (the client fixture reader 0x00ab6f40 has no such field). A move keeps it. `0` means
    /// "not recorded" (documents from housing service schema 1); restore attributes it to the
    /// lot owner, the only character who could place before co-builders (`attributed_to`).
    #[serde(default)]
    pub placed_by: u64,
}

impl SavedFixture {
    /// The schema-1 migration rule: an unrecorded placer is the lot owner.
    fn attributed_to(mut self, owner_guid: u64) -> Self {
        if self.placed_by == 0 {
            self.placed_by = owner_guid;
        }
        self
    }

    pub fn house_guid(&self) -> u64 {
        self.house_guid
    }

    pub fn to_placement(&self) -> Result<FixturePlacement, HousingError> {
        let placement = FixturePlacement {
            fixture_guid: self.fixture_guid,
            house_guid: self.house_guid,
            item_instance_id: self.item_instance_id,
            fixture_definition_id: self.fixture_definition_id,
            transform: self.transform.clone(),
        };
        placement.validate()?;
        Ok(placement)
    }
}

impl From<FixturePlacement> for SavedFixture {
    fn from(placement: FixturePlacement) -> Self {
        Self {
            fixture_guid: placement.fixture_guid,
            house_guid: placement.house_guid,
            item_instance_id: placement.item_instance_id,
            fixture_definition_id: placement.fixture_definition_id,
            transform: placement.transform,
            placed_by: 0,
        }
    }
}

impl From<PlacedFixture> for SavedFixture {
    fn from(placed: PlacedFixture) -> Self {
        Self::from(placed.to_placement())
    }
}

impl From<&PlacedFixture> for SavedFixture {
    fn from(placed: &PlacedFixture) -> Self {
        Self::from(placed.to_placement())
    }
}

/// The typed placement the lead serializes with `FixtureInstance`.
#[derive(Clone, Debug, PartialEq)]
pub struct FixturePlacement {
    pub fixture_guid: u64,
    pub house_guid: u64,
    pub item_instance_id: u32,
    pub fixture_definition_id: i32,
    pub transform: FixtureTransform,
}

impl FixturePlacement {
    pub fn validate(&self) -> Result<(), HousingError> {
        if self.fixture_guid == 0 || self.house_guid == 0 || self.fixture_definition_id <= 0 {
            return Err(HousingError::InvalidPlacement);
        }
        validate_transform(&self.transform)
    }
}

/// Community-reference house row projected into the server property model. `build_areas` are
/// `[min_x, min_y, min_z, max_x, max_y, max_z]`; the shipped client has no lot/build-box table.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyDefinition {
    pub property_id: i32,
    pub zone_id: i32,
    pub name_id: i32,
    pub max_fixtures: u32,
    pub max_landmarks: u32,
    pub build_areas: Vec<[f32; 6]>,
}

impl PropertyDefinition {
    pub(crate) fn contains(&self, position: [f32; 3]) -> bool {
        self.build_areas.iter().any(|area| {
            position[0] >= area[0]
                && position[0] <= area[3]
                && position[1] >= area[1]
                && position[1] <= area[4]
                && position[2] >= area[2]
                && position[2] <= area[5]
        })
    }

    pub(crate) fn validate(&self) -> Result<(), HousingError> {
        if self.property_id <= 0
            || self.zone_id <= 0
            || self.name_id <= 0
            || self.build_areas.is_empty()
            || self.build_areas.len() > 64
        {
            return Err(HousingError::InvalidProperty);
        }
        for area in &self.build_areas {
            if area.iter().any(|value| !value.is_finite())
                || area[0] > area[3]
                || area[1] > area[4]
                || area[2] > area[5]
                || area.iter().any(|value| value.abs() > MAX_FIXTURE_MAGNITUDE)
            {
                return Err(HousingError::InvalidBuildArea);
            }
        }
        Ok(())
    }
}

/// Local lot snapshot projected from the Legacy `PlayerHousingInstanceData` reference shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LotSnapshot {
    pub owner_guid: u64,
    pub property_id: i32,
    pub lot_guid: u64,
    pub zone_id: i32,
    pub name_id: i32,
    pub max_fixtures: u32,
    pub max_landmarks: u32,
    /// Local count; a corresponding field exists in the Legacy reference model.
    pub count: u32,
    /// Community-reference build areas, flattened `[min_x, min_y, min_z, max_x, max_y, max_z]`.
    /// Persisted so a restart can still validate placement bounds. This is not client evidence.
    pub build_areas: Vec<[f32; 6]>,
    /// Sorted by fixture guid, exactly as the restored lot stores them.
    pub fixtures: Vec<SavedFixture>,
    #[serde(default)]
    pub permissions: Vec<LotPermission>,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LotPermission {
    pub guid: u64,
    pub level: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HousingError {
    InvalidOwner,
    InvalidProperty,
    InvalidBuildArea,
    PropertyAlreadyOwned,
    PropertyNotOwned,
    NotPropertyOwner,
    NotPermitted,
    LotLocked,
    LotNotFound,
    UnknownFixture,
    InstanceAlreadyPlaced { instance_id: u32 },
    InstanceNotOwned { instance_id: u32 },
    FixtureLimitReached,
    LandmarkLimitReached,
    OutsideBuildArea,
    InvalidPlacement,
    NonFiniteTransform,
    TransformOutOfBounds,
    NonUnitFacing,
    DuplicateFixture,
    DuplicateProperty { property_id: i32 },
    CorruptSnapshot,
    SnapshotTooLarge,
    InventoryNotOwned { instance_id: u32 },
    SnapshotIo(io::ErrorKind),
}

impl From<io::Error> for HousingError {
    fn from(error: io::Error) -> Self {
        Self::SnapshotIo(error.kind())
    }
}

/// Reconstructed conservation ledger. The live inventory owns actual stacks; this stand-in serves
/// the isolated state machine tests only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FixtureItem {
    pub definition_id: i32,
    pub tint_id: i32,
    pub owned: u32,
    pub placed: u32,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct FixtureItemLedger {
    items: BTreeMap<(i32, i32), FixtureItem>,
    /// Seeded inventory instances by definition. Placement rejects an unowned instance id, so the
    /// ledger tracks identity rather than quantity alone.
    spawnable: BTreeSet<(i32, u32)>,
}

impl FixtureItemLedger {
    /// Stage a stock grant. This is called by the owning inventory path, never by housing itself.
    pub fn grant(&mut self, definition_id: i32, tint_id: i32, count: u32) -> bool {
        if definition_id <= 0 || tint_id < 0 || count == 0 {
            return false;
        }
        let entry = self
            .items
            .entry((definition_id, tint_id))
            .or_insert(FixtureItem {
                definition_id,
                tint_id,
                owned: 0,
                placed: 0,
            });
        entry.owned = entry.owned.saturating_add(count);
        true
    }

    pub fn owned(&self, definition_id: i32, tint_id: i32) -> bool {
        self.items
            .get(&(definition_id, tint_id))
            .is_some_and(|item| item.owned > item.placed)
    }

    /// Seed one furniture stack, mirroring how the lead's inventory hands furniture
    /// instances to housing. Returns the instance ids a caller may place.
    pub fn seed(
        &mut self,
        definition_id: i32,
        tint_id: i32,
        ids: impl IntoIterator<Item = u32>,
    ) -> bool {
        if definition_id <= 0 || tint_id < 0 {
            return false;
        }
        let ids: Vec<u32> = ids.into_iter().filter(|id| *id > 0).collect();
        if ids.is_empty() {
            return false;
        }
        // Validate the whole request before mutating: a rejected seed must not consume any id.
        let mut unique = BTreeSet::new();
        if ids
            .iter()
            .any(|id| !unique.insert(*id) || self.spawnable.contains(&(definition_id, *id)))
        {
            return false;
        }
        let count = ids.len() as u32;
        for id in &ids {
            self.spawnable.insert((definition_id, *id));
        }
        let entry = self
            .items
            .entry((definition_id, tint_id))
            .or_insert(FixtureItem {
                definition_id,
                tint_id,
                owned: 0,
                placed: 0,
            });
        entry.owned = entry.owned.saturating_add(count);
        true
    }

    pub fn available(&self, definition_id: i32, tint_id: i32) -> u32 {
        self.items
            .get(&(definition_id, tint_id))
            .map_or(0, |item| item.owned.saturating_sub(item.placed))
    }

    /// A movement may carry a repaint. Keep the conservation ledger on the same tint the fixture now
    /// uses so a later pick-up can still return the instance.
    pub fn rekey_tint(
        &mut self,
        definition_id: i32,
        from: i32,
        to: i32,
    ) -> Result<(), HousingError> {
        if from == to {
            return Ok(());
        }
        let moved = self
            .items
            .get_mut(&(definition_id, from))
            .filter(|item| item.placed > 0)
            .map(|item| {
                item.placed -= 1;
                item.placed
            });
        if moved.is_none() {
            return Err(HousingError::InventoryNotOwned { instance_id: 0 });
        }
        let entry = self
            .items
            .entry((definition_id, to))
            .or_insert(FixtureItem {
                definition_id,
                tint_id: to,
                owned: 0,
                placed: 0,
            });
        entry.placed += 1;
        Ok(())
    }

    pub fn stage(&mut self, instance: FixtureInstance) -> Result<(), HousingError> {
        if !self
            .spawnable
            .contains(&(instance.definition_id, instance.instance_id))
        {
            return Err(HousingError::InstanceNotOwned {
                instance_id: instance.instance_id,
            });
        }
        let entry = self
            .items
            .entry((instance.definition_id, instance.tint_id))
            .or_insert(FixtureItem {
                definition_id: instance.definition_id,
                tint_id: instance.tint_id,
                owned: 0,
                placed: 0,
            });
        if entry.owned <= entry.placed {
            return Err(HousingError::InstanceNotOwned {
                instance_id: instance.instance_id,
            });
        }
        entry.placed += 1;
        Ok(())
    }

    pub fn restore(&mut self, instance: FixtureInstance) -> Result<(), HousingError> {
        let entry = self
            .items
            .get_mut(&(instance.definition_id, instance.tint_id))
            .ok_or(HousingError::InventoryNotOwned {
                instance_id: instance.instance_id,
            })?;
        if entry.placed == 0 {
            return Err(HousingError::InventoryNotOwned {
                instance_id: instance.instance_id,
            });
        }
        entry.placed -= 1;
        Ok(())
    }

    pub fn items(&self) -> impl Iterator<Item = &FixtureItem> {
        self.items.values()
    }
}

/// RECONSTRUCTED instancing key. The original client carries an item instance id next to the
/// fixture guid; the concrete id space is not recovered, so an id is only unique per property.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixtureInstance {
    pub instance_id: u32,
    pub definition_id: i32,
    pub tint_id: i32,
}

/// RECONSTRUCTED ownership record. Property ids are never silently reassigned; a second grant for a
/// property already owned by someone else is rejected. A grant carries the full property definition
/// so a restored server cannot create a lot whose limits or build areas are missing.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyGrant {
    pub owner_guid: u64,
    pub property: PropertyDefinition,
    pub lot_guid: u64,
}

/// A staged inventory change. It is a proposal only: nothing is minted until the owning inventory
/// applies it, and it is dropped unchanged when the housing transaction fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryProposal {
    pub instance_id: u32,
    pub definition_id: i32,
    pub tint_id: i32,
}

impl InventoryProposal {
    pub fn instance(&self) -> FixtureInstance {
        FixtureInstance {
            instance_id: self.instance_id,
            definition_id: self.definition_id,
            tint_id: self.tint_id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagedChange {
    Consume(InventoryProposal),
    Return(InventoryProposal),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlacementOutcome {
    pub placed: PlacedFixture,
    pub saved: LotSnapshot,
    pub staged: StagedChange,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickupOutcome {
    pub removed: PlacedFixture,
    pub staged: StagedChange,
    pub saved: LotSnapshot,
}

/// Local command projection for client request 127/5. The request shape is client evidence; applying
/// the transform to persisted state is reconstructed policy (R).
#[derive(Clone, Debug, PartialEq)]
pub struct SaveFixtureCommand {
    pub owner_guid: u64,
    pub house_guid: u64,
    pub placed: SavedFixture,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveFixtureProposal {
    pub owner_guid: u64,
    pub house_guid: u64,
    pub instance_id: u32,
    pub created: bool,
}

#[derive(Debug)]
struct Lot {
    owner_guid: u64,
    property_id: i32,
    lot_guid: u64,
    zone_id: i32,
    name_id: i32,
    max_fixtures: u32,
    max_landmarks: u32,
    locked: bool,
    build_areas: Vec<[f32; 6]>,
    fixtures: BTreeMap<u64, SavedFixture>,
    permissions: BTreeMap<u64, i32>,
}

impl Lot {
    fn snapshot(&self) -> LotSnapshot {
        LotSnapshot {
            owner_guid: self.owner_guid,
            property_id: self.property_id,
            lot_guid: self.lot_guid,
            zone_id: self.zone_id,
            name_id: self.name_id,
            max_fixtures: self.max_fixtures,
            max_landmarks: self.max_landmarks,
            count: self.fixtures.len() as u32,
            build_areas: self.build_areas.clone(),
            fixtures: self.fixtures.values().cloned().collect(),
            permissions: self
                .permissions
                .iter()
                .map(|(guid, level)| LotPermission {
                    guid: *guid,
                    level: *level,
                })
                .collect(),
            locked: self.locked,
        }
    }
}

/// The restored housing state machine for one server process.
///
/// `Clone` is the staging seam the housing service uses: a caller clones the state, runs one
/// fallible placement/move/pickup against the clone, then commits the clone only after the paired
/// inventory mutation succeeded. Cloning this state never touches a file.
#[derive(Clone, Debug)]
pub struct Housing {
    properties: BTreeMap<i32, PropertyDefinition>,
    grants: BTreeMap<u64, PropertyGrant>,
    lots: BTreeMap<u64, Lot>,
}

impl Default for Housing {
    fn default() -> Self {
        Self::new()
    }
}

impl Housing {
    pub fn new() -> Self {
        Self {
            properties: BTreeMap::new(),
            grants: BTreeMap::new(),
            lots: BTreeMap::new(),
        }
    }

    /// RECONSTRUCTED: a lot guid is derived from the owner and property so a restored snapshot
    /// cannot collide with a live gameplay zone guid.
    pub fn lot_guid(&self, owner_guid: u64, property_id: i32) -> Result<u64, HousingError> {
        if owner_guid == 0 {
            return Err(HousingError::InvalidOwner);
        }
        if property_id <= 0 {
            return Err(HousingError::InvalidProperty);
        }
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in owner_guid
            .to_be_bytes()
            .into_iter()
            .chain((property_id as u32).to_be_bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Ok(HOUSE_GUID_BASE | (hash & 0x00ff_ffff_ffff_ffff))
    }

    pub fn register_property(&mut self, property: PropertyDefinition) {
        property
            .validate()
            .expect("property definitions are private content and must validate");
        if self.properties.contains_key(&property.property_id) {
            return;
        }
        self.properties.insert(property.property_id, property);
    }

    /// Reconstructed owner gate, guided by Legacy `PlayerOwnsHouse(ownerId, zoneDefinitionId)`.
    pub fn grant_property(&mut self, grant: PropertyGrant) -> Result<(), HousingError> {
        if grant.owner_guid == 0 {
            return Err(HousingError::InvalidOwner);
        }
        let property = grant.property;
        property.validate()?;
        let lot_guid = if grant.lot_guid == 0 {
            self.lot_guid(grant.owner_guid, property.property_id)?
        } else {
            grant.lot_guid
        };
        if let Some(existing) = self.grants.get(&lot_guid) {
            if existing.owner_guid == grant.owner_guid
                && existing.property.property_id == property.property_id
            {
                return Ok(());
            }
            return Err(HousingError::PropertyAlreadyOwned);
        }
        let property_id = property.property_id;
        let zone_id = property.zone_id;
        let name_id = property.name_id;
        let max_fixtures = property.max_fixtures;
        let max_landmarks = property.max_landmarks;
        self.properties.entry(property_id).or_insert(property);
        self.grants.insert(
            lot_guid,
            PropertyGrant {
                owner_guid: grant.owner_guid,
                property: self
                    .properties
                    .get(&property_id)
                    .expect("the granted property was just inserted")
                    .clone(),
                lot_guid,
            },
        );
        self.lots.insert(
            lot_guid,
            Lot {
                owner_guid: grant.owner_guid,
                property_id,
                lot_guid,
                zone_id,
                name_id,
                max_fixtures,
                max_landmarks,
                locked: false,
                build_areas: self
                    .properties
                    .get(&property_id)
                    .expect("the granted property was just inserted")
                    .build_areas
                    .clone(),
                fixtures: BTreeMap::new(),
                permissions: BTreeMap::new(),
            },
        );
        Ok(())
    }

    pub fn saved_fixtures(&self, house_guid: u64) -> Result<LotSnapshot, HousingError> {
        self.lots
            .get(&house_guid)
            .map(Lot::snapshot)
            .ok_or(HousingError::LotNotFound)
    }

    /// Test and migration hook: replace a lot's fixture set without inventing a placement. The lot
    /// itself must already exist, the replacement must keep the lot's identity, and its property
    /// fields must validate. The content table is realigned, so placement limits and build areas can
    /// never disagree with the lot that persists them.
    pub fn replace_saved_fixtures(
        &mut self,
        house_guid: u64,
        snapshot: LotSnapshot,
    ) -> Result<(), HousingError> {
        if snapshot.count != snapshot.fixtures.len() as u32 {
            return Err(HousingError::CorruptSnapshot);
        }
        let property = PropertyDefinition {
            property_id: snapshot.property_id,
            zone_id: snapshot.zone_id,
            name_id: snapshot.name_id,
            max_fixtures: snapshot.max_fixtures,
            max_landmarks: snapshot.max_landmarks,
            build_areas: snapshot.build_areas.clone(),
        };
        property
            .validate()
            .map_err(|_| HousingError::CorruptSnapshot)?;
        let lot = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?;
        if lot.lot_guid != snapshot.lot_guid
            || lot.property_id != property.property_id
            || lot.zone_id != snapshot.zone_id
            || lot.name_id != snapshot.name_id
        {
            return Err(HousingError::CorruptSnapshot);
        }
        lot.owner_guid = snapshot.owner_guid;
        lot.max_fixtures = snapshot.max_fixtures;
        lot.max_landmarks = snapshot.max_landmarks;
        lot.build_areas = snapshot.build_areas;
        lot.fixtures = snapshot
            .fixtures
            .into_iter()
            .map(|fixture| {
                (
                    fixture.fixture_guid,
                    fixture.attributed_to(snapshot.owner_guid),
                )
            })
            .collect();
        lot.permissions = snapshot
            .permissions
            .into_iter()
            .map(|permission| (permission.guid, permission.level))
            .collect();
        lot.locked = snapshot.locked;
        self.properties.insert(property.property_id, property);
        Ok(())
    }

    /// T872: add or replace one lot from a snapshot (a visitor's copy of another character's
    /// lot), with the same fixture and property checks as a restore. The property row is added
    /// only when this process has none for it; no grant is recorded, so the copy owns nothing.
    pub fn admit_lot(&mut self, snapshot: LotSnapshot) -> Result<(), HousingError> {
        let property = PropertyDefinition {
            property_id: snapshot.property_id,
            zone_id: snapshot.zone_id,
            name_id: snapshot.name_id,
            max_fixtures: snapshot.max_fixtures,
            max_landmarks: snapshot.max_landmarks,
            build_areas: snapshot.build_areas.clone(),
        };
        property
            .validate()
            .map_err(|_| HousingError::CorruptSnapshot)?;
        let (guid, lot) = self
            .rebuild(SnapshotDocument {
                schema_version: SNAPSHOT_SCHEMA_VERSION,
                properties: vec![snapshot],
            })?
            .pop_first()
            .ok_or(HousingError::CorruptSnapshot)?;
        self.properties
            .entry(property.property_id)
            .or_insert(property);
        self.lots.insert(guid, lot);
        Ok(())
    }

    /// T872: drop a visitor's copy. Only `admit_lot` copies come through here (the service tracks
    /// them), never a lot this character owns.
    pub fn forget_lot(&mut self, house_guid: u64) {
        self.lots.remove(&house_guid);
    }

    pub fn set_locked(
        &mut self,
        actor_guid: u64,
        house_guid: u64,
        locked: bool,
    ) -> Result<(), HousingError> {
        let lot = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?;
        if lot.owner_guid != actor_guid {
            return Err(HousingError::NotPropertyOwner);
        }
        lot.locked = locked;
        Ok(())
    }

    pub fn set_permission(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        guid: u64,
        level: i32,
    ) -> Result<(), HousingError> {
        let lot = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?;
        if lot.owner_guid != owner_guid {
            return Err(HousingError::NotPropertyOwner);
        }
        if level <= 0 {
            lot.permissions.remove(&guid);
        } else {
            lot.permissions.insert(guid, level);
        }
        Ok(())
    }

    /// RECONSTRUCTED guid minting. A rejected request never reaches this function, so the next
    /// accepted placement for the same lot and instance is deterministic across a retry. The fixture
    /// definition participates because an owned instance id and a fixture definition together are
    /// the lot-unique identity.
    fn mint_fixture_guid(
        &self,
        house_guid: u64,
        fixture_definition_id: i32,
        item_instance_id: u32,
    ) -> u64 {
        let mut hash = house_guid
            ^ u64::from(item_instance_id).wrapping_mul(0x9e37_79b9_7f4a_7c15)
            ^ u64::from(fixture_definition_id as u32).wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
        hash ^= hash >> 29;
        hash = hash.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        hash ^= hash >> 32;
        (FIXTURE_GUID_MASK | (hash & 0x00ff_ffff_ffff_ffff)) & !(1 << 55)
    }

    fn permission_for(&self, lot: &Lot, actor_guid: u64) -> Result<(), HousingError> {
        if actor_guid == 0 {
            return Err(HousingError::InvalidOwner);
        }
        if lot.owner_guid == actor_guid {
            return Ok(());
        }
        match lot.permissions.get(&actor_guid) {
            Some(level) if *level >= PERMISSION_CAN_EDIT => Ok(()),
            _ => Err(HousingError::NotPropertyOwner),
        }
    }

    /// Reconstructed edit gate: owner or explicit level-3 editor. Client permission semantics beyond
    /// edit mode are not recovered. The service checks this before entering edit mode; mutations
    /// repeat the check at their state-machine boundary.
    pub fn can_enter(&self, actor_guid: u64, house_guid: u64) -> Result<(), HousingError> {
        let lot = self.clone_lot(house_guid)?;
        self.permission_for(&lot, actor_guid)
    }

    /// RECONSTRUCTED visitor policy: owners and level-1 visitors may enter unlocked lots. Only
    /// owners and level-3 editors pass `can_enter`, which gates every edit operation.
    pub fn can_visit(&self, actor_guid: u64, house_guid: u64) -> Result<(), HousingError> {
        let lot = self.clone_lot(house_guid)?;
        if actor_guid == 0 {
            return Err(HousingError::InvalidOwner);
        }
        if lot.owner_guid == actor_guid {
            return Ok(());
        }
        if lot.locked {
            return Err(HousingError::LotLocked);
        }
        match lot.permissions.get(&actor_guid) {
            Some(level) if *level >= PERMISSION_CAN_VISIT => Ok(()),
            _ => Err(HousingError::NotPermitted),
        }
    }

    /// Client request 127/1 and placement request 127/2 are in the housing guide. Immediate commit
    /// and inventory movement here are reconstructed policy (R), not recovered server behavior.
    pub fn place_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_definition_id: i32,
        item_instance_id: u32,
        transform: FixtureTransform,
        inventory: &mut FixtureItemLedger,
    ) -> Result<PlacementOutcome, HousingError> {
        let staged_lot = self.clone_lot(house_guid)?;
        self.permission_for(&staged_lot, owner_guid)?;
        if staged_lot.locked {
            return Err(HousingError::LotLocked);
        }
        if fixture_definition_id <= 0 || item_instance_id == 0 {
            return Err(HousingError::InvalidPlacement);
        }
        let property = self
            .properties
            .get(&staged_lot.property_id)
            .ok_or(HousingError::InvalidProperty)?;
        if staged_lot.fixtures.len() as u32 >= property.max_fixtures {
            return Err(HousingError::FixtureLimitReached);
        }
        validate_transform(&transform)?;
        if !property.contains(transform.position) {
            return Err(HousingError::OutsideBuildArea);
        }
        if staged_lot.fixtures.values().any(|fixture| {
            fixture.item_instance_id == item_instance_id
                && fixture.fixture_definition_id == fixture_definition_id
        }) {
            return Err(HousingError::InstanceAlreadyPlaced {
                instance_id: item_instance_id,
            });
        }

        let instance = FixtureInstance {
            instance_id: item_instance_id,
            definition_id: fixture_definition_id,
            tint_id: transform.tint_id,
        };
        let mut staged_inventory = inventory.clone();
        staged_inventory.stage(instance)?;
        let placed = PlacedFixture {
            fixture_guid: self.mint_fixture_guid(
                house_guid,
                fixture_definition_id,
                item_instance_id,
            ),
            house_guid,
            item_instance_id,
            fixture_definition_id,
            transform,
        };
        placed.validate()?;
        let staged = StagedChange::Consume(InventoryProposal {
            instance_id: item_instance_id,
            definition_id: fixture_definition_id,
            tint_id: placed.transform.tint_id,
        });
        let fixture = SavedFixture {
            placed_by: owner_guid,
            ..SavedFixture::from(placed.clone())
        };
        let lot = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?;
        if lot.fixtures.contains_key(&fixture.fixture_guid) {
            // A guid collision must never overwrite an existing fixture.
            return Err(HousingError::DuplicateFixture);
        }
        lot.fixtures.insert(fixture.fixture_guid, fixture);
        *inventory = staged_inventory;
        Ok(PlacementOutcome {
            placed,
            saved: lot.snapshot(),
            staged,
        })
    }

    /// Client evidence identifies request 127/3 and reply 127/41. Returning an item to inventory is
    /// reconstructed policy (R); no retail server pickup handler was recovered.
    pub fn pickup_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_guid: u64,
        inventory: &mut FixtureItemLedger,
    ) -> Result<PickupOutcome, HousingError> {
        let staged_lot = self.clone_lot(house_guid)?;
        self.permission_for(&staged_lot, owner_guid)?;
        if staged_lot.locked {
            return Err(HousingError::LotLocked);
        }
        let fixture = staged_lot
            .fixtures
            .get(&fixture_guid)
            .cloned()
            .ok_or(HousingError::UnknownFixture)?;
        let instance = FixtureInstance {
            instance_id: fixture.item_instance_id,
            definition_id: fixture.fixture_definition_id,
            tint_id: fixture.transform.tint_id,
        };
        let mut staged_inventory = inventory.clone();
        staged_inventory.restore(instance)?;
        let removed = PlacedFixture {
            fixture_guid: fixture.fixture_guid,
            house_guid,
            item_instance_id: fixture.item_instance_id,
            fixture_definition_id: fixture.fixture_definition_id,
            transform: fixture.transform.clone(),
        };
        removed.validate()?;
        let lot = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?;
        lot.fixtures.remove(&fixture_guid);
        *inventory = staged_inventory;
        Ok(PickupOutcome {
            removed,
            staged: StagedChange::Return(InventoryProposal {
                instance_id: instance.instance_id,
                definition_id: instance.definition_id,
                tint_id: instance.tint_id,
            }),
            saved: lot.snapshot(),
        })
    }

    /// Client evidence identifies fixture-update reply 127/40. Stable identity across moves is
    /// reconstructed policy (R).
    pub fn move_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_guid: u64,
        transform: FixtureTransform,
        inventory: &mut FixtureItemLedger,
    ) -> Result<FixtureTransform, HousingError> {
        let staged_lot = self.clone_lot(house_guid)?;
        self.permission_for(&staged_lot, owner_guid)?;
        if staged_lot.locked {
            return Err(HousingError::LotLocked);
        }
        if !staged_lot.fixtures.contains_key(&fixture_guid) {
            return Err(HousingError::UnknownFixture);
        }
        validate_transform(&transform)?;
        let property = self
            .properties
            .get(&staged_lot.property_id)
            .ok_or(HousingError::InvalidProperty)?;
        if !property.contains(transform.position) {
            return Err(HousingError::OutsideBuildArea);
        }
        let mut staged_inventory = inventory.clone();
        let fixture = self
            .lots
            .get_mut(&house_guid)
            .ok_or(HousingError::LotNotFound)?
            .fixtures
            .get_mut(&fixture_guid)
            .ok_or(HousingError::UnknownFixture)?;
        staged_inventory.rekey_tint(
            fixture.fixture_definition_id,
            fixture.transform.tint_id,
            transform.tint_id,
        )?;
        fixture.transform = transform.clone();
        *inventory = staged_inventory;
        Ok(transform)
    }

    /// Client evidence identifies request 127/5. Applying its transform and minting a local fixture
    /// guid are reconstructed policy (R); server semantics remain unverified.
    pub fn save_fixture_placement(
        &mut self,
        command: SaveFixtureCommand,
    ) -> Result<SaveFixtureProposal, HousingError> {
        let staged_lot = self.clone_lot(command.house_guid)?;
        self.permission_for(&staged_lot, command.owner_guid)?;
        if staged_lot.locked {
            return Err(HousingError::LotLocked);
        }
        if command.placed.item_instance_id == 0 || command.placed.fixture_definition_id <= 0 {
            return Err(HousingError::InvalidPlacement);
        }
        command.placed.transform.validate()?;

        let existing = staged_lot.fixtures.values().find(|fixture| {
            fixture.item_instance_id == command.placed.item_instance_id
                && fixture.fixture_definition_id == command.placed.fixture_definition_id
        });
        if let Some(existing) = existing {
            if existing.fixture_guid != command.placed.fixture_guid
                || existing.fixture_definition_id != command.placed.fixture_definition_id
            {
                return Err(HousingError::InstanceAlreadyPlaced {
                    instance_id: command.placed.item_instance_id,
                });
            }
            let property = self
                .properties
                .get(&staged_lot.property_id)
                .ok_or(HousingError::InvalidProperty)?;
            if !property.contains(command.placed.transform.position) {
                return Err(HousingError::OutsideBuildArea);
            }
            let fixture = self
                .lots
                .get_mut(&command.house_guid)
                .ok_or(HousingError::LotNotFound)?
                .fixtures
                .get_mut(&command.placed.fixture_guid)
                .ok_or(HousingError::UnknownFixture)?;
            fixture.transform = command.placed.transform;
            return Ok(SaveFixtureProposal {
                owner_guid: command.owner_guid,
                house_guid: command.house_guid,
                instance_id: command.placed.item_instance_id,
                created: false,
            });
        }
        if staged_lot.fixtures.len() as u32 >= staged_lot.max_fixtures {
            return Err(HousingError::FixtureLimitReached);
        }
        let property = self
            .properties
            .get(&staged_lot.property_id)
            .ok_or(HousingError::InvalidProperty)?;
        if !property.contains(command.placed.transform.position) {
            return Err(HousingError::OutsideBuildArea);
        }
        let fixture_guid = if command.placed.fixture_guid == 0 {
            self.mint_fixture_guid(
                command.house_guid,
                command.placed.fixture_definition_id,
                command.placed.item_instance_id,
            )
        } else {
            command.placed.fixture_guid
        };
        let mut placed = command.placed.clone();
        placed.fixture_guid = fixture_guid;
        // The lot was resolved from the command's house guid, so the stored fixture carries that
        // identity; a client-supplied house guid must not be able to persist an orphan fixture.
        placed.house_guid = command.house_guid;
        // Likewise the placer is the authorized actor, never a caller-supplied guid.
        placed.placed_by = command.owner_guid;
        let lot = self
            .lots
            .get_mut(&command.house_guid)
            .ok_or(HousingError::LotNotFound)?;
        if lot.fixtures.contains_key(&fixture_guid) {
            return Err(HousingError::DuplicateFixture);
        }
        lot.fixtures.insert(fixture_guid, placed.clone());
        Ok(SaveFixtureProposal {
            owner_guid: command.owner_guid,
            house_guid: command.house_guid,
            instance_id: placed.item_instance_id,
            created: true,
        })
    }

    pub fn validate_snapshot(&self) -> Result<(), HousingError> {
        let mut lots = BTreeSet::new();
        let mut properties = BTreeSet::new();
        for lot in self.lots.values() {
            if lot.owner_guid == 0
                || lot.lot_guid == 0
                || lot.property_id <= 0
                || lot.zone_id <= 0
                || lot.name_id <= 0
            {
                return Err(HousingError::CorruptSnapshot);
            }
            if !lots.insert(lot.lot_guid) {
                return Err(HousingError::CorruptSnapshot);
            }
            if !properties.insert(lot.property_id) {
                return Err(HousingError::DuplicateProperty {
                    property_id: lot.property_id,
                });
            }
            let mut fixtures = BTreeSet::new();
            let mut instances = BTreeSet::new();
            for fixture in lot.fixtures.values() {
                if fixture.house_guid != lot.lot_guid {
                    return Err(HousingError::CorruptSnapshot);
                }
                fixture.to_placement()?;
                if !fixtures.insert(fixture.fixture_guid) {
                    return Err(HousingError::DuplicateFixture);
                }
                if !instances.insert((fixture.fixture_definition_id, fixture.item_instance_id)) {
                    return Err(HousingError::InstanceAlreadyPlaced {
                        instance_id: fixture.item_instance_id,
                    });
                }
            }
            if lot.fixtures.len() as u32 > lot.max_fixtures {
                return Err(HousingError::FixtureLimitReached);
            }
        }
        Ok(())
    }

    /// Path-free snapshot rows. This is the seam the lead serializes into the single canonical
    /// save document; it never chooses a file, so housing cannot write a second ledger.
    pub fn snapshot_lots(&self) -> Vec<LotSnapshot> {
        self.lots.values().map(Lot::snapshot).collect()
    }

    /// Path-free restore of the rows above. Every row is validated before any live state is
    /// replaced, so a rejected document leaves the caller exactly as it was.
    pub fn restore_lots(&mut self, lots: Vec<LotSnapshot>) -> Result<(), HousingError> {
        for property in &lots {
            if property.count != property.fixtures.len() as u32 {
                return Err(HousingError::CorruptSnapshot);
            }
        }
        let restored = self.rebuild(SnapshotDocument {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            properties: lots,
        })?;
        let mut properties = BTreeMap::new();
        let mut grants = BTreeMap::new();
        for lot in restored.values() {
            let property = PropertyDefinition {
                property_id: lot.property_id,
                zone_id: lot.zone_id,
                name_id: lot.name_id,
                max_fixtures: lot.max_fixtures,
                max_landmarks: lot.max_landmarks,
                build_areas: lot.build_areas.clone(),
            };
            property
                .validate()
                .map_err(|_| HousingError::CorruptSnapshot)?;
            properties
                .entry(lot.property_id)
                .or_insert(property.clone());
            grants.insert(
                lot.lot_guid,
                PropertyGrant {
                    owner_guid: lot.owner_guid,
                    property,
                    lot_guid: lot.lot_guid,
                },
            );
        }
        let staged = Housing {
            properties,
            grants,
            lots: restored,
        };
        if staged.validate_snapshot().is_err() {
            // A rejected document must not replace the live state, not even the content table: the
            // restored lots were never swapped in, so a partial adoption would aim the live lots at
            // another document's limits.
            return Err(HousingError::CorruptSnapshot);
        }
        self.properties = staged.properties;
        self.grants = staged.grants;
        self.lots = staged.lots;
        Ok(())
    }

    /// Atomic snapshot write. A rejected snapshot leaves the previous file untouched.
    ///
    /// Prefer `snapshot_lots` plus the lead's canonical `game_store` document: this standalone
    /// path exists for the deployed local snapshot and is not a second persistence owner.
    pub fn save_state(&self, path: &Path) -> Result<(), HousingError> {
        self.validate_snapshot()?;
        let document = SnapshotDocument {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            properties: self.snapshot_lots(),
        };
        let bytes =
            serde_json::to_vec_pretty(&document).map_err(|_| HousingError::CorruptSnapshot)?;
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(HousingError::SnapshotTooLarge);
        }
        replace_bytes(path, &bytes).map_err(HousingError::from)
    }

    /// Restore a snapshot. A rejected document leaves the in-memory state untouched.
    pub fn load_state(&mut self, path: &Path) -> Result<(), HousingError> {
        let file = File::open(path).map_err(HousingError::from)?;
        let mut bytes = Vec::new();
        file.take((MAX_SNAPSHOT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(HousingError::from)?;
        if bytes.len() > MAX_SNAPSHOT_BYTES {
            return Err(HousingError::SnapshotTooLarge);
        }
        let document: SnapshotDocument =
            serde_json::from_slice(&bytes).map_err(|_| HousingError::CorruptSnapshot)?;
        if document.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(HousingError::CorruptSnapshot);
        }
        self.restore_lots(document.properties)
    }

    fn rebuild(&self, document: SnapshotDocument) -> Result<BTreeMap<u64, Lot>, HousingError> {
        let mut lots = BTreeMap::new();
        for snapshot in document.properties {
            let expected = snapshot.count as usize;
            let mut fixtures = BTreeMap::new();
            for fixture in &snapshot.fixtures {
                fixture
                    .to_placement()
                    .map_err(|_| HousingError::CorruptSnapshot)?;
                if fixtures
                    .insert(
                        fixture.fixture_guid,
                        fixture.clone().attributed_to(snapshot.owner_guid),
                    )
                    .is_some()
                {
                    return Err(HousingError::CorruptSnapshot);
                }
            }
            if fixtures.len() != expected {
                return Err(HousingError::CorruptSnapshot);
            }
            if fixtures.len() as u32 > snapshot.max_fixtures {
                return Err(HousingError::CorruptSnapshot);
            }
            let lot = Lot {
                owner_guid: snapshot.owner_guid,
                property_id: snapshot.property_id,
                lot_guid: snapshot.lot_guid,
                zone_id: snapshot.zone_id,
                name_id: snapshot.name_id,
                max_fixtures: snapshot.max_fixtures,
                max_landmarks: snapshot.max_landmarks,
                locked: snapshot.locked,
                build_areas: snapshot.build_areas,
                fixtures,
                permissions: snapshot
                    .permissions
                    .into_iter()
                    .map(|permission| (permission.guid, permission.level))
                    .collect(),
            };
            if lots.insert(snapshot.lot_guid, lot).is_some() {
                // Two rows for one lot would otherwise drop the first lot and its fixtures silently.
                return Err(HousingError::CorruptSnapshot);
            }
        }
        Ok(lots)
    }
}

impl Clone for Lot {
    fn clone(&self) -> Self {
        Self {
            owner_guid: self.owner_guid,
            property_id: self.property_id,
            lot_guid: self.lot_guid,
            zone_id: self.zone_id,
            name_id: self.name_id,
            max_fixtures: self.max_fixtures,
            max_landmarks: self.max_landmarks,
            locked: self.locked,
            build_areas: self.build_areas.clone(),
            fixtures: self.fixtures.clone(),
            permissions: self.permissions.clone(),
        }
    }
}

impl Housing {
    fn clone_lot(&self, house_guid: u64) -> Result<Lot, HousingError> {
        self.lots
            .get(&house_guid)
            .cloned()
            .ok_or(HousingError::LotNotFound)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotDocument {
    schema_version: u32,
    properties: Vec<LotSnapshot>,
}

fn validate_transform(transform: &FixtureTransform) -> Result<(), HousingError> {
    for value in transform
        .position
        .iter()
        .chain(transform.rotation.iter())
        .chain(transform.tilt.iter())
        .chain(std::iter::once(&transform.scale))
    {
        if !value.is_finite() {
            return Err(HousingError::NonFiniteTransform);
        }
    }
    if transform
        .position
        .iter()
        .chain(transform.rotation.iter())
        .any(|value| value.abs() > MAX_FIXTURE_MAGNITUDE)
        || transform.scale <= 0.0
        || transform.scale > MAX_FIXTURE_MAGNITUDE
    {
        return Err(HousingError::TransformOutOfBounds);
    }
    // The rotation is the one source of the facing; a stored tilt that disagrees is refused with
    // `NonUnitFacing` even when it is itself unit length.
    let implied = FixtureTransform::facing(transform.rotation);
    if transform
        .tilt
        .iter()
        .zip(implied)
        .any(|(stored, implied)| (stored - implied).abs() > FACING_TOLERANCE)
    {
        return Err(HousingError::NonUnitFacing);
    }
    Ok(())
}

/// Same-directory atomic replacement, matching the `character_store` contract: a synced temporary
/// file replaces the destination, and the previous snapshot is never deleted before replacement.
/// This module keeps its own copy so the `#[path]`-included test target compiles before the lead
/// registers the module in `src/lib.rs`.
fn replace_bytes(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut temporary = create_temporary(path)?;
    let file = temporary.file.as_mut().expect("temporary file is open");
    file.write_all(bytes)?;
    file.sync_all()?;
    // Windows requires closing our temporary-file handle before replacement.
    drop(temporary.file.take());
    let mut retries = 0;
    loop {
        match fs::rename(&temporary.path, path) {
            Ok(()) => break,
            // A concurrent Windows reader without FILE_SHARE_DELETE can deny replacement briefly.
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(5 | 32))
                    && retries < 20 =>
            {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
    temporary.committed = true;
    Ok(())
}

struct TemporaryFile {
    path: PathBuf,
    file: Option<File>,
    committed: bool,
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.committed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn create_temporary(destination: &Path) -> io::Result<TemporaryFile> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.{}.{}.tmp",
            destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("housing"),
            std::process::id(),
            sequence
        );
        let candidate = parent.join(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                return Ok(TemporaryFile {
                    path: candidate,
                    file: Some(file),
                    committed: false,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a housing snapshot temporary file",
    ))
}
