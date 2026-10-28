//! Type8 Mining18 gateway integration acceptance (dev-tools entry, private fixtures).
//!
//! Every fixture copies a private lab content directory into a temp directory and adds a synthetic
//! `mining.json`; no shared content, user save, launcher, client or release file is written.

use osfr_tools::{
    character_store::CharacterState,
    encounter::EncounterEngine,
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    mining::{
        FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, SIDE, Type8BoardSpec, Type8Engine,
        Type8Goal, Type8Kind, Type8Output,
    },
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const BOTTOM: usize = TOP + PLAYABLE - 1;
const ORE: u16 = 601;
const ORE_TWO: u16 = 602;
/// Item definition 2739 (mined ore) exists in the private campaign inventory manifest.
const ORE_DEFINITION: i32 = 2739;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn base_content() -> PathBuf {
    std::env::var_os("OSFR_TYPE8_CONTENT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/opening-journal-text-v2/content")
        })
}

fn reference_dir() -> PathBuf {
    std::env::var_os("OSFR_GATEWAY_REFERENCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../lab/references/rust-gateway-reference")
        })
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn empty_rows() -> [[i32; PLAYABLE]; PLAYABLE] {
    [[1; PLAYABLE]; PLAYABLE]
}

fn spec(
    rows: [[i32; PLAYABLE]; PLAYABLE],
    goals: Vec<Type8Goal>,
    outputs: Vec<Type8Output>,
) -> Type8BoardSpec {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, value) in row.iter().enumerate() {
            shape[(TOP + dy) * SIDE + LEFT + dx] = *value;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![401, 402, 403, 404],
        collectable: vec![ORE, ORE_TWO],
        tiles: vec![
            Type8Kind {
                value: 401,
                category: 400,
                icon_id: 1,
            },
            Type8Kind {
                value: ORE,
                category: 600,
                icon_id: 909,
            },
        ],
        goals,
        outputs,
        duration_seconds: 180,
        seed: MINING_GAME_ID as u32,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice:
            "Synthetic private gateway fixture; original Mining18 level tables unrecovered.".into(),
    }
}

fn winning_spec() -> Type8BoardSpec {
    let mut rows = empty_rows();
    rows[PLAYABLE - 1][0] = -i32::from(ORE);
    spec(
        rows,
        vec![Type8Goal {
            index: 0,
            value: ORE,
            quota: 1,
            per_reward: 1,
            icon_id: 909,
        }],
        vec![Type8Output {
            value: ORE,
            definition_id: ORE_DEFINITION,
            cluster_quantity: 1,
        }],
    )
}

struct Fixture {
    directory: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    fn new(board: &Type8BoardSpec) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-type8-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        let fixture = Self {
            directory,
            reference: reference_dir(),
        };
        fixture.write_mining_json(&json!({
            "schema_version": 1,
            "game_id": MINING_GAME_ID,
            "profile_id": MINER_PROFILE_ID,
            "name_id": 7910,
            "description_id": 7911,
            "icon_id": 909,
            "spec": board,
        }));
        fixture
    }

    fn write_mining_json(&self, value: &Value) {
        fs::write(
            self.directory.join("mining.json"),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
    }

    fn miner_state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = MINER_PROFILE_ID;
        profiles.unlocked.insert(MINER_PROFILE_ID);
        state
    }

    fn gateway(&self, state: GameplayState, dev_tools: bool) -> Gateway {
        let character = Gateway::reference_state(&self.reference).unwrap();
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.directory, Some(state))
                .unwrap();
        gateway.set_dev_tools(dev_tools);
        gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
        login(&mut gateway);
        gateway
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn wrap(inner: &[u8]) -> Vec<u8> {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    b
}

/// Accepts both tunneled gateway replies and direct inner packets.
fn inner(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn chat_inner(text: &str) -> Vec<u8> {
    let mut p = vec![15, 0, 1, 0, 0, 0];
    p.extend(1u64.to_le_bytes());
    p.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        p.extend([0; 12]);
        p.extend(0u32.to_le_bytes());
        p.extend(0u32.to_le_bytes());
    }
    p.extend((text.len() as u32).to_le_bytes());
    p.extend(text.as_bytes());
    p.extend([0; 28]);
    p
}

fn login(gateway: &mut Gateway) {
    let mut b = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""]
        .iter()
        .enumerate()
    {
        b.extend((s.len() as u32).to_le_bytes());
        b.extend(s.as_bytes());
        if i == 0 {
            b.extend(1u64.to_le_bytes());
        }
    }
    gateway.handle_at(&b, 0).unwrap();
    send(gateway, &13u16.to_le_bytes(), 1_000);
    send(gateway, &10u16.to_le_bytes(), 1_000);
}

fn send(gateway: &mut Gateway, packet: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(packet), now_ms)
        .expect("gateway frame")
}

fn say(gateway: &mut Gateway, text: &str, now_ms: u64) -> GatewayOutcome {
    send(gateway, &chat_inner(text), now_ms)
}

fn chat_text(outcome: &GatewayOutcome) -> String {
    let mut text = String::new();
    for reply in &outcome.replies {
        let packet = inner(reply);
        if packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8 {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            text.push_str(std::str::from_utf8(&packet[8..8 + length]).unwrap());
            text.push('\n');
        }
    }
    text
}

fn has_inner(outcome: &GatewayOutcome, prefix: &[u8]) -> bool {
    outcome
        .replies
        .iter()
        .map(|reply| inner(reply))
        .any(|packet| packet.starts_with(prefix))
}

fn payload_texts(outcome: &GatewayOutcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter_map(|reply| {
            let packet = inner(reply);
            if packet.len() < 19 || packet[0..3] != [39, 0, 14] {
                return None;
            }
            let length = i32::from_le_bytes(packet[15..19].try_into().unwrap());
            if length < 1 {
                return None;
            }
            let end = 19 + length as usize - 1;
            if packet.len() != end + 1 || packet[end] != 0 {
                return None;
            }
            String::from_utf8(packet[19..end].to_vec()).ok()
        })
        .collect()
}

fn has_text(outcome: &GatewayOutcome, prefix: &str) -> bool {
    payload_texts(outcome)
        .iter()
        .any(|text| text == prefix || text.starts_with(&format!("{prefix}\t")))
}

/// `[u16 39][sub][i32 instance][i32 -1][i32 game]` client frame.
fn frame(sub: u8, instance: i32, game: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [instance, -1, game] {
        b.extend(n.to_le_bytes());
    }
    b
}

fn payload(instance: i32, text: &str) -> Vec<u8> {
    let mut b = frame(14, instance, -1);
    b.extend(((text.len() + 1) as i32).to_le_bytes());
    b.extend(text.as_bytes());
    b.push(0);
    b
}

fn gameplay_json(gateway: &Gateway) -> Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn mining_json(gateway: &Gateway) -> Value {
    gameplay_json(gateway)["mining"].clone()
}

fn active_instance(gateway: &Gateway) -> Option<i64> {
    mining_json(gateway)["active"]["instance_id"].as_i64()
}

fn count(gateway: &Gateway, definition: i32) -> u32 {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .inventory
        .items
        .iter()
        .filter(|item| item.definition_id == definition && item.rental.is_none())
        .map(|item| item.count)
        .sum()
}

/// Drives a fresh run to `Playing` through the gateway frame path.
fn play_handshake(gateway: &mut Gateway, instance: i32, now_ms: u64) {
    send(gateway, &frame(5, instance, -1), now_ms);
    send(gateway, &payload(instance, "OnConnectMsg"), now_ms + 100);
    send(gateway, &payload(instance, "OnStartGameMsg"), now_ms + 200);
    send(gateway, &payload(instance, "OnStartLevelMsg"), now_ms + 300);
    assert_eq!(
        mining_json(gateway)["active"]["phase"].as_str(),
        Some("Playing")
    );
}

fn fall_bottom_ore(gateway: &mut Gateway, instance: i32, now_ms: u64) -> GatewayOutcome {
    send(
        gateway,
        &payload(
            instance,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT, BOTTOM),
        ),
        now_ms,
    )
}

#[test]
fn older_snapshots_default_absent_mining_state() {
    let old = json!({"inventory":{"coins":0,"items":[],"equipment":[]},"quests":{"version":1,"quests":{},"trial_best_ms":null,"trial_completions":0},"combat":{"enemy_health":100,"defeats":0}});
    let state: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(state.mining.is_none());
    assert_eq!(serde_json::to_value(state).unwrap(), old);
}

#[test]
fn disabled_dev_tools_refuses_mining_and_changes_nothing() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, false);

    let before = gateway.snapshot();
    let outcome = say(&mut gateway, "realm mine", 2_000);
    assert!(
        chat_text(&outcome).contains("Developer mining is disabled"),
        "disabled dev tools must refuse the command explicitly"
    );
    assert!(active_instance(&gateway).is_none());
    assert_eq!(
        gateway.snapshot(),
        before,
        "a refusal must not mutate state"
    );
}

#[test]
fn wrong_or_locked_miner_profile_is_refused() {
    let fixture = Fixture::new(&winning_spec());

    let mut wrong = fixture.miner_state();
    wrong.profiles.as_mut().unwrap().active = 1;
    let mut gateway = fixture.gateway(wrong, true);
    let outcome = say(&mut gateway, "realm mine", 2_000);
    assert!(
        chat_text(&outcome).contains("Switch to the Miner job"),
        "only the accepted Miner job may begin the run"
    );
    assert!(active_instance(&gateway).is_none());

    let mut locked = fixture.miner_state();
    locked.profiles.as_mut().unwrap().active = 1;
    locked
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .remove(&MINER_PROFILE_ID);
    let mut gateway = fixture.gateway(locked, true);
    let outcome = say(&mut gateway, "realm mine", 2_000);
    assert!(
        chat_text(&outcome).contains("not unlocked"),
        "an unlocked Miner is required even with dev tools on"
    );
    assert!(active_instance(&gateway).is_none());
}

#[test]
fn developer_gather_refuses_while_an_encounter_is_active() {
    let fixture = Fixture::new(&winning_spec());
    let state = encounter_state(&fixture);
    let mut gateway = gateway_in_encounter(&fixture, state);
    let before = gateway.snapshot();
    let outcome = say(&mut gateway, "realm mine", 2_000);
    assert_eq!(
        chat_text(&outcome).trim_end(),
        "Finish or exit the active activity before starting the mining test."
    );
    assert!(active_instance(&gateway).is_none());
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn developer_gather_refuses_when_mining_content_is_absent() {
    let fixture = Fixture::new(&winning_spec());
    fs::remove_file(fixture.directory.join("mining.json")).unwrap();
    let mut gateway = fixture.gateway(fixture.miner_state(), true);
    let before = gateway.snapshot();
    let outcome = say(&mut gateway, "realm mine", 2_000);
    assert_eq!(
        chat_text(&outcome).trim_end(),
        "Mining content is not loaded in this campaign."
    );
    assert!(active_instance(&gateway).is_none());
    assert_eq!(gateway.snapshot(), before);
}

#[test]
fn the_reachable_gather_strings_are_byte_identical() {
    let fixture = Fixture::new(&winning_spec());

    // Content absent.
    let missing = Fixture::new(&winning_spec());
    fs::remove_file(missing.directory.join("mining.json")).unwrap();
    let mut gateway = missing.gateway(missing.miner_state(), true);
    assert_eq!(
        chat_text(&say(&mut gateway, "realm mine", 2_000)).trim_end(),
        "Mining content is not loaded in this campaign."
    );

    // Finish-first: one live encounter.
    let state = encounter_state(&fixture);
    let mut gateway = gateway_in_encounter(&fixture, state);
    assert_eq!(
        chat_text(&say(&mut gateway, "realm mine", 2_000)).trim_end(),
        "Finish or exit the active activity before starting the mining test."
    );

    // The job is not unlocked.
    let mut locked = fixture.miner_state();
    locked.profiles.as_mut().unwrap().active = 1;
    locked
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .remove(&MINER_PROFILE_ID);
    let mut gateway = fixture.gateway(locked, true);
    assert_eq!(
        chat_text(&say(&mut gateway, "realm mine", 2_000)).trim_end(),
        "The Miner job is not unlocked in this save."
    );

    // Unlocked but not the active job.
    let mut wrong = fixture.miner_state();
    wrong.profiles.as_mut().unwrap().active = 1;
    let mut gateway = fixture.gateway(wrong, true);
    assert_eq!(
        chat_text(&say(&mut gateway, "realm mine", 2_000)).trim_end(),
        "Switch to the Miner job before starting the mining test."
    );

    // Started.
    let mut gateway = fixture.gateway(fixture.miner_state(), true);
    assert_eq!(
        chat_text(&say(&mut gateway, "realm mine", 2_000)).trim_end(),
        "Mining test started; type realm exit to cancel."
    );
}

/// One live dungeon encounter built from the fixture's own `encounters.json`.
fn encounter_state(fixture: &Fixture) -> GameplayState {
    let mut state = fixture.miner_state();
    let mut encounter = EncounterEngine::load(&fixture.directory, None).unwrap();
    encounter
        .begin(
            411,
            CharacterState {
                guid: 1,
                zone_id: 0,
                position: [9.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 1.0],
            },
            MINER_PROFILE_ID,
        )
        .expect("fixture encounter begins");
    state.encounter = Some(encounter.state().clone());
    state
}

/// A dev-tools gateway whose character stands in the arena zone of the active encounter.
fn gateway_in_encounter(fixture: &Fixture, state: GameplayState) -> Gateway {
    let mut character = Gateway::reference_state(&fixture.reference).unwrap();
    character.zone_id = 411;
    character.position = [0.0, 0.0, 0.0];
    let mut gateway = Gateway::load_gameplay(
        &fixture.reference,
        character,
        &fixture.directory,
        Some(state),
    )
    .unwrap();
    gateway.set_dev_tools(true);
    gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
    login(&mut gateway);
    gateway
}

#[test]
fn type8_frames_route_only_to_the_active_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);

    // No active run: a TYPE8 payload is inert.
    let before = gateway.snapshot();
    let idle = send(
        &mut gateway,
        &payload(FIRST_MINING_INSTANCE, "OnConnectMsg"),
        2_000,
    );
    assert!(idle.replies.is_empty());
    assert!(!idle.gameplay_changed);
    assert_eq!(gateway.snapshot(), before);

    // `realm mine` begins the run in its own instance namespace.
    let started = say(&mut gateway, "realm mine", 2_100);
    assert!(chat_text(&started).contains("Mining test started"));
    assert!(has_inner(&started, &[39, 0, 16]), "39/16 info packet");
    let instance = active_instance(&gateway).expect("active run");
    assert_eq!(instance, i64::from(FIRST_MINING_INSTANCE));

    // A second begin is refused while the run is live.
    let again = say(&mut gateway, "realm mine", 2_150);
    assert!(chat_text(&again).contains("Close the mining game"));
    assert_eq!(active_instance(&gateway), Some(instance));

    // Client frames reach the active engine only.
    let launched = send(&mut gateway, &frame(5, instance as i32, -1), 2_200);
    assert!(has_inner(&launched, &[39, 0, 17]));
    assert!(has_inner(&launched, &[26, 0, 12]));
    let connected = send(
        &mut gateway,
        &payload(instance as i32, "OnConnectMsg"),
        2_300,
    );
    assert!(has_text(&connected, "OnServerReadyMsg"));

    // A stale instance id is ignored and does not advance the run.
    let before_stale = gateway.snapshot();
    let stale = send(
        &mut gateway,
        &payload(instance as i32 + 1, "OnStartGameMsg"),
        2_400,
    );
    assert!(stale.replies.is_empty());
    assert!(!stale.gameplay_changed);
    assert_eq!(gateway.snapshot(), before_stale);
}

#[test]
fn active_mining_blocks_unrelated_actions_like_crafting() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm mine", 2_000);
    let instance = active_instance(&gateway).unwrap() as i32;

    let class_select = {
        let mut b = vec![26, 0, 13, 0];
        b.extend(2_i32.to_le_bytes());
        b
    };
    let interact = {
        let mut b = vec![26, 0, 8, 0];
        b.extend(1_u64.to_le_bytes());
        b.push(1);
        b
    };
    let interact_select = {
        let mut b = vec![26, 0, 10, 0];
        b.extend(1_u64.to_le_bytes());
        b.extend(1_i32.to_le_bytes());
        b
    };
    let equip = {
        let mut b = vec![36, 0, 10, 0];
        b.extend(1_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b
    };
    let inventory = vec![42, 0, 3, 0];

    for blocked in [class_select, interact, interact_select, equip, inventory] {
        let before = gateway.snapshot();
        let outcome = send(&mut gateway, &blocked, 2_100);
        assert!(
            chat_text(&outcome).contains("Close the mining game"),
            "frame {blocked:?} must be blocked while mining is active"
        );
        assert_eq!(gateway.snapshot(), before);
    }

    // Developer teleport stays blocked by the active-movie chat guard.
    let before = gateway.snapshot();
    let teleport = say(&mut gateway, "realm goto mae farnum", 2_200);
    assert!(chat_text(&teleport).contains("Close the mining game"));
    assert_eq!(gateway.snapshot(), before);

    // The run is still the same live instance after the refusals.
    assert_eq!(active_instance(&gateway), Some(i64::from(instance)));
}

#[test]
fn save_reconnect_and_cancel_paths_release_the_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm mine", 2_000);
    let instance = active_instance(&gateway).unwrap();

    // (R) Save/reconnect releases the live run: the new client has no movie to resume.
    let snapshot = gateway.snapshot();
    let path = fixture.directory.join("type8-save.json");
    game_store::save(&path, &snapshot).unwrap();
    let loaded = game_store::load_or_create(&path, snapshot.character.clone()).unwrap();
    let mut restored = Gateway::load_gameplay(
        &fixture.reference,
        loaded.character,
        &fixture.directory,
        loaded.gameplay,
    )
    .unwrap();
    restored.set_dev_tools(true);
    restored.set_role(osfr_tools::account_store::AccountRole::Admin);
    login(&mut restored);
    assert!(active_instance(&restored).is_none());
    assert_eq!(count(&restored, ORE_DEFINITION), 0);

    // A stale results Close from the old movie changes nothing.
    let left = send(&mut restored, &[26, 0, 42, 0], 3_000);
    assert!(left.replies.is_empty());

    let after = restored.snapshot();
    let replay = send(&mut restored, &[26, 0, 42, 0], 3_100);
    assert!(replay.replies.is_empty());
    assert_eq!(restored.snapshot(), after);
    let stale = send(
        &mut restored,
        &payload(instance as i32, "OnConnectMsg"),
        3_200,
    );
    assert!(stale.replies.is_empty());
    assert_eq!(restored.snapshot(), after);

    // `realm exit` cancels a fresh run through the chat guard.
    say(&mut restored, "realm mine", 4_000);
    assert!(active_instance(&restored).is_some());
    let exit = say(&mut restored, "realm exit", 4_100);
    assert!(has_inner(&exit, &[39, 0, 19]));
    assert!(active_instance(&restored).is_none());
    assert_eq!(count(&restored, ORE_DEFINITION), 0);
}

#[test]
fn authoritative_loss_results_close_once_and_never_grant() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm mine", 2_000);
    let instance = active_instance(&gateway).unwrap() as i32;
    play_handshake(&mut gateway, instance, 2_100);

    // The server timer runs the level out; the client cannot report the loss.
    let ticked = gateway.tick_at(182_500).unwrap();
    assert!(ticked.state_changed);
    assert_eq!(
        mining_json(&gateway)["active"]["phase"].as_str(),
        Some("Lost")
    );

    let results = send(
        &mut gateway,
        &payload(instance, "FRServer_GameOver"),
        182_600,
    );
    assert!(has_inner(&results, &[39, 0, 18]));
    assert!(has_inner(&results, &[39, 0, 47]));
    assert!(has_inner(&results, &[39, 0, 19]));
    assert_eq!(count(&gateway, ORE_DEFINITION), 0, "a loss never grants");

    let replay = send(
        &mut gateway,
        &payload(instance, "FRServer_GameOver"),
        182_700,
    );
    assert!(!has_inner(&replay, &[39, 0, 18]));
    assert!(!has_inner(&replay, &[39, 0, 47]));

    send(
        &mut gateway,
        &payload(instance, "FRServer_GameClose"),
        182_800,
    );
    assert!(active_instance(&gateway).is_none());
    assert_eq!(count(&gateway, ORE_DEFINITION), 0);
}

#[test]
fn repeat_win_grants_the_configured_bundle_exactly_once_per_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm mine", 2_000);
    let first = active_instance(&gateway).unwrap() as i32;
    play_handshake(&mut gateway, first, 2_100);

    // Client score reports are never authoritative and never grant.
    let forged = send(
        &mut gateway,
        &payload(first, "FRServer_ScoreSet\t999999"),
        2_500,
    );
    assert!(!forged.gameplay_changed);
    assert_eq!(count(&gateway, ORE_DEFINITION), 0);

    let fall = fall_bottom_ore(&mut gateway, first, 2_600);
    assert!(has_text(&fall, "OnStopLevel"));
    assert_eq!(
        count(&gateway, ORE_DEFINITION),
        0,
        "no grant before the native next-level request"
    );

    let win = send(&mut gateway, &payload(first, "OnNextLevelMsg"), 2_700);
    assert!(has_text(&win, "OnGameWonMsg"));
    assert!(win.gameplay_changed);
    assert_eq!(count(&gateway, ORE_DEFINITION), 1);
    assert_eq!(mining_json(&gateway)["completed"].as_u64(), Some(1));

    let replay = send(&mut gateway, &payload(first, "OnNextLevelMsg"), 2_800);
    assert!(!replay.gameplay_changed);
    assert_eq!(
        count(&gateway, ORE_DEFINITION),
        1,
        "a replayed win request must not grant twice"
    );

    // A second run gets a fresh instance in the same namespace and grants once more.
    send(&mut gateway, &payload(first, "FRServer_GameClose"), 2_900);
    assert!(active_instance(&gateway).is_none());
    say(&mut gateway, "realm mine", 3_000);
    let second = active_instance(&gateway).unwrap() as i32;
    assert_eq!(second, first + 1, "unique instance namespace");
    play_handshake(&mut gateway, second, 3_100);
    fall_bottom_ore(&mut gateway, second, 3_600);
    send(&mut gateway, &payload(second, "OnNextLevelMsg"), 3_700);
    assert_eq!(count(&gateway, ORE_DEFINITION), 2);
}

#[test]
fn mining_content_loads_fail_closed_and_skips_an_absent_file() {
    let fixture = Fixture::new(&winning_spec());

    // Wrong game binding.
    fixture.write_mining_json(&json!({
        "schema_version": 1,
        "game_id": 91,
        "profile_id": MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": winning_spec(),
    }));
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Wrong job binding: the gateway must not accept a non-Miner content file.
    fixture.write_mining_json(&json!({
        "schema_version": 1,
        "game_id": MINING_GAME_ID,
        "profile_id": 1,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": winning_spec(),
    }));
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Orphaned output: the engine refuses the spec at load instead of dropping grants.
    let mut orphan = winning_spec();
    orphan.outputs = vec![Type8Output {
        value: ORE_TWO,
        definition_id: ORE_DEFINITION,
        cluster_quantity: 1,
    }];
    fixture.write_mining_json(&json!({
        "schema_version": 1,
        "game_id": MINING_GAME_ID,
        "profile_id": MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": orphan,
    }));
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Restore the valid bound file for the saved-run check below.
    fixture.write_mining_json(&json!({
        "schema_version": 1,
        "game_id": MINING_GAME_ID,
        "profile_id": MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": winning_spec(),
    }));

    // The valid, present file above stays fail-closed (every check before it). An ABSENT file
    // is a skip (T852): the module stays inactive, one event names it, and the saved run is
    // written back unchanged instead of the login failing.
    let state = fixture.miner_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm mine", 2_000);
    let snapshot = gateway.snapshot();
    assert!(active_instance(&gateway).is_some());
    let saved = snapshot.gameplay.expect("the run keeps gameplay");
    fs::remove_file(fixture.directory.join("mining.json")).unwrap();
    let loaded = Gateway::load_gameplay(
        &fixture.reference,
        snapshot.character,
        &fixture.directory,
        Some(saved.clone()),
    )
    .expect("a missing mining file is a skip, not a failed load");
    let records = loaded
        .gameplay()
        .expect("gameplay binds without mining")
        .skipped_modules();
    assert!(
        records.iter().any(|record| {
            record["event"] == "gameplay_module_skipped"
                && record["module"] == "mining"
                && record["missing"] == "mining.json"
        }),
        "the skip is recorded: {records:?}"
    );
    assert_eq!(
        loaded.snapshot().gameplay.expect("the save stays").mining,
        saved.mining,
        "the skipped module's saved run is kept"
    );
}

#[test]
fn mining_write_content_helper_binds_game_and_job() {
    let directory = std::env::temp_dir().join(format!(
        "osfr-type8-write-content-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();

    let path = osfr_tools::mining::write_content(&directory, MINING_GAME_ID).unwrap();
    assert_eq!(path.file_name().unwrap(), "mining.json");
    let engine = Type8Engine::load(
        &directory,
        "mining.json",
        MINING_GAME_ID,
        FIRST_MINING_INSTANCE,
        None,
    )
    .unwrap();
    assert_eq!(engine.profile_id(), MINER_PROFILE_ID);
    assert_eq!(engine.preload_file(), "MiningAssets.txt");
    assert_eq!(engine.launch_movie(), "game_mining.gfx");
    assert!(engine.spec().reconstructed_rules);
    assert!(
        osfr_tools::mining::write_content(&directory, 91).is_err(),
        "unrecovered TYPE8 variants stay unbound"
    );

    // Optional handoff dump: `OSFR_TYPE8_DUMP_CONTENT` captures the exact bytes the
    // registered `write_content` produces so the lead can drop them into a private
    // native content directory without compiling a separate prep harness.
    if let Some(dump) = std::env::var_os("OSFR_TYPE8_DUMP_CONTENT") {
        fs::copy(&path, PathBuf::from(dump)).unwrap();
    }

    let _ = fs::remove_dir_all(&directory);
}
