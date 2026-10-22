//! W111: synthetic, host-authorized membership fixture. No shared combat claim.
#[path = "../src/bin/osfr-bot.rs"]
#[allow(dead_code)]
mod bot;
#[path = "../src/bin/osfr-gateway.rs"]
#[allow(dead_code)]
mod gateway_runner;
#[path = "common/group_invite_wire.rs"]
mod group_invite_wire;
#[path = "common/startup_selfdata_fixture.rs"]
mod startup_selfdata_fixture;
use gateway_runner::{Options, Server};
use osfr_tools::{
    account::{AccountFlags, SlotPolicy, create_character},
    account_store::AccountStore,
    character_store::CharacterState,
    identity::{NativeCreation, Resources},
    login,
    players::ZoneInstance,
    social_chat::{self, ChatChannel, NameData, PacketChat, QuickChatCatalog},
    startup_snapshot::StartupSnapshot,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
const VERSION: &str = "synthetic.1";
const HANDOFF_KEY: &[u8] = b"synthetic-handoff-key";
// Upper bound for "the server gets to it", not a pace: waits return as soon as their condition
// holds. A loaded machine (parallel cargo builds and test binaries) can starve the gateway thread
// for seconds, so no wait in this file is sized by how long an idle machine takes.
const PATIENCE: Duration = Duration::from_secs(30);

type HostAction = Box<dyn FnOnce(&mut Server) + Send>;
struct Fixture {
    addr: SocketAddr,
    tx: Option<mpsc::Sender<HostAction>>,
    handle: Option<thread::JoinHandle<()>>,
    root: PathBuf,
    accounts: PathBuf,
    version: String,
}

impl Fixture {
    fn new() -> Self {
        Self::start(false)
    }

    /// Real gameplay content. The chat-scope test moves the speaker with the `realm goto`
    /// developer command, which only exists when gameplay content is loaded; the synthetic
    /// reference the `new` path builds has none.
    fn with_content() -> Self {
        Self::start(true)
    }

    fn start(content: bool) -> Self {
        let version = if content { "1.910.1.530630" } else { VERSION };
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(format!(
                "lab/reports/shared-world-w108/slice-1/socket-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("fixture clock precedes Unix epoch")
                    .as_nanos()
            ));
        assert!(!root.exists(), "scratch fixture already exists");
        fs::create_dir_all(&root).unwrap();
        let reference = if content {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        } else {
            create_reference(&root)
        };
        let accounts = create_account_store(&root);
        let options = Options {
            bind: "127.0.0.1:0".parse().unwrap(),
            reference,
            state: root.join("state.json"),
            content: content.then(|| {
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../lab/references/rust-opening-cooking-original-assets")
            }),
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
        Self {
            // Loading real content on a loaded shared host can outlast a fixed 10 s; this is
            // "the server gets to it", the same bound as every other wait in this file.
            addr: addr_rx.recv_timeout(PATIENCE).unwrap(),
            tx: Some(tx),
            handle: Some(handle),
            root,
            accounts,
            version: version.to_owned(),
        }
    }

    fn host<T: Send + 'static>(&self, f: impl FnOnce(&mut Server) -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        self.tx
            .as_ref()
            .unwrap()
            .send(Box::new(move |server| {
                tx.send(f(server)).unwrap();
            }))
            .unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap()
    }

    fn bot(&self, guid: u64, connection_id: u32) -> bot::Bot {
        let store = AccountStore::load(&self.accounts).unwrap();
        let account = store
            .account(store.character(guid).unwrap().owner_account_id)
            .unwrap();
        bot::Bot::connect(bot::BotConfig {
            gateway: self.addr,
            name: guid.to_string(),
            connection_id,
            guid,
            ticket: login::gateway_binding_ticket(HANDOFF_KEY, account, guid),
            version: self.version.clone(),
            expect_timeout_ms: 3000,
            quiet: true,
        })
        .unwrap()
    }

    fn ready(&self, guid: u64, a: &mut bot::Bot, b: &mut bot::Bot) -> [Vec<Vec<u8>>; 2] {
        let deadline = Instant::now() + PATIENCE;
        let mut messages = [Vec::new(), Vec::new()];
        let mut load_sent = false;
        loop {
            let batch = collect(a, b, Duration::from_millis(5));
            messages[0].extend(batch[0].clone());
            messages[1].extend(batch[1].clone());
            if self.host(move |s| s.test_location(guid).is_some_and(|row| row.2)) {
                return messages;
            }
            if !load_sent {
                let bot = if guid == 101 { &mut *a } else { &mut *b };
                bot.send_message(&packet(13, &[]), "13").unwrap();
                bot.send_message(&packet(10, &[]), "10").unwrap();
                load_sent = true;
            }
            assert!(Instant::now() < deadline, "readiness barrier for {guid}");
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        // Keep the numeric-only scratch saves as review evidence.
    }
}

fn collect(a: &mut bot::Bot, b: &mut bot::Bot, interval: Duration) -> [Vec<Vec<u8>>; 2] {
    let mut out = [Vec::new(), Vec::new()];
    let deadline = Instant::now() + interval;
    while Instant::now() < deadline {
        for (index, bot) in [&mut *a, &mut *b].into_iter().enumerate() {
            bot.pump(Duration::from_millis(2)).unwrap();
            while let Ok(message) = bot.wait_for(Duration::ZERO, |_| true) {
                out[index].push(message);
            }
        }
    }
    out
}

/// Collect into `acc` until `done` holds (or PATIENCE), then keep listening briefly so a
/// duplicate delivery still shows up in the exact-count assertions that follow.
fn settle(
    a: &mut bot::Bot,
    b: &mut bot::Bot,
    acc: &mut [Vec<Vec<u8>>; 2],
    done: impl Fn(&[Vec<Vec<u8>>; 2]) -> bool,
) {
    let deadline = Instant::now() + PATIENCE;
    while !done(acc) && Instant::now() < deadline {
        let batch = collect(a, b, Duration::from_millis(20));
        for i in 0..2 {
            acc[i].extend(batch[i].clone());
        }
    }
    let grace = collect(a, b, Duration::from_millis(500));
    for i in 0..2 {
        acc[i].extend(grace[i].clone());
    }
}

fn is_sub(message: &[u8], family: u16, sub: u16) -> bool {
    message.starts_with(&[
        family as u8,
        (family >> 8) as u8,
        sub as u8,
        (sub >> 8) as u8,
    ])
}
fn actor_packets(messages: &[Vec<u8>], sub: u16, guid: u64) -> usize {
    messages
        .iter()
        .filter(|m| is_sub(m, 35, sub) && m.get(4..12) == Some(guid.to_le_bytes().as_slice()))
        .count()
}
fn name(guid: u64) -> NameData {
    NameData {
        first: numeric_name(guid),
        last: numeric_name(guid),
        ..Default::default()
    }
}
// The creator accepts letters only. Derive synthetic labels; never retain player names.
fn numeric_name(id: u64) -> String {
    id.to_string()
        .bytes()
        .map(|digit| char::from(b'a' + digit - b'0'))
        .collect()
}
fn chat(channel: ChatChannel, sequence: u64) -> Vec<u8> {
    social_chat::encode_packet_chat(&PacketChat {
        channel,
        sender_guid: 0,
        target_guid: 0,
        sender_name: NameData::default(),
        target_name: if channel == ChatChannel::Tell {
            name(202)
        } else {
            NameData::default()
        },
        message: sequence.to_string(),
        position: [0.0; 4],
        guild_guid: 0,
        language: 0,
        area_id: None,
    })
    .unwrap()
}
fn name_record(out: &mut Vec<u8>, guid: u64) {
    out.extend([0; 12]);
    put_string(out, &numeric_name(guid));
    put_string(out, &numeric_name(guid));
}
fn group_request(accept: bool, invite_id: u32) -> Vec<u8> {
    let mut out = vec![40, 0, if accept { 4 } else { 1 }, 0];
    out.extend(101u64.to_le_bytes());
    name_record(&mut out, 101);
    out.extend(202u64.to_le_bytes());
    name_record(&mut out, 202);
    out.extend(invite_id.to_le_bytes());
    if accept {
        out.push(1);
    }
    out.extend([0; 20]);
    out
}
fn chat_count(messages: &[Vec<u8>], sequence: u64) -> usize {
    messages
        .iter()
        .filter_map(|m| social_chat::decode_packet_chat(m).ok())
        .filter(|row| row.sender_guid == 101 && row.message == sequence.to_string())
        .count()
}
fn assert_chat(messages: &[Vec<u8>], sequence: u64) {
    assert_eq!(chat_count(messages, sequence), 1);
}

#[test]
fn two_bot_socket_membership_and_transfer() {
    let f = Fixture::new();
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let store = AccountStore::load(&f.accounts).unwrap();
    for (guid, bot) in [(101, &mut a), (202, &mut b)] {
        let account = store.account(guid).unwrap();
        bot.send_message(
            &bot::login_message(
                guid,
                &login::gateway_binding_ticket(HANDOFF_KEY, account, guid),
                VERSION,
            ),
            "1",
        )
        .unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
        bot.send_message(&packet(13, &[]), "13").unwrap();
        bot.wait_for(PATIENCE, |_| true).unwrap();
    }
    a.send_message(&packet(10, &[]), "10").unwrap();
    f.ready(101, &mut a, &mut b);
    b.send_message(&packet(10, &[]), "10").unwrap();
    let mut public = f.ready(202, &mut a, &mut b);
    // The real socket join runs `SharedWorld::join_player`: `joined`, the player row and the
    // chat member must appear together. A join that skips `joined` leaves the session looking
    // unjoined, which diverges ride/chat/spawn from the registry.
    for guid in [101, 202] {
        assert_eq!(
            f.host(move |s| s.test_membership(guid)),
            Some((true, true, true)),
            "real join membership for {guid}"
        );
    }
    settle(&mut a, &mut b, &mut public, |m| {
        actor_packets(&m[0], 1, 202) >= 1 && actor_packets(&m[1], 1, 101) >= 1
    });
    assert_eq!(actor_packets(&public[0], 1, 202), 1);
    assert_eq!(actor_packets(&public[1], 1, 101), 1);
    assert_eq!(f.host(|s| s.test_location(101).unwrap().1), 0);
    // A real family-40 party, not an injected social state. The answer echoes the
    // generation from the delivered 40/1, not the group id.
    a.send_message(&bot::wrap(&group_request(false, 0)), "40/1")
        .unwrap();
    let invite = b.wait_for(PATIENCE, |m| is_sub(m, 40, 1)).unwrap();
    let invite_id = group_invite_wire::invite_generation(&invite);
    b.send_message(&bot::wrap(&group_request(true, invite_id)), "40/4")
        .unwrap();
    a.wait_for(PATIENCE, |m| is_sub(m, 40, 2)).unwrap();
    let (i1, i2) = f.host(|s| (s.test_child(), s.test_child()));
    assert_ne!(i1, i2);
    f.host(move |s| s.test_transfer(101, i1, false)).unwrap();
    f.ready(101, &mut a, &mut b);
    collect(&mut a, &mut b, Duration::from_millis(500));
    f.host(move |s| s.test_transfer(202, i2, false)).unwrap();
    let mut isolated = f.ready(202, &mut a, &mut b);
    // Both transfers ran `leave_player` and joined again; the three sets stay in lockstep.
    for guid in [101, 202] {
        assert_eq!(
            f.host(move |s| s.test_membership(guid)),
            Some((true, true, true)),
            "post-transfer membership for {guid}"
        );
    }
    let tail = collect(&mut a, &mut b, Duration::from_millis(500));
    for i in 0..2 {
        isolated[i].extend(tail[i].clone());
    }
    assert_eq!(
        actor_packets(&isolated[0], 1, 202),
        0,
        "isolation: A saw B in a different child"
    );
    assert_eq!(
        actor_packets(&isolated[1], 1, 101),
        0,
        "isolation: B saw A in a different child"
    );
    a.send_message(
        &bot::wrap(&bot::move_request(101, [2.0, 0.0, 0.0], [0.0, 0.0, 1.0])),
        "125",
    )
    .unwrap();
    b.send_message(
        &bot::wrap(&bot::move_request(202, [0.0, 0.0, 0.0], [0.0, 0.0, 1.0])),
        "125",
    )
    .unwrap();
    // The positions are the server-side proof that both move requests were handled.
    let mut moved = Default::default();
    settle(&mut a, &mut b, &mut moved, |_| {
        f.host(|s| s.test_location(101).unwrap().0.position) == [2.0, 0.0, 0.0]
            && f.host(|s| s.test_location(202).unwrap().0.position) == [0.0, 0.0, 0.0]
    });
    assert!(
        !moved[0]
            .iter()
            .any(|m| m.starts_with(&[125, 0])
                && m.get(2..10) == Some(202u64.to_le_bytes().as_slice()))
    );
    assert!(
        !moved[1]
            .iter()
            .any(|m| m.starts_with(&[125, 0])
                && m.get(2..10) == Some(101u64.to_le_bytes().as_slice()))
    );
    assert_eq!(
        f.host(|s| s.test_location(101).unwrap().0.position),
        [2.0, 0.0, 0.0]
    );
    assert_eq!(
        f.host(|s| s.test_location(202).unwrap().0.position),
        [0.0, 0.0, 0.0]
    );
    a.send_message(&bot::wrap(&chat(ChatChannel::Say, 108001)), "15/1")
        .unwrap();
    let mut local = Default::default();
    settle(&mut a, &mut b, &mut local, |m| {
        chat_count(&m[0], 108001) >= 1
    });
    assert_chat(&local[0], 108001);
    assert!(!local[1].iter().any(|m| is_sub(m, 15, 1)));
    let quick = social_chat::encode_quick_chat_channel(
        &QuickChatCatalog::from_ids([1]),
        1,
        ChatChannel::Say,
        0,
        &NameData::default(),
        0,
    )
    .unwrap();
    a.send_message(&bot::wrap(&quick), "67/3").unwrap();
    let mut local = Default::default();
    settle(&mut a, &mut b, &mut local, |m| {
        m[0].iter().any(|m| is_sub(m, 67, 3))
    });
    assert_eq!(local[0].iter().filter(|m| is_sub(m, 67, 3)).count(), 1);
    assert!(!local[1].iter().any(|m| is_sub(m, 67, 3)));
    for (channel, seq) in [(ChatChannel::Group, 222222), (ChatChannel::Tell, 333333)] {
        a.send_message(&bot::wrap(&chat(channel, seq)), "15/1")
            .unwrap();
        let mut messages = Default::default();
        settle(&mut a, &mut b, &mut messages, |m| {
            chat_count(&m[1], seq) >= 1
        });
        assert_chat(&messages[1], seq);
    }
    let before = f.host(|s| (s.test_location(202).unwrap(), s.test_membership(202)));
    let save_path = f.root.join("characters/202.json");
    // B's move was a pose-only change; the gateway defers that save by up to POSITION_SAVE_MS
    // ((R), 5 s). Wait for that write first, so `saved` is the settled file and the byte
    // comparison below can only fail if a rejected transfer wrote something.
    let flush_deadline = Instant::now() + PATIENCE;
    let saved = loop {
        let bytes = fs::read(&save_path).unwrap_or_default();
        if serde_json::from_slice::<serde_json::Value>(&bytes)
            .is_ok_and(|doc| doc["character"]["position"] == json!([0.0, 0.0, 0.0]))
        {
            break bytes;
        }
        assert!(
            Instant::now() < flush_deadline,
            "deferred pose save never reached disk"
        );
        collect(&mut a, &mut b, Duration::from_millis(50));
    };
    // A forged encounter reply carrying another instance is not entry authorization.
    let mut forged = vec![41, 0, 103, 0];
    forged.extend(0i32.to_le_bytes());
    forged.extend(i1.to_le_bytes());
    forged.push(1);
    b.send_message(&bot::wrap(&forged), "41/103").unwrap();
    collect(&mut a, &mut b, Duration::from_millis(500));
    assert_eq!(
        before,
        f.host(|s| (s.test_location(202).unwrap(), s.test_membership(202)))
    );
    assert!(f.host(|s| s.test_transfer(202, -1, false)).is_err());
    assert_eq!(
        before,
        f.host(|s| (s.test_location(202).unwrap(), s.test_membership(202)))
    );
    assert!(f.host(move |s| s.test_transfer(202, i1, true)).is_err());
    assert_eq!(
        before,
        f.host(|s| (s.test_location(202).unwrap(), s.test_membership(202)))
    );
    assert_eq!(saved, fs::read(&save_path).unwrap());
    let rejected = collect(&mut a, &mut b, Duration::from_millis(500));
    assert!(
        rejected
            .iter()
            .flatten()
            .all(|m| !is_sub(m, 35, 1) && !is_sub(m, 35, 3) && !m.starts_with(&[31, 0]))
    );
    f.host(move |s| s.test_transfer(202, i1, false)).unwrap();
    let mut together = f.ready(202, &mut a, &mut b);
    // The final reunion is a full leave/rejoin into 101's instance too: the three views must
    // still agree after it, not only after the earlier transfers.
    for guid in [101, 202] {
        assert_eq!(
            f.host(move |s| s.test_membership(guid)),
            Some((true, true, true)),
            "reunion membership for {guid}"
        );
    }
    settle(&mut a, &mut b, &mut together, |m| {
        actor_packets(&m[0], 1, 202) >= 1 && actor_packets(&m[1], 1, 101) >= 1
    });
    assert_eq!(actor_packets(&together[0], 1, 202), 1);
    assert_eq!(actor_packets(&together[1], 1, 101), 1);
    let row = f.host(|s| s.test_location(202).unwrap());
    assert_eq!(row.1, i1);
    assert_eq!(row.3, BTreeSet::from([101, 202]));
    a.send_message(&bot::wrap(&chat(ChatChannel::Say, 444444)), "15/1")
        .unwrap();
    let mut local = Default::default();
    settle(&mut a, &mut b, &mut local, |m| {
        chat_count(&m[1], 444444) >= 1
    });
    assert_chat(&local[1], 444444);
    println!(
        "public mutual AddPc; child isolation; movement/say/quick isolation; cross-instance party/tell; rollback; same-scene reunion: passed"
    );
}

#[test]
fn chat_scope_derives_zone_and_position_from_the_live_registry() {
    const FAR_SAY: u64 = 7;
    const NEAR_SAY: u64 = 800_000;
    const CROSS_INSTANCE_YELL: u64 = 8_675_309;
    const OTHER_ZONE_YELL: u64 = 90_000_000_000;
    const REJOINED_SAY: u64 = 1_234_567_890_123;
    let f = Fixture::with_content();
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let store = AccountStore::load(&f.accounts).unwrap();
    for (guid, bot) in [(101, &mut a), (202, &mut b)] {
        let account = store.account(guid).unwrap();
        bot.send_message(
            &bot::login_message(
                guid,
                &login::gateway_binding_ticket(HANDOFF_KEY, account, guid),
                &f.version,
            ),
            "1",
        )
        .unwrap();
    }
    for bot in [&mut a, &mut b] {
        bot.wait_for(PATIENCE, |_| true).unwrap();
        bot.send_message(&packet(13, &[]), "13").unwrap();
    }
    for bot in [&mut a, &mut b] {
        bot.wait_for(PATIENCE, |_| true).unwrap();
    }
    a.send_message(&packet(10, &[]), "10").unwrap();
    f.ready(101, &mut a, &mut b);
    b.send_message(&packet(10, &[]), "10").unwrap();
    f.ready(202, &mut a, &mut b);

    let child = f.host(|s| s.test_field_child());
    f.host(move |s| s.test_transfer(101, child, false)).unwrap();
    f.ready(101, &mut a, &mut b);
    f.host(move |s| s.test_transfer(202, child, false)).unwrap();
    f.ready(202, &mut a, &mut b);
    let (state, instance, ..) = f.host(|s| s.test_location(101).unwrap());
    assert_eq!(state.zone_id, 0);
    assert_ne!(state.zone_id as u64, instance as u64);

    f.host(|s| {
        s.test_observer_position(101, [31., 0., 0.]);
        s.test_observer_position(202, [0., 0., 0.]);
    });
    a.send_message(&bot::wrap(&chat(ChatChannel::Say, FAR_SAY)), "15/1")
        .unwrap();
    let mut far = Default::default();
    settle(&mut a, &mut b, &mut far, |m| {
        chat_count(&m[0], FAR_SAY) >= 1
    });
    assert_eq!(chat_count(&far[0], FAR_SAY), 1, "far say echo");
    assert_eq!(chat_count(&far[1], FAR_SAY), 0, "far say listener");

    f.host(|s| s.test_observer_position(101, [0., 0., 0.]));
    a.send_message(&bot::wrap(&chat(ChatChannel::Say, NEAR_SAY)), "15/1")
        .unwrap();
    let mut near = Default::default();
    // Wait for both copies: the sender and listener are independent sockets, so waiting only for
    // one makes the exact-count assertion on the other depend on the 500 ms grace.
    settle(&mut a, &mut b, &mut near, |m| {
        chat_count(&m[0], NEAR_SAY) >= 1 && chat_count(&m[1], NEAR_SAY) >= 1
    });
    assert_eq!(chat_count(&near[0], NEAR_SAY), 1, "near say echo");
    assert_eq!(chat_count(&near[1], NEAR_SAY), 1, "near say listener");

    // A second child of the same zone is a different instance: Yell must stay inside the
    // sender's instance (R), the scope the pre-D003 filter used.
    let sibling = f.host(|s| s.test_field_child());
    assert_ne!(sibling, child);
    f.host(move |s| s.test_transfer(101, sibling, false))
        .unwrap();
    f.ready(101, &mut a, &mut b);
    let (state, instance, ..) = f.host(|s| s.test_location(101).unwrap());
    assert_eq!(state.zone_id, 0);
    assert_eq!(instance, sibling);
    collect(&mut a, &mut b, Duration::from_millis(500));
    a.send_message(
        &bot::wrap(&chat(ChatChannel::Yell, CROSS_INSTANCE_YELL)),
        "15/1",
    )
    .unwrap();
    let mut cross_instance = Default::default();
    settle(&mut a, &mut b, &mut cross_instance, |m| {
        chat_count(&m[0], CROSS_INSTANCE_YELL) >= 1
    });
    assert_eq!(
        chat_count(&cross_instance[0], CROSS_INSTANCE_YELL),
        1,
        "cross-instance yell echo"
    );
    assert_eq!(
        chat_count(&cross_instance[1], CROSS_INSTANCE_YELL),
        0,
        "cross-instance yell listener"
    );

    let other_zone = f.host(|s| s.test_child());
    f.host(move |s| s.test_transfer(101, other_zone, false))
        .unwrap();
    f.ready(101, &mut a, &mut b);
    assert_eq!(f.host(|s| s.test_location(101).unwrap().0.zone_id), 9600);
    collect(&mut a, &mut b, Duration::from_millis(500));
    a.send_message(
        &bot::wrap(&chat(ChatChannel::Yell, OTHER_ZONE_YELL)),
        "15/1",
    )
    .unwrap();
    let mut yell = Default::default();
    settle(&mut a, &mut b, &mut yell, |m| {
        chat_count(&m[0], OTHER_ZONE_YELL) >= 1
    });
    assert_eq!(chat_count(&yell[0], OTHER_ZONE_YELL), 1, "yell echo");
    assert_eq!(chat_count(&yell[1], OTHER_ZONE_YELL), 0, "yell other zone");

    f.host(move |s| s.test_transfer(101, child, false)).unwrap();
    f.ready(101, &mut a, &mut b);
    collect(&mut a, &mut b, Duration::from_millis(500));
    a.send_message(&bot::wrap(&chat(ChatChannel::Say, REJOINED_SAY)), "15/1")
        .unwrap();
    let mut rejoined = Default::default();
    settle(&mut a, &mut b, &mut rejoined, |m| {
        chat_count(&m[0], REJOINED_SAY) >= 1 && chat_count(&m[1], REJOINED_SAY) >= 1
    });
    assert_eq!(chat_count(&rejoined[0], REJOINED_SAY), 1, "rejoined echo");
    assert_eq!(
        chat_count(&rejoined[1], REJOINED_SAY),
        1,
        "rejoined listener"
    );
}

#[test]
fn conversions_cap_and_ready_mismatch() {
    assert!(ZoneInstance::from_world(0, -1).is_err());
    assert!(
        ZoneInstance {
            zone_id: 0,
            instance_id: i32::MAX as u64 + 1
        }
        .world_instance_id()
        .is_err()
    );
    let mut world = osfr_tools::world::World::runtime();
    let change = world
        .prepare_membership(101, 0, None, osfr_tools::world::WorldPose::ORIGIN, false, 0)
        .unwrap();
    world.commit_membership(change);
    assert!(world.mark_zone_ready(101, 1).is_err());
    assert!(!world.player_location(101).unwrap().ready);
    assert!(world.mark_zone_ready(101, 0).unwrap());
    let mut invalid_pose = osfr_tools::world::WorldPose::ORIGIN;
    invalid_pose.position[0] = f32::NAN;
    assert!(
        world
            .prepare_membership(101, 0, Some(0), invalid_pose, false, 0)
            .is_err()
    );
    assert!(world.player_location(101).unwrap().ready);
    for _ in 0..16 {
        world
            .create_child_instance(
                0,
                0,
                osfr_tools::world::InstanceKind::Encounter { encounter_id: 0 },
                0,
            )
            .unwrap();
    }
    assert!(
        world
            .create_child_instance(
                0,
                0,
                osfr_tools::world::InstanceKind::Encounter { encounter_id: 0 },
                0
            )
            .is_err()
    );
    assert_eq!(world.player_location(101).unwrap().instance_id, 0);
    let mut overflow = osfr_tools::world::World::new(
        osfr_tools::content::ContentDb {
            zones: vec![osfr_tools::content::Zone {
                id: i32::MAX as u32 - 1,
                name: String::new(),
                geometry_id: Some(0),
                zone_type: Some(2),
                tutorial: Some(false),
                sky: Some(String::new()),
                source: vec![],
            }],
            npcs: vec![],
            pois: vec![],
            items: vec![],
            quests: vec![],
            jobs: vec![],
            dungeons: vec![],
            minigames: vec![],
        },
        60_000,
    );
    let entry = overflow
        .enter_public(
            101,
            i32::MAX as u32 - 1,
            osfr_tools::world::WorldPose::ORIGIN,
            0,
        )
        .unwrap();
    for _ in 0..2 {
        assert_eq!(
            overflow.create_child_instance(
                entry.instance_id,
                i32::MAX as u32 - 1,
                osfr_tools::world::InstanceKind::Encounter { encounter_id: 0 },
                0
            ),
            Err(osfr_tools::world::WorldError::InstanceIdsExhausted)
        );
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
        serde_json::to_vec(&{
            let mut snapshot = StartupSnapshot::empty();
            snapshot
                .world
                .quick_chat
                .push(osfr_tools::startup_world::QuickChatRow {
                    key: 1,
                    id: 1,
                    display_string_id: 1,
                    say_string_id: 1,
                    animation_id: 0,
                    parent_id: 0,
                    admin_only: 0,
                    menu_icon_id: 0,
                    item_id: 0,
                    item_category: 0,
                });
            snapshot
        })
        .unwrap(),
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
    let path = root.join("accounts.json");
    AccountStore::new().save(&path).unwrap();
    for id in [101u64, 202] {
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
