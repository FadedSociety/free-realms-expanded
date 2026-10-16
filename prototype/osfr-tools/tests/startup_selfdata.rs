use osfr_tools::startup_selfdata::{
    AccountState, ActionBar, ActionBarSlot, Appearance, CharacterName, CollectionKeyedRow,
    CollectionRecord, EffectRecord, Identity, JobProfile, JobState, KeyedStringRecord,
    PostQuestRecords, PreActionRecord, ProfileAbility, ProfileTrailingRecord, QuestFirstRecord,
    QuestObjectiveRecord, QuestRecords, QuestSecondRecord, QuestWireBinding, RecipeRecord,
    RewardBundle, RewardEntry, SelfData, Transform, VariableRecord, build_self, empty_quest_state,
    parse_self, parse_self_with_quest_bindings,
};
use osfr_tools::{inventory, quests, startup_selfdata};
use std::{fs, path::Path};

fn synthetic_profile(map_key: u32) -> JobProfile {
    let mut wire = Vec::new();
    for value in [map_key, 12, 13, 14, 15, 16, 17, 18] {
        wire.extend_from_slice(&value.to_le_bytes());
    }
    wire.push(1);
    wire.extend_from_slice(&19u32.to_le_bytes());
    wire.extend_from_slice(&0i32.to_le_bytes());
    wire.push(0);
    wire.extend_from_slice(&20u32.to_le_bytes());
    wire.extend_from_slice(&21u32.to_le_bytes());
    wire.push(1);
    for value in [22u32, 23, 24, 25, 26, 27] {
        wire.extend_from_slice(&value.to_le_bytes());
    }
    wire.extend_from_slice(&0i32.to_le_bytes());
    wire.extend_from_slice(&28u32.to_le_bytes());
    wire.extend_from_slice(&8u32.to_le_bytes());
    for _ in 0..8 {
        wire.extend_from_slice(&0u32.to_le_bytes());
    }
    wire.extend_from_slice(&0u32.to_le_bytes());
    assert_eq!(wire.len(), 123);

    // The second keyed pair-list count begins at profile offset 75.
    JobProfile {
        map_key,
        opaque_before_equipment_without_key: wire[4..75].to_vec(),
        after_equipment_value: 28,
        abilities: vec![
            ProfileAbility {
                kind: 0,
                kind_fields: Vec::new(),
                common_fields: None,
                common_flag: None,
            };
            8
        ],
        trailing_records: Vec::new(),
    }
}

fn synthetic_item(rental: bool, id: i32) -> inventory::InventoryItem {
    inventory::InventoryItem {
        definition_id: 500 + id,
        tint: 7,
        id,
        count: 2,
        consumed_count: 1,
        stored_count: 0,
        last_cast_time: -30,
        rental: rental.then_some(inventory::RentalState {
            expiration: 0x0102_0304_0506_0708,
            upsells: 3,
            bundle_id: 4,
        }),
        ability_count: 5,
        activate_enabled: true,
    }
}

fn sample() -> SelfData {
    let mut first_tail = [0; 17];
    first_tail[..4].copy_from_slice(&3i32.to_le_bytes());
    let quest_records = QuestRecords {
        first: Vec::new(),
        first_tail,
        second: Vec::new(),
        second_tail: [0; 4],
    };

    let post_quest_records = PostQuestRecords {
        keyed_strings: Vec::new(),
        recipes: Vec::new(),
        variable_records: Vec::new(),
        pet_source_marker: -1,
        pet_source_value: 0,
        pre_action_records: Vec::new(),
    };

    let mut quests = empty_quest_state();
    quests.quests.insert(
        900001,
        quests::QuestProgress {
            accepted: false,
            counts: vec![3],
            completed: false,
            rewarded: false,
        },
    );

    SelfData {
        identity: Identity {
            unknown_identity_u64: 0x1020_3040_5060_7080,
            guid: 0x8877_6655_4433_2211,
            model: 3,
        },
        appearance: Appearance {
            head: "head-a".into(),
            hair: "hair-a".into(),
            hair_color: 0x1122_3344,
            eye_color: 0x5566_7788,
            skin_tone: "skin-a".into(),
            face_paint: "paint-a".into(),
            model_customization: "custom-a".into(),
            customization_ids: [31, 32, 33, 34, 35],
        },
        transform: Transform {
            position: [
                1.25f32.to_bits(),
                (-2.5f32).to_bits(),
                3.75f32.to_bits(),
                1.0f32.to_bits(),
            ],
            rotation: [
                0.0f32.to_bits(),
                0.0f32.to_bits(),
                0.0f32.to_bits(),
                1.0f32.to_bits(),
            ],
        },
        name: CharacterName {
            fields: [41, 42, 43],
            first: "synthetic-name-a".into(),
            last: "synthetic-last-a".into(),
        },
        account: AccountState {
            birthday: 0x0102_0304_0506_0708,
            age: 12,
            play_time: 3456,
            underage: false,
            open_chat: true,
            membership_status: 2,
            show_member_nag: false,
            chat_country_id: 7,
            chat_language_id: 8,
            preferred_language: 9,
            chat_language: 10,
            login_count: 11,
            grandfathered: false,
            active_vehicle_loadouts: [12, 13],
        },
        jobs: JobState {
            profiles: vec![synthetic_profile(4)],
            active_profile_id: 4,
        },
        profile_types: vec![[4, 2], [12, 3]],
        collections: Vec::new(),
        inventory: inventory::InventoryState {
            coins: 101,
            items: Vec::new(),
            equipment: Vec::new(),
            next_instance_id: None,
        },
        gender: 1,
        quest_records,
        quests,
        quest_bindings: vec![QuestWireBinding {
            quest_id: 900001,
            objective_index: 0,
            offset: 4,
        }],
        post_quest_records,
        action_bars: vec![ActionBar {
            key: 1,
            value: 2,
            slots: vec![
                ActionBarSlot {
                    key: 10,
                    fields: [0x11; 51],
                },
                ActionBarSlot {
                    key: 20,
                    fields: [0x22; 51],
                },
            ],
        }],
        action_bar_flag: true,
        action_keys: vec![31, 32],
        tutorial_keys: vec![5],
        effects: vec![EffectRecord {
            key: 2,
            payload: vec![0x77; 81],
        }],
        post_effect_tail: vec![0xa5, 0x5a, 0x11],
    }
}

fn empty_reward_bundle(success: u8, marker: u8) -> RewardBundle {
    RewardBundle {
        success,
        fixed_fields: [marker; 60],
        entries: Vec::new(),
        trailing_value: marker as u32,
    }
}

fn body(packet: &[u8]) -> &[u8] {
    assert_eq!(&packet[..3], &[5, 0, 1]);
    let tunnel_len = u32::from_le_bytes(packet[3..7].try_into().unwrap()) as usize;
    assert_eq!(packet.len(), tunnel_len + 7);
    let inner = &packet[7..];
    assert_eq!(u16::from_le_bytes(inner[..2].try_into().unwrap()), 12);
    let body_len = u32::from_le_bytes(inner[2..6].try_into().unwrap()) as usize;
    assert_eq!(inner.len(), body_len + 6);
    &inner[6..]
}

fn counted_string_len(value: &str) -> usize {
    4 + value.len()
}

fn reward_bundle_wire_len(bundle: &startup_selfdata::RewardBundle) -> usize {
    1 + 60
        + 4
        + bundle
            .entries
            .iter()
            .map(|entry| 4 + 1 + 24 + 4 + entry.display_bytes.len() + 5 + entry.kind_fields.len())
            .sum::<usize>()
        + 4
}

fn pre_profile_len(data: &SelfData) -> usize {
    20 + counted_string_len(&data.appearance.head)
        + counted_string_len(&data.appearance.hair)
        + 8
        + counted_string_len(&data.appearance.skin_tone)
        + counted_string_len(&data.appearance.face_paint)
        + counted_string_len(&data.appearance.model_customization)
        + 20
        + 32
        + 12
        + counted_string_len(&data.name.first)
        + counted_string_len(&data.name.last)
        + 56
}

fn profile_wire_len(data: &SelfData, profile: &JobProfile) -> usize {
    let profile_id = profile.map_key as i32;
    profile.opaque_before_equipment_without_key.len()
        + 4
        + 4
        + 12 * data
            .inventory
            .equipment
            .iter()
            .filter(|entry| entry.profile_id == profile_id)
            .count()
        + 4
        + 4
        + profile
            .abilities
            .iter()
            .map(|ability| {
                4 + 4 * ability.kind_fields.len() + if ability.kind == 0 { 0 } else { 7 * 4 + 1 }
            })
            .sum::<usize>()
        + 37 * profile.trailing_records.len()
        + 4
}

fn inventory_count_body_offset(data: &SelfData) -> usize {
    pre_profile_len(data)
        + 4
        + data
            .jobs
            .profiles
            .iter()
            .map(|profile| profile_wire_len(data, profile))
            .sum::<usize>()
        + 4
        + 4
        + data.profile_types.len() * 8
        + 4
        + data
            .collections
            .iter()
            .map(|collection| {
                28 + reward_bundle_wire_len(&collection.reward)
                    + 4
                    + collection.keyed_rows.len() * 33
            })
            .sum::<usize>()
}

fn quest_body_offset(data: &SelfData) -> usize {
    inventory_count_body_offset(data)
        + 4
        + data
            .inventory
            .items
            .iter()
            .map(|item| if item.rental.is_some() { 46 } else { 30 })
            .sum::<usize>()
        + 4
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn private_self_capture_rebuilds_identically() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for the startup byte oracle");
    let packet = fs::read(Path::new(&reference).join("0007-login.bin")).unwrap();
    let decoded = parse_self(&packet);
    assert_eq!(decoded.collections.len(), 253);
    assert_eq!(decoded.inventory.items.len(), 1386);
    assert_eq!(decoded.jobs.profiles.len(), 16);
    assert_eq!(decoded.action_bars.len(), 1);
    assert_eq!(decoded.action_bars[0].slots.len(), 4);
    assert_eq!(decoded.post_effect_tail.len(), 1962);
    let rebuilt = build_self(&decoded);
    assert_eq!(rebuilt.len(), packet.len(), "self packet length mismatch");
    let first_difference = rebuilt
        .iter()
        .zip(&packet)
        .position(|(left, right)| left != right);
    if let Some(offset) = first_difference {
        panic!("self packet differs at packet byte offset {offset}");
    }
}

#[test]
fn synthetic_typed_sections_round_trip() {
    let expected = sample();
    let packet = build_self(&expected);
    let decoded = parse_self_with_quest_bindings(&packet, &expected.quest_bindings);
    assert_eq!(decoded, expected);
    assert_eq!(body(&packet).len(), packet.len() - 13);
}

#[test]
fn built_self_message_contains_another_characters_inventory_equipment_and_pose() {
    let mut data = sample();
    data.identity.guid = 0x9988_7766_5544_3322;
    data.transform.position = [101.5f32, -20.25, 33.75, 1.0].map(f32::to_bits);
    data.transform.rotation = [0.25f32, 0.5, 0.75, 0.0].map(f32::to_bits);
    data.inventory.items = vec![synthetic_item(false, 77)];
    data.inventory.equipment = vec![inventory::EquippedItem {
        profile_id: 4,
        slot: 2,
        item_id: 77,
        category: 3,
    }];

    let built = build_self(&data);
    let decoded = parse_self_with_quest_bindings(&built, &data.quest_bindings);

    assert_eq!(decoded.identity.guid, data.identity.guid);
    assert_eq!(decoded.transform.position, data.transform.position);
    assert_eq!(decoded.transform.rotation, data.transform.rotation);
    assert_eq!(decoded.inventory.items, data.inventory.items);
    assert_eq!(decoded.inventory.equipment, data.inventory.equipment);
}

#[test]
fn synthetic_inventory_zero_one_many_items_uses_server_item_type() {
    let empty = sample();
    let empty_packet = build_self(&empty);
    let mut one = empty.clone();
    one.inventory.items.push(synthetic_item(false, 101));
    let one_packet = build_self(&one);
    let mut many = one.clone();
    many.inventory.items = vec![
        synthetic_item(false, 101),
        synthetic_item(true, 102),
        synthetic_item(false, 103),
    ];
    let many_packet = build_self(&many);
    let count_at = inventory_count_body_offset(&empty);

    assert_eq!(
        &body(&empty_packet)[..count_at],
        &body(&one_packet)[..count_at]
    );
    assert_eq!(one_packet.len(), empty_packet.len() + 30);
    assert_eq!(many_packet.len(), empty_packet.len() + 30 + 46 + 30);
    assert_eq!(
        parse_self_with_quest_bindings(&one_packet, &one.quest_bindings)
            .inventory
            .items,
        one.inventory.items
    );
    assert_eq!(
        parse_self_with_quest_bindings(&many_packet, &many.quest_bindings)
            .inventory
            .items,
        many.inventory.items
    );
}

#[test]
fn synthetic_profile_equipment_uses_inventory_equipped_item_type() {
    let original = sample();
    let first = build_self(&original);
    let mut changed = original.clone();
    changed.inventory.equipment.push(inventory::EquippedItem {
        profile_id: 4,
        slot: 3,
        item_id: 7001,
        category: 9,
    });
    let second = build_self(&changed);
    let decoded = parse_self_with_quest_bindings(&second, &changed.quest_bindings);
    assert_eq!(decoded.inventory.equipment, changed.inventory.equipment);
    assert_eq!(second.len(), first.len() + 12);
    assert_eq!(decoded.jobs.profiles[0].map_key, 4);
}

#[test]
fn synthetic_quest_progress_uses_shared_quest_state_binding() {
    let original = sample();
    let first = build_self(&original);
    let mut changed = original.clone();
    changed.quests.quests.get_mut(&900001).unwrap().counts[0] = 8;
    let second = build_self(&changed);
    let start = quest_body_offset(&original) + changed.quest_bindings[0].offset;
    let range = 13 + start..13 + start + 4;
    assert!(
        (0..first.len())
            .filter(|&index| first[index] != second[index])
            .all(|index| range.contains(&index))
    );
    let parsed = parse_self_with_quest_bindings(&second, &changed.quest_bindings);
    assert_eq!(parsed.quests.quests[&900001].counts[0], 8);
}

#[test]
fn synthetic_action_bar_key_and_slot_changes_round_trip() {
    let original = sample();
    let mut changed = original.clone();
    changed.action_bars[0].key = 7;
    changed.action_bars[0].slots[1].key = 44;
    changed.action_bars[0].slots[1].fields[17] ^= 0x7f;
    let packet = build_self(&changed);
    let parsed = parse_self_with_quest_bindings(&packet, &changed.quest_bindings);
    assert_eq!(parsed.action_bars, changed.action_bars);
    assert_ne!(build_self(&original), packet);
}

#[test]
fn synthetic_collection_reward_and_keyed_rows_round_trip() {
    let mut expected = sample();
    expected.collections.push(CollectionRecord {
        prefix: [1, 2, 3, 4, 5, 6, 7],
        reward: RewardBundle {
            success: 1,
            fixed_fields: [0x55; 60],
            entries: vec![
                RewardEntry {
                    kind: 1,
                    hidden: 1,
                    common_fields: [0x66; 24],
                    display_bytes: b"reward-a".to_vec(),
                    display_fields: [0x77; 5],
                    kind_fields: vec![0x88; 4],
                },
                RewardEntry {
                    kind: 3,
                    hidden: 0,
                    common_fields: [0x99; 24],
                    display_bytes: b"reward-b".to_vec(),
                    display_fields: [0xaa; 5],
                    kind_fields: Vec::new(),
                },
            ],
            trailing_value: 8,
        },
        keyed_rows: vec![CollectionKeyedRow {
            key: 9,
            payload: [0xbb; 29],
        }],
    });
    let packet = build_self(&expected);
    let decoded = parse_self_with_quest_bindings(&packet, &expected.quest_bindings);
    assert_eq!(decoded.collections, expected.collections);
    assert_eq!(decoded, expected);
}

#[test]
fn synthetic_profile_ability_variants_and_sentinel_rows_round_trip() {
    let mut expected = sample();
    expected.jobs.profiles[0].abilities = vec![
        ProfileAbility {
            kind: 1,
            kind_fields: vec![101, 102],
            common_fields: Some([0x1122_3344; 7]),
            common_flag: Some(0xa5),
        },
        ProfileAbility {
            kind: 2,
            kind_fields: vec![201],
            common_fields: Some([0x5566_7788; 7]),
            common_flag: Some(1),
        },
        ProfileAbility {
            kind: 7,
            kind_fields: Vec::new(),
            common_fields: Some([0x99aa_bbcc; 7]),
            common_flag: Some(0),
        },
        ProfileAbility {
            kind: 0,
            kind_fields: Vec::new(),
            common_fields: None,
            common_flag: None,
        },
    ];
    expected.jobs.profiles[0].trailing_records = vec![ProfileTrailingRecord {
        key: 33,
        payload: [0x44; 33],
    }];
    let packet = build_self(&expected);
    let decoded = parse_self_with_quest_bindings(&packet, &expected.quest_bindings);
    assert_eq!(decoded.jobs.profiles, expected.jobs.profiles);
}

#[test]
fn synthetic_quest_list_records_and_nested_objectives_round_trip() {
    let mut expected = sample();
    expected.quest_bindings.clear();
    expected.quests = empty_quest_state();
    expected.quest_records.first.push(QuestFirstRecord {
        prefix: [0x11; 34],
        reward: empty_reward_bundle(1, 0x22),
        objectives: vec![QuestObjectiveRecord {
            key: 55,
            prefix: [0x33; 13],
            reward: empty_reward_bundle(0, 0x44),
            tail: [0x55; 21],
        }],
        tail: [0x66; 6],
    });
    expected.quest_records.second.push(QuestSecondRecord {
        prefix: [0x77; 33],
        reward: empty_reward_bundle(0, 0x88),
        objectives: Vec::new(),
        tail: [0x99; 21],
    });
    let packet = build_self(&expected);
    let decoded = parse_self_with_quest_bindings(&packet, &expected.quest_bindings);
    assert_eq!(decoded.quest_records, expected.quest_records);
    assert_eq!(decoded.quests, expected.quests);
}

#[test]
fn synthetic_post_quest_record_families_round_trip() {
    let mut expected = sample();
    expected
        .post_quest_records
        .keyed_strings
        .push(KeyedStringRecord {
            key_fields: [0x11; 8],
            value_bytes: b"content-a".to_vec(),
            tail: [0x22; 13],
        });
    expected.post_quest_records.recipes.push(RecipeRecord {
        prefix: [0x33; 29],
        components: vec![[0x44; 32], [0x55; 32]],
        tail: [0x66; 9],
    });
    expected
        .post_quest_records
        .variable_records
        .push(VariableRecord {
            prefix: [0x77; 26],
            first_rows: vec![[0x88; 37]],
            second_rows: vec![101, 102],
            third_rows: vec![[0x99; 12]],
            first_string: b"content-b".to_vec(),
            middle: [0xaa; 8],
            second_string: b"content-c".to_vec(),
            tail: [0xbb; 58],
        });
    expected.post_quest_records.pet_source_marker = -7;
    expected.post_quest_records.pet_source_value = 0x0102_0304_0506_0708;
    expected
        .post_quest_records
        .pre_action_records
        .push(PreActionRecord {
            prefix: [0xcc; 25],
            string_bytes: b"content-d".to_vec(),
            tail: [0xdd; 2],
        });
    let packet = build_self(&expected);
    let decoded = parse_self_with_quest_bindings(&packet, &expected.quest_bindings);
    assert_eq!(decoded.post_quest_records, expected.post_quest_records);
}

#[test]
#[ignore = "requires the private captured gateway reference"]
fn captured_opcode12_tutorial_keys_are_not_the_npx_mode_flag() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("OSFR_GATEWAY_REFERENCE must point to the private capture");
    let packet = fs::read(Path::new(&reference).join("0007-login.bin")).unwrap();
    let captured = parse_self(&packet);
    let rebuilt = build_self(&captured);
    let decoded = parse_self(&rebuilt);

    assert_eq!(decoded.tutorial_keys, captured.tutorial_keys);
    println!(
        "opcode12 tutorial_keys rows preserved: {}; no IsInNPXTutorial field in SelfData",
        captured.tutorial_keys.len()
    );
}
