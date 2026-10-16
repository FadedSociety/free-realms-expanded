//! Explicit, local-only capture replay. No game payloads are embedded here.
//! `scripts/verify-soe-capture.ps1` verifies input SHA-256 hashes and supplies the
//! index. The Rust codec independently checks envelopes and ordered reassembly.

use osfr_tools::soe::{
    CUMULATIVE_ACK, Codec, Error, RELIABLE_DATA, RELIABLE_FRAGMENT, ReceiveDisposition,
    ReliableReceiver,
};

#[test]
#[ignore = "requires an explicit local capture index from scripts/verify-soe-capture.ps1"]
fn replay_local_legacy_capture() {
    let index_path = std::env::var_os("OSFR_CAPTURE_INDEX")
        .expect("Run scripts/verify-soe-capture.ps1 -CaptureDirectory <local lab capture>");
    let report_path = std::env::var_os("OSFR_CAPTURE_REPORT").expect("capture report path missing");
    let index = std::fs::read_to_string(index_path).expect("read verified capture index");
    let mut seed = None;
    let mut crc_length = None;
    let mut expected_count = None;
    let mut expected_bytes = None;
    let mut frames = Vec::new();
    let mut expected_messages = Vec::new();
    for line in index.trim_start_matches('\u{feff}').lines() {
        let fields: Vec<_> = line.split('\t').collect();
        match fields[0] {
            "seed" => seed = Some(fields[1].parse::<u32>().unwrap()),
            "crc_length" => crc_length = Some(fields[1].parse::<u8>().unwrap()),
            "expected_messages" => expected_count = Some(fields[1].parse::<usize>().unwrap()),
            "expected_application_bytes" => {
                expected_bytes = Some(fields[1].parse::<usize>().unwrap())
            }
            "frame" => {
                assert_eq!(fields.len(), 4);
                frames.push((
                    fields[1],
                    fields[2].parse::<u16>().unwrap(),
                    fields[3].parse::<usize>().unwrap(),
                ));
            }
            "message" => {
                assert_eq!(fields.len(), 3);
                expected_messages.push((fields[1], fields[2].parse::<usize>().unwrap()));
            }
            kind => panic!("unknown capture index kind {kind}"),
        }
    }
    let codec = Codec::new(seed.unwrap(), crc_length.unwrap(), true, 1_048_576).unwrap();
    let mut receiver = ReliableReceiver::new(0, 1024, 64 * 1024 * 1024, 8 * 1024 * 1024).unwrap();
    let mut delivered = Vec::new();
    let mut compressed = 0;
    let mut fragments = 0;
    let mut data = 0;
    let mut acknowledgements = 0;
    let mut corruption_rejections = 0;
    for (path, expected_opcode, expected_body_length) in &frames {
        let wire = std::fs::read(path).expect("read verified captured frame");
        let packet = codec
            .decode(&wire)
            .unwrap_or_else(|error| panic!("{path}: {error}"));
        assert_eq!(packet.opcode, *expected_opcode, "{path}: opcode");
        assert_eq!(
            packet.payload.len(),
            *expected_body_length,
            "{path}: expanded body length"
        );
        compressed += usize::from(wire[2] == 1);
        let mut corrupt = wire.clone();
        *corrupt.last_mut().unwrap() ^= 1;
        assert_eq!(
            codec.decode(&corrupt),
            Err(Error::BadCrc),
            "{path}: modified CRC accepted"
        );
        corruption_rejections += 1;
        match packet.opcode {
            RELIABLE_DATA | RELIABLE_FRAGMENT => {
                data += usize::from(packet.opcode == RELIABLE_DATA);
                fragments += usize::from(packet.opcode == RELIABLE_FRAGMENT);
                let outcome = receiver
                    .receive(&packet)
                    .unwrap_or_else(|error| panic!("{path}: {error}"));
                assert_eq!(
                    outcome.disposition,
                    ReceiveDisposition::Accepted,
                    "{path}: capture order"
                );
                assert_eq!(
                    outcome.rejected_buffered, None,
                    "{path}: invalid queued packet"
                );
                assert!(
                    outcome.acknowledgement.is_some(),
                    "{path}: missing acknowledgement"
                );
                delivered.extend(outcome.messages);
            }
            CUMULATIVE_ACK => {
                assert_eq!(packet.payload.len(), 2, "{path}: ack sequence length");
                acknowledgements += 1;
            }
            opcode => panic!("{path}: unhandled capture opcode {opcode}"),
        }
    }
    assert_eq!(receiver.buffered_bytes(), 0);
    assert_eq!(
        receiver.fragment_bytes(),
        0,
        "incomplete final captured fragment"
    );
    assert_eq!(delivered.len(), expected_count.unwrap());
    assert_eq!(delivered.len(), expected_messages.len());
    let mut application_bytes = 0;
    for (actual, (path, expected_length)) in delivered.iter().zip(&expected_messages) {
        let expected = std::fs::read(path).expect("read verified captured application message");
        assert_eq!(actual.len(), *expected_length, "{path}: message length");
        assert!(actual == &expected, "{path}: reassembled bytes differ");
        application_bytes += actual.len();
    }
    assert_eq!(application_bytes, expected_bytes.unwrap());
    let report = format!(
        "{{\"decoded_frames\":{},\"compressed_frames\":{compressed},\"reliable_data_frames\":{data},\"fragment_frames\":{fragments},\"acknowledgement_frames\":{acknowledgements},\"crc_corruption_rejections\":{corruption_rejections},\"complete_messages\":{},\"application_bytes\":{application_bytes},\"next_receive_sequence\":{},\"byte_comparisons_passed\":true}}\n",
        frames.len(),
        delivered.len(),
        receiver.next_sequence(),
    );
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(report_path)
        .unwrap()
        .write_all(report.as_bytes())
        .unwrap();
    println!("{}", report.trim());
}
