//! Bounded legacy SOE framing and ordered receive state for reliable channel 0.
//!
//! Wire evidence comes from the supplied LibSOE serializer. This module handles
//! post-session, unencrypted transport packets only. Handshake exceptions,
//! application/multi-message parsing, sockets, timers and sending reliability
//! belong to the caller. A successful decode does not establish a game session.

use std::collections::BTreeMap;
use std::fmt;
use std::time::{Duration, Instant};

use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::stream::{inflate, InflateState};
use miniz_oxide::{DataFormat, MZFlush, MZStatus};

pub const RELIABLE_DATA: u16 = 9;
pub const RELIABLE_FRAGMENT: u16 = 13;
pub const SELECTIVE_ACK: u16 = 17;
pub const CUMULATIVE_ACK: u16 = 21;
const MAX_DATAGRAM: usize = 65_507;
/// (R) Match the gateway parser's existing 1 MiB inbound-message limit.
pub const MAX_LOGICAL_MESSAGE: usize = 1_048_576;
// (R) Keep one receiver's out-of-order payload budget at the gateway's current cap.
const MAX_REORDER_BUFFERED: usize = 2 * 1024 * 1024;
// (R) Bound an incomplete logical message to 30 seconds; a reliable fragment has
// no independent continuation marker, so expiry makes this receiver unusable.
const REASSEMBLY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidConfiguration,
    Truncated,
    BadCrc,
    InvalidCompressionFlag(u8),
    InvalidZlib,
    PayloadTooLarge,
    DatagramTooLarge,
    UnsupportedOpcode(u16),
    InvalidFragment,
    UnexpectedDataDuringFragment,
    AmbiguousSequence,
    BufferLimit,
    ConflictingDuplicate,
    ReceiverPoisoned,
    InvalidDirectPose,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => write!(f, "invalid SOE limits or CRC length"),
            Self::Truncated => write!(f, "truncated SOE packet"),
            Self::BadCrc => write!(f, "SOE checksum mismatch"),
            Self::InvalidCompressionFlag(flag) => write!(f, "invalid compression flag {flag}"),
            Self::InvalidZlib => write!(f, "invalid, truncated or trailing zlib data"),
            Self::PayloadTooLarge => write!(f, "SOE payload exceeds configured limit"),
            Self::DatagramTooLarge => write!(f, "SOE datagram exceeds UDP size limit"),
            Self::UnsupportedOpcode(opcode) => write!(f, "unsupported reliable opcode {opcode}"),
            Self::InvalidFragment => write!(f, "invalid fragment length or fragment overflow"),
            Self::UnexpectedDataDuringFragment => {
                write!(f, "data interrupted a fragmented message")
            }
            Self::AmbiguousSequence => write!(f, "sequence is exactly half a rollover away"),
            Self::BufferLimit => write!(f, "reliable receive buffer limit exceeded"),
            Self::ConflictingDuplicate => write!(f, "different packets share a buffered sequence"),
            Self::ReceiverPoisoned => {
                write!(f, "reliable receiver failed; a new session is required")
            }
            Self::InvalidDirectPose => write!(f, "invalid direct kart pose envelope"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u16,
    /// Bytes after opcode, decompression and CRC removal. Reliable packets retain
    /// their two-byte sequence and initial fragments retain their length header.
    pub payload: Vec<u8>,
}

/// A negotiated post-session envelope. Compression flag bytes are present only
/// when `compression_enabled` is true. CRC length zero is supported explicitly.
/// The requested expanded-payload ceiling is additionally capped at
/// `MAX_LOGICAL_MESSAGE`.
#[derive(Debug, Clone)]
pub struct Codec {
    seed: u32,
    crc_len: usize,
    compression_enabled: bool,
    max_payload: usize,
}

impl Codec {
    pub fn new(
        seed: u32,
        crc_len: u8,
        compression_enabled: bool,
        max_payload: usize,
    ) -> Result<Self, Error> {
        if crc_len > 4 || max_payload == 0 {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            seed,
            crc_len: crc_len.into(),
            compression_enabled,
            max_payload: max_payload.min(MAX_LOGICAL_MESSAGE),
        })
    }

    /// Encode with the supplied serializer's threshold: payloads over 100 bytes
    /// receive flag 1 and zlib encoding; smaller payloads receive flag 0. Zlib
    /// output bytes may differ by compressor while representing the same data.
    pub fn encode(&self, opcode: u16, payload: &[u8]) -> Result<Vec<u8>, Error> {
        if payload.len() > self.max_payload {
            return Err(Error::PayloadTooLarge);
        }
        let mut wire = Vec::with_capacity(payload.len().min(MAX_DATAGRAM) + 7);
        wire.extend_from_slice(&opcode.to_be_bytes());
        if self.compression_enabled {
            if payload.len() > 100 {
                wire.push(1);
                wire.extend_from_slice(&compress_to_vec_zlib(payload, 6));
            } else {
                wire.push(0);
                wire.extend_from_slice(payload);
            }
        } else {
            wire.extend_from_slice(payload);
        }
        if wire.len() + self.crc_len > MAX_DATAGRAM {
            return Err(Error::DatagramTooLarge);
        }
        let checksum = seeded_crc32(self.seed, &wire).to_be_bytes();
        wire.extend_from_slice(&checksum[4 - self.crc_len..]);
        Ok(wire)
    }

    /// Verify the transmitted frame before allocating decompressed output.
    /// Both expanded bytes and wire size are bounded. Concatenated streams or
    /// bytes after the zlib stream are rejected rather than silently discarded.
    pub fn decode(&self, wire: &[u8]) -> Result<Packet, Error> {
        if wire.len() > MAX_DATAGRAM {
            return Err(Error::DatagramTooLarge);
        }
        let header_len = 2 + usize::from(self.compression_enabled);
        if wire.len() < header_len + self.crc_len {
            return Err(Error::Truncated);
        }
        let content_len = wire.len() - self.crc_len;
        let (content, trailer) = wire.split_at(content_len);
        let checksum = seeded_crc32(self.seed, content).to_be_bytes();
        if trailer != &checksum[4 - self.crc_len..] {
            return Err(Error::BadCrc);
        }
        let opcode = u16::from_be_bytes([content[0], content[1]]);
        let encoded = &content[header_len..];
        let payload = if self.compression_enabled {
            match content[2] {
                0 => self.copy_bounded(encoded)?,
                1 => self.inflate_bounded(encoded)?,
                flag => return Err(Error::InvalidCompressionFlag(flag)),
            }
        } else {
            self.copy_bounded(encoded)?
        };
        Ok(Packet { opcode, payload })
    }

    /// The native kart sends poses using a one-byte application opcode before
    /// the negotiated compression flag, unlike two-byte SOE controls. Admit
    /// only the recovered 96-byte C33/6 tunnel; all actions stay reliable.
    pub fn decode_kart_pose_datagram(&self, wire: &[u8]) -> Result<Vec<u8>, Error> {
        if wire.len() > MAX_DATAGRAM {
            return Err(Error::DatagramTooLarge);
        }
        let header_len = 1 + usize::from(self.compression_enabled);
        if wire.len() < header_len + self.crc_len {
            return Err(Error::Truncated);
        }
        let (content, trailer) = wire.split_at(wire.len() - self.crc_len);
        let checksum = seeded_crc32(self.seed, content).to_be_bytes();
        if trailer != &checksum[4 - self.crc_len..] {
            return Err(Error::BadCrc);
        }
        if content[0] != 5 {
            return Err(Error::InvalidDirectPose);
        }
        // Narrow expanded bound, including for highly compressed forged data.
        let bounded = Self {
            max_payload: self.max_payload.min(95),
            ..self.clone()
        };
        let encoded = &content[header_len..];
        let payload = if self.compression_enabled {
            match content[1] {
                0 => bounded.copy_bounded(encoded)?,
                1 => bounded.inflate_bounded(encoded)?,
                flag => return Err(Error::InvalidCompressionFlag(flag)),
            }
        } else {
            bounded.copy_bounded(encoded)?
        };
        if payload.len() != 95
            || payload[0] != 0
            || !matches!(payload[1], 0 | 1)
            || payload[2..6] != 89u32.to_le_bytes()
            || payload[6..10] != [33, 0, 6, 0]
        {
            return Err(Error::InvalidDirectPose);
        }
        let mut application = Vec::with_capacity(96);
        application.push(5);
        application.extend(payload);
        Ok(application)
    }

    fn copy_bounded(&self, bytes: &[u8]) -> Result<Vec<u8>, Error> {
        if bytes.len() > self.max_payload {
            return Err(Error::PayloadTooLarge);
        }
        Ok(bytes.to_vec())
    }

    fn inflate_bounded(&self, encoded: &[u8]) -> Result<Vec<u8>, Error> {
        let mut state = InflateState::new_boxed(DataFormat::Zlib);
        let mut input = encoded;
        let mut output = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let result = inflate(&mut state, input, &mut chunk, MZFlush::None);
            if result.bytes_consumed > input.len() || result.bytes_written > chunk.len() {
                return Err(Error::InvalidZlib);
            }
            input = &input[result.bytes_consumed..];
            if result.bytes_written > self.max_payload - output.len() {
                return Err(Error::PayloadTooLarge);
            }
            output.extend_from_slice(&chunk[..result.bytes_written]);
            match result.status {
                Ok(MZStatus::StreamEnd) if input.is_empty() => return Ok(output),
                Ok(MZStatus::Ok) if result.bytes_consumed != 0 || result.bytes_written != 0 => {}
                _ => return Err(Error::InvalidZlib),
            }
        }
    }
}

/// The legacy CRC is ordinary reflected CRC-32/ISO-HDLC over the seed's four
/// little-endian bytes followed by the actual transmitted envelope. The trailer
/// uses the low negotiated number of checksum bytes in big-endian order.
pub fn seeded_crc32(seed: u32, packet: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in seed.to_le_bytes().iter().chain(packet) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

/// A Group (0x19) child's length prefix, as the client reads it (GetVariableValue `0x010daea0`;
/// writer PutVariableValue `0x010dae40`): one byte below `FF`, else `FF hi lo` big-endian, and
/// only `FF FF FF` selects a big-endian u32. Returns (value, prefix bytes); None if truncated.
pub fn variable_length(bytes: &[u8]) -> Option<(usize, usize)> {
    match *bytes {
        [0xff, 0xff, 0xff, a, b, c, d, ..] => Some((u32::from_be_bytes([a, b, c, d]) as usize, 7)),
        [0xff, 0xff, 0xff, ..] => None,
        [0xff, hi, lo, ..] => Some((usize::from(u16::from_be_bytes([hi, lo])), 3)),
        [first, ..] if first != 0xff => Some((usize::from(first), 1)),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ack {
    /// 17 acknowledges one buffered packet; 21 acknowledges the contiguous run.
    /// Legacy LibSOE calls 17 "out of order"; Sanctuary names it selective Ack1.
    pub opcode: u16,
    pub sequence: u16,
}

impl Ack {
    pub fn packet(self) -> Packet {
        Packet {
            opcode: self.opcode,
            payload: self.sequence.to_be_bytes().to_vec(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveDisposition {
    Accepted,
    Buffered,
    Duplicate,
    OutsideWindow,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReceiveOutcome {
    /// Logical messages in sequence order, without sequence/fragment headers.
    /// The caller still interprets any application multi-message header.
    pub messages: Vec<Vec<u8>>,
    pub acknowledgement: Option<Ack>,
    pub disposition: ReceiveDisposition,
    /// A queued future packet can only be fully checked once its predecessor
    /// arrives. This reports that failure while preserving earlier deliveries.
    /// The rejected sequence is not advanced or acknowledged cumulatively, and
    /// the receiver rejects subsequent calls because ordered state is unusable.
    pub rejected_buffered: Option<(u16, Error)>,
}

struct PartialMessage {
    expected_length: usize,
    bytes: Vec<u8>,
    started_at: Instant,
}

/// Ordered reliable channel 0, with bounded reordering and fragment assembly.
/// The sequence window must be below half the u16 range. There is one active
/// fragmented message; ordinary data cannot interrupt it. Sending, resend
/// timers, channels 1–3 and stale-session rejection remain caller concerns.
pub struct ReliableReceiver {
    next: u16,
    window: u16,
    max_message: usize,
    max_buffered: usize,
    buffered_bytes: usize,
    buffered: BTreeMap<u16, Packet>,
    partial: Option<PartialMessage>,
    accepted_any: bool,
    poisoned: bool,
}

impl ReliableReceiver {
    /// Construct a receiver with per-instance limits. Larger requested logical
    /// and reorder-buffer limits are capped at the transport's 1 MiB and 2 MiB
    /// ceilings respectively.
    pub fn new(
        next_sequence: u16,
        window: u16,
        max_message: usize,
        max_buffered: usize,
    ) -> Result<Self, Error> {
        if window == 0 || window >= 0x8000 || max_message == 0 {
            return Err(Error::InvalidConfiguration);
        }
        Ok(Self {
            next: next_sequence,
            window,
            max_message: max_message.min(MAX_LOGICAL_MESSAGE),
            max_buffered: max_buffered.min(MAX_REORDER_BUFFERED),
            buffered_bytes: 0,
            buffered: BTreeMap::new(),
            partial: None,
            accepted_any: false,
            poisoned: false,
        })
    }

    pub fn next_sequence(&self) -> u16 {
        self.next
    }
    pub fn buffered_bytes(&self) -> usize {
        self.buffered_bytes
    }
    pub fn fragment_bytes(&self) -> usize {
        self.partial
            .as_ref()
            .map_or(0, |partial| partial.bytes.len())
    }

    /// A direct `Err` leaves sequence, fragment and queue state unchanged unless
    /// it first expires a timed-out fragment. Future fragments are buffered but
    /// not selectively acknowledged because their role cannot be validated
    /// until missing predecessors arrive.
    pub fn receive(&mut self, packet: &Packet) -> Result<ReceiveOutcome, Error> {
        self.receive_at(packet, Instant::now())
    }

    /// Process one reliable packet against a caller-supplied monotonic time.
    /// The gateway should also call `expire_reassembly` when its deadline fires,
    /// including while processing KeepAlive packets.
    pub fn receive_at(&mut self, packet: &Packet, now: Instant) -> Result<ReceiveOutcome, Error> {
        self.expire_reassembly(now);
        if self.poisoned {
            return Err(Error::ReceiverPoisoned);
        }
        if !matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT) {
            return Err(Error::UnsupportedOpcode(packet.opcode));
        }
        if packet.payload.len() < 3 {
            return Err(Error::Truncated);
        }
        let sequence = u16::from_be_bytes([packet.payload[0], packet.payload[1]]);
        let distance = sequence.wrapping_sub(self.next);
        let mut result = ReceiveOutcome {
            messages: Vec::new(),
            acknowledgement: None,
            disposition: ReceiveDisposition::Accepted,
            rejected_buffered: None,
        };
        if distance == 0x8000 {
            return Err(Error::AmbiguousSequence);
        }
        if distance > 0x8000 {
            result.disposition = ReceiveDisposition::Duplicate;
            result.acknowledgement = self.contiguous_ack();
            return Ok(result);
        }
        if distance >= self.window {
            result.disposition = ReceiveDisposition::OutsideWindow;
            return Ok(result);
        }
        if distance != 0 {
            // Data needs only its two-byte sequence. A possible first fragment
            // additionally needs a four-byte logical-message length. Do not
            // selectively acknowledge data that can never pass ordered bounds.
            let header_length = if packet.opcode == RELIABLE_DATA { 2 } else { 6 };
            if packet.payload.len() > self.max_message.saturating_add(header_length) {
                return Err(Error::PayloadTooLarge);
            }
            if packet.opcode == RELIABLE_DATA {
                result.acknowledgement = Some(Ack {
                    opcode: SELECTIVE_ACK,
                    sequence,
                });
            }
            if let Some(previous) = self.buffered.get(&sequence) {
                if previous != packet {
                    return Err(Error::ConflictingDuplicate);
                }
                result.disposition = ReceiveDisposition::Duplicate;
            } else {
                if packet.payload.len() > self.max_buffered.saturating_sub(self.buffered_bytes) {
                    return Err(Error::BufferLimit);
                }
                self.buffered_bytes += packet.payload.len();
                self.buffered.insert(sequence, packet.clone());
                result.disposition = ReceiveDisposition::Buffered;
            }
            return Ok(result);
        }

        if let Some(message) = self.process_ordered(packet, now)? {
            result.messages.push(message);
        }
        self.advance();
        while let Some(queued) = self.buffered.remove(&self.next) {
            self.buffered_bytes -= queued.payload.len();
            match self.process_ordered(&queued, now) {
                Ok(Some(message)) => result.messages.push(message),
                Ok(None) => {}
                Err(error) => {
                    result.rejected_buffered = Some((self.next, error));
                    self.poisoned = true;
                    break;
                }
            }
            self.advance();
        }
        result.acknowledgement = self.contiguous_ack();
        Ok(result)
    }

    /// Expire an incomplete message at its fixed deadline. KeepAlive and later
    /// fragments never extend it. The deadline must be polled by the session
    /// owner even when no reliable packet arrives.
    pub fn expire_reassembly(&mut self, now: Instant) -> bool {
        let expired = self.partial.as_ref().is_some_and(|partial| {
            now.saturating_duration_since(partial.started_at) >= REASSEMBLY_TIMEOUT
        });
        if expired {
            self.partial = None;
            self.buffered.clear();
            self.buffered_bytes = 0;
            self.poisoned = true;
        }
        expired
    }

    /// Absolute monotonic deadline for the event loop's next timer wakeup.
    pub fn reassembly_deadline(&self) -> Option<Instant> {
        self.partial
            .as_ref()
            .map(|partial| partial.started_at + REASSEMBLY_TIMEOUT)
    }

    fn advance(&mut self) {
        self.next = self.next.wrapping_add(1);
        self.accepted_any = true;
    }

    fn contiguous_ack(&self) -> Option<Ack> {
        self.accepted_any.then(|| Ack {
            opcode: CUMULATIVE_ACK,
            sequence: self.next.wrapping_sub(1),
        })
    }

    fn process_ordered(&mut self, packet: &Packet, now: Instant) -> Result<Option<Vec<u8>>, Error> {
        let bytes = &packet.payload[2..];
        if packet.opcode == RELIABLE_DATA {
            if self.partial.is_some() {
                return Err(Error::UnexpectedDataDuringFragment);
            }
            if bytes.len() > self.max_message {
                return Err(Error::PayloadTooLarge);
            }
            return Ok(Some(bytes.to_vec()));
        }

        if let Some(partial) = self.partial.as_mut() {
            if bytes.len() > partial.expected_length.saturating_sub(partial.bytes.len()) {
                return Err(Error::InvalidFragment);
            }
            partial.bytes.extend_from_slice(bytes);
            if partial.bytes.len() == partial.expected_length {
                return self
                    .partial
                    .take()
                    .map(|complete| Some(complete.bytes))
                    .ok_or(Error::InvalidFragment);
            }
            return Ok(None);
        }
        if bytes.len() < 5 {
            return Err(Error::InvalidFragment);
        }
        let expected_length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let fragment = &bytes[4..];
        if expected_length == 0 || fragment.len() > expected_length {
            return Err(Error::InvalidFragment);
        }
        if expected_length > self.max_message {
            return Err(Error::PayloadTooLarge);
        }
        if fragment.len() == expected_length {
            return Ok(Some(fragment.to_vec()));
        }
        self.partial = Some(PartialMessage {
            expected_length,
            bytes: fragment.to_vec(),
            started_at: now,
        });
        Ok(None)
    }
}
