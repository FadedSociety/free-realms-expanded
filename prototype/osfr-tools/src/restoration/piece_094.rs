//! 094: several-client reliable-transport stress over the production SOE codec.
//!
//! ORIGINAL / EXISTING PRODUCTION facts this harness drives (read-only, all
//! provenance in `notes/Research/Pieces/094 Reliable transport stress.md`):
//! * reliable framing, seeded CRC, compression and the bounded ordered receiver
//!   in `src/soe.rs`; retransmission and the bounded send queue in
//!   `src/soe_send.rs`; recovered handshake seed `session::CRC_SEED`;
//! * session replacement policy in `src/bin/osfr-gateway.rs`: a new session keys
//!   its codec with `CRC_SEED ^ generation` and a negotiated `crc_length == 0` is
//!   refused because a zero trailer cannot isolate stale frames. Both halves of
//!   that policy are exercised here, including the unguarded counterfactuals.
//!
//! RECONSTRUCTED (stress policy only, no invented wire facts): the deterministic
//! loss / duplicate / reorder schedule, the bounded one-way latency model and the
//! retained wire-byte accounting. There are no sockets, timers, gameplay or
//! economy effects, and a passing stress run is not native acceptance.
//!
//! INTEGRATION: the lead registers `pub mod piece_094;` in `src/lib.rs` unchanged;
//! this module only uses `crate::soe`, `crate::soe_send` and `crate::session`.
//! Until then the owned test target declares those three modules by path.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use crate::session::CRC_SEED;
use crate::soe::{Codec, ReceiveDisposition, ReliableReceiver, SELECTIVE_ACK};
pub use crate::soe_send::SendError;
use crate::soe_send::{ReliableSender, SenderConfig};

/// How a replacement session keys its codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPolicy {
    /// Production gateway policy: `CRC_SEED ^ generation` per replacement.
    RotateSeed,
    /// Negative control only; the replacement reuses the previous key.
    ReuseSeed,
}

/// Everything a stress run is allowed to vary. Defaults mirror the gateway's
/// sender/receiver limits where the gateway chooses them (window 32 is relaxed to
/// 16 so a small run reaches window pressure quickly).
#[derive(Debug, Clone)]
pub struct StressConfig {
    pub clients: usize,
    pub crc_len: u8,
    pub session_policy: SessionPolicy,
    pub datagram_limit: usize,
    pub window: usize,
    pub max_message: usize,
    pub max_queued_bytes: usize,
    pub retry_ms: u64,
    pub max_retries: u32,
    pub tick_ms: u64,
    /// One-way latency in ticks before a datagram may be delivered.
    pub latency_ticks: u64,
    pub rng_seed: u64,
    pub loss_percent: u32,
    pub duplicate_percent: u32,
    pub reorder_percent: u32,
    pub ack_loss_percent: u32,
    pub ack_duplicate_percent: u32,
}

impl StressConfig {
    pub fn new(clients: usize) -> Self {
        Self {
            clients,
            crc_len: 3,
            session_policy: SessionPolicy::RotateSeed,
            datagram_limit: 512,
            window: 16,
            max_message: 65_536,
            max_queued_bytes: 8192,
            retry_ms: 25,
            max_retries: 12,
            tick_ms: 10,
            latency_ticks: 1,
            rng_seed: 0x5a71_3f89_2c6d_1e07,
            loss_percent: 20,
            duplicate_percent: 15,
            reorder_percent: 25,
            ack_loss_percent: 10,
            ack_duplicate_percent: 10,
        }
    }

    fn seed_for(&self, session: u32) -> u32 {
        match self.session_policy {
            SessionPolicy::RotateSeed => CRC_SEED ^ session,
            SessionPolicy::ReuseSeed => CRC_SEED,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StressError {
    InvalidConfiguration,
    InvalidClient(usize),
    EmptyMessage,
    MessageTooLarge,
    Sender { client: usize, error: SendError },
    TickLimit { ticks: usize },
}

impl fmt::Display for StressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration => write!(f, "invalid stress configuration"),
            Self::InvalidClient(client) => write!(f, "unknown stress client {client}"),
            Self::EmptyMessage => write!(f, "empty application message"),
            Self::MessageTooLarge => write!(f, "application message exceeds the stress limit"),
            Self::Sender { client, error } => write!(f, "client {client} sender: {error}"),
            Self::TickLimit { ticks } => write!(f, "stress run exceeded {ticks} ticks"),
        }
    }
}

impl std::error::Error for StressError {}

/// Per-client counters. They are observations of the run, not production state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientStats {
    pub session: u32,
    pub queued: usize,
    pub delivered: usize,
    pub sent: usize,
    pub retransmits: usize,
    pub loss: usize,
    pub duplicate: usize,
    pub reordered: usize,
    pub ack_lost: usize,
    pub duplicate_acks: usize,
    pub outside_window: usize,
    pub stale_rejected: usize,
    pub stale_accepted: usize,
    pub corrupt_rejected: usize,
    pub receive_errors: usize,
    pub queue_full: usize,
    pub discarded: usize,
    pub failed: bool,
}

struct Client {
    session: u32,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    backlog: VecDeque<Vec<u8>>,
    delivered: Vec<Vec<u8>>,
    /// Distinct unacknowledged sequences mapped to their retained wire bytes.
    outstanding: BTreeMap<u16, usize>,
    retained_bytes: usize,
    retained_peak: usize,
    backlog_peak: usize,
    retired: bool,
    partitioned: bool,
    failed: Option<SendError>,
    stats: ClientStats,
}

impl Client {
    fn new(config: &StressConfig, session: u32) -> Result<Self, StressError> {
        let seed = config.seed_for(session);
        let codec = Codec::new(
            seed,
            config.crc_len,
            true,
            config.max_message.saturating_add(8),
        )
        .map_err(|_| StressError::InvalidConfiguration)?;
        let sender = ReliableSender::new(
            codec.clone(),
            SenderConfig {
                datagram_limit: config.datagram_limit,
                window: config.window,
                max_message: config.max_message,
                max_queued_bytes: config.max_queued_bytes,
                retry_ms: config.retry_ms,
                max_retries: config.max_retries,
            },
        )
        .map_err(|_| StressError::InvalidConfiguration)?;
        let receiver = ReliableReceiver::new(
            0,
            1024,
            config.max_message,
            config
                .max_queued_bytes
                .saturating_mul(2)
                .max(config.max_message),
        )
        .map_err(|_| StressError::InvalidConfiguration)?;
        Ok(Self {
            session,
            codec,
            sender,
            receiver,
            backlog: VecDeque::new(),
            delivered: Vec::new(),
            outstanding: BTreeMap::new(),
            retained_bytes: 0,
            retained_peak: 0,
            backlog_peak: 0,
            retired: false,
            partitioned: false,
            failed: None,
            stats: ClientStats {
                session,
                ..ClientStats::default()
            },
        })
    }

    fn track(&mut self, sequence: u16, wire_len: usize) {
        if self.outstanding.insert(sequence, wire_len).is_none() {
            self.retained_bytes += wire_len;
            self.retained_peak = self.retained_peak.max(self.retained_bytes);
        }
    }
}

struct Datagram {
    client: usize,
    session: u32,
    wire: Vec<u8>,
    deliver_at: u64,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn chance(&mut self, percent: u32) -> bool {
        percent > 0 && self.next_u64() % 100 < u64::from(percent)
    }
}

/// Several independent link pairs driven through the production codec, sender
/// and receiver. One client means one sender/receiver pair, not a socket.
pub struct TransportStress {
    config: StressConfig,
    rng: Rng,
    clients: Vec<Client>,
    in_flight: Vec<Datagram>,
    now_ms: u64,
    ticks: usize,
}

impl TransportStress {
    pub fn new(config: StressConfig) -> Result<Self, StressError> {
        if config.clients == 0
            || config.window == 0
            || config.window >= 0x8000
            || config.max_message == 0
            || config.max_queued_bytes == 0
            || config.retry_ms == 0
            || config.tick_ms == 0
        {
            return Err(StressError::InvalidConfiguration);
        }
        let mut clients = Vec::with_capacity(config.clients);
        for index in 0..config.clients {
            clients.push(Client::new(&config, index as u32 + 1)?);
        }
        Ok(Self {
            rng: Rng::new(config.rng_seed),
            config,
            clients,
            in_flight: Vec::new(),
            now_ms: 0,
            ticks: 0,
        })
    }

    /// Queue one application message for a client. Unbounded here: the harness
    /// backlog is the caller's queue, and the production send queue is what has
    /// to stay bounded.
    pub fn queue(&mut self, client: usize, message: &[u8]) -> Result<(), StressError> {
        if message.is_empty() {
            return Err(StressError::EmptyMessage);
        }
        if message.len() > self.config.max_message {
            return Err(StressError::MessageTooLarge);
        }
        let link = self
            .clients
            .get_mut(client)
            .ok_or(StressError::InvalidClient(client))?;
        link.backlog.push_back(message.to_vec());
        link.stats.queued += 1;
        link.backlog_peak = link.backlog_peak.max(link.backlog.len());
        Ok(())
    }

    /// Replace a client's session exactly as the gateway does: a new key, a fresh
    /// sender and receiver starting at sequence zero. Frames already in flight
    /// keep their old session tag and are counted as stale when they arrive;
    /// unsent send-queue content is reported as discarded rather than replayed.
    pub fn replace_session(&mut self, client: usize) -> Result<(), StressError> {
        if self
            .clients
            .get(client)
            .ok_or(StressError::InvalidClient(client))?
            .retired
        {
            return Err(StressError::InvalidClient(client));
        }
        let session = self.clients[client].session.wrapping_add(1);
        let replacement = Client::new(&self.config, session)?;
        let link = &mut self.clients[client];
        link.stats.discarded += link.sender.pending_packets();
        link.session = session;
        link.codec = replacement.codec;
        link.sender = replacement.sender;
        link.receiver = replacement.receiver;
        link.outstanding.clear();
        link.retained_bytes = 0;
        link.failed = None;
        link.stats.session = session;
        Ok(())
    }

    pub fn retire_client(&mut self, client: usize) -> Result<(), StressError> {
        let link = self
            .clients
            .get_mut(client)
            .ok_or(StressError::InvalidClient(client))?;
        link.retired = true;
        link.backlog.clear();
        self.in_flight.retain(|datagram| datagram.client != client);
        Ok(())
    }

    pub fn set_partitioned(&mut self, client: usize, partitioned: bool) -> Result<(), StressError> {
        let link = self
            .clients
            .get_mut(client)
            .ok_or(StressError::InvalidClient(client))?;
        link.partitioned = partitioned;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), StressError> {
        self.ticks += 1;
        self.now_ms += self.config.tick_ms;
        let mut reported = None;
        for index in 0..self.clients.len() {
            if let Err(error) = self.pump(index)
                && reported.is_none()
            {
                reported = Some(error);
            }
        }
        self.deliver_due();
        match reported {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub fn run_ticks(&mut self, count: usize) -> Result<(), StressError> {
        for _ in 0..count {
            self.tick()?;
        }
        Ok(())
    }

    pub fn run_until(
        &mut self,
        predicate: impl Fn(&Self) -> bool,
        max_ticks: usize,
    ) -> Result<usize, StressError> {
        for _ in 0..max_ticks {
            if predicate(self) {
                return Ok(self.ticks);
            }
            self.tick()?;
        }
        if predicate(self) {
            return Ok(self.ticks);
        }
        Err(StressError::TickLimit { ticks: max_ticks })
    }

    pub fn run_until_idle(&mut self, max_ticks: usize) -> Result<usize, StressError> {
        self.run_until(|stress| stress.is_idle(), max_ticks)
    }

    pub fn is_idle(&self) -> bool {
        self.in_flight.is_empty()
            && self.clients.iter().all(|link| {
                link.retired
                    || (link.failed.is_none() && link.backlog.is_empty() && link.sender.is_idle())
            })
    }

    pub fn delivered(&self, client: usize) -> &[Vec<u8>] {
        &self.clients[client].delivered
    }

    pub fn stats(&self, client: usize) -> ClientStats {
        self.clients[client].stats
    }

    pub fn backlog(&self, client: usize) -> usize {
        self.clients[client].backlog.len()
    }

    pub fn retained_peak(&self, client: usize) -> usize {
        self.clients[client].retained_peak
    }

    pub fn in_flight_for(&self, client: usize) -> usize {
        self.in_flight
            .iter()
            .filter(|datagram| datagram.client == client)
            .count()
    }

    pub fn in_flight(&self) -> usize {
        self.in_flight.len()
    }

    pub fn ticks(&self) -> usize {
        self.ticks
    }

    /// One machine-readable line per scenario. The piece 094 checker parses these
    /// keys; keep them stable.
    pub fn summary_line(&self, scenario: &str) -> String {
        let sum = |pick: fn(&ClientStats) -> usize| {
            self.clients
                .iter()
                .map(|link| pick(&link.stats))
                .sum::<usize>()
        };
        let peak = |pick: fn(&Client) -> usize| self.clients.iter().map(pick).max().unwrap_or(0);
        format!(
            "piece094 summary scenario={scenario} clients={} ticks={} policy={:?} crc_len={} window={} queue_cap={} \
queued={} delivered={} stale_accepted={} stale_rejected={} corrupt_rejected={} receive_errors={} outside_window={} \
queue_full={} loss={} duplicate={} reordered={} retransmits={} ack_lost={} duplicate_acks={} discarded={} \
retained_peak={} backlog_peak={} in_flight={} failed={}",
            self.clients.len(),
            self.ticks,
            self.config.session_policy,
            self.config.crc_len,
            self.config.window,
            self.config.max_queued_bytes,
            sum(|stats| stats.queued),
            sum(|stats| stats.delivered),
            sum(|stats| stats.stale_accepted),
            sum(|stats| stats.stale_rejected),
            sum(|stats| stats.corrupt_rejected),
            sum(|stats| stats.receive_errors),
            sum(|stats| stats.outside_window),
            sum(|stats| stats.queue_full),
            sum(|stats| stats.loss),
            sum(|stats| stats.duplicate),
            sum(|stats| stats.reordered),
            sum(|stats| stats.retransmits),
            sum(|stats| stats.ack_lost),
            sum(|stats| stats.duplicate_acks),
            sum(|stats| stats.discarded),
            peak(|link| link.retained_peak),
            peak(|link| link.backlog_peak),
            self.in_flight.len(),
            self.clients
                .iter()
                .filter(|link| link.failed.is_some())
                .count(),
        )
    }

    fn pump(&mut self, index: usize) -> Result<(), StressError> {
        let config = self.config.clone();
        let rng = &mut self.rng;
        let in_flight = &mut self.in_flight;
        let now_ms = self.now_ms;
        let link = &mut self.clients[index];
        if link.retired || link.failed.is_some() {
            return Ok(());
        }
        while let Some(message) = link.backlog.front() {
            match link.sender.queue_message(message) {
                Ok(()) => {
                    link.backlog.pop_front();
                }
                Err(SendError::QueueFull) => {
                    link.stats.queue_full += 1;
                    break;
                }
                Err(error) => {
                    link.failed = Some(error.clone());
                    return Err(StressError::Sender {
                        client: index,
                        error,
                    });
                }
            }
        }
        let session = link.session;
        let wires = match link.sender.poll(now_ms) {
            Ok(wires) => wires,
            Err(error) => {
                link.failed = Some(error.clone());
                return Err(StressError::Sender {
                    client: index,
                    error,
                });
            }
        };
        for wire in wires {
            let sequence = match link.codec.decode(&wire) {
                Ok(packet) => u16::from_be_bytes([packet.payload[0], packet.payload[1]]),
                Err(error) => {
                    let error = SendError::Codec(error);
                    link.failed = Some(error.clone());
                    return Err(StressError::Sender {
                        client: index,
                        error,
                    });
                }
            };
            if link.outstanding.contains_key(&sequence) {
                link.stats.retransmits += 1;
            } else {
                link.stats.sent += 1;
            }
            link.track(sequence, wire.len());
            if link.partitioned || rng.chance(config.loss_percent) {
                link.stats.loss += 1;
                continue;
            }
            let copies = if rng.chance(config.duplicate_percent) {
                link.stats.duplicate += 1;
                2
            } else {
                1
            };
            let mut deliver_at = now_ms + config.latency_ticks * config.tick_ms;
            if rng.chance(config.reorder_percent) {
                link.stats.reordered += 1;
                deliver_at += config.tick_ms;
            }
            for _ in 0..copies {
                in_flight.push(Datagram {
                    client: index,
                    session,
                    wire: wire.clone(),
                    deliver_at,
                });
            }
        }
        Ok(())
    }

    fn deliver_due(&mut self) {
        let now_ms = self.now_ms;
        let mut due = Vec::new();
        let mut later = Vec::new();
        for datagram in self.in_flight.drain(..) {
            if datagram.deliver_at <= now_ms {
                due.push(datagram);
            } else {
                later.push(datagram);
            }
        }
        self.in_flight = later;
        for datagram in due {
            self.deliver(datagram);
        }
    }

    fn deliver(&mut self, datagram: Datagram) {
        let config = self.config.clone();
        let rng = &mut self.rng;
        let link = &mut self.clients[datagram.client];
        if link.retired {
            return;
        }
        let packet = match link.codec.decode(&datagram.wire) {
            Ok(packet) => packet,
            Err(_) => {
                if datagram.session == link.session {
                    link.stats.corrupt_rejected += 1;
                } else {
                    link.stats.stale_rejected += 1;
                }
                return;
            }
        };
        if datagram.session != link.session {
            link.stats.stale_accepted += 1;
        }
        match link.receiver.receive(&packet) {
            Ok(outcome) => {
                if outcome.disposition == ReceiveDisposition::OutsideWindow {
                    link.stats.outside_window += 1;
                }
                link.stats.delivered += outcome.messages.len();
                link.delivered.extend(outcome.messages);
                let Some(ack) = outcome.acknowledgement else {
                    return;
                };
                if rng.chance(config.ack_loss_percent) {
                    link.stats.ack_lost += 1;
                    return;
                }
                apply_ack(link, ack);
                if rng.chance(config.ack_duplicate_percent) {
                    apply_ack(link, ack);
                }
            }
            Err(_) => link.stats.receive_errors += 1,
        }
    }
}

/// Mirror of the production ACK handling for the harness's own bookkeeping. The
/// production call is authoritative; the byte accounting only follows a success.
fn apply_ack(link: &mut Client, ack: crate::soe::Ack) {
    match link.sender.acknowledge(&ack.packet()) {
        Ok(()) => {
            if ack.opcode == SELECTIVE_ACK {
                if let Some(bytes) = link.outstanding.remove(&ack.sequence) {
                    link.retained_bytes -= bytes;
                }
            } else {
                let released: Vec<u16> = link
                    .outstanding
                    .keys()
                    .copied()
                    .filter(|sequence| ack.sequence.wrapping_sub(*sequence) < 0x8000)
                    .collect();
                for sequence in released {
                    let bytes = link.outstanding.remove(&sequence).unwrap_or(0);
                    link.retained_bytes -= bytes;
                }
            }
        }
        Err(SendError::InvalidAcknowledgement) => link.stats.duplicate_acks += 1,
        Err(error) => link.failed = Some(error),
    }
}
