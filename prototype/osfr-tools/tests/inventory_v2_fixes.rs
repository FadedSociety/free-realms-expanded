use osfr_tools::inventory_v2_fixes;

use inventory_v2_fixes::{
    InventoryV2Error, ItemUpdateV2, encode_definition_batch_v2, encode_item_update_v2,
};

fn word(value: i32) -> [u8; 4] {
    value.to_le_bytes()
}

fn current_definition_encoder(definitions: &[Vec<u8>]) -> Vec<u8> {
    // Mirrors the current definitions_packet: available rows go out without
    // checking the owned DefinitionId set.
    let payload_len = 4 + definitions.iter().map(Vec::len).sum::<usize>();
    let mut packet = [35u16.to_le_bytes(), 37u16.to_le_bytes()].concat();
    packet.extend((payload_len as i32).to_le_bytes());
    packet.extend((definitions.len() as i32).to_le_bytes());
    for record in definitions {
        packet.extend_from_slice(record);
    }
    packet
}

fn current_item_update(
    item_id: i32,
    count: i32,
    consumed_count: i32,
    ability_count: i32,
    rental_expiration: i64,
) -> Vec<u8> {
    let mut packet = [38u16.to_le_bytes(), 3u16.to_le_bytes()].concat();
    for value in [item_id, count, -1, consumed_count, ability_count] {
        packet.extend(value.to_le_bytes());
    }
    packet.extend(rental_expiration.to_le_bytes());
    packet
}

#[test]
fn catalogue_encoder_rejects_owned_ids_missing_from_35_37() {
    let records = vec![word(10).to_vec()];
    let old = current_definition_encoder(&records);
    assert_eq!(i32::from_le_bytes(old[8..12].try_into().unwrap()), 1);

    let error = encode_definition_batch_v2(&records, &[10, 20]).unwrap_err();
    assert_eq!(error, InventoryV2Error::MissingDefinitions(vec![20]));
    println!("old 35/37={old:02x?}; owned ids [10, 20], fixed encoder: {error}");
}

#[test]
fn complete_catalogue_keeps_35_37_byte_length_and_definition_count() {
    let records = vec![word(20).to_vec(), word(10).to_vec()];
    let packet = encode_definition_batch_v2(&records, &[10, 20]).unwrap();
    let payload_len = i32::from_le_bytes(packet[4..8].try_into().unwrap());
    let count = i32::from_le_bytes(packet[8..12].try_into().unwrap());

    assert_eq!(&packet[..4], &[35, 0, 37, 0]);
    assert_eq!(payload_len as usize, packet.len() - 8);
    assert_eq!(count, 2);
    assert_eq!(&packet[12..16], &word(10));
    assert_eq!(&packet[16..20], &word(20));
}

#[test]
fn item_update_uses_stored_count_and_the_rental_expiry_not_ability_or_last_cast() {
    let old = current_item_update(77, 9, 1, 6, 0x1122);
    let fixed = encode_item_update_v2(ItemUpdateV2 {
        item_id: 77,
        count: 9,
        item_bar_slot: -1,
        consumed_count: 1,
        stored_count: 3,
        rental_expiration: 0x1122_3344_5566_7788,
    });

    let expected = [
        [38u16.to_le_bytes(), 3u16.to_le_bytes()].concat(),
        word(77).to_vec(),
        word(9).to_vec(),
        word(-1).to_vec(),
        word(1).to_vec(),
        word(3).to_vec(),
        0x1122_3344_5566_7788i64.to_le_bytes().to_vec(),
    ]
    .concat();
    assert_eq!(fixed, expected);
    assert_ne!(old, fixed);
    println!("old 38/3={old:02x?}; fixed={fixed:02x?}");
}
