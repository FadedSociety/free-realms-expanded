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
        world: Some(scratch.join("world")),
        stock_text: None,
        allow_lan: false,
        security_log: Some(log_path.clone()),
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
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
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
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
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
    assert_eq!(trace::parse_hex("2300"), Some(vec![0x23, 0x00]));
    assert_eq!(trace::parse_hex("0\u{e9}0"), None);
    assert_eq!(trace::parse_hex("zz"), None);
    assert_eq!(trace::parse_hex("230"), None);
    assert_eq!(trace::parse_hex(""), None);
}

#[test]
fn live_outfit_fanout_respects_directed_visibility_cap() {
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
