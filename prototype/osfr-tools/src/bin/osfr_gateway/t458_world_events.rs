//! T458: world events through the gateway host. An admin's `realm event start|stop` goes through
//! `Session::dispatch_message` on a real `Server` (host dev role, no account store); the world
//! tick is `world_event_routing::tick`, the function `Server::step` calls, on a fixed clock. Two
//! viewer sessions sit in the open-world registry, one near the event actor and one far. The
//! content is the synthetic vendor set plus a fixture `world_events.json`; the expected bytes
//! follow the client readers cited in `osfr_tools::world_events`.
use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::account_store::AccountRole;
use osfr_tools::inventory::InventoryItem;

// T538: the shared client packet builders (26/8, 26/6 and the 5/1 tunnel), not a local copy.
#[path = "../../../tests/common/gateway_packets.rs"]
mod gateway_packets;

const STAR: [f32; 3] = [2000.0, 0.0, 2000.0];
const NEAR: [f32; 3] = [2005.0, 0.0, 2000.0];
const FAR: [f32; 3] = [2500.0, 0.0, 2000.0];
const OPEN_WORLD: ZoneInstance = ZoneInstance {
    zone_id: 0,
    instance_id: 0,
};
const T0: u64 = 10_000;
const POOF: i32 = 21;
const BASE: i32 = 7;
const PLAY: i32 = 9;
const SPARK: i32 = 33;
const AURA: i32 = 50;

/// One actor and every op. The timeline is deliberately not in time order: the loader sorts it,
/// and running it as written (the canary) breaks the order the near session must see.
fn show() -> serde_json::Value {
    json!({"events": [{"key": "show",
    "actors": [{"key": "star", "model_id": 1, "name_string_id": 0, "npc_name": "Event Star",
        "position": STAR, "heading": 0.0, "stand_animation_id": 1, "hide_nameplate": false,
        "source": "T458 synthetic fixture"}],
    "timeline": [
        {"at_ms": 3000, "op": {"remove": {"actor": "star", "effect_id": POOF}}},
        {"at_ms": 2000, "op": {"tag_off": {"actor": "star", "effect_id": AURA}}},
        {"at_ms": 0, "op": {"spawn": {"actor": "star", "effect_id": POOF}}},
        {"at_ms": 1500, "op": {"tag_on": {"actor": "star", "effect_id": AURA}}},
        {"at_ms": 1000, "op": {"base_anim": {"actor": "star", "id": BASE}}},
        {"at_ms": 1200, "op": {"play_anim": {"actor": "star", "id": PLAY}}},
        {"at_ms": 1300, "op": {"effect": {"actor": "star", "id": SPARK}}}
    ]}]})
}

fn bytes(fields: &[&[u8]]) -> Vec<u8> {
    fields.concat()
}

/// 35/8, reader 0x008e5dd0: guid, animation id, i32 (0), base flag.
fn animation(guid: u64, id: i32, base: bool) -> Vec<u8> {
    let flag = [u8::from(base)];
    bytes(&[
        &[35, 0, 8, 0],
        &guid.to_le_bytes(),
        &id.to_le_bytes(),
        &[0; 4],
        &flag,
    ])
}

/// 35/16, reader 0x008e5f50: guid, u64 (0), effect, i32 (0), delay (0), x y z 1, clear (0).
fn play_effect(guid: u64, effect: i32) -> Vec<u8> {
    let position: Vec<u8> = [STAR[0], STAR[1], STAR[2], 1.0]
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    let head = bytes(&[&[35, 0, 16, 0], &guid.to_le_bytes(), &[0; 8]]);
    bytes(&[&head, &effect.to_le_bytes(), &[0; 8], &position, &[0]])
}

/// 35/41, reader 0x008d7ad0: guid, tag id, composite effect, source guid, u64 (0).
fn tag_on(guid: u64, tag: i32) -> Vec<u8> {
    let head = bytes(&[&[35, 0, 41, 0], &guid.to_le_bytes(), &tag.to_le_bytes()]);
    bytes(&[&head, &AURA.to_le_bytes(), &guid.to_le_bytes(), &[0; 8]])
}

/// 35/42, reader 0x008d7bc0: guid and the tag id 35/41 used (not the effect id).
fn tag_off(guid: u64, tag: i32) -> Vec<u8> {
    bytes(&[&[35, 0, 42, 0], &guid.to_le_bytes(), &tag.to_le_bytes()])
}

/// 35/3 submode 1, readers 0x008d69f0 and 0x008d6a60: guid, animate, delay, effect delay,
/// composite effect, duration.
fn removal(guid: u64, effect: i32) -> Vec<u8> {
    let head = bytes(&[&[35, 0, 3, 0, 1, 0], &guid.to_le_bytes(), &[0; 9]]);
    bytes(&[&head, &effect.to_le_bytes(), &[0; 4]])
}

/// The guid of an AddNpc (35/2 reader 0x00921a10 starts with it), checked for the 'EVNT' tag.
fn spawned(packet: &[u8]) -> u64 {
    assert_eq!(&packet[..4], &[35, 0, 2, 0], "not an AddNpc");
    let guid = u64::from_le_bytes(packet[4..12].try_into().unwrap());
    assert_eq!(
        guid >> 32,
        0x4556_4e54,
        "an event guid carries the EVNT tag"
    );
    guid
}

struct Rig {
    _scratch: Scratch,
    server: Server,
    admin: SessionKey,
    viewers: [SessionKey; 2],
}

impl Rig {
    /// `None` when the private lab fixtures are absent.
    fn new(tag: &str, events: Option<serde_json::Value>) -> Option<Self> {
        if !test_support::lab_content().is_dir() || !test_support::bin_test_target(tag) {
            eprintln!("skip {tag}: the private lab fixtures are absent");
            return None;
        }
        let scratch = Scratch::new(tag);
        let content = test_support::vendor_content(&scratch);
        if let Some(file) = events {
            fs::write(content.join("world_events.json"), file.to_string()).unwrap();
        }
        let mut options = test_support::options(scratch.state());
        options.content = Some(content);
        options.dev_tools = true;
        let server = Server::new(options).unwrap();
        let mut rig = Self {
            _scratch: scratch,
            server,
            admin: SessionKey {
                peer: "127.0.0.1:44001".parse().unwrap(),
                connection_id: 1,
            },
            viewers: [2, 3].map(|id| SessionKey {
                peer: format!("127.0.0.1:{}", 44_000 + id).parse().unwrap(),
                connection_id: id,
            }),
        };
        rig.add(rig.admin, None);
        let mut login = 1u16.to_le_bytes().to_vec();
        for (index, text) in ["t458", "1.910.1.530630", ""].into_iter().enumerate() {
            login.extend((text.len() as u32).to_le_bytes());
            login.extend(text.as_bytes());
            if index == 0 {
                login.extend(1u64.to_le_bytes());
            }
        }
        rig.dispatch(rig.admin, &login, 100);
        rig.dispatch(rig.admin, &wrap_application(&[13, 0]), 200);
        rig.dispatch(rig.admin, &wrap_application(&[10, 0]), 300);
        assert!(rig.server.sessions[&rig.admin].gateway.is_world_loaded());
        assert_eq!(
            rig.server.sessions[&rig.admin].gateway.role(),
            AccountRole::Admin
        );
        let admin = rig
            .server
            .world
            .players
            .get(SessionId(1))
            .map(|p| p.position);
        assert!(
            admin.is_none_or(|p| (p[0] - STAR[0]).hypot(p[2] - STAR[2]) > 200.0),
            "the admin stays out of the actor's view"
        );
        for (index, position) in [NEAR, FAR].into_iter().enumerate() {
            rig.add(rig.viewers[index], Some(position));
        }
        rig.drain(rig.admin, 300);
        Some(rig)
    }

    /// A session; a viewer (`position`) joins the open-world registry without logging in.
    fn add(&mut self, key: SessionKey, position: Option<[f32; 3]>) {
        let id = u64::from(key.connection_id);
        let scratch = self
            .server
            .options
            .state
            .with_file_name(format!("viewer-{id}.json"));
        let mut options = test_support::options(scratch);
        options.content = None;
        let options = if position.is_some() {
            &options
        } else {
            &self.server.options
        };
        let mut session = Session::new(
            SessionId(id),
            key.peer,
            SessionRequest {
                id: key.connection_id,
                crc_length: 3,
                buffer_size: 1400,
            },
            options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        if let Some(position) = position {
            session.character_guid = 500 + id;
            let players = &mut self.server.world.players;
            players
                .join(session.id, 500 + id, 500 + id, OPEN_WORLD, position, 0.0)
                .unwrap();
        }
        self.server.world.session_keys.insert(session.id, key);
        self.server.sessions.insert(key, session);
    }

    fn dispatch(&mut self, key: SessionKey, message: &[u8], now: u64) {
        let server = &mut self.server;
        let mut session = server.sessions.remove(&key).unwrap();
        let result = session.dispatch_message(
            message,
            &server.options,
            now,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        );
        server.sessions.insert(key, session);
        result.unwrap();
    }

    fn drain(&mut self, key: SessionKey, now: u64) -> Vec<Vec<u8>> {
        let session = self.server.sessions.get_mut(&key).unwrap();
        super::super::f120_fishing_hour_reconnect::drain_replies(session, now)
    }

    /// `realm <text>` from the admin at `now`; the system lines it got back.
    fn realm(&mut self, text: &str, now: u64) -> Vec<String> {
        let inner = osfr_tools::chat::local_request(&format!("realm {text}"));
        self.dispatch(self.admin, &wrap_application(&inner), now);
        let replies = self.drain(self.admin, now);
        let evnt = 0x4556_4e54_u32.to_le_bytes();
        assert!(
            !replies.iter().any(|m| m.windows(4).any(|w| w == evnt)),
            "the admin is out of view and gets no event packets"
        );
        replies
            .iter()
            .filter(|message| message.get(7..11) == Some(&[15, 0, 3, 0]))
            .map(|message| {
                let length = u32::from_le_bytes(message[11..15].try_into().unwrap()) as usize;
                String::from_utf8(message[15..15 + length].to_vec()).unwrap()
            })
            .collect()
    }

    /// One world-event tick on the real wall clock, then each viewer's inner packets.
    fn tick(&mut self, now: u64) -> [Vec<Vec<u8>>; 2] {
        self.tick_at(now, osfr_tools::client_areas::unix_now_ms())
    }

    /// `tick` on a chosen wall clock (T462: schedules run on it).
    fn tick_at(&mut self, now: u64, unix_ms: u64) -> [Vec<Vec<u8>>; 2] {
        let (world, sessions) = (&mut self.server.world, &mut self.server.sessions);
        world_event_routing::tick(world, sessions, now, unix_ms);
        self.viewers.map(|key| {
            let session = self.server.sessions.get_mut(&key).unwrap();
            super::super::f120_fishing_hour_reconnect::drain_replies(session, now)
                .into_iter()
                .map(|message| {
                    assert_eq!(&message[..3], &[5, 0, 1], "a 5/1 application tunnel");
                    message[7..].to_vec()
                })
                .collect()
        })
    }

    fn walk(&mut self, viewer: usize, position: [f32; 3]) {
        let id = SessionId(u64::from(self.viewers[viewer].connection_id));
        let players = &mut self.server.world.players;
        players.update_pose(id, OPEN_WORLD, position, 0.0).unwrap();
    }

    /// One session tick, as `Server::step` runs it after the world tick. T538: the staged
    /// gameplay of this transaction is where a queued dance-off join hook is emitted and saved.
    fn session_tick(&mut self, key: SessionKey, now: u64) {
        let server = &mut self.server;
        let mut session = server.sessions.remove(&key).unwrap();
        session
            .tick(
                &server.options,
                now,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
            )
            .unwrap();
        server.sessions.insert(key, session);
    }

    /// Everything a refused command must leave alone.
    fn snapshot(
        &self,
    ) -> (
        osfr_tools::game_store::Snapshot,
        Option<Vec<u8>>,
        osfr_tools::world_events::WorldEvents,
    ) {
        (
            self.server.sessions[&self.admin].gateway.snapshot(),
            fs::read(&self.server.options.state).ok(),
            self.server.world.world_events.clone(),
        )
    }
}

#[test]
fn t458_the_timeline_plays_in_time_order_for_the_near_session_only() {
    let Some(mut rig) = Rig::new("t458-order", Some(show())) else {
        return;
    };
    assert_eq!(
        rig.realm("event start show", T0),
        ["World event show started."]
    );
    let [near, far] = rig.tick(T0);
    assert_eq!(near.len(), 2, "AddNpc then the spawn poof");
    let guid = spawned(&near[0]);
    assert_eq!(near[1], play_effect(guid, POOF));
    assert!(far.is_empty(), "the far session gets nothing");
    // One late tick runs every step that fell due, once each, in time order.
    let [near, far] = rig.tick(T0 + 3_000);
    assert_eq!(
        near,
        [
            animation(guid, BASE, true),
            animation(guid, PLAY, false),
            play_effect(guid, SPARK),
            tag_on(guid, 1),
            tag_off(guid, 1),
            removal(guid, POOF),
        ]
    );
    assert!(far.is_empty(), "the far session gets nothing");
    // The run ended by itself: its last step ran and no actor is live.
    assert_eq!(rig.tick(T0 + 4_000), [Vec::<Vec<u8>>::new(), Vec::new()]);
    assert_eq!(
        rig.realm("event stop show", T0 + 4_000),
        ["World event show is not running."]
    );
}

#[test]
fn t458_a_session_that_walks_in_gets_the_current_state_once() {
    let Some(mut rig) = Rig::new("t458-late", Some(show())) else {
        return;
    };
    rig.realm("event start show", T0);
    let [near, _] = rig.tick(T0);
    let guid = spawned(&near[0]);
    let [near, far] = rig.tick(T0 + 1_600);
    assert_eq!(near.len(), 4, "base, play, effect and tag");
    assert!(far.is_empty());
    rig.walk(1, [2010.0, 0.0, 2000.0]);
    let [near, far] = rig.tick(T0 + 1_700);
    assert!(
        near.is_empty(),
        "the near session is not shown the actor again"
    );
    assert_eq!(
        far.len(),
        3,
        "AddNpc, current base animation, live tag: {far:?}"
    );
    assert_eq!(spawned(&far[0]), guid);
    assert_eq!(far[1..], [animation(guid, BASE, true), tag_on(guid, 1)]);
    assert_eq!(
        rig.tick(T0 + 1_800)[1],
        Vec::<Vec<u8>>::new(),
        "exactly once"
    );
    // Walking away removes it (no poof); walking back shows it again.
    rig.walk(1, FAR);
    assert_eq!(rig.tick(T0 + 1_850)[1], [removal(guid, 0)]);
    rig.walk(1, [2010.0, 0.0, 2000.0]);
    let [_, far] = rig.tick(T0 + 1_900);
    assert_eq!((far.len(), spawned(&far[0])), (3, guid));
}

#[test]
fn t458_stop_removes_every_live_actor_and_a_restart_uses_a_fresh_guid() {
    let Some(mut rig) = Rig::new("t458-stop", Some(show())) else {
        return;
    };
    rig.realm("event start show", T0);
    let [near, _] = rig.tick(T0);
    let first = spawned(&near[0]);
    assert_eq!(
        rig.realm("event stop show", T0 + 500),
        ["World event show stopped."]
    );
    // The removal carries the timeline's remove poof.
    assert_eq!(rig.tick(T0 + 500)[0], [removal(first, POOF)]);
    assert_eq!(
        rig.tick(T0 + 5_000),
        [Vec::<Vec<u8>>::new(), Vec::new()],
        "the run is gone"
    );
    assert_eq!(
        rig.realm("event start show", T0 + 6_000),
        ["World event show started."]
    );
    let [near, _] = rig.tick(T0 + 6_000);
    let second = spawned(&near[0]);
    assert_ne!(first, second, "a restart uses fresh guids");
    assert_eq!(near[1], play_effect(second, POOF));
}

#[test]
fn t458_refused_commands_change_nothing() {
    let Some(mut rig) = Rig::new("t458-refused", Some(show())) else {
        return;
    };
    rig.realm("event start show", T0);
    rig.tick(T0);
    let before = rig.snapshot();
    for (command, reply) in [
        (
            "event start show",
            "World event show is already running. Nothing changed.",
        ),
        (
            "event start nope",
            "nope is not a world event in world_events.json. Nothing changed.",
        ),
        ("event stop nope", "World event nope is not running."),
    ] {
        assert_eq!(rig.realm(command, T0 + 100), [reply]);
        assert_eq!(rig.snapshot(), before, "{command} changed something");
    }
    for role in [AccountRole::Player, AccountRole::Builder] {
        let admin = rig.admin;
        rig.server
            .sessions
            .get_mut(&admin)
            .unwrap()
            .gateway
            .set_role(role);
        for command in ["event start show", "event stop show"] {
            let reply = rig.realm(command, T0 + 200);
            assert_eq!(reply.len(), 1, "{role:?}: {reply:?}");
            assert!(
                reply[0].starts_with("Cannot do that"),
                "{role:?}: {reply:?}"
            );
            assert_eq!(
                rig.snapshot(),
                before,
                "{role:?}: {command} changed something"
            );
        }
    }
    assert_eq!(rig.tick(T0 + 300), [Vec::<Vec<u8>>::new(), Vec::new()]);
}

/// With no events configured, a start is refused and ticks send nothing.
#[test]
fn t458_no_events_configured_sends_nothing() {
    let Some(mut rig) = Rig::new("t458-idle", None) else {
        return;
    };
    assert_eq!(
        rig.realm("event start show", T0),
        ["show is not a world event in world_events.json. Nothing changed."]
    );
    for now in [T0, T0 + 50, T0 + 10_000] {
        assert_eq!(rig.tick(now), [Vec::<Vec<u8>>::new(), Vec::new()]);
    }
    assert_eq!(rig.server.world.world_events, Default::default());
}

/// A reliable sender that holds at most `max_queued_bytes` of unacknowledged wire bytes.
fn queue_limit(max_queued_bytes: usize) -> ReliableSender {
    let codec = Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap();
    let config = SenderConfig {
        datagram_limit: 1400,
        window: 32,
        max_message: MAX_MESSAGE,
        max_queued_bytes,
        retry_ms: 250,
        max_retries: 40,
    };
    ReliableSender::new(codec, config).unwrap()
}

/// The first run's actor: serial 1, index 0 (`world_events::GUID_TAG`).
const GUID: u64 = 0x4556_4e54_0000_0100;

/// A full reliable queue refuses only that viewer: the tick returns, the other viewer is served,
/// and once the queue drains a later tick shows the refused viewer the actor.
#[test]
fn t458_a_refusing_queue_only_affects_its_own_session() {
    let Some(mut rig) = Rig::new("t458-queue", Some(show())) else {
        return;
    };
    rig.walk(1, [2010.0, 0.0, 2000.0]);
    let refused = rig.viewers[0];
    rig.server.sessions.get_mut(&refused).unwrap().sender = queue_limit(1);
    rig.realm("event start show", T0);
    let [near, other] = rig.tick(T0);
    assert!(near.is_empty(), "the full queue took nothing");
    assert_eq!(
        other.len(),
        2,
        "the other viewer got the AddNpc and the poof"
    );
    let guid = spawned(&other[0]);
    rig.server.sessions.get_mut(&refused).unwrap().sender = queue_limit(32 * 1024 * 1024);
    let [near, other] = rig.tick(T0 + 1_100);
    assert_eq!(
        other,
        [animation(guid, BASE, true)],
        "the other viewer just goes on"
    );
    assert_eq!(
        near.len(),
        3,
        "the removal it is owed, then AddNpc and the current base animation"
    );
    assert_eq!(near[0], removal(guid, 0));
    assert_eq!(
        (spawned(&near[1]), &near[2]),
        (guid, &animation(guid, BASE, true))
    );
}

/// Finding 3: once a session refuses a packet, the packets behind it are not tried. A queue that
/// takes the small spawn poof but not the AddNpc must not get the poof for an actor it never
/// saw; it is later shown the actor once, behind a removal.
#[test]
fn t458_a_refused_packet_skips_the_rest_of_that_sessions_tick() {
    let Some(mut rig) = Rig::new("t458-skip", Some(show())) else {
        return;
    };
    let mut probe = queue_limit(100);
    assert!(
        probe
            .queue_message(&wrap_application(&play_effect(GUID, POOF)))
            .is_ok(),
        "the poof alone fits the small queue"
    );
    let near_key = rig.viewers[0];
    rig.server.sessions.get_mut(&near_key).unwrap().sender = queue_limit(100);
    rig.realm("event start show", T0);
    let [near, far] = rig.tick(T0);
    assert!(
        near.is_empty() && far.is_empty(),
        "the AddNpc was refused, so the poof behind it was not sent"
    );
    rig.server.sessions.get_mut(&near_key).unwrap().sender = queue_limit(32 * 1024 * 1024);
    let [near, _] = rig.tick(T0 + 1_100);
    assert_eq!(near.len(), 3, "removal, AddNpc, base animation");
    assert_eq!(near[0], removal(GUID, 0));
    assert_eq!(spawned(&near[1]), GUID);
    assert_eq!(near[2], animation(GUID, BASE, true));
}

/// Finding 3: a removal a session refused is not lost; the actor does not stay on its client.
#[test]
fn t458_a_refused_removal_is_sent_when_the_queue_has_room() {
    let Some(mut rig) = Rig::new("t458-ghost", Some(show())) else {
        return;
    };
    rig.realm("event start show", T0);
    let [near, _] = rig.tick(T0);
    assert_eq!(spawned(&near[0]), GUID);
    rig.realm("event stop show", T0 + 500);
    let near_key = rig.viewers[0];
    rig.server.sessions.get_mut(&near_key).unwrap().sender = queue_limit(1);
    let [near, _] = rig.tick(T0 + 500);
    assert!(near.is_empty(), "the removal was refused");
    rig.server.sessions.get_mut(&near_key).unwrap().sender = queue_limit(32 * 1024 * 1024);
    let [near, far] = rig.tick(T0 + 600);
    assert_eq!(near, [removal(GUID, POOF)], "owed, with its poof");
    assert!(far.is_empty());
    assert_eq!(rig.tick(T0 + 700), [Vec::<Vec<u8>>::new(), Vec::new()]);
}

/// The production tick is wired into `Server::step`: one step after a start runs the spawn.
#[test]
fn t458_server_step_runs_the_world_event_tick() {
    let Some(mut rig) = Rig::new("t458-step", Some(show())) else {
        return;
    };
    // The server clock, which already passed the rig's last admin poll (300 ms). The fixture is
    // fast enough to finish before that on a quiet machine, so wait for the clock explicitly.
    let deadline = Instant::now() + std::time::Duration::from_secs(2);
    while rig.server.start.elapsed().as_millis() < 300 && Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let now = rig.server.start.elapsed().as_millis() as u64;
    assert!(now >= 300, "the sender clock never runs backwards here");
    rig.realm("event start show", now);
    let started = rig.server.world.world_events.clone();
    for session in rig.server.sessions.values_mut() {
        session.last_activity = Instant::now();
    }
    rig.server.step().unwrap();
    assert_ne!(
        rig.server.world.world_events, started,
        "the step ran the spawn"
    );
    assert_eq!(
        rig.server.sessions.len(),
        3,
        "every session is still connected"
    );
}

/// T460: `say`, `broadcast` and `area` reach the viewers through the production tick. The near
/// viewer (5 m) hears the 15/4 line, both viewers get the zone-wide 15/3 and the area packet, and
/// `realm event stop` switches the area off for both.
#[test]
fn t460_say_broadcast_and_area_reach_the_viewers_through_the_tick() {
    let line = "Let the Dance-off begin!";
    let talk = json!({"events": [{"key": "talk",
        "actors": show()["events"][0]["actors"].clone(),
        "timeline": [
            {"at_ms": 0, "op": {"spawn": {"actor": "star"}}},
            {"at_ms": 100, "op": {"say": {"actor": "star", "string_id": 432_431}}},
            {"at_ms": 200, "op": {"broadcast": {"text": line, "locale_id": 432_431}}},
            {"at_ms": 300, "op": {"area": {"area_id": 4299, "on": true}}}]}]});
    let Some(mut rig) = Rig::new("t460-ops", Some(talk)) else {
        return;
    };
    let areas = rig._scratch.path().join("areas");
    fs::create_dir_all(&areas).unwrap();
    let switchable = json!({"schema_version": 1,
        "switchable": {"area_ids": [4299], "source": "T460 test"}});
    fs::write(areas.join("client_areas.json"), switchable.to_string()).unwrap();
    rig.server.world.client_areas = osfr_tools::client_areas::ClientAreas::load(&areas).unwrap();
    let area = |on: bool| bytes(&[&[89, 0], &4299_i32.to_le_bytes(), &[u8::from(on)], &[0; 4]]);
    let broadcast = osfr_tools::chat::system(line);

    rig.realm("event start talk", T0);
    let [near, far] = rig.tick(T0);
    let guid = spawned(&near[0]);
    assert_eq!((near.len(), far.len()), (1, 0));
    let [near, far] = rig.tick(T0 + 100);
    assert_eq!(near, [osfr_tools::chat::string_say(guid, 432_431)]);
    assert!(far.is_empty(), "the far viewer never saw the speaker");
    assert_eq!(
        rig.tick(T0 + 200),
        [vec![broadcast.clone()], vec![broadcast]]
    );
    assert_eq!(rig.tick(T0 + 300), [vec![area(true)], vec![area(true)]]);
    // The admin stands in the open world too: it heard the zone-wide line, then its own reply.
    assert_eq!(
        rig.realm("event stop talk", T0 + 400),
        [line, "World event talk stopped."]
    );
    assert_eq!(
        rig.tick(T0 + 400),
        [vec![removal(guid, 0), area(false)], vec![area(false)]]
    );
}

/// T462: `Server::new` reads the schedule from the content directory, and the production tick
/// announces to the whole zone one lead before the slot, starts the event for the viewer who is
/// near its actor, and does not start it again. The daily slot is 12 hours ahead of the real
/// clock the server read at startup, so the test needs no sleep.
#[test]
fn t462_a_scheduled_event_announces_then_starts_through_the_tick() {
    let line = "Showtime in one minute!";
    let now_minute = osfr_tools::client_areas::unix_now_ms() / 60_000;
    let mut file = show();
    file["events"][0]["schedule"] = json!({"slot_minutes": 1440,
        "offset_minutes": (now_minute + 720) % 1440, "announce_lead_ms": 60_000,
        "announce": {"text": line, "locale_id": 432_430}});
    let Some(mut rig) = Rig::new("t462-schedule", Some(file)) else {
        return;
    };
    let start = (now_minute + 720) * 60_000;
    let broadcast = osfr_tools::chat::system(line);
    let quiet = [Vec::<Vec<u8>>::new(), Vec::new()];
    assert_eq!(rig.tick_at(T0, start - 61_000), quiet, "before the lead");
    let heard = rig.tick_at(T0 + 100, start - 60_000);
    assert_eq!(heard, [vec![broadcast.clone()], vec![broadcast]]);
    assert_eq!(
        rig.tick_at(T0 + 200, start - 1_000),
        quiet,
        "the line goes out once"
    );
    let [near, far] = rig.tick_at(T0 + 300, start);
    assert_eq!(
        near[0][..4],
        [35, 0, 2, 0],
        "the AddNpc of the scheduled run"
    );
    assert!(far.is_empty(), "the far viewer is out of range");
    assert_eq!(
        rig.tick_at(T0 + 400, start + 1_000),
        quiet,
        "the slot does not repeat"
    );
}

/// T538: a dance-off whose area is the logged-in admin's own position, with the Voting Box and
/// both cakes within interaction reach. Ids are the shipped client rows the unit tests use.
fn t538_event(at: [f32; 3]) -> serde_json::Value {
    let cake = [at[0] + 1.0, at[1], at[2]];
    json!({"events": [{
        "key": "dance_off",
        "actors": [
            {"key": "voting_box", "model_id": 1, "name_string_id": 432403, "npc_name": "Voting Box", "position": at, "heading": 0.0, "stand_animation_id": 1, "hide_nameplate": false, "source": "T538 bin fixture"},
            {"key": "cake_vampire", "model_id": 1, "name_string_id": 432404, "npc_name": "Vampire Cake", "position": cake, "heading": 0.0, "stand_animation_id": 1, "hide_nameplate": false, "source": "T538 bin fixture"},
            {"key": "cake_werewolf", "model_id": 1, "name_string_id": 432405, "npc_name": "Werewolf Cake", "position": cake, "heading": 0.0, "stand_animation_id": 1, "hide_nameplate": false, "source": "T538 bin fixture"}
        ],
        "timeline": [
            {"at_ms": 0, "op": {"spawn": {"actor": "voting_box"}}},
            {"at_ms": 4000, "op": {"one_of": {"branches": [
                [{"at_ms": 100, "op": {"spawn": {"actor": "cake_vampire"}}}],
                [{"at_ms": 100, "op": {"spawn": {"actor": "cake_werewolf"}}}]]}}}
        ],
        "dance_off": {
            "center": at, "radius_m": 30.0, "voting_box": "voting_box", "join_scan_ms": 500,
            "prompt_string_id": 432429, "already_voted_string_id": 5252,
            "not_participant_string_id": 432505, "used_cake_string_id": 432430,
            "teams": [
                {"team": "vampire", "costume_name_ids": [14688], "cake_actor": "cake_vampire",
                 "vote_response_id": 401, "vote_label_string_id": 432207,
                 "cake_say_string_id": 432434},
                {"team": "werewolf", "costume_name_ids": [14690], "cake_actor": "cake_werewolf",
                 "vote_response_id": 402, "vote_label_string_id": 432208,
                 "cake_say_string_id": 432435}
            ],
            "source": "T538 bin fixture"
        }
    }]})
}

/// The guid of every AddNpc in `messages` (the 5/1 tunnel stripped).
fn t538_spawned(messages: &[Vec<u8>]) -> Vec<u64> {
    messages
        .iter()
        .filter_map(|message| {
            let body = gateway_packets::body(message);
            (body.starts_with(&[35, 0, 2, 0]))
                .then(|| u64::from_le_bytes(body[4..12].try_into().unwrap()))
        })
        .collect()
}

/// The string id of the first 15/4 line in `messages`.
fn t538_says(messages: &[Vec<u8>]) -> Option<i32> {
    messages.iter().find_map(|message| {
        let body = gateway_packets::body(message);
        (body.starts_with(&[15, 0, 4, 0]))
            .then(|| i32::from_le_bytes(body[12..16].try_into().unwrap()))
    })
}

/// Give the session a costume item and wear it in the whole-body slot on the profile others see,
/// the way a purchase and an equip would. `adopt_trade_inventory` is the public inventory
/// boundary that validates the state, so the fixture cannot wear an item the catalogue does not
/// define. Returns after asserting that the costume scan sees `name_id`.
fn t538_wears(session: &mut Session, definition_id: i32, name_id: i32) {
    let game = session.gateway.gameplay_mut().expect("gameplay is loaded");
    let mut state = game.inventory_state().clone();
    let next = state.next_instance_id.unwrap_or(0).max(
        state
            .items
            .iter()
            .map(|item| i64::from(item.id))
            .max()
            .unwrap_or(0)
            + 1,
    );
    let id = i32::try_from(next).expect("an instance id fits");
    state.next_instance_id = Some(i64::from(id) + 1);
    state.items.push(InventoryItem {
        definition_id,
        tint: 0,
        id,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    let profile = game.active_profile().unwrap_or(1);
    let slot = state
        .equipment
        .iter()
        .position(|entry| entry.profile_id == profile && entry.slot == 17)
        .or_else(|| state.equipment.iter().position(|entry| entry.slot == 17))
        .expect("a whole-body costume slot");
    state.equipment[slot].item_id = id;
    game.adopt_trade_inventory(state)
        .expect("the costume equips");
    assert!(
        game.equipped_costume_name_ids().contains(&name_id),
        "the worn costume's client NameId is what the dance-off scans"
    );
}

/// T538 end to end: the real `route_application` for 26/8 and 26/6, the world tick's costume
/// join, the votes choosing the ending and the winning cake serving each participant once.
#[test]
fn t538_costume_votes_choose_the_cake_which_serves_participants_once() {
    let Some(mut rig) = Rig::new("t538-dance", None) else {
        return;
    };
    let at = rig.server.sessions[&rig.admin].gateway.state().position;
    // The t458 viewers are bare spectators; T538 needs their client packets (votes and cake
    // clicks) to take the real post-commit route, so they join the shared world like the admin.
    for key in rig.viewers {
        rig.server
            .world
            .joined
            .insert(SessionId(u64::from(key.connection_id)));
    }
    let content = rig
        .server
        .options
        .content
        .clone()
        .expect("content directory");
    fs::write(
        content.join("world_events.json"),
        t538_event(at).to_string(),
    )
    .unwrap();
    // The admin wears the werewolf outfit (definition 17736, client NameId 14690).
    t538_wears(
        rig.server.sessions.get_mut(&rig.admin).unwrap(),
        17_736,
        14_690,
    );
    assert_eq!(
        rig.realm("event start dance_off", T0),
        ["World event dance_off started."]
    );
    // Both viewers stand at the dance area so their votes can answer the box.
    rig.walk(0, at);
    rig.walk(1, at);
    let _ = rig.tick(T0);
    let box_guid = *t538_spawned(&rig.drain(rig.admin, T0))
        .first()
        .expect("the Voting Box AddNpc");
    let serial = rig
        .server
        .world
        .world_events
        .open_dance_offs()
        .first()
        .map(|(_, serial)| *serial)
        .expect("the open dance-off");
    // The world tick joined the werewolf costume before the event steps; the team is the run's
    // record, keyed by character guid (the same key a reconnect reuses).
    assert_eq!(
        rig.server
            .world
            .world_events
            .dance_off_participant(serial, rig.server.sessions[&rig.admin].character_guid),
        Some(osfr_tools::quest_engine::DanceTeam::Werewolf),
        "the costume join recorded the werewolf team"
    );
    // The world tick only queues the quest hook on the live gameplay; the session's own tick
    // emits it from its staged clone, so the existing commit saves it (no second save writer).
    assert!(
        !rig.server.sessions[&rig.admin]
            .gateway
            .gameplay()
            .unwrap()
            .dance_off_join_credited(serial, osfr_tools::quest_engine::DanceTeam::Werewolf),
        "the world tick only queues the join hook"
    );
    rig.session_tick(rig.admin, T0 + 1);
    assert!(
        rig.server.sessions[&rig.admin]
            .gateway
            .gameplay()
            .unwrap()
            .dance_off_join_credited(serial, osfr_tools::quest_engine::DanceTeam::Werewolf),
        "the session tick emitted the join hook"
    );
    // Admin: open the vote prompt, step out of the box's reach and answer. The refusal sends no
    // close and changes nothing; a client 26/4 echo then must not clear the prompt either.
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::interact(box_guid)),
        T0 + 10,
    );
    let prompt = rig.drain(rig.admin, T0 + 10);
    assert!(
        prompt
            .iter()
            .any(|message| gateway_packets::body(message).starts_with(&[26, 0, 3, 0])),
        "the Voting Box answers the 26/3 prompt"
    );
    let me = rig.server.sessions[&rig.admin].id;
    let far = [at[0] + 100.0, at[1], at[2]];
    rig.server
        .world
        .players
        .update_pose(me, OPEN_WORLD, far, 0.0)
        .unwrap();
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::dialogue_choice(401)),
        T0 + 20,
    );
    let refused = rig.drain(rig.admin, T0 + 20);
    assert!(
        refused.is_empty(),
        "an out-of-reach answer sends no close, so the client keeps its prompt: {refused:?}"
    );
    assert_eq!(
        rig.server.world.world_events, before,
        "the refused answer changed nothing"
    );
    // The client echoes a 26/4 it was sent; it is not the server's close here. It must not clear
    // the prompt, or the player could never answer after stepping back.
    rig.dispatch(rig.admin, &gateway_packets::wrap(&[26, 0, 4, 0]), T0 + 25);
    let echo = rig.drain(rig.admin, T0 + 25);
    assert!(echo.is_empty(), "a stray close answers nothing: {echo:?}");
    assert_eq!(
        rig.server.world.world_events, before,
        "the close echo changed nothing"
    );
    // Back in reach, the same prompt still answers: no second click on the box is needed.
    rig.server
        .world
        .players
        .update_pose(me, OPEN_WORLD, at, 0.0)
        .unwrap();
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::dialogue_choice(401)),
        T0 + 30,
    );
    let _ = rig.drain(rig.admin, T0 + 30);
    // Viewers: one more vampire vote and one werewolf vote make it 2-1.
    for (index, response) in [(0usize, 401i32), (1, 402)] {
        let key = rig.viewers[index];
        rig.dispatch(
            key,
            &gateway_packets::wrap(&gateway_packets::interact(box_guid)),
            T0 + 30,
        );
        let _ = rig.drain(key, T0 + 30);
        rig.dispatch(
            key,
            &gateway_packets::wrap(&gateway_packets::dialogue_choice(response)),
            T0 + 40,
        );
        let _ = rig.drain(key, T0 + 40);
    }
    // A second vote is refused with the client's already-voted line and changes nothing: the
    // prompt it re-opened stays open, so the snapshot is taken after the click.
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::interact(box_guid)),
        T0 + 50,
    );
    let _ = rig.drain(rig.admin, T0 + 50);
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::dialogue_choice(402)),
        T0 + 60,
    );
    let refused = rig.drain(rig.admin, T0 + 60);
    assert_eq!(
        t538_says(&refused),
        Some(5252),
        "already voted: {refused:?}"
    );
    assert_eq!(
        rig.server.world.world_events, before,
        "the second vote changed no state"
    );
    // A disconnect keeps the character's vote and team: drop the viewer's session, re-add a
    // fresh one with the same character guid, and its second vote is still refused.
    let gone = rig.viewers[0];
    let gone_id = SessionId(u64::from(gone.connection_id));
    rig.server.sessions.remove(&gone);
    let _ = rig.server.world.players.leave(gone_id);
    rig.server.world.session_keys.remove(&gone_id);
    rig.server.world.joined.remove(&gone_id);
    rig.add(gone, Some(at));
    rig.server.world.joined.insert(gone_id);
    rig.dispatch(
        gone,
        &gateway_packets::wrap(&gateway_packets::interact(box_guid)),
        T0 + 70,
    );
    let _ = rig.drain(gone, T0 + 70);
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        gone,
        &gateway_packets::wrap(&gateway_packets::dialogue_choice(401)),
        T0 + 80,
    );
    let reconnected = rig.drain(gone, T0 + 80);
    assert_eq!(
        t538_says(&reconnected),
        Some(5252),
        "the vote survived the reconnect"
    );
    assert_eq!(
        rig.server.world.world_events, before,
        "the refused second vote changed no state"
    );
    // The ending: 2-1 for the vampires, so the vampire cake spawns.
    let _ = rig.tick(T0 + 4_100);
    let ended = rig.drain(rig.admin, T0 + 4_100);
    let cakes = t538_spawned(&ended);
    assert_eq!(cakes.len(), 1, "exactly the winning cake spawns: {cakes:?}");
    let cake_guid = cakes[0];
    // The werewolf-costumed admin is a participant, so the winning cake serves it once.
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::interact(cake_guid)),
        T0 + 4_110,
    );
    let used = rig.drain(rig.admin, T0 + 4_110);
    assert_eq!(
        t538_says(&used),
        Some(432434),
        "the vampire cake's line: the votes chose it"
    );
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        rig.admin,
        &gateway_packets::wrap(&gateway_packets::interact(cake_guid)),
        T0 + 4_120,
    );
    let again = rig.drain(rig.admin, T0 + 4_120);
    assert_eq!(t538_says(&again), Some(432430), "a second use is refused");
    assert_eq!(
        rig.server.world.world_events, before,
        "the second use changed no state"
    );
    // A non-participant (a voter with no costume) is refused and changes nothing.
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        rig.viewers[0],
        &gateway_packets::wrap(&gateway_packets::interact(cake_guid)),
        T0 + 4_130,
    );
    let denied = rig.drain(rig.viewers[0], T0 + 4_130);
    assert_eq!(t538_says(&denied), Some(432505), "not a participant");
    assert_eq!(
        rig.server.world.world_events, before,
        "the non-participant changed no state"
    );
    // After the ending the box opens nothing and a stray vote changes nothing.
    let before = rig.server.world.world_events.clone();
    rig.dispatch(
        rig.viewers[1],
        &gateway_packets::wrap(&gateway_packets::interact(box_guid)),
        T0 + 4_140,
    );
    let late = rig.drain(rig.viewers[1], T0 + 4_140);
    assert!(
        late.iter()
            .all(|message| !gateway_packets::body(message).starts_with(&[26, 0, 3, 0])),
        "no prompt after the ending"
    );
    rig.dispatch(
        rig.viewers[1],
        &gateway_packets::wrap(&gateway_packets::dialogue_choice(401)),
        T0 + 4_150,
    );
    let late_vote = rig.drain(rig.viewers[1], T0 + 4_150);
    assert!(
        late_vote.is_empty(),
        "a vote after the ending gets no reply and changes nothing: {late_vote:?}"
    );
    assert_eq!(
        rig.server.world.world_events, before,
        "a vote after the ending changed nothing"
    );
}
