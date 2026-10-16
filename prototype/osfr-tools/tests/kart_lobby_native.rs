//! Kart native lobby definition codec (opcode 102 sub 2) — owned vectors, MMO piece for the kart
//! lobby puzzle.
//!
//! ORIGINAL evidence used by these vectors (all outside this test):
//! * `notes/Research/Kart Lobby Definition Framing.md` — the native handler `FUN_0091e060` gates on
//!   `sub == 2` (`FUN_008d67a0`), `FUN_008f74c0` re-reads the header and `FUN_008bfc00` reads the
//!   `i32` body length with a `cursor + length <= end` check before `FUN_00919b60` parses five
//!   tables in a fixed order.
//! * `lab/reports/kart-lobby-independent-worker-review-20261009T155631Z/review.json` — independent
//!   re-derivation: raw capture 2393 bytes, outer 5, flag 1, inner length 2386, body 2378,
//!   counts `[6, 30, 43, 37, 17]`, body sha256
//!   `51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c`, and the explicit
//!   "trailing bytes after the five tables" entry on its unverified list.
//! * the private capture `lab/reports/original-minigame-packets/
//!   racing-23867-flow0-frame44523-op102-sub2.bin` (raw sha256
//!   `4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932`), read only by the ignored
//!   gate below through `OSFR_KART_LOBBY_CAPTURE`. No captured bytes are committed here.
//!
//! Field *semantics* that the preserved sources do not recover are preserved as `unknown*`
//! components instead of being named or interpreted: options/values triples, the display row's
//! second word, the games row's fourth word and the settings word order.
//!
//! This file exercises the registered production library module. Integration step: route the
//! gateway's 102/2 response through `kart_lobby_native` instead of
//! `kart_racing::LobbyGameDefinition` (the 140-byte community subset).

use osfr_tools::kart_lobby_native::{
    LOBBY_HEADER_LEN, LOBBY_INNER_OPCODE, LOBBY_INNER_SUB, LobbyCodecError, LobbyDisplayRow,
    LobbyGamesRow, LobbyNativeBody, LobbyOptionsRow, LobbySettingsRow, LobbyTable, LobbyValuesRow,
    TrailingBytes, TrailingPosition, decode_inner, decode_tunnel, encode_body, encode_inner,
    encode_tunnel,
};

fn empty_body() -> LobbyNativeBody {
    LobbyNativeBody::default()
}

fn minimal_kart_body() -> LobbyNativeBody {
    LobbyNativeBody {
        games: vec![LobbyGamesRow {
            key: 1,
            id: 1,
            game_type: 1,
            unknown: 37,
            name_id: 3052,
        }],
        ..LobbyNativeBody::default()
    }
}

/// Builds an 102/2 frame from parts so a vector can declare a length that does not match the bytes.
fn raw_frame(declared_body_length: i32, payload: &[u8], after_body: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend(LOBBY_INNER_OPCODE.to_le_bytes());
    frame.extend(LOBBY_INNER_SUB.to_le_bytes());
    frame.extend(declared_body_length.to_le_bytes());
    frame.extend(payload);
    frame.extend(after_body);
    frame
}

const MINIMAL_BODY_HEX: &str = concat!(
    "01000000", "01000000", "01000000", "01000000", "25000000", "ec0b0000", "00000000", "00000000",
    "00000000", "00000000",
);

#[test]
fn minimal_kart_body_matches_the_recovered_layout() {
    let body = minimal_kart_body();
    let bytes = encode_body(&body).expect("encodes");
    assert_eq!(hex(&bytes), MINIMAL_BODY_HEX);
    assert_eq!(bytes.len(), 40, "games table only: 4 + 20 + 4*4");

    let inner = encode_inner(&body).expect("encodes");
    assert_eq!(inner.len(), 48);
    assert_eq!(&inner[..4], &[102, 0, 2, 0]);
    assert_eq!(&inner[4..8], &40i32.to_le_bytes());
    assert_eq!(&inner[LOBBY_HEADER_LEN..], &bytes[..]);

    let tunnel = encode_tunnel(5, 1, &body).expect("encodes");
    assert_eq!(tunnel.len(), 55);
    assert_eq!(&tunnel[..2], &5u16.to_le_bytes());
    assert_eq!(tunnel[2], 1);
    assert_eq!(&tunnel[3..7], &(inner.len() as u32).to_le_bytes());
}

#[test]
fn one_row_per_table_roundtrips_at_the_expected_lengths() {
    let body = LobbyNativeBody {
        games: minimal_kart_body().games,
        options: vec![LobbyOptionsRow {
            key: 7,
            unknown: [1, 2, 3],
        }],
        values: vec![LobbyValuesRow {
            key: 8,
            unknown: [4, 5, 6],
        }],
        display: vec![LobbyDisplayRow {
            key: 9,
            unknown: 1,
            locale_id: 3032,
            text: b"2".to_vec(),
        }],
        settings: vec![LobbySettingsRow {
            key: 10,
            fields: [1, 2, 3, 4, 5],
        }],
    };
    let bytes = encode_body(&body).expect("encodes");
    assert_eq!(bytes.len(), 113);
    assert_eq!(encode_inner(&body).expect("encodes").len(), 121);
    assert_eq!(encode_tunnel(5, 1, &body).expect("encodes").len(), 128);

    let decoded = decode_inner(
        &encode_inner(&body).expect("encodes"),
        TrailingBytes::Strict,
    )
    .expect("roundtrips");
    assert_eq!(decoded.body, body);
    assert_eq!(decoded.trailing_bytes, 0);
}

#[test]
fn each_table_is_count_prefixed_and_ordered() {
    // Two games rows put the options count at 4 + 2*20; each empty table then adds only its count.
    let body = LobbyNativeBody {
        games: vec![minimal_kart_body().games[0], minimal_kart_body().games[0]],
        ..LobbyNativeBody::default()
    };
    let bytes = encode_body(&body).expect("encodes");
    assert_eq!(bytes.len(), 60);
    assert_eq!(&bytes[0..4], &2i32.to_le_bytes(), "games count first");
    assert_eq!(
        &bytes[44..48],
        &0i32.to_le_bytes(),
        "options count after the games table"
    );
    assert_eq!(&bytes[48..52], &0i32.to_le_bytes(), "values count");
    assert_eq!(&bytes[52..56], &0i32.to_le_bytes(), "display count");
    assert_eq!(&bytes[56..60], &0i32.to_le_bytes(), "settings count");

    let empty = encode_body(&empty_body()).expect("encodes");
    assert_eq!(empty, vec![0u8; 20], "five empty counts");
}

#[test]
fn unknown_fields_are_preserved_verbatim() {
    let body = LobbyNativeBody {
        games: vec![LobbyGamesRow {
            key: 0xffff_fffe,
            id: -1,
            game_type: 7,
            unknown: -37,
            name_id: 417078,
        }],
        options: vec![LobbyOptionsRow {
            key: 2,
            unknown: [-1, i32::MIN, 4],
        }],
        values: vec![LobbyValuesRow {
            key: 3,
            unknown: [i32::MAX, 0, -2],
        }],
        display: vec![LobbyDisplayRow {
            key: 4,
            unknown: -1,
            locale_id: 3032,
            text: vec![0x00, 0xff, b'2'],
        }],
        settings: vec![LobbySettingsRow {
            key: 5,
            fields: [-1, -2, -3, -4, -5],
        }],
    };
    let decoded = decode_inner(
        &encode_inner(&body).expect("encodes"),
        TrailingBytes::Strict,
    )
    .expect("roundtrips");
    assert_eq!(decoded.body, body);
}

#[test]
fn decode_refuses_a_wrong_opcode_or_subtype() {
    let mut frame = encode_inner(&minimal_kart_body()).expect("encodes");
    frame[0..2].copy_from_slice(&101u16.to_le_bytes());
    assert_eq!(
        decode_inner(&frame, TrailingBytes::Strict),
        Err(LobbyCodecError::UnsupportedInnerOpcode(101))
    );

    let mut frame = encode_inner(&minimal_kart_body()).expect("encodes");
    frame[2..4].copy_from_slice(&3u16.to_le_bytes());
    assert_eq!(
        decode_inner(&frame, TrailingBytes::Strict),
        Err(LobbyCodecError::UnsupportedInnerSub(3))
    );
}

#[test]
fn decode_refuses_a_negative_or_short_body_length() {
    let payload = encode_body(&minimal_kart_body()).expect("encodes");

    let negative = raw_frame(-1, &payload, &[]);
    assert_eq!(
        decode_inner(&negative, TrailingBytes::Strict),
        Err(LobbyCodecError::NegativeBodyLength(-1))
    );

    let truncated = raw_frame(100, &payload, &[]);
    assert_eq!(
        decode_inner(&truncated, TrailingBytes::Strict),
        Err(LobbyCodecError::BodyLengthMismatch {
            declared: 100,
            available: 40
        })
    );

    let short = vec![102u8, 0, 2, 0, 1, 0, 0];
    assert_eq!(
        decode_inner(&short, TrailingBytes::Strict),
        Err(LobbyCodecError::TooShort {
            needed: LOBBY_HEADER_LEN,
            available: 7
        })
    );
}

#[test]
fn trailing_bytes_policy_is_explicit_at_frame_and_body_level() {
    let payload = encode_body(&minimal_kart_body()).expect("encodes");

    // Three bytes after the declared body: native `FUN_008bfc00` only bounds the body.
    let frame_trailing = raw_frame(40, &payload, &[0xaa, 0xbb, 0xcc]);
    assert_eq!(
        decode_inner(&frame_trailing, TrailingBytes::Strict),
        Err(LobbyCodecError::TrailingBytes {
            position: TrailingPosition::Frame,
            count: 3
        })
    );
    let tolerant = decode_inner(&frame_trailing, TrailingBytes::Tolerant).expect("tolerant");
    assert_eq!(tolerant.trailing_bytes, 3);
    assert_eq!(tolerant.body, minimal_kart_body());
    assert_eq!(
        encode_inner(&tolerant.body).expect("re-encodes"),
        raw_frame(40, &payload, &[]),
        "a tolerant decode re-encodes the declared frame, not the ignored tail"
    );

    // Three bytes inside the declared body after the five tables.
    let mut inside = payload.clone();
    inside.extend([0xaa, 0xbb, 0xcc]);
    let body_trailing = raw_frame(43, &inside, &[]);
    assert_eq!(
        decode_inner(&body_trailing, TrailingBytes::Strict),
        Err(LobbyCodecError::TrailingBytes {
            position: TrailingPosition::Body,
            count: 3
        })
    );
    assert_eq!(
        decode_inner(&body_trailing, TrailingBytes::Tolerant)
            .expect("tolerant")
            .trailing_bytes,
        3
    );
}

#[test]
fn decode_refuses_negative_and_oversized_counts() {
    let mut body = encode_body(&empty_body()).expect("encodes");
    body[0..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(
        decode_inner(&raw_frame(20, &body, &[]), TrailingBytes::Strict),
        Err(LobbyCodecError::NegativeCount {
            table: LobbyTable::Games,
            count: -1
        })
    );

    let mut body = encode_body(&empty_body()).expect("encodes");
    body[0..4].copy_from_slice(&1_000i32.to_le_bytes());
    assert_eq!(
        decode_inner(&raw_frame(20, &body, &[]), TrailingBytes::Strict),
        Err(LobbyCodecError::CountExceedsRemaining {
            table: LobbyTable::Games,
            count: 1_000,
            available: 16,
        })
    );
}

#[test]
fn decode_refuses_negative_and_oversized_display_text() {
    // Three empty tables put the display count at offset 12; the settings count follows the rows.
    let mut body = Vec::new();
    body.extend(0i32.to_le_bytes()); // games
    body.extend(0i32.to_le_bytes()); // options
    body.extend(0i32.to_le_bytes()); // values
    body.extend(1i32.to_le_bytes()); // display
    body.extend(99u32.to_le_bytes()); // key
    body.extend(1i32.to_le_bytes()); // unknown
    body.extend(3032i32.to_le_bytes()); // locale id
    body.extend((-5i32).to_le_bytes()); // negative length
    body.extend(0i32.to_le_bytes()); // settings
    assert_eq!(
        decode_inner(
            &raw_frame(body.len() as i32, &body, &[]),
            TrailingBytes::Strict
        ),
        Err(LobbyCodecError::NegativeTextLength { row: 0, length: -5 })
    );

    let mut body = Vec::new();
    body.extend(0i32.to_le_bytes());
    body.extend(0i32.to_le_bytes());
    body.extend(0i32.to_le_bytes());
    body.extend(1i32.to_le_bytes());
    body.extend(99u32.to_le_bytes());
    body.extend(1i32.to_le_bytes());
    body.extend(3032i32.to_le_bytes());
    body.extend(0x2000_0000i32.to_le_bytes()); // 512 MiB, far beyond the buffer
    body.extend(0i32.to_le_bytes()); // settings
    assert_eq!(
        decode_inner(
            &raw_frame(body.len() as i32, &body, &[]),
            TrailingBytes::Strict
        ),
        Err(LobbyCodecError::TextExceedsRemaining {
            row: 0,
            length: 0x2000_0000,
            available: 4
        })
    );
}

#[test]
fn tunnel_framing_roundtrips_and_refuses_bad_headers() {
    let body = minimal_kart_body();
    let tunnel = encode_tunnel(6, 1, &body).expect("encodes");
    let decoded = decode_tunnel(&tunnel, TrailingBytes::Strict).expect("roundtrips");
    assert_eq!(decoded.body, body);
    assert_eq!(decoded.trailing_bytes, 0);

    let mut wrong_outer = tunnel.clone();
    wrong_outer[0..2].copy_from_slice(&7u16.to_le_bytes());
    assert_eq!(
        decode_tunnel(&wrong_outer, TrailingBytes::Strict),
        Err(LobbyCodecError::UnsupportedOuterOpcode(7))
    );

    let mut wrong_length = tunnel.clone();
    wrong_length[3..7].copy_from_slice(&99u32.to_le_bytes());
    assert_eq!(
        decode_tunnel(&wrong_length, TrailingBytes::Strict),
        Err(LobbyCodecError::InnerLengthMismatch {
            declared: 99,
            actual: 48
        })
    );
}

/// Reads the private 2378-byte capture and requires a byte-exact round trip.
///
/// Run with:
/// `OSFR_KART_LOBBY_CAPTURE=<...racing-23867-flow0-frame44523-op102-sub2.bin>
///  cargo test --test kart_lobby_native -- --ignored --exact private_capture_roundtrips_the_full_2378_byte_body`
#[test]
#[ignore = "requires the private lab capture named by OSFR_KART_LOBBY_CAPTURE"]
fn private_capture_roundtrips_the_full_2378_byte_body() {
    use sha2::{Digest, Sha256};

    let path = std::env::var_os("OSFR_KART_LOBBY_CAPTURE")
        .expect("OSFR_KART_LOBBY_CAPTURE must point at the preserved 102/2 frame");
    let raw = std::fs::read(&path).expect("readable capture");
    assert_eq!(
        Sha256::digest(&raw)[..]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "4f6d75c49554e39588a8117a94a650b0154f5d40bb0bf6a45a1e0073a2cb6932"
    );
    assert_eq!(raw.len(), 2393, "original tunnel frame length");
    assert_eq!(&raw[..2], &5u16.to_le_bytes());
    assert_eq!(raw[2], 1);
    assert_eq!(&raw[3..7], &2386u32.to_le_bytes());
    let body = &raw[15..];
    assert_eq!(body.len(), 2378);
    assert_eq!(
        Sha256::digest(body)[..]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "51c8f011c781fe376496c851889380ba36f6bbed911daa60eb964ecc71fd5a0c"
    );

    // The capture is an exact-consumption frame: strict decodes it and re-encodes byte for byte.
    let decoded = decode_tunnel(&raw, TrailingBytes::Strict).expect("capture decodes strictly");
    assert_eq!(decoded.trailing_bytes, 0);
    assert_eq!(encode_tunnel(5, 1, &decoded.body).expect("re-encodes"), raw);
    assert_eq!(encode_body(&decoded.body).expect("re-encodes"), body);

    let body_decoded = &decoded.body;
    assert_eq!(body_decoded.games.len(), 6);
    assert_eq!(body_decoded.options.len(), 30);
    assert_eq!(body_decoded.values.len(), 43);
    assert_eq!(body_decoded.display.len(), 37);
    assert_eq!(body_decoded.settings.len(), 17);
    assert_eq!(
        body_decoded.games,
        vec![
            LobbyGamesRow {
                key: 1,
                id: 1,
                game_type: 1,
                unknown: 37,
                name_id: 3030
            },
            LobbyGamesRow {
                key: 2,
                id: 2,
                game_type: 2,
                unknown: 0,
                name_id: 0
            },
            LobbyGamesRow {
                key: 3,
                id: 3,
                game_type: 3,
                unknown: 0,
                name_id: 0
            },
            LobbyGamesRow {
                key: 4,
                id: 4,
                game_type: 4,
                unknown: 65,
                name_id: 0
            },
            LobbyGamesRow {
                key: 5,
                id: 5,
                game_type: 5,
                unknown: 68,
                name_id: 0
            },
            LobbyGamesRow {
                key: 6,
                id: 6,
                game_type: 6,
                unknown: 60,
                name_id: 417078
            },
        ]
    );
    assert_eq!(
        body_decoded.display[..8]
            .iter()
            .map(|row| (row.key, row.locale_id, row.text.clone()))
            .collect::<Vec<_>>(),
        vec![
            (1, 3032, b"2".to_vec()),
            (2, 3033, b"4".to_vec()),
            (3, 3034, b"6".to_vec()),
            (4, 3036, b"1".to_vec()),
            (5, 3037, b"3".to_vec()),
            (6, 3040, b"5".to_vec()),
            (7, 384347, b"64".to_vec()),
            (8, 384347, b"68".to_vec()),
        ]
    );
    assert_eq!(body_decoded.settings.len(), 17);
    assert_eq!(
        body_decoded.settings[0].fields[2], 3031,
        "settings row name id"
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
