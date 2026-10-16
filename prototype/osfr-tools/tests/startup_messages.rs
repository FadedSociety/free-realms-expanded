use osfr_tools::startup_messages::{self, ZoneDetails};
use std::{fs, path::Path};

fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
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

fn read_counted_string(bytes: &[u8], offset: &mut usize) -> String {
    let length = u32_at(bytes, *offset) as usize;
    *offset += 4;
    let value = String::from_utf8(bytes[*offset..*offset + length].to_vec()).unwrap();
    *offset += length;
    value
}

#[test]
#[ignore = "requires private startup reference via OSFR_GATEWAY_REFERENCE"]
fn mapped_messages_reproduce_the_private_capture() {
    let reference = std::env::var("OSFR_GATEWAY_REFERENCE")
        .expect("private reference path required for the startup byte oracle");
    let reference = Path::new(&reference);

    let packet = captured(reference, 1, "login");
    let bytes = inner(&packet);
    assert_eq!(startup_messages::build_login_reply(bytes[2]), packet);

    let packet = captured(reference, 2, "login");
    let bytes = inner(&packet);
    assert_eq!(u16::from_le_bytes(bytes[..2].try_into().unwrap()), 166);
    let end = bytes[2..].iter().position(|byte| *byte == 0).unwrap() + 2;
    assert!(bytes[end + 1..].is_empty());
    let environment = std::str::from_utf8(&bytes[2..end]).unwrap();
    assert_eq!(
        startup_messages::build_initialization_parameters(environment),
        packet
    );

    let packet = captured(reference, 3, "login");
    let bytes = inner(&packet);
    assert_eq!(u16::from_le_bytes(bytes[..2].try_into().unwrap()), 43);
    let mut offset = 2;
    let name = read_counted_string(bytes, &mut offset);
    let zone_type = i32_at(bytes, offset);
    offset += 4;
    let tutorial = bytes[offset] != 0;
    offset += 2; // tutorial and captured unknown flag
    let sky = read_counted_string(bytes, &mut offset);
    offset += 1; // captured unknown flag after sky
    let zone_id = u32_at(bytes, offset);
    offset += 4;
    let geometry_id = u32_at(bytes, offset);
    offset += 4;
    assert_eq!(&bytes[offset..], &[0, 0]);
    assert_eq!(
        startup_messages::build_zone_details(&ZoneDetails {
            name: &name,
            zone_type,
            tutorial,
            sky: &sky,
            zone_id,
            geometry_id,
        }),
        packet
    );

    let packet = captured(reference, 33, "ready");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[38, 0, 1, 0]);
    assert_eq!(
        startup_messages::build_hit_points(i32_at(bytes, 4), i32_at(bytes, 8)),
        packet
    );

    let packet = captured(reference, 34, "ready");
    let bytes = inner(&packet);
    assert_eq!(&bytes[..4], &[38, 0, 13, 0]);
    assert_eq!(
        startup_messages::build_mana(i32_at(bytes, 4), i32_at(bytes, 8), bytes[12]),
        packet
    );

    let packet = captured(reference, 39, "ready");
    let bytes = inner(&packet);
    assert_eq!(u16::from_le_bytes(bytes[..2].try_into().unwrap()), 209);
    assert_eq!(u32_at(bytes, 2), 2);
    let count = u32_at(bytes, 6) as usize;
    assert_eq!(bytes.len(), 10 + count * 8);
    let entries = (0..count)
        .map(|index| {
            let offset = 10 + index * 8;
            (u32_at(bytes, offset), u32_at(bytes, offset + 4))
        })
        .collect::<Vec<_>>();
    assert_eq!(startup_messages::build_journal_set(&entries), packet);

    let packet = captured(reference, 43, "ready");
    assert_eq!(
        startup_messages::build_done_sending_preload_characters(),
        packet
    );
}

#[test]
fn synthetic_zone_name_and_id_change_their_fields() {
    let original = startup_messages::build_zone_details(&ZoneDetails {
        name: "zone-a",
        zone_type: 2,
        tutorial: false,
        sky: "sky-a",
        zone_id: 7,
        geometry_id: 12,
    });
    let changed = startup_messages::build_zone_details(&ZoneDetails {
        name: "zone-b-longer",
        zone_type: 2,
        tutorial: false,
        sky: "sky-a",
        zone_id: 19,
        geometry_id: 12,
    });
    assert_ne!(original, changed);
    let bytes = inner(&changed);
    let mut offset = 2;
    assert_eq!(read_counted_string(bytes, &mut offset), "zone-b-longer");
    offset += 6; // type and two booleans
    let _sky = read_counted_string(bytes, &mut offset);
    offset += 1;
    assert_eq!(u32_at(bytes, offset), 19);
}

#[test]
fn synthetic_journal_size_and_values_change_the_message() {
    let one = startup_messages::build_journal_set(&[(101, 3)]);
    let two = startup_messages::build_journal_set(&[(101, 3), (205, 9)]);
    assert_ne!(one, two);
    let bytes = inner(&two);
    assert_eq!(u32_at(bytes, 6), 2);
    assert_eq!((u32_at(bytes, 10), u32_at(bytes, 14)), (101, 3));
    assert_eq!((u32_at(bytes, 18), u32_at(bytes, 22)), (205, 9));
}
