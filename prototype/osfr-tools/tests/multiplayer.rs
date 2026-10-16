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
    put_string(&mut out, VERSION);
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
        request.extend_from_slice(&512u32.to_be_bytes());
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
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&2u64.to_le_bytes());
    group_name(&mut out, "Aster", "One");
    out.extend_from_slice(&3u64.to_le_bytes());
    group_name(&mut out, "Beryl", "Two");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&[0; 20]);
    out
}

fn group_accept() -> Vec<u8> {
    let mut out = vec![40, 0, 4, 0];
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&2u64.to_le_bytes());
    group_name(&mut out, "Aster", "One");
    out.extend_from_slice(&3u64.to_le_bytes());
    group_name(&mut out, "Beryl", "Two");
    out.extend_from_slice(&1u32.to_le_bytes());
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
    let reference = create_reference(&temp.0);
    let account_store = create_account_store(&temp.0);
    let server = Server::new(Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference,
        state: temp.0.join("gateway-state.json"),
        content: None,
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
    })
    .unwrap();
    (server, account_store)
}

fn log_in(server: &mut Server, connection_id: u32, guid: u64, store: &Path) -> (Client, Vec<Vec<u8>>) {
    let mut client = Client::connect(server, connection_id);
    let mut messages = client.send(server, &login(guid, store));
    messages.extend(client.send(server, &packet(13, &[])));
    messages.extend(client.send(server, &packet(10, &[])));
    (client, messages)
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
    first.send(&mut server, &application(6, &chat_request(0, "still here", None)));
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
    let accepted = second.send(&mut server, &application(6, &group_accept()));
    let mut group_updates = accepted;
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
