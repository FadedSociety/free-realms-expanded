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
    time::Duration,
};

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
    let ticket = if account.id == 1 && guid == 1 {
        "synthetic-ticket".to_owned()
    } else {
        login::gateway_binding_ticket(b"synthetic-handoff-key", account, guid)
    };
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

struct Client {
    socket: UdpSocket,
    server: SocketAddr,
    connection_id: u32,
    codec: Codec,
    sender: ReliableSender,
    receiver: ReliableReceiver,
    clock_ms: u64,
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
        }
    }

    fn send(&mut self, server: &mut Server, message: &[u8]) -> Vec<Vec<u8>> {
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
                    server.step().unwrap();
                }
            }
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
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: Some(temp.0.join("security.jsonl")),
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
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
