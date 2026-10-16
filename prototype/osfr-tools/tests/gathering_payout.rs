//! W140: Mining and Harvesting lifecycle exit and payout, driven through the gateway frame path.
//!
//! Both games run on the shared Type8 engine, so every case runs for Miner/Mining18 and
//! Chef/Harvesting111. Fixtures copy the private lab content into a temp directory and add a
//! synthetic one-bundle-per-goal board; no shared content, save, launcher or client is touched.
//!
//! Recovered facts (client): Quit sends `39/6 [instance,-1,-1]` (`009be8e0`) and the server answers
//! with the `39/19` removal only; the results Close is the four-byte `26/42` (`0092f300`); the
//! `39/7` acknowledgement changes nothing (`009be8f0`). Server policy marked (R): a run that is
//! not Won pays nothing, the reward is the configured bundle derived from server-side quota
//! progress, and a reconnect drops an unfinished run.

use osfr_tools::{
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    harvesting::{
        CHEF_PROFILE_ID, CONTENT_FILE, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE,
        FIRST_INSTANCE_ID, lifecycle::HarvestingLifecycle,
    },
    mining::{
        self, FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, SIDE, Type8BoardSpec,
        Type8Engine, Type8Goal, Type8Kind, Type8Output, lifecycle::MiningLifecycle,
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
/// Manifest-defined stand-ins that exist in the private campaign inventory.
const OUTPUTS: [i32; 4] = [2739, 879, 65834, 2436];
const VALUES: [u16; 4] = [601, 602, 603, 604];
const HARVEST_GAME_ID: i32 = 111;

static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct Game {
    name: &'static str,
    command: &'static str,
    json_key: &'static str,
    profile: i32,
    /// Collectables (and so quota goals) on the synthetic board.
    goals: usize,
}

const MINING: Game = Game {
    name: "mining",
    command: "realm mine",
    json_key: "mining",
    profile: MINER_PROFILE_ID,
    goals: 1,
};
const HARVEST: Game = Game {
    name: "harvesting",
    command: "realm harvest",
    json_key: "harvesting",
    profile: CHEF_PROFILE_ID,
    goals: 4,
};
const GAMES: [Game; 2] = [MINING, HARVEST];
/// Run phases, in order, with the client steps that reach each one from `Offered`.
const PHASES: [&str; 6] = [
    "Offered",
    "Launched",
    "Connected",
    "LevelOffered",
    "Playing",
    "LevelComplete",
];

fn board(goals: usize) -> Type8BoardSpec {
    let values = &VALUES[..goals];
    let mut shape = vec![0_i32; SIDE * SIDE];
    for dy in 0..PLAYABLE {
        for dx in 0..PLAYABLE {
            shape[(TOP + dy) * SIDE + LEFT + dx] = 1;
        }
    }
    for (slot, value) in values.iter().enumerate() {
        shape[(TOP + PLAYABLE - 1) * SIDE + LEFT + slot] = -i32::from(*value);
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    Type8BoardSpec {
        width: SIDE,
        height: SIDE,
        shape,
        selectable: vec![401, 402, 403, 404],
        collectable: values.to_vec(),
        tiles: [401, 402, 403, 404]
            .iter()
            .map(|value| Type8Kind {
                value: *value,
                category: 400,
                icon_id: i32::from(*value) - 400,
            })
            .chain(values.iter().map(|value| Type8Kind {
                value: *value,
                category: 600,
                icon_id: 700 + i32::from(*value),
            }))
            .collect(),
        goals: values
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
        outputs: values
            .iter()
            .enumerate()
            .map(|(slot, value)| Type8Output {
                value: *value,
                definition_id: OUTPUTS[slot],
                cluster_quantity: 1,
            })
            .collect(),
        duration_seconds: 180,
        seed: 140,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Synthetic private W140 fixture; original level tables unrecovered.".into(),
    }
}

struct Fixture {
    game: Game,
    directory: PathBuf,
    reference: PathBuf,
}

impl Fixture {
    fn new(game: Game) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-gathering-payout-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&base_content(), &directory);
        let (file, content) = if game.name == "mining" {
            (
                mining::CONTENT_FILE,
                json!({
                    "schema_version": 1, "game_id": MINING_GAME_ID, "profile_id": MINER_PROFILE_ID,
                    "name_id": 7910, "description_id": 7911, "icon_id": 909,
                    "spec": board(game.goals),
                }),
            )
        } else {
            (
                CONTENT_FILE,
                json!({
                    "schema_version": 1, "game_id": HARVEST_GAME_ID, "profile_id": CHEF_PROFILE_ID,
                    "name_id": 100834, "description_id": 94150, "icon_id": 908,
                    "preload_file": FARMING_PRELOAD_FILE, "launch_movie": FARMING_LAUNCH_MOVIE,
                    "spec": board(game.goals),
                }),
            )
        };
        fs::write(directory.join(file), serde_json::to_vec(&content).unwrap()).unwrap();
        Self {
            game,
            directory,
            reference: manifest_path(
                "../../../lab/references/rust-gateway-reference",
                "OSFR_GATEWAY_REFERENCE",
            ),
        }
    }

    fn state(&self) -> GameplayState {
        let mut state = Gameplay::load(&self.directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = self.game.profile;
        profiles.unlocked.insert(self.game.profile);
        state
    }

    fn gateway_from(&self, state: Option<GameplayState>) -> Gateway {
        let character = Gateway::reference_state(&self.reference).unwrap();
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.directory, state).unwrap();
        gateway.set_dev_tools(true);
        gateway.set_role(osfr_tools::account_store::AccountRole::Admin);
        login(&mut gateway);
        gateway
    }

    /// Fresh gateway with the run offered; returns the instance id.
    fn started(&self) -> (Gateway, i32) {
        let mut gateway = self.gateway_from(Some(self.state()));
        say(&mut gateway, self.game.command, 2_000);
        let id = active(&gateway, self.game).expect("run offered") as i32;
        (gateway, id)
    }

    /// Save, reload and log in again, as a disconnect and reconnect would.
    fn reconnected(&self, gateway: &Gateway) -> Gateway {
        let snapshot = gateway.snapshot();
        let path = self.directory.join("w140-save.json");
        game_store::save(&path, &snapshot).unwrap();
        let loaded = game_store::load_or_create(&path, snapshot.character.clone()).unwrap();
        self.gateway_from(loaded.gameplay)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn manifest_path(relative: &str, variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative))
}

fn base_content() -> PathBuf {
    manifest_path(
        "../../../lab/references/opening-journal-text-v2/content",
        "OSFR_TYPE8_CONTENT",
    )
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

fn quit(instance: i32) -> Vec<u8> {
    frame(6, instance, -1)
}

const CLOSE: [u8; 4] = [26, 0, 42, 0];

/// Third byte of every family-39 reply (the minigame sub-opcode), in order.
fn subs(outcome: &GatewayOutcome) -> Vec<u8> {
    outcome
        .replies
        .iter()
        .map(|reply| inner(reply))
        .filter(|packet| packet.len() > 2 && packet[..2] == [39, 0])
        .map(|packet| packet[2])
        .collect()
}

fn payload_texts(outcome: &GatewayOutcome) -> Vec<String> {
    outcome
        .replies
        .iter()
        .filter_map(|reply| {
            let packet = inner(reply);
            if packet.len() < 20 || packet[0..3] != [39, 0, 14] {
                return None;
            }
            String::from_utf8(packet[19..packet.len() - 1].to_vec()).ok()
        })
        .collect()
}

fn gameplay_json(gateway: &Gateway) -> Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn run_json(gateway: &Gateway, game: Game) -> Value {
    gameplay_json(gateway)[game.json_key].clone()
}

fn active(gateway: &Gateway, game: Game) -> Option<i64> {
    run_json(gateway, game)["active"]["instance_id"].as_i64()
}

fn phase(gateway: &Gateway, game: Game) -> Option<String> {
    run_json(gateway, game)["active"]["phase"]
        .as_str()
        .map(str::to_owned)
}

/// Everything a payout or refusal could touch (inventory, coins, XP, quests), minus the run itself.
fn economy(gateway: &Gateway, game: Game) -> Value {
    let mut json = gameplay_json(gateway);
    json.as_object_mut().unwrap().remove(game.json_key);
    json
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

fn ore(gateway: &Gateway, game: Game) -> Vec<u32> {
    OUTPUTS[..game.goals]
        .iter()
        .map(|d| count(gateway, *d))
        .collect()
}

fn fall(gateway: &mut Gateway, instance: i32, slot: usize, now_ms: u64) -> GatewayOutcome {
    let text = format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM);
    send(gateway, &payload(instance, &text), now_ms)
}

/// Drive a freshly offered run to `target`, asserting the engine really is there.
fn drive(gateway: &mut Gateway, game: Game, id: i32, target: &str) {
    let steps = [
        frame(5, id, -1),
        payload(id, "OnConnectMsg"),
        payload(id, "OnStartGameMsg"),
        payload(id, "OnStartLevelMsg"),
    ];
    let reach = PHASES.iter().position(|p| *p == target).unwrap();
    for (step, packet) in steps.iter().enumerate().take(reach.min(4)) {
        send(gateway, packet, 2_100 + 100 * step as u64);
    }
    if target == "LevelComplete" {
        for slot in 0..game.goals {
            fall(gateway, id, slot, 2_600 + slot as u64);
        }
    }
    assert_eq!(
        phase(gateway, game).as_deref(),
        Some(target),
        "{}",
        game.name
    );
}

/// Win the level and take the payout. Returns the reply to the win request.
fn win(gateway: &mut Gateway, game: Game, id: i32) -> GatewayOutcome {
    drive(gateway, game, id, "LevelComplete");
    send(gateway, &payload(id, "OnNextLevelMsg"), 2_700)
}

/// Messages a stale client could still send after a run is gone or paid.
fn stale(id: i32) -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("39/6 quit", quit(id)),
        ("39/7 ack", frame(7, id, -1)),
        ("26/42 close", CLOSE.to_vec()),
        ("OnNextLevelMsg", payload(id, "OnNextLevelMsg")),
        ("FRServer_GameOver", payload(id, "FRServer_GameOver")),
        ("FRServer_GameClose", payload(id, "FRServer_GameClose")),
        (
            "fall",
            payload(id, &format!("OnLevelEventMsg\t50\t{LEFT}\t{BOTTOM}")),
        ),
        ("OnStartLevelMsg", payload(id, "OnStartLevelMsg")),
    ]
}

fn assert_inert(
    gateway: &mut Gateway,
    game: Game,
    packets: Vec<(&'static str, Vec<u8>)>,
    what: &str,
) {
    for (name, packet) in packets {
        let before = gateway.snapshot();
        let out = send(gateway, &packet, 9_000);
        assert!(
            out.replies.is_empty(),
            "{} {what}: {name} replied",
            game.name
        );
        assert_eq!(
            gateway.snapshot(),
            before,
            "{} {what}: {name} mutated",
            game.name
        );
    }
}

#[test]
fn ordinary_win_pays_once_then_close_returns_to_the_world() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (mut gateway, id) = fixture.started();
        let before = economy(&gateway, game);
        assert_eq!(ore(&gateway, game), vec![0; game.goals]);

        let won = win(&mut gateway, game, id);
        assert!(
            payload_texts(&won).iter().any(|t| t == "OnGameWonMsg"),
            "{}",
            game.name
        );
        assert_eq!(
            ore(&gateway, game),
            vec![1; game.goals],
            "{} bundle",
            game.name
        );
        assert_ne!(economy(&gateway, game), before);
        assert_eq!(phase(&gateway, game).as_deref(), Some("Won"));

        // Results sequence is sent once; a replay sends nothing.
        let results = send(&mut gateway, &payload(id, "FRServer_GameOver"), 2_800);
        assert_eq!(subs(&results), vec![18, 47, 19], "{} results", game.name);
        let paid = economy(&gateway, game);
        let again = send(&mut gateway, &payload(id, "FRServer_GameOver"), 2_900);
        assert!(subs(&again).is_empty(), "{} results replay", game.name);

        // A replayed win request and a Quit after the win pay and change nothing.
        for packet in [payload(id, "OnNextLevelMsg"), quit(id), frame(7, id, -1)] {
            let out = send(&mut gateway, &packet, 3_000);
            assert!(out.replies.is_empty(), "{}", game.name);
        }
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);
        assert_eq!(economy(&gateway, game), paid);
        assert_eq!(phase(&gateway, game).as_deref(), Some("Won"));

        // Results Close (26/42) releases the run; the movie already ended, so no reply.
        let closed = send(&mut gateway, &CLOSE, 3_100);
        assert!(subs(&closed).is_empty(), "{} close", game.name);
        assert!(active(&gateway, game).is_none());
        assert_eq!(economy(&gateway, game), paid);
        assert_inert(&mut gateway, game, stale(id), "after close");
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);

        // Canary: the same player can run again and is paid exactly one more bundle.
        say(&mut gateway, game.command, 4_000);
        let second = active(&gateway, game).unwrap() as i32;
        assert_eq!(second, id + 1, "{} fresh instance", game.name);
        win(&mut gateway, game, second);
        assert_eq!(ore(&gateway, game), vec![2; game.goals]);
        // The first run's identifiers stay dead while the second is live. (26/42 carries no
        // instance, so it is not an instance-bound message.)
        let old: Vec<_> = stale(id)
            .into_iter()
            .filter(|(n, _)| !n.starts_with("26/42"))
            .collect();
        assert_inert(&mut gateway, game, old, "old instance");
        assert_eq!(ore(&gateway, game), vec![2; game.goals]);
    }
}

#[test]
fn quit_before_win_clears_the_run_and_pays_nothing_in_every_phase() {
    for game in GAMES {
        for target in PHASES {
            let fixture = Fixture::new(game);
            let (mut gateway, id) = fixture.started();
            drive(&mut gateway, game, id, target);
            let before = economy(&gateway, game);

            let out = send(&mut gateway, &quit(id), 5_000);
            // The recovered Quit answer is the removal packet only: no results, no scores.
            assert_eq!(subs(&out), vec![19], "{} {target}", game.name);
            assert!(active(&gateway, game).is_none(), "{} {target}", game.name);
            assert_eq!(
                economy(&gateway, game),
                before,
                "{} {target} economy",
                game.name
            );
            assert_eq!(ore(&gateway, game), vec![0; game.goals]);
            assert_eq!(run_json(&gateway, game)["completed"].as_u64(), Some(0));

            // Every late message from the old movie is inert.
            assert_inert(&mut gateway, game, stale(id), target);
            assert_eq!(ore(&gateway, game), vec![0; game.goals]);

            // Canary: the player can start over and win normally afterwards.
            say(&mut gateway, game.command, 6_000);
            let next = active(&gateway, game).unwrap() as i32;
            win(&mut gateway, game, next);
            assert_eq!(
                ore(&gateway, game),
                vec![1; game.goals],
                "{} {target}",
                game.name
            );
        }
    }
}

#[test]
fn forged_quit_and_forged_terminal_reports_change_nothing() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (mut gateway, id) = fixture.started();
        drive(&mut gateway, game, id, "Playing");
        let snapshot = gateway.snapshot();

        // Quit for another instance, with a non-sentinel group or game: ignored.
        for packet in [
            quit(id + 1),
            quit(id - 1),
            quit(0),
            quit(-1),
            frame(6, id, game_id_of(game)),
            {
                let mut p = quit(id);
                p[7..11].copy_from_slice(&5_i32.to_le_bytes());
                p
            },
        ] {
            let out = send(&mut gateway, &packet, 5_000);
            assert!(out.replies.is_empty(), "{}", game.name);
            assert_eq!(gateway.snapshot(), snapshot, "{} forged quit", game.name);
        }
        // Truncated or oversized Quit frames are refused without touching the run.
        for length in [3, 14, 16] {
            let mut p = quit(id);
            p.resize(length, 0);
            let before = gateway.snapshot();
            let _ = gateway.handle_at(&wrap(&p), 5_100);
            assert_eq!(gateway.snapshot(), before, "{} len {length}", game.name);
        }
        // Client score and win reports are not authority.
        for text in [
            "FRServer_GameWon\t999999",
            "FRServer_GameWon\t999999\t1",
            "FRServer_GameLost\t0",
            "FRServer_GameQuit\t0",
            "FRServer_ScoreSet\t999999",
        ] {
            let before = gateway.snapshot();
            let _ = gateway.handle_at(&wrap(&payload(id, text)), 5_200);
            assert_eq!(gateway.snapshot(), before, "{} {text}", game.name);
        }
        // A client cannot declare the level won; only server quota progress reaches Won.
        let before = gateway.snapshot();
        let _ = gateway.handle_at(&wrap(&payload(id, "OnNextLevelMsg")), 5_300);
        assert_eq!(gateway.snapshot(), before, "{} early next-level", game.name);
        let _ = gateway.handle_at(&wrap(&payload(id, "OnGameWonMsg")), 5_400);
        assert_eq!(ore(&gateway, game), vec![0; game.goals]);
        // The unforged run is still live and still pays (positive control).
        win(&mut gateway, game, id);
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);
    }
}

#[test]
fn forged_board_progress_never_advances_the_quota() {
    // The only way to the reward is server-validated board progress: a fall must name a goal
    // collectable on the bottom row, a chain three idle same-colour orthogonal tiles.
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (mut gateway, id) = fixture.started();
        drive(&mut gateway, game, id, "Playing");
        let bottom = BOTTOM;
        let off_board = LEFT + PLAYABLE + 3;
        let plain_slot = LEFT + game.goals; // a bottom cell that holds no collectable
        let forged = [
            format!("OnLevelEventMsg\t50\t{off_board}\t{bottom}"),
            format!("OnLevelEventMsg\t50\t{LEFT}\t{TOP}"),
            format!("OnLevelEventMsg\t50\t{plain_slot}\t{bottom}"),
            "OnLevelEventMsg\t50\t-1\t-1".to_owned(),
            // Too short, repeated, non-adjacent, off-board and not-yet-idle chains.
            format!("OnChainMsg\t{LEFT},{TOP},{},{TOP},", LEFT + 1),
            format!("OnChainMsg\t{LEFT},{TOP},{LEFT},{TOP},{LEFT},{TOP},"),
            format!(
                "OnChainMsg\t{LEFT},{TOP},{},{TOP},{},{TOP},",
                LEFT + 2,
                LEFT + 4
            ),
            format!(
                "OnChainMsg\t{off_board},{TOP},{off_board},{},{off_board},{},",
                TOP + 1,
                TOP + 2
            ),
            format!(
                "OnChainMsg\t{LEFT},{TOP},{},{TOP},{},{TOP},",
                LEFT + 1,
                LEFT + 2
            ),
            // Powerup action on a board that has none.
            format!("OnActionMsg\t951\t{LEFT},{TOP},"),
        ];
        for text in forged {
            let before = gateway.snapshot();
            let _ = gateway.handle_at(&wrap(&payload(id, &text)), 5_000);
            assert_eq!(gateway.snapshot(), before, "{} forged {text:?}", game.name);
        }
        assert_eq!(
            run_json(&gateway, game)["active"]["collected"],
            json!(vec![0; game.goals])
        );
        // Without real progress the level never completes, so the win request pays nothing.
        let early = gateway.snapshot();
        let _ = gateway.handle_at(&wrap(&payload(id, "OnNextLevelMsg")), 5_100);
        assert_eq!(gateway.snapshot(), early);
        assert_eq!(ore(&gateway, game), vec![0; game.goals]);
        // Canary: the real fall still advances and pays.
        win(&mut gateway, game, id);
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);
    }
}

fn game_id_of(game: Game) -> i32 {
    if game.name == "mining" {
        MINING_GAME_ID
    } else {
        HARVEST_GAME_ID
    }
}

#[test]
fn quota_met_but_never_confirmed_pays_nothing_when_the_player_closes() {
    // (R) The bundle is committed only with the client's OnNextLevelMsg after the server-owned
    // OnStopLevel; Close or Quit in LevelComplete forfeits it rather than guessing a payout.
    for game in GAMES {
        for use_close in [true, false] {
            let fixture = Fixture::new(game);
            let (mut gateway, id) = fixture.started();
            drive(&mut gateway, game, id, "LevelComplete");
            let exit = if use_close { CLOSE.to_vec() } else { quit(id) };
            send(&mut gateway, &exit, 5_000);
            assert!(active(&gateway, game).is_none(), "{}", game.name);
            assert_eq!(ore(&gateway, game), vec![0; game.goals], "{}", game.name);
            assert_inert(&mut gateway, game, stale(id), "after forfeit");
        }
    }
}

#[test]
fn a_win_saved_and_replayed_after_reload_pays_once() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (mut gateway, id) = fixture.started();
        win(&mut gateway, game, id);
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);

        // The save carries the Won run. Replaying the terminal message at any point pays nothing.
        let snapshot = gateway.snapshot();
        let path = fixture.directory.join("w140-won.json");
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
        assert_eq!(ore(&restored, game), vec![1; game.goals]);
        for _ in 0..2 {
            send(&mut restored, &payload(id, "OnNextLevelMsg"), 7_000);
            assert_eq!(
                ore(&restored, game),
                vec![1; game.goals],
                "{} replay",
                game.name
            );
        }
    }
}

// Needs the gameplay.rs patch in `lab/runs/claude/W140/final.md` (initialize_world must cancel an
// unfinished Mining/Harvesting run the way it already does for Smelting/Cooking/Forging).
// Without it a reconnecting player keeps a live run with no movie and every job/equipment/
// interaction request is refused ("Close the mining game ...") until a developer types
// `realm exit`.
#[test]
fn reconnect_releases_an_unfinished_run_without_paying() {
    for game in GAMES {
        for target in ["Offered", "Playing", "LevelComplete"] {
            let fixture = Fixture::new(game);
            let (mut gateway, id) = fixture.started();
            drive(&mut gateway, game, id, target);
            let reconnected = fixture.reconnected(&gateway);
            assert!(
                active(&reconnected, game).is_none(),
                "{} {target}",
                game.name
            );
            assert_eq!(
                ore(&reconnected, game),
                vec![0; game.goals],
                "{} {target}",
                game.name
            );
        }
        // A paid Won run is also released on reconnect, keeping its single payout.
        let fixture = Fixture::new(game);
        let (mut gateway, id) = fixture.started();
        win(&mut gateway, game, id);
        let mut reconnected = fixture.reconnected(&gateway);
        assert!(active(&reconnected, game).is_none(), "{} won", game.name);
        assert_eq!(ore(&reconnected, game), vec![1; game.goals]);
        assert_inert(&mut reconnected, game, stale(id), "after reconnect");
        assert_eq!(ore(&reconnected, game), vec![1; game.goals]);
    }
}

/// What the live gateway route (`Type8Engine` plus gameplay's `26/42` -> `cancel()`) and the shared
/// W38 lifecycle adapter each do with the same client bytes.
trait Route {
    fn offer(&mut self, profile: i32);
    fn feed(&mut self, packet: &[u8]) -> Vec<u8>;
    fn live(&self) -> bool;
    fn next_id(&self) -> i32;
}

struct Direct(Type8Engine);

impl Route for Direct {
    fn offer(&mut self, profile: i32) {
        self.0.begin(profile).unwrap();
    }
    fn feed(&mut self, packet: &[u8]) -> Vec<u8> {
        let out = if packet == CLOSE {
            self.0.cancel()
        } else {
            self.0.handle(packet).unwrap()
        };
        raw_subs(&out.replies)
    }
    fn live(&self) -> bool {
        self.0.active()
    }
    fn next_id(&self) -> i32 {
        self.0.state().next_instance_id
    }
}

struct Mining18(MiningLifecycle);
struct Harvest111(HarvestingLifecycle);

impl Route for Mining18 {
    fn offer(&mut self, profile: i32) {
        self.0.begin(profile).unwrap();
    }
    fn feed(&mut self, packet: &[u8]) -> Vec<u8> {
        raw_subs(&self.0.handle(packet, None).unwrap().replies)
    }
    fn live(&self) -> bool {
        self.0.activity_state().active.is_some()
    }
    fn next_id(&self) -> i32 {
        self.0.activity_state().next_instance_id
    }
}

impl Route for Harvest111 {
    fn offer(&mut self, profile: i32) {
        self.0.begin(profile).unwrap();
    }
    fn feed(&mut self, packet: &[u8]) -> Vec<u8> {
        raw_subs(&self.0.handle(packet, None).unwrap().replies)
    }
    fn live(&self) -> bool {
        self.0.activity_state().active.is_some()
    }
    fn next_id(&self) -> i32 {
        self.0.activity_state().next_instance_id
    }
}

fn raw_subs(replies: &[Vec<u8>]) -> Vec<u8> {
    replies
        .iter()
        .filter(|packet| packet.len() > 2 && packet[..2] == [39, 0])
        .map(|packet| packet[2])
        .collect()
}

fn engine(fixture: &Fixture) -> Type8Engine {
    let (file, game_id, first) = if fixture.game.name == "mining" {
        (mining::CONTENT_FILE, MINING_GAME_ID, FIRST_MINING_INSTANCE)
    } else {
        (CONTENT_FILE, HARVEST_GAME_ID, FIRST_INSTANCE_ID)
    };
    Type8Engine::load(&fixture.directory, file, game_id, first, None).unwrap()
}

fn routes(fixture: &Fixture) -> (Direct, Box<dyn Route>) {
    let game = fixture.game;
    let direct = Direct(engine(fixture));
    let shared: Box<dyn Route> = if game.name == "mining" {
        Box::new(Mining18(
            MiningLifecycle::new(engine(fixture), 140).unwrap(),
        ))
    } else {
        Box::new(Harvest111(
            HarvestingLifecycle::new(engine(fixture), HARVEST_GAME_ID, 140).unwrap(),
        ))
    };
    (direct, shared)
}

/// Offer a run and bring it to `target` through the same client messages.
fn reach(route: &mut dyn Route, game: Game, target: &str) -> i32 {
    let id = route.next_id();
    route.offer(game.profile);
    let steps = [
        frame(5, id, -1),
        payload(id, "OnConnectMsg"),
        payload(id, "OnStartGameMsg"),
        payload(id, "OnStartLevelMsg"),
    ];
    let upto = PHASES.iter().position(|p| *p == target).unwrap();
    for packet in steps.iter().take(upto.min(4)) {
        route.feed(packet);
    }
    if target == "LevelComplete" {
        for slot in 0..game.goals {
            let text = format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM);
            route.feed(&payload(id, &text));
        }
    }
    id
}

/// Compares what the live route (`Type8Engine`, with gameplay's `26/42` -> `cancel()`) and the
/// shared W38 lifecycle adapter reply and keep alive for the same client bytes. Run with
/// `--nocapture` to print the table.
#[test]
fn live_route_and_shared_lifecycle_agree_on_quit_and_leave_ack() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        for target in PHASES {
            for probe in ["quit", "ack", "close"] {
                let observe = |shared: bool| {
                    let (mut direct, mut lifecycle) = routes(&fixture);
                    let route: &mut dyn Route = if shared { &mut *lifecycle } else { &mut direct };
                    let id = reach(route, game, target);
                    let packet = match probe {
                        "quit" => quit(id),
                        "ack" => frame(7, id, -1),
                        _ => CLOSE.to_vec(),
                    };
                    (route.feed(&packet), route.live())
                };
                let (live, shared) = (observe(false), observe(true));
                eprintln!(
                    "{} {target:<12} {probe:<5} live-route {live:?} shared-lifecycle {shared:?}",
                    game.name
                );
                if probe != "close" {
                    assert_eq!(live, shared, "{} {target} {probe}", game.name);
                }
                if probe == "close" && target == "Offered" {
                    // Native OfferCancel: 26/42 then the 39/19 removal, run gone.
                    assert_eq!(live, (vec![19], false), "{} offered close", game.name);
                    assert_eq!(live, shared, "{} offered close", game.name);
                }
            }
        }
    }
}
