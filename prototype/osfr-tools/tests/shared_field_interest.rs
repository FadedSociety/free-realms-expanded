// Reuse W116's synthetic content, numeric accounts, Bot and serviced barriers.
// Its historical private socket check remains opt-in; W127's route runs below.
include!("shared_field_state.rs");

fn interest_fixture(passive: usize, exclude_pc: bool) -> Fixture {
    interest_fixture_with(passive, exclude_pc, |_| {})
}

fn interest_fixture_with(
    passive: usize,
    exclude_pc: bool,
    edit_content: impl FnOnce(&Path),
) -> Fixture {
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/reports/shared-world-w108/slice-3")
        .join(format!(
            "socket-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
    fs::create_dir_all(&root).unwrap();
    let accounts = create_account_store(&root);
    let content = create_content(&root);
    edit_content(&content);
    // Empty passive world except the requested cap fixture. No campaign edits.
    fs::write(
        content.join("npc_spawns_extra.json"),
        json!({"records":[]}).to_string(),
    )
    .unwrap();
    for row in fs::read_dir(content.join("npcs")).unwrap() {
        fs::remove_file(row.unwrap().path()).unwrap();
    }
    fs::write(
        content.join("npcs/0.json"),
        json!({"schema_version":1,"records":[{"npc_id":930108,"name":"930108","zone_id":0,"position":[200,0,200],"roles":["ambient"]}]})
            .to_string(),
    )
    .unwrap();
    fs::write(
        content.join("vendors.json"),
        json!({"schema_version":1,"vendors":[]}).to_string(),
    )
    .unwrap();
    let placements: Vec<_> = (0..passive).map(|i| json!({"guid":130000000000_u64+i as u64,"model_id":3,"position":[2,0,0],"heading":0,"scale":1})).collect();
    fs::write(
        content.join("npc_placements.json"),
        json!({"schema_version":1,"records":placements}).to_string(),
    )
    .unwrap();
    // No marker in this fixture: exactly one hostile and passive cap rows.
    let mut battles: serde_json::Value =
        serde_json::from_slice(&fs::read(content.join("field_battles.json")).unwrap()).unwrap();
    battles["battles"] = json!([]);
    fs::write(content.join("field_battles.json"), battles.to_string()).unwrap();
    let options = Options {
        bind: "127.0.0.1:0".parse().unwrap(),
        reference: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../lab/references/rust-gateway-reference"),
        state: root.join("state.json"),
        content: Some(content),
        identity: None,
        account_store: Some(accounts.clone()),
        gateway_ticket: String::new(),
        handoff_key: HANDOFF_KEY.to_vec(),
        seconds: 0,
        dev_tools: false,
        studio_port: 0,
        client_logs: None,
        terrain: Some({
            let directory = root.join("terrain");
            fs::create_dir_all(&directory).unwrap();
            let mut grid =
                osfr_tools::terrain::HeightGrid::new([-256., -256.], [256., 256.], 2.).unwrap();
            grid.add_triangle([[-256., 0., -256.], [256., 0., -256.], [256., 0., 256.]]);
            grid.add_triangle([[-256., 0., -256.], [256., 0., 256.], [-256., 0., 256.]]);
            fs::write(directory.join("zone_0.hgt"), grid.to_bytes()).unwrap();
            std::sync::Arc::new(osfr_tools::terrain::Terrain::load(&directory).unwrap())
        }),
        world: None,
        stock_text: None,
        allow_lan: false,
        security_log: None,
        group_invite_ttl_ms: osfr_tools::social_lists::DEFAULT_GROUP_INVITE_TTL_MS,
    };
    let (tx, rx) = mpsc::channel::<HostAction>();
    let (addr_tx, addr_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut server = Server::new(options).unwrap();
        if exclude_pc {
            server.test_pc_interest_limit(0);
        }
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
    Fixture {
        addr: addr_rx.recv_timeout(PATIENCE).unwrap(),
        tx: Some(tx),
        handle: Some(handle),
        root,
        accounts,
    }
}

/// The same synthetic field, with one guaranteed big-heart drop whose 0.5 m
/// pickup radius stays outside A's 1 m attack standoff.
fn pickup_fixture() -> Fixture {
    interest_fixture_with(0, false, |content| {
        let path = content.join("field_enemies.json");
        let mut rows: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        rows["enemies"][0]["power_up_table"] = serde_json::json!("boss");
        fs::write(&path, rows.to_string()).unwrap();
        let mut power_ups: serde_json::Value = serde_json::from_slice(
            &fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/power_ups.json"))
                .unwrap(),
        )
        .unwrap();
        power_ups["pickup"]["radius"] = serde_json::json!(0.5);
        fs::write(content.join("power_ups.json"), power_ups.to_string()).unwrap();
    })
}

fn interest_login(f: &Fixture, a: &mut bot::Bot, b: &mut bot::Bot) -> [Vec<Vec<u8>>; 2] {
    let store = AccountStore::load(&f.accounts).unwrap();
    for (guid, client) in [(101, &mut *a), (202, &mut *b)] {
        client
            .send_message(
                &bot::login_message(
                    guid,
                    &login::gateway_binding_ticket(HANDOFF_KEY, store.account(guid).unwrap(), guid),
                    VERSION,
                ),
                "1",
            )
            .unwrap();
        client.wait_for(PATIENCE, |_| true).unwrap();
        client.send_message(&packet(13, &[]), "13").unwrap();
        client.wait_for(PATIENCE, |_| true).unwrap();
    }
    a.send_message(&packet(10, &[]), "10").unwrap();
    let mut out = f.ready(101, a, b);
    b.send_message(&packet(10, &[]), "10").unwrap();
    let batch = f.ready(202, a, b);
    for i in 0..2 {
        out[i].extend(batch[i].clone());
    }
    let deadline = Instant::now() + PATIENCE;
    loop {
        let tail = collect(a, b, Duration::from_millis(20));
        for i in 0..2 {
            out[i].extend(tail[i].clone());
        }
        if f.host(|s| s.test_queues_idle()) {
            break;
        }
        assert!(Instant::now() < deadline, "startup reliable drain");
    }
    out
}

fn interest_cast(client: &mut bot::Bot) {
    let request = [
        vec![36, 0, 10, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        E.to_le_bytes().to_vec(),
    ]
    .concat();
    client.send_message(&bot::wrap(&request), "36/10").unwrap();
}
fn hits_from(packets: &[Vec<u8>], actor: u64) -> usize {
    packets
        .iter()
        .filter(|p| {
            is_sub(p, 32, 7)
                && p.get(4..12) == Some(actor.to_le_bytes().as_slice())
                && p.get(20..28) == Some(E.to_le_bytes().as_slice())
        })
        .count()
}

/// Power-up actor GUIDs in a batch. `PICKUP_GUID_BASE` in `encounter.rs`.
fn pickups(packets: &[Vec<u8>]) -> Vec<u64> {
    packets
        .iter()
        .filter(|p| is_sub(p, 35, 2))
        .filter_map(|p| {
            let guid = u64::from_le_bytes(p.get(4..12)?.try_into().ok()?);
            (guid & 0xffff_ffff_0000_0000 == 0x5057_5550_0000_0000).then_some(guid)
        })
        .collect()
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn w127_two_bot_observer_death_respawn_route() {
    let f = interest_fixture(0, false);
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let startup = interest_login(&f, &mut a, &mut b);
    for packets in &startup {
        assert_eq!(actor_packets(packets, 2, E), 1);
        assert_eq!(hp_values(packets), vec![300]);
    }
    let mut damage = [Vec::new(), Vec::new()];
    let enemy_hit_101 = |p: &Vec<u8>| {
        is_sub(p, 32, 7)
            && p.get(4..12) == Some(E.to_le_bytes().as_slice())
            && p.get(20..28) == Some(101_u64.to_le_bytes().as_slice())
    };
    settle(&mut a, &mut b, &mut damage, |d| {
        d[0].iter().any(|p| is_sub(p, 38, 1)) && d.iter().all(|m| m.iter().any(enemy_hit_101))
    });
    assert!(
        damage[0].iter().any(|p| is_sub(p, 38, 1)),
        "victim receives private HUD HP"
    );
    assert!(
        !damage[1].iter().any(|p| is_sub(p, 38, 1)),
        "observer must not receive victim HUD HP"
    );
    for packets in &damage {
        assert!(packets.iter().any(|p| is_sub(p, 32, 7)
            && p.get(4..12) == Some(E.to_le_bytes().as_slice())
            && p.get(20..28) == Some(101_u64.to_le_bytes().as_slice())));
    }
    let banks = interest_banks(&f);
    interest_cast(&mut a);
    let first = interest_committed_result(&f, &mut a, &mut b, 200, 0);
    for packets in &first {
        assert_eq!(hp_values(packets), vec![200]);
        assert_eq!(
            hits_from(packets, 101),
            1,
            "observer must know the real attacker PC"
        );
    }
    interest_cast(&mut b);
    let second = interest_committed_result(&f, &mut a, &mut b, 100, 1);
    for packets in &second {
        assert_eq!(hp_values(packets), vec![100]);
        assert_eq!(hits_from(packets, 202), 1);
    }
    let cooldown = collect(&mut a, &mut b, Duration::from_millis(500));
    for packets in &cooldown {
        assert!(hp_values(packets).is_empty());
    }
    interest_cast(&mut a);
    let lethal = interest_committed_result(&f, &mut a, &mut b, 0, 0);
    for packets in &lethal {
        assert_eq!(hp_values(packets), vec![0]);
        assert_eq!(hits_from(packets, 101), 1);
        assert_eq!(removals(packets, E), 1);
        let hp = packets
            .iter()
            .position(|p| is_sub(p, 35, 5) && p.get(4..12) == Some(E.to_le_bytes().as_slice()))
            .unwrap();
        let removed = packets
            .iter()
            .position(|p| is_sub(p, 35, 3) && p.get(6..14) == Some(E.to_le_bytes().as_slice()))
            .unwrap();
        assert!(hp < removed, "lethal HP precedes removal");
    }
    interest_respawn_and_range(&f, &mut a, &mut b);
    assert_eq!(
        interest_banks(&f),
        banks,
        "shared deaths must pay no credit/items/XP"
    );
    println!(
        "W127 socket: startup once; A/B/A HP [200,100,0] and real actors; death once; lethal retries silent; one full-HP respawn; range exit/removal and dead return. {}",
        f.root.display()
    );
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn t032_two_bot_private_instance_shared_death() {
    let f = interest_fixture(0, false);
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let _public_startup = interest_login(&f, &mut a, &mut b);
    assert_eq!(f.host(|s| s.test_field_health(E)), Some(300));
    let child = f.host(|s| s.test_field_child());
    assert_ne!(child, 0);
    f.host(move |s| s.test_transfer(101, child, false)).unwrap();
    let mut entered_a = f.ready(101, &mut a, &mut b);
    assert_eq!(f.host(move |s| s.test_field_health_in(child, E)), Some(300));
    settle(&mut a, &mut b, &mut entered_a, |messages| {
        actor_packets(&messages[0], 2, E) == 1
            && hp_values(&messages[0]) == vec![300]
            && f.host(|s| s.test_known_npcs(101).contains(&E))
    });
    assert_eq!(actor_packets(&entered_a[0], 2, E), 1);
    assert_eq!(hp_values(&entered_a[0]), vec![300]);
    assert_eq!(f.host(|s| s.test_location(101).unwrap().1), child);
    let child_banks = interest_banks(&f);
    interest_cast(&mut a);
    let first = interest_committed_result_in(&f, child, &mut a, &mut b, 200, 0);
    assert_eq!(hp_values(&first[0]), vec![200]);
    assert_eq!(hits_from(&first[0], 101), 1);
    assert!(hp_values(&first[1]).is_empty());

    // The second member joins only after the first hit. Its initial actor
    // snapshot must use the committed HP and must not reset the child field.
    f.host(move |s| s.test_transfer(202, child, false)).unwrap();
    let mut entered_b = f.ready(202, &mut a, &mut b);
    assert_eq!(
        f.host(move |s| s.test_field_health_in(child, E)),
        Some(200),
        "joining must preserve damage already committed in the child"
    );
    settle(&mut a, &mut b, &mut entered_b, |messages| {
        actor_packets(&messages[1], 2, E) == 1
            && hp_values(&messages[1]) == vec![200]
            && f.host(|s| s.test_known_npcs(202).contains(&E))
    });
    assert_eq!(actor_packets(&entered_b[1], 2, E), 1);
    assert_eq!(hp_values(&entered_b[1]), vec![200]);
    assert_eq!(f.host(|s| s.test_location(202).unwrap().1), child);

    interest_cast(&mut b);
    let second = interest_committed_result_in(&f, child, &mut a, &mut b, 100, 1);
    let cooldown = collect(&mut a, &mut b, Duration::from_millis(500));
    for packets in cooldown {
        assert!(hp_values(&packets).is_empty());
    }
    interest_cast(&mut a);
    let lethal = interest_committed_result_in(&f, child, &mut a, &mut b, 0, 0);
    for (batch, hp, attacker) in [(&second, 100, 202), (&lethal, 0, 101)] {
        for packets in batch {
            assert_eq!(hp_values(packets), vec![hp]);
            assert_eq!(hits_from(packets, attacker), 1);
        }
    }
    for packets in &lethal {
        assert_eq!(removals(packets, E), 1);
    }
    assert_eq!(
        f.host(|s| s.test_field_instances()),
        [0, child],
        "public and child runtimes remain independently addressable"
    );
    assert_eq!(
        f.host(|s| s.test_field_health(E)),
        Some(300),
        "the public population is a distinct runtime"
    );
    assert_eq!(
        interest_banks(&f),
        child_banks,
        "shared deaths still pay no personal credit or items"
    );
    println!(
        "T032 socket: first hit at 200, late second member kept 200, then HP 100/0 and one death each; public field remained full"
    );
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn f132_not_ready_same_instance_removes_known_field_actor() {
    let f = interest_fixture(0, false);
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let _public_startup = interest_login(&f, &mut a, &mut b);
    assert!(f.host(|s| s.test_known_npcs(101).contains(&E)));
    assert!(f.host(|s| s.test_known_npcs(202).contains(&E)));
    let before = f.host(|s| s.test_location(202).unwrap());
    assert!(before.2, "observer starts ready");

    // Readiness alone: the session's world stays loaded, so its own commit
    // re-derives readiness on the next tick and the actor then returns.
    f.host(|s| s.test_set_ready(202, false));
    let after = f.host(|s| s.test_location(202).unwrap());
    assert_eq!(after.1, before.1, "readiness changed in the same instance");

    let mut cycle = [Vec::new(), Vec::new()];
    settle(&mut a, &mut b, &mut cycle, |messages| {
        removals(&messages[1], E) >= 1 && actor_packets(&messages[1], 2, E) >= 1
    });
    assert_eq!(removals(&cycle[1], E), 1);
    assert_eq!(removals(&cycle[0], E), 0);
    let removal = cycle[1]
        .iter()
        .find(|packet| {
            is_sub(packet, 35, 3) && packet.get(6..14) == Some(E.to_le_bytes().as_slice())
        })
        .unwrap();
    let mut expected = vec![35_u8, 0, 3, 0];
    expected.extend(1_u16.to_le_bytes());
    expected.extend(E.to_le_bytes());
    expected.extend([0; 17]);
    assert_eq!(removal, &expected, "35/3 field layout");
    // Readiness returned: the current actor was re-added at its current HP.
    assert_eq!(actor_packets(&cycle[1], 2, E), 1);
    assert_eq!(hp_values(&cycle[1]), vec![300]);
    assert!(f.host(|s| s.test_location(202).unwrap()).2);
    assert!(f.host(|s| s.test_known_npcs(202).contains(&E)));
    assert!(f.host(|s| s.test_known_npcs(101).contains(&E)));

    // World-load loss alone: membership readiness stays true.
    f.host(|s| s.test_lose_world(202));
    let loaded_loss = f.host(|s| s.test_location(202).unwrap());
    assert!(loaded_loss.2, "membership readiness is unchanged");
    let mut lost = [Vec::new(), Vec::new()];
    settle(&mut a, &mut b, &mut lost, |messages| {
        removals(&messages[1], E) >= 1
    });
    assert_eq!(
        removals(&lost[1], E),
        1,
        "losing the loaded world removes its ghost actors"
    );
    assert_eq!(removals(&lost[0], E), 0);
    assert!(!f.host(|s| s.test_known_npcs(202).contains(&E)));
    println!(
        "F132 socket: readiness-only and world-load-only loss each removed the stale actor once; readiness restore re-added it; ready observer unchanged"
    );
}

fn removals(packets: &[Vec<u8>], guid: u64) -> usize {
    packets
        .iter()
        .filter(|p| is_sub(p, 35, 3) && p.get(6..14) == Some(guid.to_le_bytes().as_slice()))
        .count()
}

fn interest_engine() -> osfr_tools::encounter::EncounterEngine {
    let dir = unit::fixture();
    let mut engine = osfr_tools::encounter::EncounterEngine::load(&dir, None).unwrap();
    engine
        .load_field_enemies(&dir.join("field_enemies.json"))
        .unwrap();
    engine
}

fn scene_zone(id: u32) -> osfr_tools::content::Zone {
    osfr_tools::content::Zone {
        id,
        name: id.to_string(),
        geometry_id: Some(id),
        zone_type: Some(2),
        tutorial: Some(false),
        sky: Some(String::new()),
        source: Vec::new(),
    }
}

#[test]
fn w127_exact_death_respawn_clock_and_no_personal_credit() {
    let mut a = interest_engine();
    let mut b = interest_engine();
    let mut field = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let mut views = [
        std::collections::BTreeSet::new(),
        std::collections::BTreeSet::new(),
    ];
    let visibility = osfr_tools::world::NpcVisibility::default();
    for view in &mut views {
        let (next, _) = visibility
            .shared_view(0, [1., 0., 0.], &field.interest_candidates(false), view)
            .unwrap();
        let init: Vec<_> = next
            .difference(view)
            .flat_map(|guid| field.initialization(*guid).unwrap())
            .collect();
        assert_eq!(actor_packets(&init, 2, E), 1);
        assert_eq!(hp_values(&init), vec![300]);
        *view = next;
    }
    for (who, now, hp) in [(101, 0, 200), (202, 100, 100), (101, 600, 0)] {
        let engine = if who == 101 { &mut a } else { &mut b };
        let out = engine
            .shared_attack(&mut field, who, E, [1., 0., 0.], 2, now)
            .unwrap();
        assert_eq!(hp_values(&out.replies), vec![hp]);
        assert_eq!(hits_from(&out.replies, who), 1);
        assert!(out.defeated.is_empty() && out.rewards.is_empty());
    }
    assert_eq!(field.respawn_at(E), Some(2600));
    for engine in [&mut a, &mut b] {
        let out = engine
            .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 1200)
            .unwrap();
        assert!(out.replies.is_empty());
        assert!(out.defeated.is_empty() && out.rewards.is_empty());
    }
    assert_eq!(field.respawn_at(E), Some(2600));
    for view in &mut views {
        let (next, packets) = visibility
            .shared_view(0, [20., 0., 0.], &field.interest_candidates(false), view)
            .unwrap();
        assert_eq!(removals(&packets, E), 1);
        *view = next;
        let (_, retry) = visibility
            .shared_view(0, [20., 0., 0.], &field.interest_candidates(false), view)
            .unwrap();
        assert!(retry.is_empty());
    }
    // Exact boundary uses the existing deterministic simulation clock.
    a.shared_motion(&mut field, &[], 2599, 100).unwrap();
    assert_eq!(field.health(E), Some(0));
    assert!(field.initialization(E).unwrap().is_empty());
    a.shared_motion(&mut field, &[], 2600, 1).unwrap();
    assert_eq!(field.health(E), Some(300));
    assert_eq!(field.respawn_at(E), None);
    for view in &mut views {
        let (next, _) = visibility
            .shared_view(0, [20., 0., 0.], &field.interest_candidates(false), view)
            .unwrap();
        let init: Vec<_> = next
            .difference(view)
            .flat_map(|guid| field.initialization(*guid).unwrap())
            .collect();
        assert_eq!(actor_packets(&init, 2, E), 1);
        assert_eq!(hp_values(&init), vec![300]);
        *view = next;
        let (again, packets) = visibility
            .shared_view(0, [20., 0., 0.], &field.interest_candidates(false), view)
            .unwrap();
        assert_eq!(&again, view);
        assert!(packets.is_empty());
    }
    println!(
        "W127 exact-clock: death 600; 2599 no spawn; 2600 one AddNpc/full HP per observer; replay no death/credit"
    );
}

#[test]
fn t032_private_instance_field_is_one_shared_death_for_two_members() {
    use osfr_tools::world::{InstanceKind, World, WorldPose};

    let mut a = interest_engine();
    let mut b = interest_engine();
    let seed = a.take_shared_fields().unwrap();
    b.take_shared_fields();
    let mut world = World::runtime();
    world.install_field_seed(0, seed);
    let child = world
        .create_child_instance(0, 0, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    assert_ne!(child, 0);
    assert!(
        world.field(child).is_none(),
        "private field starts unseeded"
    );
    let first = world
        .prepare_membership(101, 0, Some(child), WorldPose::ORIGIN, true, 0)
        .unwrap();
    world.commit_membership(first);
    let mut field = world.field(child).unwrap().clone();
    let first_hit = a
        .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 0)
        .unwrap();
    assert_eq!(hp_values(&first_hit.replies), vec![200]);
    world.install_field(child, field.clone());
    let second = world
        .prepare_membership(202, 0, Some(child), WorldPose::ORIGIN, true, 0)
        .unwrap();
    assert_eq!(
        world.field(child).unwrap().health(E),
        Some(200),
        "a second member must not reset the private field"
    );
    world.commit_membership(second);
    field = world.field(child).unwrap().clone();
    assert_eq!(
        world.field(child).unwrap().health(E),
        Some(200),
        "committing the second member must preserve damaged HP"
    );
    assert_eq!(
        world.field_instances(),
        [0, child],
        "one runtime for public and one for the joined child"
    );
    let mut views = [
        std::collections::BTreeSet::new(),
        std::collections::BTreeSet::new(),
    ];
    let visibility = osfr_tools::world::NpcVisibility::default();
    for (view, at) in views.iter_mut().zip([[1., 0., 0.], [-1., 0., 0.]]) {
        let (next, _) = visibility
            .shared_view(0, at, &field.interest_candidates(false), view)
            .unwrap();
        let init: Vec<_> = next
            .difference(view)
            .flat_map(|guid| field.initialization(*guid).unwrap())
            .collect();
        assert_eq!(actor_packets(&init, 2, E), 1);
        assert_eq!(hp_values(&init), vec![200]);
        *view = next;
    }
    for (who, now, hp) in [(202, 100, 100), (101, 600, 0)] {
        let engine = if who == 101 { &mut a } else { &mut b };
        let out = engine
            .shared_attack(&mut field, who, E, [1., 0., 0.], 2, now)
            .unwrap();
        assert_eq!(hp_values(&out.replies), vec![hp]);
        assert_eq!(hits_from(&out.replies, who), 1);
        assert!(out.defeated.is_empty() && out.rewards.is_empty());
        world.install_field(child, field.clone());
        for member in [101, 202] {
            let (instance, shared) = world.field_for_player(member, 0).unwrap();
            assert_eq!(instance, child);
            assert_eq!(shared.health(E), Some(hp));
        }
    }
    for (view, at) in views.iter_mut().zip([[1., 0., 0.], [-1., 0., 0.]]) {
        let (next, packets) = visibility
            .shared_view(0, at, &field.interest_candidates(false), view)
            .unwrap();
        assert_eq!(removals(&packets, E), 1);
        assert!(next.is_empty());
    }
    assert_eq!(
        world.field(0).unwrap().health(E),
        Some(300),
        "private combat must not alter the public population"
    );
    println!(
        "T032 private instance: HP [200,100,0] across two members, one death removal each, public field unchanged"
    );
}

#[test]
fn t032_a_seeded_nonzero_zone_enters_one_shared_public_instance() {
    use osfr_tools::world::{InstanceKind, World, WorldPose};

    let mut a = interest_engine();
    let seed = a.take_shared_fields().unwrap();
    let mut world = World::runtime();
    world.register_runtime_zone(scene_zone(9600)).unwrap();
    world.install_field_seed(9600, seed);
    let first = world
        .prepare_membership(101, 9600, None, WorldPose::ORIGIN, true, 0)
        .unwrap();
    let instance = first.instance_id();
    assert_ne!(instance, 0, "a nonzero seeded zone keeps a distinct id");
    world.commit_membership(first);
    assert_eq!(
        world.instance(instance).unwrap().kind,
        InstanceKind::Public,
        "every seeded zone's first entrant joins its shared public instance"
    );
    assert_eq!(world.public_instance(9600), Some(instance));
    let mut field = world.field(instance).unwrap().clone();
    let hit = a
        .shared_attack(&mut field, 101, E, [1., 0., 0.], 2, 0)
        .unwrap();
    assert_eq!(hp_values(&hit.replies), vec![200]);
    world.install_field(instance, field);
    let second = world
        .prepare_membership(202, 9600, None, WorldPose::ORIGIN, true, 0)
        .unwrap();
    assert_eq!(
        second.instance_id(),
        instance,
        "the second entrant must share the same public instance, not get a solo child"
    );
    world.commit_membership(second);
    assert_eq!(
        world.field(instance).unwrap().health(E),
        Some(200),
        "committing the second member must preserve the shared damaged HP"
    );
    assert_eq!(world.field_instances(), [instance]);
    assert_eq!(world.instance(instance).unwrap().members, [101, 202].into());
    println!("T032 nonzero seeded zone: one public instance, shared field, late join keeps HP 200");
}

#[test]
fn t032_a_seeded_zone_is_seeded_by_public_entry() {
    use osfr_tools::world::{World, WorldPose};

    let mut a = interest_engine();
    let seed = a.take_shared_fields().unwrap();
    let mut world = World::runtime();
    world.register_runtime_zone(scene_zone(9600)).unwrap();
    world.install_field_seed(9600, seed);
    let entered = world.enter_public(101, 9600, WorldPose::ORIGIN, 0).unwrap();
    assert_eq!(
        world.instance(entered.instance_id).unwrap().kind,
        osfr_tools::world::InstanceKind::Public
    );
    assert!(
        world.field(entered.instance_id).is_some(),
        "entering a seeded public instance must install its field runtime, not only membership commit"
    );
}

#[test]
fn t032_a_private_child_never_aliases_a_seeded_public_instance() {
    use osfr_tools::world::{InstanceKind, World, WorldPose};

    let mut a = interest_engine();
    let seed = a.take_shared_fields().unwrap();
    let mut world = World::runtime();
    // The free-id cursor gives this child id 1 before zone 1 is registered.
    let child = world
        .create_child_instance(0, 0, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    assert_eq!(child, 1);
    world.register_runtime_zone(scene_zone(1)).unwrap();
    world.install_field_seed(1, seed);
    let entered = world.enter_public(101, 1, WorldPose::ORIGIN, 0).unwrap();
    assert_ne!(
        entered.instance_id, child,
        "the public instance must take a free id, not replace the private child"
    );
    assert_eq!(
        world.instance(child).unwrap().kind,
        InstanceKind::Encounter { encounter_id: 0 },
        "the private child keeps its kind and id"
    );
    assert_eq!(
        world.instance(entered.instance_id).unwrap().kind,
        InstanceKind::Public
    );
}

#[test]
fn t032_a_seeded_zone_takes_a_free_public_id_when_a_child_holds_the_old_one() {
    use osfr_tools::world::{InstanceKind, World, WorldPose};

    let mut a = interest_engine();
    let seed = a.take_shared_fields().unwrap();
    let mut world = World::runtime();
    let child = world
        .create_child_instance(0, 0, InstanceKind::Encounter { encounter_id: 0 }, 0)
        .unwrap();
    assert_eq!(child, 1);
    world.register_runtime_zone(scene_zone(1)).unwrap();
    world.install_field_seed(1, seed);
    let first = world
        .prepare_membership(101, 1, None, WorldPose::ORIGIN, true, 0)
        .unwrap();
    let public = first.instance_id();
    assert_ne!(
        public, child,
        "a taken id is never aliased as the seeded public instance"
    );
    world.commit_membership(first);
    assert_eq!(world.public_instance(1), Some(public));
    let second = world
        .prepare_membership(202, 1, None, WorldPose::ORIGIN, true, 0)
        .unwrap();
    assert_eq!(
        second.instance_id(),
        public,
        "a second entrant shares the zone's public instance"
    );
    world.commit_membership(second);
    assert_eq!(
        world.instance(child).unwrap().kind,
        InstanceKind::Encounter { encounter_id: 0 },
        "the private child was not aliased as public space"
    );
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn t032_same_zone_transfer_discards_the_pickups_the_client_left_behind() {
    let f = pickup_fixture();
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let _startup = interest_login(&f, &mut a, &mut b);
    let child = f.host(|s| s.test_field_child());
    assert_ne!(child, 0);
    // A kills E with the guaranteed big heart. A stands 1 m away and the drop
    // radius is 0.5 m, so the pickup is still on the ground after commit.
    let mut kill = [Vec::new(), Vec::new()];
    for (index, hp) in [200, 100, 0].into_iter().enumerate() {
        interest_cast(&mut a);
        let batch = interest_committed_result(&f, &mut a, &mut b, hp, 0);
        for i in 0..2 {
            kill[i].extend(batch[i].clone());
        }
        if index < 2 {
            // One player's basic-swing cooldown; the wait is not a pace, the
            // server refuses a faster second swing.
            let _ = collect(&mut a, &mut b, Duration::from_millis(700));
        }
    }
    let dropped = pickups(&kill[0]);
    assert_eq!(dropped.len(), 1, "the lethal kill spawns one pickup actor");
    let pickup = dropped[0];
    assert_eq!(f.host(|s| s.test_pickup_count(101)), Some(1));
    // A refused transfer clears only its staged clone, so location, membership
    // and the ground drop stay exactly as they were.
    let before = f.host(|s| s.test_location(101).unwrap());
    assert!(f.host(move |s| s.test_transfer(101, child, true)).is_err());
    assert_eq!(f.host(|s| s.test_location(101).unwrap()), before);
    assert_eq!(
        f.host(|s| s.test_pickup_count(101)),
        Some(1),
        "a refused transfer changes nothing"
    );
    // The same-zone transfer is a client scene reload: the drop is not moved
    // into the new instance and cannot pay out there.
    f.host(move |s| s.test_transfer(101, child, false)).unwrap();
    assert_eq!(f.host(|s| s.test_location(101).unwrap()).1, child);
    assert_eq!(
        f.host(|s| s.test_pickup_count(101)),
        Some(0),
        "the old instance's ground pickups are gone from the reloaded client"
    );
    let _entered = f.ready(101, &mut a, &mut b);
    f.host(|s| s.test_observer_position(101, [0., 0., 0.]));
    let after = collect(&mut a, &mut b, Duration::from_millis(500));
    let ghost =
        |p: &Vec<u8>| is_sub(p, 35, 3) && p.get(6..14) == Some(pickup.to_le_bytes().as_slice());
    assert!(
        !after.iter().flatten().any(ghost),
        "the stale pickup must not be collected in the new instance"
    );
}

#[test]
fn w127_common_budget_cutoff_engagement_and_exclusion() {
    let mut engine = interest_engine();
    let mut field = engine.take_shared_fields().unwrap();
    let mut visibility = osfr_tools::world::NpcVisibility::default();
    for i in 0..64 {
        visibility.add_static_actor(130000000000 + i, "", 0, 3, [2., 0., 0.], 0);
    }
    visibility.exclude_actor_guids([E]); // Shared E still has its independent writer.
    let empty = std::collections::BTreeSet::new();
    let (a, _) = visibility
        .shared_view(0, [1., 0., 0.], &field.interest_candidates(false), &empty)
        .unwrap();
    let (b, _) = visibility
        .shared_view(0, [-1., 0., 0.], &field.interest_candidates(false), &empty)
        .unwrap();
    assert_eq!(a.len(), 64);
    assert!(!a.contains(&E));
    assert_eq!(b.len(), 64);
    assert!(b.contains(&E));
    engine
        .shared_attack(&mut field, 202, E, [-1., 0., 0.], 2, 0)
        .unwrap();
    let (engaged, _) = visibility
        .shared_view(0, [1., 0., 0.], &field.interest_candidates(false), &a)
        .unwrap();
    assert_eq!(engaged.len(), 64);
    assert!(!engaged.contains(&E));
    let (engaged, _) = visibility
        .shared_view(0, [-1., 0., 0.], &field.interest_candidates(false), &b)
        .unwrap();
    assert!(engaged.contains(&E));
    let init = field.initialization(E).unwrap();
    assert_eq!(hp_values(&init), vec![200]);
    let (far, removed) = visibility
        .shared_view(
            0,
            [101., 0., 0.],
            &field.interest_candidates(false),
            &engaged,
        )
        .unwrap();
    assert!(!far.contains(&E));
    assert_eq!(removals(&removed, E), 1);
    assert!(
        visibility
            .shared_view(0, [f32::NAN, 0., 0.], &[], &empty)
            .is_err()
    );
}

#[test]
fn w127_grid_candidates_are_not_directed_pc_views() {
    use osfr_tools::players::{InterestPolicy, PlayerRegistry, SessionId, ZoneInstance};
    let mut registry = PlayerRegistry::new(InterestPolicy::new(100., 0).unwrap());
    let public = ZoneInstance {
        zone_id: 0,
        instance_id: 0,
    };
    let private = ZoneInstance {
        zone_id: 0,
        instance_id: 7,
    };
    registry
        .join(SessionId(10801), 101, 101, public, [1., 0., 0.], 0.)
        .unwrap();
    registry
        .join(SessionId(10802), 202, 202, public, [-1., 0., 0.], 0.)
        .unwrap();
    registry
        .join(SessionId(10803), 303, 303, private, [0., 0., 0.], 0.)
        .unwrap();
    assert_eq!(
        registry.nearby_sessions(public, [0., 0., 0.]),
        [SessionId(10801), SessionId(10802)].into()
    );
    assert!(!registry.knows_character(SessionId(10802), 101));
    assert!(
        registry
            .nearby_sessions(public, [f32::INFINITY, 0., 0.])
            .is_empty()
    );
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn w127_two_bot_attacker_pc_excluded() {
    let f = interest_fixture(0, true);
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let startup = interest_login(&f, &mut a, &mut b);
    for packets in &startup {
        assert_eq!(actor_packets(packets, 2, E), 1);
        assert_eq!(hp_values(packets), vec![300]);
    }
    interest_cast(&mut a);
    let first = interest_committed_result(&f, &mut a, &mut b, 200, 0);
    interest_cast(&mut b);
    let second = interest_committed_result(&f, &mut a, &mut b, 100, 1);
    let _ = collect(&mut a, &mut b, Duration::from_millis(500));
    interest_cast(&mut a);
    let lethal = interest_committed_result(&f, &mut a, &mut b, 0, 0);
    for (batch, hp) in [(&first, 200), (&second, 100), (&lethal, 0)] {
        for packets in batch {
            assert_eq!(hp_values(packets), vec![hp]);
        }
    }
    assert_eq!(hits_from(&first[1], 101), 0);
    assert_eq!(hits_from(&second[0], 202), 0);
    assert_eq!(hits_from(&lethal[1], 101), 0);
    for packets in &lethal {
        assert_eq!(removals(packets, E), 1);
        assert!(
            !packets
                .iter()
                .any(|p| is_sub(p, 32, 7) && p.get(4..12) == Some(1_u64.to_le_bytes().as_slice()))
        );
    }
    interest_respawn_and_range(&f, &mut a, &mut b);
    println!("W127 PC cap 0: enemy HP/death converge; unknown-PC feedback omitted");
}

#[test]
#[ignore = "requires private OSFR_PLAY_PACKAGE; synthetic accounts/content only"]
fn w127_two_bot_npc_cap_cutoff() {
    let f = interest_fixture(64, false);
    let mut a = f.bot(101, 10801);
    let mut b = f.bot(202, 10802);
    let startup = interest_login(&f, &mut a, &mut b);
    assert_eq!(actor_packets(&startup[0], 2, E), 0);
    assert_eq!(actor_packets(&startup[1], 2, E), 1);
    for packets in &startup {
        assert_eq!(
            packets
                .iter()
                .filter(|p| is_sub(p, 35, 2)
                    && p.get(4..12)
                        .and_then(|b| b.try_into().ok())
                        .map(u64::from_le_bytes)
                        .is_some_and(
                            |guid| guid == E || (130000000000..130000000064).contains(&guid)
                        ))
                .count(),
            64
        );
    }
    interest_cast(&mut a);
    let refused = collect(&mut a, &mut b, Duration::from_millis(500));
    for packets in &refused {
        assert!(hp_values(packets).is_empty());
    }
    assert_eq!(f.host(|s| s.test_field_health(E)), Some(300));
    interest_cast(&mut b);
    let first = interest_committed_result(&f, &mut a, &mut b, 200, 1);
    assert_eq!(actor_packets(&first[0], 2, E), 0);
    assert!(hp_values(&first[0]).is_empty());
    assert_eq!(hp_values(&first[1]), vec![200]);
    f.host(|s| s.test_observer_position(101, [-1., 0., 0.]));
    let mut entered = [Vec::new(), Vec::new()];
    settle(&mut a, &mut b, &mut entered, |e| {
        actor_packets(&e[0], 2, E) >= 1 && removals(&e[0], 130000000063) >= 1
    });
    assert_eq!(actor_packets(&entered[0], 2, E), 1);
    assert_eq!(hp_values(&entered[0]), vec![200]);
    assert_eq!(
        removals(&entered[0], 130000000063),
        1,
        "entry displaces one passive row"
    );
    interest_cast(&mut a);
    let second = interest_committed_result(&f, &mut a, &mut b, 100, 0);
    for packets in &second {
        assert_eq!(hp_values(packets), vec![100]);
    }
    let _ = collect(&mut a, &mut b, Duration::from_millis(500));
    interest_cast(&mut b);
    let lethal = interest_committed_result(&f, &mut a, &mut b, 0, 1);
    for packets in &lethal {
        assert_eq!(hp_values(packets), vec![0]);
        assert_eq!(removals(packets, E), 1);
    }
    interest_respawn_and_range(&f, &mut a, &mut b);
    println!(
        "W127 NPC cap 64: unknown-target cast refused; engaged E replaces one passive; current HP then shared death"
    );
}

/// Longest a due respawn may sit undelivered once the server clock passed it. It guards
/// "the tick delivers it", not a latency target: 24 parallel suites measured >2 s stalls (W157).
const RESPAWN_LATENESS_MS: u64 = 5_000;

fn interest_respawn_and_range(f: &Fixture, a: &mut bot::Bot, b: &mut bot::Bot) {
    let (_, deadline) = f.host(|s| s.test_field_clock(E));
    let deadline = deadline.unwrap();
    interest_cast(a);
    interest_cast(b);
    // Keep both out of aggro during corpse/respawn assertions.
    f.host(|s| {
        s.test_observer_position(101, [-20., 0., 0.]);
        s.test_observer_position(202, [-20., 0., 0.]);
    });
    let retry = collect(a, b, Duration::from_millis(500));
    for packets in &retry {
        assert!(hp_values(packets).is_empty());
        assert_eq!(removals(packets, E), 0);
        assert_eq!(actor_packets(packets, 2, E), 0);
    }
    let mut respawn = [Vec::new(), Vec::new()];
    let limit = Instant::now() + PATIENCE;
    loop {
        let batch = collect(a, b, Duration::from_millis(5));
        let now = f.host(|s| s.test_field_clock(E).0);
        for i in 0..2 {
            respawn[i].extend(batch[i].clone());
            if now < deadline {
                assert_eq!(actor_packets(&respawn[i], 2, E), 0, "early respawn");
            }
        }
        if respawn.iter().all(|p| actor_packets(p, 2, E) == 1) {
            assert!(
                now <= deadline + RESPAWN_LATENESS_MS,
                "respawn exceeded serviced delivery tolerance"
            );
            break;
        }
        assert!(Instant::now() < limit, "due respawn not delivered");
    }
    let tail = collect(a, b, Duration::from_millis(500));
    for i in 0..2 {
        respawn[i].extend(tail[i].clone());
        assert_eq!(actor_packets(&respawn[i], 2, E), 1);
        assert_eq!(hp_values(&respawn[i]), vec![300]);
    }
    // Second engagement: B leaves while E lives, then returns during death.
    f.host(|s| {
        s.test_observer_position(101, [-1., 0., 0.]);
        s.test_observer_position(202, [-101., 0., 0.]);
    });
    let mut left = [Vec::new(), Vec::new()];
    settle(a, b, &mut left, |l| removals(&l[1], E) >= 1);
    assert_eq!(removals(&left[1], E), 1);
    assert_eq!(f.host(|s| s.test_field_health(E)), Some(300));
    assert!(
        f.host(|s| s.test_known_npcs(101)).contains(&E),
        "A lost E before second engagement"
    );
    for (who, hp) in [(0, 200), (0, 100), (0, 0)] {
        interest_cast(a);
        let delta = interest_committed_result(f, a, b, hp, who);
        assert_eq!(hp_values(&delta[0]), vec![hp]);
        assert!(hp_values(&delta[1]).is_empty());
        assert_eq!(hits_from(&delta[1], 101), 0);
        if hp > 0 {
            let silent = collect(a, b, Duration::from_millis(500));
            assert!(hp_values(&silent[1]).is_empty());
        }
    }
    f.host(|s| {
        s.test_observer_position(101, [-20., 0., 0.]);
        s.test_observer_position(202, [-20., 0., 0.]);
    });
    let returned = collect(a, b, Duration::from_millis(500));
    assert_eq!(actor_packets(&returned[1], 2, E), 0);
    assert!(hp_values(&returned[1]).is_empty());
    assert_eq!(removals(&returned[1], E), 0);
}

fn interest_committed_result(
    f: &Fixture,
    a: &mut bot::Bot,
    b: &mut bot::Bot,
    hp: u32,
    recipient: usize,
) -> [Vec<Vec<u8>>; 2] {
    interest_committed_result_in(f, 0, a, b, hp, recipient)
}

fn interest_committed_result_in(
    f: &Fixture,
    instance: i32,
    a: &mut bot::Bot,
    b: &mut bot::Bot,
    hp: u32,
    recipient: usize,
) -> [Vec<Vec<u8>>; 2] {
    let mut packets = [Vec::new(), Vec::new()];
    let deadline = Instant::now() + PATIENCE;
    loop {
        let batch = collect(a, b, Duration::from_millis(5));
        for i in 0..2 {
            packets[i].extend(batch[i].clone());
        }
        if f.host(move |s| s.test_field_health_in(instance, E)) == Some(hp) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "shared HP commit missing: expected {hp}, A location {:?}, A knows {}",
            f.host(|s| s.test_location(101).unwrap().0.position),
            f.host(|s| s.test_known_npcs(101)).contains(&E)
        );
    }
    let delivery_deadline = Instant::now() + PATIENCE;
    while hp_values(&packets[recipient]).is_empty() {
        let batch = collect(a, b, Duration::from_millis(5));
        for i in 0..2 {
            packets[i].extend(batch[i].clone());
        }
        assert!(
            Instant::now() < delivery_deadline,
            "committed HP delivery missing"
        );
    }
    drain_deliveries(f, a, b, &mut packets);
    packets
}

fn interest_banks(f: &Fixture) -> Vec<serde_json::Value> {
    [101,202].into_iter().map(|guid| {
        let save:serde_json::Value=serde_json::from_slice(&fs::read(f.root.join(format!("characters/{guid}.json"))).unwrap()).unwrap();
        json!({"inventory":save["gameplay"]["inventory"],"experience":save["gameplay"]["profiles"]["experience"],"quests":save["gameplay"]["npc_quests"]})
    }).collect()
}

#[test]
fn w127_staged_passive_content_edits_do_not_leak() {
    let mut live = osfr_tools::world::NpcVisibility::default();
    live.add_static_actor(130000000001, "", 0, 3, [0., 0., 0.], 0);
    let mut staged = live.clone();
    assert!(staged.remove(130000000001));
    staged.add_static_actor(130000000002, "", 0, 3, [0., 0., 0.], 0);
    let empty = std::collections::BTreeSet::new();
    let (before, _) = live.shared_view(0, [0., 0., 0.], &[], &empty).unwrap();
    let (after, _) = staged.shared_view(0, [0., 0., 0.], &[], &empty).unwrap();
    assert_eq!(before, [130000000001].into());
    assert_eq!(after, [130000000002].into());
}

#[test]
fn f160_hidden_view_forgets_every_known_actor_in_the_shared_field_layout() {
    let known = std::collections::BTreeSet::from([E, E + 1]);
    let (wanted, packets) = osfr_tools::world::NpcVisibility::hidden_view(&known);
    assert!(wanted.is_empty(), "a hidden viewer keeps no actor");
    let expected: Vec<_> = [E, E + 1]
        .iter()
        .map(|guid| {
            let mut bytes = vec![35_u8, 0, 3, 0];
            bytes.extend(1_u16.to_le_bytes());
            bytes.extend(guid.to_le_bytes());
            bytes.extend([0; 17]);
            bytes
        })
        .collect();
    assert_eq!(packets, expected, "one 35/3 removal per known actor");
    let (wanted, packets) = osfr_tools::world::NpcVisibility::hidden_view(&Default::default());
    assert!(wanted.is_empty() && packets.is_empty());
}
