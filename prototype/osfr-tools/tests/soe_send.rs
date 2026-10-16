use osfr_tools::soe::{
    Ack, CUMULATIVE_ACK, Codec, Packet, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver,
    SELECTIVE_ACK,
};
use osfr_tools::soe_send::{ReliableSender, SendError, SenderConfig};

const SEED: u32 = 1_884_358_976;

fn codec() -> Codec {
    Codec::new(SEED, 3, true, 1_048_576).unwrap()
}

fn config() -> SenderConfig {
    SenderConfig {
        datagram_limit: 512,
        window: 32,
        max_message: 1_048_576,
        max_queued_bytes: 2_097_152,
        retry_ms: 100,
        max_retries: 2,
    }
}

fn ack(opcode: u16, sequence: u16) -> Packet {
    Ack { opcode, sequence }.packet()
}
fn sequence(packet: &Packet) -> u16 {
    u16::from_be_bytes(packet.payload[..2].try_into().unwrap())
}

fn binary(length: usize) -> Vec<u8> {
    let mut state = 0x5a71_3f89u32;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect()
}

#[test]
fn selective_ack_holes_keep_window_bounded_and_unsent_packets_safe() {
    let mut sender = ReliableSender::new(
        codec(),
        SenderConfig {
            window: 3,
            ..config()
        },
    )
    .unwrap();
    for message in 0..5 {
        sender.queue_message(&[message]).unwrap();
    }
    assert_eq!(sender.pending_packets(), 5);
    assert_eq!(
        sender.acknowledge(&ack(CUMULATIVE_ACK, 0)),
        Err(SendError::InvalidAcknowledgement)
    );
    assert_eq!(sender.poll(0).unwrap().len(), 3);
    sender.acknowledge(&ack(SELECTIVE_ACK, 2)).unwrap();
    assert_eq!(sender.pending_packets(), 4);
    assert!(sender.poll(0).unwrap().is_empty());
    assert_eq!(
        sender.acknowledge(&ack(SELECTIVE_ACK, 2)),
        Err(SendError::InvalidAcknowledgement)
    );
    assert_eq!(
        sender.acknowledge(&ack(CUMULATIVE_ACK, 4)),
        Err(SendError::InvalidAcknowledgement)
    );
    assert_eq!(sender.pending_packets(), 4);
    sender.acknowledge(&ack(CUMULATIVE_ACK, 0)).unwrap();
    let next = sender.poll(1).unwrap();
    assert_eq!(next.len(), 1);
    assert_eq!(sequence(&codec().decode(&next[0]).unwrap()), 3);
    assert_eq!(
        sender.acknowledge(&ack(CUMULATIVE_ACK, 4)),
        Err(SendError::InvalidAcknowledgement)
    );
    sender.acknowledge(&ack(CUMULATIVE_ACK, 3)).unwrap();
    assert_eq!(
        sender.acknowledge(&ack(CUMULATIVE_ACK, 3)),
        Err(SendError::InvalidAcknowledgement)
    );
    let last = sender.poll(2).unwrap();
    assert_eq!(last.len(), 1);
    assert_eq!(sequence(&codec().decode(&last[0]).unwrap()), 4);
    sender.acknowledge(&ack(SELECTIVE_ACK, 4)).unwrap();
    assert!(sender.is_idle());
}

#[test]
fn loss_reordering_and_duplicates_reassemble_multiple_messages_once() {
    let codec = codec();
    let mut sender = ReliableSender::new(
        codec.clone(),
        SenderConfig {
            window: 4,
            ..config()
        },
    )
    .unwrap();
    let mut receiver = ReliableReceiver::new(0, 64, 1_048_576, 1_048_576).unwrap();
    let messages = vec![
        b"hello".to_vec(),
        binary(5000),
        vec![b'z'; 1100],
        binary(700),
    ];
    for message in &messages {
        sender.queue_message(message).unwrap();
    }
    let mut dropped = [false; 2];
    let mut delivered = Vec::new();
    for tick in 0..100 {
        let output = sender.poll(tick * 100).unwrap();
        for wire in output.into_iter().rev() {
            let packet = codec.decode(&wire).unwrap();
            let seq = sequence(&packet);
            if seq == 0 && !dropped[0] {
                dropped[0] = true;
                continue;
            }
            if seq == 5 && !dropped[1] {
                dropped[1] = true;
                continue;
            }
            let result = receiver.receive(&packet).unwrap();
            assert_eq!(result.rejected_buffered, None);
            delivered.extend(result.messages);
            if let Some(ack) = result.acknowledgement {
                let result = sender.acknowledge(&ack.packet());
                assert!(result.is_ok() || result == Err(SendError::InvalidAcknowledgement));
            }
            let duplicate = receiver.receive(&packet).unwrap();
            assert!(duplicate.messages.is_empty());
            if let Some(ack) = duplicate.acknowledgement {
                let result = sender.acknowledge(&ack.packet());
                assert!(result.is_ok() || result == Err(SendError::InvalidAcknowledgement));
            }
        }
        if sender.is_idle() {
            break;
        }
    }
    assert!(dropped.iter().all(|dropped| *dropped));
    assert!(sender.is_idle());
    assert_eq!(delivered, messages);
    assert_eq!(receiver.fragment_bytes(), 0);
    assert_eq!(receiver.buffered_bytes(), 0);
}

#[test]
fn every_wire_packet_fits_limits_even_when_zlib_expands_binary_data() {
    let message = binary(4096);
    for crc_length in 0..=4 {
        for compression in [false, true] {
            for limit in [14, 32, 112, 512] {
                let codec = Codec::new(SEED, crc_length, compression, 1_048_576).unwrap();
                let mut sender = ReliableSender::new(
                    codec.clone(),
                    SenderConfig {
                        datagram_limit: limit,
                        ..config()
                    },
                )
                .unwrap();
                let mut receiver = ReliableReceiver::new(0, 64, 1_048_576, 1_048_576).unwrap();
                sender.queue_message(&message).unwrap();
                let mut delivered = Vec::new();
                let mut first = true;
                while !sender.is_idle() {
                    let output = sender.poll(0).unwrap();
                    assert!(!output.is_empty());
                    for wire in output {
                        assert!(
                            wire.len() <= limit,
                            "CRC {crc_length}, compression {compression}, limit {limit}"
                        );
                        let packet = codec.decode(&wire).unwrap();
                        assert_eq!(packet.opcode, RELIABLE_FRAGMENT);
                        if first {
                            assert_eq!(
                                u32::from_be_bytes(packet.payload[2..6].try_into().unwrap()),
                                message.len() as u32
                            );
                            first = false;
                        }
                        let outcome = receiver.receive(&packet).unwrap();
                        delivered.extend(outcome.messages);
                        sender
                            .acknowledge(&outcome.acknowledgement.unwrap().packet())
                            .unwrap();
                    }
                }
                assert_eq!(delivered, vec![message.clone()]);
            }
        }
    }
}

#[test]
fn retries_are_identical_timed_and_terminal_when_exhausted() {
    let mut sender = ReliableSender::new(codec(), config()).unwrap();
    sender.queue_message(b"retry me").unwrap();
    let first = sender.poll(0).unwrap();
    assert!(sender.poll(99).unwrap().is_empty());
    assert_eq!(sender.poll(100).unwrap(), first);
    assert!(sender.poll(199).unwrap().is_empty());
    assert_eq!(sender.poll(200).unwrap(), first);
    assert_eq!(sender.poll(300), Err(SendError::RetryLimit { sequence: 0 }));
    assert_eq!(sender.poll(400), Err(SendError::SenderFailed));
    assert_eq!(sender.queue_message(b"later"), Err(SendError::SenderFailed));
    assert_eq!(
        sender.acknowledge(&ack(CUMULATIVE_ACK, 0)),
        Err(SendError::SenderFailed)
    );

    let mut sender = ReliableSender::new(
        codec(),
        SenderConfig {
            max_retries: 0,
            ..config()
        },
    )
    .unwrap();
    sender.queue_message(b"once").unwrap();
    sender.poll(0).unwrap();
    assert_eq!(sender.poll(100), Err(SendError::RetryLimit { sequence: 0 }));
}

#[test]
fn queue_limits_and_failed_enqueue_preserve_sequences_and_release_on_ack() {
    let codec = Codec::new(SEED, 3, false, 1_048_576).unwrap();
    let mut sender = ReliableSender::new(
        codec.clone(),
        SenderConfig {
            max_queued_bytes: 30,
            max_message: 1000,
            ..config()
        },
    )
    .unwrap();
    assert_eq!(
        sender.queue_message(&binary(500)),
        Err(SendError::QueueFull)
    );
    assert_eq!(
        sender.queue_message(&binary(1001)),
        Err(SendError::MessageTooLarge)
    );
    assert_eq!(sender.queue_message(&[]), Err(SendError::EmptyMessage));
    assert!(sender.is_idle());
    sender.queue_message(b"0123456789").unwrap();
    assert_eq!(
        sender.queue_message(b"0123456789"),
        Err(SendError::QueueFull)
    );
    let wire = sender.poll(0).unwrap();
    assert_eq!(sequence(&codec.decode(&wire[0]).unwrap()), 0);
    sender.acknowledge(&ack(CUMULATIVE_ACK, 0)).unwrap();
    sender.queue_message(b"0123456789").unwrap();
    let wire = sender.poll(0).unwrap();
    assert_eq!(sequence(&codec.decode(&wire[0]).unwrap()), 1);
    sender.acknowledge(&ack(CUMULATIVE_ACK, 1)).unwrap();
    assert!(sender.is_idle());
}

#[test]
fn wire_sequences_roll_over_without_acknowledging_queued_future_packets() {
    let codec = codec();
    let mut sender = ReliableSender::new(codec.clone(), config()).unwrap();
    for index in 0u32..65_540 {
        sender.queue_message(&index.to_be_bytes()).unwrap();
    }
    let mut index = 0u32;
    while !sender.is_idle() {
        let output = sender.poll(0).unwrap();
        assert!(output.len() <= 32);
        let mut last = 0;
        for wire in output {
            let packet = codec.decode(&wire).unwrap();
            assert_eq!(packet.opcode, RELIABLE_DATA);
            last = sequence(&packet);
            assert_eq!(last, index as u16);
            assert_eq!(&packet.payload[2..], &index.to_be_bytes());
            index += 1;
        }
        assert_eq!(
            sender.acknowledge(&ack(CUMULATIVE_ACK, last.wrapping_add(1))),
            Err(SendError::InvalidAcknowledgement)
        );
        sender.acknowledge(&ack(CUMULATIVE_ACK, last)).unwrap();
    }
    assert_eq!(index, 65_540);
    sender.queue_message(b"next").unwrap();
    let wire = sender.poll(0).unwrap();
    assert_eq!(sequence(&codec.decode(&wire[0]).unwrap()), 4);
}

#[test]
fn malformed_ack_clock_and_configuration_errors_do_not_mutate_sender() {
    assert!(matches!(
        ReliableSender::new(
            codec(),
            SenderConfig {
                window: 0x8000,
                ..config()
            }
        ),
        Err(SendError::InvalidConfiguration)
    ));
    assert!(matches!(
        ReliableSender::new(
            codec(),
            SenderConfig {
                datagram_limit: 12,
                ..config()
            }
        ),
        Err(SendError::InvalidConfiguration)
    ));
    assert!(matches!(
        ReliableSender::new(
            codec(),
            SenderConfig {
                retry_ms: 0,
                ..config()
            }
        ),
        Err(SendError::InvalidConfiguration)
    ));
    let mut sender = ReliableSender::new(codec(), config()).unwrap();
    sender.queue_message(b"clock").unwrap();
    sender.poll(100).unwrap();
    assert_eq!(sender.poll(99), Err(SendError::ClockWentBackwards));
    assert_eq!(
        sender.acknowledge(&Packet {
            opcode: 21,
            payload: vec![0]
        }),
        Err(SendError::InvalidAcknowledgement)
    );
    assert_eq!(
        sender.acknowledge(&Packet {
            opcode: 9,
            payload: vec![0, 0]
        }),
        Err(SendError::InvalidAcknowledgement)
    );
    assert_eq!(sender.pending_packets(), 1);
    assert!(sender.poll(199).unwrap().is_empty());
    sender.acknowledge(&ack(CUMULATIVE_ACK, 0)).unwrap();
    assert!(sender.is_idle());
}
