//! Typed builders for the captured world-content startup messages.
//!
//! Client field order follows the readers listed in `notes/Research/Startup World Content.md`.
//! Values whose meaning is not established by those readers are retained as `(R)` fields.

use serde::{Deserialize, Serialize};

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];
const MAX_ROWS: usize = 100_000;

fn tunnel(inner: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(7 + inner.len());
    out.extend_from_slice(&TUNNEL_PREFIX);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_i32(
        out,
        i32::try_from(value.len()).expect("wire string exceeds i32 length"),
    );
    out.extend_from_slice(value.as_bytes());
}

fn put_count(out: &mut Vec<u8>, count: usize) {
    put_i32(
        out,
        i32::try_from(count).expect("wire row count exceeds i32"),
    );
}

#[derive(Debug)]
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or_else(|| "reader offset overflow".to_owned())?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| format!("short read at {} for {length} bytes", self.offset))?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_bits(self.u32()?))
    }

    fn string(&mut self) -> Result<String, String> {
        let length = self.i32()?;
        if length < 0 {
            return Err(format!("negative string length {length}"));
        }
        String::from_utf8(self.take(length as usize)?.to_vec())
            .map_err(|error| format!("invalid UTF-8 string: {error}"))
    }

    fn count(&mut self) -> Result<usize, String> {
        let count = self.i32()?;
        if !(0..=MAX_ROWS as i32).contains(&count) {
            return Err(format!("invalid row count {count}"));
        }
        Ok(count as usize)
    }

    fn finish(self) -> Result<(), String> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(format!("{} trailing bytes", self.bytes.len() - self.offset))
        }
    }
}

fn expect_opcode(reader: &mut Reader<'_>, expected: u16) -> Result<(), String> {
    let actual = reader.u16()?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("opcode {actual}, expected {expected}"))
    }
}

fn expect_u16(reader: &mut Reader<'_>, expected: u16) -> Result<(), String> {
    let actual = reader.u16()?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("sub-opcode {actual}, expected {expected}"))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct QuickChatRow {
    /// Hash key read before the nine-dword row by `00b90750`.
    pub key: i32,
    /// The client reader consumes nine dwords per row. Its Lua consumer maps the first five.
    pub id: i32,
    pub display_string_id: i32,
    pub say_string_id: i32,
    pub animation_id: i32,
    pub parent_id: i32,
    // (R) The remaining row columns are retained; the client reader has not established their meaning.
    pub admin_only: i32,
    pub menu_icon_id: i32,
    pub item_id: i32,
    pub item_category: i32,
}

fn write_quick_chat_row(out: &mut Vec<u8>, row: &QuickChatRow) {
    for value in [
        row.key,
        row.id,
        row.display_string_id,
        row.say_string_id,
        row.animation_id,
        row.parent_id,
        row.admin_only,
        row.menu_icon_id,
        row.item_id,
        row.item_category,
    ] {
        put_i32(out, value);
    }
}

pub fn build_quick_chat(rows: &[QuickChatRow]) -> Vec<u8> {
    let mut inner = Vec::with_capacity(8 + rows.len() * 40);
    inner.extend_from_slice(&67u16.to_le_bytes());
    inner.extend_from_slice(&1u16.to_le_bytes());
    put_count(&mut inner, rows.len());
    for row in rows {
        write_quick_chat_row(&mut inner, row);
    }
    tunnel(&inner)
}

pub fn parse_quick_chat(bytes: &[u8]) -> Result<Vec<QuickChatRow>, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 67)?;
    expect_u16(&mut reader, 1)?;
    let count = reader.count()?;
    let mut rows = Vec::with_capacity(count);
    for _ in 0..count {
        rows.push(QuickChatRow {
            key: reader.i32()?,
            id: reader.i32()?,
            display_string_id: reader.i32()?,
            say_string_id: reader.i32()?,
            animation_id: reader.i32()?,
            parent_id: reader.i32()?,
            admin_only: reader.i32()?,
            menu_icon_id: reader.i32()?,
            item_id: reader.i32()?,
            item_category: reader.i32()?,
        });
    }
    reader.finish()?;
    Ok(rows)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PointOfInterest {
    pub id: u32,
    pub name_string_id: u32,
    pub location_id: u32,
    pub position: [f32; 3],
    pub heading: f32,
    pub icon_id: u32,
    pub notification_type: u32,
    pub sub_name_string_id: u32,
    // (R) These fields are read by the client but their server meaning is unresolved.
    pub unknown_field: u32,
    pub breadcrumb_quest_id: u32,
    pub teleport_location_id: u32,
    pub atlas_name: String,
    // (R) Client reader consumes this trailing dword without using it in the mapped display path.
    pub unknown_trailing_field: u32,
}

fn read_point_of_interest(reader: &mut Reader<'_>) -> Result<PointOfInterest, String> {
    Ok(PointOfInterest {
        id: reader.u32()?,
        name_string_id: reader.u32()?,
        location_id: reader.u32()?,
        position: [reader.f32()?, reader.f32()?, reader.f32()?],
        heading: reader.f32()?,
        icon_id: reader.u32()?,
        notification_type: reader.u32()?,
        sub_name_string_id: reader.u32()?,
        unknown_field: reader.u32()?,
        breadcrumb_quest_id: reader.u32()?,
        teleport_location_id: reader.u32()?,
        atlas_name: reader.string()?,
        unknown_trailing_field: reader.u32()?,
    })
}

fn write_point_of_interest(out: &mut Vec<u8>, row: &PointOfInterest) {
    for value in [row.id, row.name_string_id, row.location_id] {
        put_u32(out, value);
    }
    for value in row.position {
        put_f32(out, value);
    }
    put_f32(out, row.heading);
    for value in [
        row.icon_id,
        row.notification_type,
        row.sub_name_string_id,
        row.unknown_field,
        row.breadcrumb_quest_id,
        row.teleport_location_id,
    ] {
        put_u32(out, value);
    }
    put_string(out, &row.atlas_name);
    put_u32(out, row.unknown_trailing_field);
}

pub fn build_points_of_interest(rows: &[PointOfInterest]) -> Vec<u8> {
    let mut payload = Vec::new();
    for row in rows {
        payload.push(1);
        write_point_of_interest(&mut payload, row);
    }
    payload.push(0);

    let mut inner = Vec::with_capacity(6 + payload.len());
    inner.extend_from_slice(&57u16.to_le_bytes());
    put_i32(
        &mut inner,
        i32::try_from(payload.len()).expect("POI payload exceeds i32 length"),
    );
    inner.extend_from_slice(&payload);
    tunnel(&inner)
}

pub fn parse_points_of_interest(bytes: &[u8]) -> Result<Vec<PointOfInterest>, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 57)?;
    let payload_length = reader.i32()?;
    if payload_length < 0 {
        return Err(format!("negative POI payload length {payload_length}"));
    }
    let payload = reader.take(payload_length as usize)?;
    reader.finish()?;

    let mut payload_reader = Reader::new(payload);
    let mut rows = Vec::new();
    loop {
        match payload_reader.u8()? {
            0 => break,
            1 => rows.push(read_point_of_interest(&mut payload_reader)?),
            flag => return Err(format!("invalid POI row marker {flag}")),
        }
    }
    payload_reader.finish()?;
    Ok(rows)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JournalRegion {
    pub key: u32,
    // (R) Native reader proves seven dwords; per-column meanings remain unverified here.
    pub fields: [u32; 7],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JournalHub {
    pub key: u32,
    // (R) Native reader proves ten dwords; per-column meanings remain unverified here.
    pub fields: [u32; 10],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JournalHubQuest {
    pub key: u32,
    // (R) Native reader proves three dwords; per-column meanings remain unverified here.
    pub fields: [u32; 3],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JournalSticker {
    pub key: u32,
    // (R) Native reader proves nine dwords; per-column meanings remain unverified here.
    pub fields: [u32; 9],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JournalDefinitions {
    pub regions: Vec<JournalRegion>,
    pub hubs: Vec<JournalHub>,
    pub hub_quests: Vec<JournalHubQuest>,
    pub stickers: Vec<JournalSticker>,
}

fn read_dwords<const N: usize>(reader: &mut Reader<'_>) -> Result<[u32; N], String> {
    let mut fields = [0; N];
    for field in &mut fields {
        *field = reader.u32()?;
    }
    Ok(fields)
}

fn write_dwords(out: &mut Vec<u8>, fields: &[u32]) {
    for field in fields {
        put_u32(out, *field);
    }
}

fn read_journal_table<const N: usize>(
    reader: &mut Reader<'_>,
) -> Result<Vec<(u32, [u32; N])>, String> {
    let count = reader.count()?;
    (0..count)
        .map(|_| Ok((reader.u32()?, read_dwords::<N>(reader)?)))
        .collect()
}

fn write_journal_table<const N: usize>(
    out: &mut Vec<u8>,
    rows: impl IntoIterator<Item = (u32, [u32; N])>,
    count: usize,
) {
    put_count(out, count);
    for (key, fields) in rows {
        put_u32(out, key);
        write_dwords(out, &fields);
    }
}

pub fn build_journal_definitions(value: &JournalDefinitions) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.extend_from_slice(&209u16.to_le_bytes());
    put_u32(&mut inner, 1);
    write_journal_table(
        &mut inner,
        value.regions.iter().map(|row| (row.key, row.fields)),
        value.regions.len(),
    );
    write_journal_table(
        &mut inner,
        value.hubs.iter().map(|row| (row.key, row.fields)),
        value.hubs.len(),
    );
    write_journal_table(
        &mut inner,
        value.hub_quests.iter().map(|row| (row.key, row.fields)),
        value.hub_quests.len(),
    );
    write_journal_table(
        &mut inner,
        value.stickers.iter().map(|row| (row.key, row.fields)),
        value.stickers.len(),
    );
    tunnel(&inner)
}

pub fn parse_journal_definitions(bytes: &[u8]) -> Result<JournalDefinitions, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 209)?;
    let subtype = reader.u32()?;
    if subtype != 1 {
        return Err(format!("journal subtype {subtype}, expected 1"));
    }
    let regions = read_journal_table::<7>(&mut reader)?
        .into_iter()
        .map(|(key, fields)| JournalRegion { key, fields })
        .collect();
    let hubs = read_journal_table::<10>(&mut reader)?
        .into_iter()
        .map(|(key, fields)| JournalHub { key, fields })
        .collect();
    let hub_quests = read_journal_table::<3>(&mut reader)?
        .into_iter()
        .map(|(key, fields)| JournalHubQuest { key, fields })
        .collect();
    let stickers = read_journal_table::<9>(&mut reader)?
        .into_iter()
        .map(|(key, fields)| JournalSticker { key, fields })
        .collect();
    reader.finish()?;
    Ok(JournalDefinitions {
        regions,
        hubs,
        hub_quests,
        stickers,
    })
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WelcomeContentRow {
    // (R) The client reads a pair of dwords; their labels are not established by the native reader.
    pub first_string_id: u32,
    pub second_string_id: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WelcomeClaimCodeRow {
    pub code: String,
    // (R) Native reader proves three dwords between the two strings; exact display labels are unresolved.
    pub title_string_id: u32,
    pub description_string_id: u32,
    pub image_set_id: u32,
    pub tint_alias: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WelcomeScreen {
    pub show_welcome_screen: u8,
    pub contents: Vec<WelcomeContentRow>,
    pub claim_codes: Vec<WelcomeClaimCodeRow>,
    pub seconds_since_last_login: i32,
    pub starting_wallet_balance: i32,
}

pub fn build_welcome_screen(value: &WelcomeScreen) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.extend_from_slice(&93u16.to_le_bytes());
    inner.push(value.show_welcome_screen);
    put_count(&mut inner, value.contents.len());
    for row in &value.contents {
        put_u32(&mut inner, row.first_string_id);
        put_u32(&mut inner, row.second_string_id);
    }
    put_count(&mut inner, value.claim_codes.len());
    for row in &value.claim_codes {
        put_string(&mut inner, &row.code);
        put_u32(&mut inner, row.title_string_id);
        put_u32(&mut inner, row.description_string_id);
        put_u32(&mut inner, row.image_set_id);
        put_string(&mut inner, &row.tint_alias);
    }
    put_i32(&mut inner, value.seconds_since_last_login);
    put_i32(&mut inner, value.starting_wallet_balance);
    tunnel(&inner)
}

pub fn parse_welcome_screen(bytes: &[u8]) -> Result<WelcomeScreen, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 93)?;
    let show_welcome_screen = reader.u8()?;
    let content_count = reader.count()?;
    let mut contents = Vec::with_capacity(content_count);
    for _ in 0..content_count {
        contents.push(WelcomeContentRow {
            first_string_id: reader.u32()?,
            second_string_id: reader.u32()?,
        });
    }
    let claim_count = reader.count()?;
    let mut claim_codes = Vec::with_capacity(claim_count);
    for _ in 0..claim_count {
        claim_codes.push(WelcomeClaimCodeRow {
            code: reader.string()?,
            title_string_id: reader.u32()?,
            description_string_id: reader.u32()?,
            image_set_id: reader.u32()?,
            tint_alias: reader.string()?,
        });
    }
    let seconds_since_last_login = reader.i32()?;
    let starting_wallet_balance = reader.i32()?;
    reader.finish()?;
    Ok(WelcomeScreen {
        show_welcome_screen,
        contents,
        claim_codes,
        seconds_since_last_login,
        starting_wallet_balance,
    })
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Motd {
    // (R) Constructor 008ea4e0 proves two strings; their roles are not mapped by a client reader.
    pub first_string: String,
    pub second_string: String,
}

pub fn build_motd(value: &Motd) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.extend_from_slice(&87u16.to_le_bytes());
    put_string(&mut inner, &value.first_string);
    put_string(&mut inner, &value.second_string);
    tunnel(&inner)
}

pub fn parse_motd(bytes: &[u8]) -> Result<Motd, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 87)?;
    let first_string = reader.string()?;
    let second_string = reader.string()?;
    reader.finish()?;
    Ok(Motd {
        first_string,
        second_string,
    })
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NpcAttachment {
    // (R) Names are a peer-schema crosswalk; 008e86f0 proves three strings and three dwords.
    pub model_name: String,
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
    pub composite_effect_id: i32,
    pub slot: i32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NpcEffectTag {
    // (R) Nested effect-tag semantics remain unverified; captured NPC rows have zero tags.
    pub instance_id: i32,
    pub definition_id: i32,
    pub field_3: i32,
    pub field_4: i32,
    pub field_5: i32,
    pub field_6: i32,
    pub field_7: i32,
    pub flag_8: u8,
    pub field_9: i64,
    pub field_10: u32,
    pub field_11: u32,
    pub field_12: i32,
    pub composite_effect_id: i32,
    pub field_14: i64,
    pub field_15: i32,
    pub field_16: i32,
    pub flag_17: u8,
    pub flag_18: u8,
    pub flag_19: u8,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NpcCustomization {
    // (R) Names are a peer-schema crosswalk; 008e8510 proves the wire sequence and types.
    pub type_id: i32,
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
    pub texture_override: String,
}

/// A static actor spawn. Semantics not proved by the native reader are retained as raw typed fields.
// (R) Crosswalk labels follow the read-only peer schema; 00921a10 proves the native field sequence/types.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct NpcSpawn {
    pub actor_id: u64,
    pub name_string_id: i32,
    pub model_id: i32,
    // (R) These flag semantics are unknown; u8 retains the exact capture value.
    pub unknown_flag_1: u8,
    pub chat_bubble_foreground_color: i32,
    pub chat_bubble_background_color: i32,
    pub chat_bubble_size: i32,
    pub scale: f32,
    pub position: [f32; 4],
    pub rotation: [f32; 4],
    pub animation_id: i32,
    pub attachments: Vec<NpcAttachment>,
    pub disposition: i32,
    pub texture_alias: String,
    pub tint_alias: String,
    pub tint_id: i32,
    // (R) The reader consumes this flag but does not identify its server meaning.
    pub unknown_flag_16: u8,
    pub vertical_offset: f32,
    pub composite_effect_id: i32,
    pub wield_type: i32,
    pub npc_name: String,
    pub hide_nameplate: u8,
    // (R) Numeric field meanings are not assigned by reader 00921a10.
    pub unknown_float_22: f32,
    pub unknown_float_23: f32,
    pub unknown_float_24: f32,
    pub terrain_object_id: i32,
    pub has_attachments: u8,
    pub speed: f32,
    pub unknown_flag_28: u8,
    pub interact_range: i32,
    pub walk_animation_id: i32,
    pub run_animation_id: i32,
    pub stand_animation_id: i32,
    pub unknown_flag_33: u8,
    pub unknown_flag_34: u8,
    pub sub_text_name_string_id: i32,
    pub unknown_field_36: i32,
    pub temporary_appearance: i32,
    pub effect_tags: Vec<NpcEffectTag>,
    // (R) These reader fields retain captured bytes; native semantics are unresolved.
    pub unknown_flag_38: u8,
    pub unknown_field_39: i32,
    pub unknown_flag_40: u8,
    pub unknown_flag_41: u8,
    pub unknown_flag_42: u8,
    pub has_tilt: u8,
    pub customization: NpcCustomization,
    pub tilt: [f32; 4],
    pub name_color: i32,
    pub area_definition_id: i32,
    pub image_set_id: i32,
    pub is_interactable: u8,
    pub rider_id: u64,
    pub movement_type: i32,
    pub unknown_float_51: f32,
    // (R) Target payload meaning is unresolved; this dword is retained exactly.
    pub unknown_target_word: i32,
    pub character_variable_keys: Vec<u32>,
    // (R) Native field semantics are unresolved for this trailing scalar block.
    pub unknown_field_52: i32,
    pub unknown_float_53: f32,
    pub unknown_vector_54: [f32; 4],
    pub unknown_field_55: i32,
    pub unknown_float_56: f32,
    pub unknown_float_57: f32,
    pub unknown_float_58: f32,
    pub head: String,
    pub hair: String,
    pub model_customization: String,
    pub replace_terrain_object: u8,
    // (R) These trailing values are retained without recovered server semantics.
    pub unknown_field_63: i32,
    pub unknown_field_64: i32,
    pub fly_by_effect_id: i32,
    pub active_profile: i32,
    pub unknown_field_67: i32,
    pub unknown_field_68: i32,
    pub name_scale: f32,
    pub nameplate_image_id: i32,
}

fn read_f32_array<const N: usize>(reader: &mut Reader<'_>) -> Result<[f32; N], String> {
    let mut values = [0.0; N];
    for value in &mut values {
        *value = reader.f32()?;
    }
    Ok(values)
}

fn read_npc_attachment(reader: &mut Reader<'_>) -> Result<NpcAttachment, String> {
    Ok(NpcAttachment {
        model_name: reader.string()?,
        texture_alias: reader.string()?,
        tint_alias: reader.string()?,
        tint_id: reader.i32()?,
        composite_effect_id: reader.i32()?,
        slot: reader.i32()?,
    })
}

fn read_effect_tag(reader: &mut Reader<'_>) -> Result<NpcEffectTag, String> {
    Ok(NpcEffectTag {
        instance_id: reader.i32()?,
        definition_id: reader.i32()?,
        field_3: reader.i32()?,
        field_4: reader.i32()?,
        field_5: reader.i32()?,
        field_6: reader.i32()?,
        field_7: reader.i32()?,
        flag_8: reader.u8()?,
        field_9: i64::from_le_bytes(reader.take(8)?.try_into().unwrap()),
        field_10: reader.u32()?,
        field_11: reader.u32()?,
        field_12: reader.i32()?,
        composite_effect_id: reader.i32()?,
        field_14: i64::from_le_bytes(reader.take(8)?.try_into().unwrap()),
        field_15: reader.i32()?,
        field_16: reader.i32()?,
        flag_17: reader.u8()?,
        flag_18: reader.u8()?,
        flag_19: reader.u8()?,
    })
}

fn write_npc_attachment(out: &mut Vec<u8>, value: &NpcAttachment) {
    put_string(out, &value.model_name);
    put_string(out, &value.texture_alias);
    put_string(out, &value.tint_alias);
    put_i32(out, value.tint_id);
    put_i32(out, value.composite_effect_id);
    put_i32(out, value.slot);
}

fn write_effect_tag(out: &mut Vec<u8>, value: &NpcEffectTag) {
    for field in [
        value.instance_id,
        value.definition_id,
        value.field_3,
        value.field_4,
        value.field_5,
        value.field_6,
        value.field_7,
    ] {
        put_i32(out, field);
    }
    out.push(value.flag_8);
    out.extend_from_slice(&value.field_9.to_le_bytes());
    put_u32(out, value.field_10);
    put_u32(out, value.field_11);
    put_i32(out, value.field_12);
    put_i32(out, value.composite_effect_id);
    out.extend_from_slice(&value.field_14.to_le_bytes());
    put_i32(out, value.field_15);
    put_i32(out, value.field_16);
    out.extend_from_slice(&[value.flag_17, value.flag_18, value.flag_19]);
}

fn write_npc_body(out: &mut Vec<u8>, value: &NpcSpawn) {
    put_u64(out, value.actor_id);
    put_i32(out, value.name_string_id);
    put_i32(out, value.model_id);
    out.push(value.unknown_flag_1);
    put_i32(out, value.chat_bubble_foreground_color);
    put_i32(out, value.chat_bubble_background_color);
    put_i32(out, value.chat_bubble_size);
    put_f32(out, value.scale);
    for field in value.position {
        put_f32(out, field);
    }
    for field in value.rotation {
        put_f32(out, field);
    }
    put_i32(out, value.animation_id);
    put_count(out, value.attachments.len());
    for row in &value.attachments {
        write_npc_attachment(out, row);
    }
    put_i32(out, value.disposition);
    put_string(out, &value.texture_alias);
    put_string(out, &value.tint_alias);
    put_i32(out, value.tint_id);
    out.push(value.unknown_flag_16);
    put_f32(out, value.vertical_offset);
    put_i32(out, value.composite_effect_id);
    put_i32(out, value.wield_type);
    put_string(out, &value.npc_name);
    out.push(value.hide_nameplate);
    put_f32(out, value.unknown_float_22);
    put_f32(out, value.unknown_float_23);
    put_f32(out, value.unknown_float_24);
    put_i32(out, value.terrain_object_id);
    out.push(value.has_attachments);
    put_f32(out, value.speed);
    out.push(value.unknown_flag_28);
    put_i32(out, value.interact_range);
    put_i32(out, value.walk_animation_id);
    put_i32(out, value.run_animation_id);
    put_i32(out, value.stand_animation_id);
    out.extend_from_slice(&[value.unknown_flag_33, value.unknown_flag_34]);
    put_i32(out, value.sub_text_name_string_id);
    put_i32(out, value.unknown_field_36);
    put_i32(out, value.temporary_appearance);
    put_count(out, value.effect_tags.len());
    for row in &value.effect_tags {
        write_effect_tag(out, row);
    }
    out.push(value.unknown_flag_38);
    put_i32(out, value.unknown_field_39);
    out.extend_from_slice(&[
        value.unknown_flag_40,
        value.unknown_flag_41,
        value.unknown_flag_42,
        value.has_tilt,
    ]);
    put_i32(out, value.customization.type_id);
    put_string(out, &value.customization.texture_alias);
    put_string(out, &value.customization.tint_alias);
    put_i32(out, value.customization.tint_id);
    put_string(out, &value.customization.texture_override);
    for field in value.tilt {
        put_f32(out, field);
    }
    put_i32(out, value.name_color);
    put_i32(out, value.area_definition_id);
    put_i32(out, value.image_set_id);
    out.push(value.is_interactable);
    put_u64(out, value.rider_id);
    put_i32(out, value.movement_type);
    put_f32(out, value.unknown_float_51);
    put_i32(out, value.unknown_target_word);
    put_count(out, value.character_variable_keys.len());
    for key in &value.character_variable_keys {
        put_u32(out, *key);
    }
    put_i32(out, value.unknown_field_52);
    put_f32(out, value.unknown_float_53);
    for field in value.unknown_vector_54 {
        put_f32(out, field);
    }
    put_i32(out, value.unknown_field_55);
    put_f32(out, value.unknown_float_56);
    put_f32(out, value.unknown_float_57);
    put_f32(out, value.unknown_float_58);
    put_string(out, &value.head);
    put_string(out, &value.hair);
    put_string(out, &value.model_customization);
    out.push(value.replace_terrain_object);
    put_i32(out, value.unknown_field_63);
    put_i32(out, value.unknown_field_64);
    put_i32(out, value.fly_by_effect_id);
    put_i32(out, value.active_profile);
    put_i32(out, value.unknown_field_67);
    put_i32(out, value.unknown_field_68);
    put_f32(out, value.name_scale);
    put_i32(out, value.nameplate_image_id);
}

pub fn build_add_npc(value: &NpcSpawn) -> Vec<u8> {
    let mut inner = Vec::new();
    inner.extend_from_slice(&35u16.to_le_bytes());
    inner.extend_from_slice(&2u16.to_le_bytes());
    write_npc_body(&mut inner, value);
    tunnel(&inner)
}

pub fn parse_add_npc(bytes: &[u8]) -> Result<NpcSpawn, String> {
    let mut reader = Reader::new(bytes);
    expect_opcode(&mut reader, 35)?;
    expect_u16(&mut reader, 2)?;
    let actor_id = reader.u64()?;
    let name_string_id = reader.i32()?;
    let model_id = reader.i32()?;
    let unknown_flag_1 = reader.u8()?;
    let chat_bubble_foreground_color = reader.i32()?;
    let chat_bubble_background_color = reader.i32()?;
    let chat_bubble_size = reader.i32()?;
    let scale = reader.f32()?;
    let position = read_f32_array::<4>(&mut reader)?;
    let rotation = read_f32_array::<4>(&mut reader)?;
    let animation_id = reader.i32()?;
    let attachment_count = reader.count()?;
    let mut attachments = Vec::with_capacity(attachment_count);
    for _ in 0..attachment_count {
        attachments.push(read_npc_attachment(&mut reader)?);
    }
    let disposition = reader.i32()?;
    let texture_alias = reader.string()?;
    let tint_alias = reader.string()?;
    let tint_id = reader.i32()?;
    let unknown_flag_16 = reader.u8()?;
    let vertical_offset = reader.f32()?;
    let composite_effect_id = reader.i32()?;
    let wield_type = reader.i32()?;
    let npc_name = reader.string()?;
    let hide_nameplate = reader.u8()?;
    let unknown_float_22 = reader.f32()?;
    let unknown_float_23 = reader.f32()?;
    let unknown_float_24 = reader.f32()?;
    let terrain_object_id = reader.i32()?;
    let has_attachments = reader.u8()?;
    let speed = reader.f32()?;
    let unknown_flag_28 = reader.u8()?;
    let interact_range = reader.i32()?;
    let walk_animation_id = reader.i32()?;
    let run_animation_id = reader.i32()?;
    let stand_animation_id = reader.i32()?;
    let unknown_flag_33 = reader.u8()?;
    let unknown_flag_34 = reader.u8()?;
    let sub_text_name_string_id = reader.i32()?;
    let unknown_field_36 = reader.i32()?;
    let temporary_appearance = reader.i32()?;
    let effect_count = reader.count()?;
    let mut effect_tags = Vec::with_capacity(effect_count);
    for _ in 0..effect_count {
        effect_tags.push(read_effect_tag(&mut reader)?);
    }
    let unknown_flag_38 = reader.u8()?;
    let unknown_field_39 = reader.i32()?;
    let unknown_flag_40 = reader.u8()?;
    let unknown_flag_41 = reader.u8()?;
    let unknown_flag_42 = reader.u8()?;
    let has_tilt = reader.u8()?;
    let customization = NpcCustomization {
        type_id: reader.i32()?,
        texture_alias: reader.string()?,
        tint_alias: reader.string()?,
        tint_id: reader.i32()?,
        texture_override: reader.string()?,
    };
    let tilt = read_f32_array::<4>(&mut reader)?;
    let name_color = reader.i32()?;
    let area_definition_id = reader.i32()?;
    let image_set_id = reader.i32()?;
    let is_interactable = reader.u8()?;
    let rider_id = reader.u64()?;
    let movement_type = reader.i32()?;
    let unknown_float_51 = reader.f32()?;
    let unknown_target_word = reader.i32()?;
    let variable_count = reader.count()?;
    let mut character_variable_keys = Vec::with_capacity(variable_count);
    for _ in 0..variable_count {
        character_variable_keys.push(reader.u32()?);
    }
    let unknown_field_52 = reader.i32()?;
    let unknown_float_53 = reader.f32()?;
    let unknown_vector_54 = read_f32_array::<4>(&mut reader)?;
    let unknown_field_55 = reader.i32()?;
    let unknown_float_56 = reader.f32()?;
    let unknown_float_57 = reader.f32()?;
    let unknown_float_58 = reader.f32()?;
    let head = reader.string()?;
    let hair = reader.string()?;
    let model_customization = reader.string()?;
    let replace_terrain_object = reader.u8()?;
    let unknown_field_63 = reader.i32()?;
    let unknown_field_64 = reader.i32()?;
    let fly_by_effect_id = reader.i32()?;
    let active_profile = reader.i32()?;
    let unknown_field_67 = reader.i32()?;
    let unknown_field_68 = reader.i32()?;
    let name_scale = reader.f32()?;
    let nameplate_image_id = reader.i32()?;
    reader.finish()?;
    Ok(NpcSpawn {
        actor_id,
        name_string_id,
        model_id,
        unknown_flag_1,
        chat_bubble_foreground_color,
        chat_bubble_background_color,
        chat_bubble_size,
        scale,
        position,
        rotation,
        animation_id,
        attachments,
        disposition,
        texture_alias,
        tint_alias,
        tint_id,
        unknown_flag_16,
        vertical_offset,
        composite_effect_id,
        wield_type,
        npc_name,
        hide_nameplate,
        unknown_float_22,
        unknown_float_23,
        unknown_float_24,
        terrain_object_id,
        has_attachments,
        speed,
        unknown_flag_28,
        interact_range,
        walk_animation_id,
        run_animation_id,
        stand_animation_id,
        unknown_flag_33,
        unknown_flag_34,
        sub_text_name_string_id,
        unknown_field_36,
        temporary_appearance,
        effect_tags,
        unknown_flag_38,
        unknown_field_39,
        unknown_flag_40,
        unknown_flag_41,
        unknown_flag_42,
        has_tilt,
        customization,
        tilt,
        name_color,
        area_definition_id,
        image_set_id,
        is_interactable,
        rider_id,
        movement_type,
        unknown_float_51,
        unknown_target_word,
        character_variable_keys,
        unknown_field_52,
        unknown_float_53,
        unknown_vector_54,
        unknown_field_55,
        unknown_float_56,
        unknown_float_57,
        unknown_float_58,
        head,
        hair,
        model_customization,
        replace_terrain_object,
        unknown_field_63,
        unknown_field_64,
        fly_by_effect_id,
        active_profile,
        unknown_field_67,
        unknown_field_68,
        name_scale,
        nameplate_image_id,
    })
}
