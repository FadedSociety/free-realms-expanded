use osfr_tools::session::{SessionRequest, clock_reflect, decode_controls};
use osfr_tools::soe::{Codec, Packet};

#[test]
fn shared_session_matches_compiled_fixture_and_real_client_crc_length() {
    let request = include_bytes!("fixtures/session-request.bin");
    let response = include_bytes!("fixtures/session-response.bin");
    assert_eq!(SessionRequest::parse(request).unwrap().response(), response);
    let mut real = request.to_vec();
    real[2..6].copy_from_slice(&3u32.to_be_bytes());
    let parsed = SessionRequest::parse(&real).unwrap();
    assert_eq!(parsed.crc_length, 3);
    assert_eq!(parsed.response()[10], 3);
    for cut in 0..request.len() {
        assert!(SessionRequest::parse(&request[..cut]).is_err());
    }
}

#[test]
fn cooked_multi_packet_children_have_no_individual_crc_headers() {
    let codec = Codec::new(123, 3, true, 1024).unwrap();
    let body = [4, 0, 21, 0, 9, 2, 0, 6];
    let decoded = decode_controls(&codec, &codec.encode(3, &body).unwrap()).unwrap();
    assert_eq!(
        decoded,
        vec![
            Packet {
                opcode: 21,
                payload: vec![0, 9]
            },
            Packet {
                opcode: 6,
                payload: Vec::new()
            },
        ]
    );
    let bad_tail = [4, 0, 21, 0, 9, 5, 0, 6];
    assert!(decode_controls(&codec, &codec.encode(3, &bad_tail).unwrap()).is_err());
    assert!(decode_controls(&codec, &codec.encode(3, &[]).unwrap()).is_err());
    assert_eq!(decode_controls(&codec, &[0, 6]).unwrap()[0].opcode, 6);
}

#[test]
fn multi_child_length_stays_one_byte_at_255() {
    // Pin: client Multi (opcode 3) reads ONE length byte, dispatcher 0x010e3dd0 case 3 at
    // 0x010e4294 (`movzx ecx, byte ptr [eax]`). Only Group (0x19) uses the `FF hi lo` form.
    let codec = Codec::new(123, 3, true, 1024).unwrap();
    let body = [vec![255, 0, 21], vec![9; 253]].concat();
    let decoded = decode_controls(&codec, &codec.encode(3, &body).unwrap()).unwrap();
    assert_eq!(
        decoded,
        vec![Packet {
            opcode: 21,
            payload: vec![9; 253]
        }]
    );
}

#[test]
fn clock_reflection_preserves_peer_fields_and_supplies_server_counts() {
    let packet = Packet {
        opcode: 7,
        payload: (0..38).collect(),
    };
    let result = clock_reflect(&packet, 1000, 33, 44).unwrap();
    assert_eq!(result.opcode, 8);
    assert_eq!(result.payload.len(), 38);
    assert_eq!(&result.payload[..2], &packet.payload[..2]);
    assert_eq!(&result.payload[6..22], &packet.payload[22..38]);
    assert_eq!(&result.payload[22..30], &33u64.to_be_bytes());
    assert_eq!(&result.payload[30..38], &44u64.to_be_bytes());
    assert!(
        clock_reflect(
            &Packet {
                opcode: 7,
                payload: vec![0; 37]
            },
            0,
            0,
            0
        )
        .is_none()
    );
}
