//! Headless bot client: speaks the SOE UDP protocol to a local osfr-gateway,
//! logs in like the native client (login, 13 ready, 10 loaded) and plays a
//! scripted flow. One process is one character; run several for multiplayer.
//!
//! Script steps (one per line, `#` comments allowed, or repeated `--cmd`):
//!   chat <text>                 15/1 local chat (e.g. `chat realm goto Troll Summoner`)
//!   wait <ms>                   keep the connection serviced for <ms>
//!   move <x> <y> <z> [dx dy dz] 125 position update (direction defaults to 0 0 1)
//!   attack                      36/10 basic ability: bar 1, slot 0, target 0, guid 1
//!   ability <slot> [guid]       36/10 on bar 1 with target type 0 (guid defaults to 1)
//!   raw <hex>                   send an arbitrary tunnelled (outer 5) inner message
//!   expect <family> [sub] [ms]  wait for a server message; fails after the timeout
//!   expect-text <text>          wait for a server message whose bytes contain <text>
//!   quit                        disconnect and stop
//! `expect` looks at messages received since the latest sending step.
use ed25519_dalek::{Signer, SigningKey};
use osfr_tools::{
    account_store::AccountStore,
    chat, login,
    login_auth::{login_handle_hash, signing_message},
    session::decode_controls,
    soe::{
        CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SenderConfig},
};
use std::{
    collections::VecDeque,
    fs,
    io::{Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const MAX_MESSAGE: usize = 16 * 1024 * 1024;
const DEFAULT_VERSION: &str = "1.910.1.530630";
const KEEPALIVE: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Chat(String),
    Wait(u64),
    Move([f32; 3], [f32; 3]),
    Ability {
        slot: i32,
        guid: u64,
    },
    Raw(Vec<u8>),
    Expect {
        family: u16,
        sub: Option<u16>,
        timeout_ms: Option<u64>,
    },
    ExpectText(String),
    Quit,
}

pub fn parse_step(line: &str) -> Result<Option<Step>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let (verb, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    let args: Vec<&str> = rest.split_whitespace().collect();
    let num = |text: &str| -> Result<f64, String> {
        text.parse::<f64>()
            .map_err(|_| format!("`{line}`: `{text}` is not a number"))
    };
    let step = match verb.to_ascii_lowercase().as_str() {
        "chat" | "say" if !rest.is_empty() => Step::Chat(rest.to_owned()),
        "wait" if args.len() == 1 => Step::Wait(num(args[0])? as u64),
        "move" if args.len() == 3 || args.len() == 6 => {
            let v = args
                .iter()
                .map(|a| num(a).map(|v| v as f32))
                .collect::<Result<Vec<_>, _>>()?;
            let dir = if v.len() == 6 {
                [v[3], v[4], v[5]]
            } else {
                [0.0, 0.0, 1.0]
            };
            Step::Move([v[0], v[1], v[2]], dir)
        }
        "attack" if args.is_empty() => Step::Ability { slot: 0, guid: 1 },
        "ability" if (1..=2).contains(&args.len()) => Step::Ability {
            slot: num(args[0])? as i32,
            guid: args
                .get(1)
                .map(|g| g.parse())
                .transpose()
                .map_err(|_| format!("`{line}`: bad guid"))?
                .unwrap_or(1),
        },
        "raw" if !rest.is_empty() => {
            let hex: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
            if hex.len() % 2 != 0 {
                return Err(format!("`{line}`: odd hex length"));
            }
            Step::Raw(
                (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
                    .collect::<Result<_, _>>()
                    .map_err(|_| format!("`{line}`: bad hex"))?,
            )
        }
        "expect" if (1..=3).contains(&args.len()) => Step::Expect {
            family: num(args[0])? as u16,
            sub: args.get(1).map(|a| num(a).map(|v| v as u16)).transpose()?,
            timeout_ms: args.get(2).map(|a| num(a).map(|v| v as u64)).transpose()?,
        },
        "expect-text" if !rest.is_empty() => Step::ExpectText(rest.to_owned()),
        "quit" | "exit" => Step::Quit,
        _ => return Err(format!("unrecognised script step `{line}`")),
    };
    Ok(Some(step))
}

pub fn parse_script(text: &str) -> Result<Vec<Step>, String> {
    let mut steps = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if let Some(step) = parse_step(line).map_err(|e| format!("line {}: {e}", index + 1))? {
            steps.push(step);
        }
    }
    Ok(steps)
}

pub struct BotConfig {
    pub gateway: SocketAddr,
    pub name: String,
    pub connection_id: u32,
    pub guid: u64,
    pub ticket: String,
    pub version: String,
    pub expect_timeout_ms: u64,
    pub quiet: bool,
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value.as_bytes());
}

/// Gateway tunnel framing: outer opcode 5, flag 1, u32 length, inner bytes.
pub fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut out = vec![5, 0, 1];
    out.extend((inner.len() as u32).to_le_bytes());
    out.extend(inner);
    out
}

pub fn login_message(guid: u64, ticket: &str, version: &str) -> Vec<u8> {
    let mut out = 1u16.to_le_bytes().to_vec();
    put_string(&mut out, ticket);
    out.extend(guid.to_le_bytes());
    put_string(&mut out, version);
    put_string(&mut out, "");
    out
}

pub fn ability_request(slot: i32, guid: u64) -> Vec<u8> {
    let mut out = vec![36, 0, 10, 0];
    out.extend(1i32.to_le_bytes()); // bar
    out.extend(slot.to_le_bytes());
    out.extend(0i32.to_le_bytes()); // target type: entity
    out.extend(guid.to_le_bytes());
    out
}

pub fn move_request(guid: u64, position: [f32; 3], direction: [f32; 3]) -> Vec<u8> {
    let mut out = 125u16.to_le_bytes().to_vec();
    out.extend(guid.to_le_bytes());
    for value in position.into_iter().chain(direction) {
        out.extend(value.to_le_bytes());
    }
    out.extend([0, 0]); // movement state, unknown
    out
}

/// Split an application-level 0x0019 bundle, then strip the outer-5 tunnel.
fn logical_messages(message: Vec<u8>, out: &mut Vec<Vec<u8>>) {
    if message.len() < 2 || message[..2] != [0, 25] {
        let tunnelled = message.len() >= 7
            && message[..3] == [5, 0, 1]
            && u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize == message.len() - 7;
        out.push(if tunnelled {
            message[7..].to_vec()
        } else {
            message
        });
        return;
    }
    let mut at = 2;
    let take = |at: &mut usize, n: usize| -> Option<&[u8]> {
        let bytes = message.get(*at..*at + n)?;
        *at += n;
        Some(bytes)
    };
    while at < message.len() {
        let Some(&[first]) = take(&mut at, 1) else {
            return;
        };
        let mut size = first as usize;
        if size == 255 {
            let Some(b) = take(&mut at, 2) else { return };
            size = u16::from_be_bytes([b[0], b[1]]) as usize;
            if size == 255 {
                let Some(b) = take(&mut at, 4) else { return };
                size = u32::from_be_bytes(b.try_into().unwrap()) as usize;
            }
        }
        let Some(child) = take(&mut at, size) else {
            return;
        };
        logical_messages(child.to_vec(), out);
    }
}

fn family_of(inner: &[u8]) -> (u16, Option<u16>) {
    let family = inner
        .get(..2)
        .map_or(0, |b| u16::from_le_bytes([b[0], b[1]]));
    (
        family,
        inner.get(2..4).map(|b| u16::from_le_bytes([b[0], b[1]])),
    )
}

fn matches_family(inner: &[u8], family: u16, sub: Option<u16>) -> bool {
    let (f, s) = family_of(inner);
    f == family
        && sub
            .is_none_or(|sub| s == Some(sub) || inner.get(2).is_some_and(|&b| u16::from(b) == sub))
}

pub struct Bot {
    config: BotConfig,
    socket: UdpSocket,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    started: Instant,
    last_sent: Instant,
    inbox: VecDeque<Vec<u8>>,
    pub received: Vec<(u16, Option<u16>, usize)>,
    /// Set by ClientBeginZoning (31); the gateway waits for ready/loaded again.
    rezone: bool,
    in_world: bool,
    login_crypto: Option<(login::Rc4, login::Rc4)>,
}

impl Bot {
    pub fn connect(config: BotConfig) -> Result<Self, String> {
        Self::connect_protocol(config, None)
    }

    fn connect_protocol(config: BotConfig, key: Option<&[u8]>) -> Result<Self, String> {
        let socket = UdpSocket::bind(if config.gateway.is_ipv6() {
            "[::1]:0"
        } else {
            "127.0.0.1:0"
        })
        .map_err(|e| format!("bind: {e}"))?;
        socket
            .set_read_timeout(Some(Duration::from_millis(10)))
            .map_err(|e| e.to_string())?;
        let mut request = vec![0, 1];
        request.extend(if key.is_some() { 2u32 } else { 3 }.to_be_bytes());
        request.extend(config.connection_id.to_be_bytes());
        request.extend(512u32.to_be_bytes()); // buffer size
        request.extend(if key.is_some() {
            b"LoginUdp_6\0".as_slice()
        } else {
            b"CGAPI_527\0"
        });
        let mut response = [0u8; 64];
        let mut handshake = None;
        for _ in 0..10 {
            socket
                .send_to(&request, config.gateway)
                .map_err(|e| format!("send: {e}"))?;
            let deadline = Instant::now() + Duration::from_millis(500);
            while Instant::now() < deadline {
                if let Ok((length, peer)) = socket.recv_from(&mut response) {
                    // Native accept handler 010e3dd0 installs the seed only for
                    // the pending connection id. The core response is 17 bytes;
                    // the gateway also sends the optional four-byte version.
                    // Reject encryption: this bot implements no encrypted framing.
                    if peer == config.gateway
                        && matches!(length, 17 | 21)
                        && response[..2] == [0, 2]
                        && response[2..6] == config.connection_id.to_be_bytes()
                        && response[10] <= 4
                        && response[11] <= 1
                        && response[12] == 0
                    {
                        handshake = Some(length);
                        break;
                    }
                }
            }
            if handshake.is_some() {
                break;
            }
        }
        socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        handshake.ok_or("no SessionResponse from gateway (is it running? only one session without --account-store)")?;
        let seed = u32::from_be_bytes(response[6..10].try_into().unwrap());
        let crc_length = response[10];
        let compression = response[11] != 0;
        let max_message = if key.is_some() {
            login::MAX_LOGIN_MESSAGE + 1
        } else {
            MAX_MESSAGE
        };
        let codec =
            Codec::new(seed, crc_length, compression, max_message).map_err(|e| e.to_string())?;
        let sender = ReliableSender::new(
            codec.clone(),
            SenderConfig {
                datagram_limit: 512,
                window: 32,
                max_message,
                max_queued_bytes: 32 * 1024 * 1024,
                retry_ms: 250,
                max_retries: 40,
            },
        )
        .map_err(|e| e.to_string())?;
        let bot = Self {
            receiver: ReliableReceiver::new(0, 1024, max_message, 2 * 1024 * 1024)
                .map_err(|e| e.to_string())?,
            config,
            socket,
            codec,
            sender,
            started: Instant::now(),
            last_sent: Instant::now(),
            inbox: VecDeque::new(),
            received: Vec::new(),
            rezone: false,
            in_world: false,
            login_crypto: key
                .map(|key| {
                    Ok::<_, String>((
                        login::Rc4::new(key).map_err(|_| "invalid login transport key")?,
                        login::Rc4::new(key).map_err(|_| "invalid login transport key")?,
                    ))
                })
                .transpose()?,
        };
        bot.log(&format!("connected seed={seed:08x} crc={crc_length}"));
        Ok(bot)
    }

    fn log(&self, line: &str) {
        println!(
            "[{} {:>6}ms] {line}",
            self.config.name,
            self.started.elapsed().as_millis()
        );
    }

    fn send_raw(&mut self, opcode: u16, payload: &[u8]) -> Result<(), String> {
        let wire = self
            .codec
            .encode(opcode, payload)
            .map_err(|e| e.to_string())?;
        self.socket
            .send_to(&wire, self.config.gateway)
            .map_err(|e| format!("send: {e}"))?;
        self.last_sent = Instant::now();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let now = self.started.elapsed().as_millis() as u64;
        for wire in self
            .sender
            .poll(now)
            .map_err(|e| format!("reliable send: {e}"))?
        {
            self.socket
                .send_to(&wire, self.config.gateway)
                .map_err(|e| format!("send: {e}"))?;
            self.last_sent = Instant::now();
        }
        if self.last_sent.elapsed() > KEEPALIVE {
            self.send_raw(6, &[])?;
        }
        Ok(())
    }

    /// Queue one reliable application message; clears the expect inbox.
    pub fn send_message(&mut self, message: &[u8], what: &str) -> Result<(), String> {
        self.inbox.clear();
        self.log(&format!("-> {what} len={}", message.len()));
        let mut framed = Vec::new();
        if let Some((_, tx)) = &mut self.login_crypto {
            framed.extend_from_slice(message);
            tx.apply(&mut framed);
            framed = login::frame_ciphertext(&framed).map_err(|_| "invalid login frame")?;
        }
        self.sender
            .queue_message(if self.login_crypto.is_some() {
                &framed
            } else {
                message
            })
            .map_err(|e| format!("queue: {e}"))?;
        self.flush()
    }

    /// Drain every waiting datagram, then send one coalesced acknowledgement
    /// (a cumulative ACK plus selective ACKs beyond it). Acking each packet
    /// separately costs the gateway one event-loop step per ACK.
    fn service(&mut self) -> Result<(), String> {
        self.flush()?;
        let mut buffer = [0u8; 65_535];
        let (mut cumulative, mut selective) = (None::<u16>, Vec::<u16>::new());
        let mut received_any = false;
        // (R) A flooding login peer must not keep service() past the caller's
        // deadline. Leave ordinary gateway draining unchanged.
        let budget = if self.login_crypto.is_some() {
            256
        } else {
            usize::MAX
        };
        for _ in 0..budget {
            let length = match self.socket.recv_from(&mut buffer) {
                Ok((length, peer)) if peer == self.config.gateway => length,
                Ok(_) => continue,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                // Windows reports ICMP port-unreachable as ConnectionReset on UDP.
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                    return Err("gateway port unreachable".into());
                }
                Err(e) => return Err(format!("recv: {e}")),
            };
            received_any = true;
            let Ok(packets) = decode_controls(&self.codec, &buffer[..length]) else {
                continue; // bad CRC or foreign datagram
            };
            for packet in packets {
                match packet.opcode {
                    CUMULATIVE_ACK | SELECTIVE_ACK => {
                        let _ = self.sender.acknowledge(&packet);
                    }
                    RELIABLE_DATA | RELIABLE_FRAGMENT => {
                        let outcome = self
                            .receiver
                            .receive(&packet)
                            .map_err(|e| format!("reliable receive: {e}"))?;
                        match outcome.acknowledgement.map(|ack| ack.packet()) {
                            Some(ack) if ack.opcode == CUMULATIVE_ACK => {
                                cumulative =
                                    Some(u16::from_be_bytes([ack.payload[0], ack.payload[1]]));
                            }
                            Some(ack) => {
                                selective.push(u16::from_be_bytes([ack.payload[0], ack.payload[1]]))
                            }
                            None => {}
                        }
                        let mut messages = Vec::new();
                        for message in outcome.messages {
                            if let Some((rx, _)) = &mut self.login_crypto {
                                for chunk in login::ciphertext_messages(&message)
                                    .map_err(|_| "invalid login frame")?
                                {
                                    let mut plain = chunk.to_vec();
                                    rx.apply(&mut plain);
                                    messages.push(plain);
                                }
                            } else {
                                logical_messages(message, &mut messages);
                            }
                        }
                        for inner in messages {
                            if self.login_crypto.is_some() && self.inbox.len() >= 32 {
                                return Err("native login reply queue exceeds bound".into());
                            }
                            let (family, sub) = family_of(&inner);
                            if !self.config.quiet && self.login_crypto.is_none() {
                                // Some families (39, 45) carry a one-byte sub-code.
                                let label = sub.map_or(String::new(), |s| match s {
                                    0..=255 => format!("/{s}"),
                                    _ => format!("/{} ({s:#06x})", s & 0xff),
                                });
                                self.log(&format!("<- {family}{label} len={}", inner.len()));
                            }
                            self.rezone |= family == 31 && self.in_world;
                            self.received.push((family, sub, inner.len()));
                            self.inbox.push_back(inner);
                        }
                    }
                    5 => return Err("gateway disconnected the session".into()),
                    _ => {}
                }
            }
        }
        if let Some(sequence) = cumulative {
            self.send_raw(CUMULATIVE_ACK, &sequence.to_be_bytes())?;
        }
        selective.sort_unstable();
        selective.dedup();
        for sequence in selective {
            if cumulative.is_none_or(|c| (sequence.wrapping_sub(c) as i16) > 0) {
                self.send_raw(SELECTIVE_ACK, &sequence.to_be_bytes())?;
            }
        }
        if std::mem::take(&mut self.rezone) {
            // Re-enter the world like the native client after a zone change.
            // Queued directly: send_message would clear the inbox mid-wait.
            for (opcode, what) in [(13u16, "ready (13)"), (10, "loaded (10)")] {
                self.log(&format!("-> zoning {what}"));
                self.sender
                    .queue_message(&wrap(&opcode.to_le_bytes()))
                    .map_err(|e| format!("queue: {e}"))?;
            }
            self.flush()?;
        }
        if !received_any {
            std::thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }

    pub fn pump(&mut self, duration: Duration) -> Result<(), String> {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            self.service()?;
        }
        Ok(())
    }

    /// Wait for a message satisfying `predicate`; consumes it and older ones.
    pub fn wait_for(
        &mut self,
        timeout: Duration,
        predicate: impl Fn(&[u8]) -> bool,
    ) -> Result<Vec<u8>, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(index) = self.inbox.iter().position(|m| predicate(m)) {
                let found = self.inbox.drain(..=index).last().unwrap();
                return Ok(found);
            }
            if Instant::now() >= deadline {
                return Err(format!("timed out after {} ms", timeout.as_millis()));
            }
            self.service()?;
        }
    }

    pub fn login(&mut self) -> Result<(), String> {
        let timeout = Duration::from_millis(self.config.expect_timeout_ms);
        let message = login_message(self.config.guid, &self.config.ticket, &self.config.version);
        self.send_message(&message, &format!("login guid={}", self.config.guid))?;
        self.wait_for(timeout, |_| true)
            .map_err(|e| format!("login: no reply ({e}); check guid/ticket/version"))?;
        self.pump(Duration::from_millis(200))?;
        self.send_message(&wrap(&13u16.to_le_bytes()), "ready (13)")?;
        self.wait_for(timeout, |_| true)
            .map_err(|e| format!("ready: no reply ({e})"))?;
        self.pump(Duration::from_millis(200))?;
        self.send_message(&wrap(&10u16.to_le_bytes()), "loaded (10)")?;
        self.pump(Duration::from_millis(300))?;
        // Keep the messages produced by 10/loaded. A late joiner's AddPc and
        // ride actors arrive in this window and belong to the caller's scenario.
        self.in_world = true;
        self.log("in world");
        Ok(())
    }

    /// Runs one step; returns false when the script asked to quit.
    pub fn run_step(&mut self, step: &Step) -> Result<bool, String> {
        match step {
            Step::Chat(text) => {
                self.send_message(&wrap(&chat::local_request(text)), &format!("chat {text:?}"))?
            }
            Step::Wait(ms) => self.pump(Duration::from_millis(*ms))?,
            Step::Move(position, direction) => {
                let message = wrap(&move_request(self.config.guid, *position, *direction));
                self.send_message(&message, &format!("move {position:?}"))?;
            }
            Step::Ability { slot, guid } => {
                self.send_message(
                    &wrap(&ability_request(*slot, *guid)),
                    &format!("ability 36/10 slot={slot} guid={guid}"),
                )?;
            }
            Step::Raw(inner) => self.send_message(&wrap(inner), "raw")?,
            Step::Expect {
                family,
                sub,
                timeout_ms,
            } => {
                let timeout =
                    Duration::from_millis(timeout_ms.unwrap_or(self.config.expect_timeout_ms));
                let label = format!("{family}{}", sub.map_or(String::new(), |s| format!("/{s}")));
                let (family, sub) = (*family, *sub);
                self.wait_for(timeout, |m| matches_family(m, family, sub))
                    .map_err(|e| format!("expect {label}: {e}"))?;
                self.log(&format!("ok expect {label}"));
            }
            Step::ExpectText(text) => {
                let timeout = Duration::from_millis(self.config.expect_timeout_ms);
                let needle = text.as_bytes();
                self.wait_for(timeout, |m| m.windows(needle.len()).any(|w| w == needle))
                    .map_err(|e| format!("expect-text {text:?}: {e}"))?;
                self.log(&format!("ok expect-text {text:?}"));
            }
            Step::Quit => return Ok(false),
        }
        Ok(true)
    }

    pub fn disconnect(&mut self) {
        // Let pending reliable data drain before the SOE disconnect (opcode 5).
        let deadline = Instant::now() + Duration::from_millis(1000);
        while !self.sender.is_idle() && Instant::now() < deadline {
            if self.service().is_err() {
                break;
            }
        }
        let mut payload = self.config.connection_id.to_be_bytes().to_vec();
        payload.extend([0, 0]);
        let _ = self.send_raw(5, &payload);
        self.log("disconnected");
    }
}

/// Connect, log in, run every step and disconnect.
pub fn run(config: BotConfig, steps: &[Step]) -> Result<Bot, String> {
    let mut bot = Bot::connect(config)?;
    let result = bot.login().and_then(|_| {
        for step in steps {
            if !bot.run_step(step)? {
                break;
            }
        }
        Ok(())
    });
    bot.disconnect();
    result.map(|_| bot)
}

// (R) Trusted local/tunnel profile. Paths resolve relative to this file. The
// native RC4 key is transport configuration, never the Ed25519 private seed.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BrokerProfile {
    handle: String,
    key_file: PathBuf,
    server_id_file: PathBuf,
    broker: SocketAddr,
    login: SocketAddr,
    login_key_file: PathBuf,
}

fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|f| f.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|_| "cannot read profile/key file")?;
    if bytes.len() as u64 > limit {
        return Err("profile/key file exceeds bound".into());
    }
    Ok(bytes)
}

fn auth_exchange(address: SocketAddr, body: &[u8]) -> Result<[u8; 69], String> {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|_| "broker connection failed")?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut frame = (body.len() as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(body);
    let mut remaining = frame.as_slice();
    while !remaining.is_empty() {
        let timeout = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("broker deadline")?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|_| "broker I/O failed")?;
        let n = stream.write(remaining).map_err(|_| "broker I/O failed")?;
        if n == 0 {
            return Err("broker closed".into());
        }
        remaining = &remaining[n..];
    }
    let mut reply = [0; 69];
    let mut at = 0;
    while at < reply.len() {
        let timeout = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("broker deadline")?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|_| "broker I/O failed")?;
        let n = stream
            .read(&mut reply[at..])
            .map_err(|_| "broker I/O failed")?;
        if n == 0 {
            return Err("broker closed".into());
        }
        at += n;
    }
    Ok(reply)
}

fn take<'a>(input: &mut &'a [u8], count: usize) -> Result<&'a [u8], String> {
    let bytes = input.get(..count).ok_or("malformed native login reply")?;
    *input = &input[count..];
    Ok(bytes)
}

fn word(input: &mut &[u8]) -> Result<u32, String> {
    Ok(u32::from_le_bytes(take(input, 4)?.try_into().unwrap()))
}

fn login_string<'a>(input: &mut &'a [u8], limit: usize) -> Result<&'a str, String> {
    let count = word(input)? as usize;
    if count > limit {
        return Err("native login string exceeds bound".into());
    }
    std::str::from_utf8(take(input, count)?).map_err(|_| "invalid native login string".into())
}

fn login_request(client: &mut Bot, body: &[u8]) -> Result<Vec<u8>, String> {
    client.send_message(body, "native login request")?;
    client.wait_for(Duration::from_secs(5), |_| true)
}

/// AUTH2 is reconstructed policy (R). Native layouts reuse login.rs and the
/// W94 reference client; provenance is in Key-pair Login Contract.md.
fn broker_handoff(
    path: &Path,
    selected: Option<u64>,
    connection_id: u32,
) -> Result<(SocketAddr, u64, String), String> {
    let profile: BrokerProfile =
        serde_json::from_slice(&read_bounded(path, 4096)?).map_err(|_| "invalid broker profile")?;
    login_handle_hash(&profile.handle).map_err(|_| "invalid account handle")?;
    if !profile.broker.ip().is_loopback() || !profile.login.ip().is_loopback() {
        return Err("broker/native login requires loopback or an encrypted tunnel".into());
    }
    let base = path.parent().unwrap_or(Path::new("."));
    let pin = read_bounded(&base.join(profile.server_id_file), 32)?;
    if pin.len() != 32 {
        return Err("server identity must contain 32 raw bytes".into());
    }
    // (R) A harness may deliver its disposable seed through a private parent pipe.
    let mut seed = if profile.key_file == Path::new("-") {
        let mut seed = Vec::new();
        std::io::stdin()
            .take(33)
            .read_to_end(&mut seed)
            .map_err(|_| "cannot read key pipe")?;
        seed
    } else {
        read_bounded(&base.join(profile.key_file), 32)?
    };
    let raw: &mut [u8; 32] = seed
        .as_mut_slice()
        .try_into()
        .map_err(|_| "Ed25519 key must contain 32 raw bytes")?;
    let signing = SigningKey::from_bytes(raw);
    raw.fill(0);
    let transport_key = read_bounded(&base.join(profile.login_key_file), 256)?;
    login::Rc4::new(&transport_key).map_err(|_| "invalid login transport key")?;
    let mut challenge = b"AUTH2\x01".to_vec();
    put_string(&mut challenge, &profile.handle);
    let challenge = auth_exchange(profile.broker, &challenge)?;
    if challenge[0] != 1 {
        return Err("broker refused".into());
    }
    if challenge[1..33] != pin {
        return Err("server identity mismatch".into());
    }
    if u32::from_le_bytes(challenge[65..69].try_into().unwrap()) != 60 {
        return Err("invalid broker lifetime".into());
    }
    let mut proof = b"AUTH2\x02".to_vec();
    proof.extend_from_slice(&challenge[33..65]);
    proof.extend(signing.verifying_key().to_bytes());
    proof.extend(
        signing
            .sign(
                &signing_message(
                    pin.as_slice().try_into().unwrap(),
                    challenge[33..65].try_into().unwrap(),
                    &profile.handle,
                )
                .map_err(|_| "invalid signing transcript")?,
            )
            .to_bytes(),
    );
    let mut token = auth_exchange(profile.broker, &proof)?;
    if token[0] != 1 {
        return Err("broker refused".into());
    }
    if !token[1..33]
        .iter()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
        || token[33..].iter().any(|b| *b != 0)
    {
        return Err("invalid broker token reply".into());
    }
    println!("broker authenticated");
    // No native UDP socket exists until the proof and identity have passed.
    let mut client = Bot::connect_protocol(
        BotConfig {
            gateway: profile.login,
            name: "login".into(),
            connection_id,
            guid: 0,
            ticket: String::new(),
            version: String::new(),
            expect_timeout_ms: 5000,
            quiet: true,
        },
        Some(&transport_key),
    )?;
    let result = (|| {
        let mut body = vec![1];
        body.extend(32u32.to_le_bytes());
        body.extend_from_slice(&token[1..33]);
        body.extend(0u32.to_le_bytes()); // Fingerprint: empty, no authority.
        token.fill(0);
        let sent = client.send_message(&body, "native Session login");
        body.fill(0);
        sent?;
        let accepted = client.wait_for(Duration::from_secs(5), |_| true)?;
        if accepted.get(..6) != Some(&[2, 1, 1, 0, 0, 0]) {
            return Err("native login refused".into());
        }
        let list = login_request(&mut client, &[11])?;
        let mut input = list.as_slice();
        if take(&mut input, 1)? != [12] || word(&mut input)? != 1 {
            return Err("character list refused".into());
        }
        if take(&mut input, 1)?[0] > 1 {
            return Err("invalid character list".into());
        }
        let count = word(&mut input)?;
        if count > 32 {
            return Err("character list exceeds bound".into());
        }
        let mut choice = None;
        for _ in 0..count {
            let guid = u64::from_le_bytes(take(&mut input, 8)?.try_into().unwrap());
            let server = word(&mut input)?;
            let status = word(&mut input)?;
            let length = word(&mut input)? as usize;
            take(&mut input, length)?;
            if status == 1 && selected.is_none_or(|id| id == guid) && choice.is_none() {
                choice = Some((guid, server));
            }
        }
        if !input.is_empty() {
            return Err("trailing character list data".into());
        }
        let (guid, server) = choice.ok_or("selected character unavailable")?;
        let mut body = vec![7];
        body.extend(guid.to_le_bytes());
        body.extend(server.to_le_bytes());
        let mut data = Vec::new();
        put_string(&mut data, "en_US");
        data.extend(8i32.to_le_bytes());
        data.extend([0, 0]);
        data.extend([0; 8]);
        body.extend((data.len() as u32).to_le_bytes());
        body.extend(data);
        let reply = login_request(&mut client, &body)?;
        let mut input = reply.as_slice();
        if take(&mut input, 1)? != [8]
            || take(&mut input, 8)? != guid.to_le_bytes()
            || word(&mut input)? != server
            || word(&mut input)? != 1
        {
            return Err("character selection refused".into());
        }
        let length = word(&mut input)? as usize;
        if input.len() != length {
            return Err("invalid gateway handoff length".into());
        }
        let address: SocketAddr = login_string(&mut input, 128)?
            .parse()
            .map_err(|_| "invalid gateway address")?;
        if !address.ip().is_loopback() {
            return Err("gateway handoff requires loopback or an encrypted tunnel".into());
        }
        let ticket = login_string(&mut input, 128)?.to_owned();
        let key = login_string(&mut input, 128)?;
        if !matches!(ticket.len(), 16 | 24 | 32)
            || !ticket.is_ascii()
            || key != ticket
            || take(&mut input, 8)? != guid.to_le_bytes()
        {
            return Err("invalid gateway credentials".into());
        }
        take(&mut input, 8)?;
        for _ in 0..3 {
            login_string(&mut input, 128)?;
        }
        if !input.is_empty() {
            return Err("trailing gateway handoff data".into());
        }
        println!("native login selected character {guid}");
        Ok((address, guid, ticket))
    })();
    client.disconnect();
    result
}

const USAGE: &str = "usage: osfr-bot --gateway 127.0.0.1:PORT [--script FILE] [--cmd STEP]...
  [--guid N] [--ticket T] [--version V] [--reference DIR]
  [--account-store FILE --handoff-key-file FILE [--account N | --guid N]]
  [--broker-profile FILE] (instead of direct gateway credentials; --guid selects a listed character)
  [--name NAME] [--connection-id N] [--expect-timeout MS] [--quiet]";

fn config_from_args() -> Result<(BotConfig, Vec<Step>), String> {
    let mut args = std::env::args().skip(1);
    let (mut gateway, mut guid, mut ticket, mut version, mut reference) =
        (None, None, None, None, None::<PathBuf>);
    let (mut account_store, mut key_file, mut account) =
        (None::<PathBuf>, None::<PathBuf>, None::<u64>);
    let (mut name, mut connection_id, mut expect_timeout_ms, mut quiet) = (None, None, 5000, false);
    let mut steps = Vec::new();
    let mut broker_profile = None::<PathBuf>;
    while let Some(key) = args.next() {
        if key == "--quiet" {
            quiet = true;
            continue;
        }
        if key == "--help" || key == "-h" {
            return Err(USAGE.into());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{key} needs a value\n{USAGE}"))?;
        let number = |v: &str| {
            v.parse::<u64>()
                .map_err(|_| format!("{key}: `{v}` is not a number"))
        };
        match key.as_str() {
            "--gateway" => {
                gateway = Some(
                    value
                        .parse::<SocketAddr>()
                        .map_err(|e| format!("--gateway: {e}"))?,
                )
            }
            "--broker-profile" => broker_profile = Some(value.into()),
            "--script" => {
                let text = fs::read_to_string(&value).map_err(|e| format!("{value}: {e}"))?;
                steps.extend(parse_script(&text).map_err(|e| format!("{value}: {e}"))?);
            }
            "--cmd" => steps.extend(parse_step(&value)?),
            "--guid" => guid = Some(number(&value)?),
            "--ticket" => ticket = Some(value),
            "--version" => version = Some(value),
            "--reference" => reference = Some(value.into()),
            "--account-store" => account_store = Some(value.into()),
            "--handoff-key-file" => key_file = Some(value.into()),
            "--account" => account = Some(number(&value)?),
            "--name" => name = Some(value),
            "--connection-id" => connection_id = Some(number(&value)? as u32),
            "--expect-timeout" => expect_timeout_ms = number(&value)?,
            _ => return Err(format!("unknown option {key}\n{USAGE}")),
        }
    }
    if let Some(profile) = broker_profile {
        if gateway.is_some()
            || ticket.is_some()
            || reference.is_some()
            || account_store.is_some()
            || key_file.is_some()
            || account.is_some()
        {
            return Err(
                "--broker-profile cannot be combined with direct gateway credentials".into(),
            );
        }
        let handoff = broker_handoff(&profile, guid, connection_id.unwrap_or(std::process::id()))?;
        gateway = Some(handoff.0);
        guid = Some(handoff.1);
        ticket = Some(handoff.2);
    }
    let gateway =
        gateway.ok_or_else(|| format!("--gateway or --broker-profile is required\n{USAGE}"))?;
    if let Some(reference) = &reference {
        // Default guid/version from the gateway reference manifest (single-save mode).
        let path = reference.join("manifest.json");
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| format!("{}: {e}", path.display()))?;
        version = version.or_else(|| manifest["client_version"].as_str().map(str::to_owned));
        guid = guid.or_else(|| manifest["initial_state"]["guid"].as_u64());
    }
    if let Some(store_path) = &account_store {
        let key_file = key_file.ok_or("--account-store needs --handoff-key-file")?;
        let key = fs::read(&key_file).map_err(|e| format!("{}: {e}", key_file.display()))?;
        let store =
            AccountStore::load(store_path).map_err(|e| format!("{}: {e}", store_path.display()))?;
        if guid.is_none() {
            let id = account.ok_or("--account-store needs --guid or --account")?;
            let owner = store
                .account(id)
                .ok_or_else(|| format!("account {id} not in store"))?;
            guid = Some(
                *owner
                    .character_guids
                    .first()
                    .ok_or_else(|| format!("account {id} has no characters"))?,
            );
        }
        let guid = guid.unwrap();
        let record = store
            .character(guid)
            .ok_or_else(|| format!("character {guid} not in store"))?;
        let owner = store
            .account(record.owner_account_id)
            .ok_or("character owner missing")?;
        ticket = ticket.or_else(|| Some(login::gateway_binding_ticket(&key, owner, guid)));
    }
    let guid = guid.unwrap_or(1);
    let connection_id =
        connection_id.unwrap_or_else(|| std::process::id() ^ (guid as u32).rotate_left(16));
    Ok((
        BotConfig {
            gateway,
            name: name.unwrap_or_else(|| format!("bot{guid}")),
            connection_id,
            guid,
            ticket: ticket.unwrap_or_else(|| login::DEFAULT_GATEWAY_TICKET.to_owned()),
            version: version.unwrap_or_else(|| DEFAULT_VERSION.to_owned()),
            expect_timeout_ms,
            quiet,
        },
        steps,
    ))
}

fn main() {
    let (config, steps) = match config_from_args() {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let name = config.name.clone();
    match run(config, &steps) {
        Ok(bot) => println!("[{name}] PASS ({} messages received)", bot.received.len()),
        Err(error) => {
            println!("[{name}] FAIL: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod handshake_tests {
    use super::*;
    use osfr_tools::session::SessionRequest;

    #[test]
    fn handshake_ignores_foreign_peer_and_connection_id() {
        let gateway = UdpSocket::bind("127.0.0.1:0").unwrap();
        gateway
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let address = gateway.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let mut buffer = [0; 128];
            let (length, peer) = gateway.recv_from(&mut buffer).unwrap();
            let request = SessionRequest::parse(&buffer[..length]).unwrap();
            let foreign = UdpSocket::bind("127.0.0.1:0").unwrap();
            foreign
                .send_to(&request.response_with_seed(1), peer)
                .unwrap();
            let mut wrong = request.clone();
            wrong.id ^= 1;
            gateway.send_to(&wrong.response_with_seed(2), peer).unwrap();
            gateway
                .send_to(&request.response_with_seed(3)[..10], peer)
                .unwrap();
            gateway
                .send_to(&request.response_with_seed(4), peer)
                .unwrap();
        });
        let bot = Bot::connect(BotConfig {
            gateway: address,
            name: "handshake-check".into(),
            connection_id: 123,
            guid: 1,
            ticket: String::new(),
            version: DEFAULT_VERSION.into(),
            expect_timeout_ms: 1000,
            quiet: true,
        })
        .unwrap();
        server.join().unwrap();
        assert_eq!(
            bot.codec.encode(6, &[]).unwrap(),
            Codec::new(4, 3, true, MAX_MESSAGE)
                .unwrap()
                .encode(6, &[])
                .unwrap()
        );
    }
}
