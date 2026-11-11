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
    derby::{
        DerbyMessage, DerbyPhase, ParticipantKey, VehicleCreate, VehiclePose, decode_inner,
        encode_inner,
    },
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
/// L931: the shipped hosted rows in file order after the default row: 604, the Games window rows
/// 40/41 and tutorials 289/290, then the quest rows 607, 615, 616, 624 and 627, then the rest of
/// the client's named derby rows (2026-11-08).
const SHIPPED_ROWS: [i32; 30] = [39, 604, 40, 41, 289, 290, 607, 615, 616, 624, 627, 603, 605, 606, 608, 609, 610, 611, 612, 613, 614, 617, 618, 619, 620, 621, 622, 623, 625, 626];
const DERBY_FILES: [&str; 4] = [
    "derby_entry.json",
    "derby_gates.json",
    "derby_rows.json",
    "derby_minigames.json",
];
const NOW: u64 = 10_000;
/// The client's Loaded event (gateway opcode 10) after a scene change.
const LOADED: [u8; 2] = 10_u16.to_le_bytes();
/// L931: after the arena's zoning the client sends Ready (13) and then Loaded (10).
const READY: [u8; 2] = 13_u16.to_le_bytes();
const NPC_DRIVERS: usize = 7;
const KNOCKOUTS_TO_WIN: usize = 4;
const RESULTS_HOLD_MS: u64 = 10_000;
/// From a match's base time to the tick that ends it: the countdown reaches 0 at base + 2_004 and
/// the shipped clock runs 240 s.
const FINISH_AT: u64 = 2_004 + 240_000;

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
    pose_at(key, [0.0; 3])
}

/// L931: the player's car at `at` (the arena's cars stand apart; a hit is made beside one).
fn pose_at(key: ParticipantKey, at: [f32; 3]) -> Vec<u8> {
    encode_inner(&DerbyMessage::VehiclePose(VehiclePose {
        key,
        transform_bits: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, at[0], at[1], at[2], 1.0,
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
    let mut entered = gateway_send(gateway, &npc_select(driver, option), base + 1);
    // L931: the arena loads first; the 39/16 offer follows its Ready and Loaded.
    entered.extend(gateway_send(gateway, &READY, base + 2));
    entered.extend(gateway_send(gateway, &LOADED, base + 3));
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

/// L931: pose the player's car beside `npc`'s, where a contact counts.
fn beside(gateway: &mut Gateway, key: ParticipantKey, npc: ParticipantKey, now: u64) {
    let at = gateway.gameplay().unwrap().derby_driver_position(npc);
    gateway_send(gateway, &pose_at(key, at.expect("the car has a pose")), now);
}

/// Knock out all seven NPC drivers, wait out their respawn, knock out four more, then let the
/// shipped 240 s clock end the match (no score target: retail high scores reach 50 in one match,
/// `nb5-qin8zz0_154.00`). Returns the finishing tick's reply bodies.
fn drive_to_results(
    gateway: &mut Gateway,
    key: ParticipantKey,
    npcs: &[ParticipantKey],
    base: u64,
) -> Vec<Vec<u8>> {
    gateway_send(gateway, &pose(key), base + 2_900);
    for (index, npc) in npcs.iter().enumerate() {
        let at = base + 3_000 + index as u64 * 100;
        beside(gateway, key, *npc, at - 1);
        let out = gateway_send(gateway, &hit_for(key, *npc, 10_000.0), at);
        assert!(!out.is_empty(), "the contact is resolved by the server");
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
        let at = respawn_at + 100 + index as u64 * 100;
        beside(gateway, key, *npc, at - 1);
        gateway_send(gateway, &hit_for(key, *npc, 10_000.0), at);
    }
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Running),
        "eleven knock-outs end nothing: only the clock does"
    );
    let finished = gateway.tick_at(base + FINISH_AT).unwrap();
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Results),
        "the shipped 240 s clock ends the match"
    );
    finished
        .replies
        .iter()
        .map(|packet| packet[7..].to_vec())
        .collect()
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
        .completion_grant(character.guid, None)
        .expect("the row pays its own reward");
    assert_eq!(grant.coins, 7);
    assert_eq!(
        grant.job_xp.get(&DEMO_DERBY_DRIVER_PROFILE),
        Some(&3),
        "the row's inherited/overridden XP comes from its effective package"
    );
    // L927: the job's activity XP from job_xp_curve.json replaces the package's XP, not its coins.
    let rated = gateway.completion_grant(character.guid, Some(78)).unwrap();
    assert_eq!(
        (rated.coins, rated.job_xp.get(&DEMO_DERBY_DRIVER_PROFILE)),
        (7, Some(&78))
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

    // A row with the default package's own id overrides the default row in place of the
    // captured `derby_entry.json` (its NPCs and rules), so the capture itself never changes.
    let mut default_row = one_row(DEFAULT_GAME, &[MAD_JACK]);
    default_row["rules"] = json!({"score_target": null, "time_limit_ms": 90_000});
    package.write_rows(vec![default_row]);
    let gateway = DerbyGateway::load(package.path(), None)
        .expect("the default-id row overrides the default row")
        .expect("the default package exists");
    assert_eq!(gateway.rows().len(), 1, "no second row 39 is hosted");
    assert_eq!(gateway.rows()[0].game_id(), DEFAULT_GAME);
    assert_eq!(gateway.rows()[0].package().rules.score_target, None);
    assert_eq!(
        gateway.rows()[0].package().rules.time_limit_ms,
        Some(90_000)
    );
    assert!(gateway.rows_for(CRASH).next().is_none(), "the override's NPCs");
    assert_eq!(
        gateway.rows_for(MAD_JACK).next().map(|row| row.game_id()),
        Some(DEFAULT_GAME)
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
        SHIPPED_ROWS
    );
    for gate in [MAD_JACK, CRASH, BURT, RIP] {
        assert_eq!(
            gateway
                .rows_for(gate)
                .map(|row| row.game_id())
                .collect::<Vec<_>>(),
            SHIPPED_ROWS,
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
    let replay = gateway_send(
        &mut gateway,
        &hit_for(key, npcs[0], 10_000.0),
        NOW + FINISH_AT + 100,
    );
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
        hit_for(key, npcs[0], 10_000.0),
        phase_for(key, 10),
        start_request(first),
    ] {
        gateway_send(&mut gateway, &record, NOW + FINISH_AT + 600);
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

    let base = NOW + FINISH_AT + RESULTS_HOLD_MS + 20_000;
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

// ---- L931: the quest rows, row rules, server-driven NPCs and the Games window ----

/// A row's `rules` object changes only the keys it names; every other key inherits.
#[test]
fn row_rules_are_a_partial_override() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    let default = default_package(&package);
    let mut row = one_row(BREAK_ER_IN, &[MAD_JACK]);
    row["rules"] = json!({"time_limit_ms": 90_000});
    package.write_rows(vec![row.clone()]);
    let gateway = DerbyGateway::load(package.path(), None)
        .expect("a partial rules row loads")
        .expect("the default package exists");
    let mut expected = default.rules.clone();
    expected.time_limit_ms = Some(90_000);
    assert_eq!(gateway.rows()[1].package().rules, expected);
    assert_eq!(gateway.rows()[0].package().rules, default.rules);

    for bad in [json!({"no_such_rule": 1}), json!({"max_drivers": 1})] {
        row["rules"] = bad;
        package.write_rows(vec![row.clone()]);
        assert!(
            DerbyGateway::load(package.path(), None).is_err(),
            "{row}: an unknown key or a value the service refuses fails the load"
        );
    }
}

fn driving_json() -> Value {
    json!({
        "pose_interval_ms": 100, "speed_mps": 12.0, "spawn_radius_m": 25.0,
        "bump_impact": 5.0, "bump_interval_ms": 1500
    })
}

/// `npc_driving` is validated at load; `default_npc_driving` is the default row's switch.
#[test]
fn npc_driving_is_validated_at_load_and_the_default_row_has_a_switch() {
    let package = Package::new(&[
        "derby_entry.json",
        "derby_gates.json",
        "derby_minigames.json",
    ]);
    let mut row = one_row(BREAK_ER_IN, &[MAD_JACK]);
    row["npc_driving"] = driving_json();
    package.write_rows(vec![row.clone()]);
    let gateway = DerbyGateway::load(package.path(), None).unwrap().unwrap();
    assert!(
        gateway.rows()[0].npc_driving().is_none(),
        "the default row stays inert"
    );
    let driving = gateway.rows()[1]
        .npc_driving()
        .expect("the row drives its NPCs");
    assert_eq!(driving.bump_interval_ms, 1_500);

    row["npc_driving"]["speed_mps"] = json!(-1.0);
    package.write_rows(vec![row]);
    assert!(
        DerbyGateway::load(package.path(), None).is_err(),
        "a bad row is refused"
    );

    let mut file = rows_file(vec![]);
    file["default_npc_driving"] = driving_json();
    json::write_pretty(&package.path().join("derby_rows.json"), &file);
    let gateway = DerbyGateway::load(package.path(), None).unwrap().unwrap();
    assert!(
        gateway.rows()[0].npc_driving().is_some(),
        "the default row drives"
    );
    file["default_npc_driving"]["bump_impact"] = json!(0.0);
    json::write_pretty(&package.path().join("derby_rows.json"), &file);
    assert!(
        DerbyGateway::load(package.path(), None).is_err(),
        "a bad default is refused"
    );
}

/// The shipped quest rows run driven, clocked matches; the earlier rows are untouched.
#[test]
fn shipped_quest_rows_run_driven_matches_on_the_clock() {
    let gateway = DerbyGateway::load(&content_dir(), None).unwrap().unwrap();
    for row in gateway.rows() {
        // L931: as live, the server drives every row's AI cars on the arena's 240 s clock.
        assert!(row.npc_driving().is_some(), "row {}", row.game_id());
        let rules = &row.package().rules;
        assert_eq!(rules.time_limit_ms, Some(240_000), "row {}", row.game_id());
        // Retail high scores reach 50 in one match (`nb5-qin8zz0_154.00`): the clock alone ends
        // it, except the Wildwood Derby 1000 (626), whose client text 394481 says the first to 22
        // points wins.
        let target = (row.game_id() == 626).then_some(22);
        assert_eq!(rules.score_target, target, "row {}", row.game_id());
    }
}

/// The Frost Ridge and Tangletrack rows zone into their own arenas (maps.md 5.2, 5.3) with their
/// retail rosters and their own start slots, through the Games window Play the player uses.
#[test]
fn arena_rows_play_in_their_own_arena_with_their_roster() {
    let frost_ridge = [19_354, 19_357, 19_352, 19_351, 19_353, 19_355, 19_356];
    // Six names read off the 2009 results board; the seventh car's driver, Jess Roadrunner 19342,
    // is the client's one unplaced Tangletrack driver name (R).
    let tangletrack = [19_337, 19_340, 19_338, 19_343, 19_341, 19_339, 19_342];
    for (offset, (live_row, game, arena, start, names)) in [
        // F3 (rc-0136): Frost Ridge's player start moved off the slope that rolled the car into the
        // central pile while the arena loaded (240, 106, 215 is now AI slot 7's).
        (67_u32, 40_i32, "sh_dm_arena", [240.0_f32, 102.0, 310.0], &frost_ridge[..]),
        (68, 41, "bw_dm_arena", [305.0, 3.0, 265.0], &tangletrack[..]),
    ]
    .into_iter()
    .enumerate()
    {
        let guid = 931_300 + offset as u64;
        let fixture = derby_fixture(guid);
        fixture.copy_content("activity_catalogue.json");
        let (_, position) = npc_location(CRASH);
        let mut gateway = fixture.session(guid, position, None);
        let mut replies = gateway_send(&mut gateway, &play(live_row), NOW);
        assert_eq!(
            gateway.gameplay().unwrap().current_zone().unwrap().name,
            arena,
            "row {live_row} zones into its own arena"
        );
        replies.extend(gateway_send(&mut gateway, &READY, NOW + 1));
        replies.extend(gateway_send(&mut gateway, &LOADED, NOW + 2));
        assert_eq!(offer_game_id(&replies), game);
        let instance = offer_instance(&replies);
        let key = ParticipantKey(guid);
        gateway_send(&mut gateway, &start_request(instance), NOW + 300);
        let setup = gateway_send(&mut gateway, &register_for(key), NOW + 400);
        let at = |bits: &[u32; 16]| [12, 13, 14].map(|slot| f32::from_bits(bits[slot]));
        let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
        let mut drivers = Vec::new();
        let mut human_start = None;
        for (index, packet) in setup.iter().enumerate() {
            let Ok(DerbyMessage::CreateVehicle(car)) = decode_inner(packet) else {
                continue;
            };
            if car.key == key {
                human_start = Some(at(&car.start_transform_bits));
                continue;
            }
            // L959: a named slot's driver NPC precedes its car; an unnamed slot has none.
            if let Some(npc) = index
                .checked_sub(1)
                .and_then(|i| osfr_tools::startup_world::parse_add_npc(&setup[i]).ok())
            {
                assert_eq!(npc.actor_id, car.key.0);
                assert!(
                    near(at(&car.start_transform_bits), npc.position[..3].try_into().unwrap()),
                    "the driver stands at the car's own slot"
                );
                assert!(
                    !near(at(&car.start_transform_bits), [360.09, 15.0, 215.0]),
                    "row {live_row}: the Wildwood slots are not reused"
                );
                drivers.push(npc.name_string_id);
            }
        }
        assert!(
            near(human_start.expect("the player's car"), start),
            "row {live_row}: the player's car starts in its arena: {human_start:?}"
        );
        assert_eq!(drivers, names, "row {live_row}: the arena's own roster");
    }
}

const WEAR_AND_TEAR: i32 = 607;

fn play(row: u32) -> Vec<u8> {
    let mut inner = vec![167, 0, 1, 2];
    inner.extend(row.to_le_bytes());
    inner.extend(0u32.to_le_bytes());
    inner
}

/// Games window Play starts the row the player chose: 65 -> 39, 67 -> 40, 68 -> 41 and the
/// tutorial 1047 -> 289 (their MiniGameData ContentIds), each as its own 39/16 offer.
#[test]
fn games_window_play_starts_the_row_chosen() {
    for (offset, (live_row, game)) in [(65_u32, 39_i32), (67, 40), (68, 41), (1047, 289)]
        .into_iter()
        .enumerate()
    {
        let guid = 931_100 + offset as u64;
        let fixture = derby_fixture(guid);
        fixture.copy_content("activity_catalogue.json");
        let (_, position) = npc_location(CRASH);
        let mut gateway = fixture.session(guid, position, None);
        let mut replies = gateway_send(&mut gateway, &play(live_row), NOW);
        replies.extend(gateway_send(&mut gateway, &READY, NOW + 1));
        replies.extend(gateway_send(&mut gateway, &LOADED, NOW + 2));
        assert_eq!(offer_game_id(&replies), game, "Play on row {live_row}");
        assert!(gateway.gameplay().unwrap().derby_active());
    }
}

/// A driven quest row (607) is entered at a gate, its NPCs appear and chase once the human's
/// first pose arrives, the clock ends the match, and the finish credits 607 and nothing else.
#[test]
fn driven_row_finishes_on_the_clock_and_credits_its_own_id() {
    let guid = 931_005;
    let fixture = derby_fixture(guid);
    patch_derby_quest(&fixture, &[WEAR_AND_TEAR, DEFAULT_GAME], 1);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    {
        let character = gateway.state().clone();
        accept_license(gateway.gameplay_mut().unwrap(), &character);
    }
    let (coins_before, xp_before) = money(&gateway);
    let (instance, game_id) = enter_derby(&mut gateway, CRASH, NOW);
    assert_eq!(
        game_id, WEAR_AND_TEAR,
        "the open 607 objective selects its row"
    );
    let (key, _) = start_and_ready(&mut gateway, instance, WEAR_AND_TEAR, NOW);

    gateway_send(&mut gateway, &pose(key), NOW + 2_100);
    // L931: the arena's cars start on their own slots (the 34/10 create) and broadcast as they
    // drive; every one moves within the first steps.
    let mut moved = std::collections::BTreeSet::new();
    for at in [NOW + 2_200, NOW + 2_300] {
        let step = gateway.tick_at(at).unwrap();
        moved.extend(
            step.replies
                .iter()
                .map(|packet| &packet[7..])
                .filter(|packet| packet.starts_with(&[34, 0, 5]))
                .map(|packet| packet[4..12].to_vec()),
        );
        assert!(
            !step.state_changed && !step.gameplay_changed,
            "ephemeral NPC poses never force a character save"
        );
    }
    assert_eq!(
        moved.len(),
        NPC_DRIVERS,
        "every NPC drives and is broadcast"
    );
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Running)
    );
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![0, 0]
    );

    // Countdown zero was acknowledged at NOW + 2_004: the 240 s limit ends the match.
    let finished = gateway.tick_at(NOW + 2_004 + 240_000).unwrap();
    assert_eq!(
        gateway.gameplay().unwrap().derby_phase(),
        Some(DerbyPhase::Results)
    );
    // (R) Wear and Tear's goal (four health power-ups, top 3) was not met by an idle car: the
    // match pays its completion once, says so, and credits no quest.
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![0, 0],
        "a missed goal credits nothing"
    );
    assert!(
        finished
            .replies
            .iter()
            .any(|reply| String::from_utf8_lossy(reply).contains("didn't meet this derby's goal")),
        "the results say the goal was missed"
    );
    let (coins_after, xp_after) = money(&gateway);
    assert_eq!((coins_after, xp_after), (coins_before + 25, xp_before + 10));
    assert_eq!(ledger(&gateway), vec![instance]);
}

/// A goal row met through play: Paws 'n Claws Smash-a-thon (616, Frost Ridge: score 12, top 3)
/// is entered at a gate by its open objective, zones into `sh_dm_arena`, and eleven knock-outs
/// meet the goal, so the clocked finish credits it.
#[test]
fn a_met_goal_row_credits_its_quest_from_its_own_arena() {
    const SMASH_A_THON: i32 = 616;
    let guid = 931_006;
    let fixture = derby_fixture(guid);
    patch_derby_quest(&fixture, &[SMASH_A_THON], 1);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    {
        let character = gateway.state().clone();
        accept_license(gateway.gameplay_mut().unwrap(), &character);
    }
    let (instance, game_id) = enter_derby(&mut gateway, CRASH, NOW);
    assert_eq!(game_id, SMASH_A_THON);
    assert_eq!(
        gateway.gameplay().unwrap().current_zone().unwrap().name,
        "sh_dm_arena",
        "a Frost Ridge quest row plays in the snow arena"
    );
    let (key, npcs) = start_and_ready(&mut gateway, instance, SMASH_A_THON, NOW);
    let results = drive_to_results(&mut gateway, key, &npcs, NOW);
    assert!(
        !results
            .iter()
            .any(|reply| String::from_utf8_lossy(reply).contains("didn't meet")),
        "score 22 in first place meets the goal"
    );
    assert_eq!(
        counts(gateway.gameplay().unwrap(), LICENSE_TO_DRIVE),
        vec![1],
        "the met goal credits the row"
    );
    assert_eq!(ledger(&gateway), vec![instance]);
}

/// L931: the shipped package plays in the live arena (the captured op31 `sg_dm_arena` frame).
/// The menu option zones the player in and the 39/16 offer waits for the arena's Loaded; the cars
/// appear on the captured start slots with the captured bodies; a save in the arena keeps the
/// world pose; the end of the match zones the player back to it.
#[test]
fn the_shipped_derby_plays_in_its_arena_and_returns_to_the_world() {
    let guid = 931_200;
    let fixture = derby_fixture(guid);
    let (_, position) = npc_location(CRASH);
    let mut gateway = fixture.session(guid, position, None);
    let world = gateway.state().position;
    let menu = gateway_send(&mut gateway, &npc_interact(CRASH), NOW);
    let packet = menu.iter().find(|p| p.starts_with(&[26, 0, 9, 0])).unwrap();
    let option = menu_option_bytes(packet, DERBY_LABEL);
    let selected = gateway_send(&mut gateway, &npc_select(CRASH, option), NOW + 1);
    assert!(
        !selected.iter().any(|p| p.starts_with(&[39, 0, 16])),
        "the offer waits for the arena"
    );
    let zone = |gateway: &Gateway| gateway.gameplay().unwrap().current_zone().unwrap().name;
    assert_eq!(zone(&gateway), "sg_dm_arena");
    gateway_send(&mut gateway, &READY, NOW + 2);
    let instance = offer_instance(&gateway_send(&mut gateway, &LOADED, NOW + 3));

    let key = ParticipantKey(guid);
    gateway_send(&mut gateway, &start_request(instance), NOW + 300);
    let setup = gateway_send(&mut gateway, &register_for(key), NOW + 400);
    let cars: Vec<_> = setup
        .iter()
        .filter_map(|packet| match decode_inner(packet) {
            Ok(DerbyMessage::CreateVehicle(car)) => Some(car),
            _ => None,
        })
        .collect();
    assert_eq!(cars.len(), NPC_DRIVERS + 1);
    // L959: each AI car's driver comes first as an NPC with its live name and body (the 35/2 before
    // its 34/10), so the results and the kill feed name it; the human's car brings none.
    let mut names = Vec::new();
    for (index, packet) in setup.iter().enumerate() {
        let Ok(DerbyMessage::CreateVehicle(car)) = decode_inner(packet) else {
            continue;
        };
        let before = index.checked_sub(1).map(|i| setup[i].as_slice());
        if car.key == key {
            assert!(!before.is_some_and(|p| p.starts_with(&[35, 0, 2, 0])));
            continue;
        }
        let npc = osfr_tools::startup_world::parse_add_npc(before.expect("a driver NPC"))
            .expect("the driver NPC precedes its car");
        assert_eq!(npc.actor_id, car.key.0, "the NPC is the car's driver");
        names.push((npc.name_string_id, npc.model_id));
    }
    // The live drivers (packets_demoderby_minigame.pcapng; client locale): Tucker Thunderjolt,
    // Willow Slamdash, Mikko Ragerunner, Jet Madedge, Morgan Slipdrifter, Nina the Wrecker, Carly
    // Collision.
    assert_eq!(
        names,
        [(19365, 1331), (19366, 1297), (19367, 870), (19368, 891), (19369, 888), (19370, 870), (19371, 870)]
    );
    let at = |bits: &[u32; 16]| [12, 13, 14].map(|slot| f32::from_bits(bits[slot]));
    let human = cars.iter().find(|car| car.key == key).unwrap();
    // The live start slots (`opus/minigames/derby.md`), to the capture's precision.
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
    assert!(near(at(&human.start_transform_bits), [373.99, 6.0, 274.55]));
    assert!(human.unknown_flag, "the live human create carries flag 1");
    let slots: Vec<_> = cars.iter().filter(|car| car.key != key).collect();
    assert!(near(
        at(&slots[0].start_transform_bits),
        [360.09, 15.0, 215.0]
    ));
    let body = |car: &&VehicleCreate, model: &[u8]| {
        car.definition.windows(model.len()).any(|w| w == model)
    };
    assert!(body(&slots[1], b"sg_dm_ripper_body_bbec_002.adr"));
    assert!(body(&slots[3], b"sg_dm_crusher_body_bbec_002.adr"));

    assert_eq!(
        gateway.snapshot().character.position,
        world,
        "a save in the arena keeps the world pose"
    );
    let npcs: Vec<_> = slots.iter().map(|car| car.key).collect();
    for value in 3..=8 {
        gateway_send(
            &mut gateway,
            &phase_for(key, value),
            NOW + 1_000 + value as u64,
        );
    }
    for value in (0..=4).rev() {
        gateway_send(
            &mut gateway,
            &countdown_for(key, value),
            NOW + 2_000 + (4 - value) as u64,
        );
    }
    drive_to_results(&mut gateway, key, &npcs, NOW);
    // The results hold ends (close enabled) and the client closes the match (39/6).
    gateway
        .tick_at(NOW + FINISH_AT + RESULTS_HOLD_MS + 1_000)
        .unwrap();
    let mut quit = vec![39, 0, 6];
    for word in [instance, -1, -1] {
        quit.extend(word.to_le_bytes());
    }
    gateway_send(&mut gateway, &quit, NOW + FINISH_AT + RESULTS_HOLD_MS + 1_100);
    assert!(!gateway.gameplay().unwrap().derby_active());
    assert_ne!(
        zone(&gateway),
        "sg_dm_arena",
        "the end of the match leaves the arena"
    );
    assert_eq!(
        gateway.state().position,
        world,
        "back at the pose kept at entry"
    );
}
