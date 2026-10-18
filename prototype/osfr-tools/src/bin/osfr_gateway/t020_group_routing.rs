use super::*;
use osfr_tools::players::PlayerPresentation;
use osfr_tools::social_chat::{ChatChannel, PacketChat};

const GUIDS: [u64; 3] = [301, 302, 303];
const NAMES: [&str; 3] = ["P301", "P302", "P303"];

struct Rig {
    sessions: [Session; 3],
    world: SharedWorld,
    scratch: PathBuf,
    options: Options,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

fn options(scratch: &Path) -> Options {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state: scratch.join("character.json"),
        content: Some(root.join("references/rust-opening-cooking-original-assets")),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        world: Some(scratch.join("world")),
        stock_text: None,
        allow_lan: false,
        security_log: None,
    }
}

fn sessions(options: &Options) -> [Session; 3] {
    std::array::from_fn(|index| {
        let id = 17 + index as u64;
        let mut session = Session::new(
            SessionId(id),
            "127.0.0.1:12345".parse().unwrap(),
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
        session.character_guid = GUIDS[index];
        session.account_guid = GUIDS[index] + 10_000;
        session
    })
}

fn session_key(session: &Session) -> SessionKey {
    SessionKey {
        peer: session.peer,
        connection_id: session.request.id,
    }
}

fn seed_world(world: &mut SharedWorld, sessions: &[Session; 3]) {
    for (index, session) in sessions.iter().enumerate() {
        let guid = GUIDS[index];
        let first = NAMES[index];
        world
            .players
            .join_with_presentation(
                session.id,
                guid + 10_000,
                guid,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [index as f32 * 5.0, 0.0, 0.0],
                0.0,
                PlayerPresentation {
                    character_guid: guid,
                    first_name: first.to_owned(),
                    ..PlayerPresentation::default()
                },
            )
            .unwrap();
        let identity = SocialIdentity {
            account_id: guid + 10_000,
            character_guid: guid,
            name: SocialName::new(first, "").unwrap(),
        };
        world.identities.insert(guid, identity.clone());
        world
            .social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        world.social.login(guid).unwrap();
        world.active_sessions.insert(guid, session.id);
        world.chat_sessions.insert(
            session.id,
            ChatSession {
                session_id: session.id.0,
                character_guid: guid,
                name: NameData {
                    fields: [0; 3],
                    first: first.to_owned(),
                    last: String::new(),
                },
                zone_id: 1,
                position: [index as f32 * 5.0, 0.0, 0.0],
                chat_area_id: None,
                group_id: None,
                guild_id: None,
                is_member: true,
            },
        );
        world.chat_settings.join(guid, None);
        world.joined.insert(session.id);
        world.session_keys.insert(session.id, session_key(session));
    }
}

fn rig(name: &str) -> Rig {
    let scratch = std::env::temp_dir().join(format!("osfr-t020-{name}-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let options = options(&scratch);
    let sessions = sessions(&options);
    let mut world = SharedWorld::new();
    seed_world(&mut world, &sessions);
    Rig {
        sessions,
        world,
        scratch,
        options,
    }
}

pub(super) fn server_rig(name: &str) -> (Server, [SessionKey; 3], PathBuf) {
    let scratch = std::env::temp_dir().join(format!("osfr-t020-{name}-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let mut server = Server::new(options(&scratch)).unwrap();
    let sessions = sessions(&server.options);
    let keys = std::array::from_fn(|index| session_key(&sessions[index]));
    seed_world(&mut server.world, &sessions);
    for (key, session) in keys.into_iter().zip(sessions) {
        server.sessions.insert(key, session);
    }
    (server, keys, scratch)
}

fn send(rig: &mut Rig, index: usize, packet: &[u8]) -> Vec<Delivery> {
    let deliveries = rig.sessions[index].route_application(packet, &mut rig.world, 1_000);
    // Production calls this after every routed application message; group chat reads the
    // same canonical membership from the chat session.
    rig.world.refresh_group_ids();
    deliveries
}

fn route_server(server: &mut Server, key: SessionKey, packet: &[u8]) -> Vec<Delivery> {
    let mut session = server.sessions.remove(&key).unwrap();
    let deliveries = session.route_application(packet, &mut server.world, 1_000);
    server.sessions.insert(key, session);
    server.world.refresh_group_ids();
    deliveries
}

fn inner(delivery: &Delivery) -> &[u8] {
    assert_eq!(&delivery.bytes[..3], &[5, 0, 1]);
    &delivery.bytes[7..]
}

fn put_string(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as i32).to_le_bytes());
    out.extend_from_slice(value.as_bytes());
}

fn put_name(out: &mut Vec<u8>, index: usize) {
    for _ in 0..3 {
        out.extend_from_slice(&0i32.to_le_bytes());
    }
    put_string(out, NAMES[index]);
    put_string(out, "");
}

fn group_header(sub: u16) -> Vec<u8> {
    let mut out = 40u16.to_le_bytes().to_vec();
    out.extend_from_slice(&sub.to_le_bytes());
    out
}

fn invite(from: usize, to: usize, group_id: u32) -> Vec<u8> {
    let mut out = group_header(1);
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&GUIDS[from].to_le_bytes());
    put_name(&mut out, from);
    out.extend_from_slice(&GUIDS[to].to_le_bytes());
    put_name(&mut out, to);
    out.extend_from_slice(&group_id.to_le_bytes());
    out.extend_from_slice(
        &[0u32; 5]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    out
}

fn response(from: usize, to: usize, group_id: u32, accepted: bool) -> Vec<u8> {
    let mut out = group_header(4);
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&GUIDS[from].to_le_bytes());
    put_name(&mut out, from);
    out.extend_from_slice(&GUIDS[to].to_le_bytes());
    put_name(&mut out, to);
    out.extend_from_slice(&group_id.to_le_bytes());
    out.push(u8::from(accepted));
    out.extend_from_slice(
        &[0u32; 5]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>(),
    );
    out
}

fn leave(index: usize) -> Vec<u8> {
    let mut out = group_header(3);
    out.extend_from_slice(&GUIDS[index].to_le_bytes());
    put_name(&mut out, index);
    out
}

fn kick(index: usize) -> Vec<u8> {
    let mut out = group_header(6);
    out.extend_from_slice(&GUIDS[index].to_le_bytes());
    put_name(&mut out, index);
    out
}

fn chat(channel: ChatChannel) -> Vec<u8> {
    social_chat::encode_packet_chat(&PacketChat {
        channel,
        sender_guid: 0,
        target_guid: 0,
        sender_name: NameData::default(),
        target_name: NameData::default(),
        message: "synthetic line".to_owned(),
        position: [0.0; 4],
        guild_guid: 0,
        language: 0,
        area_id: None,
    })
    .unwrap()
}

fn family_sub(delivery: &Delivery) -> (u16, u16) {
    let bytes = inner(delivery);
    (
        u16::from_le_bytes([bytes[0], bytes[1]]),
        u16::from_le_bytes([bytes[2], bytes[3]]),
    )
}

fn has(deliveries: &[Delivery], index: usize, family: u16, sub: u16) -> bool {
    deliveries.iter().any(|delivery| {
        delivery.recipient == SessionId(17 + index as u64) && family_sub(delivery) == (family, sub)
    })
}

fn result_for(deliveries: &[Delivery], index: usize, sub: u16) -> Option<u32> {
    deliveries
        .iter()
        .find(|delivery| {
            delivery.recipient == SessionId(17 + index as u64) && family_sub(delivery) == (40, sub)
        })
        .map(|delivery| u32::from_le_bytes(inner(delivery)[4..8].try_into().unwrap()))
}

fn recipients(deliveries: &[Delivery]) -> Vec<u64> {
    let mut ids: Vec<u64> = deliveries.iter().map(|d| d.recipient.0).collect();
    ids.sort();
    ids
}

#[test]
fn two_sessions_form_chat_and_leave_a_party_and_refusals_change_nothing() {
    let mut rig = rig("party");
    let a = 0;
    let b = 1;
    let c = 2;

    // A singleton cannot invite itself: no solo group and no pending invite may appear.
    let refused = send(&mut rig, c, &invite(c, c, 0));
    assert_eq!(result_for(&refused, c, 2), Some(3));
    assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
    assert_eq!(rig.world.social.group_invitee(GUIDS[c]), None);

    // 40/1 creates the transient leader row (40/8), invites the target, and answers 40/2=0.
    let invited = send(&mut rig, a, &invite(a, b, 0));
    assert!(has(&invited, a, 40, 8));
    assert_eq!(result_for(&invited, a, 2), Some(0));
    assert!(has(&invited, b, 40, 1));
    let group_id = rig.world.social.group_of(GUIDS[a]).unwrap();
    let pending = rig.world.social.group_invitee(GUIDS[b]);
    assert_eq!(pending, Some((group_id, GUIDS[a])));

    // Repeats and cross-groups refuse against unchanged membership.
    let repeated = send(&mut rig, a, &invite(a, b, group_id));
    assert_eq!(result_for(&repeated, a, 2), Some(6));
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a]].as_slice())
    );
    assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

    let cross = send(&mut rig, c, &invite(c, b, 0));
    assert_eq!(result_for(&cross, c, 2), Some(4));
    assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a]].as_slice())
    );
    assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

    let self_while_grouped = send(&mut rig, a, &invite(a, a, group_id));
    assert_eq!(result_for(&self_while_grouped, a, 2), Some(3));
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a]].as_slice())
    );
    assert_eq!(rig.world.social.group_invitee(GUIDS[b]), pending);

    // 40/4 accept fans out the invitation notice, member row and full roster.
    let accepted = send(&mut rig, b, &response(a, b, group_id, true));
    assert!(has(&accepted, a, 40, 4));
    assert!(has(&accepted, a, 40, 9));
    assert!(has(&accepted, a, 40, 8));
    assert!(has(&accepted, b, 40, 4));
    assert!(has(&accepted, b, 40, 9));
    assert!(has(&accepted, b, 40, 8));
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a], GUIDS[b]].as_slice())
    );

    // B's channel-4 chat reaches A and echoes to B; C is not a member and hears nothing.
    let party_chat = send(&mut rig, b, &chat(ChatChannel::Group));
    assert_eq!(
        recipients(&party_chat),
        vec![rig.sessions[a].id.0, rig.sessions[b].id.0]
    );
    let line = social_chat::decode_packet_chat(inner(
        party_chat
            .iter()
            .find(|delivery| delivery.recipient == rig.sessions[a].id)
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(line.sender_guid, GUIDS[b]);
    assert_eq!(line.sender_name.first, NAMES[b]);
    assert_eq!(line.channel, ChatChannel::Group);
    assert!(send(&mut rig, c, &chat(ChatChannel::Group)).is_empty());

    // Add C, then prove a non-leader cannot kick and the leader can.
    send(&mut rig, a, &invite(a, c, group_id));
    let accepted_c = send(&mut rig, c, &response(a, c, group_id, true));
    assert!(has(&accepted_c, a, 40, 8));
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a], GUIDS[b], GUIDS[c]].as_slice())
    );

    let refused_kick = send(&mut rig, b, &kick(a));
    assert_eq!(result_for(&refused_kick, b, 7), Some(2));
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a], GUIDS[b], GUIDS[c]].as_slice())
    );

    let kicked = send(&mut rig, a, &kick(c));
    assert!(has(&kicked, a, 40, 6));
    assert!(has(&kicked, b, 40, 6));
    assert!(has(&kicked, c, 40, 6));
    assert!(has(&kicked, a, 40, 8));
    assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[a], GUIDS[b]].as_slice())
    );
    assert_eq!(rig.world.chat_sessions[&rig.sessions[c].id].group_id, None);
    rig.sessions[c].spam_gate = SpamGate::default();
    assert!(send(&mut rig, c, &chat(ChatChannel::Group)).is_empty());

    // 40/3 leaves; with one member left the volatile party dissolves for both players.
    let left = send(&mut rig, b, &leave(b));
    assert!(has(&left, a, 40, 3));
    assert!(has(&left, b, 40, 3));
    assert_eq!(rig.world.social.group_of(GUIDS[a]), None);
    assert_eq!(rig.world.social.group_of(GUIDS[b]), None);
    assert!(send(&mut rig, a, &chat(ChatChannel::Group)).is_empty());
}

#[test]
fn interleaved_invites_cannot_overlap_membership_or_group_chat() {
    let mut rig = rig("interleaved");
    let (a, b, c) = (0, 1, 2);

    // A invited B, but B created a different party and C joined it first.
    send(&mut rig, a, &invite(a, b, 0));
    let a_group = rig.world.social.group_of(GUIDS[a]).unwrap();
    send(&mut rig, b, &invite(b, c, 0));
    let b_group = rig.world.social.group_of(GUIDS[b]).unwrap();
    assert_ne!(a_group, b_group);
    send(&mut rig, c, &response(b, c, b_group, true));

    // B's stale acceptance of A's invite must leave both parties untouched.
    let refused = send(&mut rig, b, &response(a, b, a_group, true));
    assert_eq!(result_for(&refused, b, 5), Some(1));
    assert_eq!(rig.world.social.group_of(GUIDS[a]), Some(a_group));
    assert_eq!(rig.world.social.group_of(GUIDS[b]), Some(b_group));
    assert_eq!(rig.world.social.group_of(GUIDS[c]), Some(b_group));
    assert_eq!(
        rig.world.social.group_members(a_group),
        Some([GUIDS[a]].as_slice())
    );
    assert_eq!(
        rig.world.social.group_members(b_group),
        Some([GUIDS[b], GUIDS[c]].as_slice())
    );
    assert_eq!(
        rig.world.social.group_invitee(GUIDS[b]),
        Some((a_group, GUIDS[a]))
    );

    // Chat follows the same canonical mappings as both rosters.
    let b_chat = send(&mut rig, b, &chat(ChatChannel::Group));
    assert_eq!(
        recipients(&b_chat),
        vec![rig.sessions[b].id.0, rig.sessions[c].id.0]
    );
    let a_chat = send(&mut rig, a, &chat(ChatChannel::Group));
    assert_eq!(recipients(&a_chat), vec![rig.sessions[a].id.0]);

    // C leaving dissolves B's two-member party, not A's unrelated party.
    send(&mut rig, c, &leave(c));
    assert_eq!(rig.world.social.group_of(GUIDS[a]), Some(a_group));
    assert_eq!(rig.world.social.group_of(GUIDS[b]), None);
    assert_eq!(rig.world.social.group_of(GUIDS[c]), None);
    assert_eq!(
        rig.world.social.group_members(a_group),
        Some([GUIDS[a]].as_slice())
    );
    assert_eq!(
        rig.world.social.group_invitee(GUIDS[b]),
        Some((a_group, GUIDS[a]))
    );
    assert_eq!(
        rig.world.chat_sessions[&rig.sessions[a].id].group_id,
        Some(u64::from(a_group))
    );
    assert_eq!(rig.world.chat_sessions[&rig.sessions[b].id].group_id, None);
    assert_eq!(rig.world.chat_sessions[&rig.sessions[c].id].group_id, None);
    rig.sessions[b].spam_gate = SpamGate::default();
    assert!(send(&mut rig, b, &chat(ChatChannel::Group)).is_empty());
}

#[test]
fn gateway_tick_expires_an_idle_invite() {
    let mut rig = rig("tick-expiry");
    send(&mut rig, 0, &invite(0, 1, 0));
    let group_id = rig.world.social.group_of(GUIDS[0]).unwrap();
    assert_eq!(
        rig.world.social.group_invitee(GUIDS[1]),
        Some((group_id, GUIDS[0]))
    );

    let mut membership = World::runtime();
    membership
        .register_runtime_zone(osfr_tools::content::Zone {
            id: 1,
            name: String::new(),
            geometry_id: Some(1),
            zone_type: Some(2),
            tutorial: Some(false),
            sky: Some(String::new()),
            source: Vec::new(),
        })
        .unwrap();
    let mut sessions = SessionMap::new();
    rig.sessions[0]
        .tick(
            &rig.options,
            61_000,
            &mut rig.world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();

    assert_eq!(rig.world.social.group_invitee(GUIDS[1]), None);
    assert_eq!(
        rig.world.social.group_members(group_id),
        Some([GUIDS[0]].as_slice())
    );
}

#[test]
fn disconnect_dissolves_the_party_and_clears_party_chat_for_the_survivor() {
    let mut rig = rig("disconnect");
    let (a, b) = (0, 1);
    send(&mut rig, a, &invite(a, b, 0));
    let group_id = rig.world.social.group_of(GUIDS[a]).unwrap();
    send(&mut rig, b, &response(a, b, group_id, true));
    assert_eq!(
        rig.world.chat_sessions[&rig.sessions[a].id].group_id,
        Some(u64::from(group_id))
    );
    assert_eq!(
        rig.world.chat_sessions[&rig.sessions[b].id].group_id,
        Some(u64::from(group_id))
    );

    // This is the same order as `Server::end_session`: remove the player/chat row, then
    // disconnect the social state. The survivor must lose the dissolved channel too.
    rig.world.leave_player(rig.sessions[b].id);
    let delivered = rig.world.social_disconnect(GUIDS[b]).unwrap();
    assert!(delivered.iter().any(
        |delivery| delivery.recipient == rig.sessions[a].id && family_sub(delivery) == (40, 3)
    ));
    assert_eq!(rig.world.social.group_of(GUIDS[a]), None);
    assert_eq!(rig.world.chat_sessions[&rig.sessions[a].id].group_id, None);
    assert!(send(&mut rig, a, &chat(ChatChannel::Group)).is_empty());
}

#[test]
fn end_session_of_the_owner_dissolves_the_party_and_clears_survivor_chat() {
    let (mut server, keys, scratch) = server_rig("end-session-owner");
    let (a, b) = (0, 1);
    route_server(&mut server, keys[a], &invite(a, b, 0));
    let group_id = server.world.social.group_of(GUIDS[a]).unwrap();
    route_server(&mut server, keys[b], &response(a, b, group_id, true));
    assert_eq!(
        server.world.chat_sessions[&server.sessions[&keys[a]].id].group_id,
        Some(u64::from(group_id))
    );

    let session_b = server.sessions.remove(&keys[b]).unwrap();
    server.end_session(keys[b], session_b, "test_owner");

    assert_eq!(server.world.social.group_of(GUIDS[a]), None);
    assert_eq!(server.world.social.group_of(GUIDS[b]), None);
    assert_eq!(server.world.social.group_members(group_id), None);
    assert_eq!(
        server.world.chat_sessions[&server.sessions[&keys[a]].id].group_id,
        None
    );
    assert!(!server.world.active_sessions.contains_key(&GUIDS[b]));
    assert!(route_server(&mut server, keys[a], &chat(ChatChannel::Group)).is_empty());

    drop(server);
    fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn stale_non_owning_end_session_preserves_the_live_party_and_chat() {
    let (mut server, keys, scratch) = server_rig("end-session-stale");
    let (a, b) = (0, 1);
    route_server(&mut server, keys[a], &invite(a, b, 0));
    let group_id = server.world.social.group_of(GUIDS[a]).unwrap();
    route_server(&mut server, keys[b], &response(a, b, group_id, true));

    let live_b_id = server.sessions[&keys[b]].id;
    let mut stale = Session::new(
        SessionId(40),
        "127.0.0.1:12348".parse().unwrap(),
        SessionRequest {
            id: 40,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    stale.character_guid = GUIDS[b];
    stale.account_guid = GUIDS[b] + 10_000;
    let stale_key = session_key(&stale);
    server.world.session_keys.insert(stale.id, stale_key);
    server.sessions.insert(stale_key, stale);

    let memberships = GUIDS.map(|guid| server.world.social.group_of(guid));
    let roster = server
        .world
        .social
        .group_members(group_id)
        .unwrap()
        .to_vec();
    let chat_groups =
        [a, b].map(|index| server.world.chat_sessions[&server.sessions[&keys[index]].id].group_id);

    let stale = server.sessions.remove(&stale_key).unwrap();
    server.end_session(stale_key, stale, "duplicate_session");

    assert!(server.sessions.contains_key(&keys[b]));
    assert_eq!(server.world.active_sessions[&GUIDS[b]], live_b_id);
    assert_eq!(
        GUIDS.map(|guid| server.world.social.group_of(guid)),
        memberships
    );
    assert_eq!(
        server.world.social.group_members(group_id),
        Some(roster.as_slice())
    );
    assert_eq!(
        [a, b].map(|index| {
            server.world.chat_sessions[&server.sessions[&keys[index]].id].group_id
        }),
        chat_groups
    );
    let party_chat = route_server(&mut server, keys[a], &chat(ChatChannel::Group));
    assert_eq!(
        recipients(&party_chat),
        vec![server.sessions[&keys[a]].id.0, live_b_id.0]
    );

    drop(server);
    fs::remove_dir_all(scratch).unwrap();
}
