#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;

use gateway_runner::{Options, Server};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    identity::{NativeCreation, Resources},
    login,
    soe::{
        CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver, SELECTIVE_ACK,
    },
    soe_send::{ReliableSender, SenderConfig},
    startup_snapshot::StartupSnapshot,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[path = "common/combat_scale.rs"]
mod combat_scale;
#[path = "common/field_reward_fixture.rs"]
mod field_reward_fixture;
#[path = "common/group_invite_wire.rs"]
mod group_invite_wire;
#[path = "common/startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
const VERSION: &str = "synthetic.1";
const MAX_MESSAGE: usize = 16 * 1024 * 1024;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-multiplayer-{}-{}",
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

fn login(guid: u64, account_store_path: &Path) -> Vec<u8> {
    login_version(guid, account_store_path, VERSION)
}

fn login_version(guid: u64, account_store_path: &Path, version: &str) -> Vec<u8> {
    let store = AccountStore::load(account_store_path).unwrap();
    let record = store.character(guid).unwrap();
    let account = store.account(record.owner_account_id).unwrap();
    let ticket = login::gateway_binding_ticket(b"synthetic-handoff-key", account, guid);
    let mut out = 1u16.to_le_bytes().to_vec();
    put_string(&mut out, &ticket);
    out.extend_from_slice(&guid.to_le_bytes());
    put_string(&mut out, version);
    put_string(&mut out, "");
    out
}

fn name_record(out: &mut Vec<u8>, first: &str, last: &str) {
    out.extend_from_slice(&[0; 12]);
    put_string(out, first);
    put_string(out, last);
}

fn create_reference(root: &Path) -> PathBuf {
    let reference = root.join("lab/references/rust-gateway-reference");
    fs::create_dir_all(&reference).unwrap();
    let state = CharacterState {
        guid: 1,
        zone_id: 0,
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 1.0],
    };
    let (self_packet, position_offset, rotation_offset) =
        startup_selfdata_fixture::self_data_packet(state.guid, state.position, state.rotation);

    let mut zone = Vec::new();
    put_string(&mut zone, "test zone");
    zone.extend_from_slice(&2u32.to_le_bytes());
    zone.extend_from_slice(&[0, 0]);
    put_string(&mut zone, "sky");
    zone.push(0);
    let zone_id_offset = 9 + zone.len();
    zone.extend_from_slice(&state.zone_id.to_le_bytes());
    zone.extend_from_slice(&5u32.to_le_bytes());
    zone.extend_from_slice(&[0, 0]);
    let zone_packet = wrapped(5, &{
        let mut inner = 43u16.to_le_bytes().to_vec();
        inner.extend_from_slice(&zone);
        inner
    });

    let mut templates = Vec::new();
    for index in 0..46 {
        let opcode = if index < 8 {
            [41, 2, 166, 43, 144, 35, 193, 12][index]
        } else if index == 8 {
            67
        } else if index == 45 {
            14
        } else {
            200
        };
        let bytes = match index {
            3 => zone_packet.clone(),
            7 => self_packet.clone(),
            _ => packet(opcode, &[]),
        };
        let name = format!("{index:04}.bin");
        fs::write(reference.join(&name), &bytes).unwrap();
        templates.push(json!({
            "kind": if index < 8 { "login" } else { "ready" },
            "file": name,
            "length": bytes.len(),
            "sha256": format!("{:x}", Sha256::digest(&bytes)),
            "opcode": opcode
        }));
    }
    let manifest = json!({
        "schema_version": 1,
        "client_version": VERSION,
        "initial_state": state,
        "self_layout": {
            "guid_offset": 21,
            "position_offset": position_offset,
            "rotation_offset": rotation_offset
        },
        "zone_layout": {"zone_id_offset": zone_id_offset},
        "templates": templates
    });
    fs::write(
        reference.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    let startup = root.join("lab/content/startup-snapshot/startup.json");
    fs::create_dir_all(startup.parent().unwrap()).unwrap();
    fs::write(
        startup,
        serde_json::to_vec(&StartupSnapshot::empty()).unwrap(),
    )
    .unwrap();
    reference
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
        store
            .create_account(
                [9; 32],
                AccountFlags::default(),
                SlotPolicy { max_characters: 1 },
            )
            .unwrap(),
    ];
    for (index, (first, last, temporary)) in [
        ("Cedar", "Record", "Tmp"),
        ("Aster", "One", "TempA"),
        ("Beryl", "Two", "TempB"),
        ("Dune", "Four", "TempD"),
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

struct Client {
    socket: UdpSocket,
    server: SocketAddr,
    connection_id: u32,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    clock_ms: u64,
    /// Service ACKs a window at a time. A content login queues hundreds of
    /// packets; stepping the server after each ACK pays its socket timeout per
    /// packet and can outlive another live session's retry window.
    batch_acks: bool,
    /// Every message this client sent, for the no-dev-command audit.
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
            batch_acks: false,
            sent: Vec::new(),
        }
    }

    fn send(&mut self, server: &mut Server, message: &[u8]) -> Vec<Vec<u8>> {
        self.sent.push(message.to_vec());
        self.sender.queue_message(message).unwrap();
        self.clock_ms += 1;
        for wire in self.sender.poll(self.clock_ms).unwrap() {
            self.socket.send_to(&wire, self.server).unwrap();
            server.step().unwrap();
        }
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
                    if self.batch_acks {
                        // A step after every ack pays the server's 10 ms socket
                        // timeout per packet. Batch a sender window and flush
                        // the partial batch below.
                        pending_acks += 1;
                        if pending_acks == 32 {
                            server.step().unwrap();
                            pending_acks = 0;
                        }
                    } else {
                        server.step().unwrap();
                    }
                }
            }
        }
        if self.batch_acks && pending_acks > 0 {
            server.step().unwrap();
        }
        messages
    }
}

fn application(outer: u16, inner: &[u8]) -> Vec<u8> {
    wrapped(outer, inner)
}

fn chat_request(channel: u16, message: &str, target: Option<(&str, &str)>) -> Vec<u8> {
    let mut out = vec![15, 0, 1, 0];
    out.extend_from_slice(&channel.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    name_record(&mut out, "", "");
    if let Some((first, last)) = target {
        name_record(&mut out, first, last);
    } else {
        name_record(&mut out, "", "");
    }
    put_string(&mut out, message);
    for value in [0f32, 0.0, 0.0, 1.0] {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out
}

fn friend_request(first: &str, last: &str) -> Vec<u8> {
    let mut out = vec![26, 0, 14, 0];
    put_string(&mut out, first);
    put_string(&mut out, last);
    out
}

fn friend_response(requester_guid: u64, target_guid: u64) -> Vec<u8> {
    let mut out = vec![26, 0, 17, 0];
    out.extend_from_slice(&requester_guid.to_le_bytes());
    out.extend_from_slice(&target_guid.to_le_bytes());
    out.push(1); // (R) The W23 response parser interprets nonzero as acceptance.
    name_record(&mut out, "Aster", "One");
    out
}

fn ignore_request(add: bool, first: &str, last: &str) -> Vec<u8> {
    let mut out = vec![26, 0, 30, 0, u8::from(add)];
    put_string(&mut out, first);
    put_string(&mut out, last);
    out
}

fn group_name(out: &mut Vec<u8>, first: &str, last: &str) {
    name_record(out, first, last);
}

fn group_invite() -> Vec<u8> {
    let mut out = vec![40, 0, 1, 0];
    out.extend_from_slice(&2u64.to_le_bytes());
    group_name(&mut out, "Aster", "One");
    out.extend_from_slice(&3u64.to_le_bytes());
    group_name(&mut out, "Beryl", "Two");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&[0; 20]);
    out
}

fn group_accept(invite_id: u32) -> Vec<u8> {
    let mut out = vec![40, 0, 4, 0];
    out.extend_from_slice(&2u64.to_le_bytes());
    group_name(&mut out, "Aster", "One");
    out.extend_from_slice(&3u64.to_le_bytes());
    group_name(&mut out, "Beryl", "Two");
    out.extend_from_slice(&invite_id.to_le_bytes());
    out.push(1);
    out.extend_from_slice(&[0; 20]);
    out
}

fn is_family(message: &[u8], family: u16, sub: u16) -> bool {
    message.len() >= 13
        && message[..3] == [5, 0, 1]
        && u16::from_le_bytes(message[7..9].try_into().unwrap()) == family
        && u16::from_le_bytes(message[9..11].try_into().unwrap()) == sub
}

fn is_family_byte_sub(message: &[u8], family: u16, sub: u8) -> bool {
    message.len() >= 10
        && message[..3] == [5, 0, 1]
        && u16::from_le_bytes(message[7..9].try_into().unwrap()) == family
        && message[9] == sub
}

fn is_opcode(message: &[u8], opcode: u16) -> bool {
    message.len() >= 9
        && message[..3] == [5, 0, 1]
        && u16::from_le_bytes(message[7..9].try_into().unwrap()) == opcode
}

fn start_server(temp: &TempDir) -> (Server, PathBuf) {
    start_server_with_content(temp, None)
}

fn start_server_with_content(temp: &TempDir, content: Option<PathBuf>) -> (Server, PathBuf) {
    let reference = if content.is_some() {
        PathBuf::from(std::env::var_os("OSFR_AUDIT_REFERENCE").expect("private reference"))
    } else {
        create_reference(&temp.0)
    };
    start_server_at(temp, reference, content)
}

/// One Options block for every suite in this file: only the reference root and
/// the content tree differ.
fn start_server_at(
    temp: &TempDir,
    reference: PathBuf,
    content: Option<PathBuf>,
) -> (Server, PathBuf) {
    let account_store = create_account_store(&temp.0);
    let server = Server::new(Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference,
        state: temp.0.join("gateway-state.json"),
        content,
        identity: None,
        account_store: Some(account_store.clone()),
        gateway_ticket: "synthetic-ticket".into(),
        handoff_key: b"synthetic-handoff-key".to_vec(),
        seconds: 0,
        dev_tools: true,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: Some(temp.0.join("security.jsonl")),
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    })
    .unwrap();
    (server, account_store)
}

fn log_in(
    server: &mut Server,
    connection_id: u32,
    guid: u64,
    store: &Path,
) -> (Client, Vec<Vec<u8>>) {
    log_in_version(server, connection_id, guid, store, VERSION)
}

fn log_in_version(
    server: &mut Server,
    connection_id: u32,
    guid: u64,
    store: &Path,
    version: &str,
) -> (Client, Vec<Vec<u8>>) {
    let mut client = Client::connect(server, connection_id);
    let mut messages = client.send(server, &login_version(guid, store, version));
    messages.extend(client.send(server, &packet(13, &[])));
    messages.extend(client.send(server, &packet(10, &[])));
    (client, messages)
}

/// The content-bound login: batch ACK servicing so two live sessions survive
/// the ~600-packet self-data transfer. Every other suite keeps the per-ACK step.
fn log_in_version_batched(
    server: &mut Server,
    connection_id: u32,
    guid: u64,
    store: &Path,
    version: &str,
) -> (Client, Vec<Vec<u8>>) {
    let mut client = Client::connect(server, connection_id);
    client.batch_acks = true;
    let mut messages = client.send(server, &login_version(guid, store, version));
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

fn move_player(client: &mut Client, server: &mut Server, guid: u64, x: f32) {
    let mut fields = guid.to_le_bytes().to_vec();
    for value in [x, 2.0, 3.0, 0.0, 0.0, 1.0] {
        fields.extend_from_slice(&value.to_le_bytes());
    }
    fields.extend_from_slice(&[0, 0]);
    client.send(server, &packet(125, &fields));
}

/// W103: reproduce the crossed numeric domains without renumbering existing saves.
#[test]
fn audit_crossed_account_and_character_ids_can_join_together() {
    let temp = TempDir::new();
    let (mut server, store_path) = start_server(&temp);
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&store_path).unwrap()).unwrap();
    document["store"]["accounts"]["2"]["character_guids"] = json!([3]);
    document["store"]["accounts"]["3"]["character_guids"] = json!([2]);
    document["store"]["characters"]["2"]["owner_account_id"] = json!(3);
    document["store"]["characters"]["3"]["owner_account_id"] = json!(2);
    fs::write(&store_path, serde_json::to_vec(&document).unwrap()).unwrap();
    let store = AccountStore::load(&store_path).unwrap();
    assert_eq!(store.character(3).unwrap().owner_account_id, 2);
    assert_eq!(store.character(2).unwrap().owner_account_id, 3);
    // Persist/reload keeps both numeric namespaces and character save filenames intact.
    store.save(&store_path).unwrap();
    assert_eq!(store, AccountStore::load(&store_path).unwrap());
    let (mut first, _) = log_in(&mut server, 201, 3, &store_path);
    let (_second, replies) = log_in(&mut server, 202, 2, &store_path);
    assert_eq!(
        server.active_session_count(),
        2,
        "crossed IDs rejected a valid owner"
    );
    assert!(replies.iter().any(|m| is_family(m, 35, 1)));
    assert!(first.drain(&mut server).iter().any(|m| is_family(m, 35, 1)));
}

/// W103 threat: a second authenticated connection holds an old snapshot before Loaded.
/// Atomic file replacement alone does not establish which session may write it.
fn check_duplicate_snapshot(shutdown: bool) {
    let temp = TempDir::new();
    let (mut server, store) = start_server(&temp);
    let mut survivor = Client::connect(&mut server, 211);
    let mut duplicate = Client::connect(&mut server, 212);
    if survivor.socket.local_addr().unwrap() > duplicate.socket.local_addr().unwrap() {
        std::mem::swap(&mut survivor, &mut duplicate);
    }
    survivor.send(&mut server, &login(2, &store));
    survivor.send(&mut server, &packet(13, &[]));
    survivor.send(&mut server, &packet(10, &[]));
    duplicate.send(&mut server, &login(2, &store));
    // The survivor advances after the other connection could have loaded its snapshot.
    move_player(&mut survivor, &mut server, 2, 4.0);
    let path = temp.0.join("characters/2.json");
    let progress = fs::read(&path).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&progress).unwrap();
    assert_eq!(saved["character"]["position"][0], 4.0);
    if shutdown {
        // Shutdown visits the older owner's socket first, then the duplicate.
        server.shutdown().unwrap();
    } else {
        disconnect(&survivor, &mut server);
        disconnect(&duplicate, &mut server);
    }
    let after: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        after["character"], saved["character"],
        "stale snapshot restored; shutdown={shutdown}"
    );
    assert_eq!(fs::read(&path).unwrap(), progress);
    assert_eq!(server.active_session_count(), 0);
}

#[test]
fn audit_duplicate_snapshot_cannot_overwrite_progress_on_disconnect() {
    check_duplicate_snapshot(false);
}

#[test]
fn audit_duplicate_snapshot_cannot_overwrite_progress_on_shutdown() {
    check_duplicate_snapshot(true);
}

#[test]
fn audit_duplicate_account_is_rejected_before_loading_another_character_save() {
    let temp = TempDir::new();
    let (mut server, store) = start_server(&temp);
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&store).unwrap()).unwrap();
    document["store"]["accounts"]["2"]["character_guids"] = json!([2, 3]);
    document["store"]["accounts"]["2"]["slots"]["max_characters"] = json!(2);
    document["store"]["accounts"]["3"]["character_guids"] = json!([]);
    document["store"]["characters"]["3"]["owner_account_id"] = json!(2);
    fs::write(&store, serde_json::to_vec(&document).unwrap()).unwrap();
    AccountStore::load(&store).unwrap();
    let mut survivor = Client::connect(&mut server, 231);
    survivor.send(&mut server, &login(2, &store)); // Reserve before Ready/Loaded, too.
    let mut duplicate = Client::connect(&mut server, 232);
    let replies = duplicate.send(&mut server, &login(3, &store));
    assert!(
        !replies.iter().any(|m| is_opcode(m, 12)),
        "duplicate received self data"
    );
    assert_eq!(server.active_session_count(), 1);
    assert!(
        !temp.0.join("characters/3.json").exists(),
        "rejected login wrote a save"
    );
    survivor.send(&mut server, &packet(13, &[]));
    survivor.send(&mut server, &packet(10, &[]));
    move_player(&mut survivor, &mut server, 2, 4.0);
    server.shutdown().unwrap();
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("characters/2.json")).unwrap()).unwrap();
    assert_eq!(saved["character"]["position"][0], 4.0);
}

#[test]
#[ignore = "requires OSFR_AUDIT_CONTENT pointing at a scratch copy of round70 content"]
fn audit_content_social_delivers_without_trace() {
    for outer in [5, 6] {
        let temp = TempDir::new();
        let content = PathBuf::from(std::env::var_os("OSFR_AUDIT_CONTENT").unwrap());
        let (mut server, store_path) = start_server_with_content(&temp, Some(content));
        let store = AccountStore::load(&store_path).unwrap();
        let first_name = &store.character(2).unwrap().identity;
        let second_name = &store.character(3).unwrap().identity;
        let reference = PathBuf::from(std::env::var_os("OSFR_AUDIT_REFERENCE").unwrap());
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
        let version = manifest["client_version"].as_str().unwrap();
        let (mut first, first_login) = log_in_version(&mut server, 221, 2, &store_path, version);
        let (mut second, second_login) = log_in_version(&mut server, 222, 3, &store_path, version);
        assert!(
            first_login.iter().any(|m| is_opcode(m, 12)),
            "first login failed"
        );
        assert!(
            second_login.iter().any(|m| is_opcode(m, 12)),
            "second login failed"
        );
        first.drain(&mut server);
        let say = format!("{:032x}", 0x15u128);
        first.send(
            &mut server,
            &application(outer, &chat_request(0, &say, None)),
        );
        let delivered = second.drain(&mut server);
        assert!(
            delivered
                .iter()
                .any(|m| is_family(m, 15, 1) && m.windows(say.len()).any(|w| w == say.as_bytes())),
            "say outer={outer}"
        );
        let tell = format!("{:032x}", u128::MAX);
        second.send(
            &mut server,
            &application(
                outer,
                &chat_request(
                    1,
                    &tell,
                    Some((&first_name.first_name, &first_name.last_name)),
                ),
            ),
        );
        let delivered = first.drain(&mut server);
        assert!(delivered.iter().any(|m| is_family(m, 15, 1)
            && m.windows(tell.len()).any(|w| w == tell.as_bytes())), "tell outer={outer}");
        first.send(
            &mut server,
            &application(
                outer,
                &friend_request(&second_name.first_name, &second_name.last_name),
            ),
        );
        assert!(
            second
                .drain(&mut server)
                .iter()
                .any(|m| is_family(m, 26, 16)),
            "friend request outer={outer}"
        );
        first.send(&mut server, &application(outer, &group_invite()));
        assert!(
            second
                .drain(&mut server)
                .iter()
                .any(|m| is_family(m, 40, 1)),
            "group invite outer={outer}"
        );
        server.shutdown().unwrap();
    }
}

#[test]
#[ignore = "requires OSFR_AUDIT_CONTENT and OSFR_AUDIT_REFERENCE scratch copies"]
fn audit_two_bots_deliver_say_and_tell_with_content() {
    use std::process::{Child, Command, Stdio};
    use std::time::Instant;

    // Own every helper handle, including cleanup after a failed assertion.
    struct Helper(Child);
    impl Drop for Helper {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let temp = TempDir::new();
    let content = PathBuf::from(std::env::var_os("OSFR_AUDIT_CONTENT").unwrap());
    let reference = PathBuf::from(std::env::var_os("OSFR_AUDIT_REFERENCE").unwrap());
    let store_path = create_account_store(&temp.0);
    let key_path = temp.0.join("handoff.key");
    fs::write(&key_path, b"synthetic-handoff-key").unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap().to_string();
    drop(socket);
    let output = std::env::var_os("OSFR_AUDIT_EVIDENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| temp.0.clone());
    fs::create_dir_all(&output).unwrap();
    let gateway_log = output.join("bot-gateway.txt");
    let log = fs::File::create(&gateway_log).unwrap();
    let mut gateway = Helper(
        Command::new(env!("CARGO_BIN_EXE_osfr-gateway"))
            .arg(&address)
            .arg("--reference")
            .arg(&reference)
            .arg("--state")
            .arg(temp.0.join("gateway.json"))
            .arg("--content")
            .arg(&content)
            .arg("--account-store")
            .arg(&store_path)
            .arg("--handoff-key-file")
            .arg(&key_path)
            .args(["--studio-port", "0", "--seconds", "30"])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while !fs::read_to_string(&gateway_log)
        .unwrap()
        .contains("\"event\":\"listening\"")
    {
        assert!(
            gateway.0.try_wait().unwrap().is_none(),
            "{}",
            fs::read_to_string(&gateway_log).unwrap()
        );
        assert!(Instant::now() < deadline, "scratch gateway did not listen");
        std::thread::sleep(Duration::from_millis(20));
    }
    let store = AccountStore::load(&store_path).unwrap();
    let target = &store.character(2).unwrap().identity;
    let say = format!("{:032x}", 0x15u128);
    let tell = format!("{:032x}", u128::MAX);
    let tell_wire = chat_request(1, &tell, Some((&target.first_name, &target.last_name)))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut bots = Vec::new();
    for (guid, commands) in [
        (
            3,
            vec![
                format!("expect-text {say}"),
                format!("raw {tell_wire}"),
                "wait 300".into(),
                "quit".into(),
            ],
        ),
        (
            2,
            vec![
                format!("chat {say}"),
                format!("expect-text {say}"),
                format!("expect-text {tell}"),
                "quit".into(),
            ],
        ),
    ] {
        let log_path = output.join(format!("bot-{guid}.txt"));
        let log = fs::File::create(&log_path).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-bot"));
        command
            .arg("--gateway")
            .arg(&address)
            .arg("--reference")
            .arg(&reference)
            .arg("--account-store")
            .arg(&store_path)
            .arg("--handoff-key-file")
            .arg(&key_path)
            .arg("--guid")
            .arg(guid.to_string())
            .args(["--expect-timeout", "12000"]);
        for step in commands {
            command.arg("--cmd").arg(step);
        }
        bots.push((
            Helper(
                command
                    .stdout(Stdio::from(log.try_clone().unwrap()))
                    .stderr(Stdio::from(log))
                    .spawn()
                    .unwrap(),
            ),
            log_path,
        ));
        if guid == 3 {
            // Bot::login clears its inbox. Start the sender only after the
            // receiver has finished login and can retain the say notification.
            let deadline = Instant::now() + Duration::from_secs(12);
            let (receiver, log_path) = bots.last_mut().unwrap();
            while !fs::read_to_string(&log_path).unwrap().contains("in world") {
                assert!(
                    receiver.0.try_wait().unwrap().is_none(),
                    "{}",
                    fs::read_to_string(&log_path).unwrap()
                );
                assert!(Instant::now() < deadline, "receiver did not finish login");
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(20);
    for (bot, log) in &mut bots {
        loop {
            if let Some(status) = bot.0.try_wait().unwrap() {
                assert!(status.success(), "{}", fs::read_to_string(&log).unwrap());
                break;
            }
            assert!(Instant::now() < deadline, "bot exceeded bounded runtime");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let first = fs::read_to_string(output.join("bot-2.txt")).unwrap();
    let second = fs::read_to_string(output.join("bot-3.txt")).unwrap();
    assert!(first.contains(&format!("ok expect-text {say:?}")));
    assert!(first.contains(&format!("ok expect-text {tell:?}")));
    assert!(second.contains(&format!("ok expect-text {say:?}")));
    println!(
        "two osfr-bot processes: say received by both; tell received by target; tracing off; Studio disabled"
    );
}

/// Security Model L6/L9 (W91): one account's social actions share a 5/s, burst-10
/// budget; the excess is dropped silently and counted in the security log.
#[test]
fn an_account_past_its_action_budget_is_dropped_and_logged() {
    let temp = TempDir::new();
    let (mut server, store) = start_server(&temp);
    let (mut first, _) = log_in(&mut server, 81, 2, &store);
    let (_second, _) = log_in(&mut server, 82, 3, &store);
    // Ignore toggles always answer the sender (112/2 add, 112/3 remove), so silence means a drop.
    let start = std::time::Instant::now();
    let replies: Vec<usize> = (0..30)
        .map(|n| {
            first
                .send(
                    &mut server,
                    &application(6, &ignore_request(n % 2 == 0, "Beryl", "Two")),
                )
                .iter()
                .filter(|m| is_family_byte_sub(m, 112, 2) || is_family_byte_sub(m, 112, 3))
                .count()
        })
        .collect();
    let refill = start.elapsed().as_secs_f64() * 5.0;
    server.flush_security();
    let log = fs::read_to_string(temp.0.join("security.jsonl")).unwrap();
    assert!(replies[..10].iter().all(|n| *n > 0), "{replies:?}");
    let accepted = replies.iter().sum::<usize>() as f64;
    // The bucket refills 5/s: a run slowed by a loaded machine earns tokens
    // back, so the hard bound is burst + refill, and a refusal must show
    // only when the run was fast enough to exhaust the bucket.
    assert!(
        accepted <= 10.0 + refill.ceil(),
        "{replies:?} refill {refill}"
    );
    if refill < 15.0 {
        assert!(accepted < 30.0, "{replies:?}");
        assert!(log.contains(r#""kind":"budget_refusal""#), "{log}");
    }
    assert!(!log.contains("Beryl"), "names never reach the log");
}

/// Security Model L6: the gateway admits at most 16 sessions (pending included).
#[test]
fn the_seventeenth_connection_is_refused() {
    let temp = TempDir::new();
    let (mut server, _) = start_server(&temp);
    let admitted: Vec<Client> = (0..16)
        .map(|n| Client::connect(&mut server, 100 + n))
        .collect();
    assert_eq!(server.active_session_count(), 16);
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let mut request = vec![0, 1];
    request.extend_from_slice(&3u32.to_be_bytes());
    request.extend_from_slice(&999u32.to_be_bytes());
    request.extend_from_slice(&512u32.to_be_bytes());
    request.extend_from_slice(b"CGAPI_527 ");
    socket
        .send_to(&request, server.local_addr().unwrap())
        .unwrap();
    server.step().unwrap();
    assert!(
        socket.recv_from(&mut [0; 64]).is_err(),
        "no handshake reply past the cap"
    );
    assert_eq!(server.active_session_count(), 16);
    drop(admitted);
}

/// A connection that never logs in is dropped after 5 s; KeepAlive or not,
/// a logged-in player is unaffected.
#[test]
fn a_connection_that_never_logs_in_is_dropped() {
    let temp = TempDir::new();
    let (mut server, store) = start_server(&temp);
    let (_player, _) = log_in(&mut server, 71, 2, &store);
    let _lurker = Client::connect(&mut server, 72);
    assert_eq!(server.active_session_count(), 2);
    std::thread::sleep(Duration::from_millis(5_200));
    server.step().unwrap();
    assert_eq!(server.active_session_count(), 1);
}

/// A player whose game exits without a goodbye must not take the server down:
/// Windows reports the next datagram to their closed port as ConnectionReset.
#[test]
fn a_vanished_client_does_not_stop_the_server() {
    let temp = TempDir::new();
    let (mut server, store) = start_server(&temp);
    let (mut first, _) = log_in(&mut server, 61, 2, &store);
    let (second, _) = log_in(&mut server, 62, 3, &store);
    drop(second);
    let mut fields = 2u64.to_le_bytes().to_vec();
    for value in [4.0f32, 2.0, 3.0, 0.0, 0.0, 1.0] {
        fields.extend_from_slice(&value.to_le_bytes());
    }
    fields.extend_from_slice(&[0, 0]);
    first.send(&mut server, &packet(125, &fields)); // relayed to the closed port
    // Every step() here unwraps: the reset must not end the server loop.
    first.send(
        &mut server,
        &application(6, &chat_request(0, "still here", None)),
    );
    assert_eq!(server.active_session_count(), 2); // the quiet one times out later
}

#[test]
fn two_clients_share_world_routing_but_keep_their_sessions_and_saves() {
    let temp = TempDir::new();
    let (mut server, account_store) = start_server(&temp);

    let mut first = Client::connect(&mut server, 51);
    let mut first_login = first.send(&mut server, &login(2, &account_store));
    first_login.extend(first.send(&mut server, &packet(13, &[])));
    first_login.extend(first.send(&mut server, &packet(10, &[])));
    assert!(first_login.iter().any(|message| is_opcode(message, 12)));

    let mut second = Client::connect(&mut server, 52);
    let mut second_login = second.send(&mut server, &login(3, &account_store));
    second_login.extend(second.send(&mut server, &packet(13, &[])));
    second_login.extend(second.send(&mut server, &packet(10, &[])));
    let first_spawn = first.drain(&mut server);
    assert!(first_spawn.iter().any(|message| is_family(message, 35, 1)));
    assert!(second_login.iter().any(|message| is_family(message, 35, 1)));
    assert_eq!(server.active_session_count(), 2);

    let move_fields = {
        let mut fields = 3u64.to_le_bytes().to_vec();
        for value in [3.0f32, 2.0, 3.0, 0.0, 0.0, 1.0] {
            fields.extend_from_slice(&value.to_le_bytes());
        }
        fields.extend_from_slice(&[0, 0]);
        fields
    };
    let mut moved = second.send(&mut server, &packet(125, &move_fields));
    moved.extend(first.drain(&mut server));
    assert!(moved.iter().any(|message| is_opcode(message, 125)));

    // Build Plan 1.3: the other player's AddPc carries their real name, and a
    // flags-only movement change still reaches the other client.
    let sees_name = |messages: &[Vec<u8>], name: &[u8]| {
        messages
            .iter()
            .any(|m| is_family(m, 35, 1) && m.windows(name.len()).any(|w| w == name))
    };
    assert!(sees_name(&second_login, b"Aster"));
    assert!(sees_name(&first_spawn, b"Beryl"));
    let mut state_only = move_fields.clone();
    let end = state_only.len();
    state_only[end - 2] = 1;
    second.send(&mut server, &packet(125, &state_only));
    let relayed = first.drain(&mut server);
    assert!(
        relayed
            .iter()
            .any(|m| is_opcode(m, 125) && m.ends_with(&[1, 0])),
        "state-only change was not relayed"
    );

    let say = chat_request(0, "synthetic say", None);
    let mut say_delivery = first.send(&mut server, &application(6, &say));
    say_delivery.extend(second.drain(&mut server));
    assert!(say_delivery.iter().any(|message| is_family(message, 15, 1)));

    let tell = chat_request(1, "synthetic tell", Some(("Aster", "One")));
    let mut tell_delivery = second.send(&mut server, &application(6, &tell));
    tell_delivery.extend(first.drain(&mut server));
    assert!(
        tell_delivery
            .iter()
            .any(|message| is_family(message, 15, 1))
    );

    let friend_notice = second.send(
        &mut server,
        &application(6, &friend_request("Aster", "One")),
    );
    assert!(
        friend_notice
            .iter()
            .any(|message| is_family_byte_sub(message, 74, 8))
    );
    let friend_invite = first.drain(&mut server);
    assert!(
        friend_invite
            .iter()
            .any(|message| is_family(message, 26, 16))
    );
    let mut friend_added = first.send(&mut server, &application(6, &friend_response(3, 2)));
    friend_added.extend(second.drain(&mut server));
    assert!(
        friend_added
            .iter()
            .any(|message| is_family_byte_sub(message, 74, 6))
    );
    for guid in [2, 3] {
        let path = temp
            .0
            .join("characters")
            .join(format!("{guid}.social.json"));
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert!(saved["profile"]["friends"].as_object().unwrap().len() == 1);
    }

    let ignored = first.send(
        &mut server,
        &application(6, &ignore_request(true, "Beryl", "Two")),
    );
    assert!(
        ignored
            .iter()
            .any(|message| is_family_byte_sub(message, 112, 2))
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("characters/2.social.json")).unwrap())
            .unwrap();
    assert!(saved["profile"]["ignored"].get("3").is_some());
    let blocked_tell = second.send(
        &mut server,
        &application(
            6,
            &chat_request(1, "synthetic ignored tell", Some(("Aster", "One"))),
        ),
    );
    assert!(!blocked_tell.iter().any(|message| is_family(message, 15, 1)));
    let unignored = first.send(
        &mut server,
        &application(6, &ignore_request(false, "Beryl", "Two")),
    );
    assert!(
        unignored
            .iter()
            .any(|message| is_family_byte_sub(message, 112, 3))
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.0.join("characters/2.social.json")).unwrap())
            .unwrap();
    assert!(saved["profile"]["ignored"].as_object().unwrap().is_empty());

    let invite = group_invite();
    first.send(&mut server, &application(6, &invite));
    let pending = second.drain(&mut server);
    let invite_id = group_invite_wire::invite_generation(
        pending
            .iter()
            .find(|message| is_family(message, 40, 1))
            .expect("target received the 40/1 invite"),
    );
    let accepted = second.send(&mut server, &application(6, &group_accept(invite_id)));
    let mut group_updates = pending;
    group_updates.extend(accepted);
    group_updates.extend(first.drain(&mut server));
    group_updates.extend(second.drain(&mut server));
    assert!(
        group_updates
            .iter()
            .any(|message| is_family(message, 40, 8))
    );
    assert!(
        group_updates
            .iter()
            .any(|message| is_family(message, 40, 9))
    );

    let per_character = temp.0.join("characters");
    assert!(per_character.join("2.json").is_file());
    assert!(per_character.join("3.json").is_file());
    assert!(per_character.join("2.social.json").is_file());
    assert!(per_character.join("3.social.json").is_file());

    let mut disconnect = first.connection_id.to_be_bytes().to_vec();
    disconnect.extend_from_slice(&[0, 0]);
    let wire = first.codec.encode(5, &disconnect).unwrap();
    first.socket.send_to(&wire, first.server).unwrap();
    server.step().unwrap();
    let disconnected = second.drain(&mut server);
    assert!(disconnected.iter().any(|message| is_family(message, 35, 3)));
    assert!(
        disconnected
            .iter()
            .any(|message| is_family_byte_sub(message, 74, 3))
    );
    assert_eq!(server.active_session_count(), 1);
}

// ---------------------------------------------------------------------------
// T335: the two-player playtest loop (visibility, group, chat, shared kill,
// two saves). The field tests run on this same real `Server`/`Client` harness;
// they add one scratch content tree built from the opening fixture plus the
// repository content (the T301/T256 pattern) and one synthetic defeat quest,
// so the quest id and reward values are test content, not retail values.
//
// Client evidence (2014 client, `lab/autopilot/ida_query.py`): AddPc
// constructor `0x0091d000` (the vtable slot invoked from `0x0092feca`), 26/8
// `0x008b6bc0`, 26/10 `0x00a36f80`, 26/6 `0x009f5890`, 36/10 writer
// `0x00a342e0` / request path `0x00a35150`, 49/9 `0x00c7af60`, 49/10
// `0x00c7b070`, 49/13 `0x00c7b990`. The family-40 shapes come from
// `tests/common/group_invite_wire.rs` and `notes/Research/Friends and
// Groups.md`; no new wire shape is introduced.
// ---------------------------------------------------------------------------

#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;

use field_reward_fixture::RewardFixture;
use npc_quest_chain::{
    QUEST_CHOICE, TALK_LABEL, dialogue_choice, npc_interact, npc_select, quest_end_reply,
    quest_reply, select_job,
};

const T335_GIVER: u64 = 9_700_000_335;
const T335_QUEST: u32 = 3_999_990;
const T335_WOLF_DEFINITION: i32 = 3_156;
const T335_WOLF_ONE: u64 = 131_000_335_001;
const T335_WOLF_TWO: u64 = 131_000_335_002;
const T335_WOLF_THREE: u64 = 131_000_335_003;
const T335_WOLF_FOUR: u64 = 131_000_335_004;
const T335_BRAWLER: i32 = 43;
const T335_ITEM: i32 = 8;
const T335_QUEST_COINS: i32 = 10;
const T335_QUEST_XP: u32 = 20;
/// `data/content/field_kill_rewards.json` level 1-2 band.
const T335_FIELD_COINS: i32 = 2;
const T335_FIELD_XP: i32 = 5;
/// The Medic combat profile: B's active job, distinct from A's Brawler (43).
const T335_MEDIC: i32 = 11;
const T335_TITLE: u32 = 9_700_351;
const T335_END_TEXT: u32 = 9_700_352;
/// The opening fixture's `encounters.json` world pose (zone 0), which
/// `Gameplay::initial_character` gives every new session.
const T335_START: [f32; 3] = [-1904.883, -39.7098, 412.6024];
const T335_PATIENCE: Duration = Duration::from_secs(30);

/// One private gameplay content tree: the opening fixture's profile,
/// inventory and opening files, the repository's zone/job/field-reward files,
/// then the synthetic T335 quest giver and four field wolves.
struct PlaytestContent {
    fixture: RewardFixture,
}

impl PlaytestContent {
    fn new() -> Self {
        // The opening fixture plus only the files this loop reads. The full
        // bundled overlay costs ~20 s per content login and would bind the
        // opening quest story into the exact reward deltas.
        let fixture = RewardFixture::minimal(
            t335_actors(),
            &[
                "zones.json",
                "jobs.json",
                "field_kill_rewards.json",
                "npc_spawns_extra.json",
            ],
        );
        let _ = fs::create_dir_all(fixture.dir().join("quests"));
        let _ = fs::create_dir_all(fixture.dir().join("npcs"));
        write_json(
            &fixture.dir().join("stores.json"),
            json!({"schema_version": 1, "marketplace": {
                "enabled_on_handshake": false, "orders_enabled": false,
                "payment_sources_enabled": false, "not_available_message": "disabled"
            }, "items": []}),
        );
        write_json(
            &fixture.dir().join("vendors.json"),
            json!({"schema_version": 1, "vendors": []}),
        );
        write_json(
            &fixture.dir().join("quests/t335.json"),
            t335_quest_content(),
        );
        write_json(&fixture.dir().join("npcs/t335.json"), t335_giver_content());
        let jobs_path = fixture.dir().join("jobs.json");
        let mut jobs = npc_quest_chain::read_json(&jobs_path);
        // The split loader requires every `job.quest_ids` entry to have a quest
        // definition. Only this loop's synthetic quest is in the fixture, so
        // clear the shipped references this test does not copy.
        for job in jobs["records"]
            .as_array_mut()
            .expect("jobs.json records")
            .iter_mut()
        {
            job["quest_ids"] = json!([]);
        }
        let brawler = jobs["records"]
            .as_array_mut()
            .expect("jobs.json records")
            .iter_mut()
            .find(|job| job["id"] == "brawler")
            .expect("the Brawler job row");
        brawler["quest_ids"] = json!([T335_QUEST]);
        write_json(&jobs_path, jobs);
        Self { fixture }
    }

    fn path(&self) -> &Path {
        self.fixture.dir()
    }
}

fn write_json(path: &Path, value: serde_json::Value) {
    fs::write(
        path,
        serde_json::to_vec(&value).expect("fixture JSON serializes"),
    )
    .unwrap_or_else(|error| panic!("fixture {} writes: {error}", path.display()));
}

fn t335_source() -> serde_json::Value {
    json!([{
        "kind": "reconstructed",
        "evidence": "(R) T335 synthetic two-player acceptance content: test-only values, not recovered retail rewards."
    }])
}

/// A stationary field wolf. Damage 0 and a long attack clock keep the two-
/// player loop deterministic; ownership of the kill is what the test varies.
fn t335_wolf(guid: u64, position: [f32; 3]) -> serde_json::Value {
    json!({
        "guid": guid,
        "definition_id": T335_WOLF_DEFINITION,
        "model_id": 3284,
        "name_id": 5_100_200,
        "name": "Synthetic T335 wolf",
        "texture_alias": "",
        "position": position,
        "max_health": 200,
        "damage": 0,
        "boss": false,
        "level": 1,
        "respawn_seconds": 600,
        "aggro_range": 8.0,
        "leash_radius": 12.0,
        "behaviour": {"mode": "stationary", "reach": 2.5, "attack_ms": 1_000_000},
        "quest_ids": [],
        "source": t335_source()
    })
}

fn t335_actors() -> Vec<serde_json::Value> {
    vec![
        t335_wolf(
            T335_WOLF_ONE,
            [T335_START[0], T335_START[1], T335_START[2] + 1.0],
        ),
        t335_wolf(
            T335_WOLF_TWO,
            [T335_START[0] + 1.0, T335_START[1], T335_START[2]],
        ),
        t335_wolf(
            T335_WOLF_THREE,
            [T335_START[0], T335_START[1], T335_START[2] - 1.0],
        ),
        t335_wolf(
            T335_WOLF_FOUR,
            [T335_START[0] - 1.0, T335_START[1], T335_START[2]],
        ),
    ]
}

/// One synthetic defeat quest: one wolf defeat, 10 coins, 20 Brawler job XP
/// and one item 8, then a Talk to the giver for the turn-in.
fn t335_quest_content() -> serde_json::Value {
    json!({
        "schema_version": 1,
        "records": [{
            "id": T335_QUEST,
            "title_string_id": T335_TITLE,
            "end_text_string_id": T335_END_TEXT,
            "giver_npc_id": T335_GIVER,
            "prerequisite_quest_ids": [],
            "objectives": [{
                "kind": "defeat",
                "id": 1,
                "model_id": T335_WOLF_DEFINITION,
                "count": 1,
                "source": t335_source()
            }],
            "rewards": [{
                "coins": T335_QUEST_COINS,
                "items": [{"definition_id": T335_ITEM, "count": 1, "source": t335_source()}],
                "job_xp": {"43": T335_QUEST_XP},
                "source": t335_source()
            }],
            "source": t335_source(),
            "enabled": true
        }]
    })
}

/// A drawable regional giver: `legacy_data` plus a finite position is the
/// production draw rule, and `npc_id` is the runtime actor id.
fn t335_giver_content() -> serde_json::Value {
    json!({
        "schema_version": 1,
        "records": [{
            "npc_id": T335_GIVER,
            "name": "Synthetic T335 giver",
            "name_string_id": 19_685,
            "zone_id": 0,
            "position": [T335_START[0] + 2.0, T335_START[1], T335_START[2]],
            "placement": "legacy_data",
            "model_id": 958,
            "roles": ["quest_giver"],
            "quest_ids_given": [],
            "quests_given": [],
            "quests_accepted": [],
            "source": t335_source()
        }]
    })
}

/// The captured reference's client version, which its manifest pins and the
/// login request must repeat.
fn field_version() -> String {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(reference.join("manifest.json")).unwrap()).unwrap();
    manifest["client_version"].as_str().unwrap().to_owned()
}

/// A content-backed two-session server: the captured reference, the account
/// store fixture and the private content tree above.
fn start_field_server(temp: &TempDir, content: &Path) -> (Server, PathBuf) {
    // The captured reference's typed self-data carries the configured job
    // rows the content-bound self message patches; the synthetic reference
    // above cannot (it has no profile rows).
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-gateway-reference");
    start_server_at(temp, reference, Some(content.to_owned()))
}

/// Service every client and the server for `duration`, returning each
/// client's drained messages. The server only ticks the shared field on its
/// 20 Hz clock, so a test that waits must keep stepping.
fn pump(server: &mut Server, clients: &mut [&mut Client], duration: Duration) -> Vec<Vec<Vec<u8>>> {
    let mut out = vec![Vec::new(); clients.len()];
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        server.step().unwrap();
        for (index, client) in clients.iter_mut().enumerate() {
            out[index].extend(client.drain(server));
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    out
}

/// Pump until `done` observes the expected server state, or fail after
/// `timeout`. The returned messages are per client, in `clients` order.
fn wait_until(
    server: &mut Server,
    clients: &mut [&mut Client],
    timeout: Duration,
    mut done: impl FnMut(&Server) -> bool,
) -> Vec<Vec<Vec<u8>>> {
    let deadline = Instant::now() + timeout;
    let mut out = vec![Vec::new(); clients.len()];
    loop {
        let batch = pump(server, clients, Duration::from_millis(10));
        for (index, messages) in batch.into_iter().enumerate() {
            out[index].extend(messages);
        }
        if done(server) {
            return out;
        }
        assert!(Instant::now() < deadline, "T335 loop condition not reached");
    }
}

/// Pump every client until the server's reliable queues are idle, so the
/// next blocking content login cannot outlive another session's retries.
fn settle(server: &mut Server, clients: &mut [&mut Client]) -> Vec<Vec<Vec<u8>>> {
    wait_until(server, clients, T335_PATIENCE, |server| {
        server.test_queues_idle()
    })
}

fn knows(server: &Server, guid: u64, actor: u64) -> bool {
    server.test_location(guid).is_some() && server.test_known_npcs(guid).contains(&actor)
}

/// Pump both sessions until client `index` receives `needle`, returning only
/// that client's messages. A sender's own echo cannot satisfy this.
fn wait_for_client_text(
    server: &mut Server,
    clients: &mut [&mut Client],
    index: usize,
    needle: &str,
) -> Vec<Vec<u8>> {
    let deadline = Instant::now() + T335_PATIENCE;
    let mut seen = Vec::new();
    loop {
        let batch = pump(server, clients, Duration::from_millis(20));
        seen.extend(batch[index].clone());
        if has_text(&seen, needle) {
            return seen;
        }
        assert!(
            Instant::now() < deadline,
            "client {index} did not receive `{needle}`"
        );
    }
}

/// The full application body after the 7-byte tunnel header.
fn family_body(message: &[u8], family: u16, sub: u16) -> Option<&[u8]> {
    (message.len() >= 11
        && message[..3] == [5, 0, 1]
        && u16::from_le_bytes(message[7..9].try_into().unwrap()) == family
        && u16::from_le_bytes(message[9..11].try_into().unwrap()) == sub)
        .then(|| &message[7..])
}

/// One tunneled gameplay request (outer 5).
fn app(client: &mut Client, server: &mut Server, inner: &[u8]) -> Vec<Vec<u8>> {
    client.send(server, &application(5, inner))
}

#[derive(Debug, PartialEq)]
struct AddPc {
    character_guid: u64,
    first: String,
    last: String,
    model: i32,
    active_profile: i32,
    wield_type: i32,
}

fn read_len_string(bytes: &[u8], at: &mut usize) -> Option<String> {
    let length = i32::from_le_bytes(bytes.get(*at..*at + 4)?.try_into().ok()?);
    let length = usize::try_from(length).ok()?;
    *at += 4;
    let text = std::str::from_utf8(bytes.get(*at..*at + length)?)
        .ok()?
        .to_owned();
    *at += length;
    Some(text)
}

/// Read 35/1 AddPc through the real body reader `0x0091aa10` (base header
/// `0x008d6830`; guid +16, names via `0x008e7290`, model +160, active profile
/// +364): header, guid, three name ids, first/last name, model, three bubble
/// fields, pose, attachments, the appearance block, speed and flags, temporary
/// model, guild, active profile, titles, effects, the mount block and the wield
/// type (+420, `0x0091aee4`). `players.rs::build_add_pc` writes that order.
/// (R) The parser assumes the guild and effect rows are empty, as the current
/// writer emits them.
fn parse_add_pc(message: &[u8]) -> Option<AddPc> {
    let body = family_body(message, 35, 1)?;
    let character_guid = u64::from_le_bytes(body.get(4..12)?.try_into().ok()?);
    let mut at = 12 + 12; // three name ids
    let first = read_len_string(body, &mut at)?;
    let last = read_len_string(body, &mut at)?;
    let model = i32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?);
    at += 4 + 12 + 16 + 16; // three bubble fields, position, rotation
    let rows = u32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?);
    at += 4;
    for _ in 0..rows {
        for _ in 0..3 {
            read_len_string(body, &mut at)?;
        }
        at += 12;
    }
    read_len_string(body, &mut at)?; // head
    read_len_string(body, &mut at)?; // hair
    at += 4 + 4 + 4; // hair colour, eye colour, unknown3
    read_len_string(body, &mut at)?; // skin tone
    read_len_string(body, &mut at)?; // face paint
    read_len_string(body, &mut at)?; // model customisation
    at += 4 + 3 + 4 + 4; // speed, three flags, temporary model, guild
    let active_profile = i32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?);
    at += 4 + 16 + 4 + 20; // profile, four title dwords, effect count, mount block
    let wield_type = i32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?);
    Some(AddPc {
        character_guid,
        first,
        last,
        model,
        active_profile,
        wield_type,
    })
}

/// 36/10 `ClientRequestStartAbility`: bar, slot, target mode then the target
/// guid (writer `0x00a342e0`; the same 24-byte body `shared_field_state` pins).
fn field_attack(client: &mut Client, server: &mut Server, target: u64) -> Vec<Vec<u8>> {
    let mut body = vec![10, 0];
    for value in [1i32, 0, 0] {
        body.extend(value.to_le_bytes());
    }
    body.extend(target.to_le_bytes());
    client.send(server, &packet(36, &body))
}

/// The selected option id for `label` in the 26/9 interaction menu, proving the
/// menu names the clicked actor. The row walk is the shared
/// `npc_quest_chain::menu_option_bytes`, which cites the client readers.
fn menu_choice(messages: &[Vec<u8>], target: u64, label: u32) -> i32 {
    let body = messages
        .iter()
        .find_map(|message| family_body(message, 26, 9))
        .expect("26/8 returns the 26/9 interaction menu");
    assert_eq!(
        body.get(4..12),
        Some(target.to_le_bytes().as_slice()),
        "the menu names the clicked actor"
    );
    npc_quest_chain::menu_option_bytes(body, label)
}

fn save_json(temp: &TempDir, guid: u64) -> serde_json::Value {
    serde_json::from_slice(
        &fs::read(temp.0.join(format!("characters/{guid}.json")))
            .expect("the character save exists"),
    )
    .expect("the character save is JSON")
}

fn saved_coins(temp: &TempDir, guid: u64) -> i32 {
    i32::try_from(
        save_json(temp, guid)["gameplay"]["inventory"]["coins"]
            .as_i64()
            .expect("the saved coin balance"),
    )
    .unwrap()
}

fn saved_job_xp(temp: &TempDir, guid: u64, profile: i32) -> i32 {
    i32::try_from(
        save_json(temp, guid)["gameplay"]["profiles"]["experience"][profile.to_string()]
            .as_i64()
            .unwrap_or(0),
    )
    .unwrap()
}

fn quest_counts(save: &serde_json::Value, quest: u32) -> Vec<u32> {
    save["gameplay"]["npc_quests"]["quests"][quest.to_string()]["counts"]
        .as_array()
        .unwrap_or_else(|| panic!("quest {quest} counts are saved"))
        .iter()
        .map(|count| u32::try_from(count.as_u64().unwrap()).unwrap())
        .collect()
}

/// Every 38/19 coin balance delivered to one client, in arrival order.
fn coin_balances(messages: &[Vec<u8>]) -> Vec<i32> {
    messages
        .iter()
        .filter_map(|message| {
            let body = family_body(message, 38, 19)?;
            Some(i32::from_le_bytes(body.get(4..8)?.try_into().ok()?))
        })
        .collect()
}

/// Every 38/14 update for `profile` as `(rank, progress)`. Reader `008d86e0`
/// reads profile, rank and per-rank progress; the third field is not the saved
/// total, which is `rank * 100 + progress` on the local curve.
fn job_updates(messages: &[Vec<u8>], profile: i32) -> Vec<(i32, i32)> {
    messages
        .iter()
        .filter_map(|message| {
            let body = family_body(message, 38, 14)?;
            if i32::from_le_bytes(body.get(4..8)?.try_into().ok()?) != profile {
                return None;
            }
            Some((
                i32::from_le_bytes(body.get(8..12)?.try_into().ok()?),
                i32::from_le_bytes(body.get(12..16)?.try_into().ok()?),
            ))
        })
        .collect()
}

/// The (rank, progress) pair a 38/14 must carry for a saved total: the same
/// reconstructed 100-XP curve and rank-20 cap as `Profiles::rank`
/// (`src/profiles.rs:228`) and `rewards::experience_packets` (`src/rewards.rs:177`).
fn expected_job_update(total: i32) -> (i32, i32) {
    let rank = (1 + total / 100).min(20);
    (rank, if rank == 20 { 0 } else { total % 100 })
}

fn objective_update(messages: &[Vec<u8>], quest: u32, objective: u32, current: u32) -> bool {
    messages.iter().any(|message| {
        // 49/9 reader `0x00c7af60` runs through the quest base header
        // `0x00c7acd0` (u16 family + i32 subtype), so the quest id is at 6.
        let Some(body) = family_body(message, 49, 9) else {
            return false;
        };
        u32::from_le_bytes(body.get(6..10).unwrap().try_into().unwrap()) == quest
            && u32::from_le_bytes(body.get(10..14).unwrap().try_into().unwrap()) == objective
            && u32::from_le_bytes(body.get(14..18).unwrap().try_into().unwrap()) == current
    })
}

/// Switch one session to `profile` through the client's own 26/13 request; the
/// real path also grants that job's starter gear, and the commit saves the new
/// active profile before the reply lands.
fn become_job(
    client: &mut Client,
    server: &mut Server,
    temp: &TempDir,
    guid: u64,
    profile: i32,
) -> Vec<Vec<u8>> {
    let replies = app(client, server, &select_job(profile));
    assert_eq!(
        save_json(temp, guid)["gameplay"]["profiles"]["active"],
        json!(profile),
        "the job switch commits its active profile"
    );
    replies
}

fn become_brawler(
    client: &mut Client,
    server: &mut Server,
    temp: &TempDir,
    guid: u64,
) -> Vec<Vec<u8>> {
    become_job(client, server, temp, guid, T335_BRAWLER)
}

#[test]
fn two_players_see_each_other_with_the_real_name_and_job() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store_path) = start_field_server(&temp, content.path());
    let store = AccountStore::load(&store_path).unwrap();
    let identity = |guid: u64| {
        let identity = &store.character(guid).unwrap().identity;
        (
            identity.first_name.clone(),
            identity.last_name.clone(),
            identity.model,
        )
    };
    let (b_first, b_last, b_model) = identity(3);
    let (a_first, a_last, a_model) = identity(2);

    let version = field_version();
    let (mut first, mut first_login) =
        log_in_version_batched(&mut server, 401, 2, &store_path, &version);
    let settled = settle(&mut server, &mut [&mut first]);
    first_login.extend(settled[0].clone());
    let (mut second, mut second_login) =
        log_in_version_batched(&mut server, 402, 3, &store_path, &version);
    let settled = settle(&mut server, &mut [&mut first, &mut second]);
    first_login.extend(settled[0].clone());
    second_login.extend(settled[1].clone());
    assert!(
        server.test_location(2).is_some_and(|row| row.2)
            && server.test_location(3).is_some_and(|row| row.2),
        "both sessions are ready in the world"
    );

    let other = |messages: &[Vec<u8>], guid: u64| -> Vec<AddPc> {
        messages
            .iter()
            .filter_map(|message| parse_add_pc(message))
            .filter(|add| add.character_guid == guid)
            .collect()
    };
    let sees_b = other(&first_login, 3);
    assert_eq!(sees_b.len(), 1, "one AddPc for the other player");
    assert_eq!(sees_b[0].first, b_first);
    assert_eq!(sees_b[0].last, b_last);
    assert_eq!(sees_b[0].model, b_model);
    // The public snapshot names the character's own saved active job.
    let b_profile = save_json(&temp, 3)["gameplay"]["profiles"]["active"].clone();
    assert_eq!(json!(sees_b[0].active_profile), b_profile);

    let sees_a = other(&second_login, 2);
    assert_eq!(sees_a.len(), 1, "one AddPc for the other player");
    assert_eq!(sees_a[0].first, a_first);
    assert_eq!(sees_a[0].last, a_last);
    assert_eq!(sees_a[0].model, a_model);

    // A switches job; the current viewer and a later joiner both get the new
    // active profile. Skipping the presentation refresh leaves the late
    // joiner with the pre-switch profile.
    let mut switched = become_brawler(&mut first, &mut server, &temp, 2);
    let settled = settle(&mut server, &mut [&mut first, &mut second]);
    switched.extend(settled[0].clone());
    first_login.extend(settled[0].clone());
    second_login.extend(settled[1].clone());
    assert!(
        switched
            .iter()
            .chain(settled[1].iter())
            .any(|message| family_body(message, 35, 7).is_some()),
        "the current viewer receives the 35/7 job switch"
    );
    assert_eq!(
        save_json(&temp, 2)["gameplay"]["profiles"]["active"],
        json!(T335_BRAWLER)
    );
    let (mut late, mut late_login) =
        log_in_version_batched(&mut server, 403, 1, &store_path, &version);
    let settled = settle(&mut server, &mut [&mut first, &mut second, &mut late]);
    first_login.extend(settled[0].clone());
    second_login.extend(settled[1].clone());
    late_login.extend(settled[2].clone());
    let current = other(&late_login, 2);
    assert_eq!(current.len(), 1, "the late joiner gets one current AddPc");
    assert_eq!(current[0].active_profile, T335_BRAWLER);
    assert_eq!(current[0].first, a_first);
    assert_eq!(current[0].model, a_model);
    let late_sees_b = other(&late_login, 3);
    assert_eq!(late_sees_b.len(), 1, "the late joiner gets one AddPc for B");
    assert_eq!(late_sees_b[0].first, b_first);
    assert_eq!(late_sees_b[0].model, b_model);
    assert_eq!(json!(late_sees_b[0].active_profile), b_profile);

    // A fourth session 10 km away is outside the 100 m interest radius: it
    // receives neither player's AddPc and the pair receives none for it. The
    // far login is asserted ready first, so a refused session cannot make the
    // "no AddPc" checks below pass vacuously; B's inbox is inspected too.
    seed_far_character(&temp, content.path(), 4);
    let (mut far, mut far_login) =
        log_in_version_batched(&mut server, 404, 4, &store_path, &version);
    let settled = settle(
        &mut server,
        &mut [&mut first, &mut second, &mut late, &mut far],
    );
    first_login.extend(settled[0].clone());
    second_login.extend(settled[1].clone());
    late_login.extend(settled[2].clone());
    far_login.extend(settled[3].clone());
    assert_eq!(
        server.active_session_count(),
        4,
        "the far session did not log in"
    );
    let (_, _, far_ready, far_instance) = server
        .test_location(4)
        .expect("the far session is in the world");
    assert!(far_ready, "the far session never became ready in the world");
    // The pair's own instance, so distance alone keeps the AddPc rows apart.
    assert!(
        far_instance.contains(&2) && far_instance.contains(&3),
        "the far session is not in the pair's instance"
    );
    assert!(
        far_login.iter().any(|message| is_opcode(message, 12)),
        "the far session received no world-entry self data"
    );
    assert!(
        other(&far_login, 2).is_empty(),
        "an out-of-range player received A's AddPc"
    );
    assert!(
        other(&far_login, 3).is_empty(),
        "an out-of-range player received B's AddPc"
    );
    assert!(
        other(&first_login, 4).is_empty(),
        "A received an out-of-range player's AddPc"
    );
    assert!(
        other(&second_login, 4).is_empty(),
        "B received an out-of-range player's AddPc"
    );
    assert!(
        other(&late_login, 4).is_empty(),
        "the late joiner received an out-of-range player's AddPc"
    );
}

#[test]
fn a_shared_field_kill_pays_both_and_credits_the_killer_once() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    // B's active job is Medic (11), not A's Brawler (43), so a payout that
    // names the killer's profile instead of the recipient's is visible.
    seed_second_character(&temp, content.path(), T335_MEDIC, false);
    let (mut server, store_path) = start_field_server(&temp, content.path());
    let version = field_version();
    let (mut first, _) = log_in_version_batched(&mut server, 411, 2, &store_path, &version);
    settle(&mut server, &mut [&mut first]);
    let (mut second, _) = log_in_version_batched(&mut server, 412, 3, &store_path, &version);
    settle(&mut server, &mut [&mut first, &mut second]);
    {
        let mut clients = [&mut first, &mut second];
        wait_until(&mut server, &mut clients, T335_PATIENCE, |server| {
            knows(server, 2, T335_WOLF_ONE) && knows(server, 3, T335_WOLF_ONE)
        });
    }
    become_brawler(&mut first, &mut server, &temp, 2);
    become_job(&mut second, &mut server, &temp, 3, T335_MEDIC);
    settle(&mut server, &mut [&mut first, &mut second]);

    // A accepts the synthetic defeat quest through the ordinary NPC menu.
    let menu = app(&mut first, &mut server, &npc_interact(T335_GIVER));
    let choice = menu_choice(&menu, T335_GIVER, TALK_LABEL);
    app(&mut first, &mut server, &npc_select(T335_GIVER, choice));
    let offer = app(&mut first, &mut server, &dialogue_choice(QUEST_CHOICE));
    assert!(
        offer
            .iter()
            .any(|message| family_body(message, 49, 1).is_some()),
        "the Talk offers the synthetic quest"
    );
    let accepted = app(&mut first, &mut server, &quest_reply(T335_QUEST, true));
    assert!(
        accepted
            .iter()
            .any(|message| family_body(message, 49, 3).is_some()),
        "the accept adds the quest"
    );

    // B wounds the wolf; A lands the lethal hit. One shared death, two
    // participants inside T240's window.
    let (a_coins, b_coins) = (saved_coins(&temp, 2), saved_coins(&temp, 3));
    let (a_xp, b_xp) = (
        saved_job_xp(&temp, 2, T335_BRAWLER),
        saved_job_xp(&temp, 3, T335_MEDIC),
    );
    let mut b_replies = field_attack(&mut second, &mut server, T335_WOLF_ONE);
    let claimed = attack_until_dead_all(
        &mut server,
        &mut [&mut first, &mut second],
        0,
        T335_WOLF_ONE,
    );
    let mut a_replies = claimed[0].clone();
    b_replies.extend(claimed[1].clone());
    let deadline = Instant::now() + T335_PATIENCE;
    while coin_balances(&b_replies).is_empty() {
        assert!(
            Instant::now() < deadline,
            "the contributor's T240 payout did not arrive"
        );
        let more = {
            let mut clients = [&mut first, &mut second];
            pump(&mut server, &mut clients, Duration::from_millis(20))
        };
        a_replies.extend(more[0].clone());
        b_replies.extend(more[1].clone());
    }
    // Let both shared ticks finish, so a duplicate credit from the killer
    // would still be in the collected messages and the save.
    let after = settle(&mut server, &mut [&mut first, &mut second]);
    a_replies.extend(after[0].clone());
    b_replies.extend(after[1].clone());
    assert_eq!(server.test_field_health(T335_WOLF_ONE), Some(0));
    assert_eq!(
        saved_coins(&temp, 2),
        a_coins + T335_FIELD_COINS,
        "the killer's saved balance changed exactly once"
    );
    assert_eq!(
        saved_coins(&temp, 3),
        b_coins + T335_FIELD_COINS,
        "the contributor's saved balance changed exactly once"
    );
    let a_xp_after = saved_job_xp(&temp, 2, T335_BRAWLER);
    let b_xp_after = saved_job_xp(&temp, 3, T335_MEDIC);
    assert_eq!(a_xp_after, a_xp + T335_FIELD_XP);
    assert_eq!(b_xp_after, b_xp + T335_FIELD_XP);
    assert_eq!(
        coin_balances(&a_replies),
        vec![a_coins + T335_FIELD_COINS],
        "the killer received exactly one payout"
    );
    assert_eq!(
        job_updates(&a_replies, T335_BRAWLER),
        vec![expected_job_update(a_xp_after)],
        "the killer's 38/14 names its active job exactly once"
    );
    assert_eq!(
        coin_balances(&b_replies),
        vec![b_coins + T335_FIELD_COINS],
        "the contributor received exactly one payout"
    );
    assert_eq!(
        job_updates(&b_replies, T335_MEDIC),
        vec![expected_job_update(b_xp_after)],
        "the contributor's 38/14 names its own active job exactly once"
    );
    assert!(
        job_updates(&b_replies, T335_BRAWLER).is_empty(),
        "the contributor's payout must not use the killer's job"
    );
    assert!(
        objective_update(&a_replies, T335_QUEST, 1, 1),
        "the killer's quest advances exactly one kill"
    );
    assert_eq!(quest_counts(&save_json(&temp, 2), T335_QUEST), [1]);
    // T302 may credit the contributor's quest too; do not assert it either way.

    // A replay on the corpse and a second kill attempt pay and credit nothing.
    let replay_a = field_attack(&mut first, &mut server, T335_WOLF_ONE);
    let replay_b = field_attack(&mut second, &mut server, T335_WOLF_ONE);
    assert!(coin_balances(&replay_a).is_empty());
    assert!(coin_balances(&replay_b).is_empty());
    assert!(!objective_update(&replay_a, T335_QUEST, 1, 2));
    assert_eq!(quest_counts(&save_json(&temp, 2), T335_QUEST), [1]);
}

// ---------------------------------------------------------------------------
// T335: the remaining two-player loop tests.
// ---------------------------------------------------------------------------

/// Seed B's own character save with a different active job from A's Brawler
/// and, when `seeded_item`, a distinct starting item stack. Item 8 has
/// `max_stack: 1`, so the stack proves each character keeps its own items.
fn seed_second_character(temp: &TempDir, content: &Path, active: i32, seeded_item: bool) {
    let mut state = osfr_tools::gameplay::Gameplay::load(content, None)
        .expect("the fixture content loads")
        .state();
    state
        .profiles
        .as_mut()
        .expect("the fixture has profile content")
        .unlocked
        .insert(active);
    if seeded_item {
        state
            .inventory
            .items
            .push(osfr_tools::inventory::InventoryItem {
                definition_id: T335_ITEM,
                tint: 0,
                id: 900_001,
                count: 1,
                consumed_count: 0,
                stored_count: 0,
                last_cast_time: 0,
                rental: None,
                ability_count: 0,
                activate_enabled: false,
            });
        state.inventory.next_instance_id = Some(900_002);
    }
    fs::create_dir_all(temp.0.join("characters")).unwrap();
    osfr_tools::game_store::save(
        &temp.0.join("characters/3.json"),
        &osfr_tools::game_store::Snapshot {
            character: CharacterState {
                guid: 3,
                zone_id: 0,
                position: T335_START,
                rotation: [0.0, 0.0, 1.0],
            },
            gameplay: Some(state),
            pets: None,
        },
    )
    .expect("the seeded second save writes");
}

/// Seed a saved character far outside the 100 m interest radius.
fn seed_far_character(temp: &TempDir, content: &Path, guid: u64) {
    let state = osfr_tools::gameplay::Gameplay::load(content, None)
        .expect("the fixture content loads")
        .state();
    fs::create_dir_all(temp.0.join("characters")).unwrap();
    osfr_tools::game_store::save(
        &temp.0.join(format!("characters/{guid}.json")),
        &osfr_tools::game_store::Snapshot {
            character: CharacterState {
                guid,
                zone_id: 0,
                position: [T335_START[0] + 10_000.0, T335_START[1], T335_START[2]],
                rotation: [0.0, 0.0, 1.0],
            },
            gameplay: Some(state),
            pets: None,
        },
    )
    .expect("the far character save writes");
}

fn saved_item_count(temp: &TempDir, guid: u64, definition: i32) -> u64 {
    save_json(temp, guid)["gameplay"]["inventory"]["items"]
        .as_array()
        .expect("the saved inventory items")
        .iter()
        .filter(|item| item["definition_id"] == json!(definition))
        .map(|item| item["count"].as_u64().unwrap_or(0))
        .sum()
}

fn saved_item_ids(temp: &TempDir, guid: u64, definition: i32) -> Vec<i64> {
    let mut ids: Vec<i64> = save_json(temp, guid)["gameplay"]["inventory"]["items"]
        .as_array()
        .expect("the saved inventory items")
        .iter()
        .filter(|item| item["definition_id"] == json!(definition))
        .map(|item| item["id"].as_i64().unwrap())
        .collect();
    ids.sort_unstable();
    ids
}

fn saved_quest_state(temp: &TempDir, guid: u64) -> serde_json::Value {
    save_json(temp, guid)["gameplay"]["npc_quests"].clone()
}

fn has_text(messages: &[Vec<u8>], needle: &str) -> bool {
    let needle = needle.as_bytes();
    messages
        .iter()
        .any(|message| message.windows(needle.len()).any(|window| window == needle))
}

/// The group id snapshot's member GUIDs in roster order (40/8 reader
/// `0093daf0` case 8 -> `0092aed0`: two i16, count, member rows).
fn group_snapshot_guids(messages: &[Vec<u8>]) -> Vec<Vec<u64>> {
    messages
        .iter()
        .filter_map(|message| {
            let body = family_body(message, 40, 8)?;
            let count = i32::from_le_bytes(body.get(8..12)?.try_into().ok()?);
            let mut at = 12;
            let mut guids = Vec::new();
            for _ in 0..count {
                guids.push(u64::from_le_bytes(body.get(at..at + 8)?.try_into().ok()?));
                at += 8 + 12; // guid, three name fields
                for _ in 0..2 {
                    let length = i32::from_le_bytes(body.get(at..at + 4)?.try_into().ok()?);
                    let length = usize::try_from(length).ok()?;
                    at += 4 + length;
                }
                at += 7 * 4 + 2 + 4; // opaque fields, two flags, minigame type
            }
            Some(guids)
        })
        .collect()
}

fn group_result_code(messages: &[Vec<u8>], sub: u16) -> Option<u32> {
    messages.iter().find_map(|message| {
        let body = family_body(message, 40, sub)?;
        Some(u32::from_le_bytes(body.get(4..8)?.try_into().ok()?))
    })
}

fn accept_quest(client: &mut Client, server: &mut Server) -> Vec<Vec<u8>> {
    let mut replies = app(client, server, &npc_interact(T335_GIVER));
    let choice = menu_choice(&replies, T335_GIVER, TALK_LABEL);
    replies.extend(app(client, server, &npc_select(T335_GIVER, choice)));
    let offer = app(client, server, &dialogue_choice(QUEST_CHOICE));
    assert!(
        offer
            .iter()
            .any(|message| family_body(message, 49, 1).is_some()),
        "the Talk offers the synthetic quest"
    );
    replies.extend(offer);
    let accepted = app(client, server, &quest_reply(T335_QUEST, true));
    assert!(
        accepted
            .iter()
            .any(|message| family_body(message, 49, 3).is_some()),
        "the accept adds the synthetic quest"
    );
    replies.extend(accepted);
    replies
}

fn turn_in_quest(client: &mut Client, server: &mut Server) -> Vec<Vec<u8>> {
    let mut replies = app(client, server, &npc_interact(T335_GIVER));
    let choice = menu_choice(&replies, T335_GIVER, TALK_LABEL);
    replies.extend(app(client, server, &npc_select(T335_GIVER, choice)));
    let end = app(client, server, &dialogue_choice(QUEST_CHOICE));
    assert!(
        end.iter()
            .any(|message| family_body(message, 49, 13).is_some()),
        "the completed Talk holds the 49/13 end screen"
    );
    replies.extend(end);
    let done = app(client, server, &quest_end_reply(T335_QUEST));
    assert!(
        done.iter()
            .any(|message| family_body(message, 49, 4).is_some()),
        "the 49/14 turn-in completes the quest"
    );
    replies.extend(done);
    replies
}

/// As `attack_until_dead`, but keeps every client's drained messages.
fn attack_until_dead_all(
    server: &mut Server,
    clients: &mut [&mut Client],
    attacker: usize,
    target: u64,
) -> Vec<Vec<Vec<u8>>> {
    let mut replies = vec![Vec::new(); clients.len()];
    replies[attacker].extend(field_attack(clients[attacker], server, target));
    let deadline = Instant::now() + Duration::from_secs(15);
    while server.test_field_health(target) != Some(0) {
        assert!(
            Instant::now() < deadline,
            "field target {target} did not die"
        );
        std::thread::sleep(Duration::from_millis(750));
        let batch = pump(server, clients, Duration::from_millis(50));
        for (index, messages) in batch.into_iter().enumerate() {
            replies[index].extend(messages);
        }
        if server.test_field_health(target) == Some(0) {
            break;
        }
        replies[attacker].extend(field_attack(clients[attacker], server, target));
    }
    replies
}

/// Basic-attack `target` until the shared field reports it dead, waiting out
/// the production 600 ms swing gate (`src/encounter.rs:1934-1937`) with margin
/// and servicing every live client.
fn attack_until_dead(
    server: &mut Server,
    clients: &mut [&mut Client],
    attacker: usize,
    target: u64,
) -> Vec<Vec<u8>> {
    attack_until_dead_all(server, clients, attacker, target).swap_remove(attacker)
}

fn assert_no_dev_commands(clients: &[&Client]) {
    for client in clients {
        for message in &client.sent {
            let lower = message.to_ascii_lowercase();
            assert!(
                !lower.windows(6).any(|window| window == b"realm "),
                "the loop sent a `realm` developer command"
            );
            assert!(
                !lower.windows(8).any(|window| window == b"teleport"),
                "the loop sent a teleport developer command"
            );
        }
    }
}

#[test]
fn a_group_forms_and_party_chat_reaches_both() {
    let temp = TempDir::new();
    let (mut server, store_path) = start_server(&temp);
    let store = AccountStore::load(&store_path).unwrap();
    let b_name = {
        let identity = &store.character(3).unwrap().identity;
        (identity.first_name.clone(), identity.last_name.clone())
    };
    let (mut first, _) = log_in(&mut server, 431, 2, &store_path);
    let (mut second, _) = log_in(&mut server, 432, 3, &store_path);

    // The real 40/1 invite creates the leader row and reaches the target.
    let invited = first.send(&mut server, &application(6, &group_invite()));
    assert!(invited.iter().any(|message| is_family(message, 40, 8)));
    assert_eq!(group_result_code(&invited, 2), Some(0));
    let pending = second.drain(&mut server);
    let invite_id = group_invite_wire::invite_generation(
        pending
            .iter()
            .find(|message| is_family(message, 40, 1))
            .expect("the target received the 40/1 invite"),
    );

    // A stale pop-up generation is refused and must not consume the live
    // invitation: the real generation below still forms the group. Removing
    // the generation check accepts this body and mutates the group.
    let stale = second.send(
        &mut server,
        &application(6, &group_accept(invite_id ^ 0x8000_0000)),
    );
    assert_eq!(group_result_code(&stale, 5), Some(3));
    assert!(
        !stale.iter().any(|message| {
            is_family(message, 40, 4) || is_family(message, 40, 8) || is_family(message, 40, 9)
        }),
        "a stale accept changed the group"
    );

    // The real 40/4 accept fans the notice, the member row and the roster to
    // both members.
    let mut b_side = second.send(&mut server, &application(6, &group_accept(invite_id)));
    b_side.extend(second.drain(&mut server));
    let a_side = first.drain(&mut server);
    assert!(
        group_snapshot_guids(&a_side).contains(&vec![2, 3]),
        "A's snapshot names the leader and both members"
    );
    assert!(
        group_snapshot_guids(&b_side).contains(&vec![2, 3]),
        "B's snapshot names the leader and both members"
    );
    let mut both = pending;
    both.extend(a_side);
    both.extend(b_side);
    assert!(both.iter().any(|message| is_family(message, 40, 4)));
    assert!(both.iter().any(|message| is_family(message, 40, 9)));

    // Party chat (channel 4) echoes to the sender and reaches B's inbox. The
    // echo and B's delivery are asserted separately: a union would pass after
    // the non-sender recipients were dropped.
    let party = first.send(
        &mut server,
        &application(6, &chat_request(4, "synthetic party line", None)),
    );
    assert!(
        has_text(&party, "synthetic party line"),
        "the sender does not receive its party echo"
    );
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        "synthetic party line",
    );
    // The server's (R) spam gate allows one line per 330 ms per session.
    std::thread::sleep(Duration::from_millis(400));
    // A local say in range echoes to A and reaches B.
    let say = first.send(
        &mut server,
        &application(6, &chat_request(0, "synthetic local say", None)),
    );
    assert!(has_text(&say, "synthetic local say"));
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        "synthetic local say",
    );
    std::thread::sleep(Duration::from_millis(400));
    // A tell from A reaches B by name. Tell recipients are the target only
    // (`resolve_recipients`), so there is no sender echo to assert.
    let tell = first.send(
        &mut server,
        &application(
            6,
            &chat_request(1, "synthetic tell", Some((&b_name.0, &b_name.1))),
        ),
    );
    assert!(!has_text(&tell, "synthetic tell"));
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        "synthetic tell",
    );

    // A refused duplicate invite leaves the group unchanged: B is already in
    // this group, so the service refuses the repeat as the client's
    // already-grouped result (3) rather than a duplicate invite (6).
    let repeated = first.send(&mut server, &application(6, &group_invite()));
    assert_eq!(group_result_code(&repeated, 2), Some(3));
    assert!(
        !second
            .drain(&mut server)
            .iter()
            .any(|m| is_family(m, 40, 1)),
        "a refused duplicate invite reached the target"
    );

    // The party still works: A's line reaches B and B's own line reaches A.
    std::thread::sleep(Duration::from_millis(400));
    let still = first.send(
        &mut server,
        &application(6, &chat_request(4, "synthetic still grouped", None)),
    );
    assert!(has_text(&still, "synthetic still grouped"));
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        "synthetic still grouped",
    );
    let from_b = second.send(
        &mut server,
        &application(6, &chat_request(4, "synthetic from b", None)),
    );
    assert!(has_text(&from_b, "synthetic from b"));
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        0,
        "synthetic from b",
    );
}

#[test]
fn each_character_keeps_its_own_coins_and_quest_after_relog() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    seed_second_character(&temp, content.path(), T335_MEDIC, true);
    let (mut server, store_path) = start_field_server(&temp, content.path());
    let version = field_version();
    let (mut first, _) = log_in_version_batched(&mut server, 441, 2, &store_path, &version);
    settle(&mut server, &mut [&mut first]);
    let (mut second, _) = log_in_version_batched(&mut server, 442, 3, &store_path, &version);
    settle(&mut server, &mut [&mut first, &mut second]);
    {
        let mut clients = [&mut first, &mut second];
        wait_until(&mut server, &mut clients, T335_PATIENCE, |server| {
            knows(server, 2, T335_WOLF_ONE)
                && knows(server, 3, T335_WOLF_ONE)
                && knows(server, 2, T335_WOLF_TWO)
                && knows(server, 3, T335_WOLF_TWO)
        });
    }
    become_brawler(&mut first, &mut server, &temp, 2);
    become_job(&mut second, &mut server, &temp, 3, T335_MEDIC);
    settle(&mut server, &mut [&mut first, &mut second]);
    accept_quest(&mut first, &mut server);
    accept_quest(&mut second, &mut server);

    // A kills wolf one alone; A wounds wolf two and B lands the lethal hit.
    // A gets two T240 payouts (killer + contributor), B gets one.
    let (a_start, b_start) = (saved_coins(&temp, 2), saved_coins(&temp, 3));
    attack_until_dead(
        &mut server,
        &mut [&mut first, &mut second],
        0,
        T335_WOLF_ONE,
    );
    // A's last swing at wolf one starts its 600 ms production cooldown.
    std::thread::sleep(Duration::from_millis(750));
    let before_b = saved_coins(&temp, 3);
    field_attack(&mut first, &mut server, T335_WOLF_TWO);
    let mut b_claim = attack_until_dead(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        T335_WOLF_TWO,
    );
    let deadline = Instant::now() + T335_PATIENCE;
    while coin_balances(&b_claim).is_empty() {
        assert!(Instant::now() < deadline, "B's T240 payout did not arrive");
        let batch = {
            let mut clients = [&mut first, &mut second];
            pump(&mut server, &mut clients, Duration::from_millis(20))
        };
        b_claim.extend(batch[1].clone());
    }
    let settled = settle(&mut server, &mut [&mut first, &mut second]);
    b_claim.extend(settled[1].clone());
    assert_eq!(
        saved_coins(&temp, 3),
        before_b + T335_FIELD_COINS,
        "B's saved balance changed exactly once for the shared kill"
    );
    assert_eq!(
        coin_balances(&b_claim),
        vec![before_b + T335_FIELD_COINS],
        "B's kill paid exactly once"
    );
    assert_eq!(
        job_updates(&b_claim, T335_MEDIC).len(),
        1,
        "B's 38/14 names B's own active job"
    );
    assert!(
        job_updates(&b_claim, T335_BRAWLER).is_empty(),
        "B's payout must not use the killer's job"
    );
    assert_eq!(server.test_field_health(T335_WOLF_TWO), Some(0));

    // Both turn the synthetic quest in through the ordinary Talk path.
    turn_in_quest(&mut first, &mut server);
    turn_in_quest(&mut second, &mut server);
    let a_quest_state = saved_quest_state(&temp, 2);
    let b_quest_state = saved_quest_state(&temp, 3);
    let (a_coins, b_coins) = (saved_coins(&temp, 2), saved_coins(&temp, 3));
    let (a_xp, b_xp) = (
        saved_job_xp(&temp, 2, T335_BRAWLER),
        saved_job_xp(&temp, 3, T335_MEDIC),
    );
    assert_eq!(
        a_coins,
        a_start + 2 * T335_FIELD_COINS + T335_QUEST_COINS,
        "A's saved balance is its own kill, its shared-kill contribution and the quest"
    );
    assert_eq!(
        b_coins,
        b_start + T335_FIELD_COINS + T335_QUEST_COINS,
        "B's saved balance is the shared kill and the quest"
    );
    assert!(
        b_xp > 0,
        "B's field payout went to its own active job, not the killer's"
    );
    assert_eq!(saved_item_count(&temp, 2, T335_ITEM), 1);
    assert_eq!(saved_item_count(&temp, 3, T335_ITEM), 2);
    assert_eq!(
        saved_item_ids(&temp, 2, T335_ITEM).len(),
        1,
        "A's quest item is one new stack"
    );
    let b_item_ids = saved_item_ids(&temp, 3, T335_ITEM);
    assert_eq!(
        b_item_ids.len(),
        2,
        "B keeps its seeded stack and the reward stack"
    );
    assert_eq!(b_item_ids[0], 900_001, "B keeps its own seeded stack");
    assert!(
        b_item_ids[1] > 900_001,
        "the quest grant allocated a fresh id after the seeded stack"
    );
    assert_ne!(
        saved_item_ids(&temp, 2, T335_ITEM)[0],
        b_item_ids[1],
        "the two characters' reward stacks must not alias"
    );

    // Save both characters through the real session save and reload them
    // through the production gateway login path.
    disconnect(&first, &mut server);
    disconnect(&second, &mut server);
    assert_eq!(server.active_session_count(), 0);
    assert_eq!(saved_coins(&temp, 2), a_coins);
    assert_eq!(saved_coins(&temp, 3), b_coins);
    assert_eq!(saved_quest_state(&temp, 2), a_quest_state);
    assert_eq!(saved_quest_state(&temp, 3), b_quest_state);
    assert_eq!(save_json(&temp, 2)["character"]["guid"], json!(2));
    assert_eq!(save_json(&temp, 3)["character"]["guid"], json!(3));

    let (mut first, _) = log_in_version_batched(&mut server, 443, 2, &store_path, &version);
    settle(&mut server, &mut [&mut first]);
    let (mut second, _) = log_in_version_batched(&mut server, 444, 3, &store_path, &version);
    settle(&mut server, &mut [&mut first, &mut second]);
    {
        let mut clients = [&mut first, &mut second];
        wait_until(&mut server, &mut clients, T335_PATIENCE, |server| {
            knows(server, 2, T335_WOLF_THREE) && knows(server, 3, T335_WOLF_FOUR)
        });
    }

    // A replayed hit on the first kill's corpse pays and advances nothing.
    let replay_a = field_attack(&mut first, &mut server, T335_WOLF_ONE);
    let replay_b = field_attack(&mut second, &mut server, T335_WOLF_TWO);
    assert!(coin_balances(&replay_a).is_empty());
    assert!(coin_balances(&replay_b).is_empty());
    assert!(!objective_update(&replay_a, T335_QUEST, 1, 2));
    assert!(!objective_update(&replay_b, T335_QUEST, 1, 2));

    // Fresh wolves after the relog pay on top of each character's own saved
    // balance, which proves the loader restored each character's wallet and
    // job XP independently.
    let mut a_after = attack_until_dead(
        &mut server,
        &mut [&mut first, &mut second],
        0,
        T335_WOLF_THREE,
    );
    let settled = settle(&mut server, &mut [&mut first, &mut second]);
    a_after.extend(settled[0].clone());
    let a_xp_after = saved_job_xp(&temp, 2, T335_BRAWLER);
    assert_eq!(a_xp_after, a_xp + T335_FIELD_XP);
    assert_eq!(
        saved_coins(&temp, 2),
        a_coins + T335_FIELD_COINS,
        "A's saved balance changed exactly once after the relog"
    );
    assert_eq!(
        coin_balances(&a_after),
        vec![a_coins + T335_FIELD_COINS],
        "A's post-relog kill paid exactly once"
    );
    assert_eq!(
        job_updates(&a_after, T335_BRAWLER),
        vec![expected_job_update(a_xp_after)]
    );
    assert!(
        job_updates(&a_after, T335_MEDIC).is_empty(),
        "A's payout must not use the contributor's job"
    );
    let mut b_after = attack_until_dead(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        T335_WOLF_FOUR,
    );
    let settled = settle(&mut server, &mut [&mut first, &mut second]);
    b_after.extend(settled[1].clone());
    let b_xp_after = saved_job_xp(&temp, 3, T335_MEDIC);
    assert_eq!(b_xp_after, b_xp + T335_FIELD_XP);
    assert_eq!(
        saved_coins(&temp, 3),
        b_coins + T335_FIELD_COINS,
        "B's saved balance changed exactly once after the relog"
    );
    assert_eq!(
        coin_balances(&b_after),
        vec![b_coins + T335_FIELD_COINS],
        "B's post-relog kill paid exactly once"
    );
    assert_eq!(
        job_updates(&b_after, T335_MEDIC),
        vec![expected_job_update(b_xp_after)]
    );
    assert!(
        job_updates(&b_after, T335_BRAWLER).is_empty(),
        "B's post-relog payout must not use A's job"
    );
    // The completed quest counters are the same as before the relog.
    assert_eq!(saved_quest_state(&temp, 2), a_quest_state);
    assert_eq!(saved_quest_state(&temp, 3), b_quest_state);
    assert_eq!(saved_item_count(&temp, 2, T335_ITEM), 1);
    assert_eq!(saved_item_count(&temp, 3, T335_ITEM), 2);
}

#[test]
fn one_players_disconnect_or_bad_packet_does_not_stop_the_other() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store_path) = start_field_server(&temp, content.path());
    let version = field_version();
    let (mut first, _) = log_in_version_batched(&mut server, 451, 2, &store_path, &version);
    settle(&mut server, &mut [&mut first]);
    let (mut second, _) = log_in_version_batched(&mut server, 452, 3, &store_path, &version);
    settle(&mut server, &mut [&mut first, &mut second]);
    {
        let mut clients = [&mut first, &mut second];
        wait_until(&mut server, &mut clients, T335_PATIENCE, |server| {
            knows(server, 2, T335_WOLF_TWO) && knows(server, 3, T335_WOLF_TWO)
        });
    }
    become_brawler(&mut first, &mut server, &temp, 2);
    become_brawler(&mut second, &mut server, &temp, 3);
    settle(&mut server, &mut [&mut first, &mut second]);
    let before = saved_coins(&temp, 3);
    let before_quests = saved_quest_state(&temp, 3);
    // Drain B's inbox now, so the post-refusal check sees only new replies.
    second.drain(&mut server);

    // A hostile 36/10 with an unknown action kind is a malformed field
    // request. The gateway's single-message refusal drops it for A alone
    // (a bundled child would end A's session), and it changes nothing.
    let a_coins = saved_coins(&temp, 2);
    let a_quests = saved_quest_state(&temp, 2);
    let field_before = server.test_field_health(T335_WOLF_TWO);
    assert_eq!(
        field_before,
        Some(combat_scale::enemy_health(200)),
        "the wolf starts at full health"
    );
    let mut bad = vec![10, 0];
    for value in [1i32, 0, 2] {
        bad.extend(value.to_le_bytes());
    }
    bad.extend(T335_WOLF_TWO.to_le_bytes());
    let refused = first.send(&mut server, &packet(36, &bad));
    assert!(
        !refused
            .iter()
            .any(|message| family_body(message, 38, 19).is_some()
                || family_body(message, 49, 9).is_some()),
        "A's malformed field packet paid or advanced something"
    );
    let delivered = second.drain(&mut server);
    assert!(
        !delivered
            .iter()
            .any(|message| family_body(message, 38, 19).is_some()
                || family_body(message, 49, 9).is_some()),
        "B received a payout or objective update from A's malformed field packet"
    );
    assert_eq!(
        server.active_session_count(),
        2,
        "a single malformed application packet must not end a session"
    );
    assert_eq!(saved_coins(&temp, 2), a_coins);
    assert_eq!(saved_quest_state(&temp, 2), a_quests);
    assert_eq!(saved_coins(&temp, 3), before, "the refusal paid B");
    assert_eq!(
        saved_quest_state(&temp, 3),
        before_quests,
        "the refusal advanced B's quest"
    );
    assert_eq!(
        server.test_field_health(T335_WOLF_TWO),
        field_before,
        "the malformed request damaged the wolf"
    );
    // The refusal also consumed no swing/attempt state: A's immediate valid
    // attack still reaches the wolf.
    field_attack(&mut first, &mut server, T335_WOLF_TWO);
    assert_ne!(
        server.test_field_health(T335_WOLF_TWO),
        field_before,
        "the refused request consumed A's swing"
    );

    // A then disconnects mid-action: only A's session ends and its save is
    // written; B keeps the shared field and the tick loop.
    disconnect(&first, &mut server);
    assert_eq!(
        server.active_session_count(),
        1,
        "A's disconnect ended another session"
    );

    // B still moves, speaks and kills in the same tick loop.
    let mut move_fields = 3u64.to_le_bytes().to_vec();
    for value in [
        T335_START[0],
        T335_START[1] + 0.1,
        T335_START[2],
        0.0,
        0.0,
        1.0,
    ] {
        move_fields.extend_from_slice(&value.to_le_bytes());
    }
    move_fields.extend_from_slice(&[0, 0]);
    second.send(&mut server, &packet(125, &move_fields));
    assert_eq!(server.active_session_count(), 1);
    let say = second.send(
        &mut server,
        &application(6, &chat_request(0, "synthetic survivor say", None)),
    );
    assert!(has_text(&say, "synthetic survivor say"));

    let killed = attack_until_dead(&mut server, &mut [&mut second], 0, T335_WOLF_TWO);
    assert_eq!(server.test_field_health(T335_WOLF_TWO), Some(0));
    assert_eq!(
        coin_balances(&killed),
        vec![before + T335_FIELD_COINS],
        "the survivor's kill paid exactly once"
    );
    server.step().unwrap();
    assert_eq!(server.active_session_count(), 1);
}

#[test]
fn the_social_loop_uses_no_dev_commands() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store_path) = start_field_server(&temp, content.path());
    let version = field_version();
    let (mut first, _) = log_in_version_batched(&mut server, 461, 2, &store_path, &version);
    settle(&mut server, &mut [&mut first]);
    let (mut second, _) = log_in_version_batched(&mut server, 462, 3, &store_path, &version);
    settle(&mut server, &mut [&mut first, &mut second]);
    {
        let mut clients = [&mut first, &mut second];
        wait_until(&mut server, &mut clients, T335_PATIENCE, |server| {
            knows(server, 2, T335_WOLF_ONE) && knows(server, 3, T335_WOLF_ONE)
        });
    }
    become_brawler(&mut first, &mut server, &temp, 2);
    settle(&mut server, &mut [&mut first, &mut second]);

    // Group and party chat through the ordinary social requests.
    let invited = first.send(&mut server, &application(6, &group_invite()));
    assert_eq!(group_result_code(&invited, 2), Some(0));
    let pending = second.drain(&mut server);
    let invite_id = group_invite_wire::invite_generation(
        pending
            .iter()
            .find(|message| is_family(message, 40, 1))
            .expect("the target received the 40/1 invite"),
    );
    second.send(&mut server, &application(6, &group_accept(invite_id)));
    let party = first.send(
        &mut server,
        &application(6, &chat_request(4, "synthetic loop line", None)),
    );
    assert!(has_text(&party, "synthetic loop line"));
    wait_for_client_text(
        &mut server,
        &mut [&mut first, &mut second],
        1,
        "synthetic loop line",
    );

    // One ordinary field kill.
    let before = saved_coins(&temp, 2);
    let killed = attack_until_dead(
        &mut server,
        &mut [&mut first, &mut second],
        0,
        T335_WOLF_ONE,
    );
    assert_eq!(
        coin_balances(&killed),
        vec![before + T335_FIELD_COINS],
        "the loop's kill paid exactly once"
    );

    // A relog on the real session save, then the loop has used only ordinary
    // client requests: no `realm` or teleport command was sent.
    disconnect(&first, &mut server);
    assert_eq!(server.active_session_count(), 1);
    let (mut relogged, _) = log_in_version_batched(&mut server, 463, 2, &store_path, &version);
    settle(&mut server, &mut [&mut second, &mut relogged]);
    assert_eq!(server.active_session_count(), 2);
    assert_no_dev_commands(&[&first, &second, &relogged]);
}

// ---------------------------------------------------------------------------
// T405: the AddPc wield type. Spawn hands AddPc +0x1a4 (reader `0x0091aee4`)
// to the actor's wield setter `0x009582c0` (`0x009300f6`), the same setter the
// live 35/6 trailing dword reaches (`0x009315a9`). A late joiner must get the
// active job's equipped weapon value, never the old 0.
// ---------------------------------------------------------------------------

/// The opening fixture's `inventory.json` wield types: the Brawler hammer 75030
/// its saved Brawler weapon slot holds, and the Medic starter weapon 4622.
const BRAWLER_WIELD: i32 = 13;
const MEDIC_WIELD: i32 = 14;

/// Every AddPc about `guid` in `messages`, in arrival order.
fn add_pcs(messages: &[Vec<u8>], guid: u64) -> Vec<AddPc> {
    messages
        .iter()
        .filter_map(|message| parse_add_pc(message))
        .filter(|add| add.character_guid == guid)
        .collect()
}

/// The trailing wield dword of every 35/6 about `guid` (reader `0x0090a1f0`,
/// packet +0x1ec), in arrival order.
fn equip_change_wields(messages: &[Vec<u8>], guid: u64) -> Vec<i32> {
    messages
        .iter()
        .filter_map(|message| family_body(message, 35, 6))
        .filter(|body| body.get(4..12) == Some(guid.to_le_bytes().as_slice()))
        .map(|body| i32::from_le_bytes(body[body.len() - 4..].try_into().unwrap()))
        .collect()
}

/// The item id saved in `guid`'s `profile` equipment `slot`.
fn saved_equipped(temp: &TempDir, guid: u64, profile: i32, slot: i32) -> i32 {
    save_json(temp, guid)["gameplay"]["inventory"]["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["profile_id"] == json!(profile) && entry["slot"] == json!(slot))
        .and_then(|entry| entry["item_id"].as_i64())
        .map(|id| i32::try_from(id).unwrap())
        .expect("the saved equipment row")
}

/// 42/3 equip by item id: item, profile, slot.
fn equip(item: i32, profile: i32, slot: i32) -> Vec<u8> {
    [
        [42, 0, 3, 0],
        item.to_le_bytes(),
        profile.to_le_bytes(),
        slot.to_le_bytes(),
    ]
    .concat()
}

/// A (guid 2) logs in and becomes a Brawler; then B (guid 3) logs in beside A.
/// Returns both clients and B's login messages.
fn brawler_and_viewer(
    server: &mut Server,
    temp: &TempDir,
    store: &Path,
) -> (Client, Client, Vec<Vec<u8>>) {
    let version = field_version();
    let (mut a, _) = log_in_version_batched(server, 4051, 2, store, &version);
    settle(server, &mut [&mut a]);
    become_brawler(&mut a, server, temp, 2);
    settle(server, &mut [&mut a]);
    let (mut b, mut b_login) = log_in_version_batched(server, 4052, 3, store, &version);
    b_login.extend(settle(server, &mut [&mut a, &mut b]).swap_remove(1));
    (a, b, b_login)
}

#[test]
fn a_late_joiner_sees_the_equipped_weapon_wield_type() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store) = start_field_server(&temp, content.path());
    let (_a, _b, b_login) = brawler_and_viewer(&mut server, &temp, &store);
    let seen = add_pcs(&b_login, 2);
    assert_eq!(seen.len(), 1, "one AddPc for the armed player");
    assert_eq!(seen[0].active_profile, T335_BRAWLER);
    assert_eq!(
        seen[0].wield_type, BRAWLER_WIELD,
        "the late joiner saw the Brawler unarmed"
    );
    assert!(
        equip_change_wields(&b_login, 2).is_empty(),
        "the late joiner needed a 35/6 correction"
    );
}

#[test]
fn the_live_equip_path_and_initial_addpc_agree() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store) = start_field_server(&temp, content.path());
    let (mut a, mut b, _) = brawler_and_viewer(&mut server, &temp, &store);
    let hammer = saved_equipped(&temp, 2, T335_BRAWLER, 7);
    let unequip = [
        [42, 0, 2, 0],
        7i32.to_le_bytes(),
        T335_BRAWLER.to_le_bytes(),
    ]
    .concat();
    app(&mut a, &mut server, &unequip);
    app(&mut a, &mut server, &equip(hammer, T335_BRAWLER, 7));
    let viewed = settle(&mut server, &mut [&mut a, &mut b]).swap_remove(1);
    let live = equip_change_wields(&viewed, 2);
    assert_eq!(live, [0, BRAWLER_WIELD], "B's live 35/6 unequip then equip");

    let (mut c, mut c_login) =
        log_in_version_batched(&mut server, 4053, 1, &store, &field_version());
    c_login.extend(settle(&mut server, &mut [&mut a, &mut b, &mut c]).swap_remove(2));
    let seen = add_pcs(&c_login, 2);
    assert_eq!(seen.len(), 1);
    assert_eq!(
        seen[0].wield_type, live[1],
        "the AddPc disagrees with the live 35/6"
    );
}

#[test]
fn an_inactive_jobs_weapon_does_not_change_the_public_wield_type() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store) = start_field_server(&temp, content.path());
    let version = field_version();
    let (mut a, _) = log_in_version_batched(&mut server, 4054, 2, &store, &version);
    settle(&mut server, &mut [&mut a]);
    let active = save_json(&temp, 2)["gameplay"]["profiles"]["active"].clone();
    assert_ne!(active, json!(T335_BRAWLER));
    assert_ne!(
        saved_equipped(&temp, 2, T335_BRAWLER, 7),
        0,
        "the inactive Brawler holds its hammer"
    );
    let (mut b, mut b_login) = log_in_version_batched(&mut server, 4055, 3, &store, &version);
    b_login.extend(settle(&mut server, &mut [&mut a, &mut b]).swap_remove(1));
    let seen = add_pcs(&b_login, 2);
    assert_eq!(seen.len(), 1);
    assert_eq!(json!(seen[0].active_profile), active);
    assert_eq!(seen[0].wield_type, 0, "an inactive job's weapon leaked");
}

#[test]
fn the_wield_type_survives_save_and_relog() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store) = start_field_server(&temp, content.path());
    let (a, mut b, _) = brawler_and_viewer(&mut server, &temp, &store);
    disconnect(&a, &mut server);
    settle(&mut server, &mut [&mut b]);
    assert!(
        !save_json(&temp, 2).to_string().contains("wield"),
        "the wield type is derived, never persisted"
    );
    let (mut a, _) = log_in_version_batched(&mut server, 4056, 2, &store, &field_version());
    let viewed = settle(&mut server, &mut [&mut a, &mut b]).swap_remove(1);
    let seen = add_pcs(&viewed, 2);
    assert_eq!(seen.len(), 1, "B meets the relogged Brawler once");
    assert_eq!(seen[0].wield_type, BRAWLER_WIELD);
}

#[test]
fn a_refused_equip_does_not_change_the_public_wield_type() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    let (mut server, store) = start_field_server(&temp, content.path());
    let (mut a, mut b, b_login) = brawler_and_viewer(&mut server, &temp, &store);
    let raw_add_pc = |messages: &[Vec<u8>]| -> Vec<Vec<u8>> {
        messages
            .iter()
            .filter(|message| parse_add_pc(message).is_some_and(|add| add.character_guid == 2))
            .cloned()
            .collect()
    };
    let before = raw_add_pc(&b_login);
    assert_eq!(before.len(), 1);
    let saved = save_json(&temp, 2);
    let shirt = saved_equipped(&temp, 2, T335_BRAWLER, 3);
    assert_ne!(shirt, 0);
    // An unowned item, then an owned shirt, into the weapon slot.
    app(&mut a, &mut server, &equip(999_999, T335_BRAWLER, 7));
    app(&mut a, &mut server, &equip(shirt, T335_BRAWLER, 7));
    let viewed = settle(&mut server, &mut [&mut a, &mut b]).swap_remove(1);
    assert!(equip_change_wields(&viewed, 2).is_empty());
    assert_eq!(server.active_session_count(), 2);
    assert_eq!(
        save_json(&temp, 2),
        saved,
        "a refused equip changed the save"
    );

    let (mut c, mut c_login) =
        log_in_version_batched(&mut server, 4057, 1, &store, &field_version());
    c_login.extend(settle(&mut server, &mut [&mut a, &mut b, &mut c]).swap_remove(2));
    assert_eq!(raw_add_pc(&c_login), before, "the next AddPc changed");
}

#[test]
fn two_players_keep_their_own_wield_type() {
    let temp = TempDir::new();
    let content = PlaytestContent::new();
    seed_second_character(&temp, content.path(), T335_MEDIC, false);
    let (mut server, store) = start_field_server(&temp, content.path());
    let (mut a, mut b, _) = brawler_and_viewer(&mut server, &temp, &store);
    become_job(&mut b, &mut server, &temp, 3, T335_MEDIC);
    settle(&mut server, &mut [&mut a, &mut b]);
    let (mut c, mut c_login) =
        log_in_version_batched(&mut server, 4058, 1, &store, &field_version());
    c_login.extend(settle(&mut server, &mut [&mut a, &mut b, &mut c]).swap_remove(2));
    let wields = |guid| {
        add_pcs(&c_login, guid)
            .iter()
            .map(|add| add.wield_type)
            .collect::<Vec<_>>()
    };
    assert_eq!(wields(2), [BRAWLER_WIELD]);
    assert_eq!(wields(3), [MEDIC_WIELD]);
}

/// The accessor behind AddPc fails to 0: an empty weapon slot, a default-state
/// weapon whose definition the catalogue lacks (the load keeps that link, R),
/// and a negative content value. Inventory owns the lookup, so its content is
/// what this edits.
#[test]
fn no_weapon_unknown_definition_and_negative_values_are_zero() {
    use osfr_tools::inventory::Inventory;
    let fixture = RewardFixture::minimal(Vec::new(), &[]);
    let path = fixture.dir().join("inventory.json");
    let mut catalogue: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let load = |catalogue: &serde_json::Value| {
        fs::write(&path, serde_json::to_vec(catalogue).unwrap()).unwrap();
        Inventory::load(fixture.dir(), None).unwrap()
    };
    let initial = load(&catalogue);
    assert_eq!(initial.equipped_wield_type(T335_BRAWLER), BRAWLER_WIELD);
    assert_eq!(
        initial.equipped_wield_type(T335_MEDIC),
        0,
        "an empty weapon slot"
    );
    assert_eq!(initial.equipped_wield_type(-1), 0, "an unknown profile");

    for row in catalogue["definitions"].as_array_mut().unwrap() {
        if row["id"] == json!(75_030) {
            row["wield_type"] = json!(-3);
        }
    }
    assert_eq!(load(&catalogue).equipped_wield_type(T335_BRAWLER), 0);

    // The default state's Brawler weapon is item 4235 (definition 13658).
    catalogue["initial_state"] = serde_json::Value::Null;
    catalogue["definitions"]
        .as_array_mut()
        .unwrap()
        .retain(|row| row["id"] != json!(13_658));
    let defaults = load(&catalogue);
    assert_eq!(
        defaults
            .state()
            .equipment
            .iter()
            .find(|entry| entry.profile_id == T335_BRAWLER && entry.slot == 7)
            .map(|entry| entry.item_id),
        Some(4235)
    );
    assert_eq!(defaults.equipped_wield_type(T335_BRAWLER), 0);
    assert_eq!(
        defaults.equipped_wield_type(2),
        9,
        "a known weapon still reads"
    );
}
