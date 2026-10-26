use super::*;
use miniz_oxide::deflate::compress_to_vec_zlib;

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut app = vec![5, 0, 1];
    app.extend((inner.len() as u32).to_le_bytes());
    app.extend(inner);
    app
}

#[test]
fn w120_player_denial_is_logged_and_studio_requires_admin() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = std::env::temp_dir().join(format!("osfr-w120-session-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let log_path = scratch.join("security.jsonl");
    let _ = fs::remove_file(&log_path);
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
        dev_tools: true,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: Some(scratch.join("world")),
        stock_text: None,
        allow_lan: false,
        security_log: Some(log_path.clone()),
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    };
    let mut server = Server::new(options).unwrap();
    let mut session = Session::new(
        SessionId(120),
        "127.0.0.1:12345".parse().unwrap(),
        SessionRequest {
            id: 120,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    // A no-account-store session can only be the host's own loopback dev
    // session: admin with --dev-tools. Demote it to test a player's denial.
    assert_eq!(
        session.gateway.role(),
        osfr_tools::account_store::AccountRole::Admin
    );
    session
        .gateway
        .set_role(osfr_tools::account_store::AccountRole::Player);
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, value) in ["w120", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
    let before = session.gateway.snapshot();
    session
        .dispatch_message(
            &wrap(&osfr_tools::chat::local_request("realm god")),
            &server.options,
            1000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .unwrap();
    assert_eq!(session.gateway.snapshot(), before);
    let mut invalid_equip = vec![42, 0, 3, 0];
    invalid_equip.extend((-1_i32).to_le_bytes());
    invalid_equip.extend([0; 8]);
    let mut bundle = vec![0, 25];
    for child in [
        wrap(&osfr_tools::chat::local_request("realm god")),
        wrap(&invalid_equip),
    ] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    assert!(
        session
            .dispatch_message(
                &bundle,
                &server.options,
                1000,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None
            )
            .is_err()
    );
    assert_eq!(session.gateway.snapshot(), before);
    server.flush_security();
    let rows: Vec<serde_json::Value> = fs::read_to_string(&log_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        rows.iter()
            .any(|row| row["kind"] == "dev_command_denial" && row["reason"] == "authorization"),
        "{rows:?}"
    );
    let denied: u64 = rows
        .iter()
        .filter(|row| row["kind"] == "dev_command_denial")
        .map(|row| row["count"].as_u64().unwrap())
        .sum();
    assert_eq!(
        denied, 2,
        "rollback must not erase an authorization refusal"
    );
    let key = SessionKey {
        peer: session.peer,
        connection_id: session.request.id,
    };
    server.sessions.insert(key, session);
    let request = studio::Request {
        method: "GET".into(),
        path: "/api/state".into(),
        body: Vec::new(),
    };
    assert_eq!(server.studio_route(&request, 1001).0, 403);
    server
        .sessions
        .get_mut(&key)
        .unwrap()
        .gateway
        .set_role(osfr_tools::account_store::AccountRole::Builder);
    assert_eq!(server.studio_route(&request, 1002).0, 403);
    server
        .sessions
        .get_mut(&key)
        .unwrap()
        .gateway
        .set_role(osfr_tools::account_store::AccountRole::Admin);
    assert_eq!(server.studio_route(&request, 1003).0, 200);
    drop(server);
    fs::remove_dir_all(scratch).unwrap();
}
#[test]
fn pose_only_saves_are_coalesced_and_flushed_by_tick() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = std::env::temp_dir().join(format!("osfr-pose-save-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let options = Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state: scratch.join("character.json"),
        content: None,
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    };
    let mut server = Server::new(options).unwrap();
    let mut session = Session::new(
        SessionId(7),
        "127.0.0.1:12346".parse().unwrap(),
        SessionRequest {
            id: 7,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, value) in ["pose", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
    let start = session.gateway.state().clone();
    let saved_x = || -> f32 {
        let text = fs::read_to_string(&session_path(&scratch)).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
        doc["character"]["position"][0].as_f64().unwrap() as f32
    };
    let step = |session: &mut Session, server: &mut Server, dx: f32, now_ms: u64| {
        let mut pose = vec![125, 0];
        pose.extend(start.guid.to_le_bytes());
        let p = [start.position[0] + dx, start.position[1], start.position[2]];
        for value in p.into_iter().chain(start.rotation) {
            pose.extend(value.to_le_bytes());
        }
        pose.extend([2, 255]);
        session
            .dispatch_message(
                &wrap(&pose),
                &server.options,
                now_ms,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .unwrap();
    };
    // The first pose after the interval is written at once.
    step(&mut session, &mut server, 0.5, 10_000);
    assert_eq!(saved_x(), start.position[0] + 0.5);
    // A second pose inside the interval waits: the file keeps the old one.
    step(&mut session, &mut server, 1.0, 10_500);
    assert_eq!(session.gateway.state().position[0], start.position[0] + 1.0);
    assert_eq!(saved_x(), start.position[0] + 0.5);
    // Canary: the tick before the interval ends writes nothing yet...
    session
        .tick(
            &server.options,
            14_000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
        )
        .unwrap();
    assert_eq!(saved_x(), start.position[0] + 0.5);
    // ...and the first tick after it writes the newest pose.
    session
        .tick(
            &server.options,
            15_000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
        )
        .unwrap();
    assert_eq!(saved_x(), start.position[0] + 1.0);
    drop(server);
    fs::remove_dir_all(scratch).unwrap();
}

fn session_path(scratch: &std::path::Path) -> PathBuf {
    scratch.join("character.json")
}

fn direct(app: &[u8], compressed: bool) -> Vec<u8> {
    let mut wire = vec![app[0], u8::from(compressed)];
    if compressed {
        wire.extend(compress_to_vec_zlib(&app[1..], 6));
    } else {
        wire.extend(&app[1..]);
    }
    let crc = osfr_tools::soe::seeded_crc32(CRC_SEED, &wire).to_be_bytes();
    wire.extend(&crc[1..]);
    wire
}

#[test]
#[ignore = "requires OSFR_KART_WEAPONS private content and initialization"]
fn private_direct_pose_reaches_session_and_unlocks_reliable_rocket_use() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let root = PathBuf::from(env::var_os("OSFR_KART_WEAPONS").unwrap());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let saved = env::temp_dir().join(format!(
        "osfr-kart-direct-{}-{nonce}.json",
        std::process::id()
    ));
    std::fs::copy(root.join("initial-character.json"), &saved).unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    let options = Options {
        bind: socket.local_addr().unwrap(),
        reference: root.join("../../references/rust-gateway-reference"),
        state: saved.clone(),
        content: Some(root.join("content")),
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.to_owned(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: true,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    };
    let mut session = Session::new(
        SessionId(17),
        client.local_addr().unwrap(),
        SessionRequest {
            id: 17,
            crc_length: 3,
            buffer_size: 512,
        },
        &options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut world = SharedWorld::new();
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    world.session_keys.insert(
        session.id,
        SessionKey {
            peer: session.peer,
            connection_id: session.request.id,
        },
    );
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["private-direct-regression", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle_at(&login, 0).unwrap();
    session.gateway.handle_at(&wrap(&[13, 0]), 100).unwrap();
    session.gateway.handle_at(&wrap(&[10, 0]), 200).unwrap();
    let mut chat = vec![15, 0, 1, 0, 0, 0];
    chat.extend(1u64.to_le_bytes());
    chat.extend(0u64.to_le_bytes());
    chat.extend([0; 40]);
    chat.extend(10u32.to_le_bytes());
    chat.extend(b"realm race");
    chat.extend([0; 28]);
    session.gateway.handle_at(&wrap(&chat), 1100).unwrap();
    session.gateway.handle_at(&wrap(&[13, 0]), 1200).unwrap();
    let mut register = vec![33, 0, 9, 0];
    register.extend(1u64.to_le_bytes());
    register.extend([0; 8]);
    session.gateway.handle_at(&wrap(&register), 1500).unwrap();
    session.gateway.handle_at(&wrap(&[10, 0]), 1600).unwrap();
    let doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("initial-character.json")).unwrap())
            .unwrap();
    let instance = doc["gameplay"]["kart_entry"]["next_instance"]
        .as_u64()
        .unwrap() as u32;
    let mut start = vec![41, 0, 108, 0];
    start.extend(917566u32.to_le_bytes());
    start.extend(instance.to_le_bytes());
    start.extend(1u64.to_le_bytes());
    session.gateway.handle_at(&wrap(&start), 2000).unwrap();
    session.gateway.tick_at(20_000).unwrap();
    let content: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("content/kart_entry.json")).unwrap())
            .unwrap();
    let position: [f32; 3] =
        serde_json::from_value(content["race"]["pickups"][0]["position"].clone()).unwrap();
    let mut claim = vec![33, 0, 17, 0];
    claim.extend(1u64.to_le_bytes());
    for v in position {
        claim.extend(v.to_le_bytes());
    }
    claim.extend(1f32.to_le_bytes());
    session.gateway.handle_at(&wrap(&claim), 20_100).unwrap();
    let mut use_item = vec![33, 0, 19, 0];
    use_item.extend(1u64.to_le_bytes());
    assert!(
        session
            .gateway
            .handle_at(&wrap(&use_item), 20_105)
            .unwrap()
            .replies
            .is_empty()
    );
    let before = serde_json::to_value(session.gateway.snapshot()).unwrap();
    assert_eq!(
        before["gameplay"]["kart_entry"]["race"]["queued_items"],
        json!([1])
    );
    // Corrupt pose and a valid-CRC direct use must not change live gameplay.
    let mut pose = vec![33, 0, 6, 0];
    let mut body = vec![0; 85];
    for (i, v) in [1f32, 0., 0., 0., 1., 0., 0., 0., 1.]
        .iter()
        .chain(position.iter())
        .enumerate()
    {
        body[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    }
    body[81..85].copy_from_slice(&1u32.to_le_bytes());
    pose.extend(body);
    let mut corrupt = direct(&wrap(&pose), false);
    corrupt[20] ^= 1;
    session
        .handle(
            &socket,
            &corrupt,
            &options,
            20_110,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    session
        .handle(
            &socket,
            &direct(&wrap(&use_item), true),
            &options,
            20_111,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(session.gateway.snapshot()).unwrap(),
        before
    );
    for compressed in [false, true] {
        session
            .handle(
                &socket,
                &direct(&wrap(&pose), compressed),
                &options,
                20_120,
                &mut world,
                &mut membership,
                &mut sessions,
            )
            .unwrap();
    }
    let after = serde_json::to_value(session.gateway.snapshot()).unwrap();
    assert!(after["gameplay"]["kart_entry"]["race"]["weapon_pose"].is_object());
    assert_eq!(
        after["gameplay"]["kart_entry"]["race"]["queued_items"],
        json!([1])
    );
    let mut reliable = 0u16.to_be_bytes().to_vec();
    reliable.extend(wrap(&use_item));
    session
        .handle(
            &socket,
            &session.codec.encode(RELIABLE_DATA, &reliable).unwrap(),
            &options,
            20_130,
            &mut world,
            &mut membership,
            &mut sessions,
        )
        .unwrap();
    let paid = serde_json::to_value(session.gateway.snapshot()).unwrap();
    assert_eq!(
        paid["gameplay"]["kart_entry"]["race"]["queued_items"],
        json!([])
    );
    assert_eq!(
        paid["gameplay"]["kart_entry"]["race"]["weapons"]["weapons"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(paid["character"], before["character"]);
    assert_eq!(
        load_or_create(&saved, session.gateway.state().clone()).unwrap(),
        session.gateway.snapshot()
    );
    std::fs::remove_file(saved).unwrap();
}

/// Lead (2026-10-18): `/api/send` sliced its hex by byte index, so an even-length body with a
/// multi-byte character ("0\u{e9}0") panicked the server. Non-hex input is now refused.
#[test]
fn studio_hex_refuses_non_hex_input_instead_of_panicking() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    assert_eq!(trace::parse_hex("2300"), Some(vec![0x23, 0x00]));
    assert_eq!(trace::parse_hex("0\u{e9}0"), None);
    assert_eq!(trace::parse_hex("zz"), None);
    assert_eq!(trace::parse_hex("230"), None);
    assert_eq!(trace::parse_hex(""), None);
}

#[test]
fn live_outfit_fanout_respects_directed_visibility_cap() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let mut world = SharedWorld::new();
    world.players = PlayerRegistry::new(InterestPolicy::new(100.0, 1).unwrap());
    let zone = ZoneInstance {
        zone_id: 1,
        instance_id: 1,
    };
    for (session, character, position) in [
        (1, 101, [0.0, 0.0, 0.0]),
        (2, 103, [20.0, 0.0, 0.0]),
        (3, 102, [50.0, 0.0, 0.0]),
    ] {
        let mut presentation = osfr_tools::players::PlayerPresentation::default();
        presentation.character_guid = character;
        world
            .players
            .join_with_presentation(
                SessionId(session),
                1_000 + session,
                character,
                zone,
                position,
                0.0,
                presentation,
            )
            .unwrap();
        world.joined.insert(SessionId(session));
    }
    assert!(world.players.knows_character(SessionId(2), 101));
    assert!(!world.players.knows_character(SessionId(3), 101));

    let mut outfit = vec![35, 0, 7, 0];
    outfit.extend(101u64.to_le_bytes());
    outfit.extend(0i32.to_le_bytes());
    outfit.extend(0i32.to_le_bytes());
    let mut slot = vec![35, 0, 6, 0];
    slot.extend(101u64.to_le_bytes());
    slot.extend(0i32.to_le_bytes());
    let nearby = pet_removal_packet(900);
    let deliveries = world.ride_fanout(
        SessionId(1),
        &[outfit.clone(), slot.clone(), nearby.clone()],
    );
    assert!(deliveries.iter().any(|delivery| {
        delivery.recipient == SessionId(2) && delivery.bytes.ends_with(&outfit)
    }));
    assert!(
        deliveries.iter().any(|delivery| {
            delivery.recipient == SessionId(2) && delivery.bytes.ends_with(&slot)
        })
    );
    assert!(!deliveries.iter().any(|delivery| {
        delivery.recipient == SessionId(3) && delivery.bytes.ends_with(&outfit)
    }));
    assert!(
        !deliveries.iter().any(|delivery| {
            delivery.recipient == SessionId(3) && delivery.bytes.ends_with(&slot)
        })
    );
    assert!(deliveries.iter().any(|delivery| {
        delivery.recipient == SessionId(3) && delivery.bytes.ends_with(&nearby)
    }));
}

/// F179/F264: the session splits a bundle that contains a combat/shared interaction before
/// `handle_at`, so the library's per-message budget never sees the other children. The reply
/// budget accumulates across those ordered children; the third large definition read puts a
/// read-only bundle over it and the dispatch is refused before that child commits. The pose
/// child dispatched first must survive that refusal: only the session's ordered per-child split
/// commits it, so the assertion proves the split ran rather than a whole-bundle `handle_at`.
#[test]
fn a_split_combat_bundle_obeys_the_bundle_reply_budget() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let scratch = test_support::Scratch::new("f179-split");
    let content = test_support::vendor_content(&scratch);
    // 43 maximum-size rows are ~2.7 MiB per child: two children stay under the 8 MiB bundle
    // budget, the third pushes the split bundle over it.
    let ids: Vec<u32> = (0..43).map(|index| 930_000 + index).collect();
    test_support::item_catalogue(
        &content,
        &ids.iter().map(|id| (*id, 65_536)).collect::<Vec<_>>(),
    );

    let mut options = test_support::options(scratch.state());
    options.content = Some(content);
    let mut server = Server::new(options).unwrap();
    let mut session = Session::new(
        SessionId(179),
        "127.0.0.1:12347".parse().unwrap(),
        SessionRequest {
            id: 179,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, value) in ["f179", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
    let start = session.gateway.state().clone();
    // Client reader `008e5940`: u16 family/subtype, u64 actor, position xyz, rotation xyz,
    // then the two tail bytes.
    let pose_at = |dx: f32| {
        let mut pose = vec![125, 0];
        pose.extend(start.guid.to_le_bytes());
        for value in [start.position[0] + dx, start.position[1], start.position[2]]
            .into_iter()
            .chain(start.rotation)
        {
            pose.extend(value.to_le_bytes());
        }
        pose.extend([2, 255]);
        wrap(&pose)
    };
    // The first movement attunes the nearby warpstone (a gameplay change); do it before the
    // snapshot so the split bundle's own pose only moves the character.
    session
        .dispatch_message(
            &pose_at(0.25),
            &server.options,
            900,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .unwrap();
    let before = session.gateway.snapshot();
    let mut request = vec![165, 0, 2, 0];
    request.extend((ids.len() as i32).to_le_bytes());
    ids.iter().for_each(|id| request.extend(id.to_le_bytes()));
    // The combat trigger only has to make the session split the bundle; a selection for an
    // unknown actor changes nothing. Client constructor `00a36f80`: u64 actor, i32 option.
    let mut trigger = vec![26, 0, 10, 0];
    trigger.extend(u64::MAX.to_le_bytes());
    trigger.extend(0i32.to_le_bytes());
    // The pose is dispatched first and committed on its own: that commit survives the later
    // refusal, which a whole-bundle `handle_at` would roll back instead.
    let children = [
        pose_at(0.75),
        wrap(&trigger),
        wrap(&request),
        wrap(&request),
        wrap(&request),
    ];
    let mut bundle = vec![0, 25];
    for child in &children {
        assert!(child.len() < 255);
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    let refused = session.dispatch_message(
        &bundle,
        &server.options,
        1000,
        &mut server.world,
        &mut server.membership,
        &mut server.sessions,
        None,
    );
    let error = match refused {
        Ok(_) => panic!("the split bundle staged more than the outbound budget"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("outbound budget"),
        "expected the reply budget refusal, got {error}"
    );
    let mut expected = before.clone();
    expected.character.position[0] = start.position[0] + 0.75;
    assert_eq!(
        session.gateway.state().position[0],
        expected.character.position[0],
        "the split's first child must commit before the crossing child is refused"
    );
    assert_eq!(
        session.gateway.snapshot(),
        expected,
        "the refused read changed character or gameplay state"
    );
    assert_eq!(
        session.bundle_budget.used(),
        None,
        "a dispatch must reset the bundle budget"
    );

    // Positive control: one definition read alone stays under the per-request cap.
    session
        .dispatch_message(
            &wrap(&request),
            &server.options,
            1000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .unwrap();
    assert_eq!(session.gateway.snapshot(), expected);

    // A whole (non-split) bundle still answers after a refused split: the budget is reset for
    // every top-level message, so the refused message's bytes cannot charge the next one.
    let mut ordinary = vec![0, 25];
    for child in [wrap(&request), wrap(&request)] {
        assert!(child.len() < 255);
        ordinary.push(child.len() as u8);
        ordinary.extend(child);
    }
    session
        .dispatch_message(
            &ordinary,
            &server.options,
            1000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .expect("a whole bundle under the 8 MiB budget must not be charged twice");
    assert_eq!(
        session.gateway.snapshot(),
        expected,
        "a whole bundle must change nothing"
    );
    // The reset, not the next dispatch arming the budget again, must leave it outside: a tick's
    // replies after a successful bundle would otherwise charge an already-finished message.
    assert_eq!(session.bundle_budget.used(), None);
}

/// F286 (F179 finding 4, the documented-accounting alternative): a begin-zoning child that refuses
/// before its commit keeps its staged replies charged to the message's bundle budget until the
/// dispatch ends; `dispatch_message` then releases the message budget (final assertion).
/// `BundleBudget` owns the rule: the ceiling bounds bytes staged for one message, not bytes the
/// sender queues. `realm race` is a real zone transfer; deleting the save directory makes its
/// precommit save fail, which is the `Err(_) if transfer` refusal path. The control bundle's last
/// read fits by 528 bytes; in the real bundle the same read is over by 371 because the refused
/// transfer's 899 staged bytes stayed charged. A refund (F264's `release`) would make the last
/// read fit, so this test would fail. The control also establishes the live `World` membership;
/// the refusal must leave that membership, the public instance and the zone unchanged (F322 review
/// finding 1). The body's name says what it pins; the old F264 name is a thin alias below because
/// `lab/autopilot/protected-pass.txt` still selects it.
#[test]
fn a_refused_precommit_transfer_keeps_its_staged_bytes_until_dispatch_end() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let scratch = test_support::Scratch::new("f286-transfer");
    let content = test_support::vendor_content(&scratch);
    // Each staged reply is charged with its 7-byte application wrapper (`wrapped`). The 26/10
    // trigger stages no reply. A 43-row read charges 43*(7+12+65,536) + (7+181) = 2,819,053;
    // the 41-row read charges 41*65,555 + (7+173) = 2,687,935. Those three reads leave 62,567
    // bytes of the 8 MiB budget; the 62,000-byte row's read charges (7+12+62,000) + (7+13) =
    // 62,039, so the control fits by 528. The refused transfer's staged replies charge
    // 394+418+87 = 899, putting the real bundle 371 over the budget.
    let ids: Vec<u32> = (0..43).map(|index| 930_000 + index).collect();
    test_support::item_catalogue(
        &content,
        &ids.iter()
            .map(|id| (*id, 65_536))
            .chain([(930_100, 62_000)])
            .collect::<Vec<_>>(),
    );
    let mut options = test_support::options(scratch.state());
    options.content = Some(content);
    // `realm race` is the developer entry to the recovered kart scene transfer.
    options.dev_tools = true;
    let mut server = Server::new(options).unwrap();
    let mut session = Session::new(
        SessionId(286),
        "127.0.0.1:12349".parse().unwrap(),
        SessionRequest {
            id: 286,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, value) in ["f286", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    session.gateway.handle(&login).unwrap();
    session.gateway.handle(&wrap(&[13, 0])).unwrap();
    session.gateway.handle(&wrap(&[10, 0])).unwrap();
    let zone_id = session.gateway.state().zone_id;
    let before = session.gateway.snapshot();
    let read = |ids: &[u32]| {
        let mut request = vec![165, 0, 2, 0];
        request.extend((ids.len() as i32).to_le_bytes());
        ids.iter().for_each(|id| request.extend(id.to_le_bytes()));
        wrap(&request)
    };
    let mut trigger = vec![26, 0, 10, 0];
    trigger.extend(u64::MAX.to_le_bytes());
    trigger.extend(0i32.to_le_bytes());
    let bundle = |transfer: bool| {
        let mut children = vec![read(&ids), read(&ids), read(&ids[..41])];
        if transfer {
            children.push(wrap(&osfr_tools::chat::local_request("realm race")));
        }
        children.push(wrap(&trigger));
        children.push(read(&[930_100]));
        let mut bundle = vec![0, 25];
        for child in children {
            assert!(child.len() < 255);
            bundle.push(child.len() as u8);
            bundle.extend(child);
        }
        bundle
    };
    let dispatch = |session: &mut Session,
                    server: &mut Server,
                    message: &[u8]|
     -> Result<(), Box<dyn std::error::Error>> {
        session.dispatch_message(
            message,
            &server.options,
            1000,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
    };
    // The same children without the transfer child fit the budget: the last read is accepted.
    dispatch(&mut session, &mut server, &bundle(false))
        .expect("the control bundle must fit the 8 MiB bundle budget");
    assert_eq!(
        session.gateway.snapshot(),
        before,
        "the control bundle changed character or gameplay state"
    );
    // The control bundle is the positive control: it fits, and it also establishes the
    // session's live membership. A refusal must leave that live row and public instance alone;
    // the character+gameplay `Snapshot` above cannot see either (review finding 1).
    let player = session.character_guid;
    let membership = server.membership.player_location(player).cloned();
    assert!(
        membership.is_some(),
        "the control bundle must establish live membership before the refusal"
    );
    let public_instance = server.membership.public_instance(0);
    // The transfer cannot commit: its precommit save has nowhere to write. Its staged bytes stay
    // charged to the message, so the identical last read is now over the budget. F344: this
    // refusal is the save's own failure, so asserting that the save path stays absent says nothing
    // about whether the refusal attempted a save (3cec461 review finding 1). The exact budget
    // error below still catches a save on the refusal branch that propagates its failure; the
    // persisted state is observed through the snapshot, zone and membership assertions.
    fs::remove_dir_all(scratch.path()).unwrap();
    let error = dispatch(&mut session, &mut server, &bundle(true))
        .expect_err("the refused transfer's staged replies must stay charged");
    assert!(
        error.to_string().contains("outbound budget"),
        "expected the reply budget refusal, got {error}"
    );
    assert_eq!(
        session.gateway.state().zone_id,
        zone_id,
        "a refused precommit transfer must not commit the move"
    );
    assert_eq!(
        session.gateway.snapshot(),
        before,
        "the refused transfer's bundle changed character or gameplay state"
    );
    assert_eq!(
        server.membership.player_location(player),
        membership.as_ref(),
        "a refused precommit transfer changed live membership"
    );
    assert_eq!(
        server.membership.public_instance(0),
        public_instance,
        "a refused precommit transfer changed the public instance"
    );
    assert_eq!(
        session.bundle_budget.used(),
        None,
        "a dispatch must reset the bundle budget"
    );
}

/// The F264 protected name (`lab/autopilot/protected-pass.txt`, and the six integration suites that
/// embed this module): an alias for the test above. It keeps the historical selector working while
/// the real test name says that the refused transfer's staged bytes stay charged until the dispatch
/// ends, not that a refund happens. F322 review finding 4.
#[test]
fn a_refused_precommit_transfer_releases_its_reply_budget() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    a_refused_precommit_transfer_keeps_its_staged_bytes_until_dispatch_end();
}

/// Session-test scratch directory and `Options` builder, shared with `lead_trade_commit`.
#[cfg(test)]
#[path = "test_support.rs"]
pub(super) mod test_support;

/// T008/T266: the opcode-164 jump body reaches exactly the sessions whose
/// directed view contains the sender, at the request's final committed pose.
/// The name is pinned by `lab/autopilot/protected-pass.txt`; T266 removed the
/// server relay window it used to assert, so every accepted jump now relays.
#[test]
fn jump_relay_reaches_only_directed_viewers_and_is_throttled() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = std::env::temp_dir().join(format!("osfr-jump-{}", std::process::id()));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let options = Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: root.join("references/rust-gateway-reference"),
        state: scratch.join("character.json"),
        content: None,
        identity: None,
        account_store: None,
        gateway_ticket: login::DEFAULT_GATEWAY_TICKET.into(),
        handoff_key: Vec::new(),
        seconds: 1,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: None,
        resources: None,
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
        guild_invite_ttl_ms: osfr_tools::guilds::DEFAULT_INVITE_WINDOW_MS,
    };
    let new_session = |id: u64| {
        Session::new(
            SessionId(id),
            format!("127.0.0.1:{}", 30_000 + id).parse().unwrap(),
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
        .unwrap()
    };
    let mut sender = new_session(17);
    let recipient = new_session(18);
    let non_viewer = new_session(19);
    let mut world = SharedWorld::new();
    // One directed slot: a nearby session can fall outside the sender's view.
    world.players = PlayerRegistry::new(InterestPolicy::new(50.0, 1).unwrap());
    let mut membership = World::runtime();
    let mut sessions = SessionMap::new();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["jump-relay", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    for (message, now) in [(login, 0), (wrap(&[13, 0]), 100), (wrap(&[10, 0]), 200)] {
        sender
            .dispatch_message(
                &message,
                &options,
                now,
                &mut world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();
    }
    let joined = world
        .players
        .get(sender.id)
        .expect("sender entered the world");
    let zone = joined.zone;
    let start = joined.position;
    let character = joined.character_guid;
    for (session, guid, dx) in [(&recipient, 2u64, 1.0f32), (&non_viewer, 3, 5.0)] {
        let mut presentation = osfr_tools::players::PlayerPresentation::default();
        presentation.character_guid = guid;
        world
            .players
            .join_with_presentation(
                session.id,
                guid + 10_000,
                guid,
                zone,
                [start[0] + dx, start[1], start[2]],
                0.0,
                presentation,
            )
            .unwrap();
        world.joined.insert(session.id);
        world.session_keys.insert(
            session.id,
            SessionKey {
                peer: session.peer,
                connection_id: session.request.id,
            },
        );
    }
    assert!(world.players.knows_character(recipient.id, character));
    assert!(!world.players.knows_character(non_viewer.id, character));
    let recipient_key = SessionKey {
        peer: recipient.peer,
        connection_id: recipient.request.id,
    };
    let non_viewer_key = SessionKey {
        peer: non_viewer.peer,
        connection_id: non_viewer.request.id,
    };
    let non_viewer_id = non_viewer.id;
    sessions.insert(recipient_key, recipient);
    sessions.insert(non_viewer_key, non_viewer);
    let jump_body = |guid: u64, position: [f32; 3], direction: [f32; 3], velocity: f32| {
        let mut inner = 164u16.to_le_bytes().to_vec();
        inner.extend(guid.to_le_bytes());
        for value in position.into_iter().chain(direction) {
            inner.extend(value.to_le_bytes());
        }
        // Native sender `0092e670` forces state bit 2; the 125 pose below keeps
        // a state without it, so the bundle pins which byte the relay uses.
        inner.extend([4, 255]);
        inner.extend(velocity.to_le_bytes());
        assert_eq!(inner.len(), osfr_tools::jump_relay::BODY_LEN);
        inner
    };
    let jump = |dx: f32| {
        let position = [start[0] + dx, start[1], start[2]];
        wrap(&jump_body(character, position, [0.0, 0.0, 1.0], 0.5))
    };
    let jump_bodies = |messages: Vec<Vec<u8>>| -> Vec<Vec<u8>> {
        messages
            .into_iter()
            .filter_map(|message| message.get(7..).map(<[u8]>::to_vec))
            .filter(|inner| inner.starts_with(&[164, 0]))
            .collect()
    };
    sender
        .dispatch_message(
            &jump(0.5),
            &options,
            1_000,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    let relayed = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_000,
    ));
    assert_eq!(relayed.len(), 1, "the visible peer receives one jump");
    let relay = osfr_tools::jump_relay::decode(&relayed[0], character).unwrap();
    assert_eq!(relay.position, [start[0] + 0.5, start[1], start[2]]);
    assert_eq!(relay.vertical_velocity, 0.5);
    assert_eq!((relay.state, relay.unknown), (4, 255));
    assert_eq!(relay.state & 4, 4, "bit 2 carries the jump velocity");
    assert!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&non_viewer_key).unwrap(),
            1_000,
        ))
        .is_empty(),
        "a nearby session outside the directed view receives nothing"
    );
    assert!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            &mut sender,
            1_000,
        ))
        .is_empty(),
        "the sender never receives its own jump"
    );
    // T266: no server-side relay window. A second jump 60 ms after the first
    // (the native sender's own gate is 50 ms) still reaches the viewer.
    sender
        .dispatch_message(
            &jump(1.0),
            &options,
            1_060,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    let second = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_060,
    ));
    assert_eq!(second.len(), 1, "every accepted jump relays");
    assert_eq!(
        osfr_tools::jump_relay::decode(&second[0], character)
            .unwrap()
            .position[0],
        start[0] + 1.0
    );
    assert_eq!(sender.gateway.state().position[0], start[0] + 1.0);
    // T266: later jumps travel far enough that the one directed slot would
    // switch from the recipient to the nearby non-viewer; that early case is
    // already pinned above, so drop it to keep the later relays unambiguous.
    let _ = world.players.leave(non_viewer_id);
    world.joined.remove(&non_viewer_id);
    // A refusal is discarded before the staged transaction commits: the pose
    // and world are unchanged.
    let settled = sender.gateway.state().clone();
    let direction = [0.0, 0.0, 1.0];
    let exact = jump_body(
        character,
        [start[0] + 2.0, start[1], start[2]],
        direction,
        0.5,
    );
    let refusals = [
        (
            "foreign guid",
            jump_body(
                character + 1,
                [start[0] + 2.0, start[1], start[2]],
                direction,
                0.5,
            ),
        ),
        (
            "nonfinite position",
            jump_body(character, [f32::NAN, start[1], start[2]], direction, 0.5),
        ),
        (
            "nonfinite direction",
            jump_body(
                character,
                [start[0] + 2.0, start[1], start[2]],
                [f32::INFINITY, 0.0, 1.0],
                0.5,
            ),
        ),
        (
            "out-of-bounds position",
            jump_body(
                character,
                [
                    osfr_tools::movement::MAX_POSITION_COMPONENT + 1.0,
                    start[1],
                    start[2],
                ],
                direction,
                0.5,
            ),
        ),
        ("short body", exact[..39].to_vec()),
        ("trailing byte", [exact, vec![0]].concat()),
    ];
    for (label, inner) in refusals {
        assert!(
            sender
                .dispatch_message(
                    &wrap(&inner),
                    &options,
                    1_200,
                    &mut world,
                    &mut membership,
                    &mut sessions,
                    None,
                )
                .is_err(),
            "{label} is refused"
        );
        assert_eq!(
            sender.gateway.state(),
            &settled,
            "{label} changed the sender"
        );
        assert!(
            jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
                sessions.get_mut(&recipient_key).unwrap(),
                1_200,
            ))
            .is_empty(),
            "{label} relayed"
        );
    }
    // T266: the refusals left the session on its committed pose and still able
    // to relay; there is no window for them to consume.
    sender
        .dispatch_message(
            &jump(2.0),
            &options,
            1_250,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert_eq!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            1_250,
        ))
        .len(),
        1,
        "an accepted jump after refusals relays"
    );
    // T266 finding 1: a 164 followed by a 125 in one bundle still relays, at the
    // bundle's final committed pose (P4) rather than the stale 164 pose (P3).
    // T266/F290: the 125's state (2, no bit 2) must not replace the 164's state.
    let mut newer = 125u16.to_le_bytes().to_vec();
    newer.extend(character.to_le_bytes());
    for value in [start[0] + 4.0, start[1], start[2], 0.0, 0.0, 1.0] {
        newer.extend(value.to_le_bytes());
    }
    newer.extend([2, 255]);
    let mut bundle = vec![0, 25];
    for child in [
        wrap(&jump_body(
            character,
            [start[0] + 3.0, start[1], start[2]],
            direction,
            3.0,
        )),
        wrap(&newer),
    ] {
        bundle.push(child.len() as u8);
        bundle.extend(child);
    }
    sender
        .dispatch_message(
            &bundle,
            &options,
            1_300,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert_eq!(sender.gateway.state().position[0], start[0] + 4.0);
    let bundled = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_300,
    ));
    assert_eq!(bundled.len(), 1, "the bundled jump is relayed once");
    let relayed = osfr_tools::jump_relay::decode(&bundled[0], character).unwrap();
    assert_eq!(
        relayed.position[0],
        start[0] + 4.0,
        "the relay uses the final pose, not the stale 164 pose"
    );
    assert_eq!(relayed.vertical_velocity, 3.0);
    assert_eq!(
        (relayed.state, relayed.unknown),
        (4, 255),
        "the relay keeps the 164's state bytes, not the later 125 pose's"
    );
    assert_eq!(relayed.state & 4, 4, "bit 2 carries the jump velocity");
    assert!(
        sender.gateway.take_jump_relay().is_none(),
        "the host consumed the relay body"
    );
    // T266: a soft-over-speed jump is corrected for the sender and still relays
    // the pose the server committed (the clamp target), not the client's P62.
    // The sleep refills the per-character relay budget (one token per 100 ms);
    // opcode 125/164 movement is not part of the account action budget.
    std::thread::sleep(std::time::Duration::from_millis(120));
    sender
        .dispatch_message(
            &jump(62.0),
            &options,
            1_400,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    let clamped_pose = sender.gateway.state().position;
    assert!(clamped_pose[0] < start[0] + 62.0);
    let clamped = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_400,
    ));
    assert_eq!(clamped.len(), 1, "a clamped jump is relayed");
    assert_eq!(
        osfr_tools::jump_relay::decode(&clamped[0], character)
            .unwrap()
            .position,
        clamped_pose,
        "the relay uses the server's clamped pose"
    );
    assert!(
        super::f120_fishing_hour_reconnect::drain_replies(&mut sender, 1_400)
            .iter()
            .any(|message| message[7..].starts_with(&[38, 0, 12, 0])),
        "the clamped sender receives the location correction"
    );
    // T266: a finite velocity over the relay bound is clamped, not refused (a
    // refused body would end the session); the pose is still validated. The
    // sleep refills the per-character relay budget so the relay is not dropped.
    std::thread::sleep(std::time::Duration::from_millis(120));
    sender
        .dispatch_message(
            &wrap(&jump_body(
                character,
                [start[0] + 6.0, start[1], start[2]],
                direction,
                osfr_tools::jump_relay::MAX_VERTICAL_SPEED + 1.0,
            )),
            &options,
            1_500,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert_eq!(sender.gateway.state().position[0], start[0] + 6.0);
    let bounded = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_500,
    ));
    assert_eq!(bounded.len(), 1);
    assert_eq!(
        osfr_tools::jump_relay::decode(&bounded[0], character)
            .unwrap()
            .vertical_velocity,
        osfr_tools::jump_relay::MAX_VERTICAL_SPEED
    );
    // T266/F290: a 164 flood from one character is capped by the per-character
    // relay budget, while every accepted pose still fans out through opcode 125.
    // The budget uses the server's monotonic wall clock (the lead-accepted F290
    // behaviour), so the deterministic bound is the burst plus the tokens that
    // can refill while this loop runs: no host load can flake it, and bypassing
    // the limiter still fails it. The sleep refills the relay budget to full.
    std::thread::sleep(std::time::Duration::from_millis(500));
    let flood_started = std::time::Instant::now();
    for step in 0..10u64 {
        sender
            .dispatch_message(
                &jump(7.0 + step as f32 * 0.25),
                &options,
                1_800 + step * 10,
                &mut world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();
    }
    let flood_seconds = flood_started.elapsed().as_secs_f64();
    let flood = super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_890,
    );
    let count = |inner: &[u8]| {
        flood
            .iter()
            .filter(|message| {
                message
                    .get(7..)
                    .is_some_and(|bytes| bytes.starts_with(inner))
            })
            .count()
    };
    let burst = osfr_tools::jump_relay::RELAY_BURST as usize;
    let refilled = (flood_seconds * osfr_tools::jump_relay::RELAY_RATE_PER_SECOND).ceil() as usize;
    assert!(
        count(&[164, 0]) >= burst,
        "the full relay burst always relays"
    );
    assert!(
        count(&[164, 0]) <= burst + refilled,
        "a 164 flood relays at most the burst plus the {refilled} tokens its \
         {flood_seconds:.3}s could refill"
    );
    assert_eq!(count(&[125, 0]), 10, "every accepted pose still fans out");
    // A normal ~0.5 s jump cadence is never dropped.
    std::thread::sleep(std::time::Duration::from_millis(500));
    sender
        .dispatch_message(
            &jump(10.0),
            &options,
            2_000,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert_eq!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            2_000,
        ))
        .len(),
        1,
        "a normal jump after the flood relays"
    );
    // F323 pins the signed budget clock: three jumps with the same dispatch
    // time, 120 ms apart, relay on the server wall clock. The session-anchored
    // dispatch clock F309 introduced refills zero tokens between them and drops
    // the third. The bounded re-drain only covers reliable-sender timing.
    std::thread::sleep(std::time::Duration::from_millis(120));
    for iteration in 0..3u64 {
        sender
            .dispatch_message(
                &jump(10.0 + iteration as f32 * 0.1),
                &options,
                2_100,
                &mut world,
                &mut membership,
                &mut sessions,
                None,
            )
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(120));
    }
    let mut relayed = Vec::new();
    for attempt in 0..5u64 {
        relayed.extend(jump_bodies(
            super::f120_fishing_hour_reconnect::drain_replies(
                sessions.get_mut(&recipient_key).unwrap(),
                2_130 + attempt * 10,
            ),
        ));
        if relayed.len() >= 3 {
            break;
        }
    }
    assert_eq!(
        relayed.len(),
        3,
        "the wall clock refills 1.2 tokens per 120 ms while the dispatch clock \
         only refills between dispatch times"
    );
    // The world-entry gate is shared with opcode 125: a jump before Ready is refused.
    let mut early = new_session(20);
    let before_ready = early.gateway.state().clone();
    let early_guid = early.gateway.state().guid;
    assert!(
        early
            .dispatch_message(
                &wrap(&jump_body(
                    early_guid,
                    [start[0] + 0.5, start[1], start[2]],
                    direction,
                    0.5,
                )),
                &options,
                1_600,
                &mut world,
                &mut membership,
                &mut sessions,
                None,
            )
            .is_err(),
        "a jump before Ready is refused"
    );
    assert_eq!(early.gateway.state(), &before_ready);
    assert!(
        early.gateway.take_jump_relay().is_none(),
        "a refused jump stages no relay body"
    );
    drop(world);
    let _ = fs::remove_dir_all(&scratch);
}

/// F309: ending the owning session forgets the character's per-character relay
/// budget. This charges `Security::jump_relays` through the real dispatch/commit
/// path, then calls `Server::end_session`. The `is_empty` assertion after the
/// leave is the cleanup guard: a deleted or mis-keyed `forget_jump_relay` call
/// leaves the character's key behind. The relay budget uses the server's
/// monotonic wall clock (the lead-accepted F290 behaviour), so a stalled host
/// may refill one token mid-burst; the reconnect jump is a smoke check on top of
/// the empty-bucket guard, not a second proof of the cleanup.
#[test]
fn end_session_forgets_the_character_jump_relay_budget() {
    if !test_support::bin_test_target("gateway") {
        return;
    }
    let scratch = test_support::Scratch::new("f309-jump-leave");
    let mut server = Server::new(test_support::options(scratch.state())).unwrap();
    let mut login = 1u16.to_le_bytes().to_vec();
    for (index, text) in ["jump-leave", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((text.len() as u32).to_le_bytes());
        login.extend(text.as_bytes());
        if index == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    let (mut sender, recipient) = {
        let new_session = |id: u64| {
            Session::new(
                SessionId(id),
                format!("127.0.0.1:{}", 30_000 + id).parse().unwrap(),
                SessionRequest {
                    id: id as u32,
                    crc_length: 3,
                    buffer_size: 512,
                },
                &server.options,
                CRC_SEED,
                None,
                None,
            )
            .unwrap()
        };
        (new_session(17), new_session(18))
    };
    for (message, now) in [
        (login.clone(), 0u64),
        (wrap(&[13, 0]), 100),
        (wrap(&[10, 0]), 200),
    ] {
        sender
            .dispatch_message(
                &message,
                &server.options,
                now,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .unwrap();
    }
    let joined = server
        .world
        .players
        .get(sender.id)
        .expect("sender entered the world");
    let zone = joined.zone;
    let start = joined.position;
    let character = joined.character_guid;
    assert_eq!(
        server.world.active_sessions.get(&character),
        Some(&sender.id),
        "the real join path owns the character's active session"
    );
    let mut presentation = osfr_tools::players::PlayerPresentation::default();
    presentation.character_guid = 2;
    let recipient_id = recipient.id;
    server
        .world
        .players
        .join_with_presentation(
            recipient_id,
            10_002,
            2,
            zone,
            [start[0] + 1.0, start[1], start[2]],
            0.0,
            presentation,
        )
        .unwrap();
    server.world.joined.insert(recipient_id);
    let recipient_key = SessionKey {
        peer: recipient.peer,
        connection_id: recipient.request.id,
    };
    server
        .world
        .session_keys
        .insert(recipient_id, recipient_key);
    server.sessions.insert(recipient_key, recipient);
    assert!(
        server
            .world
            .players
            .knows_character(recipient_id, character),
        "the observer must already know the sender for a jump to be charged"
    );
    let jump = |step: u64| {
        let mut inner = 164u16.to_le_bytes().to_vec();
        inner.extend(character.to_le_bytes());
        for value in [start[0] + step as f32, start[1], start[2], 0.0, 0.0, 1.0] {
            inner.extend(value.to_le_bytes());
        }
        inner.extend([4, 255]);
        inner.extend(0.5f32.to_le_bytes());
        assert_eq!(inner.len(), osfr_tools::jump_relay::BODY_LEN);
        wrap(&inner)
    };
    let relays = |server: &mut Server, now: u64| {
        super::f120_fishing_hour_reconnect::drain_replies(
            server.sessions.get_mut(&recipient_key).unwrap(),
            now,
        )
        .iter()
        .filter(|message| {
            message
                .get(7..)
                .is_some_and(|inner| inner.starts_with(&[164, 0]))
        })
        .count()
    };
    // Four jumps spend the burst; the fifth is the over-budget probe. On the
    // wall clock a stalled host may refill one token, so the range is
    // timing-proof; a limiter bypass relays all five.
    for step in 0..5u64 {
        sender
            .dispatch_message(
                &jump(step),
                &server.options,
                1_000 + step * 10,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .unwrap();
    }
    let relayed = relays(&mut server, 1_050);
    assert!(
        (osfr_tools::jump_relay::RELAY_BURST as usize..=5).contains(&relayed),
        "the burst relays and the fifth jump is dropped ({relayed} of 5)"
    );
    // `len` only proves one bucket was charged; `KeyedBuckets` exposes no keys
    // (net_policy is lead-only), so the empty check after the leave is the guard.
    assert_eq!(server.world.security.lock().unwrap().jump_relays.len(), 1);
    // End the owning session: the leave path must forget the character's key.
    let sender_key = SessionKey {
        peer: sender.peer,
        connection_id: sender.request.id,
    };
    server.end_session(sender_key, sender, "test_jump_leave");
    assert!(
        server.world.security.lock().unwrap().jump_relays.is_empty(),
        "end_session must forget the character's relay budget"
    );
    // A reconnect starts a fresh burst through the same real dispatch path.
    let mut reconnected = Session::new(
        SessionId(21),
        "127.0.0.1:30021".parse().unwrap(),
        SessionRequest {
            id: 21,
            crc_length: 3,
            buffer_size: 512,
        },
        &server.options,
        CRC_SEED,
        None,
        None,
    )
    .unwrap();
    for (message, now) in [
        (login, 2_000u64),
        (wrap(&[13, 0]), 2_100),
        (wrap(&[10, 0]), 2_200),
    ] {
        reconnected
            .dispatch_message(
                &message,
                &server.options,
                now,
                &mut server.world,
                &mut server.membership,
                &mut server.sessions,
                None,
            )
            .unwrap();
    }
    assert_eq!(
        reconnected.gateway.state().guid,
        character,
        "the reconnect loads the saved character"
    );
    // The first jump's save captured the starting pose (step 0) and the later
    // jumps were deferred by POSITION_SAVE_MS, so only the leave path's save
    // puts the last committed pose (step 4) on disk.
    assert_eq!(
        reconnected.gateway.state().position[0],
        start[0] + 4.0,
        "the leave path saved the pose of the last committed jump"
    );
    assert!(
        server
            .world
            .players
            .knows_character(recipient_id, character),
        "the observer knows the reconnected character"
    );
    reconnected
        .dispatch_message(
            &jump(5),
            &server.options,
            2_300,
            &mut server.world,
            &mut server.membership,
            &mut server.sessions,
            None,
        )
        .unwrap();
    assert_eq!(
        relays(&mut server, 2_350),
        1,
        "a reconnected character starts with a fresh relay burst"
    );
}

/// F161: a pet adoption spends a certificate and adds a pet in one atomic document; a refused
/// replacement leaves the document and the live session unchanged. Declared here because the
/// hub file is at its line ceiling.
#[path = "f161_pet_adoption_persist.rs"]
mod f161_pet_adoption_persist;

/// F261: the begin-zoning commit clears the previous scene's ground pickups on the staged
/// clone, including a free revive into the instance the session is already bound to. Declared
/// here because the hub file is at its line ceiling; the `#[cfg(test)]` marker also lets
/// `tests/spine_rules.rs` recognise this test-only fixture, whose content writes do not count
/// against the production `fs::write` ratchet.
#[cfg(test)]
#[path = "f261_pickup_reload.rs"]
mod f261_pickup_reload;

/// L011 (S12/L8): a vendor purchase whose character save fails changes nothing and ends
/// only that session. Test-only proof; no production code changes. The `#[cfg(test)]`
/// marker also lets `tests/spine_rules.rs` recognise this test-only file, so its fixture
/// copies do not count against the production `fs::write` ratchet.
#[cfg(test)]
#[path = "l011_vendor_save_failure.rs"]
mod l011_vendor_save_failure;

/// T258 (S12/L8): a quest-reward turn-in whose character save fails changes nothing and ends
/// only that session. Test-only proof; no production code changes.
#[cfg(test)]
#[path = "quest_reward_save_failure.rs"]
mod quest_reward_save_failure;

/// F276: a refused field replay through the real session writes no character save.
#[cfg(test)]
#[path = "f276_field_session_save.rs"]
mod f276_field_session_save;

/// L039: a save off the map or deep under the terrain enters at the nearest warpstone.
#[cfg(test)]
#[path = "l039_login_pose.rs"]
mod l039_login_pose;

/// L043 (lead): admin `realm move` of an online, loading or offline character, through a real
/// `Server` in account-store mode.
#[cfg(test)]
#[path = "l043_admin_move.rs"]
mod l043_admin_move;

/// T458: world events through a real `Server`: admin `realm event start|stop`, the world tick,
/// late viewers, refusals and a refusing queue.
#[cfg(test)]
#[path = "t458_world_events.rs"]
mod t458_world_events;

/// T478: a style card through the real dispatch and commit path: viewers in range, a late
/// joiner's AddPc, a failed save and a relog.
#[cfg(test)]
#[path = "style_card_use.rs"]
mod style_card_use;
