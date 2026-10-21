//! Housing integration service: the callable seam between the recovered native housing requests,
//! the restored `housing` state machine and the live character `Inventory`.
//!
//! Client facts routed here are documented in `notes/Build Guides/Gameplay/Housing.md` and
//! `notes/Reviews/Housing Placement Selection Identity Review.md`:
//! * housing is packet family 127; the client-to-server inner opcodes include 1
//!   (`PlaceFixtureRequest`), 2 (`PlaceFixture`), 3 (`PickupFixture`), 5 (`SaveFixture`), 6
//!   (`SetEditMode`), 19 (`EnterRequest`) and 20 (`PreviewByItemDefinitionIdRequest`);
//! * the `127/2` first dword is the client's placement *selection id*: it is neither the item
//!   definition nor an inventory instance identity, so a placement needs a server-side binding;
//! * client readers expose fixture and item definition ids in `127/40` records. This does not
//!   establish how the original server authorized or paired them.
//!
//! RECONSTRUCTED here (R): the selection-id binding, fixture configuration table, wire-transform
//! projection, all server reply pairings, and numeric/text policy. The guide identifies client
//! senders/readers but has no original server implementation or recorded-session evidence. No
//! native client has exchanged a housing packet with this service.
//!
//! Ownership split (see `notes/Research/Housing Service Integration.md`):
//! * this module never writes a file; `HousingServiceState` is serialized by the lead into the one
//!   canonical `game_store` document next to the existing `InventoryState`;
//! * the live `Inventory` stays the single record of what the character owns. The service derives
//!   its placement ledger from that inventory plus the lot it is editing, and never keeps a second
//!   ledger of its own;
//! * a request is validated and staged on a cloned `Housing`; the real inventory is mutated last,
//!   and only its success commits the staged lot. `Inventory::consume`/`grant_with_tint` are
//!   internally atomic, so a failure there leaves both sides exactly as they were.
//!
//! ponytail: one request decoder, one staged commit path per verb and one configuration table. No
//! packet loop, no second ledger, no new dependency.

use crate::housing::{
    FixtureInstance, FixtureItemLedger, FixtureTransform, Housing, HousingError, LotSnapshot,
    PlacedFixture, PropertyDefinition, PropertyGrant, SavedFixture, StagedChange,
};
use crate::housing_native::{
    CouplingDisplay, Customization, FixtureAsset, FixtureDefinition,
    FixtureInstance as WireFixtureInstance, FixtureInstanceInfo, FixtureItemList, FixtureRemove,
    FixtureUpdate, HouseInfo, HousePermissionRecord, InstanceData, InstanceList,
    PlaceFixture as WirePlaceFixture, PlacementDenied, PlayerHousingInstanceInfo, PositionUpdate,
    ZoneData,
};
use crate::inventory::{Inventory, InventoryItem, InventoryState};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
};

// The recovered client writers live in `src/restoration/piece_025.rs`, which is not registered in
// `lib.rs`, so the transcription is reused through `#[path]` (the same pattern `inventory.rs` uses
// for `piece_081`). When the lead registers the restoration modules, replace this include with
// `crate::restoration::piece_025`.
#[path = "restoration/piece_025.rs"]
#[allow(dead_code)]
mod wire;

/// Base opcode of the whole housing family (`BaseHousingPacket`).
pub const BASE_OPCODE: u16 = 127;
/// Byte length of the recovered housing header: base opcode then inner opcode.
pub const HEADER_LEN: usize = 4;

pub const OP_PLACE_REQUEST: u16 = 1;
pub const OP_PLACE: u16 = 2;
pub const OP_PICKUP: u16 = 3;
pub const OP_PICKUP_ALL: u16 = 4;
pub const OP_SAVE: u16 = 5;
pub const OP_SET_EDIT_MODE: u16 = 6;
pub const OP_LEAVE: u16 = 10;
/// `RequestPlayerHouses` (`00ac3f40`): `u64` owner guid, then a length-prefixed string.
pub const OP_REQUEST_HOUSES: u16 = 14;
pub const OP_ENTER: u16 = 19;
pub const OP_PREVIEW: u16 = 20;
/// `RequestGrant` (`00ac7af0`, latched once per client session): empty body.
pub const OP_REQUEST_GRANT: u16 = 23;

pub const REPLY_INSTANCE_DATA: u16 = 28;
pub const REPLY_INSTANCE_LIST: u16 = 39;
pub const REPLY_FIXTURE_UPDATE: u16 = 40;
pub const REPLY_FIXTURE_REMOVE: u16 = 41;
pub const REPLY_PLACE_REQUEST: u16 = 42;
pub const REPLY_ITEM_LIST: u16 = 43;
pub const REPLY_HOUSE_INFO: u16 = 44;
pub const REPLY_ZONE_DATA: u16 = 45;
pub const REPLY_POSITION: u16 = 51;
pub const REPLY_PLACEMENT_DENIED: u16 = 53;

/// Schema version of the persisted service state. Bump it when `HousingServiceState` changes.
pub const SERVICE_SCHEMA_VERSION: u32 = 1;
/// RECONSTRUCTED bound on the lots one character snapshot may carry.
pub const MAX_STATE_LOTS: usize = 64;
/// RECONSTRUCTED bound on the placed fixtures one character snapshot may carry.
pub const MAX_STATE_FIXTURES: usize = 4_096;
/// RECONSTRUCTED bound for one housing content document.
const MAX_HOUSING_CONTENT_BYTES: usize = 1024 * 1024;
/// RECONSTRUCTED bound on one fixture-configuration table.
const MAX_CONFIG_ROWS: usize = 4_096;

/// (R) Placeholder for the `127/44` field whose retail server value is not recovered.
const RECONSTRUCTED_HOUSE_UNUSED: i32 = 10;
/// (R) Default flora policy; the retail server value is not recovered.
const RECONSTRUCTED_FLORA_ALLOWED: bool = true;
/// (R) Identity tilt because the client `127/2` body has no separate tilt field.
const IDENTITY_TILT: [f32; 4] = [1.0, 0.0, 0.0, 0.0];

/// Client `127/2` body projection, without the family prefix: an uninterpreted selection dword, two
/// 16-byte transform groups and a final dword. Mapping these groups to this server transform is R.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WireTransform {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: f32,
}

/// Scene configuration used by the existing zone-transition coordinator. Scene-to-lot mappings
/// are not present in the shipped housing resources; populated mappings must cite their source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousingScene {
    pub name: String,
    pub zone_type: u8,
    pub sky: String,
    pub tutorial: bool,
    pub unknown: u32,
    pub zone_id: u32,
    pub update_radius: f32,
    pub geometry_id: u32,
    pub override_update_radius: bool,
    pub wait_for_zone_ready: bool,
}

/// Source-tagged lot definition. Fixture limits are server policy (R) because the client receives
/// them from the server and ships no per-lot cap table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousingLotDefinition {
    pub property_id: i32,
    pub zone_id: i32,
    pub name_id: i32,
    pub icon_id: i32,
    pub max_fixtures: u32,
    pub max_landmarks: u32,
    pub build_areas: Vec<[f32; 6]>,
    pub spawn_position: [f32; 4],
    pub spawn_rotation: [f32; 4],
    pub scene: Option<HousingScene>,
    /// (R) The client exposes no property-to-scene association.
    pub scene_source: Option<String>,
    /// (R) Legacy bounds are provisional, not recovered client build areas.
    pub bounds_source: String,
    /// (R) Why this row has no safe community scene-name candidate.
    pub disabled_reason: Option<String>,
    pub source_tag: String,
    pub enabled: bool,
}

impl HousingLotDefinition {
    fn property(&self) -> PropertyDefinition {
        PropertyDefinition {
            property_id: self.property_id,
            zone_id: self.zone_id,
            name_id: self.name_id,
            max_fixtures: self.max_fixtures,
            max_landmarks: self.max_landmarks,
            build_areas: self.build_areas.clone(),
        }
    }

    fn validate(&self) -> Result<(), ServiceError> {
        if self.source_tag.trim().is_empty()
            || self.bounds_source.trim().is_empty()
            || self.icon_id <= 0
            || self.spawn_position.iter().any(|value| !value.is_finite())
            || self.spawn_rotation.iter().any(|value| !value.is_finite())
            || self
                .spawn_position
                .iter()
                .chain(&self.spawn_rotation)
                .any(|value| value.abs() > 1_000_000.0)
            || self.scene.as_ref().is_some_and(|scene| {
                scene.name.is_empty()
                    || scene.name.len() > 256
                    || scene.sky.len() > 256
                    || !scene.update_radius.is_finite()
                    || scene.update_radius < 0.0
            })
            || (self.enabled
                && (self.scene.is_none()
                    || self
                        .scene_source
                        .as_ref()
                        .map_or(true, |source| source.trim().is_empty())
                    || self.disabled_reason.is_some()))
            || (!self.enabled
                && (self.scene.is_some()
                    || self.scene_source.is_some()
                    || self
                        .disabled_reason
                        .as_ref()
                        .map_or(true, |reason| reason.trim().is_empty())))
        {
            return Err(ServiceError::HousingContentInvalid(
                "invalid or untagged lot row",
            ));
        }
        self.property().validate().map_err(ServiceError::Housing)
    }
}

/// One ordinary scene transition emitted before housing-family replies. Scene transition wire
/// encoding belongs to the lead-owned zone coordinator.
#[derive(Clone, Debug, PartialEq)]
pub enum HousingZoneChange {
    EnterHouse {
        actor_guid: u64,
        house_guid: u64,
        scene: HousingScene,
        spawn_position: [f32; 4],
        spawn_rotation: [f32; 4],
    },
    /// The existing zone coordinator owns the prior world location and starts the return trip.
    ReturnToPreviousZone { actor_guid: u64, house_guid: u64 },
}

/// Content document for lot rows. `client_evidence` stores the client resource facts that anchor
/// names and scene families; each house row separately names its source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousingContent {
    pub schema_version: u32,
    pub provenance: String,
    pub starter_property_id: i32,
    pub client_evidence: Vec<String>,
    pub lots: Vec<HousingLotDefinition>,
}

impl HousingContent {
    pub fn bundled() -> Result<Self, ServiceError> {
        Self::parse(include_bytes!("../data/content/houses.json"))
    }

    pub fn load(path: &Path) -> Result<Self, ServiceError> {
        let file = File::open(path)
            .map_err(|_| ServiceError::HousingContentInvalid("cannot open housing content"))?;
        if !file
            .metadata()
            .map_err(|_| ServiceError::HousingContentInvalid("cannot inspect housing content"))?
            .is_file()
        {
            return Err(ServiceError::HousingContentInvalid(
                "housing content must be a file",
            ));
        }
        let mut bytes = Vec::new();
        file.take((MAX_HOUSING_CONTENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| ServiceError::HousingContentInvalid("cannot read housing content"))?;
        Self::parse(&bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, ServiceError> {
        if bytes.len() > MAX_HOUSING_CONTENT_BYTES {
            return Err(ServiceError::HousingContentInvalid(
                "housing content exceeds 1 MiB",
            ));
        }
        let content: Self = serde_json::from_slice(bytes)
            .map_err(|_| ServiceError::HousingContentInvalid("invalid housing content JSON"))?;
        content.validate()?;
        Ok(content)
    }

    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.schema_version != 1
            || self.provenance.trim().is_empty()
            || self.client_evidence.is_empty()
            || self
                .client_evidence
                .iter()
                .any(|fact| fact.trim().is_empty())
            || self.lots.is_empty()
            || self.lots.len() > MAX_STATE_LOTS
            || self.starter_property_id <= 0
        {
            return Err(ServiceError::HousingContentInvalid(
                "invalid housing content header",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut starter = false;
        for lot in &self.lots {
            lot.validate()?;
            if !ids.insert(lot.property_id) {
                return Err(ServiceError::HousingContentInvalid(
                    "duplicate lot property id",
                ));
            }
            starter |= lot.property_id == self.starter_property_id;
        }
        if !starter {
            return Err(ServiceError::HousingContentInvalid(
                "starter property id has no lot row",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestError {
    TooShort,
    WrongBaseOpcode {
        found: u16,
    },
    UnknownOpcode {
        opcode: u16,
    },
    Truncated,
    TrailingBytes {
        count: usize,
    },
    /// The body reached the housing codec but failed its own policy (length, finite floats).
    BodyRejected,
}

/// A decoded client-to-server housing request. For 127/1, only the first u32 is used as an opaque
/// selector; it must resolve through the reconstructed `offer` binding to a currently owned item.
/// The other 127/1 fields and 127/5 nested block remain opaque. The 127/20 item-definition field is
/// decoded, but its preview response is outside this service slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Request {
    Place {
        selection_id: u32,
        transform: WireTransform,
    },
    Pickup {
        fixture_guid: u64,
    },
    PickupAll,
    Move {
        fixture_guid: u64,
        transform: WireTransform,
    },
    SetEditMode {
        in_edit_mode: bool,
    },
    Leave,
    Enter {
        house_guid: u64,
    },
    PlaceFixtureRequest {
        selection_id: u32,
        second: u32,
        third: u32,
        flag: u8,
    },
    SaveFixture,
    PreviewByItemDefinition,
    /// 127/23: the client asks once per session whether it has a house to receive.
    RequestGrant,
    /// 127/14: list one player's houses (the trailing string is not used).
    RequestHouses {
        owner_guid: u64,
    },
}

/// Decode a complete inner game packet: base opcode 127, inner opcode, then the recovered body.
pub fn decode_request(inner: &[u8]) -> Result<Request, RequestError> {
    if inner.len() < HEADER_LEN {
        return Err(RequestError::TooShort);
    }
    let base = u16::from_le_bytes([inner[0], inner[1]]);
    if base != BASE_OPCODE {
        return Err(RequestError::WrongBaseOpcode { found: base });
    }
    let opcode = u16::from_le_bytes([inner[2], inner[3]]);
    if opcode == OP_REQUEST_GRANT {
        return match inner.len() - HEADER_LEN {
            0 => Ok(Request::RequestGrant),
            count => Err(RequestError::TrailingBytes { count }),
        };
    }
    if opcode == OP_REQUEST_HOUSES {
        // u64 owner guid, then a u32-length string (bounded, otherwise unused).
        let body = &inner[HEADER_LEN..];
        if body.len() < 12 {
            return Err(RequestError::Truncated);
        }
        let text_len = u32::from_le_bytes(body[8..12].try_into().unwrap()) as usize;
        if text_len > 4096 {
            return Err(RequestError::BodyRejected);
        }
        if body.len() != 12 + text_len {
            return Err(if body.len() < 12 + text_len {
                RequestError::Truncated
            } else {
                RequestError::TrailingBytes {
                    count: body.len() - 12 - text_len,
                }
            });
        }
        return Ok(Request::RequestHouses {
            owner_guid: u64::from_le_bytes(body[..8].try_into().unwrap()),
        });
    }
    if opcode == OP_PICKUP_ALL || opcode == OP_LEAVE {
        if inner.len() != HEADER_LEN {
            return Err(RequestError::TrailingBytes {
                count: inner.len().saturating_sub(HEADER_LEN),
            });
        }
        return Ok(if opcode == OP_PICKUP_ALL {
            Request::PickupAll
        } else {
            Request::Leave
        });
    }
    if opcode == OP_PLACE {
        // The codec starts at the subtype, so the family prefix is excluded here.
        let place =
            WirePlaceFixture::decode(&inner[2..]).map_err(|_| RequestError::BodyRejected)?;
        return Ok(Request::Place {
            selection_id: place.first_dword,
            transform: WireTransform {
                position: place.position,
                rotation: place.rotation,
                scale: place.scale,
            },
        });
    }
    if opcode == OP_SAVE {
        // Client writer FUN_00abf620 emits the fixture key, two four-float groups, scale, then
        // an opaque nested block. Only the prefix used for transform validation is interpreted.
        let body = &inner[HEADER_LEN..];
        if body.len() < 44 || body.len() > 64 * 1024 {
            return Err(RequestError::BodyRejected);
        }
        let fixture_guid = u64::from_le_bytes(body[..8].try_into().unwrap());
        if fixture_guid == 0 {
            return Err(RequestError::BodyRejected);
        }
        let read_float4 = |bytes: &[u8]| -> [f32; 4] {
            std::array::from_fn(|index| {
                f32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
            })
        };
        let position = read_float4(&body[8..24]);
        let rotation = read_float4(&body[24..40]);
        let scale = f32::from_le_bytes(body[40..44].try_into().unwrap());
        return Ok(Request::Move {
            fixture_guid,
            transform: WireTransform {
                position,
                rotation,
                scale,
            },
        });
    }
    match wire::decode(inner).map_err(RequestError::from)? {
        wire::HousingWireMessage::PlaceFixtureRequest {
            first,
            second,
            third,
            flag,
        } => Ok(Request::PlaceFixtureRequest {
            selection_id: first,
            second,
            third,
            flag,
        }),
        wire::HousingWireMessage::PickupFixture { fixture_guid } => {
            Ok(Request::Pickup { fixture_guid })
        }
        wire::HousingWireMessage::SaveFixture(_) => Ok(Request::SaveFixture),
        wire::HousingWireMessage::SetEditMode { in_edit_mode } => {
            Ok(Request::SetEditMode { in_edit_mode })
        }
        wire::HousingWireMessage::EnterRequest { house_guid, .. } => {
            Ok(Request::Enter { house_guid })
        }
        wire::HousingWireMessage::PreviewByItemDefinitionIdRequest { .. } => {
            Ok(Request::PreviewByItemDefinition)
        }
    }
}

impl From<wire::WireError> for RequestError {
    fn from(error: wire::WireError) -> Self {
        match error {
            wire::WireError::TooShort => Self::TooShort,
            wire::WireError::WrongBaseOpcode { found } => Self::WrongBaseOpcode { found },
            wire::WireError::UnknownOpcode { opcode } => Self::UnknownOpcode { opcode },
            wire::WireError::Truncated => Self::Truncated,
            wire::WireError::TrailingBytes { count } => Self::TrailingBytes { count },
        }
    }
}

/// One real furniture instance the live inventory owns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedInstance {
    pub instance_id: u32,
    pub definition_id: i32,
    pub tint_id: i32,
    pub count: u32,
}

/// The live-inventory seam. `Inventory` implements it below; tests and the gateway can supply the
/// same shape without private content. Implementations must apply a mutation atomically: a
/// `consume`/`return_instance` that returns `Err` must leave the stock unchanged.
pub trait FurnitureStock {
    fn instances(&self) -> Vec<OwnedInstance>;
    fn consume(&mut self, instance_id: u32, quantity: u32) -> Result<(), ServiceError>;
    fn return_instance(&mut self, definition_id: i32, tint_id: i32) -> Result<(), ServiceError>;
    /// Apply all returns atomically. Implementations stage their own clone before committing.
    fn return_instances(&mut self, items: &[(i32, i32)]) -> Result<(), ServiceError>;
}

impl FurnitureStock for Inventory {
    fn instances(&self) -> Vec<OwnedInstance> {
        self.state()
            .items
            .iter()
            .filter_map(owned_instance)
            .collect()
    }

    fn consume(&mut self, instance_id: u32, quantity: u32) -> Result<(), ServiceError> {
        let id = i32::try_from(instance_id)
            .map_err(|_| ServiceError::UnknownInstance { instance_id })?;
        self.consume(id, quantity)
            .map_err(|_| ServiceError::StockRejected {
                operation: StockOperation::Consume,
                instance_id,
            })?;
        Ok(())
    }

    fn return_instance(&mut self, definition_id: i32, tint_id: i32) -> Result<(), ServiceError> {
        // Inventory hands a returned unit a fresh instance id; the placed fixture's id is released
        // with the fixture itself, so conservation holds per definition and tint, not per id.
        self.grant_with_tint(0, definition_id, 1, tint_id)
            .map_err(|_| ServiceError::StockRejected {
                operation: StockOperation::Return,
                instance_id: 0,
            })?;
        Ok(())
    }

    fn return_instances(&mut self, items: &[(i32, i32)]) -> Result<(), ServiceError> {
        let mut staged = self.clone();
        for &(definition_id, tint_id) in items {
            staged.return_instance(definition_id, tint_id)?;
        }
        *self = staged;
        Ok(())
    }
}

fn owned_instance(item: &InventoryItem) -> Option<OwnedInstance> {
    if item.count == 0 || item.tint < 0 {
        return None;
    }
    Some(OwnedInstance {
        instance_id: u32::try_from(item.id).ok()?,
        definition_id: item.definition_id,
        tint_id: item.tint,
        count: item.count,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockOperation {
    Consume,
    Return,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceError {
    /// The datagram is not a recovered housing request this build can route.
    MalformedRequest(RequestError),
    /// A recovered request whose remaining fields are not decoded; `missing` names them.
    UnresolvedRequest {
        opcode: u16,
        missing: &'static [&'static str],
    },
    /// The `127/2` selection dword has no server-side binding for this actor.
    UnresolvedSelection {
        selection_id: u32,
    },
    /// The actor has no successfully entered lot in this session.
    NotInHouse {
        actor_guid: u64,
    },
    /// The live inventory does not hold that instance.
    UnknownInstance {
        instance_id: u32,
    },
    /// A furniture stack of more than one unit cannot back one placed fixture.
    StackedInstanceUnsupported {
        instance_id: u32,
        count: u32,
    },
    /// No fixture-configuration row covers the definition id looked up.
    FixtureDefinitionUnknown {
        definition_id: i32,
    },
    /// More than one fixture-configuration row claims the item definition.
    FixtureDefinitionAmbiguous {
        definition_id: i32,
    },
    /// The fixture-configuration table itself is not usable.
    FixtureConfigInvalid(&'static str),
    HousingContentInvalid(&'static str),
    HouseSceneUnavailable {
        property_id: i32,
    },
    /// RECONSTRUCTED server authorization: edit requests require the session's confirmed mode.
    EditModeRequired,
    /// The live inventory refused the mutation; nothing was committed.
    StockRejected {
        operation: StockOperation,
        instance_id: u32,
    },
    /// The restored housing state machine refused the request.
    Housing(HousingError),
}

impl From<HousingError> for ServiceError {
    fn from(error: HousingError) -> Self {
        Self::Housing(error)
    }
}

impl ServiceError {
    /// A bounded single-line reason for the recovered `127/53` body. Fixed literals only, so the
    /// message always fits the codec's string bound.
    pub fn denial_message(&self) -> String {
        let text = match self {
            Self::Housing(
                HousingError::NotPropertyOwner
                | HousingError::InvalidOwner
                | HousingError::NotPermitted,
            ) => "Only the lot owner can edit this lot.",
            Self::Housing(HousingError::LotLocked) => "This lot is locked.",
            Self::Housing(HousingError::FixtureLimitReached) => "Fixture limit reached.",
            Self::Housing(HousingError::OutsideBuildArea) => {
                "That position is outside the build area."
            }
            Self::Housing(HousingError::InstanceAlreadyPlaced { .. }) => {
                "That item is already placed."
            }
            Self::Housing(HousingError::UnknownFixture) => "That fixture is no longer on this lot.",
            Self::Housing(
                HousingError::InvalidPlacement
                | HousingError::NonFiniteTransform
                | HousingError::TransformOutOfBounds
                | HousingError::NonUnitRotation,
            ) => "That placement is not valid.",
            Self::UnknownInstance { .. } => "That item is not in this character's inventory.",
            Self::StackedInstanceUnsupported { .. } => "That item cannot be placed from a stack.",
            Self::UnresolvedSelection { .. } => "That fixture selection is no longer available.",
            Self::FixtureDefinitionUnknown { .. }
            | Self::FixtureDefinitionAmbiguous { .. }
            | Self::FixtureConfigInvalid(_) => "That item has no placeable fixture definition.",
            Self::HousingContentInvalid(_) | Self::HouseSceneUnavailable { .. } => {
                "That house is unavailable."
            }
            Self::EditModeRequired => "Enter edit mode before changing fixtures.",
            Self::NotInHouse { .. } => "Enter a lot before editing it.",
            Self::StockRejected { .. } => "The inventory could not apply that change.",
            Self::MalformedRequest(_) | Self::UnresolvedRequest { .. } | Self::Housing(_) => {
                "Fixture placement denied."
            }
        };
        text.to_string()
    }

    /// The recovered `127/53` reply for a denied request. The client reader `FUN_00ab3030` reads
    /// exactly one length-prefixed string and falls back to `HousingMaxFixtures` when it is empty,
    /// so a denial always carries a reason.
    pub fn denial_reply(&self) -> HousingReply {
        HousingReply::PlacementDenied(PlacementDenied {
            message: self.denial_message(),
        })
    }
}

/// One typed server-to-client housing reply. `Unrecovered` marks a family opcode whose body this
/// build cannot encode, so a caller can never mistake a marker for a serialized packet.
#[derive(Clone, Debug, PartialEq)]
pub enum HousingReply {
    ZoneData(ZoneData),
    InstanceData(InstanceData),
    FixtureUpdate(FixtureUpdate),
    FixtureRemove(FixtureRemove),
    FixtureAsset(FixtureAsset),
    FixtureItemList(FixtureItemList),
    HouseInfo(HouseInfo),
    PositionUpdate(PositionUpdate),
    PlacementDenied(PlacementDenied),
    InstanceList(InstanceList),
    Unrecovered { opcode: u16 },
}

impl HousingReply {
    pub fn opcode(&self) -> u16 {
        match self {
            Self::ZoneData(_) => REPLY_ZONE_DATA,
            Self::InstanceData(_) => REPLY_INSTANCE_DATA,
            Self::FixtureUpdate(_) => REPLY_FIXTURE_UPDATE,
            Self::FixtureRemove(_) => REPLY_FIXTURE_REMOVE,
            Self::FixtureAsset(_) => REPLY_PLACE_REQUEST,
            Self::FixtureItemList(_) => REPLY_ITEM_LIST,
            Self::HouseInfo(_) => REPLY_HOUSE_INFO,
            Self::PositionUpdate(_) => REPLY_POSITION,
            Self::PlacementDenied(_) => REPLY_PLACEMENT_DENIED,
            Self::InstanceList(_) => REPLY_INSTANCE_LIST,
            Self::Unrecovered { opcode } => *opcode,
        }
    }

    /// Encode the subtype-level body, exactly like `housing_native` typed methods: subtype `u16`
    /// then fields, with the family opcode excluded.
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::ZoneData(value) => value.encode(),
            Self::InstanceData(value) => value.encode(),
            Self::FixtureUpdate(value) => value.encode(),
            Self::FixtureRemove(value) => value.encode(),
            Self::FixtureAsset(value) => value.encode(),
            Self::FixtureItemList(value) => value.encode(),
            Self::HouseInfo(value) => value.encode(),
            Self::PositionUpdate(value) => value.encode(),
            Self::PlacementDenied(value) => value.encode(),
            Self::InstanceList(value) => value.encode(),
            Self::Unrecovered { opcode } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("housing reply {opcode} body is not recovered"),
            )),
        }
    }

    /// Encode the complete inner game packet: one family prefix, then the body.
    pub fn encode_inner(&self) -> io::Result<Vec<u8>> {
        let body = self.encode()?;
        let mut inner = Vec::with_capacity(body.len() + 2);
        inner.extend_from_slice(&BASE_OPCODE.to_le_bytes());
        inner.extend_from_slice(&body);
        Ok(inner)
    }
}

/// The reply set and change flag for one request. `changed` means the caller must persist
/// `HousingService::state()` again; a denied or unrecovered request never sets it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServiceOutcome {
    /// An outer scene change that the gateway must begin before sending `replies`.
    pub zone_change: Option<HousingZoneChange>,
    pub replies: Vec<HousingReply>,
    pub changed: bool,
}

impl ServiceOutcome {
    fn unrecovered(opcode: u16) -> Self {
        Self {
            zone_change: None,
            replies: vec![HousingReply::Unrecovered { opcode }],
            changed: false,
        }
    }

    /// The typed refusal outcome for a request the caller must answer: one recovered `127/53`
    /// body with a bounded reason. Nothing changed, so the caller does not re-persist.
    pub fn denied(error: &ServiceError) -> Self {
        Self {
            zone_change: None,
            replies: vec![error.denial_reply()],
            changed: false,
        }
    }
}

/// The fields the recovered `127/5` body still does not identify.
const SAVE_MISSING: &[&str] = &[
    "placed fixture identity",
    "quad_a/quad_b interpretation",
    "opaque FUN_00abd1d0 block",
];

/// RECONSTRUCTED programmatic stand-in for the unrecovered retail fixture-definition table: which
/// item definition places as which fixture definition and model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureConfigRow {
    pub fixture_definition_id: i32,
    pub item_definition_id: i32,
    pub model_id: i32,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub lua_call: String,
    /// Provenance. The bundled fallback is reconstructed from an older packet-derived artifact;
    /// it is not client-table evidence or verified retail fixture content.
    pub provenance: String,
}

/// The fixture configuration table. It is content, not save state: the lead may load it from a
/// package `content/` file later, which is why it is serde-enabled.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureConfig {
    pub schema_version: u32,
    pub provenance: String,
    pub rows: Vec<FixtureConfigRow>,
}

impl FixtureConfig {
    /// Historical reconstructed fixture rows used as test defaults. The current client guide does
    /// not validate these tuples; flags, category, Lua calls, actions and effects remain unknown.
    pub fn capture_defaults() -> Self {
        let row = |fixture_definition_id: i32, item_definition_id: i32, model_id: i32| {
            FixtureConfigRow {
                fixture_definition_id,
                item_definition_id,
                model_id,
                category: String::new(),
                lua_call: String::new(),
                provenance: format!(
                    "R:unverified-legacy-packet-derived-artifact; lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap 127/40 record: \
                     fixture definition {fixture_definition_id}, item definition \
                     {item_definition_id}, model {model_id}"
                ),
            }
        };
        Self {
            schema_version: 1,
            provenance: "R: unverified packet-derived fixture rows; the retail server-side \
                         fixture definition and action table is unrecovered (work item 029)"
                .to_string(),
            rows: vec![
                row(34, 7_466, 1_926),
                row(902, 14_300, 2_159),
                row(490, 77_259, 1_988),
                row(323, 7_462, 1_945),
                row(68, 16_039, 3_002),
                row(1_074, 18_110, 3_058),
                row(69, 16_040, 3_003),
            ],
        }
    }

    /// The row that places this item definition. Fails closed when the table has no row, or more
    /// than one row (an ambiguous item would let content pick a fixture at random).
    pub fn row_for_item_definition(
        &self,
        item_definition_id: i32,
    ) -> Result<&FixtureConfigRow, ServiceError> {
        let mut rows = self
            .rows
            .iter()
            .filter(|row| row.item_definition_id == item_definition_id);
        let first = rows.next().ok_or(ServiceError::FixtureDefinitionUnknown {
            definition_id: item_definition_id,
        })?;
        if rows.next().is_some() {
            return Err(ServiceError::FixtureDefinitionAmbiguous {
                definition_id: item_definition_id,
            });
        }
        Ok(first)
    }

    fn row_for_fixture_definition(
        &self,
        fixture_definition_id: i32,
    ) -> Result<&FixtureConfigRow, ServiceError> {
        let mut rows = self
            .rows
            .iter()
            .filter(|row| row.fixture_definition_id == fixture_definition_id);
        let first = rows.next().ok_or(ServiceError::FixtureDefinitionUnknown {
            definition_id: fixture_definition_id,
        })?;
        if rows.next().is_some() {
            return Err(ServiceError::FixtureDefinitionAmbiguous {
                definition_id: fixture_definition_id,
            });
        }
        Ok(first)
    }

    fn validate(&self) -> Result<(), ServiceError> {
        if self.schema_version != 1 {
            return Err(ServiceError::FixtureConfigInvalid(
                "unsupported fixture configuration schema",
            ));
        }
        if self.rows.is_empty() {
            return Err(ServiceError::FixtureConfigInvalid(
                "empty fixture configuration",
            ));
        }
        if self.rows.len() > MAX_CONFIG_ROWS {
            return Err(ServiceError::FixtureConfigInvalid(
                "fixture configuration is too large",
            ));
        }
        let mut fixture_ids = BTreeSet::new();
        if self.rows.iter().any(|row| {
            row.fixture_definition_id <= 0 || row.item_definition_id <= 0 || row.model_id <= 0
        }) {
            return Err(ServiceError::FixtureConfigInvalid(
                "invalid fixture configuration id",
            ));
        }
        if self
            .rows
            .iter()
            .any(|row| !fixture_ids.insert(row.fixture_definition_id))
        {
            return Err(ServiceError::FixtureConfigInvalid(
                "duplicate fixture definition id",
            ));
        }
        Ok(())
    }
}

/// Why a persisted `HousingServiceState` could not be restored. The document is rejected whole, so
/// a caller can never continue with half of another character's lot.
#[derive(Clone, Debug, PartialEq)]
pub enum StateError {
    SchemaVersion {
        found: u32,
    },
    TooManyLots {
        count: usize,
    },
    TooManyFixtures {
        count: usize,
    },
    /// The same inventory instance backs more than one placed fixture.
    PlacedInstanceDuplicated {
        instance_id: u32,
    },
    /// A placed fixture's instance is still in the backpack: the saved halves disagree.
    PlacedInstanceStillOwned {
        instance_id: u32,
    },
    /// The restored lots failed the housing state machine's own validation.
    Housing(HousingError),
    /// The supplied fixture configuration is not usable.
    Config(ServiceError),
}

/// The cloneable persisted state for one character. It holds no path and no inventory: the lead
/// serializes it beside the existing `InventoryState` in the one atomic snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HousingServiceState {
    pub schema_version: u32,
    pub lots: Vec<LotSnapshot>,
}

/// The housing service for one server process.
#[derive(Clone, Debug)]
pub struct HousingService {
    config: FixtureConfig,
    housing: Housing,
    /// Session state: the lot each actor last entered, and its last edit-mode request. Neither is
    /// persisted, because a reconnect re-enters.
    entered: BTreeMap<u64, u64>,
    edit_mode: BTreeMap<u64, bool>,
    /// RECONSTRUCTED binding from the client's placement selection dword to a real inventory
    /// instance, keyed by actor. The client guide documents the selection echo; its owning
    /// catalogue is unrecovered, so inventory authorization remains an explicit server binding.
    selections: BTreeMap<(u64, u32), u32>,
    house_definitions: BTreeMap<i32, HousingLotDefinition>,
    starter_property_id: Option<i32>,
}

impl HousingService {
    pub fn new(config: FixtureConfig) -> Self {
        Self {
            config,
            housing: Housing::new(),
            entered: BTreeMap::new(),
            edit_mode: BTreeMap::new(),
            selections: BTreeMap::new(),
            house_definitions: BTreeMap::new(),
            starter_property_id: None,
        }
    }

    pub fn with_content(
        config: FixtureConfig,
        content: HousingContent,
    ) -> Result<Self, ServiceError> {
        config.validate()?;
        content.validate()?;
        let mut service = Self::new(config);
        service.starter_property_id = Some(content.starter_property_id);
        for definition in content.lots {
            service.register_house_definition(definition)?;
        }
        Ok(service)
    }

    pub fn register_house_definition(
        &mut self,
        definition: HousingLotDefinition,
    ) -> Result<(), ServiceError> {
        definition.validate()?;
        if self
            .house_definitions
            .get(&definition.property_id)
            .is_some_and(|current| current != &definition)
        {
            return Err(ServiceError::HousingContentInvalid(
                "conflicting lot property definition",
            ));
        }
        let property = definition.property();
        self.housing.register_property(property);
        self.house_definitions
            .insert(definition.property_id, definition);
        Ok(())
    }

    /// RECONSTRUCTED policy: give each character the same one starter property type, idempotently.
    pub fn grant_starter_lot(&mut self, owner_guid: u64) -> Result<u64, ServiceError> {
        let property_id = self
            .starter_property_id
            .ok_or(ServiceError::HousingContentInvalid(
                "starter property is not configured",
            ))?;
        let definition = self.house_definitions.get(&property_id).cloned().ok_or(
            ServiceError::HousingContentInvalid("starter property row is missing"),
        )?;
        if let Some(existing) = self
            .state()
            .lots
            .into_iter()
            .find(|lot| lot.owner_guid == owner_guid && lot.property_id == property_id)
        {
            return Ok(existing.lot_guid);
        }
        let lot_guid = self.housing.lot_guid(owner_guid, property_id)?;
        self.housing.grant_property(PropertyGrant {
            owner_guid,
            property: definition.property(),
            lot_guid,
        })?;
        Ok(lot_guid)
    }

    /// Restore one character's persisted state. Bounded, whole-document validation: the schema
    /// version, both size caps, the housing state machine's own checks, cross-lot instance
    /// duplication and the inventory half of the conservation invariant.
    pub fn from_state(
        config: FixtureConfig,
        state: &HousingServiceState,
        inventory: &InventoryState,
    ) -> Result<Self, StateError> {
        config.validate().map_err(StateError::Config)?;
        if state.schema_version != SERVICE_SCHEMA_VERSION {
            return Err(StateError::SchemaVersion {
                found: state.schema_version,
            });
        }
        if state.lots.len() > MAX_STATE_LOTS {
            return Err(StateError::TooManyLots {
                count: state.lots.len(),
            });
        }
        let fixtures: usize = state.lots.iter().map(|lot| lot.fixtures.len()).sum();
        if fixtures > MAX_STATE_FIXTURES {
            return Err(StateError::TooManyFixtures { count: fixtures });
        }
        let mut housing = Housing::new();
        housing
            .restore_lots(state.lots.clone())
            .map_err(StateError::Housing)?;
        let owned: BTreeSet<u32> = inventory
            .items
            .iter()
            .filter_map(|item| u32::try_from(item.id).ok())
            .collect();
        let mut placed = BTreeSet::new();
        for lot in &state.lots {
            for fixture in &lot.fixtures {
                if !placed.insert(fixture.item_instance_id) {
                    return Err(StateError::PlacedInstanceDuplicated {
                        instance_id: fixture.item_instance_id,
                    });
                }
                if owned.contains(&fixture.item_instance_id) {
                    return Err(StateError::PlacedInstanceStillOwned {
                        instance_id: fixture.item_instance_id,
                    });
                }
            }
        }
        Ok(Self {
            config,
            housing,
            entered: BTreeMap::new(),
            edit_mode: BTreeMap::new(),
            selections: BTreeMap::new(),
            house_definitions: BTreeMap::new(),
            starter_property_id: None,
        })
    }

    pub fn from_state_with_content(
        config: FixtureConfig,
        content: HousingContent,
        state: &HousingServiceState,
        inventory: &InventoryState,
    ) -> Result<Self, StateError> {
        content.validate().map_err(StateError::Config)?;
        let mut service = Self::from_state(config, state, inventory)?;
        service.starter_property_id = Some(content.starter_property_id);
        for definition in content.lots {
            service
                .register_house_definition(definition)
                .map_err(StateError::Config)?;
        }
        Ok(service)
    }

    /// The persisted half of this service, for the lead's canonical snapshot.
    pub fn state(&self) -> HousingServiceState {
        HousingServiceState {
            schema_version: SERVICE_SCHEMA_VERSION,
            lots: self.housing.snapshot_lots(),
        }
    }

    pub fn config(&self) -> &FixtureConfig {
        &self.config
    }

    pub fn lot(&self, house_guid: u64) -> Result<LotSnapshot, ServiceError> {
        Ok(self.housing.saved_fixtures(house_guid)?)
    }

    pub fn register_property(&mut self, property: PropertyDefinition) {
        self.housing.register_property(property);
    }

    pub fn grant_property(&mut self, grant: PropertyGrant) -> Result<(), ServiceError> {
        if self.starter_property_id == Some(grant.property.property_id)
            && self.state().lots.iter().any(|lot| {
                lot.owner_guid == grant.owner_guid && lot.property_id == grant.property.property_id
            })
        {
            return Ok(());
        }
        Ok(self.housing.grant_property(grant)?)
    }

    pub fn set_locked(
        &mut self,
        actor_guid: u64,
        house_guid: u64,
        locked: bool,
    ) -> Result<(), ServiceError> {
        Ok(self.housing.set_locked(actor_guid, house_guid, locked)?)
    }

    pub fn set_permission(
        &mut self,
        owner_guid: u64,
        house_guid: u64,
        guid: u64,
        level: i32,
    ) -> Result<(), ServiceError> {
        Ok(self
            .housing
            .set_permission(owner_guid, house_guid, guid, level)?)
    }

    /// (R) Bind one placement selection dword to a real furniture instance the actor owns. The
    /// instance must exist in live inventory with a single unit and a configured fixture row.
    pub fn offer(
        &mut self,
        actor_guid: u64,
        selection_id: u32,
        item_instance_id: u32,
        stock: &dyn FurnitureStock,
    ) -> Result<(), ServiceError> {
        let house_guid = self.house_of(actor_guid)?;
        self.housing.can_enter(actor_guid, house_guid)?;
        self.require_edit_mode(actor_guid)?;
        if selection_id == 0 {
            return Err(ServiceError::HousingContentInvalid(
                "zero placement selection id",
            ));
        }
        let instance = stock
            .instances()
            .into_iter()
            .find(|instance| instance.instance_id == item_instance_id)
            .ok_or(ServiceError::UnknownInstance {
                instance_id: item_instance_id,
            })?;
        if instance.count != 1 {
            return Err(ServiceError::StackedInstanceUnsupported {
                instance_id: item_instance_id,
                count: instance.count,
            });
        }
        self.config
            .row_for_item_definition(instance.definition_id)?;
        self.selections
            .insert((actor_guid, selection_id), item_instance_id);
        Ok(())
    }

    /// Route one decoded native housing request. `inner` is the complete inner game packet: base
    /// opcode 127, inner opcode, then the recovered body.
    pub fn handle(
        &mut self,
        actor_guid: u64,
        inner: &[u8],
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        let request = decode_request(inner).map_err(ServiceError::MalformedRequest)?;
        match request {
            Request::Enter { house_guid } => self.enter(actor_guid, house_guid, stock),
            Request::Leave => self.leave(actor_guid),
            Request::RequestGrant => self.request_grant(actor_guid),
            Request::RequestHouses { owner_guid } => self.house_list(actor_guid, owner_guid),
            Request::Place {
                selection_id,
                transform,
            } => {
                // The session must be inside a lot before a selection id is even looked up: an
                // actor who never entered must not be able to probe another character's bindings.
                self.house_of(actor_guid)?;
                let instance_id = self
                    .selections
                    .get(&(actor_guid, selection_id))
                    .copied()
                    .ok_or(ServiceError::UnresolvedSelection { selection_id })?;
                self.place(actor_guid, instance_id, transform, stock)
            }
            Request::Pickup { fixture_guid } => self.pickup(actor_guid, fixture_guid, stock),
            Request::PickupAll => self.pickup_all(actor_guid, stock),
            Request::Move {
                fixture_guid,
                transform,
            } => {
                let house_guid = self.house_of(actor_guid)?;
                self.move_fixture(actor_guid, house_guid, fixture_guid, transform, stock)
            }
            Request::SetEditMode { in_edit_mode } => self.set_edit_mode(actor_guid, in_edit_mode),
            Request::PlaceFixtureRequest {
                selection_id,
                second: _,
                third: _,
                flag: _,
            } => self.fixture_asset(actor_guid, selection_id, stock),
            Request::PreviewByItemDefinition => {
                Ok(ServiceOutcome::unrecovered(REPLY_PLACE_REQUEST))
            }
            Request::SaveFixture => Err(ServiceError::UnresolvedRequest {
                opcode: OP_SAVE,
                missing: SAVE_MISSING,
            }),
        }
    }

    /// Gateway-facing placement path: authorization and unresolved selector failures become the
    /// recovered `127/53` string reply. Decode errors remain explicit for the outer router.
    pub fn handle_placement_for_client(
        &mut self,
        actor_guid: u64,
        inner: &[u8],
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        if !matches!(
            decode_request(inner).map_err(ServiceError::MalformedRequest)?,
            Request::Place { .. }
        ) {
            return self.handle(actor_guid, inner, stock);
        }
        match self.handle(actor_guid, inner, stock) {
            Ok(outcome) => Ok(outcome),
            Err(error) => Ok(ServiceOutcome::denied(&error)),
        }
    }

    /// The typed move command. The recovered `127/5` body carries no fixture identity and no
    /// proven position, so the wire path cannot call this yet; the gateway/lead supplies the
    /// resolved fixture guid and transform.
    pub fn move_fixture(
        &mut self,
        actor_guid: u64,
        house_guid: u64,
        fixture_guid: u64,
        transform: WireTransform,
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        self.housing.can_enter(actor_guid, house_guid)?;
        self.require_edit_mode(actor_guid)?;
        let lot = self.housing.saved_fixtures(house_guid)?;
        let placed = lot
            .fixtures
            .iter()
            .find(|fixture| fixture.fixture_guid == fixture_guid)
            .cloned()
            .ok_or(ServiceError::Housing(HousingError::UnknownFixture))?;
        let housing_transform =
            housing_transform(&transform, placed.transform.tint_id, placed.transform.tilt)?;
        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        staged.move_fixture(
            actor_guid,
            house_guid,
            fixture_guid,
            housing_transform,
            &mut ledger,
        )?;
        self.housing = staged;
        let moved = self
            .housing
            .saved_fixtures(house_guid)?
            .fixtures
            .into_iter()
            .find(|fixture| fixture.fixture_guid == fixture_guid)
            .ok_or(ServiceError::Housing(HousingError::UnknownFixture))?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![
                HousingReply::PositionUpdate(PositionUpdate {
                    fixture_guid,
                    position: [
                        moved.transform.position[0],
                        moved.transform.position[1],
                        moved.transform.position[2],
                        1.0,
                    ],
                    orientation: moved.transform.rotation,
                }),
                HousingReply::HouseInfo(
                    self.house_info(actor_guid, &self.housing.saved_fixtures(house_guid)?),
                ),
            ],
            changed: true,
        })
    }

    fn enter(
        &mut self,
        actor_guid: u64,
        house_guid: u64,
        stock: &dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        self.housing.can_visit(actor_guid, house_guid)?;
        let lot = self.housing.saved_fixtures(house_guid)?;
        let definition = self
            .house_definitions
            .get(&lot.property_id)
            .filter(|definition| definition.enabled)
            .ok_or(ServiceError::HouseSceneUnavailable {
                property_id: lot.property_id,
            })?;
        let scene = definition
            .scene
            .clone()
            .ok_or(ServiceError::HouseSceneUnavailable {
                property_id: lot.property_id,
            })?;
        let mut replies = vec![
            HousingReply::ZoneData(self.zone_data(&lot, definition)),
            HousingReply::InstanceData(self.instance_data(&lot, definition)?),
            HousingReply::HouseInfo(self.house_info(actor_guid, &lot)),
            HousingReply::FixtureItemList(self.fixture_item_list(stock)?),
        ];
        for saved in &lot.fixtures {
            let placed = placed_from_saved(saved);
            replies.push(HousingReply::FixtureUpdate(
                self.fixture_update(&placed, saved.fixture_definition_id)?,
            ));
        }
        self.entered.insert(actor_guid, house_guid);
        self.edit_mode.insert(actor_guid, false);
        self.selections.retain(|(owner, _), _| *owner != actor_guid);
        Ok(ServiceOutcome {
            zone_change: Some(HousingZoneChange::EnterHouse {
                actor_guid,
                house_guid,
                scene,
                spawn_position: definition.spawn_position,
                spawn_rotation: definition.spawn_rotation,
            }),
            replies,
            changed: false,
        })
    }

    /// 127/23 (R): the first ask grants the starter lot (idempotent: a repeat grants nothing) and
    /// answers with the owner's 127/39 list either way. 127/49 is not sent: no lot row carries
    /// the house-type text it prints.
    fn request_grant(&mut self, actor_guid: u64) -> Result<ServiceOutcome, ServiceError> {
        let before = self.lots_of(actor_guid).len();
        self.grant_starter_lot(actor_guid)?;
        let mut outcome = self.house_list(actor_guid, actor_guid)?;
        outcome.changed = self.lots_of(actor_guid).len() > before;
        Ok(outcome)
    }

    /// 127/14 and the tail of 127/23: one owner's houses that `actor_guid` may visit (R: a
    /// visitor sees only lots `can_visit` allows; the owner sees all of their own).
    fn house_list(
        &self,
        actor_guid: u64,
        owner_guid: u64,
    ) -> Result<ServiceOutcome, ServiceError> {
        let records = self
            .lots_of(owner_guid)
            .into_iter()
            .filter(|lot| self.housing.can_visit(actor_guid, lot.lot_guid).is_ok())
            .filter_map(|lot| {
                let definition = self.house_definitions.get(&lot.property_id)?;
                Some(self.instance_info(&lot, definition))
            })
            .collect();
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![HousingReply::InstanceList(InstanceList {
                records,
                owner_guid,
            })],
            changed: false,
        })
    }

    fn lots_of(&self, owner_guid: u64) -> Vec<LotSnapshot> {
        self.state()
            .lots
            .into_iter()
            .filter(|lot| lot.owner_guid == owner_guid)
            .collect()
    }

    fn leave(&mut self, actor_guid: u64) -> Result<ServiceOutcome, ServiceError> {
        let house_guid = self
            .entered
            .remove(&actor_guid)
            .ok_or(ServiceError::NotInHouse { actor_guid })?;
        self.edit_mode.remove(&actor_guid);
        self.selections.retain(|(owner, _), _| *owner != actor_guid);
        Ok(ServiceOutcome {
            zone_change: Some(HousingZoneChange::ReturnToPreviousZone {
                actor_guid,
                house_guid,
            }),
            ..ServiceOutcome::default()
        })
    }

    fn set_edit_mode(
        &mut self,
        actor_guid: u64,
        in_edit_mode: bool,
    ) -> Result<ServiceOutcome, ServiceError> {
        let house_guid = self.house_of(actor_guid)?;
        self.housing.can_enter(actor_guid, house_guid)?;
        self.edit_mode.insert(actor_guid, in_edit_mode);
        let lot = self.housing.saved_fixtures(house_guid)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![HousingReply::HouseInfo(self.house_info(actor_guid, &lot))],
            changed: false,
        })
    }

    fn place(
        &mut self,
        actor_guid: u64,
        item_instance_id: u32,
        transform: WireTransform,
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        let house_guid = self.house_of(actor_guid)?;
        self.housing.can_enter(actor_guid, house_guid)?;
        self.require_edit_mode(actor_guid)?;
        let instance = stock
            .instances()
            .into_iter()
            .find(|instance| instance.instance_id == item_instance_id)
            .ok_or(ServiceError::UnknownInstance {
                instance_id: item_instance_id,
            })?;
        if instance.count != 1 {
            return Err(ServiceError::StackedInstanceUnsupported {
                instance_id: item_instance_id,
                count: instance.count,
            });
        }
        let row = self
            .config
            .row_for_item_definition(instance.definition_id)?;
        let fixture_definition_id = row.fixture_definition_id;
        let lot = self.housing.saved_fixtures(house_guid)?;
        let housing_transform = housing_transform(&transform, instance.tint_id, IDENTITY_TILT)?;

        // Stage the whole housing side first. A refusal anywhere below leaves the live state and
        // the live inventory untouched, because the staged clone is dropped.
        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        let outcome = staged.place_fixture(
            actor_guid,
            house_guid,
            fixture_definition_id,
            item_instance_id,
            housing_transform,
            &mut ledger,
        )?;
        let StagedChange::Consume(_) = outcome.staged else {
            return Err(ServiceError::Housing(HousingError::InvalidPlacement));
        };
        // Build the reply before anything commits, so no fallible step can run after the state
        // changed. The record only needs the configuration row and the staged fixture.
        let update = self.fixture_update(&outcome.placed, fixture_definition_id)?;
        // The live inventory stays the single owner of the item: mutate it last, and only after
        // every housing check passed. `Inventory::consume` is internally atomic, so a refusal here
        // drops the staged lot and leaves both sides exactly as they were.
        stock.consume(item_instance_id, 1)?;
        self.housing = staged;
        let saved = self.housing.saved_fixtures(house_guid)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![
                HousingReply::FixtureUpdate(update),
                HousingReply::HouseInfo(self.house_info(actor_guid, &saved)),
            ],
            changed: true,
        })
    }

    fn pickup(
        &mut self,
        actor_guid: u64,
        fixture_guid: u64,
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        let house_guid = self.house_of(actor_guid)?;
        self.housing.can_enter(actor_guid, house_guid)?;
        self.require_edit_mode(actor_guid)?;
        let lot = self.housing.saved_fixtures(house_guid)?;
        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        let outcome = staged.pickup_fixture(actor_guid, house_guid, fixture_guid, &mut ledger)?;
        let StagedChange::Return(proposal) = outcome.staged else {
            return Err(ServiceError::Housing(HousingError::UnknownFixture));
        };
        // Mirror of placement: the real inventory gets the unit back before the staged lot commits.
        // Housing keys its ledger on the fixture definition, so the item definition the backpack
        // knows is resolved through the same configuration row the placement used.
        let item_definition_id = self
            .config
            .row_for_fixture_definition(proposal.definition_id)?
            .item_definition_id;
        stock.return_instance(item_definition_id, proposal.tint_id)?;
        self.housing = staged;
        let saved = self.housing.saved_fixtures(house_guid)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![
                HousingReply::FixtureRemove(FixtureRemove { fixture_guid }),
                HousingReply::HouseInfo(self.house_info(actor_guid, &saved)),
            ],
            changed: true,
        })
    }

    fn pickup_all(
        &mut self,
        actor_guid: u64,
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        let house_guid = self.house_of(actor_guid)?;
        self.housing.can_enter(actor_guid, house_guid)?;
        self.require_edit_mode(actor_guid)?;
        let lot = self.housing.saved_fixtures(house_guid)?;
        if lot.fixtures.is_empty() {
            return Ok(ServiceOutcome {
                zone_change: None,
                replies: vec![HousingReply::HouseInfo(self.house_info(actor_guid, &lot))],
                changed: false,
            });
        }

        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        let mut returns = Vec::with_capacity(lot.fixtures.len());
        let mut removed = Vec::with_capacity(lot.fixtures.len());
        for fixture in &lot.fixtures {
            let outcome =
                staged.pickup_fixture(actor_guid, house_guid, fixture.fixture_guid, &mut ledger)?;
            let StagedChange::Return(proposal) = outcome.staged else {
                return Err(ServiceError::Housing(HousingError::UnknownFixture));
            };
            let row = self
                .config
                .row_for_fixture_definition(proposal.definition_id)?;
            returns.push((row.item_definition_id, proposal.tint_id));
            removed.push(HousingReply::FixtureRemove(FixtureRemove {
                fixture_guid: fixture.fixture_guid,
            }));
        }

        // The live inventory performs a single staged batch so a later return cannot leave a
        // partially emptied lot with only some fixtures restored to the backpack.
        stock.return_instances(&returns)?;
        self.housing = staged;
        let saved = self.housing.saved_fixtures(house_guid)?;
        removed.push(HousingReply::HouseInfo(self.house_info(actor_guid, &saved)));
        Ok(ServiceOutcome {
            zone_change: None,
            replies: removed,
            changed: true,
        })
    }

    fn house_of(&self, actor_guid: u64) -> Result<u64, ServiceError> {
        self.entered
            .get(&actor_guid)
            .copied()
            .ok_or(ServiceError::NotInHouse { actor_guid })
    }

    fn require_edit_mode(&self, actor_guid: u64) -> Result<(), ServiceError> {
        if self.edit_mode.get(&actor_guid).copied().unwrap_or(false) {
            Ok(())
        } else {
            Err(ServiceError::EditModeRequired)
        }
    }

    /// The placement ledger for one lot, projected from the two real owners of furniture: the live
    /// inventory and the fixtures the lot already holds. It is rebuilt per request and never kept,
    /// so it cannot drift into a second ledger of record.
    fn derive_ledger(&self, lot: &LotSnapshot, stock: &dyn FurnitureStock) -> FixtureItemLedger {
        let mut ledger = FixtureItemLedger::default();
        for instance in stock.instances() {
            // The ledger keys on the fixture definition, because that is the identity a placed
            // fixture carries; the item definition only resolves to it through the configuration.
            if let Ok(row) = self.config.row_for_item_definition(instance.definition_id) {
                let _ = ledger.seed(
                    row.fixture_definition_id,
                    instance.tint_id,
                    [instance.instance_id],
                );
            }
        }
        for fixture in &lot.fixtures {
            let instance = FixtureInstance {
                instance_id: fixture.item_instance_id,
                definition_id: fixture.fixture_definition_id,
                tint_id: fixture.transform.tint_id,
            };
            let _ = ledger.seed(
                instance.definition_id,
                instance.tint_id,
                [instance.instance_id],
            );
            let _ = ledger.stage(instance);
        }
        ledger
    }

    fn house_info(&self, actor_guid: u64, lot: &LotSnapshot) -> HouseInfo {
        HouseInfo {
            in_edit_mode: self.edit_mode.get(&actor_guid).copied().unwrap_or(false),
            unused: RECONSTRUCTED_HOUSE_UNUSED,
            is_locked: lot.locked,
            is_flora_allowed: RECONSTRUCTED_FLORA_ALLOWED,
            pet_autospawn: false,
            cur_fixture_count: lot.count.min(i32::MAX as u32) as i32,
            cur_landmark_count: 0,
            furniture_score: 0,
        }
    }

    /// Build the 127/45 record in the client reader's field order. Identity/count values come
    /// from the owned lot; display strings, ratings, timestamps and the leading flags/word are R
    /// defaults because no server capture or retail value table supplies them.
    fn zone_data(&self, lot: &LotSnapshot, definition: &HousingLotDefinition) -> ZoneData {
        ZoneData {
            flag_a: false,
            flag_b: false,
            unknown_word: 0,
            instance: self.instance_info(lot, definition),
        }
    }

    /// The `00aafe10` record shared by 127/45 and each 127/39 row.
    fn instance_info(
        &self,
        lot: &LotSnapshot,
        definition: &HousingLotDefinition,
    ) -> PlayerHousingInstanceInfo {
        PlayerHousingInstanceInfo {
            owner_guid: lot.owner_guid,
            instance_guid: lot.lot_guid,
            name_id: lot.name_id,
            house_name: String::new(),
            owner_name: String::new(),
            icon_id: definition.icon_id,
            fixture_count: lot.count.min(i32::MAX as u32) as i32,
            furniture_score: 0,
            last_visited: [0; 2],
            is_locked: lot.locked,
            is_members_only: false,
            is_flora_allowed: RECONSTRUCTED_FLORA_ALLOWED,
            description: String::new(),
            keyword_list: String::new(),
            unknown21: String::new(),
            rating: 0.0,
            votes: 0,
            has_rating: false,
            can_vote: false,
            factory_plot_id: 0,
            when_created: [0; 2],
        }
    }

    /// Build the full 127/28 reader shape. The six-coordinate Legacy bounds are expanded to two
    /// float4s with zero w values (R); keyed list ordinals and absent factory/display metadata are
    /// also R because the client supplies no lot resource table or server capture.
    fn instance_data(
        &self,
        lot: &LotSnapshot,
        definition: &HousingLotDefinition,
    ) -> Result<InstanceData, ServiceError> {
        let mut fixtures = Vec::with_capacity(lot.fixtures.len());
        for (index, saved) in lot.fixtures.iter().enumerate() {
            let key = u32::try_from(index + 1)
                .map_err(|_| ServiceError::HousingContentInvalid("too many house fixtures"))?;
            let placed = placed_from_saved(saved);
            fixtures.push((
                key,
                self.fixture_update(&placed, saved.fixture_definition_id)?,
            ));
        }
        let permissions = lot
            .permissions
            .iter()
            .enumerate()
            .map(|(index, permission)| {
                Ok(HousePermissionRecord {
                    key: u32::try_from(index + 1).map_err(|_| {
                        ServiceError::HousingContentInvalid("too many house permissions")
                    })?,
                    pair: [permission.guid as u32, (permission.guid >> 32) as u32],
                    level: permission.level.max(0) as u32,
                })
            })
            .collect::<Result<Vec<_>, ServiceError>>()?;
        let build_areas = lot
            .build_areas
            .iter()
            .map(|area| {
                [
                    [area[0], area[1], area[2], 0.0],
                    [area[3], area[4], area[5], 0.0],
                ]
            })
            .collect();
        Ok(InstanceData {
            house_guid: lot.lot_guid,
            owner_guid: lot.owner_guid,
            owner_name: String::new(),
            unknown4: 0,
            name_id: lot.name_id as u32,
            name: String::new(),
            unknown7: 0,
            max_fixtures: lot.max_fixtures,
            max_landmarks: lot.max_landmarks,
            fixtures,
            permissions,
            is_locked: lot.locked,
            is_flora_allowed: RECONSTRUCTED_FLORA_ALLOWED,
            cur_fixture_count: lot.count,
            cur_landmark_count: 0,
            unknown14: false,
            unknown15: 0,
            build_areas,
            icon_id: definition.icon_id as u32,
            preview: false,
            unknown18: false,
            furniture_score: 0,
            unknown20: 0,
            is_members_only: false,
            unknown22: String::new(),
            unknown23: String::new(),
            unknown24: false,
        })
    }

    /// The client 127/43 reader exposes fixture-info, definition and dword lists, but no selector
    /// field. This projects current inventory instances into that shape (R); selectors still must
    /// be bound explicitly by `offer` and are only resolved in the 127/1 -> 127/42 path.
    fn fixture_item_list(
        &self,
        stock: &dyn FurnitureStock,
    ) -> Result<FixtureItemList, ServiceError> {
        let mut instances = stock.instances();
        instances.sort_by_key(|instance| instance.instance_id);
        let mut items = Vec::new();
        let mut definitions = BTreeMap::new();
        for instance in instances {
            let Ok(row) = self.config.row_for_item_definition(instance.definition_id) else {
                continue;
            };
            items.push(FixtureInstanceInfo {
                fixture_guid: u64::from(instance.instance_id),
                item_definition_id: row.item_definition_id,
                coupling: CouplingDisplay {
                    id: 0,
                    npc_guids: Vec::new(),
                    composite_effect: 0,
                    effect_type: 0,
                },
            });
            definitions
                .entry(row.fixture_definition_id)
                .or_insert_with(|| fixture_definition_from_row(row));
        }
        Ok(FixtureItemList {
            items,
            definitions: definitions.into_values().collect(),
            words: Vec::new(),
        })
    }

    /// Resolve the first 127/1 selector through the explicitly registered owned-item binding.
    /// The selection namespace itself is not recovered, so missing or stale bindings fail closed.
    fn fixture_asset(
        &self,
        actor_guid: u64,
        selection_id: u32,
        stock: &dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        self.house_of(actor_guid)?;
        self.require_edit_mode(actor_guid)?;
        let instance_id = self
            .selections
            .get(&(actor_guid, selection_id))
            .copied()
            .ok_or(ServiceError::UnresolvedSelection { selection_id })?;
        let instance = stock
            .instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .filter(|instance| instance.count == 1)
            .ok_or(ServiceError::UnknownInstance { instance_id })?;
        let row = self
            .config
            .row_for_item_definition(instance.definition_id)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![HousingReply::FixtureAsset(FixtureAsset {
                unknown_word: 0,
                selection_id,
                definition: fixture_definition_from_row(row),
                bytes_a: String::new(),
                bytes_b: String::new(),
                unknown_a: 0,
                unknown_b: 0,
                bytes_c: String::new(),
                guids: Vec::new(),
                unknown_c: 0,
                unknown_d: 0,
                flag: false,
            })],
            changed: false,
        })
    }

    /// Build a `127/40` fixture record using the current local codec. Unresolved definition and tail
    /// fields use empty/zero placeholders (R); successful encoding is not client-acceptance proof.
    fn fixture_update(
        &self,
        placed: &PlacedFixture,
        fixture_definition_id: i32,
    ) -> Result<FixtureUpdate, ServiceError> {
        let row = self
            .config
            .row_for_fixture_definition(fixture_definition_id)?;
        let [x, y, z] = placed.transform.position;
        Ok(FixtureUpdate {
            instance: WireFixtureInstance {
                guid: placed.fixture_guid,
                house_guid: placed.house_guid,
                fixture_definition_id: placed.fixture_definition_id,
                unknown4: 0,
                position: [x, y, z, 1.0],
                rotation: placed.transform.rotation,
                tilt: placed.transform.tilt,
                npc_guid: 0,
                tint_id: placed.transform.tint_id,
                unknown10: 0,
                customization: Customization {
                    kind: 0,
                    texture_alias: String::new(),
                    tint_alias: String::new(),
                    tint_id: 0,
                    texture_override: String::new(),
                },
                unknown11: String::new(),
                unknown12: String::new(),
                unknown13: 0,
                xml_data: String::new(),
                scale: placed.transform.scale,
                unknown16: false,
                unknown17: 0,
            },
            info: FixtureInstanceInfo {
                fixture_guid: placed.fixture_guid,
                item_definition_id: row.item_definition_id,
                coupling: CouplingDisplay {
                    id: 0,
                    npc_guids: Vec::new(),
                    composite_effect: 0,
                    effect_type: 0,
                },
            },
            definition: fixture_definition_from_row(row),
            unknown_guids: Vec::new(),
            words: [0, 0, 0],
        })
    }
}

fn fixture_definition_from_row(row: &FixtureConfigRow) -> FixtureDefinition {
    FixtureDefinition {
        id: row.fixture_definition_id,
        item_definition_id: row.item_definition_id,
        unknown3: 0,
        model_id: row.model_id,
        flags: [false; 6],
        category: row.category.clone(),
        lua_call: row.lua_call.clone(),
        actions: Vec::new(),
        composite_effect_id: 0,
        unknown14: 0.0,
        unknown15: 0.0,
        flags2: [false; 4],
    }
}

fn placed_from_saved(saved: &SavedFixture) -> PlacedFixture {
    PlacedFixture {
        fixture_guid: saved.fixture_guid,
        house_guid: saved.house_guid,
        item_instance_id: saved.item_instance_id,
        fixture_definition_id: saved.fixture_definition_id,
        transform: saved.transform.clone(),
    }
}

/// Map the client wire transform onto the housing model under reconstructed policy (R): drop the
/// position w component, map an all-zero rotation block to identity to satisfy the local unit-
/// quaternion invariant, and validate every other rotation through the housing state machine.
fn housing_transform(
    transform: &WireTransform,
    tint_id: i32,
    tilt: [f32; 4],
) -> Result<FixtureTransform, ServiceError> {
    if transform
        .position
        .iter()
        .chain(transform.rotation.iter())
        .chain(std::iter::once(&transform.scale))
        .any(|value| !value.is_finite())
    {
        return Err(ServiceError::Housing(HousingError::NonFiniteTransform));
    }
    let rotation = if transform.rotation == [0.0; 4] {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        transform.rotation
    };
    Ok(FixtureTransform {
        position: [
            transform.position[0],
            transform.position[1],
            transform.position[2],
        ],
        rotation,
        tilt,
        scale: transform.scale,
        tint_id,
    })
}
