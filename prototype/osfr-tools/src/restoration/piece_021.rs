//! Property acquisition (MMO work item 021): recovered property definitions, purchase/grant
//! channels, once-only ownership and Station Cash accounting.
//!
//! ORIGINAL facts this module is built on (all read outside this file, listed in
//! `notes/Research/Pieces/021 Property acquisition.md`):
//! * The housing family is base opcode 127. The client builds `ClientHousingPacketRequestGrant`
//!   with inner opcode 23 (`FUN_00aa4fd0`, `FUN_00aa4990(0x17)`) and sends it once per UI state
//!   from `FUN_00ac7af0`; it builds `HousingPacketNotifyHouseAdded` with inner opcode 49
//!   (`FUN_00aa8700`, `FUN_00aa4990(0x31)`) and prints the `HousingNotifyNewHouse` key as
//!   "You have received a new %s." A request with no unlocked house prints the
//!   `HousingServerNoUnlockedHouses` key, so houses are unlocked per character.
//! * `Houses.json` carries 31 property rows; the client item definitions carry 36 `Type == 16`
//!   property deeds, 26 of which join a row by `NameId` (25 rows, one row has two deeds).
//! * The wiki lot pages record the Station Cash price pair (regular, member) and the grant
//!   channels: one free home for every new character (Apartment, replaced by the Wilds Condo on
//!   2012-01-26 with existing owners receiving it as well), the members-only Small Wilds House
//!   claim, the February 2014 membership-pack Sunset Party Boat and the TCG loot-card Rumbledome
//!   Lot. Membership is a 10% discount, matching the item `MemberDiscount` field.
//!
//! RECONSTRUCTED here: the channel names, the check order, the one-starter-home-per-character rule
//! and every error case. No original server code for a property purchase was recovered, and the
//! bodies of `ClientHousingPacketRequestGrant` (23) and `HousingPacketNotifyHouseAdded` (49) are
//! only partly recovered, so nothing here has been accepted by the native client.
//!
//! Deliberate boundaries:
//! * the local Station Cash map is a working copy; `StationCashDebit` is a proposal for the
//!   lead-owned atomic economy layer and this module never mints currency or items;
//! * no upkeep billing (`ClientHousingPacketPayUpkeep` = 7 is a known opcode with no recovered
//!   payload), no refunds, no sales tax, no Station Cash purchase (real money) and no house
//!   naming, visiting or permissions (items 022, 023);
//! * a property whose channels are empty stays known and refuses every channel instead of
//!   inventing one.
//!
//! ponytail: one property table, one ownership set and one balance map. No trait objects, no
//! packet writer, no persistence. Add a snapshot when the lead persists houses.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Schema version of the recovered catalogue document this module accepts.
pub const PROPERTY_CATALOGUE_SCHEMA_VERSION: u32 = 1;
/// ORIGINAL: membership is a 10% discount (wiki `Membership`, rev 48146), matching the item
/// `MemberDiscount` field and every recorded lot price pair.
pub const MEMBER_DISCOUNT_PERCENT: u32 = 10;

/// RECONSTRUCTED channel names. Each value is only attached to a property when the recovered
/// evidence states it: `Marketplace` needs a recorded Station Cash pair, `MembershipClaim` needs a
/// members-only deed, and the two free channels need a cited wiki grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionChannel {
    StarterGrant,
    Marketplace,
    MembershipClaim,
    BonusGrant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationCashPrice {
    pub regular: u64,
    pub member: u64,
}

/// One recovered property row: `Houses.json` identity plus the deed item(s) and the recorded
/// acquisition policy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyRow {
    pub property_id: i32,
    pub zone_id: i32,
    pub name_id: i32,
    pub name: String,
    #[serde(default)]
    pub deed_item_ids: Vec<i32>,
    #[serde(default)]
    pub members_only: bool,
    pub channels: Vec<AcquisitionChannel>,
    #[serde(default)]
    pub station_cash: Option<StationCashPrice>,
    #[serde(default)]
    pub evidence: BTreeMap<String, String>,
}

/// A proposal for the lead-owned economy layer. Applying it is not this module's job.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationCashDebit {
    pub character_guid: u64,
    pub amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionRequest {
    pub character_guid: u64,
    pub property_id: i32,
    pub channel: AcquisitionChannel,
    pub member: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionOutcome {
    pub character_guid: u64,
    pub property_id: i32,
    pub channel: AcquisitionChannel,
    pub charged: u64,
    pub debit: Option<StationCashDebit>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcquisitionError {
    InvalidOwner,
    UnknownProperty {
        property_id: i32,
    },
    AlreadyOwned {
        property_id: i32,
    },
    ChannelNotAllowed {
        property_id: i32,
        channel: AcquisitionChannel,
    },
    MembershipRequired {
        property_id: i32,
    },
    StarterGrantAlreadyUsed {
        character_guid: u64,
    },
    InsufficientStationCash {
        required: u64,
        available: u64,
    },
    UnsupportedSchema {
        version: u32,
    },
    CorruptCatalogue {
        reason: String,
    },
}

#[derive(Deserialize)]
struct CatalogueDocument {
    schema_version: u32,
    #[serde(default)]
    member_discount_percent: Option<u32>,
    properties: Vec<PropertyRow>,
}

/// The recovered property table plus the local ownership and balance state.
#[derive(Debug)]
pub struct PropertyAcquisition {
    schema_version: u32,
    properties: BTreeMap<i32, PropertyRow>,
    station_cash: BTreeMap<u64, u64>,
    owned: BTreeMap<u64, BTreeSet<i32>>,
    starter_granted: BTreeSet<u64>,
}

impl PropertyAcquisition {
    /// Parse and validate a recovered catalogue. A document that does not match the recovered
    /// rules is refused rather than trusted.
    pub fn from_json(bytes: &[u8]) -> Result<Self, AcquisitionError> {
        let document: CatalogueDocument =
            serde_json::from_slice(bytes).map_err(|error| AcquisitionError::CorruptCatalogue {
                reason: format!("unreadable catalogue: {error}"),
            })?;
        if document.schema_version != PROPERTY_CATALOGUE_SCHEMA_VERSION {
            return Err(AcquisitionError::UnsupportedSchema {
                version: document.schema_version,
            });
        }
        if let Some(discount) = document.member_discount_percent {
            if discount != MEMBER_DISCOUNT_PERCENT {
                return Err(AcquisitionError::CorruptCatalogue {
                    reason: format!("member discount {discount} is not the recovered 10%"),
                });
            }
        }
        if document.properties.is_empty() {
            return Err(AcquisitionError::CorruptCatalogue {
                reason: "catalogue carries no properties".to_string(),
            });
        }
        let mut properties = BTreeMap::new();
        let mut deeds: BTreeMap<i32, i32> = BTreeMap::new();
        for property in document.properties {
            validate_property(&property, &properties, &mut deeds)?;
            properties.insert(property.property_id, property);
        }
        Ok(Self {
            schema_version: document.schema_version,
            properties,
            station_cash: BTreeMap::new(),
            owned: BTreeMap::new(),
            starter_granted: BTreeSet::new(),
        })
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn property(&self, property_id: i32) -> Option<&PropertyRow> {
        self.properties.get(&property_id)
    }

    pub fn properties(&self) -> impl Iterator<Item = &PropertyRow> {
        self.properties.values()
    }

    /// Seed or replace the local working balance. The lead's economy layer stays authoritative.
    pub fn set_station_cash(&mut self, character_guid: u64, amount: u64) {
        self.station_cash.insert(character_guid, amount);
    }

    pub fn station_cash(&self, character_guid: u64) -> u64 {
        self.station_cash.get(&character_guid).copied().unwrap_or(0)
    }

    pub fn owns(&self, character_guid: u64, property_id: i32) -> bool {
        self.owned
            .get(&character_guid)
            .is_some_and(|properties| properties.contains(&property_id))
    }

    /// The recovered starter grant is one free home per character; the 2012-01-26 Wilds Condo
    /// migration is a separate `BonusGrant`.
    pub fn starter_grant_used(&self, character_guid: u64) -> bool {
        self.starter_granted.contains(&character_guid)
    }

    /// Acquire one property through one channel.
    ///
    /// Every check runs before the first mutation, so a refused acquisition leaves ownership, the
    /// starter flag and the balance exactly as they were.
    pub fn acquire(
        &mut self,
        request: AcquisitionRequest,
    ) -> Result<AcquisitionOutcome, AcquisitionError> {
        if request.character_guid == 0 {
            return Err(AcquisitionError::InvalidOwner);
        }
        let property =
            self.properties
                .get(&request.property_id)
                .ok_or(AcquisitionError::UnknownProperty {
                    property_id: request.property_id,
                })?;
        if self.owns(request.character_guid, request.property_id) {
            return Err(AcquisitionError::AlreadyOwned {
                property_id: request.property_id,
            });
        }
        if !property.channels.contains(&request.channel) {
            return Err(AcquisitionError::ChannelNotAllowed {
                property_id: request.property_id,
                channel: request.channel,
            });
        }
        if request.channel == AcquisitionChannel::MembershipClaim && !property.members_only {
            return Err(AcquisitionError::CorruptCatalogue {
                reason: format!(
                    "property {} claims membership without a members-only deed",
                    property.name
                ),
            });
        }
        if request.channel == AcquisitionChannel::MembershipClaim && !request.member {
            return Err(AcquisitionError::MembershipRequired {
                property_id: request.property_id,
            });
        }
        if request.channel == AcquisitionChannel::StarterGrant
            && self.starter_granted.contains(&request.character_guid)
        {
            return Err(AcquisitionError::StarterGrantAlreadyUsed {
                character_guid: request.character_guid,
            });
        }
        let charge = match request.channel {
            AcquisitionChannel::Marketplace => {
                let price =
                    property
                        .station_cash
                        .ok_or_else(|| AcquisitionError::CorruptCatalogue {
                            reason: format!(
                                "property {} offers the marketplace without a recorded price",
                                property.name
                            ),
                        })?;
                if request.member {
                    price.member
                } else {
                    price.regular
                }
            }
            AcquisitionChannel::StarterGrant
            | AcquisitionChannel::MembershipClaim
            | AcquisitionChannel::BonusGrant => 0,
        };
        let available = self.station_cash(request.character_guid);
        if charge > available {
            return Err(AcquisitionError::InsufficientStationCash {
                required: charge,
                available,
            });
        }

        // Past every check: commit ownership, the starter flag and the balance together.
        if charge > 0 {
            self.station_cash
                .insert(request.character_guid, available - charge);
        }
        if request.channel == AcquisitionChannel::StarterGrant {
            self.starter_granted.insert(request.character_guid);
        }
        self.owned
            .entry(request.character_guid)
            .or_default()
            .insert(request.property_id);
        Ok(AcquisitionOutcome {
            character_guid: request.character_guid,
            property_id: request.property_id,
            channel: request.channel,
            charged: charge,
            debit: (charge > 0).then_some(StationCashDebit {
                character_guid: request.character_guid,
                amount: charge,
            }),
        })
    }
}

fn validate_property(
    property: &PropertyRow,
    existing: &BTreeMap<i32, PropertyRow>,
    deeds: &mut BTreeMap<i32, i32>,
) -> Result<(), AcquisitionError> {
    let corrupt = |reason: String| AcquisitionError::CorruptCatalogue { reason };
    if property.property_id <= 0 || property.zone_id <= 0 || property.name_id <= 0 {
        return Err(corrupt(format!(
            "property {} has a non-positive id, zone id or name id",
            property.property_id
        )));
    }
    if property.name.is_empty() {
        return Err(corrupt(format!(
            "property {} has no name",
            property.property_id
        )));
    }
    if existing.contains_key(&property.property_id) {
        return Err(corrupt(format!(
            "duplicate property id {}",
            property.property_id
        )));
    }
    let mut channels = BTreeSet::new();
    for channel in &property.channels {
        if !channels.insert(*channel) {
            return Err(corrupt(format!(
                "property {} repeats a channel",
                property.name
            )));
        }
    }
    if channels.contains(&AcquisitionChannel::Marketplace) && property.station_cash.is_none() {
        return Err(corrupt(format!(
            "property {} offers the marketplace without a recorded price",
            property.name
        )));
    }
    if let Some(price) = property.station_cash {
        let expected = price.regular * (100 - MEMBER_DISCOUNT_PERCENT as u64) / 100;
        if price.regular == 0 || price.member != expected {
            return Err(corrupt(format!(
                "property {} member price {} is not the recovered discount of {}",
                property.name, price.member, price.regular
            )));
        }
    }
    if channels.contains(&AcquisitionChannel::MembershipClaim) && !property.members_only {
        return Err(corrupt(format!(
            "property {} claims membership without a members-only deed",
            property.name
        )));
    }
    let mut local = BTreeSet::new();
    for item_id in &property.deed_item_ids {
        if *item_id <= 0 {
            return Err(corrupt(format!(
                "property {} has an invalid deed item",
                property.name
            )));
        }
        if !local.insert(*item_id) {
            return Err(corrupt(format!(
                "property {} repeats deed item {item_id}",
                property.name
            )));
        }
        if let Some(other) = deeds.insert(*item_id, property.property_id) {
            return Err(corrupt(format!(
                "deed item {item_id} also unlocks property {other}"
            )));
        }
    }
    Ok(())
}
