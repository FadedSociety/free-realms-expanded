//! W163: real player-to-player trade, from the radial Trade entry to the validated settlement.
//!
//! Every fixture identifier is synthetic. The service never moves items or coins: the tests
//! hand the settlement back with `settled`, the way the host's atomic commit would.

use osfr_tools::inventory::{EquippedItem, InventoryItem, InventoryState, RentalState};
use osfr_tools::player_radial::{
    Chosen, INSPECT_ENTRY_ID, MenuOffer, PlayerRadial, RadialError, TRADE_ENTRY_ID,
};
use osfr_tools::players::{
    InterestPolicy, PlayerPresentation, PlayerRegistry, SessionId, ZoneInstance,
};
use osfr_tools::social_lists::{CharacterIdentity, SocialLists, SocialName, SocialService};
use osfr_tools::trade::{
    ItemMove, TradeError, TradeInventory, TradeItemDefinition, TradeMessage, TradeOutcome,
    TradeRules, TradeService,
};
use std::collections::{BTreeMap, BTreeSet};

const A: u64 = 101;
const B: u64 = 202;
const C: u64 = 303;

fn sess(guid: u64) -> SessionId {
    SessionId(guid + 1_000)
}

fn zone(instance: u64) -> ZoneInstance {
    ZoneInstance {
        zone_id: 1,
        instance_id: instance,
    }
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

fn inventory(state: InventoryState, bag_capacity: usize) -> TradeInventory {
    let def = |no_trade, max_stack| TradeItemDefinition {
        no_trade,
        max_stack: Some(max_stack),
    };
    TradeInventory::new(
        state,
        BTreeMap::from([
            (100, def(false, 99)),
            (200, def(false, 99)),
            (300, def(true, 99)),
            (400, def(false, 5)),
        ]),
        BTreeSet::new(),
        bag_capacity,
    )
    .unwrap()
}

struct World {
    players: PlayerRegistry,
    social: SocialService,
    trades: TradeService,
    now: u64,
}

impl World {
    /// A at the origin, B 5 m away, C 5 m the other way; one zone instance.
    fn new(a: TradeInventory, b: TradeInventory) -> Self {
        let mut world = Self {
            players: PlayerRegistry::new(InterestPolicy::new(100.0, 16).unwrap()),
            social: SocialService::new(8).unwrap(),
            trades: TradeService::new(),
            now: 1_000,
        };
        world.add(A, [0.0, 0.0, 0.0], a);
        world.add(B, [5.0, 0.0, 0.0], b);
        world.add(C, [-5.0, 0.0, 0.0], inventory(state(0, vec![]), 8));
        world
    }

    fn add(&mut self, guid: u64, position: [f32; 3], inventory: TradeInventory) {
        let identity = CharacterIdentity {
            account_id: guid + 10_000,
            character_guid: guid,
            // Synthetic fixture identifiers, not recovered player names.
            name: SocialName::new(format!("P{guid}"), "").unwrap(),
        };
        self.social
            .register(SocialLists::new(identity).unwrap())
            .unwrap();
        self.social.login(guid).unwrap();
        let presentation = PlayerPresentation {
            character_guid: guid,
            first_name: format!("P{guid}"),
            ..PlayerPresentation::default()
        };
        self.players
            .join_with_presentation(
                sess(guid),
                guid + 10_000,
                guid,
                zone(1),
                position,
                0.0,
                presentation,
            )
            .unwrap();
        let player = self.players.get(sess(guid)).unwrap().clone();
        self.trades.register_player(&player, inventory).unwrap();
    }

    fn send(&mut self, guid: u64, bytes: &[u8]) -> Result<TradeOutcome, TradeError> {
        self.trades
            .handle(&self.players, &self.social, guid, bytes, self.now)
    }

    fn invite(&mut self, from: u64, to: u64) -> Result<Vec<TradeMessage>, TradeError> {
        self.trades
            .invite(&self.players, &self.social, from, to, self.now)
    }

    fn maintain(&mut self) -> Vec<TradeMessage> {
        self.trades.maintain(&self.players, &self.social, self.now)
    }

    /// Invite A -> B and accept through the real client packet.
    fn open(&mut self) -> u32 {
        self.invite(A, B).unwrap();
        let id = self.trades.session_id_for(A).unwrap();
        let started = self.send(B, &reply(id, true)).unwrap();
        assert_eq!(started.messages.iter().map(sub).collect::<Vec<_>>(), [4, 4]);
        id
    }

    fn counter(&self, id: u32) -> u32 {
        self.trades.session_counter(id).unwrap()
    }

    /// Both lock, wait out the native timer, both accept; returns the second accept's outcome.
    fn lock_and_accept(&mut self, id: u32) -> TradeOutcome {
        let counter = self.counter(id);
        self.send(A, &lock(id, counter, true)).unwrap();
        let locked = self.send(B, &lock(id, counter, true)).unwrap();
        assert_eq!(locked.messages.iter().filter(|m| sub(m) == 18).count(), 2);
        self.now += 10_000;
        let first = self.send(A, &accept(id, counter)).unwrap();
        assert!(first.messages.is_empty() && first.settlement.is_none());
        self.send(B, &accept(id, counter)).unwrap()
    }

    fn free(&self, guid: u64) -> bool {
        self.trades.session_id_for(guid).is_none()
    }

    fn inv(&self, guid: u64) -> InventoryState {
        self.trades.inventory(guid).unwrap().clone()
    }
}

fn pkt(sub: u8, session: u32, body: &[u8]) -> Vec<u8> {
    let mut out = vec![0x3b, 0, sub];
    out.extend(session.to_le_bytes());
    out.extend(body);
    out
}

fn reply(session: u32, accepted: bool) -> Vec<u8> {
    pkt(3, session, &[u8::from(accepted)])
}

fn accept(session: u32, counter: u32) -> Vec<u8> {
    pkt(6, session, &counter.to_le_bytes())
}

fn cancel(session: u32) -> Vec<u8> {
    pkt(7, session, &[])
}

fn lock(session: u32, counter: u32, locked: bool) -> Vec<u8> {
    let mut body = counter.to_le_bytes().to_vec();
    body.push(u8::from(locked));
    pkt(8, session, &body)
}

fn offer_item(session: u32, item: i32, total: u32) -> Vec<u8> {
    let mut body = item.to_le_bytes().to_vec();
    body.extend(total.to_le_bytes());
    pkt(9, session, &body)
}

fn offer_coins(session: u32, coins: u32) -> Vec<u8> {
    pkt(10, session, &coins.to_le_bytes())
}

fn sub(message: &TradeMessage) -> u8 {
    message.bytes[2]
}

fn status(message: &TradeMessage) -> u8 {
    assert_eq!(sub(message), 15);
    message.bytes[3]
}

fn subs(messages: &[TradeMessage]) -> Vec<u8> {
    messages.iter().map(sub).collect()
}

/// Statuses and ends of a close, per recipient: `[(session, status)]`.
fn closes(messages: &[TradeMessage]) -> Vec<(SessionId, u8)> {
    messages
        .iter()
        .filter(|m| sub(m) == 15)
        .map(|m| (m.recipient, status(m)))
        .collect()
}

fn totals(states: &[InventoryState]) -> (u64, BTreeMap<i32, u32>) {
    let mut by_definition = BTreeMap::new();
    for state in states {
        for item in &state.items {
            *by_definition.entry(item.definition_id).or_insert(0) += item.count;
        }
    }
    (
        states.iter().map(|state| u64::from(state.coins)).sum(),
        by_definition,
    )
}

fn is_closed_clean(world: &World, messages: &[TradeMessage], statuses: [u8; 2]) {
    assert_eq!(subs(messages), [15, 5, 15, 5]);
    let got: Vec<u8> = closes(messages).iter().map(|(_, s)| *s).collect();
    assert_eq!(got, statuses);
    assert!(world.free(A) && world.free(B));
}

// ---------------------------------------------------------------------------------------------
// The radial entry
// ---------------------------------------------------------------------------------------------

fn select(own: u64, target: u64) -> Vec<u8> {
    let mut b = vec![26, 0, 19, 0];
    b.extend(own.to_le_bytes());
    b.extend(target.to_le_bytes());
    b
}

fn choice(guid: u64, id: i32) -> Vec<u8> {
    let mut b = vec![26, 0, 10, 0];
    b.extend(guid.to_le_bytes());
    b.extend(id.to_le_bytes());
    b
}

/// Entry rows of a 26/9 packet: (id, icon, label).
fn menu_rows(packet: &[u8]) -> Vec<(i32, i32, i32)> {
    let word = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    (0..word(13) as usize)
        .map(|n| (word(17 + 28 * n), word(21 + 28 * n), word(25 + 28 * n)))
        .collect()
}

fn open_menu(
    world: &World,
    radial: &mut PlayerRadial,
    viewer: u64,
    target: u64,
    inspect: bool,
) -> Result<Vec<Vec<u8>>, RadialError> {
    radial.select_menu(&world.players, sess(viewer), &select(viewer, target), |t| {
        MenuOffer {
            inspect,
            trade: world
                .trades
                .can_invite(&world.players, &world.social, viewer, t),
        }
    })
}

#[test]
fn trade_entry_is_offered_to_a_visible_in_reach_free_player_before_inspect() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let mut radial = PlayerRadial::default();
    let menu = open_menu(&world, &mut radial, A, B, true).unwrap();
    assert_eq!(
        menu_rows(&menu[0]),
        [
            (TRADE_ENTRY_ID, 43_188, 3_299),
            (INSPECT_ENTRY_ID, 133, 3_902)
        ]
    );
    // Trade alone is a valid menu too (no public look yet).
    let menu = open_menu(&world, &mut radial, A, B, false).unwrap();
    assert_eq!(menu_rows(&menu[0]), [(TRADE_ENTRY_ID, 43_188, 3_299)]);

    // Busy target (pending invite from C): no Trade entry; with no Inspect either, no menu.
    world.invite(C, B).unwrap();
    assert_eq!(
        open_menu(&world, &mut radial, A, B, false),
        Err(RadialError::NoLook)
    );
    let menu = open_menu(&world, &mut radial, A, B, true).unwrap();
    assert_eq!(menu_rows(&menu[0]), [(INSPECT_ENTRY_ID, 133, 3_902)]);
    // Busy viewer: same.
    assert_eq!(
        open_menu(&world, &mut radial, C, A, false),
        Err(RadialError::NoLook)
    );
}

#[test]
fn trade_entry_needs_reach_and_zone_and_a_registered_player() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let mut radial = PlayerRadial::default();
    // 41 m: inside the radial's 40 m? no - outside both reaches.
    world
        .players
        .update_pose(sess(B), zone(1), [41.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(open_menu(&world, &mut radial, A, B, false).is_err());
    // 39 m: in reach.
    world
        .players
        .update_pose(sess(B), zone(1), [39.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(open_menu(&world, &mut radial, A, B, false).is_ok());
    // Another instance next to the viewer.
    world
        .players
        .update_pose(sess(B), zone(2), [1.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(open_menu(&world, &mut radial, A, B, false).is_err());
    // A stranger the trade service never heard of.
    world
        .players
        .update_pose(sess(B), zone(1), [5.0, 0.0, 0.0], 0.0)
        .unwrap();
    world.trades.unregister_player(B);
    assert!(open_menu(&world, &mut radial, A, B, false).is_err());
}

#[test]
fn choosing_trade_starts_the_invite_and_the_menu_is_one_shot() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let mut radial = PlayerRadial::default();
    open_menu(&world, &mut radial, A, B, true).unwrap();
    let chosen = radial
        .choose_entry(&world.players, sess(A), &choice(B, TRADE_ENTRY_ID), |_| {
            None
        })
        .unwrap();
    assert_eq!(chosen, Some(Chosen::Trade(B)));
    // Replaying the same choice: the menu is gone, so it is not a player-menu choice any more.
    assert_eq!(
        radial.choose_entry(&world.players, sess(A), &choice(B, TRADE_ENTRY_ID), |_| {
            None
        }),
        Ok(None)
    );
    let invite = world.invite(A, B).unwrap();
    assert_eq!(subs(&invite), [2]);
    assert_eq!(invite[0].recipient, sess(B));
}

#[test]
fn a_trade_choice_not_on_the_menu_or_gone_stale_starts_nothing() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let mut radial = PlayerRadial::default();
    // Menu without Trade (Inspect only, via the legacy entry point).
    radial
        .select_player(&world.players, sess(A), &select(A, B), |_| true)
        .unwrap();
    assert_eq!(
        radial.choose_entry(&world.players, sess(A), &choice(B, TRADE_ENTRY_ID), |_| {
            None
        }),
        Err(RadialError::InvalidChoice)
    );
    assert_eq!(radial.menu_target(), None);
    // The legacy choose path can never start a trade.
    open_menu(&world, &mut radial, A, B, true).unwrap();
    assert_eq!(
        radial.choose(&world.players, sess(A), &choice(B, TRADE_ENTRY_ID), |_| {
            None
        }),
        Err(RadialError::InvalidChoice)
    );
    // An unknown id on a Trade menu.
    open_menu(&world, &mut radial, A, B, false).unwrap();
    assert_eq!(
        radial.choose_entry(&world.players, sess(A), &choice(B, 4242), |_| None),
        Err(RadialError::InvalidChoice)
    );
    // The target walked away between menu and click.
    open_menu(&world, &mut radial, A, B, false).unwrap();
    world
        .players
        .update_pose(sess(B), zone(1), [60.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert_eq!(
        radial.choose_entry(&world.players, sess(A), &choice(B, TRADE_ENTRY_ID), |_| {
            None
        }),
        Err(RadialError::TooFar)
    );
    assert!(world.free(A) && world.free(B));
}

// ---------------------------------------------------------------------------------------------
// The full lifecycle and the settlement
// ---------------------------------------------------------------------------------------------

#[test]
fn item_for_coins_settlement_has_exact_deltas_and_nothing_moves_until_committed() {
    let mut world = World::new(
        inventory(state(50, vec![item(10, 100, 3)]), 8),
        inventory(state(70, vec![item(20, 200, 1)]), 8),
    );
    let before = [world.inv(A), world.inv(B)];
    let id = world.open();
    // A offers 2 of item 10; B offers 30 coins.
    let echoes = world.send(A, &offer_item(id, 10, 2)).unwrap().messages;
    assert_eq!(subs(&echoes), [12, 12]);
    assert_eq!(echoes[0].recipient, sess(A));
    assert_eq!(
        (echoes[0].bytes[11], echoes[1].bytes[11]),
        (1, 0),
        "own side 1, partner side 0"
    );
    let echoes = world.send(B, &offer_coins(id, 30)).unwrap().messages;
    assert_eq!(subs(&echoes), [11, 11]);

    let done = world.lock_and_accept(id);
    assert!(done.messages.is_empty(), "no status until the host answers");
    let settlement = done.settlement.expect("both accepted");
    assert_eq!(settlement.session_id, id);
    let [side_a, side_b] = &settlement.sides;
    assert_eq!((side_a.guid, side_b.guid), (A, B));
    assert_eq!(side_a.before, before[0]);
    assert_eq!(side_b.before, before[1]);
    assert_eq!(side_a.after.coins, 80);
    assert_eq!(side_b.after.coins, 40);
    assert_eq!(
        side_a.gave,
        [ItemMove {
            item_id: 10,
            definition_id: 100,
            tint: 0,
            count: 2
        }]
    );
    assert!(side_a.received.is_empty() && side_b.gave.is_empty());
    assert_eq!(
        side_b.received,
        [ItemMove {
            item_id: 21,
            definition_id: 100,
            tint: 0,
            count: 2
        }]
    );
    assert_eq!(side_a.after.items.len(), 1);
    assert_eq!(
        (side_a.after.items[0].id, side_a.after.items[0].count),
        (10, 1)
    );
    let b_items: Vec<_> = side_b
        .after
        .items
        .iter()
        .map(|i| (i.id, i.definition_id, i.count))
        .collect();
    assert_eq!(b_items, [(20, 200, 1), (21, 100, 2)]);
    // Conservation: coins and items per definition are unchanged in total.
    assert_eq!(
        totals(&[side_a.after.clone(), side_b.after.clone()]),
        totals(&before)
    );
    // The service moved nothing yet and both windows are still open.
    assert_eq!(
        (world.inv(A), world.inv(B)),
        (before[0].clone(), before[1].clone())
    );
    assert!(!world.free(A));

    let end = world.trades.settled(id, true).unwrap();
    is_closed_clean(&world, &end, [6, 6]);
    assert_eq!(world.inv(A), side_a.after);
    assert_eq!(world.inv(B), side_b.after);
}

#[test]
fn item_for_item_settlement_merges_into_stacks_and_allocates_fresh_rows() {
    let mut world = World::new(
        inventory(state(5, vec![item(10, 100, 3), item(11, 200, 2)]), 8),
        inventory(state(5, vec![item(30, 100, 5), item(31, 400, 4)]), 8),
    );
    let before = [world.inv(A), world.inv(B)];
    let id = world.open();
    world.send(A, &offer_item(id, 10, 2)).unwrap(); // def 100 x2 -> B (merges into 30)
    world.send(A, &offer_item(id, 11, 2)).unwrap(); // def 200 x2 -> B (new row)
    world.send(B, &offer_item(id, 30, 1)).unwrap(); // def 100 x1 -> A (merges into 10)
    world.send(B, &offer_item(id, 31, 4)).unwrap(); // def 400 x4 -> A (new row, max stack 5)
    let settlement = world.lock_and_accept(id).settlement.unwrap();
    let [side_a, side_b] = &settlement.sides;
    let mv = |item_id, definition_id, count| ItemMove {
        item_id,
        definition_id,
        tint: 0,
        count,
    };
    assert_eq!(side_a.gave, [mv(10, 100, 2), mv(11, 200, 2)]);
    assert_eq!(side_b.gave, [mv(30, 100, 1), mv(31, 400, 4)]);
    // A: its def-100 stack (id 10) took B's one unit; def 400 is a fresh row.
    assert_eq!(side_a.received, [mv(10, 100, 1), mv(12, 400, 4)]);
    // B: its def-100 stack (id 30) took A's two units; def 200 is a fresh row.
    assert_eq!(side_b.received, [mv(30, 100, 2), mv(32, 200, 2)]);
    let rows = |side: &osfr_tools::trade::SettlementSide| -> Vec<(i32, i32, u32)> {
        side.after
            .items
            .iter()
            .map(|i| (i.id, i.definition_id, i.count))
            .collect()
    };
    // A's emptied def-200 row (all 2 given) is gone.
    assert_eq!(rows(side_a), [(10, 100, 2), (12, 400, 4)]);
    assert_eq!(rows(side_b), [(30, 100, 6), (32, 200, 2)]);
    assert_eq!(
        totals(&[side_a.after.clone(), side_b.after.clone()]),
        totals(&before)
    );
    world.trades.settled(id, true).unwrap();
    assert_eq!(world.inv(A), side_a.after);
}

#[test]
fn a_changed_offer_resets_both_locks_and_both_accepts() {
    let mut world = World::new(
        inventory(state(40, vec![item(10, 100, 3)]), 8),
        inventory(state(40, vec![item(20, 200, 2)]), 8),
    );
    let id = world.open();
    world.send(A, &offer_item(id, 10, 1)).unwrap();
    let counter = world.counter(id);
    world.send(A, &lock(id, counter, true)).unwrap();
    world.send(B, &lock(id, counter, true)).unwrap();
    world.now += 10_000;
    assert!(
        world
            .send(A, &accept(id, counter))
            .unwrap()
            .messages
            .is_empty()
    );
    // B (who has not accepted) changes a coin offer: locks and A's accept are void.
    let reset = world.send(B, &offer_coins(id, 5)).unwrap().messages;
    assert_eq!(subs(&reset), [17, 17, 11, 11]);
    assert!(reset[..2].iter().all(|m| m.bytes[7] == 0));
    assert!(!world.trades.is_locked(id, A) && !world.trades.is_locked(id, B));
    // Accepting the old state is refused, and B accepting cannot complete A's old accept.
    for guid in [A, B] {
        assert_eq!(
            world.send(guid, &accept(id, counter)),
            Err(TradeError::StaleCounter)
        );
        assert_eq!(
            world.send(guid, &accept(id, world.counter(id))),
            Err(TradeError::NotBothLocked)
        );
    }
    // Locking with the old counter is refused and answered with an unlock for the popup.
    let refused = world.send(A, &lock(id, counter, true)).unwrap();
    assert_eq!(subs(&refused.messages), [17]);
    assert_eq!(refused.messages[0].bytes[7], 0);
    // Re-lock and wait: A's old accept does not count; both must accept again.
    let fresh = world.counter(id);
    world.send(A, &lock(id, fresh, true)).unwrap();
    world.send(B, &lock(id, fresh, true)).unwrap();
    world.now += 10_000;
    let only_b = world.send(B, &accept(id, fresh)).unwrap();
    assert!(only_b.settlement.is_none() && only_b.messages.is_empty());
    assert!(
        world
            .send(A, &accept(id, fresh))
            .unwrap()
            .settlement
            .is_some()
    );
    // Each kind of change resets: item removal, item quantity, unlock by the partner.
    let mut again = World::new(
        inventory(state(40, vec![item(10, 100, 3)]), 8),
        inventory(state(40, vec![]), 8),
    );
    let id = again.open();
    again.send(A, &offer_item(id, 10, 2)).unwrap();
    for change in [offer_item(id, 10, 1), offer_item(id, 10, 0)] {
        let counter = again.counter(id);
        again.send(A, &lock(id, counter, true)).unwrap();
        again.send(B, &lock(id, counter, true)).unwrap();
        again.send(A, &change).unwrap();
        assert!(!again.trades.is_locked(id, A) && !again.trades.is_locked(id, B));
    }
}

#[test]
fn unlocking_clears_the_partner_lock_and_both_accepts() {
    let mut world = World::new(
        inventory(state(10, vec![]), 8),
        inventory(state(10, vec![]), 8),
    );
    let id = world.open();
    let counter = world.counter(id);
    world.send(A, &lock(id, counter, true)).unwrap();
    world.send(B, &lock(id, counter, true)).unwrap();
    world.now += 10_000;
    world.send(A, &accept(id, counter)).unwrap();
    // The unlock may carry an old counter (the client sends it right after a server reset).
    let unlock = world
        .send(B, &lock(id, counter + 5, false))
        .unwrap()
        .messages;
    assert_eq!(subs(&unlock), [17, 17]);
    assert_eq!(
        world.send(B, &accept(id, counter)),
        Err(TradeError::NotBothLocked)
    );
}

#[test]
fn accept_before_the_timer_or_both_locks_is_refused() {
    let mut world = World::new(
        inventory(state(10, vec![]), 8),
        inventory(state(10, vec![]), 8),
    );
    let id = world.open();
    let counter = world.counter(id);
    assert_eq!(
        world.send(A, &accept(id, counter)),
        Err(TradeError::NotBothLocked)
    );
    world.send(A, &lock(id, counter, true)).unwrap();
    assert_eq!(
        world.send(A, &accept(id, counter)),
        Err(TradeError::NotBothLocked)
    );
    world.send(B, &lock(id, counter, true)).unwrap();
    world.now += 9_999;
    assert_eq!(
        world.send(A, &accept(id, counter)),
        Err(TradeError::AcceptTooEarly)
    );
    assert_eq!(
        world.send(A, &accept(id, counter + 1)),
        Err(TradeError::StaleCounter)
    );
}

// ---------------------------------------------------------------------------------------------
// Refusals and replays change nothing
// ---------------------------------------------------------------------------------------------

/// Run `request` as `guid`; the service must be byte-for-byte what it was (canary), and any
/// reply may only go to the sender.
fn refused(world: &mut World, guid: u64, request: &[u8]) -> Option<TradeError> {
    let before = world.trades.clone();
    let outcome = world.send(guid, request);
    assert_eq!(world.trades, before, "a refusal must change nothing");
    match outcome {
        Ok(outcome) => {
            assert!(outcome.settlement.is_none());
            assert!(outcome.messages.iter().all(|m| m.recipient == sess(guid)));
            None
        }
        Err(error) => Some(error),
    }
}

#[test]
fn every_refused_offer_changes_nothing_and_corrects_the_senders_row() {
    let mut owned = item(11, 100, 4);
    owned.consumed_count = 1;
    owned.stored_count = 1; // 2 available
    let mut rented = item(14, 200, 1);
    rented.rental = Some(RentalState {
        expiration: 0,
        upsells: 0,
        bundle_id: 0,
    });
    let mut a_state = state(
        20,
        vec![
            item(10, 300, 1), // no-trade definition
            owned,
            item(12, 200, 1), // equipped
            item(13, 200, 1), // on a pet
            rented,
            item(15, 400, 5),
        ],
    );
    a_state.equipment.push(EquippedItem {
        profile_id: 1,
        slot: 1,
        item_id: 12,
        category: 1,
    });
    let mut a_inventory = inventory(a_state, 8);
    a_inventory.equipped_on_pet.insert(13);
    let mut world = World::new(a_inventory, inventory(state(0, vec![item(30, 100, 1)]), 8));
    let id = world.open();
    world.send(A, &offer_item(id, 15, 5)).unwrap(); // a real offer already standing
    for (item_id, total) in [
        (10, 1),  // no-trade
        (12, 1),  // equipped
        (13, 1),  // equipped on a pet
        (14, 1),  // rental
        (11, 3),  // more than the available 2
        (999, 1), // not owned
        (30, 1),  // the PARTNER's item id
        (15, 6),  // more than owned
        (15, 99), // way more than owned
    ] {
        let counter = world.counter(id);
        let before = world.trades.clone();
        let outcome = world.send(A, &offer_item(id, item_id, total)).unwrap();
        assert_eq!(
            world.trades, before,
            "refused offer {item_id}x{total} must change nothing"
        );
        // One sub-13 correction to the sender only: side 1, the server's real total.
        assert_eq!(subs(&outcome.messages), [13]);
        let m = &outcome.messages[0];
        assert_eq!(m.recipient, sess(A));
        assert_eq!(
            u32::from_le_bytes(m.bytes[7..11].try_into().unwrap()),
            counter
        );
        assert_eq!(m.bytes[11], 1);
        let real = if item_id == 15 { 5 } else { 0 };
        assert_eq!(
            u32::from_le_bytes(m.bytes[16..20].try_into().unwrap()),
            real
        );
    }
    // Removing a row that was never offered is a quiet no-op.
    assert!(refused_ok(&mut world, A, &offer_item(id, 11, 0)).is_empty());
    // The quantity that IS available goes through.
    assert_eq!(
        subs(&world.send(A, &offer_item(id, 11, 2)).unwrap().messages),
        [12, 12]
    );
}

#[test]
fn coin_offers_beyond_the_wallet_are_refused_and_change_nothing() {
    let mut world = World::new(
        inventory(state(20, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let id = world.open();
    for coins in [21, 1_000, u32::MAX, i32::MAX as u32 + 1] {
        assert_eq!(
            refused(&mut world, A, &offer_coins(id, coins)),
            Some(TradeError::InsufficientCoins)
        );
    }
    assert_eq!(
        subs(&world.send(A, &offer_coins(id, 20)).unwrap().messages),
        [11, 11]
    );
}

#[test]
fn strangers_wrong_sessions_and_replays_change_nothing() {
    let mut world = World::new(
        inventory(state(20, vec![item(10, 100, 2)]), 8),
        inventory(state(20, vec![item(20, 200, 2)]), 8),
    );
    let id = world.open();
    world.send(A, &offer_item(id, 10, 1)).unwrap();
    // C is in the same zone and registered but not in this trade: every request is answered
    // with a lone 59/5 echoing the id C itself sent, and nothing in the service changes.
    let counter = world.counter(id);
    for request in [
        reply(id, true),
        reply(id, false),
        accept(id, counter),
        cancel(id),
        lock(id, counter, true),
        offer_item(id, 10, 2),
        offer_coins(id, 1),
    ] {
        let before = world.trades.clone();
        let outcome = world.send(C, &request).unwrap();
        assert_eq!(world.trades, before);
        assert_eq!(subs(&outcome.messages), [5]);
        assert_eq!(outcome.messages[0].recipient, sess(C));
        assert_eq!(&outcome.messages[0].bytes[3..7], &request[3..7]);
    }
    // An id nobody holds is answered the same way (it closes a stale client window).
    for request in [accept(id + 7, 1), cancel(id + 7), offer_coins(id + 7, 1)] {
        assert_eq!(subs(&refused_ok(&mut world, A, &request)), [5]);
    }
    // The invitee-only reply: the inviter cannot answer their own invite.
    let mut other = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    other.invite(A, B).unwrap();
    let pending = other.trades.session_id_for(A).unwrap();
    assert_eq!(subs(&refused_ok(&mut other, A, &reply(pending, true))), [5]);
    assert_eq!(subs(&refused_ok(&mut other, C, &reply(pending, true))), [5]);
    assert_eq!(other.trades.session_id_for(B), Some(pending));
}

/// Like `refused` for requests that are answered (Ok) without changing anything.
fn refused_ok(world: &mut World, guid: u64, request: &[u8]) -> Vec<TradeMessage> {
    let before = world.trades.clone();
    let outcome = world.send(guid, request).unwrap();
    assert_eq!(world.trades, before);
    assert!(outcome.settlement.is_none());
    outcome.messages
}

#[test]
fn malformed_and_hostile_requests_are_dropped_without_effect() {
    let mut world = World::new(
        inventory(state(20, vec![item(10, 100, 2)]), 8),
        inventory(state(20, vec![]), 8),
    );
    let id = world.open();
    let counter = world.counter(id);
    let good = [
        reply(id, true),
        accept(id, counter),
        cancel(id),
        lock(id, counter, true),
        offer_item(id, 10, 1),
        offer_coins(id, 1),
    ];
    for request in &good {
        // Every truncation and every extension is a different (wrong) length.
        for cut in 0..request.len() {
            assert!(
                refused(&mut world, A, &request[..cut]).is_some(),
                "cut {cut} of {request:?}"
            );
        }
        let mut longer = request.clone();
        longer.push(0);
        assert_eq!(refused(&mut world, A, &longer), Some(TradeError::Malformed));
    }
    // The idle sentinel, session zero, server-only subs, unknown subs, other families.
    for request in [
        accept(u32::MAX, 1),
        cancel(u32::MAX),
        cancel(0),
        pkt(2, id, &[0; 8]),
        pkt(4, id, &[0; 8]),
        pkt(5, id, &[]),
        pkt(11, id, &[0; 9]),
        pkt(15, id, &[1]),
        pkt(19, id, &[0; 8]),
        pkt(0, id, &[]),
        pkt(1, id, &[]),
        pkt(200, id, &[]),
        vec![0x3c, 0, 7, 1, 0, 0, 0],
        vec![0x3b, 1, 7, 1, 0, 0, 0],
    ] {
        assert!(refused(&mut world, A, &request).is_some(), "{request:?}");
    }
    // An item id above i32::MAX cannot be an instance id.
    let mut huge = vec![0x3b, 0, 9];
    huge.extend(id.to_le_bytes());
    huge.extend(u32::MAX.to_le_bytes());
    huge.extend(1u32.to_le_bytes());
    assert_eq!(refused(&mut world, A, &huge), Some(TradeError::Malformed));
    // A guid the service never registered.
    let before = world.trades.clone();
    assert_eq!(
        world.send(9_999, &cancel(id)),
        Err(TradeError::UnknownPlayer)
    );
    assert_eq!(world.trades, before);
}

#[test]
fn random_bytes_never_panic_or_move_anything() {
    let mut world = World::new(
        inventory(state(20, vec![item(10, 100, 2)]), 8),
        inventory(state(20, vec![item(20, 200, 2)]), 8),
    );
    let id = world.open();
    let before = [world.inv(A), world.inv(B)];
    let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for round in 0..4_000 {
        let length = (next() % 24) as usize;
        let mut bytes: Vec<u8> = (0..length).map(|_| next() as u8).collect();
        if round % 2 == 0 && bytes.len() >= 7 {
            // Plausible header: family 59, sub 3..=10, the live session id.
            bytes[..3].copy_from_slice(&[0x3b, 0, 3 + (next() % 8) as u8]);
            bytes[3..7].copy_from_slice(&id.to_le_bytes());
        }
        let guid = if round % 3 == 0 { B } else { A };
        if let Ok(outcome) = world.send(guid, &bytes) {
            assert!(outcome.settlement.is_none(), "random bytes cannot settle");
        }
        if world.free(A) {
            break; // a lucky valid cancel/decline closed it: still never a move
        }
    }
    assert_eq!([world.inv(A), world.inv(B)], before);
}

#[test]
fn invites_are_refused_for_busy_ignored_absent_and_far_players() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    let check = |world: &mut World, from, to, expect: TradeError| {
        let before = world.trades.clone();
        assert_eq!(world.invite(from, to), Err(expect));
        assert_eq!(world.trades, before);
    };
    check(&mut world, A, A, TradeError::NotParticipant);
    check(&mut world, A, 777, TradeError::UnknownPlayer);
    check(&mut world, 777, A, TradeError::UnknownPlayer);
    // Ignored.
    let name = world.social.profile(A).unwrap().name.clone();
    world.social.profile_mut(B).unwrap().ignored.insert(A, name);
    check(&mut world, A, B, TradeError::Ignored);
    world.social.profile_mut(B).unwrap().ignored.clear();
    // Out of reach, other instance.
    world
        .players
        .update_pose(sess(B), zone(1), [40.5, 0.0, 0.0], 0.0)
        .unwrap();
    check(&mut world, A, B, TradeError::OutOfReach);
    world
        .players
        .update_pose(sess(B), zone(7), [1.0, 0.0, 0.0], 0.0)
        .unwrap();
    check(&mut world, A, B, TradeError::OutOfReach);
    world
        .players
        .update_pose(sess(B), zone(1), [5.0, 0.0, 0.0], 0.0)
        .unwrap();
    // Offline in the social layer.
    world.social.logout(B).unwrap();
    check(&mut world, A, B, TradeError::PlayerOffline);
    world.social.login(B).unwrap();
    // Busy on either side (a pending invite counts).
    world.invite(A, B).unwrap();
    check(&mut world, C, A, TradeError::PlayerBusy);
    check(&mut world, C, B, TradeError::PlayerBusy);
    check(&mut world, B, C, TradeError::PlayerBusy);
    check(&mut world, A, C, TradeError::PlayerBusy);
}

#[test]
fn declining_or_a_duplicate_reply_releases_both_and_changes_nothing_more() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    world.invite(A, B).unwrap();
    let id = world.trades.session_id_for(A).unwrap();
    let declined = world.send(B, &reply(id, false)).unwrap().messages;
    assert_eq!(subs(&declined), [16]);
    assert_eq!(declined[0].recipient, sess(A));
    assert!(world.free(A) && world.free(B));
    // Replays of the decline and of an accept for the dead invite open nothing.
    for request in [reply(id, false), reply(id, true)] {
        assert_eq!(subs(&refused_ok(&mut world, B, &request)), [5]);
    }
    assert!(world.free(A) && world.free(B));
    assert!(world.invite(A, B).is_ok());
}

// ---------------------------------------------------------------------------------------------
// Cancellation
// ---------------------------------------------------------------------------------------------

fn standing_trade() -> (World, u32, [InventoryState; 2]) {
    let mut world = World::new(
        inventory(state(30, vec![item(10, 100, 3)]), 8),
        inventory(state(30, vec![item(20, 200, 3)]), 8),
    );
    let before = [world.inv(A), world.inv(B)];
    let id = world.open();
    world.send(A, &offer_item(id, 10, 1)).unwrap();
    world.send(B, &offer_coins(id, 9)).unwrap();
    (world, id, before)
}

fn assert_cancelled_clean(
    world: &mut World,
    messages: &[TradeMessage],
    statuses: [u8; 2],
    before: &[InventoryState; 2],
) {
    is_closed_clean(world, messages, statuses);
    assert_eq!([world.inv(A), world.inv(B)], *before);
}

#[test]
fn a_client_cancel_closes_both_windows_and_a_replay_does_nothing() {
    let (mut world, id, before) = standing_trade();
    let end = world.send(A, &cancel(id)).unwrap().messages;
    // T885: the canceller reads 7, the partner reads 1 ("<name> canceled the trade").
    assert_cancelled_clean(&mut world, &end, [7, 1], &before);
    assert_eq!(subs(&refused_ok(&mut world, A, &cancel(id))), [5]);
    assert_eq!(subs(&refused_ok(&mut world, B, &accept(id, 1))), [5]);
}

#[test]
fn walking_out_of_reach_cancels_with_the_plain_status() {
    let (mut world, id, before) = standing_trade();
    world
        .players
        .update_pose(sess(B), zone(1), [39.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(world.maintain().is_empty(), "still inside 40 m");
    world
        .players
        .update_pose(sess(B), zone(1), [41.0, 0.0, 0.0], 0.0)
        .unwrap();
    let end = world.maintain();
    assert_cancelled_clean(&mut world, &end, [8, 8], &before);
    assert_eq!(subs(&refused_ok(&mut world, B, &accept(id, 1))), [5]);
}

#[test]
fn zoning_into_another_instance_cancels_and_covers_a_battle_instance() {
    let (mut world, _, before) = standing_trade();
    world
        .players
        .update_pose(sess(A), zone(9), [0.0, 0.0, 0.0], 0.0)
        .unwrap();
    let end = world.maintain();
    assert_cancelled_clean(&mut world, &end, [8, 8], &before);
}

#[test]
fn a_battle_the_host_sees_cancels_a_session_and_an_invite() {
    let (mut world, _, before) = standing_trade();
    let end = world.trades.set_away(B, true);
    assert_cancelled_clean(&mut world, &end, [8, 8], &before);
    // While away B cannot be invited, cannot invite, and gets no Trade entry.
    assert_eq!(world.invite(A, B), Err(TradeError::PlayerBusy));
    assert_eq!(world.invite(B, A), Err(TradeError::PlayerBusy));
    let mut radial = PlayerRadial::default();
    assert!(open_menu(&world, &mut radial, A, B, false).is_err());
    assert!(world.trades.set_away(B, true).is_empty(), "already away");
    assert!(world.trades.set_away(B, false).is_empty());
    // Back from the battle, trading works again. An invite pending when a player goes away is
    // withdrawn (status 2 is what clears the toast).
    world.invite(A, B).unwrap();
    let end = world.trades.set_away(A, true);
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    assert!(world.free(A) && world.free(B));
    world.trades.set_away(A, false);
    // The plain hook cancels without marking anyone away.
    let end = world.trades.cancel_for(B);
    assert!(end.is_empty());
    world.invite(A, B).unwrap();
    let session = world.trades.session_id_for(A).unwrap();
    world.send(B, &reply(session, true)).unwrap();
    let end = world.trades.cancel_for(B);
    assert_eq!(closes(&end).iter().map(|c| c.1).collect::<Vec<_>>(), [8, 8]);
}

#[test]
fn the_plain_cancel_hook_covers_a_session_and_an_invite() {
    let (mut world, _, before) = standing_trade();
    let end = world.trades.cancel_for(B);
    assert_cancelled_clean(&mut world, &end, [8, 8], &before);
    // A pending invite: the toast is withdrawn from the invitee (status 2 is what clears it).
    world.invite(A, B).unwrap();
    let end = world.trades.cancel_for(A);
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    assert!(world.free(A) && world.free(B));
    assert!(
        world.trades.cancel_for(A).is_empty(),
        "nothing to cancel twice"
    );
}

#[test]
fn leaving_the_game_cancels_for_the_partner() {
    let (mut world, _, before) = standing_trade();
    world.players.leave(sess(B));
    world.social.logout(B).unwrap();
    let end = world.maintain();
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    assert_eq!(closes(&end).iter().map(|c| c.1).collect::<Vec<_>>(), [2, 2]);
    assert!(world.free(A) && world.free(B));
    assert_eq!(world.inv(A), before[0]);
    // The explicit hook does the same and forgets the snapshot.
    let (mut world, _, _) = standing_trade();
    let end = world.trades.unregister_player(A);
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    assert!(world.trades.inventory(A).is_none());
    assert!(world.free(B));
}

#[test]
fn expiry_closes_stale_invites_and_sessions_and_the_rules_are_tunable() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    world.invite(A, B).unwrap();
    world.now += 59_999;
    assert!(world.maintain().is_empty());
    world.now += 1;
    let end = world.maintain();
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    assert!(world.free(A) && world.free(B));

    let id = world.open();
    world.now += 299_999;
    assert!(world.maintain().is_empty());
    world.now += 1;
    let end = world.maintain();
    assert_eq!(closes(&end).iter().map(|c| c.1).collect::<Vec<_>>(), [8, 8]);
    assert_eq!(world.trades.session_counter(id), None);

    assert!(
        TradeService::with_rules(TradeRules {
            reach_m: 0.0,
            ..TradeRules::default()
        })
        .is_none()
    );
    assert!(
        TradeService::with_rules(TradeRules {
            reach_m: f32::NAN,
            ..TradeRules::default()
        })
        .is_none()
    );
    assert!(
        TradeService::with_rules(TradeRules {
            invite_ttl_ms: 0,
            ..TradeRules::default()
        })
        .is_none()
    );
    let tight = TradeRules {
        reach_m: 4.0,
        ..TradeRules::default()
    };
    let mut service = TradeService::with_rules(tight).unwrap();
    let player = world.players.get(sess(A)).unwrap().clone();
    service
        .register_player(&player, inventory(state(0, vec![]), 8))
        .unwrap();
    let player = world.players.get(sess(B)).unwrap().clone();
    service
        .register_player(&player, inventory(state(0, vec![]), 8))
        .unwrap();
    assert_eq!(
        service.invite(&world.players, &world.social, A, B, 0),
        Err(TradeError::OutOfReach),
        "5 m apart is outside a 4 m rule"
    );
}

#[test]
fn an_invitee_who_accepts_after_the_inviter_walked_away_gets_no_session() {
    let mut world = World::new(
        inventory(state(0, vec![]), 8),
        inventory(state(0, vec![]), 8),
    );
    world.invite(A, B).unwrap();
    let id = world.trades.session_id_for(A).unwrap();
    world
        .players
        .update_pose(sess(A), zone(1), [80.0, 0.0, 0.0], 0.0)
        .unwrap();
    let out = world.send(B, &reply(id, true)).unwrap().messages;
    assert_eq!(subs(&out), [15, 5, 15, 5]);
    assert!(world.free(A) && world.free(B));
    assert_eq!(world.trades.session_counter(id), None);
}

#[test]
fn entering_an_instanced_zone_reports_viewer_relative_statuses() {
    let (mut world, _, before) = standing_trade();
    let end = world.trades.enter_instanced_zone(B);
    assert_cancelled_clean(&mut world, &end, [8, 7], &before);
}

// ---------------------------------------------------------------------------------------------
// The settlement hand-off
// ---------------------------------------------------------------------------------------------

#[test]
fn a_session_waiting_for_the_commit_answer_cannot_be_changed_or_closed_from_outside() {
    let (mut world, id, before) = standing_trade();
    let settlement = world.lock_and_accept(id).settlement.unwrap();
    assert_eq!(settlement.session_id, id);
    let counter = world.counter(id);
    let waiting = world.trades.clone();
    for request in [
        offer_item(id, 10, 2),
        offer_coins(id, 1),
        lock(id, counter, false),
        lock(id, counter, true),
        accept(id, counter),
        cancel(id),
    ] {
        assert_eq!(
            world.send(A, &request),
            Err(TradeError::Settling),
            "{request:?}"
        );
        assert_eq!(
            world.send(B, &request),
            Err(TradeError::Settling),
            "{request:?}"
        );
        assert_eq!(world.trades, waiting);
    }
    // Housekeeping and the cancel hooks leave it for `settled`, even if time runs out or a
    // player vanishes; nothing is half-closed.
    world.now += 10_000_000;
    world
        .players
        .update_pose(sess(B), zone(5), [90.0, 0.0, 0.0], 0.0)
        .unwrap();
    assert!(world.maintain().is_empty());
    assert!(world.trades.cancel_for(A).is_empty());
    assert!(world.trades.disconnect(B).is_empty());
    assert!(world.trades.enter_instanced_zone(A).is_empty());
    assert!(world.trades.timeout(id).unwrap().is_empty());
    assert!(world.trades.unregister_player(B).is_empty());
    assert!(
        world.trades.inventory(B).is_some(),
        "the snapshot waits for the answer too"
    );
    assert_eq!(world.inv(A), before[0]);
    assert_eq!(world.inv(B), before[1]);
}

#[test]
fn a_failed_commit_cancels_and_leaves_the_snapshots_alone() {
    let (mut world, id, before) = standing_trade();
    world.lock_and_accept(id).settlement.unwrap();
    let end = world.trades.settled(id, false).unwrap();
    assert_cancelled_clean(&mut world, &end, [8, 8], &before);
}

#[test]
fn the_commit_answer_is_single_use_and_unknown_answers_change_nothing() {
    let (mut world, id, _) = standing_trade();
    // No settlement yet: the answer is refused.
    let before = world.trades.clone();
    assert_eq!(
        world.trades.settled(id, true),
        Err(TradeError::UnknownSession)
    );
    assert_eq!(
        world.trades.settled(id + 1, true),
        Err(TradeError::UnknownSession)
    );
    assert_eq!(world.trades, before);
    let settlement = world.lock_and_accept(id).settlement.unwrap();
    let end = world.trades.settled(id, true).unwrap();
    assert_eq!(subs(&end), [15, 5, 15, 5]);
    let done = world.trades.clone();
    // Replaying either answer, or any request for the finished session, changes nothing.
    assert_eq!(
        world.trades.settled(id, true),
        Err(TradeError::UnknownSession)
    );
    assert_eq!(
        world.trades.settled(id, false),
        Err(TradeError::UnknownSession)
    );
    assert_eq!(world.trades, done);
    assert_eq!(subs(&refused_ok(&mut world, A, &accept(id, 3))), [5]);
    assert_eq!(
        subs(&refused_ok(&mut world, B, &offer_item(id, 20, 1))),
        [5]
    );
    assert_eq!(world.inv(A), settlement.sides[0].after);
    assert_eq!(world.inv(B), settlement.sides[1].after);
}

#[test]
fn one_accept_cannot_be_replayed_into_a_settlement() {
    let (mut world, id, _) = standing_trade();
    let counter = world.counter(id);
    world.send(A, &lock(id, counter, true)).unwrap();
    world.send(B, &lock(id, counter, true)).unwrap();
    world.now += 10_000;
    for _ in 0..5 {
        let outcome = world.send(A, &accept(id, counter)).unwrap();
        assert!(outcome.settlement.is_none() && outcome.messages.is_empty());
    }
    assert!(
        world
            .send(B, &accept(id, counter))
            .unwrap()
            .settlement
            .is_some()
    );
}

// ---------------------------------------------------------------------------------------------
// Revalidation when the exchange is finally accepted
// ---------------------------------------------------------------------------------------------

#[test]
fn an_offer_that_became_invalid_cancels_with_the_offering_sides_status() {
    let mut world = World::new(
        inventory(state(30, vec![item(10, 100, 2)]), 8),
        inventory(state(30, vec![]), 8),
    );
    let id = world.open();
    world.send(A, &offer_item(id, 10, 2)).unwrap();
    world.send(A, &offer_coins(id, 25)).unwrap();
    // A spends the coins and loses the item elsewhere; the host syncs the new snapshot.
    world
        .trades
        .sync_inventory(A, inventory(state(1, vec![]), 8))
        .unwrap();
    let outcome = world.lock_and_accept(id);
    assert!(outcome.settlement.is_none());
    assert_eq!(closes(&outcome.messages), [(sess(A), 4), (sess(B), 3)]);
    assert!(world.free(A) && world.free(B));
    assert_eq!(world.inv(A).coins, 1);
}

#[test]
fn a_full_bag_or_an_overfull_stack_cancels_before_any_settlement() {
    // B has no room for a new row.
    let mut world = World::new(
        inventory(state(0, vec![item(10, 100, 1)]), 8),
        inventory(state(0, vec![item(20, 200, 1)]), 1),
    );
    let before = [world.inv(A), world.inv(B)];
    let id = world.open();
    world.send(A, &offer_item(id, 10, 1)).unwrap();
    let outcome = world.lock_and_accept(id);
    assert!(outcome.settlement.is_none());
    assert_eq!(closes(&outcome.messages), [(sess(A), 4), (sess(B), 3)]);
    assert_eq!([world.inv(A), world.inv(B)], before);

    // B's stack of 4 (max 5) cannot take 3 more and B has no free row: the stack is the limit.
    let mut world = World::new(
        inventory(state(0, vec![item(10, 400, 3)]), 8),
        inventory(state(0, vec![item(20, 400, 4)]), 1),
    );
    let id = world.open();
    world.send(A, &offer_item(id, 10, 3)).unwrap();
    let outcome = world.lock_and_accept(id);
    assert!(outcome.settlement.is_none());
    // Offering side is told its offer exceeds the partner's max stack (9); the partner gets 10.
    assert_eq!(closes(&outcome.messages), [(sess(A), 9), (sess(B), 10)]);
}

#[test]
fn coins_over_the_wire_ceiling_cannot_be_received() {
    let mut world = World::new(
        inventory(state(10, vec![]), 8),
        inventory(state(i32::MAX as u32, vec![]), 8),
    );
    let id = world.open();
    world.send(A, &offer_coins(id, 10)).unwrap();
    let outcome = world.lock_and_accept(id);
    assert!(outcome.settlement.is_none());
    assert_eq!(closes(&outcome.messages), [(sess(A), 4), (sess(B), 3)]);
    assert_eq!(world.inv(B).coins, i32::MAX as u32);
}

#[test]
fn an_empty_trade_settles_to_identical_states() {
    let mut world = World::new(
        inventory(state(7, vec![item(10, 100, 1)]), 8),
        inventory(state(8, vec![]), 8),
    );
    let before = [world.inv(A), world.inv(B)];
    let id = world.open();
    let settlement = world.lock_and_accept(id).settlement.unwrap();
    for (side, before) in settlement.sides.iter().zip(&before) {
        assert_eq!(&side.after, before);
        assert!(side.gave.is_empty() && side.received.is_empty());
    }
}

#[test]
fn a_receiver_needs_no_catalogue_entry_for_an_item_it_does_not_own_yet() {
    // B's snapshot knows only definition 200 (what B owns); A's definition 100 arrives anyway.
    let only_200 = TradeInventory::new(
        state(0, vec![item(20, 200, 1)]),
        BTreeMap::from([(
            200,
            TradeItemDefinition {
                no_trade: false,
                max_stack: Some(99),
            },
        )]),
        BTreeSet::new(),
        8,
    )
    .unwrap();
    let mut world = World::new(inventory(state(0, vec![item(10, 100, 2)]), 8), only_200);
    let id = world.open();
    world.send(A, &offer_item(id, 10, 2)).unwrap();
    let settlement = world.lock_and_accept(id).settlement.unwrap();
    assert_eq!(settlement.sides[1].received.len(), 1);
    assert_eq!(settlement.sides[1].after.items.len(), 2);
}
