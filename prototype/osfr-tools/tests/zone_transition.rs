//! World scene transition (opcode 31) codec acceptance.
//!
//! This target tests the registered production module. Raw captured game
//! bytes are not embedded here: the captured round-trip is an ignored,
//! env-gated test that reads the preserved lab file only when
//! `OSFR_ZONE_OP31_CAPTURE` is set.
use osfr_tools::zone_transition::{
    MAX_STRING_BYTES, OPCODE, OUTER_FAMILY, ZoneTransition, decode, encode,
};
use std::fs;

/// Field values documented for the captured `sg_kt_track` frame, rebuilt here
/// from its field map (not from embedded raw bytes).
fn captured_fields() -> ZoneTransition {
    ZoneTransition {
        envelope_flag: 1,
        scene: "sg_kt_track".to_string(),
        type_id: 2,
        position: [719.60999, 72.0, 421.5, 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
        sky: "sky.xml".to_string(),
        tutorial: false,
        unknown: 1,
        id: 0,
        update_radius: 0.0,
        geometry_id: 101,
        override_update_radius: true,
        wait_for_zone_ready: false,
    }
}

fn i32_at(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn f32_at(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn valid_bytes() -> Vec<u8> {
    encode(&captured_fields()).unwrap()
}

#[test]
fn encode_lays_out_the_native_reader_field_order() {
    let bytes = valid_bytes();
    assert_eq!(bytes.len(), 87, "captured frame size");
    assert_eq!(
        u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
        OUTER_FAMILY
    );
    assert_eq!(bytes[2], 1, "envelope flag preserved");
    assert_eq!(
        u32::from_le_bytes(bytes[3..7].try_into().unwrap()) as usize,
        bytes.len() - 7,
        "declared length covers opcode and body"
    );
    assert_eq!(u16::from_le_bytes(bytes[7..9].try_into().unwrap()), OPCODE);
    assert_eq!(i32_at(&bytes, 9), 11, "scene byte length");
    assert_eq!(&bytes[13..24], b"sg_kt_track");
    assert_eq!(i32_at(&bytes, 24), 2, "type id");
    for (index, expected) in [719.60999_f32, 72.0, 421.5, 1.0].iter().enumerate() {
        assert_eq!(
            f32_at(&bytes, 28 + index * 4),
            *expected,
            "position {index}"
        );
    }
    for (index, expected) in [0.0_f32, 0.0, 1.0, 0.0].iter().enumerate() {
        assert_eq!(
            f32_at(&bytes, 44 + index * 4),
            *expected,
            "rotation {index}"
        );
    }
    assert_eq!(i32_at(&bytes, 60), 7, "sky byte length");
    assert_eq!(&bytes[64..71], b"sky.xml");
    assert_eq!(bytes[71], 0, "tutorial");
    assert_eq!(bytes[72], 1, "unknown byte");
    assert_eq!(i32_at(&bytes, 73), 0, "id");
    assert_eq!(f32_at(&bytes, 77), 0.0, "update radius");
    assert_eq!(i32_at(&bytes, 81), 101, "geometry id");
    assert_eq!(bytes[85], 1, "override update radius");
    assert_eq!(bytes[86], 0, "wait for zone ready");
}

#[test]
fn decode_round_trips_the_documented_fields() {
    let bytes = valid_bytes();
    assert_eq!(decode(&bytes).unwrap(), captured_fields());
}

#[test]
fn utf8_names_use_byte_length_and_round_trip() {
    let mut packet = captured_fields();
    packet.scene = "sg_kt_трack".to_string();
    packet.sky = "небо.xml".to_string();
    let bytes = encode(&packet).unwrap();
    let scene_bytes = packet.scene.as_bytes().len();
    assert_eq!(i32_at(&bytes, 9) as usize, scene_bytes);
    assert_eq!(decode(&bytes).unwrap(), packet);
}

#[test]
fn decode_refuses_wrong_envelope_opcode_and_length() {
    let mut wrong_family = valid_bytes();
    wrong_family[0] = 6;
    assert!(decode(&wrong_family).is_err());

    let mut wrong_opcode = valid_bytes();
    wrong_opcode[7] = 32;
    assert!(decode(&wrong_opcode).is_err());

    let mut short_length = valid_bytes();
    short_length[3] -= 1;
    assert!(decode(&short_length).is_err());

    let mut long_length = valid_bytes();
    long_length[3] += 1;
    assert!(decode(&long_length).is_err());

    assert!(decode(&[]).is_err());
    assert!(decode(&valid_bytes()[..8]).is_err());
}

#[test]
fn bounded_string_reads_refuse_oversize_overrun_and_negative_lengths() {
    let mut oversize = valid_bytes();
    oversize[9..13].copy_from_slice(&(MAX_STRING_BYTES as i32 + 1).to_le_bytes());
    assert!(decode(&oversize).is_err());

    let mut overrun = valid_bytes();
    overrun[9..13].copy_from_slice(&(i32::MAX).to_le_bytes());
    assert!(decode(&overrun).is_err());

    let mut negative = valid_bytes();
    negative[9..13].copy_from_slice(&(-1_i32).to_le_bytes());
    assert!(decode(&negative).is_err());

    let truncated = &valid_bytes()[..12];
    assert!(decode(truncated).is_err());

    let mut bad_utf8 = valid_bytes();
    bad_utf8[13] = 0xff;
    assert!(decode(&bad_utf8).is_err());
}

#[test]
fn non_finite_floats_are_refused_on_both_paths() {
    for position in 0..4 {
        let mut packet = captured_fields();
        packet.position[position] = f32::NAN;
        assert!(encode(&packet).is_err(), "encode NaN position {position}");
        let mut bytes = valid_bytes();
        bytes[28 + position * 4..32 + position * 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(decode(&bytes).is_err(), "decode NaN position {position}");
    }
    for rotation in 0..4 {
        let mut packet = captured_fields();
        packet.rotation[rotation] = f32::INFINITY;
        assert!(encode(&packet).is_err(), "encode inf rotation {rotation}");
        let mut bytes = valid_bytes();
        bytes[44 + rotation * 4..48 + rotation * 4].copy_from_slice(&f32::INFINITY.to_le_bytes());
        assert!(decode(&bytes).is_err(), "decode inf rotation {rotation}");
    }
    let mut packet = captured_fields();
    packet.update_radius = f32::NEG_INFINITY;
    assert!(encode(&packet).is_err(), "encode inf radius");
    let mut bytes = valid_bytes();
    bytes[77..81].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(decode(&bytes).is_err(), "decode NaN radius");
}

#[test]
fn booleans_decode_as_nonzero_and_encode_canonically() {
    let mut bytes = valid_bytes();
    bytes[71] = 0x02;
    bytes[85] = 0xff;
    bytes[86] = 0x80;
    let decoded = decode(&bytes).unwrap();
    assert!(decoded.tutorial);
    assert!(decoded.override_update_radius);
    assert!(decoded.wait_for_zone_ready);
    let reencoded = encode(&decoded).unwrap();
    assert_eq!(reencoded[71], 1);
    assert_eq!(reencoded[85], 1);
    assert_eq!(reencoded[86], 1);
    let mut zeroed = decoded;
    zeroed.tutorial = false;
    zeroed.override_update_radius = false;
    zeroed.wait_for_zone_ready = false;
    let reencoded = encode(&zeroed).unwrap();
    assert_eq!(reencoded[71], 0);
    assert_eq!(reencoded[85], 0);
    assert_eq!(reencoded[86], 0);
}

#[test]
fn short_tails_trailing_bytes_and_oversize_strings_are_refused() {
    let bytes = valid_bytes();
    assert!(decode(&bytes[..bytes.len() - 1]).is_err(), "short tail");
    let mut trailing = bytes.clone();
    trailing.push(0);
    trailing[3] += 1;
    assert!(decode(&trailing).is_err(), "trailing byte");

    let mut packet = captured_fields();
    packet.scene = "x".repeat(MAX_STRING_BYTES + 1);
    assert!(encode(&packet).is_err(), "oversize scene");
    let mut packet = captured_fields();
    packet.sky = "x".repeat(MAX_STRING_BYTES + 1);
    assert!(encode(&packet).is_err(), "oversize sky");
}

#[test]
fn defaults_do_not_invent_flags_or_geometry() {
    let packet = ZoneTransition::default();
    assert_eq!(packet.envelope_flag, 0);
    assert_eq!(packet.type_id, 0);
    assert_eq!(packet.geometry_id, 0);
    assert_eq!(packet.id, 0);
    assert_eq!(packet.unknown, 0);
    assert!(!packet.tutorial && !packet.override_update_radius && !packet.wait_for_zone_ready);
    assert!(packet.scene.is_empty() && packet.sky.is_empty());
    let bytes = encode(&packet).unwrap();
    assert_eq!(bytes[2], 0);
    assert_eq!(decode(&bytes).unwrap(), packet);
}

/// Raw captured game bytes stay in the private lab: this gate only runs when
/// the caller points it at the preserved file.
#[test]
#[ignore = "private lab gate: set OSFR_ZONE_OP31_CAPTURE to the preserved op31 frame"]
fn captured_op31_frame_round_trips_byte_exact() {
    let path = std::env::var("OSFR_ZONE_OP31_CAPTURE").expect("OSFR_ZONE_OP31_CAPTURE");
    let bytes = fs::read(&path).expect("captured frame");
    assert_eq!(bytes.len(), 87);
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "bace4cbaef5d50eb3601543847e77e1b4058ce765405f91556dd386f8e32c373"
    );
    let decoded = decode(&bytes).expect("captured frame decodes");
    assert_eq!(decoded.scene, "sg_kt_track");
    assert_eq!(decoded.position[0], 719.60999);
    assert_eq!(decoded.geometry_id, 101);
    assert_eq!(encode(&decoded).unwrap(), bytes, "byte-exact re-encode");
}
