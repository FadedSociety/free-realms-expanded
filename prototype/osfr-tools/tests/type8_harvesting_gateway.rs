//! Type8 Harvesting111 gateway integration acceptance (dev-tools entry, private fixtures).
//!
//! Every fixture copies a private lab content directory into a temp directory and adds a
//! synthetic `harvesting.json`; no shared content, user save, launcher, client or release
//! file is written. Harvesting stays an unregistered opt-in candidate for Chef45.

use osfr_tools::{
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    harvesting::{
        self, CHEF_PROFILE_ID, CONTENT_FILE, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE,
        FIRST_INSTANCE_ID,
    },
    mining::{
        self, Type8BoardSpec, Type8Engine, Type8Goal, Type8Kind, Type8Output,
        FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, SIDE,
    },
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const HARVESTING_GAME_ID: i32 = 111;
const LEFT: usize = 3;
const TOP: usize = 3;
const PLAYABLE: usize = 8;
const BOTTOM: usize = TOP + PLAYABLE - 1;
/// Region 111's recovered bundle definitions in locale order.
const BUNDLES: [i32; 4] = [2885, 2887, 2890, 2877];
/// Manifest-defined stand-ins used to exercise the configured transaction: the
/// private campaign manifest does not yet define the four recovered region-111
/// bundle definitions, so a live grant of them fails "unknown reward
/// definition" until that content lands. The shipped config still binds BUNDLES.
const FIXTURE_OUTPUTS: [i32; 4] = [2739, 879, 65834, 2436];
const VALUES: [u16; 4] = [601, 602, 603, 604];
/// An unrelated stackable definition present in the private campaign manifest.
const FILLER: i32 = 1195;
const PRODUCTION_MAX_ITEMS: usize = 4096;

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
        collectable: VALUES.to_vec(),
        tiles: [401, 402, 403, 404]
            .iter()
            .enumerate()
            .map(|(index, value)| Type8Kind {
                value: *value,
                category: 400,
                icon_id: index as i32 + 1,
            })
            .chain(VALUES.iter().map(|value| Type8Kind {
                value: *value,
                category: 600,
                icon_id: 700 + i32::from(*value),
            }))
            .collect(),
        goals,
        outputs,
        duration_seconds: 180,
        seed: HARVESTING_GAME_ID as u32,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice:
            "Synthetic private gateway fixture; original Harvesting111 level tables unrecovered."
                .into(),
    }
}

/// Four recoverable goals, one per column of the bottom playable row.
fn winning_spec() -> Type8BoardSpec {
    let mut rows = empty_rows();
    for (slot, value) in VALUES.iter().enumerate() {
        rows[PLAYABLE - 1][slot] = -i32::from(*value);
    }
    spec(
        rows,
        VALUES
            .iter()
            .enumerate()
            .map(|(slot, value)| Type8Goal {
                index: slot as i32,
                value: *value,
                quota: 1,
                per_reward: 1,
                icon_id: 700 + i32::from(*value),
            })
            .collect(),
        VALUES
            .iter()
            .enumerate()
            .map(|(slot, value)| Type8Output {
                value: *value,
                definition_id: FIXTURE_OUTPUTS[slot],
                cluster_quantity: 1,
            })
            .collect(),
    )
}

/// Minimal Mining18 spec so cross-game exclusivity can be exercised.
fn mining_spec() -> Type8BoardSpec {
    let mut rows = empty_rows();
    rows[PLAYABLE - 1][0] = -601;
    spec(
        rows,
        vec![Type8Goal {
            index: 0,
            value: 601,
            quota: 1,
            per_reward: 1,
            icon_id: 909,
        }],
        vec![Type8Output {
            value: 601,
            definition_id: 2739,
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
            "osfr-harvesting-gateway-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        let fixture = Self {
            directory,
            reference: reference_dir(),
        };
        fixture.write_harvesting(&json!({
            "schema_version": 1,
            "game_id": HARVESTING_GAME_ID,
            "profile_id": CHEF_PROFILE_ID,
            "name_id": 100834,
            "description_id": 94150,
            "icon_id": 908,
            "preload_file": FARMING_PRELOAD_FILE,
            "launch_movie": FARMING_LAUNCH_MOVIE,
            "spec": board,
        }));
        fixture
    }

    fn write_harvesting(&self, value: &Value) {
        fs::write(
            self.directory.join(CONTENT_FILE),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
    }

    fn write_mining(&self, value: &Value) {
        fs::write(
            self.directory.join(mining::CONTENT_FILE),
            serde_json::to_vec(value).unwrap(),
        )
        .unwrap();
    }

    fn bound_mining_json(&self) -> Value {
        json!({
            "schema_version": 1,
            "game_id": MINING_GAME_ID,
            "profile_id": MINER_PROFILE_ID,
            "name_id": 7910,
            "description_id": 7911,
            "icon_id": 909,
            "spec": mining_spec(),
        })
    }

    fn chef_state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = CHEF_PROFILE_ID;
        profiles.unlocked.insert(CHEF_PROFILE_ID);
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

fn harvesting_json(gateway: &Gateway) -> Value {
    gameplay_json(gateway)["harvesting"].clone()
}

fn active_instance(gateway: &Gateway) -> Option<i64> {
    harvesting_json(gateway)["active"]["instance_id"].as_i64()
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
        harvesting_json(gateway)["active"]["phase"].as_str(),
        Some("Playing")
    );
}

fn fall(gateway: &mut Gateway, instance: i32, slot: usize, now_ms: u64) -> GatewayOutcome {
    send(
        gateway,
        &payload(
            instance,
            &format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM),
        ),
        now_ms,
    )
}

fn fill_inventory(state: &mut GameplayState) {
    let items = &mut state.inventory.items;
    let mut next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    while items.len() < PRODUCTION_MAX_ITEMS {
        items.push(osfr_tools::inventory::InventoryItem {
            id: next,
            definition_id: FILLER,
            tint: 0,
            count: 1,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: i64::from(i32::MAX),
            rental: None,
            ability_count: 0,
            activate_enabled: false,
        });
        next += 1;
    }
    state.inventory.next_instance_id = Some(i64::from(next));
}

#[test]
fn older_snapshots_default_absent_harvesting_state() {
    let old = json!({"inventory":{"coins":0,"items":[],"equipment":[]},"quests":{"version":1,"quests":{},"trial_best_ms":null,"trial_completions":0},"combat":{"enemy_health":100,"defeats":0}});
    let state: GameplayState = serde_json::from_value(old.clone()).unwrap();
    assert!(state.harvesting.is_none());
    assert_eq!(
        serde_json::to_value(state).unwrap(),
        old,
        "a save without harvesting stays byte-identical"
    );
}

#[test]
fn disabled_dev_tools_refuses_harvesting_and_changes_nothing() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, false);

    let before = gateway.snapshot();
    let outcome = say(&mut gateway, "realm harvest", 2_000);
    assert!(
        chat_text(&outcome).contains("Developer harvesting is disabled"),
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
fn wrong_or_locked_chef_profile_is_refused() {
    let fixture = Fixture::new(&winning_spec());

    let mut wrong = fixture.chef_state();
    wrong.profiles.as_mut().unwrap().active = 1;
    let mut gateway = fixture.gateway(wrong, true);
    let outcome = say(&mut gateway, "realm harvest", 2_000);
    assert!(
        chat_text(&outcome).contains("Switch to the Chef job"),
        "only the accepted Chef job may begin the run"
    );
    assert!(active_instance(&gateway).is_none());

    let mut locked = fixture.chef_state();
    locked.profiles.as_mut().unwrap().active = 1;
    locked
        .profiles
        .as_mut()
        .unwrap()
        .unlocked
        .remove(&CHEF_PROFILE_ID);
    let mut gateway = fixture.gateway(locked, true);
    let outcome = say(&mut gateway, "realm harvest", 2_000);
    assert!(
        chat_text(&outcome).contains("not unlocked"),
        "an unlocked Chef is required even with dev tools on"
    );
    assert!(active_instance(&gateway).is_none());
}

#[test]
fn harvesting_frames_bind_region_111_and_route_only_to_the_active_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);

    // No active run: a TYPE8 payload is inert.
    let before = gateway.snapshot();
    let idle = send(
        &mut gateway,
        &payload(FIRST_INSTANCE_ID, "OnConnectMsg"),
        2_000,
    );
    assert!(idle.replies.is_empty());
    assert!(!idle.gameplay_changed);
    assert_eq!(gateway.snapshot(), before);

    // `realm harvest` begins the run in the harvesting instance namespace.
    let started = say(&mut gateway, "realm harvest", 2_100);
    assert!(chat_text(&started).contains("Harvesting test started"));
    let info = started
        .replies
        .iter()
        .map(|reply| inner(reply))
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("39/16 info packet")
        .to_vec();
    for expected in [
        100834_i32.to_le_bytes(),
        908_i32.to_le_bytes(),
        94150_i32.to_le_bytes(),
        CHEF_PROFILE_ID.to_le_bytes(),
    ] {
        assert!(
            info.windows(4).any(|window| window == expected),
            "39/16 must carry the region 111 identity and Chef profile"
        );
    }
    assert!(
        info.windows(FARMING_PRELOAD_FILE.len())
            .any(|window| window == FARMING_PRELOAD_FILE.as_bytes()),
        "39/16 must advertise the recovered farming preload list"
    );
    let instance = active_instance(&gateway).expect("active run");
    assert_eq!(instance, i64::from(FIRST_INSTANCE_ID));

    // A second begin is refused while the run is live.
    let again = say(&mut gateway, "realm harvest", 2_150);
    assert!(chat_text(&again).contains("Close the harvesting game"));
    assert_eq!(active_instance(&gateway), Some(instance));

    // The launcher requests the original farming movie.
    let launched = send(&mut gateway, &frame(5, instance as i32, -1), 2_200);
    assert!(has_inner(&launched, &[39, 0, 17]));
    let launch = launched
        .replies
        .iter()
        .map(|reply| inner(reply))
        .find(|packet| packet.starts_with(&[26, 0, 12]))
        .expect("26/12 launcher")
        .to_vec();
    assert!(
        launch
            .windows(FARMING_LAUNCH_MOVIE.len())
            .any(|window| window == FARMING_LAUNCH_MOVIE.as_bytes()),
        "the launcher must request game_farming.gfx"
    );
    assert!(
        !launch
            .windows(15)
            .any(|window| window == b"game_mining.gfx"),
        "the launcher must not request the mining movie"
    );

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
fn active_harvesting_blocks_other_games_and_unrelated_actions() {
    let fixture = Fixture::new(&winning_spec());
    fixture.write_mining(&fixture.bound_mining_json());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm harvest", 2_000);
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
    let equip = {
        let mut b = vec![36, 0, 10, 0];
        b.extend(1_i32.to_le_bytes());
        b.extend(0_i32.to_le_bytes());
        b
    };
    for blocked in [class_select, interact, equip, vec![42, 0, 3, 0]] {
        let before = gateway.snapshot();
        let outcome = send(&mut gateway, &blocked, 2_100);
        assert!(
            chat_text(&outcome).contains("Close the harvesting game"),
            "frame {blocked:?} must be blocked while harvesting is active"
        );
        assert_eq!(gateway.snapshot(), before);
    }

    // Another game and a developer teleport stay blocked too.
    for command in ["realm mine", "realm goto mae farnum"] {
        let before = gateway.snapshot();
        let outcome = say(&mut gateway, command, 2_200);
        assert!(
            chat_text(&outcome).contains("Close the harvesting game"),
            "{command} must be blocked while harvesting is active"
        );
        assert_eq!(gateway.snapshot(), before);
    }
    assert_eq!(active_instance(&gateway), Some(i64::from(instance)));
}

#[test]
fn save_reconnect_cancel_and_restart_rebind_the_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm harvest", 2_000);
    assert!(active_instance(&gateway).is_some());

    let snapshot = gateway.snapshot();
    let path = fixture.directory.join("harvesting-save.json");
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
    // (R) Reconnect releases the run: the new client has no movie to resume.
    assert!(active_instance(&restored).is_none());
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&restored, definition), 0);
    }

    // A stale results Close from the old movie changes nothing.
    let left = send(&mut restored, &[26, 0, 42, 0], 3_000);
    assert!(left.replies.is_empty());
    let after = restored.snapshot();
    let replay = send(&mut restored, &[26, 0, 42, 0], 3_100);
    assert!(replay.replies.is_empty());
    assert_eq!(restored.snapshot(), after);

    // `realm exit` cancels a fresh run and grants nothing.
    say(&mut restored, "realm harvest", 4_000);
    assert!(active_instance(&restored).is_some());
    let exit = say(&mut restored, "realm exit", 4_100);
    assert!(has_inner(&exit, &[39, 0, 19]));
    assert!(active_instance(&restored).is_none());
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&restored, definition), 0);
    }
}

#[test]
fn authoritative_timer_pause_loss_and_results_close() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm harvest", 2_000);
    let instance = active_instance(&gateway).unwrap() as i32;
    play_handshake(&mut gateway, instance, 2_100);

    // Pause is a client signal the server owns: the timer must not advance.
    let pause = send(
        &mut gateway,
        &payload(instance, "FRServer_Pause\ttrue"),
        2_500,
    );
    assert!(has_text(&pause, "OnPauseMsg"));
    let remaining_before = harvesting_json(&gateway)["active"]["remaining_ms"].as_u64();
    gateway.tick_at(60_000).unwrap();
    assert_eq!(
        harvesting_json(&gateway)["active"]["remaining_ms"].as_u64(),
        remaining_before,
        "a paused run keeps its remaining time"
    );
    assert_eq!(
        harvesting_json(&gateway)["active"]["phase"].as_str(),
        Some("Playing")
    );

    let resumed = send(
        &mut gateway,
        &payload(instance, "FRServer_Pause\tfalse"),
        60_100,
    );
    assert!(has_text(&resumed, "OnPauseMsg"));

    // The server timer runs the level out; the client cannot report the loss.
    let ticked = gateway.tick_at(250_000).unwrap();
    assert!(ticked.state_changed);
    assert_eq!(
        harvesting_json(&gateway)["active"]["phase"].as_str(),
        Some("Lost")
    );
    let results = send(
        &mut gateway,
        &payload(instance, "FRServer_GameOver"),
        250_100,
    );
    assert!(has_inner(&results, &[39, 0, 18]));
    assert!(has_inner(&results, &[39, 0, 47]));
    assert!(has_inner(&results, &[39, 0, 19]));
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&gateway, definition), 0, "a loss never grants");
    }
    let replay = send(
        &mut gateway,
        &payload(instance, "FRServer_GameOver"),
        250_200,
    );
    assert!(!has_inner(&replay, &[39, 0, 18]));
    assert!(!has_inner(&replay, &[39, 0, 47]));
}

#[test]
fn quota_win_grants_the_configured_bundles_once_per_run() {
    let fixture = Fixture::new(&winning_spec());
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm harvest", 2_000);
    let first = active_instance(&gateway).unwrap() as i32;
    play_handshake(&mut gateway, first, 2_100);

    // Client score and win reports are never authoritative and never grant.
    for forged in ["FRServer_ScoreSet\t999999", "FRServer_GameWon\t999999"] {
        let outcome = send(&mut gateway, &payload(first, forged), 2_500);
        assert!(!outcome.gameplay_changed, "{forged} must be inert");
        for definition in FIXTURE_OUTPUTS {
            assert_eq!(count(&gateway, definition), 0);
        }
    }

    for (slot, definition) in FIXTURE_OUTPUTS.iter().enumerate() {
        let outcome = fall(&mut gateway, first, slot, 2_600 + slot as u64 * 10);
        assert_eq!(
            count(&gateway, *definition),
            0,
            "no grant before the native next-level request"
        );
        if slot == FIXTURE_OUTPUTS.len() - 1 {
            assert!(has_text(&outcome, "OnStopLevel"));
        }
    }

    let win = send(&mut gateway, &payload(first, "OnNextLevelMsg"), 2_700);
    assert!(has_text(&win, "OnGameWonMsg"));
    assert!(win.gameplay_changed);
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&gateway, definition), 1, "configured bundle granted");
    }
    assert_eq!(harvesting_json(&gateway)["completed"].as_u64(), Some(1));

    let replay = send(&mut gateway, &payload(first, "OnNextLevelMsg"), 2_800);
    assert!(!replay.gameplay_changed);
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(
            count(&gateway, definition),
            1,
            "replay must not grant twice"
        );
    }

    // A second run gets a fresh instance in the same namespace and grants once more.
    send(&mut gateway, &payload(first, "FRServer_GameClose"), 2_900);
    assert!(active_instance(&gateway).is_none());
    say(&mut gateway, "realm harvest", 3_000);
    let second = active_instance(&gateway).unwrap() as i32;
    assert_eq!(second, first + 1, "unique instance namespace");
    play_handshake(&mut gateway, second, 3_100);
    for slot in 0..FIXTURE_OUTPUTS.len() {
        fall(&mut gateway, second, slot, 3_600 + slot as u64 * 10);
    }
    send(&mut gateway, &payload(second, "OnNextLevelMsg"), 3_700);
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&gateway, definition), 2);
    }
}

#[test]
fn full_bag_rolls_back_the_win_and_retry_grants_after_space() {
    let fixture = Fixture::new(&winning_spec());
    let mut state = fixture.chef_state();
    fill_inventory(&mut state);
    let mut gateway = fixture.gateway(state, true);
    gateway.set_trace_enabled(true);
    say(&mut gateway, "realm harvest", 2_000);
    let instance = active_instance(&gateway).unwrap() as i32;
    play_handshake(&mut gateway, instance, 2_100);
    for slot in 0..FIXTURE_OUTPUTS.len() {
        fall(&mut gateway, instance, slot, 2_600 + slot as u64 * 10);
    }

    // The staged grant cannot mint rows in a full bag: the frame fails and the
    // gateway keeps the pre-frame snapshot (run still LevelComplete).
    let before = gateway.snapshot();
    let blocked = gateway
        .handle_at(&wrap(&payload(instance, "OnNextLevelMsg")), 2_800)
        .expect("a gameplay rejection must drop the application and keep the session");
    assert!(
        blocked.trace.iter().any(|event| {
            event.outcome == "rejected"
                && event
                    .reason
                    .as_deref()
                    .is_some_and(|reason| reason.starts_with("inventory "))
        }),
        "trace={:?}",
        blocked.trace
    );
    assert_eq!(gateway.snapshot(), before);
    assert_eq!(
        harvesting_json(&gateway)["active"]["phase"].as_str(),
        Some("LevelComplete"),
        "the run stays retryable after a rollback"
    );
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&gateway, definition), 0);
    }

    // (R) Reconnect releases the unpaid run: no payout and no stuck state. With space freed, a
    // fresh run pays exactly once.
    let mut state = gateway.snapshot().gameplay.unwrap();
    for _ in 0..FIXTURE_OUTPUTS.len() {
        state.inventory.items.pop();
    }
    let character = gateway.snapshot().character;
    let mut reopened = Gateway::load_gameplay(
        &fixture.reference,
        character,
        &fixture.directory,
        Some(state),
    )
    .unwrap();
    reopened.set_dev_tools(true);
    reopened.set_role(osfr_tools::account_store::AccountRole::Admin);
    login(&mut reopened);
    assert!(active_instance(&reopened).is_none());
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&reopened, definition), 0);
    }
    say(&mut reopened, "realm harvest", 2_850);
    let retry = active_instance(&reopened).unwrap() as i32;
    play_handshake(&mut reopened, retry, 2_900);
    for slot in 0..FIXTURE_OUTPUTS.len() {
        fall(&mut reopened, retry, slot, 3_500 + slot as u64 * 10);
    }
    let win = send(&mut reopened, &payload(retry, "OnNextLevelMsg"), 3_600);
    assert!(has_text(&win, "OnGameWonMsg"));
    for definition in FIXTURE_OUTPUTS {
        assert_eq!(count(&reopened, definition), 1);
    }
}

#[test]
fn harvesting_content_loads_fail_closed_and_binds_region_111() {
    let fixture = Fixture::new(&winning_spec());

    // Wrong game binding.
    let mut wrong_game = fixture.bound_content(winning_spec());
    wrong_game["game_id"] = json!(109);
    fixture.write_harvesting(&wrong_game);
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Wrong job binding: only Chef45 may be linked.
    let mut wrong_job = fixture.bound_content(winning_spec());
    wrong_job["profile_id"] = json!(15);
    fixture.write_harvesting(&wrong_job);
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Wrong launch assets: the farming movie/preload are validated.
    let mut wrong_assets = fixture.bound_content(winning_spec());
    wrong_assets["launch_movie"] = json!("game_mining.gfx");
    fixture.write_harvesting(&wrong_assets);
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // Orphaned output: the engine refuses the spec at load instead of dropping grants.
    let mut orphan = winning_spec();
    orphan.outputs[0].value = 599;
    let mut orphan_content = fixture.bound_content(orphan);
    orphan_content["spec"]["outputs"][0]["value"] = json!(599);
    fixture.write_harvesting(&orphan_content);
    assert!(Gameplay::load(&fixture.directory, None).is_err());

    // The real region 111 content keeps the recovered profile, assets and outputs.
    fixture.write_harvesting(
        &serde_json::from_str(&harvesting::content_json(HARVESTING_GAME_ID).unwrap()).unwrap(),
    );
    let engine = Type8Engine::load(
        &fixture.directory,
        CONTENT_FILE,
        HARVESTING_GAME_ID,
        FIRST_INSTANCE_ID,
        None,
    )
    .unwrap();
    assert_eq!(engine.profile_id(), CHEF_PROFILE_ID);
    assert_eq!(engine.preload_file(), FARMING_PRELOAD_FILE);
    assert_eq!(engine.launch_movie(), FARMING_LAUNCH_MOVIE);
    let spec = engine.spec();
    assert_eq!(
        spec.outputs
            .iter()
            .map(|output| output.definition_id)
            .collect::<Vec<_>>(),
        BUNDLES.to_vec()
    );
    assert!(
        spec.goals
            .iter()
            .all(|goal| goal.quota == 25 && goal.per_reward == 5),
        "the reconstructed quotas stay unchanged"
    );

    // The real bundle definitions are absent from the frozen catalogue, so a
    // gateway load of this exact content is refused before any offer (see
    // tests/harvesting_reward_catalogue.rs). Restore the manifest-present
    // stand-ins for the remaining gateway assertions.
    assert!(
        Gameplay::load(&fixture.directory, None).is_err(),
        "missing reward definitions must refuse the gateway load"
    );
    fixture.write_harvesting(&fixture.bound_content(winning_spec()));

    // A saved run without its content file refuses to load.
    let state = fixture.chef_state();
    let mut gateway = fixture.gateway(state, true);
    say(&mut gateway, "realm harvest", 2_000);
    assert!(active_instance(&gateway).is_some());
    let snapshot = gateway.snapshot();
    fs::remove_file(fixture.directory.join(CONTENT_FILE)).unwrap();
    assert!(Gateway::load_gameplay(
        &fixture.reference,
        snapshot.character,
        &fixture.directory,
        snapshot.gameplay,
    )
    .is_err());
}

impl Fixture {
    fn bound_content(&self, board: Type8BoardSpec) -> Value {
        json!({
            "schema_version": 1,
            "game_id": HARVESTING_GAME_ID,
            "profile_id": CHEF_PROFILE_ID,
            "name_id": 100834,
            "description_id": 94150,
            "icon_id": 908,
            "preload_file": FARMING_PRELOAD_FILE,
            "launch_movie": FARMING_LAUNCH_MOVIE,
            "spec": board,
        })
    }
}

/// The private fixture must know which definitions exist; this guard fails
/// loudly if the campaign manifest changes.
#[test]
fn fixture_definitions_exist_in_the_private_campaign() {
    let fixture = Fixture::new(&winning_spec());
    let mut inventory = osfr_tools::inventory::Inventory::load(&fixture.directory, None).unwrap();
    for definition in FIXTURE_OUTPUTS.iter().chain([FILLER].iter()) {
        inventory
            .grant_with_tint(0, *definition, 1, 0)
            .unwrap_or_else(|error| {
                panic!("definition {definition} is absent from the private campaign: {error}")
            });
        assert!(
            inventory
                .state()
                .items
                .iter()
                .any(|item| item.definition_id == *definition),
            "definition {definition} must be mintable from the manifest"
        );
    }
    assert!(FIRST_MINING_INSTANCE > 0);
}
