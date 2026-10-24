use super::*;
use osfr_tools::players::PlayerPresentation;
use osfr_tools::progression_extras::{ClaimWindow, claim_request};
use osfr_tools::rewards::{RewardDisplay, RewardGrant};
use osfr_tools::social_chat::{BubbleStyle, ChatChannel, PacketChat};

const GUIDS: [u64; 3] = [301, 302, 303];

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

fn rig(name: &str) -> Rig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = std::env::temp_dir().join(format!("osfr-w173-{name}-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let options = Options {
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
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    };
    let session = |index: usize| {
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
    };
    let sessions = [session(0), session(1), session(2)];
    let mut world = SharedWorld::new();
    for (index, session) in sessions.iter().enumerate() {
        let guid = GUIDS[index];
        let x = index as f32 * 5.0;
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
                [x, 0.0, 0.0],
                0.0,
                PlayerPresentation {
                    character_guid: guid,
                    first_name: format!("P{guid}"),
                    ..PlayerPresentation::default()
                },
            )
            .unwrap();
        world.joined.insert(session.id);
        world.chat_sessions.insert(
            session.id,
            ChatMember {
                // Synthetic fixture names, not recovered player names.
                name: NameData {
                    fields: [0; 3],
                    first: format!("P{guid}"),
                    last: String::new(),
                },
                is_member: true,
            },
        );
        world.chat_settings.join(guid, None);
    }
    Rig {
        sessions,
        world,
        scratch,
        options,
    }
}

fn channel_packet(sub: u8, name: &str) -> Vec<u8> {
    let mut bytes = vec![26, 0, sub, 0];
    bytes.extend((name.len() as i32).to_le_bytes());
    bytes.extend(name.as_bytes());
    bytes
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}

fn log_in(session: &mut Session, ticket: &str) {
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in [ticket, "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
}

fn out_rows(session: &Session) -> usize {
    session
        .recent
        .iter()
        .filter(|row| row["dir"] == "out")
        .count()
}

fn dispatch(
    session: &mut Session,
    options: &Options,
    message: &[u8],
    now_ms: u64,
    world: &mut SharedWorld,
    membership: &mut World,
    sessions: &mut SessionMap,
) {
    session
        .dispatch_message(message, options, now_ms, world, membership, sessions, None)
        .unwrap();
}

fn bubble_packet(fg: i32, bg: i32, size: i32, guid: u64) -> Vec<u8> {
    let mut bytes = vec![26, 0, 18, 0];
    for value in [fg, bg, size] {
        bytes.extend(value.to_le_bytes());
    }
    bytes.extend(guid.to_le_bytes());
    bytes
}

fn chat(channel: ChatChannel, to: Option<&str>) -> Vec<u8> {
    social_chat::encode_packet_chat(&PacketChat {
        channel,
        sender_guid: 0,
        target_guid: 0,
        sender_name: NameData::default(),
        target_name: NameData {
            fields: [0; 3],
            first: to.unwrap_or_default().to_owned(),
            last: String::new(),
        },
        message: "synthetic line".to_owned(),
        position: [0.0; 4],
        guild_guid: 0,
        language: 0,
        area_id: None,
    })
    .unwrap()
}

fn recipients(deliveries: Vec<Delivery>) -> Vec<u64> {
    let mut ids: Vec<u64> = deliveries.iter().map(|d| d.recipient.0).collect();
    ids.sort();
    ids
}

/// Spend the rest of an account's action budget at a future instant (L006, L016). The budget
/// refills at 5 tokens/s against the real clock, and a later real-time charge then sees zero
/// elapsed time, so no token can refill however slowly a loaded machine runs the test.
fn drain_budget_later(world: &SharedWorld, account: u64) {
    let later = Instant::now() + std::time::Duration::from_secs(5);
    let mut security = world.security.lock().unwrap_or_else(|e| e.into_inner());
    while security
        .budgets
        .allow(account, &ClaimWindow::list_request(), later)
    {}
}

fn send(session: &mut Session, world: &mut SharedWorld, packet: &[u8]) -> Vec<Delivery> {
    session.route_application(packet, world, 1_000)
}

fn muted(world: &SharedWorld, index: usize, channel: ChatChannel) -> bool {
    world
        .chat_settings
        .get(GUIDS[index])
        .unwrap()
        .is_muted(channel)
}

#[test]
fn a_muted_channel_is_dropped_for_that_player_only() {
    let mut rig = rig("mute");
    let [a, b, c] = &mut rig.sessions;
    let world = &mut rig.world;
    // B switches Trade off and Area on; the client gets no reply to either.
    assert!(send(b, world, &channel_packet(33, "WorldTrade")).is_empty());
    assert!(send(b, world, &channel_packet(32, "WorldArea")).is_empty());
    assert!(muted(world, 1, ChatChannel::Trade));
    assert!(!muted(world, 0, ChatChannel::Trade));
    // C speaks on Trade: A and C hear it, B does not.
    let trade = send(c, world, &chat(ChatChannel::Trade, None));
    assert_eq!(recipients(trade), vec![17, 19]);
    // A tells B, and B's Say reaches everyone: neither channel can be switched off.
    let tell = send(a, world, &chat(ChatChannel::Tell, Some("P302")));
    assert_eq!(recipients(tell), vec![18]);
    let say = send(b, world, &chat(ChatChannel::Say, None));
    assert_eq!(recipients(say), vec![17, 18, 19]);
    // Switching Trade back on clears the mute.
    assert!(send(b, world, &channel_packet(32, "WorldTrade")).is_empty());
    assert!(!muted(world, 1, ChatChannel::Trade));
}

#[test]
fn forged_names_and_wild_values_change_nothing() {
    let mut rig = rig("forged");
    let [a, ..] = &mut rig.sessions;
    let world = &mut rig.world;
    let before = world.chat_settings.clone();
    for bad in [
        channel_packet(33, "WorldSay"),
        channel_packet(33, "GroupSay"),
        channel_packet(32, &"x".repeat(5_000)),
        bubble_packet(-1, 0, 1, 0),
        bubble_packet(0, 0x0100_0000, 1, 0),
        bubble_packet(0, 0, 99, 0),
        vec![26, 0, 18, 0, 1],
        vec![26, 0, 32, 0],
    ] {
        assert!(send(a, world, &bad).is_empty());
        assert_eq!(world.chat_settings, before);
    }
}

#[test]
fn a_bubble_colour_is_stored_for_the_sender_alone() {
    let mut rig = rig("bubble");
    let [_, b, _] = &mut rig.sessions;
    let world = &mut rig.world;
    // The wire guid slot is forged to name player A; it must be ignored.
    let forged = bubble_packet(0x112233, 0x445566, 3, GUIDS[0]);
    assert!(send(b, world, &forged).is_empty());
    let style = |guid| world.chat_settings.get(guid).unwrap().bubble();
    let chosen = BubbleStyle {
        foreground: 0x112233,
        background: 0x445566,
        size: 3,
    };
    assert_eq!(style(GUIDS[1]), chosen);
    assert_eq!(style(GUIDS[0]), BubbleStyle::default());
}

#[test]
fn claim_requests_and_bundled_children_are_budgeted_before_staging() {
    const CLAIM: u64 = 0xA11C_E5;
    let mut rig = rig("claim-budget");
    let session = &mut rig.sessions[0];
    log_in(session, "claim-budget");
    session
        .gateway
        .issue_claim(
            CLAIM,
            RewardGrant {
                coins: 1,
                items: Vec::new(),
                job_xp: Default::default(),
                sc: 0,
                display: RewardDisplay {
                    source_guid: 0,
                    player_guid: 1,
                    reward_kind: 0,
                    icon_id: 0,
                    name_id: 0,
                    rows: Vec::new(),
                },
            },
        )
        .unwrap();
    session.recent_on = true;
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    let recover = |session: &Session| {
        (
            session.gateway.snapshot(),
            fs::read(&session.state_path).ok(),
        )
    };
    let baseline = recover(session);
    for _ in 0..9 {
        session
            .dispatch_message(
                &wrap(&ClaimWindow::list_request()),
                &rig.options,
                1_000,
                &mut rig.world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();
    }
    session.recent.clear();
    drain_budget_later(&rig.world, session.account_guid);

    let mut bundle = vec![0, 25];
    for request in [
        wrap(&claim_request(0x5151, &[CLAIM]).unwrap()),
        wrap(&ClaimWindow::list_request()),
    ] {
        bundle.push(request.len() as u8);
        bundle.extend(request);
    }
    session
        .dispatch_message(
            &bundle,
            &rig.options,
            1_000,
            &mut rig.world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();

    assert_eq!(
        recover(session),
        baseline,
        "refusal changed gameplay or saved bytes"
    );
    assert!(
        session.recent.iter().all(|row| row["dir"] != "out"),
        "a refused bundle produced replies: {:?}",
        session.recent
    );
}

#[test]
fn combat_bundle_children_are_budgeted_once() {
    let mut rig = rig("combat-budget");
    let session = &mut rig.sessions[0];
    log_in(session, "combat-budget");

    let mut bundle = vec![0, 25];
    for _ in 0..2 {
        let mut request = vec![26, 0, 10, 0];
        request.extend(0u64.to_le_bytes());
        request.extend(0i32.to_le_bytes());
        let request = wrap(&request);
        bundle.push(request.len() as u8);
        bundle.extend(request);
    }
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    // Probe with an instant no later than the bundle's own charge, so wall-clock
    // time cannot refill the 5/s action budget between dispatch and this check.
    let before = Instant::now();
    session
        .dispatch_message(
            &bundle,
            &rig.options,
            1_000,
            &mut rig.world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();

    let now = before;
    let mut security = rig.world.security.lock().unwrap_or_else(|e| e.into_inner());
    let allowed = (0..10)
        .filter(|_| {
            security
                .budgets
                .allow(session.account_guid, &ClaimWindow::list_request(), now)
        })
        .count();
    assert_eq!(
        allowed, 8,
        "a combat bundle must charge each child once, not once per recursive dispatch"
    );
}

/// A bundle that mixes a free (non-budgeted) child with a budgeted child is
/// refused as a whole before staging: the free child must not run when its
/// sibling has no budget left. The free child is proven effective on its own.
#[test]
fn mixed_bundle_is_dropped_whole_when_its_budgeted_child_is_refused() {
    let mut rig = rig("mixed-budget");
    let session = &mut rig.sessions[0];
    log_in(session, "mixed-budget");
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();

    // 167/1/4 is a portal refresh: not in the budget families, and it replies
    // when it runs.
    session.recent_on = true;
    dispatch(
        session,
        &rig.options,
        &wrap(&[167, 0, 1, 4]),
        1_000,
        &mut rig.world,
        &mut membership,
        &mut sessions,
    );
    assert!(
        out_rows(session) >= 1,
        "the free portal refresh is effective on its own"
    );

    // Exhaust the account bucket; the extra requests are silently refused.
    for _ in 0..12 {
        dispatch(
            session,
            &rig.options,
            &wrap(&ClaimWindow::list_request()),
            1_000,
            &mut rig.world,
            &mut membership,
            &mut sessions,
        );
    }
    // At least the portal reply plus the burst; a loaded machine may refill one more token
    // during the loop (the exact burst size is not this test's subject).
    assert!(
        out_rows(session) >= 11,
        "the portal reply plus the allowed list burst"
    );
    session.recent.clear();
    drain_budget_later(&rig.world, session.account_guid);

    let before = session.gateway.snapshot();
    let mut bundle = vec![0, 25];
    for child in [wrap(&[167, 0, 1, 4]), wrap(&ClaimWindow::list_request())] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    dispatch(
        session,
        &rig.options,
        &bundle,
        2_000,
        &mut rig.world,
        &mut membership,
        &mut sessions,
    );
    assert_eq!(
        session.gateway.snapshot(),
        before,
        "a refused budgeted child must not let the free sibling stage"
    );
    assert_eq!(out_rows(session), 0, "a refused bundle produces no replies");

    // The same free child still replies after the refusal, so the empty result
    // above came from the bundle drop, not from a stale portal throttle.
    dispatch(
        session,
        &rig.options,
        &wrap(&[167, 0, 1, 4]),
        3_000,
        &mut rig.world,
        &mut membership,
        &mut sessions,
    );
    assert!(out_rows(session) >= 1);
}

/// The action budget is one bucket per authenticated account, never per
/// connection: a second session of the same account inherits the exhaustion,
/// while another account still has its own allowance.
#[test]
fn account_budget_is_shared_across_sessions_and_not_across_accounts() {
    let mut rig = rig("account-budget");
    rig.sessions[1].account_guid = rig.sessions[0].account_guid;
    for (index, ticket) in ["account-a", "account-a-second", "account-b"]
        .into_iter()
        .enumerate()
    {
        log_in(&mut rig.sessions[index], ticket);
    }
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();

    let first = &mut rig.sessions[0];
    first.recent_on = true;
    first.recent.clear();
    for _ in 0..10 {
        dispatch(
            first,
            &rig.options,
            &wrap(&ClaimWindow::list_request()),
            1_000,
            &mut rig.world,
            &mut membership,
            &mut sessions,
        );
    }
    assert_eq!(out_rows(first), 10, "the first session spends the burst");
    // No refill between the two sessions, however slowly the machine runs (L016).
    drain_budget_later(&rig.world, first.account_guid);

    let second = &mut rig.sessions[1];
    second.recent_on = true;
    dispatch(
        second,
        &rig.options,
        &wrap(&ClaimWindow::list_request()),
        1_000,
        &mut rig.world,
        &mut membership,
        &mut sessions,
    );
    assert_eq!(
        out_rows(second),
        0,
        "a second connection on the exhausted account is refused"
    );

    let other = &mut rig.sessions[2];
    other.recent_on = true;
    dispatch(
        other,
        &rig.options,
        &wrap(&ClaimWindow::list_request()),
        1_000,
        &mut rig.world,
        &mut membership,
        &mut sessions,
    );
    assert_eq!(
        out_rows(other),
        1,
        "another account keeps its own allowance"
    );
}
