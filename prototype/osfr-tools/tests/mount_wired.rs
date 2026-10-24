//! W162 patch proof: the `gameplay.rs` / `gateway.rs` wiring of the mount service. Drop this file
//! in as `tests/mount_wired.rs` AFTER `apply_shared_patch.py` is applied; it does not compile
//! before (`Gameplay::is_mounted`, `settle_mounts`, `Outcome::nearby`, `Gateway::settle_rides`).
//!
//! Gameplay level: a request for an owned ride answers with the roster, the ride actor and the
//! boarding reply, the actor and boarding packets are also `nearby` (the gateway's interest
//! fanout carries them to a second player), an unowned ride is refused with a notice, and each
//! server-side trigger (knockout, a chasing enemy, an instance battle, zoning) ends the ride.
//! Gateway level: `emote_context().mounted` follows the mount state, which is what refuses an
//! emote on the quick-chat route.
use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    chat,
    gameplay::{Gameplay, Outcome},
    gateway::{Gateway, GatewayOutcome},
    harvest_nodes::{GatheringNodes, NodeActor},
    harvesting::CHEF_PROFILE_ID,
    inventory::InventoryItem,
    mounts::MountService,
    npc_interaction::NpcInteractRequest,
};
use std::path::PathBuf;

/// T482: 168/6 names a roster row, an inventory instance: the first ride item `game_with` grants
/// (the gateway test grants 900_001 itself).
const ROW: i32 = 900_001;
const RIDE_ITEM: i32 = 14289; // its item definition, present in the captured catalogue
const NOW: u64 = 10_000;

fn lab() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab")
}
fn reference() -> PathBuf {
    lab().join("references/rust-opening-cooking-original-assets")
}
fn who() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-440.0, 0.8, 1292.0],
        rotation: [1.0, 0.0, 0.0],
    }
}
fn ride_guid() -> u64 {
    MountService::ride_guid(1)
}

fn spawn() -> Vec<u8> {
    [vec![168, 0, 6], ROW.to_le_bytes().to_vec()].concat()
}
fn board() -> Vec<u8> {
    [
        vec![168, 0, 1],
        ride_guid().to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat()
}
const PUT_AWAY: [u8; 3] = [168, 0, 3];

/// A game that owns `items` (as stacks of one); returns it with world entry done.
fn game_with(items: &[i32]) -> Gameplay {
    let mut state = Gameplay::load(&reference(), None).unwrap().state();
    let mut id = ROW - 1;
    assert!(state.inventory.items.iter().all(|i| i.id < ROW));
    for definition_id in items {
        id += 1;
        state.inventory.items.push(InventoryItem {
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
        });
    }
    let mut game = Gameplay::load(&reference(), Some(state)).unwrap();
    game.initialize_world(&who(), 0).unwrap();
    game
}

fn has(out: &Outcome, family: u16, sub: u8) -> bool {
    out.replies
        .iter()
        .any(|r| r.len() > 2 && r[..2] == family.to_le_bytes() && r[2] == sub)
}
fn removes_ride(out: &Outcome) -> bool {
    out.replies
        .iter()
        .any(|r| r.starts_with(&[35, 0, 3, 0, 1, 0]) && r[6..14] == ride_guid().to_le_bytes())
}
fn notice(out: &Outcome) -> bool {
    out.replies.iter().any(|r| r.starts_with(&[15, 0, 3, 0]))
}
fn add_npc(packet: &[u8]) -> bool {
    packet.starts_with(&[35, 0, 2, 0]) && packet[4..12] == ride_guid().to_le_bytes()
}

fn mounted_game() -> Gameplay {
    let mut game = game_with(&[RIDE_ITEM]);
    game.handle_in_world(&spawn(), &who(), NOW).unwrap();
    game.handle_in_world(&board(), &who(), NOW + 10).unwrap();
    assert!(game.is_mounted());
    game
}

#[test]
fn an_owned_ride_is_summoned_boarded_and_put_away_through_gameplay() {
    let mut game = game_with(&[RIDE_ITEM]);
    assert!(!game.is_mounted());

    let summoned = game.handle_in_world(&spawn(), &who(), NOW).unwrap();
    assert!(has(&summoned, 168, 5), "roster with the actor guid");
    assert!(summoned.replies.iter().any(|r| add_npc(r)), "ride actor");
    assert!(
        summoned.nearby.iter().any(|r| add_npc(r)),
        "also for neighbours"
    );
    assert!(!summoned.nearby.iter().any(|r| r.starts_with(&[168, 0, 5])));
    assert!(!summoned.changed, "ride state is not saved");
    assert!(!game.is_mounted());

    let boarded = game.handle_in_world(&board(), &who(), NOW + 10).unwrap();
    assert!(has(&boarded, 168, 2));
    assert!(boarded.nearby.iter().any(|r| r.starts_with(&[168, 0, 2])));
    assert!(game.is_mounted());
    assert!(game.ride_view(1).is_some());

    let away = game.handle_in_world(&PUT_AWAY, &who(), NOW + 20).unwrap();
    assert!(has(&away, 168, 4) && removes_ride(&away));
    assert!(away.nearby.iter().any(|r| r.starts_with(&[168, 0, 4])));
    assert!(!game.is_mounted());
    assert!(game.ride_view(1).is_none());
    // Putting nothing away is silent.
    let none = game.handle_in_world(&PUT_AWAY, &who(), NOW + 30).unwrap();
    assert!(none.replies.is_empty() && none.nearby.is_empty());
}

#[test]
fn a_ride_the_player_does_not_own_is_refused_with_a_notice() {
    let mut game = game_with(&[]);
    for request in [
        spawn(),
        [vec![168, 0, 8], RIDE_ITEM.to_le_bytes().to_vec(), vec![1]].concat(),
    ] {
        let out = game.handle_in_world(&request, &who(), NOW).unwrap();
        assert!(notice(&out));
        assert!(out.nearby.is_empty());
        assert!(!out.replies.iter().any(|r| add_npc(r)));
    }
    assert!(!game.is_mounted() && game.ride_view(1).is_none());
    // Malformed bodies are errors, not panics.
    assert!(game.handle_in_world(&[168, 0, 6, 1], &who(), NOW).is_err());
}

#[test]
fn a_knockout_a_chasing_enemy_an_instance_battle_and_zoning_each_end_the_ride() {
    // Nothing to end while on foot.
    let mut foot = game_with(&[RIDE_ITEM]);
    assert!(foot.settle_mounts(&who(), true).replies.is_empty());

    // Knockout.
    let mut game = mounted_game();
    game.receive_field_damage(u32::MAX, NOW + 100, false)
        .unwrap();
    assert_eq!(game.state().encounter.unwrap().player_health, 0);
    let out = game.settle_mounts(&who(), false);
    assert!(has(&out, 168, 4) && removes_ride(&out), "knockout");
    assert!(!out.nearby.is_empty());
    assert!(!game.is_mounted());
    // A knocked-out player cannot summon until revived.
    let refused = game.handle_in_world(&spawn(), &who(), NOW + 200).unwrap();
    assert!(notice(&refused) && refused.nearby.is_empty());

    // A field enemy chasing the player (combat start in the open field).
    let mut game = mounted_game();
    game.receive_field_damage(0, NOW + 100, true).unwrap();
    let out = game.settle_mounts(&who(), false);
    assert!(has(&out, 168, 4) && removes_ride(&out), "combat start");
    // Cannot be summoned while the chase lasts; allowed once it stops.
    assert!(notice(
        &game.handle_in_world(&spawn(), &who(), NOW + 5_000).unwrap()
    ));
    game.receive_field_damage(0, NOW + 6_000, false).unwrap();
    let again = game.handle_in_world(&spawn(), &who(), NOW + 7_000).unwrap();
    assert!(again.replies.iter().any(|r| add_npc(r)));

    // Entering an instance battle.
    let mut game = mounted_game();
    game.set_dev_tools(true);
    game.set_role(AccountRole::Admin);
    game.handle_in_world(&chat::local_request("realm battle 411"), &who(), NOW + 100)
        .unwrap();
    let out = game.settle_mounts(&who(), false);
    assert!(has(&out, 168, 4) && removes_ride(&out), "battle");
    assert!(!game.is_mounted());

    // Zoning.
    let mut game = mounted_game();
    let out = game.settle_mounts(&who(), true);
    assert!(has(&out, 168, 4) && removes_ride(&out), "zoning");
    assert!(!game.is_mounted());

    // A summoned-only ride is removed too, and a new network session starts without one.
    let mut game = game_with(&[RIDE_ITEM]);
    game.handle_in_world(&spawn(), &who(), NOW).unwrap();
    assert!(game.ride_view(1).is_some());
    game.reset_session();
    assert!(game.ride_view(1).is_none());
}

// ------------------------------------------------------------------ gateway level

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut bytes = vec![5, 0, 1];
    bytes.extend((inner.len() as u32).to_le_bytes());
    bytes.extend(inner);
    bytes
}
fn login() -> Vec<u8> {
    let mut bytes = 1u16.to_le_bytes().to_vec();
    for (i, text) in ["w162-synthetic-ticket", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        bytes.extend((text.len() as u32).to_le_bytes());
        bytes.extend(text.as_bytes());
        if i == 0 {
            bytes.extend(1u64.to_le_bytes());
        }
    }
    bytes
}

/// The emote route reads `Gateway::emote_context`; it must follow the ride state. The emote
/// itself is refused by `authorize_emote` (covered in `tests/mount_routing.rs`).
#[test]
fn the_gateway_reports_mounted_to_the_emote_rules_and_settles_rides() {
    let content = lab().join("reports/round59-native-20261015T124443Z/content");
    let reference = lab().join("references/rust-gateway-reference");
    assert!(
        content.join("harvesting.json").is_file(),
        "private content fixture"
    );
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.position = [0.0; 3];
    let mut state = Gameplay::load(&content, None).unwrap().state();
    state.inventory.items.push(InventoryItem {
        definition_id: RIDE_ITEM,
        tint: 0,
        id: 900_001,
        count: 1,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: i64::from(i32::MAX),
        rental: None,
        ability_count: 0,
        activate_enabled: false,
    });
    let mut gateway = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
    let guid = gateway.state().guid;
    let actor = MountService::ride_guid(guid);
    assert!(!gateway.emote_context(NOW).mounted);

    let summoned = gateway.handle_at(&wrap(&spawn()), NOW + 1).unwrap();
    assert!(
        !summoned.nearby.is_empty(),
        "the ride actor is for neighbours too"
    );
    assert!(
        summoned
            .nearby
            .iter()
            .any(|r| r[..4] == [35, 0, 2, 0] && r[4..12] == actor.to_le_bytes())
    );
    assert!(
        !gateway.emote_context(NOW + 2).mounted,
        "summoned, not yet on it"
    );

    let request = [
        vec![168, 0, 1],
        actor.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat();
    gateway.handle_at(&wrap(&request), NOW + 10).unwrap();
    assert!(gateway.emote_context(NOW + 11).mounted);
    assert!(gateway.ride_view().is_some());

    // Knocked out: the gateway-level settle removes the ride and clears the signal.
    gateway
        .receive_field_damage(u32::MAX, NOW + 20, false)
        .unwrap();
    let ended = gateway.settle_rides(false);
    assert!(ended.replies.iter().any(|r| r[7..10] == [168, 0, 4]));
    assert!(
        ended
            .nearby
            .iter()
            .any(|r| r.starts_with(&[35, 0, 3, 0, 1, 0]))
    );
    assert!(!gateway.emote_context(NOW + 21).mounted);
    assert!(gateway.ride_view().is_none());
}

// ------------------------------------------------- activities end the ride (T481)

/// Client `SendDismountRequest` 0x00b06cc0 has four callers (ability, item use, movement state)
/// and none is the 26/8 interaction path (`PlayClient::RequestInteraction` 0x0092efe0), so the
/// server ends the ride when an activity starts: the `ride` gate in `gameplay/occupation.rs`.
const MINER_PROFILE: i32 = 14;
const BLACKSMITH_PROFILE: i32 = 16;
const ANVIL: [f32; 3] = [354.3665, 48.24031, 316.3203];
/// Blacksmith recipe 11 at the world anvil and its inputs (as in `crafting_stations_route.rs`).
const ANVIL_RECIPE: i32 = 11;
const ANVIL_INPUTS: [(i32, u32); 4] = [(21785, 10), (2752, 5), (55, 5), (56, 5)];

fn node(name: &str) -> NodeActor {
    let content = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let nodes = GatheringNodes::load(&content).unwrap().unwrap();
    nodes.actors().find(|a| a.name == name).unwrap().clone()
}

/// An ordinary player standing at `position` with `job` active, owning the ride and `items`,
/// already summoned and on it.
fn rider_at(position: [f32; 3], job: i32, items: &[(i32, u32)]) -> Gateway {
    let content = lab().join("reports/round59-native-20261015T124443Z/content");
    let reference = lab().join("references/rust-gateway-reference");
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = 0;
    character.position = position;
    let mut state = Gameplay::load(&content, None).unwrap().state();
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(job);
    profiles.active = job;
    // The ride item is granted first, so it is row `ROW`.
    let mut id = ROW - 1;
    assert!(state.inventory.items.iter().all(|i| i.id < ROW));
    for (definition_id, count) in [(RIDE_ITEM, 1)].iter().chain(items) {
        id += 1;
        state.inventory.items.push(InventoryItem {
            definition_id: *definition_id,
            tint: 0,
            id,
            count: *count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: true,
        });
    }
    let mut gateway = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    gateway.set_dev_tools(false);
    gateway.set_role(AccountRole::Player);
    gateway.handle_at(&login(), NOW).unwrap();
    gateway.handle_at(&wrap(&[13, 0]), NOW).unwrap();
    gateway.handle_at(&wrap(&[10, 0]), NOW).unwrap();
    let actor = MountService::ride_guid(gateway.state().guid);
    let board = [
        vec![168, 0, 1],
        actor.to_le_bytes().to_vec(),
        vec![1, 0, 0, 0, 0, 0],
    ]
    .concat();
    gateway.handle_at(&wrap(&spawn()), NOW + 1).unwrap();
    gateway.handle_at(&wrap(&board), NOW + 10).unwrap();
    assert!(gateway.emote_context(NOW + 11).mounted && gateway.ride_view().is_some());
    gateway
}

fn click(gateway: &mut Gateway, guid: u64) -> GatewayOutcome {
    let request = NpcInteractRequest {
        target: guid,
        trailing_flag: false,
    };
    gateway
        .handle_at(&wrap(&request.encode()), NOW + 100)
        .unwrap()
}

fn book_start(gateway: &mut Gateway) -> GatewayOutcome {
    let packet = [vec![26, 0, 24, 0], ANVIL_RECIPE.to_le_bytes().to_vec()].concat();
    gateway.handle_at(&wrap(&packet), NOW + 100).unwrap()
}

/// The server's wire frames wrap the packet in a 7-byte envelope.
fn gateway_has(out: &GatewayOutcome, family: u16, sub: u8) -> bool {
    out.replies
        .iter()
        .any(|r| r.len() > 9 && r[7..9] == family.to_le_bytes() && r[9] == sub)
}

fn mounted(gateway: &Gateway) -> bool {
    gateway.emote_context(NOW + 500).mounted
}

fn gameplay_json(gateway: &Gateway) -> serde_json::Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn assert_ride_ended(gateway: &mut Gateway) {
    let ended = gateway.settle_rides(false);
    assert!(gateway_has(&ended, 168, 4), "dismount to the rider");
    assert!(
        ended
            .nearby
            .iter()
            .any(|r| r.starts_with(&[35, 0, 3, 0, 1, 0])),
        "ride actor removed for neighbours"
    );
    assert!(!mounted(gateway) && gateway.ride_view().is_none());
    // Idempotent: later messages during the activity send nothing more.
    let again = gateway.settle_rides(false);
    assert!(again.replies.is_empty() && again.nearby.is_empty());
}

#[test]
fn starting_a_gather_ends_the_ride() {
    let node = node("Mushrooms");
    let at = [node.position[0] + 2.0, node.position[1], node.position[2]];
    let mut gateway = rider_at(at, CHEF_PROFILE_ID, &[]);
    click(&mut gateway, node.guid);
    // The gather starts. (R) The gateway's per-message settle runs after the handler, so the
    // gather's own replies come first and 168/4 follows them (the bin appends `settle_rides`).
    assert!(
        gameplay_json(&gateway)["harvesting"]["active"]["instance_id"].is_i64(),
        "the gather started"
    );
    assert_ride_ended(&mut gateway);
}

#[test]
fn a_refused_gather_keeps_the_ride() {
    let node = node("Mushrooms");
    let near = [node.position[0] + 2.0, node.position[1], node.position[2]];
    let far = [node.position[0] + 40.0, node.position[1], node.position[2]];
    // Out of reach, and the wrong active job.
    for (at, job) in [(far, CHEF_PROFILE_ID), (near, MINER_PROFILE)] {
        let mut gateway = rider_at(at, job, &[]);
        click(&mut gateway, node.guid);
        assert!(gameplay_json(&gateway)["harvesting"]["active"].is_null());
        let settled = gateway.settle_rides(false);
        assert!(settled.replies.is_empty() && settled.nearby.is_empty());
        assert!(mounted(&gateway) && gateway.ride_view().is_some());
    }
}

#[test]
fn no_ride_can_be_summoned_while_gathering() {
    let node = node("Mushrooms");
    let at = [node.position[0] + 2.0, node.position[1], node.position[2]];
    let mut gateway = rider_at(at, CHEF_PROFILE_ID, &[]);
    click(&mut gateway, node.guid);
    assert_ride_ended(&mut gateway);

    let before = gateway.snapshot();
    let refused = gateway.handle_at(&wrap(&spawn()), NOW + 200).unwrap();
    assert!(
        refused.replies.iter().any(|r| r[7..11] == [15, 0, 3, 0]),
        "the notice"
    );
    assert!(gateway_has(&refused, 168, 5), "the roster refresh");
    assert!(refused.nearby.is_empty() && !gateway_has(&refused, 35, 2));
    assert!(gateway.ride_view().is_none() && !mounted(&gateway));
    assert_eq!(gateway.snapshot(), before, "a refusal changes nothing");
    assert!(gameplay_json(&gateway)["harvesting"]["active"]["instance_id"].is_i64());
}

#[test]
fn starting_a_crafting_station_ends_the_ride() {
    let mut gateway = rider_at(ANVIL, BLACKSMITH_PROFILE, &ANVIL_INPUTS);
    book_start(&mut gateway);
    assert!(
        gameplay_json(&gateway)["forging"]["active"]["instance_id"].is_i64(),
        "the forging run started"
    );
    assert_ride_ended(&mut gateway);
}
