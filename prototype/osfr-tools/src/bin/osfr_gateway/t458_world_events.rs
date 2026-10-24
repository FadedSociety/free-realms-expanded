//! T458: world events through the gateway host. An admin's `realm event start|stop` goes through
//! `Session::dispatch_message` on a real `Server` (host dev role, no account store); the world
//! tick is `world_event_routing::tick`, the function `Server::step` calls, on a fixed clock. Two
//! viewer sessions sit in the open-world registry, one near the event actor and one far. The
//! content is the synthetic vendor set plus a fixture `world_events.json`; the expected bytes
//! follow the client readers cited in `osfr_tools::world_events`.
use super::test_support::{self, Scratch};
use super::*;
use osfr_tools::account_store::AccountRole;

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
        let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
        if !lab
            .join("reports/round59-native-20261015T124443Z/content")
            .is_dir()
            || !test_support::bin_test_target(tag)
        {
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

    /// One world-event tick, then each viewer's inner packets.
    fn tick(&mut self, now: u64) -> [Vec<Vec<u8>>; 2] {
        world_event_routing::tick(&mut self.server.world, &mut self.server.sessions, now);
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

/// A full reliable queue refuses only that viewer: the tick returns, the other viewer is served,
/// and once the queue drains a later tick shows the refused viewer the actor.
#[test]
fn t458_a_refusing_queue_only_affects_its_own_session() {
    let Some(mut rig) = Rig::new("t458-queue", Some(show())) else {
        return;
    };
    rig.walk(1, [2010.0, 0.0, 2000.0]);
    let full = |max_queued_bytes| {
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
    };
    let refused = rig.viewers[0];
    rig.server.sessions.get_mut(&refused).unwrap().sender = full(1);
    rig.realm("event start show", T0);
    let [near, other] = rig.tick(T0);
    assert!(near.is_empty(), "the full queue took nothing");
    assert_eq!(
        other.len(),
        2,
        "the other viewer got the AddNpc and the poof"
    );
    let guid = spawned(&other[0]);
    rig.server.sessions.get_mut(&refused).unwrap().sender = full(32 * 1024 * 1024);
    let [near, other] = rig.tick(T0 + 1_100);
    assert_eq!(
        other,
        [animation(guid, BASE, true)],
        "the other viewer just goes on"
    );
    assert_eq!(
        near.len(),
        2,
        "shown again: AddNpc and the current base animation"
    );
    assert_eq!(
        (spawned(&near[0]), &near[1]),
        (guid, &animation(guid, BASE, true))
    );
}

/// The production tick is wired into `Server::step`: one step after a start runs the spawn.
#[test]
fn t458_server_step_runs_the_world_event_tick() {
    let Some(mut rig) = Rig::new("t458-step", Some(show())) else {
        return;
    };
    // The server clock, which already passed the rig's last admin poll (300 ms).
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
