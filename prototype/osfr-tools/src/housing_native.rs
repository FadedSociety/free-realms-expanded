//! Native housing placement codecs. Typed encode/decode methods use subtype-level bodies,
//! excluding the opcode-127 family prefix; HousingPacket also offers explicit inner framing.
//!
//! Units: every `encode`/`decode` entry point starts at the **subtype `u16`** and excludes the
//! base opcode 127. Lengths are local codec expectations, not proof of original server payloads or
//! native client acceptance. The client guide records reader/writer evidence and explicitly excludes
//! OSFR-held packet captures. Legacy Sanctuary packet classes are data-shape references only.
//!
//! Client anchors: `notes/Build Guides/Gameplay/Housing.md` and native-map readers such as
//! `00ac9dd0` (127/28), `00ac1f50` (127/42), `00ac9a90` (127/43), `00ab6c80` (127/45),
//! `00ab2d10` (127/51), `00ac2180` (127/40), `00ab8ee0` (127/44) and `00ab3030` (127/53).
//! Where the guide marks server values unknown, zero/empty fields below are reconstructed (R).
//!
//! Policies (fail closed, documented rather than invented):
//! * every float must be finite on encode and on decode;
//! * strings are `i32` byte length + UTF-8, length `0..=MAX_STRING`, invalid UTF-8 is refused;
//! * array counts are `i32`, `0..=MAX_COUNT`, and must fit the remaining input;
//! * booleans decode only from `0`/`1` (the native client treats any nonzero byte as true; this
//!   codec is deliberately stricter);
//! * every decoder consumes its whole payload exactly — trailing bytes are an error, including
//!   the unconfirmed bytes after the 127/53 string;
//! * unknown positional fields keep their `unknownN` names; no meaning is invented. The 127/2
//!   `first_dword` is the client selection id (`+0x40`), never an inventory identity.

use std::io;

pub const MAX_STRING: usize = 4096;
pub const MAX_COUNT: usize = 4096;

pub const SUB_PLACE: u16 = 2;
pub const SUB_INSTANCE_DATA: u16 = 28;
pub const SUB_FIXTURE_UPDATE: u16 = 40;
pub const SUB_FIXTURE_REMOVE: u16 = 41;
pub const SUB_FIXTURE_ASSET: u16 = 42;
pub const SUB_FIXTURE_ITEM_LIST: u16 = 43;
pub const SUB_HOUSE_INFO: u16 = 44;
pub const SUB_ZONE_DATA: u16 = 45;
pub const SUB_POSITION: u16 = 51;
pub const SUB_PLACEMENT_DENIED: u16 = 53;

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn finite(values: &[f32]) -> io::Result<()> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(invalid("housing codec requires finite floats"))
    }
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn take(&mut self, count: usize) -> io::Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| invalid("housing codec offset overflow"))?;
        if end > self.data.len() {
            return Err(invalid("truncated housing payload"));
        }
        let chunk = &self.data[self.offset..end];
        self.offset = end;
        Ok(chunk)
    }

    fn subtype(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u8(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }

    fn bool(&mut self) -> io::Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(invalid(&format!("raw boolean byte {other} is not 0 or 1"))),
        }
    }

    fn u32(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> io::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> io::Result<f32> {
        let value = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
        finite(&[value])?;
        Ok(value)
    }

    fn f32x4(&mut self) -> io::Result<[f32; 4]> {
        Ok([self.f32()?, self.f32()?, self.f32()?, self.f32()?])
    }

    fn string(&mut self) -> io::Result<String> {
        let length = self.i32()?;
        if length < 0 || length as usize > MAX_STRING {
            return Err(invalid("housing string length out of range"));
        }
        if length == 0 {
            return Ok(String::new());
        }
        let bytes = self.take(length as usize)?;
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|_| invalid("housing string is not utf-8"))
    }

    fn count(&mut self) -> io::Result<usize> {
        let value = self.i32()?;
        if value < 0 || value as usize > MAX_COUNT {
            return Err(invalid("housing array count out of range"));
        }
        Ok(value as usize)
    }

    fn finish(&self) -> io::Result<()> {
        if self.offset == self.data.len() {
            Ok(())
        } else {
            Err(invalid("trailing housing bytes"))
        }
    }
}

#[derive(Default)]
struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn new(subtype: u16) -> Self {
        let mut out = Vec::with_capacity(64);
        out.extend_from_slice(&subtype.to_le_bytes());
        Self { out }
    }

    fn bool(&mut self, value: bool) {
        self.out.push(u8::from(value));
    }

    fn u32(&mut self, value: u32) {
        self.out.extend_from_slice(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.out.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.out.extend_from_slice(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) -> io::Result<()> {
        finite(&[value])?;
        self.out.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn f32x4(&mut self, values: [f32; 4]) -> io::Result<()> {
        finite(&values)?;
        for value in values {
            self.out.extend_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }

    fn string(&mut self, value: &str) -> io::Result<()> {
        if value.len() > MAX_STRING {
            return Err(invalid("housing string too long"));
        }
        self.i32(value.len() as i32);
        self.out.extend_from_slice(value.as_bytes());
        Ok(())
    }

    fn count(&mut self, value: usize) -> io::Result<()> {
        if value > MAX_COUNT {
            return Err(invalid("housing array count too large"));
        }
        self.i32(value as i32);
        Ok(())
    }

    fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// Client 127/2 `ClientHousingPacketPlaceFixture` body (42 bytes with the subtype).
/// `first_dword` is the selection id at placement-object `+0x40`; `rotation` is a local name for
/// the second float4 because its client-side semantics are not fully established.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceFixture {
    pub first_dword: u32,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub scale: f32,
}

impl PlaceFixture {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_PLACE);
        writer.u32(self.first_dword);
        writer.f32x4(self.position)?;
        writer.f32x4(self.rotation)?;
        writer.f32(self.scale)?;
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_PLACE {
            return Err(invalid("not a 127/2 place payload"));
        }
        let place = Self {
            first_dword: reader.u32()?,
            position: reader.f32x4()?,
            rotation: reader.f32x4()?,
            scale: reader.f32()?,
        };
        reader.finish()?;
        Ok(place)
    }
}

/// Legacy-reference customization block (Sanctuary `CustomizationDetail` order); retail semantics
/// and value rules remain unverified.
#[derive(Clone, Debug, PartialEq)]
pub struct Customization {
    pub kind: i32,
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
    pub texture_override: String,
}

impl Customization {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.i32(self.kind);
        writer.string(&self.texture_alias)?;
        writer.string(&self.tint_alias)?;
        writer.i32(self.tint_id);
        writer.string(&self.texture_override)?;
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            kind: reader.i32()?,
            texture_alias: reader.string()?,
            tint_alias: reader.string()?,
            tint_id: reader.i32()?,
            texture_override: reader.string()?,
        })
    }
}

/// Local fixture-record projection guided by the client reader and Legacy `FixtureInstance` shape.
/// Unknown fields are not evidence of retail server values.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureInstance {
    pub guid: u64,
    pub house_guid: u64,
    pub fixture_definition_id: i32,
    pub unknown4: i32,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub tilt: [f32; 4],
    pub npc_guid: u64,
    pub tint_id: i32,
    pub unknown10: i32,
    pub customization: Customization,
    pub unknown11: String,
    pub unknown12: String,
    pub unknown13: i32,
    pub xml_data: String,
    pub scale: f32,
    pub unknown16: bool,
    pub unknown17: i32,
}

impl FixtureInstance {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.u64(self.guid);
        writer.u64(self.house_guid);
        writer.i32(self.fixture_definition_id);
        writer.i32(self.unknown4);
        writer.f32x4(self.position)?;
        writer.f32x4(self.rotation)?;
        writer.f32x4(self.tilt)?;
        writer.u64(self.npc_guid);
        writer.i32(self.tint_id);
        writer.i32(self.unknown10);
        self.customization.encode(writer)?;
        writer.string(&self.unknown11)?;
        writer.string(&self.unknown12)?;
        writer.i32(self.unknown13);
        writer.string(&self.xml_data)?;
        writer.f32(self.scale)?;
        writer.bool(self.unknown16);
        writer.i32(self.unknown17);
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            guid: reader.u64()?,
            house_guid: reader.u64()?,
            fixture_definition_id: reader.i32()?,
            unknown4: reader.i32()?,
            position: reader.f32x4()?,
            rotation: reader.f32x4()?,
            tilt: reader.f32x4()?,
            npc_guid: reader.u64()?,
            tint_id: reader.i32()?,
            unknown10: reader.i32()?,
            customization: Customization::decode(reader)?,
            unknown11: reader.string()?,
            unknown12: reader.string()?,
            unknown13: reader.i32()?,
            xml_data: reader.string()?,
            scale: reader.f32()?,
            unknown16: reader.bool()?,
            unknown17: reader.i32()?,
        })
    }
}

/// Legacy-reference `CouplingDisplayData` order (id, npc guid list, effect, effect type).
#[derive(Clone, Debug, PartialEq)]
pub struct CouplingDisplay {
    pub id: i32,
    pub npc_guids: Vec<u64>,
    pub composite_effect: i32,
    pub effect_type: i32,
}

impl CouplingDisplay {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.i32(self.id);
        writer.count(self.npc_guids.len())?;
        for guid in &self.npc_guids {
            writer.u64(*guid);
        }
        writer.i32(self.composite_effect);
        writer.i32(self.effect_type);
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        let id = reader.i32()?;
        let count = reader.count()?;
        let mut npc_guids = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            npc_guids.push(reader.u64()?);
        }
        Ok(Self {
            id,
            npc_guids,
            composite_effect: reader.i32()?,
            effect_type: reader.i32()?,
        })
    }
}

/// Legacy-reference `FixtureInstanceInfo` shape; 28 bytes with the current empty-coupling codec.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureInstanceInfo {
    pub fixture_guid: u64,
    pub item_definition_id: i32,
    pub coupling: CouplingDisplay,
}

impl FixtureInstanceInfo {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.u64(self.fixture_guid);
        writer.i32(self.item_definition_id);
        self.coupling.encode(writer)
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            fixture_guid: reader.u64()?,
            item_definition_id: reader.i32()?,
            coupling: CouplingDisplay::decode(reader)?,
        })
    }
}

/// Legacy-reference `FixtureDefinition.FixtureAction` dictionary entry: map key, then value record
/// (`Id`, `Name`, `RequirementId`, `Unknown4`). The client guide does not establish this server
/// payload layout; map key and id are kept separate.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureAction {
    pub map_key: i32,
    pub id: i32,
    pub name: String,
    pub requirement_id: i32,
    pub unknown4: i32,
}

/// Legacy-reference `FixtureDefinition` projection (variable length; 50 bytes with empty strings
/// and actions under this local codec).
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureDefinition {
    pub id: i32,
    pub item_definition_id: i32,
    pub unknown3: i32,
    pub model_id: i32,
    pub flags: [bool; 6],
    pub category: String,
    pub lua_call: String,
    pub actions: Vec<FixtureAction>,
    pub composite_effect_id: i32,
    pub unknown14: f32,
    pub unknown15: f32,
    pub flags2: [bool; 4],
}

impl FixtureDefinition {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.i32(self.id);
        writer.i32(self.item_definition_id);
        writer.i32(self.unknown3);
        writer.i32(self.model_id);
        for flag in self.flags {
            writer.bool(flag);
        }
        writer.string(&self.category)?;
        writer.string(&self.lua_call)?;
        writer.count(self.actions.len())?;
        for action in &self.actions {
            writer.i32(action.map_key);
            writer.i32(action.id);
            writer.string(&action.name)?;
            writer.i32(action.requirement_id);
            writer.i32(action.unknown4);
        }
        writer.i32(self.composite_effect_id);
        writer.f32(self.unknown14)?;
        writer.f32(self.unknown15)?;
        for flag in self.flags2 {
            writer.bool(flag);
        }
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        let id = reader.i32()?;
        let item_definition_id = reader.i32()?;
        let unknown3 = reader.i32()?;
        let model_id = reader.i32()?;
        let mut flags = [false; 6];
        for flag in &mut flags {
            *flag = reader.bool()?;
        }
        let category = reader.string()?;
        let lua_call = reader.string()?;
        let count = reader.count()?;
        let mut actions = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            actions.push(FixtureAction {
                map_key: reader.i32()?,
                id: reader.i32()?,
                name: reader.string()?,
                requirement_id: reader.i32()?,
                unknown4: reader.i32()?,
            });
        }
        let composite_effect_id = reader.i32()?;
        let unknown14 = reader.f32()?;
        let unknown15 = reader.f32()?;
        let mut flags2 = [false; 4];
        for flag in &mut flags2 {
            *flag = reader.bool()?;
        }
        Ok(Self {
            id,
            item_definition_id,
            unknown3,
            model_id,
            flags,
            category,
            lua_call,
            actions,
            composite_effect_id,
            unknown14,
            unknown15,
            flags2,
        })
    }
}

/// S2C 127/40 client-reader projection. The complete server payload is not recovered; positional
/// list and trailing words remain unnamed, and the local codec's placeholders are not retail evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureUpdate {
    pub instance: FixtureInstance,
    pub info: FixtureInstanceInfo,
    pub definition: FixtureDefinition,
    pub unknown_guids: Vec<u64>,
    pub words: [u32; 3],
}

impl FixtureUpdate {
    pub const TAIL_BYTES_EMPTY: usize = 94;

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_FIXTURE_UPDATE);
        self.encode_record(&mut writer)?;
        Ok(writer.finish())
    }

    fn encode_record(&self, writer: &mut Writer) -> io::Result<()> {
        self.instance.encode(writer)?;
        self.info.encode(writer)?;
        self.definition.encode(writer)?;
        writer.count(self.unknown_guids.len())?;
        for guid in &self.unknown_guids {
            writer.u64(*guid);
        }
        for word in self.words {
            writer.u32(word);
        }
        Ok(())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_FIXTURE_UPDATE {
            return Err(invalid("not a 127/40 fixture update"));
        }
        let update = Self::decode_record(&mut reader)?;
        reader.finish()?;
        Ok(update)
    }

    fn decode_record(reader: &mut Reader<'_>) -> io::Result<Self> {
        let instance = FixtureInstance::decode(reader)?;
        let info = FixtureInstanceInfo::decode(reader)?;
        let definition = FixtureDefinition::decode(reader)?;
        let count = reader.count()?;
        let mut unknown_guids = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            unknown_guids.push(reader.u64()?);
        }
        Ok(Self {
            instance,
            info,
            definition,
            unknown_guids,
            words: [reader.u32()?, reader.u32()?, reader.u32()?],
        })
    }
}

/// S2C 127/41 `HousingPacketFixtureRemove` (10 bytes with the subtype).
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureRemove {
    pub fixture_guid: u64,
}

impl FixtureRemove {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_FIXTURE_REMOVE);
        writer.u64(self.fixture_guid);
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_FIXTURE_REMOVE {
            return Err(invalid("not a 127/41 remove payload"));
        }
        let remove = Self {
            fixture_guid: reader.u64()?,
        };
        reader.finish()?;
        Ok(remove)
    }
}

/// S2C 127/44 `HousingPacketUpdateHouseInfo` (22 bytes with the subtype, fixed codec fields;
/// per-instance values and the meaning of `unused` are not fully recovered).
#[derive(Clone, Debug, PartialEq)]
pub struct HouseInfo {
    pub in_edit_mode: bool,
    pub unused: i32,
    pub is_locked: bool,
    pub is_flora_allowed: bool,
    pub pet_autospawn: bool,
    pub cur_fixture_count: i32,
    pub cur_landmark_count: i32,
    pub furniture_score: i32,
}

impl HouseInfo {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_HOUSE_INFO);
        writer.bool(self.in_edit_mode);
        writer.i32(self.unused);
        writer.bool(self.is_locked);
        writer.bool(self.is_flora_allowed);
        writer.bool(self.pet_autospawn);
        writer.i32(self.cur_fixture_count);
        writer.i32(self.cur_landmark_count);
        writer.i32(self.furniture_score);
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_HOUSE_INFO {
            return Err(invalid("not a 127/44 house-info payload"));
        }
        let info = Self {
            in_edit_mode: reader.bool()?,
            unused: reader.i32()?,
            is_locked: reader.bool()?,
            is_flora_allowed: reader.bool()?,
            pet_autospawn: reader.bool()?,
            cur_fixture_count: reader.i32()?,
            cur_landmark_count: reader.i32()?,
            furniture_score: reader.i32()?,
        };
        reader.finish()?;
        Ok(info)
    }
}

/// Client `FUN_00aafe10` record embedded in 127/45. Field order is recovered; values other than
/// the owner/lot identifiers and counts supplied by the service remain reconstructed (R).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerHousingInstanceInfo {
    pub owner_guid: u64,
    pub instance_guid: u64,
    pub name_id: i32,
    pub house_name: String,
    pub owner_name: String,
    pub icon_id: i32,
    pub fixture_count: i32,
    pub furniture_score: i32,
    pub last_visited: [u32; 2],
    pub is_locked: bool,
    pub is_members_only: bool,
    pub is_flora_allowed: bool,
    pub description: String,
    pub keyword_list: String,
    pub unknown21: String,
    pub rating: f32,
    pub votes: i32,
    pub has_rating: bool,
    pub can_vote: bool,
    pub factory_plot_id: i32,
    pub when_created: [u32; 2],
}

impl PlayerHousingInstanceInfo {
    fn encode(&self, writer: &mut Writer) -> io::Result<()> {
        writer.u64(self.owner_guid);
        writer.u64(self.instance_guid);
        writer.i32(self.name_id);
        writer.string(&self.house_name)?;
        writer.string(&self.owner_name)?;
        writer.i32(self.icon_id);
        writer.i32(self.fixture_count);
        writer.u32(self.last_visited[0]);
        writer.u32(self.last_visited[1]);
        writer.i32(self.furniture_score);
        writer.bool(self.is_locked);
        writer.bool(self.is_members_only);
        writer.bool(self.is_flora_allowed);
        writer.string(&self.description)?;
        writer.string(&self.keyword_list)?;
        writer.string(&self.unknown21)?;
        writer.f32(self.rating)?;
        writer.i32(self.votes);
        writer.bool(self.has_rating);
        writer.bool(self.can_vote);
        writer.i32(self.factory_plot_id);
        writer.u32(self.when_created[0]);
        writer.u32(self.when_created[1]);
        Ok(())
    }

    fn decode(reader: &mut Reader<'_>) -> io::Result<Self> {
        Ok(Self {
            owner_guid: reader.u64()?,
            instance_guid: reader.u64()?,
            name_id: reader.i32()?,
            house_name: reader.string()?,
            owner_name: reader.string()?,
            icon_id: reader.i32()?,
            fixture_count: reader.i32()?,
            last_visited: [reader.u32()?, reader.u32()?],
            furniture_score: reader.i32()?,
            is_locked: reader.bool()?,
            is_members_only: reader.bool()?,
            is_flora_allowed: reader.bool()?,
            description: reader.string()?,
            keyword_list: reader.string()?,
            unknown21: reader.string()?,
            rating: reader.f32()?,
            votes: reader.i32()?,
            has_rating: reader.bool()?,
            can_vote: reader.bool()?,
            factory_plot_id: reader.i32()?,
            when_created: [reader.u32()?, reader.u32()?],
        })
    }
}

/// S2C 127/45. The two flags and word are parsed at `00ab6c80`; their meanings/default values are
/// not recovered and remain caller-supplied reconstructed policy (R).
#[derive(Clone, Debug, PartialEq)]
pub struct ZoneData {
    pub flag_a: bool,
    pub flag_b: bool,
    pub unknown_word: i32,
    pub instance: PlayerHousingInstanceInfo,
}

impl ZoneData {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_ZONE_DATA);
        writer.bool(self.flag_a);
        writer.bool(self.flag_b);
        writer.i32(self.unknown_word);
        self.instance.encode(&mut writer)?;
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_ZONE_DATA {
            return Err(invalid("not a 127/45 zone-data payload"));
        }
        let value = Self {
            flag_a: reader.bool()?,
            flag_b: reader.bool()?,
            unknown_word: reader.i32()?,
            instance: PlayerHousingInstanceInfo::decode(&mut reader)?,
        };
        reader.finish()?;
        Ok(value)
    }
}

/// One keyed permission record in 127/28. `key`, pair words and level are encoded as read by
/// `00ab8d10` -> `00ab3950`; interpreting them as permissions is a Legacy-shaped reconstructed
/// projection (R).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HousePermissionRecord {
    pub key: u32,
    pub pair: [u32; 2],
    pub level: u32,
}

/// S2C 127/28 client `00ac9dd0` housing instance record. The reader fixes this complete field order.
/// The zero/empty values selected by the service, and the six-to-eight-float bounds conversion,
/// are reconstructed policy (R); the optional factory payload is deliberately fail-closed.
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceData {
    pub house_guid: u64,
    pub owner_guid: u64,
    pub owner_name: String,
    pub unknown4: u64,
    pub name_id: u32,
    pub name: String,
    pub unknown7: u32,
    pub max_fixtures: u32,
    pub max_landmarks: u32,
    pub fixtures: Vec<(u32, FixtureUpdate)>,
    pub permissions: Vec<HousePermissionRecord>,
    pub is_locked: bool,
    pub is_flora_allowed: bool,
    pub cur_fixture_count: u32,
    pub cur_landmark_count: u32,
    pub unknown14: bool,
    pub unknown15: u32,
    pub build_areas: Vec<[[f32; 4]; 2]>,
    pub icon_id: u32,
    pub preview: bool,
    pub unknown18: bool,
    pub furniture_score: u32,
    pub unknown20: u64,
    pub is_members_only: bool,
    pub unknown22: String,
    pub unknown23: String,
    pub unknown24: bool,
}

impl InstanceData {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_INSTANCE_DATA);
        writer.u64(self.house_guid);
        writer.u64(self.owner_guid);
        writer.string(&self.owner_name)?;
        writer.u64(self.unknown4);
        writer.u32(self.name_id);
        writer.string(&self.name)?;
        writer.u32(self.unknown7);
        writer.u32(self.max_fixtures);
        writer.u32(self.max_landmarks);
        writer.count(self.fixtures.len())?;
        for (key, fixture) in &self.fixtures {
            writer.u32(*key);
            fixture.encode_record(&mut writer)?;
        }
        writer.count(self.permissions.len())?;
        for permission in &self.permissions {
            writer.u32(permission.key);
            writer.u32(permission.pair[0]);
            writer.u32(permission.pair[1]);
            writer.u32(permission.level);
        }
        writer.bool(self.is_locked);
        writer.bool(self.is_flora_allowed);
        writer.u32(self.cur_fixture_count);
        writer.u32(self.cur_landmark_count);
        writer.bool(self.unknown14);
        writer.u32(self.unknown15);
        writer.count(self.build_areas.len())?;
        for [minimum, maximum] in &self.build_areas {
            writer.f32x4(*minimum)?;
            writer.f32x4(*maximum)?;
        }
        writer.u32(self.icon_id);
        writer.bool(self.preview);
        writer.bool(self.unknown18);
        writer.u32(self.furniture_score);
        writer.u64(self.unknown20);
        writer.bool(self.is_members_only);
        writer.string(&self.unknown22)?;
        writer.string(&self.unknown23)?;
        writer.bool(self.unknown24);
        writer.bool(false); // R: no factory instance data is available in this model.
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_INSTANCE_DATA {
            return Err(invalid("not a 127/28 instance-data payload"));
        }
        let house_guid = reader.u64()?;
        let owner_guid = reader.u64()?;
        let owner_name = reader.string()?;
        let unknown4 = reader.u64()?;
        let name_id = reader.u32()?;
        let name = reader.string()?;
        let unknown7 = reader.u32()?;
        let max_fixtures = reader.u32()?;
        let max_landmarks = reader.u32()?;
        let fixture_count = reader.count()?;
        let mut fixtures = Vec::with_capacity(fixture_count.min(64));
        for _ in 0..fixture_count {
            fixtures.push((reader.u32()?, FixtureUpdate::decode_record(&mut reader)?));
        }
        let permission_count = reader.count()?;
        let mut permissions = Vec::with_capacity(permission_count.min(64));
        for _ in 0..permission_count {
            permissions.push(HousePermissionRecord {
                key: reader.u32()?,
                pair: [reader.u32()?, reader.u32()?],
                level: reader.u32()?,
            });
        }
        let is_locked = reader.bool()?;
        let is_flora_allowed = reader.bool()?;
        let cur_fixture_count = reader.u32()?;
        let cur_landmark_count = reader.u32()?;
        let unknown14 = reader.bool()?;
        let unknown15 = reader.u32()?;
        let area_count = reader.count()?;
        let mut build_areas = Vec::with_capacity(area_count.min(64));
        for _ in 0..area_count {
            build_areas.push([reader.f32x4()?, reader.f32x4()?]);
        }
        let value = Self {
            house_guid,
            owner_guid,
            owner_name,
            unknown4,
            name_id,
            name,
            unknown7,
            max_fixtures,
            max_landmarks,
            fixtures,
            permissions,
            is_locked,
            is_flora_allowed,
            cur_fixture_count,
            cur_landmark_count,
            unknown14,
            unknown15,
            build_areas,
            icon_id: reader.u32()?,
            preview: reader.bool()?,
            unknown18: reader.bool()?,
            furniture_score: reader.u32()?,
            unknown20: reader.u64()?,
            is_members_only: reader.bool()?,
            unknown22: reader.string()?,
            unknown23: reader.string()?,
            unknown24: reader.bool()?,
        };
        if reader.bool()? {
            return Err(invalid("127/28 factory instance payload is unsupported"));
        }
        reader.finish()?;
        Ok(value)
    }
}

/// Client 127/42 fixture asset record from `00ac1f50`. Field positions are recovered; opaque
/// strings/words and their values are represented without inferred semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureAsset {
    pub unknown_word: u32,
    pub selection_id: u32,
    pub definition: FixtureDefinition,
    pub bytes_a: String,
    pub bytes_b: String,
    pub unknown_a: u32,
    pub unknown_b: u32,
    pub bytes_c: String,
    pub guids: Vec<u64>,
    pub unknown_c: u32,
    pub unknown_d: u32,
    pub flag: bool,
}

impl FixtureAsset {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_FIXTURE_ASSET);
        writer.u32(self.unknown_word);
        writer.u32(self.selection_id);
        self.definition.encode(&mut writer)?;
        writer.string(&self.bytes_a)?;
        writer.string(&self.bytes_b)?;
        writer.u32(self.unknown_a);
        writer.u32(self.unknown_b);
        writer.string(&self.bytes_c)?;
        writer.count(self.guids.len())?;
        for guid in &self.guids {
            writer.u64(*guid);
        }
        writer.u32(self.unknown_c);
        writer.u32(self.unknown_d);
        writer.bool(self.flag);
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_FIXTURE_ASSET {
            return Err(invalid("not a 127/42 fixture-asset payload"));
        }
        let unknown_word = reader.u32()?;
        let selection_id = reader.u32()?;
        let definition = FixtureDefinition::decode(&mut reader)?;
        let bytes_a = reader.string()?;
        let bytes_b = reader.string()?;
        let unknown_a = reader.u32()?;
        let unknown_b = reader.u32()?;
        let bytes_c = reader.string()?;
        let count = reader.count()?;
        let mut guids = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            guids.push(reader.u64()?);
        }
        let value = Self {
            unknown_word,
            selection_id,
            definition,
            bytes_a,
            bytes_b,
            unknown_a,
            unknown_b,
            bytes_c,
            guids,
            unknown_c: reader.u32()?,
            unknown_d: reader.u32()?,
            flag: reader.bool()?,
        };
        reader.finish()?;
        Ok(value)
    }
}

/// Client 127/43: counted fixture-info records, fixture definitions and dword values, in the order
/// read by `00ac9a90`. Which list represents an inventory selector catalogue is not established.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureItemList {
    pub items: Vec<FixtureInstanceInfo>,
    pub definitions: Vec<FixtureDefinition>,
    pub words: Vec<u32>,
}

impl FixtureItemList {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_FIXTURE_ITEM_LIST);
        writer.count(self.items.len())?;
        for item in &self.items {
            item.encode(&mut writer)?;
        }
        writer.count(self.definitions.len())?;
        for definition in &self.definitions {
            definition.encode(&mut writer)?;
        }
        writer.count(self.words.len())?;
        for word in &self.words {
            writer.u32(*word);
        }
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_FIXTURE_ITEM_LIST {
            return Err(invalid("not a 127/43 fixture-item-list payload"));
        }
        let count = reader.count()?;
        let mut items = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            items.push(FixtureInstanceInfo::decode(&mut reader)?);
        }
        let count = reader.count()?;
        let mut definitions = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            definitions.push(FixtureDefinition::decode(&mut reader)?);
        }
        let count = reader.count()?;
        let mut words = Vec::with_capacity(count.min(64));
        for _ in 0..count {
            words.push(reader.u32()?);
        }
        reader.finish()?;
        Ok(Self {
            items,
            definitions,
            words,
        })
    }
}

/// Client 127/51 transform update from `00ab2d10`: guid followed by two float4 values. The names
/// position/orientation are local projections of fields whose semantics are not fully proven.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionUpdate {
    pub fixture_guid: u64,
    pub position: [f32; 4],
    pub orientation: [f32; 4],
}

impl PositionUpdate {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_POSITION);
        writer.u64(self.fixture_guid);
        writer.f32x4(self.position)?;
        writer.f32x4(self.orientation)?;
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_POSITION {
            return Err(invalid("not a 127/51 position payload"));
        }
        let value = Self {
            fixture_guid: reader.u64()?,
            position: reader.f32x4()?,
            orientation: reader.f32x4()?,
        };
        reader.finish()?;
        Ok(value)
    }
}

/// S2C 127/53 has one length-prefixed string in native client reader `FUN_00ab3030`. Trailing-byte
/// rejection is a strict local parsing policy (R); the client reader alone does not establish that
/// the original server refused them.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacementDenied {
    pub message: String,
}

impl PlacementDenied {
    pub fn encode(&self) -> io::Result<Vec<u8>> {
        let mut writer = Writer::new(SUB_PLACEMENT_DENIED);
        writer.string(&self.message)?;
        Ok(writer.finish())
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        let mut reader = Reader::new(payload);
        if reader.subtype()? != SUB_PLACEMENT_DENIED {
            return Err(invalid("not a 127/53 placement-denied payload"));
        }
        let denied = Self {
            message: reader.string()?,
        };
        reader.finish()?;
        Ok(denied)
    }
}

/// One decoded subtype-level housing placement body; the base opcode 127 is excluded.
#[derive(Clone, Debug, PartialEq)]
pub enum HousingPacket {
    Place(PlaceFixture),
    InstanceData(InstanceData),
    FixtureUpdate(FixtureUpdate),
    FixtureRemove(FixtureRemove),
    FixtureAsset(FixtureAsset),
    FixtureItemList(FixtureItemList),
    HouseInfo(HouseInfo),
    ZoneData(ZoneData),
    PositionUpdate(PositionUpdate),
    PlacementDenied(PlacementDenied),
}

impl HousingPacket {
    /// Encode the inner game packet: family opcode 127, subtype, then fields.
    /// The gateway transport envelope is intentionally excluded.
    pub fn encode_inner(&self) -> io::Result<Vec<u8>> {
        let body = self.encode()?;
        let mut inner = Vec::with_capacity(body.len() + 2);
        inner.extend_from_slice(&127u16.to_le_bytes());
        inner.extend_from_slice(&body);
        Ok(inner)
    }

    /// Decode a complete inner game packet, including exactly one family prefix.
    pub fn decode_inner(inner: &[u8]) -> io::Result<Self> {
        if inner.len() < 4 || inner[..2] != 127u16.to_le_bytes() {
            return Err(invalid("not a complete opcode-127 housing packet"));
        }
        Self::decode(&inner[2..])
    }

    pub fn subtype(&self) -> u16 {
        match self {
            Self::Place(_) => SUB_PLACE,
            Self::InstanceData(_) => SUB_INSTANCE_DATA,
            Self::FixtureUpdate(_) => SUB_FIXTURE_UPDATE,
            Self::FixtureRemove(_) => SUB_FIXTURE_REMOVE,
            Self::FixtureAsset(_) => SUB_FIXTURE_ASSET,
            Self::FixtureItemList(_) => SUB_FIXTURE_ITEM_LIST,
            Self::HouseInfo(_) => SUB_HOUSE_INFO,
            Self::ZoneData(_) => SUB_ZONE_DATA,
            Self::PositionUpdate(_) => SUB_POSITION,
            Self::PlacementDenied(_) => SUB_PLACEMENT_DENIED,
        }
    }

    pub fn encode(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Place(value) => value.encode(),
            Self::InstanceData(value) => value.encode(),
            Self::FixtureUpdate(value) => value.encode(),
            Self::FixtureRemove(value) => value.encode(),
            Self::FixtureAsset(value) => value.encode(),
            Self::FixtureItemList(value) => value.encode(),
            Self::HouseInfo(value) => value.encode(),
            Self::ZoneData(value) => value.encode(),
            Self::PositionUpdate(value) => value.encode(),
            Self::PlacementDenied(value) => value.encode(),
        }
    }

    pub fn decode(payload: &[u8]) -> io::Result<Self> {
        if payload.len() < 2 {
            return Err(invalid("truncated housing subtype"));
        }
        match u16::from_le_bytes(payload[..2].try_into().unwrap()) {
            SUB_PLACE => Ok(Self::Place(PlaceFixture::decode(payload)?)),
            SUB_INSTANCE_DATA => Ok(Self::InstanceData(InstanceData::decode(payload)?)),
            SUB_FIXTURE_UPDATE => Ok(Self::FixtureUpdate(FixtureUpdate::decode(payload)?)),
            SUB_FIXTURE_REMOVE => Ok(Self::FixtureRemove(FixtureRemove::decode(payload)?)),
            SUB_FIXTURE_ASSET => Ok(Self::FixtureAsset(FixtureAsset::decode(payload)?)),
            SUB_FIXTURE_ITEM_LIST => Ok(Self::FixtureItemList(FixtureItemList::decode(payload)?)),
            SUB_HOUSE_INFO => Ok(Self::HouseInfo(HouseInfo::decode(payload)?)),
            SUB_ZONE_DATA => Ok(Self::ZoneData(ZoneData::decode(payload)?)),
            SUB_POSITION => Ok(Self::PositionUpdate(PositionUpdate::decode(payload)?)),
            SUB_PLACEMENT_DENIED => Ok(Self::PlacementDenied(PlacementDenied::decode(payload)?)),
            _ => Err(invalid("unknown housing subtype")),
        }
    }
}
