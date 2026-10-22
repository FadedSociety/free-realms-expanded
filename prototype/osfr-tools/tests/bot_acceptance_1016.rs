//! W154: bot acceptance for the 2026-10-16 features, over real sockets.
//!
//! Every scenario starts the real `osfr-gateway` binary (and `osfr-login` for
//! the broker scenario) on loopback, with an account store and the private
//! opening content, and drives `osfr-bot` (the library for message-level
//! checks, the CLI for the key-pair broker flow). Nothing here is mocked.
//!
//! Inputs (private, not in Git): `lab/references/rust-gateway-reference` and
//! `lab/reports/round70-native-20261016T061000Z/content`, overlaid on
//! `data/content` exactly as `scripts/stress/run.py` does. If either is
//! missing the rig panics with the path; the scenarios are not ignored because
//! both exist on the dev machine.
//!
//! Server rules proved here are reconstructed policy (R) unless the feature's
//! own test file says otherwise; this file only proves the wiring.
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
/// How long a save may take to show a change (the gateway can be starved).
const SAVE_WAIT: Duration = Duration::from_secs(30);
/// How long a negative check listens after the server has answered the
/// request. Two seconds, so a starved gateway cannot hide a late packet.
const QUIET: Duration = Duration::from_secs(2);
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
    // The bots are strict about timing (a 5 s handshake window) and the server
    // under test builds a whole session per handshake, so on a machine busy with
    // builds and other test binaries the services must not queue behind them.
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
        Self::start_with(roles, broker, true)
    }

    /// `trim_npcs` keeps only the starting village's NPCs (see below).
    fn start_with(roles: &[AccountRole], broker: bool, trim_npcs: bool) -> Self {
        Self::start_with_default_job(roles, broker, trim_npcs, None)
    }

    /// T202: a scenario whose content is offered only to one active job starts the
    /// captured default profile state on that job instead of the Brawler default.
    fn start_with_default_job(
        roles: &[AccountRole],
        broker: bool,
        trim_npcs: bool,
        default_job: Option<i32>,
    ) -> Self {
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
        if let Some(job) = default_job {
            let path = content.join("profiles.json");
            let mut profiles: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            let state = profiles["default_state"].as_object_mut().unwrap();
            state["active"] = json!(job);
            let unlocked = state["unlocked"].as_array_mut().unwrap();
            if !unlocked.contains(&json!(job)) {
                unlocked.push(json!(job));
            }
            fs::write(&path, serde_json::to_vec(&profiles).unwrap()).unwrap();
        }
        // The gateway builds a whole gameplay (24 MB of NPC JSON, parsed three
        // times) for every session, in a debug build: 2.5 to 8 s per bot depending
        // on load, past the bot's 5 s handshake window. No scenario needs more than
        // the starting village's NPCs, so the rest are left out.
        if trim_npcs {
            for entry in fs::read_dir(content.join("npcs")).unwrap() {
                let path = entry.unwrap().path();
                if path.file_name().unwrap() != "cobblestone_village.json" {
                    fs::remove_file(path).unwrap();
                }
            }
            let village: Value = serde_json::from_slice(
                &fs::read(content.join("npcs/cobblestone_village.json")).unwrap(),
            )
            .unwrap();
            let kept: Vec<_> = village["records"]
                .as_array()
                .unwrap()
                .iter()
                .map(|npc| npc["npc_id"].clone())
                .collect();
            let vendors_path = content.join("vendors.json");
            let mut vendors: Value =
                serde_json::from_slice(&fs::read(&vendors_path).unwrap()).unwrap();
            vendors["vendors"]
                .as_array_mut()
                .unwrap()
                .retain(|vendor| kept.contains(&vendor["npc_id"]));
            fs::write(&vendors_path, serde_json::to_vec(&vendors).unwrap()).unwrap();
        }

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

    /// The character's save file, as the gateway wrote it. A read that fails
    /// (the gateway replaces the file atomically; Windows can briefly deny a
    /// reader) is retried for a few seconds before it counts as a failure.
    fn save(&self, account: usize) -> Value {
        self.save_when(None, account, "any save", |_| true)
    }

    fn save_path(&self, account: usize) -> PathBuf {
        self.root.join(format!(
            "state/characters/{}.json",
            self.accounts[account].guid
        ))
    }

    /// Poll the save until `ready` accepts it (the gateway coalesces pose
    /// saves). Pass the live `bot` so its socket is serviced while we wait: a
    /// bot that only sleeps never acknowledges or retransmits, so one lost or
    /// late datagram would stall the very request being waited for.
    fn save_when(
        &self,
        mut bot: Option<&mut bot::Bot>,
        account: usize,
        what: &str,
        ready: impl Fn(&Value) -> bool,
    ) -> Value {
        let deadline = Instant::now() + SAVE_WAIT;
        let mut last;
        loop {
            match fs::read(self.save_path(account)) {
                Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                    Ok(value) if ready(&value) => return value,
                    Ok(value) => last = format!("save never showed {what}: {value:#}"),
                    Err(e) => last = format!("save for {what} is not JSON: {e}"),
                },
                Err(e) => last = format!("save for {what} unreadable: {e}"),
            }
            if Instant::now() > deadline {
                panic!("{last}");
            }
            match bot.as_deref_mut() {
                Some(bot) => bot.pump(Duration::from_millis(100)).unwrap(),
                None => std::thread::sleep(Duration::from_millis(100)),
            }
        }
    }

    /// Client-initiated session ends the gateway has logged. `end_session`
    /// writes the character save before printing its event, so a seed written
    /// after this count increases is the logged-out state, not the login's.
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

    /// Wait until a gateway stdout line is JSON for which `ready` is true.
    /// Gateway events (`event()`, `npc_event()`) are one JSON line each. Pump
    /// the live bot while waiting so a dropped request can be retransmitted.
    fn wait_log(&self, bot: &mut bot::Bot, what: &str, ready: impl Fn(&Value) -> bool) {
        let path = self.root.join("gateway.out.log");
        let deadline = Instant::now() + T;
        loop {
            if let Ok(text) = fs::read_to_string(&path) {
                if text
                    .lines()
                    .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                    .any(|value| ready(&value))
                {
                    return;
                }
            }
            assert!(
                Instant::now() < deadline,
                "{what}: nothing matching in {}",
                path.display()
            );
            bot.pump(Duration::from_millis(100)).unwrap();
        }
    }

    /// Count one gateway event/outcome pair; used to prove a refusal reached the
    /// handler without changing the character save.
    fn log_outcomes(&self, event: &str, outcome: &str) -> usize {
        fs::read_to_string(self.root.join("gateway.out.log"))
            .map(|text| {
                text.lines()
                    .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                    .filter(|value| value["event"] == event && value["outcome"] == outcome)
                    .count()
            })
            .unwrap_or(0)
    }

    fn wait_log_outcomes(&self, what: &str, event: &str, outcome: &str, at_least: usize) {
        let deadline = Instant::now() + T;
        while self.log_outcomes(event, outcome) < at_least {
            assert!(
                Instant::now() < deadline,
                "{what}: no gateway event/outcome"
            );
            std::thread::sleep(Duration::from_millis(50));
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
            // A handshake that went unanswered is still queued at the gateway,
            // which builds a full session for it before it can look at the next
            // datagram; a prompt retry only lengthens that queue. Let it drain.
            let unanswered = last.contains("no SessionResponse");
            std::thread::sleep(Duration::from_millis(if unanswered { 8000 } else { 700 }));
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
        #[cfg(windows)]
        std::os::windows::process::CommandExt::creation_flags(&mut command, 0x0000_8000);
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

// ----------------------------------------------------------- bot helpers

/// Text of a 15/3 system chat message.
fn system_text(message: &[u8]) -> Option<String> {
    if message.get(..4)? != [15, 0, 3, 0] {
        return None;
    }
    let len = u32::from_le_bytes(message.get(4..8)?.try_into().ok()?) as usize;
    Some(String::from_utf8_lossy(message.get(8..8 + len)?).into_owned())
}

fn say(bot: &mut bot::Bot, text: &str) {
    let message = bot::wrap(&osfr_tools::chat::local_request(text));
    bot.send_message(&message, text).unwrap();
}

fn raw(bot: &mut bot::Bot, inner: &[u8]) {
    bot.send_message(&bot::wrap(inner), "raw").unwrap();
}

/// Send chat and wait for a system line containing `needle`.
fn ask(bot: &mut bot::Bot, text: &str, needle: &str) -> String {
    say(bot, text);
    let reply = bot
        .wait_for(T, |m| system_text(m).is_some_and(|t| t.contains(needle)))
        .unwrap_or_else(|e| panic!("`{text}` never answered `{needle}`: {e}"));
    system_text(&reply).unwrap()
}

/// Processing barrier: a read-only builder request. The gateway handles datagrams
/// in order on one thread, so its reply is queued only after every earlier
/// request's transaction, including any save that request scheduled, finished.
fn barrier(bot: &mut bot::Bot) {
    let _ = ask(bot, "realm here", "Zone ");
}

/// A server `49/10 ObjectiveComplete`. The bot records only headers.
fn is_objective_complete(message: &(u16, Option<u16>, usize)) -> bool {
    message.0 == 49 && message.1 == Some(10)
}

/// `realm here` (builder/admin): the server's own idea of our position.
fn here(bot: &mut bot::Bot) -> [f32; 3] {
    let text = ask(bot, "realm here", "Zone ");
    let number = |tag: &str| -> f32 {
        let at = text
            .find(tag)
            .unwrap_or_else(|| panic!("no `{tag}` in {text}"))
            + tag.len();
        text[at..]
            .split(|c: char| c != '-' && c != '.' && !c.is_ascii_digit())
            .next()
            .unwrap()
            .trim_end_matches('.')
            .parse()
            .unwrap_or_else(|_| panic!("bad number after `{tag}` in {text}"))
    };
    [number(" x "), number(" y "), number(" z ")]
}

fn pose(bot: &mut bot::Bot, guid: u64, at: [f32; 3]) {
    let message = bot::wrap(&bot::move_request(guid, at, [0.0, 0.0, 1.0]));
    bot.send_message(&message, &format!("move {at:?}")).unwrap();
    bot.pump(Duration::from_millis(400)).unwrap();
}

/// Walk in steps of at most 10 m, 400 ms apart (inside the movement limit).
fn walk(bot: &mut bot::Bot, guid: u64, from: [f32; 3], to: [f32; 3]) {
    let distance = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
    let steps = (distance / 10.0).ceil().max(1.0) as usize;
    for step in 1..=steps {
        let f = step as f32 / steps as f32;
        pose(
            bot,
            guid,
            [0, 1, 2].map(|i| from[i] + (to[i] - from[i]) * f),
        );
    }
}

fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn is_add_pc_of(message: &[u8], guid: u64) -> bool {
    message.starts_with(&[35, 0, 1, 0]) && message.get(4..12) == Some(&guid.to_le_bytes()[..])
}

/// Wait until `bot`'s field view contains `guid`. A login can return before
/// the gateway has joined the bot, and a state change sent in between (a chat
/// line) is not replayed to the late joiner.
fn sees(bot: &mut bot::Bot, guid: u64, what: &str) {
    bot.wait_for(T, |m| is_add_pc_of(m, guid))
        .unwrap_or_else(|e| panic!("{what}: no AddPc ({e}); got {}", tail(bot)));
}

/// The last few (family, sub, length) triples a bot received, for failure messages.
fn tail(bot: &bot::Bot) -> String {
    let from = bot.received.len().saturating_sub(8);
    format!("{:?}", &bot.received[from..])
}

fn disconnect(mut bot: bot::Bot) {
    bot.disconnect();
}

/// Disconnect `bot` and wait until the gateway has written its logout save.
/// The SOE disconnect is an unreliable datagram, so it is resent while waiting;
/// a repeated disconnect for an absent session is ignored.
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

fn text_of(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// ------------------------------------------------------------ scenario 1

/// Decision 1 (`264db2a`): over the key-pair broker, a builder account runs a
/// builder `realm` command on a gateway started WITHOUT `--dev-tools`; the same
/// command is refused for a player account.
#[test]
fn builder_realm_command_runs_without_dev_tools_and_player_is_refused() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Builder, AccountRole::Player], true);
    assert_eq!(
        rig.gateway_event["dev_tools"], false,
        "gateway must not run --dev-tools"
    );

    let builder = rig.cli(0, &["chat realm here", "expect-text Zone 0 at x", "quit"]);
    let out = text_of(&builder);
    assert!(
        builder.status.success(),
        "builder `realm here` failed:\n{out}"
    );
    assert!(
        out.contains("native login selected character"),
        "not via the broker:\n{out}"
    );

    let player = rig.cli(
        1,
        &[
            "chat realm here",
            "expect-text your account role does not allow this command",
            "quit",
        ],
    );
    let out = text_of(&player);
    assert!(
        player.status.success(),
        "player was not refused with the role line:\n{out}"
    );

    // Canary: the player never receives the success line (the refusal above is
    // the rule, not a coincidence of text matching).
    let canary = rig.cli(1, &["chat realm here", "expect-text Zone 0 at x", "quit"]);
    assert!(
        !canary.status.success(),
        "player got builder output:\n{}",
        text_of(&canary)
    );

    // The refusals are logged (decision 1): one `dev_command` authorization event each.
    let log = rig.root.join("security.jsonl");
    let deadline = Instant::now() + Duration::from_secs(30);
    while fs::read_to_string(&log).map_or(0, |t| t.matches("dev_command").count()) < 2 {
        assert!(
            Instant::now() < deadline,
            "no dev_command events in the security log"
        );
        std::thread::sleep(Duration::from_millis(200));
    }

    // A builder is still limited to building commands, so cheats stay admin-only.
    let cheat = rig.cli(
        0,
        &[
            "chat realm god",
            "expect-text your account role does not allow this command",
            "quit",
        ],
    );
    assert!(
        cheat.status.success(),
        "builder `realm god` not refused:\n{}",
        text_of(&cheat)
    );
}

// ------------------------------------------------------------ scenario 6

/// 4ba97ad: pose-only saves are coalesced, but a logout flushes the newest pose
/// and a reconnect restores it.
#[test]
fn walk_disconnect_reconnect_restores_the_last_pose() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Builder], false);
    let guid = rig.accounts[0].guid;
    let mut bot = rig.connect(0, "pose1");
    let logged_in = Instant::now();
    let start = here(&mut bot);
    let steps = [
        [start[0] + 3.0, start[1], start[2]],
        [start[0] + 6.0, start[1], start[2]],
        [start[0] + 9.0, start[1], start[2] + 3.0],
    ];
    for at in steps {
        pose(&mut bot, guid, at);
    }
    let last = steps[2];
    assert!(
        planar(here(&mut bot), last) < 0.2,
        "server did not accept the walk"
    );
    // Coalescing: inside the 5 s window the file still holds an older pose. On a
    // starved machine the window can pass before we look (the tick then writes
    // the newest pose, which is correct), so the check only runs when we are
    // clearly inside it. The logout flush and the restore below always run.
    let mid = rig.save(0)["character"]["position"].clone();
    let mid: Vec<f64> = serde_json::from_value(mid).unwrap();
    if logged_in.elapsed() < Duration::from_millis(3500) {
        assert!(
            (mid[0] as f32 - last[0]).abs() > 1.0,
            "the newest pose was saved at once, nothing was coalesced: {mid:?}"
        );
    }
    disconnect(bot);
    let saved = rig.save_when(None, 0, "the last pose after logout", |v| {
        v["character"]["position"][0]
            .as_f64()
            .is_some_and(|x| (x as f32 - last[0]).abs() < 0.1)
    });
    assert!((saved["character"]["position"][2].as_f64().unwrap() as f32 - last[2]).abs() < 0.1);

    let mut again = rig.connect(0, "pose2");
    let restored = here(&mut again);
    assert!(
        planar(restored, last) < 0.2,
        "restored {restored:?}, walked to {last:?}"
    );
    disconnect(again);
}

// ------------------------------------------------------------ scenario 2

fn equip_request(item: i32, profile: i32, slot: i32) -> Vec<u8> {
    let mut out = vec![42, 0, 3, 0];
    for value in [item, profile, slot] {
        out.extend(value.to_le_bytes());
    }
    out
}

/// The item id worn in `profile`'s `slot`, from a character save.
fn worn(save: &Value, profile: i64, slot: i64) -> i64 {
    save["gameplay"]["inventory"]["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["profile_id"] == profile && e["slot"] == slot)
        .unwrap_or_else(|| panic!("no equipment row {profile}/{slot}"))["item_id"]
        .as_i64()
        .unwrap()
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

/// W138: an owned, eligible item equips (38/5 + 35/6, saved); owned items the
/// character may not wear are refused with one chat line each and nothing changes.
#[test]
fn eligible_equip_succeeds_and_ineligible_equip_is_refused_without_state_change() {
    let _serial = serial();
    // Definitions from the round70 catalogue (rows decoded with the W138 layout):
    // 76476 is a plain slot-4 item for profile 1, 76497 a slot-3 item for girls
    // (the character is a boy), 75859 a slot-3 item needing rank 4 on profile 43.
    const PLAIN: i64 = 101;
    const GIRLS_ONLY: i64 = 102;
    const RANK_4: i64 = 103;
    let rig = Rig::start(&[AccountRole::Player], false);
    // The first login writes the save; the logout must be finished (and the
    // save written) before the seeded items are added.
    logout(&rig, rig.connect(0, "equip0"), "the first login");
    seed_items(
        &rig,
        0,
        &[(PLAIN, 76476), (GIRLS_ONLY, 76497), (RANK_4, 75859)],
    );

    let mut bot = rig.connect(0, "equip");
    let before = rig.save(0);
    assert!(
        before["gameplay"]["inventory"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == PLAIN),
        "the gateway did not load the seeded items"
    );
    let refusal = |bot: &mut bot::Bot, item: i64, profile: i64, slot: i64, text: &str| {
        raw(
            bot,
            &equip_request(item as i32, profile as i32, slot as i32),
        );
        // Match the refusal or the accepted acknowledgement, so an out-of-order
        // 38/5 cannot slip past the assertion below.
        let answered = bot
            .wait_for(T, |m| {
                system_text(m).is_some_and(|t| t == text) || m.starts_with(&[38, 0, 5, 0])
            })
            .unwrap_or_else(|e| panic!("no `{text}` for item {item} ({e}); got {}", tail(bot)));
        assert!(
            system_text(&answered).is_some_and(|t| t == text),
            "refused equip of {item} still answered 38/5"
        );
        assert!(
            bot.wait_for(QUIET, |m| m.starts_with(&[38, 0, 5, 0]))
                .is_err(),
            "refused equip of {item} still answered 38/5"
        );
    };
    refusal(&mut bot, GIRLS_ONLY, 1, 3, "This item is only for girls.");
    refusal(&mut bot, RANK_4, 43, 3, "This item needs class rank 4.");
    assert_eq!(
        rig.save(0)["gameplay"]["inventory"]["equipment"],
        before["gameplay"]["inventory"]["equipment"],
        "a refusal changed the equipment"
    );

    // Canary: the same route equips an eligible item, so the refusals were the rule.
    raw(&mut bot, &equip_request(PLAIN as i32, 1, 4));
    bot.wait_for(T, |m| m.starts_with(&[38, 0, 5, 0]))
        .unwrap_or_else(|e| panic!("no 38/5 after the eligible equip ({e}); got {}", tail(&bot)));
    let saved = rig.save_when(Some(&mut bot), 0, "the equip", |v| worn(v, 1, 4) == PLAIN);
    assert_eq!(
        worn(&saved, 1, 3),
        worn(&before, 1, 3),
        "a refused item leaked into the save"
    );
    assert_eq!(
        worn(&saved, 43, 3),
        worn(&before, 43, 3),
        "a refused item leaked into the save"
    );
    disconnect(bot);
}

// ------------------------------------------------------------ scenario 3

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

/// W139: a quick-chat emote reaches a second player. L038: so does Dance (184),
/// whose animation 3200 is the client's `emo_dance` group.
#[test]
fn emote_and_dance_reach_a_second_bot() {
    let _serial = serial();
    const WAVE: i32 = 135;
    const DANCE: i32 = 184;
    let rig = Rig::start(&[AccountRole::Player, AccountRole::Player], false);
    let mut a = rig.connect(0, "emoteA");
    let mut b = rig.connect(1, "emoteB");
    // Both sessions must be joined before A's chat: a bot that joins after
    // the broadcast never receives it.
    sees(&mut a, rig.accounts[1].guid, "emoteA: B enters the field");
    sees(&mut b, rig.accounts[0].guid, "emoteB: A is in the field");

    raw(&mut a, &quick_chat(WAVE, "FixtureA"));
    b.wait_for(T, |m| is_quick_chat(m, WAVE))
        .expect("emote 135 not delivered to the second bot");

    raw(&mut a, &quick_chat(DANCE, "FixtureA"));
    b.wait_for(T, |m| is_quick_chat(m, DANCE))
        .expect("Dance (184) not delivered to the second bot");
    disconnect(a);
    disconnect(b);
}

// ------------------------------------------------------------ scenario 4

const SANCTUARY: [f32; 3] = [-1414.636, -27.631, 351.567];
const LAKESHORE: [f32; 3] = [-733.952, 9.09, 945.973];

fn travel(id: i32) -> Vec<u8> {
    let mut out = 90u16.to_le_bytes().to_vec();
    out.extend(id.to_le_bytes());
    out
}

/// W142: walking within 12 m attunes a warpstone; `90` then relocates to it
/// and the new position is saved; a locked id moves nobody. Attunement survives
/// a relog. Teleports around the map use the builder `realm goto`.
#[test]
fn warpstone_attunes_by_walking_travel_relocates_and_locked_ids_do_not() {
    let _serial = serial();
    const NOT_ATTUNED: &str = "have not attuned";
    let rig = Rig::start(&[AccountRole::Builder], false);
    let guid = rig.accounts[0].guid;
    let goto = |bot: &mut bot::Bot, at: [f32; 3]| {
        ask(
            bot,
            &format!("realm goto {} {} {}", at[0], at[1], at[2]),
            "Developer teleport",
        );
        assert!(planar(here(bot), at) < 0.5, "goto did not land at {at:?}");
    };

    let mut bot = rig.connect(0, "warp1");
    goto(&mut bot, LAKESHORE);
    // Nothing is attuned yet: the request is refused and nobody moves.
    raw(&mut bot, &travel(5));
    bot.wait_for(T, |m| {
        system_text(m).is_some_and(|t| t.contains(NOT_ATTUNED))
    })
    .expect("travel before attuning was not refused");
    assert!(
        planar(here(&mut bot), LAKESHORE) < 0.5,
        "refused travel moved the player"
    );

    // Walk toward Sanctuary: 20 m away is outside the 12 m radius.
    let far = [SANCTUARY[0] - 45.0, SANCTUARY[1], SANCTUARY[2]];
    goto(&mut bot, far);
    let near20 = [SANCTUARY[0] - 20.0, SANCTUARY[1], SANCTUARY[2]];
    walk(&mut bot, guid, far, near20);
    raw(&mut bot, &travel(5));
    bot.wait_for(T, |m| {
        system_text(m).is_some_and(|t| t.contains(NOT_ATTUNED))
    })
    .expect("20 m from the stone already attuned it");
    // Eight metres away attunes it, with the notice and a refreshed list (57).
    let near8 = [SANCTUARY[0] - 8.0, SANCTUARY[1], SANCTUARY[2]];
    let message = bot::wrap(&bot::move_request(guid, near8, [0.0, 0.0, 1.0]));
    bot.send_message(&message, "move to 8 m").unwrap();
    let notice = bot
        .wait_for(T, |m| {
            system_text(m).is_some_and(|t| t.contains("attuned to the Sanctuary"))
        })
        .expect("no attunement notice within 12 m");
    assert!(
        system_text(&notice)
            .unwrap()
            .contains("Sanctuary Warpstone")
    );
    rig.save_when(Some(&mut bot), 0, "the attunement", |v| {
        v["gameplay"]["warpstones"]["attuned"] == json!([5])
    });

    // Far away again: a locked id moves nobody, the attuned id relocates.
    goto(&mut bot, LAKESHORE);
    raw(&mut bot, &travel(4));
    bot.wait_for(T, |m| {
        system_text(m).is_some_and(|t| t.contains(NOT_ATTUNED))
    })
    .expect("locked destination 4 was not refused");
    assert!(
        planar(here(&mut bot), LAKESHORE) < 0.5,
        "locked travel moved the player"
    );
    raw(&mut bot, &travel(5));
    bot.wait_for(T, |m| m.starts_with(&[38, 0, 12, 0]))
        .expect("no 38/12 location update");
    let arrived = here(&mut bot);
    assert!(
        planar(arrived, SANCTUARY) < 2.0,
        "arrived at {arrived:?}, not Sanctuary"
    );
    disconnect(bot);

    // The new position was saved, and the attunement survives the relog.
    let saved = rig.save_when(None, 0, "the arrival position", |v| {
        v["character"]["position"][0]
            .as_f64()
            .is_some_and(|x| (x as f32 - SANCTUARY[0]).abs() < 2.0)
    });
    assert_eq!(saved["gameplay"]["warpstones"]["attuned"], json!([5]));
    let mut again = rig.connect(0, "warp2");
    assert!(
        planar(here(&mut again), SANCTUARY) < 2.0,
        "relog did not restore the arrival"
    );
    goto(&mut again, LAKESHORE);
    raw(&mut again, &travel(5));
    again
        .wait_for(T, |m| m.starts_with(&[38, 0, 12, 0]))
        .expect("travel after relog");
    disconnect(again);
}

// ------------------------------------------------------------ scenario 5

struct QuestPlan {
    quest: u32,
    npc: u64,
    at: [f32; 3],
}

/// The enabled, prerequisite-free quest whose giver is nearest `start`
/// (the same pick as `scripts/stress/run.py`).
fn plan_quest(content: &Path, start: [f32; 3]) -> QuestPlan {
    let read = |dir: &str| -> Vec<Value> {
        let mut rows = Vec::new();
        for entry in fs::read_dir(content.join(dir)).unwrap() {
            let file: Value =
                serde_json::from_slice(&fs::read(entry.unwrap().path()).unwrap()).unwrap();
            rows.extend(file["records"].as_array().cloned().unwrap_or_default());
        }
        rows
    };
    let npcs = read("npcs");
    let mut best: Option<(f32, QuestPlan)> = None;
    for quest in read("quests") {
        let id = quest["id"].as_u64().unwrap();
        let giver = quest["giver_npc_id"].as_u64();
        let Some(npc) = npcs.iter().find(|n| n["npc_id"].as_u64() == giver) else {
            continue;
        };
        let Some(position) = npc["position"].as_array().filter(|p| p.len() == 3) else {
            continue;
        };
        let at = [0, 1, 2].map(|i| position[i].as_f64().unwrap() as f32);
        let given = npc["quest_ids_given"]
            .as_array()
            .is_some_and(|ids| ids.iter().any(|g| *g == id));
        // A job-gated quest is not offerable to this rig's default profile (T252's Brawler-only
        // C1 at Carlos), so the plan keeps to quests the default character can accept.
        let free = quest["prerequisite_quest_ids"]
            .as_array()
            .is_none_or(|p| p.is_empty())
            && quest["enabled"].as_bool().unwrap_or(true)
            && quest["job"].is_null();
        let distance = planar(start, at);
        if given && free && best.as_ref().is_none_or(|(d, _)| distance < *d) {
            best = Some((
                distance,
                QuestPlan {
                    quest: id as u32,
                    npc: giver.unwrap(),
                    at,
                },
            ));
        }
    }
    best.expect("no quest with a positioned giver in the content")
        .1
}

fn npc_packet(sub: u8, npc: u64, tail: &[u8]) -> Vec<u8> {
    let mut out = vec![26, 0, sub, 0];
    out.extend(npc.to_le_bytes());
    out.extend(tail);
    out
}

/// W145: 47/12 selects an owned active quest; the selection is saved and is
/// still there after a relog. A forged id changes nothing.
#[test]
fn quest_journal_selection_persists_across_relog() {
    let _serial = serial();
    let rig = Rig::start(&[AccountRole::Player], false);
    let plan = plan_quest(&rig.content, rig.start_position);
    let guid = rig.accounts[0].guid;
    let selected = |save: &Value| save["gameplay"]["npc_quests"]["selected_quest_id"].clone();
    let status = |save: &Value| {
        save["gameplay"]["npc_quests"]["quests"][plan.quest.to_string()]["status"].clone()
    };
    let select = |id: i32| [&[47u8, 0, 12][..], &id.to_le_bytes()].concat();

    let mut bot = rig.connect(0, "quest1");
    walk(&mut bot, guid, rig.start_position, plan.at);
    // Take the quest from its giver the way the stress bots do.
    for inner in [
        npc_packet(8, plan.npc, &[0]),
        npc_packet(10, plan.npc, &1i32.to_le_bytes()),
        [&[26u8, 0, 6, 0][..], &301i32.to_le_bytes()].concat(),
        [&[49u8, 0, 2, 0, 0, 0][..], &plan.quest.to_le_bytes(), &[1]].concat(),
    ] {
        raw(&mut bot, &inner);
    }
    let active = rig.save_when(Some(&mut bot), 0, "the accepted quest", |v| {
        status(v) == "active"
    });
    assert_eq!(
        selected(&active),
        Value::Null,
        "accepting must not select the quest"
    );

    // A quest the character does not own cannot be selected. The gateway logs
    // the journal outcome before the handler returns, so wait for that line
    // rather than guessing how long the request takes.
    bot.drop_next_matching(&[47, 0, 12]);
    raw(&mut bot, &select(999_999));
    rig.wait_log(&mut bot, "the forged selection", |value| {
        value["event"] == "quest_journal"
            && value["outcome"] == "rejected"
            && value["request"]
                .as_str()
                .is_some_and(|request| request.contains("999999"))
    });
    assert!(
        bot.dropped_datagram(),
        "the first forged selection was not dropped"
    );
    assert_eq!(
        selected(&rig.save(0)),
        Value::Null,
        "a forged id was selected"
    );

    raw(&mut bot, &select(plan.quest as i32));
    rig.save_when(Some(&mut bot), 0, "the selection", |v| {
        selected(v) == plan.quest
    });
    disconnect(bot);

    let mut again = rig.connect(0, "quest2");
    // Selecting the same quest again must be an unchanged no-op: that is only
    // true if the relog restored both the active quest and its selection.
    raw(&mut again, &select(plan.quest as i32));
    rig.wait_log(&mut again, "the restored selection", |value| {
        value["event"] == "quest_journal"
            && value["outcome"] == "unchanged"
            && value["request"]
                .as_str()
                .is_some_and(|request| request.contains(&format!("quest_id: {}", plan.quest)))
    });
    let restored = rig.save(0);
    assert_eq!(
        selected(&restored),
        plan.quest,
        "relog lost the selected quest"
    );
    assert_eq!(status(&restored), "active");
    disconnect(again);
}

/// W175/T003 review: the shipped Cooking Table breadcrumb advances an ordered
/// visit goal, registers the newly opened 45/11 goal on that movement, accepts
/// one 45/7 completion, refuses inactive/forged/duplicate claims, and persists
/// the completed reward through the real gateway and bot sockets.
#[test]
fn client_objective_breadcrumb_is_reachable_once_and_persists() {
    let _serial = serial();
    const ORIGINAL: u32 = 1811;
    const QUEST: u32 = 3_050_004;
    const CLIENT_ID: u32 = 30_500_041;
    const TRAINER: u64 = 9_000_000_019;
    const TABLE: [f32; 3] = [-509.0114, -5.469004, 163.4705];

    let rig = Rig::start_with_default_job(&[AccountRole::Builder], false, false, Some(45));
    let guid = rig.accounts[0].guid;
    let claim = |id: u32| [&[45u8, 0, 7][..], &(id as i32).to_le_bytes()].concat();
    let quest = |save: &Value| save["gameplay"]["npc_quests"]["quests"][QUEST.to_string()].clone();
    let mut bot = rig.connect(0, "objective");

    // Leave the starting enemies before taking any refusal snapshot: the checks
    // below compare the complete gameplay save, including combat state.
    ask(
        &mut bot,
        &format!("realm goto {} {} {}", TABLE[0], TABLE[1], TABLE[2]),
        "Developer teleport",
    );
    pose(&mut bot, guid, TABLE);

    // Inactive: a valid-looking id before the quest exists reaches the handler,
    // sends no completion, and leaves the complete gameplay save untouched. The
    // barrier request is handled after the refusal, so its reply proves that any
    // save the refusal scheduled (a bug) has been written before this read.
    let inactive = rig.log_outcomes("client_objective", "ignored");
    let before_inactive = rig.save(0)["gameplay"].clone();
    let from = bot.received.len();
    raw(&mut bot, &claim(CLIENT_ID));
    rig.wait_log_outcomes(
        "the inactive client objective",
        "client_objective",
        "ignored",
        inactive + 1,
    );
    barrier(&mut bot);
    assert_eq!(
        rig.save(0)["gameplay"],
        before_inactive,
        "the inactive claim changed persisted gameplay"
    );
    assert!(
        !bot.received[from..].iter().any(is_objective_complete),
        "the inactive claim sent an objective completion"
    );

    // 49/15 names the original Cooking Table id; the regional Cooking 101 offer is
    // accepted through the normal 49/2 path.
    raw(
        &mut bot,
        &[&[49u8, 0, 15, 0, 0, 0][..], &ORIGINAL.to_le_bytes()].concat(),
    );
    bot.wait_for(T, |message| {
        message.starts_with(&[49, 0, 1, 0, 0, 0])
            && message.get(6..10) == Some(&QUEST.to_le_bytes()[..])
    })
    .unwrap_or_else(|error| panic!("no Cooking 101 offer ({error}); got {}", tail(&bot)));
    raw(
        &mut bot,
        &[&[49u8, 0, 2, 0, 0, 0][..], &QUEST.to_le_bytes(), &[1]].concat(),
    );
    rig.save_when(
        Some(&mut bot),
        0,
        "the accepted Cooking 101 quest",
        |save| quest(save)["status"] == "active",
    );

    // The character is already at the table; this accepted pose runs the location
    // check. The visit goal advances first, and its newly opened 45/11
    // registration must arrive in that same movement response.
    pose(&mut bot, guid, TABLE);
    let mut watch = vec![45, 0, 11];
    watch.extend(CLIENT_ID.to_le_bytes());
    watch.push(0);
    bot.wait_for(T, |message| message == watch)
        .unwrap_or_else(|error| {
            panic!(
                "movement did not register the opened client goal ({error}); got {}",
                tail(&bot)
            )
        });
    rig.save_when(Some(&mut bot), 0, "the visit step", |save| {
        quest(save)["counts"] == json!([1, 0])
    });

    // A forged id is handled and inert.
    let ignored = rig.log_outcomes("client_objective", "ignored");
    let before_forged = rig.save(0)["gameplay"].clone();
    let from = bot.received.len();
    raw(&mut bot, &claim(CLIENT_ID + 1));
    rig.wait_log_outcomes(
        "the forged client objective",
        "client_objective",
        "ignored",
        ignored + 1,
    );
    barrier(&mut bot);
    assert_eq!(
        rig.save(0)["gameplay"],
        before_forged,
        "the forged claim changed persisted gameplay"
    );
    assert!(
        !bot.received[from..].iter().any(is_objective_complete),
        "the forged claim sent an objective completion"
    );

    // The real id completes exactly once.
    raw(&mut bot, &claim(CLIENT_ID));
    bot.wait_for(T, |message| {
        message.starts_with(&[49, 0, 10, 0, 0, 0])
            && message.get(6..10) == Some(&QUEST.to_le_bytes()[..])
    })
    .unwrap_or_else(|error| panic!("no objective complete ({error}); got {}", tail(&bot)));
    let completed = rig.save_when(Some(&mut bot), 0, "the client objective", |save| {
        quest(save)["counts"] == json!([1, 1])
    });

    // A duplicate is handled and inert.
    let ignored = rig.log_outcomes("client_objective", "ignored");
    let from = bot.received.len();
    raw(&mut bot, &claim(CLIENT_ID));
    rig.wait_log_outcomes(
        "the duplicate client objective",
        "client_objective",
        "ignored",
        ignored + 1,
    );
    barrier(&mut bot);
    assert_eq!(
        rig.save(0)["gameplay"],
        completed["gameplay"],
        "the duplicate claim changed persisted gameplay"
    );
    assert!(
        !bot.received[from..].iter().any(is_objective_complete),
        "the duplicate claim sent a second objective completion"
    );

    // Turn in at the spawned Chef trainer. The five-coin row and 25 Chef (profile 45)
    // job XP are paid once after the server sees the completed objective.
    raw(&mut bot, &npc_packet(8, TRAINER, &[0]));
    raw(&mut bot, &npc_packet(10, TRAINER, &1i32.to_le_bytes()));
    raw(
        &mut bot,
        &[&[26u8, 0, 6, 0][..], &301i32.to_le_bytes()].concat(),
    );
    bot.wait_for(T, |message| {
        message.starts_with(&[49, 0, 13, 0, 0, 0])
            && message.get(14..18) == Some(&QUEST.to_le_bytes()[..])
    })
    .unwrap_or_else(|error| panic!("no quest end screen ({error}); got {}", tail(&bot)));
    let coins = completed["gameplay"]["inventory"]["coins"]
        .as_u64()
        .unwrap();
    raw(
        &mut bot,
        &[&[49u8, 0, 14, 0, 0, 0][..], &QUEST.to_le_bytes()].concat(),
    );
    bot.wait_for(T, |message| message.starts_with(&[49, 0, 4, 0, 0, 0]))
        .unwrap_or_else(|error| panic!("no quest completion ({error}); got {}", tail(&bot)));
    let paid = rig.save_when(Some(&mut bot), 0, "the Cooking 101 reward", |save| {
        quest(save)["status"] == "completed"
            && save["gameplay"]["inventory"]["coins"] == json!(coins + 5)
    });

    // The logout save is the state a new gateway login restores. Neither the
    // completion nor the single payout may be replayed after reconnecting.
    logout(&rig, bot, "the Cooking 101 logout");
    let mut again = rig.connect(0, "objective2");
    let restored = rig.save(0);
    assert_eq!(quest(&restored), quest(&paid));
    let ignored = rig.log_outcomes("client_objective", "ignored");
    let from = again.received.len();
    raw(&mut again, &claim(CLIENT_ID));
    rig.wait_log_outcomes(
        "the replayed client objective after relog",
        "client_objective",
        "ignored",
        ignored + 1,
    );
    barrier(&mut again);
    assert_eq!(
        rig.save(0)["gameplay"],
        paid["gameplay"],
        "the replayed claim changed persisted gameplay"
    );
    assert!(
        !again.received[from..].iter().any(is_objective_complete),
        "the replayed claim sent a second objective completion"
    );
    disconnect(again);
}

// ------------------------------------------------------------ scenario 8 (T027)

fn read_string(message: &[u8], at: &mut usize) -> Option<String> {
    let length = i32::from_le_bytes(message.get(*at..*at + 4)?.try_into().ok()?);
    if length < 0 {
        return None;
    }
    let length = length as usize;
    let text = String::from_utf8(message.get(*at + 4..*at + 4 + length)?.to_vec()).ok()?;
    *at += 4 + length;
    Some(text)
}

struct EquipChange {
    item: i32,
    model: String,
    slot: i32,
    profile: i32,
}

/// 35/6 EquipItemChange for `guid`: item, model, slot and profile.
fn equip_change(message: &[u8], guid: u64) -> Option<EquipChange> {
    if message.get(..4)? != [35, 0, 6, 0]
        || u64::from_le_bytes(message.get(4..12)?.try_into().ok()?) != guid
    {
        return None;
    }
    let item = i32::from_le_bytes(message.get(12..16)?.try_into().ok()?);
    let mut at = 16;
    let model = read_string(message, &mut at)?;
    read_string(message, &mut at)?;
    read_string(message, &mut at)?;
    if message.len() != at + 24 {
        return None;
    }
    let slot = i32::from_le_bytes(message.get(at + 8..at + 12)?.try_into().ok()?);
    let profile = i32::from_le_bytes(message.get(at + 12..at + 16)?.try_into().ok()?);
    Some(EquipChange {
        item,
        model,
        slot,
        profile,
    })
}

/// 35/7 EquippedItemsChange for `guid`: count, model names and trailing value.
fn equipped_change(message: &[u8], guid: u64) -> Option<(i32, Vec<String>, i32)> {
    if message.get(..4)? != [35, 0, 7, 0]
        || u64::from_le_bytes(message.get(4..12)?.try_into().ok()?) != guid
    {
        return None;
    }
    let count = i32::from_le_bytes(message.get(12..16)?.try_into().ok()?);
    if !(0..=64).contains(&count) {
        return None;
    }
    let mut at = 16;
    let mut models = Vec::new();
    for _ in 0..count {
        models.push(read_string(message, &mut at)?);
        read_string(message, &mut at)?;
        read_string(message, &mut at)?;
        at += 12; // tint, effect, slot
    }
    if message.len() != at + 4 {
        return None;
    }
    let trailing = i32::from_le_bytes(message.get(at..at + 4)?.try_into().ok()?);
    Some((count, models, trailing))
}

/// T027: a neighbour sees an equip, an unequip and a job switch as the native
/// player-update packets. A replay and a refused switch tell nobody and leave
/// the saved state alone.
#[test]
fn neighbours_see_equipment_and_job_changes_live() {
    let _serial = serial();
    const CHEST_SLOT: i32 = 3;
    const UNLOCKED_JOB: i32 = 43; // Brawler, in the default unlocked set
    const LOCKED_JOB: i32 = 45; // not in the default unlocked set
    let rig = Rig::start(&[AccountRole::Player, AccountRole::Player], false);
    let mut a = rig.connect(0, "lookA");
    let mut b = rig.connect(1, "lookB");
    let guid = rig.accounts[0].guid;
    a.pump(Duration::from_millis(600)).unwrap();
    b.pump(Duration::from_millis(600)).unwrap();
    // B knows A, so its client can resolve the actor a 35/6 names.
    assert!(
        b.wait_for(T, |m| m.starts_with(&[35, 0, 1, 0])
            && m.get(4..12) == Some(&guid.to_le_bytes()[..]))
            .is_ok(),
        "B did not receive A's AddPc: {}",
        tail(&b)
    );
    let chest_item = worn(&rig.save(0), 1, CHEST_SLOT as i64) as i32;
    assert!(chest_item > 0, "the fixture has no starting chest");

    // Unequip the starting chest: an empty model is the client's removal path.
    let mut unequip = vec![42, 0, 2, 0];
    unequip.extend(CHEST_SLOT.to_le_bytes());
    unequip.extend(1i32.to_le_bytes());
    raw(&mut a, &unequip);
    let message = b
        .wait_for(T, |m| {
            equip_change(m, guid).is_some_and(|change| change.slot == CHEST_SLOT)
        })
        .expect("the neighbour did not see the unequip");
    let change = equip_change(&message, guid).unwrap();
    assert_eq!(change.model, "", "the removal carries an empty model");
    assert_eq!(change.item, 0);
    assert_eq!(change.profile, 1);

    // Equip it again: the same slot with the real attachment.
    let mut equip = vec![42, 0, 3, 0];
    equip.extend(chest_item.to_le_bytes());
    equip.extend(1i32.to_le_bytes());
    equip.extend(CHEST_SLOT.to_le_bytes());
    raw(&mut a, &equip);
    let message = b
        .wait_for(T, |m| {
            equip_change(m, guid)
                .is_some_and(|change| change.slot == CHEST_SLOT && change.item == chest_item)
        })
        .expect("the neighbour did not see the equip");
    assert!(!equip_change(&message, guid).unwrap().model.is_empty());

    // A replay changes nothing and tells nobody.
    raw(&mut a, &equip);
    assert!(
        b.wait_for(Duration::from_secs(2), |m| equip_change(m, guid).is_some())
            .is_err(),
        "a no-op equip was broadcast"
    );

    // A job switch sends the whole new outfit (35/7).
    let mut switch = vec![26, 0, 13, 0];
    switch.extend(UNLOCKED_JOB.to_le_bytes());
    raw(&mut a, &switch);
    let message = b
        .wait_for(T, |m| equipped_change(m, guid).is_some())
        .expect("the neighbour did not see the job switch");
    let (count, models, trailing) = equipped_change(&message, guid).unwrap();
    assert!(count >= 5, "the new outfit is not just a hat: {count}");
    assert_eq!(trailing, 0);
    assert!(
        models.iter().any(|model| model.starts_with("brawler_")),
        "{models:?}"
    );
    let saved = rig.save_when(Some(&mut a), 0, "the job switch", |save| {
        save["gameplay"]["profiles"]["active"] == json!(UNLOCKED_JOB)
    });
    assert_eq!(saved["gameplay"]["profiles"]["active"], json!(UNLOCKED_JOB));

    // A locked job is refused and tells nobody; the save still names the job above.
    let mut refused = vec![26, 0, 13, 0];
    refused.extend(LOCKED_JOB.to_le_bytes());
    raw(&mut a, &refused);
    assert!(
        b.wait_for(Duration::from_secs(2), |m| equipped_change(m, guid)
            .is_some())
            .is_err(),
        "a refused switch was broadcast"
    );
    assert_eq!(
        rig.save(0)["gameplay"]["profiles"]["active"],
        json!(UNLOCKED_JOB)
    );

    // Re-selecting the same job still repairs a completely empty outfit, so
    // neighbours must receive the restored attachment group.
    let slots: Vec<i32> = rig.save(0)["gameplay"]["inventory"]["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            entry["profile_id"] == json!(UNLOCKED_JOB)
                && entry["item_id"].as_i64().is_some_and(|item| item != 0)
        })
        .map(|entry| entry["slot"].as_i64().unwrap() as i32)
        .collect();
    assert!(!slots.is_empty(), "the selected job has no gear to remove");
    for slot in slots {
        let mut unequip = vec![42, 0, 2, 0];
        unequip.extend(slot.to_le_bytes());
        unequip.extend(UNLOCKED_JOB.to_le_bytes());
        raw(&mut a, &unequip);
        b.wait_for(T, |m| {
            equip_change(m, guid).is_some_and(|change| {
                change.slot == slot && change.profile == UNLOCKED_JOB && change.item == 0
            })
        })
        .unwrap_or_else(|e| panic!("the neighbour missed slot {slot} unequip ({e})"));
    }
    rig.save_when(Some(&mut a), 0, "the empty outfit", |save| {
        save["gameplay"]["inventory"]["equipment"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["profile_id"] == json!(UNLOCKED_JOB))
            .all(|entry| entry["item_id"] == json!(0))
    });
    raw(&mut a, &switch);
    let message = b
        .wait_for(T, |m| equipped_change(m, guid).is_some())
        .expect("the neighbour did not see the same-job outfit repair");
    let (count, models, _) = equipped_change(&message, guid).unwrap();
    assert!(count >= 5, "the repaired outfit is incomplete: {count}");
    assert!(
        models.iter().any(|model| model.starts_with("brawler_")),
        "{models:?}"
    );
    rig.save_when(Some(&mut a), 0, "the repaired outfit", |save| {
        save["gameplay"]["inventory"]["equipment"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| {
                entry["profile_id"] == json!(UNLOCKED_JOB)
                    && entry["item_id"].as_i64().is_some_and(|item| item != 0)
            })
    });

    // A viewer who arrives after the switch gets the new outfit in AddPc.
    disconnect(b);
    std::thread::sleep(Duration::from_millis(1_500));
    let mut late = rig.connect(1, "lookLate");
    late.pump(Duration::from_millis(600)).unwrap();
    let add_pc = late
        .wait_for(T, |m| {
            m.starts_with(&[35, 0, 1, 0]) && m.get(4..12) == Some(&guid.to_le_bytes()[..])
        })
        .expect("the late viewer did not receive A's AddPc");
    assert!(
        add_pc
            .windows(b"brawler_".len())
            .any(|part| part == b"brawler_"),
        "the late viewer got the stale outfit"
    );
    disconnect(late);
    disconnect(a);
}

// ------------------------------------------------------- server finding

/// Finding W154-F1 (server, for the lead): with the full content, a handshake
/// makes the gateway build a whole gameplay before it answers (every NPC file is
/// parsed three times), so under load the SessionResponse arrives after the
/// bot's 5 s window and the session then dies at `login_deadline`. Un-ignore once
/// `Session::new` stops re-loading content; see `lab/runs/claude/W154/final2.md`.
#[test]
#[ignore = "server finding W154-F1: a handshake costs seconds of content loading in a debug build"]
fn a_handshake_is_answered_quickly_even_with_full_content() {
    let _serial = serial();
    let rig = Rig::start_with(&[AccountRole::Player], false, false);
    let mut slowest = Duration::ZERO;
    for round in 0..3u32 {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(20)))
            .unwrap();
        let mut request = vec![0, 1];
        request.extend(3u32.to_be_bytes());
        request.extend((900_000 + round).to_be_bytes());
        request.extend(512u32.to_be_bytes());
        request.extend(b"CGAPI_527\0");
        let sent = Instant::now();
        socket.send_to(&request, rig.gateway).unwrap();
        let mut buffer = [0u8; 64];
        while sent.elapsed() < Duration::from_secs(60) {
            if let Ok((n, _)) = socket.recv_from(&mut buffer) {
                if n >= 17 && buffer[..2] == [0, 2] {
                    break;
                }
            }
        }
        slowest = slowest.max(sent.elapsed());
        std::thread::sleep(Duration::from_millis(300));
    }
    assert!(
        slowest < Duration::from_millis(1500),
        "the slowest SessionResponse took {slowest:?}; the bot gives up after 5 s"
    );
}
