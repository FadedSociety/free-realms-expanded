//! Work item 023: house permissions (access and edit roles).
//!
//! Evidence labels match the worker contract: ORIGINAL means the fact is read from a preserved
//! Free Realms artifact, RECONSTRUCTED means this module supplies a bounded local rule that the
//! lead must replace with native evidence before any historical-fidelity claim.
//!
//! ORIGINAL facts implemented here:
//! * housing is the inner-opcode family of base opcode 127
//!   (`lab/decompiled/server/LibSOE/SOE/BasePackets.cs:78`, `BaseHousingPackets.cs`). The
//!   access-relevant members also appear as client constructors in
//!   `lab/decompiled/client/FreeRealms/pseudocode.c`: `FUN_00aa4990(6)` ->
//!   `ClientHousingPacketSetEditMode` (1384085), `FUN_00aa4990(0x17)` ->
//!   `ClientHousingPacketRequestGrant` (1384220), `FUN_00aa4990(0xf)` ->
//!   `ClientHousingPacketInvitePlayer` (1385769), `FUN_00aa4990(0x16)` ->
//!   `ClientHousingPacketLockHouse` (1385871), `FUN_00aa4990(0x2e)` ->
//!   `HousingPacketHousingError` (1384530).
//! * a permission record is `{ guid: u64, level: i32 }` and level 3 is documented "can edit"
//!   (`lab/references/Sanctuary/src/Sanctuary.Packet.Common/Housing/InstancePermission.cs:7-10`).
//! * the instance permission map is keyed by an `int` and written as `count` then
//!   `key, guid, level` per entry
//!   (`PlayerHousingInstanceData.cs:55,78` plus `Sanctuary.Core/IO/PacketWriter.cs`
//!   `Write<TValue>(IDictionary<int, TValue>)`); the client holds the same map as
//!   `SoeUtil::HashListMap<int,InstancePermission,0,-1>`.
//! * a housing instance carries `OwnerGuid` and `IsLocked`
//!   (`PlayerHousingInstanceData.cs:10,21`).
//! * the client's house-browser row exposes the recovered columns "Permission Level" and
//!   "Can Edit" (`pseudocode.c:1795998`, `1796016`).
//! * `HousingPacketHousingError` (46) carries exactly one 4-byte code: the client handler
//!   `FUN_00ab9080` (1400087) reads one field (1400096) and switches on it (1400101) with the
//!   recovered messages at 1397301-1397491, whose permission text is locale id 421055
//!   ("You do not have permission to enter that house.").
//!
//! RECONSTRUCTED here (explicitly not original):
//! * who counts as owner/editor/guest/blocked, that an edit right also implies entry, that an
//!   unlocked house admits an unlisted visitor, the visitor cap handling, and the choice to answer
//!   a denied edit with the recovered permission-denied code.
//! * the local defensive bound on the decoded permission list.
//! * levels other than 3 keep their numeric value but grant no recovered capability; the client
//!   shows them verbatim.
//!
//! Deliberate gaps: member-only gating, invite/decline state, house naming, upkeep, furniture
//! scoring, and the world-side "which house may this player see" query are out of scope. Nothing
//! here is a native acceptance claim; the packet writer for the housing family is still owned by
//! the integration item.
//!
//! ponytail: one permission vector, one lock flag, one visitor counter. No trait objects, no
//! packet writer, no state machine. Add invite/session state only when a caller needs it.

/// ORIGINAL: `BasePackets.BaseHousingPacket` (supplied server, `BasePackets.cs:78`).
pub const BASE_HOUSING_PACKET: u16 = 127;

/// ORIGINAL inner opcodes from the supplied server's `BaseHousingPackets` enum, cross-checked
/// against the client constructors. Only the access-relevant members are published here; the
/// placement members live in the housing placement module.
pub mod opcodes {
    /// `ClientHousingPacketSetEditMode`
    pub const CLIENT_SET_EDIT_MODE: u16 = 6;
    /// `ClientHousingPacketLeaveHouse`
    pub const CLIENT_LEAVE_HOUSE: u16 = 10;
    /// `ClientHousingPacketToggleLocked`
    pub const CLIENT_TOGGLE_LOCKED: u16 = 11;
    /// `ClientHousingPacketRequestPlayerHouses`
    pub const CLIENT_REQUEST_PLAYER_HOUSES: u16 = 14;
    /// `ClientHousingPacketInvitePlayer`
    pub const CLIENT_INVITE_PLAYER: u16 = 15;
    /// `ClientHousingPacketDeclineInvite`
    pub const CLIENT_DECLINE_INVITE: u16 = 16;
    /// `ClientHousingPacketEnterRequest`
    pub const CLIENT_ENTER_REQUEST: u16 = 19;
    /// `ClientHousingPacketRequestVisitToNeighbor`
    pub const CLIENT_REQUEST_VISIT_TO_NEIGHBOR: u16 = 21;
    /// `ClientHousingPacketLockHouse`
    pub const CLIENT_LOCK_HOUSE: u16 = 22;
    /// `ClientHousingPacketRequestGrant`
    pub const CLIENT_REQUEST_GRANT: u16 = 23;
    /// `HousingPacketInstanceData` - the server-to-client body that carries the permission list.
    pub const INSTANCE_DATA: u16 = 28;
    /// `HousingPacketUpdateHouseInfo` - carries the `InEditMode` flag the client acts on.
    pub const UPDATE_HOUSE_INFO: u16 = 44;
    /// `HousingPacketZoneData`
    pub const ZONE_DATA: u16 = 45;
    /// `HousingPacketHousingError` - the denial reply.
    pub const HOUSING_ERROR: u16 = 46;
    /// `HousingPacketInviteNotification`
    pub const INVITE_NOTIFICATION: u16 = 47;
    /// `HousingPacketDeclineNotification`
    pub const DECLINE_NOTIFICATION: u16 = 48;
    /// `HousingPacketNotifyHouseAdded`
    pub const NOTIFY_HOUSE_ADDED: u16 = 49;
}

/// ORIGINAL: the only recovered permission level meaning. `InstancePermission.Level == 3` is
/// documented "Can Edit" and the client shows the same value in its "Permission Level" column.
pub const PERMISSION_CAN_EDIT: i32 = 3;

/// RECONSTRUCTED local defensive bound for a decoded permission list. The recovered model has no
/// documented maximum; this keeps a malformed count from allocating.
pub const MAX_PERMISSION_ENTRIES: usize = 4096;

/// ORIGINAL: `HousingPacketHousingError` codes recovered from the client switch (`FUN_00ab9080`).
pub const ERROR_SERVER_NOT_FOUND: i32 = 1;
/// Permission denied; the client prints "You do not have permission to enter that house."
pub const ERROR_PERMISSION_DENIED: i32 = 2;
/// The housing zone could not be found.
pub const ERROR_ZONE_NOT_FOUND: i32 = 3;
/// The housing zone failed to launch.
pub const ERROR_ZONE_FAILED_TO_LAUNCH: i32 = 4;
/// The house already has the maximum number of visitors.
pub const ERROR_ZONE_FULL: i32 = 5;
/// A second permission-denied code: the client treats 2 and 6 identically.
pub const ERROR_PERMISSION_DENIED_ALT: i32 = 6;

/// ORIGINAL client strings for the recovered error codes, as they appear in the client
/// (`pseudocode.c:1397303-1397491`). Codes outside this table are not recovered.
pub fn housing_error_message(code: i32) -> Option<&'static str> {
    match code {
        ERROR_SERVER_NOT_FOUND => Some("The housing server is not currently available."),
        ERROR_PERMISSION_DENIED | ERROR_PERMISSION_DENIED_ALT => {
            Some("You do not have permission to enter that house.")
        }
        ERROR_ZONE_NOT_FOUND => Some("That housing zone could not be found."),
        ERROR_ZONE_FAILED_TO_LAUNCH => Some("The housing zone failed to launch."),
        ERROR_ZONE_FULL => Some("The house already has the maximum number of visitors."),
        _ => None,
    }
}

/// Malformed permission bodies are rejected instead of silently repaired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PermissionError {
    /// The declared entry count exceeds the recovered-model defensive bound.
    TooManyEntries,
    /// The body ended before the declared entries did.
    Truncated,
    /// Bytes remained after the declared entries.
    TrailingBytes,
}

/// ORIGINAL record shape: `InstancePermission { Guid: u64, Level: i32 }` plus the `int` map key
/// the client and the community packet model both keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstancePermission {
    pub key: i32,
    pub guid: u64,
    pub level: i32,
}

impl InstancePermission {
    /// RECONSTRUCTED: level 3 is the recovered edit level; higher values keep the same capability
    /// so a future recovered level cannot silently lose an existing edit grant.
    pub fn can_edit(&self) -> bool {
        self.level >= PERMISSION_CAN_EDIT
    }
}

/// ORIGINAL wire layout: `count:i32` then `key:i32, guid:u64, level:i32` per entry, little-endian.
pub fn encode_permissions(entries: &[InstancePermission]) -> Result<Vec<u8>, PermissionError> {
    if entries.len() > MAX_PERMISSION_ENTRIES {
        return Err(PermissionError::TooManyEntries);
    }
    let mut bytes = Vec::with_capacity(4 + entries.len() * 16);
    bytes.extend_from_slice(&(entries.len() as i32).to_le_bytes());
    for entry in entries {
        bytes.extend_from_slice(&entry.key.to_le_bytes());
        bytes.extend_from_slice(&entry.guid.to_le_bytes());
        bytes.extend_from_slice(&entry.level.to_le_bytes());
    }
    Ok(bytes)
}

/// ORIGINAL wire layout, strict: the body must contain exactly the declared entries.
pub fn decode_permissions(bytes: &[u8]) -> Result<Vec<InstancePermission>, PermissionError> {
    if bytes.len() < 4 {
        return Err(PermissionError::Truncated);
    }
    let count = i32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes"));
    if count < 0 || count as usize > MAX_PERMISSION_ENTRIES {
        return Err(PermissionError::TooManyEntries);
    }
    let count = count as usize;
    if bytes.len() != 4 + count * 16 {
        return if bytes.len() < 4 + count * 16 {
            Err(PermissionError::Truncated)
        } else {
            Err(PermissionError::TrailingBytes)
        };
    }
    let mut entries = Vec::with_capacity(count);
    for chunk in bytes[4..].chunks_exact(16) {
        entries.push(InstancePermission {
            key: i32::from_le_bytes(chunk[0..4].try_into().expect("4 bytes")),
            guid: u64::from_le_bytes(chunk[4..12].try_into().expect("8 bytes")),
            level: i32::from_le_bytes(chunk[12..16].try_into().expect("4 bytes")),
        });
    }
    Ok(entries)
}

/// `HousingPacketHousingError` body: exactly one 4-byte code (`pseudocode.c:1400096-1400101`).
pub fn encode_housing_error(code: i32) -> Vec<u8> {
    code.to_le_bytes().to_vec()
}

/// Strict decoder for the denial body.
pub fn decode_housing_error(bytes: &[u8]) -> Result<i32, PermissionError> {
    if bytes.len() < 4 {
        return Err(PermissionError::Truncated);
    }
    if bytes.len() > 4 {
        return Err(PermissionError::TrailingBytes);
    }
    Ok(i32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")))
}

/// The client-visible role of one actor for one house.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessRole {
    /// The recovered `OwnerGuid` of the instance.
    Owner,
    /// A listed actor whose level grants the recovered "Can Edit" capability.
    Editor,
    /// A listed actor without an edit grant, or an unlisted actor while the house is unlocked.
    Guest,
    /// An unlisted actor while the house is locked.
    Blocked,
}

/// A refusal that the caller turns into the recovered `HousingPacketHousingError` reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HousingDenial {
    /// Recovered code 2 (the client also accepts 6).
    PermissionDenied,
    /// Recovered code 5.
    ZoneFull,
}

impl HousingDenial {
    /// The recovered `HousingPacketHousingError` code for this refusal.
    pub fn code(self) -> i32 {
        match self {
            HousingDenial::PermissionDenied => ERROR_PERMISSION_DENIED,
            HousingDenial::ZoneFull => ERROR_ZONE_FULL,
        }
    }

    /// The recovered client message for this refusal.
    pub fn client_message(self) -> &'static str {
        housing_error_message(self.code()).expect("recovered code has a recovered message")
    }
}

/// The permission-relevant part of one housing instance, mirroring the recovered
/// `PlayerHousingInstanceData` fields this work item owns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseAccess {
    /// ORIGINAL `OwnerGuid`.
    pub owner_guid: u64,
    /// ORIGINAL `IsLocked`.
    pub locked: bool,
    /// RECONSTRUCTED: current visitor count, owned by the caller's zone/session state.
    pub visitors: u32,
    /// RECONSTRUCTED: the client proves a visitor cap exists but no recovered field carries its
    /// value, so `None` means "no local cap known".
    pub max_visitors: Option<u32>,
    /// ORIGINAL permission list, decoded from `HousingPacketInstanceData`.
    pub permissions: Vec<InstancePermission>,
}

impl HouseAccess {
    /// RECONSTRUCTED role rule over the recovered fields.
    pub fn role_of(&self, actor_guid: u64) -> AccessRole {
        if actor_guid == self.owner_guid {
            return AccessRole::Owner;
        }
        match self
            .permissions
            .iter()
            .find(|entry| entry.guid == actor_guid)
        {
            Some(entry) if entry.can_edit() => AccessRole::Editor,
            Some(_) => AccessRole::Guest,
            None if self.locked => AccessRole::Blocked,
            None => AccessRole::Guest,
        }
    }

    /// RECONSTRUCTED: the client's "Permission Level" column shows the actor's listed level.
    /// The owner has no listed entry in the recovered model, so the owner answers with the
    /// recovered edit level instead of an invented row.
    pub fn client_permission_level(&self, actor_guid: u64) -> Option<i32> {
        if actor_guid == self.owner_guid {
            return Some(PERMISSION_CAN_EDIT);
        }
        self.permissions
            .iter()
            .find(|entry| entry.guid == actor_guid)
            .map(|entry| entry.level)
    }

    /// RECONSTRUCTED: the client's "Can Edit" column.
    pub fn client_can_edit(&self, actor_guid: u64) -> bool {
        matches!(
            self.role_of(actor_guid),
            AccessRole::Owner | AccessRole::Editor
        )
    }

    /// RECONSTRUCTED entry rule: the owner is never locked out or counted against the visitor
    /// cap; any other role needs an entry grant and a free visitor slot.
    pub fn can_enter(&self, actor_guid: u64) -> Result<AccessRole, HousingDenial> {
        let role = self.role_of(actor_guid);
        if role == AccessRole::Blocked {
            return Err(HousingDenial::PermissionDenied);
        }
        if role != AccessRole::Owner && self.visitor_cap_reached() {
            return Err(HousingDenial::ZoneFull);
        }
        Ok(role)
    }

    /// RECONSTRUCTED edit rule: only the owner and a level-3 (or higher) editor may request edit
    /// mode, place, move, pick up or save. Everything else answers the recovered denial code.
    pub fn can_edit(&self, actor_guid: u64) -> Result<(), HousingDenial> {
        if self.client_can_edit(actor_guid) {
            Ok(())
        } else {
            Err(HousingDenial::PermissionDenied)
        }
    }

    fn visitor_cap_reached(&self) -> bool {
        match self.max_visitors {
            Some(max) => self.visitors >= max,
            None => false,
        }
    }
}
