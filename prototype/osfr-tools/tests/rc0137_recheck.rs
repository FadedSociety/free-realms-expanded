//! The owner's native recheck of build 3a3280a3 (rc-0137), through the server's own path: the
//! release content (`data/content` with `data/release`), a live gateway load, the shared field
//! handed over at load as the gateway binary does, login, the client's 13 and 10, then the
//! packets the client sends. Each test reads what the client would be sent.
//!
//! - `realm goto list kaboomer` named nothing: every session hands its field enemies to the
//!   shared field at load, so the session's goto list had none of them;
//! - the Kaboomers drew the press-X prompt: an engine enemy's AddNpc carried a 3 m reach;
//! - `realm goto Bixie Hive` put the player inside the hive: an entrance had no heading, so the
//!   arrival followed the player's facing;
//! - the Frostfang Snarler den (and the spawner huts) walked: their starters roamed 5 m;
//! - the Troll Summoner and Petal stood twice;
//! - the Troll Summoner's boss bar ticked sporadically: every hit sent a 35/35 as well as its
//!   32/7, and the client applies the 35/35 on the enemy's own animation clock.
#[path = "common/content_copy.rs"]
#[allow(dead_code)]
mod content_copy;
#[path = "common/field_battle_fixture.rs"]
mod field_battle_fixture;
#[path = "common/gateway_packets.rs"]
#[allow(dead_code)]
mod gateway_packets;

use gateway_packets::{body, interact, login_with_version, melee, move_to, wrap};
use osfr_tools::{
    account_store::AccountRole,
    chat,
    encounter::SharedFieldState,
    gateway::Gateway,
    startup_world::{NpcSpawn, parse_add_npc},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

const GUID: u64 = 0x0137_0000_0001;
const BRAWLER: i32 = 43;
/// `sg_bixie_hive_nodoor_01.adr`, the hive body, 3.5 m behind its door
/// (`lab/references/fr-spatial-20261029/prop_placements_2014.tsv`, chunk FabledRealms_0_-24).
const HIVE_BODY: [f32; 2] = [-1445.10, 45.30];

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// The release content, built once per test binary (the gateway only reads it). One folder,
/// copied again by each run, so runs leave no 75 MB copy behind each.
fn release() -> &'static Path {
    static CONTENT: OnceLock<PathBuf> = OnceLock::new();
    CONTENT.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("rc0137-release");
        let _ = fs::remove_dir_all(&root);
        content_copy::copy_tree(&manifest().join("data/content"), &root);
        content_copy::copy_tree(&manifest().join("data/release"), &root);
        root
    })
}

/// A player in the world at `at`, facing `facing`: the gateway binary's load (`load_live`,
/// then the shared field taken), login, then the client's 13 and 10.
struct Player {
    gateway: Gateway,
    field: SharedFieldState,
    now: u64,
    world: Vec<Vec<u8>>,
}

impl Player {
    fn enter(at: [f32; 3], facing: [f32; 3]) -> Self {
        let content = release();
        let mut state = Gateway::content_state(content).unwrap();
        state.guid = GUID;
        state.zone_id = 0;
        state.position = at;
        state.rotation = facing;
        let mut gateway = Gateway::load_live(None, state, content, None).unwrap();
        let field = gateway.take_shared_fields().expect("the field enemies load");
        gateway.set_dev_tools(true);
        gateway.set_role(AccountRole::Admin);
        let version = read(&content.join("startup/manifest.json"))["client_version"]
            .as_str()
            .unwrap()
            .to_owned();
        gateway
            .handle_at(&login_with_version(GUID, &version), 0)
            .unwrap();
        let mut player = Self {
            gateway,
            field,
            now: 100,
            world: Vec::new(),
        };
        player.send(&[13, 0]);
        player.world = player.send(&[10, 0]);
        player
    }

    /// One client message, and a shared-field action resolved against the field as the server
    /// does. Returns the application bodies the client is sent.
    fn send(&mut self, inner: &[u8]) -> Vec<Vec<u8>> {
        self.now += 100;
        let mut outcome = self.gateway.handle_at(&wrap(inner), self.now).unwrap();
        let mut replies: Vec<Vec<u8>> =
            (outcome.replies.iter()).map(|r| body(r).to_vec()).collect();
        if let Some(action) = outcome.shared_field.take() {
            let known: BTreeSet<u64> = (self.field.interest_candidates(false).iter())
                .map(|n| n.guid)
                .collect();
            let shared = (self.gateway)
                .handle_shared_field(&action, &mut self.field, &known, self.now)
                .unwrap();
            replies.extend(shared.replies);
        }
        replies
    }

    fn chat(&mut self, command: &str) -> (String, Vec<Vec<u8>>) {
        let replies = self.send(&chat::local_request(command));
        (system_text(&replies), replies)
    }

    /// Walk to `to` in steps the movement check accepts, as `field_battle_fixture::walk` does.
    fn walk(&mut self, to: [f32; 3]) {
        let from = self.gateway.state().position;
        let far = ((to[0] - from[0]).powi(2) + (to[2] - from[2]).powi(2)).sqrt();
        let steps = (far / 8.0).ceil().max(1.0) as usize;
        for step in 1..=steps {
            let k = step as f32 / steps as f32;
            self.now += 300;
            self.send(&move_to(GUID, [0, 1, 2].map(|i| from[i] + (to[i] - from[i]) * k)));
        }
    }

    /// Press X on a battle starter as a level-12 Brawler, then Go, and load the arena: the
    /// client's 26/13 (a fresh character holds no combat job, and only an entrance switches
    /// one in), 26/8, its 41/108 from the start card (writer `0x008b6e70`), then 13 and 10.
    fn battle(&mut self, starter: u64) -> Vec<Vec<u8>> {
        self.send(&field_battle_fixture::switch_job(BRAWLER));
        self.chat("realm level 12");
        let card = self.send(&interact(starter));
        let details = (card.iter())
            .find(|p| p.starts_with(&[41, 0, 114, 0]))
            .expect("the start card")
            .clone();
        let mut go = vec![41, 0, 108, 0];
        go.extend(&details[4..12]);
        go.extend(0_u64.to_le_bytes());
        self.send(&go);
        self.send(&[13, 0]);
        self.send(&[10, 0])
    }

    /// The AddNpc the server sends a late observer of the shared field for `guid`.
    fn shared_add(&self, guid: u64) -> Option<NpcSpawn> {
        (self.field.initialization(guid).unwrap().iter())
            .find(|p| p.starts_with(&[35, 0, 2, 0]))
            .map(|p| parse_add_npc(p).unwrap())
    }
}

fn system_text(replies: &[Vec<u8>]) -> String {
    (replies.iter())
        .filter(|p| p.starts_with(&[15, 0, 3, 0]) && p.len() >= 8)
        .map(|p| {
            let n = u32::from_le_bytes(p[4..8].try_into().unwrap()) as usize;
            String::from_utf8_lossy(&p[8..8 + n]).into_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The 38/12 relocation's position (`dev_teleport::location_packet`).
fn relocated(replies: &[Vec<u8>]) -> Option<[f32; 3]> {
    let p = replies.iter().find(|p| p.starts_with(&[38, 0, 12, 0]))?;
    let f = |at: usize| f32::from_le_bytes(p[at..at + 4].try_into().unwrap());
    Some([f(4), f(8), f(12)])
}

fn flat(a: [f32; 3], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[1]).powi(2)).sqrt()
}

fn entrance(id: &str) -> Value {
    let table = read(&manifest().join("data/content/entrances.json"));
    (table["entrances"].as_array().unwrap().iter())
        .find(|row| row["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("entrance {id}"))
}

/// A `field_battles.json` row's own marker: (guid, where it stands).
fn battle_marker(id: u64) -> (u64, [f32; 3]) {
    let file = read(&manifest().join("data/content/field_battles.json"));
    let row = (file["battles"].as_array().unwrap().iter())
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("battle {id}"));
    (row["spawner"]["guid"].as_u64().unwrap(), at(&row["spawner"]["position"]))
}

fn at(value: &Value) -> [f32; 3] {
    [0, 1, 2].map(|i| value[i].as_f64().unwrap() as f32)
}

fn kaboomers() -> Vec<(u64, [f32; 3])> {
    let camps = read(&manifest().join("data/content/field_camps.json"));
    let extras = read(&manifest().join("data/content/npc_spawns_extra.json"));
    let home = |guid: u64| {
        (extras["records"].as_array().unwrap().iter())
            .find(|r| r["spawn_guid"] == guid)
            .map(|r| at(&r["position"]))
            .unwrap()
    };
    (camps["camps"].as_array().unwrap().iter())
        .filter(|c| c["definition"] == "robgoblin_kaboomer")
        .flat_map(|c| c["members"].as_array().unwrap().clone())
        .map(|g| (g.as_u64().unwrap(), home(g.as_u64().unwrap())))
        .collect()
}

#[test]
fn goto_finds_the_kaboomers_the_session_handed_to_the_shared_field() {
    let kaboomers = kaboomers();
    assert!(!kaboomers.is_empty(), "the Kaboomer camp loads");
    let (_, home) = kaboomers[0];
    let mut player = Player::enter([home[0] + 40.0, home[1], home[2]], [0.0, 0.0, 1.0]);
    let (list, _) = player.chat("realm goto list kaboomer");
    for (guid, _) in &kaboomers {
        assert!(list.contains(&guid.to_string()), "{guid} is not listed: {list}");
    }
    let (guid, home) = kaboomers[3];
    let (reply, replies) = player.chat(&format!("realm goto {guid}"));
    let to = relocated(&replies).unwrap_or_else(|| panic!("no relocation: {reply}"));
    assert!(flat(to, [home[0], home[2]]) <= 6.0, "{to:?} is far from {home:?}");
}

#[test]
fn an_open_world_enemy_is_clickable_but_draws_no_press_x() {
    let player = Player::enter([-793.0, -2.0, 1016.0], [0.0, 0.0, 1.0]);
    let candidates = player.field.interest_candidates(false);
    let (mut enemies, mut starters) = (0, 0);
    for guid in candidates.iter().map(|n| n.guid) {
        let packets = player.field.initialization(guid).unwrap();
        let npc = player.shared_add(guid).unwrap();
        let cursor = (packets.iter())
            .find(|p| p.starts_with(&[35, 0, 12, 0]))
            .map(|p| (p[16], p[17]));
        if npc.disposition == 0 {
            // The client's prompt takes the nearest interactive actor in reach (`00baf6e0`).
            assert_eq!(npc.interact_range, 0, "enemy {guid} has a reach");
            assert_eq!(npc.is_interactable, 1, "enemy {guid} cannot be clicked");
            assert_eq!(cursor, Some((1, 1)), "enemy {guid}: the combat cursor (Cursors.txt 1)");
            enemies += 1;
        } else {
            assert!(npc.interact_range > 0, "starter {guid} lost its press-X");
            starters += 1;
        }
    }
    assert!(enemies > 0 && starters > 0, "{enemies} enemies, {starters} starters");
}

#[test]
fn goto_bixie_hive_lands_outside_the_hive_whichever_way_the_player_faces() {
    let door = entrance("bixie_hive");
    let (door_at, heading) = (at(&door["at"]), door["heading"].as_f64().unwrap() as f32);
    // Facing north put the arrival behind the door, in the hive body.
    for facing in [[0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]] {
        let mut player = Player::enter([door_at[0] + 60.0, door_at[1], door_at[2]], facing);
        let (reply, replies) = player.chat("realm goto Bixie Hive");
        let to = relocated(&replies).unwrap_or_else(|| panic!("no relocation: {reply}"));
        assert!(
            flat(to, HIVE_BODY) >= 6.0,
            "facing {facing:?}: {to:?} is {:.1} m from the hive body",
            flat(to, HIVE_BODY)
        );
        let ahead = (to[0] - door_at[0]) * heading.sin() + (to[2] - door_at[2]) * heading.cos();
        assert!(ahead > 4.0, "facing {facing:?}: {to:?} is not on the door's side");
    }
}

#[test]
fn the_snarler_den_and_the_spawner_huts_stand_still() {
    let table = read(&manifest().join("data/content/entrances.json"));
    let dens: Vec<(u64, [f32; 3])> = (table["entrances"].as_array().unwrap().iter())
        .filter(|row| [1913, 1914, 1938, 4546].contains(&row["model_id"].as_i64().unwrap()))
        .map(|row| (row["guid"].as_u64().unwrap(), at(&row["at"])))
        .collect();
    assert!(!dens.is_empty(), "the starters drawn as dens and huts");
    let snarler = entrance("frostfang_snarlers");
    let near = at(&snarler["at"]);
    // In sight but past the 8 m a starter stops at to face the player (`WANDER_NOTICE`).
    let mut player = Player::enter([near[0] + 20.0, near[1], near[2]], [0.0, 0.0, 1.0]);
    let me = player.gateway.field_participant().unwrap();
    let mut walked = BTreeSet::new();
    for step in 0..300_u64 {
        let now = 1_000 + step * 100;
        let (out, _) = (player.gateway)
            .tick_shared_motion(&mut player.field, std::slice::from_ref(&me), now, 100)
            .unwrap();
        for p in out.replies.iter().filter(|p| p.starts_with(&125_u16.to_le_bytes())) {
            walked.insert(u64::from_le_bytes(p[2..10].try_into().unwrap()));
        }
    }
    for (guid, home) in &dens {
        assert!(!walked.contains(guid), "starter {guid} walked");
        if let Some(npc) = player.shared_add(*guid) {
            assert_eq!(&npc.position[..3], home, "starter {guid} moved");
        }
    }
}

#[test]
fn the_troll_summoner_and_petal_each_stand_once() {
    let (_, summoner) = battle_marker(96);
    let mut player = Player::enter(summoner, [0.0, 0.0, 1.0]);
    let named = |player: &Player, name: &str| {
        (player.field.interest_candidates(false).iter())
            .filter_map(|n| player.shared_add(n.guid))
            .filter(|npc| npc.npc_name == name)
            .count()
    };
    assert_eq!(named(&player, "Troll Summoner"), 1, "one Troll Summoner is drawn");
    let (list, _) = player.chat("realm goto list troll summoner");
    assert_eq!(list.matches("Troll Summoner (").count(), 1, "{list}");
    // Petal Oceanmist (Global.Text 394904): the seaside row the Kart Driver quests name.
    let (list, _) = player.chat("realm goto list petal oceanmist");
    assert_eq!(list.matches("Petal Oceanmist (").count(), 1, "{list}");
    // The server's own NPC fan-out for a shared session (`Gameplay::shared_npc_view`).
    let petal = Player::enter([-241.1, 1.2, -62.7], [0.0, 0.0, 1.0]);
    let fields = petal.field.interest_candidates(false);
    let (_, view) = (petal.gateway.gameplay().unwrap())
        .shared_npc_view(petal.gateway.state(), &fields, &BTreeSet::new())
        .unwrap();
    let drawn = (view.iter())
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .filter_map(|p| parse_add_npc(p).ok())
        .filter(|npc| npc.name_string_id == 394_904)
        .map(|npc| npc.actor_id)
        .collect::<Vec<_>>();
    assert_eq!(drawn, vec![9_500_000_613], "Petal is drawn once");
}

#[test]
fn every_hit_on_the_troll_summoner_moves_its_bar_with_the_swing_alone() {
    let (guid, summoner) = battle_marker(96);
    let mut player = Player::enter(summoner, [0.0, 0.0, 1.0]);
    let arena = player.battle(guid);
    let boss = (arena.iter())
        .filter(|p| p.starts_with(&[35, 0, 2, 0]))
        .filter_map(|p| parse_add_npc(p).ok())
        .find(|npc| npc.npc_name == "Troll Summoner" || npc.name_string_id == 15_623)
        .expect("the Troll Summoner spawns in the arena");
    // The spawn's statement sets the client's copy the 32/7s then take from.
    let statement = |p: &Vec<u8>| {
        p.starts_with(&[35, 0, 35, 0]) && p[12..20] == boss.actor_id.to_le_bytes()
    };
    assert!(arena.iter().any(statement), "the spawn sends the boss bar its statement");
    let mut hits = 0;
    for _ in 0..40 {
        let state = player.gateway.snapshot().gameplay.unwrap().encounter.unwrap();
        let enemies = state.active.unwrap().enemies;
        if !(enemies.iter()).any(|e| e.guid == boss.actor_id && e.health > 0) {
            break;
        }
        // The helpers the summoner calls in are cleared first, as a player would (rc-0136).
        let Some((target, at)) = (enemies.iter())
            .filter(|e| e.health > 0 && !e.dormant)
            .min_by_key(|e| e.guid == boss.actor_id)
            .map(|e| (e.guid, e.position))
        else {
            break;
        };
        player.walk([at[0] + 1.5, at[1], at[2]]);
        player.now += 1_500;
        let replies = player.send(&melee(target));
        let swings = (replies.iter())
            .filter(|p| p.starts_with(&[32, 0, 7, 0]) && p[20..28] == boss.actor_id.to_le_bytes())
            .count();
        let statements = replies.iter().filter(|p| statement(p)).count();
        assert_eq!(statements, 0, "a hit sends no statement of its own");
        hits += swings;
    }
    // More than one swing, each moving the bar alone (two at the natural curve since the owner's
    // multipliers went back to 1.0, 2026-11-13; three under rc-0137's x2 health).
    assert!(hits >= 2, "{hits} hits landed");
}

/// No fight dropped a power-up (owner). The Hooligan Bullies' pacing drops a heart on a kill
/// (`instance_pacing.json`), and its fighters drop by their rank (`power_ups.json`); the heart
/// reaches the client as a 35/2 pickup actor in the reply to the killing hit.
#[test]
fn a_hooligan_kill_drops_a_heart_the_client_is_sent() {
    let starter = entrance("hooligan_bullies");
    let mut player = Player::enter(at(&starter["at"]), [0.0, 0.0, 1.0]);
    player.battle(starter["guid"].as_u64().unwrap());
    let mut drops = Vec::new();
    for _ in 0..200 {
        player.now += 500;
        player.gateway.tick_at(player.now).unwrap();
        let state = player.gateway.snapshot().gameplay.unwrap().encounter.unwrap();
        let Some((guid, at)) = (state.active.unwrap().enemies.iter())
            .find(|e| e.health > 0 && !e.dormant)
            .map(|e| (e.guid, e.position))
        else {
            continue;
        };
        player.walk([at[0] + 1.5, at[1], at[2]]);
        player.now += 1_500;
        let replies = player.send(&melee(guid));
        drops.extend(
            (replies.iter())
                .filter(|p| p.starts_with(&[35, 0, 2, 0]))
                .filter_map(|p| parse_add_npc(p).ok())
                .filter(|npc| npc.actor_id >> 32 == 0x5057_5550),
        );
        if !drops.is_empty() {
            break;
        }
    }
    let heart = drops.first().expect("a kill dropped a power-up");
    let kinds = read(&manifest().join("data/content/power_ups.json"));
    let models: BTreeSet<i64> = (kinds["kinds"].as_object().unwrap().values())
        .filter_map(|k| k["model_id"].as_i64())
        .collect();
    assert!(models.contains(&i64::from(heart.model_id)), "{heart:?}");
}
