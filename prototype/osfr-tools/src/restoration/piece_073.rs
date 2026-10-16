//! Discoveries and exploration triggers (work item 073).
//!
//! ORIGINAL facts implemented here, with the recovered source of each:
//! * opcodes 56/57 for `PacketPointOfInterestDefinitionRequest`/`Reply`. The client's reply
//!   constructor writes `0x39` into the packet header (`pseudocode.c`), the supplied server's
//!   `LibSOE` enum spells both names, and the client ships the `PointOfInterestDefinitionManager`,
//!   `ClientPointOfInterestManager` and `CurrentPOIDataSource` vtables.
//! * the POI data-source column order `Id, Name, ZoneID, X, y, z, Heading, IconId,
//!   NotificationType, SubName, BreadcrumbQuestId, AtlasName` and the `BaseClient.CurrentPOI`
//!   columns `POIName, POIId, AreaId`, from the shipped `client-ui/ScriptsBase.lua`.
//! * the wire shape of one definition, from the captured gateway message with inner opcode 57
//!   (`lab/captures/legacy-gateway-20261008T014620082859Z/messages/0009-ready.bin`): a `true`
//!   entry flag, 13 little-endian fixed fields, a u32-length-prefixed ASCII atlas name, one
//!   trailing int, then a `false` list terminator. The supplied server's writer produces exactly
//!   that byte stream; the original client is the consumer of that message id in the recorded
//!   gateway milestone, though this piece did not re-inspect the native map/POI panel.
//! * `CollectionCategories.txt` row `1^93636^9535`: category id 1 is the exploration category;
//!   the shipped locale resolves name id 93636 to `Exploration`.
//!
//! COMMUNITY facts used, labelled: the pinned Sanctuary project's serializer writes the same field
//! order and names the notification types `Dungeon=3`, `Local=6`, `ZoneHub=7`, `Ruins=10`; the wiki
//! documents that an exploration collection entry is unlocked by approaching its listed location
//! and that rare-exploration sets award clothing.
//!
//! RECONSTRUCTED here - this module's own bounded policy, not recovered server code: planar (X/Z)
//! proximity, nearest-candidate-wins selection, the caller-supplied activation radius, the
//! per-character once-only ledger, and the optional location filter. The real activation radius is
//! NOT recovered by this piece, so it is a parameter. Nothing here grants experience, items, tokens,
//! collections or currency, and nothing here claims a native discovery run.
//!
//! Explicit gaps, do not imply these:
//! * the activation radius and the trigger shape (point radius vs. area volume) are unrecovered;
//! * the meanings of `unknown_11` and `unknown_15` are unrecovered; `unknown_15` is *not* asserted
//!   to be a radius;
//! * which `LocationId` a zone maps to is not modelled; the caller passes the filter;
//! * notification types 0 and 3 and the unused community type 10 are not resolved to first-party
//!   names here; 0 appears in the supplied table, 10 does not;
//! * no reward value is modelled: the recovered reward rows live in the evidence document.

use std::collections::{BTreeMap, BTreeSet};

/// ORIGINAL: `PacketPointOfInterestDefinitionRequest`, the client's own packet constructor argument.
pub const PACKET_POI_DEFINITION_REQUEST: u16 = 56;
/// ORIGINAL: `PacketPointOfInterestDefinitionReply`; the client constructor writes `0x39`.
pub const PACKET_POI_DEFINITION_REPLY: u16 = 57;

/// ORIGINAL: the `true` boolean the supplied server writes before each definition.
pub const POI_ENTRY_PRESENT: u8 = 1;
/// ORIGINAL: the `false` boolean that ends a definition list.
pub const POI_LIST_TERMINATOR: u8 = 0;

/// ORIGINAL: `BaseClient.PointOfInterestData` data-source name.
pub const POI_DATASOURCE_NAME: &str = "BaseClient.PointOfInterestData";
/// ORIGINAL: the columns of that data source, in index order.
pub const POI_DATASOURCE_COLUMNS: [&str; 12] = [
    "Id",
    "Name",
    "ZoneID",
    "X",
    "y",
    "z",
    "Heading",
    "IconId",
    "NotificationType",
    "SubName",
    "BreadcrumbQuestId",
    "AtlasName",
];
/// ORIGINAL: `BaseClient.CurrentPOI` data-source name.
pub const CURRENT_POI_DATASOURCE_NAME: &str = "BaseClient.CurrentPOI";
/// ORIGINAL: the columns of the current-POI data source.
pub const CURRENT_POI_COLUMNS: [&str; 3] = ["POIName", "POIId", "AreaId"];

/// ORIGINAL: the Lua update handler only reacts to these ids, which are the zone-hub rows.
pub const ZONE_NOTIFICATION_MIN_POI_ID: i32 = 2;
pub const ZONE_NOTIFICATION_MAX_POI_ID: i32 = 10;

/// ORIGINAL: `CollectionCategories.txt` row `1^93636^9535`, whose locale text is `Exploration`.
pub const COLLECTION_CATEGORY_EXPLORATION: i32 = 1;

/// COMMUNITY: Sanctuary's `PointOfInterestNotificationType` labels.
pub const POI_NOTIFICATION_DUNGEON: i32 = 3;
pub const POI_NOTIFICATION_LOCAL: i32 = 6;
pub const POI_NOTIFICATION_ZONE_HUB: i32 = 7;
pub const POI_NOTIFICATION_RUINS: i32 = 10;

/// Upper bound for one atlas name. The captured payload's longest is `sacredgrove` (11 bytes); the
/// bound keeps a corrupt length prefix from allocating unbounded memory.
const MAX_ATLAS_NAME_LEN: u32 = 4096;

/// ORIGINAL: one point-of-interest definition, in the captured field order.
#[derive(Clone, Debug, PartialEq)]
pub struct PoiDefinition {
    pub id: i32,
    pub name_id: i32,
    pub location_id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub heading: f32,
    pub icon_id: i32,
    pub notification_type: i32,
    pub sub_name_id: i32,
    pub unknown_11: i32,
    pub breadcrumb_quest_id: i32,
    pub teleport_location_id: i32,
    pub atlas_name: String,
    pub unknown_15: i32,
}

/// Why a byte stream is not a definition list. Every case fails closed instead of guessing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoiDecodeError {
    /// The stream ended before a complete definition or the list terminator.
    UnexpectedEof,
    /// An entry flag other than `1` (present) or `0` (terminator).
    UnknownFlag(u8),
    /// The stream continued after the terminator.
    TrailingBytes,
    /// The atlas name holds a non-ASCII byte; the field is written as ASCII.
    NonAsciiString,
    /// The atlas name length prefix is implausible.
    OversizedString,
}

/// Decode the body of a `PacketPointOfInterestDefinitionReply` into definitions.
///
/// The body is the payload after the two-byte opcode and the four-byte length: a sequence of
/// `POI_ENTRY_PRESENT`-flagged definitions terminated by `POI_LIST_TERMINATOR`.
pub fn decode_poi_definitions(bytes: &[u8]) -> Result<Vec<PoiDefinition>, PoiDecodeError> {
    let mut cursor = 0usize;
    let mut definitions = Vec::new();
    loop {
        let flag = *bytes.get(cursor).ok_or(PoiDecodeError::UnexpectedEof)?;
        cursor += 1;
        match flag {
            POI_LIST_TERMINATOR => break,
            POI_ENTRY_PRESENT => {}
            other => return Err(PoiDecodeError::UnknownFlag(other)),
        }

        let id = read_i32(bytes, &mut cursor)?;
        let name_id = read_i32(bytes, &mut cursor)?;
        let location_id = read_i32(bytes, &mut cursor)?;
        let x = read_f32(bytes, &mut cursor)?;
        let y = read_f32(bytes, &mut cursor)?;
        let z = read_f32(bytes, &mut cursor)?;
        let heading = read_f32(bytes, &mut cursor)?;
        let icon_id = read_i32(bytes, &mut cursor)?;
        let notification_type = read_i32(bytes, &mut cursor)?;
        let sub_name_id = read_i32(bytes, &mut cursor)?;
        let unknown_11 = read_i32(bytes, &mut cursor)?;
        let breadcrumb_quest_id = read_i32(bytes, &mut cursor)?;
        let teleport_location_id = read_i32(bytes, &mut cursor)?;

        let name_length = read_u32(bytes, &mut cursor)?;
        if name_length > MAX_ATLAS_NAME_LEN {
            return Err(PoiDecodeError::OversizedString);
        }
        let name_start = cursor;
        let name_end = name_start
            .checked_add(name_length as usize)
            .ok_or(PoiDecodeError::OversizedString)?;
        let name_bytes = bytes
            .get(name_start..name_end)
            .ok_or(PoiDecodeError::UnexpectedEof)?;
        if !name_bytes.is_ascii() {
            return Err(PoiDecodeError::NonAsciiString);
        }
        let atlas_name =
            String::from_utf8(name_bytes.to_vec()).map_err(|_| PoiDecodeError::NonAsciiString)?;
        cursor = name_end;

        let unknown_15 = read_i32(bytes, &mut cursor)?;

        definitions.push(PoiDefinition {
            id,
            name_id,
            location_id,
            x,
            y,
            z,
            heading,
            icon_id,
            notification_type,
            sub_name_id,
            unknown_11,
            breadcrumb_quest_id,
            teleport_location_id,
            atlas_name,
            unknown_15,
        });
    }

    if cursor != bytes.len() {
        return Err(PoiDecodeError::TrailingBytes);
    }
    Ok(definitions)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, PoiDecodeError> {
    let slice = bytes
        .get(*cursor..*cursor + 4)
        .ok_or(PoiDecodeError::UnexpectedEof)?;
    *cursor += 4;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn read_i32(bytes: &[u8], cursor: &mut usize) -> Result<i32, PoiDecodeError> {
    read_u32(bytes, cursor).map(|value| value as i32)
}

fn read_f32(bytes: &[u8], cursor: &mut usize) -> Result<f32, PoiDecodeError> {
    read_u32(bytes, cursor).map(f32::from_bits)
}

/// Why a definition list cannot be indexed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PoiTableError {
    /// Two definitions share an id; the wire format keys definitions by id.
    DuplicateId(i32),
}

/// Definitions indexed by id and by name id.
///
/// Ids are unique in the captured table and a repeat is refused. Name ids are NOT unique (the real
/// capture repeats `382885`), so the name index keeps every definition and callers must handle a
/// shared display name instead of this type picking one.
#[derive(Debug, PartialEq)]
pub struct PoiTable {
    definitions: Vec<PoiDefinition>,
    by_id: BTreeMap<i32, usize>,
    by_name_id: BTreeMap<i32, Vec<usize>>,
}

impl PoiTable {
    pub fn new(definitions: Vec<PoiDefinition>) -> Result<Self, PoiTableError> {
        let mut by_id = BTreeMap::new();
        let mut by_name_id: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
        for (index, definition) in definitions.iter().enumerate() {
            if by_id.insert(definition.id, index).is_some() {
                return Err(PoiTableError::DuplicateId(definition.id));
            }
            by_name_id
                .entry(definition.name_id)
                .or_default()
                .push(index);
        }
        Ok(Self {
            definitions,
            by_id,
            by_name_id,
        })
    }

    pub fn definitions(&self) -> &[PoiDefinition] {
        &self.definitions
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn by_id(&self, id: i32) -> Option<&PoiDefinition> {
        self.by_id.get(&id).map(|index| &self.definitions[*index])
    }

    /// Every definition carrying this display name. The captured table repeats one name id, so this
    /// returns a list; the small allocation is deliberate (149 rows in the shipped table).
    pub fn by_name_id(&self, name_id: i32) -> Vec<&PoiDefinition> {
        self.by_name_id
            .get(&name_id)
            .map(|indexes| {
                indexes
                    .iter()
                    .map(|index| &self.definitions[*index])
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// RECONSTRUCTED: an exploration collection entry carries a name id; when that name id is also a
/// POI name id, the entry names the place whose coordinates the definition holds. A name shared by
/// more than one definition is ambiguous and returns `None`; callers can inspect `by_name_id`.
pub fn poi_for_exploration_entry(table: &PoiTable, entry_name_id: i32) -> Option<&PoiDefinition> {
    let mut matches = table.by_name_id(entry_name_id);
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}

/// A character position on the ground plane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// The closest definition to a position, whether or not it was inside the radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearestPoi {
    pub poi_id: i32,
    pub distance: f32,
}

/// What one proximity evaluation produced. `NoCandidate` carries the nearest filtered definition
/// and its distance so a caller can log how far the character was.
#[derive(Clone, Debug, PartialEq)]
pub enum DiscoveryOutcome {
    Discovered(PoiDefinition),
    AlreadyDiscovered(i32),
    NoCandidate(Option<NearestPoi>),
}

/// A refused evaluation. Nothing is recorded when a request is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryError {
    /// The radius was negative or not finite. The real radius is unrecovered, so it is a parameter.
    InvalidRadius,
    /// A position component was not finite.
    InvalidPosition,
}

/// RECONSTRUCTED: once-only discovery ledger for one character. The captured profile carries a
/// `collected` flag on every exploration entry, so a place is discovered at most once per character;
/// this ledger is the server-side equivalent and grants nothing by itself.
#[derive(Debug)]
pub struct DiscoveryTracker {
    character_guid: u64,
    discovered: BTreeSet<i32>,
}

impl DiscoveryTracker {
    pub fn new(character_guid: u64) -> Self {
        Self {
            character_guid,
            discovered: BTreeSet::new(),
        }
    }

    pub fn character_guid(&self) -> u64 {
        self.character_guid
    }

    pub fn is_discovered(&self, poi_id: i32) -> bool {
        self.discovered.contains(&poi_id)
    }

    pub fn discovered_count(&self) -> usize {
        self.discovered.len()
    }

    /// Evaluate one position against the table.
    ///
    /// RECONSTRUCTED policy: candidates are the definitions whose `location_id` matches the caller's
    /// filter (`None` means every definition); distance is planar (X and Z); the nearest candidate
    /// wins; a candidate is in range when `distance <= radius`; the first in-range visit records the
    /// discovery and later visits return `AlreadyDiscovered`.
    pub fn evaluate(
        &mut self,
        table: &PoiTable,
        position: Position,
        location_filter: Option<i32>,
        radius: f32,
    ) -> Result<DiscoveryOutcome, DiscoveryError> {
        if !radius.is_finite() || radius < 0.0 {
            return Err(DiscoveryError::InvalidRadius);
        }
        if !position.x.is_finite() || !position.y.is_finite() || !position.z.is_finite() {
            return Err(DiscoveryError::InvalidPosition);
        }

        let mut nearest_any: Option<NearestPoi> = None;
        let mut nearest_in_range: Option<(usize, f32)> = None;
        for (index, definition) in table.definitions().iter().enumerate() {
            if let Some(location) = location_filter {
                if definition.location_id != location {
                    continue;
                }
            }
            let dx = definition.x - position.x;
            let dz = definition.z - position.z;
            let distance = (dx * dx + dz * dz).sqrt();
            if nearest_any.is_none_or(|nearest| distance < nearest.distance) {
                nearest_any = Some(NearestPoi {
                    poi_id: definition.id,
                    distance,
                });
            }
            if distance <= radius && nearest_in_range.is_none_or(|(_, best)| distance < best) {
                nearest_in_range = Some((index, distance));
            }
        }

        let Some((index, _)) = nearest_in_range else {
            return Ok(DiscoveryOutcome::NoCandidate(nearest_any));
        };
        let definition = table.definitions()[index].clone();
        if self.discovered.contains(&definition.id) {
            return Ok(DiscoveryOutcome::AlreadyDiscovered(definition.id));
        }
        self.discovered.insert(definition.id);
        Ok(DiscoveryOutcome::Discovered(definition))
    }
}
