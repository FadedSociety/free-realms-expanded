//! Pure builders for startup catalogue and status packets.

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];

// (R) Packet 144 has no mapped body reader. Keep its captured body values fixed.
const GAME_SETTINGS_UNKNOWN_DWORD_0: u32 = 4;
const GAME_SETTINGS_UNKNOWN_DWORD_1: u32 = 7;
const GAME_SETTINGS_UNKNOWN_DWORD_2: u32 = 268;
const GAME_SETTINGS_UNKNOWN_FLAG: u8 = 1;
const GAME_SETTINGS_UNKNOWN_FLOAT_BITS: u32 = 0x3f80_0000;

// (R) Opcode 193's native dispatcher route is known, but this captured inner
// layout has not been matched to its announcement-family reader.
const UNIDENTIFIED_193_2_CAPTURE_REST: [u8; 62] = [
    0x02, 0x01, 0x00, 0x00, 0x00, 0x8b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x78, 0xba, 0x00,
    0x00, 0xa3, 0xde, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x77, 0x07, 0x00, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x45, 0x78, 0x69, 0x74, 0x05, 0x00, 0x00, 0x00, 0x50, 0x61, 0x72, 0x61, 0x6d, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// (R) These two encounter dwords are not interpreted by the recovered client
// reader; both retain their captured values.
const OVERWORLD_COMBAT_UNKNOWN_DWORD_0: u32 = 0;
const OVERWORLD_COMBAT_UNKNOWN_DWORD_1: u32 = 0;

fn tunnel(inner: &[u8]) -> Vec<u8> {
    let length = u32::try_from(inner.len()).expect("startup packet exceeds u32 length");
    let mut packet = Vec::with_capacity(TUNNEL_PREFIX.len() + 4 + inner.len());
    packet.extend_from_slice(&TUNNEL_PREFIX);
    packet.extend_from_slice(&length.to_le_bytes());
    packet.extend_from_slice(inner);
    packet
}

fn header(family: u16, sub_opcode: u16) -> Vec<u8> {
    [family.to_le_bytes(), sub_opcode.to_le_bytes()].concat()
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn count_i32(count: usize) -> i32 {
    i32::try_from(count).expect("startup list exceeds the client count field")
}

#[derive(Clone, Copy, Debug)]
pub struct ItemDefinition<'a> {
    pub id: i32,
    pub serialized_record: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct CoinStoreEntry {
    pub id: i32,
    pub unknown_words: [u32; 3],
    pub unknown_flags: [u8; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct CoinStoreItemLists<'a> {
    pub static_items: &'a [CoinStoreEntry],
    pub dynamic_items: &'a [CoinStoreEntry],
}

#[derive(Clone, Copy, Debug)]
pub struct AbilitySetEntry<'a> {
    pub kind: i32,
    pub serialized_fields: &'a [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct AbilitySetDefinition<'a> {
    pub id: i32,
    pub entries: &'a [AbilitySetEntry<'a>],
}

#[derive(Clone, Copy, Debug)]
pub struct EffectTag<'a> {
    pub id: i32,
    pub definition_blob: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StatValue {
    Integer(i32),
    Float(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterStat {
    pub id: i32,
    pub value: StatValue,
}

#[derive(Clone, Copy, Debug)]
pub struct StatUpdate<'a> {
    pub target_guid: u64,
    pub stats: &'a [CharacterStat],
}

/// Item definitions, family 35/sub-opcode 37.
///
/// The serialized records are kept in supplied order, including duplicate ids:
/// the startup capture contains 1,383 records while `Inventory::definitions_packet`
/// normalizes its content to 1,329 unique ids. The reader accepts back-to-back
/// definitions after one byte-length and one count (`0092f460`, `00a3db70`,
/// `008e3760`, `00919e10`); `00a3c290` caches/replaces definitions by id.
pub fn build_item_definitions(definitions: &[ItemDefinition<'_>]) -> Vec<u8> {
    let records_length = definitions.iter().fold(0usize, |length, definition| {
        assert!(definition.serialized_record.len() >= 4);
        assert_eq!(
            &definition.serialized_record[..4],
            definition.id.to_le_bytes().as_slice(),
            "item definition id differs from its serialized record"
        );
        length
            .checked_add(definition.serialized_record.len())
            .expect("item catalogue length overflow")
    });
    let payload_length = records_length
        .checked_add(4)
        .expect("item catalogue payload length overflow");
    let mut inner = header(35, 37);
    put_i32(&mut inner, count_i32(payload_length));
    put_i32(&mut inner, count_i32(definitions.len()));
    for definition in definitions {
        inner.extend_from_slice(definition.serialized_record);
    }
    tunnel(&inner)
}

/// Coin store family 165/sub-opcode 1: counted static and dynamic lists.
///
/// Each key is followed by three dwords and three bytes (`00c75fc0`,
/// `00c71510`). The client converts the latter to booleans; their meanings and
/// the dword meanings remain (R), so preserve their original byte values.
pub fn build_coin_store_item_lists(lists: CoinStoreItemLists<'_>) -> Vec<u8> {
    let mut inner = header(165, 1);
    write_coin_store_list(&mut inner, lists.static_items);
    write_coin_store_list(&mut inner, lists.dynamic_items);
    tunnel(&inner)
}

fn write_coin_store_list(out: &mut Vec<u8>, entries: &[CoinStoreEntry]) {
    put_i32(out, count_i32(entries.len()));
    for entry in entries {
        put_i32(out, entry.id);
        for word in entry.unknown_words {
            put_u32(out, word);
        }
        out.extend_from_slice(&entry.unknown_flags);
    }
}

/// Ability family 36/sub-opcode 5: set id and counted tagged slot records.
///
/// The reader initializes an eight-entry ability set, reads the set id, then a
/// count and kind-tagged records (`00a35cc0`, case 5; `00a32fe0`, `00a2ccf0`,
/// `008fa5c0`, `008e6760`). Kind 0 is an empty slot. Other kind-specific tails
/// remain (R) content bytes because their field meanings are not established.
pub fn build_ability_set_definition(definition: AbilitySetDefinition<'_>) -> Vec<u8> {
    let mut inner = header(36, 5);
    put_i32(&mut inner, definition.id);
    put_i32(&mut inner, count_i32(definition.entries.len()));
    for entry in definition.entries {
        assert!(
            (0..=3).contains(&entry.kind),
            "unsupported ability-set record kind"
        );
        if entry.kind == 0 {
            assert!(entry.serialized_fields.is_empty());
        }
        put_i32(&mut inner, entry.kind);
        inner.extend_from_slice(entry.serialized_fields);
    }
    tunnel(&inner)
}

/// Effect-tag update family 38/sub-opcode 16.
///
/// The packet contains an id and a length-prefixed definition blob. The client
/// routes the id and blob to its effect-tag map (`009392c0`, case 16; reader
/// `008d8870`; `00a19f00`/`00a19e70`). Inner blob field names remain (R).
pub fn build_effect_tag(tag: EffectTag<'_>) -> Vec<u8> {
    let mut inner = header(38, 16);
    put_i32(&mut inner, tag.id);
    put_i32(&mut inner, count_i32(tag.definition_blob.len()));
    inner.extend_from_slice(tag.definition_blob);
    tunnel(&inner)
}

/// Character stat update family 38/sub-opcode 7.
///
/// The two dwords identify the target GUID; a count follows, then records of
/// stat id, value kind (0 integer, 1 float), and value (`0091e1a0`, `0090a540`,
/// `008fc4c0`, `008e7640`). The client rejects other value kinds and NaN floats.
pub fn build_stat_update(update: StatUpdate<'_>) -> Vec<u8> {
    let mut inner = header(38, 7);
    inner.extend_from_slice(&update.target_guid.to_le_bytes());
    put_i32(&mut inner, count_i32(update.stats.len()));
    for stat in update.stats {
        put_i32(&mut inner, stat.id);
        match stat.value {
            StatValue::Integer(value) => {
                put_i32(&mut inner, 0);
                put_i32(&mut inner, value);
            }
            StatValue::Float(value) => {
                assert!(!value.is_nan(), "client stat reader rejects NaN");
                put_i32(&mut inner, 1);
                inner.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    tunnel(&inner)
}

/// Overworld combat flag family 41/sub-opcode 132.
///
/// The encounter reader consumes two unclassified captured dwords and a bool;
/// the bool reaches `SetInWorldCombatFlag` (`00aa36c0`, case 0x84, `00a9ed60`,
/// `009058d0`).
pub fn build_overworld_combat_flag(in_combat: bool) -> Vec<u8> {
    let mut inner = header(41, 132);
    put_u32(&mut inner, OVERWORLD_COMBAT_UNKNOWN_DWORD_0);
    put_u32(&mut inner, OVERWORLD_COMBAT_UNKNOWN_DWORD_1);
    inner.push(u8::from(in_combat));
    tunnel(&inner)
}

/// Packet 144. The captured 17-byte body has no mapped client reader.
pub fn build_client_game_settings() -> Vec<u8> {
    let mut inner = 144u16.to_le_bytes().to_vec();
    put_u32(&mut inner, GAME_SETTINGS_UNKNOWN_DWORD_0);
    put_u32(&mut inner, GAME_SETTINGS_UNKNOWN_DWORD_1);
    put_u32(&mut inner, GAME_SETTINGS_UNKNOWN_DWORD_2);
    inner.push(GAME_SETTINGS_UNKNOWN_FLAG);
    put_u32(&mut inner, GAME_SETTINGS_UNKNOWN_FLOAT_BITS);
    tunnel(&inner)
}

/// Opaque captured opcode 193 packet, retained until its announcement reader
/// is correlated with the 193/2 capture (`009471a0`, case 0xc1; `00a40aa0`).
pub fn build_unidentified_193_2() -> Vec<u8> {
    let mut inner = 193u16.to_le_bytes().to_vec();
    inner.extend_from_slice(&UNIDENTIFIED_193_2_CAPTURE_REST);
    tunnel(&inner)
}

/// Empty opcode 14 marker. Client dispatcher case 0xe writes `NetInfo.log` and
/// sets its +0x24 flag (`009471a0`); the marker's purpose remains unresolved.
pub fn build_opcode_14_marker() -> Vec<u8> {
    tunnel(&14u16.to_le_bytes())
}
