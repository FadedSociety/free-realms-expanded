//! House loading and visiting (work item 022): recovered entrance packets and bounded
//! instance routing.
//!
//! Evidence labels follow `notes/Research/Housing Restoration.md` and
//! `notes/Research/Pieces/022 House loading and visiting.md`:
//!
//! ORIGINAL (wire bytes recovered from a preserved live-session capture, cross-checked against the
//! supplied client and the supplied server's packet enum):
//! * base packet 127 (`BaseHousingPacket`) with sub-opcodes: `ClientHousingPacketRequestGrant` 23,
//!   `ClientHousingPacketEnterRequest` 19, `HousingPacketZoneData` 45,
//!   `HousingPacketFixtureUpdate` 40;
//! * the native client constructs the same family (`FUN_00aa4990(0x13)` for EnterRequest,
//!   `FUN_00ab07a0` for ZoneData with `IsPreview`/`Unused`/`HeadSize` default 10);
//! * `ClientUpdatePacketBeginZoning` (inner opcode 31) is the zone-entry packet; the captured one
//!   names the lot zone `hsg_emptylot_seaside_beach_01` with zone id 1, geometry id 214,
//!   position (440.632, -0.071, 432.801) and rotation (-0.9999741, 0, -0.0072035603, 0);
//! * the captured 127/45 response repeats the requested house guid as `InstanceGuid` and names the
//!   owner guid, house name and owner name, so the client's "which instance am I in" identity is
//!   the house guid, not the ephemeral server zone id.
//!
//! RECOVERED-COMMUNITY (pinned community sources, not the supplied binaries):
//! * housing zone instances are keyed by `(zone definition id, owner id)` and require a non-null
//!   owner (`Sanctuary/src/Sanctuary.Game/ZoneManager.cs`), so an owner and a visiting guest must
//!   resolve to the same instance;
//! * empty zone instances are evicted and later recreated with a fresh runtime id
//!   (`ZoneManager.EvictIfEmpty`), while the house data behind them stays put;
//! * `PlayerHousingInstanceInfo` / `HousingPacketZoneData` field order (validated here by
//!   re-encoding the captured bodies byte for byte).
//!
//! RECONSTRUCTED (this module only, deliberately bounded):
//! * the monotonic runtime instance ids and the "first zone-data response" flag;
//! * the snapshot format;
//! * treating the request's `unknown` int as the preview flag. The community model comments say
//!   "set by EnterPreview"; the only capture has 0 with `IsPreview = false`, so non-zero => preview
//!   is an inference, not a recovered rule;
//! * no permission policy. Invites, locks and refusals belong to piece 023
//!   (`can_enter`/`HousingDenial`/`encode_housing_error`); this module only distinguishes owner
//!   from guest and never fabricates a denial packet.
//!
//! Deliberate gaps (see the research note): no capture of a guest visiting another player's house,
//! no recovered body for `HousingPacketHousingError` (46), no captured instance eviction, no
//! furniture/landmark persistence here, and the meaning of the 127/45 tail word observed as
//! `0x0013_0000` (community name "Votes") is unexplained, so it is carried verbatim as data.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::{io, io::Write, path::Path};

/// ORIGINAL: `BasePackets.BaseHousingPacket` in the supplied server enum and the community model.
pub const BASE_PACKET_HOUSING: u16 = 127;
/// ORIGINAL: `ClientHousingPacketRequestGrant` (captured body empty).
pub const CLIENT_REQUEST_GRANT: u16 = 23;
/// ORIGINAL: `ClientHousingPacketEnterRequest`.
pub const CLIENT_ENTER_REQUEST: u16 = 19;
/// ORIGINAL: `HousingPacketZoneData`; the response that names the instance the client entered.
pub const HOUSING_ZONE_DATA: u16 = 45;
/// ORIGINAL: `HousingPacketFixtureUpdate`; binds fixtures to the same house guid.
pub const HOUSING_FIXTURE_UPDATE: u16 = 40;
/// ORIGINAL: `ClientUpdatePacketBeginZoning` inner opcode.
pub const BASE_PACKET_CLIENT_BEGIN_ZONING: u16 = 31;
/// ORIGINAL: the captured housing zone entry carried BeginZoning type 2.
pub const ZONE_TYPE_HOUSING: i32 = 2;
/// ORIGINAL: `HeadSize` was 10 in both captured 127/45 responses and is the client's default
/// (`FUN_00ab07a0` writes 10; `HousingPacketSetHeadSize` defaults to 10).
pub const DEFAULT_HEAD_SIZE: i32 = 10;
/// RECONSTRUCTED local bound so a malformed frame cannot allocate unbounded strings.
pub const MAX_MESSAGE_BYTES: usize = 4096;
/// RECONSTRUCTED snapshot schema for the local house table (no original save format recovered).
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// ORIGINAL: `ClientHousingPacketEnterRequest` body, validated against the captured 17-byte
/// message `7f 00 13 00 <house guid:8> <unknown:i32> <unknown2:bool>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterRequest {
    pub house_guid: u64,
    /// Community model comment: "Set by EnterPreview". Meaning is otherwise unrecovered.
    pub unknown: i32,
    pub unknown_bool: bool,
}

/// One persisted house: the data the server must keep whether or not a runtime zone exists.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HouseRecord {
    pub house_guid: u64,
    pub owner_guid: u64,
    /// BeginZoning `Id` and the property row's `ZoneId` (captured value 1 for the seaside beach
    /// lot; `Houses.json` row 1 also has `ZoneId` 1).
    pub zone_definition_id: i32,
    pub zone_name: String,
    /// BeginZoning `GeometryId` (captured 214 for `hsg_emptylot_seaside_beach_01`).
    pub geometry_id: i32,
    pub sky: String,
    pub spawn_position: [f32; 3],
    pub spawn_rotation: [f32; 4],
    pub name_id: i32,
    pub house_name: String,
    pub owner_name: String,
    pub icon_id: i32,
    pub fixture_count: i32,
    pub furniture_score: i32,
    pub is_locked: bool,
    pub is_members_only: bool,
    pub is_flora_allowed: bool,
    /// Community `PlayerHousingInstanceInfo.Votes` slot. The one capture sent `0x0013_0000` here,
    /// which is not a plausible vote count, so the field is preserved verbatim and not interpreted.
    pub tail_word: u32,
}

/// ORIGINAL BeginZoning field set as observed in the capture.
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneEntry {
    pub zone_name: String,
    pub zone_type: i32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub sky: String,
    pub tutorial: bool,
    /// Community `Unknown`, byte 1 in the capture.
    pub unknown: u8,
    pub zone_id: i32,
    pub update_radius: f32,
    pub geometry_id: i32,
    pub override_update_radius: bool,
    pub wait_for_zone_ready: bool,
}

impl ZoneEntry {
    /// RECONSTRUCTED: the per-packet flags are the captured values; only the lot fields come from
    /// the house record. If another capture shows different flags, add them to `HouseRecord`.
    pub fn from_house(house: &HouseRecord) -> Self {
        Self {
            zone_name: house.zone_name.clone(),
            zone_type: ZONE_TYPE_HOUSING,
            position: house.spawn_position,
            rotation: house.spawn_rotation,
            sky: house.sky.clone(),
            tutorial: false,
            unknown: 1,
            zone_id: house.zone_definition_id,
            update_radius: 0.0,
            geometry_id: house.geometry_id,
            override_update_radius: true,
            wait_for_zone_ready: false,
        }
    }
}

/// ORIGINAL `HousingPacketZoneData` field set; the tail word is opaque (see `tail_word`).
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneDataFields {
    pub is_preview: bool,
    /// Community `Unused`. The capture set it on the first response after entering and cleared it
    /// afterwards; the reason is unrecovered.
    pub unused_flag: bool,
    pub head_size: i32,
    pub owner_guid: u64,
    pub instance_guid: u64,
    pub name_id: i32,
    pub house_name: String,
    pub owner_name: String,
    pub icon_id: i32,
    pub fixture_count: i32,
    pub furniture_score: i32,
    pub last_visited: i64,
    pub is_locked: bool,
    pub is_members_only: bool,
    pub is_flora_allowed: bool,
    pub description: String,
    pub keyword_list: String,
    pub unknown21: String,
    pub rating: f32,
    pub tail_word: u32,
    pub has_rating: bool,
    pub can_vote: bool,
    pub factory_plot_id: i32,
    pub when_created: i64,
}

impl ZoneDataFields {
    /// Builds a response for one house. `instance_guid` is the requested house guid, matching the
    /// capture; `unused_flag` is supplied by the caller because its meaning is unrecovered.
    pub fn from_house(house: &HouseRecord, is_preview: bool, unused_flag: bool) -> Self {
        Self {
            is_preview,
            unused_flag,
            head_size: DEFAULT_HEAD_SIZE,
            owner_guid: house.owner_guid,
            instance_guid: house.house_guid,
            name_id: house.name_id,
            house_name: house.house_name.clone(),
            owner_name: house.owner_name.clone(),
            icon_id: house.icon_id,
            fixture_count: house.fixture_count,
            furniture_score: house.furniture_score,
            last_visited: 0,
            is_locked: house.is_locked,
            is_members_only: house.is_members_only,
            is_flora_allowed: house.is_flora_allowed,
            description: String::new(),
            keyword_list: String::new(),
            unknown21: String::new(),
            rating: 0.0,
            tail_word: house.tail_word,
            has_rating: false,
            can_vote: false,
            factory_plot_id: 0,
            when_created: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntranceRole {
    Owner,
    Guest,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntranceGrant {
    pub role: EntranceRole,
    /// RECONSTRUCTED runtime zone id, keyed by `(zone_definition_id, owner_guid)` as in the
    /// community `ZoneManager`. It is not the BeginZoning `Id` (that is the zone definition).
    pub instance_id: u32,
    pub zone: ZoneEntry,
    pub zone_data: ZoneDataFields,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntranceError {
    ShortMessage,
    TrailingData,
    UnexpectedOpcode,
    ZeroHouseGuid,
    ZeroOwnerGuid,
    NonFiniteGeometry,
    UnknownHouse(u64),
    DuplicateHouse(u64),
    DuplicateLotOwner {
        zone_definition_id: i32,
        owner_guid: u64,
    },
    UnknownInstance(u32),
    InstanceIdExhausted,
    SnapshotSchema(u32),
    SnapshotCorrupt,
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[offset..offset + 8]);
    u64::from_le_bytes(raw)
}

/// ORIGINAL decode of `7f 00 13 00 <house guid> <unknown i32> <unknown bool>`; strict, matching
/// the captured 17-byte message exactly.
pub fn decode_enter_request(message: &[u8]) -> Result<EnterRequest, EntranceError> {
    const BODY: usize = 4 + 8 + 4 + 1;
    if message.len() > MAX_MESSAGE_BYTES {
        return Err(EntranceError::TrailingData);
    }
    if message.len() < 4 {
        return Err(EntranceError::ShortMessage);
    }
    if read_u16(message, 0) != BASE_PACKET_HOUSING || read_u16(message, 2) != CLIENT_ENTER_REQUEST {
        return Err(EntranceError::UnexpectedOpcode);
    }
    if message.len() < BODY {
        return Err(EntranceError::ShortMessage);
    }
    if message.len() > BODY {
        return Err(EntranceError::TrailingData);
    }
    let house_guid = read_u64(message, 4);
    if house_guid == 0 {
        return Err(EntranceError::ZeroHouseGuid);
    }
    let unknown = i32::from_le_bytes([message[12], message[13], message[14], message[15]]);
    Ok(EnterRequest {
        house_guid,
        unknown,
        unknown_bool: message[16] != 0,
    })
}

/// ORIGINAL decode of `ClientHousingPacketRequestGrant`, whose captured body is empty.
pub fn decode_request_grant(message: &[u8]) -> Result<(), EntranceError> {
    if message.len() < 4 {
        return Err(EntranceError::ShortMessage);
    }
    if read_u16(message, 0) != BASE_PACKET_HOUSING || read_u16(message, 2) != CLIENT_REQUEST_GRANT {
        return Err(EntranceError::UnexpectedOpcode);
    }
    if message.len() > 4 {
        return Err(EntranceError::TrailingData);
    }
    Ok(())
}

fn push_string(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(&(text.len() as i32).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
}

/// ORIGINAL encoder for the BeginZoning inner message, in the captured field order:
/// `u16 opcode`, string name, `i32 type`, position f32 x4 (w = 1), rotation f32 x4, string sky,
/// bool tutorial, `u8 unknown`, `i32 id`, `f32 update radius`, `i32 geometry id`,
/// bool override update radius, bool wait for zone ready.
pub fn begin_zoning_message(entry: &ZoneEntry) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(&BASE_PACKET_CLIENT_BEGIN_ZONING.to_le_bytes());
    push_string(&mut out, &entry.zone_name);
    out.extend_from_slice(&entry.zone_type.to_le_bytes());
    for value in entry.position {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&1.0_f32.to_le_bytes());
    for value in entry.rotation {
        out.extend_from_slice(&value.to_le_bytes());
    }
    push_string(&mut out, &entry.sky);
    out.push(u8::from(entry.tutorial));
    out.push(entry.unknown);
    out.extend_from_slice(&entry.zone_id.to_le_bytes());
    out.extend_from_slice(&entry.update_radius.to_le_bytes());
    out.extend_from_slice(&entry.geometry_id.to_le_bytes());
    out.push(u8::from(entry.override_update_radius));
    out.push(u8::from(entry.wait_for_zone_ready));
    out
}

/// ORIGINAL encoder for `HousingPacketZoneData`, in the captured field order. The bodies this
/// produces were checked byte for byte against both captured 127/45 responses.
pub fn zone_data_message(data: &ZoneDataFields) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(&BASE_PACKET_HOUSING.to_le_bytes());
    out.extend_from_slice(&HOUSING_ZONE_DATA.to_le_bytes());
    out.push(u8::from(data.is_preview));
    out.push(u8::from(data.unused_flag));
    out.extend_from_slice(&data.head_size.to_le_bytes());
    out.extend_from_slice(&data.owner_guid.to_le_bytes());
    out.extend_from_slice(&data.instance_guid.to_le_bytes());
    out.extend_from_slice(&data.name_id.to_le_bytes());
    push_string(&mut out, &data.house_name);
    push_string(&mut out, &data.owner_name);
    out.extend_from_slice(&data.icon_id.to_le_bytes());
    out.extend_from_slice(&data.fixture_count.to_le_bytes());
    out.extend_from_slice(&data.furniture_score.to_le_bytes());
    out.extend_from_slice(&data.last_visited.to_le_bytes());
    out.push(u8::from(data.is_locked));
    out.push(u8::from(data.is_members_only));
    out.push(u8::from(data.is_flora_allowed));
    push_string(&mut out, &data.description);
    push_string(&mut out, &data.keyword_list);
    push_string(&mut out, &data.unknown21);
    out.extend_from_slice(&data.rating.to_le_bytes());
    out.extend_from_slice(&data.tail_word.to_le_bytes());
    out.push(u8::from(data.has_rating));
    out.push(u8::from(data.can_vote));
    out.extend_from_slice(&data.factory_plot_id.to_le_bytes());
    out.extend_from_slice(&data.when_created.to_le_bytes());
    out
}

#[derive(Clone, Debug)]
struct LiveInstance {
    instance_id: u32,
    occupants: BTreeSet<u64>,
}

/// Bounded entrance router. The persisted identity of a house is its `house_guid`; the runtime
/// zone instance behind it is keyed by `(zone_definition_id, owner_guid)`, so an owner and a guest
/// visiting the same house share one runtime instance and different houses never collide.
#[derive(Clone, Debug, Default)]
pub struct HousingEntranceRouter {
    houses: BTreeMap<u64, HouseRecord>,
    live: BTreeMap<(i32, u64), LiveInstance>,
    by_instance: BTreeMap<u32, (i32, u64)>,
    greeted: BTreeSet<u32>,
    next_instance_id: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EntranceSnapshot {
    pub schema_version: u32,
    pub houses: Vec<HouseRecord>,
}

impl HousingEntranceRouter {
    pub fn new() -> Self {
        // RECONSTRUCTED: the community ZoneManager's first assigned id is 1.
        Self {
            next_instance_id: 1,
            ..Self::default()
        }
    }

    /// Registers one persisted house. One `(zone definition, owner)` pair maps to one house, which
    /// mirrors the recovered keying; a second house for the same pair is refused rather than
    /// silently splitting the owner's instance.
    pub fn register_house(&mut self, house: HouseRecord) -> Result<(), EntranceError> {
        if house.house_guid == 0 {
            return Err(EntranceError::ZeroHouseGuid);
        }
        if house.owner_guid == 0 {
            return Err(EntranceError::ZeroOwnerGuid);
        }
        if !house.spawn_position.iter().all(|value| value.is_finite())
            || !house.spawn_rotation.iter().all(|value| value.is_finite())
        {
            return Err(EntranceError::NonFiniteGeometry);
        }
        if self.houses.contains_key(&house.house_guid) {
            return Err(EntranceError::DuplicateHouse(house.house_guid));
        }
        if self.houses.values().any(|existing| {
            existing.zone_definition_id == house.zone_definition_id
                && existing.owner_guid == house.owner_guid
        }) {
            return Err(EntranceError::DuplicateLotOwner {
                zone_definition_id: house.zone_definition_id,
                owner_guid: house.owner_guid,
            });
        }
        self.houses.insert(house.house_guid, house);
        Ok(())
    }

    pub fn house(&self, house_guid: u64) -> Option<&HouseRecord> {
        self.houses.get(&house_guid)
    }

    pub fn live_instance(&self, zone_definition_id: i32, owner_guid: u64) -> Option<u32> {
        self.live
            .get(&(zone_definition_id, owner_guid))
            .map(|live| live.instance_id)
    }

    pub fn occupants(&self, instance_id: u32) -> Option<usize> {
        self.by_instance
            .get(&instance_id)
            .and_then(|key| self.live.get(key))
            .map(|live| live.occupants.len())
    }

    /// Resolves an entrance request. The runtime instance is created on demand and reused for
    /// every later owner or guest entrance to the same house. Unknown houses are refused with a
    /// typed error; no denial packet is invented (that wire format belongs to piece 023).
    pub fn enter(
        &mut self,
        request: &EnterRequest,
        requester_guid: u64,
    ) -> Result<EntranceGrant, EntranceError> {
        if requester_guid == 0 {
            return Err(EntranceError::ZeroOwnerGuid);
        }
        let house = self
            .houses
            .get(&request.house_guid)
            .cloned()
            .ok_or(EntranceError::UnknownHouse(request.house_guid))?;
        let key = (house.zone_definition_id, house.owner_guid);
        let instance_id = match self.live.get(&key) {
            Some(live) => live.instance_id,
            None => {
                if self.next_instance_id == u32::MAX {
                    return Err(EntranceError::InstanceIdExhausted);
                }
                let id = self.next_instance_id;
                self.next_instance_id += 1;
                self.live.insert(
                    key,
                    LiveInstance {
                        instance_id: id,
                        occupants: BTreeSet::new(),
                    },
                );
                self.by_instance.insert(id, key);
                id
            }
        };
        if let Some(live) = self.live.get_mut(&key) {
            live.occupants.insert(requester_guid);
        }
        let first_response = self.greeted.insert(instance_id);
        let role = if requester_guid == house.owner_guid {
            EntranceRole::Owner
        } else {
            EntranceRole::Guest
        };
        Ok(EntranceGrant {
            role,
            instance_id,
            zone: ZoneEntry::from_house(&house),
            zone_data: ZoneDataFields::from_house(&house, request.unknown != 0, first_response),
        })
    }

    /// Removes one occupant. Returns `true` when the runtime instance became empty and was
    /// evicted, matching the community `EvictIfEmpty` behaviour; the house record is untouched.
    pub fn leave(&mut self, instance_id: u32, player_guid: u64) -> Result<bool, EntranceError> {
        let key = *self
            .by_instance
            .get(&instance_id)
            .ok_or(EntranceError::UnknownInstance(instance_id))?;
        let Some(live) = self.live.get_mut(&key) else {
            return Err(EntranceError::UnknownInstance(instance_id));
        };
        live.occupants.remove(&player_guid);
        let evicted = live.occupants.is_empty();
        if evicted {
            self.live.remove(&key);
            self.by_instance.remove(&instance_id);
            self.greeted.remove(&instance_id);
        }
        Ok(evicted)
    }

    pub fn snapshot(&self) -> EntranceSnapshot {
        EntranceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            houses: self.houses.values().cloned().collect(),
        }
    }

    /// Restores only the persisted houses. Runtime instances are deliberately not restored: the
    /// community server recreates zone instances on demand, so a restart starts with none.
    pub fn restore(&mut self, snapshot: EntranceSnapshot) -> Result<(), EntranceError> {
        if snapshot.schema_version != SNAPSHOT_SCHEMA_VERSION {
            return Err(EntranceError::SnapshotSchema(snapshot.schema_version));
        }
        let mut fresh = Self::new();
        for house in snapshot.houses {
            fresh.register_house(house)?;
        }
        *self = fresh;
        Ok(())
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), EntranceError> {
        let bytes = serde_json::to_vec_pretty(&self.snapshot())
            .map_err(|_| EntranceError::SnapshotCorrupt)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(EntranceError::SnapshotCorrupt);
        }
        write_atomic(path, &bytes).map_err(|_| EntranceError::SnapshotCorrupt)
    }

    pub fn load_from_path(path: &Path) -> Result<Self, EntranceError> {
        let bytes = fs::read(path).map_err(|_| EntranceError::SnapshotCorrupt)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(EntranceError::SnapshotCorrupt);
        }
        let snapshot: EntranceSnapshot =
            serde_json::from_slice(&bytes).map_err(|_| EntranceError::SnapshotCorrupt)?;
        let mut router = Self::new();
        router.restore(snapshot)?;
        Ok(router)
    }
}

/// Same-directory temporary file plus rename, matching the existing `character_store` / `housing`
/// contract: the destination only ever holds a complete snapshot.
fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut temporary = path.to_path_buf();
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("piece_022.json");
    temporary.set_file_name(format!(".{name}.{}.tmp", std::process::id()));
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    let mut retries = 0;
    loop {
        match fs::rename(&temporary, path) {
            Ok(()) => return Ok(()),
            // A concurrent Windows reader without FILE_SHARE_DELETE can deny replacement briefly;
            // same retry contract as `housing.rs`.
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(5 | 32))
                    && retries < 20 =>
            {
                retries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(error);
            }
        }
    }
}
