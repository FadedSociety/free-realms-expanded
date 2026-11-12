//! B1: one real event consumable per effect kind, pressed on the item bar through the gateway's
//! own session path (`Session::dispatch_message`), seen by the owner and a neighbour, and expired
//! by the owner's session tick. Rows: `data/content/consumables.json` (`scripts/event_effects.py`).
//! C3: the same for the item-behaviour audit's potions, foods, scrolls, orbs and props (below).

#[path = "../../../tests/common/round70_content.rs"]
#[allow(dead_code)] // the round2 table uses the paths, not the fixture builder
mod round70_content;
use super::SessionKey;
use super::test_support::{self, Rig, Scratch, Table};
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

// C3: the item-behaviour audit's consumables that did nothing (`scripts/event_effects.py` C3
// rows), one per effect kind, pressed with the client's own 36/10 (or its 42/16 and 42/15).

fn health(rig: &Rig) -> u32 {
    let game = rig.scene.session.gateway.gameplay().unwrap();
    game.state().encounter.unwrap().player_health
}

/// The owner takes a field hit, as B1's cupcake test does; returns the health left.
fn hurt(rig: &mut Rig) -> u32 {
    let game = rig.scene.session.gateway.gameplay_mut().unwrap();
    game.receive_field_damage(200, 2_600, true).unwrap();
    health(rig)
}

/// `(actor, composite)` of each 35/16 played effect (encoder `encode_play_composite_effect`).
fn played(messages: &[Vec<u8>]) -> Vec<(u64, i32)> {
    let plays = with_head(messages, [35, 0, 16, 0]);
    plays.iter().map(|p| (guid(p, 4), word(p, 20))).collect()
}

/// `(guid, kind, value bits)` of each one-record 38/7 for `stat` (reader `008e7640`).
fn stat_records(messages: &[Vec<u8>], stat: i32) -> Vec<(u64, i32, i32)> {
    let stats = with_head(messages, [38, 0, 7, 0]);
    let ours = stats.iter().filter(|p| word(p, 12) == 1 && word(p, 16) == stat);
    ours.map(|p| (guid(p, 4), word(p, 20), word(p, 24))).collect()
}

/// The shipped row of `item`, as the rig's content directory holds it.
fn shipped(rig: &Rig, item: i32) -> osfr_tools::item_bar::EffectKind {
    let content = rig.options.content.as_ref().unwrap();
    let effects = osfr_tools::item_bar::Effects::load(content).unwrap();
    effects.get(item).unwrap().effect
}

#[test]
fn c3_a_jumbo_replenishing_potion_heals_at_once_and_shows_its_cast() {
    const JUMBO: i32 = 2677;
    let Some((mut rig, near)) = rig("c3-replenish", JUMBO) else {
        return;
    };
    let me = owner_guid(&rig);
    let hurt = hurt(&mut rig);
    let owner = rig.press(PRESS);
    assert_eq!(with_head(&owner, [38, 0, 1, 0]).len(), 1, "the HUD gets its 38/1");
    assert!(health(&rig) > hurt, "{hurt} -> {}", health(&rig));
    assert_eq!(played(&owner), [(me, 16326)], "the purple replenish cast");
    assert_eq!(played(&rig.drain(near, PRESS)), [(me, 16326)], "seen by the neighbour");
    assert_eq!(rig.count(JUMBO), None, "the one potion was drunk");
}

#[test]
fn c3_a_heal_over_time_potion_heals_a_step_a_second() {
    const POTION: i32 = 30;
    let Some((mut rig, _near)) = rig("c3-hot", POTION) else {
        return;
    };
    let hurt = hurt(&mut rig);
    let owner = rig.press(PRESS);
    assert!(with_head(&owner, [38, 0, 1, 0]).is_empty(), "nothing at once");
    assert_eq!(health(&rig), hurt);
    assert_eq!(rig.count(POTION), None);
    // The engine ticks at most every 100 ms, so look just before the first second.
    assert!(with_head(&tick_owner(&mut rig, PRESS + 900), [38, 0, 1, 0]).is_empty());
    let first = tick_owner(&mut rig, PRESS + 1_000);
    assert_eq!(with_head(&first, [38, 0, 1, 0]).len(), 1, "the first step");
    let stepped = health(&rig);
    assert!(stepped > hurt, "{hurt} -> {stepped}");
    let over = match shipped(&rig, POTION) {
        osfr_tools::item_bar::EffectKind::Restore { over_ms, .. } => over_ms,
        other => panic!("{other:?}"),
    };
    tick_owner(&mut rig, PRESS + over);
    let done = health(&rig);
    assert!(done > stepped, "the rest over the potion's {over} ms");
    tick_owner(&mut rig, PRESS + 2 * over);
    assert_eq!(health(&rig), done, "and no more after it");
}

#[test]
fn c3_a_crazy_coin_scroll_raises_coins_for_its_time() {
    const SCROLL: i32 = 2654;
    let Some((mut rig, _near)) = rig("c3-boost", SCROLL) else {
        return;
    };
    let owner = rig.press(PRESS);
    assert_eq!(with_head(&owner, [38, 0, 16, 0]).len(), 1, "one buff row");
    assert_eq!(rig.count(SCROLL), None);
    let boost = |rig: &Rig| {
        let game = rig.scene.session.gateway.gameplay().unwrap();
        game.consumable_effects().boost().coins
    };
    assert!(boost(&rig) > 0, "coins are boosted");
    let length = match shipped(&rig, SCROLL) {
        osfr_tools::item_bar::EffectKind::Boost { duration_ms, .. } => duration_ms,
        other => panic!("{other:?}"),
    };
    assert!(with_head(&tick_owner(&mut rig, PRESS + length - 1), [38, 0, 17, 0]).is_empty());
    let ended = tick_owner(&mut rig, PRESS + length);
    assert_eq!(with_head(&ended, [38, 0, 17, 0]).len(), 1, "the row goes on time");
    assert_eq!(boost(&rig), 0);
}

#[test]
fn c3_a_talent_food_raises_talent_on_the_owner() {
    const FOOD: i32 = 21811; // Tangy Tumtumnip Mishmash
    let Some((mut rig, near)) = rig("c3-talent", FOOD) else {
        return;
    };
    let me = owner_guid(&rig);
    let owner = rig.press(PRESS);
    let talent = stat_records(&owner, 24);
    assert_eq!(talent.len(), 1, "{owner:?}");
    assert_eq!((talent[0].0, talent[0].1), (me, 0), "an integer stat on the owner");
    assert!(talent[0].2 > 0);
    assert!(stat_records(&rig.drain(near, PRESS), 24).is_empty());
    assert_eq!(rig.count(FOOD), None);
}

#[test]
fn c3_darkthornes_embrace_guards_without_a_stat_and_refills_instead_of_running_out() {
    const EMBRACE: i32 = 1914;
    let Some((mut rig, _near)) = rig("c3-defense", EMBRACE) else {
        return;
    };
    let mitigation = |rig: &Rig| rig.scene.session.gateway.field_participant().unwrap().mitigation;
    assert_eq!(mitigation(&rig), 1.0, "no guard before");
    let owner = rig.press(PRESS);
    assert_eq!(with_head(&owner, [38, 0, 16, 0]).len(), 1, "one buff row");
    assert!(stat_records(&owner, 18).is_empty(), "a combat stat is the server's");
    assert_eq!(rig.count(EMBRACE), Some(1), "it refills: never spent");
    // A field hit is the enemy's damage times this (`field.rs` `attack_damage`).
    let percent = match shipped(&rig, EMBRACE) {
        osfr_tools::item_bar::EffectKind::Stat { value, .. } => value as f32,
        other => panic!("{other:?}"),
    };
    assert!((mitigation(&rig) - (1.0 - percent / 100.0)).abs() < 1e-6, "hits land smaller");
    let again = rig.press(PRESS + 5_000);
    assert_eq!(with_head(&again, [36, 0, 1, 0]).len(), 1, "still refilling");
    assert!(with_head(&again, [38, 0, 16, 0]).is_empty());
}

#[test]
fn c3_a_lilipad_lollipop_makes_the_player_a_frog_for_its_time() {
    const LOLLIPOP: i32 = 2586;
    let Some((mut rig, near)) = rig("c3-frog", LOLLIPOP) else {
        return;
    };
    let me = owner_guid(&rig);
    let owner = rig.press(PRESS);
    let (model, length) = match shipped(&rig, LOLLIPOP) {
        osfr_tools::item_bar::EffectKind::Transform {
            model_id,
            duration_ms,
            ..
        } => (model_id, duration_ms),
        other => panic!("{other:?}"),
    };
    let models: Vec<_> = with_head(&owner, [35, 0, 14, 0])
        .iter()
        .map(|p| (word(p, 4), guid(p, 8)))
        .collect();
    assert_eq!(models, [(model, me)], "the frog on the owner");
    assert_eq!(with_head(&rig.drain(near, PRESS), [35, 0, 14, 0]).len(), 1);
    assert!(stat_records(&owner, 68).is_empty(), "no jump stat");
    assert_eq!(rig.count(LOLLIPOP), None);
    let ended = tick_owner(&mut rig, PRESS + length);
    assert_eq!(with_head(&ended, [35, 0, 15, 0]).len(), 1, "back after its time");
}

#[test]
fn c3_a_lure_outside_fishing_is_refused_with_a_reason_and_kept() {
    const LURE: i32 = 68152; // 16oz Steak
    let Some((mut rig, _near)) = rig("c3-lure", LURE) else {
        return;
    };
    let owner = rig.press(PRESS);
    assert_eq!(with_head(&owner, [36, 0, 1, 0]).len(), 1, "the client's cast failure");
    let said = inners(&owner)
        .iter()
        .any(|p| String::from_utf8_lossy(p).contains("Use a lure while you are fishing."));
    assert!(said, "and the reason: {owner:?}");
    assert!(with_head(&owner, [38, 0, 16, 0]).is_empty(), "no buff row");
    assert_eq!(rig.count(LURE), Some(1), "not spent");
}

// C3 round 2: two brawlers beside the field wolf on one gateway (`test_support::Table`), every
// packet through `Session::dispatch_message` and `commit_outcome`, as the socket takes it.

const WOLF: u64 = 131000102001;
const WOLF_AT: [f32; 3] = [-1899.204, -50.59, 307.902];
const BRAWLER: i32 = 43;

/// The table, and the instance id of each `(definition, count)` the first player holds.
fn c3_table(tag: &str, items: &[(i32, u32)]) -> Option<(Table, Vec<i32>)> {
    if !test_support::bin_test_target("c3") {
        return None;
    }
    let scratch = Scratch::new(tag);
    let content = test_support::repo_and_runtime_content(&scratch, &round70_content::source());
    let release = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release");
    for name in ["item-table.json", "item-table-supplement.json"] {
        std::fs::copy(release.join(name), content.join(name)).unwrap();
    }
    let mut ids = Vec::new();
    let mut players = Vec::new();
    for index in 0..2_u64 {
        let reference = round70_content::reference();
        let mut character = osfr_tools::gateway::Gateway::reference_state(&reference).unwrap();
        character.guid = index + 1;
        character.zone_id = 0;
        character.position = [WOLF_AT[0] + 1.0 + index as f32, WOLF_AT[1], WOLF_AT[2]];
        let mut game = osfr_tools::gameplay::Gameplay::load(&content, None).unwrap();
        game.initialize_world(&character, 0).unwrap();
        let mut saved = game.state();
        let profiles = saved.profiles.as_mut().unwrap();
        profiles.unlocked.insert(BRAWLER);
        profiles.active = BRAWLER;
        let held = saved.inventory.items.iter().map(|i| i.id).max().unwrap_or(0) + 1000;
        for (n, &(definition_id, count)) in items.iter().enumerate() {
            let id = held + n as i32;
            saved.inventory.items.push(osfr_tools::inventory::InventoryItem {
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
            });
            if index == 0 {
                ids.push(id);
            }
        }
        players.push((character, saved));
    }
    let mut table = Table::new(scratch, &content, &round70_content::reference(), &players)?;
    table.tick(1_000);
    Some((table, ids))
}

/// The first player puts instance `id` on item-bar slot `slot` (42/6).
fn bind(table: &mut Table, slot: i32, id: i32, now: u64) {
    table.send(0, &test_support::msg(42, 6, &[slot, id]), now);
}

/// 36/10 on the item bar from the first player, aimed at `target`.
fn press_at(table: &mut Table, slot: i32, target: u64, now: u64) -> Vec<Vec<u8>> {
    table.send(0, &encode_client_request_start_ability(2, slot, Target::Guid(target)), now)
}

fn table_health(table: &Table, index: usize) -> u32 {
    let session = &table.server.sessions[&table.keys[index]];
    let state = session.gateway.snapshot().gameplay.unwrap();
    state.encounter.unwrap().player_health
}

/// The last energy each 38/13 (`build_energy`: current, then max) in `replies` carried.
fn energy(replies: &[Vec<u8>]) -> Option<(i32, i32)> {
    let sent = replies.iter().filter(|p| p.starts_with(&[38, 0, 13, 0])).last()?;
    Some((word(sent, 4), word(sent, 8)))
}

/// Both players named, online and in one group, in the same zone and instance.
fn group_of_two(table: &mut Table) {
    use osfr_tools::social_lists::{
        CharacterIdentity, CharacterLookup, SocialLists, SocialName,
    };
    struct Lookup(Vec<CharacterIdentity>);
    impl CharacterLookup for Lookup {
        fn find_by_name(&self, name: &SocialName) -> Option<CharacterIdentity> {
            self.0.iter().find(|i| &i.name == name).cloned()
        }
    }
    let ids: Vec<_> = (0..2_u64)
        .map(|i| CharacterIdentity {
            account_id: 10_001 + i,
            character_guid: i + 1,
            name: SocialName::new(["Cee", "Dee"][i as usize], "Tester").unwrap(),
        })
        .collect();
    for (index, identity) in ids.iter().enumerate() {
        let id = table.server.sessions[&table.keys[index]].id;
        let world = &mut table.server.world;
        world.identities.insert(identity.character_guid, identity.clone());
        if world.social.profile(identity.character_guid).is_none() {
            let profile = SocialLists::new(identity.clone()).unwrap();
            world.social.register(profile).unwrap();
        }
        world.social.login(identity.character_guid).unwrap();
        world.active_sessions.insert(identity.character_guid, id);
        world.joined.insert(id);
    }
    let social = &mut table.server.world.social;
    let sent = social.invite_to_group(1, &ids[1].name, &Lookup(ids.clone()), 1_000).unwrap();
    // The 40/1 the invitee gets ends with the echoed invite id and five zero words.
    let invite = sent
        .iter()
        .find(|m| m.recipient_guid == 2 && m.bytes.starts_with(&[40, 0, 1, 0]))
        .expect("the invitee gets the 40/1");
    let at = invite.bytes.len() - 24;
    let invite_id = u32::from_le_bytes(invite.bytes[at..at + 4].try_into().unwrap());
    social.respond_group_invite(2, invite_id, true, 1_000).unwrap();
    assert!(social.group_of(1).is_some() && social.group_of(1) == social.group_of(2));
    for guid in [1, 2] {
        super::super::t020_group_routing::enter_member(&mut table.server.membership, guid, None);
    }
}

#[test]
fn c3_a_shared_health_potion_heals_a_hurt_group_member_once_per_press() {
    const SHARED: i32 = 49997;
    let Some((mut table, ids)) = c3_table("c3-group", &[(SHARED, 2)]) else {
        return;
    };
    group_of_two(&mut table);
    bind(&mut table, 0, ids[0], 1_500);
    let hurt = |table: &mut Table, now: u64| {
        let key = table.keys[1];
        let session = table.server.sessions.get_mut(&key).unwrap();
        session.gateway.receive_field_damage(200, now, true).unwrap();
        table.drain(1, now);
        table_health(table, 1)
    };
    let low = hurt(&mut table, 1_600);
    press_at(&mut table, 0, 1, 2_000);
    let member = table.drain(1, 2_000);
    assert_eq!(with_head_inner(&member, [38, 0, 1, 0]).len(), 1, "one 38/1: {member:?}");
    let healed = table_health(&table, 1);
    assert!(healed > low, "the group member is healed: {low} -> {healed}");
    // A second press after the recast heals again, once.
    let low = hurt(&mut table, 39_000);
    press_at(&mut table, 0, 1, 40_000);
    let member = table.drain(1, 40_000);
    assert_eq!(with_head_inner(&member, [38, 0, 1, 0]).len(), 1);
    assert!(table_health(&table, 1) > low);
}

/// `with_head` for the table's already unwrapped bodies.
fn with_head_inner(bodies: &[Vec<u8>], head: [u8; 4]) -> Vec<Vec<u8>> {
    bodies.iter().filter(|p| p.starts_with(&head)).cloned().collect()
}

#[test]
fn c3_a_medium_energy_potion_gives_energy_back_at_once_and_over_time() {
    const MEDIUM_ENERGY: i32 = 3023;
    let Some((mut table, ids)) = c3_table("c3-energy", &[(MEDIUM_ENERGY, 1)]) else {
        return;
    };
    bind(&mut table, 0, ids[0], 1_500);
    // The brawler kit's super (bar 1, slot 1) costs the whole bar.
    let mut cast = vec![36, 0, 10, 0];
    for word in [1_i32, 1, 0] {
        cast.extend(word.to_le_bytes());
    }
    cast.extend(WOLF.to_le_bytes());
    let spent = table.send(0, &cast, 2_000);
    let (low, max) = energy(&spent).expect("the super spends energy (38/13)");
    assert!(low < max, "{low} of {max}");
    let (instant, over_ms) = match shipped_in(&table, MEDIUM_ENERGY) {
        osfr_tools::item_bar::EffectKind::Restore {
            energy_percent,
            instant_percent,
            over_ms,
            ..
        } => ((instant_percent, energy_percent), over_ms),
        other => panic!("{other:?}"),
    };
    let drunk = press_at(&mut table, 0, 1, 2_100);
    let (now, _) = energy(&drunk).expect("the part at once is a 38/13");
    assert_eq!(now, low + instant.0 as i32 * max / 100, "the part at once");
    // The engine's own regeneration only adds, so the potion's whole amount is a floor.
    for step in 1..=over_ms / 1_000 + 1 {
        let at = 2_100 + step * 1_000;
        super::t889_shared_gateway::tick_client(&mut table, 0, at);
        table.drain(0, at);
    }
    let after = energy(&table.seen[0]).unwrap().0;
    assert!(after >= low + instant.1 as i32 * max / 100, "{low} -> {after}");
}

fn shipped_in(table: &Table, item: i32) -> osfr_tools::item_bar::EffectKind {
    let content = table.server.options.content.as_ref().unwrap();
    let effects = osfr_tools::item_bar::Effects::load(content).unwrap();
    effects.get(item).unwrap().effect
}

#[test]
fn c3_a_frost_grenade_and_a_cocoon_orb_hold_the_wolf_they_are_thrown_at() {
    const GRENADE: i32 = 55850;
    const COCOON: i32 = 48156;
    let Some((mut table, ids)) = c3_table("c3-hold", &[(GRENADE, 1), (COCOON, 1)]) else {
        return;
    };
    bind(&mut table, 0, ids[0], 1_500);
    bind(&mut table, 1, ids[1], 1_500);
    // `apply_stun`: the held enemy plays the client's com_stunned clip (35/8 animation 1401).
    let stunned = |replies: &[Vec<u8>]| {
        replies.iter().any(|p| {
            p.starts_with(&[35, 0, 8, 0]) && guid(p, 4) == WOLF && word(p, 12) == 1401
        })
    };
    let thrown = press_at(&mut table, 0, WOLF, 2_000);
    table.tick(2_050);
    assert!(stunned(&table.seen[0]), "the grenade holds the wolf: {thrown:?}");
    let count = |table: &Table, item: i32| {
        let state = table.server.sessions[&table.keys[0]].gateway.snapshot().gameplay.unwrap();
        let held = state.inventory.items.iter().filter(|i| i.definition_id == item);
        held.map(|i| i.count).sum::<u32>()
    };
    assert_eq!(count(&table, GRENADE), 0, "the grenade is spent");
    table.seen[0].clear();
    press_at(&mut table, 1, WOLF, 9_000);
    table.tick(9_050);
    assert!(stunned(&table.seen[0]), "the cocoon holds it again");
    assert_eq!(count(&table, COCOON), 0);
}

#[test]
fn c3_a_frost_grenade_with_no_enemy_in_reach_is_refused_and_kept() {
    const GRENADE: i32 = 55850;
    let Some((mut rig, _near)) = rig("c3-strike", GRENADE) else {
        return;
    };
    let owner = rig.press(PRESS);
    assert_eq!(with_head(&owner, [36, 0, 1, 0]).len(), 1, "the client's cast failure");
    assert_eq!(rig.count(GRENADE), Some(1), "not spent");
}

#[test]
fn c3_inventory_use_of_a_mana_potion_confirms_then_drinks_it_once() {
    use osfr_tools::item_bar::use_wire::{ImmediateActivation, encode_immediate_activation};
    const MANA: i32 = 176;
    let Some((mut rig, near)) = rig("c3-mana", MANA) else {
        return;
    };
    let me = owner_guid(&rig);
    let check = encode_immediate_activation(ImmediateActivation::Check, rig.instance);
    let checked = rig.exchange(&check, PRESS);
    assert_eq!(with_head(&checked, [38, 0, 30, 0]).len(), 1, "the confirm window");
    let used = encode_immediate_activation(ImmediateActivation::Use, rig.instance);
    let drunk = rig.exchange(&used, PRESS + 1);
    assert_eq!(played(&drunk), [(me, 16325)], "the blue mana cast");
    assert_eq!(played(&rig.drain(near, PRESS + 1)), [(me, 16325)]);
    assert_eq!(rig.count(MANA), None, "drunk once");
}

#[test]
fn c3_the_other_kinds_show_on_the_owner_and_the_neighbour() {
    // Grape Soda (aura), Masher Mask (reusable transform), Cupcake of Tim the Tiny (scale) and a
    // Red Pinwheel Firework (burst).
    for (tag, item, head, spent) in [
        ("c3-aura", 83, [35, 0, 41, 0], true),
        ("c3-mask", 77043, [35, 0, 14, 0], false),
        ("c3-scale", 2582, [35, 0, 13, 0], true),
        ("c3-burst", 1167, [35, 0, 41, 0], true),
    ] {
        let Some((mut rig, near)) = rig(tag, item) else {
            return;
        };
        let owner = rig.press(PRESS);
        assert_eq!(with_head(&owner, head).len(), 1, "{tag}: {owner:?}");
        assert_eq!(with_head(&rig.drain(near, PRESS), head).len(), 1, "{tag}");
        assert_eq!(rig.count(item).is_none(), spent, "{tag}");
    }
}

#[test]
fn c3_a_haunted_headstone_is_placed_and_a_click_makes_the_clicker_a_skeleton() {
    const HEADSTONE: i32 = 69820;
    let Some(mut rig) = Rig::with_files("c3-placed", HEADSTONE, &FILES) else {
        return;
    };
    rig.scene.world.cakes = super::super::cake_routing::load(&rig.options).unwrap();
    rig.drain_owner(2_500);
    let me = owner_guid(&rig);
    let placed = rig.press(PRESS);
    let stone = with_head(&placed, [35, 0, 2, 0])
        .iter()
        .map(|p| guid(p, 4))
        .find(|g| osfr_tools::placed_cakes::is_cake(*g))
        .expect("the headstone is an AddNpc");
    assert_eq!(rig.count(HEADSTONE), Some(1), "a placed prop is not spent");
    let click = osfr_tools::npc_interaction::NpcInteractRequest {
        target: stone,
        trailing_flag: false,
    }
    .encode();
    rig.scene.send(&rig.options, &click, PRESS + 1_000).unwrap();
    let clicked = rig.drain_owner(PRESS + 1_000);
    let models: Vec<_> = with_head(&clicked, [35, 0, 14, 0])
        .iter()
        .map(|p| (word(p, 4), guid(p, 8)))
        .collect();
    assert_eq!(models, [(1692, me)], "skeleton_m_01.adr");
}
