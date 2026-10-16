use std::time::{Duration, Instant};

use miniz_oxide::deflate::compress_to_vec_zlib;
use osfr_tools::session::{decode_controls, SessionRequest};
use osfr_tools::soe::{
    seeded_crc32, Ack, Codec, Error, Packet, ReceiveDisposition, ReliableReceiver, CUMULATIVE_ACK,
    RELIABLE_DATA, RELIABLE_FRAGMENT, SELECTIVE_ACK,
};
use osfr_tools::soe_send::{ReliableSender, SendError, SenderConfig};

const SEED: u32 = 1_884_358_976;
const MAX_LOGICAL_MESSAGE: usize = 1_048_576;

fn data(sequence: u16, bytes: &[u8]) -> Packet {
    packet(RELIABLE_DATA, sequence, bytes)
}

fn packet(opcode: u16, sequence: u16, bytes: &[u8]) -> Packet {
    let mut payload = sequence.to_be_bytes().to_vec();
    payload.extend_from_slice(bytes);
    Packet { opcode, payload }
}

fn first_fragment(sequence: u16, total: u32, bytes: &[u8]) -> Packet {
    let mut body = total.to_be_bytes().to_vec();
    body.extend_from_slice(bytes);
    packet(RELIABLE_FRAGMENT, sequence, &body)
}

fn checked_zlib_frame(opcode: u16, compressed: &[u8], crc_len: usize) -> Vec<u8> {
    let mut wire = opcode.to_be_bytes().to_vec();
    wire.push(1);
    wire.extend_from_slice(compressed);
    let checksum = seeded_crc32(SEED, &wire).to_be_bytes();
    wire.extend_from_slice(&checksum[4 - crc_len..]);
    wire
}

fn crc_controls(wire: &mut [u8], crc_len: usize) {
    let content_len = wire.len() - crc_len;
    let checksum = seeded_crc32(SEED, &wire[..content_len]).to_be_bytes();
    wire[content_len..].copy_from_slice(&checksum[4 - crc_len..]);
}

fn framed_child(opcode: u16, payload: &[u8]) -> Vec<u8> {
    let length = 2 + payload.len();
    assert!(length <= u8::MAX as usize);
    let mut child = vec![length as u8];
    child.extend_from_slice(&opcode.to_be_bytes());
    child.extend_from_slice(payload);
    child
}

#[test]
fn t1_session_handshake_rejects_every_truncation_bad_magic_crc_and_buffer_boundary() {
    let original = include_bytes!("fixtures/session-request.bin");
    assert_eq!(SessionRequest::parse(original).unwrap().buffer_size, 512);

    for length in 0..original.len() {
        assert!(
            SessionRequest::parse(&original[..length]).is_err(),
            "cut={length}"
        );
    }
    let mut longer = original.to_vec();
    longer.push(0);
    assert!(SessionRequest::parse(&longer).is_err());

    for byte in 0..2 {
        let mut bad_magic = original.to_vec();
        bad_magic[byte] ^= 0x80;
        assert!(SessionRequest::parse(&bad_magic).is_err());
    }

    for (crc, accepted) in [
        (0, true),
        (1, true),
        (2, true),
        (3, true),
        (4, true),
        (5, false),
    ] {
        let mut request = original.to_vec();
        request[2..6].copy_from_slice(&(crc as u32).to_be_bytes());
        assert_eq!(
            SessionRequest::parse(&request).is_ok(),
            accepted,
            "crc={crc}"
        );
        assert_eq!(
            Codec::new(SEED, crc, true, 1024).is_ok(),
            accepted,
            "codec crc={crc}"
        );
    }

    for (buffer, accepted) in [
        (0, false),
        (63, false),
        (64, true),
        (512, true),
        (65_507, true),
        (65_508, false),
    ] {
        let mut request = original.to_vec();
        request[10..14].copy_from_slice(&(buffer as u32).to_be_bytes());
        assert_eq!(
            SessionRequest::parse(&request).is_ok(),
            accepted,
            "buffer={buffer}"
        );
    }
}

#[test]
fn t1_data_frame_lengths_crc_flags_and_datagram_limits_are_checked() {
    let codec = Codec::new(SEED, 3, true, MAX_LOGICAL_MESSAGE).unwrap();
    let wire = codec.encode(RELIABLE_DATA, &[0, 7, 1, 2, 3, 4]).unwrap();
    assert_eq!(codec.decode(&wire).unwrap().payload, [0, 7, 1, 2, 3, 4]);
    for length in 0..wire.len() {
        assert!(codec.decode(&wire[..length]).is_err(), "cut={length}");
    }
    let mut longer = wire.clone();
    longer.push(0);
    assert!(codec.decode(&longer).is_err());

    let mut bad_crc = wire.clone();
    *bad_crc.last_mut().unwrap() ^= 1;
    assert_eq!(codec.decode(&bad_crc), Err(Error::BadCrc));

    let mut bad_flag = codec.encode(RELIABLE_DATA, &[0, 0]).unwrap();
    bad_flag[2] = 2;
    crc_controls(&mut bad_flag, 3);
    assert_eq!(
        codec.decode(&bad_flag),
        Err(Error::InvalidCompressionFlag(2))
    );

    for crc_len in 0..=4 {
        let no_crc = Codec::new(SEED, crc_len, false, 1024).unwrap();
        let roundtrip = no_crc.encode(RELIABLE_DATA, &[0, 0, 9]).unwrap();
        assert_eq!(no_crc.decode(&roundtrip).unwrap().payload, [0, 0, 9]);
    }
    assert_eq!(codec.decode(&vec![0; 65_508]), Err(Error::DatagramTooLarge));
}

#[test]
fn t1_zlib_bombs_trailing_bytes_and_concatenated_streams_fail_bounded() {
    let codec = Codec::new(SEED, 3, true, 16 * 1024 * 1024).unwrap();
    let bomb = compress_to_vec_zlib(&vec![b'Z'; MAX_LOGICAL_MESSAGE * 4], 6);
    assert!(matches!(
        codec.decode(&checked_zlib_frame(RELIABLE_DATA, &bomb, 3)),
        Err(Error::PayloadTooLarge)
    ));

    let compressed = compress_to_vec_zlib(&vec![b'x'; 256], 6);
    let normal = checked_zlib_frame(RELIABLE_DATA, &compressed, 3);
    assert_eq!(codec.decode(&normal).unwrap().payload.len(), 256);

    let mut trailing = normal[..normal.len() - 3].to_vec();
    trailing.push(0x5a);
    let checksum = seeded_crc32(SEED, &trailing).to_be_bytes();
    trailing.extend_from_slice(&checksum[1..]);
    assert_eq!(codec.decode(&trailing), Err(Error::InvalidZlib));

    let mut concatenated = normal[..normal.len() - 3].to_vec();
    concatenated.extend_from_slice(&compress_to_vec_zlib(b"second stream", 6));
    let checksum = seeded_crc32(SEED, &concatenated).to_be_bytes();
    concatenated.extend_from_slice(&checksum[1..]);
    assert_eq!(codec.decode(&concatenated), Err(Error::InvalidZlib));
}

#[test]
fn t2_reordered_and_conflicting_fragments_are_bounded_and_only_valid_data_is_acked() {
    let mut receiver = ReliableReceiver::new(65_535, 16, 64, 128).unwrap();
    let future = packet(RELIABLE_FRAGMENT, 0, b"ef");
    let buffered = receiver.receive(&future).unwrap();
    assert_eq!(buffered.disposition, ReceiveDisposition::Buffered);
    assert_eq!(buffered.acknowledgement, None);
    assert_eq!(receiver.receive(&future).unwrap().acknowledgement, None);
    assert_eq!(
        receiver.receive(&packet(RELIABLE_FRAGMENT, 0, b"zz")),
        Err(Error::ConflictingDuplicate)
    );

    let completed = receiver
        .receive(&first_fragment(65_535, 6, b"abcd"))
        .unwrap();
    assert_eq!(completed.messages, vec![b"abcdef".to_vec()]);
    assert_eq!(
        completed.acknowledgement,
        Some(Ack {
            opcode: CUMULATIVE_ACK,
            sequence: 0,
        })
    );
    assert_eq!(receiver.buffered_bytes(), 0);
}

#[test]
fn t2_zero_and_oversized_fragment_lengths_never_advance_or_ack_invalid_sequences() {
    for (total, expected) in [
        (0, Error::InvalidFragment),
        (MAX_LOGICAL_MESSAGE as u32 + 1, Error::PayloadTooLarge),
    ] {
        let mut receiver = ReliableReceiver::new(0, 8, 16 * 1024 * 1024, 128).unwrap();
        let queued = receiver.receive(&first_fragment(1, total, b"a")).unwrap();
        assert_eq!(queued.acknowledgement, None);
        assert_eq!(queued.disposition, ReceiveDisposition::Buffered);
        let outcome = receiver.receive(&data(0, b"valid predecessor")).unwrap();
        assert_eq!(outcome.messages, vec![b"valid predecessor".to_vec()]);
        assert_eq!(outcome.rejected_buffered, Some((1, expected)));
        assert_eq!(
            outcome.acknowledgement,
            Some(Ack {
                opcode: CUMULATIVE_ACK,
                sequence: 0,
            })
        );
        assert_eq!(receiver.next_sequence(), 1);
        assert_eq!(receiver.buffered_bytes(), 0);
    }
}

#[test]
fn t2_logical_messages_and_reorder_memory_match_the_inbound_parser_limit() {
    let mut receiver = ReliableReceiver::new(0, 32, 16 * 1024 * 1024, usize::MAX).unwrap();
    let accepted = receiver
        .receive(&data(0, &vec![7; MAX_LOGICAL_MESSAGE]))
        .unwrap();
    assert_eq!(accepted.messages[0].len(), MAX_LOGICAL_MESSAGE);

    let mut receiver = ReliableReceiver::new(0, 32, 16 * 1024 * 1024, usize::MAX).unwrap();
    assert!(matches!(
        receiver.receive(&data(1, &vec![7; MAX_LOGICAL_MESSAGE + 1])),
        Err(Error::PayloadTooLarge)
    ));
    assert_eq!(receiver.buffered_bytes(), 0);

    let mut receiver = ReliableReceiver::new(0, 16, 16 * 1024 * 1024, usize::MAX).unwrap();
    let chunk = vec![9; 500_000];
    for sequence in 1..=4 {
        let outcome = receiver.receive(&data(sequence, &chunk)).unwrap();
        assert_eq!(outcome.disposition, ReceiveDisposition::Buffered);
        assert_eq!(
            outcome.acknowledgement,
            Some(Ack {
                opcode: SELECTIVE_ACK,
                sequence,
            })
        );
    }
    assert!(receiver.buffered_bytes() <= 2 * 1024 * 1024);
    assert!(matches!(
        receiver.receive(&data(5, &chunk)),
        Err(Error::BufferLimit)
    ));
    assert!(receiver.buffered_bytes() <= 2 * 1024 * 1024);

    let codec = Codec::new(SEED, 3, true, 16 * 1024 * 1024).unwrap();
    let mut sender = ReliableSender::new(
        codec,
        SenderConfig {
            datagram_limit: 512,
            window: 32,
            max_message: 16 * 1024 * 1024,
            max_queued_bytes: 2 * 1024 * 1024,
            retry_ms: 100,
            max_retries: 2,
        },
    )
    .unwrap();
    assert_eq!(
        sender.queue_message(&vec![0; MAX_LOGICAL_MESSAGE + 1]),
        Err(SendError::MessageTooLarge)
    );
}

#[test]
fn t2_sequence_wrap_and_half_range_are_unambiguous() {
    let mut receiver = ReliableReceiver::new(65_535, 8, 16, 64).unwrap();
    assert_eq!(
        receiver
            .receive(&data(65_535, b"before wrap"))
            .unwrap()
            .messages,
        vec![b"before wrap".to_vec()]
    );
    assert_eq!(
        receiver.receive(&data(0, b"after wrap")).unwrap().messages,
        vec![b"after wrap".to_vec()]
    );
    assert_eq!(
        receiver.receive(&data(0x8001, b"half range")),
        Err(Error::AmbiguousSequence)
    );
    assert_eq!(receiver.next_sequence(), 1);
}

#[test]
fn t2_control_bundles_reject_excess_depth_and_child_count_atomically() {
    let codec = Codec::new(SEED, 3, true, MAX_LOGICAL_MESSAGE).unwrap();
    let mut nested = framed_child(6, &[]);
    for _ in 0..2 {
        nested = framed_child(3, &nested);
    }
    let decoded = decode_controls(&codec, &codec.encode(3, &nested).unwrap()).unwrap();
    assert_eq!(
        decoded,
        vec![Packet {
            opcode: 6,
            payload: Vec::new()
        }]
    );

    nested = framed_child(3, &nested);
    assert!(decode_controls(&codec, &codec.encode(3, &nested).unwrap()).is_err());

    let child = framed_child(6, &[]);
    for count in [256, 257] {
        let mut bundle = Vec::with_capacity(child.len() * count);
        for _ in 0..count {
            bundle.extend_from_slice(&child);
        }
        let result = decode_controls(&codec, &codec.encode(3, &bundle).unwrap());
        if count == 256 {
            assert_eq!(result.unwrap().len(), 256);
        } else {
            assert!(result.is_err());
        }
    }
}

#[test]
fn t2_incomplete_fragment_expires_despite_keepalive() {
    let start = Instant::now();
    let mut receiver = ReliableReceiver::new(0, 8, 64, 128).unwrap();
    assert!(receiver
        .receive_at(&first_fragment(0, 8, b"ab"), start)
        .unwrap()
        .messages
        .is_empty());
    assert_eq!(
        receiver.reassembly_deadline(),
        Some(start + Duration::from_secs(30))
    );

    let codec = Codec::new(SEED, 3, true, MAX_LOGICAL_MESSAGE).unwrap();
    for _ in 0..10 {
        assert_eq!(decode_controls(&codec, &[0, 6]).unwrap()[0].opcode, 6);
    }
    assert!(!receiver.expire_reassembly(start + Duration::from_secs(29)));
    assert!(receiver.expire_reassembly(start + Duration::from_secs(30)));
    assert_eq!(receiver.fragment_bytes(), 0);
    assert_eq!(receiver.buffered_bytes(), 0);
    assert_eq!(
        receiver.receive_at(
            &packet(RELIABLE_FRAGMENT, 1, b"cdefgh"),
            start + Duration::from_secs(31),
        ),
        Err(Error::ReceiverPoisoned)
    );
}
