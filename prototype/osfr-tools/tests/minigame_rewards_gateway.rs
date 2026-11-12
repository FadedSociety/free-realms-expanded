//! KNOWN-BUGS C4, C5, C9 through the gateway, from the client's own packets: a won match-three
//! board shows its prize wheel (39/45 between 39/18 and 39/19, matched against the 39/16
//! preview bundle) and pays its goals and the wheel's slot once; a chain through a locked tile
//! unlocks it and moves the "Unlock N locked tiles" goal; the shipped boards spawn locked tiles
//! and clickable powerups.
//!
//! Fixtures copy the private lab content directory into a temp directory and add their own
//! `mining.json`, `harvesting.json` and `minigame_rewards.json`; nothing shared is written.

use osfr_tools::{
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    minigame::{MinigameRewards, Prize},
    mining::{FIRST_MINING_INSTANCE, MINER_PROFILE_ID, MINING_GAME_ID, SIDE},
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
/// Item definition 2739 (mined ore) exists in the private campaign inventory manifest.
const ORE_DEFINITION: i32 = 2739;
const CHEF: i32 = 45;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn lab(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../lab/references").join(path)
}

fn shipped(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content").join(file)
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

/// A Mining18 board on the 14x14 grid: `rows` is the 8x8 playable block (shape codes), with the
/// dropzone row under it.
fn mining_json(rows: [[i32; PLAYABLE]; PLAYABLE], extra: Value) -> Value {
    let mut shape = vec![0_i32; SIDE * SIDE];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, value) in row.iter().enumerate() {
            shape[(TOP + dy) * SIDE + LEFT + dx] = *value;
        }
    }
    for x in LEFT..LEFT + PLAYABLE {
        shape[(TOP + PLAYABLE) * SIDE + x] = 4;
    }
    let mut content = json!({
        "schema_version": 1,
        "game_id": MINING_GAME_ID,
        "profile_id": MINER_PROFILE_ID,
        "name_id": 7910,
        "description_id": 7911,
        "icon_id": 909,
        "spec": {
            "width": SIDE, "height": SIDE, "shape": shape,
            "selectable": [401, 402, 403, 404], "collectable": [ORE],
            "tiles": [
                {"value": 401, "category": 400, "icon_id": 1},
                {"value": 402, "category": 400, "icon_id": 2},
                {"value": ORE, "category": 600, "icon_id": 909}
            ],
            "goals": [{"index": 0, "value": ORE, "quota": 1, "per_reward": 1, "icon_id": 909}],
            "outputs": [{"value": ORE, "definition_id": ORE_DEFINITION, "cluster_quantity": 1}],
            "duration_seconds": 180, "seed": 18,
            "wildcard_percent": 0, "collectable_percent": 0, "blocker_percent": 0, "powerup_percent": 0,
            "meter_id": 0, "reconstructed_rules": true,
            "rule_notice": "Synthetic private gateway fixture."
        }
    });
    for (key, value) in extra.as_object().unwrap() {
        content[key] = value.clone();
    }
    content
}

/// One ore on the bottom row: a fall clears the level.
fn winning_rows() -> [[i32; PLAYABLE]; PLAYABLE] {
    let mut rows = [[1; PLAYABLE]; PLAYABLE];
    rows[PLAYABLE - 1][0] = -i32::from(ORE);
    rows
}

struct Fixture {
    directory: PathBuf,
}

impl Fixture {
    fn new(files: &[(&str, Vec<u8>)]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-minigame-rewards-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(&lab("opening-journal-text-v2/content"), &directory);
        for (name, bytes) in files {
            fs::write(directory.join(name), bytes).unwrap();
        }
        Self { directory }
    }

    fn gateway(&self, job: i32) -> Gateway {
        let mut state: GameplayState = Gameplay::load(&self.directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        profiles.active = job;
        profiles.unlocked.insert(job);
        let reference = lab("rust-gateway-reference");
        let character = Gateway::reference_state(&reference).unwrap();
        let mut gateway =
            Gateway::load_gameplay(&reference, character, &self.directory, Some(state)).unwrap();
        gateway.set_dev_tools(true);
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

fn login(gateway: &mut Gateway) {
    let mut b = 1u16.to_le_bytes().to_vec();
    for (i, s) in ["local-private-test", "1.910.1.530630", ""].iter().enumerate() {
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

fn send(gateway: &mut Gateway, inner: &[u8], now_ms: u64) -> GatewayOutcome {
    let mut b = vec![5, 0, 1];
    b.extend((inner.len() as u32).to_le_bytes());
    b.extend(inner);
    gateway.handle_at(&b, now_ms).expect("gateway frame")
}

/// The client's chat line (15/1), as the dev command arrives.
fn say(gateway: &mut Gateway, text: &str, now_ms: u64) -> GatewayOutcome {
    let mut p = vec![15, 0, 1, 0, 0, 0];
    p.extend(1u64.to_le_bytes());
    p.extend(0u64.to_le_bytes());
    for _ in 0..2 {
        p.extend([0; 20]);
    }
    p.extend((text.len() as u32).to_le_bytes());
    p.extend(text.as_bytes());
    p.extend([0; 28]);
    send(gateway, &p, now_ms)
}

fn inner(reply: &[u8]) -> &[u8] {
    if reply.starts_with(&[5, 0, 1]) && reply.len() >= 7 {
        let length = u32::from_le_bytes(reply[3..7].try_into().unwrap()) as usize;
        if length == reply.len() - 7 {
            return &reply[7..];
        }
    }
    reply
}

fn packets(outcome: &GatewayOutcome) -> Vec<Vec<u8>> {
    outcome.replies.iter().map(|reply| inner(reply).to_vec()).collect()
}

/// `[u16 39][sub][i32 instance][i32 -1][i32 game]`, the client's minigame frame.
fn frame(sub: u8, instance: i32, game: i32) -> Vec<u8> {
    let mut b = 39_u16.to_le_bytes().to_vec();
    b.push(sub);
    for n in [instance, -1, game] {
        b.extend(n.to_le_bytes());
    }
    b
}

/// The movie's 39/14 text message.
fn movie(instance: i32, text: &str) -> Vec<u8> {
    let mut b = frame(14, instance, -1);
    b.extend(((text.len() + 1) as i32).to_le_bytes());
    b.extend(text.as_bytes());
    b.push(0);
    b
}

fn texts(outcome: &GatewayOutcome) -> Vec<String> {
    packets(outcome)
        .into_iter()
        .filter(|p| p.len() > 19 && p[..3] == [39, 0, 14])
        .map(|p| String::from_utf8_lossy(&p[19..p.len() - 1]).into_owned())
        .collect()
}

fn snapshot(gateway: &Gateway) -> Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn coins(gateway: &Gateway) -> u64 {
    snapshot(gateway)["inventory"]["coins"].as_u64().unwrap()
}

fn xp(gateway: &Gateway, job: i32) -> u64 {
    snapshot(gateway)["profiles"]["experience"][job.to_string()].as_u64().unwrap_or(0)
}

fn count(gateway: &Gateway, definition: i32) -> u64 {
    let state = snapshot(gateway);
    (state["inventory"]["items"].as_array().unwrap().iter())
        .filter(|item| item["definition_id"] == definition)
        .map(|item| item["count"].as_u64().unwrap())
        .sum()
}

fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

/// The movie handshake to Playing, as the client sends it.
fn handshake(gateway: &mut Gateway, instance: i32, now_ms: u64) -> Vec<String> {
    send(gateway, &frame(5, instance, -1), now_ms);
    send(gateway, &movie(instance, "OnConnectMsg"), now_ms + 100);
    send(gateway, &movie(instance, "OnStartGameMsg"), now_ms + 200);
    texts(&send(gateway, &movie(instance, "OnStartLevelMsg"), now_ms + 300))
}

/// Every tile value the level start sent (the board and the replacement queue).
fn spawned(level_start: &[String]) -> Vec<u16> {
    (level_start.iter())
        .filter_map(|text| text.strip_prefix("OnTileQueueMsg\t-1\t"))
        .flat_map(|csv| csv.split(',').map(|v| v.parse::<u16>().unwrap()))
        .collect()
}

const REWARDS: &str = r#"{
  "schema_version": 1,
  "games": [{
    "game_id": 18,
    "goals": [{"coins": 11, "stars": 230}, {"coins": 7, "stars": 0}],
    "wheel": {"coins": 447, "slices": [
      {"item_id": 2739, "name_id": 16221, "icon_id": 909, "quantity": 2, "gender": 0}
    ]}
  }]
}"#;

/// C4 + C9: the offer previews the wheel and shows each goal's payout; the win pays the met
/// goals and the wheel's slot once; the results land the wheel ahead of 39/19; the client's
/// stop report and a replayed win pay nothing more.
#[test]
fn a_won_board_lands_its_wheel_and_pays_its_goals_once() {
    let objectives = json!({"objectives": [
        {"name_id": 102511, "category": 1, "counts": {"bin": 0}, "target": 1},
        {"name_id": 92848, "category": 2, "counts": "score", "target": 999999}
    ]});
    let fixture = Fixture::new(&[
        ("mining.json", serde_json::to_vec(&mining_json(winning_rows(), objectives)).unwrap()),
        ("minigame_rewards.json", REWARDS.as_bytes().to_vec()),
    ]);
    let mut gateway = fixture.gateway(MINER_PROFILE_ID);
    let wheel = MinigameRewards::parse(REWARDS.as_bytes()).unwrap().game(18).unwrap().wheel.clone();

    let offer = say(&mut gateway, "realm mine", 2_000);
    let info = packets(&offer).into_iter().find(|p| p.starts_with(&[39, 0, 16])).expect("39/16");
    // Header 15, six words and the members byte: the reward bundles start at 40. The first two
    // are empty; the third (the preview) holds the wheel: its coins, then one Item entry.
    let preview = 40 + 2 * 69;
    assert_eq!(i32_at(&info, preview + 1), 447, "the preview's Coins slot");
    assert_eq!(i32_at(&info, preview + 61), 1, "one item slot");
    assert_eq!(i32_at(&info, preview + 65), 1, "an Item entry");
    assert_eq!(i32_at(&info, preview + 82), 2, "its quantity");
    assert_eq!(i32_at(&info, preview + 86), ORE_DEFINITION, "its item");
    // The goals follow the preview: each shows its coins and stars.
    let goals = preview + 69 + 38;
    assert_eq!(i32_at(&info, goals), 2);
    assert_eq!((i32_at(&info, goals + 18), i32_at(&info, goals + 22)), (11, 230));

    let instance = FIRST_MINING_INSTANCE;
    handshake(&mut gateway, instance, 2_100);
    let (coins_before, xp_before, ore_before) =
        (coins(&gateway), xp(&gateway, MINER_PROFILE_ID), count(&gateway, ORE_DEFINITION));
    let fall = format!("OnLevelEventMsg\t50\t{LEFT}\t{BOTTOM}");
    send(&mut gateway, &movie(instance, &fall), 2_600);
    let win = send(&mut gateway, &movie(instance, "OnNextLevelMsg"), 2_700);
    assert!(texts(&win).iter().any(|t| t == "OnGameWonMsg"));

    // Only the primary goal was met; the secondary's 7 coins stay unpaid.
    let prize = wheel.prize(instance).expect("a wheel with slots lands");
    let (prize_coins, prize_ore) = match &prize {
        Prize::Coins(coins) => (u64::from(*coins), 0),
        Prize::Item(slice) => (0, u64::from(slice.quantity)),
    };
    assert_eq!(coins(&gateway), coins_before + 11 + prize_coins);
    assert_eq!(xp(&gateway, MINER_PROFILE_ID), xp_before + 230);
    assert_eq!(count(&gateway, ORE_DEFINITION), ore_before + 1 + prize_ore);

    let results = packets(&send(&mut gateway, &movie(instance, "FRServer_GameOver"), 2_800));
    let at = |sub: u8| results.iter().position(|p| p[..3] == [39, 0, sub]).expect("result packet");
    assert!(at(18) < at(47) && at(47) < at(45) && at(45) < at(19), "39/18, 39/47, 39/45, 39/19");
    let land = &results[at(45)];
    assert_eq!(i32_at(land, 3), instance);
    match prize {
        Prize::Item(_) => {
            assert_eq!(i32_at(land, 15 + 61), 1);
            assert_eq!(i32_at(land, 15 + 86), ORE_DEFINITION, "the landing entry is a preview entry");
        }
        Prize::Coins(coins) => {
            assert_eq!(i32_at(land, 15 + 1), coins as i32, "the Coins slot");
            assert_eq!(i32_at(land, 15 + 61), 0);
        }
    }

    // The wheel's stop report and a replayed win change nothing and keep the session.
    let paid = gateway.snapshot();
    let stop = send(&mut gateway, &frame(46, instance, -1), 2_900);
    assert!(!stop.gameplay_changed);
    send(&mut gateway, &movie(instance, "OnNextLevelMsg"), 3_000);
    assert_eq!(gateway.snapshot(), paid, "the win paid once");
}

/// C5: a locked tile travels as value + 100; a chain through it unlocks it in place (it stays
/// on the board as its selectable value and fills no bin) and moves the Elite goal.
#[test]
fn a_chain_through_a_locked_tile_unlocks_it_and_moves_the_elite_goal() {
    let mut rows = [[1; PLAYABLE]; PLAYABLE];
    rows[0][0] = -401;
    rows[0][1] = -501;
    rows[0][2] = -401;
    let extra = json!({"objectives": [
        {"name_id": 92946, "category": 5, "counts": "locked", "target": 1}
    ]});
    let fixture = Fixture::new(&[("mining.json", serde_json::to_vec(&mining_json(rows, extra)).unwrap())]);
    let mut gateway = fixture.gateway(MINER_PROFILE_ID);
    say(&mut gateway, "realm mine", 2_000);
    let instance = FIRST_MINING_INSTANCE;
    let level = handshake(&mut gateway, instance, 2_100);
    let board = spawned(&level);
    let cell = TOP * SIDE + LEFT + 1;
    assert_eq!(board[cell], 501, "the client receives the locked value");

    let chain = format!("OnChainMsg\t{},{TOP},{},{TOP},{},{TOP},", LEFT, LEFT + 1, LEFT + 2);
    let out = send(&mut gateway, &movie(instance, &chain), 2_500);
    // Goal 1 (the Elite row) is met: 47/3 and 45/3.
    let done = packets(&out);
    assert!(done.iter().any(|p| p[..3] == [47, 0, 3] && i32_at(p, 3) == 1), "panel row met");
    assert!(done.iter().any(|p| p[..3] == [45, 0, 3] && i32_at(p, 3) == 1), "offer record met");
    let state = snapshot(&gateway);
    let after = state["mining"]["active"]["board"].as_array().unwrap();
    assert_eq!(after[cell], 401, "the unlocked tile stays as its selectable value");
    assert_eq!(state["mining"]["active"]["collected"][0], 0, "it filled no bin");
}

/// C5: the shipped boards spawn the movie's clickable powerups (`OnPowerupInitMsg`, 950-953) and,
/// where a board has an "Unlock N locked tiles" goal, locked tiles (500-599).
#[test]
fn the_shipped_boards_spawn_powerups_and_locked_tiles() {
    // The release item table beside the campaign inventory: the shipped boards pay produce and
    // ore the campaign inventory lacks.
    let items = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/release/item-table.json");
    let fixture = Fixture::new(&[
        ("mining.json", fs::read(shipped("mining.json")).unwrap()),
        ("harvesting.json", fs::read(shipped("harvesting.json")).unwrap()),
        ("item-table.json", fs::read(items).unwrap()),
    ]);
    let mut gateway = fixture.gateway(MINER_PROFILE_ID);
    say(&mut gateway, "realm mine", 2_000);
    let level = handshake(&mut gateway, FIRST_MINING_INSTANCE, 2_100);
    let powerups = level.iter().filter(|t| t.starts_with("OnPowerupInitMsg\t95")).count();
    assert_eq!(powerups, 4, "the four clickable powerups are announced");
    assert!(spawned(&level).iter().any(|v| (950..=953).contains(v)), "a mining powerup spawns");

    let mut gateway = fixture.gateway(CHEF);
    say(&mut gateway, "realm harvest", 2_000);
    let instance = snapshot(&gateway)["harvesting"]["active"]["instance_id"].as_i64().unwrap() as i32;
    let tiles = spawned(&handshake(&mut gateway, instance, 2_100));
    assert!(tiles.iter().any(|v| (500..600).contains(v)), "a locked tile spawns");
    assert!(tiles.iter().any(|v| (950..=953).contains(v)), "a harvesting powerup spawns");
}
