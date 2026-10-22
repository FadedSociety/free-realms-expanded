use osfr_tools::startup_catalogues::{
    self, AbilitySetDefinition, AbilitySetEntry, CharacterStat, CoinStoreEntry, CoinStoreItemLists,
    EffectTag, ItemDefinition, StatUpdate, StatValue,
};
use std::{fs, path::Path};

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn inner(packet: &[u8]) -> &[u8] {
    assert_eq!(&packet[..3], &[5, 0, 1]);
    let length = u32_at(packet, 3) as usize;
    assert_eq!(packet.len(), length + 7);
    &packet[7..]
}

fn captured(reference: &Path, index: usize, phase: &str) -> Vec<u8> {
    fs::read(reference.join(format!("{index:04}-{phase}.bin"))).unwrap()
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16).unwrap();
            let low = (pair[1] as char).to_digit(16).unwrap();
            ((high << 4) | low) as u8
        })
        .collect()
}

fn read_coin_entries(bytes: &[u8], offset: &mut usize, count: usize) -> Vec<CoinStoreEntry> {
    (0..count)
        .map(|_| {
            let id = i32_at(bytes, *offset);
            *offset += 4;
            let words = std::array::from_fn(|_| {
                let value = u32_at(bytes, *offset);
                *offset += 4;
                value
            });
            let flags = std::array::from_fn(|_| {
                let value = bytes[*offset];
                *offset += 1;
                value
            });
            CoinStoreEntry {
                id,
                unknown_words: words,
                unknown_flags: flags,
            }
        })
        .collect()
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn mapped_catalogue_and_status_messages_reproduce_the_private_capture() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for the startup byte oracle");
    let reference = Path::new(&reference);

    let content_path = reference
        .parent()
        .expect("gateway reference must be under lab/references")
        .join("rust-gameplay/inventory.json");
    let content: serde_json::Value =
        serde_json::from_slice(&fs::read(content_path).unwrap()).unwrap();
    let source_definitions = content["definitions"].as_array().unwrap();
    let definition_records = source_definitions
        .iter()
        .map(|definition| decode_hex(definition["serialized_hex"].as_str().unwrap()))
        .collect::<Vec<_>>();
    let definitions = definition_records
        .iter()
        .zip(source_definitions)
        .map(|(record, definition)| ItemDefinition {
            id: definition["id"].as_i64().unwrap() as i32,
            serialized_record: record,
        })
        .collect::<Vec<_>>();
    let packet = captured(reference, 5, "login");
    assert_eq!(
        startup_catalogues::build_item_definitions(&definitions),
        packet
    );

    let packet = captured(reference, 40, "ready");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[165, 0, 1, 0]);
    let static_count = u32_at(bytes, 4) as usize;
    let mut offset = 8;
    let static_items = read_coin_entries(bytes, &mut offset, static_count);
    let dynamic_count = u32_at(bytes, offset) as usize;
    offset += 4;
    let dynamic_items = read_coin_entries(bytes, &mut offset, dynamic_count);
    assert_eq!(offset, bytes.len());
    assert_eq!(
        startup_catalogues::build_coin_store_item_lists(CoinStoreItemLists {
            static_items: &static_items,
            dynamic_items: &dynamic_items,
        }),
        packet
    );

    let packet = captured(reference, 35, "ready");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[36, 0, 5, 0]);
    let id = i32_at(bytes, 4);
    let count = u32_at(bytes, 8) as usize;
    let entries = (0..count)
        .map(|index| {
            let offset = 12 + index * 4;
            assert_eq!(i32_at(bytes, offset), 0);
            AbilitySetEntry {
                kind: 0,
                serialized_fields: &[],
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(12 + count * 4, bytes.len());
    assert_eq!(
        startup_catalogues::build_ability_set_definition(AbilitySetDefinition {
            id,
            entries: &entries,
        }),
        packet
    );

    for index in [36, 37] {
        let packet = captured(reference, index, "ready");
        let bytes = inner(&packet);
        assert_eq!(&bytes[..4], &[38, 0, 16, 0]);
        let id = i32_at(bytes, 4);
        let length = i32_at(bytes, 8) as usize;
        assert_eq!(bytes.len(), 12 + length);
        assert_eq!(
            startup_catalogues::build_effect_tag(EffectTag {
                id,
                definition_blob: &bytes[12..],
            }),
            packet
        );
    }

    let packet = captured(reference, 44, "ready");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[38, 0, 7, 0]);
    let target_guid = u64::from(u32_at(bytes, 4)) | (u64::from(u32_at(bytes, 8)) << 32);
    let count = u32_at(bytes, 12) as usize;
    let stats = (0..count)
        .map(|index| {
            let offset = 16 + index * 12;
            let id = i32_at(bytes, offset);
            let value_kind = u32_at(bytes, offset + 4);
            let raw_value = u32_at(bytes, offset + 8);
            let value = match value_kind {
                0 => StatValue::Integer(raw_value as i32),
                1 => StatValue::Float(f32::from_bits(raw_value)),
                other => panic!("unsupported captured stat value kind {other}"),
            };
            CharacterStat { id, value }
        })
        .collect::<Vec<_>>();
    assert_eq!(16 + count * 12, bytes.len());
    assert_eq!(
        startup_catalogues::build_stat_update(StatUpdate {
            target_guid,
            stats: &stats,
        }),
        packet
    );

    let packet = captured(reference, 0, "login");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[41, 0, 132, 0]);
    assert_eq!(&bytes[4..12], &[0; 8]);
    assert_eq!(
        startup_catalogues::build_overworld_combat_flag(bytes[12] != 0),
        packet
    );
    assert_eq!(
        startup_catalogues::build_client_game_settings(),
        captured(reference, 4, "login")
    );
    // T437 (R): the row's action is 'closeWindow' where the capture says 'Exit'; nothing else moves.
    let announcement = captured(reference, 6, "login");
    assert_eq!(&announcement[38..46], b"\x04\x00\x00\x00Exit");
    let action = &b"\x0b\x00\x00\x00closeWindow"[..];
    let mut expected = [&announcement[..38], action, &announcement[46..]].concat();
    let length = expected.len() as u32 - 7;
    expected[3..7].copy_from_slice(&length.to_le_bytes());
    assert_eq!(startup_catalogues::build_announcement_data(), expected);
    assert_eq!(
        startup_catalogues::build_opcode_14_marker(),
        captured(reference, 42, "ready")
    );
}

#[test]
fn synthetic_item_definitions_change_count_and_records() {
    let mut one_record = 1i32.to_le_bytes().to_vec();
    one_record.extend([0xaa, 0xbb]);
    let mut two_record = 2i32.to_le_bytes().to_vec();
    two_record.push(0xcc);
    let one = [ItemDefinition {
        id: 1,
        serialized_record: &one_record,
    }];
    let two = [
        ItemDefinition {
            id: 1,
            serialized_record: &one_record,
        },
        ItemDefinition {
            id: 2,
            serialized_record: &two_record,
        },
    ];
    let first = startup_catalogues::build_item_definitions(&one);
    let second = startup_catalogues::build_item_definitions(&two);
    assert_ne!(first, second);
    assert_eq!(u32_at(inner(&second), 8), 2);
    assert_eq!(&inner(&second)[12..], [one_record, two_record].concat());
}

#[test]
fn synthetic_coin_store_lists_change_with_content() {
    let entry = CoinStoreEntry {
        id: 7,
        unknown_words: [10, 20, 30],
        unknown_flags: [1, 0, 1],
    };
    let empty = startup_catalogues::build_coin_store_item_lists(CoinStoreItemLists {
        static_items: &[],
        dynamic_items: &[],
    });
    let populated = startup_catalogues::build_coin_store_item_lists(CoinStoreItemLists {
        static_items: &[entry],
        dynamic_items: &[],
    });
    assert_ne!(empty, populated);
    let bytes = inner(&populated);
    assert_eq!(u32_at(bytes, 4), 1);
    assert_eq!(i32_at(bytes, 8), 7);
    assert_eq!(&bytes[24..27], &[1, 0, 1]);
    assert_eq!(u32_at(bytes, 27), 0);
}

#[test]
fn synthetic_ability_set_changes_id_and_entry_count() {
    let empty_slot = AbilitySetEntry {
        kind: 0,
        serialized_fields: &[],
    };
    let one = [empty_slot];
    let two = [empty_slot, empty_slot];
    let first = startup_catalogues::build_ability_set_definition(AbilitySetDefinition {
        id: 3,
        entries: &one,
    });
    let second = startup_catalogues::build_ability_set_definition(AbilitySetDefinition {
        id: 4,
        entries: &two,
    });
    assert_ne!(first, second);
    assert_eq!(i32_at(inner(&second), 4), 4);
    assert_eq!(u32_at(inner(&second), 8), 2);
}

#[test]
fn synthetic_effect_tag_changes_id_and_definition_blob() {
    let first = startup_catalogues::build_effect_tag(EffectTag {
        id: 9,
        definition_blob: &[1, 2],
    });
    let second = startup_catalogues::build_effect_tag(EffectTag {
        id: 10,
        definition_blob: &[1, 2, 3],
    });
    assert_ne!(first, second);
    let bytes = inner(&second);
    assert_eq!(i32_at(bytes, 4), 10);
    assert_eq!(i32_at(bytes, 8), 3);
    assert_eq!(&bytes[12..], &[1, 2, 3]);
}

#[test]
fn synthetic_stat_update_changes_target_and_rows() {
    let one = [CharacterStat {
        id: 2,
        value: StatValue::Integer(8),
    }];
    let two = [
        CharacterStat {
            id: 4,
            value: StatValue::Float(1.5),
        },
        CharacterStat {
            id: 6,
            value: StatValue::Integer(1),
        },
    ];
    let first = startup_catalogues::build_stat_update(StatUpdate {
        target_guid: 1,
        stats: &one,
    });
    let second = startup_catalogues::build_stat_update(StatUpdate {
        target_guid: 0x1_0000_0002,
        stats: &two,
    });
    assert_ne!(first, second);
    let bytes = inner(&second);
    assert_eq!(u32_at(bytes, 4), 2);
    assert_eq!(u32_at(bytes, 8), 1);
    assert_eq!(u32_at(bytes, 12), 2);
    assert_eq!(i32_at(bytes, 16), 4);
    assert_eq!(u32_at(bytes, 20), 1);
    assert_eq!(u32_at(bytes, 24), 1.5f32.to_bits());
}

#[test]
fn synthetic_overworld_combat_flag_changes_only_the_known_flag() {
    let inactive = startup_catalogues::build_overworld_combat_flag(false);
    let active = startup_catalogues::build_overworld_combat_flag(true);
    assert_ne!(inactive, active);
    assert_eq!(inner(&inactive).len(), 13);
    assert_eq!(&inner(&active)[4..12], &[0; 8]);
    assert_eq!(inner(&active)[12], 1);
}
