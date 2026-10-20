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
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
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
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
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
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
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

/// F179: the session splits a bundle that contains a combat/shared interaction before
/// `handle_at`, so the library's per-message budget never sees the other children. The reply
/// budget now accumulates across those ordered children; the third large definition read puts a
/// read-only bundle over it and the dispatch is refused before that child commits.
#[test]
fn a_split_combat_bundle_obeys_the_bundle_reply_budget() {
    use sha2::{Digest, Sha256};

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = test_support::Scratch::new("f179-split");
    let content = scratch.path().join("content");
    fs::create_dir_all(content.join("npcs")).unwrap();
    fs::create_dir_all(content.join("quests")).unwrap();
    let source = std::env::var_os("OSFR_APP_FUZZ_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("reports/round59-native-20261015T124443Z/content"));
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            fs::copy(&path, content.join(path.file_name().unwrap())).unwrap();
        }
    }
    let provenance = json!([{"kind":"reconstructed","evidence":"F179 synthetic test policy"}]);
    let write = |name: &str, value: serde_json::Value| {
        fs::write(content.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    write(
        "npcs/synthetic.json",
        json!({"schema_version":1,"records":[
            {"npc_id":700,"name":"Synthetic NPC","zone_id":0,"position":[0.,0.,0.],
             "placement":"legacy_data","model_id":1,"roles":["vendor"],"vendor_stock":"synthetic"}
        ]}),
    );
    write(
        "npc_spawns_extra.json",
        json!({"schema_version":1,"records":[]}),
    );
    write(
        "zones.json",
        json!({"schema_version":1,"records":[{"id":0,"name":"Synthetic zone"}]}),
    );
    write(
        "vendors.json",
        json!({"schema_version":1,"vendors":[{"npc_id":700,"item_ids":[8]}]}),
    );
    write(
        "stores.json",
        json!({"schema_version":1,"marketplace":{
            "enabled_on_handshake":false,"orders_enabled":false,
            "payment_sources_enabled":false,"not_available_message":"disabled"},
            "items":[{"item_id":8,"purchase_price_coins":15,"member_discount_percent":null,
                      "member_price_coins":null,"sell_price_coins":7,"no_sale":false}]}),
    );
    write(
        "quests/synthetic.json",
        json!({"schema_version":1,"records":[{
            "id":701,"title_string_id":1,"giver_npc_id":700,"prerequisite_quest_ids":[],
            "objectives":[{"kind":"talk_to_npc","id":702,"npc_id":700,"source":provenance}],
            "rewards":[{"coins":7,"items":[],"job_xp":{},"source":provenance}],"source":provenance
        }]}),
    );
    write(
        "jobs.json",
        json!({"schema_version":1,"records":[{
            "id":"synthetic","name":"Synthetic job","profile_id":null,"category":"freestyle",
            "max_level":20,"members_only":false,"trainer_npc_ids":[],"quest_ids":[701],
            "abilities":[],"source":provenance
        }]}),
    );
    // 43 maximum-size rows are ~2.7 MiB per child: two children stay under the 8 MiB bundle
    // budget, the third pushes the split bundle over it.
    let ids: Vec<u32> = (0..43).map(|index| 930_000 + index).collect();
    let definitions: Vec<serde_json::Value> = ids
        .iter()
        .map(|id| {
            let mut bytes = (*id as i32).to_le_bytes().to_vec();
            bytes.resize(65_536, 0x5a);
            json!({
                "id": *id as i32, "slot": 0, "class_id": 1, "default_tint": 0, "max_stack": 10,
                "model": "", "texture": "", "tint_alias": "", "effect": 0, "wield_type": 0,
                "serialized_hex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                "sha256": format!("{:x}", Sha256::digest(&bytes))
            })
        })
        .collect();
    write(
        "item-catalogue.json",
        json!({"schema_version":1,"definitions":definitions}),
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
    let before = session.gateway.snapshot();

    let mut request = vec![165, 0, 2, 0];
    request.extend((ids.len() as i32).to_le_bytes());
    ids.iter().for_each(|id| request.extend(id.to_le_bytes()));
    let mut select = vec![26, 0, 10, 0];
    select.extend(700u64.to_le_bytes());
    select.extend(2i32.to_le_bytes());
    let children = [
        wrap(&select),
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
    assert_eq!(
        session.gateway.snapshot(),
        before,
        "the refused child changed state"
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
    assert_eq!(session.gateway.snapshot(), before);

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
    assert_eq!(session.gateway.snapshot(), before);
}

/// Session-test scratch directory and `Options` builder, shared with `lead_trade_commit`.
#[path = "test_support.rs"]
pub(crate) mod test_support;

/// T008: the opcode-164 jump body reaches exactly the sessions whose directed
/// view contains the sender, and the relay window is throttled per session.
#[test]
fn jump_relay_reaches_only_directed_viewers_and_is_throttled() {
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
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
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
    sessions.insert(recipient_key, recipient);
    sessions.insert(non_viewer_key, non_viewer);
    let jump_body = |guid: u64, position: [f32; 3], direction: [f32; 3], velocity: f32| {
        let mut inner = 164u16.to_le_bytes().to_vec();
        inner.extend(guid.to_le_bytes());
        for value in position.into_iter().chain(direction) {
            inner.extend(value.to_le_bytes());
        }
        inner.extend([2, 255]);
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
    assert_eq!((relay.state, relay.unknown), (2, 255));
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
    // 50 ms later the relay is throttled, but the validated pose still applies.
    sender
        .dispatch_message(
            &jump(1.0),
            &options,
            1_050,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            1_050,
        ))
        .is_empty(),
        "one relay per session window"
    );
    assert_eq!(sender.gateway.state().position[0], start[0] + 1.0);
    // The next window publishes the jump again.
    sender
        .dispatch_message(
            &jump(1.5),
            &options,
            1_100,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    let again = jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
        sessions.get_mut(&recipient_key).unwrap(),
        1_100,
    ));
    assert_eq!(again.len(), 1);
    assert_eq!(
        osfr_tools::jump_relay::decode(&again[0], character)
            .unwrap()
            .position[0],
        start[0] + 1.5
    );
    // A refusal is discarded before the staged transaction commits: the pose,
    // the world and the relay window are unchanged.
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
            "excessive velocity",
            jump_body(
                character,
                [start[0] + 2.0, start[1], start[2]],
                direction,
                osfr_tools::jump_relay::MAX_VERTICAL_SPEED + 1.0,
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
    // A refused jump consumed no relay window: this is 100 ms after the last
    // accepted jump, so the viewer still receives it.
    sender
        .dispatch_message(
            &jump(2.5),
            &options,
            1_200,
            &mut world,
            &mut membership,
            &mut sessions,
            None,
        )
        .unwrap();
    assert_eq!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            1_200,
        ))
        .len(),
        1,
        "a refusal did not consume the relay window"
    );
    // A later 125 in the same bundle supersedes the earlier 164 pose: observers
    // must not receive the stale jump after the final pose fanout.
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
    assert!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            1_300,
        ))
        .is_empty(),
        "a later 125 superseded the jump relay"
    );
    // A soft-over-speed jump is corrected for the sender and is not relayed.
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
    assert!(sender.gateway.state().position[0] < start[0] + 62.0);
    assert!(
        jump_bodies(super::f120_fishing_hour_reconnect::drain_replies(
            sessions.get_mut(&recipient_key).unwrap(),
            1_400,
        ))
        .is_empty(),
        "a clamped jump is not relayed"
    );
    assert!(
        super::f120_fishing_hour_reconnect::drain_replies(&mut sender, 1_400)
            .iter()
            .any(|message| message[7..].starts_with(&[38, 0, 12, 0])),
        "the clamped sender receives the location correction"
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
                1_500,
                &mut world,
                &mut membership,
                &mut sessions,
                None,
            )
            .is_err(),
        "a jump before Ready is refused"
    );
    assert_eq!(early.gateway.state(), &before_ready);
    drop(world);
    let _ = fs::remove_dir_all(&scratch);
}

/// F161: a pet adoption spends a certificate and adds a pet in one atomic document; a refused
/// replacement leaves the document and the live session unchanged. Declared here because the
/// hub file is at its line ceiling.
#[path = "f161_pet_adoption_persist.rs"]
mod f161_pet_adoption_persist;
