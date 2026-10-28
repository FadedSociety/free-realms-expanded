//! Housing integration service: the callable seam between the recovered native housing requests,
//! the restored `housing` state machine and the live character `Inventory`.
//!
//! Client facts routed here are documented in `notes/Build Guides/Gameplay/Housing.md` and
//! `notes/Reviews/Housing Placement Selection Identity Review.md`:
//! * housing is packet family 127; the client-to-server inner opcodes include 1
//!   (`PlaceFixtureRequest`), 2 (`PlaceFixture`), 3 (`PickupFixture`), 5 (`SaveFixture`), 6
//!   (`SetEditMode`), 19 (`EnterRequest`) and 20 (`PreviewByItemDefinitionIdRequest`);
//! * the `127/2` first dword is the client's placement *selection id*. The decorate bar passes the
//!   inventory item guid (`ScriptsBase.lua` `PopulateItems` -> `ItemSelected` ->
//!   `House.PlaceFixture`; binding `00c0b710` parses it and calls `00ac4200`), so 127/1 binds an
//!   owned single-unit fixture item under its own id, and 127/2 only resolves a bound id;
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
    FixtureInstance, FixtureItemLedger, FixtureTransform, Housing, HousingError, LotPermission,
    LotSnapshot, PERMISSION_CAN_VISIT, PlacedFixture, PropertyDefinition, PropertyGrant,
    SavedFixture, StagedChange,
};
use crate::housing_native::{
    CouplingDisplay, Customization, FixtureAsset, FixtureDefinition,
    FixtureInstance as WireFixtureInstance, FixtureInstanceInfo, FixtureItemList, FixtureRemove,
    FixtureUpdate, HouseInfo, HousePermissionRecord, InstanceData, InstanceList, MAX_STRING,
    NotifyHouseAdded, PlaceFixture as WirePlaceFixture, PlacementDenied, PlayerHousingInstanceInfo,
    PositionUpdate, ZoneData,
};
use crate::inventory::{
    Inventory, InventoryItem, InventoryState, instance_ids::allocate_from_snapshot,
};
use crate::startup_world::{self, ActorLook, NpcSpawn};
use crate::world;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

// The recovered client writers live in `src/restoration/piece_025.rs`, which is not registered in
// `lib.rs`, so the transcription is reused through `#[path]` (the same pattern `inventory.rs` uses
// for `piece_081`). When the lead registers the restoration modules, replace this include with
// `crate::restoration::piece_025`.
#[path = "restoration/piece_025.rs"]
#[allow(dead_code)]
mod wire;

pub mod creative;

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
pub const REPLY_NOTIFY_HOUSE_ADDED: u16 = 49;
pub const REPLY_POSITION: u16 = 51;
pub const REPLY_PLACEMENT_DENIED: u16 = 53;

/// Schema version of the persisted service state. Bump it when `HousingServiceState` changes.
/// 2 (L045) adds `SavedFixture::placed_by`; a schema-1 document still loads, its fixtures
/// attributed to the lot owner on restore, and is written back as 2.
pub const SERVICE_SCHEMA_VERSION: u32 = 2;
/// RECONSTRUCTED bound on the lots one character snapshot may carry.
pub const MAX_STATE_LOTS: usize = 64;
/// RECONSTRUCTED bound on the placed fixtures one character snapshot may carry.
pub const MAX_STATE_FIXTURES: usize = 4_096;
/// RECONSTRUCTED bound for one housing content document.
const MAX_HOUSING_CONTENT_BYTES: usize = 1024 * 1024;
/// RECONSTRUCTED bound on one fixture-configuration table.
const MAX_CONFIG_ROWS: usize = 4_096;
/// RECONSTRUCTED bound for one fixture-configuration file (`housing_fixtures.json`; the generated
/// table is about 460 KiB).
const MAX_FIXTURE_CONTENT_BYTES: usize = 4 * 1024 * 1024;
/// RECONSTRUCTED sanity bound on the fixtures a starter kit stands in a new lot (L040).
pub const MAX_STARTER_KIT_FIXTURES: usize = 16;

/// (R) Placeholder for the `127/44` field whose retail server value is not recovered.
const RECONSTRUCTED_HOUSE_UNUSED: i32 = 10;
/// (R) Default flora policy; the retail server value is not recovered.
const RECONSTRUCTED_FLORA_ALLOWED: bool = true;
/// 127/45 head scale in tenths: the client's own constructor default (`00ab07a0`), read at
/// `00ab6c80` and applied through `CuteMode:SetSizeInstant` (`00ac36c0` -> `00ab96e0`); 0 hides
/// the head. A future tunable should stay within about 1..60 (`CuteMode:Inflate` reaches 5.2).
const HOUSE_HEAD_SIZE_TENTHS: i32 = 10;
/// `Housing::mint_fixture_guid` always leaves this bit clear; set, it names the fixture's actor.
const FIXTURE_ACTOR_BIT: u64 = 1 << 55;

/// The guid of the NPC that draws a fixture (T806). The client keeps a fixture record and draws
/// nothing from it: in the community-preserved `p2.pcap` capture the server sends each fixture's
/// world actor as a 35/2 AddNpc and puts its guid in the record's NPC field (`FixtureInstance`
/// +216; `sub_AC8340` indexes the record by it, `sub_105DE10`), and moves it with a 127/51
/// addressed to that guid (handler `00ab8dc0`, entity lookup `sub_984960`). Derived from the
/// fixture guid, so a saved lot needs no second id.
pub fn fixture_actor_guid(fixture_guid: u64) -> u64 {
    fixture_guid | FIXTURE_ACTOR_BIT
}

/// The 35/3 removals for every fixture the lot holds. The actor guid is derived from the fixture
/// guid ([`fixture_actor_guid`]), so a leave and a pickup-all remove exactly the same actors.
fn fixture_removals(lot: &LotSnapshot) -> Vec<FixtureActor> {
    lot.fixtures
        .iter()
        .map(|fixture| FixtureActor::Remove(fixture_actor_guid(fixture.fixture_guid)))
        .collect()
}

/// Client transform projection: two 16-byte groups and a final dword. `127/2` (place) sends an
/// uninterpreted selection dword before the groups; `127/5` (move, writer `00abf620`) sends the
/// fixture guid instead. Mapping these groups to this server transform is R.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WireTransform {
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    /// `127/2`'s final dword is the placement scale (retail capture `p2.pcap` frame 31494). `127/5`
    /// puts 0 at the same offset in every real move (`p2.pcap` frames 32782 and 33039; the live
    /// 2026-10-27 client), so a move is not a rescale: [`HousingService::move_fixture`] keeps the
    /// placed fixture's scale.
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
    /// (R) Where the box came from: the Lxzxrus zone file or whereami readings, else Legacy
    /// (provisional, not a recovered client build area).
    pub bounds_source: String,
    /// (R) Why this row has no safe community scene-name candidate.
    pub disabled_reason: Option<String>,
    pub source_tag: String,
    pub enabled: bool,
    /// L046: the deed whose Marketplace purchase grants this lot; `None` when no deed is
    /// recorded. A deed names at most one lot (`HousingContent::validate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deed: Option<LotDeed>,
}

/// L046: the property deed (a Type 16 `ClientItemDefinition`) that sells one lot, with its source.
/// `store_bundle_ids` are the original Marketplace bundles whose entry is this deed, kept as
/// provenance: the revival lists a deed under its own item id, like every ride (R).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LotDeed {
    pub item_id: i32,
    #[serde(default)]
    pub store_bundle_ids: Vec<u32>,
    pub source: String,
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
            || self.deed.as_ref().is_some_and(|deed| {
                deed.item_id <= 0
                    || deed.source.trim().is_empty()
                    || deed.store_bundle_ids.contains(&0)
            })
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
    /// (R) What a new starter lot comes with. Absent: the lot comes empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starter_kit: Option<StarterKit>,
    pub client_evidence: Vec<String>,
    pub lots: Vec<HousingLotDefinition>,
}

/// (R) The starter lot's kit (L040): one unit for the bag, and fixtures already standing in the
/// lot (tint 0, identity rotation, scale 1). Both items need a fixture configuration row (checked
/// at load), so the unit can be placed and a standing fixture picked up.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarterKit {
    /// Granted as its own single-unit row, never merged into a stack, so it can be placed.
    pub item_id: i32,
    /// The item each standing fixture stands for; a pickup returns one unit of it.
    pub placed_item_id: i32,
    /// Where they stand: inside the starter lot's build area, at most `MAX_STARTER_KIT_FIXTURES`.
    pub placed_positions: Vec<[f32; 3]>,
}

impl StarterKit {
    /// Free issuance (the owner, 2026-10-20: starter items sell back for 0 coins), so every copy of
    /// these items resells for 0, as F228 does for job starter outfits.
    pub fn items(&self) -> [i32; 2] {
        [self.item_id, self.placed_item_id]
    }
}

impl HousingContent {
    pub fn bundled() -> Result<Self, ServiceError> {
        Self::parse(include_bytes!("../data/content/houses.json"))
    }

    /// The starter kit's `items` from `directory/houses.json`; none without that file.
    pub fn kit_items(directory: &Path) -> Result<Vec<i32>, ServiceError> {
        let path = directory.join("houses.json");
        if !path.is_file() {
            return Ok(Vec::new());
        }
        Ok(Self::load(&path)?
            .starter_kit
            .map_or(Vec::new(), |kit| kit.items().to_vec()))
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
        let mut deeds = BTreeSet::new();
        for lot in &self.lots {
            lot.validate()?;
            if !ids.insert(lot.property_id) {
                return Err(ServiceError::HousingContentInvalid(
                    "duplicate lot property id",
                ));
            }
            // L046: a purchase must resolve to exactly one lot.
            if lot
                .deed
                .as_ref()
                .is_some_and(|deed| !deeds.insert(deed.item_id))
            {
                return Err(ServiceError::HousingContentInvalid(
                    "one deed names two lots",
                ));
            }
        }
        let starter = self
            .lots
            .iter()
            .find(|lot| lot.property_id == self.starter_property_id)
            .ok_or(ServiceError::HousingContentInvalid(
                "starter property id has no lot row",
            ))?
            .property();
        // A kit that could not stand in the starter lot fails here, at load, not on a player's
        // first 127/23. `contains` is the placement's own build-area test (false for NaN).
        if self.starter_kit.as_ref().is_some_and(|kit| {
            kit.item_id <= 0
                || kit.placed_item_id <= 0
                || kit.placed_positions.len() > MAX_STARTER_KIT_FIXTURES
                || kit.placed_positions.len() as u32 > starter.max_fixtures
                || !kit.placed_positions.iter().all(|&at| starter.contains(at))
        }) {
            return Err(ServiceError::HousingContentInvalid("invalid starter kit"));
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

impl Request {
    /// The decorate requests (127/1, 2, 3, 4 and 6). A refusal of one is answered with the
    /// client's own 127/53 reason; other housing refusals stay a chat line.
    pub fn answers_with_denial(&self) -> bool {
        matches!(
            self,
            Self::PlaceFixtureRequest { .. }
                | Self::Place { .. }
                | Self::Pickup { .. }
                | Self::PickupAll
                | Self::SetEditMode { .. }
        )
    }
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
        // Client writer FUN_00abf620: fixture key, two four-float groups, then a 24-byte block
        // whose dword is 0 in every real move (not the 127/2 scale; see `WireTransform::scale`).
        // Only the transform prefix is interpreted.
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
    /// Read-only: the instance id the next grant would hand out. It and every id above it are
    /// held by no row and were handed out by no earlier grant.
    fn first_free_instance_id(&self) -> Result<u32, ServiceError>;
    /// The starter kit (L040), atomically: `ids` (strictly ascending, from
    /// `first_free_instance_id` up) are handed out. The first becomes one unit of `item_id` at
    /// tint 0, its own row and never merged into a stack, so it can be placed; the rest stand in
    /// the new lot, so no row is added for them and the counter moves past them.
    fn grant_kit(&mut self, item_id: i32, ids: &[u32]) -> Result<(), ServiceError>;
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

    fn first_free_instance_id(&self) -> Result<u32, ServiceError> {
        let live: Vec<i32> = self.state().items.iter().map(|item| item.id).collect();
        let (id, _) = allocate_from_snapshot(self.state().next_instance_id, &live)
            .map_err(|_| kit_refused())?;
        u32::try_from(id).map_err(|_| kit_refused())
    }

    fn grant_kit(&mut self, item_id: i32, ids: &[u32]) -> Result<(), ServiceError> {
        // Distinct and at or above the counter: no id is in the bag, was handed out before, or
        // is shared by the unit and a standing fixture (which would fail the save's conservation
        // check on the next load).
        let first = FurnitureStock::first_free_instance_id(self)?;
        let (Some(&unit), Some(&last)) = (ids.first(), ids.last()) else {
            return Err(kit_refused());
        };
        if unit < first || ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(kit_refused());
        }
        let mut state = self.state().clone();
        state.next_instance_id = Some(i64::from(last) + 1);
        state.items.push(InventoryItem {
            definition_id: item_id,
            tint: 0,
            id: i32::try_from(unit).map_err(|_| kit_refused())?,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        // Commits only if the whole bag validates: a known definition, at most MAX_ITEMS rows (a
        // full bag refuses), and the counter above every row and within the i32 id range.
        self.adopt_state(state).map_err(|_| kit_refused())?;
        Ok(())
    }
}

/// The bag refused the kit; nothing was committed.
fn kit_refused() -> ServiceError {
    ServiceError::StockRejected {
        operation: StockOperation::Return,
        instance_id: 0,
    }
}

fn owned_instance(item: &InventoryItem) -> Option<OwnedInstance> {
    let instance_id = u32::try_from(item.id).ok()?;
    if item.count == 0 || item.tint < 0 || creative::is_reserved_id(instance_id) {
        return None;
    }
    Some(OwnedInstance {
        instance_id,
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

/// 127/46 enter failure (client reader `0x00ab9080`): one u32 that clears the client's enter busy
/// flag and picks its message; 1 server not found, 2 and 6 permission denied, 3 zone missing.
pub const REPLY_ENTER_FAILED: u16 = 46;

impl ServiceError {
    /// T872: the 127/46 code for a refused 127/19. A lot this session cannot see is a house whose
    /// owner is not online (no copy was admitted); any other refusal is a permission one.
    pub fn enter_failure(&self) -> u32 {
        match self {
            Self::Housing(HousingError::LotNotFound) => 1,
            Self::HouseSceneUnavailable { .. } => 3,
            _ => 2,
        }
    }
}

/// The 127/46 packet for `code` (see `ServiceError::enter_failure`).
pub fn enter_failed(code: u32) -> Vec<u8> {
    [BASE_OPCODE, REPLY_ENTER_FAILED]
        .iter()
        .flat_map(|opcode| opcode.to_le_bytes())
        .chain(code.to_le_bytes())
        .collect()
}

/// T872: an owner's lot edit for the visitors inside: the house, its new snapshot (each visitor's
/// copy is replaced with it) and the packets they draw it from (fixture records and actors).
#[derive(Clone, Debug)]
pub struct HouseBroadcast {
    pub house_guid: u64,
    pub lot: LotSnapshot,
    pub packets: Vec<Vec<u8>>,
}

impl HouseBroadcast {
    /// Two edits in one bundle: the later snapshot, and both edits' packets in order.
    pub fn merge(earlier: Option<Self>, later: Option<Self>) -> Option<Self> {
        match (earlier, later) {
            (Some(mut first), Some(second)) if first.house_guid == second.house_guid => {
                first.packets.extend(second.packets);
                first.lot = second.lot;
                Some(first)
            }
            (first, second) => second.or(first),
        }
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
                | HousingError::NonUnitFacing,
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
    NotifyHouseAdded(NotifyHouseAdded),
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
            Self::NotifyHouseAdded(_) => REPLY_NOTIFY_HOUSE_ADDED,
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
            Self::NotifyHouseAdded(value) => value.encode(),
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

/// A fixture's world actor (T806): the NPC the client draws it as, in the opcode-35 family, not
/// the housing family. See [`fixture_actor_guid`].
#[derive(Clone, Debug, PartialEq)]
pub enum FixtureActor {
    /// 35/2 AddNpc.
    Spawn(Box<NpcSpawn>),
    /// 35/3 RemoveNpc of this actor guid.
    Remove(u64),
}

impl FixtureActor {
    /// The complete inner game packet.
    pub fn encode_inner(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Spawn(npc) => world::unframe_tunnel(&startup_world::build_add_npc(npc)),
            Self::Remove(guid) => Ok(crate::abilities::effect_packets::encode_remove_gracefully(
                *guid, 0,
            )),
        }
    }
}

/// The reply set and change flag for one request. `changed` means the caller must persist
/// `HousingService::state()` again; a denied or unrecovered request never sets it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServiceOutcome {
    /// An outer scene change that the gateway must begin before sending `replies`.
    pub zone_change: Option<HousingZoneChange>,
    pub replies: Vec<HousingReply>,
    /// The fixtures' actors, sent after `replies`: a retail placement answers 127/40, 127/44,
    /// then the 35/2, and a pickup 127/41, 127/44, then the 35/3.
    pub actors: Vec<FixtureActor>,
    pub changed: bool,
}

impl ServiceOutcome {
    fn unrecovered(opcode: u16) -> Self {
        Self {
            zone_change: None,
            replies: vec![HousingReply::Unrecovered { opcode }],
            actors: Vec::new(),
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

/// One row of the fixture configuration table: which item definition places as which fixture
/// definition and model. The production rows are generated from the pinned item data, the client
/// `Models.txt` and the Lxzxrus `HousingPlacementData.txt` (`scripts/build_housing_fixtures.py`,
/// T518/T869); the fixture definition id is our own convention (R) except for the seven captured
/// rows, because the client receives the definition in 127/42 and 127/43 and only needs a stable
/// unique key. (R) The retail server-side fixture definition and action table is not recovered.
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
    /// The item's texture material alias, sent in the 127/42 alias slots and as the 127/40
    /// customization texture.
    #[serde(default)]
    pub texture_alias: String,
    /// The item's dye-tint material alias (127/42 and 127/40).
    #[serde(default)]
    pub tint_alias: String,
    /// (R) The item's `Icon.TintId`; the client's default tint for the fixture. 0 when the item
    /// has no tint.
    #[serde(default)]
    pub default_tint_id: i32,
    /// Provenance. `capture_defaults` rows are reconstructed from an older packet-derived
    /// artifact; generated rows cite the pinned item row and the client model id.
    pub provenance: String,
}

/// The fixture configuration table. It is content, not save state: `HousingSession::load` reads
/// `housing_fixtures.json` from the content directory, and `capture_defaults` is the fallback when
/// that file is absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureConfig {
    pub schema_version: u32,
    pub provenance: String,
    pub rows: Vec<FixtureConfigRow>,
}

impl FixtureConfig {
    /// Historical reconstructed fixture rows, used when the content directory has no
    /// `housing_fixtures.json` and by the tests, plus the two starter-kit rows (L040). The current
    /// client guide does not validate these tuples; flags, category, Lua calls, actions and
    /// effects remain unknown.
    pub fn capture_defaults() -> Self {
        let row = |fixture_definition_id: i32, item_definition_id: i32, model_id: i32| {
            FixtureConfigRow {
                fixture_definition_id,
                item_definition_id,
                model_id,
                category: String::new(),
                lua_call: String::new(),
                texture_alias: String::new(),
                tint_alias: String::new(),
                default_tint_id: 0,
                provenance: format!(
                    "R:unverified-legacy-packet-derived-artifact; lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap 127/40 record: \
                     fixture definition {fixture_definition_id}, item definition \
                     {item_definition_id}, model {model_id}"
                ),
            }
        };
        // Not captured: the fixture id is the item id (the item-table convention) and the model
        // is the client's own Models.txt row for the item's ModelName.
        let kit = |item_definition_id: i32, model_id: i32, model: &str| FixtureConfigRow {
            provenance: format!(
                "R:L040 starter kit; legacy-data:FreeRealms-Legacy@9e08202 \
                 ClientItemDefinitions.json Id={item_definition_id} ModelName {model}; client \
                 Models.txt id {model_id}; fixture definition id = item id"
            ),
            ..row(item_definition_id, item_definition_id, model_id)
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
                kit(70_429, 535, "chicken.adr"),
                kit(32_647, 1_926, "hsg_block_01.adr"),
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

    /// Read and validate one fixture-configuration file. A broken document is refused whole, so a
    /// caller never continues with a partial table.
    pub fn load(path: &Path) -> Result<Self, ServiceError> {
        let file = File::open(path)
            .map_err(|_| ServiceError::FixtureConfigInvalid("cannot open fixture configuration"))?;
        let mut bytes = Vec::new();
        file.take((MAX_FIXTURE_CONTENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| ServiceError::FixtureConfigInvalid("cannot read fixture configuration"))?;
        if bytes.len() > MAX_FIXTURE_CONTENT_BYTES {
            return Err(ServiceError::FixtureConfigInvalid(
                "fixture configuration exceeds 4 MiB",
            ));
        }
        let config: Self = serde_json::from_slice(&bytes).map_err(|_| {
            ServiceError::FixtureConfigInvalid("invalid fixture configuration JSON")
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ServiceError> {
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
        let mut item_ids = BTreeSet::new();
        if self.rows.iter().any(|row| {
            row.fixture_definition_id <= 0 || row.item_definition_id <= 0 || row.model_id <= 0
        }) {
            return Err(ServiceError::FixtureConfigInvalid(
                "invalid fixture configuration id",
            ));
        }
        // A negative tint has no unsigned wire form, and an alias longer than the codec's string
        // bound would fail every 127/42 for that row: both are refused here, at load.
        if self.rows.iter().any(|row| {
            row.default_tint_id < 0
                || row.texture_alias.len() > MAX_STRING
                || row.tint_alias.len() > MAX_STRING
        }) {
            return Err(ServiceError::FixtureConfigInvalid(
                "invalid fixture configuration row",
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
        if self
            .rows
            .iter()
            .any(|row| !item_ids.insert(row.item_definition_id))
        {
            return Err(ServiceError::FixtureConfigInvalid(
                "duplicate item definition id",
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
    /// The fixture table is shared by every session clone. `HousingSession` clones its whole
    /// service per family-127 packet, so the table must not be copied with it.
    config: Arc<FixtureConfig>,
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
    starter_kit: Option<StarterKit>,
    /// T872: other characters' lots copied in for a visit. Served like any lot (edits stay the
    /// owner's through `can_enter`), never saved, and dropped when the visit ends.
    foreign: BTreeSet<u64>,
    /// T870: the decorate tray offers every fixture row, unowned and unlimited (`creative`).
    /// Content, not save state; off until `with_creative_catalog`.
    creative_catalog: bool,
}

impl HousingService {
    pub fn new(config: FixtureConfig) -> Self {
        Self {
            config: Arc::new(config),
            housing: Housing::new(),
            entered: BTreeMap::new(),
            edit_mode: BTreeMap::new(),
            selections: BTreeMap::new(),
            house_definitions: BTreeMap::new(),
            starter_property_id: None,
            starter_kit: None,
            foreign: BTreeSet::new(),
            creative_catalog: false,
        }
    }

    pub fn with_content(
        config: FixtureConfig,
        content: HousingContent,
    ) -> Result<Self, ServiceError> {
        config.validate()?;
        content.validate()?;
        let mut service = Self::new(config);
        service.adopt_content(content)?;
        Ok(service)
    }

    /// T870: switch the creative catalog (`creative::catalog_setting`). It changes only what the
    /// tray offers and what a selector may name; saved fixtures and the bag are untouched.
    pub fn with_creative_catalog(mut self, on: bool) -> Self {
        self.creative_catalog = on;
        self
    }

    /// The item a catalog selector names, while the catalog is on.
    fn catalog_item(&self, selection: u32) -> Option<i32> {
        self.creative_catalog
            .then(|| creative::selected_item(selection))
            .flatten()
    }

    /// The starter choices and lot rows of `content`. A kit item without a configuration row
    /// fails here, at load, rather than on a player's first 127/23.
    fn adopt_content(&mut self, content: HousingContent) -> Result<(), ServiceError> {
        for item in content.starter_kit.iter().flat_map(StarterKit::items) {
            self.config.row_for_item_definition(item)?;
        }
        self.starter_property_id = Some(content.starter_property_id);
        self.starter_kit = content.starter_kit;
        for definition in content.lots {
            self.register_house_definition(definition)?;
        }
        Ok(())
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

    /// L046 (R): a paid Marketplace deed grants the one enabled lot that names it. An owner who
    /// already has that lot is refused, so a repeated purchase is never charged. Replies: 127/49
    /// ("You have received a new %s.") with the lot's name, then the owner's 127/39 list. A deed
    /// with no enabled lot is the lookup refusal `InvalidProperty`, not a content error.
    ///
    /// The starter lot's deed is that refusal too: the starter is free and its kit (L040) rides
    /// only its 127/23 grant, so a deed must never hand it out kitless or for Station Cash.
    ///
    /// Not atomic on its own: an error after `grant_property` leaves the lot granted. Callers
    /// stage it on a clone (`HousingSession::grant_deed`, the only production caller).
    pub fn grant_deed(
        &mut self,
        owner_guid: u64,
        deed_item_id: i32,
    ) -> Result<ServiceOutcome, ServiceError> {
        let definition = self
            .house_definitions
            .values()
            .find(|lot| {
                lot.enabled
                    && Some(lot.property_id) != self.starter_property_id
                    && lot
                        .deed
                        .as_ref()
                        .is_some_and(|deed| deed.item_id == deed_item_id)
            })
            .cloned()
            .ok_or(HousingError::InvalidProperty)?;
        if self
            .lots_of(owner_guid)
            .iter()
            .any(|lot| lot.property_id == definition.property_id)
        {
            return Err(HousingError::PropertyAlreadyOwned.into());
        }
        let lot_guid = self.housing.lot_guid(owner_guid, definition.property_id)?;
        self.housing.grant_property(PropertyGrant {
            owner_guid,
            property: definition.property(),
            lot_guid,
        })?;
        let mut outcome = self.house_list(owner_guid, owner_guid)?;
        outcome.replies.insert(
            0,
            HousingReply::NotifyHouseAdded(NotifyHouseAdded {
                name_id: definition.name_id,
            }),
        );
        outcome.changed = true;
        Ok(outcome)
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
        if !(1..=SERVICE_SCHEMA_VERSION).contains(&state.schema_version) {
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
            .restore_lots(state.lots.iter().cloned().map(migrated_lot).collect())
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
            config: Arc::new(config),
            housing,
            entered: BTreeMap::new(),
            edit_mode: BTreeMap::new(),
            selections: BTreeMap::new(),
            house_definitions: BTreeMap::new(),
            starter_property_id: None,
            starter_kit: None,
            foreign: BTreeSet::new(),
            creative_catalog: false,
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
        service.adopt_content(content).map_err(StateError::Config)?;
        Ok(service)
    }

    /// The persisted half of this service, for the lead's canonical snapshot.
    /// The character's own lots; a visit's foreign copies are never saved.
    pub fn state(&self) -> HousingServiceState {
        HousingServiceState {
            schema_version: SERVICE_SCHEMA_VERSION,
            lots: (self.housing.snapshot_lots().into_iter())
                .filter(|lot| !self.foreign.contains(&lot.lot_guid))
                .collect(),
        }
    }

    /// T872: serve another character's lot to a visitor the gateway has already let in (F874:
    /// only then is a copy made). The copy is rebuilt with a restore's checks and replaces any
    /// older copy (the gateway refreshes it after each owner edit). It is read-only, unlocked and
    /// carries a level-1 pass for this visitor only. A lot this character owns is never replaced.
    pub fn admit_foreign_lot(
        &mut self,
        mut lot: LotSnapshot,
        visitor: u64,
    ) -> Result<(), ServiceError> {
        if lot.owner_guid == visitor || self.has_own_lot(lot.lot_guid) {
            return Err(HousingError::NotPermitted.into());
        }
        // A copy is read-only: an edit would move the visitor's furniture into a lot that is never
        // saved. The owner's real lot keeps its levels and its lock.
        for permission in &mut lot.permissions {
            permission.level = permission.level.min(PERMISSION_CAN_VISIT);
        }
        lot.permissions
            .retain(|permission| permission.guid != visitor);
        lot.permissions.push(LotPermission {
            guid: visitor,
            level: PERMISSION_CAN_VISIT,
        });
        lot.locked = false;
        let guid = lot.lot_guid;
        self.housing.admit_lot(lot)?;
        self.foreign.insert(guid);
        Ok(())
    }

    /// The house a visitor's copy is of, if any: the gateway refreshes only copies it handed out.
    pub fn has_foreign_lot(&self, house_guid: u64) -> bool {
        self.foreign.contains(&house_guid)
    }

    /// F874: whether this character owns the lot (never a visit's copy); a cheap check before
    /// the gateway clones one lot for a visitor.
    pub fn has_own_lot(&self, house_guid: u64) -> bool {
        !self.foreign.contains(&house_guid) && self.housing.has_lot(house_guid)
    }

    /// Drop every foreign copy except the one being visited (a visit keeps its own lot). F874:
    /// the gateway calls this after every message, so a listing or a refused enter keeps nothing.
    pub fn forget_foreign_lots(&mut self, keep: Option<u64>) {
        for guid in std::mem::take(&mut self.foreign) {
            if Some(guid) == keep {
                self.foreign.insert(guid);
            } else {
                self.housing.forget_lot(guid);
            }
        }
    }

    pub fn config(&self) -> &FixtureConfig {
        self.config.as_ref()
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
        if let Some(item) = self.catalog_item(item_instance_id) {
            self.config.row_for_item_definition(item)?;
            self.selections
                .insert((actor_guid, selection_id), item_instance_id);
            return Ok(());
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
            Request::RequestGrant => self.request_grant(actor_guid, stock),
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

    /// The typed move command. `127/5` carries the fixture guid and the two transform groups; its
    /// final dword is not `127/2`'s scale (see [`WireTransform::scale`]), so the placement's scale
    /// carries over and the position and rotation are applied. The client applies the confirmed
    /// transform from the `127/51` reply (handler `00ab8dc0`).
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
        let housing_transform = FixtureTransform {
            // A move never rescales: the real 127/5 dword is 0 (`WireTransform::scale`).
            scale: placed.transform.scale,
            ..housing_transform(&transform, placed.transform.tint_id)?
        };
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
                    actor_guid: fixture_actor_guid(fixture_guid),
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
            actors: Vec::new(),
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
        let mut actors = Vec::with_capacity(lot.fixtures.len());
        for saved in &lot.fixtures {
            let placed = placed_from_saved(saved);
            replies.push(HousingReply::FixtureUpdate(
                self.fixture_update(&placed, saved.fixture_definition_id)?,
            ));
            actors.push(self.fixture_actor(&placed, saved.fixture_definition_id)?);
        }
        let (spawn_position, spawn_rotation) =
            (definition.spawn_position, definition.spawn_rotation);
        self.entered.insert(actor_guid, house_guid);
        self.edit_mode.insert(actor_guid, false);
        self.selections.retain(|(owner, _), _| *owner != actor_guid);
        self.forget_foreign_lots(Some(house_guid));
        Ok(ServiceOutcome {
            zone_change: Some(HousingZoneChange::EnterHouse {
                actor_guid,
                house_guid,
                scene,
                spawn_position,
                spawn_rotation,
            }),
            replies,
            actors,
            changed: false,
        })
    }

    /// 127/23 (R): the first ask grants the starter lot (idempotent: a repeat grants nothing) and
    /// answers with the owner's 127/39 list either way. A new lot comes with the content's
    /// starter kit: the lot, its standing fixtures and the unit in the bag commit together or not
    /// at all. 127/49 is not sent here; a purchased lot sends it (L046 `grant_deed`).
    fn request_grant(
        &mut self,
        actor_guid: u64,
        stock: &mut dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        let mut staged = self.clone();
        let lot_guid = staged.grant_starter_lot(actor_guid)?;
        let granted = staged.lots_of(actor_guid).len() > self.lots_of(actor_guid).len();
        let kit = self.starter_kit.as_ref().filter(|_| granted);
        let ids = match kit {
            Some(kit) => staged.stand_kit(actor_guid, lot_guid, kit, stock)?,
            None => Vec::new(),
        };
        let mut outcome = staged.house_list(actor_guid, actor_guid)?;
        outcome.changed = granted;
        if let Some(kit) = kit {
            // Last, like placement: the bag refuses atomically and the staged lot is dropped.
            stock.grant_kit(kit.item_id, &ids)?;
        }
        *self = staged;
        Ok(outcome)
    }

    /// Stand the kit's fixtures in the new lot (on this staged clone) and return the ids for
    /// `grant_kit`: the bag unit's, then one per fixture. They start at the bag's first free id
    /// and above every id this character's lots hold, because a save without a counter floors
    /// the bag's next id on its own rows only.
    fn stand_kit(
        &mut self,
        owner_guid: u64,
        lot_guid: u64,
        kit: &StarterKit,
        stock: &dyn FurnitureStock,
    ) -> Result<Vec<u32>, ServiceError> {
        let lots = self.state().lots;
        let held = lots.iter().flat_map(|lot| &lot.fixtures);
        let held = held
            .map(|fixture| fixture.item_instance_id)
            .max()
            .unwrap_or(0);
        let first = stock.first_free_instance_id()?.max(held.saturating_add(1));
        let ids = (0..=kit.placed_positions.len())
            .map(|offset| first.checked_add(u32::try_from(offset).ok()?))
            .collect::<Option<Vec<u32>>>()
            .ok_or_else(kit_refused)?;
        let fixture_definition_id = self
            .config
            .row_for_item_definition(kit.placed_item_id)?
            .fixture_definition_id;
        // The placements below draw on exactly these ids; an id the seed refused fails them.
        let mut ledger = FixtureItemLedger::default();
        ledger.seed(fixture_definition_id, 0, ids[1..].iter().copied());
        for (&id, &position) in ids[1..].iter().zip(&kit.placed_positions) {
            let transform = FixtureTransform {
                position,
                ..FixtureTransform::default()
            };
            self.housing.place_fixture(
                owner_guid,
                lot_guid,
                fixture_definition_id,
                id,
                transform,
                &mut ledger,
            )?;
        }
        Ok(ids)
    }

    /// 127/14 and the tail of 127/23: one owner's houses that `actor_guid` may visit (R: a
    /// visitor sees only lots `can_visit` allows; the owner sees all of their own).
    fn house_list(&self, actor_guid: u64, owner_guid: u64) -> Result<ServiceOutcome, ServiceError> {
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
            actors: Vec::new(),
            changed: false,
        })
    }

    /// Every lot of one owner this service holds, a visit's foreign copies included (127/14 for
    /// another player lists the copies the gateway admitted).
    fn lots_of(&self, owner_guid: u64) -> Vec<LotSnapshot> {
        (self.housing.snapshot_lots().into_iter())
            .filter(|lot| lot.owner_guid == owner_guid)
            .collect()
    }

    /// Forget the actor's visit: the entered lot, edit mode and selections. Leave uses it, and so
    /// does a reconnect, which ends the visit without a 127/10.
    pub fn end_visit(&mut self, actor_guid: u64) -> Option<u64> {
        // F874: copies go even when the actor never entered (a listing, a refused enter).
        self.forget_foreign_lots(None);
        let house_guid = self.entered.remove(&actor_guid)?;
        self.edit_mode.remove(&actor_guid);
        self.selections.retain(|(owner, _), _| *owner != actor_guid);
        Some(house_guid)
    }

    /// Leaving removes every fixture's actor (35/3) the visit spawned, so none is left behind in the
    /// zone the player returns to (T806 review finding 3). The removal timing is reconstructed (R):
    /// a pickup's 35/3 is captured, the leave order is not.
    fn leave(&mut self, actor_guid: u64) -> Result<ServiceOutcome, ServiceError> {
        let house_guid =
            *(self.entered.get(&actor_guid)).ok_or(ServiceError::NotInHouse { actor_guid })?;
        // Before `end_visit`, which drops a visitor's copy of the lot.
        let actors = self
            .housing
            .saved_fixtures(house_guid)
            .map(|lot| fixture_removals(&lot))
            .unwrap_or_default();
        self.end_visit(actor_guid);
        Ok(ServiceOutcome {
            zone_change: Some(HousingZoneChange::ReturnToPreviousZone {
                actor_guid,
                house_guid,
            }),
            actors,
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
            actors: Vec::new(),
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
        // A catalog selector places a fresh creative piece (tint 0, its own placed id); anything
        // else is a real instance of the bag.
        let catalog = self.catalog_item(item_instance_id);
        let (definition_id, tint_id, instance_id) = match catalog {
            Some(item) => {
                let placed = creative::next_placed_id(&self.housing.snapshot_lots())
                    .ok_or(HousingError::InvalidPlacement)?;
                (item, 0, placed)
            }
            None => {
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
                (instance.definition_id, instance.tint_id, item_instance_id)
            }
        };
        let row = self.config.row_for_item_definition(definition_id)?;
        let fixture_definition_id = row.fixture_definition_id;
        let lot = self.housing.saved_fixtures(house_guid)?;
        let housing_transform = housing_transform(&transform, tint_id)?;

        // Stage the whole housing side first. A refusal anywhere below leaves the live state and
        // the live inventory untouched, because the staged clone is dropped.
        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        if catalog.is_some() && !ledger.seed(fixture_definition_id, tint_id, [instance_id]) {
            return Err(HousingError::InvalidPlacement.into());
        }
        let outcome = staged.place_fixture(
            actor_guid,
            house_guid,
            fixture_definition_id,
            instance_id,
            housing_transform,
            &mut ledger,
        )?;
        let StagedChange::Consume(_) = outcome.staged else {
            return Err(ServiceError::Housing(HousingError::InvalidPlacement));
        };
        // Build the reply before anything commits, so no fallible step can run after the state
        // changed. The record only needs the configuration row and the staged fixture.
        let update = self.fixture_update(&outcome.placed, fixture_definition_id)?;
        let actor = self.fixture_actor(&outcome.placed, fixture_definition_id)?;
        // The live inventory stays the single owner of the item: mutate it last, and only after
        // every housing check passed. `Inventory::consume` is internally atomic, so a refusal here
        // drops the staged lot and leaves both sides exactly as they were. A creative piece
        // consumes nothing.
        if catalog.is_none() {
            stock.consume(item_instance_id, 1)?;
        }
        self.housing = staged;
        let saved = self.housing.saved_fixtures(house_guid)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![
                HousingReply::FixtureUpdate(update),
                HousingReply::HouseInfo(self.house_info(actor_guid, &saved)),
            ],
            actors: vec![actor],
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
        // A creative piece (T870) was never an item, so nothing comes back.
        if !creative::is_placed_id(proposal.instance_id) {
            let item_definition_id = self
                .config
                .row_for_fixture_definition(proposal.definition_id)?
                .item_definition_id;
            stock.return_instance(item_definition_id, proposal.tint_id)?;
        }
        self.housing = staged;
        let saved = self.housing.saved_fixtures(house_guid)?;
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![
                HousingReply::FixtureRemove(FixtureRemove { fixture_guid }),
                HousingReply::HouseInfo(self.house_info(actor_guid, &saved)),
            ],
            actors: vec![FixtureActor::Remove(fixture_actor_guid(fixture_guid))],
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
                actors: Vec::new(),
                changed: false,
            });
        }

        let mut staged = self.housing.clone();
        let mut ledger = self.derive_ledger(&lot, stock);
        let mut returns = Vec::with_capacity(lot.fixtures.len());
        let mut removed = Vec::with_capacity(lot.fixtures.len());
        let actors = fixture_removals(&lot);
        for fixture in &lot.fixtures {
            let outcome =
                staged.pickup_fixture(actor_guid, house_guid, fixture.fixture_guid, &mut ledger)?;
            let StagedChange::Return(proposal) = outcome.staged else {
                return Err(ServiceError::Housing(HousingError::UnknownFixture));
            };
            if !creative::is_placed_id(proposal.instance_id) {
                let row = self
                    .config
                    .row_for_fixture_definition(proposal.definition_id)?;
                returns.push((row.item_definition_id, proposal.tint_id));
            }
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
            actors,
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
    /// from the owned lot and the head size is the client default; display strings, ratings,
    /// timestamps and the leading flags are R defaults because no server capture or retail value
    /// table supplies them.
    fn zone_data(&self, lot: &LotSnapshot, definition: &HousingLotDefinition) -> ZoneData {
        ZoneData {
            flag_a: false,
            flag_b: false,
            head_size_tenths: HOUSE_HEAD_SIZE_TENTHS,
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
            rating: 0.0,
            votes: 0,
            has_rating: false,
            can_vote: false,
            factory_plot_id: 0,
            directory_snapshot: String::new(),
            when_created: [0; 2],
        }
    }

    /// Build the full 127/28 reader shape. The six-coordinate Legacy bounds are expanded to two
    /// float4s with w = 1: the community-preserved `p2.pcap` client capture (reliable record seq
    /// 12619, frame 29882) carries w = 1.0 on both corners, and the client's containment test
    /// `sub_105C290` masks w out (`xmmword_1CB54A0` = (1, 1, 1, 0), set by `sub_173C140`), so w
    /// cannot fail it. Keyed list ordinals and absent factory/display metadata are R: the capture
    /// is a different lot, so no 127/28 of this one exists.
    fn instance_data(
        &self,
        lot: &LotSnapshot,
        definition: &HousingLotDefinition,
    ) -> Result<InstanceData, ServiceError> {
        let mut fixtures = Vec::with_capacity(lot.fixtures.len());
        for (index, saved) in lot.fixtures.iter().enumerate() {
            let key = u32::try_from(index + 1)
                .map_err(|_| ServiceError::HousingContentInvalid("too many house fixtures"))?;
            fixtures.push((key, Self::fixture_instance(&placed_from_saved(saved))));
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
                    [area[0], area[1], area[2], 1.0],
                    [area[3], area[4], area[5], 1.0],
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
    /// field. This projects current inventory instances into that shape (R); a selector is bound
    /// by `offer` in the 127/1 -> 127/42 path.
    fn fixture_item_list(
        &self,
        stock: &dyn FurnitureStock,
    ) -> Result<FixtureItemList, ServiceError> {
        if self.creative_catalog {
            return Ok(self.catalog_item_list());
        }
        let mut instances = stock.instances();
        instances.sort_by_key(|instance| instance.instance_id);
        let mut items = Vec::new();
        let mut definitions = BTreeMap::new();
        for instance in instances {
            let Ok(row) = self.config.row_for_item_definition(instance.definition_id) else {
                continue;
            };
            items.push(fixture_info(
                u64::from(instance.instance_id),
                row.item_definition_id,
            ));
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

    /// T870: the 127/43 list of a creative catalog: one record per fixture row (unlimited, so
    /// none is ever used up), named by its selector. It replaces the owned list, which stays
    /// reachable through the bag. The table is bounded by `MAX_CONFIG_ROWS`, within the codec's
    /// `MAX_COUNT`.
    fn catalog_item_list(&self) -> FixtureItemList {
        let mut list = FixtureItemList {
            items: Vec::new(),
            definitions: Vec::new(),
            words: Vec::new(),
        };
        for row in &self.config.rows {
            let Some(selector) = creative::selector(row.item_definition_id) else {
                continue;
            };
            list.items
                .push(fixture_info(u64::from(selector), row.item_definition_id));
            list.definitions.push(fixture_definition_from_row(row));
        }
        list
    }

    /// Resolve the 127/1 selector. The decorate bar sends the inventory item guid (module note),
    /// so an unbound id is bound to itself through `offer`, which accepts only an owned
    /// single-unit item with a fixture row on the actor's own editable lot. A stale binding is
    /// re-checked below; anything else fails closed. A new binding replaces the actor's others:
    /// the client holds one cursor item, and a place/pickup loop must not grow the map (pickup
    /// returns the unit under a fresh id).
    fn fixture_asset(
        &mut self,
        actor_guid: u64,
        selection_id: u32,
        stock: &dyn FurnitureStock,
    ) -> Result<ServiceOutcome, ServiceError> {
        self.house_of(actor_guid)?;
        self.require_edit_mode(actor_guid)?;
        if !self.selections.contains_key(&(actor_guid, selection_id)) {
            self.offer(actor_guid, selection_id, selection_id, stock)?;
            self.selections
                .retain(|&(owner, id), _| owner != actor_guid || id == selection_id);
        }
        let instance_id = self
            .selections
            .get(&(actor_guid, selection_id))
            .copied()
            .ok_or(ServiceError::UnresolvedSelection { selection_id })?;
        let item_definition_id = match self.catalog_item(instance_id) {
            Some(item) => item,
            None => {
                stock
                    .instances()
                    .into_iter()
                    .find(|instance| instance.instance_id == instance_id)
                    .filter(|instance| instance.count == 1)
                    .ok_or(ServiceError::UnknownInstance { instance_id })?
                    .definition_id
            }
        };
        let row = self.config.row_for_item_definition(item_definition_id)?;
        // Client reader 00ac1f50: the model dword, the selection dword, the definition, then the
        // two alias strings and the tint dword. The aliases and tint come from the item row (R for
        // the semantic names; the slot order is the client's).
        Ok(ServiceOutcome {
            zone_change: None,
            replies: vec![HousingReply::FixtureAsset(FixtureAsset {
                model_id: u32::try_from(row.model_id)
                    .map_err(|_| ServiceError::FixtureConfigInvalid("invalid fixture model id"))?,
                selection_id,
                definition: fixture_definition_from_row(row),
                texture_alias: row.texture_alias.clone(),
                tint_alias: row.tint_alias.clone(),
                tint_id: u32::try_from(row.default_tint_id)
                    .map_err(|_| ServiceError::FixtureConfigInvalid("invalid fixture tint id"))?,
                unknown_b: 0,
                bytes_c: String::new(),
                guids: Vec::new(),
                unknown_c: 0,
                unknown_d: 0,
                flag: false,
            })],
            actors: Vec::new(),
            changed: false,
        })
    }

    /// The bare `FixtureInstance` inside a 127/28 keyed list (client `00ab8c50` -> `00ab6f40`).
    /// The list carries only this instance; the 127/40 carries the definition, info and tail, and
    /// the model, aliases and tint ride the fixture's actor (T806), so no configuration row is read
    /// here. That keeps the 127/28 list decoupled from a definition lookup it does not need.
    fn fixture_instance(placed: &PlacedFixture) -> WireFixtureInstance {
        let [x, y, z] = placed.transform.position;
        WireFixtureInstance {
            guid: placed.fixture_guid,
            house_guid: placed.house_guid,
            fixture_definition_id: placed.fixture_definition_id,
            unknown4: 0,
            position: [x, y, z, 1.0],
            rotation: placed.transform.rotation,
            // Derived, so a lot saved before T806 (which stored a wrong tilt) is sent right.
            tilt: FixtureTransform::facing(placed.transform.rotation),
            npc_guid: fixture_actor_guid(placed.fixture_guid),
            tint_id: placed.transform.tint_id,
            unknown10: 0,
            // Client 127/40 reader sub_AB6F40 -> sub_8E8510: kind dword, texture alias, tint
            // alias, tint dword, override string; all empty, as the capture sends them.
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
        }
    }

    /// Build a `127/40` fixture record using the current local codec. The record carries no look:
    /// all 9 127/40s and all 2,015 127/28 instances of the community-preserved `p2.pcap` capture
    /// leave the customization block empty, and the model, texture alias and tint ride the
    /// fixture's actor ([`Self::fixture_actor`]). Unresolved definition and tail fields use
    /// empty/zero placeholders (R); successful encoding is not client-acceptance proof.
    fn fixture_update(
        &self,
        placed: &PlacedFixture,
        fixture_definition_id: i32,
    ) -> Result<FixtureUpdate, ServiceError> {
        let row = self
            .config
            .row_for_fixture_definition(fixture_definition_id)?;
        Ok(FixtureUpdate {
            instance: Self::fixture_instance(placed),
            info: fixture_info(placed.fixture_guid, row.item_definition_id),
            definition: fixture_definition_from_row(row),
            unknown_guids: Vec::new(),
            words: [0, 0, 0],
        })
    }

    /// The NPC that draws a placed fixture (T806), the 35/2 a server sends beside its 127/40.
    /// Values are those of the 2,015 fixture AddNpcs of the community-preserved `p2.pcap` capture:
    /// the fixture's position, `rotation` the facing and `tilt` the Euler rotation (the pairing
    /// held for every one, rotated or not), scale 1, the item's model, aliases and tint (the dyed
    /// tint when the item has one, else the item's own default), nameplate hidden and interactable.
    /// Name id, status and health follow-ups (35/9, 35/20) are not sent (R): the actor draws
    /// without them.
    fn fixture_actor(
        &self,
        placed: &PlacedFixture,
        fixture_definition_id: i32,
    ) -> Result<FixtureActor, ServiceError> {
        let row = self
            .config
            .row_for_fixture_definition(fixture_definition_id)?;
        let [x, y, z] = placed.transform.position;
        let tint_id = match placed.transform.tint_id {
            0 => row.default_tint_id,
            dyed => dyed,
        };
        let mut npc = NpcSpawn {
            actor_id: fixture_actor_guid(placed.fixture_guid),
            model_id: row.model_id,
            scale: placed.transform.scale,
            position: [x, y, z, 1.0],
            rotation: FixtureTransform::facing(placed.transform.rotation),
            tilt: placed.transform.rotation,
            has_tilt: 1,
            hide_nameplate: 1,
            sub_text_name_string_id: 0,
            movement_type: 0,
            unknown_flag_33: 1,
            unknown_flag_34: 1,
            active_profile: 144,
            unknown_field_68: 0,
            ..world::content_spawn_template()
        };
        ActorLook {
            texture_alias: row.texture_alias.clone(),
            tint_alias: row.tint_alias.clone(),
            tint_id,
        }
        .apply(&mut npc);
        Ok(FixtureActor::Spawn(Box::new(npc)))
    }
}

/// A 127/40 or 127/43 fixture record with no coupling effect (the capture sends none).
fn fixture_info(fixture_guid: u64, item_definition_id: i32) -> FixtureInstanceInfo {
    FixtureInstanceInfo {
        fixture_guid,
        item_definition_id,
        coupling: CouplingDisplay {
            id: 0,
            npc_guids: Vec::new(),
            composite_effect: 0,
            effect_type: 0,
        },
    }
}

/// T783: `unknown3` is the placement type. The client's per-frame validity check (`sub_ABB630`,
/// `0x00ABB6AD`) gives up unless it is 1, 0xE or 0xF, which left every preview invalid ("That
/// fixture can not be placed there.", `sub_AC8940` `0x00AC899E`) and sent no 127/2. All 18
/// captured retail definitions carry 1, flag 3 set and both scales 1.0.
fn fixture_definition_from_row(row: &FixtureConfigRow) -> FixtureDefinition {
    FixtureDefinition {
        id: row.fixture_definition_id,
        item_definition_id: row.item_definition_id,
        unknown3: 1,
        model_id: row.model_id,
        flags: [false, false, false, true, false, false],
        category: row.category.clone(),
        lua_call: row.lua_call.clone(),
        actions: Vec::new(),
        composite_effect_id: 0,
        unknown14: 1.0,
        unknown15: 1.0,
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

/// A lot saved before T806/T826, brought to the current fixture records. Idempotent, and keyed on
/// the old shapes themselves, so the schema version stays:
/// * a fixture guid with bit 63 set (the old namespace, top byte `ff`) loses that bit: the client
///   prints it as a negative decimal and its Lua guid reader turns that into guid 0, so no radial
///   menu action (move, rotate, remove) reached the fixture and rotating crashed the client;
/// * the pair rotation `[1, 0, 0, 0]` with tilt `[1, 0, 0, 0]` is what the old identity mapping
///   stored for a fixture the client placed unrotated. As Euler lanes it is a one radian yaw, and
///   no placement made since can have it (the tilt is always the facing of the rotation).
fn migrated_lot(mut lot: LotSnapshot) -> LotSnapshot {
    for fixture in &mut lot.fixtures {
        fixture.fixture_guid &= !(1 << 63);
        let transform = &mut fixture.transform;
        if transform.rotation == [1.0, 0.0, 0.0, 0.0] && transform.tilt == [1.0, 0.0, 0.0, 0.0] {
            transform.rotation = [0.0; 4];
        }
        // The rotation is the one source; a stale unit tilt in an old or hand-edited save is
        // corrected here, and validation refuses any new pair that disagrees.
        transform.tilt = FixtureTransform::facing(transform.rotation);
    }
    lot
}

/// Map the client wire transform onto the housing model: drop the position w component, keep the
/// rotation lanes as the client sent them (Euler radians, see [`FixtureTransform`]) and derive the
/// tilt they imply. The all-zero block the client sends for an unrotated fixture stays zero.
fn housing_transform(
    transform: &WireTransform,
    tint_id: i32,
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
    Ok(FixtureTransform {
        position: [
            transform.position[0],
            transform.position[1],
            transform.position[2],
        ],
        rotation: transform.rotation,
        tilt: FixtureTransform::facing(transform.rotation),
        scale: transform.scale,
        tint_id,
    })
}
