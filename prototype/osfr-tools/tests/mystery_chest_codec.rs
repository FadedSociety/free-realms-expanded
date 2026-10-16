//! Focused byte-level tests for the Mystery Chest codec.
//!
//! Expected bytes below are laid out independently by hand (little-endian field order), never by
//! calling the codec's own encoders. Source provenance for the layout:
//! - Open request: base inventory packet `FUN_00a10400` writes `0x2a` then the sub-opcode
//!   (`pseudocode.c` 1268375-1268385). Use sub-opcode `0x0f` (`FUN_00c10940`, 1664929-1664950);
//!   check sub-opcode `0x10` (`FUN_00c109f0`, 1664964-1664985). Pinned
//!   `Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs` 423-424 agrees
//!   (15 use, 16 check). The lead reproduced a `0x0e` (14) check sending nothing.
//! - Check reply: base client-update packet `FUN_0089d180` writes `0x26` then the sub-opcode
//!   (973123-973133); `ClientUpdatePacketImmediateActivationCheck` uses `0x1e` (`FUN_0089ea30`,
//!   974298-974319). Body order matches the donor
//!   `ClientUpdatePacketImmediateActivationCheck.Serialize` (5x i32 then bool) and the Lua
//!   `InventoryHandler.OnIsItemUsable(success, guid, catId, bundleId, titleStrId, bodyStrId)`.
//! - Reward bundle: `RewardBasePacket` writes `0x32` + one byte subtype (`FUN_00b88ad0`,
//!   1559710-1559724; base reader `FUN_00b89190`, 1560136-1560159). Empty subtype-1 body is 69
//!   bytes per `FUN_008e7930` (1032033-1032188) and the reward note; entry fields follow pinned
//!   `RewardBundleEntryBase`/`RewardBundleEntryItem` (item entry adds a trailing i32).

use osfr_tools::mystery_chest;

use std::io::ErrorKind;

use mystery_chest::{Request, check_reply, item_reward, parse_request};

const OPCODE_INVENTORY: u16 = 42;
const OPCODE_CLIENT_UPDATE: u16 = 38;
const OPCODE_REWARD: u16 = 50;

const SUBUSE: u16 = 15;
const SUBCHECK: u16 = 16;
const SUBCHECK_REPLY: u16 = 30;
const SUBREWARD_BUNDLE: u8 = 1;

fn inventory_request(subtype: u16, item_id: i32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&OPCODE_INVENTORY.to_le_bytes());
    bytes.extend_from_slice(&subtype.to_le_bytes());
    bytes.extend_from_slice(&item_id.to_le_bytes());
    bytes
}

/// Independent hand-layout of the subtype-1 reward bundle with a single item entry.
///
/// Offsets (body-relative in parentheses) are the ones asserted by the reward note:
/// coins 4, XP 8, kind 16, source GUID 40, player GUID 48, icon 56, name 60, count 64,
/// entry 68..110, trailing word 110..114.
fn expected_item_reward(
    source_guid: u64,
    player_guid: u64,
    item_id: i32,
    definition_id: i32,
    tint: i32,
    icon_tint: i32,
    icon_id: i32,
    name_id: i32,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(114);
    // Root 50 with the byte subtype 1.
    bytes.extend_from_slice(&OPCODE_REWARD.to_le_bytes());
    bytes.push(SUBREWARD_BUNDLE);
    // Body 0: success flag.
    bytes.push(1);
    // Body 1 and 5: coin and XP award amounts stay zero for an item-only grant.
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    // Body 9: unresolved word, zero.
    bytes.extend_from_slice(&0i32.to_le_bytes());
    // Body 13: reward kind; 16 is the item-notification (Mystery Chest) kind.
    bytes.extend_from_slice(&16i32.to_le_bytes());
    // Body 17 and 21: unresolved words, zero.
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    // Body 25: multiplier, exactly 1.0f32.
    bytes.extend_from_slice(&1.0f32.to_le_bytes());
    // Body 29 and 33: unresolved words, zero.
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    // Body 37 and 45: GUIDs.
    bytes.extend_from_slice(&source_guid.to_le_bytes());
    bytes.extend_from_slice(&player_guid.to_le_bytes());
    // Body 53 and 57: panel icon and name.
    bytes.extend_from_slice(&icon_id.to_le_bytes());
    bytes.extend_from_slice(&name_id.to_le_bytes());
    // Body 61: entry count.
    bytes.extend_from_slice(&1i32.to_le_bytes());
    // Entry: type 1 (item), hidden flag, icon id, icon tint id, name id, quantity,
    // definition id, inventory tint, empty description, item text color, members-only,
    // then the item-specific instance GUID.
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&icon_id.to_le_bytes());
    bytes.extend_from_slice(&icon_tint.to_le_bytes());
    bytes.extend_from_slice(&name_id.to_le_bytes());
    bytes.extend_from_slice(&1i32.to_le_bytes());
    bytes.extend_from_slice(&definition_id.to_le_bytes());
    bytes.extend_from_slice(&tint.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&item_id.to_le_bytes());
    // Appended final word after the entries.
    bytes.extend_from_slice(&0i32.to_le_bytes());
    bytes
}

#[test]
fn check_and_use_requests_parse_with_their_native_subtypes() {
    let check = inventory_request(SUBCHECK, 1234);
    assert_eq!(
        parse_request(&check).unwrap(),
        Some(Request::Check(1234)),
        "42/16 is the native check request (FUN_00c109f0 passes 0x10)"
    );

    let use_item = inventory_request(SUBUSE, 987_654);
    assert_eq!(
        parse_request(&use_item).unwrap(),
        Some(Request::Use(987_654)),
        "42/15 is the native use request (FUN_00c10940 passes 0x0f)"
    );
}

#[test]
fn subtype14_is_not_a_check_request() {
    let legacy = inventory_request(14, 555);
    assert_eq!(
        parse_request(&legacy).unwrap(),
        None,
        "the lead reproduced subtype 14 producing no client dialog"
    );
}

#[test]
fn unrelated_packets_are_ignored() {
    // Reward bundle 50/1.
    assert_eq!(parse_request(&[0x32, 0x00, 0x01, 0x00]).unwrap(), None);
    // Client-update root 38.
    assert_eq!(parse_request(&[0x26, 0x00, 0x1e, 0x00]).unwrap(), None);
    // Unknown inventory subtype 17.
    assert_eq!(parse_request(&inventory_request(17, 5)).unwrap(), None);
    // Zero-length and one-byte buffers cannot name an owned packet.
    assert!(parse_request(&[]).is_err());
    assert!(parse_request(&[0x2a]).is_err());
    assert!(
        parse_request(&[0x2a, 0x00]).is_err(),
        "an owned root without a subtype is truncated, not unrelated"
    );
}

#[test]
fn every_truncation_of_an_owned_request_is_an_error() {
    for subtype in [SUBCHECK, SUBUSE] {
        let full = inventory_request(subtype, 4242);
        for len in 0..full.len() {
            let err = parse_request(&full[..len]).expect_err("truncated request must fail");
            if len < 2 {
                assert_eq!(err.kind(), ErrorKind::UnexpectedEof, "len {len}");
            }
        }
        // 2..7 bytes carry the owned opcode but cannot complete the fixed body.
        for len in 2..full.len() {
            let err = parse_request(&full[..len]).expect_err("short owned request must fail");
            assert_eq!(err.kind(), ErrorKind::UnexpectedEof, "len {len}");
        }
    }
}

#[test]
fn trailing_bytes_on_an_owned_request_are_an_error() {
    let mut full = inventory_request(SUBCHECK, 7);
    full.push(0);
    let err = parse_request(&full).expect_err("trailing byte must fail");
    assert_eq!(err.kind(), ErrorKind::InvalidData);

    let mut long = inventory_request(SUBUSE, 7);
    long.extend_from_slice(&[1, 2, 3, 4]);
    let err = parse_request(&long).expect_err("trailing payload must fail");
    assert_eq!(err.kind(), ErrorKind::InvalidData);
}

#[test]
fn non_positive_instance_ids_are_rejected() {
    for id in [0, -1, i32::MIN] {
        for subtype in [SUBCHECK, SUBUSE] {
            let err = parse_request(&inventory_request(subtype, id))
                .expect_err("non-positive instance id must fail");
            assert_eq!(err.kind(), ErrorKind::InvalidData, "id {id}");
        }
    }
}

#[test]
fn check_reply_matches_the_client_reader_bytes() {
    let reply = check_reply(7, true);
    let expected: [u8; 25] = [
        0x26, 0x00, // root 38
        0x1e, 0x00, // subtype 30
        0x07, 0x00, 0x00, 0x00, // item instance id 7
        0xf3, 0x00, 0x00, 0x00, // category 243
        0x9d, 0x13, 0x00, 0x00, // bundle 5021
        0xfa, 0xa8, 0x06, 0x00, // title string 436474
        0xf8, 0xa8, 0x06, 0x00, // body string 436472
        0x01, // success
    ];
    assert_eq!(reply, expected);

    let denied = check_reply(7, false);
    assert_eq!(denied.len(), 25);
    assert_eq!(&denied[..24], &expected[..24]);
    assert_eq!(denied[24], 0x00);
}

#[test]
fn check_reply_carries_the_requested_instance_id() {
    let first = check_reply(0x0102_0304, true);
    assert_eq!(&first[4..8], &[0x04, 0x03, 0x02, 0x01]);
    let second = check_reply(-2, false);
    assert_eq!(&second[4..8], &[0xfe, 0xff, 0xff, 0xff]);
}

#[test]
fn item_reward_matches_the_independent_byte_layout() {
    let source_guid = 0x1122_3344_5566_7788u64;
    let player_guid = 0x99aa_bbcc_ddee_ff00u64;
    let item_id = 9_001i32;
    let definition_id = 78_138i32;
    let tint = -1i32;
    let icon_tint = -1i32;
    let icon_id = 12_345i32;
    let name_id = 555i32;

    let actual = item_reward(
        source_guid,
        player_guid,
        item_id,
        definition_id,
        tint,
        icon_tint,
        icon_id,
        name_id,
    )
    .expect("valid reward request");

    let expected = expected_item_reward(
        source_guid,
        player_guid,
        item_id,
        definition_id,
        tint,
        icon_tint,
        icon_id,
        name_id,
    );
    assert_eq!(actual.len(), 114, "3 header + 65 base + 42 entry + 4 word");
    assert_eq!(actual, expected);
}

#[test]
fn item_reward_field_offsets_match_the_note_and_native_reader() {
    let packet = item_reward(
        0x0102_0304_0506_0708,
        0x1112_1314_1516_1718,
        0x0000_2a2b,
        0x0001_3132,
        -3,
        -4,
        0x0000_4142,
        0x0000_5152,
    )
    .unwrap();

    assert_eq!(&packet[0..2], &[0x32, 0x00], "root 50");
    assert_eq!(packet[2], 1, "one byte subtype 1");
    assert_eq!(packet[3], 1, "success flag");
    assert_eq!(&packet[4..8], &[0, 0, 0, 0], "coins stay zero");
    assert_eq!(&packet[8..12], &[0, 0, 0, 0], "XP stays zero");
    assert_eq!(&packet[12..16], &[0, 0, 0, 0]);
    assert_eq!(
        &packet[16..20],
        &16i32.to_le_bytes(),
        "kind 16 selects the Mystery Chest item notification"
    );
    assert_eq!(&packet[20..24], &[0, 0, 0, 0]);
    assert_eq!(&packet[24..28], &[0, 0, 0, 0]);
    assert_eq!(
        &packet[28..32],
        &1.0f32.to_le_bytes(),
        "multiplier must be exactly 1.0"
    );
    assert_eq!(&packet[32..36], &[0, 0, 0, 0]);
    assert_eq!(&packet[36..40], &[0, 0, 0, 0]);
    assert_eq!(
        &packet[40..48],
        &0x0102_0304_0506_0708u64.to_le_bytes(),
        "source guid"
    );
    assert_eq!(
        &packet[48..56],
        &0x1112_1314_1516_1718u64.to_le_bytes(),
        "player guid"
    );
    assert_eq!(&packet[56..60], &0x0000_4142i32.to_le_bytes(), "icon id");
    assert_eq!(&packet[60..64], &0x0000_5152i32.to_le_bytes(), "name id");
    assert_eq!(&packet[64..68], &1i32.to_le_bytes(), "one entry");

    let entry = &packet[68..110];
    assert_eq!(&entry[0..4], &1i32.to_le_bytes(), "entry type 1 (item)");
    assert_eq!(entry[4], 0, "not hidden");
    assert_eq!(&entry[5..9], &0x0000_4142i32.to_le_bytes(), "entry icon id");
    assert_eq!(&entry[9..13], &(-4i32).to_le_bytes(), "entry icon tint id");
    assert_eq!(
        &entry[13..17],
        &0x0000_5152i32.to_le_bytes(),
        "entry name id"
    );
    assert_eq!(&entry[17..21], &1i32.to_le_bytes(), "quantity 1");
    assert_eq!(
        &entry[21..25],
        &0x0001_3132i32.to_le_bytes(),
        "definition id"
    );
    assert_eq!(&entry[25..29], &(-3i32).to_le_bytes(), "inventory tint");
    assert_eq!(&entry[29..33], &[0, 0, 0, 0], "empty description length");
    assert_eq!(&entry[33..37], &[0, 0, 0, 0], "item text color");
    assert_eq!(entry[37], 0, "not members only");
    assert_eq!(
        &entry[38..42],
        &0x0000_2a2bi32.to_le_bytes(),
        "item instance guid"
    );

    assert_eq!(
        &packet[110..114],
        &[0, 0, 0, 0],
        "appended final word after the entries"
    );
}

#[test]
fn item_reward_rejects_non_positive_ids_but_keeps_unknown_fields() {
    for bad in [0, -9] {
        assert_eq!(
            item_reward(1, 2, bad, 3, 0, 0, 0, 0).unwrap_err().kind(),
            ErrorKind::InvalidData,
            "item id {bad}"
        );
        assert_eq!(
            item_reward(1, 2, 3, bad, 0, 0, 0, 0).unwrap_err().kind(),
            ErrorKind::InvalidData,
            "definition id {bad}"
        );
    }
    // -1 tint fields stay byte-identical to the caller's input (originaldata sentinel).
    let packet = item_reward(1, 2, 3, 4, -1, -1, 5, 6).unwrap();
    assert_eq!(&packet[93..97], &(-1i32).to_le_bytes());
    assert_eq!(&packet[77..81], &(-1i32).to_le_bytes());
}

#[test]
fn item_reward_preserves_maximal_and_minimal_values() {
    let packet = item_reward(
        u64::MAX,
        0,
        i32::MAX,
        i32::MAX,
        i32::MIN,
        i32::MAX,
        i32::MIN,
        i32::MAX,
    )
    .unwrap();
    assert_eq!(
        &packet[40..48],
        &u64::MAX.to_le_bytes(),
        "source guid is unsigned"
    );
    assert_eq!(
        &packet[106..110],
        &i32::MAX.to_le_bytes(),
        "entry item guid keeps item id"
    );
    let expected = expected_item_reward(
        u64::MAX,
        0,
        i32::MAX,
        i32::MAX,
        i32::MIN,
        i32::MAX,
        i32::MIN,
        i32::MAX,
    );
    assert_eq!(packet, expected);
}
