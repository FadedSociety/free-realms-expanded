//! T440 (Shared World slice 5): the ordinary group encounter invitation over two real sockets.
//!
//! The party is formed with the family-40 codec. A starts battle 96 (marker click, then Go);
//! B decodes exactly one 41/102 with the recorded fields. Decline keeps both grouped and B's
//! location, membership and save bytes. Accept puts B into A's World instance through one
//! transfer, also when the reply carries a stale non-zero word B (a real client's `client+0xc10`
//! is its last 41/107 instance). A reply with no invite, an expired deadline (injected clock), a
//! replayed accept and an accept after the starter left the run change nothing and send nothing
//! (both inboxes stay empty), and a refused accept leaves the invite answerable. Accept begins
//! the same battle for B in A's run. Ungrouped and ineligible players get no invite.
//!
//! Client layouts (IDA): 41/102 reader `00a9f920`, 41/103 writer `00a97720` (see
//! `src/encounter_invite.rs`).
#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;
#[path = "common/group_invite_wire.rs"]
mod group_invite_wire;
use gateway_runner::{Options, Server};
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

const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";
// Upper bound for "the server gets to it", never a pace. A loaded integration
// host can take twice as long as an idle one, so leave headroom.
const PATIENCE: Duration = Duration::from_secs(60);
const BRAWLER: i32 = 43;
const BATTLE: u32 = 96;
const A: u64 = 101;
const B: u64 = 202;
/// A non-zero header word B: the last battle instance the invitee's client saw (`client+0xc10`).
const STALE_RUN: i32 = 7_777;

type HostAction = Box<dyn FnOnce(&mut Server) + Send>;
/// Everything the server holds for one player: pose, instance, readiness and members, the three
/// membership views, the party, and the exact save bytes.
type Place = (
    Option<(CharacterState, i32, bool, BTreeSet<u64>)>,
    Option<(bool, bool, bool)>,
    Option<u32>,
    Vec<u8>,
);

struct Rig {
    tx: Option<mpsc::Sender<HostAction>>,
    handle: Option<thread::JoinHandle<()>>,
    seed: field_battle_fixture::Fixture,
    root: PathBuf,
    accounts: PathBuf,
    a: bot::Bot,
    b: bot::Bot,
    /// The job a fresh character holds (not a combat job): where B switches to go ineligible.
    idle_job: i32,
    /// Everything each bot received since the last `take`.
    inbox: [Vec<Vec<u8>>; 2],
}

impl Rig {
    /// Both players at the battle marker, in a party when `grouped`; B holds a combat job unless
    /// `b_job` says otherwise.
    fn new(grouped: bool, b_job: Option<i32>) -> Self {
        let seed = field_battle_fixture::Fixture::new();
        let root = seed.root.join("server");
        fs::create_dir_all(root.join("characters")).unwrap();
        let battles = field_battle_fixture::battles();
        let marker = field_battle_fixture::position(
            &field_battle_fixture::battle(&battles, BATTLE)["spawner"]["position"],
        );
        let mut idle_job = 0;
        for (guid, job) in [(A, Some(BRAWLER)), (B, b_job)] {
            let (mut gateway, _) = seed.session(marker, guid);
            idle_job = field_battle_fixture::active_profile(&gateway);
            if let Some(job) = job {
                field_battle_fixture::send(
                    &mut gateway,
                    &field_battle_fixture::switch_job(job),
                    300,
                );
            }
            osfr_tools::game_store::save(
                &root.join(format!("characters/{guid}.json")),
                &gateway.snapshot(),
            )
            .unwrap();
        }
        let accounts = create_account_store(&root);
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
        let store = AccountStore::load(&accounts).unwrap();
        let connect = |guid: u64, connection_id: u32| {
            let account = store
                .account(store.character(guid).unwrap().owner_account_id)
                .unwrap();
            bot::Bot::connect(bot::BotConfig {
                gateway: addr,
                name: guid.to_string(),
                connection_id,
                guid,
                ticket: login::gateway_binding_ticket(HANDOFF_KEY, account, guid),
                version: seed.version.clone(),
                expect_timeout_ms: PATIENCE.as_millis() as u64,
                quiet: true,
            })
            .unwrap()
        };
        let (a, b) = (connect(A, 10901), connect(B, 10902));
        let mut rig = Rig {
            tx: Some(tx),
            handle: Some(handle),
            seed,
            root,
            accounts,
            a,
            b,
            idle_job,
            inbox: Default::default(),
        };
        rig.log_in();
        if grouped {
            rig.form_party();
        }
        rig
    }

    fn log_in(&mut self) {
        let store = AccountStore::load(&self.accounts).unwrap();
        let version = self.seed.version.clone();
        // Both handshakes go out before either reply is awaited: the gateway
        // drops a session that has not sent its login by its login deadline, so
        // a slow first reply must not spend the second bot's window.
        for (guid, bot) in [(A, &mut self.a), (B, &mut self.b)] {
            let account = store.account(guid).unwrap();
            let ticket = login::gateway_binding_ticket(HANDOFF_KEY, account, guid);
            bot.send_message(&bot::login_message(guid, &ticket, &version), "1")
                .unwrap();
        }
        for bot in [&mut self.a, &mut self.b] {
            bot.wait_for(PATIENCE, |_| true).unwrap();
        }
        for bot in [&mut self.a, &mut self.b] {
            bot.send_message(&packet(13), "13").unwrap();
        }
        for bot in [&mut self.a, &mut self.b] {
            bot.wait_for(PATIENCE, |_| true).unwrap();
        }
        self.a.send_message(&packet(10), "10").unwrap();
        self.ready(A);
        self.b.send_message(&packet(10), "10").unwrap();
        self.ready(B);
        self.pump(Duration::from_millis(500));
        self.take(0);
        self.take(1);
    }

    /// A real family-40 party, not an injected social state.
    fn form_party(&mut self) {
        self.a
            .send_message(&bot::wrap(&group_request(false, 0)), "40/1")
            .unwrap();
        let invite = self.b.wait_for(PATIENCE, |m| is_sub(m, 40, 1)).unwrap();
        let id = group_invite_wire::invite_generation(&invite);
        self.b
            .send_message(&bot::wrap(&group_request(true, id)), "40/4")
            .unwrap();
        self.a.wait_for(PATIENCE, |m| is_sub(m, 40, 2)).unwrap();
        assert!(self.group(A).is_some() && self.group(A) == self.group(B));
    }

    fn host<T: Send + 'static>(&self, f: impl FnOnce(&mut Server) -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        self.tx
            .as_ref()
            .unwrap()
            .send(Box::new(move |server| tx.send(f(server)).unwrap()))
            .unwrap();
        rx.recv_timeout(PATIENCE).unwrap()
    }

    fn group(&self, guid: u64) -> Option<u32> {
        self.host(move |s| s.test_group(guid))
    }

    fn place(&self, guid: u64) -> Place {
        let (location, views) =
            self.host(move |s| (s.test_location(guid), s.test_membership(guid)));
        (
            location,
            views,
            self.group(guid),
            fs::read(self.root.join(format!("characters/{guid}.json"))).unwrap(),
        )
    }

    fn pump(&mut self, interval: Duration) {
        let deadline = Instant::now() + interval;
        while Instant::now() < deadline {
            for (index, bot) in [&mut self.a, &mut self.b].into_iter().enumerate() {
                bot.pump(Duration::from_millis(2)).unwrap();
                while let Ok(message) = bot.wait_for(Duration::ZERO, |_| true) {
                    self.inbox[index].push(message);
                }
            }
        }
    }

    fn take(&mut self, who: usize) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.inbox[who])
    }

    fn until(&mut self, what: &str, done: impl Fn(&Rig) -> bool) {
        let deadline = Instant::now() + PATIENCE;
        while !done(self) {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            self.pump(Duration::from_millis(10));
        }
    }

    /// The readiness barrier the loader sends (13 then 10), until the server marks them ready.
    fn ready(&mut self, guid: u64) {
        let deadline = Instant::now() + PATIENCE;
        let mut sent = false;
        loop {
            self.pump(Duration::from_millis(5));
            if self.host(move |s| s.test_location(guid).is_some_and(|row| row.2)) {
                return;
            }
            if !sent {
                let bot = if guid == A { &mut self.a } else { &mut self.b };
                bot.send_message(&packet(13), "13").unwrap();
                bot.send_message(&packet(10), "10").unwrap();
                sent = true;
            }
            assert!(Instant::now() < deadline, "readiness barrier for {guid}");
        }
    }

    /// A clicks the battle marker, presses Go, and loads into the arena. Returns A's run.
    fn a_starts_battle(&mut self) -> i32 {
        let battles = field_battle_fixture::battles();
        let marker = field_battle_fixture::u64_field(
            &field_battle_fixture::battle(&battles, BATTLE)["spawner"],
            "guid",
        );
        let click = [vec![26, 0, 8, 0], marker.to_le_bytes().to_vec(), vec![0]].concat();
        self.take(0);
        self.a.send_message(&bot::wrap(&click), "26/8").unwrap();
        self.until("A's start screen", |rig| {
            rig.inbox[0].iter().any(|m| is_sub(m, 41, 114))
        });
        let details = self
            .take(0)
            .into_iter()
            .find(|m| is_sub(m, 41, 114))
            .unwrap();
        self.a
            .send_message(&bot::wrap(&field_battle_fixture::go(&details)), "41/108")
            .unwrap();
        self.until("A in the arena", |rig| {
            rig.host(|s| s.test_location(A).is_some_and(|row| row.0.zone_id == 9600))
        });
        self.ready(A);
        self.host(|s| s.test_location(A).unwrap().1)
    }

    /// A leaves the run (41/109) and loads back into the open world.
    fn a_leaves_battle(&mut self, run: i32) {
        let mut exit = vec![41, 0, 109, 0];
        exit.extend(0_i32.to_le_bytes());
        exit.extend(run.to_le_bytes());
        self.a.send_message(&bot::wrap(&exit), "41/109").unwrap();
        self.until("A back in the world", |rig| {
            rig.host(|s| s.test_location(A).is_some_and(|row| row.0.zone_id == 0))
        });
        self.ready(A);
    }

    /// `who` sends the family-40 leave request and the server drops the party.
    fn leave_party(&mut self, who: u64) {
        let mut out = vec![40, 0, 3, 0];
        out.extend(who.to_le_bytes());
        name_record(&mut out, who);
        let bot = if who == A { &mut self.a } else { &mut self.b };
        bot.send_message(&bot::wrap(&out), "40/3").unwrap();
        self.until("the party to end", |rig| rig.group(who).is_none());
    }

    /// B switches to a non-combat job (38/21 comes back), after the invite is out.
    fn b_goes_idle(&mut self) {
        self.take(1);
        let job = field_battle_fixture::switch_job(self.idle_job);
        self.b.send_message(&bot::wrap(&job), "26/13").unwrap();
        self.until("B's job change", |rig| {
            rig.inbox[1].iter().any(|m| is_sub(m, 38, 21))
        });
    }

    /// B sends a 41/103; both inboxes are emptied first so only the answer's effects remain.
    fn b_answers(&mut self, reply: Reply) {
        self.b_sends(&invite::encode_reply(&reply));
    }

    fn b_sends(&mut self, inner: &[u8]) {
        self.settle();
        self.b.send_message(&bot::wrap(inner), "41/103").unwrap();
    }

    /// Pump until nothing but ambient traffic has arrived for 400 ms (a scene load can stream for
    /// seconds), then empty both inboxes.
    fn settle(&mut self) {
        let deadline = Instant::now() + PATIENCE;
        let mut quiet_since = Instant::now();
        while quiet_since.elapsed() < Duration::from_millis(400) {
            assert!(Instant::now() < deadline, "the server never went quiet");
            self.pump(Duration::from_millis(50));
            if self.inbox.iter().flatten().any(|m| !ambient(m)) {
                quiet_since = Instant::now();
            }
            self.take(0);
            self.take(1);
        }
    }

    /// Nothing B owns changed and, after the server settled, neither bot was sent anything but
    /// ambient actor traffic (family 125 updates, 35/8 NPC idle animations): those arrive on the
    /// server's own timer whatever B answered. Any other message, such as a refusal chat, an
    /// error reply or a transfer, fails.
    fn assert_inert(&mut self, what: &str, before: &Place) {
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

fn invites_in(messages: &[Vec<u8>]) -> Vec<invite::InvitePacket> {
    messages
        .iter()
        .filter(|m| is_sub(m, 41, 102))
        .map(|m| invite::decode_invite(m).expect("a strict 41/102"))
        .collect()
}

fn reply(encounter: i32, run: i32, character: u64, accept: bool) -> Reply {
    Reply {
        encounter,
        run,
        character,
        accept,
    }
}

#[test]
fn a_grouped_start_invites_the_member_and_only_accept_enters_the_run() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    let battles = field_battle_fixture::battles();
    let title =
        field_battle_fixture::i32_field(field_battle_fixture::battle(&battles, BATTLE), "title_id");

    // No invite outstanding: an accept changes nothing.
    let before = rig.place(B);
    rig.b_answers(reply(0, 0, B, true));
    rig.assert_inert("a reply with no invite", &before);

    // Round 1: B gets exactly one prompt with the recorded fields.
    let run = rig.a_starts_battle();
    rig.pump(Duration::from_millis(600));
    let invites = invites_in(&rig.take(1));
    assert_eq!(
        invites,
        vec![invite::InvitePacket {
            encounter: BATTLE,
            run,
            inviter: A,
            title_id: title,
            countdown: invite::COUNTDOWN,
        }]
    );
    assert!(
        invites_in(&rig.take(0)).is_empty(),
        "the starter gets no prompt"
    );

    // Malformed shapes: the pre-codec forged layout (13 bytes), trailing bytes, accept flag 2.
    // They leave the invite answerable.
    let before = rig.place(B);
    let good = invite::encode_reply(&reply(0, 0, B, true));
    let mut flag = good.clone();
    flag[20] = 2;
    for (what, bytes) in [
        ("short", good[..13].to_vec()),
        ("trailing", [good.clone(), vec![0]].concat()),
        ("accept flag 2", flag),
    ] {
        rig.b_sends(&bytes);
        rig.assert_inert(what, &before);
    }

    // Decline with the stale word B a client that already saw a battle sends: the invite is
    // cleared and nothing else moves; both stay grouped.
    rig.b_answers(reply(0, STALE_RUN, B, false));
    rig.assert_inert("decline", &before);
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));
    // A late accept finds no invite.
    rig.b_answers(reply(0, 0, B, true));
    rig.assert_inert("accept after decline", &before);

    // Round 2: A leaves the run after inviting; B's accept is then inert (the run is gone).
    rig.a_leaves_battle(run);
    let run = rig.a_starts_battle();
    rig.pump(Duration::from_millis(600));
    assert_eq!(
        invites_in(&rig.take(1)).len(),
        1,
        "an invite for the new run"
    );
    rig.a_leaves_battle(run);
    let before = rig.place(B);
    rig.b_answers(reply(0, STALE_RUN, B, true));
    rig.assert_inert("accept after the starter left", &before);

    // Round 3: an expired reply (clock moved past the 30 s deadline) is inert.
    let run = rig.a_starts_battle();
    rig.pump(Duration::from_millis(600));
    assert_eq!(
        invites_in(&rig.take(1)).len(),
        1,
        "a fresh invite for the new run"
    );
    rig.host(|s| s.test_advance_clock(invite::TTL_MS + 1_000));
    let before = rig.place(B);
    rig.b_answers(reply(0, 0, B, true));
    rig.assert_inert("expired", &before);

    // Round 4: Accept enters A's instance through exactly one transfer. The reply carries the
    // stale word B, a wrong word A and another character's GUID: none of them is a key.
    rig.a_leaves_battle(run);
    let run = rig.a_starts_battle();
    rig.pump(Duration::from_millis(600));
    assert_eq!(invites_in(&rig.take(1)).len(), 1);
    let outside = rig.place(B);
    rig.b_answers(reply(95, STALE_RUN, A, true));
    rig.until("B in A's instance", |rig| {
        rig.host(move |s| s.test_location(B).is_some_and(|row| row.1 == run))
    });
    rig.ready(B);
    rig.pump(Duration::from_millis(600));
    let received = rig.take(1);
    assert_eq!(
        received.iter().filter(|m| m.starts_with(&[31, 0])).count(),
        1,
        "exactly one transfer"
    );
    let at_a = rig.host(|s| s.test_location(A).unwrap());
    let at_b = rig.host(|s| s.test_location(B).unwrap());
    assert_eq!((at_b.0.zone_id, at_b.1), (9600, run));
    assert_eq!(at_a.1, at_b.1, "B shares A's instance id");
    assert_eq!(at_b.3, BTreeSet::from([A, B]));
    // The battle itself began for B: the same battle, in A's run, live (not just a transfer).
    let title =
        field_battle_fixture::i32_field(field_battle_fixture::battle(&battles, BATTLE), "title_id");
    assert_eq!(
        rig.host(|s| s.test_joinable_run(A)),
        Some((BATTLE, run, title))
    );
    assert_eq!(
        rig.host(|s| s.test_joinable_run(B)),
        Some((BATTLE, run, title))
    );
    assert_ne!(outside, rig.place(B));
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));

    // A replayed accept sends nothing and moves nobody.
    let before = rig.place(B);
    rig.b_answers(reply(0, 0, B, true));
    rig.assert_inert("replayed accept", &before);
}

#[test]
fn an_accept_rechecks_the_party_and_the_go_gates_after_the_invite() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    // Each round: A starts the battle and B holds one live invite; then B's standing changes
    // and B accepts. The invite is still outstanding, so only Accept's own rechecks refuse.
    let round = |rig: &mut Rig, what: &str, change: fn(&mut Rig)| {
        let run = rig.a_starts_battle();
        rig.pump(Duration::from_millis(600));
        assert_eq!(invites_in(&rig.take(1)).len(), 1, "{what}: one invite");
        change(rig);
        rig.pump(Duration::from_millis(600));
        let before = rig.place(B);
        rig.b_answers(reply(0, STALE_RUN, B, true));
        rig.assert_inert(what, &before);
        let at = rig.host(|s| s.test_location(B).unwrap());
        assert_ne!(at.1, run, "{what}: B is not in A's run");
        rig.a_leaves_battle(run);
    };
    // B leaves the party after the invite: no longer grouped with A, so Accept enters nothing.
    round(&mut rig, "B left the party", |rig| rig.leave_party(B));
    assert!(rig.group(A).is_none() && rig.group(B).is_none());
    rig.form_party();
    // A leaves the party after the invite (A stays in its run).
    round(&mut rig, "A left the party", |rig| rig.leave_party(A));
    assert!(rig.group(A).is_none() && rig.group(B).is_none());
    rig.form_party();
    // B drops the combat job after the invite: Accept's Go gates refuse it.
    round(&mut rig, "B lost the combat job", Rig::b_goes_idle);
    assert!(rig.group(A).is_some() && rig.group(A) == rig.group(B));
}

#[test]
fn a_refused_accept_leaves_the_invite_answerable() {
    let mut rig = Rig::new(true, Some(BRAWLER));
    let run = rig.a_starts_battle();
    rig.pump(Duration::from_millis(600));
    assert_eq!(invites_in(&rig.take(1)).len(), 1);
    // B drops the combat job, so Accept's Go gates refuse it: nothing moves and no message goes.
    rig.b_goes_idle();
    rig.pump(Duration::from_millis(600));
    let before = rig.place(B);
    rig.b_answers(reply(0, STALE_RUN, B, true));
    rig.assert_inert("accept without a combat job", &before);
    assert_eq!(rig.host(|s| s.test_joinable_run(B)), None);
    // B picks the combat job back up and answers the same invite: it was not burned.
    rig.take(1);
    let job = field_battle_fixture::switch_job(BRAWLER);
    rig.b.send_message(&bot::wrap(&job), "26/13").unwrap();
    rig.until("B's job change back", |rig| {
        rig.inbox[1].iter().any(|m| is_sub(m, 38, 21))
    });
    rig.b_answers(reply(0, 0, B, true));
    rig.until("B in A's instance", |rig| {
        rig.host(move |s| s.test_location(B).is_some_and(|row| row.1 == run))
    });
    assert_eq!(rig.host(|s| s.test_joinable_run(B)).map(|r| r.1), Some(run));
}

#[test]
fn the_world_sweep_voids_an_invite_when_a_player_or_the_run_goes_away() {
    use osfr_tools::{
        encounter_invite::Pending,
        world::{InstanceKind, World, WorldPose},
    };
    let mut world = World::runtime();
    world
        .register_runtime_zone(osfr_tools::content::Zone {
            id: 9600,
            name: "9600".into(),
            geometry_id: Some(9600),
            zone_type: Some(2),
            tutorial: Some(false),
            sky: Some(String::new()),
            source: Vec::new(),
        })
        .unwrap();
    let run = world
        .create_child_instance(0, 9600, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    let pose = WorldPose::ORIGIN;
    let place = |world: &mut World, guid: u64, zone: u32, target: Option<i32>| {
        let change = world
            .prepare_membership(guid, zone, target, pose, true, 0)
            .unwrap();
        world.commit_membership(change);
    };
    let row = || Pending {
        inviter: A,
        run,
        encounter: BATTLE,
        deadline_ms: 30_000,
    };
    let offered = |world: &mut World| {
        place(world, A, 9600, Some(run));
        place(world, B, 0, None);
        assert!(world.invites_mut().offer(B, row(), 0, 16));
        world.collect_empty_instances(1_000);
        assert_eq!(
            world.invites_mut().len(),
            1,
            "both placed, inviter in its run"
        );
    };
    offered(&mut world);
    // The inviter leaves the run.
    place(&mut world, A, 0, None);
    world.collect_empty_instances(1_000);
    assert!(world.invites_mut().is_empty(), "inviter left the run");
    // The invitee disconnects (the server's `end_session` calls `leave`).
    offered(&mut world);
    world.leave(B, 2_000);
    world.collect_empty_instances(2_000);
    assert!(world.invites_mut().is_empty(), "invitee gone");
    // The inviter disconnects.
    offered(&mut world);
    world.leave(A, 3_000);
    world.collect_empty_instances(3_000);
    assert!(world.invites_mut().is_empty(), "inviter gone");
    // The deadline passes.
    offered(&mut world);
    world.collect_empty_instances(30_000);
    assert!(world.invites_mut().is_empty(), "expired");
}

#[test]
fn an_ungrouped_or_ineligible_player_gets_no_invite() {
    // Ungrouped: the solo Go sends B nothing.
    let mut rig = Rig::new(false, Some(BRAWLER));
    rig.a_starts_battle();
    rig.pump(Duration::from_millis(800));
    assert!(invites_in(&rig.take(1)).is_empty());
    assert!(invites_in(&rig.take(0)).is_empty());
    drop(rig);
    // Grouped but without a combat job: B could not press Go, so it gets no prompt (R).
    let mut rig = Rig::new(true, None);
    rig.a_starts_battle();
    rig.pump(Duration::from_millis(800));
    assert!(invites_in(&rig.take(1)).is_empty());
}

/// Actor traffic the server sends on its own timer: family 125 updates, 35/8 NPC animations.
fn ambient(message: &[u8]) -> bool {
    message.starts_with(&[125, 0]) || is_sub(message, 35, 8)
}
fn is_sub(message: &[u8], family: u16, sub: u16) -> bool {
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
/// 40/1 (invite) and 40/4 (accept) as the client sends them.
fn group_request(accept: bool, invite_id: u32) -> Vec<u8> {
    let mut out = vec![40, 0, if accept { 4 } else { 1 }, 0];
    out.extend(A.to_le_bytes());
    name_record(&mut out, A);
    out.extend(B.to_le_bytes());
    name_record(&mut out, B);
    out.extend(invite_id.to_le_bytes());
    if accept {
        out.push(1);
    }
    out.extend([0; 20]);
    out
}
fn packet(opcode: u16) -> Vec<u8> {
    let mut out = 5_u16.to_le_bytes().to_vec();
    out.push(1);
    out.extend(2_u32.to_le_bytes());
    out.extend(opcode.to_le_bytes());
    out
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
    let path = root.join("accounts.json");
    AccountStore::new().save(&path).unwrap();
    for id in [A, B] {
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
