//! W156 shared-file patch gate: a bot sends every trade request through the real gateway
//! binary and receives the native close lifecycle (same harness as tests/bot_smoke.rs).
#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;

#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;

#[path = "common/startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;

use gateway_runner::{Options, Server};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    identity::{NativeCreation, Resources},
    login,
    startup_snapshot::StartupSnapshot,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
const VERSION: &str = "synthetic.1";
const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "osfr-bot-smoke-{}-{}",
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

/// A gateway Server built and stepped on its own thread until dropped.
struct RunningGateway {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl RunningGateway {
    fn start(options: Options) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let flag = stop.clone();
        let handle = thread::spawn(move || {
            let mut server = Server::new(options).unwrap();
            tx.send(server.local_addr().unwrap()).unwrap();
            while !flag.load(Ordering::Relaxed) {
                server.step().unwrap();
            }
        });
        Self {
            addr: rx.recv().expect("gateway thread failed to start"),
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for RunningGateway {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let joined = handle.join();
            if !thread::panicking() {
                joined.expect("gateway thread panicked");
            }
        }
    }
}

fn options(root: &Path, reference: PathBuf, account_store: Option<PathBuf>) -> Options {
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference,
        state: root.join("gateway-state.json"),
        content: None,
        identity: None,
        account_store,
        gateway_ticket: "synthetic-ticket".into(),
        handoff_key: HANDOFF_KEY.to_vec(),
        seconds: 0,
        dev_tools: true,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
    }
}

fn config(
    gateway: SocketAddr,
    name: &str,
    connection_id: u32,
    guid: u64,
    ticket: String,
) -> bot::BotConfig {
    bot::BotConfig {
        gateway,
        name: name.into(),
        connection_id,
        guid,
        ticket,
        version: VERSION.into(),
        expect_timeout_ms: 5000,
        quiet: false,
    }
}

fn binding_ticket(store: &Path, guid: u64) -> String {
    let store = AccountStore::load(store).unwrap();
    let owner = store
        .account(store.character(guid).unwrap().owner_account_id)
        .unwrap();
    login::gateway_binding_ticket(HANDOFF_KEY, owner, guid)
}

const CANCELLED: u16 = 271; // 59/15 status 1: bytes `0f 01` read as the u16 sub.
const END: u16 = 5; // 59/5 with session id 0.

#[test]
fn bot_trade_requests_get_the_close_lifecycle_and_nothing_else() {
    let temp = TempDir::new();
    let reference = create_reference(&temp.0);
    let gateway = RunningGateway::start(options(&temp.0, reference, None));
    // Session id 0 keeps the sub-5 reply readable as u16 sub 5. Order matters: offer item,
    // offer coins, accepted reply, decline (silent), cancel, then hostile shapes (silent).
    let steps = bot::parse_script(
        "raw 3b0009 00000000 01000000 02000000
wait 200
         raw 3b000a 00000000 09000000
wait 200
         raw 3b0003 00000000 01
wait 200
         raw 3b0003 00000000 00
wait 200
         raw 3b0007 00000000
wait 200
         raw 3b0007 ffffffff
wait 200
         raw 3b0009 00000000 01
wait 200
         raw 3b0005 00000000
wait 200
         chat hello bot
expect 15 1
",
    )
    .unwrap();
    let bot = bot::run(
        config(gateway.addr, "trade", 62, 1, "synthetic-ticket".into()),
        &steps,
    )
    .expect("bot script passes");
    let trade: Vec<_> = bot
        .received
        .iter()
        .filter(|&&(family, _, _)| family == 59)
        .map(|&(_, sub, len)| (sub, len))
        .collect();
    assert_eq!(
        trade,
        vec![
            (Some(CANCELLED), 4),
            (Some(END), 7),
            (Some(CANCELLED), 4),
            (Some(END), 7),
            (Some(END), 7),
            (Some(END), 7),
        ]
    );
}
