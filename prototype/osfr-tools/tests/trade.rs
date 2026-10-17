use osfr_tools::{inventory, players, social_lists, trade};

use inventory::{EquippedItem, InventoryItem, InventoryState};
use players::{InterestPolicy, PlayerRegistry, PlayerSession, SessionId, ZoneInstance};
use social_lists::{CharacterIdentity, SocialLists, SocialName, SocialService};
use std::collections::{BTreeMap, BTreeSet};
use trade::{TradeError, TradeInventory, TradeItemDefinition, TradeMessage, TradeService};

const A: u64 = 101;
const B: u64 = 202;

struct World {
    players: PlayerRegistry,
    social: SocialService,
    trades: TradeService,
}

fn item(id: i32, definition_id: i32, count: u32) -> InventoryItem {
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

fn state(coins: u32, items: Vec<InventoryItem>) -> InventoryState {
    let next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    InventoryState {
        coins,
        items,
        equipment: Vec::new(),
        next_instance_id: Some(i64::from(next)),
    }
}

fn trade_inventory(state: InventoryState, bag_capacity: usize) -> TradeInventory {
    TradeInventory::new(
        state,
        BTreeMap::from([
            (
                100,
                TradeItemDefinition {
                    no_trade: false,
                    max_stack: Some(99),
                },
            ),
            (
                200,
                TradeItemDefinition {
                    no_trade: false,
                    max_stack: Some(99),
                },
            ),
            (
                300,
                TradeItemDefinition {
                    no_trade: true,
                    max_stack: Some(99),
                },
            ),
        ]),
        BTreeSet::new(),
        bag_capacity,
    )
    .unwrap()
}

fn world(first: TradeInventory, second: TradeInventory) -> World {
    let mut players = PlayerRegistry::new(InterestPolicy::new(100.0, 16).unwrap());
    let mut social = SocialService::new(8).unwrap();
    let mut trades = TradeService::new();
    for (guid, inventory) in [(A, first), (B, second)] {
        let identity = CharacterIdentity {
            account_id: guid + 10_000,
            character_guid: guid,
            // Synthetic fixture identifiers, not recovered player names.
            name: SocialName::new(format!("P{guid}"), "").unwrap(),
        };
        social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        social.login(guid).unwrap();
        let session_id = SessionId(guid + 1_000);
        players
            .join(
                session_id,
                guid + 10_000,
                guid,
                ZoneInstance {
                    zone_id: 1,
                    instance_id: 1,
                },
                [0.0; 3],
                0.0,
            )
            .unwrap();
        let player: &PlayerSession = players.get(session_id).unwrap();
        trades.register_player(player, inventory).unwrap();
    }
    World {
        players,
        social,
        trades,
    }
}

fn open(world: &mut World) -> u32 {
    let messages = world
        .trades
        .invite(&world.players, &world.social, A, B, 0)
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(sub(&messages[0]), 2);
    let session = world.trades.session_id_for(A).unwrap();
    assert_ne!(session, u32::MAX);
    let started = world
        .trades
        .reply_invite(&world.players, &world.social, B, session, true, 0)
        .unwrap();
    assert_eq!(started.len(), 2);
    assert!(started.iter().all(|message| sub(message) == 4));
    session
}

fn sub(message: &TradeMessage) -> u8 {
    message.bytes[2]
}

fn status(message: &TradeMessage) -> u8 {
    message.bytes[3]
}

fn state_of(world: &World, guid: u64) -> &InventoryState {
    world.trades.inventory(guid).unwrap()
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

#[test]
fn items_and_coins_swap_exactly_and_close_both_windows() {
    let mut world = world(
        trade_inventory(state(50, vec![item(10, 100, 3)]), 8),
        trade_inventory(state(20, vec![item(20, 200, 4)]), 8),
    );
    let session = open(&mut world);
    world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 2)
        .unwrap();
    world
        .trades
        .offer_coins(&world.players, &world.social, A, session, 15)
        .unwrap();
    world
        .trades
        .offer_item(&world.players, &world.social, B, session, 20, 4)
        .unwrap();
    world
        .trades
        .offer_coins(&world.players, &world.social, B, session, 5)
        .unwrap();
    let counter = world.trades.session_counter(session).unwrap();
    world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            A,
            session,
            counter,
            true,
            100,
        )
        .unwrap();
    let locked = world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            B,
            session,
            counter,
            true,
            200,
        )
        .unwrap();
    assert_eq!(
        locked.iter().filter(|message| sub(message) == 18).count(),
        2
    );
    assert!(
        world
            .trades
            .accept(&world.players, &world.social, A, session, counter, 10_200)
            .unwrap()
            .messages
            .is_empty()
    );
    let accepted = world
        .trades
        .accept(&world.players, &world.social, B, session, counter, 10_200)
        .unwrap();
    // The service moved nothing: the settlement is the host's to commit.
    assert!(accepted.messages.is_empty());
    assert_eq!(state_of(&world, A).coins, 50);
    let settlement = accepted.settlement.unwrap();
    assert_eq!(settlement.session_id, session);
    assert_eq!(settlement.sides[0].before.coins, 50);
    assert_eq!(settlement.sides[0].after.coins, 40);
    assert_eq!(settlement.sides[1].after.coins, 30);
    let finished = world.trades.settled(session, true).unwrap();
    assert_eq!(
        finished.iter().map(sub).collect::<Vec<_>>(),
        vec![15, 5, 15, 5]
    );
    assert!(
        finished
            .iter()
            .filter(|message| sub(message) == 15)
            .all(|message| status(message) == 6)
    );
    assert_eq!(state_of(&world, A).coins, 40);
    assert_eq!(state_of(&world, B).coins, 30);
    assert_eq!(
        state_of(&world, A)
            .items
            .iter()
            .map(|item| (item.definition_id, item.count))
            .collect::<Vec<_>>(),
        vec![(100, 1), (200, 4)]
    );
    assert_eq!(
        state_of(&world, B)
            .items
            .iter()
            .map(|item| (item.definition_id, item.count))
            .collect::<Vec<_>>(),
        vec![(100, 2)]
    );
}

#[test]
fn builders_match_native_trade_reader_layouts() {
    let mut world = world(
        trade_inventory(state(20, vec![item(10, 100, 3)]), 8),
        trade_inventory(state(0, vec![]), 8),
    );
    let invite = world
        .trades
        .invite(&world.players, &world.social, A, B, 0)
        .unwrap();
    let session = world.trades.session_id_for(A).unwrap();
    let invite_bytes = &invite[0].bytes;
    assert_eq!(invite_bytes.len(), 39);
    assert_eq!(&invite_bytes[..3], &[0x3b, 0, 2]);
    assert!(invite_bytes[3..15].iter().all(|byte| *byte == 0));
    assert_eq!(read_u32(invite_bytes, 15), 4);
    assert_eq!(read_u32(invite_bytes, 23), 0);
    assert_eq!(
        u64::from_le_bytes(invite_bytes[27..35].try_into().unwrap()),
        A
    );
    assert_eq!(read_u32(invite_bytes, 35), session);

    let starts = world
        .trades
        .reply_invite(&world.players, &world.social, B, session, true, 0)
        .unwrap();
    assert_eq!(starts[0].bytes.len(), 39);
    assert_eq!(&starts[0].bytes[..3], &[0x3b, 0, 4]);
    assert_eq!(
        u64::from_le_bytes(starts[0].bytes[3..11].try_into().unwrap()),
        B
    );
    assert_eq!(read_u32(&starts[0].bytes, 35), session);

    let added = world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 2)
        .unwrap();
    assert_eq!(added[0].bytes.len(), 41);
    assert_eq!(&added[0].bytes[..3], &[0x3b, 0, 12]);
    assert_eq!(read_u32(&added[0].bytes, 3), session);
    assert_eq!(read_u32(&added[0].bytes, 7), 2);
    assert_eq!(added[0].bytes[11], 1);
    assert_eq!(read_u32(&added[0].bytes, 12), 100);
    assert_eq!(read_u32(&added[0].bytes, 20), 10);
    assert_eq!(read_u32(&added[0].bytes, 24), 2);
    assert_eq!(added[0].bytes[36], 0);
    assert_eq!(read_u32(&added[0].bytes, 37), 0);
    assert_eq!(added[1].bytes[11], 0);

    let changed = world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 1)
        .unwrap();
    assert_eq!(changed[0].bytes.len(), 20);
    assert_eq!(&changed[0].bytes[..3], &[0x3b, 0, 13]);
    assert_eq!(read_u32(&changed[0].bytes, 7), 3);
    assert_eq!(read_u32(&changed[0].bytes, 12), 10);
    assert_eq!(read_u32(&changed[0].bytes, 16), 1);

    let coins = world
        .trades
        .offer_coins(&world.players, &world.social, A, session, 5)
        .unwrap();
    assert_eq!(coins[0].bytes.len(), 16);
    assert_eq!(&coins[0].bytes[..3], &[0x3b, 0, 11]);
    assert_eq!(read_u32(&coins[0].bytes, 7), 4);
    assert_eq!(coins[0].bytes[11], 1);
    assert_eq!(read_u32(&coins[0].bytes, 12), 5);
    assert_eq!(coins[1].bytes[11], 0);

    let counter = world.trades.session_counter(session).unwrap();
    assert_eq!(
        world.trades.set_lock(
            &world.players,
            &world.social,
            A,
            session,
            counter - 1,
            true,
            0
        ),
        Err(TradeError::StaleCounter)
    );
    let lock = world
        .trades
        .set_lock(&world.players, &world.social, A, session, counter, true, 0)
        .unwrap();
    assert_eq!(lock[0].bytes, [0x3b, 0, 17, session as u8, 0, 0, 0, 1]);
    let confirmation = world
        .trades
        .set_lock(&world.players, &world.social, B, session, counter, true, 1)
        .unwrap();
    assert_eq!(
        confirmation
            .iter()
            .filter(|message| sub(message) == 18)
            .count(),
        2
    );
    assert!(
        confirmation
            .iter()
            .filter(|message| sub(message) == 18)
            .all(|message| message.bytes == [0x3b, 0, 18, session as u8, 0, 0, 0])
    );
    let ended = world.trades.cancel(A, session).unwrap();
    assert_eq!(ended[0].bytes, [0x3b, 0, 15, 1]);
    assert_eq!(ended[1].bytes, [0x3b, 0, 5, session as u8, 0, 0, 0]);
}

#[test]
fn offer_change_after_lock_clears_locks_and_prior_acceptance() {
    let mut world = world(
        trade_inventory(state(50, vec![]), 8),
        trade_inventory(state(0, vec![]), 8),
    );
    let session = open(&mut world);
    world
        .trades
        .offer_coins(&world.players, &world.social, A, session, 10)
        .unwrap();
    let counter = world.trades.session_counter(session).unwrap();
    world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            A,
            session,
            counter,
            true,
            100,
        )
        .unwrap();
    world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            B,
            session,
            counter,
            true,
            100,
        )
        .unwrap();
    assert_eq!(
        world
            .trades
            .accept(&world.players, &world.social, A, session, counter, 10_099),
        Err(TradeError::AcceptTooEarly)
    );
    world
        .trades
        .accept(&world.players, &world.social, A, session, counter, 10_100)
        .unwrap();
    let changed = world
        .trades
        .offer_coins(&world.players, &world.social, A, session, 5)
        .unwrap();
    assert!(
        changed
            .iter()
            .any(|message| sub(message) == 17 && message.bytes[7] == 0)
    );
    assert!(!world.trades.is_locked(session, A));
    assert!(!world.trades.is_locked(session, B));
    let counter = world.trades.session_counter(session).unwrap();
    assert_eq!(
        world
            .trades
            .accept(&world.players, &world.social, B, session, counter, 20_000),
        Err(TradeError::NotBothLocked)
    );

    world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            A,
            session,
            counter,
            true,
            200,
        )
        .unwrap();
    world
        .trades
        .set_lock(
            &world.players,
            &world.social,
            B,
            session,
            counter,
            true,
            200,
        )
        .unwrap();
    assert!(
        world
            .trades
            .accept(&world.players, &world.social, B, session, counter, 10_200)
            .unwrap()
            .messages
            .is_empty()
    );
    assert!(world.trades.session_counter(session).is_some());
    let accepted = world
        .trades
        .accept(&world.players, &world.social, A, session, counter, 10_200)
        .unwrap();
    assert!(accepted.settlement.is_some());
    let completed = world.trades.settled(session, true).unwrap();
    assert_eq!(
        completed
            .iter()
            .filter(|message| sub(message) == 15)
            .count(),
        2
    );
    assert_eq!(state_of(&world, A).coins, 45);
    assert_eq!(state_of(&world, B).coins, 5);
}

#[test]
fn untradable_unowned_and_over_quantity_items_are_rejected() {
    let mut owned = item(11, 100, 4);
    owned.consumed_count = 1;
    owned.stored_count = 1;
    let mut first_state = state(
        0,
        vec![item(10, 300, 1), owned, item(12, 200, 1), item(13, 200, 1)],
    );
    first_state.equipment.push(EquippedItem {
        profile_id: 1,
        slot: 1,
        item_id: 12,
        category: 1,
    });
    let mut first = trade_inventory(first_state, 8);
    first.equipped_on_pet.insert(13);
    let mut world = world(first, trade_inventory(state(0, vec![]), 8));
    let session = open(&mut world);
    assert_eq!(
        world
            .trades
            .offer_item(&world.players, &world.social, A, session, 10, 1),
        Err(TradeError::UntradableItem)
    );
    assert_eq!(
        world
            .trades
            .offer_item(&world.players, &world.social, A, session, 12, 1),
        Err(TradeError::UntradableItem)
    );
    assert_eq!(
        world
            .trades
            .offer_item(&world.players, &world.social, A, session, 13, 1),
        Err(TradeError::UntradableItem)
    );
    assert_eq!(
        world
            .trades
            .offer_item(&world.players, &world.social, A, session, 999, 1),
        Err(TradeError::UnknownItem)
    );
    assert_eq!(
        world
            .trades
            .offer_item(&world.players, &world.social, A, session, 11, 3),
        Err(TradeError::InvalidQuantity)
    );
    assert_eq!(world.trades.session_counter(session), Some(1));
    assert_eq!(state_of(&world, A).items[1].count, 4);
}

#[test]
fn full_bag_rejects_exchange_without_mutating_either_inventory() {
    let mut world = world(
        trade_inventory(state(10, vec![item(10, 100, 1)]), 4),
        trade_inventory(state(20, vec![item(20, 200, 1)]), 1),
    );
    let before_a = state_of(&world, A).clone();
    let before_b = state_of(&world, B).clone();
    let session = open(&mut world);
    world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 1)
        .unwrap();
    let counter = world.trades.session_counter(session).unwrap();
    world
        .trades
        .set_lock(&world.players, &world.social, A, session, counter, true, 0)
        .unwrap();
    world
        .trades
        .set_lock(&world.players, &world.social, B, session, counter, true, 0)
        .unwrap();
    world
        .trades
        .accept(&world.players, &world.social, A, session, counter, 10_000)
        .unwrap();
    let cancelled = world
        .trades
        .accept(&world.players, &world.social, B, session, counter, 10_000)
        .unwrap();
    assert!(cancelled.settlement.is_none());
    let cancelled = cancelled.messages;
    assert_eq!(state_of(&world, A), &before_a);
    assert_eq!(state_of(&world, B), &before_b);
    assert!(
        cancelled
            .iter()
            .any(|message| sub(message) == 15 && status(message) == 4)
    );
    assert!(
        cancelled
            .iter()
            .any(|message| sub(message) == 15 && status(message) == 3)
    );
    assert!(cancelled.iter().filter(|message| sub(message) == 5).count() == 2);
}

#[test]
fn cancel_and_disconnect_close_without_changing_inventories() {
    let mut world = world(
        trade_inventory(state(31, vec![item(10, 100, 2)]), 8),
        trade_inventory(state(13, vec![item(20, 200, 3)]), 8),
    );
    let before_a = state_of(&world, A).clone();
    let before_b = state_of(&world, B).clone();
    let cancelled_session = open(&mut world);
    let cancelled = world.trades.cancel(A, cancelled_session).unwrap();
    assert_eq!(
        cancelled.iter().map(sub).collect::<Vec<_>>(),
        vec![15, 5, 15, 5]
    );
    assert_eq!(state_of(&world, A), &before_a);
    assert_eq!(state_of(&world, B), &before_b);

    let disconnected_session = open(&mut world);
    let disconnected = world.trades.disconnect(A);
    assert_eq!(
        disconnected.iter().map(sub).collect::<Vec<_>>(),
        vec![15, 5, 15, 5]
    );
    assert!(
        disconnected
            .iter()
            .filter(|message| sub(message) == 15)
            .all(|message| status(message) == 2)
    );
    assert_eq!(world.trades.session_counter(disconnected_session), None);
    assert_eq!(state_of(&world, A), &before_a);
    assert_eq!(state_of(&world, B), &before_b);
}

#[test]
fn ignored_target_cannot_be_invited() {
    let mut world = world(
        trade_inventory(state(0, vec![]), 8),
        trade_inventory(state(0, vec![]), 8),
    );
    let inviter_name = world.social.profile(A).unwrap().name.clone();
    world
        .social
        .profile_mut(B)
        .unwrap()
        .ignored
        .insert(A, inviter_name);
    assert_eq!(
        world.trades.invite(&world.players, &world.social, A, B, 0),
        Err(TradeError::Ignored)
    );
    assert_eq!(world.trades.session_id_for(A), None);
}

#[test]
fn decline_notifies_inviter_and_releases_both_players() {
    let mut world = world(
        trade_inventory(state(0, vec![]), 8),
        trade_inventory(state(0, vec![]), 8),
    );
    world
        .trades
        .invite(&world.players, &world.social, A, B, 0)
        .unwrap();
    let session = world.trades.session_id_for(A).unwrap();
    let declined = world
        .trades
        .reply_invite(&world.players, &world.social, B, session, false, 0)
        .unwrap();
    assert_eq!(declined.len(), 1);
    assert_eq!(sub(&declined[0]), 16);
    assert_eq!(world.trades.session_id_for(A), None);
    assert_eq!(world.trades.session_id_for(B), None);
    assert!(open(&mut world) > session);
}

#[test]
fn duplicate_accept_cannot_settle_twice() {
    let mut world = world(
        trade_inventory(state(10, vec![item(10, 100, 1)]), 8),
        trade_inventory(state(20, vec![item(20, 200, 1)]), 8),
    );
    let session = open(&mut world);
    world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 1)
        .unwrap();
    world
        .trades
        .offer_item(&world.players, &world.social, B, session, 20, 1)
        .unwrap();
    let counter = world.trades.session_counter(session).unwrap();
    world
        .trades
        .set_lock(&world.players, &world.social, A, session, counter, true, 0)
        .unwrap();
    world
        .trades
        .set_lock(&world.players, &world.social, B, session, counter, true, 0)
        .unwrap();
    world
        .trades
        .accept(&world.players, &world.social, A, session, counter, 10_000)
        .unwrap();
    assert!(
        world
            .trades
            .accept(&world.players, &world.social, B, session, counter, 10_000)
            .unwrap()
            .settlement
            .is_some()
    );
    world.trades.settled(session, true).unwrap();
    let after_a = state_of(&world, A).clone();
    let after_b = state_of(&world, B).clone();
    assert_eq!(
        world
            .trades
            .accept(&world.players, &world.social, A, session, counter, 20_000),
        Err(TradeError::UnknownSession)
    );
    assert_eq!(state_of(&world, A), &after_a);
    assert_eq!(state_of(&world, B), &after_b);
}

#[test]
fn timeout_closes_active_trade_without_settlement() {
    let mut world = world(
        trade_inventory(state(10, vec![item(10, 100, 1)]), 8),
        trade_inventory(state(20, vec![]), 8),
    );
    let before_a = state_of(&world, A).clone();
    let before_b = state_of(&world, B).clone();
    let session = open(&mut world);
    world
        .trades
        .offer_item(&world.players, &world.social, A, session, 10, 1)
        .unwrap();
    let timeout = world.trades.timeout(session).unwrap();
    assert_eq!(
        timeout.iter().map(sub).collect::<Vec<_>>(),
        vec![15, 5, 15, 5]
    );
    assert!(
        timeout
            .iter()
            .filter(|message| sub(message) == 15)
            .all(|message| status(message) == 1)
    );
    assert_eq!(state_of(&world, A), &before_a);
    assert_eq!(state_of(&world, B), &before_b);
}

#[test]
fn instanced_zone_exit_sends_viewer_relative_statuses_and_end() {
    let mut world = world(
        trade_inventory(state(0, vec![]), 8),
        trade_inventory(state(0, vec![]), 8),
    );
    let session = open(&mut world);
    let messages = world.trades.enter_instanced_zone(B);
    assert_eq!(
        messages.iter().map(sub).collect::<Vec<_>>(),
        vec![15, 5, 15, 5]
    );
    assert_eq!(status(&messages[0]), 8);
    assert_eq!(status(&messages[2]), 7);
    assert_eq!(world.trades.session_counter(session), None);
}
