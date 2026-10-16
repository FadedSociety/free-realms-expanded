//! House loading and visiting (piece 022) acceptance for `src/restoration/piece_022.rs`.
//!
//! Every `VEC_*` byte string below is a whole tunneled application message recovered by
//! `lab/reports/piece-022-20261009T025254Z/extract-entrance.py` from the preserved community
//! capture `lab/references/OpenSourceFreeRealms/ofrserver/Packets/p2.pcap` (sha256
//! `65ac0801bf49fa9af4ee0bc95568b751392ec4d594fe6ac4e89605f9ee0d0108`). The same session is
//! re-captured in `p12.pcap` (sha256 `99309eb5d3287bb458bf3e79332efcfc7ded3ba742077bb7258ab34a8addfe97`)
//! and `packets_2.pcapng` (sha256 `221b8d548d0c01bea5f84bb0eadece91e7902ec0c81ffddd989cdea6e0775f37`);
//! all three agree byte for byte for these messages.
//!
//! Cross-source facts these tests rely on (see `notes/Research/Pieces/022 House loading and
//! visiting.md`):
//! * the housing family is base packet 127 / `BaseHousingPacket`; `ClientHousingPacketEnterRequest`
//!   is sub-opcode 19, `ClientHousingPacketRequestGrant` 23, `HousingPacketZoneData` 45 and
//!   `HousingPacketFixtureUpdate` 40 in both the supplied server enum and the community model;
//! * the native client constructs the same family (`FUN_00aa83c0` = 0x13 EnterRequest,
//!   `FUN_00ab07a0` = 0x2d ZoneData with `IsPreview`/`Unused`/`HeadSize` default 10);
//! * `ClientUpdatePacketBeginZoning` (inner opcode 31) is the zone-entry packet; the captured one
//!   names the housing lot zone `hsg_emptylot_seaside_beach_01` and matches the community test
//!   constants (440.632, -0.071, 432.801), (-0.9999741, 0.0, -0.0072035603, 0.0), geometry 214.
//!
//! This file uses a `#[path]` include so the module compiles before the lead registers
//! `pub mod piece_022;` in `src/lib.rs`. Integration step for the lead: register the module
//! (or move it out of `restoration/`) and drop the include.
#[path = "../src/restoration/piece_022.rs"]
#[allow(dead_code)]
mod piece_022;

use piece_022::{
    BASE_PACKET_CLIENT_BEGIN_ZONING, BASE_PACKET_HOUSING, CLIENT_ENTER_REQUEST,
    CLIENT_REQUEST_GRANT, DEFAULT_HEAD_SIZE, EntranceError, EntranceRole, HOUSING_FIXTURE_UPDATE,
    HOUSING_ZONE_DATA, HouseRecord, HousingEntranceRouter, ZoneDataFields, ZoneEntry,
    begin_zoning_message, decode_enter_request, decode_request_grant, zone_data_message,
};
use sha2::{Digest, Sha256};

/// `p2.pcap` flow 2, frame 27110, reliable seq 2008: `7f 00 17 00` (127/23, empty body).
const VEC_REQUEST_GRANT: &str = "7f001700";
/// `p2.pcap` flow 2, frame 27195, reliable seq 2010: 127/19 with house guid
/// `0x07f53169b82d10da` = 573418857690894554, unknown int 0, unknown bool false.
const VEC_ENTER_REQUEST: &str = "7f001300da102db86931f5070000000000";
/// `p2.pcap` flow 0, frame 27311, reliable seq 10371: inner opcode 31 for
/// `hsg_emptylot_seaside_beach_01`.
const VEC_BEGIN_ZONING: &str = "1f001d0000006873675f656d7074796c6f745f736561736964655f62656163685f303102000000e550dc43736891bd8766d8430000803f4dfe7fbf00000000d80becbb0000000011000000736b795f7365617369646532342e786d6c00010100000000000000d60000000100";
/// `p2.pcap` flow 0, frame 29881, reliable seq 12618: first 127/45 zone-data response,
/// second flag byte set.
const VEC_ZONE_DATA_FIRST: &str = "7f002d0000010a00000091ca73956acc524bda102db86931f50781200000100000007b6e317d20436f6f6c20436173746c65070000004d61c3bf68656db38100000000000000000000000000000000000000000100000000000000000000000000000000000013000000000000000000000000000000";
/// `p2.pcap` flow 0, frame 31311, reliable seq 13616: later 127/45 zone-data response with the
/// same instance data and the second flag byte clear.
const VEC_ZONE_DATA_LATER: &str = "7f002d0000000a00000091ca73956acc524bda102db86931f50781200000100000007b6e317d20436f6f6c20436173746c65070000004d61c3bf68656db38100000000000000000000000000000000000000000100000000000000000000000000000000000013000000000000000000000000000000";
/// `p2.pcap` flow 0, frame 31507, reliable seq 13669: first 127/40 fixture update. It binds
/// fixture guid `0x07f54ca17bf5c001` to the same house guid the enter request named.
const VEC_FIXTURE_UPDATE_PREFIX: &str = "7f00280001c0f57ba14cf507da102db86931f507";

const SHA_REQUEST_GRANT: &str = "21759f50d68355847fde34569f13761327727d1b0e225feffb48dfe38e4b8bbb";
const SHA_ENTER_REQUEST: &str = "f41afd28fc71267d09fe252b6ffed2b4a30fcc963db51175272e731c49d60205";
const SHA_BEGIN_ZONING: &str = "f3b03575443e99b5d3bd15f24752eedeec258375ac0d995a876a78cb5a435f07";
const SHA_ZONE_DATA_FIRST: &str =
    "24a4e13793bab260e7eedcb293d5715c642a07790a50befaa5028792cb816200";
const SHA_ZONE_DATA_LATER: &str =
    "d00b1b4ba137911bb123e99116295b8f3a6c0ca5f44ddb915117afc6d4831ea6";

const HOUSE_GUID: u64 = 573_418_857_690_894_554;
const OWNER_GUID: u64 = 5_427_625_259_080_862_353;
const GUEST_GUID: u64 = 0x2222_3333_4444_5555;
const ZONE_NAME: &str = "hsg_emptylot_seaside_beach_01";
const ZONE_SKY: &str = "sky_seaside24.xml";
const HOUSE_NAME: &str = "{n1} Cool Castle";
const OWNER_NAME: &str = "Ma\u{ff}hem";
const CAPTURE_HEAD_SIZE: i32 = 10;
const CAPTURE_NAME_ID: i32 = 8321;
const CAPTURE_ICON_ID: i32 = 33203;
const CAPTURE_TAIL_WORD: u32 = 0x0013_0000;
/// Fixture guid carried by the first captured 127/40 update (`c0f57ba14cf507` little-endian).
const FIXTURE_GUID: u64 = 0x07f5_4ca1_7bf5_c001;

fn hex(text: &str) -> Vec<u8> {
    assert!(text.len() % 2 == 0, "odd hex length");
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn captured_house() -> HouseRecord {
    HouseRecord {
        house_guid: HOUSE_GUID,
        owner_guid: OWNER_GUID,
        zone_definition_id: 1,
        zone_name: ZONE_NAME.to_string(),
        geometry_id: 214,
        sky: ZONE_SKY.to_string(),
        spawn_position: [
            f32::from_bits(0x43dc_50e5),
            f32::from_bits(0xbd91_6873),
            f32::from_bits(0x43d8_6687),
        ],
        spawn_rotation: [
            f32::from_bits(0xbf7f_fe4d),
            0.0,
            f32::from_bits(0xbbec_0bd8),
            0.0,
        ],
        name_id: CAPTURE_NAME_ID,
        house_name: HOUSE_NAME.to_string(),
        owner_name: OWNER_NAME.to_string(),
        icon_id: CAPTURE_ICON_ID,
        fixture_count: 0,
        furniture_score: 0,
        is_locked: false,
        is_members_only: false,
        is_flora_allowed: true,
        tail_word: CAPTURE_TAIL_WORD,
    }
}

fn router_with_captured_house() -> HousingEntranceRouter {
    let mut router = HousingEntranceRouter::new();
    router.register_house(captured_house()).unwrap();
    router
}

#[test]
fn captured_vectors_match_their_recorded_hashes() {
    assert_eq!(digest(&hex(VEC_REQUEST_GRANT)), SHA_REQUEST_GRANT);
    assert_eq!(digest(&hex(VEC_ENTER_REQUEST)), SHA_ENTER_REQUEST);
    assert_eq!(digest(&hex(VEC_BEGIN_ZONING)), SHA_BEGIN_ZONING);
    assert_eq!(digest(&hex(VEC_ZONE_DATA_FIRST)), SHA_ZONE_DATA_FIRST);
    assert_eq!(digest(&hex(VEC_ZONE_DATA_LATER)), SHA_ZONE_DATA_LATER);
}

#[test]
fn captured_request_grant_and_enter_request_decode() {
    assert_eq!(BASE_PACKET_HOUSING, 127);
    assert_eq!(CLIENT_REQUEST_GRANT, 23);
    assert_eq!(CLIENT_ENTER_REQUEST, 19);
    decode_request_grant(&hex(VEC_REQUEST_GRANT)).unwrap();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    assert_eq!(request.house_guid, HOUSE_GUID);
    assert_eq!(request.unknown, 0);
    assert!(!request.unknown_bool);
}

#[test]
fn begin_zoning_encoder_reproduces_the_capture() {
    assert_eq!(BASE_PACKET_CLIENT_BEGIN_ZONING, 31);
    let house = captured_house();
    let entry = ZoneEntry::from_house(&house);
    assert_eq!(entry.zone_name, ZONE_NAME);
    assert_eq!(entry.zone_id, 1);
    assert_eq!(entry.geometry_id, 214);
    assert_eq!(entry.zone_type, 2);
    // The community handler's commented test vector for this lot quotes the same numbers these
    // bits decode to, which is the cross-source check that the zone-entry geometry is the real one.
    assert!((entry.position[0] - 440.632).abs() < 1e-3);
    assert!((entry.position[1] - -0.071).abs() < 1e-3);
    assert!((entry.position[2] - 432.801).abs() < 1e-3);
    assert!((entry.rotation[0] - -0.999_974_1).abs() < 1e-6);
    assert!((entry.rotation[2] - -0.007_203_560_3).abs() < 1e-8);
    assert_eq!(begin_zoning_message(&entry), hex(VEC_BEGIN_ZONING));
}

#[test]
fn zone_data_encoder_reproduces_both_captured_responses() {
    assert_eq!(HOUSING_ZONE_DATA, 45);
    assert_eq!(DEFAULT_HEAD_SIZE, CAPTURE_HEAD_SIZE);
    let first = ZoneDataFields::from_house(&captured_house(), false, true);
    assert_eq!(first.instance_guid, HOUSE_GUID);
    assert_eq!(first.owner_guid, OWNER_GUID);
    assert_eq!(first.house_name, HOUSE_NAME);
    assert_eq!(first.owner_name, OWNER_NAME);
    assert_eq!(zone_data_message(&first), hex(VEC_ZONE_DATA_FIRST));

    let later = ZoneDataFields::from_house(&captured_house(), false, false);
    assert_eq!(zone_data_message(&later), hex(VEC_ZONE_DATA_LATER));
}

#[test]
fn owner_and_guest_enter_the_same_instance() {
    let mut router = router_with_captured_house();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    let owner = router.enter(&request, OWNER_GUID).unwrap();
    assert_eq!(owner.role, EntranceRole::Owner);
    let guest = router.enter(&request, GUEST_GUID).unwrap();
    assert_eq!(guest.role, EntranceRole::Guest);
    assert_eq!(guest.instance_id, owner.instance_id);
    assert_eq!(guest.zone_data.instance_guid, owner.zone_data.instance_guid);
    assert_eq!(guest.zone_data.owner_guid, OWNER_GUID);
}

#[test]
fn a_guest_revisiting_alone_reaches_the_owners_instance() {
    let mut router = router_with_captured_house();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    let guest_first = router.enter(&request, GUEST_GUID).unwrap();
    assert!(router.leave(guest_first.instance_id, GUEST_GUID).unwrap());
    let owner = router.enter(&request, OWNER_GUID).unwrap();
    // The empty runtime instance was evicted, so the owner got a fresh runtime id while the house
    // identity itself did not move.
    assert_ne!(owner.instance_id, guest_first.instance_id);
    let guest_again = router.enter(&request, GUEST_GUID).unwrap();
    assert_eq!(guest_again.role, EntranceRole::Guest);
    assert_eq!(guest_again.instance_id, owner.instance_id);
    assert_eq!(guest_again.zone_data.instance_guid, HOUSE_GUID);
}

#[test]
fn every_house_keeps_its_own_instance() {
    let mut router = router_with_captured_house();
    let mut second = captured_house();
    second.house_guid = 0x0abc_0000_0000_0001;
    second.owner_guid = GUEST_GUID;
    second.zone_definition_id = 2;
    second.zone_name = "hsg_emptylot_briarwood_01".to_string();
    second.house_name = "Guest Cottage".to_string();
    second.owner_name = "Guest".to_string();
    second.tail_word = 0;
    router.register_house(second.clone()).unwrap();

    let owner_request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    let owner = router.enter(&owner_request, OWNER_GUID).unwrap();
    let second_request = piece_022::EnterRequest {
        house_guid: second.house_guid,
        unknown: 0,
        unknown_bool: false,
    };
    let guest_own = router.enter(&second_request, GUEST_GUID).unwrap();
    assert_eq!(guest_own.role, EntranceRole::Owner);
    assert_ne!(guest_own.instance_id, owner.instance_id);

    // The guest visiting the first house still lands in the first instance, not their own.
    let visiting = router.enter(&owner_request, GUEST_GUID).unwrap();
    assert_eq!(visiting.role, EntranceRole::Guest);
    assert_eq!(visiting.instance_id, owner.instance_id);
    assert_ne!(visiting.instance_id, guest_own.instance_id);
}

#[test]
fn unknown_house_is_denied_without_inventing_a_reply() {
    let mut router = HousingEntranceRouter::new();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    assert_eq!(
        router.enter(&request, OWNER_GUID),
        Err(EntranceError::UnknownHouse(HOUSE_GUID))
    );
}

#[test]
fn leaving_evicts_the_runtime_instance_but_keeps_the_house() {
    let mut router = router_with_captured_house();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    let first = router.enter(&request, OWNER_GUID).unwrap();
    assert!(!router.leave(first.instance_id, GUEST_GUID).unwrap());
    assert!(router.leave(first.instance_id, OWNER_GUID).unwrap());
    let second = router.enter(&request, OWNER_GUID).unwrap();
    assert_ne!(second.instance_id, first.instance_id);
    assert_eq!(second.zone_data.instance_guid, HOUSE_GUID);
    assert_eq!(second.zone_data.house_name, HOUSE_NAME);
}

#[test]
fn restart_restores_house_routing_for_owner_and_guest() {
    let dir = std::env::temp_dir().join(format!(
        "piece022-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("housing.json");

    let mut router = router_with_captured_house();
    let request = decode_enter_request(&hex(VEC_ENTER_REQUEST)).unwrap();
    let before = router.enter(&request, OWNER_GUID).unwrap();
    router.save_to_path(&path).unwrap();

    let mut restored = HousingEntranceRouter::load_from_path(&path).unwrap();
    let owner = restored.enter(&request, OWNER_GUID).unwrap();
    let guest = restored.enter(&request, GUEST_GUID).unwrap();
    assert_eq!(owner.role, EntranceRole::Owner);
    assert_eq!(guest.role, EntranceRole::Guest);
    assert_eq!(guest.instance_id, owner.instance_id);
    assert_eq!(owner.zone_data.house_name, HOUSE_NAME);
    assert_eq!(owner.zone_data.owner_guid, OWNER_GUID);
    assert_eq!(owner.zone_data.tail_word, before.zone_data.tail_word);
    assert_eq!(
        zone_data_message(&owner.zone_data),
        hex(VEC_ZONE_DATA_FIRST)
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn malformed_requests_and_duplicate_houses_are_rejected() {
    assert_eq!(
        decode_enter_request(&hex("7f001300da102db86931f507000000000001")),
        Err(EntranceError::TrailingData)
    );
    assert_eq!(
        decode_enter_request(&hex("7f001300da102db86931f5070000")),
        Err(EntranceError::ShortMessage)
    );
    assert_eq!(
        decode_enter_request(&hex("2d001300da102db86931f5070000000000")),
        Err(EntranceError::UnexpectedOpcode)
    );
    assert_eq!(
        decode_enter_request(&hex("7f00130000000000000000000000000000")),
        Err(EntranceError::ZeroHouseGuid)
    );
    decode_request_grant(&hex("7f001701")).unwrap_err();

    let mut router = router_with_captured_house();
    assert_eq!(
        router.register_house(captured_house()),
        Err(EntranceError::DuplicateHouse(HOUSE_GUID))
    );
    let mut same_lot_other_house = captured_house();
    same_lot_other_house.house_guid = 0x0abc_0000_0000_0002;
    assert_eq!(
        router.register_house(same_lot_other_house),
        Err(EntranceError::DuplicateLotOwner {
            zone_definition_id: 1,
            owner_guid: OWNER_GUID,
        })
    );
}

#[test]
fn fixture_updates_bind_to_the_entered_house_guid() {
    // The captured fixture update carries the fixture guid first and the house guid second; the
    // house guid is the same instance identity the enter request named.
    assert_eq!(HOUSING_FIXTURE_UPDATE, 40);
    let prefix = hex(VEC_FIXTURE_UPDATE_PREFIX);
    assert_eq!(prefix[0], 0x7f);
    assert_eq!(prefix[1], 0x00);
    assert_eq!(&prefix[4..12], &FIXTURE_GUID.to_le_bytes());
    assert_eq!(&prefix[12..20], &HOUSE_GUID.to_le_bytes());
    assert_ne!(FIXTURE_GUID, HOUSE_GUID);
}
