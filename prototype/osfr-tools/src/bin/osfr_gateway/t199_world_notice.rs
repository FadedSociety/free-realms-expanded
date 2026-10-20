//! T199: `realm broadcast <text>` fanout through the real gateway dispatch and commit path.
//! Each session has its own reference guid, so several characters share one world without an
//! account store; the admin role is the host dev role, and demotion uses the same server-owned
//! `set_role` seam other denial tests use. Recipients that only receive need no content pack.
//! The replay case goes through `Session::handle` with a real reliable datagram, so the
//! transport duplicate check runs too; the other cases drive `Session::dispatch_message`.
use super::f120_fishing_hour_reconnect::drain_replies;
use super::*;
use fixtures::{copy_tree, wrap};
use osfr_tools::account_store::AccountRole;
use sha2::{Digest, Sha256};

#[path = "fishing_fixtures.rs"]
#[allow(dead_code)]
mod fixtures;

const NOW: u64 = 1_000;

/// The reference client login for this session's character (the fixture hardcodes guid 1).
fn login(guid: u64) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, value) in ["t199", "1.910.1.530630", ""].into_iter().enumerate() {
        bytes.extend((value.len() as u32).to_le_bytes());
        bytes.extend(value.bytes());
        if index == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

fn content() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}

/// A copy of the reference whose save starts at `guid`; each session is its own character.
/// The gateway looks for the startup snapshot at `<lab root>/content/startup-snapshot`, so
/// the scratch layout mirrors `lab/references` and `lab/content`.
fn reference(scratch: &Path, guid: u64) -> PathBuf {
    let lab = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let target = scratch.join("references").join(format!("reference-{guid}"));
    copy_tree(&lab.join("references/rust-gateway-reference"), &target);
    let manifest_path = target.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["initial_state"]["guid"] = serde_json::json!(guid);
    // The self-data template embeds the same guid, and the manifest pins its hash.
    let offset = manifest["self_layout"]["guid_offset"].as_u64().unwrap() as usize;
    let template = target.join(manifest["templates"][7]["file"].as_str().unwrap());
    let mut bytes = fs::read(&template).unwrap();
    bytes[offset..offset + 8].copy_from_slice(&guid.to_le_bytes());
    fs::write(&template, &bytes).unwrap();
    manifest["templates"][7]["sha256"] = serde_json::json!(format!("{:x}", Sha256::digest(&bytes)));
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let snapshot = scratch.join("content/startup-snapshot/startup.json");
    if !snapshot.is_file() {
        fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        fs::copy(lab.join("content/startup-snapshot/startup.json"), &snapshot).unwrap();
    }
    target
}

fn options(scratch: &Path, guid: u64, with_content: bool) -> Options {
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: reference(scratch, guid),
        state: scratch.join(format!("character-{guid}.json")),
        content: with_content.then(content),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: true,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
    }
}

struct Rig {
    scratch: PathBuf,
    world: SharedWorld,
    membership: World,
    sessions: SessionMap,
    keys: Vec<SessionKey>,
    options: Vec<Options>,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

impl Rig {
    /// `specs` are `(guid, with_content)`; every session logs in and enters the world.
    fn new(name: &str, specs: &[(u64, bool)]) -> Self {
        let scratch = std::env::temp_dir().join(format!("osfr-t199-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).unwrap();
        let mut rig = Self {
            scratch,
            world: SharedWorld::new(),
            membership: World::runtime(),
            sessions: SessionMap::new(),
            keys: Vec::new(),
            options: Vec::new(),
        };
        for (index, (guid, with_content)) in specs.iter().enumerate() {
            let id = 51 + index as u64;
            let options = options(&rig.scratch, *guid, *with_content);
            let mut session = Session::new(
                SessionId(id),
                format!("127.0.0.1:{}", 40_000 + id).parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    // The 200-character escaped 15/3 packet is ~870 bytes; the 512-byte
                    // datagram limit would fragment it and the drain helper skips fragments.
                    buffer_size: 1400,
                },
                &options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap();
            assert_eq!(session.character_guid, *guid);
            assert_eq!(session.gateway.role(), AccountRole::Admin);
            let key = SessionKey {
                peer: session.peer,
                connection_id: session.request.id,
            };
            rig.world.session_keys.insert(session.id, key);
            for (message, now) in [
                (login(*guid), 0u64),
                (wrap(&[13, 0]), 100),
                (wrap(&[10, 0]), 200),
            ] {
                session
                    .dispatch_message(
                        &message,
                        &options,
                        now,
                        &mut rig.world,
                        &mut rig.membership,
                        &mut rig.sessions,
                        None,
                    )
                    .unwrap();
            }
            rig.keys.push(key);
            rig.options.push(options);
            rig.sessions.insert(key, session);
        }
        rig.drain_all(NOW);
        assert_eq!(
            rig.world.chat_sessions.len(),
            specs.len(),
            "every session must have entered the world"
        );
        rig
    }

    fn send(&mut self, index: usize, message: &[u8], now: u64) {
        self.try_send_raw(index, &wrap(message), now).unwrap();
    }

    /// Send one already-framed top-level message; a combined 0/25 bundle is not re-wrapped.
    fn try_send_raw(
        &mut self,
        index: usize,
        message: &[u8],
        now: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Rig {
            world,
            membership,
            sessions,
            keys,
            options,
            ..
        } = self;
        let key = keys[index];
        let mut session = sessions.remove(&key).unwrap();
        let result = session.dispatch_message(
            message,
            &options[index],
            now,
            world,
            membership,
            sessions,
            None,
        );
        sessions.insert(key, session);
        result
    }

    /// Send one reliable application packet through the real transport framing. The same
    /// `sequence` twice is the client transport's duplicate/replay case.
    fn reliable(&mut self, index: usize, inner: &[u8], sequence: u16, now: u64) {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let Rig {
            world,
            membership,
            sessions,
            keys,
            options,
            ..
        } = self;
        let key = keys[index];
        let mut session = sessions.remove(&key).unwrap();
        let mut payload = sequence.to_be_bytes().to_vec();
        payload.extend(wrap(inner));
        let wire = session.codec.encode(RELIABLE_DATA, &payload).unwrap();
        let result = session.handle(
            &socket,
            &wire,
            &options[index],
            now,
            world,
            membership,
            sessions,
        );
        sessions.insert(key, session);
        result.unwrap();
    }

    /// Point this session's next save at a directory so its transaction fails after staging.
    fn break_save(&mut self, index: usize) {
        let key = self.keys[index];
        self.sessions.get_mut(&key).unwrap().state_path = self.scratch.clone();
    }

    fn drain(&mut self, index: usize, now: u64) -> Vec<Vec<u8>> {
        let Rig { sessions, keys, .. } = self;
        let key = keys[index];
        let mut session = sessions.remove(&key).unwrap();
        let messages = drain_replies(&mut session, now);
        sessions.insert(key, session);
        messages
    }

    fn drain_all(&mut self, now: u64) -> Vec<Vec<Vec<u8>>> {
        (0..self.keys.len())
            .map(|index| self.drain(index, now))
            .collect()
    }

    fn set_role(&mut self, index: usize, role: AccountRole) {
        let Rig { sessions, keys, .. } = self;
        let key = keys[index];
        let mut session = sessions.remove(&key).unwrap();
        session.gateway.set_role(role);
        sessions.insert(key, session);
    }

    fn snapshot(&self, index: usize) -> osfr_tools::game_store::Snapshot {
        let key = self.keys[index];
        self.sessions[&key].gateway.snapshot()
    }

    fn save(&self, index: usize) -> Vec<u8> {
        fs::read(&self.options[index].state).unwrap()
    }

    /// Keep one connected session out of the world: its sender stays registered so a bug that
    /// fans out to `session_keys` or `sessions` instead of `chat_sessions` would be caught.
    fn add_lurker(&mut self, guid: u64) {
        let id = 90 + guid;
        let options = options(&self.scratch, guid, false);
        let session = Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 41_000 + id).parse().unwrap(),
            SessionRequest {
                id: id as u32,
                crc_length: 3,
                buffer_size: 512,
            },
            &options,
            CRC_SEED,
            None,
            None,
        )
        .unwrap();
        let key = SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        };
        self.world.session_keys.insert(session.id, key);
        self.sessions.insert(key, session);
        self.keys.push(key);
        self.options.push(options);
    }
}

/// One 15/3 `ChatPacketDebugChat` message inside the 5/1 application tunnel.
fn system_text(message: &[u8]) -> String {
    assert_eq!(&message[..3], &[5, 0, 1], "not an application message");
    let inner = &message[7..];
    assert_eq!(&inner[..4], &[15, 0, 3, 0], "not a 15/3 system line");
    let length = u32::from_le_bytes(inner[4..8].try_into().unwrap()) as usize;
    assert_eq!(inner.len(), 8 + length + 1);
    String::from_utf8(inner[8..8 + length].to_vec()).unwrap()
}

fn system_texts(messages: &[Vec<u8>]) -> Vec<String> {
    messages
        .iter()
        .filter(|message| message.len() > 10 && &message[7..11] == [15, 0, 3, 0])
        .map(|message| system_text(message))
        .collect()
}

fn echoes_command(messages: &[Vec<u8>], text: &str) -> bool {
    messages.iter().any(|message| {
        message.len() > 10
            && message[7] == 15
            && message
                .windows(text.len())
                .any(|window| window == text.as_bytes())
    })
}

fn broadcast(text: &str) -> Vec<u8> {
    osfr_tools::chat::local_request(&format!("realm broadcast {text}"))
}

/// One combined SOE application bundle (`0/25`) of wrapped child messages.
fn bundle(messages: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0, 25];
    for message in messages {
        let child = wrap(message);
        bytes.push(child.len() as u8);
        bytes.extend(child);
    }
    bytes
}

#[test]
fn admin_broadcast_reaches_every_online_session_once() {
    let mut rig = Rig::new("all", &[(1, true), (2, false), (3, false)]);
    let text = "Scavenger hunt starts at noon";
    rig.send(0, &broadcast(text), NOW);
    for (index, delivery) in rig.drain_all(NOW).into_iter().enumerate() {
        assert_eq!(
            system_texts(&delivery),
            vec![text.to_owned()],
            "session {index}"
        );
        assert!(
            !echoes_command(&delivery, "realm broadcast"),
            "session {index}"
        );
    }
}

#[test]
fn player_and_builder_broadcasts_are_refused_without_fanout() {
    let mut rig = Rig::new("refused", &[(1, true), (2, false)]);
    let before = (rig.snapshot(0), rig.save(0));
    for role in [AccountRole::Builder, AccountRole::Player] {
        rig.set_role(0, role);
        rig.send(0, &broadcast("refused notice"), NOW);
        let deliveries = rig.drain_all(NOW);
        let refusal = system_texts(&deliveries[0]);
        assert_eq!(refusal.len(), 1, "{role:?}: {refusal:?}");
        assert!(
            refusal[0].contains("Cannot do that"),
            "{role:?}: {refusal:?}"
        );
        assert!(
            system_texts(&deliveries[1]).is_empty(),
            "{role:?}: a witness received a refused notice"
        );
        assert!(
            !echoes_command(&deliveries[0], "realm broadcast"),
            "{role:?}"
        );
    }
    assert_eq!(
        rig.snapshot(0),
        before.0,
        "a refusal changed character state"
    );
    assert_eq!(rig.save(0), before.1, "a refusal wrote a save");
}

#[test]
fn announcement_does_not_change_character_state_or_save() {
    let mut rig = Rig::new("save", &[(1, true), (2, false)]);
    let before = (rig.snapshot(0), rig.save(0));
    rig.send(0, &broadcast("No state change"), NOW);
    rig.drain_all(NOW);
    assert_eq!(
        rig.snapshot(0),
        before.0,
        "the announcement changed gameplay"
    );
    assert_eq!(rig.save(0), before.1, "the announcement wrote a save");
}

#[test]
fn two_admins_each_get_one_announcement() {
    let mut rig = Rig::new("two", &[(1, true), (2, true), (3, false)]);
    for (index, text) in [(0, "First notice"), (1, "Second notice")] {
        rig.send(index, &broadcast(text), NOW);
        for (recipient, delivery) in rig.drain_all(NOW).into_iter().enumerate() {
            assert_eq!(
                system_texts(&delivery),
                vec![text.to_owned()],
                "command {index}, session {recipient}"
            );
        }
    }
}

#[test]
fn no_online_sessions_is_a_no_op() {
    let mut rig = Rig::new("solo", &[(1, true)]);
    rig.add_lurker(2);
    let before = rig.save(0);
    rig.send(0, &broadcast("Only the admin is online"), NOW);
    let deliveries = rig.drain_all(NOW);
    assert_eq!(
        system_texts(&deliveries[0]),
        vec!["Only the admin is online".to_owned()]
    );
    assert!(
        system_texts(&deliveries[1]).is_empty(),
        "a session that never entered the world received the notice"
    );
    assert_eq!(rig.save(0), before);
}

#[test]
fn invalid_text_is_refused_without_a_save_or_a_witness_notice() {
    let mut rig = Rig::new("invalid", &[(1, true), (2, false)]);
    let before = (rig.snapshot(0), rig.save(0));
    let long = "a".repeat(201);
    let cases = [
        ("realm broadcast".to_owned(), "needs a message", None),
        ("realm broadcast    ".to_owned(), "needs a message", None),
        (
            "realm broadcast line\r\nbreak".to_owned(),
            "control characters",
            Some("line\r\nbreak"),
        ),
        (
            "realm broadcast line\tbreak".to_owned(),
            "control characters",
            Some("line\tbreak"),
        ),
        (
            format!("realm broadcast {long}"),
            "200 characters maximum",
            Some(long.as_str()),
        ),
    ];
    for (request, expected, payload) in cases {
        rig.send(0, &osfr_tools::chat::local_request(&request), NOW);
        let deliveries = rig.drain_all(NOW);
        let refusal = system_texts(&deliveries[0]);
        assert_eq!(refusal.len(), 1, "{request:?}: {refusal:?}");
        assert!(refusal[0].contains(expected), "{request:?}: {refusal:?}");
        if let Some(payload) = payload {
            assert!(
                !refusal[0].contains(payload),
                "{request:?}: the refusal echoed the text: {refusal:?}"
            );
        }
        assert!(
            system_texts(&deliveries[1]).is_empty(),
            "{request:?}: a witness received a refused notice"
        );
    }
    assert_eq!(
        rig.snapshot(0),
        before.0,
        "an invalid notice changed character state"
    );
    assert_eq!(rig.save(0), before.1, "an invalid notice wrote a save");
}

/// A combined bundle applies its children in order: the second child has no notice of its
/// own and must not clear the one the first child staged.
#[test]
fn a_bundle_that_broadcasts_then_helps_keeps_the_notice() {
    let mut rig = Rig::new("bundle", &[(1, true), (2, false)]);
    let text = "Hunt at noon";
    rig.try_send_raw(
        0,
        &bundle(&[
            broadcast(text),
            osfr_tools::chat::local_request("realm help"),
        ]),
        NOW,
    )
    .unwrap();
    let deliveries = rig.drain_all(NOW);
    let sender = system_texts(&deliveries[0]);
    assert!(
        sender.contains(&text.to_owned()),
        "the bundle lost the notice: {sender:?}"
    );
    assert!(
        sender.iter().any(|line| line.contains("Opening:")),
        "the help reply is missing: {sender:?}"
    );
    assert_eq!(system_texts(&deliveries[1]), vec![text.to_owned()]);
}

#[test]
fn a_failed_save_never_announces() {
    let mut rig = Rig::new("failsave", &[(1, true), (2, false)]);
    let before = rig.save(0);
    rig.break_save(0);
    // Past POSITION_SAVE_MS so the teleport's pose save is not deferred: it runs inside the
    // transaction and fails, which must drop the notice staged beside it.
    let now = NOW + 10_000;
    let result = rig.try_send_raw(
        0,
        &bundle(&[
            broadcast("Should never leave the server"),
            osfr_tools::chat::local_request("realm goto 1 2 3"),
        ]),
        now,
    );
    assert!(result.is_err(), "the broken save must fail the transaction");
    for (index, delivery) in rig.drain_all(now).into_iter().enumerate() {
        assert!(
            system_texts(&delivery).is_empty(),
            "session {index} received a notice from a failed save: {delivery:?}"
        );
    }
    assert_eq!(rig.save(0), before, "a failed transaction changed the save");
}

#[test]
fn a_replayed_reliable_packet_announces_once() {
    let mut rig = Rig::new("replay", &[(1, true), (2, false)]);
    let text = "Only once";
    let message = broadcast(text);
    // The same client sequence twice: the receiver drops the second datagram as a duplicate
    // before gameplay sees it, so the notice cannot fan out twice.
    rig.reliable(0, &message, 0, NOW);
    rig.reliable(0, &message, 0, NOW);
    for (index, delivery) in rig.drain_all(NOW).into_iter().enumerate() {
        assert_eq!(
            system_texts(&delivery),
            vec![text.to_owned()],
            "session {index}"
        );
    }
}

#[test]
fn maximum_length_escaped_text_reaches_each_recipient_byte_exactly() {
    let mut rig = Rig::new("maximum", &[(1, true), (2, false), (3, false)]);
    let text = "<&>".repeat(66) + "ab";
    assert_eq!(text.chars().count(), 200);
    rig.send(0, &broadcast(&text), NOW);
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    for (index, delivery) in rig.drain_all(NOW).into_iter().enumerate() {
        assert_eq!(
            system_texts(&delivery),
            vec![escaped.clone()],
            "session {index}"
        );
        let packet = delivery
            .iter()
            .find(|message| message.len() > 10 && &message[7..11] == [15, 0, 3, 0])
            .unwrap_or_else(|| panic!("session {index}: no 15/3 packet: {delivery:?}"));
        let inner = &packet[7..];
        assert_eq!(&inner[..4], &[15, 0, 3, 0], "session {index}");
        assert_eq!(
            u32::from_le_bytes(inner[4..8].try_into().unwrap()) as usize,
            escaped.len(),
            "session {index}"
        );
        assert_eq!(
            &inner[8..8 + escaped.len()],
            escaped.as_bytes(),
            "session {index}"
        );
        assert_eq!(inner.len(), 8 + escaped.len() + 1, "session {index}");
        assert_eq!(inner[8 + escaped.len()], 1, "session {index}");
    }
}
