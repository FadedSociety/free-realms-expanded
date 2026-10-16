//! 026: housing inventory transactions for one restored server.
//!
//! This module is the transaction boundary between the item 006 housing state machine and the
//! player's owned furniture. Item 006 keeps a `FixtureItemLedger` in memory only and commits a
//! placement before anything is written, so a failed snapshot write leaves the placed fixture and
//! the consumed unit live in memory while the durable state says otherwise. This module owns the
//! durable owned-furniture pool, derives the ledger from the pair, and rolls back on failure.
//!
//! ORIGINAL facts used here (all outside Git, see `notes/Research/Pieces/026*.md`):
//! * housing is packet family 127; `ClientHousingPacketPlaceFixtureRequest` = 1,
//!   `ClientHousingPacketPlaceFixture` = 2, `ClientHousingPacketPickupFixture` = 3,
//!   `ClientHousingPacketSaveFixture` = 5, `HousingPacketFixtureRemove` = 41,
//!   `HousingPacketFixtureItemList` = 43
//!   (`lab/decompiled/server/LibSOE/SOE/BaseHousingPackets.cs`).
//! * `HousingPacketFixtureItemList` carries `FixtureInstanceInfo` (fixture guid plus
//!   `ItemDefinitionId`) and `FixtureDefinition` (fixture id, `ItemDefinitionId`, model, category,
//!   `LuaCall`, effects). The client constructs that packet
//!   (`lab/decompiled/client/FreeRealms/pseudocode.c`, `HousingPacketFixtureItemList::vftable`), so
//!   the client sees a list of owned furniture items alongside the lot's placed fixtures
//!   (`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseHousingPacket/`).
//! * `PlayerHousingInstanceData` carries `MaxFixtureCount`, `CurFixtureCount` and the fixture map;
//!   `FixtureInstance` carries the fixture guid, house guid, fixture definition id, position,
//!   rotation, tilt, tint id and scale
//!   (`Sanctuary.Packet.Common/Housing/PlayerHousingInstanceData.cs`, `FixtureInstance.cs`).
//!
//! RECONSTRUCTED here: the durable pool document, its schema, the transaction ordering, every
//! numeric bound and every error. No original server code for placement, pick-up or saving was
//! recovered, so conservation is a local invariant, not a native rule. The original client has not
//! accepted anything this module produces.
//!
//! Deliberate boundaries:
//! * the pool counts owned units; the item identity behind a unit is item 081's inventory, which is
//!   not wired here. Nothing in this module mints currency or grants a reward;
//! * a unit's durable tint is authoritative only while the unit is unplaced. While placed, the
//!   fixture's tint is authoritative, because item 006's `rekey_tint`/`restore` pair returns a
//!   picked-up unit to the tint it was placed with;
//! * one unit placed in two lots at two different tints cannot be represented by that count model
//!   and is refused as `AmbiguousPlacedTint` instead of being silently folded;
//! * collision, snap, landmark scoring and upkeep stay out of scope exactly as in item 006.
//!
//! ponytail: one pool document next to the existing housing snapshot, one derived ledger pass, no
//! journal, no second index. Two-file commit is used only when a pick-up must move a repainted
//! unit's tint; see `restore_after_failed_housing_write`.

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
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
    FixtureInstance, FixtureItemLedger, FixtureTransform, Housing, HousingError, LotSnapshot,
    PickupOutcome, PlacementOutcome, PropertyDefinition, PropertyGrant,
};

use housing::{MAX_SNAPSHOT_BYTES as MAX_HOUSING_BYTES, SNAPSHOT_SCHEMA_VERSION};

/// RECONSTRUCTED schema version for the owned-furniture pool.
pub const STOCK_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED pool document bound, in the same spirit as `housing::MAX_SNAPSHOT_BYTES`.
pub const MAX_STOCK_BYTES: usize = 256 * 1024;
/// RECONSTRUCTED pool size bound.
pub const MAX_STOCK_INSTANCES: usize = 8192;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// RECONSTRUCTED unit row of the owned-furniture pool. `instance_id` is the captured item instance
/// the housing module references as `item_instance_id`; `tint_id` is the unit's own tint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedFixture {
    pub instance_id: u32,
    pub definition_id: i32,
    pub tint_id: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransactionError {
    /// The housing state machine refused the request. Nothing changed, in memory or on disk.
    Rejected(HousingError),
    /// The commit write failed after the in-memory change; the transaction rolled back.
    PersistHousing(HousingError),
    /// The pool write failed after the in-memory change; the transaction rolled back.
    PersistStock(io::ErrorKind),
    /// A compensating write failed, so the durable pair no longer matches its pre-transaction bytes.
    RollbackFailed,
    HousingIo(io::ErrorKind),
    HousingCorrupt(&'static str),
    StockIo(io::ErrorKind),
    StockCorrupt(&'static str),
    InvalidStockInstance {
        instance_id: u32,
    },
    DuplicateStockInstance {
        instance_id: u32,
    },
    UnknownStockInstance {
        instance_id: u32,
    },
    /// The unit is placed on a lot, so it cannot leave the pool.
    InstanceInUse {
        instance_id: u32,
    },
    /// A placed fixture references a unit the pool does not own.
    PlacedInstanceNotOwned {
        instance_id: u32,
    },
    PlacedExceedsOwned {
        definition_id: i32,
        owned: u32,
        placed: u32,
    },
    /// One unit is placed in two lots at two different tints; the count model cannot represent it.
    AmbiguousPlacedTint {
        instance_id: u32,
    },
    /// `PlayerHousingInstanceData.CurFixtureCount` disagrees with the stored fixture map.
    FixtureCountMismatch {
        lot_guid: u64,
        count: u32,
        fixtures: u32,
    },
}

impl From<HousingError> for TransactionError {
    fn from(error: HousingError) -> Self {
        Self::Rejected(error)
    }
}

/// RECONSTRUCTED durable pool of owned furniture units. Sorted by `(definition_id, instance_id)`
/// and unique on that pair, so a serialized document is deterministic.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FurnitureStock {
    instances: Vec<OwnedFixture>,
}

impl FurnitureStock {
    pub fn instances(&self) -> &[OwnedFixture] {
        &self.instances
    }

    pub fn owned(&self, definition_id: i32) -> u32 {
        self.instances
            .iter()
            .filter(|row| row.definition_id == definition_id)
            .count() as u32
    }

    pub fn find(&self, definition_id: i32, instance_id: u32) -> Option<&OwnedFixture> {
        self.index(definition_id, instance_id)
            .map(|index| &self.instances[index])
    }

    fn index(&self, definition_id: i32, instance_id: u32) -> Option<usize> {
        self.instances
            .binary_search_by_key(&(definition_id, instance_id), |row| {
                (row.definition_id, row.instance_id)
            })
            .ok()
    }

    pub(crate) fn grant(&mut self, instance: OwnedFixture) -> Result<(), TransactionError> {
        if instance.definition_id <= 0 || instance.instance_id == 0 || instance.tint_id < 0 {
            return Err(TransactionError::InvalidStockInstance {
                instance_id: instance.instance_id,
            });
        }
        if self
            .index(instance.definition_id, instance.instance_id)
            .is_some()
        {
            return Err(TransactionError::DuplicateStockInstance {
                instance_id: instance.instance_id,
            });
        }
        if self.instances.len() >= MAX_STOCK_INSTANCES {
            return Err(TransactionError::StockCorrupt(
                "owned pool exceeds its bound",
            ));
        }
        let position = self.instances.partition_point(|row| {
            (row.definition_id, row.instance_id) < (instance.definition_id, instance.instance_id)
        });
        self.instances.insert(position, instance);
        Ok(())
    }

    pub(crate) fn remove(
        &mut self,
        definition_id: i32,
        instance_id: u32,
    ) -> Result<OwnedFixture, TransactionError> {
        let index = self
            .index(definition_id, instance_id)
            .ok_or(TransactionError::UnknownStockInstance { instance_id })?;
        Ok(self.instances.remove(index))
    }

    pub(crate) fn set_tint(
        &mut self,
        definition_id: i32,
        instance_id: u32,
        tint_id: i32,
    ) -> Result<(), TransactionError> {
        if tint_id < 0 {
            return Err(TransactionError::InvalidStockInstance { instance_id });
        }
        let index = self
            .index(definition_id, instance_id)
            .ok_or(TransactionError::UnknownStockInstance { instance_id })?;
        self.instances[index].tint_id = tint_id;
        Ok(())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, TransactionError> {
        let document = StockDocument {
            schema_version: STOCK_SCHEMA_VERSION,
            instances: self.instances.clone(),
        };
        let bytes = serde_json::to_vec_pretty(&document)
            .map_err(|_| TransactionError::StockCorrupt("owned pool does not serialize"))?;
        if bytes.len() > MAX_STOCK_BYTES {
            return Err(TransactionError::StockCorrupt(
                "owned pool exceeds its byte bound",
            ));
        }
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, TransactionError> {
        if bytes.len() > MAX_STOCK_BYTES {
            return Err(TransactionError::StockCorrupt(
                "owned pool exceeds its byte bound",
            ));
        }
        let document: StockDocument = serde_json::from_slice(bytes)
            .map_err(|_| TransactionError::StockCorrupt("owned pool is not a valid document"))?;
        if document.schema_version != STOCK_SCHEMA_VERSION {
            return Err(TransactionError::StockCorrupt(
                "unsupported owned pool version",
            ));
        }
        let mut stock = Self::default();
        for instance in document.instances {
            stock.grant(instance)?;
        }
        Ok(stock)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StockDocument {
    schema_version: u32,
    instances: Vec<OwnedFixture>,
}

/// ORIGINAL: `PlayerHousingInstanceData.MaxFixtureCount` / `CurFixtureCount`. The owned count comes
/// from this module's pool, the placed count from the lot snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionBalance {
    pub definition_id: i32,
    pub owned: u32,
    pub placed: u32,
    pub available: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conservation {
    pub definitions: Vec<DefinitionBalance>,
    pub owned: u32,
    pub placed: u32,
    pub available: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacedUnit {
    pub lot_guid: u64,
    pub fixture_guid: u64,
    pub definition_id: i32,
    pub instance_id: u32,
    pub tint_id: i32,
}

#[derive(Clone)]
struct Rollback {
    lot: LotSnapshot,
    stock: FurnitureStock,
}

/// The transaction boundary. It owns the item 006 state machine, the durable owned-furniture pool
/// and both file paths, and it is the only writer of either file.
pub struct HousingInventoryTransaction {
    housing: Housing,
    lots: BTreeSet<u64>,
    stock: FurnitureStock,
    housing_path: PathBuf,
    stock_path: PathBuf,
}

/// The state machine has no `Debug` of its own; the transaction reports only its paths and sizes so
/// a failed commit can be logged without dumping a whole lot.
impl std::fmt::Debug for HousingInventoryTransaction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HousingInventoryTransaction")
            .field("housing_path", &self.housing_path)
            .field("stock_path", &self.stock_path)
            .field("lots", &self.lots.len())
            .field("owned_units", &self.stock.instances.len())
            .finish()
    }
}

impl HousingInventoryTransaction {
    /// Load the durable pair. A missing file is an empty half; a corrupt file, an unloadable pair or
    /// a pair whose owned pool does not cover its placed fixtures is an error.
    pub fn open(
        housing_path: impl Into<PathBuf>,
        stock_path: impl Into<PathBuf>,
    ) -> Result<Self, TransactionError> {
        let housing_path = housing_path.into();
        let stock_path = stock_path.into();
        let mut housing = Housing::new();
        let mut lots = BTreeSet::new();
        if let Some(bytes) = read_bounded(&housing_path, MAX_HOUSING_BYTES)
            .map_err(|error| TransactionError::HousingIo(error.kind()))?
        {
            let document: HousingDocument = serde_json::from_slice(&bytes).map_err(|_| {
                TransactionError::HousingCorrupt("housing snapshot is not a valid document")
            })?;
            if document.schema_version != SNAPSHOT_SCHEMA_VERSION {
                return Err(TransactionError::HousingCorrupt(
                    "unsupported housing snapshot version",
                ));
            }
            for lot in &document.properties {
                lots.insert(lot.lot_guid);
            }
            housing
                .load_state(&housing_path)
                .map_err(|_| TransactionError::HousingCorrupt("housing snapshot is rejected"))?;
        }
        let stock = match read_bounded(&stock_path, MAX_STOCK_BYTES)
            .map_err(|error| TransactionError::StockIo(error.kind()))?
        {
            Some(bytes) => FurnitureStock::from_bytes(&bytes)?,
            None => FurnitureStock::default(),
        };
        let transaction = Self {
            housing,
            lots,
            stock,
            housing_path,
            stock_path,
        };
        transaction.verify()?;
        Ok(transaction)
    }

    pub fn housing(&self) -> &Housing {
        &self.housing
    }

    pub fn stock(&self) -> &FurnitureStock {
        &self.stock
    }

    pub fn housing_path(&self) -> &Path {
        &self.housing_path
    }

    pub fn stock_path(&self) -> &Path {
        &self.stock_path
    }

    /// ORIGINAL: ownership is granted before a placement is answered (`PlayerOwnsHouse`). Property
    /// acquisition itself is item 021; this grants in memory and the lot becomes durable at the next
    /// successful placement, pick-up or move.
    pub fn grant_property(
        &mut self,
        owner_guid: u64,
        property: PropertyDefinition,
    ) -> Result<u64, TransactionError> {
        let lot_guid = self.housing.lot_guid(owner_guid, property.property_id)?;
        self.housing.grant_property(PropertyGrant {
            owner_guid,
            property,
            lot_guid,
        })?;
        self.lots.insert(lot_guid);
        Ok(lot_guid)
    }

    /// Add one owned unit to the pool. The write is atomic: a rejected or failed write leaves both
    /// the memory and the file at their previous value. The real inventory (item 081) is the caller.
    pub fn grant_instance(&mut self, instance: OwnedFixture) -> Result<(), TransactionError> {
        let previous = self.stock.clone();
        self.stock.grant(instance)?;
        if let Err(error) = self.save_stock() {
            self.stock = previous;
            return Err(error);
        }
        Ok(())
    }

    /// Remove one owned unit. A placed unit cannot leave the pool, because that is what conserves
    /// placement and removal against the durable split.
    pub fn consume_instance(
        &mut self,
        definition_id: i32,
        instance_id: u32,
    ) -> Result<OwnedFixture, TransactionError> {
        if self
            .placed_units()?
            .iter()
            .any(|unit| unit.definition_id == definition_id && unit.instance_id == instance_id)
        {
            return Err(TransactionError::InstanceInUse { instance_id });
        }
        let previous = self.stock.clone();
        let removed = self.stock.remove(definition_id, instance_id)?;
        if let Err(error) = self.save_stock() {
            self.stock = previous;
            return Err(error);
        }
        Ok(removed)
    }

    /// ORIGINAL: `ClientHousingPacketPlaceFixtureRequest` (1) is answered by
    /// `ClientHousingPacketPlaceFixture` (2). The placement consumes one owned unit and commits the
    /// lot snapshot. A failed commit restores the lot and the derived pool state.
    pub fn place_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_definition_id: i32,
        item_instance_id: u32,
        transform: FixtureTransform,
    ) -> Result<PlacementOutcome, TransactionError> {
        let rollback = self.capture(house_guid)?;
        let mut ledger = self.ledger()?;
        let outcome = self.housing.place_fixture(
            owner_guid,
            house_guid,
            fixture_definition_id,
            item_instance_id,
            transform,
            &mut ledger,
        )?;
        if let Err(error) = self.verify() {
            self.restore(house_guid, &rollback)?;
            return Err(error);
        }
        if let Err(error) = self.housing.save_state(&self.housing_path) {
            self.restore(house_guid, &rollback)?;
            return Err(TransactionError::PersistHousing(error));
        }
        Ok(outcome)
    }

    /// ORIGINAL: `ClientHousingPacketPickupFixture` (3) is answered by `HousingPacketFixtureRemove`
    /// (41). The unit returns to the pool at the tint it was placed with.
    pub fn pickup_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_guid: u64,
    ) -> Result<PickupOutcome, TransactionError> {
        let rollback = self.capture(house_guid)?;
        let mut ledger = self.ledger()?;
        let outcome =
            self.housing
                .pickup_fixture(owner_guid, house_guid, fixture_guid, &mut ledger)?;
        let repainted = self
            .stock
            .find(
                outcome.removed.fixture_definition_id,
                outcome.removed.item_instance_id,
            )
            .is_some_and(|row| row.tint_id != outcome.removed.transform.tint_id);
        if repainted {
            self.stock.set_tint(
                outcome.removed.fixture_definition_id,
                outcome.removed.item_instance_id,
                outcome.removed.transform.tint_id,
            )?;
        }
        if let Err(error) = self.verify() {
            self.restore(house_guid, &rollback)?;
            return Err(error);
        }
        if repainted {
            // The pool write comes first: if the lot write fails, the durable pair still describes
            // the fixture as placed at the tint the pool now records, which is the same state.
            if let Err(error) = self.save_stock() {
                self.restore(house_guid, &rollback)?;
                return Err(error);
            }
        }
        if let Err(error) = self.housing.save_state(&self.housing_path) {
            self.restore_after_failed_housing_write(house_guid, &rollback, repainted)?;
            return Err(TransactionError::PersistHousing(error));
        }
        Ok(outcome)
    }

    /// ORIGINAL: `HousingPacketFixtureUpdate` (40). A move never changes ownership; a tint change
    /// moves the placed unit's effective tint because the fixture is authoritative while placed.
    pub fn move_fixture(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        fixture_guid: u64,
        transform: FixtureTransform,
    ) -> Result<FixtureTransform, TransactionError> {
        let rollback = self.capture(house_guid)?;
        let mut ledger = self.ledger()?;
        let moved = self.housing.move_fixture(
            owner_guid,
            house_guid,
            fixture_guid,
            transform,
            &mut ledger,
        )?;
        if let Err(error) = self.verify() {
            self.restore(house_guid, &rollback)?;
            return Err(error);
        }
        if let Err(error) = self.housing.save_state(&self.housing_path) {
            self.restore(house_guid, &rollback)?;
            return Err(TransactionError::PersistHousing(error));
        }
        Ok(moved)
    }

    /// Every placed fixture of every known lot, ordered by lot guid and fixture guid.
    pub fn placed_units(&self) -> Result<Vec<PlacedUnit>, TransactionError> {
        let mut units = Vec::new();
        for lot_guid in &self.lots {
            let lot = self.housing.saved_fixtures(*lot_guid)?;
            if lot.count != lot.fixtures.len() as u32 {
                return Err(TransactionError::FixtureCountMismatch {
                    lot_guid: *lot_guid,
                    count: lot.count,
                    fixtures: lot.fixtures.len() as u32,
                });
            }
            for fixture in &lot.fixtures {
                if fixture.house_guid != *lot_guid {
                    return Err(TransactionError::HousingCorrupt(
                        "fixture house guid differs from its lot",
                    ));
                }
                units.push(PlacedUnit {
                    lot_guid: *lot_guid,
                    fixture_guid: fixture.fixture_guid,
                    definition_id: fixture.fixture_definition_id,
                    instance_id: fixture.item_instance_id,
                    tint_id: fixture.transform.tint_id,
                });
            }
        }
        Ok(units)
    }

    /// The conservation invariant: every placed fixture is an owned unit, and a definition never has
    /// more units placed than it owns.
    pub fn conservation(&self) -> Result<Conservation, TransactionError> {
        let placed = self.placed_units()?;
        let mut rows: BTreeMap<i32, (u32, u32)> = BTreeMap::new();
        for row in self.stock.instances() {
            let entry = rows.entry(row.definition_id).or_insert((0, 0));
            entry.0 = entry
                .0
                .checked_add(1)
                .ok_or(TransactionError::StockCorrupt("owned count overflow"))?;
        }
        for unit in &placed {
            if self
                .stock
                .find(unit.definition_id, unit.instance_id)
                .is_none()
            {
                return Err(TransactionError::PlacedInstanceNotOwned {
                    instance_id: unit.instance_id,
                });
            }
            let entry = rows.entry(unit.definition_id).or_insert((0, 0));
            entry.1 = entry
                .1
                .checked_add(1)
                .ok_or(TransactionError::StockCorrupt("placed count overflow"))?;
        }
        let mut definitions = Vec::with_capacity(rows.len());
        let mut owned = 0u32;
        let mut placed_total = 0u32;
        for (definition_id, (definition_owned, definition_placed)) in rows {
            if definition_placed > definition_owned {
                return Err(TransactionError::PlacedExceedsOwned {
                    definition_id,
                    owned: definition_owned,
                    placed: definition_placed,
                });
            }
            owned = owned.saturating_add(definition_owned);
            placed_total = placed_total.saturating_add(definition_placed);
            definitions.push(DefinitionBalance {
                definition_id,
                owned: definition_owned,
                placed: definition_placed,
                available: definition_owned - definition_placed,
            });
        }
        Ok(Conservation {
            definitions,
            owned,
            placed: placed_total,
            available: owned.saturating_sub(placed_total),
        })
    }

    /// Rebuild the item 006 ledger from the durable pair. This is the same view item 006 stages
    /// against, so a placement can never see more available units than the pair records.
    pub fn ledger(&self) -> Result<FixtureItemLedger, TransactionError> {
        let _ = self.conservation()?;
        let placed = self.placed_units()?;
        let mut effective: BTreeMap<(i32, u32), i32> = BTreeMap::new();
        for unit in &placed {
            match effective.entry((unit.definition_id, unit.instance_id)) {
                std::collections::btree_map::Entry::Occupied(entry) => {
                    if *entry.get() != unit.tint_id {
                        return Err(TransactionError::AmbiguousPlacedTint {
                            instance_id: unit.instance_id,
                        });
                    }
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(unit.tint_id);
                }
            }
        }
        let mut groups: BTreeMap<(i32, i32), BTreeSet<u32>> = BTreeMap::new();
        for row in self.stock.instances() {
            let tint_id = effective
                .get(&(row.definition_id, row.instance_id))
                .copied()
                .unwrap_or(row.tint_id);
            groups
                .entry((row.definition_id, tint_id))
                .or_default()
                .insert(row.instance_id);
        }
        let mut ledger = FixtureItemLedger::default();
        for ((definition_id, tint_id), ids) in groups {
            if !ledger.seed(definition_id, tint_id, ids) {
                return Err(TransactionError::StockCorrupt("owned pool does not seed"));
            }
        }
        for unit in &placed {
            ledger
                .stage(FixtureInstance {
                    instance_id: unit.instance_id,
                    definition_id: unit.definition_id,
                    tint_id: unit.tint_id,
                })
                .map_err(|_| TransactionError::StockCorrupt("placed unit has no pool capacity"))?;
        }
        Ok(ledger)
    }

    /// Structural check plus a full ledger derivation. Every commit runs this before it writes.
    pub fn verify(&self) -> Result<Conservation, TransactionError> {
        let report = self.conservation()?;
        let _ = self.ledger()?;
        Ok(report)
    }

    fn capture(&self, house_guid: u64) -> Result<Rollback, TransactionError> {
        Ok(Rollback {
            lot: self.housing.saved_fixtures(house_guid)?,
            stock: self.stock.clone(),
        })
    }

    fn restore(&mut self, house_guid: u64, rollback: &Rollback) -> Result<(), TransactionError> {
        self.housing
            .replace_saved_fixtures(house_guid, rollback.lot.clone())
            .map_err(|_| TransactionError::RollbackFailed)?;
        self.stock = rollback.stock.clone();
        Ok(())
    }

    /// A pick-up that already wrote the pool but could not write the lot. The pool row is put back
    /// so the durable pair matches its pre-transaction bytes; when even that write fails, the
    /// reported residue is harmless because a placed unit's tint follows its fixture, and the
    /// in-memory state is rolled back either way.
    fn restore_after_failed_housing_write(
        &mut self,
        house_guid: u64,
        rollback: &Rollback,
        pool_written: bool,
    ) -> Result<(), TransactionError> {
        if pool_written {
            let bytes = rollback
                .stock
                .to_bytes()
                .map_err(|_| TransactionError::RollbackFailed)?;
            atomic_write(&self.stock_path, &bytes).map_err(|_| TransactionError::RollbackFailed)?;
        }
        self.restore(house_guid, rollback)
    }

    fn save_stock(&self) -> Result<(), TransactionError> {
        let bytes = self.stock.to_bytes()?;
        atomic_write(&self.stock_path, &bytes)
            .map_err(|error| TransactionError::PersistStock(error.kind()))
    }
}

/// RECONSTRUCTED mirror of the housing snapshot's envelope, read only to learn which lot guids exist
/// before `Housing::load_state` validates the document itself. Item 006 owns that schema.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HousingDocument {
    schema_version: u32,
    properties: Vec<LotSnapshot>,
}

fn read_bounded(path: &Path, maximum: usize) -> io::Result<Option<Vec<u8>>> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take((maximum + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "durable pairing file exceeds its bound",
        ));
    }
    Ok(Some(bytes))
}

/// Same-directory atomic replacement for the pool file. The housing snapshot has its own copy
/// inside item 006, and `character_store::replace_bytes` is not reachable from the `#[path]`-included
/// test target, so this module keeps the smallest equivalent: write a synced temporary file in the
/// destination directory, then rename it over the destination with a bounded Windows retry for a
/// transient sharing violation.
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = None;
    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("furniture-stock"),
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
                temporary = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temporary_path, mut file) = temporary.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not create an owned pool temporary file",
        )
    })?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        let mut retries = 0;
        loop {
            match fs::rename(&temporary_path, path) {
                Ok(()) => return Ok(()),
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
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}
