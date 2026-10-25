//! W171: mining and harvesting from a real node, as an ordinary player.
//!
//! A player with dev tools off clicks (26/8) a node actor in range, plays the Type8 game the
//! node starts, wins and is paid once; the node then hides and grows back (R timers). Refusals
//! (out of range, busy, hidden, forged ids, wrong job, full bag) change no state. The board is
//! the synthetic W140 one, the node content is the repository's `data/content`.

use osfr_tools::{
    account_store::AccountRole,
    character_store::CharacterState,
    game_store,
    gameplay::{Gameplay, GameplayState},
    gateway::{Gateway, GatewayOutcome},
    harvest_nodes::{GatheringNodes, NodeActor},
    harvesting::{
        self, CHEF_PROFILE_ID, FARMING_LAUNCH_MOVIE, FARMING_PRELOAD_FILE,
        lifecycle::RECONSTRUCTED_CHEF_XP_PER_WIN,
    },
    inventory::InventoryItem,
    mining::{
        self, MINER_PROFILE_ID, MINING_GAME_ID, SIDE, Type8BoardSpec, Type8Goal, Type8Kind,
        Type8Output, Type8Phase, Type8Run, lifecycle::RECONSTRUCTED_MINER_XP_PER_WIN,
    },
    npc_interaction::NpcInteractRequest,
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
const FILLER: i32 = 1195;
const COPPER_RECIPE: i32 = 4;
const WORLD_FORGE: [f32; 3] = [-2631.933, 0.03999766, -216.5819];
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct Game {
    node: &'static str,
    json_key: &'static str,
    profile: i32,
    goals: usize,
}

const MINING: Game = Game {
    node: "Copper Vein",
    json_key: "mining",
    profile: MINER_PROFILE_ID,
    goals: 1,
};
const HARVEST: Game = Game {
    node: "Mushrooms",
    json_key: "harvesting",
    profile: CHEF_PROFILE_ID,
    goals: 4,
};
const GAMES: [Game; 2] = [MINING, HARVEST];

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
        seed: 171,
        wildcard_percent: 0,
        collectable_percent: 0,
        blocker_percent: 0,
        powerup_percent: 0,
        meter_id: 0,
        reconstructed_rules: true,
        rule_notice: "Synthetic private W171 fixture; original level tables unrecovered.".into(),
    }
}

fn manifest_path(relative: &str, variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative))
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

struct Fixture {
    game: Game,
    directory: PathBuf,
    reference: PathBuf,
    node: NodeActor,
    /// A second node of the same game, for the busy case.
    other: NodeActor,
}

impl Fixture {
    fn new(game: Game) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "osfr-gathering-nodes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        copy_dir(
            &manifest_path(
                "../../../lab/references/opening-journal-text-v2/content",
                "OSFR_TYPE8_CONTENT",
            ),
            &directory,
        );
        let mining = json!({
            "schema_version": 1, "game_id": MINING_GAME_ID, "profile_id": MINER_PROFILE_ID,
            "name_id": 7910, "description_id": 7911, "icon_id": 909, "spec": board(1),
        });
        let harvest = json!({
            "schema_version": 1, "game_id": 111, "profile_id": CHEF_PROFILE_ID,
            "name_id": 100834, "description_id": 94150, "icon_id": 908,
            "preload_file": FARMING_PRELOAD_FILE, "launch_movie": FARMING_LAUNCH_MOVIE,
            "spec": board(4),
        });
        for (file, content) in [
            (mining::CONTENT_FILE, mining),
            (harvesting::CONTENT_FILE, harvest),
        ] {
            fs::write(directory.join(file), serde_json::to_vec(&content).unwrap()).unwrap();
        }
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let nodes = GatheringNodes::load(&content).unwrap().unwrap();
        let mut named = nodes.actors().filter(|actor| actor.name == game.node);
        let node = named.next().expect("node content for the game").clone();
        let other = named.next().expect("two nodes").clone();
        Self {
            game,
            directory,
            reference: manifest_path(
                "../../../lab/references/rust-gateway-reference",
                "OSFR_GATEWAY_REFERENCE",
            ),
            node,
            other,
        }
    }

    /// Save state with the game's job unlocked and active (`active` false: left on the default).
    fn state(&self, active: bool, unlocked: bool) -> GameplayState {
        let mut state = Gameplay::load(&self.directory, None).unwrap().state();
        let profiles = state.profiles.as_mut().unwrap();
        if unlocked {
            profiles.unlocked.insert(self.game.profile);
        }
        if active {
            profiles.active = self.game.profile;
        }
        state
    }

    /// An ordinary player: dev tools off, role Player, standing `offset` metres east of `at`.
    fn gateway_at(&self, state: GameplayState, at: &NodeActor, offset: f32) -> Gateway {
        self.open(state, at, offset).0
    }

    /// Like `gateway_at`, with everything the server sent while logging in.
    fn open(
        &self,
        state: GameplayState,
        at: &NodeActor,
        offset: f32,
    ) -> (Gateway, Vec<GatewayOutcome>) {
        let mut character: CharacterState = Gateway::reference_state(&self.reference).unwrap();
        character.zone_id = 0;
        character.position = [at.position[0] + offset, at.position[1], at.position[2]];
        let mut gateway =
            Gateway::load_gameplay(&self.reference, character, &self.directory, Some(state))
                .unwrap();
        let sent = login(&mut gateway);
        (gateway, sent)
    }

    fn near(&self, state: GameplayState) -> Gateway {
        self.gateway_at(state, &self.node, 2.0)
    }

    /// Standing at the node, ready to click it: the usual start.
    fn ready(&self) -> Gateway {
        self.near(self.state(true, true))
    }

    /// Dev tools on, so the `realm mine` / `realm harvest` command reaches the
    /// shared developer gather path. The node route itself needs no dev tools.
    fn dev_near(&self, state: GameplayState) -> Gateway {
        let mut gateway = self.near(state);
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        gateway
    }

    fn with_shipped_mining(self) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/mining.json");
        fs::copy(source, self.directory.join(mining::CONTENT_FILE)).unwrap();
        self
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

fn login(gateway: &mut Gateway) -> Vec<GatewayOutcome> {
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
    vec![
        gateway.handle_at(&b, 0).unwrap(),
        send(gateway, &13u16.to_le_bytes(), 1_000),
        send(gateway, &10u16.to_le_bytes(), 1_000),
    ]
}

fn send(gateway: &mut Gateway, packet: &[u8], now_ms: u64) -> GatewayOutcome {
    gateway
        .handle_at(&wrap(packet), now_ms)
        .expect("gateway frame")
}

fn tick(gateway: &mut Gateway, now_ms: u64) -> GatewayOutcome {
    gateway.tick_at(now_ms).expect("tick")
}

/// The client's click on an actor: 26/8 with the target guid.
fn click(gateway: &mut Gateway, guid: u64, now_ms: u64) -> GatewayOutcome {
    let request = NpcInteractRequest {
        target: guid,
        trailing_flag: false,
    };
    send(gateway, &request.encode(), now_ms)
}

/// The client's movement sender `sub_934A20` fills the opcode-125 object and
/// serializer `sub_912A80` writes it as u16 125, the u64 guid, position and
/// direction (six f32), then the state and unknown bytes.
fn move_pose(gateway: &mut Gateway, position: [f32; 3], now_ms: u64) -> GatewayOutcome {
    let mut body = 125u16.to_le_bytes().to_vec();
    body.extend(1u64.to_le_bytes());
    for value in position.into_iter().chain([0.0, 0.0, 1.0]) {
        body.extend(value.to_le_bytes());
    }
    body.extend([0, 0]);
    send(gateway, &body, now_ms)
}

/// Client `RequestRecipeStart 0x00c23970` builds the request via constructor
/// `0x00c14b30`; serializer `0x00c21290` calls header writer `0x0090ac70`:
/// family 26, sub 24, then one i32 recipe id.
fn book_start(gateway: &mut Gateway, recipe: i32, now_ms: u64) -> GatewayOutcome {
    let mut body = vec![26, 0, 24, 0];
    body.extend(recipe.to_le_bytes());
    send(gateway, &body, now_ms)
}

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

const CLOSE: [u8; 4] = [26, 0, 42, 0];

fn gameplay_json(gateway: &Gateway) -> Value {
    serde_json::to_value(gateway.snapshot().gameplay.unwrap()).unwrap()
}

fn active(gateway: &Gateway, game: Game) -> Option<i64> {
    gameplay_json(gateway)[game.json_key]["active"]["instance_id"].as_i64()
}

fn wall_ms(gateway: &Gateway, game: Game) -> u64 {
    gameplay_json(gateway)[game.json_key]["active"]["wall_ms"]
        .as_u64()
        .unwrap()
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

fn xp(gateway: &Gateway, game: Game) -> u64 {
    gameplay_json(gateway)["profiles"]["experience"][game.profile.to_string()]
        .as_u64()
        .unwrap_or(0)
}

fn profile_xp(gateway: &Gateway, profile: i32) -> u64 {
    gameplay_json(gateway)["profiles"]["experience"][profile.to_string()]
        .as_u64()
        .unwrap_or(0)
}

fn coins(gateway: &Gateway) -> i64 {
    gameplay_json(gateway)["inventory"]["coins"]
        .as_i64()
        .unwrap()
}

/// Play the offered run to the win request and take the payout.
fn win(gateway: &mut Gateway, game: Game, id: i32, base_ms: u64) {
    for (step, packet) in [
        frame(5, id, -1),
        payload(id, "OnConnectMsg"),
        payload(id, "OnStartGameMsg"),
        payload(id, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(gateway, packet, base_ms + 100 * step as u64);
    }
    for slot in 0..game.goals {
        let text = format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM);
        send(gateway, &payload(id, &text), base_ms + 600 + slot as u64);
    }
    send(gateway, &payload(id, "OnNextLevelMsg"), base_ms + 700);
}

fn mining_run(gateway: &Gateway) -> Type8Run {
    gateway
        .snapshot()
        .gameplay
        .unwrap()
        .mining
        .unwrap()
        .active
        .unwrap()
}

fn extend_chain(run: &Type8Run, value: u16, path: &mut Vec<(usize, usize)>) -> bool {
    if path.len() == 3 {
        return true;
    }
    let (x, y) = *path.last().unwrap();
    for (next_x, next_y) in [
        (x.checked_sub(1), Some(y)),
        (Some(x + 1), Some(y)),
        (Some(x), y.checked_sub(1)),
        (Some(x), Some(y + 1)),
    ] {
        let (Some(next_x), Some(next_y)) = (next_x, next_y) else {
            continue;
        };
        let at = (next_x, next_y);
        let index = next_y * SIDE + next_x;
        if index >= run.board.len()
            || !(run.board[index] == value || run.board[index] == 200)
            || !run.idle[index]
            || path.contains(&at)
        {
            continue;
        }
        path.push(at);
        if extend_chain(run, value, path) {
            return true;
        }
        path.pop();
    }
    false
}

fn chain_from(run: &Type8Run, start: (usize, usize)) -> Option<Vec<(usize, usize)>> {
    let value = run.board[start.1 * SIDE + start.0];
    if !(400..500).contains(&value) || !run.idle[start.1 * SIDE + start.0] {
        return None;
    }
    let mut path = vec![start];
    extend_chain(run, value, &mut path).then_some(path)
}

fn mining_chain(run: &Type8Run) -> Vec<(usize, usize)> {
    let width = SIDE;
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            if !matches!(run.board[y * width + x], 601 | 602) {
                continue;
            }
            for below in (y + 1)..TOP + PLAYABLE {
                if let Some(path) = chain_from(run, (x, below)) {
                    return path;
                }
            }
        }
    }
    for y in TOP..TOP + PLAYABLE {
        for x in LEFT..LEFT + PLAYABLE {
            if let Some(path) = chain_from(run, (x, y)) {
                return path;
            }
        }
    }
    panic!(
        "shipped Mining18 board has no idle three-tile chain: {:?}",
        run.board
    );
}

/// Play the shipped Mining18 board through ordinary match/fall messages. The
/// board is read only to choose legal client moves; the server still owns the
/// quotas, drops and payout.
fn finish_shipped_mining(gateway: &mut Gateway, id: i32, base_ms: u64) {
    for step in 0..1000 {
        let run = mining_run(gateway);
        if run.phase == Type8Phase::LevelComplete {
            return;
        }
        let idle: Vec<(usize, usize)> = run
            .idle
            .iter()
            .enumerate()
            .filter(|(_, idle)| !**idle)
            .map(|(index, _)| (index % SIDE, index / SIDE))
            .collect();
        if !idle.is_empty() {
            let list = idle
                .iter()
                .flat_map(|(x, y)| [x.to_string(), y.to_string()])
                .collect::<Vec<_>>()
                .join(",");
            send(
                gateway,
                &payload(id, &format!("OnIdleMsg\t{list}")),
                base_ms + step * 2,
            );
            continue;
        }
        if let Some(x) =
            (LEFT..LEFT + PLAYABLE).find(|x| matches!(run.board[BOTTOM * SIDE + x], 601 | 602))
        {
            send(
                gateway,
                &payload(id, &format!("OnLevelEventMsg\t50\t{x}\t{BOTTOM}")),
                base_ms + step * 2,
            );
            continue;
        }
        let chain = mining_chain(&run);
        let list = chain
            .iter()
            .flat_map(|(x, y)| [x.to_string(), y.to_string()])
            .collect::<Vec<_>>()
            .join(",");
        send(
            gateway,
            &payload(id, &format!("OnChainMsg\t{list}")),
            base_ms + step * 2,
        );
    }
    let run = mining_run(gateway);
    panic!(
        "shipped Mining18 board did not complete its quotas: phase={:?} collected={:?} board={:?}",
        run.phase, run.collected, run.board
    );
}

#[test]
fn shipped_mining_board_pays_ore_and_xp_once_through_the_node() {
    let fixture = Fixture::new(MINING).with_shipped_mining();
    let mut gateway = fixture.ready();
    tick(&mut gateway, 1_500);
    click(&mut gateway, fixture.node.guid, 2_000);
    let id = active(&gateway, MINING).expect("shipped Mining18 run offered") as i32;

    for (step, packet) in [
        frame(5, id, -1),
        payload(id, "OnConnectMsg"),
        payload(id, "OnStartGameMsg"),
        payload(id, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(&mut gateway, packet, 2_100 + 20 * step as u64);
    }
    finish_shipped_mining(&mut gateway, id, 2_200);
    assert_eq!(mining_run(&gateway).phase, Type8Phase::LevelComplete);
    assert_eq!(count(&gateway, 2739), 0);
    assert_eq!(xp(&gateway, MINING), 0);

    send(&mut gateway, &payload(id, "OnNextLevelMsg"), 5_000);
    assert_eq!(mining_run(&gateway).phase, Type8Phase::Won);
    assert_eq!(
        count(&gateway, 2739),
        12,
        "six 601 drops plus six 602 drops"
    );
    assert_eq!(xp(&gateway, MINING), 25, "(R) Miner XP per win");
    let paid = gateway.snapshot();

    let replay = send(&mut gateway, &payload(id, "OnNextLevelMsg"), 5_100);
    assert!(replay.replies.is_empty());
    assert_eq!(gateway.snapshot(), paid, "a replayed win paid again");
}

/// The packets in `outcome` as inner family/sub pairs plus the raw bytes.
fn inners(outcome: &GatewayOutcome) -> Vec<&[u8]> {
    outcome.replies.iter().map(|r| inner(r)).collect()
}

fn has_guid(packet: &[u8], guid: u64) -> bool {
    let bytes = guid.to_le_bytes();
    packet.windows(8).any(|w| w == bytes)
}

fn added(outcome: &GatewayOutcome, guid: u64) -> bool {
    inners(outcome)
        .iter()
        .any(|p| p.starts_with(&[35, 0, 2, 0]) && has_guid(p, guid))
}

fn removed(outcome: &GatewayOutcome, guid: u64) -> bool {
    inners(outcome)
        .iter()
        .any(|p| p.starts_with(&[35, 0, 3, 0]) && has_guid(p, guid))
}

fn said(outcome: &GatewayOutcome, needle: &str) -> bool {
    inners(outcome)
        .iter()
        .any(|p| String::from_utf8_lossy(p).contains(needle))
}

/// The exact `chat::system` lines in `outcome`, one per line.
fn chat_text(outcome: &GatewayOutcome) -> String {
    let mut text = String::new();
    for packet in inners(outcome) {
        if packet.starts_with(&[15, 0, 3, 0]) && packet.len() >= 8 {
            let length = u32::from_le_bytes(packet[4..8].try_into().unwrap()) as usize;
            text.push_str(std::str::from_utf8(&packet[8..8 + length]).unwrap());
            text.push('\n');
        }
    }
    text
}

fn fill_inventory(state: &mut GameplayState) {
    let items = &mut state.inventory.items;
    let mut next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    while items.len() < 4096 {
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

fn stock_copper_recipe(state: &mut GameplayState) {
    let items = &mut state.inventory.items;
    let mut next = items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
    for (definition_id, count) in [(2739, 5), (879, 3)] {
        items.push(InventoryItem {
            definition_id,
            tint: 0,
            id: next,
            count,
            consumed_count: 0,
            stored_count: 0,
            last_cast_time: 0,
            rental: None,
            ability_count: 0,
            activate_enabled: true,
        });
        next += 1;
    }
    state.inventory.next_instance_id = Some(i64::from(next));
}

#[test]
fn an_ordinary_player_gathers_from_a_node_is_paid_once_and_the_node_grows_back() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let mut gateway = fixture.ready();
        let guid = fixture.node.guid;

        // The node is drawn for this player and dev tools are off.
        tick(&mut gateway, 1_500);
        let started = click(&mut gateway, guid, 2_000);
        let id =
            active(&gateway, game).unwrap_or_else(|| panic!("{} run offered", game.node)) as i32;
        assert!(!inners(&started).is_empty(), "{}", game.node);
        assert_eq!(ore(&gateway, game), vec![0; game.goals]);
        let coins_before = coins(&gateway);

        win(&mut gateway, game, id, 2_100);
        assert_eq!(
            ore(&gateway, game),
            vec![1; game.goals],
            "{} paid",
            game.node
        );
        let reward_xp = match game.profile {
            MINER_PROFILE_ID => RECONSTRUCTED_MINER_XP_PER_WIN,
            CHEF_PROFILE_ID => RECONSTRUCTED_CHEF_XP_PER_WIN,
            _ => unreachable!(),
        };
        assert_eq!(
            xp(&gateway, game),
            u64::from(reward_xp),
            "{} (R) XP",
            game.node
        );
        let other = if game.profile == MINER_PROFILE_ID {
            CHEF_PROFILE_ID
        } else {
            MINER_PROFILE_ID
        };
        assert_eq!(profile_xp(&gateway, other), 0, "{} other job XP", game.node);
        assert_eq!(coins(&gateway), coins_before, "{} coins", game.node);

        // The node stays until the results close, then hides.
        let waiting = tick(&mut gateway, 3_000);
        assert!(!removed(&waiting, guid), "{}", game.node);
        send(&mut gateway, &CLOSE, 3_100);
        assert!(active(&gateway, game).is_none());
        let gone = tick(&mut gateway, 3_200);
        assert!(removed(&gone, guid), "{} hides", game.node);

        // A paid node cannot be gathered again until it returns; the click changes nothing.
        let before = gateway.snapshot();
        let refused = click(&mut gateway, guid, 4_000);
        assert!(said(&refused, "already been gathered"), "{}", game.node);
        assert_eq!(gateway.snapshot(), before);
        assert_eq!(ore(&gateway, game), vec![1; game.goals]);

        // Not back before its time (60 s from the hide), back after it.
        assert!(!added(&tick(&mut gateway, 63_000), guid));
        let back = tick(&mut gateway, 63_300);
        assert!(added(&back, guid), "{} returns", game.node);
        click(&mut gateway, guid, 63_400);
        let second = active(&gateway, game).unwrap() as i32;
        assert_eq!(second, id + 1, "{} fresh instance", game.node);
        win(&mut gateway, game, second, 63_500);
        assert_eq!(ore(&gateway, game), vec![2; game.goals]);
        assert_eq!(xp(&gateway, game), u64::from(reward_xp) * 2);
        assert_eq!(profile_xp(&gateway, other), 0);
        assert_eq!(coins(&gateway), coins_before);
    }
}

#[test]
fn a_player_standing_at_a_node_is_sent_its_actor_and_not_a_far_ones() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (_, sent) = fixture.open(fixture.state(true, true), &fixture.node, 2.0);
        let drawn = |guid| sent.iter().any(|outcome| added(outcome, guid));
        assert!(drawn(fixture.node.guid), "{} drawn at login", game.node);
        let (_, sent) = fixture.open(fixture.state(true, true), &fixture.node, 900.0);
        let drawn = |guid| sent.iter().any(|outcome| added(outcome, guid));
        assert!(
            !drawn(fixture.node.guid),
            "{} not drawn from afar",
            game.node
        );
    }
}

#[test]
fn quit_and_reconnect_before_a_win_pay_nothing_and_the_node_stays() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let mut gateway = fixture.ready();
        let guid = fixture.node.guid;
        click(&mut gateway, guid, 2_000);
        let id = active(&gateway, game).unwrap() as i32;
        send(&mut gateway, &frame(6, id, -1), 2_100); // Quit
        assert!(active(&gateway, game).is_none());
        assert_eq!(ore(&gateway, game), vec![0; game.goals]);
        assert_eq!(xp(&gateway, game), 0);
        assert!(
            !removed(&tick(&mut gateway, 2_200), guid),
            "{} stays",
            game.node
        );

        // The same node starts a fresh run at once; a reconnect mid-run forfeits it.
        click(&mut gateway, guid, 2_300);
        assert_eq!(active(&gateway, game), Some(i64::from(id) + 1));
        let snapshot = gateway.snapshot();
        let path = fixture.directory.join("w171-save.json");
        game_store::save(&path, &snapshot).unwrap();
        let loaded = game_store::load_or_create(&path, snapshot.character.clone()).unwrap();
        let mut again = fixture.gateway_at(loaded.gameplay.unwrap(), &fixture.node, 2.0);
        assert!(active(&again, game).is_none(), "{} forfeited", game.node);
        assert_eq!(ore(&again, game), vec![0; game.goals]);
        click(&mut again, guid, 2_000);
        assert!(active(&again, game).is_some(), "{} node intact", game.node);
    }
}

#[test]
fn refusals_leave_no_trace() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let guid = fixture.node.guid;

        // Out of range (a 40 m walk away).
        let mut far = fixture.gateway_at(fixture.state(true, true), &fixture.node, 40.0);
        let before = far.snapshot();
        let out = click(&mut far, guid, 2_000);
        assert!(said(&out, "closer"), "{}", game.node);
        assert_eq!(far.snapshot(), before);

        // Job not unlocked, or unlocked but not the active job.
        let mut none = fixture.near(fixture.state(false, false));
        let before = none.snapshot();
        assert!(said(&click(&mut none, guid, 2_000), "You need to be a"));
        assert_eq!(none.snapshot(), before);
        let mut wrong = fixture.near(fixture.state(false, true));
        let before = wrong.snapshot();
        assert!(said(&click(&mut wrong, guid, 2_000), "Switch to the"));
        assert_eq!(wrong.snapshot(), before);

        // Busy: a run is under way, so a second node (even in range) and a repeat click do nothing.
        let mut gateway = fixture.ready();
        click(&mut gateway, guid, 2_000);
        let id = active(&gateway, game).unwrap();
        let before = gateway.snapshot();
        for target in [guid, fixture.other.guid] {
            let out = click(&mut gateway, target, 2_100);
            assert!(said(&out, "Finish what"), "{}", game.node);
            assert_eq!(gateway.snapshot(), before);
        }
        assert_eq!(active(&gateway, game), Some(id));

        // Forged ids: other tags, other spawn ids, zero, max, a neighbouring bit. Nothing starts.
        let mut gateway = fixture.ready();
        let before = gateway.snapshot();
        for forged in [
            0,
            u64::MAX,
            guid ^ (1 << 40),
            guid ^ (1 << 63),
            guid + 1_000_000,
            (guid & !0xffff_ffff) | 0xffff_fff0,
        ] {
            click(&mut gateway, forged, 2_000);
            assert_eq!(
                gateway.snapshot(),
                before,
                "{} forged {forged:#x}",
                game.node
            );
        }
        // Malformed requests on a real node id: dropped (or refused) with no state change.
        let mut bad = NpcInteractRequest {
            target: guid,
            trailing_flag: false,
        }
        .encode();
        bad[12] = 2;
        for packet in [
            bad.clone(),
            bad[..12].to_vec(),
            [bad.clone(), vec![0]].concat(),
        ] {
            let _ = gateway.handle_at(&wrap(&packet), 2_100);
            assert_eq!(gateway.snapshot(), before, "{} malformed", game.node);
        }

        // The ordinary route does not open the developer commands.
        let before = gateway.snapshot();
        let command = if game.json_key == "mining" {
            "realm mine"
        } else {
            "realm harvest"
        };
        send(&mut gateway, &chat_inner(command), 2_200);
        assert_eq!(gateway.snapshot(), before, "{} dev command", game.node);
        assert!(active(&gateway, game).is_none());
    }
}

#[test]
fn a_full_bag_refuses_the_start_instead_of_stranding_the_win() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let mut state = fixture.state(true, true);
        fill_inventory(&mut state);
        let mut gateway = fixture.near(state);
        let before = gateway.snapshot();
        let before_bytes = serde_json::to_vec(&before).unwrap();
        let out = click(&mut gateway, fixture.node.guid, 2_000);
        assert_eq!(
            chat_text(&out).trim_end(),
            "Your bag is full. Make room before gathering.",
            "{}",
            game.node
        );
        assert_eq!(gateway.snapshot(), before);
        assert_eq!(
            serde_json::to_vec(&gateway.snapshot()).unwrap(),
            before_bytes,
            "{}: the full-bag refusal changed the save bytes",
            game.node
        );
        assert!(active(&gateway, game).is_none());
    }
}

#[test]
fn begin_gather_stamps_the_clock_only_after_begin() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let mut gateway = fixture.ready();
        let guid = fixture.node.guid;

        tick(&mut gateway, 1_500);
        click(&mut gateway, guid, 2_000);
        assert!(active(&gateway, game).is_some(), "{}", game.node);
        assert_eq!(
            wall_ms(&gateway, game),
            0,
            "{}: a start must not tick the run",
            game.node
        );

        // The 250 ms movie step is due only because begin_gather stamped the
        // shared clock after the engine accepted the run.
        tick(&mut gateway, 2_100);
        assert_eq!(
            wall_ms(&gateway, game),
            0,
            "{}: a not-due tick advanced the run",
            game.node
        );
        tick(&mut gateway, 2_250);
        assert_eq!(
            wall_ms(&gateway, game),
            250,
            "{}: the due tick did not run",
            game.node
        );
    }
}

#[test]
fn unlocked_but_inactive_job_is_refused_on_both_paths() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let guid = fixture.node.guid;
        let (word, noun) = if game.json_key == "mining" {
            ("Miner", "mining")
        } else {
            ("Chef", "harvesting")
        };

        // The node admission names the job.
        let mut gateway = fixture.near(fixture.state(false, true));
        let before = gateway.snapshot();
        let node = click(&mut gateway, guid, 2_000);
        assert_eq!(
            chat_text(&node).trim_end(),
            format!("Switch to the {word} job to gather that."),
            "{}",
            game.node
        );
        assert_eq!(gateway.snapshot(), before);

        // The developer command names the same job and its own test.
        let mut gateway = fixture.dev_near(fixture.state(false, true));
        let before = gateway.snapshot();
        let command = if game.json_key == "mining" {
            "realm mine"
        } else {
            "realm harvest"
        };
        let dev = send(&mut gateway, &chat_inner(command), 2_000);
        assert_eq!(
            chat_text(&dev).trim_end(),
            format!("Switch to the {word} job before starting the {noun} test."),
            "{}",
            game.node
        );
        assert_eq!(gateway.snapshot(), before);
        assert!(active(&gateway, game).is_none());
    }
}

#[test]
fn a_refused_developer_gather_and_node_click_change_nothing() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let guid = fixture.node.guid;
        let command = if game.json_key == "mining" {
            "realm mine"
        } else {
            "realm harvest"
        };

        for unlocked in [false, true] {
            let mut gateway = fixture.dev_near(fixture.state(false, unlocked));
            let before = gateway.snapshot();
            let before_bytes = serde_json::to_vec(&before).unwrap();

            // A node click refused by the node admission changes nothing, including
            // the save bytes.
            let refused = click(&mut gateway, guid, 2_000);
            assert!(
                !refused.gameplay_changed,
                "{}: node refusal saved",
                game.node
            );
            assert_eq!(gateway.snapshot(), before, "{}: node refusal", game.node);
            assert_eq!(
                serde_json::to_vec(&gateway.snapshot()).unwrap(),
                before_bytes,
                "{}: node refusal bytes",
                game.node
            );

            // The developer command is refused by the same eligibility and also
            // changes nothing.
            let refused = send(&mut gateway, &chat_inner(command), 2_100);
            assert!(
                !refused.gameplay_changed,
                "{}: command refusal saved",
                game.node
            );
            assert_eq!(gateway.snapshot(), before, "{}: command refusal", game.node);
            assert_eq!(
                serde_json::to_vec(&gateway.snapshot()).unwrap(),
                before_bytes,
                "{}: command refusal bytes",
                game.node
            );
            assert!(active(&gateway, game).is_none(), "{}", game.node);
        }
    }
}

#[test]
fn an_unclosed_mining_run_refuses_recipe_start_without_a_save() {
    let fixture = Fixture::new(MINING);
    let mut state = fixture.state(true, true);
    stock_copper_recipe(&mut state);
    let mut gateway = fixture.gateway_at(state, &fixture.node, 2.0);

    click(&mut gateway, fixture.node.guid, 2_000);
    assert!(active(&gateway, MINING).is_some());

    let near_node = [
        fixture.node.position[0] + 1.0,
        fixture.node.position[1],
        fixture.node.position[2],
    ];
    move_pose(&mut gateway, near_node, 2_100);
    move_pose(&mut gateway, WORLD_FORGE, 600_000);
    assert_eq!(gateway.snapshot().character.position, WORLD_FORGE);

    let before = gateway.snapshot();
    let refused = book_start(&mut gateway, COPPER_RECIPE, 600_100);
    assert!(said(&refused, "Finish what"));
    assert!(!refused.state_changed, "refusal must not request a save");
    assert!(
        !refused.gameplay_changed,
        "refusal must not persist gameplay"
    );
    assert_eq!(gateway.snapshot(), before);
    assert!(active(&gateway, MINING).is_some());
}

#[test]
fn node_content_is_bounded_and_unique() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let nodes = GatheringNodes::load(&content).unwrap().unwrap();
    let actors: Vec<_> = nodes.actors().collect();
    // 12 + 2 mushroom nodes (pool maxima), 4 copper veins and 5 quest-only tin-ore
    // pickups (T503, `quest_nodes.json`).
    assert_eq!(actors.len(), 23);
    let mut ids: Vec<_> = actors.iter().map(|a| a.guid).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 23);
    assert!(
        actors
            .iter()
            .all(|a| a.position.iter().all(|v| v.is_finite()))
    );
    assert!(nodes.is_node(actors[0].guid) && !nodes.is_node(0));
}
