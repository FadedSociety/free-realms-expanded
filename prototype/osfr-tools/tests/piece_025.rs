//! Housing placement client wire (MMO work queue item 025).
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `lab/decompiled/client/FreeRealms/pseudocode.c` — the client writes every housing packet body as
//!   `u16 base opcode (127)` then `u16 inner opcode` (`FUN_00aba910`, base opcode stored at source
//!   `+0x04`, inner opcode at `+0x08` by `FUN_00aa4990`).
//! * Request writers recovered from the same export:
//!   `FUN_00abcb90` PlaceFixtureRequest (opcode 1, body u32/u32/u32/u8, ctor `FUN_00aa5340`),
//!   `FUN_00abcb30` SetEditMode (opcode 6, body u8, ctor `FUN_00aa4cc0`),
//!   `FUN_00abca80` EnterRequest (opcode 19, body u64/u32/u8, ctor `FUN_00aa83c0`),
//!   `FUN_00abcd90` PreviewByItemDefinitionIdRequest (opcode 20, body u32, ctor `FUN_00aa54f0`).
//! * `lab/references/FreeRealms-Legacy/src/Sanctuary.Packet/BaseHousingPacket/
//!   ClientHousingPacketEnterRequest.cs` and `ClientHousingPacketSetEditMode.cs` are the pinned
//!   community readers and independently agree with the two client writers above.
//! * `FUN_00ac0510` writes PickupFixture (opcode 3) as header + one u64 (`FUN_008fd1a0`).
//! * `FUN_00abf620` writes SaveFixture (opcode 5) as header + u64 + two 16-byte quads
//!   (`FUN_0091a640` writes four u32) + u32 + an undecoded nested block. The nested block stays an
//!   opaque tail here; no reconstructed field names are attached to it.
//!
//! This file is a `#[path]` include so it compiles and exercises the real module before the lead
//! registers it in `src/lib.rs`. Integration step for the lead: register the module, drop the
//! include, and route decoded requests into `housing.rs` (item 006).
#[path = "../src/restoration/piece_025.rs"]
#[allow(dead_code)]
mod piece_025;

use piece_025::{BASE_OPCODE, HousingWireMessage, SaveFixture, WireError, decode, encode, schema};

fn place_fixture() -> HousingWireMessage {
    HousingWireMessage::PlaceFixtureRequest {
        first: 0x1122_3344,
        second: 0x5566_7788,
        third: 0x99aa_bbcc,
        flag: 0x01,
    }
}

#[test]
fn place_fixture_request_matches_recovered_writer_order() {
    let bytes = encode(&place_fixture());
    assert_eq!(
        bytes,
        vec![
            0x7f, 0x00, // base opcode 127, u16 little endian
            0x01, 0x00, // inner opcode 1
            0x44, 0x33, 0x22, 0x11, // u32
            0x88, 0x77, 0x66, 0x55, // u32
            0xcc, 0xbb, 0xaa, 0x99, // u32
            0x01, // u8
        ]
    );
    assert_eq!(decode(&bytes).unwrap(), place_fixture());
}

#[test]
fn set_edit_mode_is_one_byte_after_the_header() {
    let message = HousingWireMessage::SetEditMode { in_edit_mode: true };
    let bytes = encode(&message);
    assert_eq!(bytes, vec![0x7f, 0x00, 0x06, 0x00, 0x01]);
    assert_eq!(decode(&bytes).unwrap(), message);
}

#[test]
fn enter_request_matches_the_community_reader() {
    let message = HousingWireMessage::EnterRequest {
        house_guid: 0x0102_0304_0506_0708,
        unknown: 0x0000_0007,
        unknown_flag: false,
    };
    let bytes = encode(&message);
    assert_eq!(
        bytes,
        vec![
            0x7f, 0x00, 0x13, 0x00, // opcode 19
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // u64
            0x07, 0x00, 0x00, 0x00, // u32
            0x00, // bool
        ]
    );
    assert_eq!(decode(&bytes).unwrap(), message);
}

#[test]
fn preview_request_carries_one_definition_id() {
    let message = HousingWireMessage::PreviewByItemDefinitionIdRequest {
        item_definition_id: 0x0f00_1234,
    };
    let bytes = encode(&message);
    assert_eq!(bytes, vec![0x7f, 0x00, 0x14, 0x00, 0x34, 0x12, 0x00, 0x0f]);
    assert_eq!(decode(&bytes).unwrap(), message);
}

#[test]
fn pickup_fixture_carries_the_fixture_guid() {
    let message = HousingWireMessage::PickupFixture {
        fixture_guid: 0x0f00_0000_0000_0001,
    };
    let bytes = encode(&message);
    assert_eq!(
        bytes,
        vec![
            0x7f, 0x00, 0x03, 0x00, //
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f,
        ]
    );
    assert_eq!(decode(&bytes).unwrap(), message);
}

#[test]
fn save_fixture_keeps_the_undecoded_tail() {
    let message = HousingWireMessage::SaveFixture(SaveFixture {
        house_guid: 0x0f00_0000_0000_0002,
        quad_a: [1, 2, 3, 4],
        quad_b: [5, 6, 7, 8],
        value: 0x2a,
        tail: vec![0xde, 0xad, 0xbe, 0xef],
        trailing: 0x0f00,
    });
    let bytes = encode(&message);
    assert_eq!(
        bytes,
        vec![
            0x7f, 0x00, 0x05, 0x00, // opcode 5
            0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0f, // house guid
            0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, // quad a
            0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x06, 0x00,
            0x00, 0x00, // quad b
            0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x2a, 0x00, 0x00, 0x00, // u32
            0xde, 0xad, 0xbe, 0xef, // opaque tail
            0x00, 0x0f, 0x00, 0x00, // trailing u32
        ]
    );
    assert_eq!(decode(&bytes).unwrap(), message);
}

#[test]
fn decode_rejects_wrong_base_opcode_truncation_and_unknown_opcodes() {
    assert_eq!(
        decode(&[0x7e, 0x00, 0x06, 0x00, 0x01]),
        Err(WireError::WrongBaseOpcode { found: 0x7e })
    );
    assert_eq!(decode(&[0x7f, 0x00, 0x06]), Err(WireError::TooShort));
    assert_eq!(decode(&[0x7f, 0x00, 0x06, 0x00]), Err(WireError::Truncated));
    assert_eq!(
        decode(&[0x7f, 0x00, 0x07, 0x00]),
        Err(WireError::UnknownOpcode { opcode: 7 })
    );
    assert_eq!(
        decode(&[0x7f, 0x00, 0x14, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05]),
        Err(WireError::TrailingBytes { count: 1 })
    );
}

#[test]
fn schema_table_matches_the_encoder() {
    assert_eq!(BASE_OPCODE, 127);
    let table = schema();
    assert_eq!(table.len(), 6);
    for entry in table {
        let message = entry.sample();
        let bytes = encode(&message);
        assert_eq!(bytes.len(), entry.wire_size(), "{} size", entry.packet);
        assert_eq!(
            decode(&bytes).unwrap(),
            message,
            "{} round trip",
            entry.packet
        );
        assert_eq!(
            &bytes[2..4],
            &entry.opcode.to_le_bytes(),
            "{} opcode",
            entry.packet
        );
    }
}
