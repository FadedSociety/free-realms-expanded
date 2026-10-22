//! Explicitly local native character creation/selection service. The fixed
//! legacy client key is read from a private file, never embedded in this binary.
use osfr_tools::trace;

use osfr_tools::{
    account_store::{AccountStore, AccountStoreLock},
    identity::Resources,
    login::{self, LoginConfig, LoginService, Rc4},
    login_auth::{LoginAuth, LoginFailures, login_handle_hash},
    net_policy::{Bucket, IpLimiter},
    session::{clock_reflect, decode_controls},
    soe::{
        CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SendError, SenderConfig},
};
use serde_json::json;
use std::{
    collections::HashMap,
    env, fs, io,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Options {
    bind: SocketAddr,
    resources: PathBuf,
    identity: PathBuf,
    state: PathBuf,
    account_store: PathBuf,
    key: Vec<u8>,
    handoff_key: Vec<u8>,
    gateway: String,
    session: String,
    ticket: String,
    max_characters: u32,
    underage: bool,
    open_chat: bool,
    seconds: u64,
    trace_file: Option<PathBuf>,
    /// Accept a private-network bind and peers (Security Model L1).
    allow_lan: bool,
    authenticated: bool,
    auth_bind: SocketAddr,
    server_id: Option<[u8; 32]>,
}
impl Options {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut args = env::args().skip(1);
        let mut bind = "127.0.0.1:20042".parse::<SocketAddr>()?;
        let (
            mut resources,
            mut identity,
            mut state,
            mut account_store,
            mut key_file,
            mut handoff_key_file,
            mut gateway,
        ) = (None, None, None, None, None, None, None);
        let (mut session, mut ticket, mut seconds, mut trace_file) = (
            "local-session".to_owned(),
            login::DEFAULT_GATEWAY_TICKET.to_owned(),
            0,
            None,
        );
        let (mut max_characters, mut underage, mut open_chat) = (6, false, true);
        let mut allow_lan = false;
        let mut auth_bind: SocketAddr = "127.0.0.1:20043".parse()?;
        let mut server_id_file = None;
        while let Some(flag) = args.next() {
            if flag == "--allow-lan" {
                allow_lan = true;
                continue;
            }
            let value = args.next().ok_or("missing login option value")?;
            match flag.as_str() {
                "--bind" => bind = value.parse()?,
                "--resources" => resources = Some(PathBuf::from(value)),
                "--identity" => identity = Some(PathBuf::from(value)),
                "--state" => state = Some(PathBuf::from(value)),
                "--account-store" => account_store = Some(PathBuf::from(value)),
                "--auth-bind" => auth_bind = value.parse()?,
                "--server-id-file" => server_id_file = Some(PathBuf::from(value)),
                "--key-file" => key_file = Some(PathBuf::from(value)),
                "--handoff-key-file" => handoff_key_file = Some(PathBuf::from(value)),
                "--gateway" => gateway = Some(value),
                "--session" => session = value,
                "--ticket" => ticket = value,
                "--max-characters" => max_characters = value.parse()?,
                "--underage" => underage = value.parse()?,
                "--open-chat" => open_chat = value.parse()?,
                "--seconds" => seconds = value.parse()?,
                "--trace" => trace_file = Some(PathBuf::from(value)),
                _ => return Err(format!("unknown login option {flag}").into()),
            }
        }
        if !osfr_tools::net_policy::allowed(bind.ip(), allow_lan) {
            return Err("login binds loopback, or a private-network IP with --allow-lan".into());
        }
        // L053: every client is sent the gateway address; loopback, or a private/VPN one with --allow-lan.
        if let Some(gateway) = &gateway {
            let ip = gateway.parse::<SocketAddr>().map_err(|_| "invalid --gateway address")?.ip();
            if !osfr_tools::net_policy::allowed(ip, allow_lan) {
                return Err("login hands players to a loopback gateway, or a private-network one with --allow-lan".into());
            }
        }
        let authenticated = account_store.is_some();
        // Security Model decision 2 (2026-10-16): the broker may listen on the
        // VPN address under the same rule as the UDP binds; public stays closed.
        if !osfr_tools::net_policy::allowed(auth_bind.ip(), allow_lan) {
            return Err(
                "credential broker binds loopback, or a private-network IP with --allow-lan".into(),
            );
        }
        if !authenticated && (!bind.ip().is_loopback() || server_id_file.is_some()) {
            return Err(
                "non-loopback login requires --account-store; server ID requires it too".into(),
            );
        }
        let server_id = if authenticated {
            let bytes =
                fs::read(server_id_file.ok_or("--server-id-file required with --account-store")?)?;
            Some(
                bytes
                    .try_into()
                    .map_err(|_| "server ID file must contain exactly 32 bytes")?,
            )
        } else {
            None
        };
        let key_path = key_file.ok_or("--key-file required")?;
        let key = fs::read(&key_path)?;
        Rc4::new(&key)?;
        let handoff_key = fs::read(handoff_key_file.ok_or("--handoff-key-file required")?)?;
        if handoff_key.len() < 16 {
            return Err("handoff key must contain at least 16 bytes".into());
        }
        let identity = identity.ok_or("--identity required")?;
        Ok(Self {
            bind,
            resources: resources.ok_or("--resources required")?,
            state: state.unwrap_or_else(|| identity.with_extension("state.json")),
            account_store: account_store
                .unwrap_or_else(|| identity.with_extension("accounts.json")),
            identity,
            key,
            handoff_key,
            gateway: gateway.ok_or("--gateway required")?,
            session,
            ticket,
            max_characters,
            underage,
            open_chat,
            seconds,
            trace_file,
            allow_lan,
            authenticated,
            auth_bind,
            server_id,
        })
    }
    fn service(&self) -> io::Result<LoginService> {
        self.service_for(&self.session)
    }
    fn service_for(&self, handle: &str) -> io::Result<LoginService> {
        LoginService::new(
            Resources::load(&self.resources)?,
            LoginConfig {
                identity_file: self.identity.clone(),
                legacy_save_file: self.state.clone(),
                account_store_file: self.account_store.clone(),
                gateway_address: self.gateway.clone(),
                local_session: handle.to_owned(),
                gateway_ticket: self.ticket.clone(),
                handoff_key: self.handoff_key.clone(),
                max_characters: self.max_characters,
                underage: self.underage,
                open_chat: self.open_chat,
            },
        )
    }
}

// (R) Launcher-only AUTH2 broker. Fixed 69-byte replies never echo credentials.
// One request/connection, at most 192 body bytes and 16 sockets, 3 s total.
struct BrokerPeer {
    stream: TcpStream,
    started: Instant,
    input: Vec<u8>,
    reply: Option<[u8; 69]>,
    written: usize,
}

struct Broker {
    listener: TcpListener,
    peers: Vec<BrokerPeer>,
    auth: LoginAuth,
    server_id: [u8; 32],
    limits: LoginFailures,
    ips: IpLimiter,
    native_ips: IpLimiter,
    global: Bucket,
    allow_lan: bool,
}

impl Broker {
    fn new(options: &Options) -> io::Result<Self> {
        let server_id = options
            .server_id
            .ok_or_else(|| io::Error::other("missing server ID"))?;
        let listener = TcpListener::bind(options.auth_bind)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            peers: Vec::new(),
            auth: LoginAuth::new(server_id),
            server_id,
            limits: LoginFailures::default(),
            // (R) Stricter than failure-only: all broker connections count,
            // including success and malformed frames. Shared NAT/tunnel budget.
            ips: IpLimiter::new(20.0 / 60.0, 20.0, 1024),
            native_ips: IpLimiter::new(20.0 / 60.0, 20.0, 1024),
            global: Bucket::full(20.0, Instant::now()),
            allow_lan: options.allow_lan,
        })
    }

    fn answer(&mut self, path: &std::path::Path, body: &[u8], now: Instant) -> [u8; 69] {
        let mut reply = [0; 69];
        let result = (|| -> Option<()> {
            if body.get(..5)? != b"AUTH2" {
                return None;
            }
            let store = AccountStore::load(path).ok()?;
            match *body.get(5)? {
                1 => {
                    let length = u32::from_le_bytes(body.get(6..10)?.try_into().ok()?) as usize;
                    if !(3..=32).contains(&length) || body.len() != 10 + length {
                        return None;
                    }
                    let handle = std::str::from_utf8(body.get(10..)?).ok()?;
                    let hash = login_handle_hash(handle).ok()?;
                    if !self.limits.allow(hash, now) {
                        return None;
                    }
                    let nonce = self.auth.issue_challenge(&store, handle, now).ok()?;
                    reply[1..33].copy_from_slice(&self.server_id);
                    reply[33..65].copy_from_slice(&nonce);
                    reply[65..69].copy_from_slice(&60u32.to_le_bytes());
                }
                2 => {
                    if body.len() != 134 {
                        return None;
                    }
                    let nonce = &body[6..38];
                    let hash = self.auth.challenge_hash(nonce)?;
                    if !self.limits.allow(hash, now) {
                        self.auth.discard_challenge(nonce);
                        return None;
                    }
                    match self
                        .auth
                        .verify(&store, nonce, &body[38..70], &body[70..134], now)
                    {
                        Ok(token) => {
                            self.limits.succeeded(&hash);
                            reply[1..33].copy_from_slice(token.as_bytes());
                        }
                        Err(_) => {
                            self.limits.failed(hash, now);
                            return None;
                        }
                    }
                }
                _ => return None,
            }
            Some(())
        })();
        if result.is_some() {
            reply[0] = 1;
        } else {
            eprintln!(
                "{}",
                json!({"event":"auth_refused","reason":"invalid_credentials"})
            );
        }
        reply
    }

    fn poll(&mut self, path: &std::path::Path, now: Instant) -> io::Result<()> {
        self.auth.expire(now);
        // Bounded accepts, reads and writes keep a trickling TCP peer off SOE's clock.
        for _ in 0..8 {
            match self.listener.accept() {
                Ok((stream, address)) => {
                    // Peers pass the bind rule; the per-IP and global buckets
                    // below apply to every peer, loopback included.
                    if !osfr_tools::net_policy::allowed(address.ip(), self.allow_lan)
                        || self.peers.len() >= 16
                    {
                        continue;
                    }
                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }
                    let admitted =
                        self.ips.allow(address.ip(), now) && self.global.take(1.0, 2.0, 20.0, now);
                    self.peers.push(BrokerPeer {
                        stream,
                        started: now,
                        input: Vec::new(),
                        written: 0,
                        reply: if admitted { None } else { Some([0; 69]) },
                    });
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut index = 0;
        while index < self.peers.len() {
            let mut peer = self.peers.swap_remove(index);
            if now.saturating_duration_since(peer.started) >= Duration::from_secs(3) {
                continue;
            }
            if peer.reply.is_none() {
                let mut buffer = [0; 196];
                let remaining = buffer.len() - peer.input.len();
                match peer.stream.read(&mut buffer[..remaining]) {
                    Ok(0) => continue,
                    Ok(n) => peer.input.extend_from_slice(&buffer[..n]),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => continue,
                }
                if peer.input.len() >= 4 {
                    let length = u32::from_le_bytes(peer.input[..4].try_into().unwrap()) as usize;
                    if !(10..=192).contains(&length) || peer.input.len() > length + 4 {
                        peer.reply = Some([0; 69]);
                    } else if peer.input.len() == length + 4 {
                        peer.reply = Some(self.answer(path, &peer.input[4..], now));
                    }
                }
            }
            if let Some(reply) = peer.reply {
                match peer.stream.write(&reply[peer.written..]) {
                    Ok(0) => continue,
                    Ok(n) => peer.written += n,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => continue,
                }
                if peer.written == reply.len() {
                    continue;
                }
            }
            self.peers.insert(index, peer);
            index += 1;
        }
        Ok(())
    }
}

/// Parse the existing two-string native login before consuming a Session token.
fn native_session(bytes: &[u8]) -> io::Result<&[u8]> {
    let bad = || io::Error::new(io::ErrorKind::InvalidData, "invalid credentials");
    if bytes.first() != Some(&1) {
        return Err(bad());
    }
    let mut offset = 1usize;
    let mut session = &[][..];
    for (index, bound) in [1024, 4096].into_iter().enumerate() {
        let raw = bytes.get(offset..offset + 4).ok_or_else(bad)?;
        let length = u32::from_le_bytes(raw.try_into().map_err(|_| bad())?) as usize;
        if length > bound {
            return Err(bad());
        }
        offset += 4;
        let end = offset.checked_add(length).ok_or_else(bad)?;
        let value = bytes.get(offset..end).ok_or_else(bad)?;
        std::str::from_utf8(value).map_err(|_| bad())?;
        if index == 0 {
            session = value;
        }
        offset = end;
    }
    if offset != bytes.len() {
        return Err(bad());
    }
    Ok(session)
}

fn handle_login_request(handle: &str) -> Vec<u8> {
    let mut out = vec![1];
    out.extend((handle.len() as u32).to_le_bytes());
    out.extend(handle.as_bytes());
    out.extend(0u32.to_le_bytes());
    out
}

#[derive(PartialEq, Eq)]
struct Request {
    id: u32,
    crc: u8,
    buffer: usize,
}
impl Request {
    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 25 || bytes[..2] != [0, 1] || &bytes[14..] != b"LoginUdp_6\0" {
            return None;
        }
        let crc = u32::from_be_bytes(bytes[2..6].try_into().ok()?);
        let buffer = u32::from_be_bytes(bytes[10..14].try_into().ok()?) as usize;
        if crc == 0 || crc > 4 || !(64..=65507).contains(&buffer) {
            return None;
        }
        Some(Self {
            id: u32::from_be_bytes(bytes[6..10].try_into().ok()?),
            crc: crc.min(2) as u8,
            buffer: buffer.min(512),
        })
    }
    fn response(&self, seed: u32) -> Vec<u8> {
        let mut out = vec![0, 2];
        out.extend(self.id.to_be_bytes());
        out.extend(seed.to_be_bytes());
        // Login compression disabled: both encryption-method transport flags 0.
        // Application RC4 remains enabled, as it is hardcoded in this client.
        out.extend([self.crc, 0, 0]);
        out.extend((self.buffer as u32).to_be_bytes());
        out.extend(3u32.to_be_bytes());
        out
    }
}

struct Session {
    peer: SocketAddr,
    request: Request,
    response: Vec<u8>,
    codec: Codec,
    receiver: ReliableReceiver,
    sender: ReliableSender,
    service: LoginService,
    rx: Rc4,
    tx: Rc4,
    last: Instant,
    sent: u64,
    received: u64,
    trace: Option<trace::Trace>,
    principal: Option<(u64, String)>,
    born: Instant,
    packets: Bucket,
}
impl Session {
    fn new(
        peer: SocketAddr,
        request: Request,
        options: &Options,
        seed: u32,
        trace: Option<trace::Trace>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let codec = Codec::new(seed, request.crc, false, login::MAX_LOGIN_MESSAGE + 1)?;
        let sender = ReliableSender::new(
            codec.clone(),
            SenderConfig {
                datagram_limit: request.buffer,
                window: 32,
                max_message: login::MAX_LOGIN_MESSAGE + 1,
                max_queued_bytes: 256 * 1024,
                retry_ms: 250,
                max_retries: 40,
            },
        )?;
        let response = request.response(seed);
        Ok(Self {
            peer,
            request,
            response,
            codec,
            receiver: ReliableReceiver::new(0, 128, login::MAX_LOGIN_MESSAGE + 1, 128 * 1024)?,
            sender,
            service: options.service()?,
            rx: Rc4::new(&options.key)?,
            tx: Rc4::new(&options.key)?,
            last: Instant::now(),
            sent: 0,
            received: 0,
            trace,
            principal: None,
            born: Instant::now(),
            packets: Bucket::full(256.0, Instant::now()),
        })
    }
    fn send(&mut self, socket: &UdpSocket, bytes: &[u8]) -> io::Result<()> {
        socket.send_to(bytes, self.peer)?;
        self.sent += 1;
        if let Some(trace) = &self.trace {
            let opcode = bytes
                .get(..2)
                .map(|raw| u16::from_be_bytes([raw[0], raw[1]]));
            let decoded = self.codec.decode(bytes).ok();
            trace.record(json!({
                "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                "direction":"out",
                "transport":{"soe_opcode":decoded.as_ref().map_or(opcode, |packet| Some(packet.opcode)),
                    "soe_name":decoded.as_ref().map_or_else(|| opcode.map(trace::transport_name), |packet| Some(trace::transport_name(packet.opcode))),
                    "reliable_seq":decoded.as_ref().and_then(|packet| packet.payload.get(..2)).map(|raw| u16::from_be_bytes([raw[0], raw[1]]))},
                "body_len":decoded.as_ref().map_or(bytes.len(), |packet| packet.payload.len()),
                "body_head_hex":null,"outcome":"sent"
            }))?;
        }
        Ok(())
    }
    fn handle(
        &mut self,
        socket: &UdpSocket,
        wire: &[u8],
        now: u64,
        options: &Options,
        mut broker: Option<&mut Broker>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let packets = match decode_controls(&self.codec, wire) {
            Ok(v) => v,
            Err(error) => {
                if let Some(trace) = &self.trace {
                    trace.record(json!({
                        "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                        "direction":"in","transport":{"soe_opcode":wire.get(..2).map(|raw| u16::from_be_bytes([raw[0], raw[1]]))},
                        "body_len":wire.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                    }))?;
                }
                return Ok(false);
            }
        };
        self.received += 1;
        for packet in packets {
            match packet.opcode {
                RELIABLE_DATA | RELIABLE_FRAGMENT => {
                    let outcome = match self.receiver.receive(&packet) {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            if let Some(trace) = &self.trace {
                                trace.record(json!({
                                    "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                    "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),
                                        "reliable_seq":packet.payload.get(..2).map(|raw| u16::from_be_bytes([raw[0], raw[1]])),"mode":if packet.opcode == RELIABLE_FRAGMENT { "fragment" } else { "complete" }},
                                    "body_len":packet.payload.len().saturating_sub(2),"body_head_hex":null,
                                    "outcome":"rejected","reason":error.to_string()
                                }))?;
                            }
                            return Err(error.into());
                        }
                    };
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),
                                "reliable_seq":packet.payload.get(..2).map(|raw| u16::from_be_bytes([raw[0], raw[1]])),
                                "mode":if packet.opcode == RELIABLE_FRAGMENT { "fragment" } else if outcome.messages.len() > 1 { "combined" } else { "complete" }},
                            "body_len":packet.payload.len().saturating_sub(2),"body_head_hex":null,
                            "outcome":if outcome.rejected_buffered.is_some() { "rejected" } else { "received" },
                            "reason":outcome.rejected_buffered.as_ref().map(|(seq, error)| format!("seq={seq}: {error}"))
                        }))?;
                    }
                    if outcome.rejected_buffered.is_some() {
                        return Err("invalid buffered login packet".into());
                    }
                    if let Some(ack) = outcome.acknowledgement {
                        let p = ack.packet();
                        self.send(socket, &self.codec.encode(p.opcode, &p.payload)?)?;
                    }
                    for message in outcome.messages {
                        let ciphertexts = match login::ciphertext_messages(&message) {
                            Ok(ciphertexts) => ciphertexts,
                            Err(error) => {
                                if let Some(trace) = &self.trace {
                                    trace.record(json!({
                                        "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                        "direction":"in","handler":"login.ciphertext_messages",
                                        "body_len":message.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                                    }))?;
                                }
                                return Err(error.into());
                            }
                        };
                        for ciphertext in ciphertexts {
                            let mut application = ciphertext.to_vec();
                            self.rx.apply(&mut application);
                            let result = match self.application(
                                &application,
                                options,
                                broker.as_deref_mut(),
                            ) {
                                Ok(result) => result,
                                Err(error) => {
                                    if let Some(trace) = &self.trace {
                                        trace.record(json!({
                                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                            "direction":"in","login_opcode":application.first(),
                                            "handler":"LoginService::handle","body_len":application.len(),
                                            "body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                                        }))?;
                                    }
                                    return Err(error.into());
                                }
                            };
                            self.last = Instant::now();
                            if let Some(trace) = &self.trace {
                                trace.record(json!({
                                    "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                    "direction":"in","login_opcode":application.first(),
                                    "handler":"LoginService::handle","body_len":application.len(),
                                    "body_head_hex":null,
                                    "outcome":if result.creation_rejection.is_some() { "rejected" } else if result.created { "created" } else if result.selected { "selected" } else { "handled" },
                                    "reason":result.creation_rejection.clone()
                                }))?;
                            }
                            println!(
                                "{}",
                                json!({"event":"login_application","opcode":application[0],"bytes":application.len(),"created":result.created,"selected":result.selected,"creation_rejection":result.creation_rejection})
                            );
                            for mut reply in result.replies {
                                if let Some(trace) = &self.trace {
                                    trace.record(json!({
                                        "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                        "direction":"out","login_opcode":reply.first(),
                                        "handler":"LoginService::handle","body_len":reply.len(),
                                        "body_head_hex":null,"outcome":"queued"
                                    }))?;
                                }
                                self.tx.apply(&mut reply);
                                self.sender
                                    .queue_message(&login::frame_ciphertext(&reply)?)?;
                            }
                        }
                    }
                }
                CUMULATIVE_ACK | SELECTIVE_ACK => match self.sender.acknowledge(&packet) {
                    Ok(()) => {
                        self.last = Instant::now();
                        if let Some(trace) = &self.trace {
                            trace.record(json!({
                                    "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                    "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                                    "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"handled"
                                }))?;
                        }
                    }
                    Err(SendError::InvalidAcknowledgement) => {
                        if let Some(trace) = &self.trace {
                            trace.record(json!({
                                    "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                    "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                                    "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"ignored","reason":"invalid acknowledgement"
                                }))?;
                        }
                    }
                    Err(error) => {
                        if let Some(trace) = &self.trace {
                            trace.record(json!({
                                    "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                                    "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                                    "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"rejected","reason":error.to_string()
                                }))?;
                        }
                        return Err(error.into());
                    }
                },
                6 => {
                    if packet.payload.is_empty() {
                        self.last = Instant::now();
                    }
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                            "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"handled"
                        }))?;
                    }
                    self.send(socket, &[0, 6])?;
                }
                7 => {
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                            "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"handled"
                        }))?;
                    }
                    if let Some(p) = clock_reflect(&packet, now, self.sent, self.received) {
                        self.last = Instant::now();
                        self.send(socket, &self.codec.encode(p.opcode, &p.payload)?)?;
                    }
                }
                5 => {
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                            "body_len":packet.payload.len(),"body_head_hex":null,
                            "outcome":if packet.payload.len() == 6 && packet.payload[..4] == self.request.id.to_be_bytes() { "handled" } else { "ignored" }
                        }))?;
                    }
                    if packet.payload.len() == 6
                        && packet.payload[..4] == self.request.id.to_be_bytes()
                    {
                        return Ok(true);
                    }
                }
                _ => {
                    if let Some(trace) = &self.trace {
                        trace.record(json!({
                            "event":"packet","session":self.request.id,"peer":self.peer.to_string(),
                            "direction":"in","transport":{"soe_opcode":packet.opcode,"soe_name":trace::transport_name(packet.opcode),"mode":"control"},
                            "body_len":packet.payload.len(),"body_head_hex":null,"outcome":"ignored","reason":"unsupported SOE opcode"
                        }))?;
                    }
                }
            }
        }
        Ok(false)
    }

    fn application(
        &mut self,
        bytes: &[u8],
        options: &Options,
        broker: Option<&mut Broker>,
    ) -> io::Result<login::LoginOutcome> {
        let Some(broker) = broker else {
            return self.service.handle(bytes);
        };
        if bytes.first() == Some(&1) {
            self.principal = None;
            self.service.reset_connection();
            // Validate both strings and exact end before spending the token.
            let token = native_session(bytes)?;
            let redeemed = if broker.native_ips.allow(self.peer.ip(), Instant::now()) {
                let store = AccountStore::load(&options.account_store)?;
                broker.auth.redeem_identity(&store, token, Instant::now())
            } else {
                Err(osfr_tools::login_auth::AuthError::InvalidCredentials)
            };
            match redeemed {
                Ok(identity) => self.principal = Some(identity),
                Err(_) => {
                    return Ok(login::LoginOutcome {
                        replies: vec![login::login_reply(false)],
                        created: false,
                        selected: false,
                        selected_binding: None,
                        creation_rejection: None,
                    });
                }
            }
        }
        let (id, handle) = self
            .principal
            .as_ref()
            .ok_or_else(|| io::Error::other("login required"))?;
        let store = AccountStore::load(&options.account_store)?;
        if store.account_id_by_login_hash(&login_handle_hash(handle).map_err(io::Error::other)?)
            != Some(*id)
            || !store.public_keys(*id).is_some_and(|keys| !keys.is_empty())
        {
            self.principal = None;
            return Err(io::Error::other("invalid credentials"));
        }
        // ponytail: reuse all native character handlers unchanged. Reload their
        // cached store before each request. L042: only the handlers that save take
        // the store writer lock (login.rs), so read spam cannot stall staff writes.
        // This also reloads resource tables; replace with the lead's shared-store
        // API if measured login load warrants it. No packet can choose this
        // synthetic identity.
        self.service = options.service_for(handle)?;
        let authenticated = self.service.handle(&handle_login_request(handle))?;
        if bytes.first() == Some(&1) {
            return Ok(authenticated);
        }
        self.service.handle(bytes)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.service.reset_connection();
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse()?;
    // (R) Account-store mode never creates/migrates a store from network input.
    // L042: startup only validates the store and refuses a stale lock; each save locks it.
    if options.authenticated {
        let _check = AccountStoreLock::acquire(&options.account_store)?;
        AccountStore::load(&options.account_store)?;
    }
    options.service()?;
    // T575: the startup line says whether creation refuses the looks the pinned
    // creator tables hide (identity.rs); a bundle's login-resources turns it on.
    let creator_allow_list = Resources::load(&options.resources)?.filters_looks();
    let trace = options
        .trace_file
        .as_deref()
        .map(trace::Trace::open)
        .transpose()?;
    let mut broker = if options.authenticated {
        Some(Broker::new(&options)?)
    } else {
        None
    };
    let socket = UdpSocket::bind(options.bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(10)))?;
    println!(
        "{}",
        json!({"event":"listening","address":socket.local_addr()?.to_string(),
        "protocol":"LoginUdp_6","identity":options.identity,
        "creator_allow_list":creator_allow_list})
    );
    if let Some(broker) = &broker {
        println!(
            "{}",
            json!({"event":"auth_listening","address":broker.listener.local_addr()?.to_string(),
            "protocol":"AUTH2"})
        );
    }
    let start = Instant::now();
    let mut sessions: HashMap<SocketAddr, Session> = HashMap::new();
    let mut handshakes = IpLimiter::new(20.0, 32.0, 1024);
    let mut buffer = [0; 65535];
    let mut generation = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
    loop {
        let instant = Instant::now();
        let now = start.elapsed().as_millis() as u64;
        if options.seconds > 0 && start.elapsed() >= Duration::from_secs(options.seconds) {
            break;
        }
        if let Some(broker) = &mut broker {
            broker.poll(&options.account_store, instant)?;
        }
        match socket.recv_from(&mut buffer) {
            Ok((n, peer)) if osfr_tools::net_policy::allowed(peer.ip(), options.allow_lan) => {
                let wire = &buffer[..n];
                if wire.starts_with(&[0, 1]) {
                    if handshakes.allow(peer.ip(), instant) {
                        if let Some(request) = Request::parse(wire) {
                            // (R) Only an exact retry may reuse an occupied peer.
                            // A new handshake never replaces another connection.
                            if let Some(session) = sessions.get_mut(&peer) {
                                if session.request == request {
                                    let reply = session.response.clone();
                                    let _ = session.send(&socket, &reply);
                                }
                            } else if sessions.len() < 32 {
                                generation = generation.wrapping_add(1);
                                match Session::new(
                                    peer,
                                    request,
                                    &options,
                                    generation ^ 0x70617274,
                                    trace.clone(),
                                ) {
                                    Ok(mut session) => {
                                        let reply = session.response.clone();
                                        if session.send(&socket, &reply).is_ok() {
                                            sessions.insert(peer, session);
                                        }
                                    }
                                    Err(_) => {} // one malformed/unserviceable peer cannot stop login
                                }
                            }
                        }
                    }
                } else if let Some(session) = sessions.get_mut(&peer) {
                    if session.packets.take(1.0, 200.0, 256.0, instant)
                        && !matches!(
                            session.handle(&socket, wire, now, &options, broker.as_mut()),
                            Ok(false)
                        )
                    {
                        sessions.remove(&peer);
                    }
                }
            }
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::ConnectionReset
                ) => {}
            Err(e) => return Err(e.into()),
        }
        sessions.retain(|_, session| {
            let age = instant.saturating_duration_since(session.born);
            if age >= Duration::from_secs(600)
                || session.last.elapsed() > Duration::from_secs(120)
                || (options.authenticated
                    && session.principal.is_none()
                    && age >= Duration::from_secs(30))
            {
                return false;
            }
            match session.sender.poll(now) {
                Ok(packets) => packets
                    .into_iter()
                    .all(|bytes| session.send(&socket, &bytes).is_ok()),
                Err(_) => false,
            }
        });
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("osfr-login: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_login_parser_rejects_all_truncations_trailing_and_oversized_lengths() {
        let packet = handle_login_request("0123456789abcdef0123456789abcdef");
        assert_eq!(
            native_session(&packet).unwrap(),
            b"0123456789abcdef0123456789abcdef"
        );
        for end in 0..packet.len() {
            assert!(native_session(&packet[..end]).is_err());
        }
        let mut extra = packet.clone();
        extra.push(0);
        assert!(native_session(&extra).is_err());
        for offset in [1, 37] {
            let mut bad = packet.clone();
            bad[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(native_session(&bad).is_err());
        }
        let mut invalid_utf8 = packet;
        invalid_utf8[5] = 255;
        assert!(native_session(&invalid_utf8).is_err());
    }
}
