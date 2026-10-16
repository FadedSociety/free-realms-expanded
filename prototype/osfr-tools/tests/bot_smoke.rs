//! End-to-end check of the headless bot (src/bin/osfr-bot.rs) against an
//! in-process gateway Server stepped on its own thread, using the same
//! synthetic reference and account store as tests/multiplayer.rs.
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

#[test]
fn script_parser_accepts_every_step() {
    let steps = bot::parse_script(
        "# comment\nchat realm goto Troll Summoner\nwait 50\nmove 1 2 3\nattack\nability 2 77\nraw 0f000100\nexpect 15 1 900\nexpect 12\nexpect-text hi there\nquit\n",
    )
    .unwrap();
    assert_eq!(steps.len(), 10);
    assert_eq!(
        steps[0],
        bot::Step::Chat("realm goto Troll Summoner".into())
    );
    assert_eq!(steps[3], bot::Step::Ability { slot: 0, guid: 1 });
    assert_eq!(
        steps[6],
        bot::Step::Expect {
            family: 15,
            sub: Some(1),
            timeout_ms: Some(900)
        }
    );
    assert!(bot::parse_script("fly 1 2").is_err());
    // 36/10 targeted basic ability is the 24-byte layout encounter.rs accepts.
    let attack = bot::ability_request(0, 1);
    assert_eq!(attack.len(), 24);
    assert_eq!(&attack[..4], &[36, 0, 10, 0]);
    assert_eq!(i32::from_le_bytes(attack[4..8].try_into().unwrap()), 1);
}

#[test]
fn bot_logs_in_to_single_save_gateway_and_chats() {
    let temp = TempDir::new();
    let reference = create_reference(&temp.0);
    let gateway = RunningGateway::start(options(&temp.0, reference, None));
    // Without --content there is no gameplay, so `realm` commands get no reply
    // (the gateway keeps them out of social chat); plain say chat echoes as 15/1.
    // The chat spam gate drops similar lines sent within 1 s, hence `wait 1100`.
    let steps = bot::parse_script(
        "chat hello bot\nexpect 15 1\nchat realm help\nmove 1.5 2 3\nwait 100\nattack\nwait 1100\nchat still here\nexpect-text still here\n",
    )
    .unwrap();
    let bot = bot::run(
        config(gateway.addr, "solo", 61, 1, "synthetic-ticket".into()),
        &steps,
    )
    .expect("bot script passes");
    assert!(
        bot.received.iter().any(|&(family, _, _)| family == 12),
        "login stream arrived"
    );
}

#[test]
fn bot_fails_cleanly_on_unmet_expectation() {
    let temp = TempDir::new();
    let reference = create_reference(&temp.0);
    let gateway = RunningGateway::start(options(&temp.0, reference, None));
    let steps = bot::parse_script("expect 999 1 300").unwrap();
    let error = bot::run(
        config(gateway.addr, "fail", 62, 1, "synthetic-ticket".into()),
        &steps,
    )
    .err()
    .expect("unmet expectation fails");
    assert!(error.contains("expect 999/1"), "{error}");
}

#[test]
fn two_bots_share_an_account_store_gateway() {
    let temp = TempDir::new();
    let reference = create_reference(&temp.0);
    let store = create_account_store(&temp.0);
    let gateway = RunningGateway::start(options(&temp.0, reference, Some(store.clone())));
    let listener = config(gateway.addr, "beryl", 72, 3, binding_ticket(&store, 3));
    let speaker = config(gateway.addr, "aster", 71, 2, binding_ticket(&store, 2));
    thread::scope(|scope| {
        let listening = scope.spawn(move || {
            let steps = bot::parse_script("expect-text hello from aster\nwait 200").unwrap();
            bot::run(listener, &steps)
        });
        let speaking = scope.spawn(move || {
            let steps =
                bot::parse_script("wait 1500\nchat hello from aster\nexpect 15 1\nwait 300")
                    .unwrap();
            bot::run(speaker, &steps)
        });
        speaking.join().unwrap().expect("speaker passes");
        listening
            .join()
            .unwrap()
            .expect("listener hears the speaker");
    });
}

/// Real dev-tools content: `realm here` answers with a 15/3 system message.
#[test]
#[ignore = "requires lab/references rust-gateway-reference and opening-anvil-placement-v1 content"]
fn bot_runs_dev_command_against_real_content() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    let temp = TempDir::new();
    let mut opts = options(&temp.0, base.join("rust-gateway-reference"), None);
    opts.content = Some(base.join("opening-anvil-placement-v1/content"));
    let gateway = RunningGateway::start(opts);
    let mut cfg = config(
        gateway.addr,
        "dev",
        81,
        1,
        login::DEFAULT_GATEWAY_TICKET.into(),
    );
    cfg.version = "1.910.1.530630".into();
    cfg.expect_timeout_ms = 30_000;
    let steps = bot::parse_script("chat realm here\nexpect 15 3\nattack\nwait 300").unwrap();
    bot::run(cfg, &steps).expect("dev command answered");
}

/// Battle cheats move the player into the arena (31/28 ClientBeginZoning);
/// the bot must re-send ready/loaded to receive the encounter details.
#[test]
#[ignore = "requires lab/references rust-gateway-reference and opening-anvil-placement-v1 content"]
fn bot_zones_into_battle_arena_and_wins() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references");
    let temp = TempDir::new();
    let mut opts = options(&temp.0, base.join("rust-gateway-reference"), None);
    opts.content = Some(base.join("opening-anvil-placement-v1/content"));
    let gateway = RunningGateway::start(opts);
    let mut cfg = config(
        gateway.addr,
        "arena",
        82,
        1,
        login::DEFAULT_GATEWAY_TICKET.into(),
    );
    cfg.version = "1.910.1.530630".into();
    cfg.expect_timeout_ms = 30_000;
    let steps = bot::parse_script(
        "chat realm level 10\nexpect 15 3\n\
         chat realm battle 96\nexpect 41 114\n\
         chat realm win\nexpect 39 19",
    )
    .unwrap();
    bot::run(cfg, &steps).expect("battle started, zoned and won");
}
