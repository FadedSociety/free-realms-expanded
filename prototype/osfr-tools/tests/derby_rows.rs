//! T791: the derby hosts more than one MiniGameData row.
//!
//! Client facts: `Resources/MiniGameData.txt` row 39 is `39^11^37938^37938^9869^1^...` (TYPE 11,
//! NAME_ID 37938 "Wildwood Demo Derby") and row 604 is
//! `604^11^393294^394459^9870^1^...` (TYPE 11, NAME_ID 393294, description 394459). The 39/16
//! native-game offer already names the game id at bytes 11..15 (`DerbyService::offer_packet`), so a
//! row needs no new wire field.
//!
//! The tests drive the production path: `DerbyGateway::load` for every content bound,
//! `Gateway` 26/8 -> 26/10 for ordinary entry, the 39/16 offer -> 39/5 Go -> 34/6 register ->
//! phase/countdown acknowledgements -> 34/16 contacts -> results credit, plus the real quest
//! catalogue's 49/1/49/2 offer/accept. No test calls a copy of the feature.

#[path = "support/json.rs"]
mod json;
#[path = "support/npc_quest_chain.rs"]
mod npc_quest_chain;
#[path = "support/scratch.rs"]
mod scratch;
#[path = "support/soccer_play.rs"]
mod soccer_play;

#[path = "support/derby_play.rs"]
mod derby_play;

use derby_play::{countdown_for, hit_for, phase_for, register_for, start_request};
use npc_quest_chain::{
    Fixture, accept, content_dir, counts, gateway_send, menu_option_bytes, menu_option_opt,
    npc_interact, npc_location, npc_select, offer,
};
use osfr_tools::{
    character_store::CharacterState,
    derby::{DerbyMessage, DerbyPhase, ParticipantKey, VehiclePose, decode_inner, encode_inner},
    derby_gateway::{
        DEMO_DERBY_DRIVER_PROFILE, DerbyGateway, DerbyGatewayContent, DerbyGatewayState,
        MAX_COMPLETION_REWARD,
    },
    gameplay::{Gameplay, GameplayState},
    gateway::Gateway,
};
use scratch::Scratch;
use serde_json::{Value, json};
use soccer_play::offer_instance;
use std::{fs, path::Path};

/// The shipped arena drivers (`derby_gates.json`).
const MAD_JACK: u64 = 9_000_000_391;
const CRASH: u64 = 9_000_000_289;
const BURT: u64 = 9_500_000_731;
const RIP: u64 = 9_000_000_183;
/// A real spawned Wilds NPC that is not a derby gate (Dalton Narrowbrow).
const NOT_A_GATE: u64 = 9_000_000_295;
const DERBY_LABEL: u32 = 3_053;
const LICENSE_TO_DRIVE: u32 = 3_060_005;
const DEFAULT_GAME: i32 = 39;
const BREAK_ER_IN: i32 = 604;
const DERBY_FILES: [&str; 4] = [
    "derby_entry.json",
    "derby_gates.json",
    "derby_rows.json",
    "derby_minigames.json",
];
const NOW: u64 = 10_000;
const NPC_DRIVERS: usize = 7;
const KNOCKOUTS_TO_WIN: usize = 4;
const RESULTS_HOLD_MS: u64 = 10_000;

fn test_source() -> Value {
    json!([{
        "kind": "client",
        "evidence": "T791 test fixture: Resources/MiniGameData.txt row 604 is TYPE 11 with NAME_ID 393294."
    }])
}

fn one_row(game_id: i32, npcs: &[u64]) -> Value {
    json!({
        "game_id": game_id,
        "npcs": npcs,
        "rules": null,
        "pickups": null,
        "completion_reward_coins": null,
        "completion_reward_xp": null,
        "source": test_source(),
    })
}

fn rows_file(rows: Vec<Value>) -> Value {
    json!({"schema_version": 1, "rows": rows})
}

/// A scratch content tree holding copies of the shipped derby files the test names.
struct Package(Scratch);

impl Package {
    fn new(files: &[&str]) -> Self {
        let scratch = Scratch::new("derby-rows");
        for file in files {
            fs::copy(content_dir().join(file), scratch.path().join(file)).unwrap();
        }
        Self(scratch)
    }

    fn shipped() -> Self {
        Self::new(&DERBY_FILES)
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn write_rows(&self, rows: Vec<Value>) {
        json::write_pretty(&self.path().join("derby_rows.json"), &rows_file(rows));
    }
}

fn default_package(package: &Package) -> DerbyGatewayContent {
    serde_json::from_slice(&fs::read(package.path().join("derby_entry.json")).unwrap()).unwrap()
}

/// A quest-chapter fixture that also carries the shipped derby package.
fn derby_fixture(guid: u64) -> Fixture {
    let fixture = Fixture::for_character(guid);
    for file in DERBY_FILES {
        fixture.copy_content(file);
    }
    fixture
}

/// Rewrite 3060005 into the test's row quest: no job or prerequisite gate and one
/// `finish_minigame` objective per `minigame_ids` entry. The giver stays Crash Davis, a shipped
/// derby gate that already lists this quest, so the offer still travels the real NPC menu.
fn patch_derby_quest(fixture: &Fixture, minigame_ids: &[i32], count: u32) {
    let objectives: Vec<Value> = minigame_ids
        .iter()
        .enumerate()
        .map(|(index, minigame_id)| {
            json!({
                "kind": "finish_minigame",
                "id": index as u32 + 1,
                "minigame_id": minigame_id,
                "count": count,
                "text_string_id": 394_068,
                "source": test_source(),
            })
        })
        .collect();
    fixture.patch_quest("demo_derby_driver.json", LICENSE_TO_DRIVE, |record| {
        let object = record.as_object_mut().unwrap();
        object.remove("job");
        object.insert("prerequisite_quest_ids".into(), json!([]));
        object.insert("objectives".into(), Value::Array(objectives));
    });
}

/// Accept 3060005 through the real 49/1 offer and 49/2 accept path.
fn accept_license(game: &mut Gameplay, character: &CharacterState) {
    let mut now = NOW;
    offer(game, character, CRASH, LICENSE_TO_DRIVE, now);
    now += 10;
    accept(game, character, LICENSE_TO_DRIVE, now);
}

fn offer_game_id(replies: &[Vec<u8>]) -> i32 {
    replies
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .map(|packet| i32::from_le_bytes(packet[11..15].try_into().unwrap()))
        .expect("the derby offer")
}

fn pose(key: ParticipantKey) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
        .map(f32::to_bits),
        vector_a_bits: [0; 4],
        vector_b_bits: [0; 4],
        controls_bits: [0; 3],
        flags: [0; 4],
        sequence: 1,
    }))
    .unwrap()
}

/// Take the driver's real derby menu option. Returns `(instance, game_id)` from the 39/16 offer.
fn enter_derby(gateway: &mut Gateway, driver: u64, base: u64) -> (i32, i32) {
    let menu = gateway_send(gateway, &npc_interact(driver), base);
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("the driver opens its 26/9 menu");
    let option = menu_option_bytes(packet, DERBY_LABEL);
    let entered = gateway_send(gateway, &npc_select(driver, option), base + 1);
    (offer_instance(&entered), offer_game_id(&entered))
}

/// Send the native 39/5 Go, register, and drive the client-owned readiness gates and countdown to
/// Running. Returns the local key and the seven server-created NPC keys.
fn start_and_ready(
    gateway: &mut Gateway,
    instance: i32,
    expected_game: i32,
    base: u64,
) -> (ParticipantKey, Vec<ParticipantKey>) {
    let key = ParticipantKey(gateway.state().guid);
    let started = gateway_send(gateway, &start_request(instance), base + 300);
    let ack = started
        .iter()
        .find(|packet| packet.starts_with(&[39, 0, 17]))
        .expect("the native 39/5 Go is acknowledged");
    assert_eq!(
        i32::from_le_bytes(ack[11..15].try_into().unwrap()),
        expected_game,
        "the 39/17 acknowledgement names the entered row"
    );
    assert!(
        started
            .iter()
            .any(|packet| packet.starts_with(&[35, 0, 20])),
        "the native Go sends the type-11 active carrier"
    );
    let setup = gateway_send(gateway, &register_for(key), base + 400);
    let npcs: Vec<_> = setup
        .iter()
        .filter(|packet| packet.starts_with(&[34, 0, 10]))
        .filter_map(|packet| match decode_inner(packet) {
            Ok(DerbyMessage::CreateVehicle(vehicle)) if vehicle.key != key => Some(vehicle.key),
            _ => None,
        })
        .collect();
    assert_eq!(
        npcs.len(),
        NPC_DRIVERS,
        "the shipped max_drivers fills NPCs"
    );
    for value in 3..=8 {
        let out = gateway_send(gateway, &phase_for(key, value), base + 1_000 + value as u64);
        assert!(!out.is_empty(), "phase {value}");
    }
    for value in (0..=4).rev() {
        let out = gateway_send(
            gateway,
            &countdown_for(key, value),
            base + 2_000 + (4 - value) as u64,
        );
        assert!(!out.is_empty(), "countdown {value}");
    }
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Running)
    );
    (key, npcs)
}

/// Knock out all seven NPC drivers, wait out their respawn, then knock out four more for the
/// shipped first-to-22 target. Returns the final results reply bodies.
fn drive_to_results(
    gateway: &mut Gateway,
    key: ParticipantKey,
    npcs: &[ParticipantKey],
    base: u64,
) -> Vec<Vec<u8>> {
    gateway_send(gateway, &pose(key), base + 2_900);
    for (index, npc) in npcs.iter().enumerate() {
        let out = gateway_send(
            gateway,
            &hit_for(key, *npc, 100.0),
            base + 3_000 + index as u64 * 100,
        );
        assert!(!out.is_empty(), "the contact is resolved by the server");
        assert_eq!(
            gateway.gameplay().unwrap().derby_phase(),
            Some(DerbyPhase::Running),
            "the first seven knockouts stay below the 22-point target"
        );
    }
    let respawn_at = base + 8_000;
    gateway_send(gateway, &pose(key), respawn_at - 100);
    let respawned = gateway.tick_at(respawn_at).unwrap();
    assert!(
        respawned
            .replies
            .iter()
            .map(|packet| &packet[7..])
            .any(|packet| packet.starts_with(&[34, 0, 18])),
        "the knocked-out drivers reset"
    );
    for (index, npc) in npcs.iter().take(KNOCKOUTS_TO_WIN).enumerate() {
        let out = gateway_send(
            gateway,
            &hit_for(key, *npc, 100.0),
            respawn_at + 100 + index as u64 * 100,
        );
        if index + 1 == KNOCKOUTS_TO_WIN {
            assert_eq!(
                gateway.gameplay().unwrap().derby_phase(),
                Some(DerbyPhase::Results),
                "the shipped 22-point target ends the match"
            );
            return out;
        }
    }
    panic!("the shipped 22-point target was not reached");
}

fn money(gateway: &Gateway) -> (u32, u32) {
    let state = gateway
        .snapshot()
        .gameplay
        .expect("the session has gameplay");
    let xp = state
        .profiles
        .as_ref()
        .unwrap()
        .experience
        .get(&osfr_tools::derby_gateway::DEMO_DERBY_DRIVER_PROFILE)
        .copied()
        .unwrap_or(0);
    (state.inventory.coins, xp)
}

fn ledger(gateway: &Gateway) -> Vec<i32> {
    gateway
        .snapshot()
        .gameplay
        .and_then(|state| state.derby)
        .map(|derby| derby.paid_matches)
        .unwrap_or_default()
}

fn saved_bytes(gateway: &Gateway) -> Vec<u8> {
    serde_json::to_vec(&gateway.snapshot()).unwrap()
}

#[test]
fn missing_rows_file_means_one_row() {
    let package = Package::new(&["derby_entry.json", "derby_gates.json"]);
    let gateway = DerbyGateway::load(package.path(), None)
        .expect("the default package loads")
        .expect("the default package exists");
    assert_eq!(gateway.rows().len(), 1);
    assert_eq!(gateway.rows()[0].game_id(), DEFAULT_GAME);
    assert_eq!(
        gateway
            .rows_for(MAD_JACK)
            .map(|row| row.game_id())
            .collect::<Vec<_>>(),
        vec![DEFAULT_GAME]
    );
    assert_eq!(
        gateway
            .rows_for(BURT)
            .map(|row| row.game_id())
            .collect::<Vec<_>>(),
        vec![DEFAULT_GAME]
    );
    assert!(gateway.rows_for(NOT_A_GATE).next().is_none());
}

#[test]
fn rows_inherit_the_default_package() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    package.write_rows(vec![one_row(BREAK_ER_IN, &[MAD_JACK])]);
    let gateway = DerbyGateway::load(package.path(), None)
        .expect("the row inherits every unset field")
        .expect("the default package exists");
    let default = default_package(&package);
    let row = gateway
        .rows()
        .iter()
        .find(|row| row.game_id() == BREAK_ER_IN)
        .expect("the extra row is hosted");
    assert_eq!(row.package().vehicle, default.vehicle);
    assert_eq!(row.package().rules, default.rules);
    assert_eq!(row.package().pickups, default.pickups);
    assert_eq!(
        row.package().completion_reward_coins,
        default.completion_reward_coins
    );
    assert_eq!(
        row.package().completion_reward_xp,
        default.completion_reward_xp
    );
    assert_eq!(
        gateway
            .rows_for(MAD_JACK)
            .map(|row| row.game_id())
            .collect::<Vec<_>>(),
        vec![DEFAULT_GAME, BREAK_ER_IN],
        "the default row is entered first; the extra row only by an open objective"
    );
    assert_eq!(
        gateway.rows_for(CRASH).next().map(|row| row.game_id()),
        Some(DEFAULT_GAME),
        "a row lists only its own NPCs"
    );
}

#[test]
fn row_overrides_are_the_effective_package() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    let default = default_package(&package);
    let mut rules = serde_json::to_value(&default.rules).unwrap();
    rules["score_target"] = json!(3);
    let pickup = json!({
        "id": 1,
        "definition_id": 1,
        "kind": "health",
        "position_bits": [0, 0, 0, 0],
        "unknown_floats_bits": [0, 0],
        "response_parameters_bits": [0, 0, 0],
    });
    let mut row = one_row(BREAK_ER_IN, &[MAD_JACK]);
    row["completion_reward_coins"] = json!(7);
    row["completion_reward_xp"] = json!(3);
    row["rules"] = rules;
    row["pickups"] = json!([pickup]);
    package.write_rows(vec![row]);
    let mut gateway = DerbyGateway::load(package.path(), None)
        .expect("the override row loads")
        .expect("the default package exists");
    let character = CharacterState {
        guid: 791_007,
        zone_id: 0,
        position: [0.0; 3],
        rotation: [0.0; 3],
    };
    let hosted = gateway
        .rows()
        .iter()
        .find(|row| row.game_id() == BREAK_ER_IN)
        .unwrap()
        .package()
        .clone();
    assert_eq!(hosted.rules.score_target, Some(3), "the rules override");
    assert_ne!(hosted.rules, default.rules);
    assert_eq!(hosted.pickups.len(), 1, "the pickups override");
    assert_eq!(hosted.vehicle, default.vehicle, "unset fields inherit");
    assert_eq!(
        gateway.rows()[0].package(),
        &default,
        "the default row is untouched by another row's overrides"
    );
    let offer = gateway
        .begin_row(&character, 0, Some(BREAK_ER_IN))
        .expect("the hosted row starts");
    assert_eq!(
        i32::from_le_bytes(offer[11..15].try_into().unwrap()),
        BREAK_ER_IN
    );
    assert_eq!(gateway.game_id(), BREAK_ER_IN as u32);
    let grant = gateway
        .completion_grant(character.guid)
        .expect("the row pays its own reward");
    assert_eq!(grant.coins, 7);
    assert_eq!(
        grant.job_xp.get(&DEMO_DERBY_DRIVER_PROFILE),
        Some(&3),
        "the row's inherited/overridden XP comes from its effective package"
    );
}

#[test]
fn duplicated_rows_and_the_default_id_are_rejected() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    package.write_rows(vec![
        one_row(BREAK_ER_IN, &[MAD_JACK]),
        one_row(BREAK_ER_IN, &[CRASH]),
    ]);
    let before = fs::read(package.path().join("derby_rows.json")).unwrap();
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error
            .to_string()
            .contains("duplicate derby row game id 604"),
        "{error}"
    );
    assert_eq!(
        fs::read(package.path().join("derby_rows.json")).unwrap(),
        before,
        "a refused load changes no file"
    );

    package.write_rows(vec![one_row(DEFAULT_GAME, &[MAD_JACK])]);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error.to_string().contains("duplicate derby row game id 39"),
        "the default package id is already hosted: {error}"
    );
}

#[test]
fn row_with_wrong_type_is_rejected() {
    let package = Package::shipped();
    package.write_rows(vec![one_row(BREAK_ER_IN, &[MAD_JACK])]);
    let mini = package.path().join("derby_minigames.json");
    let rows = package.path().join("derby_rows.json");
    let mut catalogue = json::read(&mini);
    catalogue["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == BREAK_ER_IN)
        .expect("the shipped catalogue names row 604")["game_type_id"] = json!(3);
    json::write_pretty(&mini, &catalogue);
    let before_mini = fs::read(&mini).unwrap();
    let before_rows = fs::read(&rows).unwrap();
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error.to_string().contains("604") && error.to_string().contains("type-11 row"),
        "a non-derby MiniGameData row must be refused: {error}"
    );
    assert_eq!(
        fs::read(&mini).unwrap(),
        before_mini,
        "no file is rewritten"
    );
    assert_eq!(
        fs::read(&rows).unwrap(),
        before_rows,
        "no file is rewritten"
    );

    catalogue["records"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == BREAK_ER_IN)
        .unwrap()["game_type_id"] = json!(11);
    json::write_pretty(&mini, &catalogue);
    let gateway = DerbyGateway::load(package.path(), None)
        .expect("a TYPE-11 row loads")
        .expect("the default package exists");
    assert_eq!(
        gateway
            .rows()
            .iter()
            .map(|row| row.game_id())
            .collect::<Vec<_>>(),
        vec![DEFAULT_GAME, BREAK_ER_IN]
    );
}

#[test]
fn row_with_unknown_id_is_rejected() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    package.write_rows(vec![one_row(999_999, &[MAD_JACK])]);
    let before = fs::read(package.path().join("derby_rows.json")).unwrap();
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error.to_string().contains("999999") && error.to_string().contains("type-11 row"),
        "an id with no MiniGameData record must be refused: {error}"
    );
    assert_eq!(
        fs::read(package.path().join("derby_rows.json")).unwrap(),
        before,
        "a refused load changes no file"
    );
}

#[test]
fn row_npc_outside_gates_is_rejected() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    package.write_rows(vec![one_row(BREAK_ER_IN, &[NOT_A_GATE])]);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error
            .to_string()
            .contains(&format!("NPC {NOT_A_GATE} outside derby_gates.json")),
        "{error}"
    );
}

#[test]
fn row_reward_above_cap_is_rejected_at_load() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    let mut row = one_row(BREAK_ER_IN, &[MAD_JACK]);
    row["completion_reward_coins"] = json!(MAX_COMPLETION_REWARD + 1);
    package.write_rows(vec![row]);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("the package is refused");
    assert!(
        error.to_string().contains("derby row 604") && error.to_string().contains("invalid derby"),
        "{error}"
    );
}

#[test]
fn default_row_must_be_type_11_when_the_table_is_present() {
    let package = Package::shipped();
    let entry = package.path().join("derby_entry.json");
    let mut content = json::read(&entry);
    // MiniGameData row 11 is the Trading Card Game, never a derby row.
    content["game_id"] = json!(11);
    json::write_pretty(&entry, &content);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("a non-derby default id is refused");
    assert!(
        error.to_string().contains("default row 11") && error.to_string().contains("type-11 row"),
        "{error}"
    );
}

#[test]
fn rows_without_the_minigame_table_are_rejected() {
    let package = Package::new(&["derby_entry.json", "derby_gates.json"]);
    package.write_rows(vec![one_row(BREAK_ER_IN, &[MAD_JACK])]);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("an untyped row is refused");
    assert!(
        error.to_string().contains("derby_minigames.json"),
        "{error}"
    );
}

#[test]
fn minigame_table_follows_the_content_layer_rules() {
    let package = Package::shipped();
    let table = package.path().join("derby_minigames.json");
    let shipped = json::read(&table);

    let mut unsourced = shipped.clone();
    unsourced["records"][1]["source"] = json!([]);
    json::write_pretty(&table, &unsourced);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("a record with no provenance is refused");
    assert!(error.to_string().contains("minigame 604"), "{error}");

    let mut duplicated = shipped.clone();
    let first = duplicated["records"][0].clone();
    duplicated["records"].as_array_mut().unwrap().push(first);
    json::write_pretty(&table, &duplicated);
    let error = DerbyGateway::load(package.path(), None)
        .err()
        .expect("a duplicate minigame id is refused");
    assert!(error.to_string().contains("39"), "{error}");

    json::write_pretty(&table, &shipped);
    assert!(DerbyGateway::load(package.path(), None).is_ok());
}

#[test]
fn shipped_rows_are_hosted_at_the_four_gates() {
    let gateway = DerbyGateway::load(&content_dir(), None)
        .expect("the shipped package and rows load")
        .expect("the shipped derby package exists");
    assert_eq!(
        gateway
            .rows()
            .iter()
            .map(|row| row.game_id())
            .collect::<Vec<_>>(),
        vec![DEFAULT_GAME, BREAK_ER_IN]
    );
    for gate in [MAD_JACK, CRASH, BURT, RIP] {
        assert_eq!(
            gateway
                .rows_for(gate)
                .map(|row| row.game_id())
                .collect::<Vec<_>>(),
            vec![DEFAULT_GAME, BREAK_ER_IN],
            "gate {gate}"
        );
    }
    assert!(gateway.rows_for(NOT_A_GATE).next().is_none());
}

#[test]
fn unlisted_npc_enters_no_row() {
    let guid = 791_001;
    let fixture = derby_fixture(guid);
    let (_, position) = npc_location(NOT_A_GATE);
    let mut gateway = fixture.session(guid, position, None);
    let before = saved_bytes(&gateway);
    let menu = gateway_send(&mut gateway, &npc_interact(NOT_A_GATE), NOW);
    let packet = menu
        .iter()
        .find(|packet| packet.starts_with(&[26, 0, 9, 0]))
        .expect("the non-gate NPC opens its menu");
    assert!(
        menu_option_opt(packet, DERBY_LABEL).is_none(),
        "a non-gate NPC lists no derby option"
    );
    let refused = gateway_send(&mut gateway, &npc_select(NOT_A_GATE, 9_999), NOW + 1);
    assert!(refused.is_empty(), "the forged selection answers nothing");
    assert_eq!(
        saved_bytes(&gateway),
        before,
        "an unlisted NPC changes no saved state"
    );
    assert!(!gateway.gameplay().unwrap().derby_active());
}

#[test]
fn finished_match_credits_the_entered_row() {
    let guid = 791_002;
    let fixture = derby_fixture(guid);
    patch_derby_quest(&fixture, &[BREAK_ER_IN, DEFAULT_GAME], 1);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    {
        let character = gateway.state().clone();
        let game = gateway.gameplay_mut().unwrap();
        accept_license(game, &character);
        assert_eq!(counts(game, LICENSE_TO_DRIVE), vec![0, 0]);
    }
    let (coins_before, xp_before) = money(&gateway);
    let (instance, game_id) = enter_derby(&mut gateway, CRASH, NOW);
    assert_eq!(
        game_id, BREAK_ER_IN,
        "the open 604 objective selects its row"
    );
    let (key, npcs) = start_and_ready(&mut gateway, instance, BREAK_ER_IN, NOW);
    let results = drive_to_results(&mut gateway, key, &npcs, NOW);
    assert!(
        results
            .iter()
            .filter(|packet| packet.starts_with(&[34, 0, 1]))
            .any(|packet| decode_inner(packet) == Ok(DerbyMessage::GameState(10))),
        "the server-observed results phase"
    );
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1, 0],
        "only the entered row's objective is credited"
    );
    let (coins_after, xp_after) = money(&gateway);
    assert_eq!(
        coins_after,
        coins_before + 25,
        "the entered row's inherited completion reward"
    );
    assert_eq!(xp_after, xp_before + 10);
    assert_eq!(ledger(&gateway), vec![instance]);
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Results)
    );
}

/// Each finished match credits its row once, and records replayed after results (and after a
/// reconnect) change nothing. It cannot replay a finish inside one live match: a reconnect drops
/// the match, so the same-instance double `take_completion` is unreachable through a session.
#[test]
fn finished_match_credits_once_and_replays_change_nothing() {
    let guid = 791_003;
    let fixture = derby_fixture(guid);
    patch_derby_quest(&fixture, &[BREAK_ER_IN, DEFAULT_GAME], 2);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    {
        let character = gateway.state().clone();
        let game = gateway.gameplay_mut().unwrap();
        accept_license(game, &character);
    }
    let (first, game_id) = enter_derby(&mut gateway, CRASH, NOW);
    assert_eq!(game_id, BREAK_ER_IN);
    let (key, npcs) = start_and_ready(&mut gateway, first, BREAK_ER_IN, NOW);
    drive_to_results(&mut gateway, key, &npcs, NOW);
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1, 0]
    );
    assert_eq!(ledger(&gateway), vec![first]);

    // Replaying the record that drove the match to results changes nothing.
    let replay = gateway_send(&mut gateway, &hit_for(key, npcs[0], 100.0), NOW + 8_500);
    assert!(
        replay.is_empty(),
        "the results phase ignores a contact replay"
    );
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1, 0],
        "a replayed record does not credit twice"
    );
    assert_eq!(ledger(&gateway), vec![first]);

    // A reconnect while the results screen shows drops the live match but keeps the ledger; the
    // client replaying its finishing records into the new session must not credit again.
    let saved: GameplayState = gateway
        .snapshot()
        .gameplay
        .expect("the session has gameplay");
    let mut gateway = fixture.session(guid, position, Some(saved));
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_eq!(ledger(&gateway), vec![first], "the ledger survives");
    let before = saved_bytes(&gateway);
    for record in [
        hit_for(key, npcs[0], 100.0),
        phase_for(key, 10),
        start_request(first),
    ] {
        gateway_send(&mut gateway, &record, NOW + 9_000);
    }
    assert_eq!(
        saved_bytes(&gateway),
        before,
        "a replayed finishing record changes nothing"
    );
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1, 0],
        "the replayed finish credits nothing"
    );

    let base = NOW + 8_400 + RESULTS_HOLD_MS + 20_000;
    let (second, game_id) = enter_derby(&mut gateway, CRASH, base);
    assert_eq!(
        game_id, BREAK_ER_IN,
        "the objective is still open at count 2"
    );
    assert!(second > first, "a paid instance id is never reissued");
    let (key, npcs) = start_and_ready(&mut gateway, second, BREAK_ER_IN, base);
    drive_to_results(&mut gateway, key, &npcs, base);
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![2, 0],
        "the second finish credits its row once"
    );
    assert_eq!(
        ledger(&gateway),
        vec![first, second],
        "the row-agnostic ledger records each match once"
    );
}

#[test]
fn reconnect_keeps_the_entered_row() {
    let guid = 791_004;
    let fixture = derby_fixture(guid);
    patch_derby_quest(&fixture, &[BREAK_ER_IN], 1);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    {
        let character = gateway.state().clone();
        let game = gateway.gameplay_mut().unwrap();
        accept_license(game, &character);
    }
    let (instance, game_id) = enter_derby(&mut gateway, CRASH, NOW);
    assert_eq!(game_id, BREAK_ER_IN);
    let (_key, _npcs) = start_and_ready(&mut gateway, instance, BREAK_ER_IN, NOW);

    // A mid-match reconnect drops the live match (as before), keeps the open objective, and the
    // next entry therefore selects the same row instead of re-admitting into another one.
    let saved = gateway.snapshot();
    let saved_state: GameplayState = saved.gameplay.expect("the session has gameplay");
    assert!(
        saved_state
            .derby
            .as_ref()
            .is_none_or(|derby| derby.paid_matches.is_empty()),
        "an unfinished match writes no ledger row"
    );
    let mut reconnected = fixture.session(guid, position, Some(saved_state));
    assert!(!reconnected.gameplay().unwrap().derby_active());
    assert_eq!(
        counts(reconnected.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![0],
        "the open row objective survived the reconnect"
    );
    let (second, game_id) = enter_derby(&mut reconnected, CRASH, NOW + 20_000);
    assert_eq!(
        game_id, BREAK_ER_IN,
        "re-entry keeps the entered row while its objective is open"
    );

    let (key, npcs) = start_and_ready(&mut reconnected, second, BREAK_ER_IN, NOW + 20_000);
    drive_to_results(&mut reconnected, key, &npcs, NOW + 20_000);
    assert_eq!(
        counts(reconnected.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1]
    );
    let paid: Vec<i32> = ledger(&reconnected);
    assert_eq!(paid, vec![second]);
    let saved = reconnected.snapshot();
    let reloaded = fixture.session(guid, position, saved.gameplay);
    let state = reloaded
        .snapshot()
        .gameplay
        .expect("the session has gameplay");
    assert_eq!(
        state.derby,
        Some(DerbyGatewayState {
            previous_job: None,
            last_paid_unix: None,
            paid_matches: vec![second],
        }),
        "the paid ledger survives the reconnect"
    );
    assert_eq!(
        counts(reloaded.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1],
        "the credited objective survives the reconnect"
    );
}

/// Row choice is per session. Each `Gateway` session owns its own `DerbyGateway` and instance
/// counter, so lobbies are separate whatever the row logic does; this test pins the row each
/// session enters and that one session's start leaves the other's lobby alone, not a
/// cross-session lobby-isolation invariant of the row feature.
#[test]
fn each_session_enters_its_own_row() {
    let first_guid = 791_005;
    let second_guid = 791_006;
    let fixture = derby_fixture(first_guid);
    patch_derby_quest(&fixture, &[BREAK_ER_IN], 1);
    let (_, first_position) = npc_location(CRASH);
    let (_, second_position) = npc_location(MAD_JACK);
    let mut first = fixture.session(first_guid, first_position, None);
    {
        let character = first.state().clone();
        let game = first.gameplay_mut().unwrap();
        accept_license(game, &character);
    }
    let mut second = fixture.session(second_guid, second_position, None);
    let (first_instance, first_game) = enter_derby(&mut first, CRASH, NOW);
    let (second_instance, second_game) = enter_derby(&mut second, MAD_JACK, NOW);
    assert_eq!(first_game, BREAK_ER_IN);
    assert_eq!(second_game, DEFAULT_GAME, "no objective selects row 39");
    assert_eq!(
        first_instance, second_instance,
        "each session owns its counter"
    );
    gateway_send(&mut first, &start_request(first_instance), NOW + 1_000);
    assert_eq!(
        first.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Starting)
    );
    assert_eq!(
        second.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Lobby),
        "the first session's start does not move the second lobby"
    );
    assert!(!gateway_send(&mut second, &start_request(second_instance), NOW + 1_100).is_empty());
    assert_eq!(
        first.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Starting),
        "the second session's start does not move the first"
    );
}
