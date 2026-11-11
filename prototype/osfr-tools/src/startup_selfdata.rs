//! Opcode 12 (`SendSelfToClient`) reader and serializer.
//!
//! Section boundaries and primitive widths below follow the native client
//! readers. Unnamed records remain bounded byte spans marked `(R)`; this is
//! capture preservation, not an assertion about server policy.
//!
//! A malformed packet, template or save is refused with `InvalidData`, never a panic
//! (`try_parse_self`, `try_build_self`). `parse_self` and `build_self` are the panicking
//! forms for tests and fixtures that hold known-good data.

use crate::le;
use crate::startup_snapshot::{decode_hex, encode_hex};
use osfr_tools::{inventory, quests};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];
const OPCODE: u16 = 12;
const REWARD_BUNDLE_FIXED_BYTES: usize = 60;
const ACTION_SLOT_BYTES: usize = 51;
const EFFECT_PAYLOAD_BYTES: usize = 81;

/// JSON form of the opaque byte spans (`[u8; N]` and `Vec<u8>` fields): one hex string each.
mod hex {
    use super::{decode_hex, encode_hex};
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(bytes: &impl AsRef<[u8]>, out: S) -> Result<S::Ok, S::Error> {
        out.serialize_str(&encode_hex(bytes.as_ref()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: TryFrom<Vec<u8>>>(
        input: D,
    ) -> Result<T, D::Error> {
        let bytes = decode_hex(&String::deserialize(input)?).map_err(D::Error::custom)?;
        T::try_from(bytes).map_err(|_| D::Error::custom("wrong byte count in a hex field"))
    }
}

/// The same, for a list of fixed-size spans.
mod hex_rows {
    use super::{decode_hex, encode_hex};
    use serde::{Deserialize, Deserializer, Serializer, de::Error, ser::SerializeSeq};

    pub fn serialize<S: Serializer, T: AsRef<[u8]>>(rows: &[T], out: S) -> Result<S::Ok, S::Error> {
        let mut seq = out.serialize_seq(Some(rows.len()))?;
        for row in rows {
            seq.serialize_element(&encode_hex(row.as_ref()))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: TryFrom<Vec<u8>>>(
        input: D,
    ) -> Result<Vec<T>, D::Error> {
        Vec::<String>::deserialize(input)?
            .iter()
            .map(|row| {
                let bytes = decode_hex(row).map_err(D::Error::custom)?;
                T::try_from(bytes).map_err(|_| D::Error::custom("wrong byte count in a hex row"))
            })
            .collect()
    }
}

/// (R) The local quest-content binding used by the gateway's original journal
/// patch. Its offset is relative to the quest span beginning after gender.
/// The reader does not name the quest ID or objective semantics.
const CAPTURE_QUEST_BINDINGS: [QuestWireBinding; 1] = [QuestWireBinding {
    quest_id: 1617,
    objective_index: 0,
    offset: 4112,
}];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelfData {
    pub identity: Identity,
    pub appearance: Appearance,
    pub transform: Transform,
    pub name: CharacterName,
    pub account: AccountState,
    pub jobs: JobState,
    pub profile_types: Vec<[u32; 2]>,
    /// Collection rows have native record/reward/keyed-row structure. Unnamed
    /// payloads stay exact bytes and are not assigned server-policy meaning.
    pub collections: Vec<CollectionRecord>,
    /// Shared server inventory type. Coins, items, and per-profile equipment
    /// all serialize from this state.
    pub inventory: inventory::InventoryState,
    pub gender: u32,
    /// Native quest-list structure with unnamed fixed fields retained as bytes.
    /// Bound counters are overlaid from `quests` using `quest_bindings`. The captured
    /// first list is the original character's; every login replaces it with the
    /// character's own (`Gameplay::write_self_quests`, T623).
    pub quest_records: QuestRecords,
    pub quests: quests::QuestState,
    /// (R) Bindings imported from the gateway's local quest content. Offsets
    /// are relative to `quest_records`; accepted/completed/rewarded flags are
    /// not encoded by this binding.
    pub quest_bindings: Vec<QuestWireBinding>,
    /// Native records between quest readers and action bars, with unnamed
    /// fixed fields retained as bounded bytes.
    pub post_quest_records: PostQuestRecords,
    pub action_bars: Vec<ActionBar>,
    /// (R) Unnamed byte read immediately after action bars.
    pub action_bar_flag: bool,
    pub action_keys: Vec<u32>,
    pub tutorial_keys: Vec<u32>,
    pub effects: Vec<EffectRecord>,
    /// (R) Exact bytes after the decoded effect records. The captured next
    /// count is 268, which cannot fit the native minimum row size in the span.
    #[serde(with = "hex")]
    pub post_effect_tail: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// (R) Captured value stored at client object +0x3c0; its meaning is unknown.
    pub unknown_identity_u64: u64,
    pub guid: u64,
    pub model: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    pub head: String,
    pub hair: String,
    pub hair_color: u32,
    pub eye_color: u32,
    pub skin_tone: String,
    pub face_paint: String,
    pub model_customization: String,
    pub customization_ids: [u32; 5],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// IEEE-754 component bit patterns, retained without float normalization.
    pub position: [u32; 4],
    /// IEEE-754 component bit patterns, retained without float normalization.
    pub rotation: [u32; 4],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CharacterName {
    pub fields: [u32; 3],
    pub first: String,
    pub last: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccountState {
    pub birthday: u64,
    pub age: u32,
    pub play_time: u32,
    pub underage: bool,
    pub open_chat: bool,
    pub membership_status: u32,
    pub show_member_nag: bool,
    pub chat_country_id: u32,
    pub chat_language_id: u32,
    pub preferred_language: u32,
    pub chat_language: u32,
    pub login_count: u32,
    pub grandfathered: bool,
    pub active_vehicle_loadouts: [u32; 2],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobState {
    pub profiles: Vec<JobProfile>,
    pub active_profile_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobProfile {
    /// The profile map key in the first wire dword, read by `00a18990`.
    pub map_key: u32,
    /// (R) Profile bytes before the equipment keyed list, with the map key
    /// removed so it can be changed independently.
    #[serde(with = "hex")]
    pub opaque_before_equipment_without_key: Vec<u8>,
    /// (R) Final dword read by `00911d60`; meaning unresolved.
    pub after_equipment_value: u32,
    /// `008fa5c0` count-prefixed tagged ability rows.
    pub abilities: Vec<ProfileAbility>,
    /// `008ca050` rows, ending at the zero-key sentinel.
    pub trailing_records: Vec<ProfileTrailingRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProfileAbility {
    pub kind: u32,
    /// (R) Conditional words read for kinds 1/3 (two) or 2 (one).
    pub kind_fields: Vec<u32>,
    /// (R) Seven common dwords; the fourth is consumed as a float bit pattern.
    /// These dwords are retained without assigning game meanings.
    pub common_fields: Option<[u32; 7]>,
    /// (R) Final bool byte, retained without normalizing nonzero values.
    pub common_flag: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProfileTrailingRecord {
    pub key: u32,
    /// (R) Native row payload: one flag byte and eight dwords.
    #[serde(with = "hex")]
    pub payload: [u8; 33],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CollectionRecord {
    /// Seven dwords consumed by `009099d0`; meanings remain unresolved (R).
    pub prefix: [u32; 7],
    pub reward: RewardBundle,
    pub keyed_rows: Vec<CollectionKeyedRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RewardBundle {
    /// Native success byte; retained without bool normalization.
    pub success: u8,
    /// Nine dwords, two u64 pairs and two scalars (`008e7930`); meanings R.
    #[serde(with = "hex")]
    pub fixed_fields: [u8; REWARD_BUNDLE_FIXED_BYTES],
    pub entries: Vec<RewardEntry>,
    /// Trailing dword consumed by the reward-bundle reader; meaning R.
    pub trailing_value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RewardEntry {
    pub kind: u32,
    /// Native hidden byte; retained without bool normalization.
    pub hidden: u8,
    /// (R) Paired IDs and four common dwords.
    #[serde(with = "hex")]
    pub common_fields: [u8; 24],
    /// Counted client string bytes; not assumed to be UTF-8.
    #[serde(with = "hex")]
    pub display_bytes: Vec<u8>,
    /// (R) Text color and member-only bytes.
    #[serde(with = "hex")]
    pub display_fields: [u8; 5],
    /// (R) Type/success-dependent final dword, when the reader consumes one.
    #[serde(with = "hex")]
    pub kind_fields: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CollectionKeyedRow {
    pub key: u32,
    /// Seven dwords and bool consumed by `008dacc0`; meanings R.
    #[serde(with = "hex")]
    pub payload: [u8; 29],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuestRecords {
    pub first: Vec<QuestFirstRecord>,
    /// (R) Fixed 17-byte tail after the first list.
    #[serde(with = "hex")]
    pub first_tail: [u8; 17],
    pub second: Vec<QuestSecondRecord>,
    /// The dword after the second list: the reader `00940e60` stores it at receiver+0x1100, which
    /// the inspect path exposes as the achievement-points value (GetValue 29). Not a constant.
    #[serde(with = "hex")]
    pub second_tail: [u8; 4],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuestFirstRecord {
    /// (R) Fixed 34-byte prefix read before the reward bundle.
    #[serde(with = "hex")]
    pub prefix: [u8; 34],
    pub reward: RewardBundle,
    pub objectives: Vec<QuestObjectiveRecord>,
    /// (R) Fixed six-byte record tail.
    #[serde(with = "hex")]
    pub tail: [u8; 6],
}

impl QuestFirstRecord {
    /// The record's first dword, the quest id (`0093c840`, runtime `+0`).
    pub fn quest_id(&self) -> u32 {
        let [a, b, c, d, ..] = self.prefix;
        u32::from_le_bytes([a, b, c, d])
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuestSecondRecord {
    /// (R) Fixed 33-byte prefix read before the reward bundle.
    #[serde(with = "hex")]
    pub prefix: [u8; 33],
    pub reward: RewardBundle,
    pub objectives: Vec<QuestObjectiveRecord>,
    /// (R) Fixed 21-byte record tail.
    #[serde(with = "hex")]
    pub tail: [u8; 21],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuestObjectiveRecord {
    pub key: u32,
    /// (R) Fixed 13-byte objective prefix.
    #[serde(with = "hex")]
    pub prefix: [u8; 13],
    pub reward: RewardBundle,
    /// (R) Fixed 21-byte objective tail.
    #[serde(with = "hex")]
    pub tail: [u8; 21],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PostQuestRecords {
    pub keyed_strings: Vec<KeyedStringRecord>,
    pub recipes: Vec<RecipeRecord>,
    pub variable_records: Vec<VariableRecord>,
    /// (R) Captured signed marker and u64 value before pre-action rows.
    pub pet_source_marker: i32,
    pub pet_source_value: u64,
    /// Misnamed: these are the owned-ride records, read by `008e74c0` like the rows of a 168/5
    /// (`00b05830`). They are still the captured rows; world entry replaces the client's list
    /// with a live 168/5 (`gameplay/rides.rs`). Rebuilding this section is open (T424).
    pub pre_action_records: Vec<PreActionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyedStringRecord {
    /// (R) Key pair read by `008e73b0`.
    #[serde(with = "hex")]
    pub key_fields: [u8; 8],
    #[serde(with = "hex")]
    pub value_bytes: Vec<u8>,
    /// (R) Two dwords and a flag.
    #[serde(with = "hex")]
    pub tail: [u8; 13],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecipeRecord {
    /// (R) Fixed fields read by `0090b930`.
    #[serde(with = "hex")]
    pub prefix: [u8; 29],
    #[serde(with = "hex_rows")]
    pub components: Vec<[u8; 32]>,
    /// (R) Fixed recipe tail.
    #[serde(with = "hex")]
    pub tail: [u8; 9],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct VariableRecord {
    /// (R) Fixed prefix before three nested arrays.
    #[serde(with = "hex")]
    pub prefix: [u8; 26],
    #[serde(with = "hex_rows")]
    pub first_rows: Vec<[u8; 37]>,
    pub second_rows: Vec<u32>,
    #[serde(with = "hex_rows")]
    pub third_rows: Vec<[u8; 12]>,
    #[serde(with = "hex")]
    pub first_string: Vec<u8>,
    /// (R) Fixed eight-byte field between counted strings.
    #[serde(with = "hex")]
    pub middle: [u8; 8],
    #[serde(with = "hex")]
    pub second_string: Vec<u8>,
    /// (R) Fixed tail read by `00912cf0`.
    #[serde(with = "hex")]
    pub tail: [u8; 58],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PreActionRecord {
    /// (R) Fixed fields before the counted string.
    #[serde(with = "hex")]
    pub prefix: [u8; 25],
    #[serde(with = "hex")]
    pub string_bytes: Vec<u8>,
    /// (R) Two bool bytes.
    #[serde(with = "hex")]
    pub tail: [u8; 2],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuestWireBinding {
    pub quest_id: u32,
    pub objective_index: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionBar {
    pub key: u32,
    /// Native `0091a9b0` reads this scalar before the slot-list reader.
    pub value: u32,
    pub slots: Vec<ActionBarSlot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionBarSlot {
    pub key: u32,
    /// (R) The native reader consumes 51 fixed bytes; field meanings remain
    /// unresolved, so the byte payload is preserved exactly.
    #[serde(with = "hex")]
    pub fields: [u8; ACTION_SLOT_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EffectRecord {
    pub key: u32,
    /// (R) Native `00a18cc0` consumes this 81-byte record; most fields are
    /// unnamed here and therefore remain exact bytes.
    #[serde(with = "hex")]
    pub payload: Vec<u8>,
}

#[derive(Clone)]
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    fn take(&mut self, len: usize) -> io::Result<&'a [u8]> {
        let truncated = || invalid("truncated self-data packet");
        let end = self.at.checked_add(len).ok_or_else(truncated)?;
        let part = self.bytes.get(self.at..end).ok_or_else(truncated)?;
        self.at = end;
        Ok(part)
    }

    fn array<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        self.take(N)?
            .first_chunk()
            .copied()
            .ok_or_else(|| invalid("truncated self-data packet"))
    }

    fn u8(&mut self) -> io::Result<u8> {
        let [byte] = self.array()?;
        Ok(byte)
    }

    fn u32(&mut self) -> io::Result<u32> {
        self.array().map(u32::from_le_bytes)
    }

    fn u32s<const N: usize>(&mut self) -> io::Result<[u32; N]> {
        let mut values = [0; N];
        for value in &mut values {
            *value = self.u32()?;
        }
        Ok(values)
    }

    fn i32(&mut self) -> io::Result<i32> {
        self.array().map(i32::from_le_bytes)
    }

    fn u64(&mut self) -> io::Result<u64> {
        self.array().map(u64::from_le_bytes)
    }

    fn counted_string(&mut self) -> io::Result<String> {
        let len = self.count(1)?;
        String::from_utf8(self.take(len)?.to_vec())
            .map_err(|error| invalid(format!("self-data string is not UTF-8: {error}")))
    }

    fn count(&mut self, minimum_item_size: usize) -> io::Result<usize> {
        let count_offset = self.at;
        let Ok(count) = usize::try_from(self.i32()?) else {
            return Err(invalid("negative self-data collection count"));
        };
        // `max(1)` keeps the divisor nonzero; every caller passes a row size of at least 1.
        if count > self.remaining() / minimum_item_size.max(1) {
            return Err(invalid(format!(
                "self-data count {count} at body offset {count_offset} exceeds remaining packet"
            )));
        }
        Ok(count)
    }

    /// `at` only moves to the end of a successful `get`, so it never passes the length.
    fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn put_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A u32 length or count field. `refusal` names the list for the (impossible in practice)
/// case of more than `u32::MAX` rows or bytes.
fn put_len(out: &mut Vec<u8>, len: usize, refusal: &'static str) -> io::Result<()> {
    let Ok(len) = u32::try_from(len) else {
        return Err(invalid(refusal));
    };
    put_u32(out, len);
    Ok(())
}

fn put_string(out: &mut Vec<u8>, value: &str) -> io::Result<()> {
    put_len(out, value.len(), "self-data string too long")?;
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn parse_counted_bytes(cursor: &mut Cursor<'_>) -> io::Result<Vec<u8>> {
    let len = cursor.count(1)?;
    Ok(cursor.take(len)?.to_vec())
}

/// The bytes a reward entry's type adds after its display fields (`008e7930`).
fn reward_kind_bytes(kind: u32, success: u8) -> io::Result<usize> {
    match kind {
        1 if success != 0 => Ok(4),
        1 | 3 | 12 => Ok(0),
        6 | 10 => Ok(4),
        _ => Err(invalid(format!("unmapped native reward-entry type {kind}"))),
    }
}

/// The conditional words a profile ability's kind adds (`008fa5c0`).
fn ability_kind_fields(kind: u32) -> usize {
    match kind {
        1 | 3 => 2,
        2 => 1,
        _ => 0,
    }
}

fn parse_reward_bundle(cursor: &mut Cursor<'_>) -> io::Result<RewardBundle> {
    let success = cursor.u8()?;
    let fixed_fields = cursor.array()?;
    let entries = cursor.count(38)?;
    let mut parsed_entries = Vec::with_capacity(entries);
    for _ in 0..entries {
        let kind = cursor.u32()?;
        let hidden = cursor.u8()?;
        let common_fields = cursor.array()?;
        let display_len = cursor.count(1)?;
        let display_bytes = cursor.take(display_len)?.to_vec();
        let display_fields = cursor.array()?;
        let kind_fields = cursor.take(reward_kind_bytes(kind, success)?)?.to_vec();
        parsed_entries.push(RewardEntry {
            kind,
            hidden,
            common_fields,
            display_bytes,
            display_fields,
            kind_fields,
        });
    }
    Ok(RewardBundle {
        success,
        fixed_fields,
        entries: parsed_entries,
        trailing_value: cursor.u32()?,
    })
}

fn write_reward_bundle(out: &mut Vec<u8>, bundle: &RewardBundle) -> io::Result<()> {
    put_u8(out, bundle.success);
    out.extend_from_slice(&bundle.fixed_fields);
    put_len(out, bundle.entries.len(), "too many reward entries")?;
    for entry in &bundle.entries {
        put_u32(out, entry.kind);
        put_u8(out, entry.hidden);
        out.extend_from_slice(&entry.common_fields);
        put_len(
            out,
            entry.display_bytes.len(),
            "reward display string too long",
        )?;
        out.extend_from_slice(&entry.display_bytes);
        out.extend_from_slice(&entry.display_fields);
        if entry.kind_fields.len() != reward_kind_bytes(entry.kind, bundle.success)? {
            return Err(invalid("reward-entry fields do not match its type"));
        }
        out.extend_from_slice(&entry.kind_fields);
    }
    put_u32(out, bundle.trailing_value);
    Ok(())
}

fn parse_objective_list(cursor: &mut Cursor<'_>) -> io::Result<Vec<QuestObjectiveRecord>> {
    // `0093c840`/`0093ca50`: keyed objective row, the `008fd770` prefix,
    // reward bundle, and the reader's fixed tail.
    let count = cursor.count(107)?;
    (0..count)
        .map(|_| {
            Ok(QuestObjectiveRecord {
                key: cursor.u32()?,
                prefix: cursor.array()?,
                reward: parse_reward_bundle(cursor)?,
                tail: cursor.array()?,
            })
        })
        .collect()
}

fn parse_profile(
    cursor: &mut Cursor<'_>,
    equipment: &mut Vec<inventory::EquippedItem>,
) -> io::Result<JobProfile> {
    let map_key = cursor.u32()?;
    let opaque_start = cursor.at;

    // `00911d60`: fixed prefix (after the key) and first keyed pair-list.
    cursor.take(8 * 4 + 1)?;
    let first_pairs = cursor.count(12)?;
    // `count(12)` proved `first_pairs * 12` fits in the remaining bytes.
    cursor.take(first_pairs * 12)?;
    cursor.take(1 + 8 + 1 + 6 * 4)?;
    let opaque_before_equipment_without_key = cursor
        .bytes
        .get(opaque_start..cursor.at)
        .ok_or_else(|| invalid("truncated self-data packet"))?
        .to_vec();

    // The second keyed pair-list is independently proven to be the profile
    // equipment span: its count and 12-byte rows match inventory.json spans
    // and `Inventory::profile_equipment` exactly.
    let equipment_count = cursor.count(12)?;
    let Ok(profile_id) = i32::try_from(map_key) else {
        return Err(invalid("profile map key outside server range"));
    };
    for _ in 0..equipment_count {
        equipment.push(inventory::EquippedItem {
            profile_id,
            slot: cursor.i32()?,
            item_id: cursor.i32()?,
            category: cursor.i32()?,
        });
    }
    let after_equipment_value = cursor.u32()?;

    // `008fa5c0` reads the count-prefixed ability vector. `00ae1f70` reserves
    // eight elements, but the serialized count controls the rows.
    let abilities = cursor.count(4)?;
    let mut ability_records = Vec::with_capacity(abilities);
    for _ in 0..abilities {
        let kind = cursor.u32()?;
        let kind_fields = (0..ability_kind_fields(kind))
            .map(|_| cursor.u32())
            .collect::<io::Result<_>>()?;
        let (common_fields, common_flag) = if kind == 0 {
            (None, None)
        } else {
            (Some(cursor.u32s()?), Some(cursor.u8()?))
        };
        ability_records.push(ProfileAbility {
            kind,
            kind_fields,
            common_fields,
            common_flag,
        });
    }

    // `008ca050` ends at the zero-key sentinel; rows contain a flag and eight
    // dwords after the key.
    let mut trailing_records = Vec::new();
    loop {
        let key = cursor.u32()?;
        if key == 0 {
            break;
        }
        trailing_records.push(ProfileTrailingRecord {
            key,
            payload: cursor.array()?,
        });
    }

    Ok(JobProfile {
        map_key,
        opaque_before_equipment_without_key,
        after_equipment_value,
        abilities: ability_records,
        trailing_records,
    })
}

fn write_profile(
    out: &mut Vec<u8>,
    profile: &JobProfile,
    equipment: &[inventory::EquippedItem],
) -> io::Result<()> {
    let before = &profile.opaque_before_equipment_without_key;
    put_u32(out, profile.map_key);
    out.extend_from_slice(before);

    let Ok(profile_id) = i32::try_from(profile.map_key) else {
        return Err(invalid("profile map key outside server range"));
    };
    let entries: Vec<_> = equipment
        .iter()
        .filter(|entry| entry.profile_id == profile_id)
        .collect();
    put_len(out, entries.len(), "too many equipped items")?;
    for entry in entries {
        put_i32(out, entry.slot);
        put_i32(out, entry.item_id);
        put_i32(out, entry.category);
    }
    put_u32(out, profile.after_equipment_value);
    put_len(out, profile.abilities.len(), "too many profile abilities")?;
    for ability in &profile.abilities {
        put_u32(out, ability.kind);
        if ability.kind_fields.len() != ability_kind_fields(ability.kind) {
            return Err(invalid("profile ability fields do not match its kind"));
        }
        for field in &ability.kind_fields {
            put_u32(out, *field);
        }
        match (ability.kind, ability.common_fields, ability.common_flag) {
            (0, None, None) => {}
            (1.., Some(fields), Some(flag)) => {
                for field in fields {
                    put_u32(out, field);
                }
                put_u8(out, flag);
            }
            _ => {
                return Err(invalid(
                    "profile ability native fields do not match its kind",
                ));
            }
        }
    }
    for record in &profile.trailing_records {
        if record.key == 0 {
            return Err(invalid("profile trailing key collides with sentinel"));
        }
        put_u32(out, record.key);
        out.extend_from_slice(&record.payload);
    }
    put_u32(out, 0);
    Ok(())
}

fn parse_inventory_item(cursor: &mut Cursor<'_>) -> io::Result<inventory::InventoryItem> {
    let definition_id = cursor.i32()?;
    let tint = cursor.i32()?;
    let id = cursor.i32()?;
    let count = cursor.u32()?;
    let consumed_count = cursor.u32()?;
    let last_cast_time = i64::from(cursor.i32()?);
    let rental = if cursor.u8()? == 0 {
        None
    } else {
        Some(inventory::RentalState {
            expiration: i64::from_le_bytes(cursor.array()?),
            upsells: cursor.i32()?,
            bundle_id: cursor.i32()?,
        })
    };
    let ability_count = cursor.i32()?;
    let activate_enabled = cursor.u8()? != 0;
    Ok(inventory::InventoryItem {
        definition_id,
        tint,
        id,
        count,
        consumed_count,
        // A captured template's last dword stays `ability_count` (P-banking: the client reads it
        // as the stored count, `InventoryItem::record_tail`); the live state owns the bank.
        stored_count: 0,
        last_cast_time,
        rental,
        ability_count,
        activate_enabled,
    })
}

fn write_inventory_item(out: &mut Vec<u8>, item: &inventory::InventoryItem) -> io::Result<()> {
    // A saved item's last-cast time: the inventory loader does not bound it to i32.
    let Ok(last_cast_time) = i32::try_from(item.last_cast_time) else {
        return Err(invalid(
            "self-data last-cast value exceeds its native i32 field",
        ));
    };
    for value in [item.definition_id, item.tint, item.id] {
        put_i32(out, value);
    }
    put_u32(out, item.count);
    put_u32(out, item.consumed_count);
    put_i32(out, last_cast_time);
    put_u8(out, u8::from(item.rental.is_some()));
    if let Some(rental) = &item.rental {
        out.extend_from_slice(&rental.expiration.to_le_bytes());
        put_i32(out, rental.upsells);
        put_i32(out, rental.bundle_id);
    }
    put_i32(out, item.record_tail());
    put_u8(out, u8::from(item.activate_enabled));
    Ok(())
}

fn parse_collection_rows(cursor: &mut Cursor<'_>) -> io::Result<Vec<CollectionRecord>> {
    let count = cursor.count(101)?;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let prefix = cursor.u32s()?;
        let reward = parse_reward_bundle(cursor)?;
        let keyed_count = cursor.count(33)?;
        let keyed_rows = (0..keyed_count)
            .map(|_| {
                Ok(CollectionKeyedRow {
                    key: cursor.u32()?,
                    payload: cursor.array()?,
                })
            })
            .collect::<io::Result<_>>()?;
        records.push(CollectionRecord {
            prefix,
            reward,
            keyed_rows,
        });
    }
    Ok(records)
}

/// `00940c60` (quest manager, player `+0x1add0`): the quest list, then the 17-byte tail
/// (completed count, selected quest, auto-switch byte, two quest limits).
fn parse_quest_journal(cursor: &mut Cursor<'_>) -> io::Result<(Vec<QuestFirstRecord>, [u8; 17])> {
    let first_count = cursor.count(113)?;
    let first = (0..first_count)
        .map(|_| {
            Ok(QuestFirstRecord {
                prefix: cursor.array()?,
                reward: parse_reward_bundle(cursor)?,
                objectives: parse_objective_list(cursor)?,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;
    Ok((first, cursor.array()?))
}

fn parse_quest_records(cursor: &mut Cursor<'_>) -> io::Result<QuestRecords> {
    let (first, first_tail) = parse_quest_journal(cursor)?;

    let second_count = cursor.count(127)?;
    let second = (0..second_count)
        .map(|_| {
            Ok(QuestSecondRecord {
                prefix: cursor.array()?,
                reward: parse_reward_bundle(cursor)?,
                objectives: parse_objective_list(cursor)?,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;
    let second_tail = cursor.array()?;
    Ok(QuestRecords {
        first,
        first_tail,
        second,
        second_tail,
    })
}

fn write_objective_list(out: &mut Vec<u8>, rows: &[QuestObjectiveRecord]) -> io::Result<()> {
    put_len(out, rows.len(), "too many quest objectives")?;
    for row in rows {
        put_u32(out, row.key);
        out.extend_from_slice(&row.prefix);
        write_reward_bundle(out, &row.reward)?;
        out.extend_from_slice(&row.tail);
    }
    Ok(())
}

fn write_quest_records(out: &mut Vec<u8>, records: &QuestRecords) -> io::Result<()> {
    put_len(out, records.first.len(), "too many first-list quests")?;
    for record in &records.first {
        out.extend_from_slice(&record.prefix);
        write_reward_bundle(out, &record.reward)?;
        write_objective_list(out, &record.objectives)?;
        out.extend_from_slice(&record.tail);
    }
    out.extend_from_slice(&records.first_tail);
    put_len(out, records.second.len(), "too many second-list quests")?;
    for record in &records.second {
        out.extend_from_slice(&record.prefix);
        write_reward_bundle(out, &record.reward)?;
        write_objective_list(out, &record.objectives)?;
        out.extend_from_slice(&record.tail);
    }
    out.extend_from_slice(&records.second_tail);
    Ok(())
}

fn parse_post_quest_records(cursor: &mut Cursor<'_>) -> io::Result<PostQuestRecords> {
    // `008e73b0`: key pair, counted string, two dwords and a flag.
    let strings = cursor.count(25)?;
    let keyed_strings = (0..strings)
        .map(|_| {
            Ok(KeyedStringRecord {
                key_fields: cursor.array()?,
                value_bytes: parse_counted_bytes(cursor)?,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;

    // `0090b930` recipe rows and fixed 32-byte component rows.
    let recipes = cursor.count(42)?;
    let recipes = (0..recipes)
        .map(|_| {
            let prefix = cursor.array()?;
            let components = cursor.count(32)?;
            let components = (0..components)
                .map(|_| cursor.array())
                .collect::<io::Result<_>>()?;
            Ok(RecipeRecord {
                prefix,
                components,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;

    // `00912cf0`: three variable nested arrays, two strings and a fixed tail.
    let variable_records = cursor.count(1)?;
    let variable_records = (0..variable_records)
        .map(|_| {
            let prefix = cursor.array()?;
            let first_count = cursor.count(37)?;
            let first_rows = (0..first_count)
                .map(|_| cursor.array())
                .collect::<io::Result<_>>()?;
            let second_count = cursor.count(4)?;
            let second_rows = (0..second_count)
                .map(|_| cursor.u32())
                .collect::<io::Result<_>>()?;
            let third_count = cursor.count(12)?;
            let third_rows = (0..third_count)
                .map(|_| cursor.array())
                .collect::<io::Result<_>>()?;
            let first_string = parse_counted_bytes(cursor)?;
            let middle = cursor.array()?;
            let second_string = parse_counted_bytes(cursor)?;
            Ok(VariableRecord {
                prefix,
                first_rows,
                second_rows,
                third_rows,
                first_string,
                middle,
                second_string,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;

    // The capture carries the signed -1 pet-source marker followed by a u64.
    let pet_source_marker = cursor.i32()?;
    let pet_source_value = cursor.u64()?;

    // `008e74c0`: three dwords, a pair, bool, dword, counted string, two bools.
    let pre_action_rows = cursor.count(31)?;
    let pre_action_records = (0..pre_action_rows)
        .map(|_| {
            Ok(PreActionRecord {
                prefix: cursor.array()?,
                string_bytes: parse_counted_bytes(cursor)?,
                tail: cursor.array()?,
            })
        })
        .collect::<io::Result<_>>()?;

    Ok(PostQuestRecords {
        keyed_strings,
        recipes,
        variable_records,
        pet_source_marker,
        pet_source_value,
        pre_action_records,
    })
}

fn write_counted_bytes(out: &mut Vec<u8>, value: &[u8]) -> io::Result<()> {
    put_len(out, value.len(), "self-data string too long")?;
    out.extend_from_slice(value);
    Ok(())
}

fn write_post_quest_records(out: &mut Vec<u8>, records: &PostQuestRecords) -> io::Result<()> {
    put_len(out, records.keyed_strings.len(), "too many keyed strings")?;
    for record in &records.keyed_strings {
        out.extend_from_slice(&record.key_fields);
        write_counted_bytes(out, &record.value_bytes)?;
        out.extend_from_slice(&record.tail);
    }
    put_len(out, records.recipes.len(), "too many recipes")?;
    for recipe in &records.recipes {
        out.extend_from_slice(&recipe.prefix);
        put_len(out, recipe.components.len(), "too many recipe components")?;
        for component in &recipe.components {
            out.extend_from_slice(component);
        }
        out.extend_from_slice(&recipe.tail);
    }
    put_len(
        out,
        records.variable_records.len(),
        "too many variable records",
    )?;
    for record in &records.variable_records {
        out.extend_from_slice(&record.prefix);
        put_len(out, record.first_rows.len(), "too many first nested rows")?;
        for row in &record.first_rows {
            out.extend_from_slice(row);
        }
        put_len(out, record.second_rows.len(), "too many second nested rows")?;
        for row in &record.second_rows {
            put_u32(out, *row);
        }
        put_len(out, record.third_rows.len(), "too many third nested rows")?;
        for row in &record.third_rows {
            out.extend_from_slice(row);
        }
        write_counted_bytes(out, &record.first_string)?;
        out.extend_from_slice(&record.middle);
        write_counted_bytes(out, &record.second_string)?;
        out.extend_from_slice(&record.tail);
    }
    put_i32(out, records.pet_source_marker);
    put_u64(out, records.pet_source_value);
    put_len(
        out,
        records.pre_action_records.len(),
        "too many pre-action records",
    )?;
    for record in &records.pre_action_records {
        out.extend_from_slice(&record.prefix);
        write_counted_bytes(out, &record.string_bytes)?;
        out.extend_from_slice(&record.tail);
    }
    Ok(())
}

fn parse_action_bars(cursor: &mut Cursor<'_>) -> io::Result<Vec<ActionBar>> {
    let count = cursor.count(12)?;
    let mut bars = Vec::with_capacity(count);
    for _ in 0..count {
        let key = cursor.u32()?;
        let value = cursor.u32()?;
        let slots_count = cursor.count(4 + ACTION_SLOT_BYTES)?;
        let mut slots = Vec::with_capacity(slots_count);
        for _ in 0..slots_count {
            let slot_key = cursor.u32()?;
            let fields = cursor.array()?;
            slots.push(ActionBarSlot {
                key: slot_key,
                fields,
            });
        }
        bars.push(ActionBar { key, value, slots });
    }
    Ok(bars)
}

fn state_from_quest_bindings(
    quest_records: &[u8],
    bindings: &[QuestWireBinding],
) -> quests::QuestState {
    let mut state = quests::QuestState {
        version: 1,
        quests: BTreeMap::new(),
        trial_best_ms: None,
        trial_completions: 0,
    };
    for binding in bindings {
        // A binding past the span, or a negative counter, carries no progress.
        let Some(Ok(value)) = le::i32(quest_records, binding.offset).map(u32::try_from) else {
            continue;
        };
        let Some(len) = binding.objective_index.checked_add(1) else {
            continue;
        };
        let progress =
            state
                .quests
                .entry(binding.quest_id)
                .or_insert_with(|| quests::QuestProgress {
                    // (R) These flags are not represented by the known wire binding.
                    accepted: false,
                    counts: Vec::new(),
                    completed: false,
                    rewarded: false,
                });
        progress.counts.resize(len, 0);
        if let Some(count) = progress.counts.get_mut(binding.objective_index) {
            *count = value;
        }
    }
    state
}

fn overlay_quest_state(
    quest_records: &mut [u8],
    state: &quests::QuestState,
    bindings: &[QuestWireBinding],
) -> io::Result<()> {
    for binding in bindings {
        let Some(progress) = state.quests.get(&binding.quest_id) else {
            continue;
        };
        let Some(value) = progress.counts.get(binding.objective_index) else {
            continue;
        };
        let Some(target) = quest_records
            .get_mut(binding.offset..)
            .and_then(<[u8]>::first_chunk_mut::<4>)
        else {
            continue;
        };
        let Ok(value) = i32::try_from(*value) else {
            return Err(invalid("quest progress exceeds native i32 field"));
        };
        *target = value.to_le_bytes();
    }
    Ok(())
}

/// Keep only the quest records `own` accepts, so a character never inherits the captured
/// character's journal. Bound counters are written in first: the bindings index the captured
/// span, so they are spent here. The tail's completed count and selection described the
/// captured character: they become `completed`, the first kept quest (or -1) and auto-switch
/// off, as the opening journal writes them (R); the two limits after them stay. A record that
/// cannot round-trip is refused and `data` is left unchanged.
pub fn retain_quests(
    data: &mut SelfData,
    own: impl Fn(u32) -> bool,
    completed: i32,
) -> io::Result<()> {
    let mut wire = Vec::new();
    write_quest_records(&mut wire, &data.quest_records)?;
    overlay_quest_state(&mut wire, &data.quests, &data.quest_bindings)?;
    let (mut first, mut tail) = parse_quest_journal(&mut Cursor::new(&wire))?;
    data.quest_bindings.clear();
    first.retain(|record| own(record.quest_id()));
    // The native field is an i32 quest id; u32::MAX is its -1 (no quest selected).
    let selected = first.first().map_or(u32::MAX, QuestFirstRecord::quest_id);
    tail[..4].copy_from_slice(&completed.to_le_bytes());
    tail[4..8].copy_from_slice(&selected.to_le_bytes());
    tail[8] = 0;
    data.quest_records.first = first;
    data.quest_records.first_tail = tail;
    Ok(())
}

/// Replace the quest list and its tail with a server-built `00940c60` span. The captured
/// bindings point into the replaced span, so they are dropped. A malformed span is refused
/// and `data` is left unchanged.
pub fn replace_quest_journal(data: &mut SelfData, span: &[u8]) -> io::Result<()> {
    let mut cursor = Cursor::new(span);
    let journal = parse_quest_journal(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(invalid("quest journal span has trailing bytes"));
    }
    (data.quest_records.first, data.quest_records.first_tail) = journal;
    data.quest_bindings.clear();
    Ok(())
}

/// `try_parse_self` for tests and fixtures with a known-good packet: panics on a malformed one.
pub fn parse_self(packet: &[u8]) -> SelfData {
    try_parse_self(packet).expect("test self-data packet is well formed")
}

/// `try_parse_self_with_quest_bindings` for tests and fixtures: panics on a malformed packet.
pub fn parse_self_with_quest_bindings(
    packet: &[u8],
    quest_bindings: &[QuestWireBinding],
) -> SelfData {
    try_parse_self_with_quest_bindings(packet, quest_bindings)
        .expect("test self-data packet is well formed")
}

/// Parse opcode 12 using the quest binding recovered by the existing local
/// journal content. Call `try_parse_self_with_quest_bindings` when a caller has a
/// different reviewed binding table.
pub fn try_parse_self(packet: &[u8]) -> io::Result<SelfData> {
    try_parse_self_with_quest_bindings(packet, &CAPTURE_QUEST_BINDINGS)
}

/// The span after `start`, when the u32 length at `length_at` covers exactly it.
fn length_prefixed(bytes: &[u8], length_at: usize, start: usize) -> Option<&[u8]> {
    let length = usize::try_from(le::u32(bytes, length_at)?).ok()?;
    bytes.get(start..).filter(|rest| rest.len() == length)
}

/// Parse opcode 12 while overlaying the supplied content bindings into the
/// shared `QuestState` representation.
pub fn try_parse_self_with_quest_bindings(
    packet: &[u8],
    quest_bindings: &[QuestWireBinding],
) -> io::Result<SelfData> {
    if packet.len() < 13 {
        return Err(invalid("short self-data tunnel"));
    }
    if le::bytes(packet, 0) != Some(TUNNEL_PREFIX) {
        return Err(invalid("wrong self-data tunnel id"));
    }
    let inner =
        length_prefixed(packet, 3, 7).ok_or_else(|| invalid("self-data tunnel length mismatch"))?;
    if le::u16(inner, 0) != Some(OPCODE) {
        return Err(invalid("wrong self-data opcode"));
    }
    let body =
        length_prefixed(inner, 2, 6).ok_or_else(|| invalid("self-data body length mismatch"))?;
    let mut cursor = Cursor::new(body);

    let identity = Identity {
        unknown_identity_u64: cursor.u64()?,
        guid: cursor.u64()?,
        model: cursor.u32()?,
    };
    let appearance = Appearance {
        head: cursor.counted_string()?,
        hair: cursor.counted_string()?,
        hair_color: cursor.u32()?,
        eye_color: cursor.u32()?,
        skin_tone: cursor.counted_string()?,
        face_paint: cursor.counted_string()?,
        model_customization: cursor.counted_string()?,
        customization_ids: cursor.u32s()?,
    };
    let transform = Transform {
        position: cursor.u32s()?,
        rotation: cursor.u32s()?,
    };
    let name = CharacterName {
        fields: cursor.u32s()?,
        first: cursor.counted_string()?,
        last: cursor.counted_string()?,
    };

    let coins = cursor.u32()?;
    let account = AccountState {
        birthday: cursor.u64()?,
        age: cursor.u32()?,
        play_time: cursor.u32()?,
        underage: cursor.u8()? != 0,
        open_chat: cursor.u8()? != 0,
        membership_status: cursor.u32()?,
        show_member_nag: cursor.u8()? != 0,
        chat_country_id: cursor.u32()?,
        chat_language_id: cursor.u32()?,
        preferred_language: cursor.u32()?,
        chat_language: cursor.u32()?,
        login_count: cursor.u32()?,
        grandfathered: cursor.u8()? != 0,
        active_vehicle_loadouts: cursor.u32s()?,
    };

    let profile_count = cursor.count(91)?;
    let mut equipment = Vec::new();
    let profiles = (0..profile_count)
        .map(|_| parse_profile(&mut cursor, &mut equipment))
        .collect::<io::Result<_>>()?;
    let active_profile_id = cursor.u32()?;
    let profile_type_count = cursor.count(8)?;
    let profile_types = (0..profile_type_count)
        .map(|_| cursor.u32s())
        .collect::<io::Result<_>>()?;

    let collections = parse_collection_rows(&mut cursor)?;
    let item_count = cursor.count(30)?;
    let items = (0..item_count)
        .map(|_| parse_inventory_item(&mut cursor))
        .collect::<io::Result<_>>()?;
    let gender = cursor.u32()?;

    let quest_records = parse_quest_records(&mut cursor)?;
    let mut quest_wire = Vec::new();
    write_quest_records(&mut quest_wire, &quest_records)?;
    let quests = state_from_quest_bindings(&quest_wire, quest_bindings);
    let post_quest_records = parse_post_quest_records(&mut cursor)?;
    let action_bars = parse_action_bars(&mut cursor)?;
    let action_bar_flag = cursor.u8()? != 0;

    // `00909fd0` reads its flag, then `008e8230` reads a count and dword keys.
    let action_key_count = cursor.count(4)?;
    let action_keys = (0..action_key_count)
        .map(|_| cursor.u32())
        .collect::<io::Result<_>>()?;
    let tutorial_count = cursor.count(4)?;
    let tutorial_keys = (0..tutorial_count)
        .map(|_| cursor.u32())
        .collect::<io::Result<_>>()?;

    let effect_count = cursor.count(4 + EFFECT_PAYLOAD_BYTES)?;
    let mut effects = Vec::with_capacity(effect_count);
    for _ in 0..effect_count {
        effects.push(EffectRecord {
            key: cursor.u32()?,
            payload: cursor.take(EFFECT_PAYLOAD_BYTES)?.to_vec(),
        });
    }
    let post_effect_tail = cursor.take(cursor.remaining())?.to_vec();

    Ok(SelfData {
        identity,
        appearance,
        transform,
        name,
        account,
        jobs: JobState {
            profiles,
            active_profile_id,
        },
        profile_types,
        collections,
        inventory: inventory::InventoryState {
            coins,
            items,
            equipment,
            next_instance_id: None,
        },
        gender,
        quest_records,
        quests,
        quest_bindings: quest_bindings.to_vec(),
        post_quest_records,
        action_bars,
        action_bar_flag,
        action_keys,
        tutorial_keys,
        effects,
        post_effect_tail,
    })
}

/// `try_build_self` for a caller that cannot return the refusal yet: tests, fixtures and the
/// login stream (`gateway/startup_stream.rs`, which L118 asks to switch to `try_build_self`).
/// Content is checked by `try_build_self` at load (`gateway/startup_source.rs`).
pub fn build_self(data: &SelfData) -> Vec<u8> {
    try_build_self(data).expect("self-data checked at load serializes")
}

/// Serialize typed self state into an opcode 12 gateway tunnel. A record the native readers
/// cannot take (an unmapped reward type, fields that do not match their kind, a saved value
/// outside its wire field) is refused.
pub fn try_build_self(data: &SelfData) -> io::Result<Vec<u8>> {
    let mut body = Vec::new();
    put_u64(&mut body, data.identity.unknown_identity_u64);
    put_u64(&mut body, data.identity.guid);
    put_u32(&mut body, data.identity.model);
    put_string(&mut body, &data.appearance.head)?;
    put_string(&mut body, &data.appearance.hair)?;
    put_u32(&mut body, data.appearance.hair_color);
    put_u32(&mut body, data.appearance.eye_color);
    put_string(&mut body, &data.appearance.skin_tone)?;
    put_string(&mut body, &data.appearance.face_paint)?;
    put_string(&mut body, &data.appearance.model_customization)?;
    for value in data.appearance.customization_ids {
        put_u32(&mut body, value);
    }
    for value in data
        .transform
        .position
        .into_iter()
        .chain(data.transform.rotation)
    {
        put_u32(&mut body, value);
    }
    for value in data.name.fields {
        put_u32(&mut body, value);
    }
    put_string(&mut body, &data.name.first)?;
    put_string(&mut body, &data.name.last)?;

    put_u32(&mut body, data.inventory.coins);
    put_u64(&mut body, data.account.birthday);
    put_u32(&mut body, data.account.age);
    put_u32(&mut body, data.account.play_time);
    put_u8(&mut body, u8::from(data.account.underage));
    put_u8(&mut body, u8::from(data.account.open_chat));
    put_u32(&mut body, data.account.membership_status);
    put_u8(&mut body, u8::from(data.account.show_member_nag));
    put_u32(&mut body, data.account.chat_country_id);
    put_u32(&mut body, data.account.chat_language_id);
    put_u32(&mut body, data.account.preferred_language);
    put_u32(&mut body, data.account.chat_language);
    put_u32(&mut body, data.account.login_count);
    put_u8(&mut body, u8::from(data.account.grandfathered));
    for value in data.account.active_vehicle_loadouts {
        put_u32(&mut body, value);
    }

    put_len(&mut body, data.jobs.profiles.len(), "too many profiles")?;
    for profile in &data.jobs.profiles {
        write_profile(&mut body, profile, &data.inventory.equipment)?;
    }
    put_u32(&mut body, data.jobs.active_profile_id);
    put_len(
        &mut body,
        data.profile_types.len(),
        "too many profile types",
    )?;
    for pair in &data.profile_types {
        put_u32(&mut body, pair[0]);
        put_u32(&mut body, pair[1]);
    }

    put_len(&mut body, data.collections.len(), "too many collections")?;
    for collection in &data.collections {
        for value in collection.prefix {
            put_u32(&mut body, value);
        }
        write_reward_bundle(&mut body, &collection.reward)?;
        put_len(
            &mut body,
            collection.keyed_rows.len(),
            "too many collection rows",
        )?;
        for row in &collection.keyed_rows {
            put_u32(&mut body, row.key);
            body.extend_from_slice(&row.payload);
        }
    }
    put_len(
        &mut body,
        data.inventory.items.len(),
        "too many inventory items",
    )?;
    for item in &data.inventory.items {
        write_inventory_item(&mut body, item)?;
    }
    put_u32(&mut body, data.gender);

    let mut quest_wire = Vec::new();
    write_quest_records(&mut quest_wire, &data.quest_records)?;
    overlay_quest_state(&mut quest_wire, &data.quests, &data.quest_bindings)?;
    body.extend_from_slice(&quest_wire);
    write_post_quest_records(&mut body, &data.post_quest_records)?;

    put_len(&mut body, data.action_bars.len(), "too many action bars")?;
    for bar in &data.action_bars {
        put_u32(&mut body, bar.key);
        put_u32(&mut body, bar.value);
        put_len(&mut body, bar.slots.len(), "too many action slots")?;
        for slot in &bar.slots {
            put_u32(&mut body, slot.key);
            body.extend_from_slice(&slot.fields);
        }
    }
    put_u8(&mut body, u8::from(data.action_bar_flag));
    put_len(&mut body, data.action_keys.len(), "too many action keys")?;
    for key in &data.action_keys {
        put_u32(&mut body, *key);
    }
    put_len(
        &mut body,
        data.tutorial_keys.len(),
        "too many tutorial keys",
    )?;
    for key in &data.tutorial_keys {
        put_u32(&mut body, *key);
    }
    put_len(&mut body, data.effects.len(), "too many effects")?;
    for effect in &data.effects {
        if effect.payload.len() != EFFECT_PAYLOAD_BYTES {
            return Err(invalid("self-data effect payload is not 81 bytes"));
        }
        put_u32(&mut body, effect.key);
        body.extend_from_slice(&effect.payload);
    }
    body.extend_from_slice(&data.post_effect_tail);

    let mut inner = Vec::with_capacity(body.len() + 6);
    inner.extend_from_slice(&OPCODE.to_le_bytes());
    put_len(&mut inner, body.len(), "self-data body too long")?;
    inner.extend_from_slice(&body);

    let mut packet = Vec::with_capacity(inner.len() + 7);
    packet.extend_from_slice(&TUNNEL_PREFIX);
    put_len(&mut packet, inner.len(), "self-data tunnel too long")?;
    packet.extend_from_slice(&inner);
    Ok(packet)
}

/// Empty local quest state for synthetic builder tests and callers that have
/// not loaded a persisted quest snapshot.
pub fn empty_quest_state() -> quests::QuestState {
    quests::QuestState {
        version: 1,
        quests: BTreeMap::new(),
        trial_best_ms: None,
        trial_completions: 0,
    }
}
