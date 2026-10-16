#[path = "../src/bin/osfr-gateway.rs"]
mod gateway_runtime;

use osfr_tools::{
    account_store::AccountStore,
    login,
    soe::{CUMULATIVE_ACK, Codec, RELIABLE_DATA, RELIABLE_FRAGMENT, ReliableReceiver},
    soe_send::{ReliableSender, SenderConfig},
};
use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SAMANTHA_GUID: u64 = 9_000_000_189;
const MAX_MESSAGE: usize = 16 * 1024 * 1024;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("osfr-w67-runtime-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as u32).to_le_bytes());
    bytes.extend(value.as_bytes());
}

fn account_login(guid: u64, ticket: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    put_string(&mut bytes, ticket);
    bytes.extend(guid.to_le_bytes());
    put_string(&mut bytes, "1.910.1.530630");
    put_string(&mut bytes, "");
    bytes
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn chat(text: &str) -> Vec<u8> {
    let mut bytes = vec![15, 0, 1, 0, 0, 0];
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        bytes.extend([0; 12]);
        bytes.extend(0u32.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
    }
    put_string(&mut bytes, text);
    bytes.extend([0; 28]);
    wrap(&bytes)
}

fn npc_interact(guid: u64) -> Vec<u8> {
    let mut bytes = vec![26, 0, 8, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.push(0);
    wrap(&bytes)
}

fn npc_select(guid: u64, option: i32) -> Vec<u8> {
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(option.to_le_bytes());
    wrap(&bytes)
}

fn app_body(message: &[u8]) -> Option<&[u8]> {
    if message.len() < 7 || message[..3] != [5, 0, 1] {
        return None;
    }
    let size = u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize;
    (size == message.len() - 7).then_some(&message[7..])
}

fn family(message: &[u8]) -> Option<(u16, u16)> {
    let body = app_body(message)?;
    Some((
        u16::from_le_bytes(body.get(..2)?.try_into().ok()?),
        u16::from_le_bytes(body.get(2..4)?.try_into().ok()?),
    ))
}

fn send_application(
    server: &mut gateway_runtime::Server,
    socket: &UdpSocket,
    server_addr: SocketAddr,
    codec: &Codec,
    sender: &mut ReliableSender,
    receiver: &mut ReliableReceiver,
    started: Instant,
    message: &[u8],
) -> Vec<Vec<u8>> {
    sender.queue_message(message).unwrap();
    for wire in sender.poll(started.elapsed().as_millis() as u64).unwrap() {
        socket.send_to(&wire, server_addr).unwrap();
    }
    server.step().unwrap();

    let mut messages = Vec::new();
    let mut quiet_reads = 0;
    let mut wire = [0u8; 65_507];
    while quiet_reads < 3 {
        match socket.recv_from(&mut wire) {
            Ok((length, _)) => {
                quiet_reads = 0;
                let packet = codec.decode(&wire[..length]).unwrap();
                if matches!(
                    packet.opcode,
                    CUMULATIVE_ACK | osfr_tools::soe::SELECTIVE_ACK
                ) {
                    let _ = sender.acknowledge(&packet);
                } else if matches!(packet.opcode, RELIABLE_DATA | RELIABLE_FRAGMENT) {
                    let received = receiver.receive(&packet).unwrap();
                    if let Some(ack) = received.acknowledgement {
                        let ack = ack.packet();
                        let ack_wire = codec.encode(ack.opcode, &ack.payload).unwrap();
                        socket.send_to(&ack_wire, server_addr).unwrap();
                        server.step().unwrap();
                    }
                    messages.extend(received.messages);
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                quiet_reads += 1
            }
            Err(error) => panic!("gateway UDP read failed: {error}"),
        }
    }
    messages
}

#[test]
#[ignore = "requires the round6 native save fixture and local gateway reference"]
fn account_store_gateway_talk_to_samantha_sends_quest_offer() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = std::env::var_os("OSFR_W67_FIXTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("../../../lab/reports/round6-native-20261014T072624Z"));
    let reference = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("../../../lab/references/rust-gateway-reference"));
    let temp = TempDir::new();
    for file in ["character.json", "identity.json", "accounts.json"] {
        fs::copy(fixture.join(file), temp.0.join(file)).unwrap();
    }
    copy_tree(&fixture.join("content"), &temp.0.join("content"));
    let handoff_key = b"W67-runtime-regression-key-32b";
    let handoff_key_file = temp.0.join("handoff.key");
    fs::write(&handoff_key_file, handoff_key).unwrap();

    let store = AccountStore::load(&temp.0.join("accounts.json")).unwrap();
    let account = store.account(1).expect("fixture account");
    let guid = *account.character_guids.first().expect("fixture character");
    let ticket = login::gateway_binding_ticket(handoff_key, account, guid);
    let mut server = gateway_runtime::Server::new(gateway_runtime::Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference,
        state: temp.0.join("character.json"),
        content: Some(temp.0.join("content")),
        identity: Some(temp.0.join("identity.json")),
        account_store: Some(temp.0.join("accounts.json")),
        gateway_ticket: "00000000000000000000000000000001".into(),
        handoff_key: fs::read(handoff_key_file).unwrap(),
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
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let server_addr = server.local_addr().unwrap();
    let mut request = vec![0, 1];
    request.extend(3u32.to_be_bytes());
    request.extend(17u32.to_be_bytes());
    request.extend(512u32.to_be_bytes());
    request.extend(b"CGAPI_527\0");
    socket.send_to(&request, server_addr).unwrap();
    server.step().unwrap();
    let mut response = [0u8; 64];
    let (response_length, _) = socket.recv_from(&mut response).expect("session response");
    assert!(response_length >= 10);
    assert_eq!(&response[..2], &[0, 2]);
    let seed = u32::from_be_bytes(response[6..10].try_into().unwrap());
    let codec = Codec::new(seed, 3, true, MAX_MESSAGE).unwrap();
    let mut sender = ReliableSender::new(
        codec.clone(),
        SenderConfig {
            datagram_limit: 512,
            window: 32,
            max_message: MAX_MESSAGE,
            max_queued_bytes: 32 * 1024 * 1024,
            retry_ms: 250,
            max_retries: 8,
        },
    )
    .unwrap();
    let mut receiver = ReliableReceiver::new(0, 1024, MAX_MESSAGE, 2 * 1024 * 1024).unwrap();
    let started = Instant::now();

    send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &account_login(guid, &ticket),
    );
    send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &wrap(&13u16.to_le_bytes()),
    );
    send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &wrap(&10u16.to_le_bytes()),
    );
    send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &chat("realm goto Samantha"),
    );
    let menu_replies = send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &npc_interact(SAMANTHA_GUID),
    );
    let menu = menu_replies
        .iter()
        .filter_map(|message| app_body(message))
        .find(|body| body.starts_with(&[26, 0, 9, 0]))
        .expect("26/8 interaction returns 26/9 radial menu");
    let option_count = i32::from_le_bytes(menu[13..17].try_into().unwrap());
    assert_eq!(
        option_count, 1,
        "Samantha offers through the sole Talk choice"
    );

    let talk_replies = send_application(
        &mut server,
        &socket,
        server_addr,
        &codec,
        &mut sender,
        &mut receiver,
        started,
        &npc_select(SAMANTHA_GUID, 1),
    );
    let families: Vec<_> = talk_replies
        .iter()
        .filter_map(|message| family(message))
        .collect();
    // The offer window stands alone, like the original (native run 143124 with
    // the 114-byte 49/1); a leading 26/3 left a blank bubble behind it.
    assert!(
        families.contains(&(49, 1)) && !families.contains(&(26, 3)),
        "Talk selection must send only the 49/1 offer; menu option count={option_count}, reply families={families:?}"
    );
}
