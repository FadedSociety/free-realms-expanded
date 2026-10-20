use super::*;
use osfr_tools::inventory::{InventoryItem, InventoryState};
use osfr_tools::players::PlayerPresentation;
use osfr_tools::trade::{TradeInventory, TradeItemDefinition};
use std::collections::{BTreeMap, BTreeSet};

const A: u64 = 101;
const B: u64 = 202;

struct Rig {
    a: Session,
    b: Session,
    world: SharedWorld,
    scratch: PathBuf,
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

fn row(id: i32, definition_id: i32, count: u32) -> InventoryItem {
    InventoryItem {
        definition_id,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    }
}

fn inventory(coins: u32, items: Vec<InventoryItem>) -> TradeInventory {
    let next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    TradeInventory::new(
        InventoryState {
            coins,
            items,
            equipment: Vec::new(),
            next_instance_id: Some(i64::from(next)),
        },
        BTreeMap::from([(
            100,
            TradeItemDefinition {
                no_trade: false,
                max_stack: Some(99),
            },
        )]),
        BTreeSet::new(),
        100,
    )
    .unwrap()
}

fn rig(name: &str) -> Rig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab");
    let scratch = std::env::temp_dir().join(format!("osfr-w163-{name}-{}", std::process::id()));
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
    let session = |id: u64, guid: u64| {
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
        session.character_guid = guid;
        session
    };
    let (a, b) = (session(17, A), session(18, B));
    let mut world = SharedWorld::new();
    for (session, guid, x, coins, items) in [
        (&a, A, 0.0, 50, vec![row(10, 100, 3)]),
        (&b, B, 5.0, 70, vec![]),
    ] {
        let presentation = PlayerPresentation {
            character_guid: guid,
            first_name: format!("P{guid}"),
            ..PlayerPresentation::default()
        };
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
                presentation,
            )
            .unwrap();
        let identity = SocialIdentity {
            account_id: guid + 10_000,
            character_guid: guid,
            // Synthetic fixture identifiers, not recovered player names.
            name: SocialName::new(format!("P{guid}"), "").unwrap(),
        };
        world
            .social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        world.social.login(guid).unwrap();
        world.joined.insert(session.id);
        let player = world.players.get(session.id).unwrap().clone();
        world
            .trades
            .register_player(&player, inventory(coins, items))
            .unwrap();
    }
    Rig {
        a,
        b,
        world,
        scratch,
    }
}

fn inner(delivery: &Delivery) -> &[u8] {
    assert_eq!(&delivery.bytes[..3], &[5, 0, 1]);
    &delivery.bytes[7..]
}

fn select(own: u64, target: u64) -> Vec<u8> {
    let mut bytes = vec![26, 0, 19, 0];
    bytes.extend(own.to_le_bytes());
    bytes.extend(target.to_le_bytes());
    bytes
}

fn choice(guid: u64, id: i32) -> Vec<u8> {
    let mut bytes = vec![26, 0, 10, 0];
    bytes.extend(guid.to_le_bytes());
    bytes.extend(id.to_le_bytes());
    bytes
}

fn pkt(sub: u8, session: u32, body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x3b, 0, sub];
    bytes.extend(session.to_le_bytes());
    bytes.extend(body);
    bytes
}

/// Every delivery as (recipient session, family-59 sub).
fn subs(deliveries: &[Delivery]) -> Vec<(u64, u8)> {
    deliveries
        .iter()
        .map(|delivery| (delivery.recipient.0, inner(delivery)[2]))
        .collect()
}

#[test]
fn radial_trade_entry_starts_an_invite_for_the_other_session() {
    let mut rig = rig("invite");
    let menu = rig
        .a
        .route_application(&select(A, B), &mut rig.world, 1_000);
    assert_eq!(menu.len(), 1);
    assert_eq!(menu[0].recipient, rig.a.id);
    let packet = inner(&menu[0]);
    assert_eq!(&packet[..4], &[26, 0, 9, 0]);
    // One entry, the Trade id: Inspect is absent because B has no public look here.
    assert_eq!(i32::from_le_bytes(packet[13..17].try_into().unwrap()), 1);
    assert_eq!(
        i32::from_le_bytes(packet[17..21].try_into().unwrap()),
        osfr_tools::player_radial::TRADE_ENTRY_ID
    );
    let invite = rig.a.route_application(
        &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
        &mut rig.world,
        1_001,
    );
    assert_eq!(subs(&invite), [(18, 2)], "the invite goes to B only");
    assert!(rig.world.trades.session_id_for(A).is_some());
    // Replaying the choice, or a second menu for a busy player, offers nothing.
    assert!(
        rig.a
            .route_application(
                &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
                &mut rig.world,
                1_002
            )
            .is_empty()
    );
    assert!(
        rig.a
            .route_application(&select(A, B), &mut rig.world, 1_003)
            .is_empty()
    );
}

#[test]
fn a_full_trade_ends_in_a_settlement_and_cancels_when_the_commit_refuses() {
    let mut rig = rig("full");
    rig.a
        .route_application(&select(A, B), &mut rig.world, 1_000);
    rig.a.route_application(
        &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
        &mut rig.world,
        1_000,
    );
    let id = rig.world.trades.session_id_for(A).unwrap();
    let started = rig
        .b
        .route_application(&pkt(3, id, &[1]), &mut rig.world, 2_000);
    let mut got = subs(&started);
    got.sort();
    assert_eq!(got, [(17, 4), (18, 4)]);

    let mut item = 10i32.to_le_bytes().to_vec();
    item.extend(2u32.to_le_bytes());
    let echoed = rig
        .a
        .route_application(&pkt(9, id, &item), &mut rig.world, 2_100);
    assert_eq!(subs(&echoed), [(17, 12), (18, 12)]);
    let coins = rig
        .b
        .route_application(&pkt(10, id, &30u32.to_le_bytes()), &mut rig.world, 2_200);
    assert_eq!(subs(&coins), [(17, 11), (18, 11)]);

    let counter = rig.world.trades.session_counter(id).unwrap();
    let mut lock = counter.to_le_bytes().to_vec();
    lock.push(1);
    rig.a
        .route_application(&pkt(8, id, &lock), &mut rig.world, 2_300);
    let confirm = rig
        .b
        .route_application(&pkt(8, id, &lock), &mut rig.world, 2_300);
    assert_eq!(
        subs(&confirm).iter().filter(|(_, sub)| *sub == 18).count(),
        2
    );
    let accept = pkt(6, id, &counter.to_le_bytes());
    assert!(
        rig.a
            .route_application(&accept, &mut rig.world, 12_300)
            .is_empty()
    );
    let before = [
        rig.world.trades.inventory(A).unwrap().clone(),
        rig.world.trades.inventory(B).unwrap().clone(),
    ];
    // The second accept queues a settlement for `settle_trades`; nothing is sent or moved
    // while staging, so a staged world that is dropped leaves no half trade behind.
    assert!(
        rig.b
            .route_application(&accept, &mut rig.world, 12_300)
            .is_empty()
    );
    assert_eq!(rig.world.trade_settlements.len(), 1);
    let settlement = rig.world.trade_settlements[0].clone();
    assert_eq!(settlement.session_id, id);
    assert_eq!(settlement.sides[0].after.coins, 80);
    assert_eq!(settlement.sides[1].after.coins, 40);
    assert_eq!(rig.world.trades.inventory(A).unwrap(), &before[0]);

    // This rig's sessions hold no live inventory equal to the settlement's `before`, so the
    // lead's commit refuses: both are told "canceled" (status 1), the window closes (59/5)
    // and nothing in either inventory changed (the commit's own tests cover success).
    let end = settle_trades(&mut rig.a, &mut SessionMap::new(), &mut rig.world).unwrap();
    assert_eq!(subs(&end).len(), 4);
    for pair in end.chunks(2) {
        assert_eq!(inner(&pair[0])[2], 15);
        assert_eq!(inner(&pair[0])[3], 1);
        assert_eq!(inner(&pair[1])[2], 5);
    }
    assert!(rig.world.trade_settlements.is_empty());
    assert_eq!(rig.world.trades.inventory(A).unwrap(), &before[0]);
    assert_eq!(rig.world.trades.inventory(B).unwrap(), &before[1]);
    assert!(rig.world.trades.session_id_for(A).is_none());
    // Replays after the end only get the stale-window close, to the sender.
    let replay = rig.a.route_application(&accept, &mut rig.world, 13_000);
    assert_eq!(subs(&replay), [(17, 5)]);
    assert!(
        settle_trades(&mut rig.a, &mut SessionMap::new(), &mut rig.world)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn leaving_the_world_or_walking_away_ends_the_trade_for_the_partner() {
    let mut rig = rig("leave");
    rig.a
        .route_application(&select(A, B), &mut rig.world, 1_000);
    rig.a.route_application(
        &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
        &mut rig.world,
        1_000,
    );
    let id = rig.world.trades.session_id_for(A).unwrap();
    rig.b
        .route_application(&pkt(3, id, &[1]), &mut rig.world, 1_100);
    // B walks 60 m away: the housekeeping `commit_outcome` runs after each step ends it.
    rig.world
        .players
        .update_pose(
            rig.b.id,
            ZoneInstance {
                zone_id: 1,
                instance_id: 1,
            },
            [60.0, 0.0, 0.0],
            0.0,
        )
        .unwrap();
    let end = trade_deliveries(rig.world.trades.maintain(
        &rig.world.players,
        &rig.world.social,
        1_200,
    ));
    assert_eq!(subs(&end).len(), 4);
    assert!(rig.world.trades.session_id_for(A).is_none());

    // A second trade, ended by A leaving (disconnect or zoning both call leave_player).
    rig.world
        .players
        .update_pose(
            rig.b.id,
            ZoneInstance {
                zone_id: 1,
                instance_id: 1,
            },
            [5.0, 0.0, 0.0],
            0.0,
        )
        .unwrap();
    rig.a
        .route_application(&select(A, B), &mut rig.world, 2_000);
    rig.a.route_application(
        &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
        &mut rig.world,
        2_000,
    );
    let id = rig.world.trades.session_id_for(A).unwrap();
    rig.b
        .route_application(&pkt(3, id, &[1]), &mut rig.world, 2_100);
    let gone = rig.world.leave_player(rig.a.id);
    let to_b: Vec<_> = gone
        .iter()
        .filter(|delivery| {
            delivery.recipient == rig.b.id
                && delivery.bytes.starts_with(&[5, 0, 1])
                && inner(delivery)[..2] == [0x3b, 0]
        })
        .map(|delivery| inner(delivery)[2])
        .collect();
    assert_eq!(to_b, [15, 5], "status then end for the partner");
    assert!(rig.world.trades.session_id_for(B).is_none());
    assert!(rig.world.trades.inventory(A).is_none());
}

#[test]
fn hostile_trade_packets_from_one_session_cannot_reach_the_other() {
    let mut rig = rig("hostile");
    rig.a
        .route_application(&select(A, B), &mut rig.world, 1_000);
    rig.a.route_application(
        &choice(B, osfr_tools::player_radial::TRADE_ENTRY_ID),
        &mut rig.world,
        1_000,
    );
    let id = rig.world.trades.session_id_for(A).unwrap();
    rig.b
        .route_application(&pkt(3, id, &[1]), &mut rig.world, 1_100);
    let before = rig.world.trades.clone();
    // A nobody session with the right ids: answered to itself only, state untouched.
    let mut stranger = rig
        .b
        .route_application(&pkt(7, id + 9, &[]), &mut rig.world, 1_200);
    stranger.extend(
        rig.b
            .route_application(&pkt(6, 0, &[0; 4]), &mut rig.world, 1_200),
    );
    assert_eq!(subs(&stranger), [(18, 5)]);
    // Truncated, oversized and server-only subs are dropped.
    for bad in [
        vec![0x3b, 0, 9, 1, 0],
        pkt(9, id, &[0; 9]),
        pkt(5, id, &[]),
        pkt(2, id, &[0; 8]),
        vec![0x3b, 0],
    ] {
        assert!(
            rig.a
                .route_application(&bad, &mut rig.world, 1_300)
                .is_empty()
        );
    }
    assert_eq!(rig.world.trades, before);
}

#[test]
fn a_loaded_gameplay_inventory_is_registered_and_kept_current_for_trading() {
    let mut rig = rig("refresh");
    let mut login = 1u16.to_le_bytes().to_vec();
    for (i, value) in ["w163", "1.910.1.530630", ""].into_iter().enumerate() {
        login.extend((value.len() as u32).to_le_bytes());
        login.extend(value.as_bytes());
        if i == 0 {
            login.extend(1u64.to_le_bytes());
        }
    }
    rig.a.gateway.handle(&login).unwrap();
    rig.a.gateway.handle(&wrap_application(&[13, 0])).unwrap();
    rig.a.gateway.handle(&wrap_application(&[10, 0])).unwrap();
    let loaded = rig
        .a
        .gateway
        .trade_inventory()
        .expect("a loaded gameplay validates as a trade inventory");
    // Register, then sync: the service ends up with exactly the gameplay's inventory.
    rig.world.trades.unregister_player(A);
    assert!(refresh_trade_player(&mut rig.world, rig.a.id, &rig.a.gateway).is_empty());
    assert_eq!(rig.world.trades.inventory(A), Some(&loaded.state));
    assert!(refresh_trade_player(&mut rig.world, rig.a.id, &rig.a.gateway).is_empty());
    assert_eq!(rig.world.trades.inventory(A), Some(&loaded.state));
    // A session without loaded gameplay is not tradable: it is unregistered.
    rig.world.trades.unregister_player(B);
    assert!(rig.world.trades.inventory(B).is_none());
    refresh_trade_player(&mut rig.world, rig.b.id, &rig.b.gateway);
    assert_eq!(
        rig.world.trades.inventory(B).is_some(),
        rig.b.gateway.trade_inventory().is_some()
    );
}
