//! Packet-path characterization for S6: crafting station refusals, the movie
//! gate, gathering bonuses and their negative cases.
//!
//! The packet-path tests require the private lab content and are ignored by
//! default; `reward_count_excludes_rented_items` needs no content. The movie
//! drivers, request bodies and S6 fixture are the shared `tests/common`
//! helpers, so a later protocol change updates one copy instead of a private
//! duplicate.
//!
//! Wire prefixes are the shipped client's request shapes: 26/8 builder
//! 0x008b6bc0, 26/10 constructor 0x00a36f80, 26/13 sender 0x00938e10,
//! 36/10 writer 0x00a342e0 and 42/3 writer 0x00bdb450. Results Close is the
//! four-byte writer 0x0092f300. The chat request reader is 0x00b11600.
#[path = "common/cooking_movie.rs"]
mod cooking_movie;
#[path = "common/forging_movie.rs"]
mod forging_movie;
#[path = "common/gateway_packets.rs"]
mod gateway_packets;
#[path = "common/s6.rs"]
mod s6;
#[path = "common/smelting_movie.rs"]
mod smelting_movie;

use gateway_packets::{blocked_families, chat, login_with_version as login_request, select};
use osfr_tools::{
    account_store::AccountRole,
    gateway::{Gateway, GatewayOutcome},
    inventory::{InventoryItem, InventoryState, RentalState},
};
use s6::{TutorialGoal, chat_texts, reference};
use serde_json::Value;
use smelting_movie::{send, wrap};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Count only owned stacks: a rented item is not a permanent reward.
fn count_definition_owned(state: &InventoryState, definition_id: i32) -> u32 {
    state
        .items
        .iter()
        .filter(|item| item.definition_id == definition_id && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

const NOW: u64 = 1_000;
const UNKNOWN_ACTOR: u64 = u64::MAX - 1;

#[derive(Clone, Copy)]
struct Station {
    content: &'static str,
    profile: i32,
    recipe: u32,
    noun: &'static str,
    wrong_job: &'static str,
    pending: &'static str,
    shortage: &'static str,
    outside: &'static str,
    changed: &'static str,
    success: Option<&'static str>,
}

const FORGING: Station = Station {
    content: "rust-opening-forging",
    profile: 16,
    recipe: 2003,
    noun: "forging",
    wrong_job: "Switch to Blacksmith before forging.",
    pending: "Accept the forging objective and finish its earlier steps first.",
    shortage: "Forging requires 10 Copper Bars, 5 Turquoise, 5 Wooden Planks and 5 Thorny Fasteners.",
    outside: "forging completion is outside a Blacksmith station",
    changed: "forging ingredients changed before completion",
    success: Some(
        "Original Spunky Scrapper Hammer controls; board tuning, consumption and tint choice are reconstructed.",
    ),
};

const COOKING: Station = Station {
    content: "rust-opening-cooking-original-assets",
    profile: 45,
    recipe: 2001,
    noun: "cooking",
    wrong_job: "Switch to Chef before cooking.",
    pending: "Accept the cooking objective and finish its earlier steps first.",
    shortage: "StrongArm Stew requires 4 Bat Meat, 4 Gummyfruit, 4 Rainleaf and 4 Spice.",
    outside: "cooking completion is outside a Chef station",
    changed: "cooking ingredients changed before completion",
    success: None,
};

const SMELTING: Station = Station {
    content: "rust-opening",
    profile: 14,
    recipe: 2002,
    noun: "smelting",
    wrong_job: "Switch to Miner before smelting.",
    pending: "Accept the smelting objective and finish its earlier steps first.",
    shortage: "Smelting requires 5 Copper Ore and 3 Singing Coal. Mine the accepted nearby ore objective first.",
    outside: "smelting completion is outside a Miner station",
    changed: "smelting ingredients changed before completion",
    success: Some(
        "Original four-stage smelting controls; phase tuning and this local recipe output are reconstructed.",
    ),
};

fn gateway_reference() -> PathBuf {
    let path = std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| s6::references().join("rust-gateway-reference"));
    assert!(
        path.is_dir(),
        "S6 requires the private gateway reference at {}",
        path.display()
    );
    path
}

/// A bundled request commits or fails as one unit, so it surfaces the exact
/// handler error a single application would hide in the gateway trace.
fn bundle(children: &[Vec<u8>]) -> Vec<u8> {
    let mut message = vec![0, 25];
    for child in children {
        let child = wrap(child);
        assert!(child.len() < 255, "bundle child exceeds one-byte length");
        message.push(child.len() as u8);
        message.extend(child);
    }
    message
}

fn login(game: &mut Gateway) {
    let manifest: Value =
        serde_json::from_slice(&fs::read(gateway_reference().join("manifest.json")).unwrap())
            .unwrap();
    let version = manifest["client_version"]
        .as_str()
        .expect("manifest client_version");
    game.handle_at(&login_request(1, version), 0).unwrap();
    send(game, &13u16.to_le_bytes(), NOW);
    send(game, &10u16.to_le_bytes(), NOW);
}

fn move_to(game: &mut Gateway, position: [f32; 3], now: &mut u64) {
    *now += 800;
    let mut packet = 125u16.to_le_bytes().to_vec();
    packet.extend(1u64.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        packet.extend(value.to_le_bytes());
    }
    packet.extend([0, 0]);
    send(game, &packet, *now);
}

fn requirements(case: Station, content: &Path) -> Vec<(i32, u32)> {
    let file = match case.profile {
        16 => "forging.json",
        45 => "cooking.json",
        14 => "smelting.json",
        _ => unreachable!(),
    };
    let content: Value = serde_json::from_slice(&fs::read(content.join(file)).unwrap()).unwrap();
    content["recipe"]["consumes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row[0].as_i64().unwrap() as i32,
                row[1].as_u64().unwrap() as u32,
            )
        })
        .collect()
}

struct StationFixture {
    game: Gateway,
    actor: u64,
    position: [f32; 3],
    stage_id: u32,
    objective_index: usize,
}

fn opening_fixture(
    content_name: &str,
    goal: TutorialGoal,
    unlocked: &[i32],
    accepted: bool,
    active: i32,
    supplies: &[(i32, u32)],
) -> StationFixture {
    let s6::OpeningFixture {
        content,
        state,
        actor,
        stage_id,
        objective_index,
    } = s6::opening_fixture(content_name, goal, unlocked, accepted, active, supplies);
    let reference = gateway_reference();
    let mut character = Gateway::reference_state(&reference).unwrap();
    character.zone_id = actor.zone_id;
    character.position = actor.position;
    let mut game = Gateway::load_gameplay(&reference, character, &content, Some(state)).unwrap();
    login(&mut game);
    StationFixture {
        game,
        actor: actor.guid,
        position: actor.position,
        stage_id,
        objective_index,
    }
}

fn station_fixture(case: Station, accepted: bool, active: i32, supplies: bool) -> StationFixture {
    let content = reference(case.content);
    let required = if supplies {
        requirements(case, &content)
    } else {
        Vec::new()
    };
    opening_fixture(
        case.content,
        TutorialGoal::Craft(case.recipe),
        &[case.profile],
        accepted,
        active,
        &required,
    )
}

fn movie_active(game: &Gateway, case: Station) -> bool {
    let state = game.snapshot().gameplay.unwrap();
    match case.profile {
        16 => state
            .forging
            .is_some_and(|forging| forging.active.is_some()),
        45 => state
            .cooking
            .is_some_and(|cooking| cooking.active.is_some()),
        14 => state
            .smelting
            .is_some_and(|smelting| smelting.active.is_some()),
        _ => unreachable!(),
    }
}

fn start_station(fixture: &mut StationFixture, case: Station) -> GatewayOutcome {
    let out = send(&mut fixture.game, &select(fixture.actor, 1), NOW);
    assert!(
        movie_active(&fixture.game, case),
        "the {} station did not start its movie",
        case.noun
    );
    out
}

/// A refused request may reply with the client-facing text, but it must not
/// fan out or stage a commit. `Gateway::snapshot()` equality covers the saved
/// state; this covers the side channels the gateway would otherwise publish to
/// neighbours or the save writer.
fn assert_gateway_refusal(out: &GatewayOutcome) {
    assert!(out.nearby.is_empty(), "a refusal fanned out to neighbours");
    assert!(
        out.shared_field.is_none(),
        "a refusal staged a shared field action"
    );
    assert!(out.group_heal.is_none(), "a refusal staged a group heal");
    assert!(!out.pets_changed, "a refusal marked the pet roster changed");
    assert!(!out.began_zoning, "a refusal began zoning");
    assert!(!out.state_changed, "a refusal changed the character");
    assert!(!out.gameplay_changed, "a refusal changed gameplay state");
    assert!(!out.entered_world);
    assert_eq!(out.dev_command_denials, 0);
}

fn assert_station_refusals(case: Station) {
    for (active, accepted, supplies, expected) in [
        (1, true, true, case.wrong_job),
        (case.profile, false, true, case.pending),
        (case.profile, true, false, case.shortage),
    ] {
        let mut fixture = station_fixture(case, accepted, active, supplies);
        let before = fixture.game.snapshot();
        let out = send(&mut fixture.game, &select(fixture.actor, 1), NOW);
        assert_eq!(chat_texts(&out.replies), vec![expected.to_owned()]);
        assert_gateway_refusal(&out);
        assert_eq!(fixture.game.snapshot(), before);
    }
}

fn assert_station_success(case: Station) {
    let mut fixture = station_fixture(case, true, case.profile, true);
    let out = start_station(&mut fixture, case);
    assert_eq!(
        chat_texts(&out.replies),
        case.success
            .map(|text| vec![text.to_owned()])
            .unwrap_or_default()
    );
}

fn terminal_payload(fixture: &StationFixture, case: Station) -> Vec<u8> {
    let state = fixture.game.snapshot().gameplay.unwrap();
    let instance = match case.profile {
        16 => state.forging.unwrap().active.unwrap().instance_id,
        45 => state.cooking.unwrap().active.unwrap().instance_id,
        14 => state.smelting.unwrap().active.unwrap().instance_id,
        _ => unreachable!(),
    };
    let mut packet = match case.profile {
        16 => osfr_tools::forging::payload_packet(instance, "OnNextLevelMsg"),
        45 => osfr_tools::cooking::payload_packet(instance, "FRServer_GameWon\t900\t0"),
        14 => osfr_tools::smelting::payload_packet(instance, "FRServer_GameWon\t900\t1"),
        _ => unreachable!(),
    }
    .unwrap();
    packet[11..15].copy_from_slice(&(-1_i32).to_le_bytes());
    packet
}

fn prepare_terminal(fixture: &mut StationFixture, case: Station) {
    match case.profile {
        16 => {
            forging_movie::start(&mut fixture.game, NOW);
            forging_movie::reach_quota(&mut fixture.game, NOW);
        }
        45 => {
            cooking_movie::start(&mut fixture.game, NOW);
            cooking_movie::rounds(&mut fixture.game, NOW);
        }
        14 => {
            // ReadyForGameData publishes OnGameDataMsg, one message per
            // content phase and OnDataComplete: six replies for this content.
            let ready = smelting_movie::start(&mut fixture.game, NOW);
            assert_eq!(ready.replies.len(), 6);
            smelting_movie::rounds(&mut fixture.game, NOW);
        }
        _ => unreachable!(),
    }
}

fn assert_terminal_error(fixture: &mut StationFixture, case: Station, expected: &str) {
    let payload = terminal_payload(fixture, case);
    let error = fixture
        .game
        .handle_at(&bundle(&[payload]), NOW)
        .err()
        .expect("the terminal must be refused");
    assert_eq!(error.to_string(), expected);
}

fn assert_movie_gate(case: Station) {
    let mut fixture = station_fixture(case, true, case.profile, true);
    start_station(&mut fixture, case);
    let expected = format!(
        "Close the {} game before changing jobs, equipment or interacting.",
        case.noun
    );
    for packet in blocked_families(UNKNOWN_ACTOR) {
        let before = fixture.game.snapshot();
        let out = send(&mut fixture.game, &packet, NOW);
        assert_eq!(chat_texts(&out.replies), vec![expected.clone()]);
        assert_gateway_refusal(&out);
        assert_eq!(fixture.game.snapshot(), before);
    }

    let before = fixture.game.snapshot();
    let error = fixture
        .game
        .handle_at(&bundle(&[vec![26, 0, 42, 0, 0]]), NOW)
        .err()
        .expect("malformed leave must fail");
    assert_eq!(
        error.to_string(),
        format!("invalid {} leave request", case.noun)
    );
    assert_eq!(fixture.game.snapshot(), before);
}

fn assert_completion_errors(case: Station) {
    let mut fixture = station_fixture(case, true, case.profile, true);
    start_station(&mut fixture, case);
    prepare_terminal(&mut fixture, case);
    let mut now = NOW;

    let outside = [
        fixture.position[0] + 20.0,
        fixture.position[1],
        fixture.position[2],
    ];
    move_to(&mut fixture.game, outside, &mut now);
    let before = fixture.game.snapshot();
    assert_terminal_error(&mut fixture, case, case.outside);
    assert_eq!(fixture.game.snapshot(), before);

    move_to(&mut fixture.game, fixture.position, &mut now);
    let mut inventory = fixture.game.inventory_state().unwrap().clone();
    for (definition, _) in requirements(case, &reference(case.content)) {
        inventory
            .items
            .retain(|item| item.definition_id != definition || item.rental.is_some());
    }
    fixture.game.adopt_trade_inventory(inventory).unwrap();
    let before = fixture.game.snapshot();
    assert_terminal_error(&mut fixture, case, case.changed);
    assert_eq!(fixture.game.snapshot(), before);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn ability_request_is_the_clients_24_byte_mode_zero_form() {
    let mut fixture = station_fixture(FORGING, true, FORGING.profile, true);
    // Client writer 0x00a342e0: i32 bar, i32 slot, i32 target mode, then the
    // u64 target id when the mode is 0. A bundled request surfaces handler
    // errors, so a malformed cast shape fails here instead of hiding.
    let attack = [
        vec![36, 0, 10, 0],
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        0_i32.to_le_bytes().to_vec(),
        UNKNOWN_ACTOR.to_le_bytes().to_vec(),
    ]
    .concat();
    assert_eq!(attack.len(), 24);
    fixture
        .game
        .handle_at(&bundle(&[attack]), NOW)
        .expect("the client's 24-byte mode-0 ability request must parse");
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn forging_characterization_pins_station_gate_and_completion_errors() {
    assert_station_refusals(FORGING);
    assert_station_success(FORGING);
    assert_movie_gate(FORGING);
    assert_completion_errors(FORGING);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn cooking_characterization_pins_station_gate_and_completion_errors() {
    assert_station_refusals(COOKING);
    assert_station_success(COOKING);
    assert_movie_gate(COOKING);
    assert_completion_errors(COOKING);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn smelting_characterization_pins_station_gate_and_completion_errors() {
    assert_station_refusals(SMELTING);
    assert_station_success(SMELTING);
    assert_movie_gate(SMELTING);
    assert_completion_errors(SMELTING);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn forging_chat_gate_refuses_other_realm_actions_and_exit_clears_the_movie() {
    let mut fixture = station_fixture(FORGING, true, FORGING.profile, true);
    start_station(&mut fixture, FORGING);
    fixture.game.set_dev_tools(true);
    fixture.game.set_role(AccountRole::Admin);
    let before = fixture.game.snapshot();
    let out = send(&mut fixture.game, &chat("realm x"), NOW);
    assert_eq!(
        chat_texts(&out.replies),
        vec!["Close the forging game before other opening actions.".to_owned()]
    );
    assert_gateway_refusal(&out);
    assert_eq!(fixture.game.snapshot(), before);

    let out = send(&mut fixture.game, &chat(""), NOW);
    assert!(out.replies.is_empty());
    assert_gateway_refusal(&out);
    assert_eq!(fixture.game.snapshot(), before);

    let out = send(&mut fixture.game, &chat("realm exit"), NOW);
    assert!(out.state_changed);
    let after = fixture.game.snapshot().gameplay.unwrap();
    assert!(after.forging.is_none_or(|forging| forging.active.is_none()));
    assert_eq!(after.inventory, before.gameplay.unwrap().inventory);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn smelting_completion_advances_opening_craft_and_grants_every_bar() {
    let mut fixture = station_fixture(SMELTING, true, SMELTING.profile, true);
    start_station(&mut fixture, SMELTING);
    prepare_terminal(&mut fixture, SMELTING);
    let before = fixture.game.snapshot();
    let payload = terminal_payload(&fixture, SMELTING);
    send(&mut fixture.game, &payload, NOW);
    let after = fixture.game.snapshot();
    let mut expected = before
        .gameplay
        .as_ref()
        .unwrap()
        .opening
        .as_ref()
        .unwrap()
        .stages[&fixture.stage_id]
        .counts
        .clone();
    expected[fixture.objective_index] += 5;
    assert_eq!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&fixture.stage_id]
            .counts,
        expected
    );
    assert_eq!(
        count_definition_owned(&after.gameplay.as_ref().unwrap().inventory, 21785),
        count_definition_owned(&before.gameplay.as_ref().unwrap().inventory, 21785) + 5
    );
    assert_eq!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .smelting
            .as_ref()
            .unwrap()
            .completed,
        before
            .gameplay
            .as_ref()
            .unwrap()
            .smelting
            .as_ref()
            .unwrap()
            .completed
            + 1
    );
}

fn gather_fixture(
    resource_id: u32,
    profile: i32,
    content: &str,
    accepted: bool,
    active: i32,
) -> (Gateway, u64, u32, usize) {
    let fixture = opening_fixture(
        content,
        TutorialGoal::Gather(resource_id),
        &[profile],
        accepted,
        active,
        &[],
    );
    (
        fixture.game,
        fixture.actor,
        fixture.stage_id,
        fixture.objective_index,
    )
}

fn assert_bonus_drop(
    resource_id: u32,
    profile: i32,
    content: &'static str,
    expected: &[(i32, u32)],
) {
    let (mut game, actor, stage_id, objective_index) =
        gather_fixture(resource_id, profile, content, true, profile);
    let before = game.snapshot();
    send(&mut game, &select(actor, 1), NOW);
    let after = game.snapshot();
    for (definition, count) in expected {
        assert_eq!(
            count_definition_owned(&after.gameplay.as_ref().unwrap().inventory, *definition),
            count_definition_owned(&before.gameplay.as_ref().unwrap().inventory, *definition)
                + count,
            "definition {definition}"
        );
    }
    assert_eq!(
        after
            .gameplay
            .as_ref()
            .unwrap()
            .opening
            .as_ref()
            .unwrap()
            .stages[&stage_id]
            .counts[objective_index],
        1
    );
}

fn assert_no_bonus_drop(
    resource_id: u32,
    profile: i32,
    content: &'static str,
    accepted: bool,
    expected_reply: Option<&str>,
) {
    let (mut game, actor, stage_id, objective_index) =
        gather_fixture(resource_id, profile, content, accepted, profile);
    let before = game.snapshot();
    let out = send(&mut game, &select(actor, 1), NOW);
    let texts = chat_texts(&out.replies);
    match expected_reply {
        Some(prefix) => {
            assert_eq!(texts.len(), 1, "the refused gather replies once");
            assert!(
                texts[0].starts_with(prefix),
                "unexpected gather refusal: {:?}",
                texts[0]
            );
        }
        None => assert!(texts.is_empty(), "unexpected gather refusal: {texts:?}"),
    }
    assert_gateway_refusal(&out);
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.snapshot().gameplay.unwrap().opening.unwrap().stages[&stage_id].counts
            [objective_index],
        0
    );
}

#[test]
fn reward_count_excludes_rented_items() {
    let item = |id, count, rental| InventoryItem {
        definition_id: 21785,
        tint: 0,
        id,
        count,
        consumed_count: 0,
        stored_count: 0,
        last_cast_time: 0,
        rental,
        ability_count: 0,
        activate_enabled: false,
    };
    let rental = RentalState {
        expiration: i64::MAX,
        upsells: 0,
        bundle_id: 0,
    };
    let state = InventoryState {
        coins: 0,
        items: vec![item(1, 5, None), item(2, 3, Some(rental))],
        equipment: Vec::new(),
        next_instance_id: Some(3),
    };
    assert_eq!(count_definition_owned(&state, 21785), 5);
}

#[test]
#[ignore = "requires private forging, cooking and smelting content"]
fn opening_gather_bonus_drops_are_counted_only_with_valid_job_progress() {
    assert_bonus_drop(1003, 14, "rust-opening", &[(2739, 1), (879, 1)]);
    assert_bonus_drop(
        1001,
        45,
        "rust-opening-cooking-original-assets",
        &[(2884, 1)],
    );
    assert_bonus_drop(
        1002,
        45,
        "rust-opening-cooking-original-assets",
        &[(2875, 1)],
    );
    // Wrong job: the same accepted step must not pay an item or a count.
    assert_no_bonus_drop(
        1003,
        1,
        "rust-opening",
        true,
        Some("Switch to Miner before using "),
    );
    // Stage not accepted: the correct job alone must not pay either.
    assert_no_bonus_drop(1003, 14, "rust-opening", false, None);
}
