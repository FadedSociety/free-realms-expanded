//! Headless launcher client and real LoginUdp_6 route checks. Synthetic keys only.
use ed25519_dalek::{Signer, SigningKey};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy},
    account_store::AccountStore,
    login::{self, Rc4},
    login_auth::{login_handle_hash, signing_message},
    session::decode_controls,
    soe::{
        CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SenderConfig},
};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    path::PathBuf,
    process::{Child, ChildStdout, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const SERVER: [u8; 32] = [42; 32];
/// Upper bound for any wait on a child process; never a pace. Parallel suites starve
/// the services for seconds, so fixed 2-5 s waits flaked in principle (W157).
const PATIENCE: Duration = Duration::from_secs(30);
const HANDLE: &str = "acct_001";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Server {
    child: Child,
    directory: PathBuf,
    auth: SocketAddr,
    udp: SocketAddr,
    _output: BufReader<ChildStdout>,
}
impl Server {
    fn start(secure: bool) -> Self {
        Self::start_at(secure, "127.0.0.1:0".parse().unwrap(), false)
    }

    fn start_at(secure: bool, auth_bind: SocketAddr, allow_lan: bool) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let directory = root.join(format!(
            "test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(directory.join("CharacterCreate")).unwrap();
        let mut model = vec!["0"; 17];
        model[0] = "1";
        model[5] = "1";
        model[13] = "1";
        fs::write(directory.join("Models.txt"), model.join("^")).unwrap();
        for file in [
            "HeadMappings.txt",
            "HairMappings.txt",
            "ModelCustomizationMappings.txt",
            "FacePaintMappings.txt",
            "SkinToneMappings.txt",
            "ItemDefinitions.txt",
        ] {
            let content = if file == "ItemDefinitions.txt" {
                let mut row = vec!["0"; 51];
                row[0] = "1";
                row.join("^")
            } else {
                "1^synthetic.adr^".to_owned()
            };
            fs::write(directory.join("CharacterCreate").join(file), content).unwrap();
        }
        fs::write(directory.join("key"), [9; 32]).unwrap();
        fs::write(directory.join("server-id"), SERVER).unwrap();
        let mut store = AccountStore::new();
        for (handle, seed, max_characters) in [(HANDLE, 7, 2), ("acct_002", 8, 3)] {
            let id = store
                .create_account(
                    login_handle_hash(handle).unwrap(),
                    AccountFlags::default(),
                    SlotPolicy { max_characters },
                )
                .unwrap();
            store
                .add_public_key(
                    id,
                    &SigningKey::from_bytes(&[seed; 32])
                        .verifying_key()
                        .to_bytes(),
                )
                .unwrap();
        }
        // Enrolled account without keys is disabled, never passwordless.
        store
            .create_account(
                login_handle_hash("acct_003").unwrap(),
                AccountFlags::default(),
                SlotPolicy { max_characters: 2 },
            )
            .unwrap();
        store.save(&directory.join("accounts.json")).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-login"));
        command
            .args([
                "--bind",
                "127.0.0.1:0",
                "--gateway",
                "127.0.0.1:20260",
                "--seconds",
                "300",
            ])
            .arg("--resources")
            .arg(&directory)
            .arg("--identity")
            .arg(directory.join("identity.json"))
            .arg("--trace")
            .arg(directory.join("trace.jsonl"))
            .arg("--key-file")
            .arg(directory.join("key"))
            .arg("--handoff-key-file")
            .arg(directory.join("key"));
        if secure {
            command
                .arg("--account-store")
                .arg(directory.join("accounts.json"))
                .arg("--auth-bind")
                .arg(auth_bind.to_string())
                .arg("--server-id-file")
                .arg(directory.join("server-id"));
        }
        if allow_lan {
            command.arg("--allow-lan");
        }
        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut output = BufReader::new(child.stdout.take().unwrap());
        let mut udp = None;
        let mut auth = None;
        while udp.is_none() || (secure && auth.is_none()) {
            let mut line = String::new();
            if output.read_line(&mut line).unwrap() == 0 {
                let result = child.wait_with_output().unwrap();
                panic!(
                    "login server did not listen: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            let event: serde_json::Value = serde_json::from_str(&line).unwrap();
            match event["event"].as_str() {
                Some("listening") => udp = event["address"].as_str().map(|s| s.parse().unwrap()),
                Some("auth_listening") => {
                    auth = event["address"].as_str().map(|s| s.parse().unwrap())
                }
                _ => {}
            }
        }
        Self {
            child,
            directory,
            auth: auth.unwrap_or("127.0.0.1:1".parse().unwrap()),
            udp: udp.unwrap(),
            _output: output,
        }
    }
}

/// L053: login sends every client the gateway address, so it takes loopback, or a private/VPN
/// address only with --allow-lan, and never a public one (the LAN/VPN preflight found login refusing
/// every non-loopback gateway, which blocked the runbook's VPN host setup).
#[test]
fn login_gateway_address_follows_the_lan_rule() {
    for (gateway, allow_lan, refused) in [
        ("8.8.8.8:20041", true, true),
        ("100.64.0.1:20041", false, true),
        ("192.168.1.10:20041", false, true),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-login"));
        command.args(["--bind", "127.0.0.1:0", "--gateway", gateway]);
        if allow_lan {
            command.arg("--allow-lan");
        }
        let result = command.output().unwrap();
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert_eq!(
            stderr.contains("loopback gateway, or a private-network one"),
            refused,
            "{gateway}: {stderr}"
        );
    }
    // With --allow-lan a private gateway passes this gate (the run then stops on missing resources).
    let result = Command::new(env!("CARGO_BIN_EXE_osfr-login"))
        .args([
            "--bind",
            "127.0.0.1:0",
            "--gateway",
            "100.64.0.1:20041",
            "--allow-lan",
        ])
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&result.stderr).contains("loopback gateway"));
}

#[test]
fn broker_refuses_public_wildcard_and_private_without_allow_lan() {
    for (address, allow_lan) in [
        ("8.8.8.8:20043", true),
        ("0.0.0.0:20043", true),
        ("[2001:4860::1]:20043", true),
        ("[::]:20043", true),
        ("10.0.0.1:20043", false),
        ("100.64.0.1:20043", false),
        ("[fd12::1]:20043", false),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-login"));
        command.args(["--auth-bind", address]);
        if allow_lan {
            command.arg("--allow-lan");
        }
        let result = command.output().unwrap();
        assert!(!result.status.success(), "{address}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("credential broker binds loopback"),
            "{address}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

/// Listening on this PC's LAN address makes Windows Firewall prompt once per build folder, so
/// builder and integrator suites skip it. Login, login_auth, net_policy and osfr-login are
/// lead-only files, and every lead security review runs it:
/// `cargo test --test login_broker -- --ignored`.
#[test]
#[ignore = "binds the LAN address; lead security reviews run it with --ignored"]
fn private_broker_bind_admits_private_peers_and_keeps_failure_limits() {
    // UDP connect selects a local route without sending a packet. Override for
    // hosts whose default route is not a private interface.
    let ip = std::env::var("OSFR_TEST_PRIVATE_IP")
        .map(|ip| ip.parse().unwrap())
        .unwrap_or_else(|_| {
            let route = UdpSocket::bind("0.0.0.0:0").unwrap();
            route.connect("192.0.2.1:9").unwrap();
            route.local_addr().unwrap().ip()
        });
    assert!(osfr_tools::net_policy::is_private_network(ip));
    assert!(!ip.is_loopback());
    let server = Server::start_at(true, SocketAddr::new(ip, 0), true);
    assert_eq!(server.auth.ip(), ip);
    let c = challenge(server.auth, HANDLE);
    assert_eq!(c[0], 1);
    assert_eq!(
        proof(
            server.auth,
            HANDLE,
            &c,
            &SigningKey::from_bytes(&[7; 32]),
            &SERVER
        )[0],
        1
    );
    for _ in 0..5 {
        let c = challenge(server.auth, HANDLE);
        assert_eq!(c[0], 1);
        assert_eq!(
            proof(
                server.auth,
                HANDLE,
                &c,
                &SigningKey::from_bytes(&[10; 32]),
                &SERVER
            ),
            [0; 69]
        );
    }
    assert_eq!(challenge(server.auth, HANDLE), [0; 69]);
    // Thirteen connections spent; seven malformed attempts exhaust the IP burst.
    for _ in 0..7 {
        assert_eq!(exchange(server.auth, b"AUTH2\x03xxxx"), [0; 69]);
    }
    // Independent account is refused by source-IP admission, not its own lock.
    assert_eq!(challenge(server.auth, "acct_002"), [0; 69]);
    std::thread::sleep(Duration::from_millis(3100));
    assert_eq!(challenge(server.auth, "acct_002")[0], 1);
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if std::thread::panicking() {
            let mut errors = String::new();
            self.child
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut errors)
                .unwrap();
            eprintln!("test server stderr: {errors}");
            let trace = fs::read_to_string(self.directory.join("trace.jsonl")).unwrap_or_default();
            eprintln!("test server trace: {trace}");
            eprintln!("retained diagnostics: {}", self.directory.display());
        } else {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }
}

/// One bounded AUTH2 exchange per connection; no private key crosses the socket.
fn exchange(address: SocketAddr, body: &[u8]) -> [u8; 69] {
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream
        .write_all(&(body.len() as u32).to_le_bytes())
        .unwrap();
    stream.write_all(body).unwrap();
    let mut reply = [0; 69];
    stream.read_exact(&mut reply).unwrap();
    reply
}

fn challenge(address: SocketAddr, handle: &str) -> [u8; 69] {
    let mut body = b"AUTH2\x01".to_vec();
    body.extend((handle.len() as u32).to_le_bytes());
    body.extend(handle.as_bytes());
    exchange(address, &body)
}

fn proof(
    address: SocketAddr,
    handle: &str,
    challenge: &[u8; 69],
    key: &SigningKey,
    server: &[u8; 32],
) -> [u8; 69] {
    let nonce: [u8; 32] = challenge[33..65].try_into().unwrap();
    let mut body = b"AUTH2\x02".to_vec();
    body.extend(nonce);
    body.extend(key.verifying_key().to_bytes());
    body.extend(
        key.sign(&signing_message(server, &nonce, handle).unwrap())
            .to_bytes(),
    );
    exchange(address, &body)
}

#[test]
fn broker_good_signature_wrong_key_wrong_server_and_no_enumeration() {
    let server = Server::start(true);
    let good = SigningKey::from_bytes(&[7; 32]);
    let c = challenge(server.auth, HANDLE);
    assert_eq!(c[0], 1);
    assert_eq!(&c[1..33], &SERVER);
    assert_eq!(u32::from_le_bytes(c[65..69].try_into().unwrap()), 60);
    let token = proof(server.auth, HANDLE, &c, &good, &SERVER);
    assert_eq!(token[0], 1);
    assert!(token[1..33].iter().all(u8::is_ascii_hexdigit));
    assert_eq!(&token[33..], &[0; 36]);
    assert_eq!(proof(server.auth, HANDLE, &c, &good, &SERVER), [0; 69]);
    for handle in [HANDLE, "acct_999", "acct_003"] {
        let c = challenge(server.auth, handle);
        assert_eq!(c[0], 1);
        assert_eq!(
            proof(
                server.auth,
                handle,
                &c,
                &SigningKey::from_bytes(&[10; 32]),
                &SERVER
            ),
            [0; 69]
        );
    }
    let c = challenge(server.auth, HANDLE);
    assert_eq!(proof(server.auth, HANDLE, &c, &good, &[43; 32]), [0; 69]);
}

/// Native headless client: the broker token becomes Session; Fingerprint has no authority.
struct NativeClient {
    socket: UdpSocket,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    rx: Rc4,
    tx: Rc4,
    start: Instant,
    drop_next_datagram: bool,
}
impl NativeClient {
    fn connect(address: SocketAddr) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.connect(address).unwrap();
        socket.set_read_timeout(Some(PATIENCE)).unwrap();
        let mut request = vec![0, 1];
        request.extend(2u32.to_be_bytes());
        request.extend(71u32.to_be_bytes());
        request.extend(512u32.to_be_bytes());
        request.extend(b"LoginUdp_6\0");
        socket.send(&request).unwrap();
        let mut bytes = [0; 512];
        let n = socket.recv(&mut bytes).unwrap();
        assert_eq!(n, 21);
        let response = bytes[..n].to_vec();
        socket.send(&request).unwrap();
        let n = socket.recv(&mut bytes).unwrap();
        assert_eq!(&bytes[..n], response);
        // A different request from the occupied peer must not reset RC4/state.
        request[6..10].copy_from_slice(&72u32.to_be_bytes());
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        socket.send(&request).unwrap();
        assert!(matches!(
            socket.recv(&mut [0; 512]).unwrap_err().kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        socket.set_read_timeout(Some(PATIENCE)).unwrap();
        let seed = u32::from_be_bytes(bytes[6..10].try_into().unwrap());
        let codec = Codec::new(seed, bytes[10], false, login::MAX_LOGIN_MESSAGE + 1).unwrap();
        let sender = ReliableSender::new(
            codec.clone(),
            SenderConfig {
                datagram_limit: 512,
                window: 32,
                max_message: login::MAX_LOGIN_MESSAGE + 1,
                max_queued_bytes: 256 * 1024,
                retry_ms: 250,
                max_retries: 20,
            },
        )
        .unwrap();
        Self {
            socket,
            codec,
            sender,
            receiver: ReliableReceiver::new(0, 128, login::MAX_LOGIN_MESSAGE + 1, 128 * 1024)
                .unwrap(),
            rx: Rc4::new(&[9; 32]).unwrap(),
            tx: Rc4::new(&[9; 32]).unwrap(),
            start: Instant::now(),
            drop_next_datagram: false,
        }
    }

    fn request(&mut self, body: &[u8]) -> Vec<u8> {
        let mut ciphertext = body.to_vec();
        self.tx.apply(&mut ciphertext);
        self.sender
            .queue_message(&login::frame_ciphertext(&ciphertext).unwrap())
            .unwrap();
        self.socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let waiting = Instant::now();
        let mut buffer = [0; 2048];
        for _ in 0..1024 {
            if waiting.elapsed() >= PATIENCE {
                break;
            }
            for wire in self
                .sender
                .poll(self.start.elapsed().as_millis() as u64)
                .unwrap()
            {
                if self.drop_next_datagram {
                    self.drop_next_datagram = false;
                    continue;
                }
                self.socket.send(&wire).unwrap();
            }
            let n = match self.socket.recv(&mut buffer) {
                Ok(n) => n,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    continue;
                }
                Err(error) => panic!("native opcode {} receive error: {error}", body[0]),
            };
            for packet in decode_controls(&self.codec, &buffer[..n]).unwrap() {
                match packet.opcode {
                    CUMULATIVE_ACK | SELECTIVE_ACK => self.sender.acknowledge(&packet).unwrap(),
                    RELIABLE_DATA | RELIABLE_FRAGMENT => {
                        let outcome = self.receiver.receive(&packet).unwrap();
                        if let Some(ack) = outcome.acknowledgement {
                            let packet = ack.packet();
                            self.socket
                                .send(&self.codec.encode(packet.opcode, &packet.payload).unwrap())
                                .unwrap();
                        }
                        for message in outcome.messages {
                            let chunks = login::ciphertext_messages(&message).unwrap();
                            assert_eq!(chunks.len(), 1);
                            let mut reply = chunks[0].to_vec();
                            self.rx.apply(&mut reply);
                            return reply;
                        }
                    }
                    _ => {}
                }
            }
        }
        panic!(
            "native opcode {} had no reply within deadline/packet budget",
            body[0]
        );
    }

    fn login(&mut self, session: &[u8]) -> Vec<u8> {
        let mut body = vec![1];
        body.extend((session.len() as u32).to_le_bytes());
        body.extend(session);
        body.extend(0u32.to_le_bytes());
        self.request(&body)
    }
}

fn token(server: &Server, handle: &str, seed: u8) -> Vec<u8> {
    let c = challenge(server.auth, handle);
    assert_eq!(c[0], 1);
    let reply = proof(
        server.auth,
        handle,
        &c,
        &SigningKey::from_bytes(&[seed; 32]),
        &SERVER,
    );
    assert_eq!(reply[0], 1);
    reply[1..33].to_vec()
}

#[test]
fn tokens_route_to_distinct_native_accounts_replay_fails_and_peers_stay_live() {
    let server = Server::start(true);
    let original = fs::read(server.directory.join("accounts.json")).unwrap();
    let a_token = token(&server, HANDLE, 7);
    let b_token = token(&server, "acct_002", 8);
    let mut a = NativeClient::connect(server.udp);
    a.drop_next_datagram = true; // prove headless client's reliable retransmission
    assert_eq!(a.login(&a_token), accepted(2));
    let mut b = NativeClient::connect(server.udp);
    assert_eq!(b.login(&b_token), accepted(3));
    let mut replay = NativeClient::connect(server.udp);
    assert_eq!(replay.login(&a_token), login::login_reply(false));
    assert_eq!(replay.login(b"local-session"), login::login_reply(false));
    assert_eq!(replay.login(b"acct_001"), login::login_reply(false));
    // All peers chose the same request ID; source socket remains the identity.
    for client in [&mut a, &mut b] {
        let reply = client.request(&[11]);
        assert_eq!(reply[0], 12);
        assert_eq!(u32::from_le_bytes(reply[6..10].try_into().unwrap()), 0);
    }
    assert_eq!(
        fs::read(server.directory.join("accounts.json")).unwrap(),
        original
    );
    assert!(server.directory.join("accounts.json.lock").is_file());
    assert!(
        osfr_tools::account_store::AccountStoreLock::acquire(
            &server.directory.join("accounts.json")
        )
        .is_err()
    );
}

#[test]
fn loopback_dev_login_without_account_store_still_works() {
    let server = Server::start(false);
    let mut client = NativeClient::connect(server.udp);
    // L057: the account this login creates is a member (IsMember byte 6, payload member byte 11).
    let mut member = accepted(6);
    member[6] = 1;
    member[11] = 1;
    assert_eq!(client.login(b"local-session"), member);
    assert_eq!(client.request(&[11])[0], 12);
}

#[test]
fn account_lockout_unknown_lockout_ip_budget_and_malformed_replies_are_generic() {
    for handle in [HANDLE, "acct_999"] {
        let server = Server::start(true);
        for _ in 0..5 {
            let c = challenge(server.auth, handle);
            assert_eq!(c[0], 1);
            assert_eq!(
                proof(
                    server.auth,
                    handle,
                    &c,
                    &SigningKey::from_bytes(&[10; 32]),
                    &SERVER
                ),
                [0; 69]
            );
        }
        assert_eq!(challenge(server.auth, handle), [0; 69]);
        // Exhaust the shared IP's budget using malformed requests; none reveal state.
        for _ in 0..10 {
            assert_eq!(exchange(server.auth, b"AUTH2\x03xxxx"), [0; 69]);
        }
        assert_eq!(challenge(server.auth, "acct_002"), [0; 69]);
        assert_eq!(challenge(server.auth, "acct_999"), [0; 69]);
        std::thread::sleep(Duration::from_millis(3100));
        assert_eq!(challenge(server.auth, "acct_002")[0], 1); // IP budget refills
    }
}

fn accepted(slots: u32) -> Vec<u8> {
    let mut reply = login::login_reply(true);
    reply[12..16].copy_from_slice(&slots.to_le_bytes());
    reply
}

#[test]
fn slow_broker_peer_cannot_block_native_login_and_expires() {
    let server = Server::start(true);
    let mut slow = TcpStream::connect_timeout(&server.auth, Duration::from_secs(2)).unwrap();
    slow.set_read_timeout(Some(PATIENCE)).unwrap();
    slow.write_all(&[134, 0, 0]).unwrap(); // incomplete frame length
    let token = token(&server, HANDLE, 7);
    let mut native = NativeClient::connect(server.udp);
    assert_eq!(native.login(&token), accepted(2));
    assert_eq!(slow.read(&mut [0; 1]).unwrap(), 0);
}
