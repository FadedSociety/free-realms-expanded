//! T527: a Training Saddle of Gliding upgrades one owned winged ride, and the training is saved.
//!
//! Driven through the real 168 dispatch (`Gameplay::handle_in_world` -> `gameplay/rides.rs` ->
//! `mounts.rs`), never a copy of the rule. RECOVERED (client): 168/14 is two i32s, the mount id
//! then the item definition id (writer `00bff500`; `Mount.Upgrade` `00bffbc0` logs
//! "Upgrading Mount mountId = %d, upgradeItemDefinitionId = %d"); 168/15 is one i32 (reader
//! `00b05590` -> `00b05220` stores it at +0xc) which the client answers by dismounting and
//! re-sending 168/6 with that id (`00b06100` case 15, "Automount after upgrade: id %d").
//! RECONSTRUCTED (R): the server-side upgrade-item allowlist, per-instance training, the refusal
//! notices, and consuming the saddle.
use osfr_tools::{
    character_store::CharacterState,
    gameplay::{Gameplay, GameplayState, Outcome},
    harvest_nodes::{GatheringNodes, NodeActor},
    inventory::InventoryItem,
    mounts::{MountCatalog, MountRequest, MountService, decode_request},
};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::PathBuf};

const OWNER: u64 = 1;
/// Scaled Dragon (ride 12672, upgradable) and the Training Saddle of Gliding.
const SCALED_DRAGON: i32 = 76_856;
const SADDLE: i32 = 79_150;
/// The Wolf is a ride but `is_upgradable: false`.
const WOLF: i32 = 14_289;
const DRAGON_ROW: i32 = 900_001;
const SADDLE_ROW: i32 = 900_002;
const SPARE_SADDLE_ROW: i32 = 900_003;
const WOLF_ROW: i32 = 900_004;
const CHEF: i32 = 45;
const NOW: u64 = 10_000;
/// `mounts.json` `trained_glide`: GlideEnabled 1 and the trained glide values, with the Scaled
/// Dragon's own max speed and zero jump bonus.
const TRAINED_BLOCK: [f32; 9] = [12.5, 8.0, 2.0, 18.0, 0.75, 4.0, 1.0, 4.0, 0.0];

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}

/// The runtime content package whose item catalogue defines 76856 and 79150.
fn content() -> PathBuf {
    lab().join("reports/round70-native-20261016T061000Z/content")
}

fn mounts_json() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content/mounts.json")
}

fn who() -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}

fn character_at(position: [f32; 3]) -> CharacterState {
    CharacterState {
        guid: OWNER,
        zone_id: 0,
        position,
        rotation: [1.0, 0.0, 0.0],
    }
}

fn item(definition_id: i32, id: i32, count: u32) -> InventoryItem {
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

/// A game owning `items` (definition id, instance id, count), with world entry done.
fn game_with_edit(items: &[(i32, i32, u32)], edit: impl FnOnce(&mut GameplayState)) -> Gameplay {
    let content = content();
    let mut state = Gameplay::load(&content, None).unwrap().state();
    assert!(
        state
            .inventory
            .items
            .iter()
            .all(|item| item.id < DRAGON_ROW)
    );
    for (definition_id, id, count) in items {
        state
            .inventory
            .items
            .push(item(*definition_id, *id, *count));
    }
    edit(&mut state);
    let mut game = Gameplay::load(&content, Some(state)).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    game
}

fn game_with(items: &[(i32, i32, u32)]) -> Gameplay {
    game_with_edit(items, |_| {})
}

/// 168/14: the writer `00bff500` writes `this+0xc` then `this+0x10`; `Mount.Upgrade` supplies
/// the Lua mount id first and the upgrade item definition id second.
fn upgrade(row_id: i32, item_definition_id: i32) -> Vec<u8> {
    let mut bytes = vec![168, 0, 14];
    bytes.extend(row_id.to_le_bytes());
    bytes.extend(item_definition_id.to_le_bytes());
    bytes
}

fn spawn_request(row_id: i32) -> Vec<u8> {
    [vec![168, 0, 6], row_id.to_le_bytes().to_vec()].concat()
}

fn board_request(actor: u64) -> Vec<u8> {
    [
        vec![168, 0, 1],
        actor.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat()
}

fn ride(game: &mut Gameplay, row_id: i32, now: u64) -> Outcome {
    game.handle_in_world(&spawn_request(row_id), &who(), now)
        .unwrap();
    let boarded = game
        .handle_in_world(
            &board_request(MountService::ride_guid(OWNER)),
            &who(),
            now + 10,
        )
        .unwrap();
    assert!(game.is_mounted());
    boarded
}

fn notice(out: &Outcome) -> bool {
    out.replies.iter().any(|r| r.starts_with(&[15, 0, 3, 0]))
}

fn has_roster(out: &Outcome) -> bool {
    out.replies.iter().any(|r| r.starts_with(&[168, 0, 5]))
}

/// The 168/15 row id; `None` when the request was refused.
fn upgrade_reply(out: &Outcome) -> Option<i32> {
    out.replies
        .iter()
        .find(|r| r.starts_with(&[168, 0, 15]))
        .map(|r| i32::from_le_bytes(r[3..7].try_into().unwrap()))
}

/// One 168/5 record as the client reads it: (row id, is_upgradable, is_upgraded).
fn roster_rows(list: &[u8]) -> Vec<(i32, bool, bool)> {
    assert_eq!(&list[..3], &[168, 0, 5]);
    let count = i32::from_le_bytes(list[3..7].try_into().unwrap());
    let word = |at: usize| i32::from_le_bytes(list[at..at + 4].try_into().unwrap());
    let mut at = 7;
    let mut rows = Vec::new();
    for _ in 0..count {
        let id = word(at);
        let alias_len = word(at + 25) as usize;
        rows.push((
            id,
            list[at + 29 + alias_len] != 0,
            list[at + 30 + alias_len] != 0,
        ));
        at += 31 + alias_len;
    }
    assert_eq!(at, list.len(), "the roster has trailing bytes");
    rows
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stat {
    Int(i32),
    Float(f32),
}

type Update = (u64, Vec<(i32, Stat)>);

/// Reader-faithful 38/7 parse (u16 38, u16 7, u64 target, i32 count, then id/kind/value).
fn stat_update(packet: &[u8]) -> Option<Update> {
    if packet.get(..4)? != [38, 0, 7, 0] {
        return None;
    }
    let word = |at: usize| -> [u8; 4] { packet[at..at + 4].try_into().unwrap() };
    let target = u64::from_le_bytes(packet[4..12].try_into().unwrap());
    let count = i32::from_le_bytes(word(12));
    let mut stats = Vec::new();
    let mut at = 16;
    for _ in 0..count {
        let value = match i32::from_le_bytes(word(at + 4)) {
            0 => Stat::Int(i32::from_le_bytes(word(at + 8))),
            1 => Stat::Float(f32::from_le_bytes(word(at + 8))),
            kind => panic!("the client rejects stat kind {kind}"),
        };
        stats.push((i32::from_le_bytes(word(at)), value));
        at += 12;
    }
    assert_eq!(at, packet.len(), "38/7 has trailing bytes");
    Some((target, stats))
}

fn owner_updates(out: &Outcome) -> Vec<Update> {
    assert!(
        out.nearby.iter().all(|p| stat_update(p).is_none()),
        "a 38/7 reached observers"
    );
    out.replies.iter().filter_map(|p| stat_update(p)).collect()
}

/// The boarded block in send order: speed, glide 59-63, GlideEnabled (int), GlideAccel,
/// JumpHeight.
fn ride_block(values: [f32; 9]) -> Vec<(i32, Stat)> {
    [2, 59, 60, 61, 62, 63, 64, 67, 68]
        .into_iter()
        .zip(values)
        .map(|(id, value)| match id {
            64 => (id, Stat::Int(value as i32)),
            _ => (id, Stat::Float(value)),
        })
        .collect()
}

/// A refusal must leave the saved state and the inventory exactly as they were, with the roster
/// refresh and a notice (never 168/15).
fn assert_refused(game: &mut Gameplay, row_id: i32, item_definition_id: i32, label: &str) {
    let before_state = game.state();
    let before_inventory = game.inventory_state().clone();
    let out = game
        .handle_in_world(&upgrade(row_id, item_definition_id), &who(), NOW + 500)
        .unwrap();
    assert!(notice(&out), "{label}: a notice");
    assert!(has_roster(&out), "{label}: the roster refresh");
    assert!(upgrade_reply(&out).is_none(), "{label}: no 168/15");
    assert_eq!(game.state(), before_state, "{label}: state changed");
    assert_eq!(
        game.inventory_state(),
        &before_inventory,
        "{label}: inventory changed"
    );
}

#[test]
fn a_saddle_upgrades_an_owned_winged_ride() {
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);

    let out = game
        .handle_in_world(&upgrade(DRAGON_ROW, SADDLE), &who(), NOW)
        .unwrap();
    assert!(out.changed, "the training is a saved change");
    assert_eq!(
        out.replies.len(),
        3,
        "the replies are the item update, then 168/5, then 168/15"
    );
    // The saddle instance's last unit is gone: the normal 38/4 removal.
    assert_eq!(&out.replies[0][..4], &[38, 0, 4, 0]);
    assert_eq!(
        i32::from_le_bytes(out.replies[0][4..8].try_into().unwrap()),
        SADDLE_ROW
    );
    assert_eq!(roster_rows(&out.replies[1]), [(DRAGON_ROW, true, true)]);
    // 168/15 is exactly the header and one i32 (the client's reader 00b05590 reads one).
    let mut reply = vec![168, 0, 15];
    reply.extend(DRAGON_ROW.to_le_bytes());
    assert_eq!(out.replies[2], reply);
    assert_eq!(upgrade_reply(&out), Some(DRAGON_ROW));
    assert!(
        !game
            .inventory_state()
            .items
            .iter()
            .any(|item| item.definition_id == SADDLE),
        "exactly one saddle was consumed and it was the only one"
    );
    assert_eq!(
        game.state().upgraded_rides,
        BTreeSet::from([DRAGON_ROW]),
        "the row's training is saved"
    );

    // The next board sends the trained glide block (GlideEnabled 1 and the trained values).
    let boarded = ride(&mut game, DRAGON_ROW, NOW + 100);
    assert_eq!(
        owner_updates(&boarded),
        vec![(OWNER, ride_block(TRAINED_BLOCK))]
    );
}

#[test]
fn a_repeat_upgrade_is_refused_and_consumes_nothing() {
    // Canary: drop the `AlreadyUpgraded` guard in `MountService::upgrade_plan` and this test
    // fails by consuming the second saddle.
    let mut game = game_with(&[
        (SCALED_DRAGON, DRAGON_ROW, 1),
        (SADDLE, SADDLE_ROW, 1),
        (SADDLE, SPARE_SADDLE_ROW, 1),
    ]);
    let first = game
        .handle_in_world(&upgrade(DRAGON_ROW, SADDLE), &who(), NOW)
        .unwrap();
    assert_eq!(upgrade_reply(&first), Some(DRAGON_ROW));

    let before_state = game.state();
    let before_inventory = game.inventory_state().clone();
    let again = game
        .handle_in_world(&upgrade(DRAGON_ROW, SADDLE), &who(), NOW + 100)
        .unwrap();
    assert!(notice(&again) && has_roster(&again));
    assert!(upgrade_reply(&again).is_none());
    assert_eq!(game.state(), before_state, "a repeat changed the save");
    assert_eq!(
        game.inventory_state(),
        &before_inventory,
        "a repeat consumed the spare saddle"
    );
    let saddles: Vec<_> = game
        .inventory_state()
        .items
        .iter()
        .filter(|item| item.definition_id == SADDLE)
        .collect();
    assert_eq!(saddles.len(), 1, "the spare saddle is still owned");
    assert_eq!(saddles[0].id, SPARE_SADDLE_ROW);
    // The roster still marks the row trained.
    let roster = again
        .replies
        .iter()
        .find(|r| r.starts_with(&[168, 0, 5]))
        .unwrap();
    assert_eq!(roster_rows(roster), [(DRAGON_ROW, true, true)]);
}

fn mushroom_node() -> NodeActor {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    GatheringNodes::load(&content)
        .unwrap()
        .unwrap()
        .actors()
        .find(|node| node.name == "Mushrooms")
        .expect("the shipped Mushrooms node")
        .clone()
}

fn npc_interact(target: u64) -> Vec<u8> {
    let mut packet = vec![26, 0, 8, 0];
    packet.extend(target.to_le_bytes());
    packet.push(0);
    packet
}

#[test]
fn refusals_change_nothing() {
    // The Wolf is a ride but not upgradable.
    let mut game = game_with(&[(WOLF, WOLF_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);
    assert_refused(&mut game, WOLF_ROW, SADDLE, "the Wolf");

    // A row the player does not own.
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);
    assert_refused(&mut game, 999_999, SADDLE, "an unowned row");

    // No saddle at all.
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1)]);
    assert_refused(&mut game, DRAGON_ROW, SADDLE, "no saddle");

    // An item the player owns but the content does not list as an upgrade item.
    let mut game = game_with(&[
        (SCALED_DRAGON, DRAGON_ROW, 1),
        (SADDLE, SADDLE_ROW, 1),
        (WOLF, WOLF_ROW, 1),
    ]);
    assert_refused(&mut game, DRAGON_ROW, WOLF, "an item not in upgrade_items");

    // While gathering: start a real Chef harvest on the shipped mushroom node.
    let mut game = game_with_edit(
        &[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)],
        |state| {
            let profiles = state.profiles.as_mut().expect("round70 has job profiles");
            profiles.unlocked.insert(CHEF);
            profiles.active = CHEF;
        },
    );
    let node = mushroom_node();
    let at = [node.position[0] + 2.0, node.position[1], node.position[2]];
    game.handle_in_world(&npc_interact(node.guid), &character_at(at), NOW)
        .unwrap();
    assert!(
        game.state()
            .harvesting
            .and_then(|state| state.active)
            .is_some(),
        "the gather started"
    );
    assert_refused(&mut game, DRAGON_ROW, SADDLE, "while gathering");

    // Knocked out.
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);
    game.receive_field_damage(u32::MAX, NOW, false).unwrap();
    assert_refused(&mut game, DRAGON_ROW, SADDLE, "knocked out");
}

#[test]
fn malformed_upgrade_requests_are_rejected() {
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);
    let before = game.state();
    for length in [0usize, 4, 12] {
        let mut request = vec![168u8, 0, 14];
        request.extend(vec![0u8; length]);
        assert!(
            game.handle_in_world(&request, &who(), NOW).is_err(),
            "a {length}-byte 168/14 body is malformed"
        );
        assert_eq!(game.state(), before, "a {length}-byte body changed state");
    }
}

#[test]
fn the_upgrade_survives_save_and_reload() {
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1), (SADDLE, SADDLE_ROW, 1)]);
    game.handle_in_world(&upgrade(DRAGON_ROW, SADDLE), &who(), NOW)
        .unwrap();
    let saved = game.state();
    assert_eq!(saved.upgraded_rides, BTreeSet::from([DRAGON_ROW]));
    let text = serde_json::to_string(&saved).unwrap();
    assert!(
        text.contains("\"upgraded_rides\":["),
        "the training is in the save"
    );

    let content = content();
    let mut reloaded = Gameplay::load(&content, Some(saved)).unwrap();
    reloaded.initialize_world(&who(), 0).unwrap();
    assert_eq!(
        reloaded.state().upgraded_rides,
        BTreeSet::from([DRAGON_ROW]),
        "the training came back with the save"
    );
    let boarded = ride(&mut reloaded, DRAGON_ROW, NOW + 100);
    assert_eq!(
        owner_updates(&boarded),
        vec![(OWNER, ride_block(TRAINED_BLOCK))],
        "the reloaded ride still glides"
    );

    // A ride traded away loses its training on the next save (R).
    let mut without = reloaded.inventory_state().clone();
    without.items.retain(|item| item.id != DRAGON_ROW);
    reloaded.adopt_trade_inventory(without).unwrap();
    assert!(
        reloaded.state().upgraded_rides.is_empty(),
        "a traded-away ride keeps no training"
    );

    // An older save without the field still serializes without it (skip_serializing_if).
    let fresh = Gameplay::load(&content, None).unwrap().state();
    assert!(
        !serde_json::to_string(&fresh)
            .unwrap()
            .contains("upgraded_rides")
    );
}

#[test]
fn the_upgrade_request_is_a_row_and_an_item() {
    assert_eq!(
        decode_request(&upgrade(DRAGON_ROW, SADDLE)).unwrap(),
        Some(MountRequest::Upgrade {
            row_id: DRAGON_ROW,
            item_definition_id: SADDLE,
        })
    );
    // A 168/14 body is exactly two i32s; anything else is malformed, never ignored.
    for length in [0usize, 4, 5, 12] {
        let mut request = vec![168u8, 0, 14];
        request.extend(vec![0u8; length]);
        assert!(decode_request(&request).is_err(), "a {length}-byte body");
    }
}

/// The saddle's own catalogue name and description ids (round70 `item-catalogue.json` row 79150,
/// bytes 4 and 8; the locale text is "Training Saddle of Gliding" and the "Use this saddle to
/// permanently teach..." line).
const SADDLE_NAME_ID: i32 = 441_011;
const SADDLE_DESCRIPTION_ID: i32 = 441_012;

/// 42/16 (`00c109f0`, bound to `EquipScreen.CheckImmediateActivationItem`) and 42/15
/// (`00c10940`, `UseImmediateActivationItem`): the header and one i32 instance id. The client's
/// inventory `useItem` event sends 42/16 first and 42/15 only after the server's 38/30 success
/// dialog is accepted (`ScriptsBase.lua:24093-24135`).
fn check_item(instance: i32) -> Vec<u8> {
    [vec![42, 0, 16, 0], instance.to_le_bytes().to_vec()].concat()
}

fn use_item(instance: i32) -> Vec<u8> {
    [vec![42, 0, 15, 0], instance.to_le_bytes().to_vec()].concat()
}

/// Reader-faithful 36/17 `AbilityPacketExecuteClientLua` (`00a30060`): the 4-byte header, an i32
/// byte length and the function name, then three f32 that the dispatcher (`00a367ea`) pushes to
/// Lua as arguments 0, 1 and 2. `None` when any length or the tail does not match.
fn execute_client_lua(packet: &[u8]) -> Option<(String, [f32; 3])> {
    if packet.get(..4)? != [36, 0, 17, 0] {
        return None;
    }
    let length = usize::try_from(i32::from_le_bytes(packet.get(4..8)?.try_into().ok()?)).ok()?;
    let end = 8usize.checked_add(length)?;
    let name = String::from_utf8(packet.get(8..end)?.to_vec()).ok()?;
    let at = end;
    let number = |offset: usize| -> Option<f32> {
        Some(f32::from_le_bytes(
            packet.get(offset..offset + 4)?.try_into().ok()?,
        ))
    };
    let args = [number(at)?, number(at + 4)?, number(at + 8)?];
    (packet.len() == at + 12).then_some((name, args))
}

fn check_reply(out: &Outcome) -> Option<&Vec<u8>> {
    out.replies
        .iter()
        .find(|packet| packet.starts_with(&[38, 0, 30, 0]))
}

/// T528: the saddle is not a bar item, so using it answers with 36/17 and opens the client's own
/// picker (`BrowserV2:ShowUpgradableMounts`, `ScriptsBase.lua:34655`) on the item definition id.
#[test]
fn using_a_saddle_opens_the_upgrade_picker() {
    // Canary: change `mounts::UPGRADE_PICKER_LUA` to the dot form and the name assertion fails.
    let mut game = game_with(&[(SADDLE, SADDLE_ROW, 1)]);
    let before_state = game.state();
    let before_inventory = game.inventory_state().clone();

    // 42/16: the client's confirmation dialog carries the saddle's own name and description.
    let checked = game
        .handle_in_world(&check_item(SADDLE_ROW), &who(), NOW)
        .unwrap();
    let packet = check_reply(&checked).expect("the 38/30 check reply");
    assert_eq!(packet.len(), 25);
    let word = |at: usize| i32::from_le_bytes(packet[at..at + 4].try_into().unwrap());
    assert_eq!(word(4), SADDLE_ROW, "the instance the Okay button uses");
    assert_eq!(word(16), SADDLE_NAME_ID, "the saddle's name");
    assert_eq!(word(20), SADDLE_DESCRIPTION_ID, "the saddle's description");
    assert_eq!(packet[24], 1, "success: Okay sends 42/15");
    assert!(!checked.changed && checked.nearby.is_empty());
    assert_eq!(game.state(), before_state, "a check consumes nothing");

    // 42/15: exactly one 36/17 to the owner, nothing to observers.
    let used = game
        .handle_in_world(&use_item(SADDLE_ROW), &who(), NOW + 1)
        .unwrap();
    assert_eq!(used.replies.len(), 1, "one packet for the owner");
    assert!(used.nearby.is_empty(), "the picker goes only to the owner");
    let (name, args) = execute_client_lua(&used.replies[0]).expect("a well-formed 36/17");
    assert_eq!(name, "BrowserV2:ShowUpgradableMounts");
    assert_eq!(args, [79_150.0, 0.0, 0.0], "the item definition id first");
    assert!(!used.changed);
    assert_eq!(
        game.state(),
        before_state,
        "using the saddle consumes nothing"
    );
    assert_eq!(game.inventory_state(), &before_inventory);

    // The picker never pends: a repeated use opens it again and still spends nothing.
    let again = game
        .handle_in_world(&use_item(SADDLE_ROW), &who(), NOW + 2)
        .unwrap();
    assert_eq!(
        execute_client_lua(&again.replies[0]).map(|(name, _)| name),
        Some("BrowserV2:ShowUpgradableMounts".to_string())
    );
    assert_eq!(game.state(), before_state);
}

/// T528: an instance the player does not hold, a knocked-out player and a busy one are all
/// refused with one system-chat line and no packet; nothing changes.
#[test]
fn a_saddle_use_is_refused_when_not_owned_or_busy() {
    // No such instance: the saddle row is not in the bag.
    let mut game = game_with(&[(SCALED_DRAGON, DRAGON_ROW, 1)]);
    let before_state = game.state();
    let before_inventory = game.inventory_state().clone();
    for request in [check_item(SADDLE_ROW), use_item(SADDLE_ROW)] {
        let out = game.handle_in_world(&request, &who(), NOW).unwrap();
        assert!(notice(&out), "a notice");
        assert_eq!(out.replies.len(), 1, "nothing but the notice");
        assert!(check_reply(&out).is_none(), "no dialog");
        assert!(!out.changed);
        assert_eq!(game.state(), before_state, "state changed");
        assert_eq!(
            game.inventory_state(),
            &before_inventory,
            "inventory changed"
        );
    }

    // Knocked out. A knockout runs no movie, so this request reaches the saddle's own
    // `ride_restrictions()` gate (`knocked_out` is part of `RideRestrictions::allows`).
    let mut game = game_with(&[(SADDLE, SADDLE_ROW, 1)]);
    game.receive_field_damage(u32::MAX, NOW, false).unwrap();
    let before_state = game.state();
    let out = game
        .handle_in_world(&use_item(SADDLE_ROW), &who(), NOW + 10)
        .unwrap();
    assert!(notice(&out), "a knocked-out refusal");
    assert_eq!(out.replies.len(), 1, "nothing but the notice");
    assert_eq!(
        game.state(),
        before_state,
        "a knockout refusal changed the save"
    );

    // While gathering: start a real Chef harvest on the shipped mushroom node. The harvesting
    // movie owns 42/* while it runs, so this refusal is the movie's (also notice-only, no packet).
    let mut game = game_with_edit(&[(SADDLE, SADDLE_ROW, 1)], |state| {
        let profiles = state.profiles.as_mut().expect("round70 has job profiles");
        profiles.unlocked.insert(CHEF);
        profiles.active = CHEF;
    });
    let node = mushroom_node();
    let at = [node.position[0] + 2.0, node.position[1], node.position[2]];
    game.handle_in_world(&npc_interact(node.guid), &character_at(at), NOW)
        .unwrap();
    assert!(
        game.state()
            .harvesting
            .and_then(|state| state.active)
            .is_some(),
        "the gather started"
    );
    let before_state = game.state();
    let before_inventory = game.inventory_state().clone();
    let out = game
        .handle_in_world(&use_item(SADDLE_ROW), &who(), NOW + 10)
        .unwrap();
    assert!(notice(&out), "a busy refusal");
    assert_eq!(out.replies.len(), 1, "nothing but the notice");
    assert_eq!(
        game.state(),
        before_state,
        "a busy refusal changed the save"
    );
    assert_eq!(game.inventory_state(), &before_inventory);
}

#[test]
fn upgrade_item_data_is_bounded() {
    let bundled: Value = serde_json::from_slice(&fs::read(mounts_json()).unwrap()).unwrap();
    let load = |edit: &dyn Fn(&mut Value)| {
        let mut document = bundled.clone();
        edit(&mut document);
        let dir = std::env::temp_dir().join(format!(
            "osfr-t527-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mounts.json");
        fs::write(&path, serde_json::to_vec(&document).unwrap()).unwrap();
        let loaded = MountCatalog::load(&path);
        fs::remove_dir_all(&dir).unwrap();
        loaded
    };
    let catalog = load(&|_| {}).unwrap();
    assert!(catalog.is_upgrade_item(SADDLE));
    assert!(!catalog.is_upgrade_item(WOLF));
    // An empty list is a valid "no upgrades in this content" and accepts nothing.
    let none = load(&|d| d["upgrade_items"] = serde_json::json!([])).unwrap();
    assert!(!none.is_upgrade_item(SADDLE));

    let refused: Vec<(&str, Box<dyn Fn(&mut Value)>)> = vec![
        (
            "non-positive",
            Box::new(|d| d["upgrade_items"][0]["item_definition_id"] = (-1).into()),
        ),
        (
            "unsourced",
            Box::new(|d| d["upgrade_items"][0]["source"] = serde_json::json!([])),
        ),
        (
            "duplicate",
            Box::new(|d| {
                let row = d["upgrade_items"][0].clone();
                d["upgrade_items"].as_array_mut().unwrap().push(row);
            }),
        ),
        (
            "unbounded",
            Box::new(|d| {
                d["upgrade_items"] = (1..=17)
                    .map(|id| {
                        serde_json::json!({
                            "item_definition_id": id,
                            "source": [{"kind": "reconstructed", "evidence": "synthetic (R)"}]
                        })
                    })
                    .collect::<Vec<_>>()
                    .into();
            }),
        ),
        (
            "missing",
            Box::new(|d| {
                d.as_object_mut().unwrap().remove("upgrade_items");
            }),
        ),
    ];
    for (name, edit) in refused {
        assert!(load(&*edit).is_err(), "{name} loaded");
    }
}
