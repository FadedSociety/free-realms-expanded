//! The release-candidate combat fixes (owner check rc-0136, fix group F2), through the shipped
//! content and the real engine paths:
//! - the boss bar (`BaseClient.Bosses`, cell getter `00a79eb0`) reads the actor's current and
//!   maximum health, which only 35/35 and 32/7 set (`008ba940`); every enemy health statement
//!   now carries the 35/35 beside the 35/5 nameplate bar;
//! - the results card is the run's own (kills, time, the knockout left, the goal);
//! - battle 96 fights at the 2014 numbers of its tier;
//! - an open-world enemy sends the compact combat marker live sent, not the battle starter's
//!   named one (the Kaboomers "show the interact ui for instance enemies");
//! - nothing that fights is drawn as a prop or spawner model or a model with no skeleton,
//!   while a battle starter may be (the Frostfang Snarler's den is the 2014 starter; its
//!   battle's wolves are bodies), and every drawn spawner hut starts its battle or says why;
//! - `realm goto` arrivals at entrances and battles stand on the ground (Bixie Hive), by the
//!   one rule enemies follow, which finds every shipped enemy home at rest (F6's floor);
//! - the Bone Bog Cemetery snakes start their battle instead of fighting in the open world.
//!
//! The model and terrain tests read the client's model table and the terrain bake from the
//! lab, like `content_enemies`; run them on this PC (`BUILD_SLOT_LOCAL=1`).
#[path = "support/models.rs"]
mod models;

use osfr_tools::{
    character_store::CharacterState,
    dev_teleport,
    encounter::EncounterEngine,
    encounter::EncounterPhase,
    npc_life::SETTLED_M,
    terrain::{MAX_SNAP, Terrain},
};
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

const BRAWLER: i32 = 43;
const SUMMONER: u64 = 100_000_096_001;
/// The Bone Bog Cemetery snake that starts Snakes in a Maze!, and the four beside it that are
/// scenery again.
const ASP: u64 = 100_000_004_419;
const BONE_BOG: [u64; 4] = [
    100_000_001_597,
    100_000_004_416,
    100_000_004_417,
    100_000_004_418,
];

fn content() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/content")
}

fn json(rel: &str) -> Value {
    serde_json::from_slice(&fs::read(content().join(rel)).unwrap()).unwrap()
}

/// The shipped engine as `Gameplay::load` builds it: behaviours, field enemies, camps, battles.
fn engine() -> EncounterEngine {
    let dir = content();
    let mut engine = EncounterEngine::load(&dir, None).unwrap_or_else(|e| panic!("{e}"));
    engine
        .load_behaviours(&dir.join("behaviours.json"))
        .unwrap();
    engine
        .load_field_enemies(&dir.join("field_enemies.json"))
        .unwrap();
    engine.load_field_camps(&dir).unwrap();
    engine
        .load_field_battles(&dir.join("field_battles.json"))
        .unwrap();
    engine
}

/// The lab terrain bake as the gateway loads it: the height grids and the baked collision.
fn terrain() -> &'static Terrain {
    static TERRAIN: OnceLock<Terrain> = OnceLock::new();
    TERRAIN.get_or_init(|| {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/terrain");
        Terrain::load(&dir).expect("the lab terrain bake")
    })
}

fn home() -> CharacterState {
    CharacterState {
        guid: 1,
        zone_id: 0,
        position: [-793.0, -2.0, 1016.0],
        rotation: [0.0, 0.0, 1.0],
    }
}

fn i32_at(b: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

fn packets<'a>(replies: &'a [Vec<u8>], op: u16, sub: u16) -> impl Iterator<Item = &'a Vec<u8>> {
    let head = [op.to_le_bytes(), sub.to_le_bytes()].concat();
    replies.iter().filter(move |p| p.starts_with(&head))
}

/// (max, current) of the 35/35 health statement about `guid` (target at +12).
fn statement(replies: &[Vec<u8>], guid: u64) -> Option<(i32, i32, i32)> {
    packets(replies, 35, 35)
        .filter(|p| u64_at(p, 12) == guid)
        .map(|p| (i32_at(p, 21), i32_at(p, 25), i32_at(p, 29)))
        .last()
}

fn enemies(engine: &EncounterEngine) -> Vec<osfr_tools::encounter::EnemyState> {
    engine.state().active.as_ref().unwrap().enemies.clone()
}

/// Battle 96 entered and loaded at `now`.
fn battle_96(now: u64) -> (EncounterEngine, Vec<Vec<u8>>) {
    let mut engine = engine();
    engine.begin(96, home(), BRAWLER).unwrap();
    let ready = engine.on_zone_ready(BRAWLER, now).unwrap();
    (engine, ready.replies)
}

#[test]
fn every_hit_sends_the_health_statement_the_boss_bar_reads() {
    let (mut engine, ready) = battle_96(0);
    // Spawn: AddNpc, the nameplate bar, the statement, then the boss bar's registration.
    let at = |sub: u16| {
        ready
            .iter()
            .position(|p| p.starts_with(&[35, 0, sub as u8, 0]) && u64_at(p, 4) == SUMMONER)
    };
    let (add, bar) = (at(2).unwrap(), at(5).unwrap());
    let boss = (ready.iter())
        .position(|p| p.starts_with(&[32, 0, 9, 0]) && u64_at(p, 4) == SUMMONER)
        .unwrap();
    let said = (ready.iter())
        .position(|p| p.starts_with(&[35, 0, 35, 0]) && u64_at(p, 12) == SUMMONER)
        .unwrap();
    assert!(
        add < bar && bar < said && said < boss,
        "{add} {bar} {said} {boss}"
    );
    let max = statement(&ready, SUMMONER).unwrap().0;
    assert_eq!(statement(&ready, SUMMONER), Some((max, max, 0)));
    let spot = enemies(&engine)[0].position;
    let hit = engine.attack(SUMMONER, spot, BRAWLER, 0, 1_000).unwrap();
    let now = enemies(&engine)[0].health as i32;
    assert!(now < max, "the hit landed");
    // rc-0137: the 32/7 alone moves the boss bar, in step with the swing: it takes its damage
    // off the copy the spawn's statement set (`00a2ba40`) and carries the health left (+46).
    // A statement per hit as well came on the summoner's own animation clock (`008d3020`) and
    // made the bar tick sporadically (owner), so the hit sends none; the 35/5 moves the plate.
    let swing = (hit.replies.iter())
        .find(|p| p.starts_with(&[32, 0, 7, 0]) && u64_at(p, 20) == SUMMONER)
        .unwrap();
    assert_eq!(i32_at(swing, 46), now, "the 32/7 carries the health the bar shows");
    assert!(packets(&hit.replies, 35, 5).any(|p| u64_at(p, 4) == SUMMONER));
    assert_eq!(statement(&hit.replies, SUMMONER), None, "no statement per hit");
}

#[test]
fn battle_96_fights_at_the_numbers_of_its_tier() {
    // The tier's numbers times the owner's multipliers (`combat_scale.json`: enemy health x2,
    // enemy hits x1.5, rc-0137): the Summoner rank tough, its helpers rank minion.
    let (mut engine, ready) = battle_96(0);
    let summoner = statement(&ready, SUMMONER);
    let spot = enemies(&engine)[0].position;
    // Below 70 % the first pair of helpers rises.
    let (mut hit, mut raised) = (None, Vec::new());
    for n in 0..12 {
        hit = Some(engine.attack(SUMMONER, spot, BRAWLER, 0, 1_000 + n * 700).unwrap());
        raised = (enemies(&engine).iter())
            .filter(|e| e.guid != SUMMONER && !e.dormant && e.health > 0)
            .map(|e| e.guid)
            .collect();
        if !raised.is_empty() {
            break;
        }
    }
    let hit = hit.unwrap();
    assert_eq!(
        raised,
        [100_000_096_002, 100_000_096_003],
        "two Scoutlings at once"
    );
    let helpers: Vec<_> = raised.iter().map(|&g| statement(&hit.replies, g)).collect();
    // Every one of them bites for the tier's raw hit, once every 2.3 s.
    let mut bites = Vec::new();
    let mut now = 1_000 + 12 * 700;
    while now < 30_000 {
        now += 250;
        let out = engine.tick(spot, BRAWLER, now).unwrap();
        for p in packets(&out.replies, 32, 7) {
            bites.push((u64_at(p, 4), i32_at(p, 28), now));
        }
    }
    let raw: BTreeSet<i32> = bites.iter().map(|b| b.1).collect();
    assert_eq!(
        (summoner, helpers, raw),
        (
            Some((3_453, 3_453, 0)),
            vec![Some((520, 520, 0)); 2],
            BTreeSet::from([38])
        ),
        "rank tough and rank minion at tier 1, x2 health and x1.5 hits"
    );
    // Paced (`instance_pacing.json` 96): one enemy swings at a time, each at most every 2.3 s.
    assert!(!bites.is_empty(), "they fight");
    for w in bites.windows(2) {
        assert!(w[0].2 != w[1].2, "two swings in one tick: {w:?}");
    }
    for guid in bites.iter().map(|b| b.0).collect::<BTreeSet<_>>() {
        let mine: Vec<_> = bites.iter().filter(|b| b.0 == guid).collect();
        assert!(mine.windows(2).all(|w| w[1].2 - w[0].2 >= 2_250), "{mine:?}");
    }
}

#[test]
fn the_results_card_is_the_runs_own() {
    let (mut engine, _) = battle_96(10_000);
    let mut now = 11_000;
    let mut last = None;
    // The paced rounds (`instance_pacing.json`) come back on the engine's tick; the player is
    // kept standing so the card is the win's.
    for _ in 0..3_000 {
        engine.heal_player_percent(100);
        let tick = engine.tick(enemies(&engine)[0].position, BRAWLER, now).unwrap();
        if engine.state().active.as_ref().unwrap().phase == EncounterPhase::Won {
            last = Some((tick, now));
            break;
        }
        let alive: Vec<_> = (enemies(&engine).into_iter())
            .filter(|e| e.health > 0 && !e.dormant)
            .collect();
        // The Summoner hides behind its helpers: clear them first.
        let Some(target) = (alive.iter().find(|e| e.guid != SUMMONER))
            .or_else(|| alive.iter().find(|e| e.guid == SUMMONER))
        else {
            now += 700; // the gap before the next round
            continue;
        };
        let out = engine
            .attack(target.guid, target.position, BRAWLER, 0, now)
            .unwrap();
        if engine.state().active.as_ref().unwrap().phase == EncounterPhase::Won {
            last = Some((out, now));
            break;
        }
        now += 700;
    }
    let (won, at) = last.unwrap_or_else(|| {
        let phase = &engine.state().active.as_ref().unwrap().phase;
        panic!("battle 96 is not won through the attack path: {phase:?}")
    });
    // 39/47: a one-byte sub-opcode (`wire::minigame`).
    let card = (won.replies.iter()).filter(|p| p.starts_with(&[39, 0, 47]))
        .next()
        .expect("the results card");
    let mut rows = Vec::new();
    let mut cursor = 19;
    for _ in 0..i32_at(card, 15) {
        let len = i32_at(card, cursor) as usize;
        let name = String::from_utf8(card[cursor + 4..cursor + 4 + len].to_vec()).unwrap();
        cursor += 4 + len;
        let fields: Vec<i32> = (0..5).map(|i| i32_at(card, cursor + 4 * i)).collect();
        cursor += 20;
        rows.push((name, fields));
    }
    assert_eq!(cursor, card.len());
    let row = |name: &str| (rows.iter().find(|r| r.0 == name)).unwrap().1.clone();
    let kills = enemies(&engine).iter().filter(|e| e.health == 0).count() as i32;
    assert!(kills >= 3, "the helpers and the Summoner");
    let seconds = ((at - 10_000) / 1_000) as i32;
    let bonus = 100_000 - 500 * (seconds - 1).max(0);
    assert_eq!(row("scoreEnemiesDefeated"), [-1, 0, kills, -1, kills * 300]);
    assert_eq!(row("scoreTimeBonus"), [-1, 2, seconds, -1, bonus]);
    assert_eq!(row("scorePlayerKnockouts"), [-1, 1, 1, 1, 25_000]);
    assert_eq!(row("scoreObjectives"), [-1, 3, 1, 1, 5_000]);
    let total = kills * 300 + bonus + 25_000 + 5_000;
    assert_eq!(row("scoreTotalScore"), [-1, 4, -1, -1, total]);
    assert_ne!(total, 5_000, "no longer the fixed award");
}

#[test]
fn open_world_enemies_send_the_compact_marker_live_sent() {
    let mut engine = engine();
    let world = engine.on_zone_ready(BRAWLER, 0).unwrap().replies;
    let kaboomers: BTreeSet<u64> = (json("field_camps.json")["camps"].as_array().unwrap().iter())
        .filter(|c| c["definition"] == "robgoblin_kaboomer")
        .flat_map(|c| {
            c["members"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| g.as_u64().unwrap())
        })
        .collect();
    assert_eq!(kaboomers.len(), 8);
    let markers: Vec<&Vec<u8>> = packets(&world, 35, 10)
        .filter(|p| kaboomers.contains(&u64_at(p, 8)))
        .collect();
    assert_eq!(markers.len(), kaboomers.len(), "one marker each");
    for p in &markers {
        // count 1, guid, compact 1, kind 3 (Combat), the trailing byte: 22 bytes, as captured.
        assert_eq!((p.len(), i32_at(p, 4), p[16], i32_at(p, 17)), (22, 1, 1, 3));
    }
    // The battle starters keep the full, named record (their label and hover panel).
    let (_, starter) = engine.battle_markers().into_iter().next().unwrap();
    assert!(starter.image_id > 0 && starter.name_id > 0);
    // A late observer of the shared field gets the same compact row.
    let shared = engine.take_shared_fields().unwrap();
    let late = shared.initialization(*kaboomers.first().unwrap()).unwrap();
    assert_eq!(packets(&late, 35, 10).next().unwrap()[16], 1);
}

/// Model-table names that are not a creature's body.
fn not_a_body(file: &str) -> bool {
    let f = file.to_ascii_lowercase();
    [
        "spawner", "_prop", "house", "_den_", "_den.", "sign", "door", "gate", "chest", "crate",
    ]
    .iter()
    .chain(&["barrel", "tent", "invisible", "cube"])
    .any(|m| f.contains(m))
}

/// Whether a client model file has a skeleton, as the Studio's model check reads it ("has no
/// skeleton"): an `.adr` that names a `.dsk`, or an `.agr` actor set with such a member. `None`:
/// the lab's pack catalog lacks the file (one the community host or the launcher's full install
/// supplies, such as the opening's `gloam_m_construct.adr`); the Studio reads those on an install.
fn skeletons() -> impl Fn(&str) -> Option<bool> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../lab/extracted/client-assets");
    let catalog = fs::read_to_string(root.join("catalog.tsv")).expect("the pack catalog");
    let packed: HashMap<String, PathBuf> = (catalog.lines().skip(1))
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('\t').collect();
            Some((cells.get(2)?.to_ascii_lowercase(), root.join(cells.get(6)?)))
        })
        .collect();
    let read = move |file: &str| fs::read(packed.get(&file.to_ascii_lowercase())?).ok();
    let names_a_skeleton = |adr: &[u8]| adr.windows(4).any(|w| w.eq_ignore_ascii_case(b".dsk"));
    move |file: &str| {
        let bytes = read(file)?;
        if !file.to_ascii_lowercase().ends_with(".agr") {
            return Some(names_a_skeleton(&bytes));
        }
        let set = String::from_utf8_lossy(&bytes).into_owned();
        let members = (set.split("name=\"").skip(1)).filter_map(|cell| cell.split('"').next());
        Some(
            members
                .filter(|member| member.to_ascii_lowercase().ends_with(".adr"))
                .any(|member| read(member).is_some_and(|adr| names_a_skeleton(&adr))),
        )
    }
}

/// The coordinator's audit: everything that fights wears a creature's body, a model with a
/// skeleton (the Studio found battle 12310's Yetis on the banner `sh_standard_yeti_01.adr`,
/// which has none). A battle starter may be a prop, as 2014 drew the Frostfang Snarlers' den
/// (video 5-IM2pBh2yY, frames 01048 and 01080) and the Frosty Yeti's banner, and so may a door.
#[test]
fn no_creature_is_drawn_as_a_prop_or_spawner_model() {
    let models = models::read_models();
    let file = |id: i64| models.get(&id).map(|n| n.iter().next().unwrap().clone());
    let skeleton = skeletons();
    let mut bad = Vec::new();
    let mut check = |what: String, model: i64| {
        let fault = match file(model) {
            None => "is in no model table row".to_string(),
            Some(name) if not_a_body(&name) => format!("{name} is a prop's file"),
            Some(name) => match skeleton(&name) {
                Some(true) => return,
                Some(false) => format!("{name} has no skeleton"),
                None => return,
            },
        };
        bad.push(format!("{what}: model {model} {fault}"));
    };
    let definitions = json("enemies.json");
    // A definition is checked where it fights: the fights' kinds, the battle rows, the field.
    for f in fs::read_dir(content().join("fights")).unwrap().flatten() {
        let fight: Value = serde_json::from_slice(&fs::read(f.path()).unwrap()).unwrap();
        let fought = serde_json::to_string(&(&fight["actors"], &fight["waves"])).unwrap();
        for (key, kind) in fight["kinds"].as_object().unwrap() {
            let object = kind["rank"] == "object";
            if !object && fought.contains(&format!("\"{key}\"")) {
                check(
                    format!("{:?} kind {key}", f.file_name()),
                    kind["model_id"].as_i64().unwrap(),
                );
            }
        }
    }
    for b in json("field_battles.json")["battles"].as_array().unwrap() {
        for a in b["enemies"].as_array().unwrap() {
            check(
                format!("battle {} {}", b["id"], a["name"]),
                a["model_id"].as_i64().unwrap(),
            );
        }
    }
    // A quest-gated field row is drawn while one of its quests is live; one whose quests are all
    // off (the Snarler den row of 3150002) waits for its quest's own wiring.
    let mut live = BTreeSet::new();
    for f in fs::read_dir(content().join("quests")).unwrap().flatten() {
        let file: Value = serde_json::from_slice(&fs::read(f.path()).unwrap()).unwrap();
        for q in file["records"].as_array().into_iter().flatten() {
            if q["enabled"] == true {
                live.insert(q["id"].as_u64().unwrap());
            }
        }
    }
    for r in json("field_enemies.json")["enemies"].as_array().unwrap() {
        let quests = r["quest_ids"].as_array().unwrap();
        if quests.is_empty() || quests.iter().any(|q| live.contains(&q.as_u64().unwrap())) {
            check(
                format!("field enemy {}", r["guid"]),
                r["model_id"].as_i64().unwrap(),
            );
        }
    }
    // What the engine really draws for every open-world enemy, as its AddNpc leaves the
    // server (camp bodies wear their kind's model). The Snarlers' starter is the den.
    let mut engine = engine();
    let den = (engine.entrance_spawners().into_iter())
        .find(|s| s.1 == "Frostfang Snarler")
        .expect("the Frostfang Snarlers starter");
    assert_eq!((den.0, den.3), (100_000_003_592, 1938), "the Legacy den, as a starter");
    let field: BTreeSet<u64> = engine.field_guids().into_iter().collect();
    for p in packets(&engine.on_zone_ready(BRAWLER, 0).unwrap().replies, 35, 2) {
        if field.contains(&u64_at(p, 4)) {
            check(
                format!("field enemy {}", u64_at(p, 4)),
                i64::from(i32_at(p, 16)),
            );
        }
    }
    // NPC rows named after an enemy kind and drawn as themselves (a camp's body is drawn by the
    // camp, and an entrance takes over the body it stands for, as the Snarlers' den 3592). A
    // copy of a battle starter's prop is the starter's look, which the hut test below checks.
    let kinds: BTreeSet<&str> = (definitions["definitions"].as_array().unwrap().iter())
        .filter_map(|d| d["kind"].as_str())
        .filter(|kind| !kind.ends_with('!'))
        .collect();
    let starters = starter_names();
    let extras = json("npc_spawns_extra.json");
    for r in extras["records"].as_array().unwrap() {
        let guid = r["spawn_guid"].as_u64().unwrap();
        let name = r["name"].as_str().unwrap_or_default();
        let prop = file(r["model_id"].as_i64().unwrap()).is_some_and(|f| skeleton(&f) == Some(false));
        let named = kinds.contains(name) && !(starters.contains(name) && prop);
        if named && !field.contains(&guid) && engine.entrance(guid).is_none() {
            check(
                format!("npc_spawns_extra {guid} {}", r["name"]),
                r["model_id"].as_i64().unwrap(),
            );
        }
    }
    assert!(
        bad.is_empty(),
        "{} creature rows wear no body:\n{}",
        bad.len(),
        bad.join("\n")
    );
    // The two rows the captures fix outright.
    let model = |id: &str| {
        (definitions["definitions"].as_array().unwrap().iter())
            .find(|d| d["id"] == id)
            .unwrap()["model_id"]
            .as_i64()
    };
    assert_eq!(model("frostfang_growler"), Some(176), "live: wolf.adr");
    assert_eq!(model("yeti"), Some(51), "(R) sasquatch_01.adr, the fork's wave Yeti");
    assert_eq!(
        model("robgoblin_kaboomer"),
        Some(1682),
        "live: robgoblin_m_master.adr"
    );
    assert_eq!(model("frostfang_snarler"), Some(1938), "the starter's den");
    let snarlers: Value = json("fights/14055.json");
    assert_eq!(
        snarlers["kinds"]["frostfang_snarler"]["model_id"],
        177,
        "(R) the wolves the Snarlers battle's waves fight with"
    );
}

/// Every `realm goto` to an entrance or a battle arrives on its floor: the shared rule
/// (`Terrain::snapped`) lifts the Bixie Hive arrival out of the hillside, and a cave door far
/// below the surface keeps its height.
#[test]
fn goto_entrances_and_battles_arrives_on_the_ground() {
    let terrain = terrain();
    let engine = engine();
    let targets = engine.field_teleport_targets();
    let starters: Vec<_> = targets.iter().filter(|t| t.kind == "battle").collect();
    assert!(starters.len() >= 60, "{}", starters.len());
    let mut lifted = Vec::new();
    for target in &starters {
        let character = CharacterState {
            position: [
                target.position[0] - 30.0,
                target.position[1],
                target.position[2],
            ],
            ..home()
        };
        let Some(dev_teleport::Command::Relocate { relocation, .. }) =
            dev_teleport::handle(&format!("goto {}", target.guid), &targets, &character)
        else {
            panic!("{} relocates", target.name);
        };
        let landed = relocation.clone().on_ground(Some(terrain), 0);
        let (from, y) = (relocation.position, landed.position[1]);
        let ground = terrain.height_at(0, from[0], from[2]);
        let floor = terrain.floor_at(0, landed.position);
        let stands = floor.is_some_and(|floor| (floor - y).abs() <= SETTLED_M);
        let deep = ground.is_none_or(|ground| (ground - from[1]).abs() > MAX_SNAP);
        assert!(
            stands || (deep && y == from[1]),
            "{} lands at {y} from {from:?}: floor {floor:?}, ground {ground:?}",
            target.name
        );
        if y - from[1] > 2.0 {
            lifted.push(target.name.clone());
        }
    }
    assert!(lifted.iter().any(|n| n == "Bixie Hive"), "{lifted:?}");
}

/// F6 set the shipped bodies on their floor (`Terrain::floor_at`), the two Highroad Hooligan
/// homes it lowered among them. The encounter's ground rule (`Terrain::snapped`, which every
/// home, step and arrival goes through) finds every field enemy and camp home on its floor at
/// rest and moves none. Only a body `npc_poses.json` keeps above its floor with evidence (a
/// flyer, a Legacy spawn-script height) has its leash home on the floor below: it is drawn at
/// its own height and comes back to the floor after a chase, as under the old grid rule.
#[test]
fn the_ground_rule_moves_no_shipped_enemy_home() {
    const LOWERED_BY_F6: [u64; 2] = [5_064_655_448_674_664_449, 5_064_655_448_674_664_450];
    let at = |row: &Value| -> [f32; 3] {
        let p = row["position"].as_array().expect("a position");
        [0, 1, 2].map(|i| p[i].as_f64().unwrap() as f32)
    };
    let guid = |row: &Value, key: &str| row[key].as_u64().unwrap();
    let mut homes: Vec<(u64, [f32; 3])> = (json("field_enemies.json")["enemies"].as_array())
        .unwrap()
        .iter()
        .map(|row| (guid(row, "guid"), at(row)))
        .collect();
    let members: BTreeSet<u64> = (json("field_camps.json")["camps"].as_array().unwrap().iter())
        .flat_map(|camp| camp["members"].as_array().unwrap().iter())
        .filter_map(Value::as_u64)
        .collect();
    homes.extend(
        (json("npc_spawns_extra.json")["records"].as_array().unwrap().iter())
            .filter(|row| members.contains(&guid(row, "spawn_guid")))
            .map(|row| (guid(row, "spawn_guid"), at(row))),
    );
    assert!(homes.len() >= 150, "{} field enemy and camp homes", homes.len());
    for lowered in LOWERED_BY_F6 {
        assert!(homes.iter().any(|h| h.0 == lowered), "{lowered} is a home");
    }
    let terrain = terrain();
    let life = osfr_tools::npc_life::Life::load(&content()).unwrap();
    let (mut kept, mut moved) = (0, Vec::new());
    for &(guid, p) in &homes {
        let to = terrain.snapped(0, p);
        if to == p {
            continue;
        }
        if life.keeps_height(guid) {
            kept += 1;
        } else {
            moved.push(format!("{guid} at {p:?} to {}", to[1]));
        }
    }
    println!("{} homes at rest, {kept} kept above their floor", homes.len() - kept);
    assert!(moved.is_empty(), "{} homes move:
{}", moved.len(), moved.join("
"));
}

#[test]
fn the_bone_bog_snakes_start_their_battle_and_fight_no_more_in_the_open() {
    let engine = engine();
    let field: BTreeSet<u64> = engine.field_guids().into_iter().collect();
    let spawners = engine.entrance_spawners();
    for guid in BONE_BOG {
        assert!(!field.contains(&guid), "{guid} is no field enemy");
        assert!(engine.entrance(guid).is_none(), "{guid} is scenery");
    }
    {
        let guid = ASP;
        assert!(!field.contains(&guid), "{guid} is no field enemy");
        let (encounter, open) = engine.entrance(guid).expect("a battle starter");
        assert!(open, "{guid} opens its fight");
        let (_, name, _, model, at, _) = spawners.iter().find(|s| s.0 == guid).unwrap();
        assert_eq!((name.as_str(), *model), ("Amazing Asp", 180));
        // The click opens the start card (41/114), never a fight in the open world.
        let mut click = vec![26, 0, 8, 0];
        click.extend(guid.to_le_bytes());
        click.push(0);
        let out = engine
            .clone()
            .handle(&click, 1, *at, BRAWLER, 1_000)
            .unwrap();
        let first = &out.replies[0];
        assert_eq!(
            (&first[..4], i32_at(first, 4)),
            (&[41, 0, 114, 0][..], encounter as i32)
        );
    }
    // Other snakes elsewhere keep their camp.
    assert!(field.contains(&100_000_006_251));
}

/// The names the entrances' starters and doors are drawn with.
fn starter_names() -> BTreeSet<String> {
    (json("entrances.json")["entrances"].as_array().unwrap().iter())
        .map(|row| row["name"].as_str().unwrap().to_string())
        .collect()
}

/// The Studio's model check: every drawn starter prop is a battle's starter. The nine Legacy
/// huts on `sg_spawner_robgoblin_01` and `sg_spawner_human_01` are the starters the fork's
/// `Battles.json` names (`StarterNpcIds` 3564 to 4499, reference spawns), not scenery, and the
/// Legacy Ice Troll Scout hut 3593 (no skeleton) is a second spot of its battle's starter. Each
/// opens its battle, or waits in `entrances.json` `held` as proven, with its guid and the
/// reason (no fight file yet, a scene no client pack holds, or a battle that has its starter).
#[test]
fn every_drawn_spawner_hut_starts_its_battle_or_says_why() {
    let models = models::read_models();
    let skeleton = skeletons();
    let starters = starter_names();
    let engine = engine();
    let held: Vec<String> = (json("entrances.json")["held"].as_array().unwrap().iter())
        .map(|h| h["reason"].as_str().unwrap().to_string())
        .filter(|reason| reason.starts_with("proven, held: "))
        .collect();
    let (mut open, mut waiting, mut bad) = (Vec::new(), Vec::new(), Vec::new());
    for r in json("npc_spawns_extra.json")["records"].as_array().unwrap() {
        let model = r["model_id"].as_i64().unwrap();
        let files = models.get(&model).cloned().unwrap_or_default();
        let spawner = files.iter().any(|f| f.contains("spawner"));
        let named = r["name"].as_str().is_some_and(|n| starters.contains(n));
        let prop = files.iter().any(|f| skeleton(f) == Some(false));
        if !spawner && !(named && prop) {
            continue;
        }
        let guid = r["spawn_guid"].as_u64().unwrap();
        match engine.entrance(guid) {
            Some((_, true)) => open.push(guid),
            _ if held.iter().any(|reason| reason.contains(&guid.to_string())) => waiting.push(guid),
            found => bad.push(format!("{guid} model {model}: {found:?}")),
        }
    }
    assert!(bad.is_empty(), "spawner huts that start no battle:\n{}", bad.join("\n"));
    // The Snarlers' den, Hooligan Bullies!, Robgoblin Troublemakers!, Explosive Robgoblins!,
    // Robgoblin Adept Trouble!, Robgoblin Geomancer!, the Ice Troll Scout's hut and the Frostfang
    // Caverns door open, and (L1192) Hooligan Brawling Club!; Band of Robgoblins!, Robgoblin Camp!,
    // Robgoblin Pondblasters! (its scene's .gzne is in no pack) and the second Ice Troll
    // Scout hut wait.
    open.sort_unstable();
    waiting.sort_unstable();
    assert_eq!(
        open,
        [
            100_000_003_565,
            100_000_003_592,
            100_000_004_099,
            100_000_004_110,
            100_000_004_156,
            100_000_004_162,
            100_000_004_303,
            100_000_033_822,
            100_000_033_823
        ]
    );
    assert_eq!(
        waiting,
        [
            100_000_003_564,
            100_000_003_593,
            100_000_004_102,
            100_000_004_499
        ]
    );
}
