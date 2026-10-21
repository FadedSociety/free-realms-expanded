//! T370: the real server path for one normal player.
//!
//! A `Player`-role account with developer tools off logs in through the real
//! `Server` (SOE handshake, login 1, ready 13, loaded 10), moves with 125/1,
//! speaks with 15/1, logs out cleanly, and relogs against a restarted server on
//! the same state directory. A content-backed session proves the live role gate
//! refuses `realm` input. The suite then proves that a malformed packet ends
//! only that session and that an abandoned socket leaves the server and every
//! other session running until the existing idle timeout.
//!
//! Test-only: no login, admission, session, transport, save-ordering, role or
//! content code changes. The frames are the client's own shapes:
//! * movement 125 reader `0x008e5940`: `u16 opcode`, `u64 GUID`, `3*f32
//!   position`, `3*f32 direction`, `u8 state`, `u8 unknown`. The reader also
//!   writes the position W as 1.0 and the direction W as 0.0, so only 24 float
//!   bytes are on the wire; handler `0x0090de90`;
//! * typed chat 15/1 sender `0x00b15340`, reader `0x00b140c0`; the request body
//!   is the production `chat::local_request` builder
//!   (`notes/Research/Unrouted Client Requests.md` R051);
//! * login request opcode 1, ready 13 and loaded 10 are the native
//!   `gateway.login -> gateway.ready -> gateway.loaded` sequence recorded in
//!   `lab/autopilot/native-notes/2026-10-20.md` (run 204059Z).
//!
//! The in-process harness has no trace sink (`Server::new` does not take one),
//! so acceptance is asserted from the real server/session state and replies
//! rather than from `rejected` rows; the security log is used for the
//! developer-command audit.

#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;

#[path = "common/gateway_reference.rs"]
mod gateway_reference;

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use gateway_reference::Reference;
use gateway_runner::{Options, Server};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::{AccountRole, AccountStore},
    chat,
    identity::{NativeCreation, Resources},
    login,
    soe::{
        CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SenderConfig},
};
use serde_json::Value;
use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

const VERSION: &str = "synthetic.1";
const MAX_MESSAGE: usize = 16 * 1024 * 1024;
const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";
/// Upper bound for one handshake, login stream or refused session; never a pace.
const PATIENCE: Duration = Duration::from_secs(20);
/// A parseable pose far outside the server's movement envelope (`movement.rs`
/// hard speed multiplier); a component past `MAX_POSITION_COMPONENT` would be
/// refused by the wire reader instead.
const GROSS_MOVE_X: f32 = 10_000.0;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-playtest-gateway-smoke-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn wrapped(outer: u16, inner: &[u8]) -> Vec<u8> {
    let mut out = outer.to_le_bytes().to_vec();
    out.push(1);
    out.extend_from_slice(&(inner.len() as u32).to_le_bytes());
    out.extend_from_slice(inner);
    out
}

fn packet(opcode: u16, fields: &[u8]) -> Vec<u8> {
    let mut inner = opcode.to_le_bytes().to_vec();
    inner.extend_from_slice(fields);
    wrapped(5, &inner)
}

/// One tunneled application request (outer 5), as the client's reliable
/// application channel sends its gameplay messages.
fn application(inner: &[u8]) -> Vec<u8> {
    wrapped(5, inner)
}

/// The real login request: opcode 1, binding ticket, character GUID, client
/// version, empty fourth string. Character 1 keeps the synthetic dev ticket,
/// which is the gateway's own fixture path.
fn login(guid: u64, account_store_path: &Path) -> Vec<u8> {
    let store = AccountStore::load(account_store_path).unwrap();
    let record = store.character(guid).unwrap();
    let account = store.account(record.owner_account_id).unwrap();
    let ticket = if account.id == 1 && guid == 1 {
        "synthetic-ticket".to_owned()
    } else {
        login::gateway_binding_ticket(HANDOFF_KEY, account, guid)
    };
    let mut out = 1u16.to_le_bytes().to_vec();
    put_string(&mut out, &ticket);
    out.extend_from_slice(&guid.to_le_bytes());
    put_string(&mut out, VERSION);
    put_string(&mut out, "");
    out
}

/// Movement 125 reader `0x008e5940`: GUID, position, direction, state, unknown.
fn position_request(guid: u64, pose: [f32; 3], direction: [f32; 3]) -> Vec<u8> {
    let mut fields = guid.to_le_bytes().to_vec();
    for value in pose.into_iter().chain(direction) {
        fields.extend_from_slice(&value.to_le_bytes());
    }
    fields.extend_from_slice(&[0, 0]);
    packet(125, &fields)
}

fn create_account_store(root: &Path) -> PathBuf {
    let resources_dir = root.join("creator");
    let character_create = resources_dir.join("CharacterCreate");
    fs::create_dir_all(&character_create).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1";
    model[13] = "1";
    fs::write(resources_dir.join("Models.txt"), model.join("^")).unwrap();
    for (file, row) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ] {
        fs::write(character_create.join(file), row).unwrap();
    }
    let mut definitions = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 42];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "item.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "0".into();
        row[41] = "tint".into();
        definitions.push_str(&row.join("^"));
        definitions.push('\n');
    }
    fs::write(character_create.join("ItemDefinitions.txt"), definitions).unwrap();
    let resources = Resources::load(&resources_dir).unwrap();
    let mut store = AccountStore::new();
    let accounts = [
        store
            .create_account(
                [6; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
        store
            .create_account(
                [7; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
        store
            .create_account(
                [8; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
    ];
    for (index, (first, last, temporary)) in [
        ("Cedar", "Record", "Tmp"),
        ("Aster", "One", "TempA"),
        ("Beryl", "Two", "TempB"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut choices = [0; 21];
        choices[0] = 1;
        choices[1] = 1;
        choices[2] = 2;
        choices[5] = 1;
        choices[7] = 201;
        choices[8] = 189;
        choices[9] = 199;
        choices[20] = 1;
        let reply = create_character(
            &mut store,
            accounts[index],
            NativeCreation {
                first_name: first.into(),
                last_name: last.into(),
                temporary_first_name: temporary.into(),
                temporary_last_name: "TempName".into(),
                choices,
            },
            &resources,
        );
        assert_eq!(
            reply.status,
            osfr_tools::account::CharacterCreateStatus::Created
        );
    }
    let path = root.join("accounts.json");
    store.save(&path).unwrap();
    path
}

fn options(temp: &TempDir, reference: &Reference, store: &Path, dev_tools: bool) -> Options {
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: reference.directory.clone(),
        state: temp.0.join("gateway-state.json"),
        content: None,
        identity: None,
        account_store: Some(store.to_owned()),
        gateway_ticket: "synthetic-ticket".into(),
        handoff_key: HANDOFF_KEY.to_vec(),
        seconds: 0,
        dev_tools,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: Some(temp.0.join("security.jsonl")),
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    }
}

fn start_server(temp: &TempDir, reference: &Reference, store: &Path, dev_tools: bool) -> Server {
    Server::new(options(temp, reference, store, dev_tools)).unwrap()
}

/// One real client session: the SOE handshake, the reliable sender/receiver and
/// every message this test sent (for the no-dev-command audit).
struct Client {
    socket: UdpSocket,
    server: SocketAddr,
    connection_id: u32,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    clock_ms: u64,
    sent: Vec<Vec<u8>>,
}

impl Client {
    fn connect(server: &mut Server, connection_id: u32) -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        let mut request = vec![0, 1];
        request.extend_from_slice(&3u32.to_be_bytes());
        request.extend_from_slice(&connection_id.to_be_bytes());
        request.extend_from_slice(&8192u32.to_be_bytes());
        request.extend_from_slice(b"CGAPI_527\0");
        socket
            .send_to(&request, server.local_addr().unwrap())
            .unwrap();
        server.step().unwrap();
        let mut response = [0; 64];
        let (length, _) = socket.recv_from(&mut response).unwrap();
        assert!(length >= 10);
        assert_eq!(&response[..2], &[0, 2]);
        let seed = u32::from_be_bytes(response[6..10].try_into().unwrap());
        let codec = Codec::new(seed, 3, true, MAX_MESSAGE).unwrap();
        Self {
            socket,
            server: server.local_addr().unwrap(),
            connection_id,
            sender: ReliableSender::new(
                codec.clone(),
                SenderConfig {
                    datagram_limit: 512,
                    window: 32,
                    max_message: MAX_MESSAGE,
                    max_queued_bytes: 32 * 1024 * 1024,
                    retry_ms: 250,
                    max_retries: 40,
                },
            )
            .unwrap(),
            receiver: ReliableReceiver::new(0, 1024, MAX_MESSAGE, 2 * 1024 * 1024).unwrap(),
            codec,
            clock_ms: 0,
            sent: Vec::new(),
        }
    }

    /// Send one message and service the server until its reliable data is
    /// acknowledged, returning every reply drained meanwhile.
    fn send(&mut self, server: &mut Server, message: &[u8]) -> Vec<Vec<u8>> {
        self.sent.push(message.to_vec());
        self.sender.queue_message(message).unwrap();
        let deadline = Instant::now() + PATIENCE;
        let mut messages = Vec::new();
        loop {
            self.clock_ms += 1;
            for wire in self.sender.poll(self.clock_ms).unwrap() {
                self.socket.send_to(&wire, self.server).unwrap();
            }
            server.step().unwrap();
            messages.extend(self.drain(server));
            if self.sender.is_idle() {
                return messages;
            }
            assert!(
                Instant::now() < deadline,
                "the session did not acknowledge a message"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// For a packet the server may refuse by ending the session: transmit it
    /// once and service the server. A refused message is never acknowledged.
    fn send_unacknowledged(&mut self, server: &mut Server, message: &[u8]) -> Vec<Vec<u8>> {
        self.sent.push(message.to_vec());
        self.sender.queue_message(message).unwrap();
        self.clock_ms += 1;
        for wire in self.sender.poll(self.clock_ms).unwrap() {
            self.socket.send_to(&wire, self.server).unwrap();
            server.step().unwrap();
        }
        server.step().unwrap();
        self.drain(server)
    }

    fn drain(&mut self, server: &mut Server) -> Vec<Vec<u8>> {
        let mut messages = Vec::new();
        let mut pending_acks = 0;
        for _ in 0..512 {
            let mut buffer = [0; 65_535];
            let Ok((length, _)) = self.socket.recv_from(&mut buffer) else {
                break;
            };
            let Ok(packet) = self.codec.decode(&buffer[..length]) else {
                continue;
            };
            if matches!(packet.opcode, CUMULATIVE_ACK | SELECTIVE_ACK) {
                let _ = self.sender.acknowledge(&packet);
            } else if matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT) {
                let result = self.receiver.receive(&packet).unwrap();
                messages.extend(result.messages);
                if let Some(ack) = result.acknowledgement {
                    let ack = ack.packet();
                    let wire = self.codec.encode(ack.opcode, &ack.payload).unwrap();
                    self.socket.send_to(&wire, self.server).unwrap();
                    // Batch a window's worth so the server's 10 ms socket
                    // timeout is not paid per acknowledged packet.
                    pending_acks += 1;
                    if pending_acks == 32 {
                        server.step().unwrap();
                        pending_acks = 0;
                    }
                }
            }
        }
        messages
    }
}

/// Service the server and every live client until every server reply has been
/// delivered and acknowledged. The server only ticks and flushes on `step`.
fn settle_clients(server: &mut Server, clients: &mut [&mut Client]) -> Vec<Vec<Vec<u8>>> {
    let deadline = Instant::now() + PATIENCE;
    let mut out = vec![Vec::new(); clients.len()];
    loop {
        server.step().unwrap();
        for (index, client) in clients.iter_mut().enumerate() {
            out[index].extend(client.drain(server));
        }
        if server.test_queues_idle() && clients.iter().all(|client| client.sender.is_idle()) {
            return out;
        }
        assert!(Instant::now() < deadline, "the server did not settle");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Service a single session through its login stream.
fn settle(server: &mut Server, client: &mut Client) -> Vec<Vec<u8>> {
    settle_clients(server, &mut [client]).remove(0)
}

/// Keep the server and the live clients moving for `duration` without waiting
/// for a globally idle queue (an abandoned session can still hold fanout).
fn pump_clients(
    server: &mut Server,
    clients: &mut [&mut Client],
    duration: Duration,
) -> Vec<Vec<Vec<u8>>> {
    let deadline = Instant::now() + duration;
    let mut out = vec![Vec::new(); clients.len()];
    loop {
        server.step().unwrap();
        for (index, client) in clients.iter_mut().enumerate() {
            out[index].extend(client.drain(server));
        }
        if Instant::now() >= deadline {
            return out;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Connect and send the real login 1, ready 13 and loaded 10 handshake. The
/// caller settles the stream (with every live session) when it is ready to.
fn log_in(
    server: &mut Server,
    connection_id: u32,
    guid: u64,
    store: &Path,
) -> (Client, Vec<Vec<u8>>) {
    let mut client = Client::connect(server, connection_id);
    let mut messages = client.send(server, &login(guid, store));
    messages.extend(client.send(server, &packet(13, &[])));
    messages.extend(client.send(server, &packet(10, &[])));
    (client, messages)
}

fn disconnect(client: &Client, server: &mut Server) {
    let mut fields = client.connection_id.to_be_bytes().to_vec();
    fields.extend_from_slice(&[0, 0]);
    let wire = client.codec.encode(5, &fields).unwrap();
    client.socket.send_to(&wire, client.server).unwrap();
    server.step().unwrap();
}

/// The tunneled application opcode at offset 7 (the `[5, 0, 1]` prefix plus the
/// u32 length).
fn tunneled_opcode(message: &[u8]) -> Option<u16> {
    (message.len() >= 9 && message[..3] == [5, 0, 1])
        .then(|| u16::from_le_bytes(message[7..9].try_into().unwrap()))
}

fn is_family(message: &[u8], family: u16, sub: u16) -> bool {
    message.len() >= 11
        && message[..3] == [5, 0, 1]
        && u16::from_le_bytes(message[7..9].try_into().unwrap()) == family
        && u16::from_le_bytes(message[9..11].try_into().unwrap()) == sub
}

fn is_opcode(message: &[u8], opcode: u16) -> bool {
    tunneled_opcode(message) == Some(opcode)
}

fn contains_bytes(message: &[u8], needle: &[u8]) -> bool {
    needle.len() <= message.len() && message.windows(needle.len()).any(|window| window == needle)
}

/// The login-phase typed self-data packet (opcode 12). The client's self-data
/// reader `0x00944fe0` (dispatched from `0x009471a0`) reads two `u64` identity
/// fields and then the model `u32`, so the character GUID is the second `u64`
/// at offset 21. The gateway validates that offset against the template and the
/// reference manifest (`gateway.rs::derive_self_layout`); the built packet was
/// native-accepted in `notes/Reviews/Native Startup Builders 215401.md`.
fn self_packet(messages: &[Vec<u8>]) -> Option<&Vec<u8>> {
    messages.iter().find(|message| is_opcode(message, 12))
}

fn self_guid_offset(reference: &Reference) -> usize {
    reference.manifest["self_layout"]["guid_offset"]
        .as_u64()
        .unwrap() as usize
}

fn self_guid(message: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        message
            .get(offset..offset.checked_add(8)?)?
            .try_into()
            .ok()?,
    ))
}

fn save_json(temp: &TempDir, guid: u64) -> Value {
    serde_json::from_slice(
        &fs::read(temp.0.join(format!("characters/{guid}.json")))
            .expect("the character save exists"),
    )
    .expect("the character save is JSON")
}

#[test]
fn a_normal_player_logs_in_enters_moves_chats_and_relogs() {
    let temp = TempDir::new();
    let reference = Reference::new(VERSION, None);
    let store_path = create_account_store(&temp.0);
    let store = AccountStore::load(&store_path).unwrap();
    let identity = store.character(2).unwrap().identity.clone();
    let guid_offset = self_guid_offset(&reference);
    let pose = [4.0, 2.0, 3.0];
    // The pose set inside the 5 s save coalesce window; only the logout save
    // can persist it.
    let logout_pose = [5.0, 2.0, 3.0];
    let direction = [0.0, 0.0, 1.0];

    // One normal player on the real server: login 1, ready 13, loaded 10.
    let mut server = start_server(&temp, &reference, &store_path, false);
    let (mut client, mut login_messages) = log_in(&mut server, 501, 2, &store_path);
    login_messages.extend(settle(&mut server, &mut client));
    let self_data = self_packet(&login_messages).expect("login returns the 12 self-data packet");
    assert_eq!(
        self_guid(self_data, guid_offset),
        Some(2),
        "the self-data record names the selected character"
    );
    let (state, _, ready, _) = server
        .test_location(2)
        .expect("the character entered the world");
    assert!(ready, "loaded 10 marks the session ready");
    assert_eq!(state.guid, 2);
    assert_eq!(state.position, [1.0, 2.0, 3.0], "the synthetic start pose");
    assert_eq!(server.active_session_count(), 1);

    // Movement 125/1 inside the supported zone: accepted, never rolled back.
    let replies = client.send(&mut server, &position_request(2, pose, direction));
    assert!(
        !replies.iter().any(|message| is_family(message, 38, 12)),
        "an accepted pose must not be answered with the location snap-back"
    );
    let (state, _, ready, _) = server.test_location(2).unwrap();
    assert!(ready);
    assert_eq!(state.position, pose, "the server accepted the client pose");
    assert_eq!(state.rotation, direction);
    let saved = save_json(&temp, 2);
    assert_eq!(saved["character"]["guid"], serde_json::json!(2));
    assert_eq!(saved["character"]["position"], serde_json::json!(pose));

    // Chat 15/1: the ordinary line is delivered back to its sender.
    let replies = client.send(
        &mut server,
        &application(&chat::local_request("synthetic smoke line")),
    );
    assert!(
        replies
            .iter()
            .any(|message| is_family(message, 15, 1)
                && contains_bytes(message, b"synthetic smoke line")),
        "the say line must reach its own session"
    );
    assert!(
        !replies
            .iter()
            .any(|message| is_family(message, 15, 3)
                && contains_bytes(message, b"developer command")),
        "an ordinary line is not a developer-command refusal"
    );
    assert_eq!(server.active_session_count(), 1);

    // A second pose lands inside the 5 s save coalesce window, so the ordinary
    // commit defers it; only the logout save can write it. Prove the file still
    // holds the earlier pose before the disconnect.
    let replies = client.send(&mut server, &position_request(2, logout_pose, direction));
    assert!(
        !replies.iter().any(|message| is_family(message, 38, 12)),
        "the coalesced pose must not be refused"
    );
    assert_eq!(server.test_location(2).unwrap().0.position, logout_pose);
    assert_eq!(
        save_json(&temp, 2)["character"]["position"],
        serde_json::json!(pose),
        "the pose inside the coalesce window must still be deferred"
    );

    // Clean logout: the real session save writes the last accepted pose.
    disconnect(&client, &mut server);
    assert_eq!(server.active_session_count(), 0);
    let save_path = temp.0.join("characters/2.json");
    let after_logout = fs::read(&save_path).unwrap();
    let saved: Value = serde_json::from_slice(&after_logout).unwrap();
    assert_eq!(saved["character"]["guid"], serde_json::json!(2));
    assert_eq!(saved["character"]["zone_id"], serde_json::json!(0));
    assert_eq!(
        saved["character"]["position"],
        serde_json::json!(logout_pose),
        "the logout path wrote the deferred pose"
    );
    server.shutdown().unwrap();
    drop(server);

    // Restart on the same state directory: the save is the source of truth.
    let mut server = start_server(&temp, &reference, &store_path, false);
    let (mut relogged, mut relogin) = log_in(&mut server, 502, 2, &store_path);
    relogin.extend(settle(&mut server, &mut relogged));
    let self_data = self_packet(&relogin).expect("the relog returns the 12 self-data packet");
    assert_eq!(
        self_guid(self_data, guid_offset),
        Some(2),
        "the same character comes back"
    );
    // The manifest `position_offset` describes the captured template, but
    // `build_self_message` patches the appearance strings from the account
    // store, so their serialized lengths move the transform offset; parse the
    // emitted bytes with the production typed reader instead.
    let parsed = osfr_tools::startup_selfdata::parse_self(self_data);
    let position = parsed.transform.position.map(f32::from_bits);
    let rotation = parsed.transform.rotation.map(f32::from_bits);
    assert_eq!(
        position,
        [logout_pose[0], logout_pose[1], logout_pose[2], 1.0],
        "the self-data record carries the saved position"
    );
    assert_eq!(
        rotation,
        [direction[0], direction[1], direction[2], 0.0],
        "the self-data record carries the saved rotation"
    );
    let transform: Vec<u8> = logout_pose
        .into_iter()
        .chain([1.0])
        .chain(direction)
        .chain([0.0])
        .flat_map(f32::to_le_bytes)
        .collect();
    assert_eq!(
        self_data
            .windows(transform.len())
            .filter(|window| *window == transform.as_slice())
            .count(),
        1,
        "the position/W=1 rotation/W=0 transform appears exactly once"
    );
    assert!(
        contains_bytes(self_data, identity.first_name.as_bytes())
            && contains_bytes(self_data, identity.last_name.as_bytes()),
        "the account store's character name comes back in self-data"
    );
    let (restored, _, ready, _) = server
        .test_location(2)
        .expect("the relogged character entered the world");
    assert!(ready);
    assert_eq!(restored.guid, 2);
    assert_eq!(restored.zone_id, 0);
    assert_eq!(restored.position, logout_pose);
    assert_eq!(restored.rotation, direction);
    assert_eq!(
        server.active_session_count(),
        1,
        "one session after the relog"
    );

    // A duplicate login of the same character is refused before it reads or
    // writes the save; the live owner keeps its session.
    let mut duplicate = Client::connect(&mut server, 503);
    let before_duplicate = fs::read(&save_path).unwrap();
    let refused = duplicate.send_unacknowledged(&mut server, &login(2, &store_path));
    assert!(
        !refused.iter().any(|message| is_opcode(message, 12)),
        "a duplicate login received self data"
    );
    assert_eq!(
        server.active_session_count(),
        1,
        "a duplicate login created a second session"
    );
    assert_eq!(
        fs::read(&save_path).unwrap(),
        before_duplicate,
        "a refused duplicate login changed the save"
    );
    let replies = relogged.send(
        &mut server,
        &application(&chat::local_request("synthetic relog line")),
    );
    assert!(
        replies
            .iter()
            .any(|message| is_family(message, 15, 1)
                && contains_bytes(message, b"synthetic relog line")),
        "the owner's session is still usable after the refused duplicate"
    );

    disconnect(&relogged, &mut server);
    assert_eq!(server.active_session_count(), 0);
    server.shutdown().unwrap();
}

#[test]
fn the_smoke_uses_no_dev_commands_or_roles() {
    let temp = TempDir::new();
    let reference = Reference::new(VERSION, None);
    let store_path = create_account_store(&temp.0);
    let store = AccountStore::load(&store_path).unwrap();
    let owner = store.character(2).unwrap().owner_account_id;
    let role = store
        .account_role(owner)
        .expect("the account store records the account role");
    assert_eq!(role, AccountRole::Player, "a normal player account");
    // The session role gate itself is exercised through the real dispatch in
    // `a_player_role_session_refuses_a_realm_command`; this test only proves
    // the ordinary flow sends no developer text.

    let mut server = start_server(&temp, &reference, &store_path, false);
    let (mut client, _) = log_in(&mut server, 511, 2, &store_path);
    settle(&mut server, &mut client);
    client.send(
        &mut server,
        &position_request(2, [3.0, 2.0, 3.0], [0.0, 0.0, 1.0]),
    );
    let replies = client.send(
        &mut server,
        &application(&chat::local_request("synthetic ordinary line")),
    );
    assert!(
        replies.iter().any(|message| is_family(message, 15, 1)
            && contains_bytes(message, b"synthetic ordinary line")),
        "the ordinary say line is delivered"
    );

    // The flow sent no developer text, no role change and no teleport; every
    // message is one of the ordinary login/ready/loaded/movement/chat frames.
    assert_eq!(
        &client.sent[0][..2],
        &[1, 0],
        "the first message is login 1"
    );
    for message in &client.sent[1..] {
        let opcode = tunneled_opcode(message).unwrap_or_else(|| {
            panic!("message is not a tunneled application request: {message:?}")
        });
        assert!(
            matches!(opcode, 10 | 13 | 125 | 15),
            "unexpected ordinary-flow opcode {opcode}"
        );
        if opcode == 15 {
            assert_eq!(
                u16::from_le_bytes(message[9..11].try_into().unwrap()),
                1,
                "typed chat is family 15 subtype 1"
            );
        }
    }
    let saw = |opcode: u16| {
        client
            .sent
            .iter()
            .any(|message| tunneled_opcode(message) == Some(opcode))
    };
    assert!(
        saw(13) && saw(10) && saw(125) && saw(15),
        "ordinary families only"
    );
    for message in &client.sent {
        let lower = message.to_ascii_lowercase();
        assert!(
            !contains_bytes(&lower, b"realm "),
            "the flow sent a `realm` developer command"
        );
        assert!(
            !contains_bytes(&lower, b"teleport"),
            "the flow sent a teleport developer command"
        );
    }
    assert_eq!(
        server.active_session_count(),
        1,
        "the session was not refused"
    );

    // No developer-command denial and no budget refusal reached the security
    // log, and the log does not echo the character name or the login ticket.
    server.flush_security();
    let log = fs::read_to_string(temp.0.join("security.jsonl")).unwrap_or_default();
    assert!(!log.contains("dev_command_denial"), "{log}");
    assert!(!log.contains("budget_refusal"), "{log}");
    let identity = &store.character(2).unwrap().identity;
    assert!(!log.contains(&identity.first_name), "name reached the log");
    assert!(!log.contains("synthetic-ticket"), "ticket reached the log");
    let peer = client.socket.local_addr().unwrap().to_string();
    assert!(!log.contains(&peer), "peer address reached the log");
}

/// The live session, not a separately constructed `Gameplay`, must apply the
/// account store's `Player` role: a `realm` developer command is refused on the
/// real dispatch path and the refusal changes nothing.
#[test]
fn a_player_role_session_refuses_a_realm_command() {
    let temp = TempDir::new();
    let fixture = npc_quest_chain::Fixture::new();
    let reference = Reference::new(VERSION, Some(fixture.path()));
    let store_path = create_account_store(&temp.0);
    let owner = AccountStore::load(&store_path)
        .unwrap()
        .character(2)
        .unwrap()
        .owner_account_id;
    assert_eq!(
        AccountStore::load(&store_path).unwrap().account_role(owner),
        Some(AccountRole::Player)
    );

    let mut server_options = options(&temp, &reference, &store_path, false);
    server_options.content = Some(fixture.path().to_owned());
    let mut server = Server::new(server_options).unwrap();
    let (mut client, _) = log_in(&mut server, 531, 2, &store_path);
    settle(&mut server, &mut client);
    let save_path = temp.0.join("characters/2.json");
    let before = fs::read(&save_path).unwrap();
    let before_pose = server.test_location(2).unwrap().0.position;

    let replies = client.send(
        &mut server,
        &application(&chat::local_request("realm give 10")),
    );
    assert!(
        replies
            .iter()
            .any(|message| is_family(message, 15, 3)
                && contains_bytes(message, b"role does not allow")),
        "the live Player session must refuse the developer command"
    );
    assert_eq!(
        server.active_session_count(),
        1,
        "a refused developer command ended the session"
    );
    assert_eq!(
        server.test_location(2).unwrap().0.position,
        before_pose,
        "a refused developer command moved the character"
    );
    assert_eq!(
        fs::read(&save_path).unwrap(),
        before,
        "a refused developer command changed the save"
    );

    // The session still works after the refusal.
    let replies = client.send(
        &mut server,
        &application(&chat::local_request("synthetic after refusal")),
    );
    assert!(
        replies.iter().any(|message| is_family(message, 15, 1)
            && contains_bytes(message, b"synthetic after refusal")),
        "the session speaks after the refused developer command"
    );

    // The refusal is the existing server-owned audit event.
    server.flush_security();
    let log = fs::read_to_string(temp.0.join("security.jsonl")).unwrap_or_default();
    assert!(log.contains("dev_command_denial"), "{log}");
    assert!(!log.contains("synthetic after refusal"), "{log}");
    server.shutdown().unwrap();
    assert_eq!(server.active_session_count(), 0);
}

#[test]
fn a_bad_packet_or_abrupt_disconnect_ends_only_that_session() {
    let temp = TempDir::new();
    let reference = Reference::new(VERSION, None);
    let store_path = create_account_store(&temp.0);
    let save_path = temp.0.join("characters/2.json");
    let guid_offset = self_guid_offset(&reference);
    let start = [1.0, 2.0, 3.0];
    let direction = [0.0, 0.0, 1.0];

    let mut server = start_server(&temp, &reference, &store_path, false);
    let (mut client, _) = log_in(&mut server, 521, 2, &store_path);
    settle(&mut server, &mut client);
    let before_out_of_bounds = fs::read(&save_path).unwrap();

    // A pose outside the supported range is refused: the character stays on its
    // accepted pose and the server sends its own location message instead of
    // committing the client's.
    let replies = client.send(
        &mut server,
        &position_request(2, [GROSS_MOVE_X, start[1], start[2]], direction),
    );
    assert_eq!(
        server.active_session_count(),
        1,
        "a bad pose ended the session"
    );
    assert_eq!(
        server.test_location(2).unwrap().0.position,
        start,
        "an out-of-range pose moved the character"
    );
    let location = replies
        .iter()
        .find(|message| is_family(message, 38, 12))
        .expect("the refused pose is answered with the server location");
    let expected: Vec<u8> = start.iter().flat_map(|value| value.to_le_bytes()).collect();
    assert_eq!(
        &location[11..23],
        expected.as_slice(),
        "the location reply restores the accepted pose"
    );
    assert_eq!(
        fs::read(&save_path).unwrap(),
        before_out_of_bounds,
        "a refused pose changed the save"
    );

    // A truncated owned packet is refused with a contained error: only this
    // session ends, and it saves no invented state.
    let before_malformed = fs::read(&save_path).unwrap();
    client.send_unacknowledged(&mut server, &packet(125, &2u64.to_le_bytes()));
    assert_eq!(
        server.active_session_count(),
        0,
        "a malformed packet must end that session"
    );
    assert_eq!(
        fs::read(&save_path).unwrap(),
        before_malformed,
        "a malformed packet changed the save"
    );
    server.step().unwrap();

    // A fresh session with the same character still works after the refusal.
    let (mut fresh, fresh_login) = log_in(&mut server, 522, 2, &store_path);
    settle(&mut server, &mut fresh);
    assert_eq!(
        self_guid(
            self_packet(&fresh_login).expect("fresh login self data"),
            guid_offset
        ),
        Some(2)
    );
    assert!(
        server.test_location(2).unwrap().2,
        "the fresh session is ready"
    );
    let moved = [5.0, 2.0, 3.0];
    fresh.send(&mut server, &position_request(2, moved, direction));
    assert_eq!(server.test_location(2).unwrap().0.position, moved);
    let replies = fresh.send(
        &mut server,
        &application(&chat::local_request("synthetic survivor line")),
    );
    assert!(
        replies.iter().any(|message| is_family(message, 15, 1)
            && contains_bytes(message, b"synthetic survivor line")),
        "the fresh session speaks"
    );

    // Separately, drop a session's socket mid-world while the other session
    // stays live; the server keeps stepping, the abandoned session stays until
    // the existing idle timeout (so its character cannot be relogged), and the
    // live session and a new visitor still work. Its last pose is deferred, so
    // only an end-of-session save can write it.
    let (mut dropped, mut dropped_login) = log_in(&mut server, 523, 3, &store_path);
    let settled = settle_clients(&mut server, &mut [&mut fresh, &mut dropped]);
    dropped_login.extend(settled[1].clone());
    assert_eq!(
        self_guid(
            self_packet(&dropped_login).expect("dropped login self data"),
            guid_offset
        ),
        Some(3)
    );
    let dropped_saved = [2.0, 2.0, 3.0];
    let dropped_deferred = [3.0, 2.0, 3.0];
    dropped.send(&mut server, &position_request(3, dropped_saved, direction));
    dropped.send(
        &mut server,
        &position_request(3, dropped_deferred, direction),
    );
    assert_eq!(
        save_json(&temp, 3)["character"]["position"],
        serde_json::json!(dropped_saved),
        "the last pose is inside the save coalesce window"
    );
    drop(dropped);
    for _ in 0..5 {
        server.step().unwrap();
    }
    assert_eq!(
        server.active_session_count(),
        2,
        "an abandoned socket stays until the existing idle timeout"
    );
    assert!(
        server.test_location(2).is_some(),
        "dropping one socket must not end the live session"
    );
    // The abandoned session still owns character 3, so a relog is refused
    // before it can read or write that character's save.
    let before_refused_relog = fs::read(temp.0.join("characters/3.json")).unwrap();
    let mut relog = Client::connect(&mut server, 525);
    let refused = relog.send_unacknowledged(&mut server, &login(3, &store_path));
    assert!(
        !refused.iter().any(|message| is_opcode(message, 12)),
        "a relog of an abandoned session received self data"
    );
    assert_eq!(
        server.active_session_count(),
        2,
        "a relog of an abandoned session changed the session set"
    );
    assert_eq!(
        fs::read(temp.0.join("characters/3.json")).unwrap(),
        before_refused_relog,
        "a refused relog changed the abandoned character's save"
    );
    let (mut visitor, mut visitor_login) = log_in(&mut server, 524, 1, &store_path);
    let settled = pump_clients(
        &mut server,
        &mut [&mut fresh, &mut visitor],
        Duration::from_millis(300),
    );
    visitor_login.extend(settled[1].clone());
    assert_eq!(
        self_guid(
            self_packet(&visitor_login).expect("visitor login self data"),
            guid_offset
        ),
        Some(1)
    );
    assert!(
        server.test_location(1).unwrap().2,
        "the visitor session is ready"
    );
    let replies = visitor.send(
        &mut server,
        &application(&chat::local_request("synthetic visitor line")),
    );
    assert!(
        replies.iter().any(|message| is_family(message, 15, 1)
            && contains_bytes(message, b"synthetic visitor line")),
        "the fresh session after the dropped socket speaks"
    );
    // The live session kept working through the drop and the visitor's join.
    let after_drop = [6.0, 2.0, 3.0];
    fresh.send(&mut server, &position_request(2, after_drop, direction));
    assert_eq!(
        server.test_location(2).unwrap().0.position,
        after_drop,
        "the live session still moves after the dropped socket"
    );

    // Leave every live session with a pose inside the save coalesce window, so
    // these poses can only reach the files through the shutdown save path.
    let visitor_saved = [2.0, 2.0, 3.0];
    let visitor_deferred = [3.0, 2.0, 3.0];
    visitor.send(&mut server, &position_request(1, visitor_saved, direction));
    visitor.send(
        &mut server,
        &position_request(1, visitor_deferred, direction),
    );
    let fresh_saved = [7.0, 2.0, 3.0];
    let fresh_deferred = [8.0, 2.0, 3.0];
    fresh.send(&mut server, &position_request(2, fresh_saved, direction));
    fresh.send(&mut server, &position_request(2, fresh_deferred, direction));
    for (guid, final_pose) in [
        (1, visitor_deferred),
        (2, fresh_deferred),
        (3, dropped_deferred),
    ] {
        assert_ne!(
            save_json(&temp, guid)["character"]["position"],
            serde_json::json!(final_pose),
            "character {guid}'s last pose must still be deferred before shutdown"
        );
    }

    // Shutdown ends every session through the same save path and writes each
    // session's final pose, including the abandoned one.
    server.shutdown().unwrap();
    assert_eq!(server.active_session_count(), 0);
    for (guid, final_pose) in [
        (1, visitor_deferred),
        (2, fresh_deferred),
        (3, dropped_deferred),
    ] {
        assert_eq!(
            save_json(&temp, guid)["character"]["position"],
            serde_json::json!(final_pose),
            "the shutdown save wrote character {guid}'s final pose"
        );
    }
}
