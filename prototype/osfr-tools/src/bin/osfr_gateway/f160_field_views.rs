//! F160: shared-field view reconciliation around readiness/world-load loss, and
//! the per-recipient delivery isolation that keeps one full queue out of the tick.
use super::*;
use osfr_tools::quest_engine::{QuestProgress, QuestStatus};
use serde_json::Value;

impl Server {
    /// Set only the server-owned membership readiness flag.
    pub fn test_set_ready(&mut self, guid: u64, ready: bool) {
        let location = self.membership.player_location(guid).unwrap().clone();
        let zone_id = self
            .membership
            .instance(location.instance_id)
            .unwrap()
            .zone_id;
        let change = self
            .membership
            .prepare_membership(
                guid,
                zone_id,
                Some(location.instance_id),
                location.pose,
                ready,
                self.start.elapsed().as_millis() as u64,
            )
            .unwrap();
        self.membership.commit_membership(change);
    }

    /// World-load loss alone: the loader reset the session while the character
    /// stays assigned to and ready in its instance.
    pub fn test_lose_world(&mut self, guid: u64) {
        let id = self.world.active_sessions[&guid];
        let key = self.world.session_keys[&id];
        self.sessions.get_mut(&key).unwrap().gateway.reset_session();
        self.test_set_ready(guid, true);
    }
}

fn sender(max_queued_bytes: usize) -> ReliableSender {
    ReliableSender::new(
        Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
        SenderConfig {
            datagram_limit: 512,
            window: 32,
            max_message: MAX_MESSAGE,
            max_queued_bytes,
            retry_ms: 250,
            max_retries: 40,
        },
    )
    .unwrap()
}

/// A full reliable queue must refuse only that viewer: the shared field keeps
/// ticking, the other viewer's removal is delivered, and a later tick retries.
#[test]
fn f160_a_full_reliable_queue_refuses_only_its_own_view() {
    if !super::t020_group_routing::fixtures_present() {
        eprintln!("skip f160-view: the private lab fixtures are absent");
        return;
    }
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("f160-view");
    let known = [131000108001_u64, 131000108002];
    // Both players are in field instance 0 and not ready: the F132 removal
    // window. Only the first still knows the field actors.
    for (index, guid) in [301_u64, 302].into_iter().enumerate() {
        let change = server
            .membership
            .prepare_membership(guid, 0, Some(0), WorldPose::ORIGIN, false, 0)
            .unwrap();
        server.membership.commit_membership(change);
        server
            .sessions
            .get_mut(&keys[index])
            .unwrap()
            .known_npcs
            .extend(known);
    }
    // The first viewer cannot accept any reliable frame.
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(1);
    let before = server.sessions[&keys[0]].known_npcs.clone();
    server.tick_shared_fields(1_000);
    assert!(
        server.faulted_fields.is_empty(),
        "one full queue must not fault the shared field"
    );
    assert_eq!(
        server.sessions[&keys[0]].known_npcs, before,
        "a refused view delivery changes nothing"
    );
    assert!(
        server.sessions[&keys[1]].known_npcs.is_empty(),
        "the other viewer is still reconciled"
    );
    assert!(
        !server.sessions[&keys[1]].sender.is_idle(),
        "the other viewer's removal was queued"
    );
    // The next tick retries the refused viewer once its queue can accept.
    server.sessions.get_mut(&keys[0]).unwrap().sender = sender(32 * 1024 * 1024);
    server.tick_shared_fields(2_000);
    assert!(server.sessions[&keys[0]].known_npcs.is_empty());
    // The whole event loop still runs with both sessions alive. Refresh the
    // idle budget first (as `isolation` does): a loaded host can stall this test
    // for the 60 s session budget, and the idle timeout would then end a
    // session instead of the event loop being exercised.
    for session in server.sessions.values_mut() {
        session.last_activity = std::time::Instant::now();
    }
    server.step().unwrap();
    assert_eq!(server.sessions.len(), 3);
    let _ = fs::remove_dir_all(scratch);
}

const F213_JONES: u64 = 9_000_000_121;
const F213_REFERRAL_TO_JONES: u32 = 3_030_006;

fn f213_login(guid: u64, version: &str, ticket: &str) -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (index, text) in [ticket, version, ""].iter().enumerate() {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if index == 0 {
            bytes.extend(guid.to_le_bytes());
        }
    }
    bytes
}

fn f213_npc_position(npc_id: u64) -> [f32; 3] {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/content/npcs/cobblestone_village.json");
    let document: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let npc = document["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["npc_id"].as_u64() == Some(npc_id))
        .expect("the chapter NPC exists");
    serde_json::from_value(npc["position"].clone()).expect("the chapter NPC has a position")
}

fn f213_sender() -> ReliableSender {
    ReliableSender::new(
        Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap(),
        SenderConfig {
            datagram_limit: 60_000,
            window: 4_096,
            max_message: MAX_MESSAGE,
            max_queued_bytes: 32 * 1024 * 1024,
            retry_ms: 250,
            max_retries: 40,
        },
    )
    .unwrap()
}

fn f213_application_body(message: &[u8]) -> Option<&[u8]> {
    if message.len() < 7 || message[..3] != [5, 0, 1] {
        return None;
    }
    let length = u32::from_le_bytes(message[3..7].try_into().unwrap()) as usize;
    (message.len() == length + 7).then_some(&message[7..])
}

fn f213_bodies(sender: &mut ReliableSender, now: u64) -> Vec<Vec<u8>> {
    let codec = Codec::new(CRC_SEED, 3, true, MAX_MESSAGE).unwrap();
    sender
        .poll(now)
        .unwrap()
        .iter()
        .filter_map(|wire| {
            let packet = codec.decode(wire).expect("the queued frame decodes");
            assert_eq!(packet.opcode, RELIABLE_DATA);
            // Reliable data: two-byte sequence, then the tunneled application body.
            f213_application_body(&packet.payload[2..]).map(<[u8]>::to_vec)
        })
        .collect()
}

/// F213: the shipped shared-field fan-out (`Server::sync_field_views`, reached from the
/// shared tick) must queue the saved opener's 35/2 before replaying the pending 49/13,
/// and the screen must name that runtime actor. The service-only regression cannot see
/// the `known` set, readiness and queueing that this path adds.
#[test]
fn f213_sync_field_views_replays_pending_end_screen_after_the_speaker_spawn() {
    // `server_rig` needs the private lab fixture pair. A clean checkout must not fail this
    // regression in setup (it reads as a product defect), so report a visible skip and
    // return; the test stays un-`#[ignore]`d, so it still runs wherever the fixtures exist.
    if !super::t020_group_routing::fixtures_present() {
        eprintln!("skip f213-turnin: the private lab fixtures are absent");
        return;
    }
    let (mut server, keys, scratch) = super::t020_group_routing::server_rig("f213-turnin");
    let now = 1_000;
    let guid = server.sessions[&keys[0]].character_guid;
    let content = server
        .options
        .content
        .clone()
        .expect("the field rig loads gameplay content");
    let manifest: Value =
        serde_json::from_slice(&fs::read(server.options.reference.join("manifest.json")).unwrap())
            .unwrap();
    let version = manifest["client_version"].as_str().unwrap().to_owned();
    let jones = f213_npc_position(F213_JONES);
    let mut character = server.sessions[&keys[0]].gateway.state().clone();
    character.zone_id = 0;
    character.position = jones;
    // The reference character GUID owns the saved gameplay; the session GUID owns membership.
    let login_guid = character.guid;
    let mut saved = server.sessions[&keys[0]]
        .gateway
        .gameplay()
        .expect("the session loads gameplay")
        .state();
    let quests = saved
        .npc_quests
        .as_mut()
        .expect("the bundled quest catalogue loads");
    quests.quests.insert(
        F213_REFERRAL_TO_JONES,
        QuestProgress {
            status: QuestStatus::Active,
            counts: vec![1],
        },
    );
    quests.pending_turnin = Some(F213_REFERRAL_TO_JONES);
    quests.pending_turnin_actor = Some(F213_JONES);
    // Production `load_gateway_from` takes the shared fields after loading gameplay.
    let mut gateway = Gateway::load_gameplay(
        &server.options.reference,
        character.clone(),
        &content,
        Some(saved),
    )
    .expect("the saved chapter loads");
    gateway.take_shared_fields();
    gateway
        .handle_at(&f213_login(login_guid, &version, "f213-replay"), now)
        .unwrap();
    gateway
        .handle_at(&wrap_application(&[13, 0]), now + 1)
        .unwrap();
    gateway
        .handle_at(&wrap_application(&[10, 0]), now + 2)
        .expect("the reconnect reaches the loaded event");
    let session = server.sessions.get_mut(&keys[0]).unwrap();
    session.gateway = gateway;
    session.sender = f213_sender();
    let pose = WorldPose {
        position: [jones[0], jones[1], jones[2], 1.0],
        rotation: [0.0, 0.0, 1.0, 0.0],
    };
    let change = server
        .membership
        .prepare_membership(guid, 0, Some(0), pose, true, now + 3)
        .expect("the shared scene accepts the viewer");
    server.membership.commit_membership(change);
    let field = server
        .membership
        .field(0)
        .expect("the shared field seed is installed")
        .clone();
    let before_sync = server.sessions[&keys[0]].gateway.snapshot();
    server
        .sync_field_views(0, &field, false)
        .expect("the shared field view syncs");
    assert_eq!(
        server.sessions[&keys[0]].gateway.snapshot(),
        before_sync,
        "replaying the saved end screen must not change saved state"
    );
    let bodies = f213_bodies(
        &mut server.sessions.get_mut(&keys[0]).unwrap().sender,
        now + 10,
    );
    let spawn = bodies
        .iter()
        .position(|body| {
            body.starts_with(&[35, 0, 2, 0])
                && body.len() >= 12
                && u64::from_le_bytes(body[4..12].try_into().unwrap()) == F213_JONES
        })
        .expect("the shared fan-out spawns the saved opener");
    let screen = bodies
        .iter()
        .position(|body| body.starts_with(&[49, 0, 13, 0]))
        .expect("the shared fan-out replays the saved end screen");
    assert!(
        spawn < screen,
        "the 49/13 replay must follow the opener's 35/2 in the queued batch"
    );
    let end = &bodies[screen];
    // Client 49/13 QuestEndPacket::ReadFields 0x00c7b990 keeps the actor at 6..14 and the
    // 49/14 reply quest id at 14..18; the 35/2 AddNpc reader 0x00921a10 reads 4..12.
    assert_eq!(
        u64::from_le_bytes(end[6..14].try_into().unwrap()),
        F213_JONES,
        "the replay names the runtime actor the client must resolve"
    );
    assert_eq!(
        u32::from_le_bytes(end[14..18].try_into().unwrap()),
        F213_REFERRAL_TO_JONES,
        "the replay keeps the quest id 49/14 sends back"
    );
    assert!(server.sessions[&keys[0]].known_npcs.contains(&F213_JONES));

    // Readiness gating: an unready viewer keeps its instance but takes the hidden view.
    // The saved 49/13 and the opener's 35/2 must not be queued; the known actor is removed.
    let change = server
        .membership
        .prepare_membership(guid, 0, Some(0), pose, false, now + 4)
        .expect("the shared scene accepts the unready viewer");
    server.membership.commit_membership(change);
    server
        .sync_field_views(0, &field, false)
        .expect("the unready view reconciles");
    assert!(
        !server.sessions[&keys[0]].known_npcs.contains(&F213_JONES),
        "an unready viewer must drop the saved opener"
    );
    assert_eq!(
        server.sessions[&keys[0]].gateway.snapshot(),
        before_sync,
        "the hidden view must not change saved state"
    );
    let bodies = f213_bodies(
        &mut server.sessions.get_mut(&keys[0]).unwrap().sender,
        now + 20,
    );
    assert!(
        !bodies.iter().any(|body| body.starts_with(&[49, 0, 13, 0])),
        "an unready viewer must not receive the saved end screen"
    );
    assert!(
        !bodies.iter().any(|body| {
            body.starts_with(&[35, 0, 2, 0])
                && body.len() >= 12
                && u64::from_le_bytes(body[4..12].try_into().unwrap()) == F213_JONES
        }),
        "an unready viewer must not receive the opener spawn"
    );
    assert!(
        bodies.iter().any(|body| {
            // Client removal reader 0x008d69f0: 4-byte prefix, 2 bytes, then the actor.
            body.starts_with(&[35, 0, 3, 0])
                && body.len() >= 14
                && u64::from_le_bytes(body[6..14].try_into().unwrap()) == F213_JONES
        }),
        "the hidden view removes the actor the viewer knew"
    );
    let _ = fs::remove_dir_all(scratch);
}
