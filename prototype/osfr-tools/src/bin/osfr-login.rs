//! Explicitly local native character creation/selection service. The fixed
//! legacy client key is read from a private file, never embedded in this binary.
#[path = "../trace.rs"]
#[allow(dead_code)]
mod trace;

use osfr_tools::{
    identity::Resources,
    login::{self, LoginConfig, LoginService, Rc4},
    session::{clock_reflect, decode_controls},
    soe::{
        Codec, ReliableReceiver, CUMULATIVE_ACK, RELIABLE_DATA, RELIABLE_FRAGMENT, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SendError, SenderConfig},
};
use serde_json::json;
use std::{
    env, fs, io,
    net::{SocketAddr, UdpSocket},
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
        })
    }
    fn service(&self) -> io::Result<LoginService> {
        LoginService::new(
            Resources::load(&self.resources)?,
            LoginConfig {
                identity_file: self.identity.clone(),
                legacy_save_file: self.state.clone(),
                account_store_file: self.account_store.clone(),
                gateway_address: self.gateway.clone(),
                local_session: self.session.clone(),
                gateway_ticket: self.ticket.clone(),
                handoff_key: self.handoff_key.clone(),
                max_characters: self.max_characters,
                underage: self.underage,
                open_chat: self.open_chat,
            },
        )
    }
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
        self.last = Instant::now();
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
                            let result = match self.service.handle(&application) {
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
}

impl Drop for Session {
    fn drop(&mut self) {
        self.service.reset_connection();
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::parse()?;
    options.service()?;
    let trace = options
        .trace_file
        .as_deref()
        .map(trace::Trace::open)
        .transpose()?;
    let socket = UdpSocket::bind(options.bind)?;
    socket.set_read_timeout(Some(Duration::from_millis(10)))?;
    println!(
        "{}",
        json!({"event":"listening","address":socket.local_addr()?.to_string(),"protocol":"LoginUdp_6","identity":options.identity})
    );
    let start = Instant::now();
    let mut session: Option<Session> = None;
    let mut buffer = [0; 65535];
    let mut generation = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
    loop {
        let now = start.elapsed().as_millis() as u64;
        if options.seconds > 0 && start.elapsed() >= Duration::from_secs(options.seconds) {
            break;
        }
        match socket.recv_from(&mut buffer) {
            Ok((n, peer)) if osfr_tools::net_policy::allowed(peer.ip(), options.allow_lan) => {
                let wire = &buffer[..n];
                if wire.starts_with(&[0, 1]) {
                    if let Some(request) = Request::parse(wire) {
                        if !session
                            .as_ref()
                            .is_some_and(|s| s.peer == peer && s.request == request)
                        {
                            generation = generation.wrapping_add(1);
                            session = Some(Session::new(
                                peer,
                                request,
                                &options,
                                generation ^ 0x70617274,
                                trace.clone(),
                            )?);
                            if let Some(trace) = &trace {
                                let session_id = session.as_ref().map(|s| s.request.id);
                                trace.record(json!({
                                    "event":"session_started","session":session_id,"peer":peer.to_string(),
                                    "direction":"local","outcome":"accepted"
                                }))?;
                            }
                            println!(
                                "{}",
                                json!({"event":"session_started","peer":peer.to_string()})
                            );
                        }
                        if let Some(s) = session.as_mut() {
                            let reply = s.response.clone();
                            s.send(&socket, &reply)?;
                        }
                    } else if let Some(trace) = &trace {
                        trace.record(json!({
                            "event":"packet","peer":peer.to_string(),"direction":"in",
                            "transport":{"soe_opcode":1,"soe_name":"SessionRequest","mode":"handshake"},
                            "body_len":wire.len(),"body_head_hex":null,"outcome":"ignored","reason":"invalid login session request"
                        }))?;
                    }
                } else if let Some(s) = session.as_mut().filter(|s| s.peer == peer) {
                    let result = s.handle(&socket, wire, now);
                    if !matches!(result, Ok(false)) {
                        if let Some(trace) = &s.trace {
                            trace.record(json!({
                                "event":"session_ended","session":s.request.id,"peer":s.peer.to_string(),
                                "direction":"local","outcome":"closed",
                                "reason":result.as_ref().err().map(|e|e.to_string()).unwrap_or_else(||"client_disconnect".into())
                            }))?;
                        }
                        println!(
                            "{}",
                            json!({"event":"session_ended","reason":result.err().map(|e|e.to_string()).unwrap_or_else(||"client_disconnect".into())})
                        );
                        session = None;
                    }
                }
            }
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) => {}
            Err(e) => return Err(e.into()),
        }
        if let Some(s) = session.as_mut() {
            let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                for bytes in s.sender.poll(now)? {
                    s.send(&socket, &bytes)?;
                }
                Ok(())
            })();
            if result.is_err() || s.last.elapsed() > Duration::from_secs(120) {
                if let Some(trace) = &s.trace {
                    trace.record(json!({
                        "event":"session_ended","session":s.request.id,"peer":s.peer.to_string(),
                        "direction":"local","outcome":"closed",
                        "reason":result.as_ref().err().map(|e|e.to_string()).unwrap_or_else(||"idle_timeout".into())
                    }))?;
                }
                println!(
                    "{}",
                    json!({"event":"session_ended","reason":result.err().map(|e|e.to_string()).unwrap_or_else(||"idle_timeout".into())})
                );
                session = None;
            }
        }
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("osfr-login: {e}");
        std::process::exit(1);
    }
}
