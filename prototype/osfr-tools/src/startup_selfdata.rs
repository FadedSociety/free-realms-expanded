//! Opcode 12 (`SendSelfToClient`) reader and serializer.
//!
//! Section boundaries and primitive widths below follow the native client
//! readers. Unnamed records remain bounded byte spans marked `(R)`; this is
//! capture preservation, not an assertion about server policy.

use osfr_tools::{inventory, quests};
use std::collections::BTreeMap;

const TUNNEL_PREFIX: [u8; 3] = [5, 0, 1];
const OPCODE: u16 = 12;
const REWARD_BUNDLE_FIXED_BYTES: usize = 60;
const ACTION_SLOT_BYTES: usize = 51;
const EFFECT_PAYLOAD_BYTES: usize = 81;

/// (R) The local quest-content binding used by the gateway's original journal
/// patch. Its offset is relative to the quest span beginning after gender.
/// The reader does not name the quest ID or objective semantics.
const CAPTURE_QUEST_BINDINGS: [QuestWireBinding; 1] = [QuestWireBinding {
    quest_id: 1617,
    objective_index: 0,
    offset: 4112,
}];

#[derive(Clone, Debug, PartialEq)]
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
    /// Bound counters are overlaid from `quests` using `quest_bindings`.
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
    pub post_effect_tail: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identity {
    /// (R) Captured value stored at client object +0x3c0; its meaning is unknown.
    pub unknown_identity_u64: u64,
    pub guid: u64,
    pub model: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transform {
    /// IEEE-754 component bit patterns, retained without float normalization.
    pub position: [u32; 4],
    /// IEEE-754 component bit patterns, retained without float normalization.
    pub rotation: [u32; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterName {
    pub fields: [u32; 3],
    pub first: String,
    pub last: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobState {
    pub profiles: Vec<JobProfile>,
    pub active_profile_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobProfile {
    /// The profile map key in the first wire dword, read by `00a18990`.
    pub map_key: u32,
    /// (R) Profile bytes before the equipment keyed list, with the map key
    /// removed so it can be changed independently.
    pub opaque_before_equipment_without_key: Vec<u8>,
    /// (R) Final dword read by `00911d60`; meaning unresolved.
    pub after_equipment_value: u32,
    /// `008fa5c0` count-prefixed tagged ability rows.
    pub abilities: Vec<ProfileAbility>,
    /// `008ca050` rows, ending at the zero-key sentinel.
    pub trailing_records: Vec<ProfileTrailingRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileTrailingRecord {
    pub key: u32,
    /// (R) Native row payload: one flag byte and eight dwords.
    pub payload: [u8; 33],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionRecord {
    /// Seven dwords consumed by `009099d0`; meanings remain unresolved (R).
    pub prefix: [u32; 7],
    pub reward: RewardBundle,
    pub keyed_rows: Vec<CollectionKeyedRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardBundle {
    /// Native success byte; retained without bool normalization.
    pub success: u8,
    /// Nine dwords, two u64 pairs and two scalars (`008e7930`); meanings R.
    pub fixed_fields: [u8; REWARD_BUNDLE_FIXED_BYTES],
    pub entries: Vec<RewardEntry>,
    /// Trailing dword consumed by the reward-bundle reader; meaning R.
    pub trailing_value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardEntry {
    pub kind: u32,
    /// Native hidden byte; retained without bool normalization.
    pub hidden: u8,
    /// (R) Paired IDs and four common dwords.
    pub common_fields: [u8; 24],
    /// Counted client string bytes; not assumed to be UTF-8.
    pub display_bytes: Vec<u8>,
    /// (R) Text color and member-only bytes.
    pub display_fields: [u8; 5],
    /// (R) Type/success-dependent final dword, when the reader consumes one.
    pub kind_fields: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionKeyedRow {
    pub key: u32,
    /// Seven dwords and bool consumed by `008dacc0`; meanings R.
    pub payload: [u8; 29],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestRecords {
    pub first: Vec<QuestFirstRecord>,
    /// (R) Fixed 17-byte tail after the first list.
    pub first_tail: [u8; 17],
    pub second: Vec<QuestSecondRecord>,
    /// (R) Fixed four-byte tail after the second list.
    pub second_tail: [u8; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestFirstRecord {
    /// (R) Fixed 34-byte prefix read before the reward bundle.
    pub prefix: [u8; 34],
    pub reward: RewardBundle,
    pub objectives: Vec<QuestObjectiveRecord>,
    /// (R) Fixed six-byte record tail.
    pub tail: [u8; 6],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestSecondRecord {
    /// (R) Fixed 33-byte prefix read before the reward bundle.
    pub prefix: [u8; 33],
    pub reward: RewardBundle,
    pub objectives: Vec<QuestObjectiveRecord>,
    /// (R) Fixed 21-byte record tail.
    pub tail: [u8; 21],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestObjectiveRecord {
    pub key: u32,
    /// (R) Fixed 13-byte objective prefix.
    pub prefix: [u8; 13],
    pub reward: RewardBundle,
    /// (R) Fixed 21-byte objective tail.
    pub tail: [u8; 21],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostQuestRecords {
    pub keyed_strings: Vec<KeyedStringRecord>,
    pub recipes: Vec<RecipeRecord>,
    pub variable_records: Vec<VariableRecord>,
    /// (R) Captured signed marker and u64 value before pre-action rows.
    pub pet_source_marker: i32,
    pub pet_source_value: u64,
    pub pre_action_records: Vec<PreActionRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyedStringRecord {
    /// (R) Key pair read by `008e73b0`.
    pub key_fields: [u8; 8],
    pub value_bytes: Vec<u8>,
    /// (R) Two dwords and a flag.
    pub tail: [u8; 13],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeRecord {
    /// (R) Fixed fields read by `0090b930`.
    pub prefix: [u8; 29],
    pub components: Vec<[u8; 32]>,
    /// (R) Fixed recipe tail.
    pub tail: [u8; 9],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableRecord {
    /// (R) Fixed prefix before three nested arrays.
    pub prefix: [u8; 26],
    pub first_rows: Vec<[u8; 37]>,
    pub second_rows: Vec<u32>,
    pub third_rows: Vec<[u8; 12]>,
    pub first_string: Vec<u8>,
    /// (R) Fixed eight-byte field between counted strings.
    pub middle: [u8; 8],
    pub second_string: Vec<u8>,
    /// (R) Fixed tail read by `00912cf0`.
    pub tail: [u8; 58],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreActionRecord {
    /// (R) Fixed fields before the counted string.
    pub prefix: [u8; 25],
    pub string_bytes: Vec<u8>,
    /// (R) Two bool bytes.
    pub tail: [u8; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuestWireBinding {
    pub quest_id: u32,
    pub objective_index: usize,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionBar {
    pub key: u32,
    /// Native `0091a9b0` reads this scalar before the slot-list reader.
    pub value: u32,
    pub slots: Vec<ActionBarSlot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionBarSlot {
    pub key: u32,
    /// (R) The native reader consumes 51 fixed bytes; field meanings remain
    /// unresolved, so the byte payload is preserved exactly.
    pub fields: [u8; ACTION_SLOT_BYTES],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRecord {
    pub key: u32,
    /// (R) Native `00a18cc0` consumes this 81-byte record; most fields are
    /// unnamed here and therefore remain exact bytes.
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

    fn take(&mut self, len: usize) -> &'a [u8] {
        self.try_take(len).expect("truncated self-data packet")
    }

    fn try_take(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.at.checked_add(len).expect("self-data length overflow");
        let part = self.bytes.get(self.at..end)?;
        self.at = end;
        Some(part)
    }

    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }

    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.take(4).try_into().unwrap())
    }

    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take(8).try_into().unwrap())
    }

    fn counted_string(&mut self) -> String {
        let len = self.count(1);
        String::from_utf8(self.take(len).to_vec()).expect("self-data string is not UTF-8")
    }

    fn count(&mut self, minimum_item_size: usize) -> usize {
        let count = self.i32();
        assert!(count >= 0, "negative self-data collection count");
        let count = count as usize;
        let count_offset = self.at - 4;
        assert!(
            count <= self.remaining() / minimum_item_size.max(1),
            "self-data count {count} at body offset {count_offset} exceeds remaining packet"
        );
        count
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
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

fn put_string(out: &mut Vec<u8>, value: &str) {
    put_u32(
        out,
        u32::try_from(value.len()).expect("self-data string too long"),
    );
    out.extend_from_slice(value.as_bytes());
}

fn parse_counted_bytes(cursor: &mut Cursor<'_>) -> Vec<u8> {
    let len = cursor.count(1);
    cursor.take(len).to_vec()
}

fn parse_reward_bundle(cursor: &mut Cursor<'_>) -> RewardBundle {
    let success = cursor.u8();
    let fixed_fields = cursor.take(REWARD_BUNDLE_FIXED_BYTES).try_into().unwrap();
    let entries = cursor.count(38);
    let mut parsed_entries = Vec::with_capacity(entries);
    for _ in 0..entries {
        let kind = cursor.u32();
        let hidden = cursor.u8();
        let common_fields = cursor.take(8 + 16).try_into().unwrap();
        let display_len = cursor.count(1);
        let display_bytes = cursor.take(display_len).to_vec();
        let display_fields = cursor.take(4 + 1).try_into().unwrap();
        let kind_fields = match kind {
            1 if success != 0 => cursor.take(4).to_vec(),
            1 | 3 | 12 => Vec::new(),
            6 | 10 => cursor.take(4).to_vec(),
            _ => panic!("unmapped native reward-entry type {kind}"),
        };
        parsed_entries.push(RewardEntry {
            kind,
            hidden,
            common_fields,
            display_bytes,
            display_fields,
            kind_fields,
        });
    }
    RewardBundle {
        success,
        fixed_fields,
        entries: parsed_entries,
        trailing_value: cursor.u32(),
    }
}

fn write_reward_bundle(out: &mut Vec<u8>, bundle: &RewardBundle) {
    put_u8(out, bundle.success);
    out.extend_from_slice(&bundle.fixed_fields);
    put_u32(
        out,
        u32::try_from(bundle.entries.len()).expect("too many reward entries"),
    );
    for entry in &bundle.entries {
        put_u32(out, entry.kind);
        put_u8(out, entry.hidden);
        out.extend_from_slice(&entry.common_fields);
        put_u32(
            out,
            u32::try_from(entry.display_bytes.len()).expect("reward display string too long"),
        );
        out.extend_from_slice(&entry.display_bytes);
        out.extend_from_slice(&entry.display_fields);
        let expected_kind_bytes = match entry.kind {
            1 if bundle.success != 0 => 4,
            1 | 3 | 12 => 0,
            6 | 10 => 4,
            _ => panic!("unmapped native reward-entry type {}", entry.kind),
        };
        assert_eq!(entry.kind_fields.len(), expected_kind_bytes);
        out.extend_from_slice(&entry.kind_fields);
    }
    put_u32(out, bundle.trailing_value);
}

fn parse_objective_list(cursor: &mut Cursor<'_>) -> Vec<QuestObjectiveRecord> {
    // `0093c840`/`0093ca50`: keyed objective row, the `008fd770` prefix,
    // reward bundle, and the reader's fixed tail.
    let count = cursor.count(107);
    (0..count)
        .map(|_| QuestObjectiveRecord {
            key: cursor.u32(),
            prefix: cursor.take(13).try_into().unwrap(),
            reward: parse_reward_bundle(cursor),
            tail: cursor.take(21).try_into().unwrap(),
        })
        .collect()
}

fn parse_profile(
    cursor: &mut Cursor<'_>,
    equipment: &mut Vec<inventory::EquippedItem>,
) -> JobProfile {
    let start = cursor.at;
    let map_key = u32::from_le_bytes(cursor.bytes[start..start + 4].try_into().unwrap());

    // `00911d60`: fixed prefix and first keyed pair-list.
    cursor.take(8 * 4 + 1 + 4);
    let first_pairs = cursor.count(12);
    cursor.take(first_pairs * 12);
    cursor.take(1 + 8 + 1 + 6 * 4);

    // The second keyed pair-list is independently proven to be the profile
    // equipment span: its count and 12-byte rows match inventory.json spans
    // and `Inventory::profile_equipment` exactly.
    let equipment_count_at = cursor.at;
    let equipment_count = cursor.count(12);
    let profile_id = i32::try_from(map_key).expect("profile map key outside server range");
    for _ in 0..equipment_count {
        equipment.push(inventory::EquippedItem {
            profile_id,
            slot: cursor.i32(),
            item_id: cursor.i32(),
            category: cursor.i32(),
        });
    }
    let after_equipment_value = cursor.u32();

    // `008fa5c0` reads the count-prefixed ability vector. `00ae1f70` reserves
    // eight elements, but the serialized count controls the rows.
    let abilities = cursor.count(4);
    let mut ability_records = Vec::with_capacity(abilities);
    for _ in 0..abilities {
        let kind = cursor.u32();
        let kind_field_count = match kind {
            1 | 3 => 2,
            2 => 1,
            _ => 0,
        };
        let kind_fields = (0..kind_field_count).map(|_| cursor.u32()).collect();
        let (common_fields, common_flag) = if kind == 0 {
            (None, None)
        } else {
            (
                Some(std::array::from_fn(|_| cursor.u32())),
                Some(cursor.u8()),
            )
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
        let key = cursor.u32();
        if key == 0 {
            break;
        }
        trailing_records.push(ProfileTrailingRecord {
            key,
            payload: cursor.take(1 + 8 * 4).try_into().unwrap(),
        });
    }

    let wire = &cursor.bytes[start..cursor.at];
    let before = &wire[..equipment_count_at - start];
    JobProfile {
        map_key,
        opaque_before_equipment_without_key: before[4..].to_vec(),
        after_equipment_value,
        abilities: ability_records,
        trailing_records,
    }
}

fn write_profile(out: &mut Vec<u8>, profile: &JobProfile, equipment: &[inventory::EquippedItem]) {
    let before = &profile.opaque_before_equipment_without_key;
    put_u32(out, profile.map_key);
    out.extend_from_slice(before);

    let profile_id = i32::try_from(profile.map_key).expect("profile map key outside server range");
    let entries: Vec<_> = equipment
        .iter()
        .filter(|entry| entry.profile_id == profile_id)
        .collect();
    put_u32(
        out,
        u32::try_from(entries.len()).expect("too many equipped items"),
    );
    for entry in entries {
        put_i32(out, entry.slot);
        put_i32(out, entry.item_id);
        put_i32(out, entry.category);
    }
    put_u32(out, profile.after_equipment_value);
    put_u32(
        out,
        u32::try_from(profile.abilities.len()).expect("too many profile abilities"),
    );
    for ability in &profile.abilities {
        put_u32(out, ability.kind);
        let expected_kind_fields = match ability.kind {
            1 | 3 => 2,
            2 => 1,
            _ => 0,
        };
        assert_eq!(ability.kind_fields.len(), expected_kind_fields);
        for field in &ability.kind_fields {
            put_u32(out, *field);
        }
        if ability.kind == 0 {
            assert!(ability.common_fields.is_none());
            assert!(ability.common_flag.is_none());
        } else {
            let fields = ability
                .common_fields
                .expect("nonzero profile ability missing native fields");
            for field in fields {
                put_u32(out, field);
            }
            put_u8(
                out,
                ability
                    .common_flag
                    .expect("nonzero profile ability missing native flag"),
            );
        }
    }
    for record in &profile.trailing_records {
        assert_ne!(record.key, 0, "profile trailing key collides with sentinel");
        put_u32(out, record.key);
        out.extend_from_slice(&record.payload);
    }
    put_u32(out, 0);
}

fn parse_inventory_item(cursor: &mut Cursor<'_>) -> inventory::InventoryItem {
    let definition_id = cursor.i32();
    let tint = cursor.i32();
    let id = cursor.i32();
    let count = cursor.u32();
    let consumed_count = cursor.u32();
    let last_cast_time = i64::from(cursor.i32());
    let rental = if cursor.u8() == 0 {
        None
    } else {
        Some(inventory::RentalState {
            expiration: i64::from_le_bytes(cursor.take(8).try_into().unwrap()),
            upsells: cursor.i32(),
            bundle_id: cursor.i32(),
        })
    };
    let ability_count = cursor.i32();
    let activate_enabled = cursor.u8() != 0;
    inventory::InventoryItem {
        definition_id,
        tint,
        id,
        count,
        consumed_count,
        // (R) `stored_count` is not read or written by the self-data client
        // serializer; the production inventory state keeps its existing value.
        stored_count: 0,
        last_cast_time,
        rental,
        ability_count,
        activate_enabled,
    }
}

fn write_inventory_item(out: &mut Vec<u8>, item: &inventory::InventoryItem) {
    let last_cast_time = i32::try_from(item.last_cast_time)
        .expect("self-data last-cast value exceeds its native i32 field");
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
    put_i32(out, item.ability_count);
    put_u8(out, u8::from(item.activate_enabled));
}

fn parse_collection_rows(cursor: &mut Cursor<'_>) -> Vec<CollectionRecord> {
    let count = cursor.count(101);
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let prefix = std::array::from_fn(|_| cursor.u32());
        let reward = parse_reward_bundle(cursor);
        let keyed_count = cursor.count(33);
        let keyed_rows = (0..keyed_count)
            .map(|_| CollectionKeyedRow {
                key: cursor.u32(),
                payload: cursor.take(29).try_into().unwrap(),
            })
            .collect();
        records.push(CollectionRecord {
            prefix,
            reward,
            keyed_rows,
        });
    }
    records
}

fn parse_quest_records(cursor: &mut Cursor<'_>) -> QuestRecords {
    let first_count = cursor.count(113);
    let first = (0..first_count)
        .map(|_| QuestFirstRecord {
            prefix: cursor.take(34).try_into().unwrap(),
            reward: parse_reward_bundle(cursor),
            objectives: parse_objective_list(cursor),
            tail: cursor.take(6).try_into().unwrap(),
        })
        .collect();
    let first_tail = cursor.take(17).try_into().unwrap();

    let second_count = cursor.count(127);
    let second = (0..second_count)
        .map(|_| QuestSecondRecord {
            prefix: cursor.take(33).try_into().unwrap(),
            reward: parse_reward_bundle(cursor),
            objectives: parse_objective_list(cursor),
            tail: cursor.take(21).try_into().unwrap(),
        })
        .collect();
    let second_tail = cursor.take(4).try_into().unwrap();
    QuestRecords {
        first,
        first_tail,
        second,
        second_tail,
    }
}

fn write_objective_list(out: &mut Vec<u8>, rows: &[QuestObjectiveRecord]) {
    put_u32(
        out,
        u32::try_from(rows.len()).expect("too many quest objectives"),
    );
    for row in rows {
        put_u32(out, row.key);
        out.extend_from_slice(&row.prefix);
        write_reward_bundle(out, &row.reward);
        out.extend_from_slice(&row.tail);
    }
}

fn write_quest_records(out: &mut Vec<u8>, records: &QuestRecords) {
    put_u32(
        out,
        u32::try_from(records.first.len()).expect("too many first-list quests"),
    );
    for record in &records.first {
        out.extend_from_slice(&record.prefix);
        write_reward_bundle(out, &record.reward);
        write_objective_list(out, &record.objectives);
        out.extend_from_slice(&record.tail);
    }
    out.extend_from_slice(&records.first_tail);
    put_u32(
        out,
        u32::try_from(records.second.len()).expect("too many second-list quests"),
    );
    for record in &records.second {
        out.extend_from_slice(&record.prefix);
        write_reward_bundle(out, &record.reward);
        write_objective_list(out, &record.objectives);
        out.extend_from_slice(&record.tail);
    }
    out.extend_from_slice(&records.second_tail);
}

fn parse_post_quest_records(cursor: &mut Cursor<'_>) -> PostQuestRecords {
    // `008e73b0`: key pair, counted string, two dwords and a flag.
    let strings = cursor.count(25);
    let keyed_strings = (0..strings)
        .map(|_| KeyedStringRecord {
            key_fields: cursor.take(8).try_into().unwrap(),
            value_bytes: parse_counted_bytes(cursor),
            tail: cursor.take(4 + 8 + 1).try_into().unwrap(),
        })
        .collect();

    // `0090b930` recipe rows and fixed 32-byte component rows.
    let recipes = cursor.count(42);
    let recipes = (0..recipes)
        .map(|_| {
            let prefix = cursor.take(29).try_into().unwrap();
            let components = cursor.count(32);
            let components = (0..components)
                .map(|_| cursor.take(32).try_into().unwrap())
                .collect();
            RecipeRecord {
                prefix,
                components,
                tail: cursor.take(9).try_into().unwrap(),
            }
        })
        .collect();

    // `00912cf0`: three variable nested arrays, two strings and a fixed tail.
    let variable_records = cursor.count(1);
    let variable_records = (0..variable_records)
        .map(|_| {
            let prefix = cursor.take(4 + 1 + 4 + 16 + 1).try_into().unwrap();
            let first_count = cursor.count(37);
            let first_rows = (0..first_count)
                .map(|_| cursor.take(37).try_into().unwrap())
                .collect();
            let second_count = cursor.count(4);
            let second_rows = (0..second_count).map(|_| cursor.u32()).collect();
            let third_count = cursor.count(12);
            let third_rows = (0..third_count)
                .map(|_| cursor.take(12).try_into().unwrap())
                .collect();
            let first_string = parse_counted_bytes(cursor);
            let middle = cursor.take(8).try_into().unwrap();
            let second_string = parse_counted_bytes(cursor);
            VariableRecord {
                prefix,
                first_rows,
                second_rows,
                third_rows,
                first_string,
                middle,
                second_string,
                tail: cursor
                    .take(4 + 1 + 4 + 1 + 4 * 4 + 8 * 4)
                    .try_into()
                    .unwrap(),
            }
        })
        .collect();

    // The capture carries the signed -1 pet-source marker followed by a u64.
    let pet_source_marker = cursor.i32();
    let pet_source_value = cursor.u64();

    // `008e74c0`: three dwords, a pair, bool, dword, counted string, two bools.
    let pre_action_rows = cursor.count(31);
    let pre_action_records = (0..pre_action_rows)
        .map(|_| PreActionRecord {
            prefix: cursor.take(3 * 4 + 8 + 1 + 4).try_into().unwrap(),
            string_bytes: parse_counted_bytes(cursor),
            tail: cursor.take(2).try_into().unwrap(),
        })
        .collect();

    PostQuestRecords {
        keyed_strings,
        recipes,
        variable_records,
        pet_source_marker,
        pet_source_value,
        pre_action_records,
    }
}

fn write_counted_bytes(out: &mut Vec<u8>, value: &[u8]) {
    put_u32(
        out,
        u32::try_from(value.len()).expect("self-data string too long"),
    );
    out.extend_from_slice(value);
}

fn write_post_quest_records(out: &mut Vec<u8>, records: &PostQuestRecords) {
    put_u32(
        out,
        u32::try_from(records.keyed_strings.len()).expect("too many keyed strings"),
    );
    for record in &records.keyed_strings {
        out.extend_from_slice(&record.key_fields);
        write_counted_bytes(out, &record.value_bytes);
        out.extend_from_slice(&record.tail);
    }
    put_u32(
        out,
        u32::try_from(records.recipes.len()).expect("too many recipes"),
    );
    for recipe in &records.recipes {
        out.extend_from_slice(&recipe.prefix);
        put_u32(
            out,
            u32::try_from(recipe.components.len()).expect("too many recipe components"),
        );
        for component in &recipe.components {
            out.extend_from_slice(component);
        }
        out.extend_from_slice(&recipe.tail);
    }
    put_u32(
        out,
        u32::try_from(records.variable_records.len()).expect("too many variable records"),
    );
    for record in &records.variable_records {
        out.extend_from_slice(&record.prefix);
        put_u32(
            out,
            u32::try_from(record.first_rows.len()).expect("too many first nested rows"),
        );
        for row in &record.first_rows {
            out.extend_from_slice(row);
        }
        put_u32(
            out,
            u32::try_from(record.second_rows.len()).expect("too many second nested rows"),
        );
        for row in &record.second_rows {
            put_u32(out, *row);
        }
        put_u32(
            out,
            u32::try_from(record.third_rows.len()).expect("too many third nested rows"),
        );
        for row in &record.third_rows {
            out.extend_from_slice(row);
        }
        write_counted_bytes(out, &record.first_string);
        out.extend_from_slice(&record.middle);
        write_counted_bytes(out, &record.second_string);
        out.extend_from_slice(&record.tail);
    }
    put_i32(out, records.pet_source_marker);
    put_u64(out, records.pet_source_value);
    put_u32(
        out,
        u32::try_from(records.pre_action_records.len()).expect("too many pre-action records"),
    );
    for record in &records.pre_action_records {
        out.extend_from_slice(&record.prefix);
        write_counted_bytes(out, &record.string_bytes);
        out.extend_from_slice(&record.tail);
    }
}

fn parse_action_bars(cursor: &mut Cursor<'_>) -> Vec<ActionBar> {
    let count = cursor.count(12);
    let mut bars = Vec::with_capacity(count);
    for _ in 0..count {
        let key = cursor.u32();
        let value = cursor.u32();
        let slots_count = cursor.count(4 + ACTION_SLOT_BYTES);
        let mut slots = Vec::with_capacity(slots_count);
        for _ in 0..slots_count {
            let slot_key = cursor.u32();
            let fields = cursor.take(ACTION_SLOT_BYTES).try_into().unwrap();
            slots.push(ActionBarSlot {
                key: slot_key,
                fields,
            });
        }
        bars.push(ActionBar { key, value, slots });
    }
    bars
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
        let Some(raw) = quest_records.get(binding.offset..binding.offset + 4) else {
            continue;
        };
        let value = i32::from_le_bytes(raw.try_into().unwrap());
        if value < 0 {
            continue;
        }
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
        progress.counts.resize(binding.objective_index + 1, 0);
        progress.counts[binding.objective_index] = value as u32;
    }
    state
}

fn overlay_quest_state(
    quest_records: &mut [u8],
    state: &quests::QuestState,
    bindings: &[QuestWireBinding],
) {
    for binding in bindings {
        let Some(progress) = state.quests.get(&binding.quest_id) else {
            continue;
        };
        let Some(value) = progress.counts.get(binding.objective_index) else {
            continue;
        };
        let offset = binding.offset;
        let Some(target) = quest_records.get_mut(offset..offset + 4) else {
            continue;
        };
        let value = i32::try_from(*value).expect("quest progress exceeds native i32 field");
        target.copy_from_slice(&value.to_le_bytes());
    }
}

/// Parse opcode 12 using the quest binding recovered by the existing local
/// journal content. Call `parse_self_with_quest_bindings` when a caller has a
/// different reviewed binding table.
pub fn parse_self(packet: &[u8]) -> SelfData {
    parse_self_with_quest_bindings(packet, &CAPTURE_QUEST_BINDINGS)
}

/// Parse opcode 12 while overlaying the supplied content bindings into the
/// shared `QuestState` representation.
pub fn parse_self_with_quest_bindings(
    packet: &[u8],
    quest_bindings: &[QuestWireBinding],
) -> SelfData {
    assert!(packet.len() >= 13, "short self-data tunnel");
    assert_eq!(&packet[..3], &TUNNEL_PREFIX, "wrong self-data tunnel id");
    let tunnel_len = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    assert_eq!(
        packet.len(),
        tunnel_len + 7,
        "self-data tunnel length mismatch"
    );

    let inner = &packet[7..];
    assert_eq!(u16::from_le_bytes(inner[..2].try_into().unwrap()), OPCODE);
    let body_len = u32::from_le_bytes(inner[2..6].try_into().unwrap()) as usize;
    assert_eq!(inner.len(), body_len + 6, "self-data body length mismatch");
    let mut cursor = Cursor::new(&inner[6..]);

    let identity = Identity {
        unknown_identity_u64: cursor.u64(),
        guid: cursor.u64(),
        model: cursor.u32(),
    };
    let appearance = Appearance {
        head: cursor.counted_string(),
        hair: cursor.counted_string(),
        hair_color: cursor.u32(),
        eye_color: cursor.u32(),
        skin_tone: cursor.counted_string(),
        face_paint: cursor.counted_string(),
        model_customization: cursor.counted_string(),
        customization_ids: std::array::from_fn(|_| cursor.u32()),
    };
    let transform = Transform {
        position: std::array::from_fn(|_| cursor.u32()),
        rotation: std::array::from_fn(|_| cursor.u32()),
    };
    let name = CharacterName {
        fields: std::array::from_fn(|_| cursor.u32()),
        first: cursor.counted_string(),
        last: cursor.counted_string(),
    };

    let coins = cursor.u32();
    let account = AccountState {
        birthday: cursor.u64(),
        age: cursor.u32(),
        play_time: cursor.u32(),
        underage: cursor.u8() != 0,
        open_chat: cursor.u8() != 0,
        membership_status: cursor.u32(),
        show_member_nag: cursor.u8() != 0,
        chat_country_id: cursor.u32(),
        chat_language_id: cursor.u32(),
        preferred_language: cursor.u32(),
        chat_language: cursor.u32(),
        login_count: cursor.u32(),
        grandfathered: cursor.u8() != 0,
        active_vehicle_loadouts: [cursor.u32(), cursor.u32()],
    };

    let profile_count = cursor.count(91);
    let mut equipment = Vec::new();
    let profiles = (0..profile_count)
        .map(|_| parse_profile(&mut cursor, &mut equipment))
        .collect();
    let active_profile_id = cursor.u32();
    let profile_type_count = cursor.count(8);
    let profile_types = (0..profile_type_count)
        .map(|_| [cursor.u32(), cursor.u32()])
        .collect();

    let collections = parse_collection_rows(&mut cursor);
    let item_count = cursor.count(30);
    let items = (0..item_count)
        .map(|_| parse_inventory_item(&mut cursor))
        .collect();
    let gender = cursor.u32();

    let quest_records = parse_quest_records(&mut cursor);
    let mut quest_wire = Vec::new();
    write_quest_records(&mut quest_wire, &quest_records);
    let quests = state_from_quest_bindings(&quest_wire, quest_bindings);
    let post_quest_records = parse_post_quest_records(&mut cursor);
    let action_bars = parse_action_bars(&mut cursor);
    let action_bar_flag = cursor.u8() != 0;

    // `00909fd0` reads its flag, then `008e8230` reads a count and dword keys.
    let action_key_count = cursor.count(4);
    let action_keys = (0..action_key_count).map(|_| cursor.u32()).collect();
    let tutorial_count = cursor.count(4);
    let tutorial_keys = (0..tutorial_count).map(|_| cursor.u32()).collect();

    let effect_count = cursor.count(4 + EFFECT_PAYLOAD_BYTES);
    let mut effects = Vec::with_capacity(effect_count);
    for _ in 0..effect_count {
        effects.push(EffectRecord {
            key: cursor.u32(),
            payload: cursor.take(EFFECT_PAYLOAD_BYTES).to_vec(),
        });
    }
    let post_effect_tail = cursor.take(cursor.remaining()).to_vec();

    SelfData {
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
    }
}

/// Serialize typed self state into an opcode 12 gateway tunnel.
pub fn build_self(data: &SelfData) -> Vec<u8> {
    let mut body = Vec::new();
    put_u64(&mut body, data.identity.unknown_identity_u64);
    put_u64(&mut body, data.identity.guid);
    put_u32(&mut body, data.identity.model);
    put_string(&mut body, &data.appearance.head);
    put_string(&mut body, &data.appearance.hair);
    put_u32(&mut body, data.appearance.hair_color);
    put_u32(&mut body, data.appearance.eye_color);
    put_string(&mut body, &data.appearance.skin_tone);
    put_string(&mut body, &data.appearance.face_paint);
    put_string(&mut body, &data.appearance.model_customization);
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
    put_string(&mut body, &data.name.first);
    put_string(&mut body, &data.name.last);

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

    put_u32(
        &mut body,
        u32::try_from(data.jobs.profiles.len()).expect("too many profiles"),
    );
    for profile in &data.jobs.profiles {
        write_profile(&mut body, profile, &data.inventory.equipment);
    }
    put_u32(&mut body, data.jobs.active_profile_id);
    put_u32(
        &mut body,
        u32::try_from(data.profile_types.len()).expect("too many profile types"),
    );
    for pair in &data.profile_types {
        put_u32(&mut body, pair[0]);
        put_u32(&mut body, pair[1]);
    }

    put_u32(
        &mut body,
        u32::try_from(data.collections.len()).expect("too many collections"),
    );
    for collection in &data.collections {
        for value in collection.prefix {
            put_u32(&mut body, value);
        }
        write_reward_bundle(&mut body, &collection.reward);
        put_u32(
            &mut body,
            u32::try_from(collection.keyed_rows.len()).expect("too many collection rows"),
        );
        for row in &collection.keyed_rows {
            put_u32(&mut body, row.key);
            body.extend_from_slice(&row.payload);
        }
    }
    put_u32(
        &mut body,
        u32::try_from(data.inventory.items.len()).expect("too many inventory items"),
    );
    for item in &data.inventory.items {
        write_inventory_item(&mut body, item);
    }
    put_u32(&mut body, data.gender);

    let mut quest_wire = Vec::new();
    write_quest_records(&mut quest_wire, &data.quest_records);
    overlay_quest_state(&mut quest_wire, &data.quests, &data.quest_bindings);
    body.extend_from_slice(&quest_wire);
    write_post_quest_records(&mut body, &data.post_quest_records);

    put_u32(
        &mut body,
        u32::try_from(data.action_bars.len()).expect("too many action bars"),
    );
    for bar in &data.action_bars {
        put_u32(&mut body, bar.key);
        put_u32(&mut body, bar.value);
        put_u32(
            &mut body,
            u32::try_from(bar.slots.len()).expect("too many action slots"),
        );
        for slot in &bar.slots {
            put_u32(&mut body, slot.key);
            body.extend_from_slice(&slot.fields);
        }
    }
    put_u8(&mut body, u8::from(data.action_bar_flag));
    put_u32(
        &mut body,
        u32::try_from(data.action_keys.len()).expect("too many action keys"),
    );
    for key in &data.action_keys {
        put_u32(&mut body, *key);
    }
    put_u32(
        &mut body,
        u32::try_from(data.tutorial_keys.len()).expect("too many tutorial keys"),
    );
    for key in &data.tutorial_keys {
        put_u32(&mut body, *key);
    }
    put_u32(
        &mut body,
        u32::try_from(data.effects.len()).expect("too many effects"),
    );
    for effect in &data.effects {
        assert_eq!(effect.payload.len(), EFFECT_PAYLOAD_BYTES);
        put_u32(&mut body, effect.key);
        body.extend_from_slice(&effect.payload);
    }
    body.extend_from_slice(&data.post_effect_tail);

    let mut inner = Vec::with_capacity(body.len() + 6);
    inner.extend_from_slice(&OPCODE.to_le_bytes());
    put_u32(
        &mut inner,
        u32::try_from(body.len()).expect("self-data body too long"),
    );
    inner.extend_from_slice(&body);

    let mut packet = Vec::with_capacity(inner.len() + 7);
    packet.extend_from_slice(&TUNNEL_PREFIX);
    put_u32(
        &mut packet,
        u32::try_from(inner.len()).expect("self-data tunnel too long"),
    );
    packet.extend_from_slice(&inner);
    packet
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
