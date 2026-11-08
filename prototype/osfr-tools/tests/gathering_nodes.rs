//! W171: mining and harvesting from a real node, as an ordinary player.
//!
//! A player with dev tools off clicks (26/8) a node actor in range, plays the Type8 game the
//! node starts, wins and is paid once; the node then hides and grows back (R timers). Refusals
//! (out of range, busy, hidden, forged ids, wrong job, full bag) change no state. The board is
//! the synthetic W140 one, the node content is the repository's `data/content`.

#[path = "support/json.rs"]
mod json;
#[path = "support/node_content.rs"]
mod node_content;

use osfr_tools::{
    account_store::AccountRole,
    activity_portal::{MISSING_JOB_TEXT, join_error},
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
    artifact_hunt, mail_sorting,
    mining::{
        self, MINER_PROFILE_ID, MINING_GAME_ID, Type8BoardSpec, Type8Goal, Type8Kind,
        Type8Output, Type8Phase, Type8Run, lifecycle::RECONSTRUCTED_MINER_XP_PER_WIN,
    },
    npc_interaction::NpcInteractRequest,
    startup_world::parse_add_npc,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

// The shipped Copper board: ten by ten on the 13 by 13 grid.
const SIDE: usize = 13;
const LEFT: usize = 2;
const TOP: usize = 1;
const PLAYABLE: usize = 10;
const BOTTOM: usize = TOP + PLAYABLE - 1;
/// Manifest-defined stand-ins that exist in the private campaign inventory.
const OUTPUTS: [i32; 4] = [2739, 879, 65834, 2436];
/// T553: the regional Harvesting games the farm plots start, with disjoint manifest-defined
/// stand-ins for their produce (the real bundle definitions are not all in the private campaign).
const REGION_OUTPUTS: [(i32, [i32; 4]); 3] = [
    (109, [2875, 2876, 2877, 2880]),
    (115, [2882, 2884, 2886, 2887]),
    (117, [2889, 1194, 1196, 3941]),
];
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
    /// The produce definitions the game pays, in goal order.
    outputs: [i32; 4],
    /// T553: the Harvesting game a farm plot of this name starts; `None` for every other node.
    region: Option<i32>,
}

const MINING: Game = Game {
    node: "Copper Vein",
    json_key: "mining",
    profile: MINER_PROFILE_ID,
    goals: 1,
    outputs: OUTPUTS,
    region: None,
};
const HARVEST: Game = Game {
    node: "Mushrooms",
    json_key: "harvesting",
    profile: CHEF_PROFILE_ID,
    goals: 4,
    outputs: OUTPUTS,
    region: None,
};
/// T553: a Queensfields farm plot. The Lakesfield and Perry's Pastures plots are covered by
/// `each_farm_plot_starts_its_own_region_game_and_pays_its_own_produce`.
const PLOT: Game = Game {
    node: "Harvesting",
    json_key: "harvesting",
    profile: CHEF_PROFILE_ID,
    goals: 4,
    outputs: REGION_OUTPUTS[0].1,
    region: Some(REGION_OUTPUTS[0].0),
};
const GAMES: [Game; 3] = [MINING, HARVEST, PLOT];

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
        let spec = board(4);
        let games: Vec<Value> = REGION_OUTPUTS
            .iter()
            .map(|(id, outputs)| {
                let region = harvesting::region(*id).unwrap();
                json!({
                    "game_id": id, "name_id": region.name_id,
                    "description_id": region.description_id, "icon_id": region.icon_id,
                    "selectable": spec.selectable, "tiles": spec.tiles, "goals": spec.goals,
                    "outputs": VALUES.iter().zip(outputs).map(|(value, definition_id)| json!({
                        "value": value, "definition_id": definition_id, "cluster_quantity": 1,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let harvest = json!({
            "schema_version": 1, "game_id": 111, "profile_id": CHEF_PROFILE_ID,
            "name_id": 100834, "description_id": 94150, "icon_id": 908,
            "preload_file": FARMING_PRELOAD_FILE, "launch_movie": FARMING_LAUNCH_MOVIE,
            "games": games, "spec": spec,
        });
        for (file, content) in [
            (mining::CONTENT_FILE, mining),
            (harvesting::CONTENT_FILE, harvest),
        ] {
            fs::write(directory.join(file), serde_json::to_vec(&content).unwrap()).unwrap();
        }
        let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
        let nodes = GatheringNodes::load(&content).unwrap().unwrap();
        let named: Vec<_> = nodes
            .actors()
            .filter(|actor| actor.name == game.node)
            .collect();
        // A plot is the one of the game's own region; any other node of the name is the spare.
        let node = (*named
            .iter()
            .find(|actor| game.region.is_none() || nodes.game_id(actor.guid) == game.region)
            .expect("node content for the game"))
        .clone();
        let other = (*named
            .iter()
            .find(|actor| actor.guid != node.guid)
            .expect("two nodes"))
        .clone();
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

    /// A campaign whose `harvesting.json` predates T553: no `games`, so no plot game is offered.
    fn without_plot_games(self) -> Self {
        let path = self.directory.join(harvesting::CONTENT_FILE);
        let mut content = json::read(&path);
        content.as_object_mut().unwrap().remove("games").unwrap();
        json::write_pretty(&path, &content);
        self
    }

    /// L927: the release `job_xp_curve.json` beside the content, so a win pays the job's `activity_xp`.
    fn with_activity_rates(self) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release/job_xp_curve.json");
        fs::copy(source, self.directory.join("job_xp_curve.json")).unwrap();
        self
    }

    fn with_shipped_mining(self) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/mining.json");
        fs::copy(source, self.directory.join(mining::CONTENT_FILE)).unwrap();
        self
    }

    /// L934: the shipped regions, Chugaroot Farm (113) and Thorny Tillage (119) among them.
    fn with_shipped_harvesting(self) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/harvesting.json");
        fs::copy(source, self.directory.join(harvesting::CONTENT_FILE)).unwrap();
        self
    }

    /// L934: the release item table beside the campaign inventory, which lacks the iron, silver
    /// and gold ore and some produce the shipped boards pay.
    fn with_release_items(self) -> Self {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/release/item-table.json");
        fs::copy(source, self.directory.join("item-table.json")).unwrap();
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
    game.outputs[..game.goals]
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

/// The next level of a run `win` has just finished one level of.
fn win_next_level(gateway: &mut Gateway, game: Game, id: i32, base_ms: u64) -> GatewayOutcome {
    send(gateway, &payload(id, "OnStartLevelMsg"), base_ms);
    for slot in 0..game.goals {
        let text = format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM);
        send(gateway, &payload(id, &text), base_ms + 600 + slot as u64);
    }
    send(gateway, &payload(id, "OnNextLevelMsg"), base_ms + 700)
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
    if !run.idle[start.1 * SIDE + start.0] {
        return None;
    }
    // A wildcard takes the colour of what follows, as the player's chain does.
    let colours = match value {
        200 => 400..500,
        400..500 => value..value + 1,
        _ => return None,
    };
    colours.into_iter().find_map(|colour| {
        let mut path = vec![start];
        extend_chain(run, colour, &mut path).then_some(path)
    })
}

fn mining_chain(run: &Type8Run) -> Vec<(usize, usize)> {
    let width = SIDE;
    for y in 0..SIDE {
        for x in 0..SIDE {
            if !matches!(run.board[y * width + x], 600..800) {
                continue;
            }
            for below in (y + 1)..SIDE {
                if let Some(path) = chain_from(run, (x, below)) {
                    return path;
                }
            }
        }
    }
    for y in 0..SIDE {
        for x in 0..SIDE {
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
    for step in 0..6000 {
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
        // A shaped board's columns end on different rows, each above its own dropzone.
        let foot = |x: usize| (0..SIDE - 1).find(|y| run.board[(y + 1) * SIDE + x] == 100);
        let falling = (0..SIDE).find_map(|x| {
            let y = foot(x)?;
            matches!(run.board[y * SIDE + x], 600..800).then_some((x, y))
        });
        if let Some((x, y)) = falling {
            send(
                gateway,
                &payload(id, &format!("OnLevelEventMsg\t50\t{x}\t{y}")),
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
    // Six 601 drops and six 602 drops, and a tile that falls past a met quota still pays.
    assert!(count(&gateway, 2739) >= 12);
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

/// T553: a farm plot's AddNpc carries its terrain object id and does not replace the terrain
/// (client `00968c80` then adopts the terrain's own plot, so it draws once); every other node
/// sends no terrain id.
#[test]
fn a_farm_plot_binds_to_its_terrain_object_and_other_nodes_do_not() {
    let file = json::read(&node_content::content_dir().join("harvest_nodes.json"));
    for game in GAMES {
        let fixture = Fixture::new(game);
        let (_, sent) = fixture.open(fixture.state(true, true), &fixture.node, 2.0);
        let guid = fixture.node.guid;
        let spawn = sent
            .iter()
            .flat_map(inners)
            .filter(|p| p.starts_with(&[35, 0, 2, 0]) && has_guid(p, guid))
            .filter_map(|p| parse_add_npc(p).ok())
            .find(|spawn| spawn.actor_id == guid)
            .expect("the node's AddNpc");
        let row = file["spawns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == guid & 0xffff_ffff);
        let expected = row
            .and_then(|row| row["terrain_object_id"].as_i64())
            .unwrap_or(0);
        assert_eq!(
            i64::from(spawn.terrain_object_id),
            expected,
            "{}",
            game.node
        );
        assert_eq!(spawn.replace_terrain_object, 0, "{}", game.node);
        assert_eq!(expected != 0, game.region.is_some(), "{}", game.node);
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

        // Job not unlocked. L936: an unlocked job that is not the active one is no refusal any
        // more (the click switches into it): see `unlocked_but_inactive_job_switches_on_a_node_*`.
        let mut none = fixture.near(fixture.state(false, false));
        let before = none.snapshot();
        assert!(said(&click(&mut none, guid, 2_000), "You need to be a"));
        assert_eq!(none.snapshot(), before);

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

/// L936 (client 76669, "you'll automatically switch into the necessary Job"): a node click from an
/// unlocked-but-inactive job switches into the node's job and starts the run; the developer
/// command keeps its own "Switch to the X job" refusal.
#[test]
fn unlocked_but_inactive_job_switches_on_a_node_click_and_the_dev_command_refuses() {
    for game in GAMES {
        let fixture = Fixture::new(game);
        let guid = fixture.node.guid;
        let (word, noun) = if game.json_key == "mining" {
            ("Miner", "mining")
        } else {
            ("Chef", "harvesting")
        };

        // The node click switches into the job (38/21 activation) and starts the run.
        let mut gateway = fixture.near(fixture.state(false, true));
        assert_ne!(
            gateway.gameplay().unwrap().active_profile(),
            Some(game.profile),
            "{}",
            game.node
        );
        let node = click(&mut gateway, guid, 2_000);
        assert!(node.gameplay_changed, "{}", game.node);
        assert_eq!(
            gateway.gameplay().unwrap().active_profile(),
            Some(game.profile),
            "{}",
            game.node
        );
        assert!(
            inners(&node).iter().any(|p| p.starts_with(&[38, 0, 21, 0])),
            "{}: the job switch sends an activation",
            game.node
        );
        let at = |head: &[u8]| inners(&node).iter().position(|p| p.starts_with(head)).unwrap();
        assert!(
            at(&[39, 0, 16]) < at(&[38, 0, 21, 0]),
            "{}: the offer goes first, or the start screen waits and may never draw",
            game.node
        );
        assert!(!said(&node, "Switch to the"), "{}", chat_text(&node));
        assert!(active(&gateway, game).is_some(), "{}", game.node);
        assert_eq!(
            gameplay_json(&gateway)["profiles"]["active"],
            game.profile,
            "{}: the save records the new job",
            game.node
        );

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

/// L936: the developer command stays refused for a missing or inactive job and changes nothing;
/// a node click is refused (and changes nothing) only without the job unlocked. With the job
/// unlocked but inactive it switches into the job and starts the run instead.
#[test]
fn a_refused_developer_gather_and_a_jobless_node_click_change_nothing() {
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

            // The developer command is refused by the same eligibility and changes nothing,
            // including the save bytes.
            let refused = send(&mut gateway, &chat_inner(command), 2_000);
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

            if unlocked {
                // L936: an unlocked job is admitted and switched into, not refused.
                let started = click(&mut gateway, guid, 2_100);
                assert!(started.gameplay_changed, "{}: switch saved", game.node);
                assert_eq!(
                    gateway.gameplay().unwrap().active_profile(),
                    Some(game.profile),
                    "{}",
                    game.node
                );
                assert!(active(&gateway, game).is_some(), "{}", game.node);
                continue;
            }

            // A node click without the job unlocked changes nothing, including the save bytes.
            let refused = click(&mut gateway, guid, 2_100);
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

/// The first T550 harvest plant: its actor guid and position. The row is content data, so the
/// test follows a shipped row rather than hard-coding one region's coordinates.
fn t550_plant() -> (u64, [f32; 3]) {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let extras: Value =
        serde_json::from_slice(&fs::read(content.join("npc_spawns_extra.json")).unwrap()).unwrap();
    let row = extras["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["model_file_name"]
                .as_str()
                .is_some_and(|model| model.to_ascii_lowercase().contains("harvestable"))
                && row["source"].as_array().is_some_and(|sources| {
                    sources.iter().any(|source| {
                        source["evidence"]
                            .as_str()
                            .is_some_and(|evidence| evidence.starts_with("(R) T550:"))
                    })
                })
        })
        .expect("a T550 plant row");
    let position: [f32; 3] = serde_json::from_value(row["position"].clone()).unwrap();
    (row["spawn_guid"].as_u64().unwrap(), position)
}

#[test]
fn a_plant_click_opens_nothing_and_changes_nothing() {
    let (guid, at) = t550_plant();
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let nodes = GatheringNodes::load(&content).unwrap().unwrap();
    assert!(!nodes.is_node(guid), "a harvest plant became a gather node");

    let fixture = Fixture::new(MINING);
    let mut gateway = fixture.near(fixture.state(true, true));
    // Stand on the plant; the click is the same 26/8 an ordinary player sends
    // (client constructor 0x008b6bc0 stores the u64 target and one trailing byte).
    move_pose(&mut gateway, at, 2_000);
    let before = gateway.snapshot();
    let before_bytes = serde_json::to_vec(&before).unwrap();
    let clicked = click(&mut gateway, guid, 2_100);
    assert!(clicked.replies.is_empty(), "a decoration click replied");
    assert!(!clicked.gameplay_changed, "a decoration click persisted");
    assert_eq!(gateway.snapshot(), before, "a plant click changed state");
    assert_eq!(
        serde_json::to_vec(&gateway.snapshot()).unwrap(),
        before_bytes,
        "a plant click changed the save bytes"
    );
    assert!(active(&gateway, MINING).is_none());
    assert!(active(&gateway, HARVEST).is_none());
}

#[test]
fn node_content_is_bounded_and_unique() {
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let nodes = GatheringNodes::load(&content).unwrap().unwrap();
    let actors: Vec<_> = nodes.actors().collect();
    // 12 + 2 mushroom nodes (pool maxima), 4 copper veins, 5 quest-only tin-ore pickups
    // (T503, `quest_nodes.json`), 21 T539 quest pickups (5 hammers, 5 chisels, 3 rocks,
    // 8 homework pages), 166 placed collection nodes (T524, the 34 pool maxima) and the
    // 8 Snowhill copper veins T547 places from the 17 recovered rows and the 13 T553 farm
    // plots (every recovered plot is active), and 219 spots for the quest pick-ups (with the Chapter 1 caravan)
    // (`quest_wire.py`: up to count + 2 each, on a ring round the giver).
    assert_eq!(actors.len(), 450);
    let mut ids: Vec<_> = actors.iter().map(|a| a.guid).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 450);
    assert!(
        actors
            .iter()
            .all(|a| a.position.iter().all(|v| v.is_finite()))
    );
    assert!(nodes.is_node(actors[0].guid) && !nodes.is_node(0));
}

/// The game a started run advertises: the 39/16 info packet's header game id, then its name and
/// description ids (`mining.rs` `info_packet`).
fn started_game(outcome: &GatewayOutcome) -> (i32, i32, i32) {
    let info = inners(outcome)
        .into_iter()
        .find(|packet| packet.starts_with(&[39, 0, 16]))
        .expect("a 39/16 info packet");
    let at = |offset: usize| i32::from_le_bytes(info[offset..offset + 4].try_into().unwrap());
    (at(11), at(15), at(23))
}

fn mushroom() -> NodeActor {
    node_content::shipped_nodes()
        .actors()
        .find(|actor| actor.name == "Mushrooms")
        .expect("a mushroom node")
        .clone()
}

/// The client's Games window Play (167/1/2): the live row id, then the clock dword it never
/// trusts.
fn play(gateway: &mut Gateway, row: u32, now_ms: u64) -> GatewayOutcome {
    let mut packet = vec![167, 0, 1, 2];
    packet.extend(row.to_le_bytes());
    packet.extend(0_u32.to_le_bytes());
    send(gateway, &packet, now_ms)
}

/// L100: Play on a live harvest row starts that row's game 60 m from every node, under the
/// node's own job lines, and pays its produce; a live row no engine offers stays closed. With
/// the shipped `mining.json`, Copper Mining (row 400) starts game 91 on the copper board.
/// L936: Play from an unlocked-but-inactive job switches into it and starts the game; Play without
/// the job unlocked is the refusal that changes nothing.
#[test]
fn games_window_play_starts_the_rows_gather_game_away_from_every_node() {
    let fixture = Fixture::new(PLOT);
    let mut none = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    let before = none.snapshot();
    let refused = play(&mut none, 410, 2_000);
    assert!(
        inners(&refused).contains(&join_error(MISSING_JOB_TEXT).as_slice()),
        "B4: the client's own missing-job alert (147/2 116214)"
    );
    assert_eq!(none.snapshot(), before, "a refused Play changes nothing");

    let mut idle = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    assert_ne!(
        idle.gameplay().unwrap().active_profile(),
        Some(CHEF_PROFILE_ID)
    );
    tick(&mut idle, 1_500);
    let switched = play(&mut idle, 410, 2_000);
    assert!(
        !said(&switched, "Switch to the"),
        "{}",
        chat_text(&switched)
    );
    assert_eq!(
        idle.gameplay().unwrap().active_profile(),
        Some(CHEF_PROFILE_ID)
    );
    assert!(
        inners(&switched)
            .iter()
            .any(|p| p.starts_with(&[38, 0, 21, 0])),
        "the job switch sends an activation"
    );
    assert_eq!(started_game(&switched).0, 109);
    assert!(active(&idle, PLOT).is_some(), "the row's run is offered");

    let mut gateway = fixture.gateway_at(fixture.state(true, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let started = play(&mut gateway, 410, 2_000);
    let queensfields = harvesting::region(109).unwrap();
    assert_eq!(
        started_game(&started),
        (109, queensfields.name_id, queensfields.description_id)
    );
    let id = active(&gateway, PLOT).expect("the row's run is offered") as i32;
    win(&mut gateway, PLOT, id, 2_100);
    assert_eq!(ore(&gateway, PLOT), vec![1; 4], "game 109 produce");
    send(&mut gateway, &CLOSE, 3_100);
    let closed = play(&mut gateway, 412, 3_200);
    assert!(said(&closed, "not open yet"), "{}", chat_text(&closed));

    let fixture = Fixture::new(MINING).with_shipped_mining();
    let mut gateway = fixture.gateway_at(fixture.state(true, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let started = play(&mut gateway, 400, 2_000);
    assert_eq!(started_game(&started), (91, 16221, 7911));
    assert!(
        active(&gateway, MINING).is_some(),
        "Copper Mining run offered"
    );
    // L934: Tin Mining (row 401) is open now, so a run in progress refuses it as busy.
    let tin = play(&mut gateway, 401, 2_100);
    assert!(
        said(&tin, "Finish what you are doing"),
        "{}",
        chat_text(&tin)
    );
}

/// Artifact Hunt, Games window row 413 (game 57): an arcade row, so Play neither asks for a job
/// nor changes the one the player is in, and a finished hunt pays that job once and no item.
#[test]
fn games_window_play_hunts_artifacts_in_any_job() {
    const POSTMAN: i32 = mail_sorting::POSTMAN_PROFILE_ID;
    let hunt = Game {
        node: "",
        json_key: "artifact_hunt",
        profile: artifact_hunt::ADVENTURER_PROFILE_ID,
        goals: 4,
        outputs: OUTPUTS,
        region: None,
    };
    let fixture = Fixture::new(PLOT).with_activity_rates();
    let mut spec = serde_json::to_value(board(4)).unwrap();
    spec["outputs"] = json!([]);
    let content = json!({
        "schema_version": 1, "game_id": 57, "profile_id": artifact_hunt::ADVENTURER_PROFILE_ID,
        "name_id": 35604, "description_id": 35605, "icon_id": 912,
        "preload_file": "ArtifactAssets.txt", "launch_movie": "game_artifact.gfx", "spec": spec,
    });
    let path = fixture.directory.join(artifact_hunt::CONTENT_FILE);
    fs::write(path, serde_json::to_vec(&content).unwrap()).unwrap();
    let mut state = fixture.state(false, false);
    let profiles = state.profiles.as_mut().unwrap();
    profiles.unlocked.insert(POSTMAN);
    profiles.active = POSTMAN;
    let mut gateway = fixture.gateway_at(state, &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let started = play(&mut gateway, 413, 2_000);
    assert_eq!(started_game(&started), (57, 35604, 35605));
    assert_eq!(gateway.gameplay().unwrap().active_profile(), Some(POSTMAN));
    let id = active(&gateway, hunt).expect("the hunt is offered") as i32;
    let (xp, items) = (
        profile_xp(&gateway, POSTMAN),
        gameplay_json(&gateway)["inventory"].clone(),
    );
    win(&mut gateway, hunt, id, 2_100);
    let paid = artifact_hunt::RECONSTRUCTED_XP_PER_WIN;
    assert_eq!(profile_xp(&gateway, POSTMAN), xp + u64::from(paid));
    assert_eq!(gameplay_json(&gateway)["inventory"], items, "no item is paid");
    send(&mut gateway, &payload(id, "OnNextLevelMsg"), 3_000);
    assert_eq!(profile_xp(&gateway, POSTMAN), xp + u64::from(paid), "paid once");
}

/// L103: Mail Sorting runs on a third Type8 engine for the Postman. Play on the live Sanctuary
/// row (414, game 376) refuses a player without the job unlocked and changes nothing; L936: an
/// unlocked-but-inactive Postman is switched into the job; as the Postman it starts the board
/// 60 m from every node, and a won run pays Postman XP once and no item. The board is the
/// synthetic W171 one with no outputs, the shipped file's payout shape.
#[test]
fn games_window_play_sorts_mail_for_the_postman() {
    const POSTMAN: i32 = mail_sorting::POSTMAN_PROFILE_ID;
    let mail = Game {
        node: "",
        json_key: "mail_sorting",
        profile: POSTMAN,
        goals: 4,
        outputs: OUTPUTS,
        region: None,
    };
    let fixture = Fixture::new(PLOT);
    let mut spec = serde_json::to_value(board(4)).unwrap();
    spec["outputs"] = json!([]);
    let content = json!({
        "schema_version": 1, "game_id": 376, "profile_id": POSTMAN, "name_id": 102030,
        "description_id": 35611, "icon_id": 911, "preload_file": "MailsortingAssets.txt",
        "launch_movie": "game_mailsorting.gfx", "spec": spec,
    });
    let path = fixture.directory.join(mail_sorting::CONTENT_FILE);
    fs::write(path, serde_json::to_vec(&content).unwrap()).unwrap();
    // L936: the switch needs a job it can dress (Chest and Legs). The reference campaign has no
    // Postman starter row, so give it the play package's two pieces (`postman_trainer.rs`).
    let profiles_path = fixture.directory.join("profiles.json");
    let mut profiles = json::read(&profiles_path);
    profiles["starter_items"]["4"] = json!([12_827, 13_197]);
    json::write_pretty(&profiles_path, &profiles);
    let state = |active: bool| {
        let mut state = fixture.state(false, false);
        let profiles = state.profiles.as_mut().unwrap();
        profiles.unlocked.insert(POSTMAN);
        if active {
            profiles.active = POSTMAN;
        }
        state
    };

    let mut none = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    let before = none.snapshot();
    let refused = play(&mut none, 414, 2_000);
    assert!(
        inners(&refused).contains(&join_error(MISSING_JOB_TEXT).as_slice()),
        "B4: the client's own missing-job alert (147/2 116214)"
    );
    assert_eq!(none.snapshot(), before, "a refused Play changes nothing");

    let mut idle = fixture.gateway_at(state(false), &fixture.node, 60.0);
    assert_ne!(idle.gameplay().unwrap().active_profile(), Some(POSTMAN));
    tick(&mut idle, 1_500);
    let switched = play(&mut idle, 414, 2_000);
    assert!(
        !said(&switched, "Switch to the"),
        "{}",
        chat_text(&switched)
    );
    assert_eq!(idle.gameplay().unwrap().active_profile(), Some(POSTMAN));
    assert!(
        inners(&switched)
            .iter()
            .any(|p| p.starts_with(&[38, 0, 21, 0])),
        "the job switch sends an activation"
    );
    assert_eq!(started_game(&switched), (376, 102030, 35611));
    assert!(active(&idle, mail).is_some(), "the mail run is offered");

    let mut gateway = fixture.gateway_at(state(true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let started = play(&mut gateway, 414, 2_000);
    assert_eq!(started_game(&started), (376, 102030, 35611));
    let id = active(&gateway, mail).expect("the mail run is offered") as i32;
    let (xp, items) = (
        profile_xp(&gateway, POSTMAN),
        gameplay_json(&gateway)["inventory"].clone(),
    );
    win(&mut gateway, mail, id, 2_100);
    let paid = mail_sorting::RECONSTRUCTED_POSTMAN_XP_PER_WIN;
    assert_eq!(profile_xp(&gateway, POSTMAN), xp + u64::from(paid));
    assert_eq!(
        gameplay_json(&gateway)["inventory"],
        items,
        "no item is paid"
    );
    send(&mut gateway, &payload(id, "OnNextLevelMsg"), 3_000);
    assert_eq!(
        profile_xp(&gateway, POSTMAN),
        xp + u64::from(paid),
        "paid once"
    );
}

/// L927: with the release `job_xp_curve.json` in the content, a won run pays the job's `activity_xp`
/// (Miner 63, Chef 86); the (R) 25 of the constants is only the fallback the other tests keep.
#[test]
fn a_won_gather_run_pays_the_jobs_activity_xp_from_the_curve_file() {
    for (game, rate) in [(MINING, 63), (HARVEST, 86), (PLOT, 86)] {
        let fixture = Fixture::new(game).with_activity_rates();
        let mut gateway = fixture.ready();
        tick(&mut gateway, 1_500);
        click(&mut gateway, fixture.node.guid, 2_000);
        let id = active(&gateway, game).expect("run offered") as i32;
        win(&mut gateway, game, id, 2_100);
        assert_eq!(xp(&gateway, game), rate, "{}", game.node);
    }
}

/// L104: Play on the live checkers row (514, MiniGameData 21) offers the table; after the
/// movie's handshake a player move is answered by the server's own computer move.
#[test]
fn games_window_play_opens_a_checkers_table_against_the_server() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let offered = play(&mut gateway, 514, 2_000);
    let offer = inners(&offered)
        .into_iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the 39/16 offer")
        .to_vec();
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 21);
    let id = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    send(&mut gateway, &frame(5, id, -1), 2_100);
    for (step, text) in ["OnConnectMsg", "OnStartGameMsg\t0", "OnReadyMsg"]
        .iter()
        .enumerate()
    {
        send(&mut gateway, &payload(id, text), 2_200 + step as u64);
    }
    let moved = send(&mut gateway, &payload(id, "OnMoveMsg\t21\t17"), 2_300);
    assert!(
        said(&moved, "OnCheckersEventMsg\t10\t21\t17"),
        "the player's move"
    );
    assert!(
        said(&moved, "OnCheckersEventMsg\t2\t1"),
        "the computer's turn ended"
    );
}

/// L961: Play on a live Feeling the Beat! row (511, MiniGameData 201) offers that song; the
/// movie launches on 39/5, its data request gets the song's row, and a second Play while it is
/// open is refused by the occupation gate.
#[test]
fn games_window_play_opens_a_feeling_the_beat_song() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let offered = play(&mut gateway, 511, 2_000);
    let offer = inners(&offered)
        .into_iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the 39/16 offer")
        .to_vec();
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 201);
    assert!(
        (offer.windows(25)).any(|w| w == b"MusicConductingAssets.txt"),
        "the movie's preload list, or the start screen stays on Loading..."
    );
    let id = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let again = play(&mut gateway, 512, 2_050);
    // The open song is an occupation, so the shared gate refuses another activity.
    assert_eq!(
        chat_text(&again).trim_end(),
        "Finish what you are doing before starting an activity."
    );
    let started = send(&mut gateway, &frame(5, id, -1), 2_100);
    assert!(said(&started, "game_music.gfx"), "the music movie launches");
    send(&mut gateway, &payload(id, "OnConnectMsg"), 2_200);
    let data = send(
        &mut gateway,
        &payload(id, "FRServer_ReadyForGameData"),
        2_300,
    );
    assert!(said(&data, "OnGameDataMsg	201"), "the song's data");
    assert!(said(&data, "OnPhaseDataMsg	20101^201^"), "its level row");

    // L961: the song's end (37 s after its row) wins once and pays its coins once.
    let before = coins(&gateway);
    send(&mut gateway, &payload(id, "FRServer_ScoreSet	1000"), 20_000);
    let won = send(
        &mut gateway,
        &payload(id, "FRServer_GameWon	1000	0"),
        39_400,
    );
    let result = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .map(<[u8]>::to_vec)
    };
    assert_eq!(result(&won).expect("the result screen")[15], 1, "won");
    let paid = i64::from(osfr_tools::music_game::WIN_COINS);
    assert_eq!(coins(&gateway), before + paid);
    let again = send(
        &mut gateway,
        &payload(id, "FRServer_GameWon	1000	0"),
        39_500,
    );
    assert!(result(&again).is_none(), "the verdict is given once");
    assert_eq!(coins(&gateway), before + paid, "and paid once");
    send(&mut gateway, &payload(id, "FRServer_GameOver"), 39_600);
    send(&mut gateway, &CLOSE, 40_000);
    let next = play(&mut gateway, 568, 41_000);
    let offer = inners(&next)
        .into_iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("a new song after the close");
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 204);
}

/// Play on row 510: the offer names the preload list, the movie launches, the rows and the sword
/// row go out, six rounds win once and pay once, and the job never changes. A later Play reopens.
#[test]
fn games_window_play_opens_hack_n_slash_and_keeps_the_job() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let job = gateway.gameplay().unwrap().active_profile();
    let offer_of = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 16]))
            .map(<[u8]>::to_vec)
    };
    let offered = play(&mut gateway, 510, 2_000);
    let offer = offer_of(&offered).expect("the 39/16 offer");
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 79);
    assert!(
        (offer.windows(22)).any(|w| w == b"SwordTestingAssets.txt"),
        "the movie's preload list, or the start screen stays on Loading..."
    );
    let id = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let started = send(&mut gateway, &frame(5, id, -1), 2_100);
    assert!(said(&started, "game_sword.gfx"), "the sword movie launches");
    send(&mut gateway, &payload(id, "OnConnectMsg"), 2_200);
    let data = send(
        &mut gateway,
        &payload(id, "FRServer_ReadyForGameData"),
        2_300,
    );
    assert!(said(&data, "OnPhaseDataMsg\t7906^79^6^"), "the last round");
    assert!(said(&data, "OnCustomDataMsg\t7901^79^3^"), "the sword row");

    let before = coins(&gateway);
    for round in 1..=6 {
        let total = format!("FRServer_ScoreSet\t{}", round * 3_000);
        send(&mut gateway, &payload(id, &total), 5_000 * round);
        send(&mut gateway, &payload(id, "FRServer_CompleteLevel"), 5_000 * round);
    }
    send(&mut gateway, &payload(id, "FRServer_StopLevel\t6\t1"), 31_000);
    let win = payload(id, "FRServer_GameWon\t18000\t0");
    let won = send(&mut gateway, &win, 31_100);
    let result = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .map(<[u8]>::to_vec)
    };
    assert_eq!(result(&won).expect("the result screen")[15], 1, "won");
    let content = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let paid = osfr_tools::sword_game::SwordContent::load(&content).unwrap().win_coins;
    assert_eq!(coins(&gateway), before + i64::from(paid));
    assert!(result(&send(&mut gateway, &win, 31_200)).is_none(), "given once");
    assert_eq!(coins(&gateway), before + i64::from(paid), "and paid once");
    assert_eq!(gateway.gameplay().unwrap().active_profile(), job);
    send(&mut gateway, &payload(id, "FRServer_GameOver"), 31_300);
    send(&mut gateway, &CLOSE, 32_000);
    let next = play(&mut gateway, 510, 33_000);
    assert!(offer_of(&next).is_some(), "a new game after the close");
}

/// A ship report as the client's frame update sends it (UpdateShipPhysics, 117 bytes): the guid,
/// a flag and a stamp, the identity rotation, the position row, a vec4 and four values.
fn ship_report(guid: u64, at: [f32; 3]) -> Vec<u8> {
    let mut packet = vec![94, 0, 16, 0];
    packet.extend(guid.to_le_bytes());
    packet.extend([0; 9]);
    let rows = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    for value in rows.into_iter().chain([at[0], at[1], at[2], 1.0]) {
        packet.extend(f32::to_le_bytes(value));
    }
    packet.extend([0; 32]);
    packet
}

/// L966: in a Pirate's Plunder match the client's ship reports take a chest within reach and bank
/// it at the player's dock: the chest goes (DestroyProxiedPickup), then the gold shows (UpdatePlayer
/// and RefreshUI).
#[test]
fn a_pirates_ship_takes_a_chest_and_banks_it_at_its_dock() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let go = press_go(&mut gateway, &loaded, 2_150);
    let start: Vec<_> = inners(&go).into_iter().map(<[u8]>::to_vec).collect();
    let ship = (start.iter())
        .find(|p| p.starts_with(&[94, 0, 9, 0]))
        .expect("the ship");
    let guid = u64::from_le_bytes(ship[4..12].try_into().unwrap());
    assert!(
        start.iter().any(|p| p.starts_with(&[94, 0, 28, 0])),
        "the chests"
    );
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir)
        .unwrap()
        .unwrap();
    let arena = &content.arenas[0];
    let chest = arena.pickups[1];
    let gone: &[u8] = &[94, 0, 29, 0, 2, 0, 0, 0];
    let took = |out: &GatewayOutcome| inners(out).iter().any(|p| p.starts_with(gone));
    // The client is ready at once (clients, assets, physics); the countdown then runs. Reports
    // before Playing take nothing, even one with the chest in reach of an earlier one.
    send(&mut gateway, &[94, 0, 3, 0, 0x1a, 0, 0, 0], 2_200);
    let dock = arena.docks[0].bank;
    assert!(!took(&send(&mut gateway, &ship_report(guid, dock), 2_250)));
    assert!(!took(&send(&mut gateway, &ship_report(guid, chest), 6_250)));
    let playing: &[u8] = &[94, 0, 2, 0, 10, 0, 0, 0];
    let mut now = 6_300;
    while !inners(&tick(&mut gateway, now))
        .iter()
        .any(|p| *p == playing)
    {
        now += 500;
        assert!(now < 30_000, "Playing");
    }
    // The client takes keys only from a ship with health and without its held flag: its first
    // report in Playing, and only that one, sets both.
    let first = send(&mut gateway, &ship_report(guid, dock), now + 100);
    let state = (inners(&first).into_iter())
        .find(|p| p.starts_with(&[94, 0, 17, 0]))
        .expect("the ship let go");
    assert_eq!(state[12..16], 0_u32.to_le_bytes(), "the flags");
    assert_eq!(state[16..20], content.ship.health.to_le_bytes(), "the health");
    assert!(inners(&send(&mut gateway, &ship_report(guid, dock), now + 200)).is_empty());
    // The leg from the pad to the chest at the speed bound, with a second to spare.
    let leg = (chest[0] - dock[0]).hypot(chest[2] - dock[2]) / content.treasure.max_speed;
    let leg_ms = (leg.ceil() as u64 + 1) * 1000;
    let other = send(&mut gateway, &ship_report(guid + 1, chest), now + leg_ms);
    assert!(!took(&other), "another ship's report takes nothing");
    let taken = send(&mut gateway, &ship_report(guid, chest), now + leg_ms);
    assert!(took(&taken), "chest 2 taken");
    let banked = send(&mut gateway, &ship_report(guid, dock), now + 2 * leg_ms);
    let gold = (inners(&banked).into_iter())
        .find(|p| p.starts_with(&[94, 0, 8, 0]))
        .expect("the gold");
    assert_eq!(gold[16..20], content.treasure.value.to_le_bytes());
    assert!(
        inners(&banked).iter().any(|p| *p == [94, 0, 4, 0]),
        "RefreshUI"
    );
}

/// A dock book stand lists the match and a choice zones into the arena, as the Games row does.
#[test]
fn a_dock_book_stand_sails_a_pirates_match() {
    const STAND: u64 = 100_000_001_910;
    let fixture = Fixture::new(PLOT);
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir)
        .unwrap()
        .expect("the shipped arenas");
    let stand = content.stands.iter().find(|s| s.npc == STAND).unwrap();
    let mut at = fixture.node.clone();
    at.position[..3].copy_from_slice(&stand.position);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &at, 1.0);
    tick(&mut gateway, 1_500);
    let mut click = vec![26, 0, 8, 0];
    click.extend(STAND.to_le_bytes());
    click.push(0);
    let menu = send(&mut gateway, &click, 2_000);
    assert!(inners(&menu).iter().any(|p| p.starts_with(&[26, 0, 9, 0])), "the stand's menu");
    let mut choose = vec![26, 0, 10, 0];
    choose.extend(STAND.to_le_bytes());
    choose.extend(1_i32.to_le_bytes());
    let entered = send(&mut gateway, &choose, 2_100);
    assert!(said(&entered, "ss_pirate_arena"), "the arena zone");
}

/// A game table lists chess and checkers, and each choice offers that game.
#[test]
fn a_game_table_offers_chess_and_checkers() {
    const TABLE: u64 = 100_000_001_404;
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content/board_tables.json");
    let file: serde_json::Value = serde_json::from_slice(&std::fs::read(data).unwrap()).unwrap();
    let tables = file["tables"].as_array().unwrap();
    assert_eq!(tables.len(), 18);
    let table = tables.iter().find(|table| table["npc"] == TABLE).unwrap();
    let fixture = Fixture::new(PLOT);
    let mut at = fixture.node.clone();
    for (axis, value) in table["position"].as_array().unwrap().iter().enumerate() {
        at.position[axis] = value.as_f64().unwrap() as f32;
    }
    for (choice, game) in [(1_i32, 20_i32), (2, 21)] {
        let mut gateway = fixture.gateway_at(fixture.state(false, true), &at, 1.0);
        tick(&mut gateway, 1_500);
        let mut click = vec![26, 0, 8, 0];
        click.extend(TABLE.to_le_bytes());
        click.push(0);
        let menu = send(&mut gateway, &click, 2_000);
        assert!(inners(&menu).iter().any(|p| p.starts_with(&[26, 0, 9, 0])), "the menu");
        let mut choose = vec![26, 0, 10, 0];
        choose.extend(TABLE.to_le_bytes());
        choose.extend(choice.to_le_bytes());
        let offered = send(&mut gateway, &choose, 2_100);
        let offers: Vec<_> = (inners(&offered).into_iter())
            .filter(|p| p.starts_with(&[39, 0, 16]))
            .map(|p| i32::from_le_bytes(p[11..15].try_into().unwrap()))
            .collect();
        assert_eq!(offers, [game]);
    }
}

/// Play on row 1087: the offer names the preload list, the movie launches, its tier goes out, a
/// finished round wins once and pays once, and the job never changes. Row 1089 runs its own tier.
#[test]
fn games_window_play_opens_spot_the_difference_and_keeps_the_job() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let job = gateway.gameplay().unwrap().active_profile();
    let offer_of = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 16]))
            .map(<[u8]>::to_vec)
    };
    let offered = play(&mut gateway, 1087, 2_000);
    let offer = offer_of(&offered).expect("the 39/16 offer");
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 1040);
    assert!(
        (offer.windows(19)).any(|w| w == b"ForcePerception.txt"),
        "the movie's preload list, or the start screen stays on Loading..."
    );
    let id = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let started = send(&mut gateway, &frame(5, id, -1), 2_100);
    assert!(said(&started, "ForcePerception.gfx"), "the movie launches");
    let stage = send(&mut gateway, &payload(id, "FRServer_RequestStageId"), 2_300);
    assert!(said(&stage, "VOnServerSetStageIdMsg\t1"), "the tier");

    let before = coins(&gateway);
    for _ in 0..5 {
        send(&mut gateway, &payload(id, "FRServer_StatUpdate\t22\t1"), 5_000);
    }
    let win = payload(id, "FRServer_GameWon\t15000");
    let won = send(&mut gateway, &win, 30_000);
    let result = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .map(<[u8]>::to_vec)
    };
    assert_eq!(result(&won).expect("the result screen")[15], 1, "won");
    let paid = coins(&gateway) - before;
    assert!(paid > 0, "a win pays");
    assert!(result(&send(&mut gateway, &win, 30_100)).is_none(), "given once");
    assert_eq!(coins(&gateway), before + paid, "and paid once");
    assert_eq!(gateway.gameplay().unwrap().active_profile(), job);
    send(&mut gateway, &payload(id, "FRServer_GameClose"), 30_200);
    send(&mut gateway, &CLOSE, 31_000);
    let next = play(&mut gateway, 1089, 33_000);
    let offer = offer_of(&next).expect("a new game after the close");
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 1044);
}

/// A telescope lists all three games, and a choice offers that game.
#[test]
fn a_telescope_offers_spot_the_difference() {
    const TELESCOPE: u64 = 100_000_033_332;
    let fixture = Fixture::new(PLOT);
    let mut at = fixture.node.clone();
    at.position = [-1313.8291, -15.344327, 291.77377];
    for (choice, game) in [(1_i32, 1040_i32), (3, 1044)] {
        let mut gateway = fixture.gateway_at(fixture.state(false, true), &at, 1.0);
        tick(&mut gateway, 1_500);
        let mut click = vec![26, 0, 8, 0];
        click.extend(TELESCOPE.to_le_bytes());
        click.push(0);
        let menu = send(&mut gateway, &click, 2_000);
        assert!(inners(&menu).iter().any(|p| p.starts_with(&[26, 0, 9, 0])), "the menu");
        let mut choose = vec![26, 0, 10, 0];
        choose.extend(TELESCOPE.to_le_bytes());
        choose.extend(choice.to_le_bytes());
        let offered = send(&mut gateway, &choose, 2_100);
        let offers: Vec<_> = (inners(&offered).into_iter())
            .filter(|p| p.starts_with(&[39, 0, 16]))
            .map(|p| i32::from_le_bytes(p[11..15].try_into().unwrap()))
            .collect();
        assert_eq!(offers, [game]);
    }
}

/// Two levels: clearing the first offers the second with no win; clearing the last wins once. A
/// run that clears level 1 and then runs out of time, or leaves, has won too (R).
#[test]
fn a_two_level_hunt_wins_after_its_last_level_or_on_time_after_the_first() {
    let hunt = Game {
        node: "",
        json_key: "artifact_hunt",
        profile: artifact_hunt::ADVENTURER_PROFILE_ID,
        goals: 4,
        outputs: OUTPUTS,
        region: None,
    };
    let fixture = Fixture::new(PLOT).with_activity_rates();
    let mut spec = serde_json::to_value(board(4)).unwrap();
    spec["outputs"] = json!([]);
    let level = json!({
        "selectable": spec["selectable"], "tiles": spec["tiles"], "goals": spec["goals"],
        "duration_seconds": 30,
    });
    let content = json!({
        "schema_version": 1, "game_id": 57, "profile_id": artifact_hunt::ADVENTURER_PROFILE_ID,
        "name_id": 35604, "description_id": 35605, "icon_id": 912,
        "preload_file": "ArtifactAssets.txt", "launch_movie": "game_artifact.gfx",
        "levels": [level], "spec": spec,
    });
    let path = fixture.directory.join(artifact_hunt::CONTENT_FILE);
    fs::write(path, serde_json::to_vec(&content).unwrap()).unwrap();
    let completed = |gateway: &Gateway| gameplay_json(gateway)["artifact_hunt"]["completed"].clone();

    // The last level wins.
    let mut gateway = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 413, 2_000);
    let id = active(&gateway, hunt).expect("the hunt is offered") as i32;
    win(&mut gateway, hunt, id, 2_100);
    assert_eq!(completed(&gateway), 0, "level 1 is not the end");
    let done = win_next_level(&mut gateway, hunt, id, 4_000);
    assert!(said(&done, "OnGameWonMsg"), "the last level wins");
    assert_eq!(completed(&gateway), 1);

    // Time runs out on level 2: still a win.
    let mut gateway = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 413, 2_000);
    let id = active(&gateway, hunt).expect("the hunt is offered") as i32;
    win(&mut gateway, hunt, id, 2_100);
    send(&mut gateway, &payload(id, "OnStartLevelMsg"), 4_000);
    tick(&mut gateway, 40_000);
    tick(&mut gateway, 41_000);
    assert_eq!(completed(&gateway), 0, "the client ends the level first");
    let ended = send(&mut gateway, &payload(id, "OnNextLevelMsg"), 41_100);
    assert!(said(&ended, "OnGameWonMsg"), "level 1 was cleared");
    assert_eq!(completed(&gateway), 1);

    // Leaving on level 2: a win too, and the run is gone.
    let mut gateway = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 413, 2_000);
    let id = active(&gateway, hunt).expect("the hunt is offered") as i32;
    win(&mut gateway, hunt, id, 2_100);
    send(&mut gateway, &payload(id, "OnStartLevelMsg"), 4_000);
    send(&mut gateway, &frame(6, id, -1), 4_100);
    assert_eq!(completed(&gateway), 1, "level 1 was cleared");
    assert!(active(&gateway, hunt).is_none(), "and the player is free");

    // Leaving in the pause after level 1, before the next level is asked for: a win too.
    let mut gateway = fixture.gateway_at(fixture.state(false, false), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 413, 2_000);
    let id = active(&gateway, hunt).expect("the hunt is offered") as i32;
    for (step, text) in ["OnConnectMsg", "OnStartGameMsg", "OnStartLevelMsg"].iter().enumerate() {
        if step == 0 {
            send(&mut gateway, &frame(5, id, -1), 2_100);
        }
        send(&mut gateway, &payload(id, text), 2_200 + step as u64);
    }
    for slot in 0..hunt.goals {
        let text = format!("OnLevelEventMsg\t50\t{}\t{}", LEFT + slot, BOTTOM);
        send(&mut gateway, &payload(id, &text), 2_700 + slot as u64);
    }
    send(&mut gateway, &frame(6, id, -1), 2_800);
    assert_eq!(completed(&gateway), 1, "level 1 was cleared");
}

/// A Practice Dummy has one menu row, and it offers Hack 'n' Slash.
#[test]
fn a_practice_dummy_offers_hack_n_slash() {
    const DUMMY: u64 = 100_000_003_571;
    let fixture = Fixture::new(PLOT);
    let mut at = fixture.node.clone();
    at.position = [342.4284, 26.54423, 433.1978];
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &at, 1.0);
    tick(&mut gateway, 1_500);
    let mut click = vec![26, 0, 8, 0];
    click.extend(DUMMY.to_le_bytes());
    click.push(0);
    let menu = send(&mut gateway, &click, 2_000);
    assert!(inners(&menu).iter().any(|p| p.starts_with(&[26, 0, 9, 0])), "the menu");
    let mut choose = vec![26, 0, 10, 0];
    choose.extend(DUMMY.to_le_bytes());
    choose.extend(1_i32.to_le_bytes());
    let offered = send(&mut gateway, &choose, 2_100);
    let offers: Vec<_> = (inners(&offered).into_iter())
        .filter(|p| p.starts_with(&[39, 0, 16]))
        .map(|p| i32::from_le_bytes(p[11..15].try_into().unwrap()))
        .collect();
    assert_eq!(offers, [79]);
}

/// The offer in `loaded` (39/16) answered with Go (39/5).
fn press_go(gateway: &mut Gateway, loaded: &[GatewayOutcome], now_ms: u64) -> GatewayOutcome {
    let offer = (loaded.iter().flat_map(inners))
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the offer");
    let mut go = vec![39, 0, 5];
    go.extend(&offer[3..7]);
    go.extend([255; 8]);
    send(gateway, &go, now_ms)
}

/// Steering into the base's cave opens Ship Selection once a visit; a choice swaps the ship on the
/// pad, and a SelectShip with no window open changes nothing.
#[test]
fn the_caves_window_swaps_the_ship() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let go = press_go(&mut gateway, &loaded, 2_150);
    let ship = (inners(&go).into_iter())
        .find(|p| p.starts_with(&[94, 0, 9, 0]))
        .expect("the ship");
    let guid = u64::from_le_bytes(ship[4..12].try_into().unwrap());
    send(&mut gateway, &[94, 0, 3, 0, 0x1a, 0, 0, 0], 2_200);
    let playing: &[u8] = &[94, 0, 2, 0, 10, 0, 0, 0];
    let mut now = 6_300;
    while !inners(&tick(&mut gateway, now)).iter().any(|p| *p == playing) {
        now += 500;
        assert!(now < 30_000, "Playing");
    }
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir).unwrap().unwrap();
    let dock = &content.arenas[0].docks[0];
    send(&mut gateway, &ship_report(guid, dock.bank), now + 100);
    let mut select = vec![94, 0, 21, 0];
    select.extend(3_u32.to_le_bytes());
    select.extend(b"Eel");
    select.extend(0_u32.to_le_bytes());
    let swapped = |out: &GatewayOutcome| inners(out).iter().any(|p| p.starts_with(&[94, 0, 10, 0]));
    assert!(!swapped(&send(&mut gateway, &select, now + 200)), "no window, no swap");
    let leg = (dock.enter[0] - dock.bank[0]).hypot(dock.enter[2] - dock.bank[2]);
    let later = now + 200 + ((leg / content.treasure.max_speed).ceil() as u64 + 1) * 1000;
    let window: &[u8] = &[94, 0, 34, 0, 5, 0, 0, 0];
    let cave = send(&mut gateway, &ship_report(guid, dock.enter), later);
    assert!(inners(&cave).iter().any(|p| p.starts_with(window)), "the window opens");
    let again = send(&mut gateway, &ship_report(guid, dock.enter), later + 100);
    assert!(!inners(&again).iter().any(|p| p.starts_with(window)), "once a visit");
    let swap = send(&mut gateway, &select, later + 200);
    assert!(swapped(&swap), "the swap");
    assert!(!swapped(&send(&mut gateway, &select, later + 300)), "the window closed");
    // The new ship is held until the client has built it, then let go.
    let state: &[u8] = &[94, 0, 17, 0];
    let held = tick(&mut gateway, later + 1_000);
    assert!(!inners(&held).iter().any(|p| p.starts_with(state)), "still held");
    let free = tick(&mut gateway, later + 2_000);
    assert!(inners(&free).iter().any(|p| p.starts_with(state)), "let go");
}

/// The fire button in play: the loaded guns fire, the same frame's repeat does not, the balls come
/// down and the bars hear the guns come back.
#[test]
fn a_broadside_flies_and_splashes() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let go = press_go(&mut gateway, &loaded, 2_150);
    let ship = (inners(&go).into_iter())
        .find(|p| p.starts_with(&[94, 0, 9, 0]))
        .expect("the ship");
    let guid = u64::from_le_bytes(ship[4..12].try_into().unwrap());
    let mut fire = vec![94, 0, 20, 0];
    fire.extend([0; 16]);
    fire.extend(5_u32.to_le_bytes());
    let count = |out: &GatewayOutcome, sub: u8| {
        inners(out).iter().filter(|p| p.starts_with(&[94, 0, sub, 0])).count()
    };
    assert_eq!(count(&send(&mut gateway, &fire, 2_200), 26), 0, "not in play yet");
    send(&mut gateway, &[94, 0, 3, 0, 0x1a, 0, 0, 0], 2_200);
    let playing: &[u8] = &[94, 0, 2, 0, 10, 0, 0, 0];
    let mut now = 6_300;
    let mut bars = 0;
    loop {
        let out = tick(&mut gateway, now);
        bars += count(&out, 19);
        if inners(&out).iter().any(|p| *p == playing) {
            break;
        }
        now += 500;
        assert!(now < 30_000, "Playing");
    }
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir).unwrap().unwrap();
    let dock = content.arenas[0].docks[0].bank;
    send(&mut gateway, &ship_report(guid, dock), now + 100);
    assert_eq!(bars + count(&tick(&mut gateway, now + 150), 19), 1, "the bars start full");
    let cannons = content.opening_ship().cannons.len();
    assert_eq!(count(&send(&mut gateway, &fire, now + 200), 26), cannons, "both broadsides");
    assert_eq!(count(&send(&mut gateway, &fire, now + 216), 26), 0, "held down: still loading");
    let flight = (content.gunnery.flight_seconds * 1000.0) as u64;
    assert_eq!(count(&tick(&mut gateway, now + 200 + flight - 1), 27), 0, "in the air");
    let down = tick(&mut gateway, now + 200 + flight);
    assert_eq!(count(&down, 27), cannons, "in the water");
}

/// The captains come with the match: a ship and a row each, and they sail once Playing has begun.
#[test]
fn the_captains_sail_beside_the_player() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let go = press_go(&mut gateway, &loaded, 2_150);
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir).unwrap().unwrap();
    let seats = content.crew(&content.arenas[0]).len();
    assert!(seats > 0);
    let ships: Vec<_> = (inners(&go).into_iter())
        .filter(|p| p.starts_with(&[94, 0, 9, 0]))
        .collect();
    assert_eq!(ships.len(), 1 + seats, "the player's first");
    let guid = u64::from_le_bytes(ships[0][4..12].try_into().unwrap());
    let count: &[u8] = &[94, 0, 7, 0, 1, 0, 0, 0];
    let counted = inners(&go).into_iter().find(|p| p.starts_with(count)).expect("the count");
    assert_eq!(counted[8..12], (seats as u32).to_le_bytes());
    send(&mut gateway, &[94, 0, 3, 0, 0x1a, 0, 0, 0], 2_200);
    let playing: &[u8] = &[94, 0, 2, 0, 10, 0, 0, 0];
    let mut now = 6_300;
    while !inners(&tick(&mut gateway, now)).iter().any(|p| *p == playing) {
        now += 500;
        assert!(now < 30_000, "Playing");
    }
    let sailing = |out: &GatewayOutcome| {
        inners(out).iter().filter(|p| p.starts_with(&[94, 0, 16, 0])).count()
    };
    assert_eq!(sailing(&tick(&mut gateway, now + 50)), 0, "held until the round begins");
    let dock = content.arenas[0].docks[0].bank;
    let begun = send(&mut gateway, &ship_report(guid, dock), now + 100);
    let states = inners(&begun).iter().filter(|p| p.starts_with(&[94, 0, 17, 0])).count();
    assert_eq!(states, 1 + seats, "every ship is let go");
    tick(&mut gateway, now + 200);
    assert_eq!(sailing(&tick(&mut gateway, now + 500)), seats, "they report");
}

/// A forged position past the speed bound opens no window, so it cannot call the ship home.
#[test]
fn a_forged_report_opens_no_ship_selection() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let go = press_go(&mut gateway, &loaded, 2_150);
    let ship = (inners(&go).into_iter())
        .find(|p| p.starts_with(&[94, 0, 9, 0]))
        .expect("the ship");
    let guid = u64::from_le_bytes(ship[4..12].try_into().unwrap());
    send(&mut gateway, &[94, 0, 3, 0, 0x1a, 0, 0, 0], 2_200);
    let playing: &[u8] = &[94, 0, 2, 0, 10, 0, 0, 0];
    let mut now = 6_300;
    while !inners(&tick(&mut gateway, now)).iter().any(|p| *p == playing) {
        now += 500;
        assert!(now < 30_000, "Playing");
    }
    let content_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content");
    let content = osfr_tools::ship_combat::Content::load(&content_dir).unwrap().unwrap();
    let dock = &content.arenas[0].docks[0];
    send(&mut gateway, &ship_report(guid, dock.bank), now + 100);
    // Sail far away over time, then claim the cave in one hop.
    let far = [dock.bank[0] + 2_000.0, dock.bank[1], dock.bank[2]];
    send(&mut gateway, &ship_report(guid, far), now + 60_000);
    let forged = send(&mut gateway, &ship_report(guid, dock.enter), now + 60_100);
    let window: &[u8] = &[94, 0, 34, 0, 5, 0, 0, 0];
    assert!(!inners(&forged).iter().any(|p| p.starts_with(window)));
}

/// Closing the offer leaves the arena before anything started.
#[test]
fn closing_a_pirates_offer_zones_back() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    for op in [13_u16, 10] {
        send(&mut gateway, &op.to_le_bytes(), 2_100);
    }
    let closed = send(&mut gateway, &[26, 0, 42, 0], 3_000);
    assert!(inners(&closed).iter().any(|p| p.starts_with(&[39, 0, 19])), "the offer closed");
    assert!(tick(&mut gateway, 5_500).began_zoning, "the zone back");
}

/// The X in play ends the match at once: the run closes as a loss, Exiting goes out and the player
/// zones back, unpaid.
#[test]
fn quitting_a_pirates_match_zones_back_unpaid() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    play(&mut gateway, 1059, 2_000);
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let offer: Vec<u8> = (loaded.iter().flat_map(inners))
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the offer")
        .to_vec();
    press_go(&mut gateway, &loaded, 2_150);
    let before = coins(&gateway);
    let mut quit = vec![39, 0, 6];
    quit.extend(&offer[3..7]);
    quit.extend([255; 8]);
    let left = send(&mut gateway, &quit, 3_000);
    assert!(inners(&left).iter().any(|p| p.starts_with(&[39, 0, 19])), "the run closed");
    assert!(inners(&left).iter().any(|p| *p == [94, 0, 2, 0, 13, 0, 0, 0]), "Exiting");
    assert!(tick(&mut gateway, 5_500).began_zoning, "the zone back");
    assert_eq!(coins(&gateway), before, "a quit pays nothing");
}

/// L963: Play on the Pirate's Plunder row (1059) zones into the first arena; once it loads the start
/// packets go out (the player's own ship among them) and Play again is the busy refusal. The match
/// then runs on its timeouts and clock alone (a silent client) to Exiting and zones back.
#[test]
fn games_window_play_zones_into_a_pirates_arena() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let entered = play(&mut gateway, 1059, 2_000);
    assert!(said(&entered, "ss_pirate_arena"), "the arena zone");
    let loaded = [13_u16, 10].map(|op| send(&mut gateway, &op.to_le_bytes(), 2_100));
    let is_ship = |p: &[u8]| p.starts_with(&[94, 0, 9, 0]);
    assert!(!loaded.iter().flat_map(inners).any(is_ship), "no ship before Go");
    let go = press_go(&mut gateway, &loaded, 2_150);
    assert!(inners(&go).into_iter().any(is_ship), "CreateProxiedShip on Go");
    assert!(inners(&go).iter().any(|p| *p == [94, 0, 4, 0]), "the standings show at once");
    let again = play(&mut gateway, 1059, 2_200);
    assert_eq!(
        chat_text(&again).trim_end(),
        "Finish what you are doing before starting an activity."
    );
    let exiting: &[u8] = &[94, 0, 2, 0, 13, 0, 0, 0];
    let (mut exited, mut back, mut board) = (false, false, 0);
    let before = coins(&gateway);
    // Three report timeouts, the countdown, PlayTime (540 s), Game Over's timeout and the exit beat.
    for second in 3..=900 {
        let out = tick(&mut gateway, second * 1_000);
        exited |= inners(&out).iter().any(|p| *p == exiting);
        board += inners(&out).iter().filter(|p| p.starts_with(&[94, 0, 24, 0])).count();
        if out.began_zoning {
            back = true;
            break;
        }
    }
    assert!(exited && back, "Exiting, then the zone back");
    assert_eq!(board, 1, "PlayerAwards opens the results board, once");
    assert_eq!(coins(&gateway), before + 10, "the content's win_coins, once");
}

/// L962: Play on the Easy "Penguin Produce" row (532) offers Tower Defense game 350 with the movie's
/// preload list; its data goes out once, a cleared map (stages 0-10 reported) wins once and pays once,
/// and after the close Play on a Hard row (195) offers game 150.
#[test]
fn games_window_play_opens_a_tower_defense_map() {
    let fixture = Fixture::new(PLOT);
    let mut gateway = fixture.gateway_at(fixture.state(false, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let offered = play(&mut gateway, 532, 2_000);
    let offer = inners(&offered)
        .into_iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("the 39/16 offer")
        .to_vec();
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 350);
    assert!(
        (offer.windows(22)).any(|w| w == b"TowerDefenseAssets.txt"),
        "the movie's preload list, or the start screen stays on Loading..."
    );
    let id = i32::from_le_bytes(offer[3..7].try_into().unwrap());
    let again = play(&mut gateway, 195, 2_050);
    assert_eq!(
        chat_text(&again).trim_end(),
        "Finish what you are doing before starting an activity."
    );
    let started = send(&mut gateway, &frame(5, id, -1), 2_100);
    assert!(said(&started, "towerdefense.gfx"), "the movie launches");
    let data = send(
        &mut gateway,
        &payload(id, "FRServer_ReadyForGameData"),
        2_300,
    );
    assert!(said(&data, "OnGameDataMsg	350"), "the game");
    assert!(said(&data, "OnMapDataMsg	350^350^"), "its map row");
    assert!(said(&data, "OnGameDataCompleteMsg"), "and the start");
    let twice = send(
        &mut gateway,
        &payload(id, "FRServer_ReadyForGameData"),
        2_400,
    );
    assert!(!said(&twice, "OnMapDataMsg"), "the data goes out once");

    let before = coins(&gateway);
    for stage in 0..=10 {
        let text = format!("FRServer_CompletedStage	{stage}");
        send(&mut gateway, &payload(id, &text), 3_000 + stage);
    }
    let won = send(&mut gateway, &payload(id, "FRServer_GameWon	4321"), 90_000);
    let result = |out: &GatewayOutcome| {
        (inners(out).into_iter())
            .find(|p| p.starts_with(&[39, 0, 18]))
            .map(<[u8]>::to_vec)
    };
    assert_eq!(result(&won).expect("the result screen")[15], 1, "won");
    let paid = before + 10;
    assert_eq!(coins(&gateway), paid, "the content's win_coins");
    let again = send(&mut gateway, &payload(id, "FRServer_GameWon	4321"), 90_100);
    assert!(result(&again).is_none(), "the verdict is given once");
    assert_eq!(coins(&gateway), paid, "and paid once");
    send(&mut gateway, &payload(id, "FRServer_GameOver"), 90_200);
    send(&mut gateway, &CLOSE, 91_000);
    let next = play(&mut gateway, 195, 92_000);
    let offer = inners(&next)
        .into_iter()
        .find(|p| p.starts_with(&[39, 0, 16]))
        .expect("a new game after the close");
    assert_eq!(i32::from_le_bytes(offer[11..15].try_into().unwrap()), 150);
}

#[test]
fn each_farm_plot_starts_its_own_region_game_and_pays_its_own_produce() {
    for (region, outputs) in REGION_OUTPUTS {
        let game = Game {
            outputs,
            region: Some(region),
            ..PLOT
        };
        let fixture = Fixture::new(game);
        let mut gateway = fixture.ready();
        tick(&mut gateway, 1_500);
        let started = click(&mut gateway, fixture.node.guid, 2_000);
        let named = harvesting::region(region).unwrap();
        assert_eq!(
            started_game(&started),
            (region, named.name_id, named.description_id),
            "plot of game {region}"
        );
        let id = active(&gateway, game).expect("the plot's run is offered") as i32;
        win(&mut gateway, game, id, 2_100);
        assert_eq!(ore(&gateway, game), vec![1; 4], "game {region} produce");
        assert_eq!(ore(&gateway, HARVEST), vec![0; 4], "game 111's produce");
        assert_eq!(
            xp(&gateway, game),
            u64::from(RECONSTRUCTED_CHEF_XP_PER_WIN),
            "game {region} (R) XP"
        );
        send(&mut gateway, &CLOSE, 3_100);

        // The same Chef then clicks a mushroom: the engine is back on game 111, whose produce is
        // paid, and the run ids go on counting from the plot's.
        let saved = gateway.snapshot().gameplay.unwrap();
        let mut again = fixture.gateway_at(saved, &mushroom(), 2.0);
        let started = click(&mut again, mushroom().guid, 4_000);
        assert_eq!(started_game(&started), (111, 100834, 94150));
        assert_eq!(active(&again, HARVEST), Some(i64::from(id) + 1));
        win(&mut again, HARVEST, id + 1, 4_100);
        assert_eq!(ore(&again, HARVEST), vec![1; 4], "game 111 produce");
        assert_eq!(ore(&again, game), vec![1; 4], "the region's produce stays");
    }
}

#[test]
fn a_campaign_without_plot_games_refuses_the_plot_and_keeps_the_mushrooms() {
    let fixture = Fixture::new(PLOT).without_plot_games();
    let mut gateway = fixture.ready();
    let before = gateway.snapshot();
    let before_bytes = serde_json::to_vec(&before).unwrap();
    let refused = click(&mut gateway, fixture.node.guid, 2_000);
    assert_eq!(
        chat_text(&refused).trim_end(),
        "Gathering is not available in this campaign."
    );
    assert!(!refused.gameplay_changed, "the refusal saved");
    assert_eq!(gateway.snapshot(), before);
    assert_eq!(
        serde_json::to_vec(&gateway.snapshot()).unwrap(),
        before_bytes
    );
    assert!(active(&gateway, PLOT).is_none());

    let mut again = fixture.gateway_at(fixture.state(true, true), &mushroom(), 2.0);
    let started = click(&mut again, mushroom().guid, 2_000);
    assert_eq!(started_game(&started).0, 111);
}

/// L934: the Games window ore rows (row, MiniGameData game, the ore it pays, the Miner rank it
/// asks), and every ore the Miner's bars smelt.
/// Row, game, ore, and the ore its first goal asks (the client's "Collect N ... ore!").
const ORE_ROWS: [(u32, i32, i32, u32); 4] = [
    (401, 93, 2740, 10),
    (402, 95, 2741, 10),
    (577, 97, 2742, 8),
    (578, 99, 2743, 6),
];
const ORES: [i32; 5] = [2739, 2740, 2741, 2742, 2743];

/// The handshake a client sends after the 39/16 offer, up to the first level.
fn begin_level(gateway: &mut Gateway, id: i32, base_ms: u64) {
    for (step, packet) in [
        frame(5, id, -1),
        payload(id, "OnConnectMsg"),
        payload(id, "OnStartGameMsg"),
        payload(id, "OnStartLevelMsg"),
    ]
    .iter()
    .enumerate()
    {
        send(gateway, packet, base_ms + 20 * step as u64);
    }
}

/// L934: Tin, Iron, Silver and Gold Mining start from their Games window rows at their Miner
/// rank (a lower Miner is refused and nothing changes), play the shipped board and pay their own
/// ore, once, and no other ore.
#[test]
fn games_window_ore_rows_pay_their_own_ore_at_their_miner_rank() {
    let fixture = Fixture::new(MINING)
        .with_shipped_mining()
        .with_release_items();
    let miner = |xp: u32| {
        let mut state = fixture.state(true, true);
        let profiles = state.profiles.as_mut().unwrap();
        profiles.experience.insert(MINER_PROFILE_ID, xp);
        state
    };
    for (row, game, paid, asked) in ORE_ROWS {
        let mut novice = fixture.gateway_at(miner(0), &fixture.node, 60.0);
        let before = novice.snapshot();
        let refused = play(&mut novice, row, 2_000);
        assert!(
            said(&refused, "Miner rank is too low"),
            "row {row}: {}",
            chat_text(&refused)
        );
        assert_eq!(
            novice.snapshot(),
            before,
            "row {row}: a refused Play changed state"
        );

        let mut gateway = fixture.gateway_at(miner(1_000_000), &fixture.node, 60.0);
        tick(&mut gateway, 1_500);
        let started = play(&mut gateway, row, 2_000);
        assert_eq!(started_game(&started).0, game, "row {row}");
        let id = active(&gateway, MINING).expect("the ore run is offered") as i32;
        begin_level(&mut gateway, id, 2_100);
        finish_shipped_mining(&mut gateway, id, 2_200);
        send(&mut gateway, &payload(id, "OnNextLevelMsg"), 5_000);
        assert_eq!(mining_run(&gateway).phase, Type8Phase::Won, "row {row}");
        for ore in ORES {
            // A tile that falls past a met quota still pays.
            let held = count(&gateway, ore);
            let right = if ore == paid { held >= asked } else { held == 0 };
            assert!(right, "row {row}: ore {ore} holds {held}");
        }
        let paid_state = gateway.snapshot();
        send(&mut gateway, &payload(id, "OnNextLevelMsg"), 5_100);
        assert_eq!(
            gateway.snapshot(),
            paid_state,
            "row {row}: a replayed win paid again"
        );
    }
}

/// L934: Mining Practice (row 1113, game 375) plays to a win and pays nothing: no ore, no job XP.
#[test]
fn mining_practice_finishes_and_pays_nothing() {
    let fixture = Fixture::new(MINING)
        .with_shipped_mining()
        .with_release_items();
    let mut gateway = fixture.gateway_at(fixture.state(true, true), &fixture.node, 60.0);
    tick(&mut gateway, 1_500);
    let started = play(&mut gateway, 1113, 2_000);
    assert_eq!(started_game(&started), (375, 92481, 92528));
    let id = active(&gateway, MINING).expect("the practice run is offered") as i32;
    let before = (
        gameplay_json(&gateway)["inventory"].clone(),
        xp(&gateway, MINING),
    );
    begin_level(&mut gateway, id, 2_100);
    finish_shipped_mining(&mut gateway, id, 2_200);
    send(&mut gateway, &payload(id, "OnNextLevelMsg"), 5_000);
    assert_eq!(mining_run(&gateway).phase, Type8Phase::Won);
    assert_eq!(gameplay_json(&gateway)["mining"]["completed"], json!(1));
    assert_eq!(
        (
            gameplay_json(&gateway)["inventory"].clone(),
            xp(&gateway, MINING)
        ),
        before,
        "practice paid something"
    );
}

/// L934: Chugaroot Farm (row 412, game 113) and Thorny Tillage (row 1036, game 119) start from
/// the Games window with their recovered names; the produce they pay is pinned against the
/// recovered region table in `farm_plots.rs`.
#[test]
fn games_window_starts_chugaroot_and_thorny_harvesting() {
    let fixture = Fixture::new(HARVEST)
        .with_shipped_harvesting()
        .with_release_items();
    for (row, game) in [(412, 113), (1036, 119)] {
        let mut gateway = fixture.gateway_at(fixture.state(true, true), &fixture.node, 60.0);
        tick(&mut gateway, 1_500);
        let started = play(&mut gateway, row, 2_000);
        let region = harvesting::region(game).unwrap();
        assert_eq!(
            started_game(&started),
            (game, region.name_id, region.description_id),
            "row {row}"
        );
        assert!(
            active(&gateway, HARVEST).is_some(),
            "row {row}: no run offered"
        );
    }
}
