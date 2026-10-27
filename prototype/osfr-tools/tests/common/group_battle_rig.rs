//! The two-socket (and four-socket) group battle rig shared by the Shared World suites:
//! T440's invite tests and T441's shared arena. A real `Server::step` loop runs on its own
//! thread; the bots are real UDP clients that log in, form real family-40 parties, press Go on the
//! battle marker and hit enemies. `host` runs a closure on the server thread between steps.
//!
//! Include with `#[path = "common/group_battle_rig.rs"] mod group_battle_rig;` next to the modules
//! `bot`, `field_battle_fixture`, `gateway_runner` and `group_invite_wire` it uses.
#![allow(dead_code)] // each suite uses a different subset

use crate::gateway_runner::{Options, Server};
use crate::{bot, field_battle_fixture, group_invite_wire};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    encounter_invite::{self as invite, Reply},
    identity::{NativeCreation, Resources},
    login,
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";
// Upper bound for "the server gets to it", never a pace. A loaded integration
// host can take twice as long as an idle one, so leave headroom.
pub const PATIENCE: Duration = Duration::from_secs(60);
pub const BRAWLER: i32 = 43;
pub const BATTLE: u32 = 96;
/// The players a rig can seat, in seat order.
pub const PLAYERS: [u64; 4] = [101, 202, 303, 404];
pub const A: u64 = PLAYERS[0];
pub const B: u64 = PLAYERS[1];
pub const C: u64 = PLAYERS[2];
pub const D: u64 = PLAYERS[3];

pub type HostAction = Box<dyn FnOnce(&mut Server) + Send>;
/// Everything the server holds for one player: pose, instance, readiness and members, the three
/// membership views, the party, and the exact save bytes.
pub type Place = (
    Option<(CharacterState, i32, bool, BTreeSet<u64>)>,
    Option<(bool, bool, bool)>,
    Option<u32>,
    Vec<u8>,
);

pub struct Rig {
    tx: Option<mpsc::Sender<HostAction>>,
    handle: Option<thread::JoinHandle<()>>,
    pub seed: field_battle_fixture::Fixture,
    pub root: PathBuf,
    accounts: PathBuf,
    addr: std::net::SocketAddr,
    /// One bot per seat of `PLAYERS`, seated as each logs in.
    pub bots: Vec<bot::Bot>,
    /// The job a fresh character holds (not a combat job): where B switches to go ineligible.
    idle_job: i32,
    /// Everything each bot received since the last `take`.
    pub inbox: Vec<Vec<Vec<u8>>>,
}

impl Rig {
    /// Both players at the battle marker, in a party when `grouped`; B holds a combat job unless
    /// `b_job` says otherwise.
    pub fn new(grouped: bool, b_job: Option<i32>) -> Self {
        let jobs = |seat| if seat == 1 { b_job } else { Some(BRAWLER) };
        Self::build(2, grouped, jobs, 0)
    }

    /// `count` players (2 or 4) at the battle marker in parties of two (A and B, C and D), all
    /// on `job` at `level` through the seed session's `realm level`.
    pub fn fighters(count: usize, job: i32, level: u32) -> Self {
        Self::build(count, true, |_| Some(job), level)
    }

    fn build(count: usize, grouped: bool, jobs: impl Fn(usize) -> Option<i32>, level: u32) -> Self {
        let seed = field_battle_fixture::Fixture::new();
        let root = seed.root.join("server");
        fs::create_dir_all(root.join("characters")).unwrap();
        let battles = field_battle_fixture::battles();
        let marker = field_battle_fixture::position(
            &field_battle_fixture::battle(&battles, BATTLE)["spawner"]["position"],
        );
        let mut idle_job = 0;
        for (seat, guid) in PLAYERS[..count].iter().copied().enumerate() {
            let (mut gateway, _) = seed.session(marker, guid);
            idle_job = field_battle_fixture::active_profile(&gateway);
            if let Some(job) = jobs(seat) {
                field_battle_fixture::send(
                    &mut gateway,
                    &field_battle_fixture::switch_job(job),
                    300,
                );
            }
            if level > 1 {
                let command = osfr_tools::chat::local_request(&format!("realm level {level}"));
                field_battle_fixture::send(&mut gateway, &command, 350);
            }
            osfr_tools::game_store::save(
                &root.join(format!("characters/{guid}.json")),
                &gateway.snapshot(),
            )
            .unwrap();
        }
        let accounts = create_account_store(&root, count);
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference: seed.reference.clone(),
            state: root.join("state.json"),
            content: Some(seed.content.clone()),
            identity: None,
            account_store: Some(accounts.clone()),
            gateway_ticket: String::new(),
            handoff_key: HANDOFF_KEY.to_vec(),
            seconds: 0,
            dev_tools: false,
            studio_port: 0,
            client_logs: None,
            terrain: None,
            world: None,
            resources: None,
            stock_text: None,
            allow_lan: false,
            security_log: None,
            group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
            guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
        };
        let (tx, rx) = mpsc::channel::<HostAction>();
        let (addr_tx, addr_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut server = Server::new(options).unwrap();
            addr_tx.send(server.local_addr().unwrap()).unwrap();
            loop {
                match rx.try_recv() {
                    Ok(action) => action(&mut server),
                    Err(mpsc::TryRecvError::Empty) => {}
                    Err(mpsc::TryRecvError::Disconnected) => break,
                }
                server.step().unwrap();
            }
            server.shutdown().unwrap();
        });
        let addr = addr_rx.recv_timeout(PATIENCE).unwrap();
        let mut rig = Rig {
            tx: Some(tx),
            handle: Some(handle),
            seed,
            root,
            accounts,
            addr,
            bots: Vec::new(),
            idle_job,
            inbox: vec![Vec::new(); count],
        };
        rig.log_in(count);
        if grouped {
            for pair in PLAYERS[..count].chunks(2) {
                rig.form_party_of(pair[0], pair[1]);
            }
        }
        rig
    }

    /// The seat of `guid` in `PLAYERS`.
    pub fn seat(guid: u64) -> usize {
        PLAYERS.iter().position(|p| *p == guid).unwrap()
    }

    pub fn bot(&mut self, guid: u64) -> &mut bot::Bot {
        &mut self.bots[Self::seat(guid)]
    }

    /// A bot for `guid` on a new connection, past the login handshake: the server ends a session
    /// that has not logged in within its deadline, so a seat connects only when its turn comes.
    fn connect_seat(&self, guid: u64, connection_id: u32) -> bot::Bot {
        let store = AccountStore::load(&self.accounts).unwrap();
        let account = store.account(guid).unwrap();
        let ticket = login::gateway_binding_ticket(HANDOFF_KEY, account, guid);
        let version = self.seed.version.clone();
        let mut bot = bot::Bot::connect(bot::BotConfig {
            gateway: self.addr,
            name: guid.to_string(),
            connection_id,
            guid,
            ticket: ticket.clone(),
            version: version.clone(),
            expect_timeout_ms: PATIENCE.as_millis() as u64,
            quiet: true,
        })
        .unwrap();
        bot.send_message(&bot::login_message(guid, &ticket, &version), "1")
            .unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
        bot.send_message(&packet(13), "13").unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
        bot.send_message(&packet(10), "10").unwrap();
        bot
    }

    /// Every seat logs in, one after the other, and loads into the open world.
    fn log_in(&mut self, count: usize) {
        for (seat, guid) in PLAYERS[..count].iter().copied().enumerate() {
            let bot = self.connect_seat(guid, 10901 + seat as u32);
            self.bots.push(bot);
            self.ready(guid);
            self.drain(guid);
        }
        self.pump(Duration::from_millis(500));
        for seat in 0..count {
            self.take(seat);
        }
    }

    /// A real family-40 party of A and B, not an injected social state.
    pub fn form_party(&mut self) {
        self.form_party_of(A, B);
    }

    /// A real family-40 party: `from` invites `to`, who accepts. A starved host can park the
    /// first prompt, and the server refuses a repeat while its 30 s invite row lives (and drops
    /// a prompt its target's queue cannot hold), so a missing prompt is clicked again the way a
    /// player would; the attempts straddle the row's deadline.
    pub fn form_party_of(&mut self, from: u64, to: u64) {
        let mut invite = None;
        for _ in 0..4 {
            self.bot(from)
                .send_message(&bot::wrap(&group_request(from, to, false, 0)), "40/1")
                .unwrap();
            match self
                .bot(to)
                .wait_for(Duration::from_secs(12), |m| is_sub(m, 40, 1))
            {
                Ok(found) => {
                    invite = Some(found);
                    break;
                }
                Err(_) => continue,
            }
        }
        let invite = invite.expect("a family-40 invite reached the member");
        let id = group_invite_wire::invite_generation(&invite);
        self.bot(to)
            .send_message(&bot::wrap(&group_request(from, to, true, id)), "40/4")
            .unwrap();
        self.bot(from)
            .wait_for(PATIENCE, |m| is_sub(m, 40, 2))
            .unwrap();
        assert!(self.group(from).is_some() && self.group(from) == self.group(to));
    }

    pub fn host<T: Send + 'static>(&self, f: impl FnOnce(&mut Server) -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        self.tx
            .as_ref()
            .unwrap()
            .send(Box::new(move |server| tx.send(f(server)).unwrap()))
            .unwrap();
        rx.recv_timeout(PATIENCE).unwrap()
    }

    pub fn has_invite(&self, guid: u64) -> bool {
        self.host(move |s| s.test_has_invite(guid))
    }

    pub fn group(&self, guid: u64) -> Option<u32> {
        self.host(move |s| s.test_group(guid))
    }

    pub fn place(&self, guid: u64) -> Place {
        let (location, views) =
            self.host(move |s| (s.test_location(guid), s.test_membership(guid)));
        (
            location,
            views,
            self.group(guid),
            fs::read(self.root.join(format!("characters/{guid}.json"))).unwrap(),
        )
    }

    pub fn pump(&mut self, interval: Duration) {
        let deadline = Instant::now() + interval;
        while Instant::now() < deadline {
            for (index, bot) in self.bots.iter_mut().enumerate() {
                bot.pump(Duration::from_millis(2)).unwrap();
                while let Ok(message) = bot.wait_for(Duration::ZERO, |_| true) {
                    self.inbox[index].push(message);
                }
            }
        }
    }

    pub fn take(&mut self, who: usize) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.inbox[who])
    }

    pub fn until(&mut self, what: &str, done: impl Fn(&Rig) -> bool) {
        let deadline = Instant::now() + PATIENCE;
        while !done(self) {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            self.pump(Duration::from_millis(10));
        }
    }

    /// The readiness barrier the loader sends (13 then 10), until the server marks the scene
    /// ready and loaded. Loading answers with that scene's startup stream; the load helpers
    /// drain it with `drain`, so a late retransmit cannot land in the next assertion's window.
    pub fn ready(&mut self, guid: u64) {
        let deadline = Instant::now() + PATIENCE;
        let mut sent = false;
        loop {
            self.pump(Duration::from_millis(5));
            if self.host(move |s| s.test_location(guid).is_some_and(|row| row.2)) {
                break;
            }
            if !sent {
                let bot = self.bot(guid);
                bot.send_message(&packet(13), "13").unwrap();
                bot.send_message(&packet(10), "10").unwrap();
                sent = true;
            }
            assert!(Instant::now() < deadline, "readiness barrier for {guid}");
        }
        // 10 is Loaded: the server answers it with the new scene's startup stream.
        let loaded = Instant::now() + PATIENCE;
        while !self.host(move |s| {
            s.test_gateway(guid)
                .is_some_and(|gateway| gateway.is_world_loaded())
        }) {
            assert!(Instant::now() < loaded, "the scene load for {guid}");
            self.pump(Duration::from_millis(10));
        }
    }

    /// Pump until `guid`'s inbox has been quiet for 400 ms, then empty it: the scene-load stream
    /// belongs to that player alone. Another seat's inbox (an invite, a partner's results) stays.
    /// Each pass first lets the server finish a step, so a starved server cannot look quiet.
    fn drain(&mut self, guid: u64) {
        let seat = Self::seat(guid);
        let deadline = Instant::now() + PATIENCE;
        let mut quiet_since = Instant::now();
        while quiet_since.elapsed() < Duration::from_millis(400) {
            assert!(Instant::now() < deadline, "the server never went quiet");
            let _ = self.host(|_| ());
            self.pump(Duration::from_millis(50));
            if self.inbox[seat].iter().any(|m| !ambient(m)) {
                quiet_since = Instant::now();
            }
            self.inbox[seat].clear();
        }
    }

    /// `guid` clicks the battle marker, presses Go, and loads into the arena. Returns its run.
    pub fn starts_battle(&mut self, guid: u64) -> i32 {
        let seat = Self::seat(guid);
        let battles = field_battle_fixture::battles();
        let marker = field_battle_fixture::u64_field(
            &field_battle_fixture::battle(&battles, BATTLE)["spawner"],
            "guid",
        );
        let click = [vec![26, 0, 8, 0], marker.to_le_bytes().to_vec(), vec![0]].concat();
        self.take(seat);
        self.bot(guid)
            .send_message(&bot::wrap(&click), "26/8")
            .unwrap();
        self.until("the start screen", |rig| {
            rig.inbox[seat].iter().any(|m| is_sub(m, 41, 114))
        });
        let details = self
            .take(seat)
            .into_iter()
            .find(|m| is_sub(m, 41, 114))
            .unwrap();
        self.bot(guid)
            .send_message(&bot::wrap(&field_battle_fixture::go(&details)), "41/108")
            .unwrap();
        self.until("the player in the arena", |rig| {
            rig.host(move |s| {
                s.test_location(guid)
                    .is_some_and(|row| row.0.zone_id == 9600)
            })
        });
        self.ready(guid);
        // The arena load ends its actor burst with the 35/12 relevance list (after every spawn
        // and health), so a delayed stream cannot leak into the hit the caller goes on to make.
        self.until("the arena load's relevance", |rig| {
            rig.inbox[seat].iter().any(|m| is_sub(m, 35, 12))
        });
        self.drain(guid);
        self.host(move |s| s.test_location(guid).unwrap().1)
    }

    /// `guid` hits `target` with the basic attack: 36/10, bar 1, slot 0, an entity target.
    pub fn hit(&mut self, guid: u64, target: u64) {
        let request = bot::ability_request(0, target);
        self.bot(guid)
            .send_message(&bot::wrap(&request), "36/10")
            .unwrap();
    }

    /// Walk `guid` to `to` in steps the movement rules allow (8 m per 450 ms).
    pub fn walk(&mut self, guid: u64, to: [f32; 3]) {
        let from = self.host(move |s| s.test_gateway(guid).unwrap().state().position);
        let flat = (to[0] - from[0]).hypot(to[2] - from[2]);
        let steps = (flat / 8.0).ceil().max(1.0) as usize;
        for step in 1..=steps {
            let amount = step as f32 / steps as f32;
            let at = [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * amount);
            let request = bot::move_request(guid, at, [0.0, 0.0, 1.0]);
            self.bot(guid)
                .send_message(&bot::wrap(&request), "125")
                .unwrap();
            self.pump(Duration::from_millis(450));
        }
    }

    /// Walk `guid` up to `enemy` (3 m melee reach) and hit it once. True when the enemy's health
    /// fell: an enemy that moved on, or a shielded one, answers false and the caller tries again.
    pub fn strike(&mut self, guid: u64, enemy: u64) -> bool {
        let Some((at, before)) = self.host(move |s| s.test_enemy(guid, enemy)) else {
            return false;
        };
        self.walk(guid, [at[0], at[1], at[2] - 1.5]);
        self.hit(guid, enemy);
        // The server takes the hit on its next step; a loaded machine takes longer. Once it did,
        // its packets still have to reach the bots.
        for _ in 0..15 {
            self.pump(Duration::from_millis(100));
            if self
                .host(move |s| s.test_enemy(guid, enemy))
                .is_some_and(|(_, after)| after < before)
            {
                self.pump(Duration::from_millis(400));
                return true;
            }
        }
        false
    }

    /// `strike` until it lands (a kiting summoner is out of reach for a moment).
    pub fn land(&mut self, guid: u64, enemy: u64) {
        for _ in 0..12 {
            if self.strike(guid, enemy) {
                return;
            }
        }
        panic!("{guid} could not hit {enemy}");
    }

    /// `guid` drops its connection (the server ends the session).
    pub fn disconnect(&mut self, guid: u64) {
        self.bot(guid).disconnect();
        self.until("the session to end", |rig| {
            rig.host(move |s| s.test_location(guid).is_none())
        });
    }

    /// `guid` connects again on a new connection and logs back in to the open world.
    pub fn reconnect(&mut self, guid: u64) {
        let seat = Self::seat(guid);
        self.bots[seat] = self.connect_seat(guid, 11901 + seat as u32);
        self.inbox[seat].clear();
        self.ready(guid);
        self.drain(guid);
    }

    /// `guid` leaves the run (41/109) and loads back into the open world.
    pub fn leaves_battle(&mut self, guid: u64, run: i32) {
        let mut exit = vec![41, 0, 109, 0];
        exit.extend(0_i32.to_le_bytes());
        exit.extend(run.to_le_bytes());
        self.bot(guid)
            .send_message(&bot::wrap(&exit), "41/109")
            .unwrap();
        self.until("the player back in the world", |rig| {
            rig.host(move |s| s.test_location(guid).is_some_and(|row| row.0.zone_id == 0))
        });
        self.ready(guid);
        self.drain(guid);
    }

    /// `who` sends the family-40 leave request and the server drops the party.
    pub fn leave_party(&mut self, who: u64) {
        let mut out = vec![40, 0, 3, 0];
        out.extend(who.to_le_bytes());
        name_record(&mut out, who);
        self.bot(who)
            .send_message(&bot::wrap(&out), "40/3")
            .unwrap();
        self.until("the party to end", |rig| rig.group(who).is_none());
    }

    /// B switches to a non-combat job (38/21 comes back), after the invite is out.
    pub fn b_goes_idle(&mut self) {
        self.take(1);
        let job = field_battle_fixture::switch_job(self.idle_job);
        self.bot(B).send_message(&bot::wrap(&job), "26/13").unwrap();
        self.until("B's job change", |rig| {
            rig.inbox[1].iter().any(|m| is_sub(m, 38, 21))
        });
    }

    /// `guid` sends a 41/103; every inbox is emptied first so only the answer's effects remain.
    pub fn answers(&mut self, guid: u64, reply: Reply) {
        self.sends(guid, &invite::encode_reply(&reply));
    }

    pub fn sends(&mut self, guid: u64, inner: &[u8]) {
        self.settle();
        self.bot(guid)
            .send_message(&bot::wrap(inner), "41/103")
            .unwrap();
    }

    /// Pump until nothing but ambient traffic has arrived for 400 ms (a scene load can stream for
    /// seconds), then empty every inbox. Each pass lets the server finish a step so a starved
    /// server cannot look quiet.
    pub fn settle(&mut self) {
        let deadline = Instant::now() + PATIENCE;
        let mut quiet_since = Instant::now();
        while quiet_since.elapsed() < Duration::from_millis(400) {
            assert!(Instant::now() < deadline, "the server never went quiet");
            let _ = self.host(|_| ());
            self.pump(Duration::from_millis(50));
            if self.inbox.iter().flatten().any(|m| !ambient(m)) {
                quiet_since = Instant::now();
            }
            for seat in 0..self.bots.len() {
                self.take(seat);
            }
        }
    }

    /// Nothing B owns changed and, after the server settled, neither bot was sent anything but
    /// ambient actor traffic (family 125 updates, 35/8 NPC idle animations): those arrive on the
    /// server's own timer whatever B answered. Any other message, such as a refusal chat, an
    /// error reply or a transfer, fails.
    pub fn assert_inert(&mut self, what: &str, before: &Place) {
        self.pump(Duration::from_millis(600));
        assert_eq!(*before, self.place(B), "{what}: B's place changed");
        for (who, got) in [("A", self.take(0)), ("B", self.take(1))] {
            let sent: Vec<_> = got
                .iter()
                .filter(|m| !ambient(m))
                .map(|m| m[..m.len().min(8)].to_vec())
                .collect();
            assert!(sent.is_empty(), "{what}: {who} was sent {sent:?}");
        }
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

/// Actor traffic the server sends on its own timer: family 125 updates, 35/8 NPC animations.
fn ambient(message: &[u8]) -> bool {
    message.starts_with(&[125, 0]) || is_sub(message, 35, 8)
}
pub fn is_sub(message: &[u8], family: u16, sub: u16) -> bool {
    message.starts_with(&[
        family as u8,
        (family >> 8) as u8,
        sub as u8,
        (sub >> 8) as u8,
    ])
}
fn numeric_name(id: u64) -> String {
    id.to_string()
        .bytes()
        .map(|digit| char::from(b'a' + digit - b'0'))
        .collect()
}
fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}
fn name_record(out: &mut Vec<u8>, guid: u64) {
    out.extend([0; 12]);
    put_string(out, &numeric_name(guid));
    put_string(out, &numeric_name(guid));
}
/// 40/1 (invite) and 40/4 (accept) as the client sends them, `from` inviting `to`.
fn group_request(from: u64, to: u64, accept: bool, invite_id: u32) -> Vec<u8> {
    let mut out = vec![40, 0, if accept { 4 } else { 1 }, 0];
    out.extend(from.to_le_bytes());
    name_record(&mut out, from);
    out.extend(to.to_le_bytes());
    name_record(&mut out, to);
    out.extend(invite_id.to_le_bytes());
    if accept {
        out.push(1);
    }
    out.extend([0; 20]);
    out
}
pub fn packet(opcode: u16) -> Vec<u8> {
    let mut out = 5_u16.to_le_bytes().to_vec();
    out.push(1);
    out.extend(2_u32.to_le_bytes());
    out.extend(opcode.to_le_bytes());
    out
}
fn create_account_store(root: &Path, count: usize) -> PathBuf {
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
    let path = root.join("accounts.json");
    AccountStore::new().save(&path).unwrap();
    for id in PLAYERS[..count].iter().copied() {
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        document["store"]["next_account_id"] = json!(id);
        document["store"]["next_character_guid"] = json!(id);
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let mut store = AccountStore::load(&path).unwrap();
        let account = store
            .create_account(
                [id as u8; 32],
                AccountFlags {
                    open_chat: true,
                    ..Default::default()
                },
                SlotPolicy { max_characters: 1 },
            )
            .unwrap();
        let mut choices = [0; 21];
        for (index, value) in [
            (0, 1),
            (1, 1),
            (2, 2),
            (5, 1),
            (7, 201),
            (8, 189),
            (9, 199),
            (20, 1),
        ] {
            choices[index] = value;
        }
        let reply = create_character(
            &mut store,
            account,
            NativeCreation {
                first_name: numeric_name(id),
                last_name: numeric_name(id),
                temporary_first_name: String::new(),
                temporary_last_name: String::new(),
                choices,
            },
            &resources,
        );
        assert_eq!(
            reply.status,
            osfr_tools::account::CharacterCreateStatus::Created
        );
        store.save(&path).unwrap();
    }
    path
}
