use miniz_oxide::deflate::compress_to_vec_zlib;
use osfr_tools::soe::{Codec, admit_direct, seeded_crc32};

const SEED: u32 = 0x12345678;
fn pose() -> Vec<u8> {
    let mut app = vec![5, 0, 1];
    app.extend(89u32.to_le_bytes());
    app.extend([33, 0, 6, 0]);
    app.resize(96, 0);
    app
}
fn wire(app: &[u8], compressed: bool, crc: usize) -> Vec<u8> {
    let mut bytes = vec![app[0], u8::from(compressed)];
    if compressed {
        bytes.extend(compress_to_vec_zlib(&app[1..], 6));
    } else {
        bytes.extend(&app[1..]);
    }
    let hash = seeded_crc32(SEED, &bytes).to_be_bytes();
    bytes.extend(&hash[4 - crc..]);
    bytes
}
#[test]
fn direct_pose_roundtrips_both_encodings_and_negotiated_crc_widths() {
    for width in 0..=4 {
        let codec = Codec::new(SEED, width, true, 512).unwrap();
        for compressed in [false, true] {
            assert_eq!(
                codec
                    .decode_direct_datagram(&wire(&pose(), compressed, width as usize))
                    .unwrap(),
                pose()
            );
        }
    }
}
#[test]
fn direct_pose_rejects_forged_use_headers_lengths_and_crc() {
    let codec = Codec::new(SEED, 3, true, 512).unwrap();
    for index in [0, 1, 2, 3, 7, 8, 9, 10] {
        let mut app = pose();
        app[index] ^= 0x80;
        assert!(
            codec.decode_direct_datagram(&wire(&app, false, 3)).is_err(),
            "index {index}"
        );
    }
    let mut app = pose();
    app[9] = 19; // No direct item-use dispatch.
    assert!(codec.decode_direct_datagram(&wire(&app, true, 3)).is_err());
    for len in [95, 97, 4096] {
        let mut app = pose();
        app.resize(len, 0);
        assert!(codec.decode_direct_datagram(&wire(&app, true, 3)).is_err());
    }
    // L965: a pose whose length field agrees with it is still refused at any length but 89.
    let mut long = pose();
    long[3..7].copy_from_slice(&90u32.to_le_bytes());
    long.push(0);
    let refused = codec.decode_direct_datagram(&wire(&long, true, 3));
    assert_eq!(
        refused.unwrap_err().to_string(),
        "unadmitted direct application 33/6 (90 bytes)"
    );
    // Every byte of the length field is checked.
    for index in 3..7 {
        let mut app = pose();
        app[index] ^= 1;
        assert!(
            codec.decode_direct_datagram(&wire(&app, true, 3)).is_err(),
            "index {index}"
        );
    }
    let mut bad = wire(&pose(), false, 3);
    bad[20] ^= 1;
    assert!(codec.decode_direct_datagram(&bad).is_err());
    for len in 0..bad.len() {
        assert!(codec.decode_direct_datagram(&bad[..len]).is_err());
    }
}
#[test]
fn direct_pose_requires_single_complete_zlib_stream_and_supported_flag() {
    let codec = Codec::new(SEED, 3, true, 512).unwrap();
    for flag in [2, 255] {
        let mut bytes = wire(&pose(), false, 0);
        bytes[1] = flag;
        let crc = seeded_crc32(SEED, &bytes).to_be_bytes();
        bytes.extend(&crc[1..]);
        assert!(codec.decode_direct_datagram(&bytes).is_err());
    }
    for extra in [vec![0], compress_to_vec_zlib(b"extra", 6)] {
        let mut bytes = wire(&pose(), true, 0);
        bytes.extend(extra);
        let crc = seeded_crc32(SEED, &bytes).to_be_bytes();
        bytes.extend(&crc[1..]);
        assert!(codec.decode_direct_datagram(&bytes).is_err());
    }
    let tiny = Codec::new(SEED, 3, true, 94).unwrap();
    assert!(
        tiny.decode_direct_datagram(&wire(&pose(), true, 3))
            .is_err()
    );
}
/// L965: Pirate's Plunder's UpdateShipPhysics (94/16, 117 bytes) comes direct too; any other
/// direct packet, or the ship's at another length, is refused and named.
#[test]
fn direct_ship_physics_is_admitted_at_its_exact_length_only() {
    let codec = Codec::new(SEED, 3, true, 512).unwrap();
    let ship = |len: u32| {
        let mut app = vec![5, 0, 0];
        app.extend(len.to_le_bytes());
        app.extend([94, 0, 16, 0]);
        app.resize(7 + len as usize, 0x11);
        app
    };
    for compressed in [false, true] {
        let app = ship(117);
        assert_eq!(
            codec
                .decode_direct_datagram(&wire(&app, compressed, 3))
                .unwrap(),
            app
        );
    }
    let wrong = codec.decode_direct_datagram(&wire(&ship(116), true, 3));
    assert_eq!(
        wrong.unwrap_err().to_string(),
        "unadmitted direct application 94/16 (116 bytes)"
    );
    let mut fire = ship(117);
    fire[9] = 20; // FireAtLocation is not admitted direct.
    assert!(codec.decode_direct_datagram(&wire(&fire, true, 3)).is_err());
}

/// A Multi sub-packet's tunnel passes the same shape list as a lone direct packet.
#[test]
fn tunnels_out_of_multi_frames_keep_to_the_admitted_shapes() {
    let tunnel = |inner: &[u8]| {
        let mut out = vec![0, 1];
        out.extend(u32::try_from(inner.len()).unwrap().to_le_bytes());
        out.extend(inner);
        out
    };
    let mut fire = vec![94, 0, 20, 0];
    fire.resize(24, 0);
    let message = admit_direct(tunnel(&fire)).unwrap();
    assert_eq!(message[0], 5);
    assert_eq!(message[7..], fire[..]);
    let mut report = vec![94, 0, 16, 0];
    report.resize(117, 0);
    assert!(admit_direct(tunnel(&report)).is_ok());
    let mut other = vec![94, 0, 21, 0];
    other.resize(24, 0);
    assert!(admit_direct(tunnel(&other)).is_err(), "a shape off the list");
    assert!(admit_direct(tunnel(&fire[..23])).is_err(), "the wrong length");
    assert!(admit_direct(vec![0; 300]).is_err(), "over the bound");
}

/// F3 (rc-0136 m-derby): the derby car's 34/5 pose comes direct every frame (writer `009f0ce0`,
/// sender `009f1a70`), 132 bytes. Refused, the server never had the player's car: the AI could not
/// hunt it and every hit the player reported failed the fresh-pose check (0 damage on the tally).
#[test]
fn the_derby_car_pose_is_admitted_direct_at_its_one_size() {
    let tunnel = |inner: &[u8]| {
        let mut out = vec![0, 1];
        out.extend(u32::try_from(inner.len()).unwrap().to_le_bytes());
        out.extend(inner);
        out
    };
    let mut pose = vec![34, 0, 5, 0];
    pose.resize(132, 0);
    let message = admit_direct(tunnel(&pose)).unwrap();
    assert_eq!(message[7..], pose[..], "the pose reaches the application intact");
    assert!(admit_direct(tunnel(&pose[..131])).is_err(), "only the client's own size");
    let mut other = vec![34, 0, 16, 0];
    other.resize(132, 0);
    assert!(admit_direct(tunnel(&other)).is_err(), "a hit report stays reliable");
}
