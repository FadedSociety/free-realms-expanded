//! W177 end-to-end proof, over real sockets: a saved owned pet is listed after relog, summons one
//! actor that a neighbour and a late arrival see, dismisses and removes it for both, and an
//! unowned summon is refused without creating an actor. This copies the W162 bot rig.
#![allow(dead_code)]
#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;

use ed25519_dalek::SigningKey;
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::{AccountRole, AccountStore},
    gameplay::Gameplay,
    gateway::Gateway,
    identity::{NativeCreation, Resources},
    login,
    login_auth::login_handle_hash,
    pets::PetService,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicU32, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const VERSION: &str = "1.910.1.530630";
/// Generous: under a loaded full suite a gateway reply can take seconds.
const T: Duration = Duration::from_secs(20);
static NEXT: AtomicU32 = AtomicU32::new(0);
/// Each scenario starts its own gateway and login server, each loading the
/// 58k-row catalogue; running them one at a time keeps their timing honest.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

// ---------------------------------------------------------------- processes

struct Service(Child);
impl Drop for Service {
    fn drop(&mut self) {
        // Our own child handle only; never a kill by name.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Spawn `command`, tee its stdout/stderr into `log`, and return once it prints
/// the JSON event `event` (carrying `address`) together with that event line.
fn spawn(mut command: Command, event: &str, log: &Path) -> (Service, Value) {
    // The bots are strict about timing (a 5 s handshake window) and the server under test
    // builds a whole session per handshake, so on a machine busy with builds and other test
    // binaries the services must not queue behind them (T628; as `bot_acceptance_1016`).
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0000_8000); // ABOVE_NORMAL_PRIORITY_CLASS
    let mut child = Service(
        command
            .stdout(Stdio::piped())
            .stderr(fs::File::create(log.with_extension("err.log")).unwrap())
            .spawn()
            .unwrap(),
    );
    let stdout = child.0.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let wanted = event.to_owned();
    let mut out = fs::File::create(log.with_extension("out.log")).unwrap();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = writeln!(out, "{line}");
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                if value["event"] == wanted {
                    let _ = tx.send(value);
                }
            }
        }
    });
    let value = rx
        .recv_timeout(Duration::from_secs(120))
        .unwrap_or_else(|_| panic!("service did not print `{event}`; see {}", log.display()));
    (child, value)
}

// --------------------------------------------------------------------- rig

struct Acct {
    id: u64,
    guid: u64,
    handle: String,
}

/// A scratch gateway (+ optional login broker) with one character per account.
struct Rig {
    root: PathBuf,
    accounts: Vec<Acct>,
    handoff: Vec<u8>,
    gateway_event: Value,
    gateway: SocketAddr,
    /// (login UDP address, broker TCP address) when started with a broker.
    broker: Option<(SocketAddr, SocketAddr)>,
    start_position: [f32; 3],
    content: PathBuf,
    _services: Vec<Service>,
}

impl Drop for Rig {
    fn drop(&mut self) {
        self._services.clear(); // stop children before deleting their files
        for a in &self.accounts {
            let _ = fs::remove_file(self.root.join(format!("{}.ed25519", a.handle)));
        }
        let _ = fs::remove_file(self.root.join("handoff.key"));
        let _ = fs::remove_file(self.root.join("login.key"));
        if !std::thread::panicking() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend((value.len() as u32).to_le_bytes());
    out.extend(value.as_bytes());
}

/// Native character-creation resources (synthetic rows, as in `bot_broker_login`).
fn write_creator_resources(dir: &Path) {
    let creator = dir.join("CharacterCreate");
    fs::create_dir_all(&creator).unwrap();
    let mut model = vec!["0"; 17];
    model[0] = "1";
    model[5] = "1"; // gender 1: a boy
    model[13] = "1";
    fs::write(dir.join("Models.txt"), model.join("^")).unwrap();
    for (file, row) in [
        ("HeadMappings.txt", "1^head.adr^"),
        ("HairMappings.txt", "2^hair.adr^"),
        ("ModelCustomizationMappings.txt", "1^beard.adr^"),
        ("FacePaintMappings.txt", "0^none^"),
        ("SkinToneMappings.txt", "1^skin.adr^"),
    ] {
        fs::write(creator.join(file), row).unwrap();
    }
    let mut rows = String::new();
    for (id, slot) in [(201, 5), (189, 4), (199, 3)] {
        let mut row = vec!["0".to_owned(); 42];
        row[0] = id.to_string();
        row[12] = slot.to_string();
        row[16] = "item.adr".into();
        row[17] = "1".into();
        row[18] = "cloth".into();
        row[19] = "0".into();
        row[41] = "tint".into();
        rows.push_str(&row.join("^"));
        rows.push('\n');
    }
    fs::write(creator.join("ItemDefinitions.txt"), rows).unwrap();
}

fn free_udp() -> SocketAddr {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

impl Rig {
    /// One account per role, in order. `broker` also starts `osfr-login`.
    fn start(roles: &[AccountRole], broker: bool) -> Self {
        let lab = lab();
        let reference = lab.join("references/rust-gateway-reference");
        let bundle = lab.join("reports/round70-native-20261016T061000Z/content");
        for needed in [&reference, &bundle] {
            assert!(
                needed.is_dir(),
                "private input missing: {}",
                needed.display()
            );
        }
        let root = lab.join("reports/bot-acceptance-1016").join(format!(
            "run-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        // Content: bundled data/content with the round70 opening bundle on top.
        let content = root.join("content");
        copy_dir(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content"),
            &content,
        );
        copy_dir(&bundle, &content);

        // Accounts, keys and characters.
        let creator = root.join("creator");
        write_creator_resources(&creator);
        let resources = Resources::load(&creator).unwrap();
        let mut store = AccountStore::new();
        let mut accounts = Vec::new();
        for (index, role) in roles.iter().enumerate() {
            let handle = format!("acct_{:03}", index + 1);
            let id = store
                .create_account(
                    login_handle_hash(&handle).unwrap(),
                    AccountFlags::default(),
                    SlotPolicy { max_characters: 1 },
                )
                .unwrap();
            let mut seed = [0u8; 32];
            getrandom::getrandom(&mut seed).unwrap();
            store
                .add_public_key(
                    id,
                    &SigningKey::from_bytes(&seed).verifying_key().to_bytes(),
                )
                .unwrap();
            store.set_account_role_offline(id, *role).unwrap();
            let mut choices = [0; 21];
            for (slot, value) in [
                (0, 1),
                (1, 1),
                (2, 2),
                (5, 1),
                (7, 201),
                (8, 189),
                (9, 199),
                (20, 1),
            ] {
                choices[slot] = value;
            }
            let letter = (b'A' + index as u8) as char;
            let reply = create_character(
                &mut store,
                id,
                NativeCreation {
                    first_name: format!("Fixture{letter}"),
                    last_name: "Record".into(),
                    temporary_first_name: format!("Tmp{letter}"),
                    temporary_last_name: "TempName".into(),
                    choices,
                },
                &resources,
            );
            assert_eq!(
                reply.status,
                osfr_tools::account::CharacterCreateStatus::Created
            );
            let guid = store.account(id).unwrap().character_guids[0];
            fs::write(root.join(format!("{handle}.ed25519")), seed).unwrap();
            accounts.push(Acct { id, guid, handle });
        }
        let store_path = root.join("accounts.json");
        store.save(&store_path).unwrap();
        let mut handoff = vec![0u8; 32];
        getrandom::getrandom(&mut handoff).unwrap();
        fs::write(root.join("handoff.key"), &handoff).unwrap();

        // Where a new character appears (the opening world's start).
        let start =
            Gameplay::initial_character(&content, Gateway::reference_state(&reference).unwrap())
                .unwrap();

        // The gateway: no --dev-tools, so only account roles can authorise `realm`.
        let mut gateway = Command::new(env!("CARGO_BIN_EXE_osfr-gateway"));
        gateway
            .arg("127.0.0.1:0")
            .arg("--reference")
            .arg(&reference)
            .arg("--content")
            .arg(&content)
            .arg("--state")
            .arg(root.join("state/gateway.json"))
            .arg("--world")
            .arg(root.join("world"))
            .arg("--account-store")
            .arg(&store_path)
            .arg("--handoff-key-file")
            .arg(root.join("handoff.key"))
            .arg("--security-log")
            .arg(root.join("security.jsonl"))
            .args(["--studio-port", "0"]);
        let (gateway_service, gateway_event) = spawn(gateway, "listening", &root.join("gateway"));
        let gateway_addr: SocketAddr = gateway_event["address"].as_str().unwrap().parse().unwrap();
        let mut services = vec![gateway_service];

        let mut broker_addrs = None;
        if broker {
            let mut key = [0u8; 32];
            for name in ["login.key", "server.id"] {
                getrandom::getrandom(&mut key).unwrap();
                fs::write(root.join(name), key).unwrap();
            }
            let udp = free_udp();
            let mut login = Command::new(env!("CARGO_BIN_EXE_osfr-login"));
            login
                .args(["--bind", &udp.to_string(), "--auth-bind", "127.0.0.1:0"])
                .args(["--gateway", &gateway_addr.to_string()])
                .arg("--resources")
                .arg(&creator)
                .arg("--identity")
                .arg(root.join("identity.json"))
                .arg("--account-store")
                .arg(&store_path)
                .arg("--server-id-file")
                .arg(root.join("server.id"))
                .arg("--key-file")
                .arg(root.join("login.key"))
                .arg("--handoff-key-file")
                .arg(root.join("handoff.key"));
            let (login_service, event) = spawn(login, "auth_listening", &root.join("login"));
            services.push(login_service);
            broker_addrs = Some((udp, event["address"].as_str().unwrap().parse().unwrap()));
        }
        Rig {
            root,
            accounts,
            handoff,
            gateway_event,
            gateway: gateway_addr,
            broker: broker_addrs,
            start_position: start.position,
            content,
            _services: services,
        }
    }

    /// The character's save file, as the gateway wrote it.
    fn save(&self, account: usize) -> Value {
        let path = self.root.join(format!(
            "state/characters/{}.json",
            self.accounts[account].guid
        ));
        serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        )
        .unwrap()
    }

    fn save_path(&self, account: usize) -> PathBuf {
        self.root.join(format!(
            "state/characters/{}.json",
            self.accounts[account].guid
        ))
    }

    /// Poll the save until `ready` accepts it (the gateway coalesces pose saves).
    fn save_when(&self, account: usize, what: &str, ready: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Ok(bytes) = fs::read(self.root.join(format!(
                "state/characters/{}.json",
                self.accounts[account].guid
            ))) {
                if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                    if ready(&value) {
                        return value;
                    }
                    if Instant::now() > deadline {
                        panic!("save never showed {what}: {value:#}");
                    }
                }
            }
            if Instant::now() > deadline {
                panic!("no readable save for {what}");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Client-initiated session ends the gateway has logged. `end_session` writes the
    /// character save before printing its event, so a seed written after this count
    /// increases is the logged-out state, not the login's.
    fn client_disconnects(&self) -> usize {
        fs::read_to_string(self.root.join("gateway.out.log"))
            .map(|text| {
                text.lines()
                    .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                    .filter(|value| {
                        value["event"] == "session_ended" && value["reason"] == "client_disconnect"
                    })
                    .count()
            })
            .unwrap_or(0)
    }

    fn ticket(&self, account: usize) -> String {
        let store = AccountStore::load(&self.root.join("accounts.json")).unwrap();
        let a = &self.accounts[account];
        login::gateway_binding_ticket(&self.handoff, store.account(a.id).unwrap(), a.guid)
    }

    /// Log a bot in directly through the gateway's character binding ticket.
    /// A reconnect right after a disconnect can race the gateway noticing it,
    /// so a refused login is retried a few times.
    fn connect(&self, account: usize, name: &str) -> bot::Bot {
        let mut last = String::new();
        for _ in 0..8 {
            let config = bot::BotConfig {
                gateway: self.gateway,
                name: name.into(),
                connection_id: std::process::id()
                    .wrapping_mul(31)
                    .wrapping_add(NEXT.fetch_add(1, Ordering::Relaxed) + 1000),
                guid: self.accounts[account].guid,
                ticket: self.ticket(account),
                version: VERSION.into(),
                expect_timeout_ms: 20000,
                quiet: true,
            };
            let attempt = bot::Bot::connect(config).and_then(|mut bot| bot.login().map(|_| bot));
            match attempt {
                Ok(bot) => return bot,
                Err(error) => last = error,
            }
            std::thread::sleep(Duration::from_millis(700));
        }
        panic!("{name} could not log in: {last}");
    }

    /// Drive the `osfr-bot` binary through AUTH2, native login and the handoff.
    fn cli(&self, account: usize, cmds: &[&str]) -> Output {
        let (login, auth) = self.broker.expect("rig started without a broker");
        let a = &self.accounts[account];
        let profile = self.root.join(format!("{}.profile.json", a.handle));
        fs::write(
            &profile,
            serde_json::to_vec(&json!({
                "handle": a.handle,
                "key_file": format!("{}.ed25519", a.handle),
                "server_id_file": "server.id",
                "broker": auth.to_string(),
                "login": login.to_string(),
                "login_key_file": "login.key",
            }))
            .unwrap(),
        )
        .unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_osfr-bot"));
        command
            .arg("--broker-profile")
            .arg(&profile)
            .args(["--guid", &a.guid.to_string(), "--version", VERSION])
            .args(["--expect-timeout", "15000"]);
        for cmd in cmds {
            command.args(["--cmd", cmd]);
        }
        command.output().unwrap()
    }
}

/// Text of a 15/3 system chat message.
fn system_text(message: &[u8]) -> Option<String> {
    if message.get(..4)? != [15, 0, 3, 0] {
        return None;
    }
    let len = u32::from_le_bytes(message.get(4..8)?.try_into().ok()?) as usize;
    Some(String::from_utf8_lossy(message.get(8..8 + len)?).into_owned())
}

fn raw(bot: &mut bot::Bot, inner: &[u8]) {
    bot.send_message(&bot::wrap(inner), "raw").unwrap();
}

fn tail(bot: &bot::Bot) -> String {
    let from = bot.received.len().saturating_sub(8);
    format!("{:?}", &bot.received[from..])
}

fn disconnect(mut bot: bot::Bot) {
    bot.disconnect();
}

/// Disconnect `bot` and wait until the gateway has written its logout save (T628: a fixed
/// sleep let a loaded gateway overwrite a seed with the late logout save). The SOE disconnect
/// is an unreliable datagram, so it is resent while waiting; a repeated disconnect for an
/// absent session is ignored.
fn logout(rig: &Rig, mut bot: bot::Bot, what: &str) {
    let before = rig.client_disconnects();
    bot.disconnect();
    let deadline = Instant::now() + T;
    let mut resent = Instant::now();
    while rig.client_disconnects() <= before {
        if Instant::now() >= deadline {
            panic!("{what}: the gateway never logged the logout");
        }
        if resent.elapsed() >= Duration::from_secs(1) {
            bot.disconnect();
            resent = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Add owned items to a character's save while it is logged out, the way a
/// purchase or a drop would have (the starter inventory is ten items).
fn seed_items(rig: &Rig, account: usize, items: &[(i64, i64)]) {
    let path = rig.save_path(account);
    let mut save: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let owned = save["gameplay"]["inventory"]["items"]
        .as_array_mut()
        .unwrap();
    for &(id, definition_id) in items {
        owned.push(json!({
            "definition_id": definition_id, "tint": 0, "id": id, "count": 1,
            "consumed_count": 0, "last_cast_time": i32::MAX, "rental": null,
            "ability_count": 0, "activate_enabled": false
        }));
    }
    let highest = items.iter().map(|i| i.0).max().unwrap_or(0);
    save["gameplay"]["inventory"]["next_instance_id"] = json!(highest + 1);
    fs::write(&path, serde_json::to_vec(&save).unwrap()).unwrap();
}

/// Add one saved pet and the active item that grants it to a logged-out character.
fn seed_pet(rig: &Rig, account: usize) -> u32 {
    const CHAR_SPECIES: u32 = 1_000_015;
    const CHAR_ITEM: i64 = 70352;
    seed_items(rig, account, &[(900_001, CHAR_ITEM)]);
    let content = PetService::bundled_content().unwrap();
    let definition = content.definition(CHAR_SPECIES).unwrap();
    let mut pets = PetService::new(content).unwrap();
    let pet_id = pets.adopt_vendor_grant(1, &definition, 0).unwrap();
    let path = rig.save_path(account);
    let mut save: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    save["pets"] = pets.document_value().unwrap();
    fs::write(&path, serde_json::to_vec(&save).unwrap()).unwrap();
    pet_id
}

fn quick_chat(phrase: i32, first: &str) -> Vec<u8> {
    let mut out = vec![67, 0, 3, 0];
    out.extend(phrase.to_le_bytes());
    out.extend(0u64.to_le_bytes()); // sender guid: ignored, the server uses its own
    out.extend([0u8; 12]); // name data fields
    put_string(&mut out, first);
    put_string(&mut out, "Record");
    out.extend(0u16.to_le_bytes()); // channel: say
    out.extend(0u32.to_le_bytes());
    out.extend(0u64.to_le_bytes()); // guild
    out
}

fn is_quick_chat(message: &[u8], phrase: i32) -> bool {
    message.starts_with(&[67, 0, 3, 0]) && message.get(4..8) == Some(&phrase.to_le_bytes()[..])
}

// ------------------------------------------------------------------ W177 scenario

fn summon_request(pet_id: u32) -> Vec<u8> {
    [vec![53, 0, 4], pet_id.to_le_bytes().to_vec(), vec![1]].concat()
}
fn is_pet_actor(m: &[u8], actor: u64) -> bool {
    m.starts_with(&[35, 0, 2, 0]) && m.get(4..12) == Some(&actor.to_le_bytes()[..])
}
fn is_pet_removal(m: &[u8], actor: u64) -> bool {
    m.starts_with(&[35, 0, 3, 0, 1, 0]) && m.get(6..14) == Some(&actor.to_le_bytes()[..])
}
fn is_pet_active(m: &[u8], pet_id: u32) -> bool {
    m.starts_with(&[53, 0, 9]) && m.get(3..7) == Some(&pet_id.to_le_bytes()[..])
}
fn is_pet_list(m: &[u8]) -> bool {
    m.starts_with(&[53, 0, 5])
}

#[test]
fn a_saved_pet_is_summoned_seen_by_two_players_and_dismissed() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Player; 3], false);

    // The first login writes the save; seed a pet and its item while A is logged out.
    logout(&rig, rig.connect(0, "petA0"), "the first login's logout");
    let pet_id = seed_pet(&rig, 0);

    let ga = rig.accounts[0].guid;
    let gb = rig.accounts[1].guid;
    let actor = PetService::pet_guid(ga, pet_id);
    let mut a = rig.connect(0, "petA");
    let mut b = rig.connect(1, "petB");
    a.pump(Duration::from_millis(500)).unwrap();
    b.pump(Duration::from_millis(500)).unwrap();

    // B owns no pet: a visible refusal, and neither A nor B sees a B-owned actor.
    raw(&mut b, &summon_request(pet_id));
    b.wait_for(T, |m| {
        system_text(m).is_some_and(|t| t.contains("do not own"))
    })
    .unwrap_or_else(|e| panic!("B: no refusal ({e}); got {}", tail(&b)));
    assert!(
        a.wait_for(Duration::from_secs(1), |m| is_pet_actor(
            m,
            PetService::pet_guid(gb, pet_id)
        ))
        .is_err(),
        "an unowned pet appeared"
    );

    // A summons exactly one saved pet. A receives its own actor and active pair; B sees the actor.
    raw(&mut a, &summon_request(pet_id));
    // The actor must precede activation (the client resolves it on PetActive).
    a.wait_for(T, |m| is_pet_actor(m, actor))
        .expect("A: pet actor");
    a.wait_for(T, |m| is_pet_active(m, pet_id))
        .unwrap_or_else(|e| panic!("A: no active pair ({e}); got {}", tail(&a)));
    b.wait_for(T, |m| is_pet_actor(m, actor))
        .expect("B: neighbour sees pet actor");

    // A late arrival receives the saved roster and the existing pet actor.
    let mut c = rig.connect(2, "petC");
    c.wait_for(T, |m| is_pet_actor(m, actor))
        .unwrap_or_else(|e| panic!("C: late pet actor missing ({e}); got {}", tail(&c)));

    // The client's serializer-shaped dismiss (u32 id word, action 0) removes the
    // actor for everyone.
    raw(&mut a, &[53, 0, 4, 0, 0, 0, 0, 0]);
    for (name, bot) in [("A", &mut a), ("B", &mut b), ("C", &mut c)] {
        bot.wait_for(T, |m| is_pet_removal(m, actor))
            .unwrap_or_else(|e| panic!("{name}: pet not removed ({e}); got {}", tail(bot)));
    }

    // Leaving with a fresh summon also removes it from neighbours.
    raw(&mut a, &summon_request(pet_id));
    b.wait_for(T, |m| is_pet_actor(m, actor))
        .expect("B: second pet actor");
    disconnect(a);
    b.wait_for(T, |m| is_pet_removal(m, actor))
        .unwrap_or_else(|e| panic!("B: pet stayed after A left ({e}); got {}", tail(&b)));
    disconnect(b);
    disconnect(c);
}
