use osfr_tools::soe::{
    Ack, CUMULATIVE_ACK, Codec, Error, Packet, RELIABLE_DATA, RELIABLE_FRAGMENT,
    ReceiveDisposition, ReliableReceiver, SELECTIVE_ACK, seeded_crc32,
};

const SEED: u32 = 1_884_358_976;

fn hex(text: &str) -> Vec<u8> {
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

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

#[test]
fn compiled_legacy_serializer_golden_vectors() {
    let mut count = 0;
    for line in include_str!("fixtures/transport-vectors.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7);
        let label = format!("{} seed={} crc={}", fields[0], fields[1], fields[2]);
        let seed = fields[1].parse().unwrap();
        let crc_len: u8 = fields[2].parse().unwrap();
        let compression = fields[3] == "1";
        let payload = hex(fields[4]);
        let legacy_wire = hex(fields[5]);
        let checksum = u32::from_str_radix(fields[6], 16).unwrap();
        let codec = Codec::new(seed, crc_len, compression, 4096).unwrap();
        let decoded = codec
            .decode(&legacy_wire)
            .unwrap_or_else(|e| panic!("{label}: {e}"));
        assert_eq!(decoded.payload, payload, "{label}");
        assert_eq!(
            seeded_crc32(
                seed,
                &legacy_wire[..legacy_wire.len() - usize::from(crc_len)]
            ),
            checksum,
            "{label}"
        );
        let encoded = codec.encode(decoded.opcode, &payload).unwrap();
        if !compression || payload.len() <= 100 {
            assert_eq!(encoded, legacy_wire, "{label}");
        } else {
            // Different conforming zlib encoders need not emit identical bytes.
            assert_eq!(encoded[2], 1, "{label}");
            assert_eq!(codec.decode(&encoded).unwrap(), decoded, "{label}");
        }
        count += 1;
    }
    assert_eq!(count, 105);
}

#[test]
fn corrupt_crc_truncated_frames_and_configuration_are_rejected() {
    assert!(matches!(
        Codec::new(SEED, 5, true, 1024),
        Err(Error::InvalidConfiguration)
    ));
    assert!(matches!(
        Codec::new(SEED, 2, true, 0),
        Err(Error::InvalidConfiguration)
    ));
    for crc_len in 1..=4 {
        let codec = Codec::new(SEED, crc_len, true, 1024).unwrap();
        let wire = codec.encode(RELIABLE_DATA, &[0, 0, 1, 2]).unwrap();
        for i in 0..wire.len() {
            let mut corrupt = wire.clone();
            corrupt[i] ^= 0x80;
            assert_eq!(
                codec.decode(&corrupt),
                Err(Error::BadCrc),
                "crc={crc_len} byte={i}"
            );
        }
        for length in 0..(3 + usize::from(crc_len)) {
            assert_eq!(codec.decode(&wire[..length]), Err(Error::Truncated));
        }
    }
}

#[test]
fn compression_flags_zlib_checksums_and_expansion_are_bounded() {
    let codec = Codec::new(SEED, 0, true, 200_000).unwrap();
    assert_eq!(
        codec.decode(&[0, 9, 2]),
        Err(Error::InvalidCompressionFlag(2))
    );
    assert_eq!(codec.decode(&[0, 9, 1]), Err(Error::InvalidZlib));
    assert_eq!(
        codec.decode(&[0, 9, 1, 0x78, 0x9c]),
        Err(Error::InvalidZlib)
    );

    let mut wire = codec.encode(RELIABLE_DATA, &vec![b'a'; 200_000]).unwrap();
    assert_eq!(
        Codec::new(SEED, 0, true, 64).unwrap().decode(&wire),
        Err(Error::PayloadTooLarge)
    );
    let decoded = codec.decode(&wire).unwrap();
    assert_eq!(decoded.payload.len(), 200_000);
    wire.push(0);
    assert_eq!(codec.decode(&wire), Err(Error::InvalidZlib));
    wire.pop();
    let last = wire.len() - 1;
    wire[last] ^= 1;
    assert_eq!(codec.decode(&wire), Err(Error::InvalidZlib));

    let codec = Codec::new(SEED, 0, false, 4).unwrap();
    assert_eq!(
        codec.decode(&[0, 9, 1, 2, 3, 4, 5]),
        Err(Error::PayloadTooLarge)
    );
    assert_eq!(
        codec.encode(9, &[1, 2, 3, 4, 5]),
        Err(Error::PayloadTooLarge)
    );
    let codec = Codec::new(SEED, 4, false, 100_000).unwrap();
    assert_eq!(
        codec.encode(9, &vec![0; 65_507]),
        Err(Error::DatagramTooLarge)
    );
    assert_eq!(codec.decode(&vec![0; 65_508]), Err(Error::DatagramTooLarge));
}

#[test]
fn ordered_delivery_buffers_gaps_and_survives_sequence_rollover() {
    let mut receiver = ReliableReceiver::new(65_535, 16, 1024, 4096).unwrap();
    let future = receiver.receive(&data(0, b"second")).unwrap();
    assert!(future.messages.is_empty());
    assert_eq!(future.disposition, ReceiveDisposition::Buffered);
    assert_eq!(
        future.acknowledgement,
        Some(Ack {
            opcode: SELECTIVE_ACK,
            sequence: 0
        })
    );
    let duplicate = receiver.receive(&data(0, b"second")).unwrap();
    assert_eq!(duplicate.disposition, ReceiveDisposition::Duplicate);
    assert_eq!(receiver.buffered_bytes(), 8);
    let contiguous = receiver.receive(&data(65_535, b"first")).unwrap();
    assert_eq!(
        contiguous.messages,
        vec![b"first".to_vec(), b"second".to_vec()]
    );
    assert_eq!(
        contiguous.acknowledgement,
        Some(Ack {
            opcode: CUMULATIVE_ACK,
            sequence: 0
        })
    );
    assert_eq!(receiver.next_sequence(), 1);
    assert_eq!(receiver.buffered_bytes(), 0);
    let duplicate = receiver.receive(&data(65_535, b"first")).unwrap();
    assert!(duplicate.messages.is_empty());
    assert_eq!(duplicate.disposition, ReceiveDisposition::Duplicate);
    assert_eq!(duplicate.acknowledgement, contiguous.acknowledgement);
}

#[test]
fn receive_windows_and_buffer_limits_do_not_ack_dropped_data() {
    let mut receiver = ReliableReceiver::new(0, 4, 8, 5).unwrap();
    assert_eq!(
        receiver
            .receive(&data(65_535, b"old"))
            .unwrap()
            .acknowledgement,
        None
    );
    let outside = receiver.receive(&data(4, b"future")).unwrap();
    assert_eq!(outside.disposition, ReceiveDisposition::OutsideWindow);
    assert_eq!(outside.acknowledgement, None);
    assert_eq!(
        receiver.receive(&data(0x8000, b"ambiguous")),
        Err(Error::AmbiguousSequence)
    );
    receiver.receive(&data(1, b"abc")).unwrap();
    assert_eq!(receiver.buffered_bytes(), 5);
    assert_eq!(receiver.receive(&data(2, b"x")), Err(Error::BufferLimit));
    assert_eq!(
        receiver.receive(&data(1, b"xyz")),
        Err(Error::ConflictingDuplicate)
    );
    assert_eq!(receiver.next_sequence(), 0);
    assert!(matches!(
        ReliableReceiver::new(0, 0x8000, 8, 5),
        Err(Error::InvalidConfiguration)
    ));
}

#[test]
fn oversized_future_data_is_rejected_before_buffering_or_acknowledgement() {
    let mut receiver = ReliableReceiver::new(0, 16, 4, 128).unwrap();
    // Previously the first-fragment header allowance admitted this data packet,
    // selectively acknowledged it, then rejected it only after filling the gap.
    assert_eq!(
        receiver.receive(&data(1, b"12345")),
        Err(Error::PayloadTooLarge)
    );
    assert_eq!(receiver.buffered_bytes(), 0);
    assert_eq!(receiver.next_sequence(), 0);
    let future = receiver.receive(&data(1, b"1234")).unwrap();
    assert_eq!(
        future.acknowledgement,
        Some(Ack {
            opcode: SELECTIVE_ACK,
            sequence: 1
        })
    );
    let delivered = receiver.receive(&data(0, b"zero")).unwrap();
    assert_eq!(delivered.messages, vec![b"zero".to_vec(), b"1234".to_vec()]);
    assert_eq!(delivered.rejected_buffered, None);
    assert_eq!(receiver.next_sequence(), 2);

    // First fragments retain their separate length-header allowance.
    let mut receiver = ReliableReceiver::new(0, 16, 4, 128).unwrap();
    receiver.receive(&first_fragment(1, 4, b"1234")).unwrap();
    let delivered = receiver.receive(&data(0, b"zero")).unwrap();
    assert_eq!(delivered.messages, vec![b"zero".to_vec(), b"1234".to_vec()]);
}

#[test]
fn fragments_reassemble_once_across_gap_duplicate_and_rollover() {
    let mut receiver = ReliableReceiver::new(65_535, 16, 6, 64).unwrap();
    receiver
        .receive(&packet(RELIABLE_FRAGMENT, 1, b"ef"))
        .unwrap();
    let first = first_fragment(65_535, 6, b"ab");
    assert!(receiver.receive(&first).unwrap().messages.is_empty());
    assert_eq!(receiver.fragment_bytes(), 2);
    assert!(receiver.receive(&first).unwrap().messages.is_empty());
    assert_eq!(receiver.fragment_bytes(), 2);
    let result = receiver
        .receive(&packet(RELIABLE_FRAGMENT, 0, b"cd"))
        .unwrap();
    assert_eq!(result.messages, vec![b"abcdef".to_vec()]);
    assert_eq!(receiver.next_sequence(), 2);
    assert_eq!(receiver.fragment_bytes(), 0);
    assert_eq!(
        result.acknowledgement,
        Some(Ack {
            opcode: CUMULATIVE_ACK,
            sequence: 1
        })
    );
    assert_eq!(
        result.acknowledgement.unwrap().packet(),
        Packet {
            opcode: 21,
            payload: vec![0, 1]
        }
    );
}

#[test]
fn malformed_fragments_leave_sequence_and_partial_data_unchanged() {
    let mut receiver = ReliableReceiver::new(0, 16, 8, 128).unwrap();
    assert_eq!(
        receiver.receive(&first_fragment(0, 0, b"a")),
        Err(Error::InvalidFragment)
    );
    assert_eq!(
        receiver.receive(&first_fragment(0, 9, b"a")),
        Err(Error::PayloadTooLarge)
    );
    assert_eq!(
        receiver.receive(&first_fragment(0, 1, b"ab")),
        Err(Error::InvalidFragment)
    );
    assert_eq!(receiver.next_sequence(), 0);
    receiver.receive(&first_fragment(0, 4, b"ab")).unwrap();
    assert_eq!(
        receiver.receive(&data(1, b"interrupt")),
        Err(Error::UnexpectedDataDuringFragment)
    );
    assert_eq!(
        receiver.receive(&packet(RELIABLE_FRAGMENT, 1, b"cde")),
        Err(Error::InvalidFragment)
    );
    assert_eq!(receiver.next_sequence(), 1);
    assert_eq!(receiver.fragment_bytes(), 2);
    let done = receiver
        .receive(&packet(RELIABLE_FRAGMENT, 1, b"cd"))
        .unwrap();
    assert_eq!(done.messages, vec![b"abcd".to_vec()]);
    let single = receiver.receive(&first_fragment(2, 1, b"x")).unwrap();
    assert_eq!(single.messages, vec![b"x".to_vec()]);
}

#[test]
fn invalid_buffered_fragment_preserves_prior_delivery_and_stops_ack_run() {
    let mut receiver = ReliableReceiver::new(0, 16, 8, 128).unwrap();
    receiver.receive(&first_fragment(1, 0, b"invalid")).unwrap();
    let result = receiver.receive(&data(0, b"valid")).unwrap();
    assert_eq!(result.messages, vec![b"valid".to_vec()]);
    assert_eq!(result.rejected_buffered, Some((1, Error::InvalidFragment)));
    assert_eq!(
        result.acknowledgement,
        Some(Ack {
            opcode: CUMULATIVE_ACK,
            sequence: 0
        })
    );
    assert_eq!(receiver.next_sequence(), 1);
    assert_eq!(receiver.buffered_bytes(), 0);
    assert_eq!(
        receiver.receive(&data(1, b"retry")),
        Err(Error::ReceiverPoisoned)
    );
    assert_eq!(receiver.next_sequence(), 1);
    assert_eq!(
        receiver.receive(&data(0, b"duplicate")),
        Err(Error::ReceiverPoisoned)
    );
}

#[test]
fn reordered_retransmissions_deliver_every_sequence_once_over_multiple_rollovers() {
    let start = 65_520u16;
    let mut receiver = ReliableReceiver::new(start, 64, 8, 4096).unwrap();
    let mut delivered = Vec::new();
    // Reverse each loss/reordering window, repeat packets, then fill its head.
    // More than two full sequence spaces prevents a special-case-only rollover.
    for base in (0u32..131_104).step_by(16) {
        for offset in (1u32..16).rev() {
            let index = base + offset;
            let sequence = start.wrapping_add(index as u16);
            let payload = index.to_be_bytes();
            assert!(
                receiver
                    .receive(&data(sequence, &payload))
                    .unwrap()
                    .messages
                    .is_empty()
            );
            assert!(
                receiver
                    .receive(&data(sequence, &payload))
                    .unwrap()
                    .messages
                    .is_empty()
            );
        }
        let result = receiver
            .receive(&data(start.wrapping_add(base as u16), &base.to_be_bytes()))
            .unwrap();
        delivered.extend(
            result
                .messages
                .into_iter()
                .map(|bytes| u32::from_be_bytes(bytes.try_into().unwrap())),
        );
        assert_eq!(receiver.buffered_bytes(), 0);
    }
    assert_eq!(delivered, (0u32..131_104).collect::<Vec<_>>());
    assert_eq!(
        receiver.next_sequence(),
        start.wrapping_add(131_104u32 as u16)
    );
}

#[test]
#[ignore = "requires Windows PowerShell and the local user-supplied LibSOE.dll"]
fn compiled_legacy_decoder_accepts_rust_compression() {
    use std::io::Write;
    let codec = Codec::new(SEED, 2, true, 1_000_000).unwrap();
    let hex_string = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let path = std::env::temp_dir().join(format!("osfr-rust-decoder-{}.tsv", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    writeln!(file, "name\tpayload_hex\tpacket_hex").unwrap();
    let cases = [
        ("small", vec![0, 0, 1, 2, 3]),
        ("threshold100", vec![b'a'; 100]),
        ("threshold101", vec![b'a'; 101]),
        ("binary", (0..=255).collect()),
        ("expanded", vec![b'z'; 200_000]),
    ];
    for (name, payload) in cases {
        let wire = codec.encode(RELIABLE_DATA, &payload).unwrap();
        writeln!(
            file,
            "{name}\t{}\t{}",
            hex_string(&payload),
            hex_string(&wire)
        )
        .unwrap();
    }
    drop(file);
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/check-legacy-decode.ps1");
    let result = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(script)
        .arg("-Vectors")
        .arg(&path)
        .output();
    std::fs::remove_file(path).unwrap();
    let output = result.unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
