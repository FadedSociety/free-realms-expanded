use miniz_oxide::deflate::compress_to_vec_zlib;
use osfr_tools::soe::{Codec, seeded_crc32};

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
                    .decode_kart_pose_datagram(&wire(&pose(), compressed, width as usize))
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
            codec
                .decode_kart_pose_datagram(&wire(&app, false, 3))
                .is_err(),
            "index {index}"
        );
    }
    let mut app = pose();
    app[9] = 19; // No direct item-use dispatch.
    assert!(
        codec
            .decode_kart_pose_datagram(&wire(&app, true, 3))
            .is_err()
    );
    for len in [95, 97, 4096] {
        let mut app = pose();
        app.resize(len, 0);
        assert!(
            codec
                .decode_kart_pose_datagram(&wire(&app, true, 3))
                .is_err()
        );
    }
    let mut bad = wire(&pose(), false, 3);
    bad[20] ^= 1;
    assert!(codec.decode_kart_pose_datagram(&bad).is_err());
    for len in 0..bad.len() {
        assert!(codec.decode_kart_pose_datagram(&bad[..len]).is_err());
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
        assert!(codec.decode_kart_pose_datagram(&bytes).is_err());
    }
    for extra in [vec![0], compress_to_vec_zlib(b"extra", 6)] {
        let mut bytes = wire(&pose(), true, 0);
        bytes.extend(extra);
        let crc = seeded_crc32(SEED, &bytes).to_be_bytes();
        bytes.extend(&crc[1..]);
        assert!(codec.decode_kart_pose_datagram(&bytes).is_err());
    }
    let tiny = Codec::new(SEED, 3, true, 94).unwrap();
    assert!(
        tiny.decode_kart_pose_datagram(&wire(&pose(), true, 3))
            .is_err()
    );
}
