//! Bounded channel-zero SOE reliable sending, without sockets or clock access.
//!
//! The caller passes monotonic milliseconds to `poll` and supplies already
//! decoded ACK packets. Opcode 17 selectively acknowledges one sent sequence;
//! opcode 21 acknowledges the contiguous prefix through a sent sequence. This
//! follows pinned Sanctuary's Ack1/AckAll1 behavior; old LibSOE misleadingly
//! names opcode 17 an out-of-order notice. Neither form can release unsent data.

use std::collections::VecDeque;
use std::fmt;

use crate::soe::{
    Codec, Error, Packet, CUMULATIVE_ACK, MAX_LOGICAL_MESSAGE, RELIABLE_DATA, RELIABLE_FRAGMENT,
    SELECTIVE_ACK,
};

#[derive(Debug, Clone, Copy)]
pub struct SenderConfig {
    pub datagram_limit: usize,
    /// Sequence span, including selectively acknowledged holes. Must be <32768.
    pub window: usize,
    pub max_message: usize,
    /// Retained encoded bytes across both unsent and in-flight packets.
    pub max_queued_bytes: usize,
    pub retry_ms: u64,
    /// Retransmissions after the first send. Zero allows only the initial send.
    pub max_retries: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendError {
    InvalidConfiguration,
    EmptyMessage,
    MessageTooLarge,
    QueueFull,
    Codec(Error),
    /// Malformed, stale, duplicate or future ACK. State is unchanged. Callers
    /// may ignore these because duplicated UDP acknowledgements are expected.
    InvalidAcknowledgement,
    ClockWentBackwards,
    RetryLimit {
        sequence: u16,
    },
    SenderFailed,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => write!(f, "invalid reliable sender configuration"),
            Self::EmptyMessage => write!(f, "empty reliable application message"),
            Self::MessageTooLarge => write!(f, "application message exceeds sender limit"),
            Self::QueueFull => write!(f, "reliable send queue byte limit exceeded"),
            Self::Codec(error) => write!(f, "SOE encoding: {error}"),
            Self::InvalidAcknowledgement => write!(f, "ACK does not identify a sent packet"),
            Self::ClockWentBackwards => write!(f, "sender poll time went backwards"),
            Self::RetryLimit { sequence } => {
                write!(f, "retry limit reached for sequence {sequence}")
            }
            Self::SenderFailed => write!(f, "reliable sender failed; a new session is required"),
        }
    }
}

impl std::error::Error for SendError {}

struct Queued {
    sequence: u16,
    wire: Vec<u8>,
}

struct Pending {
    sequence: u16,
    // A selective ACK releases retained bytes but keeps its sequence-span slot
    // until all preceding packets are acknowledged.
    wire: Option<Vec<u8>>,
    sent_at: u64,
    retries: u32,
}

pub struct ReliableSender {
    codec: Codec,
    config: SenderConfig,
    data_capacity: usize,
    next_sequence: u16,
    retained_bytes: usize,
    queued: VecDeque<Queued>,
    pending: VecDeque<Pending>,
    last_poll: Option<u64>,
    failed: bool,
}

impl ReliableSender {
    pub fn new(codec: Codec, config: SenderConfig) -> Result<Self, SendError> {
        if config.datagram_limit > 65_507
            || config.window == 0
            || config.window >= 0x8000
            || config.max_message == 0
            || config.max_queued_bytes == 0
            || config.retry_ms == 0
        {
            return Err(SendError::InvalidConfiguration);
        }
        let empty_wire = codec
            .encode(RELIABLE_DATA, &[0, 0])
            .map_err(SendError::Codec)?;
        // A first fragment needs four length bytes and at least one data byte.
        if config.datagram_limit < empty_wire.len() + 5
            || codec
                .encode(RELIABLE_FRAGMENT, &[0, 0, 0, 0, 0, 1, 1])
                .is_err()
        {
            return Err(SendError::InvalidConfiguration);
        }
        // (R) Keep transmitted logical messages within the gateway parser's cap.
        let config = SenderConfig {
            max_message: config.max_message.min(MAX_LOGICAL_MESSAGE),
            ..config
        };
        Ok(Self {
            data_capacity: config.datagram_limit - empty_wire.len(),
            codec,
            config,
            next_sequence: 0,
            retained_bytes: 0,
            queued: VecDeque::new(),
            pending: VecDeque::new(),
            last_poll: None,
            failed: false,
        })
    }

    /// Queue a logical application message atomically. Sequencing and memory
    /// counters change only after every fragment fits both the wire and queue
    /// limits. Fragment boundaries can differ from legacy LibSOE's 512-byte
    /// slices; their total-length prefix and ordered bytes remain equivalent.
    pub fn queue_message(&mut self, message: &[u8]) -> Result<(), SendError> {
        if self.failed {
            return Err(SendError::SenderFailed);
        }
        if message.is_empty() {
            return Err(SendError::EmptyMessage);
        }
        if message.len() > self.config.max_message || message.len() > u32::MAX as usize {
            return Err(SendError::MessageTooLarge);
        }
        let available = self.config.max_queued_bytes - self.retained_bytes;
        let mut frames = VecDeque::new();
        let mut bytes = 0;
        let mut sequence = self.next_sequence;
        if message.len() <= self.data_capacity {
            let mut payload = sequence.to_be_bytes().to_vec();
            payload.extend_from_slice(message);
            match self.codec.encode(RELIABLE_DATA, &payload) {
                Ok(wire) if wire.len() <= self.config.datagram_limit => {
                    if wire.len() > available {
                        return Err(SendError::QueueFull);
                    }
                    bytes = wire.len();
                    frames.push_back(Queued { sequence, wire });
                    sequence = sequence.wrapping_add(1);
                }
                Ok(_) | Err(Error::PayloadTooLarge | Error::DatagramTooLarge) => {}
                Err(error) => return Err(SendError::Codec(error)),
            }
        }
        if frames.is_empty() {
            let mut offset = 0;
            while offset < message.len() {
                let first = offset == 0;
                let capacity = self.data_capacity - if first { 4 } else { 0 };
                let mut take = capacity.min(message.len() - offset);
                let wire = loop {
                    if take == 0 {
                        return Err(SendError::InvalidConfiguration);
                    }
                    let mut payload = sequence.to_be_bytes().to_vec();
                    if first {
                        payload.extend_from_slice(&(message.len() as u32).to_be_bytes());
                    }
                    payload.extend_from_slice(&message[offset..offset + take]);
                    match self.codec.encode(RELIABLE_FRAGMENT, &payload) {
                        Ok(wire) if wire.len() <= self.config.datagram_limit => break wire,
                        Ok(wire) => {
                            // Zlib can expand incompressible data. Shrink by at
                            // least the observed excess, then verify again.
                            take = take
                                .saturating_sub((wire.len() - self.config.datagram_limit).max(1));
                        }
                        Err(Error::PayloadTooLarge | Error::DatagramTooLarge) => take /= 2,
                        Err(error) => return Err(SendError::Codec(error)),
                    }
                };
                if wire.len() > available - bytes {
                    return Err(SendError::QueueFull);
                }
                bytes += wire.len();
                frames.push_back(Queued { sequence, wire });
                sequence = sequence.wrapping_add(1);
                offset += take;
            }
        }
        self.retained_bytes += bytes;
        self.next_sequence = sequence;
        self.queued.append(&mut frames);
        Ok(())
    }

    /// Return due retries and newly available window packets. Polling does not
    /// assume delivery: returned bytes stay retained until acknowledged. Retry
    /// exhaustion is terminal; create a new sender for a new session.
    pub fn poll(&mut self, now_ms: u64) -> Result<Vec<Vec<u8>>, SendError> {
        if self.failed {
            return Err(SendError::SenderFailed);
        }
        if self.last_poll.is_some_and(|previous| now_ms < previous) {
            return Err(SendError::ClockWentBackwards);
        }
        for packet in &self.pending {
            if packet.wire.is_some()
                && now_ms.saturating_sub(packet.sent_at) >= self.config.retry_ms
                && packet.retries >= self.config.max_retries
            {
                self.failed = true;
                return Err(SendError::RetryLimit {
                    sequence: packet.sequence,
                });
            }
        }
        self.last_poll = Some(now_ms);
        let mut output = Vec::new();
        for packet in &mut self.pending {
            if let Some(wire) = &packet.wire {
                if now_ms.saturating_sub(packet.sent_at) >= self.config.retry_ms {
                    output.push(wire.clone());
                    packet.sent_at = now_ms;
                    packet.retries += 1;
                }
            }
        }
        while self.pending.len() < self.config.window {
            let Some(packet) = self.queued.pop_front() else {
                break;
            };
            output.push(packet.wire.clone());
            self.pending.push_back(Pending {
                sequence: packet.sequence,
                wire: Some(packet.wire),
                sent_at: now_ms,
                retries: 0,
            });
        }
        Ok(output)
    }

    /// ACK input must already have passed the session codec's integrity checks.
    /// Unknown/duplicate/stale/future ACKs return InvalidAcknowledgement without
    /// changing state. They should normally be ignored by the socket owner.
    pub fn acknowledge(&mut self, packet: &Packet) -> Result<(), SendError> {
        if self.failed {
            return Err(SendError::SenderFailed);
        }
        if !matches!(packet.opcode, CUMULATIVE_ACK | SELECTIVE_ACK) || packet.payload.len() != 2 {
            return Err(SendError::InvalidAcknowledgement);
        }
        let sequence = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
        let Some(position) = self
            .pending
            .iter()
            .position(|pending| pending.sequence == sequence)
        else {
            return Err(SendError::InvalidAcknowledgement);
        };
        if packet.opcode == SELECTIVE_ACK {
            let Some(wire) = self.pending[position].wire.take() else {
                return Err(SendError::InvalidAcknowledgement);
            };
            self.retained_bytes -= wire.len();
        } else {
            for _ in 0..=position {
                if let Some(wire) = self.pending.pop_front().unwrap().wire {
                    self.retained_bytes -= wire.len();
                }
            }
        }
        while self
            .pending
            .front()
            .is_some_and(|packet| packet.wire.is_none())
        {
            self.pending.pop_front();
        }
        Ok(())
    }

    /// Includes unsent packets and sent packets still awaiting acknowledgement.
    pub fn pending_packets(&self) -> usize {
        self.queued.len()
            + self
                .pending
                .iter()
                .filter(|packet| packet.wire.is_some())
                .count()
    }

    pub fn is_idle(&self) -> bool {
        self.pending_packets() == 0
    }
}
