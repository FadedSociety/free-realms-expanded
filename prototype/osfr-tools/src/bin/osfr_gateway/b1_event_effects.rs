//! B1: one real event consumable per effect kind, pressed on the item bar through the gateway's
//! own session path (`Session::dispatch_message`), seen by the owner and a neighbour, and expired
//! by the owner's session tick. Rows: `data/content/consumables.json` (`scripts/event_effects.py`).

use super::SessionKey;
use super::test_support::Rig;
use osfr_tools::item_bar::use_wire::{Target, encode_client_request_start_ability};

/// The shipped rows, the cake table their placed rows name, and the release item table (event
/// items are not in the captured catalogue).
const FILES: [&str; 4] = [
    "consumables.json",
    "placed_cakes.json",
    "../release/item-table.json",
    "../release/item-table-supplement.json",
];
const NEIGHBOUR: u64 = 2;
const PRESS: u64 = 3_000;

fn rig(tag: &str, item: i32) -> Option<(Rig, SessionKey)> {
    let mut rig = Rig::with_files(tag, item, &FILES)?;
    let near = rig.player(2, NEIGHBOUR, 2.0);
    rig.drain_owner(2_500);
    rig.drain(near, 2_500);
    Some((rig, near))
}

fn inners(messages: &[Vec<u8>]) -> Vec<Vec<u8>> {
    messages.iter().filter_map(|m| m.get(7..)).map(<[u8]>::to_vec).collect()
}

fn with_head(messages: &[Vec<u8>], head: [u8; 4]) -> Vec<Vec<u8>> {
    let mut found = inners(messages);
    found.retain(|p| p.starts_with(&head));
    found
}

fn word(packet: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(packet[at..at + 4].try_into().unwrap())
}

fn guid(packet: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(packet[at..at + 8].try_into().unwrap())
}

/// `(actor, tag, composite, source)` of each 35/41 (client reader `008d7ad0`).
fn composites(messages: &[Vec<u8>]) -> Vec<(u64, i32, i32, u64)> {
    let added = with_head(messages, [35, 0, 41, 0]);
    added
        .iter()
        .map(|p| (guid(p, 4), word(p, 12), word(p, 16), guid(p, 20)))
        .collect()
}

/// `(actor, tag)` of each 35/42 (client reader `008d7bc0`).
fn removed(messages: &[Vec<u8>]) -> Vec<(u64, i32)> {
    let gone = with_head(messages, [35, 0, 42, 0]);
    gone.iter().map(|p| (guid(p, 4), word(p, 12))).collect()
}

fn owner_guid(rig: &Rig) -> u64 {
    rig.scene.session.character_guid
}

/// The owner's session tick at `now` (the loop's), and what it sent the owner.
fn tick_owner(rig: &mut Rig, now: u64) -> Vec<Vec<u8>> {
    rig.scene.tick(&rig.options, now);
    rig.drain_owner(now)
}

fn live(rig: &Rig) -> osfr_tools::consumable_effects::Visual {
    let game = rig.scene.session.gateway.gameplay().unwrap();
    game.consumable_effects().visual()
}

#[test]
fn b1_a_vampire_cake_slice_is_a_black_bat_for_five_minutes() {
    const SLICE: i32 = 10433;
    let Some((mut rig, near)) = rig("b1-transform", SLICE) else {
        return;
    };
    let me = owner_guid(&rig);
    let owner = rig.press(PRESS);
    let models: Vec<_> = with_head(&owner, [35, 0, 14, 0])
        .iter()
        .map(|p| (word(p, 4), guid(p, 8)))
        .collect();
    assert_eq!(models, [(3316, me)], "bat_black.adr on the owner");
    assert_eq!(with_head(&owner, [38, 0, 16, 0]).len(), 1, "one buff row");
    assert_eq!(rig.count(SLICE), None, "the single slice was eaten");
    assert_eq!(with_head(&rig.drain(near, PRESS), [35, 0, 14, 0]).len(), 1);
    assert_eq!(live(&rig).temporary_model, 3316);

    assert!(with_head(&tick_owner(&mut rig, PRESS + 299_000), [35, 0, 15, 0]).is_empty());
    let ended = tick_owner(&mut rig, PRESS + 300_000);
    assert_eq!(with_head(&ended, [35, 0, 15, 0]).len(), 1, "the bat reverts");
    assert_eq!(with_head(&ended, [38, 0, 17, 0]).len(), 1, "and its buff row goes");
    assert_eq!(live(&rig).temporary_model, 0);
}

#[test]
fn b1_candy_hearts_put_an_aura_of_hearts_on_for_two_minutes() {
    const HEARTS: i32 = 76801;
    let Some((mut rig, near)) = rig("b1-aura", HEARTS) else {
        return;
    };
    let me = owner_guid(&rig);
    let owner = rig.press(PRESS);
    let added = composites(&owner);
    assert_eq!(added.len(), 1);
    assert_eq!((added[0].0, added[0].2), (me, 16462), "the candy-hearts loop");
    assert_eq!(composites(&rig.drain(near, PRESS)), added);

    let ended = tick_owner(&mut rig, PRESS + 120_000);
    assert_eq!(removed(&ended), [(me, added[0].1)]);
    assert_eq!(with_head(&ended, [38, 0, 17, 0]).len(), 1);
    assert!(live(&rig).auras.is_empty());
}

#[test]
fn b1_a_gingerbread_cookie_makes_the_player_small_for_thirty_seconds() {
    const COOKIE: i32 = 76622;
    let Some((mut rig, near)) = rig("b1-scale", COOKIE) else {
        return;
    };
    let scale = |messages: &[Vec<u8>]| -> Vec<f32> {
        let sized = with_head(messages, [35, 0, 13, 0]);
        let value = |p: &Vec<u8>| f32::from_le_bytes(p[12..16].try_into().unwrap());
        sized.iter().map(value).collect()
    };
    assert_eq!(scale(&rig.press(PRESS)), [0.5]);
    assert_eq!(scale(&rig.drain(near, PRESS)), [0.5]);
    assert_eq!(live(&rig).scale, 0.5);
    assert_eq!(scale(&tick_owner(&mut rig, PRESS + 30_000)), [1.0]);
    assert_eq!(live(&rig).scale, 1.0);
}

#[test]
fn b1_a_birthday_firework_goes_off_on_the_player_and_ends_without_a_buff_row() {
    const FIREWORK: i32 = 77435;
    let Some((mut rig, near)) = rig("b1-burst", FIREWORK) else {
        return;
    };
    let me = owner_guid(&rig);
    let owner = rig.press(PRESS);
    let added = composites(&owner);
    assert_eq!(added.len(), 1);
    assert_eq!((added[0].0, added[0].2, added[0].3), (me, 16517, me));
    assert!(with_head(&owner, [38, 0, 16, 0]).is_empty(), "a burst is not a buff");
    assert_eq!(composites(&rig.drain(near, PRESS)), added);
    assert_eq!(rig.count(FIREWORK), None, "the firework was spent");

    let ended = tick_owner(&mut rig, PRESS + 10_000);
    assert_eq!(removed(&ended), [(me, added[0].1)]);
    assert!(with_head(&ended, [38, 0, 17, 0]).is_empty());
}

#[test]
fn b1_blue_silly_string_sprays_the_aimed_player_from_the_user() {
    const STRING: i32 = 77439;
    let Some((mut rig, near)) = rig("b1-spray", STRING) else {
        return;
    };
    let me = owner_guid(&rig);
    let aimed = encode_client_request_start_ability(2, 0, Target::Guid(NEIGHBOUR));
    rig.scene.send(&rig.options, &aimed, PRESS).unwrap();
    let owner = rig.drain_owner(PRESS);
    let added = composites(&owner);
    assert_eq!(added.len(), 1, "{owner:?}");
    // The beam plays on the neighbour and draws from the user (the p2p source).
    assert_eq!((added[0].0, added[0].2, added[0].3), (NEIGHBOUR, 15825, me));
    assert_eq!(composites(&rig.drain(near, PRESS)), added);

    let ended = tick_owner(&mut rig, PRESS + 5_000);
    assert_eq!(removed(&ended), [(NEIGHBOUR, added[0].1)]);
    let ended = rig.drain(near, PRESS + 5_000);
    assert_eq!(removed(&ended), [(NEIGHBOUR, added[0].1)]);
}

#[test]
fn b1_a_heavenly_donut_raises_timing_for_thirty_minutes_on_the_owner_only() {
    const DONUT: i32 = 60269;
    let Some((mut rig, near)) = rig("b1-stat", DONUT) else {
        return;
    };
    let me = owner_guid(&rig);
    // 38/7: guid, one record of (stat id, kind 0 integer, value) (reader `008e7640`).
    let timing = |messages: &[Vec<u8>]| -> Vec<(u64, i32, i32, i32)> {
        let stats = with_head(messages, [38, 0, 7, 0]);
        // After the guid, a record count (1 for ours), then id, kind and value.
        let record = |p: &Vec<u8>| (guid(p, 4), word(p, 16), word(p, 20), word(p, 24));
        let ours = stats.iter().filter(|p| word(p, 12) == 1);
        ours.map(record).filter(|r| r.1 == 27).collect()
    };
    let owner = rig.press(PRESS);
    assert_eq!(timing(&owner), [(me, 27, 0, 2)], "Timing +2");
    assert_eq!(with_head(&owner, [38, 0, 16, 0]).len(), 1, "one buff row");
    assert!(timing(&rig.drain(near, PRESS)).is_empty(), "a stat is the owner's");

    assert!(timing(&tick_owner(&mut rig, PRESS + 1_799_000)).is_empty());
    let ended = tick_owner(&mut rig, PRESS + 1_800_000);
    assert_eq!(timing(&ended), [(me, 27, 0, 0)], "back to 0");
    assert_eq!(with_head(&ended, [38, 0, 17, 0]).len(), 1);
}

#[test]
fn b1_a_vanilla_cupcake_heals_a_fifth_of_health_once() {
    const CUPCAKE: i32 = 77437;
    let Some((mut rig, _near)) = rig("b1-heal", CUPCAKE) else {
        return;
    };
    let health = |rig: &Rig| {
        let game = rig.scene.session.gateway.gameplay().unwrap();
        game.state().encounter.map(|e| e.player_health)
    };
    let game = rig.scene.session.gateway.gameplay_mut().unwrap();
    game.receive_field_damage(60, 2_600, true).unwrap();
    let hurt = health(&rig).expect("the open world has an encounter engine");
    let owner = rig.press(PRESS);
    assert!(
        with_head(&owner, [38, 0, 1, 0]).len() == 1,
        "the HUD gets its 38/1"
    );
    let healed = health(&rig).unwrap();
    assert!(healed > hurt, "{hurt} -> {healed}");
    assert_eq!(rig.count(CUPCAKE), None, "the cupcake was eaten");
    // A heal leaves nothing live to expire.
    assert!(with_head(&tick_owner(&mut rig, PRESS + 60_000), [38, 0, 17, 0]).is_empty());
}

#[test]
fn b1_turkleton_treats_are_placed_and_a_click_turns_the_clicker_into_a_bear_for_ten_minutes() {
    const TREATS: i32 = 77330;
    let Some(mut rig) = Rig::with_files("b1-placed", TREATS, &FILES) else {
        return;
    };
    // Relative, as `test_support` reaches it: `tests/bot_smoke.rs` and
    // `tests/playtest_gateway_smoke.rs` include the gateway as a module, where `crate::` is theirs.
    rig.scene.world.cakes = super::super::cake_routing::load(&rig.options).unwrap();
    rig.drain_owner(2_500);
    let me = owner_guid(&rig);
    let placed = rig.press(PRESS);
    let spawned = with_head(&placed, [35, 0, 2, 0]);
    let basket = spawned
        .iter()
        .map(|p| guid(p, 4))
        .find(|g| osfr_tools::placed_cakes::is_cake(*g))
        .expect("the treat basket is an AddNpc");
    assert_eq!(rig.count(TREATS), Some(1), "a placed prop is not spent");

    let click = osfr_tools::npc_interaction::NpcInteractRequest {
        target: basket,
        trailing_flag: false,
    }
    .encode();
    rig.scene.send(&rig.options, &click, PRESS + 1_000).unwrap();
    let clicked = rig.drain_owner(PRESS + 1_000);
    let models: Vec<_> = with_head(&clicked, [35, 0, 14, 0])
        .iter()
        .map(|p| (word(p, 4), guid(p, 8)))
        .collect();
    assert_eq!(models, [(2033, me)], "the chocolate bear");
    assert_eq!(live(&rig).temporary_model, 2033);

    let ended = tick_owner(&mut rig, PRESS + 1_000 + 600_000);
    assert_eq!(with_head(&ended, [35, 0, 15, 0]).len(), 1, "ten minutes, then back");
    assert_eq!(live(&rig).temporary_model, 0);
}
