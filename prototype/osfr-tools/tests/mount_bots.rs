//! W162 end-to-end proof, over real sockets: a player summons and boards an owned ride, a second
//! player in range sees the ride actor and the boarding, a third who arrives later sees them too,
//! an unowned ride is refused, a mounted emote is dropped, and putting the ride away (or leaving)
//! removes it for everybody. Needs `lab/references/rust-gateway-reference` and the round70
//! content bundle, like `tests/bot_acceptance_1016.rs`, whose rig this file copies. Drop it in as
//! `tests/mount_bots.rs` AFTER `apply_shared_patch.py` is applied.
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

// ------------------------------------------------------------------ W162 scenario

/// Wolf (mount 993) and the item definition that grants it; 14289 exists in the captured catalogue.
const WOLF: i32 = 993;
const WOLF_ITEM: i64 = 14289;
const WAVE: i32 = 135;

fn ride_of(character: u64) -> u64 {
    osfr_tools::mounts::MountService::ride_guid(character)
}
fn spawn_request(mount: i32) -> Vec<u8> {
    [vec![168, 0, 6], mount.to_le_bytes().to_vec()].concat()
}
fn board_request(ride: u64) -> Vec<u8> {
    [
        vec![168, 0, 1],
        ride.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat()
}
fn is_ride_actor(m: &[u8], ride: u64) -> bool {
    m.starts_with(&[35, 0, 2, 0]) && m.get(4..12) == Some(&ride.to_le_bytes()[..])
}
fn is_ride_removal(m: &[u8], ride: u64) -> bool {
    m.starts_with(&[35, 0, 3, 0, 1, 0]) && m.get(6..14) == Some(&ride.to_le_bytes()[..])
}
fn is_mount_reply(m: &[u8], sub: u8, rider: u64) -> bool {
    m.starts_with(&[168, 0, sub]) && m.get(3..11) == Some(&rider.to_le_bytes()[..])
}

#[test]
fn riders_are_seen_by_neighbours_and_late_arrivals_and_a_mounted_emote_is_dropped() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Player; 3], false);
    // The first login writes the save; give player A the wolf ride item while logged out.
    disconnect(rig.connect(0, "mountA0"));
    std::thread::sleep(Duration::from_millis(1500));
    seed_items(&rig, 0, &[(900_001, WOLF_ITEM)]);

    let (ga, gb) = (rig.accounts[0].guid, rig.accounts[1].guid);
    let ride = ride_of(ga);
    let mut a = rig.connect(0, "mountA");
    let mut b = rig.connect(1, "mountB");
    a.pump(Duration::from_millis(500)).unwrap();
    b.pump(Duration::from_millis(500)).unwrap();

    // B owns no ride: refused with a notice, and A never sees a ride for B.
    raw(&mut b, &spawn_request(WOLF));
    b.wait_for(T, |m| {
        system_text(m).is_some_and(|t| t.contains("do not own"))
    })
    .unwrap_or_else(|e| panic!("no refusal ({e}); got {}", tail(&b)));
    assert!(
        a.wait_for(Duration::from_secs(1), |m| is_ride_actor(m, ride_of(gb)))
            .is_err(),
        "an unowned ride appeared"
    );

    // A summons: the roster (private) then the ride actor; B, in range, sees the actor too.
    raw(&mut a, &spawn_request(WOLF));
    a.wait_for(T, |m| m.starts_with(&[168, 0, 5]))
        .unwrap_or_else(|e| panic!("no roster ({e}); got {}", tail(&a)));
    a.wait_for(T, |m| is_ride_actor(m, ride))
        .expect("A: ride actor");
    b.wait_for(T, |m| is_ride_actor(m, ride))
        .expect("B: ride actor");

    // The client automounts with 168/1; the boarding reaches both players.
    raw(&mut a, &board_request(ride));
    a.wait_for(T, |m| is_mount_reply(m, 2, ga))
        .expect("A: boarded");
    b.wait_for(T, |m| is_mount_reply(m, 2, ga))
        .expect("B: sees A board");

    // A mounted player's emote is dropped; B hears nothing.
    raw(&mut a, &quick_chat(WAVE, "FixtureA"));
    assert!(
        b.wait_for(Duration::from_secs(2), |m| is_quick_chat(m, WAVE))
            .is_err(),
        "a mounted emote was delivered"
    );

    // C arrives later and is shown the ride and the rider's attachment.
    let mut c = rig.connect(2, "mountC");
    c.wait_for(T, |m| is_ride_actor(m, ride))
        .unwrap_or_else(|e| panic!("C: no ride actor ({e}); got {}", tail(&c)));
    c.wait_for(T, |m| is_mount_reply(m, 2, ga))
        .expect("C: sees A mounted");

    // One 168/3 dismounts and removes the ride for everybody; the emote works again.
    raw(&mut a, &[168, 0, 3]);
    for (who, bot) in [("A", &mut a), ("B", &mut b), ("C", &mut c)] {
        bot.wait_for(T, |m| is_mount_reply(m, 4, ga))
            .unwrap_or_else(|e| panic!("{who}: no dismount ({e})"));
        bot.wait_for(T, |m| is_ride_removal(m, ride))
            .unwrap_or_else(|e| panic!("{who}: ride not removed ({e})"));
    }
    raw(&mut a, &quick_chat(WAVE, "FixtureA"));
    b.wait_for(T, |m| is_quick_chat(m, WAVE))
        .expect("the emote stayed refused after dismounting");

    // Leaving takes the ride away as well.
    std::thread::sleep(Duration::from_millis(1200)); // server spawn debounce (1 s)
    raw(&mut a, &spawn_request(WOLF));
    b.wait_for(T, |m| is_ride_actor(m, ride))
        .expect("B: second ride actor");
    disconnect(a);
    b.wait_for(T, |m| is_ride_removal(m, ride))
        .unwrap_or_else(|e| panic!("B: ride stayed after A left ({e})"));
    disconnect(b);
    disconnect(c);
}

// ------------------------------------------------------------------ W167 scenario

fn is_move_of(m: &[u8], guid: u64) -> bool {
    m.starts_with(&[125, 0]) && m.get(2..10) == Some(&guid.to_le_bytes()[..])
}

fn is_add_pc_of(m: &[u8], guid: u64) -> bool {
    m.starts_with(&[35, 0, 1, 0]) && m.get(4..12) == Some(&guid.to_le_bytes()[..])
}

/// `bot` walks to `from` shifted by each `dx`; each step is one 125 to the server.
fn walk(bot: &mut bot::Bot, guid: u64, from: [f32; 3], steps: &[f32]) {
    for dx in steps {
        let to = [from[0] + dx, from[1], from[2]];
        raw(bot, &bot::move_request(guid, to, [0.0, 0.0, 1.0]));
        bot.pump(Duration::from_millis(300)).unwrap();
    }
}

#[test]
fn the_ride_moves_with_its_rider_and_a_late_arrival_gets_the_native_mount_tail() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Player; 3], false);
    disconnect(rig.connect(0, "followA0"));
    std::thread::sleep(Duration::from_millis(1500));
    seed_items(&rig, 0, &[(900_001, WOLF_ITEM)]);

    let ga = rig.accounts[0].guid;
    let ride = ride_of(ga);
    let start = rig.start_position;
    let mut a = rig.connect(0, "followA");
    let mut b = rig.connect(1, "followB");
    a.pump(Duration::from_millis(500)).unwrap();
    b.pump(Duration::from_millis(500)).unwrap();

    // Summoned but not boarded: the ride stands where it appeared, so B gets A's move only.
    raw(&mut a, &spawn_request(WOLF));
    b.wait_for(T, |m| is_ride_actor(m, ride))
        .expect("B: ride actor");
    walk(&mut a, ga, start, &[0.5]);
    b.wait_for(T, |m| is_move_of(m, ga))
        .unwrap_or_else(|e| panic!("B: no move of A ({e}); got {}", tail(&b)));
    assert!(
        b.wait_for(Duration::from_secs(1), |m| is_move_of(m, ride))
            .is_err(),
        "an unboarded ride followed"
    );

    // Boarded: every move of A reaches B followed by the ride's copy, same pose.
    raw(&mut a, &board_request(ride));
    b.wait_for(T, |m| is_mount_reply(m, 2, ga))
        .expect("B: sees A board");
    walk(&mut a, ga, start, &[1.0]);
    let rider_move = b
        .wait_for(T, |m| is_move_of(m, ga))
        .unwrap_or_else(|e| panic!("B: no move of A ({e}); got {}", tail(&b)));
    let ride_move = b
        .wait_for(T, |m| is_move_of(m, ride))
        .unwrap_or_else(|e| panic!("B: the ride did not follow ({e}); got {}", tail(&b)));
    assert_eq!(ride_move[10..], rider_move[10..], "same pose and state");

    // C arrives later: A's AddPc carries the native mount block, then the ride actor and 168/2.
    let mut c = rig.connect(2, "followC");
    let add_pc = c
        .wait_for(T, |m| is_add_pc_of(m, ga))
        .unwrap_or_else(|e| panic!("C: no AddPc of A ({e}); got {}", tail(&c)));
    let block = &add_pc[add_pc.len() - 40..add_pc.len() - 20];
    assert_eq!(block[..8], ride.to_le_bytes(), "mount guid");
    assert_eq!(block[8..12], 0_i32.to_le_bytes(), "seat");
    assert_eq!(block[12..16], 1_i32.to_le_bytes(), "queue position");
    assert_eq!(block[16..], 0.0_f32.to_le_bytes(), "lift");
    c.wait_for(T, |m| is_ride_actor(m, ride))
        .expect("C: ride actor");
    c.wait_for(T, |m| is_mount_reply(m, 2, ga))
        .expect("C: sees A mounted");
    walk(&mut a, ga, start, &[1.5]);
    c.wait_for(T, |m| is_move_of(m, ride))
        .unwrap_or_else(|e| panic!("C: the ride did not follow ({e}); got {}", tail(&c)));

    // Put away: the ride is gone and A's next move is A's alone.
    raw(&mut a, &[168, 0, 3]);
    b.wait_for(T, |m| is_ride_removal(m, ride))
        .expect("B: ride removed");
    walk(&mut a, ga, start, &[2.0]);
    b.wait_for(T, |m| is_move_of(m, ga))
        .unwrap_or_else(|e| panic!("B: no move of A ({e}); got {}", tail(&b)));
    assert!(
        b.wait_for(Duration::from_secs(1), |m| is_move_of(m, ride))
            .is_err(),
        "a put-away ride kept following"
    );
    disconnect(a);
    disconnect(b);
    disconnect(c);
}
