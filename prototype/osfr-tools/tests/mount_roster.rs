//! T424: the owned-ride roster (168/5, client 00b06100 case 5 -> 00b05830 -> 008e74c0 records)
//! at world entry and after every ride ownership change, and the ride put away when its item is
//! traded away.
//!
//! A "message" is `handle_in_world` followed by `settle_mounts`, and a "tick" is `tick` followed
//! by `settle_mounts`: the order in which the gateway commits them (`commit_outcome` runs
//! `settle_rides` after every message, shared-field action and tick).
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    inventory::{InventoryItem, InventoryState},
    mounts::{MountCatalog, MountService, RideRestrictions},
};
use std::path::PathBuf;

// Ride items the captured reference catalogue defines (Lava Tiger has two captured items).
const WOLF_ITEM: i32 = 14289;
const BAT_ITEM: i32 = 14288;
const LAVA_TIGER_ITEMS: [i32; 2] = [17240, 76906];
const NOW: u64 = 10_000;
const PUT_AWAY: [u8; 3] = [168, 0, 3];

fn reference() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../lab/references/rust-opening-cooking-original-assets")
}
fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}
fn actor() -> u64 {
    MountService::ride_guid(1)
}

/// A game owning `items` (stacks of one), before world entry, with the dev commands enabled.
fn game_with(items: &[i32]) -> Gameplay {
    let mut state = Gameplay::load(&reference(), None).unwrap().state();
    let first = state
        .inventory
        .items
        .iter()
        .map(|i| i.id)
        .max()
        .unwrap_or(0)
        + 1000;
    state.inventory.items.extend(stacks(first, items));
    let mut game = Gameplay::load(&reference(), Some(state)).unwrap();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game
}
fn stacks(first_id: i32, items: &[i32]) -> impl Iterator<Item = InventoryItem> + '_ {
    items
        .iter()
        .zip(first_id..)
        .map(|(definition_id, id)| InventoryItem {
            definition_id: *definition_id,
            tint: 0,
            id,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        })
}
fn enter(game: &mut Gameplay) -> Outcome {
    game.initialize_world(&who(), 0).unwrap()
}
fn settled(game: &mut Gameplay, mut out: Outcome) -> Outcome {
    let settled = game.settle_mounts(&who(), false);
    out.replies.extend(settled.replies);
    out.nearby.extend(settled.nearby);
    out
}
fn message(game: &mut Gameplay, inner: &[u8], at: u64) -> Outcome {
    let out = game.handle_in_world(inner, &who(), at).unwrap();
    settled(game, out)
}
fn tick(game: &mut Gameplay, at: u64) -> Outcome {
    let out = game.tick(&who(), at).unwrap();
    settled(game, out)
}
fn give(item: i32) -> Vec<u8> {
    chat::local_request(&format!("realm give {item}"))
}
/// T482: 168/6 names a roster row, which is an inventory instance: the lowest id the game gave
/// this item.
fn row(game: &Gameplay, item: i32) -> i32 {
    let rows = game.inventory_state().items.iter();
    rows.filter(|i| i.definition_id == item)
        .map(|i| i.id)
        .min()
        .unwrap()
}
fn spawn(row: i32) -> Vec<u8> {
    [vec![168, 0, 6], row.to_le_bytes().to_vec()].concat()
}
fn board() -> Vec<u8> {
    [
        vec![168, 0, 1],
        actor().to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat()
}
/// The trade path: adopt the settlement's validated `after` inventory, here without `item`.
fn trade_away(game: &mut Gameplay, item: i32) {
    let mut after = game.inventory_state().clone();
    after.items.retain(|row| row.definition_id != item);
    game.adopt_trade_inventory(after).unwrap();
}
fn removes_ride(packet: &[u8]) -> bool {
    packet.starts_with(&[35, 0, 3, 0, 1, 0]) && packet[6..14] == actor().to_le_bytes()
}

/// One 168/5 record as `008e74c0` reads it.
#[derive(Debug, PartialEq)]
struct Row {
    id: i32,
    name: i32,
    image: i32,
    guid: u64,
    members_only: u8,
    tint: i32,
    tint_alias: String,
    upgradable: u8,
    upgraded: u8,
}
fn decode(packet: &[u8]) -> Vec<Row> {
    let mut at = 3;
    let mut take = |n: usize| {
        at += n;
        &packet[at - n..at]
    };
    let i32_at = |bytes: &[u8]| i32::from_le_bytes(bytes.try_into().unwrap());
    let count = i32_at(take(4));
    let rows = (0..count)
        .map(|_| Row {
            id: i32_at(take(4)),
            name: i32_at(take(4)),
            image: i32_at(take(4)),
            guid: u64::from_le_bytes(take(8).try_into().unwrap()),
            members_only: take(1)[0],
            tint: i32_at(take(4)),
            tint_alias: {
                let len = i32_at(take(4)) as usize;
                String::from_utf8(take(len).to_vec()).unwrap()
            },
            upgradable: take(1)[0],
            upgraded: take(1)[0],
        })
        .collect();
    assert_eq!(at, packet.len(), "no trailing bytes");
    rows
}
/// Every 168/5 the owner got, decoded.
fn rosters(out: &Outcome) -> Vec<Vec<Row>> {
    let lists = out.replies.iter().filter(|r| r.starts_with(&[168, 0, 5]));
    lists.map(|r| decode(r)).collect()
}
/// (ride id, actor guid) of every row of every 168/5.
fn rides(out: &Outcome) -> Vec<Vec<(i32, u64)>> {
    let rows = |list: Vec<Row>| -> Vec<_> { list.iter().map(|row| (row.id, row.guid)).collect() };
    rosters(out).into_iter().map(rows).collect()
}

#[test]
fn world_entry_sends_the_owned_ride_roster() {
    let mut game = game_with(&[WOLF_ITEM]);
    let entry = enter(&mut game);
    let wolf = Row {
        id: row(&game, WOLF_ITEM),
        name: 374,
        image: 6873,
        guid: 0,
        members_only: 0,
        tint: 0,
        tint_alias: "dyetint".into(),
        upgradable: 0,
        upgraded: 0,
    };
    assert_eq!(rosters(&entry), vec![vec![wolf]]);
    // The helper still sends the pet roster (53/5) beside it.
    assert!(entry.replies.iter().any(|r| r.starts_with(&[53, 0, 5])));
    assert!(rosters(&tick(&mut game, NOW)).is_empty(), "nothing changed");

    // No rides: count 0, which also replaces the captured self-data rows.
    let mut none = game_with(&[]);
    assert_eq!(rosters(&enter(&mut none)), vec![vec![]]);
}

#[test]
fn grants_send_the_roster_once_and_nothing_else_does() {
    let mut game = game_with(&[]);
    let other_item = game.state().inventory.items[0].definition_id;
    enter(&mut game);
    let given = message(&mut game, &give(WOLF_ITEM), NOW);
    let wolf = row(&game, WOLF_ITEM);
    assert_eq!(rides(&given), vec![vec![(wolf, 0)]], "realm give");
    // A second Wolf item (it stacks into the same instance, so no new row), a non-ride grant and
    // an idle tick change no ride.
    assert!(rosters(&message(&mut game, &give(WOLF_ITEM), NOW + 10)).is_empty());
    assert!(rosters(&message(&mut game, &give(other_item), NOW + 20)).is_empty());
    assert!(rosters(&tick(&mut game, NOW + 30)).is_empty());
    // A later summon sends its own roster, with the actor, and the sync adds no duplicate.
    let summoned = message(&mut game, &spawn(wolf), NOW + 40);
    assert_eq!(rides(&summoned), vec![vec![(wolf, actor())]]);
    assert!(rosters(&tick(&mut game, NOW + 50)).is_empty());
}

/// The summon, put-away and refusal rosters each record what the client now holds. Each is sent
/// while a grant is still unsynced, so a send that did not record it would be followed by a second
/// roster from the sync.
#[test]
fn summon_put_away_and_refusal_rosters_leave_nothing_for_the_sync() {
    // Summon.
    let mut game = game_with(&[]);
    enter(&mut game);
    game.handle_in_world(&give(WOLF_ITEM), &who(), NOW).unwrap();
    let wolf = row(&game, WOLF_ITEM);
    let out = message(&mut game, &spawn(wolf), NOW + 10);
    assert_eq!(rides(&out), vec![vec![(wolf, actor())]], "summon");

    // Put away, after a second ride arrived.
    let mut game = game_with(&[WOLF_ITEM]);
    enter(&mut game);
    let wolf = row(&game, WOLF_ITEM);
    message(&mut game, &spawn(wolf), NOW);
    game.handle_in_world(&give(BAT_ITEM), &who(), NOW + 10)
        .unwrap();
    let bat = row(&game, BAT_ITEM);
    let out = message(&mut game, &PUT_AWAY, NOW + 20);
    let mut both = vec![(bat, 0), (wolf, 0)];
    both.sort();
    assert_eq!(rides(&out), vec![both], "put away");

    // Refused summon of a ride not owned.
    let mut game = game_with(&[]);
    enter(&mut game);
    game.handle_in_world(&give(WOLF_ITEM), &who(), NOW).unwrap();
    let wolf = row(&game, WOLF_ITEM);
    let out = message(&mut game, &spawn(wolf + 1_000_000), NOW + 10);
    assert!(
        out.replies.iter().any(|r| r.starts_with(&[15, 0, 3, 0])),
        "notice"
    );
    assert_eq!(rides(&out), vec![vec![(wolf, 0)]], "refusal");
}

#[test]
fn a_new_session_sends_the_roster_again_at_world_entry_and_nothing_before() {
    let mut game = game_with(&[WOLF_ITEM]);
    assert!(
        rosters(&game.settle_mounts(&who(), false)).is_empty(),
        "before entry"
    );
    enter(&mut game);
    let wolf = row(&game, WOLF_ITEM);
    message(&mut game, &spawn(wolf), NOW);
    game.reset_session();
    // The ride did not survive the disconnect; nothing is sent until the new world entry.
    assert!(
        rosters(&game.settle_mounts(&who(), false)).is_empty(),
        "after reset"
    );
    assert_eq!(rides(&enter(&mut game)), vec![vec![(wolf, 0)]]);
}

#[test]
fn losing_the_ride_item_puts_the_ride_away() {
    for mounted in [false, true] {
        let mut game = game_with(&[WOLF_ITEM, BAT_ITEM]);
        enter(&mut game);
        let bat = row(&game, BAT_ITEM);
        let wolf = row(&game, WOLF_ITEM);
        message(&mut game, &spawn(wolf), NOW);
        if mounted {
            message(&mut game, &board(), NOW + 10);
            assert!(game.is_mounted());
        }
        trade_away(&mut game, WOLF_ITEM);
        let out = tick(&mut game, NOW + 100);
        // Observers: one despawn, and the dismount when the owner was riding.
        assert_eq!(out.nearby.iter().filter(|r| removes_ride(r)).count(), 1);
        let dismounts = out.nearby.iter().filter(|r| r.starts_with(&[168, 0, 4]));
        assert_eq!(
            dismounts.count(),
            usize::from(mounted),
            "mounted: {mounted}"
        );
        // Owner: exactly one roster, the remaining ride and no active guid.
        assert_eq!(rides(&out), vec![vec![(bat, 0)]], "mounted: {mounted}");
        assert!(!game.is_mounted() && game.ride_view(1).is_none());
        let after = tick(&mut game, NOW + 200);
        assert!(rosters(&after).is_empty() && !after.nearby.iter().any(|r| removes_ride(r)));
    }

    // T482: a ride is one item instance. Another owned item of the same ride is another row: losing
    // it leaves the ride out (the roster loses that row and nothing else is sent), and losing the
    // summoned item puts the ride away.
    for mounted in [false, true] {
        let mut game = game_with(&LAVA_TIGER_ITEMS);
        enter(&mut game);
        let summoned = row(&game, LAVA_TIGER_ITEMS[0]);
        message(&mut game, &spawn(summoned), NOW);
        if mounted {
            message(&mut game, &board(), NOW + 10);
        }
        trade_away(&mut game, LAVA_TIGER_ITEMS[1]);
        let out = tick(&mut game, NOW + 100);
        assert_eq!(rides(&out), vec![vec![(summoned, actor())]]);
        assert!(out.nearby.is_empty());
        assert_eq!(game.is_mounted(), mounted);
        assert!(game.ride_view(1).is_some());
        trade_away(&mut game, LAVA_TIGER_ITEMS[0]);
        let out = tick(&mut game, NOW + 200);
        assert_eq!(out.nearby.iter().filter(|r| removes_ride(r)).count(), 1);
        assert!(!game.is_mounted() && game.ride_view(1).is_none());
    }
}

/// The brief's own example, at the service level: the captured reference catalogue lacks these two
/// Scaled Dragon items, so `Gameplay` cannot hold them in a test. T482: the ride is the instance
/// it was summoned from (row 2, item 76856); the other item (row 1) is only another row.
#[test]
fn a_scaled_dragon_stays_out_while_another_of_its_items_is_owned() {
    let catalog = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json");
    let mut service = MountService::new(MountCatalog::load(catalog).unwrap());
    let owned = |items: &[i32]| InventoryState {
        coins: 0,
        items: stacks(1, items).collect(),
        equipment: Vec::new(),
        next_instance_id: None,
    };
    let free = RideRestrictions {
        zone_allows_riding: true,
        in_minigame: false,
        ability_active: false,
        knocked_out: false,
    };
    let both = owned(&[7960, 76856]);
    service.list_packet(&both); // world entry
    let summoned = service.spawn_row(1, &both, 2, actor(), free, NOW, &[]);
    assert!(summoned.unwrap().changed);
    let kept = InventoryState {
        items: stacks(2, &[76856]).collect(),
        ..owned(&[])
    };
    // The roster loses row 1 for the owner only; the ride stays out.
    let lost_sibling = service.settle(1, &kept, false);
    assert_eq!(lost_sibling.effects.len(), 1);
    assert!(!lost_sibling.effects[0].nearby());
    assert!(service.state().active_ride().is_some());
    // The last one gone: put away, with one roster.
    let ended = service.settle(1, &owned(&[]), false);
    assert!(service.state().active_ride().is_none());
    let lists = ended.effects.iter().filter(|e| !e.nearby());
    assert_eq!(lists.count(), 1);
}
